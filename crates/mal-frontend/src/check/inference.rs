//! Local type-argument constraints for generic value references and calls.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast as resolved;
use crate::resolve::ast::TypeId;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{Expression, Type};
use super::float::is_contextual_float;
use super::integer::is_contextual_integer;
use super::{CheckResult, Checker, GenericSignature};

enum GenericCallExpectation<'a> {
    Result(Option<&'a Type>),
    ReturnedFunctionParameter(&'a Type),
}

impl Checker {
    pub(super) fn check_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: Vec<Type>,
        span: Span,
    ) -> CheckResult<Expression> {
        let signature = self
            .generic_signatures
            .get(&reference.id)
            .expect("generic reference has a collected signature")
            .clone();
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
                return Err(
                    Diagnostic::error("generic application lacks a Storable requirement")
                        .with_primary(
                            reference.name.span,
                            format!(
                                "type argument `{}` is not known to be storable",
                                super::types::type_name(&arguments[index])
                            ),
                        )
                        .into(),
                );
            }
        }
        let substitutions = signature
            .parameters
            .iter()
            .map(|parameter| parameter.id)
            .zip(arguments.iter().cloned())
            .collect();
        let kind = if self.operation_families.contains(&reference.id) {
            if self.active_generic.is_some()
                && !self.active_operations.iter().any(|requirement| {
                    requirement.family.id == reference.id && requirement.arguments == arguments
                })
            {
                self.active_operations
                    .push(super::ast::OperationRequirement {
                        family: reference.clone(),
                        arguments: arguments.clone(),
                    });
            }
            super::ast::ExpressionKind::OperationReference {
                family: reference.clone(),
                arguments,
            }
        } else {
            if self.active_generic.is_some() {
                for requirement in &signature.operations {
                    let requirement_arguments = requirement
                        .arguments
                        .iter()
                        .map(|argument| super::types::substitute_type(argument, &substitutions))
                        .collect::<Vec<_>>();
                    if !self.active_operations.iter().any(|existing| {
                        existing.family.id == requirement.family.id
                            && existing.arguments == requirement_arguments
                    }) {
                        self.active_operations
                            .push(super::ast::OperationRequirement {
                                family: requirement.family.clone(),
                                arguments: requirement_arguments,
                            });
                    }
                }
            }
            super::ast::ExpressionKind::GenericReference {
                reference: reference.clone(),
                arguments,
            }
        };
        Ok(Expression {
            kind,
            ty: super::types::substitute_type(&signature.ty, &substitutions),
            span,
        })
    }

    pub(super) fn check_inferred_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        if let Some(expected) = expected {
            constrain(
                &signature.ty,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?;
        }
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_inferred_generic_continuation_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let Type::Function {
            parameter: parameter_template,
            ..
        } = &signature.ty
        else {
            return self.check_inferred_generic_reference(reference, span, None);
        };
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        constrain(
            parameter_template,
            parameter,
            &flexible,
            &mut substitutions,
            reference.name.span,
        )?;
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_inferred_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            arguments,
            span,
            GenericCallExpectation::Result(expected),
        )
    }

    pub(super) fn check_inferred_generic_continuation_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            arguments,
            span,
            GenericCallExpectation::ReturnedFunctionParameter(parameter),
        )
    }

    fn check_inferred_generic_call_with_expectations(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: GenericCallExpectation<'_>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let Type::Function { parameter, result } = &signature.ty else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(reference.name.span, "this generic value is not a function")
                .into());
        };
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        match expected {
            GenericCallExpectation::Result(Some(expected)) => constrain(
                result,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?,
            GenericCallExpectation::ReturnedFunctionParameter(expected)
                if let Type::Function { parameter, .. } = result.as_ref() =>
            {
                constrain(
                    parameter,
                    expected,
                    &flexible,
                    &mut substitutions,
                    reference.name.span,
                )?;
            }
            GenericCallExpectation::Result(None)
            | GenericCallExpectation::ReturnedFunctionParameter(_) => {}
        }

        let templates = argument_templates(parameter, arguments.len());
        let mut allow_defaults = false;
        loop {
            let before = substitutions.len();
            match &templates {
                Some(templates) => {
                    for (argument, template) in arguments.iter().zip(templates) {
                        self.probe_constraint(
                            argument,
                            template,
                            &flexible,
                            &mut substitutions,
                            allow_defaults,
                        )?;
                    }
                }
                None => {
                    let mut probe = self.clone();
                    if let Ok(actual) = probe.check_untyped_argument(arguments, span) {
                        constrain(parameter, &actual.ty, &flexible, &mut substitutions, span)?;
                    }
                }
            }
            if substitutions.len() == before {
                if allow_defaults {
                    break;
                }
                allow_defaults = true;
            }
        }

        let type_arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        let callee =
            self.check_generic_reference(reference, type_arguments, reference.name.span)?;
        let Type::Function { parameter, result } = &callee.ty else {
            unreachable!("the generic signature was a function");
        };
        let argument = match self.check_argument(arguments, parameter, span) {
            Ok(argument) => argument,
            Err(super::CheckFailure::Abrupt(_)) => {
                return Err(Diagnostic::error(
                    "function value is unreachable after abrupt argument evaluation",
                )
                .with_primary(callee.span, "this callee cannot be evaluated")
                .into());
            }
            Err(error) => return Err(error),
        };
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: super::ast::ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }

    fn probe_constraint(
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
        let instantiated = super::types::substitute_type(template, substitutions);
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
        let substituted = super::types::substitute_type(result_template, substitutions);
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

fn parameter_ids(signature: &GenericSignature) -> HashSet<TypeId> {
    signature
        .parameters
        .iter()
        .map(|parameter| parameter.id)
        .collect()
}

fn argument_templates(parameter: &Type, count: usize) -> Option<Vec<&Type>> {
    match (count, parameter) {
        (0, Type::Unit) => Some(Vec::new()),
        (1, parameter) => Some(vec![parameter]),
        (_, Type::Product(elements)) if elements.len() == count => Some(elements.iter().collect()),
        _ => None,
    }
}

fn inferred_arguments(
    signature: &GenericSignature,
    substitutions: &HashMap<TypeId, Type>,
    span: Span,
) -> Result<Vec<Type>, super::CheckFailure> {
    let missing = signature
        .parameters
        .iter()
        .filter(|parameter| !substitutions.contains_key(&parameter.id))
        .map(|parameter| parameter.name.text.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(
            Diagnostic::error("generic type arguments cannot be inferred")
                .with_primary(
                    span,
                    format!("write explicit type arguments for {}", missing.join(", ")),
                )
                .into(),
        );
    }
    Ok(signature
        .parameters
        .iter()
        .map(|parameter| resolve_substitution(parameter.id, substitutions, &mut HashSet::new()))
        .collect())
}

fn constrain_generic_scheme(
    signature: &GenericSignature,
    expected: &Type,
    outer_flexible: &HashSet<TypeId>,
    outer_substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    let inner_flexible = parameter_ids(signature);
    let flexible = outer_flexible
        .union(&inner_flexible)
        .copied()
        .collect::<HashSet<_>>();
    let mut substitutions = outer_substitutions.clone();
    unify_flexible(&signature.ty, expected, &flexible, &mut substitutions, span)?;
    for id in outer_flexible {
        if substitutions.contains_key(id) {
            let resolved = resolve_substitution(*id, &substitutions, &mut HashSet::new());
            if !contains_unbound_from(&resolved, &inner_flexible, &substitutions) {
                outer_substitutions.insert(*id, resolved);
            }
        }
    }
    Ok(())
}

fn unify_flexible(
    left: &Type,
    right: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Type::Parameter { id, .. } = left
        && flexible.contains(id)
    {
        if let Some(bound) = substitutions.get(id).cloned() {
            return unify_flexible(&bound, right, flexible, substitutions, span);
        }
        substitutions.insert(*id, right.clone());
        return Ok(());
    }
    if let Type::Parameter { id, .. } = right
        && flexible.contains(id)
    {
        if let Some(bound) = substitutions.get(id).cloned() {
            return unify_flexible(left, &bound, flexible, substitutions, span);
        }
        substitutions.insert(*id, left.clone());
        return Ok(());
    }
    match (left, right) {
        (Type::Buffer(left), Type::Buffer(right)) => {
            unify_flexible(left, right, flexible, substitutions, span)
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            for (left, right) in left.iter().zip(right.iter()) {
                unify_flexible(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (
            Type::Function {
                parameter: left_parameter,
                result: left_result,
            },
            Type::Function {
                parameter: right_parameter,
                result: right_result,
            },
        ) => {
            unify_flexible(
                left_parameter,
                right_parameter,
                flexible,
                substitutions,
                span,
            )?;
            unify_flexible(left_result, right_result, flexible, substitutions, span)
        }
        _ if left == right => Ok(()),
        _ => Err(inference_conflict(left, right, span)),
    }
}

fn resolve_substitution(
    id: TypeId,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
) -> Type {
    if !visiting.insert(id) {
        return substitutions[&id].clone();
    }
    let resolved = resolve_type(&substitutions[&id], substitutions, visiting);
    visiting.remove(&id);
    resolved
}

fn resolve_type(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
) -> Type {
    match ty {
        Type::Parameter { id, .. } if substitutions.contains_key(id) => {
            resolve_substitution(*id, substitutions, visiting)
        }
        Type::Buffer(element) => {
            Type::Buffer(resolve_type(element, substitutions, visiting).into())
        }
        Type::Product(elements) => Type::Product(
            elements
                .iter()
                .map(|element| resolve_type(element, substitutions, visiting))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Sum(members) => Type::Sum(
            members
                .iter()
                .map(|member| resolve_type(member, substitutions, visiting))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Function { parameter, result } => Type::Function {
            parameter: resolve_type(parameter, substitutions, visiting).into(),
            result: resolve_type(result, substitutions, visiting).into(),
        },
        _ => ty.clone(),
    }
}

fn contains_unbound_from(
    ty: &Type,
    parameters: &HashSet<TypeId>,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    match ty {
        Type::Parameter { id, .. } => parameters.contains(id) && !substitutions.contains_key(id),
        Type::Buffer(element) => contains_unbound_from(element, parameters, substitutions),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| contains_unbound_from(element, parameters, substitutions)),
        Type::Function { parameter, result } => {
            contains_unbound_from(parameter, parameters, substitutions)
                || contains_unbound_from(result, parameters, substitutions)
        }
        _ => false,
    }
}

fn has_unresolved(
    ty: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    match ty {
        Type::Parameter { id, .. } => flexible.contains(id) && !substitutions.contains_key(id),
        Type::Buffer(element) => has_unresolved(element, flexible, substitutions),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| has_unresolved(element, flexible, substitutions)),
        Type::Function { parameter, result } => {
            has_unresolved(parameter, flexible, substitutions)
                || has_unresolved(result, flexible, substitutions)
        }
        _ => false,
    }
}

fn constrain(
    template: &Type,
    actual: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Type::Parameter { id, .. } = template
        && flexible.contains(id)
    {
        if let Some(previous) = substitutions.get(id) {
            if previous == actual {
                return Ok(());
            }
            return Err(inference_conflict(previous, actual, span));
        }
        substitutions.insert(*id, actual.clone());
        return Ok(());
    }
    match (template, actual) {
        (Type::Buffer(left), Type::Buffer(right)) => {
            constrain(left, right, flexible, substitutions, span)
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            for (left, right) in left.iter().zip(right.iter()) {
                constrain(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (
            Type::Function {
                parameter: left_parameter,
                result: left_result,
            },
            Type::Function {
                parameter: right_parameter,
                result: right_result,
            },
        ) => {
            constrain(
                left_parameter,
                right_parameter,
                flexible,
                substitutions,
                span,
            )?;
            constrain(left_result, right_result, flexible, substitutions, span)
        }
        _ if template == actual => Ok(()),
        _ => Err(inference_conflict(template, actual, span)),
    }
}

fn inference_conflict(expected: &Type, actual: &Type, span: Span) -> Diagnostic {
    Diagnostic::error("conflicting generic type inference").with_primary(
        span,
        format!(
            "inferred both `{}` and `{}`",
            super::types::type_name(expected),
            super::types::type_name(actual)
        ),
    )
}
