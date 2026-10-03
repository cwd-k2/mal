use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, FunctionId, Pattern, Program, Reference, TopLevelPattern,
};
use crate::control::ast::{self as control, StateId, Terminator};
use crate::core::ast::BufferOperation;
use crate::flow::ClosureFlow;
use mal_frontend::check::ast::Type;

use super::plan::{ClosureSource, ClosureUse, Scope};

pub(super) type Sources = HashSet<ClosureSource>;

pub(super) struct Provenance {
    sites: HashMap<StateId, Vec<ClosureSource>>,
    uses: Vec<ClosureUse>,
}

impl Provenance {
    pub(super) fn new(
        closure: &Program,
        control: &control::Program,
        flow: &ClosureFlow,
        relevant: &HashSet<FunctionId>,
    ) -> Self {
        let mut analysis = Analysis::new(closure, control, flow);
        while std::mem::take(&mut analysis.changed) {
            analysis.pass();
        }
        let sites = control
            .states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                let site = StateId(index);
                let owner = analysis.owners[index]?;
                let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
                    &state.terminator
                else {
                    return None;
                };
                let allowed = flow.callee(site)?;
                let mut sources = analysis
                    .atom(owner, callee)
                    .into_iter()
                    .filter(|source| {
                        allowed.contains(&source.function())
                            && relevant.contains(&source.function())
                    })
                    .collect::<Vec<_>>();
                sources.sort_by_key(|source| source_key(*source));
                Some((site, sources))
            })
            .collect();
        let uses = super::use_sites::collect(control, &analysis, relevant);
        Self { sites, uses }
    }

    pub(super) fn at(&self, site: StateId) -> Vec<ClosureSource> {
        self.sites.get(&site).cloned().unwrap_or_default()
    }

    pub(super) fn uses(&self) -> &[ClosureUse] {
        &self.uses
    }
}

pub(super) struct Analysis<'a> {
    control: &'a control::Program,
    flow: &'a ClosureFlow,
    owners: Vec<Option<Scope>>,
    parameters: HashMap<FunctionId, Option<ValueId>>,
    parameter_types: HashMap<FunctionId, Type>,
    signatures: HashMap<FunctionId, (Type, Type)>,
    values: HashMap<ValueId, Sources>,
    captures: HashMap<(FunctionId, usize), Sources>,
    returns: HashMap<Scope, Sources>,
    buffer_elements: Sources,
    changed: bool,
}

