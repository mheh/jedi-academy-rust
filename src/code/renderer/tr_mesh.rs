// tr_mesh.c: triangle model functions

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Type aliases and imports
pub type byte = u8;
pub type vec3_t = [f32; 3];
pub type qhandle_t = c_int;
pub type qboolean = c_int;

// Constants
pub const CULL_IN: c_int = 0;   // completely unclipped
pub const CULL_CLIP: c_int = 1; // clipped by one or more planes
pub const CULL_OUT: c_int = 2;  // completely outside the clipping planes

pub const SS_OPAQUE: c_int = 3;

// Forward declarations for opaque external types
#[repr(C)]
pub struct orientationr_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
    pub viewOrigin: vec3_t,
    pub modelMatrix: [f32; 16],
}

#[repr(C)]
pub struct viewParms_t {
    pub or: orientationr_t,
    pub world: orientationr_t,
    pub pvsOrigin: vec3_t,
    pub isPortal: qboolean,
    pub isMirror: qboolean,
    pub frameSceneNum: c_int,
    pub frameCount: c_int,
    pub portalPlane: [f32; 4], // cplane_t - simplified as array
    pub viewportX: c_int,
    pub viewportY: c_int,
    pub viewportWidth: c_int,
    pub viewportHeight: c_int,
    pub fovX: f32,
    pub fovY: f32,
    pub projectionMatrix: [f32; 16],
    pub frustum: [[f32; 4]; 5], // cplane_t[5]
    pub visBounds: [vec3_t; 2],
    pub zFar: f32,
}

