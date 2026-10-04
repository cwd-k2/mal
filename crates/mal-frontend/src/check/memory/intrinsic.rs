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
        self.check_memory_intrinsic_with_element(reference, element, arguments, span)
    }

    pub(crate) fn check_inferred_memory_intrinsic(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let expected = expected.map(|ty| super::super::types::representation_view(ty, span.file()));
        let Some(Type::Buffer(element)) = expected else {
            return Err(
                Diagnostic::error("memory intrinsic type argument cannot be inferred")
                    .with_primary(
                        reference.name.span,
                        "write an explicit type argument or provide an expected Buffer type",
                    )
                    .into(),
            );
        };
        let element = element.as_ref().clone();
        self.validate_memory_element(reference, &element, reference.name.span)?;
        self.check_memory_intrinsic_with_element(reference, element, arguments, span)
    }

    fn check_memory_intrinsic_with_element(
        &mut self,
        reference: &resolved::ValueReference,
        element: Type,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let (primitive, parameter) = match reference.id {
            crate::resolve::MAKE_VALUE => (MemoryPrimitive::BufferMake, Type::USize),
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
        self.validate_memory_element(reference, &element, argument.span)?;
        Ok(element)
    }

    fn validate_memory_element(
        &self,
        reference: &resolved::ValueReference,
        element: &Type,
        span: Span,
    ) -> CheckResult<()> {
        debug_assert_eq!(reference.id, crate::resolve::MAKE_VALUE);
        if !super::super::types::satisfies_storable_requirement(element, &self.active_requirements)
        {
            return Err(Diagnostic::error("make requires a storable element type")
                .with_primary(
                    span,
                    format!(
                        "`{}` is not known to satisfy the Buffer element lifecycle contract",
                        super::super::types::type_name(element)
                    ),
                )
                .into());
        }
        Ok(())
    }
}
