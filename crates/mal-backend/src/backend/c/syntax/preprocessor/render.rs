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
            Self::ExpressionDefine {
                name,
                parameters,
                expression,
            } => {
                let mut output = format!("#define {name}(");
                render_macro_parameters(&mut output, parameters);
                output.push_str(") ");
                expression.render(&mut output);
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
            Self::FunctionDefinitionsDefine {
                name,
                parameters,
                definitions,
            } => {
                let mut prefix = format!("#define {name}(");
                render_macro_parameters(&mut prefix, parameters);
                prefix.push_str(") \\\n");
                let mut output = MacroReplacementWriter::new(prefix);
                for definition in definitions {
                    definition.render_into(&mut output);
                }
                output.finish()
            }
            Self::InvocationsDefine {
                name,
                parameters,
                invocations,
            } => {
                if invocations.is_empty() {
                    let mut output = format!("#define {name}(");
                    render_macro_parameters(&mut output, parameters);
                    output.push_str(")\n");
                    output
                } else {
                    render_replacement(name, parameters, |output| {
                        for invocation in invocations {
                            invocation.render_into(output);
                            output.push('\n');
                        }
                    })
                }
            }
            Self::AggregateDefine {
                name,
                parameters,
                definition,
            } => render_replacement(name, parameters, |output| {
                definition.render_into(output);
            }),
            Self::AggregateFieldsDefine {
                name,
                parameters,
                fields,
            } => render_replacement(name, parameters, |output| {
                crate::backend::c::syntax::unit::render::render_fields(output, fields, 0);
            }),
            Self::InitializersDefine {
                name,
                parameters,
                initializers,
            } => render_replacement(name, parameters, |output| {
                for initializer in initializers {
                    initializer.render(output);
                    output.push_str(",\n");
                }
            }),
            Self::StatementsDefine {
                name,
                parameters,
                statements,
            } => render_replacement(name, parameters, |output| {
                for statement in statements {
                    statement.render(output, 0);
                }
            }),
            Self::SwitchCasesDefine {
                name,
                parameters,
                cases,
            } => render_replacement(name, parameters, |output| {
                for case in cases {
                    case.render(output, 0);
                }
            }),
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

fn render_replacement(
    name: &Identifier,
    parameters: &[MacroParameter],
    render: impl FnOnce(&mut MacroReplacementWriter),
) -> String {
    let mut prefix = format!("#define {name}(");
    render_macro_parameters(&mut prefix, parameters);
    prefix.push_str(") \\\n");
    let mut output = MacroReplacementWriter::new(prefix);
    render(&mut output);
    output.finish()
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
