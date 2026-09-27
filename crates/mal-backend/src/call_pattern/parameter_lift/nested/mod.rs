//! Propagation through a callback captured by another closure.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomId, FunctionId};

mod analysis;
mod rewrite;

pub(super) use analysis::find;
pub(super) use rewrite::{extend_creator, prepare_function};

pub(super) struct Use {
    pub(super) function: FunctionId,
    pub(super) capture: usize,
    pub(super) aliases: HashSet<ValueId>,
    pub(super) capture_atoms: HashSet<AtomId>,
}
