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
