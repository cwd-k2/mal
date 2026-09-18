use std::collections::HashMap;

use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::ValueId;

use super::super::ast::{Pattern, TopItem, Type};

const LIMIT: usize = 65_536;

pub(super) fn entry_binding<'a>(
    items: impl IntoIterator<Item = &'a Node<TopItem>>,
    program_span: crate::source::Span,
) -> Result<ValueId, Diagnostic> {
    for item in items {
        let TopItem::Binding(binding) = &item.kind else {
            continue;
        };
        if let Pattern::Binding { binding, ty } = &binding.pattern
            && binding.name.text == "main"
        {
            let valid = matches!(
                ty,
                Type::Function { parameter, result }
                    if **result == Type::Int32
                        && (**parameter == Type::Unit
                            || **parameter
                                == Type::Product(vec![Type::USize, Type::Address].into()))
            );
            if !valid {
                return Err(Diagnostic::error("invalid entry point type").with_primary(
                    binding.name.span,
                    "expected `Unit -> Int32` or `(USize, Address) -> Int32`",
                ));
            }
            return Ok(binding.id);
        }
    }
    Err(Diagnostic::error("missing entry point")
        .with_primary(program_span, "the root file must declare `main`"))
}

pub(super) fn collect_pattern_bindings(
    pattern: &Pattern,
    item: usize,
    bindings: &mut HashMap<ValueId, usize>,
) {
    match pattern {
        Pattern::Binding { binding, .. } => {
            bindings.insert(binding.id, item);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_bindings(element, item, bindings);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

pub(super) fn admit_specialization(
    count: usize,
    span: crate::source::Span,
) -> Result<(), Diagnostic> {
    if count < LIMIT {
        return Ok(());
    }
    Err(
        Diagnostic::error("specialization limit exceeded").with_primary(
            span,
            format!("one program may contain at most {LIMIT} specialization nodes"),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, Span};

    #[test]
    fn admits_the_specialization_limit_and_rejects_the_next_node_at_its_span() {
        let span = Span::new(FileId::new(91), 4, 9);
        assert!(admit_specialization(LIMIT - 1, span).is_ok());
        let diagnostic = admit_specialization(LIMIT, span).expect_err("node beyond limit");
        assert_eq!(diagnostic.primary.as_ref().unwrap().span, span);
        assert!(diagnostic.primary.unwrap().message.contains("65536"));
    }
}
