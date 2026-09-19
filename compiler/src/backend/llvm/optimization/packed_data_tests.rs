use crate::core::ast::PackedBuilderOperation;
use crate::source::{FileId, SourceFile};

use super::super::{OptimizationPlan, OptimizationSet, Technique};
use super::StablePackedAccessPlan;

#[test]
fn stabilizes_unique_access_after_growth_has_finished() {
    let execution = lower(
        "main :: Unit -> Int32 := () -> { values := pack<Int32>((new, get, put) -> { new(1i32); value := get(0usize); put(0usize, value + 1i32); (); }); (values # 0usize); };",
    );
    let plan = StablePackedAccessPlan::new(&execution);

    assert!(
        sites(&execution, PackedBuilderOperation::Get)
            .all(|site| plan.stable_access(site).is_some())
    );
    assert!(
        sites(&execution, PackedBuilderOperation::PutUnique)
            .all(|site| plan.stable_access(site).is_some())
    );
    assert!(sites(&execution, PackedBuilderOperation::Get).all(|site| {
        StablePackedAccessPlan::empty()
            .stable_access(site)
            .is_none()
    }));
}

#[test]
fn validates_the_exact_optional_stable_access_decisions() {
    let execution = lower(
        "main :: Unit -> Int32 := () -> { values := pack<Int32>((new, get, _) -> { new(1i32); value := get(0usize); (); }); (values # 0usize); };",
    );
    let enabled = OptimizationSet::none().with(Technique::StablePackedAccess);
    let mut plan = OptimizationPlan::new(&execution, enabled);
    let site = sites(&execution, PackedBuilderOperation::Get)
        .next()
        .expect("Get application");

    assert!(plan.stable_packed_access(site).is_some());
    assert!(plan.is_valid(&execution, enabled));
    assert!(!plan.is_valid(&execution, OptimizationSet::none()));

    plan.packed_data.stable_applications.remove(&site);
    assert!(!plan.is_valid(&execution, enabled));
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .stable_packed_access(site)
            .is_none()
    );
}

#[test]
fn keeps_access_dynamic_when_unique_growth_can_follow() {
    let execution = lower(
        "main :: Unit -> Int32 := () -> { values := pack<Int32>((new, get, _) -> { new(1i32); value := get(0usize); new(value); (); }); (values # 0usize); };",
    );
    let plan = StablePackedAccessPlan::new(&execution);

    assert!(
        sites(&execution, PackedBuilderOperation::Get)
            .all(|site| plan.stable_access(site).is_none())
    );
}

#[test]
fn keeps_edit_access_dynamic_across_copy_on_write() {
    let execution = lower(
        "main :: Unit -> Int32 := () -> { original := pack<Int32>((new, _, _) -> { new(1i32); (); }); changed := edit<Int32>(original, (_, get, put) -> { value := get(0usize); put(0usize, value + 1i32); (); }); (changed # 0usize); };",
    );
    let plan = StablePackedAccessPlan::new(&execution);

    assert!(
        sites(&execution, PackedBuilderOperation::Get)
            .all(|site| plan.stable_access(site).is_none())
    );
    assert!(
        sites(&execution, PackedBuilderOperation::Put)
            .all(|site| plan.stable_access(site).is_none())
    );
}

#[test]
fn rejects_a_mixed_ordinary_and_capability_target_set() {
    let execution = lower(
        "fallback :: USize -> Int32 := (index) -> { index.i32; }; main :: Unit -> Int32 := () -> { values := pack<Int32>((new, get, _) -> { new(1i32); reader := if (true) then { get } else { fallback }; value := reader(0usize); (); }); (values # 0usize); };",
    );
    let plan = StablePackedAccessPlan::new(&execution);

    let mixed = execution.applications.sites().find(|(site, _)| {
        let targets = execution.applications.targets(*site).unwrap_or_default();
        targets.len() > 1
            && targets.iter().any(|target| {
                execution.lowered.functions.iter().any(|function| {
                    function.id == *target
                        && matches!(
                            function.kind,
                            crate::closure::ast::FunctionKind::PackedCapability {
                                operation: PackedBuilderOperation::Get,
                                ..
                            }
                        )
                })
            })
    });
    let (site, _) = mixed.expect("application with ordinary and Get targets");
    assert!(plan.stable_access(site).is_none());
}

fn lower(text: &str) -> crate::execution::Program {
    let source = SourceFile::new(FileId::new(108), "packed-data-plan.mal", text.into());
    let checked = crate::pipeline::check(&source).expect("check Packed data fixture");
    let specialized = crate::check::specialize(checked).expect("specialize Packed data fixture");
    let core = crate::core::lower(&specialized);
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::execution::lower(closure, crate::execution::OptimizationSet::production())
}

fn sites(
    execution: &crate::execution::Program,
    operation: PackedBuilderOperation,
) -> impl Iterator<Item = crate::control::ast::StateId> + '_ {
    execution.applications.sites().filter_map(move |(site, _)| {
        execution
            .applications
            .targets(site)
            .is_some_and(|targets| {
                targets.iter().any(|target| {
                    execution.lowered.functions.iter().any(|function| {
                        function.id == *target
                            && matches!(
                                function.kind,
                                crate::closure::ast::FunctionKind::PackedCapability {
                                    operation: candidate,
                                    ..
                                } if candidate == operation
                            )
                    })
                })
            })
            .then_some(site)
    })
}
