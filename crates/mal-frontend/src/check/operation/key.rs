//! The shape and coherence of one implementation key.

use crate::resolve::ast as resolved;
use mal_syntax::diagnostic::Diagnostic;

use super::super::ast::{Kind, Type};
use super::super::{CheckResult, Checker};
use super::overlap::operation_patterns_overlap;
use super::pattern::{contains_parameter, contains_parameter_id, has_nominal_head};

impl Checker {
    /// Rejects a key that specialization could not select from deterministically: a constructor position
    /// without a nominal head, open types in an exact key, a catch-all or partially bound pattern, or an overlap
    /// with an earlier key of the same family.
    pub(super) fn check_implementation_key(
        &self,
        family: &resolved::ValueReference,
        parameters: &[resolved::TypeBinding],
        parameter_kinds: &[Kind],
        arguments: &[Type],
    ) -> CheckResult<()> {
        let span = family.name.span;
        if parameter_kinds
            .iter()
            .zip(arguments)
            .any(|(kind, argument)| {
                matches!(kind, Kind::Function { .. })
                    && contains_parameter(argument)
                    && !has_nominal_head(argument)
            })
        {
            return Err(
                Diagnostic::error("operation constructor key needs a nominal head")
                    .with_primary(
                        span,
                        "apply an opaque type or Buffer here; only its arguments may be type variables",
                    )
                    .into(),
            );
        }
        if parameters.is_empty() && arguments.iter().any(contains_parameter) {
            return Err(
                Diagnostic::error("exact operation implementation requires closed types")
                    .with_primary(span, "remove generic parameters from this key")
                    .into(),
            );
        }
        if !parameters.is_empty()
            && arguments
                .iter()
                .all(|argument| matches!(argument, Type::Parameter { .. }))
        {
            let names = parameters
                .iter()
                .map(|parameter| format!("`{}`", parameter.name.text))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(
                Diagnostic::error("generic operation implementation requires structure")
                    .with_primary(span, "a catch-all parameter key is not supported")
                    .with_note(if parameters.len() == 1 {
                        format!(
                            "{names} names no visible type, so the key reads it as a type variable"
                        )
                    } else {
                        format!(
                            "{names} name no visible types, so the key reads them as type variables"
                        )
                    })
                    .into(),
            );
        }
        if let Some(parameter) = parameters.iter().find(|parameter| {
            !arguments
                .iter()
                .any(|argument| contains_parameter_id(argument, parameter.id))
        }) {
            return Err(
                Diagnostic::error("generic operation pattern leaves a parameter unbound")
                    .with_primary(
                        parameter.name.span,
                        "use this family parameter in the implementation key",
                    )
                    .into(),
            );
        }
        if self.operation_keys.iter().any(|(id, existing)| {
            *id == family.id && operation_patterns_overlap(existing, arguments)
        }) {
            return Err(Diagnostic::error("duplicate operation implementation")
                .with_primary(span, "this family key overlaps an existing implementation")
                .into());
        }
        Ok(())
    }
}
