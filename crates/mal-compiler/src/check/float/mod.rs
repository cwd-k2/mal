use std::cmp::Ordering;

use crate::diagnostic::Diagnostic;
use crate::lexer::{DecimalFloatLiteral, FloatSuffix};
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, Type};
use super::types::type_name;

mod big_uint;

use self::big_uint::BigUint;

// Binary64 rounding boundaries have a denominator no larger than 2^1075, so
// their terminating decimal coefficients are shorter than this prefix.
const MAX_DECIMAL_COEFFICIENT_DIGITS: usize = 1_100;

#[derive(Clone, Copy)]
struct Format {
    precision: u32,
    fraction_bits: u32,
    exponent_bits: u32,
    bias: i32,
    minimum_exponent: i32,
    maximum_exponent: i32,
    decimal_limit: i64,
}

const BINARY32: Format = Format {
    precision: 24,
    fraction_bits: 23,
    exponent_bits: 8,
    bias: 127,
    minimum_exponent: -126,
    maximum_exponent: 127,
    decimal_limit: 60,
};

const BINARY64: Format = Format {
    precision: 53,
    fraction_bits: 52,
    exponent_bits: 11,
    bias: 1023,
    minimum_exponent: -1022,
    maximum_exponent: 1023,
    decimal_limit: 400,
};

impl Checker {
    pub(super) fn check_float(
        &self,
        literal: &DecimalFloatLiteral,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let ty = match literal.suffix {
            Some(FloatSuffix::Float32) => Type::Float32,
            Some(FloatSuffix::Float64) => Type::Float64,
            None => expected
                .filter(|ty| is_float(ty))
                .cloned()
                .unwrap_or(Type::Float64),
        };
        let format = match ty {
            Type::Float32 => BINARY32,
            Type::Float64 => BINARY64,
            _ => unreachable!("float literals have a floating-point type"),
        };
        let bits = round_decimal(literal, format).ok_or_else(|| {
            Diagnostic::error(format!("{} literal is out of range", type_name(&ty)))
                .with_primary(span, "the value exceeds the largest finite value")
        })?;
        Ok(Expression {
            kind: ExpressionKind::Float(bits),
            ty,
            span,
        })
    }
}

pub(super) fn is_float(ty: &Type) -> bool {
    matches!(ty, Type::Float32 | Type::Float64)
}

pub(super) fn is_contextual_float(expression: &crate::ast::Node<resolved::Expression>) -> bool {
    match &expression.kind {
        resolved::Expression::Float(literal) => literal.suffix.is_none(),
        resolved::Expression::Parenthesized(inner) => is_contextual_float(inner),
        _ => false,
    }
}

