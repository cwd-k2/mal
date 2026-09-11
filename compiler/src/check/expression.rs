use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::Checker;
use super::ast::{Capture, Expression, ExpressionKind, Lambda, LambdaBody, Type};
use super::float::is_float;
use super::integer::is_integer;
use super::types::{function_placeholder, type_name};

impl Checker {
    pub(super) fn check_expression(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let checked =
            match &expression.kind {
                resolved::Expression::Reference(reference) => Expression {
                    kind: ExpressionKind::Reference(reference.clone()),
                    ty: self.value_type(reference)?,
                    span: expression.span,
                },
                resolved::Expression::Integer(literal) => {
                    self.check_integer(literal, expression.span, expected)?
                }
                resolved::Expression::Float(literal) => {
                    self.check_float(literal, expression.span, expected)?
                }
                resolved::Expression::Byte(value) => Expression {
                    kind: ExpressionKind::Integer(i128::from(*value)),
                    ty: Type::UInt8,
                    span: expression.span,
                },
                resolved::Expression::Symbol(value) => Expression {
                    kind: ExpressionKind::Symbol(value.clone()),
                    ty: Type::Symbol,
                    span: expression.span,
                },
                resolved::Expression::TypeQualifiedPrimitive { type_ref, member } => {
                    self.check_qualified_memory_primitive(type_ref, member, expression.span)?
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
                resolved::Expression::Product(elements) => {
                    self.check_product(elements, expression.span, expected)?
                }
                resolved::Expression::Lambda(lambda) => {
                    self.check_lambda(lambda, expression.span, expected)?
                }
                resolved::Expression::Call { callee, arguments } => {
                    self.check_call(callee, arguments, expression.span)?
                }
                resolved::Expression::ContinuationApplication {
                    value,
                    continuations,
                } => self.check_continuation_application(
                    value,
                    continuations,
                    expression.span,
                    expected,
                )?,
                resolved::Expression::Conversion {
                    type_ref,
                    lambda_id,
                    value,
                } => {
                    let target = self.expand_type_id(type_ref.id, type_ref.name.span)?;
                    if let Type::Sum(members) = &target {
                        let resolved::Expression::Integer(index) = &value.kind else {
                            return Err(Diagnostic::error(
                                "sum constructor requires a variant index",
                            )
                            .with_primary(value.span, "use a compile-time integer literal here"));
                        };
                        let index = super::integer::parse_index(index, value.span)?;
                        let member = members.get(index).ok_or_else(|| {
                            Diagnostic::error("sum variant index is out of range").with_primary(
                                value.span,
                                format!("this sum has {} variants", members.len()),
                            )
                        })?;
                        Expression {
                            kind: ExpressionKind::InjectionConstructor {
                                lambda_id: *lambda_id,
                                index,
                            },
                            ty: Type::Function {
                                parameter: Box::new(member.clone()),
                                result: Box::new(target.clone()),
                            },
                            span: expression.span,
                        }
                    } else {
                        if !is_integer(&target) && !is_float(&target) {
                            return Err(Diagnostic::error(
                                "numeric conversion requires a numeric type",
                            )
                            .with_primary(type_ref.name.span, "this is not a numeric type"));
                        }
                        let value = self.check_expression(value, None)?;
                        if !is_integer(&value.ty) && !is_float(&value.ty) {
                            return Err(Diagnostic::error(
                                "numeric conversion requires a numeric value",
                            )
                            .with_primary(
                                value.span,
                                format!("this has type `{}`", type_name(&value.ty)),
                            ));
                        }
                        Expression {
                            kind: ExpressionKind::NumericConversion {
                                value: Box::new(value),
                            },
                            ty: target,
                            span: expression.span,
                        }
                    }
                }
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
                resolved::Expression::When { condition, body } => {
                    self.check_when(condition, body, expression.span)?
                }
                resolved::Expression::Unary { operator, operand } => {
                    self.check_unary(operator, operand, expression.span, expected)?
                }
                resolved::Expression::Binary {
                    operator,
                    left,
                    right,
                } => self.check_binary(operator, left, right, expression.span, expected)?,
            };
        if let Some(expected) = expected {
            self.require_type(&checked.ty, expected, checked.span)?;
        }
        Ok(checked)
    }

    fn check_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let (expected_parameter, expected_result) = match expected {
            Some(Type::Function { parameter, result }) => {
                (parameter.as_ref().clone(), result.as_ref().clone())
            }
            Some(other) => {
                return Err(self.type_mismatch(other, &function_placeholder(), span));
            }
            None => {
                return Err(
                    Diagnostic::error("lambda requires an expected function type").with_primary(
                        span,
                        "add a function type annotation or use this lambda in a typed context",
                    ),
                );
            }
        };

        self.check_lambda_against(lambda, span, expected_parameter, Some(&expected_result))
    }

