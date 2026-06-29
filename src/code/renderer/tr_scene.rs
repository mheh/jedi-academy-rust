// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
//
// #include "tr_local.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

use super::tr_types_h::*;

// Type aliases for types from tr_types_h
pub type byte = u8;

// Opaque type stubs for external structs
#[repr(C)]
pub struct trGlobals_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct shader_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct backEndData_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct backEndCommand_t {
    pub used: c_int,
}

#[repr(C)]
pub struct surfaceType_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct srfPoly_t {
    pub surfaceType: c_int,
    pub hShader: qhandle_t,
    pub fogIndex: c_int,
    pub numVerts: c_int,
    pub verts: *mut polyVert_t,
}

#[repr(C)]
pub struct drawSurf_t {
    pub sort: c_int,
    pub surface: *mut surfaceType_t,
}

#[repr(C)]
pub struct dlight_t {
    pub origin: vec3_t,
    pub color: vec3_t,
    pub radius: f32,
    pub transformed: vec3_t,
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub axisLength: f32,
    pub needDlights: qboolean,
    pub lightingCalculated: qboolean,
    pub lightDir: vec3_t,
    pub ambientLight: vec3_t,
    pub ambientLightInt: c_int,
    pub directedLight: vec3_t,
    pub dlightBits: c_int,
}

#[repr(C)]
pub struct fog_t {
    pub originalBrushNumber: c_int,
    pub bounds: [vec3_t; 2],
    pub colorInt: c_int,
    pub tcScale: f32,
    pub parms: fogParms_t,
    pub hasSurface: qboolean,
    pub surface: [f32; 4],
}

#[repr(C)]
pub struct fogParms_t {
    pub color: vec3_t,
    pub depthForOpaque: f32,
}

#[repr(C)]
pub struct world_t {
    pub numfogs: c_int,
    pub fogs: *mut fog_t,
}

