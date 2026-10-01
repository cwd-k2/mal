//! Kinds of type arguments: unification with a signature's parameter kinds, the kind equations learned
//! about the enclosing binding's parameters, and their check once specialization makes them concrete.

use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Kind, Type, kind, kind_substitution, kind_variables, normalization};
use crate::check::ast::KindRequirement;

/// Unifies `parameters` with the kinds of `arguments`. Kind variables of type parameters mentioned by the
/// arguments belong to the enclosing binding: they are never rewritten, and what unification learns about
/// them is returned as equations for specialization to check.
fn unify_arguments(
    parameters: &[Kind],
    arguments: &[Type],
    span: Span,
) -> Result<(HashMap<u32, Kind>, Vec<KindRequirement>), Diagnostic> {
    let rigid = kind_variables::collect_parameters(arguments.iter());
    let mut substitutions = HashMap::new();
    for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
        kind::unify_retaining(
            parameter,
            &argument.kind(),
            &rigid,
            &mut substitutions,
            span,
        )?;
    }
    let mut variables = rigid.into_iter().collect::<Vec<_>>();
    variables.sort_unstable();
    let requirements = variables
        .into_iter()
        .filter_map(|variable| {
            let left = Kind::Variable(variable);
            let right = kind::substitute(&left, &substitutions);
            (right != left).then_some(KindRequirement { left, right, span })
        })
        .collect::<Vec<_>>();
    for requirement in &requirements {
        if let Kind::Variable(variable) = requirement.left {
            substitutions.remove(&variable);
        }
    }
    Ok((substitutions, requirements))
}

pub(in crate::check::types) fn normalize_argument_kinds(
    parameters: &[Kind],
    arguments: &mut [Type],
    span: Span,
) -> Result<Vec<KindRequirement>, Diagnostic> {
    let rigid = kind_variables::collect_parameters(arguments.iter());
    let (substitutions, requirements) = unify_arguments(parameters, arguments, span)?;
    let mut budget = normalization::Budget::new(span);
    for argument in arguments.iter_mut() {
        *argument = kind_substitution::rewrite(argument, &substitutions, &mut budget, 0)?;
    }
    kind_substitution::canonicalize_variables(arguments, &rigid, &mut budget)?;
    Ok(requirements)
}

pub(in crate::check) fn instantiate_kinds(
    parameters: &[Kind],
    arguments: &[Type],
    ty: &Type,
    span: Span,
) -> Result<(Vec<Kind>, Type, Vec<KindRequirement>), Diagnostic> {
    let (substitutions, requirements) = unify_arguments(parameters, arguments, span)?;
    let mut budget = normalization::Budget::new(span);
    Ok((
        parameters
            .iter()
            .map(|parameter| kind::substitute(parameter, &substitutions))
            .collect(),
        kind_substitution::rewrite(ty, &substitutions, &mut budget, 0)?,
        requirements,
    ))
}

/// Checks the kind equations of a generic body against concrete type arguments.
pub(in crate::check) fn check_kind_requirements(
    parameters: &[Kind],
    arguments: &[Type],
    requirements: &[KindRequirement],
    span: Span,
) -> Result<(), Diagnostic> {
    if requirements.is_empty() {
        return Ok(());
    }
    let mut substitutions = HashMap::new();
    for (parameter, argument) in parameters.iter().zip(arguments) {
        kind::unify(parameter, &argument.kind(), &mut substitutions, span)?;
    }
    for requirement in requirements {
        let left = kind::substitute(&requirement.left, &substitutions);
        let right = kind::substitute(&requirement.right, &substitutions);
        if kind::unify(&left, &right, &mut substitutions, requirement.span).is_err() {
            return Err(
                Diagnostic::error("generic instance violates a kind requirement")
                    .with_primary(
                        requirement.span,
                        format!(
                            "this application needs kind `{}` but the instance gives `{}`",
                            kind::name(&right),
                            kind::name(&left)
                        ),
                    )
                    .with_note(
                        "kinds come from signatures; a body that uses a parameter at a narrower kind is checked when it is specialized",
                    ),
            );
        }
    }
    Ok(())
}
