use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;
use mal_syntax::source::Span;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, CaseArm, Function, FunctionId, Join, Operation, Parameter,
    Pattern, Program, Reference,
};
use crate::closure::rewrite::Identities;

use super::plan::Demand;

/// Builds one capture-free worker by evaluating the admitted producer slice symbolically.
/// Closure values and local products remain symbolic until their only application; observable
/// operations are copied in source order. A recursive call to the demand root becomes a call to
/// the worker itself.
pub(super) fn worker(
    program: &Program,
    demand: &Demand,
    worker: FunctionId,
    ids: &mut Identities,
) -> Option<Function> {
    let root = program
        .functions
        .iter()
        .find(|function| function.id == demand.producer)?;
    if !root.captures.is_empty() {
        return None;
    }
    let Type::Function {
        parameter: demanded,
        result,
    } = &root.body.result.ty
    else {
        return None;
    };
    if **demanded != demand.argument.ty || **result == root.body.result.ty {
        return None;
    }

    let parameter_type =
        Type::Product(vec![root.parameter.ty.clone(), demand.argument.ty.clone()].into());
    let parameter = ids.value();
    let original_argument = ids.value();
    let demanded_argument = ids.value();
    let span = root.parameter.span;
    let mut bindings = vec![Binding {
        pattern: Pattern::Product {
            elements: vec![
                Pattern::Binding {
                    id: original_argument,
                    ty: root.parameter.ty.clone(),
                },
                Pattern::Binding {
                    id: demanded_argument,
                    ty: demand.argument.ty.clone(),
                },
            ],
            ty: parameter_type.clone(),
            span,
        },
        operation: Operation::Atom(Atom {
            id: ids.atom(),
            kind: AtomKind::Reference(Reference::Binding(parameter)),
            ty: parameter_type.clone(),
            span,
        }),
        span,
    }];

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
    let mut evaluator = Evaluator {
        ids,
        known,
        functions,
        root: root.id,
        worker,
        active: HashSet::from([root.id]),
        fallback_span: root.body.span,
    };
    let mut environment = Environment::default();
    if let Some(binding) = root.parameter.binding {
        environment.values.insert(
            binding,
            Value::Bound(original_argument, root.parameter.ty.clone()),
        );
    }
    let demand = Value::Bound(demanded_argument, demand.argument.ty.clone());
    let value = evaluator.block(
        &root.body,
        &root.joins,
        environment,
        Some(demand),
        &mut bindings,
    )?;
    let result = evaluator.materialize(value, &mut bindings)?;
    Some(Function {
        id: worker,
        captures: Vec::new(),
        parameter: Parameter {
            binding: Some(parameter),
            ty: parameter_type,
            span,
        },
        body: Block {
            bindings,
            result,
            span: root.body.span,
        },
        joins: Vec::new(),
    })
}

#[derive(Clone, Default)]
struct Environment {
    values: HashMap<ValueId, Value>,
    captures: Vec<Value>,
}

#[derive(Clone)]
enum Value {
    Bound(ValueId, Type),
    Literal(AtomKind, Type, Span),
    Closure {
        function: FunctionId,
        captures: Vec<Value>,
        ty: Type,
    },
    Product(Vec<Value>, Type, Span),
    Choice(Box<Choice>),
}

impl Value {
    fn ty(&self) -> &Type {
        match self {
            Self::Bound(_, ty)
            | Self::Literal(_, ty, _)
            | Self::Closure { ty, .. }
            | Self::Product(_, ty, _) => ty,
            Self::Choice(choice) => choice.ty(),
        }
    }
}

#[derive(Clone)]
enum Choice {
    Case {
        scrutinee: Atom,
        arms: Vec<ChoiceArm>,
        ty: Type,
        span: Span,
    },
    Branch {
        operator: crate::core::ast::BinaryPrimitive,
        left: Atom,
        right: Atom,
        otherwise: Box<EvalBlock>,
        then: Box<EvalBlock>,
        ty: Type,
        span: Span,
    },
}

impl Choice {
    fn ty(&self) -> &Type {
        match self {
            Self::Case { ty, .. } | Self::Branch { ty, .. } => ty,
        }
    }
}

#[derive(Clone)]
struct ChoiceArm {
    index: usize,
    pattern: Pattern,
    value: EvalBlock,
    span: Span,
}

#[derive(Clone)]
struct EvalBlock {
    bindings: Vec<Binding>,
    value: Value,
    span: Span,
}

struct Evaluator<'a> {
    ids: &'a mut Identities,
    known: HashMap<ValueId, FunctionId>,
    functions: HashMap<FunctionId, &'a Function>,
    root: FunctionId,
    worker: FunctionId,
    active: HashSet<FunctionId>,
    fallback_span: Span,
}

