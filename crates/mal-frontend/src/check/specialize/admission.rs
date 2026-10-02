//! The limit on how many instances one program may specialize.

use mal_syntax::diagnostic::Diagnostic;

const LIMIT: usize = 65_536;

pub(super) fn admit_specialization(
    count: usize,
    span: mal_syntax::source::Span,
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
    use mal_syntax::source::{FileId, Span};

    #[test]
    fn admits_the_specialization_limit_and_rejects_the_next_node_at_its_span() {
        let span = Span::new(FileId::new(91), 4, 9);
        assert!(admit_specialization(LIMIT - 1, span).is_ok());
        let diagnostic = admit_specialization(LIMIT, span).expect_err("node beyond limit");
        assert_eq!(diagnostic.primary.as_ref().unwrap().span, span);
        assert!(diagnostic.primary.unwrap().message.contains("65536"));
    }
}
