# wobblechar

Parse ASCII/UTF-8 waveform diagrams into a stream of decoded samples.

`wobblechar` creates a bridge between the human-friendly world of text waveforms
and the structured data needed for programmatic processing. It was born out of the
need to make testing less tedious and more intuitive.

```text
CLK: _|‾|_|‾|_
DAT: ___‾‾‾___
```

## Getting started

Add to your `Cargo.toml`:

```toml
[dependencies]
wobblechar = "0.1"
```

Then parse a waveform:

```rust
use wobblechar::Builder;

for item in Builder::<2>::new_from_string("
    _|‾|_
    ‾|_|‾
")
    .with_def_bool_mapper()
    .build()
{
    println!("t={} values={:?} changed={}", item.index, item.values, item.changed);
}
```

## Default character mapping

| Character | Meaning |
|-----------|---------|
| `_` | Low (`false` / `0`) |
| `|` | Edge — toggles the previous value |
| anything else (e.g. `‾`, `X`) | High (`true` / `1`) |

Custom mappings can be provided via `Builder::with_const_bool_map`,
`Builder::with_const_num_mapper`, or (with the `std` feature) the
`HashMap`-based variants.

## Multi-line labeled format

Lines can carry a label prefix (`Name: content`) to allow non-contiguous
continuation across blocks. This is useful when interleaving multiple signals:

```text
CLK: _|‾|_|‾|_   # clock
DAT: ___‾‾‾___   # data
# some comment
CLK: ‾‾‾___‾‾‾   # clock continued
DAT: ___‾‾‾___   # data continued
```

## `no_std` support

`wobblechar` is `no_std` by default and works in embedded environments.
Enable the `std` feature for `HashMap`-backed mappers:

```toml
[dependencies]
wobblechar = { version = "0.1", features = ["std"] }
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

---

[Repository](https://github.com/mous-design/wobblechar) · [Documentation](https://docs.rs/wobblechar)
