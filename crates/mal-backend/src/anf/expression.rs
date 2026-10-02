//! Lowering of one core expression into ANF blocks, evaluating operands left to right.

use super::*;

impl Lowerer {
    pub(super) fn lower_expression(&mut self, expression: &core::Expression) -> Block {
        match &expression.kind {
            core::ExpressionKind::Reference(id) => {
                self.atom_block(expression, AtomKind::Reference(self.core_id(*id)))
            }
            core::ExpressionKind::Integer(value) => {
                self.atom_block(expression, AtomKind::Integer(*value))
            }
            core::ExpressionKind::Float(bits) => {
                self.atom_block(expression, AtomKind::Float(*bits))
            }
            core::ExpressionKind::Symbol(value) => {
                self.atom_block(expression, AtomKind::Symbol(value.clone()))
            }
            core::ExpressionKind::Unit => self.atom_block(expression, AtomKind::Unit),
            core::ExpressionKind::Product(elements) => {
                let mut builder = ExpressionBuilder::default();
                let elements = elements
                    .iter()
                    .map(|element| builder.append(self, element))
                    .collect();
                builder.finish(self, expression, Operation::Product(elements))
            }
            core::ExpressionKind::Let { binding, body } => {
                self.lower_let_chain(binding, body, expression)
            }
            core::ExpressionKind::Goto { target, value } => {
                let (builder, value) = self.lower_operand(value);
                builder.finish(
                    self,
                    expression,
                    Operation::Goto {
                        target: *target,
                        value,
                    },
                )
            }
            core::ExpressionKind::Lambda(lambda) => {
                let lambda = Lambda {
                    id: lambda.id,
                    self_binding: lambda.self_binding.map(|id| self.core_id(id)),
                    captures: lambda
                        .captures
                        .iter()
                        .map(|capture| Capture {
                            source: self.core_id(capture.source),
                            binding: self.core_id(capture.binding),
                            ty: capture.ty.clone(),
                        })
                        .collect(),
                    parameter: Parameter {
                        binding: lambda.parameter.binding.map(|id| self.core_id(id)),
                        ty: lambda.parameter.ty.clone(),
                        span: lambda.parameter.span,
                    },
                    body: self.lower_expression(&lambda.body),
                    joins: lambda
                        .joins
                        .iter()
                        .map(|join| self.lower_join(join))
                        .collect(),
                };
                self.operation_block(expression, Operation::Lambda(lambda))
            }
            core::ExpressionKind::Call { callee, argument } => {
                // Mal evaluates an argument before its callee. Append in semantic order even though source syntax and
                // the final operation store the callee first.
                let (mut builder, argument) = self.lower_operand(argument);
                let callee = builder.append(self, callee);
                builder.finish(self, expression, Operation::Call { callee, argument })
            }
            core::ExpressionKind::SymbolOperation {
                primitive,
                operands,
            } => {
                let mut builder = ExpressionBuilder::default();
                let operands = operands
                    .iter()
                    .map(|operand| builder.append(self, operand))
                    .collect();
                builder.finish(
                    self,
                    expression,
                    Operation::Symbol {
                        primitive: *primitive,
                        operands,
                    },
                )
            }
            core::ExpressionKind::Memory {
                primitive,
                operands,
            } => {
                let mut builder = ExpressionBuilder::default();
                let operands = operands
                    .iter()
                    .map(|operand| builder.append(self, operand))
                    .collect();
                builder.finish(
                    self,
                    expression,
                    Operation::Memory {
                        primitive: *primitive,
                        operands,
                    },
                )
            }
            core::ExpressionKind::Buffer {
                operation,
                element,
                operands,
            } => {
                let mut builder = ExpressionBuilder::default();
                let operands = operands
                    .iter()
                    .map(|operand| builder.append(self, operand))
                    .collect();
                builder.finish(
                    self,
                    expression,
                    Operation::Buffer {
                        operation: *operation,
                        element: element.clone(),
                        operands,
                    },
                )
            }
            core::ExpressionKind::ExternalCall { id, argument } => {
                let (builder, argument) = self.lower_operand(argument);
                builder.finish(
                    self,
                    expression,
                    Operation::ExternalCall { id: *id, argument },
                )
            }
            core::ExpressionKind::NumericConversion { value } => {
                let (builder, operand) = self.lower_operand(value);
                builder.finish(self, expression, Operation::NumericConversion { operand })
            }
            core::ExpressionKind::SumInjection { index, value } => {
                let (builder, value) = self.lower_operand(value);
                builder.finish(
                    self,
                    expression,
                    Operation::SumInjection {
                        index: *index,
                        value,
                    },
                )
            }
            core::ExpressionKind::Case { scrutinee, arms } => {
                let (builder, scrutinee) = self.lower_operand(scrutinee);
                let arms = arms.iter().map(|arm| self.lower_case_arm(arm)).collect();
                builder.finish(self, expression, Operation::Case { scrutinee, arms })
            }
            core::ExpressionKind::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
                let (mut builder, left) = self.lower_operand(left);
                let right = builder.append(self, right);
                let otherwise = self.lower_expression(otherwise);
                let then = self.lower_expression(then);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveBranch {
                        operator: *operator,
                        left,
                        right,
                        otherwise: Box::new(otherwise),
                        then: Box::new(then),
                    },
                )
            }
            core::ExpressionKind::PrimitiveUnary { operator, operand } => {
                let (builder, operand) = self.lower_operand(operand);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveUnary {
                        operator: *operator,
                        operand,
                    },
                )
            }
            core::ExpressionKind::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let (mut builder, left) = self.lower_operand(left);
                let right = builder.append(self, right);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveBinary {
                        operator: *operator,
                        left,
                        right,
                    },
                )
            }
        }
    }
}
