//! Lambda lifting for closures passed through a known function parameter.

use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomId, FunctionId, Operation, Program};

use analysis::*;

mod analysis;
mod nested;
mod rewrite;

#[derive(Clone)]
struct Definition {
    operation: Operation,
}

struct Candidate {
    host: FunctionId,
    host_binding: ValueId,
    parameter: ValueId,
    callback: ValueId,
    target: FunctionId,
    replacements: HashMap<AtomId, Vec<Atom>>,
    forwarded: HashSet<AtomId>,
    direct_calls: HashSet<AtomId>,
    nested: Option<nested::Use>,
    capture_types: Vec<Type>,
    capture_type: Type,
    host_parameter_type: Type,
    host_closure_type: Type,
}

pub(super) fn closure_parameters(program: &mut Program) -> bool {
    let Some(candidate) = find_candidate(program) else {
        return false;
    };
    rewrite::apply(program, candidate);
    true
}

fn find_candidate(program: &Program) -> Option<Candidate> {
    let known = program
        .bindings
        .iter()
        .filter_map(|binding| {
            let (name, function) = binding.known_function()?;
            Some((function, name))
        })
        .collect::<HashMap<_, _>>();
    let index = Index::new(program);
    let mut direct = None;

    for host in &program.functions {
        let Some(host_binding) = known.get(&host.id).copied() else {
            continue;
        };
        let Some(parameter) = host.parameter.binding else {
            continue;
        };
        for (callback, path) in parameter_callbacks(host, parameter) {
            if let Some(candidate) = admit_candidate(
                program,
                &index,
                host.id,
                host_binding,
                parameter,
                callback,
                path,
            ) {
                if candidate.nested.is_some() {
                    // Lifting the outer callback first would erase the capture edge this proof needs.
                    return Some(candidate);
                }
                direct.get_or_insert(candidate);
            }
        }
    }
    direct
}

/// Proves that replacing one callback parameter leaf with its captures is semantics-preserving.
///
/// Every call of the host must supply either the parameter itself or a closure of one target and
/// one capture shape. The callback and all such creators must have no uses outside the traced
/// forwarding and direct-call sites, and self closures are excluded because rewriting them would
/// change how their environment is obtained.
fn admit_candidate(
    program: &Program,
    index: &Index<'_>,
    host: FunctionId,
    host_binding: ValueId,
    parameter: ValueId,
    callback: ValueId,
    path: Vec<usize>,
) -> Option<Candidate> {
    let host_function = program
        .functions
        .iter()
        .find(|function| function.id == host)?;
    let Type::Function { .. } = index.binding_type(callback)? else {
        return None;
    };

    let mut creators = HashSet::new();
    let mut replacements = HashMap::new();
    let mut forwarded = HashSet::new();
    let mut allowed_callback_uses = HashSet::new();
    let mut target = None;
    let mut call_count = 0;
    for argument in index.host_call_arguments(host_binding, host) {
        call_count += 1;
        let (leaf, trace) = resolve_path(argument, &path, &index.definitions)?;
        let leaf_origin = origin(&leaf, &index.aliases)?;
        if leaf_origin == callback {
            if leaf.binding() != Some(callback) {
                return None;
            }
            forwarded.insert(leaf.id);
            allowed_callback_uses.extend(trace);
            continue;
        }
        let Some(Definition {
            operation: Operation::MakeClosure { function, captures },
        }) = index.definitions.get(&leaf_origin)
        else {
            return None;
        };
        if captures.is_empty() || target.is_some_and(|value| value != *function) {
            return None;
        }
        target = Some(*function);
        creators.insert(leaf_origin);
        replacements.insert(leaf.id, captures.clone());
    }
    let target = target?;
    if call_count == 0 || creators.is_empty() {
        return None;
    }

    let target_function = program
        .functions
        .iter()
        .find(|function| function.id == target)?;
    if target_function.captures.is_empty() || index.refers_to_self_closure(target) {
        return None;
    }
    if target_function.captures.len() != replacements.values().next().map_or(0, Vec::len) {
        return None;
    }
    let capture_types = target_function
        .captures
        .iter()
        .map(|capture| capture.ty.clone())
        .collect::<Vec<_>>();
    let capture_type = Type::Product(capture_types.clone().into());
    if replacements.values().any(|captures| {
        Type::Product(
            captures
                .iter()
                .map(|capture| capture.ty.clone())
                .collect::<Vec<_>>()
                .into(),
        ) != capture_type
    }) {
        return None;
    }

    let mut direct_calls = HashSet::new();
    for callee in index.callees_from(callback) {
        if callee.binding() != Some(callback) {
            return None;
        }
        direct_calls.insert(callee.id);
        allowed_callback_uses.insert(callee.id);
    }
    let nested = nested::find(program, index, callback);
    if let Some(nested) = &nested {
        allowed_callback_uses.extend(nested.capture_atoms.iter().copied());
    }
    let allowed_creator_uses = replacements.keys().copied().collect::<HashSet<_>>();
    if (direct_calls.is_empty() && nested.is_none())
        || index.has_unapproved_uses(callback, &allowed_callback_uses)
        || creators
            .iter()
            .any(|creator| index.has_unapproved_uses(*creator, &allowed_creator_uses))
        || index.capturing_creators(target) != creators
    {
        return None;
    }

    let host_parameter_type = replace_type(&host_function.parameter.ty, &path, &capture_type)?;
    let host_closure_type = Type::Function {
        parameter: host_parameter_type.clone().into(),
        result: host_function.body.result.ty.clone().into(),
    };
    Some(Candidate {
        host,
        host_binding,
        parameter,
        callback,
        target,
        replacements,
        forwarded,
        direct_calls,
        nested,
        capture_types,
        capture_type,
        host_parameter_type,
        host_closure_type,
    })
}
