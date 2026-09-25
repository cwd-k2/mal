# Examples

The examples are executable explanations of current language and host-boundary behavior. Each
directory documents the representation, mutation, and resource assumptions relevant to its program.
Source comments explain contracts and non-obvious decisions rather than restating syntax.

An example does not treat its carrier as the domain meaning itself. Indexed examples name the
operations and validators that interpret coordinates, tags, or byte offsets; host examples keep
resource meaning in extern contracts; scalar examples make language rules observable through a
narrow host operation. Transparent aliases improve vocabulary but never stand in for validation or
nominal proof.

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

All `.mal` files are formatter fixtures. Representative directories also build and execute through
the public compiler driver tests.
