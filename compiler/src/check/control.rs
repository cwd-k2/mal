use std::collections::HashSet;

use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::Checker;
use super::ast::{CaseArm, Expression, ExpressionBlock, ExpressionKind, Type};
use super::expression::{bool_type, type_name};
use super::integer::parse_index;

impl Checker {
    pub(super) fn check_if(
        &mut self,
        condition: &Node<resolved::Expression>,
        then_branch: &resolved::ExpressionBlock,
        else_branch: &resolved::ExpressionBlock,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let bool_type = bool_type();
        let condition = self.check_expression(condition, Some(&bool_type))?;
        let then_branch = self.check_expression_block(then_branch, expected)?;
        let branch_type = then_branch.result.ty.clone();
        let else_branch = self.check_expression_block(else_branch, Some(&branch_type))?;
        Ok(Expression {
            kind: ExpressionKind::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            ty: branch_type,
            span,
        })
    }

    fn check_expression_block(
        &mut self,
        block: &resolved::ExpressionBlock,
        expected: Option<&Type>,
    ) -> Result<ExpressionBlock, Diagnostic> {
        let mut items = Vec::with_capacity(block.items.len());
        for item in &block.items {
            items.push(self.check_body_item(item)?);
        }
        Ok(ExpressionBlock {
            items,
            result: Box::new(self.check_expression(&block.result, expected)?),
            span: block.span,
        })
    }

    pub(super) fn check_case(
        &mut self,
        scrutinee: &Node<resolved::Expression>,
        arms: &[resolved::CaseArm],
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let scrutinee = self.check_expression(scrutinee, None)?;
        let Type::Sum(members) = &scrutinee.ty else {
            return Err(Diagnostic::error("case requires a sum value").with_primary(
                scrutinee.span,
                format!("this has type `{}`", type_name(&scrutinee.ty)),
            ));
        };
        let members = members.clone();
        let mut seen = HashSet::new();
        let mut checked_arms = Vec::with_capacity(arms.len());
        let mut result_type = expected.cloned();
        for arm in arms {
            let index = parse_index(&arm.index.kind, arm.index.span)?;
            let member = members.get(index).ok_or_else(|| {
                Diagnostic::error("case variant index is out of range").with_primary(
                    arm.index.span,
                    format!("this sum has {} variants", members.len()),
                )
            })?;
            if !seen.insert(index) {
                return Err(
                    Diagnostic::error(format!("duplicate case arm for variant {index}"))
                        .with_primary(arm.index.span, "this variant was already handled"),
                );
            }
            let pattern = self.check_pattern(&arm.pattern, member)?;
            let value = self.check_expression(&arm.value, result_type.as_ref())?;
            result_type.get_or_insert_with(|| value.ty.clone());
            checked_arms.push(CaseArm {
                index,
                pattern,
                value,
                span: arm.span,
            });
        }
        if seen.len() != members.len() {
            return Err(Diagnostic::error("non-exhaustive case expression")
                .with_primary(span, "every sum variant must have one arm"));
        }
        Ok(Expression {
            kind: ExpressionKind::Case {
                scrutinee: Box::new(scrutinee),
                arms: checked_arms,
            },
            ty: result_type.expect("the parser requires at least one case arm"),
            span,
        })
    }
}
