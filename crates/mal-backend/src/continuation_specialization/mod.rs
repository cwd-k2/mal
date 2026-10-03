//! Producer-consumer slices whose function-valued result has one application demand.

mod application;
mod concrete;
mod fuse;
mod index;
mod inventory;
mod materialize;
mod plan;
mod provenance;
mod request;
mod rewrite;
mod symbolic;
mod trace;
mod worker;

pub(crate) use plan::Plan;

#[cfg(test)]
mod tests;
