use crate::backend::llvm::syntax::{MetadataDefinition, llvm_metadata};

pub(super) fn buffer_alias() -> Vec<MetadataDefinition> {
    // Buffer object fields and element storage are distinct allocations. Their TBAA types preserve
    // that boundary across inlining. Runtime slot writes do not carry this metadata, so growth still
    // invalidates an active data pointer.
    vec![
        llvm_metadata!(0 => [(text "Simple C/C++ TBAA")]),
        llvm_metadata!(1 => [
            (text "omnipotent char"),
            (node 0),
            (integer int(64) => 0),
        ]),
        llvm_metadata!(2 => [
            (text "mal buffer element storage"),
            (node 1),
            (integer int(64) => 0),
        ]),
        llvm_metadata!(3 => [(node 2), (node 2), (integer int(64) => 0)]),
        llvm_metadata!(distinct 4 => [
            (node 4),
            (text "mal buffer object allocation"),
        ]),
        llvm_metadata!(distinct 5 => [
            (node 5),
            (node 4),
            (text "mal buffer object metadata"),
        ]),
        llvm_metadata!(6 => [(node 5)]),
        llvm_metadata!(7 => [
            (text "mal buffer object field"),
            (node 1),
            (integer int(64) => 0),
        ]),
        llvm_metadata!(8 => [(node 7), (node 7), (integer int(64) => 0)]),
    ]
}
