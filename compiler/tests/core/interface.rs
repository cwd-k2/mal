use super::*;

#[test]
fn preserves_type_alias_names_as_backend_metadata() {
    let program = lower_ok(
        "Flag :: [Unit, Unit];\n\
         extern choose :: Flag -> Int32;\n\
         main :: Unit -> Int32 := () { 0; };",
    );

    assert_eq!(program.interface.type_aliases.len(), 1);
    assert_eq!(program.interface.type_aliases[0].name, "Flag");
    assert_eq!(
        program.interface.type_aliases[0].element_aliases,
        [None, None]
    );
    assert_eq!(program.interface.externals.len(), 1);
    assert_eq!(program.interface.externals[0].name, "choose");
    assert_eq!(
        program.interface.externals[0].parameter_alias.as_deref(),
        Some("Flag")
    );
    assert_eq!(program.bindings.len(), 2);
    let TopLevelPattern::Binding { name, .. } = &program.bindings[0].pattern else {
        panic!("expected named top-level binding");
    };
    assert_eq!(name, "choose");
}

#[test]
fn preserves_aliases_immediately_named_by_a_type_definition() {
    let program = lower_ok(
        "Count :: UInt64;\n\
         Payload :: (UInt8, Int32);\n\
         Request :: (Count, Payload);",
    );

    let request = &program.interface.type_aliases[2];
    assert_eq!(request.name, "Request");
    assert_eq!(request.target_alias, None);
    assert_eq!(
        request.element_aliases,
        [Some("Count".into()), Some("Payload".into())]
    );
}

#[test]
fn preserves_a_product_parameter_alias_before_boundary_flattening() {
    let program = lower_ok(
        "Count :: UInt64;\n\
         Payload :: (UInt8, Int32);\n\
         Request :: (Count, Payload);\n\
         extern exchange :: Request -> Count;\n\
         main :: Unit -> Int32 := () { 0; };",
    );

    let external = &program.interface.externals[0];
    assert_eq!(external.parameter_alias.as_deref(), Some("Request"));
    assert_eq!(
        external.parameter_aliases,
        [Some("Count".into()), Some("Payload".into())]
    );
    assert_eq!(external.result_alias.as_deref(), Some("Count"));
}

#[test]
fn extracts_the_host_interface_without_lowering_value_bindings() {
    let source = SourceFile::new(
        FileId::new(42),
        "core-test.mal",
        "Pair :: (Int32, Symbol);\n\
         extern send :: Pair -> Unit;\n\
         value :: Int32 := 1;"
            .into(),
    );
    let parsed = parser::parse(&source).expect("parsed program");
    let resolved = resolve::resolve(&parsed).expect("resolved program");
    let checked = check::check(&resolved).expect("checked program");

    let interface = core::lower_interface(&checked);

    assert_eq!(interface.type_aliases.len(), 1);
    assert_eq!(interface.external_types.len(), 0);
    assert_eq!(interface.externals.len(), 1);
    assert_eq!(interface.externals[0].name, "send");
}
