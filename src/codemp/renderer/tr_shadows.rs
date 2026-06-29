//Anything above this #include will be ignored by the compiler
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::addr_of_mut;

/*

  for a projection shadow:

  point[x] += light vector * ( z - shadow plane )
  point[y] +=
  point[z] = shadow plane

  1 0 light[x] / light[z]

*/

// Type aliases
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type qboolean = c_int;

const QTRUE: c_int = 1;
const QFALSE: c_int = 0;

const MAX_EDGE_DEFS: usize = 32;
const SHADER_MAX_VERTEXES: usize = 1000;
const SHADER_MAX_INDEXES: usize = 6000;

#[repr(C)]
pub struct edgeDef_t {
    pub i2: c_int,
    pub facing: c_int,
}

// Local static arrays
static mut edgeDefs: [[edgeDef_t; MAX_EDGE_DEFS]; SHADER_MAX_VERTEXES] =
    [[edgeDef_t { i2: 0, facing: 0 }; MAX_EDGE_DEFS]; SHADER_MAX_VERTEXES];
static mut numEdgeDefs: [c_int; SHADER_MAX_VERTEXES] = [0; SHADER_MAX_VERTEXES];
static mut facing: [c_int; SHADER_MAX_INDEXES / 3] = [0; SHADER_MAX_INDEXES / 3];

// Type stubs for structural coherence
#[repr(C)]
pub struct glState_t {
    pub stencilBits: c_int,
    // Placeholder for other fields
}

#[repr(C)]
pub struct image_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct glconfig_t {
    pub stencilBits: c_int,
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub maxTextureSize: c_int,
    // Placeholder for other fields
}

