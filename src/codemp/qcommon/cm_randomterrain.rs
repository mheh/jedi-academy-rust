//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

//! Random terrain generation implementation.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, vec2_t, vec4_t, vec_t};
use crate::codemp::qcommon::cm_randomterrain_h::*;
use crate::codemp::qcommon::cm_terrainmap_h::CCMLandScape;
use crate::codemp::qcommon::q_math_h::{Com_Clamp, Round};
use core::ffi::{c_int, c_char, c_uint};
use core::mem::size_of;

const NOISE_SIZE: usize = 256;
const NOISE_MASK: usize = NOISE_SIZE - 1;

static mut noiseTable: [f32; NOISE_SIZE] = [0.0; NOISE_SIZE];
static mut noisePerm: [c_int; NOISE_SIZE] = [0; NOISE_SIZE];

// #if 0
// static void CM_NoiseInit( CCMLandScape *landscape )
// {
// 	int		i;
//
// 	for ( i = 0; i < NOISE_SIZE; i++ )
// 	{
// 		noiseTable[i] = landscape->flrand(-1.0f, 1.0f);
// 		noisePerm[i] = (byte)landscape->irand(0, 255);
// 	}
// }
// #endif

// #define VAL( a ) noisePerm[ ( a ) & ( NOISE_MASK )]
#[inline]
fn VAL(a: c_int) -> c_int {
	unsafe { noisePerm[(a as usize) & NOISE_MASK] }
}

// #define INDEX( x, y, z, t ) VAL( x + VAL( y + VAL( z + VAL( t ) ) ) )
#[inline]
fn INDEX(x: c_int, y: c_int, z: c_int, t: c_int) -> c_int {
	VAL(x + VAL(y + VAL(z + VAL(t))))
}

// #define LERP( a, b, w ) ( a * ( 1.0f - w ) + b * w )
#[inline]
fn LERP(a: f32, b: f32, w: f32) -> f32 {
	a * (1.0f32 - w) + b * w
}

fn GetNoiseValue(x: c_int, y: c_int, z: c_int, t: c_int) -> f32 {
	let index = INDEX(x, y, z, t);
	unsafe { noiseTable[index as usize] }
}

// #if 0
// static float GetNoiseTime( int t )
// {
// 	int index = VAL( t );
//
// 	return (1 + noiseTable[index]);
// }
// #endif

fn CM_NoiseGet4f(mut x: f32, mut y: f32, mut z: f32, mut t: f32) -> f32 {
	let mut i: c_int;
	let mut ix: c_int;
	let mut iy: c_int;
	let mut iz: c_int;
	let mut it: c_int;
	let fx: f32;
	let fy: f32;
	let fz: f32;
	let ft: f32;
	let mut front: [f32; 4] = [0.0; 4];
	let mut back: [f32; 4] = [0.0; 4];
	let mut fvalue: f32;
	let mut bvalue: f32;
	let mut value: [f32; 2] = [0.0; 2];
	let mut finalvalue: f32;

	ix = x.floor() as c_int;
	fx = x - ix as f32;
	iy = y.floor() as c_int;
	fy = y - iy as f32;
	iz = z.floor() as c_int;
	fz = z - iz as f32;
	it = t.floor() as c_int;
	ft = t - it as f32;

	i = 0;
	while i < 2 {
		front[0] = GetNoiseValue(ix, iy, iz, it + i);
		front[1] = GetNoiseValue(ix + 1, iy, iz, it + i);
		front[2] = GetNoiseValue(ix, iy + 1, iz, it + i);
		front[3] = GetNoiseValue(ix + 1, iy + 1, iz, it + i);

		back[0] = GetNoiseValue(ix, iy, iz + 1, it + i);
		back[1] = GetNoiseValue(ix + 1, iy, iz + 1, it + i);
		back[2] = GetNoiseValue(ix, iy + 1, iz + 1, it + i);
		back[3] = GetNoiseValue(ix + 1, iy + 1, iz + 1, it + i);

		fvalue = LERP(LERP(front[0], front[1], fx), LERP(front[2], front[3], fx), fy);
		bvalue = LERP(LERP(back[0], back[1], fx), LERP(back[2], back[3], fx), fy);

		value[i as usize] = LERP(fvalue, bvalue, fz);

		i += 1;
	}

	finalvalue = LERP(value[0], value[1], ft);
	finalvalue
}

/****** lincrv.c ******/
/* Ken Shoemake, 1994 */

/* Perform a generic vector unary operation. */
// #define V_Op(vdst,gets,vsrc,n) {register int V_i;\
//     for(V_i=(n)-1;V_i>=0;V_i--) (vdst)[V_i] gets ((vsrc)[V_i]);}

fn V_Op_assign(vdst: &mut vec4_t, vsrc: &vec4_t, n: usize) {
	let mut V_i = n as i32 - 1;
	while V_i >= 0 {
		vdst[V_i as usize] = vsrc[V_i as usize];
		V_i -= 1;
	}
}

