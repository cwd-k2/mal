use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, AtomKind, Program, Reference};
use crate::closure::rewrite::{
    Identities, copy_top_level,
    walk::{self, Visitor},
};

use super::clone::copy_function;
use super::plan::Request;

/// Copies the requested functions and points their call sites at the copies; reports whether anything changed.
pub(super) fn apply(program: &mut Program, requests: Vec<Request>, budget: usize) -> bool {
    let mut ids = Identities::after(program);
    let mut redirects = HashMap::<AtomId, ValueId>::new();
    let mut inserted = Vec::new();
    for request in requests {
        if program.functions.len() >= budget {
            break;
        }
        let copy = copy_function(program, request.function, &mut ids);
        let original = program
            .bindings
            .iter()
            .position(|binding| {
                binding
                    .known_function()
                    .is_some_and(|(binding, _)| binding == request.binding)
            })
            .expect("a request names a known top-level function");
        let binding = copy_top_level(&program.bindings[original], copy.renaming, &mut ids);
        let (bound, _) = binding
            .known_function()
            .expect("the copy binds the copied function");
        for site in request.sites {
            redirects.insert(site, bound);
        }
        program.functions.extend(copy.functions);
        inserted.push((original, binding));
    }
    if inserted.is_empty() {
        return false;
    }
    // Later positions first, so earlier insertions keep their index.
    inserted.sort_by_key(|(position, _)| std::cmp::Reverse(*position));
    for (position, binding) in inserted {
        program.bindings.insert(position + 1, binding);
    }
    walk::program(program, &mut Redirect(&redirects));
    true
}

struct Redirect<'a>(&'a HashMap<AtomId, ValueId>);

impl Visitor for Redirect<'_> {
    fn atom(&mut self, atom: &mut Atom) {
        if let Some(binding) = self.0.get(&atom.id) {
            atom.kind = AtomKind::Reference(Reference::Binding(*binding));
        }
    }
}
