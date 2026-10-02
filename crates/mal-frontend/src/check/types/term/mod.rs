use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Kind, Type, type_name};

mod argument_kinds;
mod indices;
mod kind;
mod kind_substitution;
mod kind_variables;
mod normalization;
mod substitution;

pub(in crate::check) struct Normalizer {
    budget: normalization::Budget,
}

impl Normalizer {
    pub(in crate::check) fn new(span: Span) -> Self {
        Self {
            budget: normalization::Budget::new(span),
        }
    }

    pub(in crate::check) fn apply(
        &mut self,
        constructor: Type,
        argument: Type,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        apply_with_budget(constructor, argument, span, &mut self.budget, 0)
    }

    /// Applies an abstraction to an argument its body never uses, so the argument is never formed. The caller knows the
    /// parameter is unused from its declaration; anything else is rejected as an application of a complete type.
    pub(in crate::check) fn apply_unused(
        &mut self,
        constructor: Type,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        match constructor {
            Type::Abstraction { body, .. } => {
                indices::shift_bounded(&body, 0, -1, &mut self.budget, 0)
            }
            constructor => Err(
                Diagnostic::error("type does not accept arguments").with_primary(
                    span,
                    format!("`{}` has kind `Type`", type_name(&constructor)),
                ),
            ),
        }
    }

    pub(in crate::check) fn abstraction(
        &mut self,
        parameter_kind: Kind,
        body: Type,
    ) -> Result<Type, Diagnostic> {
        abstraction_with_budget(parameter_kind, body, &mut self.budget, 0)
    }
}

fn apply_with_budget(
    constructor: Type,
    argument: Type,
    span: Span,
    budget: &mut normalization::Budget,
    depth: usize,
) -> Result<Type, Diagnostic> {
    budget.visit(depth)?;
    let Kind::Function { parameter, result } = constructor.kind() else {
        return Err(
            Diagnostic::error("type does not accept arguments").with_primary(
                span,
                format!("`{}` has kind `Type`", type_name(&constructor)),
            ),
        );
    };
    let mut kinds = HashMap::new();
    kind::unify(&parameter, &argument.kind(), &mut kinds, span)?;
    let constructor = kind_substitution::rewrite(&constructor, &kinds, budget, depth + 1)?;
    let argument = kind_substitution::rewrite(&argument, &kinds, budget, depth + 1)?;
    let applied = match constructor {
        Type::Abstraction { body, .. } => {
            substitution::bound(&body, &argument, 0, budget, depth + 1)?
        }
        constructor => Type::Application {
            constructor: constructor.into(),
            argument: argument.into(),
            kind: kind::substitute(&result, &kinds),
            span,
        },
    };
    if let Type::Buffer(element) = &applied {
        super::ensure_buffer_storable(element, span)?;
    }
    Ok(applied)
}

/// Rebuilds `ty` with each child replaced by `child(child, under_binder, budget)`, through the canonical constructors:
/// an application whose new constructor is an abstraction reduces, and an abstraction whose new body is an eta-redex
/// contracts. A traversal that can remove occurrences of a bound variable, such as substitution, rebuilds through here.
/// Traversals that keep every occurrence and every node shape, such as shifting and kind rewriting, cannot create a
/// redex and rebuild nodes directly.
fn rebuild_canonical(
    ty: &Type,
    budget: &mut normalization::Budget,
    depth: usize,
    mut child: impl FnMut(&Type, bool, &mut normalization::Budget) -> Result<Type, Diagnostic>,
) -> Result<Type, Diagnostic> {
    match ty {
        Type::Application {
            constructor,
            argument,
            span,
            ..
        } => {
            let constructor = child(constructor, false, budget)?;
            let argument = child(argument, false, budget)?;
            apply_with_budget(constructor, argument, *span, budget, depth)
        }
        Type::Abstraction {
            parameter_kind,
            body,
        } => {
            let body = child(body, true, budget)?;
            abstraction_with_budget(parameter_kind.clone(), body, budget, depth)
        }
        _ => indices::map_children_bounded(ty, |inner| child(inner, false, budget)),
    }
}

fn abstraction_with_budget(
    parameter_kind: Kind,
    body: Type,
    budget: &mut normalization::Budget,
    depth: usize,
) -> Result<Type, Diagnostic> {
    if let Type::Application {
        constructor,
        argument,
        ..
    } = &body
        && matches!(argument.as_ref(), Type::Bound { index: 0, kind } if kind == &parameter_kind)
        && !indices::contains_bound_bounded(constructor, 0, budget, depth + 1)?
    {
        return indices::shift_bounded(constructor, 0, -1, budget, depth + 1);
    }
    budget.visit(depth)?;
    Ok(Type::Abstraction {
        parameter_kind,
        body: body.into(),
    })
}

pub(in crate::check::types) fn freshen_kind_variables(
    ty: &Type,
    next: &mut u32,
    normalizer: &mut Normalizer,
) -> Result<Type, Diagnostic> {
    kind_substitution::freshen_variables(ty, next, &mut normalizer.budget)
}

pub(super) use kind::name as kind_name;

pub(super) use argument_kinds::normalize_argument_kinds;
pub(in crate::check) use argument_kinds::{
    check_kind_requirements, instantiate_kinds, kinds_unify,
};
