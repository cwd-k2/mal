//! The host-visible types reached by extern signatures and their source aliases.

use super::*;

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
                match ty {
                    Type::Product(elements) | Type::Sum(elements) => {
                        pending.extend(elements.iter())
                    }
                    Type::Buffer(element) => pending.push(element),
                    _ => {}
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
                match ty {
                    Type::Product(elements) | Type::Sum(elements) => {
                        pending.extend(elements.iter())
                    }
                    Type::Buffer(element) => pending.push(element),
                    _ => {}
                }
            }
        }
    }

    pub(in crate::backend::c::types) fn contains(&self, ty: &Type) -> bool {
        ty.shared_id()
            .is_some_and(|id| self.collected.contains(&id))
            || self.types.contains(ty)
    }

    pub(in crate::backend::c::types) fn external_contains(&self, ty: &Type) -> bool {
        ty.shared_id()
            .is_some_and(|id| self.external_collected.contains(&id))
            || self.external_types.contains(ty)
    }

    pub(in crate::backend::c::types) fn exposes_alias(
        &self,
        alias: &crate::core::ast::TypeAlias,
    ) -> bool {
        self.external_aliases.contains(&alias.name)
    }

    pub(in crate::backend::c::types) fn exposes_external_alias(
        &self,
        alias: &crate::core::ast::TypeAlias,
    ) -> bool {
        self.external_aliases.contains(&alias.name)
    }
}
