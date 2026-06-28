#![allow(non_snake_case)]

use core::ffi::c_int;

pub static mut coef32: [f32; 31] = [0.0; 31]; /* 32 pt dct coefs */ // !!!!!!!!!!!!!!!!!! (only generated once (always to same value)

/*------------------------------------------------------------*/
pub unsafe fn dct_coef_addr() -> *mut f32 {
    core::ptr::addr_of_mut!(coef32).cast::<f32>()
}

/*------------------------------------------------------------*/
unsafe fn forward_bf(m: c_int, n: c_int, x: *mut f32, f: *mut f32, coef: *mut f32) {
    let mut i: c_int;
    let mut j: c_int;
    let n2: c_int;
    let mut p: c_int;
    let mut q: c_int;
    let mut p0: c_int;
    let mut k: c_int;

    p0 = 0;
    n2 = n >> 1;
    i = 0;
    while i < m {
        k = 0;
        p = p0;
        q = p + n - 1;
        j = 0;
        while j < n2 {
            *f.offset(p as isize) = *x.offset(p as isize) + *x.offset(q as isize);
            *f.offset((n2 + p) as isize) =
                *coef.offset(k as isize) * (*x.offset(p as isize) - *x.offset(q as isize));
            j += 1;
            p += 1;
            q -= 1;
            k += 1;
        }
        i += 1;
        p0 += n;
    }
}

/*------------------------------------------------------------*/
unsafe fn back_bf(m: c_int, n: c_int, x: *mut f32, f: *mut f32) {
    let mut i: c_int;
    let mut j: c_int;
    let n2: c_int;
    let n21: c_int;
    let mut p: c_int;
    let mut q: c_int;
    let mut p0: c_int;

    p0 = 0;
    n2 = n >> 1;
    n21 = n2 - 1;
    i = 0;
    while i < m {
        p = p0;
        q = p0;
        j = 0;
        while j < n2 {
            *f.offset(p as isize) = *x.offset(q as isize);
            j += 1;
            p += 2;
            q += 1;
        }
        p = p0 + 1;
        j = 0;
        while j < n21 {
            *f.offset(p as isize) = *x.offset(q as isize) + *x.offset((q + 1) as isize);
            j += 1;
            p += 2;
            q += 1;
        }
        *f.offset(p as isize) = *x.offset(q as isize);
        i += 1;
        p0 += n;
    }
}

