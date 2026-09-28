# Opaque HashMap example

This example implements a fixed-capacity HashMap with open addressing and linear probing. A slot is
either empty or contains one key-value pair, so construction does not require default key or value
operations. `hashMapGet` and `hashMapPut` acquire `hash<K>` and `equal<K>` requirements from their
bodies, and specialization selects the exact `Symbol` implementations in `program.mal`.

`HashMap<K, V>` is file-local opaque over its slot Buffer. Requiring the module exposes the map
identity and public operations without exposing construction or raw slot mutation. Those operations
preserve the slot invariant, stop lookup at an empty slot, overwrite an existing key, and scan at
most the capacity when collisions wrap around.
Insertion returns `false` for a full or zero-capacity map. Resizing and deletion are deliberately
outside this example.

The keys `"a"`, `"e"`, and `"i"` collide modulo four under the example hash. The executable checks
collision traversal, overwrite, missing lookup, full capacity, and zero capacity.
Its lookup assertions use `const<A, B> :: A -> (B -> A)` as a partially applied constant
continuation; the continuation's expected parameter type supplies the otherwise unknown `B`.
The probing functions use `identity<A> :: A -> A` to unwrap an occupied slot before applying the
remaining lookup or insertion logic, while an empty-slot result exits through the enclosing result
binder.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/hash-map/program.mal --output /tmp/mal-hash-map
/tmp/mal-hash-map
```

The executable prints nothing and exits with status 0.
