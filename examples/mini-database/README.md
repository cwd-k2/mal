# Mini database example

This example implements a persistent fixed-size key/value database. The C host owns file handles and
transfer allocations. Mal copies initialized host ranges into `Buffer<UInt8>`, validates and mutates
the managed database image, then uses `buffer.into` before saving it.

The file is 4112 bytes: a 16-byte header followed by 64 fixed-size records. Keys are limited to 16
bytes and values to 40 bytes. Commands on standard input are `put KEY VALUE`, `get KEY`, `del KEY`,
and `quit`; both LF and CRLF lines are accepted.

The byte Buffer alone is not a database. `validateDatabase` interprets header and record coordinates,
checks field bounds and active-record metadata, and only then admits the image to query operations.
Those operations own the key/value interpretation; neither `Buffer<UInt8>` nor the physical byte
offsets establish it by themselves.

Input is read into a reusable host transfer area and copied into a Buffer per chunk. Output Symbols
are converted to byte Buffers and copied to the host's bounded output area in chunks. No mal-owned
Buffer crosses an extern boundary.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/mini-database/program.mal --output /tmp/mal-mini-database
/tmp/mal-mini-database /tmp/example.db
```
