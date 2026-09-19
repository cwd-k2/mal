use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, PackedBuilderOperation, Pattern, ValueId};
use crate::check::ast as checked;

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

        let buffer_type = checked::Type::Buffer(element.clone().into());
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
            ty: buffer_type.clone(),
            span: expression.span,
        };
        let callback_call = Expression {
            kind: ExpressionKind::Call {
                callee: Box::new(self.reference(callback_id, callback.ty.clone(), callback.span)),
                argument: Box::new(self.reference(
                    builder_id,
                    buffer_type.clone(),
                    expression.span,
                )),
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
                    buffer_type.clone(),
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
        let callback_and_finish = if source.is_some() {
            let prepare = Expression {
                kind: ExpressionKind::PackedBuilder {
                    operation: PackedBuilderOperation::Prepare,
                    element: element.clone(),
                    argument: Box::new(self.reference(
                        builder_id,
                        buffer_type.clone(),
                        expression.span,
                    )),
                },
                ty: checked::Type::Unit,
                span: expression.span,
            };
            self.let_expression(
                Pattern::Wildcard {
                    ty: checked::Type::Unit,
                    span: expression.span,
                },
                prepare,
                after_callback,
                expression.span,
            )
        } else {
            after_callback
        };
        let after_builder = self.let_expression(
            Pattern::Binding {
                id: builder_id,
                ty: buffer_type,
            },
            builder,
            callback_and_finish,
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

    pub(super) fn lower_buffer_operation(
        &mut self,
        primitive: checked::MemoryPrimitive,
        operands: &[checked::Expression],
        expression: &checked::Expression,
    ) -> Expression {
        let [buffer, rest @ ..] = operands else {
            unreachable!("checked Buffer operation has a receiver")
        };
        let checked::Type::Buffer(element) = &buffer.ty else {
            unreachable!("checked Buffer operation has a Buffer receiver")
        };
        let buffer = self.lower_expression(buffer);
        let (operation, argument) = match (primitive, rest) {
            (checked::MemoryPrimitive::BufferNew, [value]) => {
                let value = self.lower_expression(value);
                (
                    PackedBuilderOperation::New,
                    self.product(vec![buffer, value], expression.span),
                )
            }
            (checked::MemoryPrimitive::BufferGet, [index]) => {
                let index = self.lower_expression(index);
                (
                    PackedBuilderOperation::Get,
                    self.product(vec![buffer, index], expression.span),
                )
            }
            (checked::MemoryPrimitive::BufferPut, [index, value]) => {
                let index = self.lower_expression(index);
                let value = self.lower_expression(value);
                let put = self.product(vec![index, value], expression.span);
                (
                    PackedBuilderOperation::Put,
                    self.product(vec![buffer, put], expression.span),
                )
            }
            _ => unreachable!("checked Buffer operation has valid operands"),
        };
        Expression {
            kind: ExpressionKind::PackedBuilder {
                operation,
                element: element.as_ref().clone(),
                argument: Box::new(argument),
            },
            ty: expression.ty.clone(),
            span: expression.span,
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

    fn product(&self, elements: Vec<Expression>, span: crate::source::Span) -> Expression {
        Expression {
            ty: checked::Type::Product(
                elements
                    .iter()
                    .map(|element| element.ty.clone())
                    .collect::<Vec<_>>()
                    .into(),
            ),
            kind: ExpressionKind::Product(elements),
            span,
        }
    }

    fn reference_from_expression(&self, id: ValueId, value: &Expression) -> Expression {
        self.reference(id, value.ty.clone(), value.span)
    }
}
