use mal_syntax::source::{FileId, SourceFile};

use crate::closure::ast::{AtomKind, Pattern, Reference};
use crate::control::ast::Operation;

use super::super::{OptimizationSet, Program, lower};

fn plan(text: &str) -> Program {
    let source = SourceFile::new(FileId::new(97), "environment-alias.mal", text.into());
    let checked = mal_frontend::analysis::check(&source).expect("check environment fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize environment fixture"),
    );
    let anf = crate::anf::lower(&core);
    lower(crate::closure::convert(&anf), OptimizationSet::production())
}

/// The managed leaves bound by the destructuring of a captured value, in source order.
fn captured_leaves(program: &Program) -> Vec<crate::anf::ast::ValueId> {
    let mut leaves = Vec::new();
    for state in &program.control.states {
        for binding in &state.bindings {
            let Operation::Atom(atom) = &binding.operation else {
                continue;
            };
            if !matches!(atom.kind, AtomKind::Reference(Reference::Capture(_))) {
                continue;
            }
            collect_bindings(&binding.pattern, &mut leaves);
        }
    }
    leaves
}

fn collect_bindings(pattern: &Pattern, leaves: &mut Vec<crate::anf::ast::ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => leaves.push(*id),
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_bindings(element, leaves);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

const APPLY: &str =
    "apply :: ((Int64 -> Int64), Int64) -> Int64 := (function, value) -> { function(value); };\n";

#[test]
fn borrows_a_managed_value_read_from_a_capture() {
    let program = plan(&format!(
        "{APPLY}main :: Unit -> Int32 := () -> {{\n\
           pair := (make<Int64>(1usize), make<Int64>(2usize));\n\
           total := apply((value) -> {{ (first, second) := pair; value + (#first).i64 + (#second).i64; }}, 1i64);\n\
           total.i32;\n\
         }};"
    ));
    let leaves = captured_leaves(&program);
    assert!(!leaves.is_empty());
    assert!(
        leaves
            .iter()
            .all(|leaf| program.ownership.binding_is_borrowed(*leaf)),
        "a capture read needs no reference of its own"
    );
}

#[test]
fn takes_a_reference_when_a_captured_value_reaches_a_tail_call() {
    let program = plan(&format!(
        "{APPLY}length :: Buffer<Int64> -> Int64 := (buffer) -> {{ (#buffer).i64; }};\n\
         main :: Unit -> Int32 := () -> {{\n\
           pair := (make<Int64>(1usize), 0i64);\n\
           total := apply((value) -> {{ (buffer, _) := pair; length(buffer); }}, 1i64);\n\
           total.i32;\n\
         }};"
    ));
    let leaves = captured_leaves(&program);
    assert!(!leaves.is_empty());
    assert!(
        leaves
            .iter()
            .all(|leaf| !program.ownership.binding_is_borrowed(*leaf)),
        "the tail call runs after the environment is released"
    );
}
