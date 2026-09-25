use super::escape;

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
                let (escaped, next) =
                    escape::decode(bytes, offset, b'"').map_err(|(offset, label)| DecodeError {
                        end: escape::recover(bytes, offset, b'"'),
                        label,
                    })?;
                value.push(escaped);
                offset = next;
            }
            Some(b'\r' | b'\n') | None => {
                return Err(DecodeError {
                    end: escape::recover(bytes, offset, b'"'),
                    label: "expected a closing double quote",
                });
            }
            Some(byte) => {
                value.push(byte);
                offset += 1;
            }
        }
    }
}
