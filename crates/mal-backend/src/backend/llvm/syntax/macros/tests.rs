use std::cell::Cell;

use crate::backend::llvm::syntax::{
    BinaryOperator, FunctionBuilder, FunctionSignature, Module, Type, TypedValue,
};

#[test]
fn composes_types_parameters_and_signatures_with_template_interpolation() {
    let name = "convert";
    let result = Type::integer(32_u16);
    let trailing = [super::llvm_parameter!("%value" : int(8))];
    let signature = super::llvm_signature!(internal fn { name }(
        "%context": ptr,
        {{ trailing }},
    ) -> { result }; attributes [nounwind, willreturn]);

    let mut function = FunctionBuilder::new(signature);
    assert!(function.start_block("entry"));
    assert!(
        function
            .terminate(super::llvm_terminator!(return { Type::integer(32_u16) } => "0").unwrap())
    );
    assert_eq!(
        function.finish().unwrap().render(),
        concat!(
            "define internal i32 @convert(ptr %context, i8 %value) nounwind willreturn {\n",
            "entry:\n",
            "  ret i32 0\n",
            "}"
        )
    );
}

#[test]
fn composes_module_leaf_definitions_with_the_same_template_boundaries() {
    let extra = [super::llvm_metadata_operand!(node 0)];
    let mut module = Module::new("test-target", "e-p:64:64");
    module.declare(super::llvm_declaration!(fn "observe"(
        _: ptr,
        _: int(8) [immarg],
    ) -> void; attributes [nounwind]));
    module.add_global(super::llvm_global!(byte_owner "message"; bytes { b"ok" }; align 1).unwrap());
    module.add_metadata([
        super::llvm_metadata!(0 => [(text "root")]),
        super::llvm_metadata!(distinct 1 => [
            (integer int(64) => 0),
            {{ extra }},
        ]),
    ]);

    assert_eq!(
        module.render().unwrap(),
        concat!(
            "target datalayout = \"e-p:64:64\"\n",
            "target triple = \"test-target\"\n\n",
            "declare void @observe(ptr, i8 immarg) nounwind\n\n",
            "@message = private constant { i64, i64, i8, [7 x i8], [2 x i8] } ",
            "{ i64 -1, i64 2, i8 0, [7 x i8] zeroinitializer, [2 x i8] c\"ok\" }, align 1\n\n",
            "!0 = !{!\"root\"}\n",
            "!1 = distinct !{i64 0, !0}\n",
        )
    );
}

#[test]
fn composes_static_embedded_and_runtime_typed_values_in_order() {
    let evaluations = Cell::new(0);
    let dynamic = || {
        evaluations.set(evaluations.get() + 1);
        TypedValue::new(Type::Pointer, "%dynamic").unwrap()
    };
    let pair_evaluations = Cell::new(0);
    let pairs = || {
        pair_evaluations.set(pair_evaluations.get() + 1);
        [(Type::integer(16_u16), "2")]
    };
    let pairs = pairs()
        .into_iter()
        .map(|(ty, value)| TypedValue::new(ty, value))
        .collect::<Option<Vec<_>>>()
        .unwrap();
    let trailing = [TypedValue::new(Type::integer(8_u16), "7").unwrap()];
    let instruction = super::llvm_instruction!(
        call { Some("%result".into()) }, false, (int(32_u16)), direct "work"; [
            (typed (int(32_u16)) => "1"),
            { dynamic() },
            {{ pairs }},
            {{ trailing }},
        ]
    )
    .unwrap();
    let mut rendered = String::new();
    instruction.render_into(&mut rendered);

    assert_eq!(evaluations.get(), 1);
    assert_eq!(pair_evaluations.get(), 1);
    assert_eq!(
        rendered,
        "%result = call i32 @work(i32 1, ptr %dynamic, i16 2, i8 7)"
    );
}

#[test]
fn composes_nested_constants_and_dynamic_fields() {
    let trailing =
        [super::llvm_typed_constant!(typed { Type::integer(8_u16) } => (atom 7)).unwrap()];
    let constant = super::llvm_constant!(structure [
        (typed { Type::integer(32_u16) } => (binary { BinaryOperator::Add };
            (typed { Type::integer(32_u16) } => (atom 1));
            (typed { Type::integer(32_u16) } => (atom 2))
        )),
        {{ trailing }},
    ])
    .unwrap();

    assert_eq!(constant.render(), "{ i32 add (i32 1, i32 2), i8 7 }");
}

#[test]
fn composes_switch_cases_and_finishes_the_function() {
    let trailing = [("1".to_string(), "one".to_string())];
    let mut function = FunctionBuilder::new(FunctionSignature::new(Type::Void, "choose", []));
    assert!(function.start_block("entry"));
    assert!(
        function.terminate(
            super::llvm_terminator!(switch { Type::integer(8_u16) } => "%tag";
                default "other";
                [
                    (case 0 => "zero"),
                    {{ trailing }},
                ]
            )
            .unwrap()
        )
    );
    for block in ["zero", "one", "other"] {
        assert!(function.start_block(block));
        assert!(function.terminate(super::llvm_terminator!(return_void).unwrap()));
    }

    assert!(function.finish().is_some());
}
