#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_char, c_void};
use std::ptr::{addr_of_mut, null_mut};
use crate::code::qcommon::cm_randomterrain_h::*;

// Extern C function declarations
extern "C" {
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn sqrt(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn fabs(x: f64) -> f64;

    // Engine stubs
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Clamp(min: c_int, max: c_int, value: c_int) -> c_int;
    fn Q_irand(min: c_int, max: c_int) -> c_int;
    fn R_Resample(
        in_data: *mut u8,
        inwidth: c_int,
        inheight: c_int,
        out_data: *mut u8,
        outwidth: c_int,
        outheight: c_int,
        byteCount: c_int,
    );
}

const NOISE_SIZE: usize = 256;
const NOISE_MASK: usize = NOISE_SIZE - 1;

static mut noiseTable: [f32; NOISE_SIZE] = [0.0; NOISE_SIZE];
static mut noisePerm: [u8; NOISE_SIZE] = [0; NOISE_SIZE];

// Stubs for external types
pub struct CCMLandScape;

unsafe impl Send for CCMLandScape {}
unsafe impl Sync for CCMLandScape {}

// Macro translations
macro_rules! VAL {
    ($a:expr) => {
        unsafe { noisePerm[($a) & (NOISE_MASK)] }
    };
}

macro_rules! INDEX {
    ($x:expr, $y:expr, $z:expr, $t:expr) => {
        VAL!($x + VAL!($y + VAL!($z + VAL!($t))))
    };
}

macro_rules! LERP {
    ($a:expr, $b:expr, $w:expr) => {
        $a * (1.0f32 - $w) + $b * $w
    };
}

fn CM_NoiseInit(landscape: *mut CCMLandScape) {
    let mut i;

    for i in 0..NOISE_SIZE {
        unsafe {
            noiseTable[i] = (*landscape).flrand(-1.0f32, 1.0f32);
            noisePerm[i] = ((*landscape).irand(0, 255) as u8);
        }
    }
}

fn GetNoiseValue(x: i32, y: i32, z: i32, t: i32) -> f32 {
    let index = INDEX!(x, y, z, t) as usize;

    unsafe { noiseTable[index] }
}

// #if 0 - disabled code
// static float GetNoiseTime( int t )
// {
//     int index = VAL( t );
//
//     return (1 + noiseTable[index]);
// }
// #endif

fn CM_NoiseGet4f(x: f32, y: f32, z: f32, t: f32) -> f32 {
    let mut ix: i32;
    let mut iy: i32;
    let mut iz: i32;
    let mut it: i32;
    let mut fx: f32;
    let mut fy: f32;
    let mut fz: f32;
    let mut ft: f32;
    let mut front: [f32; 4] = [0.0; 4];
    let mut back: [f32; 4] = [0.0; 4];
    let mut fvalue: f32;
    let mut bvalue: f32;
    let mut value: [f32; 2] = [0.0; 2];
    let mut finalvalue: f32;
    let mut i;

    ix = unsafe { floor(x as f64) as i32 };
    fx = x - ix as f32;
    iy = unsafe { floor(y as f64) as i32 };
    fy = y - iy as f32;
    iz = unsafe { floor(z as f64) as i32 };
    fz = z - iz as f32;
    it = unsafe { floor(t as f64) as i32 };
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

        fvalue = LERP!(LERP!(front[0], front[1], fx), LERP!(front[2], front[3], fx), fy);
        bvalue = LERP!(LERP!(back[0], back[1], fx), LERP!(back[2], back[3], fx), fy);

        value[i] = LERP!(fvalue, bvalue, fz);

        i += 1;
    }

    finalvalue = LERP!(value[0], value[1], ft);
    finalvalue
}

/****** lincrv.c ******/
/* Ken Shoemake, 1994 */

/* Perform a generic vector unary operation. */
#[inline]
fn V_Op(vdst: &mut [f32], vsrc: &[f32]) {
    let n = vdst.len().min(vsrc.len());
    for i in (0..n).rev() {
        vdst[i] = vsrc[i];
    }
}

fn lerp(t: f32, a0: f32, a1: f32, p0: &[f32], p1: &[f32], m: usize, p: &mut [f32]) {
    let t0 = (a1 - t) / (a1 - a0);
    let t1 = 1.0f32 - t0;
    for i in (0..m).rev() {
        p[i] = t0 * p0[i] + t1 * p1[i];
    }
}

/* DialASpline(t,a,p,m,n,work,Cn,interp,val) computes a point val at parameter
    t on a spline with knot values a and control points p. The curve will have
    Cn continuity, and if interp is TRUE it will interpolate the control points.
    Possibilities include Langrange interpolants, Bezier curves, Catmull-Rom
    interpolating splines, and B-spline curves. Points have m coordinates, and
    n+1 of them are provided. The work array must have room for n+1 points.
 */
