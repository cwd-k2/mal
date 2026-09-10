use crate::check::ast::Type;

mod transfer;

pub(crate) use transfer::OwnershipPlan;

pub(crate) fn is_managed(ty: &Type) -> bool {
    match ty {
        Type::Symbol | Type::Function { .. } => true,
        Type::Product(elements) | Type::Sum(elements) => elements.iter().any(is_managed),
        Type::External { .. }
        | Type::Unit
        | Type::Int8
        | Type::Int16
        | Type::Int32
        | Type::Int64
        | Type::UInt8
        | Type::UInt16
        | Type::UInt32
        | Type::UInt64
        | Type::Float32
        | Type::Float64
        | Type::Ptr => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::source::{FileId, SourceFile};

    #[test]
    fn preserves_transfer_decisions_when_atoms_cross_stage_boundaries() {
        let source = SourceFile::new(
            FileId::new(78),
            "ownership-identity.mal",
            "take :: Unit -> Symbol := \\() { value := \"x\"; value; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check ownership fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let program = crate::closure::convert(&anf);
        let execution = crate::execution::lower(program);
        let result = execution
            .control
            .states
            .iter()
            .find_map(|state| match &state.terminator {
                crate::control::ast::Terminator::Return(result) => Some(result),
                _ => None,
            })
            .expect("return atom");

        assert!(execution.ownership.can_transfer(result, false));
    }
}
