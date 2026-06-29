#![allow(non_snake_case)]

// tr_light.c

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

// #include "tr_local.h"

use core::ffi::{c_int, c_void};

const DLIGHT_AT_RADIUS: c_int = 16;
// at the edge of a dlight's influence, this amount of light will be added

const DLIGHT_MINIMUM_RADIUS: c_int = 16;
// never calculate a range less than this to prevent huge light numbers

// Local type stubs for structural coherence
#[repr(C)]
pub struct dlight_t {
    pub origin: [f32; 3],
    pub transformed: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
}

#[repr(C)]
pub struct orientationr_t {
    pub origin: [f32; 3],
    pub axis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct bmodel_t {
    pub bounds: [[f32; 3]; 2],
    pub numSurfaces: c_int,
    pub firstSurface: *mut msurface_t,
}

#[repr(C)]
pub struct msurface_t {
    pub data: *mut c_void,
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    pub dlightBits: c_int,
}

#[repr(C)]
pub struct srfGridMesh_t {
    pub dlightBits: c_int,
}

#[repr(C)]
pub struct srfTriangles_t {
    pub dlightBits: c_int,
}

#[repr(C)]
pub struct refEntity_t {
    pub hModel: c_int,
    pub ghoul2: *mut c_void,
    pub renderfx: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub axis: [[f32; 3]; 3],
    pub lightingOrigin: [f32; 3],
    pub customShader: *mut c_void,
    pub shaderRGBA: [u8; 4],
    pub reType: c_int,
    pub radius: f32,
    pub rotation: f32,
    pub oldorigin: [f32; 3],
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub lightingCalculated: c_int,
    pub ambientLight: [f32; 3],
    pub directedLight: [f32; 3],
    pub lightDir: [f32; 3],
    pub ambientLightInt: c_int,
    pub needDlights: c_int,
    pub dlightBits: c_int,
}

#[repr(C)]
pub struct trRefdef_t {
    pub rdflags: c_int,
    pub num_dlights: c_int,
    pub dlights: *mut dlight_t,
}

#[repr(C)]
pub struct mgrid_t {
    pub latLong: [u8; 2],
    pub flags: u8,
    pub styles: [u8; 4],
    pub data: usize,
    pub ambientLight: [[u8; 3]; 4],
    pub directLight: [[u8; 3]; 4],
}

#[repr(C)]
pub struct worldData_t {
    pub lightGridOrigin: [f32; 3],
    pub lightGridInverseSize: [f32; 3],
    pub lightGridSize: [f32; 3],
    pub lightGridBounds: [c_int; 3],
    pub lightGridArray: *mut u16,
    pub lightGridData: *mut mgrid_t,
    pub numGridArrayElements: usize,
}

#[repr(C)]
pub struct tr_t {
    pub refdef: trRefdef_t,
    pub or: orientationr_t,
    pub currentEntity: *mut trRefEntity_t,
    pub world: *mut worldData_t,
    pub identityLight: f32,
    pub identityLightByte: u8,
    pub sunDirection: [f32; 3],
    pub sinTable: [f32; 2048],
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
    pub integer: c_int,
}

// Flags and constants
const RF_LIGHTING_ORIGIN: c_int = 0x0008;
const RF_FIRST_PERSON: c_int = 0x0001;
const RF_MORELIGHT: c_int = 0x0040;
const RF_DEPTHHACK: c_int = 0x0020;
const RDF_NOWORLDMODEL: c_int = 0x0001;
const RDF_doLAGoggles: c_int = 0x0020;
const SF_FACE: u32 = 1;
const SF_GRID: u32 = 2;
const SF_TRIANGLES: u32 = 3;
const MAXLIGHTMAPS: usize = 4;
const FUNCTABLE_SIZE: usize = 2048;
const FUNCTABLE_MASK: usize = FUNCTABLE_SIZE - 1;
const LS_NONE: u8 = 255;
const RT_MODEL: c_int = 1;
const RT_SPRITE: c_int = 2;
const RT_LINE: c_int = 4;

// External globals
extern "C" {
    pub static mut tr: tr_t;
    pub static mut styleColors: [[f32; 3]; 32];
    pub static r_ambientScale: *const cvar_t;
    pub static r_directedScale: *const cvar_t;
    pub static r_debugLight: *const cvar_t;
    pub static r_fullbright: *const cvar_t;

    // Functions
    fn VectorSubtract(a: [f32; 3], b: [f32; 3], out: *mut [f32; 3]);
    fn DotProduct(a: [f32; 3], b: [f32; 3]) -> f32;
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorCopy(src: [f32; 3], dest: *mut [f32; 3]);
    fn VectorScale(v: [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorMA(v1: [f32; 3], scale: f32, v2: [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize2(v: [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorLength(v: [f32; 3]) -> f32;
    fn VID_Printf(level: c_int, fmt: *const u8, ...);
    fn myftol(f: f32) -> c_int;
    fn vectoangles(vec: [f32; 3], angles: *mut [f32; 3]);
    fn AnglesToAxis(angles: [f32; 3], axis: *mut [[f32; 3]; 3]);
    fn RE_RegisterShader(name: *const u8) -> *mut c_void;
    fn RE_AddRefEntityToScene(ent: *mut refEntity_t);
}

// Constants for VID_Printf
const PRINT_ALL: c_int = 1;

/*
===============
R_TransformDlights

Transforms the origins of an array of dlights.
Used by both the front end (for DlightBmodel) and
the back end (before doing the lighting calculation)
===============
*/
pub unsafe fn R_TransformDlights(count: c_int, mut dl: *mut dlight_t, or_: *const orientationr_t) {
    let mut i: c_int = 0;

    while i < count {
        let mut temp: [f32; 3] = [0.0; 3];
        VectorSubtract((*dl).origin, (*or_).origin, &mut temp);
        (*dl).transformed[0] = DotProduct(temp, (*or_).axis[0]);
        (*dl).transformed[1] = DotProduct(temp, (*or_).axis[1]);
        (*dl).transformed[2] = DotProduct(temp, (*or_).axis[2]);
        dl = dl.add(1);
        i += 1;
    }
}

/*
=============
R_DlightBmodel

Determine which dynamic lights may effect this bmodel
=============
*/
#[cfg(not(feature = "VV_LIGHTING"))]
pub unsafe fn R_DlightBmodel(bmodel: *mut bmodel_t, NoLight: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut dl: *mut dlight_t;
    let mut mask: c_int;
    let mut surf: *mut msurface_t;

    // transform all the lights
    R_TransformDlights(tr.refdef.num_dlights, tr.refdef.dlights, &tr.or);

    mask = 0;
    if NoLight == 0 {
        i = 0;
        while i < tr.refdef.num_dlights {
            dl = &mut *tr.refdef.dlights.add(i as usize);

            // see if the point is close enough to the bounds to matter
            j = 0;
            while j < 3 {
                if (*dl).transformed[j as usize] - (*bmodel).bounds[1][j as usize] > (*dl).radius {
                    break;
                }
                if (*bmodel).bounds[0][j as usize] - (*dl).transformed[j as usize] > (*dl).radius {
                    break;
                }
                j += 1;
            }
            if j < 3 {
                i += 1;
                continue;
            }

            // we need to check this light
            mask |= 1 << i;
            i += 1;
        }
    }

    (*tr.currentEntity).needDlights = if mask != 0 { 1 } else { 0 };
    (*tr.currentEntity).dlightBits = mask;

    // set the dlight bits in all the surfaces
    i = 0;
    while i < (*bmodel).numSurfaces {
        surf = (*bmodel).firstSurface.add(i as usize);
        let surf_data_type = *((*surf).data as *const u32);
        if surf_data_type == SF_FACE {
            (*((*surf).data as *mut srfSurfaceFace_t)).dlightBits = mask;
        } else if surf_data_type == SF_GRID {
            (*((*surf).data as *mut srfGridMesh_t)).dlightBits = mask;
        } else if surf_data_type == SF_TRIANGLES {
            (*((*surf).data as *mut srfTriangles_t)).dlightBits = mask;
        }
        i += 1;
    }
}

/*
=============================================================================

LIGHT SAMPLING

=============================================================================
*/

/*
=================
R_SetupEntityLightingGrid

=================
*/
#[cfg(feature = "VV_LIGHTING")]
pub unsafe fn R_SetupEntityLightingGrid(ent: *mut trRefEntity_t) {
    _R_SetupEntityLightingGrid_impl(ent);
}

#[cfg(not(feature = "VV_LIGHTING"))]
pub unsafe fn R_SetupEntityLightingGrid(ent: *mut trRefEntity_t) {
    _R_SetupEntityLightingGrid_impl(ent);
}

unsafe fn _R_SetupEntityLightingGrid_impl(ent: *mut trRefEntity_t) {
    let mut lightOrigin: [f32; 3];
    let mut pos: [c_int; 3] = [0; 3];
    let mut i: usize;
    let mut j: usize;
    let mut frac: [f32; 3];
    let mut gridStep: [usize; 3];
    let mut direction: [f32; 3];
    let mut totalFactor: f32;
    let mut startGridPos: *mut u16;

    #[cfg(target_os = "xbox")]
    let zeroArray: [u8; 3] = [0, 0, 0];

    if (*r_fullbright).integer != 0 || (tr.refdef.rdflags & RDF_doLAGoggles) != 0 {
        (*ent).ambientLight[0] = 255.0;
        (*ent).ambientLight[1] = 255.0;
        (*ent).ambientLight[2] = 255.0;
        (*ent).directedLight[0] = 255.0;
        (*ent).directedLight[1] = 255.0;
        (*ent).directedLight[2] = 255.0;
        VectorCopy(tr.sunDirection, &mut (*ent).lightDir);
        return;
    }

    if (*ent).e.renderfx & RF_LIGHTING_ORIGIN != 0 {
        // seperate lightOrigins are needed so an object that is
        // sinking into the ground can still be lit, and so
        // multi-part models can be lit identically
        VectorCopy((*ent).e.lightingOrigin, &mut lightOrigin);
    } else {
        VectorCopy((*ent).e.origin, &mut lightOrigin);
    }

    // #define ACCURATE_LIGHTGRID_SAMPLING 1
    // #if ACCURATE_LIGHTGRID_SAMPLING
    let startLightOrigin: [f32; 3];
    VectorCopy(lightOrigin, &mut startLightOrigin);
    // #endif

    VectorSubtract(lightOrigin, (*tr.world).lightGridOrigin, &mut lightOrigin);
    i = 0;
    while i < 3 {
        let mut v: f32;

        v = lightOrigin[i] * (*tr.world).lightGridInverseSize[i];
        pos[i] = v.floor() as c_int;
        frac[i] = v - pos[i] as f32;
        if pos[i] < 0 {
            pos[i] = 0;
        } else if pos[i] >= (*tr.world).lightGridBounds[i] - 1 {
            pos[i] = (*tr.world).lightGridBounds[i] - 1;
        }
        i += 1;
    }

    VectorClear(&mut (*ent).ambientLight);
    VectorClear(&mut (*ent).directedLight);
    VectorClear(&mut direction);

    // trilerp the light value
    gridStep[0] = 1;
    gridStep[1] = (*tr.world).lightGridBounds[0] as usize;
    gridStep[2] = ((*tr.world).lightGridBounds[0] * (*tr.world).lightGridBounds[1]) as usize;
    startGridPos = (*tr.world).lightGridArray
        .add((pos[0] as usize * gridStep[0] + pos[1] as usize * gridStep[1] + pos[2] as usize * gridStep[2]) as usize);

    // #if ACCURATE_LIGHTGRID_SAMPLING
    let mut startGridOrg: [f32; 3];
    VectorCopy((*tr.world).lightGridOrigin, &mut startGridOrg);
    startGridOrg[0] += pos[0] as f32 * (*tr.world).lightGridSize[0];
    startGridOrg[1] += pos[1] as f32 * (*tr.world).lightGridSize[1];
    startGridOrg[2] += pos[2] as f32 * (*tr.world).lightGridSize[2];
    // #endif

    totalFactor = 0.0;
    i = 0;
    while i < 8 {
        let mut factor: f32;
        let mut data: *mut mgrid_t;
        let mut gridPos: *mut u16;
        let mut lat: u8;
        let mut lng: u8;
        let mut normal: [f32; 3];

        // #if ACCURATE_LIGHTGRID_SAMPLING
        let mut gridOrg: [f32; 3];
        VectorCopy(startGridOrg, &mut gridOrg);
        // #endif

        factor = 1.0;
        gridPos = startGridPos;
        j = 0;
        while j < 3 {
            if i & (1 << j) != 0 {
                factor *= frac[j];
                gridPos = gridPos.add(gridStep[j]);
                // #if ACCURATE_LIGHTGRID_SAMPLING
                gridOrg[j] += (*tr.world).lightGridSize[j];
                // #endif
            } else {
                factor *= 1.0 - frac[j];
            }
            j += 1;
        }

        if gridPos >= (*tr.world).lightGridArray.add((*tr.world).numGridArrayElements) {
            // we've gone off the array somehow
            i += 1;
            continue;
        }
        data = (*tr.world).lightGridData.add(*gridPos as usize);

        #[cfg(target_os = "xbox")]
        {
            let memory: *const u8 = ((*tr.world).lightGridData as *const u8).add((*data).data as usize);

            let style: u8 = if (*data).flags & (1 << 4) != 0 {
                *memory
            } else {
                LS_NONE
            };
            if style == LS_NONE {
                i += 1;
                continue; // ignore samples in walls
            }

            totalFactor += factor;

            let mut array: *const u8;
            let mut memory_ptr = memory;

            j = 0;
            while j < MAXLIGHTMAPS {
                let local_style: u8 = if ((*data).flags as usize) & (1 << (j + 4)) != 0 {
                    *memory_ptr
                } else {
                    LS_NONE
                };
                if local_style != LS_NONE {
                    memory_ptr = memory_ptr.add(1);
                }

                if local_style != LS_NONE {
                    if ((*data).flags as usize) & (1 << j) != 0 {
                        array = memory_ptr;
                        memory_ptr = memory_ptr.add(3);
                    } else {
                        array = zeroArray.as_ptr();
                    }

                    (*ent).ambientLight[0] += factor * (*array.add(0)) as f32 * styleColors[local_style as usize][0] / 255.0;
                    (*ent).ambientLight[1] += factor * (*array.add(1)) as f32 * styleColors[local_style as usize][1] / 255.0;
                    (*ent).ambientLight[2] += factor * (*array.add(2)) as f32 * styleColors[local_style as usize][2] / 255.0;

                    if array != zeroArray.as_ptr() {
                        array = memory_ptr;
                        memory_ptr = memory_ptr.add(3);
                    }

                    (*ent).directedLight[0] += factor * (*array.add(0)) as f32 * styleColors[local_style as usize][0] / 255.0;
                    (*ent).directedLight[1] += factor * (*array.add(1)) as f32 * styleColors[local_style as usize][1] / 255.0;
                    (*ent).directedLight[2] += factor * (*array.add(2)) as f32 * styleColors[local_style as usize][2] / 255.0;
                } else {
                    break;
                }
                j += 1;
            }
        }

        #[cfg(not(target_os = "xbox"))]
        {
            if (*data).styles[0] == LS_NONE {
                i += 1;
                continue; // ignore samples in walls
            }

            // if !SV_inPVS(startLightOrigin, gridOrg) {
            //     continue;
            // }

            totalFactor += factor;

            j = 0;
            while j < MAXLIGHTMAPS {
                if (*data).styles[j] != LS_NONE {
                    let style: u8 = (*data).styles[j];

                    (*ent).ambientLight[0] += factor * (*data).ambientLight[j][0] as f32 * styleColors[style as usize][0] / 255.0;
                    (*ent).ambientLight[1] += factor * (*data).ambientLight[j][1] as f32 * styleColors[style as usize][1] / 255.0;
                    (*ent).ambientLight[2] += factor * (*data).ambientLight[j][2] as f32 * styleColors[style as usize][2] / 255.0;

                    (*ent).directedLight[0] += factor * (*data).directLight[j][0] as f32 * styleColors[style as usize][0] / 255.0;
                    (*ent).directedLight[1] += factor * (*data).directLight[j][1] as f32 * styleColors[style as usize][1] / 255.0;
                    (*ent).directedLight[2] += factor * (*data).directLight[j][2] as f32 * styleColors[style as usize][2] / 255.0;
                } else {
                    break;
                }
                j += 1;
            }
        }

        lat = (*data).latLong[1];
        lng = (*data).latLong[0];
        let lat_i = (lat as usize) * (FUNCTABLE_SIZE / 256);
        let lng_i = (lng as usize) * (FUNCTABLE_SIZE / 256);

        // decode X as cos( lat ) * sin( long )
        // decode Y as sin( lat ) * sin( long )
        // decode Z as cos( long )

        normal[0] = tr.sinTable[(lat_i + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK] * tr.sinTable[lng_i];
        normal[1] = tr.sinTable[lat_i] * tr.sinTable[lng_i];
        normal[2] = tr.sinTable[(lng_i + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK];

        VectorMA(direction, factor, normal, &mut direction);

        // #if ACCURATE_LIGHTGRID_SAMPLING
        // #ifndef _XBOX
        if (*r_debugLight).integer != 0 && (*ent).e.hModel == -1 {
            // draw
            let mut refEnt: refEntity_t = core::mem::zeroed();
            refEnt.hModel = 0;
            refEnt.ghoul2 = core::ptr::null_mut();
            refEnt.renderfx = 0;
            VectorCopy(gridOrg, &mut refEnt.origin);
            vectoangles(normal, &mut refEnt.angles);
            AnglesToAxis(refEnt.angles, &mut refEnt.axis);
            refEnt.reType = RT_MODEL;
            RE_AddRefEntityToScene(&mut refEnt);

            refEnt.renderfx = RF_DEPTHHACK;
            refEnt.reType = RT_SPRITE;
            refEnt.customShader = RE_RegisterShader(b"gfx/misc/debugAmbient\0" as *const u8);
            refEnt.shaderRGBA[0] = (*data).ambientLight[0][0];
            refEnt.shaderRGBA[1] = (*data).ambientLight[0][1];
            refEnt.shaderRGBA[2] = (*data).ambientLight[0][2];
            refEnt.shaderRGBA[3] = 255;
            refEnt.radius = factor * 50.0 + 2.0; // maybe always give it a minimum size?
            refEnt.rotation = 0.0; // don't let the sprite wobble around
            RE_AddRefEntityToScene(&mut refEnt);

            refEnt.reType = RT_LINE;
            refEnt.customShader = RE_RegisterShader(b"gfx/misc/debugArrow\0" as *const u8);
            refEnt.shaderRGBA[0] = (*data).directLight[0][0];
            refEnt.shaderRGBA[1] = (*data).directLight[0][1];
            refEnt.shaderRGBA[2] = (*data).directLight[0][2];
            refEnt.shaderRGBA[3] = 255;
            VectorCopy(refEnt.origin, &mut refEnt.oldorigin);
            VectorMA(gridOrg, (factor * -255.0) - 2.0, normal, &mut refEnt.origin); // maybe always give it a minimum length
            refEnt.radius = 1.5;
            RE_AddRefEntityToScene(&mut refEnt);
        }
        // #endif // _XBOX
        // #endif

        i += 1;
    }

    if totalFactor > 0.0 && totalFactor < 0.99 {
        totalFactor = 1.0 / totalFactor;
        VectorScale((*ent).ambientLight, totalFactor, &mut (*ent).ambientLight);
        VectorScale((*ent).directedLight, totalFactor, &mut (*ent).directedLight);
    }

    VectorScale((*ent).ambientLight, (*r_ambientScale).value, &mut (*ent).ambientLight);
    VectorScale((*ent).directedLight, (*r_directedScale).value, &mut (*ent).directedLight);

    VectorNormalize2(direction, &mut (*ent).lightDir);
}

/*
===============
LogLight
===============
*/
unsafe fn LogLight(ent: *mut trRefEntity_t) {
    let mut max1: i32;
    let mut max2: i32;

    /*
    if ( !(ent->e.renderfx & RF_FIRST_PERSON ) ) {
        return;
    }
    */

    max1 = VectorLength((*ent).ambientLight) as i32;
    /*
    max1 = ent->ambientLight[0];
    if ( ent->ambientLight[1] > max1 ) {
        max1 = ent->ambientLight[1];
    } else if ( ent->ambientLight[2] > max1 ) {
        max1 = ent->ambientLight[2];
    }
    */

    max2 = VectorLength((*ent).directedLight) as i32;
    /*
    max2 = ent->directedLight[0];
    if ( ent->directedLight[1] > max2 ) {
        max2 = ent->directedLight[1];
    } else if ( ent->directedLight[2] > max2 ) {
        max2 = ent->directedLight[2];
    }
    */

    VID_Printf(
        PRINT_ALL,
        b"amb:%i  dir:%i  direction: (%4.2f, %4.2f, %4.2f)\n\0" as *const u8,
        max1,
        max2,
        (*ent).lightDir[0],
        (*ent).lightDir[1],
        (*ent).lightDir[2],
    );
}

/*
=================
R_SetupEntityLighting

Calculates all the lighting values that will be used
by the Calc_* functions
=================
*/
pub unsafe fn R_SetupEntityLighting(refdef: *const trRefdef_t, ent: *mut trRefEntity_t) {
    #[cfg(not(feature = "VV_LIGHTING"))]
    {
        let mut i: c_int;
        let mut dl: *mut dlight_t;
        let mut power: f32;
        let mut dir: [f32; 3];
        let mut d: f32;
        let mut lightDir: [f32; 3];
        let mut lightOrigin: [f32; 3];

        // lighting calculations
        if (*ent).lightingCalculated != 0 {
            return;
        }
        (*ent).lightingCalculated = 1;

        //
        // trace a sample point down to find ambient light
        //
        if (*ent).e.renderfx & RF_LIGHTING_ORIGIN != 0 {
            // seperate lightOrigins are needed so an object that is
            // sinking into the ground can still be lit, and so
            // multi-part models can be lit identically
            VectorCopy((*ent).e.lightingOrigin, &mut lightOrigin);
        } else {
            VectorCopy((*ent).e.origin, &mut lightOrigin);
        }

        // if NOWORLDMODEL, only use dynamic lights (menu system, etc)
        if ((*refdef).rdflags & RDF_NOWORLDMODEL) == 0 && !(*tr.world).lightGridData.is_null() {
            R_SetupEntityLightingGrid(ent);
        } else {
            (*ent).ambientLight[0] = tr.identityLight * 150.0;
            (*ent).ambientLight[1] = tr.identityLight * 150.0;
            (*ent).ambientLight[2] = tr.identityLight * 150.0;
            (*ent).directedLight[0] = tr.identityLight * 150.0;
            (*ent).directedLight[1] = tr.identityLight * 150.0;
            (*ent).directedLight[2] = tr.identityLight * 150.0;
            VectorCopy(tr.sunDirection, &mut (*ent).lightDir);
        }

        // bonus items and view weapons have a fixed minimum add
        if (*ent).e.renderfx & RF_MORELIGHT != 0 {
            (*ent).ambientLight[0] += tr.identityLight * 96.0;
            (*ent).ambientLight[1] += tr.identityLight * 96.0;
            (*ent).ambientLight[2] += tr.identityLight * 96.0;
        } else {
            // give everything a minimum light add
            (*ent).ambientLight[0] += tr.identityLight * 32.0;
            (*ent).ambientLight[1] += tr.identityLight * 32.0;
            (*ent).ambientLight[2] += tr.identityLight * 32.0;
        }

        //
        // modify the light by dynamic lights
        //
        d = VectorLength((*ent).directedLight);
        VectorScale((*ent).lightDir, d, &mut lightDir);

        i = 0;
        while i < (*refdef).num_dlights {
            dl = &mut *(*refdef).dlights.add(i as usize);
            VectorSubtract((*dl).origin, lightOrigin, &mut dir);
            d = VectorNormalize(&mut dir);

            power = DLIGHT_AT_RADIUS as f32 * ((*dl).radius * (*dl).radius);
            if d < DLIGHT_MINIMUM_RADIUS as f32 {
                d = DLIGHT_MINIMUM_RADIUS as f32;
            }
            d = power / (d * d);

            VectorMA((*ent).directedLight, d, (*dl).color, &mut (*ent).directedLight);
            VectorMA(lightDir, d, dir, &mut lightDir);
            i += 1;
        }

        // clamp ambient
        i = 0;
        while i < 3 {
            if (*ent).ambientLight[i as usize] > tr.identityLightByte as f32 {
                (*ent).ambientLight[i as usize] = tr.identityLightByte as f32;
            }
            i += 1;
        }

        if (*r_debugLight).integer != 0 {
            LogLight(ent);
        }

        // save out the byte packet version
        let ambient_bytes = &mut (*ent).ambientLightInt as *mut c_int as *mut u8;
        *ambient_bytes.add(0) = myftol((*ent).ambientLight[0]) as u8;
        *ambient_bytes.add(1) = myftol((*ent).ambientLight[1]) as u8;
        *ambient_bytes.add(2) = myftol((*ent).ambientLight[2]) as u8;
        *ambient_bytes.add(3) = 0xff;

        // transform the direction to local space
        VectorNormalize(&mut lightDir);
        (*ent).lightDir[0] = DotProduct(lightDir, (*ent).e.axis[0]);
        (*ent).lightDir[1] = DotProduct(lightDir, (*ent).e.axis[1]);
        (*ent).lightDir[2] = DotProduct(lightDir, (*ent).e.axis[2]);
    }
}

// pass in origin
pub unsafe fn RE_GetLighting(
    origin: [f32; 3],
    ambientLight: *mut [f32; 3],
    directedLight: *mut [f32; 3],
    lightDir: *mut [f32; 3],
) -> c_int {
    let mut tr_ent: trRefEntity_t = core::mem::zeroed();

    if tr.world.is_null() || (*tr.world).lightGridData.is_null() {
        (*ambientLight)[0] = 255.0;
        (*ambientLight)[1] = 255.0;
        (*ambientLight)[2] = 255.0;
        (*directedLight)[0] = 255.0;
        (*directedLight)[1] = 255.0;
        (*directedLight)[2] = 255.0;
        VectorCopy(tr.sunDirection, lightDir);
        return 0; // qfalse
    }

    if (*ambientLight)[0] as i32 == 666 {
        // HAX0R
        tr_ent.e.hModel = -1;
    }

    VectorCopy(origin, &mut tr_ent.e.origin);
    R_SetupEntityLightingGrid(&mut tr_ent);
    VectorCopy(tr_ent.ambientLight, ambientLight);
    VectorCopy(tr_ent.directedLight, directedLight);
    VectorCopy(tr_ent.lightDir, lightDir);
    return 1; // qtrue
}

/*
=================
R_LightForPoint
=================
*/
pub unsafe fn R_LightForPoint(
    point: [f32; 3],
    ambientLight: *mut [f32; 3],
    directedLight: *mut [f32; 3],
    lightDir: *mut [f32; 3],
) -> c_int {
    let mut ent: trRefEntity_t = core::mem::zeroed();

    // bk010103 - this segfaults with -nolight maps
    if (*tr.world).lightGridData.is_null() {
        return 0; // qfalse
    }

    VectorCopy(point, &mut ent.e.origin);
    R_SetupEntityLightingGrid(&mut ent);
    VectorCopy(ent.ambientLight, ambientLight);
    VectorCopy(ent.directedLight, directedLight);
    VectorCopy(ent.lightDir, lightDir);

    return 1; // qtrue
}
