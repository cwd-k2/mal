use mal_compiler::resolve::ast::{Expression, TopItem};
use mal_compiler::source::{FileId, SourceFile, SourceGraph, SourceRequirement};

fn make_graph(
    sources: &[(&str, &str)],
    requirements: &[&[(u32, usize)]],
) -> (SourceGraph, Vec<mal_compiler::ast::Program>) {
    let files = sources
        .iter()
        .enumerate()
        .map(|(index, (path, text))| {
            SourceFile::new(FileId::new(index as u32), path, (*text).into())
        })
        .collect::<Vec<_>>();
    let parsed = files
        .iter()
        .map(|source| mal_compiler::parser::parse(source).unwrap())
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
                "_helper :: Int32 -> Int32 := (x) -> { x + 1 }; left :: Int32 -> Int32 := (x) -> { _helper(x) };",
            ),
            (
                "right.mal",
                "_helper :: Int32 -> Int32 := (x) -> { x + 2 }; right :: Int32 -> Int32 := (x) -> { _helper(x) };",
            ),
        ],
        &[&[(1, 0), (2, 1)], &[], &[]],
    );

    let resolved = mal_compiler::resolve::resolve_graph(&graph, &parsed).unwrap();
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
                "require \"./leaf.mal\"; middle :: Int32 -> Int32 := (x) -> { leaf(x) };",
            ),
            ("leaf.mal", "leaf :: Int32 -> Int32 := (x) -> { x };"),
        ],
        &[&[(1, 0)], &[(2, 0)], &[]],
    );

    let error = mal_compiler::resolve::resolve_graph(&graph, &parsed).unwrap_err();
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
    let error = mal_compiler::resolve::resolve_graph(&graph, &parsed).unwrap_err();
    assert_eq!(error.message, "duplicate imported top-level value `same`");

    let (graph, parsed) = make_graph(
        &[
            ("root.mal", "require \"./dependency.mal\";"),
            ("dependency.mal", "main :: Unit -> Int32 := () -> { 0 };"),
        ],
        &[&[(1, 0)], &[] as &[(u32, usize)]],
    );
    let error = mal_compiler::resolve::resolve_graph(&graph, &parsed).unwrap_err();
    assert_eq!(error.message, "`main` declared outside the root file");
}

#[test]
fn resolves_deep_requirement_chains_without_host_recursion() {
    let depth = 4096;
    let files = (0..depth)
        .map(|index| {
            let text = if index + 1 == depth {
                format!("value{index} := 0;")
            } else {
                format!("require \"./file{}.mal\"; value{index} := 0;", index + 1)
            };
            SourceFile::new(FileId::new(index as u32), format!("file{index}.mal"), text)
        })
        .collect::<Vec<_>>();
    let parsed = files
        .iter()
        .map(|source| mal_compiler::parser::parse(source).unwrap())
        .collect::<Vec<_>>();
    let requirements = (0..depth)
        .map(|index| {
            if index + 1 == depth {
                Vec::new()
            } else {
                vec![SourceRequirement {
                    target: FileId::new((index + 1) as u32),
                    span: parsed[index].requirements[0].kind.path_span,
                }]
            }
        })
        .collect();
    let graph = SourceGraph::new(FileId::new(0), files, requirements, Vec::new());

    let resolved = mal_compiler::resolve::resolve_graph(&graph, &parsed).unwrap();

    assert_eq!(resolved.items.len(), depth);
}
