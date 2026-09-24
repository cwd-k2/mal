use crate::resolve::ast::TypeExpression;
use mal_syntax::ast::Node;

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
