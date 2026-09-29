use super::*;

#[test]
fn falls_back_to_control_frames_before_a_small_native_stack_boundary() {
    let directory = NativeFixture::new("driver-small-native-stack");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";
         extern touch :: Int32 -> Int32;
         sum :: Int32 -> Int32 := (value) -> {
           if (value == 0i32) then { 0i32 } else {
             rest := sum(value - 1i32);
             touch(value) + rest;
           };
         };
         main :: Unit -> Int32 := () -> { sum(10000i32) - 50005000i32; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static volatile int32_t observed;\n\
         MAL_DEFINE_touch(call, value) {\n\
             observed = value;\n\
             return mal_Int32_return(call, value);\n\
         }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--optimization"),
        OsStr::new("production"),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = std::process::Command::new("sh")
        .args(["-c", "ulimit -s 64; exec \"$1\"", "mal-small-stack"])
        .arg(executable)
        .output()
        .expect("run native fixture executable with a 64 KiB stack");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
