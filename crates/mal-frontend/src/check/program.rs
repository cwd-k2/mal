//! The order in which a program's top-level items are checked, and the program they form.

use super::*;

impl Checker {
    pub(super) fn check_program(self, program: &resolved::Program) -> CheckResult<Program> {
        let mut checker = self.prepare_program(program)?;
        let mut items = Vec::with_capacity(program.items.len());
        let mut entry = None;
        for item in &program.items {
            if let Some((item, candidate)) = checker.check_top_item(item)? {
                items.push(item);
                entry = candidate.or(entry);
            }
        }
        Ok(Program {
            items,
            span: program.span,
            entry,
        })
    }

    pub(super) fn check_program_for_editor(
        self,
        program: &resolved::Program,
    ) -> Result<(Program, Option<Diagnostic>), Diagnostic> {
        let mut checker = self.prepare_program(program)?;
        let mut items = Vec::with_capacity(program.items.len());
        let mut entry = None;
        let mut first_diagnostic = None;
        for item in &program.items {
            let checkpoint = checker.clone();
            match checker.check_top_item(item) {
                Ok(Some((item, candidate))) => {
                    items.push(item);
                    entry = candidate.or(entry);
                }
                Ok(None) => {}
                Err(error) => {
                    first_diagnostic.get_or_insert_with(|| diagnostic(error));
                    checker = checkpoint;
                }
            }
        }
        Ok((
            Program {
                items,
                span: program.span,
                entry,
            },
            first_diagnostic,
        ))
    }

    fn prepare_program(mut self, program: &resolved::Program) -> Result<Self, Diagnostic> {
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
        Ok(self)
    }

    fn check_top_item(
        &mut self,
        item: &Node<resolved::TopItem>,
    ) -> CheckResult<Option<(Node<TopItem>, Option<ast::EntryPoint>)>> {
        entry::reject_declared_main(&item.kind)?;
        if matches!(item.kind, resolved::TopItem::GenericTypeAlias { .. }) {
            return Ok(None);
        }
        if let resolved::TopItem::TypeAlias { binding, .. } = &item.kind
            && self.expand_term_id(binding.id, binding.name.span)?.kind() != Kind::Type
        {
            return Ok(None);
        }
        if let resolved::TopItem::OperationFamily {
            binding,
            parameters,
            annotation,
        } = &item.kind
        {
            let family = self.check_operation_family(binding, parameters, annotation, item.span)?;
            return Ok(Some((
                Node::new(TopItem::OperationFamily(Box::new(family)), item.span),
                None,
            )));
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
            return Ok(Some((
                Node::new(
                    TopItem::OperationImplementation(Box::new(implementation)),
                    item.span,
                ),
                None,
            )));
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
            return Ok(Some((
                Node::new(TopItem::GenericBinding(Box::new(binding)), item.span),
                None,
            )));
        }
        let (kind, entry) = match &item.kind {
            resolved::TopItem::TypeAlias { binding, value } => {
                let ty = self.expand_type_id(binding.id, binding.name.span)?;
                let element_aliases = match &ty {
                    Type::Product(_) | Type::Sum(_) => self.aggregate_aliases(value, &ty),
                    _ => Vec::new(),
                };
                (
                    TopItem::TypeAlias {
                        host_memory_access: !binding.name.text.starts_with('_')
                            && interface::is_host_mappable(&ty)
                            && types::is_memory_representable(&ty),
                        binding: binding.clone(),
                        ty,
                        element_aliases,
                    },
                    None,
                )
            }
            resolved::TopItem::OpaqueType { binding, .. } => (
                TopItem::OpaqueType {
                    binding: binding.clone(),
                },
                None,
            ),
            resolved::TopItem::ExternalType { binding } => (
                TopItem::ExternalType {
                    binding: binding.clone(),
                },
                None,
            ),
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
                (
                    TopItem::ExternalOperation {
                        id: *id,
                        binding: binding.clone(),
                        lambda_id: *lambda_id,
                        parameter: signature.parameter.clone(),
                        parameter_alias: signature.parameter_alias.clone(),
                        parameter_aliases: signature.parameter_aliases.clone(),
                        result: signature.result.clone(),
                        result_alias: signature.result_alias.clone(),
                    },
                    None,
                )
            }
            resolved::TopItem::Binding(binding) => {
                let checked = self.check_binding(binding, item.span)?;
                self.check_top_level_initializer(&checked.value)?;
                let entry = entry::entry_point(&checked)?;
                (TopItem::Binding(Box::new(checked)), entry)
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
        Ok(Some((Node::new(kind, item.span), entry)))
    }
}