fn lerp(t: f32, a0: f32, a1: f32, p0: &vec4_t, p1: &vec4_t, m: usize, p: &mut vec4_t) {
	let t0 = (a1 - t) / (a1 - a0);
	let t1 = 1.0f32 - t0;
	let mut i = (m as i32) - 1;
	while i >= 0 {
		p[i as usize] = t0 * p0[i as usize] + t1 * p1[i as usize];
		i -= 1;
	}
}

/* DialASpline(t,a,p,m,n,work,Cn,interp,val) computes a point val at parameter
    t on a spline with knot values a and control points p. The curve will have
    Cn continuity, and if interp is TRUE it will interpolate the control points.
    Possibilities include Langrange interpolants, Bezier curves, Catmull-Rom
    interpolating splines, and B-spline curves. Points have m coordinates, and
    n+1 of them are provided. The work array must have room for n+1 points.
 */
fn DialASpline(t: f32, a: &[f32], p: &[vec4_t], m: usize, n: usize, work: &mut [vec4_t],
				mut Cn: c_uint, interp: bool, val: &mut vec4_t) -> c_int {
	let mut i: c_int;
	let mut j: c_int;
	let mut k: c_int;
	let mut h: c_int;
	let mut lo: c_int;
	let mut hi: c_int;

	if Cn > (n as c_uint) - 1 {
		Cn = (n as c_uint) - 1;       /* Anything greater gives one polynomial */
	}
	k = 0;
	while t > a[k as usize] {
		k += 1;    /* Find enclosing knot interval */
	}
	h = k;
	while t == a[k as usize] {
		k += 1;    /* May want to use fewer legs */
	}
	if k > n as c_int {
		k = n as c_int;
		if h > k {
			h = k;
		}
	}
	h = 1 + Cn as c_int - (k - h);
	k -= 1;
	lo = k - Cn as c_int;
	hi = k + 1 + Cn as c_int;

	if interp {               /* Lagrange interpolation steps */
		let mut drop = 0;
		if lo < 0 {
			lo = 0;
			drop += Cn as c_int - k;
			if hi - lo < Cn as c_int {
				drop += Cn as c_int - hi;
				hi = Cn as c_int;
			}
		}
		if hi > n as c_int {
			hi = n as c_int;
			drop += k + 1 + Cn as c_int - n as c_int;
			if hi - lo < Cn as c_int {
				drop += lo - (n as c_int - Cn as c_int);
				lo = n as c_int - Cn as c_int;
			}
		}
		i = lo;
		while i <= hi {
			V_Op_assign(&mut work[i as usize], &p[i as usize], m);
			i += 1;
		}
		j = 1;
		while j <= Cn as c_int {
			i = lo;
			while i <= hi - j {
				lerp(t, a[i as usize], a[(i + j) as usize], &work[i as usize], &work[(i + 1) as usize], m, &mut work[i as usize]);
				i += 1;
			}
			j += 1;
		}
		h = 1 + Cn as c_int - drop;
	} else {                    /* Prepare for B-spline steps */
		if lo < 0 {
			h += lo;
			lo = 0;
		}
		i = lo;
		while i <= lo + h {
			V_Op_assign(&mut work[i as usize], &p[i as usize], m);
			i += 1;
		}
		if h < 0 {
			h = 0;
		}
	}
	j = 0;
	while j < h {
		let tmp = 1 + Cn as c_int - j;
		i = h - 1;
		while i >= j {
			lerp(t, a[(lo as usize) + i as usize], a[(lo as usize) + i as usize + tmp as usize], &work[(lo as usize) + i as usize], &work[(lo as usize) + i as usize + 1], m, &mut work[(lo as usize) + i as usize + 1]);
			i -= 1;
		}
		j += 1;
	}
	V_Op_assign(val, &work[(lo as usize) + h as usize], m);
	k
}

const BIG: f32 = 1.0e12;

fn Vector2Normalize(v: &mut vec2_t) -> f32 {
	let mut length: f32;
	let ilength: f32;

	length = v[0] * v[0] + v[1] * v[1];
	length = length.sqrt();

	if length > 0.0 {
		ilength = 1.0f32 / length;
		v[0] *= ilength;
		v[1] *= ilength;
	}

	length
}