#[repr(C)]
pub struct orientationr_t {
    pub origin: vec3_t,
    pub axis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct viewParms_t {
    pub isMirror: qboolean,
    // Placeholder for other fields
}

#[repr(C)]
pub struct miniRefEntity_t {
    pub shadowPlane: f32,
    // Placeholder for other fields
}

#[repr(C)]
pub struct refEntity_t {
    pub e: miniRefEntity_t,
    pub lightDir: vec3_t,
    pub directedLight: [f32; 3],
    // Placeholder for other fields
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub lightDir: vec3_t,
    // Placeholder for other fields
}

#[repr(C)]
pub struct trRefdef_t {
    pub num_dlights: c_int,
    pub dlights: *mut c_void,
    pub time: c_int,
    // Placeholder for other fields
}

#[repr(C)]
pub struct backEndState_t {
    pub currentEntity: *mut trRefEntity_t,
    pub ori: orientationr_t,
    pub viewParms: viewParms_t,
    pub refdef: trRefdef_t,
    // Placeholder for other fields
}

#[repr(C)]
pub struct shaderCommands_t {
    pub xyz: [[f32; 4]; SHADER_MAX_VERTEXES],
    pub indexes: [c_int; SHADER_MAX_INDEXES],
    pub numVertexes: c_int,
    pub numIndexes: c_int,
    // Placeholder for other fields
}

#[repr(C)]
pub struct trGlobals_t {
    pub whiteImage: *mut image_t,
    pub screenImage: *mut image_t,
    // Placeholder for other fields
}

// External C globals
extern "C" {
    pub static mut tess: shaderCommands_t;
    pub static mut backEnd: backEndState_t;
    pub static mut tr: trGlobals_t;
    pub static mut glConfig: glconfig_t;

    // Vector operations
    pub fn VectorCopy(in_: *const f32, out: *mut f32);
    pub fn VectorNormalize(v: *mut f32) -> f32;
    pub fn VectorAdd(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, out: *mut f32);
    pub fn VectorSubtract(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn CrossProduct(v1: *const f32, v2: *const f32, cross: *mut f32);
    pub fn DotProduct(v1: *const f32, v2: *const f32) -> f32;
    pub fn Com_Memset(dst: *mut c_void, c: c_int, count: usize) -> *mut c_void;

    // GL functions
    pub fn qglBegin(mode: c_int);
    pub fn qglEnd();
    pub fn qglVertex3fv(v: *const f32);
    pub fn qglVertex2f(x: f32, y: f32);
    pub fn qglVertex3f(x: f32, y: f32, z: f32);
    pub fn qglColor3f(red: f32, green: f32, blue: f32);
    pub fn qglColor4f(red: f32, green: f32, blue: f32, alpha: f32);
    pub fn qglEnable(cap: c_int);
    pub fn qglDisable(cap: c_int);
    pub fn qglStencilFunc(func: c_int, ref_: c_int, mask: c_int);
    pub fn qglStencilOp(fail: c_int, zfail: c_int, zpass: c_int);
    pub fn qglColorMask(red: qboolean, green: qboolean, blue: qboolean, alpha: qboolean);
    pub fn qglDepthFunc(func: c_int);
    pub fn qglCullFace(mode: c_int);
    pub fn qglPushMatrix();
    pub fn qglPopMatrix();
    pub fn qglLoadIdentity();
    pub fn qglMatrixMode(mode: c_int);
    pub fn qglOrtho(left: f32, right: f32, bottom: f32, top: f32, nearVal: f32, farVal: f32);
    pub fn qglTexCoord2f(s: f32, t: f32);
    pub fn qglIsEnabled(cap: c_int) -> qboolean;

    // Rendering functions
    pub fn GL_Bind(image: *mut image_t);
    pub fn GL_State(state: c_int);
    pub fn GL_Cull(cullType: c_int);

    // Screen capture
    pub fn qglCopyTexImage2D(
        target: c_int,
        level: c_int,
        internalformat: c_int,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        border: c_int,
    );
    pub fn qglReadPixels(
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        format: c_int,
        type_: c_int,
        pixels: *mut c_void,
    );
    pub fn qglTexImage2D(
        target: c_int,
        level: c_int,
        internalformat: c_int,
        width: c_int,
        height: c_int,
        border: c_int,
        format: c_int,
        type_: c_int,
        pixels: *const c_void,
    );
    pub fn qglPolygonMode(face: c_int, mode: c_int);
}

// GL constants (stub definitions for reference)
const GL_TRIANGLE_STRIP: c_int = 0x0005;
const GL_TRIANGLES: c_int = 0x0004;
const GL_QUADS: c_int = 0x0007;
const GL_STENCIL_TEST: c_int = 0x0B90;
const GL_ALWAYS: c_int = 0x0207;
const GL_KEEP: c_int = 0x1E00;
const GL_INCR: c_int = 0x1E02;
const GL_DECR: c_int = 0x1E03;
const GL_FALSE: c_int = 0;
const GL_TRUE: c_int = 1;
const GL_CULL_FACE: c_int = 0x0B44;
const GL_FRONT: c_int = 0x0404;
const GL_BACK: c_int = 0x0405;
const GL_FRONT_AND_BACK: c_int = 0x0408;
const GL_LINE: c_int = 0x1B01;
const GL_FILL: c_int = 0x1B02;
const GL_DEPTH_TEST: c_int = 0x0B71;
const GL_NOTEQUAL: c_int = 0x0205;
const GL_LESS: c_int = 0x0201;
const GL_LEQUAL: c_int = 0x0203;
const GL_CLIP_PLANE0: c_int = 0x3000;
const GL_PROJECTION: c_int = 0x1701;
const GL_MODELVIEW: c_int = 0x1700;
const GL_RGBA: c_int = 0x1908;
const GL_RGBA16: c_int = 0x805B;
const GL_UNSIGNED_BYTE: c_int = 0x1401;
const GL_TEXTURE_2D: c_int = 0x0DE1;

// GL state constants
const GLS_SRCBLEND_ONE: c_int = 0x00000001;
const GLS_DSTBLEND_ZERO: c_int = 0x00000000;
const GLS_SRCBLEND_SRC_ALPHA: c_int = 0x00000004;
const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: c_int = 0x00000008;
const GLS_DSTBLEND_SRC_ALPHA: c_int = 0x0000000C;
const GLS_DSTBLEND_ONE_MINUS_SRC_COLOR: c_int = 0x00000014;
const GLS_DEPTHMASK_TRUE: c_int = 0x00010000;

// CT constants for GL_Cull
const CT_FRONT_SIDED: c_int = 0;
const CT_BACK_SIDED: c_int = 1;
const CT_TWO_SIDED: c_int = 2;

unsafe fn R_AddEdgeDef(i1: c_int, i2: c_int, facing: c_int) {
    let i1_usize = i1 as usize;
    let c = numEdgeDefs[i1_usize] as usize;

    if c == MAX_EDGE_DEFS {
        return; // overflow
    }
    edgeDefs[i1_usize][c].i2 = i2;
    edgeDefs[i1_usize][c].facing = facing;

    numEdgeDefs[i1_usize] += 1;
}

unsafe fn R_RenderShadowEdges() {
    let mut i: c_int;
    let mut c: c_int;
    let mut j: c_int;
    let mut i2: c_int;
    let mut c_edges: c_int = 0;
    let mut c_rejected: c_int = 0;

    // an edge is NOT a silhouette edge if its face doesn't face the light,
    // or if it has a reverse paired edge that also faces the light.
    // A well behaved polyhedron would have exactly two faces for each edge,
    // but lots of models have dangling edges or overfanned edges
    c_edges = 0;
    c_rejected = 0;

    i = 0;
    while i < tess.numVertexes {
        c = numEdgeDefs[i as usize];
        j = 0;
        while j < c {
            if edgeDefs[i as usize][j as usize].facing == 0 {
                j += 1;
                continue;
            }

            //with this system we can still get edges shared by more than 2 tris which
            //produces artifacts including seeing the shadow through walls. So for now
            //we are going to render all edges even though it is a tiny bit slower. -rww
            i2 = edgeDefs[i as usize][j as usize].i2;
            qglBegin(GL_TRIANGLE_STRIP);
            qglVertex3fv(addr_of_mut!(tess.xyz[i as usize][0]));
            qglVertex3fv(addr_of_mut!(tess.xyz[(i + tess.numVertexes) as usize][0]));
            qglVertex3fv(addr_of_mut!(tess.xyz[i2 as usize][0]));
            qglVertex3fv(addr_of_mut!(tess.xyz[(i2 + tess.numVertexes) as usize][0]));
            qglEnd();

            j += 1;
        }
        i += 1;
    }
}

/*
=================
RB_ShadowTessEnd

triangleFromEdge[ v1 ][ v2 ]


  set triangle from edge( v1, v2, tri )
  if ( facing[ triangleFromEdge[ v1 ][ v2 ] ] && !facing[ triangleFromEdge[ v2 ][ v1 ] ) {
  }
=================
*/

unsafe fn RB_DoShadowTessEnd(lightPos: *const f32) {
    let mut i: c_int;
    let mut numTris: c_int;
    let mut lightDir: vec3_t = [0.0; 3];

    // we can only do this if we have enough space in the vertex buffers
    if tess.numVertexes >= (SHADER_MAX_VERTEXES / 2) as c_int {
        return;
    }

    if glConfig.stencilBits < 4 {
        return;
    }

    //controlled method - try to keep shadows in range so they don't show through so much -rww
    let mut worldxyz: vec3_t = [0.0; 3];
    let mut entLight: vec3_t = [0.0; 3];
    let mut groundDist: f32;

    VectorCopy(
        addr_of_mut!(backEnd.currentEntity.as_mut().unwrap().lightDir[0]) as *const f32,
        addr_of_mut!(entLight[0]),
    );
    entLight[2] = 0.0f32;
    VectorNormalize(addr_of_mut!(entLight[0]));

    //Oh well, just cast them straight down no matter what onto the ground plane.
    //This presets no chance of screwups and still looks better than a stupid
    //shader blob.
    VectorSet(
        addr_of_mut!(lightDir[0]),
        entLight[0] * 0.3f32,
        entLight[1] * 0.3f32,
        1.0f32,
    );
    // project vertexes away from light direction
    i = 0;
    while i < tess.numVertexes {
        //add or.origin to vert xyz to end up with world oriented coord, then figure
        //out the ground pos for the vert to project the shadow volume to
        VectorAdd(
            addr_of_mut!(tess.xyz[i as usize][0]) as *const f32,
            addr_of_mut!(backEnd.ori.origin[0]) as *const f32,
            addr_of_mut!(worldxyz[0]),
        );
        groundDist = worldxyz[2] - backEnd.currentEntity.as_ref().unwrap().e.shadowPlane;
        groundDist += 16.0f32; //fudge factor
        VectorMA(
            addr_of_mut!(tess.xyz[i as usize][0]) as *const f32,
            -groundDist,
            addr_of_mut!(lightDir[0]) as *const f32,
            addr_of_mut!(tess.xyz[(i + tess.numVertexes) as usize][0]),
        );
        i += 1;
    }

    // decide which triangles face the light
    Com_Memset(
        addr_of_mut!(numEdgeDefs[0]) as *mut c_void,
        0,
        4 * tess.numVertexes as usize,
    );

    numTris = tess.numIndexes / 3;
    i = 0;
    while i < numTris {
        let mut i1: c_int;
        let mut i2: c_int;
        let mut i3: c_int;
        let mut d1: vec3_t = [0.0; 3];
        let mut d2: vec3_t = [0.0; 3];
        let mut normal: vec3_t = [0.0; 3];
        let mut v1: *mut f32;
        let mut v2: *mut f32;
        let mut v3: *mut f32;
        let mut d: f32;

        i1 = tess.indexes[(i * 3 + 0) as usize];
        i2 = tess.indexes[(i * 3 + 1) as usize];
        i3 = tess.indexes[(i * 3 + 2) as usize];

        v1 = addr_of_mut!(tess.xyz[i1 as usize][0]);
        v2 = addr_of_mut!(tess.xyz[i2 as usize][0]);
        v3 = addr_of_mut!(tess.xyz[i3 as usize][0]);

        if lightPos.is_null() {
            VectorSubtract(v2 as *const f32, v1 as *const f32, addr_of_mut!(d1[0]));
            VectorSubtract(v3 as *const f32, v1 as *const f32, addr_of_mut!(d2[0]));
            CrossProduct(
                addr_of_mut!(d1[0]) as *const f32,
                addr_of_mut!(d2[0]) as *const f32,
                addr_of_mut!(normal[0]),
            );

            d = DotProduct(addr_of_mut!(normal[0]) as *const f32, addr_of_mut!(lightDir[0]) as *const f32);
        } else {
            let mut planeEq: [f32; 4] = [0.0; 4];
            planeEq[0] = v1[1] * (v2[2] - v3[2]) + v2[1] * (v3[2] - v1[2]) + v3[1] * (v1[2] - v2[2]);
            planeEq[1] = v1[2] * (v2[0] - v3[0]) + v2[2] * (v3[0] - v1[0]) + v3[2] * (v1[0] - v2[0]);
            planeEq[2] = v1[0] * (v2[1] - v3[1]) + v2[0] * (v3[1] - v1[1]) + v3[0] * (v1[1] - v2[1]);
            planeEq[3] = -(v1[0] * (v2[1] * v3[2] - v3[1] * v2[2])
                + v2[0] * (v3[1] * v1[2] - v1[1] * v3[2])
                + v3[0] * (v1[1] * v2[2] - v2[1] * v1[2]));

            d = planeEq[0] * *lightPos + planeEq[1] * *(lightPos.add(1)) + planeEq[2] * *(lightPos.add(2))
                + planeEq[3];
        }

        if d > 0.0 {
            facing[i as usize] = 1;
        } else {
            facing[i as usize] = 0;
        }

        // create the edges
        R_AddEdgeDef(i1, i2, facing[i as usize]);
        R_AddEdgeDef(i2, i3, facing[i as usize]);
        R_AddEdgeDef(i3, i1, facing[i as usize]);

        i += 1;
    }

    GL_Bind(tr.whiteImage);
    //qglEnable( GL_CULL_FACE );
    GL_State((GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO) as c_int);

    qglColor3f(0.2f32, 0.2f32, 0.2f32);

    // don't write to the color buffer
    qglColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);

    qglEnable(GL_STENCIL_TEST);
    qglStencilFunc(GL_ALWAYS, 1, 255);

    qglDepthFunc(GL_LESS);

    //now using the Carmack Reverse<tm> -rww
    if backEnd.viewParms.isMirror != 0 {
        //qglCullFace( GL_BACK );
        GL_Cull(CT_BACK_SIDED);
        qglStencilOp(GL_KEEP, GL_INCR, GL_KEEP);

        R_RenderShadowEdges();

        //qglCullFace( GL_FRONT );
        GL_Cull(CT_FRONT_SIDED);
        qglStencilOp(GL_KEEP, GL_DECR, GL_KEEP);

        R_RenderShadowEdges();
    } else {
        //qglCullFace( GL_FRONT );
        GL_Cull(CT_FRONT_SIDED);
        qglStencilOp(GL_KEEP, GL_INCR, GL_KEEP);

        R_RenderShadowEdges();

        //qglCullFace( GL_BACK );
        GL_Cull(CT_BACK_SIDED);
        qglStencilOp(GL_KEEP, GL_DECR, GL_KEEP);

        R_RenderShadowEdges();
    }

    qglDepthFunc(GL_LEQUAL);

    // reenable writing to the color buffer
    qglColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
}

#[no_mangle]
pub unsafe extern "C" fn RB_ShadowTessEnd() {
    RB_DoShadowTessEnd(core::ptr::null());
}

/*
=================
RB_ShadowFinish

Darken everything that is is a shadow volume.
We have to delay this until everything has been shadowed,
because otherwise shadows from different body parts would
overlap and double darken.
=================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_ShadowFinish() {
    let mut planeZeroBack: bool = false;

    // r_shadows->integer != 2 check would require extern cvar access
    // For now, we proceed with the logic (commented check in original)

    if glConfig.stencilBits < 4 {
        return;
    }

    qglEnable(GL_STENCIL_TEST);
    qglStencilFunc(GL_NOTEQUAL, 0, 255);

    qglStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);

    planeZeroBack = false;
    if qglIsEnabled(GL_CLIP_PLANE0) != 0 {
        planeZeroBack = true;
        qglDisable(GL_CLIP_PLANE0);
    }
    GL_Cull(CT_TWO_SIDED);
    //qglDisable (GL_CULL_FACE);

    GL_Bind(tr.whiteImage);

    qglPushMatrix();
    qglLoadIdentity();

    qglColor4f(0.0f32, 0.0f32, 0.0f32, 0.5f32);
    //GL_State( GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA );
    GL_State((GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as c_int);

    qglBegin(GL_QUADS);
    qglVertex3f(-100.0f32, 100.0f32, -10.0f32);
    qglVertex3f(100.0f32, 100.0f32, -10.0f32);
    qglVertex3f(100.0f32, -100.0f32, -10.0f32);
    qglVertex3f(-100.0f32, -100.0f32, -10.0f32);
    qglEnd();

    qglColor4f(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    qglDisable(GL_STENCIL_TEST);
    if planeZeroBack {
        qglEnable(GL_CLIP_PLANE0);
    }
    qglPopMatrix();
}

/*
=================
RB_ProjectionShadowDeform

=================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_ProjectionShadowDeform() {
    let mut xyz: *mut f32;
    let mut i: c_int;
    let mut h: f32;
    let mut ground: vec3_t = [0.0; 3];
    let mut light: vec3_t = [0.0; 3];
    let mut groundDist: f32;
    let mut d: f32;
    let mut lightDir: vec3_t = [0.0; 3];

    xyz = addr_of_mut!(tess.xyz[0][0]);

    ground[0] = backEnd.ori.axis[0][2];
    ground[1] = backEnd.ori.axis[1][2];
    ground[2] = backEnd.ori.axis[2][2];

    groundDist = backEnd.ori.origin[2] - backEnd.currentEntity.as_ref().unwrap().e.shadowPlane;

    VectorCopy(
        addr_of_mut!(backEnd.currentEntity.as_mut().unwrap().lightDir[0]) as *const f32,
        addr_of_mut!(lightDir[0]),
    );
    d = DotProduct(addr_of_mut!(lightDir[0]) as *const f32, addr_of_mut!(ground[0]) as *const f32);
    // don't let the shadows get too long or go negative
    if d < 0.5 {
        VectorMA(
            addr_of_mut!(lightDir[0]) as *const f32,
            0.5 - d,
            addr_of_mut!(ground[0]) as *const f32,
            addr_of_mut!(lightDir[0]),
        );
        d = DotProduct(addr_of_mut!(lightDir[0]) as *const f32, addr_of_mut!(ground[0]) as *const f32);
    }
    d = 1.0 / d;

    light[0] = lightDir[0] * d;
    light[1] = lightDir[1] * d;
    light[2] = lightDir[2] * d;

    i = 0;
    while i < tess.numVertexes {
        h = DotProduct(xyz, addr_of_mut!(ground[0]) as *const f32) + groundDist;

        *xyz -= light[0] * h;
        *xyz.add(1) -= light[1] * h;
        *xyz.add(2) -= light[2] * h;

        xyz = xyz.add(4);
        i += 1;
    }
}

//update tr.screenImage
#[no_mangle]
pub unsafe extern "C" fn RB_CaptureScreenImage() {
    let mut radX: c_int = 2048;
    let mut radY: c_int = 2048;
    let mut x: c_int = glConfig.vidWidth / 2;
    let mut y: c_int = glConfig.vidHeight / 2;
    let mut cX: c_int;
    let mut cY: c_int;

    GL_Bind(tr.screenImage);
    //using this method, we could pixel-filter the texture and all sorts of crazy stuff.
    //but, it is slow as hell.
    /*
    static byte *tmp = NULL;
    if (!tmp)
    {
        tmp = (byte *)Z_Malloc((sizeof(byte)*4)*(glConfig.vidWidth*glConfig.vidHeight), TAG_ICARUS, qtrue);
    }
    qglReadPixels(0, 0, glConfig.vidWidth, glConfig.vidHeight, GL_RGBA, GL_UNSIGNED_BYTE, tmp);
    qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 512, 512, 0, GL_RGBA, GL_UNSIGNED_BYTE, tmp);
    */

