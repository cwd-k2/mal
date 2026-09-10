use crate::check::ast::Type;
use crate::core::ast::ProgramInterface;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    fn collect(&mut self, ty: &Type) {
        if is_bool(ty) {
            return;
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect(element);
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
            | Type::Ptr => return,
        }
        if !self.aggregates.contains(ty) {
            self.aggregates.push(ty.clone());
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
        if !self.types.contains(ty) {
            self.types.push(ty.clone());
        }
        if is_bool(ty) {
            return;
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect_type(element, registry);
                }
                registry.collect(ty);
            }
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => {}
        }
    }

    pub(super) fn contains(&self, ty: &Type) -> bool {
        self.types.contains(ty)
    }
}
