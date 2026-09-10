#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend::c) enum UnaryOperator {
    AddressOf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend::c) enum BinaryOperator {
    Equal,
    NotEqual,
    LogicalAnd,
}

impl UnaryOperator {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::AddressOf => "&",
        }
    }
}

impl BinaryOperator {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LogicalAnd => "&&",
        }
    }
}
