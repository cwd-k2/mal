# Canonical memory

The program copies product values between a managed Buffer and deliberately unaligned external
storage. The host reports the available element count separately from the opaque address, updates
the second element, and uses generated `mal_Sample_read` and `mal_Sample_write` helpers. It therefore
does not assume native alignment or C struct layout. `from<T>` and `buffer.into` are the only typed
observations of the external bytes on the mal side.

> [!NOTE]
> `Address` carries external authority, not a mal element type. The type argument on `from<T>` and
> the Buffer element type choose the canonical encoding used for a particular copy.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/canonical-memory/program.mal --output /tmp/mal-canonical-memory
/tmp/mal-canonical-memory
```

Success is silent and exits with status 0.
