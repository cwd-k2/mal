use super::*;

#[test]
fn completes_requirement_paths_while_the_symbol_is_unclosed() {
    let files = TestFiles::new();
    std::fs::create_dir(files.path.join("library")).expect("create library directory");
    files.write("library/util.mal", "value :: Int32 := 1;\n");
    files.write("library/util.c", "");
    files.write("library/util.txt", "");
    std::fs::create_dir(files.path.join("library/utilities")).expect("create nested directory");
    let text = "require \"library/ut";
    let root_path = files.write("program.mal", text);
    let uri = path_to_uri(&root_path);
    let mut server = open_document(&uri, text);

    let completion = request_at(
        &mut server,
        25,
        "textDocument/completion",
        &uri,
        text,
        text.len(),
    );
    let items = completion["result"].as_array().unwrap();
    let labels = items
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(labels, ["util.c", "util.mal", "utilities/"]);
    assert!(
        items.iter().all(|item| {
            item["textEdit"]["range"]["start"] == json!({"line": 0, "character": 17})
        })
    );
    assert!(items.iter().any(|item| item["kind"] == 19));
}

#[test]
fn navigates_from_a_requirement_path_even_when_semantic_analysis_fails() {
    let files = TestFiles::new();
    let library_path = files.write("library.mal", "value :: Int32 := 1;\n");
    let text = "require \"library.mal\";\nbroken :: Int32 := ;\n";
    let root_path = files.write("program.mal", text);
    let root_uri = path_to_uri(&root_path);
    let mut server = open_document(&root_uri, text);

    let definition = request_at(
        &mut server,
        26,
        "textDocument/definition",
        &root_uri,
        text,
        text.find("library").unwrap(),
    );

    assert_eq!(definition["result"]["uri"], path_to_uri(&library_path));
    assert_eq!(
        definition["result"]["range"],
        json!({
            "start": {"line": 0, "character": 0},
            "end": {"line": 0, "character": 0}
        })
    );
}
