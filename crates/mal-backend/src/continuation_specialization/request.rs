use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomId, FunctionId, Program};
use crate::closure::rewrite::{Identities, copy_budget};

use super::plan::{ClosureSource, Demand, Plan, ProducerResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Worker {
    pub(crate) original: FunctionId,
    pub(crate) worker: FunctionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Request {
    pub(crate) demand: Demand,
    pub(crate) workers: Vec<Worker>,
    pub(crate) creators: Vec<ValueId>,
    pub(crate) sites: Vec<AtomId>,
}

impl Request {
    pub(crate) fn is_valid(&self, program: &Program, plan: &Plan) -> bool {
        prepare(program, plan).as_ref() == Some(self)
    }

    pub(crate) fn copy_workers(&self, program: &Program) -> Program {
        super::rewrite::copy_workers(program, self)
    }
}

pub(super) fn prepare(program: &Program, plan: &Plan) -> Option<Request> {
    let [demand] = plan.demands.as_slice() else {
        return None;
    };
    if !plan.closed || !plan.creators_complete || !plan.transport_closed {
        return None;
    }

    let mut functions = HashSet::new();
    for step in &plan.steps {
        functions.insert(step.function);
        let (target, creator) = match &step.result {
            ProducerResult::Call { function, .. } => (*function, None),
            ProducerResult::Closure {
                creator, function, ..
            } => (*function, Some(*creator)),
        };
        functions.insert(target);
        if creator.is_some_and(|creator| {
            !plan
                .creators
                .iter()
                .any(|candidate| candidate.binding == creator && candidate.function == target)
        }) {
            return None;
        }
    }
    for application in &plan.applications {
        functions.insert(application.host);
        functions.extend(application.targets.iter().copied());
    }
    if functions.is_empty()
        || plan.creators.is_empty()
        || plan.call_sites.is_empty()
        || plan.call_sites.iter().any(|call| call.sources.is_empty())
        || plan
            .call_sites
            .iter()
            .flat_map(|call| &call.sources)
            .any(|source| match source {
                ClosureSource::Creator { function, .. } | ClosureSource::SelfClosure(function) => {
                    !functions.contains(function)
                }
                ClosureSource::Unbound(_) => true,
            })
        || program.functions.len() + functions.len() > copy_budget(program.functions.len())
    {
        return None;
    }

    let mut functions = functions.into_iter().collect::<Vec<_>>();
    functions.sort_by_key(|function| function_number(*function));
    let mut copy = program.clone();
    let mut ids = Identities::after(&mut copy);
    let workers = functions
        .into_iter()
        .map(|original| Worker {
            original,
            worker: ids.function(),
        })
        .collect();
    let mut creators = plan
        .creators
        .iter()
        .map(|creator| creator.binding)
        .collect::<Vec<_>>();
    creators.sort_by_key(|creator| value_number(*creator));
    creators.dedup();
    let mut sites = plan
        .call_sites
        .iter()
        .map(|call| call.site)
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| site.0);
    sites.dedup();
    Some(Request {
        demand: demand.clone(),
        workers,
        creators,
        sites,
    })
}

fn value_number(value: ValueId) -> u64 {
    match value {
        ValueId::Core(crate::core::ast::ValueId::Source(id)) => u64::from(id.0),
        ValueId::Core(crate::core::ast::ValueId::Temporary(id)) => 1_u64 << 32 | u64::from(id),
        ValueId::Temporary(id) => 2_u64 << 32 | u64::from(id),
    }
}

fn function_number(function: FunctionId) -> u32 {
    let FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) = function;
    number
}
