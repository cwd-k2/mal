use super::*;

/// The runtime sources that only a program using Symbol or Buffer links.
const BYTE_RUNTIME: [&str; 8] = [
    "bytes.c",
    "bytes_internal.h",
    "buffer.c",
    "buffer_range.c",
    "buffer_host.c",
    "buffer_symbol.c",
    "buffer_internal.h",
    "symbol.c",
];

#[path = "artifacts/aggregate_ownership.rs"]
mod aggregate_ownership;
#[path = "artifacts/byte_transfer.rs"]
mod byte_transfer;
#[path = "artifacts/calls.rs"]
mod calls;
#[path = "artifacts/closure_ownership.rs"]
mod closure_ownership;
#[path = "artifacts/entry.rs"]
mod entry;
#[path = "artifacts/frames.rs"]
mod frames;
#[path = "artifacts/handoff.rs"]
mod handoff;
#[path = "artifacts/host_values.rs"]
mod host_values;
#[path = "artifacts/interface.rs"]
mod interface;
#[path = "artifacts/symbols.rs"]
mod symbols;
#[path = "artifacts/value_aggregates.rs"]
mod value_aggregates;
#[path = "artifacts/value_scalars.rs"]
mod value_scalars;
#[path = "artifacts/values.rs"]
mod values;
