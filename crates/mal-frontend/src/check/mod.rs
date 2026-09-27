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

pub fn type_name(ty: &ast::Type) -> String {
    types::type_name(ty)
}

use self::ast::{AbruptExpression, Completion, Program, TopItem, Type};
use self::interface::ExternalSignature;
use self::types::GenericAliasDefinition;

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

pub fn admit_monomorphic(program: Program) -> Result<ast::MonomorphicProgram, Diagnostic> {
    if let Some(item) = program
        .items
        .iter()
        .find(|item| matches!(item.kind, TopItem::GenericBinding(_)))
    {
        return Err(Diagnostic::error("generic program requires specialization")
            .with_primary(item.span, "this generic binding has not been specialized"));
    }
    Ok(ast::MonomorphicProgram::new(program))
}

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

struct Checker {
    aliases: HashMap<TypeId, Node<resolved::TypeExpression>>,
    generic_aliases: HashMap<TypeId, GenericAliasDefinition>,
    type_substitutions: std::sync::Arc<HashMap<TypeId, Type>>,
    active_requirements: HashSet<TypeId>,
    active_generic: Option<(ValueId, Vec<TypeId>)>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    aggregate_alias_sources: HashMap<TypeId, Option<Vec<Node<resolved::TypeExpression>>>>,
    expanding: HashSet<TypeId>,
    values: HashMap<ValueId, Type>,
    generic_signatures: HashMap<ValueId, GenericSignature>,
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
            type_substitutions: Default::default(),
            active_requirements: HashSet::new(),
            active_generic: None,
            external_types: HashMap::new(),
            expanded_aliases: HashMap::new(),
            aggregate_alias_sources: HashMap::new(),
            expanding: HashSet::new(),
            values: HashMap::from([(FALSE_VALUE, bool_type.clone()), (TRUE_VALUE, bool_type)]),
            generic_signatures: HashMap::new(),
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
                },
            );
            let checked_value = self.check_expression(value, Some(&ty))?;
            self.check_top_level_initializer(&checked_value)?;
            Ok(ast::GenericBinding {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                value: checked_value,
                span,
            })
        })();
        self.type_substitutions = previous;
        self.active_requirements = previous_requirements;
        self.active_generic = previous_generic;
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
