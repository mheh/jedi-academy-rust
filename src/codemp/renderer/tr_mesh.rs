// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// tr_mesh.c: triangle model functions

// #include "tr_local.h"

// #ifdef VV_LIGHTING
// #include "tr_lightmanager.h"
// #endif

use core::ffi::{c_int, c_char, c_void};
use core::mem;

// External type declarations
#[repr(C)]
pub struct vec3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Local stubs for unported engine dependencies
#[repr(C)]
pub struct trRefEntity_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct refEntity_t {
    // Placeholder - full definition
}

#[repr(C)]
pub struct md3Header_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct md3Frame_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct md3Surface_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct md3Shader_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct shader_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct model_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct fog_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct skin_t {
    // Placeholder - full definition from tr_local.h
}

#[repr(C)]
pub struct surfaceType_t {
    // Placeholder - full definition from tr_local.h
}

// Global stubs for unported engine state
#[repr(C)]
pub struct backendState_t {
    // Placeholder
}

pub static mut tr: backendState_t = backendState_t {};

// External function declarations
extern "C" {
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn Q_fabs(x: f32) -> f32;
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    fn RadiusFromBounds(mins: *const f32, maxs: *const f32) -> f32;
    fn myftol(f: f32) -> c_int;
    fn R_CullLocalPointAndRadius(origin: *const f32, radius: f32) -> c_int;
    fn R_CullLocalBox(bounds: *const [f32; 3]) -> c_int;
    fn R_GetModelByHandle(handle: c_int) -> *mut model_t;
    fn R_GetShaderByHandle(handle: c_int) -> *mut shader_t;
    fn R_GetSkinByHandle(handle: c_int) -> *mut skin_t;
    fn R_SetupEntityLighting(refdef: *const c_void, ent: *mut trRefEntity_t);
    fn R_AddDrawSurf(surface: *const surfaceType_t, shader: *mut shader_t, fogNum: c_int, dlightBits: c_int);
    fn Com_DPrintf(fmt: *const c_char, ...);
}

// Conditional external for VV_LIGHTING
#[cfg(feature = "vv_lighting")]
extern "C" {
    static mut VVLightMan: VVLightManager;
}

#[cfg(feature = "vv_lighting")]
#[repr(C)]
pub struct VVLightManager {
    // Placeholder
}

// Constants for cull types
const CULL_IN: c_int = 1;
const CULL_CLIP: c_int = 0;
const CULL_OUT: c_int = -1;

// Cvar and shader stubs
extern "C" {
    static r_lodscale: *const cvar_t;
    static r_autolodscalevalue: *const cvar_t;
    static r_lodbias: *const cvar_t;
    static r_shadows: *const cvar_t;
}

#[repr(C)]
pub struct cvar_t {
    // Placeholder
}

const S_COLOR_RED: &[u8] = b"^1";

pub fn ProjectRadius(r: f32, location: [f32; 3]) -> f32 {
    let mut pr: f32;
    let mut dist: f32;
    let c: f32;
    let mut p: [f32; 3] = [0.0; 3];
    let width: f32;
    let depth: f32;

    unsafe {
        c = DotProduct(
            addr_of!((*addr_of!(tr).viewParms.ori.axis[0])) as *const f32,
            addr_of!((*addr_of!(tr).viewParms.ori.origin)) as *const f32,
        );
        dist = DotProduct(
            addr_of!((*addr_of!(tr).viewParms.ori.axis[0])) as *const f32,
            location.as_ptr(),
        ) - c;

        if dist <= 0.0 {
            return 0.0;
        }

        p[0] = 0.0;
        p[1] = Q_fabs(r);
        p[2] = -dist;

        width = p[0] * (*addr_of!(tr).viewParms.projectionMatrix[1])
            + p[1] * (*addr_of!(tr).viewParms.projectionMatrix[5])
            + p[2] * (*addr_of!(tr).viewParms.projectionMatrix[9])
            + (*addr_of!(tr).viewParms.projectionMatrix[13]);

        depth = p[0] * (*addr_of!(tr).viewParms.projectionMatrix[3])
            + p[1] * (*addr_of!(tr).viewParms.projectionMatrix[7])
            + p[2] * (*addr_of!(tr).viewParms.projectionMatrix[11])
            + (*addr_of!(tr).viewParms.projectionMatrix[15]);

        pr = width / depth;
    }

    #[cfg(target_os = "xbox")]
    {
        pr = -pr;
    }

    if pr > 1.0 {
        pr = 1.0;
    }

    pr
}

