use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, FunctionId, Program};

use super::index;

/// The lexical scope containing a creator or application site.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Scope {
    TopLevel(usize),
    Function(FunctionId),
}

/// One function-valued producer result consumed by exactly one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Demand {
    pub(crate) producer_result: ValueId,
    pub(crate) producer: FunctionId,
    pub(crate) producer_site: AtomId,
    pub(crate) producer_argument: Atom,
    pub(crate) consumer: AtomId,
    pub(crate) argument: Atom,
}

/// One direct edge followed while propagating an application demand through producer results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProducerStep {
    pub(crate) function: FunctionId,
    pub(crate) result: ProducerResult,
}

/// The producer named directly by a function body result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProducerResult {
    Call {
        function: FunctionId,
        site: AtomId,
        argument: Atom,
    },
    Closure {
        creator: ValueId,
        function: FunctionId,
        captures: Vec<Atom>,
    },
}

/// One application reached while executing a demanded closure target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplicationStep {
    pub(crate) host: FunctionId,
    pub(crate) site: AtomId,
    pub(crate) argument: Atom,
    pub(crate) targets: Vec<FunctionId>,
}

/// One closure instance whose code participates in the candidate slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Creator {
    pub(crate) scope: Scope,
    pub(crate) binding: ValueId,
    pub(crate) function: FunctionId,
    pub(crate) captures: Vec<AtomId>,
}

/// A concrete closure origin reaching an application site.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ClosureSource {
    Creator {
        binding: ValueId,
        function: FunctionId,
    },
    SelfClosure(FunctionId),
    Unbound(FunctionId),
}

impl ClosureSource {
    pub(crate) fn function(self) -> FunctionId {
        match self {
            Self::Creator { function, .. }
            | Self::SelfClosure(function)
            | Self::Unbound(function) => function,
        }
    }
}

/// One program-wide call site that can invoke code participating in the candidate slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallSite {
    pub(crate) scope: Scope,
    pub(crate) site: AtomId,
    pub(crate) targets: Vec<FunctionId>,
    pub(crate) sources: Vec<ClosureSource>,
}

/// Closed application demands admitted from a closure program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    pub(in crate::continuation_specialization) demands: Vec<Demand>,
    pub(in crate::continuation_specialization) steps: Vec<ProducerStep>,
    pub(in crate::continuation_specialization) applications: Vec<ApplicationStep>,
    pub(in crate::continuation_specialization) creators: Vec<Creator>,
    pub(in crate::continuation_specialization) call_sites: Vec<CallSite>,
    pub(in crate::continuation_specialization) closed: bool,
    pub(in crate::continuation_specialization) creators_complete: bool,
}

impl Plan {
    pub(crate) fn new(program: &Program) -> Self {
        let (demands, steps) = index::analyze(program);
        let applications = super::application::trace(program, &steps);
        let (creators, call_sites) = super::inventory::collect(program, &steps, &applications);
        let closed = call_sites.iter().all(|call| {
            demands.iter().any(|demand| {
                call.site == demand.producer_site || call.site == demand.consumer
            }) || steps.iter().any(|step| {
                matches!(&step.result, ProducerResult::Call { site, .. } if *site == call.site)
            }) || applications
                .iter()
                .any(|application| application.site == call.site)
        });
        let creators_complete = call_sites
            .iter()
            .flat_map(|call| &call.sources)
            .all(|source| match source {
                ClosureSource::Creator { binding, function } => creators
                    .iter()
                    .any(|creator| creator.binding == *binding && creator.function == *function),
                ClosureSource::SelfClosure(_) => true,
                ClosureSource::Unbound(_) => false,
            })
            && creators.iter().all(|creator| {
                call_sites.iter().any(|call| {
                    call.sources.iter().any(|source| {
                        matches!(
                            source,
                            ClosureSource::Creator { binding, function }
                                if *binding == creator.binding && *function == creator.function
                        )
                    })
                })
            });
        Self {
            demands,
            steps,
            applications,
            creators,
            call_sites,
            closed,
            creators_complete,
        }
    }

    pub(crate) fn is_valid(&self, program: &Program) -> bool {
        *self == Self::new(program)
    }
}
