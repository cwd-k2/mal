# Canonical memory

The program copies product values between a managed Buffer and deliberately unaligned external
storage. The C host uses generated `mal_SampleRecord_read` and `mal_SampleRecord_write` helpers, so it
does not assume native struct alignment or layout. `Address` remains opaque to mal; only `from<T>` and
`buffer.into` admit and observe typed canonical memory.

> [!NOTE]
> `Address` carries external authority, not a mal element type. The type argument on `from<T>` and
> the Buffer element type choose the canonical encoding used for a particular copy.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/canonical-memory/program.mal --output /tmp/mal-canonical-memory
/tmp/mal-canonical-memory
```

Success is silent and exits with status 0.
