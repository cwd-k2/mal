use super::*;

#[test]
fn preserves_type_alias_names_as_backend_metadata() {
    let program = lower_ok(
        "Flag :: [Unit, Unit];\n\
         extern choose :: Flag -> Int32;\n\
         main :: Unit -> Int32 := \\() { 0; };",
    );

    assert_eq!(program.interface.type_aliases.len(), 1);
    assert_eq!(program.interface.type_aliases[0].name, "Flag");
    assert_eq!(program.interface.externals.len(), 1);
    assert_eq!(program.interface.externals[0].name, "choose");
    assert_eq!(program.bindings.len(), 1);
    let TopLevelPattern::Binding { name, .. } = &program.bindings[0].pattern else {
        panic!("expected named top-level binding");
    };
    assert_eq!(name, "main");
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
