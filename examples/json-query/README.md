# JSON query example

This example reads one JSON value from standard input, validates its complete syntax, and applies a
query selected by the sole process argument. `count` counts JSON values, excluding object keys, and
`depth` reports the maximum container/value nesting depth. Both results are rendered as JSON by the
mal program and written to standard output.

The parser accepts objects, arrays, strings with JSON escapes, numbers, booleans, null, and JSON
whitespace. It defunctionalizes the recursive-descent control flow into one `_parse` dispatcher and
a central stack of parser frames instead of building a recursive syntax tree. Each four-bit frame
records what the enclosing container must do after a child value completes. The intentionally
fixed-width `UInt64` stack admits at most 15 nested containers; a dynamically allocated stack could
remove this example-specific limit without changing the parser states. The host owns input and
output buffers exposed through Address/USize descriptors; admission, Symbol construction, and
decimal rendering stay in mal.

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
