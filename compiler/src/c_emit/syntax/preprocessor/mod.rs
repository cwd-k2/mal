use std::ops::Deref;

use super::{Expr, FunctionSignature, Identifier, StringLiteral};

mod render;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum BinaryOperator {
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum PreprocessorExpr {
    Defined(Identifier),
    Identifier(Identifier),
    Integer(u64),
    Binary {
        operator: BinaryOperator,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum MacroValue {
    Expression(Expr),
    Attribute(Attribute),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Attribute {
    Unused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Pragma {
    FenvAccessOn,
    FpContractOff,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum PastePart {
    Text(TokenFragment),
    Parameter(Identifier),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct TokenFragment(String);

impl TokenFragment {
    fn new(value: String) -> Self {
        assert!(
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
            "generated C token fragment is invalid: {value:?}"
        );
        Self(value)
    }
}

impl Deref for TokenFragment {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct MacroParameter(Identifier);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct IncludePath(String);

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
pub(in crate::c_emit) enum Directive {
    IncludeQuoted(IncludePath),
    IncludeSystem(IncludePath),
    Define {
        name: Identifier,
        value: Option<MacroValue>,
    },
    FunctionAlias {
        name: Identifier,
        parameters: Vec<MacroParameter>,
        replacement: Vec<PastePart>,
    },
    FunctionSignatureDefine {
        name: Identifier,
        parameters: Vec<MacroParameter>,
        signature: FunctionSignature,
    },
    If(PreprocessorExpr),
    Ifndef(Identifier),
    Else,
    Endif,
    Error(StringLiteral),
    Pragma(Pragma),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct MacroInvocation {
    name: Identifier,
    arguments: Vec<Expr>,
}

impl PreprocessorExpr {
    pub(in crate::c_emit) fn defined(name: impl Into<Identifier>) -> Self {
        Self::Defined(name.into())
    }

    pub(in crate::c_emit) fn identifier(name: impl Into<Identifier>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::c_emit) fn integer(value: u64) -> Self {
        Self::Integer(value)
    }

    fn binary(operator: BinaryOperator, left: Self, right: Self) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub(in crate::c_emit) fn not_equal(left: Self, right: Self) -> Self {
        Self::binary(BinaryOperator::NotEqual, left, right)
    }

    pub(in crate::c_emit) fn logical_and(left: Self, right: Self) -> Self {
        Self::binary(BinaryOperator::LogicalAnd, left, right)
    }

    pub(in crate::c_emit) fn logical_or(left: Self, right: Self) -> Self {
        Self::binary(BinaryOperator::LogicalOr, left, right)
    }
}

impl PastePart {
    pub(in crate::c_emit) fn text(value: impl Into<String>) -> Self {
        Self::Text(TokenFragment::new(value.into()))
    }

    pub(in crate::c_emit) fn parameter(name: impl Into<Identifier>) -> Self {
        Self::Parameter(name.into())
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
    pub(in crate::c_emit) fn include_quoted(path: impl Into<String>) -> Self {
        let path = path.into();
        assert!(Self::is_valid_quoted_include(&path));
        Self::IncludeQuoted(IncludePath(path))
    }

    pub(in crate::c_emit) fn include_system(path: impl Into<String>) -> Self {
        let path = path.into();
        assert!(is_valid_include_path(&path, ['<', '>']));
        Self::IncludeSystem(IncludePath(path))
    }

    pub(in crate::c_emit) fn is_valid_quoted_include(path: &str) -> bool {
        is_valid_include_path(path, ['"', '\\'])
    }

    pub(in crate::c_emit) fn error(message: impl Into<String>) -> Self {
        Self::Error(StringLiteral::new(message))
    }

    pub(in crate::c_emit) fn define_empty(name: impl Into<Identifier>) -> Self {
        Self::Define {
            name: name.into(),
            value: None,
        }
    }

    pub(in crate::c_emit) fn define_expr(name: impl Into<Identifier>, value: Expr) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Expression(value)),
        }
    }

    pub(in crate::c_emit) fn define_attribute(
        name: impl Into<Identifier>,
        attribute: Attribute,
    ) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Attribute(attribute)),
        }
    }

    pub(in crate::c_emit) fn function_alias(
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = impl Into<MacroParameter>>,
        replacement: impl IntoIterator<Item = PastePart>,
    ) -> Self {
        Self::FunctionAlias {
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            replacement: replacement.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn function_signature_define(
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = impl Into<MacroParameter>>,
        signature: FunctionSignature,
    ) -> Self {
        Self::FunctionSignatureDefine {
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            signature,
        }
    }

    pub(in crate::c_emit) fn pragma(pragma: Pragma) -> Self {
        Self::Pragma(pragma)
    }
}

fn is_valid_include_path(path: &str, forbidden: [char; 2]) -> bool {
    !path.is_empty()
        && !path
            .chars()
            .any(|character| character.is_control() || forbidden.contains(&character))
}

impl MacroInvocation {
    pub(in crate::c_emit) fn new(
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
