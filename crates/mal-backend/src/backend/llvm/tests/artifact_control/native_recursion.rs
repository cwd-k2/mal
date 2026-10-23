use super::super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn self_recursive_functions_have_a_native_and_a_frames_version_only_when_the_technique_is_enabled()
{
    let source = SourceFile::new(
        FileId::new(102),
        "hybrid-recursion.mal",
        "fib :: Int64 -> Int64 := (n) -> { if (n < 2i64) then { n } else { fib(n - 1i64) + fib(n - 2i64); }; };\n\
         weigh :: Symbol -> Int64 := (s) -> { if (#s > 3usize) then { 0i64 } else { weigh(s + \"x\") + 1i64; }; };\n\
         main :: Unit -> Int32 := () -> { (fib(10i64) + weigh(\"a\")).i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check hybrid recursion");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize hybrid recursion"),
    );
    let anf = crate::anf::lower(&core);
    let execution = crate::execution::lower(
        crate::closure::convert(&anf),
        crate::execution::OptimizationSet::production(),
    );
    let module = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("hybrid recursion is supported")
    .module;

    let frames = |module: &str, prefix: &str| {
        module
            .lines()
            .filter(|line| line.contains("_frames(") && line.trim_start().starts_with(prefix))
            .count()
    };
    assert_eq!(
        frames(&module, "define"),
        2,
        "`fib` and `weigh` each have a frames version, whatever their parameter type"
    );
    assert_eq!(
        frames(&module, "%"),
        2,
        "each native version continues there once"
    );
    assert!(module.contains("call ptr @llvm.stacksave()"));
    assert!(module.contains("call i8 @mal_native_stack_is_deep(ptr %mal_context, ptr"));

    let baseline = crate::execution::lower(
        crate::closure::convert(&anf),
        crate::execution::OptimizationSet::none(),
    );
    let baseline_module = generate(
        &baseline,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("baseline hybrid recursion is supported")
    .module;
    assert!(
        !baseline_module.contains("_frames("),
        "baseline runs recursion on frames only"
    );
}

#[test]
fn native_worker_abi_carries_only_changing_parameter_leaves() {
    let source = SourceFile::new(
        FileId::new(109),
        "native-recursion-scalar.mal",
        "walk :: (Int64, Int64, Int64) -> Int64 := (fixed, scale, depth) -> {
           if (depth == 0i64) then { fixed } else {
             child := walk(fixed, scale, depth - 1i64);
             child + scale;
           };
         };
         main :: Unit -> Int32 := () -> { walk(1i64, 2i64, 2i64).i32 - 5i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check scalar native fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize scalar native fixture"),
    );
    let anf = crate::anf::lower(&core);
    let execution = crate::execution::lower(
        crate::closure::convert(&anf),
        crate::execution::OptimizationSet::production(),
    );
    let module = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("scalar native fixture is supported")
    .module;
    let worker = module
        .lines()
        .find(|line| line.starts_with("define internal") && line.contains("_native("))
        .expect("native worker definition");

    assert!(worker.contains("(ptr %mal_native_context, i64 %mal_native_parameter_0)"));
    assert!(!worker.contains("%mal_context"));
    assert!(module.contains("call ptr @llvm.stacksave()"));
    assert!(module.contains("_frames(ptr %mal_context, ptr %mal_control_top"));
}

#[test]
fn native_recursion_borrows_managed_parameter_leaves_preserved_by_every_self_edge() {
    let source = SourceFile::new(
        FileId::new(108),
        "native-recursion-borrow.mal",
        "walk :: ((Buffer<Int32>, Buffer<Int32>), Int32) -> Int32 := (index, depth) -> {
           (values, _) := index;
           if (depth == 0i32) then { values.get(0usize) } else {
             child := walk(index, depth - 1i32);
             child + values.get(0usize);
           };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           unused := make<Int32>(1usize);
           values.new(1i32);
           unused.new(0i32);
           walk((values, unused), 2i32) - 3i32;
         };"
        .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check native borrow fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize native borrow fixture"),
    );
    let anf = crate::anf::lower(&core);
    let execution = crate::execution::lower(
        crate::closure::convert(&anf),
        crate::execution::OptimizationSet::production(),
    );
    let module = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("native borrow fixture is supported")
    .module;
    let native = module
        .split("\ndefine internal ")
        .find(|definition| definition.contains("call ptr @llvm.stacksave()"))
        .expect("native recursive definition")
        .split_once("\n}\n")
        .map_or("", |(body, _)| body);

    assert!(!native.contains("mal_runtime_owner_retain"));
    assert!(!native.contains("mal_runtime_owner_release"));
}
