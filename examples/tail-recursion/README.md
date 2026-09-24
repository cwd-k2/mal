# Tail recursion example

This example sums one million integers with an annotated self-recursive function. Its recursive
call is in direct tail position, so the LLVM backend emits a control transition without consuming
native stack space in proportion to the input.

The `(total, remaining)` parameters are the complete logical loop state. No list of pending additions
or recursive data value is constructed. This example is specifically about direct annotated
self-recursion; it does not claim that every recursive call or mutually recursive cycle is stack
constant.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- check examples/tail-recursion/program.mal
nix develop --command cargo run -p mal-compiler -- build examples/tail-recursion/program.mal --output /tmp/mal-tail-recursion
/tmp/mal-tail-recursion
```

The executable prints nothing and exits with status 0 after checking the result.
