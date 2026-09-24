use crate::check::ast::Type;
use crate::core::ast::ProgramInterface;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(super) fn collect(&mut self, ty: &Type) {
        let mut pending = vec![(ty, false)];
        while let Some((ty, expanded)) = pending.pop() {
            if is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(elements) | Type::Sum(elements) => {
                    if expanded {
                        self.intern(ty, elements);
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
                | Type::Address
                | Type::ByteSize
                | Type::USize => {}
                Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                    unreachable!("open or memory-indexed types are not host mappable")
                }
            }
        }
    }

    pub(super) fn index(&self, ty: &Type) -> usize {
        let id = ty
            .shared_id()
            .expect("only aggregates have representation indices");
        self.indices
            .get(&id)
            .copied()
            .expect("all emitted types are collected before rendering")
    }

    fn intern(&mut self, ty: &Type, elements: &[Type]) {
        let key = match ty {
            Type::Product(_) => super::AggregateKey::Product(
                elements.iter().map(|ty| self.element_key(ty)).collect(),
            ),
            Type::Sum(_) => {
                super::AggregateKey::Sum(elements.iter().map(|ty| self.element_key(ty)).collect())
            }
            _ => unreachable!("only aggregates are interned"),
        };
        let index = if let Some(index) = self.structural_indices.get(&key) {
            *index
        } else {
            let index = self.aggregates.len();
            self.aggregates.push(ty.clone());
            self.structural_indices.insert(key, index);
            index
        };
        self.indices.insert(
            ty.shared_id()
                .expect("aggregate types have shared identity"),
            index,
        );
    }

    fn element_key(&self, ty: &Type) -> super::ElementKey {
        match ty {
            Type::Unit => super::ElementKey::Unit,
            Type::Int8 => super::ElementKey::Int8,
            Type::Int16 => super::ElementKey::Int16,
            Type::Int32 => super::ElementKey::Int32,
            Type::Int64 => super::ElementKey::Int64,
            Type::UInt8 => super::ElementKey::UInt8,
            Type::UInt16 => super::ElementKey::UInt16,
            Type::UInt32 => super::ElementKey::UInt32,
            Type::UInt64 => super::ElementKey::UInt64,
            Type::Float32 => super::ElementKey::Float32,
            Type::Float64 => super::ElementKey::Float64,
            Type::Address => super::ElementKey::Address,
            Type::ByteSize => super::ElementKey::ByteSize,
            Type::USize => super::ElementKey::USize,
            Type::External { id, .. } => super::ElementKey::External(*id),
            Type::Product(_) | Type::Sum(_) => super::ElementKey::Aggregate(self.index(ty)),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                unreachable!("open or memory-indexed types are not host mappable")
            }
        }
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
            host.collect_external_type(&external.parameter, registry);
            host.collect_external_type(&external.result, registry);
            host.external_aliases.extend(
                external
                    .parameter_alias
                    .iter()
                    .chain(external.result_alias.iter())
                    .cloned(),
            );
            host.external_aliases
                .extend(external.parameter_aliases.iter().flatten().cloned());
        }
        let mut pending_aliases = host.external_aliases.iter().cloned().collect::<Vec<_>>();
        while let Some(name) = pending_aliases.pop() {
            let Some(alias) = interface
                .type_aliases
                .iter()
                .find(|alias| alias.name == name)
            else {
                continue;
            };
            for name in alias.element_aliases.iter().flatten() {
                if host.external_aliases.insert(name.clone()) {
                    pending_aliases.push(name.clone());
                }
            }
        }
        for alias in &interface.type_aliases {
            if alias.host_memory_access {
                host.collect_memory_type(&alias.ty, registry);
            }
        }
        for alias in &interface.type_aliases {
            if host.exposes_alias(alias) {
                registry.collect(&alias.ty);
            }
        }
        host
    }

    fn collect_type(&mut self, ty: &Type, registry: &mut TypeRegistry) {
        registry.collect(ty);
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            if matches!(ty, Type::Function { .. }) {
                unreachable!("type checking excludes functions from extern signatures")
            }
            let newly_collected = ty
                .shared_id()
                .map_or_else(|| !self.types.contains(ty), |id| self.collected.insert(id));
            if newly_collected {
                self.types.push(ty.clone());
                if let Type::Product(elements) | Type::Sum(elements) = ty {
                    pending.extend(elements.iter());
                }
            }
        }
    }

    fn collect_external_type(&mut self, ty: &Type, registry: &mut TypeRegistry) {
        self.collect_type(ty, registry);
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            let newly_collected = ty.shared_id().map_or_else(
                || !self.external_types.contains(ty),
                |id| self.external_collected.insert(id),
            );
            if newly_collected {
                self.external_types.push(ty.clone());
                if let Type::Product(elements) | Type::Sum(elements) = ty {
                    pending.extend(elements.iter());
                }
            }
        }
    }

    fn collect_memory_type(&mut self, ty: &Type, registry: &mut TypeRegistry) {
        self.collect_type(ty, registry);
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            let newly_collected = ty.shared_id().map_or_else(
                || !self.memory_types.contains(ty),
                |id| self.memory_collected.insert(id),
            );
            if newly_collected {
                self.memory_types.push(ty.clone());
                if let Type::Product(elements) | Type::Sum(elements) = ty {
                    pending.extend(elements.iter());
                }
            }
        }
    }

    pub(super) fn contains(&self, ty: &Type) -> bool {
        ty.shared_id()
            .is_some_and(|id| self.collected.contains(&id))
            || self.types.contains(ty)
    }

    pub(super) fn memory_contains(&self, ty: &Type) -> bool {
        ty.shared_id()
            .is_some_and(|id| self.memory_collected.contains(&id))
            || self.memory_types.contains(ty)
    }

    pub(super) fn external_contains(&self, ty: &Type) -> bool {
        ty.shared_id()
            .is_some_and(|id| self.external_collected.contains(&id))
            || self.external_types.contains(ty)
    }

    pub(super) fn exposes_alias(&self, alias: &crate::core::ast::TypeAlias) -> bool {
        alias.host_memory_access || self.external_aliases.contains(&alias.name)
    }

    pub(super) fn exposes_external_alias(&self, alias: &crate::core::ast::TypeAlias) -> bool {
        self.external_aliases.contains(&alias.name)
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

    #[test]
    fn interns_independent_structurally_equal_dags() {
        let mut left = Type::UInt8;
        let mut right = Type::UInt8;
        for _ in 0..64 {
            left = Type::Sum(vec![left.clone(), left].into());
            right = Type::Sum(vec![right.clone(), right].into());
        }
        let mut registry = TypeRegistry::default();

        registry.collect(&left);
        registry.collect(&right);

        assert_eq!(registry.aggregates.len(), 64);
        assert_eq!(registry.index(&left), registry.index(&right));
    }

    #[test]
    fn registers_host_visible_alias_dags_before_rendering() {
        let mut external = Type::UInt8;
        let mut alias = Type::UInt8;
        for _ in 0..64 {
            external = Type::Sum(vec![external.clone(), external].into());
            alias = Type::Sum(vec![alias.clone(), alias].into());
        }
        let interface = ProgramInterface {
            type_aliases: vec![crate::core::ast::TypeAlias {
                name: "Alias".into(),
                ty: alias.clone(),
                element_aliases: vec![None, None],
                host_memory_access: false,
            }],
            external_types: Vec::new(),
            externals: vec![crate::core::ast::ExternalOperation {
                id: crate::resolve::ast::ExternalOperationId(0),
                name: "inspect".into(),
                parameter: external.clone(),
                parameter_alias: Some("Alias".into()),
                parameter_aliases: vec![None, None],
                result: Type::Unit,
                result_alias: None,
                span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
            }],
        };
        let mut registry = TypeRegistry::default();

        HostTypes::collect(&interface, &mut registry);

        assert_eq!(registry.index(&external), registry.index(&alias));
    }
}
