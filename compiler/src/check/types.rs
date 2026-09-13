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
        self.expand([Expansion::Expression(ty.clone())])
    }

    pub(super) fn expand_type_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Reference(id, use_span)])
    }

    fn expand(&mut self, initial: impl IntoIterator<Item = Expansion>) -> Result<Type, Diagnostic> {
        let mut pending = initial.into_iter().collect::<Vec<_>>();
        let mut values = Vec::new();
        while let Some(expansion) = pending.pop() {
            match expansion {
                Expansion::Expression(expression) => match expression.kind {
                    resolved::TypeExpression::Named(reference) => {
                        pending.push(Expansion::Reference(reference.id, reference.name.span));
                    }
                    resolved::TypeExpression::Unit => values.push(Type::Unit),
                    resolved::TypeExpression::Parenthesized(inner) => {
                        pending.push(Expansion::Expression(*inner));
                    }
                    resolved::TypeExpression::Product(elements) => {
                        pending.push(Expansion::Product(elements.len()));
                        pending.extend(elements.into_iter().rev().map(Expansion::Expression));
                    }
                    resolved::TypeExpression::Sum(members) => {
                        pending.push(Expansion::Sum(members.len()));
                        pending.extend(members.into_iter().rev().map(Expansion::Expression));
                    }
                    resolved::TypeExpression::Function { parameter, result } => {
                        pending.push(Expansion::Function);
                        pending.push(Expansion::Expression(*result));
                        pending.push(Expansion::Expression(*parameter));
                    }
                },
                Expansion::Reference(id, use_span) => {
                    if let Some(ty) = predefined_type(id) {
                        values.push(ty);
                    } else if let Some(binding) = self.external_types.get(&id) {
                        values.push(Type::External {
                            id,
                            name: binding.name.text.clone(),
                        });
                    } else if let Some(expanded) = self.expanded_aliases.get(&id) {
                        values.push(expanded.clone());
                    } else {
                        if !self.expanding.insert(id) {
                            return Err(Diagnostic::error("recursive type alias")
                                .with_primary(use_span, "this reference forms an alias cycle"));
                        }
                        let definition = self
                            .aliases
                            .get(&id)
                            .expect("resolved type IDs must have a definition");
                        pending.push(Expansion::Alias(id));
                        pending.push(Expansion::Expression(definition.value.clone()));
                    }
                }
                Expansion::Alias(id) => {
                    let expanded = values.last().expect("alias expansion must produce a type");
                    self.expanded_aliases.insert(id, expanded.clone());
                    assert!(self.expanding.remove(&id));
                }
                Expansion::Product(length) => {
                    let elements = take_last(&mut values, length);
                    values.push(Type::Product(elements.into()));
                }
                Expansion::Sum(length) => {
                    let members = take_last(&mut values, length);
                    values.push(Type::Sum(members.into()));
                }
                Expansion::Function => {
                    let [parameter, result] = take_last(&mut values, 2).try_into().unwrap();
                    values.push(Type::Function {
                        parameter: parameter.into(),
                        result: result.into(),
                    });
                }
            }
        }
        let [value] = values
            .try_into()
            .expect("one expansion must produce one type");
        Ok(value)
    }
}

enum Expansion {
    Expression(Node<resolved::TypeExpression>),
    Reference(TypeId, Span),
    Alias(TypeId),
    Product(usize),
    Sum(usize),
    Function,
}

fn predefined_type(id: TypeId) -> Option<Type> {
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
        PTR_TYPE => Type::Ptr,
        _ => return None,
    })
}

fn take_last(values: &mut Vec<Type>, length: usize) -> Vec<Type> {
    values.split_off(
        values
            .len()
            .checked_sub(length)
            .expect("composite expansion must have all children"),
    )
}

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit].into())
}

pub(super) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
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
        Type::Sum(members) if members.as_ref() == [Type::Unit, Type::Unit] => "Bool".into(),
        Type::Sum(members) => format!(
            "[{}]",
            members.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        Type::Function { parameter, result } => {
            format!("{} -> {}", type_name(parameter), type_name(result))
        }
    }
}
