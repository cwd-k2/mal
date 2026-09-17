use crate::resolve::ast::{LambdaId, ValueBinding, ValueId, ValueOwner};

use super::GenericDefinition;
use super::ast::*;

pub(super) struct NextIdentities {
    pub(super) value: u32,
    pub(super) lambda: u32,
}

pub(super) fn next_identities(
    program: &Program,
    definitions: &[GenericDefinition],
) -> Option<NextIdentities> {
    let mut bounds = IdentityBounds::default();
    for item in &program.items {
        match &item.kind {
            TopItem::ExternalOperation {
                binding, lambda_id, ..
            } => {
                bounds.binding(binding);
                bounds.lambda(*lambda_id);
            }
            TopItem::Binding(binding) => bounds.binding_value(binding),
            TopItem::TypeAlias { .. } | TopItem::ExternalType { .. } => {}
        }
    }
    for definition in definitions {
        bounds.binding(&definition.binding);
        bounds.expression(&definition.value);
    }
    Some(NextIdentities {
        value: bounds.value.checked_add(1)?,
        lambda: bounds.lambda.checked_add(1)?,
    })
}

#[derive(Default)]
struct IdentityBounds {
    value: u32,
    lambda: u32,
}

impl IdentityBounds {
    fn value(&mut self, id: ValueId) {
        self.value = self.value.max(id.0);
    }

    fn lambda(&mut self, id: LambdaId) {
        self.lambda = self.lambda.max(id.0);
    }

    fn binding(&mut self, binding: &ValueBinding) {
        self.value(binding.id);
        if let ValueOwner::Lambda(id) | ValueOwner::Result(id) = binding.owner {
            self.lambda(id);
        }
    }

    fn pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { binding, .. } => self.binding(binding),
            Pattern::Product { elements, .. } => {
                for element in elements {
                    self.pattern(element);
                }
            }
            Pattern::Wildcard { .. } => {}
        }
    }

    fn binding_value(&mut self, binding: &Binding) {
        self.pattern(&binding.pattern);
        self.expression(&binding.value);
    }

    fn expression(&mut self, expression: &Expression) {
        match &expression.kind {
            ExpressionKind::Reference(reference) => self.value(reference.id),
            ExpressionKind::GenericReference { reference, .. } => self.value(reference.id),
            ExpressionKind::Product(values) => {
                for value in values {
                    self.expression(value);
                }
            }
            ExpressionKind::Parenthesized(value)
            | ExpressionKind::SymbolLength { value }
            | ExpressionKind::NumericConversion { value }
            | ExpressionKind::SumInjection { value, .. } => self.expression(value),
            ExpressionKind::Block(block) => self.block(block),
            ExpressionKind::ResultBlock {
                target,
                result_binders,
                body,
            } => {
                self.value(*target);
                for binder in result_binders {
                    self.binding(&binder.binding);
                }
                self.block(body);
            }
            ExpressionKind::Lambda(lambda) => {
                self.lambda(lambda.id);
                if let Some(binding) = lambda.self_binding {
                    self.value(binding);
                }
                for capture in &lambda.captures {
                    self.value(capture.source.id);
                    self.binding(&capture.binding);
                }
                if let Some(parameter) = &lambda.parameter {
                    self.pattern(parameter);
                }
                self.body(&lambda.body);
            }
            ExpressionKind::Call { callee, argument } => {
                self.expression(callee);
                self.expression(argument);
            }
            ExpressionKind::SymbolAt { argument } | ExpressionKind::Memory { argument, .. } => {
                self.expression(argument);
            }
            ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                self.expression(scrutinee);
                for continuation in continuations {
                    self.expression(continuation);
                }
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition);
                self.block(then_branch);
                self.block(else_branch);
            }
            ExpressionKind::Unary { operand, .. } => self.expression(operand),
            ExpressionKind::Binary { left, right, .. } => {
                self.expression(left);
                self.expression(right);
            }
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::StorageSize(_)
            | ExpressionKind::Unit => {}
        }
    }

    fn body(&mut self, body: &LambdaBody) {
        self.items(&body.items);
        self.completion(&body.result);
    }

    fn block(&mut self, block: &ExpressionBlock) {
        self.items(&block.items);
        self.completion(&block.result);
    }

    fn items(&mut self, items: &[BodyItem]) {
        for item in items {
            match item {
                BodyItem::Binding(binding) => self.binding_value(binding),
                BodyItem::Expression(expression) => self.expression(expression),
            }
        }
    }

    fn completion(&mut self, completion: &Completion) {
        match completion {
            Completion::Value(value) => self.expression(value),
            Completion::Abrupt(abrupt) => {
                for value in &abrupt.preceding {
                    self.expression(value);
                }
                match &abrupt.kind {
                    AbruptExpressionKind::ResultTransfer { target, value } => {
                        self.value(*target);
                        self.expression(value);
                    }
                    AbruptExpressionKind::EmptyElimination { scrutinee } => {
                        self.expression(scrutinee);
                    }
                    AbruptExpressionKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    } => {
                        self.expression(condition);
                        self.block(then_branch);
                        self.block(else_branch);
                    }
                    AbruptExpressionKind::Block(block) => self.block(block),
                }
            }
        }
    }
}
