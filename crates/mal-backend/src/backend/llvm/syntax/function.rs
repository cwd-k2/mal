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
        result: impl std::fmt::Display,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            linkage: None,
            result: result.to_string(),
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

    fn is_valid(&self) -> bool {
        self.linkage
            .as_ref()
            .is_none_or(|linkage| is_valid_name(linkage))
            && is_single_line(&self.result)
            && is_valid_name(&self.name)
            && self
                .parameters
                .iter()
                .all(|parameter| is_single_line(parameter))
            && self
                .attributes
                .iter()
                .all(|attribute| is_single_line(attribute))
    }
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
        block.push_instruction(instruction.into())
    }

    pub(in crate::backend::llvm) fn terminate(&mut self, terminator: Terminator) -> bool {
        let Some(block) = self.blocks.last_mut() else {
            return false;
        };
        block.terminate(terminator)
    }

    pub(in crate::backend::llvm) fn entry_instruction(
        &mut self,
        instruction: impl Into<String>,
    ) -> bool {
        let instruction = instruction.into();
        let Some(instruction) = Instruction::new(instruction) else {
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
        terminator: Terminator,
    ) -> Option<Self> {
        let label = label.into();
        is_valid_name(&label).then_some(())?;
        let mut block = Self::empty(label);
        for instruction in instructions {
            block.push_instruction(instruction.into()).then_some(())?;
        }
        block.terminate(terminator).then_some(block)
    }

    fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    fn push_instruction(&mut self, instruction: String) -> bool {
        if self.terminator.is_some() {
            return false;
        }
        let Some(instruction) = Instruction::new(instruction) else {
            return false;
        };
        self.instructions.push(instruction);
        true
    }

    fn terminate(&mut self, terminator: Terminator) -> bool {
        if self.terminator.is_some() {
            return false;
        }
        self.terminator = Some(terminator);
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
            terminator.render_into(output);
            output.push('\n');
        }
    }

    fn uses_byte_runtime(&self) -> bool {
        self.instructions
            .iter()
            .any(|instruction| is_byte_runtime_reference(&instruction.0))
            || self
                .terminator
                .as_ref()
                .is_some_and(Terminator::uses_byte_runtime)
    }
}

fn is_byte_runtime_reference(text: &str) -> bool {
    [
        "@mal_runtime_bytes_",
        "@mal_runtime_buffer_",
        "@mal_runtime_symbol_",
    ]
    .iter()
    .any(|prefix| text.contains(prefix))
}

pub(super) fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'.' | b'$'))
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$' | b'-'))
}

pub(super) fn is_single_line(text: &str) -> bool {
    !text.is_empty() && !text.contains(['\n', '\r'])
}

#[derive(Clone)]
struct Instruction(String);

impl Instruction {
    fn new(text: String) -> Option<Self> {
        (!text.is_empty() && !text.contains(['\n', '\r'])).then_some(Self(text))
    }
}

#[derive(Clone)]
pub(in crate::backend::llvm) enum Terminator {
    Branch {
        target: String,
    },
    ConditionalBranch {
        condition: String,
        then_target: String,
        else_target: String,
    },
    ReturnVoid,
    Return {
        ty: String,
        value: String,
    },
    Switch {
        ty: String,
        value: String,
        default: String,
        cases: Vec<(String, String)>,
    },
    Unreachable,
}

impl Terminator {
    pub(in crate::backend::llvm) fn branch(target: impl Into<String>) -> Option<Self> {
        let target = target.into();
        is_valid_name(&target).then_some(Self::Branch { target })
    }

    pub(in crate::backend::llvm) fn conditional_branch(
        condition: impl Into<String>,
        then_target: impl Into<String>,
        else_target: impl Into<String>,
    ) -> Option<Self> {
        let condition = condition.into();
        let then_target = then_target.into();
        let else_target = else_target.into();
        (is_single_line(&condition) && is_valid_name(&then_target) && is_valid_name(&else_target))
            .then_some(Self::ConditionalBranch {
                condition,
                then_target,
                else_target,
            })
    }

    pub(in crate::backend::llvm) fn return_void() -> Self {
        Self::ReturnVoid
    }

