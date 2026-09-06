use crate::check::ast::Type;
use crate::closure::ast::Atom;
use crate::core::ast::BinaryPrimitive;

use crate::c_emit::scalar::integer_type;
use crate::c_emit::syntax::Expr;
use crate::c_emit::types::is_bool;

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
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
                    let symbol = match operator {
                        BinaryPrimitive::Multiply => "*",
                        BinaryPrimitive::Add => "+",
                        BinaryPrimitive::Subtract => "-",
                        _ => unreachable!(),
                    };
                    return Expr::binary(symbol, left, right);
                }
                let integer = integer_type(&operand_type).unwrap();
                let unsigned = integer.unsigned;
                let carrier = integer.carrier;
                let symbol = match operator {
                    BinaryPrimitive::Multiply => "*",
                    BinaryPrimitive::Add => "+",
                    BinaryPrimitive::Subtract => "-",
                    _ => unreachable!(),
                };
                let expression = Expr::binary(
                    symbol,
                    Expr::cast(carrier, Expr::cast(unsigned, left)),
                    Expr::cast(carrier, Expr::cast(unsigned, right)),
                );
                self.wrap_integer(&operand_type, expression)
            }
            BinaryPrimitive::Divide => {
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    return Expr::binary("/", left, right);
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
                let symbol = match operator {
                    BinaryPrimitive::BitwiseAnd => "&",
                    BinaryPrimitive::BitwiseXor => "^",
                    BinaryPrimitive::BitwiseOr => "|",
                    _ => unreachable!(),
                };
                let expression = Expr::binary(
                    symbol,
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
                    Expr::named_call("UINT8_C", [Expr::literal("1")]),
                    Expr::named_call("UINT8_C", [Expr::literal("0")]),
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
                Expr::unary("!", equality)
            };
        }
        let symbol = match operator {
            BinaryPrimitive::Less => "<",
            BinaryPrimitive::LessEqual => "<=",
            BinaryPrimitive::Greater => ">",
            BinaryPrimitive::GreaterEqual => ">=",
            BinaryPrimitive::Equal => "==",
            BinaryPrimitive::NotEqual => "!=",
            _ => unreachable!("primitive branches contain only comparison operators"),
        };
        Expr::binary(symbol, left, right)
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
