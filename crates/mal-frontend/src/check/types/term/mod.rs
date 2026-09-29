use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Kind, Type, type_name};

mod kind;
mod substitution;

pub(in crate::check) fn apply(
    constructor: Type,
    argument: Type,
    span: Span,
) -> Result<Type, Diagnostic> {
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
    let constructor = substitution::kinds(&constructor, &kinds);
    let argument = substitution::kinds(&argument, &kinds);
    let applied = match constructor {
        Type::Abstraction { body, .. } => substitution::bound(&body, &argument, 0),
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

pub(in crate::check) fn abstraction(parameter_kind: Kind, body: Type) -> Type {
    if let Type::Application {
        constructor,
        argument,
        ..
    } = &body
        && matches!(argument.as_ref(), Type::Bound { index: 0, kind } if kind == &parameter_kind)
        && !substitution::contains_bound(constructor, 0)
    {
        return substitution::shift(constructor, 0, -1);
    }
    Type::Abstraction {
        parameter_kind,
        body: body.into(),
    }
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
    for argument in arguments.iter_mut() {
        *argument = substitution::kinds(argument, &substitutions);
    }
    substitution::canonicalize_kind_variables(arguments);
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
    Ok((
        parameters
            .iter()
            .map(|parameter| kind::substitute(parameter, &substitutions))
            .collect(),
        substitution::kinds(ty, &substitutions),
    ))
}

pub(super) use kind::name as kind_name;
