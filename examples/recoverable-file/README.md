# Recoverable file example

This example copies one file to standard output while representing expected host failures as sum
results rather than traps. It exercises explicit error propagation and cleanup with C-owned shared
memory.

The host returns an `OwnedBuffer` containing an opaque `Allocation` handle and a mal-visible
`ByteBuffer`. `releaseBuffer` consumes the program's logical ownership of the handle after either open
failure or a completed read/close sequence. The language does not enforce this ownership: the handle
remains copyable, and calling `releaseBuffer` twice would violate the host contract.

`OpenResult`, `ReadResult`, and `CloseResult` carry a numeric `IoError`. The mal program converts each
operation-specific result into `CopyResult`, preserves a read error over a later close error, closes every
successfully opened file, and releases the buffer along both recoverable paths. Allocation failure and
standard-output failure still trap because this example does not attempt to recover from them.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/recoverable-file/program.mal --output /tmp/mal-recoverable-file
/tmp/mal-recoverable-file examples/recoverable-file/README.md
```

The result aliases document distinct operations but remain structurally typed. In particular,
`CloseResult` and `CopyResult` have the same underlying type and are interchangeable to the type checker.
