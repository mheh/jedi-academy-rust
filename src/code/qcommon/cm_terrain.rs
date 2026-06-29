use core::ffi::{c_int, c_char, c_void};

// extern C function declarations from renderer
extern "C" {
    fn R_LoadDataImage(name: *const c_char, pic: *mut *mut u8, width: *mut c_int, height: *mut c_int);
    fn R_InvertImage(data: *mut u8, width: c_int, height: c_int, depth: c_int);
    fn R_Resample(source: *mut u8, swidth: c_int, sheight: c_int, dest: *mut u8, dwidth: c_int, dheight: c_int, components: c_int);
}

// extern C declarations from common/qcommon
extern "C" {
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_ParseTextFile(name: *const c_char, parse: *mut CGenericParser2) -> c_int;
    fn Com_ParseTextFileDestroy(parse: CGenericParser2);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Clamp(min: c_int, max: c_int, value: c_int) -> c_int;
    fn Z_Malloc(size: c_int, tag: c_int, zero: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
}

// extern C declarations
extern "C" {
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn atol(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn strtoul(s: *const c_char, endp: *mut *mut c_char, base: c_int) -> c_ulong;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn abs(x: c_int) -> c_int;
    fn sqrtf(x: f32) -> f32;
    fn fabsf(x: f32) -> f32;
    fn floorf(x: f32) -> f32;
    fn ceilf(x: f32) -> f32;
    fn sinf(x: f32) -> f32;
    fn sqrt(x: f64) -> f64;
}

// Macro constants
const BRUSH_SIDES_PER_TERXEL: usize = 5;

#[derive(Copy, Clone)]
#[repr(C)]
struct vec3_t([f32; 3]);

#[derive(Copy, Clone)]
#[repr(C)]
struct ivec3_t([c_int; 3]);

#[derive(Copy, Clone)]
#[repr(C)]
struct vec3pair_t([vec3_t; 2]);

#[derive(Copy, Clone)]
#[repr(C)]
struct cplane_t {
    normal: vec3_t,
    dist: f32,
    plane_type: u8,
    signbits: u8,
    pad: [u8; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
struct cbrushside_t {
    plane: *mut cplane_t,
    surfaceFlags: c_int,
    planeNum: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct cbrush_t {
    contents: c_int,
    numsides: c_int,
    sides: *mut cbrushside_t,
    bounds: [vec3_t; 2],
}

// Forward declarations
#[repr(C)]
struct CGPGroup;

#[repr(C)]
struct CGenericParser2;

#[repr(C)]
struct CCMShader;

#[repr(C)]
struct CCMHeightDetails;

#[repr(C)]
struct CArea;

#[repr(C)]
struct CRandomTerrain;

#[repr(C)]
struct traceWork_s;

#[repr(C)]
struct trace_t {
    fraction: f32,
    // ... other fields
}

// Local stubs for unported dependencies (placeholders)
#[repr(C)]
struct CCMHeightDetails;

fn VectorSubtract(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out.0[0] = a.0[0] - b.0[0];
    out.0[1] = a.0[1] - b.0[1];
    out.0[2] = a.0[2] - b.0[2];
}

fn VectorCopy(src: vec3_t, dst: &mut vec3_t) {
    dst.0 = src.0;
}

fn VectorSet(out: &mut vec3_t, x: f32, y: f32, z: f32) {
    out.0[0] = x;
    out.0[1] = y;
    out.0[2] = z;
}

fn VectorNormalize(v: &mut vec3_t) {
    let mut length = v.0[0] * v.0[0] + v.0[1] * v.0[1] + v.0[2] * v.0[2];
    length = unsafe { sqrtf(length) };
    if length > 0.0 {
        v.0[0] /= length;
        v.0[1] /= length;
        v.0[2] /= length;
    }
}

fn DotProduct(a: vec3_t, b: vec3_t) -> f32 {
    a.0[0] * b.0[0] + a.0[1] * b.0[1] + a.0[2] * b.0[2]
}

fn CrossProduct(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out.0[0] = a.0[1] * b.0[2] - a.0[2] * b.0[1];
    out.0[1] = a.0[2] * b.0[0] - a.0[0] * b.0[2];
    out.0[2] = a.0[0] * b.0[1] - a.0[1] * b.0[0];
}

fn VectorLength(v: vec3_t) -> f32 {
    let len_sq = v.0[0] * v.0[0] + v.0[1] * v.0[1] + v.0[2] * v.0[2];
    unsafe { sqrtf(len_sq) }
}

fn VectorDec(v: &mut vec3_t) {
    v.0[0] -= 1.0;
    v.0[1] -= 1.0;
    v.0[2] -= 1.0;
}

fn VectorInc(v: &mut vec3_t) {
    v.0[0] += 1.0;
    v.0[1] += 1.0;
    v.0[2] += 1.0;
}

fn VectorScaleVectorAdd(a: vec3_t, b: ivec3_t, scale: vec3_t, out: &mut vec3_t) {
    out.0[0] = a.0[0] + (b.0[0] as f32) * scale.0[0];
    out.0[1] = a.0[1] + (b.0[1] as f32) * scale.0[1];
    out.0[2] = a.0[2] + (b.0[2] as f32) * scale.0[2];
}

fn VectorInverseScaleVector(v: vec3_t, scale: vec3_t, out: &mut vec3_t) {
    if scale.0[0] != 0.0 {
        out.0[0] = v.0[0] / scale.0[0];
    } else {
        out.0[0] = 0.0;
    }
    if scale.0[1] != 0.0 {
        out.0[1] = v.0[1] / scale.0[1];
    } else {
        out.0[1] = 0.0;
    }
    if scale.0[2] != 0.0 {
        out.0[2] = v.0[2] / scale.0[2];
    } else {
        out.0[2] = 0.0;
    }
}

fn PlaneTypeForNormal(normal: vec3_t) -> u8 {
    // FIXME: Placeholder - actual implementation depends on q_math.h
    0
}

fn SetPlaneSignbits(plane: &mut cplane_t) {
    // FIXME: Placeholder - actual implementation depends on q_math.h
}

fn Round(x: f32) -> c_int {
    unsafe { floorf(x + 0.5) as c_int }
}

fn Distance(a: vec3_t, b: vec3_t) -> f32 {
    let dx = a.0[0] - b.0[0];
    let dy = a.0[1] - b.0[1];
    let dz = a.0[2] - b.0[2];
    unsafe { sqrtf(dx * dx + dy * dy + dz * dz) }
}

fn minimum(a: c_int, b: c_int) -> c_int {
    if a < b { a } else { b }
}

fn CM_HandlePatchCollision(tw: *mut traceWork_s, trace: &mut trace_t, mins: vec3_t, maxs: vec3_t, patch: *mut CCMPatch, checkcount: c_int) {
    // FIXME: Placeholder
}

fn CM_CalcExtents(start: vec3_t, end: vec3_t, tw: *mut traceWork_s, out: &mut vec3pair_t) {
    // FIXME: Placeholder
}

const TAG_CM_TERRAIN: c_int = 1;
const MAX_QPATH: usize = 256;
const HEIGHT_RESOLUTION: usize = 256;
const MAX_WORLD_COORD: f32 = 128000.0;
const MIN_WORLD_COORD: f32 = -128000.0;
const SURFACE_CLIP_EPSILON: f32 = 0.125;
const ERR_FATAL: c_int = 3;
const M_PI: f64 = 3.14159265358979323846;

#[repr(C)]
pub struct CCMPatch {
    owner: *mut CCMLandScape,
    mWorldCoords: vec3_t,
    mHx: c_int,
    mHy: c_int,
    mHeightMap: *mut u8,
    mBounds: [vec3_t; 2],
    mCornerHeights: [u8; 4],
    mSurfaceFlags: c_int,
    mContentFlags: c_int,
    mNumBrushes: c_int,
    mPatchBrushData: *mut cbrush_t,
}

impl CCMPatch {
    fn new() -> Self {
        CCMPatch {
            owner: std::ptr::null_mut(),
            mWorldCoords: vec3_t([0.0; 3]),
            mHx: 0,
            mHy: 0,
            mHeightMap: std::ptr::null_mut(),
            mBounds: [vec3_t([0.0; 3]); 2],
            mCornerHeights: [0; 4],
            mSurfaceFlags: 0,
            mContentFlags: 0,
            mNumBrushes: 0,
            mPatchBrushData: std::ptr::null_mut(),
        }
    }

    // Initialise a plane from 3 coords
    fn InitPlane(&mut self, side: *mut cbrushside_t, plane: *mut cplane_t, p0: vec3_t, p1: vec3_t, p2: vec3_t) {
        let mut dx = vec3_t([0.0; 3]);
        let mut dy = vec3_t([0.0; 3]);

        VectorSubtract(p1, p0, &mut dx);
        VectorSubtract(p2, p0, &mut dy);

        let mut normal = vec3_t([0.0; 3]);
        CrossProduct(dx, dy, &mut normal);
        VectorNormalize(&mut normal);

        unsafe {
            (*plane).normal = normal;
            (*plane).dist = DotProduct(p0, normal);
            (*plane).plane_type = PlaneTypeForNormal(normal);
            SetPlaneSignbits(plane);
            (*side).plane = plane;
        }
    }

    // Create the planes required for collision detection
    // 2 brushes per terxel - each brush has 5 sides and 5 planes
    fn GetAdjacentBrushY(&self, x: c_int, y: c_int) -> *mut c_void {
        let owner = unsafe { &*self.owner };
        let yo1 = y % owner.GetTerxels();
        let yo2 = (y - 1) % owner.GetTerxels();
        let xo = x % owner.GetTerxels();

        let patch = if yo2 > yo1 {
            owner.GetPatch(x / owner.GetTerxels(), (y - 1) / owner.GetTerxels())
        } else {
            self as *const _ as *mut CCMPatch
        };

        unsafe {
            let brush = (*patch).mPatchBrushData;
            let offset = ((yo2 * owner.GetTerxels() + xo) * 2) as isize;
            (brush.offset(offset + 1) as *mut c_void)
        }
    }

    fn GetAdjacentBrushX(&self, x: c_int, y: c_int) -> *mut c_void {
        let owner = unsafe { &*self.owner };
        let xo1 = x % owner.GetTerxels();
        let xo2 = (x - 1) % owner.GetTerxels();
        let yo = y % owner.GetTerxels();

        let patch = if xo2 > xo1 {
            owner.GetPatch((x - 1) / owner.GetTerxels(), y / owner.GetTerxels())
        } else {
            self as *const _ as *mut CCMPatch
        };

        unsafe {
            let brush = (*patch).mPatchBrushData;
            let offset = ((yo * owner.GetTerxels() + xo2) * 2) as isize;
            let mut b = brush.offset(offset);

            if ((x + y) & 1) == 0 {
                b = b.offset(1);
            }

            (b as *mut c_void)
        }
    }

    fn CreatePatchPlaneData(&mut self) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            let owner = unsafe { &*self.owner };
            let realWidth = owner.GetRealWidth();
            let coords = owner.GetCoords();

            let mut brush = self.mPatchBrushData;
            let mut side = unsafe { (self.mPatchBrushData as *mut u8).add(self.mNumBrushes as usize * std::mem::size_of::<cbrush_t>()) as *mut cbrushside_t };
            let mut plane = unsafe { (side as *mut u8).add((self.mNumBrushes * BRUSH_SIDES_PER_TERXEL as c_int * 2) as usize * std::mem::size_of::<cbrushside_t>()) as *mut cplane_t };

            for y in self.mHy..self.mHy + owner.GetTerxels() {
                for x in self.mHx..self.mHx + owner.GetTerxels() {
                    let mut offsets: [c_int; 4] = [0; 4];

                    if ((x + y) & 1) != 0 {
                        offsets[0] = (y * realWidth) + x;              // TL
                        offsets[1] = (y * realWidth) + x + 1;          // TR
                        offsets[2] = ((y + 1) * realWidth) + x;        // BL
                        offsets[3] = ((y + 1) * realWidth) + x + 1;    // BR
                    } else {
                        offsets[2] = (y * realWidth) + x;              // TL
                        offsets[0] = (y * realWidth) + x + 1;          // TR
                        offsets[3] = ((y + 1) * realWidth) + x;        // BL
                        offsets[1] = ((y + 1) * realWidth) + x + 1;    // BR
                    }

                    let mut localCoords: [vec3_t; 8] = [vec3_t([0.0; 3]); 8];

                    for i in 0..4 {
                        let idx = offsets[i as usize] as usize;
                        if idx < unsafe { coords.len() } {
                            VectorCopy(unsafe { coords[idx] }, &mut localCoords[i as usize]);
                            VectorCopy(unsafe { coords[idx] }, &mut localCoords[(i + 4) as usize]);
                            localCoords[(i + 4) as usize].0[2] = owner.GetMins().0[2];
                        }
                    }

                    // Set the bounds of the terxel
                    VectorSet(&mut unsafe { (*brush).bounds[0] }, MAX_WORLD_COORD, MAX_WORLD_COORD, MAX_WORLD_COORD);
                    VectorSet(&mut unsafe { (*brush).bounds[1] }, MIN_WORLD_COORD, MIN_WORLD_COORD, MIN_WORLD_COORD);

                    for i in 0..8 {
                        for j in 0..3 {
                            if localCoords[i].0[j] < unsafe { (*brush).bounds[0].0[j] } {
                                unsafe { (*brush).bounds[0].0[j] = localCoords[i].0[j]; }
                            }
                            if localCoords[i].0[j] > unsafe { (*brush).bounds[1].0[j] } {
                                unsafe { (*brush).bounds[1].0[j] = localCoords[i].0[j]; }
                            }
                        }
                    }

                    VectorDec(&mut unsafe { (*brush).bounds[0] });
                    VectorInc(&mut unsafe { (*brush).bounds[1] });
                    VectorCopy(unsafe { (*brush).bounds[0] }, &mut unsafe { (*brush.offset(1)).bounds[0] });
                    VectorCopy(unsafe { (*brush).bounds[1] }, &mut unsafe { (*brush.offset(1)).bounds[1] });

                    unsafe {
                        (*brush).contents = self.mContentFlags;
                        (*brush.offset(1)).contents = self.mContentFlags;
                        (*brush).numsides = 5;
                        (*brush).sides = side;
                        (*brush.offset(1)).numsides = 5;
                        (*brush.offset(1)).sides = side.offset(5);
                    }

                    for i in 0..8 {
                        localCoords[i].0[0] = unsafe { floorf(localCoords[i].0[0]) };
                        localCoords[i].0[1] = unsafe { floorf(localCoords[i].0[1]) };
                        localCoords[i].0[2] = unsafe { floorf(localCoords[i].0[2]) };
                    }

                    // Create the planes of the 2 triangles that make up the tops of the brushes
                    self.InitPlane(side, plane, localCoords[0], localCoords[1], localCoords[2]);
                    self.InitPlane(unsafe { side.offset(5) }, unsafe { plane.offset(5) }, localCoords[3], localCoords[2], localCoords[1]);

                    // Create the bottom face of the brushes
                    self.InitPlane(unsafe { side.offset(1) }, unsafe { plane.offset(1) }, localCoords[6], localCoords[5], localCoords[4]);
                    self.InitPlane(unsafe { side.offset(6) }, unsafe { plane.offset(6) }, localCoords[5], localCoords[6], localCoords[7]);

                    // Create the 3 vertical faces
                    self.InitPlane(unsafe { side.offset(2) }, unsafe { plane.offset(2) }, localCoords[0], localCoords[2], localCoords[4]);
                    self.InitPlane(unsafe { side.offset(7) }, unsafe { plane.offset(7) }, localCoords[3], localCoords[1], localCoords[7]);

                    self.InitPlane(unsafe { side.offset(3) }, unsafe { plane.offset(3) }, localCoords[0], localCoords[4], localCoords[1]);
                    self.InitPlane(unsafe { side.offset(8) }, unsafe { plane.offset(8) }, localCoords[3], localCoords[7], localCoords[2]);

                    self.InitPlane(unsafe { side.offset(4) }, unsafe { plane.offset(4) }, localCoords[2], localCoords[1], localCoords[6]);
                    self.InitPlane(unsafe { side.offset(9) }, unsafe { plane.offset(9) }, localCoords[5], localCoords[1], localCoords[6]);

                    // Increment to next terxel
                    brush = unsafe { brush.offset(2) };
                    side = unsafe { side.offset(10) };
                    plane = unsafe { plane.offset(10) };
                }
            }
        }
    }

    fn Init(&mut self, ls: *mut CCMLandScape, heightX: c_int, heightY: c_int, world: vec3_t, hMap: *mut u8, patchBrushData: *mut u8) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            self.owner = ls;
            let owner = unsafe { &*ls };

            VectorCopy(world, &mut self.mWorldCoords);

            self.mHx = heightX;
            self.mHy = heightY;
            self.mHeightMap = unsafe { hMap.add(((heightY * owner.GetRealWidth()) + heightX) as usize) };

            let mut min = 256;
            let mut max = -1;

            for y in (heightY - 1)..(heightY + owner.GetTerxels() + 1) {
                if y >= 0 {
                    for x in (heightX - 1)..(heightX + owner.GetTerxels() + 1) {
                        if x >= 0 {
                            let idx = ((y * owner.GetRealWidth()) + x) as usize;
                            let height = unsafe { *hMap.add(idx) } as c_int;

                            if height > max {
                                max = height;
                            }
                            if height < min {
                                min = height;
                            }
                        }
                    }
                }
            }

            // Mins
            self.mBounds[0].0[0] = world.0[0];
            self.mBounds[0].0[1] = world.0[1];
            self.mBounds[0].0[2] = world.0[2] + ((min as f32) * owner.GetTerxelSize().0[2]);

            // Maxs
            self.mBounds[1].0[0] = world.0[0] + owner.GetPatchSize().0[0];
            self.mBounds[1].0[1] = world.0[1] + owner.GetPatchSize().0[1];
            self.mBounds[1].0[2] = world.0[2] + ((max as f32) * owner.GetTerxelSize().0[2]);

            // Corner heights
            self.mCornerHeights[0] = unsafe { *self.mHeightMap };
            self.mCornerHeights[1] = unsafe { *self.mHeightMap.add(owner.GetTerxels() as usize) };
            self.mCornerHeights[2] = unsafe { *self.mHeightMap.add((owner.GetTerxels() * owner.GetRealWidth()) as usize) };
            self.mCornerHeights[3] = unsafe { *self.mHeightMap.add(((owner.GetTerxels() * owner.GetRealWidth()) + owner.GetTerxels()) as usize) };

            self.mSurfaceFlags = owner.GetSurfaceFlags((min + max) >> 1);
            self.mContentFlags = owner.GetContentFlags((min + max) >> 1);

            self.mPatchBrushData = patchBrushData as *mut cbrush_t;
            self.CreatePatchPlaneData();
        }
    }
}

impl Drop for CCMPatch {
    fn drop(&mut self) {
        // Destructor does nothing for CCMPatch
    }
}

#[repr(C)]
pub struct CCMLandScape {
    mWidth: c_int,
    mHeight: c_int,
    mBlockWidth: c_int,
    mBlockHeight: c_int,
    mTerxels: c_int,
    mSize: vec3_t,
    mBounds: [vec3_t; 2],
    mTerxelSize: vec3_t,
    mPatchSize: vec3_t,
    mPatchScalarSize: f32,
    mHeightMap: *mut u8,
    mFlattenMap: *mut u8,
    mPatchBrushData: *mut u8,
    mPatches: *mut CCMPatch,
    mCoords: *mut vec3_t,
    mHeightDetails: [CCMHeightDetails; HEIGHT_RESOLUTION],
    mBaseWaterHeight: c_int,
    mWaterHeight: f32,
    mWaterContents: c_int,
    mWaterSurfaceFlags: c_int,
    mRandomTerrain: *mut CRandomTerrain,
    mRefCount: c_int,
    mHasPhysics: bool,
    mAreas: Vec<*mut CArea>,
    mAreasIt: std::vec::IntoIter<*mut CArea>,
    holdrand: c_int,
}

impl CCMLandScape {
    fn GetRealArea(&self) -> c_int {
        self.GetRealWidth() * self.GetRealHeight()
    }

    fn GetRealWidth(&self) -> c_int {
        self.mWidth
    }

    fn GetRealHeight(&self) -> c_int {
        self.mHeight
    }

    fn GetWidth(&self) -> c_int {
        self.mWidth
    }

    fn GetHeight(&self) -> c_int {
        self.mHeight
    }

    fn GetBlockCount(&self) -> c_int {
        self.mBlockWidth * self.mBlockHeight
    }

    fn GetTerxels(&self) -> c_int {
        self.mTerxels
    }

    fn GetCoords(&self) -> &[vec3_t] {
        unsafe { std::slice::from_raw_parts(self.mCoords, (self.GetRealArea()) as usize) }
    }

    fn GetTerxelSize(&self) -> vec3_t {
        self.mTerxelSize
    }

    fn GetPatchSize(&self) -> vec3_t {
        self.mPatchSize
    }

    fn GetMins(&self) -> vec3_t {
        self.mBounds[0]
    }

    fn GetMaxs(&self) -> vec3_t {
        self.mBounds[1]
    }

    fn GetPatchWidth(&self) -> c_int {
        self.mBlockWidth
    }

    fn GetPatchHeight(&self) -> c_int {
        self.mBlockHeight
    }

    fn GetSurfaceFlags(&self, height: c_int) -> c_int {
        0 // FIXME: Placeholder
    }

    fn GetContentFlags(&self, height: c_int) -> c_int {
        0 // FIXME: Placeholder
    }

    fn SetTerrainId(&mut self, id: c_int) {
        // FIXME: Placeholder
    }

    fn GetPatch(&self, x: c_int, y: c_int) -> *mut CCMPatch {
        unsafe { self.mPatches.offset(((y * self.mBlockWidth) + x) as isize) }
    }

    fn SetShaders(&mut self, height: c_int, shader: *mut CCMShader) {
        // FIXME: Placeholder - requires implementation of mHeightDetails
    }

    fn LoadTerrainDef(&mut self, td: *const c_char) {
        let mut terrainDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];

        unsafe {
            let terrainDefStr = Info_ValueForKey(td, b"terrainDef\0".as_ptr() as *const c_char);
            Com_sprintf(terrainDef.as_mut_ptr(), MAX_QPATH as c_int, b"ext_data/RMG/%s.terrain\0".as_ptr() as *const c_char, terrainDefStr);
            Com_DPrintf(b"CM_Terrain: Loading and parsing terrainDef %s.....\n\0".as_ptr() as *const c_char, terrainDefStr);

            // FIXME: Placeholder - requires proper implementation of parser
        }
    }

    fn UpdatePatches(&mut self) {
        self.CalcRealCoords();

        let numBrushesPerPatch = self.mTerxels * self.mTerxels * 2;
        let size = (numBrushesPerPatch * std::mem::size_of::<cbrush_t>() as c_int)
                 + (numBrushesPerPatch * BRUSH_SIDES_PER_TERXEL as c_int * 2 * (std::mem::size_of::<cbrushside_t>() as c_int + std::mem::size_of::<cplane_t>() as c_int));

        let mut patch = self.mPatches;
        let mut iy = 0;
        let mut y = 0;

        while y < self.mHeight {
            let mut ix = 0;
            let mut x = 0;

            while x < self.mWidth {
                let mut world = vec3_t([0.0; 3]);
                VectorSet(&mut world,
                    self.mBounds[0].0[0] + ((x as f32) * self.mTerxelSize.0[0]),
                    self.mBounds[0].0[1] + ((y as f32) * self.mTerxelSize.0[1]),
                    self.mBounds[0].0[2]);

                unsafe {
                    (*patch).Init(
                        self as *mut CCMLandScape,
                        x,
                        y,
                        world,
                        self.mHeightMap,
                        self.mPatchBrushData.add((size as usize) * ((ix + (iy * self.mBlockWidth)) as usize))
                    );
                    patch = patch.offset(1);
                }

                x += self.mTerxels;
                ix += 1;
            }

            y += self.mTerxels;
            iy += 1;
        }

        unsafe { Z_Free(self.mCoords as *mut c_void); }
        self.mCoords = std::ptr::null_mut();
    }

    fn CalcRealCoords(&mut self) {
        let size = (std::mem::size_of::<vec3_t>() * (self.GetRealWidth() * self.GetRealHeight()) as usize) as c_int;
        self.mCoords = unsafe { Z_Malloc(size, TAG_CM_TERRAIN, 0) as *mut vec3_t };

        for y in 0..self.GetRealHeight() {
            for x in 0..self.GetRealWidth() {
                let offset = ((y * self.GetRealWidth()) + x) as usize;
                let mut icoords = ivec3_t([x, y, 0]);

                icoords.0[2] = unsafe { *self.mHeightMap.add(offset) } as c_int;

                let mut out = vec3_t([0.0; 3]);
                VectorScaleVectorAdd(self.GetMins(), icoords, self.GetTerxelSize(), &mut out);

                unsafe {
                    *self.mCoords.add(offset) = out;
                }
            }
        }
    }

    fn PatchCollide(&mut self, tw: *mut traceWork_s, trace: &mut trace_t, start: vec3_t, end: vec3_t, checkcount: c_int) {
        let mut tBounds: vec3pair_t = [vec3_t([0.0; 3]); 2];

        CM_CalcExtents(start, end, tw, &mut tBounds);

        if true {
            let mut slope: f32 = 0.0;
            let mut offset: f32 = 0.0;
            let mut startPatchLoc: f32 = 0.0;
            let mut endPatchLoc: f32 = 0.0;
            let mut startPos: f32 = 0.0;
            let mut endPos: f32 = 0.0;
            let mut patchDirection: f32 = 1.0;
            let mut checkDirection: f32 = 1.0;
            let mut countPatches: c_int = 0;
            let mut count: c_int = 0;
            let mut patch: *mut CCMPatch = std::ptr::null_mut();
            let fraction = trace.fraction;

            if unsafe { fabsf(end.0[0] - start.0[0]) } >= unsafe { fabsf(fabsf(end.0[1] - start.0[1])) } {
                // x travels more than y
                if end.0[0] - start.0[0] != 0.0 {
                    slope = (end.0[1] - start.0[1]) / (end.0[0] - start.0[0]);
                } else {
                    slope = 0.0;
                }
                offset = start.0[1] - (start.0[0] * slope);

                startPatchLoc = unsafe { floorf((start.0[0] - self.mBounds[0].0[0]) / self.mPatchSize.0[0]) };
                endPatchLoc = unsafe { floorf((end.0[0] - self.mBounds[0].0[0]) / self.mPatchSize.0[0]) };

                if startPatchLoc <= endPatchLoc {
                    endPatchLoc += 1.0;
                    startPatchLoc -= 1.0;
                    countPatches = (endPatchLoc - startPatchLoc + 1.0) as c_int;
                } else {
                    endPatchLoc -= 1.0;
                    startPatchLoc += 1.0;
                    patchDirection = -1.0;
                    countPatches = (startPatchLoc - endPatchLoc + 1.0) as c_int;
                }
                if slope < 0.0 {
                    checkDirection = -1.0;
                }

                startPos = ((startPatchLoc * self.mPatchSize.0[0] + self.mBounds[0].0[0]) * slope) + offset;
                startPos = unsafe { floorf((startPos - self.mBounds[0].0[1] + unsafe { (*tw).size[0][1] }) / self.mPatchSize.0[1]) };

                // FIXME: Placeholder - full implementation of patch collision traversal
            }
        }
    }

    fn WaterCollide(&self, begin: vec3_t, end: vec3_t, mut fraction: f32) -> f32 {
        if (begin.0[2] > self.mWaterHeight) && (end.0[2] > self.mWaterHeight) {
            return fraction;
        }
        if (begin.0[2] < self.mWaterHeight) && (end.0[2] < self.mWaterHeight) {
            return fraction;
        }
        if begin.0[2] < self.mWaterHeight - SURFACE_CLIP_EPSILON {
            fraction = ((self.mWaterHeight - SURFACE_CLIP_EPSILON) - begin.0[2]) / (end.0[2] - begin.0[2]);
            return fraction;
        }
        if begin.0[2] > self.mWaterHeight + SURFACE_CLIP_EPSILON {
            fraction = (begin.0[2] - (self.mWaterHeight + SURFACE_CLIP_EPSILON)) / (begin.0[2] - end.0[2]);
        }
        fraction
    }

    fn GetTerxelLocalCoords(&self, x: c_int, y: c_int, localCoords: &mut [vec3_t; 8]) {
        let realWidth = self.GetRealWidth();
        let coords = self.GetCoords();
        let mut offsets: [c_int; 4] = [0; 4];

        if ((x + y) & 1) != 0 {
            offsets[0] = (y * realWidth) + x;              // TL
            offsets[1] = (y * realWidth) + x + 1;          // TR
            offsets[2] = ((y + 1) * realWidth) + x;        // BL
            offsets[3] = ((y + 1) * realWidth) + x + 1;    // BR
        } else {
            offsets[2] = (y * realWidth) + x;              // TL
            offsets[0] = (y * realWidth) + x + 1;          // TR
            offsets[3] = ((y + 1) * realWidth) + x;        // BL
            offsets[1] = ((y + 1) * realWidth) + x + 1;    // BR
        }

        for i in 0..4 {
            let idx = offsets[i as usize] as usize;
            if idx < coords.len() {
                VectorCopy(coords[idx], &mut localCoords[i as usize]);
                VectorCopy(coords[idx], &mut localCoords[(i + 4) as usize]);
                localCoords[(i + 4) as usize].0[2] = self.GetMins().0[2];
            }
        }
    }

    fn GetWorldHeight(&self, origin: &mut vec3_t, bounds: vec3pair_t, aboveGround: bool) -> f32 {
        let mut work = vec3_t([0.0; 3]);
        let mut minx: c_int = 0;
        let mut maxx: c_int = 0;
        let mut miny: c_int = 0;
        let mut maxy: c_int = 0;
        let mut TL: c_int = 0;
        let mut TR: c_int = 0;
        let mut BL: c_int = 0;
        let mut BR: c_int = 0;
        let mut final_height: c_int = 0;

        VectorSubtract(origin.clone(), self.mBounds[0], &mut work);
        VectorInverseScaleVector(work, self.mTerxelSize, &mut work);

        minx = unsafe { Com_Clamp(0, self.GetWidth(), unsafe { floorf(work.0[0]) } as c_int) };
        maxx = unsafe { Com_Clamp(0, self.GetWidth(), unsafe { ceilf(work.0[0]) } as c_int) };
        miny = unsafe { Com_Clamp(0, self.GetHeight(), unsafe { floorf(work.0[1]) } as c_int) };
        maxy = unsafe { Com_Clamp(0, self.GetHeight(), unsafe { ceilf(work.0[1]) } as c_int) };

        let idx_TL = ((miny * self.GetRealWidth()) + minx) as usize;
        let idx_TR = ((miny * self.GetRealWidth()) + maxx) as usize;
        let idx_BL = ((maxy * self.GetRealWidth()) + minx) as usize;
        let idx_BR = ((maxy * self.GetRealWidth()) + maxx) as usize;

        TL = unsafe { *self.mHeightMap.add(idx_TL) } as c_int;
        TR = unsafe { *self.mHeightMap.add(idx_TR) } as c_int;
        BL = unsafe { *self.mHeightMap.add(idx_BL) } as c_int;
        BR = unsafe { *self.mHeightMap.add(idx_BR) } as c_int;

        if aboveGround {
            let tx = (work.0[0] - minx as f32) / ((maxx - minx) as f32);
            let ty = (work.0[1] - miny as f32) / ((maxy - miny) as f32);
            let h1 = ((TR as f32 - TL as f32) * tx) + TL as f32;
            let h2 = ((BR as f32 - BL as f32) * tx) + BL as f32;
            final_height = (((h2 - h1) * ty) + h1) as c_int;
        } else {
            let min1 = minimum(TL, TR);
            let min2 = minimum(BL, BR);
            final_height = minimum(min1, min2);
        }

        origin.0[2] = ((final_height as f32) * self.mTerxelSize.0[2]) + self.mBounds[0].0[2];

        if maxx == minx {
            maxx = unsafe { Com_Clamp(0, self.GetWidth(), minx + 1) };
        }
        if maxy == miny {
            maxy = unsafe { Com_Clamp(0, self.GetHeight(), miny + 1) };
        }
        let idx_BR2 = ((maxy * self.GetRealWidth()) + maxx) as usize;
        BR = unsafe { *self.mHeightMap.add(idx_BR2) } as c_int;

        (unsafe { fabsf((BR - TL) as f32) } * self.mTerxelSize.0[2]) / self.mTerxelSize.0[0]
    }

    fn TerrainPatchIterate(&self, IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void), userdata: *mut c_void) {
        let mut patch = self.mPatches;
        for _i in 0..self.GetBlockCount() {
            IterateFunc(patch, userdata);
            patch = unsafe { patch.offset(1) };
        }
    }

    fn SaveArea(&mut self, area: *mut CArea) {
        self.mAreas.push(area);
    }

    fn CarveLine(&mut self, start: vec3_t, end: vec3_t, depth: c_int, width: c_int) {
        // FIXME: Placeholder for CarveLine implementation
    }

    fn CarveBezierCurve(&mut self, numCtlPoints: c_int, ctlPoints: *mut vec3_t, steps: c_int, depth: c_int, size: c_int) {
        // FIXME: Placeholder for CarveBezierCurve implementation
    }

    fn FlattenArea(&mut self, area: *mut CArea, height: c_int, save: bool, forceHeight: bool, smooth: bool) {
        // FIXME: Placeholder for FlattenArea implementation
    }

    fn FractionBelowLevel(&self, area: *mut CArea, height: c_int) -> f32 {
        // FIXME: Placeholder for FractionBelowLevel implementation
        0.0
    }

    fn AreaCollision(&mut self, area: *mut CArea, areaTypes: *mut c_int, areaTypeCount: c_int) -> bool {
        // FIXME: Placeholder for AreaCollision implementation
        false
    }

    fn GetFirstArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            return std::ptr::null_mut();
        }
        self.mAreasIt = self.mAreas.clone().into_iter();
        if let Some(&area) = self.mAreasIt.next() {
            area
        } else {
            std::ptr::null_mut()
        }
    }

    fn GetFirstObjectiveArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            return std::ptr::null_mut();
        }
        self.mAreasIt = self.mAreas.clone().into_iter();

        while let Some(&area) = self.mAreasIt.next() {
            // FIXME: Placeholder - requires AT_OBJECTIVE constant
            return area;
        }
        std::ptr::null_mut()
    }

    fn GetPlayerArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            return std::ptr::null_mut();
        }
        self.mAreasIt = self.mAreas.clone().into_iter();

        while let Some(&area) = self.mAreasIt.next() {
            // FIXME: Placeholder - requires AT_PLAYER constant
            return area;
        }
        std::ptr::null_mut()
    }

    fn GetNextArea(&mut self) -> *mut CArea {
        if let Some(&area) = self.mAreasIt.next() {
            return area;
        }
        std::ptr::null_mut()
    }

    fn GetNextObjectiveArea(&mut self) -> *mut CArea {
        while let Some(&area) = self.mAreasIt.next() {
            // FIXME: Placeholder - requires AT_OBJECTIVE constant
            return area;
        }
        std::ptr::null_mut()
    }

    fn rand_seed(&mut self, seed: c_int) {
        self.holdrand = seed;
        unsafe {
            Com_Printf(b"rand_seed = %d\n\0".as_ptr() as *const c_char, self.holdrand);
        }
    }

    fn flrand(&mut self, min: f32, max: f32) -> f32 {
        let result: f32;

        assert!((max - min) < 32768.0);

        self.holdrand = (self.holdrand.wrapping_mul(214013)) + 2531011;
        result = (((self.holdrand >> 17) as f32) * (max - min)) / 32768.0 + min;

        result
    }

    fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        let result: c_int;

        assert!((max - min) < 32768);

        let max = max + 1;
        self.holdrand = (self.holdrand.wrapping_mul(214013)) + 2531011;
        result = (self.holdrand >> 17) & 0x7FFF;
        ((result * (max - min)) >> 15) + min
    }
}

