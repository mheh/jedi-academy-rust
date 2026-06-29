////////////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK II
//  (c) 2002 Activision
//
// World Effects
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_assignments
)]

// #include "../qcommon/exe_headers.h"
use crate::codemp::qcommon::exe_headers_h::*;

// #pragma warning( disable : 4512 )

// Returns a float min <= x < max (exclusive; will get max - 0.00001; but never max)
#[inline]
fn WE_flrand(min: f32, max: f32) -> f32 {
    unsafe { ((rand() as f32 * (max - min)) / (RAND_MAX as f32 + 1.0_f32)) + min }
}

////////////////////////////////////////////////////////////////////////////////////////
// Externs & Fwd Decl.
////////////////////////////////////////////////////////////////////////////////////////
extern "C" {
    fn ParseVector(text: *mut *const c_char, count: c_int, v: *mut f32) -> qboolean;
    fn SetViewportAndScissor();
}

////////////////////////////////////////////////////////////////////////////////////////
// Includes
////////////////////////////////////////////////////////////////////////////////////////
// #include "tr_local.h"
use crate::codemp::renderer::tr_local_h::*;
// #include "tr_WorldEffects.h"
use crate::codemp::renderer::tr_WorldEffects_h::*;

// #include "../Ravl/CVec.h"
use crate::codemp::Ravl::CVec_h::*;
// #include "../Ratl/vector_vs.h"
use crate::codemp::Ratl::vector_vs_h::*;
// #include "../Ratl/bits_vs.h"
use crate::codemp::Ratl::bits_vs_h::*;

// #ifdef _XBOX
// #include "../win32/glw_win_dx8.h"
// #else
// #include "glext.h"
// #endif
#[cfg(feature = "xbox")]
use crate::codemp::win32::glw_win_dx8_h::*;
#[cfg(not(feature = "xbox"))]
use crate::codemp::renderer::glext_h::*;

use core::ffi::{c_int, c_char, c_uint};
use core::ptr::{addr_of, addr_of_mut};

// C stdlib declarations (from system headers)
extern "C" {
    fn rand() -> c_int;
    fn srand(seed: c_uint);
    fn atoi(s: *const c_char) -> c_int;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;
}

// Standard RAND_MAX constant (consistent with tr_noise.rs in this codebase)
const RAND_MAX: c_int = 2147483647;

// ulong: C unsigned long, 32-bit on original Windows/MSVC JK2 target
type ulong = u32;

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
const GLS_ALPHA: c_int = GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA;
const MAX_WIND_ZONES: usize = 10;
const MAX_WEATHER_ZONES: usize = 10;
const MAX_PUFF_SYSTEMS: usize = 2;
const MAX_PARTICLE_CLOUDS: usize = 5;

// #ifdef _XBOX
// Note to Vv:
// you guys may want to look into lowering that number.  I've optimized the storage
// space by breaking it up into small boxes (weather zones) around the areas we care about
// in order to speed up load time and reduce memory.  A very high number here will mean
// that weather related effects like rain, fog, snow, etc will bleed through to where
// they shouldn't...
// #else
// Both _XBOX and non-_XBOX branches define the same value:
const POINTCACHE_CELL_SIZE: f32 = 96.0;

////////////////////////////////////////////////////////////////////////////////////////
// Globals
////////////////////////////////////////////////////////////////////////////////////////
static mut mMillisecondsElapsed: f32 = 0.0;
static mut mSecondsElapsed: f32 = 0.0;
static mut mFrozen: bool = false;

static mut mGlobalWindVelocity: CVec3 = unsafe { core::mem::zeroed() };
static mut mGlobalWindDirection: CVec3 = unsafe { core::mem::zeroed() };
static mut mGlobalWindSpeed: f32 = 0.0;
static mut mParticlesRendered: c_int = 0;

////////////////////////////////////////////////////////////////////////////////////////
// Handy Functions
////////////////////////////////////////////////////////////////////////////////////////
#[inline]
fn VectorFloor(in_: vec3_t) {
    // In C++, vec3_t (float[3]) decays to float* - modifies array in place through pointer
    // Porting note: vec3_t passed by value in Rust loses C decay-to-pointer semantics;
    // these functions are dead code in this translation unit.
    unsafe {
        let p: *mut f32 = in_.as_ptr() as *mut f32;
        *p = (*p).floor();
        *p.add(1) = (*p.add(1)).floor();
        *p.add(2) = (*p.add(2)).floor();
    }
}

#[inline]
fn VectorCeil(in_: vec3_t) {
    unsafe {
        let p: *mut f32 = in_.as_ptr() as *mut f32;
        *p = (*p).ceil();
        *p.add(1) = (*p.add(1)).ceil();
        *p.add(2) = (*p.add(2)).ceil();
    }
}

#[inline]
fn FloatRand() -> f32 {
    unsafe { rand() as f32 / RAND_MAX as f32 }
}

#[inline]
fn SnapFloatToGrid(f: &mut f32, GridSize: c_int) {
    *f = *f as c_int as f32;

    let fNeg: bool = *f < 0.0;
    if fNeg {
        *f *= -1.0; // Temporarly make it positive
    }

    let mut Offset: c_int = (*f as c_int) % GridSize;
    let OffsetAbs: c_int = Offset.abs();
    if OffsetAbs > (GridSize / 2) {
        Offset = (GridSize - OffsetAbs) * -1;
    }

    *f -= Offset as f32;

    if fNeg {
        *f *= -1.0; // Put It Back To Negative
    }

    *f = *f as c_int as f32;

    debug_assert!((*f as c_int % GridSize) == 0);
}

#[inline]
fn SnapVectorToGrid(Vec: &mut CVec3, GridSize: c_int) {
    SnapFloatToGrid(&mut Vec[0], GridSize);
    SnapFloatToGrid(&mut Vec[1], GridSize);
    SnapFloatToGrid(&mut Vec[2], GridSize);
}

////////////////////////////////////////////////////////////////////////////////////////
// Range Structures
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct SVecRange {
    mMins: CVec3,
    mMaxs: CVec3,
}

impl SVecRange {
    #[inline]
    fn Clear(&mut self) {
        unsafe {
            self.mMins.Clear();
            self.mMaxs.Clear();
        }
    }

    #[inline]
    fn Pick(&self, V: &mut CVec3) {
        V[0] = WE_flrand(self.mMins[0], self.mMaxs[0]);
        V[1] = WE_flrand(self.mMins[1], self.mMaxs[1]);
        V[2] = WE_flrand(self.mMins[2], self.mMaxs[2]);
    }

    #[inline]
    fn Wrap(&mut self, V: &mut CVec3, _spawnRange: &SVecRange) {
        if V[0] < self.mMins[0] {
            let d = self.mMins[0] - V[0];
            V[0] = self.mMaxs[0] - (d % (self.mMaxs[0] - self.mMins[0]));
        }
        if V[0] > self.mMaxs[0] {
            let d = V[0] - self.mMaxs[0];
            V[0] = self.mMins[0] + (d % (self.mMaxs[0] - self.mMins[0]));
        }

        if V[1] < self.mMins[1] {
            let d = self.mMins[1] - V[1];
            V[1] = self.mMaxs[1] - (d % (self.mMaxs[1] - self.mMins[1]));
        }
        if V[1] > self.mMaxs[1] {
            let d = V[1] - self.mMaxs[1];
            V[1] = self.mMins[1] + (d % (self.mMaxs[1] - self.mMins[1]));
        }

        if V[2] < self.mMins[2] {
            let d = self.mMins[2] - V[2];
            V[2] = self.mMaxs[2] - (d % (self.mMaxs[2] - self.mMins[2]));
        }
        if V[2] > self.mMaxs[2] {
            let d = V[2] - self.mMaxs[2];
            V[2] = self.mMins[2] + (d % (self.mMaxs[2] - self.mMins[2]));
        }
    }

    #[inline]
    fn In(&self, V: &CVec3) -> bool {
        *V > self.mMins && *V < self.mMaxs
    }
}

#[repr(C)]
struct SFloatRange {
    mMin: f32,
    mMax: f32,
}

impl SFloatRange {
    #[inline]
    fn Clear(&mut self) {
        self.mMin = 0.0;
        self.mMin = 0.0; // BUG preserved from original: assigns mMin twice (should be mMax)
    }
    #[inline]
    fn Pick(&self, V: &mut f32) {
        *V = WE_flrand(self.mMin, self.mMax);
    }
    #[inline]
    fn In(&self, V: f32) -> bool {
        V > self.mMin && V < self.mMax
    }
}

#[repr(C)]
struct SIntRange {
    mMin: c_int,
    mMax: c_int,
}

