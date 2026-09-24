use mal_backend::core;
use mal_backend::core::ast::{
    BinaryPrimitive, Expression, ExpressionKind, JoinId, Lambda, Pattern, TopLevelPattern, ValueId,
};
use mal_frontend::check;
use mal_frontend::resolve;
use mal_syntax::parser;
use mal_syntax::source::{FileId, SourceFile};

fn lower_ok(text: &str) -> core::ast::Program {
    let source = SourceFile::new(FileId::new(41), "core-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"))
}

fn lambda_body(expression: &Expression) -> &Expression {
    &lambda(expression).body
}

fn lambda(expression: &Expression) -> &Lambda {
    let ExpressionKind::Lambda(lambda) = &expression.kind else {
        panic!("expected a lambda, found {:#?}", expression.kind);
    };
    lambda
}

fn top_lambda_definition<'a>(program: &'a core::ast::Program, name: &str) -> &'a Lambda {
    let binding = program
        .bindings
        .iter()
        .find(|binding| {
            matches!(&binding.pattern, TopLevelPattern::Binding { name: candidate, .. } if candidate == name)
        })
        .expect("named top-level binding");
    lambda(&binding.value)
}

fn top_lambda<'a>(program: &'a core::ast::Program, name: &str) -> &'a Expression {
    let binding = program
        .bindings
        .iter()
        .find(|binding| {
            matches!(&binding.pattern, TopLevelPattern::Binding { name: candidate, .. } if candidate == name)
        })
        .expect("named top-level binding");
    lambda_body(&binding.value)
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
