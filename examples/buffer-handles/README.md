# Buffer handles example

This example derives file-local opaque position handles from a `Buffer`. `Cell<A>` hides a Buffer,
an index, and write permission; `View<A>` hides a Buffer, an offset, a length, and write permission.
Lifetime authority stays in the Buffer; the numbers carry none.

A number becomes a handle only in `at`, `split`, and `uncons`. `at` and `split` check it against the
view length and return a sum. The count of a `Buffer` never shrinks, so a handle that passed the check
stays in range. `read` and `write` then take only the `Cell`.

Both handle types carry a `Bool` write permission. `freeze` clears it on a view, every handle derived
from that view inherits it, and `write` reports whether the write happened. The permission is a
runtime field because freezing changes a value-level permission, while opaque identity prevents
callers from fabricating or destructuring either handle representation.

| Function | Shows |
|---|---|
| `sumFrom` | Traversal with `uncons`, without an index |
| `sumHalves` | Divide and conquer over disjoint halves from `split` |
| `poke` | A write through a writable view and a rejected write through a frozen one |

Build and run from the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/buffer-handles/program.mal --output /tmp/mal-buffer-handles
/tmp/mal-buffer-handles
```

The executable exits with status 38, the sum 19 of `[10, 2, 3, 4]` computed by both traversals.
