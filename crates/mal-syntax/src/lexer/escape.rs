/// Decodes the escape whose backslash is at `offset`. Byte and Symbol literals share every escape except the one for
/// their own quote. Returns the byte and the offset after the escape, or the offset of the byte that fails and why.
pub(super) fn decode(
    bytes: &[u8],
    offset: usize,
    quote: u8,
) -> Result<(u8, usize), (usize, &'static str)> {
    let offset = offset + 1;
    let value = match bytes.get(offset).copied() {
        Some(b'\\') => b'\\',
        Some(byte) if byte == quote => quote,
        Some(b'n') => b'\n',
        Some(b'r') => b'\r',
        Some(b't') => b'\t',
        Some(b'0') => b'\0',
        Some(b'x') => {
            let digit = |offset: usize| {
                bytes
                    .get(offset)
                    .copied()
                    .and_then(hex_value)
                    .ok_or((offset, "expected two hexadecimal digits"))
            };
            let high = digit(offset + 1)?;
            let low = digit(offset + 2)?;
            return Ok((high * 16 + low, offset + 3));
        }
        _ => return Err((offset, "unknown escape")),
    };
    Ok((value, offset + 1))
}

/// The end of a malformed literal: the byte after the next `quote`, or the end of the line, starting at `offset`. The
/// line terminator stays outside, so the span never ends between the two bytes of a CRLF.
pub(super) fn recover(bytes: &[u8], mut offset: usize, quote: u8) -> usize {
    while let Some(&byte) = bytes.get(offset) {
        if matches!(byte, b'\n' | b'\r') {
            break;
        }
        offset += 1;
        if byte == quote {
            break;
        }
    }
    offset
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
