use super::*;

#[test]
fn pool_kernel_preserves_coordinates_header_and_identity_across_growth() {
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

// Models a specialization-time runtime carrier shared by generated C and LLVM:
// a trivial external-opaque field beside an owned Engram field and plain data.
typedef struct {
    uintptr_t external_bits;
    void *child;
    uint32_t tag;
} RuntimeCarrier;

static int retained = 0;
static int released = 0;
static int destroyed = 0;

static void destroy_child(void *child) {
    (void)child;
    ++destroyed;
}

static void *make_child(MalContext *context) {
    return mal_runtime_owner_allocate(context, 1, destroy_child);
}

static void retain_child(void *carrier) {
    void *child = ((RuntimeCarrier *)carrier)->child;
    ++retained;
    mal_runtime_owner_retain(child);
}

static void release_child(void *carrier) {
    void *child = ((RuntimeCarrier *)carrier)->child;
    ++released;
    mal_runtime_owner_release(child);
}

int main(int argc, char **argv) {
    MalContext context = {0};
    if (argc == 2) {
        uint8_t failure_header = 0;
        if (argv[1][0] == 'c') {
            void *failure = mal_runtime_pool_make(&context, &failure_header, 0, 0);
            mal_runtime_pool_grow(&context, failure, 1);
            mal_runtime_pool_grow(&context, failure, SIZE_MAX);
        }
        if (argv[1][0] == 'p') {
            void *failure = mal_runtime_pool_make(&context, &failure_header, 0, 2);
            mal_runtime_pool_grow(&context, failure, SIZE_MAX);
        }
        if (argv[1][0] == 'a') {
            void *failure = mal_runtime_pool_make(&context, &failure_header, 0, 1);
            mal_runtime_pool_grow(&context, failure, SIZE_MAX);
        }
        if (argv[1][0] == 'h') {
            (void)mal_runtime_pool_make(&context, &failure_header, SIZE_MAX, 1);
        }
        return 99;
    }
    Header header = {3, 5, 7};
    void *pool = mal_runtime_pool_make(&context, &header, sizeof header, sizeof(uint64_t));
    void *alias = mal_runtime_owner_retain(pool);
    if (mal_runtime_pool_capacity(pool) != 0) return 1;

    Header observed = {0};
    mal_runtime_pool_header(pool, &observed);
    if (observed.first != 3 || observed.second != 5 || observed.third != 7) return 2;
    Header replacement = {11, 13, 17};
    Header previous = {0};
    mal_runtime_pool_swap_header(alias, &replacement, &previous);
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
    mal_runtime_pool_header(alias, &observed);
    if (observed.first != 11 || observed.second != 13 || observed.third != 17) return 14;

    uint8_t unit = 0;
    void *zero = mal_runtime_pool_make(&context, &unit, 0, 0);
    mal_runtime_pool_grow(&context, zero, 9);
    if (mal_runtime_pool_swap(zero, 8, 1, 0, 0) != 0) return 15;
    mal_runtime_pool_grow(&context, zero, 9);
    if (mal_runtime_pool_peek(zero, 8, 0) != 1) return 16;
    mal_runtime_owner_release(zero);

    RuntimeCarrier managed_header = {(uintptr_t)0x101u, make_child(&context), 7};
    void *managed = mal_runtime_pool_make_managed(
        &context,
        &managed_header,
        sizeof managed_header,
        sizeof(RuntimeCarrier),
        retain_child,
        release_child,
        retain_child,
        release_child
    );
    mal_runtime_pool_grow(&context, managed, 4);
    RuntimeCarrier element = {(uintptr_t)0x202u, make_child(&context), 11};
    RuntimeCarrier old_carrier = {0};
    if (mal_runtime_pool_swap(managed, 2, 1, &element, &old_carrier) != 0) return 18;
    RuntimeCarrier shared = {0};
    if (mal_runtime_pool_peek_managed(&context, managed, 2, &shared) != 1) return 19;
    if (shared.external_bits != (uintptr_t)0x202u || shared.child != element.child || shared.tag != 11) return 20;
    release_child(&shared);

    RuntimeCarrier shared_header = {0};
    mal_runtime_pool_header_managed(&context, managed, &shared_header);
    if (shared_header.external_bits != (uintptr_t)0x101u
        || shared_header.child != managed_header.child
        || shared_header.tag != 7) return 21;
    release_child(&shared_header);
    mal_runtime_pool_grow(&context, managed, 60);
    if (destroyed != 0) return 22;

    RuntimeCarrier next_header = {(uintptr_t)0x303u, make_child(&context), 13};
    mal_runtime_pool_swap_header(managed, &next_header, &old_carrier);
    if (old_carrier.child != managed_header.child || old_carrier.external_bits != (uintptr_t)0x101u) return 23;
    release_child(&old_carrier);
    if (mal_runtime_pool_swap(managed, 2, 0, 0, &old_carrier) != 1) return 24;
    if (old_carrier.child != element.child || old_carrier.external_bits != (uintptr_t)0x202u) return 25;
    release_child(&old_carrier);
    RuntimeCarrier last_element = {(uintptr_t)0x404u, make_child(&context), 17};
    if (mal_runtime_pool_swap(managed, 63, 1, &last_element, &old_carrier) != 0) return 26;
    mal_runtime_owner_release(managed);
    if (retained != 2 || released != 6 || destroyed != 4) return 27;

    mal_runtime_owner_release(pool);
    if (mal_runtime_pool_capacity(alias) != 80) return 28;
    mal_runtime_owner_release(alias);
    return 0;
}
"#,
    );
    let configurations: [(&str, &[&str]); 2] = [
        ("pool-check", &["-O2", "-flto", "-fuse-ld=lld"]),
        (
            "pool-check-sanitize",
            &[
                "-O1",
                "-fsanitize=address,undefined",
                "-fno-omit-frame-pointer",
            ],
        ),
    ];
    for (name, options) in configurations {
        let executable = fixture.join(name);
        let compiled = Command::new("clang")
            .args([
                OsStr::new("-std=c11"),
                OsStr::new("-Wall"),
                OsStr::new("-Wextra"),
                OsStr::new("-Werror"),
            ])
            .args(options)
            .args([
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

    for (mode, message) in [
        ("capacity", "mal trap: pool capacity overflow"),
        ("payload", "mal trap: pool payload size overflow"),
        ("allocation", "mal trap: pool allocation size overflow"),
        ("header", "mal trap: pool header size overflow"),
    ] {
        let trapped = Command::new(fixture.join("pool-check"))
            .arg(mode)
            .output()
            .expect("run Pool overflow probe");
        assert!(!trapped.status.success());
        assert!(
            String::from_utf8_lossy(&trapped.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&trapped.stderr)
        );
    }
}
