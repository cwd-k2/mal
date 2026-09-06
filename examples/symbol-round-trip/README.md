# Symbol round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with Symbol
operators, appends a suffix with `Symbol + Symbol`, and sends the resulting Symbol to the host. The
host copies its result with `mal_Symbol_copy_from_bytes`, then overwrites the stack scratch buffer
before mal observes the value. Stack storage avoids leaving an untransferred heap resource when the
copy helper traps on allocation failure.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/symbol-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip --link examples/symbol-round-trip/host.c
/tmp/mal-symbol-round-trip
```

Expected output:

```text
9 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, `\xff`, and the concatenated suffix.
