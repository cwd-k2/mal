use super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn emits_targeted_llvm_and_a_c_shim_from_one_bridge_plan() {
    let source = SourceFile::new(
        FileId::new(75),
        "llvm-constant.mal",
        "main :: Unit -> Int32 := () -> { 7; };".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check LLVM fixture");
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
    assert!(
        !artifacts
            .module
            .contains("declare ptr @mal_runtime_bytes_data")
    );
    assert!(
        !artifacts
            .runtime
            .iter()
            .any(|source| source.name == "bytes.c")
    );
    assert!(artifacts.shim.contains(
        "void mal_program_entry(void *mal_context, const void *mal_argument, void *mal_result);"
    ));
}

#[test]
fn adopts_managed_results_returned_by_c_extensions() {
    let source = SourceFile::new(
        FileId::new(109),
        "llvm-external-result.mal",
        "extern bytes :: Unit -> Buffer<UInt8>; main :: Unit -> Int32 := () -> { _ := bytes(); 0; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check external result fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let execution = crate::execution::lower(
        crate::closure::convert(&crate::anf::lower(&core)),
        crate::execution::OptimizationSet::none(),
    );
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("external result fixture is supported");

    assert!(
        artifacts
            .module
            .contains("call void @mal_bridge_external_0")
    );
    assert!(
        artifacts
            .module
            .contains("call void @mal_runtime_owner_release")
    );
    assert!(
        !artifacts
            .module
            .contains("call ptr @mal_runtime_owner_retain")
    );
}

#[test]
fn includes_byte_runtime_when_only_the_process_entry_shim_uses_it() {
    let source = SourceFile::new(
        FileId::new(105),
        "llvm-ignored-arguments.mal",
        "main :: Buffer<Symbol> -> Int32 := (_) -> { 0i32; };".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check argument fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let execution = crate::execution::lower(
        crate::closure::convert(&crate::anf::lower(&core)),
        crate::execution::OptimizationSet::production(),
    );
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::production(),
    )
    .expect("ignored process arguments are supported");

    assert!(artifacts.shim.contains("mal_runtime_buffer_from_arguments"));
    assert!(
        artifacts
            .runtime
            .iter()
            .any(|source| source.name == "bytes.c")
    );
}

#[test]
fn emits_symbol_partition_as_shared_range_views() {
    let source = SourceFile::new(
        FileId::new(106),
        "llvm-symbol-partition.mal",
        "main :: Unit -> Int32 := () -> { value := \"abcdef\"; prefix := value / 3usize; suffix := value % 3usize; if (prefix + suffix == value) then 0i32 else 1i32; };".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check Symbol partition fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let execution = crate::execution::lower(
        crate::closure::convert(&crate::anf::lower(&core)),
        crate::execution::OptimizationSet::none(),
    );
    let artifacts = generate(
        &execution,
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        },
        OptimizationSet::none(),
    )
    .expect("Symbol partition is supported");

    assert_eq!(
        artifacts
            .module
            .matches("call void @mal_runtime_symbol_slice")
            .count(),
        2
    );
    assert!(artifacts.module.contains("sub i64"));
    assert!(
        artifacts
            .runtime
            .iter()
            .any(|source| source.name == "symbol.c")
    );
}

#[test]
fn selects_the_entry_function_from_checked_identity() {
    let source = SourceFile::new(
        FileId::new(94),
        "llvm-entry-identity.mal",
        "main :: Unit -> Int32 := () -> { 7i32; };".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check LLVM fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
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
fn emits_long_left_associative_expressions_without_host_recursion() {
    let expression = std::iter::repeat_n("0i32", 4096)
        .collect::<Vec<_>>()
        .join(" + ");
    let source = SourceFile::new(
        FileId::new(76),
        "llvm-long-expression.mal",
        format!("main :: Unit -> Int32 := () -> {{ {expression}; }};"),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check long expression");
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
    let checked = mal_frontend::analysis::check(&source).expect("check long completion sequence");
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
    let checked = mal_frontend::analysis::check(&source).expect("check shared extern sum");
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
