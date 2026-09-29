use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, Operation};

use super::super::Definition;

pub(in crate::call_pattern::parameter_lift) fn aliases(
    definitions: &HashMap<ValueId, Definition>,
) -> HashMap<ValueId, ValueId> {
    let direct = definitions
        .iter()
        .filter_map(|(id, definition)| {
            let Definition {
                operation: Operation::Atom(atom),
            } = definition
            else {
                return None;
            };
            Some((*id, atom.binding()?))
        })
        .collect::<HashMap<_, _>>();
    alias_origins(definitions.keys().copied(), &direct)
}

fn alias_origins(
    ids: impl IntoIterator<Item = ValueId>,
    direct: &HashMap<ValueId, ValueId>,
) -> HashMap<ValueId, ValueId> {
    let mut aliases = HashMap::new();
    let mut cycle_members = HashSet::new();
    for id in ids {
        if aliases.contains_key(&id) && !cycle_members.contains(&id) {
            continue;
        }
        let mut current = id;
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        let mut cyclic = false;
        let origin = loop {
            if let Some(origin) = aliases.get(&current)
                && !cycle_members.contains(&current)
            {
                break *origin;
            }
            if !seen.insert(current) {
                cyclic = true;
                let cycle_start = path
                    .iter()
                    .position(|member| *member == current)
                    .expect("repeated alias must be on the current path");
                cycle_members.extend(path[cycle_start..].iter().copied());
                break current;
            }
            path.push(current);
            let Some(next) = direct.get(&current) else {
                break current;
            };
            current = *next;
        };
        if cyclic {
            aliases.insert(id, origin);
        } else {
            for alias in path {
                aliases.insert(alias, origin);
            }
        }
    }
    aliases
}

pub(in crate::call_pattern::parameter_lift) fn origin(
    atom: &Atom,
    aliases: &HashMap<ValueId, ValueId>,
) -> Option<ValueId> {
    let binding = atom.binding()?;
    Some(aliases.get(&binding).copied().unwrap_or(binding))
}

#[cfg(test)]
mod tests {
    use super::alias_origins;
    use crate::anf::ast::ValueId;
    use std::collections::HashMap;

    #[test]
    fn resolves_long_alias_chains_once() {
        let ids = (0..20_000).map(ValueId::Temporary).collect::<Vec<_>>();
        let direct = ids
            .windows(2)
            .map(|pair| (pair[0], pair[1]))
            .collect::<HashMap<_, _>>();

        let aliases = alias_origins(ids.iter().copied(), &direct);

        assert_eq!(aliases.len(), ids.len());
        assert!(aliases.values().all(|origin| *origin == ids[19_999]));
    }

    #[test]
    fn keeps_each_cyclic_alias_as_its_own_origin() {
        let first = ValueId::Temporary(0);
        let second = ValueId::Temporary(1);
        let direct = HashMap::from([(first, second), (second, first)]);

        let aliases = alias_origins([first, second], &direct);

        assert_eq!(aliases[&first], first);
        assert_eq!(aliases[&second], second);
    }
}
