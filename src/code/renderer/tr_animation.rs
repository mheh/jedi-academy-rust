// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_void};
use std::mem::addr_of_mut;

// ============================================================================
// Type Stubs (minimal declarations for structural coherence)
// ============================================================================

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type byte = u8;
pub type surfaceType_t = c_void;

const MD4_MAX_BONES: usize = 128;

const CULL_OUT: c_int = 1;
const CULL_IN: c_int = 2;
const CULL_CLIP: c_int = 3;

const PRINT_ALL: c_int = 0;
const PRINT_DEVELOPER: c_int = 1;

const RF_THIRD_PERSON: c_int = 0x0001;
const RF_CAP_FRAMES: c_int = 0x0004;
const RF_WRAP_FRAMES: c_int = 0x0008;
const RF_SHADOW_PLANE: c_int = 0x0040;
const RF_NOSHADOW: c_int = 0x0080;
const RF_DEPTHHACK: c_int = 0x0100;

const RDF_NOWORLDMODEL: c_int = 0x0001;

const SS_OPAQUE: c_int = 0;

#[repr(C)]
pub struct md4Bone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct md4Frame_t {
    pub bounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub radius: f32,
    pub bones: [md4Bone_t; MD4_MAX_BONES],
}

#[repr(C)]
pub struct md4CompBone_t {
    pub Comp: [c_int; 4],
}

#[repr(C)]
pub struct md4CompFrame_t {
    pub bounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub radius: f32,
    pub bones: [md4CompBone_t; MD4_MAX_BONES],
}

#[repr(C)]
pub struct md4Weight_t {
    pub boneIndex: c_int,
    pub boneWeight: f32,
    pub offset: [f32; 3],
}

#[repr(C)]
pub struct md4Vertex_t {
    pub normal: [f32; 3],
    pub texCoords: [[f32; 2]; 1],
    pub numWeights: c_int,
    pub weights: [md4Weight_t; 1],
}