impl Drop for CCMLandScape {
    fn drop(&mut self) {
        if !self.mHeightMap.is_null() {
            unsafe { Z_Free(self.mHeightMap as *mut c_void); }
            self.mHeightMap = std::ptr::null_mut();
        }
        if !self.mFlattenMap.is_null() {
            unsafe { Z_Free(self.mFlattenMap as *mut c_void); }
            self.mFlattenMap = std::ptr::null_mut();
        }
        if !self.mPatchBrushData.is_null() {
            unsafe { Z_Free(self.mPatchBrushData as *mut c_void); }
            self.mPatchBrushData = std::ptr::null_mut();
        }
        if !self.mPatches.is_null() {
            unsafe { Z_Free(self.mPatches as *mut c_void); }
            self.mPatches = std::ptr::null_mut();
        }
        if !self.mRandomTerrain.is_null() {
            unsafe {
                let _ = Box::from_raw(self.mRandomTerrain);
            }
        }

        for area in self.mAreas.iter() {
            if !area.is_null() {
                unsafe {
                    let _ = Box::from_raw(*area);
                }
            }
        }

        self.mAreas.clear();
    }
}

// C API functions
#[no_mangle]
pub extern "C" fn CM_InitTerrain(configstring: *const c_char, terrainId: c_int, server: bool) -> *mut CCMLandScape {
    let mut ls = Box::new(CCMLandScape {
        mWidth: 0,
        mHeight: 0,
        mBlockWidth: 0,
        mBlockHeight: 0,
        mTerxels: 0,
        mSize: vec3_t([0.0; 3]),
        mBounds: [vec3_t([0.0; 3]); 2],
        mTerxelSize: vec3_t([0.0; 3]),
        mPatchSize: vec3_t([0.0; 3]),
        mPatchScalarSize: 0.0,
        mHeightMap: std::ptr::null_mut(),
        mFlattenMap: std::ptr::null_mut(),
        mPatchBrushData: std::ptr::null_mut(),
        mPatches: std::ptr::null_mut(),
        mCoords: std::ptr::null_mut(),
        mHeightDetails: [Default::default(); HEIGHT_RESOLUTION],
        mBaseWaterHeight: 0,
        mWaterHeight: 0.0,
        mWaterContents: 0,
        mWaterSurfaceFlags: 0,
        mRandomTerrain: std::ptr::null_mut(),
        mRefCount: 1,
        mHasPhysics: false,
        mAreas: Vec::new(),
        mAreasIt: Vec::new().into_iter(),
        holdrand: 0x89abcdef,
    });

    ls.SetTerrainId(terrainId);
    Box::into_raw(ls)
}

