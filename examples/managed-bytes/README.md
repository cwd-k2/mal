# Managed bytes

The host creates and returns a mal-owned `Buffer<UInt8>` directly through `extern`. Assignment
creates an alias to the same growable mutable identity. Converting
the Buffer to `Symbol` creates an immutable snapshot, so later mutation and append through the alias
do not alter the earlier text value. The final bytes cross back as a borrowed `Symbol`; the host may
inspect its carrier during the call without taking over mal's reference.

> [!NOTE]
> Assigning a Buffer copies its managed identity, while converting it to Symbol copies its current
> bytes. Those two operations deliberately have different aliasing behavior.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/managed-bytes/program.mal --output /tmp/mal-managed-bytes
/tmp/mal-managed-bytes
```

The host validates the bytes and prints `9 bytes`.
