// Macro defines for the line list data format.

// ----- Old/original line list header definitions.  This version uses a
// ----- three element header, but the maximum values are limited to 32K by
// ----- the use of type short.

// Line list definitions (accessed as a short integer array).
// const OLL_NREF: usize = 0; // number of references (kept for reference to the original implementation)
// const OLL_BLEN: usize = 1; // length of buffer containing LL (kept for reference to the original implementation)
const OLL_LEN: usize = 2; // length of encoded line list
const OLL_FIRST: usize = 3; // first data range entry in list

// ----- New format line list header definitions.  This version uses a
// ----- variable length header and a version number to allow new encodings
// ----- while retaining backwards compatibility.

const LL_CURVERSION: i16 = -100; // LL version code (must be negative)
//const	LL_OLDFORMAT(a)	(a[LL_VERSION] > 0)
const LL_CURHDRLEN: i16 = 7;

// Line list definitions (accessed as a short integer array).
const LL_NREFS: usize = 0; // number of references
const LL_HDRLEN: usize = 1; // length of encoded line list
const LL_VERSION: usize = 2; // version number (negative)
const LL_LENLO: usize = 3; // length of encoded line list
const LL_LENHI: usize = 4; // length of encoded line list
// const LL_BLENLO: usize = 5; // length of LL buffer (kept for reference to the original implementation)
// const LL_BLENHI: usize = 6; // length of LL buffer (kept for reference to the original implementation)

// Packed instruction decoding.
const I_SHIFT: i32 = 4096;
const I_DATAMAX: i32 = 4095;

// LL instruction opcodes.
const I_ZN: i32 = 0; // N zeros
const I_HN: i32 = 4; // N high values
const I_PN: i32 = 5; // N-1 zeros and 1 high value
const I_SH: i32 = 1; // set high value (2 words)
const I_IH: i32 = 2; // increment high value
const I_DH: i32 = 3; // decrement high value
const I_IS: i32 = 6; // increment and output 1 high value
const I_DS: i32 = 7; // decrement and output 1 high value

// The LL instruction opcodes again, but as bitmasks this time.
const M_SH: i32 = 4096;
const M_IH: i32 = 8192;
const M_DH: i32 = 12288;
const M_HN: i32 = 16384;
const M_PN: i32 = 20480;

// The following bit is set if the instruction changes the current position.
const M_MOVE: i16 = 16384;

