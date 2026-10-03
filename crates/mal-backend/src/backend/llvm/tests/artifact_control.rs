use super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn places_activation_temporaries_in_the_entry_block_before_recursive_back_edges() {
    let source = SourceFile::new(
        FileId::new(99),
        "generic-loop-entry-alloca.mal",
        "Choice :: [UInt64, UInt64];\n\
         choose :: UInt64 -> Choice := (value) -> [left, right] => left(value);\n\
         loop<A, B> :: (A, A -> [A, B]) -> B := (state, step) -> step(state)[(next) -> loop<A, B>(next, step), (result) -> result];\n\
         main :: Unit -> Int32 := () -> {\n\
           initial := make<Choice>(1usize);\n\
           initial.new(choose(1u64));\n\
           loop<(USize, Buffer<Choice>), Int32>((0usize, initial), (state) -> [next, done] => {\n\
             (index, values) := state;\n\
             when (index == 4usize) done(0i32);\n\
             value := values.get(0usize)[(left) -> left, (right) -> right];\n\
             next((index + value.usize, values));\n\
           });\n\
         };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check generic loop fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize generic loop"),
    );
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
    .expect("generic loop fixture is supported");

    for definition in artifacts.module.split("\ndefine internal ").skip(1) {
        let body = definition
            .split_once("\n}\n")
            .map_or(definition, |(body, _)| body);
        let mut left_entry = false;
        for line in body.lines() {
            if line.ends_with(':') && line != "entry:" {
                left_entry = true;
            }
            assert!(
                !left_entry || !line.contains(" = alloca "),
                "alloca outside the entry block: {line}"
            );
        }
    }
}

#[test]
fn borrows_managed_tail_carriers_from_the_outer_call() {
    let source = SourceFile::new(
        FileId::new(95),
        "llvm-managed-tail-carrier.mal",
        "walk :: (Symbol, Int32) -> Int32 := (text, remaining) -> { if (remaining == 0i32) then { (#text).i32 } else { walk((text, remaining - 1i32)) }; }; main :: Unit -> Int32 := () -> { walk((\"x\", 4i32)) - 1i32; };"
            .into(),
    );
    let checked =
        mal_frontend::analysis::check(&source).expect("check managed tail carrier fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
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
        let checked = mal_frontend::analysis::check(&source).expect("check wildcard fixture");
        let core = crate::core::lower(&mal_frontend::check::specialize(checked).expect("specialize checked program"));
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
    let checked = mal_frontend::analysis::check(&source).expect("check baseline fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
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

#[test]
fn nested_combinators_with_one_callback_signature_use_no_control_frames() {
    let combinator = |name: &str| {
        format!(
            "{name}<A, B> :: (A, A -> [A, B]) -> B := (state, step) -> step(state)[(next) -> {name}<A, B>(next, step), (result) -> result];\n"
        )
    };
    let source = SourceFile::new(
        FileId::new(100),
        "nested-combinators.mal",
        format!(
            "{}{}{}\
             main :: Unit -> Int32 := () -> {{\n\
               outer<UInt64, Int32>(0u64, (i) -> [continue, break] => {{\n\
                 when (i == 3u64) {{ break(0i32); }};\n\
                 middle<UInt64, Int32>(0u64, (j) -> [continue, break] => {{\n\
                   when (j == 3u64) {{ break(0i32); }};\n\
                   inner<UInt64, Int32>(0u64, (k) -> [continue, break] => {{\n\
                     when (k == 3u64) {{ break(0i32); }};\n\
                     continue(k + 1u64);\n\
                   }});\n\
                   continue(j + 1u64);\n\
                 }});\n\
                 continue(i + 1u64);\n\
               }});\n\
             }};",
            combinator("outer"),
            combinator("middle"),
            combinator("inner")
        ),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check nested combinators");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize nested combinators"),
    );
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
    .expect("nested combinators are supported");

    assert!(
        !artifacts.module.contains("mal_control_reserve_frame"),
        "the callbacks share one type but no callback can reach an enclosing combinator"
    );
}

#[test]
fn nested_uses_of_one_combinator_instance_use_no_control_frames() {
    let source = SourceFile::new(
        FileId::new(101),
        "shared-combinator.mal",
        "loop<A, B> :: (A, A -> [A, B]) -> B := (state, step) -> step(state)[(next) -> loop<A, B>(next, step), (result) -> result];\n\
         main :: Unit -> Int32 := () -> {\n\
           loop<UInt64, Int32>(0u64, (i) -> [continue, break] => {\n\
             when (i == 3u64) { break(0i32); };\n\
             loop<UInt64, Int32>(0u64, (j) -> [continue, break] => {\n\
               when (j == 3u64) { break(0i32); };\n\
               continue(j + 1u64);\n\
             });\n\
             continue(i + 1u64);\n\
           });\n\
         };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check shared combinator");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize shared combinator"),
    );
    let anf = crate::anf::lower(&core);
    let execution = |enabled| crate::execution::lower(crate::closure::convert(&anf), enabled);
    let generate = |execution: &crate::execution::Program| {
        generate(
            execution,
            Target {
                triple: "x86_64-unknown-linux-gnu",
                data_layout: "e-p:64:64",
            },
            OptimizationSet::production(),
        )
        .expect("shared combinator is supported")
        .module
    };

    assert!(
        generate(&execution(crate::execution::OptimizationSet::none()))
            .contains("mal_control_reserve_frame"),
        "without call-pattern specialization the two uses form one recursive region"
    );
    assert!(
        !generate(&execution(crate::execution::OptimizationSet::production()))
            .contains("mal_control_reserve_frame"),
        "each use of the instance gets a function of its own"
    );
}

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
