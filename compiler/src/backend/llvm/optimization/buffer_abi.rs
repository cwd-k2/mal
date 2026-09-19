use std::collections::{HashMap, HashSet};

use crate::check::ast::Type;
use crate::closure::ast::FunctionId;
use crate::control::ast::{Operation, StateId, Terminator};
use crate::core::ast::PackedBuilderOperation;

pub(super) fn plan(execution: &crate::execution::Program) -> HashSet<FunctionId> {
    let functions = execution
        .control
        .functions
        .iter()
        .map(|function| (function.id, function))
        .collect::<HashMap<_, _>>();
    let mut stable = functions
        .keys()
        .copied()
        .filter(|function| has_no_relocation(execution, functions[function].entry))
        .collect::<HashSet<_>>();
    close_stability_over_calls(execution, &mut stable);

    let mut direct = execution
        .lowered
        .functions
        .iter()
        .filter(|function| {
            stable.contains(&function.id)
                && buffer_parameter_is_supported(&function.parameter.ty)
                && !contains_buffer(&function.body.result.ty)
                && !captures_buffer_in_nested_closure(execution, functions[&function.id].entry)
                && function.kind.captures().is_some_and(|captures| {
                    captures.iter().all(|capture| !contains_buffer(&capture.ty))
                })
        })
        .map(|function| function.id)
        .collect::<HashSet<_>>();

    loop {
        let before = direct.len();
        close_regions(execution, &mut direct);
        close_buffer_calls(execution, &mut direct);
        if direct.len() == before {
            return direct;
        }
    }
}

fn close_stability_over_calls(
    execution: &crate::execution::Program,
    stable: &mut HashSet<FunctionId>,
) {
    loop {
        let unstable = stable
            .iter()
            .copied()
            .filter(|function| {
                execution
                    .applications
                    .sites_from(*function)
                    .any(|(_, targets)| {
                        targets.is_empty() || targets.iter().any(|target| !stable.contains(target))
                    })
            })
            .collect::<Vec<_>>();
        if unstable.is_empty() {
            return;
        }
        for function in unstable {
            stable.remove(&function);
        }
    }
}

fn close_regions(execution: &crate::execution::Program, direct: &mut HashSet<FunctionId>) {
    for region in execution.control_regions.ids() {
        let functions = execution.control_regions.functions(region);
        if functions.iter().any(|function| !direct.contains(function)) {
            for function in functions {
                direct.remove(function);
            }
        }
    }
}

fn close_buffer_calls(execution: &crate::execution::Program, direct: &mut HashSet<FunctionId>) {
    let mut remove = HashSet::new();
    for (site, caller) in execution.applications.sites() {
        let Some(argument) = call_argument(&execution.control.states[site.0].terminator) else {
            continue;
        };
        if !contains_buffer(&argument.ty) {
            continue;
        }
        let Some(targets) = execution.applications.targets(site) else {
            continue;
        };
        if execution.applications.direct_target(site).is_none()
            && targets.iter().any(|target| !direct.contains(target))
        {
            remove.extend(
                targets
                    .iter()
                    .filter(|target| direct.contains(target))
                    .copied(),
            );
        }
        if caller.is_some_and(|caller| direct.contains(&caller))
            && targets.iter().any(|target| !direct.contains(target))
        {
            remove.extend(caller);
        }
    }
    direct.retain(|function| !remove.contains(function));
}

fn has_no_relocation(execution: &crate::execution::Program, entry: StateId) -> bool {
    reachable_states(&execution.control, entry).all(|state| {
        execution.control.states[state.0]
            .bindings
            .iter()
            .all(|binding| match &binding.operation {
                Operation::PackedBuilder { operation, .. } => matches!(
                    operation,
                    PackedBuilderOperation::Get | PackedBuilderOperation::Put
                ),
                _ => true,
            })
    })
}

fn captures_buffer_in_nested_closure(
    execution: &crate::execution::Program,
    entry: StateId,
) -> bool {
    reachable_states(&execution.control, entry).any(|state| {
        execution.control.states[state.0]
            .bindings
            .iter()
            .any(|binding| {
                matches!(
                    &binding.operation,
                    Operation::MakeClosure { captures, .. }
                        if captures.iter().any(|capture| contains_buffer(&capture.ty))
                )
            })
    })
}

fn buffer_parameter_is_supported(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut found = false;
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Buffer(_) => found = true,
            Type::Product(elements) => pending.extend(elements.iter()),
            _ if contains_buffer(ty) => return false,
            _ => {}
        }
    }
    found
}

pub(super) fn contains_buffer(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut seen = HashSet::new();
    while let Some(ty) = pending.pop() {
        if let Some(identity) = ty.shared_id()
            && !seen.insert(identity)
        {
            continue;
        }
        match ty {
            Type::Buffer(_) => return true,
            Type::Cursor(element) | Type::Region(element) | Type::Packed(element) => {
                pending.push(element)
            }
            Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
            Type::Function { parameter, result } => {
                pending.extend([parameter.as_ref(), result.as_ref()]);
            }
            _ => {}
        }
    }
    false
}

