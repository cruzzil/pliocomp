/* stdlib is needed for the abs function */
#include <stdlib.h>
/*
   The following prototype code was provided by Doug Tody, NRAO, for
   performing conversion between pixel arrays and line lists.  The
   compression technique is used in IRAF.
*/
int pl_p2li_new (int *pxsrc, int xs, short *lldst, int npix);
int pl_l2pi_new (short *ll_src, int xs, int *px_dst, int npix);


// # Macro defines for the line list data format.

// # ----- Old/original line list header definitions.  This version uses a
// # ----- three element header, but the maximum values are limited to 32K by
// # ----- the use of type short.

// # Line list definitions (accessed as a short integer array).
#define	OLL_NREF	1		// # number of references
#define	OLL_BLEN	2		// # length of buffer containing LL
#define	OLL_LEN		3		// # length of encoded line list
#define	OLL_FIRST	4		// # first data range entry in list

// # ----- New format line list header definitions.  This version uses a 
// # ----- variable length header and a version number to allow new encodings
// # ----- while retaining backwards compatibility.

#define LL_CURVERSION	(-100)		// # LL version code (must be negative)
#define	LL_OLDFORMAT(a)	(a[LL_VERSION] > 0)
#define	LL_CURHDRLEN	7

// # Line list definitions (accessed as a short integer array).
#define	LL_NREFS	1		// # number of references
#define	LL_HDRLEN	2		// # length of encoded line list
#define	LL_VERSION	3		// # version number (negative)
#define	LL_LENLO	4		// # length of encoded line list
#define	LL_LENHI	5		// # length of encoded line list
#define	LL_BLENLO	6		// # length of LL buffer
#define	LL_BLENHI	7		// # length of LL buffer

// # Packed instruction decoding.
#define I_SHIFT 4096
#define I_DATAMAX 4095

//# LL instruction opcodes.
#define	I_ZN		0		// N zeros
#define	I_HN		4		// N high values
#define	I_PN		5		// N-1 zeros and 1 high value
#define	I_SH		1		// set high value (2 words)
#define	I_IH		2		// increment high value
#define	I_DH		3		// decrement high value
#define	I_IS		6		// increment and output 1 high value
#define	I_DS		7		// decrement and output 1 high value

//# The LL instruction opcodes again, but as bitmasks this time.
#define M_SH 4096
#define M_IH 8192
#define M_DH 12288
#define M_HN 16384
#define M_PN 20480

// # The following bit is set if the instruction changes the current position.
#define M_MOVE 16384

#ifndef min
#define min(a,b)        (((a)<(b))?(a):(b))
#endif
#ifndef max
#define max(a,b)        (((a)>(b))?(a):(b))
#endif


/*
 * PL_P2L -- Convert a pixel array to a line list.  The length of the list is
 * returned as the function value.
 *
 * Translated from the SPP version using xc -f, f2c.  8Sep99 DCT.
 */

int pl_p2li_new (int *pxsrc, int xs, short *lldst, int npix)
/* int *pxsrc;                      input pixel array */
/* int xs;                          starting index in pxsrc (?) */
/* short *lldst;                    encoded line list */
/* int npix;                        number of pixels to convert */
{

    /* Local variables */
    int zero, v, x1, hi, ip, dv, xe, np, op, iz, nv = 0, pv, nz;

    /* Parameter adjustments */
    --lldst;
    --pxsrc;

    // # No input pixels?
    if (npix <= 0) {
        return 0;
    }


// Initialize the linelist header.
    lldst[LL_VERSION] = LL_CURVERSION;
    lldst[LL_HDRLEN] = LL_CURHDRLEN;
    lldst[LL_NREFS] = 0;
    lldst[6] = 0;
    lldst[7] = 0;

    
    xe = xs + npix - 1;
    op = LL_CURHDRLEN + 1;
    /*
    # Pack the pixel array into a line list.  This is done by scanning
	# the pixel list for successive ranges of pixels of constant nonzero
	# value, where each range is described as follows:
    */

    zero = 0;
    pv = max(zero,pxsrc[xs]); // # pixel value of current range
    x1 = xs;    // # start index of current range
    iz = xs;    // # start index of range of zeros
    hi = 1;     // # current high value

    // # Process the data array.
    for (ip = xs; ip <= xe; ++ip) {
        if (ip < xe) {
            // # Get the next pixel value, loop again if same as previous one.
            nv = max(zero,pxsrc[ip + 1]);
            if (nv == pv) {
                continue;
            }

            // # If current range is zero, loop again to get nonzero range.
            if (pv == 0) {
                pv = nv;
                x1 = ip + 1;
                continue;
            }
        } else if (pv == 0) {
            x1 = xe + 1;
        }

        /*
        # Encode an instruction to regenerate the current range I0-IP
        # of N data values of nonzero level PV.  In the most complex case
        # we must update the high value and output a range of zeros,
        # followed by a range of NP high values.  If NP is 1, we can
        # probably use a PN or [ID]S instruction to save space.
        */

        np = ip - x1 + 1;
        nz = x1 - iz;


        // Change the high value?
        if  (pv > 0) {
            dv = pv - hi;
            if (dv != 0) {
                // Output IH or DH instruction?
                hi = pv;
                if  (abs(dv) > I_DATAMAX) {
                    lldst[op] = (short) ((pv & I_DATAMAX) + M_SH);
                    ++op;
                    lldst[op] = (short) (pv / I_SHIFT);
                    ++op;
                } else {
                    if (dv < 0) {
                        lldst[op] = (short) (-dv + M_DH);
                    } else {
                        lldst[op] = (short) (dv + M_IH);
                    }
                    ++op;

                    // Convert to IS or DS if range is a single pixel.
                    if  (np == 1 && nz == 0) {
                        v = lldst[op - 1];
                        lldst[op - 1] = (short) (v | M_MOVE);
                        goto done;
                    }
                }
            }
        }
        /*
        # Output range of zeros to catch up to current range?
        # The I_DATAMAX-1 limit is to allow adding M_PN+1 without
        # overflowing the range of the data segment.
        */
        if (nz > 0) {
            // # Output the ZN instruction.
            for (; nz > 0;nz = nz - (I_DATAMAX-1)) {
                lldst[op] = (short) min(I_DATAMAX-1,nz);
                ++op;
            }

            // # Convert to PN if range is a single pixel.
            if (np == 1 && pv > 0) {
                lldst[op - 1] = (short) (lldst[op - 1] + M_PN + 1);
                goto done;
            }
        }

        // # The only thing left is the HN instruction if we get here.
        for (; np > 0; np = np - I_DATAMAX) {
            lldst[op] = (short) (min(I_DATAMAX,np) + M_HN);
            ++op;
        }

done:   
        x1 = ip + 1;
        iz = x1;
        pv = nv;

    }

    // LL_SETLEN(ll_dst, op - 1)
    lldst[4] = (short) ((op - 1) % 32768);
    lldst[5] = (short) ((op - 1) / 32768);
    return op - 1;

} 

