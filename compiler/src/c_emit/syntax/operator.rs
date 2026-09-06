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

impl UnaryOperator {
    pub(super) fn symbol(self) -> &'static str {
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
    pub(super) fn symbol(self) -> &'static str {
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
