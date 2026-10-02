//! Function definitions: a signature and basic blocks, built through `FunctionBuilder`.

use std::collections::HashSet;

use super::lexical::is_valid_name;
use super::{Instruction, Type};

mod builder;
mod signature;
mod terminator;

pub(in crate::backend::llvm) use builder::{BasicBlock, FunctionBuilder};
pub(in crate::backend) use signature::{
    FunctionAttribute, FunctionSignature, Linkage, Parameter, ParameterAttribute,
};
pub(in crate::backend::llvm) use terminator::Terminator;

#[derive(Clone)]
pub(in crate::backend::llvm) struct FunctionDefinition {
    signature: FunctionSignature,
    blocks: Vec<BasicBlock>,
}

impl FunctionDefinition {
    pub(in crate::backend::llvm) fn new(
        signature: FunctionSignature,
        blocks: Vec<BasicBlock>,
    ) -> Option<Self> {
        let mut labels = HashSet::new();
        (signature.is_valid()
            && !blocks.is_empty()
            && blocks
                .iter()
                .all(|block| labels.insert(block.label.clone())))
        .then_some(Self { signature, blocks })
    }

    pub(in crate::backend::llvm) fn render(&self) -> String {
        let mut output = format!("define {} {{\n", self.signature.render());
        for block in &self.blocks {
            block.render_into(&mut output);
        }
        output.push('}');
        output
    }

    pub(super) fn name(&self) -> &str {
        self.signature.name()
    }

    pub(in crate::backend::llvm) fn uses_byte_runtime(&self) -> bool {
        self.blocks.iter().any(BasicBlock::uses_byte_runtime)
    }
}

#[cfg(test)]
mod tests;