// CPathInfo struct is defined in cm_randomterrain_h.rs
// Rust doesn't have automatic constructors like C++, so we implement methods here
impl CPathInfo {
	pub fn new(landscape: *mut CCMLandScape, numPoints: c_int, bx: f32, by: f32, ex: f32, ey: f32,
				minWidth: f32, maxWidth: f32, depth: f32, deviation: f32, breadth: f32,
				Connected: *mut CPathInfo, CreationFlags: c_uint) -> Box<Self> {
		let mut path = Box::new(CPathInfo {
			mPoints: std::ptr::null_mut(),
			mWork: std::ptr::null_mut(),
			mWeights: std::ptr::null_mut(),
			mNumPoints: numPoints,
			mMinWidth: minWidth,
			mMaxWidth: maxWidth,
			mInc: 0.0,
			mDepth: depth,
			mBreadth: breadth,
			mDeviation: deviation,
			mCircleStamp: [[0; CIRCLE_STAMP_SIZE]; CIRCLE_STAMP_SIZE],
		});

		let mut i: c_int;
		let mut numConnected: c_int;
		let mut index: c_int;
		let mut point: *const f32;
		let mut currentWidth: f32;
		let mut currentPosition: f32;
		let mut testPoint: vec2_t = [0.0; 2];
		let mut percPoint: vec2_t = [0.0; 2];
		let mut diffPoint: vec2_t = [0.0; 2];
		let mut normalizedPath: vec2_t = [0.0; 2];
		let mut distance: f32;

		path.CreateCircle();

		numConnected = -1;
		if !Connected.is_null() {	// we are connecting to an existing spline
			unsafe {
				numConnected = (*Connected).GetNumPoints();
			}
			if numConnected >= SPLINE_MERGE_SIZE as c_int {	// plenty of points to choose from
				path.mNumPoints += SPLINE_MERGE_SIZE as c_int;
			} else {	// the existing spline doesn't have enough points
				path.mNumPoints += numConnected;
			}
		}

		unsafe {
			path.mPoints = libc::malloc(size_of::<vec4_t>() * path.mNumPoints as usize) as *mut vec4_t;
			path.mWork = libc::malloc(size_of::<vec4_t>() * (path.mNumPoints + 1) as usize) as *mut vec4_t;
			path.mWeights = libc::malloc(size_of::<f32>() * (path.mNumPoints + 1) as usize) as *mut f32;
		}

		let _length = ((ex - bx) * (ex - bx) + (ey - by) * (ey - by)).sqrt();
		let horizontal: bool;
		let mut position: f32;
		let goal: f32;
		let deltaGoal: f32;
		if (ex - bx).abs() >= (ey - by).abs() {	// this appears to be a horizontal path
			path.mInc = 1.0f32 / (ex - bx).abs();
			horizontal = true;
			position = by;
			goal = ey;
			deltaGoal = (ey - by) / (numPoints - 1) as f32;
		} else {	// this appears to be a vertical path
			path.mInc = 1.0f32 / (ey - by).abs();
			horizontal = false;
			position = bx;
			goal = ex;
			deltaGoal = (ex - bx) / (numPoints - 1) as f32;
		}
		normalizedPath[0] = ex - bx;
		normalizedPath[1] = ey - by;
		Vector2Normalize(&mut normalizedPath);
		// approx calculate how much we need to iterate through the spline to hit every point
		path.mInc /= 16.0;

		let mut currentWidth = unsafe {
			(*landscape).flrand(minWidth, maxWidth)
		};
		let mut currentPosition = 0.0f32;

		i = 0;
		while i < path.mNumPoints {
			// weights are evenly distributed
			unsafe {
				*path.mWeights.offset(i as isize) = i as f32 / (path.mNumPoints - 1) as f32;
			}

			if i < numConnected && i < SPLINE_MERGE_SIZE as c_int {	// we are connecting to an existing spline, so copy over the first few points
				if (CreationFlags & PATH_CREATION_CONNECT_FRONT) != 0 {	// copy from the front
					index = i;
				} else {	// copy from the end
					index = numConnected - SPLINE_MERGE_SIZE as c_int + i;
				}
				unsafe {
					point = (*Connected).GetPoint(index as usize);
					(*path.mPoints.offset(i as isize))[0] = *point;
					(*path.mPoints.offset(i as isize))[1] = *point.offset(1);
					(*path.mPoints.offset(i as isize))[3] = *point.offset(3);
				}
			} else {
				if horizontal {	// we appear to be going horizontal, so spread the randomness across the vertical
					unsafe {
						(*path.mPoints.offset(i as isize))[0] = ((ex - bx) * currentPosition) + bx;
						(*path.mPoints.offset(i as isize))[1] = position;
					}
				} else {	// we appear to be going vertical, so spread the randomness across the horizontal
					unsafe {
						(*path.mPoints.offset(i as isize))[0] = position;
						(*path.mPoints.offset(i as isize))[1] = ((ey - by) * currentPosition) + by;
					}
				}
				currentPosition += 1.0f32 / (numPoints - 1) as f32;

				// set the width of the spline
				unsafe {
					(*path.mPoints.offset(i as isize))[3] = currentWidth;
				}
				currentWidth += unsafe {
					(*landscape).flrand(-0.10, 0.10)
				};
				if currentWidth < minWidth {
					currentWidth = minWidth;
				} else if currentWidth > maxWidth {
					currentWidth = maxWidth;
				}

				// see how far we are from the goal
				//			delta = (goal - position) * currentPosition;
				// calculate the randomness we are allowed at this place
				// random = landscape->flrand(-mDeviation/1.0, mDeviation/1.0) * (1.0 - currentPosition);
				// position += delta + random;

				if i == path.mNumPoints - 2 {	// -2 because we are calculating for the next point
					position = goal;
				} else {
					if i == 0 {
						position += deltaGoal + unsafe {
							(*landscape).flrand(-deviation / 10.0, deviation / 10.0)
						};
					} else {
						position += deltaGoal + unsafe {
							(*landscape).flrand(-deviation * 1.5, deviation * 1.5)
						};
					}
				}

				if position > 0.9 {	// too far over, so move back a bit
					position = 0.9 - unsafe {
						(*landscape).flrand(0.02, 0.1)
					};
				}
				if position < 0.1 {	// too near, so move bakc a bit
					position = 0.1 + unsafe {
						(*landscape).flrand(0.02, 0.1)
					};
				}

				// check our deviation from the straight line to the end
				if horizontal {
					testPoint[0] = ((ex - bx) * currentPosition) + bx;
					testPoint[1] = position;
				} else {
					testPoint[0] = position;
					testPoint[1] = ((ey - by) * currentPosition) + by;
				}
				// dot product of the normal of the path to the point we are at
				distance = ((testPoint[0] - bx) * normalizedPath[0]) + ((testPoint[1] - by) * normalizedPath[1]);
				// find the perpendicular place that is intersected by the point and the path
				percPoint[0] = (distance * normalizedPath[0]) + bx;
				percPoint[1] = (distance * normalizedPath[1]) + by;
				// calculate the difference between the perpendicular point and the test point
				diffPoint[0] = testPoint[0] - percPoint[0];
				diffPoint[1] = testPoint[1] - percPoint[1];
				// calculate the distance
				distance = ((diffPoint[0] * diffPoint[0]) + (diffPoint[1] * diffPoint[1])).sqrt();
				if distance > deviation {	// we are beyond our allowed deviation, so head back
					if horizontal {
						position = (ey - by) * currentPosition + by;
					} else {
						position = (ex - bx) * currentPosition + bx;
					}
					position += unsafe {
						(*landscape).flrand(-deviation / 2.0, deviation / 2.0)
					};
				}
			}

			i += 1;
		}
		unsafe {
			*path.mWeights.offset(path.mNumPoints as isize) = BIG;
		}

		path
	}

