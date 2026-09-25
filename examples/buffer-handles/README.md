# Buffer handles example

This example derives position handles from a `Buffer` without adding a language feature. `Cell<A>` is
a `Buffer<A>` with an index and `View<A>` is a `Buffer<A>` with an offset and a length. Lifetime
authority stays in the `Buffer`; the numbers carry none.

A number becomes a handle only in `at`, `split`, and `uncons`. `at` and `split` check it against the
view length and return a sum. The count of a `Buffer` never shrinks, so a handle that passed the check
stays in range. `read` and `write` then take only the `Cell`.

Both handle types carry a `Bool` write permission. `freeze` clears it on a view, every handle derived
from that view inherits it, and `write` reports whether the write happened. The permission is a
runtime field because transparent aliases cannot give a read-only handle a distinct type.

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
