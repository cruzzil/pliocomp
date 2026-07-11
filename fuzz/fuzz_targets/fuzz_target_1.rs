#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate pliocomp;
use crate::pliocomp::*;

fuzz_target!(|data: Data| {
    if data.d.len() < 10 {
        return;
    }
    
    let input = data.d;
    let xs = 0;
    let mut compressed: Vec<i16> = vec![0; 20000];
    let npix = input.len();

    let res = pl_p2li(&input, xs, &mut compressed, npix);

    //println!("Compressed items: {res}");

    let mut uncompressed: Vec<i32> = vec![0; npix];

    let res2 = pl_l2pi(&compressed, xs, &mut uncompressed, npix);

    //println!("Uncompressed items: {res}");

    assert_eq!(input, uncompressed);
});
