use mal_syntax::diagnostic::Diagnostic;

use super::{Type, indices::shift_bounded, normalization::Budget};

/// Replaces bound variable `binder_depth` with `argument`, renumbering the variables bound outside it. Removing a
/// variable can expose a redex in any enclosing node, so the term is rebuilt through the canonical constructors.
pub(super) fn bound(
    ty: &Type,
    argument: &Type,
    binder_depth: usize,
    budget: &mut Budget,
    traversal_depth: usize,
) -> Result<Type, Diagnostic> {
    budget.visit(traversal_depth)?;
    match ty {
        Type::Bound { index, .. } if *index == binder_depth => shift_bounded(
            argument,
            0,
            binder_depth as isize,
            budget,
            traversal_depth + 1,
        ),
        Type::Bound { index, kind } if *index > binder_depth => Ok(Type::Bound {
            index: index - 1,
            kind: kind.clone(),
        }),
        _ => super::rebuild_canonical(
            ty,
            budget,
            traversal_depth + 1,
            |child, under_binder, budget| {
                bound(
                    child,
                    argument,
                    binder_depth + usize::from(under_binder),
                    budget,
                    traversal_depth + 1,
                )
            },
        ),
    }
}
