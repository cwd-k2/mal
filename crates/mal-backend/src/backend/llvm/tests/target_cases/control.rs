use super::*;

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
fn uses_narrow_control_storage_offsets() {
    let source = SourceFile::new(
        FileId::new(101),
        "llvm-16-bit-control.mal",
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
    let checked = mal_frontend::analysis::check(&source).expect("check 16-bit control fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    for bits in [8, 16] {
        let data_layout = format!("e-p:{bits}:{bits}-i64:64");
        let artifacts = generate(
            &execution,
            Target {
                triple: "synthetic-unknown-none",
                data_layout: &data_layout,
            },
            OptimizationSet::production(),
        )
        .expect("narrow control fixture is supported");

        assert!(artifacts.module.contains(&format!(
            "declare ptr @mal_control_reserve_frame(ptr, i{bits}, i{bits})"
        )));
        assert!(
            artifacts
                .module
                .contains(&format!("%mal_control_top = alloca i{bits}"))
        );
    }
}

#[test]
fn rejects_control_frames_larger_than_the_target_index_range() {
    let elements = std::iter::repeat_n("Symbol", 86)
        .collect::<Vec<_>>()
        .join(", ");
    let values = std::iter::repeat_n("value", 86)
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!(
        "_Wide :: ({elements});\n\
         consume :: _Wide -> Int32 := (_) -> 0i32;\n\
         walk :: (Int32, Symbol) -> Int32 := (depth, value) -> {{\n\
           wide := ({values});\n\
           if (depth == 0i32) then {{ 0i32 }} else {{\n\
             rest := walk(depth - 1i32, value);\n\
             consume(wide) + rest;\n\
           }};\n\
         }};\n\
         main :: Unit -> Int32 := () -> walk(1i32, \"x\");"
    );
    let source = SourceFile::new(FileId::new(103), "llvm-control-layout.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check control layout fixture");
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
            triple: "synthetic-unknown-none",
            data_layout: "e-p:8:8-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("control frame exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("control frame diagnostic span");
    assert!(
        source.text()[primary.span.start()..primary.span.end()]
            .contains("walk(depth - 1i32, value)"),
        "diagnostic points at the suspended call"
    );
    assert!(primary.message.contains("264"));
    assert!(primary.message.contains("255"));
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
