//! Termination of requirement expansion: each operation goal of a generic implementation body names a proper
//! subterm of the key, or the key itself for direct recursion.

use crate::resolve::ast as resolved;
use mal_syntax::diagnostic::Diagnostic;

use super::super::CheckResult;
use super::super::ast::{self, Type};

/// Requires each operation goal of a generic implementation body to name a proper subterm of the key, or the key
/// itself for direct recursion, so that expanding requirements terminates.
pub(super) fn require_decreasing(
    family: &resolved::ValueReference,
    arguments: &[Type],
    operations: &[ast::OperationRequirement],
) -> CheckResult<()> {
    for requirement in operations {
        let direct_self = requirement.family.id == family.id && requirement.arguments == arguments;
        if !direct_self && !decreases(arguments, &requirement.arguments) {
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

fn decreases(pattern: &[Type], requirement: &[Type]) -> bool {
    requirement.iter().all(|required| {
        pattern
            .iter()
            .any(|root| proper_type_subterm(root, required))
    })
}

fn proper_type_subterm(root: &Type, required: &Type) -> bool {
    let mut pending = Vec::new();
    extend_children(root, &mut pending);
    while let Some(candidate) = pending.pop() {
        if candidate == required {
            return true;
        }
        extend_children(candidate, &mut pending);
    }
    false
}

fn extend_children<'a>(ty: &'a Type, pending: &mut Vec<&'a Type>) {
    match ty {
        Type::Abstraction { body, .. } => pending.push(body),
        Type::Buffer(element) => pending.push(element),
        Type::Opaque { arguments, .. } => pending.extend(arguments.iter()),
        Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
        Type::Function { parameter, result } => {
            pending.push(parameter);
            pending.push(result);
        }
        _ => {}
    }
}
