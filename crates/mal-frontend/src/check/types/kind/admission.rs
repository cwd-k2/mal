//! Resource admission shared by kind inference entry points.

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

pub(super) const MAX_PARAMETERS: usize = 256;
const MAX_FUNCTION_DEPTH: usize = 256;
const MAX_CONSTRUCTION_STEPS: usize = 65_536;

pub(super) fn parameters(count: usize, span: Span) -> Result<(), Diagnostic> {
    if count <= MAX_PARAMETERS {
        return Ok(());
    }
    Err(too_large().with_primary(
        span,
        format!("malc supports at most {MAX_PARAMETERS} type parameters per declaration"),
    ))
}

pub(super) struct Budget {
    span: Span,
    nodes: usize,
}

impl Budget {
    pub(super) fn new(span: Span) -> Self {
        Self { span, nodes: 0 }
    }

    pub(super) fn enter(&mut self, function_depth: usize) -> Result<(), Diagnostic> {
        self.nodes = self.nodes.saturating_add(1);
        if function_depth <= MAX_FUNCTION_DEPTH && self.nodes <= MAX_CONSTRUCTION_STEPS {
            return Ok(());
        }
        Err(too_large().with_primary(
            self.span,
            format!(
                "malc supports kinds up to {MAX_FUNCTION_DEPTH} nested function levels and {MAX_CONSTRUCTION_STEPS} construction steps"
            ),
        ))
    }
}

fn too_large() -> Diagnostic {
    Diagnostic::error("type kind is too large")
}
