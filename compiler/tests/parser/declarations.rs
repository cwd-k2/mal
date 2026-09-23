use super::*;

#[test]
fn parses_requirements_before_top_level_items() {
    let program = parse_ok("require \"./support.mal\";\nrequire \"./host.c\";\n_private := 1;\n");

    assert_eq!(program.requirements.len(), 2);
    assert_eq!(program.requirements[0].kind.path, b"./support.mal");
    assert_eq!(program.requirements[1].kind.path, b"./host.c");
    let TopItem::Binding(binding) = &program.items[0].kind else {
        panic!("expected a private binding");
    };
    let Pattern::Name(name) = &binding.pattern.kind else {
        panic!("expected a name pattern");
    };
    assert_eq!(name.text, "_private");
}

#[test]
fn rejects_requirements_after_top_level_items() {
    let source = source("value := 1; require \"./late.mal\";");
    let error = parse(&source).expect_err("late requirement should be rejected");

    assert_eq!(error.message, "require declaration after a top-level item");
}

#[test]
fn parses_the_basic_host_example() {
    let program = parse_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := () -> {\n\
           printInt32(42);\n\
           0;\n\
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
    assert!(lambda.parameter.is_none());
    assert_eq!(lambda.body.items.len(), 1);
    assert!(matches!(
        lambda.body.items[0],
        BodyItem::Expression(malc::ast::Node {
            kind: Expression::Call { .. },
            ..
        })
    ));
}

#[test]
fn rejects_extern_at_a_call_site() {
    let source = source(
        "extern print :: UInt8 -> Unit; main :: Unit -> Unit := () -> { extern print(1u8) };",
    );
    let error = parse(&source).expect_err("call-site extern should be rejected");

    assert_eq!(error.message, "expected an expression");
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
fn rejects_missing_top_level_semicolon_at_eof() {
    let source = source("value := 1");
    let error = parse(&source).expect_err("semicolon is required");

    assert_eq!(error.message, "expected `;`");
    assert!(error.render(&source).contains("parser-test.mal:1:11"));
}

#[test]
fn parses_generic_aliases_bindings_and_nested_indexed_types() {
    let program = parse_ok(
        "Pair<A> :: (A, A);\n\
         identity<A> :: A -> A := (value) -> value;\n\
         read :: Buffer<(Address, USize)> -> Buffer<(Address, USize)> := (values) -> identity<Buffer<(Address, USize)>>(values);",
    );

    let TopItem::GenericTypeAlias {
        parameters, value, ..
    } = &program.items[0].kind
    else {
        panic!("expected a generic alias");
    };
    assert_eq!(
        parameters
            .iter()
            .map(|name| name.text.as_str())
            .collect::<Vec<_>>(),
        ["A"]
    );
    assert!(matches!(value.kind, TypeExpression::Product(_)));

    let TopItem::GenericBinding { parameters, .. } = &program.items[1].kind else {
        panic!("expected a generic binding");
    };
    assert_eq!(parameters[0].text, "A");

    let TopItem::Binding(read) = &program.items[2].kind else {
        panic!("expected a monomorphic binding");
    };
    let Some(annotation) = &read.annotation else {
        panic!("expected an annotation")
    };
    let TypeExpression::Function { parameter, .. } = &annotation.kind else {
        panic!("expected a function type");
    };
    assert!(matches!(parameter.kind, TypeExpression::Application { .. }));
}
