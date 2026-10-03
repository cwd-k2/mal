use super::*;

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
fn rejects_symbol_literal_storage_larger_than_the_target_index_range() {
    let literal = "a".repeat(232);
    let source = SourceFile::new(
        FileId::new(107),
        "llvm-target-symbol-literal.mal",
        format!("main :: Unit -> Int32 := () -> (#\"{literal}\").i32;"),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check Symbol literal fixture");
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
        Ok(_) => panic!("Symbol literal owner exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("Symbol literal diagnostic span");
    assert_eq!(
        &source.text()[primary.span.start()..primary.span.end()],
        format!("\"{literal}\"")
    );
    assert!(primary.message.contains("256"));
    assert!(primary.message.contains("255"));
}

#[test]
fn rejects_canonical_layouts_larger_than_the_target_index_range() {
    let mut text = String::from("T0 :: UInt64;\n");
    for index in 1..=13 {
        text.push_str(&format!("T{index} :: (T{}, T{});\n", index - 1, index - 1));
    }
    text.push_str("main :: Unit -> Int32 := () -> 0i32;");
    let source = SourceFile::new(FileId::new(97), "llvm-target-layout.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check target layout fixture");
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
            data_layout: "e-p:16:16-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("canonical layout exceeds the 16-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("layout diagnostic span");
    assert!(
        source.text()[primary.span.start()..primary.span.end()].starts_with("T13 ::"),
        "diagnostic points at the oversized alias"
    );
    assert!(primary.message.contains("65536"));
    assert!(primary.message.contains("65535"));
}

#[test]
fn rejects_external_layouts_larger_than_the_target_index_range() {
    let mut text = String::from("_T0 :: UInt64;\n");
    for index in 1..=5 {
        text.push_str(&format!(
            "_T{index} :: (_T{}, _T{});\n",
            index - 1,
            index - 1
        ));
    }
    text.push_str("extern inspect :: _T5 -> Unit;\nmain :: Unit -> Int32 := () -> 0i32;");
    let source = SourceFile::new(FileId::new(102), "llvm-external-layout.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check external layout fixture");
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
        Ok(_) => panic!("external layout exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("external layout diagnostic span");
    assert_eq!(
        &source.text()[primary.span.start()..primary.span.end()],
        "extern inspect :: _T5 -> Unit;"
    );
    assert!(primary.message.contains("256"));
    assert!(primary.message.contains("255"));
}

#[test]
fn rejects_external_runtime_storage_larger_than_the_target_index_range() {
    let payload = std::iter::repeat_n("UInt8", 252)
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!(
        "_Payload :: ({payload});\n\
         _Result :: [_Payload, _Payload];\n\
         extern inspect :: _Result -> Unit;\n\
         main :: Unit -> Int32 := () -> 0i32;"
    );
    let source = SourceFile::new(FileId::new(104), "llvm-external-storage.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check external storage fixture");
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
        Ok(_) => panic!("external runtime storage exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic
        .primary
        .expect("external runtime storage diagnostic span");
    assert_eq!(
        &source.text()[primary.span.start()..primary.span.end()],
        "extern inspect :: _Result -> Unit;"
    );
    assert!(primary.message.contains("256"));
    assert!(primary.message.contains("255"));
}

#[test]
fn rejects_binding_storage_larger_than_the_target_index_range() {
    let elements = std::iter::repeat_n("UInt64", 32)
        .collect::<Vec<_>>()
        .join(", ");
    let values = std::iter::repeat_n("0u64", 32)
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!(
        "_Wide :: ({elements});\n\
         consume :: _Wide -> Int32 := (_) -> 0i32;\n\
         main :: Unit -> Int32 := () -> {{\n\
           wide := ({values});\n\
           consume(wide);\n\
         }};"
    );
    let source = SourceFile::new(FileId::new(105), "llvm-binding-storage.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check binding storage fixture");
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
        Ok(_) => panic!("binding storage exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("binding storage diagnostic span");
    let highlighted = &source.text()[primary.span.start()..primary.span.end()];
    assert!(
        highlighted.contains("0u64"),
        "diagnostic points at the oversized binding value: {highlighted:?}"
    );
    assert!(primary.message.contains("256"));
    assert!(primary.message.contains("255"));
}

#[test]
fn rejects_closure_environments_larger_than_the_target_index_range() {
    let bindings = (0..32)
        .map(|index| format!("value{index} := {index}u64;"))
        .collect::<Vec<_>>()
        .join("\n");
    let values = (0..32)
        .map(|index| format!("value{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!(
        "keep :: (Unit -> Int32) -> (Unit -> Int32) := (function) -> function;\n\
         main :: Unit -> Int32 := () -> {{\n\
           {bindings}\n\
           operation :: Unit -> Int32 := () -> {{ _ := ({values}); 0i32; }};\n\
           kept := keep(operation);\n\
           kept();\n\
         }};"
    );
    let source = SourceFile::new(FileId::new(106), "llvm-closure-storage.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check closure storage fixture");
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
        Ok(_) => panic!("closure environment exceeds the 8-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic
        .primary
        .expect("closure environment diagnostic span");
    let highlighted = &source.text()[primary.span.start()..primary.span.end()];
    assert!(
        highlighted.contains("value0") && highlighted.contains("value31"),
        "diagnostic points at the oversized closure: {highlighted:?}"
    );
    assert!(primary.message.contains("256"));
    assert!(primary.message.contains("255"));
}

#[test]
fn rejects_oversized_canonical_buffer_elements() {
    let mut text = String::from("_T0 :: UInt64;\n");
    for index in 1..=13 {
        text.push_str(&format!(
            "_T{index} :: (_T{}, _T{});\n",
            index - 1,
            index - 1
        ));
    }
    text.push_str("main :: Unit -> Int32 := () -> { make<_T13>(0usize); 0i32; };");
    let source = SourceFile::new(FileId::new(98), "llvm-buffer-layout.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check Buffer layout fixture");
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
            data_layout: "e-p:16:16-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("Buffer element stride exceeds the 16-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic.primary.expect("Buffer layout diagnostic span");
    assert!(
        source.text()[primary.span.start()..primary.span.end()].contains("make<_T13>"),
        "diagnostic points at the Buffer operation"
    );
    assert!(primary.message.contains("65536"));
    assert!(primary.message.contains("65535"));
}

#[test]
fn rejects_oversized_runtime_owned_buffer_elements() {
    let mut text = String::from("_T0 :: Symbol;\n");
    for index in 1..=14 {
        text.push_str(&format!(
            "_T{index} :: (_T{}, _T{});\n",
            index - 1,
            index - 1
        ));
    }
    text.push_str("main :: Unit -> Int32 := () -> { make<_T14>(0usize); 0i32; };");
    let source = SourceFile::new(
        FileId::new(100),
        "llvm-runtime-owned-buffer-layout.mal",
        text,
    );
    let checked =
        mal_frontend::analysis::check(&source).expect("check runtime-owned Buffer layout fixture");
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
            data_layout: "e-p:16:16-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("runtime-owned Buffer element stride exceeds the 16-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic
        .primary
        .expect("runtime-owned Buffer layout diagnostic span");
    assert!(
        source.text()[primary.span.start()..primary.span.end()].contains("make<_T14>"),
        "diagnostic points at the runtime-owned Buffer operation"
    );
    assert!(primary.message.contains("98304"));
    assert!(primary.message.contains("65535"));
}

#[test]
fn rejects_oversized_canonical_host_copy_elements() {
    let mut text = String::from("extern memory :: Unit -> Address;\n_T0 :: UInt64;\n");
    for index in 1..=13 {
        text.push_str(&format!(
            "_T{index} :: (_T{}, _T{});\n",
            index - 1,
            index - 1
        ));
    }
    text.push_str(
        "main :: Unit -> Int32 := () -> { address := memory(); from<_T13>(address, 0usize, 0usize); 0i32; };",
    );
    let source = SourceFile::new(FileId::new(99), "llvm-host-copy-layout.mal", text);
    let checked = mal_frontend::analysis::check(&source).expect("check host copy layout fixture");
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
            data_layout: "e-p:16:16-i64:64",
        },
        OptimizationSet::production(),
    ) {
        Ok(_) => panic!("host copy element stride exceeds the 16-bit target range"),
        Err(error) => error,
    };
    let Error::Diagnostic(diagnostic) = error else {
        panic!("target admission must return a diagnostic")
    };
    let primary = diagnostic
        .primary
        .expect("host copy layout diagnostic span");
    assert!(
        source.text()[primary.span.start()..primary.span.end()].contains("from<_T13>"),
        "diagnostic points at the host copy operation"
    );
    assert!(primary.message.contains("65536"));
    assert!(primary.message.contains("65535"));
}
