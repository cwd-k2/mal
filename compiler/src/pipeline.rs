use crate::diagnostic::Diagnostic;
use crate::source::SourceFile;

pub struct Analysis {
    pub resolved: crate::resolve::ast::Program,
    pub checked: crate::check::ast::Program,
}

pub fn analyze(source: &SourceFile) -> Result<Analysis, Diagnostic> {
    let parsed = crate::parser::parse(source)?;
    let resolved = crate::resolve::resolve(&parsed)?;
    let checked = crate::check::check(&resolved)?;
    Ok(Analysis { resolved, checked })
}

pub fn check(source: &SourceFile) -> Result<crate::check::ast::Program, Diagnostic> {
    analyze(source).map(|analysis| analysis.checked)
}

pub fn emit_c(source: &SourceFile) -> Result<crate::c_emit::Output, Diagnostic> {
    let checked = check(source)?;
    let core = crate::core::lower(&checked);
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::c_emit::emit(&closure)
}
