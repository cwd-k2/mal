use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Function, FunctionId, Reference, TopLevelBinding};

use super::{
    Identities,
    walk::{self, Visitor},
};

/// Copies already selected functions under the supplied identities and freshens every local identity.
pub(crate) fn copy_functions(
    mut functions: Vec<Function>,
    renaming: HashMap<FunctionId, FunctionId>,
    ids: &mut Identities,
) -> Vec<Function> {
    let mut binders = Binders::default();
    for function in &mut functions {
        walk::function(function, &mut binders);
    }
    let mut rename = Rename::new(binders, renaming, ids);
    for function in &mut functions {
        walk::function(function, &mut rename);
    }
    functions
}

/// Copies a top-level binding and redirects functions according to `renaming`.
pub(crate) fn copy_top_level(
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