	fn CreateCircle(&mut self) {
		let mut x: c_int;
		let mut y: c_int;
		let mut r: f32;
		let mut d: f32;

		// memset(mCircleStamp, 0, sizeof(mCircleStamp));
		for i in 0..(CIRCLE_STAMP_SIZE * CIRCLE_STAMP_SIZE) {
			self.mCircleStamp[i / CIRCLE_STAMP_SIZE][i % CIRCLE_STAMP_SIZE] = 0;
		}

		r = CIRCLE_STAMP_SIZE as f32;
		x = 0;
		while x < CIRCLE_STAMP_SIZE as c_int {
			y = 0;
			while y < CIRCLE_STAMP_SIZE as c_int {
				d = ((x * x + y * y) as f32).sqrt();
				if d > r {
					self.mCircleStamp[y as usize][x as usize] = 255;
				} else {
					self.mCircleStamp[y as usize][x as usize] = ((d / r * std::f32::consts::PI / 2.0).sin().powf(self.mBreadth) * 255.0) as byte;
				}
				y += 1;
			}
			x += 1;
		}
	}

	fn Stamp(&self, x: c_int, y: c_int, size: c_int, depth: c_int, Data: *mut byte, DataWidth: c_int, DataHeight: c_int) {
		//	int xPos;
		//	float yPos;
		let mut dx: c_int;
		let mut dy: c_int;
		let mut fx: c_int;
		let mut fy: c_int;
		let offset: f32;
		let mut value: byte;
		let invDepth: byte;

		offset = (CIRCLE_STAMP_SIZE as f32 - 1.0) / size as f32;
		invDepth = (255 - depth) as byte;

		dx = -size;
		while dx <= size {
			dy = -size;
			while dy <= size {
				let d: f32;

				d = (dx * dx + dy * dy) as f32;
				if d > (size * size) as f32 {
					dy += 1;
					continue;
				}

				fx = x + dx;
				if fx < 2 || fx > DataWidth - 2 {
					dy += 1;
					continue;
				}

				fy = y + dy;
				if fy < 2 || fy > DataHeight - 2 {
					dy += 1;
					continue;
				}

				value = ((d / ((size * size) as f32) * std::f32::consts::PI / 2.0).sin().powf(self.mBreadth) * invDepth as f32 + depth as f32) as byte;
				unsafe {
					let idx = (fy * DataWidth + fx) as usize;
					if value < (*Data.offset(idx as isize)) {
						*Data.offset(idx as isize) = value;
					}
				}

				dy += 1;
			}
			dx += 1;
		}
	}

	fn GetNumPoints(&self) -> c_int {
		self.mNumPoints
	}

	fn GetPoint(&self, index: usize) -> *const f32 {
		unsafe {
			&(*self.mPoints.offset(index as isize))[0] as *const f32
		}
	}

