use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, Program};

use super::index;

/// One function-valued producer result consumed by exactly one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Demand {
    pub(crate) producer_result: ValueId,
    pub(crate) consumer: AtomId,
    pub(crate) argument: Atom,
}

/// Closed application demands admitted from a closure program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    pub(in crate::continuation_specialization) demands: Vec<Demand>,
}

impl Plan {
    pub(crate) fn new(program: &Program) -> Self {
        Self {
            demands: index::demands(program),
        }
    }

    pub(crate) fn is_valid(&self, program: &Program) -> bool {
        *self == Self::new(program)
    }
}