fn round_decimal(literal: &DecimalFloatLiteral, format: Format) -> Option<u64> {
    let significant = literal.digits.trim_start_matches('0');
    if significant.is_empty() {
        return Some(0);
    }

    let mut exponent = explicit_exponent(literal)
        .saturating_sub(i64::try_from(literal.fractional_digits).unwrap_or(i64::MAX));
    let trailing = significant.len() - significant.trim_end_matches('0').len();
    let significant = &significant[..significant.len() - trailing];
    exponent = exponent.saturating_add(i64::try_from(trailing).unwrap_or(i64::MAX));

    let decimal_order = i64::try_from(significant.len())
        .unwrap_or(i64::MAX)
        .saturating_add(exponent);
    if decimal_order > format.decimal_limit {
        return None;
    }
    if decimal_order < -format.decimal_limit {
        return Some(0);
    }

    let (digits, exponent) = bounded_coefficient(significant, exponent);

    let coefficient = BigUint::from_decimal(&digits);
    let (numerator, denominator) = if exponent >= 0 {
        let power = BigUint::power_of_ten(u32::try_from(exponent).ok()?);
        (coefficient.multiply(&power), BigUint::from_u64(1))
    } else {
        (
            coefficient,
            BigUint::power_of_ten(u32::try_from(-exponent).ok()?),
        )
    };

    let maximum_significand = (1_u64 << format.precision) - 1;
    let maximum_shift = u32::try_from(format.maximum_exponent - (format.precision as i32 - 1))
        .expect("binary formats have a positive maximum significand shift");
    let maximum = denominator
        .multiply_u64(maximum_significand)
        .shift_left(maximum_shift);
    if numerator > maximum {
        return None;
    }

    let mut exponent_two = numerator.bit_length() as i32 - denominator.bit_length() as i32;
    if compare_ratio_to_power_of_two(&numerator, &denominator, exponent_two) == Ordering::Less {
        exponent_two -= 1;
    }

    if exponent_two < format.minimum_exponent {
        let shift = u32::try_from((format.precision as i32 - 1) - format.minimum_exponent)
            .expect("subnormal scale is positive");
        let significand = rounded_quotient(&numerator.shift_left(shift), &denominator, format);
        if significand == 1_u64 << format.fraction_bits {
            return Some(1_u64 << format.fraction_bits);
        }
        return Some(significand);
    }

    let scale = (format.precision as i32 - 1) - exponent_two;
    let (scaled_numerator, scaled_denominator) = if scale >= 0 {
        (
            numerator.shift_left(u32::try_from(scale).expect("nonnegative scale")),
            denominator,
        )
    } else {
        (
            numerator,
            denominator.shift_left(u32::try_from(-scale).expect("negative scale magnitude")),
        )
    };
    let mut significand = rounded_quotient(&scaled_numerator, &scaled_denominator, format);
    if significand == 1_u64 << format.precision {
        significand >>= 1;
        exponent_two += 1;
    }
    let encoded_exponent = u64::try_from(exponent_two + format.bias)
        .expect("normal exponents encode as nonnegative values");
    debug_assert!(encoded_exponent < (1_u64 << format.exponent_bits) - 1);
    let fraction = significand - (1_u64 << format.fraction_bits);
    Some((encoded_exponent << format.fraction_bits) | fraction)
}

fn bounded_coefficient(significant: &str, exponent: i64) -> (String, i64) {
    if significant.len() <= MAX_DECIMAL_COEFFICIENT_DIGITS {
        return (significant.into(), exponent);
    }

    let omitted = significant.len() - MAX_DECIMAL_COEFFICIENT_DIGITS;
    let mut digits = significant[..MAX_DECIMAL_COEFFICIENT_DIGITS].to_owned();
    digits.push('1');
    let exponent = exponent
        .saturating_add(i64::try_from(omitted).unwrap_or(i64::MAX))
        .saturating_sub(1);
    (digits, exponent)
}

fn explicit_exponent(literal: &DecimalFloatLiteral) -> i64 {
    let magnitude = literal.exponent_digits.bytes().fold(0_i64, |value, digit| {
        value
            .saturating_mul(10)
            .saturating_add(i64::from(digit - b'0'))
    });
    if literal.exponent_negative {
        -magnitude
    } else {
        magnitude
    }
}

fn compare_ratio_to_power_of_two(
    numerator: &BigUint,
    denominator: &BigUint,
    exponent: i32,
) -> Ordering {
    if exponent >= 0 {
        numerator.cmp(&denominator.shift_left(exponent as u32))
    } else {
        numerator.shift_left((-exponent) as u32).cmp(denominator)
    }
}

fn rounded_quotient(numerator: &BigUint, denominator: &BigUint, format: Format) -> u64 {
    let mut low = 0_u64;
    let mut high = (1_u64 << format.precision) + 1;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        if denominator.multiply_u64(middle) <= *numerator {
            low = middle;
        } else {
            high = middle;
        }
    }
    let product = denominator.multiply_u64(low);
    let mut remainder = numerator.clone();
    remainder.subtract(&product);
    match remainder.multiply_u64(2).cmp(denominator) {
        Ordering::Greater => low + 1,
        Ordering::Equal if low & 1 == 1 => low + 1,
        _ => low,
    }
}
