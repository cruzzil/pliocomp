[![Crates.io](https://img.shields.io/crates/v/pliocomp.svg)](https://crates.io/crates/pliocomp)
[![Actions Status](https://github.com/petesmc/pliocomp/workflows/CI/badge.svg)](https://github.com/petesmc/pliocomp/actions)
[![Documentation](https://docs.rs/pliocomp/badge.svg)](https://docs.rs/pliocomp/)
[![codecov](https://codecov.io/gh/petesmc/pliocomp/branch/master/graph/badge.svg?token=YZEX06JT7K)](https://codecov.io/gh/petesmc/pliocomp)
[![Dependency status](https://deps.rs/repo/github/petesmc/pliocomp/status.svg)](https://deps.rs/repo/github/petesmc/pliocomp)

Algorithm description section 6.3 https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf

Original related IRAF PLIO docs: https://github.com/iraf-community/iraf/blob/main/sys/plio/PLIO.hlp

Port of:

https://github.com/iraf-community/iraf/blob/main/sys/plio/plp2l.gx

https://github.com/iraf-community/iraf/blob/main/sys/plio/pll2p.gx

https://github.com/iraf-community/iraf/blob/main/lib/plio.h Note 'B' type on constant is Octal

Secondary Port of pliocomp.c in CFITSIO to remove most goto statements and clean up magic constants.