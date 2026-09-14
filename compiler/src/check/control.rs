use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::ast::{
    AbruptExpression, AbruptExpressionKind, Completion, Expression, ExpressionBlock,
    ExpressionKind, Type,
};
use super::types::bool_type;
use super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_block(
        &mut self,
        block: &resolved::ExpressionBlock,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let block = self.check_expression_block(block, expected)?;
        let ty = match block.result.as_ref() {
            Completion::Value(value) => value.ty.clone(),
            Completion::Abrupt(_) => {
                return Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
                    preceding: Vec::new(),
                    kind: AbruptExpressionKind::Block(block),
                    span,
                })));
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Block(block),
            ty,
            span,
        })
    }

    pub(super) fn check_result_block(
        &mut self,
        bindings: &[resolved::ValueBinding],
        body: &resolved::ExpressionBlock,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let result_type = expected.cloned().ok_or_else(|| {
            Diagnostic::error("result block requires an expected result type").with_primary(
                span,
                "add a type annotation or use this block in a typed context",
            )
        })?;
        let target = bindings
            .first()
            .expect("the parser requires a non-empty result binder group")
            .id;
        let result_binders = self.check_result_binders(bindings, &result_type, body.span)?;
        for binder in &result_binders {
            self.result_targets.insert(
                binder.binding.id,
                super::ResultTarget {
                    parameter: binder.parameter_type.clone(),
                    result: result_type.clone(),
                    variant: binder.variant,
                    boundary: target,
                },
            );
        }
        let checked_body = self.check_expression_block(body, Some(&result_type));
        for binder in &result_binders {
            self.result_targets.remove(&binder.binding.id);
        }
        let body = checked_body?;
        if let Completion::Value(value) = body.result.as_ref() {
            return Err(Diagnostic::error("result block cannot fall through")
                .with_primary(value.span, "call a result binder on every reachable path")
                .into());
        }
        if !self.used_result_targets.remove(&target) {
            return Err(Diagnostic::error("result block does not produce a result")
                .with_primary(span, "call one of this block's result binders")
                .into());
        }
        Ok(Expression {
            kind: ExpressionKind::ResultBlock {
                target,
                result_binders,
                body,
            },
            ty: result_type,
            span,
        })
    }

    fn check_result_binders(
        &self,
        bindings: &[resolved::ValueBinding],
        result: &Type,
        body_span: crate::source::Span,
    ) -> CheckResult<Vec<super::ast::ResultBinder>> {
        Ok(match bindings {
            [] => unreachable!("the parser requires a non-empty result binder group"),
            [binding] => {
                if matches!(result, Type::Sum(members) if members.is_empty()) {
                    return Err(Diagnostic::error("`[]` result has no result value")
                        .with_primary(binding.name.span, "remove the result block")
                        .into());
                }
                vec![super::ast::ResultBinder {
                    binding: binding.clone(),
                    parameter_type: result.clone(),
                    variant: None,
                }]
            }
            _ => {
                let Type::Sum(members) = result else {
                    return Err(
                        Diagnostic::error("multiple result binders require a sum result")
                            .with_primary(
                                body_span,
                                format!("result type is `{}`", super::type_name(result)),
                            )
                            .into(),
                    );
                };
                if bindings.len() != members.len() {
                    return Err(Diagnostic::error(
                        "result binder count does not match the sum result",
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
                    .map(
                        |(variant, (binding, parameter_type))| super::ast::ResultBinder {
                            binding: binding.clone(),
                            parameter_type: parameter_type.clone(),
                            variant: Some(variant),
                        },
                    )
                    .collect()
            }
        })
    }

    pub(super) fn check_when(
        &mut self,
        condition: &Node<resolved::Expression>,
        body: &resolved::ExpressionBlock,
        span: crate::source::Span,
    ) -> CheckResult<Expression> {
        let bool_type = bool_type();
        let condition = self.check_before(condition, Some(&bool_type), body.span)?;
        let body = self.check_expression_block(body, Some(&Type::Unit))?;
        if let Completion::Value(value) = body.result.as_ref() {
            self.require_type(&value.ty, &Type::Unit, value.span)?;
        }
        let unit = Expression {
            kind: ExpressionKind::Unit,
            ty: Type::Unit,
            span,
        };
        Ok(Expression {
            kind: ExpressionKind::If {
                condition: Box::new(condition),
                then_branch: body,
                else_branch: ExpressionBlock {
                    items: Vec::new(),
                    result: Box::new(Completion::Value(unit)),
                    span,
                },
            },
            ty: Type::Unit,
            span,
        })
    }

    pub(super) fn check_if(
        &mut self,
        condition: &Node<resolved::Expression>,
        then_branch: &resolved::ExpressionBlock,
        else_branch: &resolved::ExpressionBlock,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let bool_type = bool_type();
        let condition = self.check_before(condition, Some(&bool_type), then_branch.span)?;
        let then_branch = self.check_expression_block(then_branch, expected)?;
        let branch_type = match then_branch.result.as_ref() {
            Completion::Value(value) => Some(value.ty.clone()),
            Completion::Abrupt(_) => expected.cloned(),
        };
        let else_branch = self.check_expression_block(else_branch, branch_type.as_ref())?;
        let result_type = match (then_branch.result.as_ref(), else_branch.result.as_ref()) {
            (Completion::Value(then_value), Completion::Value(else_value)) => {
                self.require_type(&else_value.ty, &then_value.ty, else_value.span)?;
                Some(then_value.ty.clone())
            }
            (Completion::Value(value), Completion::Abrupt(_))
            | (Completion::Abrupt(_), Completion::Value(value)) => Some(value.ty.clone()),
            (Completion::Abrupt(_), Completion::Abrupt(_)) => None,
        };
        if let Some(ty) = result_type {
            Ok(Expression {
                kind: ExpressionKind::If {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                },
                ty,
                span,
            })
        } else {
            Err(CheckFailure::Abrupt(Box::new(AbruptExpression {
                preceding: Vec::new(),
                kind: AbruptExpressionKind::If {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                },
                span,
            })))
        }
    }

    fn check_expression_block(
        &mut self,
        block: &resolved::ExpressionBlock,
        expected: Option<&Type>,
    ) -> CheckResult<ExpressionBlock> {
        Ok(ExpressionBlock {
            items: self.check_body_items(&block.items, block.result.span)?,
            result: Box::new(self.check_completion(&block.result, expected)?),
            span: block.span,
        })
    }
}
