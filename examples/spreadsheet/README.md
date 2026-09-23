# Spreadsheet example

This example represents a spreadsheet as finite dimensions and a flat Buffer of cell
rows. A row stores a formula kind, a literal value, and two source coordinates. The type describes
that carrier; `_evaluateAt` interprets source coordinates as dependency edges.

The literal is payload, the formula kind selects how the row is interpreted, and the source
coordinates are relation indicators only for sum and product rows. Row-major position supplies cell
identity; physical adjacency between cells does not imply a dependency.

Cells use row-major coordinates. Literal cells ignore both source fields, sum and product cells read
them as references, and the requested result is computed without materializing a recursive formula
value. `validSheet` establishes the example's invariant before evaluation: dimensions match the row
count, formula kinds are known, and every formula references only an earlier row. The last condition
makes the dependency relation acyclic and keeps recursive evaluation finite.

`setLiteral` mutates the shared cell Buffer and returns the same sheet identity. Coordinates remain
stable because the operation does not reorder or resize rows. Existing aliases observe the changed
literal. Dependent cells are not eagerly rewritten; later evaluation follows the same relation over
the changed values. A transformation that reordered rows would need to remap every stored coordinate.

The representation is intentionally simple rather than a complete spreadsheet system. A larger
implementation could store values and dependency edges in separate column-oriented carriers without
changing the logical contracts of validation and evaluation.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/spreadsheet/program.mal --output /tmp/mal-spreadsheet
/tmp/mal-spreadsheet
```

The executable produces no output and exits with status 0 after validation, initial evaluation, and
recalculation through the updated carrier all succeed.
