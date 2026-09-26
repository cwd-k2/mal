use super::FunctionDefinition;

pub(in crate::backend::llvm) struct FunctionDeclaration(String);

impl FunctionDeclaration {
    pub(in crate::backend::llvm) fn new(signature: impl Into<String>) -> Self {
        Self(signature.into())
    }

    fn render(&self) -> String {
        format!("declare {}", self.0)
    }
}

pub(in crate::backend::llvm) struct Module<'a> {
    triple: &'a str,
    data_layout: &'a str,
    declarations: Vec<FunctionDeclaration>,
    items: Vec<ModuleItem>,
}

enum ModuleItem {
    GlobalFragment(String),
    Function(FunctionDefinition),
    Metadata(String),
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

    pub(in crate::backend::llvm) fn add_global_fragment(&mut self, fragment: impl Into<String>) {
        self.add_nonempty(fragment, ModuleItem::GlobalFragment);
    }

    pub(in crate::backend::llvm) fn define(&mut self, definition: FunctionDefinition) {
        self.items.push(ModuleItem::Function(definition));
    }

    pub(in crate::backend::llvm) fn add_metadata(&mut self, metadata: impl Into<String>) {
        self.add_nonempty(metadata, ModuleItem::Metadata);
    }

    pub(in crate::backend::llvm) fn render(&self) -> String {
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
        sections.join("\n\n") + "\n"
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
            Self::GlobalFragment(fragment) | Self::Metadata(fragment) => fragment.clone(),
            Self::Function(definition) => definition.render(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::llvm::syntax::BasicBlock;

    #[test]
    fn renders_only_the_declarations_added_to_a_module() {
        let mut module = Module::new("test-target", "e-p:64:64");
        module.declare(FunctionDeclaration::new("void @always()"));
        module.define(FunctionDefinition::new(
            "void @entry()",
            vec![BasicBlock::new("entry", ["call void @always()", "ret void"]).unwrap()],
        ));

        assert_eq!(
            module.render(),
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
        assert!(!module.render().contains("unused"));
    }
}
