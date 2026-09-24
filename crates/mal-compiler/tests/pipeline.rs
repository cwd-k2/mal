use mal_syntax::source::{FileId, SourceFile};

#[test]
fn checks_in_memory_source_without_an_external_boundary() {
    let source = SourceFile::new(
        FileId::new(101),
        "memory.mal",
        "value :: Int32 := 1;".into(),
    );

    assert!(mal_frontend::analysis::check(&source).is_ok());
}

#[test]
fn preserves_structured_frontend_diagnostics() {
    let source = SourceFile::new(FileId::new(102), "memory.mal", "value :: Unit := 1;".into());

    let diagnostic = mal_frontend::analysis::check(&source).expect_err("type mismatch");
    let primary = diagnostic.primary.expect("primary label");
    assert_eq!(diagnostic.message, "type mismatch");
    assert_eq!(primary.span.file(), source.id());
}

#[test]
fn emits_host_stubs_with_a_validated_header_name() {
    let source = SourceFile::new(
        FileId::new(104),
        "memory.mal",
        "extern print :: (Address, USize) -> Unit;".into(),
    );

    let output = mal_compiler::pipeline::emit_host(&source, "custom.h").expect("host output");
    assert!(output.starts_with("#include \"custom.h\"\n"));

    let diagnostic = mal_compiler::pipeline::emit_host(&source, "invalid\"name.h")
        .expect_err("invalid quoted include");
    assert!(
        diagnostic
            .message
            .contains("not valid in a quoted C include")
    );
}
