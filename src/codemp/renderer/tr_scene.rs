//Anything above this #include will be ignored by the compiler

use core::ffi::c_int;

// Stub imports for types defined in other modules
// These represent external declarations from the C codebase
pub use crate::codemp::renderer::tr_local_h::*;
pub use crate::codemp::ghoul2::G2_h::*;
pub use crate::codemp::ghoul2::G2_local_h::*;

// Forward declarations for types that need to be defined in other modules
extern "C" {
    pub static mut backEndData: *mut BackEndData;
    pub static mut tr: TR;
    pub static mut glConfig: GLConfig;
    pub static r_markcount: *mut Cvar;
    pub static com_timescale: *mut Cvar;
    pub static r_norefresh: *mut Cvar;
    pub static r_dynamiclight: *mut Cvar;
    pub static r_vertexLight: *mut Cvar;

    pub fn Com_Printf(fmt: *const core::ffi::c_char, ...) -> ();
    pub fn Com_Memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, count: usize) -> ();
    pub fn Com_Memset(dest: *mut core::ffi::c_void, c: core::ffi::c_int, count: usize) -> ();
    pub fn Com_Error(code: core::ffi::c_int, fmt: *const core::ffi::c_char, ...) -> !;
    pub fn R_GetShaderByHandle(handle: qhandle_t) -> *mut shader_t;
    pub fn R_AddDrawSurf(surface: *mut surfaceType_t, sh: *mut shader_t, fogIndex: c_int, allow_merging: qboolean) -> ();
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t) -> ();
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> ();
    pub fn PerpendicularVector(dst: *mut vec3_t, src: *const vec3_t) -> ();
    pub fn RotatePointAroundVector(dst: *mut vec3_t, dir: *const vec3_t, point: *const vec3_t, degrees: f32) -> ();
    pub fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t) -> ();
    pub fn VectorSubtract(va: *const vec3_t, vb: *const vec3_t, vc: *mut vec3_t) -> ();
    pub fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
    pub fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t) -> ();
    pub fn AddPointToBounds(v: *const vec3_t, mins: *mut vec3_t, maxs: *mut vec3_t) -> ();
    pub fn GLimp_LogComment(comment: *const core::ffi::c_char) -> ();
    pub fn Sys_Milliseconds() -> c_int;
    pub fn R_MarkFragments(numPoints: c_int, points: *const *const vec3_t, projection: vec3_t, maxPoints: c_int, pointBuffer: *mut vec3_t, maxFragments: c_int, fragmentBuffer: *mut markFragment_t) -> c_int;
    pub fn R_RenderView(parms: *mut viewParms_t) -> ();
}

static mut r_firstSceneDrawSurf: c_int = 0;

static mut r_numdlights: c_int = 0;
static mut r_firstSceneDlight: c_int = 0;

static mut r_numentities: c_int = 0;
static mut r_firstSceneEntity: c_int = 0;
static mut r_numminientities: c_int = 0;
static mut r_firstSceneMiniEntity: c_int = 0;
static mut refEntParent: c_int = -1;

static mut r_numpolys: c_int = 0;
static mut r_firstScenePoly: c_int = 0;

static mut r_numpolyverts: c_int = 0;

pub static mut skyboxportal: c_int = 0;
pub static mut drawskyboxportal: c_int = 0;

/*
====================
R_ToggleSmpFrame

====================
*/
pub unsafe fn R_ToggleSmpFrame() {
    (*backEndData).commands.used = 0;

    r_firstSceneDrawSurf = 0;

    #[cfg(feature = "vv_lighting")]
    {
        VVLightMan.num_dlights = 0;
    }
    r_numdlights = 0;
    r_firstSceneDlight = 0;

    r_numentities = 0;
    r_firstSceneEntity = 0;
    refEntParent = -1;
    r_numminientities = 0;
    r_firstSceneMiniEntity = 0;

    r_numpolys = 0;
    r_firstScenePoly = 0;

    r_numpolyverts = 0;
}


/*
====================
RE_ClearScene

====================
*/
pub unsafe fn RE_ClearScene() {
    r_firstSceneDlight = r_numdlights;
    r_firstSceneEntity = r_numentities;
    r_firstScenePoly = r_numpolys;
    refEntParent = -1;
    r_firstSceneMiniEntity = r_numminientities;
}

/*
===========================================================================

DISCRETE POLYS

===========================================================================
*/