	pub fn GetInfo(&self, PercentInto: f32, Coord: &mut vec4_t, Vector: &mut vec4_t) {
		let mut before: vec4_t = [0.0; 4];
		let mut after: vec4_t = [0.0; 4];
		let mut testPercent: f32;

		let weights_slice = unsafe {
			std::slice::from_raw_parts(self.mWeights, (self.mNumPoints + 1) as usize)
		};
		let points_slice = unsafe {
			std::slice::from_raw_parts(self.mPoints, self.mNumPoints as usize)
		};
		let work_slice = unsafe {
			std::slice::from_raw_parts_mut(self.mWork, (self.mNumPoints + 1) as usize)
		};

		DialASpline(PercentInto, weights_slice, points_slice, size_of::<vec4_t>() / size_of::<f32>(), (self.mNumPoints - 1) as usize, work_slice, 2, true, Coord);

		testPercent = PercentInto - 0.01;
		if testPercent < 0.0 {
			testPercent = 0.0;
		}
		DialASpline(testPercent, weights_slice, points_slice, size_of::<vec4_t>() / size_of::<f32>(), (self.mNumPoints - 1) as usize, work_slice, 2, true, &mut before);

		testPercent = PercentInto + 0.01;
		if testPercent > 1.0 {
			testPercent = 1.0;
		}
		DialASpline(testPercent, weights_slice, points_slice, size_of::<vec4_t>() / size_of::<f32>(), (self.mNumPoints - 1) as usize, work_slice, 2, true, &mut after);

		Coord[2] = self.mDepth;

		Vector[0] = after[0] - before[0];
		Vector[1] = after[1] - before[1];
	}

	pub fn DrawPath(&self, Data: *mut byte, DataWidth: c_int, DataHeight: c_int) {
		let mut t: f32;
		let mut val: vec4_t = [0.0; 4];
		let mut vector: vec4_t = [0.0; 4];
		let mut size: c_int;
		let inc: f32;
		let mut x: c_int;
		let mut y: c_int;
		let mut lastX: c_int;
		let mut lastY: c_int;
		let depth: f32;

		let inc = self.mInc / DataWidth as f32;

		lastX = -999;
		lastY = -999;

		t = 0.0;
		while t <= 1.0 {
			self.GetInfo(t, &mut val, &mut vector);

			x = (val[0] * DataWidth as f32) as c_int;
			y = (val[1] * DataHeight as f32) as c_int;

			if x == lastX && y == lastY {
				t += inc;
				continue;
			}

			lastX = x;
			lastY = y;

			size = (val[3] * DataWidth as f32) as c_int;

			let depth = self.mDepth * 255.0f32;

			self.Stamp(x, y, size, depth as c_int, Data, DataWidth, DataHeight);

			t += inc;
		}
	}
}

impl Drop for CPathInfo {
	fn drop(&mut self) {
		unsafe {
			libc::free(self.mWeights as *mut core::ffi::c_void);
			libc::free(self.mWork as *mut core::ffi::c_void);
			libc::free(self.mPoints as *mut core::ffi::c_void);
		}
	}
}

// CRandomTerrain struct is defined in cm_randomterrain_h.rs
impl CRandomTerrain {
	pub fn new() -> Self {
		CRandomTerrain {
			mLandScape: std::ptr::null_mut(),
			mWidth: 0,
			mHeight: 0,
			mArea: 0,
			mBorder: 0,
			mGrid: std::ptr::null_mut(),
			mPaths: [std::ptr::null_mut(); MAX_RANDOM_PATHS],
		}
	}

	pub fn Init(&mut self, landscape: *mut CCMLandScape, grid: *mut byte, width: c_int, height: c_int) {
		self.Shutdown();
		self.mLandScape = landscape;
		self.mWidth = width;
		self.mHeight = height;
		self.mArea = self.mWidth * self.mHeight;
		self.mBorder = (width + height) >> 6;
		self.mGrid = grid;
	}

	pub fn ClearPaths(&mut self) {
		let mut i: usize;

		i = 0;
		while i < MAX_RANDOM_PATHS {
			if !self.mPaths[i].is_null() {
				unsafe {
					let _ = Box::from_raw(self.mPaths[i]);
				}
				self.mPaths[i] = std::ptr::null_mut();
			}
			i += 1;
		}

		// memset(mPaths, 0, sizeof(mPaths));
		for j in 0..MAX_RANDOM_PATHS {
			self.mPaths[j] = std::ptr::null_mut();
		}
	}

	pub fn Shutdown(&mut self) {
		self.ClearPaths();
	}

	pub fn CreatePath(&mut self, PathID: c_int, ConnectedID: c_int, CreationFlags: c_uint, numPoints: c_int,
						bx: f32, by: f32, ex: f32, ey: f32,
						minWidth: f32, maxWidth: f32, depth: f32, deviation: f32, breadth: f32) -> bool {
		let connected: *mut CPathInfo = if ConnectedID >= 0 && ConnectedID < MAX_RANDOM_PATHS as c_int {
			self.mPaths[ConnectedID as usize]
		} else {
			std::ptr::null_mut()
		};

		if PathID < 0 || PathID >= MAX_RANDOM_PATHS as c_int || !self.mPaths[PathID as usize].is_null() {
			return false;
		}

		let path_box = CPathInfo::new(self.mLandScape, numPoints, bx, by, ex, ey,
			minWidth, maxWidth, depth, deviation, breadth,
			connected, CreationFlags);
		self.mPaths[PathID as usize] = Box::into_raw(path_box);

		true
	}

