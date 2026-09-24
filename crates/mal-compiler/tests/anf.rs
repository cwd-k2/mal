use mal_compiler::anf;
use mal_compiler::anf::ast::{AtomKind, Block, Lambda, Operation, Pattern};
use mal_compiler::check;
use mal_compiler::core;
use mal_compiler::core::ast::BinaryPrimitive;
use mal_compiler::resolve;
use mal_syntax::parser;
use mal_syntax::source::{FileId, SourceFile};

fn lower_ok(text: &str) -> anf::ast::Program {
    let source = SourceFile::new(FileId::new(53), "anf-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    anf::lower(&core)
}

fn top_lambda<'a>(program: &'a anf::ast::Program, name: &str) -> &'a Lambda {
    let binding = program
        .bindings
        .iter()
        .find(|binding| {
            matches!(&binding.pattern, anf::ast::TopLevelPattern::Binding { name: candidate, .. } if candidate == name)
        })
        .expect("named top-level binding");
    let block = &binding.value;
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
         main :: Unit -> Int32 := () -> {\n\
           left() + right();\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
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
fn evaluates_an_argument_before_its_callee_and_application() {
    let program = lower_ok(
        "create :: Unit -> (Unit -> Int32) := () -> {\n\
           () -> { 4; };\n\
         };\n\
         argument :: Unit -> Unit := () -> { (); };\n\
         main :: Unit -> Int32 := () -> {\n\
           create()(argument());\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
    let Operation::Call { callee, argument } = &bindings[2].operation else {
        panic!("expected the final application");
    };
    assert!(matches!(
        callee.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[1])
    ));
    assert!(matches!(
        argument.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[0])
    ));
}

#[test]
fn evaluates_a_receiver_before_remaining_receiver_call_arguments() {
    let program = lower_ok(
        "extern receiver :: Unit -> Int32;\n\
         extern argument :: Unit -> Int32;\n\
         combine :: (Int32, Int32) -> Int32 := (left, right) -> { left + right };\n\
         main :: Unit -> Int32 := () -> {\n\
           receiver().combine(argument());\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 4);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
    let Operation::Product(elements) = &bindings[2].operation else {
        panic!("expected the receiver and argument product");
    };
    assert!(matches!(
        elements[0].kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[0])
    ));
    assert!(matches!(
        elements[1].kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[1])
    ));
    let Operation::Call { argument, .. } = &bindings[3].operation else {
        panic!("expected the receiver-first application");
    };
    assert!(matches!(
        argument.kind,
        AtomKind::Reference(id) if id == binding_id(&bindings[2])
    ));
}

