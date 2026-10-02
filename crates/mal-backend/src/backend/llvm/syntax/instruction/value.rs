//! Callees, typed operands, and the lexical checks that admit a value or an atom in an operand position.

use super::*;

#[derive(Clone)]
pub(in crate::backend) enum Callee {
    Direct(String),
    Indirect(String),
}

#[derive(Clone)]
pub(in crate::backend) struct TypedValue {
    pub(super) ty: Type,
    pub(super) value: String,
}

impl Callee {
    pub(in crate::backend) fn direct(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        is_valid_name(&name).then_some(Self::Direct(name))
    }

    pub(in crate::backend) fn indirect(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_atom(&value).then_some(Self::Indirect(value))
    }

    pub(super) fn render_into(&self, output: &mut String) {
        match self {
            Self::Direct(name) => output.push_str(&format!("@{name}")),
            Self::Indirect(value) => output.push_str(value),
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        matches!(self, Self::Direct(name) if name.starts_with("mal_runtime_bytes_")
            || name.starts_with("mal_runtime_buffer_")
            || name.starts_with("mal_runtime_symbol_"))
    }
}

impl TypedValue {
    pub(in crate::backend) fn new(ty: Type, value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self { ty, value })
    }

    pub(in crate::backend) fn from_pairs<Value: Into<String>>(
        values: impl IntoIterator<Item = (Type, Value)>,
    ) -> Option<Vec<Self>> {
        values
            .into_iter()
            .map(|(ty, value)| Self::new(ty, value))
            .collect()
    }
}

pub(super) fn local_name(value: &str) -> Option<&str> {
    value.strip_prefix('%')
}
