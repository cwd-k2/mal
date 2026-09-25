use mal_frontend::resolve;
use mal_frontend::resolve::ast::{
    self as resolved, FALSE_VALUE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, SYMBOL_TYPE,
    TopItem, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, ValueOwner,
};
use mal_syntax::ast;
use mal_syntax::parser::parse;
use mal_syntax::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(19), "resolve-test.mal", text.into())
}

fn resolve_ok(text: &str) -> resolved::Program {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)))
}

fn resolve_error(text: &str) -> mal_syntax::diagnostic::Diagnostic {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    resolve::resolve(&parsed).expect_err("name resolution should fail")
}

fn top_binding(item: &ast::Node<TopItem>) -> &resolved::Binding {
    let TopItem::Binding(binding) = &item.kind else {
        panic!("expected a top-level binding");
    };
    binding
}

#[path = "resolve/captures.rs"]
mod captures;
#[path = "resolve/continuations.rs"]
mod continuations;
#[path = "resolve/declarations.rs"]
mod declarations;
#[path = "resolve/files.rs"]
mod files;
#[path = "resolve/scopes.rs"]
mod scopes;
