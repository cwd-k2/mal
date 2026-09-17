# Mini database example

This example implements a persistent fixed-capacity key-value database. The mal program owns the
binary layout, validation, query parser, lookup, updates, complete-file transfer, and line framing.
Its C adapter supplies only allocation, thin file operations, and byte output.

The source is split by authority and operation rather than kept in one application module:

- `host.mal` defines positional boundary descriptors and the extern contract.
- `bytes.mal` owns `Region<UInt8>`/`Packed<UInt8>` byte search, admission, and transfer.
- `input.mal` owns buffered standard-input and line framing.
- `database.mal` owns the persistent binary layout, validation, lookup, and mutation.
- `query.mal` parses commands, maps database results to responses, and decides when to persist.
- `application.mal` acquires and releases resources and connects the other responsibilities.
- `program.mal` admits the process argument and owns the entry point.

Each file directly requires the public names it uses; required names are not re-exported. `host.mal`
requires `host.c`, so building the root source still discovers the adapter transitively. Private
helpers use leading `_` names.

The shared-memory interface uses three representations with separate responsibilities:

- `Allocator` is an opaque C-owned arena handle. All allocations remain live until the arena is
  destroyed.
- `AllocatedBytes` is the host-mappable `(Address, capacity)` descriptor returned by the C allocator.
  Mal immediately places it as a `Region<UInt8>`; `ByteBuffer` pairs that region with its initialized
  prefix length and is updated immutably by mal code.
- `WritableBytes` and `ReadableBytes` are directional `(Address, USize)` descriptors used only at the
  host boundary.
- Each completed query line is admitted from `Region<UInt8>` into `Packed<UInt8>` before parsing.
  Search, slicing, and database copies therefore use element counts rather than raw byte arithmetic.
- `Reader` combines a `File`, an input `ByteBuffer`, and a cursor. Sum values return either end-of-file or
  a byte together with the next immutable reader state.

These distinctions document intent but do not add ownership or bounds enforcement to the language.
Aliases remain structural, opaque handles remain copyable, and the host contract determines the
lifetime of every `Address`.

Comments beside positional product and sum aliases name each field or variant. The comments are part
of the example's protocol documentation: transparent aliases do not create named fields or nominal
variants in the language.

Standard input is transferred into one reusable 4 KiB region. The mal program carries unread input
between calls, detects line endings and overlong lines, and copies the current line into a second
reusable buffer. Each line becomes a temporary owned `Packed<UInt8>` and is released after the query;
processing more queries does not retain one new value for every input line.

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
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/mini-database/program.mal --output /tmp/mal-mini-database
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

`createAllocator` returns an empty arena. `allocateBuffer` returns a nonempty writable buffer whose
storage remains live until `destroyAllocator`; destroying the arena invalidates every buffer it
returned. The mal program must destroy each allocator exactly once and must not use its buffers
afterward.

`readFile` writes at most `capacity` bytes into the supplied live writable region, advances the file
position, and returns the transferred length. Zero means end-of-file when `capacity` is nonzero.
`writeFile` reads at most `length` bytes from the supplied live region, advances the position, and
returns the transferred length. The mal program handles short transfers. `rewindFile` returns the
position to the beginning, `flushFile` makes buffered output visible to the underlying file, and
`closeFile` invalidates the handle.

Allocation, path conversion, file operations, and output trap on unrecoverable host failure. The
adapter does not provide recoverable I/O errors or automatic resource cleanup.

`outputBuffer` returns the same writable scratch buffer on each call with a nonzero capacity. A
`writeStdout` or `writeStderr` call synchronously consumes exactly the supplied prefix and does not
retain its address. Mal splits longer `Symbol` values into capacity-sized `Packed<UInt8>` chunks,
stores each chunk into the buffer region, and writes it before reusing the storage.
