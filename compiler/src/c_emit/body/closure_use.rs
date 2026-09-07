use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Block, FunctionId, Operation, Pattern};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct DirectClosure {
    pub(super) creator: ValueId,
    pub(super) function: FunctionId,
}

pub(super) struct ClosureUsePlan {
    direct: HashMap<ValueId, DirectClosure>,
}

impl ClosureUsePlan {
    pub(super) fn new(program: &closure::Program) -> Self {
        let mut candidates = HashMap::new();
        for binding in &program.bindings {
            collect_candidates(&binding.value, &mut candidates);
        }
        for function in &program.functions {
            collect_candidates(&function.body, &mut candidates);
        }

        let mut direct_uses = HashSet::new();
        let mut other_uses = HashSet::new();
        for binding in &program.bindings {
            collect_uses(
                &binding.value,
                &candidates,
                &mut direct_uses,
                &mut other_uses,
            );
        }
        for function in &program.functions {
            collect_uses(
                &function.body,
                &candidates,
                &mut direct_uses,
                &mut other_uses,
            );
        }
        candidates.retain(|_, target| {
            direct_uses.contains(&target.creator) && !other_uses.contains(&target.creator)
        });
        Self { direct: candidates }
    }

    pub(super) fn direct_closure(&self, id: ValueId) -> Option<DirectClosure> {
        self.direct.get(&id).copied()
    }

    pub(super) fn is_direct_alias(&self, id: ValueId, source: ValueId) -> bool {
        self.direct.contains_key(&id) && self.direct.get(&id) == self.direct.get(&source)
    }

    pub(super) fn has_direct_creator(&self, function: FunctionId) -> bool {
        self.direct
            .values()
            .any(|candidate| candidate.function == function)
    }
}

fn collect_candidates(block: &Block, candidates: &mut HashMap<ValueId, DirectClosure>) {
    for binding in &block.bindings {
        if let Pattern::Binding { id, .. } = binding.pattern {
            match &binding.operation {
                Operation::MakeClosure { function, .. } => {
                    candidates.insert(
                        id,
                        DirectClosure {
                            creator: id,
                            function: *function,
                        },
                    );
                }
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(closure::Reference::Binding(source)),
                    ..
                }) => {
                    if let Some(target) = candidates.get(source).copied() {
                        candidates.insert(id, target);
                    }
                }
                _ => {}
            }
        }
        collect_nested_candidates(&binding.operation, candidates);
    }
}

fn collect_nested_candidates(
    operation: &Operation,
    candidates: &mut HashMap<ValueId, DirectClosure>,
) {
    match operation {
        Operation::Case { arms, .. } => {
            for arm in arms {
                collect_candidates(&arm.value, candidates);
            }
        }
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            collect_candidates(otherwise, candidates);
            collect_candidates(then, candidates);
        }
        _ => {}
    }
}

fn collect_uses(
    block: &Block,
    candidates: &HashMap<ValueId, DirectClosure>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    collect_atom_use(&block.result, false, candidates, direct_uses, other_uses);
    for binding in &block.bindings {
        if let (
            Pattern::Binding { id, .. },
            Operation::Atom(Atom {
                kind: AtomKind::Reference(closure::Reference::Binding(source)),
                ..
            }),
        ) = (&binding.pattern, &binding.operation)
            && candidates.contains_key(id)
            && candidates.get(id) == candidates.get(source)
        {
            continue;
        }
        collect_operation_uses(&binding.operation, candidates, direct_uses, other_uses);
    }
}

fn collect_operation_uses(
    operation: &Operation,
    candidates: &HashMap<ValueId, DirectClosure>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    let mut atom = |atom: &Atom, direct| {
        collect_atom_use(atom, direct, candidates, direct_uses, other_uses);
    };
    match operation {
        Operation::Atom(value)
        | Operation::SymbolLength { value }
        | Operation::NumericConversion { operand: value }
        | Operation::PrimitiveUnary { operand: value, .. } => atom(value, false),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for capture in captures {
                atom(capture, false);
            }
        }
        Operation::Call { callee, argument } => {
            atom(callee, true);
            atom(argument, false);
        }
        Operation::SymbolAt { argument }
        | Operation::Memory { argument, .. }
        | Operation::ExternalCall { argument, .. }
        | Operation::SumInjection {
            value: argument, ..
        } => atom(argument, false),
        Operation::Case { scrutinee, arms } => {
            atom(scrutinee, false);
            for arm in arms {
                collect_uses(&arm.value, candidates, direct_uses, other_uses);
            }
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            atom(left, false);
            atom(right, false);
            collect_uses(otherwise, candidates, direct_uses, other_uses);
            collect_uses(then, candidates, direct_uses, other_uses);
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            atom(left, false);
            atom(right, false);
        }
    }
}

fn collect_atom_use(
    atom: &Atom,
    direct_callee: bool,
    candidates: &HashMap<ValueId, DirectClosure>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    let AtomKind::Reference(closure::Reference::Binding(id)) = atom.kind else {
        return;
    };
    let Some(target) = candidates.get(&id) else {
        return;
    };
    if direct_callee {
        direct_uses.insert(target.creator);
    } else {
        other_uses.insert(target.creator);
    }
}
