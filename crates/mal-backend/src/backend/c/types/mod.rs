//! Structural registry and target-aware C mapping of host-visible types.

use crate::backend::c::syntax::{
    Directive, TranslationUnit, TypeName, c_expr, c_invocation, c_items, c_type,
};
use mal_frontend::check::ast::{SharedTypeId, Type};

mod collect;
mod declarations;
mod host;
#[cfg(test)]
mod tests;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    collected: std::collections::HashSet<SharedTypeId>,
    indices: std::collections::HashMap<SharedTypeId, RepresentationId>,
    structural_indices: std::collections::HashMap<AggregateKey, RepresentationId>,
    fingerprint_keys: std::collections::HashMap<RepresentationId, AggregateKey>,
}

#[derive(Default)]
pub(super) struct HostTypes {
    types: Vec<Type>,
    collected: std::collections::HashSet<SharedTypeId>,
    external_types: Vec<Type>,
    external_collected: std::collections::HashSet<SharedTypeId>,
    external_aliases: std::collections::HashSet<String>,
    opaque_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AggregateKey {
    Product(Vec<ElementKey>),
    Sum(Vec<ElementKey>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ElementKey {
    Unit,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Symbol,
    Buffer(RepresentationId),
    ByteSize,
    USize,
    External(String),
    Aggregate(RepresentationId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct RepresentationId(u64);

impl std::fmt::Display for RepresentationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

impl TypeRegistry {
    pub(super) fn c_type(&self, ty: &Type) -> TypeName {
        if is_bool(ty) {
            return c_type!(MalType_Bool);
        }
        match ty {
            Type::Unit => c_type!(MalType_Unit),
            Type::Int8 => c_type!(MalType_Int8),
            Type::Int16 => c_type!(MalType_Int16),
            Type::Int32 => c_type!(MalType_Int32),
            Type::Int64 => c_type!(MalType_Int64),
            Type::UInt8 => c_type!(MalType_UInt8),
            Type::UInt16 => c_type!(MalType_UInt16),
            Type::UInt32 => c_type!(MalType_UInt32),
            Type::UInt64 => c_type!(MalType_UInt64),
            Type::Float32 => c_type!(MalType_Float32),
            Type::Float64 => c_type!(MalType_Float64),
            Type::Symbol => c_type!(MalType_Symbol),
            Type::ByteSize => c_type!(MalType_ByteSize),
            Type::USize => c_type!(MalType_USize),
            Type::External { name, .. } => c_type!({ format!("MalType_{name}") }),
            Type::Product(_) => {
                c_type!({ format!("MalRepr_Product_{}", self.index(ty)) })
            }
            Type::Sum(_) => c_type!({ format!("MalRepr_Sum_{}", self.index(ty)) }),
            Type::Function { .. } => {
                c_type!({ format!("MalRepr_Closure_{}", self.index(ty)) })
            }
            Type::Buffer(_) => c_type!(MalType_Buffer),
            Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. }
            | Type::Abstraction { .. }
            | Type::Opaque { .. } => {
                unreachable!("these types never enter the C host registry")
            }
        }
    }

    pub(super) fn host_value_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        if let Some(alias) = alias {
            return c_type!({ format!("mal_{alias}_t") });
        }
        if is_bool(ty) {
            return c_type!(mal_Bool_t);
        }
        match ty {
            Type::Unit => c_type!(mal_Unit_t),
            Type::Int8 => c_type!(mal_Int8_t),
            Type::Int16 => c_type!(mal_Int16_t),
            Type::Int32 => c_type!(mal_Int32_t),
            Type::Int64 => c_type!(mal_Int64_t),
            Type::UInt8 => c_type!(mal_UInt8_t),
            Type::UInt16 => c_type!(mal_UInt16_t),
            Type::UInt32 => c_type!(mal_UInt32_t),
            Type::UInt64 => c_type!(mal_UInt64_t),
            Type::Float32 => c_type!(mal_Float32_t),
            Type::Float64 => c_type!(mal_Float64_t),
            Type::Symbol => c_type!(mal_Symbol_t),
            Type::ByteSize => c_type!(mal_ByteSize_t),
            Type::USize => c_type!(mal_USize_t),
            Type::External { name, .. } => c_type!({ format!("mal_{name}_t") }),
            Type::Product(_) => c_type!({ format!("mal_repr_product_{}_t", self.index(ty)) }),
            Type::Sum(_) => c_type!({ format!("mal_repr_sum_{}_t", self.index(ty)) }),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            Type::Buffer(_) => c_type!(mal_Buffer_t),
            Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. }
            | Type::Abstraction { .. }
            | Type::Opaque { .. } => {
                unreachable!("open types are not extern carriers")
            }
        }
    }

    fn element_id(&self, ty: &Type) -> RepresentationId {
        match ty {
            Type::Product(_) | Type::Sum(_) => self.index(ty),
            _ => RepresentationId(
                mal_frontend::check::type_fingerprint::TypeFingerprints::default()
                    .signature(&Type::Unit, ty)
                    .1,
            ),
        }
    }
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(members) if members.as_ref() == [Type::Unit, Type::Unit])
}
