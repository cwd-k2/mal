use crate::check::ast::Type;
use crate::closure::ast::Atom;
use crate::core::ast::BinaryPrimitive;

use crate::c_emit::scalar::integer_type;
use crate::c_emit::types::is_bool;

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(super) fn emit_binary(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        result: &Type,
    ) -> String {
        let operand_type = left.ty.clone();
        let left = self.emit_atom(left);
        let right = self.emit_atom(right);
        match operator {
            BinaryPrimitive::Multiply | BinaryPrimitive::Add | BinaryPrimitive::Subtract => {
                if operator == BinaryPrimitive::Add && operand_type == Type::Symbol {
                    self.needs.symbol_concatenate = true;
                    return format!("mal_symbol_concatenate(mal_context, {left}, {right})");
                }
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    let symbol = match operator {
                        BinaryPrimitive::Multiply => "*",
                        BinaryPrimitive::Add => "+",
                        BinaryPrimitive::Subtract => "-",
                        _ => unreachable!(),
                    };
                    return format!("({left} {symbol} {right})");
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
                let expression = format!(
                    "({carrier})({unsigned})({left}) {symbol} ({carrier})({unsigned})({right})"
                );
                self.wrap_integer(&operand_type, &expression)
            }
            BinaryPrimitive::Divide => {
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    return format!("({left} / {right})");
                }
                let integer = integer_type(&operand_type).unwrap();
                self.needs.divide |= integer.mask();
                let name = integer.name;
                format!("mal_{name}_divide(mal_context, {left}, {right})")
            }
            BinaryPrimitive::Remainder => {
                let integer = integer_type(&operand_type).unwrap();
                self.needs.remainder |= integer.mask();
                let name = integer.name;
                format!("mal_{name}_remainder(mal_context, {left}, {right})")
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
                format!("mal_{name}_{direction}(mal_context, {left}, {right})")
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
                let expression = format!("({unsigned})({left}) {symbol} ({unsigned})({right})");
                self.wrap_integer(&operand_type, &expression)
            }
            BinaryPrimitive::Less
            | BinaryPrimitive::LessEqual
            | BinaryPrimitive::Greater
            | BinaryPrimitive::GreaterEqual
            | BinaryPrimitive::Equal
            | BinaryPrimitive::NotEqual => {
                let condition = self.comparison_text(operator, &operand_type, &left, &right);
                debug_assert!(is_bool(result));
                format!("({condition}) ? UINT8_C(1) : UINT8_C(0)")
            }
        }
    }

    pub(in crate::c_emit::body) fn emit_primitive_condition(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
    ) -> String {
        self.comparison_text(
            operator,
            &left.ty,
            &self.emit_atom(left),
            &self.emit_atom(right),
        )
    }

    fn comparison_text(
        &mut self,
        operator: BinaryPrimitive,
        operand_type: &Type,
        left: &str,
        right: &str,
    ) -> String {
        if *operand_type == Type::Symbol {
            self.needs.symbol_equality = true;
            let equality = format!("mal_symbol_equal({left}, {right})");
            return if operator == BinaryPrimitive::Equal {
                equality
            } else {
                format!("!{equality}")
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
        format!("{left} {symbol} {right}")
    }

    pub(super) fn wrap_integer(&mut self, ty: &Type, expression: &str) -> String {
        let integer = integer_type(ty).unwrap();
        let name = integer.name;
        let unsigned = integer.unsigned;
        if integer.signed() {
            self.needs.wrap |= integer.mask();
            format!("mal_{name}_from_{unsigned}(({unsigned})({expression}))")
        } else {
            format!("({unsigned})({expression})")
        }
    }
}