	pub fn GetPathInfo(&self, PathNum: c_int, PercentInto: f32, Coord: &mut vec4_t, Vector: &mut vec4_t) -> bool {
		if PathNum < 0 || PathNum >= MAX_RANDOM_PATHS as c_int || self.mPaths[PathNum as usize].is_null() {
			return false;
		}

		unsafe {
			(*self.mPaths[PathNum as usize]).GetInfo(PercentInto, Coord, Vector);
		}

		true
	}

	pub fn ParseGenerate(&mut self, GenerateFile: *const c_char) {
		// stub
	}

	pub fn Smooth(&mut self) {
		// Scale down to 1/4 size then back up to smooth out the terrain
		let temp: *mut byte;
		let x: c_int;
		let y: c_int;
		let mut o: c_int;

		unsafe {
			temp = (*self.mLandScape).GetFlattenMap();
		}

		// Copy over anything in the flatten map
		o = 0;
		while o < self.mHeight * self.mWidth {
			unsafe {
				if *temp.offset(o as isize) > 0 {
					*self.mGrid.offset(o as isize) = (*temp.offset(o as isize)) & 0x7F;
				}
			}
			o += 1;
		}

		unsafe {
			temp = Z_Malloc(self.mWidth * self.mHeight, TAG_RESAMPLE);
		}

		// #if 1
		{
			let mut total: c_uint;
			let mut count: c_uint;
			let mut x: c_int = 1;
			while x < self.mWidth - 1 {
				let mut y: c_int = 1;
				while y < self.mHeight - 1 {
					total = 0;
					count = 2;

					// Left
					unsafe {
						total += *self.mGrid.offset(((y) * self.mWidth + (x - 1)) as isize) as c_uint;
					}
					count += 1;

					// Right
					unsafe {
						total += *self.mGrid.offset(((y) * self.mWidth + (x + 1)) as isize) as c_uint;
					}
					count += 1;

					// Up
					unsafe {
						total += *self.mGrid.offset((((y - 1) * self.mWidth) + (x)) as isize) as c_uint;
					}
					count += 1;

					// Down
					unsafe {
						total += *self.mGrid.offset((((y + 1) * self.mWidth) + (x)) as isize) as c_uint;
					}
					count += 1;

					// Up-Left
					unsafe {
						total += *self.mGrid.offset((((y - 1) * self.mWidth) + (x - 1)) as isize) as c_uint;
					}
					count += 1;

					// Down-Left
					unsafe {
						total += *self.mGrid.offset((((y + 1) * self.mWidth) + (x - 1)) as isize) as c_uint;
					}
					count += 1;

					// Up-Right
					unsafe {
						total += *self.mGrid.offset((((y - 1) * self.mWidth) + (x + 1)) as isize) as c_uint;
					}
					count += 1;

					// Down-Right
					unsafe {
						total += *self.mGrid.offset((((y + 1) * self.mWidth) + (x + 1)) as isize) as c_uint;
					}
					count += 1;

					unsafe {
						total += (*self.mGrid.offset(((y) * self.mWidth + (x)) as isize) as c_uint) * 2;
					}

					unsafe {
						*temp.offset(((y) * self.mWidth + (x)) as isize) = (total / count) as byte;
					}

					y += 1;
				}
				x += 1;
			}

			unsafe {
				libc::memcpy(
					self.mGrid as *mut core::ffi::c_void,
					temp as *const core::ffi::c_void,
					(self.mWidth * self.mHeight) as usize
				);
			}
		}

		// #else
		// ... (full filtering code omitted in C code, would be ported if needed)
		// #endif

		unsafe {
			Z_Free(temp);
		}

		/* Uncomment to see the symmetry line on the map

			for ( x = 0; x < mWidth; x ++ )
			{
				mGrid[x * mWidth + x] = 255;
			}
		*/
	}

