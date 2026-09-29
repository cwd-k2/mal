# Control and iteration

`loop<State, Result>` represents a transition as `[State, Result]`: the first variant carries the
complete next state and the second finishes with an independently typed result. `foldRange` and
`foldBuffer` specialize that protocol. `sumDownFrom` shows the lower-level alternative: annotated
direct self-recursion with no work pending after the recursive edge.

> [!NOTE]
> `continue` and `finish` are ordinary result-binder names chosen by the program. mal has no hidden
> loop, `break`, or `continue` construct behind this protocol.

The inclusive range checks its endpoint before incrementing, so the final `UInt64` value cannot
overflow. The executable traverses one million values in both `baseline` and `production` compiler
profiles without making iteration a language primitive.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/control-and-iteration/program.mal --output /tmp/mal-control
/tmp/mal-control
```

Success is silent and exits with status 0.
