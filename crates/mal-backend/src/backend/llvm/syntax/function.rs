use std::collections::HashSet;

#[derive(Clone)]
pub(in crate::backend::llvm) struct FunctionDefinition {
    signature: FunctionSignature,
    blocks: Vec<BasicBlock>,
}

#[derive(Clone)]
pub(in crate::backend) struct FunctionSignature {
    linkage: Option<String>,
    result: String,
    name: String,
    parameters: Vec<String>,
    attributes: Vec<String>,
}

impl FunctionSignature {
    pub(in crate::backend) fn new(
        result: impl Into<String>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            linkage: None,
            result: result.into(),
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend::llvm) fn with_linkage(mut self, linkage: impl Into<String>) -> Self {
        self.linkage = Some(linkage.into());
        self
    }

    pub(in crate::backend::llvm) fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.attributes = attributes.into_iter().map(Into::into).collect();
        self
    }

    fn render(&self) -> String {
        let linkage = self
            .linkage
            .as_ref()
            .map_or(String::new(), |linkage| format!("{linkage} "));
        let attributes = if self.attributes.is_empty() {
            String::new()
        } else {
            format!(" {}", self.attributes.join(" "))
        };
        format!(
            "{linkage}{} @{}({}){attributes}",
            self.result,
            self.name,
            self.parameters.join(", ")
        )
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

impl FunctionDefinition {
    pub(in crate::backend::llvm) fn new(
        signature: FunctionSignature,
        blocks: Vec<BasicBlock>,
    ) -> Option<Self> {
        let mut labels = HashSet::new();
        (!blocks.is_empty()
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
}

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

    pub(in crate::backend::llvm) fn instruction(&mut self, instruction: impl Into<String>) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.push(instruction.into())
    }

    pub(in crate::backend::llvm) fn entry_instruction(
        &mut self,
        instruction: impl Into<String>,
    ) -> bool {
        let Some(instruction) = Instruction::new(instruction.into()) else {
            return false;
        };
        self.entry_prefix.push(instruction);
        true
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
    label: String,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

impl BasicBlock {
    pub(in crate::backend::llvm) fn new(
        label: impl Into<String>,
        instructions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<Self> {
        let label = label.into();
        is_valid_name(&label).then_some(())?;
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
            let Some(terminator) = Terminator::from_text(instruction) else {
                return false;
            };
            self.terminator = Some(terminator);
        } else {
            let Some(instruction) = Instruction::new(instruction) else {
                return false;
            };
            self.instructions.push(instruction);
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

fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'.' | b'$'))
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$' | b'-'))
}

#[derive(Clone)]
struct Instruction(String);

impl Instruction {
    fn new(text: String) -> Option<Self> {
        (!text.is_empty() && !text.contains(['\n', '\r']) && !Terminator::recognizes(&text))
            .then_some(Self(text))
    }
}

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
        let mut function = FunctionBuilder::new(
            FunctionSignature::new("void", "example", std::iter::empty::<&str>())
                .with_linkage("internal"),
        );
        assert!(function.start_block("entry"));
        assert!(function.instruction("br label %body"));
        assert!(function.start_block("body"));
        assert!(function.instruction("ret void"));
        assert!(function.entry_instruction("%storage = alloca i32, align 4"));

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

        let mut function = FunctionBuilder::new(FunctionSignature::new(
            "void",
            "missing_terminator",
            std::iter::empty::<&str>(),
        ));
        assert!(function.start_block("entry"));
        assert!(function.instruction("call void @work()"));
        assert!(function.finish().is_none());
    }

    #[test]
    fn rejects_invalid_and_duplicate_block_labels() {
        assert!(BasicBlock::new("0invalid", ["ret void"]).is_none());

        let mut function = FunctionBuilder::new(FunctionSignature::new(
            "void",
            "duplicate",
            std::iter::empty::<&str>(),
        ));
        assert!(function.start_block("entry"));
        assert!(function.instruction("ret void"));
        assert!(!function.start_block("entry"));

        let entry = BasicBlock::new("entry", ["ret void"]).unwrap();
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new("void", "duplicate_direct", std::iter::empty::<&str>(),),
                vec![entry.clone(), entry],
            )
            .is_none()
        );
    }

    #[test]
    fn rejects_invalid_entry_prefix_instructions() {
        let mut function = FunctionBuilder::new(FunctionSignature::new(
            "void",
            "invalid_prefix",
            std::iter::empty::<&str>(),
        ));
        assert!(function.start_block("entry"));
        assert!(function.instruction("ret void"));

        assert!(!function.entry_instruction("ret void"));
        assert!(!function.entry_instruction("call void @work()\nret void"));
    }
}
