use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{AbruptExpression, AbruptExpressionKind, Expression, ExpressionKind, Type};
use super::super::types::type_name;
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_continuation_application(
        &mut self,
        value: &Node<resolved::Expression>,
        continuations: &[resolved::Continuation],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        match continuations {
            [] => self.check_empty_elimination(value),
            [continuation] => self.check_single_continuation(
                value,
                function_continuation(continuation),
                span,
                expected,
            ),
            _ => self.check_sum_elimination(value, continuations, span, expected),
        }
    }

    fn check_empty_elimination(
        &mut self,
        value: &Node<resolved::Expression>,
    ) -> CheckResult<Expression> {
        let value = self.check_expression(value, None)?;
        if !matches!(&value.ty, Type::Sum(members) if members.is_empty()) {
            return Err(
                Diagnostic::error("zero continuations require an `[]` value")
                    .with_primary(
                        value.span,
                        format!("this has type `{}`", type_name(&value.ty)),
                    )
                    .into(),
            );
        }
        let span = value.span;
        Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
            preceding: Vec::new(),
            kind: AbruptExpressionKind::EmptyElimination {
                scrutinee: Box::new(value),
            },
            span,
        })))
    }

    /// A single continuation is an ordinary application, whatever the value's type.
    fn check_single_continuation(
        &mut self,
        value: &Node<resolved::Expression>,
        continuation: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if let resolved::Expression::Reference(reference) = &continuation.kind
            && let Some(target) = self.result_targets.get(&reference.id).cloned()
        {
            let argument = self.check_expression(value, Some(&target.parameter))?;
            self.used_result_targets.insert(target.boundary);
            let value = if let Some(index) = target.variant {
                Expression {
                    kind: ExpressionKind::SumInjection {
                        index,
                        value: Box::new(argument),
                    },
                    ty: target.result,
                    span,
                }
            } else {
                argument
            };
            return Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
                preceding: Vec::new(),
                kind: AbruptExpressionKind::ResultTransfer {
                    target: target.boundary,
                    value: Box::new(value),
                },
                span,
            })));
        }
        if !matches!(continuation.kind, resolved::Expression::Lambda(_)) {
            let checked = self.check_expression(continuation, None)?;
            let Type::Function { parameter, result } = &checked.ty else {
                return Err(Diagnostic::error("continuation must be a function")
                    .with_primary(
                        checked.span,
                        format!("this has type `{}`", type_name(&checked.ty)),
                    )
                    .into());
            };
            if let Some(expected) = expected {
                self.require_type(result, expected, checked.span)?;
            }
            let result = result.as_ref().clone();
            let value = self.check_before(value, Some(parameter), continuation.span)?;
            return Ok(Expression {
                kind: ExpressionKind::Call {
                    callee: Box::new(checked),
                    argument: Box::new(value),
                },
                ty: result,
                span,
            });
        }
        let value = self.check_before(value, None, continuation.span)?;
        let checked = self.check_continuation(continuation, &value.ty, expected)?;
        let Type::Function { result, .. } = &checked.ty else {
            unreachable!("checked continuation has a function type");
        };
        let result = result.as_ref().clone();
        Ok(Expression {
            kind: ExpressionKind::Call {
                callee: Box::new(checked),
                argument: Box::new(value),
            },
            ty: result,
            span,
        })
    }

    /// Checks a function-valued continuation against the payload type it will receive.
    pub(super) fn check_continuation(
        &mut self,
        continuation: &Node<resolved::Expression>,
        parameter: &Type,
        result: Option<&Type>,
    ) -> CheckResult<Expression> {
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
            return Err(Diagnostic::error("sum continuation must be a function")
                .with_primary(
                    checked.span,
                    format!("this has type `{}`", type_name(&checked.ty)),
                )
                .into());
        };
        self.require_type(actual_parameter, parameter, checked.span)?;
        if let Some(result) = result {
            self.require_type(actual_result, result, checked.span)?;
        }
        Ok(checked)
    }

    pub(super) fn check_call(
        &mut self,
        callee: &Node<resolved::Expression>,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        _expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if let resolved::Expression::GenericReference {
            reference,
            arguments: type_arguments,
        } = &callee.kind
            && matches!(
                reference.id,
                crate::resolve::MAKE_VALUE | crate::resolve::FROM_VALUE
            )
        {
            return self.check_memory_intrinsic(reference, type_arguments, arguments, span);
        }
        if let resolved::Expression::Reference(reference) = &callee.kind
            && matches!(
                reference.id,
                crate::resolve::NEW_VALUE
                    | crate::resolve::GET_VALUE
                    | crate::resolve::PUT_VALUE
                    | crate::resolve::FILL_VALUE
                    | crate::resolve::COPY_VALUE
                    | crate::resolve::INTO_VALUE
            )
        {
            return self.check_memory_operation(reference, arguments, span);
        }
        if let resolved::Expression::Reference(reference) = &callee.kind
            && let Some(target) = self.result_targets.get(&reference.id).cloned()
        {
            let argument = self.check_argument(arguments, &target.parameter, span)?;
            self.used_result_targets.insert(target.boundary);
            let value = if let Some(index) = target.variant {
                Expression {
                    kind: ExpressionKind::SumInjection {
                        index,
                        value: Box::new(argument),
                    },
                    ty: target.result,
                    span,
                }
            } else {
                argument
            };
            return Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
                preceding: Vec::new(),
                kind: AbruptExpressionKind::ResultTransfer {
                    target: target.boundary,
                    value: Box::new(value),
                },
                span,
            })));
        }
        let callee = match self.check_expression(callee, None) {
            Ok(callee) => callee,
            Err(CheckFailure::Abrupt(abrupt)) => {
                let argument = self.check_untyped_argument(arguments, span)?;
                return Err(CheckFailure::Abrupt(Box::new(
                    (*abrupt).preceded_by(vec![argument]),
                )));
            }
            Err(error) => return Err(error),
        };
        let Type::Function { parameter, result } = &callee.ty else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(
                    callee.span,
                    format!("this has type `{}`", type_name(&callee.ty)),
                )
                .into());
        };
        let argument = match self.check_argument(arguments, parameter, span) {
            Ok(argument) => argument,
            Err(CheckFailure::Abrupt(_)) => {
                return Err(Diagnostic::error(
                    "function value is unreachable after abrupt argument evaluation",
                )
                .with_primary(callee.span, "this callee cannot be evaluated")
                .into());
            }
            Err(error) => return Err(error),
        };
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    pub(crate) fn check_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        parameter: &Type,
        span: Span,
    ) -> CheckResult<Expression> {
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
            _ => {
                let argument = self.check_product(arguments, span, Some(parameter))?;
                self.require_type(&argument.ty, parameter, argument.span)?;
                Ok(argument)
            }
        }
    }

    fn check_untyped_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        match arguments {
            [] => Ok(Expression {
                kind: ExpressionKind::Unit,
                ty: Type::Unit,
                span,
            }),
            [argument] => self.check_expression(argument, None),
            _ => self.check_product(arguments, span, None),
        }
    }
}

/// Continuations with fewer than two entries are never branches, so the resolver only produces functions.
fn function_continuation(continuation: &resolved::Continuation) -> &Node<resolved::Expression> {
    match continuation {
        resolved::Continuation::Function(expression) => expression,
        resolved::Continuation::Branch(_) => unreachable!("only a sum elimination has branches"),
    }
}
