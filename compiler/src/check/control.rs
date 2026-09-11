use crate::ast::Node;
use crate::resolve::ast as resolved;

use super::ast::{
    AbruptExpression, AbruptExpressionKind, Completion, Expression, ExpressionBlock,
    ExpressionKind, Type,
};
use super::types::bool_type;
use super::{CheckFailure, CheckResult, Checker};

impl Checker {
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
        let mut items = Vec::with_capacity(block.items.len());
        for item in &block.items {
            match self.check_body_item(item) {
                Ok(item) => items.push(item),
                Err(CheckFailure::Abrupt(abrupt)) => {
                    return Err(CheckFailure::Diagnostic(
                        crate::diagnostic::Diagnostic::error(
                            "unreachable code after abrupt completion",
                        )
                        .with_primary(
                            block.result.span,
                            format!(
                                "this expression cannot be reached after control leaves at byte {}",
                                abrupt.span.start()
                            ),
                        ),
                    ));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(ExpressionBlock {
            items,
            result: Box::new(self.check_completion(&block.result, expected)?),
            span: block.span,
        })
    }
}
