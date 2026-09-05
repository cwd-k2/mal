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
        format("choose::Bool->Int32:=\\(condition::Bool){return if(condition)then{1}else{2};};");

    assert_eq!(
        formatted,
        concat!(
            "choose :: Bool -> Int32 := \\(condition :: Bool) {\n",
            "    return if (condition)\n",
            "        then {\n",
            "            1\n",
            "        }\n",
            "        else {\n",
            "            2\n",
            "        };\n",
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
fn formatting_is_idempotent_and_preserves_checked_behavior() {
    let input = "main::Unit->Int32:=\\(){return(40+2);};";
    let first = format(input);
    let second = format(&first);

    assert_eq!(second, first);
    assert!(malc::pipeline::check(&source(input)).is_ok());
    assert!(malc::pipeline::check(&source(&first)).is_ok());
}

#[test]
fn aligns_case_arms_at_the_continuation_indent() {
    let formatted = format(
        "pick::[Int32,UInt8]->Int32:=\\(value::[Int32,UInt8]){return case(value)[0](x){x}[1](x){Int32(x)};};",
    );

    assert!(formatted.contains(concat!(
        "    return case (value)\n",
        "        [0](x) {\n",
        "            x\n",
        "        }\n",
        "        [1](x) {\n",
        "            Int32(x)\n",
        "        };\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn rejects_malformed_source() {
    let diagnostic = malc::formatter::format(&source("value := ;")).expect_err("syntax error");

    assert_eq!(diagnostic.message, "expected an expression");
    assert!(diagnostic.primary.is_some());
}
