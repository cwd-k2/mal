//! Explicit control-state construction from closure-converted blocks and lexical joins.

use crate::closure::ast::{self as closure, Atom, AtomKind, Pattern, Reference};

pub(crate) mod ast;
mod block;
mod forwarding;
mod graph;
mod liveness;
mod operation;

use self::ast::{
    Binding, CaseArm, Function, Operation, Program, State, StateId, Terminator, TopLevelBinding,
};
use self::graph::reachable_states;
pub(crate) use self::liveness::binding_use_counts;
use self::liveness::local_values;

/// Builds the explicit state graph and computes liveness only after forwarding continuations have been normalized.
pub(crate) fn lower(program: &closure::Program) -> Program {
    Lowerer::new().lower_program(program)
}

struct Lowerer {
    states: Vec<State>,
    joins: Vec<StateId>,
}

#[derive(Clone, Copy)]
enum Destination {
    Return,
    Jump(StateId),
}

impl Lowerer {
    fn new() -> Self {
        Self {
            states: Vec::new(),
            joins: Vec::new(),
        }
    }

    fn lower_program(mut self, program: &closure::Program) -> Program {
        let bindings = program
            .bindings
            .iter()
            .map(|binding| {
                self.joins.clear();
                let locals = local_values(&binding.value, None, &[]);
                let start = self.states.len();
                let entry = self.lower_block(&binding.value, Destination::Return);
                // Forwarding changes call terminators and therefore the successor graph that liveness must inspect.
                forwarding::normalize_calls(&mut self.states, start);
                self.resolve_liveness(start, &locals);
                TopLevelBinding {
                    pattern: binding.pattern.clone(),
                    entry,
                    states: Vec::new(),
                    span: binding.span,
                }
            })
            .collect();
        let functions = program
            .functions
            .iter()
            .map(|function| {
                self.joins.clear();
                let locals =
                    local_values(&function.body, Some(&function.parameter), &function.joins);
                let start = self.states.len();
                for join in &function.joins {
                    let entry = self.lower_block(&join.body, Destination::Return);
                    debug_assert!(self.states[entry.0].input.is_none());
                    self.states[entry.0].input = Some(join.parameter.clone());
                    self.joins.push(entry);
                }
                let entry = self.lower_block(&function.body, Destination::Return);
                // Keep this before liveness for the same reason as top-level initializers above.
                forwarding::normalize_calls(&mut self.states, start);
                self.resolve_liveness(start, &locals);
                Function {
                    id: function.id,
                    parameter: function.parameter.clone(),
                    entry,
                    states: Vec::new(),
                }
            })
            .collect();
        let mut lowered = Program {
            bindings,
            functions,
            entry: program.entry.map(|entry| entry.function),
            states: self.states,
            span: program.span,
        };
        for index in 0..lowered.bindings.len() {
            lowered.bindings[index].states =
                reachable_states(&lowered, lowered.bindings[index].entry);
        }
        for index in 0..lowered.functions.len() {
            lowered.functions[index].states =
                reachable_states(&lowered, lowered.functions[index].entry);
        }
        lowered
    }
}
