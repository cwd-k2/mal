//! Which native call sites hand their managed argument to the callee, and which functions receive it that way.
//!
//! A parameter the callee keeps (returns, stores, or captures) has to be owned by the callee. Passing it borrowed
//! makes the callee take a reference of its own on entry while the caller releases its reference after the call, a
//! pair a caller that no longer needs the value could avoid by moving it. Every function that one call site may
//! target has to accept the same convention, so functions are grouped by the call sites they share (through the
//! closure flow) and a group passes its argument owned when any member keeps it. A member that would only borrow
//! it releases the value on entry, which the borrowed convention avoided.
//!
//! Regions, the process entry, and functions reached by a call the region machinery executes keep the borrowed
//! convention, and so does every group that contains one of them.

use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{FunctionId, Pattern};
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::super::derived::{close, derived_from, managed_leaves};
use super::super::{
    ApplicationGraph, ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan,
};
use super::managed::is_managed;

#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct OwnedConvention {
    pub(super) functions: HashSet<FunctionId>,
    pub(super) sites: HashSet<StateId>,
}

impl OwnedConvention {
    pub(super) fn new(
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
        regions: &ControlRegionPlan,
        frames: &ControlFramePlan,
    ) -> Self {
        let mut groups = Groups::default();
        let mut pinned = HashSet::<FunctionId>::new();
        pinned.extend(control.entry);
        for function in &control.functions {
            if regions.function_region(function.id).is_some() {
                pinned.insert(function.id);
            }
        }
        let is_native_site = |site: StateId| match calls.mode(site) {
            Some(ControlCallMode::Direct(_)) => frames.frame(site).is_none(),
            Some(ControlCallMode::Dispatch) => {
                frames.frame(site).is_none()
                    && !regions
                        .site_region(site)
                        .is_some_and(|region| calls.requires_common_control(region))
            }
            _ => false,
        };
        let mut native_sites = Vec::new();
        for (site, _) in applications.sites() {
            let Some(targets) = applications.targets(site) else {
                continue;
            };
            if is_native_site(site) {
                groups.join(targets);
                native_sites.push(site);
            } else if matches!(
                calls.mode(site),
                Some(ControlCallMode::Direct(_) | ControlCallMode::Dispatch)
            ) {
                pinned.extend(targets.iter().copied());
            }
        }
        let keeps = kept_parameters(control, applications);
        let mut owned_roots = HashSet::new();
        let mut pinned_roots = HashSet::new();
        for function in &control.functions {
            let root = groups.root(function.id);
            if keeps.contains(&function.id) {
                owned_roots.insert(root);
            }
            if pinned.contains(&function.id) {
                pinned_roots.insert(root);
            }
        }
        let functions = control
            .functions
            .iter()
            .map(|function| function.id)
            .filter(|id| {
                let root = groups.root(*id);
                owned_roots.contains(&root) && !pinned_roots.contains(&root)
            })
            .collect::<HashSet<_>>();
        let sites = native_sites
            .into_iter()
            .filter(|site| {
                applications.targets(*site).is_some_and(|targets| {
                    !targets.is_empty() && targets.iter().all(|t| functions.contains(t))
                })
            })
            .collect();
        Self { functions, sites }
    }
}

/// Union-find over functions that share a call site.
#[derive(Default)]
struct Groups {
    parent: HashMap<FunctionId, FunctionId>,
}

impl Groups {
    fn root(&mut self, function: FunctionId) -> FunctionId {
        let mut current = function;
        while let Some(&parent) = self.parent.get(&current) {
            if parent == current {
                break;
            }
            current = parent;
        }
        let root = current;
        let mut current = function;
        while current != root {
            let next = self.parent.insert(current, root).unwrap_or(root);
            current = next;
        }
        root
    }

    fn join(&mut self, targets: &[FunctionId]) {
        let Some((&first, rest)) = targets.split_first() else {
            return;
        };
        let first = self.root(first);
        for target in rest {
            let root = self.root(*target);
            if root != first {
                self.parent.insert(root, first);
            }
        }
    }
}

/// The functions that keep their managed parameter: they return it, capture it, or hand it to a callee that keeps
/// its own. The fixed point follows a parameter that only passes through a chain of calls to the function that
/// finally keeps it.
fn kept_parameters(control: &Program, applications: &ApplicationGraph) -> HashSet<FunctionId> {
    let candidates = control
        .functions
        .iter()
        .filter_map(|function| {
            let binding = function.parameter.binding?;
            if !is_managed(&function.parameter.ty) {
                return None;
            }
            let states = &function.states;
            let mut derived = HashSet::new();
            managed_leaves(
                &Pattern::Binding {
                    id: binding,
                    ty: function.parameter.ty.clone(),
                },
                &mut derived,
            );
            close(control, states, &mut derived);
            Some((function.id, states, derived))
        })
        .collect::<Vec<_>>();
    let mut kept = HashSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for (function, states, derived) in &candidates {
            if !kept.contains(function)
                && states
                    .iter()
                    .any(|site| escapes(control, applications, *site, derived, &kept))
            {
                kept.insert(*function);
                changed = true;
            }
        }
    }
    kept
}

fn escapes(
    control: &Program,
    applications: &ApplicationGraph,
    site: StateId,
    derived: &HashSet<ValueId>,
    kept: &HashSet<FunctionId>,
) -> bool {
    let state = &control.states[site.0];
    let captured = state.bindings.iter().any(|binding| {
        matches!(&binding.operation, Operation::MakeClosure { captures, .. }
            if captures.iter().any(|atom| derived_from(atom, derived)))
    });
    captured
        || match &state.terminator {
            Terminator::Return(atom) => derived_from(atom, derived),
            Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => {
                derived_from(argument, derived)
                    && applications
                        .targets(site)
                        .is_some_and(|targets| targets.iter().any(|target| kept.contains(target)))
            }
            _ => false,
        }
}
