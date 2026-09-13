use crate::diagnostic::Diagnostic;

use super::{
    DecimalFloatLiteral, FloatSuffix, IntegerLiteral, IntegerSuffix, Lexer, Radix, TokenKind,
};

impl Lexer<'_> {
    pub(super) fn lex_number(&mut self, start: usize) -> Result<(), Diagnostic> {
        let radix = if self.starts_with(b"0x") {
            self.offset += 2;
            Radix::Hexadecimal
        } else if self.starts_with(b"0b") {
            self.offset += 2;
            Radix::Binary
        } else {
            Radix::Decimal
        };
        let digits_start = self.offset;
        self.lex_digit_sequence(start, radix)?;
        let digits_end = self.offset;

        if radix != Radix::Decimal {
            return self.finish_integer(start, radix, digits_start, digits_end);
        }

        let mut fractional_digits = 0;
        let mut is_float = false;
        let mut digits = self.source.text()[digits_start..digits_end].replace('_', "");
        let dot_starts_receiver_suffix = self.peek() == Some(b'.')
            && (self
                .peek_next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
                || (self.peek_next() == Some(b'_')
                    && self
                        .bytes
                        .get(self.offset + 2)
                        .is_some_and(|byte| byte.is_ascii_lowercase())));
        if self.peek() == Some(b'.') && !dot_starts_receiver_suffix {
            is_float = true;
            self.offset += 1;
            let fraction_start = self.offset;
            if self.peek() == Some(b'_') {
                self.consume_number_like();
                return Err(self.invalid_separator(start));
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected digits after the decimal point",
                ));
            }
            fractional_digits = self.lex_digit_sequence(start, Radix::Decimal)?;
            digits.push_str(&self.source.text()[fraction_start..self.offset].replace('_', ""));
        }

        let mut exponent_negative = false;
        let mut exponent_digits = String::new();
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                exponent_negative = self.peek() == Some(b'-');
                self.offset += 1;
            }
            let exponent_start = self.offset;
            if self.peek() == Some(b'_') {
                self.consume_number_like();
                return Err(self.invalid_separator(start));
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected decimal exponent digits",
                ));
            }
            self.lex_digit_sequence(start, Radix::Decimal)?;
            exponent_digits = self.source.text()[exponent_start..self.offset].replace('_', "");
        }

        let suffixes = [("f32", FloatSuffix::Float32), ("f64", FloatSuffix::Float64)];
        let suffix = suffixes
            .into_iter()
            .find(|(text, _)| self.starts_with(text.as_bytes()))
            .map(|(text, suffix)| {
                self.offset += text.len();
                suffix
            });
        is_float |= suffix.is_some();

        if is_float {
            if self
                .peek()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected `f32`, `f64`, or the end of the literal",
                ));
            }
            self.push(
                TokenKind::Float(DecimalFloatLiteral {
                    digits,
                    fractional_digits,
                    exponent_negative,
                    exponent_digits,
                    suffix,
                }),
                start,
            );
            return Ok(());
        }

        self.finish_integer(start, radix, digits_start, digits_end)
    }

    fn lex_digit_sequence(&mut self, start: usize, radix: Radix) -> Result<usize, Diagnostic> {
        let mut previous_was_digit = false;
        let mut digit_count = 0;

        while let Some(byte) = self.peek() {
            if is_digit_for_radix(byte, radix) {
                previous_was_digit = true;
                digit_count += 1;
                self.offset += 1;
            } else if byte == b'_' {
                if !previous_was_digit
                    || !self
                        .peek_next()
                        .is_some_and(|next| is_digit_for_radix(next, radix))
                {
                    self.consume_number_like();
                    return Err(self.invalid_separator(start));
                }
                previous_was_digit = false;
                self.offset += 1;
            } else {
                break;
            }
        }

        if digit_count == 0 {
            self.consume_number_like();
            return Err(self.error(
                start,
                self.offset,
                "invalid integer literal",
                "expected a digit after the radix prefix",
            ));
        }

        Ok(digit_count)
    }

    fn finish_integer(
        &mut self,
        start: usize,
        radix: Radix,
        digits_start: usize,
        digits_end: usize,
    ) -> Result<(), Diagnostic> {
        let suffixes = [
            ("i8", IntegerSuffix::Int8),
            ("i16", IntegerSuffix::Int16),
            ("i32", IntegerSuffix::Int32),
            ("i64", IntegerSuffix::Int64),
            ("u8", IntegerSuffix::UInt8),
            ("u16", IntegerSuffix::UInt16),
            ("u32", IntegerSuffix::UInt32),
            ("u64", IntegerSuffix::UInt64),
        ];
        let suffix = suffixes
            .into_iter()
            .find(|(text, _)| self.starts_with(text.as_bytes()))
            .map(|(text, suffix)| {
                self.offset += text.len();
                suffix
            });

        if self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.consume_number_like();
            return Err(self.error(
                start,
                self.offset,
                "invalid integer literal",
                "expected a fixed-width integer suffix",
            ));
        }

        let digits = self.source.text()[digits_start..digits_end].replace('_', "");
        self.push(
            TokenKind::Integer(IntegerLiteral {
                radix,
                digits,
                suffix,
            }),
            start,
        );
        Ok(())
    }

    fn invalid_separator(&self, start: usize) -> Diagnostic {
        self.error(
            start,
            self.offset,
            "invalid numeric separator",
            "`_` must occur once between two digits",
        )
    }

    fn consume_number_like(&mut self) {
        self.consume_identifier_like();
    }
}

fn is_digit_for_radix(byte: u8, radix: Radix) -> bool {
    match radix {
        Radix::Binary => matches!(byte, b'0' | b'1'),
        Radix::Decimal => byte.is_ascii_digit(),
        Radix::Hexadecimal => byte.is_ascii_hexdigit(),
    }
}