/// Convert a pixel array to a line list.
///
/// Arguments
///
/// * `pxsrc` - input pixel array
/// * `xs` - starting index in pxsrc (?)
/// * `lldst` - encoded line list
/// * `npix` - number of pixels to convert
///
/// Returns
///
/// * The length of the list is returned as the function value
pub fn pl_p2li(pxsrc: &[i32], xs: i32, lldst: &mut [i16], npix: usize) -> usize {
    let mut v;

    let mut dv: i32;
    let mut np: i32;

    let mut nv: i32 = 0;

    let mut nz: i32;

    /* Parameter adjustments */
    //--lldst;
    //--pxsrc;

    // No input pixels?
    if npix == 0 {
        return 0;
    }

    // Initialize the linelist header.
    lldst[LL_VERSION] = LL_CURVERSION;
    lldst[LL_HDRLEN] = LL_CURHDRLEN;
    lldst[LL_NREFS] = 0;
    lldst[5] = 0;
    lldst[6] = 0;

    let xe = xs + (npix as i32) - 1;
    let mut op: usize = (LL_CURHDRLEN) as usize;

    // Pack the pixel array into a line list.  This is done by scanning
    // the pixel list for successive ranges of pixels of constant nonzero
    // value, where each range is described as follows:

    let zero: i32 = 0;
    let mut pv: i32 = i32::max(zero, pxsrc[xs as usize]); // pixel value of current range
    let mut x1: i32 = xs; // start index of current range
    let mut iz: i32 = xs; // start index of range of zeros
    let mut hi: i32 = 1; // current high value

    // Process the data array.
    for ip in xs..=xe {
        //for (ip = xs; ip <= i__1; ++ip) {
        if ip < xe {
            // Get the next pixel value, loop again if same as previous one.
            nv = i32::max(zero, pxsrc[(ip + 1) as usize]);
            if nv == pv {
                continue;
            }

            // If current range is zero, loop again to get nonzero range.
            if pv == 0 {
                pv = nv;
                x1 = ip + 1;
                continue;
            }
        } else if pv == 0 {
            x1 = xe + 1;
        }

        // Encode an instruction to regenerate the current range I0-IP
        // of N data values of nonzero level PV.  In the most complex case
        // we must update the high value and output a range of zeros,
        // followed by a range of NP high values.  If NP is 1, we can
        // probably use a PN or [ID]S instruction to save space.

        np = ip - x1 + 1;
        nz = x1 - iz;

        // Change the high value?
        if pv > 0 {
            dv = pv - hi;
            if dv != 0 {
                // Output IH or DH instruction?
                hi = pv;
                if dv.abs() > I_DATAMAX {
                    lldst[op] = ((pv & I_DATAMAX) + M_SH) as i16;
                    op += 1;
                    lldst[op] = (pv / I_SHIFT) as i16;
                    op += 1;
                } else {
                    if dv < 0 {
                        lldst[op] = (-dv + M_DH) as i16;
                    } else {
                        lldst[op] = (dv + M_IH) as i16;
                    }
                    op += 1;

                    // Convert to IS or DS if range is a single pixel.
                    if np == 1 && nz == 0 {
                        v = lldst[op - 1];
                        lldst[op - 1] = v | M_MOVE;
                        np = 0; // goto done
                    }
                }
            }
        }

        // Output range of zeros to catch up to current range?
        // The I_DATAMAX-1 limit is to allow adding M_PN+1 without
        // overflowing the range of the data segment.

        if nz > 0 {
            // Output the ZN instruction.
            while nz > 0 {
                lldst[op] = i32::min(I_DATAMAX - 1, nz) as i16;
                op += 1;
                nz -= I_DATAMAX - 1
            }

            // Convert to PN if range is a single pixel.
            if np == 1 && pv > 0 {
                lldst[op - 1] = lldst[op - 1] + (M_PN as i16) + 1;
                np = 0; //goto done
            }
        }

        // The only thing left is the HN instruction if we get here.
        while np > 0 {
            lldst[op] = (i32::min(I_DATAMAX, np) + M_HN) as i16;
            op += 1;
            np -= I_DATAMAX;
        }

        // done:
        x1 = ip + 1;
        iz = x1;
        pv = nv;
    }

    // LL_LEN is the total number of words in the line list, header included --
    // the same number this function returns.
    //
    // The original C is 1-based (it starts `op` at 8 and decrements `lldst` on
    // entry), so there `op - 1` is simultaneously the word count and the
    // 1-based index of the last word, and it stores and returns that.  Here
    // `op` is 0-based, so the count is `op` itself: storing `op - 1` would put
    // a length one short of the truth on the wire, which a conforming decoder
    // reads as one instruction fewer than was written.
    lldst[LL_LENLO] = (op % 32768) as i16;
    lldst[LL_LENHI] = (op / 32768) as i16;
    op
}

