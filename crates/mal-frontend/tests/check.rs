use mal_frontend::check;
use mal_frontend::check::ast::{ExpressionKind, TopItem, Type};
use mal_frontend::resolve;
use mal_syntax::parser::parse;
use mal_syntax::source::{FileId, SourceFile};

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

fn check_error(text: &str) -> mal_syntax::diagnostic::Diagnostic {
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

fn completion_value(completion: &check::ast::Completion) -> &check::ast::Expression {
    let check::ast::Completion::Value(value) = completion else {
        panic!("expected value completion");
    };
    value
}

#[path = "check/continuations.rs"]
mod continuations;
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
