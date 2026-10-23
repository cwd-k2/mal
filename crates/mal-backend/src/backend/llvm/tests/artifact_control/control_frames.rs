use super::super::super::*;
use mal_syntax::source::{FileId, SourceFile};

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
