#![allow(non_snake_case)]

use core::ptr;

// Constants from MatComp.h
const MC_BITS_X: i32 = 16;
const MC_BITS_Y: i32 = 16;
const MC_BITS_Z: i32 = 16;
const MC_BITS_VECT: i32 = 16;

const MC_SCALE_X: f32 = 1.0 / 64.0;
const MC_SCALE_Y: f32 = 1.0 / 64.0;
const MC_SCALE_Z: f32 = 1.0 / 64.0;

// MC_COMP_BYTES = ((16+16+16+16*9)+7)/8 = 24
const MC_COMP_BYTES: usize = 24;

// Mask calculations (translated from macros)
const MC_MASK_X: u32 = (1u32 << MC_BITS_X) - 1;
const MC_MASK_Y: u32 = (1u32 << MC_BITS_Y) - 1;
const MC_MASK_Z: u32 = (1u32 << MC_BITS_Z) - 1;
const MC_MASK_VECT: u32 = (1u32 << MC_BITS_VECT) - 1;

const MC_SCALE_VECT: f32 = 1.0 / (((1i32 << (MC_BITS_VECT - 1)) - 2) as f32);

// Position and shift calculations (translated from macros)
const MC_POS_X: usize = 0;
const MC_SHIFT_X: u32 = 0;

const MC_POS_Y: usize = (MC_BITS_X / 8) as usize;
const MC_SHIFT_Y: u32 = (MC_BITS_X % 8) as u32;

const MC_POS_Z: usize = ((MC_BITS_X + MC_BITS_Y) / 8) as usize;
const MC_SHIFT_Z: u32 = ((MC_BITS_X + MC_BITS_Y) % 8) as u32;

const MC_POS_V11: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z) / 8) as usize;
const MC_SHIFT_V11: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z) % 8) as u32;

const MC_POS_V12: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT) / 8) as usize;
const MC_SHIFT_V12: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT) % 8) as u32;

const MC_POS_V13: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 2) / 8) as usize;
const MC_SHIFT_V13: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 2) % 8) as u32;

const MC_POS_V21: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 3) / 8) as usize;
const MC_SHIFT_V21: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 3) % 8) as u32;

const MC_POS_V22: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 4) / 8) as usize;
const MC_SHIFT_V22: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 4) % 8) as u32;

const MC_POS_V23: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 5) / 8) as usize;
const MC_SHIFT_V23: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 5) % 8) as u32;

const MC_POS_V31: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 6) / 8) as usize;
const MC_SHIFT_V31: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 6) % 8) as u32;

const MC_POS_V32: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 7) / 8) as usize;
const MC_SHIFT_V32: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 7) % 8) as u32;

const MC_POS_V33: usize = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 8) / 8) as usize;
const MC_SHIFT_V33: u32 = ((MC_BITS_X + MC_BITS_Y + MC_BITS_Z + MC_BITS_VECT * 8) % 8) as u32;

#[no_mangle]
pub unsafe extern "C" fn MC_Compress(mat: *const [[f32; 4]; 3], _comp: *mut u8) {
    let mut comp = [0u8; MC_COMP_BYTES * 2];
    let mat = &*mat;

    let mut val: i32;

    val = (mat[0][3] / MC_SCALE_X) as i32;
    val += 1 << (MC_BITS_X - 1);
    if val >= (1 << MC_BITS_X) {
        val = (1 << MC_BITS_X) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_X) as *mut u32) |= (val as u32) << MC_SHIFT_X;

    val = (mat[1][3] / MC_SCALE_Y) as i32;
    val += 1 << (MC_BITS_Y - 1);
    if val >= (1 << MC_BITS_Y) {
        val = (1 << MC_BITS_Y) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_Y) as *mut u32) |= (val as u32) << MC_SHIFT_Y;

    val = (mat[2][3] / MC_SCALE_Z) as i32;
    val += 1 << (MC_BITS_Z - 1);
    if val >= (1 << MC_BITS_Z) {
        val = (1 << MC_BITS_Z) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_Z) as *mut u32) |= (val as u32) << MC_SHIFT_Z;

    val = (mat[0][0] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V11) as *mut u32) |= (val as u32) << MC_SHIFT_V11;

    val = (mat[0][1] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V12) as *mut u32) |= (val as u32) << MC_SHIFT_V12;

    val = (mat[0][2] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V13) as *mut u32) |= (val as u32) << MC_SHIFT_V13;

    val = (mat[1][0] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V21) as *mut u32) |= (val as u32) << MC_SHIFT_V21;

    val = (mat[1][1] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V22) as *mut u32) |= (val as u32) << MC_SHIFT_V22;

    val = (mat[1][2] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V23) as *mut u32) |= (val as u32) << MC_SHIFT_V23;

    val = (mat[2][0] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V31) as *mut u32) |= (val as u32) << MC_SHIFT_V31;

    val = (mat[2][1] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V32) as *mut u32) |= (val as u32) << MC_SHIFT_V32;

    val = (mat[2][2] / MC_SCALE_VECT) as i32;
    val += 1 << (MC_BITS_VECT - 1);
    if val >= (1 << MC_BITS_VECT) {
        val = (1 << MC_BITS_VECT) - 1;
    }
    if val < 0 {
        val = 0;
    }
    *(comp.as_mut_ptr().add(MC_POS_V33) as *mut u32) |= (val as u32) << MC_SHIFT_V33;

    // I added this because the line above actually ORs data into an int at the 22 byte (from 0), and therefore technically
    //	is writing beyond the 24th byte of the output array. This *should** be harmless if the OR'd-in value doesn't change
    //	those bytes, but BoundsChecker says that it's accessing undefined memory (which it does, sometimes). This is probably
    //	bad, so...
    ptr::copy_nonoverlapping(comp.as_ptr(), _comp, MC_COMP_BYTES);
}

