//! Admission of host memory layouts and runtime storage against the target.

use super::*;

pub(super) fn admit_host_memory_layouts(
    program: &execution::Program,
    layouts: SourceLayouts,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    for alias in program
        .lowered
        .interface
        .type_aliases
        .iter()
        .filter(|alias| alias.host_memory_access)
    {
        admit_canonical_layout(&alias.ty, alias.span, layouts, maximum)?;
    }
    for external in &program.lowered.interface.externals {
        for ty in [&external.parameter, &external.result] {
            if let Some(layout) = layouts.layout(ty) {
                admit_canonical_stride(layout.stride, external.span, maximum)?;
            }
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

pub(super) fn admit_canonical_layout(
    ty: &Type,
    span: mal_syntax::source::Span,
    layouts: SourceLayouts,
    maximum: u128,
) -> Result<(), Diagnostic> {
    let Some(layout) = layouts.layout(ty) else {
        return Err(Diagnostic::error(
            "canonical memory layout is not representable for the target",
        )
        .with_primary(
            span,
            "this type's layout exceeds the target object-size range",
        ));
    };
    admit_canonical_stride(layout.stride, span, maximum)
}

fn admit_canonical_stride(
    stride: usize,
    span: mal_syntax::source::Span,
    maximum: u128,
) -> Result<(), Diagnostic> {
    if stride as u128 <= maximum {
        return Ok(());
    }
    Err(
        Diagnostic::error("canonical memory layout is not representable for the target")
            .with_primary(
                span,
                format!(
                    "this type has a {stride}-byte stride, exceeding the target maximum {maximum}"
                ),
            ),
    )
}
