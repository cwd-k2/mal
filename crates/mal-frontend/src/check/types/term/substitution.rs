use mal_syntax::diagnostic::Diagnostic;

use super::{
    Type,
    indices::{map_children_bounded, shift_bounded},
    normalization::Budget,
};

pub(super) fn bound(
    ty: &Type,
    argument: &Type,
    binder_depth: usize,
    budget: &mut Budget,
    traversal_depth: usize,
) -> Result<Type, Diagnostic> {
    budget.visit(traversal_depth)?;
    Ok(match ty {
        Type::Bound { index, .. } if *index == binder_depth => shift_bounded(
            argument,
            0,
            binder_depth as isize,
            budget,
            traversal_depth + 1,
        )?,
        Type::Bound { index, kind } if *index > binder_depth => Type::Bound {
            index: index - 1,
            kind: kind.clone(),
        },
        Type::Application {
            constructor,
            argument: applied,
            span,
            ..
        } => {
            let constructor = bound(
                constructor,
                argument,
                binder_depth,
                budget,
                traversal_depth + 1,
            )?;
            let applied = bound(applied, argument, binder_depth, budget, traversal_depth + 1)?;
            super::apply_with_budget(constructor, applied, *span, budget, traversal_depth + 1)?
        }
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: parameter_kind.clone(),
            body: bound(
                body,
                argument,
                binder_depth + 1,
                budget,
                traversal_depth + 1,
            )?
            .into(),
        },
        _ => map_children_bounded(ty, |child| {
            bound(child, argument, binder_depth, budget, traversal_depth + 1)
        })?,
    })
}
