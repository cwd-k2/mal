use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomId, AtomKind, Binding, Block, FunctionId, Operation, Pattern, Program, Reference,
    TopLevelPattern,
};

use super::super::{ids::Identities, lambda_lift};
use super::Candidate;

mod pattern;

use pattern::{
    expand_callback_pattern, rewrite_pattern, rewrite_top_level_patterns, set_pattern_type,
};

/// One admitted candidate with the capture bindings its host destructures.
struct Plan {
    candidate: Candidate,
    capture_bindings: Vec<(ValueId, Type)>,
}

/// Independent candidates rewritten in one walk, and the identities that select each one's rewrite.
struct Batch {
    plans: Vec<Plan>,
    shapes: HashMap<FunctionId, lambda_lift::LiftedShape>,
    by_target: HashMap<FunctionId, usize>,
    by_host: HashMap<FunctionId, usize>,
    by_replacement: HashMap<AtomId, usize>,
    by_forwarded: HashMap<AtomId, usize>,
    by_direct_call: HashMap<AtomId, usize>,
}

impl Batch {
    fn new(plans: Vec<Plan>, shapes: HashMap<FunctionId, lambda_lift::LiftedShape>) -> Self {
        let mut batch = Self {
            plans,
            shapes,
            by_target: HashMap::new(),
            by_host: HashMap::new(),
            by_replacement: HashMap::new(),
            by_forwarded: HashMap::new(),
            by_direct_call: HashMap::new(),
        };
        for (index, plan) in batch.plans.iter().enumerate() {
            let candidate = &plan.candidate;
            batch.by_target.insert(candidate.target, index);
            batch.by_host.insert(candidate.host, index);
            for atom in candidate.replacements.keys() {
                batch.by_replacement.insert(*atom, index);
            }
            for atom in &candidate.forwarded {
                batch.by_forwarded.insert(*atom, index);
            }
            for atom in &candidate.direct_calls {
                batch.by_direct_call.insert(*atom, index);
            }
        }
        batch
    }

    fn shape(&self, plan: usize) -> &lambda_lift::LiftedShape {
        &self.shapes[&self.plans[plan].candidate.target]
    }
}

/// Rewrites candidates whose footprints are disjoint, so each one's rewrite is the one it would get alone.
pub(super) fn apply(program: &mut Program, candidates: Vec<Candidate>) {
    let mut ids = Identities::after(program);
    let targets = candidates
        .iter()
        .map(|candidate| candidate.target)
        .collect::<HashSet<_>>();
    let shapes = lambda_lift::lift_functions(program, &targets, &mut ids);
    let mut types = HashMap::new();
    let mut plans = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let shape = &shapes[&candidate.target];
        let capture_bindings = candidate
            .capture_types
            .iter()
            .map(|ty| (ids.value(), ty.clone()))
            .collect::<Vec<_>>();
        if let Some(nested) = &candidate.nested {
            super::nested::prepare_function(
                program,
                nested,
                &candidate.capture_types,
                shape,
                &mut ids,
            );
        }
        for function in &mut program.functions {
            if function.id == candidate.host {
                function.parameter.ty = candidate.host_parameter_type.clone();
                expand_callback_pattern(function, &candidate, &capture_bindings, shape);
            }
        }
        types.insert(candidate.parameter, candidate.host_parameter_type.clone());
        types.insert(candidate.callback, shape.closure_type.clone());
        types.insert(candidate.host_binding, candidate.host_closure_type.clone());
        rewrite_top_level_patterns(
            &mut program.bindings,
            candidate.host_binding,
            &candidate.host_closure_type,
        );
        plans.push(Plan {
            candidate,
            capture_bindings,
        });
    }
    let batch = Batch::new(plans, shapes);
    for binding in &mut program.bindings {
        rewrite_block(&mut binding.value, &batch, &mut types, &mut ids);
    }
    for function in &mut program.functions {
        rewrite_block(&mut function.body, &batch, &mut types, &mut ids);
        for join in &mut function.joins {
            rewrite_block(&mut join.body, &batch, &mut types, &mut ids);
        }
    }
}

