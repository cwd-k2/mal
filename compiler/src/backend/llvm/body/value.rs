use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};

use super::scalar::{integer_literal, scalar_type};
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn atom(&mut self, atom: &Atom) -> Option<EmittedValue> {
        match (&atom.ty, &atom.kind) {
            (ty, AtomKind::Integer(value)) if scalar_type(ty).is_some() => Some(EmittedValue {
                ty: ty.clone(),
                representation: integer_literal(ty, *value)?,
            }),
            (Type::Float32, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float32,
                representation: format!("{:.9e}", f32::from_bits(*bits as u32)),
            }),
            (Type::Float64, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float64,
                representation: format!("{:.17e}", f64::from_bits(*bits)),
            }),
            (Type::UInt64, AtomKind::StorageSize(measured)) => Some(EmittedValue {
                ty: Type::UInt64,
                representation: self.types.value(measured)?.size.to_string(),
            }),
            (ty, AtomKind::Reference(Reference::Binding(id))) if self.types.value(ty).is_some() => {
                let slot = self.slots.get(id)?.clone();
                if slot.ty != *ty {
                    return None;
                }
                let value_type = self.types.value(ty)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = load {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, slot.index, value_type.alignment
                ));
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: register,
                })
            }
            (Type::Unit, AtomKind::Unit) => Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
            }),
            _ => None,
        }
    }

    pub(super) fn store_pattern(
        &mut self,
        pattern: &Pattern,
        value: Option<&EmittedValue>,
    ) -> Option<()> {
        match pattern {
            Pattern::Binding { id, ty } if self.types.value(ty).is_some() => {
                let value = value?;
                if value.ty != *ty {
                    return None;
                }
                let slot = self.slots.get(id)?.clone();
                let value_type = self.types.value(ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
                ));
            }
            Pattern::Product { elements, ty, .. } => {
                let value = value?;
                let Type::Product(element_types) = ty else {
                    return None;
                };
                if value.ty != *ty || elements.len() != element_types.len() {
                    return None;
                }
                let aggregate_type = self.types.value(ty)?;
                for (index, (element, element_type)) in
                    elements.iter().zip(element_types).enumerate()
                {
                    let register = self.register();
                    self.line(format!(
                        "  {register} = extractvalue {} {}, {index}",
                        aggregate_type.llvm, value.representation
                    ));
                    self.store_pattern(
                        element,
                        Some(&EmittedValue {
                            ty: element_type.clone(),
                            representation: register,
                        }),
                    )?;
                }
            }
            Pattern::Wildcard { .. } => {}
            _ => return None,
        }
        Some(())
    }
}
