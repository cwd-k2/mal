use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;
use mal_syntax::source::Span;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, Function, FunctionId, Join, Operation, Pattern, Program,
    Reference,
};
use crate::closure::rewrite::Identities;

use super::symbolic::{Choice, ChoiceArm, Environment, EvalBlock, Value, same_context, transport};

pub(super) struct Evaluator<'a> {
    ids: &'a mut Identities,
    known: HashMap<ValueId, FunctionId>,
    functions: HashMap<FunctionId, &'a Function>,
    current: FunctionId,
    worker: FunctionId,
    workers: &'a HashMap<FunctionId, (FunctionId, ValueId)>,
    targets: &'a HashMap<crate::closure::ast::AtomId, FunctionId>,
    contexts: &'a mut HashMap<FunctionId, Vec<Value>>,
    active: HashSet<FunctionId>,
    fallback_span: Span,
}

impl Evaluator<'_> {
    pub(super) fn new<'a>(
        program: &'a Program,
        current: FunctionId,
        worker: FunctionId,
        workers: &'a HashMap<FunctionId, (FunctionId, ValueId)>,
        targets: &'a HashMap<crate::closure::ast::AtomId, FunctionId>,
        contexts: &'a mut HashMap<FunctionId, Vec<Value>>,
        ids: &'a mut Identities,
    ) -> Evaluator<'a> {
        let known = program
            .bindings
            .iter()
            .filter_map(|binding| binding.known_function())
            .collect::<HashMap<_, _>>();
        let functions = program
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect::<HashMap<_, _>>();
        let fallback_span = functions
            .get(&current)
            .expect("current function exists")
            .body
            .span;
        Evaluator {
            ids,
            known,
            functions,
            current,
            worker,
            workers,
            targets,
            contexts,
            active: HashSet::from([current]),
            fallback_span,
        }
    }

    pub(super) fn block(
        &mut self,
        block: &Block,
        joins: &[Join],
        mut environment: Environment,
        demand: Option<Value>,
        output: &mut Vec<Binding>,
    ) -> Option<Value> {
        for binding in &block.bindings {
            let value = self.operation(
                &binding.operation,
                binding.pattern.ty(),
                binding.span,
                joins,
                &environment,
                output,
            )?;
            self.bind(&binding.pattern, value, &mut environment, output)?;
        }
        let value = self.resolve(&block.result, &environment)?;
        match demand {
            Some(argument) => self.apply(value, argument, output),
            None => Some(value),
        }
    }

    fn operation(
        &mut self,
        operation: &Operation,
        ty: &Type,
        span: Span,
        joins: &[Join],
        environment: &Environment,
        output: &mut Vec<Binding>,
    ) -> Option<Value> {
        match operation {
            Operation::Atom(atom) => self.resolve(atom, environment),
            Operation::MakeClosure { function, captures } => Some(Value::Closure {
                function: *function,
                captures: captures
                    .iter()
                    .map(|atom| self.resolve(atom, environment))
                    .collect::<Option<_>>()?,
                ty: ty.clone(),
            }),
            Operation::Product(elements) => Some(Value::Product(
                elements
                    .iter()
                    .map(|atom| self.resolve(atom, environment))
                    .collect::<Option<_>>()?,
                ty.clone(),
                span,
            )),
            Operation::Call { callee, argument } => {
                let mut resolved = self.resolve(callee, environment)?;
                if matches!(resolved, Value::Bound(_, _))
                    && let Some(function) = self.targets.get(&callee.id).copied()
                    && self.functions.get(&function)?.captures.is_empty()
                {
                    resolved = Value::Closure {
                        function,
                        captures: Vec::new(),
                        ty: callee.ty.clone(),
                    };
                }
                let argument = self.resolve(argument, environment)?;
                self.apply(resolved, argument, output)
            }
            Operation::Goto { target, value } => {
                let join = joins.get(target.0)?;
                let value = self.resolve(value, environment)?;
                let mut joined = environment.clone();
                self.bind(&join.parameter, value, &mut joined, output)?;
                self.block(&join.body, joins, joined, None, output)
            }
            Operation::Case { scrutinee, arms } => {
                let scrutinee = self.resolve_concrete(scrutinee, environment, output)?;
                let mut rewritten = Vec::with_capacity(arms.len());
                for arm in arms {
                    let mut arm_environment = environment.clone();
                    let pattern = self.fresh_pattern(&arm.pattern, &mut arm_environment);
                    let mut bindings = Vec::new();
                    let value =
                        self.block(&arm.value, joins, arm_environment, None, &mut bindings)?;
                    rewritten.push(ChoiceArm {
                        index: arm.index,
                        pattern,
                        value: EvalBlock {
                            bindings,
                            value,
                            span: arm.value.span,
                        },
                        span: arm.span,
                    });
                }
                Some(Value::Choice(Box::new(Choice::Case {
                    scrutinee,
                    arms: rewritten,
                    ty: ty.clone(),
                    span,
                })))
            }
            Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
                let left = self.resolve_concrete(left, environment, output)?;
                let right = self.resolve_concrete(right, environment, output)?;
                let otherwise = self.eval_nested(otherwise, joins, environment)?;
                let then = self.eval_nested(then, joins, environment)?;
                Some(Value::Choice(Box::new(Choice::Branch {
                    operator: *operator,
                    left,
                    right,
                    otherwise: Box::new(otherwise),
                    then: Box::new(then),
                    ty: ty.clone(),
                    span,
                })))
            }
            _ => {
                let operation = self.concrete_operation(operation, environment, output)?;
                let id = self.ids.value();
                output.push(Binding {
                    pattern: Pattern::Binding { id, ty: ty.clone() },
                    operation,
                    span,
                });
                Some(Value::Bound(id, ty.clone()))
            }
        }
    }

    fn eval_nested(
        &mut self,
        block: &Block,
        joins: &[Join],
        environment: &Environment,
    ) -> Option<EvalBlock> {
        let mut bindings = Vec::new();
        let value = self.block(block, joins, environment.clone(), None, &mut bindings)?;
        Some(EvalBlock {
            bindings,
            value,
            span: block.span,
        })
    }

    fn apply(
        &mut self,
        callee: Value,
        argument: Value,
        output: &mut Vec<Binding>,
    ) -> Option<Value> {
        match callee {
            Value::Closure {
                function, captures, ..
            } if self.active.contains(&function)
                && captures.len() == self.functions.get(&function)?.captures.len()
                && !matches!(
                    self.functions.get(&function)?.body.result.ty,
                    Type::Function { .. }
                ) =>
            {
                let result = self.functions.get(&function)?.body.result.ty.clone();
                self.record_context(function, &captures)?;
                let mut combined_values = Vec::new();
                for capture in &captures {
                    transport(capture, &mut combined_values)?;
                }
                combined_values.push(argument);
                let combined_ty = Type::Product(
                    combined_values
                        .iter()
                        .map(|value| value.ty().clone())
                        .collect::<Vec<_>>()
                        .into(),
                );
                let combined =
                    Value::Product(combined_values, combined_ty.clone(), self.fallback_span);
                let argument = self.materialize(combined, output)?;
                let id = self.ids.value();
                let (_, target_binding) = self.workers.get(&function).copied()?;
                let callee = if function == self.current {
                    Reference::SelfClosure(self.worker)
                } else {
                    Reference::Binding(target_binding)
                };
                output.push(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: result.clone(),
                    },
                    operation: Operation::Call {
                        callee: Atom {
                            id: self.ids.atom(),
                            kind: AtomKind::Reference(callee),
                            ty: Type::Function {
                                parameter: combined_ty.into(),
                                result: result.clone().into(),
                            },
                            span: self.fallback_span,
                        },
                        argument,
                    },
                    span: self.fallback_span,
                });
                Some(Value::Bound(id, result))
            }
            Value::Closure {
                function,
                captures,
                ty,
            } if self.active.contains(&function)
                && captures.len() == self.functions.get(&function)?.captures.len() + 1 =>
            {
                let (producer_argument, environment) = captures.split_last()?;
                self.record_context(function, environment)?;
                let Type::Function { result, .. } = ty else {
                    return None;
                };
                let mut combined_values = Vec::new();
                for capture in environment {
                    transport(capture, &mut combined_values)?;
                }
                combined_values.extend([producer_argument.clone(), argument]);
                let combined_ty = Type::Product(
                    combined_values
                        .iter()
                        .map(|value| value.ty().clone())
                        .collect::<Vec<_>>()
                        .into(),
                );
                let combined =
                    Value::Product(combined_values, combined_ty.clone(), self.fallback_span);
                let argument = self.materialize(combined, output)?;
                let id = self.ids.value();
                let (_, target_binding) = self.workers.get(&function).copied()?;
                let callee = if function == self.current {
                    Reference::SelfClosure(self.worker)
                } else {
                    Reference::Binding(target_binding)
                };
                output.push(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: (*result).clone(),
                    },
                    operation: Operation::Call {
                        callee: Atom {
                            id: self.ids.atom(),
                            kind: AtomKind::Reference(callee),
                            ty: Type::Function {
                                parameter: combined_ty.into(),
                                result: result.clone(),
                            },
                            span: self.fallback_span,
                        },
                        argument,
                    },
                    span: self.fallback_span,
                });
                Some(Value::Bound(id, (*result).clone()))
            }
            Value::Closure {
                function, captures, ..
            } => self.call(function, captures, argument, output),
            Value::Choice(choice) => self.apply_choice(*choice, argument, output),
            Value::Bound(_, _) | Value::Literal(_, _, _) | Value::Product(_, _, _) => None,
        }
    }

    fn record_context(&mut self, function: FunctionId, captures: &[Value]) -> Option<()> {
        match self.contexts.get(&function) {
            Some(existing) if !same_context(existing, captures) => None,
            Some(_) => Some(()),
            None => {
                self.contexts.insert(function, captures.to_vec());
                Some(())
            }
        }
    }

    fn apply_choice(
        &mut self,
        mut choice: Choice,
        argument: Value,
        _output: &mut Vec<Binding>,
    ) -> Option<Value> {
        match &mut choice {
            Choice::Case { arms, ty, .. } => {
                for arm in &mut *arms {
                    arm.value.value = self.apply(
                        arm.value.value.clone(),
                        argument.clone(),
                        &mut arm.value.bindings,
                    )?;
                }
                *ty = arms.first()?.value.value.ty().clone();
            }
            Choice::Branch {
                otherwise,
                then,
                ty,
                ..
            } => {
                otherwise.value = self.apply(
                    otherwise.value.clone(),
                    argument.clone(),
                    &mut otherwise.bindings,
                )?;
                then.value = self.apply(then.value.clone(), argument, &mut then.bindings)?;
                *ty = then.value.ty().clone();
            }
        }
        Some(Value::Choice(Box::new(choice)))
    }

    fn call(
        &mut self,
        function: FunctionId,
        captures: Vec<Value>,
        argument: Value,
        output: &mut Vec<Binding>,
    ) -> Option<Value> {
        if self.active.contains(&function) {
            let result_ty = self.functions.get(&function)?.body.result.ty.clone();
            let Type::Function { parameter, result } = result_ty else {
                return None;
            };
            // A recursive producer is only valid while applying its result. `call` itself
            // does not know that demand, so the closure is retained symbolically until apply.
            return Some(Value::Closure {
                function,
                captures: captures
                    .into_iter()
                    .chain(std::iter::once(argument))
                    .collect(),
                ty: Type::Function { parameter, result },
            });
        }
        if !self.active.insert(function) {
            return None;
        }
        let definition = *self.functions.get(&function)?;
        if definition.captures.len() != captures.len() {
            self.active.remove(&function);
            return None;
        }
        let mut environment = Environment {
            values: HashMap::new(),
            captures,
        };
        if let Some(parameter) = definition.parameter.binding {
            environment.values.insert(parameter, argument);
        }
        let value = self.block(
            &definition.body,
            &definition.joins,
            environment,
            None,
            output,
        );
        self.active.remove(&function);
        value
    }

    fn bind(
        &mut self,
        pattern: &Pattern,
        value: Value,
        environment: &mut Environment,
        output: &mut Vec<Binding>,
    ) -> Option<()> {
        match (pattern, value) {
            (Pattern::Binding { id, .. }, value) => {
                environment.values.insert(*id, value);
                Some(())
            }
            (Pattern::Wildcard { .. }, value) => {
                if matches!(value, Value::Closure { .. }) {
                    None
                } else {
                    Some(())
                }
            }
            (Pattern::Product { elements, .. }, Value::Product(values, _, _))
                if elements.len() == values.len() =>
            {
                for (element, value) in elements.iter().zip(values) {
                    self.bind(element, value, environment, output)?;
                }
                Some(())
            }
            (Pattern::Product { .. }, value) => {
                let atom = self.materialize(value, output)?;
                let span = atom.span;
                let pattern = self.fresh_pattern(pattern, environment);
                output.push(Binding {
                    pattern,
                    operation: Operation::Atom(atom),
                    span,
                });
                Some(())
            }
        }
    }

    fn resolve(&mut self, atom: &Atom, environment: &Environment) -> Option<Value> {
        match atom.kind {
            AtomKind::Reference(Reference::Binding(binding)) => {
                environment.values.get(&binding).cloned().or_else(|| {
                    self.known
                        .get(&binding)
                        .copied()
                        .map(|function| Value::Closure {
                            function,
                            captures: Vec::new(),
                            ty: atom.ty.clone(),
                        })
                })
            }
            AtomKind::Reference(Reference::Capture(index)) => {
                environment.captures.get(index).cloned()
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => Some(Value::Closure {
                function,
                captures: environment.captures.clone(),
                ty: atom.ty.clone(),
            }),
            _ => Some(Value::Literal(
                atom.kind.clone(),
                atom.ty.clone(),
                atom.span,
            )),
        }
    }

    fn resolve_concrete(
        &mut self,
        atom: &Atom,
        environment: &Environment,
        output: &mut Vec<Binding>,
    ) -> Option<Atom> {
        let value = self.resolve(atom, environment)?;
        let mut concrete = self.materialize(value, output)?;
        concrete.span = atom.span;
        Some(concrete)
    }

    pub(super) fn materialize(&mut self, value: Value, output: &mut Vec<Binding>) -> Option<Atom> {
        super::materialize::value(value, output, self.ids, self.fallback_span)
    }

    fn fresh_pattern(&mut self, pattern: &Pattern, environment: &mut Environment) -> Pattern {
        match pattern {
            Pattern::Binding { id, ty } => {
                let fresh = self.ids.value();
                environment
                    .values
                    .insert(*id, Value::Bound(fresh, ty.clone()));
                Pattern::Binding {
                    id: fresh,
                    ty: ty.clone(),
                }
            }
            Pattern::Wildcard { ty, span } => Pattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            Pattern::Product { elements, ty, span } => Pattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.fresh_pattern(element, environment))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    fn concrete_operation(
        &mut self,
        operation: &Operation,
        environment: &Environment,
        output: &mut Vec<Binding>,
    ) -> Option<Operation> {
        super::concrete::operation(operation, |value| {
            self.resolve_concrete(value, environment, output)
        })
    }
}
