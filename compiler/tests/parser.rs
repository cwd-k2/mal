use malc::ast::{
    BinaryOperator, BodyItem, Expression, Pattern, TopItem, TypeExpression, UnaryOperator,
};
use malc::parser::parse;
use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(11), "parser-test.mal", text.into())
}

fn parse_ok(text: &str) -> malc::ast::Program {
    parse(&source(text)).unwrap_or_else(|error| panic!("{}", error.render(&source(text))))
}

fn binding_value(text: &str) -> Expression {
    let mut program = parse_ok(text);
    let TopItem::Binding(binding) = program.items.remove(0).kind else {
        panic!("expected a binding");
    };
    binding.value.kind
}

#[path = "parser/atoms.rs"]
mod atoms;
#[path = "parser/declarations.rs"]
mod declarations;
#[path = "parser/forms.rs"]
mod forms;
#[path = "parser/operators.rs"]
mod operators;