#[repr(C)]
pub struct md4Surface_t {
    pub ident: c_int,
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numCompFrames: c_int,
    pub numBoneReferences: c_int,
    pub numVerts: c_int,
    pub numTriangles: c_int,
    pub ofsTriangles: c_int,
    pub ofsSt: c_int,
    pub ofsNormals: c_int,
    pub ofsEnd: c_int,
    pub ofsVerts: c_int,
    pub ofsHeader: c_int,
    pub shaderIndex: c_int,
    pub numBones: c_int,
    pub ofsBoneReferences: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md4LOD_t {
    pub numSurfaces: c_int,
    pub ofsSurfaces: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md4Header_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numFrames: c_int,
    pub numBones: c_int,
    pub numLODs: c_int,
    pub numSurfaces: c_int,
    pub ofsFrames: c_int,
    pub ofsLODs: c_int,
    pub ofsSurfaces: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct refEntity_t {
    pub frame: c_int,
    pub oldframe: c_int,
    pub backlerp: f32,
    pub origin: [f32; 3],
    pub renderfx: c_int,
    pub nonNormalizedAxes: c_int,
    pub customShader: c_int,
    pub customSkin: c_int,
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
}

#[repr(C)]
pub struct shader_t {
    pub sort: c_int,
}

#[repr(C)]
pub struct fog_t {
    pub bounds: [[f32; 3]; 2],
}

#[repr(C)]
pub struct skin_t {
    pub numSurfaces: c_int,
}

#[repr(C)]
pub struct refdef_t {
    pub rdflags: c_int,
    pub fogIndex: c_int,
}

#[repr(C)]
pub struct glState_t {
    // stub for global state
}

#[repr(C)]
pub struct world_t {
    pub numfogs: c_int,
    pub fogs: *mut fog_t,
}

#[repr(C)]
pub struct trGlobals_t {
    pub pc: perfCounter_t,
    pub currentModel: *mut model_t,
    pub refdef: refdef_t,
    pub viewParms: viewParms_t,
    pub world: *mut world_t,
    pub defaultShader: *mut shader_t,
    pub shadowShader: *mut shader_t,
    pub projectionShadowShader: *mut shader_t,
    pub numSkins: c_int,
}

#[repr(C)]
pub struct perfCounter_t {
    pub c_sphere_cull_md3_out: c_int,
    pub c_sphere_cull_md3_in: c_int,
    pub c_sphere_cull_md3_clip: c_int,
    pub c_box_cull_md3_in: c_int,
    pub c_box_cull_md3_clip: c_int,
    pub c_box_cull_md3_out: c_int,
}

#[repr(C)]
pub struct viewParms_t {
    pub isPortal: c_int,
}

#[repr(C)]
pub struct model_t {
    pub name: [c_char; 64],
    pub md4: *mut md4Header_t,
}

#[repr(C)]
pub struct backEndState_t {
    pub currentEntity: *mut trRefEntity_t,
}

#[repr(C)]
pub struct tessVertex_t {
    // stub
}

#[repr(C)]
pub struct shaderCommands_t {
    pub indexes: [c_int; 4000],
    pub numIndexes: c_int,
    pub xyz: [[f32; 3]; 4000],
    pub numVertexes: c_int,
    pub normal: [[f32; 3]; 4000],
    pub texCoords: [[[f32; 2]; 1]; 4000],
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

// Global stubs
extern "C" {
    pub static mut tr: trGlobals_t;
    pub static mut backEnd: backEndState_t;
    pub static mut tess: shaderCommands_t;
    pub static mut r_shadows: cvar_t;
}

// ============================================================================
// External Function Declarations
// ============================================================================

extern "C" {
    pub fn R_ComputeLOD(ent: *mut trRefEntity_t) -> c_int;
    pub fn R_CullLocalPointAndRadius(point: *const vec3_t, radius: f32) -> c_int;
    pub fn R_CullLocalBox(bounds: *const [vec3_t; 2]) -> c_int;
    pub fn R_GetShaderByHandle(handle: c_int) -> *mut shader_t;
    pub fn R_GetSkinByHandle(handle: c_int) -> *mut skin_t;
    pub fn R_FogParmsMatch(fog1: c_int, fog2: c_int) -> c_int;
    pub fn VID_Printf(print_level: c_int, fmt: *const c_char, ...);
    pub fn R_SetupEntityLighting(refdef: *const refdef_t, ent: *mut trRefEntity_t);
    pub fn RB_CheckOverflow(verts: c_int, indexes: c_int);
    pub fn R_AddDrawSurf(
        surface: *mut surfaceType_t,
        shader: *mut shader_t,
        fogNum: c_int,
        dlightMap: c_int,
    );
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn MC_UnCompress(unCompressed: *mut [f32; 12], compressed: *const c_int);
    pub fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
    pub fn VectorClear(v: *mut vec3_t);
    pub fn VectorAdd(v1: *const vec3_t, v2: *const vec3_t, v3: *mut vec3_t);
}

// Conditional compilation stubs
#[cfg(feature = "VV_LIGHTING")]
pub struct VVLightManager_t {
    // stub
}

#[cfg(feature = "VV_LIGHTING")]
extern "C" {
    pub static mut VVLightMan: VVLightManager_t;
}

#[cfg(feature = "VV_LIGHTING")]
impl VVLightManager_t {
    pub fn R_SetupEntityLighting(&mut self, refdef: *const refdef_t, ent: *mut trRefEntity_t) {
        // stub
    }
}

// ============================================================================
// Functions
// ============================================================================

/*

All bones should be an identity orientation to display the mesh exactly
as it is specified.

For all other frames, the bones represent the transformation from the
orientation of the bone in the base frame to the orientation in this
frame.

*/


/*
=============
R_ACullModel
=============
*/
unsafe fn R_ACullModel(header: *mut md4Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];
    let mut oldFrame: *mut md4Frame_t;
    let mut newFrame: *mut md4Frame_t;
    let mut i: c_int;
    let mut frameSize: c_int;
    // compute frame pointers

    if (*header).ofsFrames < 0 {
        // Compressed
        frameSize = std::mem::size_of::<md4CompFrame_t>() as c_int;
        newFrame = ((header as *mut byte).offset(
            -(*header).ofsFrames as isize
            + (*ent).e.frame as isize * frameSize as isize,
        )) as *mut md4Frame_t;
        oldFrame = ((header as *mut byte).offset(
            -(*header).ofsFrames as isize
            + (*ent).e.oldframe as isize * frameSize as isize,
        )) as *mut md4Frame_t;
        // HACK! These frames actually are md4CompFrames, but the first fields are the same,
        // so this will work for this routine.
    } else {
        frameSize = std::mem::size_of::<md4Frame_t>() as c_int;
        newFrame = ((header as *mut byte)
            .offset((*header).ofsFrames as isize + (*ent).e.frame as isize * frameSize as isize))
            as *mut md4Frame_t;
        oldFrame = ((header as *mut byte)
            .offset((*header).ofsFrames as isize + (*ent).e.oldframe as isize * frameSize as isize))
            as *mut md4Frame_t;
    }

    // cull bounding sphere ONLY if this is not an upscaled entity
    if (*ent).e.nonNormalizedAxes == 0 {
        if (*ent).e.frame == (*ent).e.oldframe {
            match R_CullLocalPointAndRadius(&(*newFrame).localOrigin, (*newFrame).radius) {
                CULL_OUT => {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_out += 1;
                    return CULL_OUT;
                }

                CULL_IN => {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_in += 1;
                    return CULL_IN;
                }

                CULL_CLIP => {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_clip += 1;
                }

                _ => {}
            }
        } else {
            let mut sphereCull: c_int;
            let mut sphereCullB: c_int;

            sphereCull = R_CullLocalPointAndRadius(&(*newFrame).localOrigin, (*newFrame).radius);
            if newFrame == oldFrame {
                sphereCullB = sphereCull;
            } else {
                sphereCullB =
                    R_CullLocalPointAndRadius(&(*oldFrame).localOrigin, (*oldFrame).radius);
            }

            if sphereCull == sphereCullB {
                if sphereCull == CULL_OUT {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_out += 1;
                    return CULL_OUT;
                } else if sphereCull == CULL_IN {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_in += 1;
                    return CULL_IN;
                } else {
                    (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_sphere_cull_md3_clip += 1;
                }
            }
        }
    }

    // calculate a bounding box in the current coordinate system
    i = 0;
    while i < 3 {
        bounds[0][i as usize] = if (*oldFrame).bounds[0][i as usize]
            < (*newFrame).bounds[0][i as usize]
        {
            (*oldFrame).bounds[0][i as usize]
        } else {
            (*newFrame).bounds[0][i as usize]
        };
        bounds[1][i as usize] = if (*oldFrame).bounds[1][i as usize]
            > (*newFrame).bounds[1][i as usize]
        {
            (*oldFrame).bounds[1][i as usize]
        } else {
            (*newFrame).bounds[1][i as usize]
        };
        i += 1;
    }

    match R_CullLocalBox(&bounds) {
        CULL_IN => {
            (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_box_cull_md3_in += 1;
            CULL_IN
        }
        CULL_CLIP => {
            (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_box_cull_md3_clip += 1;
            CULL_CLIP
        }
        CULL_OUT | _ => {
            (*addr_of_mut!(tr).as_mut().unwrap()).pc.c_box_cull_md3_out += 1;
            CULL_OUT
        }
    }
}


/*
=================
R_AComputeFogNum

=================
*/
unsafe fn R_AComputeFogNum(header: *mut md4Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut i: c_int;
    let mut fog: *mut fog_t;
    let mut frame: *mut md4Frame_t;
    let mut localOrigin: vec3_t = [0.0; 3];
    let mut frameSize: c_int;

    if ((*addr_of_mut!(tr).as_ref().unwrap()).refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return 0;
    }

    if (*header).ofsFrames < 0 {
        // Compressed
        frameSize = std::mem::size_of::<md4CompFrame_t>() as c_int;
        frame = ((header as *mut byte).offset(
            -(*header).ofsFrames as isize + (*ent).e.frame as isize * frameSize as isize,
        )) as *mut md4Frame_t;
        // HACK! These frames actually are md4CompFrames, but the first fields are the same,
        // so this will work for this routine.
    } else {
        frameSize = std::mem::size_of::<md4Frame_t>() as c_int;
        frame = ((header as *mut byte)
            .offset((*header).ofsFrames as isize + (*ent).e.frame as isize * frameSize as isize))
            as *mut md4Frame_t;
    }

    VectorAdd(&(*ent).e.origin, &(*frame).localOrigin, &mut localOrigin);
    let mut partialFog: c_int = 0;
    i = 1;
    while i < (*addr_of_mut!(tr).as_ref().unwrap()).world.as_ref().unwrap().numfogs {
        fog = &mut (*(*addr_of_mut!(tr).as_ref().unwrap()).world.as_ref().unwrap().fogs)
            .add(i as usize);
        if localOrigin[0] - (*frame).radius >= (*fog).bounds[0][0]
            && localOrigin[0] + (*frame).radius <= (*fog).bounds[1][0]
            && localOrigin[1] - (*frame).radius >= (*fog).bounds[0][1]
            && localOrigin[1] + (*frame).radius <= (*fog).bounds[1][1]
            && localOrigin[2] - (*frame).radius >= (*fog).bounds[0][2]
            && localOrigin[2] + (*frame).radius <= (*fog).bounds[1][2]
        {
            //totally inside it
            return i;
        }
        if ((localOrigin[0] - (*frame).radius >= (*fog).bounds[0][0]
            && localOrigin[1] - (*frame).radius >= (*fog).bounds[0][1]
            && localOrigin[2] - (*frame).radius >= (*fog).bounds[0][2]
            && localOrigin[0] - (*frame).radius <= (*fog).bounds[1][0]
            && localOrigin[1] - (*frame).radius <= (*fog).bounds[1][1]
            && localOrigin[2] - (*frame).radius <= (*fog).bounds[1][2])
            || (localOrigin[0] + (*frame).radius >= (*fog).bounds[0][0]
                && localOrigin[1] + (*frame).radius >= (*fog).bounds[0][1]
                && localOrigin[2] + (*frame).radius >= (*fog).bounds[0][2]
                && localOrigin[0] + (*frame).radius <= (*fog).bounds[1][0]
                && localOrigin[1] + (*frame).radius <= (*fog).bounds[1][1]
                && localOrigin[2] + (*frame).radius <= (*fog).bounds[1][2]))
        {
            //partially inside it
            if (*addr_of_mut!(tr).as_ref().unwrap()).refdef.fogIndex == i
                || R_FogParmsMatch((*addr_of_mut!(tr).as_ref().unwrap()).refdef.fogIndex, i) != 0
            {
                //take new one only if it's the same one that the viewpoint is in
                return i;
            } else if partialFog == 0 {
                //first partialFog
                partialFog = i;
            }
        }
        i += 1;
    }
    //if all else fails, return the first partialFog
    partialFog
}

/*
==============
R_AddAnimSurfaces
==============
*/
pub unsafe fn R_AddAnimSurfaces(ent: *mut trRefEntity_t) {
    let mut header: *mut md4Header_t;
    let mut surface: *mut md4Surface_t;
    let mut lod: *mut md4LOD_t;
    let mut shader: *mut shader_t = std::ptr::null_mut();
    let mut cust_shader: *mut shader_t = std::ptr::null_mut();
    let mut fogNum: c_int = 0;
    let mut personalModel: qboolean;
    let mut cull: c_int;
    let mut i: c_int;
    let mut whichLod: c_int;

    // don't add third_person objects if not in a portal
    personalModel = if ((*ent).e.renderfx & RF_THIRD_PERSON) != 0
        && (*addr_of_mut!(tr).as_ref().unwrap()).viewParms.isPortal == 0
    {
        1
    } else {
        0
    };

    if ((*ent).e.renderfx & RF_CAP_FRAMES) != 0 {
        if (*ent).e.frame > (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames - 1
        {
            (*ent).e.frame =
                (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames - 1;
        }
        if (*ent).e.oldframe > (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames - 1
        {
            (*ent).e.oldframe =
                (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames - 1;
        }
    } else if ((*ent).e.renderfx & RF_WRAP_FRAMES) != 0 {
        (*ent).e.frame %= (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames;
        (*ent).e.oldframe %= (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames;
    }

    //
    // Validate the frames so there is no chance of a crash.
    // This will write directly into the entity structure, so
    // when the surfaces are rendered, they don't need to be
    // range checked again.
    //
    if ((*ent).e.frame >= (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames)
        || ((*ent).e.frame < 0)
        || ((*ent).e.oldframe
            >= (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4.as_ref().unwrap().numFrames)
        || ((*ent).e.oldframe < 0)
    {
        #[cfg(debug_assertions)]
        {
            VID_Printf(
                PRINT_ALL,
                b"R_AddAnimSurfaces: no such frame %d to %d for '%s'\n\0".as_ptr() as *const c_char,
                (*ent).e.oldframe,
                (*ent).e.frame,
                (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().name.as_ptr(),
            );
        }
        #[cfg(not(debug_assertions))]
        {
            VID_Printf(
                PRINT_DEVELOPER,
                b"R_AddAnimSurfaces: no such frame %d to %d for '%s'\n\0".as_ptr() as *const c_char,
                (*ent).e.oldframe,
                (*ent).e.frame,
                (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().name.as_ptr(),
            );
        }
        (*ent).e.frame = 0;
        (*ent).e.oldframe = 0;
    }

    header = (*addr_of_mut!(tr).as_ref().unwrap()).currentModel.as_ref().unwrap().md4;

    //
    // cull the entire model if merged bounding box of both frames
    // is outside the view frustum.
    //
    cull = R_ACullModel(header, ent);
    if cull == CULL_OUT {
        return;
    }

    //
    // compute LOD
    //
    lod = (header as *mut byte).offset((*header).ofsLODs as isize) as *mut md4LOD_t;
    whichLod = R_ComputeLOD(ent);
    i = 0;
    while i < whichLod {
        lod = (lod as *mut byte).offset((*lod).ofsEnd as isize) as *mut md4LOD_t;
        i += 1;
    }

    //
    // set up lighting now that we know we aren't culled
    //
    if personalModel == 0 || (*addr_of_mut!(r_shadows).as_ref().unwrap()).integer > 1 {
        #[cfg(feature = "VV_LIGHTING")]
        {
            (*addr_of_mut!(VVLightMan).as_mut().unwrap())
                .R_SetupEntityLighting(&(*addr_of_mut!(tr).as_ref().unwrap()).refdef, ent);
        }
        #[cfg(not(feature = "VV_LIGHTING"))]
        {
            R_SetupEntityLighting(
                &(*addr_of_mut!(tr).as_ref().unwrap()).refdef,
                ent,
            );
        }
    }

    //
    // see if we are in a fog volume
    //
    fogNum = R_AComputeFogNum(header, ent);


    //
    // draw all surfaces
    //
    cust_shader = R_GetShaderByHandle((*ent).e.customShader);


    surface = (lod as *mut byte).offset((*lod).ofsSurfaces as isize) as *mut md4Surface_t;
    i = 0;
    while i < (*lod).numSurfaces {
        if (*ent).e.customShader != 0 {
            shader = cust_shader;
        } else if (*ent).e.customSkin > 0
            && (*ent).e.customSkin < (*addr_of_mut!(tr).as_ref().unwrap()).numSkins
        {
            let mut skin: *mut skin_t;
            let mut j: c_int;

            skin = R_GetSkinByHandle((*ent).e.customSkin);

            // match the surface name to something in the skin file
            shader = (*addr_of_mut!(tr).as_ref().unwrap()).defaultShader;
            j = 0;
            while j < (*skin).numSurfaces {
                // the names have both been lowercased
                if strcmp(
                    (*((*skin).as_ref() as *const _ as *const u8).add(std::mem::offset_of!(skin_t, numSurfaces) + std::mem::size_of::<c_int>()).add(j as usize * std::mem::size_of::<*const u8>()) as *const *const c_char).as_ref().unwrap()).as_ptr(),
                    (*surface).name.as_ptr(),
                ) == 0
                {
                    // stub - would need actual skin surface structure
                    // shader = skin->surfaces[j]->shader;
                    break;
                }
                j += 1;
            }
        } else {
            shader = R_GetShaderByHandle((*surface).shaderIndex);
        }
        // we will add shadows even if the main object isn't visible in the view

        // stencil shadows can't do personal models unless I polyhedron clip
        if personalModel == 0
            && (*addr_of_mut!(r_shadows).as_ref().unwrap()).integer == 2
            #[cfg(not(feature = "VV_LIGHTING"))]
            && fogNum == 0
            && ((*ent).e.renderfx & RF_SHADOW_PLANE) != 0
            && ((*ent).e.renderfx & (RF_NOSHADOW | RF_DEPTHHACK)) == 0
            && (*shader).sort == SS_OPAQUE
        {
            R_AddDrawSurf(
                surface as *mut surfaceType_t,
                (*addr_of_mut!(tr).as_ref().unwrap()).shadowShader,
                0,
                0,
            );
        }

        // projection shadows work fine with personal models
        if (*addr_of_mut!(r_shadows).as_ref().unwrap()).integer == 3
            && fogNum == 0
            && ((*ent).e.renderfx & RF_SHADOW_PLANE) != 0
            && (*shader).sort == SS_OPAQUE
        {
            R_AddDrawSurf(
                surface as *mut surfaceType_t,
                (*addr_of_mut!(tr).as_ref().unwrap()).projectionShadowShader,
                0,
                0,
            );
        }

        // don't add third_person objects if not viewing through a portal
        if personalModel == 0 {
            R_AddDrawSurf(surface as *mut surfaceType_t, shader, fogNum, 0);
        }

        surface = (surface as *mut byte).offset((*surface).ofsEnd as isize) as *mut md4Surface_t;
        i += 1;
    }
}


/*
==============
RB_SurfaceAnim
==============
*/
pub unsafe fn RB_SurfaceAnim(surface: *mut md4Surface_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut frontlerp: f32;
    let mut backlerp: f32;
    let mut triangles: *mut c_int;
    let mut indexes: c_int;
    let mut baseIndex: c_int;
    let mut baseVertex: c_int;
    let mut numVerts: c_int;
    let mut v: *mut md4Vertex_t;
    let mut bones: [md4Bone_t; MD4_MAX_BONES] = [md4Bone_t {
        matrix: [[0.0; 4]; 3],
    }; MD4_MAX_BONES];
    let mut tbone: [md4Bone_t; 2] = [md4Bone_t {
        matrix: [[0.0; 4]; 3],
    }; 2];
    let mut bonePtr: *mut md4Bone_t;
    let mut bone: *mut md4Bone_t;
    let mut header: *mut md4Header_t;
    let mut frame: *mut md4Frame_t = std::ptr::null_mut();
    let mut oldFrame: *mut md4Frame_t = std::ptr::null_mut();
    let mut cframe: *mut md4CompFrame_t = std::ptr::null_mut();
    let mut coldFrame: *mut md4CompFrame_t = std::ptr::null_mut();
    let mut frameSize: c_int;
    let mut compressed: qboolean;


    if (*addr_of_mut!(backEnd).as_ref().unwrap()).currentEntity.as_ref().unwrap().e.oldframe
        == (*addr_of_mut!(backEnd).as_ref().unwrap()).currentEntity.as_ref().unwrap().e.frame
    {
        backlerp = 0.0;
        frontlerp = 1.0;
    } else {
        backlerp = (*addr_of_mut!(backEnd).as_ref().unwrap())
            .currentEntity.as_ref().unwrap()
            .e.backlerp;
        frontlerp = 1.0 - backlerp;
    }
    header = (surface as *mut byte).offset((*surface).ofsHeader as isize) as *mut md4Header_t;

    if (*header).ofsFrames < 0 {
        // Compressed
        compressed = 1;
        frameSize = std::mem::size_of::<md4CompFrame_t>() as c_int;
        cframe = ((header as *mut byte).offset(
            -(*header).ofsFrames as isize
                + (*addr_of_mut!(backEnd).as_ref().unwrap())
                    .currentEntity.as_ref().unwrap()
                    .e.frame as isize
                    * frameSize as isize,
        )) as *mut md4CompFrame_t;
        coldFrame = ((header as *mut byte).offset(
            -(*header).ofsFrames as isize
                + (*addr_of_mut!(backEnd).as_ref().unwrap())
                    .currentEntity.as_ref().unwrap()
                    .e.oldframe as isize
                    * frameSize as isize,
        )) as *mut md4CompFrame_t;
    } else {
        compressed = 0;
        frameSize = std::mem::size_of::<md4Frame_t>() as c_int;
        frame = ((header as *mut byte)
            .offset(
                (*header).ofsFrames as isize
                    + (*addr_of_mut!(backEnd).as_ref().unwrap())
                        .currentEntity.as_ref().unwrap()
                        .e.frame as isize
                        * frameSize as isize,
            ))
            as *mut md4Frame_t;
        oldFrame = ((header as *mut byte)
            .offset(
                (*header).ofsFrames as isize
                    + (*addr_of_mut!(backEnd).as_ref().unwrap())
                        .currentEntity.as_ref().unwrap()
                        .e.oldframe as isize
                        * frameSize as isize,
            ))
            as *mut md4Frame_t;
    }



    RB_CheckOverflow((*surface).numVerts, (*surface).numTriangles);

    triangles = (surface as *mut byte).offset((*surface).ofsTriangles as isize) as *mut c_int;
    indexes = (*surface).numTriangles * 3;
    baseIndex = (*addr_of_mut!(tess).as_ref().unwrap()).numIndexes;
    baseVertex = (*addr_of_mut!(tess).as_ref().unwrap()).numVertexes;
    j = 0;
    while j < indexes {
        (*addr_of_mut!(tess).as_mut().unwrap()).indexes[baseIndex as usize + j as usize] =
            baseVertex + *triangles.offset(j as isize);
        j += 1;
    }
    (*addr_of_mut!(tess).as_mut().unwrap()).numIndexes += indexes;

    //
    // lerp all the needed bones
    //
    if backlerp == 0.0 && compressed == 0 {
        // no lerping needed
        bonePtr = (*frame).bones.as_mut_ptr();
    } else {
        bonePtr = bones.as_mut_ptr();
        if compressed != 0 {
            i = 0;
            while i < (*header).numBones {
                if backlerp == 0.0 {
                    MC_UnCompress(
                        &mut (*bonePtr.offset(i as isize)).matrix[0][0] as *mut f32 as *mut [f32; 12],
                        &(*cframe).bones[i as usize].Comp as *const c_int,
                    );
                } else {
                    MC_UnCompress(
                        &mut tbone[0].matrix[0][0] as *mut f32 as *mut [f32; 12],
                        &(*cframe).bones[i as usize].Comp as *const c_int,
                    );
                    MC_UnCompress(
                        &mut tbone[1].matrix[0][0] as *mut f32 as *mut [f32; 12],
                        &(*coldFrame).bones[i as usize].Comp as *const c_int,
                    );
                    j = 0;
                    while j < 12 {
                        *((bonePtr as *mut f32).offset(i as isize * 12 + j as isize)) =
                            frontlerp * *((tbone.as_ptr() as *const f32).offset(j as isize))
                                + backlerp
                                    * *((tbone.as_ptr().offset(1) as *const f32).offset(j as isize));
                        j += 1;
                    }
                }
                i += 1;
            }
        } else {
            i = 0;
            while i < (*header).numBones * 12 {
                *(bonePtr as *mut f32).offset(i as isize) = frontlerp
                    * *((*frame).bones.as_ptr() as *const f32).offset(i as isize)
                    + backlerp
                        * *((*oldFrame).bones.as_ptr() as *const f32).offset(i as isize);
                i += 1;
            }
        }
    }

    //
    // deform the vertexes by the lerped bones
    //
    numVerts = (*surface).numVerts;
    v = (surface as *mut byte).offset((*surface).ofsVerts as isize) as *mut md4Vertex_t;
    j = 0;
    while j < numVerts {
        let mut tempVert: vec3_t = [0.0; 3];
        let mut tempNormal: vec3_t = [0.0; 3];
        let mut w: *mut md4Weight_t;

        VectorClear(&mut tempVert);
        VectorClear(&mut tempNormal);
        w = (*v).weights.as_mut_ptr();
        k = 0;
        while k < (*v).numWeights {
            bone = bonePtr.offset((*w).boneIndex as isize);

            tempVert[0] += (*w).boneWeight
                * (DotProduct(
                    &(*bone).matrix[0] as *const f32 as *const vec3_t,
                    &(*w).offset,
                ) + (*bone).matrix[0][3]);
            tempVert[1] += (*w).boneWeight
                * (DotProduct(
                    &(*bone).matrix[1] as *const f32 as *const vec3_t,
                    &(*w).offset,
                ) + (*bone).matrix[1][3]);
            tempVert[2] += (*w).boneWeight
                * (DotProduct(
                    &(*bone).matrix[2] as *const f32 as *const vec3_t,
                    &(*w).offset,
                ) + (*bone).matrix[2][3]);

            tempNormal[0] +=
                (*w).boneWeight
                    * DotProduct(
                        &(*bone).matrix[0] as *const f32 as *const vec3_t,
                        &(*v).normal,
                    );
            tempNormal[1] +=
                (*w).boneWeight
                    * DotProduct(
                        &(*bone).matrix[1] as *const f32 as *const vec3_t,
                        &(*v).normal,
                    );
            tempNormal[2] +=
                (*w).boneWeight
                    * DotProduct(
                        &(*bone).matrix[2] as *const f32 as *const vec3_t,
                        &(*v).normal,
                    );
            w = w.offset(1);
            k += 1;
        }

        (*addr_of_mut!(tess).as_mut().unwrap()).xyz[baseVertex as usize + j as usize][0] =
            tempVert[0];
        (*addr_of_mut!(tess).as_mut().unwrap()).xyz[baseVertex as usize + j as usize][1] =
            tempVert[1];
        (*addr_of_mut!(tess).as_mut().unwrap()).xyz[baseVertex as usize + j as usize][2] =
            tempVert[2];

        (*addr_of_mut!(tess).as_mut().unwrap()).normal[baseVertex as usize + j as usize][0] =
            tempNormal[0];
        (*addr_of_mut!(tess).as_mut().unwrap()).normal[baseVertex as usize + j as usize][1] =
            tempNormal[1];
        (*addr_of_mut!(tess).as_mut().unwrap()).normal[baseVertex as usize + j as usize][2] =
            tempNormal[2];

        (*addr_of_mut!(tess).as_mut().unwrap()).texCoords[baseVertex as usize + j as usize][0][0] =
            (*v).texCoords[0][0];
        (*addr_of_mut!(tess).as_mut().unwrap()).texCoords[baseVertex as usize + j as usize][0][1] =
            (*v).texCoords[0][1];

        v = &(*v).weights[(*v).numWeights as usize] as *const md4Weight_t as *mut md4Vertex_t;
        j += 1;
    }

    (*addr_of_mut!(tess).as_mut().unwrap()).numVertexes += (*surface).numVerts;
}
