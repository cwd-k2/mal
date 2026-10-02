use super::lexical::is_valid_name;
use super::{FunctionAttribute, FunctionDefinition, Parameter, Type};
use std::collections::HashSet;

#[derive(Clone)]
pub(in crate::backend) struct FunctionDeclaration {
    result: Type,
    name: String,
    parameters: Vec<Parameter>,
    attributes: Vec<FunctionAttribute>,
}

impl FunctionDeclaration {
    pub(in crate::backend) fn new(
        result: Type,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            result,
            name: name.into(),
            parameters: parameters.into_iter().collect(),
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend) fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = FunctionAttribute>,
    ) -> Self {
        self.attributes = attributes.into_iter().collect();
        self
    }

    fn render(&self) -> String {
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
            "declare {} @{}({}){attributes}",
            self.result,
            self.name,
            self.parameters
                .iter()
                .map(Parameter::render)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_valid(&self) -> bool {
        is_valid_name(&self.name) && self.parameters.iter().all(Parameter::is_valid)
    }
}

pub(in crate::backend::llvm) struct Module<'a> {
    triple: &'a str,
    data_layout: &'a str,
    declarations: Vec<FunctionDeclaration>,
    items: Vec<ModuleItem>,
}

enum ModuleItem {
    Global(GlobalDefinition),
    Function(FunctionDefinition),
    Metadata(Vec<MetadataDefinition>),
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct MetadataDefinition {
    id: usize,
    distinct: bool,
    operands: Vec<MetadataOperand>,
}

#[derive(Clone)]
pub(in crate::backend::llvm) enum MetadataOperand {
    Node(usize),
    Text(String),
    Integer { ty: Type, value: i128 },
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct GlobalDefinition {
    name: String,
    kind: GlobalKind,
}

#[derive(Clone)]
enum GlobalKind {
    ByteOwner { bytes: Vec<u8>, alignment: usize },
}

impl GlobalDefinition {
    pub(in crate::backend::llvm) fn byte_owner(
        name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        alignment: usize,
    ) -> Option<Self> {
        let name = name.into();
        (is_valid_name(&name) && alignment.is_power_of_two()).then_some(Self {
            name,
            kind: GlobalKind::ByteOwner {
                bytes: bytes.into(),
                alignment,
            },
        })
    }

