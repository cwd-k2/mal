//! Resolution of one top-level item: the names it declares and the bodies and types it contains.

use super::*;

impl Resolver {
    pub(super) fn resolve_top_item(
        &mut self,
        item: &mal_syntax::ast::Node<mal_syntax::ast::TopItem>,
    ) -> Result<mal_syntax::ast::Node<ast::TopItem>, Diagnostic> {
        let kind = match &item.kind {
            mal_syntax::ast::TopItem::TypeAlias { name, value } => ast::TopItem::TypeAlias {
                binding: self.type_binding(name)?,
                value: self.resolve_type(value)?,
            },
            mal_syntax::ast::TopItem::GenericTypeAlias {
                name,
                parameters,
                value,
            } => {
                let bindings = self.push_type_parameters(parameters)?;
                let resolved = self.resolve_type(value);
                self.pop_type_parameters(&bindings);
                ast::TopItem::GenericTypeAlias {
                    binding: self.type_binding(name)?,
                    parameters: bindings,
                    value: resolved?,
                }
            }
            mal_syntax::ast::TopItem::OpaqueType {
                name,
                parameters,
                representation,
            } => {
                let bindings = self.push_type_parameters(parameters)?;
                let resolved = self.resolve_type(representation);
                self.pop_type_parameters(&bindings);
                ast::TopItem::OpaqueType {
                    binding: self.type_binding(name)?,
                    parameters: bindings,
                    representation: resolved?,
                }
            }
            mal_syntax::ast::TopItem::ExternalType { name } => ast::TopItem::ExternalType {
                binding: self.type_binding(name)?,
            },
            mal_syntax::ast::TopItem::ExternalOperation { name, ty } => {
                let external = self
                    .externals
                    .get(&name.text)
                    .expect("external declarations are predeclared");
                ast::TopItem::ExternalOperation {
                    id: external.id,
                    binding: external.binding.clone(),
                    lambda_id: external.lambda_id,
                    ty: self.resolve_type(ty)?,
                }
            }
            mal_syntax::ast::TopItem::Binding(binding) => {
                ast::TopItem::Binding(self.resolve_binding(binding, ValueOwner::TopLevel)?)
            }
            mal_syntax::ast::TopItem::GenericBinding {
                name,
                arguments,
                annotation,
                value,
            } => {
                let existing = self.value_scopes[0].get(&name.text).cloned();
                if value.is_none() {
                    let parameters = generic_parameter_names(arguments)?;
                    let binding = self.declare_value(name, ValueOwner::TopLevel)?;
                    self.operation_families.insert(binding.id);
                    let parameter_bindings = self.push_type_parameters(&parameters)?;
                    let annotation = self.resolve_type(annotation);
                    self.pop_type_parameters(&parameter_bindings);
                    ast::TopItem::OperationFamily {
                        binding,
                        parameters: parameter_bindings,
                        annotation: annotation?,
                    }
                } else if let Some(family) = existing
                    && self.operation_families.contains(&family.id)
                {
                    let parameter_names =
                        key::key_binders(arguments, |name| self.types.contains_key(name));
                    let similar_types = parameter_names
                        .iter()
                        .filter_map(|binder| {
                            key::similar_type(&binder.text, self.types.keys().map(String::as_str))
                                .map(|ty| (binder.text.clone(), ty.to_string()))
                        })
                        .collect();
                    let parameter_bindings = self.push_type_parameters(&parameter_names)?;
                    let resolved = (|| {
                        let arguments = arguments
                            .iter()
                            .map(|argument| self.resolve_type(argument))
                            .collect::<Result<_, _>>()?;
                        let annotation = self.resolve_type(annotation)?;
                        let value = value.as_ref().expect("implementation has an initializer");
                        let value = if let mal_syntax::ast::Expression::Lambda(lambda) = &value.kind
                        {
                            mal_syntax::ast::Node::new(
                                ast::Expression::Lambda(
                                    self.resolve_lambda_with_self(lambda, Some(family.clone()))?,
                                ),
                                value.span,
                            )
                        } else {
                            self.resolve_expression(value)?
                        };
                        Ok((arguments, annotation, value))
                    })();
                    self.pop_type_parameters(&parameter_bindings);
                    let (arguments, annotation, value) = resolved?;
                    ast::TopItem::OperationImplementation {
                        family: ast::ValueReference {
                            id: family.id,
                            name: name.clone(),
                        },
                        parameters: parameter_bindings,
                        arguments,
                        annotation,
                        value,
                        similar_types,
                    }
                } else {
                    let parameters = generic_parameter_names(arguments)?;
                    let binding = self.declare_value(name, ValueOwner::TopLevel)?;
                    let parameter_bindings = self.push_type_parameters(&parameters)?;
                    let resolved = (|| {
                        let annotation = self.resolve_type(annotation)?;
                        let value = value.as_ref().expect("generic binding has an initializer");
                        let value = if let mal_syntax::ast::Expression::Lambda(lambda) = &value.kind
                        {
                            mal_syntax::ast::Node::new(
                                ast::Expression::Lambda(
                                    self.resolve_lambda_with_self(lambda, Some(binding.clone()))?,
                                ),
                                value.span,
                            )
                        } else {
                            self.resolve_expression(value)?
                        };
                        Ok((annotation, value))
                    })();
                    self.pop_type_parameters(&parameter_bindings);
                    let (annotation, value) = resolved?;
                    ast::TopItem::GenericBinding {
                        binding,
                        parameters: parameter_bindings,
                        annotation,
                        value,
                    }
                }
            }
        };
        Ok(mal_syntax::ast::Node::new(kind, item.span))
    }
}

fn generic_parameter_names(
    arguments: &[mal_syntax::ast::Node<mal_syntax::ast::TypeExpression>],
) -> Result<Vec<mal_syntax::ast::Name>, Diagnostic> {
    arguments
        .iter()
        .map(|argument| match &argument.kind {
            mal_syntax::ast::TypeExpression::Named(name) => Ok(name.clone()),
            _ => Err(
                Diagnostic::error("generic declaration parameters must be names").with_primary(
                    argument.span,
                    "this is a type expression, not a parameter name",
                ),
            ),
        })
        .collect()
}
