#[derive(Clone)]
pub(in crate::backend::llvm) struct FunctionDefinition {
    signature: String,
    blocks: Vec<BasicBlock>,
}

impl FunctionDefinition {
    pub(in crate::backend::llvm) fn new(
        signature: impl Into<String>,
        blocks: Vec<BasicBlock>,
    ) -> Self {
        Self {
            signature: signature.into(),
            blocks,
        }
    }

    pub(in crate::backend::llvm) fn render(&self) -> String {
        let mut output = format!("define {} {{\n", self.signature);
        for block in &self.blocks {
            block.render_into(&mut output);
        }
        output.push('}');
        output
    }
}

pub(in crate::backend::llvm) struct FunctionBuilder {
    signature: String,
    entry_prefix: Vec<String>,
    blocks: Vec<BasicBlock>,
}

impl FunctionBuilder {
    pub(in crate::backend::llvm) fn new(signature: impl Into<String>) -> Self {
        Self {
            signature: signature.into(),
            entry_prefix: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub(in crate::backend::llvm) fn start_block(&mut self, label: impl Into<String>) {
        self.blocks
            .push(BasicBlock::new(label, std::iter::empty::<String>()));
    }

    pub(in crate::backend::llvm) fn instruction(&mut self, instruction: impl Into<String>) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.instructions.push(instruction.into());
        true
    }

    pub(in crate::backend::llvm) fn entry_instruction(&mut self, instruction: impl Into<String>) {
        self.entry_prefix.push(instruction.into());
    }

    pub(in crate::backend::llvm) fn finish(mut self) -> Option<FunctionDefinition> {
        let entry = self.blocks.first_mut()?;
        if !self.entry_prefix.is_empty() {
            self.entry_prefix.append(&mut entry.instructions);
            entry.instructions = self.entry_prefix;
        }
        Some(FunctionDefinition::new(self.signature, self.blocks))
    }
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct BasicBlock {
    label: String,
    instructions: Vec<String>,
}

impl BasicBlock {
    pub(in crate::backend::llvm) fn new(
        label: impl Into<String>,
        instructions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            label: label.into(),
            instructions: instructions.into_iter().map(Into::into).collect(),
        }
    }

    fn render_into(&self, output: &mut String) {
        output.push_str(&self.label);
        output.push_str(":\n");
        for instruction in &self.instructions {
            output.push_str("  ");
            output.push_str(instruction);
            output.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_late_entry_instructions_before_the_existing_entry_body() {
        let mut function = FunctionBuilder::new("internal void @example()");
        function.start_block("entry");
        assert!(function.instruction("br label %body"));
        function.start_block("body");
        assert!(function.instruction("ret void"));
        function.entry_instruction("%storage = alloca i32, align 4");

        assert_eq!(
            function.finish().unwrap().render(),
            concat!(
                "define internal void @example() {\n",
                "entry:\n",
                "  %storage = alloca i32, align 4\n",
                "  br label %body\n",
                "body:\n",
                "  ret void\n",
                "}",
            )
        );
    }
}
