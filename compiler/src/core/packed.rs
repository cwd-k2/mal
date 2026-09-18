use crate::check::ast as checked;
use crate::resolve::ast::LambdaId;

use super::Lowerer;
use super::ast::{
    Binding, Capture, Expression, ExpressionKind, Lambda, PackedBuilderOperation, Parameter,
    Pattern, ValueId,
};

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

    fn capabilities(
        &mut self,
        builder: ValueId,
        element: &checked::Type,
        span: crate::source::Span,
    ) -> Expression {
        let new = self.capability(
            builder,
            element.clone(),
            checked::Type::USize,
            PackedBuilderOperation::New,
            element,
            span,
        );
        let get = self.capability(
            builder,
            checked::Type::USize,
            element.clone(),
            PackedBuilderOperation::Get,
            element,
            span,
        );
        let put_parameter =
            checked::Type::Product(vec![checked::Type::USize, element.clone()].into());
        let put = self.capability(
            builder,
            put_parameter,
            checked::Type::Unit,
            PackedBuilderOperation::Put,
            element,
            span,
        );
        let ty =
            checked::Type::Product(vec![new.ty.clone(), get.ty.clone(), put.ty.clone()].into());
        Expression {
            kind: ExpressionKind::Product(vec![new, get, put]),
            ty,
            span,
        }
    }

    fn capability(
        &mut self,
        builder: ValueId,
        parameter_type: checked::Type,
        result_type: checked::Type,
        operation: PackedBuilderOperation,
        element: &checked::Type,
        span: crate::source::Span,
    ) -> Expression {
        let capture = self.temporary();
        let parameter = self.temporary();
        let parameter_value = if parameter_type == checked::Type::Unit {
            self.unit(span)
        } else {
            self.reference(parameter, parameter_type.clone(), span)
        };
        let argument_type =
            checked::Type::Product(vec![checked::Type::Address, parameter_type.clone()].into());
        let argument = Expression {
            kind: ExpressionKind::Product(vec![
                self.reference(capture, checked::Type::Address, span),
                parameter_value,
            ]),
            ty: argument_type,
            span,
        };
        let body = Expression {
            kind: ExpressionKind::PackedBuilder {
                operation,
                element: element.clone(),
                argument: Box::new(argument),
            },
            ty: result_type.clone(),
            span,
        };
        let function_type = checked::Type::Function {
            parameter: parameter_type.clone().into(),
            result: result_type.into(),
        };
        let id = LambdaId(self.next_lambda);
        self.next_lambda = self
            .next_lambda
            .checked_add(1)
            .expect("specialization reserved lambda identity space");
        Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id,
                self_binding: None,
                captures: vec![Capture {
                    source: builder,
                    binding: capture,
                    ty: checked::Type::Address,
                }],
                parameter: Parameter {
                    binding: (parameter_type != checked::Type::Unit).then_some(parameter),
                    ty: parameter_type,
                    span,
                },
                body: Box::new(body),
                joins: Vec::new(),
            }),
            ty: function_type,
            span,
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
