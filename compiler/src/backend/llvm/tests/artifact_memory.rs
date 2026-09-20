use super::super::*;
use crate::source::{FileId, SourceFile};

#[test]
fn emits_typed_region_access_with_unaligned_operations() {
    let source = SourceFile::new(
        FileId::new(89),
        "llvm-region.mal",
        "extern memory :: Unit -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           value := view<UInt64>(memory(), 0usize, 1usize, (region) -> {\n\
             region.put(0usize, 41u64);\n\
             region.get(0usize);\n\
           });\n\
           value.i32;\n\
         };"
        .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check Region fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("typed Region fixture is supported");

    assert!(artifacts.module.contains("store i64 %mal_value"));
    assert!(artifacts.module.contains("load i64, ptr"));
    assert!(artifacts.module.contains("align 1"));
    assert!(artifacts.header.contains("mal_Address_return"));
}

#[test]
fn emits_packed_views_indexing_and_symbol_conversion() {
    let source = SourceFile::new(
        FileId::new(91),
        "llvm-packed.mal",
        "main :: Unit -> Int32 := () -> {\n\
           packed := *\"abc\";\n\
           prefix := packed / 2usize;\n\
           byte := prefix # 1usize;\n\
           text := *prefix;\n\
           byte.i32 + (#text).i32;\n\
         };"
        .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check Packed fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("Packed fixture is supported");

    assert!(artifacts.module.contains(
        "declare ptr @mal_runtime_bytes_data(ptr) nofree nounwind willreturn memory(argmem: read)"
    ));
    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_bytes_data")
    );
    assert_eq!(
        artifacts
            .module
            .matches("call ptr @mal_runtime_bytes_retain")
            .count(),
        0
    );
    assert!(
        artifacts
            .module
            .contains("call void @mal_runtime_bytes_release")
    );
}

#[test]
fn emits_canonical_alignment_for_packed_storage_access() {
    let source = SourceFile::new(
        FileId::new(92),
        "llvm-packed-alignment.mal",
        "main :: Unit -> Int32 := () -> { values := make<Int64>(0usize, (buffer) -> { index := buffer.new(1i64); buffer.put(index, buffer.get(index) + 1i64); (); }); (values # 0usize).i32 - 2i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check aligned Packed fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64-i64:64",
        },
        OptimizationSet::production(),
    )
    .expect("aligned Packed fixture is supported");

    assert!(artifacts.module.contains("load i64, ptr"));
    assert!(artifacts.module.contains("store i64"));
    assert!(artifacts.module.contains("align 8"));
}

#[test]
fn emits_scoped_buffer_operations_without_closure_environments() {
    let source = SourceFile::new(
        FileId::new(93),
        "llvm-packed-builder.mal",
        "fill :: Buffer<Int64> -> Unit := (buffer) -> { index := buffer.new(1i64); buffer.put(index, buffer.get(index)); (); }; main :: Unit -> Int32 := () -> { first := make<Int64>(0usize, fill); second := make<Int64>(0usize, fill); ((first # 0usize) + (second # 0usize)).i32 - 2i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check scoped Packed fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("scoped Packed fixture is supported");

    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_environment_allocate")
    );
    assert_eq!(
        artifacts
            .module
            .matches("call i64 @mal_runtime_packed_builder_new")
            .count(),
        1
    );
    assert!(
        artifacts
            .module
            .contains("ptr %mal_packed_new_value, i64 8)")
    );
    assert_eq!(
        artifacts
            .module
            .matches("%mal_packed_new_value = alloca [8 x i8], align 8")
            .count(),
        1
    );
    assert!(!artifacts.module.contains("= alloca i8, i64 8"));
    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_packed_builder_get")
    );
    assert!(
        artifacts
            .module
            .contains("call ptr @mal_runtime_packed_builder_data_slot")
    );
    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_bytes_data")
    );
    assert!(artifacts.module.contains(", i64 8)"));
}

#[test]
fn passes_active_data_to_non_growing_buffer_helpers() {
    let source = SourceFile::new(
        FileId::new(97),
        "llvm-direct-buffer-abi.mal",
        "read :: Buffer<Int64> -> Int64 := (buffer) -> { buffer.get(0usize); }; main :: Unit -> Int32 := () -> { values := make<Int64>(0usize, (buffer) -> { _ := buffer.new(1i64); buffer.put(0usize, read(buffer)); }); (values # 0usize).i32 - 1i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check direct Buffer ABI fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let production = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("production direct Buffer ABI fixture is supported");
    let baseline = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("baseline direct Buffer ABI fixture is supported");

    let production_read = production
        .module
        .split("define internal i64")
        .nth(1)
        .and_then(|body| body.split("\ndefine ").next())
        .expect("production read helper");
    let baseline_read = baseline
        .module
        .split("define internal i64")
        .nth(1)
        .and_then(|body| body.split("\ndefine ").next())
        .expect("baseline read helper");
    assert!(!production_read.contains("@mal_runtime_packed_builder_data_slot"));
    assert!(production_read.contains("ptr noalias %mal_buffer_data"));
    assert!(baseline_read.contains("@mal_runtime_packed_builder_data_slot"));
}

#[test]
fn passes_each_active_data_pointer_to_multi_buffer_helpers() {
    let source = SourceFile::new(
        FileId::new(98),
        "llvm-multi-buffer-abi.mal",
        "readPair :: ((Buffer<Int64>, Buffer<Int64>), USize) -> Int64 := ((left, right), index) -> { left.get(index) + right.get(index); }; main :: Unit -> Int32 := () -> { values := make<Int64>(1usize, (buffer) -> { _ := buffer.new(1i64); _ := readPair(((buffer, buffer), 0usize)); (); }); (values # 0usize).i32 - 1i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check multi-Buffer ABI fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let production = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("production multi-Buffer ABI fixture is supported");
    let baseline = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("baseline multi-Buffer ABI fixture is supported");

    fn function_body(module: &str) -> &str {
        module
            .split("define internal i64")
            .nth(1)
            .and_then(|body| body.split("\ndefine ").next())
            .expect("multi-Buffer read helper")
    }
    let production_read = function_body(&production.module);
    let baseline_read = function_body(&baseline.module);
    assert!(!production_read.contains("@mal_runtime_packed_builder_data_slot"));
    assert!(!production_read.contains("%mal_buffer_data"));
    assert_eq!(
        baseline_read
            .matches("@mal_runtime_packed_builder_data_slot")
            .count(),
        2
    );
}
