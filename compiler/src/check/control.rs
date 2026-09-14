use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::ast::{
    AbruptExpression, AbruptExpressionKind, Completion, Expression, ExpressionBlock,
    ExpressionKind, ReturnBoundary, Type,
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
        let return_binders = self.check_return_binders(bindings, &result_type, body.span)?;
        for binder in &return_binders {
            self.return_targets.insert(
                binder.binding.id,
                super::ReturnTarget {
                    parameter: binder.parameter_type.clone(),
                    result: result_type.clone(),
                    variant: binder.variant,
                    boundary: ReturnBoundary::Block(target),
                },
            );
        }
        let checked_body = self.check_expression_block(body, Some(&result_type));
        for binder in &return_binders {
            self.return_targets.remove(&binder.binding.id);
        }
        let body = checked_body?;
        if let Completion::Value(value) = body.result.as_ref() {
            return Err(Diagnostic::error("result block cannot fall through")
                .with_primary(value.span, "call a result binder on every reachable path")
                .into());
        }
        if !self.used_return_targets.remove(&target) {
            return Err(Diagnostic::error("result block does not produce a result")
                .with_primary(span, "call one of this block's result binders")
                .into());
        }
        Ok(Expression {
            kind: ExpressionKind::ResultBlock {
                target,
                return_binders,
                body,
            },
            ty: result_type,
            span,
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
