use super::*;

#[test]
fn exits_mark_the_choices_that_leave_the_block() {
    let text = "Res :: [Int32, Unit];\npick :: Res -> Res := (r) -> [ok, fail] => {\n  v := r[(n) -> n, () -> fail()];\n  when (v == 0i32) { ok(0i32) };\n  w := if (v == 1i32) then 5i32 else ok(1i32);\n  r[ok, fail]\n};\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    let exits = document
        .exits()
        .map(|exit| &text[exit.span.start()..exit.span.end()])
        .collect::<Vec<_>>();

    assert_eq!(
        exits,
        ["() -> fail()", "{ ok(0i32) }", "ok(1i32)", "r[ok, fail]"]
    );
}

#[test]
fn a_choice_where_every_side_continues_has_no_exit() {
    let text = "pick :: Bool -> Int32 := (c) -> { if (c) then 1i32 else 2i32; };\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    assert_eq!(document.exits().count(), 0);
}

fn exit_texts(text: &str) -> Vec<String> {
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    document
        .exits()
        .map(|exit| text[exit.span.start()..exit.span.end()].to_owned())
        .collect()
}

fn exit_targets(text: &str) -> Vec<(String, Vec<String>)> {
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    document
        .exits()
        .map(|exit| {
            (
                text[exit.span.start()..exit.span.end()].to_owned(),
                exit.targets.clone(),
            )
        })
        .collect()
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn exits_name_the_result_binders_they_transfer_to() {
    let prelude = "Res :: [Int32, Unit];\npick :: Res -> Res := (r) -> [ok, fail] => {\n";
    assert_eq!(
        exit_targets(&format!(
            "{prelude}  v := r[(n) -> n, () -> fail()];\n  when (v == 0i32) {{ ok(0i32) }};\n  r[ok, fail]\n}};\n"
        )),
        [
            ("() -> fail()".to_owned(), names(&["fail"])),
            ("{ ok(0i32) }".to_owned(), names(&["ok"])),
            ("r[ok, fail]".to_owned(), names(&["ok", "fail"])),
        ]
    );
}

#[test]
fn an_exit_names_the_outer_result_block_it_crosses_to() {
    // The inner result block has its own binder; leaving to `outer` crosses it, so the name tells the two apart.
    let text = "f :: Int32 -> Int32 := (v) -> [outer] => {\n  x :: Int32 := [inner] => {\n    when (v == 0i32) { outer(1i32) };\n    inner(2i32)\n  };\n  outer(x)\n};\n";
    assert_eq!(
        exit_targets(text),
        [("{ outer(1i32) }".to_owned(), names(&["outer"]))]
    );
}

#[test]
fn a_side_that_never_returns_names_no_binder() {
    let text = "Res :: [Int32, []];\nf :: Res -> Int32 := (r) -> [return] => {\n  x := r[(n) -> n, (e) -> e[]];\n  return(x)\n};\n";
    assert_eq!(exit_targets(text), [("(e) -> e[]".to_owned(), Vec::new())]);
}

#[test]
fn a_unit_that_leaves_the_block_is_marked_once_at_its_end() {
    let prelude = "Res :: [Int32, Unit];\npick :: Res -> Res := (r) -> [ok, fail] => {\n";
    // The body of a `when` ends in a choice on result binders: the body is marked, not the choice inside it.
    assert_eq!(
        exit_texts(&format!(
            "{prelude}  when (true) {{\n    r[ok, fail]\n  }};\n  r[ok, fail]\n}};\n"
        )),
        ["{\n    r[ok, fail]\n  }", "r[ok, fail]"]
    );
    // A leaving continuation ends where its inner choice ends, so only the continuation is marked.
    assert_eq!(
        exit_texts(&format!(
            "{prelude}  x := r[(n) -> n, () -> r[ok, fail]];\n  ok(x)\n}};\n"
        )),
        ["() -> r[ok, fail]"]
    );
    // Every side leaves: the whole choice is marked, not each side.
    assert_eq!(
        exit_texts(&format!(
            "{prelude}  if (true) then r[ok, fail] else r[ok, fail]\n}};\n"
        )),
        ["if (true) then r[ok, fail] else r[ok, fail]"]
    );
}

#[test]
fn an_exit_inside_a_branch_that_continues_is_kept() {
    let text = "Res :: [Int32, Unit];\npick :: Res -> Res := (r) -> [ok, fail] => {\n  x := r[(n) -> {\n    when (n == 0i32) { fail() };\n    n\n  }, () -> fail()];\n  ok(x)\n};\n";
    assert_eq!(exit_texts(text), ["{ fail() }", "() -> fail()"]);
}

#[test]
fn an_early_exit_inside_a_leaving_unit_stays_visible() {
    let prelude = "Res :: [Int32, Unit];\nf :: Res -> Res := (r) -> [ok, fail] => {\n";
    // A `when` inside the body of a `when`: the outer body always leaves at its end, the inner one only sometimes.
    assert_eq!(
        exit_targets(&format!(
            "{prelude}  when (true) {{\n    when (false) {{ fail() }};\n    ok(1i32)\n  }};\n  r[ok, fail]\n}};\n"
        )),
        [
            (
                "{\n    when (false) { fail() };\n    ok(1i32)\n  }".to_owned(),
                names(&["ok"])
            ),
            ("{ fail() }".to_owned(), names(&["fail"])),
            ("r[ok, fail]".to_owned(), names(&["ok", "fail"])),
        ]
    );
    // A leaving continuation of a choice that also has a continuing one keeps its own early exit.
    assert_eq!(
        exit_targets(&format!(
            "{prelude}  v := r[(n) -> n, () -> {{ when (true) {{ ok(0i32) }}; fail() }}];\n  ok(v)\n}};\n"
        )),
        [
            (
                "() -> { when (true) { ok(0i32) }; fail() }".to_owned(),
                names(&["fail"])
            ),
            ("{ ok(0i32) }".to_owned(), names(&["ok"])),
        ]
    );
}

#[test]
fn an_early_exit_inside_a_choice_on_result_binders_stays_visible() {
    let text = "Res :: [Int32, Unit];\nf :: Res -> Res := (r) -> [ok, fail] => {\n  r[(n) -> {\n    when (n == 0i32) { fail() };\n    ok(n)\n  }, () -> fail()]\n};\n";
    let exits = exit_targets(text);
    assert_eq!(exits.len(), 2, "{exits:?}");
    assert!(exits[0].0.starts_with("r[(n) ->"), "{exits:?}");
    assert_eq!(exits[0].1, names(&["ok", "fail"]));
    assert_eq!(exits[1], ("{ fail() }".to_owned(), names(&["fail"])));
}
