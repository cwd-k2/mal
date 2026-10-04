# Extern runtime extension

The C runtime extension creates a mal-owned `Buffer<Sample>` directly, moves it into mal, and later
borrows the same managed value for inspection. `Sample` uses the runtime carrier layout emitted in
`program.mal.h`; the extension does not exchange a separate address or canonical byte encoding.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/extern-runtime/program.mal --output /tmp/mal-extern-runtime
/tmp/mal-extern-runtime
```

Success is silent and exits with status 0.
