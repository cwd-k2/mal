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
    validate_externals(program)?;
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
    source.push_str("typedef struct { uint8_t unused; } MalUnit;\n\n");
    source.push_str(&types.declarations());
    source.push_str(&body.environment_declarations);
    source.push_str(&runtime::emit(&body.needs));
    source.push_str(&body.globals);
    source.push_str(&body.function_declarations);
    source.push_str(&body.function_definitions);
    source.push_str(&body.initializer);
    source.push_str(&body.main);

    Ok(Output {
        source,
        header: emit_header(program),
    })
}

fn validate_externals(program: &Program) -> Result<(), Diagnostic> {
    for external in &program.externals {
        if !is_m1_scalar(&external.parameter) || !is_m1_scalar(&external.result) {
            return Err(Diagnostic::error(format!(
                "external operation `{}` is outside the scalar C ABI",
                external.name
            ))
            .with_primary(
                external.span,
                "only Unit and fixed-width integer extern parameters and results are supported",
            ));
        }
    }
    Ok(())
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

fn emit_header(program: &Program) -> String {
    let mut output = String::from(
        "#ifndef MAL_PROGRAM_MAL_H\n#define MAL_PROGRAM_MAL_H\n\n#include <stdint.h>\n\n#define MAL_C_ABI_VERSION 0x000400u\n\ntypedef struct MalContext MalContext;\n\n_Noreturn void mal_trap(MalContext *context, const char *message);\n\n",
    );
    for external in &program.externals {
        let result = match external.result {
            Type::Unit => "void",
            Type::Int8 => "int8_t",
            Type::Int16 => "int16_t",
            Type::Int32 => "int32_t",
            Type::Int64 => "int64_t",
            Type::UInt8 => "uint8_t",
            Type::UInt16 => "uint16_t",
            Type::UInt32 => "uint32_t",
            Type::UInt64 => "uint64_t",
            _ => unreachable!("extern ABI is validated before header emission"),
        };
        write!(
            output,
            "{result} mal_ext_{}(MalContext *context",
            external.name
        )
        .unwrap();
        if external.parameter != Type::Unit {
            write!(output, ", {} value", c_scalar_type(&external.parameter)).unwrap();
        }
        output.push_str(");\n");
    }
    output.push_str("\n#endif\n");
    output
}

fn is_m1_scalar(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
    )
}

fn c_scalar_type(ty: &Type) -> &'static str {
    match ty {
        Type::Int8 => "int8_t",
        Type::Int16 => "int16_t",
        Type::Int32 => "int32_t",
        Type::Int64 => "int64_t",
        Type::UInt8 => "uint8_t",
        Type::UInt16 => "uint16_t",
        Type::UInt32 => "uint32_t",
        Type::UInt64 => "uint64_t",
        _ => unreachable!("called only for integer scalar types"),
    }
}