/*
=====================
R_AddPolygonSurfaces

Adds all the scene's polys into this view's drawsurf list
=====================
*/
pub unsafe fn R_AddPolygonSurfaces() {
    let mut i: c_int;
    let mut sh: *mut shader_t;
    let mut poly: *mut srfPoly_t;

    (*tr).currentEntityNum = TR_WORLDENT;
    (*tr).shiftedEntityNum = (*tr).currentEntityNum << QSORT_ENTITYNUM_SHIFT;

    i = 0;
    poly = (*(*tr).refdef).polys;
    while i < (*(*tr).refdef).numPolys {
        sh = R_GetShaderByHandle((*poly).hShader);
        R_AddDrawSurf(poly as *mut surfaceType_t, sh, (*poly).fogIndex, qfalse);

        i += 1;
        poly = poly.add(1);
    }
}

/*
=====================
RE_AddPolyToScene

=====================
*/
pub unsafe fn RE_AddPolyToScene(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t, numPolys: c_int) {
    let mut poly: *mut srfPoly_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut fogIndex: c_int;
    let mut fog: *mut fog_t;
    let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];

    if (*tr).registered == 0 {
        return;
    }

    if hShader == 0 {
        Com_Printf(b"WARNING: RE_AddPolyToScene: NULL poly shader\n" as *const _ as *const core::ffi::c_char);
        return;
    }

    j = 0;
    while j < numPolys {
        if r_numpolyverts + numVerts > max_polyverts || r_numpolys >= max_polys {
            Com_Printf(b"WARNING: RE_AddPolyToScene: r_max_polys or r_max_polyverts reached\n" as *const _ as *const core::ffi::c_char);
            return;
        }

        poly = core::ptr::addr_of_mut!((*backEndData).polys[r_numpolys as usize]);
        (*poly).surfaceType = SF_POLY;
        (*poly).hShader = hShader;
        (*poly).numVerts = numVerts;
        (*poly).verts = core::ptr::addr_of_mut!((*backEndData).polyVerts[r_numpolyverts as usize]);

        Com_Memcpy((*poly).verts as *mut core::ffi::c_void, verts.add((numVerts * j) as usize) as *const core::ffi::c_void, (numVerts as usize) * core::mem::size_of_val(&*verts));

        // done.
        r_numpolys += 1;
        r_numpolyverts += numVerts;

        // if no world is loaded
        if (*tr).world == core::ptr::null_mut() {
            fogIndex = 0;
        }
        // see if it is in a fog volume
        else if (*(*tr).world).numfogs == 1 {
            fogIndex = 0;
        } else {
            // find which fog volume the poly is in
            VectorCopy(&(*(*poly).verts).xyz, &mut bounds[0]);
            VectorCopy(&(*(*poly).verts).xyz, &mut bounds[1]);
            i = 1;
            while i < (*poly).numVerts {
                AddPointToBounds(&(*(*poly).verts.add(i as usize)).xyz, &mut bounds[0], &mut bounds[1]);
                i += 1;
            }
            fogIndex = 1;
            while fogIndex < (*(*tr).world).numfogs {
                fog = core::ptr::addr_of_mut!((*(*tr).world).fogs[fogIndex as usize]);
                if (*bounds[1])[0] >= (*(*fog).bounds[0])[0]
                    && (*bounds[1])[1] >= (*(*fog).bounds[0])[1]
                    && (*bounds[1])[2] >= (*(*fog).bounds[0])[2]
                    && (*bounds[0])[0] <= (*(*fog).bounds[1])[0]
                    && (*bounds[0])[1] <= (*(*fog).bounds[1])[1]
                    && (*bounds[0])[2] <= (*(*fog).bounds[1])[2] {
                    break;
                }
                fogIndex += 1;
            }
            if fogIndex == (*(*tr).world).numfogs {
                fogIndex = 0;
            }
        }
        (*poly).fogIndex = fogIndex;
        j += 1;
    }
}


//=================================================================================


