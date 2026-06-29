#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// Patches up the loaded map to handle the parameters passed from the UI

// Remap sky to contents of the cvar ar_sky
// Grab sunlight properties from the indirected sky

// void R_RemapShader(const char *shaderName, const char *newShaderName, const char *timeOffset);

extern "C" {
    pub fn R_ColorShiftLightingBytes(in_: *mut u8, out_: *mut u8);
    pub fn Cvar_VariableStringBuffer(
        var_name: *const c_char,
        buffer: *mut c_char,
        buf_size: c_int,
    );
    pub fn R_FindShader(
        name: *const c_char,
        lightmaps: c_int,
        styles: c_int,
        mipmap: c_int,
    ) -> *mut shader_t;
    pub fn Com_Clamp(min: c_int, max: c_int, value: f32) -> i32;
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> u32;
    pub fn R_WorldEffectCommand(command: *const c_char);
    pub fn atol(s: *const c_char) -> i64;

    pub static mut tr: trGlobals_t;
    pub static lightmapsNone: [c_int; 4];
    pub static stylesDefault: [u8; 4];
}

// Type stubs
#[repr(C)]
pub struct shader_t {
    // Opaque
}

#[repr(C)]
pub struct fog_t {
    // Opaque
}

#[repr(C)]
pub struct dshader_t {
    // Opaque
}

#[repr(C)]
pub struct bmodel_t {
    // Opaque
}

#[repr(C)]
pub struct cplane_t {
    // Opaque
}

#[repr(C)]
pub struct mnode_t {
    // Opaque
}

#[repr(C)]
pub struct mleaf_s {
    // Opaque
}

#[repr(C)]
pub struct msurface_t {
    // Opaque
}

// Constants
const MAX_QPATH: usize = 64;
const MAXLIGHTMAPS: usize = 4;

// RAD2DEG macro expansion: (a) * INV_PI_DIV_180
// where INV_PI_DIV_180 = 57.295779513082320876798154814105
const INV_PI_DIV_180: f64 = 57.295779513082320876798154814105;

type vec3_t = [f32; 3];
type byte = u8;

// mgrid_t struct - non-_XBOX version from tr_local.h
#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct mgrid_t {
    pub ambientLight: [[byte; 3]; MAXLIGHTMAPS],
    pub directLight: [[byte; 3]; MAXLIGHTMAPS],
    pub styles: [byte; MAXLIGHTMAPS],
    pub latLong: [byte; 2],
}

// mgrid_t struct - _XBOX version
#[cfg(target_os = "xbox")]
#[repr(C)]
pub struct mgrid_t {
    // Opaque XBOX layout
}

// world_t struct - from tr_local.h
#[repr(C)]
pub struct world_t {
    pub numShaders: c_int,
    pub shaders: *mut dshader_t,
    pub bmodels: *mut bmodel_t,
    pub numplanes: c_int,
    pub planes: *mut cplane_t,
    pub numnodes: c_int,
    pub numDecisionNodes: c_int,
    pub nodes: *mut mnode_t,
    #[cfg(target_os = "xbox")]
    pub numleafs: c_int,
    #[cfg(target_os = "xbox")]
    pub leafs: *mut mleaf_s,
    pub numsurfaces: c_int,
    pub surfaces: *mut msurface_t,
    pub nummarksurfaces: c_int,
    pub marksurfaces: *mut *mut msurface_t,
    pub numfogs: c_int,
    pub fogs: *mut fog_t,
    pub globalFog: c_int,
    pub startLightMapIndex: c_int,
    pub lightGridOrigin: vec3_t,
    pub lightGridSize: vec3_t,
    pub lightGridInverseSize: vec3_t,
    pub lightGridBounds: [c_int; 3],
    pub lightGridData: *mut mgrid_t,
    pub lightGridArray: *mut u16,
    pub numGridArrayElements: c_int,
    // ... rest omitted ...
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
    pub worldDir: [c_char; MAX_QPATH],
    // Padding/other fields omitted
    pub sunLight: vec3_t,
    pub sunDirection: vec3_t,
    pub sunAmbient: vec3_t,
    pub defaultShader: *mut shader_t,
    pub distanceCull: f32,
    // ... rest omitted ...
}

fn NormalToLatLong(normal: &vec3_t, bytes: &mut [u8; 2]) {
    // check for singularities
    if normal[0] == 0.0 && normal[1] == 0.0 {
        if normal[2] > 0.0 {
            bytes[0] = 0;
            bytes[1] = 0; // lat = 0, long = 0
        } else {
            bytes[0] = 128;
            bytes[1] = 0; // lat = 0, long = 128
        }
    } else {
        let mut a: i32;
        let mut b: i32;

        a = (((normal[1] as f64).atan2(normal[0] as f64) * INV_PI_DIV_180) * (255.0 / 360.0))
            as i32;
        a &= 0xff;

        b = (((normal[2] as f64).acos() * INV_PI_DIV_180) * (255.0 / 360.0)) as i32;
        b &= 0xff;

        bytes[0] = b as u8; // longitude
        bytes[1] = a as u8; // lattitude
    }
}

