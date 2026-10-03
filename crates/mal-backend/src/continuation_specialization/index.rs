use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomId, AtomKind, FunctionId, Operation, Pattern, Program, Reference,
};

use super::plan::{Demand, ProducerStep};

pub(super) fn analyze(program: &Program) -> (Vec<Demand>, Vec<ProducerStep>) {
    let index = Index::new(program);
    let demands = index.demands(program);
    let steps = super::trace::producer_steps(&index, program, &demands);
    (demands, steps)
}

#[derive(Clone, Copy)]
enum Use {
    Alias,
    Callee(AtomId),
    Escape,
}

pub(super) struct Index {
    pub(super) definitions: HashMap<ValueId, Operation>,
    aliases: HashMap<ValueId, ValueId>,
    uses: HashMap<ValueId, Vec<Use>>,
    functions: HashMap<ValueId, FunctionId>,
}

impl Index {
    fn new(program: &Program) -> Self {
        let mut definitions = HashMap::new();
        let mut direct_aliases = HashMap::new();
        for_each_block(program, &mut |block| {
            for binding in &block.bindings {
                let Pattern::Binding { id, .. } = binding.pattern else {
                    continue;
                };
                definitions.insert(id, binding.operation.clone());
                if let Operation::Atom(atom) = &binding.operation
                    && let Some(source) = atom.binding()
                {
                    direct_aliases.insert(id, source);
                }
            }
        });
        let aliases = canonical_aliases(&direct_aliases);
        let mut index = Self {
            definitions,
            aliases,
            uses: HashMap::new(),
            functions: program
                .bindings
                .iter()
                .filter_map(|binding| binding.known_function())
                .collect(),
        };
        for_each_block(program, &mut |block| index.index_block(block));
        index
    }

    fn demands(&self, program: &Program) -> Vec<Demand> {
        let mut demands = Vec::new();
        for_each_block(program, &mut |block| {
            for (consumer_index, binding) in block.bindings.iter().enumerate() {
                let Operation::Call { callee, argument } = &binding.operation else {
                    continue;
                };
                let Some(binding) = callee.binding() else {
                    continue;
                };
                let origin = self.origin(binding);
                let Some(Operation::Call {
                    callee: producer_callee,
                    argument: producer_argument,
                }) = self.definitions.get(&origin)
                else {
                    continue;
                };
                let Some(producer) = self.function(producer_callee) else {
                    continue;
                };
                let Some(producer_index) = block.bindings.iter().position(
                    |binding| matches!(binding.pattern, Pattern::Binding { id, .. } if id == origin),
                ) else {
                    continue;
                };
                if producer_index >= consumer_index
                    || !block.bindings[producer_index + 1..consumer_index]
                        .iter()
                        .all(|binding| matches!(binding.operation, Operation::Atom(_)))
                    || !argument_available_before(block, argument, producer_index)
                {
                    continue;
                }
                let uses = self.uses.get(&origin).map_or(&[][..], Vec::as_slice);
                if uses.iter().any(|usage| matches!(usage, Use::Escape)) {
                    continue;
                }
                let consumers = uses
                    .iter()
                    .filter_map(|usage| match usage {
                        Use::Callee(site) => Some(*site),
                        Use::Alias | Use::Escape => None,
                    })
                    .collect::<Vec<_>>();
                if consumers.as_slice() == [callee.id] {
                    demands.push(Demand {
                        producer_result: origin,
                        producer,
                        producer_site: producer_callee.id,
                        producer_argument: producer_argument.clone(),
                        consumer: callee.id,
                        argument: argument.clone(),
                    });
                }
            }
        });
        demands
    }

    fn index_block(&mut self, block: &crate::closure::ast::Block) {
        for binding in &block.bindings {
            let alias_source = match (&binding.pattern, &binding.operation) {
                (Pattern::Binding { id, .. }, Operation::Atom(atom)) => {
                    atom.binding().map(|source| (*id, source))
                }
                _ => None,
            };
            match &binding.operation {
                Operation::Call { callee, argument } => {
                    self.record(callee, Use::Callee(callee.id));
                    self.record(argument, Use::Escape);
                }
                operation => operation.for_each_atom(|atom| {
                    let usage = if alias_source
                        .is_some_and(|(_, source)| atom.binding().is_some_and(|id| id == source))
                    {
                        Use::Alias
                    } else {
                        Use::Escape
                    };
                    self.record(atom, usage);
                }),
            }
        }
        self.record(&block.result, Use::Escape);
    }

    fn record(&mut self, atom: &Atom, usage: Use) {
        let Some(binding) = atom.binding() else {
            return;
        };
        let origin = self.origin(binding);
        self.uses.entry(origin).or_default().push(usage);
    }

    pub(super) fn origin(&self, binding: ValueId) -> ValueId {
        self.aliases.get(&binding).copied().unwrap_or(binding)
    }

    pub(super) fn function(&self, callee: &Atom) -> Option<FunctionId> {
        match callee.kind {
            AtomKind::Reference(Reference::Binding(binding)) => {
                self.functions.get(&self.origin(binding)).copied()
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => Some(function),
            _ => None,
        }
    }
}

fn argument_available_before(
    block: &crate::closure::ast::Block,
    argument: &Atom,
    producer_index: usize,
) -> bool {
    let Some(argument) = argument.binding() else {
        return true;
    };
    block
        .bindings
        .iter()
        .position(|binding| pattern_binds(&binding.pattern, argument))
        .is_none_or(|position| position < producer_index)
}

fn pattern_binds(pattern: &Pattern, sought: ValueId) -> bool {
    match pattern {
        Pattern::Binding { id, .. } => *id == sought,
        Pattern::Product { elements, .. } => elements
            .iter()
            .any(|element| pattern_binds(element, sought)),
        Pattern::Wildcard { .. } => false,
    }
}

fn canonical_aliases(direct: &HashMap<ValueId, ValueId>) -> HashMap<ValueId, ValueId> {
    let mut aliases = HashMap::new();
    for start in direct.keys().copied() {
        let mut current = start;
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        while seen.insert(current) {
            path.push(current);
            let Some(next) = direct.get(&current).copied() else {
                for binding in path {
                    aliases.insert(binding, current);
                }
                break;
            };
            current = next;
        }
    }
    aliases
}

fn for_each_block(program: &Program, visit: &mut impl FnMut(&crate::closure::ast::Block)) {
    fn descendants(
        block: &crate::closure::ast::Block,
        visit: &mut impl FnMut(&crate::closure::ast::Block),
    ) {
        visit(block);
        for binding in &block.bindings {
            binding
                .operation
                .for_each_nested_block(|nested| descendants(nested, visit));
        }
    }

    for binding in &program.bindings {
        descendants(&binding.value, visit);
    }
    for function in &program.functions {
        descendants(&function.body, visit);
        for join in &function.joins {
            descendants(&join.body, visit);
        }
    }
}
