use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::compatible::CompatibleTargets;
use super::owner::{Owner, state_owners};

pub(super) type Functions = HashSet<FunctionId>;

pub(super) fn solve(
    closure: &closure::Program,
    control: &control::Program,
    compatible: &mut CompatibleTargets,
) -> HashMap<StateId, Functions> {
    let mut analysis = Analysis::new(closure, control, compatible);
    while std::mem::take(&mut analysis.changed) {
        analysis.pass();
    }
    analysis.callees()
}

pub(super) struct Analysis<'a> {
    pub(super) control: &'a control::Program,
    pub(super) signatures: HashMap<FunctionId, (&'a Type, &'a Type)>,
    pub(super) parameters: HashMap<FunctionId, (Option<ValueId>, &'a Type)>,
    pub(super) owners: Vec<Option<Owner>>,
    pub(super) compatible: HashMap<StateId, Vec<FunctionId>>,
    pub(super) values: HashMap<ValueId, Functions>,
    pub(super) captures: HashMap<(FunctionId, usize), Functions>,
    pub(super) returns: HashMap<Owner, Functions>,
    pub(super) buffer_elements: Functions,
    pub(super) changed: bool,
}

impl<'a> Analysis<'a> {
    fn new(
        closure: &'a closure::Program,
        control: &'a control::Program,
        compatible: &mut CompatibleTargets,
    ) -> Self {
        let compatible = control
            .states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
                    &state.terminator
                else {
                    return None;
                };
                Some((StateId(index), compatible.for_callee(callee)))
            })
            .collect();
        Self {
            control,
            signatures: closure
                .functions
                .iter()
                .map(|function| {
                    (
                        function.id,
                        (&function.parameter.ty, &function.body.result.ty),
                    )
                })
                .collect(),
            parameters: control
                .functions
                .iter()
                .map(|function| {
                    (
                        function.id,
                        (function.parameter.binding, &function.parameter.ty),
                    )
                })
                .collect(),
            owners: state_owners(control),
            compatible,
            values: HashMap::new(),
            captures: HashMap::new(),
            returns: HashMap::new(),
            buffer_elements: Functions::new(),
            changed: true,
        }
    }

    /// The application sites narrowed to the functions that reach their callee; a site is omitted when none does.
    fn callees(&self) -> HashMap<StateId, Functions> {
        (0..self.control.states.len())
            .filter_map(|index| {
                let site = StateId(index);
                let reached = self.reached_targets(site)?;
                (!reached.is_empty()).then_some((site, reached))
            })
            .collect()
    }

    fn pass(&mut self) {
        for (index, state) in self.control.states.iter().enumerate() {
            let Some(owner) = self.owners[index] else {
                continue;
            };
            for binding in &state.bindings {
                self.operation(owner, &binding.pattern, &binding.operation);
            }
            self.terminator(StateId(index), owner, &state.terminator);
        }
        for (index, binding) in self.control.bindings.iter().enumerate() {
            let result = self.returns.get(&Owner::Binding(index)).cloned();
            if let Some(result) = result {
                self.assign_top_level(&binding.pattern, &result);
            }
        }
    }

    /// The application targets the flow established so far: the callee's compatible functions that reach it.
    pub(super) fn reached_targets(&self, site: StateId) -> Option<Functions> {
        let owner = self.owners[site.0]?;
        let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
            &self.control.states[site.0].terminator
        else {
            return None;
        };
        let reaching = self.atom(owner, callee);
        Some(
            self.compatible[&site]
                .iter()
                .copied()
                .filter(|function| reaching.contains(function))
                .collect(),
        )
    }

    pub(super) fn atom(&self, owner: Owner, atom: &Atom) -> Functions {
        match &atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => {
                self.values.get(id).cloned().unwrap_or_default()
            }
            AtomKind::Reference(Reference::Capture(index)) => match owner {
                Owner::Function(function) => self
                    .captures
                    .get(&(function, *index))
                    .cloned()
                    .unwrap_or_default(),
                Owner::Binding(_) => Functions::new(),
            },
            AtomKind::Reference(Reference::SelfClosure(function)) => Functions::from([*function]),
            AtomKind::Integer(_) | AtomKind::Float(_) | AtomKind::Symbol(_) | AtomKind::Unit => {
                Functions::new()
            }
        }
    }

    pub(super) fn add_return(&mut self, owner: Owner, value: &Functions) {
        let slot = self.returns.entry(owner).or_default();
        self.changed |= merge(slot, value);
    }
}

pub(super) fn merge(target: &mut Functions, added: &Functions) -> bool {
    let before = target.len();
    target.extend(added.iter().copied());
    target.len() != before
}
