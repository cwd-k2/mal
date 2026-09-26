use super::*;

#[test]
fn lowers_symbol_length_to_usize_on_a_32_bit_target() {
    let source = SourceFile::new(
        FileId::new(93),
        "llvm-symbol-length-32.mal",
        "main :: Unit -> Int32 := () -> (#\"abc\").i32;".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check Symbol length fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "i386-unknown-linux-gnu",
            data_layout: "e-p:32:32-i64:64",
        },
        OptimizationSet::production(),
    )
    .expect("32-bit Symbol length fixture is supported");

    assert!(artifacts.module.contains("extractvalue { ptr, ptr, i32 }"));
    assert!(!artifacts.module.contains("mal_runtime_symbol_length"));
}

#[test]
fn stores_symbol_elements_at_the_target_value_layout() {
    let source = SourceFile::new(
        FileId::new(96),
        "llvm-32-bit-symbol-buffer.mal",
        "main :: Unit -> Int32 := () -> {\n\
           names := make<Symbol>(1usize);\n\
           names.new(\"ab\");\n\
           (#(names.get(0usize))).i32;\n\
         };"
        .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check 32-bit Symbol buffer");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "i386-unknown-linux-gnu",
            data_layout: "e-p:32:32-i64:32",
        },
        OptimizationSet::production(),
    )
    .expect("32-bit Symbol buffer is supported");

    assert!(artifacts.module.contains(
        "call ptr @mal_runtime_buffer_make_managed(ptr %mal_context, i32 12, i32 1, ptr @mal_buffer_retain_0, ptr @mal_buffer_release_0)"
    ));
}
