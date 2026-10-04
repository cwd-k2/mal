use super::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, Instruction,
    MetadataAttachment, Type, TypedValue,
};

#[test]
fn renders_alloca_from_typed_fields() {
    let instruction = Instruction::alloca(
        "%storage",
        Type::structure([Type::integer(32_u16), Type::Pointer]),
        8,
    )
    .unwrap();
    let mut output = String::new();
    instruction.render_into(&mut output);

    assert_eq!(output, "%storage = alloca { i32, ptr }, align 8");
    assert!(Instruction::alloca("%storage", Type::integer(32_u16), 0).is_none());
    assert!(Instruction::alloca("%storage", Type::integer(32_u16), 3).is_none());
}

#[test]
fn renders_memory_operations_and_metadata_from_fields() {
    let mut output = String::new();
    Instruction::load(
        "%value",
        Type::integer(32_u16),
        "%address",
        4,
        [MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)],
    )
    .unwrap()
    .render_into(&mut output);
    assert_eq!(
        output,
        "%value = load i32, ptr %address, align 4, !tbaa !3, !noalias !6"
    );

    output.clear();
    Instruction::store(Type::Pointer, "null", "%address", 8, std::iter::empty())
        .unwrap()
        .render_into(&mut output);
    assert_eq!(output, "store ptr null, ptr %address, align 8");
}

#[test]
fn renders_value_and_pointer_operations_from_typed_fields() {
    let mut rendered = Vec::new();
    for instruction in [
        Instruction::binary(
            "%sum",
            BinaryOperator::Add,
            Type::integer(32_u16),
            "%a",
            "1",
        )
        .unwrap(),
        Instruction::compare(
            "%small",
            ComparisonKind::Integer,
            ComparisonPredicate::Ult,
            Type::integer(32_u16),
            "%sum",
            "8",
        )
        .unwrap(),
        Instruction::cast(
            "%wide",
            CastOperator::ZExt,
            TypedValue::new(Type::integer(1_u16), "%small").unwrap(),
            Type::integer(8_u16),
        )
        .unwrap(),
        Instruction::get_element_ptr(
            "%field",
            true,
            Type::structure([Type::integer(32_u16), Type::Pointer]),
            "%storage",
            [
                TypedValue::new(Type::integer(32_u16), "0").unwrap(),
                TypedValue::new(Type::integer(32_u16), "1").unwrap(),
            ],
        )
        .unwrap(),
    ] {
        let mut output = String::new();
        instruction.render_into(&mut output);
        rendered.push(output);
    }

    assert_eq!(
        rendered,
        [
            "%sum = add i32 %a, 1",
            "%small = icmp ult i32 %sum, 8",
            "%wide = zext i1 %small to i8",
            "%field = getelementptr inbounds { i32, ptr }, ptr %storage, i32 0, i32 1",
        ]
    );
}

#[test]
fn instruction_macro_builds_typed_calls() {
    let arguments = TypedValue::from_pairs([(Type::Pointer, "%value".to_owned())]).unwrap();
    let instruction = super::super::llvm_instruction! {
        let "%result" = call {
            tail: false,
            result_type: int(8_u16),
            callee: direct("observe"),
            arguments: [..{ arguments }],
        };
    }
    .unwrap();
    let mut output = String::new();
    instruction.render_into(&mut output);

    assert_eq!(output, "%result = call i8 @observe(ptr %value)");
}

#[test]
fn rejects_fragments_in_atom_only_positions() {
    assert!(super::Callee::indirect("%callee = bitcast ptr %other to ptr").is_none());
    assert!(TypedValue::new(Type::integer(32_u16), "1; hidden instruction").is_none());
}