/*
=====================
RE_AddRefEntityToScene

=====================
*/
pub unsafe fn RE_AddRefEntityToScene(ent: *const refEntity_t) {
    if (*tr).registered == 0 {
        return;
    }

    assert!(!ent.is_null() || (*ent).renderfx >= 0);

    if (*ent).reType == RT_ENT_CHAIN {
        //minirefents must die.
        return;
    }

    #[cfg(debug_assertions)]
    {
        if (*ent).reType == RT_MODEL {
            assert!((*ent).hModel != 0 || !(*ent).ghoul2.is_null() || (*ent).customShader != 0);
        }
    }

    if r_numentities >= TR_WORLDENT as c_int {
        #[cfg(not(feature = "final_build"))]
        {
            Com_Printf(b"WARNING: RE_AddRefEntityToScene: too many entities\n" as *const _ as *const core::ffi::c_char);
        }
        return;
    }
    if (*ent).reType < 0 || (*ent).reType >= RT_MAX_REF_ENTITY_TYPE as c_int {
        Com_Error(ERR_DROP, b"RE_AddRefEntityToScene: bad reType %i\0" as *const _ as *const core::ffi::c_char, (*ent).reType);
    }

    (*backEndData).entities[r_numentities as usize].e = *ent;
    (*backEndData).entities[r_numentities as usize].lightingCalculated = qfalse;

    if !(*ent).ghoul2.is_null() {
        let ghoul2: *mut CGhoul2Info_v = (*ent).ghoul2 as *mut CGhoul2Info_v;

        if (*(*ghoul2)[0]).mModel.is_null() {
            #[cfg(debug_assertions)]
            {
                let _g2: *mut CGhoul2Info = &mut (*ghoul2)[0];
            }
            //DebugBreak();
            Com_Printf(b"Your ghoul2 instance has no model!\n" as *const _ as *const core::ffi::c_char);
        }
    }

    /*
    if (ent->reType == RT_ENT_CHAIN)
    {
        refEntParent = r_numentities;
        backEndData->entities[r_numentities].e.uRefEnt.uMini.miniStart = r_numminientities - r_firstSceneMiniEntity;
        backEndData->entities[r_numentities].e.uRefEnt.uMini.miniCount = 0;
    }
    else
    {
    */
    refEntParent = -1;
    //}

    r_numentities += 1;
}


/************************************************************************************************
 * RE_AddMiniRefEntityToScene                                                                   *
 *    Adds a mini ref ent to the scene.  If the input parameter is null, it signifies the end   *
 *    of the chain.  Otherwise, if there is a valid chain parent, it will be added to that.     *
 *    If there is no parent, it will be added as a regular ref ent.                             *
 *                                                                                              *
 * Input                                                                                        *
 *    ent: the mini ref ent to be added                                                         *
 *                                                                                              *
 * Output / Return                                                                              *
 *    none                                                                                      *
 *                                                                                              *
 ************************************************************************************************/
pub unsafe fn RE_AddMiniRefEntityToScene(ent: *const miniRefEntity_t) {
    #[allow(dead_code)]
    #[allow(unused_variables)]
    let parent: *mut refEntity_t;

    if (*tr).registered == 0 {
        return;
    }
    if ent.is_null() {
        refEntParent = -1;
        return;
    }

    if true { //i hate you minirefent!
        let mut tempEnt: refEntity_t;

        tempEnt = core::mem::zeroed();
        core::ptr::copy_nonoverlapping(ent as *const core::ffi::c_char, core::ptr::addr_of_mut!(tempEnt) as *mut core::ffi::c_char, core::mem::size_of_val(&*ent));
        core::mem::set_bytes(
            (core::ptr::addr_of_mut!(tempEnt) as *mut core::ffi::c_char).add(core::mem::size_of_val(&*ent)),
            0,
            core::mem::size_of::<refEntity_t>() - core::mem::size_of_val(&*ent)
        );
        RE_AddRefEntityToScene(&tempEnt);
    }
    // else {
    //
    //    if ( ent->reType < 0 || ent->reType >= RT_MAX_REF_ENTITY_TYPE )
    //    {
    //        Com_Error( ERR_DROP, "RE_AddMiniRefEntityToScene: bad reType %i", ent->reType );
    //    }
    //
    //    if (!r_numentities || refEntParent == -1 || r_numminientities >= MAX_MINI_ENTITIES)
    //    { //rww - add it as a refent also if we run out of minis
    // //        Com_Error( ERR_DROP, "RE_AddMiniRefEntityToScene: mini without parent ref ent");
    //        refEntity_t		tempEnt;
    //
    //        memcpy(&tempEnt, ent, sizeof(*ent));
    //        memset(((char *)&tempEnt)+sizeof(*ent), 0, sizeof(tempEnt) - sizeof(*ent));
    //        RE_AddRefEntityToScene(&tempEnt);
    //        return;
    //    }
    //
    //    parent = &backEndData->entities[refEntParent].e;
    //    parent->uRefEnt.uMini.miniCount++;
    //
    //    backEndData->miniEntities[r_numminientities].e = *ent;
    //    r_numminientities++;
    // }
}

