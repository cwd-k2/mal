//! Expansion of one requested instance: its requirement checks, then the substituted body as a monomorphic binding.

use crate::resolve::ast::{ValueBinding, ValueId};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::*;
use super::super::types::term::check_kind_requirements;
use super::super::types::{
    runtime_type, satisfies_storable_requirement, substitute_type, type_name,
};
use super::{Specializer, Substitutions};

impl Specializer {
    pub(super) fn expand_generic(
        &mut self,
        generic: ValueId,
        arguments: Vec<Type>,
        binding: ValueBinding,
    ) -> Result<(), Diagnostic> {
        let definition = self
            .definitions
            .get(&generic)
            .expect("checked generic reference has a definition");
        check_kind_requirements(
            &definition.parameter_kinds,
            &arguments,
            &definition.kinds,
            binding.name.span,
        )?;
        let substitutions = definition
            .parameters
            .iter()
            .map(|parameter| parameter.id)
            .zip(arguments)
            .collect::<Substitutions>();
        let (value, ty, span) = (
            definition.value.clone(),
            definition.ty.clone(),
            definition.span,
        );
        self.emit(binding, value, &ty, span, &substitutions, generic)
    }

    pub(super) fn expand_implementation(
        &mut self,
        implementation: usize,
        substitutions: Substitutions,
        binding: ValueBinding,
    ) -> Result<(), Diagnostic> {
        let implementation = &self.implementations[implementation];
        let pattern_arguments = implementation
            .parameters
            .iter()
            .map(|parameter| substitutions[&parameter.id].clone())
            .collect::<Vec<_>>();
        check_kind_requirements(
            &implementation.parameter_kinds,
            &pattern_arguments,
            &implementation.kinds,
            binding.name.span,
        )?;
        check_storable_requirements(
            &implementation.requirements,
            &substitutions,
            binding.name.span,
        )?;
        let (value, ty, span, family) = (
            implementation.value.clone(),
            implementation.ty.clone(),
            implementation.span,
            implementation.family.id,
        );
        self.emit(binding, value, &ty, span, &substitutions, family)
    }

    /// Substitutes the body under fresh binder identities and records it as the instance binding. `origin` is the
    /// generic or family whose self references the body redirects to the instance.
    fn emit(
        &mut self,
        binding: ValueBinding,
        mut value: Expression,
        ty: &Type,
        span: Span,
        substitutions: &Substitutions,
        origin: ValueId,
    ) -> Result<(), Diagnostic> {
        let ty = runtime_type(&substitute_type(ty, substitutions, span)?);
        let instance = binding.id;
        self.begin_instance_identities();
        self.expression(&mut value, substitutions, Some((origin, instance)))?;
        self.specializations.push(Node::new(
            TopItem::Binding(Box::new(Binding {
                pattern: Pattern::Binding {
                    binding,
                    ty: ty.clone(),
                },
                annotation: Some(ty),
                value,
                span,
            })),
            span,
        ));
        Ok(())
    }
}

/// Checks the `Storable` atoms that a selected implementation's body assumed. A family signature cannot
/// require `Storable` inside an open application such as `F<A>`, so a key that expands to a Buffer can need
/// more than the callers of the family were asked to supply.
fn check_storable_requirements(
    requirements: &[Type],
    substitutions: &Substitutions,
    span: Span,
) -> Result<(), Diagnostic> {
    for requirement in requirements {
        let required = substitute_type(requirement, substitutions, span)?;
        if !satisfies_storable_requirement(&required, &[]) {
            return Err(Diagnostic::error("operation instance violates a Storable requirement")
                .with_primary(
                    span,
                    format!(
                        "the selected implementation needs `{}` to be storable",
                        type_name(&required)
                    ),
                )
                .with_note(
                    "a family signature cannot require Storable inside an open application, so the implementation's own requirements are checked when it is selected",
                ));
        }
    }
    Ok(())
}
