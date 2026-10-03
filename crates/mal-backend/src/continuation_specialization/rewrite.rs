use std::collections::HashMap;

use crate::closure::ast::Program;
use crate::closure::rewrite::{Identities, are_unique, copy_functions};

use super::request::Request;

/// Materializes the requested worker copies without redirecting any executable path yet.
pub(super) fn copy_workers(program: &Program, request: &Request) -> Program {
    let mut rewritten = program.clone();
    let mut ids = Identities::after(&mut rewritten);
    for worker in &request.workers {
        assert_eq!(
            ids.function(),
            worker.worker,
            "validated request owns the next worker identities"
        );
    }
    let renaming = request
        .workers
        .iter()
        .map(|worker| (worker.original, worker.worker))
        .collect::<HashMap<_, _>>();
    let originals = request
        .workers
        .iter()
        .map(|worker| {
            program
                .functions
                .iter()
                .find(|function| function.id == worker.original)
                .expect("validated request names a program function")
                .clone()
        })
        .collect();
    rewritten
        .functions
        .extend(copy_functions(originals, renaming, &mut ids));
    debug_assert!(are_unique(&mut rewritten.clone()));
    rewritten
}
