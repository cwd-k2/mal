use super::super::*;

#[test]
fn passes_process_arguments_through_the_llvm_entry_bridge() {
    let directory = NativeFixture::new("driver-llvm-arguments");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Buffer<Symbol> -> Int32 := (arguments) -> {\n\
           if (#arguments == 3usize && arguments.get(0usize) == \"alpha\" && #(arguments.get(1usize)) == 0usize\n\
               && arguments.get(2usize) == \"caf\\xc3\\xa9\")\n\
           then 0\n\
           else 1;\n\
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
    let output = std::process::Command::new(executable)
        .args(["alpha", "", "café"])
        .output()
        .expect("run argument-aware LLVM executable");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn passes_an_empty_argument_buffer_without_process_arguments() {
    let directory = NativeFixture::new("driver-llvm-empty-arguments");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Buffer<Symbol> -> Int32 := (arguments) -> {\n\
           if (#arguments == 0usize) then { 0 } else { 2 };\n\
         };",
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
fn passes_many_and_long_process_arguments_without_leaking() {
    let directory = NativeFixture::new("driver-llvm-many-arguments");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "total :: (Buffer<Symbol>, USize, USize) -> USize := (arguments, index, sum) ->\n\
           if (index == #arguments) then sum\n\
           else total(arguments, index + 1usize, sum + #(arguments.get(index)));\n\
         main :: Buffer<Symbol> -> Int32 := (arguments) ->\n\
           if (total(arguments, 0usize, 0usize) == 2000usize * 10usize + 100000usize) then 0 else 1;",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-fsanitize=address"),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arguments = std::iter::repeat_n("abcdefghij".to_owned(), 2000)
        .chain(std::iter::once("x".repeat(100_000)));
    let output = std::process::Command::new(executable)
        .args(arguments)
        .output()
        .expect("run argument-aware executable");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
