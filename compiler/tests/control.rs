use malc::anf;
use malc::check;
use malc::closure;
use malc::control;
use malc::control::ast::{Function, Operation, State, Terminator};
use malc::core;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn lower_ok(text: &str) -> control::ast::Program {
    let source = SourceFile::new(FileId::new(68), "control-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    control::lower(&closure)
}

fn top_level_function<'a>(program: &'a control::ast::Program, name: &str) -> &'a Function {
    let binding = program
        .bindings
        .iter()
        .find(|binding| {
            matches!(
                &binding.pattern,
                closure::ast::TopLevelPattern::Binding { name: candidate, .. }
                    if candidate == name
            )
        })
        .expect("top-level binding should exist");
    let function = program.states[binding.entry.0]
        .bindings
        .iter()
        .find_map(|binding| match binding.operation {
            Operation::MakeClosure { function, .. } => Some(function),
            _ => None,
        })
        .expect("top-level function should construct a closure");
    program
        .functions
        .iter()
        .find(|candidate| candidate.id == function)
        .expect("constructed closure should have a function")
}

fn reachable_states<'a>(program: &'a control::ast::Program, function: &Function) -> Vec<&'a State> {
    let mut pending = vec![function.entry];
    let mut seen = std::collections::HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let state = &program.states[id.0];
        match &state.terminator {
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => {
                pending.extend(arms.iter().map(|arm| arm.target));
            }
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
        }
        states.push(state);
    }
    states
}

#[test]
fn makes_a_directly_returned_application_a_tail_transition() {
    let program = lower_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         main :: Unit -> Int32 := \\() { identity(42i32); };",
    );
    let main = top_level_function(&program, "main");
    let states = reachable_states(&program, main);
    assert_eq!(states.len(), 1);
    assert!(matches!(states[0].terminator, Terminator::TailCall { .. }));
}

#[test]
fn records_only_caller_values_live_after_a_non_tail_call() {
    let program = lower_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         addAfter :: Int32 -> Int32 := \\(x :: Int32) {\n\
           unused := 7i32;\n\
           called := identity(x);\n\
           called + x;\n\
         };",
    );
    let function = top_level_function(&program, "addAfter");
    let parameter = function.parameter.binding.expect("named parameter");
    let resume = reachable_states(&program, function)
        .into_iter()
        .find_map(|state| match &state.terminator {
            Terminator::Call { resume, .. } => Some(&program.states[resume.0]),
            _ => None,
        })
        .expect("non-tail call should suspend");
    assert_eq!(resume.live.len(), 1);
    assert_eq!(resume.live[0].id, parameter);
    assert!(!resume.needs_environment);
}

#[test]
fn carries_the_caller_environment_when_a_resume_uses_a_capture() {
    let program = lower_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         make :: Int32 -> (Int32 -> Int32) := \\(captured :: Int32) {\n\
           \\(argument :: Int32) {\n\
             called := identity(argument);\n\
             called + captured;\n\
           };\n\
         };",
    );
    let inner = program
        .functions
        .iter()
        .find(|function| !function.environment.is_empty())
        .expect("capturing function should exist");
    assert!(reachable_states(&program, inner).into_iter().any(|state| {
        matches!(
            state.terminator,
            Terminator::Call { resume, .. }
                if program.states[resume.0].needs_environment
        )
    }));
}

#[test]
fn keeps_case_payloads_local_but_saves_them_across_calls_in_the_arm() {
    let program = lower_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         useChoice :: [Int32, Int32] -> Int32 := \\(choice :: [Int32, Int32]) {\n\
           case (choice)\n\
             [0](payload) {\n\
               called := identity(payload);\n\
               called + payload;\n\
             }\n\
             [1](payload) { payload };\n\
         };",
    );
    let function = top_level_function(&program, "useChoice");
    let states = reachable_states(&program, function);
    let Terminator::Case { arms, .. } = &program.states[function.entry.0].terminator else {
        panic!("function should enter through the case dispatch");
    };
    assert!(
        arms.iter()
            .all(|arm| program.states[arm.target.0].input.is_some())
    );
    let resume = states
        .iter()
        .find_map(|state| match &state.terminator {
            Terminator::Call { resume, .. } => Some(&program.states[resume.0]),
            _ => None,
        })
        .expect("first arm should suspend for a non-tail call");
    assert_eq!(resume.live.len(), 1);
}

#[test]
fn gives_a_symbol_live_across_a_call_a_typed_resume_field() {
    let program = lower_ok(
        "identity :: Symbol -> Symbol := \\(value :: Symbol) { value; };\n\
         appendAfter :: Symbol -> Symbol := \\(prefix :: Symbol) {\n\
           called := identity(\"value\");\n\
           prefix + called;\n\
         };",
    );
    let function = top_level_function(&program, "appendAfter");
    let parameter = function.parameter.binding.expect("named parameter");
    let resume = reachable_states(&program, function)
        .into_iter()
        .find_map(|state| match state.terminator {
            Terminator::Call { resume, .. } => Some(&program.states[resume.0]),
            _ => None,
        })
        .expect("identity call should suspend in the semantic control IR");
    assert_eq!(resume.live.len(), 1);
    assert_eq!(resume.live[0].id, parameter);
    assert_eq!(resume.live[0].ty, malc::check::ast::Type::Symbol);
}

#[test]
fn propagates_tail_position_through_primitive_branches() {
    let program = lower_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         choose :: Int32 -> Int32 := \\(x :: Int32) {\n\
           if (x == 0i32) then { identity(1i32) } else { identity(x) };\n\
         };",
    );
    let choose = top_level_function(&program, "choose");
    let states = reachable_states(&program, choose);
    assert!(matches!(
        program.states[choose.entry.0].terminator,
        Terminator::PrimitiveBranch { .. }
    ));
    assert_eq!(
        states
            .iter()
            .filter(|state| matches!(state.terminator, Terminator::TailCall { .. }))
            .count(),
        2
    );
    assert!(
        !states
            .iter()
            .any(|state| matches!(state.terminator, Terminator::Call { .. }))
    );
}
