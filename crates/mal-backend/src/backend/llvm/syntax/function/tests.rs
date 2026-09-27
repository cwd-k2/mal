use super::*;
use crate::backend::llvm::syntax::Type;

#[test]
fn renders_late_entry_instructions_before_the_existing_entry_body() {
    let mut function = FunctionBuilder::new(
        FunctionSignature::new(Type::Void, "example", std::iter::empty::<Parameter>())
            .with_linkage(Linkage::Internal),
    );
    assert!(function.start_block("entry"));
    assert!(function.terminate(Terminator::branch("body").unwrap()));
    assert!(function.start_block("body"));
    assert!(function.terminate(Terminator::return_void()));
    function.structured_entry_instruction(
        Instruction::alloca("%storage", Type::integer(32_u16), 4).unwrap(),
    );

    assert_eq!(
        function.finish().unwrap().render(),
        concat!(
            "define internal void @example() {\n",
            "entry:\n",
            "  %storage = alloca i32, align 4\n",
            "  br label %body\n",
            "body:\n",
            "  ret void\n",
            "}",
        )
    );
}

#[test]
fn rejects_missing_terminators_and_instructions_after_a_terminator() {
    let mut function = FunctionBuilder::new(FunctionSignature::new(
        Type::Void,
        "missing_terminator",
        std::iter::empty::<Parameter>(),
    ));
    assert!(function.start_block("entry"));
    assert!(
        function.structured_instruction(
            Instruction::call(
                None::<String>,
                false,
                Type::Void,
                super::super::Callee::direct("work").unwrap(),
                [],
            )
            .unwrap()
        )
    );
    assert!(function.finish().is_none());
}

#[test]
fn rejects_invalid_and_duplicate_block_labels() {
    assert!(
        BasicBlock::new(
            "0invalid",
            std::iter::empty::<Instruction>(),
            Terminator::return_void(),
        )
        .is_none()
    );

    let mut function = FunctionBuilder::new(FunctionSignature::new(
        Type::Void,
        "duplicate",
        std::iter::empty::<Parameter>(),
    ));
    assert!(function.start_block("entry"));
    assert!(function.terminate(Terminator::return_void()));
    assert!(!function.start_block("entry"));

    let entry = BasicBlock::new(
        "entry",
        std::iter::empty::<Instruction>(),
        Terminator::return_void(),
    )
    .unwrap();
    assert!(
        FunctionDefinition::new(
            FunctionSignature::new(
                Type::Void,
                "duplicate_direct",
                std::iter::empty::<Parameter>(),
            ),
            vec![entry.clone(), entry],
        )
        .is_none()
    );
}

#[test]
fn derives_byte_runtime_requirements_from_emitted_instructions() {
    let signature =
        || FunctionSignature::new(Type::Void, "example", std::iter::empty::<Parameter>());
    let call = |callee, arguments: Vec<super::super::TypedValue>| {
        Instruction::call(
            None::<String>,
            false,
            Type::Void,
            super::super::Callee::direct(callee).unwrap(),
            arguments,
        )
        .unwrap()
    };
    let mut plain = FunctionBuilder::new(signature());
    assert!(plain.start_block("entry"));
    assert!(plain.structured_instruction(call("work", Vec::new())));
    assert!(plain.terminate(Terminator::return_void()));
    let plain = plain.finish().unwrap();

    let mut bytes = FunctionBuilder::new(signature());
    assert!(bytes.start_block("entry"));
    assert!(bytes.structured_instruction(call(
        "mal_runtime_bytes_release",
        vec![super::super::TypedValue::new(Type::Pointer, "null").unwrap()],
    )));
    assert!(bytes.terminate(Terminator::return_void()));
    let bytes = bytes.finish().unwrap();

    assert!(!plain.uses_byte_runtime());
    assert!(bytes.uses_byte_runtime());
}

#[test]
fn rejects_invalid_function_signature_fragments() {
    let block = || {
        BasicBlock::new(
            "entry",
            std::iter::empty::<Instruction>(),
            Terminator::return_void(),
        )
        .unwrap()
    };
    assert!(
        FunctionDefinition::new(
            FunctionSignature::new(Type::Void, "0invalid", std::iter::empty::<Parameter>()),
            vec![block()],
        )
        .is_none()
    );
    assert!(
        FunctionDefinition::new(
            FunctionSignature::new(
                Type::Void,
                "invalid",
                [Parameter::named(Type::Pointer, "%bad\nname")],
            ),
            vec![block()],
        )
        .is_none()
    );
}

#[test]
fn rejects_fragments_in_control_operands() {
    assert!(Terminator::conditional_branch("%condition, label %extra", "yes", "no").is_none());
    assert!(
        Terminator::switch(
            Type::integer(32_u16),
            "%tag",
            "other",
            [("0, label %injected", "zero")],
        )
        .is_none()
    );
}