fn DialASpline(
    t: f32,
    a: &[f32],
    p: &[&[f32]],
    m: usize,
    n: usize,
    work: &mut [Vec<f32>],
    Cn: &mut usize,
    interp: bool,
    val: &mut [f32],
) -> usize {
    let mut i;
    let mut j;
    let mut k = 0;
    let mut h;
    let mut lo;
    let mut hi;

    if *Cn > n - 1 {
        *Cn = n - 1;
    } /* Anything greater gives one polynomial */
    for k in 0..=n {
        if t <= a[k] {
            break;
        }
    } /* Find enclosing knot interval */

    let h_start = k;
    for k in h_start..=n {
        if t != a[k] {
            break;
        }
    } /* May want to use fewer legs */

    if k > n {
        k = n;
        if h_start > k {
            h = k;
        } else {
            h = h_start;
        }
    } else {
        h = h_start;
    }

    h = 1 + *Cn - (k - h);
    k -= 1;
    lo = (k as i32 - *Cn as i32).max(0) as usize;
    hi = (k + 1 + *Cn).min(n);

    if interp {
        /* Lagrange interpolation steps */
        let mut drop = 0;
        if lo < 0 || lo == 0 {
            if lo == 0 {
                drop += *Cn - k;
                if hi - lo < *Cn {
                    drop += *Cn - hi;
                    hi = *Cn;
                }
            }
        }
        if hi > n {
            hi = n;
            drop += k + 1 + *Cn - n;
            if hi - lo < *Cn {
                drop += lo - (n - *Cn);
                lo = n - *Cn;
            }
        }
        for i in lo..=hi {
            V_Op(&mut work[i], &p[i]);
        }
        for j in 1..=*Cn {
            for i in lo..=(hi - j) {
                lerp(t, a[i], a[i + j], &work[i], &work[i + 1], m, &mut work[i]);
            }
        }
        h = 1 + *Cn - drop;
    } else {
        /* Prepare for B-spline steps */
        if lo < 0 {
            h = (h as i32 + lo as i32) as usize;
            lo = 0;
        }
        for i in lo..=(lo + h) {
            if i <= n {
                V_Op(&mut work[i], &p[i]);
            }
        }
        if (h as i32) < 0 {
            h = 0;
        }
    }
    for j in 0..h {
        let tmp = 1 + *Cn - j;
        for i in (j..h).rev() {
            lerp(t, a[lo + i], a[lo + i + tmp], &work[lo + i], &work[lo + i + 1], m, &mut work[lo + i + 1]);
        }
    }
    V_Op(val, &work[lo + h]);
    k
}

const BIG: f32 = 1.0e12;

fn Vector2Normalize(v: &mut [f32; 2]) -> f32 {
    let mut length: f32;
    let mut ilength: f32;

    length = v[0] * v[0] + v[1] * v[1];
    length = unsafe { sqrt(length as f64) as f32 };

    if length != 0.0 {
        ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
    }

    length
}

