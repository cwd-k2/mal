use malc::anf;
use malc::check;
use malc::closure;
use malc::closure::ast::{AtomKind, Function, Operation, Reference};
use malc::core;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn convert_ok(text: &str) -> closure::ast::Program {
    let source = SourceFile::new(FileId::new(67), "closure-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    let anf = anf::lower(&core);
    closure::convert(&anf)
}

fn closure_function_id(operation: &Operation) -> closure::ast::FunctionId {
    let Operation::MakeClosure { function, .. } = operation else {
        panic!("expected closure construction, found {operation:#?}");
    };
    *function
}

fn function(program: &closure::ast::Program, id: closure::ast::FunctionId) -> &Function {
    program
        .functions
        .iter()
        .find(|function| function.id == id)
        .expect("closure construction must name a lifted function")
}

fn capture_schema(function: &Function) -> &[closure::ast::CaptureField] {
    function
        .kind
        .captures()
        .expect("ordinary function should own a capture schema")
}

#[test]
fn lifts_capturing_lambdas_and_materializes_their_environment() {
    let program = convert_ok(
        "makeAdder :: Int32 -> (Int32 -> Int32) := (x) -> {\n\
           (y) -> { x + y; };\n\
         };",
    );
    let outer_id = closure_function_id(&program.bindings[0].value.bindings[0].operation);
    let outer = function(&program, outer_id);
    assert!(capture_schema(outer).is_empty());

    let Operation::MakeClosure {
        function: inner_id,
        captures,
    } = &outer.body.bindings[0].operation
    else {
        panic!("outer function should construct the returned closure");
    };
    assert_eq!(captures.len(), 1);
    assert!(matches!(
        captures[0].kind,
        AtomKind::Reference(Reference::Binding(id))
            if Some(id) == outer.parameter.binding
    ));

    let inner = function(&program, *inner_id);
    assert_eq!(capture_schema(inner).len(), 1);
    let Operation::PrimitiveBinary { left, right, .. } = &inner.body.bindings[0].operation else {
        panic!("expected the inner addition");
    };
    assert!(matches!(
        left.kind,
        AtomKind::Reference(Reference::Capture(0))
    ));
    assert!(matches!(
        right.kind,
        AtomKind::Reference(Reference::Binding(id))
            if Some(id) == inner.parameter.binding
    ));
}

#[test]
fn forwards_an_inferred_capture_through_every_lifted_function() {
    let program = convert_ok(
        "outer :: Int32 -> (Unit -> (Unit -> Int32)) := (x) -> {\n\
           () -> {\n\
             () -> { x; };\n\
           };\n\
         };",
    );
    let outer_id = closure_function_id(&program.bindings[0].value.bindings[0].operation);
    let outer = function(&program, outer_id);
    let Operation::MakeClosure {
        function: middle_id,
        captures: middle_captures,
    } = &outer.body.bindings[0].operation
    else {
        panic!("expected middle closure");
    };
    assert!(matches!(
        middle_captures[0].kind,
        AtomKind::Reference(Reference::Binding(id))
            if Some(id) == outer.parameter.binding
    ));

    let middle = function(&program, *middle_id);
    assert_eq!(capture_schema(middle).len(), 1);
    let Operation::MakeClosure {
        function: inner_id,
        captures: inner_captures,
    } = &middle.body.bindings[0].operation
    else {
        panic!("expected inner closure");
    };
    assert!(matches!(
        inner_captures[0].kind,
        AtomKind::Reference(Reference::Capture(0))
    ));

    let inner = function(&program, *inner_id);
    assert_eq!(capture_schema(inner).len(), 1);
    assert!(matches!(
        inner.body.result.kind,
        AtomKind::Reference(Reference::Capture(0))
    ));
}

#[test]
fn represents_capture_free_closures_without_environment_fields() {
    let program = convert_ok(
        "identity :: Int32 -> Int32 := (value) -> { value; };\n\
         main :: Unit -> Int32 := () -> { identity(5); };",
    );
    assert_eq!(program.functions.len(), 2);
    for binding in &program.bindings {
        let Operation::MakeClosure { captures, .. } = &binding.value.bindings[0].operation else {
            panic!("top-level lambda should become a closure");
        };
        assert!(captures.is_empty());
    }
    assert!(
        program
            .functions
            .iter()
            .all(|function| capture_schema(function).is_empty())
    );
}

#[test]
fn keeps_buffer_operations_direct_through_closure_conversion() {
    let program = convert_ok(
        "fill :: Buffer<Int64> -> Unit := (buffer) -> { index := buffer.new(1i64); buffer.put(index, buffer.get(index)); (); };\n\
         main :: Unit -> Int32 := () -> { first := pack<Int64>(fill); second := pack<Int64>(fill); ((first # 0usize) + (second # 0usize)).i32 - 2i32; };",
    );
    let buffer_operations = program
        .functions
        .iter()
        .flat_map(|function| &function.body.bindings)
        .filter(|binding| matches!(binding.operation, Operation::PackedBuilder { .. }))
        .count();
    assert_eq!(buffer_operations, 7);
}

#[test]
fn preserves_primitive_branches_while_lifting_functions() {
    let program = convert_ok(
        "choose :: Int32 -> Int32 := (value) -> {\n\
           if (value >= 0) then { value } else { 0 - value };\n\
         };",
    );
    let outer_id = closure_function_id(&program.bindings[0].value.bindings[0].operation);
    let outer = function(&program, outer_id);
    assert!(matches!(
        outer.body.bindings[0].operation,
        Operation::PrimitiveBranch { .. }
    ));
}

#[test]
fn represents_local_self_references_with_the_current_closure() {
    let program = convert_ok(
        "main :: Unit -> Int32 := () -> {\n\
           local :: Int32 -> Int32 := (n) -> {\n\
             if (n == 0) then { 0 } else { local(n - 1) };\n\
           };\n\
           local(3);\n\
         };",
    );
    let main_id = closure_function_id(&program.bindings[0].value.bindings[0].operation);
    let main = function(&program, main_id);
    let Operation::MakeClosure {
        function: local_id, ..
    } = &main.body.bindings[0].operation
    else {
        panic!("expected local recursive closure");
    };
    let local = function(&program, *local_id);
    let Operation::PrimitiveBranch { otherwise, .. } = &local.body.bindings[0].operation else {
        panic!("expected recursive conditional");
    };
    let recursive_call = otherwise
        .bindings
        .iter()
        .find_map(|binding| match &binding.operation {
            Operation::Call { callee, .. } => Some(callee),
            _ => None,
        })
        .expect("one branch should call itself");
    assert!(matches!(
        recursive_call.kind,
        AtomKind::Reference(Reference::SelfClosure(id)) if id == *local_id
    ));
}

#[test]
fn preserves_captured_products_and_destructuring_patterns() {
    let program = convert_ok(
        "make :: Unit -> (Unit -> Int32) := () -> {\n\
           pair := (20i32, 22i32);\n\
           () -> {\n\
             (left, right) := pair;\n\
             left + right;\n\
           };\n\
         };",
    );
    let outer_id = closure_function_id(&program.bindings[0].value.bindings[0].operation);
    let outer = function(&program, outer_id);
    let Operation::MakeClosure {
        function: inner_id, ..
    } = &outer.body.bindings[2].operation
    else {
        panic!("expected inner closure construction");
    };
    let inner = function(&program, *inner_id);
    assert_eq!(
        capture_schema(inner)[0].ty,
        malc::check::ast::Type::Product(
            vec![malc::check::ast::Type::Int32, malc::check::ast::Type::Int32,].into()
        )
    );
    assert!(matches!(
        inner.body.bindings[0].pattern,
        malc::closure::ast::Pattern::Product { .. }
    ));
}
