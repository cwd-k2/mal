# JSON query

This capstone reads one JSON value from standard input and applies the sole process argument:
`count` counts JSON values excluding object keys, while `depth` reports maximum value nesting. The
parser validates the complete document and renders either a JSON result or a diagnostic.

The files follow the data path. `bytes.mal` owns the cursor abstraction, `scanner.mal` recognizes
tokens, `parser.mal` owns an explicit frame stack and parser state, `query.mal` validates the query,
and `render.mal` constructs output. Parser transitions use the continue-or-finish sum protocol from
[`control-and-iteration`](../control-and-iteration/); nested structure lives in the frame relation,
not in a recursive syntax tree.

`Bytes` is a cursor over immutable `Symbol` input. Scanner failures and successes use an explicit
sum, and the parser forwards that sum into its own state-or-result transition without an error
sentinel. `JsonStatistics` is opaque outside the parser and is observed through named accessors.

> [!NOTE]
> The frame Buffer is a carrier for parser continuations. Its indices have stack meaning only through
> the push, replace, and pop operations owned by `parser.mal`.

The host reads stdin directly into a mal-owned `Buffer<UInt8>` and moves it into mal; parsing receives
an immutable Symbol snapshot. Output crosses back as a borrowed `Symbol`. Strings and escape syntax
are validated, but `\u` escapes are not decoded and unpaired UTF-16 surrogates are not rejected.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/json-query/program.mal --output /tmp/mal-json-query
'{"name":"mal","items":[true,null,35]}' | /tmp/mal-json-query count
'{"name":"mal","items":[true,null,35]}' | /tmp/mal-json-query depth
```

The two results are `{"ok":true,"count":6}` and `{"ok":true,"depth":3}`.
