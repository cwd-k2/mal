use super::*;
#[test]
fn owns_capturing_closure_environments_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-capturing-closure");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () {\n\
           base :: Int32 := 40;\n\
           add :: Int32 -> Int32 := (value) { base + value; };\n\
           add(2) - 42;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn calls_escaping_closures_with_managed_captures_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-escaping-closure");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "makePrefix :: Symbol -> (Symbol -> Symbol) := (prefix) {\n\
           append :: Symbol -> Symbol := (suffix) { prefix + suffix; };\n\
           append;\n\
         };\n\
         apply :: ((Symbol -> Symbol), Symbol) -> Symbol := (operation, value) {\n\
           operation(value);\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           append := makePrefix(\"a\" + \"b\");\n\
           result := apply(append, \"c\" + \"d\");\n\
           if (result == \"abcd\") then { 0 } else { 1 };\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}
