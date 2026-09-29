use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, Operation, Pattern, Program};

use super::Definition;

mod aliases;
mod index;
mod walk;

pub(super) use aliases::{aliases, origin};
pub(super) use index::Index;
pub(super) use walk::{for_each_block, operation_atoms};

pub(super) fn parameter_callbacks(
    function: &crate::closure::ast::Function,
    parameter: ValueId,
) -> Vec<(ValueId, Vec<usize>)> {
    let mut callbacks = Vec::new();
    for binding in &function.body.bindings {
        if let Operation::Atom(atom) = &binding.operation
            && atom.binding() == Some(parameter)
        {
            collect_function_leaves(&binding.pattern, &mut Vec::new(), &mut callbacks);
        }
    }
    callbacks
}

fn collect_function_leaves(
    pattern: &Pattern,
    path: &mut Vec<usize>,
    output: &mut Vec<(ValueId, Vec<usize>)>,
) {
    match pattern {
        Pattern::Binding {
            id,
            ty: Type::Function { .. },
        } => output.push((*id, path.clone())),
        Pattern::Product { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                path.push(index);
                collect_function_leaves(element, path, output);
                path.pop();
            }
        }
        _ => {}
    }
}

pub(super) fn definitions(program: &Program) -> HashMap<ValueId, Definition> {
    let mut definitions = HashMap::new();
    for_each_block(program, &mut |block| {
        for binding in &block.bindings {
            if let Pattern::Binding { id, .. } = binding.pattern {
                definitions.insert(
                    id,
                    Definition {
                        operation: binding.operation.clone(),
                    },
                );
            }
        }
    });
    definitions
}

/// Resolves a product-leaf path while walking backwards through SSA aliases and constructors.
///
/// The returned atom identities cover every use along that trace. Admission treats precisely
/// those identities as allowed forwarding uses of the callback. The returned values are the bindings whose
/// definitions the trace read.
pub(super) fn resolve_path(
    root: &Atom,
    path: &[usize],
    definitions: &HashMap<ValueId, Definition>,
) -> Option<(Atom, HashSet<AtomId>, HashSet<ValueId>)> {
    let mut atom = root.clone();
    let mut trace = HashSet::from([root.id]);
    let mut read = HashSet::new();
    for index in path {
        loop {
            let binding = atom.binding()?;
            read.insert(binding);
            let definition = definitions.get(&binding)?;
            match &definition.operation {
                Operation::Atom(alias) => {
                    trace.insert(alias.id);
                    atom = alias.clone();
                }
                Operation::Product(elements) => {
                    atom = elements.get(*index)?.clone();
                    trace.insert(atom.id);
                    break;
                }
                _ => return None,
            }
        }
    }
    Some((atom, trace, read))
}

pub(super) fn replace_type(ty: &Type, path: &[usize], replacement: &Type) -> Option<Type> {
    let Some((first, rest)) = path.split_first() else {
        return Some(replacement.clone());
    };
    let Type::Product(elements) = ty else {
        return None;
    };
    let mut elements = elements.to_vec();
    elements[*first] = replace_type(elements.get(*first)?, rest, replacement)?;
    Some(Type::Product(elements.into()))
}
