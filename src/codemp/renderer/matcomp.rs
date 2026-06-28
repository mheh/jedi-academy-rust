#![allow(non_snake_case)]

// From matcomp.h
const MC_BITS_X: usize = 16;
const MC_BITS_Y: usize = 16;
const MC_BITS_Z: usize = 16;
const MC_BITS_VECT: usize = 16;

const MC_SCALE_X: f32 = 1.0f32 / 64.0f32;
const MC_SCALE_Y: f32 = 1.0f32 / 64.0f32;
const MC_SCALE_Z: f32 = 1.0f32 / 64.0f32;

// currently 24 (.875)
const MC_COMP_BYTES: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*9)+7)/8;

const MC_MASK_X: u32 = (1u32 << MC_BITS_X) - 1;
const MC_MASK_Y: u32 = (1u32 << MC_BITS_Y) - 1;
const MC_MASK_Z: u32 = (1u32 << MC_BITS_Z) - 1;
const MC_MASK_VECT: u32 = (1u32 << MC_BITS_VECT) - 1;

const MC_SCALE_VECT: f32 = 1.0f32/(((1 << (MC_BITS_VECT-1))-2) as f32);

const MC_POS_X: usize = 0;
const MC_SHIFT_X: u32 = 0;

const MC_POS_Y: usize = ((MC_BITS_X)/8);
const MC_SHIFT_Y: u32 = ((MC_BITS_X%8)) as u32;

const MC_POS_Z: usize = ((MC_BITS_X+MC_BITS_Y)/8);
const MC_SHIFT_Z: u32 = ((MC_BITS_X+MC_BITS_Y)%8) as u32;

const MC_POS_V11: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z)/8);
const MC_SHIFT_V11: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z)%8) as u32;

const MC_POS_V12: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT)/8);
const MC_SHIFT_V12: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT)%8) as u32;

const MC_POS_V13: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*2)/8);
const MC_SHIFT_V13: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*2)%8) as u32;

const MC_POS_V21: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*3)/8);
const MC_SHIFT_V21: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*3)%8) as u32;

const MC_POS_V22: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*4)/8);
const MC_SHIFT_V22: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*4)%8) as u32;

const MC_POS_V23: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*5)/8);
const MC_SHIFT_V23: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*5)%8) as u32;

const MC_POS_V31: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*6)/8);
const MC_SHIFT_V31: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*6)%8) as u32;

const MC_POS_V32: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*7)/8);
const MC_SHIFT_V32: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*7)%8) as u32;

const MC_POS_V33: usize = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*8)/8);
const MC_SHIFT_V33: u32 = ((MC_BITS_X+MC_BITS_Y+MC_BITS_Z+MC_BITS_VECT*8)%8) as u32;

pub fn MC_Compress(mat: &[[f32; 4]; 3], _comp: *mut u8) {
	let mut comp = [0u8; MC_COMP_BYTES*2];

	let mut val: i32;

	val=(mat[0][3]/MC_SCALE_X) as i32;
	val+=1<<(MC_BITS_X-1);
	if val>=(1<<MC_BITS_X)
	{
		val=(1<<MC_BITS_X)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_X) as *mut u32)|=((val as u32))<<MC_SHIFT_X;
	}

	val=(mat[1][3]/MC_SCALE_Y) as i32;
	val+=1<<(MC_BITS_Y-1);
	if val>=(1<<MC_BITS_Y)
	{
		val=(1<<MC_BITS_Y)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_Y) as *mut u32)|=((val as u32))<<MC_SHIFT_Y;
	}

	val=(mat[2][3]/MC_SCALE_Z) as i32;
	val+=1<<(MC_BITS_Z-1);
	if val>=(1<<MC_BITS_Z)
	{
		val=(1<<MC_BITS_Z)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_Z) as *mut u32)|=((val as u32))<<MC_SHIFT_Z;
	}


	val=(mat[0][0]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V11) as *mut u32)|=((val as u32))<<MC_SHIFT_V11;
	}

	val=(mat[0][1]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V12) as *mut u32)|=((val as u32))<<MC_SHIFT_V12;
	}

	val=(mat[0][2]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V13) as *mut u32)|=((val as u32))<<MC_SHIFT_V13;
	}


	val=(mat[1][0]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V21) as *mut u32)|=((val as u32))<<MC_SHIFT_V21;
	}

	val=(mat[1][1]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V22) as *mut u32)|=((val as u32))<<MC_SHIFT_V22;
	}

	val=(mat[1][2]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V23) as *mut u32)|=((val as u32))<<MC_SHIFT_V23;
	}

	val=(mat[2][0]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V31) as *mut u32)|=((val as u32))<<MC_SHIFT_V31;
	}

	val=(mat[2][1]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V32) as *mut u32)|=((val as u32))<<MC_SHIFT_V32;
	}

	val=(mat[2][2]/MC_SCALE_VECT) as i32;
	val+=1<<(MC_BITS_VECT-1);
	if val>=(1<<MC_BITS_VECT)
	{
		val=(1<<MC_BITS_VECT)-1;
	}
	if val<0
	{
		val=0;
	}
	unsafe {
		*(comp.as_mut_ptr().add(MC_POS_V33) as *mut u32)|=((val as u32))<<MC_SHIFT_V33;
	}

	// I added this because the line above actually ORs data into an int at the 22 byte (from 0), and therefore technically
	//	is writing beyond the 24th byte of the output array. This *should** be harmless if the OR'd-in value doesn't change
	//	those bytes, but BoundsChecker says that it's accessing undefined memory (which it does, sometimes). This is probably
	//	bad, so...
	unsafe {
		core::ptr::copy_nonoverlapping(comp.as_ptr(), _comp, MC_COMP_BYTES);
	}
}

