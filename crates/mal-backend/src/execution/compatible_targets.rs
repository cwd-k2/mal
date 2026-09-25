use std::collections::HashMap;

use crate::closure::ast::{self as closure, FunctionId};
use mal_frontend::check::ast::Type;
use mal_frontend::check::type_fingerprint::TypeFingerprints;

/// The functions whose parameter and result types equal those of a callee. It bounds the targets of an
/// application that no flow fact narrows.
pub(super) struct CompatibleTargets {
    groups: HashMap<(u64, u64), Vec<TargetGroup>>,
    fingerprints: TypeFingerprints,
}

struct TargetGroup {
    parameter: Type,
    result: Type,
    targets: Vec<FunctionId>,
}

impl CompatibleTargets {
    pub(super) fn new(program: &closure::Program) -> Self {
        let mut index = Self {
            groups: HashMap::new(),
            fingerprints: TypeFingerprints::default(),
        };
        for function in &program.functions {
            let parameter = &function.parameter.ty;
            let result = &function.body.result.ty;
            let fingerprint = index.fingerprints.signature(parameter, result);
            let groups = index.groups.entry(fingerprint).or_default();
            if let Some(group) = groups
                .iter_mut()
                .find(|group| group.parameter == *parameter && group.result == *result)
            {
                group.targets.push(function.id);
            } else {
                groups.push(TargetGroup {
                    parameter: parameter.clone(),
                    result: result.clone(),
                    targets: vec![function.id],
                });
            }
        }
        index
    }

    pub(super) fn for_callee(&mut self, callee: &closure::Atom) -> Vec<FunctionId> {
        let Type::Function { parameter, result } = &callee.ty else {
            return Vec::new();
        };
        let fingerprint = self.fingerprints.signature(parameter, result);
        self.groups
            .get(&fingerprint)
            .and_then(|groups| {
                groups
                    .iter()
                    .find(|group| group.parameter == **parameter && group.result == **result)
            })
            .map_or_else(Vec::new, |group| group.targets.clone())
    }
}