#[repr(C)]
pub struct viewParms_t {
    pub or_origin: vec3_t,
    pub or_axis: [vec3_t; 3],
    pub viewOrigin: vec3_t,
    pub modelMatrix: [f32; 16],
    pub pvsOrigin: vec3_t,
    pub isPortal: qboolean,
    pub isMirror: qboolean,
    pub frameSceneNum: c_int,
    pub frameCount: c_int,
    pub viewportX: c_int,
    pub viewportY: c_int,
    pub viewportWidth: c_int,
    pub viewportHeight: c_int,
    pub fovX: f32,
    pub fovY: f32,
    pub projectionMatrix: [f32; 16],
    pub zFar: f32,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

#[repr(C)]
pub struct image_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct model_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct skin_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct srfTerrain_t {
    pub surfaceType: c_int,
    pub landscape: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct trRefdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    pub viewaxis: [vec3_t; 3],
    pub time: c_int,
    pub frametime: c_int,
    pub rdflags: c_int,
    pub areamask: [byte; 32],  // MAX_MAP_AREA_BYTES
    pub areamaskModified: qboolean,
    pub floatTime: f32,
    pub num_entities: c_int,
    pub entities: *mut trRefEntity_t,
    #[cfg(not(feature = "VV_LIGHTING"))]
    pub num_dlights: c_int,
    #[cfg(not(feature = "VV_LIGHTING"))]
    pub dlights: *mut dlight_t,
    pub numPolys: c_int,
    pub polys: *mut srfPoly_t,
    pub numDrawSurfs: c_int,
    pub drawSurfs: *mut drawSurf_t,
    pub fogIndex: c_int,
}

extern "C" {
    pub static mut tr: core::ffi::c_void;  // Opaque tr global - we'll access it through offsets
    pub static mut glConfig: glconfig_t;
    pub static mut backEndData: *mut backEndData_t;

    // External functions
    fn VID_Printf(level: c_int, fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Sys_Milliseconds() -> c_int;
    fn GLimp_LogComment(comment: *const c_char);
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn AddPointToBounds(p: *const f32, mins: *mut f32, maxs: *mut f32);
    fn R_GetShaderByHandle(hShader: qhandle_t) -> *mut shader_t;
    fn R_AddDrawSurf(surface: *mut surfaceType_t, shader: *mut shader_t, fogIndex: c_int, dlighted: qboolean);
    fn R_FogParmsMatch(fogIndex1: c_int, fogIndex2: c_int) -> qboolean;
    fn R_RenderView(parms: *const viewParms_t);
    fn RE_RenderWorldEffects();

    // Global variable declarations
    pub static mut skyboxportal: c_int;
    pub static mut drawskyboxportal: c_int;

    // Conditional compilation feature
    #[cfg(feature = "VV_LIGHTING")]
    pub static mut VVLightMan: VVLightManager;

    #[cfg(feature = "VV_LIGHTING")]
    pub struct VVLightManager {
        pub num_dlights: c_int,
    }
}

// Helper to access tr fields safely - tr is an opaque extern global
// We use pointer arithmetic to access fields based on trGlobals_t layout
// WARNING: Offsets are from tr_local.h and must be kept in sync
#[inline]
unsafe fn get_tr_registered() -> *mut qboolean {
    &mut *(addr_of_mut!(tr) as *mut qboolean)
}

#[inline]
unsafe fn get_tr_world() -> *mut (*mut world_t) {
    ((addr_of_mut!(tr) as usize + 28) as *mut (*mut world_t))
}

#[inline]
unsafe fn get_tr_refdef() -> *mut trRefdef_t {
    ((addr_of_mut!(tr) as usize + 1560) as *mut trRefdef_t)
}

#[inline]
unsafe fn get_tr_currentEntityNum() -> *mut c_int {
    ((addr_of_mut!(tr) as usize + 1380) as *mut c_int)
}

#[inline]
unsafe fn get_tr_shiftedEntityNum() -> *mut c_int {
    ((addr_of_mut!(tr) as usize + 1384) as *mut c_int)
}

#[inline]
unsafe fn get_tr_frameSceneNum() -> *mut c_int {
    ((addr_of_mut!(tr) as usize + 20) as *mut c_int)
}

#[inline]
unsafe fn get_tr_sceneCount() -> *mut c_int {
    ((addr_of_mut!(tr) as usize + 12) as *mut c_int)
}

#[inline]
unsafe fn get_tr_frontEndMsec() -> *mut c_int {
    ((addr_of_mut!(tr) as usize + 1544) as *mut c_int)
}

#[inline]
unsafe fn get_tr_r_norefresh() -> *mut (*mut cvar_t) {
    ((addr_of_mut!(tr) as usize + 11664) as *mut (*mut cvar_t))
}

#[inline]
unsafe fn get_tr_r_dynamiclight() -> *mut (*mut cvar_t) {
    ((addr_of_mut!(tr) as usize + 11668) as *mut (*mut cvar_t))
}

#[inline]
unsafe fn get_tr_r_vertexLight() -> *mut (*mut cvar_t) {
    ((addr_of_mut!(tr) as usize + 11672) as *mut (*mut cvar_t))
}

// Global variables
pub static mut r_firstSceneDrawSurf: c_int = 0;

pub static mut r_numdlights: c_int = 0;
pub static mut r_firstSceneDlight: c_int = 0;

pub static mut r_numentities: c_int = 0;
pub static mut r_firstSceneEntity: c_int = 0;

pub static mut r_numpolys: c_int = 0;
pub static mut r_firstScenePoly: c_int = 0;

pub static mut r_numpolyverts: c_int = 0;

// skyboxportal and drawskyboxportal are declared as extern "C" above

pub const MAX_POLYS: c_int = 4096;
pub const MAX_POLYVERTS: c_int = 12000;

pub const QSORT_ENTITYNUM_SHIFT: c_int = 7;

// Render def flags
pub const RDF_doLAGoggles: c_int = 32;
pub const RDF_doFullbright: c_int = 64;
pub const RDF_SKYBOXPORTAL: c_int = 8;
pub const RDF_DRAWSKYBOX: c_int = 16;
pub const RDF_NOWORLDMODEL: c_int = 1;

pub const qtrue: c_int = 1;
pub const qfalse: c_int = 0;

/*
====================
R_ToggleSmpFrame

====================
*/
pub unsafe extern "C" fn R_ToggleSmpFrame() {
    (*backEndData).commands.used = 0;

    r_firstSceneDrawSurf = 0;

    r_numdlights = 0;
    r_firstSceneDlight = 0;

    #[cfg(feature = "VV_LIGHTING")]
    {
        VVLightMan.num_dlights = 0;
    }

    r_numentities = 0;
    r_firstSceneEntity = 0;

    r_numpolys = 0;
    r_firstScenePoly = 0;

    r_numpolyverts = 0;
}

/*
====================
RE_ClearScene

====================
*/
pub unsafe extern "C" fn RE_ClearScene() {
    r_firstSceneDlight = r_numdlights;
    r_firstSceneEntity = r_numentities;
    r_firstScenePoly = r_numpolys;
    (*get_tr_refdef()).rdflags &= !(RDF_doLAGoggles | RDF_doFullbright);	//probably not needed since it gets copied over in RE_RenderScene
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
pub unsafe extern "C" fn R_AddPolygonSurfaces() {
    let mut i: c_int;
    let mut sh: *mut shader_t;
    let mut poly: *mut srfPoly_t;

    *get_tr_currentEntityNum() = TR_WORLDENT;
    *get_tr_shiftedEntityNum() = *get_tr_currentEntityNum() << QSORT_ENTITYNUM_SHIFT;

    i = 0;
    poly = (*get_tr_refdef()).polys;
    while i < (*get_tr_refdef()).numPolys {
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
pub unsafe extern "C" fn RE_AddPolyToScene(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t) {
    let mut poly: *mut srfPoly_t;
    let mut i: c_int;
    let mut fogIndex: c_int = 0;
    let mut fog: *mut fog_t;
    let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];

    if *get_tr_registered() == 0 {
        return;
    }

    if hShader == 0 {
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            VID_Printf(1, b"WARNING: RE_AddPolyToScene: NULL poly shader\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    if r_numpolyverts + numVerts > MAX_POLYVERTS || r_numpolys >= MAX_POLYS {
        #[cfg(feature = "DEBUG")]
        {
            Com_Error(1, b"Poly overflow!  Tell Brian.\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    poly = &mut (*backEndData).polys[r_numpolys as usize];
    (*poly).surfaceType = 5; // SF_POLY
    (*poly).hShader = hShader;
    (*poly).numVerts = numVerts;
    (*poly).verts = &mut (*backEndData).polyVerts[r_numpolyverts as usize];

    core::ptr::copy_nonoverlapping(verts, (*poly).verts, numVerts as usize);
    r_numpolys += 1;
    r_numpolyverts += numVerts;

    // see if it is in a fog volume
    if *get_tr_world() == core::ptr::null_mut() || (**get_tr_world()).numfogs == 1 {
        fogIndex = 0;
    } else {
        // find which fog volume the poly is in
        VectorCopy((*poly).verts[0].xyz.as_ptr(), bounds[0].as_mut_ptr());
        VectorCopy((*poly).verts[0].xyz.as_ptr(), bounds[1].as_mut_ptr());
        i = 1;
        while i < (*poly).numVerts {
            AddPointToBounds((*poly).verts[i as usize].xyz.as_ptr(), bounds[0].as_mut_ptr(), bounds[1].as_mut_ptr());
            i += 1;
        }
        let mut fI: c_int = 1;
        while fI < (**get_tr_world()).numfogs {
            fog = &mut (**get_tr_world()).fogs[fI as usize];
            if bounds[0][0] >= (*fog).bounds[0][0]
                && bounds[0][1] >= (*fog).bounds[0][1]
                && bounds[0][2] >= (*fog).bounds[0][2]
                && bounds[1][0] <= (*fog).bounds[1][0]
                && bounds[1][1] <= (*fog).bounds[1][1]
                && bounds[1][2] <= (*fog).bounds[1][2]
            {
                //completely in this one
                fogIndex = fI;
                break;
            } else if (bounds[0][0] >= (*fog).bounds[0][0] && bounds[0][1] >= (*fog).bounds[0][1] && bounds[0][2] >= (*fog).bounds[0][2] &&
                        bounds[0][0] <= (*fog).bounds[1][0] && bounds[0][1] <= (*fog).bounds[1][1] && bounds[0][2] <= (*fog).bounds[1][2])
                || (bounds[1][0] >= (*fog).bounds[0][0] && bounds[1][1] >= (*fog).bounds[0][1] && bounds[1][2] >= (*fog).bounds[0][2] &&
                    bounds[1][0] <= (*fog).bounds[1][0] && bounds[1][1] <= (*fog).bounds[1][1] && bounds[1][2] <= (*fog).bounds[1][2])
            {
                //partially in this one
                if (*get_tr_refdef()).fogIndex == fI || R_FogParmsMatch((*get_tr_refdef()).fogIndex, fI) != 0 {
                    //take new one only if it's the same one that the viewpoint is in
                    fogIndex = fI;
                    break;
                } else if fogIndex == 0 {
                    //didn't find one yet, so use this one
                    fogIndex = fI;
                }
            }
            fI += 1;
        }
    }
    (*poly).fogIndex = fogIndex;
}

//=================================================================================

/*
=====================
RE_AddRefEntityToScene

=====================
*/
pub unsafe extern "C" fn RE_AddRefEntityToScene(ent: *const refEntity_t) {
    if *get_tr_registered() == 0 {
        return;
    }
    if r_numentities >= TR_WORLDENT {
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            VID_Printf(1, b"WARNING: RE_AddRefEntityToScene: too many entities\n\0".as_ptr() as *const c_char);
        }
        return;
    }
    if (*ent).reType as c_int < 0 || (*ent).reType as c_int >= 12 {
        // RT_MAX_REF_ENTITY_TYPE = 12
        Com_Error(
            1,
            b"RE_AddRefEntityToScene: bad reType %i\0".as_ptr() as *const c_char,
            (*ent).reType as c_int,
        );
    }

    (*backEndData).entities[r_numentities as usize].e = *ent;
    (*backEndData).entities[r_numentities as usize].lightingCalculated = qfalse;

    r_numentities += 1;
}

/*
=====================
RE_AddLightToScene

=====================
*/
pub unsafe extern "C" fn RE_AddLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32) {
    #[cfg(not(feature = "VV_LIGHTING"))]
    {
        let mut dl: *mut dlight_t;

        if *get_tr_registered() == 0 {
            return;
        }
        if r_numdlights >= MAX_DLIGHTS {
            return;
        }
        if intensity <= 0.0 {
            return;
        }
        dl = &mut (*backEndData).dlights[r_numdlights as usize];
        r_numdlights += 1;
        VectorCopy(org as *const f32, (*dl).origin.as_mut_ptr());
        (*dl).radius = intensity;
        (*dl).color[0] = r;
        (*dl).color[1] = g;
        (*dl).color[2] = b;
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
extern "C" {
    pub static mut recursivePortalCount: c_int;
}

pub unsafe extern "C" fn RE_RenderScene(fd: *const refdef_t) {
    let mut parms: viewParms_t;
    let mut startTime: c_int;
    static mut lastTime: c_int = 0;

    if *get_tr_registered() == 0 {
        return;
    }
    GLimp_LogComment(b"====== RE_RenderScene =====\n\0".as_ptr() as *const c_char);

    if *get_tr_r_norefresh() != core::ptr::null_mut() && (**get_tr_r_norefresh()).integer != 0 {
        return;
    }

    startTime = Sys_Milliseconds();

    if *get_tr_world() == core::ptr::null_mut() && ((*fd).rdflags & RDF_NOWORLDMODEL) == 0 {
        Com_Error(1, b"R_RenderScene: NULL worldmodel\0".as_ptr() as *const c_char);
    }

    //	memcpy( tr.refdef.text, fd->text, sizeof( tr.refdef.text ) );

    (*get_tr_refdef()).x = (*fd).x;
    (*get_tr_refdef()).y = (*fd).y;
    (*get_tr_refdef()).width = (*fd).width;
    (*get_tr_refdef()).height = (*fd).height;
    (*get_tr_refdef()).fov_x = (*fd).fov_x;
    (*get_tr_refdef()).fov_y = (*fd).fov_y;

    VectorCopy((*fd).vieworg.as_ptr(), (*get_tr_refdef()).vieworg.as_mut_ptr());
    VectorCopy((*fd).viewaxis[0].as_ptr(), (*get_tr_refdef()).viewaxis[0].as_mut_ptr());
    VectorCopy((*fd).viewaxis[1].as_ptr(), (*get_tr_refdef()).viewaxis[1].as_mut_ptr());
    VectorCopy((*fd).viewaxis[2].as_ptr(), (*get_tr_refdef()).viewaxis[2].as_mut_ptr());

    (*get_tr_refdef()).time = (*fd).time;
    (*get_tr_refdef()).frametime = (*fd).time - lastTime;
    (*get_tr_refdef()).rdflags = (*fd).rdflags;

    if ((*fd).rdflags & RDF_SKYBOXPORTAL) != 0 {
        skyboxportal = 1;
    } else {
        // cdr - only change last time for the real render, not the portal
        lastTime = (*fd).time;
    }

    if ((*fd).rdflags & RDF_DRAWSKYBOX) != 0 {
        drawskyboxportal = 1;
    } else {
        drawskyboxportal = 0;
    }

    // copy the areamask data over and note if it has changed, which
    // will force a reset of the visible leafs even if the view hasn't moved
    (*get_tr_refdef()).areamaskModified = qfalse;
    if ((*get_tr_refdef()).rdflags & RDF_NOWORLDMODEL) == 0 {
        let mut areaDiff: c_int;
        let mut i: c_int;

        // compare the area bits
        areaDiff = 0;
        i = 0;
        while i < 32 / 4 {
            // MAX_MAP_AREA_BYTES/4
            areaDiff |= *((*get_tr_refdef()).areamask.as_ptr() as *const c_int).add(i as usize)
                ^ *((*fd).areamask.as_ptr() as *const c_int).add(i as usize);
            *((*get_tr_refdef()).areamask.as_mut_ptr() as *mut c_int).add(i as usize) =
                *((*fd).areamask.as_ptr() as *const c_int).add(i as usize);
            i += 1;
        }

        if areaDiff != 0 {
            // a door just opened or something
            (*get_tr_refdef()).areamaskModified = qtrue;
        }
    }

    // derived info

    (*get_tr_refdef()).floatTime = ((*get_tr_refdef()).time as f32) * 0.001;

    (*get_tr_refdef()).numDrawSurfs = r_firstSceneDrawSurf;
    (*get_tr_refdef()).drawSurfs = (*backEndData).drawSurfs;

    (*get_tr_refdef()).num_entities = r_numentities - r_firstSceneEntity;
    (*get_tr_refdef()).entities = &mut (*backEndData).entities[r_firstSceneEntity as usize];

    #[cfg(not(feature = "VV_LIGHTING"))]
    {
        (*get_tr_refdef()).num_dlights = r_numdlights - r_firstSceneDlight;
        (*get_tr_refdef()).dlights = &mut (*backEndData).dlights[r_firstSceneDlight as usize];
    }

    (*get_tr_refdef()).numPolys = r_numpolys - r_firstScenePoly;
    (*get_tr_refdef()).polys = &mut (*backEndData).polys[r_firstScenePoly as usize];

    // turn off dynamic lighting globally by clearing all the
    // dlights if it needs to be disabled or if vertex lighting is enabled
    #[cfg(not(feature = "VV_LIGHTING"))]
    {
        if *get_tr_r_dynamiclight() != core::ptr::null_mut() && (**get_tr_r_dynamiclight()).integer == 0
            || *get_tr_r_vertexLight() != core::ptr::null_mut() && (**get_tr_r_vertexLight()).integer == 1
        {
            (*get_tr_refdef()).num_dlights = 0;
        }
    }

    // a single frame may have multiple scenes draw inside it --
    // a 3D game view, 3D status bar renderings, 3D menus, etc.
    // They need to be distinguished by the light flare code, because
    // the visibility state for a given surface may be different in
    // each scene / view.
    *get_tr_frameSceneNum() += 1;
    *get_tr_sceneCount() += 1;

    // setup view parms for the initial view
    //
    // set up viewport
    // The refdef takes 0-at-the-top y coordinates, so
    // convert to GL's 0-at-the-bottom space
    //
    parms = mem::zeroed::<viewParms_t>();
    parms.viewportX = (*get_tr_refdef()).x;
    parms.viewportY = glConfig.vidHeight - ((*get_tr_refdef()).y + (*get_tr_refdef()).height);
    parms.viewportWidth = (*get_tr_refdef()).width;
    parms.viewportHeight = (*get_tr_refdef()).height;
    parms.isPortal = qfalse;

    parms.fovX = (*get_tr_refdef()).fov_x;
    parms.fovY = (*get_tr_refdef()).fov_y;

    VectorCopy((*fd).vieworg.as_ptr(), parms.or_origin.as_mut_ptr());
    VectorCopy((*fd).viewaxis[0].as_ptr(), parms.or_axis[0].as_mut_ptr());
    VectorCopy((*fd).viewaxis[1].as_ptr(), parms.or_axis[1].as_mut_ptr());
    VectorCopy((*fd).viewaxis[2].as_ptr(), parms.or_axis[2].as_mut_ptr());

    VectorCopy((*fd).vieworg.as_ptr(), parms.pvsOrigin.as_mut_ptr());

    recursivePortalCount = 0;
    R_RenderView(&parms);

    // the next scene rendered in this frame will tack on after this one
    r_firstSceneDrawSurf = (*get_tr_refdef()).numDrawSurfs;
    r_firstSceneEntity = r_numentities;
    r_firstSceneDlight = r_numdlights;
    r_firstScenePoly = r_numpolys;

    *get_tr_frontEndMsec() += Sys_Milliseconds() - startTime;
    RE_RenderWorldEffects();
}
