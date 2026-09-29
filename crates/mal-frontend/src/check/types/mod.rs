//! Canonical type terms, source expansion, kind inference, and type properties.

use super::ast::{Kind, Type};

mod canonical;
mod definitions;
mod display;
mod expand;
mod kind;
mod properties;
mod representation;
pub(in crate::check) mod term;
mod validation;

pub(super) use canonical::{
    bool_type, equivalent_in_file, function_placeholder, representation_view, runtime_type,
    substitute_type,
};
pub(super) use definitions::{GenericAliasDefinition, OpaqueDefinition};
pub(super) use display::type_name;
pub(super) use expand::require_type_argument_kinds;
pub(super) use kind::Kinds;
pub(super) use properties::{
    ensure_buffer_storable, is_memory_representable, satisfies_representable_requirement,
    satisfies_storable_requirement, storable_requirements,
};
pub(super) use representation::ensure_representable;
