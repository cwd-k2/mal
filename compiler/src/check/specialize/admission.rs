use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::resolve::ast::ValueId;

use super::super::ast::Pattern;

const LIMIT: usize = 65_536;

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
