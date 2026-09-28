//! Entry macros for the LLVM syntax DSL documented in `docs/implementation/backend-syntax-construction.md`.
//!
//! Public entries normalize `#{}` interpolation before delegating to grammar-specific helpers;
//! normalized helpers are implementation details shared by nested productions.

mod constant;
mod declaration;
mod instruction;
mod module;
mod terminator;
mod value;

pub(in crate::backend) use constant::*;
pub(in crate::backend) use declaration::*;
pub(in crate::backend) use instruction::*;
pub(in crate::backend) use module::*;
pub(in crate::backend) use terminator::*;
pub(in crate::backend) use value::*;

#[cfg(test)]
mod tests;
