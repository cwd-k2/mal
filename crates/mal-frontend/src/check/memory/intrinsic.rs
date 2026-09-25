use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckResult, Checker};

impl Checker {
    pub(crate) fn check_memory_intrinsic(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let element = self.memory_element_type(reference, type_arguments)?;
        let (primitive, parameter) = match reference.id {
            crate::resolve::MAKE_VALUE => (MemoryPrimitive::BufferMake, Type::USize),
            crate::resolve::FROM_VALUE => (
                MemoryPrimitive::BufferFromAddress,
                Type::Product(vec![Type::Address, Type::USize, Type::USize].into()),
            ),
            _ => unreachable!("caller recognizes memory intrinsic identities"),
        };
        let argument = self.check_argument(arguments, &parameter, span)?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                operands: vec![argument],
            },
            ty: Type::Buffer(element.into()),
            span,
        })
    }

    fn memory_element_type(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::TypeExpression>],
    ) -> CheckResult<Type> {
        let [argument] = arguments else {
            return Err(
                Diagnostic::error("memory intrinsic type argument arity mismatch")
                    .with_primary(
                        reference.name.span,
                        format!("expected 1 argument but found {}", arguments.len()),
                    )
                    .into(),
            );
        };
        let element = self.expand_type(argument)?;
        if reference.id == crate::resolve::MAKE_VALUE {
            if !super::super::types::satisfies_storable_requirement(
                &element,
                &self.active_requirements,
            ) {
                return Err(Diagnostic::error("make requires a storable element type")
                    .with_primary(
                        argument.span,
                        format!(
                            "`{}` is not known to be an immutable value that a Buffer can hold",
                            super::super::types::type_name(&element)
                        ),
                    )
                    .into());
            }
        } else {
            super::ensure_copyable_element(&element, argument.span)?;
        }
        Ok(element)
    }
}
