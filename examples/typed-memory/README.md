# Typed external memory example

This example obtains mutable storage from the host and accesses canonical `Sample` values through
short-lived typed views. The first view starts at the host allocation; two later views deliberately
start at unaligned addresses. The generic `replace<A>` helper receives one scoped `Region<A>` and
returns the displaced ordinary value. The caller uses that value between sequential views, so no
borrowed authority escapes or overlaps another callback. A C host operation then reads, updates,
and writes the first unaligned sample through the generated canonical-memory helpers. The program
exits with status 0 when aligned access, exact unaligned access, the swap, and the cross-language
update preserve the products.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/typed-memory/program.mal --output /tmp/mal-typed-memory
/tmp/mal-typed-memory
```
