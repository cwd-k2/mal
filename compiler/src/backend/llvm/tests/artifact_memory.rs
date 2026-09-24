use super::super::*;
use crate::source::{FileId, SourceFile};

fn generate_module(text: &str) -> String {
    let source = SourceFile::new(FileId::new(91), "llvm-buffer.mal", text.into());
    let checked = crate::pipeline::check(&source).expect("check Buffer fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize Buffer fixture"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("Buffer fixture is supported")
    .module
}

#[test]
fn emits_managed_shared_buffer_operations() {
    let module = generate_module(
        "main :: Unit -> Int32 := () -> {
           values := make<Int64>(1usize);
           alias := values;
           index := values.new(41i64);
           values.fill(1usize, 2usize, 40i64);
           alias.copy(0usize, values, 1usize, 2usize);
           alias.put(index, alias.get(index) + 1i64);
           alias.get(0usize).i32 - 42i32;
         };",
    );

    assert!(module.contains("call ptr @mal_runtime_buffer_make"));
    assert!(module.contains("call i64 @mal_runtime_buffer_new"));
    assert!(module.contains("call void @mal_runtime_buffer_fill"));
    assert!(module.contains("call void @mal_runtime_buffer_copy"));
    assert!(module.contains("call ptr @mal_runtime_buffer_data_slot"));
    assert!(module.contains("mal buffer element storage"));
    assert!(module.contains("mal buffer object allocation"));
    assert!(module.contains(", !alias.scope !6"));
    assert!(module.contains(", !noalias !6"));
}

#[test]
fn borrows_buffer_operands_without_temporary_owner_traffic() {
    let module = generate_module(
        "main :: Unit -> Int32 := () -> {
           values := make<Int64>(1usize);
           index := values.new(41i64);
           values.fill(1usize, 2usize, 41i64);
           values.copy(0usize, values, 1usize, 2usize);
           values.put(index, values.get(index) + 1i64);
           values.get(index).i32 - 42i32;
         };",
    );

    assert_eq!(
        module
            .matches("call ptr @mal_runtime_environment_retain")
            .count(),
        0
    );
    assert_eq!(
        module
            .matches("call void @mal_runtime_environment_release")
            .count(),
        1
    );
}

#[test]
fn emits_snapshot_symbol_conversions() {
    let module = generate_module(
        "main :: Unit -> Int32 := () -> {
           bytes := *\"abc\";
           bytes.put(0usize, 100u8);
           text := *bytes;
           ((#text).i32 + bytes.get(0usize).i32) - 103i32;
         };",
    );

    assert!(module.contains("call ptr @mal_runtime_bytes_read"));
    assert!(module.contains("call ptr @mal_runtime_buffer_from"));
}

#[test]
fn emits_c_host_buffer_copy_primitives() {
    let module = generate_module(
        "extern memory :: Unit -> Address;
         main :: Unit -> Int32 := () -> {
           address := memory();
           values := from<UInt64>(address, 2usize, 3usize);
           values.into(address, 1usize, 2usize);
           (#values).i32 - 3i32;
         };",
    );

    assert!(module.contains("call ptr @mal_runtime_buffer_from"));
    assert!(module.contains("call void @mal_runtime_buffer_into"));
}
