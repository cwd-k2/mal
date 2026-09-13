use crate::check::ast::Type;
use crate::core::ast::ProgramInterface;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    fn collect(&mut self, ty: &Type) {
        let mut pending = vec![(ty, false)];
        while let Some((ty, expanded)) = pending.pop() {
            if is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(elements) | Type::Sum(elements) => {
                    if expanded {
                        if !self.aggregates.contains(ty) {
                            self.aggregates.push(ty.clone());
                        }
                    } else if ty.shared_id().is_none_or(|id| self.collected.insert(id)) {
                        pending.push((ty, true));
                        pending.extend(elements.iter().rev().map(|element| (element, false)));
                    }
                }
                Type::Function { .. } => {
                    unreachable!("type checking excludes functions from extern signatures")
                }
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
                | Type::Symbol
                | Type::Ptr => {}
            }
        }
    }

    pub(super) fn index(&self, ty: &Type) -> usize {
        self.aggregates
            .iter()
            .position(|candidate| candidate == ty)
            .expect("all emitted types are collected before rendering")
    }
}

impl HostTypes {
    pub(in crate::backend::c) fn collect(
        interface: &ProgramInterface,
        registry: &mut TypeRegistry,
    ) -> Self {
        let mut host = Self::default();
        host.opaque_names.extend(
            interface
                .external_types
                .iter()
                .map(|external| external.name.clone()),
        );
        for external in &interface.externals {
            host.collect_type(&external.parameter, registry);
            host.collect_type(&external.result, registry);
        }
        host
    }

    fn collect_type(&mut self, ty: &Type, registry: &mut TypeRegistry) {
        registry.collect(ty);
        for ty in ty.data_subtypes() {
            if matches!(ty, Type::Function { .. }) {
                unreachable!("type checking excludes functions from extern signatures")
            }
            if !self.types.contains(ty) {
                self.types.push(ty.clone());
            }
        }
    }

    pub(super) fn contains(&self, ty: &Type) -> bool {
        self.types.contains(ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_shared_aggregate_dags_in_dependency_order() {
        let mut ty = Type::Unit;
        for _ in 0..64 {
            ty = Type::Product(vec![ty.clone(), ty].into());
        }
        let mut registry = TypeRegistry::default();

        registry.collect(&ty);

        assert_eq!(registry.aggregates.len(), 64);
        assert_eq!(registry.aggregates.last(), Some(&ty));
    }
}
