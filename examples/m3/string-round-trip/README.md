# M3 String round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with String primitives, and returns the borrowed String to the host. The host copies its result with `mal_string_copy`, then overwrites and frees the scratch buffer before mal observes the value.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/m3/string-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/m3/string-round-trip/program.mal --output /tmp/mal-m3-example.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m3/string-round-trip/program.mal --output /tmp/mal-m3-example --link examples/m3/string-round-trip/host.c
/tmp/mal-m3-example
```

Expected output:

```text
8 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, and `\xff`.
