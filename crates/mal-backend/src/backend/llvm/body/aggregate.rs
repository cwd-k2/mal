use crate::backend::llvm::syntax::llvm_type;
use crate::closure::ast::{Atom, AtomKind, Reference};
use crate::control::ast::{CaseArm, StateId};
use mal_frontend::check::ast::Type;

use super::types::is_bool;
use super::{EmittedValue, FunctionEmitter, PreparedValue};
use crate::execution::ownership::UseEffect;

impl FunctionEmitter<'_> {
    pub(super) fn emit_product(
        &mut self,
        elements: &[Atom],
        result_type: &Type,
        effects: &[UseEffect],
    ) -> Option<PreparedValue> {
        let Type::Product(element_types) = result_type else {
            return None;
        };
        if elements.len() != element_types.len() || elements.len() != effects.len() {
            return None;
        }
        let aggregate_type = self.types.value(result_type)?;
        let mut aggregate = "poison".to_string();
        let mut consumed_slots = Vec::new();
        for (index, ((element, expected), effect)) in elements
            .iter()
            .zip(element_types.iter())
            .zip(effects)
            .enumerate()
        {
            let element = self.prepare_atom_for_use(element, *effect)?;
            if element.value.ty != *expected {
                return None;
            }
            consumed_slots.extend(element.consumed_slots);
            let element_type = self.types.value(expected)?;
            let register = self.register();
            emit_instruction! {
                self;
                let {{ register.clone() }} = insert_value {
                    aggregate: typed({{ aggregate_type.llvm.clone() }}, {{ aggregate }}),
                    element: typed({{ element_type.llvm }}, {{ element.value.representation }}),
                    indices: [{{ index }}],
                };
            };
            aggregate = register;
        }
        Some(PreparedValue {
            value: EmittedValue {
                ty: result_type.clone(),
                representation: aggregate,
                owned: element_types.iter().zip(effects).any(|(ty, effect)| {
                    crate::execution::ownership::is_managed(ty) && *effect != UseEffect::Borrow
                }),
            },
            consumed_slots,
        })
    }

    pub(super) fn emit_sum(
        &mut self,
        index: usize,
        value: &Atom,
        result_type: &Type,
        effect: UseEffect,
    ) -> Option<PreparedValue> {
        let prepared = self.prepare_atom_for_use(value, effect)?;
        let value = self.emit_sum_value(
            index,
            prepared.value,
            result_type,
            effect != UseEffect::Borrow,
        )?;
        Some(PreparedValue {
            value,
            consumed_slots: prepared.consumed_slots,
        })
    }

    pub(super) fn emit_sum_value(
        &mut self,
        index: usize,
        mut value: EmittedValue,
        result_type: &Type,
        retain_borrowed: bool,
    ) -> Option<EmittedValue> {
        let Type::Sum(members) = result_type else {
            return None;
        };
        let member = members.get(index)?;
        if value.ty != *member {
            return None;
        }
        if is_bool(result_type) {
            return Some(EmittedValue {
                ty: result_type.clone(),
                representation: match index {
                    0 => "false".into(),
                    1 => "true".into(),
                    _ => return None,
                },
                owned: false,
            });
        }
        if retain_borrowed {
            self.retain_if_borrowed(&mut value)?;
        }
        let sum_type = self.types.value(result_type)?;
        let member_type = self.types.value(member)?;
        let tag = self.register();
        emit_instruction! {
            self;
            let {{ tag.clone() }} = insert_value {
                aggregate: typed({{ sum_type.llvm.clone() }}, "zeroinitializer"),
                element: typed((int(32_u16)), {{ index.to_string() }}),
                indices: [0],
            };
        };
        let storage = self.entry_alloca(&sum_type.llvm, sum_type.alignment);
        emit_instruction! {
            self;
            store {
                value: typed({{ sum_type.llvm.clone() }}, {{ tag }}),
                pointer: {{ storage.as_str() }},
                alignment: {{ sum_type.alignment }},
                metadata: [],
            };
        };
        let payload = self.register();
        emit_instruction! {
            self;
            let {{ payload.clone() }} = get_element_ptr {
                inbounds: true,
                element_type: {{ sum_type.llvm.clone() }},
                pointer: {{ storage.clone() }},
                indices: [typed((int(32_u16)), "0"), typed((int(32_u16)), "1")],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed({{ member_type.llvm }}, {{ value.representation }}),
                pointer: {{ payload }},
                alignment: 1,
                metadata: [],
            };
        };
        let result = self.register();
        emit_instruction! {
            self;
            let {{ result.clone() }} = load {
                ty: {{ sum_type.llvm.clone() }},
                pointer: {{ storage }},
                alignment: {{ sum_type.alignment }},
                metadata: [],
            };
        };
        let owned = value.owned;
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: result,
            owned,
        })
    }

    pub(super) fn emit_case(
        &mut self,
        site: StateId,
        scrutinee: &Atom,
        arms: &[CaseArm],
    ) -> Option<()> {
        let scrutinee_atom = scrutinee;
        let scrutinee = self.atom(scrutinee)?;
        let Type::Sum(members) = &scrutinee.ty else {
            return None;
        };
        let sum_type = self.types.value(&scrutinee.ty)?;
        let tag = if is_bool(&scrutinee.ty) {
            scrutinee.representation.clone()
        } else {
            let tag = self.register();
            emit_instruction! {
                self;
                let {{ tag.clone() }} = extract_value {
                    aggregate: typed({{ sum_type.llvm.clone() }}, {{ scrutinee.representation.clone() }}),
                    indices: [0],
                };
            };
            tag
        };
        let tag_type = llvm_type! {
            int({
                {
                    if is_bool(&scrutinee.ty) {
                        1_u16
                    } else {
                        32_u16
                    }
                }
            })
        };
        let cases = arms.iter().map(|arm| {
            (
                arm.index.to_string(),
                format!("mal_case_{}_{}", site.0, arm.index),
            )
        });
        emit_terminator! {
            self;
            switch typed({{ tag_type }}, {{ tag }})  {
                cases: [...{{ cases }}],
                default: {{ format!("mal_invalid_case_{}", site.0) }},
            };
        };
        self.block(format!("mal_invalid_case_{}", site.0));
        emit_terminator! {
            self;
            unreachable;
        };
        for (arm_ordinal, arm) in arms.iter().enumerate() {
            let member = members.get(arm.index)?;
            self.block(format!("mal_case_{}_{}", site.0, arm.index));
            let payload = if is_bool(&scrutinee.ty) {
                EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                }
            } else {
                let payload =
                    self.emit_sum_payload(&scrutinee.ty, member, &scrutinee.representation)?;
                EmittedValue {
                    ty: member.clone(),
                    representation: payload,
                    owned: false,
                }
            };
            let payload = self.prepare_case_payload(site, arm_ordinal, scrutinee_atom, payload)?;
            self.commit_consumes(&payload)?;
            self.store_input_pattern(arm.target, Some(&payload.value))?;
            self.emit_edge_drops(
                site,
                crate::execution::ownership::ControlPath::CaseArm(arm_ordinal),
            )?;
            emit_terminator! {
                self;
                branch {
                    target: {{ format!("mal_state_{}", arm.target.0) }},
                };
            };
        }
        Some(())
    }

    fn prepare_case_payload(
        &mut self,
        site: StateId,
        arm: usize,
        scrutinee: &Atom,
        mut payload: EmittedValue,
    ) -> Option<PreparedValue> {
        if !crate::execution::ownership::is_managed(&payload.ty) {
            return Some(PreparedValue {
                value: payload,
                consumed_slots: Vec::new(),
            });
        }
        let mut consumed_slots = Vec::new();
        match self.ownership.case_payload_use(site, arm) {
            Some(UseEffect::Share) => self.retain_if_borrowed(&mut payload)?,
            Some(UseEffect::Consume) => {
                let AtomKind::Reference(Reference::Binding(id)) = scrutinee.kind else {
                    return None;
                };
                let slot = self.slots.get(&id)?.clone();
                if slot.ty != scrutinee.ty {
                    return None;
                }
                payload.owned = true;
                consumed_slots.push(slot);
            }
            Some(UseEffect::Borrow) => return None,
            None => {}
        }
        Some(PreparedValue {
            value: payload,
            consumed_slots,
        })
    }

    pub(super) fn emit_sum_payload(
        &mut self,
        sum: &Type,
        member: &Type,
        value: &str,
    ) -> Option<String> {
        let sum_type = self.types.value(sum)?;
        let member_type = self.types.value(member)?;
        let storage = self.entry_alloca(&sum_type.llvm, sum_type.alignment);
        emit_instruction! {
            self;
            store {
                value: typed({{ sum_type.llvm.clone() }}, {{ value }}),
                pointer: {{ storage.as_str() }},
                alignment: {{ sum_type.alignment }},
                metadata: [],
            };
        };
        let pointer = self.register();
        emit_instruction! {
            self;
            let {{ pointer.clone() }} = get_element_ptr {
                inbounds: true,
                element_type: {{ sum_type.llvm }},
                pointer: {{ storage }},
                indices: [typed((int(32_u16)), "0"), typed((int(32_u16)), "1")],
            };
        };
        let payload = self.register();
        emit_instruction! {
            self;
            let {{ payload.clone() }} = load {
                ty: {{ member_type.llvm }},
                pointer: {{ pointer }},
                alignment: 1,
                metadata: [],
            };
        };
        Some(payload)
    }
}
