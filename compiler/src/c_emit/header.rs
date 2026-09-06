use crate::core::ast::ProgramInterface;

use super::{HostTypes, TypeRegistry, host_signature::HostSignature};
use crate::c_emit::syntax::{
    Block, Comment, Declaration, Directive, Expr, FunctionDefinition, Initializer, Statement,
};

pub(super) fn emit(interface: &ProgramInterface, types: &TypeRegistry, host: &HostTypes) -> String {
    let signatures: Vec<_> = interface
        .externals
        .iter()
        .map(|external| HostSignature::new(external, types))
        .collect();
    let mut output = emit_prefix();
    let mut declarations = types.header_declarations(host);
    declarations.push_str(&types.header_alias_declarations(host, &interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.push_str(&declarations);
    }

    let mut helpers = types.header_opaque_helpers(host);
    helpers.push_str(&types.header_alias_helpers(host, &interface.type_aliases));
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.push_str(&helpers);
    }

    if !interface.externals.is_empty() {
        begin_section(&mut output, "External operations");
        for signature in &signatures {
            emit_external_declaration(&mut output, signature);
        }
        begin_section(&mut output, "External definition helpers");
        for signature in &signatures {
            emit_definition_macro(&mut output, signature);
        }
    }
    output.push_str(&Directive::Endif.render());
    output
}

pub(super) fn emit_host(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    header_name: &str,
) -> String {
    let mut output = Directive::IncludeQuoted(header_name.into()).render();
    for external in &interface.externals {
        let signature = HostSignature::new(external, types);
        output.push('\n');
        let mut body = Block::default();
        for name in signature.parameter_names().into_iter().skip(1) {
            body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier(name),
            )));
        }
        body.push(Statement::expression(Expr::named_call(
            "mal_trap",
            [
                Expr::identifier("context"),
                Expr::literal(format!(
                    "\"external operation `{}` is not implemented\"",
                    signature.operation_name
                )),
            ],
        )));
        output.push_str(&FunctionDefinition::new(macro_invocation(&signature), body).render());
    }
    output
}

fn emit_prefix() -> String {
    let mut output = String::new();
    for directive in [
        Directive::Ifndef("MAL_PROGRAM_MAL_H".into()),
        Directive::define("MAL_PROGRAM_MAL_H", ""),
    ] {
        output.push_str(&directive.render());
    }
    output.push('\n');
    output.push_str(&Directive::IncludeSystem("stdint.h".into()).render());
    output.push('\n');
    for directive in [
        Directive::define("MAL_C_ABI_VERSION", "0x000500u"),
        Directive::function_define("MAL_TYPE", ["name"], ["MalType_##name"]),
        Directive::function_define(
            "MAL_OPERATION",
            ["type", "operation"],
            ["mal_##type##_##operation"],
        ),
        Directive::function_define(
            "MAL_TAG",
            ["type", "variant"],
            ["MAL_##type##_TAG_##variant"],
        ),
        Directive::function_define("MAL_EXTERN", ["name"], ["mal_ext_##name"]),
    ] {
        output.push_str(&directive.render());
    }
    output.push('\n');
    output.push_str(&Directive::If("defined(__clang__) || defined(__GNUC__)".into()).render());
    output.push_str(
        &Directive::define("MAL_DETAIL_MAYBE_UNUSED", "__attribute__((unused))").render(),
    );
    output.push_str(&Directive::Else.render());
    output.push_str(&Directive::define("MAL_DETAIL_MAYBE_UNUSED", "").render());
    output.push_str(&Directive::Endif.render());
    output.push('\n');
    output.push_str(&Comment::new("Runtime API").render());
    output.push('\n');
    for declaration in [
        "typedef struct MalContext MalContext",
        "typedef struct { uint8_t unused; } MalType_Unit",
        "typedef uint8_t MalType_Bool",
        "typedef int8_t MalType_Int8",
        "typedef int16_t MalType_Int16",
        "typedef int32_t MalType_Int32",
        "typedef int64_t MalType_Int64",
        "typedef uint8_t MalType_UInt8",
        "typedef uint16_t MalType_UInt16",
        "typedef uint32_t MalType_UInt32",
        "typedef uint64_t MalType_UInt64",
        "typedef float MalType_Float32",
        "typedef double MalType_Float64",
        "typedef struct { const uint8_t *data; uint64_t length; } MalType_Symbol",
        "typedef struct { uint8_t *address; } MalType_Ptr",
    ] {
        output.push_str(&Declaration::new(declaration).render());
    }
    output.push('\n');
    output.push_str(&Directive::define("MAL_FALSE", "((MalType_Bool)UINT8_C(0))").render());
    output.push_str(&Directive::define("MAL_TRUE", "((MalType_Bool)UINT8_C(1))").render());
    output.push('\n');
    for declaration in [
        "_Noreturn void mal_trap(MalContext *context, const char *message)",
        "MalType_Symbol mal_Symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length)",
    ] {
        output.push_str(&Declaration::new(declaration).render());
    }
    output.push('\n');
    append_function(
        &mut output,
        "static inline const uint8_t *mal_Symbol_data(MalType_Symbol value)",
        Expr::identifier("value").field("data"),
    );
    append_function(
        &mut output,
        "static inline uint64_t mal_Symbol_length(MalType_Symbol value)",
        Expr::identifier("value").field("length"),
    );
    append_function(
        &mut output,
        "static inline MalType_Ptr mal_Ptr_from_address(uint8_t *address)",
        Expr::compound_literal(
            "MalType_Ptr",
            [Initializer::designated(
                "address",
                Expr::identifier("address"),
            )],
        ),
    );
    append_function(
        &mut output,
        "static inline uint8_t *mal_Ptr_address(MalType_Ptr value)",
        Expr::identifier("value").field("address"),
    );
    output
}

fn begin_section(output: &mut String, title: &str) {
    if !output.ends_with("\n\n") {
        output.push('\n');
    }
    output.push_str(&Comment::new(title).render());
    output.push('\n');
}

fn emit_external_declaration(output: &mut String, signature: &HostSignature<'_>) {
    output.push_str(
        &Declaration::new(format!(
            "{} mal_ext_{}({})",
            signature.result_type,
            signature.operation_name,
            signature.parameter_declarations().join(", ")
        ))
        .render(),
    );
}

fn emit_definition_macro(output: &mut String, signature: &HostSignature<'_>) {
    output.push_str(
        &Directive::define(format!("MAL_HAS_EXTERN_{}", signature.operation_name), "1").render(),
    );
    let mut replacement = Vec::new();
    replacement.push(format!(
        "{} mal_ext_{}(",
        signature.result_type, signature.operation_name
    ));
    let parameters = signature.definition_parameter_declarations();
    replacement.extend(parameters.iter().enumerate().map(|(index, parameter)| {
        let comma = if index + 1 == parameters.len() {
            ""
        } else {
            ","
        };
        format!("{parameter}{comma}")
    }));
    replacement.push(")".into());
    output.push_str(
        &Directive::function_define(
            format!("MAL_DEFINE_{}", signature.operation_name),
            signature.parameter_names(),
            replacement,
        )
        .render(),
    );
    output.push('\n');
}

fn macro_invocation(signature: &HostSignature<'_>) -> String {
    format!(
        "MAL_DEFINE_{}({})",
        signature.operation_name,
        signature.parameter_names().join(", ")
    )
}

fn append_function(output: &mut String, signature: &str, result: Expr) {
    output.push_str(
        &FunctionDefinition::new(signature, Block::new([Statement::return_value(result)])).render(),
    );
    output.push('\n');
}
