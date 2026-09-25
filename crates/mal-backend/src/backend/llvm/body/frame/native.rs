//! Native recursion for a self-recursive function.
//!
//! The function is emitted twice. The frames version keeps the suspended callers of a recursion in the control arena,
//! so it never nests native calls. The native version, which carries the function's own name, nests a native call
//! for each recursive call and hands the whole activation to the frames version at its entry once the native stack
//! is used up. A native activation keeps its live values in its own slots and touches no control arena, so the
//! optimizer sees an ordinary recursive function whose only extra work is one stack check per activation.

use crate::closure::ast::Atom;
use crate::control::ast::{StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::super::{EmissionMode, EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    /// Whether the function can also be emitted as a native version: it recurses only into itself through frames,
    /// and its parameter is not managed, so a native call needs no ownership handoff.
    pub(in crate::backend::llvm::body) fn has_native_version(&self) -> bool {
        self.mode == EmissionMode::Standard
            && self.common_region.is_none()
            && !self.frame_sites.is_empty()
            && !crate::execution::ownership::is_managed(&self.function.parameter.ty)
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
        let arguments = if self.function.parameter.ty == Type::Unit {
            format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            let argument_type = self.types.value(&argument.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
                argument_type.llvm, argument.representation
            )
        };
        let result_type = self.current_result_type()?;
        let result_llvm = self.types.value(&result_type)?.llvm;
        let name = super::super::function_name(self.function.id)?;
        let register = self.register();
        self.line(format!(
            "  {register} = call {result_llvm} @{name}({arguments})"
        ));
        let result = EmittedValue {
            owned: crate::execution::ownership::is_managed(&result_type),
            ty: result_type,
            representation: register,
        };
        self.store_input_pattern(resume, Some(&result))?;
        self.line(format!("  br label %mal_state_{}", resume.0));
        Some(())
    }
}

impl FunctionEmitter<'_> {
    /// Continues the activation in the frames version when the native stack is used up.
    pub(in crate::backend::llvm::body) fn emit_native_entry_guard(&mut self) -> Option<()> {
        let parameter = if self.function.parameter.ty == Type::Unit {
            "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment".to_string()
        } else {
            let value = self.types.value(&self.function.parameter.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment, {} %mal_parameter",
                value.llvm
            )
        };
        let result_llvm = self.types.value(&self.result_type)?.llvm;
        let name = super::super::function_name(self.function.id)?;
        let flag = self.register();
        self.line(format!(
            "  {flag} = call i8 @mal_native_stack_is_deep(ptr %mal_context)"
        ));
        let deep = self.register();
        self.line(format!("  {deep} = icmp ne i8 {flag}, 0"));
        self.line(format!(
            "  br i1 {deep}, label %mal_deep_entry, label %mal_native_entry"
        ));
        self.line("mal_deep_entry:");
        let continued = self.register();
        self.line(format!(
            "  {continued} = call {result_llvm} @{name}_frames({parameter})"
        ));
        self.line(format!("  ret {result_llvm} {continued}"));
        self.line("mal_native_entry:");
        Some(())
    }
}
