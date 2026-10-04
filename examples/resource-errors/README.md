# Resource errors

This program copies one file to standard output while keeping expected host failures in sum values.
The C host borrows the path as a `Symbol`, owns file resources, and returns each successful read as a
mal-owned `Buffer<UInt8>`. mal carries opaque file handles, propagates numeric I/O errors, and closes
every successfully opened file. The language does not enforce resource ownership, so the extern
contract—not the type system—makes a second close invalid.

> [!NOTE]
> An opaque external handle is still copyable, while a returned Buffer responsibility is moved into
> mal. Cleanup correctness comes from the documented extern protocol and control flow, not from
> affine or linear typing.

A read error takes precedence over a later close error. Allocation and output failures trap because
the example intentionally models only recoverable input operations.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/resource-errors/program.mal --output /tmp/mal-resource-errors
/tmp/mal-resource-errors examples/resource-errors/README.md
```
