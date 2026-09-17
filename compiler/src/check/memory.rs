use crate::ast::{LayoutShape, Name, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved};
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, MemoryScalar, Type};
use super::types::type_name;
use super::{CheckResult, Checker};

enum QualifiedPrimitive {
    Size(Type),
    Function(MemoryPrimitive),
}

impl Checker {
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

    pub(super) fn check_qualified_memory_primitive(
        &mut self,
        type_ref: &resolved::TypeReference,
        member: &Name,
        span: Span,
    ) -> CheckResult<Expression> {
        match self.qualified_memory_primitive(type_ref, member)? {
            QualifiedPrimitive::Size(ty) => Ok(Expression {
                kind: ExpressionKind::StorageSize(ty),
                ty: Type::UInt64,
                span,
            }),
            QualifiedPrimitive::Function(primitive) => {
                let (parameter, result) = primitive.signature();
                Ok(Expression {
                    kind: ExpressionKind::MemoryFunction { primitive },
                    ty: Type::Function {
                        parameter: parameter.into(),
                        result: result.into(),
                    },
                    span,
                })
            }
        }
    }

    pub(super) fn check_qualified_memory_call(
        &mut self,
        type_ref: &resolved::TypeReference,
        member: &Name,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let qualified = self.qualified_memory_primitive(type_ref, member)?;
        let QualifiedPrimitive::Function(primitive) = qualified else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(member.span, "`size` is a `UInt64` constant")
                .into());
        };
        let (parameter, result) = primitive.signature();
        let argument = self.check_argument(arguments, &parameter, span)?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                argument: Box::new(argument),
            },
            ty: result,
            span,
        })
    }

    fn qualified_memory_primitive(
        &mut self,
        type_ref: &resolved::TypeReference,
        member: &Name,
    ) -> Result<QualifiedPrimitive, Diagnostic> {
        let ty = self.expand_type_id(type_ref.id, type_ref.name.span)?;
        let primitive = match (memory_scalar(&ty), &ty, member.text.as_str()) {
            (Some(_), _, "size") | (None, Type::Ptr, "size") => {
                return Ok(QualifiedPrimitive::Size(ty));
            }
            (Some(scalar), _, "load") => MemoryPrimitive::Load(scalar),
            (Some(scalar), _, "store") => MemoryPrimitive::Store(scalar),
            (None, Type::Ptr, "load") => MemoryPrimitive::LoadPtr,
            (None, Type::Ptr, "store") => MemoryPrimitive::StorePtr,
            (None, Type::Symbol, "read") => MemoryPrimitive::LoadSymbol,
            (None, Type::Symbol, "write") => MemoryPrimitive::StoreSymbol,
            _ => {
                return Err(Diagnostic::error(format!(
                    "type `{}` has no predefined memory primitive `{}`",
                    type_name(&ty),
                    member.text
                ))
                .with_primary(member.span, "this primitive is not defined for the type"));
            }
        };
        Ok(QualifiedPrimitive::Function(primitive))
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

fn memory_scalar(ty: &Type) -> Option<MemoryScalar> {
    Some(match ty {
        Type::Int8 => MemoryScalar::Int8,
        Type::Int16 => MemoryScalar::Int16,
        Type::Int32 => MemoryScalar::Int32,
        Type::Int64 => MemoryScalar::Int64,
        Type::UInt8 => MemoryScalar::UInt8,
        Type::UInt16 => MemoryScalar::UInt16,
        Type::UInt32 => MemoryScalar::UInt32,
        Type::UInt64 => MemoryScalar::UInt64,
        Type::Float32 => MemoryScalar::Float32,
        Type::Float64 => MemoryScalar::Float64,
        _ => return None,
    })
}
