//! Where each local closure is created and how its aliases are used.

use super::*;

#[derive(Clone)]
pub(super) struct Creator {
    pub(super) function: FunctionId,
    pub(super) captures: Vec<Atom>,
}

#[derive(Default)]
pub(super) struct Uses {
    pub(super) direct: bool,
    pub(super) other: bool,
}

pub(super) fn collect_definitions(
    block: &Block,
    origins: &mut HashMap<ValueId, ValueId>,
    creators: &mut HashMap<ValueId, Creator>,
    disqualified: &mut HashSet<FunctionId>,
) {
    for binding in &block.bindings {
        match (&binding.pattern, &binding.operation) {
            (Pattern::Binding { id, .. }, Operation::MakeClosure { function, captures }) => {
                origins.insert(*id, *id);
                creators.insert(
                    *id,
                    Creator {
                        function: *function,
                        captures: captures.clone(),
                    },
                );
            }
            (
                Pattern::Binding { id, .. },
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(source)),
                    ..
                }),
            ) if origins.contains_key(source) => {
                origins.insert(*id, origins[source]);
            }
            (_, Operation::MakeClosure { function, captures }) if !captures.is_empty() => {
                disqualified.insert(*function);
            }
            _ => {}
        }
        binding.operation.for_each_nested_block(|nested| {
            collect_definitions(nested, origins, creators, disqualified)
        });
    }
}

pub(super) fn collect_uses(
    block: &Block,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    atom_use(&block.result, false, origins, uses, disqualified);
    for binding in &block.bindings {
        let alias = matches!(
            (&binding.pattern, &binding.operation),
            (
                Pattern::Binding { id, .. },
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(source)),
                    ..
                })
            ) if origins.get(id) == origins.get(source) && origins.contains_key(id)
        );
        if !alias {
            operation_uses(&binding.operation, origins, uses, disqualified);
        }
    }
}

pub(super) fn operation_uses(
    operation: &Operation,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    match operation {
        Operation::Call { callee, argument } => {
            atom_use(callee, true, origins, uses, disqualified);
            atom_use(argument, false, origins, uses, disqualified);
        }
        Operation::Case { scrutinee, arms } => {
            atom_use(scrutinee, false, origins, uses, disqualified);
            for arm in arms {
                collect_uses(&arm.value, origins, uses, disqualified);
            }
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            atom_use(left, false, origins, uses, disqualified);
            atom_use(right, false, origins, uses, disqualified);
            collect_uses(otherwise, origins, uses, disqualified);
            collect_uses(then, origins, uses, disqualified);
        }
        other => other.for_each_atom(|atom| atom_use(atom, false, origins, uses, disqualified)),
    }
}

pub(super) fn atom_use(
    atom: &Atom,
    direct: bool,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(binding)) => {
            if let Some(use_) = origins
                .get(&binding)
                .and_then(|creator| uses.get_mut(creator))
            {
                if direct {
                    use_.direct = true;
                } else {
                    use_.other = true;
                }
            }
        }
        AtomKind::Reference(Reference::SelfClosure(function)) => {
            disqualified.insert(function);
        }
        _ => {}
    }
}
