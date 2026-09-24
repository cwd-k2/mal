use std::ops::Deref;

use super::{Expr, FunctionDefinition, FunctionSignature, Identifier};

mod render;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum PreprocessorExpr {
    Defined(Identifier),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum MacroValue {
    Expression(Expr),
    Attribute(Attribute),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Attribute {
    Unused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct MacroParameter(Identifier);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct IncludePath(String);

impl Deref for IncludePath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for IncludePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Directive {
    IncludeQuoted(IncludePath),
    IncludeSystem(IncludePath),
    Define {
        name: Identifier,
        value: Option<MacroValue>,
    },
    FunctionItemsDefine {
        name: Identifier,
        parameters: Vec<MacroParameter>,
        declarations: Vec<FunctionSignature>,
        definitions: Vec<FunctionDefinition>,
        trailing_signature: FunctionSignature,
    },
    If(PreprocessorExpr),
    Ifndef(Identifier),
    Else,
    Endif,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct MacroInvocation {
    name: Identifier,
    arguments: Vec<Expr>,
}

impl PreprocessorExpr {
    pub(in crate::backend) fn defined(name: impl Into<Identifier>) -> Self {
        Self::Defined(name.into())
    }
}

impl From<&str> for MacroParameter {
    fn from(value: &str) -> Self {
        Self(Identifier::from(value))
    }
}

impl From<String> for MacroParameter {
    fn from(value: String) -> Self {
        Self(Identifier::from(value))
    }
}

impl Directive {
    pub(in crate::backend) fn include_quoted(path: impl Into<String>) -> Self {
        let path = path.into();
        assert!(Self::is_valid_quoted_include(&path));
        Self::IncludeQuoted(IncludePath(path))
    }

    pub(in crate::backend) fn include_system(path: impl Into<String>) -> Self {
        let path = path.into();
        assert!(is_valid_include_path(&path, ['<', '>']));
        Self::IncludeSystem(IncludePath(path))
    }

    pub(in crate::backend) fn is_valid_quoted_include(path: &str) -> bool {
        is_valid_include_path(path, ['"', '\\'])
    }

    pub(in crate::backend) fn define_empty(name: impl Into<Identifier>) -> Self {
        Self::Define {
            name: name.into(),
            value: None,
        }
    }

    pub(in crate::backend) fn define_expr(name: impl Into<Identifier>, value: Expr) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Expression(value)),
        }
    }

    pub(in crate::backend) fn define_attribute(
        name: impl Into<Identifier>,
        attribute: Attribute,
    ) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Attribute(attribute)),
        }
    }

    pub(in crate::backend) fn function_items_define(
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = impl Into<MacroParameter>>,
        declarations: impl IntoIterator<Item = FunctionSignature>,
        definitions: impl IntoIterator<Item = FunctionDefinition>,
        trailing_signature: FunctionSignature,
    ) -> Self {
        Self::FunctionItemsDefine {
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            declarations: declarations.into_iter().collect(),
            definitions: definitions.into_iter().collect(),
            trailing_signature,
        }
    }
}

fn is_valid_include_path(path: &str, forbidden: [char; 2]) -> bool {
    !path.is_empty()
        && !path
            .chars()
            .any(|character| character.is_control() || forbidden.contains(&character))
}

impl MacroInvocation {
    pub(in crate::backend) fn new(
        name: impl Into<Identifier>,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Self {
        Self {
            name: name.into(),
            arguments: arguments.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests;
