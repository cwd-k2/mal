use super::{Expr, FunctionSignature, Identifier, MacroInvocation, TypeName, VariableDeclaration};

pub(in crate::backend::c::syntax) mod render;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Declaration {
    Function(FunctionSignature),
    TypeAlias { source: TypeName, alias: Identifier },
    StaticAssert { condition: Expr, message: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct Comment(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct RecordDefinition {
    kind: RecordKind,
    tag: Option<Identifier>,
    fields: Vec<RecordField>,
    is_typedef: bool,
    alias: Option<Identifier>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum RecordKind {
    Struct,
    Union,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum RecordField {
    Declaration(VariableDeclaration),
    MacroInvocation(MacroInvocation),
    Record {
        kind: RecordKind,
        fields: Vec<Self>,
        name: Identifier,
    },
}

impl Declaration {
    pub(in crate::backend) fn function(signature: FunctionSignature) -> Self {
        Self::Function(signature)
    }

    pub(in crate::backend) fn type_alias(
        source: impl Into<TypeName>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self::TypeAlias {
            source: source.into(),
            alias: alias.into(),
        }
    }

    pub(in crate::backend) fn static_assert(condition: Expr, message: impl Into<String>) -> Self {
        Self::StaticAssert {
            condition,
            message: message.into(),
        }
    }
}

impl Comment {
    pub(in crate::backend) fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl RecordDefinition {
    pub(in crate::backend) fn structure(
        tag: impl Into<Identifier>,
        fields: impl IntoIterator<Item = RecordField>,
    ) -> Self {
        Self {
            kind: RecordKind::Struct,
            tag: Some(tag.into()),
            fields: fields.into_iter().collect(),
            is_typedef: false,
            alias: None,
        }
    }

    pub(in crate::backend) fn typedef_structure(
        tag: Option<String>,
        fields: impl IntoIterator<Item = RecordField>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self::typedef(RecordKind::Struct, tag, fields, alias)
    }

    #[allow(
        dead_code,
        reason = "called by generated c_items! and c_record! expansions"
    )]
    pub(in crate::backend) fn union(
        tag: impl Into<Identifier>,
        fields: impl IntoIterator<Item = RecordField>,
    ) -> Self {
        Self {
            kind: RecordKind::Union,
            tag: Some(tag.into()),
            fields: fields.into_iter().collect(),
            is_typedef: false,
            alias: None,
        }
    }

    #[allow(dead_code, reason = "called by generated c_items! expansions")]
    pub(in crate::backend) fn typedef_union(
        tag: Option<String>,
        fields: impl IntoIterator<Item = RecordField>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self::typedef(RecordKind::Union, tag, fields, alias)
    }

    fn typedef(
        kind: RecordKind,
        tag: Option<String>,
        fields: impl IntoIterator<Item = RecordField>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self {
            kind,
            tag: tag.map(Identifier::from),
            fields: fields.into_iter().collect(),
            is_typedef: true,
            alias: Some(alias.into()),
        }
    }
}

impl RecordField {
    pub(in crate::backend) fn variable(
        ty: impl Into<TypeName>,
        name: impl Into<Identifier>,
    ) -> Self {
        Self::Declaration(VariableDeclaration::new(ty, name))
    }

    pub(in crate::backend) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = super::Parameter>,
    ) -> Self {
        Self::Declaration(VariableDeclaration::function_pointer(
            result, name, parameters,
        ))
    }

    pub(in crate::backend) fn record(
        kind: RecordKind,
        fields: impl IntoIterator<Item = Self>,
        name: impl Into<Identifier>,
    ) -> Self {
        Self::Record {
            kind,
            fields: fields.into_iter().collect(),
            name: name.into(),
        }
    }
}

impl From<MacroInvocation> for RecordField {
    fn from(value: MacroInvocation) -> Self {
        Self::MacroInvocation(value)
    }
}

#[cfg(test)]
mod tests;