    pub(in crate::backend::llvm) fn return_value(
        ty: impl std::fmt::Display,
        value: impl Into<String>,
    ) -> Option<Self> {
        let ty = ty.to_string();
        let value = value.into();
        (is_single_line(&ty) && is_single_line(&value)).then_some(Self::Return { ty, value })
    }

    pub(in crate::backend::llvm) fn switch(
        ty: impl Into<String>,
        value: impl Into<String>,
        default: impl Into<String>,
        cases: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Option<Self> {
        let ty = ty.into();
        let value = value.into();
        let default = default.into();
        let cases = cases
            .into_iter()
            .map(|(value, target)| (value.into(), target.into()))
            .collect::<Vec<_>>();
        (is_single_line(&ty)
            && is_single_line(&value)
            && is_valid_name(&default)
            && cases
                .iter()
                .all(|(value, target)| is_single_line(value) && is_valid_name(target)))
        .then_some(Self::Switch {
            ty,
            value,
            default,
            cases,
        })
    }

    pub(in crate::backend::llvm) fn unreachable() -> Self {
        Self::Unreachable
    }

    fn render_into(&self, output: &mut String) {
        match self {
            Self::Branch { target } => output.push_str(&format!("br label %{target}")),
            Self::ConditionalBranch {
                condition,
                then_target,
                else_target,
            } => output.push_str(&format!(
                "br i1 {condition}, label %{then_target}, label %{else_target}"
            )),
            Self::ReturnVoid => output.push_str("ret void"),
            Self::Return { ty, value } => output.push_str(&format!("ret {ty} {value}")),
            Self::Switch {
                ty,
                value,
                default,
                cases,
            } => {
                output.push_str(&format!("switch {ty} {value}, label %{default} ["));
                for (case, target) in cases {
                    output.push_str(&format!("\n    {ty} {case}, label %{target}"));
                }
                output.push_str("\n  ]");
            }
            Self::Unreachable => output.push_str("unreachable"),
        }
    }

    fn uses_byte_runtime(&self) -> bool {
        false
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
        assert!(function.terminate(Terminator::branch("body").unwrap()));
        assert!(function.start_block("body"));
        assert!(function.terminate(Terminator::return_void()));
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
        assert!(
            BasicBlock::new(
                "0invalid",
                std::iter::empty::<&str>(),
                Terminator::return_void(),
            )
            .is_none()
        );

        let mut function = FunctionBuilder::new(FunctionSignature::new(
            "void",
            "duplicate",
            std::iter::empty::<&str>(),
        ));
        assert!(function.start_block("entry"));
        assert!(function.terminate(Terminator::return_void()));
        assert!(!function.start_block("entry"));

        let entry = BasicBlock::new(
            "entry",
            std::iter::empty::<&str>(),
            Terminator::return_void(),
        )
        .unwrap();
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
        assert!(function.terminate(Terminator::return_void()));

        assert!(!function.entry_instruction("call void @work()\nret void"));
    }

    #[test]
    fn derives_byte_runtime_requirements_from_emitted_instructions() {
        let signature = || FunctionSignature::new("void", "example", std::iter::empty::<&str>());
        let plain = FunctionDefinition::new(
            signature(),
            vec![
                BasicBlock::new("entry", ["call void @work()"], Terminator::return_void()).unwrap(),
            ],
        )
        .unwrap();
        let bytes = FunctionDefinition::new(
            signature(),
            vec![
                BasicBlock::new(
                    "entry",
                    ["call void @mal_runtime_bytes_release(ptr null)"],
                    Terminator::return_void(),
                )
                .unwrap(),
            ],
        )
        .unwrap();

        assert!(!plain.uses_byte_runtime());
        assert!(bytes.uses_byte_runtime());
    }

    #[test]
    fn rejects_invalid_function_signature_fragments() {
        let block = || {
            BasicBlock::new(
                "entry",
                std::iter::empty::<&str>(),
                Terminator::return_void(),
            )
            .unwrap()
        };
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new("void", "0invalid", std::iter::empty::<&str>()),
                vec![block()],
            )
            .is_none()
        );
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new("void\nret void", "invalid", std::iter::empty::<&str>()),
                vec![block()],
            )
            .is_none()
        );
    }
}
