//! Entry macros for the C syntax DSL documented in `docs/implementation/backend-syntax-construction.md`.
//!
//! Public entries normalize `#{}` interpolation before delegating to grammar-specific helpers;
//! normalized helpers are implementation details shared by nested productions.

mod aggregate;
mod declaration;
mod expression;
mod function;
mod preprocessor;
mod signature;
mod statement;

pub(in crate::backend) use aggregate::*;
pub(in crate::backend) use declaration::*;
pub(in crate::backend) use expression::*;
pub(in crate::backend) use function::*;
pub(in crate::backend) use preprocessor::*;
pub(in crate::backend) use signature::*;
pub(in crate::backend) use statement::*;

#[cfg(test)]
mod tests;
