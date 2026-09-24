# Generic loop example

This example implements iteration as the generic function
`loop<A, B> :: (A, A -> [A, B]) -> B`. The callback returns variant 0 with the next complete state
or variant 1 with the loop result. Call sites name the existing direct result binders `continue` and
`break`; those names are not special syntax.

The example derives several iteration styles from the same function:

| Function | Visited values | Empty case | Result |
|---|---|---|---|
| `upto` | `start` through `end`, ascending and inclusive | `start > end` | Final accumulator |
| `downto` | `start` through `end`, descending and inclusive | `start < end` | Final accumulator |
| `times` | Exactly `count` transformations | `count == 0` | Initial value |
| `anyUpto` | An inclusive ascending range until a match | Reversed range | `false` |
| `foldBuffer` | Every element in index order | Empty `Buffer` | Initial accumulator |

Their call sites use receiver-first application where the starting value or repetition count is the
natural subject. `sumOddThrough` traverses one million values, `factorialDownFrom` descends without
unsigned underflow at the endpoint, and `repeat` carries an owned `Symbol`. Together they exercise
scalar, product, captured, managed, collection, and early-result paths without adding mutable
bindings or a loop primitive.

`upto`, `downto`, and `foldBuffer` pass the accumulator before the current element or index, matching
the order in their type signatures. Their implementations check the terminal endpoint before
performing the final unsigned increment or decrement. Thus `upto` remains defined when `end` is the
largest `UInt64`, and `downto` remains defined when `end` is zero.

Receiver-first application is used only where the receiver is a useful subject: a starting value,
repetition count, or sequence. It remains ordinary lexical function application—`start.upto(...)`
does not perform type-directed method lookup. The lower-level `loop(state, step)` keeps prefix form
because an arbitrary state product is configuration, not a domain object.

`foldBuffer` is the pure accumulator-carrying counterpart of a collection `for`. An effectful
`forEach` can use the same cursor state with a callback returning `Unit`; it is omitted because this
standalone example deliberately has no host-visible effects.

The recursive call in `loop` has no work pending after it. The `malc` closes the step
closure and sum continuation into a recursive control region, places activation-local temporaries in
the region function's entry block, and executes the back edge without growing the native stack with
the transition count. The example is built and executed in both `baseline` and `production` optimization modes.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/generic-loop/program.mal --output /tmp/mal-generic-loop
/tmp/mal-generic-loop
```

The executable prints nothing and exits with status 0 after checking every derived iteration form.
