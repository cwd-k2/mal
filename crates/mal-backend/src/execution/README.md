# execution

The target-layout-independent execution plan derived from the closure-converted and control-lowered program.
The LLVM emitter reads the plan; it does not re-derive call targets, regions, frame contents, or owner
responsibilities. The derivation order and its rules are documented in
`docs/implementation/application-control-lowering.md` and `docs/implementation/ownership.md`.

| Module | Responsibility |
|---|---|
| `closure` | which closures each value may be, and the application targets that follow statically |
| `environment_alias` | values that may borrow from the active closure environment |
| `application` | the possible targets of every application site |
| `optimization/*` | one applicability rule per technique, yielding decisions only |
| `pass_through` | the correspondence between a parameter pattern and its argument |
| `continuation` | continuation edges left after selected elisions |
| `region` | the recursive partition of the residual graph into control regions |
| `call` | the call mode of each site |
| `parameter` | whether each function parameter is bound or discarded |
| `self_tail_parameter` | the parameters a direct self-tail edge may rebind in place |
| `frame` | typed suspension frames |
| `frame/resume` | the pairing of returns with frames |
| `frame/replacement` | retired frame capacity that a later frame can reuse |
| `native_recursion` | the self-recursive functions that also get a native version, and the worker ABI they use |
| `derived` | managed values that may share their lifetime with a root value |
| [`ownership`](ownership/README.md) | the managed responsibility plan and its exact validator |

Every materialized plan can rebuild its expected content from its authority and is checked by an exact
validator in debug builds and focused tests.
