//! Copies of a function and the closures it creates, with identities of their own.
//!
//! A copy that kept the original's binders would let one identity name two values, and the ownership plan keys its
//! facts by identity across the whole program. Every binder, atom, and function of a copy is therefore fresh.

use std::collections::{HashMap, HashSet};

use crate::closure::ast::{Function, FunctionId, Program};
use crate::closure::rewrite::{
    Identities, copy_functions,
    walk::{self, Visitor},
};

/// The function `root` and every closure it creates, transitively, copied under new identities.
pub(super) struct FunctionCopy {
    pub(super) functions: Vec<Function>,
    pub(super) renaming: HashMap<FunctionId, FunctionId>,
}

pub(super) fn copy_function(
    program: &Program,
    root: FunctionId,
    ids: &mut Identities,
) -> FunctionCopy {
    let mut functions = Vec::new();
    let mut pending = vec![root];
    let mut copied = HashSet::new();
    while let Some(id) = pending.pop() {
        if !copied.insert(id) {
            continue;
        }
        let mut function = program
            .functions
            .iter()
            .find(|function| function.id == id)
            .expect("a created closure names a function of the program")
            .clone();
        walk::function(&mut function, &mut Created(&mut pending));
        functions.push(function);
    }
    let renaming = functions
        .iter()
        .map(|function| (function.id, ids.function()))
        .collect::<HashMap<_, _>>();
    let functions = copy_functions(functions, renaming.clone(), ids);
    FunctionCopy {
        functions,
        renaming,
    }
}

struct Created<'a>(&'a mut Vec<FunctionId>);

impl Visitor for Created<'_> {
    fn created(&mut self, function: &mut FunctionId) {
        self.0.push(*function);
    }
}
