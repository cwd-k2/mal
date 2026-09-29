//! Canonical owner resolution for borrowed-value authority.

use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;

/// Replaces intermediate aliases with the bindings that own their storage.
///
/// A cycle without a resolvable owner is not a valid borrow proof and is discarded conservatively.
pub(super) fn canonicalize(authorities: &mut HashMap<ValueId, HashSet<ValueId>>) {
    let collected = authorities.clone();
    let mut resolved = HashMap::<ValueId, Option<HashSet<ValueId>>>::new();
    for binding in collected.keys() {
        resolve(*binding, &collected, &mut resolved);
    }
    authorities.clear();
    authorities.extend(
        resolved
            .into_iter()
            .filter_map(|(binding, lenders)| lenders.map(|lenders| (binding, lenders))),
    );
}

fn resolve(
    binding: ValueId,
    authorities: &HashMap<ValueId, HashSet<ValueId>>,
    resolved: &mut HashMap<ValueId, Option<HashSet<ValueId>>>,
) -> Option<HashSet<ValueId>> {
    if let Some(lenders) = resolved.get(&binding) {
        return lenders.clone();
    }
    let Some(sources) = authorities.get(&binding) else {
        return Some(HashSet::from([binding]));
    };
    let mut stack = vec![Frame::new(binding, sources)];
    let mut visiting = HashSet::from([binding]);
    loop {
        let Some(frame) = stack.last_mut() else {
            unreachable!("lender resolution starts with one frame");
        };
        let Some(source) = frame.next_source() else {
            let frame = stack.pop().expect("completed lender frame");
            visiting.remove(&frame.binding);
            resolved.insert(frame.binding, Some(frame.lenders.clone()));
            if let Some(parent) = stack.last_mut() {
                parent.lenders.extend(frame.lenders);
                continue;
            }
            return Some(frame.lenders);
        };
        if let Some(source_lenders) = resolved.get(&source) {
            let Some(source_lenders) = source_lenders else {
                break;
            };
            stack
                .last_mut()
                .expect("current lender frame")
                .lenders
                .extend(source_lenders.iter().copied());
        } else if let Some(source_sources) = authorities.get(&source) {
            if !visiting.insert(source) {
                break;
            }
            stack.push(Frame::new(source, source_sources));
        } else {
            stack
                .last_mut()
                .expect("current lender frame")
                .lenders
                .insert(source);
        }
    }
    for frame in stack {
        resolved.insert(frame.binding, None);
    }
    None
}

struct Frame {
    binding: ValueId,
    sources: Vec<ValueId>,
    next: usize,
    lenders: HashSet<ValueId>,
}

impl Frame {
    fn new(binding: ValueId, sources: &HashSet<ValueId>) -> Self {
        Self {
            binding,
            sources: sources.iter().copied().collect(),
            next: 0,
            lenders: HashSet::new(),
        }
    }

    fn next_source(&mut self) -> Option<ValueId> {
        let source = self.sources.get(self.next).copied();
        self.next += usize::from(source.is_some());
        source
    }
}

#[cfg(test)]
#[path = "authority_lenders_tests.rs"]
mod tests;
