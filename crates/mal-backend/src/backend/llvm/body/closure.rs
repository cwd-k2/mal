//! Closure values: the code pointer, unless no application needs it, and the environment of captures.

use crate::backend::llvm::syntax::emit_instruction;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::StateId;
use crate::execution::ownership::BindingOperand;
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_make_closure(
        &mut self,
        site: StateId,
        binding: usize,
        function: &FunctionId,
        captures: &[Atom],
        result_type: Option<&Type>,
    ) -> Option<EmittedValue> {
        let result_type = result_type?.clone();
        let Type::Function { .. } = &result_type else {
            return None;
        };
        let closure_type = self.types.value(&result_type)?;
        let with_code = if self.execution.optimizations.omits_code_pointer(*function) {
            "zeroinitializer".into()
        } else {
            let with_code = self.register();
            emit_instruction! {
                self;
                let { with_code.clone() } = insert_value {
                    aggregate: ({ closure_type.llvm.clone() }, "zeroinitializer"),
                    element: (ptr, { format!("@{}", super::function_name(*function)) }),
                    indices: [0],
                };
            };
            with_code
        };
        let target = *self.index.lowered_functions.get(function)?;
        let environment = &target.captures;
        if captures.len() != environment.len()
            || captures
                .iter()
                .zip(environment)
                .any(|(capture, field)| capture.ty != field.ty)
        {
            return None;
        }
        let closure = if captures.is_empty() {
            with_code
        } else {
            let environment_type =
                Type::Product(environment.iter().map(|field| field.ty.clone()).collect());
            let effects = captures
                .iter()
                .enumerate()
                .map(|(index, atom)| {
                    self.ownership.binding_operand_use(
                        site,
                        binding,
                        BindingOperand::Capture(index),
                        atom,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let environment_value = self.emit_product(captures, &environment_type, &effects)?;
            let environment_layout = self.types.value(&environment_type)?;
            let environment = self.register();
            emit_instruction! {
                self;
                let { environment.clone() } = call {
                    tail: false,
                    result_type: ptr,
                    callee: direct("mal_runtime_owner_allocate"),
                    arguments: [
                        (ptr, "%mal_context"),
                        ({ self.types.index_llvm_type() }, { environment_layout.size.to_string() }),
                        (ptr, { format!(
                            "@mal_destroy_environment_{}",
                            super::function_number(*function)
                        ) }),
                    ],
                };
            };
            emit_instruction! {
                self;
                store {
                    value: (
                        { environment_layout.llvm },
                        { environment_value.value.representation.as_str() },
                    ),
                    pointer: { environment.as_str() },
                    alignment: { environment_layout.alignment },
                    metadata: [],
                };
            };
            self.commit_consumes(&environment_value)?;
            let closure = self.register();
            emit_instruction! {
                self;
                let { closure.clone() } = insert_value {
                    aggregate: ({ closure_type.llvm }, { with_code }),
                    element: (ptr, { environment }),
                    indices: [1],
                };
            };
            closure
        };
        Some(EmittedValue {
            ty: result_type,
            representation: closure,
            owned: true,
        })
    }
}
