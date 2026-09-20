use super::*;

impl Lowerer {
    pub(super) fn lower_lambda(&mut self, lambda: &checked::Lambda) -> Lambda {
        let outer_joins = std::mem::take(&mut self.joins);
        let parameter_type = lambda.parameter_type.clone();
        let parameter_binding = match lambda.parameter.as_deref() {
            None | Some(checked::Pattern::Wildcard { .. }) => None,
            Some(checked::Pattern::Binding { binding, .. }) => Some(ValueId::Source(binding.id)),
            Some(checked::Pattern::Product { .. }) => Some(self.temporary()),
        };
        let mut body =
            self.lower_lambda_body(&lambda.body.items, &lambda.body.result, &lambda.result_type);
        if let Some(pattern @ checked::Pattern::Product { .. }) = lambda.parameter.as_deref() {
            let parameter_id = parameter_binding.expect("a product pattern uses a product value");
            let destructuring = Binding {
                pattern: self.lower_pattern(pattern),
                value: self.reference(parameter_id, parameter_type.clone(), lambda.body.span),
                span: lambda.body.span,
            };
            let result_type = body.ty.clone();
            body = Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(destructuring),
                    body: Box::new(body),
                },
                ty: result_type,
                span: lambda.body.span,
            };
        }
        let joins = std::mem::replace(&mut self.joins, outer_joins);
        Lambda {
            id: lambda.id,
            self_binding: lambda.self_binding.map(ValueId::Source),
            kind: LambdaKind::Ordinary,
            captures: lambda
                .captures
                .iter()
                .map(|capture| Capture {
                    source: ValueId::Source(capture.source.id),
                    binding: ValueId::Source(capture.binding.id),
                    ty: capture.ty.clone(),
                })
                .collect(),
            parameter: Parameter {
                binding: parameter_binding,
                ty: parameter_type,
                span: lambda.parameter.as_deref().map_or(
                    lambda.body.span,
                    |pattern| match pattern {
                        checked::Pattern::Binding { binding, .. } => binding.name.span,
                        checked::Pattern::Wildcard { span, .. }
                        | checked::Pattern::Product { span, .. } => *span,
                    },
                ),
            },
            body: Box::new(body),
            joins,
        }
    }

    pub(super) fn lower_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
    ) -> Expression {
        let checked::Completion::Value(result) = result else {
            unreachable!("direct lowering only receives value-completing blocks");
        };
        let mut body = self.lower_expression(result);
        for item in items.iter().rev() {
            let binding = match item {
                checked::BodyItem::Binding(binding) => self.lower_binding(binding),
                checked::BodyItem::Expression(value) => Binding {
                    pattern: Pattern::Wildcard {
                        ty: value.ty.clone(),
                        span: value.span,
                    },
                    value: self.lower_expression(value),
                    span: value.span,
                },
            };
            let span = Span::new(
                binding.span.file(),
                binding.span.start().min(body.span.start()),
                binding.span.end().max(body.span.end()),
            );
            let ty = body.ty.clone();
            body = Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(binding),
                    body: Box::new(body),
                },
                ty,
                span,
            };
        }
        body
    }
}