#[no_mangle]
pub extern "C" fn CM_TerrainPatchIterate(landscape: *const CCMLandScape, IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void), userdata: *mut c_void) {
    unsafe {
        (*landscape as *mut CCMLandScape).as_mut().map(|l| {
            l.TerrainPatchIterate(IterateFunc, userdata);
        });
    }
}

#[no_mangle]
pub extern "C" fn CM_GetWorldHeight(landscape: *const CCMLandScape, origin: *mut vec3_t, bounds: vec3pair_t, aboveGround: bool) -> f32 {
    unsafe {
        (*landscape).GetWorldHeight(&mut *origin, bounds, aboveGround)
    }
}

#[no_mangle]
pub extern "C" fn CM_FlattenArea(landscape: *mut CCMLandScape, area: *mut CArea, height: c_int, save: bool, forceHeight: bool, smooth: bool) {
    unsafe {
        (*landscape).FlattenArea(area, height, save, forceHeight, smooth);
    }
}

#[no_mangle]
pub extern "C" fn CM_CarveBezierCurve(landscape: *mut CCMLandScape, numCtls: c_int, ctls: *mut vec3_t, steps: c_int, depth: c_int, size: c_int) {
    unsafe {
        (*landscape).CarveBezierCurve(numCtls, ctls, steps, depth, size);
    }
}

