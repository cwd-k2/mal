use malc::anf;
use malc::anf::ast::{AtomKind, Block, Lambda, Operation, Pattern};
use malc::check;
use malc::core;
use malc::core::ast::BinaryPrimitive;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn lower_ok(text: &str) -> anf::ast::Program {
    let source = SourceFile::new(FileId::new(53), "anf-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&checked);
    anf::lower(&core)
}

fn top_lambda(program: &anf::ast::Program, index: usize) -> &Lambda {
    let block = &program.bindings[index].value;
    assert_eq!(block.bindings.len(), 1);
    let Operation::Lambda(lambda) = &block.bindings[0].operation else {
        panic!("expected a top-level lambda");
    };
    lambda
}

fn binding_id(binding: &anf::ast::Binding) -> anf::ast::ValueId {
    let Pattern::Binding { id, .. } = binding.pattern else {
        panic!("expected a value binding");
    };
    id
}

#[test]
fn orders_primitive_operands_left_to_right() {
    let program = lower_ok(
        "extern left :: Unit -> Int32;\n\
         extern right :: Unit -> Int32;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern left() + extern right();\n\
         };",
    );
    let bindings = &top_lambda(&program, 0).body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(
        bindings[0].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[0].id
    ));
    assert!(matches!(
        bindings[1].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[1].id
    ));
    let Operation::PrimitiveBinary {
        operator: BinaryPrimitive::Add,
        left,
        right,
    } = &bindings[2].operation
    else {
        panic!("expected the addition after both operands");
    };
    assert!(matches!(
        left.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[0])
    ));
    assert!(matches!(
        right.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[1])
    ));
}

#[test]
fn evaluates_a_callee_before_its_argument_and_application() {
    let program = lower_ok(
        "make :: Unit -> (Unit -> Int32) := \\() {\n\
           \\() { 4; };\n\
         };\n\
         argument :: Unit -> Unit := \\() { (); };\n\
         main :: Unit -> Int32 := \\() {\n\
           make()(argument());\n\
         };",
    );
    let bindings = &top_lambda(&program, 2).body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
    let Operation::Call { callee, argument } = &bindings[2].operation else {
        panic!("expected the final application");
    };
    assert!(matches!(
        callee.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[0])
    ));
    assert!(matches!(
        argument.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[1])
    ));
}

#[test]
fn keeps_case_arm_effects_inside_the_selected_arm() {
    let program = lower_ok(
        "extern mark :: Unit -> Int32;\n\
         choose :: Bool -> Int32 := \\(flag :: Bool) {\n\
           if (flag) then { extern mark() } else { 0 };\n\
         };",
    );
    let body = &top_lambda(&program, 0).body;
    assert_eq!(body.bindings.len(), 1);
    let Operation::Case { arms, .. } = &body.bindings[0].operation else {
        panic!("expected lowered if case");
    };
    assert!(arms[0].value.bindings.is_empty());
    assert_eq!(arms[1].value.bindings.len(), 1);
    assert!(matches!(
        arms[1].value.bindings[0].operation,
        Operation::ExternalCall { .. }
    ));
}

#[test]
fn orders_primitive_branch_operands_before_selected_arm_effects() {
    let program = lower_ok(
        "extern left :: Unit -> Int32;\n\
         extern right :: Unit -> Int32;\n\
         extern selected :: Unit -> Int32;\n\
         main :: Unit -> Int32 := \\() {\n\
           if (extern left() < extern right())\n\
             then { extern selected() }\n\
             else { 0 };\n\
         };",
    );
    let bindings = &top_lambda(&program, 0).body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(
        bindings[0].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[0].id
    ));
    assert!(matches!(
        bindings[1].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[1].id
    ));
    let Operation::PrimitiveBranch { then, .. } = &bindings[2].operation else {
        panic!("expected primitive branch");
    };
    assert!(matches!(
        then.bindings[0].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[2].id
    ));
}

#[test]
fn flattens_core_lets_without_losing_statement_order() {
    let program = lower_ok(
        "extern mark :: Unit -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern mark();\n\
           value :: Int32 := 7;\n\
           value;\n\
         };",
    );
    let Block {
        bindings, result, ..
    } = &top_lambda(&program, 0).body;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(
        bindings[0].operation,
        Operation::ExternalCall { .. }
    ));
    assert!(matches!(bindings[1].pattern, Pattern::Wildcard { .. }));
    assert!(matches!(
        bindings[2].operation,
        Operation::Atom(anf::ast::Atom {
            kind: AtomKind::Integer(7),
            ..
        })
    ));
    assert!(matches!(
        result.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[2])
    ));
}

#[test]
fn evaluates_product_elements_left_to_right_before_construction() {
    let program = lower_ok(
        "extern first :: Unit -> Int32;\n\
         extern second :: Unit -> Int32;\n\
         main :: Unit -> Int32 := \\() {\n\
           pair := (extern first(), extern second());\n\
           (left, right) := pair;\n\
           left + right;\n\
         };",
    );
    let bindings = &top_lambda(&program, 0).body.bindings;
    assert!(matches!(
        bindings[0].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[0].id
    ));
    assert!(matches!(
        bindings[1].operation,
        Operation::ExternalCall { id, .. } if id == program.externals[1].id
    ));
    let Operation::Product(elements) = &bindings[2].operation else {
        panic!("expected product construction after its elements");
    };
    assert!(matches!(
        elements[0].kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[0])
    ));
    assert!(matches!(
        elements[1].kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[1])
    ));
}
