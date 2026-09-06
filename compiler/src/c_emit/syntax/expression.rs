use std::fmt;

use super::TypeName;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum UnaryOperator {
    AddressOf,
    Dereference,
    Negate,
    BitwiseNot,
    LogicalNot,
    PreIncrement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum BinaryOperator {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Expr {
    Number(String),
    Character(char),
    StringLiteral(String),
    ByteString(Vec<u8>),
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
        ty: TypeName,
        value: Box<Self>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Self>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Self>,
        right: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then: Box<Self>,
        otherwise: Box<Self>,
    },
    SizeofType(TypeName),
    SizeofValue(Box<Self>),
    CompoundLiteral {
        ty: TypeName,
        fields: Vec<Initializer>,
    },
    InitializerList(Vec<Initializer>),
}

macro_rules! unary_constructors {
    ($($method:ident => $operator:ident),+ $(,)?) => {
        $(
            pub(in crate::c_emit) fn $method(operand: Self) -> Self {
                Self::unary(UnaryOperator::$operator, operand)
            }
        )+
    };
}

macro_rules! binary_constructors {
    ($($method:ident => $operator:ident),+ $(,)?) => {
        $(
            pub(in crate::c_emit) fn $method(left: Self, right: Self) -> Self {
                Self::binary(BinaryOperator::$operator, left, right)
            }
        )+
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Initializer {
    designators: Vec<Designator>,
    value: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Designator {
    Field(String),
}

impl Expr {
    pub(in crate::c_emit) fn number(value: impl Into<String>) -> Self {
        Self::Number(value.into())
    }

    pub(in crate::c_emit) fn character(value: char) -> Self {
        Self::Character(value)
    }

    pub(in crate::c_emit) fn string(value: impl Into<String>) -> Self {
        Self::StringLiteral(value.into())
    }

    pub(in crate::c_emit) fn byte_string(value: impl Into<Vec<u8>>) -> Self {
        Self::ByteString(value.into())
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

    pub(in crate::c_emit) fn cast(ty: impl Into<TypeName>, value: Self) -> Self {
        Self::Cast {
            ty: ty.into(),
            value: Box::new(value),
        }
    }

    pub(in crate::c_emit) fn unary(operator: UnaryOperator, operand: Self) -> Self {
        Self::Unary {
            operator,
            operand: Box::new(operand),
        }
    }

    pub(in crate::c_emit) fn binary(operator: BinaryOperator, left: Self, right: Self) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    unary_constructors! {
        address_of => AddressOf,
        dereference => Dereference,
        negate => Negate,
        bitwise_not => BitwiseNot,
        logical_not => LogicalNot,
        pre_increment => PreIncrement,
    }

    binary_constructors! {
        multiply => Multiply,
        divide => Divide,
        add => Add,
        subtract => Subtract,
        shift_left => ShiftLeft,
        shift_right => ShiftRight,
        less => Less,
        greater => Greater,
        greater_equal => GreaterEqual,
        equal => Equal,
        not_equal => NotEqual,
        bitwise_and => BitwiseAnd,
        bitwise_or => BitwiseOr,
        logical_and => LogicalAnd,
        logical_or => LogicalOr,
    }

    pub(in crate::c_emit) fn conditional(condition: Self, then: Self, otherwise: Self) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }

    pub(in crate::c_emit) fn sizeof_type(ty: impl Into<TypeName>) -> Self {
        Self::SizeofType(ty.into())
    }

    pub(in crate::c_emit) fn sizeof_expr(value: Self) -> Self {
        Self::SizeofValue(Box::new(value))
    }

    pub(in crate::c_emit) fn compound_literal(
        ty: impl Into<TypeName>,
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
            Self::Number(value) | Self::Identifier(value) => output.push_str(value),
            Self::Character(value) => render_character(output, *value),
            Self::StringLiteral(value) => render_string(output, value),
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
                condition.render(output);
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
                | Self::Unary { .. }
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
}

impl UnaryOperator {
    fn symbol(self) -> &'static str {
        match self {
            Self::AddressOf => "&",
            Self::Dereference => "*",
            Self::Negate => "-",
            Self::BitwiseNot => "~",
            Self::LogicalNot => "!",
            Self::PreIncrement => "++",
        }
    }
}

impl BinaryOperator {
    fn symbol(self) -> &'static str {
        match self {
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::BitwiseAnd => "&",
            Self::BitwiseXor => "^",
            Self::BitwiseOr => "|",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
        }
    }
}

impl Initializer {
    pub(in crate::c_emit) fn positional(value: Expr) -> Self {
        Self {
            designators: Vec::new(),
            value,
        }
    }

    pub(in crate::c_emit) fn designated(name: impl Into<String>, value: Expr) -> Self {
        Self {
            designators: vec![Designator::Field(name.into())],
            value,
        }
    }

    pub(in crate::c_emit) fn designated_path(
        path: impl IntoIterator<Item = impl Into<String>>,
        value: Expr,
    ) -> Self {
        Self {
            designators: path
                .into_iter()
                .map(|name| Designator::Field(name.into()))
                .collect(),
            value,
        }
    }

    fn render(&self, output: &mut String) {
        for designator in &self.designators {
            match designator {
                Designator::Field(name) => {
                    output.push('.');
                    output.push_str(name);
                }
            }
        }
        if !self.designators.is_empty() {
            output.push_str(" = ");
        }
        self.value.render(output);
    }
}

fn render_string(output: &mut String, value: &str) {
    use std::fmt::Write as _;

    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_ascii_graphic() || character == ' ' => {
                output.push(character);
            }
            character => {
                write!(output, "\\x{:02x}", u32::from(character))
                    .expect("writing generated C cannot fail");
            }
        }
    }
    output.push('"');
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
                    Expr::add(Expr::identifier("left"), Expr::identifier("right")),
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

    #[test]
    fn renders_character_literals() {
        assert_eq!(Expr::character('\n').to_string(), "'\\n'");
        assert_eq!(Expr::character('\'').to_string(), "'\\''");
    }
}
