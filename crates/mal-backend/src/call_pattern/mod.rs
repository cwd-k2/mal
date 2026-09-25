//! Call-pattern specialization: gives a top-level function that receives closures a copy for each distinct set of
//! closures its call sites pass.
//!
//! The closure flow is context insensitive, so one function called with several closures merges them all in its
//! parameters. Copying the function per closure set separates the contexts; the flow of the rewritten program then
//! names one closure at each call the copy makes, and higher-order combinators that nest without truly recurring
//! stop looking recursive. The pass repeats because a copy passes its own copies of inner closures to the
//! functions it calls, and it stops at a fixed point or when the program has grown past its budget.

use crate::closure::ast::Program;
use crate::control;
use crate::flow::{ClosureFlow, CompatibleTargets};

mod clone;
mod ids;
mod plan;
mod rewrite;
mod walk;

#[cfg(test)]
mod tests;

const MAX_ROUNDS: usize = 8;
/// A program may grow to this multiple of its function count, plus `GROWTH_SLACK`.
const GROWTH_FACTOR: usize = 4;
const GROWTH_SLACK: usize = 64;

pub(crate) fn specialize(mut program: Program) -> Program {
    let budget = program.functions.len() * GROWTH_FACTOR + GROWTH_SLACK;
    for _ in 0..MAX_ROUNDS {
        let lowered = control::lower(&program);
        let mut compatible = CompatibleTargets::new(&program);
        let flow = ClosureFlow::new(&program, &lowered, &mut compatible);
        let requests = plan::requests(&program, &lowered, &flow);
        if requests.is_empty() || !rewrite::apply(&mut program, requests, budget) {
            break;
        }
    }
    debug_assert!(ids::are_unique(&mut program.clone()));
    program
}