impl<'a> Analysis<'a> {
    fn new(closure: &Program, control: &'a control::Program, flow: &'a ClosureFlow) -> Self {
        Self {
            control,
            flow,
            owners: owners(control),
            parameters: control
                .functions
                .iter()
                .map(|function| (function.id, function.parameter.binding))
                .collect(),
            parameter_types: control
                .functions
                .iter()
                .map(|function| (function.id, function.parameter.ty.clone()))
                .collect(),
            signatures: closure
                .functions
                .iter()
                .map(|function| {
                    (
                        function.id,
                        (
                            function.parameter.ty.clone(),
                            function.body.result.ty.clone(),
                        ),
                    )
                })
                .collect(),
            values: HashMap::new(),
            captures: HashMap::new(),
            returns: HashMap::new(),
            buffer_elements: Sources::new(),
            changed: true,
        }
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
            if let Some(result) = self.returns.get(&Scope::TopLevel(index)).cloned() {
                self.assign_top_level(&binding.pattern, &result);
            }
        }
    }

    fn operation(&mut self, owner: Scope, pattern: &Pattern, operation: &control::Operation) {
        match operation {
            control::Operation::Atom(atom)
            | control::Operation::SumInjection { value: atom, .. } => {
                let value = self.atom(owner, atom);
                self.assign(pattern, &value);
            }
            control::Operation::Product(atoms) => {
                let value = atoms
                    .iter()
                    .flat_map(|atom| self.atom(owner, atom))
                    .collect();
                self.assign(pattern, &value);
            }
            control::Operation::MakeClosure { function, captures } => {
                for (index, capture) in captures.iter().enumerate() {
                    let value = self.atom(owner, capture);
                    self.merge_capture((*function, index), &value);
                }
                let source = match pattern {
                    Pattern::Binding { id, .. } => ClosureSource::Creator {
                        binding: *id,
                        function: *function,
                    },
                    Pattern::Product { .. } | Pattern::Wildcard { .. } => {
                        ClosureSource::Unbound(*function)
                    }
                };
                self.assign(pattern, &Sources::from([source]));
            }
            control::Operation::Buffer {
                operation: BufferOperation::Get,
                ..
            } => {
                let elements = self.buffer_elements.clone();
                self.assign(pattern, &elements);
            }
            control::Operation::Buffer { operands, .. } => {
                for operand in operands {
                    let value = self.atom(owner, operand);
                    self.changed |= merge(&mut self.buffer_elements, &value);
                }
            }
            control::Operation::Symbol { .. }
            | control::Operation::Memory { .. }
            | control::Operation::ExternalCall { .. }
            | control::Operation::NumericConversion { .. }
            | control::Operation::PrimitiveUnary { .. }
            | control::Operation::PrimitiveBinary { .. } => {}
        }
    }

    fn terminator(&mut self, site: StateId, owner: Scope, terminator: &Terminator) {
        match terminator {
            Terminator::Return(atom) => {
                let value = self.atom(owner, atom);
                self.add_return(owner, &value);
            }
            Terminator::Jump { target, value } => {
                let value = self.atom(owner, value);
                self.assign_input(*target, &value);
            }
            Terminator::Case { scrutinee, arms } => {
                let value = self.atom(owner, scrutinee);
                for arm in arms {
                    self.assign_input(arm.target, &value);
                }
            }
            Terminator::Call {
                argument, resume, ..
            } => {
                let argument = self.atom(owner, argument);
                let mut result = Sources::new();
                for target in self.targets(site, owner) {
                    self.assign_parameter(target, &argument);
                    result.extend(
                        self.returns
                            .get(&Scope::Function(target))
                            .into_iter()
                            .flatten(),
                    );
                }
                self.assign_input(*resume, &result);
            }
            Terminator::TailCall { argument, .. } => {
                let argument = self.atom(owner, argument);
                for target in self.targets(site, owner) {
                    self.assign_parameter(target, &argument);
                    if let Some(result) = self.returns.get(&Scope::Function(target)).cloned() {
                        self.add_return(owner, &result);
                    }
                }
            }
            Terminator::Goto(_) | Terminator::PrimitiveBranch { .. } => {}
        }
    }

    fn targets(&self, site: StateId, owner: Scope) -> HashSet<FunctionId> {
        let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
            &self.control.states[site.0].terminator
        else {
            return HashSet::new();
        };
        let Some(allowed) = self.flow.callee(site) else {
            return HashSet::new();
        };
        self.atom(owner, callee)
            .into_iter()
            .map(ClosureSource::function)
            .filter(|function| allowed.contains(function))
            .collect()
    }

    pub(super) fn scope(&self, state: usize) -> Option<Scope> {
        self.owners[state]
    }

    pub(super) fn atom(&self, owner: Scope, atom: &Atom) -> Sources {
        match atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => {
                self.values.get(&id).cloned().unwrap_or_default()
            }
            AtomKind::Reference(Reference::Capture(index)) => match owner {
                Scope::Function(function) => self
                    .captures
                    .get(&(function, index))
                    .cloned()
                    .unwrap_or_default(),
                Scope::TopLevel(_) => Sources::new(),
            },
            AtomKind::Reference(Reference::SelfClosure(function)) => {
                Sources::from([ClosureSource::SelfClosure(function)])
            }
            AtomKind::Integer(_) | AtomKind::Float(_) | AtomKind::Symbol(_) | AtomKind::Unit => {
                Sources::new()
            }
        }
    }

    fn assign_parameter(&mut self, function: FunctionId, value: &Sources) {
        if let Some(Some(binding)) = self.parameters.get(&function).copied() {
            let ty = self.parameter_types[&function].clone();
            self.assign_binding(binding, &ty, value);
        }
    }

    fn assign_input(&mut self, target: StateId, value: &Sources) {
        if let Some(input) = self.control.states[target.0].input.clone() {
            self.assign(&input, value);
        }
    }

    fn assign(&mut self, pattern: &Pattern, value: &Sources) {
        match pattern {
            Pattern::Binding { id, ty } => self.assign_binding(*id, ty, value),
            Pattern::Product { elements, .. } => {
                for element in elements {
                    self.assign(element, value);
                }
            }
            Pattern::Wildcard { .. } => {}
        }
    }

    fn assign_top_level(&mut self, pattern: &TopLevelPattern, value: &Sources) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } => self.assign_binding(*id, ty, value),
            TopLevelPattern::Product { elements, .. } => {
                for element in elements {
                    self.assign_top_level(element, value);
                }
            }
            TopLevelPattern::Wildcard { .. } => {}
        }
    }

    fn merge_value(&mut self, id: ValueId, value: &Sources) {
        self.changed |= merge(self.values.entry(id).or_default(), value);
    }

    fn assign_binding(&mut self, id: ValueId, ty: &Type, value: &Sources) {
        let held = match ty {
            Type::Function { parameter, result } => value
                .iter()
                .copied()
                .filter(|source| {
                    self.signatures
                        .get(&source.function())
                        .is_some_and(|(p, r)| p == parameter.as_ref() && r == result.as_ref())
                })
                .collect(),
            _ if crate::flow::holds_function(ty) => value.clone(),
            _ => Sources::new(),
        };
        if !held.is_empty() {
            self.merge_value(id, &held);
        }
    }

    fn merge_capture(&mut self, capture: (FunctionId, usize), value: &Sources) {
        self.changed |= merge(self.captures.entry(capture).or_default(), value);
    }

    fn add_return(&mut self, owner: Scope, value: &Sources) {
        self.changed |= merge(self.returns.entry(owner).or_default(), value);
    }
}

fn owners(control: &control::Program) -> Vec<Option<Scope>> {
    let mut owners = vec![None; control.states.len()];
    let entries = control
        .functions
        .iter()
        .map(|function| (&function.states, Scope::Function(function.id)))
        .chain(
            control
                .bindings
                .iter()
                .enumerate()
                .map(|(index, binding)| (&binding.states, Scope::TopLevel(index))),
        );
    for (states, owner) in entries {
        for StateId(state) in states {
            owners[*state] = Some(owner);
        }
    }
    owners
}

fn merge(target: &mut Sources, added: &Sources) -> bool {
    let before = target.len();
    target.extend(added.iter().copied());
    target.len() != before
}

pub(super) fn source_key(source: ClosureSource) -> (u32, u8, u64) {
    let function = super::order::function(source.function());
    match source {
        ClosureSource::Creator { binding, .. } => (function, 0, super::order::value(binding)),
        ClosureSource::SelfClosure(_) => (function, 1, 0),
        ClosureSource::Unbound(_) => (function, 2, 0),
    }
}
