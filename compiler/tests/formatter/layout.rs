use super::*;

#[test]
fn formatting_is_idempotent_and_preserves_checked_behavior() {
    let input = "main::Unit->Int32:=() -> {(40+2);};";
    let first = format(input);
    let second = format(&first);

    assert_eq!(second, first);
    assert!(malc::pipeline::check(&source(input)).is_ok());
    assert!(malc::pipeline::check(&source(&first)).is_ok());
}

#[test]
fn aligns_multiline_sum_continuations_with_the_value() {
    let formatted =
        format("pick::[Int32,UInt8]->Int32:=(value) -> {value[(x) -> {x},(x) -> {x.i32}];};");

    assert!(formatted.contains(concat!(
        "    value[\n",
        "    (x) -> { x },\n",
        "    (x) -> { x.i32 }\n",
        "    ];\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn indents_control_branches_used_as_binding_rhs() {
    let formatted = format(
        "choose:=(condition, value) -> {selected:=if(condition)then{1}else{2};result:=value[(x) -> {x},(x) -> {x}];selected+result;};",
    );

    assert!(formatted.contains(concat!(
        "    selected := if (condition)\n",
        "        then { 1 }\n",
        "        else { 2 };\n",
        "    result := value[\n",
        "        (x) -> { x },\n",
        "        (x) -> { x }\n",
        "    ];\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn keeps_single_continuation_chains_inline() {
    assert_eq!(format("result:=value[f][g];"), "result := value[f][g];\n");
}

#[test]
fn formats_result_block_surface_forms() {
    let formatted =
        format("absolute::Int32->Int32:=(x) -> [return] => {when(x>=0){return(x)};return(-x)};");
    assert_eq!(
        formatted,
        concat!(
            "absolute :: Int32 -> Int32 := (x) -> [return] => {\n",
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
fn formats_sum_result_binders_and_abrupt_lambdas() {
    let formatted = format(
        "Result::[Int32,Symbol];compute::Bool->Result:=(enabled) -> [ok,err] => {when(enabled){ok(42)};err(\"disabled\")};never::Unit->[]:=() -> {never()[]};",
    );
    assert_eq!(
        formatted,
        concat!(
            "Result :: [Int32, Symbol];\n",
            "\n",
            "compute :: Bool -> Result := (enabled) -> [ok, err] => {\n",
            "    when (enabled) {\n",
            "        ok(42);\n",
            "    };\n",
            "    err(\"disabled\");\n",
            "};\n",
            "\n",
            "never :: Unit -> [] := () -> { never()[] };\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn formats_direct_blocks_and_result_blocks() {
    let formatted = format(
        "main::Unit->Int32:=() -> {value::Int32:={local:=40;local+1};[done] => {done(value+1)}};",
    );
    assert_eq!(
        formatted,
        concat!(
            "main :: Unit -> Int32 := () -> {\n",
            "    value :: Int32 := {\n",
            "        local := 40;\n",
            "        local + 1;\n",
            "    };\n",
            "    [done] => { done(value + 1) };\n",
            "};\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_one_intentional_blank_line_between_block_steps() {
    let formatted = format(
        "work::Int32->Int32:=(value) -> [return] => {first:=value+1;\n\n\n// second stage\nsecond:=first*2;\n\nreturn(second)};",
    );
    assert_eq!(
        formatted,
        concat!(
            "work :: Int32 -> Int32 := (value) -> [return] => {\n",
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
        "select::Bool->Int32:=(condition) -> {result:=if(condition)then{1}else{when(condition){noop()};2};result};",
    );
    assert!(
        formatted.contains(concat!(
            "        else {\n",
            "            when (condition) {\n",
            "                noop();\n",
            "            };\n",
            "            2;\n",
            "        };\n",
        )),
        "unexpected formatting:\n{formatted}"
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn preserves_explicit_binding_and_expression_breaks() {
    let formatted = format(
        "first::(Int32,Int32)->Int32\n:=(left, right) -> {\nresult:=left\n+right;\nemit(\nleft,\nright\n);\nresult\n};\nsecond::Unit->Int32:=\n() -> {1};",
    );

    assert!(formatted.contains(concat!(
        "first :: (Int32, Int32) -> Int32\n",
        "    := (left, right) -> {\n",
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
        "    () -> { 1 };\n",
    )));
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn indents_multiline_arguments_from_the_call_line() {
    let formatted = format(
        "map := (first, second) ->\n    call(\n        first,\n        second\n    );\nrun := (value) -> {result[(item) -> {nested(\nitem,\nvalue\n)}]};",
    );

    assert!(formatted.contains(concat!(
        "map := (first, second) ->\n",
        "    call(\n",
        "        first,\n",
        "        second\n",
        "    );\n",
    )));
    assert!(
        formatted.contains(concat!(
            "    result[(item) -> {\n",
            "        nested(\n",
            "            item,\n",
            "            value\n",
            "        );\n",
            "    }];\n",
        )),
        "unexpected formatting:\n{formatted}"
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn indents_nested_blocks_inside_expression_body_continuations() {
    let formatted = format(
        "revise :: Packed<Int32> -> Packed<Int32> := (source) ->\nsource.edit<Int32>((buffer) -> {\nbuffer.put(0usize, buffer.get(0usize));\n()\n});",
    );

    assert_eq!(
        formatted,
        concat!(
            "revise :: Packed<Int32> -> Packed<Int32> := (source) ->\n",
            "    source.edit<Int32>((buffer) -> {\n",
            "        buffer.put(0usize, buffer.get(0usize));\n",
            "        ();\n",
            "    });\n",
        )
    );
    assert_eq!(format(&formatted), formatted);
}

#[test]
fn indents_blocks_from_their_parenthesized_list_position() {
    let formatted = format(
        "parse := (source) ->\n(\nsource,\nmake<Frame>(0usize, (buffer) -> {\nbuffer.new(value);\n()\n}),\n1u64\n).parse();",
    );

    assert_eq!(
        formatted,
        concat!(
            "parse := (source) ->\n",
            "    (\n",
            "        source,\n",
            "        make<Frame>(0usize, (buffer) -> {\n",
            "            buffer.new(value);\n",
            "            ();\n",
            "        }),\n",
            "        1u64\n",
            "    ).parse();\n",
        )
    );
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
    assert_eq!(
        format("identity:=(x) -> {x;};"),
        "identity := (x) -> { x };\n"
    );
    assert_eq!(
        format("run:=() -> {first();second()};"),
        concat!(
            "run := () -> {\n",
            "    first();\n",
            "    second();\n",
            "};\n",
        )
    );
    assert_eq!(
        format("run:=() -> {first()// result\n};"),
        concat!("run := () -> {\n", "    first(); // result\n", "};\n",)
    );
}

#[test]
fn rejects_malformed_source() {
    let diagnostic = malc::formatter::format(&source("value := ;")).expect_err("syntax error");

    assert_eq!(diagnostic.message, "expected an expression");
    assert!(diagnostic.primary.is_some());
}

#[test]
fn formats_many_independent_blocks_in_one_pass() {
    let count = 4_096;
    let input = (0..count)
        .map(|index| format!("function{index}::Unit->Int32:=() -> {{0i32;}};\n"))
        .collect::<String>();

    let formatted = format(&input);

    assert_eq!(formatted.matches(" :: Unit -> Int32 :=").count(), count);
}

#[test]
fn formats_long_left_associative_expressions_without_host_recursion() {
    let expression = std::iter::repeat_n("0i32", 4_096)
        .collect::<Vec<_>>()
        .join("+");
    let input = format!("main::Unit->Int32:=() -> {{{expression};}};");

    let formatted = format(&input);

    assert_eq!(formatted.matches(" + ").count(), 4_095);
}