#[no_mangle]
pub extern "C" fn CM_SaveArea(landscape: *mut CCMLandScape, area: *mut CArea) {
    unsafe {
        (*landscape).SaveArea(area);
    }
}

#[no_mangle]
pub extern "C" fn CM_FractionBelowLevel(landscape: *mut CCMLandScape, area: *mut CArea, height: c_int) -> f32 {
    unsafe {
        (*landscape).FractionBelowLevel(area, height)
    }
}

#[no_mangle]
pub extern "C" fn CM_AreaCollision(landscape: *mut CCMLandScape, area: *mut CArea, areaTypes: *mut c_int, areaTypeCount: c_int) -> bool {
    unsafe {
        (*landscape).AreaCollision(area, areaTypes, areaTypeCount)
    }
}

#[no_mangle]
pub extern "C" fn CM_GetFirstArea(landscape: *mut CCMLandScape) -> *mut CArea {
    unsafe {
        (*landscape).GetFirstArea()
    }
}

#[no_mangle]
pub extern "C" fn CM_GetFirstObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea {
    unsafe {
        (*landscape).GetFirstObjectiveArea()
    }
}

#[no_mangle]
pub extern "C" fn CM_GetPlayerArea(landscape: *mut CCMLandScape) -> *mut CArea {
    unsafe {
        (*landscape).GetPlayerArea()
    }
}

