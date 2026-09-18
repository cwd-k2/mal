use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, PackedBuilderOperation, Pattern, ValueId};
use crate::check::ast as checked;

mod capability;

impl Lowerer {
    pub(super) fn lower_packed_build(
        &mut self,
        source: Option<&checked::Expression>,
        callback: &checked::Expression,
        element: &checked::Type,
        expression: &checked::Expression,
    ) -> Expression {
        let source = source.map(|source| self.lower_expression(source));
        let callback = self.lower_expression(callback);
        let callback_id = self.temporary();
        let builder_id = self.temporary();
        let source_id = source.as_ref().map(|_| self.temporary());
        let start_argument = match (&source, source_id) {
            (Some(source), Some(id)) => self.reference_from_expression(id, source),
            (None, None) => self.unit(expression.span),
            _ => unreachable!(),
        };

        let builder = Expression {
            kind: ExpressionKind::PackedBuilder {
                operation: if source.is_some() {
                    PackedBuilderOperation::Edit
                } else {
                    PackedBuilderOperation::Start
                },
                element: element.clone(),
                argument: Box::new(start_argument),
            },
            ty: checked::Type::Address,
            span: expression.span,
        };
        let capabilities = self.capabilities(builder_id, element, expression.span);
        let callback_call = Expression {
            kind: ExpressionKind::Call {
                callee: Box::new(self.reference(callback_id, callback.ty.clone(), callback.span)),
                argument: Box::new(capabilities),
            },
            ty: checked::Type::Unit,
            span: expression.span,
        };
        let finish = Expression {
            kind: ExpressionKind::PackedBuilder {
                operation: PackedBuilderOperation::Finish,
                element: element.clone(),
                argument: Box::new(self.reference(
                    builder_id,
                    checked::Type::Address,
                    expression.span,
                )),
            },
            ty: expression.ty.clone(),
            span: expression.span,
        };
        let after_callback = self.let_expression(
            Pattern::Wildcard {
                ty: checked::Type::Unit,
                span: expression.span,
            },
            callback_call,
            finish,
            expression.span,
        );
        let after_builder = self.let_expression(
            Pattern::Binding {
                id: builder_id,
                ty: checked::Type::Address,
            },
            builder,
            after_callback,
            expression.span,
        );
        let after_callback_value = self.let_expression(
            Pattern::Binding {
                id: callback_id,
                ty: callback.ty.clone(),
            },
            callback,
            after_builder,
            expression.span,
        );
        if let (Some(source), Some(source_id)) = (source, source_id) {
            self.let_expression(
                Pattern::Binding {
                    id: source_id,
                    ty: source.ty.clone(),
                },
                source,
                after_callback_value,
                expression.span,
            )
        } else {
            after_callback_value
        }
    }

    fn let_expression(
        &self,
        pattern: Pattern,
        value: Expression,
        body: Expression,
        span: crate::source::Span,
    ) -> Expression {
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

    fn unit(&self, span: crate::source::Span) -> Expression {
        Expression {
            kind: ExpressionKind::Unit,
            ty: checked::Type::Unit,
            span,
        }
    }

    fn reference_from_expression(&self, id: ValueId, value: &Expression) -> Expression {
        self.reference(id, value.ty.clone(), value.span)
    }
}
