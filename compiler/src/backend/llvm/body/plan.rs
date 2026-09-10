use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{AtomKind, FunctionId, Pattern, Reference, TopLevelPattern};
use crate::control::ast::{Program, StateId, Terminator};

use super::Slot;
use super::types::Types;

pub(super) fn main_function(execution: &crate::execution::Program) -> Option<FunctionId> {
    let binding = execution.lowered.bindings.iter().find(|binding| {
        matches!(&binding.pattern, TopLevelPattern::Binding { name, .. } if name == "main")
    })?;
    let TopLevelPattern::Binding { ty, .. } = &binding.pattern else {
        return None;
    };
    if *ty
        != (Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        })
    {
        return None;
    }
    closure_binding_function(binding)
}

pub(super) fn top_levels_are_capture_free_closures(execution: &crate::execution::Program) -> bool {
    execution.lowered.bindings.iter().all(|binding| {
        closure_binding_function(binding).is_some_and(|function| {
            execution
                .lowered
                .functions
                .iter()
                .find(|candidate| candidate.id == function)
                .is_some_and(|function| function.environment.is_empty())
        })
    })
}

fn closure_binding_function(binding: &crate::closure::ast::TopLevelBinding) -> Option<FunctionId> {
    let AtomKind::Reference(Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            crate::closure::ast::Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(*function)
            }
            _ => None,
        }
    })
}

pub(super) fn reachable_states(program: &Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        states.push(id);
        match &program.states[id.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => pending.extend(arms.iter().map(|arm| arm.target)),
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
}

pub(super) fn collect_pattern_slot(
    pattern: &Pattern,
    slots: &mut HashMap<ValueId, Slot>,
    types: Types,
) -> Option<()> {
    match pattern {
        Pattern::Binding { id, ty } if types.value(ty).is_some() => {
            insert_slot(slots, *id, ty.clone())
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_slot(element, slots, types)?;
            }
        }
        Pattern::Wildcard { .. } => {}
        _ => return None,
    }
    Some(())
}

pub(super) fn insert_slot(slots: &mut HashMap<ValueId, Slot>, id: ValueId, ty: Type) {
    if !slots.contains_key(&id) {
        slots.insert(
            id,
            Slot {
                index: slots.len(),
                ty,
            },
        );
    }
}

pub(super) fn pattern_value_type(pattern: &Pattern) -> Option<&Type> {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => Some(ty),
    }
}