impl CPathInfo {
    fn new(
        landscape: *mut CCMLandScape,
        numPoints: c_int,
        bx: f32,
        by: f32,
        ex: f32,
        ey: f32,
        minWidth: f32,
        maxWidth: f32,
        depth: f32,
        deviation: f32,
        breadth: f32,
        Connected: *mut CPathInfo,
        CreationFlags: c_uint,
    ) -> Self {
        let mut path = CPathInfo {
            mPoints: null_mut(),
            mWork: null_mut(),
            mWeights: null_mut(),
            mNumPoints: numPoints,
            mMinWidth: minWidth,
            mMaxWidth: maxWidth,
            mInc: 0.0,
            mDepth: depth,
            mBreadth: breadth,
            mDeviation: deviation,
            mCircleStamp: [[0; CIRCLE_STAMP_SIZE]; CIRCLE_STAMP_SIZE],
        };

        path.CreateCircle();

        let mut numConnected = -1;
        if !Connected.is_null() {
            // we are connecting to an existing spline
            numConnected = unsafe { (*Connected).GetNumPoints() };
            if numConnected >= SPLINE_MERGE_SIZE {
                // plenty of points to choose from
                path.mNumPoints += SPLINE_MERGE_SIZE;
            } else {
                // the existing spline doesn't have enough points
                path.mNumPoints += numConnected;
            }
        }

        path.mPoints = unsafe {
            malloc((std::mem::size_of::<[f32; 4]>() * path.mNumPoints as usize)) as *mut [f32; 4]
                as *mut _
        };
        path.mWork = unsafe {
            malloc((std::mem::size_of::<[f32; 4]>() * (path.mNumPoints + 1) as usize))
                as *mut [f32; 4] as *mut _
        };
        path.mWeights =
            unsafe { malloc((std::mem::size_of::<f32>() * (path.mNumPoints + 1) as usize)) as *mut _ };

        let length = unsafe { sqrt(((ex - bx) * (ex - bx) + (ey - by) * (ey - by)) as f64) as f32 };
        let horizontal: bool;
        let mut position: f32;
        let goal: f32;
        let deltaGoal: f32;

        if unsafe { fabs((ex - bx) as f64) as f32 >= fabs((ey - by) as f64) as f32 } {
            // this appears to be a horizontal path
            path.mInc = 1.0 / unsafe { fabs((ex - bx) as f64) as f32 };
            horizontal = true;
            position = by;
            goal = ey;
            deltaGoal = (ey - by) / (numPoints - 1) as f32;
        } else {
            // this appears to be a vertical path
            path.mInc = 1.0 / unsafe { fabs((ey - by) as f64) as f32 };
            horizontal = false;
            position = bx;
            goal = ex;
            deltaGoal = (ex - bx) / (numPoints - 1) as f32;
        }
        let mut normalizedPath: [f32; 2] = [ex - bx, ey - by];
        Vector2Normalize(&mut normalizedPath);
        // approx calculate how much we need to iterate through the spline to hit every point
        path.mInc /= 16.0;

        let mut currentWidth = unsafe { (*landscape).flrand(minWidth, maxWidth) };
        let mut currentPosition = 0.0f32;

        for i in 0..path.mNumPoints as usize {
            // weights are evenly distributed
            unsafe {
                *path.mWeights.add(i) = i as f32 / (path.mNumPoints - 1) as f32;
            }

            if i < numConnected as usize && i < SPLINE_MERGE_SIZE as usize {
                // we are connecting to an existing spline, so copy over the first few points
                let index: usize;
                if CreationFlags & PATH_CREATION_CONNECT_FRONT != 0 {
                    // copy from the front
                    index = i;
                } else {
                    // copy from the end
                    index = (numConnected as usize - SPLINE_MERGE_SIZE as usize + i);
                }
                let point = unsafe { (*Connected).GetPoint(index as c_int) };
                unsafe {
                    let dst = path.mPoints.add(i);
                    (*dst)[0] = (*point.add(0));
                    (*dst)[1] = (*point.add(1));
                    (*dst)[3] = (*point.add(3));
                }
            } else {
                if horizontal {
                    // we appear to be going horizontal, so spread the randomness across the vertical
                    unsafe {
                        let dst = path.mPoints.add(i);
                        (*dst)[0] = ((ex - bx) * currentPosition) + bx;
                        (*dst)[1] = position;
                    }
                } else {
                    // we appear to be going vertical, so spread the randomness across the horizontal
                    unsafe {
                        let dst = path.mPoints.add(i);
                        (*dst)[0] = position;
                        (*dst)[1] = ((ey - by) * currentPosition) + by;
                    }
                }
                currentPosition += 1.0 / (path.mNumPoints - 1) as f32;

                // set the width of the spline
                unsafe {
                    let dst = path.mPoints.add(i);
                    (*dst)[3] = currentWidth;
                }
                currentWidth += unsafe { (*landscape).flrand(-0.1f32, 0.1f32) };
                if currentWidth < minWidth {
                    currentWidth = minWidth;
                } else if currentWidth > maxWidth {
                    currentWidth = maxWidth;
                }

                // see how far we are from the goal
                /*			delta = (goal - position) * currentPosition;
                // calculate the randomness we are allowed at this place
                random = landscape->flrand(-mDeviation/1.0, mDeviation/1.0) * (1.0 - currentPosition);
                position += delta + random;*/

                if i == (path.mNumPoints - 2) as usize {
                    // -2 because we are calculating for the next point
                    position = goal;
                } else {
                    if i == 0 {
                        position += deltaGoal + unsafe { (*landscape).flrand(-deviation / 10.0, deviation / 10.0) };
                    } else {
                        position += deltaGoal + unsafe { (*landscape).flrand(-deviation * 1.5, deviation * 1.5) };
                    }
                }

                if position > 0.9 {
                    // too far over, so move back a bit
                    position = 0.9 - unsafe { (*landscape).flrand(0.02f32, 0.1f32) };
                }
                if position < 0.1 {
                    // too near, so move bakc a bit
                    position = 0.1 + unsafe { (*landscape).flrand(0.02f32, 0.1f32) };
                }

                // check our deviation from the straight line to the end
                let mut testPoint: [f32; 2];
                if horizontal {
                    testPoint = [((ex - bx) * currentPosition) + bx, position];
                } else {
                    testPoint = [position, ((ey - by) * currentPosition) + by];
                }
                // dot product of the normal of the path to the point we are at
                let mut distance =
                    ((testPoint[0] - bx) * normalizedPath[0]) + ((testPoint[1] - by) * normalizedPath[1]);
                // find the perpendicular place that is intersected by the point and the path
                let mut percPoint: [f32; 2] = [
                    (distance * normalizedPath[0]) + bx,
                    (distance * normalizedPath[1]) + by,
                ];
                // calculate the difference between the perpendicular point and the test point
                let mut diffPoint: [f32; 2] =
                    [testPoint[0] - percPoint[0], testPoint[1] - percPoint[1]];
                // calculate the distance
                distance = unsafe {
                    sqrt(((diffPoint[0] * diffPoint[0]) + (diffPoint[1] * diffPoint[1])) as f64)
                        as f32
                };
                if distance > deviation {
                    // we are beyond our allowed deviation, so head back
                    if horizontal {
                        position = (ey - by) * currentPosition + by;
                    } else {
                        position = (ex - bx) * currentPosition + bx;
                    }
                    position += unsafe { (*landscape).flrand(-deviation / 2.0, deviation / 2.0) };
                }
            }
        }
        unsafe {
            *path.mWeights.add(path.mNumPoints as usize) = BIG;
        }

        path
    }

    fn CreateCircle(&mut self) {
        let mut x;
        let mut y;
        let r: f32;
        let mut d: f32;

        unsafe {
            memset(
                addr_of_mut!(self.mCircleStamp) as *mut c_void,
                0,
                std::mem::size_of_val(&self.mCircleStamp),
            );
        }
        r = CIRCLE_STAMP_SIZE as f32;
        for x in 0..CIRCLE_STAMP_SIZE {
            for y in 0..CIRCLE_STAMP_SIZE {
                d = unsafe { sqrt(((x * x + y * y) as f32) as f64) as f32 };
                if d > r {
                    self.mCircleStamp[y][x] = 255;
                } else {
                    self.mCircleStamp[y][x] =
                        unsafe { (pow(sin((d / r * std::f32::consts::PI / 2.0) as f64), self.mBreadth as f64) as f32 * 255.0) as u8 };
                }
            }
        }
    }