impl Evaluator<'_> {
    fn block(
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
                let callee = self.resolve(callee, environment)?;
                let argument = self.resolve(argument, environment)?;
                self.apply(callee, argument, output)
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
                function,
                captures,
                ty,
            } if function == self.root
                && self.active.contains(&function)
                && captures.len() == 1 =>
            {
                let [producer_argument] = captures.as_slice() else {
                    return None;
                };
                let Type::Function { result, .. } = ty else {
                    return None;
                };
                let combined_ty = Type::Product(
                    vec![producer_argument.ty().clone(), argument.ty().clone()].into(),
                );
                let combined = Value::Product(
                    vec![producer_argument.clone(), argument],
                    combined_ty.clone(),
                    self.fallback_span,
                );
                let argument = self.materialize(combined, output)?;
                let id = self.ids.value();
                output.push(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: (*result).clone(),
                    },
                    operation: Operation::Call {
                        callee: Atom {
                            id: self.ids.atom(),
                            kind: AtomKind::Reference(Reference::SelfClosure(self.worker)),
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
        if function == self.root && self.active.contains(&function) {
            let result_ty = self.functions.get(&function)?.body.result.ty.clone();
            let Type::Function { parameter, result } = result_ty else {
                return None;
            };
            // A recursive producer is only valid while applying its result. `call` itself
            // does not know that demand, so the closure is retained symbolically until apply.
            return Some(Value::Closure {
                function,
                captures: vec![argument],
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

    fn materialize(&mut self, value: Value, output: &mut Vec<Binding>) -> Option<Atom> {
        match value {
            Value::Bound(id, ty) => Some(Atom {
                id: self.ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(id)),
                ty,
                span: self.fallback_span,
            }),
            Value::Literal(kind, ty, span) => Some(Atom {
                id: self.ids.atom(),
                kind,
                ty,
                span,
            }),
            Value::Product(elements, ty, span) => {
                let elements = elements
                    .into_iter()
                    .map(|value| self.materialize(value, output))
                    .collect::<Option<Vec<_>>>()?;
                let id = self.ids.value();
                output.push(Binding {
                    pattern: Pattern::Binding { id, ty: ty.clone() },
                    operation: Operation::Product(elements),
                    span,
                });
                Some(Atom {
                    id: self.ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(id)),
                    ty,
                    span,
                })
            }
            Value::Choice(choice) => self.materialize_choice(*choice, output),
            Value::Closure { .. } => None,
        }
    }

    fn materialize_choice(&mut self, choice: Choice, output: &mut Vec<Binding>) -> Option<Atom> {
        let (operation, ty, span) = match choice {
            Choice::Case {
                scrutinee,
                arms,
                ty,
                span,
            } => {
                let arms = arms
                    .into_iter()
                    .map(|arm| {
                        Some(CaseArm {
                            index: arm.index,
                            pattern: arm.pattern,
                            value: self.finish(arm.value)?,
                            span: arm.span,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                (Operation::Case { scrutinee, arms }, ty, span)
            }
            Choice::Branch {
                operator,
                left,
                right,
                otherwise,
                then,
                ty,
                span,
            } => (
                Operation::PrimitiveBranch {
                    operator,
                    left,
                    right,
                    otherwise: Box::new(self.finish(*otherwise)?),
                    then: Box::new(self.finish(*then)?),
                },
                ty,
                span,
            ),
        };
        let id = self.ids.value();
        output.push(Binding {
            pattern: Pattern::Binding { id, ty: ty.clone() },
            operation,
            span,
        });
        Some(Atom {
            id: self.ids.atom(),
            kind: AtomKind::Reference(Reference::Binding(id)),
            ty,
            span,
        })
    }

    fn finish(&mut self, mut block: EvalBlock) -> Option<Block> {
        let result = self.materialize(block.value, &mut block.bindings)?;
        Some(Block {
            bindings: block.bindings,
            result,
            span: block.span,
        })
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
        let mut atom =
            |this: &mut Self, value: &Atom| this.resolve_concrete(value, environment, output);
        Some(match operation {
            Operation::Symbol {
                primitive,
                operands,
            } => Operation::Symbol {
                primitive: *primitive,
                operands: operands
                    .iter()
                    .map(|value| atom(self, value))
                    .collect::<Option<_>>()?,
            },
            Operation::Memory {
                primitive,
                operands,
            } => Operation::Memory {
                primitive: *primitive,
                operands: operands
                    .iter()
                    .map(|value| atom(self, value))
                    .collect::<Option<_>>()?,
            },
            Operation::Buffer {
                operation,
                element,
                operands,
            } => Operation::Buffer {
                operation: *operation,
                element: element.clone(),
                operands: operands
                    .iter()
                    .map(|value| atom(self, value))
                    .collect::<Option<_>>()?,
            },
            Operation::ExternalCall { id, argument } => Operation::ExternalCall {
                id: *id,
                argument: atom(self, argument)?,
            },
            Operation::NumericConversion { operand } => Operation::NumericConversion {
                operand: atom(self, operand)?,
            },
            Operation::SumInjection { index, value } => Operation::SumInjection {
                index: *index,
                value: atom(self, value)?,
            },
            Operation::PrimitiveUnary { operator, operand } => Operation::PrimitiveUnary {
                operator: *operator,
                operand: atom(self, operand)?,
            },
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => Operation::PrimitiveBinary {
                operator: *operator,
                left: atom(self, left)?,
                right: atom(self, right)?,
            },
            Operation::Atom(_)
            | Operation::Goto { .. }
            | Operation::MakeClosure { .. }
            | Operation::Call { .. }
            | Operation::Product(_)
            | Operation::Case { .. }
            | Operation::PrimitiveBranch { .. } => return None,
        })
    }
}
