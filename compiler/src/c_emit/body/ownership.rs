use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Block, Operation, Pattern, Reference};

use super::has_direct_tail_call;

pub(super) struct OwnershipPlan {
    last_owned_uses: HashSet<*const Atom>,
}

impl OwnershipPlan {
    pub(super) fn new(program: &closure::Program) -> Self {
        let mut plan = Self {
            last_owned_uses: HashSet::new(),
        };
        for binding in &program.bindings {
            plan.analyze_root(&binding.value, None);
        }
        for function in &program.functions {
            let owned_parameter = has_direct_tail_call(&function.body, function.id)
                .then_some(function.parameter.binding)
                .flatten();
            plan.analyze_root(&function.body, owned_parameter);
        }
        plan
    }

    pub(super) fn can_transfer(&self, atom: &Atom) -> bool {
        self.last_owned_uses.contains(&atom_key(atom))
    }

    fn analyze_root(&mut self, block: &Block, owned_parameter: Option<ValueId>) {
        let mut owned = HashSet::new();
        collect_owned_bindings(block, &mut owned);
        if let Some(parameter) = owned_parameter {
            owned.insert(parameter);
        }
        self.analyze_block(block, &owned, &mut HashSet::new());
    }

    fn analyze_block(
        &mut self,
        block: &Block,
        owned: &HashSet<ValueId>,
        live_after: &mut HashSet<ValueId>,
    ) {
        self.analyze_atom(&block.result, owned, live_after);
        for binding in block.bindings.iter().rev() {
            remove_pattern_bindings(&binding.pattern, live_after);
            self.analyze_operation(&binding.operation, owned, live_after);
        }
    }

    fn analyze_operation(
        &mut self,
        operation: &Operation,
        owned: &HashSet<ValueId>,
        live_after: &mut HashSet<ValueId>,
    ) {
        match operation {
            Operation::Atom(atom)
            | Operation::SymbolLength { value: atom }
            | Operation::NumericConversion { operand: atom }
            | Operation::PrimitiveUnary { operand: atom, .. } => {
                self.analyze_atom(atom, owned, live_after);
            }
            Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
                for atom in captures.iter().rev() {
                    self.analyze_atom(atom, owned, live_after);
                }
            }
            Operation::Call { callee, argument } => {
                self.analyze_atom(argument, owned, live_after);
                self.analyze_atom(callee, owned, live_after);
            }
            Operation::SymbolAt { argument }
            | Operation::Memory { argument, .. }
            | Operation::ExternalCall { argument, .. }
            | Operation::SumInjection {
                value: argument, ..
            } => self.analyze_atom(argument, owned, live_after),
            Operation::Case { scrutinee, arms } => {
                let continuation = live_after.clone();
                let mut before_arms = HashSet::new();
                for arm in arms {
                    let mut arm_live = continuation.clone();
                    self.analyze_block(&arm.value, owned, &mut arm_live);
                    remove_pattern_bindings(&arm.pattern, &mut arm_live);
                    before_arms.extend(arm_live);
                }
                *live_after = before_arms;
                self.analyze_atom(scrutinee, owned, live_after);
            }
            Operation::PrimitiveBranch {
                left,
                right,
                otherwise,
                then,
                ..
            } => {
                let continuation = live_after.clone();
                let mut otherwise_live = continuation.clone();
                self.analyze_block(otherwise, owned, &mut otherwise_live);
                let mut then_live = continuation;
                self.analyze_block(then, owned, &mut then_live);
                otherwise_live.extend(then_live);
                *live_after = otherwise_live;
                self.analyze_atom(right, owned, live_after);
                self.analyze_atom(left, owned, live_after);
            }
            Operation::PrimitiveBinary { left, right, .. } => {
                self.analyze_atom(right, owned, live_after);
                self.analyze_atom(left, owned, live_after);
            }
        }
    }

    fn analyze_atom(
        &mut self,
        atom: &Atom,
        owned: &HashSet<ValueId>,
        live_after: &mut HashSet<ValueId>,
    ) {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return;
        };
        if owned.contains(&id) && !live_after.contains(&id) {
            self.last_owned_uses.insert(atom_key(atom));
        }
        live_after.insert(id);
    }
}

// The closure program is immutably borrowed for the complete emission, so an Atom's
// address is a stable occurrence identity without adding backend policy to the IR.
fn atom_key(atom: &Atom) -> *const Atom {
    std::ptr::from_ref(atom)
}

fn collect_owned_bindings(block: &Block, owned: &mut HashSet<ValueId>) {
    for binding in &block.bindings {
        collect_pattern_bindings(&binding.pattern, owned);
        match &binding.operation {
            Operation::Case { arms, .. } => {
                for arm in arms {
                    collect_pattern_bindings(&arm.pattern, owned);
                    collect_owned_bindings(&arm.value, owned);
                }
            }
            Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                collect_owned_bindings(otherwise, owned);
                collect_owned_bindings(then, owned);
            }
            Operation::Atom(_)
            | Operation::MakeClosure { .. }
            | Operation::Call { .. }
            | Operation::SymbolLength { .. }
            | Operation::SymbolAt { .. }
            | Operation::Memory { .. }
            | Operation::ExternalCall { .. }
            | Operation::NumericConversion { .. }
            | Operation::Product(_)
            | Operation::SumInjection { .. }
            | Operation::PrimitiveUnary { .. }
            | Operation::PrimitiveBinary { .. } => {}
        }
    }
}

fn collect_pattern_bindings(pattern: &Pattern, owned: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            owned.insert(*id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_bindings(element, owned);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

fn remove_pattern_bindings(pattern: &Pattern, live: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            live.remove(id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                remove_pattern_bindings(element, live);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}
