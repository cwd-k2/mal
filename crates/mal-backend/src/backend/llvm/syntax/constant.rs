use super::function::is_single_line;
use super::{BinaryOperator, CastOperator, Type, UnaryOperator};

#[derive(Clone)]
pub(in crate::backend::llvm) enum Constant {
    Atom(String),
    ZeroInitializer,
    Structure(Vec<TypedConstant>),
    GetElementPtr {
        element_type: Type,
        pointer: Box<TypedConstant>,
        indices: Vec<TypedConstant>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<TypedConstant>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<TypedConstant>,
        right: Box<TypedConstant>,
    },
    Cast {
        operator: CastOperator,
        operand: Box<TypedConstant>,
        target: Type,
    },
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct TypedConstant {
    ty: Type,
    value: Constant,
}

impl Constant {
    pub(in crate::backend::llvm) fn atom(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (is_single_line(&value) && !value.chars().any(char::is_whitespace))
            .then_some(Self::Atom(value))
    }

    pub(in crate::backend::llvm) fn structure(
        fields: impl IntoIterator<Item = TypedConstant>,
    ) -> Self {
        Self::Structure(fields.into_iter().collect())
    }

    pub(in crate::backend::llvm) fn get_element_ptr(
        element_type: Type,
        pointer: TypedConstant,
        indices: impl IntoIterator<Item = TypedConstant>,
    ) -> Self {
        Self::GetElementPtr {
            element_type,
            pointer: Box::new(pointer),
            indices: indices.into_iter().collect(),
        }
    }

    pub(in crate::backend::llvm) fn unary(operator: UnaryOperator, operand: TypedConstant) -> Self {
        Self::Unary {
            operator,
            operand: Box::new(operand),
        }
    }

    pub(in crate::backend::llvm) fn binary(
        operator: BinaryOperator,
        left: TypedConstant,
        right: TypedConstant,
    ) -> Option<Self> {
        (left.ty == right.ty).then_some(Self::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    pub(in crate::backend::llvm) fn cast(
        operator: CastOperator,
        operand: TypedConstant,
        target: Type,
    ) -> Self {
        Self::Cast {
            operator,
            operand: Box::new(operand),
            target,
        }
    }

    pub(in crate::backend::llvm) fn integer_value(&self) -> Option<i128> {
        let Self::Atom(value) = self else {
            return None;
        };
        value.parse().ok()
    }

    pub(in crate::backend::llvm) fn render(&self) -> String {
        match self {
            Self::Atom(value) => value.clone(),
            Self::ZeroInitializer => "zeroinitializer".into(),
            Self::Structure(fields) => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(TypedConstant::render)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::GetElementPtr {
                element_type,
                pointer,
                indices,
            } => {
                format!(
                    "getelementptr ({element_type}, {}{})",
                    pointer.render(),
                    indices
                        .iter()
                        .map(|index| format!(", {}", index.render()))
                        .collect::<String>()
                )
            }
            Self::Unary { operator, operand } => {
                format!("{} ({})", operator.mnemonic(), operand.render())
            }
            Self::Binary {
                operator,
                left,
                right,
            } => format!("{operator} ({}, {})", left.render(), right.render()),
            Self::Cast {
                operator,
                operand,
                target,
            } => format!("{} ({} to {target})", operator.mnemonic(), operand.render()),
        }
    }
}

impl TypedConstant {
    pub(in crate::backend::llvm) fn new(ty: Type, value: Constant) -> Self {
        Self { ty, value }
    }

    fn render(&self) -> String {
        format!("{} {}", self.ty, self.value.render())
    }
}

#[cfg(test)]
mod tests {
    use super::{Constant, TypedConstant};
    use crate::backend::llvm::syntax::{BinaryOperator, CastOperator, Type};

    #[test]
    fn rejects_text_fragments_as_atoms() {
        assert!(Constant::atom("add i32 1, 2").is_none());
        assert!(Constant::atom("1\n2").is_none());
    }

    #[test]
    fn renders_nested_constant_expressions() {
        let sum = Constant::binary(
            BinaryOperator::Add,
            TypedConstant::new(Type::integer(32_u16), Constant::atom("1").unwrap()),
            TypedConstant::new(Type::integer(32_u16), Constant::atom("2").unwrap()),
        )
        .unwrap();
        let value = Constant::cast(
            CastOperator::ZExt,
            TypedConstant::new(Type::integer(32_u16), sum),
            Type::integer(64_u16),
        );

        assert_eq!(value.render(), "zext (i32 add (i32 1, i32 2) to i64)");
    }

    #[test]
    fn renders_structured_symbol_view() {
        let address = TypedConstant::new(Type::Pointer, Constant::atom("@symbol").unwrap());
        let data = Constant::get_element_ptr(
            Type::integer(8_u16),
            address.clone(),
            [TypedConstant::new(
                Type::integer(64_u16),
                Constant::atom("16").unwrap(),
            )],
        );

        assert_eq!(
            Constant::structure([
                address,
                TypedConstant::new(Type::Pointer, data),
                TypedConstant::new(Type::integer(64_u16), Constant::atom("3").unwrap()),
            ])
            .render(),
            "{ ptr @symbol, ptr getelementptr (i8, ptr @symbol, i64 16), i64 3 }"
        );
    }
}
