# Socket packet example

This example sends one packet through a Linux `AF_UNIX` `socketpair`. It exercises real kernel socket I/O
without requiring a listening port, external server, DNS, or network availability, so the example
and its test remain deterministic.

The source-level `Packet` is `(sequence, Symbol)`. The host adapter defines a wire frame containing
an eight-byte big-endian sequence, an eight-byte big-endian payload length, and the payload bytes.
This frame is the operation-specific socket contract, not a canonical memory representation of the
`Packet` product.
The adapter completes partial `send` and `recv` operations and reports recoverable failures with
errno-compatible result variants. Payloads are limited to 256 bytes so reception can use temporary
stack storage before admitting a new mal-controlled `Symbol`. The program first verifies that a
257-byte packet is rejected without writing a partial frame, then sends and receives a valid packet.

`host.c` consistently uses the generated interface vocabulary: `MAL_DEFINE` for adapter entries,
`MAL_TYPE` for source-level types, `MAL_OPERATION` for constructors, projections, opaque bits, and
Symbol access, and `MAL_MOVE` when an owned admitted Symbol is consumed by a result constructor.
The borrowed packet and Symbol passed to `sendPacket` are observed only during the call, so cloning
them would be unnecessary and would teach the wrong ownership rule.

The `Socket` handle and operating-system socket lifetime remain under Extern authority. `Packet` and
the received `Symbol` are Engrams; the adapter validates the frame and asks the runtime to admit the
payload before returning it. Copying a `Socket` does not duplicate its file descriptor or make
multiple calls to `closeSocket` valid.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/socket-packet/program.mal --output /tmp/mal-socket-packet
/tmp/mal-socket-packet
```

The executable produces no output and exits with status 0.
