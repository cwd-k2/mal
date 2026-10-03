//! Producer-consumer slices whose function-valued result has one application demand.

mod index;
mod plan;

pub(crate) use plan::Plan;

#[cfg(test)]
mod tests;
