use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Expr {
    Literal(String),
    Identifier(String),
    Call {
        callee: Box<Self>,
        arguments: Vec<Self>,
    },
    Field {
        value: Box<Self>,
        name: String,
        indirect: bool,
    },
    Index {
        value: Box<Self>,
        index: Box<Self>,
    },
    Cast {
        ty: String,
        value: Box<Self>,
    },
    Unary {
        operator: &'static str,
        operand: Box<Self>,
    },
    Binary {
        operator: &'static str,
        left: Box<Self>,
        right: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then: Box<Self>,
        otherwise: Box<Self>,
    },
    SizeofType(String),
    CompoundLiteral {
        ty: String,
        fields: Vec<Initializer>,
    },
    InitializerList(Vec<Initializer>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Initializer {
    designator: Option<String>,
    value: Expr,
}

impl Expr {
    pub(in crate::c_emit) fn literal(value: impl Into<String>) -> Self {
        Self::Literal(value.into())
    }

    pub(in crate::c_emit) fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::c_emit) fn call(callee: Self, arguments: impl IntoIterator<Item = Self>) -> Self {
        Self::Call {
            callee: Box::new(callee),
            arguments: arguments.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn named_call(
        name: impl Into<String>,
        arguments: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self::call(Self::identifier(name), arguments)
    }

    pub(in crate::c_emit) fn field(self, name: impl Into<String>) -> Self {
        Self::Field {
            value: Box::new(self),
            name: name.into(),
            indirect: false,
        }
    }

    pub(in crate::c_emit) fn pointer_field(self, name: impl Into<String>) -> Self {
        Self::Field {
            value: Box::new(self),
            name: name.into(),
            indirect: true,
        }
    }

    pub(in crate::c_emit) fn index(self, index: Self) -> Self {
        Self::Index {
            value: Box::new(self),
            index: Box::new(index),
        }
    }

    pub(in crate::c_emit) fn cast(ty: impl Into<String>, value: Self) -> Self {
        Self::Cast {
            ty: ty.into(),
            value: Box::new(value),
        }
    }

    pub(in crate::c_emit) fn unary(operator: &'static str, operand: Self) -> Self {
        Self::Unary {
            operator,
            operand: Box::new(operand),
        }
    }

    pub(in crate::c_emit) fn binary(operator: &'static str, left: Self, right: Self) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub(in crate::c_emit) fn conditional(condition: Self, then: Self, otherwise: Self) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }

    pub(in crate::c_emit) fn sizeof_type(ty: impl Into<String>) -> Self {
        Self::SizeofType(ty.into())
    }

    pub(in crate::c_emit) fn compound_literal(
        ty: impl Into<String>,
        fields: impl IntoIterator<Item = Initializer>,
    ) -> Self {
        Self::CompoundLiteral {
            ty: ty.into(),
            fields: fields.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn initializer_list(
        fields: impl IntoIterator<Item = Initializer>,
    ) -> Self {
        Self::InitializerList(fields.into_iter().collect())
    }

    fn render(&self, output: &mut String) {
        use std::fmt::Write as _;

        match self {
            Self::Literal(value) | Self::Identifier(value) => output.push_str(value),
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
                output.push_str(operator);
                operand.render_unary_operand(output);
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                left.render_binary_operand(output);
                write!(output, " {operator} ").expect("writing generated C cannot fail");
                right.render_binary_operand(output);
            }
            Self::Conditional {
                condition,
                then,
                otherwise,
            } => {
                condition.render(output);
                output.push_str(" ? ");
                then.render(output);
                output.push_str(" : ");
                otherwise.render(output);
            }
            Self::SizeofType(ty) => {
                write!(output, "sizeof({ty})").expect("writing generated C cannot fail");
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
            Self::Literal(_)
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
            Self::Literal(_)
                | Self::Identifier(_)
                | Self::Call { .. }
                | Self::Field { .. }
                | Self::Index { .. }
                | Self::Cast { .. }
                | Self::Unary { .. }
                | Self::SizeofType(_)
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
}

impl Initializer {
    pub(in crate::c_emit) fn positional(value: Expr) -> Self {
        Self {
            designator: None,
            value,
        }
    }

    pub(in crate::c_emit) fn designated(name: impl Into<String>, value: Expr) -> Self {
        Self {
            designator: Some(name.into()),
            value,
        }
    }

    fn render(&self, output: &mut String) {
        if let Some(designator) = &self.designator {
            output.push('.');
            output.push_str(designator);
            output.push_str(" = ");
        }
        self.value.render(output);
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        self.render(&mut output);
        formatter.write_str(&output)
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Initializer};

    #[test]
    fn renders_composed_expressions() {
        let expression = Expr::named_call(
            "consume",
            [
                Expr::identifier("object").pointer_field("value"),
                Expr::cast(
                    "uint64_t",
                    Expr::binary("+", Expr::identifier("left"), Expr::identifier("right")),
                ),
            ],
        );

        assert_eq!(
            expression.to_string(),
            "consume(object->value, (uint64_t)(left + right))"
        );
    }

    #[test]
    fn renders_compound_literals_with_designators() {
        let expression = Expr::compound_literal(
            "Pair",
            [
                Initializer::designated("first", Expr::identifier("left")),
                Initializer::positional(Expr::identifier("right")),
            ],
        );

        assert_eq!(expression.to_string(), "(Pair){ .first = left, right }");
    }
}
