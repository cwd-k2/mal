# Print and closure example

This example exercises a file-private host `Int32 -> Unit` operation, source-ordered effects, an
escaping closure, and wrapping `Int32` arithmetic through the public compiler driver. The private
name `_printInt32` remains part of the host interface as `MAL_DEFINE__printInt32`; its visibility only
prevents another mal file from importing it. `makeAdder` returns code together with captured `x`; it
does not expose or prescribe the compiler's closure layout. The three writes also make evaluation
order observable.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- check examples/print-and-closure/program.mal
nix develop --command cargo run -p mal-compiler -- build examples/print-and-closure/program.mal --output /tmp/mal-print-and-closure
/tmp/mal-print-and-closure
```

Expected output:

```text
1
15
-2147483648
```
