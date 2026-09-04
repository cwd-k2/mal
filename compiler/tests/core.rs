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

#[test]
fn removes_type_aliases_and_preserves_backend_names() {
    let program = lower_ok(
        "Flag :: [Unit, Unit];\n\
         extern choose :: Flag -> Int32;\n\
         main :: Unit -> Int32 := \\() { return 0; };",
    );

    assert_eq!(program.externals.len(), 1);
    assert_eq!(program.externals[0].name, "choose");
    assert_eq!(program.bindings.len(), 1);
    let TopLevelPattern::Binding { name, .. } = &program.bindings[0].pattern else {
        panic!("expected named top-level binding");
    };
    assert_eq!(name, "main");
}

#[test]
fn lowers_if_to_false_then_true_case_arms() {
    let program = lower_ok(
        "choose :: Bool -> Int32 := \\(flag :: Bool) {\n\
           return if (flag) then { value :: Int32 := 1; value } else { 0 };\n\
         };",
    );
    let body = lambda_body(&program.bindings[0].value);
    let (_, arms) = case(body);

    assert_eq!(arms.iter().map(|arm| arm.index).collect::<Vec<_>>(), [0, 1]);
    assert!(matches!(arms[0].value.kind, ExpressionKind::Integer(0)));
    assert!(matches!(arms[1].value.kind, ExpressionKind::Let { .. }));
}

#[test]
fn lowers_short_circuit_operators_without_eager_right_evaluation() {
    let and_program = lower_ok(
        "extern observe :: Bool -> Bool;\n\
         test :: Bool -> Bool := \\(flag :: Bool) {\n\
           return flag && extern observe(flag);\n\
         };",
    );
    let body = lambda_body(&and_program.bindings[0].value);
    let (_, arms) = case(body);
    assert!(!injected_bool(&arms[0].value));
    assert!(matches!(
        arms[1].value.kind,
        ExpressionKind::ExternalCall { .. }
    ));

    let or_program = lower_ok(
        "extern observe :: Bool -> Bool;\n\
         test :: Bool -> Bool := \\(flag :: Bool) {\n\
           return flag || extern observe(flag);\n\
         };",
    );
    let body = lambda_body(&or_program.bindings[0].value);
    let (_, arms) = case(body);
    assert!(matches!(
        arms[0].value.kind,
        ExpressionKind::ExternalCall { .. }
    ));
    assert!(injected_bool(&arms[1].value));
}

#[test]
fn removes_logical_not_but_retains_typed_numeric_primitives() {
    let program = lower_ok(
        "test :: Int32 -> Bool := \\(value :: Int32) {\n\
           return !(value + 1 < 3);\n\
         };",
    );
    let body = lambda_body(&program.bindings[0].value);
    let (comparison, arms) = case(body);
    assert_eq!(arms.len(), 2);
    assert!(injected_bool(&arms[0].value));
    assert!(!injected_bool(&arms[1].value));

    let ExpressionKind::PrimitiveBinary {
        operator: BinaryPrimitive::Less,
        left,
        ..
    } = &comparison.kind
    else {
        panic!("expected an Int32 comparison");
    };
    assert!(matches!(
        left.kind,
        ExpressionKind::PrimitiveBinary {
            operator: BinaryPrimitive::Add,
            ..
        }
    ));
}

#[test]
fn binds_both_bool_equality_operands_once_before_branching() {
    let program = lower_ok(
        "extern first :: Unit -> Bool;\n\
         extern second :: Unit -> Bool;\n\
         test :: Unit -> Bool := \\() {\n\
           return extern first() == extern second();\n\
         };",
    );
    let first_let = lambda_body(&program.bindings[0].value);
    let ExpressionKind::Let {
        binding: first,
        body: second_let,
    } = &first_let.kind
    else {
        panic!("left operand should be bound first");
    };
    assert!(matches!(
        first.value.kind,
        ExpressionKind::ExternalCall { id, .. } if id == program.externals[0].id
    ));
    let Pattern::Binding { id: first_id, .. } = first.pattern else {
        panic!("expected a synthetic left binding");
    };

    let ExpressionKind::Let {
        binding: second,
        body: comparison,
    } = &second_let.kind
    else {
        panic!("right operand should be bound second");
    };
    assert!(matches!(
        second.value.kind,
        ExpressionKind::ExternalCall { id, .. } if id == program.externals[1].id
    ));
    let Pattern::Binding { id: second_id, .. } = second.pattern else {
        panic!("expected a synthetic right binding");
    };
    assert!(matches!(first_id, ValueId::Temporary(_)));
    assert!(matches!(second_id, ValueId::Temporary(_)));

    let (left_reference, arms) = case(comparison);
    assert!(matches!(
        left_reference.kind,
        ExpressionKind::Reference(id) if id == first_id
    ));
    for arm in arms {
        let (right_reference, _) = case(&arm.value);
        assert!(matches!(
            right_reference.kind,
            ExpressionKind::Reference(id) if id == second_id
        ));
    }
}

#[test]
fn lowers_lambda_statements_and_terminal_return_to_lets_and_a_result() {
    let program = lower_ok(
        "extern mark :: Unit -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern mark();\n\
           value :: Int32 := 7;\n\
           return (value);\n\
         };",
    );
    let first_let = lambda_body(&program.bindings[0].value);
    let ExpressionKind::Let {
        binding: statement,
        body: second_let,
    } = &first_let.kind
    else {
        panic!("expected the expression statement let");
    };
    assert!(matches!(statement.pattern, Pattern::Wildcard { .. }));
    assert!(matches!(
        statement.value.kind,
        ExpressionKind::ExternalCall { .. }
    ));

    let ExpressionKind::Let {
        binding,
        body: result,
    } = &second_let.kind
    else {
        panic!("expected the local binding let");
    };
    let Pattern::Binding { id, .. } = binding.pattern else {
        panic!("expected a named local binding");
    };
    assert!(matches!(
        result.kind,
        ExpressionKind::Reference(result_id) if result_id == id
    ));
}
