//! Cases derived from the language specification.
//!
//! Each file under `tests/spec` holds cases written as `### name | expectation` followed by a program. An
//! expectation is `ok` (the program checks), `err:<text>` (the checker rejects it with a message containing
//! `text`), `run:<code>` (the built program exits with `code`), or `trap` (the built program aborts).

use std::os::unix::process::ExitStatusExt;

use mal_syntax::source::{FileId, SourceFile};

mod support;

use support::NativeFixture;

enum Expectation {
    Accepted,
    Rejected(String),
    Exit(i32),
    Trap,
}

struct Case {
    name: String,
    expectation: Expectation,
    source: String,
}

fn cases(text: &str) -> Vec<Case> {
    text.split("### ")
        .filter(|chunk| !chunk.trim().is_empty())
        .map(|chunk| {
            let (header, source) = chunk.split_once('\n').expect("case header");
            let (name, expectation) = header.split_once(" | ").expect("case header separator");
            Case {
                name: name.trim().to_owned(),
                expectation: expectation_of(expectation.trim()),
                source: source.to_owned(),
            }
        })
        .collect()
}

fn expectation_of(text: &str) -> Expectation {
    if text == "ok" {
        Expectation::Accepted
    } else if text == "trap" {
        Expectation::Trap
    } else if let Some(message) = text.strip_prefix("err:") {
        Expectation::Rejected(message.to_owned())
    } else if let Some(code) = text.strip_prefix("run:") {
        Expectation::Exit(code.parse().expect("exit code"))
    } else {
        panic!("unknown expectation `{text}`");
    }
}

fn check(case: &Case) -> Result<(), String> {
    let source = SourceFile::new(FileId::new(0), "spec.mal", case.source.clone());
    let checked = mal_frontend::analysis::check(&source);
    match (&case.expectation, checked) {
        (Expectation::Accepted, Ok(_)) => Ok(()),
        (Expectation::Accepted, Err(error)) => Err(format!("rejected: {}", error.message)),
        (Expectation::Rejected(_), Ok(_)) => Err("accepted".to_owned()),
        (Expectation::Rejected(text), Err(error)) if error.message.contains(text.as_str()) => {
            Ok(())
        }
        (Expectation::Rejected(_), Err(error)) => {
            Err(format!("wrong rejection: {}", error.message))
        }
        _ => unreachable!("execution cases are not only checked"),
    }
}

fn execute(case: &Case, clang_arguments: &[&str]) -> Result<(), String> {
    let fixture = NativeFixture::new("spec");
    let source = fixture.write("program.mal", &case.source);
    let executable = fixture.join("program");
    let mut arguments = vec![
        std::ffi::OsStr::new("build"),
        source.as_os_str(),
        std::ffi::OsStr::new("-o"),
        executable.as_os_str(),
    ];
    for argument in clang_arguments {
        arguments.push(std::ffi::OsStr::new("--clang-arg"));
        arguments.push(std::ffi::OsStr::new(argument));
    }
    let built = fixture.malc(arguments);
    if !built.status.success() {
        return Err(format!(
            "build failed: {}",
            String::from_utf8_lossy(&built.stderr)
                .lines()
                .next()
                .unwrap_or("")
        ));
    }
    let status = fixture.run(&executable).status;
    match case.expectation {
        Expectation::Exit(code) if status.code() == Some(code) => Ok(()),
        Expectation::Trap if status.signal() == Some(6) => Ok(()),
        _ => Err(format!("unexpected termination: {status}")),
    }
}

fn run_case(case: &Case, clang_arguments: &[&str]) -> Result<(), String> {
    match case.expectation {
        Expectation::Accepted | Expectation::Rejected(_) => check(case),
        Expectation::Exit(_) | Expectation::Trap => execute(case, clang_arguments),
    }
}

/// Runs the cases on several threads; building and running native programs dominates the time.
fn run_all(text: &str) {
    run_all_with(text, &[]);
}

/// Like `run_all`, passing `clang_arguments` to every native build.
fn run_all_with(text: &str, clang_arguments: &[&str]) {
    let cases = cases(text);
    let threads = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8);
    let chunk = cases.len().div_ceil(threads).max(1);
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = cases
            .chunks(chunk)
            .map(|chunk| {
                scope.spawn(|| {
                    chunk
                        .iter()
                        .filter_map(|case| {
                            run_case(case, clang_arguments)
                                .err()
                                .map(|reason| format!("{}: {reason}", case.name))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("case thread"))
            .collect()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn lexical_and_grammar_rules() {
    run_all(include_str!("spec/syntax.txt"));
}

#[test]
fn expression_control_and_program_structure_rules() {
    run_all(include_str!("spec/expressions.txt"));
}

#[test]
fn sum_continuation_branch_rules() {
    run_all(include_str!("spec/sum_continuations.txt"));
}

/// AddressSanitizer turns a managed value released too early, or twice, into a failed run.
#[test]
fn managed_values_stay_owned_across_joins() {
    run_all_with(
        include_str!("spec/joins.txt"),
        &["-fsanitize=address", "-g"],
    );
}

#[test]
fn operator_and_literal_rules() {
    run_all(include_str!("spec/operators.txt"));
}

#[test]
fn generics_extern_and_buffer_rules() {
    run_all(include_str!("spec/generics_and_extern.txt"));
}

#[test]
fn evaluation_semantics() {
    run_all(include_str!("spec/evaluation.txt"));
}

#[test]
fn canonical_memory_layout_matches_the_specification() {
    let fixture = NativeFixture::new("spec-layout");
    let program =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/spec/layout/program.mal");
    let executable = fixture.join("layout");
    let built = fixture.malc([
        std::ffi::OsStr::new("build"),
        program.as_os_str(),
        std::ffi::OsStr::new("-o"),
        executable.as_os_str(),
    ]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = fixture.run(&executable);
    assert!(
        output.status.success(),
        "exit {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}
