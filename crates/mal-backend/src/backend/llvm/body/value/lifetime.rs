use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
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
        let value_type = self.types.value(&slot.ty)?;
        let value = self.register();
        self.load(
            value.clone(),
            value_type.llvm.clone(),
            format!("%mal_slot_{}", slot.index),
            value_type.alignment,
            [],
        );
        self.release_value(&slot.ty, &value)?;
        self.store(
            value_type.llvm,
            "zeroinitializer",
            format!("%mal_slot_{}", slot.index),
            value_type.alignment,
            [],
        );
        Some(())
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
                self.extract_value(owner.clone(), value_type.llvm, value, [0]);
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Pointer,
                    "mal_runtime_bytes_retain",
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (crate::backend::llvm::syntax::Type::Pointer, owner),
                    ],
                );
                Some(value.into())
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                self.extract_value(environment.clone(), value_type.llvm, value, [1]);
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Pointer,
                    "mal_runtime_environment_retain",
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (crate::backend::llvm::syntax::Type::Pointer, environment),
                    ],
                );
                Some(value.into())
            }
            Type::Buffer(_) => {
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Pointer,
                    "mal_runtime_environment_retain",
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (crate::backend::llvm::syntax::Type::Pointer, value.into()),
                    ],
                );
                Some(value.into())
            }
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    self.extract_value(field.clone(), aggregate_type.llvm.clone(), value, [index]);
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
                self.extract_value(owner.clone(), value_type.llvm, value, [0]);
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    "mal_runtime_bytes_release",
                    [(crate::backend::llvm::syntax::Type::Pointer, owner)],
                );
            }
            Type::Function { .. } => {
                let value_type = self.types.value(ty)?;
                let environment = self.register();
                self.extract_value(environment.clone(), value_type.llvm, value, [1]);
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    "mal_runtime_environment_release",
                    [(crate::backend::llvm::syntax::Type::Pointer, environment)],
                )
            }
            Type::Buffer(_) => self.direct_call(
                None,
                false,
                crate::backend::llvm::syntax::Type::Void,
                "mal_runtime_environment_release",
                [(crate::backend::llvm::syntax::Type::Pointer, value.into())],
            ),
            Type::Product(elements) => {
                let aggregate_type = self.types.value(ty)?;
                for (index, element) in elements.iter().enumerate() {
                    if !crate::execution::ownership::is_managed(element) {
                        continue;
                    }
                    let field = self.register();
                    self.extract_value(field.clone(), aggregate_type.llvm.clone(), value, [index]);
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
        self.extract_value(tag.clone(), sum_type.llvm, value, [0]);
        let cases = members
            .iter()
            .enumerate()
            .map(|(index, _)| (index.to_string(), format!("mal_{operation}_{id}_{index}")));
        self.terminate(crate::backend::llvm::syntax::Terminator::switch(
            "i32",
            tag,
            format!("mal_{operation}_{id}_invalid"),
            cases,
        ));
        self.block(format!("mal_{operation}_{id}_invalid"));
        self.unreachable();
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
            self.branch(format!("mal_{operation}_{id}_done"));
        }
        self.block(format!("mal_{operation}_{id}_done"));
        Some(())
    }
}
