mod render;
#[cfg(test)]
mod tests;

use super::{BinaryOperator, Identifier, NumericLiteral, StringLiteral, TypeName, UnaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Expr {
    Number(NumericLiteral),
    StringLiteral(StringLiteral),
    Identifier(Identifier),
    Parenthesized(Box<Self>),
    Call {
        callee: Box<Self>,
        arguments: Vec<Self>,
    },
    Field {
        value: Box<Self>,
        name: Identifier,
        indirect: bool,
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

/// Converts a Rust-side interpolation into a generated C expression.
#[allow(dead_code)]
pub(in crate::backend) trait IntoExpr {
    /// Treats strings as validated C identifiers and preserves existing expressions.
    fn into_expr(self) -> Expr;
}

impl IntoExpr for Expr {
    fn into_expr(self) -> Expr {
        self
    }
}

impl IntoExpr for String {
    fn into_expr(self) -> Expr {
        Expr::identifier(self)
    }
}

impl IntoExpr for &str {
    fn into_expr(self) -> Expr {
        Expr::identifier(self)
    }
}

macro_rules! numeric_expressions {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl IntoExpr for $ty {
                fn into_expr(self) -> Expr {
                    Expr::number(self.to_string())
                }
            }
        )+
    };
}

numeric_expressions!(u8, u16, u32, u64, usize);

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
pub(in crate::backend) enum Initializer {
    Value {
        designators: Vec<Designator>,
        value: Expr,
    },
    MacroInvocation(super::MacroInvocation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Designator {
    Field(Identifier),
}

impl Expr {
    pub(in crate::backend) fn parenthesized(value: Self) -> Self {
        Self::Parenthesized(Box::new(value))
    }

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

    #[cfg(test)]
    pub(in crate::backend) fn named_call(
        name: impl Into<Identifier>,
        arguments: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self::call(Self::identifier(name), arguments)
    }

    pub(in crate::backend) fn field(self, name: impl Into<Identifier>) -> Self {
        if let Self::Unary {
            operator: UnaryOperator::Dereference,
            operand,
        } = self
        {
            return Self::Field {
                value: operand,
                name: name.into(),
                indirect: true,
            };
        }
        Self::Field {
            value: Box::new(self),
            name: name.into(),
            indirect: false,
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
    }

    binary_constructors! {
        assign => Assign,
        add => Add,
        subtract => Subtract,
        multiply => Multiply,
        equal => Equal,
        not_equal => NotEqual,
        greater => Greater,
        logical_and => LogicalAnd,
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
    #[cfg(test)]
    pub(in crate::backend) fn positional(value: Expr) -> Self {
        Self::Value {
            designators: Vec::new(),
            value,
        }
    }

    pub(in crate::backend) fn designated(name: impl Into<Identifier>, value: Expr) -> Self {
        Self::Value {
            designators: vec![Designator::Field(name.into())],
            value,
        }
    }

    pub(in crate::backend) fn designated_path(
        path: impl IntoIterator<Item = impl Into<Identifier>>,
        value: Expr,
    ) -> Self {
        Self::Value {
            designators: path
                .into_iter()
                .map(|name| Designator::Field(name.into()))
                .collect(),
            value,
        }
    }

    pub(in crate::backend::c::syntax) fn render(
        &self,
        output: &mut impl super::render::RenderWrite,
    ) {
        let Self::Value { designators, value } = self else {
            let Self::MacroInvocation(invocation) = self else {
                unreachable!()
            };
            invocation.render_into(output);
            return;
        };
        for designator in designators {
            match designator {
                Designator::Field(name) => {
                    output.push('.');
                    output.push_str(name);
                }
            }
        }
        if !designators.is_empty() {
            output.push_str(" = ");
        }
        value.render(output);
    }
}

impl From<super::MacroInvocation> for Initializer {
    fn from(value: super::MacroInvocation) -> Self {
        Self::MacroInvocation(value)
    }
}
