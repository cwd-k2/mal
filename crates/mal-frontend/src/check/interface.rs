use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

use super::{Checker, ast::Type};

#[derive(Clone)]
pub(super) struct ExternalSignature {
    pub(super) parameter: Type,
    pub(super) parameter_alias: Option<String>,
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
            let resolved::TopItem::ExternalOperation {
                id, binding, ty, ..
            } = &item.kind
            else {
                continue;
            };
            let signature = self.expand_type(ty)?;
            let Type::Function { parameter, result } = signature else {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` must have a function type",
                    binding.name.text
                ))
                .with_primary(ty.span, "expected `parameter -> result`"));
            };
            if !is_extern_carrier(&parameter) || !is_extern_carrier(&result) {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` uses an unsupported carrier type",
                    binding.name.text
                ))
                .with_primary(
                    ty.span,
                    "extern carriers must be closed concrete types without functions",
                ));
            }
            let (source_parameter, source_result) = self
                .external_function_parts(ty)
                .expect("expanded external function types retain source components");
            let source_parameter = source_parameter.clone();
            let source_result = source_result.clone();
            let parameter_alias = self.alias_name(&source_parameter);
            let parameter_aliases = self.immediate_aliases(&source_parameter, &parameter);
            let result_alias = self.alias_name(&source_result);
            self.values.insert(
                binding.id,
                Type::Function {
                    parameter: parameter.clone(),
                    result: result.clone(),
                },
            );
            self.external_values.insert(binding.id);
            self.externals.insert(
                *id,
                ExternalSignature {
                    parameter: parameter.as_ref().clone(),
                    parameter_alias,
                    parameter_aliases,
                    result: result.as_ref().clone(),
                    result_alias,
                },
            );
        }
        Ok(())
    }

    fn external_function_parts<'a>(
        &'a self,
        ty: &'a Node<resolved::TypeExpression>,
    ) -> Option<(
        &'a Node<resolved::TypeExpression>,
        &'a Node<resolved::TypeExpression>,
    )> {
        let mut current = ty;
        loop {
            match &current.kind {
                resolved::TypeExpression::Function { parameter, result } => {
                    return Some((parameter, result));
                }
                resolved::TypeExpression::Parenthesized(inner) => current = inner,
                resolved::TypeExpression::Named(reference) => {
                    current = self.aliases.get(&reference.id)?;
                }
                _ => return None,
            }
        }
    }

    pub(super) fn immediate_aliases(
        &mut self,
        source: &Node<resolved::TypeExpression>,
        parameter: &Type,
    ) -> Vec<Option<String>> {
        match parameter {
            Type::Unit => Vec::new(),
            Type::Product(_) => self.aggregate_aliases(source, parameter),
            Type::Sum(_)
            | Type::External { .. }
            | Type::Opaque { .. }
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
            | Type::ByteSize
            | Type::USize
            | Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. }
            | Type::Abstraction { .. }
            | Type::Buffer(_)
            | Type::Function { .. } => {
                vec![self.alias_name(source)]
            }
        }
    }

    pub(super) fn aggregate_aliases(
        &mut self,
        source: &Node<resolved::TypeExpression>,
        ty: &Type,
    ) -> Vec<Option<String>> {
        let elements = match ty {
            Type::Product(elements) | Type::Sum(elements) => elements,
            _ => return Vec::new(),
        };
        self.aggregate_element_sources(source)
            .map(|sources| {
                sources
                    .iter()
                    .map(|source| self.alias_name(source))
                    .collect()
            })
            .filter(|aliases: &Vec<_>| aliases.len() == elements.len())
            .unwrap_or_else(|| vec![None; elements.len()])
    }

    pub(super) fn buffer_element_alias(
        &self,
        source: &Node<resolved::TypeExpression>,
    ) -> Option<String> {
        let mut current = source;
        loop {
            match &current.kind {
                resolved::TypeExpression::Application {
                    constructor,
                    arguments,
                } if constructor.id == resolved::BUFFER_TYPE && arguments.len() == 1 => {
                    return self.alias_name(&arguments[0]);
                }
                resolved::TypeExpression::Parenthesized(inner) => current = inner,
                resolved::TypeExpression::Named(reference) => {
                    current = self.aliases.get(&reference.id)?;
                }
                _ => return None,
            }
        }
    }

    fn aggregate_element_sources(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Option<Vec<Node<resolved::TypeExpression>>> {
        let mut current = ty;
        let mut aliases = Vec::new();
        loop {
            match &current.kind {
                resolved::TypeExpression::Product(elements)
                | resolved::TypeExpression::Sum(elements) => {
                    let elements = elements.clone();
                    for id in aliases {
                        self.aggregate_alias_sources
                            .insert(id, Some(elements.clone()));
                    }
                    return Some(elements);
                }
                resolved::TypeExpression::Parenthesized(inner) => current = inner,
                resolved::TypeExpression::Named(reference) => {
                    if let Some(elements) = self.aggregate_alias_sources.get(&reference.id) {
                        let elements = elements.clone();
                        for id in aliases {
                            self.aggregate_alias_sources.insert(id, elements.clone());
                        }
                        return elements;
                    }
                    aliases.push(reference.id);
                    current = self.aliases.get(&reference.id)?;
                }
                _ => {
                    for id in aliases {
                        self.aggregate_alias_sources.insert(id, None);
                    }
                    return None;
                }
            }
        }
    }

    pub(super) fn alias_name(&self, ty: &Node<resolved::TypeExpression>) -> Option<String> {
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

pub(super) fn is_extern_carrier(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut visited = std::collections::HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Unit
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
            | Type::ByteSize
            | Type::USize
            | Type::External { .. } => {}
            Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
            Type::Buffer(element)
            | Type::Opaque {
                representation: element,
                ..
            } => pending.push(element),
            Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. }
            | Type::Abstraction { .. }
            | Type::Function { .. } => {
                return false;
            }
        }
    }
    true
}
