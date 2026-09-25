use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

use super::ast::{Expression, ExpressionKind, Type};
use super::float::is_float;
use super::integer::is_integer;
use super::types::type_name;
use super::{CheckFailure, CheckResult, Checker};

mod application;
mod elimination;

impl Checker {
    pub(super) fn check_before(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
        later: mal_syntax::source::Span,
    ) -> CheckResult<Expression> {
        match self.check_expression(expression, expected) {
            Err(CheckFailure::Abrupt(_)) => Err(Diagnostic::error(
                "unreachable expression after abrupt completion",
            )
            .with_primary(later, "this expression cannot be reached")
            .into()),
            result => result,
        }
    }

    pub(super) fn check_after(
        &mut self,
        preceding: Expression,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<(Expression, Expression)> {
        match self.check_expression(expression, expected) {
            Ok(value) => Ok((preceding, value)),
            Err(CheckFailure::Abrupt(abrupt)) => Err(CheckFailure::Abrupt(Box::new(
                (*abrupt).preceded_by(vec![preceding]),
            ))),
            Err(error) => Err(error),
        }
    }

    pub(super) fn check_expression(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let checked = match &expression.kind {
            resolved::Expression::Reference(reference) => Expression {
                kind: if self.generic_signatures.contains_key(&reference.id) {
                    return Err(Diagnostic::error("generic value requires type arguments")
                        .with_primary(reference.name.span, "supply the declared type arguments")
                        .into());
                } else {
                    ExpressionKind::Reference(reference.clone())
                },
                ty: self.value_type(reference)?,
                span: expression.span,
            },
            resolved::Expression::GenericReference {
                reference,
                arguments,
            } => {
                let Some(signature) = self.generic_signatures.get(&reference.id).cloned() else {
                    return Err(Diagnostic::error("value does not accept type arguments")
                        .with_primary(reference.name.span, "remove these type arguments")
                        .into());
                };
                if arguments.len() != signature.parameters.len() {
                    return Err(Diagnostic::error("generic value argument arity mismatch")
                        .with_primary(
                            reference.name.span,
                            format!(
                                "expected {} arguments but found {}",
                                signature.parameters.len(),
                                arguments.len()
                            ),
                        )
                        .into());
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expand_type(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                if self
                    .active_generic
                    .as_ref()
                    .is_some_and(|(id, _)| *id == reference.id)
                {
                    let (_, parameters) = self.active_generic.as_ref().unwrap();
                    let same_key = arguments.iter().zip(parameters).all(|(argument, parameter)| {
                        matches!(argument, Type::Parameter { id, .. } if id == parameter)
                    });
                    if !same_key {
                        return Err(Diagnostic::error("polymorphic recursion is not supported")
                            .with_primary(
                                reference.name.span,
                                "self recursion must preserve the type argument list",
                            )
                            .into());
                    }
                }
                for required in &signature.requirements {
                    let index = signature
                        .parameters
                        .iter()
                        .position(|parameter| parameter.id == *required)
                        .expect("requirements refer to declared parameters");
                    if !super::types::satisfies_storable_requirement(
                        &arguments[index],
                        &self.active_requirements,
                    ) {
                        return Err(Diagnostic::error(
                            "generic application lacks a Storable requirement",
                        )
                        .with_primary(
                            reference.name.span,
                            format!(
                                "type argument `{}` is not known to be storable",
                                type_name(&arguments[index])
                            ),
                        )
                        .into());
                    }
                }
                let substitutions = signature
                    .parameters
                    .iter()
                    .map(|parameter| parameter.id)
                    .zip(arguments.iter().cloned())
                    .collect();
                Expression {
                    kind: ExpressionKind::GenericReference {
                        reference: reference.clone(),
                        arguments,
                    },
                    ty: super::types::substitute_type(&signature.ty, &substitutions),
                    span: expression.span,
                }
            }
            resolved::Expression::Integer(literal) => {
                self.check_integer(literal, expression.span, expected)?
            }
            resolved::Expression::Float(literal) => {
                self.check_float(literal, expression.span, expected)?
            }
            resolved::Expression::Byte(value) => Expression {
                kind: ExpressionKind::Integer(i128::from(*value)),
                ty: Type::UInt8,
                span: expression.span,
            },
            resolved::Expression::Symbol(value) => Expression {
                kind: ExpressionKind::Symbol(value.clone()),
                ty: Type::Symbol,
                span: expression.span,
            },
            resolved::Expression::Unit => Expression {
                kind: ExpressionKind::Unit,
                ty: Type::Unit,
                span: expression.span,
            },
            resolved::Expression::Parenthesized(inner) => {
                let inner = self.check_expression(inner, expected)?;
                Expression {
                    ty: inner.ty.clone(),
                    kind: ExpressionKind::Parenthesized(Box::new(inner)),
                    span: expression.span,
                }
            }
            resolved::Expression::Product(elements) => {
                self.check_product(elements, expression.span, expected)?
            }
            resolved::Expression::Block(block) => {
                self.check_block(block, expression.span, expected)?
            }
            resolved::Expression::ResultBlock {
                result_binders,
                body,
            } => self.check_result_block(result_binders, body, expression.span, expected)?,
            resolved::Expression::Lambda(lambda) => {
                self.check_lambda(lambda, expression.span, expected)?
            }
            resolved::Expression::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span, expected)?
            }
            resolved::Expression::ContinuationApplication {
                value,
                continuations,
            } => self.check_continuation_application(
                value,
                continuations,
                expression.span,
                expected,
            )?,
            resolved::Expression::Conversion { type_ref, value } => {
                let target = self.expand_type_id(type_ref.id, type_ref.name.span)?;
                if matches!(target, Type::Sum(_)) {
                    return Err(Diagnostic::error(
                        "sum values must be constructed through result binders",
                    )
                    .with_primary(type_ref.name.span, "this is a sum type")
                    .into());
                }
                if !is_integer(&target) && !is_float(&target) {
                    return Err(
                        Diagnostic::error("numeric conversion requires a numeric type")
                            .with_primary(type_ref.name.span, "this is not a numeric type")
                            .into(),
                    );
                }
                let value = self.check_expression(value, None)?;
                if !is_integer(&value.ty) && !is_float(&value.ty) {
                    return Err(
                        Diagnostic::error("numeric conversion requires a numeric value")
                            .with_primary(
                                value.span,
                                format!("this has type `{}`", type_name(&value.ty)),
                            )
                            .into(),
                    );
                }
                Expression {
                    kind: ExpressionKind::NumericConversion {
                        value: Box::new(value),
                    },
                    ty: target,
                    span: expression.span,
                }
            }
            resolved::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => self.check_if(
                condition,
                then_branch,
                else_branch,
                expression.span,
                expected,
            )?,
            resolved::Expression::When { condition, body } => {
                self.check_when(condition, body, expression.span)?
            }
            resolved::Expression::Unary { operator, operand } => {
                if matches!(operator.kind, mal_syntax::ast::UnaryOperator::Star) {
                    self.check_memory_unary(operator.kind, operand, expression.span)?
                } else {
                    self.check_unary(operator, operand, expression.span, expected)?
                }
            }
            resolved::Expression::Binary {
                operator,
                left,
                right,
            } => self.check_binary_chain(operator, left, right, expression.span, expected)?,
        };
        if let Some(expected) = expected {
            self.require_type(&checked.ty, expected, checked.span)?;
        }
        Ok(checked)
    }

    fn check_binary_chain(
        &mut self,
        operator: &Node<mal_syntax::ast::BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: mal_syntax::source::Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let mut outer = Vec::new();
        let mut operator = operator;
        let mut left = left;
        let mut right = right;
        let mut span = span;
        let mut expected = expected.cloned();

        while let resolved::Expression::Binary {
            operator: inner_operator,
            left: inner_left,
            right: inner_right,
        } = &left.kind
        {
            let inner_span = left.span;
            let left_expected = self.binary_left_expected(operator, expected.as_ref());
            outer.push((operator, right, span, expected));
            expected = left_expected;
            operator = inner_operator;
            left = inner_left;
            right = inner_right;
            span = inner_span;
        }

        let mut checked = self.check_binary(operator, left, right, span, expected.as_ref())?;
        while let Some((operator, right, span, _)) = outer.pop() {
            checked = self.check_binary_after_left(operator, checked, right, span)?;
        }
        Ok(checked)
    }

    pub(super) fn require_type(
        &self,
        actual: &Type,
        expected: &Type,
        span: mal_syntax::source::Span,
    ) -> Result<(), Diagnostic> {
        if actual == expected {
            Ok(())
        } else {
            Err(self.type_mismatch(expected, actual, span))
        }
    }

    pub(super) fn type_mismatch(
        &self,
        expected: &Type,
        actual: &Type,
        span: mal_syntax::source::Span,
    ) -> Diagnostic {
        Diagnostic::error("type mismatch").with_primary(
            span,
            format!(
                "expected `{}`, found `{}`",
                type_name(expected),
                type_name(actual)
            ),
        )
    }
}
