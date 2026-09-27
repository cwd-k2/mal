use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Block, Operation, Pattern, Program, Reference};

use super::super::analysis::{for_each_block, operation_atoms, origin};
use super::Use;

pub(in crate::call_pattern::parameter_lift) fn find(
    program: &Program,
    callback: ValueId,
    origins: &HashMap<ValueId, ValueId>,
) -> Option<Use> {
    for function in &program.functions {
        for (capture, field) in function.captures.iter().enumerate() {
            if !matches!(field.ty, Type::Function { .. }) {
                continue;
            }
            let Some(capture_atoms) =
                matching_creators(program, callback, origins, function.id, capture)
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

fn matching_creators(
    program: &Program,
    callback: ValueId,
    origins: &HashMap<ValueId, ValueId>,
    function: crate::closure::ast::FunctionId,
    capture: usize,
) -> Option<HashSet<crate::closure::ast::AtomId>> {
    let mut found = false;
    let mut valid = true;
    let mut atoms = HashSet::new();
    for_each_block(program, &mut |block| {
        for binding in &block.bindings {
            let Operation::MakeClosure {
                function: created,
                captures,
            } = &binding.operation
            else {
                continue;
            };
            if *created != function {
                continue;
            }
            found = true;
            let Some(atom) = captures.get(capture) else {
                valid = false;
                continue;
            };
            valid &= origin(atom, origins) == Some(callback);
            atoms.insert(atom.id);
        }
    });
    (found && valid).then_some(atoms)
}

fn capture_aliases(function: &crate::closure::ast::Function, capture: usize) -> HashSet<ValueId> {
    let mut aliases = HashSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for_function_block(function, &mut |block| {
            for item in &block.bindings {
                let Pattern::Binding { id, .. } = item.pattern else {
                    continue;
                };
                let Operation::Atom(atom) = &item.operation else {
                    continue;
                };
                let source = atom.kind == AtomKind::Reference(Reference::Capture(capture))
                    || atom.binding().is_some_and(|id| aliases.contains(&id));
                changed |= source && aliases.insert(id);
            }
        });
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
