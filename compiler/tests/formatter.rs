use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(107), "format-test.mal", text.into())
}

fn format(text: &str) -> String {
    malc::formatter::format(&source(text)).expect("formatted source")
}

#[test]
fn formats_spacing_and_blocks_canonically() {
    let formatted = format("choose::Bool->Int32:=(condition){if(condition)then{1}else{2};};");

    assert_eq!(
        formatted,
        concat!(
            "choose :: Bool -> Int32 := (condition) {\n",
            "    if (condition)\n",
            "    then { 1 }\n",
            "    else { 2 };\n",
            "};\n",
        )
    );
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
fn formats_requirements_as_a_leading_group() {
    assert_eq!(
        format("require\"./support.mal\";require \"./host.c\";main:=(){0;};"),
        concat!(
            "require \"./support.mal\";\n",
            "require \"./host.c\";\n",
            "\n",
            "main := () { 0 };\n",
        )
    );
}

#[test]
fn keeps_type_qualified_primitives_attached() {
    assert_eq!(
        format("size::UInt64:=Ptr . size+UInt64. load;"),
        "size :: UInt64 := Ptr.size + UInt64.load;\n"
    );
}

#[test]
fn keeps_receiver_first_calls_attached() {
    assert_eq!(
        format("result:=source . transform ( 1,2 ) . finish ( );"),
        "result := source.transform(1, 2).finish();\n"
    );
}

#[test]
fn preserves_receiver_first_call_chain_breaks() {
    assert_eq!(
        format(
            "result := source\n\
             .transform(option)\n\
             .finish();\n\
             size := UInt8\n\
             .size;"
        ),
        "result := source\n\
         \x20   .transform(option)\n\
         \x20   .finish();\n\
         size := UInt8.size;\n"
    );
}

#[test]
fn keeps_decimal_points_inside_float_tokens() {
    assert_eq!(format("value:=1.25f32;"), "value := 1.25f32;\n");
}

#[test]
fn formats_unary_and_binary_symbol_operators() {
    assert_eq!(
        format("inspect:=(value){# value+value#1u64;};"),
        "inspect := (value) { #value + value # 1u64 };\n"
    );
}

#[test]
fn groups_declarations_and_separates_top_level_bindings() {
    let formatted = format(
        "Pair::(Int32,Int32);extern Handle;extern use::Handle->Unit;\n\
         // first binding\nfirst:=(){1;};// result\n\
         // second binding\nsecond:=(){2;};",
    );

    assert_eq!(
        formatted,
        concat!(
            "Pair :: (Int32, Int32);\n",
            "extern Handle;\n",
            "extern use :: Handle -> Unit;\n",
            "\n",
            "// first binding\n",
            "first := () { 1 }; // result\n",
            "\n",
            "// second binding\n",
            "second := () { 2 };\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn formatting_is_idempotent_and_preserves_checked_behavior() {
    let input = "main::Unit->Int32:=(){(40+2);};";
    let first = format(input);
    let second = format(&first);

    assert_eq!(second, first);
    assert!(malc::pipeline::check(&source(input)).is_ok());
    assert!(malc::pipeline::check(&source(&first)).is_ok());
}

#[test]
fn aligns_multiline_sum_continuations_with_the_value() {
    let formatted = format("pick::[Int32,UInt8]->Int32:=(value){value[(x){x},(x){Int32(x)}];};");

    assert!(formatted.contains(concat!(
        "    value[\n",
        "    (x) { x },\n",
        "    (x) { Int32(x) }\n",
        "    ];\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn indents_control_branches_used_as_binding_rhs() {
    let formatted = format(
        "choose:=(condition, value){selected:=if(condition)then{1}else{2};result:=value[(x){x},(x){x}];selected+result;};",
    );

    assert!(formatted.contains(concat!(
        "    selected := if (condition)\n",
        "        then { 1 }\n",
        "        else { 2 };\n",
        "    result := value[\n",
        "        (x) { x },\n",
        "        (x) { x }\n",
        "    ];\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn keeps_single_continuation_chains_inline() {
    assert_eq!(format("result:=value[f][g];"), "result := value[f][g];\n");
}

#[test]
fn formats_explicit_return_surface_forms() {
    let formatted =
        format("absolute::Int32->Int32:=(x)[return]{when(x>=0){return(x)};return(-x)};");
    assert_eq!(
        formatted,
        concat!(
            "absolute :: Int32 -> Int32 := (x)[return] {\n",
            "    when (x >= 0) {\n",
            "        return(x);\n",
            "    };\n",
            "    return(-x);\n",
            "};\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn formats_sum_and_empty_return_binder_groups() {
    let formatted = format(
        "Result::[Int32,Symbol];compute::Bool->Result:=(enabled)[ok,err]{when(enabled){ok(42)};err(\"disabled\")};never::Unit->[]:=()[]{never()[]};",
    );
    assert_eq!(
        formatted,
        concat!(
            "Result :: [Int32, Symbol];\n",
            "\n",
            "compute :: Bool -> Result := (enabled)[ok, err] {\n",
            "    when (enabled) {\n",
            "        ok(42);\n",
            "    };\n",
            "    err(\"disabled\");\n",
            "};\n",
            "\n",
            "never :: Unit -> [] := ()[] { never()[] };\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_one_intentional_blank_line_between_block_steps() {
    let formatted = format(
        "work::Int32->Int32:=(value)[return]{first:=value+1;\n\n\n// second stage\nsecond:=first*2;\n\nreturn(second)};",
    );
    assert_eq!(
        formatted,
        concat!(
            "work :: Int32 -> Int32 := (value)[return] {\n",
            "    first := value + 1;\n",
            "\n",
            "    // second stage\n",
            "    second := first * 2;\n",
            "\n",
            "    return(second);\n",
            "};\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn keeps_nested_compact_control_inside_its_enclosing_branch() {
    let formatted = format(
        "select::Bool->Int32:=(condition){result:=if(condition)then{1}else{when(condition){noop()};2};result};",
    );
    assert!(formatted.contains(concat!(
        "        else {\n",
        "            when (condition) {\n",
        "                noop();\n",
        "            };\n",
        "            2;\n",
        "        };\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_explicit_binding_and_expression_breaks() {
    let formatted = format(
        "first::(Int32,Int32)->Int32\n:=(left, right){\nresult:=left\n+right;\nemit(\nleft,\nright\n);\nresult\n};\nsecond::Unit->Int32:=\n(){1};",
    );

    assert!(formatted.contains(concat!(
        "first :: (Int32, Int32) -> Int32\n",
        "    := (left, right) {\n",
        "        result := left\n",
        "            + right;\n",
        "        emit(\n",
        "            left,\n",
        "            right\n",
        "        );\n",
        "        result;\n",
        "    };\n",
        "\n",
        "second :: Unit -> Int32 :=\n",
        "    () { 1 };\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_explicit_top_level_groups_without_splitting_data_bindings() {
    let formatted = format("extern A;\n\nextern B;\nfirst:=1;\nsecond:=2;");

    assert_eq!(
        formatted,
        concat!(
            "extern A;\n",
            "\n",
            "extern B;\n",
            "first := 1;\n",
            "second := 2;\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_explicit_type_and_extern_declaration_breaks() {
    let formatted = format("Pair\n::(Int32,Int32);\nextern combine\n::(Int32,Int32)\n->Int32;");

    assert_eq!(
        formatted,
        concat!(
            "Pair\n",
            "    :: (Int32, Int32);\n",
            "extern combine\n",
            "    :: (Int32, Int32)\n",
            "    -> Int32;\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn omits_compact_result_semicolons_and_terminates_expanded_results() {
    assert_eq!(format("identity:=(x){x;};"), "identity := (x) { x };\n");
    assert_eq!(
        format("run:=(){first();second()};"),
        concat!("run := () {\n", "    first();\n", "    second();\n", "};\n",)
    );
    assert_eq!(
        format("run:=(){first()// result\n};"),
        concat!("run := () {\n", "    first(); // result\n", "};\n",)
    );
}

#[test]
fn rejects_malformed_source() {
    let diagnostic = malc::formatter::format(&source("value := ;")).expect_err("syntax error");

    assert_eq!(diagnostic.message, "expected an expression");
    assert!(diagnostic.primary.is_some());
}
