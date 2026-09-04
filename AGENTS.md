# Repository guidance

## Documentation policy

Minimize reading load while preserving the information needed for present and future decisions.

Use one language consistently within each documentation area:

- Keep `AGENTS.md` and all `README.md` files in English.
- Write files under `docs/` in Japanese.
- Keep commands, option names, identifiers, language syntax, product names, and other technical
  terms in their conventional form instead of translating them solely to satisfy the language
  boundary.

Each documentation area has one responsibility:

| Location | Contents |
| --- | --- |
| `README.md` files | Entry points, component scope, and common operating commands |
| `docs/spec/` | The current mal language and host-interface specification |
| `docs/design/` | Cross-cutting rationale, decisions, and genuinely unresolved questions |
| `docs/implementation/` | Compiler architecture, milestone order, and acceptance criteria |
| `docs/releases/` | Version scope and release-level changes |
| `docs/research/` | Prior art and research notes that inform design decisions |

Do not duplicate facts available from manifests, declarations, or a more authoritative document.
Omit defaults and transient runtime values unless they are needed to reproduce behavior or avoid an
ownership mistake. Remove completed work, stale status text, pending-commit notes, and contradictory
guidance in the same change that makes them obsolete.

Keep source-code comments in English and do not repeat what syntax or identifiers already show:

- A file comment states only file-wide purpose, ownership, or constraints.
- A function comment states its caller-facing contract, side effects, or non-obvious failure modes.
- An inline comment explains a local reason, invariant, or workaround.

Put cross-file design, operating procedures, and chronology in the responsible document. Git is the
implementation-level chronology; documentation records current design and decisions, not a work
transcript.

## Specification and implementation policy

- Treat `docs/spec/` as the source of truth for language behavior. If implementation work exposes a
  missing or conflicting rule, resolve or record the specification issue instead of silently choosing
  behavior in code.
- Follow the active milestone and work order in `docs/implementation/m0.md`. Add modules when their
  behavior is needed; do not create empty future-facing modules or a general framework in advance.
- Keep compiler stages separated along the pipeline documented in
  `docs/implementation/compiler.md`. Do not let CLI, filesystem, process, or C toolchain concerns leak
  into the core language passes.
- Reject unsupported or malformed syntax explicitly. Never accept an incomplete language construct
  merely because its later compiler stage is not implemented yet.
- Preserve source spans through compiler stages whenever they are needed to report a user-facing
  error.

## Rust compiler policy

- Keep the compiler dependency-free unless an external crate makes the implementation materially
  simpler and its cost is justified.
- Preserve `#![forbid(unsafe_code)]` in compiler crate roots.
- Keep deterministic language semantics independent of incidental host Rust or C behavior, especially
  evaluation order, integer overflow, traps, floating-point conversion, and generated ABI details.

## Code responsibilities and boundaries

Classify code by the vocabulary and policy it implements, not by its package, dependency, or whether
it is expressed through an interface. A wrapper does not create a useful boundary when its contract
still exposes the representation or failure vocabulary that should have stopped there.

- Give each module one stable responsibility. Keep a hand-written source file at or below 200 lines
  when a natural responsibility boundary permits it, and split it before 500 lines. Do not create
  numbered or arbitrary fragments to satisfy a line count; split by owned behavior or vocabulary.
- Let each compiler stage own admission and translation from the preceding representation. Its input
  may use the preceding stage's vocabulary; its successful output must use its own validated,
  stage-specific vocabulary.
- Do not use one type with optional fields or flags to represent both unvalidated input and admitted
  output. Distinct semantic states that permit different operations need distinct types.
- Keep raw bytes, paths, OS errors, process status, C toolchain arguments, and other external
  representations in `source`, CLI, or driver boundaries. Translate them into source files,
  diagnostics, or typed compiler outcomes before core passes consume them.
- Place a pure function with the stage whose vocabulary and policy it implements. Purity alone does
  not make a function shared foundation code.
- Keep executable bootstrap limited to input selection, dependency composition, and output delivery.
  Language policy and representation translation belong to their owning stage or boundary.
- Allow only explicit cross-stage concepts, such as source identity and spans, to traverse the
  pipeline. Do not let a later stage reinterpret raw input that an earlier stage was responsible for
  validating.

## Testing policy

- Test at the smallest deterministic boundary that exposes the rule as behavior, not private call
  order or incidental internal structure.
- Give each new rule focused positive and negative coverage. Add a regression test for every fixed
  defect that could recur.
- When a change crosses compiler-stage or external-tool boundaries, test the affected stages directly
  and add one representative cross-boundary path. Keep end-to-end cases few and specification-led.
- Use the real representation converter at a format boundary. In particular, backend contract tests
  compile generated C rather than validating only C-shaped strings.
- Own and clean every temporary file, directory, process, and generated artifact created by a test.

## Development and verification

Enter the pinned development environment from the repository root with `nix develop`. Run compiler
commands from `compiler/`, or pass `--manifest-path compiler/Cargo.toml` from the root.

Before considering a compiler change complete, run:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run `nix flake check` when changing the Nix development environment or flake inputs.
