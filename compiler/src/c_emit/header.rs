use crate::check::ast::Type;
use crate::closure::ast::{ExternalOperation, Program};

use super::TypeRegistry;

pub(super) fn emit(program: &Program, types: &TypeRegistry) -> String {
    let mut output = String::from(
        "#ifndef MAL_PROGRAM_MAL_H\n#define MAL_PROGRAM_MAL_H\n\n#include <stdint.h>\n\n#define MAL_C_ABI_VERSION 0x000500u\n\n#if defined(__clang__) || defined(__GNUC__)\n#define MAL_MAYBE_UNUSED __attribute__((unused))\n#else\n#define MAL_MAYBE_UNUSED\n#endif\n\n/* Runtime API */\n\ntypedef struct MalContext MalContext;\ntypedef struct { uint8_t unused; } MalUnit;\ntypedef struct { const uint8_t *data; uint64_t length; } MalString;\ntypedef struct { uint8_t *address; } MalPtr;\n\n_Noreturn void mal_trap(MalContext *context, const char *message);\nMalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);\n\nstatic inline MalPtr mal_ptr_from_address(uint8_t *address) {\n    return (MalPtr){ .address = address };\n}\n\nstatic inline uint8_t *mal_ptr_address(MalPtr value) {\n    return value.address;\n}\n",
    );
    let mut declarations = types.header_declarations();
    declarations.push_str(&types.header_alias_declarations(&program.type_aliases));
    if !declarations.is_empty() {
        output.push_str("\n/* Host-visible types */\n\n");
        output.push_str(&declarations);
    }

    let mut helpers = types.header_opaque_helpers();
    helpers.push_str(&types.header_alias_helpers(&program.type_aliases));
    if !helpers.is_empty() {
        output.push_str("/* Type helpers */\n\n");
        output.push_str(&helpers);
    }

    if !program.externals.is_empty() {
        output.push_str("/* External operations */\n\n");
        for external in &program.externals {
            emit_external_declaration(&mut output, external, types);
        }
        output.push_str("\n/* External definition helpers */\n\n");
        for external in &program.externals {
            emit_definition_macro(&mut output, external, types);
        }
        output.push('\n');
    }
    output.push_str("#endif\n");
    output
}

pub(super) fn emit_host(program: &Program) -> String {
    let mut output = String::from("#include \"program.mal.h\"\n");
    for external in &program.externals {
        output.push('\n');
        emit_macro_invocation(&mut output, external);
        output.push_str(" {\n");
        for name in external_parameter_names(external).into_iter().skip(1) {
            c_line!(&mut output, 1, "(void){name};");
        }
        c_line!(
            &mut output,
            1,
            "mal_trap(context, \"external operation `{}` is not implemented\");",
            external.name
        );
        output.push_str("}\n");
    }
    output
}

fn emit_external_declaration(
    output: &mut String,
    external: &ExternalOperation,
    types: &TypeRegistry,
) {
    c_line!(
        output,
        0,
        "{} mal_ext_{}(",
        external_result_type(external, types),
        external.name
    );
    let parameters = external_parameter_declarations(external, types, false);
    emit_indented_lines(output, &parameters, 1, false);
    output.push_str(");\n");
}

fn emit_definition_macro(output: &mut String, external: &ExternalOperation, types: &TypeRegistry) {
    c_write!(output, "#define MAL_DEFINE_{}(", external.name);
    c_write!(output, "{}", external_parameter_names(external).join(", "));
    output.push_str(") \\\n");
    c_line!(
        output,
        1,
        "{} mal_ext_{}( \\",
        external_result_type(external, types),
        external.name
    );
    let parameters = external_parameter_declarations(external, types, true);
    emit_indented_lines(output, &parameters, 2, true);
    output.push_str("    )\n");
}

fn emit_indented_lines(output: &mut String, lines: &[String], indent: usize, continuation: bool) {
    for (index, line) in lines.iter().enumerate() {
        let comma = if index + 1 == lines.len() { "" } else { "," };
        let continuation = if continuation { " \\" } else { "" };
        c_line!(output, indent, "{line}{comma}{continuation}");
    }
}

fn external_parameter_declarations(
    external: &ExternalOperation,
    types: &TypeRegistry,
    unused_context: bool,
) -> Vec<String> {
    let context = if unused_context {
        "MalContext *context MAL_MAYBE_UNUSED"
    } else {
        "MalContext *context"
    };
    let mut parameters = vec![context.into()];
    match &external.parameter {
        Type::Unit => {}
        Type::Product(elements) => {
            for (index, element) in elements.iter().enumerate() {
                parameters.push(format!(
                    "{} argument_{index}",
                    types.header_c_type(element, external.parameter_aliases[index].as_deref())
                ));
            }
        }
        parameter => parameters.push(format!(
            "{} value",
            types.header_c_type(parameter, external.parameter_aliases[0].as_deref())
        )),
    }
    parameters
}

fn external_parameter_names(external: &ExternalOperation) -> Vec<String> {
    let mut names = vec!["context".into()];
    match &external.parameter {
        Type::Unit => {}
        Type::Product(elements) => {
            names.extend((0..elements.len()).map(|index| format!("argument_{index}")));
        }
        _ => names.push("value".into()),
    }
    names
}

fn emit_macro_invocation(output: &mut String, external: &ExternalOperation) {
    let names = external_parameter_names(external);
    if names.len() <= 2 {
        c_write!(output, "MAL_DEFINE_{}({})", external.name, names.join(", "));
        return;
    }
    c_line!(output, 0, "MAL_DEFINE_{}(", external.name);
    emit_indented_lines(output, &names, 1, false);
    output.push(')');
}

fn external_result_type(external: &ExternalOperation, types: &TypeRegistry) -> String {
    if external.result == Type::Unit {
        "void".into()
    } else {
        types.header_c_type(&external.result, external.result_alias.as_deref())
    }
}