#[cfg(not(feature = "dedicated"))]
fn R_CullModel(header: *mut md3Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut bounds: [vec3_t; 2] = unsafe { mem::zeroed() };
    let mut oldFrame: *mut md3Frame_t;
    let mut newFrame: *mut md3Frame_t;
    let mut i: c_int;

    unsafe {
        // compute frame pointers
        newFrame = (((header as *mut u8).add((*header).ofsFrames)) as *mut md3Frame_t)
            .add((*ent).e.frame as usize);
        oldFrame = (((header as *mut u8).add((*header).ofsFrames)) as *mut md3Frame_t)
            .add((*ent).e.oldframe as usize);

        // cull bounding sphere ONLY if this is not an upscaled entity
        if !(*ent).e.nonNormalizedAxes {
            if (*ent).e.frame == (*ent).e.oldframe {
                match R_CullLocalPointAndRadius((*newFrame).localOrigin as *const f32, (*newFrame).radius) {
                    CULL_OUT => {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_out) += 1;
                        return CULL_OUT;
                    }
                    CULL_IN => {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_in) += 1;
                        return CULL_IN;
                    }
                    CULL_CLIP => {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_clip) += 1;
                    }
                    _ => {}
                }
            } else {
                let mut sphereCull: c_int;
                let mut sphereCullB: c_int;

                sphereCull = R_CullLocalPointAndRadius((*newFrame).localOrigin as *const f32, (*newFrame).radius);
                if newFrame == oldFrame {
                    sphereCullB = sphereCull;
                } else {
                    sphereCullB = R_CullLocalPointAndRadius((*oldFrame).localOrigin as *const f32, (*oldFrame).radius);
                }

                if sphereCull == sphereCullB {
                    if sphereCull == CULL_OUT {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_out) += 1;
                        return CULL_OUT;
                    } else if sphereCull == CULL_IN {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_in) += 1;
                        return CULL_IN;
                    } else {
                        (*addr_of_mut!(tr).pc.c_sphere_cull_md3_clip) += 1;
                    }
                }
            }
        }

        // calculate a bounding box in the current coordinate system
        i = 0;
        while i < 3 {
            let idx = i as usize;
            let b0_0 = (*oldFrame).bounds[0][idx];
            let b0_1 = (*newFrame).bounds[0][idx];
            let b1_0 = (*oldFrame).bounds[1][idx];
            let b1_1 = (*newFrame).bounds[1][idx];
            bounds[0][idx] = if b0_0 < b0_1 { b0_0 } else { b0_1 };
            bounds[1][idx] = if b1_0 > b1_1 { b1_0 } else { b1_1 };
            i += 1;
        }

        match R_CullLocalBox(bounds.as_ptr() as *const _) {
            CULL_IN => {
                (*addr_of_mut!(tr).pc.c_box_cull_md3_in) += 1;
                return CULL_IN;
            }
            CULL_CLIP => {
                (*addr_of_mut!(tr).pc.c_box_cull_md3_clip) += 1;
                return CULL_CLIP;
            }
            CULL_OUT | _ => {
                (*addr_of_mut!(tr).pc.c_box_cull_md3_out) += 1;
                return CULL_OUT;
            }
        }
    }
}

