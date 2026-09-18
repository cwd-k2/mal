use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};

use super::scalar::{integer_literal, scalar_type};
use super::{EmittedValue, FunctionEmitter, PreparedValue};
use crate::execution::ownership::UseEffect;

impl FunctionEmitter<'_> {
    pub(super) fn require_binding_borrow(
        &self,
        site: crate::control::ast::StateId,
        binding: usize,
        operand: crate::execution::ownership::BindingOperand,
        atom: &Atom,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&atom.ty) {
            return Some(());
        }
        (self.ownership.binding_use(site, binding, operand) == Some(UseEffect::Borrow))
            .then_some(())
    }

    pub(super) fn require_terminator_borrow(
        &self,
        site: crate::control::ast::StateId,
        operand: crate::execution::ownership::TerminatorOperand,
        atom: &Atom,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&atom.ty) {
            return Some(());
        }
        (self.ownership.terminator_use(site, operand) == Some(UseEffect::Borrow)).then_some(())
    }

    pub(super) fn prepare_binding_for_use(
        &mut self,
        id: crate::anf::ast::ValueId,
        effect: UseEffect,
    ) -> Option<PreparedValue> {
        let slot = self.slots.get(&id)?.clone();
        let value_type = self.types.value(&slot.ty)?;
        let register = self.register();
        self.line(format!(
            "  {register} = load {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        let mut value = EmittedValue {
            ty: slot.ty.clone(),
            representation: register,
            owned: false,
        };
        let consumed_slots = match effect {
            UseEffect::Borrow => Vec::new(),
            UseEffect::Share => {
                self.retain_if_borrowed(&mut value)?;
                Vec::new()
            }
            UseEffect::Consume => {
                value.owned = true;
                vec![slot]
            }
        };
        Some(PreparedValue {
            value,
            consumed_slots,
        })
    }

    pub(super) fn prepare_atom_for_use(
        &mut self,
        atom: &Atom,
        effect: UseEffect,
    ) -> Option<PreparedValue> {
        let mut value = self.atom(atom)?;
        let mut consumed_slots = Vec::new();
        match effect {
            UseEffect::Borrow => {}
            UseEffect::Share => self.retain_if_borrowed(&mut value)?,
            UseEffect::Consume => {
                let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
                    return None;
                };
                let slot = self.slots.get(&id)?.clone();
                if slot.ty != atom.ty || !crate::execution::ownership::is_managed(&slot.ty) {
                    return None;
                }
                value.owned = true;
                consumed_slots.push(slot);
            }
        }
        Some(PreparedValue {
            value,
            consumed_slots,
        })
    }

    pub(super) fn commit_consumes(&mut self, prepared: &PreparedValue) -> Option<()> {
        for slot in &prepared.consumed_slots {
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
                value_type.llvm, slot.index, value_type.alignment
            ));
        }
        Some(())
    }

    pub(super) fn atom(&mut self, atom: &Atom) -> Option<EmittedValue> {
        match (&atom.ty, &atom.kind) {
            (ty, AtomKind::Integer(value))
                if scalar_type(ty, self.types.index_size()).is_some() =>
            {
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: integer_literal(ty, *value, self.types.index_size())?,
                    owned: false,
                })
            }
            (Type::Float32, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float32,
                representation: format!(
                    "0x{:016X}",
                    (f32::from_bits(*bits as u32) as f64).to_bits()
                ),
                owned: false,
            }),
            (Type::Float64, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float64,
                representation: format!("0x{bits:016X}"),
                owned: false,
            }),
            (Type::ByteSize, AtomKind::StorageSize(measured)) => Some(EmittedValue {
                ty: Type::ByteSize,
                representation: self.source_layouts.layout(measured)?.stride.to_string(),
                owned: false,
            }),
            (Type::Symbol, AtomKind::Symbol(bytes)) => {
                if bytes.is_empty() {
                    return self.make_byte_view(&Type::Symbol, "null", "0", "0", true);
                }
                let name = format!(
                    "mal_symbol_literal_{}_{}",
                    super::function_number(self.function.id)?,
                    atom.id.0
                );
                self.globals
                    .push_str(&super::symbol::literal_definition(&name, bytes));
                self.make_byte_view(
                    &Type::Symbol,
                    &format!("@{name}"),
                    "0",
                    &bytes.len().to_string(),
                    true,
                )
            }
            (Type::Function { .. }, AtomKind::Reference(Reference::SelfClosure(function))) => {
                let value_type = self.types.value(&atom.ty)?;
                let with_code = self.register();
                self.line(format!(
                    "  {with_code} = insertvalue {} zeroinitializer, ptr @{}, 0",
                    value_type.llvm,
                    super::function_name(*function)?
                ));
                let environment = self.active_environment();
                let closure = self.register();
                self.line(format!(
                    "  {closure} = insertvalue {} {with_code}, ptr {environment}, 1",
                    value_type.llvm,
                ));
                Some(EmittedValue {
                    ty: atom.ty.clone(),
                    representation: closure,
                    owned: false,
                })
            }
            (ty, AtomKind::Reference(Reference::EnvironmentField(index))) => {
                let function = self.current_function()?;
                let field = function.environment.get(*index)?;
                if field.ty != *ty {
                    return None;
                }
                let environment_type = Type::Product(
                    function
                        .environment
                        .iter()
                        .map(|field| field.ty.clone())
                        .collect(),
                );
                let fields = self.types.product_fields(&environment_type)?;
                let offset = fields.get(*index)?.offset;
                let environment = self.active_environment();
                let pointer = self.register();
                self.line(format!(
                    "  {pointer} = getelementptr i8, ptr {environment}, i64 {offset}"
                ));
                let value_type = self.types.value(ty)?;
                let value = self.register();
                self.line(format!(
                    "  {value} = load {}, ptr {pointer}, align {}",
                    value_type.llvm, value_type.alignment
                ));
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: value,
                    owned: false,
                })
            }
            (ty, AtomKind::Reference(Reference::Binding(id))) if !self.slots.contains_key(id) => {
                let constant = self.top_levels.get(*id)?.clone();
                if constant.ty != *ty {
                    return None;
                }
                self.constant(constant)
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

    fn constant(&mut self, constant: super::plan::Constant) -> Option<EmittedValue> {
        if let Some(value) = constant.value() {
            return Some(EmittedValue {
                ty: constant.ty.clone(),
                representation: value.into(),
                owned: false,
            });
        }
        if let Some(elements) = constant.product() {
            let Type::Product(types) = &constant.ty else {
                return None;
            };
            if elements.len() != types.len() {
                return None;
            }
            let aggregate_type = self.types.value(&constant.ty)?;
            let mut aggregate = "poison".to_owned();
            for (index, (element, expected)) in elements.iter().zip(types.iter()).enumerate() {
                let element = self.constant(element.clone())?;
                if element.ty != *expected {
                    return None;
                }
                let element_type = self.types.value(expected)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = insertvalue {} {aggregate}, {} {}, {index}",
                    aggregate_type.llvm, element_type.llvm, element.representation
                ));
                aggregate = register;
            }
            return Some(EmittedValue {
                ty: constant.ty,
                representation: aggregate,
                owned: false,
            });
        }
        let (index, value) = constant.sum()?;
        let value = self.constant(value.clone())?;
        self.emit_sum_value(index, value, &constant.ty, false)
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
                    elements.iter().zip(element_types.iter()).enumerate()
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

    pub(super) fn release_dead_slot(&mut self, id: crate::anf::ast::ValueId) -> Option<()> {
        let Some(slot) = self.slots.get(&id).cloned() else {
            return Some(());
        };
        if !crate::execution::ownership::is_managed(&slot.ty) {
            return Some(());
        }
        self.release_slot(&slot)
    }

    fn release_slot(&mut self, slot: &super::Slot) -> Option<()> {
        let value_type = self.types.value(&slot.ty)?;
        let value = self.register();
        self.line(format!(
            "  {value} = load {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        self.release_value(&slot.ty, &value)?;
        self.line(format!(
            "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        Some(())
    }

    pub(super) fn release_local_managed(&mut self) {
        let mut slots = self
            .function_slots
            .get(&self.current_function)
            .into_iter()
            .flatten()
            .filter_map(|id| self.slots.get(id))
            .filter(|slot| crate::execution::ownership::is_managed(&slot.ty))
            .cloned()
            .collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            self.release_slot(&slot).expect("managed slot is supported");
        }
    }

    fn retain_value(&mut self, ty: &Type, value: &str) -> Option<String> {
        match ty {
            Type::Symbol | Type::Packed(_) => {
                let value_type = self.types.value(ty)?;
                let owner = self.register();
                self.line(format!(
                    "  {owner} = extractvalue {} {value}, 0",
                    value_type.llvm
                ));
                self.line(format!(
                    "  call ptr @mal_runtime_bytes_retain(ptr %mal_context, ptr {owner})"
                ));
                Some(value.into())
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                self.line(format!(
                    "  {environment} = extractvalue {} {value}, 1",
                    value_type.llvm
                ));
                self.line(format!(
                    "  call ptr @mal_runtime_environment_retain(ptr %mal_context, ptr {environment})"
                ));
                Some(value.into())
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

    pub(super) fn release_value(&mut self, ty: &Type, value: &str) -> Option<()> {
        match ty {
            Type::Symbol | Type::Packed(_) => {
                let value_type = self.types.value(ty)?;
                let owner = self.register();
                self.line(format!(
                    "  {owner} = extractvalue {} {value}, 0",
                    value_type.llvm
                ));
                self.line(format!(
                    "  call void @mal_runtime_bytes_release(ptr {owner})"
                ));
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                self.line(format!(
                    "  {environment} = extractvalue {} {value}, 1",
                    value_type.llvm
                ));
                self.line(format!(
                    "  call void @mal_runtime_environment_release(ptr {environment})"
                ))
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
                let payload = self.emit_sum_payload(ty, member, value)?;
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