impl SIntRange {
    #[inline]
    fn Clear(&mut self) {
        self.mMin = 0;
        self.mMin = 0; // BUG preserved from original: assigns mMin twice (should be mMax)
    }
    #[inline]
    fn Pick(&self, V: &mut c_int) {
        unsafe { *V = Q_irand(self.mMin, self.mMax); }
    }
    #[inline]
    fn In(&self, V: c_int) -> bool {
        V > self.mMin && V < self.mMax
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Particle Class
////////////////////////////////////////////////////////////////////////////////////////
// typedef ratl::bits_vs<FLAG_MAX> TFlags  (CWeatherParticle::TFlags)
// FLAG_MAX = 4
type TFlags = bits_vs<4>;

#[repr(C)]
struct CWeatherParticle {
    // enum { FLAG_RENDER=0, FLAG_FADEIN, FLAG_FADEOUT, FLAG_RESPAWN, FLAG_MAX }
    mAlpha: f32,
    mFlags: TFlags,
    mPosition: CVec3,
    mVelocity: CVec3,
    mMass: f32, // A higher number will more greatly resist force and result in greater gravity
}

impl CWeatherParticle {
    const FLAG_RENDER: usize = 0;
    const FLAG_FADEIN: usize = 1;
    const FLAG_FADEOUT: usize = 2;
    const FLAG_RESPAWN: usize = 3;
    const FLAG_MAX: usize = 4;
}

////////////////////////////////////////////////////////////////////////////////////////
// The Wind
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct CWindZone {
    mGlobal: bool,
    mRBounds: SVecRange,
    mRVelocity: SVecRange,
    mRDuration: SIntRange,
    mRDeadTime: SIntRange,
    mMaxDeltaVelocityPerUpdate: f32,
    mChanceOfDeadTime: f32,

    mCurrentVelocity: CVec3,
    mTargetVelocity: CVec3,
    mTargetVelocityTimeRemaining: c_int,
}

impl CWindZone {
    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize - Will setup default values for all data
    ////////////////////////////////////////////////////////////////////////////////////
    fn Initialize(&mut self) {
        unsafe {
            self.mRBounds.Clear();
            self.mGlobal = true;

            self.mRVelocity.mMins = -1500.0_f32;
            self.mRVelocity.mMins[2] = -10.0_f32;
            self.mRVelocity.mMaxs = 1500.0_f32;
            self.mRVelocity.mMaxs[2] = 10.0_f32;

            self.mMaxDeltaVelocityPerUpdate = 10.0_f32;

            self.mRDuration.mMin = 1000;
            self.mRDuration.mMax = 2000;

            self.mChanceOfDeadTime = 0.3_f32;
            self.mRDeadTime.mMin = 1000;
            self.mRDeadTime.mMax = 3000;

            self.mCurrentVelocity.Clear();
            self.mTargetVelocity.Clear();
            self.mTargetVelocityTimeRemaining = 0;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Update - Changes wind when current target velocity expires
    ////////////////////////////////////////////////////////////////////////////////////
    fn Update(&mut self) {
        unsafe {
            if self.mTargetVelocityTimeRemaining == 0 {
                if FloatRand() < self.mChanceOfDeadTime {
                    self.mRDeadTime.Pick(&mut self.mTargetVelocityTimeRemaining);
                    self.mTargetVelocity.Clear();
                } else {
                    self.mRDuration.Pick(&mut self.mTargetVelocityTimeRemaining);
                    self.mRVelocity.Pick(&mut self.mTargetVelocity);
                }
            } else if self.mTargetVelocityTimeRemaining != -1 {
                self.mTargetVelocityTimeRemaining -= 1;

                let mut DeltaVelocity: CVec3 = self.mTargetVelocity - self.mCurrentVelocity;
                let mut DeltaVelocityLen: f32 = VectorNormalize(DeltaVelocity.v.as_mut_ptr());
                if DeltaVelocityLen > self.mMaxDeltaVelocityPerUpdate {
                    DeltaVelocityLen = self.mMaxDeltaVelocityPerUpdate;
                }
                DeltaVelocity *= DeltaVelocityLen;
                self.mCurrentVelocity += DeltaVelocity;
            }
        }
    }
}

static mut mWindZones: vector_vs<CWindZone, MAX_WIND_ZONES> = unsafe { core::mem::zeroed() };

pub fn R_GetWindVector(windVector: vec3_t) -> bool {
    // In C++: VectorCopy(mGlobalWindDirection.v, windVector)
    // vec3_t decays to float* in C — windVector is ptr to caller's array
    // In Rust, vec3_t is passed by value; porting note: modification of caller's data
    // requires the caller to pass a pointer. Preserving the C call pattern faithfully:
    unsafe {
        VectorCopy(
            mGlobalWindDirection.v.as_ptr() as *mut f32,
            windVector.as_ptr() as *mut f32,
        );
    }
    true
}

pub fn R_GetWindSpeed(windSpeed: &mut f32) -> bool {
    unsafe { *windSpeed = mGlobalWindSpeed; }
    true
}

pub fn R_GetWindGusting() -> bool {
    unsafe { mGlobalWindSpeed > 1000.0_f32 }
}

////////////////////////////////////////////////////////////////////////////////////////
// Outside Point Cache
////////////////////////////////////////////////////////////////////////////////////////

// COutside::SWeatherZone::mMarkedOutside - static member, defined at file scope below class
// bool COutside::SWeatherZone::mMarkedOutside = false;
static mut SWeatherZone_mMarkedOutside: bool = false;

#[repr(C)]
struct SWeatherZone {
    // static bool mMarkedOutside; -- translated as module-level SWeatherZone_mMarkedOutside
    mPointCache: *mut ulong,
    mExtents: SVecRange,
    mSize: SVecRange,
    mWidth: c_int,
    mHeight: c_int,
    mDepth: c_int,
}

impl SWeatherZone {
    ////////////////////////////////////////////////////////////////////////////////////
    // Convert To Cell
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn ConvertToCell(&self, pos: &CVec3, x: &mut c_int, y: &mut c_int, z: &mut c_int, bit: &mut c_int) {
        *x = ((pos[0] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[0]) as c_int;
        *y = ((pos[1] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[1]) as c_int;
        *z = ((pos[2] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[2]) as c_int;

        *bit = *z & 31;
        *z >>= 5;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // CellOutside - Test to see if a given cell is outside
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn CellOutside(&self, x: c_int, y: c_int, z: c_int, bit: c_int) -> bool {
        unsafe {
            if (x < 0 || x >= self.mWidth)
                || (y < 0 || y >= self.mHeight)
                || (z < 0 || z >= self.mDepth)
                || (bit < 0 || bit >= 32)
            {
                return !SWeatherZone_mMarkedOutside;
            }
            let idx = ((z * self.mWidth * self.mHeight) + (y * self.mWidth) + x) as usize;
            let entry: ulong = *self.mPointCache.add(idx);
            SWeatherZone_mMarkedOutside == ((entry & (1u32 << bit)) != 0)
        }
    }
}

#[repr(C)]
struct COutside {
    ////////////////////////////////////////////////////////////////////////////////////
    //Global Public Outside Variables
    ////////////////////////////////////////////////////////////////////////////////////
    mOutsideShake: bool,
    mOutsidePain: f32,

    ////////////////////////////////////////////////////////////////////////////////////
    // The Outside Cache
    ////////////////////////////////////////////////////////////////////////////////////
    mCacheInit: bool, // Has It Been Cached?

    // ratl::vector_vs<SWeatherZone, MAX_WEATHER_ZONES> mWeatherZones
    mWeatherZones: vector_vs<SWeatherZone, MAX_WEATHER_ZONES>,

    ////////////////////////////////////////////////////////////////////////////////////
    // Iteration Variables
    ////////////////////////////////////////////////////////////////////////////////////
    mWCells: c_int,
    mHCells: c_int,

    mXCell: c_int,
    mYCell: c_int,
    mZBit: c_int,

    mXMax: c_int,
    mYMax: c_int,
    mZMax: c_int,
}

impl COutside {
    ////////////////////////////////////////////////////////////////////////////////////
    // Contents Outside
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn ContentsOutside(&self, contents: c_int) -> bool {
        unsafe {
            if contents & CONTENTS_WATER != 0 || contents & CONTENTS_SOLID != 0 {
                return false;
            }
            if self.mCacheInit {
                if SWeatherZone_mMarkedOutside {
                    return (contents & CONTENTS_OUTSIDE) != 0;
                }
                return (contents & CONTENTS_INSIDE) == 0;
            }
            (contents & CONTENTS_OUTSIDE) != 0
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor - Will setup default values for all data
    ////////////////////////////////////////////////////////////////////////////////////
    fn Reset(&mut self) {
        unsafe {
            self.mOutsideShake = false;
            self.mOutsidePain = 0.0;
            self.mCacheInit = false;
            SWeatherZone_mMarkedOutside = false;
            let wz_size = self.mWeatherZones.size();
            for wz in 0..wz_size {
                Z_Free(self.mWeatherZones[wz].mPointCache as *mut core::ffi::c_void);
                self.mWeatherZones[wz].mPointCache = core::ptr::null_mut();
            }
            self.mWeatherZones.clear();
        }
    }

    // COutside() { Reset(); }
    fn new() -> Self {
        let mut out: COutside = unsafe { core::mem::zeroed() };
        out.Reset();
        out
    }

    // ~COutside() { Reset(); }
    fn destroy(&mut self) {
        self.Reset();
    }

    fn Initialized(&self) -> bool {
        self.mCacheInit
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // AddWeatherZone - Will add a zone of mins and maxes
    ////////////////////////////////////////////////////////////////////////////////////
    fn AddWeatherZone(&mut self, mins: *mut f32, maxs: *mut f32) {
        unsafe {
            if !self.mWeatherZones.full() {
                let Wz: &mut SWeatherZone = self.mWeatherZones.push_back();
                Wz.mExtents.mMins = *(mins as *const CVec3);
                Wz.mExtents.mMaxs = *(maxs as *const CVec3);

                SnapVectorToGrid(&mut Wz.mExtents.mMins, POINTCACHE_CELL_SIZE as c_int);
                SnapVectorToGrid(&mut Wz.mExtents.mMaxs, POINTCACHE_CELL_SIZE as c_int);

                Wz.mSize.mMins = Wz.mExtents.mMins;
                Wz.mSize.mMaxs = Wz.mExtents.mMaxs;

                Wz.mSize.mMins /= POINTCACHE_CELL_SIZE;
                Wz.mSize.mMaxs /= POINTCACHE_CELL_SIZE;
                Wz.mWidth  =  (Wz.mSize.mMaxs[0] - Wz.mSize.mMins[0]) as c_int;
                Wz.mHeight =  (Wz.mSize.mMaxs[1] - Wz.mSize.mMins[1]) as c_int;
                Wz.mDepth  = (((Wz.mSize.mMaxs[2] - Wz.mSize.mMins[2]) as c_int) + 31) >> 5;

                let arraySize: c_int = Wz.mWidth * Wz.mHeight * Wz.mDepth;
                Wz.mPointCache = Z_Malloc(
                    (arraySize as usize) * core::mem::size_of::<ulong>(),
                    TAG_POINTCACHE,
                    qtrue,
                ) as *mut ulong;
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Cache - Will Scan the World, Creating The Cache
    ////////////////////////////////////////////////////////////////////////////////////
    fn Cache(&mut self) {
        unsafe {
            if tr.world.is_null() || self.mCacheInit {
                return;
            }

            let mut CurPos: CVec3 = core::mem::zeroed();
            let mut Size: CVec3 = core::mem::zeroed();
            let mut Mins: CVec3 = core::mem::zeroed();
            let mut x: c_int;
            let mut y: c_int;
            let mut z: c_int;
            let mut q: c_int;
            let mut zbase: c_int;
            let mut curPosOutside: bool;
            let mut contents: ulong;
            let mut bit: ulong;

            // Record The Extents Of The World Incase No Other Weather Zones Exist
            //---------------------------------------------------------------------
            if self.mWeatherZones.size() == 0 {
                Com_Printf(b"WARNING: No Weather Zones Encountered\0".as_ptr());
                self.AddWeatherZone(
                    (*(*tr.world).bmodels.add(0)).bounds[0].as_mut_ptr(),
                    (*(*tr.world).bmodels.add(0)).bounds[1].as_mut_ptr(),
                );
            }

            // Iterate Over All Weather Zones
            //--------------------------------
            let zone_count = self.mWeatherZones.size();
            for zone in 0..zone_count {
                let wz: SWeatherZone = core::ptr::read(&self.mWeatherZones[zone] as *const _);

                // Make Sure Point Contents Checks Occur At The CENTER Of The Cell
                //-----------------------------------------------------------------
                Mins = wz.mExtents.mMins;
                for xi in 0..3_usize {
                    Mins[xi] += POINTCACHE_CELL_SIZE / 2.0_f32;
                }

                // Start Scanning
                //----------------
                z = 0;
                while z < wz.mDepth {
                    q = 0;
                    while q < 32 {
                        bit = (1u32 << q) as ulong;
                        zbase = z << 5;

                        x = 0;
                        while x < wz.mWidth {
                            y = 0;
                            while y < wz.mHeight {
                                CurPos[0] = (x as f32)           * POINTCACHE_CELL_SIZE;
                                CurPos[1] = (y as f32)           * POINTCACHE_CELL_SIZE;
                                CurPos[2] = ((zbase + q) as f32) * POINTCACHE_CELL_SIZE;
                                CurPos += Mins;

                                contents = CM_PointContents(CurPos.v.as_mut_ptr(), 0) as ulong;
                                if contents & (CONTENTS_INSIDE as ulong) != 0
                                    || contents & (CONTENTS_OUTSIDE as ulong) != 0
                                {
                                    curPosOutside = (contents & (CONTENTS_OUTSIDE as ulong)) != 0;
                                    if !self.mCacheInit {
                                        self.mCacheInit = true;
                                        SWeatherZone_mMarkedOutside = curPosOutside;
                                    } else if SWeatherZone_mMarkedOutside != curPosOutside {
                                        debug_assert!(false);
                                        Com_Error(
                                            ERR_DROP,
                                            b"Weather Effect: Both Indoor and Outdoor brushs encountered in map.\n\0"
                                                .as_ptr(),
                                        );
                                        return;
                                    }

                                    // Mark The Point
                                    //----------------
                                    let idx = ((z * wz.mWidth * wz.mHeight)
                                        + (y * wz.mWidth)
                                        + x) as usize;
                                    *wz.mPointCache.add(idx) |= bit;
                                }
                                y += 1;
                            } // for (y)
                            x += 1;
                        } // for (x)
                        q += 1;
                    } // for (q)
                    z += 1;
                } // for (z)
            }

            // If no indoor or outdoor brushes were found
            //--------------------------------------------
            if !self.mCacheInit {
                self.mCacheInit = true;
                SWeatherZone_mMarkedOutside = false; // Assume All Is Outside, Except Solid
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // PointOutside - Test to see if a given point is outside
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn PointOutside(&self, pos: &CVec3) -> bool {
        unsafe {
            if !self.mCacheInit {
                return self.ContentsOutside(CM_PointContents(pos.v.as_ptr() as *mut f32, 0));
            }
            let zone_count = self.mWeatherZones.size();
            for zone in 0..zone_count {
                let wz: SWeatherZone = core::ptr::read(&self.mWeatherZones[zone] as *const _);
                if wz.mExtents.In(pos) {
                    let mut bit: c_int = 0;
                    let mut x: c_int = 0;
                    let mut y: c_int = 0;
                    let mut z: c_int = 0;
                    wz.ConvertToCell(pos, &mut x, &mut y, &mut z, &mut bit);
                    return wz.CellOutside(x, y, z, bit);
                }
            }
            !SWeatherZone_mMarkedOutside
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // PointOutside - Test to see if a given bounded plane is outside
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn PointOutside_bounded(&mut self, pos: &CVec3, width: f32, height: f32) -> bool {
        unsafe {
            let zone_count = self.mWeatherZones.size();
            for zone in 0..zone_count {
                let wz: SWeatherZone = core::ptr::read(&self.mWeatherZones[zone] as *const _);
                if wz.mExtents.In(pos) {
                    let mut bit: c_int = 0;
                    let mut x: c_int = 0;
                    let mut y: c_int = 0;
                    let mut z: c_int = 0;
                    wz.ConvertToCell(pos, &mut x, &mut y, &mut z, &mut bit);
                    if width < POINTCACHE_CELL_SIZE || height < POINTCACHE_CELL_SIZE {
                        return wz.CellOutside(x, y, z, bit);
                    }

                    self.mWCells = (width as c_int) / (POINTCACHE_CELL_SIZE as c_int);
                    self.mHCells = (height as c_int) / (POINTCACHE_CELL_SIZE as c_int);

                    self.mXMax = x + self.mWCells;
                    self.mYMax = y + self.mWCells;
                    self.mZMax = bit + self.mHCells;

                    self.mXCell = x - self.mWCells;
                    while self.mXCell <= self.mXMax {
                        self.mYCell = y - self.mWCells;
                        while self.mYCell <= self.mYMax {
                            self.mZBit = bit - self.mHCells;
                            while self.mZBit <= self.mZMax {
                                if !wz.CellOutside(self.mXCell, self.mYCell, z, self.mZBit) {
                                    return false;
                                }
                                self.mZBit += 1;
                            }
                            self.mYCell += 1;
                        }
                        self.mXCell += 1;
                    }
                    return true;
                }
            }
            !SWeatherZone_mMarkedOutside
        }
    }
}

static mut mOutside: COutside = unsafe { core::mem::zeroed() };
// bool COutside::SWeatherZone::mMarkedOutside = false;
// (definition handled by SWeatherZone_mMarkedOutside static above)

pub fn R_AddWeatherZone(mins: *mut f32, maxs: *mut f32) {
    unsafe { mOutside.AddWeatherZone(mins, maxs); }
}

pub fn R_IsOutside(pos: *mut f32) -> bool {
    unsafe {
        let v: &CVec3 = &*(pos as *const CVec3);
        mOutside.PointOutside(v)
    }
}

pub fn R_IsShaking() -> bool {
    unsafe {
        let origin: &CVec3 = &*(backEnd.viewParms.ori.origin.as_ptr() as *const CVec3);
        mOutside.mOutsideShake && mOutside.PointOutside(origin)
    }
}

pub fn R_IsOutsideCausingPain(pos: *mut f32) -> f32 {
    // C++ returns float: (float)(mOutside.mOutsidePain && mOutside.PointOutside(pos))
    unsafe {
        let v: &CVec3 = &*(pos as *const CVec3);
        (mOutside.mOutsidePain != 0.0 && mOutside.PointOutside(v)) as c_int as f32
    }
}

// #ifdef _XBOX
#[cfg(feature = "xbox")]
unsafe fn pointBegin(verts: c_int, size: f32) {
    debug_assert!(!(*glw_state).inDrawBlock);

    // start the draw block
    (*glw_state).inDrawBlock = true;
    (*glw_state).primitiveMode = D3DPT_POINTLIST;

    // update DX with any pending state changes
    (*glw_state).drawStride = 4;
    let mask: DWORD = D3DFVF_XYZ | D3DFVF_DIFFUSE;
    (*(*glw_state).device).SetVertexShader(mask);
    (*glw_state).shaderMask = mask;

    if (*glw_state).matricesDirty[glwstate_t_MatrixMode_Model as usize] {
        (*(*glw_state).device).SetTransform(
            D3DTS_VIEW,
            (*glw_state).matrixStack[glwstate_t_MatrixMode_Model as usize].GetTop(),
        );
        (*glw_state).matricesDirty[glwstate_t_MatrixMode_Model as usize] = false;
    }

    // Update the texture and states
    // NOTE: Point sprites ALWAYS go on texture stage 3
    // glwstate_t::texturexlat_t::iterator it = glw_state->textureXlat.find(glw_state->currentTexture[0]);
    let it = (*glw_state)
        .textureXlat
        .find((*glw_state).currentTexture[0]);
    (*(*glw_state).device).SetTexture(3, (*it).second.mipmap);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_COLOROP,   (*glw_state).textureEnv[0]);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_COLORARG1, D3DTA_TEXTURE);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_COLORARG2, D3DTA_CURRENT);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_ALPHAOP,   (*glw_state).textureEnv[0]);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_ALPHAARG1, D3DTA_TEXTURE);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_ALPHAARG2, D3DTA_CURRENT);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_MAXANISOTROPY, (*it).second.anisotropy);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_MINFILTER, (*it).second.minFilter);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_MIPFILTER, (*it).second.mipFilter);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_MAGFILTER, (*it).second.magFilter);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_ADDRESSU,  (*it).second.wrapU);
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_ADDRESSV,  (*it).second.wrapV);

    (*(*glw_state).device).SetTexture(0, core::ptr::null_mut());
    (*(*glw_state).device).SetTextureStageState(0, D3DTSS_COLOROP, D3DTOP_DISABLE);

    let attena: f32 = 1.0_f32;
    let attenb: f32 = 0.0_f32;
    let attenc: f32 = 0.0004_f32;
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSPRITEENABLE, TRUE);
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSCALEENABLE, TRUE);
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSIZE,     *((&size) as *const f32 as *const DWORD));
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSIZE_MIN, *((&attenb) as *const f32 as *const DWORD));
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSCALE_A,  *((&attena) as *const f32 as *const DWORD));
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSCALE_B,  *((&attenb) as *const f32 as *const DWORD));
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSCALE_C,  *((&attenc) as *const f32 as *const DWORD));