fn rewrite_block(
    block: &mut Block,
    batch: &Batch,
    types: &mut HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    let mut output = Vec::with_capacity(block.bindings.len());
    for mut binding in std::mem::take(&mut block.bindings) {
        rewrite_nested(&mut binding.operation, batch, types, ids);
        rewrite_atoms(&mut binding.operation, |atom| {
            if let Some(ty) = atom.binding().and_then(|binding| types.get(&binding)) {
                atom.ty = ty.clone();
            } else if let AtomKind::Reference(Reference::SelfClosure(function)) = atom.kind
                && let Some(&plan) = batch.by_host.get(&function)
            {
                atom.ty = batch.plans[plan].candidate.host_closure_type.clone();
            }
        });
        if let Operation::MakeClosure { function, captures } = &mut binding.operation
            && let Some(&plan) = batch.by_target.get(function)
        {
            captures.clear();
            set_pattern_type(&mut binding.pattern, &batch.shape(plan).closure_type, types);
        }
        if let Operation::MakeClosure { function, .. } = &binding.operation
            && let Some(&plan) = batch.by_host.get(function)
        {
            let host_closure_type = &batch.plans[plan].candidate.host_closure_type;
            set_pattern_type(&mut binding.pattern, host_closure_type, types);
        }
        for plan in &batch.plans {
            if let Some(nested) = &plan.candidate.nested {
                super::nested::extend_creator(
                    &mut binding.operation,
                    nested,
                    &plan.capture_bindings,
                    ids,
                );
            }
        }

        let mut capture_products = Vec::new();
        rewrite_atoms(&mut binding.operation, |atom| {
            let (plan, elements) = if let Some(&plan) = batch.by_replacement.get(&atom.id) {
                let captures = &batch.plans[plan].candidate.replacements[&atom.id];
                (plan, ids.copy_atoms(captures))
            } else if let Some(&plan) = batch.by_forwarded.get(&atom.id) {
                let elements = batch.plans[plan]
                    .capture_bindings
                    .iter()
                    .map(|(binding, ty)| Atom {
                        id: ids.atom(),
                        kind: AtomKind::Reference(Reference::Binding(*binding)),
                        ty: ty.clone(),
                        span: atom.span,
                    })
                    .collect();
                (plan, elements)
            } else {
                return;
            };
            let capture_type = &batch.plans[plan].candidate.capture_type;
            let id = ids.value();
            capture_products.push(Binding {
                pattern: Pattern::Binding {
                    id,
                    ty: capture_type.clone(),
                },
                operation: Operation::Product(elements),
                span: atom.span,
            });
            *atom = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(id)),
                ty: capture_type.clone(),
                span: atom.span,
            };
        });
        output.extend(capture_products);

        if let Operation::Call { callee, argument } = &mut binding.operation
            && let Some(&plan) = batch.by_direct_call.get(&callee.id)
        {
            let shape = batch.shape(plan);
            let mut elements = batch.plans[plan]
                .capture_bindings
                .iter()
                .map(|(id, ty)| Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(*id)),
                    ty: ty.clone(),
                    span: argument.span,
                })
                .collect::<Vec<_>>();
            elements.push(argument.clone());
            callee.ty = shape.closure_type.clone();
            let packed = ids.value();
            output.push(Binding {
                pattern: Pattern::Binding {
                    id: packed,
                    ty: shape.parameter_type.clone(),
                },
                operation: Operation::Product(elements),
                span: argument.span,
            });
            *argument = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(packed)),
                ty: shape.parameter_type.clone(),
                span: argument.span,
            };
        }

        rewrite_pattern(&mut binding.pattern, types);
        if let Operation::Product(elements) = &binding.operation {
            let ty = Type::Product(
                elements
                    .iter()
                    .map(|element| element.ty.clone())
                    .collect::<Vec<_>>()
                    .into(),
            );
            set_pattern_type(&mut binding.pattern, &ty, types);
        }
        output.push(binding);
    }
    block.bindings = output;
    if let Some(ty) = block
        .result
        .binding()
        .and_then(|binding| types.get(&binding))
    {
        block.result.ty = ty.clone();
    }
}

fn rewrite_nested(
    operation: &mut Operation,
    batch: &Batch,
    types: &mut HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    match operation {
        Operation::Case { arms, .. } => {
            for arm in arms {
                rewrite_block(&mut arm.value, batch, types, ids);
            }
        }
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            rewrite_block(otherwise, batch, types, ids);
            rewrite_block(then, batch, types, ids);
        }
        _ => {}
    }
}

fn rewrite_atoms(operation: &mut Operation, mut visit: impl FnMut(&mut Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Symbol {
            operands: captures, ..
        }
        | Operation::Buffer {
            operands: captures, ..
        } => captures.iter_mut().for_each(visit),
        Operation::Call { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Operation::Case { scrutinee, .. } => visit(scrutinee),
        Operation::PrimitiveBranch { left, right, .. }
        | Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}
