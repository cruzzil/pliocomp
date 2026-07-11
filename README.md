# pliocomp

[![Crates.io](https://img.shields.io/crates/v/pliocomp.svg)](https://crates.io/crates/pliocomp)
[![Actions Status](https://github.com/cruzzil/pliocomp/workflows/CI/badge.svg)](https://github.com/cruzzil/pliocomp/actions)
[![Documentation](https://docs.rs/pliocomp/badge.svg)](https://docs.rs/pliocomp/)
[![codecov](https://codecov.io/gh/cruzzil/pliocomp/branch/main/graph/badge.svg?token=0DNCU7VRH2)](https://codecov.io/gh/cruzzil/pliocomp)
[![Dependency status](https://deps.rs/repo/github/cruzzil/pliocomp/status.svg)](https://deps.rs/repo/github/cruzzil/pliocomp)

`pliocomp` is a small, dependency-free Rust crate implementing **PLIO** (Pixel LIst I/O)
mask compression, the run-length scheme used by IRAF and by the FITS tile-compression
convention (section 6.3 of the standard) to store integer *mask* images losslessly.

PLIO is built for masks: arrays that are mostly zero, punctuated by runs of a constant
non-negative "high value" that flag regions of interest (bad pixels, object footprints,
quality flags, and the like). Such data compresses extremely well with a compact
run-length instruction set. It is *not* meant for continuous-tone imagery, which expands
rather than shrinks.

## Installation

```bash
cargo add pliocomp
```

## Usage

The crate exposes a single pair of inverse functions — a lossless codec:

| Function | Direction | Meaning |
|----------|-----------|---------|
| `pl_p2li(pxsrc, xs, lldst, npix) -> usize` | pixels → line list | **encode** ("pixel to line list"), returns the list length |
| `pl_l2pi(ll_src, xs, px_dst, npix) -> usize` | line list → pixels | **decode** ("line list to pixel"), returns the pixel count |

```rust
use pliocomp::{pl_p2li, pl_l2pi};

// A mask line: mostly zero with a short run of a constant high value.
let pixels: Vec<i32> = vec![0, 0, 0, 5, 5, 5, 0, 0];

// Encode into a caller-sized line-list buffer (i16 words).
let mut line_list = vec![0i16; pixels.len() * 2 + 8];
let ll_len = pl_p2li(&pixels, 0, &mut line_list, pixels.len());

// Decode back into a pixel buffer.
let mut decoded = vec![0i32; pixels.len()];
let n = pl_l2pi(&line_list[..ll_len], 0, &mut decoded, pixels.len());

assert_eq!(&decoded[..n], &pixels[..]);
```

`xs` is the starting index into the pixel array and `npix` the pixel count; decoding
supports clipping to a sub-range `[xs, xs+npix)` so a caller can expand part of a line
without materializing the whole thing. The caller passes a pre-sized output buffer, and
the returned length reports how much was actually written.

### Round-trip caveats

PLIO can only represent **non-negative** values: negative inputs decode back as `0`. This
is a format limit, not a bug. Very large high values are also bounded by the `i16` words
the format is built on. See the algorithm document below for the full details.

> **Note:** versions before **0.4.0** silently corrupted any pixel value larger than 12
> bits (> 4095) on decode. If you rely on values above 4095, use 0.4.0 or later.

## Algorithm

For a full description of the line-list format, the instruction opcodes, how the encoder
and decoder work, and the format's limitations, see **[ALGORITHM.md](ALGORITHM.md)**.

## References

* Algorithm description, section 6.3 of the FITS tile-compression standard:
  <https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf>
* Original IRAF PLIO documentation:
  <https://github.com/iraf-community/iraf/blob/main/sys/plio/PLIO.hlp>
* HEASARC / CFITSIO tile compression overview:
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/compression.html>
* CFITSIO `compress_image` details:
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/compression/compress_image.html>
* fpack paper (O1.5):
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/fpack/O1.5.pdf>

### Ported from

This crate is a port of the IRAF PLIO routines, with a secondary port of the cleaned-up
`pliocomp.c` from CFITSIO (most `goto`s removed, magic constants named):

* [`plp2l.gx`](https://github.com/iraf-community/iraf/blob/main/sys/plio/plp2l.gx)
* [`pll2p.gx`](https://github.com/iraf-community/iraf/blob/main/sys/plio/pll2p.gx)
* [`plio.h`](https://github.com/iraf-community/iraf/blob/main/lib/plio.h)
  (note the `B`-suffixed constants there are **octal**)

## License

MIT
