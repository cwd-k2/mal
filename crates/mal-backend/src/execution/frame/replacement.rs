use std::collections::{HashMap, VecDeque};

use crate::control::ast::{Program, StateId};

use super::ControlFrame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Availability {
    Unreachable,
    Available(StateId),
    Unavailable,
}

impl Availability {
    /// Meets a must-property: reuse survives a merge only when every reachable path carries the
    /// same retired frame.
    fn meet(self, incoming: Self) -> Self {
        match (self, incoming) {
            (Self::Unreachable, value) | (value, Self::Unreachable) => value,
            (Self::Available(left), Self::Available(right)) if left == right => self,
            _ => Self::Unavailable,
        }
    }
}

pub(super) struct Plan {
    pub(super) replacements: HashMap<StateId, StateId>,
}

impl Plan {
    /// Finds frame allocations that can reuse storage retired on every path to their site.
    ///
    /// Function entries start unavailable, each resume point introduces its retired call frame,
    /// and another frame allocation is a propagation barrier. Merging ordinary entry flow or two
    /// distinct retired origins therefore makes reuse unavailable.
    pub(super) fn new(program: &Program, frames: &HashMap<StateId, ControlFrame>) -> Self {
        let mut availability = vec![Availability::Unreachable; program.states.len()];
        let mut pending = VecDeque::new();
        for entry in program
            .bindings
            .iter()
            .map(|binding| binding.entry)
            .chain(program.functions.iter().map(|function| function.entry))
        {
            merge(
                &mut availability,
                &mut pending,
                entry,
                Availability::Unavailable,
            );
        }
        for (site, frame) in frames {
            merge(
                &mut availability,
                &mut pending,
                frame.resume,
                Availability::Available(*site),
            );
        }

        while let Some(site) = pending.pop_front() {
            if frames.contains_key(&site) {
                continue;
            }
            let current = availability[site.0];
            for successor in program.states[site.0].terminator.successors() {
                merge(&mut availability, &mut pending, successor, current);
            }
        }

        let replacements = frames
            .keys()
            .filter_map(|site| match availability[site.0] {
                Availability::Available(frame) => Some((*site, frame)),
                Availability::Unreachable | Availability::Unavailable => None,
            })
            .collect();
        Self { replacements }
    }

    pub(super) fn retired_frame(&self, site: StateId) -> Option<StateId> {
        self.replacements.get(&site).copied()
    }

    pub(super) fn is_valid(
        &self,
        program: &Program,
        frames: &HashMap<StateId, ControlFrame>,
    ) -> bool {
        self.replacements == Self::new(program, frames).replacements
    }
}

fn merge(
    availability: &mut [Availability],
    pending: &mut VecDeque<StateId>,
    site: StateId,
    incoming: Availability,
) {
    let merged = availability[site.0].meet(incoming);
    if merged != availability[site.0] {
        availability[site.0] = merged;
        pending.push_back(site);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meet_rejects_ordinary_and_distinct_retired_origins() {
        let first = StateId(1);
        let second = StateId(2);

        assert_eq!(
            Availability::Unavailable.meet(Availability::Available(first)),
            Availability::Unavailable
        );
        assert_eq!(
            Availability::Available(first).meet(Availability::Available(second)),
            Availability::Unavailable
        );
        assert_eq!(
            Availability::Available(first).meet(Availability::Available(first)),
            Availability::Available(first)
        );
    }
}