    // set vertex counters
    (*glw_state).numVertices = 0;
    (*glw_state).totalVertices = verts;
    let mut max: c_int = (*glw_state).totalVertices;
    if max > 2040 / (*glw_state).drawStride {
        max = 2040 / (*glw_state).drawStride;
    }
    (*glw_state).maxVertices = max;

    // open a draw packet
    let num_packets: c_int;
    if verts == 0 {
        num_packets = 1;
    } else {
        num_packets = (verts / (*glw_state).maxVertices)
            + (if (verts % (*glw_state).maxVertices) != 0 { 1 } else { 0 });
    }
    let cmd_size: c_int = num_packets * 3;
    let vert_size: c_int = (*glw_state).drawStride * verts;

    (*(*glw_state).device).BeginPush(
        (vert_size + cmd_size + 2) as u32,
        &mut (*glw_state).drawArray,
    );

    *(*glw_state).drawArray.add(0) = D3DPUSH_ENCODE(D3DPUSH_SET_BEGIN_END, 1);
    *(*glw_state).drawArray.add(1) = (*glw_state).primitiveMode;
    *(*glw_state).drawArray.add(2) = D3DPUSH_ENCODE(
        D3DPUSH_NOINCREMENT_FLAG | D3DPUSH_INLINE_ARRAY,
        (*glw_state).drawStride * (*glw_state).maxVertices,
    );
    (*glw_state).drawArray = (*glw_state).drawArray.add(3);
}

