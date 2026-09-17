use crate::ast::{BinaryOperator, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::ast::{Completion, Expression, ExpressionBlock, ExpressionKind, MemoryPrimitive, Type};
use super::float::{is_contextual_float, is_float};
use super::integer::{
    integer_is_signed, integer_negative_magnitude, is_contextual_integer, is_integer, literal_type,
    parse_magnitude, unparenthesized_integer,
};
use super::types::{bool_type, type_name};
use super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn binary_left_expected(
        &self,
        operator: &Node<BinaryOperator>,
        expected: Option<&Type>,
    ) -> Option<Type> {
        match operator.kind {
            BinaryOperator::SymbolAt => Some(Type::Symbol),
            BinaryOperator::Store => None,
            BinaryOperator::Add | BinaryOperator::Subtract => expected
                .filter(|ty| {
                    **ty == Type::Ptr
                        || **ty == Type::Address
                        || (operator.kind == BinaryOperator::Add && **ty == Type::Symbol)
                        || is_integer(ty)
                        || is_float(ty)
                })
                .cloned(),
            BinaryOperator::Multiply | BinaryOperator::Divide => expected
                .filter(|ty| {
                    (is_integer(ty) || is_float(ty))
                        && (operator.kind != BinaryOperator::Multiply || **ty != Type::ByteSize)
                })
                .cloned(),
            BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
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

    pub(super) fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == UnaryOperator::SymbolLength {
            let value = self.check_expression(operand, None)?;
            if matches!(value.ty, Type::Region(_) | Type::Packed(_)) {
                return Ok(Expression {
                    kind: ExpressionKind::Memory {
                        primitive: MemoryPrimitive::ViewLength,
                        argument: Box::new(value),
                    },
                    ty: Type::USize,
                    span,
                });
            }
            self.require_type(&value.ty, &Type::Symbol, value.span)?;
            return Ok(Expression {
                kind: ExpressionKind::SymbolLength {
                    value: Box::new(value),
                },
                ty: Type::UInt64,
                span,
            });
        }
        if operator.kind == UnaryOperator::Negate
            && let Some(literal) = unparenthesized_integer(operand)
        {
            let ty = literal_type(literal.suffix).unwrap_or_else(|| {
                expected
                    .filter(|ty| is_integer(ty))
                    .cloned()
                    .unwrap_or(Type::Int64)
            });
            let magnitude = parse_magnitude(literal, operand.span)?;
            if integer_is_signed(&ty) && magnitude == integer_negative_magnitude(&ty) {
                return Ok(Expression {
                    kind: ExpressionKind::Integer(
                        -i128::try_from(magnitude).expect("signed literal magnitudes fit in i128"),
                    ),
                    ty,
                    span,
                });
            }
        }
        if operator.kind == UnaryOperator::Negate {
            let operand = self.check_expression(
                operand,
                expected.filter(|expected| is_integer(expected) || is_float(expected)),
            )?;
            if (!is_integer(&operand.ty) && !is_float(&operand.ty))
                || is_target_quantity(&operand.ty)
            {
                return Err(Diagnostic::error("negation is not defined for this type")
                    .with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    )
                    .into());
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        if operator.kind == UnaryOperator::BitwiseNot {
            let operand =
                self.check_expression(operand, expected.filter(|expected| is_integer(expected)))?;
            if !is_integer(&operand.ty) {
                return Err(
                    Diagnostic::error("integer unary operator requires an integer")
                        .with_primary(
                            operand.span,
                            format!("this has type `{}`", type_name(&operand.ty)),
                        )
                        .into(),
                );
            }
            if is_target_quantity(&operand.ty) {
                return Err(
                    Diagnostic::error("bitwise not is not defined for this type")
                        .with_primary(
                            operand.span,
                            format!("this has type `{}`", type_name(&operand.ty)),
                        )
                        .into(),
                );
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        let operand_type = bool_type();
        let operand = self.check_expression(operand, Some(&operand_type))?;
        Ok(Expression {
            kind: ExpressionKind::Unary {
                operator: operator.clone(),
                operand: Box::new(operand),
            },
            ty: operand_type,
            span,
        })
    }

    pub(super) fn check_binary(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == BinaryOperator::Store {
            let left = self.check_before(left, None, right.span)?;
            return self.check_memory_store(left, right, span);
        }
        if operator.kind == BinaryOperator::SymbolAt {
            let left = self.check_before(left, None, right.span)?;
            return self.check_binary_after_left(operator, left, right, span);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Divide | BinaryOperator::Remainder
        ) && expected.is_none()
        {
            let left = self.check_before(left, None, right.span)?;
            return self.check_binary_after_left(operator, left, right, span);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Add | BinaryOperator::Subtract
        ) {
            return self.check_pointer_or_numeric_arithmetic(operator, left, right, span, expected);
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
                        result => result?,
                    };
                    let left = self.check_before(left, Some(&right.ty), right.span)?;
                    (left, right)
                } else {
                    let left = self.check_before(left, None, right.span)?;
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
            BinaryOperator::SymbolAt
            | BinaryOperator::Store
            | BinaryOperator::Add
            | BinaryOperator::Subtract => {
                unreachable!("specialized operators are checked separately")
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            span,
        })
    }

    pub(super) fn check_binary_after_left(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        match operator.kind {
            BinaryOperator::Store => return self.check_memory_store(left, right, span),
            BinaryOperator::SymbolAt => {
                if matches!(left.ty, Type::Packed(_)) {
                    return self.check_packed_index(left, right, span);
                }
                self.require_type(&left.ty, &Type::Symbol, left.span)?;
                let (left, right) = self.check_after(left, right, Some(&Type::UInt64))?;
                return Ok(Expression {
                    kind: ExpressionKind::SymbolAt {
                        argument: Box::new(Expression {
                            kind: ExpressionKind::Product(vec![left, right]),
                            ty: Type::Product(vec![Type::Symbol, Type::UInt64].into()),
                            span,
                        }),
                    },
                    ty: Type::UInt8,
                    span,
                });
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                self.require_type(&left.ty, &bool_type(), left.span)?;
                return self.check_logical_after_left(operator, left, right, span);
            }
            BinaryOperator::Add | BinaryOperator::Subtract if left.ty == Type::Ptr => {
                let primitive = if operator.kind == BinaryOperator::Add {
                    MemoryPrimitive::OffsetForward
                } else {
                    MemoryPrimitive::OffsetBackward
                };
                return self.check_pointer_offset(primitive, left, right, span);
            }
            BinaryOperator::Add | BinaryOperator::Subtract if left.ty == Type::Address => {
                return self.check_address_offset(operator, left, right, span);
            }
            BinaryOperator::Add if left.ty == Type::Symbol => {
                let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
                return Ok(Expression {
                    kind: ExpressionKind::Binary {
                        operator: operator.clone(),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty: Type::Symbol,
                    span,
                });
            }
            BinaryOperator::Divide | BinaryOperator::Remainder
                if matches!(left.ty, Type::Region(_) | Type::Packed(_)) =>
            {
                return self.check_view_slice(operator.kind, left, right, span);
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
            BinaryOperator::SymbolAt
            | BinaryOperator::Store
            | BinaryOperator::LogicalAnd
            | BinaryOperator::LogicalOr => {
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
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            span,
        })
    }

    fn check_pointer_or_numeric_arithmetic(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let pointer_primitive = match operator.kind {
            BinaryOperator::Add => MemoryPrimitive::OffsetForward,
            BinaryOperator::Subtract => MemoryPrimitive::OffsetBackward,
            _ => unreachable!("caller selects addition or subtraction"),
        };
        if expected == Some(&Type::Ptr) {
            let left = self.check_before(left, Some(&Type::Ptr), right.span)?;
            return self.check_pointer_offset(pointer_primitive, left, right, span);
        }
        if expected == Some(&Type::Address) {
            let left = self.check_before(left, Some(&Type::Address), right.span)?;
            return self.check_address_offset(operator, left, right, span);
        }
        if operator.kind == BinaryOperator::Add && expected == Some(&Type::Symbol) {
            return self.check_symbol_concatenation(operator, left, right, span);
        }

        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right) = if expected_numeric.is_some()
            || is_contextual_integer(left)
            || is_contextual_float(left)
        {
            self.check_numeric_operands(left, right, expected_numeric)?
        } else {
            let left = self.check_before(left, None, right.span)?;
            if left.ty == Type::Ptr {
                return self.check_pointer_offset(pointer_primitive, left, right, span);
            }
            if left.ty == Type::Address {
                return self.check_address_offset(operator, left, right, span);
            }
            if operator.kind == BinaryOperator::Add && left.ty == Type::Symbol {
                let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
                return Ok(Expression {
                    kind: ExpressionKind::Binary {
                        operator: operator.clone(),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty: Type::Symbol,
                    span,
                });
            }
            let expected = left.ty.clone();
            let (left, right) = self.check_after(left, right, Some(&expected))?;
            if !is_integer(&left.ty) && !is_float(&left.ty) {
                return Err(
                    Diagnostic::error("numeric operator requires numeric operands")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        )
                        .into(),
                );
            }
            (left, right)
        };
        let result = left.ty.clone();
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            span,
        })
    }

    fn check_symbol_concatenation(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let left = self.check_before(left, Some(&Type::Symbol), right.span)?;
        let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: Type::Symbol,
            span,
        })
    }

    fn check_pointer_offset(
        &mut self,
        primitive: MemoryPrimitive,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let (left, right) = self.check_after(left, right, Some(&Type::UInt64))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![left, right]),
                    ty: Type::Product(vec![Type::Ptr, Type::UInt64].into()),
                    span,
                }),
            },
            ty: Type::Ptr,
            span,
        })
    }

    fn check_address_offset(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let (left, right) = self.check_after(left, right, Some(&Type::ByteSize))?;
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: Type::Address,
            span,
        })
    }

    fn check_memory_store(
        &mut self,
        cursor: Expression,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        if let Type::Region(element) = &cursor.ty {
            let element = element.clone();
            let packed_type = Type::Packed(element.clone());
            let (region, packed) = self.check_after(cursor, value, Some(&packed_type))?;
            return Ok(Expression {
                kind: ExpressionKind::Memory {
                    primitive: MemoryPrimitive::StorePacked,
                    argument: Box::new(Expression {
                        kind: ExpressionKind::Product(vec![region, packed]),
                        ty: Type::Product(vec![Type::Region(element.clone()), packed_type].into()),
                        span,
                    }),
                },
                ty: Type::Region(element),
                span,
            });
        }
        let Type::Cursor(element) = &cursor.ty else {
            return Err(
                Diagnostic::error("memory store requires a Cursor on the left")
                    .with_primary(
                        cursor.span,
                        format!("this has type `{}`", type_name(&cursor.ty)),
                    )
                    .into(),
            );
        };
        let element = element.clone();
        let (cursor, value) = self.check_after(cursor, value, Some(element.as_ref()))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::StoreValue,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![cursor, value]),
                    ty: Type::Product(
                        vec![Type::Cursor(element.clone()), (*element).clone()].into(),
                    ),
                    span,
                }),
            },
            ty: Type::Cursor(element),
            span,
        })
    }

    fn check_view_slice(
        &mut self,
        operator: BinaryOperator,
        value: Expression,
        count: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = value.ty.clone();
        let (value, count) = self.check_after(value, count, Some(&Type::USize))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: if operator == BinaryOperator::Divide {
                    MemoryPrimitive::Prefix
                } else {
                    MemoryPrimitive::RemainderView
                },
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![value, count]),
                    ty: Type::Product(vec![ty.clone(), Type::USize].into()),
                    span,
                }),
            },
            ty,
            span,
        })
    }

    fn check_packed_index(
        &mut self,
        packed: Expression,
        index: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let Type::Packed(element) = &packed.ty else {
            unreachable!("caller checks Packed")
        };
        let element = element.clone();
        let (packed, index) = self.check_after(packed, index, Some(&Type::USize))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::PackedIndex,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![packed, index]),
                    ty: Type::Product(vec![Type::Packed(element.clone()), Type::USize].into()),
                    span,
                }),
            },
            ty: (*element).clone(),
            span,
        })
    }

    fn check_numeric_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<(Expression, Expression)> {
        let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
        let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
        let (left, right) = if let Some(expected) = expected {
            let left = self.check_before(left, Some(expected), right.span)?;
            self.check_after(left, right, Some(expected))?
        } else if left_contextual && !right_contextual {
            let right = match self.check_expression(right, None) {
                Err(CheckFailure::Abrupt(abrupt)) => {
                    let left = self.check_expression(left, None)?;
                    return Err(CheckFailure::Abrupt(Box::new(
                        (*abrupt).preceded_by(vec![left]),
                    )));
                }
                result => result?,
            };
            let left = self.check_before(left, Some(&right.ty), right.span)?;
            (left, right)
        } else {
            let left = self.check_before(left, None, right.span)?;
            let expected = left.ty.clone();
            self.check_after(left, right, Some(&expected))?
        };
        if !is_integer(&left.ty) && !is_float(&left.ty) {
            return Err(
                Diagnostic::error("numeric operator requires numeric operands")
                    .with_primary(
                        left.span,
                        format!("this has type `{}`", type_name(&left.ty)),
                    )
                    .into(),
            );
        }
        Ok((left, right))
    }

    fn check_multiplication_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<(Expression, Expression)> {
        if let Some(expected) = expected.filter(|ty| **ty != Type::ByteSize) {
            return self.check_numeric_operands(left, right, Some(expected));
        }

        let left = self.check_before(left, None, right.span)?;
        let (left, right) = self.check_after(left, right, None)?;
        if (!is_integer(&left.ty) && !is_float(&left.ty))
            || (!is_integer(&right.ty) && !is_float(&right.ty))
        {
            return Err(unsupported_binary(BinaryOperator::Multiply, &left));
        }
        Ok((left, right))
    }

    fn check_logical(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = bool_type();
        let left = self.check_before(left, Some(&ty), right.span)?;
        self.check_logical_after_left(operator, left, right, span)
    }

    fn check_logical_after_left(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = bool_type();
        match self.check_expression(right, Some(&ty)) {
            Ok(right) => Ok(Expression {
                kind: ExpressionKind::Binary {
                    operator: operator.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                ty,
                span,
            }),
            Err(CheckFailure::Abrupt(abrupt)) => {
                let constant = |index| Expression {
                    kind: ExpressionKind::SumInjection {
                        index,
                        value: Box::new(Expression {
                            kind: ExpressionKind::Unit,
                            ty: Type::Unit,
                            span,
                        }),
                    },
                    ty: ty.clone(),
                    span,
                };
                let abrupt = Completion::Abrupt(*abrupt);
                let (otherwise, then) = if operator.kind == BinaryOperator::LogicalAnd {
                    (Completion::Value(constant(0)), abrupt)
                } else {
                    (abrupt, Completion::Value(constant(1)))
                };
                Ok(Expression {
                    kind: ExpressionKind::If {
                        condition: Box::new(left),
                        then_branch: ExpressionBlock {
                            items: Vec::new(),
                            result: Box::new(then),
                            span,
                        },
                        else_branch: ExpressionBlock {
                            items: Vec::new(),
                            result: Box::new(otherwise),
                            span,
                        },
                    },
                    ty,
                    span,
                })
            }
            Err(error) => Err(error),
        }
    }
}

