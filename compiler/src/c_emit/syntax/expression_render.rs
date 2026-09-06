use std::fmt;

use super::expression::Expr;

impl Expr {
    pub(super) fn render(&self, output: &mut String) {
        use std::fmt::Write as _;

        match self {
            Self::Number(value) => output.push_str(value),
            Self::Identifier(value) => output.push_str(value),
            Self::Character(value) => render_character(output, *value),
            Self::StringLiteral(value) => value.render(output),
            Self::ByteString(value) => {
                output.push('"');
                for byte in value {
                    write!(output, "\\x{byte:02x}").expect("writing generated C cannot fail");
                }
                output.push('"');
            }
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
            Self::Index { value, index } => {
                value.render_postfix_operand(output);
                output.push('[');
                index.render(output);
                output.push(']');
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
            Self::Conditional {
                condition,
                then,
                otherwise,
            } => {
                condition.render_conditional_condition(output);
                output.push_str(" ? ");
                then.render(output);
                output.push_str(" : ");
                otherwise.render(output);
            }
            Self::SizeofType(ty) => {
                write!(output, "sizeof({ty})").expect("writing generated C cannot fail");
            }
            Self::SizeofValue(value) => {
                output.push_str("sizeof(");
                value.render(output);
                output.push(')');
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
            Self::InitializerList(fields) => {
                output.push_str("{ ");
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
                | Self::Character(_)
                | Self::StringLiteral(_)
                | Self::ByteString(_)
                | Self::Identifier(_)
                | Self::Call { .. }
                | Self::Field { .. }
                | Self::Index { .. }
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
                | Self::Character(_)
                | Self::StringLiteral(_)
                | Self::ByteString(_)
                | Self::Identifier(_)
                | Self::Call { .. }
                | Self::Field { .. }
                | Self::Index { .. }
                | Self::Cast { .. }
                | Self::SizeofType(_)
                | Self::SizeofValue(_)
        ) {
            self.render(output);
        } else {
            output.push('(');
            self.render(output);
            output.push(')');
        }
    }

    fn render_binary_operand(&self, output: &mut String) {
        if matches!(self, Self::Binary { .. } | Self::Conditional { .. }) {
            output.push('(');
            self.render(output);
            output.push(')');
        } else {
            self.render(output);
        }
    }

    fn render_conditional_condition(&self, output: &mut String) {
        if matches!(self, Self::Conditional { .. }) {
            output.push('(');
            self.render(output);
            output.push(')');
        } else {
            self.render(output);
        }
    }
}

fn render_character(output: &mut String, value: char) {
    output.push('\'');
    match value {
        '\'' => output.push_str("\\'"),
        '\\' => output.push_str("\\\\"),
        '\n' => output.push_str("\\n"),
        '\r' => output.push_str("\\r"),
        '\t' => output.push_str("\\t"),
        value if value.is_ascii_graphic() || value == ' ' => output.push(value),
        value => {
            use std::fmt::Write as _;
            write!(output, "\\x{:02x}", u32::from(value)).expect("writing generated C cannot fail");
        }
    }
    output.push('\'');
}

impl fmt::Display for Expr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        self.render(&mut output);
        formatter.write_str(&output)
    }
}
