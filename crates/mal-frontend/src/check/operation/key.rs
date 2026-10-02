//! Shape and coherence of one implementation key, and the decrease of the requirements its body leaves.

use crate::resolve::ast as resolved;
use mal_syntax::diagnostic::Diagnostic;

use super::super::ast::{self, Kind, Type};
use super::super::{CheckResult, Checker};
use super::overlap::operation_patterns_overlap;
use super::pattern::{
    contains_parameter, contains_parameter_id, has_nominal_head, operation_requirement_decreases,
};

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
            return Err(
                Diagnostic::error("generic operation implementation requires structure")
                    .with_primary(span, "a catch-all parameter key is not supported")
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

/// Requires each operation goal of a generic implementation body to name a proper subterm of the key, or the key
/// itself for direct recursion, so that expanding requirements terminates.
pub(super) fn require_decreasing(
    family: &resolved::ValueReference,
    arguments: &[Type],
    operations: &[ast::OperationRequirement],
) -> CheckResult<()> {
    for requirement in operations {
        let direct_self = requirement.family.id == family.id && requirement.arguments == arguments;
        if !direct_self && !operation_requirement_decreases(arguments, &requirement.arguments) {
            return Err(
                Diagnostic::error("generic operation requirement does not decrease")
                    .with_primary(
                        requirement.family.name.span,
                        "the required key must be a proper subterm of the implementation key",
                    )
                    .into(),
            );
        }
    }
    Ok(())
}
