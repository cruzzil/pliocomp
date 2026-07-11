# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`pliocomp` is a small Rust library crate implementing PLIO (Pixel List I/O) mask compression, as used in FITS tile compression. It is a port of the IRAF PLIO routines (`plp2l.gx` / `pll2p.gx`) and the cleaned-up `pliocomp.c` from CFITSIO, with most `goto`s removed and magic constants named. See [README.md](README.md) for the upstream references and the algorithm spec (section 6.3 of the FITS tile-compression standard).

## Commands

```bash
cargo build
cargo test                       # runs the round-trip unit tests in src/lib.rs
cargo test it_works              # run a single test by name
cargo fmt --all -- --check       # CI enforces formatting
cargo clippy -- -D warnings      # CI treats all clippy warnings as errors
```

CI ([.github/workflows/rust.yml](.github/workflows/rust.yml)) runs build/test/fmt/clippy on stable, beta, and nightly. `fmt --check` and `clippy -D warnings` are gating — run both before considering work done.

### Fuzzing

The [fuzz/](fuzz/) crate is a separate cargo-fuzz workspace (not part of the main build). It requires nightly + `cargo-fuzz`:

```bash
cargo +nightly fuzz run fuzz_target_1
```

The fuzz target ([fuzz/fuzz_targets/fuzz_target_1.rs](fuzz/fuzz_targets/fuzz_target_1.rs)) compresses random pixel data with `pl_p2li`, decompresses with `pl_l2pi`, and asserts the round-trip is lossless. Its `Arbitrary`-derived `Data` input type and the `arbitrary` dependency live entirely in the fuzz crate — the core library exposes neither. Because PLIO only represents non-negative values up to 32767, the harness maps arbitrary input into that domain before checking the round-trip (see [ALGORITHM.md](ALGORITHM.md#limitations)).

## Architecture

The entire library lives in [src/lib.rs](src/lib.rs) (~380 lines). Two public functions form a lossless codec pair:

- `pl_p2li(pxsrc, xs, lldst, npix) -> usize` — **encode**: pixel array → encoded line list. Returns the list length.
- `pl_l2pi(ll_src, xs, px_dst, npix) -> usize` — **decode**: encoded line list → pixel array. Returns the pixel count.

These are inverses; the tests and fuzz target both assert `decode(encode(x)) == x`.

### Key concepts when editing

- **The `I_*` / `M_*` / `LL_*` / `OLL_*` constants encode the on-disk format.** `I_ZN`, `I_HN`, `I_PN`, `I_SH`, `I_IH`, `I_DH`, `I_IS`, `I_DS` are opcodes for the run-length/delta instruction stream; `M_*` are their bit-mask forms (e.g. `M_SH = 4096`). The `LL_*` and `OLL_*` indices are fixed offsets into the encoded line-list header. These mirror the IRAF `plio.h` layout — do not change their values without changing the format. Note the upstream `'B'`-suffixed constants in `plio.h` are **octal**.
- **This is a direct port of C/SPP code**, so the style is deliberately imperative and low-level (manual index bookkeeping, `i16`/`i32` widths matching the original words). Commented-out lines like `//--lldst;` are leftover pointer-adjustment artifacts from the C source. Preserve behavior exactly; prefer readability changes that don't alter output over algorithmic rewrites.
- **`xs` is the starting index into the pixel array**; `npix` the pixel count. Callers pass a pre-sized output buffer — the returned length says how much was actually written.
- The core crate has **no dependencies and no features**; all fuzzing scaffolding (the `Data` type, `arbitrary`) is confined to the fuzz crate.
