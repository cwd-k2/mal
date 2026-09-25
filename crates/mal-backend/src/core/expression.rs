use super::*;

impl Lowerer {
    pub(super) fn lower_expression(&mut self, expression: &checked::Expression) -> Expression {
        let kind = match &expression.kind {
            checked::ExpressionKind::Reference(reference) => match reference.id {
                FALSE_VALUE => return self.bool_value(false, expression.span),
                TRUE_VALUE => return self.bool_value(true, expression.span),
                id => ExpressionKind::Reference(ValueId::Source(id)),
            },
            checked::ExpressionKind::GenericReference { .. } => {
                unreachable!("specialization removes generic references before core lowering")
            }
            checked::ExpressionKind::Integer(value) => ExpressionKind::Integer(*value),
            checked::ExpressionKind::Float(bits) => ExpressionKind::Float(*bits),
            checked::ExpressionKind::Symbol(value) => ExpressionKind::Symbol(value.clone()),
            checked::ExpressionKind::Unit => ExpressionKind::Unit,
            checked::ExpressionKind::Product(elements) => ExpressionKind::Product(
                elements
                    .iter()
                    .map(|element| self.lower_expression(element))
                    .collect(),
            ),
            checked::ExpressionKind::Parenthesized(inner) => return self.lower_expression(inner),
            checked::ExpressionKind::Block(block) => {
                return self.lower_body(&block.items, &block.result);
            }
            checked::ExpressionKind::ResultBlock { .. } => {
                unreachable!("result blocks are lowered through their local continuation")
            }
            checked::ExpressionKind::Lambda(lambda) => {
                ExpressionKind::Lambda(self.lower_lambda(lambda))
            }
            checked::ExpressionKind::Call { callee, argument } => ExpressionKind::Call {
                callee: Box::new(self.lower_expression(callee)),
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::SymbolLength { value } => ExpressionKind::SymbolLength {
                value: Box::new(self.lower_expression(value)),
            },
            checked::ExpressionKind::SymbolAt { argument } => ExpressionKind::SymbolAt {
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::Memory {
                primitive,
                operands,
            } => {
                let operands = operands
                    .iter()
                    .map(|operand| self.lower_expression(operand))
                    .collect();
                buffer::memory_kind(*primitive, operands, &expression.ty)
            }
            checked::ExpressionKind::NumericConversion { value } => {
                ExpressionKind::NumericConversion {
                    value: Box::new(self.lower_expression(value)),
                }
            }
            checked::ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                let checked::Type::Sum(members) = &scrutinee.ty else {
                    unreachable!("sum elimination has a sum scrutinee");
                };
                let arms = self.lower_sum_arms(members, continuations, &expression.ty);
                ExpressionKind::Case {
                    scrutinee: Box::new(self.lower_expression(scrutinee)),
                    arms,
                }
            }
            checked::ExpressionKind::SumInjection { index, value } => {
                ExpressionKind::SumInjection {
                    index: *index,
                    value: Box::new(self.lower_expression(value)),
                }
            }
            checked::ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                return self.lower_if(
                    condition,
                    then_branch,
                    else_branch,
                    &expression.ty,
                    expression.span,
                );
            }
            checked::ExpressionKind::Unary { operator, operand } => {
                if operator.kind == UnaryOperator::LogicalNot {
                    return self.lower_logical_not(operand, expression.span);
                }
                ExpressionKind::PrimitiveUnary {
                    operator: match operator.kind {
                        UnaryOperator::Negate => UnaryPrimitive::Negate,
                        UnaryOperator::BitwiseNot => UnaryPrimitive::BitwiseNot,
                        UnaryOperator::LogicalNot
                        | UnaryOperator::SymbolLength
                        | UnaryOperator::Star => {
                            unreachable!("type checking rejects non-numeric core primitives")
                        }
                    },
                    operand: Box::new(self.lower_expression(operand)),
                }
            }
            checked::ExpressionKind::Binary { .. } => return self.lower_binary_chain(expression),
        };
        Expression {
            kind,
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }

    fn lower_binary_chain(&mut self, expression: &checked::Expression) -> Expression {
        let mut outer = Vec::new();
        let mut current = expression;
        while let checked::ExpressionKind::Binary {
            operator,
            left,
            right,
        } = &current.kind
            && matches!(left.kind, checked::ExpressionKind::Binary { .. })
        {
            outer.push((
                operator.kind,
                right.as_ref(),
                current.ty.clone(),
                current.span,
            ));
            current = left;
        }

        let checked::ExpressionKind::Binary {
            operator,
            left,
            right,
        } = &current.kind
        else {
            unreachable!("caller selects a binary expression");
        };
        let left = self.lower_expression(left);
        let mut lowered = self.lower_binary_after_left(
            operator.kind,
            left,
            right,
            current.ty.clone(),
            current.span,
        );
        let mut preceding = Vec::with_capacity(outer.len());
        while let Some((operator, right, ty, span)) = outer.pop() {
            let id = self.temporary();
            let left = self.reference(id, lowered.ty.clone(), lowered.span);
            preceding.push((id, lowered));
            lowered = self.lower_binary_after_left(operator, left, right, ty, span);
        }
        while let Some((id, value)) = preceding.pop() {
            let span = value.span;
            lowered = self.temporary_let(id, value, lowered, span);
        }
        lowered
    }

    fn lower_binary_after_left(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: &checked::Expression,
        result_type: checked::Type,
        span: Span,
    ) -> Expression {
        if matches!(
            operator,
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
        ) {
            return self.lower_short_circuit_after_left(operator, left, right, span);
        }
        if operator == BinaryOperator::SymbolAt {
            unreachable!("Symbol access is lowered before generic binary operators");
        }
        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
            && left.ty == bool_type()
        {
            return self.lower_bool_equality_after_left(operator, left, right, span);
        }
        Expression {
            kind: ExpressionKind::PrimitiveBinary {
                operator: lower_binary_primitive(operator),
                left: Box::new(left),
                right: Box::new(self.lower_expression(right)),
            },
            ty: result_type,
            span,
        }
    }
}
