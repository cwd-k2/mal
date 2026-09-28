# execution

The target-layout-independent execution plan derived from the closure-converted and control-lowered program.
The LLVM emitter reads the plan; it does not re-derive call targets, regions, frame contents, or owner
responsibilities. The derivation order and its rules are documented in
`docs/implementation/application-control-lowering.md` and `docs/implementation/ownership.md`.

| Module | Responsibility |
|---|---|
| `closure` | closure creators, aliases, and statically known application targets |
| `environment_alias` | values that may borrow from the active closure environment: capture reads and what derives from them |
| `application` | possible application graph: caller and possible targets per application site, from the closure flow and the type-compatible functions |
| `optimization/*` | one module per technique (`self_tail`, `tail_forwarder`, `direct_call`, `unique_capture`, `frame_pass_through`); each owns a single applicability rule and yields decisions only; `direct_call` also marks code pointers unused by every possible application |
| `pass_through` | structural correspondence between a parameter pattern and its argument |
| `continuation` | continuation edges left after selected elisions |
| `region` | recursive SCC partition of the residual graph, and site/target membership |
| `call` | call mode per site |
| `parameter` | `Bind` or `Discard` destination for each function parameter |
| `self_tail_parameter` | parameter leaves, persistent lenders, and entry prefixes admitted for direct self-tail edges |
| `frame`, `frame/resume`, `frame/replacement` | typed suspension frames, return/frame pairing, and retired frame capacity that can be reused |
| `native_recursion` | self-recursive region functions that also get a native version, persistent parameter lenders, and invariant/changing leaves for the native worker ABI |
| `derived` | managed values that may share their lifetime with a root value, shared by `environment_alias` and the owned-argument convention |
| `ownership/*` | managed responsibility plan and its exact validator: authority, borrow, liveness, destination, use and drop plans |

Every materialized plan can rebuild its expected content from its authority and is checked by an exact
validator in debug builds and focused tests.
