use crate::check::ast::Type;
use crate::closure::ast::Atom;
use crate::control::ast::{CaseArm, StateId};

use super::types::is_bool;
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_product(
        &mut self,
        elements: &[Atom],
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(element_types) = result_type else {
            return None;
        };
        if elements.len() != element_types.len() {
            return None;
        }
        let aggregate_type = self.types.value(result_type)?;
        let mut aggregate = "poison".to_string();
        for (index, (element, expected)) in elements.iter().zip(element_types.iter()).enumerate() {
            let mut element = self.atom(element)?;
            if element.ty != *expected {
                return None;
            }
            self.retain_if_borrowed(&mut element)?;
            let element_type = self.types.value(expected)?;
            let register = self.register();
            self.line(format!(
                "  {register} = insertvalue {} {aggregate}, {} {}, {index}",
                aggregate_type.llvm, element_type.llvm, element.representation
            ));
            aggregate = register;
        }
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: aggregate,
            owned: crate::execution::ownership::is_managed(result_type),
        })
    }

    pub(super) fn emit_sum(
        &mut self,
        index: usize,
        value: &Atom,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Sum(members) = result_type else {
            return None;
        };
        let member = members.get(index)?;
        let mut value = self.atom(value)?;
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
        self.retain_if_borrowed(&mut value)?;
        let sum_type = self.types.value(result_type)?;
        let member_type = self.types.value(member)?;
        let tag = self.register();
        self.line(format!(
            "  {tag} = insertvalue {} poison, i32 {index}, 0",
            sum_type.llvm
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {tag}, {} {}, {}",
            sum_type.llvm,
            member_type.llvm,
            value.representation,
            index + 1
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
        for arm in arms {
            let member = members.get(arm.index)?;
            self.line(format!("mal_case_{}_{}:", site.0, arm.index));
            let payload = if is_bool(&scrutinee.ty) {
                EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                }
            } else {
                let payload = self.register();
                self.line(format!(
                    "  {payload} = extractvalue {} {}, {}",
                    sum_type.llvm,
                    scrutinee.representation,
                    arm.index + 1
                ));
                EmittedValue {
                    ty: member.clone(),
                    representation: payload,
                    owned: false,
                }
            };
            let input = self.control.states[arm.target.0].input.as_ref()?;
            self.store_pattern(input, Some(&payload))?;
            self.line(format!("  br label %mal_state_{}", arm.target.0));
        }
        Some(())
    }
}
