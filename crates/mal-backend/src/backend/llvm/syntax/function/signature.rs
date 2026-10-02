//! Function signatures: linkage, result, parameters, and their attributes.

use super::*;

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

    pub(in crate::backend::llvm::syntax) fn render(&self) -> String {
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

    pub(in crate::backend::llvm::syntax) fn name(&self) -> &str {
        &self.name
    }

    pub(in crate::backend::llvm::syntax) fn is_valid(&self) -> bool {
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

    pub(in crate::backend::llvm::syntax) fn render(&self) -> String {
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

    pub(in crate::backend::llvm::syntax) fn is_valid(&self) -> bool {
        self.name
            .as_deref()
            .is_none_or(|name| name.strip_prefix('%').is_some_and(is_valid_name))
    }
}

impl Linkage {
    pub(in crate::backend::llvm::syntax) fn render(self) -> &'static str {
        match self {
            Self::Internal => "internal",
        }
    }
}

impl FunctionAttribute {
    pub(in crate::backend::llvm::syntax) fn render(self) -> &'static str {
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
