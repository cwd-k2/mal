use crate::check::ast as checked;
use crate::resolve::ast::LambdaId;

use super::super::Lowerer;
use super::super::ast::{
    Capture, Expression, ExpressionKind, Lambda, LambdaKind, PackedBuilderOperation, Parameter,
    ValueId,
};

impl Lowerer {
    pub(super) fn capabilities(
        &mut self,
        builder: ValueId,
        element: &checked::Type,
        unique: bool,
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
            if unique {
                PackedBuilderOperation::PutUnique
            } else {
                PackedBuilderOperation::Put
            },
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
        let id = self.capability_id(element, operation);
        Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id,
                self_binding: None,
                kind: LambdaKind::PackedCapability {
                    operation,
                    element: element.clone(),
                },
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

    fn capability_id(
        &mut self,
        element: &checked::Type,
        operation: PackedBuilderOperation,
    ) -> LambdaId {
        if let Some((_, _, id)) =
            self.packed_capabilities
                .iter()
                .find(|(candidate, candidate_operation, _)| {
                    candidate == element && *candidate_operation == operation
                })
        {
            return *id;
        }
        let id = LambdaId(self.next_lambda);
        self.next_lambda = self
            .next_lambda
            .checked_add(1)
            .expect("specialization reserved lambda identity space");
        self.packed_capabilities
            .push((element.clone(), operation, id));
        id
    }
}
