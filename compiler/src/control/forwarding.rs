use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{AtomKind, Pattern, Reference};

use super::ast::{Operation, State, StateId, Terminator};

pub(super) fn normalize_calls(states: &mut [State], start: usize) {
    let forwarded = (start..states.len())
        .filter(|index| {
            let Terminator::Call { resume, .. } = states[*index].terminator else {
                return false;
            };
            returns_input(states, resume)
        })
        .collect::<Vec<_>>();

    for index in forwarded {
        let Terminator::Call {
            callee, argument, ..
        } = states[index].terminator.clone()
        else {
            unreachable!("forwarded sites were selected from calls")
        };
        states[index].terminator = Terminator::TailCall { callee, argument };
    }
}

fn returns_input(states: &[State], start: StateId) -> bool {
    if is_unit_pattern(states[start.0].input.as_ref()) {
        return returns_unit(states, start);
    }
    let Some(mut value) = binding_id(states[start.0].input.as_ref()) else {
        return false;
    };
    let mut state = start;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert((state, value)) {
            return false;
        }
        let current = &states[state.0];
        for binding in &current.bindings {
            let Some(next) = binding_id(Some(&binding.pattern)) else {
                return false;
            };
            let Operation::Atom(atom) = &binding.operation else {
                return false;
            };
            if binding_reference(atom) != Some(value) {
                return false;
            }
            value = next;
        }
        match &current.terminator {
            Terminator::Return(result) => return binding_reference(result) == Some(value),
            Terminator::Goto(target) if states[target.0].input.is_none() => state = *target,
            Terminator::Jump {
                target,
                value: argument,
            } if binding_reference(argument) == Some(value) => {
                let Some(input) = binding_id(states[target.0].input.as_ref()) else {
                    return false;
                };
                state = *target;
                value = input;
            }
            _ => return false,
        }
    }
}

fn returns_unit(states: &[State], start: StateId) -> bool {
    let mut state = start;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(state) {
            return false;
        }
        let current = &states[state.0];
        for binding in &current.bindings {
            if !is_unit_pattern(Some(&binding.pattern)) {
                return false;
            }
            let Operation::Atom(atom) = &binding.operation else {
                return false;
            };
            if atom.ty != Type::Unit {
                return false;
            }
        }
        match &current.terminator {
            Terminator::Return(result) => return result.ty == Type::Unit,
            Terminator::Goto(target) if states[target.0].input.is_none() => state = *target,
            Terminator::Jump {
                target,
                value: argument,
            } if argument.ty == Type::Unit && is_unit_pattern(states[target.0].input.as_ref()) => {
                state = *target
            }
            _ => return false,
        }
    }
}

fn is_unit_pattern(pattern: Option<&Pattern>) -> bool {
    match pattern {
        Some(Pattern::Binding { ty, .. }) | Some(Pattern::Wildcard { ty, .. }) => *ty == Type::Unit,
        Some(Pattern::Product { .. }) | None => false,
    }
}

fn binding_id(pattern: Option<&Pattern>) -> Option<ValueId> {
    match pattern? {
        Pattern::Binding { id, .. } => Some(*id),
        Pattern::Wildcard { .. } | Pattern::Product { .. } => None,
    }
}

fn binding_reference(atom: &crate::closure::ast::Atom) -> Option<ValueId> {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) => Some(id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};
    use crate::{anf, check, closure, core, parser, resolve};

    fn control(source: &str) -> super::super::ast::Program {
        let source = SourceFile::new(FileId::new(92), "forwarding.mal", source.into());
        let parsed = parser::parse(&source).expect("parse forwarding fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve forwarding fixture");
        let checked = check::check(&resolved).expect("check forwarding fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize forwarding fixture"));
        let anf = anf::lower(&core);
        let closure = closure::convert(&anf);
        super::super::lower(&closure)
    }

    #[test]
    fn makes_an_identity_result_continuation_a_tail_call() {
        let program = control(
            "walk :: Int32 -> Int32 := (value) -> [return] => {\n\
               when (value == 0i32) { return(0i32); };\n\
               child := walk(value - 1i32);\n\
               return(child);\n\
             };\n\
             main :: Unit -> Int32 := () -> { walk(2i32); };",
        );
        let walk = &program.functions[0];
        let calls = program
            .states
            .iter()
            .filter_map(|state| match &state.terminator {
                Terminator::Call { callee, .. }
                    if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(id)) if id == walk.id) =>
                {
                    Some(false)
                }
                Terminator::TailCall { callee, .. }
                    if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(id)) if id == walk.id) =>
                {
                    Some(true)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(calls, vec![true]);
    }

    #[test]
    fn preserves_a_continuation_that_transforms_the_result() {
        let program = control(
            "walk :: Int32 -> Int32 := (value) -> [return] => {\n\
               when (value == 0i32) { return(0i32); };\n\
               child := walk(value - 1i32);\n\
               return(child + 1i32);\n\
             };\n\
             main :: Unit -> Int32 := () -> { walk(2i32) - 2i32; };",
        );
        let walk = &program.functions[0];
        assert!(program.states.iter().any(|state| matches!(
            &state.terminator,
            Terminator::Call { callee, .. }
                if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(id)) if id == walk.id)
        )));
    }

    #[test]
    fn makes_a_unit_result_continuation_a_tail_call() {
        let program = control(
            "walk :: Int32 -> Unit := (value) -> {\n\
               if (value == 0i32) then { () } else {\n\
                 walk(value - 1i32);\n\
               };\n\
             };\n\
             main :: Unit -> Int32 := () -> { walk(2i32); 0i32; };",
        );
        let walk = &program.functions[0];
        assert!(program.states.iter().any(|state| matches!(
            &state.terminator,
            Terminator::TailCall { callee, .. }
                if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(id)) if id == walk.id)
        )));
        assert!(!program.states.iter().any(|state| matches!(
            &state.terminator,
            Terminator::Call { callee, .. }
                if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(id)) if id == walk.id)
        )));
    }

    #[test]
    fn preserves_a_unit_continuation_with_an_external_effect() {
        let program = control(
            "extern observe :: Unit -> Unit;\n\
             walk :: Int32 -> Unit := (value) -> {\n\
               if (value == 0i32) then { () } else {\n\
                 child := walk(value - 1i32);\n\
                 observed := observe(child);\n\
                 observed;\n\
               };\n\
             };\n\
             main :: Unit -> Int32 := () -> { walk(2i32); 0i32; };",
        );
        assert!(program.states.iter().any(|state| matches!(
            &state.terminator,
            Terminator::Call { callee, .. }
                if matches!(callee.kind, AtomKind::Reference(Reference::SelfClosure(_)))
        )));
    }
}
