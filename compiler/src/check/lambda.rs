use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::ast::{Capture, Expression, ExpressionKind, Lambda, LambdaBody, Type};
use super::types::{function_placeholder, type_name};
use super::{CheckResult, Checker};

impl Checker {
    pub(super) fn check_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let (expected_parameter, expected_result) = match expected {
            Some(Type::Function { parameter, result }) => {
                (parameter.as_ref().clone(), result.as_ref().clone())
            }
            Some(other) => {
                return Err(self
                    .type_mismatch(other, &function_placeholder(), span)
                    .into());
            }
            None => {
                return Err(
                    Diagnostic::error("lambda requires an expected function type")
                        .with_primary(
                            span,
                            "add a function type annotation or use this lambda in a typed context",
                        )
                        .into(),
                );
            }
        };

        self.check_lambda_against(lambda, span, expected_parameter, Some(&expected_result))
    }

    pub(super) fn check_lambda_against(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected_parameter: Type,
        expected_result: Option<&Type>,
    ) -> CheckResult<Expression> {
        let mut captures = Vec::with_capacity(lambda.captures.len());
        for capture in &lambda.captures {
            if self.scoped_values.contains(&capture.source.id) {
                return Err(Diagnostic::error("scoped authority cannot be captured")
                    .with_primary(
                        capture.source.name.span,
                        "pass it directly to a helper instead",
                    )
                    .into());
            }
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
            _ => {
                return Err(self
                    .lambda_parameter_mismatch(&expected_parameter, span)
                    .into());
            }
        };
        let declared_result = expected_result.cloned();
        let items = self.check_body_items(&lambda.body.items, lambda.body.result.span)?;
        let result = self.check_completion(&lambda.body.result, declared_result.as_ref())?;
        let result_type = match &result {
            super::ast::Completion::Value(value) => value.ty.clone(),
            super::ast::Completion::Abrupt(_) => declared_result
                .clone()
                .expect("abrupt lambda body has a declared result"),
        };
        if super::types::contains_scoped_value(&result_type) {
            return Err(Diagnostic::error("scoped authority cannot be returned")
                .with_primary(
                    lambda.body.result.span,
                    "Region and Buffer values are limited to this invocation",
                )
                .into());
        }
        let ty = Type::Function {
            parameter: expected_parameter.clone().into(),
            result: result_type.clone().into(),
        };
        Ok(Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id: lambda.id,
                self_binding: lambda.self_binding,
                captures,
                parameter,
                parameter_type: expected_parameter,
                result_type: result_type.clone(),
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

    fn lambda_parameter_mismatch(&self, expected: &Type, span: crate::source::Span) -> Diagnostic {
        Diagnostic::error("lambda parameters do not match the expected function type").with_primary(
            span,
            format!("expected parameter type `{}`", type_name(expected)),
        )
    }
}
