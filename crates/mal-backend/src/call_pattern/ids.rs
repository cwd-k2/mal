use mal_frontend::resolve::ast::LambdaId;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, AtomKind, FunctionId, Program, Reference};

use super::walk::{self, Visitor};

/// Source of identities no part of the program uses yet.
pub(super) struct Identities {
    lambda: u32,
    temporary: u32,
    atom: usize,
}

impl Identities {
    pub(super) fn after(program: &mut Program) -> Self {
        let mut ids = Self {
            lambda: 0,
            temporary: 0,
            atom: 0,
        };
        walk::program(program, &mut ids);
        ids
    }

    pub(super) fn function(&mut self) -> FunctionId {
        self.lambda += 1;
        FunctionId::Lambda(LambdaId(self.lambda))
    }

    pub(super) fn value(&mut self) -> ValueId {
        self.temporary += 1;
        ValueId::Temporary(self.temporary)
    }

    pub(super) fn atom(&mut self) -> AtomId {
        self.atom += 1;
        AtomId(self.atom)
    }

    fn observe_value(&mut self, id: ValueId) {
        if let ValueId::Temporary(number) = id {
            self.temporary = self.temporary.max(number);
        }
    }

    fn observe_function(&mut self, id: FunctionId) {
        let FunctionId::Lambda(LambdaId(number)) = id;
        self.lambda = self.lambda.max(number);
    }
}

impl Visitor for Identities {
    fn binder(&mut self, id: &mut ValueId) {
        self.observe_value(*id);
    }

    fn atom(&mut self, atom: &mut Atom) {
        self.atom = self.atom.max(atom.id.0);
        match atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => self.observe_value(id),
            AtomKind::Reference(Reference::SelfClosure(function)) => {
                self.observe_function(function)
            }
            _ => {}
        }
    }

    fn created(&mut self, function: &mut FunctionId) {
        self.observe_function(*function);
    }

    fn defined(&mut self, function: &mut FunctionId) {
        self.observe_function(*function);
    }
}

/// Whether every binder and atom of the program has an identity no other occurrence shares.
pub(super) fn are_unique(program: &mut Program) -> bool {
    #[derive(Default)]
    struct Occurrences {
        binders: std::collections::HashSet<ValueId>,
        atoms: std::collections::HashSet<AtomId>,
        repeated: bool,
    }
    impl Visitor for Occurrences {
        fn binder(&mut self, id: &mut ValueId) {
            self.repeated |= !self.binders.insert(*id);
        }

        fn atom(&mut self, atom: &mut Atom) {
            self.repeated |= !self.atoms.insert(atom.id);
        }
    }
    let mut occurrences = Occurrences::default();
    walk::program(program, &mut occurrences);
    !occurrences.repeated
}