fn is_target_quantity(ty: &Type) -> bool {
    matches!(ty, Type::ByteSize | Type::USize)
}

fn is_fixed_width_integer(ty: &Type) -> bool {
    is_integer(ty) && !is_target_quantity(ty)
}

fn arithmetic_result(operator: BinaryOperator, left: &Type, right: &Type) -> Option<Type> {
    use BinaryOperator::*;

    if operator == Multiply
        && matches!(
            (left, right),
            (Type::ByteSize, Type::USize) | (Type::USize, Type::ByteSize)
        )
    {
        return Some(Type::ByteSize);
    }
    if left != right {
        return None;
    }
    let ordinary_numeric = is_fixed_width_integer(left) || is_float(left);
    let result = match operator {
        Add | Subtract if ordinary_numeric || is_target_quantity(left) => left.clone(),
        Multiply if ordinary_numeric || *left == Type::USize => left.clone(),
        Divide if ordinary_numeric || *left == Type::USize => left.clone(),
        Remainder if is_fixed_width_integer(left) || *left == Type::USize => left.clone(),
        Less | LessEqual | Greater | GreaterEqual
            if ordinary_numeric || is_target_quantity(left) =>
        {
            bool_type()
        }
        ShiftLeft | ShiftRight | BitwiseAnd | BitwiseXor | BitwiseOr
            if is_fixed_width_integer(left) =>
        {
            left.clone()
        }
        _ => return None,
    };
    Some(result)
}

fn unsupported_binary(operator: BinaryOperator, operand: &Expression) -> CheckFailure {
    let _ = operator;
    Diagnostic::error("binary operator is not defined for this type")
        .with_primary(
            operand.span,
            format!("this has type `{}`", type_name(&operand.ty)),
        )
        .into()
}
