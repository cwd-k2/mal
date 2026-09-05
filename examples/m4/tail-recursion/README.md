# M4 tail recursion example

This example sums one million integers with an annotated self-recursive function. Its recursive call is in direct tail position, so the generated C executes it as a loop without consuming C stack space in proportion to the input.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/m4/tail-recursion/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/m4/tail-recursion/program.mal --output /tmp/mal-m4-example.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m4/tail-recursion/program.mal --output /tmp/mal-m4-example
/tmp/mal-m4-example
```

The executable prints nothing and exits with status 0 after checking the result.
