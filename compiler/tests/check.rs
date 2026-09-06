use malc::check;
use malc::check::ast::{ExpressionKind, TopItem, Type};
use malc::parser::parse;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(23), "check-test.mal", text.into())
}

fn check_ok(text: &str) -> check::ast::Program {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)))
}

fn check_error(text: &str) -> malc::diagnostic::Diagnostic {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    check::check(&resolved).expect_err("type checking should fail")
}

fn top_binding(program: &check::ast::Program, index: usize) -> &check::ast::Binding {
    let TopItem::Binding(binding) = &program.items[index].kind else {
        panic!("expected a top-level binding");
    };
    binding
}

#[path = "check/control.rs"]
mod control;
#[path = "check/declarations.rs"]
mod declarations;
#[path = "check/functions.rs"]
mod functions;
#[path = "check/memory.rs"]
mod memory;
#[path = "check/numeric.rs"]
mod numeric;
#[path = "check/symbol.rs"]
mod symbol;
