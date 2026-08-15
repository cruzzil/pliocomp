# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0]

### Fixed

- `pl_l2pi` no longer reads past the end of `ll_src`. The word count it decodes
  comes from the line list's own header, so a truncated or corrupt list sent the
  scan beyond the source. It now returns `None` instead — for a source too short
  to hold the header, for a header declaring more words than the source holds,
  and for a two-word `I_SH` whose second word falls off the end.

  This is the bounds check CFITSIO added in 4.7.0, where `pl_l2pi` gained a
  `srclen` argument and `imcomp_decompress_tile` turns a negative return into
  `DATA_DECOMPRESSION_ERR`. There the over-read was undefined behaviour; here
  `ll_src` is a bounds-checked slice, so it was a panic and never a bad read.

### Changed

- **Breaking:** `pl_l2pi` returns `Option<usize>` rather than `usize`, matching
  `pl_p2li`. Existing calls need `.unwrap()`, `.expect(...)` or real handling; a
  list this crate encoded always decodes.

## [0.6.0]

### Fixed

- Corrected the documented worst-case size of an encoded line list from two
  `i16` words per pixel to **three, plus the seven-word header** (`3 * npix + 7`).
  A pixel differing from the running high value by more than `I_DATAMAX` costs a
  two-word `I_SH` pair *and* a one-word `I_HN`. The `npix * 2 + 8` buffer
  [README.md](README.md) recommended is too small from two pixels up, so a
  caller who followed it panicked on high-dynamic-range input.

  This is the same undersizing CFITSIO fixed in
  [PR #174](https://github.com/heasarc/cfitsio/pull/174), where `pl_p2li` wrote
  past a buffer allocated at `nx * sizeof(int)` — a heap overflow for every tile
  from 1 to 300 pixels. Here `lldst` is a bounds-checked slice, so the same bug
  was a panic and never memory corruption. The in-tree tests and fuzz target
  oversized their buffers, which is why fuzzing never surfaced it.

### Added

- `pl_p2li_max_len(npix) -> usize`, the counterpart of CFITSIO's
  `imcomp_calc_max_elem`: the worst-case word count for `npix` pixels. Size an
  encode buffer with it and `pl_p2li` cannot fail.

### Changed

- **Breaking:** `pl_p2li` now returns `Option<usize>` rather than `usize`, and
  returns `None` when `lldst` is too small instead of panicking — the Rust
  equivalent of the `dstlen` parameter and `-1` return added by CFITSIO #174.
  Space is checked incrementally as the list is built, so a buffer smaller than
  `pl_p2li_max_len` still succeeds whenever the data actually fits. Callers
  update with `.unwrap()` (safe when the buffer is `pl_p2li_max_len` words) or
  by handling `None`.
- No emitted word changed; the on-disk format is unaffected, as the
  reference-C tests pin.
- The fuzz target now sizes its buffer at exactly `pl_p2li_max_len(npix)`, so a
  future regression in the bound fails the fuzzer instead of being hidden by
  slack.

### Known issues

- `pl_l2pi` can still panic on a malformed or truncated line list — a list that
  ends mid-`I_SH` makes it read one word past the end. CFITSIO #174 is
  encoder-side only; hardening the decoder against untrusted input is tracked
  separately.

## [0.5.0]

### Fixed

- `pl_p2li`: write `LL_LEN` as the total word count of the line list, not the
  index of its last word. 0.3.0 fixed the *returned* length but left the length
  *stored in the header* one short, and `pl_l2pi` was made to scan inclusively
  to compensate, so encoder and decoder agreed with each other but not with the
  format. Line lists were therefore unreadable by CFITSIO (which drops the
  final instruction) and vice versa. `pl_l2pi` now scans `LL_HDRLEN..LL_LEN`
  exclusively, and the old-format branch starts at `OLL_FIRST` rather than
  `OLL_FIRST - 1`. New tests pin both directions against the reference C in
  [`c_example/`](c_example/), which a round-trip-only suite cannot catch.

### Changed

- Line lists written by 0.3.x and 0.4.x carry an `LL_LEN` one word short and
  need re-encoding; a 0.5.0 decoder drops their final instruction.
- [ALGORITHM.md](ALGORITHM.md) now documents that `LL_LEN` is a count, and that
  zero runs are chunked at `I_DATAMAX - 1` as in IRAF's `plp2l.gx` — CFITSIO's
  f2c'd copy chunks at `I_DATAMAX` and mis-encodes exactly 4095 zeros followed
  by a non-zero pixel.

## [0.4.0]

### Fixed

- `pl_l2pi`: fix corruption of pixel values larger than 12 bits. The `I_SH`
  ("set high value") opcode reconstructed the pixel value with
  `ll_src[i] << 12`, which shifted an `i16` before widening and overflowed
  whenever the high word was `>= 8`. Values above 4095 (e.g. `100000`) now
  round-trip correctly. The high word is now widened to `i32` before the shift.
- `pl_l2pi`: fix the same class of overflow in the line-list length
  computation (`LL_LENHI << 15`), which could truncate the length of line
  lists longer than 32767 words.

## [0.3.0]

### Fixed

- `pl_p2li`: fix an off-by-one in the returned line-list length. The encoder
  returned `op - 1` (a 1-based holdover from the C source) while `op` was
  already the 0-based word count, so callers wrote out a line list missing its
  final word and could truncate the encoded output. It now returns `op`.

### Changed

- The core crate is now dependency-free. The `arbitrary`-derived `Data` input
  type and the `arbitrary` dependency moved entirely into the `fuzz` crate and
  are no longer part of the public API.
- Migrated to Rust edition 2024.

### Added

- `ALGORITHM.md` documenting the line-list format, the instruction opcodes, and
  the format's limitations.
