#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_ushort, c_void};

// Patches up the loaded map to handle the parameters passed from the UI

// Remap sky to contents of the cvar ar_sky
// Grab sunlight properties from the indirected sky

// External C types (opaque to this module)
#[repr(C)]
pub struct shader_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct fogParms_t {
    pub color: [f32; 3],
    pub depthForOpaque: f32,
}

#[repr(C)]
pub struct fog_t {
    pub parms: fogParms_t,
    pub tcScale: f32,
    pub colorInt: u32,
}

#[repr(C)]
pub struct mgrid_t {
    pub ambientLight: [[u8; 3]; 1],
    pub directLight: [[u8; 3]; 1],
    pub latLong: [u8; 2],
}

#[repr(C)]
pub struct world_t {
    pub lightGridData: *mut c_void,
    pub lightGridArray: *mut c_ushort,
    pub numGridArrayElements: c_int,
    pub globalFog: c_int,
    pub fogs: *mut fog_t,
}

#[repr(C)]
pub struct trGlobals_t {
    // Only declare fields we actually use in this file
    pub world: *mut world_t,
    pub sunAmbient: [f32; 3],
    pub sunLight: [f32; 3],
    pub sunDirection: [f32; 3],
    pub distanceCull: f32,
    pub distanceCullSquared: f32,
    pub defaultShader: *mut shader_t,
}

const MAX_QPATH: usize = 256;

// External C functions
extern "C" {
    fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    fn R_FindShader(
        name: *const c_char,
        lightmaps: c_int,
        styles: c_int,
        miprap: c_int,
    ) -> *mut shader_t;
    fn R_RemapShader(oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char);
    fn Com_Clamp(min: c_int, max: c_int, value: f32) -> f32;
    fn Com_Clampi(min: c_int, max: c_int, value: f32) -> c_int;
    fn R_ColorShiftLightingBytes(in_: *mut u8, out: *mut u8);
    fn NormalToLatLong(normal: *const f32, latlong: *mut u8);
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn R_WorldEffectCommand(cmd: *const c_char);
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn atol(nptr: *const c_char) -> c_int;
    fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> u32;

    // Global renderer state
    static mut tr: trGlobals_t;

    // Convenience constants for R_FindShader
    static lightmapsNone: c_int;
    static stylesDefault: c_int;
}

#[no_mangle]
pub unsafe extern "C" fn R_RMGInit() {
    let mut newSky: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut newFog: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut sky: *mut shader_t;
    let mut fog: *mut shader_t;
    let mut gfog: *mut fog_t;
    let mut grid: *mut mgrid_t;
    let mut temp: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut i: c_int;
    let mut pos: *mut c_ushort;

    Cvar_VariableStringBuffer(
        b"RMG_sky\0".as_ptr() as *const c_char,
        newSky.as_mut_ptr(),
        MAX_QPATH as c_int,
    );
    // Get sunlight - this should set up all the sunlight data
    sky = R_FindShader(newSky.as_ptr(), lightmapsNone, stylesDefault, 0);

    // Remap sky
    R_RemapShader(
        b"textures/tools/_sky\0".as_ptr() as *const c_char,
        newSky.as_ptr(),
        core::ptr::null(),
    );

    // Fill in the lightgrid with sunlight
    if !(*tr.world).lightGridData.is_null() {
        #[cfg(target_os = "xbox")]
        {
            let memory: *mut u8 = (*tr.world).lightGridData as *mut u8;
            let array: *mut u8 = memory;
            let memory: *mut u8 = memory.add(3);

            *array.add(0) = Com_Clamp(0, 255, tr.sunAmbient[0] * 255.0f32) as u8;
            *array.add(1) = Com_Clamp(0, 255, tr.sunAmbient[1] * 255.0f32) as u8;
            *array.add(2) = Com_Clamp(0, 255, tr.sunAmbient[2] * 255.0f32) as u8;

            *array.add(3) = Com_Clamp(0, 255, tr.sunLight[0]) as u8;
            *array.add(4) = Com_Clamp(0, 255, tr.sunLight[1]) as u8;
            *array.add(5) = Com_Clamp(0, 255, tr.sunLight[2]) as u8;

            grid = (*tr.world).lightGridData as *mut mgrid_t;
            NormalToLatLong(tr.sunDirection.as_ptr(), (*grid).latLong.as_mut_ptr());
        }
        #[cfg(not(target_os = "xbox"))]
        {
            grid = (*tr.world).lightGridData as *mut mgrid_t;
            (*grid).ambientLight[0][0] = Com_Clampi(0, 255, tr.sunAmbient[0] * 255.0f32) as u8;
            (*grid).ambientLight[0][1] = Com_Clampi(0, 255, tr.sunAmbient[1] * 255.0f32) as u8;
            (*grid).ambientLight[0][2] = Com_Clampi(0, 255, tr.sunAmbient[2] * 255.0f32) as u8;
            R_ColorShiftLightingBytes(
                (*grid).ambientLight[0].as_mut_ptr(),
                (*grid).ambientLight[0].as_mut_ptr(),
            );

            (*grid).directLight[0][0] = Com_Clampi(0, 255, tr.sunLight[0]) as u8;
            (*grid).directLight[0][1] = Com_Clampi(0, 255, tr.sunLight[1]) as u8;
            (*grid).directLight[0][2] = Com_Clampi(0, 255, tr.sunLight[2]) as u8;
            R_ColorShiftLightingBytes(
                (*grid).directLight[0].as_mut_ptr(),
                (*grid).directLight[0].as_mut_ptr(),
            );

            NormalToLatLong(tr.sunDirection.as_ptr(), (*grid).latLong.as_mut_ptr());
        }

        pos = (*tr.world).lightGridArray;
        i = 0;
        while i < (*tr.world).numGridArrayElements {
            *pos = 0;
            pos = pos.add(1);
            i += 1;
        }
    }

    // Override the global fog with the defined one
    if (*tr.world).globalFog != -1 {
        Cvar_VariableStringBuffer(
            b"RMG_fog\0".as_ptr() as *const c_char,
            newFog.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        fog = R_FindShader(newFog.as_ptr(), lightmapsNone, stylesDefault, 0);
        if fog != tr.defaultShader {
            gfog = (*tr.world).fogs.add((*tr.world).globalFog as usize);
            (*gfog).parms = *(*fog).parms;
            if (*gfog).parms.depthForOpaque != 0.0 {
                (*gfog).tcScale = 1.0f32 / ((*gfog).parms.depthForOpaque * 8.0f32);
                tr.distanceCull = (*gfog).parms.depthForOpaque;
                tr.distanceCullSquared = tr.distanceCull * tr.distanceCull;
                Cvar_Set(
                    b"RMG_distancecull\0".as_ptr() as *const c_char,
                    va(b"%f\0".as_ptr() as *const c_char, tr.distanceCull),
                );
            } else {
                (*gfog).tcScale = 1.0f32;
            }
            (*gfog).colorInt = ColorBytes4(
                (*gfog).parms.color[0],
                (*gfog).parms.color[1],
                (*gfog).parms.color[2],
                1.0f32,
            );
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

// end
