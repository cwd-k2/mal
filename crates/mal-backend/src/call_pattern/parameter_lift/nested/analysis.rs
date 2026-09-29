use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Block, Operation, Pattern, Program, Reference};

use super::super::analysis::{Index, operation_atoms, origin};
use super::Use;

pub(in crate::call_pattern::parameter_lift) fn find(
    program: &Program,
    index: &Index<'_>,
    callback: ValueId,
) -> Option<Use> {
    for function in &program.functions {
        for (capture, field) in function.captures.iter().enumerate() {
            if !matches!(field.ty, Type::Function { .. }) {
                continue;
            }
            let Some(capture_atoms) = matching_creators(index, callback, function.id, capture)
            else {
                continue;
            };
            let aliases = capture_aliases(function, capture);
            if capture_is_only_called(function, capture, &aliases) {
                return Some(Use {
                    function: function.id,
                    capture,
                    aliases,
                    capture_atoms,
                });
            }
        }
    }
    None
}

/// The capture atoms of every closure of `function`, when each one captures `callback` at `capture`.
fn matching_creators(
    index: &Index<'_>,
    callback: ValueId,
    function: crate::closure::ast::FunctionId,
    capture: usize,
) -> Option<HashSet<crate::closure::ast::AtomId>> {
    let creators = index.closure_captures(function);
    if creators.is_empty() {
        return None;
    }
    creators
        .iter()
        .map(|captures| {
            let atom = captures.get(capture)?;
            (origin(atom, &index.aliases) == Some(callback)).then_some(atom.id)
        })
        .collect()
}

fn capture_aliases(function: &crate::closure::ast::Function, capture: usize) -> HashSet<ValueId> {
    let mut aliases = HashSet::new();
    let mut pending = Vec::new();
    let mut dependents = HashMap::<ValueId, Vec<ValueId>>::new();
    for_function_block(function, &mut |block| {
        for item in &block.bindings {
            let Pattern::Binding { id, .. } = item.pattern else {
                continue;
            };
            let Operation::Atom(atom) = &item.operation else {
                continue;
            };
            if atom.kind == AtomKind::Reference(Reference::Capture(capture)) {
                if aliases.insert(id) {
                    pending.push(id);
                }
            } else if let Some(source) = atom.binding() {
                dependents.entry(source).or_default().push(id);
            }
        }
    });
    while let Some(source) = pending.pop() {
        for alias in dependents.remove(&source).unwrap_or_default() {
            if aliases.insert(alias) {
                pending.push(alias);
            }
        }
    }
    aliases
}

fn capture_is_only_called(
    function: &crate::closure::ast::Function,
    capture: usize,
    aliases: &HashSet<ValueId>,
) -> bool {
    let is_capture = |atom: &Atom| {
        atom.kind == AtomKind::Reference(Reference::Capture(capture))
            || atom.binding().is_some_and(|id| aliases.contains(&id))
    };
    let mut direct = false;
    let mut other = false;
    for_function_block(function, &mut |block| {
        other |= is_capture(&block.result);
        for item in &block.bindings {
            let alias = matches!(
                (&item.pattern, &item.operation),
                (Pattern::Binding { id, .. }, Operation::Atom(atom))
                    if aliases.contains(id) && is_capture(atom)
            );
            match &item.operation {
                Operation::Call { callee, argument } => {
                    direct |= is_capture(callee);
                    other |= is_capture(argument);
                }
                _ if !alias => {
                    operation_atoms(&item.operation, &mut |atom| other |= is_capture(atom))
                }
                _ => {}
            }
        }
    });
    direct && !other
}

fn for_function_block(function: &crate::closure::ast::Function, visit: &mut impl FnMut(&Block)) {
    fn walk(block: &Block, visit: &mut impl FnMut(&Block)) {
        visit(block);
        for binding in &block.bindings {
            match &binding.operation {
                Operation::Case { arms, .. } => {
                    for arm in arms {
                        walk(&arm.value, visit);
                    }
                }
                Operation::PrimitiveBranch {
                    otherwise, then, ..
                } => {
                    walk(otherwise, visit);
                    walk(then, visit);
                }
                _ => {}
            }
        }
    }
    walk(&function.body, visit);
    for join in &function.joins {
        walk(&join.body, visit);
    }
}
