# Mini database example

This example implements a persistent fixed-capacity key-value database. The mal program owns the
binary layout, validation, query parser, lookup, and updates. Its C adapter supplies only allocation,
file transfer, line input, and byte output.

Standard input is read into one reusable `Ptr` buffer, so processing more queries does not retain one
new Symbol for every input line.

The query language is:

```text
put KEY VALUE
get KEY
del KEY
quit
```

Keys are 1–16 bytes, values are 0–40 bytes, and the database has 64 slots. Keys cannot contain a
space; values may. An empty input line and end-of-file also stop the interpreter.

The binary file is 4,112 bytes: a 16-byte `MALD` header followed by 64 records of 64 bytes. Every
field is defined as raw bytes, so the format does not depend on host integer byte order. The database
path is the program's single command-line argument.

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
