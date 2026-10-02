//! Lexical admission shared by the syntax model: names and single-line fragments.

pub(super) fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'.' | b'$'))
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$' | b'-'))
}

pub(super) fn is_single_line(text: &str) -> bool {
    !text.is_empty() && !text.contains(['\n', '\r'])
}

/// An operand fragment: one line without a statement terminator or NUL.
pub(super) fn is_value(value: &str) -> bool {
    is_single_line(value) && !value.contains([';', '\0'])
}

/// An operand that cannot be read as several tokens: a value without whitespace or delimiters.
pub(super) fn is_atom(value: &str) -> bool {
    is_value(value)
        && !value
            .chars()
            .any(|character| character.is_whitespace() || ",(){}[]=".contains(character))
}
