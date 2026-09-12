use super::{BinaryOperator, Identifier, NumericLiteral, StringLiteral, TypeName, UnaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Expr {
    Number(NumericLiteral),
    StringLiteral(StringLiteral),
    Identifier(Identifier),
    Call {
        callee: Box<Self>,
        arguments: Vec<Self>,
    },
    Field {
        value: Box<Self>,
        name: Identifier,
        indirect: bool,
    },
    Subscript {
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
    SizeofValue(Box<Self>),
    InitializerList(Vec<Self>),
    CompoundLiteral {
        ty: TypeName,
        fields: Vec<Initializer>,
    },
}

macro_rules! unary_constructors {
    ($($method:ident => $operator:ident),+ $(,)?) => {
        $(
            pub(in crate::backend) fn $method(operand: Self) -> Self {
                Self::unary(UnaryOperator::$operator, operand)
            }
        )+
    };
}

macro_rules! binary_constructors {
    ($($method:ident => $operator:ident),+ $(,)?) => {
        $(
            pub(in crate::backend) fn $method(left: Self, right: Self) -> Self {
                Self::binary(BinaryOperator::$operator, left, right)
            }
        )+
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct Initializer {
    designators: Vec<Designator>,
    value: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Designator {
    Field(Identifier),
}

impl Expr {
    pub(in crate::backend) fn number(value: impl Into<NumericLiteral>) -> Self {
        Self::Number(value.into())
    }

    pub(in crate::backend) fn string(value: impl Into<String>) -> Self {
        Self::StringLiteral(StringLiteral::new(value))
    }

    pub(in crate::backend) fn identifier(name: impl Into<Identifier>) -> Self {
        Self::Identifier(name.into())
    }

    pub(in crate::backend) fn call(
        callee: Self,
        arguments: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self::Call {
            callee: Box::new(callee),
            arguments: arguments.into_iter().collect(),
        }
    }

    pub(in crate::backend) fn named_call(
        name: impl Into<Identifier>,
        arguments: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self::call(Self::identifier(name), arguments)
    }

    pub(in crate::backend) fn field(self, name: impl Into<Identifier>) -> Self {
        Self::Field {
            value: Box::new(self),
            name: name.into(),
            indirect: false,
        }
    }

    pub(in crate::backend) fn pointer_field(self, name: impl Into<Identifier>) -> Self {
        Self::Field {
            value: Box::new(self),
            name: name.into(),
            indirect: true,
        }
    }

    pub(in crate::backend) fn subscript(self, index: Self) -> Self {
        Self::Subscript {
            value: Box::new(self),
            index: Box::new(index),
        }
    }

    pub(in crate::backend) fn cast(ty: impl Into<TypeName>, value: Self) -> Self {
        Self::Cast {
            ty: ty.into(),
            value: Box::new(value),
        }
    }

    pub(in crate::backend) fn unary(operator: UnaryOperator, operand: Self) -> Self {
        Self::Unary {
            operator,
            operand: Box::new(operand),
        }
    }

    pub(in crate::backend) fn binary(operator: BinaryOperator, left: Self, right: Self) -> Self {
        Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    unary_constructors! {
        address_of => AddressOf,
        dereference => Dereference,
        pre_increment => PreIncrement,
    }

    binary_constructors! {
        assign => Assign,
        add => Add,
        subtract => Subtract,
        multiply => Multiply,
        divide => Divide,
        equal => Equal,
        not_equal => NotEqual,
        less => Less,
        greater => Greater,
        logical_and => LogicalAnd,
        logical_or => LogicalOr,
    }

    pub(in crate::backend) fn conditional(condition: Self, then: Self, otherwise: Self) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }

    pub(in crate::backend) fn sizeof_value(value: Self) -> Self {
        Self::SizeofValue(Box::new(value))
    }

    pub(in crate::backend) fn initializer_list(values: impl IntoIterator<Item = Self>) -> Self {
        Self::InitializerList(values.into_iter().collect())
    }

    pub(in crate::backend) fn compound_literal(
        ty: impl Into<TypeName>,
        fields: impl IntoIterator<Item = Initializer>,
    ) -> Self {
        Self::CompoundLiteral {
            ty: ty.into(),
            fields: fields.into_iter().collect(),
        }
    }
}

impl Initializer {
    pub(in crate::backend) fn positional(value: Expr) -> Self {
        Self {
            designators: Vec::new(),
            value,
        }
    }

    pub(in crate::backend) fn designated(name: impl Into<Identifier>, value: Expr) -> Self {
        Self {
            designators: vec![Designator::Field(name.into())],
            value,
        }
    }

    pub(in crate::backend) fn designated_path(
        path: impl IntoIterator<Item = impl Into<Identifier>>,
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

    pub(super) fn render(&self, output: &mut String) {
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