#[cfg(feature = "xbox")]
unsafe fn pointEnd() {
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSPRITEENABLE, FALSE);
    (*(*glw_state).device).SetRenderState(D3DRS_POINTSCALEENABLE, FALSE);
    (*(*glw_state).device).SetTexture(3, core::ptr::null_mut());
    (*(*glw_state).device).SetTextureStageState(3, D3DTSS_COLOROP, D3DTOP_DISABLE);
}
// #endif // _XBOX

////////////////////////////////////////////////////////////////////////////////////////
// Particle Cloud
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct CWeatherParticleCloud {
    ////////////////////////////////////////////////////////////////////////////////////
    // DYNAMIC MEMORY
    ////////////////////////////////////////////////////////////////////////////////////
    mImage: *mut image_t,
    mParticles: *mut CWeatherParticle,

    ////////////////////////////////////////////////////////////////////////////////////
    // RUN TIME VARIANTS
    ////////////////////////////////////////////////////////////////////////////////////
    mSpawnSpeed: f32,
    mSpawnPlaneNorm: CVec3,
    mSpawnPlaneRight: CVec3,
    mSpawnPlaneUp: CVec3,
    mRange: SVecRange,

    mCameraPosition: CVec3,
    mCameraForward: CVec3,
    mCameraLeft: CVec3,
    mCameraDown: CVec3,
    mCameraLeftPlusUp: CVec3,
    mCameraLeftMinusUp: CVec3,

    mParticleCountRender: c_int,
    mGLModeEnum: c_int,

    mPopulated: bool,

    ////////////////////////////////////////////////////////////////////////////////////
    // CONSTANTS
    ////////////////////////////////////////////////////////////////////////////////////
    mOrientWithVelocity: bool,
    mSpawnPlaneSize: f32,
    mSpawnPlaneDistance: f32,
    mSpawnRange: SVecRange,

    mGravity: f32,       // How much gravity affects the velocity of a particle
    mColor: CVec4,       // RGBA color
    mVertexCount: c_int, // 3 for triangle, 4 for quad, other numbers not supported

    mWidth: f32,
    mHeight: f32,

    mBlendMode: c_int,  // 0 = ALPHA, 1 = SRC->SRC
    mFilterMode: c_int, // 0 = LINEAR, 1 = NEAREST

    mFade: f32, // How much to fade in and out 1.0 = instant, 0.01 = very slow

    mRotation: SFloatRange,
    mRotationDelta: f32,
    mRotationDeltaTarget: f32,
    mRotationCurrent: f32,
    mRotationChangeTimer: SIntRange,
    mRotationChangeNext: c_int,

    mMass: SFloatRange, // Determines how slowness to accelerate, higher number = slower
    mFrictionInverse: f32, // How much air friction does this particle have 1.0=none, 0.0=nomove

    mParticleCount: c_int,

    mWaterParticles: bool,
}

