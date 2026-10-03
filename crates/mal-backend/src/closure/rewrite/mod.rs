//! Mechanical support for policy-owning closure IR rewrites.

mod identities;
pub(crate) mod walk;

pub(crate) use identities::{Identities, are_unique};
