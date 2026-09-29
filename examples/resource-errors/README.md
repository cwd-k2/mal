# Resource errors

This program copies one file to standard output while keeping expected host failures in sum values.
The C host owns allocation and file resources; mal carries their opaque handles, propagates numeric
I/O errors, closes every successfully opened file, and releases the transfer allocation on both
recoverable paths. The language does not enforce ownership, so the extern contract—not the type
system—makes a second release invalid.

The same allocation first stages the path and then becomes the read buffer. The open operation only
borrows the initialized path range, so it is safe to overwrite that storage after the call returns.

> [!NOTE]
> An opaque external handle is still copyable. Cleanup correctness comes from the documented extern
> protocol and control flow, not from affine or linear typing.

A read error takes precedence over a later close error. Allocation and output failures trap because
the example intentionally models only recoverable input operations.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/resource-errors/program.mal --output /tmp/mal-resource-errors
/tmp/mal-resource-errors examples/resource-errors/README.md
```
