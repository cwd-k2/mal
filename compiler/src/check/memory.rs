use crate::ast::{BinaryOperator, LayoutShape, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved};
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::types::type_name;
use super::{CheckResult, Checker};

impl Checker {
    pub(super) fn check_address_offset(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
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

    pub(super) fn check_placement(
        &mut self,
        value: &Node<resolved::Expression>,
        operand: &resolved::PlacementOperand,
        span: Span,
    ) -> CheckResult<Expression> {
        match operand {
            resolved::PlacementOperand::Shape(shape) => {
                let element = layout_shape_type(&shape.kind);
                let value = self.check_expression(value, Some(&Type::Address))?;
                Ok(Expression {
                    kind: ExpressionKind::Memory {
                        primitive: MemoryPrimitive::Place,
                        argument: Box::new(value),
                    },
                    ty: Type::Cursor(Box::new(element).into()),
                    span,
                })
            }
            resolved::PlacementOperand::Value(count) => {
                let cursor = self.check_expression(value, None)?;
                let Type::Cursor(element) = &cursor.ty else {
                    return Err(Diagnostic::error("region placement requires a Cursor")
                        .with_primary(
                            cursor.span,
                            format!("this has type `{}`", type_name(&cursor.ty)),
                        )
                        .into());
                };
                let element = element.clone();
                let count = self.check_expression(count, Some(&Type::USize))?;
                Ok(Expression {
                    kind: ExpressionKind::Memory {
                        primitive: MemoryPrimitive::Region,
                        argument: Box::new(Expression {
                            kind: ExpressionKind::Product(vec![cursor, count]),
                            ty: Type::Product(
                                vec![Type::Cursor(element.clone()), Type::USize].into(),
                            ),
                            span,
                        }),
                    },
                    ty: Type::Region(element),
                    span,
                })
            }
        }
    }

    pub(super) fn check_stride_query(&self, shape: &Node<LayoutShape>, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::StorageSize(layout_shape_type(&shape.kind)),
            ty: Type::ByteSize,
            span,
        }
    }

    pub(super) fn check_align(
        &mut self,
        operand: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let operand = self.check_expression(operand, None)?;
        if !matches!(operand.ty, Type::Cursor(_) | Type::Region(_)) {
            return Err(Diagnostic::error("alignment requires a Cursor or Region")
                .with_primary(
                    operand.span,
                    format!("this has type `{}`", type_name(&operand.ty)),
                )
                .into());
        }
        let ty = operand.ty.clone();
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::Align,
                argument: Box::new(operand),
            },
            ty,
            span,
        })
    }

    pub(super) fn check_memory_unary(
        &mut self,
        operator: UnaryOperator,
        operand: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let operand = self.check_expression(operand, None)?;
        let (primitive, ty) = match (operator, &operand.ty) {
            (UnaryOperator::ProjectAddress, Type::Cursor(_) | Type::Region(_)) => {
                (MemoryPrimitive::ProjectAddress, Type::Address)
            }
            (UnaryOperator::Load, Type::Cursor(element)) => (
                MemoryPrimitive::LoadValue,
                Type::Product(vec![(**element).clone(), Type::Cursor(element.clone())].into()),
            ),
            (UnaryOperator::Load, Type::Region(element)) => {
                (MemoryPrimitive::AdmitRegion, Type::Packed(element.clone()))
            }
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
                        .with_primary(
                            operand.span,
                            format!("this has type `{}`", type_name(&operand.ty)),
                        )
                        .into(),
                );
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                argument: Box::new(operand),
            },
            ty,
            span,
        })
    }

    pub(super) fn check_memory_store(
        &mut self,
        cursor: Expression,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        if let Type::Region(element) = &cursor.ty {
            let element = element.clone();
            let packed_type = Type::Packed(element.clone());
            let (region, packed) = self.check_after(cursor, value, Some(&packed_type))?;
            return Ok(Expression {
                kind: ExpressionKind::Memory {
                    primitive: MemoryPrimitive::StorePacked,
                    argument: Box::new(Expression {
                        kind: ExpressionKind::Product(vec![region, packed]),
                        ty: Type::Product(vec![Type::Region(element.clone()), packed_type].into()),
                        span,
                    }),
                },
                ty: Type::Region(element),
                span,
            });
        }
        let Type::Cursor(element) = &cursor.ty else {
            return Err(
                Diagnostic::error("memory store requires a Cursor on the left")
                    .with_primary(
                        cursor.span,
                        format!("this has type `{}`", type_name(&cursor.ty)),
                    )
                    .into(),
            );
        };
        let element = element.clone();
        let (cursor, value) = self.check_after(cursor, value, Some(element.as_ref()))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::StoreValue,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![cursor, value]),
                    ty: Type::Product(
                        vec![Type::Cursor(element.clone()), (*element).clone()].into(),
                    ),
                    span,
                }),
            },
            ty: Type::Cursor(element),
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
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![value, count]),
                    ty: Type::Product(vec![ty.clone(), Type::USize].into()),
                    span,
                }),
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
            unreachable!("caller checks Packed")
        };
        let element = element.clone();
        let (packed, index) = self.check_after(packed, index, Some(&Type::USize))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::PackedIndex,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![packed, index]),
                    ty: Type::Product(vec![Type::Packed(element.clone()), Type::USize].into()),
                    span,
                }),
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
                .map(|element| layout_shape_type(&element.kind))
                .collect(),
        ),
        LayoutShape::Sum(elements) => Type::Sum(
            elements
                .iter()
                .map(|element| layout_shape_type(&element.kind))
                .collect(),
        ),
    }
}
