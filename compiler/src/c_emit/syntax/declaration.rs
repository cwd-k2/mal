use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct TypeName {
    base: TypeBase,
    is_const: bool,
    pointer_depth: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TypeBase {
    Named(String),
    Struct(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Declarator {
    Identifier(String),
    FunctionPointer {
        name: String,
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
    name: Option<String>,
    maybe_unused: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum FunctionSpecifier {
    Static,
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct FunctionSignature {
    specifiers: Vec<FunctionSpecifier>,
    result: TypeName,
    name: String,
    parameters: Vec<Parameter>,
}

impl TypeName {
    pub(in crate::c_emit) fn named(base: impl Into<String>) -> Self {
        Self {
            base: TypeBase::Named(base.into()),
            is_const: false,
            pointer_depth: 0,
        }
    }

    pub(in crate::c_emit) fn const_named(base: impl Into<String>) -> Self {
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

    pub(in crate::c_emit) fn structure(tag: impl Into<String>) -> Self {
        Self {
            base: TypeBase::Struct(tag.into()),
            is_const: false,
            pointer_depth: 0,
        }
    }

    pub(in crate::c_emit) fn render_declarator(&self, declarator: &str) -> String {
        let mut output = String::new();
        if self.is_const {
            output.push_str("const ");
        }
        self.base.render(&mut output);
        if self.pointer_depth == 0 {
            output.push(' ');
        } else {
            output.push(' ');
            for _ in 0..self.pointer_depth {
                output.push('*');
            }
        }
        output.push_str(declarator);
        output
    }
}

impl Display for TypeName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.is_const {
            formatter.write_str("const ")?;
        }
        match &self.base {
            TypeBase::Named(name) => formatter.write_str(name)?,
            TypeBase::Struct(tag) => write!(formatter, "struct {tag}")?,
        }
        if self.pointer_depth != 0 {
            formatter.write_str(" ")?;
            for _ in 0..self.pointer_depth {
                formatter.write_str("*")?;
            }
        }
        Ok(())
    }
}

impl TypeBase {
    fn render(&self, output: &mut String) {
        match self {
            Self::Named(name) => output.push_str(name),
            Self::Struct(tag) => {
                output.push_str("struct ");
                output.push_str(tag);
            }
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
    pub(in crate::c_emit) fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::c_emit) fn function_pointer(
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::FunctionPointer {
            name: name.into(),
            parameters: parameters.into_iter().collect(),
        }
    }

    fn render(&self, ty: &TypeName) -> String {
        match self {
            Self::Identifier(name) => ty.render_declarator(name),
            Self::FunctionPointer { name, parameters } => {
                format!("{} (*{name})({})", ty, render_parameters(parameters))
            }
        }
    }
}

impl VariableDeclaration {
    pub(in crate::c_emit) fn new(ty: impl Into<TypeName>, name: impl Into<String>) -> Self {
        Self {
            ty: ty.into(),
            declarator: Declarator::identifier(name),
            is_static: false,
        }
    }

    pub(in crate::c_emit) fn static_variable(
        ty: impl Into<TypeName>,
        name: impl Into<String>,
    ) -> Self {
        let mut declaration = Self::new(ty, name);
        declaration.is_static = true;
        declaration
    }

    pub(in crate::c_emit) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            ty: result.into(),
            declarator: Declarator::function_pointer(name, parameters),
            is_static: false,
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let declaration = self.declarator.render(&self.ty);
        if self.is_static {
            format!("static {declaration}")
        } else {
            declaration
        }
    }
}

impl Parameter {
    pub(in crate::c_emit) fn named(ty: impl Into<TypeName>, name: impl Into<String>) -> Self {
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

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = match &self.name {
            Some(name) => self.ty.render_declarator(name),
            None => self.ty.to_string(),
        };
        if self.maybe_unused {
            output.push_str(" MAL_DETAIL_MAYBE_UNUSED");
        }
        output
    }
}

impl FunctionSpecifier {
    fn spelling(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Inline => "inline",
            Self::NoReturn => "_Noreturn",
        }
    }
}

impl FunctionSignature {
    pub(in crate::c_emit) fn new(
        result: impl Into<TypeName>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self {
            specifiers: Vec::new(),
            result: result.into(),
            name: name.into(),
            parameters: parameters.into_iter().collect(),
        }
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
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::Static])
    }

    pub(in crate::c_emit) fn static_inline(
        result: impl Into<TypeName>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters)
            .with_specifiers([FunctionSpecifier::Static, FunctionSpecifier::Inline])
    }

    pub(in crate::c_emit) fn no_return(
        result: impl Into<TypeName>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(result, name, parameters).with_specifiers([FunctionSpecifier::NoReturn])
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = String::new();
        for specifier in &self.specifiers {
            output.push_str(match specifier {
                FunctionSpecifier::Static => "static ",
                FunctionSpecifier::Inline => "inline ",
                FunctionSpecifier::NoReturn => "_Noreturn ",
            });
        }
        output.push_str(&self.result.render_declarator(&self.name));
        output.push('(');
        output.push_str(&render_parameters(&self.parameters));
        output.push(')');
        output
    }

    pub(in crate::c_emit) fn render_macro(&self, output: &mut String) {
        for specifier in &self.specifiers {
            output.push_str(specifier.spelling());
            output.push(' ');
        }
        output.push_str(&self.result.render_declarator(&self.name));
        if self.parameters.is_empty() {
            output.push_str("(void)");
            return;
        }
        output.push_str("( \\\n");
        for (index, parameter) in self.parameters.iter().enumerate() {
            output.push_str("    ");
            output.push_str(&parameter.render());
            if index + 1 != self.parameters.len() {
                output.push(',');
            }
            output.push_str(" \\\n");
        }
        output.push(')');
    }
}

fn render_parameters(parameters: &[Parameter]) -> String {
    if parameters.is_empty() {
        return "void".into();
    }
    let mut output = String::new();
    for (index, parameter) in parameters.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&parameter.render());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{FunctionSignature, Parameter, TypeName, VariableDeclaration};

    #[test]
    fn renders_structured_function_signatures() {
        let signature = FunctionSignature::static_inline(
            TypeName::const_named("uint8_t").pointer(),
            "read_bytes",
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                Parameter::named("uint64_t", "length"),
            ],
        );

        assert_eq!(
            signature.render(),
            "static inline const uint8_t *read_bytes(MalContext *context, uint64_t length)"
        );
    }

    #[test]
    fn renders_function_pointer_declarators() {
        let declaration = VariableDeclaration::function_pointer(
            "int32_t",
            "call",
            [Parameter::unnamed(TypeName::const_named("void").pointer())],
        );

        assert_eq!(declaration.render(), "int32_t (*call)(const void *)");
    }
}
