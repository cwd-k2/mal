# Symbol round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with Symbol
operators, appends a suffix with `Symbol + Symbol`, and sends the resulting Symbol to the host. The
host writes its result into a `MalSymbolAdmission`, then finishes it as an immutable `Symbol`
before mal observes the value. The runtime owns the construction buffer throughout admission.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/symbol-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip
/tmp/mal-symbol-round-trip
```

Expected output:

```text
9 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, `\xff`, and the concatenated suffix.
