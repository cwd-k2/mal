use super::*;

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn usage_message(values: &[&str]) -> String {
    let outcome = execute(args(values));
    assert_eq!(outcome.status, ExitStatus::UsageError, "{values:?}");
    assert!(outcome.stdout.is_empty());
    outcome.stderr
}

#[test]
fn no_arguments_prints_help_successfully() {
    let outcome = execute(args(&[]));
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.stdout, HELP);
    assert!(outcome.stderr.is_empty());
}

#[test]
fn version_uses_the_version_contract() {
    let outcome = execute(args(&["--version"]));
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.stdout, "malc 0.6.0-dev (language v0.6)\n");
}

#[test]
fn unknown_commands_are_usage_errors() {
    assert!(usage_message(&["unknown", "sample.mal"]).contains("unknown command"));
    assert!(usage_message(&["format", "sample.mal"]).contains("unknown command"));
    assert!(usage_message(&["emit", "bogus", "sample.mal"]).contains("emit requires"));
}

#[test]
fn validates_build_options_before_running_the_driver() {
    assert!(usage_message(&["build", "sample.mal"]).contains("build requires --output"));
    assert!(usage_message(&["build"]).contains("build requires a source path"));
    assert!(
        usage_message(&["build", "sample.mal", "-o", "program", "--link", "host.c"])
            .contains("unknown build option '--link'")
    );
    assert!(
        usage_message(&["build", "sample.mal", "-o", "one", "--output", "two"])
            .contains("--output may only be specified once")
    );
    assert!(
        usage_message(&[
            "build",
            "sample.mal",
            "-o",
            "program",
            "--artifact-dir",
            "one",
            "--artifact-dir",
            "two",
        ])
        .contains("--artifact-dir may only be specified once")
    );
    assert!(
        usage_message(&[
            "build",
            "sample.mal",
            "-o",
            "program",
            "--optimization",
            "fast",
        ])
        .contains("must be 'baseline' or 'production'")
    );
    assert!(
        usage_message(&["build", "one.mal", "two.mal", "-o", "program"])
            .contains("build takes one source path")
    );
    assert!(usage_message(&["build", "sample.mal", "-o"]).contains("requires a value"));
}

#[test]
fn emit_commands_accept_only_their_own_options() {
    assert!(
        usage_message(&["emit", "header", "sample.mal", "--optimization", "baseline"])
            .contains("unknown emit header option '--optimization'")
    );
    assert!(
        usage_message(&["emit", "atcoder", "sample.mal", "--target", "other"])
            .contains("unknown emit atcoder option '--target'")
    );
    assert!(
        usage_message(&["emit", "host", "sample.mal", "--header", "invalid\"name.h"])
            .contains("not valid in a quoted C include")
    );
}
