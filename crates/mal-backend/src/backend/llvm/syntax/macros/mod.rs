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
