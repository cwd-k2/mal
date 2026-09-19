use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(crate) fn check_memory_intrinsic(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let element = self.memory_element_type(reference, type_arguments)?;
        match reference.id {
            crate::resolve::PACK_VALUE => self.check_address_pack(element, arguments, span),
            crate::resolve::VIEW_VALUE => {
                self.check_region_view(element, arguments, span, expected)
            }
            _ => unreachable!("caller recognizes memory intrinsic identities"),
        }
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
        if !super::super::types::satisfies_representable_requirement(
            &element,
            &self.active_requirements,
        ) {
            return Err(Diagnostic::error(
                "memory intrinsic requires a Representable element type",
            )
            .with_primary(
                argument.span,
                format!(
                    "`{}` has no available canonical memory representation",
                    super::super::types::type_name(&element)
                ),
            )
            .into());
        }
        Ok(element)
    }

    fn check_address_pack(
        &mut self,
        element: Type,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let parameter = Type::Product(vec![Type::Address, Type::USize, Type::USize].into());
        let range = self.check_argument(arguments, &parameter, span)?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive: MemoryPrimitive::PackAddress,
                operands: vec![range],
            },
            ty: Type::Packed(element.into()),
            span,
        })
    }

    fn check_region_view(
        &mut self,
        element: Type,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let [address, start, end, callback] = arguments else {
            return Err(Diagnostic::error("view argument arity mismatch")
                .with_primary(
                    span,
                    format!("expected 4 arguments but found {}", arguments.len()),
                )
                .into());
        };
        let range_type = Type::Product(vec![Type::Address, Type::USize, Type::USize].into());
        let range = self.check_product(
            &[address.clone(), start.clone(), end.clone()],
            span,
            Some(&range_type),
        )?;
        let region = Type::Region(element.clone().into());
        let checked_callback = match &callback.kind {
            resolved::Expression::Lambda(lambda) => {
                self.check_lambda_against(lambda, callback.span, region, expected)?
            }
            _ => match self.check_expression(callback, None) {
                Ok(value) => value,
                Err(CheckFailure::Abrupt(abrupt)) => {
                    return Err(CheckFailure::Abrupt(Box::new(
                        (*abrupt).preceded_by(vec![range]),
                    )));
                }
                Err(error) => return Err(error),
            },
        };
        let Type::Function { parameter, result } = &checked_callback.ty else {
            return Err(Diagnostic::error("view callback must be a function")
                .with_primary(checked_callback.span, "expected `Region<A> -> R`")
                .into());
        };
        self.require_type(
            parameter,
            &Type::Region(element.clone().into()),
            checked_callback.span,
        )?;
        if super::super::types::contains_scoped_value(result) {
            return Err(
                Diagnostic::error("view callback cannot return scoped authority")
                    .with_primary(
                        checked_callback.span,
                        "return a value that does not contain Region or Buffer",
                    )
                    .into(),
            );
        }
        let result = result.as_ref().clone();
        Ok(Expression {
            kind: ExpressionKind::RegionView {
                range: Box::new(range),
                callback: Box::new(checked_callback),
                element,
            },
            ty: result,
            span,
        })
    }
}
