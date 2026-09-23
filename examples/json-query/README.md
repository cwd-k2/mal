# JSON query example

This example reads one JSON value from standard input, validates its complete syntax, and applies a
query selected by the sole process argument. `count` counts JSON values, excluding object keys, and
`depth` reports the maximum container/value nesting depth. Both results are rendered as JSON by the
mal program and written to standard output.

The parser accepts objects, arrays, strings with JSON escapes, numbers, booleans, null, and JSON
whitespace. It defunctionalizes the recursive-descent control flow into one `_parse` dispatcher and
a central `Buffer<ParserFrame>` stack instead of building a recursive syntax tree. `_parse`
uses the same continue-or-break sum encoding introduced by the
[`generic-loop`](../generic-loop/) example: `continue` carries the next complete `ParserState`, while
`break` carries the final `ParseResult`. Each frame records what the enclosing container must do after
a child value completes. The stack pairs reusable Buffer storage with a logical count: push appends or
reuses one slot, replacement uses `put`, and pop decrements the logical count. Nesting is limited only
by the target-sized count and available memory.

The input bytes and frame bytes are finite carriers with different interpretations. Parser operations
give input positions their token meaning and frame values their control-state meaning. Recursive JSON
topology therefore exists in the transition relation followed by `_parse`, not in a recursive value
type or a materialized syntax tree.

The host owns the stdin allocation. The program admits its initialized prefix into a mal-owned
`Buffer<UInt8>` before releasing that allocation, so parsing no longer depends on host storage.
It admits the selected process argument for the same reason: arguments become ordinary program
values at the entry point. Output rendering uses mal-owned `Symbol` values, then writes them in
chunks through the fixed host buffer instead of treating its capacity as an output limit.

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
