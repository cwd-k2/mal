use mal_frontend::check::ast::Type;

/// The place lifecycle required by a runtime value. `Managed` types need type-directed share and drop glue;
/// `Trivial` values can be copied and discarded as bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Lifecycle {
    Trivial,
    Managed,
}

impl Lifecycle {
    pub(crate) const fn is_managed(self) -> bool {
        matches!(self, Self::Managed)
    }
}

/// Classifies lifecycle independently from any particular storage representation.
pub(crate) fn lifecycle(ty: &Type) -> Lifecycle {
    if ty
        .data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Buffer(_) | Type::Function { .. }))
    {
        Lifecycle::Managed
    } else {
        Lifecycle::Trivial
    }
}

pub(crate) fn is_managed(ty: &Type) -> bool {
    lifecycle(ty).is_managed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_lifecycle_once_per_shared_type_dag_node() {
        let mut unmanaged = Type::Unit;
        for _ in 0..64 {
            unmanaged = Type::Product(vec![unmanaged.clone(), unmanaged].into());
        }
        assert_eq!(lifecycle(&unmanaged), Lifecycle::Trivial);

        let managed = Type::Product(vec![unmanaged, Type::Symbol].into());
        assert_eq!(lifecycle(&managed), Lifecycle::Managed);
    }
}
