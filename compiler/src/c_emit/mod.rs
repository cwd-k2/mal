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
    let execution = crate::execution::lower(program.clone());
    emit_execution(&execution)
}

pub(crate) fn emit_execution(program: &crate::execution::Program) -> Result<Output, Diagnostic> {
    let lowered = &program.lowered;
    let main = find_main(lowered)?;
    let mut types = TypeRegistry::default();
    let host = HostTypes::collect(&lowered.interface, &mut types);
    types.collect_program_body(lowered);
    let mut emitter = BodyEmitter::new(program, &types);
    let body = emitter.emit(main);

    let mut source = syntax::TranslationUnit::default();
    source.push(syntax::Directive::include_quoted(GENERATED_HEADER_NAME));
    for header in [
        "float.h", "stddef.h", "stdint.h", "stdio.h", "stdlib.h", "string.h",
    ] {
        source.push(syntax::Directive::include_system(header));
    }
    source.blank_line();
    if types.uses_float() {
        source.extend(float_target_profile());
        if types.uses_float32() {
            source.push(float_from_bits_definition("float", "float32", "uint32_t"));
            source.blank_line();
        }
        if types.uses_float64() {
            source.push(float_from_bits_definition("double", "float64", "uint64_t"));
            source.blank_line();
        }
    }
    source.extend(types.source_declarations(&host));
    source.extend(body.environment_declarations);
    source.extend(runtime::emit(&body.needs));
    source.extend(body.control_frames);
    source.extend(types.lifetime_definitions());
    source.extend(body.environment_definitions);
    source.extend(body.globals);
    source.extend(body.function_declarations);
    source.extend(body.function_definitions);
    source.extend(body.initializer);
    source.extend(body.program_destroy);
    source.extend(body.main);

    Ok(Output {
        source: source.render(),
        header: header::emit(&lowered.interface, &types, &host),
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
    syntax::Directive::is_valid_quoted_include(header_name)
}

fn float_target_profile() -> syntax::TranslationUnit {
    use self::syntax::{Declaration, Directive, Expr, Pragma, PreprocessorExpr, TranslationUnit};

    let mut output = TranslationUnit::new([
        Directive::If(PreprocessorExpr::defined("__clang__")).into(),
        Directive::pragma(Pragma::FenvAccessOn).into(),
        Directive::pragma(Pragma::FpContractOff).into(),
        Directive::Endif.into(),
    ]);
    output.blank_line();
    for (condition, message) in [
        (
            Expr::equal(Expr::identifier("FLT_RADIX"), Expr::number("2")),
            "mal requires radix-2 floating point",
        ),
        (
            conjunction([
                Expr::equal(Expr::sizeof_type("float"), Expr::number("4")),
                Expr::equal(Expr::identifier("FLT_MANT_DIG"), Expr::number("24")),
                Expr::equal(Expr::identifier("FLT_MAX_EXP"), Expr::number("128")),
                Expr::equal(
                    Expr::identifier("FLT_MIN_EXP"),
                    Expr::negate(Expr::number("125")),
                ),
            ]),
            "mal requires binary32 float",
        ),
        (
            conjunction([
                Expr::equal(Expr::sizeof_type("double"), Expr::number("8")),
                Expr::equal(Expr::identifier("DBL_MANT_DIG"), Expr::number("53")),
                Expr::equal(Expr::identifier("DBL_MAX_EXP"), Expr::number("1024")),
                Expr::equal(
                    Expr::identifier("DBL_MIN_EXP"),
                    Expr::negate(Expr::number("1021")),
                ),
            ]),
            "mal requires binary64 double",
        ),
        (
            Expr::equal(Expr::identifier("FLT_EVAL_METHOD"), Expr::number("0")),
            "mal requires evaluation in the operand format",
        ),
    ] {
        output.push(Declaration::static_assert(condition, message));
    }
    for (macro_name, message) in [
        ("FLT_HAS_SUBNORM", "mal requires float subnormals"),
        ("DBL_HAS_SUBNORM", "mal requires double subnormals"),
    ] {
        output.push(Directive::If(PreprocessorExpr::logical_and(
            PreprocessorExpr::defined(macro_name),
            PreprocessorExpr::not_equal(
                PreprocessorExpr::identifier(macro_name),
                PreprocessorExpr::integer(1),
            ),
        )));
        output.push(Directive::error(message));
        output.push(Directive::Endif);
    }
    output.blank_line();
    output
}

fn conjunction<const N: usize>(expressions: [syntax::Expr; N]) -> syntax::Expr {
    expressions
        .into_iter()
        .reduce(syntax::Expr::logical_and)
        .expect("conjunction requires at least one expression")
}

fn float_from_bits_definition(
    c_type: &str,
    name: &str,
    bits_type: &str,
) -> syntax::FunctionDefinition {
    use self::syntax::{Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement};

    let body = Block::new([
        Statement::variable(c_type, "value", None),
        Statement::expression(Expr::named_call(
            "memcpy",
            [
                Expr::address_of(Expr::identifier("value")),
                Expr::address_of(Expr::identifier("bits")),
                Expr::sizeof_expr(Expr::identifier("value")),
            ],
        )),
        Statement::return_value(Expr::identifier("value")),
    ]);
    FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_from_bits"),
            [Parameter::named(bits_type, "bits")],
        ),
        body,
    )
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
