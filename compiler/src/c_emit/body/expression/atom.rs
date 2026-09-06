use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Reference};

use crate::c_emit::scalar::integer_type;

use super::super::{function_name, value_name};

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_atom(&self, atom: &Atom) -> String {
        match &atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => value_name(*id),
            AtomKind::Reference(Reference::EnvironmentField(index)) => {
                format!("mal_environment_fields->field_{index}")
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => format!(
                "({}){{ .call = {}, .environment = mal_environment }}",
                self.types.c_type(&atom.ty),
                function_name(*function)
            ),
            AtomKind::Integer(value) => {
                let integer = integer_type(&atom.ty).expect("integer atoms have integer types");
                if integer.minimum_value == Some(*value) {
                    integer.minimum.unwrap().into()
                } else {
                    let constant = integer.constant;
                    format!("{constant}({value})")
                }
            }
            AtomKind::Float(bits) => match atom.ty {
                Type::Float32 => format!("mal_float32_from_bits(UINT32_C({bits}))"),
                Type::Float64 => format!("mal_float64_from_bits(UINT64_C({bits}))"),
                _ => unreachable!("float atoms have Float32 or Float64 type"),
            },
            AtomKind::Symbol(value) => {
                let bytes = value
                    .iter()
                    .map(|byte| format!("\\x{byte:02x}"))
                    .collect::<String>();
                format!(
                    "(MalType_Symbol){{ (const uint8_t *)\"{bytes}\", UINT64_C({}) }}",
                    value.len()
                )
            }
            AtomKind::StorageSize(ty) => match ty {
                Type::Int8 | Type::UInt8 => "UINT64_C(1)".into(),
                Type::Int16 | Type::UInt16 => "UINT64_C(2)".into(),
                Type::Int32 | Type::UInt32 | Type::Float32 => "UINT64_C(4)".into(),
                Type::Int64 | Type::UInt64 | Type::Float64 => "UINT64_C(8)".into(),
                Type::Ptr => "((uint64_t)sizeof(MalType_Ptr))".into(),
                _ => unreachable!("only memory-storable types have storage-size atoms"),
            },
            AtomKind::Unit => "(MalType_Unit){ UINT8_C(0) }".into(),
        }
    }
}
