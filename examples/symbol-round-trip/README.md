# Symbol round-trip example

This example borrows a host scratch buffer through `Address`, admits its initialized prefix into an
immutable mal-owned `Symbol`, validates it with Symbol operators, and appends a suffix with
`Symbol + Symbol`. It then stores the resulting bytes back into the external buffer and lends the
`Address` and length to the host. No Symbol carrier or ownership crosses the ABI.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- check examples/symbol-round-trip/program.mal
nix develop --command cargo run -p mal-compiler -- build examples/symbol-round-trip/program.mal --output /tmp/mal-symbol-round-trip
/tmp/mal-symbol-round-trip
```

Expected output:

```text
9 bytes
```

The executable exits with status 0 after checking UTF-8 source bytes, an embedded NUL, `\xff`, and the concatenated suffix.
