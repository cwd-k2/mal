# Generic map

This module implements a fixed-capacity hash map with open addressing. `HashMap<K, V>` is opaque
outside `map.mal`, so callers can use its constructor, lookup, and insertion operations without
access to the slot carrier. The undeclared bodies of `hash<K>` and `equal<K>` are requirements;
specialization selects the exact `Symbol` implementations supplied by `program.mal`.
`SymbolMap` partially applies the map constructor, then passes that unary constructor through the
higher-kinded `preserve<F, A>` function.

> [!NOTE]
> `opaque` hides representation across files; it does not make a value linear or unique. A map can
> still be copied as an alias to its underlying managed Buffer.

The executable covers collisions, replacement, missing lookup, a full map, and zero capacity.
Deletion and resizing are omitted because they would obscure the generic-operation boundary.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/generic-map/program.mal --output /tmp/mal-generic-map
/tmp/mal-generic-map
```

Success is silent and exits with status 0.
