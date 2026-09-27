use crate::core::ast::ProgramInterface;

use super::{
    HostTypes, TypeRegistry,
    host_signature::{CompilerSignature, ExternalSignatures},
};
use crate::backend::c::syntax::{
    FunctionDefinition, FunctionSignature, MacroInvocation, TranslationUnit, c_block, c_comment,
    c_declaration, c_directive, c_expr, c_function, c_initializer, c_macro_invocation, c_signature,
    c_statement,
};

mod common;
mod prefix;

use self::prefix::emit_prefix;

pub(super) fn emit_common() -> String {
    common::emit()
}

pub(super) fn emit(
    interfaces: &[ProgramInterface],
    target: crate::backend::llvm::TargetLayout,
    dependencies: &[String],
    umbrella: bool,
) -> String {
    let memory_access = interfaces.iter().any(|interface| {
        interface
            .type_aliases
            .iter()
            .any(|alias| alias.host_memory_access)
    });
    let mut output = TranslationUnit::default();
    if !umbrella {
        output.push(c_directive!(ifndef "MAL_BUILD_UMBRELLA"));
    }
    output.extend(emit_prefix(
        target.index_size * 8,
        memory_access,
        dependencies,
        umbrella,
    ));
    for interface in interfaces {
        append_interface(&mut output, interface, target);
    }
    if !umbrella {
        output.push(c_directive!(endif));
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
    let guard = interface_guard(&body.render());
    output.blank_line();
    output.push(c_directive!(ifndef #{ guard.clone() }));
    output.push(c_directive!(define #{ guard };));
    output.extend(body);
    output.push(c_directive!(endif));
}

fn interface_body(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    host: &HostTypes,
    target: crate::backend::llvm::TargetLayout,
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

    let helpers = types.host_value_helpers(host, &interface.type_aliases);
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.extend(helpers);
    }

    let memory_helpers = types.memory_helpers(
        host,
        &interface.type_aliases,
        crate::backend::source_layout::SourceLayouts::new(target),
    );
    if !memory_helpers.is_empty() {
        begin_section(&mut output, "Canonical memory access");
        output.extend(memory_helpers);
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
    let mut output = TranslationUnit::new([c_directive!(include(quoted #{ header_name })).into()]);
    for external in &interface.externals {
        let signatures = ExternalSignatures::new(external, types);
        let signature = &signatures.host_body;
        output.blank_line();
        let unused_parameters = signature
            .parameter_names()
            .into_iter()
            .skip(1)
            .map(|name| c_statement!(cast((named("void")), (id(#{ name })));));
        let body = c_block! {
            ...#{ unused_parameters }
            call("mal_call_trap", [
                id("call"),
                string(#{ format!(
                    "external operation `{}` is not implemented",
                    signatures.host_body.operation_name
                ) }),
            ]);
        };
        output.push(c_function!(macro #{ host_macro_invocation(signature) } body #{ body }));
    }
    output.render()
}

fn begin_section(output: &mut TranslationUnit, title: &str) {
    output.blank_line();
    output.push(c_comment!(#{ title }));
    output.blank_line();
}

fn emit_external_declaration(output: &mut TranslationUnit, signature: &CompilerSignature<'_>) {
    output.push(c_declaration!(fn #{ external_signature(signature, false) };));
}

fn emit_definition_macro(
    output: &mut TranslationUnit,
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) {
    let signature = &signatures.compiler;
    let presence_name = format!("MAL_HAS_EXTERN_{}", signature.operation_name);
    output.push(c_directive!(define #{ presence_name } = (number(1));));
    let definition_name = format!("MAL_DEFINE_{}", signature.operation_name);
    output.push(c_directive! {
        define_items #{ definition_name } {
            parameters: #{ signatures.host_body.parameter_names() },
            declarations: #{ [signatures.host_body.signature()] },
            definitions: #{ [wrapper_definition(signatures, external, types)] },
            trailing: #{ signatures.host_body.signature() },
        }
    });
}

fn wrapper_definition(
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) -> FunctionDefinition {
    let mut arguments = vec![c_expr!(address((id("call"))))];
    match &external.parameter {
        mal_frontend::check::ast::Type::Unit => {}
        mal_frontend::check::ast::Type::Product(elements) => {
            let initializers = elements.iter().enumerate().map(|(field, _)| {
                c_initializer! {
                    field(
                        #{ format!("field_{field}") },
                        (id(#{ format!("argument_{field}") }))
                    )
                }
            });
            let raw = c_expr! {
                compound(
                    #{ types.c_type(&external.parameter) },
                    [...#{ initializers }]
                )
            };
            arguments.push(types.raw_to_host_value(
                &external.parameter,
                external.parameter_alias.as_deref(),
                c_expr!(address((id("call")))),
                raw,
            ));
        }
        ty => arguments.push(types.raw_to_host_value(
            ty,
            external.parameter_alias.as_deref(),
            c_expr!(address((id("call")))),
            c_expr!(id("value")),
        )),
    }
    let call = c_expr! {
        call(
            #{ format!("mal_detail_{}", external.name) },
            [...#{ arguments }]
        )
    };
    let terminal = if external.result == mal_frontend::check::ast::Type::Unit {
        c_statement!(#{ call };)
    } else {
        c_statement!(return #{ call };)
    };
    let body = c_block! {
        let "call": named("mal_call_t") = (compound((named("mal_call_t")), [
            field("mal_detail_context", (id("context"))),
        ]));
        #{ terminal }
    };
    c_function!(signature #{ external_signature(&signatures.compiler, true) } body #{ body })
}

fn host_macro_invocation(
    signature: &super::host_signature::HostBodySignature<'_>,
) -> MacroInvocation {
    let arguments = signature
        .parameter_names()
        .into_iter()
        .map(|name| c_expr!(id(#{ name })));
    c_macro_invocation! {
        #{ format!("MAL_DEFINE_{}", signature.operation_name) }([
            ...#{ arguments },
        ])
    }
}

fn external_signature(signature: &CompilerSignature<'_>, definition: bool) -> FunctionSignature {
    let parameters = if definition {
        signature.definition_parameters()
    } else {
        signature.parameters()
    };
    c_signature! {
        fn #{ format!("mal_ext_{}", signature.operation_name) }(
            ...#{ parameters },
        ) -> #{ signature.result_type.clone() }
    }
}
