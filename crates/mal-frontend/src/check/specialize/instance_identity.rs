//! Identity of the binders inside one generic instance.
//!
//! An instance is a copy of the generic body, and later stages key their facts by `ValueId` across the
//! whole program. Every binder in a copy therefore receives an identity that no other instance shares, and
//! the references the copy makes to those binders follow.

use crate::resolve::ast::{LambdaId, ValueBinding, ValueId, ValueOwner};
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::Pattern;
use super::Specializer;

impl Specializer {
    /// Starts the identity map of a new instance; the previous instance's binders are never referenced again.
    pub(super) fn begin_instance_identities(&mut self) {
        self.value_renames.clear();
        self.lambda_renames.clear();
    }

    pub(super) fn rename_lambda(&mut self, old: LambdaId, new: LambdaId) {
        self.lambda_renames.insert(old, new);
    }

    /// Allocates the instance's identity for a binder, or returns the one it already has.
    pub(super) fn rename_value(&mut self, id: ValueId, span: Span) -> Result<ValueId, Diagnostic> {
        if let Some(renamed) = self.value_renames.get(&id) {
            return Ok(*renamed);
        }
        let renamed = self.fresh_value(span)?;
        self.value_renames.insert(id, renamed);
        Ok(renamed)
    }

    pub(super) fn rename_binding(&mut self, binding: &mut ValueBinding) -> Result<(), Diagnostic> {
        binding.id = self.rename_value(binding.id, binding.name.span)?;
        if let ValueOwner::Lambda(lambda) | ValueOwner::Result(lambda) = &mut binding.owner
            && let Some(renamed) = self.lambda_renames.get(lambda)
        {
            *lambda = *renamed;
        }
        Ok(())
    }

    pub(super) fn rename_pattern(&mut self, pattern: &mut Pattern) -> Result<(), Diagnostic> {
        match pattern {
            Pattern::Binding { binding, .. } => self.rename_binding(binding),
            Pattern::Product { elements, .. } => {
                for element in elements {
                    self.rename_pattern(element)?;
                }
                Ok(())
            }
            Pattern::Wildcard { .. } => Ok(()),
        }
    }

    /// Follows a reference to a binder of this instance; other references name bindings shared by all instances.
    pub(super) fn renamed_reference(&self, id: ValueId) -> Option<ValueId> {
        self.value_renames.get(&id).copied()
    }
}
