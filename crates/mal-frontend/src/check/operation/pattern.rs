//! Implementation key patterns: parameter occurrence, overlap between two keys, and the structural decrease
//! that requirement keys of a generic implementation must show.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::TypeId;

use super::super::ast::Type;

pub(in crate::check) fn contains_parameter(ty: &Type) -> bool {
    contains_parameter_id_if(ty, |_| true)
}

pub(super) fn contains_parameter_id(ty: &Type, expected: TypeId) -> bool {
    contains_parameter_id_if(ty, |id| id == expected)
}

fn contains_parameter_id_if(ty: &Type, predicate: impl Copy + Fn(TypeId) -> bool) -> bool {
    match ty {
        Type::Parameter { id, .. } => predicate(*id),
        Type::Application {
            constructor,
            argument,
            ..
        } => {
            contains_parameter_id_if(constructor, predicate)
                || contains_parameter_id_if(argument, predicate)
        }
        Type::Abstraction { body, .. } => contains_parameter_id_if(body, predicate),
        Type::Buffer(element) => contains_parameter_id_if(element, predicate),
        Type::Opaque { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_parameter_id_if(argument, predicate)),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| contains_parameter_id_if(element, predicate)),
        Type::Function { parameter, result } => {
            contains_parameter_id_if(parameter, predicate)
                || contains_parameter_id_if(result, predicate)
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PatternSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct PatternVariable {
    side: PatternSide,
    id: TypeId,
}

#[derive(Clone, Copy)]
struct PatternType<'a> {
    side: PatternSide,
    ty: &'a Type,
}

impl PatternType<'_> {
    fn variable(self) -> Option<PatternVariable> {
        let Type::Parameter { id, .. } = self.ty else {
            return None;
        };
        Some(PatternVariable {
            side: self.side,
            id: *id,
        })
    }
}

pub(super) fn operation_patterns_overlap(left: &[Type], right: &[Type]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut substitutions = HashMap::new();
    let mut pending = left
        .iter()
        .map(|ty| PatternType {
            side: PatternSide::Left,
            ty,
        })
        .zip(right.iter().map(|ty| PatternType {
            side: PatternSide::Right,
            ty,
        }))
        .collect::<Vec<_>>();
    while let Some((left, right)) = pending.pop() {
        let left = resolve_operation_parameter(left, &substitutions);
        let right = resolve_operation_parameter(right, &substitutions);
        match (left.variable(), right.variable()) {
            (Some(left), Some(right)) if left == right => continue,
            (Some(variable), _) => {
                if operation_type_contains(right, variable, &substitutions) {
                    return false;
                }
                substitutions.insert(variable, right);
                continue;
            }
            (_, Some(variable)) => {
                if operation_type_contains(left, variable, &substitutions) {
                    return false;
                }
                substitutions.insert(variable, left);
                continue;
            }
            _ => {}
        }
        match (left.ty, right.ty) {
            (Type::Buffer(left_element), Type::Buffer(right_element)) => {
                pending.push((left.with_type(left_element), right.with_type(right_element)));
            }
            (
                Type::Opaque {
                    id: left_id,
                    arguments: left_arguments,
                    ..
                },
                Type::Opaque {
                    id: right_id,
                    arguments: right_arguments,
                    ..
                },
            ) if left_id == right_id && left_arguments.len() == right_arguments.len() => {
                pending.extend(
                    left_arguments
                        .iter()
                        .map(|ty| left.with_type(ty))
                        .zip(right_arguments.iter().map(|ty| right.with_type(ty))),
                );
            }
            (Type::Product(left_elements), Type::Product(right_elements))
            | (Type::Sum(left_elements), Type::Sum(right_elements))
                if left_elements.len() == right_elements.len() =>
            {
                pending.extend(
                    left_elements
                        .iter()
                        .map(|ty| left.with_type(ty))
                        .zip(right_elements.iter().map(|ty| right.with_type(ty))),
                );
            }
            (
                Type::Function {
                    parameter: left_parameter,
                    result: left_result,
                },
                Type::Function {
                    parameter: right_parameter,
                    result: right_result,
                },
            ) => {
                pending.push((
                    left.with_type(left_parameter),
                    right.with_type(right_parameter),
                ));
                pending.push((left.with_type(left_result), right.with_type(right_result)));
            }
            _ if left.ty == right.ty => {}
            _ => return false,
        }
    }
    true
}

impl<'a> PatternType<'a> {
    fn with_type(self, ty: &'a Type) -> Self {
        Self { ty, ..self }
    }
}

fn resolve_operation_parameter<'a>(
    mut ty: PatternType<'a>,
    substitutions: &HashMap<PatternVariable, PatternType<'a>>,
) -> PatternType<'a> {
    let mut seen = HashSet::new();
    while let Some(variable) = ty.variable() {
        if !seen.insert(variable) {
            break;
        }
        let Some(replacement) = substitutions.get(&variable) else {
            break;
        };
        ty = *replacement;
    }
    ty
}

fn operation_type_contains<'a>(
    root: PatternType<'a>,
    expected: PatternVariable,
    substitutions: &HashMap<PatternVariable, PatternType<'a>>,
) -> bool {
    let mut pending = vec![root];
    let mut expanded = HashSet::new();
    while let Some(ty) = pending.pop() {
        if let Some(variable) = ty.variable() {
            if variable == expected {
                return true;
            }
            if expanded.insert(variable)
                && let Some(replacement) = substitutions.get(&variable)
            {
                pending.push(*replacement);
            }
            continue;
        }
        match ty.ty {
            Type::Buffer(element) => pending.push(ty.with_type(element)),
            Type::Opaque { arguments, .. } => {
                pending.extend(arguments.iter().map(|child| ty.with_type(child)));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().map(|child| ty.with_type(child)));
            }
            Type::Function { parameter, result } => {
                pending.push(ty.with_type(parameter));
                pending.push(ty.with_type(result));
            }
            _ => {}
        }
    }
    false
}

pub(super) fn operation_requirement_decreases(pattern: &[Type], requirement: &[Type]) -> bool {
    requirement.iter().all(|required| {
        pattern
            .iter()
            .any(|root| proper_type_subterm(root, required))
    })
}

fn proper_type_subterm(root: &Type, required: &Type) -> bool {
    let mut pending = Vec::new();
    extend_children(root, &mut pending);
    while let Some(candidate) = pending.pop() {
        if candidate == required {
            return true;
        }
        extend_children(candidate, &mut pending);
    }
    false
}

fn extend_children<'a>(ty: &'a Type, pending: &mut Vec<&'a Type>) {
    match ty {
        Type::Buffer(element) => pending.push(element),
        Type::Opaque { arguments, .. } => pending.extend(arguments.iter()),
        Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
        Type::Function { parameter, result } => {
            pending.push(parameter);
            pending.push(result);
        }
        _ => {}
    }
}
