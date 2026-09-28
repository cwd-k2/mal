use crate::resolve::ast::TypeExpression;
use crate::resolve::ast::TypeId;
use mal_syntax::ast::Node;
use std::collections::HashMap;

pub(super) fn type_name(ty: &Node<TypeExpression>) -> String {
    match &ty.kind {
        TypeExpression::Named(reference) => reference.name.text.clone(),
        TypeExpression::Application {
            constructor,
            arguments,
        } => format!(
            "{}<{}>",
            constructor.name.text,
            arguments
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpression::Unit => "Unit".into(),
        TypeExpression::Parenthesized(inner) => format!("({})", type_name(inner)),
        TypeExpression::Product(elements) => format!(
            "({})",
            elements
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpression::Sum(members) => format!(
            "[{}]",
            members.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        TypeExpression::Function { parameter, result } => {
            format!("{} -> {}", type_name(parameter), type_name(result))
        }
    }
}

pub(super) fn type_name_with_substitutions(
    ty: &Node<TypeExpression>,
    substitutions: &HashMap<TypeId, String>,
) -> String {
    match &ty.kind {
        TypeExpression::Named(reference) => substitutions
            .get(&reference.id)
            .cloned()
            .unwrap_or_else(|| reference.name.text.clone()),
        TypeExpression::Application {
            constructor,
            arguments,
        } => format!(
            "{}<{}>",
            constructor.name.text,
            arguments
                .iter()
                .map(|argument| type_name_with_substitutions(argument, substitutions))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpression::Unit => "Unit".into(),
        TypeExpression::Parenthesized(inner) => {
            format!("({})", type_name_with_substitutions(inner, substitutions))
        }
        TypeExpression::Product(elements) => format!(
            "({})",
            elements
                .iter()
                .map(|element| type_name_with_substitutions(element, substitutions))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpression::Sum(members) => format!(
            "[{}]",
            members
                .iter()
                .map(|member| type_name_with_substitutions(member, substitutions))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpression::Function { parameter, result } => format!(
            "{} -> {}",
            type_name_with_substitutions(parameter, substitutions),
            type_name_with_substitutions(result, substitutions)
        ),
    }
}

pub(super) fn substitute(
    ty: &Node<TypeExpression>,
    substitutions: &HashMap<TypeId, Node<TypeExpression>>,
) -> Node<TypeExpression> {
    let kind = match &ty.kind {
        TypeExpression::Named(reference) => {
            if let Some(replacement) = substitutions.get(&reference.id) {
                return replacement.clone();
            }
            TypeExpression::Named(reference.clone())
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } => TypeExpression::Application {
            constructor: constructor.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute(argument, substitutions))
                .collect(),
        },
        TypeExpression::Unit => TypeExpression::Unit,
        TypeExpression::Parenthesized(inner) => {
            TypeExpression::Parenthesized(Box::new(substitute(inner, substitutions)))
        }
        TypeExpression::Product(elements) => TypeExpression::Product(
            elements
                .iter()
                .map(|element| substitute(element, substitutions))
                .collect(),
        ),
        TypeExpression::Sum(members) => TypeExpression::Sum(
            members
                .iter()
                .map(|member| substitute(member, substitutions))
                .collect(),
        ),
        TypeExpression::Function { parameter, result } => TypeExpression::Function {
            parameter: Box::new(substitute(parameter, substitutions)),
            result: Box::new(substitute(result, substitutions)),
        },
    };
    Node::new(kind, ty.span)
}
