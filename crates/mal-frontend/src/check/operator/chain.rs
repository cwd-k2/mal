//! The right operand of a left-associative operator chain, checked once the left side is known.

use super::*;

impl Checker {
    pub(in crate::check) fn check_binary_after_left(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        match operator.kind {
            BinaryOperator::SymbolAt => {
                self.require_type(&left.ty, &Type::Symbol, left.span)?;
                let (left, right) = self.check_after(left, right, Some(&Type::USize))?;
                return Ok(symbol(
                    SymbolPrimitive::ByteAt,
                    vec![left, right],
                    Type::UInt8,
                    span,
                ));
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                self.require_type(&left.ty, &bool_type(), left.span)?;
                return self.check_logical_after_left(operator, left, right, span);
            }
            BinaryOperator::Add if left.ty == Type::Symbol => {
                let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
                return Ok(binary_expression(operator, left, right, Type::Symbol, span));
            }
            BinaryOperator::Divide | BinaryOperator::Remainder if left.ty == Type::Symbol => {
                let (left, right) = self.check_after(left, right, Some(&Type::USize))?;
                return Ok(binary_expression(operator, left, right, Type::Symbol, span));
            }
            _ => {}
        }

        let numeric = is_integer(&left.ty) || is_float(&left.ty);
        let integer = is_integer(&left.ty);
        let expected = left.ty.clone();
        let right_expected =
            if operator.kind == BinaryOperator::Multiply && is_target_quantity(&left.ty) {
                None
            } else {
                Some(&expected)
            };
        let (left, right) = self.check_after(left, right, right_expected)?;
        let valid = match operator.kind {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                numeric && arithmetic_result(operator.kind, &left.ty, &right.ty).is_some()
            }
            BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                integer && arithmetic_result(operator.kind, &left.ty, &right.ty).is_some()
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                numeric || left.ty == bool_type() || left.ty == Type::Symbol
            }
            BinaryOperator::SymbolAt | BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                unreachable!("specialized operators return above")
            }
        };
        if !valid {
            let integer_operator = matches!(
                operator.kind,
                BinaryOperator::Remainder
                    | BinaryOperator::ShiftLeft
                    | BinaryOperator::ShiftRight
                    | BinaryOperator::BitwiseAnd
                    | BinaryOperator::BitwiseXor
                    | BinaryOperator::BitwiseOr
            );
            let message = if is_target_quantity(&left.ty) {
                "binary operator is not defined for this type"
            } else if integer_operator && !integer {
                "integer operator requires integer operands"
            } else if integer_operator {
                "operator is not defined for this integer type"
            } else if matches!(
                operator.kind,
                BinaryOperator::Equal | BinaryOperator::NotEqual
            ) {
                "equality is not defined for this type"
            } else {
                "numeric operator requires numeric operands"
            };
            return Err(Diagnostic::error(message)
                .with_primary(
                    left.span,
                    format!("this has type `{}`", type_name(&left.ty)),
                )
                .into());
        }
        let result = if matches!(
            operator.kind,
            BinaryOperator::Less
                | BinaryOperator::LessEqual
                | BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Equal
                | BinaryOperator::NotEqual
        ) {
            bool_type()
        } else {
            arithmetic_result(operator.kind, &left.ty, &right.ty).unwrap_or_else(|| left.ty.clone())
        };
        Ok(binary_expression(operator, left, right, result, span))
    }
}
