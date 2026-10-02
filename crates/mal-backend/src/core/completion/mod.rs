//! Completion-aware lowering of body sequences and control paths to lexical joins.

use mal_frontend::check::ast as checked;
use mal_syntax::ast::Node;

use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern};

type Continuation<'a> = dyn FnMut(&mut Lowerer, Expression) -> Expression + 'a;

mod abrupt;
mod branch;
mod presence;
mod result_block;
mod value;

use presence::contains_control;

impl Lowerer {
    fn lower_with_join(
        &mut self,
        parameter_type: &checked::Type,
        result_type: &checked::Type,
        span: mal_syntax::source::Span,
        continuation: &mut Continuation<'_>,
        build: impl FnOnce(&mut Lowerer, &mut Continuation<'_>) -> Expression,
    ) -> Expression {
        let parameter = self.temporary();
        let argument = self.reference(parameter, parameter_type.clone(), span);
        let body = continuation(self, argument);
        let target = super::ast::JoinId(self.joins.len());
        self.joins.push(super::ast::Join {
            parameter: Pattern::Binding {
                id: parameter,
                ty: parameter_type.clone(),
            },
            body,
            span,
        });
        let mut jump = |_: &mut Lowerer, value: Expression| Expression {
            kind: ExpressionKind::Goto {
                target,
                value: Box::new(value),
            },
            ty: result_type.clone(),
            span,
        };
        build(self, &mut jump)
    }

    pub(super) fn lower_lambda_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        result_type: &checked::Type,
    ) -> Expression {
        if let checked::Completion::Value(value) = result
            && !contains_control(value)
            && !items.iter().any(|item| match item {
                checked::BodyItem::Binding(binding) => contains_control(&binding.value),
                checked::BodyItem::Expression(value) => contains_control(value),
            })
        {
            return self.lower_body(items, result);
        }
        let mut identity = |_: &mut Lowerer, value: Expression| value;
        self.lower_items_with(items, result, result_type, &mut identity)
    }

    fn lower_items_with(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
    ) -> Expression {
        let mut body = self.lower_completion_with(result, result_type, continuation);
        for item in items.iter().rev() {
            if !contains_control(body_item_value(item)) {
                body = self.prepend_body_item(item, body);
                continue;
            }
            let mut rest = Some(body);
            let mut next = |lowerer: &mut Lowerer, value: Expression| {
                lowerer.prepend_lowered_body_item(
                    item,
                    value,
                    rest.take().expect("a join continuation is lowered once"),
                )
            };
            body = self.lower_value_with(body_item_value(item), result_type, &mut next);
        }
        body
    }

    fn prepend_body_item(&mut self, item: &checked::BodyItem, body: Expression) -> Expression {
        let (pattern, value, span) = match item {
            checked::BodyItem::Binding(binding) => (
                self.lower_pattern(&binding.pattern),
                self.lower_expression(&binding.value),
                binding.span,
            ),
            checked::BodyItem::Expression(value) => {
                let lowered = self.lower_expression(value);
                (
                    Pattern::Wildcard {
                        ty: lowered.ty.clone(),
                        span: lowered.span,
                    },
                    lowered,
                    value.span,
                )
            }
        };
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern,
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty,
            span,
        }
    }

    fn prepend_lowered_body_item(
        &mut self,
        item: &checked::BodyItem,
        value: Expression,
        body: Expression,
    ) -> Expression {
        let (pattern, span) = match item {
            checked::BodyItem::Binding(binding) => {
                (self.lower_pattern(&binding.pattern), binding.span)
            }
            checked::BodyItem::Expression(source) => (
                Pattern::Wildcard {
                    ty: value.ty.clone(),
                    span: value.span,
                },
                source.span,
            ),
        };
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern,
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty,
            span,
        }
    }

    fn lower_completion_with(
        &mut self,
        completion: &checked::Completion,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
    ) -> Expression {
        match completion {
            checked::Completion::Value(value) => {
                self.lower_value_with(value, result_type, continuation)
            }
            checked::Completion::Abrupt(abrupt) => self.lower_abrupt(abrupt, result_type),
        }
    }
}

fn body_item_value(item: &checked::BodyItem) -> &checked::Expression {
    match item {
        checked::BodyItem::Binding(binding) => &binding.value,
        checked::BodyItem::Expression(value) => value,
    }
}