    fn Stamp(
        &mut self,
        x: c_int,
        y: c_int,
        size: c_int,
        depth: c_int,
        Data: *mut u8,
        DataWidth: c_int,
        DataHeight: c_int,
    ) {
        let offset: f32 = ((CIRCLE_STAMP_SIZE - 1) as f32) / size as f32;
        let invDepth: u8 = (255 - depth) as u8;

        let mut dx = -size;
        while dx <= size {
            let mut dy = -size;
            while dy <= size {
                let d: f32 = (dx * dx + dy * dy) as f32;

                if d > (size * size) as f32 {
                    dy += 1;
                    continue;
                }

                let fx = x + dx;
                if fx < 2 || fx > DataWidth - 2 {
                    dy += 1;
                    continue;
                }

                let fy = y + dy;
                if fy < 2 || fy > DataHeight - 2 {
                    dy += 1;
                    continue;
                }

                let value: u8 = unsafe {
                    (pow(
                        sin((d / ((size * size) as f32) * std::f32::consts::PI / 2.0) as f64),
                        self.mBreadth as f64,
                    ) as f32 * invDepth as f32 + depth as f32) as u8
                };
                unsafe {
                    let idx = ((fy * DataWidth) + fx) as usize;
                    if value < *Data.add(idx) {
                        *Data.add(idx) = value;
                    }
                }

                dy += 1;
            }
            dx += 1;
        }
        /*

            fx = x + dx;
            if (fx < 2 || fx > DataWidth-2)
            {
                continue;
            }
            xPos = abs((int)(dx*offset));
            yPos = offset*size + offset;
            for(dy = -size; dy < 0; dy++)
            {
                yPos -= offset;
                fy = y + dy;
                if (fy < 2 || fy > DataHeight-2)
                {
                    continue;
                }

                value = (invDepth * mCircleStamp[(int)yPos][xPos] / 256) + depth;
                if (value < Data[(fy * DataWidth) + fx])
                {
                    Data[(fy * DataWidth) + fx] = value;
                }
            }

            yPos = -offset;
            for(; dy <= size; dy++)
            {
                yPos += offset;

                fy = y + dy;
                if (fy < 2 || fy > DataHeight-2)
                {
                    continue;
                }

                value = (invDepth * mCircleStamp[(int)yPos][xPos] / 256) + depth;
                if (value < Data[(fy * DataWidth) + fx])
                {
                    Data[(fy * DataWidth) + fx] = value;
                }
            }
        }
        */
    }

    pub fn GetInfo(&mut self, PercentInto: f32, Coord: &mut [f32; 4], Vector: &mut [f32; 4]) {
        let mut before: [f32; 4] = [0.0; 4];
        let mut after: [f32; 4] = [0.0; 4];
        let mut testPercent: f32;

        let mut weights_vec = vec![0.0f32; (self.mNumPoints + 1) as usize];
        let mut points_vec = vec![vec![0.0f32; 4]; self.mNumPoints as usize];
        let mut work_vec = vec![vec![0.0f32; 4]; (self.mNumPoints + 1) as usize];

        // Copy data from raw pointers to vectors for safe access
        unsafe {
            for i in 0..=(self.mNumPoints as usize) {
                weights_vec[i] = *self.mWeights.add(i);
            }
            for i in 0..(self.mNumPoints as usize) {
                points_vec[i].copy_from_slice(std::slice::from_raw_parts(
                    self.mPoints.add(i) as *const f32,
                    4,
                ));
            }
        }

        let mut Cn = 2usize;
        let points_refs: Vec<&[f32]> = points_vec.iter().map(|v| v.as_slice()).collect();

        DialASpline(
            PercentInto,
            &weights_vec,
            &points_refs,
            4,
            (self.mNumPoints - 1) as usize,
            &mut work_vec,
            &mut Cn,
            true,
            Coord,
        );

        testPercent = PercentInto - 0.01;
        if testPercent < 0.0 {
            testPercent = 0.0;
        }
        let mut Cn = 2usize;
        DialASpline(
            testPercent,
            &weights_vec,
            &points_refs,
            4,
            (self.mNumPoints - 1) as usize,
            &mut work_vec,
            &mut Cn,
            true,
            &mut before,
        );

        testPercent = PercentInto + 0.01;
        if testPercent > 1.0 {
            testPercent = 1.0;
        }
        let mut Cn = 2usize;
        DialASpline(
            testPercent,
            &weights_vec,
            &points_refs,
            4,
            (self.mNumPoints - 1) as usize,
            &mut work_vec,
            &mut Cn,
            true,
            &mut after,
        );

        Coord[2] = self.mDepth;

        Vector[0] = after[0] - before[0];
        Vector[1] = after[1] - before[1];
    }

