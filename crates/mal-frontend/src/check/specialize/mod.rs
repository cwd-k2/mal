//! Reachability-driven monomorphization with shared instances and globally fresh binder identities.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{LambdaId, ValueBinding, ValueId, ValueReference};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

use super::ast::*;
use super::specialization_identity::next_identities;
use super::type_fingerprint::TypeFingerprints;
use super::types::substitute_type;

mod admission;
mod expression;
mod instance_identity;
mod structure;
mod substitution;

use admission::{admit_specialization, collect_pattern_bindings};

pub(super) fn specialize(program: Program) -> Result<MonomorphicProgram, Diagnostic> {
    let identities = next_identities(&program).ok_or_else(|| {
        Diagnostic::error("compiler identity space exhausted").with_primary(
            program.span,
            "cannot allocate identities for generic specializations",
        )
    })?;
    let program_span = program.span;
    let entry = program.entry.ok_or_else(|| {
        Diagnostic::error("missing entry point")
            .with_primary(program_span, "the root file must declare `main`")
    })?;
    let mut definitions = HashMap::new();
    let mut implementations = Vec::new();
    let mut bindings = Vec::new();
    let mut binding_items = HashMap::new();
    let mut items = Vec::new();
    for item in program.items {
        match item.kind {
            TopItem::GenericBinding(definition) => {
                definitions.insert(definition.binding.id, *definition);
            }
            TopItem::OperationImplementation(implementation) => {
                implementations.push(*implementation);
            }
            TopItem::OperationFamily(_) => {}
            TopItem::Binding(binding) => {
                let index = bindings.len();
                collect_pattern_bindings(&binding.pattern, index, &mut binding_items);
                bindings.push(Some(Node::new(TopItem::Binding(binding), item.span)));
            }
            _ => items.push(item),
        }
    }
    let mut specializer = Specializer {
        definitions,
        implementations,
        bindings,
        binding_items,
        selected_bindings: HashSet::new(),
        reachable: HashMap::new(),
        specializations: Vec::new(),
        instances: Vec::new(),
        instance_buckets: HashMap::new(),
        fingerprints: TypeFingerprints::default(),
        pending: Vec::new(),
        pending_operations: Vec::new(),
        next_value: identities.value,
        next_lambda: identities.lambda,
        value_renames: HashMap::new(),
        lambda_renames: HashMap::new(),
    };
    specializer.request_binding(entry.binding)?;
    let mut generic_cursor = 0;
    let mut operation_cursor = 0;
    while generic_cursor < specializer.pending.len()
        || operation_cursor < specializer.pending_operations.len()
    {
        if generic_cursor < specializer.pending.len() {
            let (generic, arguments, binding) = specializer.pending[generic_cursor].clone();
            generic_cursor += 1;
            let definition = specializer
                .definitions
                .get(&generic)
                .cloned()
                .expect("checked generic reference has a definition");
            let substitutions = definition
                .parameters
                .iter()
                .map(|parameter| parameter.id)
                .zip(arguments)
                .collect::<HashMap<_, _>>();
            let mut value = definition.value;
            specializer.begin_instance_identities();
            specializer.expression(&mut value, &substitutions, Some((generic, binding.id)))?;
            let ty = substitute_type(&definition.ty, &substitutions);
            specializer.specializations.push(Node::new(
                TopItem::Binding(Box::new(Binding {
                    pattern: Pattern::Binding {
                        binding,
                        ty: ty.clone(),
                    },
                    annotation: Some(ty),
                    value,
                    span: definition.span,
                })),
                definition.span,
            ));
            continue;
        }

        let (family, arguments, binding) = specializer.pending_operations[operation_cursor].clone();
        operation_cursor += 1;
        let implementation = specializer
            .implementations
            .iter()
            .find(|implementation| {
                implementation.family.id == family.id && implementation.arguments == arguments
            })
            .cloned()
            .expect("a queued operation has an exact implementation");
        let mut value = implementation.value;
        specializer.begin_instance_identities();
        specializer.expression(&mut value, &HashMap::new(), Some((family.id, binding.id)))?;
        specializer.specializations.push(Node::new(
            TopItem::Binding(Box::new(Binding {
                pattern: Pattern::Binding {
                    binding,
                    ty: implementation.ty.clone(),
                },
                annotation: Some(implementation.ty),
                value,
                span: implementation.span,
            })),
            implementation.span,
        ));
    }
    for index in 0..specializer.bindings.len() {
        if let Some(binding) = specializer.reachable.remove(&index) {
            items.push(binding);
        }
    }
    items.extend(specializer.specializations);
    Ok(MonomorphicProgram::new(Program {
        items,
        span: program_span,
        entry: Some(entry),
    }))
}

