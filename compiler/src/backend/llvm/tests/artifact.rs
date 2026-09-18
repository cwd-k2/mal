use super::super::*;
use crate::source::{FileId, SourceFile};

#[test]
fn emits_targeted_llvm_and_a_c_shim_from_one_bridge_plan() {
    let source = SourceFile::new(
        FileId::new(75),
        "llvm-constant.mal",
        "main :: Unit -> Int32 := () -> { 7; };".into(),
    );
    let checked = crate::pipeline::check(&source).expect("check LLVM fixture");
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
    .expect("constant main is supported");

    assert!(matches!(
        generate(
            &execution,
            Target {
                triple: "x86_64-unknown-linux-gnu",
                data_layout: "e-p:7:8",
            },
            OptimizationSet::production(),
        ),
        Err(Error::InvalidTargetDataLayout)
    ));

    assert!(
        artifacts
            .module
            .contains("target triple = \"x86_64-unknown-linux-gnu\"")
    );
    assert!(artifacts.module.contains("ret i32 7"));
    assert!(artifacts.shim.contains(
        "void mal_program_entry(void *mal_context, const void *mal_argument, void *mal_result);"
    ));
}

#[test]
fn selects_the_entry_function_from_checked_identity() {
    let source = SourceFile::new(
        FileId::new(94),
        "llvm-entry-identity.mal",
        "main :: Unit -> Int32 := () -> { 7i32; };".into(),
    );
    let checked = crate::pipeline::check(&source).expect("check LLVM fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let mut closure = crate::closure::convert(&anf);
    let crate::closure::ast::TopLevelPattern::Binding { name, .. } =
        &mut closure.bindings[0].pattern
    else {
        panic!("entry is a binding");
    };
    name.clear();
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
    .expect("entry metadata is not reinterpreted by the backend");
    assert!(artifacts.module.contains("ret i32 7"));
}

#[test]
fn emits_typed_scalar_cursor_access_with_exact_alignment() {
    let source = SourceFile::new(
        FileId::new(89),
        "llvm-cursor.mal",
        "extern memory :: Unit -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           cursor := memory()@u64;\n\
           cursor <- 41u64;\n\
           value := <-cursor;\n\
           value.i32;\n\
         };"
        .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check cursor fixture");
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
    .expect("typed scalar cursor fixture is supported");

    assert!(artifacts.module.contains("store i64 %mal_value"));
    assert!(artifacts.module.contains("load i64, ptr"));
    assert!(artifacts.module.contains("align 1"));
    assert!(artifacts.header.contains("mal_Address_return"));
}

#[test]
fn emits_packed_views_indexing_and_symbol_conversion() {
    let source = SourceFile::new(
        FileId::new(91),
        "llvm-packed.mal",
        "main :: Unit -> Int32 := () -> {\n\
           packed := *\"abc\";\n\
           prefix := packed / 2usize;\n\
           byte := prefix # 1usize;\n\
           text := *prefix;\n\
           byte.i32 + (#text).i32;\n\
         };"
        .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check Packed fixture");
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
    .expect("Packed fixture is supported");

    assert!(artifacts.module.contains("@mal_runtime_bytes_data"));
    assert_eq!(
        artifacts
            .module
            .matches("call ptr @mal_runtime_bytes_retain")
            .count(),
        1
    );
    assert!(
        artifacts
            .module
            .contains("call void @mal_runtime_bytes_release")
    );
}

#[test]
fn emits_shared_scoped_packed_capabilities_without_owned_environments() {
    let source = SourceFile::new(
        FileId::new(93),
        "llvm-packed-builder.mal",
        "fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (new, get, put) -> { index := new(1i64); put(index, get(index)); (); }; main :: Unit -> Int32 := () -> { first := pack<Int64>(fill); second := pack<Int64>(fill); ((first # 0usize) + (second # 0usize)).i32 - 2i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check scoped Packed fixture");
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
    .expect("scoped Packed fixture is supported");

    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_environment_allocate")
    );
    assert_eq!(
        artifacts
            .module
            .matches("call ptr @mal_runtime_packed_builder_get")
            .count(),
        1
    );
    assert_eq!(
        artifacts
            .module
            .matches("call void @mal_runtime_packed_builder_put")
            .count(),
        1
    );
    assert!(
        artifacts
            .module
            .contains("ptr @mal_runtime_packed_builder_get(ptr %mal_context, ptr")
    );
    assert!(artifacts.module.contains(", i64 8)"));
}

#[test]
fn emits_long_left_associative_expressions_without_host_recursion() {
    let expression = std::iter::repeat_n("0i32", 4096)
        .collect::<Vec<_>>()
        .join(" + ");
    let source = SourceFile::new(
        FileId::new(76),
        "llvm-long-expression.mal",
        format!("main :: Unit -> Int32 := () -> {{ {expression}; }};"),
    );
    let checked = crate::pipeline::check(&source).expect("check long expression");
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
    .expect("long expression is supported");

    assert!(artifacts.module.contains("mal_function_"));
}

#[test]
fn emits_long_completion_control_sequences_without_ast_duplication() {
    let source = SourceFile::new(
        FileId::new(78),
        "llvm-long-completion.mal",
        format!(
            "main :: Unit -> Int32 := () -> [return] => {{ {}return(0i32) }};",
            "when (false) { return(1i32) };".repeat(1_024)
        ),
    );
    let checked = crate::pipeline::check(&source).expect("check long completion sequence");
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
    .expect("long completion sequence is supported");

    assert!(artifacts.module.contains("mal_function_"));
}

#[test]
fn emits_shared_extern_sum_helpers_once_per_type() {
    let mut declarations = String::from("Choice0 :: [UInt8, UInt8];\n");
    for depth in 1..16 {
        declarations.push_str(&format!(
            "Choice{depth} :: [Choice{}, Choice{}];\n",
            depth - 1,
            depth - 1
        ));
    }
    declarations.push_str(
        "extern exchange :: Choice15 -> Choice15;\n\
         main :: Unit -> Int32 := () -> { 0i32; };",
    );
    let source = SourceFile::new(FileId::new(77), "llvm-shared-extern-sum.mal", declarations);
    let checked = crate::pipeline::check(&source).expect("check shared extern sum");
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
    .expect("shared extern sum is supported");

    assert!(artifacts.shim.len() < 250_000);
    assert_eq!(
        artifacts
            .shim
            .matches("static void mal_bridge_external_0_write_sum_")
            .count(),
        16
    );
}

#[test]
fn admits_direct_self_handoffs_to_wildcard_parameters() {
    for (index, source) in [
        "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { child := walk(1i32); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
        "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { child := walk(make(1i32)); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(make(0i32)); };",
        "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { walk(1i32) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
        "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { walk(make(1i32)) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(make(0i32)); };",
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
fn selects_symbol_storage_reuse_only_when_enabled() {
    let source = SourceFile::new(
        FileId::new(87),
        "symbol-concat-optimization.mal",
        "main :: Unit -> Int32 := () -> { prefix := \"a\" + \"b\"; text := prefix + \"c\"; (#text).i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check Symbol concat fixture");
    let core =
        crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let target = || Target {
        triple: "x86_64-unknown-linux-gnu",
        data_layout: "e-p:64:64",
    };
    let baseline = generate(&execution, target(), OptimizationSet::none())
        .expect("baseline Symbol concat is supported");
    let optimized = generate(&execution, target(), OptimizationSet::production())
        .expect("optimized Symbol concat is supported");

    assert!(
        !baseline
            .module
            .contains("call void @mal_runtime_symbol_concatenate_consuming_left")
    );
    assert!(
        baseline
            .module
            .contains("call void @mal_runtime_symbol_concatenate(")
    );
    assert!(
        optimized
            .module
            .contains("call void @mal_runtime_symbol_concatenate_consuming_left")
    );
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
