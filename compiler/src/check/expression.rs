use crate::ast::{BinaryOperator, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::lexer::IntegerLiteral;
use crate::resolve::ast as resolved;

use super::Checker;
use super::ast::{Capture, Expression, ExpressionKind, Lambda, LambdaBody, Parameter, Type};

impl Checker {
    pub(super) fn check_expression(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let checked = match &expression.kind {
            resolved::Expression::Reference(reference) => Expression {
                kind: ExpressionKind::Reference(reference.clone()),
                ty: self.value_type(reference)?,
                span: expression.span,
            },
            resolved::Expression::Integer(literal) => {
                self.check_integer(literal, expression.span, expected)?
            }
            resolved::Expression::Byte(_) => {
                return Err(self.unsupported(
                    expression.span,
                    "byte literals require UInt8, which is not implemented yet",
                ));
            }
            resolved::Expression::Unit => Expression {
                kind: ExpressionKind::Unit,
                ty: Type::Unit,
                span: expression.span,
            },
            resolved::Expression::Parenthesized(inner) => {
                let inner = self.check_expression(inner, expected)?;
                Expression {
                    ty: inner.ty.clone(),
                    kind: ExpressionKind::Parenthesized(Box::new(inner)),
                    span: expression.span,
                }
            }
            resolved::Expression::Product(_) => {
                return Err(self.unsupported(
                    expression.span,
                    "product expressions are not supported in M0",
                ));
            }
            resolved::Expression::Lambda(lambda) => {
                self.check_lambda(lambda, expression.span, expected)?
            }
            resolved::Expression::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span)?
            }
            resolved::Expression::ExternalCall {
                operation,
                arguments,
            } => self.check_external_call(operation, arguments, expression.span)?,
            resolved::Expression::SumInjection {
                type_ref,
                index,
                value,
            } => self.check_sum_injection(type_ref, index, value, expression.span)?,
            resolved::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => self.check_if(
                condition,
                then_branch,
                else_branch,
                expression.span,
                expected,
            )?,
            resolved::Expression::Case { scrutinee, arms } => {
                self.check_case(scrutinee, arms, expression.span, expected)?
            }
            resolved::Expression::Unary { operator, operand } => {
                self.check_unary(operator, operand, expression.span)?
            }
            resolved::Expression::Binary {
                operator,
                left,
                right,
            } => self.check_binary(operator, left, right, expression.span)?,
        };
        if let Some(expected) = expected {
            self.require_type(&checked.ty, expected, checked.span)?;
        }
        Ok(checked)
    }

    fn check_integer(
        &self,
        literal: &IntegerLiteral,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        if !matches!(
            literal.suffix,
            None | Some(crate::lexer::IntegerSuffix::Int32)
        ) {
            return Err(self.unsupported(
                span,
                "fixed-width integer types other than Int32 are not implemented yet",
            ));
        }
        if literal.suffix.is_none() && expected.is_none() {
            return Err(
                Diagnostic::error("integer literal requires an M0 type context").with_primary(
                    span,
                    "the language default is Int64, which is not supported in M0",
                ),
            );
        }
        if let Some(expected) = expected {
            self.require_type(&Type::Int32, expected, span)?;
        }
        let magnitude = parse_magnitude(literal, span)?;
        let value = i32::try_from(magnitude).map_err(|_| {
            Diagnostic::error("Int32 literal is out of range")
                .with_primary(span, "expected a value from 0 through 2147483647")
        })?;
        Ok(Expression {
            kind: ExpressionKind::Integer(value),
            ty: Type::Int32,
            span,
        })
    }

    fn check_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        if lambda.parameters.len() > 1 {
            return Err(self.unsupported(
                span,
                "multiple lambda parameters require product types, which are not supported in M0",
            ));
        }
        let expected_function = match expected {
            Some(Type::Function { parameter, result }) => {
                Some((parameter.as_ref().clone(), result.as_ref().clone()))
            }
            Some(other) => {
                return Err(self.type_mismatch(other, &function_placeholder(), span));
            }
            None => None,
        };

        let mut captures = Vec::with_capacity(lambda.captures.len());
        for capture in &lambda.captures {
            let ty = self.value_type(&capture.source)?;
            self.values.insert(capture.binding.id, ty.clone());
            captures.push(Capture {
                source: capture.source.clone(),
                binding: capture.binding.clone(),
                ty,
            });
        }

        let mut parameters = Vec::with_capacity(lambda.parameters.len());
        for parameter in &lambda.parameters {
            let ty = self.expand_type(&parameter.ty)?;
            self.values.insert(parameter.binding.id, ty.clone());
            parameters.push(Parameter {
                binding: parameter.binding.clone(),
                ty,
                span: parameter.span,
            });
        }
        let parameter_type = parameters
            .first()
            .map_or(Type::Unit, |parameter| parameter.ty.clone());
        if let Some((expected_parameter, _)) = &expected_function {
            self.require_type(&parameter_type, expected_parameter, span)?;
        }

        let mut items = Vec::with_capacity(lambda.body.items.len());
        for item in &lambda.body.items {
            items.push(self.check_body_item(item)?);
        }
        let expected_result = expected_function.as_ref().map(|(_, result)| result);
        let result = self.check_expression(&lambda.body.result, expected_result)?;
        let result_type = result.ty.clone();
        let ty = Type::Function {
            parameter: Box::new(parameter_type),
            result: Box::new(result_type),
        };
        Ok(Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id: lambda.id,
                captures,
                parameters,
                body: LambdaBody {
                    items,
                    result: Box::new(result),
                    span: lambda.body.span,
                },
            }),
            ty,
            span,
        })
    }

    fn check_call(
        &mut self,
        callee: &Node<resolved::Expression>,
        arguments: &[Node<resolved::Expression>],
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let callee = self.check_expression(callee, None)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return Err(
                Diagnostic::error("cannot call a non-function value").with_primary(
                    callee.span,
                    format!("this has type `{}`", type_name(&callee.ty)),
                ),
            );
        };
        let argument = self.check_argument(arguments, parameter, span)?;
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    fn check_external_call(
        &mut self,
        operation: &resolved::ExternalOperationReference,
        arguments: &[Node<resolved::Expression>],
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let signature = self.external_signature(operation.id);
        let argument = self.check_argument(arguments, &signature.parameter, span)?;
        Ok(Expression {
            kind: ExpressionKind::ExternalCall {
                id: operation.id,
                name: operation.name.clone(),
                argument: Box::new(argument),
            },
            ty: signature.result,
            span,
        })
    }

    fn check_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        parameter: &Type,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        match arguments {
            [] => {
                self.require_type(&Type::Unit, parameter, span)?;
                Ok(Expression {
                    kind: ExpressionKind::Unit,
                    ty: Type::Unit,
                    span,
                })
            }
            [argument] => self.check_expression(argument, Some(parameter)),
            _ => Err(self.unsupported(
                span,
                "multiple arguments require product types, which are not supported in M0",
            )),
        }
    }

    fn check_sum_injection(
        &mut self,
        type_ref: &resolved::TypeReference,
        index: &Node<IntegerLiteral>,
        value: &Node<resolved::Expression>,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let ty = self.expand_type_id(type_ref.id, type_ref.name.span)?;
        let Type::Sum(members) = &ty else {
            return Err(
                Diagnostic::error(format!("`{}` is not a sum type", type_ref.name.text))
                    .with_primary(
                        type_ref.name.span,
                        format!("this names `{}`", type_name(&ty)),
                    ),
            );
        };
        let index_value = parse_index(&index.kind, index.span)?;
        let member = members.get(index_value).ok_or_else(|| {
            Diagnostic::error("sum variant index is out of range").with_primary(
                index.span,
                format!("this sum has {} variants", members.len()),
            )
        })?;
        let value = self.check_expression(value, Some(member))?;
        Ok(Expression {
            kind: ExpressionKind::SumInjection {
                index: index_value,
                value: Box::new(value),
            },
            ty,
            span,
        })
    }

    fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        if operator.kind == UnaryOperator::Negate
            && let Some(literal) = unparenthesized_integer(operand)
        {
            let magnitude = parse_magnitude(literal, operand.span)?;
            if magnitude == i32::MAX as u64 + 1 {
                return Ok(Expression {
                    kind: ExpressionKind::Integer(i32::MIN),
                    ty: Type::Int32,
                    span,
                });
            }
        }
        let operand_type = match operator.kind {
            UnaryOperator::Negate => Type::Int32,
            UnaryOperator::LogicalNot => bool_type(),
            UnaryOperator::BitwiseNot => {
                return Err(
                    self.unsupported(operator.span, "bitwise operators are not supported in M0")
                );
            }
        };
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

    fn check_binary(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let (left, right, result) = match operator.kind {
            BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Remainder
            | BinaryOperator::Add
            | BinaryOperator::Subtract => (
                self.check_expression(left, Some(&Type::Int32))?,
                self.check_expression(right, Some(&Type::Int32))?,
                Type::Int32,
            ),
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => (
                self.check_expression(left, Some(&Type::Int32))?,
                self.check_expression(right, Some(&Type::Int32))?,
                bool_type(),
            ),
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let (left, right) = if is_contextual_integer(left) && !is_contextual_integer(right)
                {
                    let right = self.check_expression(right, None)?;
                    let left = self.check_expression(left, Some(&right.ty))?;
                    (left, right)
                } else {
                    let left = self.check_expression(left, None)?;
                    let right = self.check_expression(right, Some(&left.ty))?;
                    (left, right)
                };
                if left.ty != Type::Int32 && left.ty != bool_type() {
                    return Err(Diagnostic::error("equality is not defined for this type")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        ));
                }
                (left, right, bool_type())
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => (
                self.check_expression(left, Some(&bool_type()))?,
                self.check_expression(right, Some(&bool_type()))?,
                bool_type(),
            ),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                return Err(self.unsupported(
                    operator.span,
                    "bit and shift operators are not supported in M0",
                ));
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

    fn require_type(
        &self,
        actual: &Type,
        expected: &Type,
        span: crate::source::Span,
    ) -> Result<(), Diagnostic> {
        if actual == expected {
            Ok(())
        } else {
            Err(self.type_mismatch(expected, actual, span))
        }
    }

    fn type_mismatch(
        &self,
        expected: &Type,
        actual: &Type,
        span: crate::source::Span,
    ) -> Diagnostic {
        Diagnostic::error("type mismatch").with_primary(
            span,
            format!(
                "expected `{}`, found `{}`",
                type_name(expected),
                type_name(actual)
            ),
        )
    }
}

