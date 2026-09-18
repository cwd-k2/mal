use crate::check::ast::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ManagedPath(Vec<PathStep>);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PathStep {
    ProductField(usize),
    SumPayload(usize),
}

impl ManagedPath {
    pub(crate) fn root() -> Self {
        Self(Vec::new())
    }
}

pub(crate) fn is_managed(ty: &Type) -> bool {
    ty.data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Packed(_) | Type::Function { .. }))
}

pub(crate) fn managed_paths(ty: &Type) -> ManagedPaths<'_> {
    ManagedPaths {
        pending: is_managed(ty)
            .then(|| (ty, ManagedPath::root()))
            .into_iter()
            .collect(),
    }
}

pub(crate) struct ManagedPaths<'a> {
    pending: Vec<(&'a Type, ManagedPath)>,
}

impl Iterator for ManagedPaths<'_> {
    type Item = ManagedPath;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((ty, path)) = self.pending.pop() {
            match ty {
                Type::Symbol | Type::Packed(_) | Type::Function { .. } => return Some(path),
                Type::Product(elements) => {
                    for (index, element) in elements.iter().enumerate().rev() {
                        let mut child = path.clone();
                        child.0.push(PathStep::ProductField(index));
                        self.pending.push((element, child));
                    }
                }
                Type::Sum(members) => {
                    for (index, member) in members.iter().enumerate().rev() {
                        let mut child = path.clone();
                        child.0.push(PathStep::SumPayload(index));
                        self.pending.push((member, child));
                    }
                }
                Type::Unit
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
                | Type::Address
                | Type::ByteSize
                | Type::USize
                | Type::Parameter { .. }
                | Type::Cursor(_)
                | Type::Region(_)
                | Type::External { .. } => {}
            }
        }
        None
    }
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

    #[test]
    fn identifies_every_managed_leaf_occurrence_by_path() {
        let shared = Type::Product(vec![Type::Symbol, Type::UInt64].into());
        let ty = Type::Product(
            vec![
                shared.clone(),
                Type::Sum(vec![Type::Unit, shared].into()),
                Type::Packed(Type::UInt8.into()),
                Type::Function {
                    parameter: Type::Symbol.into(),
                    result: Type::Symbol.into(),
                },
            ]
            .into(),
        );

        let paths = managed_paths(&ty).collect::<Vec<_>>();
        assert_eq!(
            paths
                .iter()
                .map(|path| path.0.as_slice())
                .collect::<Vec<_>>(),
            [
                &[PathStep::ProductField(0), PathStep::ProductField(0)][..],
                &[
                    PathStep::ProductField(1),
                    PathStep::SumPayload(1),
                    PathStep::ProductField(0)
                ],
                &[PathStep::ProductField(2)],
                &[PathStep::ProductField(3)],
            ]
        );
    }

    #[test]
    fn treats_closure_and_packed_owners_as_leaves() {
        let closure = Type::Function {
            parameter: Type::Product(vec![Type::Symbol].into()).into(),
            result: Type::Packed(Type::Symbol.into()).into(),
        };
        assert_eq!(
            managed_paths(&closure).collect::<Vec<_>>(),
            [ManagedPath::root()]
        );
        assert_eq!(
            managed_paths(&Type::Packed(Type::Symbol.into())).collect::<Vec<_>>(),
            [ManagedPath::root()]
        );
    }
}
