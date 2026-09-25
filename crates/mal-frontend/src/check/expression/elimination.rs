use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{
    AbruptExpression, AbruptExpressionKind, Completion, Expression, ExpressionKind, SumBranch,
    SumContinuation, SumTransfer, Type,
};
use super::super::types::type_name;
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    /// Checks a sum elimination, whose continuations are function values, branches of the enclosing
    /// invocation, or result binder names. The `Value` continuations join like the branches of an `if`.
    pub(super) fn check_sum_elimination(
        &mut self,
        value: &Node<resolved::Expression>,
        continuations: &[resolved::Continuation],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let value = self.check_before(value, None, continuation_span(&continuations[0]))?;
        let Type::Sum(members) = &value.ty else {
            return Err(
                Diagnostic::error("multiple continuations require a sum value")
                    .with_primary(
                        value.span,
                        format!("this has type `{}`", type_name(&value.ty)),
                    )
                    .into(),
            );
        };
        let members = members.clone();
        if continuations.len() != members.len() {
            return Err(
                Diagnostic::error("sum continuation count does not match its type")
                    .with_primary(
                        span,
                        format!(
                            "expected {} continuations, found {}",
                            members.len(),
                            continuations.len()
                        ),
                    )
                    .into(),
            );
        }
        let mut value_type: Option<Type> = None;
        let mut checked = Vec::with_capacity(continuations.len());
        for (position, (continuation, member)) in
            continuations.iter().zip(members.iter()).enumerate()
        {
            let hint = value_type.as_ref().or(expected);
            let (continuation, completion) =
                self.check_sum_continuation(continuation, position, member, hint)?;
            if let Some(ty) = completion {
                match &value_type {
                    Some(existing) => {
                        self.require_type(&ty, existing, checked_span(&continuation))?
                    }
                    None => value_type = Some(ty),
                }
            }
            checked.push(continuation);
        }
        match value_type {
            Some(ty) => Ok(Expression {
                kind: ExpressionKind::SumElimination {
                    scrutinee: Box::new(value),
                    continuations: checked,
                },
                ty,
                span,
            }),
            None => Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
                preceding: Vec::new(),
                kind: AbruptExpressionKind::SumElimination {
                    scrutinee: Box::new(value),
                    continuations: checked,
                },
                span,
            }))),
        }
    }

    /// Returns the checked continuation and the type it completes with, or `None` when it is `Abrupt`.
    fn check_sum_continuation(
        &mut self,
        continuation: &resolved::Continuation,
        position: usize,
        payload: &Type,
        hint: Option<&Type>,
    ) -> CheckResult<(SumContinuation, Option<Type>)> {
        match continuation {
            resolved::Continuation::Branch(branch) => {
                let (branch, completion) = self.check_sum_branch(branch, payload, hint)?;
                Ok((SumContinuation::Branch(branch), completion))
            }
            resolved::Continuation::Function(expression) => {
                if let Some(transfer) =
                    self.check_result_binder_continuation(expression, payload, position)?
                {
                    return Ok((SumContinuation::Transfer(transfer), None));
                }
                let checked = self.check_continuation(expression, payload, hint)?;
                let Type::Function { result, .. } = &checked.ty else {
                    unreachable!("checked continuation has a function type");
                };
                let result = result.as_ref().clone();
                Ok((SumContinuation::Function(checked), Some(result)))
            }
        }
    }

    fn check_sum_branch(
        &mut self,
        branch: &resolved::Branch,
        payload: &Type,
        hint: Option<&Type>,
    ) -> CheckResult<(SumBranch, Option<Type>)> {
        let parameter = match &branch.parameter {
            Some(parameter) if payload != &Type::Unit => {
                Some(Box::new(self.check_pattern(parameter, payload)?))
            }
            None if payload == &Type::Unit => None,
            _ => {
                return Err(self.lambda_parameter_mismatch(payload, branch.span).into());
            }
        };
        let body = self.check_expression_block(&branch.body, hint)?;
        let completion = match body.result.as_ref() {
            Completion::Value(value) => Some(value.ty.clone()),
            Completion::Abrupt(_) => None,
        };
        Ok((
            SumBranch {
                parameter,
                parameter_type: payload.clone(),
                body,
                span: branch.span,
            },
            completion,
        ))
    }

    /// A result binder name placed as a continuation receives the payload directly. Parentheses carry no
    /// meaning, so a parenthesized name is the same continuation.
    fn check_result_binder_continuation(
        &mut self,
        continuation: &Node<resolved::Expression>,
        payload: &Type,
        position: usize,
    ) -> CheckResult<Option<SumTransfer>> {
        let mut current = continuation;
        while let resolved::Expression::Parenthesized(inner) = &current.kind {
            current = inner;
        }
        let resolved::Expression::Reference(reference) = &current.kind else {
            return Ok(None);
        };
        let Some(target) = self.result_targets.get(&reference.id).cloned() else {
            return Ok(None);
        };
        if payload != &target.parameter {
            return Err(Diagnostic::error("result binder does not accept this payload")
                .with_primary(
                    current.span,
                    format!(
                        "`{}` takes `{}`, but this continuation receives `{}`",
                        reference.name.text,
                        type_name(&target.parameter),
                        type_name(payload)
                    ),
                )
                .with_note(format!(
                    "continuation {position} of the sum carries `{}`; name a result binder of that type, \
                     or write a lambda that converts the payload",
                    type_name(payload)
                ))
                .into());
        }
        self.used_result_targets.insert(target.boundary);
        Ok(Some(SumTransfer {
            target: target.boundary,
            variant: target.variant,
            payload_type: payload.clone(),
            result_type: target.result,
            span: current.span,
        }))
    }
}

fn continuation_span(continuation: &resolved::Continuation) -> Span {
    match continuation {
        resolved::Continuation::Function(expression) => expression.span,
        resolved::Continuation::Branch(branch) => branch.span,
    }
}

fn checked_span(continuation: &SumContinuation) -> Span {
    match continuation {
        SumContinuation::Function(expression) => expression.span,
        SumContinuation::Branch(branch) => branch.span,
        SumContinuation::Transfer(transfer) => transfer.span,
    }
}
