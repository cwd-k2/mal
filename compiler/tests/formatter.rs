use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(107), "format-test.mal", text.into())
}

fn format(text: &str) -> String {
    malc::formatter::format(&source(text)).expect("formatted source")
}

#[test]
fn formats_spacing_and_blocks_canonically() {
    let formatted = format("choose::Bool->Int32:=(condition) -> {if(condition)then{1}else{2};};");

    assert_eq!(
        formatted,
        concat!(
            "choose :: Bool -> Int32 := (condition) -> {\n",
            "    if (condition)\n",
            "    then { 1 }\n",
            "    else { 2 };\n",
            "};\n",
        )
    );
}

#[test]
fn formats_expression_bodies_for_binder_and_control_forms() {
    let formatted = format(
        "identity:=(value)->value;choose:=(condition)->if(condition)then 1 else 2;finish:=(condition) -> [return] =>{when(condition)return(1);return(0)};",
    );

    assert_eq!(
        formatted,
        concat!(
            "identity := (value) -> value;\n",
            "\n",
            "choose := (condition) -> if (condition)\n",
            "    then 1\n",
            "    else 2;\n",
            "\n",
            "finish := (condition) -> [return] => {\n",
            "    when (condition) return(1);\n",
            "    return(0);\n",
            "};\n",
        )
    );
    assert_eq!(format(&formatted), formatted);

    let nested =
        format("choose := (first, second) -> if (first) then if (second) then 1 else 2 else 3;");
    assert_eq!(
        nested,
        concat!(
            "choose := (first, second) -> if (first)\n",
            "    then if (second)\n",
            "        then 1\n",
            "        else 2\n",
            "    else 3;\n",
        )
    );
    assert_eq!(format(&nested), nested);
}

#[test]
fn separates_control_keywords_from_every_prefix_expression() {
    for expression in ["-value", "!value", "~value", "#value", "*value"] {
        let formatted = format(&format!(
            "choose := (condition, value) -> if(condition)then{expression} else{expression};"
        ));
        assert!(
            formatted.contains(&format!("then {expression}\n")),
            "missing keyword boundary in:\n{formatted}"
        );
        assert!(
            formatted.contains(&format!("else {expression};\n")),
            "missing keyword boundary in:\n{formatted}"
        );
        assert_eq!(format(&formatted), formatted);
    }
}

#[test]
fn preserves_comments_and_literal_spelling() {
    let formatted =
        format("// heading\nnumber::UInt32:=0xff_ffu32;// value\ntext::Symbol:=\"a\\x62\";\n");

    assert!(formatted.starts_with("// heading\n"));
    assert!(formatted.contains("0xff_ffu32; // value\n"));
    assert!(formatted.contains("\"a\\x62\""));
}

#[test]
fn indents_comments_and_results_after_expression_body_arrows() {
    let formatted = format(
        "identity := (value) ->\n// result\nvalue; direct := () -> [done] => // direct result\ndone(1);",
    );

    assert_eq!(
        formatted,
        concat!(
            "identity := (value) ->\n",
            "    // result\n",
            "    value;\n",
            "\n",
            "direct := () -> [done] => // direct result\n",
            "    done(1);\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn formats_requirements_as_a_leading_group() {
    assert_eq!(
        format("require\"./support.mal\";require \"./host.c\";main:=() -> {0;};"),
        concat!(
            "require \"./support.mal\";\n",
            "require \"./host.c\";\n",
            "\n",
            "main := () -> { 0 };\n",
        )
    );
}

#[test]
fn keeps_generic_delimiters_attached_without_changing_comparisons_or_shifts() {
    let formatted = format(
        "Pair < A,B >::(A,B); identity < A >::A->A:=(value)->value; value:=identity < Pair < Int32 >> (input); compare:=left<right; shift:=left>>right;",
    );
    assert_eq!(
        formatted,
        concat!(
            "Pair<A, B> :: (A, B);\n",
            "\n",
            "identity<A> :: A -> A := (value) -> value;\n",
            "\n",
            "value := identity<Pair<Int32>>(input);\n",
            "compare := left < right;\n",
            "shift := left >> right;\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn keeps_numeric_conversion_suffixes_attached() {
    assert_eq!(
        format("value::UInt64:=1i8 . u8 . u64;"),
        "value :: UInt64 := 1i8.u8.u64;\n"
    );
}

#[test]
fn keeps_receiver_first_calls_attached() {
    assert_eq!(
        format("result:=source . transform ( 1,2 ) . finish ( );"),
        "result := source.transform(1, 2).finish();\n"
    );
    assert_eq!(
        format("result := 1 . transform();"),
        "result := 1.transform();\n"
    );
}

#[test]
fn preserves_receiver_first_call_chain_breaks() {
    assert_eq!(
        format(
            "result := source\n\
             .transform(option)\n\
             .finish();\n\
             byte := value\n\
             .u8;"
        ),
        "result := source\n\
         \x20   .transform(option)\n\
         \x20   .finish();\n\
         byte := value\n\
         \x20   .u8;\n"
    );
}

#[test]
fn keeps_decimal_points_inside_float_tokens() {
    assert_eq!(format("value:=1.25f32;"), "value := 1.25f32;\n");
}

#[test]
fn formats_unary_and_binary_symbol_operators() {
    assert_eq!(
        format("inspect:=(value) -> {# value+value#1usize;};"),
        "inspect := (value) -> { #value + value # 1usize };\n"
    );
}

#[test]
fn groups_declarations_and_separates_top_level_bindings() {
    let formatted = format(
        "Pair::(Int32,Int32);extern Handle;extern use::Handle->Unit;\n\
         // first binding\nfirst:=() -> {1;};// result\n\
         // second binding\nsecond:=() -> {2;};",
    );

    assert_eq!(
        formatted,
        concat!(
            "Pair :: (Int32, Int32);\n",
            "extern Handle;\n",
            "extern use :: Handle -> Unit;\n",
            "\n",
            "// first binding\n",
            "first := () -> { 1 }; // result\n",
            "\n",
            "// second binding\n",
            "second := () -> { 2 };\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[path = "formatter/layout.rs"]
mod layout;