/*
=====================
RE_AddDynamicLightToScene

=====================
*/
#[cfg(not(feature = "vv_lighting"))]
pub unsafe fn RE_AddDynamicLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32, additive: c_int) {
    let mut dl: *mut dlight_t;

    if (*tr).registered == 0 {
        return;
    }
    if r_numdlights >= MAX_DLIGHTS as c_int {
        return;
    }
    if intensity <= 0.0 {
        return;
    }
    dl = core::ptr::addr_of_mut!((*backEndData).dlights[r_numdlights as usize]);
    r_numdlights += 1;
    VectorCopy(org, &mut (*dl).origin);
    (*dl).radius = intensity;
    (*dl).color[0] = r;
    (*dl).color[1] = g;
    (*dl).color[2] = b;
    (*dl).additive = additive;
}

/*
=====================
RE_AddLightToScene

=====================
*/
#[cfg(not(feature = "vv_lighting"))]
pub unsafe fn RE_AddLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32) {
    RE_AddDynamicLightToScene(org, intensity, r, g, b, qfalse);
}

/*
=====================
RE_AddAdditiveLightToScene

=====================
*/
#[cfg(not(feature = "vv_lighting"))]
pub unsafe fn RE_AddAdditiveLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32) {
    RE_AddDynamicLightToScene(org, intensity, r, g, b, qtrue);
}


enum DECALPOLY_TYPE {
    DECALPOLY_TYPE_NORMAL = 0,
    DECALPOLY_TYPE_FADE = 1,
    DECALPOLY_TYPE_MAX = 2,
}

const DECAL_FADE_TIME: c_int = 1000;

pub unsafe fn RE_AllocDecal(type_: c_int) -> *mut decalPoly_t;

static mut re_decalPolys: [[decalPoly_t; MAX_DECAL_POLYS as usize]; DECALPOLY_TYPE_MAX as usize] = [[core::mem::zeroed(); MAX_DECAL_POLYS as usize]; DECALPOLY_TYPE_MAX as usize];

static mut re_decalPolyHead: [c_int; DECALPOLY_TYPE_MAX as usize] = [0; DECALPOLY_TYPE_MAX as usize];
static mut re_decalPolyTotal: [c_int; DECALPOLY_TYPE_MAX as usize] = [0; DECALPOLY_TYPE_MAX as usize];

/*
===================
RE_ClearDecals

This is called to remove all decals from the world
===================
*/

pub unsafe fn RE_ClearDecals() {
    Com_Memset(core::ptr::addr_of_mut!(re_decalPolys) as *mut core::ffi::c_void, 0, core::mem::size_of_val(&re_decalPolys));
    Com_Memset(core::ptr::addr_of_mut!(re_decalPolyHead) as *mut core::ffi::c_void, 0, core::mem::size_of_val(&re_decalPolyHead));
    Com_Memset(core::ptr::addr_of_mut!(re_decalPolyTotal) as *mut core::ffi::c_void, 0, core::mem::size_of_val(&re_decalPolyTotal));
}

pub unsafe fn R_InitDecals() {
    RE_ClearDecals();
}

pub unsafe fn RE_FreeDecal(type_: c_int, index: c_int) {
    if re_decalPolys[type_ as usize][index as usize].time == 0 {
        return;
    }

    if type_ == DECALPOLY_TYPE::DECALPOLY_TYPE_NORMAL as c_int {
        let fade: *mut decalPoly_t;

        fade = RE_AllocDecal(DECALPOLY_TYPE::DECALPOLY_TYPE_FADE as c_int);

        Com_Memcpy(fade as *mut core::ffi::c_void, core::ptr::addr_of!(re_decalPolys[type_ as usize][index as usize]) as *const core::ffi::c_void, core::mem::size_of::<decalPoly_t>());

        (*fade).time = (*tr).refdef.time;
        (*fade).fadetime = (*tr).refdef.time + DECAL_FADE_TIME;
    }

    re_decalPolys[type_ as usize][index as usize].time = 0;

    re_decalPolyTotal[type_ as usize] -= 1;
}