// =================
// RE_GetModelBounds
//
//   Returns the bounds of the current model
//   (qhandle_t)hModel and (int)frame need to be set
// =================
// rwwRMG - added
pub fn RE_GetModelBounds(refEnt: *mut refEntity_t, bounds1: *mut f32, bounds2: *mut f32) {
    let mut frame: *mut md3Frame_t;
    let mut header: *mut md3Header_t;
    let mut model: *mut model_t;

    unsafe {
        // assert(refEnt);
        if refEnt.is_null() {
            return;
        }

        model = R_GetModelByHandle((*refEnt).hModel);
        // assert(model);
        if model.is_null() {
            return;
        }
        header = (*model).md3[0];
        // assert(header);
        if header.is_null() {
            return;
        }
        frame = (((header as *mut u8).add((*header).ofsFrames)) as *mut md3Frame_t)
            .add((*refEnt).frame as usize);
        // assert(frame);
        if frame.is_null() {
            return;
        }

        VectorCopy((*frame).bounds[0].as_ptr(), bounds1);
        VectorCopy((*frame).bounds[1].as_ptr(), bounds2);
    }
}

// =================
// R_ComputeLOD
//
// =================
#[cfg(not(feature = "dedicated"))]
pub fn R_ComputeLOD(ent: *mut trRefEntity_t) -> c_int {
    let mut radius: f32;
    let mut flod: f32;
    let mut lodscale: f32;
    let mut projectedRadius: f32;
    let mut frame: *mut md3Frame_t;
    let mut lod: c_int;

    unsafe {
        if (*addr_of!(tr).currentModel).numLods < 2 {
            // model has only 1 LOD level, skip computations and bias
            lod = 0;
        } else {
            // multiple LODs exist, so compute projected bounding sphere
            // and use that as a criteria for selecting LOD

            frame = ((((*addr_of!(tr).currentModel).md3[0] as *mut u8)
                .add((*(*addr_of!(tr).currentModel).md3[0]).ofsFrames)) as *mut md3Frame_t);

            frame = frame.add((*ent).e.frame as usize);

            radius = RadiusFromBounds(
                (*frame).bounds[0].as_ptr(),
                (*frame).bounds[1].as_ptr(),
            );

            projectedRadius = ProjectRadius(radius, [
                (*ent).e.origin[0],
                (*ent).e.origin[1],
                (*ent).e.origin[2],
            ]);
            if projectedRadius != 0.0 {
                lodscale = (*r_lodscale).value + (*r_autolodscalevalue).value;
                if lodscale > 20.0 {
                    lodscale = 20.0;
                } else if lodscale < 0.0 {
                    lodscale = 0.0;
                }
                flod = 1.0 - projectedRadius * lodscale;
            } else {
                // object intersects near view plane, e.g. view weapon
                flod = 0.0;
            }

            flod *= (*addr_of!(tr).currentModel).numLods as f32;
            lod = myftol(flod);

            if lod < 0 {
                lod = 0;
            } else if lod >= (*addr_of!(tr).currentModel).numLods {
                lod = (*addr_of!(tr).currentModel).numLods - 1;
            }
        }

        lod += (*r_lodbias).integer;

        if lod >= (*addr_of!(tr).currentModel).numLods {
            lod = (*addr_of!(tr).currentModel).numLods - 1;
        }
        if lod < 0 {
            lod = 0;
        }
    }

    lod
}