#[test]
fn keeps_buffer_operands_separate_and_in_source_order() {
    let program = lower_ok(
        "extern capacity :: Unit -> USize;\n\
         extern index :: Unit -> USize;\n\
         extern value :: Unit -> Int64;\n\
         main :: Unit -> Int32 := () -> {\n\
           make<Int64>(capacity()).put(index(), value());\n\
           0;\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 6);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Buffer { .. }));
    assert!(matches!(bindings[2].operation, Operation::Call { .. }));
    assert!(matches!(bindings[3].operation, Operation::Call { .. }));
    let Operation::Buffer { operands, .. } = &bindings[4].operation else {
        panic!("expected Buffer put after its three operands");
    };
    assert_eq!(operands.len(), 3);
    for (operand, binding) in operands
        .iter()
        .zip([&bindings[1], &bindings[2], &bindings[3]])
    {
        assert!(matches!(
            operand.kind,
            AtomKind::Reference(id) if id == binding_id(binding)
        ));
    }
    assert!(
        !bindings
            .iter()
            .any(|binding| matches!(binding.operation, Operation::Product(_)))
    );
}

#[test]
fn evaluates_buffer_range_operands_once_in_source_order() {
    let program = lower_ok(
        "destination :: Unit -> Buffer<Int64> := () -> make<Int64>(0usize);\n\
         extern destinationOffset :: Unit -> USize;\n\
         source :: Unit -> Buffer<Int64> := () -> make<Int64>(0usize);\n\
         extern sourceOffset :: Unit -> USize;\n\
         extern length :: Unit -> USize;\n\
         main :: Unit -> Int32 := () -> {\n\
           destination().copy(destinationOffset(), source(), sourceOffset(), length());\n\
           0;\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 7);
    for binding in &bindings[..5] {
        assert!(matches!(binding.operation, Operation::Call { .. }));
    }
    let Operation::Buffer { operands, .. } = &bindings[5].operation else {
        panic!("expected Buffer copy after its five operands");
    };
    assert_eq!(operands.len(), 5);
    for (operand, binding) in operands.iter().zip(&bindings[..5]) {
        assert!(matches!(
            operand.kind,
            AtomKind::Reference(id) if id == binding_id(binding)
        ));
    }
}

#[test]
fn evaluates_buffer_fill_value_once_in_source_order() {
    let program = lower_ok(
        "buffer :: Unit -> Buffer<Int64> := () -> make<Int64>(0usize);\n\
         extern offset :: Unit -> USize;\n\
         extern length :: Unit -> USize;\n\
         extern value :: Unit -> Int64;\n\
         main :: Unit -> Int32 := () -> {\n\
           buffer().fill(offset(), length(), value());\n\
           0;\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 6);
    for binding in &bindings[..4] {
        assert!(matches!(binding.operation, Operation::Call { .. }));
    }
    let Operation::Buffer { operands, .. } = &bindings[4].operation else {
        panic!("expected Buffer fill after its four operands");
    };
    assert_eq!(operands.len(), 4);
    for (operand, binding) in operands.iter().zip(&bindings[..4]) {
        assert!(matches!(
            operand.kind,
            AtomKind::Reference(id) if id == binding_id(binding)
        ));
    }
}

#[test]
fn keeps_case_arm_effects_inside_the_selected_arm() {
    let program = lower_ok(
        "extern mark :: Unit -> Int32;\n\
         choose :: Bool -> Int32 := (flag) -> {\n\
           if (flag) then { mark() } else { 0 };\n\
         };",
    );
    let body = &top_lambda(&program, "choose").body;
    assert_eq!(body.bindings.len(), 1);
    let Operation::Case { arms, .. } = &body.bindings[0].operation else {
        panic!("expected lowered if case");
    };
    assert!(arms[0].value.bindings.is_empty());
    assert_eq!(arms[1].value.bindings.len(), 1);
    assert!(matches!(
        arms[1].value.bindings[0].operation,
        Operation::Call { .. }
    ));
}

#[test]
fn orders_primitive_branch_operands_before_selected_arm_effects() {
    let program = lower_ok(
        "extern left :: Unit -> Int32;\n\
         extern right :: Unit -> Int32;\n\
         extern selected :: Unit -> Int32;\n\
         main :: Unit -> Int32 := () -> {\n\
           if (left() < right())\n\
             then { selected() }\n\
             else { 0 };\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
    let Operation::PrimitiveBranch { then, .. } = &bindings[2].operation else {
        panic!("expected primitive branch");
    };
    assert!(matches!(then.bindings[0].operation, Operation::Call { .. }));
}

#[test]
fn flattens_core_lets_without_losing_statement_order() {
    let program = lower_ok(
        "extern mark :: Unit -> Unit;\n\
         main :: Unit -> Int32 := () -> {\n\
           mark();\n\
           value :: Int32 := 7;\n\
           value;\n\
         };",
    );
    let Block {
        bindings, result, ..
    } = &top_lambda(&program, "main").body;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
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
         main :: Unit -> Int32 := () -> {\n\
           pair := (first(), second());\n\
           (left, right) := pair;\n\
           left + right;\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
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

#[test]
fn keeps_memory_operands_direct_and_evaluates_them_left_to_right() {
    let program = lower_ok(
        "create :: Unit -> Symbol := () -> { \"a\" };\n\
         extern index :: Unit -> USize;\n\
         main :: Unit -> Int32 := () -> {\n\
           (create() # index()).i32;\n\
         };",
    );
    let bindings = &top_lambda(&program, "main").body.bindings;
    assert!(matches!(bindings[0].operation, Operation::Call { .. }));
    assert!(matches!(bindings[1].operation, Operation::Call { .. }));
    assert!(
        bindings
            .iter()
            .any(|binding| matches!(binding.operation, Operation::SymbolAt { .. }))
    );
}
