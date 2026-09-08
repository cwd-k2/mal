use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Pattern, Reference};

pub mod ast;

use self::ast::{
    Binding, CaseArm, Function, LiveValue, Operation, Program, State, StateId, Terminator,
    TopLevelBinding,
};

pub fn lower(program: &closure::Program) -> Program {
    Lowerer::new().lower_program(program)
}

struct Lowerer {
    states: Vec<State>,
}

#[derive(Clone, Copy)]
enum Destination {
    Return,
    Jump(StateId),
}

impl Lowerer {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn lower_program(mut self, program: &closure::Program) -> Program {
        let bindings = program
            .bindings
            .iter()
            .map(|binding| {
                let locals = local_values(&binding.value, None);
                let start = self.states.len();
                let entry = self.lower_block(&binding.value, Destination::Return);
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
                let locals = local_values(&function.body, Some(&function.parameter));
                let start = self.states.len();
                let entry = self.lower_block(&function.body, Destination::Return);
                self.resolve_liveness(start, &locals);
                Function {
                    id: function.id,
                    environment: function.environment.clone(),
                    parameter: function.parameter.clone(),
                    entry,
                }
            })
            .collect();
        Program {
            interface: program.interface.clone(),
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

        for binding in bindings.iter().rev() {
            current = match &binding.operation {
                closure::Operation::Call { callee, argument } => {
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
                            live: Vec::new(),
                            needs_environment: false,
                        },
                        binding.span,
                    )
                }
                closure::Operation::Case { scrutinee, arms } => {
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
                    self.states[current.0].bindings.insert(
                        0,
                        Binding {
                            pattern: binding.pattern.clone(),
                            operation: lower_operation(operation),
                            span: binding.span,
                        },
                    );
                    current
                }
            };
        }
        current
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
            bindings,
            terminator,
            span,
        });
        id
    }

    fn resolve_liveness(&mut self, start: usize, locals: &[LiveValue]) {
        let end = self.states.len();
        let mut live_in = vec![HashSet::new(); end - start];
        let mut environment_in = vec![false; end - start];
        loop {
            let mut changed = false;
            for index in (start..end).rev() {
                let state = &self.states[index];
                let (uses, definitions, uses_environment) = state_facts(state);
                let mut next = uses;
                let mut next_environment = uses_environment;
                for successor in successors(&state.terminator) {
                    debug_assert!((start..end).contains(&successor.0));
                    next.extend(
                        live_in[successor.0 - start]
                            .iter()
                            .filter(|id| !definitions.contains(id))
                            .copied(),
                    );
                    next_environment |= environment_in[successor.0 - start];
                }
                let slot = index - start;
                if live_in[slot] != next || environment_in[slot] != next_environment {
                    live_in[slot] = next;
                    environment_in[slot] = next_environment;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        for index in start..end {
            let Terminator::Call {
                resume,
                live,
                needs_environment,
                ..
            } = &mut self.states[index].terminator
            else {
                continue;
            };
            let resume_live = &live_in[resume.0 - start];
            *live = locals
                .iter()
                .filter(|value| resume_live.contains(&value.id))
                .cloned()
                .collect();
            *needs_environment = environment_in[resume.0 - start];
        }
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
            argument,
        } => Operation::Memory {
            primitive: *primitive,
            argument: argument.clone(),
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
        | closure::Operation::Case { .. }
        | closure::Operation::PrimitiveBranch { .. } => {
            unreachable!("control operations become terminators")
        }
    }
}

fn local_values(block: &closure::Block, parameter: Option<&closure::Parameter>) -> Vec<LiveValue> {
    let mut values = Vec::new();
    if let Some(parameter) = parameter
        && let Some(id) = parameter.binding
    {
        values.push(LiveValue {
            id,
            ty: parameter.ty.clone(),
            span: parameter.span,
        });
    }
    collect_local_values(block, &mut values);
    values
}

fn collect_local_values(block: &closure::Block, values: &mut Vec<LiveValue>) {
    for binding in &block.bindings {
        collect_pattern_values(&binding.pattern, binding.span, values);
        match &binding.operation {
            closure::Operation::Case { arms, .. } => {
                for arm in arms {
                    collect_pattern_values(&arm.pattern, arm.span, values);
                    collect_local_values(&arm.value, values);
                }
            }
            closure::Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                collect_local_values(otherwise, values);
                collect_local_values(then, values);
            }
            _ => {}
        }
    }
}

fn collect_pattern_values(
    pattern: &Pattern,
    span: crate::source::Span,
    values: &mut Vec<LiveValue>,
) {
    match pattern {
        Pattern::Binding { id, ty } => values.push(LiveValue {
            id: *id,
            ty: ty.clone(),
            span,
        }),
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_values(element, span, values);
            }
        }
    }
}

fn state_facts(state: &State) -> (HashSet<ValueId>, HashSet<ValueId>, bool) {
    let mut uses = HashSet::new();
    let mut definitions = HashSet::new();
    let mut environment = false;
    if let Some(input) = &state.input {
        collect_pattern_definitions(input, &mut definitions);
    }
    for binding in &state.bindings {
        visit_operation(&binding.operation, &mut |atom| {
            collect_atom_uses(atom, &definitions, &mut uses, &mut environment)
        });
        collect_pattern_definitions(&binding.pattern, &mut definitions);
    }
    visit_terminator(&state.terminator, &mut |atom| {
        collect_atom_uses(atom, &definitions, &mut uses, &mut environment)
    });
    (uses, definitions, environment)
}

fn collect_pattern_definitions(pattern: &Pattern, definitions: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            definitions.insert(*id);
        }
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_definitions(element, definitions);
            }
        }
    }
}

fn collect_atom_uses(
    atom: &Atom,
    definitions: &HashSet<ValueId>,
    uses: &mut HashSet<ValueId>,
    environment: &mut bool,
) {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) if !definitions.contains(&id) => {
            uses.insert(id);
        }
        AtomKind::Reference(Reference::EnvironmentField(_) | Reference::SelfClosure(_)) => {
            *environment = true;
        }
        _ => {}
    }
}

fn visit_operation(operation: &Operation, visit: &mut impl FnMut(&Atom)) {
    match operation {
        Operation::Atom(value)
        | Operation::SymbolLength { value }
        | Operation::SymbolAt { argument: value }
        | Operation::Memory {
            argument: value, ..
        }
        | Operation::ExternalCall {
            argument: value, ..
        }
        | Operation::NumericConversion { operand: value }
        | Operation::PrimitiveUnary { operand: value, .. }
        | Operation::SumInjection { value, .. } => visit(value),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            captures.iter().for_each(visit)
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}

fn visit_terminator(terminator: &Terminator, visit: &mut impl FnMut(&Atom)) {
    match terminator {
        Terminator::Return(value) | Terminator::Jump { value, .. } => visit(value),
        Terminator::Goto(_) => {}
        Terminator::Call {
            callee, argument, ..
        }
        | Terminator::TailCall { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Terminator::Case { scrutinee, .. } => visit(scrutinee),
        Terminator::PrimitiveBranch { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}

fn successors(terminator: &Terminator) -> Vec<StateId> {
    match terminator {
        Terminator::Return(_) | Terminator::TailCall { .. } => Vec::new(),
        Terminator::Goto(target) | Terminator::Jump { target, .. } => vec![*target],
        Terminator::Call { resume, .. } => vec![*resume],
        Terminator::Case { arms, .. } => arms.iter().map(|arm| arm.target).collect(),
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => vec![*otherwise, *then],
    }
}
