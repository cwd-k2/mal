use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::ast::{Capture, Expression, ExpressionKind, Lambda, LambdaBody, ReturnBinder, Type};
use super::types::{function_placeholder, type_name};
use super::{CheckResult, Checker, ReturnTarget};

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
        let return_binders = match &lambda.return_binders {
            None => None,
            Some(bindings) => {
                let result = declared_result.as_ref().ok_or_else(|| {
                    Diagnostic::error("return binder requires an expected function result type")
                        .with_primary(span, "use this lambda in a fully typed context")
                })?;
                Some(self.check_return_binders(bindings, result, lambda.body.span)?)
            }
        };
        if let Some(binders) = &return_binders {
            let result = declared_result.as_ref().expect("return binder result");
            for binder in binders {
                self.return_targets.insert(
                    binder.binding.id,
                    ReturnTarget {
                        parameter: binder.parameter_type.clone(),
                        result: result.clone(),
                        variant: binder.variant,
                        boundary: super::ast::ReturnBoundary::Lambda,
                    },
                );
            }
        }

        let body_result: CheckResult<_> = (|| {
            let items = self.check_body_items(&lambda.body.items, lambda.body.result.span)?;
            let result = self.check_completion(&lambda.body.result, declared_result.as_ref())?;
            Ok((items, result))
        })();
        if let Some(binders) = &return_binders {
            for binder in binders {
                self.return_targets.remove(&binder.binding.id);
            }
        }
        let (items, result) = body_result?;
        match (&return_binders, &result) {
            (None, super::ast::Completion::Abrupt(abrupt)) => {
                return Err(Diagnostic::error(
                    "lambda without return binders must produce a value",
                )
                .with_primary(abrupt.span, "this path completes abruptly")
                .into());
            }
            (Some(_), super::ast::Completion::Value(value)) => {
                return Err(
                    Diagnostic::error("lambda with return binders cannot fall through")
                        .with_primary(value.span, "call a return binder on every reachable path")
                        .into(),
                );
            }
            _ => {}
        }
        let result_type = match &result {
            super::ast::Completion::Value(value) => value.ty.clone(),
            super::ast::Completion::Abrupt(_) => declared_result
                .clone()
                .expect("abrupt lambda body has a declared result"),
        };
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
                return_binders,
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

    pub(super) fn check_return_binders(
        &self,
        bindings: &[resolved::ValueBinding],
        result: &Type,
        body_span: crate::source::Span,
    ) -> CheckResult<Vec<ReturnBinder>> {
        Ok(match bindings {
            [] => {
                if !matches!(result, Type::Sum(members) if members.is_empty()) {
                    return Err(Diagnostic::error(
                        "empty return binder group requires `[]` result",
                    )
                    .with_primary(body_span, format!("result type is `{}`", type_name(result)))
                    .into());
                }
                Vec::new()
            }
            [binding] => {
                if matches!(result, Type::Sum(members) if members.is_empty()) {
                    return Err(Diagnostic::error("`[]` result has no return value")
                        .with_primary(binding.name.span, "use an empty return binder group")
                        .into());
                }
                vec![ReturnBinder {
                    binding: binding.clone(),
                    parameter_type: result.clone(),
                    variant: None,
                }]
            }
            _ => {
                let Type::Sum(members) = result else {
                    return Err(
                        Diagnostic::error("multiple return binders require a sum result")
                            .with_primary(
                                body_span,
                                format!("result type is `{}`", type_name(result)),
                            )
                            .into(),
                    );
                };
                if bindings.len() != members.len() {
                    return Err(Diagnostic::error(
                        "return binder count does not match the sum result",
                    )
                    .with_primary(
                        body_span,
                        format!(
                            "expected {} binders, found {}",
                            members.len(),
                            bindings.len()
                        ),
                    )
                    .into());
                }
                bindings
                    .iter()
                    .zip(members.iter())
                    .enumerate()
                    .map(|(variant, (binding, parameter_type))| ReturnBinder {
                        binding: binding.clone(),
                        parameter_type: parameter_type.clone(),
                        variant: Some(variant),
                    })
                    .collect()
            }
        })
    }

    fn lambda_parameter_mismatch(&self, expected: &Type, span: crate::source::Span) -> Diagnostic {
        Diagnostic::error("lambda parameters do not match the expected function type").with_primary(
            span,
            format!("expected parameter type `{}`", type_name(expected)),
        )
    }
}
