#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum UnaryOperator {
    AddressOf,
    Dereference,
    PreIncrement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum BinaryOperator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    Greater,
    LogicalAnd,
}

impl UnaryOperator {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::AddressOf => "&",
            Self::Dereference => "*",
            Self::PreIncrement => "++",
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
            Self::Divide => "/",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::LogicalAnd => "&&",
        }
    }
}
