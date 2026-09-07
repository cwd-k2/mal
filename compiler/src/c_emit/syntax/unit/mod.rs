use super::{Expr, FunctionSignature, Identifier, TypeName, VariableDeclaration};

mod render;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionSignature),
    TypeAlias { source: TypeName, alias: Identifier },
    StaticAssert { condition: Expr, message: Expr },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Comment(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct AggregateDefinition {
    kind: AggregateKind,
    tag: Option<Identifier>,
    fields: Vec<AggregateField>,
    is_typedef: bool,
    alias: Option<Identifier>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum AggregateKind {
    Struct,
    Union,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum AggregateField {
    Declaration(VariableDeclaration),
    Aggregate {
        kind: AggregateKind,
        fields: Vec<Self>,
        name: Identifier,
    },
}

impl Declaration {
    pub(in crate::c_emit) fn variable(declaration: VariableDeclaration) -> Self {
        Self::Variable(declaration)
    }

    pub(in crate::c_emit) fn function(signature: FunctionSignature) -> Self {
        Self::Function(signature)
    }

    pub(in crate::c_emit) fn type_alias(
        source: impl Into<TypeName>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self::TypeAlias {
            source: source.into(),
            alias: alias.into(),
        }
    }

    pub(in crate::c_emit) fn static_assert(condition: Expr, message: impl Into<String>) -> Self {
        Self::StaticAssert {
            condition,
            message: Expr::string(message),
        }
    }
}

impl Comment {
    pub(in crate::c_emit) fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl AggregateDefinition {
    pub(in crate::c_emit) fn structure(
        tag: impl Into<Identifier>,
        fields: impl IntoIterator<Item = AggregateField>,
    ) -> Self {
        Self {
            kind: AggregateKind::Struct,
            tag: Some(tag.into()),
            fields: fields.into_iter().collect(),
            is_typedef: false,
            alias: None,
        }
    }

    pub(in crate::c_emit) fn typedef_structure(
        tag: Option<String>,
        fields: impl IntoIterator<Item = AggregateField>,
        alias: impl Into<Identifier>,
    ) -> Self {
        Self {
            kind: AggregateKind::Struct,
            tag: tag.map(Identifier::from),
            fields: fields.into_iter().collect(),
            is_typedef: true,
            alias: Some(alias.into()),
        }
    }
}

impl AggregateField {
    pub(in crate::c_emit) fn variable(
        ty: impl Into<TypeName>,
        name: impl Into<Identifier>,
    ) -> Self {
        Self::Declaration(VariableDeclaration::new(ty, name))
    }

    pub(in crate::c_emit) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = super::Parameter>,
    ) -> Self {
        Self::Declaration(VariableDeclaration::function_pointer(
            result, name, parameters,
        ))
    }

    pub(in crate::c_emit) fn aggregate(
        kind: AggregateKind,
        fields: impl IntoIterator<Item = Self>,
        name: impl Into<Identifier>,
    ) -> Self {
        Self::Aggregate {
            kind,
            fields: fields.into_iter().collect(),
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests;
