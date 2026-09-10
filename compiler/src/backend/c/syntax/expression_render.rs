use std::fmt;

use super::expression::Expr;

impl Expr {
    pub(super) fn render(&self, output: &mut String) {
        use std::fmt::Write as _;

        match self {
            Self::Number(value) => output.push_str(value),
            Self::Identifier(value) => output.push_str(value),
            Self::StringLiteral(value) => value.render(output),
            Self::Call { callee, arguments } => {
                callee.render_postfix_operand(output);
                output.push('(');
                for (index, argument) in arguments.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    argument.render(output);
                }
                output.push(')');
            }
            Self::Field {
                value,
                name,
                indirect,
            } => {
                value.render_postfix_operand(output);
                output.push_str(if *indirect { "->" } else { "." });
                output.push_str(name);
            }
            Self::Cast { ty, value } => {
                write!(output, "({ty})").expect("writing generated C cannot fail");
                value.render_unary_operand(output);
            }
            Self::Unary { operator, operand } => {
                output.push_str(operator.symbol());
                operand.render_unary_operand(output);
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                left.render_binary_operand(output);
                write!(output, " {} ", operator.symbol()).expect("writing generated C cannot fail");
                right.render_binary_operand(output);
            }
            Self::CompoundLiteral { ty, fields } => {
                write!(output, "({ty}){{ ").expect("writing generated C cannot fail");
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    field.render(output);
                }
                output.push_str(" }");
            }
        }
    }

    fn render_postfix_operand(&self, output: &mut String) {
        if matches!(
            self,
            Self::Number(_)
                | Self::StringLiteral(_)
                | Self::Identifier(_)
                | Self::Call { .. }
                | Self::Field { .. }
        ) {
            self.render(output);
        } else {
            output.push('(');
            self.render(output);
            output.push(')');
        }
    }

    fn render_unary_operand(&self, output: &mut String) {
        if matches!(
            self,
            Self::Number(_)
                | Self::StringLiteral(_)
                | Self::Identifier(_)
                | Self::Call { .. }
                | Self::Field { .. }
                | Self::Cast { .. }
        ) {
            self.render(output);
        } else {
            output.push('(');
            self.render(output);
            output.push(')');
        }
    }

    fn render_binary_operand(&self, output: &mut String) {
        if matches!(self, Self::Binary { .. }) {
            output.push('(');
            self.render(output);
            output.push(')');
        } else {
            self.render(output);
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        self.render(&mut output);
        formatter.write_str(&output)
    }
}
