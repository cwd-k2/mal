# Symbol round-trip example

This example receives arbitrary bytes from a host scratch buffer, validates them with Symbol
operators, appends a suffix with `Symbol + Symbol`, and sends the resulting Symbol to the host. The
host describes its bytes with `mal_Symbol_from_bytes`; `mal_Symbol_return` copies them into an
immutable mal-owned `Symbol` before the host body ends.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/symbol-round-trip/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip
/tmp/mal-symbol-round-trip
```

Expected output:

```text
9 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, `\xff`, and the concatenated suffix.