	pub fn Generate(&mut self, symmetric: c_int) {
		let mut i: c_int;
		let mut j: c_int;
		let mut x: c_int;
		let mut y: c_int;

		// Clear out all existing data
		unsafe {
			libc::memset(self.mGrid as *mut core::ffi::c_void, 255, (self.mArea as usize));
		}

		// make landscape a little bumpy
		let t1 = unsafe {
			(*self.mLandScape).flrand(0.0, 2.0)
		};
		// #if 0
		// float t2 = mLandScape->flrand(0, 2);
		// float t3 = mLandScape->flrand(0, 2);
		// #endif

		/*
			CM_NoiseInit(mLandScape);

			for (y = 0; y < mHeight; y++)
				for (x = 0; x < mWidth; x++)
				{
					i = x + y*mWidth;
					byte val = (byte)Com_Clamp(0, 255, (int)(220 + (CM_NoiseGet4f( x*0.25, y*0.25, 0, t3 ) * 20)) + (CM_NoiseGet4f( x*0.5, y*0.5, 0, t2 ) * 15));
					mGrid[i] = val;
				}
		*/

		i = 0;
		while !self.mPaths[i as usize].is_null() && i < MAX_RANDOM_PATHS as c_int {
			unsafe {
				(*self.mPaths[i as usize]).DrawPath(self.mGrid, self.mWidth, self.mHeight);
			}
			i += 1;
		}

		y = 0;
		while y < self.mHeight {
			x = 0;
			while x < self.mWidth {
				i = x + y * self.mWidth;
				let val_i = unsafe {
					*self.mGrid.offset(i as isize)
				};
				let val = Com_Clamp(0, 255, (val_i as c_int + (CM_NoiseGet4f(x as f32, y as f32, 0.0, t1) * 5.0) as c_int)) as byte;
				unsafe {
					*self.mGrid.offset(i as isize) = val;
				}
				x += 1;
			}
			y += 1;
		}

		// if symmetric, do this now
		if symmetric != 0 {
			// assert (mWidth == mHeight); // must be square

			y = 0;
			while y < self.mHeight {
				x = 0;
				while x < (self.mWidth - y) {
					i = x + y * self.mWidth;
					j = (self.mWidth - 1 - x) + (self.mHeight - 1 - y) * self.mWidth;
					let val_i = unsafe { *self.mGrid.offset(i as isize) };
					let val_j = unsafe { *self.mGrid.offset(j as isize) };
					let val = if val_i < val_j { val_i } else { val_j };
					unsafe {
						*self.mGrid.offset(i as isize) = val;
						*self.mGrid.offset(j as isize) = val;
					}
					x += 1;
				}
				y += 1;
			}
		}
	}
}

impl Drop for CRandomTerrain {
	fn drop(&mut self) {
		self.Shutdown();
	}
}

