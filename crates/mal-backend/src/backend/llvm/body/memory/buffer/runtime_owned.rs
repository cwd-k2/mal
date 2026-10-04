use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
use crate::backend::llvm::syntax::llvm_signature;
use crate::core::ast::BufferOperation;
use mal_frontend::check::ast::Type;

use super::super::super::{EmittedFunction, FunctionEmitter};

/// The element types whose Buffer places own responsibilities. Each has one retain and one release callback,
/// numbered by position; runtime-represented trivial carriers are deliberately absent.
pub(in crate::backend::llvm::body) struct OwnedBufferElements(Vec<Type>);

impl OwnedBufferElements {
    pub(in crate::backend::llvm::body) fn collect(execution: &crate::execution::Program) -> Self {
        let mut elements = Vec::new();
        for binding in execution
            .control
            .states
            .iter()
            .flat_map(|state| &state.bindings)
        {
            if let crate::control::ast::Operation::Buffer {
                operation: BufferOperation::Make,
                element,
                ..
            } = &binding.operation
                && crate::execution::ownership::lifecycle(element).is_owned()
                && !elements.contains(element)
            {
                elements.push(element.clone());
            }
        }
        Self(elements)
    }

    pub(super) fn number(&self, element: &Type) -> Option<usize> {
        self.0.iter().position(|candidate| candidate == element)
    }
}

impl FunctionEmitter<'_> {
    /// Defines the callbacks that let the runtime retain and release one stored element of each
    /// runtime-owned element type.
    pub(in crate::backend::llvm::body) fn emit_owned_buffer_element_callbacks(
        &mut self,
    ) -> Option<EmittedFunction> {
        let index = self.index;
        for (number, element) in index.owned_buffer_elements.0.iter().enumerate() {
            self.emit_owned_element_callback(number, element, true)?;
            self.emit_owned_element_callback(number, element, false)?;
        }
        (!self.emission_failed).then(|| EmittedFunction {
            globals: std::mem::take(&mut self.globals),
            definitions: std::mem::take(&mut self.definitions),
        })
    }

    fn emit_owned_element_callback(
        &mut self,
        number: usize,
        element: &Type,
        retain: bool,
    ) -> Option<()> {
        let value_type = self.types.value(element)?;
        let signature = if retain {
            llvm_signature! {
                #[linkage(internal)] fn { format!("mal_buffer_retain_{number}") }(
                    "%mal_context": ptr,
                    "%mal_element": ptr,
                ) -> void
            }
        } else {
            llvm_signature! {
                #[linkage(internal)] fn { format!("mal_buffer_release_{number}") }(
                    "%mal_element": ptr,
                ) -> void
            }
        };
        self.begin_function(signature);
        self.block("entry");
        let value = self.register();
        emit_instruction! {
            self;
            let { value.clone() } = load {
                ty: { value_type.llvm },
                pointer: "%mal_element",
                alignment: { value_type.alignment },
                metadata: [],
            };
        };
        if retain {
            self.retain_value(element, &value)?;
        } else {
            self.release_value(element, &value)?;
        }
        emit_terminator! {
            self;
            return;
        };
        self.finish_function()
    }
}