    fn check_lambda_against(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected_parameter: Type,
        expected_result: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
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

        let parameter = match &lambda.parameter {
            Some(parameter) if expected_parameter != Type::Unit => Some(Box::new(
                self.check_pattern(parameter, &expected_parameter)?,
            )),
            None if expected_parameter == Type::Unit => None,
            _ => return Err(self.lambda_parameter_mismatch(&expected_parameter, span)),
        };
        let mut items = Vec::with_capacity(lambda.body.items.len());
        for item in &lambda.body.items {
            items.push(self.check_body_item(item)?);
        }
        let result = self.check_expression(&lambda.body.result, expected_result)?;
        let result_type = result.ty.clone();
        let ty = Type::Function {
            parameter: Box::new(expected_parameter.clone()),
            result: Box::new(result_type),
        };
        Ok(Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id: lambda.id,
                self_binding: lambda.self_binding,
                captures,
                parameter,
                parameter_type: expected_parameter,
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

    fn check_continuation_application(
        &mut self,
        value: &Node<resolved::Expression>,
        continuations: &[Node<resolved::Expression>],
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        if continuations.len() == 1
            && !matches!(continuations[0].kind, resolved::Expression::Lambda(_))
        {
            let continuation = self.check_expression(&continuations[0], None)?;
            let Type::Function { parameter, result } = &continuation.ty else {
                return Err(
                    Diagnostic::error("continuation must be a function").with_primary(
                        continuation.span,
                        format!("this has type `{}`", type_name(&continuation.ty)),
                    ),
                );
            };
            if let Some(expected) = expected {
                self.require_type(result, expected, continuation.span)?;
            }
            let result = result.as_ref().clone();
            let value = self.check_expression(value, Some(parameter))?;
            return Ok(Expression {
                kind: ExpressionKind::Call {
                    callee: Box::new(continuation),
                    argument: Box::new(value),
                },
                ty: result,
                span,
            });
        }
        let value = self.check_expression(value, None)?;
        if continuations.len() == 1 {
            let continuation = self.check_continuation(&continuations[0], &value.ty, expected)?;
            let Type::Function { result, .. } = &continuation.ty else {
                unreachable!("checked continuation has a function type");
            };
            let result = result.as_ref().clone();
            return Ok(Expression {
                kind: ExpressionKind::Call {
                    callee: Box::new(continuation),
                    argument: Box::new(value),
                },
                ty: result,
                span,
            });
        }
        let Type::Sum(members) = &value.ty else {
            return Err(
                Diagnostic::error("multiple continuations require a sum value").with_primary(
                    value.span,
                    format!("this has type `{}`", type_name(&value.ty)),
                ),
            );
        };
        let members = members.clone();
        if continuations.len() != members.len() {
            return Err(
                Diagnostic::error("sum continuation count does not match its type").with_primary(
                    span,
                    format!(
                        "expected {} continuations, found {}",
                        members.len(),
                        continuations.len()
                    ),
                ),
            );
        }
        let mut result_type = expected.cloned();
        let mut checked = Vec::with_capacity(continuations.len());
        for (continuation, member) in continuations.iter().zip(&members) {
            let continuation =
                self.check_continuation(continuation, member, result_type.as_ref())?;
            let Type::Function { result, .. } = &continuation.ty else {
                unreachable!("checked continuation has a function type");
            };
            result_type.get_or_insert_with(|| result.as_ref().clone());
            checked.push(continuation);
        }
        Ok(Expression {
            kind: ExpressionKind::SumElimination {
                scrutinee: Box::new(value),
                continuations: checked,
            },
            ty: result_type.expect("a sum has at least two continuations"),
            span,
        })
    }

    fn check_continuation(
        &mut self,
        continuation: &Node<resolved::Expression>,
        parameter: &Type,
        result: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let checked = match &continuation.kind {
            resolved::Expression::Lambda(lambda) => {
                self.check_lambda_against(lambda, continuation.span, parameter.clone(), result)?
            }
            _ => self.check_expression(continuation, None)?,
        };
        let Type::Function {
            parameter: actual_parameter,
            result: actual_result,
        } = &checked.ty
        else {
            return Err(
                Diagnostic::error("sum continuation must be a function").with_primary(
                    checked.span,
                    format!("this has type `{}`", type_name(&checked.ty)),
                ),
            );
        };
        self.require_type(actual_parameter, parameter, checked.span)?;
        if let Some(result) = result {
            self.require_type(actual_result, result, checked.span)?;
        }
        Ok(checked)
    }

    fn lambda_parameter_mismatch(&self, expected: &Type, span: crate::source::Span) -> Diagnostic {
        Diagnostic::error("lambda parameters do not match the expected function type").with_primary(
            span,
            format!("expected parameter type `{}`", type_name(expected)),
        )
    }

    fn check_call(
        &mut self,
        callee: &Node<resolved::Expression>,
        arguments: &[Node<resolved::Expression>],
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        if let resolved::Expression::TypeQualifiedPrimitive { type_ref, member } = &callee.kind {
            return self.check_qualified_memory_call(type_ref, member, arguments, span);
        }
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
        if let ExpressionKind::InjectionConstructor { index, .. } = callee.kind {
            return Ok(Expression {
                ty: result.as_ref().clone(),
                kind: ExpressionKind::SumInjection {
                    index,
                    value: Box::new(argument),
                },
                span,
            });
        }
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    pub(super) fn check_argument(
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
            _ => self.check_product(arguments, span, Some(parameter)),
        }
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
