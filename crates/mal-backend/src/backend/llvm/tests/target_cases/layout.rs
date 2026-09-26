use super::*;

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
