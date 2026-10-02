//! Operation-family declaration, implementation coherence, and termination checking.

use crate::resolve::ast as resolved;
use mal_syntax::{ast::Node, source::Span};

use super::{CheckResult, Checker, GenericSignature, ast, types};

mod key;
mod overlap;
mod pattern;
mod spelling;
mod termination;

pub(super) use pattern::contains_parameter;
pub(super) use spelling::suggest_types;
use termination::require_decreasing;

impl Checker {
    pub(super) fn check_operation_family(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        span: Span,
    ) -> CheckResult<ast::OperationFamily> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let signature_parameter_kinds = parameter_kinds.clone();
        let substitutions = types::rigid_parameters(parameters, parameter_kinds);
        self.with_body(substitutions, |checker| {
            let ty = checker.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            checker.generic_signatures.insert(
                binding.id,
                GenericSignature {
                    parameters: parameters.to_vec(),
                    parameter_kinds: signature_parameter_kinds,
                    ty: ty.clone(),
                    requirements,
                    operations: Vec::new(),
                },
            );
            checker.operation_families.insert(binding.id);
            Ok(ast::OperationFamily {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                span,
            })
        })
    }

    pub(super) fn check_operation_implementation(
        &mut self,
        family: &resolved::ValueReference,
        parameters: &[resolved::TypeBinding],
        arguments: &[Node<resolved::TypeExpression>],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::OperationImplementation> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let substitutions = types::rigid_parameters(parameters, parameter_kinds.iter().cloned());
        self.with_body(substitutions, |checker| {
            let mut arguments = arguments
                .iter()
                .map(|argument| checker.expand_type_term(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let signature = checker
                .generic_signatures
                .get(&family.id)
                .expect("an implementation refers to a checked operation family");
            let key_kinds = types::require_type_argument_kinds(
                &signature.parameter_kinds,
                &mut arguments,
                family.name.span,
            )?;
            let family_kinds = signature.parameter_kinds.clone();
            checker.check_implementation_key(family, parameters, &family_kinds, &arguments)?;
            let expected = checker
                .check_generic_reference(family, arguments.clone(), family.name.span)?
                .ty;
            let declared = checker.expand_type(annotation)?;
            checker.require_type(&declared, &expected, annotation.span)?;
            // Like a generic binding, the body may assume what its signature makes well formed.
            let requirements = types::storable_requirements(&expected);
            checker.active_requirements = requirements.clone();
            checker.active_generic = (!parameters.is_empty()).then_some((
                family.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            // Kind equations from the key are kept with those the body records, all checked at specialization.
            checker.active_kinds = key_kinds;
            let checked_value = checker.check_expression(value, Some(&expected))?;
            checker.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut checker.active_operations);
            if !parameters.is_empty() {
                require_decreasing(family, &arguments, &operations)?;
            }
            let kinds = std::mem::take(&mut checker.active_kinds);
            checker.operation_keys.push((family.id, arguments.clone()));
            Ok(ast::OperationImplementation {
                family: family.clone(),
                parameters: parameters.to_vec(),
                arguments,
                ty: expected,
                value: checked_value,
                operations,
                parameter_kinds,
                kinds,
                requirements,
                span,
            })
        })
    }
}
