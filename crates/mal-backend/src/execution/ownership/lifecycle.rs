use mal_frontend::check::ast::Type;

/// The place lifecycle required by a runtime value. `Owned` values need type-directed share and drop glue;
/// `Trivial` values can be copied and discarded as bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Lifecycle {
    Trivial,
    Owned,
}

impl Lifecycle {
    pub(crate) const fn is_owned(self) -> bool {
        matches!(self, Self::Owned)
    }
}

/// Classifies lifecycle independently from any particular storage representation.
pub(crate) fn lifecycle(ty: &Type) -> Lifecycle {
    if ty
        .data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Buffer(_) | Type::Function { .. }))
    {
        Lifecycle::Owned
    } else {
        Lifecycle::Trivial
    }
}

pub(crate) fn is_managed(ty: &Type) -> bool {
    lifecycle(ty).is_owned()
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
        assert_eq!(lifecycle(&managed), Lifecycle::Owned);
    }
}
