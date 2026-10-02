//! Operator-family admission and result typing over already resolved operands.

use crate::resolve::ast as resolved;
use mal_syntax::ast::{BinaryOperator, Node, UnaryOperator};
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{
    BinaryOperation, BinaryPrimitive, Expression, ExpressionKind, MemoryPrimitive, ShortCircuit,
    SymbolPrimitive, Type, UnaryOperation, UnaryPrimitive,
};
use super::float::{is_contextual_float, is_float};
use super::integer::{
    integer_is_signed, integer_negative_magnitude, is_contextual_integer, is_integer, literal_type,
    parse_magnitude, unparenthesized_integer,
};
use super::types::{bool_type, type_name};
use super::{CheckFailure, CheckResult, Checker};

mod arithmetic;
mod binary;
mod chain;
mod logical;
mod unary;

use arithmetic::{arithmetic_result, is_target_quantity, unsupported_binary};

/// Operators see an operand of a file-local opaque type as its representation in the declaring file, like every
/// other comparison there. The operand keeps its checked value; only the type the operator rule reads changes.
pub(super) fn view_operand(mut operand: Expression) -> Expression {
    let viewed = super::types::representation_view(&operand.ty, operand.span.file());
    if !std::ptr::eq(viewed, &operand.ty) {
        operand.ty = viewed.clone();
    }
    operand
}

/// Builds a checked binary operation. Symbol and Bool operands select the operation the operator denotes, so later
/// stages never re-derive it from operand types.
pub(super) fn binary_expression(
    operator: &Node<BinaryOperator>,
    left: Expression,
    right: Expression,
    ty: Type,
    span: Span,
) -> Expression {
    if left.ty == bool_type()
        && matches!(
            operator.kind,
            BinaryOperator::Equal | BinaryOperator::NotEqual
        )
    {
        return Expression {
            kind: ExpressionKind::BoolEquality {
                equal: operator.kind == BinaryOperator::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            span,
        };
    }
    if left.ty == Type::Symbol {
        let primitive = match operator.kind {
            BinaryOperator::Add => Some(SymbolPrimitive::Concatenate),
            BinaryOperator::Divide => Some(SymbolPrimitive::Prefix),
            BinaryOperator::Remainder => Some(SymbolPrimitive::Suffix),
            BinaryOperator::Equal => Some(SymbolPrimitive::Equal),
            BinaryOperator::NotEqual => Some(SymbolPrimitive::NotEqual),
            _ => None,
        };
        if let Some(primitive) = primitive {
            return symbol(primitive, vec![left, right], ty, span);
        }
    }
    Expression {
        kind: ExpressionKind::Binary {
            operator: Node::new(binary_operation(operator.kind), operator.span),
            left: Box::new(left),
            right: Box::new(right),
        },
        ty,
        span,
    }
}

/// The checked operation of a binary operator whose Symbol and Bool equality forms were already separated.
pub(super) fn binary_operation(operator: BinaryOperator) -> BinaryOperation {
    use BinaryPrimitive as Primitive;
    BinaryOperation::Primitive(match operator {
        BinaryOperator::Multiply => Primitive::Multiply,
        BinaryOperator::Divide => Primitive::Divide,
        BinaryOperator::Remainder => Primitive::Remainder,
        BinaryOperator::Add => Primitive::Add,
        BinaryOperator::Subtract => Primitive::Subtract,
        BinaryOperator::ShiftLeft => Primitive::ShiftLeft,
        BinaryOperator::ShiftRight => Primitive::ShiftRight,
        BinaryOperator::Less => Primitive::Less,
        BinaryOperator::LessEqual => Primitive::LessEqual,
        BinaryOperator::Greater => Primitive::Greater,
        BinaryOperator::GreaterEqual => Primitive::GreaterEqual,
        BinaryOperator::Equal => Primitive::Equal,
        BinaryOperator::NotEqual => Primitive::NotEqual,
        BinaryOperator::BitwiseAnd => Primitive::BitwiseAnd,
        BinaryOperator::BitwiseXor => Primitive::BitwiseXor,
        BinaryOperator::BitwiseOr => Primitive::BitwiseOr,
        BinaryOperator::LogicalAnd => return BinaryOperation::ShortCircuit(ShortCircuit::And),
        BinaryOperator::LogicalOr => return BinaryOperation::ShortCircuit(ShortCircuit::Or),
        BinaryOperator::SymbolAt => unreachable!("indexed access is checked as a Symbol operation"),
    })
}

pub(super) fn symbol(
    primitive: SymbolPrimitive,
    operands: Vec<Expression>,
    ty: Type,
    span: Span,
) -> Expression {
    Expression {
        kind: ExpressionKind::SymbolOperation {
            primitive,
            operands,
        },
        ty,
        span,
    }
}

/// The expected type of an operator result, seen through the declaring file's representation view.
pub(super) fn view_expected(expected: Option<&Type>, span: Span) -> Option<&Type> {
    expected.map(|ty| super::types::representation_view(ty, span.file()))
}
