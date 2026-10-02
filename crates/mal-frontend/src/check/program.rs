//! The order in which a program's top-level items are checked, and the program they form.

use super::*;

impl Checker {
    pub(super) fn check_program(mut self, program: &resolved::Program) -> CheckResult<Program> {
        self.kinds = Kinds::infer(program)?;
        self.collect_aliases(program);
        // Source order keeps the reported error the same from run to run when several aliases are invalid.
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, .. } => {
                    self.expand_term_id(binding.id, binding.name.span)?;
                }
                resolved::TopItem::GenericTypeAlias { binding, .. } => {
                    let definition = self.generic_aliases[&binding.id].clone();
                    self.validate_generic_alias(&definition)?;
                }
                resolved::TopItem::OpaqueType { binding, .. } => {
                    let definition = self.opaque_types[&binding.id].clone();
                    self.validate_opaque(&definition)?;
                    self.aggregate_alias_sources.insert(binding.id, None);
                }
                _ => {}
            }
        }
        self.collect_external_signatures(program)?;

        let mut items = Vec::with_capacity(program.items.len());
        let mut entry = None;
        for item in &program.items {
            entry::reject_declared_main(&item.kind)?;
            if matches!(item.kind, resolved::TopItem::GenericTypeAlias { .. }) {
                continue;
            }
            if let resolved::TopItem::TypeAlias { binding, .. } = &item.kind
                && self.expand_term_id(binding.id, binding.name.span)?.kind() != Kind::Type
            {
                continue;
            }
            if let resolved::TopItem::OperationFamily {
                binding,
                parameters,
                annotation,
            } = &item.kind
            {
                let family =
                    self.check_operation_family(binding, parameters, annotation, item.span)?;
                items.push(Node::new(
                    TopItem::OperationFamily(Box::new(family)),
                    item.span,
                ));
                continue;
            }
            if let resolved::TopItem::OperationImplementation {
                family,
                parameters,
                arguments,
                annotation,
                value,
                similar_types,
            } = &item.kind
            {
                let implementation = self
                    .check_operation_implementation(
                        family, parameters, arguments, annotation, value, item.span,
                    )
                    .map_err(|failure| suggest_types(failure, similar_types))?;
                items.push(Node::new(
                    TopItem::OperationImplementation(Box::new(implementation)),
                    item.span,
                ));
                continue;
            }
            if let resolved::TopItem::GenericBinding {
                binding,
                parameters,
                annotation,
                value,
            } = &item.kind
            {
                let binding =
                    self.check_generic_binding(binding, parameters, annotation, value, item.span)?;
                items.push(Node::new(
                    TopItem::GenericBinding(Box::new(binding)),
                    item.span,
                ));
                continue;
            }
            let kind = match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    let ty = self.expand_type_id(binding.id, binding.name.span)?;
                    let element_aliases = match &ty {
                        Type::Product(_) | Type::Sum(_) => self.aggregate_aliases(value, &ty),
                        _ => Vec::new(),
                    };
                    TopItem::TypeAlias {
                        host_memory_access: !binding.name.text.starts_with('_')
                            && interface::is_host_mappable(&ty)
                            && types::is_memory_representable(&ty),
                        binding: binding.clone(),
                        ty,
                        element_aliases,
                    }
                }
                resolved::TopItem::OpaqueType { binding, .. } => TopItem::OpaqueType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalType { binding } => TopItem::ExternalType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalOperation {
                    id,
                    binding,
                    lambda_id,
                    ..
                } => {
                    let signature = self
                        .externals
                        .get(id)
                        .expect("external signatures are collected before checking values");
                    TopItem::ExternalOperation {
                        id: *id,
                        binding: binding.clone(),
                        lambda_id: *lambda_id,
                        parameter: signature.parameter.clone(),
                        parameter_alias: signature.parameter_alias.clone(),
                        parameter_aliases: signature.parameter_aliases.clone(),
                        result: signature.result.clone(),
                        result_alias: signature.result_alias.clone(),
                    }
                }
                resolved::TopItem::Binding(binding) => {
                    let checked = self.check_binding(binding, item.span)?;
                    self.check_top_level_initializer(&checked.value)?;
                    if let Some(candidate) = entry::entry_point(&checked)? {
                        entry = Some(candidate);
                    }
                    TopItem::Binding(Box::new(checked))
                }
                resolved::TopItem::GenericBinding { .. } => {
                    unreachable!("generic bindings are checked before monomorphic item emission")
                }
                resolved::TopItem::GenericTypeAlias { .. } => {
                    unreachable!("generic aliases are omitted before checked program emission")
                }
                resolved::TopItem::OperationFamily { .. }
                | resolved::TopItem::OperationImplementation { .. } => {
                    unreachable!("operation items are checked before ordinary item emission")
                }
            };
            items.push(Node::new(kind, item.span));
        }
        Ok(Program {
            items,
            span: program.span,
            entry,
        })
    }
}
