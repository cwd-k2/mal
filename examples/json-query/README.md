# JSON query example

This example reads one JSON value from standard input, validates its complete syntax, and applies a
query selected by the sole process argument. `count` counts JSON values, excluding object keys, and
`depth` reports the maximum container/value nesting depth. Both results are rendered as JSON by the
mal program and written to standard output.

The parser accepts objects, arrays, strings with JSON escapes, numbers, booleans, null, and JSON
whitespace. It computes statistics with an explicit parser-frame stack instead of building a
recursive syntax tree, which keeps the representation within mal's current non-recursive type
system. The fixed-width stack admits at most 15 nested containers. The host only reads and writes
bytes and converts one ASCII byte into a `Symbol` for decimal rendering.

Input is expected to be UTF-8. The example validates JSON token and structural syntax, including the
shape of `\u` escapes, but does not decode Unicode escapes or reject unpaired UTF-16 surrogates.

The example also demonstrates passing named functions as exhaustive continuations. In particular,
`query[renderCount, renderDepth]` selects a renderer and returns it as an ordinary function value;
`stats[renderer]` then applies that selected function.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/json-query/program.mal --output /tmp/mal-json-query
'{"name":"mal","items":[true,null,35]}' | /tmp/mal-json-query count
'{"name":"mal","items":[true,null,35]}' | /tmp/mal-json-query depth
```

Expected output:

```json
{"ok":true,"count":6}
{"ok":true,"depth":3}
```
