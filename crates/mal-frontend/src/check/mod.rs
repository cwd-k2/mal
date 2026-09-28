//! Type admission and checked expression construction.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{self as resolved, FALSE_VALUE, TRUE_VALUE, TypeId, ValueId};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

pub mod ast;
mod binding;
mod control;
mod entry;
mod expression;
mod float;
mod inference;
mod initializer;
mod integer;
mod interface;
mod lambda;
mod memory;
mod operator;
mod product;
mod specialization_identity;
mod specialize;
pub mod type_fingerprint;
mod types;

/// Renders a bounded diagnostic name for an admitted type without exposing its internal sharing.
pub fn type_name(ty: &ast::Type) -> String {
    types::type_name(ty)
}

use self::ast::{AbruptExpression, Completion, Program, TopItem, Type};
use self::interface::ExternalSignature;
use self::types::GenericAliasDefinition;

/// Applies every type and completion rule to a resolved program while retaining generic declarations.
pub fn check(program: &resolved::Program) -> Result<Program, Diagnostic> {
    Checker::new()
        .check_program(program)
        .map_err(|error| match error {
            CheckFailure::Diagnostic(diagnostic) => diagnostic,
            CheckFailure::Abrupt(abrupt) => Diagnostic::error("abrupt completion outside a lambda")
                .with_primary(
                    abrupt.span,
                    "this control expression has no return boundary",
                ),
        })
}

/// Instantiates the generic bindings reachable from `main` once per concrete type argument list.
/// Fails when the program has no `main`.
pub fn specialize(program: Program) -> Result<ast::MonomorphicProgram, Diagnostic> {
    specialize::specialize(program)
}

/// Admits an already nongeneric checked program to the same downstream boundary as specialization.
pub fn admit_monomorphic(program: Program) -> Result<ast::MonomorphicProgram, Diagnostic> {
    if let Some(item) = program.items.iter().find(|item| {
        matches!(
            item.kind,
            TopItem::OpaqueType { .. }
                | TopItem::GenericBinding(_)
                | TopItem::OperationFamily(_)
                | TopItem::OperationImplementation(_)
        )
    }) {
        return Err(
            Diagnostic::error("program requires specialization").with_primary(
                item.span,
                "this opaque, generic, or operation item has not been specialized",
            ),
        );
    }
    Ok(ast::MonomorphicProgram::new(program))
}

/// Returns the first lambda identity unused by the checked program for downstream identity allocation.
pub fn next_lambda_identity(program: &Program) -> u32 {
    specialization_identity::next_identities(program)
        .expect("an admitted monomorphic program has remaining identity space")
        .lambda
}

enum CheckFailure {
    Diagnostic(Diagnostic),
    Abrupt(Box<AbruptExpression>),
}

impl From<Diagnostic> for CheckFailure {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

type CheckResult<T> = Result<T, CheckFailure>;

#[derive(Clone)]
struct Checker {
    aliases: HashMap<TypeId, Node<resolved::TypeExpression>>,
    generic_aliases: HashMap<TypeId, GenericAliasDefinition>,
    opaque_types: HashMap<TypeId, types::OpaqueDefinition>,
    type_substitutions: std::sync::Arc<HashMap<TypeId, Type>>,
    active_requirements: HashSet<TypeId>,
    active_generic: Option<(ValueId, Vec<TypeId>)>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    aggregate_alias_sources: HashMap<TypeId, Option<Vec<Node<resolved::TypeExpression>>>>,
    expanding: HashSet<TypeId>,
    values: HashMap<ValueId, Type>,
    generic_signatures: HashMap<ValueId, GenericSignature>,
    operation_families: HashSet<ValueId>,
    operation_keys: Vec<(ValueId, Vec<Type>, Span)>,
    active_operations: Vec<ast::OperationRequirement>,
    external_values: HashSet<ValueId>,
    externals: HashMap<resolved::ExternalOperationId, ExternalSignature>,
    result_targets: HashMap<ValueId, ResultTarget>,
    used_result_targets: HashSet<ValueId>,
}

#[derive(Clone)]
struct GenericSignature {
    parameters: Vec<resolved::TypeBinding>,
    ty: Type,
    requirements: HashSet<TypeId>,
    operations: Vec<ast::OperationRequirement>,
}

#[derive(Clone)]
struct ResultTarget {
    parameter: Type,
    result: Type,
    variant: Option<usize>,
    boundary: ValueId,
}

impl Checker {
    fn new() -> Self {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
        Self {
            aliases: HashMap::new(),
            generic_aliases: HashMap::new(),
            opaque_types: HashMap::new(),
            type_substitutions: Default::default(),
            active_requirements: HashSet::new(),
            active_generic: None,
            external_types: HashMap::new(),
            expanded_aliases: HashMap::new(),
            aggregate_alias_sources: HashMap::new(),
            expanding: HashSet::new(),
            values: HashMap::from([(FALSE_VALUE, bool_type.clone()), (TRUE_VALUE, bool_type)]),
            generic_signatures: HashMap::new(),
            operation_families: HashSet::new(),
            operation_keys: Vec::new(),
            active_operations: Vec::new(),
            external_values: HashSet::new(),
            externals: HashMap::new(),
            result_targets: HashMap::new(),
            used_result_targets: HashSet::new(),
        }
    }

