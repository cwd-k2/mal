use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::{Atom, Operation};
use crate::core::ast::UnaryPrimitive;

use crate::c_emit::runtime::memory::{scalar_mask, scalar_name};
use crate::c_emit::scalar::integer_type;
use crate::c_emit::syntax::{Expr, Initializer};
use crate::c_emit::types::is_bool;

mod atom;
mod primitive;

use super::{
    BodyEmitter, direct_function_name, flattened_product_values, function_name,
    has_direct_product_entry,
};

impl BodyEmitter<'_> {
    pub(super) fn emit_operation_expression(
        &mut self,
        operation: &Operation,
        result: &Type,
    ) -> Expr {
        match operation {
            Operation::Atom(atom) => self.emit_atom(atom),
            Operation::Call { callee, argument } => {
                if let Some((function, environment)) = self.direct_function(callee) {
                    let argument = self.emit_atom(argument);
                    let parameter = &self.function(function).parameter.ty;
                    if has_direct_product_entry(parameter) {
                        let mut arguments = vec![
                            Expr::identifier("mal_context"),
                            Expr::identifier(environment),
                        ];
                        arguments.extend(flattened_product_values(parameter, argument));
                        return Expr::named_call(direct_function_name(function), arguments);
                    }
                    return Expr::named_call(
                        function_name(function),
                        [
                            Expr::identifier("mal_context"),
                            Expr::identifier(environment),
                            argument,
                        ],
                    );
                }
                let callee = self.emit_atom(callee);
                Expr::call(
                    callee.clone().field("call"),
                    [
                        Expr::identifier("mal_context"),
                        callee.field("environment"),
                        self.emit_atom(argument),
                    ],
                )
            }
            Operation::SymbolLength { value } => self.emit_atom(value).field("length"),
            Operation::SymbolAt { argument } => {
                self.needs.symbol_at = true;
                let argument = self.emit_atom(argument);
                Expr::named_call(
                    "mal_symbol_at",
                    [
                        Expr::identifier("mal_context"),
                        argument.clone().field("field_0"),
                        argument.field("field_1"),
                    ],
                )
            }
            Operation::Memory {
                primitive,
                argument,
            } => {
                let argument = self.emit_atom(argument);
                match primitive {
                    MemoryPrimitive::OffsetForward => {
                        self.needs.memory_offset_forward = true;
                        Expr::named_call(
                            "mal_ptr_offset",
                            [
                                Expr::identifier("mal_context"),
                                argument.clone().field("field_0"),
                                argument.field("field_1"),
                            ],
                        )
                    }
                    MemoryPrimitive::OffsetBackward => {
                        self.needs.memory_offset_backward = true;
                        Expr::named_call(
                            "mal_ptr_offset_backward",
                            [
                                Expr::identifier("mal_context"),
                                argument.clone().field("field_0"),
                                argument.field("field_1"),
                            ],
                        )
                    }
                    MemoryPrimitive::Load(scalar) => {
                        self.needs.memory_load |= scalar_mask(*scalar);
                        Expr::named_call(format!("mal_load_{}", scalar_name(*scalar)), [argument])
                    }
                    MemoryPrimitive::Store(scalar) => {
                        self.needs.memory_store |= scalar_mask(*scalar);
                        Expr::named_call(
                            format!("mal_store_{}", scalar_name(*scalar)),
                            [argument.clone().field("field_0"), argument.field("field_1")],
                        )
                    }
                    MemoryPrimitive::LoadPtr => {
                        self.needs.memory_load_ptr = true;
                        Expr::named_call("mal_load_ptr", [argument])
                    }
                    MemoryPrimitive::StorePtr => {
                        self.needs.memory_store_ptr = true;
                        Expr::named_call(
                            "mal_store_ptr",
                            [argument.clone().field("field_0"), argument.field("field_1")],
                        )
                    }
                    MemoryPrimitive::LoadSymbol => {
                        self.needs.memory_load_symbol = true;
                        Expr::named_call(
                            "mal_load_symbol",
                            [
                                Expr::identifier("mal_context"),
                                argument.clone().field("field_0"),
                                argument.field("field_1"),
                            ],
                        )
                    }
                    MemoryPrimitive::StoreSymbol => {
                        self.needs.memory_store_symbol = true;
                        Expr::named_call(
                            "mal_store_symbol",
                            [argument.clone().field("field_0"), argument.field("field_1")],
                        )
                    }
                }
            }
            Operation::ExternalCall { id, argument } => self.emit_external_call(*id, argument),
            Operation::NumericConversion { operand } => {
                let source = &operand.ty;
                let operand = self.emit_atom(operand);
                if integer_type(source).is_some() && integer_type(result).is_some() {
                    let unsigned = integer_type(result).unwrap().unsigned;
                    self.wrap_integer(result, Expr::cast(unsigned, operand))
                } else if is_float_type(source) && integer_type(result).is_some() {
                    let source_index = usize::from(*source == Type::Float64);
                    let target = integer_type(result).unwrap();
                    let target_index = target.index;
                    self.needs.float_to_integer |= 1_u32 << (source_index * 8 + target_index);
                    let target_name = target.name;
                    let source_name = if *source == Type::Float32 {
                        "f32"
                    } else {
                        "f64"
                    };
                    Expr::named_call(
                        format!("mal_{source_name}_to_{target_name}"),
                        [Expr::identifier("mal_context"), operand],
                    )
                } else if is_float_type(result) {
                    Expr::cast(self.types.c_type(result), operand)
                } else {
                    unreachable!("type checking admits only numeric conversions")
                }
            }
            Operation::Product(elements) => Expr::compound_literal(
                self.types.c_type(result),
                elements.iter().enumerate().map(|(index, element)| {
                    Initializer::designated(format!("field_{index}"), self.emit_atom(element))
                }),
            ),
            Operation::SumInjection { index, value } => {
                if is_bool(result) {
                    debug_assert_eq!(value.ty, Type::Unit);
                    Expr::named_call("UINT8_C", [Expr::literal(index.to_string())])
                } else {
                    Expr::compound_literal(
                        self.types.c_type(result),
                        [
                            Initializer::designated(
                                "tag",
                                Expr::named_call("UINT32_C", [Expr::literal(index.to_string())]),
                            ),
                            Initializer::designated(
                                format!("payload.variant_{index}"),
                                self.emit_atom(value),
                            ),
                        ],
                    )
                }
            }
            Operation::PrimitiveUnary { operator, operand } => {
                let operand_text = self.emit_atom(operand);
                if matches!(operand.ty, Type::Float32 | Type::Float64) {
                    return match operator {
                        UnaryPrimitive::Negate => Expr::unary("-", operand_text),
                        UnaryPrimitive::BitwiseNot => {
                            unreachable!("bitwise not is not defined for Float")
                        }
                    };
                }
                let unsigned = integer_type(&operand.ty).unwrap().unsigned;
                let expression = match operator {
                    UnaryPrimitive::Negate => Expr::binary(
                        "-",
                        Expr::cast(unsigned, Expr::literal("0")),
                        Expr::cast(unsigned, operand_text),
                    ),
                    UnaryPrimitive::BitwiseNot => {
                        Expr::unary("~", Expr::cast(unsigned, operand_text))
                    }
                };
                self.wrap_integer(&operand.ty, expression)
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => self.emit_binary(*operator, left, right, result),
            Operation::MakeClosure { .. }
            | Operation::Case { .. }
            | Operation::PrimitiveBranch { .. } => {
                unreachable!("structured operations are emitted as statements")
            }
        }
    }

    pub(super) fn emit_external_call(
        &self,
        id: crate::resolve::ast::ExternalOperationId,
        argument: &Atom,
    ) -> Expr {
        let external = self.external(id);
        let mut arguments = vec![Expr::identifier("mal_context")];
        match &external.parameter {
            Type::Unit => {}
            Type::Product(elements) => {
                let argument = self.emit_atom(argument);
                arguments.extend(
                    elements
                        .iter()
                        .enumerate()
                        .map(|(index, _)| argument.clone().field(format!("field_{index}"))),
                );
            }
            _ => arguments.push(self.emit_atom(argument)),
        }
        Expr::named_call(format!("mal_ext_{}", external.name), arguments)
    }
}

fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Float32 | Type::Float64)
}
