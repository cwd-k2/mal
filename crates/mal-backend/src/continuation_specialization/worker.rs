//! Capture-free worker construction from admitted continuation demands.

use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, Function, FunctionId, Operation, Parameter, Pattern, Program,
    Reference,
};
use crate::closure::rewrite::Identities;

use super::fuse::Evaluator;
use super::symbolic::{Environment, Value, rebuild_context, transport};

/// Builds the root worker and every recursive context it discovers while symbolically
/// evaluating the admitted producer slice.
pub(super) fn workers(
    program: &Program,
    root: FunctionId,
    demanded_type: &Type,
    workers: &HashMap<FunctionId, (FunctionId, ValueId)>,
    targets: &HashMap<crate::closure::ast::AtomId, FunctionId>,
    ids: &mut Identities,
) -> Option<Vec<Function>> {
    let mut contexts = HashMap::from([(root, Vec::new())]);
    let mut generated = HashSet::new();
    let mut functions = Vec::new();
    loop {
        let next = contexts
            .keys()
            .copied()
            .find(|function| !generated.contains(function));
        let Some(original) = next else {
            break;
        };
        let context = contexts.get(&original)?.clone();
        let definition = program
            .functions
            .iter()
            .find(|function| function.id == original)?;
        let demand = match &definition.body.result.ty {
            Type::Function { parameter, .. } if **parameter == *demanded_type => {
                Some(demanded_type)
            }
            Type::Function { .. } => return None,
            _ => None,
        };
        let (worker, _) = workers.get(&original).copied()?;
        functions.push(worker_one(
            WorkerInput {
                program,
                original,
                capture_context: &context,
                demanded_type: demand,
                worker,
            },
            workers,
            targets,
            &mut contexts,
            ids,
        )?);
        generated.insert(original);
    }
    Some(functions)
}

struct WorkerInput<'a> {
    program: &'a Program,
    original: FunctionId,
    capture_context: &'a [Value],
    demanded_type: Option<&'a Type>,
    worker: FunctionId,
}

fn worker_one(
    input: WorkerInput<'_>,
    workers: &HashMap<FunctionId, (FunctionId, ValueId)>,
    targets: &HashMap<crate::closure::ast::AtomId, FunctionId>,
    contexts: &mut HashMap<FunctionId, Vec<Value>>,
    ids: &mut Identities,
) -> Option<Function> {
    let WorkerInput {
        program,
        original,
        capture_context,
        demanded_type,
        worker,
    } = input;
    let root = program
        .functions
        .iter()
        .find(|function| function.id == original)?;
    if root.captures.len() != capture_context.len()
        || root
            .captures
            .iter()
            .zip(capture_context)
            .any(|(field, value)| field.ty != *value.ty())
    {
        return None;
    }
    if let Some(demanded_type) = demanded_type {
        let Type::Function { parameter, .. } = &root.body.result.ty else {
            return None;
        };
        if **parameter != *demanded_type {
            return None;
        }
    }

    let mut transported = Vec::new();
    for value in capture_context {
        transport(value, &mut transported)?;
    }
    let mut parameter_fields = transported
        .iter()
        .map(|value| value.ty().clone())
        .collect::<Vec<_>>();
    parameter_fields.push(root.parameter.ty.clone());
    parameter_fields.extend(demanded_type.cloned());
    let parameter_type = Type::Product(parameter_fields.into());
    let parameter = ids.value();
    let capture_arguments = transported.iter().map(|_| ids.value()).collect::<Vec<_>>();
    let original_argument = ids.value();
    let demanded_argument = demanded_type.map(|_| ids.value());
    let span = root.parameter.span;
    let mut bindings = vec![Binding {
        pattern: Pattern::Product {
            elements: capture_arguments
                .iter()
                .zip(&transported)
                .map(|(id, value)| Pattern::Binding {
                    id: *id,
                    ty: value.ty().clone(),
                })
                .chain([Pattern::Binding {
                    id: original_argument,
                    ty: root.parameter.ty.clone(),
                }])
                .chain(demanded_argument.map(|id| Pattern::Binding {
                    id,
                    ty: demanded_type.expect("demand identity has a type").clone(),
                }))
                .collect(),
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

    let mut evaluator = Evaluator::new(program, root.id, worker, workers, targets, contexts, ids);
    let mut environment = Environment {
        values: HashMap::new(),
        captures: rebuild_context(
            capture_context,
            &capture_arguments
                .iter()
                .zip(&transported)
                .map(|(id, value)| Value::Bound(*id, value.ty().clone()))
                .collect::<Vec<_>>(),
        )?,
    };
    if let Some(binding) = root.parameter.binding {
        environment.values.insert(
            binding,
            Value::Bound(original_argument, root.parameter.ty.clone()),
        );
    }
    let demand = demanded_argument.map(|id| {
        Value::Bound(
            id,
            demanded_type.expect("demand identity has a type").clone(),
        )
    });
    let value = evaluator.block(&root.body, &root.joins, environment, demand, &mut bindings)?;
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
