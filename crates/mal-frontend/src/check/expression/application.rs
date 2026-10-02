use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{Expression, ExpressionKind, Type};
use super::super::types::type_name;
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_call(
        &mut self,
        callee: &Node<resolved::Expression>,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if let Some((reference, type_arguments)) = referenced_generic_value(callee)
            && matches!(
                reference.id,
                crate::resolve::MAKE_VALUE | crate::resolve::FROM_VALUE
            )
        {
            return self.check_memory_intrinsic(reference, type_arguments, arguments, span);
        }
        if let Some((reference, type_arguments)) = referenced_generic_value(callee)
            && self.generic_signatures.contains_key(&reference.id)
        {
            return self.check_explicit_generic_call(
                reference,
                type_arguments,
                arguments,
                span,
                expected,
            );
        }
        if let Some(reference) = referenced_value(callee)
            && matches!(
                reference.id,
                crate::resolve::MAKE_VALUE | crate::resolve::FROM_VALUE
            )
        {
            return self.check_inferred_memory_intrinsic(reference, arguments, span, expected);
        }
        if let Some(reference) = referenced_value(callee)
            && self.generic_signatures.contains_key(&reference.id)
        {
            return self.check_inferred_generic_call(reference, arguments, span, expected);
        }
        if let Some(reference) = referenced_value(callee)
            && matches!(
                reference.id,
                crate::resolve::NEW_VALUE
                    | crate::resolve::GET_VALUE
                    | crate::resolve::PUT_VALUE
                    | crate::resolve::FILL_VALUE
                    | crate::resolve::COPY_VALUE
                    | crate::resolve::INTO_VALUE
            )
        {
            return self.check_memory_operation(reference, arguments, span);
        }
        if let Some(reference) = referenced_value(callee)
            && let Some(target) = self.result_targets.get(&reference.id).cloned()
        {
            let argument = self.check_argument(arguments, &target.parameter, span)?;
            return self.result_transfer(target, argument, span);
        }
        let callee = match self.check_expression(callee, None) {
            Ok(callee) => callee,
            Err(CheckFailure::Abrupt(abrupt)) => {
                let argument = self.check_untyped_argument(arguments, span)?;
                return Err(CheckFailure::Abrupt(Box::new(
                    (*abrupt).preceded_by(vec![argument]),
                )));
            }
            Err(error) => return Err(error),
        };
        let viewed = super::super::types::representation_view(&callee.ty, callee.span.file());
        let Type::Function { parameter, result } = viewed else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(
                    callee.span,
                    format!("this has type `{}`", type_name(&callee.ty)),
                )
                .into());
        };
        let argument = match self.check_argument(arguments, parameter, span) {
            Ok(argument) => argument,
            Err(CheckFailure::Abrupt(_)) => {
                return Err(Diagnostic::error(
                    "function value is unreachable after abrupt argument evaluation",
                )
                .with_primary(callee.span, "this callee cannot be evaluated")
                .into());
            }
            Err(error) => return Err(error),
        };
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    pub(crate) fn check_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        parameter: &Type,
        span: Span,
    ) -> CheckResult<Expression> {
        match arguments {
            [] => {
                self.require_type(&Type::Unit, parameter, span)?;
                Ok(Expression {
                    kind: ExpressionKind::Unit,
                    ty: Type::Unit,
                    span,
                })
            }
            [argument] => self.check_expression(argument, Some(parameter)),
            _ => {
                let argument = self.check_product(arguments, span, Some(parameter))?;
                self.require_type(&argument.ty, parameter, argument.span)?;
                Ok(argument)
            }
        }
    }

    pub(in crate::check) fn check_untyped_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        match arguments {
            [] => Ok(Expression {
                kind: ExpressionKind::Unit,
                ty: Type::Unit,
                span,
            }),
            [argument] => self.check_expression(argument, None),
            _ => self.check_product(arguments, span, None),
        }
    }
}

pub(super) fn referenced_value(
    expression: &Node<resolved::Expression>,
) -> Option<&resolved::ValueReference> {
    match &unparenthesized(expression).kind {
        resolved::Expression::Reference(reference) => Some(reference),
        _ => None,
    }
}

fn referenced_generic_value(
    expression: &Node<resolved::Expression>,
) -> Option<(&resolved::ValueReference, &[Node<resolved::TypeExpression>])> {
    match &unparenthesized(expression).kind {
        resolved::Expression::GenericReference {
            reference,
            arguments,
        } => Some((reference, arguments)),
        _ => None,
    }
}

pub(super) fn unparenthesized(
    mut expression: &Node<resolved::Expression>,
) -> &Node<resolved::Expression> {
    while let resolved::Expression::Parenthesized(inner) = &expression.kind {
        expression = inner;
    }
    expression
}
