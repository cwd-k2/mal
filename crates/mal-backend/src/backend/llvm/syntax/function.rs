use std::collections::HashSet;

use super::{Instruction, Type};

#[derive(Clone)]
pub(in crate::backend::llvm) struct FunctionDefinition {
    signature: FunctionSignature,
    blocks: Vec<BasicBlock>,
}

#[derive(Clone)]
pub(in crate::backend) struct FunctionSignature {
    linkage: Option<Linkage>,
    result: Type,
    name: String,
    parameters: Vec<Parameter>,
    attributes: Vec<FunctionAttribute>,
}

#[derive(Clone)]
pub(in crate::backend) struct Parameter {
    ty: Type,
    name: Option<String>,
    attributes: Vec<ParameterAttribute>,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum Linkage {
    Internal,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum FunctionAttribute {
    NoFree,
    NoInline,
    NoUnwind,
    WillReturn,
    MemoryNone,
    MemoryArgMemRead,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum ParameterAttribute {
    ImmArg,
}

impl FunctionSignature {
    pub(in crate::backend) fn new(
        result: Type,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            linkage: None,
            result,
            name: name.into(),
            parameters: parameters.into_iter().collect(),
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend) fn with_linkage(mut self, linkage: Linkage) -> Self {
        self.linkage = Some(linkage);
        self
    }

    pub(in crate::backend) fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = FunctionAttribute>,
    ) -> Self {
        self.attributes = attributes.into_iter().collect();
        self
    }

    pub(super) fn render(&self) -> String {
        let linkage = self
            .linkage
            .as_ref()
            .map_or(String::new(), |linkage| format!("{} ", linkage.render()));
        let attributes = if self.attributes.is_empty() {
            String::new()
        } else {
            format!(
                " {}",
                self.attributes
                    .iter()
                    .map(|attribute| attribute.render())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        format!(
            "{linkage}{} @{}({}){attributes}",
            self.result,
            self.name,
            self.parameters
                .iter()
                .map(Parameter::render)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn is_valid(&self) -> bool {
        is_valid_name(&self.name) && self.parameters.iter().all(Parameter::is_valid)
    }
}

impl Parameter {
    pub(in crate::backend) fn named(ty: Type, name: impl Into<String>) -> Self {
        Self {
            ty,
            name: Some(name.into()),
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend) fn unnamed(ty: Type) -> Self {
        Self {
            ty,
            name: None,
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend::llvm) fn with_attribute(
        mut self,
        attribute: ParameterAttribute,
    ) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub(super) fn render(&self) -> String {
        let mut output = self.ty.to_string();
        for attribute in &self.attributes {
            output.push(' ');
            output.push_str(attribute.render());
        }
        if let Some(name) = &self.name {
            output.push(' ');
            output.push_str(name);
        }
        output
    }

    pub(super) fn is_valid(&self) -> bool {
        self.name
            .as_deref()
            .is_none_or(|name| name.strip_prefix('%').is_some_and(is_valid_name))
    }
}

impl Linkage {
    pub(super) fn render(self) -> &'static str {
        match self {
            Self::Internal => "internal",
        }
    }
}

impl FunctionAttribute {
    pub(super) fn render(self) -> &'static str {
        match self {
            Self::NoFree => "nofree",
            Self::NoInline => "noinline",
            Self::NoUnwind => "nounwind",
            Self::WillReturn => "willreturn",
            Self::MemoryNone => "memory(none)",
            Self::MemoryArgMemRead => "memory(argmem: read)",
        }
    }
}

impl ParameterAttribute {
    fn render(self) -> &'static str {
        match self {
            Self::ImmArg => "immarg",
        }
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
    label: String,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
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

    fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    fn push(&mut self, instruction: Instruction) -> bool {
        if self.terminator.is_some() {
            return false;
        }
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
            instruction.render_into(output);
            output.push('\n');
        }
        if let Some(terminator) = &self.terminator {
            output.push_str("  ");
            terminator.render_into(output);
            output.push('\n');
        }
    }

    fn uses_byte_runtime(&self) -> bool {
        self.instructions.iter().any(Instruction::uses_byte_runtime)
            || self
                .terminator
                .as_ref()
                .is_some_and(Terminator::uses_byte_runtime)
    }
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
        ty: Type,
        value: String,
    },
    Switch {
        ty: Type,
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
        (super::instruction::is_atom(&condition)
            && is_valid_name(&then_target)
            && is_valid_name(&else_target))
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
        ty: Type,
        value: impl Into<String>,
    ) -> Option<Self> {
        let value = value.into();
        super::instruction::is_value(&value).then_some(Self::Return { ty, value })
    }

    pub(in crate::backend::llvm) fn switch(
        ty: Type,
        value: impl Into<String>,
        default: impl Into<String>,
        cases: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Option<Self> {
        let value = value.into();
        let default = default.into();
        let cases = cases
            .into_iter()
            .map(|(value, target)| (value.into(), target.into()))
            .collect::<Vec<_>>();
        (super::instruction::is_atom(&value)
            && is_valid_name(&default)
            && cases
                .iter()
                .all(|(value, target)| super::instruction::is_atom(value) && is_valid_name(target)))
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
    use crate::backend::llvm::syntax::Type;

    #[test]
    fn renders_late_entry_instructions_before_the_existing_entry_body() {
        let mut function = FunctionBuilder::new(
            FunctionSignature::new(Type::Void, "example", std::iter::empty::<Parameter>())
                .with_linkage(Linkage::Internal),
        );
        assert!(function.start_block("entry"));
        assert!(function.terminate(Terminator::branch("body").unwrap()));
        assert!(function.start_block("body"));
        assert!(function.terminate(Terminator::return_void()));
        function.structured_entry_instruction(
            Instruction::alloca("%storage", Type::integer(32_u16), 4).unwrap(),
        );

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
            Type::Void,
            "missing_terminator",
            std::iter::empty::<Parameter>(),
        ));
        assert!(function.start_block("entry"));
        assert!(
            function.structured_instruction(
                Instruction::call(
                    None::<String>,
                    false,
                    Type::Void,
                    super::super::Callee::direct("work").unwrap(),
                    [],
                )
                .unwrap()
            )
        );
        assert!(function.finish().is_none());
    }

    #[test]
    fn rejects_invalid_and_duplicate_block_labels() {
        assert!(
            BasicBlock::new(
                "0invalid",
                std::iter::empty::<Instruction>(),
                Terminator::return_void(),
            )
            .is_none()
        );

        let mut function = FunctionBuilder::new(FunctionSignature::new(
            Type::Void,
            "duplicate",
            std::iter::empty::<Parameter>(),
        ));
        assert!(function.start_block("entry"));
        assert!(function.terminate(Terminator::return_void()));
        assert!(!function.start_block("entry"));

        let entry = BasicBlock::new(
            "entry",
            std::iter::empty::<Instruction>(),
            Terminator::return_void(),
        )
        .unwrap();
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new(
                    Type::Void,
                    "duplicate_direct",
                    std::iter::empty::<Parameter>(),
                ),
                vec![entry.clone(), entry],
            )
            .is_none()
        );
    }

    #[test]
    fn derives_byte_runtime_requirements_from_emitted_instructions() {
        let signature =
            || FunctionSignature::new(Type::Void, "example", std::iter::empty::<Parameter>());
        let call = |callee, arguments: Vec<super::super::TypedValue>| {
            Instruction::call(
                None::<String>,
                false,
                Type::Void,
                super::super::Callee::direct(callee).unwrap(),
                arguments,
            )
            .unwrap()
        };
        let mut plain = FunctionBuilder::new(signature());
        assert!(plain.start_block("entry"));
        assert!(plain.structured_instruction(call("work", Vec::new())));
        assert!(plain.terminate(Terminator::return_void()));
        let plain = plain.finish().unwrap();

        let mut bytes = FunctionBuilder::new(signature());
        assert!(bytes.start_block("entry"));
        assert!(bytes.structured_instruction(call(
            "mal_runtime_bytes_release",
            vec![super::super::TypedValue::new(Type::Pointer, "null").unwrap()],
        )));
        assert!(bytes.terminate(Terminator::return_void()));
        let bytes = bytes.finish().unwrap();

        assert!(!plain.uses_byte_runtime());
        assert!(bytes.uses_byte_runtime());
    }

    #[test]
    fn rejects_invalid_function_signature_fragments() {
        let block = || {
            BasicBlock::new(
                "entry",
                std::iter::empty::<Instruction>(),
                Terminator::return_void(),
            )
            .unwrap()
        };
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new(Type::Void, "0invalid", std::iter::empty::<Parameter>()),
                vec![block()],
            )
            .is_none()
        );
        assert!(
            FunctionDefinition::new(
                FunctionSignature::new(
                    Type::Void,
                    "invalid",
                    [Parameter::named(Type::Pointer, "%bad\nname")],
                ),
                vec![block()],
            )
            .is_none()
        );
    }

    #[test]
    fn rejects_fragments_in_control_operands() {
        assert!(Terminator::conditional_branch("%condition, label %extra", "yes", "no").is_none());
        assert!(
            Terminator::switch(
                Type::integer(32_u16),
                "%tag",
                "other",
                [("0, label %injected", "zero")],
            )
            .is_none()
        );
    }
}
