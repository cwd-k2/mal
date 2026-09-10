mod render;

use super::Identifier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) struct TypeName {
    base: TypeBase,
    is_const: bool,
    pointer_depth: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TypeBase {
    Named(Identifier),
    Struct(Identifier),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) enum Declarator {
    Identifier(Identifier),
    FunctionPointer {
        name: Identifier,
        parameters: Vec<Parameter>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) struct VariableDeclaration {
    ty: TypeName,
    declarator: Declarator,
    is_static: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) struct Parameter {
    ty: TypeName,
    name: Option<Identifier>,
    maybe_unused: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend::c) enum FunctionSpecifier {
    Static,
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) struct FunctionSignature {
    specifiers: Vec<FunctionSpecifier>,
    result: TypeName,
    name: Identifier,
    parameters: Vec<Parameter>,
}

impl TypeName {
    pub(in crate::backend::c) fn named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: false,
            pointer_depth: 0,
        }
    }

    pub(in crate::backend::c) fn const_named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: true,
            pointer_depth: 0,
        }
    }

    pub(in crate::backend::c) fn pointer(mut self) -> Self {
        self.pointer_depth += 1;
        self
    }

    pub(in crate::backend::c) fn structure(tag: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Struct(tag.into()),
            is_const: false,
            pointer_depth: 0,
        }
    }
}

impl From<&str> for TypeName {
    fn from(value: &str) -> Self {
        Self::named(value)
    }
}

impl From<String> for TypeName {
    fn from(value: String) -> Self {
        Self::named(value)
    }
}

impl Declarator {
    pub(in crate::backend::c) fn identifier(name: impl Into<Identifier>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::backend::c) fn function_pointer(
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::FunctionPointer {
            name: name.into(),
            parameters: parameters.into_iter().collect(),
        }
    }
}

impl VariableDeclaration {
    pub(in crate::backend::c) fn new(ty: impl Into<TypeName>, name: impl Into<Identifier>) -> Self {
        Self {
            ty: ty.into(),
            declarator: Declarator::identifier(name),
            is_static: false,
        }
    }

    pub(in crate::backend::c) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            ty: result.into(),
            declarator: Declarator::function_pointer(name, parameters),
            is_static: false,
        }
    }
}

impl Parameter {
    pub(in crate::backend::c) fn named(
        ty: impl Into<TypeName>,
        name: impl Into<Identifier>,
    ) -> Self {
        Self {
            ty: ty.into(),
            name: Some(name.into()),
            maybe_unused: false,
        }
    }

    pub(in crate::backend::c) fn unnamed(ty: impl Into<TypeName>) -> Self {
        Self {
            ty: ty.into(),
            name: None,
            maybe_unused: false,
        }
    }

    pub(in crate::backend::c) fn maybe_unused(mut self) -> Self {
        self.maybe_unused = true;
        self
    }
}

impl FunctionSignature {
    pub(in crate::backend::c) fn new(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            specifiers: Vec::new(),
            result: result.into(),
            name: name.into(),
            parameters: parameters.into_iter().collect(),
        }
    }

    pub(in crate::backend::c) fn with_specifiers(
        mut self,
        specifiers: impl IntoIterator<Item = FunctionSpecifier>,
    ) -> Self {
        self.specifiers = specifiers.into_iter().collect();
        self
    }

    pub(in crate::backend::c) fn static_function(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::Static])
    }

    pub(in crate::backend::c) fn static_inline(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters)
            .with_specifiers([FunctionSpecifier::Static, FunctionSpecifier::Inline])
    }

    pub(in crate::backend::c) fn no_return(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::NoReturn])
    }
}

#[cfg(test)]
mod tests;
