# Tail recursion example

This example sums one million integers with an annotated self-recursive function. Its recursive
call is in direct tail position, so the LLVM backend emits a control transition without consuming
native stack space in proportion to the input.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/tail-recursion/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/tail-recursion/program.mal --output /tmp/mal-tail-recursion
/tmp/mal-tail-recursion
```

The executable prints nothing and exits with status 0 after checking the result.
