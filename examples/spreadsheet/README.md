# Spreadsheet example

This example represents a spreadsheet as finite dimensions and a flat immutable sequence of cell
rows. A row stores a formula kind, a literal value, and two source coordinates. The type describes
that carrier; `_evaluateAt` interprets source coordinates as dependency edges.

Cells use row-major coordinates. Literal cells ignore both source fields, sum and product cells read
them as references, and the requested result is computed without materializing a recursive formula
value. `validSheet` establishes the example's invariant before evaluation: dimensions match the row
count, formula kinds are known, and every formula references only an earlier row. The last condition
makes the dependency relation acyclic and keeps recursive evaluation finite.

`setLiteral` derives a new carrier with `edit`. Coordinates remain stable because this transformation
does not reorder or resize the rows. Dependent cells are not eagerly rewritten; evaluating the new
sheet follows the same relation over the changed values. A transformation that sorted, sliced, or
compacted rows would instead need to remap every stored coordinate.

The representation is intentionally simple rather than a complete spreadsheet system. A larger
implementation could store values and dependency edges in separate column-oriented carriers without
changing the logical contracts of validation and evaluation.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/spreadsheet/program.mal --output /tmp/mal-spreadsheet
/tmp/mal-spreadsheet
```

