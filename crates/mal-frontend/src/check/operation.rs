//! Operation-family declaration, implementation coherence, and termination checking.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{self as resolved, TypeId};
use mal_syntax::{ast::Node, diagnostic::Diagnostic, source::Span};

use super::{CheckResult, Checker, GenericSignature, ast, ast::Type, types};

impl Checker {
    pub(super) fn check_operation_family(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        span: Span,
    ) -> CheckResult<ast::OperationFamily> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let signature_parameter_kinds = parameter_kinds.clone();
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .zip(parameter_kinds)
                .map(|(parameter, kind)| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                            kind,
                        },
                    )
                })
                .collect(),
        );
        let previous = std::mem::replace(&mut self.type_substitutions, substitutions);
        let result = (|| {
            let ty = self.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            self.generic_signatures.insert(
                binding.id,
                GenericSignature {
                    parameters: parameters.to_vec(),
                    parameter_kinds: signature_parameter_kinds,
                    ty: ty.clone(),
                    requirements,
                    operations: Vec::new(),
                },
            );
            self.operation_families.insert(binding.id);
            Ok(ast::OperationFamily {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                span,
            })
        })();
        self.type_substitutions = previous;
        result
    }

    pub(super) fn check_operation_implementation(
        &mut self,
        family: &resolved::ValueReference,
        parameters: &[resolved::TypeBinding],
        arguments: &[Node<resolved::TypeExpression>],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::OperationImplementation> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .zip(parameter_kinds.iter().cloned())
                .map(|(parameter, kind)| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                            kind,
                        },
                    )
                })
                .collect(),
        );
        let previous_substitutions = std::mem::replace(&mut self.type_substitutions, substitutions);
        let previous_generic = self.active_generic.take();
        let previous_operations = std::mem::take(&mut self.active_operations);
        let previous_requirements = std::mem::take(&mut self.active_requirements);
        let previous_kinds = std::mem::take(&mut self.active_kinds);
        let result = (|| {
            let mut arguments = arguments
                .iter()
                .map(|argument| self.expand_type_term(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let signature = self
                .generic_signatures
                .get(&family.id)
                .expect("an implementation refers to a checked operation family");
            let key_kinds = types::require_type_argument_kinds(
                &signature.parameter_kinds,
                &mut arguments,
                family.name.span,
            )?;
            if signature
                .parameter_kinds
                .iter()
                .zip(&arguments)
                .any(|(kind, argument)| {
                    matches!(kind, ast::Kind::Function { .. }) && contains_parameter(argument)
                })
            {
                return Err(
                    Diagnostic::error("operation constructor key must be closed")
                        .with_primary(
                            family.name.span,
                            "replace the constructor parameter with a declared type constructor",
                        )
                        .into(),
                );
            }
            if parameters.is_empty() && arguments.iter().any(contains_parameter) {
                return Err(Diagnostic::error(
                    "exact operation implementation requires closed types",
                )
                .with_primary(family.name.span, "remove generic parameters from this key")
                .into());
            }
            if !parameters.is_empty()
                && arguments
                    .iter()
                    .all(|argument| matches!(argument, Type::Parameter { .. }))
            {
                return Err(Diagnostic::error(
                    "generic operation implementation requires structure",
                )
                .with_primary(
                    family.name.span,
                    "a catch-all parameter key is not supported",
                )
                .into());
            }
            if let Some(parameter) = parameters.iter().find(|parameter| {
                !arguments
                    .iter()
                    .any(|argument| contains_parameter_id(argument, parameter.id))
            }) {
                return Err(Diagnostic::error(
                    "generic operation pattern leaves a parameter unbound",
                )
                .with_primary(
                    parameter.name.span,
                    "use this family parameter in the implementation key",
                )
                .into());
            }
            if self.operation_keys.iter().any(|(id, existing)| {
                *id == family.id && operation_patterns_overlap(existing, &arguments)
            }) {
                return Err(Diagnostic::error("duplicate operation implementation")
                    .with_primary(
                        family.name.span,
                        "this family key overlaps an existing implementation",
                    )
                    .into());
            }
            let expected = self
                .check_generic_reference(family, arguments.clone(), family.name.span)?
                .ty;
            let declared = self.expand_type(annotation)?;
            self.require_type(&declared, &expected, annotation.span)?;
            // Like a generic binding, the body may assume what its signature makes well formed.
            self.active_requirements = types::storable_requirements(&expected);
            self.active_generic = (!parameters.is_empty()).then_some((
                family.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            let checked_value = self.check_expression(value, Some(&expected))?;
            self.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut self.active_operations);
            if !parameters.is_empty() {
                for requirement in &operations {
                    let direct_self =
                        requirement.family.id == family.id && requirement.arguments == arguments;
                    if !direct_self
                        && !operation_requirement_decreases(&arguments, &requirement.arguments)
                    {
                        return Err(Diagnostic::error(
                            "generic operation requirement does not decrease",
                        )
                        .with_primary(
                            requirement.family.name.span,
                            "the required key must be a proper subterm of the implementation key",
                        )
                        .into());
                    }
                }
            }
            let mut kinds = key_kinds;
            for requirement in std::mem::take(&mut self.active_kinds) {
                if !kinds.iter().any(|existing| {
                    existing.left == requirement.left && existing.right == requirement.right
                }) {
                    kinds.push(requirement);
                }
            }
            self.operation_keys.push((family.id, arguments.clone()));
            Ok(ast::OperationImplementation {
                family: family.clone(),
                parameters: parameters.to_vec(),
                arguments,
                ty: expected,
                value: checked_value,
                operations,
                parameter_kinds,
                kinds,
                span,
            })
        })();
        self.type_substitutions = previous_substitutions;
        self.active_generic = previous_generic;
        self.active_operations = previous_operations;
        self.active_requirements = previous_requirements;
        self.active_kinds = previous_kinds;
        result
    }
}

pub(super) fn contains_parameter(ty: &Type) -> bool {
    contains_parameter_id_if(ty, |_| true)
}

fn contains_parameter_id(ty: &Type, expected: TypeId) -> bool {
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

fn operation_patterns_overlap(left: &[Type], right: &[Type]) -> bool {
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

fn operation_requirement_decreases(pattern: &[Type], requirement: &[Type]) -> bool {
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
