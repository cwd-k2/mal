use super::*;

#[test]
fn lowers_post_growth_packed_access_through_stable_data() {
    let directory = NativeFixture::new("driver-llvm-stable-packed-data");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           values := pack<Int32>((new, get, put) -> {
             new(40i32);
             value := get(0usize);
             put(0usize, value + 2i32);
             ();
           });
           (values # 0usize) - 42i32;
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
    assert!(llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(llvm.contains("call ptr @llvm.invariant.start.p0"));
    assert!(llvm.contains("getelementptr i8, ptr"));
}