/*
===================
RE_AllocDecal

Will allways succeed, even if it requires freeing an old active mark
===================
*/
pub unsafe fn RE_AllocDecal(type_: c_int) -> *mut decalPoly_t {
    let mut le: *mut decalPoly_t;

    // See if the cvar changed
    if re_decalPolyTotal[type_ as usize] > (*r_markcount).integer {
        RE_ClearDecals();
    }

    le = core::ptr::addr_of_mut!(re_decalPolys[type_ as usize][re_decalPolyHead[type_ as usize] as usize]);

    // If it has no time its the first occasion its been used
    if (*le).time != 0 {
        if (*le).time != (*tr).refdef.time {
            let mut i: c_int = re_decalPolyHead[type_ as usize];

            // since we are killing one that existed before, make sure we
            // kill all the other marks that belong to the group
            loop {
                i += 1;
                if i >= (*r_markcount).integer {
                    i = 0;
                }

                // Break out on the first one thats not part of the group
                if re_decalPolys[type_ as usize][i as usize].time != (*le).time {
                    break;
                }

                RE_FreeDecal(type_, i);

                if i == re_decalPolyHead[type_ as usize] {
                    break;
                }
            }

            RE_FreeDecal(type_, re_decalPolyHead[type_ as usize]);
        } else {
            RE_FreeDecal(type_, re_decalPolyHead[type_ as usize]);
        }
    }

    Com_Memset(le as *mut core::ffi::c_void, 0, core::mem::size_of::<decalPoly_t>());
    (*le).time = (*tr).refdef.time;

    re_decalPolyTotal[type_ as usize] += 1;

    // Move on to the next decal poly and wrap around if need be
    re_decalPolyHead[type_ as usize] += 1;
    if re_decalPolyHead[type_ as usize] >= (*r_markcount).integer {
        re_decalPolyHead[type_ as usize] = 0;
    }

    return le;
}


/*
=================
RE_AddDecalToScene

origin should be a point within a unit of the plane
dir should be the plane normal

temporary marks will not be stored or randomly oriented, but immediately
passed to the renderer.
=================
*/
const MAX_DECAL_FRAGMENTS: usize = 128;
const MAX_DECAL_POINTS: usize = 384;

