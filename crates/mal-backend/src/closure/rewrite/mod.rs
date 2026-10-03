//! Mechanical support for policy-owning closure IR rewrites.

mod copy;
mod identities;
pub(crate) mod walk;

pub(crate) use copy::{copy_functions, copy_top_level};
pub(crate) use identities::{Identities, are_unique};

const GROWTH_FACTOR: usize = 4;
const GROWTH_SLACK: usize = 64;

/// Maximum function-table size shared by closure IR rewrites that copy code.
pub(crate) fn copy_budget(functions: usize) -> usize {
    functions * GROWTH_FACTOR + GROWTH_SLACK
}
