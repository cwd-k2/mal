//! Expected-template probing for call operands and direct lambda results.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{self as resolved, TypeId};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

use super::super::Checker;
use super::super::ast::Type;
use super::super::float::is_contextual_float;
use super::super::integer::is_contextual_integer;
use super::constraint::{constrain, constrain_generic_scheme, has_unresolved};

impl Checker {
    pub(super) fn probe_constraint(
        &self,
        argument: &Node<resolved::Expression>,
        template: &Type,
        flexible: &HashSet<TypeId>,
        substitutions: &mut HashMap<TypeId, Type>,
        allow_defaults: bool,
    ) -> Result<(), Diagnostic> {
        let contextual = is_contextual_integer(argument) || is_contextual_float(argument);
        if contextual && !allow_defaults && has_unresolved(template, flexible, substitutions) {
            return Ok(());
        }
        if let resolved::Expression::Reference(reference) = &argument.kind
            && let Some(signature) = self.generic_signatures.get(&reference.id)
            && has_unresolved(template, flexible, substitutions)
        {
            constrain_generic_scheme(signature, template, flexible, substitutions, argument.span)?;
        }
        let instantiated = super::super::types::substitute_type(template, substitutions);
        let unresolved = has_unresolved(&instantiated, flexible, substitutions);
        let checked = if let resolved::Expression::Lambda(lambda) = &argument.kind
            && let Type::Function { parameter, result } = &instantiated
            && !has_unresolved(parameter, flexible, substitutions)
        {
            if has_unresolved(result, flexible, substitutions)
                && let Type::Function {
                    result: result_template,
                    ..
                } = template
            {
                self.probe_direct_result_constraints(
                    lambda,
                    result_template,
                    flexible,
                    substitutions,
                    allow_defaults,
                )?;
            }
            let mut probe = self.clone();
            let expected_result =
                (!has_unresolved(result, flexible, substitutions)).then_some(result.as_ref());
            probe.check_lambda_against(
                lambda,
                argument.span,
                parameter.as_ref().clone(),
                expected_result,
            )
        } else if contextual {
            let mut probe = self.clone();
            if unresolved {
                probe.check_expression(argument, None)
            } else {
                probe.check_expression(argument, Some(&instantiated))
            }
        } else {
            let mut probe = self.clone();
            match probe.check_expression(argument, None) {
                Ok(checked) => Ok(checked),
                Err(_) if !unresolved => {
                    let mut contextual_probe = self.clone();
                    contextual_probe.check_expression(argument, Some(&instantiated))
                }
                Err(error) => Err(error),
            }
        };
        if let Ok(checked) = checked {
            constrain(
                template,
                &checked.ty,
                flexible,
                substitutions,
                argument.span,
            )?;
        }
        Ok(())
    }

    fn probe_direct_result_constraints(
        &self,
        lambda: &resolved::Lambda,
        result_template: &Type,
        flexible: &HashSet<TypeId>,
        substitutions: &mut HashMap<TypeId, Type>,
        allow_defaults: bool,
    ) -> Result<(), Diagnostic> {
        let resolved::Expression::ResultBlock {
            result_binders,
            body,
        } = &lambda.body.result.kind
        else {
            return Ok(());
        };
        let substituted = super::super::types::substitute_type(result_template, substitutions);
        let templates = match result_binders.as_slice() {
            [_] => vec![substituted],
            _ => {
                let Type::Sum(members) = substituted else {
                    return Ok(());
                };
                if members.len() != result_binders.len() {
                    return Ok(());
                }
                members.to_vec()
            }
        };
        let targets = result_binders
            .iter()
            .zip(&templates)
            .map(|(binder, template)| (binder.id, template))
            .collect::<HashMap<_, _>>();
        let mut pending = Vec::new();
        push_body_expressions(body, &mut pending);
        while let Some(expression) = pending.pop() {
            match &expression.kind {
                resolved::Expression::Call { callee, arguments } => {
                    if let resolved::Expression::Reference(reference) = &callee.kind
                        && let Some(template) = targets.get(&reference.id)
                    {
                        match arguments.as_slice() {
                            [] => constrain(
                                template,
                                &Type::Unit,
                                flexible,
                                substitutions,
                                expression.span,
                            )?,
                            [argument] => self.probe_constraint(
                                argument,
                                template,
                                flexible,
                                substitutions,
                                allow_defaults,
                            )?,
                            _ => {
                                let mut probe = self.clone();
                                if let Ok(argument) =
                                    probe.check_untyped_argument(arguments, expression.span)
                                {
                                    constrain(
                                        template,
                                        &argument.ty,
                                        flexible,
                                        substitutions,
                                        expression.span,
                                    )?;
                                }
                            }
                        }
                    }
                    pending.push(callee);
                    pending.extend(arguments.iter());
                }
                resolved::Expression::Parenthesized(inner) => pending.push(inner),
                resolved::Expression::Product(elements) => pending.extend(elements),
                resolved::Expression::Block(block)
                | resolved::Expression::ResultBlock { body: block, .. } => {
                    push_body_expressions(block, &mut pending)
                }
                resolved::Expression::ContinuationApplication {
                    value,
                    continuations,
                } => {
                    pending.push(value);
                    for continuation in continuations {
                        match continuation {
                            resolved::Continuation::Function(expression) => {
                                pending.push(expression)
                            }
                            resolved::Continuation::Branch(branch) => {
                                push_body_expressions(&branch.body, &mut pending)
                            }
                        }
                    }
                }
                resolved::Expression::Conversion { value, .. }
                | resolved::Expression::Unary { operand: value, .. } => pending.push(value),
                resolved::Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    pending.push(condition);
                    push_body_expressions(then_branch, &mut pending);
                    push_body_expressions(else_branch, &mut pending);
                }
                resolved::Expression::When { condition, body } => {
                    pending.push(condition);
                    push_body_expressions(body, &mut pending);
                }
                resolved::Expression::Binary { left, right, .. } => {
                    pending.push(left);
                    pending.push(right);
                }
                resolved::Expression::Lambda(_)
                | resolved::Expression::Reference(_)
                | resolved::Expression::GenericReference { .. }
                | resolved::Expression::Integer(_)
                | resolved::Expression::Float(_)
                | resolved::Expression::Byte(_)
                | resolved::Expression::Symbol(_)
                | resolved::Expression::Unit => {}
            }
        }
        Ok(())
    }
}

fn push_body_expressions<'a>(
    body: &'a resolved::ExpressionBlock,
    pending: &mut Vec<&'a Node<resolved::Expression>>,
) {
    for item in &body.items {
        match item {
            resolved::BodyItem::Binding(binding) => pending.push(&binding.kind.value),
            resolved::BodyItem::Expression(expression) => pending.push(expression),
        }
    }
    pending.push(&body.result);
}
