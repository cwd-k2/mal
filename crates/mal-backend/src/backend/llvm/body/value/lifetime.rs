use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn load_slot(
        &mut self,
        slot: &super::super::Slot,
    ) -> Option<String> {
        let value_type = self.types.value(&slot.ty)?;
        let value = self.register();
        emit_instruction! {
            self;
            let #{ value.clone() } = load {
                ty: #{ value_type.llvm },
                pointer: #{ format!("%mal_slot_{}", slot.index) },
                alignment: #{ value_type.alignment },
                metadata: [],
            };
        };
        Some(value)
    }

    pub(in crate::backend::llvm::body) fn initialize_slot(
        &mut self,
        slot: &super::super::Slot,
        value: &EmittedValue,
    ) -> Option<()> {
        if slot.ty != value.ty {
            return None;
        }
        let value_type = self.types.value(&slot.ty)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, #{ value.representation.as_str() }),
                pointer: #{ format!("%mal_slot_{}", slot.index) },
                alignment: #{ value_type.alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn vacate_slot(
        &mut self,
        slot: &super::super::Slot,
    ) -> Option<()> {
        let value_type = self.types.value(&slot.ty)?;
        self.vacate_place(
            &slot.ty,
            &format!("%mal_slot_{}", slot.index),
            value_type.alignment,
        )
    }

    pub(in crate::backend::llvm::body) fn vacate_place(
        &mut self,
        ty: &Type,
        pointer: &str,
        alignment: usize,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(ty) {
            return None;
        }
        let value_type = self.types.value(ty)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, "zeroinitializer"),
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn replace_managed_place(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        alignment: usize,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&value.ty) {
            return None;
        }
        let value_type = self.types.value(&value.ty)?;
        self.retain_value(&value.ty, &value.representation)?;
        let previous = self.register();
        emit_instruction! {
            self;
            let #{ previous.clone() } = load {
                ty: #{ value_type.llvm.clone() },
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        self.release_value(&value.ty, &previous)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, #{ value.representation.as_str() }),
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn retain_if_borrowed(
        &mut self,
        value: &mut EmittedValue,
    ) -> Option<()> {
        if crate::execution::ownership::is_managed(&value.ty) && !value.owned {
            value.representation = self.retain_value(&value.ty, &value.representation)?;
            value.owned = true;
        }
        Some(())
    }

    pub(in crate::backend::llvm::body) fn release_dead_slot(
        &mut self,
        id: crate::anf::ast::ValueId,
    ) -> Option<()> {
        let Some(slot) = self.slots.get(&id).cloned() else {
            return Some(());
        };
        if !crate::execution::ownership::is_managed(&slot.ty) {
            return Some(());
        }
        self.release_slot(&slot)
    }

    pub(in crate::backend::llvm::body) fn emit_edge_drops(
        &mut self,
        site: crate::control::ast::StateId,
        path: crate::execution::ownership::ControlPath,
    ) -> Option<()> {
        let mut drops = self.ownership.drops_on_edge(site, path).to_vec();
        drops.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
        for id in drops {
            self.release_dead_slot(id)?;
        }
        Some(())
    }

    fn release_slot(&mut self, slot: &super::super::Slot) -> Option<()> {
        let value = self.load_slot(slot)?;
        self.release_value(&slot.ty, &value)?;
        self.vacate_slot(slot)
    }

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
                    let #{ owner.clone() } = extract_value {
                        aggregate: typed(#{ value_type.llvm }, #{ value }),
                        indices: [0],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_bytes_retain"),
                        arguments: [typed((ptr), "%mal_context"), typed((ptr), #{ owner })],
                    };
                };
                Some(value.into())
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                emit_instruction! {
                    self;
                    let #{ environment.clone() } = extract_value {
                        aggregate: typed(#{ value_type.llvm }, #{ value }),
                        indices: [1],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_environment_retain"),
                        arguments: [typed((ptr), "%mal_context"), typed((ptr), #{ environment })],
                    };
                };
                Some(value.into())
            }
            Type::Buffer(_) => {
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_environment_retain"),
                        arguments: [typed((ptr), "%mal_context"), typed((ptr), #{ value })],
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
                        let #{ field.clone() } = extract_value {
                            aggregate: typed(#{ aggregate_type.llvm.clone() }, #{ value }),
                            indices: [#{ index }],
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
                    let #{ owner.clone() } = extract_value {
                        aggregate: typed(#{ value_type.llvm }, #{ value }),
                        indices: [0],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (void),
                        callee: direct("mal_runtime_bytes_release"),
                        arguments: [typed((ptr), #{ owner })],
                    };
                };
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                emit_instruction! {
                    self;
                    let #{ environment.clone() } = extract_value {
                        aggregate: typed(#{ value_type.llvm }, #{ value }),
                        indices: [1],
                    };
                };
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (void),
                        callee: direct("mal_runtime_environment_release"),
                        arguments: [typed((ptr), #{ environment })],
                    };
                }
            }
            Type::Buffer(_) => {
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (void),
                        callee: direct("mal_runtime_environment_release"),
                        arguments: [typed((ptr), #{ value })],
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
                        let #{ field.clone() } = extract_value {
                            aggregate: typed(#{ aggregate_type.llvm.clone() }, #{ value }),
                            indices: [#{ index }],
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

    fn emit_sum_lifetime(
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
            let #{ tag.clone() } = extract_value {
                aggregate: typed(#{ sum_type.llvm }, #{ value }),
                indices: [0],
            };
        };
        let cases = members
            .iter()
            .enumerate()
            .map(|(index, _)| (index.to_string(), format!("mal_{operation}_{id}_{index}")));
        emit_terminator! {
            self;
            switch typed((int(32_u16)), #{ tag })  {
                cases: [...#{ cases }],
                default: #{ format!("mal_{operation}_{id}_invalid") },
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
                    target: #{ format!("mal_{operation}_{id}_done") },
                };
            };
        }
        self.block(format!("mal_{operation}_{id}_done"));
        Some(())
    }
}
