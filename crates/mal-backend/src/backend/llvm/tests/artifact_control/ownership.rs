use super::super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn shares_duplicate_owner_successors_once_before_canonical_transfer() {
    let source = SourceFile::new(
        FileId::new(93),
        "owner-successor-normalization.mal",
        "duplicate :: Unit -> (Symbol, Symbol) := () -> { value := \"a\" + \"b\"; (value, value); };\n\
         main :: Unit -> Int32 := () -> { pair := duplicate(); 0i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, crate::execution::OptimizationSet::none());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("ownership fixture is supported");

    assert_eq!(
        artifacts
            .module
            .matches("call ptr @mal_runtime_bytes_retain")
            .count(),
        1
    );
}

#[test]
fn invalidates_a_consumed_sum_before_releasing_discarded_payload_leaves() {
    let source = SourceFile::new(
        FileId::new(98),
        "case-payload-transfer-order.mal",
        "Choice :: [(Symbol, Symbol), Unit];\n\
         select :: Choice -> Symbol := (choice) -> {\n\
           choice[(keep, _) -> { keep }, () -> { \"fallback\" }]\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           result := select([only, empty] => { only(\"a\" + \"b\", \"c\" + \"d\") });\n\
           (#result).i32;\n\
         };"
        .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check case ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize case ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, crate::execution::OptimizationSet::none());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("case ownership fixture is supported");

    let arm = artifacts
        .module
        .split_once("\nmal_case_")
        .map(|(_, arm)| arm)
        .expect("case arm block");
    let invalidation = arm
        .find("zeroinitializer, ptr %mal_slot_")
        .expect("consumed sum invalidation");
    let release = arm
        .find("call void @mal_runtime_bytes_release")
        .expect("discarded payload leaf release");
    assert!(invalidation < release);
}
