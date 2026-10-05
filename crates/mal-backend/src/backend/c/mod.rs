//! Public C interface generation from checked host metadata.

use crate::core::ast::ProgramInterface;
use mal_syntax::diagnostic::Diagnostic;

mod header;
mod host_signature;
pub(in crate::backend) mod syntax;
mod types;

use self::types::{HostTypes, TypeRegistry};

/// Default filename of the program-specific public header.
pub const GENERATED_HEADER_NAME: &str = "program.mal.h";
/// Filename of the program-independent runtime and host support header.
pub const COMMON_HEADER_NAME: &str = "mal.h";
/// Checked-in common header, used to verify that its generator remains synchronized.
pub const COMMON_HEADER: &str = include_str!("../../../include/mal.h");

/// Renders the common header from the typed C construction model.
pub fn common_header() -> String {
    header::emit_common()
}

pub(crate) fn emit_file_header(
    interface: &ProgramInterface,
    dependencies: &[String],
    target: crate::backend::llvm::TargetLayout,
) -> String {
    header::emit(std::slice::from_ref(interface), target, dependencies, false)
}

pub(crate) fn emit_header_for_target(
    interface: &ProgramInterface,
    files: &[mal_syntax::source::FileId],
    target: crate::backend::llvm::TargetLayout,
) -> String {
    let interfaces = files
        .iter()
        .map(|file| interface.for_file(*file))
        .collect::<Vec<_>>();
    header::emit(&interfaces, target, &[], true)
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

    pub(in crate::backend) fn c_type(
        &self,
        ty: &mal_frontend::check::ast::Type,
    ) -> crate::backend::c::syntax::TypeName {
        self.types.c_type(ty)
    }
}

/// Returns whether `header_name` can be emitted verbatim as a quoted C include path.
pub fn is_valid_header_name(header_name: &str) -> bool {
    syntax::Directive::is_valid_quoted_include(header_name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn checked_in_common_header_matches_the_generator() {
        assert_eq!(super::COMMON_HEADER, super::common_header());
    }

    #[test]
    fn common_header_exposes_the_concise_host_surface() {
        let header = super::COMMON_HEADER;
        for spelling in [
            "#define mal_type(name)",
            "#define mal_product(...)",
            "#define mal_sum(...)",
            "#define mal_share(call, value)",
            "#define mal_move(value)",
            "#define mal_drop(value)",
            "#define mal_owned(name)",
            "#define mal_from_bits(type, raw_bits) (type){ .bits = (uintptr_t)(raw_bits) }",
            "#define mal_bits(value) (value).bits",
            "#define mal_buffer(call, storage, capacity) mal_detail_buffer(call, storage, capacity)",
            "#define mal_push(call, buffer, element)",
            "Like sizeof and _Alignof, it obtains information from a type",
            "evaluating it performs no lifecycle operation.",
            "owns one responsibility and drops it on lexical scope exit",
        ] {
            assert!(header.contains(spelling), "missing `{spelling}`");
        }
        for obsolete in [
            "mal_Unit_return",
            "mal_Buffer_make",
            "mal_Buffer_return_move",
            "MAL_DETAIL_DEFINE_SUM_VALUE_API",
            "mal_detail_from_bits",
            "mal_detail_bits(",
            "mal_detail_cleanup_UInt64",
        ] {
            assert!(!header.contains(obsolete), "obsolete `{obsolete}` remains");
        }
    }
}
