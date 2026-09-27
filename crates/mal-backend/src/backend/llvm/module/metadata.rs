use crate::backend::llvm::syntax::{MetadataDefinition, llvm_metadata};

pub(super) fn buffer_alias() -> Vec<MetadataDefinition> {
    // Buffer object fields and element storage are distinct allocations. Their TBAA types preserve
    // that boundary across inlining. Runtime slot writes do not carry this metadata, so growth still
    // invalidates an active data pointer.
    vec![
        llvm_metadata!({ id: 0, distinct: false, operands: [text("Simple C/C++ TBAA")] }),
        llvm_metadata! {
            {
                id: 1,
                distinct: false,
                operands: [text("omnipotent char"), node(0), integer(int(64), 0)],
            }
        },
        llvm_metadata! {
            {
                id: 2,
                distinct: false,
                operands: [text("mal buffer element storage"), node(1), integer(int(64), 0)],
            }
        },
        llvm_metadata! {
            {
                id: 3,
                distinct: false,
                operands: [node(2), node(2), integer(int(64), 0)],
            }
        },
        llvm_metadata! {
            {
                id: 4,
                distinct: true,
                operands: [node(4), text("mal buffer object allocation")],
            }
        },
        llvm_metadata! {
            {
                id: 5,
                distinct: true,
                operands: [node(5), node(4), text("mal buffer object metadata")],
            }
        },
        llvm_metadata!({ id: 6, distinct: false, operands: [node(5)] }),
        llvm_metadata! {
            {
                id: 7,
                distinct: false,
                operands: [text("mal buffer object field"), node(1), integer(int(64), 0)],
            }
        },
        llvm_metadata! {
            {
                id: 8,
                distinct: false,
                operands: [node(7), node(7), integer(int(64), 0)],
            }
        },
    ]
}
