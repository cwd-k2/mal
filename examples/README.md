# Examples

The examples are executable explanations of language and host-boundary design. Each directory has a
README that states which side owns a value or resource, what its aliases actually guarantee, and why
the program chose its representation. Source comments explain contracts and non-obvious decisions;
they do not repeat the syntax.

For a progression through representation and relations, read:

1. [`relation-views`](relation-views/) keeps one pair-row carrier and gives it two independent logical
   interpretations.
2. [`spreadsheet`](spreadsheet/) treats flat cell rows as a dependency graph, validates the relation,
   and evaluates formulas without a recursive value type.
3. [`packed-tree`](packed-tree/) constructs an indexed tree in mal-owned storage and preserves its
   carrier-relative coordinates across a value-only edit.
4. [`json-query`](json-query/) replaces a recursive syntax tree and mutually recursive parser control
   with input and frame carriers plus one transition operation.
5. [`mini-database`](mini-database/) interprets borrowed bytes as a validated persistent schema and
   copies only values that must cross the borrow boundary.

The neighboring authority examples show why logical structure and resource policy are separate:

- [`fallible-tree`](fallible-tree/) places a tree relation in external allocations because allocation
  failure must be recoverable.
- [`typed-memory`](typed-memory/) demonstrates scoped typed views over aligned and unaligned external
  storage.
- [`symbol-round-trip`](symbol-round-trip/) separates admission of owned bytes from reuse of a host
  transfer buffer.
- [`recoverable-file`](recoverable-file/) and [`resizable-buffer`](resizable-buffer/) implement explicit
  host-resource protocols that mal's transparent aliases do not enforce.
- [`socket-packet`](socket-packet/) keeps the logical packet distinct from its wire encoding and socket
  lifetime.

The remaining focused examples isolate particular mechanisms: [`print-and-closure`](print-and-closure/),
[`tail-recursion`](tail-recursion/), [`numeric-conversion`](numeric-conversion/),
[`strict-float`](strict-float/), [`opaque-aggregate`](opaque-aggregate/), and the larger
[`brainfuck-llvm`](brainfuck-llvm/) compiler.

All `.mal` files are formatter fixtures. Representative directories also build and execute through the
public compiler driver in `compiler/tests/driver/examples.rs`.

