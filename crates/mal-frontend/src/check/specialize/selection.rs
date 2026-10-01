//! Selection of the operation implementation whose key matches concrete family arguments, and the request
//! of its shared instance.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{ValueBinding, ValueReference};
use mal_syntax::diagnostic::Diagnostic;

use super::super::ast::*;
use super::super::types::type_name;
use super::Specializer;
use super::admission::admit_specialization;

impl Specializer {
    pub(super) fn request_operation(
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
        let (implementation, substitutions) = self
            .implementations
            .iter()
            .enumerate()
            .filter(|(_, implementation)| implementation.family.id == family.id)
            .find_map(|(index, implementation)| {
                match_operation_pattern(implementation, arguments)
                    .map(|substitutions| (index, substitutions))
            })
            .ok_or_else(|| {
                let key = arguments
                    .iter()
                    .map(type_name)
                    .collect::<Vec<_>>()
                    .join(", ");
                Diagnostic::error("missing operation implementation").with_primary(
                    family.name.span,
                    format!("no implementation of `{}<{key}>` exists", family.name.text),
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
            .push((implementation, substitutions, binding.clone()));
        Ok(ValueReference {
            id: binding.id,
            name: family.name.clone(),
        })
    }
}

fn match_operation_pattern(
    implementation: &OperationImplementation,
    arguments: &[Type],
) -> Option<HashMap<crate::resolve::ast::TypeId, Type>> {
    if implementation.arguments.len() != arguments.len() {
        return None;
    }
    let parameters = implementation
        .parameters
        .iter()
        .map(|parameter| parameter.id)
        .collect::<HashSet<_>>();
    let mut substitutions = HashMap::new();
    for (pattern, argument) in implementation.arguments.iter().zip(arguments) {
        if !match_operation_type(pattern, argument, &parameters, &mut substitutions) {
            return None;
        }
    }
    Some(substitutions)
}

fn match_operation_type(
    pattern: &Type,
    argument: &Type,
    parameters: &HashSet<crate::resolve::ast::TypeId>,
    substitutions: &mut HashMap<crate::resolve::ast::TypeId, Type>,
) -> bool {
    if let Type::Parameter { id, .. } = pattern
        && parameters.contains(id)
    {
        return match substitutions.get(id) {
            Some(existing) => existing == argument,
            None => {
                substitutions.insert(*id, argument.clone());
                true
            }
        };
    }
    match (pattern, argument) {
        (Type::Buffer(pattern), Type::Buffer(argument)) => {
            match_operation_type(pattern, argument, parameters, substitutions)
        }
        (
            Type::Opaque {
                id: pattern_id,
                arguments: pattern,
                ..
            },
            Type::Opaque {
                id: argument_id,
                arguments: argument,
                ..
            },
        ) if pattern_id == argument_id && pattern.len() == argument.len() => pattern
            .iter()
            .zip(argument.iter())
            .all(|(pattern, argument)| {
                match_operation_type(pattern, argument, parameters, substitutions)
            }),
        (Type::Product(pattern), Type::Product(argument))
        | (Type::Sum(pattern), Type::Sum(argument))
            if pattern.len() == argument.len() =>
        {
            pattern
                .iter()
                .zip(argument.iter())
                .all(|(pattern, argument)| {
                    match_operation_type(pattern, argument, parameters, substitutions)
                })
        }
        (
            Type::Function {
                parameter: pattern_parameter,
                result: pattern_result,
            },
            Type::Function {
                parameter: argument_parameter,
                result: argument_result,
            },
        ) => {
            match_operation_type(
                pattern_parameter,
                argument_parameter,
                parameters,
                substitutions,
            ) && match_operation_type(pattern_result, argument_result, parameters, substitutions)
        }
        _ => pattern == argument,
    }
}
