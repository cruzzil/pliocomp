# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
