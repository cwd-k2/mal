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
- Prefer behavior-oriented tests at compiler-stage boundaries over tests coupled to internal data
  structures.
- Keep deterministic language semantics independent of incidental host Rust or C behavior, especially
  evaluation order, integer overflow, traps, floating-point conversion, and generated ABI details.

## Development and verification

Enter the pinned development environment from the repository root with `nix develop`. Run compiler
commands from `compiler/`, or pass `--manifest-path compiler/Cargo.toml` from the root.

Before considering a compiler change complete, run:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run `nix flake check` when changing the Nix development environment or flake inputs. Add focused
positive and negative tests for every newly accepted or rejected language behavior.
