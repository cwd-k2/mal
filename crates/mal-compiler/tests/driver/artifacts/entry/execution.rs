use super::super::*;

#[test]
fn builds_a_constant_main_through_the_llvm_artifact_set() {
    let directory = NativeFixture::new("driver-llvm");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write("program.mal", "main :: Unit -> Int32 := () -> { 7; };");

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
    assert_eq!(directory.run(executable).status.code(), Some(7));
}

#[test]
fn executes_result_blocks_through_the_existing_calling_convention() {
    let directory = NativeFixture::new("driver-result-block");
    let source = directory.write(
        "program.mal",
        "Choice :: [Int32, Int32];\n\
         choose :: Bool -> Choice := (condition) -> [yes, no] => {\n\
           when (condition) { yes(40) };\n\
           no(1)\n\
         };\n\
         addTwo :: Int32 -> Int32 := (value) -> [return] => {\n\
           return(1 + if (value == 40) then { return(42) } else { value + 1 })\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           choose(true)[(value) -> { addTwo(value) }, (value) -> { value }]\n\
         };",
    );
    let executable = directory.join("program");
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
    assert_eq!(directory.run(executable).status.code(), Some(42));
}

#[test]
fn executes_direct_blocks_and_local_result_continuations() {
    let directory = NativeFixture::new("driver-result-block");
    let source = directory.write(
        "program.mal",
        "select :: Bool -> Int32 := (condition) -> [return] => {
           value :: Int32 := {
             when (condition) { return(7) };
             base :: Int32 := 40;
             base + 1
           };
           return([done] => { done(value + 1) })
         };
         main :: Unit -> Int32 := () -> {
           select(false)
         };",
    );
    let executable = directory.join("program");
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
    assert_eq!(directory.run(executable).status.code(), Some(42));
}

#[test]
fn compiles_empty_elimination_as_an_unreachable_zero_arm_case() {
    let directory = NativeFixture::new("driver-empty-return");
    let source = directory.write(
        "program.mal",
        "never :: Unit -> [] := () -> { never()[] };\n\
         main :: Unit -> Int32 := () -> { 0 };",
    );
    let executable = directory.join("program");
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
