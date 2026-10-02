//! Functions that also get a native version for their recursion.
//!
//! A region function that recurses only into itself can run its recursion on the native stack while the stack has
//! room: the native version nests a native call for each recursive call and hands the activation to the frames
//! version, which keeps suspended callers in the control arena, once the native budget is used up. The frames
//! version always exists, so a region this plan does not admit simply keeps running on frames.
//! When every managed parameter leaf passes through every self edge, the synchronous caller remains a lender for
//! the entire recursive invocation and the ownership plan can borrow those leaves at every activation.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::closure::ast::Pattern;
use crate::control::ast::{Program, Terminator};

use super::optimization::{OptimizationSet, Technique};
use super::{ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan};

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct NativeRecursionPlan {
    functions: HashSet<FunctionId>,
    borrowed_parameters: HashSet<FunctionId>,
    persistent_lenders: HashSet<ValueId>,
    scalar_parameters: Vec<NativeScalarParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeScalarParameter {
    pub(crate) function: FunctionId,
    pub(crate) varying: Vec<NativeParameterLeaf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeParameterLeaf {
    pub(crate) id: ValueId,
    pub(crate) ty: mal_frontend::check::ast::Type,
    pub(crate) path: Vec<usize>,
}

impl NativeRecursionPlan {
    pub(crate) fn new(
        control: &Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        frames: &ControlFramePlan,
        enabled: OptimizationSet,
    ) -> Self {
        if !enabled.contains(Technique::NativeRecursion) {
            return Self::default();
        }
        let functions = control
            .functions
            .iter()
            .map(|function| function.id)
            .filter(|function| {
                let Some(region) = regions.function_region(*function) else {
                    return false;
                };
                if regions.functions(region) != [*function] || calls.requires_common_control(region)
                {
                    return false;
                }
                let sites = regions
                    .sites(region)
                    .iter()
                    .copied()
                    .filter(|site| frames.frame(*site).is_some())
                    .collect::<Vec<_>>();
                !sites.is_empty()
                    && sites.iter().all(|site| {
                        matches!(control.states[site.0].terminator, Terminator::Call { .. })
                            && calls.mode(*site) == Some(ControlCallMode::DirectRegion(*function))
                    })
            })
            .collect::<HashSet<_>>();
        let pass_through = super::pass_through::ParameterPassThrough::new(control);
        let borrowed_parameters = control
            .functions
            .iter()
            .filter(|function| functions.contains(&function.id))
            .filter_map(|function| {
                let parameter = function.parameter.binding?;
                let pattern = pass_through.parameter_pattern(control, function.entry, parameter)?;
                let managed = managed_bindings(pattern);
                if managed.is_empty() {
                    return None;
                }
                let preserved = function
                    .states
                    .iter()
                    .filter_map(|site| match calls.mode(*site) {
                        Some(ControlCallMode::DirectRegion(target)) if target == function.id => {
                            let Terminator::Call { argument, .. } =
                                &control.states[site.0].terminator
                            else {
                                return None;
                            };
                            Some(pass_through.fields(pattern, argument))
                        }
                        Some(ControlCallMode::DirectSelfTail) => {
                            let Terminator::TailCall { argument, .. } =
                                &control.states[site.0].terminator
                            else {
                                return None;
                            };
                            Some(pass_through.fields(pattern, argument))
                        }
                        _ => None,
                    })
                    .reduce(|left, right| left.intersection(&right).copied().collect())?;
                managed.is_subset(&preserved).then_some(function.id)
            })
            .collect::<HashSet<FunctionId>>();
        let persistent_lenders = control
            .functions
            .iter()
            .filter(|function| borrowed_parameters.contains(&function.id))
            .filter_map(|function| {
                let parameter = function.parameter.binding?;
                let pattern = pass_through.parameter_pattern(control, function.entry, parameter)?;
                Some(managed_bindings(pattern))
            })
            .flatten()
            .collect();
        let scalar_parameters = control
            .functions
            .iter()
            .filter(|function| functions.contains(&function.id))
            .filter_map(|function| {
                let parameter = function.parameter.binding?;
                let pattern = pass_through.parameter_pattern(control, function.entry, parameter)?;
                let mut leaves = Vec::new();
                collect_parameter_leaves(pattern, &mut Vec::new(), &mut leaves)?;
                let preserved = function
                    .states
                    .iter()
                    .filter_map(|site| match calls.mode(*site) {
                        Some(ControlCallMode::DirectRegion(target)) if target == function.id => {
                            let Terminator::Call { argument, .. } =
                                &control.states[site.0].terminator
                            else {
                                return None;
                            };
                            Some(pass_through.fields(pattern, argument))
                        }
                        Some(ControlCallMode::DirectSelfTail) => {
                            let Terminator::TailCall { argument, .. } =
                                &control.states[site.0].terminator
                            else {
                                return None;
                            };
                            Some(pass_through.fields(pattern, argument))
                        }
                        _ => None,
                    })
                    .reduce(|left, right| left.intersection(&right).copied().collect())?;
                let varying = leaves
                    .iter()
                    .filter(|leaf| !preserved.contains(&leaf.id))
                    .cloned()
                    .collect::<Vec<_>>();
                let managed_are_invariant = varying
                    .iter()
                    .all(|leaf| !super::ownership::is_managed(&leaf.ty))
                    && (leaves
                        .iter()
                        .all(|leaf| !super::ownership::is_managed(&leaf.ty))
                        || borrowed_parameters.contains(&function.id));
                (!varying.is_empty() && varying.len() < leaves.len() && managed_are_invariant)
                    .then_some(NativeScalarParameter {
                        function: function.id,
                        varying,
                    })
            })
            .collect();
        Self {
            functions,
            borrowed_parameters,
            persistent_lenders,
            scalar_parameters,
        }
    }

    pub(crate) fn has_native_version(&self, function: FunctionId) -> bool {
        self.functions.contains(&function)
    }

    /// Whether every managed leaf of the parameter is preserved by every self edge. The synchronous
    /// caller then remains its lender for the complete native/frames execution of the recursive call.
    pub(crate) fn borrows_parameter(&self, function: FunctionId) -> bool {
        self.borrowed_parameters.contains(&function)
    }

    pub(crate) fn persistent_lenders(&self) -> &HashSet<ValueId> {
        &self.persistent_lenders
    }

    /// A copy-only product parameter can keep invariant leaves in the invocation context and pass
    /// only changing leaves through the recursive worker ABI.
    pub(crate) fn scalar_parameter(&self, function: FunctionId) -> Option<&NativeScalarParameter> {
        self.scalar_parameters
            .iter()
            .find(|parameter| parameter.function == function)
    }
}

fn collect_parameter_leaves(
    pattern: &Pattern,
    path: &mut Vec<usize>,
    result: &mut Vec<NativeParameterLeaf>,
) -> Option<()> {
    match pattern {
        Pattern::Binding { id, ty } => {
            result.push(NativeParameterLeaf {
                id: *id,
                ty: ty.clone(),
                path: path.clone(),
            });
            Some(())
        }
        Pattern::Product { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                path.push(index);
                collect_parameter_leaves(element, path, result)?;
                path.pop();
            }
            Some(())
        }
        Pattern::Wildcard { .. } => None,
    }
}

fn managed_bindings(pattern: &Pattern) -> HashSet<ValueId> {
    let mut result = HashSet::new();
    collect_managed_bindings(pattern, &mut result);
    result
}

fn collect_managed_bindings(pattern: &Pattern, result: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, ty } if super::ownership::is_managed(ty) => {
            result.insert(*id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_managed_bindings(element, result);
            }
        }
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => {}
    }
}

#[cfg(test)]
mod tests;
