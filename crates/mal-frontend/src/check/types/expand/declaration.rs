//! Type names resolved to the terms they stand for: substitutions, predefined types, `Buffer`, external types, and
//! aliases and opaque types, whose expansion is cached once finished. A declaration entered again before it finishes
//! forms a cycle and is rejected.

use super::*;
use crate::resolve::ast::{
    ADDRESS_TYPE, BOOL_TYPE, BUFFER_TYPE, BYTE_SIZE_TYPE, FLOAT32_TYPE, FLOAT64_TYPE, INT8_TYPE,
    INT16_TYPE, INT32_TYPE, INT64_TYPE, SYMBOL_TYPE, U_SIZE_TYPE, UINT8_TYPE, UINT16_TYPE,
    UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
};

impl Checker {
    pub(super) fn expand_reference(
        &mut self,
        machine: &mut Machine,
        id: TypeId,
        use_span: Span,
        substitutions: Substitutions,
    ) -> Result<(), Diagnostic> {
        if let Some(ty) = substitutions.get(&id) {
            machine.values.push(ty.clone());
        } else if let Some(ty) = predefined_type(id) {
            machine.values.push(ty);
        } else if id == BUFFER_TYPE {
            let constructor = machine.normalizer.abstraction(
                Kind::Type,
                Type::Buffer(
                    Type::Bound {
                        index: 0,
                        kind: Kind::Type,
                    }
                    .into(),
                ),
            )?;
            machine.values.push(constructor);
        } else if let Some(binding) = self.external_types.get(&id) {
            machine.values.push(Type::External {
                id,
                name: binding.name.text.clone(),
            });
        } else if let Some(expanded) = self.expanded_aliases.get(&id) {
            machine.values.push(expanded.clone());
        } else if let Some(expanded) = self.expanded_generic_aliases.get(&id) {
            let fresh = term::freshen_kind_variables(
                expanded,
                &mut self.next_kind_variable,
                &mut machine.normalizer,
            )?;
            machine.values.push(fresh);
        } else if let Some(definition) = self.generic_aliases.get(&id).cloned() {
            self.enter(
                id,
                use_span,
                "recursive generic type alias",
                "this application forms an alias cycle",
            )?;
            let parameter_kinds =
                self.declaration_parameter_kinds(id, definition.parameters.len(), use_span)?;
            let bound = bound_substitutions(&definition.parameters, &parameter_kinds);
            machine.pending.push(Expansion::AliasAbstraction {
                id,
                parameter_kinds,
            });
            machine
                .pending
                .push(Expansion::Expression(definition.value, Arc::new(bound)));
        } else if let Some(definition) = self.opaque_types.get(&id).cloned() {
            self.enter(
                id,
                use_span,
                "recursive opaque representation",
                "this representation forms a type cycle",
            )?;
            let parameter_kinds =
                self.declaration_parameter_kinds(id, definition.parameters.len(), use_span)?;
            let bound = bound_substitutions(&definition.parameters, &parameter_kinds);
            let arguments = definition
                .parameters
                .iter()
                .map(|parameter| bound[&parameter.id].clone())
                .collect();
            machine.pending.push(Expansion::OpaqueAbstraction {
                id,
                arguments,
                parameter_kinds,
            });
            machine.pending.push(Expansion::Expression(
                definition.representation,
                Arc::new(bound),
            ));
        } else {
            self.enter(
                id,
                use_span,
                "recursive type alias",
                "this reference forms an alias cycle",
            )?;
            let value = self
                .aliases
                .get(&id)
                .expect("resolved type IDs must have a definition");
            machine.pending.push(Expansion::Alias(id));
            machine
                .pending
                .push(Expansion::Expression(value.clone(), substitutions));
        }
        Ok(())
    }

    /// Marks `id` as being expanded, rejecting a reentry as a cycle.
    fn enter(
        &mut self,
        id: TypeId,
        span: Span,
        message: &str,
        label: &str,
    ) -> Result<(), Diagnostic> {
        if self.expanding.insert(id) {
            Ok(())
        } else {
            Err(Diagnostic::error(message).with_primary(span, label))
        }
    }

    pub(super) fn finish_alias(&mut self, machine: &Machine, id: TypeId) {
        let expanded = machine
            .values
            .last()
            .expect("alias expansion must produce a type");
        self.expanded_aliases.insert(id, expanded.clone());
        assert!(self.expanding.remove(&id));
    }

    pub(super) fn finish_generic_alias(
        &mut self,
        machine: &mut Machine,
        id: TypeId,
        parameter_kinds: Vec<Kind>,
    ) -> Result<(), Diagnostic> {
        let body = machine.pop_value("alias expansion produces one term");
        let abstraction = machine.abstract_over(parameter_kinds, body)?;
        self.expanded_generic_aliases
            .insert(id, abstraction.clone());
        machine.values.push(abstraction);
        assert!(self.expanding.remove(&id));
        Ok(())
    }

    pub(super) fn finish_opaque(
        &mut self,
        machine: &mut Machine,
        id: TypeId,
        arguments: Vec<Type>,
        parameter_kinds: Vec<Kind>,
    ) -> Result<(), Diagnostic> {
        let representation = machine.pop_value("opaque representation expansion produces a type");
        let definition = self
            .opaque_types
            .get(&id)
            .expect("opaque type exists when expansion finishes");
        let opaque = Type::Opaque {
            id,
            name: definition.binding.name.text.clone().into(),
            arguments: arguments.into(),
            representation: representation.into(),
            declaration_file: definition.binding.name.span.file(),
        };
        let abstraction = machine.abstract_over(parameter_kinds, opaque)?;
        machine.values.push(abstraction);
        assert!(self.expanding.remove(&id));
        Ok(())
    }
}

/// The bound variables a declaration's parameters stand for in its body, innermost last.
fn bound_substitutions(
    parameters: &[resolved::TypeBinding],
    kinds: &[Kind],
) -> HashMap<TypeId, Type> {
    let count = parameters.len();
    parameters
        .iter()
        .zip(kinds)
        .enumerate()
        .map(|(index, (parameter, kind))| {
            (
                parameter.id,
                Type::Bound {
                    index: count - index - 1,
                    kind: kind.clone(),
                },
            )
        })
        .collect()
}

pub(in crate::check::types) fn predefined_type(id: TypeId) -> Option<Type> {
    Some(match id {
        UNIT_TYPE => Type::Unit,
        INT8_TYPE => Type::Int8,
        INT16_TYPE => Type::Int16,
        INT32_TYPE => Type::Int32,
        INT64_TYPE => Type::Int64,
        UINT8_TYPE => Type::UInt8,
        UINT16_TYPE => Type::UInt16,
        UINT32_TYPE => Type::UInt32,
        UINT64_TYPE => Type::UInt64,
        FLOAT32_TYPE => Type::Float32,
        FLOAT64_TYPE => Type::Float64,
        BOOL_TYPE => Type::Sum(vec![Type::Unit, Type::Unit].into()),
        SYMBOL_TYPE => Type::Symbol,
        ADDRESS_TYPE => Type::Address,
        BYTE_SIZE_TYPE => Type::ByteSize,
        U_SIZE_TYPE => Type::USize,
        _ => return None,
    })
}
