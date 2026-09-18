use crate::check::ast::Type;

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
            Type::Function { .. } if self.function_type_is_scoped(ty) => Some(value.into()),
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

    pub(in crate::backend::llvm::body) fn release_value(
        &mut self,
        ty: &Type,
        value: &str,
    ) -> Option<()> {
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
            Type::Function { .. } if self.function_type_is_scoped(ty) => {}
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

    fn function_type_is_scoped(&self, ty: &Type) -> bool {
        function_type_is_scoped(self.index.lowered_functions.values().copied(), ty)
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

fn function_type_is_scoped<'a>(
    functions: impl Iterator<Item = &'a crate::closure::ast::Function>,
    ty: &Type,
) -> bool {
    let Type::Function { parameter, result } = ty else {
        return false;
    };
    let mut matching = functions.filter(|function| {
        function.parameter.ty == **parameter && function.body.result.ty == **result
    });
    matching.next().is_some_and(|first| {
        first.kind.has_scoped_environment()
            && matching.all(|function| function.kind.has_scoped_environment())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    fn lowered(source: &str) -> crate::closure::ast::Program {
        let source = SourceFile::new(FileId::new(101), "scoped-functions.mal", source.into());
        let checked = crate::pipeline::check(&source).expect("check scoped function fixture");
        let specialized = crate::check::specialize(checked).expect("specialize scoped fixture");
        crate::closure::convert(&crate::anf::lower(&crate::core::lower(&specialized)))
    }

    fn reader_type() -> Type {
        Type::Function {
            parameter: Type::USize.into(),
            result: Type::Int64.into(),
        }
    }

    #[test]
    fn classifies_a_function_type_from_every_closed_program_inhabitant() {
        let program = lowered(
            "fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (_, get, _) -> { _ := get(0usize); (); };\n\
             main :: Unit -> Int32 := () -> { _ := pack<Int64>(fill); 0i32; };",
        );

        assert!(function_type_is_scoped(
            program.functions.iter(),
            &reader_type()
        ));
    }

    #[test]
    fn preserves_managed_lifetime_when_an_ordinary_inhabitant_matches() {
        let program = lowered(
            "read :: USize -> Int64 := (_) -> { 0i64 };\n\
             fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (_, get, _) -> { _ := get(0usize); (); };\n\
             main :: Unit -> Int32 := () -> { _ := pack<Int64>(fill); _ := read(0usize); 0i32; };",
        );

        assert!(!function_type_is_scoped(
            program.functions.iter(),
            &reader_type()
        ));
    }
}