    fn check_program(mut self, program: &resolved::Program) -> CheckResult<Program> {
        self.collect_aliases(program);
        // Source order keeps the reported error the same from run to run when several aliases are invalid.
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, .. } => {
                    self.expand_type_id(binding.id, binding.name.span)?;
                }
                resolved::TopItem::GenericTypeAlias { binding, .. } => {
                    let definition = self.generic_aliases[&binding.id].clone();
                    self.validate_generic_alias(&definition)?;
                }
                resolved::TopItem::OpaqueType { binding, .. } => {
                    let definition = self.opaque_types[&binding.id].clone();
                    self.validate_opaque(&definition)?;
                    self.aggregate_alias_sources.insert(binding.id, None);
                }
                _ => {}
            }
        }
        self.collect_external_signatures(program)?;

        let mut items = Vec::with_capacity(program.items.len());
        let mut entry = None;
        for item in &program.items {
            if matches!(item.kind, resolved::TopItem::GenericTypeAlias { .. }) {
                continue;
            }
            if let resolved::TopItem::OperationFamily {
                binding,
                parameters,
                annotation,
            } = &item.kind
            {
                let family =
                    self.check_operation_family(binding, parameters, annotation, item.span)?;
                items.push(Node::new(
                    TopItem::OperationFamily(Box::new(family)),
                    item.span,
                ));
                continue;
            }
            if let resolved::TopItem::OperationImplementation {
                family,
                parameters,
                arguments,
                annotation,
                value,
            } = &item.kind
            {
                let implementation = self.check_operation_implementation(
                    family, parameters, arguments, annotation, value, item.span,
                )?;
                items.push(Node::new(
                    TopItem::OperationImplementation(Box::new(implementation)),
                    item.span,
                ));
                continue;
            }
            if let resolved::TopItem::GenericBinding {
                binding,
                parameters,
                annotation,
                value,
            } = &item.kind
            {
                let binding =
                    self.check_generic_binding(binding, parameters, annotation, value, item.span)?;
                items.push(Node::new(
                    TopItem::GenericBinding(Box::new(binding)),
                    item.span,
                ));
                continue;
            }
            let kind = match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    let ty = self.expand_type_id(binding.id, binding.name.span)?;
                    let element_aliases = match &ty {
                        Type::Product(_) | Type::Sum(_) => self.aggregate_aliases(value, &ty),
                        _ => Vec::new(),
                    };
                    TopItem::TypeAlias {
                        host_memory_access: !binding.name.text.starts_with('_')
                            && interface::is_host_mappable(&ty)
                            && types::is_memory_representable(&ty),
                        binding: binding.clone(),
                        ty,
                        element_aliases,
                    }
                }
                resolved::TopItem::OpaqueType { binding, .. } => TopItem::OpaqueType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalType { binding } => TopItem::ExternalType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalOperation {
                    id,
                    binding,
                    lambda_id,
                    ..
                } => {
                    let signature = self
                        .externals
                        .get(id)
                        .expect("external signatures are collected before checking values");
                    TopItem::ExternalOperation {
                        id: *id,
                        binding: binding.clone(),
                        lambda_id: *lambda_id,
                        parameter: signature.parameter.clone(),
                        parameter_alias: signature.parameter_alias.clone(),
                        parameter_aliases: signature.parameter_aliases.clone(),
                        result: signature.result.clone(),
                        result_alias: signature.result_alias.clone(),
                    }
                }
                resolved::TopItem::Binding(binding) => {
                    let checked = self.check_binding(binding, item.span)?;
                    self.check_top_level_initializer(&checked.value)?;
                    if let Some(candidate) = entry::entry_point(&checked)? {
                        entry = Some(candidate);
                    }
                    TopItem::Binding(Box::new(checked))
                }
                resolved::TopItem::GenericBinding { .. } => {
                    unreachable!("generic bindings are checked before monomorphic item emission")
                }
                resolved::TopItem::GenericTypeAlias { .. } => {
                    unreachable!("generic aliases are omitted before checked program emission")
                }
                resolved::TopItem::OperationFamily { .. }
                | resolved::TopItem::OperationImplementation { .. } => {
                    unreachable!("operation items are checked before ordinary item emission")
                }
            };
            items.push(Node::new(kind, item.span));
        }
        Ok(Program {
            items,
            span: program.span,
            entry,
        })
    }

    fn check_generic_binding(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::GenericBinding> {
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                        },
                    )
                })
                .collect(),
        );
        let previous = std::mem::replace(&mut self.type_substitutions, substitutions);
        let previous_requirements = std::mem::take(&mut self.active_requirements);
        let previous_generic = self.active_generic.take();
        let previous_operations = std::mem::take(&mut self.active_operations);
        let result = (|| {
            let ty = self.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            self.active_requirements = requirements.clone();
            self.active_generic = Some((
                binding.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            self.values.insert(binding.id, ty.clone());
            self.generic_signatures.insert(
                binding.id,
                GenericSignature {
                    parameters: parameters.to_vec(),
                    ty: ty.clone(),
                    requirements,
                    operations: Vec::new(),
                },
            );
            let checked_value = self.check_expression(value, Some(&ty))?;
            self.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut self.active_operations);
            self.generic_signatures
                .get_mut(&binding.id)
                .expect("active generic signature is registered")
                .operations = operations.clone();
            Ok(ast::GenericBinding {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                value: checked_value,
                operations,
                span,
            })
        })();
        self.type_substitutions = previous;
        self.active_requirements = previous_requirements;
        self.active_generic = previous_generic;
        self.active_operations = previous_operations;
        result
    }

    fn check_operation_family(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        span: Span,
    ) -> CheckResult<ast::OperationFamily> {
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
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

    fn check_operation_implementation(
        &mut self,
        family: &resolved::ValueReference,
        parameters: &[resolved::TypeBinding],
        arguments: &[Node<resolved::TypeExpression>],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::OperationImplementation> {
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                        },
                    )
                })
                .collect(),
        );
        let previous_substitutions = std::mem::replace(&mut self.type_substitutions, substitutions);
        let previous_generic = self.active_generic.take();
        let previous_operations = std::mem::take(&mut self.active_operations);
        let result = (|| {
            let arguments = arguments
                .iter()
                .map(|argument| self.expand_type(argument))
                .collect::<Result<Vec<_>, _>>()?;
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
            if self.operation_keys.iter().any(|(id, existing, _)| {
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
            self.operation_keys
                .push((family.id, arguments.clone(), span));
            Ok(ast::OperationImplementation {
                family: family.clone(),
                parameters: parameters.to_vec(),
                arguments,
                ty: expected,
                value: checked_value,
                operations,
                span,
            })
        })();
        self.type_substitutions = previous_substitutions;
        self.active_generic = previous_generic;
        self.active_operations = previous_operations;
        result
    }

    fn value_type(&self, reference: &resolved::ValueReference) -> Result<Type, Diagnostic> {
        if self.result_targets.contains_key(&reference.id) {
            return Err(Diagnostic::error("result binder is not a value")
                .with_primary(reference.name.span, "call this binder in callee position"));
        }
        self.values.get(&reference.id).cloned().ok_or_else(|| {
            Diagnostic::error(format!(
                "value `{}` has no inferred type",
                reference.name.text
            ))
            .with_primary(reference.name.span, "its binding is not available here")
        })
    }

    fn check_completion(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Completion, Diagnostic> {
        match self.check_expression(expression, expected) {
            Ok(value) => Ok(Completion::Value(value)),
            Err(CheckFailure::Abrupt(abrupt)) => Ok(Completion::Abrupt(*abrupt)),
            Err(CheckFailure::Diagnostic(diagnostic)) => Err(diagnostic),
        }
    }
}

fn contains_parameter(ty: &Type) -> bool {
    match ty {
        Type::Parameter { .. } => true,
        Type::Buffer(element) => contains_parameter(element),
        Type::Opaque { arguments, .. } => arguments.iter().any(contains_parameter),
        Type::Product(elements) | Type::Sum(elements) => elements.iter().any(contains_parameter),
        Type::Function { parameter, result } => {
            contains_parameter(parameter) || contains_parameter(result)
        }
        _ => false,
    }
}

fn contains_parameter_id(ty: &Type, expected: TypeId) -> bool {
    match ty {
        Type::Parameter { id, .. } => *id == expected,
        Type::Buffer(element) => contains_parameter_id(element, expected),
        Type::Opaque { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_parameter_id(argument, expected)),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| contains_parameter_id(element, expected)),
        Type::Function { parameter, result } => {
            contains_parameter_id(parameter, expected) || contains_parameter_id(result, expected)
        }
        _ => false,
    }
}

