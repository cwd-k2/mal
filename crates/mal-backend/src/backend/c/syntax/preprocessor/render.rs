use super::*;
use crate::backend::c::syntax::render::{MacroReplacementWriter, RenderWrite};

impl PreprocessorExpr {
    fn render(&self, output: &mut impl RenderWrite) {
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
                let mut prefix = format!("#define {name}(");
                render_macro_parameters(&mut prefix, parameters);
                prefix.push_str(") \\\n");
                let mut output = MacroReplacementWriter::new(prefix);
                for declaration in declarations {
                    declaration.render_into(&mut output);
                    output.push_str(";\n");
                }
                for definition in definitions {
                    definition.render_into(&mut output);
                }
                trailing_signature.render_multiline(&mut output);
                output.push('\n');
                output.finish()
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
    pub(in crate::backend::c::syntax) fn render_into(&self, output: &mut impl RenderWrite) {
        write!(output, "{}(", self.name).expect("writing generated C cannot fail");
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            argument.render(output);
        }
        output.push(')');
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
