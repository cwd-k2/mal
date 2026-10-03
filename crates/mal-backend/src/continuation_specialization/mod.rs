//! Producer-consumer slices whose function-valued result has one application demand.

mod application;
mod index;
mod inventory;
mod plan;
mod provenance;
mod request;
mod rewrite;
mod trace;

pub(crate) use plan::Plan;

#[cfg(test)]
mod tests;
