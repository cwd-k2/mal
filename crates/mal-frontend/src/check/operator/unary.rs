//! Unary operator admission and result typing.

use super::*;

impl Checker {
    pub(in crate::check) fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == UnaryOperator::Length {
            let value = self.check_expression(operand, None)?;
            let viewed = super::super::types::representation_view(&value.ty, value.span.file());
            if matches!(viewed, Type::Buffer(_)) {
                return Ok(Expression {
                    kind: ExpressionKind::Memory {
                        primitive: MemoryPrimitive::ViewLength,
                        operands: vec![value],
                    },
                    ty: Type::USize,
                    span,
                });
            }
            self.require_type(&value.ty, &Type::Symbol, value.span)?;
            return Ok(symbol(
                SymbolPrimitive::Length,
                vec![value],
                Type::USize,
                span,
            ));
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
            let operand = view_operand(self.check_expression(
                operand,
                expected.filter(|expected| is_integer(expected) || is_float(expected)),
            )?);
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
                    operator: Node::new(
                        UnaryOperation::Primitive(UnaryPrimitive::Negate),
                        operator.span,
                    ),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        if operator.kind == UnaryOperator::BitwiseNot {
            let operand = view_operand(
                self.check_expression(operand, expected.filter(|expected| is_integer(expected)))?,
            );
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
                    operator: Node::new(
                        UnaryOperation::Primitive(UnaryPrimitive::BitwiseNot),
                        operator.span,
                    ),
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
                operator: Node::new(UnaryOperation::LogicalNot, operator.span),
                operand: Box::new(operand),
            },
            ty: operand_type,
            span,
        })
    }
}
