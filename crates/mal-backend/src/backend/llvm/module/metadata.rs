use crate::backend::llvm::syntax::{MetadataDefinition, MetadataOperand, Type};

pub(super) fn buffer_alias() -> Vec<MetadataDefinition> {
    use MetadataOperand::{Integer, Node, Text};

    // Buffer object fields and element storage are distinct allocations. Their TBAA types preserve
    // that boundary across inlining. Runtime slot writes do not carry this metadata, so growth still
    // invalidates an active data pointer.
    vec![
        MetadataDefinition::new(0, false, [Text("Simple C/C++ TBAA".into())]),
        MetadataDefinition::new(
            1,
            false,
            [
                Text("omnipotent char".into()),
                Node(0),
                Integer {
                    ty: Type::integer(64_u16),
                    value: 0,
                },
            ],
        ),
        MetadataDefinition::new(
            2,
            false,
            [
                Text("mal buffer element storage".into()),
                Node(1),
                Integer {
                    ty: Type::integer(64_u16),
                    value: 0,
                },
            ],
        ),
        MetadataDefinition::new(
            3,
            false,
            [
                Node(2),
                Node(2),
                Integer {
                    ty: Type::integer(64_u16),
                    value: 0,
                },
            ],
        ),
        MetadataDefinition::new(
            4,
            true,
            [Node(4), Text("mal buffer object allocation".into())],
        ),
        MetadataDefinition::new(
            5,
            true,
            [Node(5), Node(4), Text("mal buffer object metadata".into())],
        ),
        MetadataDefinition::new(6, false, [Node(5)]),
        MetadataDefinition::new(
            7,
            false,
            [
                Text("mal buffer object field".into()),
                Node(1),
                Integer {
                    ty: Type::integer(64_u16),
                    value: 0,
                },
            ],
        ),
        MetadataDefinition::new(
            8,
            false,
            [
                Node(7),
                Node(7),
                Integer {
                    ty: Type::integer(64_u16),
                    value: 0,
                },
            ],
        ),
    ]
}
