use super::*;

impl Statement {
    pub(in crate::backend) fn render(&self, output: &mut String, depth: usize) {
        match self {
            Self::VariableDeclaration {
                declaration,
                initializer,
            } => {
                write_indent(output, depth);
                output.push_str(&declaration.render());
                if let Some(initializer) = initializer {
                    output.push_str(" = ");
                    initializer.render(output);
                }
                output.push_str(";\n");
            }
            Self::Expression(expression) => {
                write_indent(output, depth);
                expression.render(output);
                output.push_str(";\n");
            }
            Self::Return(value) => {
                write_indent(output, depth);
                output.push_str("return ");
                value.render(output);
                output.push_str(";\n");
            }
            Self::ReturnVoid => {
                write_indent(output, depth);
                output.push_str("return;\n");
            }
            Self::If { condition, then } => {
                write_indent(output, depth);
                output.push_str("if (");
                condition.render(output);
                output.push_str(") ");
                then.render_braced(output, depth);
            }
            Self::For {
                initializer,
                initial_value,
                condition,
                step,
                body,
            } => {
                write_indent(output, depth);
                output.push_str("for (");
                output.push_str(&initializer.render());
                output.push_str(" = ");
                initial_value.render(output);
                output.push_str("; ");
                condition.render(output);
                output.push_str("; ");
                step.render(output);
                output.push_str(") ");
                body.render_braced(output, depth);
            }
            Self::Switch { value, cases } => {
                write_indent(output, depth);
                output.push_str("switch (");
                value.render(output);
                output.push_str(") {\n");
                for case in cases {
                    case.render(output, depth + 1);
                }
                write_indent(output, depth);
                output.push_str("}\n");
            }
        }
    }
}

impl Block {
    fn render_braced(&self, output: &mut String, depth: usize) {
        output.push_str("{\n");
        for statement in &self.statements {
            statement.render(output, depth + 1);
        }
        write_indent(output, depth);
        output.push_str("}\n");
    }
}

impl SwitchCase {
    fn render(&self, output: &mut String, depth: usize) {
        write_indent(output, depth);
        match &self.label {
            Some(label) => {
                output.push_str("case ");
                label.render(output);
            }
            None => output.push_str("default"),
        }
        output.push_str(": {\n");
        for statement in &self.body.statements {
            statement.render(output, depth + 1);
        }
        write_indent(output, depth);
        output.push_str("}\n");
    }
}

impl FunctionDefinition {
    pub(in crate::backend) fn render(&self) -> String {
        let mut output = match &self.header {
            FunctionHeader::Signature(signature) => signature.render(),
            FunctionHeader::MacroInvocation(invocation) => invocation.render(),
        };
        output.push(' ');
        self.body.render_braced(&mut output, 0);
        output
    }
}

fn write_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("    ");
    }
}