    pub fn DrawPath(&self, Data: *mut u8, DataWidth: c_int, DataHeight: c_int) {
        let mut t: f32;
        let mut val: [f32; 4] = [0.0; 4];
        let mut vector: [f32; 4] = [0.0; 4];
        let inc: f32 = self.mInc / DataWidth as f32;

        let mut lastX = -999;
        let mut lastY = -999;

        t = 0.0;
        while t <= 1.0 {
            let mut path_mut = self as *const _ as *mut CPathInfo;
            unsafe {
                (*path_mut).GetInfo(t, &mut val, &mut vector);
            }

            /*		perp[0] = -vector[1];
            perp[1] = vector[0];

            if (fabs(perp[0]) > fabs(perp[1]))
            {
                perp[1] /= fabs(perp[0]);
                perp[0] /= fabs(perp[0]);
            }
            else
            {
                perp[0] /= fabs(perp[1]);
                perp[1] /= fabs(perp[1]);
            }
            */
            let x = (val[0] * DataWidth as f32) as c_int;
            let y = (val[1] * DataHeight as f32) as c_int;

            if x == lastX && y == lastY {
                t += inc;
                continue;
            }

            lastX = x;
            lastY = y;

            let size = (val[3] * DataWidth as f32) as c_int;

            let depth = self.mDepth * 255.0f32;

            let mut path_mut = self as *const _ as *mut CPathInfo;
            unsafe {
                (*path_mut).Stamp(x, y, size, depth as c_int, Data, DataWidth, DataHeight);
            }

            t += inc;
        }
    }
}

impl Drop for CPathInfo {
    fn drop(&mut self) {
        unsafe {
            free(self.mWeights as *mut c_void);
            free(self.mWork as *mut c_void);
            free(self.mPoints as *mut c_void);
        }
    }
}

impl CRandomTerrain {
    pub fn new() -> Self {
        CRandomTerrain {
            mLandScape: null_mut(),
            mWidth: 0,
            mHeight: 0,
            mArea: 0,
            mBorder: 0,
            mGrid: null_mut(),
            mPaths: [null_mut(); 30],
        }
    }

    pub fn Init(&mut self, landscape: *mut CCMLandScape, grid: *mut u8, width: c_int, height: c_int) {
        self.Shutdown();
        self.mLandScape = landscape;
        self.mWidth = width;
        self.mHeight = height;
        self.mArea = self.mWidth * self.mHeight;
        self.mBorder = (width + height) >> 6;
        self.mGrid = grid;
    }

    pub fn ClearPaths(&mut self) {
        for i in 0..30 {
            if !self.mPaths[i].is_null() {
                unsafe {
                    let _ = Box::from_raw(self.mPaths[i]);
                }
                self.mPaths[i] = null_mut();
            }
        }

        unsafe {
            memset(addr_of_mut!(self.mPaths) as *mut c_void, 0, std::mem::size_of_val(&self.mPaths));
        }
    }

    pub fn Shutdown(&mut self) {
        self.ClearPaths();
    }

    pub fn CreatePath(
        &mut self,
        PathID: c_int,
        ConnectedID: c_int,
        CreationFlags: c_uint,
        numPoints: c_int,
        bx: f32,
        by: f32,
        ex: f32,
        ey: f32,
        minWidth: f32,
        maxWidth: f32,
        depth: f32,
        deviation: f32,
        breadth: f32,
    ) -> bool {
        if PathID < 0 || PathID >= 30 as c_int || !self.mPaths[PathID as usize].is_null() {
            return false;
        }

        let mut connected: *mut CPathInfo = null_mut();

        if ConnectedID >= 0 && ConnectedID < 30 as c_int {
            connected = self.mPaths[ConnectedID as usize];
        }

        let path = Box::new(CPathInfo::new(
            self.mLandScape,
            numPoints,
            bx,
            by,
            ex,
            ey,
            minWidth,
            maxWidth,
            depth,
            deviation,
            breadth,
            connected,
            CreationFlags,
        ));

        self.mPaths[PathID as usize] = Box::into_raw(path);

        true
    }

    pub fn GetPathInfo(&self, PathNum: c_int, PercentInto: f32, Coord: &mut [f32; 4], Vector: &mut [f32; 4]) -> bool {
        if PathNum < 0 || PathNum >= 30 as c_int || self.mPaths[PathNum as usize].is_null() {
            return false;
        }

        unsafe {
            (*self.mPaths[PathNum as usize]).GetInfo(PercentInto, Coord, Vector);
        }

        true
    }

    pub fn ParseGenerate(&mut self, _GenerateFile: *const c_char) {
        // Empty implementation
    }

