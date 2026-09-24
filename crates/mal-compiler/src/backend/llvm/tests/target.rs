use super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn reads_supported_pointer_widths_from_target_data_layouts() {
    assert_eq!(target_layout("e-m:e-i64:64"), TargetLayout::natural(8, 8));
    for bits in 0_usize..=256 {
        let bytes = bits / 8;
        let expected = (bits.is_multiple_of(8) && matches!(bytes, 1 | 2 | 4 | 8))
            .then(|| TargetLayout::natural(bytes, bytes))
            .flatten();
        assert_eq!(target_layout(&format!("e-p:{bits}:{bits}")), expected);
        assert_eq!(target_layout(&format!("e-p0:{bits}:{bits}")), expected);
    }
    assert_eq!(
        target_layout("e-p1:32:32-p0:64:64:64:32"),
        TargetLayout::natural(8, 4)
    );
    assert_eq!(target_layout("e-p:invalid:64"), None);
}

#[test]
fn reads_abi_alignments_independently_from_sizes() {
    assert_eq!(
        target_layout("e-p:64:32:64:32-i16:32-i64:32-f32:64-f64:128"),
        Some(TargetLayout {
            pointer_size: 8,
            pointer_alignment: 4,
            index_size: 4,
            integer_alignments: [1, 4, 4, 4],
            float_alignments: [8, 16],
            supports_pointer_alignment: true,
        })
    );
}

#[test]
fn uses_the_target_size_type_for_control_storage_offsets() {
    let source = SourceFile::new(
        FileId::new(79),
        "llvm-32-bit-control.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> { operation(value); };\n\
         sum :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0i32)\n\
           then { 0i32 }\n\
           else {\n\
             rest := apply(sum, value - 1i32);\n\
             value + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { sum(4i32); };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check 32-bit control fixture");
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
    .expect("32-bit control fixture is supported");

    assert!(
        artifacts
            .module
            .contains("declare ptr @mal_control_reserve_frame(ptr, i32, i32)")
    );
    assert!(
        artifacts
            .module
            .contains("%mal_control_top = alloca i32, align 4")
    );
    assert!(
        artifacts
            .module
            .contains("%mal_active_environment = alloca ptr, align 4")
    );
    assert!(
        !artifacts
            .module
            .contains("ptr %mal_active_environment, align 8")
    );
}

#[test]
fn aligns_heterogeneous_frames_and_reserves_when_replacement_is_too_small() {
    let source = SourceFile::new(
        FileId::new(95),
        "llvm-aligned-control.mal",
        "walk :: (Int32, Float64) -> Float64 := (depth, value) -> {\n\
           if (depth == 0i32) then { value } else {\n\
             first := walk(depth - 1i32, value);\n\
             second := walk(depth - 1i32, first);\n\
             first + second;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { walk(2i32, 1.0f64).i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check aligned frame fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    assert!(
        (0..execution.control.states.len())
            .map(crate::control::ast::StateId)
            .any(|site| execution.control_frames.replacement(site).is_some()),
        "the second recursive call has a retired-frame replacement candidate"
    );
    let artifacts = generate(
        &execution,
        Target {
            triple: "synthetic-unknown-none",
            data_layout: "e-p:32:32-i64:64-f64:128",
        },
        OptimizationSet::production(),
    )
    .expect("heterogeneous frame fixture is supported");

    assert_eq!(
        artifacts
            .module
            .matches("call ptr @mal_control_reserve_frame")
            .count(),
        2
    );
    assert!(artifacts.module.contains(", i32 16)"));
    assert!(artifacts.module.contains(", i32 32)"));
}

#[test]
fn separates_pointer_representation_and_index_widths() {
    let source = SourceFile::new(
        FileId::new(90),
        "llvm-index-width.mal",
        "scale :: (USize, ByteSize) -> ByteSize := (count, size) -> count * size;\n\
         main :: Unit -> Int32 := () -> scale(3usize, 8bytes).usize.i32;"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check index-width fixture");
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
            triple: "synthetic-unknown-none",
            data_layout: "e-p:64:64:64:32",
        },
        OptimizationSet::production(),
    )
    .expect("split pointer/index layout is supported");

    assert!(artifacts.module.contains("mul i32"));
    assert!(
        artifacts
            .module
            .contains("declare ptr @llvm.ptrmask.p0.i64(ptr, i64)")
    );
}

#[test]
fn rejects_target_sized_literals_with_a_source_diagnostic() {
    let source = SourceFile::new(
        FileId::new(92),
        "llvm-target-literal.mal",
        "main :: Unit -> Int32 := () -> 4294967296usize.i32;".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check target literal fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());

    let error = match generate(
        &execution,
        Target {
            triple: "i386-unknown-linux-gnu",
            data_layout: "e-p:32:32-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("literal exceeds the 32-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("literal diagnostic span");
    assert_eq!(
        &source.text()[primary.span.start()..primary.span.end()],
        "4294967296usize"
    );
    assert!(primary.message.contains("4294967295"));
}

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
