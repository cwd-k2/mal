use std::collections::HashSet;

use crate::c_emit::syntax::{Block, Expr, Statement};
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Pattern};
use crate::control::ast::{self as control, StateId, Terminator};

use super::super::analysis::ControlRegionId;

pub(super) const CONTROL_STACK: &str = "mal_control";
const CONTROL_STACK_STORAGE: &str = "mal_control_storage";

pub(super) fn control_stack_field(field: &str) -> Expr {
    Expr::identifier(CONTROL_STACK).pointer_field(field)
}

pub(super) fn control_region_name(region: ControlRegionId) -> String {
    format!("control_region_{}", region.0)
}

pub(super) fn emit_control_stack_preamble(output: &mut Block, region: ControlRegionId) {
    output.push(Statement::variable(
        "MalControlStack",
        CONTROL_STACK_STORAGE,
        Some(Expr::identifier("mal_context").pointer_field(control_region_name(region))),
    ));
    output.push(Statement::variable(
        crate::c_emit::syntax::TypeName::named("MalControlStack").pointer(),
        CONTROL_STACK,
        Some(Expr::address_of(Expr::identifier(CONTROL_STACK_STORAGE))),
    ));
    output.push(Statement::assignment(
        control_stack_field("top"),
        Expr::number("0"),
    ));
    output.push(Statement::assignment(
        control_stack_field("frame"),
        Expr::number("0"),
    ));
}

pub(super) fn emit_control_stack_cache(output: &mut Block, region: ControlRegionId) {
    output.push(Statement::assignment(
        Expr::identifier("mal_context").pointer_field(control_region_name(region)),
        Expr::dereference(Expr::identifier(CONTROL_STACK)),
    ));
}

pub(super) fn state_label(state: StateId) -> String {
    format!("mal_control_state_{}", state.0)
}

pub(super) fn reachable_states(program: &control::Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(state) = pending.pop() {
        if !seen.insert(state) {
            continue;
        }
        states.push(state);
        match &program.states[state.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => {
                pending.extend(arms.iter().map(|arm| arm.target));
            }
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
}

pub(super) fn local_slots(
    program: &control::Program,
    sites: &[StateId],
    parameter: Option<crate::anf::ast::ValueId>,
) -> Vec<(crate::anf::ast::ValueId, Type)> {
    let mut slots = Vec::new();
    let mut seen = HashSet::new();
    for site in sites {
        let state = &program.states[site.0];
        if let Some(input) = &state.input {
            collect_pattern_slots(input, &mut slots, &mut seen);
        }
        for binding in &state.bindings {
            collect_pattern_slots(&binding.pattern, &mut slots, &mut seen);
        }
    }
    if let Some(parameter) = parameter {
        slots.retain(|(id, _)| *id != parameter);
    }
    slots
}

fn collect_pattern_slots(
    pattern: &Pattern,
    slots: &mut Vec<(crate::anf::ast::ValueId, Type)>,
    seen: &mut HashSet<crate::anf::ast::ValueId>,
) {
    match pattern {
        Pattern::Binding { id, ty } => {
            if seen.insert(*id) {
                slots.push((*id, ty.clone()));
            }
        }
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_slots(element, slots, seen);
            }
        }
    }
}

pub(super) fn closure_operation(operation: &control::Operation) -> closure::Operation {
    match operation {
        control::Operation::Atom(value) => closure::Operation::Atom(value.clone()),
        control::Operation::MakeClosure { function, captures } => closure::Operation::MakeClosure {
            function: *function,
            captures: captures.clone(),
        },
        control::Operation::SymbolLength { value } => closure::Operation::SymbolLength {
            value: value.clone(),
        },
        control::Operation::SymbolAt { argument } => closure::Operation::SymbolAt {
            argument: argument.clone(),
        },
        control::Operation::Memory {
            primitive,
            argument,
        } => closure::Operation::Memory {
            primitive: *primitive,
            argument: argument.clone(),
        },
        control::Operation::ExternalCall { id, argument } => closure::Operation::ExternalCall {
            id: *id,
            argument: argument.clone(),
        },
        control::Operation::NumericConversion { operand } => {
            closure::Operation::NumericConversion {
                operand: operand.clone(),
            }
        }
        control::Operation::Product(elements) => closure::Operation::Product(elements.clone()),
        control::Operation::SumInjection { index, value } => closure::Operation::SumInjection {
            index: *index,
            value: value.clone(),
        },
        control::Operation::PrimitiveUnary { operator, operand } => {
            closure::Operation::PrimitiveUnary {
                operator: *operator,
                operand: operand.clone(),
            }
        }
        control::Operation::PrimitiveBinary {
            operator,
            left,
            right,
        } => closure::Operation::PrimitiveBinary {
            operator: *operator,
            left: left.clone(),
            right: right.clone(),
        },
    }
}

pub(super) fn uint8(value: usize) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

pub(super) fn uint32(value: usize) -> Expr {
    Expr::named_call("UINT32_C", [Expr::number(value.to_string())])
}
