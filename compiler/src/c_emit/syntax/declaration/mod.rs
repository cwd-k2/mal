mod render;

use super::Identifier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct TypeName {
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
pub(in crate::c_emit) enum Declarator {
    Identifier(Identifier),
    FunctionPointer {
        name: Identifier,
        parameters: Vec<Parameter>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct VariableDeclaration {
    ty: TypeName,
    declarator: Declarator,
    is_static: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Parameter {
    ty: TypeName,
    name: Option<Identifier>,
    maybe_unused: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum FunctionSpecifier {
    Static,
    Inline,
    NoInline,
    NoReturn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct FunctionSignature {
    specifiers: Vec<FunctionSpecifier>,
    result: TypeName,
    name: Identifier,
    parameters: Vec<Parameter>,
    maybe_unused: bool,
}

impl TypeName {
    pub(in crate::c_emit) fn named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: false,
            pointer_depth: 0,
        }
    }

    pub(in crate::c_emit) fn const_named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: true,
            pointer_depth: 0,
        }
    }

    pub(in crate::c_emit) fn pointer(mut self) -> Self {
        self.pointer_depth += 1;
        self
    }

    pub(in crate::c_emit) fn structure(tag: impl Into<Identifier>) -> Self {
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
    pub(in crate::c_emit) fn identifier(name: impl Into<Identifier>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::c_emit) fn function_pointer(
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
    pub(in crate::c_emit) fn new(ty: impl Into<TypeName>, name: impl Into<Identifier>) -> Self {
        Self {
            ty: ty.into(),
            declarator: Declarator::identifier(name),
            is_static: false,
        }
    }

    pub(in crate::c_emit) fn static_variable(
        ty: impl Into<TypeName>,
        name: impl Into<Identifier>,
    ) -> Self {
        let mut declaration = Self::new(ty, name);
        declaration.is_static = true;
        declaration
    }

    pub(in crate::c_emit) fn function_pointer(
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
    pub(in crate::c_emit) fn named(ty: impl Into<TypeName>, name: impl Into<Identifier>) -> Self {
        Self {
            ty: ty.into(),
            name: Some(name.into()),
            maybe_unused: false,
        }
    }

    pub(in crate::c_emit) fn unnamed(ty: impl Into<TypeName>) -> Self {
        Self {
            ty: ty.into(),
            name: None,
            maybe_unused: false,
        }
    }

    pub(in crate::c_emit) fn maybe_unused(mut self) -> Self {
        self.maybe_unused = true;
        self
    }
}

impl FunctionSignature {
    pub(in crate::c_emit) fn new(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            specifiers: Vec::new(),
            result: result.into(),
            name: name.into(),
            parameters: parameters.into_iter().collect(),
            maybe_unused: false,
        }
    }

    pub(in crate::c_emit) fn maybe_unused(mut self) -> Self {
        self.maybe_unused = true;
        self
    }

    pub(in crate::c_emit) fn with_specifiers(
        mut self,
        specifiers: impl IntoIterator<Item = FunctionSpecifier>,
    ) -> Self {
        self.specifiers = specifiers.into_iter().collect();
        self
    }

    pub(in crate::c_emit) fn static_function(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::Static])
    }

    pub(in crate::c_emit) fn static_inline(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters)
            .with_specifiers([FunctionSpecifier::Static, FunctionSpecifier::Inline])
    }

    pub(in crate::c_emit) fn static_noinline(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters)
            .with_specifiers([FunctionSpecifier::Static, FunctionSpecifier::NoInline])
    }

    pub(in crate::c_emit) fn no_return(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::NoReturn])
    }
}

#[cfg(test)]
mod tests;