    pub fn Smooth(&mut self) {
        // Scale down to 1/4 size then back up to smooth out the terrain
        let temp: *mut u8;

        temp = unsafe { (*self.mLandScape).GetFlattenMap() };

        // Copy over anything in the flatten map
        if !temp.is_null() {
            let mut o = 0;
            while o < self.mHeight * self.mWidth {
                unsafe {
                    if *temp.add(o as usize) > 0 {
                        *self.mGrid.add(o as usize) = (*temp.add(o as usize)) & 0x7F;
                    }
                }
                o += 1;
            }
        }
        let temp = unsafe { Z_Malloc((self.mWidth * self.mHeight) as usize, 0, 0) as *mut u8 };
        {
            #[cfg(feature = "use_smooth_1")]
            {
                let mut total: c_uint;
                let mut count: c_uint;
                for x in 1..(self.mWidth - 1) {
                    for y in 1..(self.mHeight - 1) {
                        total = 0;
                        count = 2;

                        // Left
                        unsafe {
                            total += *self.mGrid.add(((y) * self.mWidth + (x - 1)) as usize) as c_uint;
                        }
                        count += 1;

                        // Right
                        unsafe {
                            total += *self.mGrid.add(((y) * self.mWidth + (x + 1)) as usize) as c_uint;
                        }
                        count += 1;

                        // Up
                        unsafe {
                            total += *self.mGrid.add((((y - 1) * self.mWidth) + (x)) as usize) as c_uint;
                        }
                        count += 1;

                        // Down
                        unsafe {
                            total += *self.mGrid.add((((y + 1) * self.mWidth) + (x)) as usize) as c_uint;
                        }
                        count += 1;

                        // Up-Left
                        unsafe {
                            total += *self.mGrid.add((((y - 1) * self.mWidth) + (x - 1)) as usize) as c_uint;
                        }
                        count += 1;

                        // Down-Left
                        unsafe {
                            total += *self.mGrid.add((((y + 1) * self.mWidth) + (x - 1)) as usize) as c_uint;
                        }
                        count += 1;

                        // Up-Right
                        unsafe {
                            total += *self.mGrid.add((((y - 1) * self.mWidth) + (x + 1)) as usize) as c_uint;
                        }
                        count += 1;

                        // Down-Right
                        unsafe {
                            total += *self.mGrid.add((((y + 1) * self.mWidth) + (x + 1)) as usize) as c_uint;
                        }
                        count += 1;

                        unsafe {
                            total += (*self.mGrid.add(((y) * self.mWidth + (x)) as usize)) as c_uint * 2;
                        }

                        unsafe {
                            *temp.add(((y) * self.mWidth + (x)) as usize) = (total / count) as u8;
                        }
                    }
                }

                unsafe {
                    memcpy(
                        self.mGrid as *mut c_void,
                        temp as *const c_void,
                        (self.mWidth * self.mHeight) as usize,
                    );
                }
            }
            #[cfg(not(feature = "use_smooth_1"))]
            {
                let FILTER_SIZE: usize = 2 * 2 + 1;
                let KERNEL_SIZE: i32 = 2;
                let mut smoothKernel: [[f32; FILTER_SIZE]; FILTER_SIZE] = [[0.0; FILTER_SIZE]; FILTER_SIZE];
                let mut xx;
                let mut yy;
                let mut dx;
                let mut dy;
                let mut total: f32;
                let mut num: f32;

                unsafe {
                    R_Resample(
                        self.mGrid,
                        self.mWidth,
                        self.mHeight,
                        temp,
                        self.mWidth >> 1,
                        self.mHeight >> 1,
                        1,
                    );
                    R_Resample(temp, self.mWidth >> 1, self.mHeight >> 1, self.mGrid, self.mWidth, self.mHeight, 1);

                    // now lets filter it.
                    memcpy(
                        temp as *mut c_void,
                        self.mGrid as *const c_void,
                        (self.mWidth * self.mHeight) as usize,
                    );
                }

                dy = -KERNEL_SIZE;
                while dy <= KERNEL_SIZE {
                    dx = -KERNEL_SIZE;
                    while dx <= KERNEL_SIZE {
                        smoothKernel[(dy + KERNEL_SIZE) as usize][(dx + KERNEL_SIZE) as usize] = 1.0f32
                            / (1.0f32
                                + unsafe { fabs((dx as f32 * dx as f32 * dx as f32) as f64) }
                                    as f32
                                + unsafe { fabs((dy as f32 * dy as f32 * dy as f32) as f64) }
                                    as f32);
                        dx += 1;
                    }
                    dy += 1;
                }

                for y in 0..self.mHeight {
                    for x in 0..self.mWidth {
                        total = 0.0f32;
                        num = 0.0f32;
                        dy = -KERNEL_SIZE;
                        while dy <= KERNEL_SIZE {
                            dx = -KERNEL_SIZE;
                            while dx <= KERNEL_SIZE {
                                xx = x + dx;
                                if xx >= 0 && xx < self.mWidth {
                                    yy = y + dy;
                                    if yy >= 0 && yy < self.mHeight {
                                        unsafe {
                                            total += smoothKernel[(dy + KERNEL_SIZE) as usize]
                                                [(dx + KERNEL_SIZE) as usize]
                                                * (*temp.add((yy * self.mWidth + xx) as usize)) as f32;
                                            num += smoothKernel[(dy + KERNEL_SIZE) as usize]
                                                [(dx + KERNEL_SIZE) as usize];
                                        }
                                    }
                                }
                                dx += 1;
                            }
                            dy += 1;
                        }
                        total /= num;
                        unsafe {
                            *self.mGrid.add((y * self.mWidth + x) as usize) =
                                unsafe { Com_Clamp(0, 255, (total.round() as c_int)) as u8 };
                        }
                    }
                }
            }
        }

        unsafe {
            Z_Free(temp as *mut c_void);
        }

        /* Uncomment to see the symmetry line on the map

        for ( x = 0; x < mWidth; x ++ )
        {
            mGrid[x * mWidth + x] = 255;
        }
        */
    }

