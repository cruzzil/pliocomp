# The PLIO Mask Compression Algorithm

`pliocomp` implements the **PLIO** (Pixel LIst I/O) compression scheme used by IRAF and
by the FITS tile-compression convention to store *mask* images. This document explains
what the format is, how the encoder and decoder work, and — importantly — the limits of
what it can represent losslessly. See [References](#references) at the end.

## What PLIO is for

PLIO is designed to compress **integer mask data**, not general images. A mask is an array
that is mostly **zero** (background), punctuated by **runs of a constant non-negative
value** (the "high value") that flag regions of interest — bad pixels, object footprints,
quality flags, etc. Because such data is dominated by long zero runs and piecewise-constant
non-zero runs, it compresses extremely well with a small run-length instruction set.

It is *not* a good fit for continuous-tone imagery: data that changes value at almost every
pixel expands rather than shrinks.

The crate exposes the codec as a pair of inverse functions in [src/lib.rs](src/lib.rs):

| Function | Direction | Meaning |
|----------|-----------|---------|
| `pl_p2li(pxsrc, xs, lldst, npix)` | pixels → line list | **encode** ("pixel to line list") |
| `pl_l2pi(ll_src, xs, px_dst, npix)` | line list → pixels | **decode** ("line list to pixel") |

A "line list" (LL) is the compressed form of one 1-D line of `npix` pixels.

## The line-list format

A line list is an array of 16-bit signed words (`i16`). It has a fixed header followed by a
stream of variable-length instructions.

### Header (new format)

The current format uses a 7-word header (`LL_CURHDRLEN`) and a **negative** version code
(`LL_CURVERSION = -100`) so it can be distinguished from the old 3-word format, which used a
*positive* version field. The decoder reads `LL_VERSION`; if it is `> 0` it falls back to the
old header layout.

| Index | Name | Meaning |
|-------|------|---------|
| 0 | `LL_NREFS`  | number of references (unused here, 0) |
| 1 | `LL_HDRLEN` | header length in words (7); also the index of the first instruction |
| 2 | `LL_VERSION`| version code, negative for the new format |
| 3 | `LL_LENLO`  | encoded length, low half |
| 4 | `LL_LENHI`  | encoded length, high half |
| 5 | `LL_BLENLO` | buffer length, low half |
| 6 | `LL_BLENHI` | buffer length, high half |

The length is split into lo/hi halves because a single `i16` cannot hold lengths above 32767.

### Instruction encoding

Each instruction is one word (two for `I_SH`). The top nibble is the **opcode**, the low 12
bits are the **data** field:

```
opcode = word / 4096      // == word >> 12
data   = word & 4095      // 12-bit payload, 0..=4095  (I_DATAMAX = 4095)
```

The eight opcodes fall into two groups. Some emit pixels ("move" the current position),
others only modify the running *high value* `pv`:

| Opcode | Value | Mask (`word` base) | Emits pixels? | Effect |
|--------|-------|--------------------|---------------|--------|
| `I_ZN` | 0 | 0        | yes | output `data` **zeros** |
| `I_SH` | 1 | `M_SH` = 4096   | no  | **set** high value; 2-word instruction, absolute value |
| `I_IH` | 2 | `M_IH` = 8192   | no  | **increment** high value by `data` |
| `I_DH` | 3 | `M_DH` = 12288  | no  | **decrement** high value by `data` |
| `I_HN` | 4 | `M_HN` = 16384  | yes | output `data` copies of the **high value** |
| `I_PN` | 5 | `M_PN` = 20480  | yes | output `data-1` zeros followed by **one** high value |
| `I_IS` | 6 | —        | yes | increment high value by `data`, then output **one** high value |
| `I_DS` | 7 | —        | yes | decrement high value by `data`, then output **one** high value |

The bit `M_MOVE = 16384` (bit 14) is set on any instruction that advances the current
position. Note `M_HN` and `M_PN` already have this bit set by construction, and `I_IS`/`I_DS`
are produced by OR-ing `M_MOVE` into an `I_IH`/`I_DH` word — that is how the single-pixel
"increment/decrement *and* emit" shortcuts are formed.

### Why the extra opcodes exist

`I_SH`/`I_IH`/`I_DH` change the high value without emitting pixels; `I_ZN`/`I_HN` emit runs.
The common case of "one lone high pixel" would otherwise cost two instructions (change value,
then emit one), so PLIO adds space-saving fused forms:

* `I_PN` — a zero run whose final pixel is a single high value (zeros then one flag).
* `I_IS` / `I_DS` — change the high value *and* emit exactly one pixel of it.

The encoder opportunistically rewrites a length-1 `I_HN` into one of these when the
surrounding context allows it (see the `np == 1` special cases in `pl_p2li`).

## How the encoder works (`pl_p2li`)

1. Write the header (version, hdrlen, zeros).
2. Walk the pixel array, coalescing maximal runs of a constant **non-negative** value.
   Negative pixels are clamped to zero (`max(0, pixel)`) before comparison — see limitations.
3. For each non-zero run it emits, in order:
   * a high-value change (`I_SH` for a big absolute jump, otherwise `I_IH`/`I_DH` for a
     delta of at most `I_DATAMAX`), then
   * a zero run (`I_ZN`) to skip the gap since the previous run, then
   * the high-value run itself (`I_HN`).
4. Runs longer than `I_DATAMAX` are split across multiple instructions by the `while` loops
   (zero runs use `I_DATAMAX - 1` as the chunk so a later `+M_PN+1` fusion cannot overflow the
   12-bit field).
5. The length words (indices 3/4) are back-patched, and the total word count is returned.

## How the decoder works (`pl_l2pi`)

The decoder keeps two counters: `pv` (current high value, starts at 1) and a line position
`x1` (starts at 1). It iterates the instruction words from `LL_HDRLEN` to the stored length:

* Value opcodes (`I_SH`/`I_IH`/`I_DH`) update `pv`. `I_SH` reads the **next** word too:
  `pv = (next_word << 12) + data`.
* Range opcodes (`I_ZN`/`I_HN`/`I_PN`) compute the segment `[x1, x1+data-1]`, clip it to the
  requested output window `[xs, xs+npix)`, and write zeros or `pv` accordingly (`I_PN` writes
  zeros then a single `pv` at the end).
* `I_IS`/`I_DS` update `pv` and emit one pixel.

Clipping to `[xs, xs+npix)` is what lets a caller decode a **sub-range** of a line without
expanding the whole thing. Any pixels not covered by the instruction stream are zero-filled at
the end, and decoding stops early once the position passes the end of the window.

## Limitations

These are properties of the format, not implementation choices, and they are the main things
to keep in mind (they are also the source of most round-trip surprises):

1. **Non-negative values only.** The encoder clamps every pixel with `max(0, pixel)`, so any
   negative input decodes back as **0**. PLIO cannot represent negative mask values.
2. **~15-bit value range.** Because values travel through `i16` words (the header comment notes
   the "maximum values are limited to 32K by the use of type short"), the largest losslessly
   representable high value is **32767**. `I_SH` stores the high part as `value / 4096` in one
   `i16` and the decoder reconstructs it with `high << 12`; for values above 32767 that shift
   overflows the `i16` and the round-trip is silently wrong. Larger inputs are *not* rejected —
   they are silently truncated.
3. **Piecewise-constant assumption.** Compression only helps when data consists of long
   constant runs. Worst-case input (a distinct value at every pixel) produces roughly two
   instruction words per pixel, i.e. it *expands*.
4. **Per-instruction run length ≤ 4095.** The 12-bit data field caps a single instruction;
   longer runs cost proportionally more words (handled transparently by the encoder).
5. **Signedness / overflow of intermediate math.** The high-value deltas (`I_IH`/`I_DH`) are
   themselves limited to `±I_DATAMAX` per step, and large absolute jumps must go through
   `I_SH`, which inherits limitation (2).

Because the fuzz harness feeds **arbitrary `i32`** values and asserts an exact round-trip,
inputs that violate (1) or (2) will (correctly) fail the assertion — those are format limits,
distinct from genuine codec bugs. A meaningful fuzz check must either constrain inputs to the
representable domain (`0..=32767`) or the codec must be hardened to handle out-of-domain input
deterministically.

## References

* HEASARC / CFITSIO tile compression overview:
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/compression.html>
* CFITSIO `compress_image` details:
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/compression/compress_image.html>
* fpack paper (O1.5):
  <https://heasarc.gsfc.nasa.gov/docs/software/fitsio/fpack/O1.5.pdf>
* FITS tile-compression standard, algorithm description in **section 6.3**:
  <https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf>
* Original IRAF PLIO documentation:
  <https://github.com/iraf-community/iraf/blob/main/sys/plio/PLIO.hlp>
* IRAF sources this crate ports:
  [`plp2l.gx`](https://github.com/iraf-community/iraf/blob/main/sys/plio/plp2l.gx),
  [`pll2p.gx`](https://github.com/iraf-community/iraf/blob/main/sys/plio/pll2p.gx),
  [`plio.h`](https://github.com/iraf-community/iraf/blob/main/lib/plio.h)
  (note the `B`-suffixed constants there are **octal**).
