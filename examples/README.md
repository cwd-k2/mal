# Examples

The examples are executable explanations of current language and host-boundary behavior. Each
directory documents the representation, mutation, and resource assumptions relevant to its program.
Source comments explain contracts and non-obvious decisions rather than restating syntax.

Run compiler commands from the repository root through the pinned environment. The compiler usage
contract, including the distinction between `check`, `build`, and `emit`, lives in
[`docs/development/compiler-usage.md`](../docs/development/compiler-usage.md); individual example
README files give only the command needed for that example.

An example does not treat its carrier as the domain meaning itself. Indexed examples place physical
carriers behind file-local opaque types, then expose operations and validators that interpret
coordinates, tags, or byte offsets. Host examples keep resource meaning in extern contracts; scalar
examples make language rules observable through a narrow host operation. Transparent aliases remain
for interchangeable vocabulary and public result shapes, not abstraction boundaries.

| Example | Focus |
| --- | --- |
| `relation-views` | one Buffer interpreted as directed edges and half-open intervals |
| `csr-dijkstra` | validated weighted CSR columns and mutable shortest-path workspace |
| `spreadsheet` | row-major cells interpreted as acyclic formula dependencies |
| `buffer-tree` | shared mutable rows interpreted through child coordinates |
| `fallible-tree` | recoverable construction in host-owned node allocations |
| `json-query` | Buffer cursors and an explicit parser-frame stack |
| `mini-database` | a managed database image copied to and from host file storage |
| `brainfuck-llvm` | Buffer source bytes compiled into textual LLVM IR |
| `typed-memory` | canonical products copied through deliberately unaligned host storage |
| `symbol-round-trip` | independent Symbol snapshots and a host transfer area |
| `socket-packet` | packet framing across sockets and a bounded transfer area |
| `recoverable-file` | explicit external I/O failure and cleanup paths |
| `resizable-buffer` | automatic Buffer growth and mutation observed through aliases |
| `buffer-handles` | Cell and View handles over a Buffer, with a runtime read-only flag |
| `generic-loop` | iteration derived from a continue-or-break sum result |
| `operation-family` | exact and structural generic type-indexed implementations |
| `hash-map` | opaque fixed-capacity linear probing driven by hash and equality families |
| `tail-recursion` | bounded-stack recursive control |
| `print-and-closure` | ordered external effects and captured values |
| `opaque-aggregate` | opaque handles inside product and sum ABI values |
| `numeric-conversion` | fixed-width wrapping and modulo conversion |
| `strict-float` | exact floating-point rounding and host-ABI bit preservation |

For representation and relation modeling, start with `relation-views`, continue through
`csr-dijkstra` and `spreadsheet`, then compare the mal-owned `buffer-tree` with the host-owned
`fallible-tree`. For the C host boundary, `typed-memory` gives the smallest `from<T>`/`buffer.into`
example; `mini-database`, `socket-packet`, and `brainfuck-llvm` show the same boundary in larger
programs.

All `.mal` files are formatter fixtures. A file that owns an external C interface has its
compiler-generated `.mal.h` checked in together with the dependency-header closure needed to include
it standalone. Tests reject both missing and unnecessary checked-in headers, compare every retained
header byte-for-byte with current compiler output, and compile it as C11. This lets clangd and standalone
C compilation resolve both quoted includes and the repository's pregenerated `mal.h`. Representative
directories also build and execute through the public compiler driver tests.
