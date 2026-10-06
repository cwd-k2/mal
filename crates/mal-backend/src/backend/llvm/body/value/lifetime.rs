//! Typed share and drop of a managed value, recursing through products and sums.

use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
use mal_frontend::check::ast::Type;

use super::super::FunctionEmitter;

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn retain_value(
        &mut self,
        ty: &Type,
        value: &str,
    ) -> Option<String> {
        match ty {
            Type::Symbol => {
                let value_type = self.types.value(ty)?;
                let owner = self.register();
                emit_instruction! {
                    self;
                    let { owner.clone() } = extract_value {
                        aggregate: ({ value_type.llvm }, { value }),
                        indices: [0],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: ptr,
                        callee: direct("mal_runtime_bytes_retain"),
                        arguments: [(ptr, { owner })],
                    };
                };
                Some(value.into())
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                emit_instruction! {
                    self;
                    let { environment.clone() } = extract_value {
                        aggregate: ({ value_type.llvm }, { value }),
                        indices: [1],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: ptr,
                        callee: direct("mal_runtime_owner_retain"),
                        arguments: [(ptr, { environment })],
                    };
                };
                Some(value.into())
            }
            Type::Buffer(_) => {
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: ptr,
                        callee: direct("mal_runtime_owner_retain"),
                        arguments: [(ptr, { value })],
                    };
                };
                Some(value.into())
            }
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    emit_instruction! {
                        self;
                        let { field.clone() } = extract_value {
                            aggregate: ({ aggregate_type.llvm.clone() }, { value }),
                            indices: [{ index }],
                        };
                    };
                    self.retain_value(element, &field)?;
                }
                Some(value.into())
            }
            Type::Sum(members) => {
                self.emit_sum_lifetime(ty, members, value, true)?;
                Some(value.into())
            }
            _ if !crate::execution::ownership::is_managed(ty) => Some(value.into()),
            _ => None,
        }
    }

    pub(in crate::backend::llvm::body) fn release_value(
        &mut self,
        ty: &Type,
        value: &str,
    ) -> Option<()> {
        match ty {
            Type::Symbol => {
                let value_type = self.types.value(ty)?;
                let owner = self.register();
                emit_instruction! {
                    self;
                    let { owner.clone() } = extract_value {
                        aggregate: ({ value_type.llvm }, { value }),
                        indices: [0],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: void,
                        callee: direct("mal_runtime_bytes_release"),
                        arguments: [(ptr, { owner })],
                    };
                };
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                emit_instruction! {
                    self;
                    let { environment.clone() } = extract_value {
                        aggregate: ({ value_type.llvm }, { value }),
                        indices: [1],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: void,
                        callee: direct("mal_runtime_owner_release"),
                        arguments: [(ptr, { environment })],
                    };
                }
            }
            Type::Buffer(_) => {
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: void,
                        callee: direct("mal_runtime_owner_release"),
                        arguments: [(ptr, { value })],
                    };
                }
            }
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    emit_instruction! {
                        self;
                        let { field.clone() } = extract_value {
                            aggregate: ({ aggregate_type.llvm.clone() }, { value }),
                            indices: [{ index }],
                        };
                    };
                    self.release_value(element, &field)?;
                }
            }
            Type::Sum(members) => self.emit_sum_lifetime(ty, members, value, false)?,
            _ if crate::execution::ownership::is_managed(ty) => return None,
            _ => {}
        }
        Some(())
    }

    pub(super) fn emit_sum_lifetime(
        &mut self,
        ty: &Type,
        members: &[Type],
        value: &str,
        retain: bool,
    ) -> Option<()> {
        let sum_type = self.types.value(ty)?;
        let id = self.label_id();
        let operation = if retain { "retain" } else { "release" };
        let tag = self.register();
        emit_instruction! {
            self;
            let { tag.clone() } = extract_value {
                aggregate: ({ sum_type.llvm }, { value }),
                indices: [0],
            };
        };
        let cases = members
            .iter()
            .enumerate()
            .map(|(index, _)| (index.to_string(), format!("mal_{operation}_{id}_{index}")));
        emit_terminator! {
            self;
            switch (int(32_u16), { tag })  {
                cases: [..{ cases }],
                default: { format!("mal_{operation}_{id}_invalid") },
            };
        };
        self.block(format!("mal_{operation}_{id}_invalid"));
        emit_terminator! {
            self;
            unreachable;
        };
        for (index, member) in members.iter().enumerate() {
            self.block(format!("mal_{operation}_{id}_{index}"));
            if crate::execution::ownership::is_managed(member) {
                let payload = self.emit_sum_payload(ty, member, value)?;
                if retain {
                    self.retain_value(member, &payload)?;
                } else {
                    self.release_value(member, &payload)?;
                }
            }
            emit_terminator! {
                self;
                branch {
                    target: { format!("mal_{operation}_{id}_done") },
                };
            };
        }
        self.block(format!("mal_{operation}_{id}_done"));
        Some(())
    }
}
