mod render;

use super::{Expr, Identifier};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct TypeName {
    base: TypeBase,
    is_const: bool,
    pointer_const: Vec<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TypeBase {
    Named(Identifier),
    Struct(Identifier),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Declarator {
    Identifier(Identifier),
    Array {
        name: Identifier,
        size: Box<Expr>,
    },
    FunctionPointer {
        name: Identifier,
        parameters: Vec<Parameter>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct VariableDeclaration {
    ty: TypeName,
    declarator: Declarator,
    is_static: bool,
    alignment: Option<Box<Expr>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct Parameter {
    ty: TypeName,
    name: Option<Identifier>,
    maybe_unused: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum FunctionSpecifier {
    Static,
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct FunctionSignature {
    specifiers: Vec<FunctionSpecifier>,
    result: TypeName,
    name: Identifier,
    parameters: Vec<Parameter>,
}

impl TypeName {
    pub(in crate::backend) fn named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: false,
            pointer_const: Vec::new(),
        }
    }

    pub(in crate::backend) fn const_named(base: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: true,
            pointer_const: Vec::new(),
        }
    }

    pub(in crate::backend) fn pointer(mut self) -> Self {
        self.pointer_const.push(false);
        self
    }

    pub(in crate::backend) fn const_pointer(mut self) -> Self {
        self.pointer_const.push(true);
        self
    }

    pub(in crate::backend) fn structure(tag: impl Into<Identifier>) -> Self {
        Self {
            base: TypeBase::Struct(tag.into()),
            is_const: false,
            pointer_const: Vec::new(),
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
    pub(in crate::backend) fn identifier(name: impl Into<Identifier>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::backend) fn function_pointer(
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
    pub(in crate::backend) fn new(ty: impl Into<TypeName>, name: impl Into<Identifier>) -> Self {
        Self {
            ty: ty.into(),
            declarator: Declarator::identifier(name),
            is_static: false,
            alignment: None,
        }
    }

    pub(in crate::backend) fn array(
        ty: impl Into<TypeName>,
        name: impl Into<Identifier>,
        size: Expr,
    ) -> Self {
        Self {
            ty: ty.into(),
            declarator: Declarator::Array {
                name: name.into(),
                size: Box::new(size),
            },
            is_static: false,
            alignment: None,
        }
    }

    pub(in crate::backend) fn aligned(mut self, alignment: Expr) -> Self {
        self.alignment = Some(Box::new(alignment));
        self
    }

    pub(in crate::backend) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            ty: result.into(),
            declarator: Declarator::function_pointer(name, parameters),
            is_static: false,
            alignment: None,
        }
    }
}

impl Parameter {
    pub(in crate::backend) fn named(ty: impl Into<TypeName>, name: impl Into<Identifier>) -> Self {
        Self {
            ty: ty.into(),
            name: Some(name.into()),
            maybe_unused: false,
        }
    }

    pub(in crate::backend) fn unnamed(ty: impl Into<TypeName>) -> Self {
        Self {
            ty: ty.into(),
            name: None,
            maybe_unused: false,
        }
    }

    pub(in crate::backend) fn maybe_unused(mut self) -> Self {
        self.maybe_unused = true;
        self
    }
}

impl FunctionSignature {
    pub(in crate::backend) fn new(
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

    pub(in crate::backend) fn with_specifiers(
        mut self,
        specifiers: impl IntoIterator<Item = FunctionSpecifier>,
    ) -> Self {
        self.specifiers = specifiers.into_iter().collect();
        self
    }

    pub(in crate::backend) fn static_function(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::Static])
    }

    pub(in crate::backend) fn static_inline(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters)
            .with_specifiers([FunctionSpecifier::Static, FunctionSpecifier::Inline])
    }

    pub(in crate::backend) fn no_return(
        result: impl Into<TypeName>,
        name: impl Into<Identifier>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::NoReturn])
    }
}

#[cfg(test)]
mod tests;
