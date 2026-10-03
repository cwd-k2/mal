//! Copies of a function and the closures it creates, with identities of their own.
//!
//! A copy that kept the original's binders would let one identity name two values, and the ownership plan keys its
//! facts by identity across the whole program. Every binder, atom, and function of a copy is therefore fresh.

use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Function, FunctionId, Program, Reference, TopLevelBinding,
};
use crate::closure::rewrite::{
    Identities,
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
    let mut binders = Binders::default();
    for function in &mut functions {
        walk::function(function, &mut binders);
    }
    let mut rename = Rename::new(binders, renaming, ids);
    for function in &mut functions {
        walk::function(function, &mut rename);
    }
    let renaming = rename.functions;
    FunctionCopy {
        functions,
        renaming,
    }
}

/// A top-level binding of a capture-free function, rebound to the function copy.
pub(super) fn copy_top_level(
    binding: &TopLevelBinding,
    renaming: HashMap<FunctionId, FunctionId>,
    ids: &mut Identities,
) -> TopLevelBinding {
    let mut copy = binding.clone();
    let mut binders = Binders::default();
    walk::top_level(&mut copy, &mut binders);
    let mut rename = Rename::new(binders, renaming, ids);
    walk::top_level(&mut copy, &mut rename);
    copy
}

struct Created<'a>(&'a mut Vec<FunctionId>);

impl Visitor for Created<'_> {
    fn created(&mut self, function: &mut FunctionId) {
        self.0.push(*function);
    }
}

/// Collects the identities that the copied code binds.
#[derive(Default)]
struct Binders(HashSet<ValueId>);

impl Visitor for Binders {
    fn binder(&mut self, id: &mut ValueId) {
        self.0.insert(*id);
    }
}

struct Rename<'a> {
    values: HashMap<ValueId, ValueId>,
    functions: HashMap<FunctionId, FunctionId>,
    ids: &'a mut Identities,
}

impl<'a> Rename<'a> {
    fn new(
        binders: Binders,
        functions: HashMap<FunctionId, FunctionId>,
        ids: &'a mut Identities,
    ) -> Self {
        let mut bound = binders.0.into_iter().collect::<Vec<_>>();
        bound.sort_by_key(|id| format!("{id:?}"));
        let values = bound.into_iter().map(|id| (id, ids.value())).collect();
        Self {
            values,
            functions,
            ids,
        }
    }
}

impl Visitor for Rename<'_> {
    fn binder(&mut self, id: &mut ValueId) {
        *id = self.values[id];
    }

    fn atom(&mut self, atom: &mut Atom) {
        atom.id = self.ids.atom();
        match &mut atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => {
                if let Some(renamed) = self.values.get(id) {
                    *id = *renamed;
                }
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => {
                if let Some(renamed) = self.functions.get(function) {
                    *function = *renamed;
                }
            }
            _ => {}
        }
    }

    fn created(&mut self, function: &mut FunctionId) {
        if let Some(renamed) = self.functions.get(function) {
            *function = *renamed;
        }
    }

    fn defined(&mut self, function: &mut FunctionId) {
        *function = self.functions[function];
    }
}
