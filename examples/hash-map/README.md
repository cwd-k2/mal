# Exact-family HashMap example

This example implements a fixed-capacity HashMap with open addressing and linear probing. A slot is
either empty or contains one key-value pair, so construction does not require default key or value
operations. `hashMapGet` and `hashMapPut` acquire `hash<K>` and `equal<K>` requirements from their
bodies, and specialization selects the exact `Symbol` implementations in `program.mal`.

The representation is a transparent `Buffer` alias. It does not establish a nominal invariant or
hide mutation from aliases. The public operations preserve the slot invariant, stop lookup at an
empty slot, overwrite an existing key, and scan at most the capacity when collisions wrap around.
Insertion returns `false` for a full or zero-capacity map. Resizing and deletion are deliberately
outside this example.

The keys `"a"`, `"e"`, and `"i"` collide modulo four under the example hash. The executable checks
collision traversal, overwrite, missing lookup, full capacity, and zero capacity.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/hash-map/program.mal --output /tmp/mal-hash-map
/tmp/mal-hash-map
```

The executable prints nothing and exits with status 0.
