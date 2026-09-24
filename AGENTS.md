# Repository guidance

## Required references

Read the authority relevant to a change before editing:

- [`docs/spec/`](docs/spec/) owns language and host-interface behavior.
- [`docs/implementation/responsibilities.md`](docs/implementation/responsibilities.md) owns compiler
  code responsibilities and translation boundaries.
- [`docs/development/testing.md`](docs/development/testing.md) owns verification policy and commands.

If implementation exposes a missing or conflicting language rule, resolve or record the specification
issue instead of silently choosing behavior in code. Reject unsupported or malformed syntax rather
than accepting a construct whose later stage is not implemented.

## Documentation policy

Minimize reading load while preserving information needed for present and future decisions. Give each
document one responsibility and link to the document that owns a rule instead of duplicating it.

- Keep `AGENTS.md` and `README.md` files outside `docs/` in English.
- Write files under `docs/`, including `docs/README.md`, in Japanese.
- Keep commands, option names, identifiers, language syntax, product names, and other technical terms
  in their conventional form.
- Do not duplicate facts available from manifests, declarations, or a more authoritative document.
- Record module-level responsibilities in the `README.md` of the stage directory that owns them, not in a central
  module list. Stage-level ownership stays in `docs/implementation/responsibilities.md`.
- Remove stale status, contradictory guidance, and obsolete planning text in the change that makes it
  obsolete. Git records implementation chronology; active documentation describes current truth.

Choose prose, lists, and tables by the relationship among the information, not by a preferred visual
style.

- Use prose for a single statement, or when sentence-to-sentence flow carries reasoning,
  qualification, or context.
- Use a list when the content is primarily two or more parallel facts, conditions, steps, or choices
  that readers may need to scan or compare.
- Keep list items parallel and concise. If each item needs its own reasoning, use paragraphs or
  subsections instead of hiding essays inside bullets.
- Do not create a one-item list or add introductory and concluding prose that merely repeats a list.
- Use a table only when readers need to compare the same fields across multiple entries.

Keep source comments in English. Comments document purpose, caller-facing contracts, non-obvious
constraints, invariants, or reasons. Do not narrate syntax or restate identifiers and assertions. Put
cross-file design and unfinished work in the responsible document rather than `TODO` or `FIXME`
comments.

## Repository workflow

Enter the pinned environment with `nix develop`. Preserve `#![forbid(unsafe_code)]` in every crate
root. Keep every crate except `mal-lsp` dependency-free unless a crate makes the implementation
materially simpler and its cost is justified.

Do not create empty future-facing modules or general frameworks in advance. Preserve source spans
through stages that can produce user-facing errors.