impl CWeatherParticleCloud {
    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize - Create Image, Particles, And Setup All Values
    ////////////////////////////////////////////////////////////////////////////////////
    fn Initialize(&mut self, count: c_int, texturePath: *const c_char, VertexCount: c_int) {
        unsafe {
            self.Reset();
            debug_assert!(self.mParticleCount == 0 && self.mParticles.is_null());
            debug_assert!(self.mImage.is_null());

            // Create The Image
            //------------------
            self.mImage = R_FindImageFile(texturePath, qfalse, qfalse, qfalse, GL_CLAMP);
            if self.mImage.is_null() {
                Com_Error(
                    ERR_DROP,
                    b"CWeatherParticleCloud: Could not texture %s\0".as_ptr(),
                    texturePath,
                );
            }

            GL_Bind(self.mImage);

            // Create The Particles
            //----------------------
            self.mParticleCount = count;
            let sz = self.mParticleCount as usize;
            let mut v: Vec<CWeatherParticle> = Vec::with_capacity(sz);
            for _ in 0..sz {
                v.push(core::mem::zeroed());
            }
            let boxed = v.into_boxed_slice();
            self.mParticles = Box::into_raw(boxed) as *mut CWeatherParticle;

            let mut part: *mut CWeatherParticle = core::ptr::null_mut();
            for particleNum in 0..self.mParticleCount {
                part = &mut *self.mParticles.add(particleNum as usize);
                (*part).mPosition.Clear();
                (*part).mVelocity.Clear();
                (*part).mAlpha = 0.0_f32;
                self.mMass.Pick(&mut (*part).mMass);
            }

            self.mVertexCount = VertexCount;

            // #ifdef _XBOX
            // if(mVertexCount == 1)
            //     mGLModeEnum = GL_POINTS;
            // else
            // #endif
            #[cfg(feature = "xbox")]
            {
                if self.mVertexCount == 1 {
                    self.mGLModeEnum = GL_POINTS;
                } else {
                    self.mGLModeEnum = if self.mVertexCount == 3 { GL_TRIANGLES } else { GL_QUADS };
                }
            }
            #[cfg(not(feature = "xbox"))]
            {
                self.mGLModeEnum = if self.mVertexCount == 3 { GL_TRIANGLES } else { GL_QUADS };
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Reset - Initializes all data to default values
    ////////////////////////////////////////////////////////////////////////////////////
    fn Reset(&mut self) {
        unsafe {
            if !self.mImage.is_null() {
                // TODO: Free Image?
            }
            self.mImage = core::ptr::null_mut();
            if self.mParticleCount != 0 {
                let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                    self.mParticles,
                    self.mParticleCount as usize,
                ));
            }
            self.mParticleCount = 0;
            self.mParticles = core::ptr::null_mut();

            self.mPopulated = false;

            // These Are The Default Startup Values For Constant Data
            //========================================================
            self.mOrientWithVelocity = false;
            self.mWaterParticles = false;

            self.mSpawnPlaneDistance = 500.0_f32;
            self.mSpawnPlaneSize = 500.0_f32;
            self.mSpawnRange.mMins = -(self.mSpawnPlaneDistance * 1.25_f32);
            self.mSpawnRange.mMaxs =  (self.mSpawnPlaneDistance * 1.25_f32);

            self.mGravity = 300.0_f32; // Units Per Second

            self.mColor = 1.0_f32;

            self.mVertexCount = 4;
            self.mWidth = 1.0_f32;
            self.mHeight = 1.0_f32;

            self.mBlendMode = 0;
            self.mFilterMode = 0;

            self.mFade = 10.0_f32;

            self.mRotation.Clear();
            self.mRotationDelta = 0.0_f32;
            self.mRotationDeltaTarget = 0.0_f32;
            self.mRotationCurrent = 0.0_f32;
            self.mRotationChangeNext = -1;
            self.mRotation.mMin = -0.7_f32;
            self.mRotation.mMax =  0.7_f32;
            self.mRotationChangeTimer.mMin = 500;
            self.mRotationChangeTimer.mMin = 2000; // BUG preserved from original: assigns mMin twice (should be mMax)

            self.mMass.mMin = 5.0_f32;
            self.mMass.mMax = 10.0_f32;

            self.mFrictionInverse = 0.7_f32; // No Friction?
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor - Will setup default values for all data
    ////////////////////////////////////////////////////////////////////////////////////
    fn new() -> Self {
        let mut cloud: CWeatherParticleCloud = unsafe { core::mem::zeroed() };
        cloud.mImage = core::ptr::null_mut();
        cloud.mParticleCount = 0;
        cloud.Reset();
        cloud
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize - Will setup default values for all data  [destructor comment preserved]
    ////////////////////////////////////////////////////////////////////////////////////
    fn destroy(&mut self) {
        self.Reset();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // UseSpawnPlane - Check To See If We Should Spawn On A Plane, Or Just Wrap The Box
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    fn UseSpawnPlane(&self) -> bool {
        self.mGravity != 0.0_f32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Update - Applies All Physics Forces To All Contained Particles
    ////////////////////////////////////////////////////////////////////////////////////
    fn Update(&mut self) {
        unsafe {
            let mut part: *mut CWeatherParticle = core::ptr::null_mut();
            let mut partForce: CVec3 = core::mem::zeroed();
            let mut partMoved: CVec3 = core::mem::zeroed();
            let mut partToCamera: CVec3 = core::mem::zeroed();
            let mut partRendering: bool;
            let mut partOutside: bool;
            let mut partInRange: bool;
            let mut partInView: bool;
            let mut particleNum: c_int;
            let particleFade: f32 = self.mFade * mSecondsElapsed;

            /* TODO: Non Global Wind Zones
                    CWindZone*	wind=0;
                    int			windNum;
                    int			windCount = mWindZones.size();
            */

            // Compute Camera
            //----------------
            {
                self.mCameraPosition = backEnd.viewParms.ori.origin;
                self.mCameraForward  = backEnd.viewParms.ori.axis[0];
                self.mCameraLeft     = backEnd.viewParms.ori.axis[1];
                self.mCameraDown     = backEnd.viewParms.ori.axis[2];

                if self.mRotationChangeNext != -1 {
                    if self.mRotationChangeNext == 0 {
                        self.mRotation.Pick(&mut self.mRotationDeltaTarget);
                        self.mRotationChangeTimer.Pick(&mut self.mRotationChangeNext);
                        if self.mRotationChangeNext <= 0 {
                            self.mRotationChangeNext = 1;
                        }
                    }
                    self.mRotationChangeNext -= 1;

                    let RotationDeltaDifference: f32 = self.mRotationDeltaTarget - self.mRotationDelta;
                    if RotationDeltaDifference.abs() > 0.01_f32 {
                        self.mRotationDelta += RotationDeltaDifference; // Blend To New Delta
                    }
                    self.mRotationCurrent += self.mRotationDelta * mSecondsElapsed;
                    let s: f32 = self.mRotationCurrent.sin();
                    let c: f32 = self.mRotationCurrent.cos();

                    let TempCamLeft: CVec3 = self.mCameraLeft;

                    self.mCameraLeft *= c * self.mWidth;
                    self.mCameraLeft.ScaleAdd(self.mCameraDown, s * self.mWidth * -1.0_f32);

                    self.mCameraDown *= c * self.mHeight;
                    self.mCameraDown.ScaleAdd(TempCamLeft, s * self.mHeight);
                } else {
                    self.mCameraLeft *= self.mWidth;
                    self.mCameraDown *= self.mHeight;
                }
            }

            // Compute Global Force
            //----------------------
            let mut force: CVec3 = core::mem::zeroed();
            {
                force.Clear();

                // Apply Gravity
                //---------------
                force[2] = -1.0_f32 * self.mGravity;

                // Apply Wind Velocity
                //---------------------
                force += mGlobalWindVelocity;
            }

            // Update Range
            //--------------
            {
                self.mRange.mMins = self.mCameraPosition + self.mSpawnRange.mMins;
                self.mRange.mMaxs = self.mCameraPosition + self.mSpawnRange.mMaxs;

                // If Using A Spawn Plane, Increase The Range Box A Bit To Account For Rotation On The Spawn Plane
                //-------------------------------------------------------------------------------------------------
                if self.UseSpawnPlane() {
                    for dim in 0..3_usize {
                        if force[dim] > 0.01_f32 {
                            self.mRange.mMins[dim] -= self.mSpawnPlaneDistance / 2.0_f32;
                        } else if force[dim] < -0.01_f32 {
                            self.mRange.mMaxs[dim] += self.mSpawnPlaneDistance / 2.0_f32;
                        }
                    }
                    self.mSpawnPlaneNorm = force;
                    self.mSpawnSpeed = VectorNormalize(self.mSpawnPlaneNorm.v.as_mut_ptr());
                    MakeNormalVectors(
                        self.mSpawnPlaneNorm.v.as_mut_ptr(),
                        self.mSpawnPlaneRight.v.as_mut_ptr(),
                        self.mSpawnPlaneUp.v.as_mut_ptr(),
                    );
                    if self.mOrientWithVelocity {
                        self.mCameraDown = self.mSpawnPlaneNorm;
                        self.mCameraDown *= self.mHeight * -1.0_f32;
                    }
                }

                // Optimization For Quad Position Calculation
                //--------------------------------------------
                if self.mVertexCount == 4 {
                    self.mCameraLeftPlusUp  = self.mCameraLeft - self.mCameraDown;
                    self.mCameraLeftMinusUp = self.mCameraLeft + self.mCameraDown;
                } else {
                    self.mCameraLeftPlusUp  = self.mCameraDown + self.mCameraLeft; // should really be called mCamera Left + Down
                }
            }

            // Stop All Additional Processing
            //--------------------------------
            if mFrozen {
                return;
            }

            // Now Update All Particles
            //--------------------------
            self.mParticleCountRender = 0;
            particleNum = 0;
            while particleNum < self.mParticleCount {
                part = &mut *self.mParticles.add(particleNum as usize);

                if !self.mPopulated {
                    self.mRange.Pick(&mut (*part).mPosition); // First Time Spawn Location
                }

                // Grab The Force And Apply Non Global Wind
                //------------------------------------------
                partForce = force;
                partForce /= (*part).mMass;

                // Apply The Force
                //-----------------
                (*part).mVelocity += partForce;
                (*part).mVelocity *= self.mFrictionInverse;

                (*part).mPosition.ScaleAdd((*part).mVelocity, mSecondsElapsed);

                partToCamera = (*part).mPosition - self.mCameraPosition;
                partRendering = (*part).mFlags.get_bit(CWeatherParticle::FLAG_RENDER);
                partOutside = (*addr_of_mut!(mOutside)).PointOutside_bounded(
                    &(*part).mPosition,
                    self.mWidth,
                    self.mHeight,
                );
                partInRange = self.mRange.In(&(*part).mPosition);
                partInView = partOutside
                    && partInRange
                    && (partToCamera.Dot(self.mCameraForward) > 0.0_f32);

                // Process Respawn
                //-----------------
                if !partInRange && !partRendering {
                    (*part).mVelocity.Clear();

                    // Reselect A Position On The Spawn Plane
                    //----------------------------------------
                    if self.UseSpawnPlane() {
                        (*part).mPosition = self.mCameraPosition;
                        (*part).mPosition -= self.mSpawnPlaneNorm * self.mSpawnPlaneDistance;
                        (*part).mPosition +=
                            self.mSpawnPlaneRight * WE_flrand(-self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                        (*part).mPosition +=
                            self.mSpawnPlaneUp * WE_flrand(-self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                    }
                    // Otherwise, Just Wrap Around To The Other End Of The Range
                    //-----------------------------------------------------------
                    else {
                        let spawn_range_copy: SVecRange = core::ptr::read(&self.mSpawnRange);
                        self.mRange.Wrap(&mut (*part).mPosition, &spawn_range_copy);
                    }
                    partInRange = true;
                }

                // Process Fade
                //--------------
                {
                    // Start A Fade Out
                    //------------------
                    if partRendering && !partInView {
                        (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEIN);
                        (*part).mFlags.set_bit(CWeatherParticle::FLAG_FADEOUT);
                    }
                    // Switch From Fade Out To Fade In
                    //---------------------------------
                    else if partRendering
                        && partInView
                        && (*part).mFlags.get_bit(CWeatherParticle::FLAG_FADEOUT)
                    {
                        (*part).mFlags.set_bit(CWeatherParticle::FLAG_FADEIN);
                        (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEOUT);
                    }
                    // Start A Fade In
                    //-----------------
                    else if !partRendering && partInView {
                        partRendering = true;
                        (*part).mAlpha = 0.0_f32;
                        (*part).mFlags.set_bit(CWeatherParticle::FLAG_RENDER);
                        (*part).mFlags.set_bit(CWeatherParticle::FLAG_FADEIN);
                        (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEOUT);
                    }

                    // Update Fade
                    //-------------
                    if partRendering {
                        // Update Fade Out
                        //-----------------
                        if (*part).mFlags.get_bit(CWeatherParticle::FLAG_FADEOUT) {
                            (*part).mAlpha -= particleFade;
                            if (*part).mAlpha <= 0.0_f32 {
                                (*part).mAlpha = 0.0_f32;
                                (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEOUT);
                                (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEIN);
                                (*part).mFlags.clear_bit(CWeatherParticle::FLAG_RENDER);
                                partRendering = false;
                            }
                        }
                        // Update Fade In
                        //----------------
                        else if (*part).mFlags.get_bit(CWeatherParticle::FLAG_FADEIN) {
                            (*part).mAlpha += particleFade;
                            if (*part).mAlpha >= self.mColor[3] {
                                (*part).mFlags.clear_bit(CWeatherParticle::FLAG_FADEIN);
                                (*part).mAlpha = self.mColor[3];
                            }
                        }
                    }
                }

                // Keep Track Of The Number Of Particles To Render
                //-------------------------------------------------
                if (*part).mFlags.get_bit(CWeatherParticle::FLAG_RENDER) {
                    self.mParticleCountRender += 1;
                }

                particleNum += 1;
            }
            self.mPopulated = true;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Render -
    ////////////////////////////////////////////////////////////////////////////////////
    fn Render(&mut self) {
        unsafe {
            let mut part: *mut CWeatherParticle = core::ptr::null_mut();
            let mut particleNum: c_int;

            // Set The GL State And Image Binding
            //------------------------------------
            GL_State(if self.mBlendMode == 0 {
                GLS_ALPHA
            } else {
                GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE
            });
            GL_Bind(self.mImage);

            // Enable And Disable Things
            //---------------------------
            /*
            if (mGLModeEnum==GL_POINTS && qglPointParameteriNV)
            {
                qglEnable(GL_POINT_SPRITE_NV);

                qglPointSize(mWidth);
                qglPointParameterfEXT( GL_POINT_SIZE_MIN_EXT, 4.0f );
                qglPointParameterfEXT( GL_POINT_SIZE_MAX_EXT, 2047.0f );

                qglTexEnvi(GL_POINT_SPRITE_NV, GL_COORD_REPLACE_NV, GL_TRUE);
            }
            else
            */
            //FIXME use this extension?
            let attenuation: [f32; 3] = [1.0_f32, 0.0_f32, 0.0004_f32];

            // #ifdef _XBOX
            #[cfg(feature = "xbox")]
            {
                if self.mGLModeEnum == GL_POINTS {
                    pointBegin(self.mParticleCountRender, self.mWidth);
                }
            }
            // #else
            #[cfg(not(feature = "xbox"))]
            {
                if self.mGLModeEnum == GL_POINTS && !qglPointParameterfEXT.is_null() {
                    //fixme use custom parameters but gotta make sure it expects them on same scale first
                    qglPointSize(10.0_f64);
                    qglPointParameterfEXT(GL_POINT_SIZE_MIN_EXT, 1.0_f32);
                    qglPointParameterfEXT(GL_POINT_SIZE_MAX_EXT, 4.0_f32);
                    qglPointParameterfvEXT(
                        GL_DISTANCE_ATTENUATION_EXT,
                        attenuation.as_ptr() as *mut f32,
                    );
                }
            }
            // #endif
            if self.mGLModeEnum != GL_POINTS {
                qglEnable(GL_TEXTURE_2D);
                //qglDisable(GL_CULL_FACE);
                //naughty, you are making the assumption that culling is on when you get here. -rww
                GL_Cull(CT_TWO_SIDED);

                qglTexParameterf(
                    GL_TEXTURE_2D,
                    GL_TEXTURE_MIN_FILTER,
                    if self.mFilterMode == 0 { GL_LINEAR } else { GL_NEAREST },
                );
                qglTexParameterf(
                    GL_TEXTURE_2D,
                    GL_TEXTURE_MAG_FILTER,
                    if self.mFilterMode == 0 { GL_LINEAR } else { GL_NEAREST },
                );

                // Setup Matrix Mode And Translation
                //-----------------------------------
                qglMatrixMode(GL_MODELVIEW);
                qglPushMatrix();

                // #ifdef _XBOX
                #[cfg(feature = "xbox")]
                qglBeginEXT(
                    self.mGLModeEnum,
                    self.mParticleCountRender * self.mVertexCount,
                    self.mParticleCountRender,
                    0,
                    self.mParticleCountRender * self.mVertexCount,
                    0,
                );
                // #endif
            }

            // Begin
            //-------
            // #ifndef _XBOX
            #[cfg(not(feature = "xbox"))]
            qglBegin(self.mGLModeEnum);
            // #endif

            particleNum = 0;
            while particleNum < self.mParticleCount {
                part = &mut *self.mParticles.add(particleNum as usize);
                if !(*part).mFlags.get_bit(CWeatherParticle::FLAG_RENDER) {
                    particleNum += 1;
                    continue;
                }

                // Blend Mode Zero -> Apply Alpha Just To Alpha Channel
                //------------------------------------------------------
                if self.mBlendMode == 0 {
                    qglColor4f(
                        self.mColor[0],
                        self.mColor[1],
                        self.mColor[2],
                        (*part).mAlpha,
                    );
                }
                // Otherwise Apply Alpha To All Channels
                //---------------------------------------
                else {
                    qglColor4f(
                        self.mColor[0] * (*part).mAlpha,
                        self.mColor[1] * (*part).mAlpha,
                        self.mColor[2] * (*part).mAlpha,
                        self.mColor[3] * (*part).mAlpha,
                    );
                }

                // Render A Point
                //----------------
                if self.mGLModeEnum == GL_POINTS {
                    qglVertex3fv((*part).mPosition.v.as_ptr() as *const f32);
                }
                // Render A Triangle
                //-------------------
                else if self.mVertexCount == 3 {
                    qglTexCoord2f(1.0_f32, 0.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0],
                        (*part).mPosition[1],
                        (*part).mPosition[2],
                    );

                    qglTexCoord2f(0.0_f32, 1.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] + self.mCameraLeft[0],
                        (*part).mPosition[1] + self.mCameraLeft[1],
                        (*part).mPosition[2] + self.mCameraLeft[2],
                    );

                    qglTexCoord2f(0.0_f32, 0.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] + self.mCameraLeftPlusUp[0],
                        (*part).mPosition[1] + self.mCameraLeftPlusUp[1],
                        (*part).mPosition[2] + self.mCameraLeftPlusUp[2],
                    );
                }
                // Render A Quad
                //---------------
                else {
                    // Left bottom.
                    qglTexCoord2f(0.0_f32, 0.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] - self.mCameraLeftMinusUp[0],
                        (*part).mPosition[1] - self.mCameraLeftMinusUp[1],
                        (*part).mPosition[2] - self.mCameraLeftMinusUp[2],
                    );

                    // Right bottom.
                    qglTexCoord2f(1.0_f32, 0.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] - self.mCameraLeftPlusUp[0],
                        (*part).mPosition[1] - self.mCameraLeftPlusUp[1],
                        (*part).mPosition[2] - self.mCameraLeftPlusUp[2],
                    );

                    // Right top.
                    qglTexCoord2f(1.0_f32, 1.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] + self.mCameraLeftMinusUp[0],
                        (*part).mPosition[1] + self.mCameraLeftMinusUp[1],
                        (*part).mPosition[2] + self.mCameraLeftMinusUp[2],
                    );

                    // Left top.
                    qglTexCoord2f(0.0_f32, 1.0_f32);
                    qglVertex3f(
                        (*part).mPosition[0] + self.mCameraLeftPlusUp[0],
                        (*part).mPosition[1] + self.mCameraLeftPlusUp[1],
                        (*part).mPosition[2] + self.mCameraLeftPlusUp[2],
                    );
                }

                particleNum += 1;
            }
            qglEnd();

            if self.mGLModeEnum == GL_POINTS {
                // #ifdef _XBOX
                #[cfg(feature = "xbox")]
                pointEnd();
                // #else
                //qglDisable(GL_POINT_SPRITE_NV);
                //qglTexEnvi(GL_POINT_SPRITE_NV, GL_COORD_REPLACE_NV, GL_FALSE);
                // #endif
            } else {
                //qglEnable(GL_CULL_FACE);
                //you don't need to do this when you are properly setting cull state.
                qglPopMatrix();
            }

            mParticlesRendered += self.mParticleCountRender;
        }
    }
}

static mut mParticleClouds: vector_vs<CWeatherParticleCloud, MAX_PARTICLE_CLOUDS> =
    unsafe { core::mem::zeroed() };

////////////////////////////////////////////////////////////////////////////////////////
// Init World Effects - Will Iterate Over All Particle Clouds, Clear Them Out, And Erase
////////////////////////////////////////////////////////////////////////////////////////
pub fn R_InitWorldEffects() {
    unsafe {
        srand(Com_Milliseconds() as c_uint);

        let sz = (*addr_of_mut!(mParticleClouds)).size();
        for i in 0..sz {
            (*addr_of_mut!(mParticleClouds))[i].Reset();
        }
        (*addr_of_mut!(mParticleClouds)).clear();
        (*addr_of_mut!(mWindZones)).clear();
        (*addr_of_mut!(mOutside)).Reset();
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Init World Effects - Will Iterate Over All Particle Clouds, Clear Them Out, And Erase
////////////////////////////////////////////////////////////////////////////////////////
pub fn R_ShutdownWorldEffects() {
    R_InitWorldEffects();
}

////////////////////////////////////////////////////////////////////////////////////////
// RB_RenderWorldEffects - If any particle clouds exist, this will update and render them
////////////////////////////////////////////////////////////////////////////////////////
pub fn RB_RenderWorldEffects() {
    unsafe {
        if tr.world.is_null()
            || (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0
            || (backEnd.refdef.rdflags & RDF_SKYBOXPORTAL) != 0
            || (*addr_of_mut!(mParticleClouds)).size() == 0
        {
            //  no world rendering or no world or no particle clouds
            return;
        }

        SetViewportAndScissor();
        qglMatrixMode(GL_MODELVIEW);
        qglLoadMatrixf(backEnd.viewParms.world.modelMatrix.as_ptr());

        // Calculate Elapsed Time For Scale Purposes
        //-------------------------------------------
        mMillisecondsElapsed = backEnd.refdef.frametime as f32;
        if mMillisecondsElapsed < 1.0_f32 {
            mMillisecondsElapsed = 1.0_f32;
        }
        if mMillisecondsElapsed > 1000.0_f32 {
            mMillisecondsElapsed = 1000.0_f32;
        }
        mSecondsElapsed = mMillisecondsElapsed / 1000.0_f32;

        // Make Sure We Are Always Outside Cached
        //----------------------------------------
        if !(*addr_of_mut!(mOutside)).Initialized() {
            (*addr_of_mut!(mOutside)).Cache();
        } else {
            // Update All Wind Zones
            //-----------------------
            if !mFrozen {
                mGlobalWindVelocity.Clear();
                let wz_sz = (*addr_of_mut!(mWindZones)).size();
                for wz in 0..wz_sz {
                    (*addr_of_mut!(mWindZones))[wz].Update();
                    if (*addr_of_mut!(mWindZones))[wz].mGlobal {
                        mGlobalWindVelocity += (*addr_of_mut!(mWindZones))[wz].mCurrentVelocity;
                    }
                }
                mGlobalWindDirection = mGlobalWindVelocity;
                mGlobalWindSpeed = VectorNormalize(mGlobalWindDirection.v.as_mut_ptr());
            }

            // Update All Particle Clouds
            //----------------------------
            mParticlesRendered = 0;
            let pc_sz = (*addr_of_mut!(mParticleClouds)).size();
            for i in 0..pc_sz {
                (*addr_of_mut!(mParticleClouds))[i].Update();
                (*addr_of_mut!(mParticleClouds))[i].Render();
            }
            if false {
                Com_Printf(
                    b"Weather: %d Particles Rendered\n\0".as_ptr(),
                    mParticlesRendered,
                );
            }
        }
    }
}

pub fn R_WorldEffect_f() {
    unsafe {
        if Cvar_VariableIntegerValue(b"sv_cheats\0".as_ptr() as *const c_char) != 0 {
            let mut temp: [c_char; 2048] = [0; 2048];
            Cmd_ArgsBuffer(temp.as_mut_ptr(), core::mem::size_of::<[c_char; 2048]>() as c_int);
            R_WorldEffectCommand(temp.as_ptr());
        }
    }
}

pub fn R_WorldEffectCommand(command: *const c_char) {
    unsafe {
        if command.is_null() {
            return;
        }

        // Mutable local copy so COM_ParseExt/ParseVector can advance the pointer
        let mut command: *const c_char = command;

        let mut token: *const c_char; //, *origCommand;

        token = COM_ParseExt(&mut command as *mut *const c_char, qfalse);

        if token.is_null() {
            return;
        }

        //Die - clean up the whole weather system -rww
        if strcmpi(token, b"die\0".as_ptr() as *const c_char) == 0 {
            R_ShutdownWorldEffects();
            return;
        }

        // Clear - Removes All Particle Clouds And Wind Zones
        //----------------------------------------------------
        else if strcmpi(token, b"clear\0".as_ptr() as *const c_char) == 0 {
            let p_sz = (*addr_of_mut!(mParticleClouds)).size();
            for p in 0..p_sz {
                (*addr_of_mut!(mParticleClouds))[p].Reset();
            }
            (*addr_of_mut!(mParticleClouds)).clear();
            (*addr_of_mut!(mWindZones)).clear();
        }

        // Freeze / UnFreeze - Stops All Particle Motion Updates
        //--------------------------------------------------------
        else if strcmpi(token, b"freeze\0".as_ptr() as *const c_char) == 0 {
            mFrozen = !mFrozen;
        }

        // Add a zone
        //---------------
        else if strcmpi(token, b"zone\0".as_ptr() as *const c_char) == 0 {
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            if ParseVector(
                &mut command as *mut *const c_char,
                3,
                mins.as_mut_ptr(),
            ) != 0
                && ParseVector(
                    &mut command as *mut *const c_char,
                    3,
                    maxs.as_mut_ptr(),
                ) != 0
            {
                (*addr_of_mut!(mOutside)).AddWeatherZone(mins.as_mut_ptr(), maxs.as_mut_ptr());
            }
        }

        // Basic Wind
        //------------
        else if strcmpi(token, b"wind\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mWindZones)).full() {
                return;
            }
            let nWind: &mut CWindZone = (*addr_of_mut!(mWindZones)).push_back();
            nWind.Initialize();
        }

        // Constant Wind
        //---------------
        else if strcmpi(token, b"constantwind\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mWindZones)).full() {
                return;
            }
            let nWind: &mut CWindZone = (*addr_of_mut!(mWindZones)).push_back();
            nWind.Initialize();
            if ParseVector(
                &mut command as *mut *const c_char,
                3,
                nWind.mCurrentVelocity.v.as_mut_ptr(),
            ) == 0
            {
                nWind.mCurrentVelocity.Clear();
                nWind.mCurrentVelocity[1] = 800.0_f32;
            }
            nWind.mTargetVelocityTimeRemaining = -1;
        }

        // Gusting Wind
        //--------------
        else if strcmpi(token, b"gustingwind\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mWindZones)).full() {
                return;
            }
            let nWind: &mut CWindZone = (*addr_of_mut!(mWindZones)).push_back();
            nWind.Initialize();
            nWind.mRVelocity.mMins       = -3000.0_f32;
            nWind.mRVelocity.mMins[2]    = -100.0_f32;
            nWind.mRVelocity.mMaxs       =  3000.0_f32;
            nWind.mRVelocity.mMaxs[2]    =  100.0_f32;

            nWind.mMaxDeltaVelocityPerUpdate = 10.0_f32;

            nWind.mRDuration.mMin = 1000;
            nWind.mRDuration.mMax = 3000;

            nWind.mChanceOfDeadTime = 0.5_f32;
            nWind.mRDeadTime.mMin   = 2000;
            nWind.mRDeadTime.mMax   = 4000;
        }

        // Create A Rain Storm
        //---------------------
        else if strcmpi(token, b"lightrain\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(500, b"gfx/world/rain.jpg\0".as_ptr() as *const c_char, 3);
            nCloud.mHeight    = 80.0_f32;
            nCloud.mWidth     = 1.2_f32;
            nCloud.mGravity   = 2000.0_f32;
            nCloud.mFilterMode = 1;
            nCloud.mBlendMode  = 1;
            nCloud.mFade      = 100.0_f32;
            nCloud.mColor     = 0.5_f32;
            nCloud.mOrientWithVelocity = true;
            nCloud.mWaterParticles = true;
        }

        // Create A Rain Storm
        //---------------------
        else if strcmpi(token, b"rain\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(1000, b"gfx/world/rain.jpg\0".as_ptr() as *const c_char, 3);
            nCloud.mHeight    = 80.0_f32;
            nCloud.mWidth     = 1.2_f32;
            nCloud.mGravity   = 2000.0_f32;
            nCloud.mFilterMode = 1;
            nCloud.mBlendMode  = 1;
            nCloud.mFade      = 100.0_f32;
            nCloud.mColor     = 0.5_f32;
            nCloud.mOrientWithVelocity = true;
            nCloud.mWaterParticles = true;
        }

        // Create A Rain Storm
        //---------------------
        else if strcmpi(token, b"acidrain\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(1000, b"gfx/world/rain.jpg\0".as_ptr() as *const c_char, 3);
            nCloud.mHeight    = 80.0_f32;
            nCloud.mWidth     = 2.0_f32;
            nCloud.mGravity   = 2000.0_f32;
            nCloud.mFilterMode = 1;
            nCloud.mBlendMode  = 1;
            nCloud.mFade      = 100.0_f32;

            nCloud.mColor[0] = 0.34_f32;
            nCloud.mColor[1] = 0.70_f32;
            nCloud.mColor[2] = 0.34_f32;
            nCloud.mColor[3] = 0.70_f32;

            nCloud.mOrientWithVelocity = true;
            nCloud.mWaterParticles = true;

            (*addr_of_mut!(mOutside)).mOutsidePain = 0.1_f32;
        }

        // Create A Rain Storm
        //---------------------
        else if strcmpi(token, b"heavyrain\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(1000, b"gfx/world/rain.jpg\0".as_ptr() as *const c_char, 3);
            nCloud.mHeight    = 80.0_f32;
            nCloud.mWidth     = 1.2_f32;
            nCloud.mGravity   = 2800.0_f32;
            nCloud.mFilterMode = 1;
            nCloud.mBlendMode  = 1;
            nCloud.mFade      = 15.0_f32;
            nCloud.mColor     = 0.5_f32;
            nCloud.mOrientWithVelocity = true;
            nCloud.mWaterParticles = true;
        }

        // Create A Snow Storm
        //---------------------
        else if strcmpi(token, b"snow\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            // #ifdef _XBOX
            #[cfg(feature = "xbox")]
            nCloud.Initialize(1000, b"gfx/effects/snowflake1.bmp\0".as_ptr() as *const c_char, 1);
            // #else
            #[cfg(not(feature = "xbox"))]
            nCloud.Initialize(1000, b"gfx/effects/snowflake1.bmp\0".as_ptr() as *const c_char, 4);
            // #endif
            nCloud.mBlendMode          = 1;
            nCloud.mRotationChangeNext = 0;
            nCloud.mColor              = 0.75_f32;
            nCloud.mWaterParticles = true;
            // #ifdef _XBOX
            #[cfg(feature = "xbox")]
            { nCloud.mWidth = 0.05_f32; }
            // #endif
        }

        // Create A Some stuff
        //---------------------
        else if strcmpi(token, b"spacedust\0".as_ptr() as *const c_char) == 0 {
            let count: c_int;
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            token = COM_ParseExt(&mut command as *mut *const c_char, qfalse);
            count = atoi(token);

            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(count, b"gfx/effects/snowpuff1.tga\0".as_ptr() as *const c_char, 4);
            nCloud.mHeight  = 1.2_f32;
            nCloud.mWidth   = 1.2_f32;
            nCloud.mGravity = 0.0_f32;
            nCloud.mBlendMode          = 1;
            nCloud.mRotationChangeNext = 0;
            nCloud.mColor              = 0.75_f32;
            nCloud.mWaterParticles = true;
            nCloud.mMass.mMax = 30.0_f32;
            nCloud.mMass.mMin = 10.0_f32;
            nCloud.mSpawnRange.mMins[0] = -1500.0_f32;
            nCloud.mSpawnRange.mMins[1] = -1500.0_f32;
            nCloud.mSpawnRange.mMins[2] = -1500.0_f32;
            nCloud.mSpawnRange.mMaxs[0] =  1500.0_f32;
            nCloud.mSpawnRange.mMaxs[1] =  1500.0_f32;
            nCloud.mSpawnRange.mMaxs[2] =  1500.0_f32;
        }

        // Create A Sand Storm
        //---------------------
        else if strcmpi(token, b"sand\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(400, b"gfx/effects/alpha_smoke2b.tga\0".as_ptr() as *const c_char, 4);

            nCloud.mGravity  = 0.0_f32;
            nCloud.mWidth    = 70.0_f32;
            nCloud.mHeight   = 70.0_f32;
            nCloud.mColor[0] = 0.9_f32;
            nCloud.mColor[1] = 0.6_f32;
            nCloud.mColor[2] = 0.0_f32;
            nCloud.mColor[3] = 0.5_f32;
            nCloud.mFade     = 5.0_f32;
            nCloud.mMass.mMax = 30.0_f32;
            nCloud.mMass.mMin = 10.0_f32;
            nCloud.mSpawnRange.mMins[2] = -150.0_f32;
            nCloud.mSpawnRange.mMaxs[2] =  150.0_f32;

            nCloud.mRotationChangeNext = 0;
        }

        // Create Blowing Clouds Of Fog
        //------------------------------
        else if strcmpi(token, b"fog\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(60, b"gfx/effects/alpha_smoke2b.tga\0".as_ptr() as *const c_char, 4);
            nCloud.mBlendMode = 1;
            nCloud.mGravity   = 0.0_f32;
            nCloud.mWidth     = 70.0_f32;
            nCloud.mHeight    = 70.0_f32;
            nCloud.mColor     = 0.2_f32;
            nCloud.mFade      = 5.0_f32;
            nCloud.mMass.mMax = 30.0_f32;
            nCloud.mMass.mMin = 10.0_f32;
            nCloud.mSpawnRange.mMins[2] = -150.0_f32;
            nCloud.mSpawnRange.mMaxs[2] =  150.0_f32;

            nCloud.mRotationChangeNext = 0;
        }

        // Create Heavy Rain Particle Cloud
        //-----------------------------------
        else if strcmpi(token, b"heavyrainfog\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(70, b"gfx/effects/alpha_smoke2b.tga\0".as_ptr() as *const c_char, 4);
            nCloud.mBlendMode = 1;
            nCloud.mGravity   = 0.0_f32;
            nCloud.mWidth     = 100.0_f32;
            nCloud.mHeight    = 100.0_f32;
            nCloud.mColor     = 0.3_f32;
            nCloud.mFade      = 1.0_f32;
            nCloud.mMass.mMax = 10.0_f32;
            nCloud.mMass.mMin = 5.0_f32;

            nCloud.mSpawnRange.mMins = -(nCloud.mSpawnPlaneDistance * 1.25_f32);
            nCloud.mSpawnRange.mMaxs =  (nCloud.mSpawnPlaneDistance * 1.25_f32);
            nCloud.mSpawnRange.mMins[2] = -150.0_f32;
            nCloud.mSpawnRange.mMaxs[2] =  150.0_f32;

            nCloud.mRotationChangeNext = 0;
        }

        // Create Blowing Clouds Of Fog
        //------------------------------
        else if strcmpi(token, b"light_fog\0".as_ptr() as *const c_char) == 0 {
            if (*addr_of_mut!(mParticleClouds)).full() {
                return;
            }
            let nCloud: &mut CWeatherParticleCloud = (*addr_of_mut!(mParticleClouds)).push_back();
            nCloud.Initialize(40, b"gfx/effects/alpha_smoke2b.tga\0".as_ptr() as *const c_char, 4);
            nCloud.mBlendMode = 1;
            nCloud.mGravity   = 0.0_f32;
            nCloud.mWidth     = 100.0_f32;
            nCloud.mHeight    = 100.0_f32;
            nCloud.mColor[0]  = 0.19_f32;
            nCloud.mColor[1]  = 0.6_f32;
            nCloud.mColor[2]  = 0.7_f32;
            nCloud.mColor[3]  = 0.12_f32;
            nCloud.mFade      = 0.10_f32;
            nCloud.mMass.mMax = 30.0_f32;
            nCloud.mMass.mMin = 10.0_f32;
            nCloud.mSpawnRange.mMins[2] = -150.0_f32;
            nCloud.mSpawnRange.mMaxs[2] =  150.0_f32;

            nCloud.mRotationChangeNext = 0;
        }

        else if strcmpi(token, b"outsideshake\0".as_ptr() as *const c_char) == 0 {
            (*addr_of_mut!(mOutside)).mOutsideShake = !(*addr_of_mut!(mOutside)).mOutsideShake;
        } else if strcmpi(token, b"outsidepain\0".as_ptr() as *const c_char) == 0 {
            (*addr_of_mut!(mOutside)).mOutsidePain =
                if (*addr_of_mut!(mOutside)).mOutsidePain != 0.0 { 0.0 } else { 1.0 };
        } else {
            Com_Printf(b"Weather Effect: Please enter a valid command.\n\0".as_ptr());
            Com_Printf(b"\tclear\n\0".as_ptr());
            Com_Printf(b"\tfreeze\n\0".as_ptr());
            Com_Printf(b"\tzone (mins) (maxs)\n\0".as_ptr());
            Com_Printf(b"\twind\n\0".as_ptr());
            Com_Printf(b"\tconstantwind (velocity)\n\0".as_ptr());
            Com_Printf(b"\tgustingwind\n\0".as_ptr());
            Com_Printf(b"\twindzone (mins) (maxs) (velocity)\n\0".as_ptr());
            Com_Printf(b"\tlightrain\n\0".as_ptr());
            Com_Printf(b"\train\n\0".as_ptr());
            Com_Printf(b"\tacidrain\n\0".as_ptr());
            Com_Printf(b"\theavyrain\n\0".as_ptr());
            Com_Printf(b"\tsnow\n\0".as_ptr());
            Com_Printf(b"\tspacedust\n\0".as_ptr());
            Com_Printf(b"\tsand\n\0".as_ptr());
            Com_Printf(b"\tfog\n\0".as_ptr());
            Com_Printf(b"\theavyrainfog\n\0".as_ptr());
            Com_Printf(b"\tlight_fog\n\0".as_ptr());
            Com_Printf(b"\toutsideshake\n\0".as_ptr());
            Com_Printf(b"\toutsidepain\n\0".as_ptr());
        }
    }
}

pub fn R_GetChanceOfSaberFizz() -> f32 {
    unsafe {
        let mut chance: f32 = 0.0_f32;
        let mut numWater: c_int = 0;
        let sz = (*addr_of_mut!(mParticleClouds)).size();
        for i in 0..sz {
            if (*addr_of_mut!(mParticleClouds))[i].mWaterParticles {
                chance += (*addr_of_mut!(mParticleClouds))[i].mGravity / 20000.0_f32;
                numWater += 1;
            }
        }
        if numWater != 0 {
            return chance / numWater as f32;
        }
        0.0_f32
    }
}

pub fn R_IsRaining() -> bool {
    unsafe { !(*addr_of_mut!(mParticleClouds)).empty() }
}

pub fn R_IsPuffing() -> bool {
    //Eh? Don't want surfacesprites to know this?
    false
}