/****** Character generation tables ******/

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ECPType {
	CP_CONSONANT = 0,
	CP_COMPLEX_CONSONANT = 1,
	CP_VOWEL = 2,
	CP_COMPLEX_VOWEL = 3,
	CP_ENDING = 4,

	CP_NUM_PIECES = 5,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SCharacterPiece {
	mPiece: *const c_char,
	mCommonality: c_int,
}

static Consonants: &[SCharacterPiece] = &[
	SCharacterPiece { mPiece: b"b\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"c\0" as *const u8 as *const c_char, mCommonality: 8 },
	SCharacterPiece { mPiece: b"d\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"f\0" as *const u8 as *const c_char, mCommonality: 5 },
	SCharacterPiece { mPiece: b"g\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"h\0" as *const u8 as *const c_char, mCommonality: 5 },
	SCharacterPiece { mPiece: b"j\0" as *const u8 as *const c_char, mCommonality: 2 },
	SCharacterPiece { mPiece: b"k\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"l\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"m\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"n\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"r\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"s\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"t\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"v\0" as *const u8 as *const c_char, mCommonality: 1 },
	SCharacterPiece { mPiece: b"w\0" as *const u8 as *const c_char, mCommonality: 2 },
	SCharacterPiece { mPiece: b"x\0" as *const u8 as *const c_char, mCommonality: 1 },
	SCharacterPiece { mPiece: b"z\0" as *const u8 as *const c_char, mCommonality: 1 },

	SCharacterPiece { mPiece: std::ptr::null(), mCommonality: 0 },
];

static ComplexConsonants: &[SCharacterPiece] = &[
	SCharacterPiece { mPiece: b"st\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ck\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ss\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"tt\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"ll\0" as *const u8 as *const c_char, mCommonality: 8 },
	SCharacterPiece { mPiece: b"nd\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"rn\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"nc\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"mp\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"sc\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"sl\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"tch\0" as *const u8 as *const c_char, mCommonality: 6 },
	SCharacterPiece { mPiece: b"th\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"rn\0" as *const u8 as *const c_char, mCommonality: 5 },
	SCharacterPiece { mPiece: b"cl\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"sp\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"st\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"fl\0" as *const u8 as *const c_char, mCommonality: 4 },
	SCharacterPiece { mPiece: b"sh\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"ng\0" as *const u8 as *const c_char, mCommonality: 4 },

	SCharacterPiece { mPiece: std::ptr::null(), mCommonality: 0 },
];

static Vowels: &[SCharacterPiece] = &[
	SCharacterPiece { mPiece: b"a\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"e\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"i\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"o\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"u\0" as *const u8 as *const c_char, mCommonality: 2 },

	SCharacterPiece { mPiece: std::ptr::null(), mCommonality: 0 },
];

static ComplexVowels: &[SCharacterPiece] = &[
	SCharacterPiece { mPiece: b"ea\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ue\0" as *const u8 as *const c_char, mCommonality: 3 },
	SCharacterPiece { mPiece: b"oi\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ai\0" as *const u8 as *const c_char, mCommonality: 8 },
	SCharacterPiece { mPiece: b"oo\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"io\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"oe\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"au\0" as *const u8 as *const c_char, mCommonality: 3 },
	SCharacterPiece { mPiece: b"ee\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"ei\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"ou\0" as *const u8 as *const c_char, mCommonality: 7 },
	SCharacterPiece { mPiece: b"ia\0" as *const u8 as *const c_char, mCommonality: 4 },

	SCharacterPiece { mPiece: std::ptr::null(), mCommonality: 0 },
];

static Endings: &[SCharacterPiece] = &[
	SCharacterPiece { mPiece: b"ing\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ed\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ute\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ance\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ey\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ation\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ous\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ent\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ate\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ible\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"age\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ity\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ist\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ism\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ime\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ic\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ant\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"etry\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ious\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ative\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"er\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"ize\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"able\0" as *const u8 as *const c_char, mCommonality: 10 },
	SCharacterPiece { mPiece: b"itude\0" as *const u8 as *const c_char, mCommonality: 10 },

	SCharacterPiece { mPiece: std::ptr::null(), mCommonality: 0 },
];

fn FindPiece(type_: ECPType, pos: &mut *mut c_char) {
	let mut search: *const SCharacterPiece;
	let start: *const SCharacterPiece;
	let mut count: c_int = 0;

	start = match type_ {
		ECPType::CP_CONSONANT => Consonants.as_ptr(),
		ECPType::CP_COMPLEX_CONSONANT => ComplexConsonants.as_ptr(),
		ECPType::CP_VOWEL => Vowels.as_ptr(),
		ECPType::CP_COMPLEX_VOWEL => ComplexVowels.as_ptr(),
		ECPType::CP_ENDING => Endings.as_ptr(),
		_ => Consonants.as_ptr(),
	};

	search = start;
	unsafe {
		while !(*search).mPiece.is_null() {
			count += (*search).mCommonality;
			search = search.offset(1);
		}
	}

	count = irand(0, count - 1);
	search = start;
	unsafe {
		while count > (*search).mCommonality {
			count -= (*search).mCommonality;
			search = search.offset(1);
		}

		strcpy(*pos, (*search).mPiece);
		*pos = (*pos).offset(strlen((*search).mPiece) as isize);
	}
}

pub extern "C" fn RMG_CreateSeed(TextSeed: *mut c_char) -> c_uint {
	let mut Length: c_int;
	let mut Ending: [c_char; 256] = [0; 256];
	let mut pos: *mut c_char;
	let mut ComplexVowelChance: c_int;
	let mut ComplexConsonantChance: c_int;
	let mut LookingFor: ECPType;
	let mut SeedValue: c_uint = 0;
	let mut high: c_uint;

	Length = irand(4, 9);

	if irand(0, 100) < 20 {
		LookingFor = ECPType::CP_VOWEL;
	} else {
		LookingFor = ECPType::CP_CONSONANT;
	}

	Ending[0] = 0;

	if irand(0, 100) < 55 {
		pos = Ending.as_mut_ptr();
		FindPiece(ECPType::CP_ENDING, &mut pos);
		Length -= (unsafe { pos.offset_from(Ending.as_ptr()) }) as c_int;
	}

	pos = TextSeed;
	unsafe {
		*pos = 0;
	}

	ComplexVowelChance = -1;
	ComplexConsonantChance = -1;

	unsafe {
		while (pos.offset_from(TextSeed)) as c_int < Length || LookingFor == ECPType::CP_CONSONANT {
			if LookingFor == ECPType::CP_VOWEL {
				if irand(0, 100) < ComplexVowelChance {
					ComplexVowelChance = -1;
					LookingFor = ECPType::CP_COMPLEX_VOWEL;
				} else {
					ComplexVowelChance += 10;
				}

				FindPiece(LookingFor, &mut pos);
				LookingFor = ECPType::CP_CONSONANT;
			} else {
				if irand(0, 100) < ComplexConsonantChance {
					ComplexConsonantChance = -1;
					LookingFor = ECPType::CP_COMPLEX_CONSONANT;
				} else {
					ComplexConsonantChance += 45;
				}

				FindPiece(LookingFor, &mut pos);
				LookingFor = ECPType::CP_VOWEL;
			}
		}

		if Ending[0] != 0 {
			strcpy(pos, Ending.as_ptr());
		}

		pos = TextSeed;
		while *pos != 0 {
			high = SeedValue >> 28;
			SeedValue ^= (SeedValue << 4) + ((*pos as c_uint) - ('a' as c_uint));
			SeedValue ^= high;
			pos = pos.offset(1);
		}
	}

	SeedValue
}

// Local stubs for external functions (these should be declared in headers)
extern "C" {
	fn irand(low: c_int, high: c_int) -> c_int;
	fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	fn strlen(s: *const c_char) -> usize;
	pub fn Z_Malloc(size: c_int, tag: c_int) -> *mut byte;
	pub fn Z_Free(ptr: *mut core::ffi::c_void);
}

const TAG_RESAMPLE: c_int = 0;
