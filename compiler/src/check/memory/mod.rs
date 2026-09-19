use crate::ast::{BinaryOperator, LayoutShape, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::{CheckResult, Checker};

mod access;
mod intrinsic;

impl Checker {
    pub(super) fn check_address_offset(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        if operator.kind != BinaryOperator::Add {
            return Err(
                Diagnostic::error("Address only supports forward byte offset")
                    .with_primary(operator.span, "use `Address + ByteSize`")
                    .into(),
            );
        }
        let (left, right) = self.check_after(left, right, Some(&Type::ByteSize))?;
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: Type::Address,
            span,
        })
    }

    pub(super) fn check_stride_query(&self, shape: &Node<LayoutShape>, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::StorageSize(layout_shape_type(&shape.kind)),
            ty: Type::ByteSize,
            span,
        }
    }

    pub(super) fn check_memory_unary(
        &mut self,
        operator: UnaryOperator,
        operand: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let operand = self.check_expression(operand, None)?;
        let (primitive, ty) = match (operator, &operand.ty) {
            (UnaryOperator::Star, Type::Packed(element)) if **element == Type::UInt8 => {
                (MemoryPrimitive::PackedToSymbol, Type::Symbol)
            }
            (UnaryOperator::Star, Type::Symbol) => (
                MemoryPrimitive::SymbolToPacked,
                Type::Packed(Type::UInt8.into()),
            ),
            _ => {
                return Err(
                    Diagnostic::error("memory operator is not defined for this type")
                        .with_primary(operand.span, "use the predefined memory API")
                        .into(),
                );
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                operands: vec![operand],
            },
            ty,
            span,
        })
    }

    pub(super) fn check_view_slice(
        &mut self,
        operator: BinaryOperator,
        value: Expression,
        count: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = value.ty.clone();
        let (value, count) = self.check_after(value, count, Some(&Type::USize))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: if operator == BinaryOperator::Divide {
                    MemoryPrimitive::Prefix
                } else {
                    MemoryPrimitive::RemainderView
                },
                operands: vec![value, count],
            },
            ty,
            span,
        })
    }

    pub(super) fn check_packed_index(
        &mut self,
        packed: Expression,
        index: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let Type::Packed(element) = &packed.ty else {
            unreachable!()
        };
        let element = element.clone();
        let (packed, index) = self.check_after(packed, index, Some(&Type::USize))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::PackedIndex,
                operands: vec![packed, index],
            },
            ty: (*element).clone(),
            span,
        })
    }
}

fn layout_shape_type(shape: &LayoutShape) -> Type {
    match shape {
        LayoutShape::Unit => Type::Unit,
        LayoutShape::Int8 => Type::Int8,
        LayoutShape::Int16 => Type::Int16,
        LayoutShape::Int32 => Type::Int32,
        LayoutShape::Int64 => Type::Int64,
        LayoutShape::UInt8 => Type::UInt8,
        LayoutShape::UInt16 => Type::UInt16,
        LayoutShape::UInt32 => Type::UInt32,
        LayoutShape::UInt64 => Type::UInt64,
        LayoutShape::Float32 => Type::Float32,
        LayoutShape::Float64 => Type::Float64,
        LayoutShape::Address => Type::Address,
        LayoutShape::ByteSize => Type::ByteSize,
        LayoutShape::USize => Type::USize,
        LayoutShape::Bool => super::types::bool_type(),
        LayoutShape::Product(elements) => Type::Product(
            elements
                .iter()
                .map(|e| layout_shape_type(&e.kind))
                .collect(),
        ),
        LayoutShape::Sum(elements) => Type::Sum(
            elements
                .iter()
                .map(|e| layout_shape_type(&e.kind))
                .collect(),
        ),
    }
}