// =================
// R_ComputeFogNum
//
// =================
#[cfg(not(feature = "dedicated"))]
pub fn R_ComputeFogNum(header: *mut md3Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut fog: *mut fog_t;
    let mut md3Frame: *mut md3Frame_t;
    let mut localOrigin: [f32; 3] = [0.0; 3];

    unsafe {
        let rdflags = (*addr_of!(tr).refdef).rdflags;
        const RDF_NOWORLDMODEL: c_int = 0x0001; // Stub value
        if (rdflags & RDF_NOWORLDMODEL) != 0 {
            return 0;
        }

        // FIXME: non-normalized axis issues
        md3Frame = (((header as *mut u8).add((*header).ofsFrames)) as *mut md3Frame_t)
            .add((*ent).e.frame as usize);
        VectorAdd(
            (*ent).e.origin.as_ptr(),
            (*md3Frame).localOrigin.as_ptr(),
            localOrigin.as_mut_ptr(),
        );
        i = 1;
        while i < (*addr_of!(tr).world).numfogs {
            fog = addr_of_mut!((*addr_of!(tr).world).fogs[i as usize]);
            j = 0;
            while j < 3 {
                let idx = j as usize;
                if localOrigin[idx] - (*md3Frame).radius >= (*fog).bounds[1][idx] {
                    break;
                }
                if localOrigin[idx] + (*md3Frame).radius <= (*fog).bounds[0][idx] {
                    break;
                }
                j += 1;
            }
            if j == 3 {
                return i;
            }
            i += 1;
        }
    }

    0
}