    fn render(&self) -> String {
        match &self.kind {
            GlobalKind::ByteOwner { bytes, alignment } => {
                let contents = bytes
                    .iter()
                    .map(|byte| match byte {
                        0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7e => (*byte as char).to_string(),
                        _ => format!("\\{byte:02X}"),
                    })
                    .collect::<String>();
                format!(
                    "@{} = private constant {{ i64, i64, i8, [7 x i8], [{} x i8] }} {{ i64 -1, i64 {}, i8 0, [7 x i8] zeroinitializer, [{} x i8] c\"{contents}\" }}, align {alignment}",
                    self.name,
                    bytes.len(),
                    bytes.len(),
                    bytes.len(),
                )
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Module<'a> {
    pub(in crate::backend::llvm) fn new(triple: &'a str, data_layout: &'a str) -> Self {
        Self {
            triple,
            data_layout,
            declarations: Vec::new(),
            items: Vec::new(),
        }
    }

    pub(in crate::backend::llvm) fn declare(&mut self, declaration: FunctionDeclaration) {
        self.declarations.push(declaration);
    }

    pub(in crate::backend::llvm) fn add_global(&mut self, definition: GlobalDefinition) {
        self.items.push(ModuleItem::Global(definition));
    }

    pub(in crate::backend::llvm) fn define(&mut self, definition: FunctionDefinition) {
        self.items.push(ModuleItem::Function(definition));
    }

    pub(in crate::backend::llvm) fn add_metadata(
        &mut self,
        metadata: impl IntoIterator<Item = MetadataDefinition>,
    ) {
        let metadata = metadata.into_iter().collect::<Vec<_>>();
        if !metadata.is_empty() {
            self.items.push(ModuleItem::Metadata(metadata));
        }
    }

    pub(in crate::backend::llvm) fn render(&self) -> Option<String> {
        let mut symbols = HashSet::new();
        self.declarations
            .iter()
            .all(|declaration| declaration.is_valid() && symbols.insert(declaration.name()))
            .then_some(())?;
        self.items
            .iter()
            .all(|item| item.name().is_none_or(|name| symbols.insert(name)))
            .then_some(())?;
        let mut sections = vec![format!(
            "target datalayout = {:?}\ntarget triple = {:?}",
            self.data_layout, self.triple
        )];
        if !self.declarations.is_empty() {
            sections.push(
                self.declarations
                    .iter()
                    .map(FunctionDeclaration::render)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        sections.extend(self.items.iter().map(ModuleItem::render));
        Some(sections.join("\n\n") + "\n")
    }
}

impl MetadataDefinition {
    pub(in crate::backend::llvm) fn new(
        id: usize,
        distinct: bool,
        operands: impl IntoIterator<Item = MetadataOperand>,
    ) -> Self {
        Self {
            id,
            distinct,
            operands: operands.into_iter().collect(),
        }
    }

    fn render(&self) -> String {
        let distinct = if self.distinct { "distinct " } else { "" };
        format!(
            "!{} = {distinct}!{{{}}}",
            self.id,
            self.operands
                .iter()
                .map(MetadataOperand::render)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl MetadataOperand {
    fn render(&self) -> String {
        match self {
            Self::Node(id) => format!("!{id}"),
            Self::Text(text) => format!("!{:?}", text),
            Self::Integer { ty, value } => format!("{ty} {value}"),
        }
    }
}

impl ModuleItem {
    fn render(&self) -> String {
        match self {
            Self::Global(definition) => definition.render(),
            Self::Metadata(definitions) => definitions
                .iter()
                .map(MetadataDefinition::render)
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Function(definition) => definition.render(),
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            Self::Global(definition) => Some(definition.name()),
            Self::Function(definition) => Some(definition.name()),
            Self::Metadata(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::llvm::syntax::{
        BasicBlock, Callee, FunctionSignature, Instruction, Terminator, Type,
    };

    #[test]
    fn renders_only_the_declarations_added_to_a_module() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new(
            Type::Void,
            "always",
            std::iter::empty::<Parameter>(),
        ));
        module.define(
            FunctionDefinition::new(
                FunctionSignature::new(Type::Void, "entry", std::iter::empty::<Parameter>()),
                vec![
                    BasicBlock::new(
                        "entry",
                        [Instruction::call(
                            None::<String>,
                            false,
                            Type::Void,
                            Callee::direct("always").unwrap(),
                            [],
                        )
                        .unwrap()],
                        Terminator::return_void(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );

        assert_eq!(
            module.render().unwrap(),
            concat!(
                "target datalayout = \"e-p:64:64\"\n",
                "target triple = \"test-target\"\n\n",
                "declare void @always()\n\n",
                "define void @entry() {\n",
                "entry:\n",
                "  call void @always()\n",
                "  ret void\n",
                "}\n",
            )
        );
        assert!(!module.render().unwrap().contains("unused"));
    }

    #[test]
    fn rejects_duplicate_module_symbols() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new(
            Type::Void,
            "duplicate",
            std::iter::empty::<Parameter>(),
        ));
        module.declare(FunctionDeclaration::new(
            Type::Void,
            "duplicate",
            std::iter::empty::<Parameter>(),
        ));

        assert!(module.render().is_none());
    }

    #[test]
    fn rejects_invalid_declaration_fragments() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new(
            Type::Void,
            "0invalid",
            std::iter::empty::<Parameter>(),
        ));

        assert!(module.render().is_none());
    }
}
