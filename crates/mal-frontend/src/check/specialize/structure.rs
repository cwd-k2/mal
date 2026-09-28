//! Specialization of the blocks, bodies, and continuations that carry binders and completions.

use std::collections::HashMap;

use crate::resolve::ast::ValueId;
use mal_syntax::diagnostic::Diagnostic;

use super::super::ast::*;
use super::super::types::{runtime_type, substitute_type};
use super::Specializer;
use super::substitution::{completion, pattern};

impl Specializer {
    pub(super) fn block(
        &mut self,
        block: &mut ExpressionBlock,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        for item in &mut block.items {
            match item {
                BodyItem::Binding(binding) => {
                    if self_instance.is_some() {
                        self.rename_pattern(&mut binding.pattern)?;
                    }
                    pattern(&mut binding.pattern, substitutions);
                    self.expression(&mut binding.value, substitutions, self_instance)?;
                }
                BodyItem::Expression(value) => {
                    self.expression(value, substitutions, self_instance)?
                }
            }
        }
        completion(&mut block.result, substitutions);
        match block.result.as_mut() {
            Completion::Value(value) => self.expression(value, substitutions, self_instance)?,
            Completion::Abrupt(abrupt) => self.abrupt(abrupt, substitutions, self_instance)?,
        }
        Ok(())
    }

    pub(super) fn body(
        &mut self,
        body: &mut LambdaBody,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        let mut block = ExpressionBlock {
            items: std::mem::take(&mut body.items),
            result: std::mem::replace(
                &mut body.result,
                Box::new(Completion::Value(Expression {
                    kind: ExpressionKind::Unit,
                    ty: Type::Unit,
                    span: body.span,
                })),
            ),
            span: body.span,
        };
        self.block(&mut block, substitutions, self_instance)?;
        body.items = block.items;
        body.result = block.result;
        Ok(())
    }

    pub(super) fn continuation(
        &mut self,
        continuation: &mut SumContinuation,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        match continuation {
            SumContinuation::Function(expression) => {
                self.expression(expression, substitutions, self_instance)
            }
            SumContinuation::Branch(branch) => {
                branch.parameter_type =
                    runtime_type(&substitute_type(&branch.parameter_type, substitutions));
                if let Some(parameter) = &mut branch.parameter {
                    if self_instance.is_some() {
                        self.rename_pattern(parameter)?;
                    }
                    pattern(parameter, substitutions);
                }
                self.block(&mut branch.body, substitutions, self_instance)
            }
            SumContinuation::Transfer(transfer) => {
                if self_instance.is_some()
                    && let Some(renamed) = self.renamed_reference(transfer.target)
                {
                    transfer.target = renamed;
                }
                transfer.payload_type =
                    runtime_type(&substitute_type(&transfer.payload_type, substitutions));
                transfer.result_type =
                    runtime_type(&substitute_type(&transfer.result_type, substitutions));
                Ok(())
            }
        }
    }

    pub(super) fn abrupt(
        &mut self,
        abrupt: &mut AbruptExpression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        for value in &mut abrupt.preceding {
            self.expression(value, substitutions, self_instance)?;
        }
        match &mut abrupt.kind {
            AbruptExpressionKind::ResultTransfer { target, value, .. } => {
                if self_instance.is_some()
                    && let Some(renamed) = self.renamed_reference(*target)
                {
                    *target = renamed;
                }
                self.expression(value, substitutions, self_instance)?
            }
            AbruptExpressionKind::EmptyElimination { scrutinee: value } => {
                self.expression(value, substitutions, self_instance)?
            }
            AbruptExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, substitutions, self_instance)?;
                self.block(then_branch, substitutions, self_instance)?;
                self.block(else_branch, substitutions, self_instance)?;
            }
            AbruptExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                self.expression(scrutinee, substitutions, self_instance)?;
                for continuation in continuations {
                    self.continuation(continuation, substitutions, self_instance)?;
                }
            }
            AbruptExpressionKind::Block(block) => {
                self.block(block, substitutions, self_instance)?
            }
        }
        Ok(())
    }
}
