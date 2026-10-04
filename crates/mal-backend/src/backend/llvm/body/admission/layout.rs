//! Admission of host memory layouts and runtime storage against the target.

use super::*;

pub(super) fn admit_external_storage(
    program: &execution::Program,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    for external in &program.lowered.interface.externals {
        for ty in [&external.parameter, &external.result] {
            admit_runtime_storage(ty, external.span, types, maximum, "external value")?;
        }
    }
    Ok(())
}

pub(super) fn admit_runtime_storage(
    ty: &Type,
    span: mal_syntax::source::Span,
    types: &Types,
    maximum: u128,
    description: &str,
) -> Result<(), Diagnostic> {
    let Some(value) = types.value(ty) else {
        return Ok(());
    };
    if value.size as u128 <= maximum {
        return Ok(());
    }
    Err(
        Diagnostic::error("runtime value layout is not representable for the target").with_primary(
            span,
            format!(
                "this {description} needs {} bytes, exceeding the target maximum {maximum}",
                value.size
            ),
        ),
    )
}
