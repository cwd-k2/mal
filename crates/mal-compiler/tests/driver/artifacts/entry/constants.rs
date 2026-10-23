use super::super::*;

#[test]
fn references_closed_top_level_numeric_constants_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-top-level-constant");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "answer :: UInt64 := 42i32.u64;\n\
         main :: Unit -> Int32 := () -> { answer.i32 - 42; };",
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
fn references_target_layout_arithmetic_constants_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-layout-arithmetic-constant");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "recordSize :: ByteSize := 1bytes + 8bytes + 4bytes + 8bytes;\n\
         main :: Unit -> Int32 := () -> { recordSize.usize.i32 - 21; };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn references_structural_closed_top_level_values_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-structural-top-level");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Unit, Symbol];\n\
         (number, text) :: (Int32, Symbol) := (-7i32, \"ok\");\n\
         enabled :: Bool := true;\n\
         makeChoice :: Symbol -> Choice := (value) -> [none, some] => { some(value) };\n\
         main :: Unit -> Int32 := () -> {\n\
           choice := makeChoice(\"yes\");\n\
           choice[\n\
             () -> { 1 },\n\
             (value) -> {\n\
               if (enabled) then {\n\
                 if (number == -7i32) then {\n\
                   if (text == \"ok\" && value == \"yes\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             }]\n\
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
