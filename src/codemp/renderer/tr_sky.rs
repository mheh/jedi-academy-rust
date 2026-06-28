//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// tr_sky.c
// #include "tr_local.h"

use core::ffi::{c_int, c_float};
use core::ptr::{addr_of, addr_of_mut};

const SKY_SUBDIVISIONS: c_int = 8;
const HALF_SKY_SUBDIVISIONS: c_int = SKY_SUBDIVISIONS / 2;

// Type stubs - these would normally be defined in tr_local.h
type vec3_t = [f32; 3];
type qboolean = bool;

const qtrue: qboolean = true;
const qfalse: qboolean = false;

const SIDE_FRONT: c_int = 0;
const SIDE_BACK: c_int = 1;
const SIDE_ON: c_int = 2;

const ON_EPSILON: f32 = 0.1f32;
const MAX_CLIP_VERTS: c_int = 64;

const SHADER_MAX_VERTEXES: c_int = 3000;

const RDF_SKYBOXPORTAL: c_int = 0x0020;

// Stub enum for error codes
enum ErrorCode {
    ERR_DROP = 1,
}

// Stub struct definitions for external types
#[repr(C)]
pub struct image_s {
    // Stub: image structure
}

#[repr(C)]
pub struct shader_t {
    // Stub: shader structure
    sky: *mut SkyParms,
    numUnfoggedPasses: c_int,
}

#[repr(C)]
pub struct SkyParms {
    outerbox: [*mut image_s; 6],
    cloudHeight: f32,
}

#[repr(C)]
pub struct shaderCommands_t {
    // Stub: shader commands structure
    numIndexes: c_int,
    indexes: [c_int; 4096],
    xyz: [[f32; 3]; 4096],
    numVertexes: c_int,
    shader: *mut shader_t,
}

#[repr(C)]
pub struct vec3_origin_t {
    // Stub for vec3_origin constant
}

#[repr(C)]
pub struct ViewParms {
    ori: Orientation,
    zFar: f32,
    world: WorldMatrix,
    rdflags: c_int,
}

#[repr(C)]
pub struct Orientation {
    origin: [f32; 3],
}

#[repr(C)]
pub struct WorldMatrix {
    modelMatrix: [f32; 16],
}

#[repr(C)]
pub struct BackEnd {
    viewParms: ViewParms,
    refdef: RefDef,
    skyRenderedThisView: bool,
}

#[repr(C)]
pub struct RefDef {
    rdflags: c_int,
}

#[repr(C)]
pub struct Tessellation {
    numIndexes: c_int,
    indexes: [c_int; 4096],
    xyz: [[f32; 3]; 4096],
    numVertexes: c_int,
    texCoords: [[[f32; 2]; 2]; 4096],
    vertexColors: [[c_int; 4]; 4096],
    shader: *mut shader_t,
    fogNum: c_int,
}

#[repr(C)]
pub struct RendererState {
    sunDirection: [f32; 3],
    identityLight: f32,
    sunShader: *mut shader_t,
    defaultImage: *mut image_s,
}

#[repr(C)]
pub struct Cvar {
    integer: c_int,
}

// External globals and functions - stubs
extern "C" {
    static mut backEnd: BackEnd;
    static mut tess: Tessellation;
    static mut tr: RendererState;
    static mut g_bRenderGlowingObjects: bool;
    static mut skyboxportal: bool;

    static mut r_drawSun: Cvar;
    static mut r_fastsky: Cvar;
    static mut r_showsky: Cvar;

    static vec3_origin: vec3_t;

    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorScale(a: *const f32, scale: f32, out: *mut f32);
    fn VectorNormalize(v: *mut f32) -> f32;
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn CrossProduct(a: *const f32, b: *const f32, out: *mut f32);
    fn PerpendicularVector(dst: *mut f32, src: *const f32);

    fn Com_Error(level: ErrorCode, fmt: *const u8, ...);
    fn Com_Memset(mem: *mut c_void, c: c_int, len: usize) -> *mut c_void;

    fn GL_Bind(image: *mut image_s);
    fn GL_State(state: c_int);

    fn qglBegin(mode: c_int);
    fn qglEnd();
    fn qglBeginEXT(mode: c_int, verts: c_int, a: c_int, b: c_int, c: c_int, d: c_int);
    fn qglTexCoord2fv(v: *const f32);
    fn qglVertex3fv(v: *const f32);

    fn qglLoadMatrixf(m: *const f32);
    fn qglTranslatef(x: f32, y: f32, z: f32);
    fn qglColor3f(r: f32, g: f32, b: f32);
    fn qglPushMatrix();
    fn qglPopMatrix();
    fn qglDepthRange(zNear: f32, zFar: f32);

    fn RB_BeginSurface(shader: *mut shader_t, fogNum: c_int);
    fn RB_EndSurface();
    fn RB_StageIteratorGeneric();

    fn Q_acos(c: f32) -> f32;
    fn myftol(f: f32) -> c_int;
}