fn call_argument(terminator: &Terminator) -> Option<&crate::closure::ast::Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
}

fn reachable_states(
    control: &crate::control::ast::Program,
    entry: StateId,
) -> impl Iterator<Item = StateId> + '_ {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    std::iter::from_fn(move || {
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            match &control.states[id.0].terminator {
                Terminator::Return(_) | Terminator::TailCall { .. } => {}
                Terminator::Goto(target) | Terminator::Jump { target, .. } => {
                    pending.push(*target);
                }
                Terminator::Call { resume, .. } => pending.push(*resume),
                Terminator::Case { arms, .. } => {
                    pending.extend(arms.iter().map(|arm| arm.target));
                }
                Terminator::PrimitiveBranch {
                    otherwise, then, ..
                } => pending.extend([*otherwise, *then]),
            }
            return Some(id);
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    #[test]
    fn admits_only_closed_direct_buffer_abis() {
        let source = SourceFile::new(
            FileId::new(92),
            "direct-buffer-abi.mal",
            "read :: Buffer<Int64> -> Int64 := (buffer) -> { buffer.get(0usize); };\n\
             grow :: Buffer<Int64> -> Int64 := (buffer) -> { _ := buffer.new(2i64); buffer.get(0usize); };\n\
             forwardGrow :: Buffer<Int64> -> Int64 := (buffer) -> { grow(buffer); };\n\
             identity :: Buffer<Int64> -> Buffer<Int64> := (buffer) -> { buffer; };\n\
             capture :: Buffer<Int64> -> Int64 := (buffer) -> { nested :: (Unit -> Int64) := () -> { buffer.get(0usize); }; nested(); };\n\
             mixedRead :: Buffer<Int32> -> Int32 := (buffer) -> { buffer.get(0usize); };\n\
             mixedGrow :: Buffer<Int32> -> Int32 := (buffer) -> { _ := buffer.new(2i32); buffer.get(0usize); };\n\
             applyMixed :: ((Buffer<Int32> -> Int32), Buffer<Int32>) -> Int32 := (function, buffer) -> { function(buffer); };\n\
             main :: Unit -> Int32 := () -> { values := pack<Int64>((buffer) -> { _ := buffer.new(1i64); _ := read(buffer); _ := forwardGrow(buffer); _ := capture(buffer); returned := identity(buffer); captured :: (Unit -> Int64) := () -> { returned.get(0usize); }; _ := captured(); (); }); other := pack<Int32>((buffer) -> { _ := buffer.new(1i32); _ := applyMixed((mixedRead, buffer)); _ := applyMixed((mixedGrow, buffer)); (); }); (values # 0usize).i32 + (other # 0usize) - 2i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check direct Buffer ABI fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize direct Buffer ABI fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let direct = plan(&execution);

        let get_only = execution
            .control
            .functions
            .iter()
            .filter(|function| {
                has_no_relocation(&execution, function.entry)
                    && reachable_states(&execution.control, function.entry).any(|state| {
                        execution.control.states[state.0]
                            .bindings
                            .iter()
                            .any(|binding| {
                                matches!(
                                    &binding.operation,
                                    Operation::PackedBuilder {
                                        operation: PackedBuilderOperation::Get,
                                        ..
                                    }
                                )
                            })
                    })
            })
            .map(|function| function.id)
            .collect::<Vec<_>>();
        assert!(!direct.is_empty());
        assert!(direct.iter().all(|function| get_only.contains(function)));
        assert!(get_only.iter().any(|function| direct.contains(function)));
        assert!(get_only.iter().any(|function| !direct.contains(function)));

        for function in &execution.lowered.functions {
            if contains_buffer(&function.body.result.ty)
                || function.kind.captures().is_some_and(|captures| {
                    captures.iter().any(|capture| contains_buffer(&capture.ty))
                })
            {
                assert!(!direct.contains(&function.id));
            }
        }
        for function in &execution.control.functions {
            if captures_buffer_in_nested_closure(&execution, function.entry) {
                assert!(!direct.contains(&function.id));
            }
        }

        let mixed_site = execution
            .applications
            .sites()
            .find(|(site, _)| {
                execution.applications.direct_target(*site).is_none()
                    && execution
                        .applications
                        .targets(*site)
                        .is_some_and(|targets| targets.len() > 1)
                    && call_argument(&execution.control.states[site.0].terminator)
                        .is_some_and(|argument| contains_buffer(&argument.ty))
            })
            .expect("mixed indirect Buffer site");
        assert!(
            execution
                .applications
                .targets(mixed_site.0)
                .expect("mixed targets")
                .iter()
                .all(|target| !direct.contains(target))
        );
    }
}
