use super::super::*;
use crate::source::{FileId, SourceFile};

#[test]
fn borrows_managed_tail_carriers_from_the_outer_call() {
    let source = SourceFile::new(
        FileId::new(95),
        "llvm-managed-tail-carrier.mal",
        "walk :: (Symbol, Int32) -> Int32 := (text, remaining) -> { if (remaining == 0i32) then { (#text).i32 } else { walk((text, remaining - 1i32)) }; }; main :: Unit -> Int32 := () -> { walk((\"x\", 4i32)) - 1i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check managed tail carrier fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("managed tail carrier fixture is supported");

    assert_eq!(
        artifacts
            .module
            .matches("call void @mal_runtime_bytes_release")
            .count(),
        0
    );
}

#[test]
fn admits_direct_self_handoffs_to_wildcard_parameters() {
    for (index, source) in [
        "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { child := walk(1i32); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
        "extern again :: Unit -> Bool; create :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { child := walk(create(1i32)); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(create(0i32)); };",
        "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { walk(1i32) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
        "extern again :: Unit -> Bool; create :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { walk(create(1i32)) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(create(0i32)); };",
    ]
    .into_iter()
    .enumerate()
    {
        let source = SourceFile::new(
            FileId::new(80),
            "direct-self-wildcard.mal",
            source.into(),
        );
        let checked = crate::pipeline::check(&source).expect("check wildcard fixture");
        let core = crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution = crate::execution::lower(
            closure,
            crate::execution::OptimizationSet::production(),
        );

        assert!(supports(&execution), "unsupported wildcard fixture {index}");
    }
}

#[test]
fn admits_recursive_control_without_optional_execution_techniques() {
    let source = SourceFile::new(
        FileId::new(86),
        "baseline-recursion.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n\
         main :: Unit -> Int32 := () -> {\n\
           walk :: Int32 -> Int32 := (value) -> {\n\
             if (value == 0i32) then { 0i32 } else { apply(walk, value - 1i32) };\n\
           };\n\
           walk(4i32);\n\
         };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check baseline fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, crate::execution::OptimizationSet::none());

    assert!(supports(&execution));
}

#[test]
fn shares_duplicate_owner_successors_once_before_canonical_transfer() {
    let source = SourceFile::new(
        FileId::new(93),
        "owner-successor-normalization.mal",
        "duplicate :: Unit -> (Symbol, Symbol) := () -> { value := \"a\" + \"b\"; (value, value); };\n\
         main :: Unit -> Int32 := () -> { pair := duplicate(); 0i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check ownership fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize ownership fixture"),
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
    let checked = crate::pipeline::check(&source).expect("check case ownership fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize case ownership fixture"),
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
