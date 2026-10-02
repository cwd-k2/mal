//! Incremental construction of a function definition, one basic block at a time.

use super::*;

pub(in crate::backend::llvm) struct FunctionBuilder {
    signature: FunctionSignature,
    entry_prefix: Vec<Instruction>,
    blocks: Vec<BasicBlock>,
    labels: HashSet<String>,
}

impl FunctionBuilder {
    pub(in crate::backend::llvm) fn new(signature: FunctionSignature) -> Self {
        Self {
            signature,
            entry_prefix: Vec::new(),
            blocks: Vec::new(),
            labels: HashSet::new(),
        }
    }

    pub(in crate::backend::llvm) fn start_block(&mut self, label: impl Into<String>) -> bool {
        let label = label.into();
        if !is_valid_name(&label) || !self.labels.insert(label.clone()) {
            return false;
        }
        self.blocks.push(BasicBlock::empty(label));
        true
    }

    pub(in crate::backend::llvm) fn structured_instruction(
        &mut self,
        instruction: Instruction,
    ) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.push(instruction)
    }

    pub(in crate::backend::llvm) fn terminate(&mut self, terminator: Terminator) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.terminate(terminator)
    }

    pub(in crate::backend::llvm) fn structured_entry_instruction(
        &mut self,
        instruction: Instruction,
    ) {
        self.entry_prefix.push(instruction);
    }

    pub(in crate::backend::llvm) fn finish(mut self) -> Option<FunctionDefinition> {
        let entry = self.blocks.first_mut()?;
        if !self.entry_prefix.is_empty() {
            let mut instructions = self.entry_prefix;
            instructions.append(&mut entry.instructions);
            entry.instructions = instructions;
        }
        self.blocks
            .iter()
            .all(|block| block.terminator.is_some())
            .then_some(())?;
        FunctionDefinition::new(self.signature, self.blocks)
    }
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct BasicBlock {
    pub(super) label: String,
    pub(super) instructions: Vec<Instruction>,
    pub(super) terminator: Option<Terminator>,
}

impl BasicBlock {
    #[cfg(test)]
    pub(in crate::backend::llvm) fn new(
        label: impl Into<String>,
        instructions: impl IntoIterator<Item = Instruction>,
        terminator: Terminator,
    ) -> Option<Self> {
        let label = label.into();
        is_valid_name(&label).then_some(())?;
        let mut block = Self::empty(label);
        for instruction in instructions {
            block.push(instruction).then_some(())?;
        }
        block.terminate(terminator).then_some(block)
    }

    pub(super) fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    pub(super) fn push(&mut self, instruction: Instruction) -> bool {
        if self.terminator.is_some() {
            return false;
        }
        self.instructions.push(instruction);
        true
    }

    pub(super) fn terminate(&mut self, terminator: Terminator) -> bool {
        if self.terminator.is_some() {
            return false;
        }
        self.terminator = Some(terminator);
        true
    }

    pub(super) fn render_into(&self, output: &mut String) {
        output.push_str(&self.label);
        output.push_str(":\n");
        for instruction in &self.instructions {
            output.push_str("  ");
            instruction.render_into(output);
            output.push('\n');
        }
        if let Some(terminator) = &self.terminator {
            output.push_str("  ");
            terminator.render_into(output);
            output.push('\n');
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        self.instructions.iter().any(Instruction::uses_byte_runtime)
            || self
                .terminator
                .as_ref()
                .is_some_and(Terminator::uses_byte_runtime)
    }
}
