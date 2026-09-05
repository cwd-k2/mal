use std::fmt::Write;

use crate::check::ast::Type;
use crate::closure::ast::{Program, TopLevelPattern};
use crate::diagnostic::Diagnostic;

mod body;
mod runtime;
mod types;

use self::body::BodyEmitter;
use self::types::TypeRegistry;

pub const GENERATED_HEADER_NAME: &str = "program.mal.h";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    pub source: String,
    pub header: String,
}

pub fn emit(program: &Program) -> Result<Output, Diagnostic> {
    let main = find_main(program)?;
    let mut types = TypeRegistry::default();
    types.collect_program(program);
    let mut emitter = BodyEmitter::new(program, &types);
    let body = emitter.emit(main);

    let mut source = String::new();
    writeln!(source, "#include \"{GENERATED_HEADER_NAME}\"").unwrap();
    source.push_str(
        "#include <stddef.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n",
    );
    source.push_str(&types.source_declarations());
    source.push_str(&body.environment_declarations);
    source.push_str(&runtime::emit(&body.needs));
    source.push_str(&body.globals);
    source.push_str(&body.function_declarations);
    source.push_str(&body.function_definitions);
    source.push_str(&body.initializer);
    source.push_str(&body.main);

    Ok(Output {
        source,
        header: emit_header(program, &types),
    })
}

fn find_main(program: &Program) -> Result<&crate::closure::ast::TopLevelBinding, Diagnostic> {
    let main = program.bindings.iter().find(|binding| {
        matches!(
            &binding.pattern,
            TopLevelPattern::Binding { name, .. } if name == "main"
        )
    });
    let Some(main) = main else {
        return Err(Diagnostic::error(
            "executable program has no `main` binding",
        ));
    };
    let TopLevelPattern::Binding { ty, .. } = &main.pattern else {
        unreachable!()
    };
    let expected = Type::Function {
        parameter: Box::new(Type::Unit),
        result: Box::new(Type::Int32),
    };
    if *ty != expected {
        return Err(Diagnostic::error("`main` has the wrong type")
            .with_primary(main.span, "expected `Unit -> Int32`"));
    }
    Ok(main)
}

fn emit_header(program: &Program, types: &TypeRegistry) -> String {
    let mut output = String::from(
        "#ifndef MAL_PROGRAM_MAL_H\n#define MAL_PROGRAM_MAL_H\n\n#include <stdint.h>\n\n#define MAL_C_ABI_VERSION 0x000400u\n\ntypedef struct MalContext MalContext;\ntypedef struct { uint8_t unused; } MalUnit;\ntypedef struct { const uint8_t *data; uint64_t length; } MalString;\n\n_Noreturn void mal_trap(MalContext *context, const char *message);\nMalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);\n\n",
    );
    output.push_str(&types.header_declarations());
    for external in &program.externals {
        let result = if external.result == Type::Unit {
            "void".into()
        } else {
            types.c_type(&external.result)
        };
        write!(
            output,
            "{result} mal_ext_{}(MalContext *context",
            external.name
        )
        .unwrap();
        match &external.parameter {
            Type::Unit => {}
            Type::Product(elements) => {
                for (index, element) in elements.iter().enumerate() {
                    write!(output, ", {} argument_{index}", types.c_type(element)).unwrap();
                }
            }
            parameter => write!(output, ", {} value", types.c_type(parameter)).unwrap(),
        }
        output.push_str(");\n");
    }
    output.push_str("\n#endif\n");
    output
}