fn parse_magnitude(literal: &IntegerLiteral, span: crate::source::Span) -> Result<u64, Diagnostic> {
    u64::from_str_radix(&literal.digits, literal.radix.value()).map_err(|_| {
        Diagnostic::error("integer literal is too large").with_primary(
            span,
            "the value does not fit in the M0 integer representation",
        )
    })
}

pub(super) fn parse_index(
    literal: &IntegerLiteral,
    span: crate::source::Span,
) -> Result<usize, Diagnostic> {
    let value = parse_magnitude(literal, span)?;
    usize::try_from(value).map_err(|_| {
        Diagnostic::error("variant index is too large").with_primary(
            span,
            "the index cannot be represented on this compiler host",
        )
    })
}

fn unparenthesized_integer(expression: &Node<resolved::Expression>) -> Option<&IntegerLiteral> {
    match &expression.kind {
        resolved::Expression::Integer(literal) => Some(literal),
        resolved::Expression::Parenthesized(inner) => unparenthesized_integer(inner),
        _ => None,
    }
}

fn is_contextual_integer(expression: &Node<resolved::Expression>) -> bool {
    unparenthesized_integer(expression).is_some_and(|literal| literal.suffix.is_none())
}

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit])
}

fn function_placeholder() -> Type {
    Type::Function {
        parameter: Box::new(Type::Unit),
        result: Box::new(Type::Unit),
    }
}

pub(super) fn type_name(ty: &Type) -> String {
    match ty {
        Type::Unit => "Unit".into(),
        Type::Int32 => "Int32".into(),
        Type::Sum(members) if *members == vec![Type::Unit, Type::Unit] => "Bool".into(),
        Type::Sum(members) => format!(
            "[{}]",
            members.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        Type::Function { parameter, result } => {
            format!("{} -> {}", type_name(parameter), type_name(result))
        }
    }
}
