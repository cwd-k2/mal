use crate::check::ast::Type;
use crate::core::ast::BinaryPrimitive;

#[derive(Clone, Copy)]
pub(super) struct ScalarType {
    pub(super) llvm: &'static str,
    pub(super) alignment: u8,
    pub(super) bits: u8,
    pub(super) signed: bool,
}

pub(super) fn scalar_type(ty: &Type) -> Option<ScalarType> {
    let (llvm, alignment, bits, signed) = match ty {
        Type::Int8 => ("i8", 1, 8, true),
        Type::Int16 => ("i16", 2, 16, true),
        Type::Int32 => ("i32", 4, 32, true),
        Type::Int64 => ("i64", 8, 64, true),
        Type::UInt8 => ("i8", 1, 8, false),
        Type::UInt16 => ("i16", 2, 16, false),
        Type::UInt32 => ("i32", 4, 32, false),
        Type::UInt64 => ("i64", 8, 64, false),
        _ => return None,
    };
    Some(ScalarType {
        llvm,
        alignment,
        bits,
        signed,
    })
}

pub(super) fn integer_literal(ty: &Type, value: i128) -> Option<String> {
    let value = match ty {
        Type::Int8 => (value as i8).to_string(),
        Type::Int16 => (value as i16).to_string(),
        Type::Int32 => (value as i32).to_string(),
        Type::Int64 => (value as i64).to_string(),
        Type::UInt8 => (value as u8).to_string(),
        Type::UInt16 => (value as u16).to_string(),
        Type::UInt32 => (value as u32).to_string(),
        Type::UInt64 => (value as u64).to_string(),
        _ => return None,
    };
    Some(value)
}

pub(super) fn arithmetic_instruction(
    operator: BinaryPrimitive,
    signed: bool,
) -> Option<&'static str> {
    match operator {
        BinaryPrimitive::Multiply => Some("mul"),
        BinaryPrimitive::Divide if signed => Some("sdiv"),
        BinaryPrimitive::Divide => Some("udiv"),
        BinaryPrimitive::Remainder if signed => Some("srem"),
        BinaryPrimitive::Remainder => Some("urem"),
        BinaryPrimitive::Add => Some("add"),
        BinaryPrimitive::Subtract => Some("sub"),
        BinaryPrimitive::ShiftLeft => Some("shl"),
        BinaryPrimitive::ShiftRight if signed => Some("ashr"),
        BinaryPrimitive::ShiftRight => Some("lshr"),
        BinaryPrimitive::BitwiseAnd => Some("and"),
        BinaryPrimitive::BitwiseXor => Some("xor"),
        BinaryPrimitive::BitwiseOr => Some("or"),
        _ => None,
    }
}

pub(super) struct ComparisonPredicate {
    signed: &'static str,
    unsigned: &'static str,
}

impl ComparisonPredicate {
    pub(super) fn for_signedness(&self, signed: bool) -> &'static str {
        if signed { self.signed } else { self.unsigned }
    }
}

pub(super) fn comparison_predicate(operator: BinaryPrimitive) -> Option<ComparisonPredicate> {
    let (signed, unsigned) = match operator {
        BinaryPrimitive::Less => ("slt", "ult"),
        BinaryPrimitive::LessEqual => ("sle", "ule"),
        BinaryPrimitive::Greater => ("sgt", "ugt"),
        BinaryPrimitive::GreaterEqual => ("sge", "uge"),
        BinaryPrimitive::Equal => ("eq", "eq"),
        BinaryPrimitive::NotEqual => ("ne", "ne"),
        _ => return None,
    };
    Some(ComparisonPredicate { signed, unsigned })
}
