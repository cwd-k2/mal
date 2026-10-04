//! Native recursion for a self-recursive function.
//!
//! The function is emitted twice. The frames version keeps the suspended callers of a recursion in the control arena,
//! so it never nests native calls. The native version, which carries the function's own name, nests a native call
//! for each recursive call and hands the whole activation to the frames version at its entry once the native stack
//! is used up. A native activation keeps its live values in its own slots and touches no control arena, so the
//! optimizer sees an ordinary recursive function whose only extra work is one stack check per activation.

use crate::backend::llvm::syntax::{
    ComparisonKind, ComparisonPredicate, Parameter, Type as LlvmType, llvm_parameter,
    llvm_parameters, llvm_signature, llvm_type,
};
use crate::closure::ast::Atom;
use crate::control::ast::{StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::super::{EmissionMode, EmittedValue, FunctionEmitter};

mod call;
mod entry;
mod wrapper;

impl FunctionEmitter<'_> {
    fn native_scalar_parameter(&self) -> Option<&crate::execution::NativeScalarParameter> {
        self.execution
            .native_recursion
            .scalar_parameter(self.function.id)
    }

    fn native_context_type(&self) -> Option<LlvmType> {
        let parameter = self.types.value(&self.function.parameter.ty)?;
        Some(llvm_type!(structure([ptr, ptr, ptr, { parameter.llvm }])))
    }

    fn native_worker_name(&self) -> String {
        let name = super::super::function_name(self.function.id);
        if self.native_scalar_parameter().is_some() {
            format!("{name}_native")
        } else {
            name
        }
    }

    pub(in crate::backend::llvm::body) fn native_worker_parameters(
        &self,
    ) -> Option<Vec<Parameter>> {
        let plan = self.native_scalar_parameter()?;
        let mut parameters = vec![llvm_parameter!("%mal_native_context": ptr)];
        for (index, leaf) in plan.varying.iter().enumerate() {
            parameters.push(llvm_parameter! {
                { format!("%mal_native_parameter_{index}") }: { self.types.value(&leaf.ty)?.llvm }
            });
        }
        Some(parameters)
    }

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
}
