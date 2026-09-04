use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Operation, Reference};
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive};

use super::{BodyEmitter, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_operation_expression(
        &mut self,
        operation: &Operation,
        result: &Type,
    ) -> String {
        match operation {
            Operation::Atom(atom) => self.emit_atom(atom),
            Operation::Call { callee, argument } => {
                let callee = self.emit_atom(callee);
                format!(
                    "{callee}.call(mal_context, {callee}.environment, {})",
                    self.emit_atom(argument)
                )
            }
            Operation::ExternalCall { id, argument } => {
                let external = self.external(*id);
                if external.parameter == Type::Unit {
                    format!("mal_ext_{}(mal_context)", external.name)
                } else {
                    format!(
                        "mal_ext_{}(mal_context, {})",
                        external.name,
                        self.emit_atom(argument)
                    )
                }
            }
            Operation::SumInjection { index, value } => format!(
                "({}){{ .tag = UINT32_C({index}), .payload.variant_{index} = {} }}",
                self.types.c_type(result),
                self.emit_atom(value)
            ),
            Operation::PrimitiveUnary { operator, operand } => match operator {
                UnaryPrimitive::Int32Negate => {
                    self.needs.wrap = true;
                    format!(
                        "mal_i32_from_u32(UINT32_C(0) - (uint32_t)({}))",
                        self.emit_atom(operand)
                    )
                }
            },
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => self.emit_binary(*operator, left, right, result),
            Operation::MakeClosure { .. } | Operation::Case { .. } => {
                unreachable!("structured operations are emitted as statements")
            }
        }
    }

    fn emit_binary(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        result: &Type,
    ) -> String {
        let left = self.emit_atom(left);
        let right = self.emit_atom(right);
        match operator {
            BinaryPrimitive::Int32Multiply
            | BinaryPrimitive::Int32Add
            | BinaryPrimitive::Int32Subtract => {
                self.needs.wrap = true;
                let symbol = match operator {
                    BinaryPrimitive::Int32Multiply => "*",
                    BinaryPrimitive::Int32Add => "+",
                    BinaryPrimitive::Int32Subtract => "-",
                    _ => unreachable!(),
                };
                format!("mal_i32_from_u32((uint32_t)({left}) {symbol} (uint32_t)({right}))")
            }
            BinaryPrimitive::Int32Divide => {
                self.needs.divide = true;
                format!("mal_i32_divide(mal_context, {left}, {right})")
            }
            BinaryPrimitive::Int32Remainder => {
                self.needs.remainder = true;
                format!("mal_i32_remainder(mal_context, {left}, {right})")
            }
            BinaryPrimitive::Int32Less
            | BinaryPrimitive::Int32LessEqual
            | BinaryPrimitive::Int32Greater
            | BinaryPrimitive::Int32GreaterEqual
            | BinaryPrimitive::Int32Equal
            | BinaryPrimitive::Int32NotEqual => {
                let symbol = match operator {
                    BinaryPrimitive::Int32Less => "<",
                    BinaryPrimitive::Int32LessEqual => "<=",
                    BinaryPrimitive::Int32Greater => ">",
                    BinaryPrimitive::Int32GreaterEqual => ">=",
                    BinaryPrimitive::Int32Equal => "==",
                    BinaryPrimitive::Int32NotEqual => "!=",
                    _ => unreachable!(),
                };
                format!(
                    "({}){{ .tag = ({left} {symbol} {right}) ? UINT32_C(1) : UINT32_C(0) }}",
                    self.types.c_type(result)
                )
            }
        }
    }

    pub(super) fn emit_atom(&self, atom: &Atom) -> String {
        match atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => value_name(id),
            AtomKind::Reference(Reference::EnvironmentField(index)) => {
                format!("mal_environment_fields->field_{index}")
            }
            AtomKind::Integer(value) => {
                if value == i32::MIN {
                    "INT32_MIN".into()
                } else {
                    format!("INT32_C({value})")
                }
            }
            AtomKind::Unit => "(MalUnit){ UINT8_C(0) }".into(),
        }
    }
}
