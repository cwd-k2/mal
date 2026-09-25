use mal_syntax::source::{FileId, SourceFile};

use crate::closure::ast::Program;

use super::{ids, specialize};

fn closure_program(text: &str) -> Program {
    let source = SourceFile::new(FileId::new(98), "call-pattern.mal", text.into());
    let checked = mal_frontend::analysis::check(&source).expect("check call pattern fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize call pattern fixture"),
    );
    crate::closure::convert(&crate::anf::lower(&core))
}

const FUNCTIONS: &str = "inc :: Int32 -> Int32 := (value) -> { value + 1i32; };\n\
     dec :: Int32 -> Int32 := (value) -> { value - 1i32; };\n\
     apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n";

fn counts(text: &str) -> (usize, usize) {
    let mut program = closure_program(text);
    let before = (program.functions.len(), program.bindings.len());
    program = specialize(program);
    (
        program.functions.len() - before.0,
        program.bindings.len() - before.1,
    )
}

#[test]
fn copies_a_function_for_each_distinct_closure_its_call_sites_pass() {
    let (functions, bindings) = counts(&format!(
        "{FUNCTIONS}main :: Unit -> Int32 := () -> {{ apply(inc, 1i32) + apply(dec, 2i32); }};"
    ));
    assert_eq!(
        (functions, bindings),
        (1, 1),
        "one copy of `apply` for `dec`"
    );
}

#[test]
fn keeps_one_function_when_every_call_site_passes_the_same_closure() {
    let (functions, bindings) = counts(&format!(
        "{FUNCTIONS}main :: Unit -> Int32 := () -> {{ apply(inc, 1i32) + apply(inc, 2i32); }};"
    ));
    assert_eq!((functions, bindings), (0, 0));
}

#[test]
fn gives_every_copied_binder_and_atom_an_identity_of_its_own() {
    let program = specialize(closure_program(&format!(
        "{FUNCTIONS}main :: Unit -> Int32 := () -> {{ apply(inc, 1i32) + apply(dec, 2i32) + apply((value) -> {{ value * 2i32; }}, 3i32); }};"
    )));
    assert!(ids::are_unique(&mut program.clone()));
}

#[test]
fn separates_combinators_that_nest_through_one_shared_instance() {
    let (functions, _) = counts(
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
         };",
    );
    assert!(functions >= 1, "the nested use of `loop` gets its own copy");
}