    pub fn Generate(&mut self, symmetric: c_int) {
        // Clear out all existing data
        unsafe {
            memset(self.mGrid as *mut c_void, 255, self.mArea as usize);
        }

        // make landscape a little bumpy
        let t1 = unsafe { (*self.mLandScape).flrand(0.0, 2.0) };
        let t2 = unsafe { (*self.mLandScape).flrand(0.0, 2.0) };
        let t3 = unsafe { (*self.mLandScape).flrand(0.0, 2.0) };

        CM_NoiseInit(self.mLandScape);

        for y in 0..self.mHeight {
            for x in 0..self.mWidth {
                let i = (x + y * self.mWidth) as usize;
                let val = unsafe {
                    Com_Clamp(
                        0,
                        255,
                        220 + ((CM_NoiseGet4f(x as f32 * 0.25, y as f32 * 0.25, 0.0, t3) * 20.0) as c_int)
                            + ((CM_NoiseGet4f(x as f32 * 0.5, y as f32 * 0.5, 0.0, t2) * 15.0) as c_int),
                    ) as u8
                };
                unsafe {
                    *self.mGrid.add(i) = val;
                }
            }
        }

        let mut i = 0;
        while i < 30 {
            if self.mPaths[i].is_null() {
                break;
            }
            unsafe {
                (*self.mPaths[i]).DrawPath(self.mGrid, self.mWidth, self.mHeight);
            }
            i += 1;
        }

        for y in 0..self.mHeight {
            for x in 0..self.mWidth {
                let i = (x + y * self.mWidth) as usize;
                let val = unsafe {
                    Com_Clamp(
                        0,
                        255,
                        (*self.mGrid.add(i)) as c_int + ((CM_NoiseGet4f(x as f32, y as f32, 0.0, t1) * 5.0) as c_int),
                    ) as u8
                };
                unsafe {
                    *self.mGrid.add(i) = val;
                }
            }
        }

        // if symmetric, do this now
        if symmetric != 0 {
            // must be square
            assert_eq!(self.mWidth, self.mHeight);

            for y in 0..self.mHeight {
                for x in 0..(self.mWidth - y) {
                    let i = (x + y * self.mWidth) as usize;
                    let j = ((self.mWidth - 1 - x) + (self.mHeight - 1 - y) * self.mWidth) as usize;
                    unsafe {
                        let val = if *self.mGrid.add(i) < *self.mGrid.add(j) {
                            *self.mGrid.add(i)
                        } else {
                            *self.mGrid.add(j)
                        };
                        *self.mGrid.add(i) = val;
                        *self.mGrid.add(j) = val;
                    }
                }
            }
        }
    }
}

impl Drop for CRandomTerrain {
    fn drop(&mut self) {
        self.Shutdown();
    }
}

#[repr(u32)]
enum ECPType {
    RMG_CP_NONE = -1i32 as u32,
    RMG_CP_CONSONANT = 0,
    RMG_CP_COMPLEX_CONSONANT = 1,
    RMG_CP_VOWEL = 2,
    RMG_CP_COMPLEX_VOWEL = 3,
    RMG_CP_ENDING = 4,
    RMG_CP_NUM_PIECES = 5,
}

#[repr(C)]
struct TCharacterPiece {
    mPiece: *const c_char,
    mCommonality: c_int,
}

static Consonants: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: b"b\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"c\0" as *const u8 as *const c_char, mCommonality: 8 },
    TCharacterPiece { mPiece: b"d\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"f\0" as *const u8 as *const c_char, mCommonality: 5 },
    TCharacterPiece { mPiece: b"g\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"h\0" as *const u8 as *const c_char, mCommonality: 5 },
    TCharacterPiece { mPiece: b"j\0" as *const u8 as *const c_char, mCommonality: 2 },
    TCharacterPiece { mPiece: b"k\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"l\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"m\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"n\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"r\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"s\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"t\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"v\0" as *const u8 as *const c_char, mCommonality: 1 },
    TCharacterPiece { mPiece: b"w\0" as *const u8 as *const c_char, mCommonality: 2 },
    TCharacterPiece { mPiece: b"x\0" as *const u8 as *const c_char, mCommonality: 1 },
    TCharacterPiece { mPiece: b"z\0" as *const u8 as *const c_char, mCommonality: 1 },
    TCharacterPiece { mPiece: null_mut() as *const c_char, mCommonality: 0 },
];

static ComplexConsonants: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: b"st\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ck\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ss\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"tt\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"ll\0" as *const u8 as *const c_char, mCommonality: 8 },
    TCharacterPiece { mPiece: b"nd\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"rn\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"nc\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"mp\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"sc\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"sl\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"tch\0" as *const u8 as *const c_char, mCommonality: 6 },
    TCharacterPiece { mPiece: b"th\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"rn\0" as *const u8 as *const c_char, mCommonality: 5 },
    TCharacterPiece { mPiece: b"cl\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"sp\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"st\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"fl\0" as *const u8 as *const c_char, mCommonality: 4 },
    TCharacterPiece { mPiece: b"sh\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"ng\0" as *const u8 as *const c_char, mCommonality: 4 },
    // {	"" },
    TCharacterPiece { mPiece: null_mut() as *const c_char, mCommonality: 0 },
];

static Vowels: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: b"a\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"e\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"i\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"o\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"u\0" as *const u8 as *const c_char, mCommonality: 2 },
    // {	"" },
    TCharacterPiece { mPiece: null_mut() as *const c_char, mCommonality: 0 },
];

static ComplexVowels: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: b"ea\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ue\0" as *const u8 as *const c_char, mCommonality: 3 },
    TCharacterPiece { mPiece: b"oi\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ai\0" as *const u8 as *const c_char, mCommonality: 8 },
    TCharacterPiece { mPiece: b"oo\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"io\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"oe\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"au\0" as *const u8 as *const c_char, mCommonality: 3 },
    TCharacterPiece { mPiece: b"ee\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"ei\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"ou\0" as *const u8 as *const c_char, mCommonality: 7 },
    TCharacterPiece { mPiece: b"ia\0" as *const u8 as *const c_char, mCommonality: 4 },
    // {	"" },
    TCharacterPiece { mPiece: null_mut() as *const c_char, mCommonality: 0 },
];