/*
 * PL_L2PI -- Translate a PLIO line list into an integer pixel array.
 * The number of pixels output (always npix) is returned as the function
 * value.
 *
 * Translated from the SPP version using xc -f, f2c.  8Sep99 DCT.
 */

int pl_l2pi_new (short *ll_src, int xs, int *px_dst, int npix)
/* short *ll_src;                   encoded line list */
/* int xs;                          starting index in ll_src */
/* int *px_dst;                    output pixel array */
/* int npix;                       number of pixels to convert */
{

    /* Local variables */
    int data, otop, i, lllen, i1, i2, x1, x2, ip, xe, np, op, pv, opcode, llfirt;
    int skipwd;
    int putpix;

    /* Parameter adjustments */
    --px_dst;
    --ll_src;

    // # Support old format line lists.
    if  (ll_src[LL_VERSION] > 0) {
        lllen = ll_src[OLL_LEN];
        llfirt = OLL_FIRST;
    } else {
        lllen = (ll_src[LL_LENHI] << 15) + ll_src[LL_LENLO];  // LL_LEN
        llfirt = ll_src[LL_HDRLEN] + 1;     // LL_FIRST
    }
    
    // # No pixels?
    if  (npix <= 0 || lllen <= 0) {
        return 0;    
    }

    xe = xs + npix - 1;
    skipwd = 0; // false
    op = 1;
    x1 = 1;
    pv = 1;

    for (ip = llfirt; ip <= lllen; ++ip) {
        if (skipwd) {
           skipwd = 0;
           continue;
        }
        
        opcode = ll_src[ip] / 4096; // I_OPCODE
        data = ll_src[ip] & 4095;   // I_DATA
        putpix = 0;

        switch (opcode) {
            case I_ZN:
            case I_HN:
            case I_PN:
                // # Determine inbounds region of segment.
                x2 = x1 + data - 1;
                i1 = max(x1,xs);
                i2 = min(x2,xe);
                // # Process segment if any region is inbounds.
                np = i2 - i1 + 1;
                if (np > 0) {
                    otop = op + np - 1;
                    if (opcode == I_HN) {
                        for (i = op; i <= otop; ++i) {
                            px_dst[i] = pv;
                        }

                    } else {
                        for (i = op; i <= otop; ++i) {
                            px_dst[i] = 0;
                        }
                        if (opcode == I_PN && i2 == x2) {
                            px_dst[otop] = pv;
                        }
                        
                    }
                    op = otop + 1;
                }
            // # Advance the line index.
            x1 = x2 + 1;
            break;
        case I_SH:
            pv = (ll_src[ip + 1] << 12) + data;
            skipwd = 1;
            break;
        case I_IH:
            pv += data;
            break;
        case I_DH:
            pv -= data;
            break;
       case I_IS:
            pv += data;
            putpix = 1;
            break;
        case I_DS:
            pv -= data;
            break; 
        }

        if (putpix) {
            if (x1 >= xs && x1 <= xe) {
                px_dst[op] = pv;
                ++op;
            }
            ++x1;
        }

        if  (x1 > xe) {            break;        }

    }

    for (i = op; i <= npix; ++i) {
        px_dst[i] = 0;

    }
    return npix;
}

