use super::FunctionDefinition;
use super::function::{is_single_line, is_valid_name};
use std::collections::HashSet;

#[derive(Clone)]
pub(in crate::backend) struct FunctionDeclaration {
    result: String,
    name: String,
    parameters: Vec<String>,
    attributes: Vec<String>,
}

impl FunctionDeclaration {
    pub(in crate::backend) fn new(
        result: impl Into<String>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            result: result.into(),
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            attributes: Vec::new(),
        }
    }

    pub(in crate::backend) fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.attributes = attributes.into_iter().map(Into::into).collect();
        self
    }

    fn render(&self) -> String {
        let attributes = if self.attributes.is_empty() {
            String::new()
        } else {
            format!(" {}", self.attributes.join(" "))
        };
        format!(
            "declare {} @{}({}){attributes}",
            self.result,
            self.name,
            self.parameters.join(", ")
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_valid(&self) -> bool {
        is_single_line(&self.result)
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

pub(in crate::backend::llvm) struct Module<'a> {
    triple: &'a str,
    data_layout: &'a str,
    declarations: Vec<FunctionDeclaration>,
    items: Vec<ModuleItem>,
}

enum ModuleItem {
    Global(GlobalDefinition),
    Function(FunctionDefinition),
    Metadata(String),
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct GlobalDefinition {
    name: String,
    definition: String,
}

impl GlobalDefinition {
    pub(in crate::backend::llvm) fn new(
        name: impl Into<String>,
        definition: impl Into<String>,
    ) -> Option<Self> {
        let name = name.into();
        let definition = definition.into();
        (is_valid_name(&name)
            && !definition.trim().is_empty()
            && !definition.contains(['\n', '\r']))
        .then_some(Self { name, definition })
    }

    fn render(&self) -> String {
        self.definition.clone()
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

    pub(in crate::backend::llvm) fn add_metadata(&mut self, metadata: impl Into<String>) {
        self.add_nonempty(metadata, ModuleItem::Metadata);
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

    fn add_nonempty(
        &mut self,
        fragment: impl Into<String>,
        item: impl FnOnce(String) -> ModuleItem,
    ) {
        let fragment = fragment.into();
        if !fragment.trim().is_empty() {
            self.items.push(item(fragment.trim_end().into()));
        }
    }
}

impl ModuleItem {
    fn render(&self) -> String {
        match self {
            Self::Global(definition) => definition.render(),
            Self::Metadata(fragment) => fragment.clone(),
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
    use crate::backend::llvm::syntax::{BasicBlock, FunctionSignature};

    #[test]
    fn renders_only_the_declarations_added_to_a_module() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new(
            "void",
            "always",
            std::iter::empty::<&str>(),
        ));
        module.define(
            FunctionDefinition::new(
                FunctionSignature::new("void", "entry", std::iter::empty::<&str>()),
                vec![BasicBlock::new("entry", ["call void @always()", "ret void"]).unwrap()],
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
            "void",
            "duplicate",
            std::iter::empty::<&str>(),
        ));
        module.declare(FunctionDeclaration::new(
            "void",
            "duplicate",
            std::iter::empty::<&str>(),
        ));

        assert!(module.render().is_none());
    }

    #[test]
    fn rejects_invalid_declaration_fragments() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new(
            "void",
            "0invalid",
            std::iter::empty::<&str>(),
        ));

        assert!(module.render().is_none());
    }
}