/// Translate a PLIO line list into an integer pixel array.
///
/// Arguments
///
/// * `ll_src` - encoded line list
/// * `xs` - starting index in ll_src
/// * `px_dst` - output pixel array
/// * `npix` - number of pixels to convert
///
/// Returns
///
/// * The number of pixels output (always npix) is returned as the function value.
pub fn pl_l2pi(ll_src: &[i16], xs: i32, px_dst: &mut [i32], npix: usize) -> usize {
    let mut data;
    let mut otop: usize;
    let lllen: i32;
    let mut i1: i32;
    let mut i2: i32;

    let mut x2: i32;

    let mut np: i32;

    let mut opcode: i32;
    let llfirt: i32;

    /* Parameter adjustments */
    //--px_dst;
    //--ll_src;

    // Support old format line lists.
    //
    // `lllen` is a word *count* and `llfirt` the 0-based index of the first
    // data word, so the scan below runs over `llfirt..lllen` -- exclusive.
    // The C reaches the same words with an inclusive loop because it is
    // 1-based throughout (`for (ip = llfirt; ip <= lllen; ++ip)` with
    // `--ll_src`, `llfirt = ll_src[2] + 1`).
    if ll_src[LL_VERSION] > 0 {
        lllen = ll_src[OLL_LEN] as i32;
        llfirt = OLL_FIRST as i32;
    } else {
        lllen = ((ll_src[LL_LENHI] as i32) << 15) + ll_src[LL_LENLO] as i32; // LL_LEN
        llfirt = (ll_src[LL_HDRLEN]) as i32; // LL_FIRST
    }

    // No pixels?
    if npix == 0 || lllen <= 0 {
        return 0;
    }

    let xe: i32 = xs + (npix as i32);
    let mut skipwd: bool = false;
    let mut op: usize = 0;
    let mut x1: i32 = 1;
    let mut pv: i32 = 1;

    for ip in llfirt..lllen {
        if skipwd {
            skipwd = false;
            continue;
        }

        opcode = (ll_src[ip as usize] / 4096) as i32; // I_OPCODE
        data = (ll_src[ip as usize] & 4095) as i32; // I_DATA

        let mut putpix = false;
        match opcode {
            I_ZN | I_HN | I_PN => {
                // Determine inbounds region of segment.
                x2 = x1 + data - 1;
                i1 = i32::max(x1, xs);
                i2 = i32::min(x2, xe);

                // Process segment if any region is inbounds.
                np = i2 - i1 + 1;
                if np > 0 {
                    otop = ((op as i32) + np - 1) as usize;
                    if opcode == I_HN {
                        #[allow(clippy::needless_range_loop)]
                        for idx in op..=otop {
                            px_dst[idx] = pv;
                        }
                    } else {
                        #[allow(clippy::needless_range_loop)]
                        for idx in op..=otop {
                            px_dst[idx] = 0;
                        }
                        if opcode == I_PN && i2 == x2 {
                            px_dst[otop] = pv;
                        }
                    }
                    op = otop + 1;
                }

                // Advance the line index.
                x1 = x2 + 1;
            }
            I_SH => {
                // Widen to i32 *before* shifting: the high word can be >= 8,
                // which would overflow the i16 when shifted left by 12 and
                // corrupt the reconstructed pixel value.
                pv = ((ll_src[(ip + 1) as usize] as i32) << 12) + data;
                skipwd = true;
            }

            I_IH => {
                pv += data;
            }

            I_DH => {
                pv -= data;
            }
            I_IS => {
                pv += data;
                putpix = true;
            }
            I_DS => {
                pv -= data;
                putpix = true;
            }
            _ => (),
        }

        if putpix {
            if x1 >= xs && x1 <= xe {
                px_dst[op] = pv;
                op += 1;
            }

            x1 += 1;
        }

        if x1 > xe {
            break;
        }
    }

    #[allow(clippy::needless_range_loop)]
    for idx in op..npix {
        px_dst[idx] = 0;
    }
    npix
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode `input` and return the line list, trimmed to the returned length.
    fn encode(input: &[i32]) -> Vec<i16> {
        let mut ll = vec![0i16; input.len() * 5 + 64];
        let n = pl_p2li(input, 0, &mut ll, input.len());
        ll.truncate(n);
        ll
    }

    /// Encode `input` and decode it again.
    fn round_trip(input: &[i32]) -> Vec<i32> {
        let ll = encode(input);
        let mut out = vec![0i32; input.len()];
        let n = pl_l2pi(&ll, 0, &mut out, input.len());
        assert_eq!(n, input.len());
        out
    }

    #[test]
    fn it_works() {
        let input: [i32; 9] = [3, 56, 3343, 22225, 3, 66, 3, 3, 3];
        let xs = 0;
        let mut compressed: [i16; 200] = [0; 200];
        let npix = 9;

        let res = pl_p2li(&input, xs, &mut compressed, npix);

        println!("Compressed items: {res}");

        // The returned length must cover the full encoded list (header + data),
        // otherwise the caller writes out a truncated list.
        assert!(res <= compressed.len());

        let mut uncompressed: [i32; 10] = [0; 10];

        let res2 = pl_l2pi(&compressed[..res], xs, &mut uncompressed, npix);

        println!("Uncompressed items: {res2}");

        // Round-trip must reproduce the original pixels exactly.
        assert_eq!(res2, npix);
        assert_eq!(&uncompressed[..npix], &input[..]);
    }

    /// `LL_LEN` is the total word count of the line list, header included --
    /// the same number `pl_p2li` returns.
    ///
    /// Storing it as the *last index* instead (one less) produces a line list
    /// that this crate could still read but no conforming PLIO decoder could:
    /// CFITSIO drops the final instruction and reconstructs the tail of the
    /// row wrongly.  See the CHANGELOG entry for 0.5.0.
    #[test]
    fn ll_len_header_is_the_word_count() {
        for input in [
            vec![7],
            vec![3, 56, 3343, 22225, 3, 66, 3, 3, 3],
            vec![0, 0, 0, 5, 5, 0, 9, 0],
            (0..500).map(|i| (i * 37) % 4096).collect::<Vec<i32>>(),
        ] {
            let ll = encode(&input);
            let ll_len = ((ll[LL_LENHI] as i32) << 15) + ll[LL_LENLO] as i32;
            assert_eq!(
                ll_len as usize,
                ll.len(),
                "LL_LEN must equal the number of words written, for {:?}...",
                &input[..input.len().min(6)]
            );
            assert_eq!(ll[LL_HDRLEN], LL_CURHDRLEN);
            assert_eq!(ll[LL_VERSION], LL_CURVERSION);
        }
    }

    /// Word-for-word against the reference C (`c_example/pliocomp.c`, itself
    /// f2c'd from IRAF's `plp2l.gx` and the copy CFITSIO ships).  These pin
    /// the on-disk format independently of our own decoder, which is the only
    /// way a self-consistent but non-conforming encoding gets caught.
    #[test]
    fn encoding_matches_the_reference_c() {
        // (pixels, expected line list)
        let cases: &[(&[i32], &[i16])] = &[
            (&[7], &[0, 7, -100, 8, 0, 0, 0, 24582]),
            (&[1, 2, 3], &[0, 7, -100, 10, 0, 0, 0, 16385, 24577, 24577]),
            (
                &[0, 0, 0, 5, 5, 0, 9, 0],
                &[0, 7, -100, 13, 0, 0, 0, 8196, 3, 16386, 8196, 20482, 1],
            ),
            (
                &[3, 56, 3343, 22225, 3, 66, 3, 3, 3],
                &[
                    0, 7, -100, 19, 0, 0, 0, 24578, 24629, 27863, 5841, 5, 16385, 4099, 0, 16385,
                    24639, 12351, 16387,
                ],
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(&encode(input)[..], *expected, "encoding of {input:?}");
        }
    }

    /// The same vectors read back, so the decoder is pinned against the
    /// reference too and not merely against our own encoder.
    #[test]
    fn decoding_matches_the_reference_c() {
        let cases: &[(&[i16], &[i32])] = &[
            (&[0, 7, -100, 8, 0, 0, 0, 24582], &[7]),
            (&[0, 7, -100, 10, 0, 0, 0, 16385, 24577, 24577], &[1, 2, 3]),
            (
                &[0, 7, -100, 13, 0, 0, 0, 8196, 3, 16386, 8196, 20482, 1],
                &[0, 0, 0, 5, 5, 0, 9, 0],
            ),
            (
                &[
                    0, 7, -100, 19, 0, 0, 0, 24578, 24629, 27863, 5841, 5, 16385, 4099, 0, 16385,
                    24639, 12351, 16387,
                ],
                &[3, 56, 3343, 22225, 3, 66, 3, 3, 3],
            ),
        ];

        for (ll, expected) in cases {
            let mut out = vec![0i32; expected.len()];
            let n = pl_l2pi(ll, 0, &mut out, expected.len());
            assert_eq!(n, expected.len());
            assert_eq!(&out[..], *expected, "decoding of {ll:?}");
        }
    }

    /// A zero run is emitted in chunks of at most `I_DATAMAX - 1`, not
    /// `I_DATAMAX`, so that the `+ M_PN + 1` conversion below it cannot carry
    /// out of the data field and into the opcode.
    ///
    /// This is where we deliberately differ from the copy of `pliocomp.c` in
    /// CFITSIO, which chunks at `I_DATAMAX` and, for exactly 4095 zeros
    /// followed by one non-zero pixel, emits 24576 -- opcode `I_IS` rather
    /// than `I_PN` -- and reconstructs the row completely wrongly.  IRAF's
    /// `plp2l.gx` has the `-1`; the f2c'd version lost it.
    #[test]
    fn long_zero_runs_do_not_overflow_the_opcode_field() {
        for n in [4093usize, 4094, 4095, 4096, 8190, 12000] {
            let mut input = vec![0i32; n];
            input.push(5);
            let out = round_trip(&input);
            assert_eq!(
                out, input,
                "{n} zeros followed by a pixel did not round trip"
            );

            // and no data word may be mistaken for a different instruction
            for w in &encode(&input)[LL_CURHDRLEN as usize..] {
                let opcode = (*w as i32) / 4096;
                assert!(
                    matches!(
                        opcode,
                        I_ZN | I_HN | I_PN | I_SH | I_IH | I_DH | I_IS | I_DS
                    ),
                    "word {w} decodes to unknown opcode {opcode}"
                );
            }
        }
    }

    /// The shapes the encoder branches on: constant runs, isolated pixels,
    /// values needing the two-word `I_SH` form, and the empty input.
    #[test]
    fn round_trips() {
        let cases: Vec<Vec<i32>> = vec![
            vec![],
            vec![0],
            vec![1],
            vec![32767],
            vec![0, 0, 0, 0],
            vec![5; 40],
            vec![0; 40],
            vec![1, 0, 1, 0, 1, 0, 1, 0],
            vec![4095, 4096, 4097],
            vec![32767, 0, 32767],
            vec![0; 20]
                .into_iter()
                .chain([9])
                .chain(vec![0; 20])
                .collect(),
            (0..1000).map(|i| (i * 7919) % 32768).collect(),
            (0..1000)
                .map(|i| if i % 3 == 0 { 0 } else { 12345 })
                .collect(),
        ];

        for input in cases {
            if input.is_empty() {
                let mut ll = [0i16; 32];
                assert_eq!(
                    pl_p2li(&input, 0, &mut ll, 0),
                    0,
                    "empty input encodes to nothing"
                );
                continue;
            }
            assert_eq!(
                round_trip(&input),
                input,
                "round trip of {} pixels",
                input.len()
            );
        }
    }
}
