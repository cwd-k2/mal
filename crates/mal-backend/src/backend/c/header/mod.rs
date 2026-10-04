//! Composition of common, file-specific, umbrella, and host-stub C translation units.

use crate::core::ast::ProgramInterface;

use super::{
    HostTypes, TypeRegistry,
    host_signature::{CompilerSignature, ExternalSignatures},
};
use crate::backend::c::syntax::{
    Directive, Expr, FunctionDefinition, FunctionSignature, MacroInvocation, TranslationUnit,
    c_block, c_expr, c_initializers, c_invocation, c_items, c_signature, c_statement,
};

mod common;
mod prefix;

use self::prefix::emit_prefix;

const C_ABI_VERSION_LITERAL: &str = "0x000a00u";

pub(super) fn emit_common() -> String {
    common::emit()
}

pub(super) fn emit(
    interfaces: &[ProgramInterface],
    target: crate::backend::llvm::TargetLayout,
    dependencies: &[String],
    umbrella: bool,
) -> String {
    let mut output = TranslationUnit::default();
    output.extend(emit_prefix(target, dependencies, umbrella));
    for interface in interfaces {
        append_interface(&mut output, interface, target);
    }
    if !umbrella {
        output = c_items! {
            if !defined(MAL_BUILD_UMBRELLA) {
                ..{ output }
            }
        };
    }
    output.render()
}

fn append_interface(
    output: &mut TranslationUnit,
    interface: &ProgramInterface,
    target: crate::backend::llvm::TargetLayout,
) {
    let mut types = TypeRegistry::default();
    let host = HostTypes::collect(interface, &mut types);
    let body = interface_body(interface, &types, &host, target);
    let has_body = !body.is_empty();
    let guard = interface_guard(&body.render());
    output.blank_line();
    let mut guarded = c_items! { define!({ guard.clone() }); };
    if has_body {
        guarded.blank_line();
    }
    guarded.extend(body);
    if has_body {
        guarded.blank_line();
    }
    output.extend(c_items! {
        if !defined({ guard }) {
            ..{ guarded }
        }
    });
}

fn interface_body(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    host: &HostTypes,
    _target: crate::backend::llvm::TargetLayout,
) -> TranslationUnit {
    let signatures: Vec<_> = interface
        .externals
        .iter()
        .map(|external| ExternalSignatures::new(external, types))
        .collect();
    let mut output = TranslationUnit::default();
    let mut declarations = types.header_declarations(host);
    declarations.extend(types.header_alias_declarations(host, &interface.type_aliases));
    declarations.extend(types.host_value_declarations(host, &interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.extend(declarations);
    }

    let mut helpers = types.host_lifecycle_helpers(host, &interface.type_aliases);
    helpers.extend(types.host_value_helpers(host));
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.extend(helpers);
    }

    if !interface.externals.is_empty() {
        begin_section(&mut output, "External operations");
        for signatures in &signatures {
            emit_external_declaration(&mut output, &signatures.compiler);
        }
        begin_section(&mut output, "External definition helpers");
        for (index, (external, signatures)) in
            interface.externals.iter().zip(&signatures).enumerate()
        {
            if index != 0 {
                output.blank_line();
            }
            emit_definition_macro(&mut output, signatures, external, types);
        }
    }
    output
}

fn interface_guard(body: &str) -> String {
    let fingerprint = body.bytes().fold(0xcbf29ce484222325_u64, |state, byte| {
        (state ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    format!("MAL_GENERATED_INTERFACE_{fingerprint:016X}_H")
}

pub(super) fn emit_host(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    header_name: &str,
) -> String {
    let mut output = c_items! { include_quoted!({ header_name }); };
    for external in &interface.externals {
        let signatures = ExternalSignatures::new(external, types);
        let signature = &signatures.host_body;
        output.blank_line();
        let unused_parameters = signature
            .parameter_names()
            .into_iter()
            .skip(1)
            .map(|name| c_statement!({ name } as void;));
        let message = Expr::string(format!(
            "external operation `{}` is not implemented",
            signatures.host_body.operation_name
        ));
        let body = c_block! {
            ..{ unused_parameters }
            mal_call_trap(call, { message });
        };
        output.push(FunctionDefinition::from_macro(
            host_macro_invocation(signature),
            body,
        ));
    }
    output.render()
}

fn begin_section(output: &mut TranslationUnit, title: &str) {
    output.blank_line();
    output.extend(c_items! { comment!({ title }); });
    output.blank_line();
}

fn emit_external_declaration(output: &mut TranslationUnit, signature: &CompilerSignature<'_>) {
    let signature = external_signature(signature, false);
    output.extend(c_items! { { signature }; });
}

fn emit_definition_macro(
    output: &mut TranslationUnit,
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) {
    let signature = &signatures.compiler;
    let presence_name = format!("MAL_HAS_EXTERN_{}", signature.operation_name);
    output.extend(c_items! { define!({ presence_name } = 1); });
    let definition_name = format!("MAL_DEFINE_{}", signature.operation_name);
    output.push(Directive::function_items_define(
        definition_name,
        signatures.host_body.parameter_names(),
        [signatures.host_body.signature()],
        [wrapper_definition(signatures, external, types)],
        signatures.host_body.signature(),
    ));
}

fn wrapper_definition(
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) -> FunctionDefinition {
    let mut arguments = vec![c_expr!(&call)];
    match &external.parameter {
        mal_frontend::check::ast::Type::Unit => {}
        mal_frontend::check::ast::Type::Product(elements) => {
            let initializers = elements.iter().enumerate().flat_map(|(field, _)| {
                let name = format!("field_{field}");
                c_initializers! { { name }: { c_expr!({ format!("argument_{field}") }) } }
            });
            let ty = types.c_type(&external.parameter);
            let raw = c_expr!({ ty } { ..{ initializers } });
            arguments.push(types.raw_to_host_value(
                &external.parameter,
                external.parameter_alias.as_deref(),
                c_expr!(&call),
                raw,
            ));
        }
        ty => arguments.push(types.raw_to_host_value(
            ty,
            external.parameter_alias.as_deref(),
            c_expr!(&call),
            c_expr!(value),
        )),
    }
    let call = c_expr!({ format!("mal_detail_{}", external.name) }(..{ arguments }));
    let terminal = if external.result == mal_frontend::check::ast::Type::Unit {
        c_statement!(({ call });)
    } else {
        let result = types.host_to_raw_value(&external.result, c_expr!(&call), call);
        c_statement!(return { result };)
    };
    let body = c_block! {
        let call: mal_call_t = mal_call_t { mal_detail_context: context };
        { terminal }
    };
    FunctionDefinition::from_signature(external_signature(&signatures.compiler, true), body)
}

fn host_macro_invocation(
    signature: &super::host_signature::HostBodySignature<'_>,
) -> MacroInvocation {
    let arguments = signature
        .parameter_names()
        .into_iter()
        .map(|name| c_expr!({ name }));
    let name = format!("MAL_DEFINE_{}", signature.operation_name);
    c_invocation!({ name }(..{ arguments }))
}

fn external_signature(signature: &CompilerSignature<'_>, definition: bool) -> FunctionSignature {
    let parameters = if definition {
        signature.definition_parameters()
    } else {
        signature.parameters()
    };
    c_signature! {
        fn { format!("mal_ext_{}", signature.operation_name) }(
            ..{ parameters },
        ) -> { signature.result_type.clone() }
    }
}
