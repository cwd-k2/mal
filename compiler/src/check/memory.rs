use crate::ast::{Name, Node};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved};
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, MemoryPrimitive, MemoryScalar, Type};
use super::types::type_name;

enum QualifiedPrimitive {
    Size(Type),
    Function(MemoryPrimitive),
}

impl Checker {
    pub(super) fn check_qualified_memory_primitive(
        &mut self,
        type_ref: &resolved::TypeReference,
        member: &Name,
        span: Span,
    ) -> Result<Expression, Diagnostic> {
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
                        parameter: Box::new(parameter),
                        result: Box::new(result),
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
    ) -> Result<Expression, Diagnostic> {
        let qualified = self.qualified_memory_primitive(type_ref, member)?;
        let QualifiedPrimitive::Function(primitive) = qualified else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(member.span, "`size` is a `UInt64` constant"));
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
