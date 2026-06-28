// tr_noise.c

use core::ffi::c_int;

const NOISE_SIZE: usize = 256;
const NOISE_MASK: usize = NOISE_SIZE - 1;

// Standard RAND_MAX value from libc
const RAND_MAX: c_int = 2147483647;

// Looks up s_noise_perm and masks with NOISE_MASK
#[inline]
fn VAL(a: usize) -> c_int {
	unsafe { s_noise_perm[a & NOISE_MASK] }
}

// Nested VAL lookups to compute an index: VAL( x + VAL( y + VAL( z + VAL( t ) ) ) )
#[inline]
fn INDEX(x: c_int, y: c_int, z: c_int, t: c_int) -> usize {
	let t_val = VAL(t as usize);
	let z_val = VAL((z + t_val) as usize);
	let y_val = VAL((y + z_val) as usize);
	VAL((x + y_val) as usize) as usize
}

// Linear interpolation
#[inline]
fn LERP(a: f32, b: f32, w: f32) -> f32 {
	a * (1.0f32 - w) + b * w
}

static mut s_noise_table: [f32; NOISE_SIZE] = [0.0; NOISE_SIZE];
static mut s_noise_perm: [c_int; NOISE_SIZE] = [0; NOISE_SIZE];

extern "C" {
	fn srand(seed: c_int);
	fn rand() -> c_int;
}

fn GetNoiseValue(x: c_int, y: c_int, z: c_int, t: c_int) -> f32 {
	let index = INDEX(x, y, z, t);

	unsafe { s_noise_table[index] }
}

pub fn GetNoiseTime(t: c_int) -> f32 {
	let index = VAL(t as usize) as usize;

	unsafe { 1.0 + s_noise_table[index] }
}

pub fn R_NoiseInit() {
	unsafe {
		srand(1001);

		for i in 0..NOISE_SIZE {
			s_noise_table[i] = ((rand() as f32 / RAND_MAX as f32) * 2.0 - 1.0);
			s_noise_perm[i] = ((rand() as f32 / RAND_MAX as f32) * 255.0) as u8 as c_int;
		}
	}
}

pub fn R_NoiseGet4f(x: f32, y: f32, z: f32, t: f32) -> f32 {
	let ix = x.floor() as c_int;
	let fx = x - ix as f32;
	let iy = y.floor() as c_int;
	let fy = y - iy as f32;
	let iz = z.floor() as c_int;
	let fz = z - iz as f32;
	let it = t.floor() as c_int;
	let ft = t - it as f32;

	let mut front: [f32; 4] = [0.0; 4];
	let mut back: [f32; 4] = [0.0; 4];
	let mut value: [f32; 2] = [0.0; 2];

	for i in 0..2 {
		front[0] = GetNoiseValue(ix, iy, iz, it + i as c_int);
		front[1] = GetNoiseValue(ix + 1, iy, iz, it + i as c_int);
		front[2] = GetNoiseValue(ix, iy + 1, iz, it + i as c_int);
		front[3] = GetNoiseValue(ix + 1, iy + 1, iz, it + i as c_int);

		back[0] = GetNoiseValue(ix, iy, iz + 1, it + i as c_int);
		back[1] = GetNoiseValue(ix + 1, iy, iz + 1, it + i as c_int);
		back[2] = GetNoiseValue(ix, iy + 1, iz + 1, it + i as c_int);
		back[3] = GetNoiseValue(ix + 1, iy + 1, iz + 1, it + i as c_int);

		let fvalue = LERP(LERP(front[0], front[1], fx), LERP(front[2], front[3], fx), fy);
		let bvalue = LERP(LERP(back[0], back[1], fx), LERP(back[2], back[3], fx), fy);

		value[i] = LERP(fvalue, bvalue, fz);
	}

	let finalvalue = LERP(value[0], value[1], ft);

	finalvalue
}
