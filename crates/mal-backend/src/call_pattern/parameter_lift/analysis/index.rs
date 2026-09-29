//! Whole-program facts that candidate admission reads, collected in one walk per lift.

use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomId, AtomKind, FunctionId, Operation, Pattern, Program, Reference,
};

use super::super::Definition;
use super::{aliases, for_each_block, operation_atoms, origin};

pub(in crate::call_pattern::parameter_lift) struct Index<'a> {
    pub(in crate::call_pattern::parameter_lift) definitions: HashMap<ValueId, Definition>,
    pub(in crate::call_pattern::parameter_lift) aliases: HashMap<ValueId, ValueId>,
    types: HashMap<ValueId, Type>,
    calls_by_binding: HashMap<ValueId, Vec<&'a Atom>>,
    calls_by_self_closure: HashMap<FunctionId, Vec<&'a Atom>>,
    callees_by_origin: HashMap<ValueId, Vec<&'a Atom>>,
    atoms_by_origin: HashMap<ValueId, Vec<AtomId>>,
    capturing_creators: HashMap<FunctionId, HashSet<ValueId>>,
    closure_captures: HashMap<FunctionId, Vec<&'a [Atom]>>,
    self_closures: HashSet<FunctionId>,
    /// Callee atoms of calls whose callee is a binding or a self closure.
    callees: HashSet<AtomId>,
    /// Self-closure atoms that are not the callee of a call.
    self_closure_values: HashSet<FunctionId>,
}

impl<'a> Index<'a> {
    pub(in crate::call_pattern::parameter_lift) fn new(program: &'a Program) -> Self {
        let definitions = super::definitions(program);
        let aliases = aliases(&definitions);
        let mut index = Self {
            definitions,
            aliases,
            types: HashMap::new(),
            calls_by_binding: HashMap::new(),
            calls_by_self_closure: HashMap::new(),
            callees_by_origin: HashMap::new(),
            atoms_by_origin: HashMap::new(),
            capturing_creators: HashMap::new(),
            closure_captures: HashMap::new(),
            self_closures: HashSet::new(),
            callees: HashSet::new(),
            self_closure_values: HashSet::new(),
        };
        for_each_block(program, &mut |block| {
            index.atom(&block.result);
            for binding in &block.bindings {
                index.pattern(&binding.pattern);
                match &binding.operation {
                    Operation::Case { arms, .. } => {
                        for arm in arms {
                            index.pattern(&arm.pattern);
                        }
                    }
                    Operation::Call { callee, argument } => {
                        index.callees.insert(callee.id);
                        if let Some(id) = callee.binding() {
                            index.calls_by_binding.entry(id).or_default().push(argument);
                        }
                        if let AtomKind::Reference(Reference::SelfClosure(function)) = callee.kind {
                            index
                                .calls_by_self_closure
                                .entry(function)
                                .or_default()
                                .push(argument);
                        }
                        if let Some(origin) = origin(callee, &index.aliases) {
                            index
                                .callees_by_origin
                                .entry(origin)
                                .or_default()
                                .push(callee);
                        }
                    }
                    Operation::MakeClosure { function, captures } => {
                        index
                            .closure_captures
                            .entry(*function)
                            .or_default()
                            .push(captures);
                        if let Pattern::Binding { id, .. } = binding.pattern
                            && !captures.is_empty()
                        {
                            index
                                .capturing_creators
                                .entry(*function)
                                .or_default()
                                .insert(id);
                        }
                    }
                    _ => {}
                }
                operation_atoms(&binding.operation, &mut |atom| index.atom(atom));
            }
        });
        index
    }

    fn atom(&mut self, atom: &Atom) {
        if let Some(origin) = origin(atom, &self.aliases) {
            self.atoms_by_origin
                .entry(origin)
                .or_default()
                .push(atom.id);
        }
        if let AtomKind::Reference(Reference::SelfClosure(function)) = atom.kind {
            self.self_closures.insert(function);
            if !self.callees.contains(&atom.id) {
                self.self_closure_values.insert(function);
            }
        }
    }

    fn pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { id, ty } => {
                self.types.insert(*id, ty.clone());
            }
            Pattern::Product { elements, .. } => {
                for element in elements {
                    self.pattern(element);
                }
            }
            Pattern::Wildcard { .. } => {}
        }
    }

    pub(in crate::call_pattern::parameter_lift) fn binding_type(
        &self,
        id: ValueId,
    ) -> Option<&Type> {
        self.types.get(&id)
    }

    /// Arguments of every call whose callee is `binding` itself or the self closure of `function`.
    pub(in crate::call_pattern::parameter_lift) fn host_call_arguments(
        &self,
        binding: ValueId,
        function: FunctionId,
    ) -> impl Iterator<Item = &'a Atom> + '_ {
        self.calls_by_binding
            .get(&binding)
            .into_iter()
            .chain(self.calls_by_self_closure.get(&function))
            .flatten()
            .copied()
    }

    /// Callees of every call whose callee originates at `origin` through aliases.
    pub(in crate::call_pattern::parameter_lift) fn callees_from(
        &self,
        origin: ValueId,
    ) -> impl Iterator<Item = &'a Atom> + '_ {
        self.callees_by_origin
            .get(&origin)
            .into_iter()
            .flatten()
            .copied()
    }

    /// Whether an atom originating at `sought` has an identity outside `allowed`.
    pub(in crate::call_pattern::parameter_lift) fn has_unapproved_uses(
        &self,
        sought: ValueId,
        allowed: &HashSet<AtomId>,
    ) -> bool {
        self.atoms_by_origin
            .get(&sought)
            .is_some_and(|atoms| atoms.iter().any(|atom| !allowed.contains(atom)))
    }

    /// The bindings of every closure of `target` that has captures.
    pub(in crate::call_pattern::parameter_lift) fn capturing_creators(
        &self,
        target: FunctionId,
    ) -> HashSet<ValueId> {
        self.capturing_creators
            .get(&target)
            .cloned()
            .unwrap_or_default()
    }

    /// The capture lists of every closure created for `function`.
    pub(in crate::call_pattern::parameter_lift) fn closure_captures(
        &self,
        function: FunctionId,
    ) -> &[&'a [Atom]] {
        self.closure_captures
            .get(&function)
            .map_or(&[], Vec::as_slice)
    }

    /// Whether the function bound at `binding`, or its self closure, is used anywhere but as the callee of a call.
    /// Rewriting a host changes its parameter type, which only its direct calls are adjusted to.
    pub(in crate::call_pattern::parameter_lift) fn host_escapes(
        &self,
        binding: ValueId,
        function: FunctionId,
    ) -> bool {
        self.self_closure_values.contains(&function)
            || self
                .atoms_by_origin
                .get(&binding)
                .is_some_and(|atoms| atoms.iter().any(|atom| !self.callees.contains(atom)))
    }

    pub(in crate::call_pattern::parameter_lift) fn refers_to_self_closure(
        &self,
        function: FunctionId,
    ) -> bool {
        self.self_closures.contains(&function)
    }
}
