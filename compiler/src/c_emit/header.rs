use crate::closure::ast::Program;

use super::{TypeRegistry, host_signature::HostSignature};

pub(super) fn emit(program: &Program, types: &TypeRegistry) -> String {
    let signatures: Vec<_> = program
        .externals
        .iter()
        .map(|external| HostSignature::new(external, types))
        .collect();
    let mut output = String::from(
        "#ifndef MAL_PROGRAM_MAL_H\n#define MAL_PROGRAM_MAL_H\n\n#include <stdint.h>\n\n#define MAL_C_ABI_VERSION 0x000500u\n\n#if defined(__clang__) || defined(__GNUC__)\n#define MAL_MAYBE_UNUSED __attribute__((unused))\n#else\n#define MAL_MAYBE_UNUSED\n#endif\n\n/* Runtime API */\n\ntypedef struct MalContext MalContext;\ntypedef struct { uint8_t unused; } MalUnit;\ntypedef struct { const uint8_t *data; uint64_t length; } MalString;\ntypedef struct { uint8_t *address; } MalPtr;\n\n_Noreturn void mal_trap(MalContext *context, const char *message);\nMalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);\n\nstatic inline MalPtr mal_ptr_from_address(uint8_t *address) {\n    return (MalPtr){ .address = address };\n}\n\nstatic inline uint8_t *mal_ptr_address(MalPtr value) {\n    return value.address;\n}\n",
    );
    let mut declarations = types.header_declarations();
    declarations.push_str(&types.header_alias_declarations(&program.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.push_str(&declarations);
    }

    let mut helpers = types.header_opaque_helpers();
    helpers.push_str(&types.header_alias_helpers(&program.type_aliases));
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.push_str(&helpers);
    }

    if !program.externals.is_empty() {
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

pub(super) fn emit_host(program: &Program, types: &TypeRegistry) -> String {
    let mut output = String::from("#include \"program.mal.h\"\n");
    for external in &program.externals {
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
