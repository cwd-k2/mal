use super::*;

#[test]
fn pool_kernel_preserves_coordinates_metadata_and_identity_across_growth() {
    let fixture = NativeFixture::new("runtime-pool");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent");
    let runtime = repository.join("crates/mal-backend/runtime/c11");
    let harness = fixture.write(
        "pool_check.c",
        r#"#include "runtime.h"

#include <stdint.h>

typedef struct {
    uint64_t first;
    uint64_t second;
    uint64_t third;
} Header;

int main(void) {
    MalContext context = {0};
    Header header = {3, 5, 7};
    void *pool = mal_runtime_pool_make(&context, &header, sizeof header, sizeof(uint64_t));
    void *alias = mal_runtime_environment_retain(&context, pool);
    if (mal_runtime_pool_capacity(pool) != 0) return 1;

    Header observed = {0};
    mal_runtime_pool_meta(pool, &observed);
    if (observed.first != 3 || observed.second != 5 || observed.third != 7) return 2;
    Header replacement = {11, 13, 17};
    Header previous = {0};
    mal_runtime_pool_swap_meta(alias, &replacement, &previous);
    if (previous.first != 3 || previous.second != 5 || previous.third != 7) return 3;

    mal_runtime_pool_grow(&context, pool, 10);
    for (size_t index = 0; index < 10; ++index) {
        uint64_t value = 0;
        if (mal_runtime_pool_peek(pool, index, &value) != 0) return 4;
    }
    uint64_t first = 41;
    uint64_t old = 0;
    if (mal_runtime_pool_swap(pool, 3, 1, &first, &old) != 0) return 5;
    uint64_t second = 97;
    if (mal_runtime_pool_swap(alias, 7, 1, &second, &old) != 0) return 6;

    mal_runtime_pool_grow(&context, alias, 70);
    if (mal_runtime_pool_capacity(pool) != 80) return 7;
    uint64_t value = 0;
    if (mal_runtime_pool_peek(pool, 3, &value) != 1 || value != 41) return 8;
    if (mal_runtime_pool_peek(pool, 7, &value) != 1 || value != 97) return 9;
    for (size_t index = 10; index < 80; ++index) {
        if (mal_runtime_pool_peek(pool, index, &value) != 0) return 10;
    }

    uint64_t next = 43;
    if (mal_runtime_pool_swap(pool, 3, 1, &next, &old) != 1 || old != 41) return 11;
    if (mal_runtime_pool_swap(pool, 3, 0, 0, &old) != 1 || old != 43) return 12;
    if (mal_runtime_pool_peek(pool, 3, &value) != 0) return 13;
    mal_runtime_pool_meta(alias, &observed);
    if (observed.first != 11 || observed.second != 13 || observed.third != 17) return 14;

    uint8_t unit = 0;
    void *zero = mal_runtime_pool_make(&context, &unit, 0, 0);
    mal_runtime_pool_grow(&context, zero, 9);
    if (mal_runtime_pool_swap(zero, 8, 1, 0, 0) != 0) return 15;
    mal_runtime_pool_grow(&context, zero, 9);
    if (mal_runtime_pool_peek(zero, 8, 0) != 1) return 16;
    mal_runtime_environment_release(zero);

    mal_runtime_environment_release(pool);
    if (mal_runtime_pool_capacity(alias) != 80) return 17;
    mal_runtime_environment_release(alias);
    return 0;
}
"#,
    );
    let executable = fixture.join("pool-check");
    let compiled = Command::new("clang")
        .args([
            OsStr::new("-std=c11"),
            OsStr::new("-O2"),
            OsStr::new("-flto"),
            OsStr::new("-fuse-ld=lld"),
            OsStr::new("-Wall"),
            OsStr::new("-Wextra"),
            OsStr::new("-Werror"),
            OsStr::new("-I"),
            runtime.as_os_str(),
            harness.as_os_str(),
            runtime.join("core.c").as_os_str(),
            runtime.join("pool.c").as_os_str(),
            OsStr::new("-o"),
            executable.as_os_str(),
        ])
        .output()
        .expect("compile Pool runtime harness");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    let executed = fixture.run(executable);
    assert_eq!(
        executed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}
