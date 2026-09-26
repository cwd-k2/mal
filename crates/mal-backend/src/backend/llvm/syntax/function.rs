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
        self.blocks.push(BasicBlock::empty(label));
    }

    pub(in crate::backend::llvm) fn instruction(&mut self, instruction: impl Into<String>) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.push(instruction.into())
    }

    pub(in crate::backend::llvm) fn entry_instruction(&mut self, instruction: impl Into<String>) {
        self.entry_prefix.push(instruction.into());
    }

    pub(in crate::backend::llvm) fn finish(mut self) -> Option<FunctionDefinition> {
        let entry = self.blocks.first_mut()?;
        if !self.entry_prefix.is_empty() {
            let mut instructions = self
                .entry_prefix
                .into_iter()
                .map(Instruction)
                .collect::<Vec<_>>();
            instructions.append(&mut entry.instructions);
            entry.instructions = instructions;
        }
        self.blocks
            .iter()
            .all(|block| block.terminator.is_some())
            .then_some(())?;
        Some(FunctionDefinition::new(self.signature, self.blocks))
    }
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct BasicBlock {
    label: String,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

impl BasicBlock {
    pub(in crate::backend::llvm) fn new(
        label: impl Into<String>,
        instructions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<Self> {
        let mut block = Self::empty(label);
        for instruction in instructions {
            block.push(instruction.into()).then_some(())?;
        }
        block.terminator.is_some().then_some(block)
    }

    fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    fn push(&mut self, instruction: String) -> bool {
        if self.terminator.is_some() {
            return false;
        }
        if Terminator::recognizes(&instruction) {
            self.terminator = Terminator::from_text(instruction);
        } else {
            self.instructions.push(Instruction(instruction));
        }
        true
    }

    fn render_into(&self, output: &mut String) {
        output.push_str(&self.label);
        output.push_str(":\n");
        for instruction in &self.instructions {
            output.push_str("  ");
            output.push_str(&instruction.0);
            output.push('\n');
        }
        if let Some(terminator) = &self.terminator {
            output.push_str("  ");
            output.push_str(terminator.text());
            output.push('\n');
        }
    }
}

#[derive(Clone)]
struct Instruction(String);

#[derive(Clone)]
enum Terminator {
    Branch(String),
    Return(String),
    Switch(String),
    Unreachable,
}

impl Terminator {
    fn recognizes(text: &str) -> bool {
        text == "unreachable"
            || text.starts_with("br ")
            || text.starts_with("ret ")
            || text.starts_with("switch ")
    }

    fn from_text(text: String) -> Option<Self> {
        if text == "unreachable" {
            Some(Self::Unreachable)
        } else if text.starts_with("br ") {
            Some(Self::Branch(text))
        } else if text.starts_with("ret ") {
            Some(Self::Return(text))
        } else if text.starts_with("switch ") {
            Some(Self::Switch(text))
        } else {
            None
        }
    }

    fn text(&self) -> &str {
        match self {
            Self::Branch(text) | Self::Return(text) | Self::Switch(text) => text,
            Self::Unreachable => "unreachable",
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

    #[test]
    fn rejects_missing_terminators_and_instructions_after_a_terminator() {
        assert!(BasicBlock::new("entry", ["call void @work()"]).is_none());
        assert!(BasicBlock::new("entry", ["ret void", "call void @late()"]).is_none());

        let mut function = FunctionBuilder::new("void @missing_terminator()");
        function.start_block("entry");
        assert!(function.instruction("call void @work()"));
        assert!(function.finish().is_none());
    }
}