    if radX > glConfig.maxTextureSize {
        radX = glConfig.maxTextureSize;
    }
    if radY > glConfig.maxTextureSize {
        radY = glConfig.maxTextureSize;
    }

    while glConfig.vidWidth < radX {
        radX /= 2;
    }
    while glConfig.vidHeight < radY {
        radY /= 2;
    }

    cX = x - (radX / 2);
    cY = y - (radY / 2);

    if cX + radX > glConfig.vidWidth {
        //would it go off screen?
        cX = glConfig.vidWidth - radX;
    } else if cX < 0 {
        //cap it off at 0
        cX = 0;
    }

    if cY + radY > glConfig.vidHeight {
        //would it go off screen?
        cY = glConfig.vidHeight - radY;
    } else if cY < 0 {
        //cap it off at 0
        cY = 0;
    }

    qglCopyTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16, cX, cY, radX, radY, 0);
}

//yeah.. not really shadow-related.. but it's stencil-related. -rww
static mut tr_distortionAlpha: f32 = 1.0f32; //opaque
static mut tr_distortionStretch: f32 = 0.0f32; //no stretch override
static mut tr_distortionPrePost: qboolean = QFALSE; //capture before postrender phase?
static mut tr_distortionNegate: qboolean = QFALSE; //negative blend mode

