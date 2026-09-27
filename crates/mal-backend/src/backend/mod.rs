pub(crate) mod abi;
pub(crate) mod artifact;
pub(crate) mod c;
pub(crate) mod llvm;
pub(crate) mod runtime;
pub(crate) mod source_layout;
mod syntax_interpolation;

pub(in crate::backend) use syntax_interpolation::normalize_syntax_interpolation;
