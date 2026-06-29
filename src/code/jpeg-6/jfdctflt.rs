/*
 * jfdctflt.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains a floating-point implementation of the
 * forward DCT (Discrete Cosine Transform).
 *
 * This implementation should be more accurate than either of the integer
 * DCT implementations.  However, it may not give the same results on all
 * machines because of differences in roundoff behavior.  Speed will depend
 * on the hardware's floating point capacity.
 *
 * A 2-D DCT can be done by 1-D DCT on each row followed by 1-D DCT
 * on each column.  Direct algorithms are also available, but they are
 * much more complex and seem not to be any faster when reduced to code.
 *
 * This implementation is based on Arai, Agui, and Nakajima's algorithm for
 * scaled DCT.  Their original paper (Trans. IEICE E-71(11):1095) is in
 * Japanese, but the algorithm is described in the Pennebaker & Mitchell
 * JPEG textbook (see REFERENCES section in file README).  The following code
 * is based directly on figure 4-8 in P&M.
 * While an 8-point DCT cannot be done in less than 11 multiplies, it is
 * possible to arrange the computation so that many of the multiplies are
 * simple scalings of the final outputs.  These multiplies can then be
 * folded into the multiplications or divisions by the JPEG quantization
 * table entries.  The AA&N method leaves only 5 multiplies and 29 adds
 * to be done in the DCT itself.
 * The primary disadvantage of this method is that with a fixed-point
 * implementation, accuracy is lost due to imprecise representation of the
 * scaled quantization values.  However, that problem does not arise if
 * we use floating point arithmetic.
 */

// DCTSIZE is specialized to 8x8 DCTs
const DCTSIZE: usize = 8;

/*
 * Perform the forward DCT on one block of samples.
 */
pub unsafe fn jpeg_fdct_float(mut data: *mut f32) {
  let mut tmp0: f32;
  let mut tmp1: f32;
  let mut tmp2: f32;
  let mut tmp3: f32;
  let mut tmp4: f32;
  let mut tmp5: f32;
  let mut tmp6: f32;
  let mut tmp7: f32;
  let mut tmp10: f32;
  let mut tmp11: f32;
  let mut tmp12: f32;
  let mut tmp13: f32;
  let mut z1: f32;
  let mut z2: f32;
  let mut z3: f32;
  let mut z4: f32;
  let mut z5: f32;
  let mut z11: f32;
  let mut z13: f32;
  let mut dataptr: *mut f32;
  let mut ctr: i32;

  /* Pass 1: process rows. */

  dataptr = data;
  ctr = (DCTSIZE - 1) as i32;
  while ctr >= 0 {
    tmp0 = *dataptr.add(0) + *dataptr.add(7);
    tmp7 = *dataptr.add(0) - *dataptr.add(7);
    tmp1 = *dataptr.add(1) + *dataptr.add(6);
    tmp6 = *dataptr.add(1) - *dataptr.add(6);
    tmp2 = *dataptr.add(2) + *dataptr.add(5);
    tmp5 = *dataptr.add(2) - *dataptr.add(5);
    tmp3 = *dataptr.add(3) + *dataptr.add(4);
    tmp4 = *dataptr.add(3) - *dataptr.add(4);

    /* Even part */

    tmp10 = tmp0 + tmp3;	/* phase 2 */
    tmp13 = tmp0 - tmp3;
    tmp11 = tmp1 + tmp2;
    tmp12 = tmp1 - tmp2;

    *dataptr.add(0) = tmp10 + tmp11; /* phase 3 */
    *dataptr.add(4) = tmp10 - tmp11;

    z1 = (tmp12 + tmp13) * 0.707106781_f32; /* c4 */
    *dataptr.add(2) = tmp13 + z1;	/* phase 5 */
    *dataptr.add(6) = tmp13 - z1;

    /* Odd part */

    tmp10 = tmp4 + tmp5;	/* phase 2 */
    tmp11 = tmp5 + tmp6;
    tmp12 = tmp6 + tmp7;

    /* The rotator is modified from fig 4-8 to avoid extra negations. */
    z5 = (tmp10 - tmp12) * 0.382683433_f32; /* c6 */
    z2 = 0.541196100_f32 * tmp10 + z5; /* c2-c6 */
    z4 = 1.306562965_f32 * tmp12 + z5; /* c2+c6 */
    z3 = tmp11 * 0.707106781_f32; /* c4 */

    z11 = tmp7 + z3;		/* phase 5 */
    z13 = tmp7 - z3;

    *dataptr.add(5) = z13 + z2;	/* phase 6 */
    *dataptr.add(3) = z13 - z2;
    *dataptr.add(1) = z11 + z4;
    *dataptr.add(7) = z11 - z4;

    dataptr = dataptr.add(DCTSIZE);		/* advance pointer to next row */
    ctr -= 1;
  }

  /* Pass 2: process columns. */

  dataptr = data;
  ctr = (DCTSIZE - 1) as i32;
  while ctr >= 0 {
    tmp0 = *dataptr.add(DCTSIZE * 0) + *dataptr.add(DCTSIZE * 7);
    tmp7 = *dataptr.add(DCTSIZE * 0) - *dataptr.add(DCTSIZE * 7);
    tmp1 = *dataptr.add(DCTSIZE * 1) + *dataptr.add(DCTSIZE * 6);
    tmp6 = *dataptr.add(DCTSIZE * 1) - *dataptr.add(DCTSIZE * 6);
    tmp2 = *dataptr.add(DCTSIZE * 2) + *dataptr.add(DCTSIZE * 5);
    tmp5 = *dataptr.add(DCTSIZE * 2) - *dataptr.add(DCTSIZE * 5);
    tmp3 = *dataptr.add(DCTSIZE * 3) + *dataptr.add(DCTSIZE * 4);
    tmp4 = *dataptr.add(DCTSIZE * 3) - *dataptr.add(DCTSIZE * 4);

    /* Even part */

    tmp10 = tmp0 + tmp3;	/* phase 2 */
    tmp13 = tmp0 - tmp3;
    tmp11 = tmp1 + tmp2;
    tmp12 = tmp1 - tmp2;

    *dataptr.add(DCTSIZE * 0) = tmp10 + tmp11; /* phase 3 */
    *dataptr.add(DCTSIZE * 4) = tmp10 - tmp11;

    z1 = (tmp12 + tmp13) * 0.707106781_f32; /* c4 */
    *dataptr.add(DCTSIZE * 2) = tmp13 + z1; /* phase 5 */
    *dataptr.add(DCTSIZE * 6) = tmp13 - z1;

    /* Odd part */

    tmp10 = tmp4 + tmp5;	/* phase 2 */
    tmp11 = tmp5 + tmp6;
    tmp12 = tmp6 + tmp7;

    /* The rotator is modified from fig 4-8 to avoid extra negations. */
    z5 = (tmp10 - tmp12) * 0.382683433_f32; /* c6 */
    z2 = 0.541196100_f32 * tmp10 + z5; /* c2-c6 */
    z4 = 1.306562965_f32 * tmp12 + z5; /* c2+c6 */
    z3 = tmp11 * 0.707106781_f32; /* c4 */

    z11 = tmp7 + z3;		/* phase 5 */
    z13 = tmp7 - z3;

    *dataptr.add(DCTSIZE * 5) = z13 + z2; /* phase 6 */
    *dataptr.add(DCTSIZE * 3) = z13 - z2;
    *dataptr.add(DCTSIZE * 1) = z11 + z4;
    *dataptr.add(DCTSIZE * 7) = z11 - z4;

    dataptr = dataptr.add(1);			/* advance pointer to next column */
    ctr -= 1;
  }
}