pub fn R_RMGInit() {
    let mut newSky: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut newFog: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut sky: *mut shader_t;
    let mut fog: *mut shader_t;
    let mut gfog: *mut fog_t;
    let mut grid: *mut mgrid_t;
    let mut temp: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut i: c_int;
    let mut pos: *mut u16;

    unsafe {
        Cvar_VariableStringBuffer(
            b"RMG_sky\0".as_ptr() as *const c_char,
            newSky.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        // Get sunlight - this should set up all the sunlight data
        sky = R_FindShader(
            newSky.as_ptr(),
            lightmapsNone[0],
            stylesDefault[0] as c_int,
            0, // qfalse
        );

        // Remap sky
        // R_RemapShader("textures/tools/_sky", newSky, NULL);

        // Fill in the lightgrid with sunlight
        if !tr.world.is_null() && !(*tr.world).lightGridData.is_null() {
            #[cfg(target_os = "xbox")]
            {
                let memory = (*tr.world).lightGridData as *mut u8;
                let mut array: *mut u8 = memory;
                array = array.offset(3);

                *array.offset(0) = Com_Clamp(0, 255, tr.sunAmbient[0] * 255.0) as u8;
                *array.offset(1) = Com_Clamp(0, 255, tr.sunAmbient[1] * 255.0) as u8;
                *array.offset(2) = Com_Clamp(0, 255, tr.sunAmbient[2] * 255.0) as u8;

                *array.offset(3) = Com_Clamp(0, 255, tr.sunLight[0]) as u8;
                *array.offset(4) = Com_Clamp(0, 255, tr.sunLight[1]) as u8;
                *array.offset(5) = Com_Clamp(0, 255, tr.sunLight[2]) as u8;

                NormalToLatLong(
                    &tr.sunDirection,
                    &mut (*((*tr.world).lightGridData as *mut mgrid_t)).latLong,
                );
            }

            #[cfg(not(target_os = "xbox"))]
            {
                grid = (*tr.world).lightGridData;
                (*grid).ambientLight[0][0] =
                    Com_Clamp(0, 255, tr.sunAmbient[0] * 255.0) as u8;
                (*grid).ambientLight[0][1] =
                    Com_Clamp(0, 255, tr.sunAmbient[1] * 255.0) as u8;
                (*grid).ambientLight[0][2] =
                    Com_Clamp(0, 255, tr.sunAmbient[2] * 255.0) as u8;
                R_ColorShiftLightingBytes(
                    (*grid).ambientLight[0].as_mut_ptr(),
                    (*grid).ambientLight[0].as_mut_ptr(),
                );

                (*grid).directLight[0][0] = Com_Clamp(0, 255, tr.sunLight[0]) as u8;
                (*grid).directLight[0][1] = Com_Clamp(0, 255, tr.sunLight[1]) as u8;
                (*grid).directLight[0][2] = Com_Clamp(0, 255, tr.sunLight[2]) as u8;
                R_ColorShiftLightingBytes(
                    (*grid).directLight[0].as_mut_ptr(),
                    (*grid).directLight[0].as_mut_ptr(),
                );

                NormalToLatLong(&tr.sunDirection, &mut (*grid).latLong);
            }

            pos = (*tr.world).lightGridArray;
            i = 0;
            while i < (*tr.world).numGridArrayElements {
                *pos = 0;
                pos = pos.offset(1);
                i += 1;
            }
        }

        // Override the global fog with the defined one
        if !tr.world.is_null() && (*tr.world).globalFog != -1 {
            Cvar_VariableStringBuffer(
                b"RMG_fog\0".as_ptr() as *const c_char,
                newFog.as_mut_ptr(),
                MAX_QPATH as c_int,
            );
            fog = R_FindShader(
                newFog.as_ptr(),
                lightmapsNone[0],
                stylesDefault[0] as c_int,
                0, // qfalse
            );
            if fog != tr.defaultShader {
                gfog = (*tr.world).fogs.offset((*tr.world).globalFog as isize);
                // gfog->parms = *fog->fogParms;
                // Note: We can't access fog->fogParms without the full shader_t struct definition

                // if (gfog->parms.depthForOpaque)
                // {
                //     gfog->tcScale = 1.0f / ( gfog->parms.depthForOpaque * 8.0f );
                //     tr.distanceCull = gfog->parms.depthForOpaque;
                //     Cvar_Set("RMG_distancecull", va("%f", tr.distanceCull));
                // }
                // else
                // {
                //     gfog->tcScale = 1.0f;
                // }
                // gfog->colorInt = ColorBytes4 ( gfog->parms.color[0],
                //                               gfog->parms.color[1],
                //                               gfog->parms.color[2], 1.0f );
            }
        }

        Cvar_VariableStringBuffer(
            b"RMG_weather\0".as_ptr() as *const c_char,
            temp.as_mut_ptr(),
            MAX_QPATH as c_int,
        );

        // Set up any weather effects
        match atol(temp.as_ptr()) {
            0 => {}
            1 => {
                R_WorldEffectCommand(b"rain init 1000\0".as_ptr() as *const c_char);
                R_WorldEffectCommand(b"rain outside\0".as_ptr() as *const c_char);
            }
            2 => {
                R_WorldEffectCommand(b"snow init 1000 outside\0".as_ptr() as *const c_char);
                R_WorldEffectCommand(b"snow outside\0".as_ptr() as *const c_char);
            }
            _ => {}
        }
    }
}

// end
