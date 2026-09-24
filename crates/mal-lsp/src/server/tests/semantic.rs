use super::*;

#[test]
fn serves_hover_navigation_references_and_identity_safe_rename() {
    let text = "create :: Int32 -> Int32 := (x) -> {\n  inner :: Unit -> Int32 := () -> { x; };\n  inner();\n};\n";
    let uri = "file:///semantic.mal";
    let mut server = open_document(uri, text);
    let reference = text.find("{ x;").unwrap() + 2;
    let parameter = text.find("(x)").unwrap() + 1;

    let hover = request_at(&mut server, 10, "textDocument/hover", uri, text, reference);
    assert_eq!(hover["result"]["contents"]["kind"], "markdown");
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nx :: Int32\n```\n\nparameter\n\nDefined in `semantic.mal:1:30`"
    );
    assert_eq!(
        hover["result"]["range"]["start"],
        text_position(text, reference)
    );

    let definition = request_at(
        &mut server,
        11,
        "textDocument/definition",
        uri,
        text,
        reference,
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(text, parameter)
    );

    let position = text_position(text, reference);
    let references = server.handle(json!({
            "jsonrpc": "2.0", "id": 12, "method": "textDocument/references",
            "params": {"textDocument": {"uri": uri}, "position": position, "context": {"includeDeclaration": true}}
        }));
    assert_eq!(
        references.messages[0]["result"].as_array().unwrap().len(),
        2
    );

    let rename = server.handle(json!({
        "jsonrpc": "2.0", "id": 13, "method": "textDocument/rename",
        "params": {"textDocument": {"uri": uri}, "position": position, "newName": "renamed"}
    }));
    assert_eq!(
        rename.messages[0]["result"]["changes"][uri]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn preserves_declared_type_aliases_in_hover() {
    let text =
        "Tree :: (Int64, Address, Address);\nf :: (Tree, Int64) -> Int64 := (tree, n) -> { n; };\n";
    let uri = "file:///alias-hover.mal";
    let mut server = open_document(uri, text);

    let alias = request_at(
        &mut server,
        14,
        "textDocument/hover",
        uri,
        text,
        text.find("Tree").unwrap(),
    );
    assert_eq!(
        alias["result"]["contents"]["value"],
        "```mal\nTree :: (Int64, Address, Address)\n```\n\ntype\n\nDefined in `alias-hover.mal:1:1`"
    );

    let function = request_at(
        &mut server,
        15,
        "textDocument/hover",
        uri,
        text,
        text.find("f ::").unwrap(),
    );
    assert_eq!(
        function["result"]["contents"]["value"],
        "```mal\nf :: (Tree, Int64) -> Int64\n```\n\nfunction\n\nDefined in `alias-hover.mal:2:1`"
    );
}

#[test]
fn expands_a_sum_result_type_hover_by_exactly_one_alias_layer() {
    let text = "Payload :: Int32;\nChoice :: [Unit, Payload];\ncreate :: Payload -> Choice := (value) -> [none, some] => { some(value) };\nread :: Unit -> Choice := () -> { create(1) };\n";
    let uri = "file:///sum-result-hover.mal";
    let mut server = open_document(uri, text);
    let constructor = text.find("-> Choice").unwrap() + 3;
    let hover = request_at(
        &mut server,
        16,
        "textDocument/hover",
        uri,
        text,
        constructor,
    );

    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nChoice :: [Unit, Payload]\n```\n\ntype\n\nDefined in `sum-result-hover.mal:2:1`"
    );

    let function = request_at(
        &mut server,
        17,
        "textDocument/hover",
        uri,
        text,
        text.find("create ::").unwrap(),
    );
    assert_eq!(
        function["result"]["contents"]["value"],
        "```mal\ncreate :: Payload -> Choice\n```\n\nfunction\n\nDefined in `sum-result-hover.mal:3:1`"
    );

    let call = text.rfind("create(1)").unwrap();
    let result = request_at(
        &mut server,
        18,
        "textDocument/hover",
        uri,
        text,
        call + "create".len(),
    );
    assert_eq!(
        result["result"]["contents"]["value"],
        "```mal\ncreate(1) :: [Unit, Int32]\n```"
    );
}

#[test]
fn serves_typed_hover_for_a_byte_literal_containing_a_closing_parenthesis() {
    let text = "closingParen :: UInt8 := ')';";
    let uri = "file:///byte-hover.mal";
    let mut server = open_document(uri, text);
    let literal = text.find("')'").unwrap();
    let hover = request_at(
        &mut server,
        20,
        "textDocument/hover",
        uri,
        text,
        literal + 1,
    );

    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\n')' :: UInt8\n```"
    );
    assert_eq!(
        hover["result"]["range"],
        json!({
            "start": text_position(text, literal),
            "end": text_position(text, literal + "')'".len())
        })
    );
}

#[test]
fn serves_reference_documentation_for_predefined_memory_operations() {
    let text = "build :: Unit -> Buffer<Int32> := () -> make<Int32>(1usize);";
    let uri = "file:///predefined-hover.mal";
    let mut server = open_document(uri, text);

    let make = request_at(
        &mut server,
        21,
        "textDocument/hover",
        uri,
        text,
        text.find("make<Int32>").unwrap(),
    );
    let make_contents = make["result"]["contents"]["value"].as_str().unwrap();
    assert!(make_contents.contains("make :: USize -> Buffer<T>"));
    assert!(make_contents.contains("initial capacity"));

    let buffer = request_at(
        &mut server,
        22,
        "textDocument/hover",
        uri,
        text,
        text.find("Buffer<Int32>").unwrap(),
    );
    let buffer_contents = buffer["result"]["contents"]["value"].as_str().unwrap();
    assert!(buffer_contents.contains("Buffer<T>"));
    assert!(buffer_contents.contains("mutable mal-owned sequence"));

    let new = request_at(
        &mut server,
        23,
        "textDocument/hover",
        uri,
        text,
        text.find("make<Int32>").unwrap(),
    );
    let new_contents = new["result"]["contents"]["value"].as_str().unwrap();
    assert!(new_contents.contains("make :: USize -> Buffer<T>"));
}

#[test]
fn serves_hover_and_definition_for_an_alias_in_an_indexed_type() {
    let text = "Byte :: UInt8;\nread :: Buffer<Byte> -> Byte := (buffer) -> buffer.get(0usize);";
    let uri = "file:///indexed-type.mal";
    let mut server = open_document(uri, text);
    let reference = text.rfind("Byte").unwrap();
    let declaration = text.find("Byte").unwrap();

    let hover = request_at(&mut server, 21, "textDocument/hover", uri, text, reference);
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nByte :: UInt8\n```\n\ntype\n\nDefined in `indexed-type.mal:1:1`"
    );

    let definition = request_at(
        &mut server,
        22,
        "textDocument/definition",
        uri,
        text,
        reference,
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(text, declaration)
    );
}
