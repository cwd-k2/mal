use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, Operation, Pattern, Program, Reference, TopLevelBinding,
    TopLevelPattern,
};
use crate::closure::rewrite::{Identities, are_unique};

use super::request::Request;

/// Materializes the root demand worker and redirects its producer-consumer pair atomically.
pub(super) fn apply(program: &Program, request: &Request) -> Option<Program> {
    let mut rewritten = program.clone();
    let mut ids = Identities::after(&mut rewritten);
    for worker in &request.workers {
        if ids.function() != worker.worker {
            return None;
        }
    }
    let demanded_type = request.demand.argument.ty.clone();
    let targets = request.targets.iter().copied().collect::<HashMap<_, _>>();
    let workers = request
        .workers
        .iter()
        .map(|worker| (worker.original, (worker.worker, ids.value())))
        .collect::<HashMap<_, _>>();
    let (worker_id, worker_binding) = workers.get(&request.demand.producer).copied()?;
    let generated = super::worker::workers(
        program,
        request.demand.producer,
        &demanded_type,
        &workers,
        &targets,
        &mut ids,
    )?;
    let root = generated.iter().find(|worker| worker.id == worker_id)?;
    let worker_type = Type::Function {
        parameter: root.parameter.ty.clone().into(),
        result: root.body.result.ty.clone().into(),
    };
    let top_levels = generated
        .iter()
        .map(|worker| {
            let binding = workers[&request
                .workers
                .iter()
                .find(|candidate| candidate.worker == worker.id)
                .expect("generated worker belongs to the request")
                .original]
                .1;
            worker_top_level(worker, binding, &mut ids)
        })
        .collect::<Vec<_>>();

    let mut redirected = 0;
    for binding in &mut rewritten.bindings {
        redirect_block(
            &mut binding.value,
            request,
            worker_binding,
            &worker_type,
            &mut ids,
            &mut redirected,
        );
    }
    for function in &mut rewritten.functions {
        redirect_block(
            &mut function.body,
            request,
            worker_binding,
            &worker_type,
            &mut ids,
            &mut redirected,
        );
        for join in &mut function.joins {
            redirect_block(
                &mut join.body,
                request,
                worker_binding,
                &worker_type,
                &mut ids,
                &mut redirected,
            );
        }
    }
    if redirected != 1 {
        return None;
    }
    rewritten.bindings.extend(top_levels);
    rewritten.functions.extend(generated);
    debug_assert!(are_unique(&mut rewritten.clone()));
    Some(rewritten)
}

fn worker_top_level(
    worker: &crate::closure::ast::Function,
    binding: ValueId,
    ids: &mut Identities,
) -> TopLevelBinding {
    let ty = Type::Function {
        parameter: worker.parameter.ty.clone().into(),
        result: worker.body.result.ty.clone().into(),
    };
    let creator = ids.value();
    let span = worker.body.span;
    TopLevelBinding {
        pattern: TopLevelPattern::Binding {
            id: binding,
            name: format!("continuation${}", function_number(worker.id)),
            ty: ty.clone(),
        },
        value: Block {
            bindings: vec![Binding {
                pattern: Pattern::Binding {
                    id: creator,
                    ty: ty.clone(),
                },
                operation: Operation::MakeClosure {
                    function: worker.id,
                    captures: Vec::new(),
                },
                span,
            }],
            result: Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(creator)),
                ty,
                span,
            },
            span,
        },
        span,
    }
}

fn redirect_block(
    block: &mut Block,
    request: &Request,
    worker: ValueId,
    worker_type: &Type,
    ids: &mut Identities,
    redirected: &mut usize,
) {
    for binding in &mut block.bindings {
        match &mut binding.operation {
            Operation::Case { arms, .. } => {
                for arm in arms {
                    redirect_block(
                        &mut arm.value,
                        request,
                        worker,
                        worker_type,
                        ids,
                        redirected,
                    );
                }
            }
            Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                redirect_block(otherwise, request, worker, worker_type, ids, redirected);
                redirect_block(then, request, worker, worker_type, ids, redirected);
            }
            _ => {}
        }
    }

    let mut aliases = HashSet::from([request.demand.producer_result]);
    loop {
        let before = aliases.len();
        for binding in &block.bindings {
            let (
                Pattern::Binding { id, .. },
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(source)),
                    ..
                }),
            ) = (&binding.pattern, &binding.operation)
            else {
                continue;
            };
            if aliases.contains(source) {
                aliases.insert(*id);
            }
        }
        if aliases.len() == before {
            break;
        }
    }
    if !block.bindings.iter().any(|binding| {
        matches!(binding.pattern, Pattern::Binding { id, .. } if id == request.demand.producer_result)
    }) || !block.bindings.iter().any(|binding| {
        matches!(&binding.operation, Operation::Call { callee, .. } if callee.id == request.demand.consumer)
    }) {
        return;
    }

    let producer_argument = request.demand.producer_argument.clone();
    let mut bindings = Vec::with_capacity(block.bindings.len());
    for mut binding in std::mem::take(&mut block.bindings) {
        let bound = match binding.pattern {
            Pattern::Binding { id, .. } => Some(id),
            _ => None,
        };
        if bound.is_some_and(|id| aliases.contains(&id)) {
            continue;
        }
        if let Operation::Call { callee, argument } = &mut binding.operation
            && callee.id == request.demand.consumer
        {
            let combined_type = match worker_type {
                Type::Function { parameter, .. } => (**parameter).clone(),
                _ => return,
            };
            let combined = ids.value();
            bindings.push(Binding {
                pattern: Pattern::Binding {
                    id: combined,
                    ty: combined_type.clone(),
                },
                operation: Operation::Product(vec![
                    Atom {
                        id: ids.atom(),
                        ..producer_argument.clone()
                    },
                    Atom {
                        id: ids.atom(),
                        ..argument.clone()
                    },
                ]),
                span: binding.span,
            });
            *callee = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(worker)),
                ty: worker_type.clone(),
                span: callee.span,
            };
            *argument = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(combined)),
                ty: combined_type,
                span: argument.span,
            };
            *redirected += 1;
        }
        bindings.push(binding);
    }
    block.bindings = bindings;
}

fn function_number(function: crate::closure::ast::FunctionId) -> u32 {
    let crate::closure::ast::FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) =
        function;
    number
}