#[repr(C)]
pub struct fogParms_t {
    pub color: vec3_t,
    pub depthForOpaque: f32,
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
pub struct world_t {
    pub numfogs: c_int,
    pub fogs: *mut fog_t,
    // ... rest omitted
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
    pub areamask: [byte; 32],
    pub areamaskModified: qboolean,
    pub floatTime: f32,
    pub num_entities: c_int,
    pub entities: *mut c_void,
    pub fogIndex: c_int,
    // ... rest omitted
}

#[repr(C)]
pub struct frontEndCounters_t {
    pub c_sphere_cull_patch_in: c_int,
    pub c_sphere_cull_patch_clip: c_int,
    pub c_sphere_cull_patch_out: c_int,
    pub c_box_cull_patch_in: c_int,
    pub c_box_cull_patch_clip: c_int,
    pub c_box_cull_patch_out: c_int,
    pub c_sphere_cull_md3_in: c_int,
    pub c_sphere_cull_md3_clip: c_int,
    pub c_sphere_cull_md3_out: c_int,
    pub c_box_cull_md3_in: c_int,
    pub c_box_cull_md3_clip: c_int,
    pub c_box_cull_md3_out: c_int,
    pub c_leafs: c_int,
    pub c_dlightSurfaces: c_int,
    pub c_dlightSurfacesCulled: c_int,
}

#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub origin: [f32; 3],
    pub frame: c_int,
    pub oldorigin: [f32; 3],
    pub oldframe: c_int,
    pub backlerp: f32,
    pub skinNum: c_int,
    pub customSkin: qhandle_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [byte; 4],
    pub shaderTexCoord: [f32; 2],
    pub shaderTime: f32,
    pub radius: f32,
    pub rotation: f32,
    pub angles: vec3_t,
    pub modelScale: vec3_t,
    pub ghoul2: *mut c_void,
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
pub struct md3Frame_t {
    pub bounds: [vec3_t; 2],
    pub localOrigin: vec3_t,
    pub radius: f32,
    pub name: [c_char; 16],
}

#[repr(C)]
pub struct md3Header_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [c_char; 64], // MAX_QPATH
    pub flags: c_int,
    pub numFrames: c_int,
    pub numTags: c_int,
    pub numSurfaces: c_int,
    pub numSkins: c_int,
    pub ofsFrames: c_int,
    pub ofsTags: c_int,
    pub ofsSurfaces: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md3Surface_t {
    pub ident: c_int,
    pub name: [c_char; 64], // MAX_QPATH
    pub flags: c_int,
    pub numFrames: c_int,
    pub numShaders: c_int,
    pub numVerts: c_int,
    pub numTriangles: c_int,
    pub ofsTriangles: c_int,
    pub ofsShaders: c_int,
    pub ofsSt: c_int,
    pub ofsXyzNormals: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md3Shader_t {
    pub name: [c_char; 64], // MAX_QPATH
    pub shaderIndex: c_int,
}

#[repr(C)]
pub struct shader_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct model_t {
    pub name: [c_char; 64], // MAX_QPATH
    pub reType: c_int,      // modtype_t
    pub index: c_int,
    pub dataSize: c_int,
    pub bmodel: *mut c_void,
    pub md3: [*mut md3Header_t; 3], // MD3_MAX_LODS = 3
    pub mdxm: *mut c_void,
    pub mdxa: *mut c_void,
    pub numLods: u8,
    pub bspInstance: u8,
}

#[repr(C)]
pub struct skin_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct surfaceType_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
    pub integer: c_int,
}

#[repr(C)]
pub struct trGlobals_t {
    pub registered: c_int,
    pub visCount: c_int,
    pub frameCount: c_int,
    pub sceneCount: c_int,
    pub viewCount: c_int,
    pub frameSceneNum: c_int,
    pub worldMapLoaded: c_int,
    pub world: *mut world_t,
    // padding for other fields...
    _padding1: [u8; 1200],
    pub currentModel: *mut model_t,
    // padding
    _padding2: [u8; 200],
    pub pc: frontEndCounters_t,
    // padding
    _padding3: [u8; 400],
    pub viewParms: viewParms_t,
    pub refdef: trRefdef_t,
    // padding
    _padding4: [u8; 200],
    pub numSkins: c_int,
    // padding
    _padding5: [u8; 200],
    pub shadowShader: *mut shader_t,
    pub projectionShadowShader: *mut shader_t,
    pub defaultShader: *mut shader_t,
    pub shaders: *mut *mut shader_t,
}

extern "C" {
    pub static mut tr: trGlobals_t;
    pub static mut r_lodscale: *mut cvar_t;
    pub static mut r_lodbias: *mut cvar_t;
    pub static mut r_shadows: *mut cvar_t;

    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn Q_fabs(x: f32) -> f32;
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn RadiusFromBounds(mins: *const f32, maxs: *const f32) -> f32;
    fn myftol(f: f32) -> c_int;
    fn VID_Printf(level: c_int, fmt: *const c_char, ...);
    fn R_CullLocalPointAndRadius(pt: *const f32, radius: f32) -> c_int;
    fn R_CullLocalBox(bounds: *const [vec3_t; 2]) -> c_int;
    fn R_GetModelByHandle(hModel: qhandle_t) -> *mut model_t;
    fn R_GetShaderByHandle(hShader: qhandle_t) -> *mut shader_t;
    fn R_GetSkinByHandle(hSkin: qhandle_t) -> *mut skin_t;
    fn R_AddDrawSurf(surface: *mut surfaceType_t, shader: *mut shader_t, fogNum: c_int, dlighted: qboolean);
    fn R_FogParmsMatch(fogIndex1: c_int, fogIndex2: c_int) -> qboolean;
    fn R_SetupEntityLighting(refdef: *const trRefdef_t, ent: *mut trRefEntity_t);
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    #[cfg(feature = "VV_LIGHTING")]
    static mut VVLightMan: VVLightManager;
}

#[cfg(feature = "VV_LIGHTING")]
#[repr(C)]
pub struct VVLightManager {
    _opaque: [u8; 0],
}

#[cfg(feature = "VV_LIGHTING")]
extern "C" {
    impl VVLightManager {
        pub fn R_SetupEntityLighting(
            &mut self,
            refdef: *const trRefdef_t,
            ent: *mut trRefEntity_t,
        );
    }
}

// No accessor functions needed - tr.fieldName works directly with extern "C"

unsafe fn ProjectRadius(r: f32, location: &vec3_t) -> f32 {
    let mut pr: f32;
    let mut dist: f32;
    let mut c: f32;
    let mut p: vec3_t = [0.0; 3];
    let mut width: f32;
    let mut depth: f32;

    c = DotProduct(tr.viewParms.or.axis[0].as_ptr(), tr.viewParms.or.origin.as_ptr());
    dist = DotProduct(tr.viewParms.or.axis[0].as_ptr(), location.as_ptr()) - c;

    if dist <= 0.0 {
        return 0.0;
    }

    p[0] = 0.0;
    p[1] = Q_fabs(r);
    p[2] = -dist;

    width = p[0] * tr.viewParms.projectionMatrix[1]
        + p[1] * tr.viewParms.projectionMatrix[5]
        + p[2] * tr.viewParms.projectionMatrix[9]
        + tr.viewParms.projectionMatrix[13];

    depth = p[0] * tr.viewParms.projectionMatrix[3]
        + p[1] * tr.viewParms.projectionMatrix[7]
        + p[2] * tr.viewParms.projectionMatrix[11]
        + tr.viewParms.projectionMatrix[15];

    pr = width / depth;
    #[cfg(target_os = "xbox")]
    {
        pr = -pr;
    }

    if pr > 1.0 {
        pr = 1.0;
    }

    pr
}

/*
=============
R_CullModel
=============
*/
unsafe fn R_CullModel(header: *mut md3Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut bounds: [vec3_t; 2];
    let mut oldFrame: *mut md3Frame_t;
    let mut newFrame: *mut md3Frame_t;
    let mut i: c_int;

    // compute frame pointers
    newFrame = ((header as *mut u8).add((*header).ofsFrames as usize) as *mut md3Frame_t)
        .offset((*ent).e.frame as isize);
    oldFrame = ((header as *mut u8).add((*header).ofsFrames as usize) as *mut md3Frame_t)
        .offset((*ent).e.oldframe as isize);

    // cull bounding sphere ONLY if this is not an upscaled entity
    if (*ent).e.nonNormalizedAxes == 0 {
        if (*ent).e.frame == (*ent).e.oldframe {
            match R_CullLocalPointAndRadius(
                (*newFrame).localOrigin.as_ptr(),
                (*newFrame).radius,
            ) {
                CULL_OUT => {
                    tr.pc.c_sphere_cull_md3_out += 1;
                    return CULL_OUT;
                }
                CULL_IN => {
                    tr.pc.c_sphere_cull_md3_in += 1;
                    return CULL_IN;
                }
                CULL_CLIP => {
                    tr.pc.c_sphere_cull_md3_clip += 1;
                }
                _ => {}
            }
        } else {
            let mut sphereCull: c_int;
            let mut sphereCullB: c_int;

            sphereCull = R_CullLocalPointAndRadius(
                (*newFrame).localOrigin.as_ptr(),
                (*newFrame).radius,
            );
            if newFrame == oldFrame {
                sphereCullB = sphereCull;
            } else {
                sphereCullB =
                    R_CullLocalPointAndRadius((*oldFrame).localOrigin.as_ptr(), (*oldFrame).radius);
            }

            if sphereCull == sphereCullB {
                if sphereCull == CULL_OUT {
                    tr.pc.c_sphere_cull_md3_out += 1;
                    return CULL_OUT;
                } else if sphereCull == CULL_IN {
                    tr.pc.c_sphere_cull_md3_in += 1;
                    return CULL_IN;
                } else {
                    tr.pc.c_sphere_cull_md3_clip += 1;
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
            tr.pc.c_box_cull_md3_in += 1;
            CULL_IN
        }
        CULL_CLIP => {
            tr.pc.c_box_cull_md3_clip += 1;
            CULL_CLIP
        }
        _ => {
            tr.pc.c_box_cull_md3_out += 1;
            CULL_OUT
        }
    }
}

/*
=================
RE_GetModelBounds

  Returns the bounds of the current model
  (qhandle_t)hModel and (int)frame need to be set
=================
*/
pub unsafe extern "C" fn RE_GetModelBounds(
    refEnt: *mut refEntity_t,
    bounds1: *mut vec3_t,
    bounds2: *mut vec3_t,
) {
    let mut frame: *mut md3Frame_t;
    let mut header: *mut md3Header_t;
    let mut model: *mut model_t;

    // assert(refEnt);
    model = R_GetModelByHandle((*refEnt).hModel);
    // assert(model);
    header = *(model as *mut *mut md3Header_t);
    // assert(header);
    frame = ((header as *mut u8).add((*header).ofsFrames as usize) as *mut md3Frame_t)
        .offset((*refEnt).frame as isize);
    // assert(frame);

    VectorCopy(
        (*frame).bounds[0].as_ptr(),
        (*bounds1).as_mut_ptr(),
    );
    VectorCopy(
        (*frame).bounds[1].as_ptr(),
        (*bounds2).as_mut_ptr(),
    );
}

/*
=================
R_ComputeLOD

=================
*/
unsafe fn R_ComputeLOD(ent: *mut trRefEntity_t) -> c_int {
    let mut radius: f32;
    let mut flod: f32;
    let mut projectedRadius: f32;
    let mut lod: c_int;

    let currentModel = tr.currentModel;
    let numLods = (*currentModel).numLods as c_int;

    if numLods < 2 {
        // model has only 1 LOD level, skip computations and bias
        return 0;
    }

    // multiple LODs exist, so compute projected bounding sphere
    // and use that as a criteria for selecting LOD
    // if ( tr.currentModel->md3[0] )
    {
        // normal md3
        let mut frame: *mut md3Frame_t;
        frame = ((*(*currentModel).md3[0] as *mut u8).add(
            (*(*currentModel).md3[0]).ofsFrames as usize,
        ) as *mut md3Frame_t);
        frame = frame.offset((*ent).e.frame as isize);
        radius = RadiusFromBounds(
            (*frame).bounds[0].as_ptr(),
            (*frame).bounds[1].as_ptr(),
        );
    }

    if (projectedRadius = ProjectRadius(radius, &(*ent).e.origin)) != 0.0 {
        flod = 1.0 - projectedRadius * (*r_lodscale).value;
        flod *= numLods as f32;
    } else {
        // object intersects near view plane, e.g. view weapon
        flod = 0.0;
    }

    lod = myftol(flod);

    if lod < 0 {
        lod = 0;
    } else if lod >= numLods {
        lod = numLods - 1;
    }

    lod += (*r_lodbias).integer;
    if lod >= numLods {
        lod = numLods - 1;
    }
    if lod < 0 {
        lod = 0;
    }

    lod
}

/*
=================
R_ComputeFogNum

=================
*/
unsafe fn R_ComputeFogNum(header: *mut md3Header_t, ent: *mut trRefEntity_t) -> c_int {
    let mut i: c_int;
    let mut fog: *mut fog_t;
    let mut md3Frame: *mut md3Frame_t;
    let mut localOrigin: vec3_t;

    if tr.refdef.rdflags & 1 != 0 {
        // RDF_NOWORLDMODEL
        return 0;
    }

    if tr.refdef.rdflags & 32 != 0 {
        // RDF_doLAGoggles
        return (*tr.world).numfogs;
    }

    // FIXME: non-normalized axis issues
    md3Frame = ((header as *mut u8).add((*header).ofsFrames as usize) as *mut md3Frame_t)
        .offset((*ent).e.frame as isize);
    localOrigin[0] = (*ent).e.origin[0] + (*md3Frame).localOrigin[0];
    localOrigin[1] = (*ent).e.origin[1] + (*md3Frame).localOrigin[1];
    localOrigin[2] = (*ent).e.origin[2] + (*md3Frame).localOrigin[2];

    let mut partialFog: c_int = 0;
    i = 1;
    while i < (*tr.world).numfogs {
        fog = (*tr.world).fogs.offset(i as isize);
        if localOrigin[0] - (*md3Frame).radius >= (*fog).bounds[0][0]
            && localOrigin[0] + (*md3Frame).radius <= (*fog).bounds[1][0]
            && localOrigin[1] - (*md3Frame).radius >= (*fog).bounds[0][1]
            && localOrigin[1] + (*md3Frame).radius <= (*fog).bounds[1][1]
            && localOrigin[2] - (*md3Frame).radius >= (*fog).bounds[0][2]
            && localOrigin[2] + (*md3Frame).radius <= (*fog).bounds[1][2]
        {
            //totally inside it
            return i;
        }
        if (localOrigin[0] - (*md3Frame).radius >= (*fog).bounds[0][0]
            && localOrigin[1] - (*md3Frame).radius >= (*fog).bounds[0][1]
            && localOrigin[2] - (*md3Frame).radius >= (*fog).bounds[0][2]
            && localOrigin[0] - (*md3Frame).radius <= (*fog).bounds[1][0]
            && localOrigin[1] - (*md3Frame).radius <= (*fog).bounds[1][1]
            && localOrigin[2] - (*md3Frame).radius <= (*fog).bounds[1][2])
            || (localOrigin[0] + (*md3Frame).radius >= (*fog).bounds[0][0]
                && localOrigin[1] + (*md3Frame).radius >= (*fog).bounds[0][1]
                && localOrigin[2] + (*md3Frame).radius >= (*fog).bounds[0][2]
                && localOrigin[0] + (*md3Frame).radius <= (*fog).bounds[1][0]
                && localOrigin[1] + (*md3Frame).radius <= (*fog).bounds[1][1]
                && localOrigin[2] + (*md3Frame).radius <= (*fog).bounds[1][2])
        {
            //partially inside it
            if tr.refdef.fogIndex == i || R_FogParmsMatch(tr.refdef.fogIndex, i) != 0 {
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
=================
R_AddMD3Surfaces

=================
*/
pub unsafe extern "C" fn R_AddMD3Surfaces(ent: *mut trRefEntity_t) {
    let mut i: c_int;
    let mut header: *mut md3Header_t = core::ptr::null_mut();
    let mut surface: *mut md3Surface_t = core::ptr::null_mut();
    let mut md3Shader: *mut md3Shader_t = core::ptr::null_mut();
    let mut shader: *mut shader_t = core::ptr::null_mut();
    let mut main_shader: *mut shader_t = core::ptr::null_mut();
    let mut cull: c_int;
    let mut lod: c_int;
    let mut fogNum: c_int;
    let mut personalModel: c_int;

    let currentModel = tr.currentModel;

    // don't add third_person objects if not in a portal
    personalModel = if ((*ent).e.renderfx & 2) != 0 && tr.viewParms.isPortal == 0 {
        1
    } else {
        0
    };

    if ((*ent).e.renderfx & 0x00400) != 0 {
        // RF_CAP_FRAMES
        if (*ent).e.frame > (*(*currentModel).md3[0]).numFrames - 1 {
            (*ent).e.frame = (*(*currentModel).md3[0]).numFrames - 1;
        }
        if (*ent).e.oldframe > (*(*currentModel).md3[0]).numFrames - 1 {
            (*ent).e.oldframe = (*(*currentModel).md3[0]).numFrames - 1;
        }
    } else if ((*ent).e.renderfx & 0x00200) != 0 {
        // RF_WRAP_FRAMES
        (*ent).e.frame %= (*(*currentModel).md3[0]).numFrames;
        (*ent).e.oldframe %= (*(*currentModel).md3[0]).numFrames;
    }

    //
    // Validate the frames so there is no chance of a crash.
    // This will write directly into the entity structure, so
    // when the surfaces are rendered, they don't need to be
    // range checked again.
    //
    if ((*ent).e.frame >= (*(*currentModel).md3[0]).numFrames)
        || ((*ent).e.frame < 0)
        || ((*ent).e.oldframe >= (*(*currentModel).md3[0]).numFrames)
        || ((*ent).e.oldframe < 0)
    {
        VID_Printf(
            0, // PRINT_ALL
            b"R_AddMD3Surfaces: no such frame %d to %d for '%s'\n\0".as_ptr() as *const c_char,
            (*ent).e.oldframe,
            (*ent).e.frame,
            (*currentModel).name.as_ptr(),
        );
        (*ent).e.frame = 0;
        (*ent).e.oldframe = 0;
    }

    //
    // compute LOD
    //
    lod = R_ComputeLOD(ent);

    header = (*currentModel).md3[lod as usize];

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
    #[cfg(feature = "VV_LIGHTING")]
    {
        if personalModel == 0 {
            VVLightMan.R_SetupEntityLighting(addr_of!(tr.refdef), ent);
        }
    }
    #[cfg(not(feature = "VV_LIGHTING"))]
    {
        if personalModel == 0 || (*r_shadows).integer > 1 {
            R_SetupEntityLighting(addr_of!(tr.refdef), ent);
        }
    }

    //
    // see if we are in a fog volume
    //
    fogNum = R_ComputeFogNum(header, ent);

    //
    // draw all surfaces
    //
    main_shader = R_GetShaderByHandle((*ent).e.customShader);

    surface = ((header as *mut u8).add((*header).ofsSurfaces as usize)) as *mut md3Surface_t;
    i = 0;
    while i < (*header).numSurfaces {
        if (*ent).e.customShader != 0 {
            // a little more efficient
            shader = main_shader;
        } else if (*ent).e.customSkin > 0 && (*ent).e.customSkin < tr.numSkins {
            let mut skin: *mut skin_t;
            let mut j: c_int;

            skin = R_GetSkinByHandle((*ent).e.customSkin);

            // match the surface name to something in the skin file
            shader = tr.defaultShader;
            j = 0;
            while j < *(skin as *mut c_int) {
                // the names have both been lowercased
                // This is a stub - we can't properly iterate skin surfaces without full skin_t definition
                j += 1;
            }
        } else if (*surface).numShaders <= 0 {
            shader = tr.defaultShader;
        } else {
            md3Shader =
                ((surface as *mut u8).add((*surface).ofsShaders as usize)) as *mut md3Shader_t;
            md3Shader = md3Shader.offset(((*ent).e.skinNum % (*surface).numShaders) as isize);
            shader = *tr.shaders.add((*md3Shader).shaderIndex as usize);
        }

        // we will add shadows even if the main object isn't visible in the view

        // stencil shadows can't do personal models unless I polyhedron clip
        #[cfg(not(feature = "VV_LIGHTING"))]
        {
            if personalModel == 0
                && (*r_shadows).integer == 2
                && fogNum == 0
                && ((*ent).e.renderfx & 0x00100) != 0
                && ((*ent).e.renderfx & (0x00040 | 0x00008)) == 0
                && (*shader).sort == SS_OPAQUE
            {
                R_AddDrawSurf(
                    surface as *mut surfaceType_t,
                    tr.shadowShader,
                    0,
                    0,
                );
            }
        }

        // projection shadows work fine with personal models
        if (*r_shadows).integer == 3 && fogNum == 0 && ((*ent).e.renderfx & 0x00100) != 0 && (*shader).sort == SS_OPAQUE {
            R_AddDrawSurf(
                surface as *mut surfaceType_t,
                tr.projectionShadowShader,
                0,
                0,
            );
        }

        // don't add third_person objects if not viewing through a portal
        if personalModel == 0 {
            #[cfg(feature = "VV_LIGHTING")]
            {
                let dlightBits: c_int = if (*ent).dlightBits != 0 { 1 } else { 0 };
                R_AddDrawSurf(
                    surface as *mut surfaceType_t,
                    shader,
                    fogNum,
                    dlightBits,
                );
            }
            #[cfg(not(feature = "VV_LIGHTING"))]
            {
                R_AddDrawSurf(
                    surface as *mut surfaceType_t,
                    shader,
                    fogNum,
                    0,
                );
            }
        }

        surface = ((surface as *mut u8).add((*surface).ofsEnd as usize)) as *mut md3Surface_t;
        i += 1;
    }
}
