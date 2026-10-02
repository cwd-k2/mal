//! Declaration-wide validation after kind inference and canonical expansion.

use mal_syntax::diagnostic::Diagnostic;

use super::super::Checker;
use super::canonical::rigid_parameters;
use super::definitions::{GenericAliasDefinition, OpaqueDefinition};
use super::ensure_representable;
use super::expand::ensure_value_type;

impl Checker {
    pub(in crate::check) fn validate_generic_alias(
        &mut self,
        definition: &GenericAliasDefinition,
    ) -> Result<(), Diagnostic> {
        let expanded = self.expand_term_id(definition.binding.id, definition.binding.name.span)?;
        if expanded.kind() == super::Kind::Type {
            ensure_representable(&expanded, definition.value.span)?;
        }
        Ok(())
    }

    pub(in crate::check) fn validate_opaque(
        &mut self,
        definition: &OpaqueDefinition,
    ) -> Result<(), Diagnostic> {
        let kinds = self.declaration_parameter_kinds(
            definition.binding.id,
            definition.parameters.len(),
            definition.binding.name.span,
        )?;
        let substitutions = rigid_parameters(&definition.parameters, kinds);
        let expanded = self.expand_expression(definition.representation.clone(), substitutions)?;
        ensure_value_type(&expanded, definition.representation.span)?;
        ensure_representable(&expanded, definition.representation.span)
    }
}
