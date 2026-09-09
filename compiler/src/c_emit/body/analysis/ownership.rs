use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Block, Operation, Pattern, Reference};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct AtomOccurrenceId(usize);

pub(in crate::c_emit::body) struct OwnershipPlan {
    occurrences: HashMap<*const Atom, AtomOccurrenceId>,
    last_owned_uses: HashSet<AtomOccurrenceId>,
    last_parameter_uses: HashSet<AtomOccurrenceId>,
}

impl OwnershipPlan {
    pub(in crate::c_emit::body) fn new(program: &closure::Program) -> Self {
        let mut plan = Self {
            occurrences: HashMap::new(),
            last_owned_uses: HashSet::new(),
            last_parameter_uses: HashSet::new(),
        };
        for binding in &program.bindings {
            analyze_locals(
                &binding.value,
                &mut plan.occurrences,
                &mut plan.last_owned_uses,
            );
        }
        for function in &program.functions {
            analyze_locals(
                &function.body,
                &mut plan.occurrences,
                &mut plan.last_owned_uses,
            );
            if let Some(parameter) = function.parameter.binding {
                analyze_parameter(
                    &function.body,
                    parameter,
                    &mut plan.occurrences,
                    &mut plan.last_parameter_uses,
                );
            }
        }
        plan
    }

    pub(in crate::c_emit::body) fn can_transfer(&self, atom: &Atom, parameter_owned: bool) -> bool {
        let Some(occurrence) = self.occurrences.get(&std::ptr::from_ref(atom)) else {
            return false;
        };
        self.last_owned_uses.contains(occurrence)
            || (parameter_owned && self.last_parameter_uses.contains(occurrence))
    }

    pub(in crate::c_emit::body) fn is_valid(&self, program: &closure::Program) -> bool {
        let expected = Self::new(program);
        self.occurrences == expected.occurrences
            && self.last_owned_uses == expected.last_owned_uses
            && self.last_parameter_uses == expected.last_parameter_uses
    }
}

fn analyze_locals(
    block: &Block,
    occurrences: &mut HashMap<*const Atom, AtomOccurrenceId>,
    transfers: &mut HashSet<AtomOccurrenceId>,
) {
    let mut owned = HashSet::new();
    collect_owned_bindings(block, &mut owned);
    analyze_block(block, &owned, &mut HashSet::new(), occurrences, transfers);
}

fn analyze_parameter(
    block: &Block,
    parameter: ValueId,
    occurrences: &mut HashMap<*const Atom, AtomOccurrenceId>,
    transfers: &mut HashSet<AtomOccurrenceId>,
) {
    analyze_block(
        block,
        &HashSet::from([parameter]),
        &mut HashSet::new(),
        occurrences,
        transfers,
    );
}

fn analyze_block(
    block: &Block,
    owned: &HashSet<ValueId>,
    live_after: &mut HashSet<ValueId>,
    occurrences: &mut HashMap<*const Atom, AtomOccurrenceId>,
    transfers: &mut HashSet<AtomOccurrenceId>,
) {
    analyze_atom(&block.result, owned, live_after, occurrences, transfers);
    for binding in block.bindings.iter().rev() {
        remove_pattern_bindings(&binding.pattern, live_after);
        analyze_operation(
            &binding.operation,
            owned,
            live_after,
            occurrences,
            transfers,
        );
    }
}

fn analyze_operation(
    operation: &Operation,
    owned: &HashSet<ValueId>,
    live_after: &mut HashSet<ValueId>,
    occurrences: &mut HashMap<*const Atom, AtomOccurrenceId>,
    transfers: &mut HashSet<AtomOccurrenceId>,
) {
    match operation {
        Operation::Atom(atom)
        | Operation::SymbolLength { value: atom }
        | Operation::NumericConversion { operand: atom }
        | Operation::PrimitiveUnary { operand: atom, .. } => {
            analyze_atom(atom, owned, live_after, occurrences, transfers);
        }
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for atom in captures.iter().rev() {
                analyze_atom(atom, owned, live_after, occurrences, transfers);
            }
        }
        Operation::Call { callee, argument } => {
            analyze_atom(argument, owned, live_after, occurrences, transfers);
            analyze_atom(callee, owned, live_after, occurrences, transfers);
        }
        Operation::SymbolAt { argument }
        | Operation::Memory { argument, .. }
        | Operation::ExternalCall { argument, .. }
        | Operation::SumInjection {
            value: argument, ..
        } => analyze_atom(argument, owned, live_after, occurrences, transfers),
        Operation::Case { scrutinee, arms } => {
            let continuation = live_after.clone();
            let mut before_arms = HashSet::new();
            for arm in arms {
                let mut arm_live = continuation.clone();
                analyze_block(&arm.value, owned, &mut arm_live, occurrences, transfers);
                remove_pattern_bindings(&arm.pattern, &mut arm_live);
                before_arms.extend(arm_live);
            }
            *live_after = before_arms;
            analyze_atom(scrutinee, owned, live_after, occurrences, transfers);
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
            analyze_block(
                otherwise,
                owned,
                &mut otherwise_live,
                occurrences,
                transfers,
            );
            let mut then_live = continuation;
            analyze_block(then, owned, &mut then_live, occurrences, transfers);
            otherwise_live.extend(then_live);
            *live_after = otherwise_live;
            analyze_atom(right, owned, live_after, occurrences, transfers);
            analyze_atom(left, owned, live_after, occurrences, transfers);
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            analyze_atom(right, owned, live_after, occurrences, transfers);
            analyze_atom(left, owned, live_after, occurrences, transfers);
        }
    }
}

fn analyze_atom(
    atom: &Atom,
    owned: &HashSet<ValueId>,
    live_after: &mut HashSet<ValueId>,
    occurrences: &mut HashMap<*const Atom, AtomOccurrenceId>,
    transfers: &mut HashSet<AtomOccurrenceId>,
) {
    let next = AtomOccurrenceId(occurrences.len());
    let occurrence = *occurrences.entry(std::ptr::from_ref(atom)).or_insert(next);
    let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
        return;
    };
    if owned.contains(&id) && !live_after.contains(&id) {
        transfers.insert(occurrence);
    }
    live_after.insert(id);
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
