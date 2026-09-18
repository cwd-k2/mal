use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Reference};
use crate::control::ast::{CaseArm, StateId};

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
            self.line(format!(
                "  {register} = insertvalue {} {aggregate}, {} {}, {index}",
                aggregate_type.llvm, element_type.llvm, element.value.representation
            ));
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
        let value = self.emit_sum_value(index, prepared.value, result_type, true)?;
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
        self.line(format!(
            "  {tag} = insertvalue {} zeroinitializer, i32 {index}, 0",
            sum_type.llvm
        ));
        let storage = self.register();
        self.line(format!(
            "  {storage} = alloca {}, align {}",
            sum_type.llvm, sum_type.alignment
        ));
        self.line(format!(
            "  store {} {tag}, ptr {storage}, align {}",
            sum_type.llvm, sum_type.alignment
        ));
        let payload = self.register();
        self.line(format!(
            "  {payload} = getelementptr inbounds {}, ptr {storage}, i32 0, i32 1",
            sum_type.llvm
        ));
        self.line(format!(
            "  store {} {}, ptr {payload}, align 1",
            member_type.llvm, value.representation
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = load {}, ptr {storage}, align {}",
            sum_type.llvm, sum_type.alignment
        ));
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: result,
            owned: crate::execution::ownership::is_managed(result_type),
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
            self.line(format!(
                "  {tag} = extractvalue {} {}, 0",
                sum_type.llvm, scrutinee.representation
            ));
            tag
        };
        let tag_type = if is_bool(&scrutinee.ty) { "i1" } else { "i32" };
        let cases = arms
            .iter()
            .map(|arm| {
                format!(
                    "    {tag_type} {}, label %mal_case_{}_{}",
                    arm.index, site.0, arm.index
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.line(format!(
            "  switch {tag_type} {tag}, label %mal_invalid_case_{} [\n{cases}\n  ]",
            site.0
        ));
        self.line(format!("mal_invalid_case_{}:", site.0));
        self.line("  unreachable");
        for (arm_ordinal, arm) in arms.iter().enumerate() {
            let member = members.get(arm.index)?;
            self.line(format!("mal_case_{}_{}:", site.0, arm.index));
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
            self.line(format!("  br label %mal_state_{}", arm.target.0));
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
        let storage = self.register();
        self.line(format!(
            "  {storage} = alloca {}, align {}",
            sum_type.llvm, sum_type.alignment
        ));
        self.line(format!(
            "  store {} {value}, ptr {storage}, align {}",
            sum_type.llvm, sum_type.alignment
        ));
        let pointer = self.register();
        self.line(format!(
            "  {pointer} = getelementptr inbounds {}, ptr {storage}, i32 0, i32 1",
            sum_type.llvm
        ));
        let payload = self.register();
        self.line(format!(
            "  {payload} = load {}, ptr {pointer}, align 1",
            member_type.llvm
        ));
        Some(payload)
    }
}