pub unsafe fn RE_AddDecalToScene(decalShader: qhandle_t, origin: *const vec3_t, dir: *const vec3_t, orientation: f32, red: f32, green: f32, blue: f32, alpha: f32, alphaFade: qboolean, radius: f32, temporary: qboolean) {
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut texCoordScale: f32;
    let mut originalPoints: [vec3_t; 4] = [[0.0; 3]; 4];
    let mut colors: [u8; 4];
    let mut i: c_int;
    let mut j: c_int;
    let mut numFragments: c_int;
    let mut markFragments: [markFragment_t; MAX_DECAL_FRAGMENTS] = [core::mem::zeroed(); MAX_DECAL_FRAGMENTS];
    let mut mf: *mut markFragment_t;
    let mut markPoints: [vec3_t; MAX_DECAL_POINTS] = [[0.0; 3]; MAX_DECAL_POINTS];
    let mut projection: vec3_t = [0.0; 3];

    assert!(decalShader != 0);

    if (*r_markcount).integer <= 0 && temporary == 0 {
        return;
    }

    if radius <= 0.0 {
        Com_Error(ERR_FATAL, b"RE_AddDecalToScene:  called with <= 0 radius\0" as *const _ as *const core::ffi::c_char);
    }

    // create the texture axis
    VectorNormalize2(dir, &mut axis[0]);
    PerpendicularVector(&mut axis[1], &axis[0]);
    RotatePointAroundVector(&mut axis[2], &axis[0], &axis[1], orientation);
    CrossProduct(&axis[0], &axis[2], &mut axis[1]);

    texCoordScale = 0.5 * 1.0 / radius;

    // create the full polygon
    i = 0;
    while i < 3 {
        originalPoints[0][i as usize] = (*origin)[i as usize] - radius * axis[1][i as usize] - radius * axis[2][i as usize];
        originalPoints[1][i as usize] = (*origin)[i as usize] + radius * axis[1][i as usize] - radius * axis[2][i as usize];
        originalPoints[2][i as usize] = (*origin)[i as usize] + radius * axis[1][i as usize] + radius * axis[2][i as usize];
        originalPoints[3][i as usize] = (*origin)[i as usize] - radius * axis[1][i as usize] + radius * axis[2][i as usize];
        i += 1;
    }

    // get the fragments
    VectorScale(dir, -20.0, &mut projection);
    numFragments = R_MarkFragments(4, &originalPoints[0] as *const vec3_t as *const *const vec3_t,
                    projection, MAX_DECAL_POINTS as c_int, markPoints[0].as_mut_ptr(),
                    MAX_DECAL_FRAGMENTS as c_int, markFragments.as_mut_ptr());

    colors[0] = (red * 255.0) as u8;
    colors[1] = (green * 255.0) as u8;
    colors[2] = (blue * 255.0) as u8;
    colors[3] = (alpha * 255.0) as u8;

    i = 0;
    mf = markFragments.as_mut_ptr();
    while i < numFragments {
        let v: *mut polyVert_t;
        let mut verts: [polyVert_t; MAX_VERTS_ON_DECAL_POLY as usize] = [core::mem::zeroed(); MAX_VERTS_ON_DECAL_POLY as usize];
        let mut decal: *mut decalPoly_t;

        // we have an upper limit on the complexity of polygons
        // that we store persistantly
        if (*mf).numPoints > MAX_VERTS_ON_DECAL_POLY as c_int {
            (*mf).numPoints = MAX_VERTS_ON_DECAL_POLY as c_int;
        }

        j = 0;
        v = verts.as_mut_ptr();
        while j < (*mf).numPoints {
            let mut delta: vec3_t = [0.0; 3];

            VectorCopy(&markPoints[(*mf).firstPoint as usize + j as usize], &mut (*v).xyz);

            VectorSubtract(&(*v).xyz, origin, &mut delta);
            (*v).st[0] = 0.5 + DotProduct(&delta, &axis[1]) * texCoordScale;
            (*v).st[1] = 0.5 + DotProduct(&delta, &axis[2]) * texCoordScale;

            *(core::ptr::addr_of_mut!((*v).modulate) as *mut c_int) = *(colors.as_ptr() as *const c_int);

            j += 1;
            v = v.add(1);
        }

        // if it is a temporary (shadow) mark, add it immediately and forget about it
        if temporary != 0 {
            RE_AddPolyToScene(decalShader, (*mf).numPoints, verts.as_ptr(), 1);
        } else {
            // otherwise save it persistantly
            decal = RE_AllocDecal(DECALPOLY_TYPE::DECALPOLY_TYPE_NORMAL as c_int);
            (*decal).time = (*tr).refdef.time;
            (*decal).shader = decalShader;
            (*decal).poly.numVerts = (*mf).numPoints;
            (*decal).color[0] = red;
            (*decal).color[1] = green;
            (*decal).color[2] = blue;
            (*decal).color[3] = alpha;
            Com_Memcpy((*decal).verts.as_mut_ptr() as *mut core::ffi::c_void, verts.as_ptr() as *const core::ffi::c_void, ((*mf).numPoints as usize) * core::mem::size_of::<polyVert_t>());
        }

        i += 1;
        mf = mf.add(1);
    }
}

/*
===============
R_AddDecals
===============
*/
unsafe fn R_AddDecals() {
    let mut decalPoly: c_int;
    let mut type_: c_int;
    static mut lastMarkCount: c_int = -1;

    if (*r_markcount).integer != lastMarkCount {
        if lastMarkCount != -1 {
            RE_ClearDecals();
        }

        lastMarkCount = (*r_markcount).integer;
    }

    if (*r_markcount).integer <= 0 {
        return;
    }

    type_ = DECALPOLY_TYPE::DECALPOLY_TYPE_NORMAL as c_int;
    while type_ < DECALPOLY_TYPE::DECALPOLY_TYPE_MAX as c_int {
        decalPoly = re_decalPolyHead[type_ as usize];

        loop {
            let p: *mut decalPoly_t = core::ptr::addr_of_mut!(re_decalPolys[type_ as usize][decalPoly as usize]);

            if (*p).time != 0 {
                if (*p).fadetime != 0 {
                    let mut t: c_int;

                    // fade all marks out with time
                    t = (*tr).refdef.time - (*p).time;
                    if t < DECAL_FADE_TIME {
                        let mut fade: f32;
                        let mut j: c_int;

                        fade = 255.0 * (1.0 - ((t as f32) / (DECAL_FADE_TIME as f32)));

                        j = 0;
                        while j < (*p).poly.numVerts {
                            (*p).verts[j as usize].modulate[3] = fade as u8;
                            j += 1;
                        }

                        RE_AddPolyToScene((*p).shader, (*p).poly.numVerts, (*p).verts.as_ptr(), 1);
                    } else {
                        RE_FreeDecal(type_, decalPoly);
                    }
                } else {
                    RE_AddPolyToScene((*p).shader, (*p).poly.numVerts, (*p).verts.as_ptr(), 1);
                }
            }

            decalPoly += 1;
            if decalPoly >= (*r_markcount).integer {
                decalPoly = 0;
            }

            if decalPoly == re_decalPolyHead[type_ as usize] {
                break;
            }
        }

        type_ += 1;
    }
}