fn operation_patterns_overlap(left: &[Type], right: &[Type]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut substitutions = HashMap::new();
    let mut pending = left
        .iter()
        .cloned()
        .zip(right.iter().cloned())
        .collect::<Vec<_>>();
    while let Some((left, right)) = pending.pop() {
        let left = resolve_operation_parameter(left, &substitutions);
        let right = resolve_operation_parameter(right, &substitutions);
        match (&left, &right) {
            (Type::Parameter { id: left, .. }, Type::Parameter { id: right, .. })
                if left == right => {}
            (Type::Parameter { id, .. }, _) => {
                if operation_type_contains(&right, *id, &substitutions) {
                    return false;
                }
                substitutions.insert(*id, right);
            }
            (_, Type::Parameter { id, .. }) => {
                if operation_type_contains(&left, *id, &substitutions) {
                    return false;
                }
                substitutions.insert(*id, left);
            }
            (Type::Buffer(left), Type::Buffer(right)) => {
                pending.push((left.as_ref().clone(), right.as_ref().clone()));
            }
            (
                Type::Opaque {
                    id: left_id,
                    arguments: left,
                    ..
                },
                Type::Opaque {
                    id: right_id,
                    arguments: right,
                    ..
                },
            ) if left_id == right_id && left.len() == right.len() => {
                pending.extend(left.iter().cloned().zip(right.iter().cloned()));
            }
            (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
                if left.len() == right.len() =>
            {
                pending.extend(left.iter().cloned().zip(right.iter().cloned()));
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
                    left_parameter.as_ref().clone(),
                    right_parameter.as_ref().clone(),
                ));
                pending.push((left_result.as_ref().clone(), right_result.as_ref().clone()));
            }
            _ if left == right => {}
            _ => return false,
        }
    }
    true
}

fn resolve_operation_parameter(mut ty: Type, substitutions: &HashMap<TypeId, Type>) -> Type {
    let mut seen = HashSet::new();
    while let Type::Parameter { id, .. } = &ty {
        if !seen.insert(*id) {
            break;
        }
        let Some(replacement) = substitutions.get(id) else {
            break;
        };
        ty = replacement.clone();
    }
    ty
}

fn operation_type_contains(
    root: &Type,
    expected: TypeId,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    let mut pending = vec![root];
    let mut expanded = HashSet::new();
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Parameter { id, .. } if *id == expected => return true,
            Type::Parameter { id, .. } if expanded.insert(*id) => {
                if let Some(replacement) = substitutions.get(id) {
                    pending.push(replacement);
                }
            }
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
    match root {
        Type::Buffer(element) => pending.push(element.as_ref()),
        Type::Opaque { arguments, .. } => pending.extend(arguments.iter()),
        Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
        Type::Function { parameter, result } => {
            pending.push(parameter);
            pending.push(result);
        }
        _ => {}
    }
    while let Some(candidate) = pending.pop() {
        if candidate == required {
            return true;
        }
        match candidate {
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
    false
}
