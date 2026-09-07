use std::collections::{HashMap, HashSet};

use crate::c_emit::types::TypeRegistry;
use crate::closure::ast::{self as closure, Block, FunctionId, Operation, Pattern};

use super::{ClosureUsePlan, OwnershipPlan, direct_function_id, has_direct_tail_call};

pub(super) struct OwnedCallPlan {
    functions: HashSet<FunctionId>,
}

impl OwnedCallPlan {
    pub(super) fn new(
        program: &crate::closure::ast::Program,
        types: &TypeRegistry,
        ownership: &OwnershipPlan,
        closure_uses: &ClosureUsePlan,
    ) -> Self {
        let mut demand = Demand::default();
        for binding in &program.bindings {
            collect_owned_callees(
                &binding.value,
                None,
                false,
                program,
                types,
                ownership,
                closure_uses,
                &mut demand,
            );
        }
        for function in &program.functions {
            collect_owned_callees(
                &function.body,
                Some(function.id),
                has_direct_tail_call(&function.body, function.id),
                program,
                types,
                ownership,
                closure_uses,
                &mut demand,
            );
        }

        // An owned parameter can become the owned argument of another direct
        // call. Each newly demanded body is rescanned once to close that graph.
        let functions_by_id: HashMap<_, _> = program
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect();
        let mut next = 0;
        while let Some(id) = demand.pending.get(next).copied() {
            next += 1;
            let function = functions_by_id
                .get(&id)
                .copied()
                .expect("direct calls preserve function identities");
            collect_owned_callees(
                &function.body,
                Some(function.id),
                true,
                program,
                types,
                ownership,
                closure_uses,
                &mut demand,
            );
        }
        Self {
            functions: demand.functions,
        }
    }

    pub(super) fn contains(&self, function: FunctionId) -> bool {
        self.functions.contains(&function)
    }
}

#[derive(Default)]
struct Demand {
    functions: HashSet<FunctionId>,
    pending: Vec<FunctionId>,
}

impl Demand {
    fn insert(&mut self, function: FunctionId) {
        if self.functions.insert(function) {
            self.pending.push(function);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_owned_callees(
    block: &Block,
    current: Option<FunctionId>,
    parameter_owned: bool,
    program: &crate::closure::ast::Program,
    types: &TypeRegistry,
    ownership: &OwnershipPlan,
    closure_uses: &ClosureUsePlan,
    demand: &mut Demand,
) {
    for (index, binding) in block.bindings.iter().enumerate() {
        match &binding.operation {
            Operation::Call { callee, argument }
                if types.contains_managed(&argument.ty)
                    && ownership.can_transfer(argument, parameter_owned) =>
            {
                if let Some(target) = direct_function_id(program, closure_uses, callee)
                    && !is_direct_self_tail_call(block, index, current)
                {
                    demand.insert(target);
                }
            }
            Operation::Case { arms, .. } => {
                for arm in arms {
                    collect_owned_callees(
                        &arm.value,
                        current,
                        parameter_owned,
                        program,
                        types,
                        ownership,
                        closure_uses,
                        demand,
                    );
                }
            }
            Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                collect_owned_callees(
                    otherwise,
                    current,
                    parameter_owned,
                    program,
                    types,
                    ownership,
                    closure_uses,
                    demand,
                );
                collect_owned_callees(
                    then,
                    current,
                    parameter_owned,
                    program,
                    types,
                    ownership,
                    closure_uses,
                    demand,
                );
            }
            _ => {}
        }
    }
}

fn is_direct_self_tail_call(
    block: &Block,
    binding_index: usize,
    current: Option<FunctionId>,
) -> bool {
    if binding_index + 1 != block.bindings.len() {
        return false;
    }
    let binding = &block.bindings[binding_index];
    matches!(
        (&block.result.kind, &binding.pattern, &binding.operation),
        (
            closure::AtomKind::Reference(closure::Reference::Binding(result)),
            Pattern::Binding { id, .. },
            Operation::Call { callee, .. },
        ) if result == id
            && matches!(
                callee.kind,
                closure::AtomKind::Reference(closure::Reference::SelfClosure(target))
                    if Some(target) == current
            )
    )
}
