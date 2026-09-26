use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::llvm) enum Type {
    Void,
    Integer(u16),
    Float,
    Double,
    Pointer,
    Array { length: usize, element: Box<Self> },
    Structure(Vec<Self>),
}

impl Type {
    pub(in crate::backend::llvm) fn integer(bits: impl Into<u16>) -> Self {
        Self::Integer(bits.into())
    }

    pub(in crate::backend::llvm) fn array(length: usize, element: Self) -> Self {
        Self::Array {
            length,
            element: Box::new(element),
        }
    }

    pub(in crate::backend::llvm) fn structure(fields: impl IntoIterator<Item = Self>) -> Self {
        Self::Structure(fields.into_iter().collect())
    }
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => formatter.write_str("void"),
            Self::Integer(bits) => write!(formatter, "i{bits}"),
            Self::Float => formatter.write_str("float"),
            Self::Double => formatter.write_str("double"),
            Self::Pointer => formatter.write_str("ptr"),
            Self::Array { length, element } => write!(formatter, "[{length} x {element}]"),
            Self::Structure(fields) => {
                formatter.write_str("{ ")?;
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    field.fmt(formatter)?;
                }
                formatter.write_str(" }")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    fn renders_nested_types_without_text_fragments() {
        assert_eq!(
            Type::structure([
                Type::integer(32_u16),
                Type::array(8, Type::integer(8_u16)),
                Type::Pointer,
            ])
            .to_string(),
            "{ i32, [8 x i8], ptr }"
        );
    }
}
