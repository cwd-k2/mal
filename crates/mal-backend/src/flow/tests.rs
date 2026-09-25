use mal_syntax::source::{FileId, SourceFile};

use crate::execution::{OptimizationSet, lower};

/// The number of targets of each application whose callee is not a statically known function, in site order.
fn indirect_target_counts(text: &str) -> Vec<usize> {
    let source = SourceFile::new(FileId::new(96), "closure-flow.mal", text.into());
    let checked = mal_frontend::analysis::check(&source).expect("check closure flow fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize closure flow fixture"),
    );
    let anf = crate::anf::lower(&core);
    let program = lower(crate::closure::convert(&anf), OptimizationSet::none());
    let mut sites = program
        .applications
        .sites()
        .filter(|(site, _)| program.applications.direct_target(*site).is_none())
        .map(|(site, _)| site)
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| site.0);
    sites
        .into_iter()
        .map(|site| {
            program
                .applications
                .targets(site)
                .expect("site targets")
                .len()
        })
        .collect()
}

const FUNCTIONS: &str = "inc :: Int32 -> Int32 := (value) -> { value + 1i32; };\n\
     dec :: Int32 -> Int32 := (value) -> { value - 1i32; };\n";

#[test]
fn narrows_an_indirect_site_to_the_functions_that_reach_its_callee() {
    let counts = indirect_target_counts(&format!(
        "{FUNCTIONS}apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> {{ function(value); }};\n\
         main :: Unit -> Int32 := () -> {{ apply(inc, 1i32) + dec(2i32); }};"
    ));
    assert_eq!(counts, [1], "only `inc` reaches `function`");
}

#[test]
fn keeps_every_function_that_reaches_a_shared_callee() {
    let counts = indirect_target_counts(&format!(
        "{FUNCTIONS}apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> {{ function(value); }};\n\
         main :: Unit -> Int32 := () -> {{ apply(inc, 1i32) + apply(dec, 2i32); }};"
    ));
    assert_eq!(counts, [2]);
}

#[test]
fn follows_a_function_through_an_aggregate_a_capture_and_a_result() {
    let counts = indirect_target_counts(&format!(
        "{FUNCTIONS}Pair :: ((Int32 -> Int32), Int32);\n\
         choose :: Pair -> (Int32 -> Int32) := ((function, _)) -> {{ (value) -> {{ function(value); }}; }};\n\
         main :: Unit -> Int32 := () -> {{ choose((inc, 0i32))(1i32) + dec(2i32); }};"
    ));
    assert!(
        !counts.is_empty() && counts.iter().all(|count| *count == 1),
        "every indirect site has one target: {counts:?}"
    );
}
