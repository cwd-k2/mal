pub(super) struct Decoded {
    pub(super) value: Vec<u8>,
    pub(super) end: usize,
}

pub(super) struct DecodeError {
    pub(super) end: usize,
    pub(super) label: &'static str,
}

pub(super) fn decode(bytes: &[u8], start: usize) -> Result<Decoded, DecodeError> {
    let mut offset = start + 1;
    let mut value = Vec::new();
    loop {
        match bytes.get(offset).copied() {
            Some(b'"') => {
                return Ok(Decoded {
                    value,
                    end: offset + 1,
                });
            }
            Some(b'\\') => {
                offset += 1;
                let escaped = match bytes.get(offset).copied() {
                    Some(b'\\') => b'\\',
                    Some(b'"') => b'"',
                    Some(b'n') => b'\n',
                    Some(b'r') => b'\r',
                    Some(b't') => b'\t',
                    Some(b'0') => b'\0',
                    Some(b'x') => {
                        offset += 1;
                        let Some(high) = bytes.get(offset).copied().and_then(hex_value) else {
                            return Err(error(bytes, offset, "expected two hexadecimal digits"));
                        };
                        offset += 1;
                        let Some(low) = bytes.get(offset).copied().and_then(hex_value) else {
                            return Err(error(bytes, offset, "expected two hexadecimal digits"));
                        };
                        high * 16 + low
                    }
                    _ => return Err(error(bytes, offset, "unknown Symbol escape")),
                };
                value.push(escaped);
                offset += 1;
            }
            Some(b'\r' | b'\n') | None => {
                return Err(error(bytes, offset, "expected a closing double quote"));
            }
            Some(byte) => {
                value.push(byte);
                offset += 1;
            }
        }
    }
}

fn error(bytes: &[u8], mut offset: usize, label: &'static str) -> DecodeError {
    while let Some(byte) = bytes.get(offset) {
        offset += 1;
        if *byte == b'"' || matches!(*byte, b'\n' | b'\r') {
            break;
        }
    }
    DecodeError { end: offset, label }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
