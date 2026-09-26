use std::cell::Cell;

use crate::backend::llvm::syntax::{
    BinaryOperator, FunctionBuilder, FunctionSignature, Type, TypedValue,
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
        function.terminate(super::llvm_terminator!(return Type::integer(32_u16) => "0").unwrap())
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
    let trailing = [TypedValue::new(Type::integer(8_u16), "7").unwrap()];
    let instruction = super::llvm_instruction!(
        call Some("%result"), false, Type::integer(32_u16), direct "work"; [
            (typed Type::integer(32_u16) => "1"),
            { dynamic() },
            (typed_extend pairs()),
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
    let trailing = [super::llvm_typed_constant!(typed Type::integer(8_u16) => (atom 7)).unwrap()];
    let constant = super::llvm_constant!(structure [
        (typed Type::integer(32_u16) => (binary BinaryOperator::Add;
            (typed Type::integer(32_u16) => (atom 1));
            (typed Type::integer(32_u16) => (atom 2))
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
            super::llvm_terminator!(switch Type::integer(8_u16) => "%tag";
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
