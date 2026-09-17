use crate::ast::BinaryOperator;

use super::ast::BinaryPrimitive;

pub(super) fn lower_binary_primitive(operator: BinaryOperator) -> BinaryPrimitive {
    match operator {
        BinaryOperator::SymbolAt => {
            unreachable!("Symbol access is lowered before generic binary primitives")
        }
        BinaryOperator::Store => unreachable!("memory store has a dedicated node"),
        BinaryOperator::Multiply => BinaryPrimitive::Multiply,
        BinaryOperator::Divide => BinaryPrimitive::Divide,
        BinaryOperator::Remainder => BinaryPrimitive::Remainder,
        BinaryOperator::Add => BinaryPrimitive::Add,
        BinaryOperator::Subtract => BinaryPrimitive::Subtract,
        BinaryOperator::ShiftLeft => BinaryPrimitive::ShiftLeft,
        BinaryOperator::ShiftRight => BinaryPrimitive::ShiftRight,
        BinaryOperator::Less => BinaryPrimitive::Less,
        BinaryOperator::LessEqual => BinaryPrimitive::LessEqual,
        BinaryOperator::Greater => BinaryPrimitive::Greater,
        BinaryOperator::GreaterEqual => BinaryPrimitive::GreaterEqual,
        BinaryOperator::Equal => BinaryPrimitive::Equal,
        BinaryOperator::NotEqual => BinaryPrimitive::NotEqual,
        BinaryOperator::BitwiseAnd => BinaryPrimitive::BitwiseAnd,
        BinaryOperator::BitwiseXor => BinaryPrimitive::BitwiseXor,
        BinaryOperator::BitwiseOr => BinaryPrimitive::BitwiseOr,
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
            unreachable!("logical operators are lowered separately")
        }
    }
}
