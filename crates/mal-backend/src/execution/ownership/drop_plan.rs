use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::control::ast::{StateId, Terminator};

use super::super::{ControlCallMode, ControlCallPlan, ControlFramePlan};
use super::borrow::BorrowPlan;
use super::identity::{ControlPath, EdgeId, UseEffect, UseId, UseLocation};
use super::liveness::local_binding_order;
use super::operand::{terminator_argument, terminator_operands};

pub(super) fn collect_edge_drops(
    control: &crate::control::ast::Program,
    calls: &ControlCallPlan,
    frames: &ControlFramePlan,
    borrows: &BorrowPlan,
    live_in: &[HashSet<ValueId>],
    uses: &HashMap<UseId, UseEffect>,
    borrowed_bindings: &HashSet<ValueId>,
) -> HashMap<EdgeId, Vec<ValueId>> {
    let local_order = local_binding_order(control);
    // Drops follow program order, so an edge that drops several values releases them in the same order everywhere.
    let mut order = HashMap::new();
    for binding in local_order {
        if !borrowed_bindings.contains(&binding) {
            let position = order.len();
            order.entry(binding).or_insert(position);
        }
    }
    let mut result = HashMap::new();
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        let live = borrows.terminator_live(&state.terminator, live_in);
        let effective_argument = calls
            .forwarded_self_argument(site)
            .or_else(|| terminator_argument(&state.terminator));
        let mut consumed_at_terminator = terminator_operands(&state.terminator, effective_argument)
            .into_iter()
            .filter_map(|(operand, atom, _)| {
                (uses.get(&UseId {
                    state: site,
                    location: UseLocation::Terminator(operand),
                }) == Some(&UseEffect::Consume))
                .then(|| atom.binding())
                .flatten()
            })
            .collect::<HashSet<_>>();
        if let Some(frame) = frames.frame(site) {
            for (field_index, field) in frame.fields.iter().enumerate() {
                if uses.get(&UseId {
                    state: site,
                    location: UseLocation::FrameField(field_index),
                }) == Some(&UseEffect::Consume)
                {
                    consumed_at_terminator.insert(field.id);
                }
            }
        }
        for (path, successor) in control_paths(site, &state.terminator, calls, frames) {
            let mut consumed = consumed_at_terminator.clone();
            if let (ControlPath::CaseArm(arm), Terminator::Case { scrutinee, .. }) =
                (path, &state.terminator)
                && uses.get(&UseId {
                    state: site,
                    location: UseLocation::CasePayload(arm),
                }) == Some(&UseEffect::Consume)
                && let Some(id) = scrutinee.binding()
            {
                consumed.insert(id);
            }
            let survivors = successor.map(|target| &live_in[target.0]);
            let mut drops = live
                .iter()
                .filter_map(|id| order.get(id).map(|position| (*position, *id)))
                .filter(|(_, id)| {
                    !consumed.contains(id) && survivors.is_none_or(|values| !values.contains(id))
                })
                .collect::<Vec<_>>();
            drops.sort_unstable_by_key(|(position, _)| *position);
            let drops = drops.into_iter().map(|(_, id)| id).collect::<Vec<_>>();
            if !drops.is_empty() {
                result.insert(EdgeId { state: site, path }, drops);
            }
        }
    }
    result
}

fn control_paths(
    site: StateId,
    terminator: &Terminator,
    calls: &ControlCallPlan,
    frames: &ControlFramePlan,
) -> Vec<(ControlPath, Option<StateId>)> {
    match terminator {
        Terminator::Goto(target) | Terminator::Jump { target, .. } => {
            vec![(ControlPath::Single, Some(*target))]
        }
        Terminator::Call { resume, .. }
            if frames.frame(site).is_none()
                && matches!(
                    calls.mode(site),
                    Some(ControlCallMode::Direct(_) | ControlCallMode::Dispatch)
                ) =>
        {
            vec![(ControlPath::Single, Some(*resume))]
        }
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => vec![
            (ControlPath::BranchOtherwise, Some(*otherwise)),
            (ControlPath::BranchThen, Some(*then)),
        ],
        Terminator::Case { arms, .. } => arms
            .iter()
            .enumerate()
            .map(|(arm, value)| (ControlPath::CaseArm(arm), Some(value.target)))
            .collect(),
        Terminator::Call { .. } | Terminator::Return(_) | Terminator::TailCall { .. } => {
            vec![(ControlPath::Single, None)]
        }
    }
}
