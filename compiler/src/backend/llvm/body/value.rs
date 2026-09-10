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
            Pattern::Binding { id, ty } if crate::execution::ownership::is_managed(ty) => {
                let mut value = value?.clone();
                if value.ty != *ty {
                    return None;
                }
                self.retain_if_borrowed(&mut value)?;
                let slot = self.slots.get(id)?.clone();
                let previous = self.register();
                let value_type = self.types.value(ty)?;
                self.line(format!(
                    "  {previous} = load {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, slot.index, value_type.alignment
                ));
                self.release_value(ty, &previous)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
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
                            owned: value.owned
                                && crate::execution::ownership::is_managed(element_type),
                        }),
                    )?;
                }
            }
            Pattern::Wildcard { ty, .. } => {
                if crate::execution::ownership::is_managed(ty) {
                    let value = value?;
                    if value.owned {
                        self.release_value(ty, &value.representation)?;
                    }
                }
            }
            _ => return None,
        }
        Some(())
    }

    pub(super) fn retain_if_borrowed(&mut self, value: &mut EmittedValue) -> Option<()> {
        if crate::execution::ownership::is_managed(&value.ty) && !value.owned {
            value.representation = self.retain_value(&value.ty, &value.representation)?;
            value.owned = true;
        }
        Some(())
    }

    pub(super) fn release_local_managed(&mut self) {
        let mut slots = self
            .slots
            .iter()
            .filter(|(_, slot)| crate::execution::ownership::is_managed(&slot.ty))
            .map(|(_, slot)| slot.clone())
            .collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value = self.register();
            let value_type = self.types.value(&slot.ty).expect("managed representation");
            self.line(format!(
                "  {value} = load {}, ptr %mal_slot_{}, align {}",
                value_type.llvm, slot.index, value_type.alignment
            ));
            self.release_value(&slot.ty, &value)
                .expect("managed slot is supported");
            self.line(format!(
                "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
                value_type.llvm, slot.index, value_type.alignment
            ));
        }
    }

    fn retain_value(&mut self, ty: &Type, value: &str) -> Option<String> {
        match ty {
            Type::Symbol => {
                let retained = self.register();
                self.line(format!(
                    "  {retained} = call ptr @mal_runtime_symbol_retain(ptr %mal_context, ptr {value})"
                ));
                Some(retained)
            }
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    self.line(format!(
                        "  {field} = extractvalue {} {value}, {index}",
                        aggregate_type.llvm
                    ));
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

    fn release_value(&mut self, ty: &Type, value: &str) -> Option<()> {
        match ty {
            Type::Symbol => self.line(format!(
                "  call void @mal_runtime_symbol_release(ptr {value})"
            )),
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    self.line(format!(
                        "  {field} = extractvalue {} {value}, {index}",
                        aggregate_type.llvm
                    ));
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
        self.line(format!(
            "  {tag} = extractvalue {} {value}, 0",
            sum_type.llvm
        ));
        let cases = members
            .iter()
            .enumerate()
            .map(|(index, _)| format!("    i32 {index}, label %mal_{operation}_{id}_{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        self.line(format!(
            "  switch i32 {tag}, label %mal_{operation}_{id}_invalid [\n{cases}\n  ]"
        ));
        self.line(format!("mal_{operation}_{id}_invalid:"));
        self.line("  unreachable");
        for (index, member) in members.iter().enumerate() {
            self.line(format!("mal_{operation}_{id}_{index}:"));
            if crate::execution::ownership::is_managed(member) {
                let payload = self.register();
                self.line(format!(
                    "  {payload} = extractvalue {} {value}, {}",
                    sum_type.llvm,
                    index + 1
                ));
                if retain {
                    self.retain_value(member, &payload)?;
                } else {
                    self.release_value(member, &payload)?;
                }
            }
            self.line(format!("  br label %mal_{operation}_{id}_done"));
        }
        self.line(format!("mal_{operation}_{id}_done:"));
        Some(())
    }
}
