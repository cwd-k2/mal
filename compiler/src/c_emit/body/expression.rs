use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::{Atom, Operation};
use crate::core::ast::UnaryPrimitive;

use crate::c_emit::runtime::memory::{scalar_mask, scalar_name};
use crate::c_emit::scalar::integer_type;
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
    ) -> String {
        match operation {
            Operation::Atom(atom) => self.emit_atom(atom),
            Operation::Call { callee, argument } => {
                if let Some((function, environment)) = self.direct_function(callee) {
                    let argument = self.emit_atom(argument);
                    let parameter = &self.function(function).parameter.ty;
                    if has_direct_product_entry(parameter) {
                        let arguments = flattened_product_values(parameter, &argument)
                            .into_iter()
                            .map(|value| format!(", {value}"))
                            .collect::<String>();
                        return format!(
                            "{}(mal_context, {environment}{arguments})",
                            direct_function_name(function)
                        );
                    }
                    return format!(
                        "{}(mal_context, {environment}, {argument})",
                        function_name(function)
                    );
                }
                let callee = self.emit_atom(callee);
                format!(
                    "{callee}.call(mal_context, {callee}.environment, {})",
                    self.emit_atom(argument)
                )
            }
            Operation::SymbolLength { value } => {
                format!("({}).length", self.emit_atom(value))
            }
            Operation::SymbolAt { argument } => {
                self.needs.symbol_at = true;
                let argument = self.emit_atom(argument);
                format!("mal_symbol_at(mal_context, {argument}.field_0, {argument}.field_1)")
            }
            Operation::Memory {
                primitive,
                argument,
            } => {
                let argument = self.emit_atom(argument);
                match primitive {
                    MemoryPrimitive::OffsetForward => {
                        self.needs.memory_offset_forward = true;
                        format!(
                            "mal_ptr_offset(mal_context, {argument}.field_0, {argument}.field_1)"
                        )
                    }
                    MemoryPrimitive::OffsetBackward => {
                        self.needs.memory_offset_backward = true;
                        format!(
                            "mal_ptr_offset_backward(mal_context, {argument}.field_0, {argument}.field_1)"
                        )
                    }
                    MemoryPrimitive::Load(scalar) => {
                        self.needs.memory_load |= scalar_mask(*scalar);
                        format!("mal_load_{}({argument})", scalar_name(*scalar))
                    }
                    MemoryPrimitive::Store(scalar) => {
                        self.needs.memory_store |= scalar_mask(*scalar);
                        format!(
                            "mal_store_{}({argument}.field_0, {argument}.field_1)",
                            scalar_name(*scalar)
                        )
                    }
                    MemoryPrimitive::LoadPtr => {
                        self.needs.memory_load_ptr = true;
                        format!("mal_load_ptr({argument})")
                    }
                    MemoryPrimitive::StorePtr => {
                        self.needs.memory_store_ptr = true;
                        format!("mal_store_ptr({argument}.field_0, {argument}.field_1)")
                    }
                    MemoryPrimitive::LoadSymbol => {
                        self.needs.memory_load_symbol = true;
                        format!(
                            "mal_load_symbol(mal_context, {argument}.field_0, {argument}.field_1)"
                        )
                    }
                    MemoryPrimitive::StoreSymbol => {
                        self.needs.memory_store_symbol = true;
                        format!("mal_store_symbol({argument}.field_0, {argument}.field_1)")
                    }
                }
            }
            Operation::ExternalCall { id, argument } => self.emit_external_call(*id, argument),
            Operation::NumericConversion { operand } => {
                let source = &operand.ty;
                let operand = self.emit_atom(operand);
                if integer_type(source).is_some() && integer_type(result).is_some() {
                    let unsigned = integer_type(result).unwrap().unsigned;
                    self.wrap_integer(result, &format!("({unsigned})({operand})"))
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
                    format!("mal_{source_name}_to_{target_name}(mal_context, {operand})")
                } else if is_float_type(result) {
                    format!("({})({operand})", self.types.c_type(result))
                } else {
                    unreachable!("type checking admits only numeric conversions")
                }
            }
            Operation::Product(elements) => format!(
                "({}){{ {} }}",
                self.types.c_type(result),
                elements
                    .iter()
                    .enumerate()
                    .map(|(index, element)| {
                        format!(".field_{index} = {}", self.emit_atom(element))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Operation::SumInjection { index, value } => {
                if is_bool(result) {
                    debug_assert_eq!(value.ty, Type::Unit);
                    format!("UINT8_C({index})")
                } else {
                    format!(
                        "({}){{ .tag = UINT32_C({index}), .payload.variant_{index} = {} }}",
                        self.types.c_type(result),
                        self.emit_atom(value)
                    )
                }
            }
            Operation::PrimitiveUnary { operator, operand } => {
                let operand_text = self.emit_atom(operand);
                if matches!(operand.ty, Type::Float32 | Type::Float64) {
                    return match operator {
                        UnaryPrimitive::Negate => format!("-({operand_text})"),
                        UnaryPrimitive::BitwiseNot => {
                            unreachable!("bitwise not is not defined for Float")
                        }
                    };
                }
                let unsigned = integer_type(&operand.ty).unwrap().unsigned;
                let expression = match operator {
                    UnaryPrimitive::Negate => {
                        format!("({unsigned})0 - ({unsigned})({operand_text})")
                    }
                    UnaryPrimitive::BitwiseNot => format!("~({unsigned})({operand_text})"),
                };
                self.wrap_integer(&operand.ty, &expression)
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
    ) -> String {
        let external = self.external(id);
        let arguments = match &external.parameter {
            Type::Unit => String::new(),
            Type::Product(elements) => {
                let argument = self.emit_atom(argument);
                elements
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!(", {argument}.field_{index}"))
                    .collect()
            }
            _ => format!(", {}", self.emit_atom(argument)),
        };
        format!("mal_ext_{}(mal_context{arguments})", external.name)
    }
}

fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Float32 | Type::Float64)
}
