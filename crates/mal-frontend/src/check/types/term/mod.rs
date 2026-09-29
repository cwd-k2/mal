use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Kind, Type, type_name};

mod indices;
mod kind;
mod kind_substitution;
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

pub(super) fn normalize_argument_kinds(
    parameters: &[Kind],
    arguments: &mut [Type],
    span: Span,
) -> Result<(), Diagnostic> {
    let mut substitutions = HashMap::new();
    for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
        kind::unify(parameter, &argument.kind(), &mut substitutions, span)?;
    }
    let mut budget = normalization::Budget::new(span);
    for argument in arguments.iter_mut() {
        *argument = kind_substitution::rewrite(argument, &substitutions, &mut budget, 0)?;
    }
    kind_substitution::canonicalize_variables(arguments, &mut budget)?;
    Ok(())
}

pub(in crate::check) fn instantiate_kinds(
    parameters: &[Kind],
    arguments: &[Type],
    ty: &Type,
    span: Span,
) -> Result<(Vec<Kind>, Type), Diagnostic> {
    let mut substitutions = HashMap::new();
    for (parameter, argument) in parameters.iter().zip(arguments) {
        kind::unify(parameter, &argument.kind(), &mut substitutions, span)?;
    }
    let mut budget = normalization::Budget::new(span);
    Ok((
        parameters
            .iter()
            .map(|parameter| kind::substitute(parameter, &substitutions))
            .collect(),
        kind_substitution::rewrite(ty, &substitutions, &mut budget, 0)?,
    ))
}

pub(super) use kind::name as kind_name;