/*------------------------------------------------------------*/
pub unsafe fn fdct32(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 32] = [0.0; 32]; /* ping pong buffers */
    let mut b: [f32; 32] = [0.0; 32];
    let mut p: c_int;
    let mut q: c_int;

    let src: *mut f32 = x;
    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage */
    p = 0;
    q = 31;
    while p < 16 {
        *a_ptr.offset(p as isize) = *src.offset(p as isize) + *src.offset(q as isize);
        *a_ptr.offset((16 + p) as isize) =
            *coef32_ptr.offset(p as isize) * (*src.offset(p as isize) - *src.offset(q as isize));
        p += 1;
        q -= 1;
    }
    forward_bf(2, 16, a_ptr, b_ptr, coef32_ptr.offset(16));
    forward_bf(4, 8, b_ptr, a_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(8, 4, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(16, 2, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(8, 4, a_ptr, b_ptr);
    back_bf(4, 8, b_ptr, a_ptr);
    back_bf(2, 16, a_ptr, b_ptr);
    back_bf(1, 32, b_ptr, c);
}

/*------------------------------------------------------------*/
pub unsafe fn fdct32_dual(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 32] = [0.0; 32]; /* ping pong buffers */
    let mut b: [f32; 32] = [0.0; 32];
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage for dual chan (interleaved x) */
    pp = 0;
    qq = 2 * 31;
    p = 0;
    while p < 16 {
        *a_ptr.offset(p as isize) = *x.offset(pp as isize) + *x.offset(qq as isize);
        *a_ptr.offset((16 + p) as isize) = *coef32_ptr.offset(p as isize)
            * (*x.offset(pp as isize) - *x.offset(qq as isize));
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 16, a_ptr, b_ptr, coef32_ptr.offset(16));
    forward_bf(4, 8, b_ptr, a_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(8, 4, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(16, 2, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(8, 4, a_ptr, b_ptr);
    back_bf(4, 8, b_ptr, a_ptr);
    back_bf(2, 16, a_ptr, b_ptr);
    back_bf(1, 32, b_ptr, c);
}

/*---------------convert dual to mono------------------------------*/
pub unsafe fn fdct32_dual_mono(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 32] = [0.0; 32]; /* ping pong buffers */
    let mut b: [f32; 32] = [0.0; 32];
    let mut t1: f32;
    let mut t2: f32;
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage  */
    pp = 0;
    qq = 2 * 31;
    p = 0;
    while p < 16 {
        t1 = 0.5f32 * (*x.offset(pp as isize) + *x.offset((pp + 1) as isize));
        t2 = 0.5f32 * (*x.offset(qq as isize) + *x.offset((qq + 1) as isize));
        *a_ptr.offset(p as isize) = t1 + t2;
        *a_ptr.offset((16 + p) as isize) = *coef32_ptr.offset(p as isize) * (t1 - t2);
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 16, a_ptr, b_ptr, coef32_ptr.offset(16));
    forward_bf(4, 8, b_ptr, a_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(8, 4, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(16, 2, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(8, 4, a_ptr, b_ptr);
    back_bf(4, 8, b_ptr, a_ptr);
    back_bf(2, 16, a_ptr, b_ptr);
    back_bf(1, 32, b_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 16 pt fdct -------------------------------*/
pub unsafe fn fdct16(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 16] = [0.0; 16]; /* ping pong buffers */
    let mut b: [f32; 16] = [0.0; 16];
    let mut p: c_int;
    let mut q: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage (drop highest sb) */
    *a_ptr.offset(0) = *x.offset(0);
    *a_ptr.offset(8) = *coef32_ptr.offset(16) * *x.offset(0);
    p = 1;
    q = 14;
    while p < 8 {
        *a_ptr.offset(p as isize) = *x.offset(p as isize) + *x.offset(q as isize);
        *a_ptr.offset((8 + p) as isize) = *coef32_ptr.offset((16 + p) as isize)
            * (*x.offset(p as isize) - *x.offset(q as isize));
        p += 1;
        q -= 1;
    }
    forward_bf(2, 8, a_ptr, b_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(4, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(8, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(4, 4, b_ptr, a_ptr);
    back_bf(2, 8, a_ptr, b_ptr);
    back_bf(1, 16, b_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 16 pt fdct dual chan---------------------*/
pub unsafe fn fdct16_dual(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 16] = [0.0; 16]; /* ping pong buffers */
    let mut b: [f32; 16] = [0.0; 16];
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage for interleaved input */
    *a_ptr.offset(0) = *x.offset(0);
    *a_ptr.offset(8) = *coef32_ptr.offset(16) * *x.offset(0);
    pp = 2;
    qq = 2 * 14;
    p = 1;
    while p < 8 {
        *a_ptr.offset(p as isize) = *x.offset(pp as isize) + *x.offset(qq as isize);
        *a_ptr.offset((8 + p) as isize) = *coef32_ptr.offset((16 + p) as isize)
            * (*x.offset(pp as isize) - *x.offset(qq as isize));
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 8, a_ptr, b_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(4, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(8, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(4, 4, b_ptr, a_ptr);
    back_bf(2, 8, a_ptr, b_ptr);
    back_bf(1, 16, b_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 16 pt fdct dual to mono-------------------*/
pub unsafe fn fdct16_dual_mono(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 16] = [0.0; 16]; /* ping pong buffers */
    let mut b: [f32; 16] = [0.0; 16];
    let mut t1: f32;
    let mut t2: f32;
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage  */
    *a_ptr.offset(0) = 0.5f32 * (*x.offset(0) + *x.offset(1));
    *a_ptr.offset(8) = *coef32_ptr.offset(16) * *a_ptr.offset(0);
    pp = 2;
    qq = 2 * 14;
    p = 1;
    while p < 8 {
        t1 = 0.5f32 * (*x.offset(pp as isize) + *x.offset((pp + 1) as isize));
        t2 = 0.5f32 * (*x.offset(qq as isize) + *x.offset((qq + 1) as isize));
        *a_ptr.offset(p as isize) = t1 + t2;
        *a_ptr.offset((8 + p) as isize) = *coef32_ptr.offset((16 + p) as isize) * (t1 - t2);
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 8, a_ptr, b_ptr, coef32_ptr.offset(16 + 8));
    forward_bf(4, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(8, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(4, 4, b_ptr, a_ptr);
    back_bf(2, 8, a_ptr, b_ptr);
    back_bf(1, 16, b_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 8 pt fdct -------------------------------*/
pub unsafe fn fdct8(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 8] = [0.0; 8]; /* ping pong buffers */
    let mut b: [f32; 8] = [0.0; 8];
    let mut p: c_int;
    let mut q: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage  */

    *b_ptr.offset(0) = *x.offset(0) + *x.offset(7);
    *b_ptr.offset(4) = *coef32_ptr.offset(16 + 8) * (*x.offset(0) - *x.offset(7));
    p = 1;
    q = 6;
    while p < 4 {
        *b_ptr.offset(p as isize) = *x.offset(p as isize) + *x.offset(q as isize);
        *b_ptr.offset((4 + p) as isize) = *coef32_ptr.offset((16 + 8 + p) as isize)
            * (*x.offset(p as isize) - *x.offset(q as isize));
        p += 1;
        q -= 1;
    }

    forward_bf(2, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(4, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(2, 4, b_ptr, a_ptr);
    back_bf(1, 8, a_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 8 pt fdct dual chan---------------------*/
pub unsafe fn fdct8_dual(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 8] = [0.0; 8]; /* ping pong buffers */
    let mut b: [f32; 8] = [0.0; 8];
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage for interleaved input */
    *b_ptr.offset(0) = *x.offset(0) + *x.offset(14);
    *b_ptr.offset(4) = *coef32_ptr.offset(16 + 8) * (*x.offset(0) - *x.offset(14));
    pp = 2;
    qq = 2 * 6;
    p = 1;
    while p < 4 {
        *b_ptr.offset(p as isize) = *x.offset(pp as isize) + *x.offset(qq as isize);
        *b_ptr.offset((4 + p) as isize) = *coef32_ptr.offset((16 + 8 + p) as isize)
            * (*x.offset(pp as isize) - *x.offset(qq as isize));
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(4, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(2, 4, b_ptr, a_ptr);
    back_bf(1, 8, a_ptr, c);
}

/*------------------------------------------------------------*/
/*---------------- 8 pt fdct dual to mono---------------------*/
pub unsafe fn fdct8_dual_mono(x: *mut f32, c: *mut f32) {
    let mut a: [f32; 8] = [0.0; 8]; /* ping pong buffers */
    let mut b: [f32; 8] = [0.0; 8];
    let mut t1: f32;
    let mut t2: f32;
    let mut p: c_int;
    let mut pp: c_int;
    let mut qq: c_int;

    let a_ptr = a.as_mut_ptr();
    let b_ptr = b.as_mut_ptr();
    let coef32_ptr = core::ptr::addr_of_mut!(coef32).cast::<f32>();

    /* special first stage  */
    t1 = 0.5f32 * (*x.offset(0) + *x.offset(1));
    t2 = 0.5f32 * (*x.offset(14) + *x.offset(15));
    *b_ptr.offset(0) = t1 + t2;
    *b_ptr.offset(4) = *coef32_ptr.offset(16 + 8) * (t1 - t2);
    pp = 2;
    qq = 2 * 6;
    p = 1;
    while p < 4 {
        t1 = 0.5f32 * (*x.offset(pp as isize) + *x.offset((pp + 1) as isize));
        t2 = 0.5f32 * (*x.offset(qq as isize) + *x.offset((qq + 1) as isize));
        *b_ptr.offset(p as isize) = t1 + t2;
        *b_ptr.offset((4 + p) as isize) =
            *coef32_ptr.offset((16 + 8 + p) as isize) * (t1 - t2);
        p += 1;
        pp += 2;
        qq -= 2;
    }
    forward_bf(2, 4, b_ptr, a_ptr, coef32_ptr.offset(16 + 8 + 4));
    forward_bf(4, 2, a_ptr, b_ptr, coef32_ptr.offset(16 + 8 + 4 + 2));
    back_bf(2, 4, b_ptr, a_ptr);
    back_bf(1, 8, a_ptr, c);
}

/*------------------------------------------------------------*/
