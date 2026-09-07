use crate::check::ast::Type;
use crate::closure::ast::Atom;
use crate::core::ast::BinaryPrimitive;

use crate::c_emit::scalar::integer_type;
use crate::c_emit::syntax::{BinaryOperator, Expr};
use crate::c_emit::types::is_bool;

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_symbol_concatenate(
        &mut self,
        left: &Atom,
        right: &Atom,
        consume_left: bool,
    ) -> Expr {
        let name = if consume_left {
            self.needs.symbol_concatenate_consuming_left = true;
            "mal_symbol_concatenate_consuming_left"
        } else {
            self.needs.symbol_concatenate = true;
            "mal_symbol_concatenate"
        };
        Expr::named_call(
            name,
            [
                Expr::identifier("mal_context"),
                self.emit_atom(left),
                self.emit_atom(right),
            ],
        )
    }

    pub(super) fn emit_binary(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        result: &Type,
    ) -> Expr {
        let operand_type = left.ty.clone();
        let left = self.emit_atom(left);
        let right = self.emit_atom(right);
        match operator {
            BinaryPrimitive::Multiply | BinaryPrimitive::Add | BinaryPrimitive::Subtract => {
                if operator == BinaryPrimitive::Add && operand_type == Type::Symbol {
                    self.needs.symbol_concatenate = true;
                    return Expr::named_call(
                        "mal_symbol_concatenate",
                        [Expr::identifier("mal_context"), left, right],
                    );
                }
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    let operator = match operator {
                        BinaryPrimitive::Multiply => BinaryOperator::Multiply,
                        BinaryPrimitive::Add => BinaryOperator::Add,
                        BinaryPrimitive::Subtract => BinaryOperator::Subtract,
                        _ => unreachable!(),
                    };
                    return Expr::binary(operator, left, right);
                }
                let integer = integer_type(&operand_type).unwrap();
                let unsigned = integer.unsigned;
                let carrier = integer.carrier;
                let operator = match operator {
                    BinaryPrimitive::Multiply => BinaryOperator::Multiply,
                    BinaryPrimitive::Add => BinaryOperator::Add,
                    BinaryPrimitive::Subtract => BinaryOperator::Subtract,
                    _ => unreachable!(),
                };
                let expression = Expr::binary(
                    operator,
                    Expr::cast(carrier, Expr::cast(unsigned, left)),
                    Expr::cast(carrier, Expr::cast(unsigned, right)),
                );
                self.wrap_integer(&operand_type, expression)
            }
            BinaryPrimitive::Divide => {
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    return Expr::divide(left, right);
                }
                let integer = integer_type(&operand_type).unwrap();
                self.needs.divide |= integer.mask();
                let name = integer.name;
                Expr::named_call(
                    format!("mal_{name}_divide"),
                    [Expr::identifier("mal_context"), left, right],
                )
            }
            BinaryPrimitive::Remainder => {
                let integer = integer_type(&operand_type).unwrap();
                self.needs.remainder |= integer.mask();
                let name = integer.name;
                Expr::named_call(
                    format!("mal_{name}_remainder"),
                    [Expr::identifier("mal_context"), left, right],
                )
            }
            BinaryPrimitive::ShiftLeft | BinaryPrimitive::ShiftRight => {
                if operator == BinaryPrimitive::ShiftLeft {
                    self.needs.shift_left |= integer_type(&operand_type).unwrap().mask();
                } else {
                    self.needs.shift_right |= integer_type(&operand_type).unwrap().mask();
                }
                let integer = integer_type(&operand_type).unwrap();
                self.needs.wrap |= integer.mask();
                let name = integer.name;
                let direction = if operator == BinaryPrimitive::ShiftLeft {
                    "shift_left"
                } else {
                    "shift_right"
                };
                Expr::named_call(
                    format!("mal_{name}_{direction}"),
                    [Expr::identifier("mal_context"), left, right],
                )
            }
            BinaryPrimitive::BitwiseAnd
            | BinaryPrimitive::BitwiseXor
            | BinaryPrimitive::BitwiseOr => {
                let unsigned = integer_type(&operand_type).unwrap().unsigned;
                let operator = match operator {
                    BinaryPrimitive::BitwiseAnd => BinaryOperator::BitwiseAnd,
                    BinaryPrimitive::BitwiseXor => BinaryOperator::BitwiseXor,
                    BinaryPrimitive::BitwiseOr => BinaryOperator::BitwiseOr,
                    _ => unreachable!(),
                };
                let expression = Expr::binary(
                    operator,
                    Expr::cast(unsigned, left),
                    Expr::cast(unsigned, right),
                );
                self.wrap_integer(&operand_type, expression)
            }
            BinaryPrimitive::Less
            | BinaryPrimitive::LessEqual
            | BinaryPrimitive::Greater
            | BinaryPrimitive::GreaterEqual
            | BinaryPrimitive::Equal
            | BinaryPrimitive::NotEqual => {
                let condition = self.comparison_text(operator, &operand_type, left, right);
                debug_assert!(is_bool(result));
                Expr::conditional(
                    condition,
                    Expr::named_call("UINT8_C", [Expr::number("1")]),
                    Expr::named_call("UINT8_C", [Expr::number("0")]),
                )
            }
        }
    }

    pub(in crate::c_emit::body) fn emit_primitive_condition(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
    ) -> Expr {
        self.comparison_text(
            operator,
            &left.ty,
            self.emit_atom(left),
            self.emit_atom(right),
        )
    }

    fn comparison_text(
        &mut self,
        operator: BinaryPrimitive,
        operand_type: &Type,
        left: Expr,
        right: Expr,
    ) -> Expr {
        if *operand_type == Type::Symbol {
            self.needs.symbol_equality = true;
            let equality = Expr::named_call("mal_symbol_equal", [left, right]);
            return if operator == BinaryPrimitive::Equal {
                equality
            } else {
                Expr::logical_not(equality)
            };
        }
        let operator = match operator {
            BinaryPrimitive::Less => BinaryOperator::Less,
            BinaryPrimitive::LessEqual => BinaryOperator::LessEqual,
            BinaryPrimitive::Greater => BinaryOperator::Greater,
            BinaryPrimitive::GreaterEqual => BinaryOperator::GreaterEqual,
            BinaryPrimitive::Equal => BinaryOperator::Equal,
            BinaryPrimitive::NotEqual => BinaryOperator::NotEqual,
            _ => unreachable!("primitive branches contain only comparison operators"),
        };
        Expr::binary(operator, left, right)
    }

    pub(super) fn wrap_integer(&mut self, ty: &Type, expression: Expr) -> Expr {
        let integer = integer_type(ty).unwrap();
        let name = integer.name;
        let unsigned = integer.unsigned;
        if integer.signed() {
            self.needs.wrap |= integer.mask();
            Expr::named_call(
                format!("mal_{name}_from_{unsigned}"),
                [Expr::cast(unsigned, expression)],
            )
        } else {
            Expr::cast(unsigned, expression)
        }
    }
}