static Endings: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: b"ing\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ed\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ute\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ance\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ey\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ation\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ous\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ent\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ate\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ible\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"age\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ity\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ist\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ism\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ime\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ic\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ant\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"etry\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ious\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ative\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"er\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"ize\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"able\0" as *const u8 as *const c_char, mCommonality: 10 },
    TCharacterPiece { mPiece: b"itude\0" as *const u8 as *const c_char, mCommonality: 10 },
    // {	"" },
    TCharacterPiece { mPiece: null_mut() as *const c_char, mCommonality: 0 },
];

fn FindPiece(type_: ECPType, pos: &mut Vec<u8>) {
    let start: &'static [TCharacterPiece];

    match type_ {
        ECPType::RMG_CP_CONSONANT => {
            start = Consonants;
        }
        ECPType::RMG_CP_COMPLEX_CONSONANT => {
            start = ComplexConsonants;
        }
        ECPType::RMG_CP_VOWEL => {
            start = Vowels;
        }
        ECPType::RMG_CP_COMPLEX_VOWEL => {
            start = ComplexVowels;
        }
        ECPType::RMG_CP_ENDING => {
            start = Endings;
        }
        _ => {
            start = Consonants;
        }
    }

    let mut search = 0;
    let mut count = 0;
    while search < start.len() && !start[search].mPiece.is_null() {
        count += start[search].mCommonality;
        search += 1;
    }

    let mut count = unsafe { Q_irand(0, count - 1) };
    search = 0;
    while search < start.len() && count > start[search].mCommonality {
        count -= start[search].mCommonality;
        search += 1;
    }

    let piece = start[search].mPiece;
    unsafe {
        let piece_len = strlen(piece);
        let piece_slice = std::slice::from_raw_parts(piece as *const u8, piece_len);
        pos.extend_from_slice(piece_slice);
    }
}

pub extern "C" fn RMG_CreateSeed(TextSeed: *mut c_char) -> c_uint {
    let Length: c_int;
    let mut Ending: [u8; 256] = [0; 256];
    let mut pos_idx: usize = 0;
    let mut ComplexVowelChance: c_int;
    let mut ComplexConsonantChance: c_int;
    let mut LookingFor: ECPType;
    let mut SeedValue: c_uint = 0;
    let mut high: c_uint;

    let Length = unsafe { Q_irand(4, 9) };

    if unsafe { Q_irand(0, 100) } < 20 {
        LookingFor = ECPType::RMG_CP_VOWEL;
    } else {
        LookingFor = ECPType::RMG_CP_CONSONANT;
    }

    Ending[0] = 0;

    if unsafe { Q_irand(0, 100) } < 55 {
        let mut ending_vec = Vec::new();
        FindPiece(ECPType::RMG_CP_ENDING, &mut ending_vec);
        for b in &ending_vec {
            Ending[pos_idx] = *b;
            pos_idx += 1;
        }
        Length -= pos_idx as c_int;
    }

    let mut text_seed_vec = Vec::new();

    ComplexVowelChance = -1;
    ComplexConsonantChance = -1;

    while (text_seed_vec.len() as c_int) < Length || LookingFor == ECPType::RMG_CP_CONSONANT {
        if LookingFor == ECPType::RMG_CP_VOWEL {
            if unsafe { Q_irand(0, 100) } < ComplexVowelChance {
                ComplexVowelChance = -1;
                LookingFor = ECPType::RMG_CP_COMPLEX_VOWEL;
            } else {
                ComplexVowelChance += 10;
            }

            FindPiece(LookingFor, &mut text_seed_vec);
            LookingFor = ECPType::RMG_CP_CONSONANT;
        } else {
            if unsafe { Q_irand(0, 100) } < ComplexConsonantChance {
                ComplexConsonantChance = -1;
                LookingFor = ECPType::RMG_CP_COMPLEX_CONSONANT;
            } else {
                ComplexConsonantChance += 45;
            }

            FindPiece(LookingFor, &mut text_seed_vec);
            LookingFor = ECPType::RMG_CP_VOWEL;
        }
    }

    if Ending[0] != 0 {
        for i in 0..256 {
            if Ending[i] == 0 {
                break;
            }
            text_seed_vec.push(Ending[i]);
        }
    }

    for ch in &text_seed_vec {
        high = SeedValue >> 28;
        SeedValue ^= (SeedValue << 4) + ((*ch as c_uint) - b'a' as c_uint);
        SeedValue ^= high;
    }

    SeedValue
}
