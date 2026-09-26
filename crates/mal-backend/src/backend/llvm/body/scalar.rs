use crate::core::ast::BinaryPrimitive;
use mal_frontend::check::ast::Type;

#[derive(Clone, Copy)]
pub(super) struct ScalarType {
    pub(super) bits: u8,
    pub(super) signed: bool,
    pub(super) floating: bool,
}

impl ScalarType {
    pub(super) fn llvm_type(self) -> crate::backend::llvm::syntax::Type {
        if self.floating {
            if self.bits == 32 {
                crate::backend::llvm::syntax::Type::Float
            } else {
                crate::backend::llvm::syntax::Type::Double
            }
        } else {
            crate::backend::llvm::syntax::Type::integer(u16::from(self.bits))
        }
    }
}

pub(super) fn scalar_type(ty: &Type, pointer_size: usize) -> Option<ScalarType> {
    let (bits, signed, floating) = match ty {
        Type::Int8 => (8, true, false),
        Type::Int16 => (16, true, false),
        Type::Int32 => (32, true, false),
        Type::Int64 => (64, true, false),
        Type::UInt8 => (8, false, false),
        Type::UInt16 => (16, false, false),
        Type::UInt32 => (32, false, false),
        Type::UInt64 => (64, false, false),
        Type::ByteSize | Type::USize => match pointer_size {
            1 => (8, false, false),
            2 => (16, false, false),
            4 => (32, false, false),
            8 => (64, false, false),
            _ => return None,
        },
        Type::Float32 => (32, true, true),
        Type::Float64 => (64, true, true),
        _ => return None,
    };
    Some(ScalarType {
        bits,
        signed,
        floating,
    })
}

pub(super) fn integer_literal(ty: &Type, value: i128, pointer_size: usize) -> Option<String> {
    let value = match ty {
        Type::Int8 => (value as i8).to_string(),
        Type::Int16 => (value as i16).to_string(),
        Type::Int32 => (value as i32).to_string(),
        Type::Int64 => (value as i64).to_string(),
        Type::UInt8 => (value as u8).to_string(),
        Type::UInt16 => (value as u16).to_string(),
        Type::UInt32 => (value as u32).to_string(),
        Type::UInt64 => (value as u64).to_string(),
        Type::ByteSize | Type::USize => {
            let bits = pointer_size.checked_mul(8)?;
            let value = u128::try_from(value).ok()?;
            if bits < 128 && value >= (1_u128 << bits) {
                return None;
            }
            value.to_string()
        }
        _ => return None,
    };
    Some(value)
}

pub(super) fn arithmetic_instruction(
    operator: BinaryPrimitive,
    scalar: ScalarType,
) -> Option<crate::backend::llvm::syntax::BinaryOperator> {
    use crate::backend::llvm::syntax::BinaryOperator as Operator;

    if scalar.floating {
        return match operator {
            BinaryPrimitive::Multiply => Some(Operator::FMul),
            BinaryPrimitive::Divide => Some(Operator::FDiv),
            BinaryPrimitive::Add => Some(Operator::FAdd),
            BinaryPrimitive::Subtract => Some(Operator::FSub),
            _ => None,
        };
    }
    match operator {
        BinaryPrimitive::Multiply => Some(Operator::Mul),
        BinaryPrimitive::Divide if scalar.signed => Some(Operator::SDiv),
        BinaryPrimitive::Divide => Some(Operator::UDiv),
        BinaryPrimitive::Remainder if scalar.signed => Some(Operator::SRem),
        BinaryPrimitive::Remainder => Some(Operator::URem),
        BinaryPrimitive::Add => Some(Operator::Add),
        BinaryPrimitive::Subtract => Some(Operator::Sub),
        BinaryPrimitive::ShiftLeft => Some(Operator::Shl),
        BinaryPrimitive::ShiftRight if scalar.signed => Some(Operator::AShr),
        BinaryPrimitive::ShiftRight => Some(Operator::LShr),
        BinaryPrimitive::BitwiseAnd => Some(Operator::And),
        BinaryPrimitive::BitwiseXor => Some(Operator::Xor),
        BinaryPrimitive::BitwiseOr => Some(Operator::Or),
        _ => None,
    }
}

pub(super) struct ComparisonPredicate {
    signed: crate::backend::llvm::syntax::ComparisonPredicate,
    unsigned: crate::backend::llvm::syntax::ComparisonPredicate,
    floating: crate::backend::llvm::syntax::ComparisonPredicate,
}

impl ComparisonPredicate {
    pub(super) fn for_scalar(
        &self,
        scalar: ScalarType,
    ) -> (
        crate::backend::llvm::syntax::ComparisonKind,
        crate::backend::llvm::syntax::ComparisonPredicate,
    ) {
        use crate::backend::llvm::syntax::ComparisonKind;

        if scalar.floating {
            (ComparisonKind::Floating, self.floating)
        } else if scalar.signed {
            (ComparisonKind::Integer, self.signed)
        } else {
            (ComparisonKind::Integer, self.unsigned)
        }
    }
}

pub(super) fn comparison_predicate(operator: BinaryPrimitive) -> Option<ComparisonPredicate> {
    use crate::backend::llvm::syntax::ComparisonPredicate as Predicate;

    let (signed, unsigned, floating) = match operator {
        BinaryPrimitive::Less => (Predicate::Slt, Predicate::Ult, Predicate::Olt),
        BinaryPrimitive::LessEqual => (Predicate::Sle, Predicate::Ule, Predicate::Ole),
        BinaryPrimitive::Greater => (Predicate::Sgt, Predicate::Ugt, Predicate::Ogt),
        BinaryPrimitive::GreaterEqual => (Predicate::Sge, Predicate::Uge, Predicate::Oge),
        BinaryPrimitive::Equal => (Predicate::Eq, Predicate::Eq, Predicate::Oeq),
        BinaryPrimitive::NotEqual => (Predicate::Ne, Predicate::Ne, Predicate::Une),
        _ => return None,
    };
    Some(ComparisonPredicate {
        signed,
        unsigned,
        floating,
    })
}