// =================
// R_AddMD3Surfaces
//
// =================
#[cfg(not(feature = "dedicated"))]
pub fn R_AddMD3Surfaces(ent: *mut trRefEntity_t) {
    let mut i: c_int;
    let mut header: *mut md3Header_t = core::ptr::null_mut();
    let mut surface: *mut md3Surface_t = core::ptr::null_mut();
    let mut md3Shader: *mut md3Shader_t = core::ptr::null_mut();
    let mut shader: *mut shader_t = core::ptr::null_mut();
    let mut cull: c_int;
    let mut lod: c_int;
    let mut fogNum: c_int;
    let mut personalModel: bool;

    unsafe {
        // don't add third_person objects if not in a portal
        let rf_third_person = 0x0002; // Stub value
        let rf_wrap_frames = 0x0004; // Stub value
        let rf_noshadow = 0x0010; // Stub value
        let rf_depthhack = 0x0020; // Stub value
        let rf_shadow_plane = 0x0040; // Stub value

        personalModel = (((*ent).e.renderfx & rf_third_person) != 0)
            && !(*addr_of!(tr).viewParms).isPortal;

        if ((*ent).e.renderfx & rf_wrap_frames) != 0 {
            (*ent).e.frame %= (*addr_of!(tr).currentModel).md3[0].numFrames;
            (*ent).e.oldframe %= (*addr_of!(tr).currentModel).md3[0].numFrames;
        }

        //
        // Validate the frames so there is no chance of a crash.
        // This will write directly into the entity structure, so
        // when the surfaces are rendered, they don't need to be
        // range checked again.
        //
        if ((*ent).e.frame >= (*addr_of!(tr).currentModel).md3[0].numFrames)
            || ((*ent).e.frame < 0)
            || ((*ent).e.oldframe >= (*addr_of!(tr).currentModel).md3[0].numFrames)
            || ((*ent).e.oldframe < 0)
        {
            Com_DPrintf(
                b"R_AddMD3Surfaces: no such frame %d to %d for '%s'\n\0".as_ptr() as *const c_char,
                (*ent).e.oldframe,
                (*ent).e.frame,
                (*addr_of!(tr).currentModel).name as *const c_char,
            );
            (*ent).e.frame = 0;
            (*ent).e.oldframe = 0;
        }

        //
        // compute LOD
        //
        lod = R_ComputeLOD(ent);

        header = (*addr_of!(tr).currentModel).md3[lod as usize];

        //
        // cull the entire model if merged bounding box of both frames
        // is outside the view frustum.
        //
        cull = R_CullModel(header, ent);
        if cull == CULL_OUT {
            return;
        }

        //
        // set up lighting now that we know we aren't culled
        //
        #[cfg(feature = "vv_lighting")]
        {
            if !personalModel {
                VVLightMan.R_SetupEntityLighting(&(*addr_of!(tr).refdef), ent);
            }
        }
        #[cfg(not(feature = "vv_lighting"))]
        {
            if !personalModel || (*r_shadows).integer > 1 {
                R_SetupEntityLighting(addr_of!((*addr_of!(tr).refdef)) as *const c_void, ent);
            }
        }

        //
        // see if we are in a fog volume
        //
        fogNum = R_ComputeFogNum(header, ent);

        //
        // draw all surfaces
        //
        surface = (((header as *mut u8).add((*header).ofsSurfaces)) as *mut md3Surface_t);
        i = 0;
        while i < (*header).numSurfaces {
            if (*ent).e.customShader != 0 {
                shader = R_GetShaderByHandle((*ent).e.customShader);
            } else if (*ent).e.customSkin > 0 && (*ent).e.customSkin < (*addr_of!(tr).numSkins) {
                let mut skin: *mut skin_t;
                let mut j: c_int;

                skin = R_GetSkinByHandle((*ent).e.customSkin);

                // match the surface name to something in the skin file
                shader = (*addr_of!(tr).defaultShader);
                j = 0;
                while j < (*skin).numSurfaces {
                    // the names have both been lowercased
                    if strcmp(
                        (*(*skin).surfaces[j as usize]).name as *const c_char,
                        (*surface).name as *const c_char,
                    ) == 0
                    {
                        shader = (*(*skin).surfaces[j as usize]).shader;
                        break;
                    }
                    j += 1;
                }
                if shader == (*addr_of!(tr).defaultShader) {
                    Com_DPrintf(
                        b"WARNING: no shader for surface %s in skin %s\n\0".as_ptr() as *const c_char,
                        (*surface).name as *const c_char,
                        (*skin).name as *const c_char,
                    );
                } else if (*shader).defaultShader != 0 {
                    Com_DPrintf(
                        b"WARNING: shader %s in skin %s not found\n\0".as_ptr() as *const c_char,
                        (*shader).name as *const c_char,
                        (*skin).name as *const c_char,
                    );
                }
            } else if (*surface).numShaders <= 0 {
                shader = (*addr_of!(tr).defaultShader);
            } else {
                md3Shader = (((surface as *mut u8).add((*surface).ofsShaders)) as *mut md3Shader_t);
                md3Shader = md3Shader.add(((*ent).e.skinNum as usize) % (*surface).numShaders as usize);
                shader = (*addr_of!(tr).shaders[(*md3Shader).shaderIndex as usize]);
            }

            // we will add shadows even if the main object isn't visible in the view

            // stencil shadows can't do personal models unless I polyhedron clip
            if !personalModel
                && (*r_shadows).integer == 2
                && fogNum == 0
                && (((*ent).e.renderfx & (rf_noshadow | rf_depthhack)) == 0)
                && (*shader).sort == 1 /* SS_OPAQUE stub */
            {
                R_AddDrawSurf(
                    surface as *const surfaceType_t,
                    (*addr_of!(tr).shadowShader),
                    0,
                    0,
                );
            }

            // projection shadows work fine with personal models
            if (*r_shadows).integer == 3
                && fogNum == 0
                && (((*ent).e.renderfx & rf_shadow_plane) != 0)
                && (*shader).sort == 1 /* SS_OPAQUE stub */
            {
                R_AddDrawSurf(
                    surface as *const surfaceType_t,
                    (*addr_of!(tr).projectionShadowShader),
                    0,
                    0,
                );
            }

            // don't add third_person objects if not viewing through a portal
            if !personalModel {
                #[cfg(feature = "vv_lighting")]
                {
                    let dlightBits = if (*ent).dlightBits != 0 { 1 } else { 0 };
                    R_AddDrawSurf(
                        surface as *const surfaceType_t,
                        shader,
                        fogNum,
                        dlightBits,
                    );
                }
                #[cfg(not(feature = "vv_lighting"))]
                {
                    R_AddDrawSurf(surface as *const surfaceType_t, shader, fogNum, 0);
                }
            }

            surface = ((surface as *mut u8).add((*surface).ofsEnd)) as *mut md3Surface_t;
            i += 1;
        }
    }
}

// Stub for strcmp - should be imported from libc or used as extern
extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}
