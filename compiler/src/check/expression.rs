use crate::ast::{BinaryOperator, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::lexer::IntegerLiteral;
use crate::resolve::ast as resolved;
use crate::resolve::ast::{BYTE_AT_VALUE, BYTE_LENGTH_VALUE};

use super::Checker;
use super::ast::{Capture, Expression, ExpressionKind, Lambda, LambdaBody, Parameter, Type};
use super::float::{is_contextual_float, is_float};
use super::integer::{
    integer_is_signed, integer_negative_magnitude, is_contextual_integer, is_integer, literal_type,
    parse_index, parse_magnitude, unparenthesized_integer,
};
use super::types::{bool_type, function_placeholder, type_name};

impl Checker {
    pub(super) fn check_expression(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let checked = match &expression.kind {
            resolved::Expression::Reference(reference) => Expression {
                kind: if matches!(reference.id, BYTE_LENGTH_VALUE | BYTE_AT_VALUE) {
                    return Err(Diagnostic::error(format!(
                        "primitive `{}` must be called directly",
                        reference.name.text
                    ))
                    .with_primary(expression.span, "expected a call at this reference"));
                } else {
                    ExpressionKind::Reference(reference.clone())
                },
                ty: self.value_type(reference)?,
                span: expression.span,
            },
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
            resolved::Expression::String(value) => Expression {
                kind: ExpressionKind::String(value.clone()),
                ty: Type::String,
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
            resolved::Expression::Lambda(lambda) => {
                self.check_lambda(lambda, expression.span, expected)?
            }
            resolved::Expression::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span)?
            }
            resolved::Expression::ExternalCall {
                operation,
                arguments,
            } => self.check_external_call(operation, arguments, expression.span)?,
            resolved::Expression::Conversion { type_ref, value } => {
                let target = self.expand_type_id(type_ref.id, type_ref.name.span)?;
                if !is_integer(&target) && !is_float(&target) {
                    return Err(
                        Diagnostic::error("numeric conversion requires a numeric type")
                            .with_primary(type_ref.name.span, "this is not a numeric type"),
                    );
                }
                let value = self.check_expression(value, None)?;
                if !is_integer(&value.ty) && !is_float(&value.ty) {
                    return Err(
                        Diagnostic::error("numeric conversion requires a numeric value")
                            .with_primary(
                                value.span,
                                format!("this has type `{}`", type_name(&value.ty)),
                            ),
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
            resolved::Expression::SumInjection {
                type_ref,
                index,
                value,
            } => self.check_sum_injection(type_ref, index, value, expression.span)?,
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
            resolved::Expression::Case { scrutinee, arms } => {
                self.check_case(scrutinee, arms, expression.span, expected)?
            }
            resolved::Expression::Unary { operator, operand } => {
                self.check_unary(operator, operand, expression.span, expected)?
            }
            resolved::Expression::Binary {
                operator,
                left,
                right,
            } => self.check_binary(operator, left, right, expression.span, expected)?,
        };
        if let Some(expected) = expected {
            self.require_type(&checked.ty, expected, checked.span)?;
        }
        Ok(checked)
    }

    fn check_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let expected_function = match expected {
            Some(Type::Function { parameter, result }) => {
                Some((parameter.as_ref().clone(), result.as_ref().clone()))
            }
            Some(other) => {
                return Err(self.type_mismatch(other, &function_placeholder(), span));
            }
            None => None,
        };

        let mut captures = Vec::with_capacity(lambda.captures.len());
        for capture in &lambda.captures {
            let ty = self.value_type(&capture.source)?;
            self.values.insert(capture.binding.id, ty.clone());
            captures.push(Capture {
                source: capture.source.clone(),
                binding: capture.binding.clone(),
                ty,
            });
        }

        let mut parameters = Vec::with_capacity(lambda.parameters.len());
        for parameter in &lambda.parameters {
            let ty = self.expand_type(&parameter.ty)?;
            self.values.insert(parameter.binding.id, ty.clone());
            parameters.push(Parameter {
                binding: parameter.binding.clone(),
                ty,
                span: parameter.span,
            });
        }
        let parameter_type = parameters
            .first()
            .map_or(Type::Unit, |parameter| parameter.ty.clone());
        let parameter_type = if parameters.len() > 1 {
            Type::Product(
                parameters
                    .iter()
                    .map(|parameter| parameter.ty.clone())
                    .collect(),
            )
        } else {
            parameter_type
        };
        if let Some((expected_parameter, _)) = &expected_function {
            self.require_type(&parameter_type, expected_parameter, span)?;
        }

        let mut items = Vec::with_capacity(lambda.body.items.len());
        for item in &lambda.body.items {
            items.push(self.check_body_item(item)?);
        }
        let expected_result = expected_function.as_ref().map(|(_, result)| result);
        let result = self.check_expression(&lambda.body.result, expected_result)?;
        let result_type = result.ty.clone();
        let ty = Type::Function {
            parameter: Box::new(parameter_type),
            result: Box::new(result_type),
        };
        Ok(Expression {
            kind: ExpressionKind::Lambda(Lambda {
                id: lambda.id,
                self_binding: lambda.self_binding,
                captures,
                parameters,
                body: LambdaBody {
                    items,
                    result: Box::new(result),
                    span: lambda.body.span,
                },
            }),
            ty,
            span,
        })
    }

    fn check_call(
        &mut self,
        callee: &Node<resolved::Expression>,
        arguments: &[Node<resolved::Expression>],
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        if let resolved::Expression::Reference(reference) = &callee.kind
            && let Some(result) = self.check_string_call(reference.id, arguments, span)
        {
            return result;
        }
        let callee = self.check_expression(callee, None)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return Err(
                Diagnostic::error("cannot call a non-function value").with_primary(
                    callee.span,
                    format!("this has type `{}`", type_name(&callee.ty)),
                ),
            );
        };
        let argument = self.check_argument(arguments, parameter, span)?;
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    fn check_external_call(
        &mut self,
        operation: &resolved::ExternalOperationReference,
        arguments: &[Node<resolved::Expression>],
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let signature = self.external_signature(operation.id);
        let argument = self.check_argument(arguments, &signature.parameter, span)?;
        Ok(Expression {
            kind: ExpressionKind::ExternalCall {
                id: operation.id,
                name: operation.name.clone(),
                argument: Box::new(argument),
            },
            ty: signature.result,
            span,
        })
    }

    pub(super) fn check_argument(
        &mut self,
        arguments: &[Node<resolved::Expression>],
        parameter: &Type,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        match arguments {
            [] => {
                self.require_type(&Type::Unit, parameter, span)?;
                Ok(Expression {
                    kind: ExpressionKind::Unit,
                    ty: Type::Unit,
                    span,
                })
            }
            [argument] => self.check_expression(argument, Some(parameter)),
            _ => self.check_product(arguments, span, Some(parameter)),
        }
    }

    fn check_sum_injection(
        &mut self,
        type_ref: &resolved::TypeReference,
        index: &Node<IntegerLiteral>,
        value: &Node<resolved::Expression>,
        span: crate::source::Span,
    ) -> Result<Expression, Diagnostic> {
        let ty = self.expand_type_id(type_ref.id, type_ref.name.span)?;
        let Type::Sum(members) = &ty else {
            return Err(
                Diagnostic::error(format!("`{}` is not a sum type", type_ref.name.text))
                    .with_primary(
                        type_ref.name.span,
                        format!("this names `{}`", type_name(&ty)),
                    ),
            );
        };
        let index_value = parse_index(&index.kind, index.span)?;
        let member = members.get(index_value).ok_or_else(|| {
            Diagnostic::error("sum variant index is out of range").with_primary(
                index.span,
                format!("this sum has {} variants", members.len()),
            )
        })?;
        let value = self.check_expression(value, Some(member))?;
        Ok(Expression {
            kind: ExpressionKind::SumInjection {
                index: index_value,
                value: Box::new(value),
            },
            ty,
            span,
        })
    }

    fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        if operator.kind == UnaryOperator::Negate
            && let Some(literal) = unparenthesized_integer(operand)
        {
            let ty = literal_type(literal.suffix).unwrap_or_else(|| {
                expected
                    .filter(|ty| is_integer(ty))
                    .cloned()
                    .unwrap_or(Type::Int64)
            });
            let magnitude = parse_magnitude(literal, operand.span)?;
            if integer_is_signed(&ty) && magnitude == integer_negative_magnitude(&ty) {
                return Ok(Expression {
                    kind: ExpressionKind::Integer(
                        -i128::try_from(magnitude).expect("signed literal magnitudes fit in i128"),
                    ),
                    ty,
                    span,
                });
            }
        }
        if operator.kind == UnaryOperator::Negate {
            let operand = self.check_expression(
                operand,
                expected.filter(|expected| is_integer(expected) || is_float(expected)),
            )?;
            if !is_integer(&operand.ty) && !is_float(&operand.ty) {
                return Err(
                    Diagnostic::error("numeric negation requires a numeric value").with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    ),
                );
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        if operator.kind == UnaryOperator::BitwiseNot {
            let operand =
                self.check_expression(operand, expected.filter(|expected| is_integer(expected)))?;
            if !is_integer(&operand.ty) {
                return Err(
                    Diagnostic::error("integer unary operator requires an integer").with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    ),
                );
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        let operand_type = bool_type();
        let operand = self.check_expression(operand, Some(&operand_type))?;
        Ok(Expression {
            kind: ExpressionKind::Unary {
                operator: operator.clone(),
                operand: Box::new(operand),
            },
            ty: operand_type,
            span,
        })
    }

    fn check_binary(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: crate::source::Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let expected_integer = expected.filter(|expected| is_integer(expected));
        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right, result) = match operator.kind {
            BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Add
            | BinaryOperator::Subtract => {
                let (left, right) = self.check_numeric_operands(left, right, expected_numeric)?;
                let result = left.ty.clone();
                (left, right, result)
            }
            BinaryOperator::Remainder => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = left.ty.clone();
                (left, right, result)
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                let (left, right) = self.check_numeric_operands(left, right, None)?;
                (left, right, bool_type())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
                let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
                let (left, right) = if left_contextual && !right_contextual {
                    let right = self.check_expression(right, None)?;
                    let left = self.check_expression(left, Some(&right.ty))?;
                    (left, right)
                } else {
                    let left = self.check_expression(left, None)?;
                    let right = self.check_expression(right, Some(&left.ty))?;
                    (left, right)
                };
                if !is_integer(&left.ty)
                    && !is_float(&left.ty)
                    && left.ty != bool_type()
                    && left.ty != Type::String
                {
                    return Err(Diagnostic::error("equality is not defined for this type")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        ));
                }
                (left, right, bool_type())
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => (
                self.check_expression(left, Some(&bool_type()))?,
                self.check_expression(right, Some(&bool_type()))?,
                bool_type(),
            ),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = left.ty.clone();
                (left, right, result)
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            span,
        })
    }

    fn check_numeric_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<(Expression, Expression), Diagnostic> {
        let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
        let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
        let (left, right) = if let Some(expected) = expected {
            (
                self.check_expression(left, Some(expected))?,
                self.check_expression(right, Some(expected))?,
            )
        } else if left_contextual && !right_contextual {
            let right = self.check_expression(right, None)?;
            let left = self.check_expression(left, Some(&right.ty))?;
            (left, right)
        } else {
            let left = self.check_expression(left, None)?;
            let right = self.check_expression(right, Some(&left.ty))?;
            (left, right)
        };
        if !is_integer(&left.ty) && !is_float(&left.ty) {
            return Err(
                Diagnostic::error("numeric operator requires numeric operands").with_primary(
                    left.span,
                    format!("this has type `{}`", type_name(&left.ty)),
                ),
            );
        }
        Ok((left, right))
    }

    fn require_type(
        &self,
        actual: &Type,
        expected: &Type,
        span: crate::source::Span,
    ) -> Result<(), Diagnostic> {
        if actual == expected {
            Ok(())
        } else {
            Err(self.type_mismatch(expected, actual, span))
        }
    }

    fn type_mismatch(
        &self,
        expected: &Type,
        actual: &Type,
        span: crate::source::Span,
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
