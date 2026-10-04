use super::*;

#[test]
fn buffer_rejects_incomplete_lifecycle_callbacks() {
    let fixture = NativeFixture::new("runtime-buffer-callbacks");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent");
    let runtime = repository.join("crates/mal-backend/runtime/c11");
    let harness = fixture.write(
        "buffer_check.c",
        r#"#include "runtime.h"

static void retain_value(MalContext *context, void *carrier) {
    (void)context;
    (void)carrier;
}

static void release_value(void *carrier) {
    (void)carrier;
}

int main(int argc, char **argv) {
    MalContext context = {0};
    if (argc != 2) return 1;
    if (argv[1][0] == 'r') {
        (void)mal_runtime_buffer_make_managed(&context, 1, 0, retain_value, 0);
    } else {
        (void)mal_runtime_buffer_make_managed(&context, 1, 0, 0, release_value);
    }
    return 2;
}
"#,
    );
    let executable = fixture.join("buffer-check");
    let compiled = Command::new("clang")
        .args([
            OsStr::new("-std=c11"),
            OsStr::new("-Wall"),
            OsStr::new("-Wextra"),
            OsStr::new("-Werror"),
            OsStr::new("-O2"),
            OsStr::new("-I"),
            runtime.as_os_str(),
            harness.as_os_str(),
            runtime.join("core.c").as_os_str(),
            runtime.join("bytes.c").as_os_str(),
            runtime.join("buffer.c").as_os_str(),
            OsStr::new("-o"),
            executable.as_os_str(),
        ])
        .output()
        .expect("compile Buffer runtime harness");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    for mode in ["retain-only", "release-only"] {
        let trapped = Command::new(&executable)
            .arg(mode)
            .output()
            .expect("run Buffer callback probe");
        assert!(!trapped.status.success());
        assert!(
            String::from_utf8_lossy(&trapped.stderr)
                .contains("mal trap: buffer lifecycle callback mismatch"),
            "{}",
            String::from_utf8_lossy(&trapped.stderr)
        );
    }
}
