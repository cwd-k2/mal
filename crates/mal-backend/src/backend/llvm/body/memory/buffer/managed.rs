use crate::backend::llvm::syntax::llvm_signature;
use crate::core::ast::BufferOperation;
use mal_frontend::check::ast::Type;

use super::super::super::{EmittedFunction, FunctionEmitter};

/// The element types of the program's Buffers that own managed values. Each has one retain and one
/// release callback, numbered by position.
pub(in crate::backend::llvm::body) struct ManagedBufferElements(Vec<Type>);

impl ManagedBufferElements {
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
                && crate::execution::ownership::is_managed(element)
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
    /// managed element type.
    pub(in crate::backend::llvm::body) fn emit_managed_buffer_element_callbacks(
        &mut self,
    ) -> Option<EmittedFunction> {
        let index = self.index;
        for (number, element) in index.managed_buffer_elements.0.iter().enumerate() {
            self.emit_managed_element_callback(number, element, true)?;
            self.emit_managed_element_callback(number, element, false)?;
        }
        (!self.emission_failed).then(|| EmittedFunction {
            globals: std::mem::take(&mut self.globals),
            definitions: std::mem::take(&mut self.definitions),
        })
    }

    fn emit_managed_element_callback(
        &mut self,
        number: usize,
        element: &Type,
        retain: bool,
    ) -> Option<()> {
        let value_type = self.types.value(element)?;
        let signature = if retain {
            llvm_signature!(internal fn { format!("mal_buffer_retain_{number}") }(
                "%mal_context": ptr,
                "%mal_element": ptr,
            ) -> void; attributes [])
        } else {
            llvm_signature!(internal fn { format!("mal_buffer_release_{number}") }(
                "%mal_element": ptr,
            ) -> void; attributes [])
        };
        self.begin_function(signature);
        self.block("entry");
        let value = self.register();
        emit_instruction!(
            self;
            load { value.clone() },
            { value_type.llvm },
            "%mal_element",
            { value_type.alignment },
            []
        );
        if retain {
            self.retain_value(element, &value)?;
        } else {
            self.release_value(element, &value)?;
        }
        emit_terminator!(self; return_void);
        self.finish_function()
    }
}
