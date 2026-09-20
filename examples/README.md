# Examples

The examples are executable explanations of language and host-boundary design. Each directory has a
README that states which side owns a value or resource, what its aliases actually guarantee, and why
the program chose its representation. Source comments explain contracts and non-obvious decisions;
they do not repeat the syntax.

Read every alias in two steps: first as its structural representation, then as the role assigned by
the operations that consume it. A role name is documentation, not a nominal proof. The examples make
that distinction visible in different settings:

An alias used by an extern signature or canonical-memory helper also contributes names to the
generated C interface. Renaming such an alias therefore requires the checked-in header and host
adapter to change together, even though mal compares the underlying types structurally.

| Example | Physical carrier or boundary | Logical interpretation owned by operations |
| --- | --- | --- |
| `relation-views` | one `Packed<PairRow>` | directed edges or half-open intervals |
| `csr-dijkstra` | offset, neighbor, and cost columns | validated weighted CSR and shortest paths |
| `spreadsheet` | row-major `Packed<CellRow>` | acyclic formula dependencies |
| `packed-tree` | `Packed<NodeRow>` | root and child-coordinate relations |
| `fallible-tree` | host node addresses | recoverably constructed and released subtrees |
| `json-query` | owned input bytes and frame bytes | JSON tokens and parser continuations |
| `mini-database` | borrowed fixed-layout file bytes | validated header, slots, keys, and values |
| `brainfuck-llvm` | owned source bytes and generated `Symbol` | bracket nesting and LLVM control flow |
| `typed-memory` | scoped external `Region<SampleRecord>` | aligned or unaligned canonical products |
| `symbol-round-trip` | host scratch bytes and owned `Symbol` | admission, concatenation, and transfer |
| `socket-packet` | external packet buffer and sockets | packet wire framing and endpoint lifetime |
| `recoverable-file` | file handles and external byte buffer | explicit I/O failure and cleanup paths |
| `resizable-buffer` | allocation handle and address descriptors | growth, initialized prefix, and stale borrows |
| `opaque-aggregate` | opaque handle inside product and sum ABI values | structural host round trip |
| `print-and-closure` | scalar ABI values and a closure environment | ordered effects and captured addition |
| `tail-recursion` | scalar accumulator state | bounded-stack recursive control |
| `generic-loop` | explicit generic state and a step closure | iteration by continue-or-break sum results |
| `numeric-conversion` | fixed-width scalar values | wrapping and modulo conversion |
| `strict-float` | binary floating-point scalars | exact rounding and host-ABI bit preservation |

For a progression through representation and relations, read:

1. [`relation-views`](relation-views/) keeps one pair-row carrier and gives it two independent logical
   interpretations.
2. [`csr-dijkstra`](csr-dijkstra/) separates CSR relation indicators from edge payload and mutable
   algorithm state.
3. [`spreadsheet`](spreadsheet/) treats flat cell rows as a dependency graph, validates the relation,
   and evaluates formulas without a recursive value type.
4. [`packed-tree`](packed-tree/) constructs an indexed tree in mal-owned storage and preserves its
   carrier-relative coordinates across a value-only edit.
5. [`json-query`](json-query/) replaces a recursive syntax tree and mutually recursive parser control
   with input and frame carriers plus one transition operation.
6. [`mini-database`](mini-database/) interprets borrowed bytes as a validated persistent schema and
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
[`tail-recursion`](tail-recursion/), [`generic-loop`](generic-loop/),
[`numeric-conversion`](numeric-conversion/),
[`strict-float`](strict-float/), [`opaque-aggregate`](opaque-aggregate/), and the larger
[`brainfuck-llvm`](brainfuck-llvm/) compiler.

The control-flow examples separate repetition policy from domain work at different scales.
`generic-loop` derives named range, repetition, search, and fold operations from a continue-or-break
sum. `json-query` uses the same shape for a whole parser state machine. `resizable-buffer` evaluates a
loop only as one local expression inside a larger allocation protocol. `relation-views` shares
`any`, `all`, and `fold` carrier traversal while leaving edge and interval meaning in domain callbacks.

The examples choose a control form by the information that must cross an iteration edge:

| Control form | Examples | Reason for the form |
| --- | --- | --- |
| Named operations derived from a generic loop | `generic-loop`, `relation-views`, immutable validation in `csr-dijkstra` | Cursor movement and termination are reusable, while callbacks need only ordinary values. |
| A local generic loop expression | `resizable-buffer` | Repetition is one phase inside a larger operation and produces a value consumed afterward. |
| A domain state machine driven by a generic loop | `json-query` | Each transition replaces the complete parser state or returns the parse result. |
| Named direct tail recursion | `mini-database`, `recoverable-file`, `socket-packet`, Brainfuck file I/O, mutable Dijkstra passes | Progress, failure, or scoped authority belongs to the operation itself; a callback would hide context or add no shared policy. |
| Structural recursion with work after a child returns | `packed-tree`, `spreadsheet`, `fallible-tree`, nested Brainfuck compilation, decimal rendering | The recursive shape or return order is part of the domain computation rather than a plain iteration edge. |
| No recursive control | The remaining focused ABI and scalar examples | Their teaching goal is a boundary or value rule, so adding an iteration abstraction would be unrelated noise. |

In particular, a scoped `Buffer` or `Region` must be passed directly to a helper rather than captured
by an iteration callback. This makes the authority boundary one of the criteria for retaining named
direct recursion; surface uniformity is not a reason to obscure it.

All `.mal` files are formatter fixtures. Representative directories also build and execute through the
public compiler driver in `compiler/tests/driver/examples.rs`.
