pub(crate) fn generate(
    program: &crate::execution::Program,
) -> Result<crate::c_emit::Output, crate::diagnostic::Diagnostic> {
    crate::c_emit::emit_execution(program)
}

pub(crate) fn emit_header(interface: &crate::core::ast::ProgramInterface) -> String {
    crate::c_emit::emit_header(interface)
}

pub(crate) fn emit_host(
    interface: &crate::core::ast::ProgramInterface,
    header_name: &str,
) -> Result<String, crate::diagnostic::Diagnostic> {
    crate::c_emit::emit_host(interface, header_name)
}