pub fn MC_UnCompress(mat: &mut [[f32; 4]; 3], comp: *const u8) {
	let mut val: i32;

	val=(unsafe { *(comp.cast::<u16>().add(0)) } as i32);
	val-=1<<(MC_BITS_X-1);
	mat[0][3]=((val) as f32)*MC_SCALE_X;

	val=(unsafe { *(comp.cast::<u16>().add(1)) } as i32);
	val-=1<<(MC_BITS_Y-1);
	mat[1][3]=((val) as f32)*MC_SCALE_Y;

	val=(unsafe { *(comp.cast::<u16>().add(2)) } as i32);
	val-=1<<(MC_BITS_Z-1);
	mat[2][3]=((val) as f32)*MC_SCALE_Z;

	val=(unsafe { *(comp.cast::<u16>().add(3)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[0][0]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(4)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[0][1]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(5)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[0][2]=((val) as f32)*MC_SCALE_VECT;


	val=(unsafe { *(comp.cast::<u16>().add(6)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[1][0]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(7)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[1][1]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(8)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[1][2]=((val) as f32)*MC_SCALE_VECT;


	val=(unsafe { *(comp.cast::<u16>().add(9)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[2][0]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(10)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[2][1]=((val) as f32)*MC_SCALE_VECT;

	val=(unsafe { *(comp.cast::<u16>().add(11)) } as i32);
	val-=1<<(MC_BITS_VECT-1);
	mat[2][2]=((val) as f32)*MC_SCALE_VECT;

}

pub fn MC_UnCompressQuat(mat: &mut [[f32; 4]; 3], comp: *const u8) {
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

	let pwIn = comp.cast::<u16>();
	let mut pwIn_idx = 0;

	w = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	w/=16383.0f32;
	w-=2.0f32;
	x = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	x/=16383.0f32;
	x-=2.0f32;
	y = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	y/=16383.0f32;
	y-=2.0f32;
	z = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	z/=16383.0f32;
	z-=2.0f32;

    fTx  = 2.0f32*x;
    fTy  = 2.0f32*y;
    fTz  = 2.0f32*z;
    fTwx = fTx*w;
    fTwy = fTy*w;
    fTwz = fTz*w;
    fTxx = fTx*x;
    fTxy = fTy*x;
    fTxz = fTz*x;
    fTyy = fTy*y;
    fTyz = fTz*y;
    fTzz = fTz*z;

	// rot...
	//
    mat[0][0] = 1.0f32-(fTyy+fTzz);
    mat[0][1] = fTxy-fTwz;
    mat[0][2] = fTxz+fTwy;
    mat[1][0] = fTxy+fTwz;
    mat[1][1] = 1.0f32-(fTxx+fTzz);
    mat[1][2] = fTyz-fTwx;
    mat[2][0] = fTxz-fTwy;
    mat[2][1] = fTyz+fTwx;
    mat[2][2] = 1.0f32-(fTxx+fTyy);

	// xlat...
	//
	f = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	f/=64.0f32;
	f-=512.0f32;
	mat[0][3] = f;

	f = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	pwIn_idx += 1;
	f/=64.0f32;
	f-=512.0f32;
	mat[1][3] = f;

	f = (unsafe { *pwIn.add(pwIn_idx) }) as f32;
	f/=64.0f32;
	f-=512.0f32;
	mat[2][3] = f;
}

