# Socket packet example

This example sends one packet through a Linux `AF_UNIX` `socketpair`. It exercises real kernel socket I/O
without requiring a listening port, external server, DNS, or network availability, so the example
and its test remain deterministic.

The source-level `Packet` is `(sequence, Symbol)`. The host adapter defines a wire frame containing
an eight-byte big-endian sequence, an eight-byte big-endian payload length, and the payload bytes.
This frame is the operation-specific socket contract, not a canonical memory representation of the
`Packet` product.
The adapter completes partial socket `write` and `read` operations and reports recoverable failures with
errno-compatible result variants. Payloads are limited to 256 bytes so reception can use temporary
stack storage; the typed terminal return copies the received bytes into a mal-controlled `Symbol` before the body ends. The program first verifies that a
257-byte packet is rejected without writing a partial frame, then sends and receives a valid packet.

`host.c` uses `MAL_DEFINE_<operation>` entries, typed host values, `mal_Symbol_to_bytes` for
call-scoped observation, and variant-specific terminal returns. The borrowed packet and Symbol passed to
`sendPacket` are observed only during the call.

The `Socket` handle and operating-system socket lifetime remain under Extern authority. `Packet` and
the received `Symbol` are Engrams; the adapter validates the frame and terminal return copies the
payload before returning it. Copying a `Socket` does not duplicate its file descriptor or make
multiple calls to `closeSocket` valid.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/socket-packet/program.mal --output /tmp/mal-socket-packet
/tmp/mal-socket-packet
```

The executable produces no output and exits with status 0.
