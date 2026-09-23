use crate::closure::ast::{self as closure, Atom, AtomKind, Pattern, Reference};

pub mod ast;
mod forwarding;
mod liveness;

use self::ast::{
    Binding, CaseArm, Function, Operation, Program, State, StateId, Terminator, TopLevelBinding,
};
pub(crate) use self::liveness::binding_use_counts;
use self::liveness::local_values;

pub fn lower(program: &closure::Program) -> Program {
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
                forwarding::normalize_calls(&mut self.states, start);
                self.resolve_liveness(start, &locals);
                TopLevelBinding {
                    pattern: binding.pattern.clone(),
                    entry,
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
                forwarding::normalize_calls(&mut self.states, start);
                self.resolve_liveness(start, &locals);
                Function {
                    id: function.id,
                    parameter: function.parameter.clone(),
                    entry,
                }
            })
            .collect();
        Program {
            bindings,
            functions,
            states: self.states,
            span: program.span,
        }
    }

    fn lower_block(&mut self, block: &closure::Block, destination: Destination) -> StateId {
        let mut bindings = block.bindings.as_slice();
        let mut current = if let Some(last) = bindings.last()
            && returns_binding(&block.result, &last.pattern)
        {
            match &last.operation {
                closure::Operation::Goto { target, value } => {
                    bindings = &bindings[..bindings.len() - 1];
                    let target = self.joins[target.0];
                    self.push_state(
                        None,
                        Vec::new(),
                        Terminator::Jump {
                            target,
                            value: value.clone(),
                        },
                        block.span,
                    )
                }
                closure::Operation::Call { callee, argument }
                    if matches!(destination, Destination::Return) =>
                {
                    bindings = &bindings[..bindings.len() - 1];
                    self.push_state(
                        None,
                        Vec::new(),
                        Terminator::TailCall {
                            callee: callee.clone(),
                            argument: argument.clone(),
                        },
                        block.span,
                    )
                }
                closure::Operation::Case { scrutinee, arms } => {
                    bindings = &bindings[..bindings.len() - 1];
                    self.lower_case(scrutinee, arms, destination, block.span)
                }
                closure::Operation::PrimitiveBranch {
                    operator,
                    left,
                    right,
                    otherwise,
                    then,
                } => {
                    bindings = &bindings[..bindings.len() - 1];
                    self.lower_branch(
                        *operator,
                        left,
                        right,
                        otherwise,
                        then,
                        destination,
                        block.span,
                    )
                }
                _ => self.terminal_state(&block.result, destination, block.span),
            }
        } else {
            self.terminal_state(&block.result, destination, block.span)
        };

        let mut pending = Vec::new();
        for binding in bindings.iter().rev() {
            current = match &binding.operation {
                closure::Operation::Call { callee, argument } => {
                    self.prepend_pending(current, &mut pending);
                    let resume = self.push_state(
                        Some(binding.pattern.clone()),
                        Vec::new(),
                        Terminator::Goto(current),
                        binding.span,
                    );
                    self.push_state(
                        None,
                        Vec::new(),
                        Terminator::Call {
                            callee: callee.clone(),
                            argument: argument.clone(),
                            resume,
                        },
                        binding.span,
                    )
                }
                closure::Operation::Case { scrutinee, arms } => {
                    self.prepend_pending(current, &mut pending);
                    let join = self.push_state(
                        Some(binding.pattern.clone()),
                        Vec::new(),
                        Terminator::Goto(current),
                        binding.span,
                    );
                    self.lower_case(scrutinee, arms, Destination::Jump(join), binding.span)
                }
                closure::Operation::PrimitiveBranch {
                    operator,
                    left,
                    right,
                    otherwise,
                    then,
                } => {
                    self.prepend_pending(current, &mut pending);
                    let join = self.push_state(
                        Some(binding.pattern.clone()),
                        Vec::new(),
                        Terminator::Goto(current),
                        binding.span,
                    );
                    self.lower_branch(
                        *operator,
                        left,
                        right,
                        otherwise,
                        then,
                        Destination::Jump(join),
                        binding.span,
                    )
                }
                operation => {
                    pending.push(Binding {
                        pattern: binding.pattern.clone(),
                        operation: lower_operation(operation),
                        span: binding.span,
                    });
                    current
                }
            };
        }
        self.prepend_pending(current, &mut pending);
        current
    }

    fn prepend_pending(&mut self, state: StateId, pending: &mut Vec<Binding>) {
        pending.reverse();
        pending.append(&mut self.states[state.0].bindings);
        self.states[state.0].bindings = std::mem::take(pending);
    }

    fn lower_case(
        &mut self,
        scrutinee: &Atom,
        arms: &[closure::CaseArm],
        destination: Destination,
        span: crate::source::Span,
    ) -> StateId {
        let arms = arms
            .iter()
            .map(|arm| {
                let target = self.lower_block(&arm.value, destination);
                debug_assert!(self.states[target.0].input.is_none());
                self.states[target.0].input = Some(arm.pattern.clone());
                CaseArm {
                    index: arm.index,
                    target,
                    span: arm.span,
                }
            })
            .collect();
        self.push_state(
            None,
            Vec::new(),
            Terminator::Case {
                scrutinee: scrutinee.clone(),
                arms,
            },
            span,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_branch(
        &mut self,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &closure::Block,
        then: &closure::Block,
        destination: Destination,
        span: crate::source::Span,
    ) -> StateId {
        let otherwise = self.lower_block(otherwise, destination);
        let then = self.lower_block(then, destination);
        self.push_state(
            None,
            Vec::new(),
            Terminator::PrimitiveBranch {
                operator,
                left: left.clone(),
                right: right.clone(),
                otherwise,
                then,
            },
            span,
        )
    }

    fn terminal_state(
        &mut self,
        result: &Atom,
        destination: Destination,
        span: crate::source::Span,
    ) -> StateId {
        let terminator = match destination {
            Destination::Return => Terminator::Return(result.clone()),
            Destination::Jump(target) => Terminator::Jump {
                target,
                value: result.clone(),
            },
        };
        self.push_state(None, Vec::new(), terminator, span)
    }

    fn push_state(
        &mut self,
        input: Option<Pattern>,
        bindings: Vec<Binding>,
        terminator: Terminator,
        span: crate::source::Span,
    ) -> StateId {
        let id = StateId(self.states.len());
        self.states.push(State {
            input,
            live: Vec::new(),
            needs_environment: false,
            bindings,
            terminator,
            span,
        });
        id
    }
}

fn returns_binding(result: &Atom, pattern: &Pattern) -> bool {
    matches!(
        (&result.kind, pattern),
        (
            AtomKind::Reference(Reference::Binding(result)),
            Pattern::Binding { id, .. }
        ) if result == id
    )
}

fn lower_operation(operation: &closure::Operation) -> Operation {
    match operation {
        closure::Operation::Atom(value) => Operation::Atom(value.clone()),
        closure::Operation::MakeClosure { function, captures } => Operation::MakeClosure {
            function: *function,
            captures: captures.clone(),
        },
        closure::Operation::SymbolLength { value } => Operation::SymbolLength {
            value: value.clone(),
        },
        closure::Operation::SymbolAt { argument } => Operation::SymbolAt {
            argument: argument.clone(),
        },
        closure::Operation::Memory {
            primitive,
            operands,
        } => Operation::Memory {
            primitive: *primitive,
            operands: operands.clone(),
        },
        closure::Operation::Buffer {
            operation,
            element,
            operands,
        } => Operation::Buffer {
            operation: *operation,
            element: element.clone(),
            operands: operands.clone(),
        },
        closure::Operation::ExternalCall { id, argument } => Operation::ExternalCall {
            id: *id,
            argument: argument.clone(),
        },
        closure::Operation::NumericConversion { operand } => Operation::NumericConversion {
            operand: operand.clone(),
        },
        closure::Operation::Product(elements) => Operation::Product(elements.clone()),
        closure::Operation::SumInjection { index, value } => Operation::SumInjection {
            index: *index,
            value: value.clone(),
        },
        closure::Operation::PrimitiveUnary { operator, operand } => Operation::PrimitiveUnary {
            operator: *operator,
            operand: operand.clone(),
        },
        closure::Operation::PrimitiveBinary {
            operator,
            left,
            right,
        } => Operation::PrimitiveBinary {
            operator: *operator,
            left: left.clone(),
            right: right.clone(),
        },
        closure::Operation::Call { .. }
        | closure::Operation::Goto { .. }
        | closure::Operation::Case { .. }
        | closure::Operation::PrimitiveBranch { .. } => {
            unreachable!("control operations become terminators")
        }
    }
}
