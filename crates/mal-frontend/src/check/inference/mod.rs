//! Generic value reference and call checking from local type constraints.

use std::collections::HashMap;

use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{self, Expression, Type};
use super::{CheckResult, Checker, types};

enum GenericCallExpectation<'a> {
    Result(Option<&'a Type>),
    ReturnedFunctionParameter(&'a Type),
}

mod arguments;
mod call;
mod constraint;
#[cfg(test)]
mod constraint_tests;
mod memo;
mod probe;
mod reference;

pub(super) use memo::ArgumentMemo;

use arguments::{argument_templates, inferred_arguments, parameter_ids};
use constraint::constrain;

impl Checker {
    pub(super) fn check_inferred_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        if let Some(expected) = expected {
            constrain(
                &signature.ty,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?;
        }
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_explicit_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let mut signature = self.generic_signatures[&reference.id].clone();
        if type_arguments.len() > signature.parameters.len() {
            return Err(Diagnostic::error("generic value argument arity mismatch")
                .with_primary(
                    reference.name.span,
                    format!(
                        "expected at most {} arguments but found {}",
                        signature.parameters.len(),
                        type_arguments.len()
                    ),
                )
                .into());
        }
        let mut explicit = type_arguments
            .iter()
            .map(|argument| self.expand_type_term(argument))
            .collect::<Result<Vec<_>, _>>()?;
        let kinds = types::require_type_argument_kinds(
            &signature.parameter_kinds[..explicit.len()],
            &mut explicit,
            reference.name.span,
        )?;
        self.record_kinds(kinds);
        let kinds;
        (signature.parameter_kinds, signature.ty, kinds) = types::instantiate_signature_kinds(
            &signature.parameter_kinds,
            &explicit,
            &signature.ty,
            reference.name.span,
        )?;
        self.record_kinds(kinds);
        let mut substitutions = signature
            .parameters
            .iter()
            .zip(explicit)
            .map(|(parameter, argument)| (parameter.id, argument))
            .collect::<HashMap<_, _>>();
        if let Some(expected) = expected {
            let instantiated =
                types::substitute_type(&signature.ty, &substitutions, reference.name.span)?;
            constrain(
                &instantiated,
                expected,
                &parameter_ids(&signature),
                &mut substitutions,
                reference.name.span,
            )?;
        }
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_inferred_generic_continuation_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let Type::Function {
            parameter: parameter_template,
            ..
        } = &signature.ty
        else {
            return self.check_inferred_generic_reference(reference, span, None);
        };
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        constrain(
            parameter_template,
            parameter,
            &flexible,
            &mut substitutions,
            reference.name.span,
        )?;
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }
}
