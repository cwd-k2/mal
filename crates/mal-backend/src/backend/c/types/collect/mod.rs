//! Interning of host-visible types into the registry under one structural identity.

use crate::core::ast::ProgramInterface;
use mal_frontend::check::ast::Type;

use super::{HostTypes, TypeRegistry, is_bool};

mod interface;
#[cfg(test)]
mod tests;

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
                Type::Buffer(element) => {
                    pending.push((element, false));
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
                | Type::ByteSize
                | Type::USize => {}
                Type::Parameter { .. }
                | Type::Bound { .. }
                | Type::Application { .. }
                | Type::Abstraction { .. }
                | Type::Opaque { .. } => {
                    unreachable!("open types are not extern carriers")
                }
            }
        }
    }

    pub(super) fn index(&self, ty: &Type) -> super::RepresentationId {
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
        let id = if let Some(id) = self.structural_indices.get(&key) {
            *id
        } else {
            let id = super::RepresentationId(fingerprint(&key));
            if let Some(existing) = self.fingerprint_keys.get(&id) {
                assert_eq!(
                    existing, &key,
                    "distinct C host aggregate representations have the same fingerprint"
                );
            }
            self.aggregates.push(ty.clone());
            self.fingerprint_keys.insert(id, key.clone());
            self.structural_indices.insert(key, id);
            id
        };
        self.indices.insert(
            ty.shared_id()
                .expect("aggregate types have shared identity"),
            id,
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
            Type::Symbol => super::ElementKey::Symbol,
            Type::Buffer(element) => super::ElementKey::Buffer(self.element_id(element)),
            Type::ByteSize => super::ElementKey::ByteSize,
            Type::USize => super::ElementKey::USize,
            Type::External { name, .. } => super::ElementKey::External(name.clone()),
            Type::Product(_) | Type::Sum(_) => super::ElementKey::Aggregate(self.index(ty)),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. }
            | Type::Abstraction { .. }
            | Type::Opaque { .. } => {
                unreachable!("open types are not extern carriers")
            }
        }
    }
}

fn fingerprint(key: &super::AggregateKey) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn write_byte(state: &mut u64, byte: u8) {
        *state ^= u64::from(byte);
        *state = state.wrapping_mul(PRIME);
    }

    fn write_bytes(state: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            write_byte(state, *byte);
        }
        write_byte(state, 0xff);
    }

    fn write_element(state: &mut u64, element: &super::ElementKey) {
        let tag = match element {
            super::ElementKey::Unit => 0,
            super::ElementKey::Int8 => 1,
            super::ElementKey::Int16 => 2,
            super::ElementKey::Int32 => 3,
            super::ElementKey::Int64 => 4,
            super::ElementKey::UInt8 => 5,
            super::ElementKey::UInt16 => 6,
            super::ElementKey::UInt32 => 7,
            super::ElementKey::UInt64 => 8,
            super::ElementKey::Float32 => 9,
            super::ElementKey::Float64 => 10,
            super::ElementKey::Symbol => 11,
            super::ElementKey::Buffer(_) => 12,
            super::ElementKey::ByteSize => 13,
            super::ElementKey::USize => 14,
            super::ElementKey::External(_) => 15,
            super::ElementKey::Aggregate(_) => 16,
        };
        write_byte(state, tag);
        match element {
            super::ElementKey::External(name) => write_bytes(state, name.as_bytes()),
            super::ElementKey::Aggregate(id) | super::ElementKey::Buffer(id) => {
                write_bytes(state, &id.0.to_le_bytes())
            }
            _ => {}
        }
    }

    let (kind, elements) = match key {
        super::AggregateKey::Product(elements) => (0_u8, elements),
        super::AggregateKey::Sum(elements) => (1_u8, elements),
    };
    let mut state = OFFSET;
    write_byte(&mut state, kind);
    write_bytes(&mut state, &(elements.len() as u64).to_le_bytes());
    for element in elements {
        write_element(&mut state, element);
    }
    state
}
