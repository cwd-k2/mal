# Socket packet example

This example sends one packet through a Linux `AF_UNIX` `socketpair`. It exercises real kernel socket I/O
without requiring a listening port, external server, DNS, or network availability, so the example
and its test remain deterministic.

The mal-internal `Packet` is `(sequence, Symbol)`. The host adapter defines a wire frame containing
an eight-byte big-endian sequence, an eight-byte big-endian payload length, and the payload bytes.
This frame is the operation-specific socket contract, not a canonical memory representation of the
`Packet` product.
The adapter completes partial socket `write` and `read` operations and reports recoverable failures with
errno-compatible result variants. Payloads are limited to 256 bytes. Mal copies outgoing Buffer bytes
into the external packet buffer and snapshots the initialized receive prefix with `from<UInt8>`.
The program first verifies that a 257-byte packet is rejected without writing a partial frame, then
sends and receives a valid packet.

`host.c` uses `MAL_DEFINE_<operation>` entries, Address/USize byte descriptors, and variant-specific
terminal returns. The host never observes a Symbol or a mal-managed owner.

The `Socket` handle, packet buffer, and operating-system socket lifetime remain under Extern authority.
The outgoing `Packet` is an Engram; the adapter validates the received frame and reports its initialized
prefix before mal borrows it. Copying a `Socket` does not duplicate its file descriptor or make multiple
calls to `closeSocket` valid.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/socket-packet/program.mal --output /tmp/mal-socket-packet
/tmp/mal-socket-packet
```

The executable produces no output and exits with status 0.
