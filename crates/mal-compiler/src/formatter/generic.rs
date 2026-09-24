use crate::ast::{BodyItem, Expression, Program, TopItem, TypeExpression};
use crate::lexer::{Lexed, TokenKind};

pub(super) fn delimiters(lexed: &Lexed, program: &Program) -> Vec<bool> {
    let mut marked = vec![false; lexed.tokens.len()];
    let mut marker = Marker {
        lexed,
        marked: &mut marked,
    };
    for item in &program.items {
        match &item.kind {
            TopItem::TypeAlias { value, .. } => marker.ty(value),
            TopItem::GenericTypeAlias { name, value, .. } => {
                marker.group(name.span.end(), value.span.start());
                marker.ty(value);
            }
            TopItem::ExternalOperation { ty, .. } => marker.ty(ty),
            TopItem::Binding(binding) => marker.binding(binding),
            TopItem::GenericBinding {
                name,
                annotation,
                value,
                ..
            } => {
                marker.group(name.span.end(), annotation.span.start());
                marker.ty(annotation);
                marker.expression(value);
            }
            TopItem::ExternalType { .. } => {}
        }
    }
    marked
}

struct Marker<'a> {
    lexed: &'a Lexed,
    marked: &'a mut [bool],
}

impl Marker<'_> {
    fn group(&mut self, start: usize, end: usize) {
        let first = self
            .lexed
            .tokens
            .partition_point(|token| token.span.end() <= start);
        let Some(open) = self.lexed.tokens[first..]
            .iter()
            .position(|token| token.span.start() < end && token.kind == TokenKind::Less)
            .map(|offset| first + offset)
        else {
            return;
        };
        let Some(close) = self.lexed.tokens[open + 1..]
            .iter()
            .take_while(|token| token.span.end() <= end)
            .enumerate()
            .filter(|(_, token)| matches!(token.kind, TokenKind::Greater | TokenKind::ShiftRight))
            .map(|(offset, _)| open + 1 + offset)
            .last()
        else {
            return;
        };
        self.marked[open] = true;
        self.marked[close] = true;
    }

    fn ty(&mut self, ty: &crate::ast::Node<TypeExpression>) {
        match &ty.kind {
            TypeExpression::Application {
                constructor,
                arguments,
            } => {
                self.group(constructor.span.end(), ty.span.end());
                for argument in arguments {
                    self.ty(argument);
                }
            }
            TypeExpression::Parenthesized(inner) => self.ty(inner),
            TypeExpression::Product(elements) | TypeExpression::Sum(elements) => {
                for element in elements {
                    self.ty(element);
                }
            }
            TypeExpression::Function { parameter, result } => {
                self.ty(parameter);
                self.ty(result);
            }
            TypeExpression::Named(_) | TypeExpression::Unit => {}
        }
    }

    fn binding(&mut self, binding: &crate::ast::Binding) {
        if let Some(annotation) = &binding.annotation {
            self.ty(annotation);
        }
        self.expression(&binding.value);
    }

    fn expression(&mut self, expression: &crate::ast::Node<Expression>) {
        let mut pending = vec![expression];
        while let Some(expression) = pending.pop() {
            match &expression.kind {
                Expression::GenericName { name, arguments } => {
                    self.group(name.span.end(), expression.span.end());
                    for argument in arguments {
                        self.ty(argument);
                    }
                }
                Expression::Parenthesized(inner) => pending.push(inner),
                Expression::Product(elements) => pending.extend(elements.iter().rev()),
                Expression::Block(block) | Expression::ResultBlock { body: block, .. } => {
                    self.body(&block.items, &block.result);
                }
                Expression::Lambda(lambda) => self.body(&lambda.body.items, &lambda.body.result),
                Expression::Call { callee, arguments } => {
                    pending.extend(arguments.iter().rev());
                    pending.push(callee);
                }
                Expression::ContinuationApplication {
                    value,
                    continuations,
                } => {
                    pending.extend(continuations.iter().rev());
                    pending.push(value);
                }
                Expression::Conversion { value, .. } | Expression::Unary { operand: value, .. } => {
                    pending.push(value)
                }
                Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.body(&else_branch.items, &else_branch.result);
                    self.body(&then_branch.items, &then_branch.result);
                    pending.push(condition);
                }
                Expression::When { condition, body } => {
                    self.body(&body.items, &body.result);
                    pending.push(condition);
                }
                Expression::Binary { left, right, .. } => {
                    pending.push(right);
                    pending.push(left);
                }
                Expression::Name(_)
                | Expression::Integer(_)
                | Expression::Float(_)
                | Expression::Byte(_)
                | Expression::Symbol(_)
                | Expression::Unit => {}
            }
        }
    }

    fn body(&mut self, items: &[BodyItem], result: &crate::ast::Node<Expression>) {
        for item in items {
            match item {
                BodyItem::Binding(binding) => self.binding(&binding.kind),
                BodyItem::Expression(expression) => self.expression(expression),
            }
        }
        self.expression(result);
    }
}
