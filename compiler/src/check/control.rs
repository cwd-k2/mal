use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::Checker;
use super::ast::{Expression, ExpressionBlock, ExpressionKind, Type};
use super::types::bool_type;

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
}
