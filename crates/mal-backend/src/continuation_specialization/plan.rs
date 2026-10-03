use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, FunctionId, Program};

use super::index;

/// The lexical scope containing a creator or application site.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scope {
    TopLevel(usize),
    Function(FunctionId),
}

/// One function-valued producer result consumed by exactly one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Demand {
    pub(crate) producer_result: ValueId,
    pub(crate) producer: FunctionId,
    pub(crate) producer_argument: Atom,
    pub(crate) consumer: AtomId,
    pub(crate) argument: Atom,
}

/// One direct edge followed while propagating an application demand through producer results.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProducerStep {
    pub(crate) function: FunctionId,
    pub(crate) result: ProducerResult,
}

/// The producer named directly by a function body result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProducerResult {
    Call(FunctionId),
    Closure(FunctionId),
}

/// One application reached while executing a demanded closure target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplicationStep {
    pub(crate) host: FunctionId,
    pub(crate) site: AtomId,
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

/// One program-wide call site that can invoke code participating in the candidate slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallSite {
    pub(crate) scope: Scope,
    pub(crate) site: AtomId,
    pub(crate) targets: Vec<FunctionId>,
}

/// Closed application demands admitted from a closure program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    pub(in crate::continuation_specialization) demands: Vec<Demand>,
    pub(in crate::continuation_specialization) steps: Vec<ProducerStep>,
    pub(in crate::continuation_specialization) applications: Vec<ApplicationStep>,
    pub(in crate::continuation_specialization) creators: Vec<Creator>,
    pub(in crate::continuation_specialization) call_sites: Vec<CallSite>,
}

impl Plan {
    pub(crate) fn new(program: &Program) -> Self {
        let (demands, steps) = index::analyze(program);
        let applications = super::application::trace(program, &steps);
        let (creators, call_sites) = super::inventory::collect(program, &steps, &applications);
        Self {
            demands,
            steps,
            applications,
            creators,
            call_sites,
        }
    }

    pub(crate) fn is_valid(&self, program: &Program) -> bool {
        *self == Self::new(program)
    }
}
