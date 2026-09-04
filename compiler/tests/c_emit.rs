use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use malc::anf;
use malc::c_emit;
use malc::check;
use malc::closure;
use malc::core;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn emit(text: &str) -> Result<c_emit::Output, malc::diagnostic::Diagnostic> {
    let source = SourceFile::new(FileId::new(79), "c-emit-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    c_emit::emit(&closure)
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("mal-c-emit-test-{}-{sequence}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test directory");
    }
}

fn compile_and_run(source: &str, host: &str) -> Output {
    let generated = emit(source).expect("emit C");
    let directory = TestDirectory::new();
    fs::write(
        directory.path().join(c_emit::GENERATED_HEADER_NAME),
        generated.header,
    )
    .expect("write header");
    fs::write(directory.path().join("program.c"), generated.source).expect("write C source");
    let executable = directory.path().join("program");
    let mut compiler = Command::new("clang");
    compiler.current_dir(directory.path()).args([
        "-std=c11",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-pedantic",
        "program.c",
    ]);
    if !host.is_empty() {
        fs::write(directory.path().join("host.c"), host).expect("write host source");
        compiler.arg("host.c");
    }
    let compilation = compiler
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("run Clang");
    assert!(
        compilation.status.success(),
        "Clang failed:\n{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    Command::new(executable)
        .output()
        .expect("run generated program")
}

const PRINT_HOST: &str = r#"#include "program.mal.h"
#include <stdio.h>

void mal_ext_printInt32(MalContext *context, int32_t value) {
    (void)context;
    printf("%d\n", value);
}
"#;

#[test]
fn emits_the_m0_host_abi_and_executes_the_host_example() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn executes_escaping_capturing_closures() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         makeAdder :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           return \\<x>(y :: Int32) { return x + y; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           addTen := makeAdder(10);\n\
           extern printInt32(addTen(5));\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "15\n");
}

#[test]
fn preserves_short_circuit_and_eager_bool_equality_order() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         marked :: Int32 -> Bool := \\(value :: Int32) {\n\
           extern printInt32(value);\n\
           return value == 1;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           false && marked(2);\n\
           true || marked(3);\n\
           marked(4) == marked(5);\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "4\n5\n");
}

#[test]
fn implements_wrapping_int32_arithmetic_without_signed_overflow() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(2147483647 + 1);\n\
           extern printInt32(-2147483648 * -1);\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "-2147483648\n-2147483648\n"
    );
}

#[test]
fn executes_sum_injection_and_case() {
    let output = compile_and_run(
        "Maybe :: [Unit, Int32];\n\
         extern printInt32 :: Int32 -> Unit;\n\
         get :: Maybe -> Int32 := \\(value :: Maybe) {\n\
           return case value { [0](_) => 0; [1](item) => item; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(get(Maybe[1](9)));\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "9\n");
}

#[test]
fn traps_invalid_int32_division_and_remainder() {
    for (expression, message) in [
        ("1 / 0", "division by zero"),
        ("-2147483648 / -1", "signed division overflow"),
        ("1 % 0", "remainder by zero"),
        ("-2147483648 % -1", "signed remainder overflow"),
    ] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ return {expression}; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "expression: {expression}"
        );
    }
}

#[test]
fn rejects_programs_outside_the_m0_c_boundary() {
    assert!(
        emit(
            "extern choose :: [Unit, Unit] -> Int32;\n\
         main :: Unit -> Int32 := \\() { return 0; };"
        )
        .unwrap_err()
        .message
        .contains("outside the M0 C ABI")
    );
    assert!(
        emit("value :: Int32 := 1Int32;")
            .unwrap_err()
            .message
            .contains("has no")
    );
    assert!(
        emit("main :: Int32 -> Int32 := \\(value :: Int32) { return value; };")
            .unwrap_err()
            .message
            .contains("wrong type")
    );
}
