use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, BOOL_TYPE, FLOAT32_TYPE, FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE,
    INT64_TYPE, PTR_TYPE, SYMBOL_TYPE, TypeId, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE,
    UNIT_TYPE,
};
use crate::source::Span;

use super::{Checker, ast::Type};

#[derive(Clone)]
pub(super) struct AliasDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) value: Node<resolved::TypeExpression>,
}

impl Checker {
    pub(super) fn collect_aliases(&mut self, program: &resolved::Program) {
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    self.aliases.insert(
                        binding.id,
                        AliasDefinition {
                            binding: binding.clone(),
                            value: value.clone(),
                        },
                    );
                }
                resolved::TopItem::ExternalType { binding } => {
                    self.external_types.insert(binding.id, binding.clone());
                }
                _ => {}
            }
        }
    }

    pub(super) fn expand_type(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        match &ty.kind {
            resolved::TypeExpression::Named(reference) => {
                self.expand_type_id(reference.id, reference.name.span)
            }
            resolved::TypeExpression::Unit => Ok(Type::Unit),
            resolved::TypeExpression::Parenthesized(inner) => self.expand_type(inner),
            resolved::TypeExpression::Product(elements) => Ok(Type::Product(
                elements
                    .iter()
                    .map(|element| self.expand_type(element))
                    .collect::<Result<_, _>>()?,
            )),
            resolved::TypeExpression::Sum(members) => Ok(Type::Sum(
                members
                    .iter()
                    .map(|member| self.expand_type(member))
                    .collect::<Result<_, _>>()?,
            )),
            resolved::TypeExpression::Function { parameter, result } => Ok(Type::Function {
                parameter: Box::new(self.expand_type(parameter)?),
                result: Box::new(self.expand_type(result)?),
            }),
        }
    }

    pub(super) fn expand_type_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        match id {
            UNIT_TYPE => return Ok(Type::Unit),
            INT8_TYPE => return Ok(Type::Int8),
            INT16_TYPE => return Ok(Type::Int16),
            INT32_TYPE => return Ok(Type::Int32),
            INT64_TYPE => return Ok(Type::Int64),
            UINT8_TYPE => return Ok(Type::UInt8),
            UINT16_TYPE => return Ok(Type::UInt16),
            UINT32_TYPE => return Ok(Type::UInt32),
            UINT64_TYPE => return Ok(Type::UInt64),
            FLOAT32_TYPE => return Ok(Type::Float32),
            FLOAT64_TYPE => return Ok(Type::Float64),
            BOOL_TYPE => return Ok(Type::Sum(vec![Type::Unit, Type::Unit])),
            SYMBOL_TYPE => return Ok(Type::Symbol),
            PTR_TYPE => return Ok(Type::Ptr),
            _ => {}
        }
        if let Some(binding) = self.external_types.get(&id) {
            return Ok(Type::External {
                id,
                name: binding.name.text.clone(),
            });
        }
        if let Some(expanded) = self.expanded_aliases.get(&id) {
            return Ok(expanded.clone());
        }
        if self.expanding.contains(&id) {
            return Err(Diagnostic::error("recursive type alias")
                .with_primary(use_span, "this reference forms an alias cycle"));
        }
        let definition = self
            .aliases
            .get(&id)
            .cloned()
            .expect("resolved type IDs must have a definition");
        self.expanding.push(id);
        let expanded = self.expand_type(&definition.value)?;
        self.expanding.pop();
        self.expanded_aliases.insert(id, expanded.clone());
        Ok(expanded)
    }
}

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit])
}

pub(super) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Box::new(Type::Unit),
        result: Box::new(Type::Unit),
    }
}

pub(super) fn type_name(ty: &Type) -> String {
    match ty {
        Type::Unit => "Unit".into(),
        Type::Int8 => "Int8".into(),
        Type::Int16 => "Int16".into(),
        Type::Int32 => "Int32".into(),
        Type::Int64 => "Int64".into(),
        Type::UInt8 => "UInt8".into(),
        Type::UInt16 => "UInt16".into(),
        Type::UInt32 => "UInt32".into(),
        Type::UInt64 => "UInt64".into(),
        Type::Float32 => "Float32".into(),
        Type::Float64 => "Float64".into(),
        Type::Symbol => "Symbol".into(),
        Type::Ptr => "Ptr".into(),
        Type::External { name, .. } => name.clone(),
        Type::Product(elements) => format!(
            "({})",
            elements
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Sum(members) if *members == vec![Type::Unit, Type::Unit] => "Bool".into(),
        Type::Sum(members) => format!(
            "[{}]",
            members.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        Type::Function { parameter, result } => {
            format!("{} -> {}", type_name(parameter), type_name(result))
        }
    }
}