/*
@@@@@@@@@@@@@@@@@@@@@
RE_RenderScene

Draw a 3D view into a part of the window, then return
to 2D drawing.

Rendering a scene may require multiple views to be rendered
to handle mirrors,
@@@@@@@@@@@@@@@@@@@@@
*/
pub unsafe fn RE_RenderWorldEffects();
pub unsafe fn RE_RenderAutoMap();
pub unsafe fn RE_RenderScene(fd: *const refdef_t) {
    let mut parms: viewParms_t;
    let mut startTime: c_int;
    static mut lastTime: c_int = 0;

    if (*tr).registered == 0 {
        return;
    }
    GLimp_LogComment(b"====== RE_RenderScene =====\n" as *const _ as *const core::ffi::c_char);

    if (*r_norefresh).integer != 0 {
        return;
    }

    startTime = (Sys_Milliseconds() as f32 * (*com_timescale).value) as c_int;

    if (*tr).world.is_null() && ((*fd).rdflags & RDF_NOWORLDMODEL) == 0 {
        Com_Error(ERR_DROP, b"R_RenderScene: NULL worldmodel\0" as *const _ as *const core::ffi::c_char);
    }

    Com_Memcpy((*(*tr).refdef).text.as_mut_ptr() as *mut core::ffi::c_void, (*fd).text.as_ptr() as *const core::ffi::c_void, core::mem::size_of_val(&(*fd).text));

    (*(*tr).refdef).x = (*fd).x;
    (*(*tr).refdef).y = (*fd).y;
    (*(*tr).refdef).width = (*fd).width;
    (*(*tr).refdef).height = (*fd).height;
    (*(*tr).refdef).fov_x = (*fd).fov_x;
    (*(*tr).refdef).fov_y = (*fd).fov_y;

    VectorCopy(&(*fd).vieworg, &mut (*(*tr).refdef).vieworg);
    VectorCopy(&(*fd).viewaxis[0], &mut (*(*tr).refdef).viewaxis[0]);
    VectorCopy(&(*fd).viewaxis[1], &mut (*(*tr).refdef).viewaxis[1]);
    VectorCopy(&(*fd).viewaxis[2], &mut (*(*tr).refdef).viewaxis[2]);

    (*(*tr).refdef).time = (*fd).time;
    (*(*tr).refdef).frametime = (*fd).time - lastTime;
    lastTime = (*fd).time;

    if ((*fd).rdflags & RDF_SKYBOXPORTAL) != 0 {
        skyboxportal = 1;
    }

    if ((*fd).rdflags & RDF_DRAWSKYBOX) != 0 {
        drawskyboxportal = 1;
    } else {
        drawskyboxportal = 0;
    }

    if (*(*tr).refdef).frametime > 500 {
        (*(*tr).refdef).frametime = 500;
    } else if (*(*tr).refdef).frametime < 0 {
        (*(*tr).refdef).frametime = 0;
    }
    (*(*tr).refdef).rdflags = (*fd).rdflags;

    // copy the areamask data over and note if it has changed, which
    // will force a reset of the visible leafs even if the view hasn't moved
    (*(*tr).refdef).areamaskModified = qfalse;
    if ((*(*tr).refdef).rdflags & RDF_NOWORLDMODEL) == 0 {
        let mut areaDiff: c_int;
        let mut i: c_int;

        // compare the area bits
        areaDiff = 0;
        i = 0;
        while i < (MAX_MAP_AREA_BYTES / 4) as c_int {
            areaDiff |= *(((*(*tr).refdef).areamask.as_ptr() as *const c_int).add(i as usize)) ^ *(((*fd).areamask.as_ptr() as *const c_int).add(i as usize));
            *(((*(*tr).refdef).areamask.as_mut_ptr() as *mut c_int).add(i as usize)) = *(((*fd).areamask.as_ptr() as *const c_int).add(i as usize));
            i += 1;
        }

        if areaDiff != 0 {
            // a door just opened or something
            (*(*tr).refdef).areamaskModified = qtrue;
        }
    }


    // derived info

    (*(*tr).refdef).floatTime = ((*(*tr).refdef).time as f32) * 0.001;

    (*(*tr).refdef).numDrawSurfs = r_firstSceneDrawSurf;
    (*(*tr).refdef).drawSurfs = (*backEndData).drawSurfs.as_mut_ptr();

    (*(*tr).refdef).num_entities = r_numentities - r_firstSceneEntity;
    (*(*tr).refdef).entities = core::ptr::addr_of_mut!((*backEndData).entities[r_firstSceneEntity as usize]);
    (*(*tr).refdef).miniEntities = core::ptr::addr_of_mut!((*backEndData).miniEntities[r_firstSceneMiniEntity as usize]);

    #[cfg(not(feature = "vv_lighting"))]
    {
        (*(*tr).refdef).num_dlights = r_numdlights - r_firstSceneDlight;
        (*(*tr).refdef).dlights = core::ptr::addr_of_mut!((*backEndData).dlights[r_firstSceneDlight as usize]);
    }

    // Add the decals here because decals add polys and we need to ensure
    // that the polys are added before the the renderer is prepared
    if ((*(*tr).refdef).rdflags & RDF_NOWORLDMODEL) == 0 {
        R_AddDecals();
    }

    (*(*tr).refdef).numPolys = r_numpolys - r_firstScenePoly;
    (*(*tr).refdef).polys = core::ptr::addr_of_mut!((*backEndData).polys[r_firstScenePoly as usize]);

    // turn off dynamic lighting globally by clearing all the
    // dlights if it needs to be disabled or if vertex lighting is enabled
    #[cfg(not(feature = "vv_lighting"))]
    {
        if (*r_dynamiclight).integer == 0 || (*r_vertexLight).integer == 1 {
            (*(*tr).refdef).num_dlights = 0;
        }
    }

    // a single frame may have multiple scenes draw inside it --
    // a 3D game view, 3D status bar renderings, 3D menus, etc.
    // They need to be distinguished by the light flare code, because
    // the visibility state for a given surface may be different in
    // each scene / view.
    (*tr).frameSceneNum += 1;
    (*tr).sceneCount += 1;

    // setup view parms for the initial view
    //
    // set up viewport
    // The refdef takes 0-at-the-top y coordinates, so
    // convert to GL's 0-at-the-bottom space
    //
    Com_Memset(core::ptr::addr_of_mut!(parms) as *mut core::ffi::c_void, 0, core::mem::size_of_val(&parms));
    parms.viewportX = (*(*tr).refdef).x;
    parms.viewportY = glConfig.vidHeight - ((*(*tr).refdef).y + (*(*tr).refdef).height);
    parms.viewportWidth = (*(*tr).refdef).width;
    parms.viewportHeight = (*(*tr).refdef).height;
    parms.isPortal = qfalse;

    parms.fovX = (*(*tr).refdef).fov_x;
    parms.fovY = (*(*tr).refdef).fov_y;

    VectorCopy(&(*fd).vieworg, &mut parms.ori.origin);
    VectorCopy(&(*fd).viewaxis[0], &mut parms.ori.axis[0]);
    VectorCopy(&(*fd).viewaxis[1], &mut parms.ori.axis[1]);
    VectorCopy(&(*fd).viewaxis[2], &mut parms.ori.axis[2]);

    VectorCopy(&(*fd).vieworg, &mut parms.pvsOrigin);

    R_RenderView(&mut parms);

    // the next scene rendered in this frame will tack on after this one
    r_firstSceneDrawSurf = (*(*tr).refdef).numDrawSurfs;
    r_firstSceneEntity = r_numentities;
    r_firstSceneMiniEntity = r_numminientities;
    r_firstSceneDlight = r_numdlights;
    r_firstScenePoly = r_numpolys;

    refEntParent = -1;

    (*tr).frontEndMsec += (Sys_Milliseconds() as f32 * (*com_timescale).value) as c_int - startTime;

    RE_RenderWorldEffects();

    if ((*(*tr).refdef).rdflags & RDF_AUTOMAP) != 0 {
        RE_RenderAutoMap();
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
mod _disabled {
    use super::*;

    // #if 0 //rwwFIXMEFIXME: Disable this before release!!!!!! I am just trying to find a crash bug.
    pub unsafe fn R_GetRNumEntities() -> c_int {
        r_numentities
    }

    pub unsafe fn R_SetRNumEntities(num: c_int) {
        r_numentities = num;
    }
    // #endif
}
