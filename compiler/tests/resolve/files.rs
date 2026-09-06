use malc::resolve::ast::{Expression, TopItem};
use malc::source::{FileId, SourceFile, SourceGraph, SourceRequirement};

fn make_graph(
    sources: &[(&str, &str)],
    requirements: &[&[(u32, usize)]],
) -> (SourceGraph, Vec<malc::ast::Program>) {
    let files = sources
        .iter()
        .enumerate()
        .map(|(index, (path, text))| {
            SourceFile::new(FileId::new(index as u32), path, (*text).into())
        })
        .collect::<Vec<_>>();
    let parsed = files
        .iter()
        .map(|source| malc::parser::parse(source).unwrap())
        .collect::<Vec<_>>();
    let edges = requirements
        .iter()
        .enumerate()
        .map(|(file, edges)| {
            edges
                .iter()
                .map(|(target, requirement)| SourceRequirement {
                    target: FileId::new(*target),
                    span: parsed[file].requirements[*requirement].kind.path_span,
                })
                .collect()
        })
        .collect();
    (
        SourceGraph::new(FileId::new(0), files, edges, Vec::new()),
        parsed,
    )
}

#[test]
fn resolves_public_names_and_keeps_private_names_per_file() {
    let (graph, parsed) = make_graph(
        &[
            (
                "root.mal",
                "require \"./left.mal\"; require \"./right.mal\"; result := left(1) + right(1);",
            ),
            (
                "left.mal",
                "_helper :: Int32 -> Int32 := \\(x :: Int32) { x + 1 }; left :: Int32 -> Int32 := \\(x :: Int32) { _helper(x) };",
            ),
            (
                "right.mal",
                "_helper :: Int32 -> Int32 := \\(x :: Int32) { x + 2 }; right :: Int32 -> Int32 := \\(x :: Int32) { _helper(x) };",
            ),
        ],
        &[&[(1, 0), (2, 1)], &[], &[]],
    );

    let resolved = malc::resolve::resolve_graph(&graph, &parsed).unwrap();
    assert_eq!(resolved.items.len(), 5);
    let TopItem::Binding(binding) = &resolved.items[4].kind else {
        panic!("expected the root binding");
    };
    assert!(matches!(binding.value.kind, Expression::Binary { .. }));
}

#[test]
fn does_not_reexport_imported_names() {
    let (graph, parsed) = make_graph(
        &[
            ("root.mal", "require \"./middle.mal\"; result := leaf(1);"),
            (
                "middle.mal",
                "require \"./leaf.mal\"; middle :: Int32 -> Int32 := \\(x :: Int32) { leaf(x) };",
            ),
            (
                "leaf.mal",
                "leaf :: Int32 -> Int32 := \\(x :: Int32) { x };",
            ),
        ],
        &[&[(1, 0)], &[(2, 0)], &[]],
    );

    let error = malc::resolve::resolve_graph(&graph, &parsed).unwrap_err();
    assert_eq!(error.message, "unknown value `leaf`");
    assert_eq!(error.primary.unwrap().span.file(), FileId::new(0));
}

#[test]
fn rejects_conflicting_imports_and_dependency_entry_points() {
    let (graph, parsed) = make_graph(
        &[
            (
                "root.mal",
                "require \"./left.mal\"; require \"./right.mal\";",
            ),
            ("left.mal", "same := 1;"),
            ("right.mal", "same := 2;"),
        ],
        &[&[(1, 0), (2, 1)], &[], &[]],
    );
    let error = malc::resolve::resolve_graph(&graph, &parsed).unwrap_err();
    assert_eq!(error.message, "duplicate imported top-level value `same`");

    let (graph, parsed) = make_graph(
        &[
            ("root.mal", "require \"./dependency.mal\";"),
            ("dependency.mal", "main :: Unit -> Int32 := \\() { 0 };"),
        ],
        &[&[(1, 0)], &[] as &[(u32, usize)]],
    );
    let error = malc::resolve::resolve_graph(&graph, &parsed).unwrap_err();
    assert_eq!(error.message, "`main` declared outside the root file");
}
