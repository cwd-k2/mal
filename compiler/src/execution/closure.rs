use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Block, FunctionId, Operation, Pattern};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct DirectClosure {
    pub(crate) creator: ValueId,
    pub(crate) function: FunctionId,
}

pub(crate) struct ClosureUsePlan {
    direct: HashMap<ValueId, DirectClosure>,
    top_levels: HashSet<ValueId>,
    known_top_levels: HashMap<ValueId, DirectClosure>,
}

impl ClosureUsePlan {
    pub(crate) fn new(program: &closure::Program) -> Self {
        let mut candidates = HashMap::new();
        let mut top_levels = HashSet::new();
        for binding in &program.bindings {
            if let Some(candidate) = top_level_candidate(binding) {
                top_levels.insert(candidate.creator);
                candidates.insert(candidate.creator, candidate);
            }
        }
        let known_top_levels = top_levels
            .iter()
            .filter_map(|id| candidates.get(id).map(|candidate| (*id, *candidate)))
            .collect();
        for binding in &program.bindings {
            collect_candidates(&binding.value, &mut candidates);
        }
        for function in &program.functions {
            collect_candidates(&function.body, &mut candidates);
            for join in &function.joins {
                collect_candidates(&join.body, &mut candidates);
            }
        }
        // Closure conversion rewrites recursive references away from their source
        // binding, so retain the creator identity needed for escape classification.
        let creators = candidates
            .iter()
            .filter_map(|(id, target)| (*id == target.creator).then_some((target.function, *id)))
            .chain(top_levels.iter().filter_map(|id| {
                candidates
                    .get(id)
                    .map(|candidate| (candidate.function, *id))
            }))
            .collect::<HashMap<_, _>>();

        let mut direct_uses = HashSet::new();
        let mut other_uses = HashSet::new();
        for binding in &program.bindings {
            collect_uses(
                &binding.value,
                &candidates,
                &creators,
                &mut direct_uses,
                &mut other_uses,
            );
        }
        for function in &program.functions {
            collect_uses(
                &function.body,
                &candidates,
                &creators,
                &mut direct_uses,
                &mut other_uses,
            );
            for join in &function.joins {
                collect_uses(
                    &join.body,
                    &candidates,
                    &creators,
                    &mut direct_uses,
                    &mut other_uses,
                );
            }
        }
        candidates.retain(|_, target| {
            direct_uses.contains(&target.creator) && !other_uses.contains(&target.creator)
        });
        Self {
            direct: candidates,
            top_levels,
            known_top_levels,
        }
    }

    pub(crate) fn direct_closure(&self, id: ValueId) -> Option<DirectClosure> {
        self.direct
            .get(&id)
            .or_else(|| self.known_top_levels.get(&id))
            .copied()
    }

    pub(crate) fn is_valid(&self, program: &closure::Program) -> bool {
        let expected = Self::new(program);
        self.direct == expected.direct
            && self.top_levels == expected.top_levels
            && self.known_top_levels == expected.known_top_levels
    }
}

fn top_level_candidate(binding: &closure::TopLevelBinding) -> Option<DirectClosure> {
    let closure::TopLevelPattern::Binding { id: creator, .. } = binding.pattern else {
        return None;
    };
    let AtomKind::Reference(closure::Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(DirectClosure {
                    creator,
                    function: *function,
                })
            }
            _ => None,
        }
    })
}

fn collect_candidates(block: &Block, candidates: &mut HashMap<ValueId, DirectClosure>) {
    for binding in &block.bindings {
        if let Pattern::Binding { id, .. } = binding.pattern {
            match &binding.operation {
                Operation::MakeClosure { function, .. }
                | Operation::MakePackedCapability { function, .. } => {
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
    creators: &HashMap<FunctionId, ValueId>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    collect_atom_use(
        &block.result,
        false,
        candidates,
        creators,
        direct_uses,
        other_uses,
    );
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
        collect_operation_uses(
            &binding.operation,
            candidates,
            creators,
            direct_uses,
            other_uses,
        );
    }
}

fn collect_operation_uses(
    operation: &Operation,
    candidates: &HashMap<ValueId, DirectClosure>,
    creators: &HashMap<FunctionId, ValueId>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    let mut atom = |atom: &Atom, direct| {
        collect_atom_use(atom, direct, candidates, creators, direct_uses, other_uses);
    };
    match operation {
        Operation::Atom(value)
        | Operation::Goto { value, .. }
        | Operation::SymbolLength { value }
        | Operation::NumericConversion { operand: value }
        | Operation::PrimitiveUnary { operand: value, .. } => atom(value, false),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for capture in captures {
                atom(capture, false);
            }
        }
        Operation::MakePackedCapability { builder, .. } => atom(builder, false),
        Operation::Call { callee, argument } => {
            atom(callee, true);
            atom(argument, false);
        }
        Operation::SymbolAt { argument }
        | Operation::Memory { argument, .. }
        | Operation::PackedBuilder { argument, .. }
        | Operation::ExternalCall { argument, .. }
        | Operation::SumInjection {
            value: argument, ..
        } => atom(argument, false),
        Operation::Case { scrutinee, arms } => {
            atom(scrutinee, false);
            for arm in arms {
                collect_uses(&arm.value, candidates, creators, direct_uses, other_uses);
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
            collect_uses(otherwise, candidates, creators, direct_uses, other_uses);
            collect_uses(then, candidates, creators, direct_uses, other_uses);
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
    creators: &HashMap<FunctionId, ValueId>,
    direct_uses: &mut HashSet<ValueId>,
    other_uses: &mut HashSet<ValueId>,
) {
    let creator = match atom.kind {
        AtomKind::Reference(closure::Reference::Binding(id)) => {
            let Some(target) = candidates.get(&id) else {
                return;
            };
            target.creator
        }
        AtomKind::Reference(closure::Reference::SelfClosure(function)) => {
            let Some(creator) = creators.get(&function) else {
                return;
            };
            *creator
        }
        _ => return,
    };
    if direct_callee {
        direct_uses.insert(creator);
    } else {
        other_uses.insert(creator);
    }
}