// Stubs for missing types
use core::ffi::c_void;

static mut s_cloudTexCoords: [[[[[f32; 2]; 9]; 9]; 6] = [[[[[0.0; 2]; 9]; 9]; 6];

static mut s_cloudTexP: [[[[f32; 9]; 9]; 6] = [[[0.0; 9]; 9]; 6];

/*
===================================================================================

POLYGON TO BOX SIDE PROJECTION

===================================================================================
*/

static sky_clip: [[f32; 3]; 6] = [
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
];

static mut sky_mins: [[f32; 6]; 2] = [[0.0; 6]; 2];
static mut sky_maxs: [[f32; 6]; 2] = [[0.0; 6]; 2];
static mut sky_min: f32 = 0.0;
static mut sky_max: f32 = 0.0;

/*
================
AddSkyPolygon
================
*/
unsafe fn AddSkyPolygon(nump: c_int, vecs: *mut f32) {
    let mut i: c_int;
    let mut j: c_int;
    let mut v: vec3_t = [0.0; 3];
    let mut av: vec3_t = [0.0; 3];
    let mut s: f32;
    let mut t: f32;
    let mut dv: f32;
    let mut axis: c_int;
    let mut vp: *mut f32;

    // s = [0]/[2], t = [1]/[2]
    static vec_to_st: [[c_int; 3]; 6] = [
        [-2, 3, 1],
        [2, 3, -1],

        [1, 3, 2],
        [-1, 3, -2],

        [-2, -1, 3],
        [-2, 1, -3],

        //	{-1,2,3},
        //	{1,2,-3}
    ];

    // decide which face it maps to
    VectorCopy(addr_of!(vec3_origin) as *const f32, addr_of_mut!(v) as *mut f32);

    i = 0;
    vp = vecs;
    while i < nump {
        VectorAdd(vp, addr_of!(v) as *const f32, addr_of_mut!(v) as *mut f32);
        i += 1;
        vp = vp.add(3);
    }

    av[0] = (v[0]).abs();
    av[1] = (v[1]).abs();
    av[2] = (v[2]).abs();

    if av[0] > av[1] && av[0] > av[2] {
        if v[0] < 0.0 {
            axis = 1;
        } else {
            axis = 0;
        }
    } else if av[1] > av[2] && av[1] > av[0] {
        if v[1] < 0.0 {
            axis = 3;
        } else {
            axis = 2;
        }
    } else {
        if v[2] < 0.0 {
            axis = 5;
        } else {
            axis = 4;
        }
    }

    // project new texture coords
    i = 0;
    while i < nump {
        j = vec_to_st[axis as usize][2];
        if j > 0 {
            dv = *vecs.add((j - 1) as usize);
        } else {
            dv = -(*vecs.add((-j - 1) as usize));
        }
        if dv < 0.001 {
            i += 1;
            vecs = vecs.add(3);
            continue; // don't divide by zero
        }
        j = vec_to_st[axis as usize][0];
        if j < 0 {
            s = -(*vecs.add((-j - 1) as usize)) / dv;
        } else {
            s = (*vecs.add((j - 1) as usize)) / dv;
        }
        j = vec_to_st[axis as usize][1];
        if j < 0 {
            t = -(*vecs.add((-j - 1) as usize)) / dv;
        } else {
            t = (*vecs.add((j - 1) as usize)) / dv;
        }

        if s < sky_mins[0][axis as usize] {
            sky_mins[0][axis as usize] = s;
        }
        if t < sky_mins[1][axis as usize] {
            sky_mins[1][axis as usize] = t;
        }
        if s > sky_maxs[0][axis as usize] {
            sky_maxs[0][axis as usize] = s;
        }
        if t > sky_maxs[1][axis as usize] {
            sky_maxs[1][axis as usize] = t;
        }

        i += 1;
        vecs = vecs.add(3);
    }
}

/*
================
ClipSkyPolygon
================
*/
unsafe fn ClipSkyPolygon(nump: c_int, vecs: *mut f32, stage: c_int) {
    let mut norm: *mut f32;
    let mut v: *mut f32;
    let mut front: qboolean;
    let mut back: qboolean;
    let mut d: f32;
    let mut e: f32;
    let mut dists: [f32; MAX_CLIP_VERTS as usize] = [0.0; MAX_CLIP_VERTS as usize];
    let mut sides: [c_int; MAX_CLIP_VERTS as usize] = [0; MAX_CLIP_VERTS as usize];
    let mut newv: [[[f32; 3]; MAX_CLIP_VERTS as usize]; 2] = [[[0.0; 3]; MAX_CLIP_VERTS as usize]; 2];
    let mut newc: [c_int; 2] = [0, 0];
    let mut i: c_int;
    let mut j: c_int;

    if nump > MAX_CLIP_VERTS - 2 {
        Com_Error(ErrorCode::ERR_DROP, b"ClipSkyPolygon: MAX_CLIP_VERTS\0".as_ptr());
    }
    if stage == 6 {
        // fully clipped, so draw it
        AddSkyPolygon(nump, vecs);
        return;
    }

    front = qfalse;
    back = qfalse;
    norm = addr_of_mut!(sky_clip[stage as usize][0]) as *mut f32;

    i = 0;
    v = vecs;
    while i < nump {
        d = DotProduct(v, norm);
        if d > ON_EPSILON {
            front = qtrue;
            sides[i as usize] = SIDE_FRONT;
        } else if d < -ON_EPSILON {
            back = qtrue;
            sides[i as usize] = SIDE_BACK;
        } else {
            sides[i as usize] = SIDE_ON;
        }
        dists[i as usize] = d;

        i += 1;
        v = v.add(3);
    }

    if !front || !back {
        // not clipped
        ClipSkyPolygon(nump, vecs, stage + 1);
        return;
    }

    // clip it
    sides[i as usize] = sides[0];
    dists[i as usize] = dists[0];
    VectorCopy(vecs, vecs.add((i * 3) as usize));
    newc[0] = 0;
    newc[1] = 0;

    i = 0;
    v = vecs;
    while i < nump {
        match sides[i as usize] {
            SIDE_FRONT => {
                VectorCopy(v, addr_of_mut!(newv[0][newc[0] as usize]) as *mut f32);
                newc[0] += 1;
            }
            SIDE_BACK => {
                VectorCopy(v, addr_of_mut!(newv[1][newc[1] as usize]) as *mut f32);
                newc[1] += 1;
            }
            SIDE_ON => {
                VectorCopy(v, addr_of_mut!(newv[0][newc[0] as usize]) as *mut f32);
                newc[0] += 1;
                VectorCopy(v, addr_of_mut!(newv[1][newc[1] as usize]) as *mut f32);
                newc[1] += 1;
            }
            _ => {}
        }

        if sides[i as usize] == SIDE_ON || sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
            i += 1;
            v = v.add(3);
            continue;
        }

        d = dists[i as usize] / (dists[i as usize] - dists[(i + 1) as usize]);
        j = 0;
        while j < 3 {
            e = *v.add(j as usize) + d * (*v.add((j + 3) as usize) - *v.add(j as usize));
            newv[0][newc[0] as usize][j as usize] = e;
            newv[1][newc[1] as usize][j as usize] = e;
            j += 1;
        }
        newc[0] += 1;
        newc[1] += 1;

        i += 1;
        v = v.add(3);
    }

    // continue
    ClipSkyPolygon(newc[0], addr_of_mut!(newv[0][0][0]) as *mut f32, stage + 1);
    ClipSkyPolygon(newc[1], addr_of_mut!(newv[1][0][0]) as *mut f32, stage + 1);
}

/*
==============
ClearSkyBox
==============
*/
unsafe fn ClearSkyBox() {
    let mut i: c_int;

    i = 0;
    while i < 6 {
        sky_mins[0][i as usize] = 9999.0;
        sky_mins[1][i as usize] = 9999.0;
        sky_maxs[0][i as usize] = -9999.0;
        sky_maxs[1][i as usize] = -9999.0;
        i += 1;
    }
}

/*
================
RB_ClipSkyPolygons
================
*/
pub unsafe fn RB_ClipSkyPolygons(input: *mut shaderCommands_t) {
    let mut p: [[f32; 3]; 5] = [[0.0; 3]; 5]; // need one extra point for clipping
    let mut i: c_int;
    let mut j: c_int;

    ClearSkyBox();

    i = 0;
    while i < (*input).numIndexes {
        j = 0;
        while j < 3 {
            VectorSubtract(
                addr_of!((*input).xyz[(*input).indexes[(i + j) as usize] as usize]) as *const f32,
                addr_of!((*addr_of!(backEnd).viewParms.ori.origin)) as *const f32,
                addr_of_mut!(p[j as usize]) as *mut f32,
            );
            j += 1;
        }
        ClipSkyPolygon(3, addr_of_mut!(p[0]) as *mut f32, 0);
        i += 3;
    }
}

/*
===================================================================================

CLOUD VERTEX GENERATION

===================================================================================
*/

/*
** MakeSkyVec
**
** Parms: s, t range from -1 to 1
*/
unsafe fn MakeSkyVec(s: f32, t: f32, axis: c_int, outSt: *mut [f32; 2], outXYZ: *mut [f32; 3]) {
    // 1 = s, 2 = t, 3 = 2048
    static st_to_vec: [[c_int; 3]; 6] = [
        [3, -1, 2],
        [-3, 1, 2],

        [1, 3, 2],
        [-1, -3, 2],

        [-2, -1, 3], // 0 degrees yaw, look straight up
        [2, -1, -3],  // look straight down
    ];

    let mut b: vec3_t = [0.0; 3];
    let mut j: c_int;
    let mut k: c_int;
    let mut boxSize: f32;

    boxSize = (*addr_of!(backEnd).viewParms).zFar / 1.75; // div sqrt(3)
    b[0] = s * boxSize;
    b[1] = t * boxSize;
    b[2] = boxSize;

    j = 0;
    while j < 3 {
        k = st_to_vec[axis as usize][j as usize];
        if k < 0 {
            (*outXYZ)[j as usize] = -b[(-k - 1) as usize];
        } else {
            (*outXYZ)[j as usize] = b[(k - 1) as usize];
        }
        j += 1;
    }

    // avoid bilerp seam
    let mut s = (s + 1.0) * 0.5;
    let mut t = (t + 1.0) * 0.5;
    if s < sky_min {
        s = sky_min;
    } else if s > sky_max {
        s = sky_max;
    }

    if t < sky_min {
        t = sky_min;
    } else if t > sky_max {
        t = sky_max;
    }

    t = 1.0 - t;

    if !outSt.is_null() {
        (*outSt)[0] = s;
        (*outSt)[1] = t;
    }
}

static mut s_skyPoints: [[[f32; 3]; 9]; 9] = [[[0.0; 3]; 9]; 9];
static mut s_skyTexCoords: [[[[f32; 2]; 9]; 9] = [[[0.0; 2]; 9]; 9];

unsafe fn DrawSkySide(image: *mut image_s, mins: *const [c_int; 2], maxs: *const [c_int; 2]) {
    let mut s: c_int;
    let mut t: c_int;

    GL_Bind(image);

    #[cfg(target_env = "xbox")]
    {
        let verts = (((*maxs)[0] + HALF_SKY_SUBDIVISIONS) - ((*mins)[0] + HALF_SKY_SUBDIVISIONS)) * 2 + 2;
    }

    t = (*mins)[1] + HALF_SKY_SUBDIVISIONS;
    while t < (*maxs)[1] + HALF_SKY_SUBDIVISIONS {
        #[cfg(target_env = "xbox")]
        {
            qglBeginEXT(5u32 as c_int, verts, 0, 0, verts, 0); // GL_TRIANGLE_STRIP = 5
        }
        #[cfg(not(target_env = "xbox"))]
        {
            qglBegin(5 as c_int); // GL_TRIANGLE_STRIP
        }

        s = (*mins)[0] + HALF_SKY_SUBDIVISIONS;
        while s <= (*maxs)[0] + HALF_SKY_SUBDIVISIONS {
            qglTexCoord2fv(addr_of!(s_skyTexCoords[t as usize][s as usize][0]) as *const f32);
            qglVertex3fv(addr_of!(s_skyPoints[t as usize][s as usize][0]) as *const f32);

            qglTexCoord2fv(addr_of!(s_skyTexCoords[(t + 1) as usize][s as usize][0]) as *const f32);
            qglVertex3fv(addr_of!(s_skyPoints[(t + 1) as usize][s as usize][0]) as *const f32);

            s += 1;
        }

        qglEnd();
        t += 1;
    }
}

unsafe fn DrawSkyBox(shader: *mut shader_t) {
    let mut i: c_int;

    sky_min = 0.0;
    sky_max = 1.0;

    Com_Memset(
        addr_of_mut!(s_skyTexCoords[0][0][0][0]) as *mut c_void,
        0,
        core::mem::size_of_val(&s_skyTexCoords),
    );

    i = 0;
    while i < 6 {
        let mut sky_mins_subd: [c_int; 2] = [0; 2];
        let mut sky_maxs_subd: [c_int; 2] = [0; 2];
        let mut s: c_int;
        let mut t: c_int;

        sky_mins[0][i as usize] = (sky_mins[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_mins[1][i as usize] = (sky_mins[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[0][i as usize] = (sky_maxs[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[1][i as usize] = (sky_maxs[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;

        if (sky_mins[0][i as usize] >= sky_maxs[0][i as usize]) || (sky_mins[1][i as usize] >= sky_maxs[1][i as usize]) {
            i += 1;
            continue;
        }

        sky_mins_subd[0] = (sky_mins[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_mins_subd[1] = (sky_mins[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_maxs_subd[0] = (sky_maxs[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_maxs_subd[1] = (sky_maxs[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32) as c_int;

        if sky_mins_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_mins_subd[1] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        if sky_maxs_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_maxs_subd[1] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        //
        // iterate through the subdivisions
        //
        t = sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS;
        while t <= sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS {
            s = sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS;
            while s <= sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS {
                MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    addr_of_mut!(s_skyTexCoords[t as usize][s as usize]) as *mut [f32; 2],
                    addr_of_mut!(s_skyPoints[t as usize][s as usize]) as *mut [f32; 3],
                );
                s += 1;
            }
            t += 1;
        }

        DrawSkySide(
            (*(*shader).sky).outerbox[i as usize],
            addr_of!(sky_mins_subd) as *const [c_int; 2],
            addr_of!(sky_maxs_subd) as *const [c_int; 2],
        );

        i += 1;
    }
}

unsafe fn FillCloudySkySide(mins: *const [c_int; 2], maxs: *const [c_int; 2], addIndexes: qboolean) {
    let mut s: c_int;
    let mut t: c_int;
    let vertexStart: c_int = (*addr_of_mut!(tess)).numVertexes;
    let mut tHeight: c_int;
    let mut sWidth: c_int;

    tHeight = (*maxs)[1] - (*mins)[1] + 1;
    sWidth = (*maxs)[0] - (*mins)[0] + 1;

    t = (*mins)[1] + HALF_SKY_SUBDIVISIONS;
    while t <= (*maxs)[1] + HALF_SKY_SUBDIVISIONS {
        s = (*mins)[0] + HALF_SKY_SUBDIVISIONS;
        while s <= (*maxs)[0] + HALF_SKY_SUBDIVISIONS {
            VectorAdd(
                addr_of!(s_skyPoints[t as usize][s as usize][0]) as *const f32,
                addr_of!((*addr_of_mut!(backEnd).viewParms.ori.origin)) as *const f32,
                addr_of_mut!((*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][0]) as *mut f32,
            );
            (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = s_skyTexCoords[t as usize][s as usize][0];
            (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = s_skyTexCoords[t as usize][s as usize][1];

            (*addr_of_mut!(tess)).numVertexes += 1;

            if (*addr_of_mut!(tess)).numVertexes >= SHADER_MAX_VERTEXES {
                Com_Error(ErrorCode::ERR_DROP, b"SHADER_MAX_VERTEXES hit in FillCloudySkySide()\n\0".as_ptr());
            }
            s += 1;
        }
        t += 1;
    }

    // only add indexes for one pass, otherwise it would draw multiple times for each pass
    if addIndexes {
        t = 0;
        while t < tHeight - 1 {
            s = 0;
            while s < sWidth - 1 {
                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + t * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;
                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + (t + 1) * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;
                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + 1 + t * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;

                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + (t + 1) * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;
                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + 1 + (t + 1) * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;
                (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = vertexStart + s + 1 + t * (sWidth);
                (*addr_of_mut!(tess)).numIndexes += 1;

                s += 1;
            }
            t += 1;
        }
    }
}

unsafe fn FillCloudBox(shader: *const shader_t, stage: c_int) {
    let mut i: c_int;

    i = 0;
    while i < 6 {
        let mut sky_mins_subd: [c_int; 2] = [0; 2];
        let mut sky_maxs_subd: [c_int; 2] = [0; 2];
        let mut s: c_int;
        let mut t: c_int;
        let mut MIN_T: f32;

        if true {
            // FIXME? shader->sky->fullClouds )
            MIN_T = -HALF_SKY_SUBDIVISIONS as f32;

            // still don't want to draw the bottom, even if fullClouds
            if i == 5 {
                i += 1;
                continue;
            }
        } else {
            match i {
                0 | 1 | 2 | 3 => {
                    MIN_T = -1.0;
                }
                5 => {
                    // don't draw clouds beneath you
                    i += 1;
                    continue;
                }
                4 | _ => {
                    // top
                    MIN_T = -HALF_SKY_SUBDIVISIONS as f32;
                }
            }
        }

        sky_mins[0][i as usize] = (sky_mins[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_mins[1][i as usize] = (sky_mins[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[0][i as usize] = (sky_maxs[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[1][i as usize] = (sky_maxs[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;

        if (sky_mins[0][i as usize] >= sky_maxs[0][i as usize]) || (sky_mins[1][i as usize] >= sky_maxs[1][i as usize]) {
            i += 1;
            continue;
        }

        sky_mins_subd[0] = myftol(sky_mins[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32);
        sky_mins_subd[1] = myftol(sky_mins[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32);
        sky_maxs_subd[0] = myftol(sky_maxs[0][i as usize] * HALF_SKY_SUBDIVISIONS as f32);
        sky_maxs_subd[1] = myftol(sky_maxs[1][i as usize] * HALF_SKY_SUBDIVISIONS as f32);

        if sky_mins_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_mins_subd[1] < MIN_T as c_int {
            sky_mins_subd[1] = MIN_T as c_int;
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        if sky_maxs_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_maxs_subd[1] < MIN_T as c_int {
            sky_maxs_subd[1] = MIN_T as c_int;
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        //
        // iterate through the subdivisions
        //
        t = sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS;
        while t <= sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS {
            s = sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS;
            while s <= sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS {
                MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    core::ptr::null_mut(),
                    addr_of_mut!(s_skyPoints[t as usize][s as usize]) as *mut [f32; 3],
                );

                s_skyTexCoords[t as usize][s as usize][0] = s_cloudTexCoords[i as usize][t as usize][s as usize][0];
                s_skyTexCoords[t as usize][s as usize][1] = s_cloudTexCoords[i as usize][t as usize][s as usize][1];

                s += 1;
            }
            t += 1;
        }

        // only add indexes for first stage
        FillCloudySkySide(
            addr_of!(sky_mins_subd) as *const [c_int; 2],
            addr_of!(sky_maxs_subd) as *const [c_int; 2],
            stage == 0,
        );

        i += 1;
    }
}

/*
** R_BuildCloudData
*/
pub unsafe fn R_BuildCloudData(input: *mut shaderCommands_t) {
    let mut i: c_int;
    let shader: *mut shader_t;

    shader = (*input).shader;

    assert!(!shader.is_null() && !(*shader).sky.is_null());

    sky_min = 1.0 / 256.0;
    sky_max = 255.0 / 256.0;

    // set up for drawing
    (*addr_of_mut!(tess)).numIndexes = 0;
    (*addr_of_mut!(tess)).numVertexes = 0;

    if (*(*input).shader).sky.is_null() {
        // no sky
    } else if (*(*(*input).shader).sky).cloudHeight != 0.0 {
        i = 0;
        while i < (*(*input).shader).numUnfoggedPasses {
            FillCloudBox((*input).shader, i);
            i += 1;
        }
    }
}

/*
** R_InitSkyTexCoords
** Called when a sky shader is parsed
*/
const SQR: fn(f32) -> f32 = |a| a * a;

pub unsafe fn R_InitSkyTexCoords(heightCloud: f32) {
    let mut i: c_int;
    let mut s: c_int;
    let mut t: c_int;
    let mut radiusWorld: f32 = 4096.0;
    let mut p: f32;
    let mut sRad: f32;
    let mut tRad: f32;
    let mut skyVec: vec3_t = [0.0; 3];
    let mut v: vec3_t = [0.0; 3];

    // init zfar so MakeSkyVec works even though
    // a world hasn't been bounded
    (*addr_of_mut!(backEnd).viewParms).zFar = 1024.0;

    i = 0;
    while i < 6 {
        t = 0;
        while t <= SKY_SUBDIVISIONS {
            s = 0;
            while s <= SKY_SUBDIVISIONS {
                // compute vector from view origin to sky side integral point
                MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    core::ptr::null_mut(),
                    addr_of_mut!(skyVec) as *mut [f32; 3],
                );

                // compute parametric value 'p' that intersects with cloud layer
                p = (1.0 / (2.0 * DotProduct(addr_of!(skyVec) as *const f32, addr_of!(skyVec) as *const f32)))
                    * (-2.0 * skyVec[2] * radiusWorld
                        + 2.0
                            * ((SQR(skyVec[2]) * SQR(radiusWorld)
                                + 2.0 * SQR(skyVec[0]) * radiusWorld * heightCloud
                                + SQR(skyVec[0]) * SQR(heightCloud)
                                + 2.0 * SQR(skyVec[1]) * radiusWorld * heightCloud
                                + SQR(skyVec[1]) * SQR(heightCloud)
                                + 2.0 * SQR(skyVec[2]) * radiusWorld * heightCloud
                                + SQR(skyVec[2]) * SQR(heightCloud))
                                .sqrt()));

                s_cloudTexP[i as usize][t as usize][s as usize] = p;

                // compute intersection point based on p
                VectorScale(addr_of!(skyVec) as *const f32, p, addr_of_mut!(v) as *mut f32);
                v[2] += radiusWorld;

                // compute vector from world origin to intersection point 'v'
                VectorNormalize(addr_of_mut!(v) as *mut f32);

                sRad = Q_acos(v[0]);
                tRad = Q_acos(v[1]);

                s_cloudTexCoords[i as usize][t as usize][s as usize][0] = sRad;
                s_cloudTexCoords[i as usize][t as usize][s as usize][1] = tRad;

                s += 1;
            }
            t += 1;
        }
        i += 1;
    }
}

//======================================================================================

/*
** RB_DrawSun
*/
pub unsafe fn RB_DrawSun() {
    let mut size: f32;
    let mut dist: f32;
    let mut origin: vec3_t = [0.0; 3];
    let mut vec1: vec3_t = [0.0; 3];
    let mut vec2: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];

    if !(*addr_of_mut!(backEnd)).skyRenderedThisView {
        return;
    }
    if (*addr_of!(r_drawSun)).integer == 0 {
        return;
    }
    qglLoadMatrixf(addr_of!((*addr_of_mut!(backEnd).viewParms.world.modelMatrix)) as *const f32);
    qglTranslatef(
        (*addr_of_mut!(backEnd).viewParms.ori.origin)[0],
        (*addr_of_mut!(backEnd).viewParms.ori.origin)[1],
        (*addr_of_mut!(backEnd).viewParms.ori.origin)[2],
    );

    dist = (*addr_of_mut!(backEnd).viewParms).zFar / 1.75; // div sqrt(3)
    size = dist * 0.4;

    VectorScale(addr_of!((*addr_of!(tr)).sunDirection) as *const f32, dist, addr_of_mut!(origin) as *mut f32);
    PerpendicularVector(addr_of_mut!(vec1) as *mut f32, addr_of!((*addr_of!(tr)).sunDirection) as *const f32);
    CrossProduct(addr_of!((*addr_of!(tr)).sunDirection) as *const f32, addr_of!(vec1) as *const f32, addr_of_mut!(vec2) as *mut f32);

    VectorScale(addr_of!(vec1) as *const f32, size, addr_of_mut!(vec1) as *mut f32);
    VectorScale(addr_of!(vec2) as *const f32, size, addr_of_mut!(vec2) as *mut f32);

    // farthest depth range
    qglDepthRange(1.0, 1.0);

    // FIXME: use quad stamp
    RB_BeginSurface((*addr_of!(tr)).sunShader, (*addr_of_mut!(tess)).fogNum);
    VectorCopy(addr_of!(origin) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorSubtract(addr_of!(temp) as *const f32, addr_of!(vec1) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorSubtract(addr_of!(temp) as *const f32, addr_of!(vec2) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorCopy(addr_of!(temp) as *const f32, addr_of_mut!((*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize]) as *mut f32);
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 0.0;
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 0.0;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = 255;
    (*addr_of_mut!(tess)).numVertexes += 1;

    VectorCopy(addr_of!(origin) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorAdd(addr_of!(temp) as *const f32, addr_of!(vec1) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorSubtract(addr_of!(temp) as *const f32, addr_of!(vec2) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorCopy(addr_of!(temp) as *const f32, addr_of_mut!((*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize]) as *mut f32);
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 0.0;
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 1.0;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = 255;
    (*addr_of_mut!(tess)).numVertexes += 1;

    VectorCopy(addr_of!(origin) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorAdd(addr_of!(temp) as *const f32, addr_of!(vec1) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorAdd(addr_of!(temp) as *const f32, addr_of!(vec2) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorCopy(addr_of!(temp) as *const f32, addr_of_mut!((*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize]) as *mut f32);
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 1.0;
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 1.0;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = 255;
    (*addr_of_mut!(tess)).numVertexes += 1;

    VectorCopy(addr_of!(origin) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorSubtract(addr_of!(temp) as *const f32, addr_of!(vec1) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorAdd(addr_of!(temp) as *const f32, addr_of!(vec2) as *const f32, addr_of_mut!(temp) as *mut f32);
    VectorCopy(addr_of!(temp) as *const f32, addr_of_mut!((*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize]) as *mut f32);
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 1.0;
    (*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 0.0;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = 255;
    (*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = 255;
    (*addr_of_mut!(tess)).numVertexes += 1;

    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 0;
    (*addr_of_mut!(tess)).numIndexes += 1;
    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 1;
    (*addr_of_mut!(tess)).numIndexes += 1;
    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 2;
    (*addr_of_mut!(tess)).numIndexes += 1;
    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 0;
    (*addr_of_mut!(tess)).numIndexes += 1;
    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 2;
    (*addr_of_mut!(tess)).numIndexes += 1;
    (*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 3;
    (*addr_of_mut!(tess)).numIndexes += 1;

    RB_EndSurface();

    // back to normal depth range
    qglDepthRange(0.0, 1.0);
}

/*
================
RB_StageIteratorSky

All of the visible sky triangles are in tess

Other things could be stuck in here, like birds in the sky, etc
================
*/
pub unsafe fn RB_StageIteratorSky() {
    if g_bRenderGlowingObjects {
        return;
    }

    if (*addr_of!(r_fastsky)).integer != 0 {
        return;
    }

    if skyboxportal && !((*addr_of_mut!(backEnd).refdef).rdflags & RDF_SKYBOXPORTAL) {
        return;
    }

    // go through all the polygons and project them onto
    // the sky box to see which blocks on each side need
    // to be drawn
    RB_ClipSkyPolygons(addr_of_mut!(tess) as *mut shaderCommands_t);

    // r_showsky will let all the sky blocks be drawn in
    // front of everything to allow developers to see how
    // much sky is getting sucked in
    if (*addr_of!(r_showsky)).integer != 0 {
        qglDepthRange(0.0, 0.0);
    } else {
        #[cfg(target_env = "xbox")]
        {
            qglDepthRange(0.99, 1.0);
        }
        #[cfg(not(target_env = "xbox"))]
        {
            qglDepthRange(1.0, 1.0);
        }
    }

    // draw the outer skybox
    if !(*(*(*addr_of_mut!(tess)).shader).sky).outerbox[0].is_null()
        && (*(*(*addr_of_mut!(tess)).shader).sky).outerbox[0] != (*addr_of!(tr)).defaultImage
    {
        qglColor3f((*addr_of!(tr)).identityLight, (*addr_of!(tr)).identityLight, (*addr_of!(tr)).identityLight);

        qglPushMatrix();
        GL_State(0);
        qglTranslatef(
            (*addr_of_mut!(backEnd).viewParms.ori.origin)[0],
            (*addr_of_mut!(backEnd).viewParms.ori.origin)[1],
            (*addr_of_mut!(backEnd).viewParms.ori.origin)[2],
        );

        DrawSkyBox((*addr_of_mut!(tess)).shader);

        qglPopMatrix();
    }

    // generate the vertexes for all the clouds, which will be drawn
    // by the generic shader routine
    R_BuildCloudData(addr_of_mut!(tess) as *mut shaderCommands_t);

    if (*addr_of_mut!(tess)).numIndexes != 0 && (*addr_of_mut!(tess)).numVertexes != 0 {
        RB_StageIteratorGeneric();
    }

    // draw the inner skybox

    // back to normal depth range
    qglDepthRange(0.0, 1.0);

    // note that sky was drawn so we will draw a sun later
    (*addr_of_mut!(backEnd)).skyRenderedThisView = qtrue;
}
