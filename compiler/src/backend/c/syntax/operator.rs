#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum UnaryOperator {
    AddressOf,
    Dereference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum BinaryOperator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Equal,
    NotEqual,
    Greater,
    LogicalAnd,
}

impl UnaryOperator {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::AddressOf => "&",
            Self::Dereference => "*",
        }
    }
}

impl BinaryOperator {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::LogicalAnd => "&&",
        }
    }
}
