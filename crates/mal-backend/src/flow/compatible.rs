use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, FunctionId};
use mal_frontend::check::ast::Type;
use mal_frontend::check::type_fingerprint::TypeFingerprints;

/// The functions whose parameter and result types equal those of a callee. It bounds the targets of an
/// application that no flow fact narrows.
pub(crate) struct CompatibleTargets {
    groups: Vec<TargetGroup>,
    signatures: HashMap<(u64, u64), Vec<CompatibleGroup>>,
    /// Each function's group and its position there, which is program order.
    members: HashMap<FunctionId, (CompatibleGroup, usize)>,
    fingerprints: TypeFingerprints,
}

/// One set of functions sharing a parameter and result type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompatibleGroup(usize);

struct TargetGroup {
    parameter: Type,
    result: Type,
    targets: Vec<FunctionId>,
}

impl CompatibleTargets {
    pub(crate) fn new(program: &closure::Program) -> Self {
        let mut index = Self {
            groups: Vec::new(),
            signatures: HashMap::new(),
            members: HashMap::new(),
            fingerprints: TypeFingerprints::default(),
        };
        for function in &program.functions {
            let parameter = &function.parameter.ty;
            let result = &function.body.result.ty;
            let group = match index.find(parameter, result) {
                Some(group) => group,
                None => {
                    let group = CompatibleGroup(index.groups.len());
                    index.groups.push(TargetGroup {
                        parameter: parameter.clone(),
                        result: result.clone(),
                        targets: Vec::new(),
                    });
                    let fingerprint = index.fingerprints.signature(parameter, result);
                    index.signatures.entry(fingerprint).or_default().push(group);
                    group
                }
            };
            let targets = &mut index.groups[group.0].targets;
            index.members.insert(function.id, (group, targets.len()));
            targets.push(function.id);
        }
        index
    }

    fn find(&mut self, parameter: &Type, result: &Type) -> Option<CompatibleGroup> {
        let fingerprint = self.fingerprints.signature(parameter, result);
        self.signatures
            .get(&fingerprint)?
            .iter()
            .copied()
            .find(|group| {
                let group = &self.groups[group.0];
                group.parameter == *parameter && group.result == *result
            })
    }

    /// The group of functions a callee of this type may call, if any function has its type.
    pub(crate) fn group(&mut self, callee: &closure::Atom) -> Option<CompatibleGroup> {
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        self.find(parameter, result)
    }

    /// Every function of the group in program order.
    pub(crate) fn targets(&self, group: Option<CompatibleGroup>) -> &[FunctionId] {
        group.map_or(&[], |group| &self.groups[group.0].targets)
    }

    pub(crate) fn contains(&self, group: Option<CompatibleGroup>, function: FunctionId) -> bool {
        group.is_some_and(|group| {
            self.members
                .get(&function)
                .is_some_and(|(member_group, _)| *member_group == group)
        })
    }

    /// The members of `group` among `functions`, in program order.
    pub(crate) fn members_of(
        &self,
        group: Option<CompatibleGroup>,
        functions: &HashSet<FunctionId>,
    ) -> Vec<FunctionId> {
        let Some(group) = group else {
            return Vec::new();
        };
        let mut members = functions
            .iter()
            .filter_map(|function| {
                let (member_group, position) = self.members.get(function)?;
                (*member_group == group).then_some((*position, *function))
            })
            .collect::<Vec<_>>();
        members.sort_unstable_by_key(|(position, _)| *position);
        members.into_iter().map(|(_, function)| function).collect()
    }
}
