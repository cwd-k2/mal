use std::fmt;
use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::c) struct Identifier(String);

impl Identifier {
    fn new(value: String) -> Self {
        assert!(
            is_c_identifier(&value),
            "generated C identifier is invalid: {value:?}"
        );
        Self(value)
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::new(value.into())
    }
}

impl From<&String> for Identifier {
    fn from(value: &String) -> Self {
        Self::new(value.clone())
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

fn is_c_identifier(value: &str) -> bool {
    let mut characters = value.bytes();
    matches!(characters.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && characters.all(|character| character.is_ascii_alphanumeric() || character == b'_')
}

#[cfg(test)]
mod tests {
    use super::is_c_identifier;

    #[test]
    fn admits_only_c_identifier_tokens() {
        assert!(is_c_identifier("mal_value_12"));
        assert!(!is_c_identifier("payload.variant"));
        assert!(!is_c_identifier("1value"));
        assert!(!is_c_identifier("value + other"));
    }
}
