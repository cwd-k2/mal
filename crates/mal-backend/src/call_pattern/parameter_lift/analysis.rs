use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomId, AtomKind, FunctionId, Operation, Pattern, Program, Reference,
};

use super::Definition;

mod aliases;
mod walk;

pub(super) use aliases::{aliases, origin};
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
/// those identities as allowed forwarding uses of the callback.
pub(super) fn resolve_path(
    root: &Atom,
    path: &[usize],
    definitions: &HashMap<ValueId, Definition>,
) -> Option<(Atom, HashSet<AtomId>)> {
    let mut atom = root.clone();
    let mut trace = HashSet::from([root.id]);
    for index in path {
        loop {
            let definition = definitions.get(&atom.binding()?)?;
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
    Some((atom, trace))
}

pub(super) fn has_unapproved_uses(
    program: &Program,
    sought: ValueId,
    aliases: &HashMap<ValueId, ValueId>,
    allowed: &HashSet<AtomId>,
) -> bool {
    let mut rejected = false;
    for_each_atom(program, &mut |atom| {
        if origin(atom, aliases) == Some(sought) && !allowed.contains(&atom.id) {
            rejected = true;
        }
    });
    rejected
}

pub(super) fn all_capturing_creators(program: &Program, target: FunctionId) -> HashSet<ValueId> {
    let mut creators = HashSet::new();
    for_each_block(program, &mut |block| {
        for binding in &block.bindings {
            if let Pattern::Binding { id, .. } = binding.pattern
                && let Operation::MakeClosure { function, captures } = &binding.operation
                && *function == target
                && !captures.is_empty()
            {
                creators.insert(id);
            }
        }
    });
    creators
}

pub(super) fn contains_self_closure(program: &Program, target: FunctionId) -> bool {
    let mut found = false;
    for_each_atom(program, &mut |atom| {
        found |= atom.kind == AtomKind::Reference(Reference::SelfClosure(target));
    });
    found
}

pub(super) fn binding_type(program: &Program, sought: ValueId) -> Option<Type> {
    let mut result = None;
    for_each_pattern(program, &mut |pattern| {
        if let Pattern::Binding { id, ty } = pattern
            && *id == sought
        {
            result = Some(ty.clone());
        }
    });
    result
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

fn for_each_atom(program: &Program, visit: &mut impl FnMut(&Atom)) {
    for_each_block(program, &mut |block| {
        visit(&block.result);
        for binding in &block.bindings {
            operation_atoms(&binding.operation, visit);
        }
    });
}

fn for_each_pattern(program: &Program, visit: &mut impl FnMut(&Pattern)) {
    fn pattern(current: &Pattern, visit: &mut impl FnMut(&Pattern)) {
        visit(current);
        if let Pattern::Product { elements, .. } = current {
            for element in elements {
                pattern(element, visit);
            }
        }
    }
    for_each_block(program, &mut |block| {
        for binding in &block.bindings {
            pattern(&binding.pattern, visit);
            if let Operation::Case { arms, .. } = &binding.operation {
                for arm in arms {
                    pattern(&arm.pattern, visit);
                }
            }
        }
    });
}
