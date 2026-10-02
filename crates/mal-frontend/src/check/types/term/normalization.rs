//! The budget that bounds one normalization transaction.

use mal_syntax::{diagnostic::Diagnostic, source::Span};

const MAX_NORMALIZATION_DEPTH: usize = 256;
const MAX_NORMALIZATION_WORK: usize = 65_536;

pub(super) struct Budget {
    remaining: usize,
    span: Span,
}

impl Budget {
    pub(super) fn new(span: Span) -> Self {
        Self {
            remaining: MAX_NORMALIZATION_WORK,
            span,
        }
    }

    pub(super) fn visit(&mut self, depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_NORMALIZATION_DEPTH || self.remaining == 0 {
            return Err(Diagnostic::error("type normalization is too large").with_primary(
                self.span,
                format!(
                    "malc supports at most {MAX_NORMALIZATION_DEPTH} nested term levels and {MAX_NORMALIZATION_WORK} normalization steps"
                ),
            ));
        }
        self.remaining -= 1;
        Ok(())
    }
}
