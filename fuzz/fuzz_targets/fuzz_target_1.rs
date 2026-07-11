#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use pliocomp::*;

/// Fuzzer input. Lives in the fuzz crate — the core library has no reason to
/// expose a harness type or depend on `arbitrary`.
#[derive(Clone, Debug, Arbitrary)]
struct Data {
    d: Vec<i32>,
}

fuzz_target!(|data: Data| {
    if data.d.len() < 10 {
        return;
    }

    // PLIO only represents non-negative mask values up to the ~15-bit `i16`
    // range (see ALGORITHM.md). Values outside 0..=32767 are format
    // limitations, not codec bugs, so map the arbitrary input into the
    // representable domain before checking the round-trip.
    let input: Vec<i32> = data.d.iter().map(|v| v.rem_euclid(32768)).collect();
    let xs = 0;
    let npix = input.len();

    // Size the output generously: worst-case (a distinct value at every pixel)
    // costs a few instruction words per pixel, so this cannot overflow.
    let mut compressed: Vec<i16> = vec![0; npix * 5 + 32];

    let res = pl_p2li(&input, xs, &mut compressed, npix);

    let mut uncompressed: Vec<i32> = vec![0; npix];

    let _res2 = pl_l2pi(&compressed[..res], xs, &mut uncompressed, npix);

    assert_eq!(input, uncompressed);
});
