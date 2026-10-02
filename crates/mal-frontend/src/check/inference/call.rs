//! Generic calls: type arguments inferred from the operand and expected result, or given explicitly.

use super::*;

impl Checker {
    pub(in crate::check) fn check_inferred_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            &[],
            arguments,
            span,
            GenericCallExpectation::Result(expected),
        )
    }

    pub(in crate::check) fn check_inferred_generic_continuation_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            &[],
            arguments,
            span,
            GenericCallExpectation::ReturnedFunctionParameter(parameter),
        )
    }

    pub(in crate::check) fn check_explicit_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        if type_arguments.len() > signature.parameters.len() {
            return Err(Diagnostic::error("generic value argument arity mismatch")
                .with_primary(
                    reference.name.span,
                    format!(
                        "expected at most {} arguments but found {}",
                        signature.parameters.len(),
                        type_arguments.len()
                    ),
                )
                .into());
        }
        let mut explicit = type_arguments
            .iter()
            .map(|argument| self.expand_type_term(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if self
            .active_generic
            .as_ref()
            .is_some_and(|(id, _)| *id == reference.id)
            && !self.operation_families.contains(&reference.id)
        {
            let (_, parameters) = self.active_generic.as_ref().unwrap();
            let preserves_key = explicit.iter().zip(parameters).all(|(argument, parameter)| {
                matches!(argument, Type::Parameter { id, .. } if id == parameter)
            });
            if !preserves_key {
                return Err(Diagnostic::error("polymorphic recursion is not supported")
                    .with_primary(
                        reference.name.span,
                        "self recursion must preserve the type argument list",
                    )
                    .into());
            }
        }
        let kinds = types::require_type_argument_kinds(
            &signature.parameter_kinds[..explicit.len()],
            &mut explicit,
            reference.name.span,
        )?;
        self.record_kinds(kinds);
        self.check_inferred_generic_call_with_expectations(
            reference,
            &explicit,
            arguments,
            span,
            GenericCallExpectation::Result(expected),
        )
    }

    pub(super) fn check_inferred_generic_call_with_expectations(
        &mut self,
        reference: &resolved::ValueReference,
        explicit: &[Type],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: GenericCallExpectation<'_>,
    ) -> CheckResult<Expression> {
        self.argument_memos.push(ArgumentMemo::new(arguments));
        let result = self
            .check_inferred_generic_call_arguments(reference, explicit, arguments, span, expected);
        self.argument_memos.pop();
        result
    }

    pub(super) fn check_inferred_generic_call_arguments(
        &mut self,
        reference: &resolved::ValueReference,
        explicit: &[Type],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: GenericCallExpectation<'_>,
    ) -> CheckResult<Expression> {
        let mut signature = self.generic_signatures[&reference.id].clone();
        let kinds;
        (signature.parameter_kinds, signature.ty, kinds) = types::instantiate_signature_kinds(
            &signature.parameter_kinds,
            explicit,
            &signature.ty,
            reference.name.span,
        )?;
        self.record_kinds(kinds);
        let mut substitutions = signature
            .parameters
            .iter()
            .zip(explicit)
            .map(|(parameter, argument)| (parameter.id, argument.clone()))
            .collect::<HashMap<_, _>>();
        let instantiated =
            types::substitute_type(&signature.ty, &substitutions, reference.name.span)?;
        let Type::Function { parameter, result } = &instantiated else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(reference.name.span, "this generic value is not a function")
                .into());
        };
        let flexible = parameter_ids(&signature);
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
                    let actual =
                        self.transaction(|probe| probe.check_untyped_argument(arguments, span));
                    if let Ok(actual) = actual {
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
            Err(super::super::CheckFailure::Abrupt(_)) => {
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
            kind: ast::ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }
}
