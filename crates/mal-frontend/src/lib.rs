#![forbid(unsafe_code)]

//! Semantic analysis from parsed source to an admitted, specialized program.
//!
//! Name resolution, type checking, specialization, and editor indexes live here. The crate does not lower programs or
//! select a target representation.

pub mod analysis;
pub mod check;
pub mod editor;
pub mod resolve;
