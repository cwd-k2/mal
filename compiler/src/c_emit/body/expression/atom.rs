use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Reference};

use crate::c_emit::scalar::integer_type;
use crate::c_emit::syntax::{Expr, Initializer};

use super::super::{function_name, value_name};

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_atom(&self, atom: &Atom) -> Expr {
        match &atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => Expr::identifier(value_name(*id)),
            AtomKind::Reference(Reference::EnvironmentField(index)) => {
                Expr::identifier("mal_environment_fields").pointer_field(format!("field_{index}"))
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => Expr::compound_literal(
                self.types.c_type(&atom.ty),
                [
                    Initializer::designated("call", Expr::identifier(function_name(*function))),
                    Initializer::designated("environment", Expr::identifier("mal_environment")),
                ],
            ),
            AtomKind::Integer(value) => {
                let integer = integer_type(&atom.ty).expect("integer atoms have integer types");
                if integer.minimum_value == Some(*value) {
                    Expr::identifier(integer.minimum.unwrap())
                } else {
                    Expr::named_call(integer.constant, [Expr::literal(value.to_string())])
                }
            }
            AtomKind::Float(bits) => match atom.ty {
                Type::Float32 => Expr::named_call(
                    "mal_float32_from_bits",
                    [Expr::named_call(
                        "UINT32_C",
                        [Expr::literal(bits.to_string())],
                    )],
                ),
                Type::Float64 => Expr::named_call(
                    "mal_float64_from_bits",
                    [Expr::named_call(
                        "UINT64_C",
                        [Expr::literal(bits.to_string())],
                    )],
                ),
                _ => unreachable!("float atoms have Float32 or Float64 type"),
            },
            AtomKind::Symbol(value) => {
                let bytes = value
                    .iter()
                    .map(|byte| format!("\\x{byte:02x}"))
                    .collect::<String>();
                Expr::compound_literal(
                    "MalType_Symbol",
                    [
                        Initializer::positional(Expr::cast(
                            "const uint8_t *",
                            Expr::literal(format!("\"{bytes}\"")),
                        )),
                        Initializer::positional(Expr::named_call(
                            "UINT64_C",
                            [Expr::literal(value.len().to_string())],
                        )),
                    ],
                )
            }
            AtomKind::StorageSize(ty) => match ty {
                Type::Int8 | Type::UInt8 => Expr::named_call("UINT64_C", [Expr::literal("1")]),
                Type::Int16 | Type::UInt16 => Expr::named_call("UINT64_C", [Expr::literal("2")]),
                Type::Int32 | Type::UInt32 | Type::Float32 => {
                    Expr::named_call("UINT64_C", [Expr::literal("4")])
                }
                Type::Int64 | Type::UInt64 | Type::Float64 => {
                    Expr::named_call("UINT64_C", [Expr::literal("8")])
                }
                Type::Ptr => Expr::cast("uint64_t", Expr::sizeof_type("MalType_Ptr")),
                _ => unreachable!("only memory-storable types have storage-size atoms"),
            },
            AtomKind::Unit => Expr::compound_literal(
                "MalType_Unit",
                [Initializer::positional(Expr::named_call(
                    "UINT8_C",
                    [Expr::literal("0")],
                ))],
            ),
        }
    }
}
