use crate::closure::ast::Program;

use super::{TypeRegistry, host_signature::HostSignature};

const HEADER_PREFIX: &str = r#"#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u

#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name

#if defined(__clang__) || defined(__GNUC__)
#define MAL_DETAIL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_DETAIL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalType_Unit;
typedef uint8_t MalType_Bool;
typedef int8_t MalType_Int8;
typedef int16_t MalType_Int16;
typedef int32_t MalType_Int32;
typedef int64_t MalType_Int64;
typedef uint8_t MalType_UInt8;
typedef uint16_t MalType_UInt16;
typedef uint32_t MalType_UInt32;
typedef uint64_t MalType_UInt64;
typedef float MalType_Float32;
typedef double MalType_Float64;
typedef struct { const uint8_t *data; uint64_t length; } MalType_Engram;
typedef struct { uint8_t *address; } MalType_Ptr;

#define MAL_FALSE ((MalType_Bool)UINT8_C(0))
#define MAL_TRUE ((MalType_Bool)UINT8_C(1))

_Noreturn void mal_trap(MalContext *context, const char *message);
MalType_Engram mal_Engram_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);

static inline const uint8_t *mal_Engram_data(MalType_Engram value) {
    return value.data;
}

static inline uint64_t mal_Engram_length(MalType_Engram value) {
    return value.length;
}

static inline MalType_Ptr mal_Ptr_from_address(uint8_t *address) {
    return (MalType_Ptr){ .address = address };
}

static inline uint8_t *mal_Ptr_address(MalType_Ptr value) {
    return value.address;
}
"#;

pub(super) fn emit(program: &Program, types: &TypeRegistry) -> String {
    let signatures: Vec<_> = program
        .interface
        .externals
        .iter()
        .map(|external| HostSignature::new(external, types))
        .collect();
    let mut output = String::from(HEADER_PREFIX);
    let mut declarations = types.header_declarations();
    declarations.push_str(&types.header_alias_declarations(&program.interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.push_str(&declarations);
    }

    let mut helpers = types.header_opaque_helpers();
    helpers.push_str(&types.header_alias_helpers(&program.interface.type_aliases));
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.push_str(&helpers);
    }

    if !program.interface.externals.is_empty() {
        begin_section(&mut output, "External operations");
        for signature in &signatures {
            emit_external_declaration(&mut output, signature);
        }
        begin_section(&mut output, "External definition helpers");
        for signature in &signatures {
            emit_definition_macro(&mut output, signature);
        }
    }
    output.push_str("#endif\n");
    output
}

fn begin_section(output: &mut String, title: &str) {
    if !output.ends_with("\n\n") {
        output.push('\n');
    }
    c_line!(output, 0, "/* {title} */");
    output.push('\n');
}

pub(super) fn emit_host(program: &Program, types: &TypeRegistry, header_name: &str) -> String {
    let mut output = format!("#include \"{header_name}\"\n");
    for external in &program.interface.externals {
        let signature = HostSignature::new(external, types);
        output.push('\n');
        emit_macro_invocation(&mut output, &signature);
        output.push_str(" {\n");
        for name in signature.parameter_names().into_iter().skip(1) {
            c_line!(&mut output, 1, "(void){name};");
        }
        c_line!(
            &mut output,
            1,
            "mal_trap(context, \"external operation `{}` is not implemented\");",
            signature.operation_name
        );
        output.push_str("}\n");
    }
    output
}

fn emit_external_declaration(output: &mut String, signature: &HostSignature<'_>) {
    c_line!(
        output,
        0,
        "{} mal_ext_{}(",
        signature.result_type,
        signature.operation_name
    );
    let parameters = signature.parameter_declarations();
    emit_indented_lines(output, &parameters, 1, false);
    output.push_str(");\n");
}

fn emit_definition_macro(output: &mut String, signature: &HostSignature<'_>) {
    c_line!(
        output,
        0,
        "#define MAL_HAS_EXTERN_{} 1",
        signature.operation_name
    );
    c_write!(output, "#define MAL_DEFINE_{}(", signature.operation_name);
    c_write!(output, "{}", signature.parameter_names().join(", "));
    output.push_str(") \\\n");
    c_line!(
        output,
        1,
        "{} mal_ext_{}( \\",
        signature.result_type,
        signature.operation_name
    );
    let parameters = signature.definition_parameter_declarations();
    emit_indented_lines(output, &parameters, 2, true);
    output.push_str("    )\n\n");
}

fn emit_indented_lines<T: AsRef<str>>(
    output: &mut String,
    lines: &[T],
    indent: usize,
    continuation: bool,
) {
    for (index, line) in lines.iter().enumerate() {
        let line = line.as_ref();
        let comma = if index + 1 == lines.len() { "" } else { "," };
        let continuation = if continuation { " \\" } else { "" };
        c_line!(output, indent, "{line}{comma}{continuation}");
    }
}

fn emit_macro_invocation(output: &mut String, signature: &HostSignature<'_>) {
    let names = signature.parameter_names();
    if names.len() <= 2 {
        c_write!(
            output,
            "MAL_DEFINE_{}({})",
            signature.operation_name,
            names.join(", ")
        );
        return;
    }
    c_line!(output, 0, "MAL_DEFINE_{}(", signature.operation_name);
    emit_indented_lines(output, &names, 1, false);
    output.push(')');
}
