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
                owned: false,
            }),
            (Type::Float32, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float32,
                representation: format!("{:.9e}", f32::from_bits(*bits as u32)),
                owned: false,
            }),
            (Type::Float64, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float64,
                representation: format!("{:.17e}", f64::from_bits(*bits)),
                owned: false,
            }),
            (Type::UInt64, AtomKind::StorageSize(measured)) => Some(EmittedValue {
                ty: Type::UInt64,
                representation: self.types.value(measured)?.size.to_string(),
                owned: false,
            }),
            (Type::Symbol, AtomKind::Symbol(bytes)) => {
                if bytes.is_empty() {
                    return Some(EmittedValue {
                        ty: Type::Symbol,
                        representation: "null".into(),
                        owned: true,
                    });
                }
                let name = format!("mal_symbol_literal_{}", atom.id.0);
                let contents = bytes
                    .iter()
                    .map(|byte| match byte {
                        0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7e => (*byte as char).to_string(),
                        _ => format!("\\{byte:02X}"),
                    })
                    .collect::<String>();
                self.globals.push_str(&format!(
                    "@{name} = private constant {{ i64, i64, [{} x i8] }} {{ i64 -1, i64 {}, [{} x i8] c\"{contents}\" }}, align 8\n",
                    bytes.len(),
                    bytes.len(),
                    bytes.len()
                ));
                Some(EmittedValue {
                    ty: Type::Symbol,
                    representation: format!("@{name}"),
                    owned: true,
                })
            }
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
                    owned: false,
                })
            }
            (Type::Unit, AtomKind::Unit) => Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
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
            Pattern::Binding {
                id,
                ty: Type::Symbol,
            } => {
                let mut value = value?.clone();
                if value.ty != Type::Symbol {
                    return None;
                }
                self.retain_symbol_if_borrowed(&mut value)?;
                let slot = self.slots.get(id)?.clone();
                let previous = self.register();
                self.line(format!(
                    "  {previous} = load ptr, ptr %mal_slot_{}, align {}",
                    slot.index,
                    self.types.value(&Type::Symbol)?.alignment
                ));
                self.line(format!(
                    "  call void @mal_runtime_symbol_release(ptr {previous})"
                ));
                self.line(format!(
                    "  store ptr {}, ptr %mal_slot_{}, align {}",
                    value.representation,
                    slot.index,
                    self.types.value(&Type::Symbol)?.alignment
                ));
            }
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
                            owned: false,
                        }),
                    )?;
                }
            }
            Pattern::Wildcard { ty, .. } => {
                if *ty == Type::Symbol {
                    let value = value?;
                    if value.owned {
                        self.line(format!(
                            "  call void @mal_runtime_symbol_release(ptr {})",
                            value.representation
                        ));
                    }
                }
            }
            _ => return None,
        }
        Some(())
    }

    pub(super) fn retain_symbol_if_borrowed(&mut self, value: &mut EmittedValue) -> Option<()> {
        if value.ty == Type::Symbol && !value.owned {
            let retained = self.register();
            self.line(format!(
                "  {retained} = call ptr @mal_runtime_symbol_retain(ptr %mal_context, ptr {})",
                value.representation
            ));
            value.representation = retained;
            value.owned = true;
        }
        Some(())
    }

    pub(super) fn release_local_symbols(&mut self) {
        let parameter = self.function.parameter.binding;
        let mut slots = self
            .slots
            .iter()
            .filter(|(id, slot)| slot.ty == Type::Symbol && Some(**id) != parameter)
            .map(|(_, slot)| slot.clone())
            .collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value = self.register();
            self.line(format!(
                "  {value} = load ptr, ptr %mal_slot_{}, align {}",
                slot.index,
                self.types
                    .value(&Type::Symbol)
                    .expect("Symbol representation")
                    .alignment
            ));
            self.line(format!(
                "  call void @mal_runtime_symbol_release(ptr {value})"
            ));
        }
    }
}
