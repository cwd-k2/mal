use super::*;

/// Byte `*` at the operand's last use moves the byte owner when nothing else holds it and copies otherwise. The
/// fixture exercises the copying cases (a live Buffer alias, a shared Symbol owner, a static literal) beside the
/// moving ones (a uniquely held prefix view, growth after a move), so both profiles must agree on every result.
#[test]
fn moves_byte_owners_at_the_last_use_of_a_conversion_operand() {
    let directory = NativeFixture::new("driver-byte-transfer");
    let source = directory.write("program.mal", include_str!("byte_transfer/program.mal"));
    for (profile, transfers) in [("baseline", false), ("production", true)] {
        let executable = directory.join(profile);
        let artifacts = directory.join(format!("{profile}-artifacts"));
        let output = directory.malc([
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
            OsStr::new("--optimization"),
            OsStr::new(profile),
        ]);
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            directory.run(executable).status.code(),
            Some(0),
            "{profile}"
        );
        let module = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
        for function in [
            "mal_runtime_buffer_into_symbol",
            "mal_runtime_symbol_into_buffer",
        ] {
            assert_eq!(
                module.contains(&format!("call ptr @{function}")),
                transfers,
                "{profile}: {function}"
            );
        }
    }
}
