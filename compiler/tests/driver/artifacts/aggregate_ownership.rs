use super::*;

#[test]
fn keeps_a_destructured_borrow_bounded_by_its_framed_aggregate_owner() {
    let directory = NativeFixture::new("driver-llvm-borrowed-destructure-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "walk :: ((Symbol, Symbol), Int32) -> Symbol := (state) -> {\n\
           (pair, depth) := state;\n\
           (left, right) := pair;\n\
           if (depth == 0i32) then { left } else {\n\
             child := walk(pair, depth - 1i32);\n\
             if (right # 0usize == 121u8) then { child } else { \"bad\" };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           left := \"x\" + \"x\";\n\
           right := \"y\" + \"y\";\n\
           result := walk((left, right), 10000i32);\n\
           (result # 1usize).i32 - 120i32;\n\
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
fn does_not_create_owners_for_discarded_product_and_sum_results() {
    let directory = NativeFixture::new("driver-llvm-discarded-aggregate");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "Choice :: [Symbol, Unit];\n\
         main :: Unit -> Int32 := () -> {\n\
           left := \"a\" + \"b\";\n\
           right := \"c\" + \"d\";\n\
           _ := (left, right);\n\
           choice :: Choice := [some, none] => { some(left) };\n\
           0i32;\n\
         };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        artifacts.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
    let llvm = std::fs::read_to_string(artifacts.join("program.ll")).expect("read generated LLVM");
    assert!(!llvm.contains("call ptr @mal_runtime_bytes_retain"));
}
