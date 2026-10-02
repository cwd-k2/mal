//! Patterns that bind a lifted callback: expanded into its captures, or retyped to the closure that replaces it.

use super::*;

pub(super) fn expand_callback_pattern(
    function: &mut crate::closure::ast::Function,
    candidate: &Candidate,
    captures: &[(ValueId, Type)],
    shape: &lambda_lift::LiftedShape,
) {
    for index in 0..function.body.bindings.len() {
        let binding = &mut function.body.bindings[index];
        let span = binding.span;
        if !matches!(&binding.operation, Operation::Atom(atom) if atom.binding() == Some(candidate.parameter))
            || !replace_callback_pattern(&mut binding.pattern, candidate.callback, captures, span)
        {
            continue;
        }
        function.body.bindings.insert(
            index + 1,
            Binding {
                pattern: Pattern::Binding {
                    id: candidate.callback,
                    ty: shape.closure_type.clone(),
                },
                operation: Operation::MakeClosure {
                    function: candidate.target,
                    captures: Vec::new(),
                },
                span,
            },
        );
        return;
    }
}

fn replace_callback_pattern(
    pattern: &mut Pattern,
    callback: ValueId,
    captures: &[(ValueId, Type)],
    span: mal_syntax::source::Span,
) -> bool {
    match pattern {
        Pattern::Binding { id, .. } if *id == callback => {
            *pattern = Pattern::Product {
                elements: captures
                    .iter()
                    .map(|(id, ty)| Pattern::Binding {
                        id: *id,
                        ty: ty.clone(),
                    })
                    .collect(),
                ty: Type::Product(
                    captures
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect::<Vec<_>>()
                        .into(),
                ),
                span,
            };
            true
        }
        Pattern::Product { elements, .. } => elements
            .iter_mut()
            .any(|element| replace_callback_pattern(element, callback, captures, span)),
        _ => false,
    }
}

pub(super) fn rewrite_pattern(pattern: &mut Pattern, types: &mut HashMap<ValueId, Type>) -> Type {
    match pattern {
        Pattern::Binding { id, ty } => {
            if let Some(replacement) = types.get(id) {
                *ty = replacement.clone();
            }
            ty.clone()
        }
        Pattern::Wildcard { ty, .. } => ty.clone(),
        Pattern::Product { elements, ty, .. } => {
            *ty = Type::Product(
                elements
                    .iter_mut()
                    .map(|element| rewrite_pattern(element, types))
                    .collect::<Vec<_>>()
                    .into(),
            );
            ty.clone()
        }
    }
}

pub(super) fn set_pattern_type(
    pattern: &mut Pattern,
    replacement: &Type,
    types: &mut HashMap<ValueId, Type>,
) {
    match pattern {
        Pattern::Binding { id, ty } => {
            *ty = replacement.clone();
            types.insert(*id, replacement.clone());
        }
        Pattern::Wildcard { ty, .. } | Pattern::Product { ty, .. } => {
            *ty = replacement.clone();
        }
    }
}

pub(super) fn rewrite_top_level_patterns(
    bindings: &mut [crate::closure::ast::TopLevelBinding],
    sought: ValueId,
    replacement: &Type,
) {
    fn rewrite(pattern: &mut TopLevelPattern, sought: ValueId, replacement: &Type) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } if *id == sought => *ty = replacement.clone(),
            TopLevelPattern::Product { elements, .. } => {
                for element in elements {
                    rewrite(element, sought, replacement);
                }
            }
            _ => {}
        }
    }
    for binding in bindings {
        rewrite(&mut binding.pattern, sought, replacement);
    }
}
