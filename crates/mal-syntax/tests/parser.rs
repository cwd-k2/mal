use mal_syntax::ast::{
    BinaryOperator, BodyItem, Expression, Pattern, TopItem, TypeExpression, UnaryOperator,
};
use mal_syntax::parser::parse;
use mal_syntax::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(11), "parser-test.mal", text.into())
}

fn parse_ok(text: &str) -> mal_syntax::ast::Program {
    parse(&source(text)).unwrap_or_else(|error| panic!("{}", error.render(&source(text))))
}

fn binding_value(text: &str) -> Expression {
    let mut program = parse_ok(text);
    let TopItem::Binding(binding) = program.items.remove(0).kind else {
        panic!("expected a binding");
    };
    binding.value.kind
}

#[test]
fn rejects_excessive_nesting() {
    let excessive = format!(
        "main :: Unit -> Int32 := () -> {{ {}0i32{}; }};",
        "(".repeat(300),
        ")".repeat(300)
    );
    let error = parse(&source(&excessive)).expect_err("excessive nesting must be rejected");
    assert_eq!(error.message, "syntax nesting limit exceeded");
    assert!(
        error
            .primary
            .expect("nesting diagnostic location")
            .message
            .contains("at most 64 levels")
    );
}

#[path = "parser/atoms.rs"]
mod atoms;
#[path = "parser/declarations.rs"]
mod declarations;
#[path = "parser/forms.rs"]
mod forms;
#[path = "parser/operators.rs"]
mod operators;
