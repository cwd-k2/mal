# Engram round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with Engram operators, and returns the borrowed Engram to the host. The host copies its result with `mal_engram_copy`, then overwrites and frees the scratch buffer before mal observes the value.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/engram-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/engram-round-trip/program.mal --output /tmp/mal-engram-round-trip.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/engram-round-trip/program.mal --output /tmp/mal-engram-round-trip --link examples/engram-round-trip/host.c
/tmp/mal-engram-round-trip
```

Expected output:

```text
8 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, and `\xff`.