struct Specializer {
    definitions: HashMap<ValueId, GenericBinding>,
    implementations: Vec<OperationImplementation>,
    bindings: Vec<Option<Node<TopItem>>>,
    binding_items: HashMap<ValueId, usize>,
    selected_bindings: HashSet<usize>,
    reachable: HashMap<usize, Node<TopItem>>,
    specializations: Vec<Node<TopItem>>,
    instances: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    instance_buckets: HashMap<(ValueId, u64), Vec<usize>>,
    fingerprints: TypeFingerprints,
    pending: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    pending_operations: Vec<(ValueReference, Vec<Type>, ValueBinding)>,
    next_value: u32,
    next_lambda: u32,
    value_renames: HashMap<ValueId, ValueId>,
    lambda_renames: HashMap<LambdaId, LambdaId>,
}

impl Specializer {
    fn fresh_value(&mut self, span: mal_syntax::source::Span) -> Result<ValueId, Diagnostic> {
        let id = ValueId(self.next_value);
        self.next_value = self.next_value.checked_add(1).ok_or_else(|| {
            Diagnostic::error("compiler identity space exhausted")
                .with_primary(span, "cannot allocate an identity for a specialized binder")
        })?;
        Ok(id)
    }

    fn request_binding(&mut self, id: ValueId) -> Result<(), Diagnostic> {
        let Some(&index) = self.binding_items.get(&id) else {
            return Ok(());
        };
        if !self.selected_bindings.insert(index) {
            return Ok(());
        }
        let mut item = self.bindings[index]
            .take()
            .expect("a selected binding is taken exactly once");
        let TopItem::Binding(binding) = &mut item.kind else {
            unreachable!("the binding table contains only bindings")
        };
        self.expression(&mut binding.value, &HashMap::new(), None)?;
        self.reachable.insert(index, item);
        Ok(())
    }

    /// Returns the shared monomorphic binding for an exact generic argument list.
    ///
    /// Fingerprints only select a collision bucket; equality of the complete argument list
    /// decides reuse. A new instance is registered before its body is expanded from `pending`, so
    /// recursive references resolve to the same identity instead of requesting another instance.
    fn request(
        &mut self,
        reference: &ValueReference,
        arguments: &[Type],
    ) -> Result<ValueReference, Diagnostic> {
        let fingerprint = self.fingerprints.arguments(arguments);
        if let Some((_, _, binding)) = self
            .instance_buckets
            .get(&(reference.id, fingerprint))
            .into_iter()
            .flatten()
            .filter_map(|&index| self.instances.get(index))
            .find(|(id, existing, _)| *id == reference.id && existing == arguments)
        {
            return Ok(ValueReference {
                id: binding.id,
                name: reference.name.clone(),
            });
        }
        admit_specialization(self.instances.len(), reference.name.span)?;
        let definition = self
            .definitions
            .get(&reference.id)
            .expect("checked generic reference has a definition");
        let name = definition.binding.name.clone();
        let owner = definition.binding.owner;
        let binding = ValueBinding {
            id: self.fresh_value(reference.name.span)?,
            name,
            owner,
        };
        let entry = (reference.id, arguments.to_vec(), binding.clone());
        let index = self.instances.len();
        self.instances.push(entry.clone());
        self.instance_buckets
            .entry((reference.id, fingerprint))
            .or_default()
            .push(index);
        self.pending.push(entry);
        Ok(ValueReference {
            id: binding.id,
            name: reference.name.clone(),
        })
    }

    fn request_operation(
        &mut self,
        family: &ValueReference,
        arguments: &[Type],
    ) -> Result<ValueReference, Diagnostic> {
        let fingerprint = self.fingerprints.arguments(arguments);
        if let Some((_, _, binding)) = self
            .instance_buckets
            .get(&(family.id, fingerprint))
            .into_iter()
            .flatten()
            .filter_map(|&index| self.instances.get(index))
            .find(|(id, existing, _)| *id == family.id && existing == arguments)
        {
            return Ok(ValueReference {
                id: binding.id,
                name: family.name.clone(),
            });
        }
        let _implementation = self
            .implementations
            .iter()
            .find(|implementation| {
                implementation.family.id == family.id && implementation.arguments == arguments
            })
            .cloned()
            .ok_or_else(|| {
                Diagnostic::error("missing operation implementation").with_primary(
                    family.name.span,
                    format!(
                        "no exact implementation of `{}` exists for these type arguments",
                        family.name.text
                    ),
                )
            })?;
        admit_specialization(self.instances.len(), family.name.span)?;
        let binding = ValueBinding {
            id: self.fresh_value(family.name.span)?,
            name: family.name.clone(),
            owner: crate::resolve::ast::ValueOwner::TopLevel,
        };
        let entry = (family.id, arguments.to_vec(), binding.clone());
        let index = self.instances.len();
        self.instances.push(entry);
        self.instance_buckets
            .entry((family.id, fingerprint))
            .or_default()
            .push(index);
        self.pending_operations
            .push((family.clone(), arguments.to_vec(), binding.clone()));
        Ok(ValueReference {
            id: binding.id,
            name: family.name.clone(),
        })
    }
}
