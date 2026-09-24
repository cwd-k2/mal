use mal_frontend::check::ast::Type;

pub(crate) fn is_managed(ty: &Type) -> bool {
    ty.data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Buffer(_) | Type::Function { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_shared_type_dags_once_per_node() {
        let mut unmanaged = Type::Unit;
        for _ in 0..64 {
            unmanaged = Type::Product(vec![unmanaged.clone(), unmanaged].into());
        }
        assert!(!is_managed(&unmanaged));

        let managed = Type::Product(vec![unmanaged, Type::Symbol].into());
        assert!(is_managed(&managed));
    }
}
