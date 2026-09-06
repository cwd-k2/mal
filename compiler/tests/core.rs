use malc::check;
use malc::core;
use malc::core::ast::{
    BinaryPrimitive, Expression, ExpressionKind, Pattern, TopLevelPattern, ValueId,
};
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn lower_ok(text: &str) -> core::ast::Program {
    let source = SourceFile::new(FileId::new(41), "core-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    core::lower(&checked)
}

fn lambda_body(expression: &Expression) -> &Expression {
    let ExpressionKind::Lambda(lambda) = &expression.kind else {
        panic!("expected a lambda, found {:#?}", expression.kind);
    };
    &lambda.body
}

fn case(expression: &Expression) -> (&Expression, &[core::ast::CaseArm]) {
    let ExpressionKind::Case { scrutinee, arms } = &expression.kind else {
        panic!("expected a case, found {:#?}", expression.kind);
    };
    (scrutinee, arms)
}

fn injected_bool(expression: &Expression) -> bool {
    let ExpressionKind::SumInjection { index, value } = &expression.kind else {
        panic!("expected a Bool injection, found {:#?}", expression.kind);
    };
    assert!(matches!(value.kind, ExpressionKind::Unit));
    *index == 1
}

#[path = "core/boolean.rs"]
mod boolean;
#[path = "core/functions.rs"]
mod functions;
#[path = "core/interface.rs"]
mod interface;
