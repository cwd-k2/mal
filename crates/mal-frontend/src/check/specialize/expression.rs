//! Specialization of value expressions: substitutes type arguments, requests the instances they name, and
//! gives the binders of an instance their own identities.

use std::collections::HashMap;

use crate::resolve::ast::{LambdaId, ValueId};
use mal_syntax::diagnostic::Diagnostic;

use super::super::ast::*;
use super::super::types::{runtime_type, substitute_type};
use super::Specializer;
use super::substitution::pattern;

impl Specializer {
    pub(super) fn expression(
        &mut self,
        expression: &mut Expression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        if matches!(&expression.kind, ExpressionKind::Binary { .. }) {
            return self.binary_expression(expression, substitutions, self_instance);
        }
        expression.ty = runtime_type(&substitute_type(
            &expression.ty,
            substitutions,
            expression.span,
        )?);
        match &mut expression.kind {
            ExpressionKind::GenericReference {
                reference,
                arguments,
            } => {
                for argument in arguments.iter_mut() {
                    *argument = substitute_type(argument, substitutions, expression.span)?;
                }
                let reference = self.request(reference, arguments)?;
                expression.kind = ExpressionKind::Reference(reference);
            }
            ExpressionKind::OperationReference { family, arguments } => {
                for argument in arguments.iter_mut() {
                    *argument = substitute_type(argument, substitutions, expression.span)?;
                }
                let reference = self.request_operation(family, arguments)?;
                expression.kind = ExpressionKind::Reference(reference);
            }
            ExpressionKind::Reference(reference) => {
                if let Some((generic, specialized)) = self_instance
                    && reference.id == generic
                {
                    reference.id = specialized;
                } else if self_instance.is_some()
                    && let Some(renamed) = self.renamed_reference(reference.id)
                {
                    reference.id = renamed;
                } else {
                    self.request_binding(reference.id)?;
                }
            }
            ExpressionKind::Product(values) => {
                for value in values {
                    self.expression(value, substitutions, self_instance)?;
                }
            }
            ExpressionKind::Parenthesized(value)
            | ExpressionKind::NumericConversion { value }
            | ExpressionKind::SumInjection { value, .. } => {
                self.expression(value, substitutions, self_instance)?
            }
            ExpressionKind::Block(block) => self.block(block, substitutions, self_instance)?,
            ExpressionKind::ResultBlock {
                target,
                result_binders,
                body,
            } => {
                if self_instance.is_some() {
                    *target = self.rename_value(*target, expression.span)?;
                    for binder in result_binders.iter_mut() {
                        self.rename_binding(&mut binder.binding)?;
                    }
                }
                for binder in result_binders {
                    binder.parameter_type = runtime_type(&substitute_type(
                        &binder.parameter_type,
                        substitutions,
                        binder.binding.name.span,
                    )?);
                }
                self.block(body, substitutions, self_instance)?;
            }
            ExpressionKind::Lambda(lambda) => {
                if self_instance.is_some() {
                    let generic = lambda.id;
                    lambda.id = LambdaId(self.next_lambda);
                    self.rename_lambda(generic, lambda.id);
                    self.next_lambda = self.next_lambda.checked_add(1).ok_or_else(|| {
                        Diagnostic::error("compiler identity space exhausted").with_primary(
                            expression.span,
                            "cannot allocate a specialized lambda identity",
                        )
                    })?;
                }
                lambda.parameter_type = runtime_type(&substitute_type(
                    &lambda.parameter_type,
                    substitutions,
                    expression.span,
                )?);
                lambda.result_type = runtime_type(&substitute_type(
                    &lambda.result_type,
                    substitutions,
                    expression.span,
                )?);
                if self_instance.is_some() {
                    for capture in &mut lambda.captures {
                        if let Some(renamed) = self.renamed_reference(capture.source.id) {
                            capture.source.id = renamed;
                        }
                        self.rename_binding(&mut capture.binding)?;
                    }
                    if let Some(parameter) = &mut lambda.parameter {
                        self.rename_pattern(parameter)?;
                    }
                }
                if let Some(parameter) = &mut lambda.parameter {
                    pattern(parameter, substitutions)?;
                }
                for capture in &mut lambda.captures {
                    capture.ty = runtime_type(&substitute_type(
                        &capture.ty,
                        substitutions,
                        capture.binding.name.span,
                    )?);
                }
                if let Some((generic, specialized)) = self_instance {
                    if lambda.self_binding == Some(generic) {
                        lambda.self_binding = Some(specialized);
                    } else if let Some(binding) = &mut lambda.self_binding
                        && let Some(renamed) = self.renamed_reference(*binding)
                    {
                        *binding = renamed;
                    }
                }
                self.body(&mut lambda.body, substitutions, self_instance)?;
            }
            ExpressionKind::Call { callee, argument } => {
                self.expression(callee, substitutions, self_instance)?;
                self.expression(argument, substitutions, self_instance)?;
            }
            ExpressionKind::Memory { operands, .. }
            | ExpressionKind::SymbolOperation { operands, .. } => {
                for operand in operands {
                    self.expression(operand, substitutions, self_instance)?;
                }
            }
            ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                self.expression(scrutinee, substitutions, self_instance)?;
                for continuation in continuations {
                    self.continuation(continuation, substitutions, self_instance)?;
                }
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, substitutions, self_instance)?;
                self.block(then_branch, substitutions, self_instance)?;
                self.block(else_branch, substitutions, self_instance)?;
            }
            ExpressionKind::Unary { operand, .. } => {
                self.expression(operand, substitutions, self_instance)?
            }
            ExpressionKind::Binary { .. } => {
                unreachable!("binary expressions are walked iteratively")
            }
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::Unit => {}
        }
        Ok(())
    }

    fn binary_expression(
        &mut self,
        expression: &mut Expression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![expression];
        while let Some(expression) = pending.pop() {
            expression.ty = runtime_type(&substitute_type(
                &expression.ty,
                substitutions,
                expression.span,
            )?);
            if matches!(&expression.kind, ExpressionKind::Binary { .. }) {
                let ExpressionKind::Binary { left, right, .. } = &mut expression.kind else {
                    unreachable!()
                };
                pending.push(right);
                pending.push(left);
                continue;
            }
            self.expression(expression, substitutions, self_instance)?;
        }
        Ok(())
    }
}