#[no_mangle]
pub unsafe extern "C" fn MC_UnCompress(mat: *mut [[f32; 4]; 3], comp: *const u8) {
    let mat = &mut *mat;
    let pwIn = comp as *const u16;

    let mut val: i32;

    val = *pwIn.add(0) as i32;
    val -= 1 << (MC_BITS_X - 1);
    mat[0][3] = (val as f32) * MC_SCALE_X;

    val = *pwIn.add(1) as i32;
    val -= 1 << (MC_BITS_Y - 1);
    mat[1][3] = (val as f32) * MC_SCALE_Y;

    val = *pwIn.add(2) as i32;
    val -= 1 << (MC_BITS_Z - 1);
    mat[2][3] = (val as f32) * MC_SCALE_Z;

    val = *pwIn.add(3) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[0][0] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(4) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[0][1] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(5) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[0][2] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(6) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[1][0] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(7) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[1][1] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(8) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[1][2] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(9) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[2][0] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(10) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[2][1] = (val as f32) * MC_SCALE_VECT;

    val = *pwIn.add(11) as i32;
    val -= 1 << (MC_BITS_VECT - 1);
    mat[2][2] = (val as f32) * MC_SCALE_VECT;
}

#[no_mangle]
pub unsafe extern "C" fn MC_UnCompressQuat(mat: *mut [[f32; 4]; 3], comp: *const u8) {
    let mat = &mut *mat;

    let mut w: f32;
    let mut x: f32;
    let mut y: f32;
    let mut z: f32;
    let mut f: f32;
    let mut fTx: f32;
    let mut fTy: f32;
    let mut fTz: f32;
    let mut fTwx: f32;
    let mut fTwy: f32;
    let mut fTwz: f32;
    let mut fTxx: f32;
    let mut fTxy: f32;
    let mut fTxz: f32;
    let mut fTyy: f32;
    let mut fTyz: f32;
    let mut fTzz: f32;

    let pwIn = comp as *const u16;

    /* RTCDC
    if(use_comp_rot_mask)
    {
        cw = *pwIn++;
        cx = *pwIn++;
        cy = *pwIn++;
        cz = *pwIn++;

        cw &= comp_rot_maskw;
        cx &= comp_rot_maskx;
        cy &= comp_rot_masky;
        cz &= comp_rot_maskz;
    }
    else
    {
        cw = *pwIn++;
        cx = *pwIn++;
        cy = *pwIn++;
        cz = *pwIn++;
    }

    if(use_comp_tra_mask)
    {
        f1 = *pwIn++;
        f2 = *pwIn++;
        f3 = *pwIn++;

        f1 &= comp_tra_maskf1;
        f2 &= comp_tra_maskf2;
        f3 &= comp_tra_maskf3;
    }
    else
    {
        f1 = *pwIn++;
        f2 = *pwIn++;
        f3 = *pwIn++;
    }
    */

    w = *pwIn as f32;
    // RTCDC w = cw;
    w /= 16383.0;
    w -= 2.0;
    x = *pwIn.add(1) as f32;
    // RTCDC x = cx;
    x /= 16383.0;
    x -= 2.0;
    y = *pwIn.add(2) as f32;
    // RTCDC y = cy;
    y /= 16383.0;
    y -= 2.0;
    z = *pwIn.add(3) as f32;
    // RTCDC z = cz;
    z /= 16383.0;
    z -= 2.0;

    fTx = 2.0 * x;
    fTy = 2.0 * y;
    fTz = 2.0 * z;
    fTwx = fTx * w;
    fTwy = fTy * w;
    fTwz = fTz * w;
    fTxx = fTx * x;
    fTxy = fTy * x;
    fTxz = fTz * x;
    fTyy = fTy * y;
    fTyz = fTz * y;
    fTzz = fTz * z;

    // rot...
    //
    mat[0][0] = 1.0 - (fTyy + fTzz);
    mat[0][1] = fTxy - fTwz;
    mat[0][2] = fTxz + fTwy;
    mat[1][0] = fTxy + fTwz;
    mat[1][1] = 1.0 - (fTxx + fTzz);
    mat[1][2] = fTyz - fTwx;
    mat[2][0] = fTxz - fTwy;
    mat[2][1] = fTyz + fTwx;
    mat[2][2] = 1.0 - (fTxx + fTyy);

    // xlat...
    //
    f = *pwIn.add(4) as f32;
    // RTCDC f = f1;
    f /= 64.0;
    f -= 512.0;
    mat[0][3] = f;

    f = *pwIn.add(5) as f32;
    // RTCDC f = f2;
    f /= 64.0;
    f -= 512.0;
    mat[1][3] = f;

    f = *pwIn.add(6) as f32;
    // RTCDC f = f3;
    f /= 64.0;
    f -= 512.0;
    mat[2][3] = f;
}
