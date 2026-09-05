# String round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with String operators, and returns the borrowed String to the host. The host copies its result with `mal_string_copy`, then overwrites and frees the scratch buffer before mal observes the value.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/string-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/string-round-trip/program.mal --output /tmp/mal-string-round-trip.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/string-round-trip/program.mal --output /tmp/mal-string-round-trip --link examples/string-round-trip/host.c
/tmp/mal-string-round-trip
```

Expected output:

```text
8 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, and `\xff`.
