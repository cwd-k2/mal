# Mini database example

This example implements a persistent fixed-capacity key-value database. The mal program owns the
binary layout, validation, query parser, lookup, updates, complete-file transfer, and line framing.
Its C adapter supplies only allocation, thin file operations, and byte output.

Standard input is read one byte at a time into one reusable `Ptr` buffer. The mal program detects
line endings and overlong lines, so processing more queries does not retain one new Symbol for every
input line.

The query language is:

```text
put KEY VALUE
get KEY
del KEY
quit
```

Keys are 1–16 bytes, values are 0–40 bytes, and the database has 64 slots. Keys cannot contain a
space; values may. An empty input line and end-of-file also stop the interpreter.

The binary file is 4,112 bytes: a 16-byte `MALD` header followed by 64 records of 64 bytes. The mal
program reads up to 4,113 bytes so it can reject an oversized file without asking the host adapter to
apply database policy. Every field is defined as raw bytes, so the format does not depend on host
integer byte order. The database path is the program's single command-line argument.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/mini-database/program.mal --output /tmp/mal-mini-database --link examples/mini-database/host.c
/tmp/mal-mini-database /tmp/mal-mini.db
```

An interactive session looks like:

```text
put language mal
OK
get language
VALUE mal
del language
OK
get language
NOT FOUND
quit
```

Run the executable again with the same path to observe persistence.

## Host contract

`openReadWriteCreate` returns a read/write `File` positioned at the beginning, creating an empty file
only when the path does not exist. `standardInput` returns a borrowed `File` that the mal program must
not close. A `File` is a copyable handle; copying it does not duplicate the stream or extend its
lifetime.

`readFile` writes at most `capacity` bytes into the supplied live writable region, advances the file
position, and returns the transferred length. Zero means end-of-file when `capacity` is nonzero.
`writeFile` reads at most `length` bytes from the supplied live region, advances the position, and
returns the transferred length. The mal program handles short transfers. `rewindFile` returns the
position to the beginning, `flushFile` makes buffered output visible to the underlying file, and
`closeFile` invalidates the handle.

Allocation, path conversion, file operations, and output trap on unrecoverable host failure. The
adapter does not provide recoverable I/O errors or automatic resource cleanup.
