use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;

use super::{Checker, ast::Type};

#[derive(Clone)]
pub(super) struct ExternalSignature {
    pub(super) parameter: Type,
    pub(super) parameter_aliases: Vec<Option<String>>,
    pub(super) result: Type,
    pub(super) result_alias: Option<String>,
}

impl Checker {
    pub(super) fn collect_external_signatures(
        &mut self,
        program: &resolved::Program,
    ) -> Result<(), Diagnostic> {
        for item in &program.items {
            let resolved::TopItem::ExternalOperation { id, name, ty } = &item.kind else {
                continue;
            };
            let signature = self.expand_type(ty)?;
            let Type::Function { parameter, result } = signature else {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` must have a function type",
                    name.text
                ))
                .with_primary(ty.span, "expected `parameter -> result`"));
            };
            if contains_function(&parameter) || contains_function(&result) {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` uses a function value",
                    name.text
                ))
                .with_primary(ty.span, "function types cannot cross the extern boundary"));
            }
            let (source_parameter, source_result) = self
                .external_function_parts(ty)
                .expect("expanded external function types retain source components");
            let parameter_aliases = self.parameter_aliases(source_parameter, &parameter);
            let result_alias = self.alias_name(source_result);
            self.externals.insert(
                *id,
                ExternalSignature {
                    parameter: *parameter,
                    parameter_aliases,
                    result: *result,
                    result_alias,
                },
            );
        }
        Ok(())
    }

    pub(super) fn external_signature(
        &self,
        id: resolved::ExternalOperationId,
    ) -> ExternalSignature {
        self.externals
            .get(&id)
            .cloned()
            .expect("resolved external calls have a collected signature")
    }

    fn external_function_parts<'a>(
        &'a self,
        ty: &'a Node<resolved::TypeExpression>,
    ) -> Option<(
        &'a Node<resolved::TypeExpression>,
        &'a Node<resolved::TypeExpression>,
    )> {
        match &ty.kind {
            resolved::TypeExpression::Function { parameter, result } => Some((parameter, result)),
            resolved::TypeExpression::Parenthesized(inner) => self.external_function_parts(inner),
            resolved::TypeExpression::Named(reference) => self
                .aliases
                .get(&reference.id)
                .and_then(|definition| self.external_function_parts(&definition.value)),
            _ => None,
        }
    }

    fn parameter_aliases(
        &self,
        source: &Node<resolved::TypeExpression>,
        parameter: &Type,
    ) -> Vec<Option<String>> {
        match parameter {
            Type::Unit => Vec::new(),
            Type::Product(elements) => self
                .product_element_sources(source)
                .map(|sources| {
                    sources
                        .iter()
                        .map(|source| self.alias_name(source))
                        .collect()
                })
                .filter(|aliases: &Vec<_>| aliases.len() == elements.len())
                .unwrap_or_else(|| vec![None; elements.len()]),
            _ => vec![self.alias_name(source)],
        }
    }

    fn product_element_sources<'a>(
        &'a self,
        ty: &'a Node<resolved::TypeExpression>,
    ) -> Option<&'a [Node<resolved::TypeExpression>]> {
        match &ty.kind {
            resolved::TypeExpression::Product(elements) => Some(elements),
            resolved::TypeExpression::Parenthesized(inner) => self.product_element_sources(inner),
            resolved::TypeExpression::Named(reference) => self
                .aliases
                .get(&reference.id)
                .and_then(|definition| self.product_element_sources(&definition.value)),
            _ => None,
        }
    }

    fn alias_name(&self, ty: &Node<resolved::TypeExpression>) -> Option<String> {
        match &ty.kind {
            resolved::TypeExpression::Named(reference)
                if self.aliases.contains_key(&reference.id) =>
            {
                Some(reference.name.text.clone())
            }
            resolved::TypeExpression::Parenthesized(inner) => self.alias_name(inner),
            _ => None,
        }
    }
}

fn contains_function(ty: &Type) -> bool {
    match ty {
        Type::Function { .. } => true,
        Type::Product(elements) => elements.iter().any(contains_function),
        Type::Sum(members) => members.iter().any(contains_function),
        Type::External { .. }
        | Type::Unit
        | Type::Int8
        | Type::Int16
        | Type::Int32
        | Type::Int64
        | Type::UInt8
        | Type::UInt16
        | Type::UInt32
        | Type::UInt64
        | Type::Float32
        | Type::Float64
        | Type::Symbol
        | Type::Ptr => false,
    }
}
