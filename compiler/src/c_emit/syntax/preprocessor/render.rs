use std::fmt::Write as _;

use super::*;

impl PreprocessorExpr {
    fn render(&self, output: &mut String) {
        match self {
            Self::Defined(name) => {
                write!(output, "defined({name})").expect("writing generated C cannot fail");
            }
            Self::Identifier(name) => output.push_str(name),
            Self::Integer(value) => {
                write!(output, "{value}").expect("writing generated C cannot fail");
            }
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

impl Directive {
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
                            expression.render(&mut output);
                        }
                        MacroValue::Attribute(Attribute::Unused) => {
                            output.push_str("__attribute__((unused))");
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
                        PastePart::Text(value) => output.push_str(value),
                        PastePart::Parameter(value) => output.push_str(value),
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
            Self::Error(message) => {
                let mut output = String::from("#error ");
                message.render(&mut output);
                output.push('\n');
                output
            }
            Self::Pragma(Pragma::FenvAccessOn) => "#pragma STDC FENV_ACCESS ON\n".into(),
            Self::Pragma(Pragma::FpContractOff) => "#pragma STDC FP_CONTRACT OFF\n".into(),
        }
    }
}

impl MacroInvocation {
    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = format!("{}(", self.name);
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            argument.render(&mut output);
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
