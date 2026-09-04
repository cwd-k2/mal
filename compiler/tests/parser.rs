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

#[test]
fn parses_the_m0_host_example() {
    let program = parse_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           return 0;\n\
         };",
    );

    assert_eq!(program.items.len(), 2);
    let TopItem::ExternalOperation { name, ty } = &program.items[0].kind else {
        panic!("expected an external operation");
    };
    assert_eq!(name.text, "printInt32");
    assert!(matches!(ty.kind, TypeExpression::Function { .. }));

    let TopItem::Binding(main) = &program.items[1].kind else {
        panic!("expected main binding");
    };
    let Expression::Lambda(lambda) = &main.value.kind else {
        panic!("expected main lambda");
    };
    assert!(lambda.parameters.is_empty());
    assert_eq!(lambda.body.items.len(), 1);
    assert!(matches!(
        lambda.body.items[0],
        BodyItem::Expression(malc::ast::Node {
            kind: Expression::ExternalCall { .. },
            ..
        })
    ));
}

#[test]
fn function_types_are_right_associative() {
    let program = parse_ok("Compose :: Int32 -> Unit -> [Unit, Int32];");
    let TopItem::TypeAlias { value, .. } = &program.items[0].kind else {
        panic!("expected a type alias");
    };
    let TypeExpression::Function { parameter, result } = &value.kind else {
        panic!("expected outer function type");
    };
    assert!(matches!(parameter.kind, TypeExpression::Named(_)));
    let TypeExpression::Function { result, .. } = &result.kind else {
        panic!("expected right-associated result");
    };
    assert!(matches!(result.kind, TypeExpression::Sum(ref members) if members.len() == 2));
}

#[test]
fn pratt_parser_preserves_precedence_and_left_associativity() {
    let expression = binding_value("value := 1 + 2 * 3 - 4;");
    let Expression::Binary {
        operator,
        left,
        right,
    } = expression
    else {
        panic!("expected subtraction");
    };
    assert_eq!(operator.kind, BinaryOperator::Subtract);
    assert!(matches!(right.kind, Expression::Integer(_)));
    let Expression::Binary {
        operator, right, ..
    } = left.kind
    else {
        panic!("expected addition on the left");
    };
    assert_eq!(operator.kind, BinaryOperator::Add);
    assert!(matches!(
        right.kind,
        Expression::Binary {
            operator: malc::ast::Node {
                kind: BinaryOperator::Multiply,
                ..
            },
            ..
        }
    ));
}

#[test]
fn calls_bind_more_tightly_than_unary_operators() {
    let expression = binding_value("value := -make()(1);");
    let Expression::Unary { operator, operand } = expression else {
        panic!("expected unary expression");
    };
    assert_eq!(operator.kind, UnaryOperator::Negate);
    let Expression::Call { callee, .. } = operand.kind else {
        panic!("expected outer call");
    };
    assert!(matches!(callee.kind, Expression::Call { .. }));
}

#[test]
fn parses_a_byte_literal_as_an_atomic_expression() {
    assert_eq!(binding_value(r"value := b'\xff';"), Expression::Byte(255));
}

#[test]
fn distinguishes_numeric_conversion_from_sum_injection() {
    assert!(matches!(
        binding_value("value := UInt8(1Int8);"),
        Expression::Conversion { .. }
    ));
    assert!(matches!(
        binding_value("value := Maybe[1](1);"),
        Expression::SumInjection { .. }
    ));
}

#[test]
fn parses_captures_parameters_and_lambda_body_items() {
    let expression = binding_value(
        "make := \\<outer>(x :: Int32, y :: Int32) {\n\
           sum :: Int32 := x + y;\n\
           extern observe(sum);\n\
           return outer(sum);\n\
         };",
    );
    let Expression::Lambda(lambda) = expression else {
        panic!("expected lambda");
    };
    assert_eq!(lambda.captures[0].text, "outer");
    assert_eq!(lambda.parameters.len(), 2);
    assert!(matches!(lambda.body.items[0], BodyItem::Binding(_)));
    assert!(matches!(lambda.body.items[1], BodyItem::Expression(_)));
    assert!(matches!(lambda.body.result.kind, Expression::Call { .. }));
}

#[test]
fn parses_if_blocks_with_local_bindings() {
    let expression = binding_value(
        "value := if (condition) then {\n\
           x := 1;\n\
           x\n\
         } else {\n\
           2\n\
         };",
    );
    let Expression::If {
        then_branch,
        else_branch,
        ..
    } = expression
    else {
        panic!("expected if expression");
    };
    assert_eq!(then_branch.items.len(), 1);
    assert!(matches!(then_branch.result.kind, Expression::Name(_)));
    assert!(matches!(else_branch.result.kind, Expression::Integer(_)));
}

#[test]
fn parses_sum_injection_and_case_arms() {
    let expression = binding_value(
        "value := case MaybeInt32[1](42) {\n\
           [0](_) => 0;\n\
           [1](x) => x;\n\
         };",
    );
    let Expression::Case { scrutinee, arms } = expression else {
        panic!("expected case expression");
    };
    assert!(matches!(scrutinee.kind, Expression::SumInjection { .. }));
    assert_eq!(arms.len(), 2);
    assert!(matches!(arms[0].pattern.kind, Pattern::Wildcard));
    assert!(matches!(arms[1].pattern.kind, Pattern::Name(_)));
}

#[test]
fn rejects_non_associative_operator_chains() {
    for text in ["value := a < b <= c;", "value := a == b != c;"] {
        let source = source(text);
        let error = parse(&source).expect_err("comparison chain should be rejected");
        assert_eq!(error.message, "non-associative operator chain");
    }
}

#[test]
fn rejects_single_member_sums_and_trailing_commas() {
    for text in [
        "Only :: [Unit];",
        "Pair :: [Unit, Int32,];",
        "value := f(1,);",
        "value := \\<>() { return 0; };",
    ] {
        assert!(parse(&source(text)).is_err(), "input should fail: {text}");
    }
}

#[test]
fn rejects_lambda_without_terminal_return() {
    let source = source("value := \\() { extern run(); };");
    let error = parse(&source).expect_err("terminal return is required");

    assert_eq!(error.message, "expected a terminal `return`");
    assert!(error.render(&source).contains("parser-test.mal:1:30"));
}

#[test]
fn rejects_missing_top_level_semicolon_at_eof() {
    let source = source("value := 1");
    let error = parse(&source).expect_err("semicolon is required");

    assert_eq!(error.message, "expected `;`");
    assert!(error.render(&source).contains("parser-test.mal:1:11"));
}
