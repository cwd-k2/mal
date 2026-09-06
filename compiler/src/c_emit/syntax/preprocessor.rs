use std::fmt::Write as _;

use super::{Expr, FunctionSignature};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum BinaryOperator {
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum PreprocessorExpr {
    Defined(String),
    Identifier(String),
    Integer(String),
    Binary {
        operator: BinaryOperator,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum MacroValue {
    Expression(Expr),
    Attribute(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum PastePart {
    Text(String),
    Parameter(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct MacroParameter(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Directive {
    IncludeQuoted(String),
    IncludeSystem(String),
    Define {
        name: String,
        value: Option<MacroValue>,
    },
    FunctionAlias {
        name: String,
        parameters: Vec<MacroParameter>,
        replacement: Vec<PastePart>,
    },
    FunctionSignatureDefine {
        name: String,
        parameters: Vec<MacroParameter>,
        signature: FunctionSignature,
    },
    If(PreprocessorExpr),
    Ifndef(String),
    Else,
    Endif,
    Error(String),
    Pragma {
        namespace: String,
        name: String,
        value: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct MacroInvocation {
    name: String,
    arguments: Vec<Expr>,
}

impl PreprocessorExpr {
    pub(in crate::c_emit) fn defined(name: impl Into<String>) -> Self {
        Self::Defined(name.into())
    }

    pub(in crate::c_emit) fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::c_emit) fn integer(value: impl Into<String>) -> Self {
        Self::Integer(value.into())
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

    fn render(&self, output: &mut String) {
        match self {
            Self::Defined(name) => {
                write!(output, "defined({name})").expect("writing generated C cannot fail");
            }
            Self::Identifier(name) | Self::Integer(name) => output.push_str(name),
            Self::Binary {
                operator,
                left,
                right,
            } => {
                left.render(output);
                write!(output, " {} ", operator.symbol()).expect("writing generated C cannot fail");
                right.render(output);
            }
        }
    }
}

impl BinaryOperator {
    fn symbol(self) -> &'static str {
        match self {
            Self::NotEqual => "!=",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
        }
    }
}

impl PastePart {
    pub(in crate::c_emit) fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub(in crate::c_emit) fn parameter(name: impl Into<String>) -> Self {
        Self::Parameter(name.into())
    }
}

impl From<&str> for MacroParameter {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for MacroParameter {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Directive {
    pub(in crate::c_emit) fn define_empty(name: impl Into<String>) -> Self {
        Self::Define {
            name: name.into(),
            value: None,
        }
    }

    pub(in crate::c_emit) fn define_expr(name: impl Into<String>, value: Expr) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Expression(value)),
        }
    }

    pub(in crate::c_emit) fn define_attribute(
        name: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        Self::Define {
            name: name.into(),
            value: Some(MacroValue::Attribute(attribute.into())),
        }
    }

    pub(in crate::c_emit) fn function_alias(
        name: impl Into<String>,
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
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = impl Into<MacroParameter>>,
        signature: FunctionSignature,
    ) -> Self {
        Self::FunctionSignatureDefine {
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            signature,
        }
    }

    pub(in crate::c_emit) fn pragma(
        namespace: impl Into<String>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Pragma {
            namespace: namespace.into(),
            name: name.into(),
            value: value.into(),
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        match self {
            Self::IncludeQuoted(path) => format!("#include \"{path}\"\n"),
            Self::IncludeSystem(path) => format!("#include <{path}>\n"),
            Self::Define { name, value } => {
                let mut output = format!("#define {name}");
                if let Some(value) = value {
                    output.push(' ');
                    match value {
                        MacroValue::Expression(expression) => {
                            output.push_str(&expression.to_string());
                        }
                        MacroValue::Attribute(attribute) => {
                            write!(output, "__attribute__(({attribute}))")
                                .expect("writing generated C cannot fail");
                        }
                    }
                }
                output.push('\n');
                output
            }
            Self::FunctionAlias {
                name,
                parameters,
                replacement,
            } => {
                let mut output = format!("#define {name}(");
                render_macro_parameters(&mut output, parameters);
                output.push_str(") ");
                for (index, part) in replacement.iter().enumerate() {
                    if index != 0 {
                        output.push_str("##");
                    }
                    match part {
                        PastePart::Text(value) | PastePart::Parameter(value) => {
                            output.push_str(value);
                        }
                    }
                }
                output.push('\n');
                output
            }
            Self::FunctionSignatureDefine {
                name,
                parameters,
                signature,
            } => {
                let mut output = format!("#define {name}(");
                render_macro_parameters(&mut output, parameters);
                output.push_str(") \\\n");
                signature.render_macro(&mut output);
                output.push('\n');
                output
            }
            Self::If(condition) => {
                let mut output = String::from("#if ");
                condition.render(&mut output);
                output.push('\n');
                output
            }
            Self::Ifndef(name) => format!("#ifndef {name}\n"),
            Self::Else => "#else\n".into(),
            Self::Endif => "#endif\n".into(),
            Self::Error(message) => format!("#error \"{}\"\n", message.replace('"', "\\\"")),
            Self::Pragma {
                namespace,
                name,
                value,
            } => format!("#pragma {namespace} {name} {value}\n"),
        }
    }
}

impl MacroInvocation {
    pub(in crate::c_emit) fn new(
        name: impl Into<String>,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Self {
        Self {
            name: name.into(),
            arguments: arguments.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = format!("{}(", self.name);
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            output.push_str(&argument.to_string());
        }
        output.push(')');
        output
    }
}

fn render_macro_parameters(output: &mut String, parameters: &[MacroParameter]) {
    for (index, parameter) in parameters.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&parameter.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{Directive, PastePart, PreprocessorExpr};

    #[test]
    fn renders_token_pasting_function_aliases() {
        let directive = Directive::function_alias(
            "MAL_TYPE",
            ["name"],
            [PastePart::text("MalType_"), PastePart::parameter("name")],
        );
        assert_eq!(
            directive.render(),
            "#define MAL_TYPE(name) MalType_##name\n"
        );
    }

    #[test]
    fn renders_typed_condition_operators() {
        let condition = PreprocessorExpr::logical_and(
            PreprocessorExpr::defined("FEATURE"),
            PreprocessorExpr::not_equal(
                PreprocessorExpr::identifier("FEATURE"),
                PreprocessorExpr::integer("1"),
            ),
        );

        assert_eq!(
            Directive::If(condition).render(),
            "#if defined(FEATURE) && FEATURE != 1\n"
        );
    }
}
