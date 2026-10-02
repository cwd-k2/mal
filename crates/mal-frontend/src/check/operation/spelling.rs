//! The types a misspelled implementation key binder may have meant, added to failures inside the implementation.

use super::super::CheckFailure;

/// Adds to a failure inside an implementation the types its binders may have meant to name.
pub(in crate::check) fn suggest_types(
    failure: CheckFailure,
    similar_types: &[(String, String)],
) -> CheckFailure {
    let CheckFailure::Diagnostic(mut diagnostic) = failure else {
        return failure;
    };
    for (binder, ty) in similar_types {
        diagnostic = diagnostic.with_note(format!(
            "`{binder}` in the implementation key is a type variable; did you mean the type `{ty}`?"
        ));
    }
    CheckFailure::Diagnostic(diagnostic)
}
