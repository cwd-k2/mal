use crate::check::ast::Type;
use crate::closure::ast::{Program, TopLevelPattern};
use crate::core::ast::ProgramInterface;
use crate::diagnostic::Diagnostic;

mod body;
mod header;
mod host_signature;
mod runtime;
mod scalar;
mod syntax;
mod types;

use self::body::BodyEmitter;
use self::types::{HostTypes, TypeRegistry};

pub const GENERATED_HEADER_NAME: &str = "program.mal.h";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    pub source: String,
    pub header: String,
}

pub fn emit(program: &Program) -> Result<Output, Diagnostic> {
    let main = find_main(program)?;
    let mut types = TypeRegistry::default();
    let host = HostTypes::collect(&program.interface, &mut types);
    types.collect_program_body(program);
    let mut emitter = BodyEmitter::new(program, &types);
    let body = emitter.emit(main);

    let mut source = String::new();
    source.push_str(&syntax::Directive::IncludeQuoted(GENERATED_HEADER_NAME.into()).render());
    for header in [
        "float.h", "stddef.h", "stdint.h", "stdio.h", "stdlib.h", "string.h",
    ] {
        source.push_str(&syntax::Directive::IncludeSystem(header.into()).render());
    }
    source.push('\n');
    if types.uses_float() {
        source.push_str(&float_target_profile());
        if types.uses_float32() {
            source.push_str(&float_from_bits_definition("float", "float32", "uint32_t"));
        }
        if types.uses_float64() {
            source.push_str(&float_from_bits_definition("double", "float64", "uint64_t"));
        }
    }
    source.push_str(&types.source_declarations(&host));
    source.push_str(&body.environment_declarations);
    source.push_str(&runtime::emit(&body.needs));
    source.push_str(&body.globals);
    source.push_str(&body.function_declarations);
    source.push_str(&body.function_definitions);
    source.push_str(&body.initializer);
    source.push_str(&body.main);

    Ok(Output {
        source,
        header: header::emit(&program.interface, &types, &host),
    })
}

pub fn emit_header(interface: &ProgramInterface) -> String {
    let mut types = TypeRegistry::default();
    let host = HostTypes::collect(interface, &mut types);
    header::emit(interface, &types, &host)
}

pub fn emit_host(interface: &ProgramInterface, header_name: &str) -> Result<String, Diagnostic> {
    if !is_valid_header_name(header_name) {
        return Err(Diagnostic::error(
            "generated host header name is not valid in a quoted C include",
        ));
    }
    let mut types = TypeRegistry::default();
    let _host = HostTypes::collect(interface, &mut types);
    Ok(header::emit_host(interface, &types, header_name))
}

pub(crate) fn is_valid_header_name(header_name: &str) -> bool {
    !header_name.is_empty()
        && !header_name
            .chars()
            .any(|character| character.is_control() || matches!(character, '"' | '\\'))
}

fn float_target_profile() -> String {
    use self::syntax::{Declaration, Directive};

    let mut output = Directive::If("defined(__clang__)".into()).render();
    output.push_str(&Directive::Pragma("STDC FENV_ACCESS ON".into()).render());
    output.push_str(&Directive::Pragma("STDC FP_CONTRACT OFF".into()).render());
    output.push_str(&Directive::Endif.render());
    output.push('\n');
    for assertion in [
        "_Static_assert(FLT_RADIX == 2, \"mal requires radix-2 floating point\")",
        "_Static_assert(sizeof(float) == 4 && FLT_MANT_DIG == 24 && FLT_MAX_EXP == 128 && FLT_MIN_EXP == -125, \"mal requires binary32 float\")",
        "_Static_assert(sizeof(double) == 8 && DBL_MANT_DIG == 53 && DBL_MAX_EXP == 1024 && DBL_MIN_EXP == -1021, \"mal requires binary64 double\")",
        "_Static_assert(FLT_EVAL_METHOD == 0, \"mal requires evaluation in the operand format\")",
    ] {
        output.push_str(&Declaration::new(assertion).render());
    }
    for (condition, message) in [
        (
            "defined(FLT_HAS_SUBNORM) && FLT_HAS_SUBNORM != 1",
            "\"mal requires float subnormals\"",
        ),
        (
            "defined(DBL_HAS_SUBNORM) && DBL_HAS_SUBNORM != 1",
            "\"mal requires double subnormals\"",
        ),
    ] {
        output.push_str(&Directive::If(condition.into()).render());
        output.push_str(&Directive::Error(message.into()).render());
        output.push_str(&Directive::Endif.render());
    }
    output.push('\n');
    output
}

fn float_from_bits_definition(c_type: &str, name: &str, bits_type: &str) -> String {
    use self::syntax::{Block, Expr, FunctionDefinition, Statement};

    let body = Block::new([
        Statement::declaration(format!("{c_type} value"), None),
        Statement::expression(Expr::named_call(
            "memcpy",
            [
                Expr::unary("&", Expr::identifier("value")),
                Expr::unary("&", Expr::identifier("bits")),
                Expr::sizeof_type("value"),
            ],
        )),
        Statement::return_value(Expr::identifier("value")),
    ]);
    let mut output = FunctionDefinition::new(
        format!("static inline {c_type} mal_{name}_from_bits({bits_type} bits)"),
        body,
    )
    .render();
    output.push('\n');
    output
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
    let Type::Function { parameter, result } = ty else {
        return Err(Diagnostic::error("`main` has the wrong type").with_primary(
            main.span,
            "expected `Unit -> Int32` or `(UInt64, Ptr) -> Int32`",
        ));
    };
    let accepts_arguments = matches!(
        parameter.as_ref(),
        Type::Product(elements)
            if elements.as_slice() == [Type::UInt64, Type::Ptr]
    );
    if **result != Type::Int32 || (**parameter != Type::Unit && !accepts_arguments) {
        return Err(Diagnostic::error("`main` has the wrong type").with_primary(
            main.span,
            "expected `Unit -> Int32` or `(UInt64, Ptr) -> Int32`",
        ));
    }
    Ok(main)
}
