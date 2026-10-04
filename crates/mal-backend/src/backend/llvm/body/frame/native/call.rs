//! A recursive call nested as a native call of the worker.

use super::*;
use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;

impl FunctionEmitter<'_> {
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
        let scalar_parameter = self.native_scalar_parameter().cloned();
        let mut arguments = if scalar_parameter.is_some() {
            vec![(llvm_type!(ptr), "%mal_native_context".into())]
        } else {
            vec![
                (llvm_type!(ptr), "%mal_context".to_owned()),
                (llvm_type!(ptr), "%mal_control_top".to_owned()),
                (llvm_type!(ptr), environment),
            ]
        };
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
            if let Some(plan) = &scalar_parameter {
                for leaf in &plan.varying {
                    let register = self.register();
                    emit_instruction! {
                        self;
                        let { register.clone() } = extract_value {
                            aggregate: ({ argument_type.llvm.clone() }, { argument.representation.clone() }),
                            indices: { leaf.path.clone() },
                        };
                    };
                    arguments.push((self.types.value(&leaf.ty)?.llvm, register));
                }
            } else {
                arguments.push((argument_type.llvm, argument.representation));
            }
        }
        let result_type = self.current_result_type()?;
        let result_llvm = self.types.value(&result_type)?.llvm;
        let name = self.native_worker_name();
        let register = self.register();
        let arguments = crate::backend::llvm::syntax::TypedValue::from_pairs(arguments)?;
        emit_instruction! {
            self;
            let { register.clone() } = call {
                tail: false,
                result_type: { result_llvm },
                callee: direct({ name }),
                arguments: [..{ arguments }],
            };
        };
        if let Some(value) = handed_over {
            self.release_value(&value.ty, &value.representation)?;
        }
        let result = EmittedValue {
            owned: crate::execution::ownership::is_managed(&result_type),
            ty: result_type,
            representation: register,
        };
        self.store_input_pattern(resume, Some(&result))?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_state_{}", resume.0) },
            };
        };
        Some(())
    }
}
