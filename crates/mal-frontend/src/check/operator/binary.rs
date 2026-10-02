//! Binary operators: the type expected of the left operand, and both operands checked together.

use super::*;

impl Checker {
    pub(in crate::check) fn binary_left_expected(
        &self,
        operator: &Node<BinaryOperator>,
        expected: Option<&Type>,
    ) -> Option<Type> {
        match operator.kind {
            BinaryOperator::SymbolAt => Some(Type::Symbol),
            BinaryOperator::Add | BinaryOperator::Subtract => expected
                .filter(|ty| {
                    (operator.kind == BinaryOperator::Add && **ty == Type::Symbol)
                        || is_integer(ty)
                        || is_float(ty)
                })
                .cloned(),
            BinaryOperator::Multiply => expected
                .filter(|ty| (is_integer(ty) || is_float(ty)) && **ty != Type::ByteSize)
                .cloned(),
            BinaryOperator::Divide => expected
                .filter(|ty| **ty == Type::Symbol || is_integer(ty) || is_float(ty))
                .cloned(),
            BinaryOperator::Remainder => expected
                .filter(|ty| **ty == Type::Symbol || is_integer(ty))
                .cloned(),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => expected.filter(|ty| is_integer(ty)).cloned(),
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => Some(bool_type()),
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual => None,
        }
    }

    pub(in crate::check) fn check_binary(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == BinaryOperator::SymbolAt {
            let left = view_operand(self.check_before(left, None, right.span)?);
            return self.check_binary_after_left(operator, left, right, span);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Add | BinaryOperator::Subtract
        ) {
            return self.check_additive(operator, left, right, span, expected);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Divide | BinaryOperator::Remainder
        ) && !is_contextual_integer(left)
            && !is_contextual_float(left)
        {
            let left_expected = self.binary_left_expected(operator, expected);
            let left = view_operand(self.check_before(left, left_expected.as_ref(), right.span)?);
            return self.check_binary_after_left(operator, left, right, span);
        }
        let expected_integer = expected.filter(|expected| is_integer(expected));
        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right, result) = match operator.kind {
            BinaryOperator::Multiply => {
                let (left, right) =
                    self.check_multiplication_operands(left, right, expected_numeric)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Divide => {
                let (left, right) = self.check_numeric_operands(left, right, expected_numeric)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Remainder => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                let (left, right) = self.check_numeric_operands(left, right, None)?;
                if arithmetic_result(operator.kind, &left.ty, &right.ty).is_none() {
                    return Err(unsupported_binary(operator.kind, &left));
                }
                (left, right, bool_type())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
                let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
                let (left, right) = if left_contextual && !right_contextual {
                    let right = match self.check_expression(right, None) {
                        Err(CheckFailure::Abrupt(abrupt)) => {
                            let left = self.check_expression(left, None)?;
                            return Err(CheckFailure::Abrupt(Box::new(
                                (*abrupt).preceded_by(vec![left]),
                            )));
                        }
                        result => view_operand(result?),
                    };
                    let left = self.check_before(left, Some(&right.ty), right.span)?;
                    (left, right)
                } else {
                    let left = view_operand(self.check_before(left, None, right.span)?);
                    let expected = left.ty.clone();
                    self.check_after(left, right, Some(&expected))?
                };
                if !is_integer(&left.ty)
                    && !is_float(&left.ty)
                    && left.ty != bool_type()
                    && left.ty != Type::Symbol
                {
                    return Err(Diagnostic::error("equality is not defined for this type")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        )
                        .into());
                }
                (left, right, bool_type())
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                return self.check_logical(operator, left, right, span);
            }
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::SymbolAt | BinaryOperator::Add | BinaryOperator::Subtract => {
                unreachable!("specialized operators are checked separately")
            }
        };
        Ok(binary_expression(operator, left, right, result, span))
    }
}
