Algorithm description section 6.3 https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf

Original related IRAF PLIO docs: https://github.com/iraf-community/iraf/blob/main/sys/plio/PLIO.hlp

Port of:

https://github.com/iraf-community/iraf/blob/main/sys/plio/plp2l.gx

https://github.com/iraf-community/iraf/blob/main/sys/plio/pll2p.gx

https://github.com/iraf-community/iraf/blob/main/lib/plio.h Note 'B' type on constant is Octal

Secondary Port of pliocomp.c in CFITSIO to remove most goto statements and clean up magic constants.