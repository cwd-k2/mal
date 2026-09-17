use crate::core::ast::ProgramInterface;
use crate::diagnostic::Diagnostic;

mod header;
mod host_signature;
pub(in crate::backend) mod syntax;
mod types;

use self::types::{HostTypes, TypeRegistry};

pub(crate) const GENERATED_HEADER_NAME: &str = "program.mal.h";

pub(crate) fn emit_header(interface: &ProgramInterface) -> String {
    emit_header_for_target(
        interface,
        crate::backend::llvm::TargetLayout::natural(
            std::mem::size_of::<*const ()>(),
            std::mem::size_of::<usize>(),
        )
        .expect("host pointer and size widths are supported"),
    )
}

pub(crate) fn emit_header_for_target(
    interface: &ProgramInterface,
    target: crate::backend::llvm::TargetLayout,
) -> String {
    let mut types = TypeRegistry::default();
    let host = HostTypes::collect(interface, &mut types);
    header::emit(interface, &types, &host, target)
}

pub(crate) fn emit_host(
    interface: &ProgramInterface,
    header_name: &str,
) -> Result<String, Diagnostic> {
    if !is_valid_header_name(header_name) {
        return Err(Diagnostic::error(
            "generated host header name is not valid in a quoted C include",
        ));
    }
    let mut types = TypeRegistry::default();
    let _host = HostTypes::collect(interface, &mut types);
    Ok(header::emit_host(interface, &types, header_name))
}

pub(crate) struct RawHostTypes {
    types: TypeRegistry,
}

impl RawHostTypes {
    pub(crate) fn new(interface: &ProgramInterface) -> Self {
        let mut types = TypeRegistry::default();
        let _host = HostTypes::collect(interface, &mut types);
        Self { types }
    }

    pub(crate) fn c_type(&self, ty: &crate::check::ast::Type) -> String {
        self.types.c_type(ty).to_string()
    }
}

pub(crate) fn is_valid_header_name(header_name: &str) -> bool {
    syntax::Directive::is_valid_quoted_include(header_name)
}