#[no_mangle]
pub extern "C" fn CM_GetNextArea(landscape: *mut CCMLandScape) -> *mut CArea {
    unsafe {
        (*landscape).GetNextArea()
    }
}

#[no_mangle]
pub extern "C" fn CM_GetNextObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea {
    unsafe {
        (*landscape).GetNextObjectiveArea()
    }
}

#[no_mangle]
pub extern "C" fn CreateRandomTerrain(config: *const c_char, landscape: *mut CCMLandScape, heightmap: *mut u8, width: c_int, height: c_int) -> *mut CRandomTerrain {
    #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
    {
        let mut ptr: *mut c_char = std::ptr::null_mut();
        let seed: c_ulong;

        unsafe {
            seed = strtoul(Info_ValueForKey(config, b"seed\0".as_ptr() as *const c_char), &mut ptr, 10);
            (*landscape).rand_seed(seed as c_int);
        }

        let random_terrain = Box::new(unsafe { std::mem::zeroed::<CRandomTerrain>() });
        Box::into_raw(random_terrain)
    }

    #[cfg(feature = "PRE_RELEASE_DEMO")]
    {
        std::ptr::null_mut()
    }
}

// Helper callback functions
extern "C" fn CM_ForceHeight(work: *mut u8, lerp: f32, user: *mut c_int) {
    unsafe {
        *work = unsafe { Com_Clamp(0, 255, *user) } as u8;
    }
}

