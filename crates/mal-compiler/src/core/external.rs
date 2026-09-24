use crate::check::ast as checked;
use crate::source::Span;

use super::Lowerer;
use super::ast::{Expression, ExpressionKind, Lambda, Parameter, TopLevelBinding, ValueId};

impl Lowerer {
    pub(super) fn lower_external_operation(
        &mut self,
        id: crate::resolve::ast::ExternalOperationId,
        binding: &crate::resolve::ast::ValueBinding,
        lambda_id: crate::resolve::ast::LambdaId,
        parameter: &checked::Type,
        result: &checked::Type,
        span: Span,
    ) -> TopLevelBinding {
        let parameter_binding = (parameter != &checked::Type::Unit).then(|| self.temporary());
        let argument = parameter_binding.map_or(
            Expression {
                kind: ExpressionKind::Unit,
                ty: checked::Type::Unit,
                span,
            },
            |parameter_binding| self.reference(parameter_binding, parameter.clone(), span),
        );
        let function_type = checked::Type::Function {
            parameter: parameter.clone().into(),
            result: result.clone().into(),
        };
        TopLevelBinding {
            pattern: super::ast::TopLevelPattern::Binding {
                id: ValueId::Source(binding.id),
                name: binding.name.text.clone(),
                ty: function_type.clone(),
            },
            value: Expression {
                kind: ExpressionKind::Lambda(Lambda {
                    id: lambda_id,
                    self_binding: None,
                    kind: super::ast::LambdaKind::Ordinary,
                    captures: Vec::new(),
                    parameter: Parameter {
                        binding: parameter_binding,
                        ty: parameter.clone(),
                        span,
                    },
                    body: Box::new(Expression {
                        kind: ExpressionKind::ExternalCall {
                            id,
                            argument: Box::new(argument),
                        },
                        ty: result.clone(),
                        span,
                    }),
                    joins: Vec::new(),
                }),
                ty: function_type,
                span,
            },
            span,
        }
    }
}