#[no_mangle]
pub unsafe extern "C" fn RB_DistortionFill() {
    let mut alpha: f32 = tr_distortionAlpha;
    let mut spost: f32 = 0.0f32;
    let mut spost2: f32 = 0.0f32;

    if glConfig.stencilBits < 4 {
        return;
    }

    //ok, cap the stupid thing now I guess
    if tr_distortionPrePost == 0 {
        RB_CaptureScreenImage();
    }

    qglEnable(GL_STENCIL_TEST);
    qglStencilFunc(GL_NOTEQUAL, 0, 0xFFFFFFFF as c_int);
    qglStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);

    qglDisable(GL_CLIP_PLANE0);
    GL_Cull(CT_TWO_SIDED);

    //reset the view matrices and go into ortho mode
    qglMatrixMode(GL_PROJECTION);
    qglPushMatrix();
    qglLoadIdentity();
    qglOrtho(
        0.0f32,
        glConfig.vidWidth as f32,
        glConfig.vidHeight as f32,
        32.0f32,
        -1.0f32,
        1.0f32,
    );
    qglMatrixMode(GL_MODELVIEW);
    qglPushMatrix();
    qglLoadIdentity();

    if tr_distortionStretch != 0.0f32 {
        //override
        spost = tr_distortionStretch;
        spost2 = tr_distortionStretch;
    } else {
        //do slow stretchy effect
        spost = (backEnd.refdef.time as f32 * 0.0005f32).sin();
        if spost < 0.0f32 {
            spost = -spost;
        }
        spost *= 0.2f32;

        spost2 = (backEnd.refdef.time as f32 * 0.0005f32).sin();
        if spost2 < 0.0f32 {
            spost2 = -spost2;
        }
        spost2 *= 0.08f32;
    }

    if alpha != 1.0f32 {
        //blend
        GL_State((GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_SRC_ALPHA) as c_int);
    } else {
        //be sure to reset the draw state
        GL_State(0);
    }

    qglBegin(GL_QUADS);
    qglColor4f(1.0f32, 1.0f32, 1.0f32, alpha);
    qglTexCoord2f(0.0f32 + spost2, 1.0f32 - spost);
    qglVertex2f(0.0f32, 0.0f32);

    qglTexCoord2f(0.0f32 + spost2, 0.0f32 + spost);
    qglVertex2f(0.0f32, glConfig.vidHeight as f32);

    qglTexCoord2f(1.0f32 - spost2, 0.0f32 + spost);
    qglVertex2f(glConfig.vidWidth as f32, glConfig.vidHeight as f32);

    qglTexCoord2f(1.0f32 - spost2, 1.0f32 - spost);
    qglVertex2f(glConfig.vidWidth as f32, 0.0f32);
    qglEnd();

    if tr_distortionAlpha == 1.0f32 && tr_distortionStretch == 0.0f32 {
        //no overrides
        if tr_distortionNegate != 0 {
            //probably the crazy alternate saber trail
            alpha = 0.8f32;
            GL_State((GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE_MINUS_SRC_COLOR) as c_int);
        } else {
            alpha = 0.5f32;
            GL_State((GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_SRC_ALPHA) as c_int);
        }

        spost = (backEnd.refdef.time as f32 * 0.0008f32).sin();
        if spost < 0.0f32 {
            spost = -spost;
        }
        spost *= 0.08f32;

        spost2 = (backEnd.refdef.time as f32 * 0.0008f32).sin();
        if spost2 < 0.0f32 {
            spost2 = -spost2;
        }
        spost2 *= 0.2f32;

        qglBegin(GL_QUADS);
        qglColor4f(1.0f32, 1.0f32, 1.0f32, alpha);
        qglTexCoord2f(0.0f32 + spost2, 1.0f32 - spost);
        qglVertex2f(0.0f32, 0.0f32);

        qglTexCoord2f(0.0f32 + spost2, 0.0f32 + spost);
        qglVertex2f(0.0f32, glConfig.vidHeight as f32);

        qglTexCoord2f(1.0f32 - spost2, 0.0f32 + spost);
        qglVertex2f(glConfig.vidWidth as f32, glConfig.vidHeight as f32);

        qglTexCoord2f(1.0f32 - spost2, 1.0f32 - spost);
        qglVertex2f(glConfig.vidWidth as f32, 0.0f32);
        qglEnd();
    }

    //pop the view matrices back
    qglMatrixMode(GL_PROJECTION);
    qglPopMatrix();
    qglMatrixMode(GL_MODELVIEW);
    qglPopMatrix();

    qglDisable(GL_STENCIL_TEST);
}
