//! Native recursion for a self-recursive function.
//!
//! The function is emitted twice. The frames version keeps the suspended callers of a recursion in the control arena,
//! so it never nests native calls. The native version, which carries the function's own name, nests a native call
//! for each recursive call and hands the whole activation to the frames version at its entry once the native stack
//! is used up. A native activation keeps its live values in its own slots and touches no control arena, so the
//! optimizer sees an ordinary recursive function whose only extra work is one stack check per activation.

use crate::backend::llvm::syntax::{ComparisonKind, ComparisonPredicate, llvm_type};
use crate::closure::ast::Atom;
use crate::control::ast::{StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::super::{EmissionMode, EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    /// Whether the function can also be emitted as a native version: every framed recursion edge targets the
    /// function itself, so the same body can use native calls until it reaches the stack budget.
    pub(in crate::backend::llvm::body) fn has_native_version(&self) -> bool {
        self.mode == EmissionMode::Standard
            && self.common_region.is_none()
            && !self.frame_sites.is_empty()
            && self
                .execution
                .native_recursion
                .has_native_version(self.function.id)
    }

    /// Turns a standard emitter into the frames version, which is emitted under another name.
    pub(in crate::backend::llvm::body) fn into_frames_version(mut self) -> Self {
        self.mode = EmissionMode::Frames;
        self
    }

    /// Turns a standard emitter into the native version, which no frame belongs to.
    pub(in crate::backend::llvm::body) fn into_native_version(mut self) -> Self {
        self.mode = EmissionMode::Native;
        self.frame_sites.clear();
        self.frame_tags.clear();
        self.local_control_top = false;
        self.local_control_storage = false;
        self
    }

    pub(in crate::backend::llvm::body) fn emit_native_self_call(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
    ) -> Option<()> {
        let Terminator::Call { resume, .. } = &self.control.states[site.0].terminator else {
            return None;
        };
        let resume = *resume;
        self.require_terminator_borrow(
            site,
            crate::execution::ownership::TerminatorOperand::CallCallee,
            callee,
        )?;
        let callee = self.atom(callee)?;
        let environment = self.closure_environment(&callee)?;
        // The callee's entry keeps its own reference to a managed argument, so a reference this call site takes for the
        // ownership plan's handoff (a share or a move) is released once the call returns.
        let mut handed_over = None;
        let mut arguments = vec![
            (llvm_type!(ptr), "%mal_context".into()),
            (llvm_type!(ptr), "%mal_control_top".into()),
            (llvm_type!(ptr), environment),
        ];
        if self.function.parameter.ty != Type::Unit {
            let argument = if crate::execution::ownership::is_managed(&argument.ty) {
                let effect = self.ownership.terminator_use(
                    site,
                    crate::execution::ownership::TerminatorOperand::CallArgument,
                )?;
                if effect == crate::execution::ownership::UseEffect::Borrow {
                    self.atom(argument)?
                } else {
                    let prepared = self.prepare_atom_for_use(argument, effect)?;
                    self.commit_consumes(&prepared)?;
                    handed_over = Some(prepared.value.clone());
                    prepared.value
                }
            } else {
                self.atom(argument)?
            };
            let argument_type = self.types.value(&argument.ty)?;
            arguments.push((argument_type.llvm, argument.representation));
        }
        let result_type = self.current_result_type()?;
        let result_llvm = self.types.value(&result_type)?.llvm;
        let name = super::super::function_name(self.function.id);
        let register = self.register();
        emit_instruction!(
            self;
            call { Some(register.clone()) },
            false,
            { result_llvm },
            direct { name },
            {{ arguments }}
        );
        if let Some(value) = handed_over {
            self.release_value(&value.ty, &value.representation)?;
        }
        let result = EmittedValue {
            owned: crate::execution::ownership::is_managed(&result_type),
            ty: result_type,
            representation: register,
        };
        self.store_input_pattern(resume, Some(&result))?;
        emit_terminator!(self; branch { format!("mal_state_{}", resume.0) });
        Some(())
    }
}

impl FunctionEmitter<'_> {
    /// Continues the activation in the frames version when the native stack is used up.
    pub(in crate::backend::llvm::body) fn emit_native_entry_guard(&mut self) -> Option<()> {
        let mut parameters = vec![
            (llvm_type!(ptr), "%mal_context".into()),
            (llvm_type!(ptr), "%mal_control_top".into()),
            (llvm_type!(ptr), "%mal_environment".into()),
        ];
        if self.function.parameter.ty != Type::Unit {
            let value = self.types.value(&self.function.parameter.ty)?;
            parameters.push((value.llvm, "%mal_parameter".into()));
        }
        let result_llvm = self.types.value(&self.result_type)?.llvm;
        let name = super::super::function_name(self.function.id);
        let stack = self.register();
        emit_instruction!(
            self;
            call { Some(stack.clone()) },
            false,
            (ptr),
            direct "llvm.stacksave";
            []
        );
        let flag = self.register();
        emit_instruction!(
            self;
            call { Some(flag.clone()) },
            false,
            (int(8_u16)),
            direct "mal_native_stack_is_deep";
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => { stack }),
            ]
        );
        let deep = self.register();
        emit_instruction!(
            self;
            compare { deep.clone() },
            { ComparisonKind::Integer },
            { ComparisonPredicate::Ne },
            (int(8_u16)),
            { flag },
            "0"
        );
        let expected = self.register();
        emit_instruction!(
            self;
            call { Some(expected.clone()) },
            false,
            (int(1_u16)),
            direct "llvm.expect.i1";
            [
                (typed (int(1_u16)) => { deep }),
                (typed (int(1_u16)) => "false"),
            ]
        );
        emit_terminator!(self; conditional
            { expected } => "mal_deep_entry", "mal_native_entry"
        );
        self.block("mal_deep_entry");
        let continued = self.register();
        emit_instruction!(
            self;
            call { Some(continued.clone()) },
            false,
            { result_llvm.clone() },
            direct { format!("{name}_frames") },
            {{ parameters }}
        );
        emit_terminator!(self; return { result_llvm } => { continued });
        self.block("mal_native_entry");
        Some(())
    }
}