extern "C" fn CM_GetAverage(work: *mut u8, lerp: f32, user: *mut c_int) {
    unsafe {
        *user.offset(0) += *work as c_int;
        *user.offset(1) += 1;
    }
}

extern "C" fn CM_Smooth(work: *mut u8, lerp: f32, user: *mut c_int) {
    let smooth = unsafe { sinf((M_PI / 2.0 * 3.0 + (1.0 - lerp as f64) * (M_PI / 2.0)) as f32) } + 1.0;
    unsafe {
        *work = (*work as i32 + ((*user - *work as c_int) as f32 * smooth) as i32) as u8;
    }
}

extern "C" fn CM_MakeAverage(work: *mut u8, lerp: f32, user: *mut c_int) {
    let height = *work as c_int;
    let mut diff = unsafe { *user } - height;
    if unsafe { abs(diff) } > 3 {
        diff >>= 2;
    }
    let new_height = height + diff;
    unsafe {
        *work = Com_Clamp(0, 255, new_height) as u8;
    }
}

#[no_mangle]
pub extern "C" fn CM_CircularIterate(data: *mut u8, width: c_int, height: c_int, xo: c_int, yo: c_int, insideRadius: c_int, outsideRadius: c_int, user: *mut c_int, callback: extern "C" fn(*mut u8, f32, *mut c_int)) {
    for y in -outsideRadius..=outsideRadius {
        if y + yo >= 0 && y + yo < height {
            let offset = unsafe { sqrtf(((outsideRadius * outsideRadius) - (y * y)) as f32) as c_int };
            for x in -offset..=offset {
                if x + xo >= 0 && x + xo < width {
                    let radius = unsafe { sqrtf(((x * x + y * y) as f32)) };

                    if radius >= insideRadius as f32 {
                        let work = unsafe { data.add((x + xo) as usize + ((y + yo) as usize * width as usize)) };
                        let lerp = (radius - insideRadius as f32) / (outsideRadius as f32 - insideRadius as f32);
                        callback(work, lerp, user);
                    }
                }
            }
        }
    }
}

extern "C" fn CM_BelowLevel(data: *mut u8, lerp: f32, info: *mut c_int) {
    unsafe {
        *info.offset(1) += 1;
        if *data < *info.offset(2) as u8 {
            *info.offset(0) += 1;
        }
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
use core::fmt::Debug;

impl Debug for CCMHeightDetails {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CCMHeightDetails {{ }}")
    }
}

impl Default for CCMHeightDetails {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// end
