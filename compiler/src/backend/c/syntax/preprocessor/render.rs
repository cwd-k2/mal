use std::fmt::Write as _;

use super::*;

impl PreprocessorExpr {
    fn render(&self, output: &mut String) {
        match self {
            Self::Defined(name) => {
                write!(output, "defined({name})").expect("writing generated C cannot fail");
            }
        }
    }
}

impl Directive {
    pub(in crate::backend) fn render(&self) -> String {
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
            Self::FunctionItemsDefine {
                name,
                parameters,
                declarations,
                definitions,
                trailing_signature,
            } => {
                let mut replacement = String::new();
                for declaration in declarations {
                    replacement.push_str(&declaration.render());
                    replacement.push_str(";\n");
                }
                for definition in definitions {
                    replacement.push_str(&definition.render());
                }
                trailing_signature.render_macro(&mut replacement);

                let mut output = format!("#define {name}(");
                render_macro_parameters(&mut output, parameters);
                output.push_str(") \\\n");
                let lines: Vec<_> = replacement.lines().collect();
                for (index, line) in lines.iter().enumerate() {
                    output.push_str(line);
                    if index + 1 != lines.len() && !line.ends_with(" \\") {
                        output.push_str(" \\\n");
                    } else if index + 1 != lines.len() {
                        output.push('\n');
                    }
                }
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
        }
    }
}

impl MacroInvocation {
    pub(in crate::backend) fn render(&self) -> String {
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
