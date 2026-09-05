use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(107), "format-test.mal", text.into())
}

fn format(text: &str) -> String {
    malc::formatter::format(&source(text)).expect("formatted source")
}

#[test]
fn formats_spacing_and_blocks_canonically() {
    let formatted =
        format("choose::Bool->Int32:=\\(condition::Bool){if(condition)then{1}else{2};};");

    assert_eq!(
        formatted,
        concat!(
            "choose :: Bool -> Int32 := \\(condition :: Bool) {\n",
            "    if (condition)\n",
            "        then { 1 }\n",
            "        else { 2 };\n",
            "};\n",
        )
    );
}

#[test]
fn preserves_comments_and_literal_spelling() {
    let formatted =
        format("// heading\nnumber::UInt32:=0xff_ffu32;// value\ntext::String:=\"a\\x62\";\n");

    assert!(formatted.starts_with("// heading\n"));
    assert!(formatted.contains("0xff_ffu32; // value\n"));
    assert!(formatted.contains("\"a\\x62\""));
}

#[test]
fn groups_declarations_and_separates_top_level_bindings() {
    let formatted = format(
        "Pair::(Int32,Int32);extern Handle;extern use::Handle->Unit;\n\
         // first binding\nfirst:=\\(){1;};// result\n\
         // second binding\nsecond:=\\(){2;};",
    );

    assert_eq!(
        formatted,
        concat!(
            "Pair :: (Int32, Int32);\n",
            "extern Handle;\n",
            "extern use :: Handle -> Unit;\n",
            "\n",
            "// first binding\n",
            "first := \\() { 1 }; // result\n",
            "\n",
            "// second binding\n",
            "second := \\() { 2 };\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn formatting_is_idempotent_and_preserves_checked_behavior() {
    let input = "main::Unit->Int32:=\\(){(40+2);};";
    let first = format(input);
    let second = format(&first);

    assert_eq!(second, first);
    assert!(malc::pipeline::check(&source(input)).is_ok());
    assert!(malc::pipeline::check(&source(&first)).is_ok());
}

#[test]
fn aligns_case_arms_at_the_continuation_indent() {
    let formatted = format(
        "pick::[Int32,UInt8]->Int32:=\\(value::[Int32,UInt8]){case(value)[0](x){x}[1](x){Int32(x)};};",
    );

    assert!(formatted.contains(concat!(
        "    case (value)\n",
        "        [0](x) { x }\n",
        "        [1](x) { Int32(x) };\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn omits_compact_result_semicolons_and_terminates_expanded_results() {
    assert_eq!(
        format("identity:=\\(x::Int32){x;};"),
        "identity := \\(x :: Int32) { x };\n"
    );
    assert_eq!(
        format("run:=\\(){extern first();extern second()};"),
        concat!(
            "run := \\() {\n",
            "    extern first();\n",
            "    extern second();\n",
            "};\n",
        )
    );
    assert_eq!(
        format("run:=\\(){extern first()// result\n};"),
        concat!("run := \\() {\n", "    extern first(); // result\n", "};\n",)
    );
}

#[test]
fn rejects_malformed_source() {
    let diagnostic = malc::formatter::format(&source("value := ;")).expect_err("syntax error");

    assert_eq!(diagnostic.message, "expected an expression");
    assert!(diagnostic.primary.is_some());
}
