use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct NumericLiteral(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct StringLiteral(String);

impl NumericLiteral {
    fn new(value: String) -> Self {
        assert!(
            is_numeric_token(&value),
            "generated C numeric literal is invalid: {value:?}"
        );
        Self(value)
    }
}

impl Deref for NumericLiteral {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for NumericLiteral {
    fn from(value: &str) -> Self {
        Self::new(value.into())
    }
}

impl From<String> for NumericLiteral {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl StringLiteral {
    pub(in crate::backend) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(super) fn render(&self, output: &mut String) {
        use std::fmt::Write as _;

        output.push('"');
        for byte in self.0.bytes() {
            match byte {
                b'"' => output.push_str("\\\""),
                b'\\' => output.push_str("\\\\"),
                b'\n' => output.push_str("\\n"),
                b'\r' => output.push_str("\\r"),
                b'\t' => output.push_str("\\t"),
                byte if byte.is_ascii_graphic() || byte == b' ' => {
                    output.push(char::from(byte));
                }
                byte => {
                    write!(output, "\\{byte:03o}").expect("writing generated C cannot fail");
                }
            }
        }
        output.push('"');
    }
}

fn is_numeric_token(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_digit()
        && bytes.iter().enumerate().all(|(index, byte)| match byte {
            b'+' | b'-' => index != 0 && matches!(bytes[index - 1], b'e' | b'E' | b'p' | b'P'),
            b'.' => true,
            byte => byte.is_ascii_alphanumeric(),
        })
}

#[cfg(test)]
mod tests {
    use super::{StringLiteral, is_numeric_token};

    #[test]
    fn admits_one_numeric_preprocessing_token() {
        for literal in ["0", "0x000500u", "1.0f", "0x1p-31", "42ULL"] {
            assert!(is_numeric_token(literal), "{literal}");
        }
        for fragment in ["", "value", "1 + 2", "1; abort()", "-1", "1+2"] {
            assert!(!is_numeric_token(fragment), "{fragment}");
        }
    }

    #[test]
    fn escapes_utf8_and_control_bytes_without_hex_escape_capture() {
        let mut output = String::new();
        StringLiteral::new("\u{1}aあ").render(&mut output);

        assert_eq!(output, "\"\\001a\\343\\201\\202\"");
    }
}
