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
        } => applied_type_name(
            substitutions
                .get(&constructor.id)
                .map(String::as_str)
                .unwrap_or(&constructor.name.text),
            arguments
                .iter()
                .map(|argument| type_name_with_substitutions(argument, substitutions)),
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
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute(argument, substitutions))
                .collect();
            if let Some(replacement) = substitutions.get(&constructor.id) {
                return apply(replacement, arguments, ty.span);
            }
            TypeExpression::Application {
                constructor: constructor.clone(),
                arguments,
            }
        }
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

fn applied_type_name(constructor: &str, arguments: impl Iterator<Item = String>) -> String {
    let arguments = arguments.collect::<Vec<_>>().join(", ");
    if let Some(partial) = constructor.strip_suffix('>') {
        format!("{partial}, {arguments}>")
    } else {
        format!("{constructor}<{arguments}>")
    }
}

fn apply(
    constructor: &Node<TypeExpression>,
    mut arguments: Vec<Node<TypeExpression>>,
    span: mal_syntax::source::Span,
) -> Node<TypeExpression> {
    match &constructor.kind {
        TypeExpression::Named(reference) => Node::new(
            TypeExpression::Application {
                constructor: reference.clone(),
                arguments,
            },
            span,
        ),
        TypeExpression::Application {
            constructor,
            arguments: partial,
        } => {
            let mut combined = partial.clone();
            combined.append(&mut arguments);
            Node::new(
                TypeExpression::Application {
                    constructor: constructor.clone(),
                    arguments: combined,
                },
                span,
            )
        }
        TypeExpression::Parenthesized(inner) => apply(inner, arguments, span),
        _ => unreachable!("checked constructor substitution must remain applicable"),
    }
}
