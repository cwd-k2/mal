//! Memoized structural fingerprints used only as grouping keys, never as type equality authority.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::ast::{SharedTypeId, Type};

#[derive(Default)]
/// Cache of structural type hashes used to form collision-checked grouping buckets.
///
/// A fingerprint is never proof of type equality; callers must compare the canonical types after
/// selecting a bucket.
pub struct TypeFingerprints {
    cache: HashMap<SharedTypeId, u64>,
}

impl TypeFingerprints {
    pub(crate) fn arguments(&mut self, arguments: &[Type]) -> u64 {
        let mut hasher = DefaultHasher::new();
        arguments.len().hash(&mut hasher);
        for argument in arguments {
            self.ty(argument).hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Returns separate structural grouping keys for a function parameter and result.
    ///
    /// Callers still compare admitted types for equality after grouping; hash equality is not type equality.
    pub fn signature(&mut self, parameter: &Type, result: &Type) -> (u64, u64) {
        (self.ty(parameter), self.ty(result))
    }

    fn ty(&mut self, ty: &Type) -> u64 {
        let mut pending = vec![Fingerprint::Type(ty)];
        let mut values = Vec::new();
        while let Some(item) = pending.pop() {
            match item {
                Fingerprint::Type(ty) => {
                    if let Some(hash) = ty.shared_id().and_then(|id| self.cache.get(&id).copied()) {
                        values.push(hash);
                        continue;
                    }
                    match ty {
                        Type::Product(elements) => {
                            pending.push(Fingerprint::Aggregate(
                                ty.shared_id(),
                                15,
                                elements.len(),
                            ));
                            pending.extend(elements.iter().rev().map(Fingerprint::Type));
                        }
                        Type::Sum(elements) => {
                            pending.push(Fingerprint::Aggregate(
                                ty.shared_id(),
                                16,
                                elements.len(),
                            ));
                            pending.extend(elements.iter().rev().map(Fingerprint::Type));
                        }
                        Type::Function { parameter, result } => {
                            pending.push(Fingerprint::Aggregate(ty.shared_id(), 17, 2));
                            pending.push(Fingerprint::Type(result));
                            pending.push(Fingerprint::Type(parameter));
                        }
                        Type::Buffer(element) => {
                            pending.push(Fingerprint::Aggregate(None, 18, 1));
                            pending.push(Fingerprint::Type(element));
                        }
                        Type::Opaque { id, arguments, .. } => {
                            pending.push(Fingerprint::Opaque(*id, arguments.len()));
                            pending.extend(arguments.iter().rev().map(Fingerprint::Type));
                        }
                        Type::Application {
                            constructor,
                            argument,
                            kind,
                            ..
                        } => {
                            pending.push(Fingerprint::Kinded(20, kind, 2));
                            pending.push(Fingerprint::Type(argument));
                            pending.push(Fingerprint::Type(constructor));
                        }
                        Type::Abstraction {
                            parameter_kind,
                            body,
                        } => {
                            pending.push(Fingerprint::Kinded(21, parameter_kind, 1));
                            pending.push(Fingerprint::Type(body));
                        }
                        _ => values.push(atom_fingerprint(ty)),
                    }
                }
                Fingerprint::Aggregate(id, tag, length) => {
                    let children = values.split_off(values.len() - length);
                    let mut hasher = DefaultHasher::new();
                    tag.hash(&mut hasher);
                    children.hash(&mut hasher);
                    let hash = hasher.finish();
                    if let Some(id) = id {
                        self.cache.insert(id, hash);
                    }
                    values.push(hash);
                }
                Fingerprint::Opaque(id, length) => {
                    let children = values.split_off(values.len() - length);
                    let mut hasher = DefaultHasher::new();
                    19_u8.hash(&mut hasher);
                    id.hash(&mut hasher);
                    children.hash(&mut hasher);
                    values.push(hasher.finish());
                }
                Fingerprint::Kinded(tag, kind, length) => {
                    let children = values.split_off(values.len() - length);
                    let mut hasher = DefaultHasher::new();
                    tag.hash(&mut hasher);
                    kind.hash(&mut hasher);
                    children.hash(&mut hasher);
                    values.push(hasher.finish());
                }
            }
        }
        values.pop().expect("one type produces one fingerprint")
    }
}

enum Fingerprint<'a> {
    Type(&'a Type),
    Aggregate(Option<SharedTypeId>, u8, usize),
    Opaque(crate::resolve::ast::TypeId, usize),
    Kinded(u8, &'a super::ast::Kind, usize),
}

fn atom_fingerprint(ty: &Type) -> u64 {
    let mut hasher = DefaultHasher::new();
    match ty {
        Type::Unit => 0_u8.hash(&mut hasher),
        Type::Int8 => 1_u8.hash(&mut hasher),
        Type::Int16 => 2_u8.hash(&mut hasher),
        Type::Int32 => 3_u8.hash(&mut hasher),
        Type::Int64 => 4_u8.hash(&mut hasher),
        Type::UInt8 => 5_u8.hash(&mut hasher),
        Type::UInt16 => 6_u8.hash(&mut hasher),
        Type::UInt32 => 7_u8.hash(&mut hasher),
        Type::UInt64 => 8_u8.hash(&mut hasher),
        Type::Float32 => 9_u8.hash(&mut hasher),
        Type::Float64 => 10_u8.hash(&mut hasher),
        Type::Symbol => 11_u8.hash(&mut hasher),
        Type::ByteSize => 13_u8.hash(&mut hasher),
        Type::USize => 14_u8.hash(&mut hasher),
        Type::External { id, name } => {
            15_u8.hash(&mut hasher);
            id.hash(&mut hasher);
            name.hash(&mut hasher);
        }
        Type::Parameter { id, name, kind } => {
            16_u8.hash(&mut hasher);
            id.hash(&mut hasher);
            name.hash(&mut hasher);
            kind.hash(&mut hasher);
        }
        Type::Bound { index, kind } => {
            17_u8.hash(&mut hasher);
            index.hash(&mut hasher);
            kind.hash(&mut hasher);
        }
        Type::Product(_)
        | Type::Sum(_)
        | Type::Function { .. }
        | Type::Buffer(_)
        | Type::Opaque { .. }
        | Type::Application { .. }
        | Type::Abstraction { .. } => {
            unreachable!("aggregate fingerprints are composed from their children")
        }
    }
    hasher.finish()
}
