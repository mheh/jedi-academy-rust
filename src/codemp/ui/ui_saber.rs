//
/*
=======================================================================

USER INTERFACE SABER LOADING & DISPLAY CODE

=======================================================================
*/

// leave this at the top of all UI_xxxx files for PCH reasons...
//
//#include "../server/exe_headers.h"
// #include "ui_local.h"
// #include "ui_shared.h"

use core::ffi::{c_char, c_int, c_void};

// Local type definitions (stubs for opaque types)
pub type qhandle_t = c_int;
pub type qboolean = c_int;
pub type vec3_t = [f32; 3];
pub type fileHandle_t = c_int;
pub type saber_colors_t = c_int;
pub type saberType_t = c_int;

// Constants for saber colors
pub const SABER_RED: c_int = 1;
pub const SABER_ORANGE: c_int = 2;
pub const SABER_YELLOW: c_int = 3;
pub const SABER_GREEN: c_int = 4;
pub const SABER_BLUE: c_int = 5;
pub const SABER_PURPLE: c_int = 6;

// Constants for saber types
pub const SABER_NONE: c_int = -1;
pub const SABER_SINGLE: c_int = 0;
pub const SABER_STAFF: c_int = 1;
pub const SABER_BROAD: c_int = 2;
pub const SABER_PRONG: c_int = 3;
pub const SABER_DAGGER: c_int = 4;
pub const SABER_ARC: c_int = 5;
pub const SABER_SAI: c_int = 6;
pub const SABER_CLAW: c_int = 7;
pub const SABER_LANCE: c_int = 8;
pub const SABER_STAR: c_int = 9;
pub const SABER_TRIDENT: c_int = 10;
pub const SABER_SITH_SWORD: c_int = 11;

pub const MAX_QPATH: usize = 64;
pub const MAX_MENUFILE: usize = 32000;
pub const FS_READ: c_int = 0;
pub const ERR_FATAL: c_int = 1;
pub const S_COLOR_RED: &str = "^1";
pub const RT_SABER_GLOW: c_int = 14;
pub const RT_LINE: c_int = 13;
pub const ORIGIN: c_int = 0;
pub const NEGATIVE_Y: c_int = 1;
pub const NEGATIVE_X: c_int = 2;
pub const POSITIVE_Z: c_int = 3;
pub const ITF_ISSABER: c_int = 0x00001000;
pub const ITF_ISSABER2: c_int = 0x00002000;
pub const ITF_ISCHARACTER: c_int = 0x00004000;

//#define MAX_SABER_DATA_SIZE 0x8000
const MAX_SABER_DATA_SIZE: usize = 0x80000;

// On Xbox, static linking lets us steal the buffer from wp_saberLoad
// Just make sure that the saber data size is the same
// Damn. OK. Gotta fix this again. Later.
pub static mut SaberParms: [c_char; MAX_SABER_DATA_SIZE] = [0; MAX_SABER_DATA_SIZE];
pub static mut ui_saber_parms_parsed: qboolean = 0; // qfalse

pub static mut redSaberGlowShader: qhandle_t = 0;
pub static mut redSaberCoreShader: qhandle_t = 0;
pub static mut orangeSaberGlowShader: qhandle_t = 0;
pub static mut orangeSaberCoreShader: qhandle_t = 0;
pub static mut yellowSaberGlowShader: qhandle_t = 0;
pub static mut yellowSaberCoreShader: qhandle_t = 0;
pub static mut greenSaberGlowShader: qhandle_t = 0;
pub static mut greenSaberCoreShader: qhandle_t = 0;
pub static mut blueSaberGlowShader: qhandle_t = 0;
pub static mut blueSaberCoreShader: qhandle_t = 0;
pub static mut purpleSaberGlowShader: qhandle_t = 0;
pub static mut purpleSaberCoreShader: qhandle_t = 0;

// Opaque type stubs
#[repr(C)]
pub struct refEntity_t {
    pub saberLength: f32,
    pub radius: f32,
    pub origin: vec3_t,
    pub axis: [[f32; 3]; 3],
    pub reType: c_int,
    pub customShader: qhandle_t,
    pub shaderRGBA: [u8; 4],
    pub oldorigin: vec3_t,
}

#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct itemDef_t {
    pub flags: c_int,
    pub ghoul2: *mut c_void,
}

// External function declarations
extern "C" {
    pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> qhandle_t;
    pub fn trap_FS_GetFileList(path: *const c_char, ext: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;
    pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut fileHandle_t, mode: c_int) -> c_int;
    pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    pub fn trap_FS_FCloseFile(f: fileHandle_t);
    pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn trap_G2API_HasGhoul2ModelOnIndex(ghoul2: *const *mut c_void, index: c_int) -> qboolean;
    pub fn trap_G2API_RemoveGhoul2Model(ghoul2: *mut *mut c_void, index: c_int);
    pub fn trap_G2API_InitGhoul2Model(ghoul2: *mut *mut c_void, modelname: *const c_char, modelindex: c_int, a: c_int, b: c_int, c: c_int, d: c_int) -> c_int;
    pub fn trap_G2API_SetSkin(ghoul2: *mut *mut c_void, model: c_int, lod: c_int, skin: qhandle_t);
    pub fn trap_G2API_AddBolt(ghoul2: *const *mut c_void, model: c_int, tag: *const c_char) -> c_int;
    pub fn trap_G2API_AttachG2Model(ghoul2: *mut *mut c_void, model: c_int, parent_ghoul2: *mut *mut c_void, parent_bolt: c_int, parent_model: c_int);
    pub fn trap_G2API_GetBoltMatrix(ghoul2: *const *mut c_void, model: c_int, bolt: c_int, matrix: *mut mdxaBone_t, angles: *const vec3_t, origin: *const vec3_t, time: c_int, model_list: *mut c_void, scale: *const vec3_t);
    pub fn trap_R_RegisterSkin(name: *const c_char) -> qhandle_t;
    pub fn trap_SP_GetStringTextString(ref_str: *const c_char, buffer: *mut c_char, bufsize: c_int);

    pub fn COM_ParseExt(data: *mut *const c_char, allow_newlines: qboolean) -> *const c_char;
    pub fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> qboolean;
    pub fn COM_BeginParseSession(filename: *const c_char);
    pub fn COM_Compress(data: *mut c_char) -> c_int;

    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dst: *mut c_char, src: *const c_char, dstsize: c_int);
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn atoi(nptr: *const c_char) -> c_int;
    pub fn atof(nptr: *const c_char) -> f32;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);

    pub fn SkipBracedSection(data: *mut *const c_char);
    pub fn SkipRestOfLine(data: *mut *const c_char);
    pub fn String_Alloc(str: *const c_char) -> *const c_char;

    pub fn VectorMA(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t);
    pub fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorSubtract(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    pub fn VectorAdd(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    pub fn VectorScale(in_: *const vec3_t, scale: f32, out: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn BG_GiveMeVectorFromMatrix(matrix: *const mdxaBone_t, vec_type: c_int, out: *mut vec3_t);
    pub fn crandom() -> f32;

    pub fn va(fmt: *const c_char, ...) -> *const c_char;

    pub extern "C" {
        pub static mut uiInfo: UIInfo;
    }
}

#[repr(C)]
pub struct UIInfo {
    pub movesTitleIndex: c_int,
    pub uiDC: UIDC,
}

#[repr(C)]
pub struct UIDC {
    pub realTime: c_int,
}

pub fn UI_CacheSaberGlowGraphics() {
    //FIXME: these get fucked by vid_restarts
    unsafe {
        redSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/red_glow\0".as_ptr() as *const c_char);
        redSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/red_line\0".as_ptr() as *const c_char);
        orangeSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/orange_glow\0".as_ptr() as *const c_char);
        orangeSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/orange_line\0".as_ptr() as *const c_char);
        yellowSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/yellow_glow\0".as_ptr() as *const c_char);
        yellowSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/yellow_line\0".as_ptr() as *const c_char);
        greenSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/green_glow\0".as_ptr() as *const c_char);
        greenSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/green_line\0".as_ptr() as *const c_char);
        blueSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/blue_glow\0".as_ptr() as *const c_char);
        blueSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/blue_line\0".as_ptr() as *const c_char);
        purpleSaberGlowShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/purple_glow\0".as_ptr() as *const c_char);
        purpleSaberCoreShader = trap_R_RegisterShaderNoMip(b"gfx/effects/sabers/purple_line\0".as_ptr() as *const c_char);
    }
}

pub fn UI_ParseLiteral(data: *mut *const c_char, string: *const c_char) -> qboolean {
    let mut token: *const c_char;

    unsafe {
        token = COM_ParseExt(data, 1); // qtrue
        if (*token as u8) == 0 {
            Com_Printf(b"unexpected EOF\n\0".as_ptr() as *const c_char);
            return 1; // qtrue
        }

        if Q_stricmp(token, string) != 0 {
            Com_Printf(b"required string '%s' missing\n\0".as_ptr() as *const c_char, string);
            return 1; // qtrue
        }
    }

    0 // qfalse
}

pub fn UI_ParseLiteralSilent(data: *mut *const c_char, string: *const c_char) -> qboolean {
    let mut token: *const c_char;

    unsafe {
        token = COM_ParseExt(data, 1); // qtrue
        if (*token as u8) == 0 {
            return 1; // qtrue
        }

        if Q_stricmp(token, string) != 0 {
            return 1; // qtrue
        }
    }

    0 // qfalse
}

pub fn UI_SaberParseParm(saberName: *const c_char, parmname: *const c_char, saberData: *mut c_char) -> qboolean {
    let mut token: *const c_char;
    let mut value: *const c_char;
    let mut p: *const c_char;

    unsafe {
        if saberName.is_null() || (*saberName as u8) == 0 {
            return 0; // qfalse
        }

        //try to parse it out
        p = core::ptr::addr_of!(SaberParms[0]) as *const c_char;
        // A bogus name is passed in
        COM_BeginParseSession(b"saberinfo\0".as_ptr() as *const c_char);

        // look for the right saber
        loop {
            if p.is_null() {
                break;
            }
            token = COM_ParseExt(&mut (p as *mut c_char), 1); // qtrue
            if (*token as u8) == 0 {
                return 0; // qfalse
            }

            if Q_stricmp(token, saberName) == 0 {
                break;
            }

            SkipBracedSection(&mut (p as *mut c_char));
        }
        if p.is_null() {
            return 0; // qfalse
        }

        if UI_ParseLiteral(&mut (p as *mut c_char), b"{\0".as_ptr() as *const c_char) != 0 {
            return 0; // qfalse
        }

        // parse the saber info block
        loop {
            token = COM_ParseExt(&mut (p as *mut c_char), 1); // qtrue
            if (*token as u8) == 0 {
                Com_Printf(b"%serror: unexpected EOF while parsing '%s'\n\0".as_ptr() as *const c_char, S_COLOR_RED, saberName);
                return 0; // qfalse
            }

            if Q_stricmp(token, b"}\0".as_ptr() as *const c_char) == 0 {
                break;
            }

            if Q_stricmp(token, parmname) == 0 {
                if COM_ParseString(&mut (p as *mut c_char), &mut value) != 0 {
                    continue;
                }
                strcpy(saberData, value);
                return 1; // qtrue
            }

            SkipRestOfLine(&mut (p as *mut c_char));
            continue;
        }
    }

    0 // qfalse
}


pub fn UI_SaberModelForSaber(saberName: *const c_char, saberModel: *mut c_char) -> qboolean {
    UI_SaberParseParm(saberName, b"saberModel\0".as_ptr() as *const c_char, saberModel)
}

pub fn UI_SaberSkinForSaber(saberName: *const c_char, saberSkin: *mut c_char) -> qboolean {
    UI_SaberParseParm(saberName, b"customSkin\0".as_ptr() as *const c_char, saberSkin)
}

pub fn UI_SaberTypeForSaber(saberName: *const c_char, saberType: *mut c_char) -> qboolean {
    UI_SaberParseParm(saberName, b"saberType\0".as_ptr() as *const c_char, saberType)
}

pub fn UI_SaberNumBladesForSaber(saberName: *const c_char) -> c_int {
    let mut numBlades: c_int;
    let mut numBladesString: [c_char; 8] = [0; 8];
    UI_SaberParseParm(saberName, b"numBlades\0".as_ptr() as *const c_char, &mut numBladesString[0]);
    unsafe {
        numBlades = atoi(&numBladesString[0]);
    }
    if numBlades < 1 {
        numBlades = 1;
    } else if numBlades > 8 {
        numBlades = 8;
    }
    numBlades
}

pub fn UI_SaberShouldDrawBlade(saberName: *const c_char, bladeNum: c_int) -> qboolean {
    let mut bladeStyle2Start: c_int = 0;
    let mut noBlade: c_int = 0;
    let mut bladeStyle2StartString: [c_char; 8] = [0; 8];
    let mut noBladeString: [c_char; 8] = [0; 8];
    UI_SaberParseParm(saberName, b"bladeStyle2Start\0".as_ptr() as *const c_char, &mut bladeStyle2StartString[0]);
    unsafe {
        if !bladeStyle2StartString[0].is_null()
            && bladeStyle2StartString[0] as u8 != 0 {
            bladeStyle2Start = atoi(&bladeStyle2StartString[0]);
        }
        if bladeStyle2Start != 0
            && bladeNum >= bladeStyle2Start {
            //use second blade style
            UI_SaberParseParm(saberName, b"noBlade2\0".as_ptr() as *const c_char, &mut noBladeString[0]);
            if !noBladeString[0].is_null()
                && noBladeString[0] as u8 != 0 {
                noBlade = atoi(&noBladeString[0]);
            }
        } else {
            //use first blade style
            UI_SaberParseParm(saberName, b"noBlade\0".as_ptr() as *const c_char, &mut noBladeString[0]);
            if !noBladeString[0].is_null()
                && noBladeString[0] as u8 != 0 {
                noBlade = atoi(&noBladeString[0]);
            }
        }
    }
    if noBlade == 0 { 1 } else { 0 } // ((qboolean)(noBlade==0))
}


pub fn UI_IsSaberTwoHanded(saberName: *const c_char) -> qboolean {
    let mut twoHanded: c_int;
    let mut twoHandedString: [c_char; 8] = [0; 8];
    UI_SaberParseParm(saberName, b"twoHanded\0".as_ptr() as *const c_char, &mut twoHandedString[0]);
    unsafe {
        if twoHandedString[0] as u8 == 0 {
            //not defined defaults to "no"
            return 0; // qfalse
        }
        twoHanded = atoi(&twoHandedString[0]);
    }
    if twoHanded != 0 { 1 } else { 0 } // ((qboolean)(twoHanded!=0))
}

pub fn UI_SaberBladeLengthForSaber(saberName: *const c_char, bladeNum: c_int) -> f32 {
    let mut lengthString: [c_char; 8] = [0; 8];
    let mut length: f32 = 40.0;
    UI_SaberParseParm(saberName, b"saberLength\0".as_ptr() as *const c_char, &mut lengthString[0]);
    unsafe {
        if lengthString[0] as u8 != 0 {
            length = atof(&lengthString[0]);
            if length < 0.0 {
                length = 0.0;
            }
        }

        // Construct the string "saberLength%d" where %d is bladeNum+1
        // Using va function which returns formatted string
        let len_parm_str = va(b"saberLength%d\0".as_ptr() as *const c_char, bladeNum + 1);
        UI_SaberParseParm(saberName, len_parm_str, &mut lengthString[0]);
        if lengthString[0] as u8 != 0 {
            length = atof(&lengthString[0]);
            if length < 0.0 {
                length = 0.0;
            }
        }
    }

    length
}

pub fn UI_SaberBladeRadiusForSaber(saberName: *const c_char, bladeNum: c_int) -> f32 {
    let mut radiusString: [c_char; 8] = [0; 8];
    let mut radius: f32 = 3.0;
    UI_SaberParseParm(saberName, b"saberRadius\0".as_ptr() as *const c_char, &mut radiusString[0]);
    unsafe {
        if radiusString[0] as u8 != 0 {
            radius = atof(&radiusString[0]);
            if radius < 0.0 {
                radius = 0.0;
            }
        }

        let rad_parm_str = va(b"saberRadius%d\0".as_ptr() as *const c_char, bladeNum + 1);
        UI_SaberParseParm(saberName, rad_parm_str, &mut radiusString[0]);
        if radiusString[0] as u8 != 0 {
            radius = atof(&radiusString[0]);
            if radius < 0.0 {
                radius = 0.0;
            }
        }
    }

    radius
}

pub fn UI_SaberProperNameForSaber(saberName: *const c_char, saberProperName: *mut c_char) -> qboolean {
    let mut stringedSaberName: [c_char; 1024] = [0; 1024];
    let ret = UI_SaberParseParm(saberName, b"name\0".as_ptr() as *const c_char, &mut stringedSaberName[0]);
    // if it's a stringed reference translate it
    unsafe {
        if ret != 0 && !stringedSaberName[0].is_null() && stringedSaberName[0] as u8 == b'@' {
            trap_SP_GetStringTextString(&stringedSaberName[1], saberProperName, 1024);
        } else {
            // no stringed so just use it as it
            strcpy(saberProperName, &stringedSaberName[0]);
        }
    }

    ret
}

pub fn UI_SaberValidForPlayerInMP(saberName: *const c_char) -> qboolean {
    let mut allowed: [c_char; 8] = [0; 8];
    unsafe {
        if UI_SaberParseParm(saberName, b"notInMP\0".as_ptr() as *const c_char, &mut allowed[0]) == 0 {
            //not defined, default is yes
            return 1; // qtrue
        }
        if allowed[0] as u8 == 0 {
            //not defined, default is yes
            return 1; // qtrue
        } else {
            //return value
            return if atoi(&allowed[0]) == 0 { 1 } else { 0 }; // ((qboolean)(atoi(allowed)==0))
        }
    }
}

pub fn UI_SaberLoadParms() {
    let mut len: c_int;
    let mut totallen: c_int;
    let mut saberExtFNLen: c_int;
    let mut fileCnt: c_int;
    let mut i: c_int;
    let mut holdChar: *mut c_char;
    let mut marker: *mut c_char;
    let mut saberExtensionListBuf: [c_char; 2048] = [0; 2048];
    let mut f: fileHandle_t = 0;
    let mut buffer: [c_char; MAX_MENUFILE] = [0; MAX_MENUFILE];

    //ui.Printf( "UI Parsing *.sab saber definitions\n" );

    unsafe {
        ui_saber_parms_parsed = 1; // qtrue
        UI_CacheSaberGlowGraphics();

        //set where to store the first one
        totallen = 0;
        marker = core::ptr::addr_of_mut!(SaberParms[0]);
        *marker = 0 as c_char;

        //now load in the extra .npc extensions
        fileCnt = trap_FS_GetFileList(b"ext_data/sabers\0".as_ptr() as *const c_char, b".sab\0".as_ptr() as *const c_char, &mut saberExtensionListBuf[0], 2048 as c_int);

        holdChar = &mut saberExtensionListBuf[0];
        i = 0;
        while i < fileCnt {
            saberExtFNLen = strlen(holdChar) as c_int;

            let path_str = va(b"ext_data/sabers/%s\0".as_ptr() as *const c_char, holdChar);
            len = trap_FS_FOpenFile(path_str, &mut f, FS_READ);

            if f == 0 {
                holdChar = (holdChar as *mut u8).add((saberExtFNLen + 1) as usize) as *mut c_char;
                i += 1;
                continue;
            }

            if len == -1 {
                Com_Printf(b"UI_SaberLoadParms: error reading %s\n\0".as_ptr() as *const c_char, holdChar);
            } else {
                if len > MAX_MENUFILE as c_int {
                    Com_Error(ERR_FATAL, b"UI_SaberLoadParms: file %s too large to read (max=%d)\0".as_ptr() as *const c_char, holdChar, MAX_MENUFILE as c_int);
                }
                trap_FS_Read(&mut buffer[0] as *mut c_char as *mut c_void, len, f);
                trap_FS_FCloseFile(f);
                buffer[len as usize] = 0 as c_char;

                if totallen != 0 && *(marker.offset(-1)) as u8 == b'}' {
                    //don't let it end on a } because that should be a stand-alone token
                    strcat(marker, b" \0".as_ptr() as *const c_char);
                    totallen += 1;
                    marker = marker.offset(1);
                }
                len = COM_Compress(&mut buffer[0]);

                if totallen + len >= MAX_SABER_DATA_SIZE as c_int {
                    Com_Error(ERR_FATAL, b"UI_SaberLoadParms: ran out of space before reading %s\n(you must make the .sab files smaller)\0".as_ptr() as *const c_char, holdChar);
                }
                strcat(marker, &buffer[0]);

                totallen += len;
                marker = marker.offset(len as isize);
            }
            holdChar = (holdChar as *mut u8).add((saberExtFNLen + 1) as usize) as *mut c_char;
            i += 1;
        }
    }
}

pub fn UI_DoSaber(origin: *const vec3_t, dir: *const vec3_t, length: f32, lengthMax: f32, radius: f32, color: saber_colors_t) {
    let mut mid: vec3_t = [0.0; 3];
    let mut rgb: vec3_t = [1.0, 1.0, 1.0];
    let mut blade: qhandle_t = 0;
    let mut glow: qhandle_t = 0;
    let mut saber: refEntity_t;
    let mut radiusmult: f32;
    let mut radiusRange: f32;
    let mut radiusStart: f32;

    unsafe {
        if length < 0.5 {
            // if the thing is so short, just forget even adding me.
            return;
        }

        // Find the midpoint of the saber for lighting purposes
        VectorMA(origin, length * 0.5, dir, &mut mid);

        match color {
            SABER_RED => {
                glow = redSaberGlowShader;
                blade = redSaberCoreShader;
                VectorSet(&mut rgb, 1.0, 0.2, 0.2);
            }
            SABER_ORANGE => {
                glow = orangeSaberGlowShader;
                blade = orangeSaberCoreShader;
                VectorSet(&mut rgb, 1.0, 0.5, 0.1);
            }
            SABER_YELLOW => {
                glow = yellowSaberGlowShader;
                blade = yellowSaberCoreShader;
                VectorSet(&mut rgb, 1.0, 1.0, 0.2);
            }
            SABER_GREEN => {
                glow = greenSaberGlowShader;
                blade = greenSaberCoreShader;
                VectorSet(&mut rgb, 0.2, 1.0, 0.2);
            }
            SABER_BLUE => {
                glow = blueSaberGlowShader;
                blade = blueSaberCoreShader;
                VectorSet(&mut rgb, 0.2, 0.4, 1.0);
            }
            SABER_PURPLE => {
                glow = purpleSaberGlowShader;
                blade = purpleSaberCoreShader;
                VectorSet(&mut rgb, 0.9, 0.2, 1.0);
            }
            _ => {}
        }

        // always add a light because sabers cast a nice glow before they slice you in half!!  or something...
        /*
        if ( doLight )
        {//FIXME: RGB combine all the colors of the sabers you're using into one averaged color!
            cgi_R_AddLightToScene( mid, (length*2.0f) + (random()*8.0f), rgb[0], rgb[1], rgb[2] );
        }
        */

        saber = core::mem::zeroed();

        // Saber glow is it's own ref type because it uses a ton of sprites, otherwise it would eat up too many
        //	refEnts to do each glow blob individually
        saber.saberLength = length;

        // Jeff, I did this because I foolishly wished to have a bright halo as the saber is unleashed.
        // It's not quite what I'd hoped tho.  If you have any ideas, go for it!  --Pat
        if length < lengthMax {
            radiusmult = 1.0 + (2.0 / length);		// Note this creates a curve, and length cannot be < 0.5.
        } else {
            radiusmult = 1.0;
        }

        radiusRange = radius * 0.075;
        radiusStart = radius - radiusRange;

        saber.radius = (radiusStart + crandom() * radiusRange) * radiusmult;
        //saber.radius = (2.8f + crandom() * 0.2f)*radiusmult;


        VectorCopy(origin, &mut saber.origin);
        VectorCopy(dir, &mut saber.axis[0]);
        saber.reType = RT_SABER_GLOW;
        saber.customShader = glow;
        saber.shaderRGBA[0] = 0xff;
        saber.shaderRGBA[1] = 0xff;
        saber.shaderRGBA[2] = 0xff;
        saber.shaderRGBA[3] = 0xff;
        //saber.renderfx = rfx;

        trap_R_AddRefEntityToScene(&saber);

        // Do the hot core
        VectorMA(origin, length, dir, &mut saber.origin);
        VectorMA(origin, -1.0, dir, &mut saber.oldorigin);
        saber.customShader = blade;
        saber.reType = RT_LINE;
        radiusStart = radius / 3.0;
        saber.radius = (radiusStart + crandom() * radiusRange) * radiusmult;
        //	saber.radius = (1.0 + crandom() * 0.2f)*radiusmult;

        trap_R_AddRefEntityToScene(&saber);
    }
}

pub fn SaberColorToString(color: saber_colors_t) -> *const c_char {
    if color == SABER_RED {
        return b"red\0".as_ptr() as *const c_char;
    }

    if color == SABER_ORANGE {
        return b"orange\0".as_ptr() as *const c_char;
    }

    if color == SABER_YELLOW {
        return b"yellow\0".as_ptr() as *const c_char;
    }

    if color == SABER_GREEN {
        return b"green\0".as_ptr() as *const c_char;
    }

    if color == SABER_BLUE {
        return b"blue\0".as_ptr() as *const c_char;
    }

    if color == SABER_PURPLE {
        return b"purple\0".as_ptr() as *const c_char;
    }
    core::ptr::null()
}

pub fn TranslateSaberColor(name: *const c_char) -> saber_colors_t {
    unsafe {
        if Q_stricmp(name, b"red\0".as_ptr() as *const c_char) == 0 {
            return SABER_RED;
        }
        if Q_stricmp(name, b"orange\0".as_ptr() as *const c_char) == 0 {
            return SABER_ORANGE;
        }
        if Q_stricmp(name, b"yellow\0".as_ptr() as *const c_char) == 0 {
            return SABER_YELLOW;
        }
        if Q_stricmp(name, b"green\0".as_ptr() as *const c_char) == 0 {
            return SABER_GREEN;
        }
        if Q_stricmp(name, b"blue\0".as_ptr() as *const c_char) == 0 {
            return SABER_BLUE;
        }
        if Q_stricmp(name, b"purple\0".as_ptr() as *const c_char) == 0 {
            return SABER_PURPLE;
        }
        if Q_stricmp(name, b"random\0".as_ptr() as *const c_char) == 0 {
            return Q_irand(SABER_ORANGE, SABER_PURPLE);
        }
    }
    SABER_BLUE
}

pub fn TranslateSaberType(name: *const c_char) -> saberType_t {
    unsafe {
        if Q_stricmp(name, b"SABER_SINGLE\0".as_ptr() as *const c_char) == 0 {
            return SABER_SINGLE;
        }
        if Q_stricmp(name, b"SABER_STAFF\0".as_ptr() as *const c_char) == 0 {
            return SABER_STAFF;
        }
        if Q_stricmp(name, b"SABER_BROAD\0".as_ptr() as *const c_char) == 0 {
            return SABER_BROAD;
        }
        if Q_stricmp(name, b"SABER_PRONG\0".as_ptr() as *const c_char) == 0 {
            return SABER_PRONG;
        }
        if Q_stricmp(name, b"SABER_DAGGER\0".as_ptr() as *const c_char) == 0 {
            return SABER_DAGGER;
        }
        if Q_stricmp(name, b"SABER_ARC\0".as_ptr() as *const c_char) == 0 {
            return SABER_ARC;
        }
        if Q_stricmp(name, b"SABER_SAI\0".as_ptr() as *const c_char) == 0 {
            return SABER_SAI;
        }
        if Q_stricmp(name, b"SABER_CLAW\0".as_ptr() as *const c_char) == 0 {
            return SABER_CLAW;
        }
        if Q_stricmp(name, b"SABER_LANCE\0".as_ptr() as *const c_char) == 0 {
            return SABER_LANCE;
        }
        if Q_stricmp(name, b"SABER_STAR\0".as_ptr() as *const c_char) == 0 {
            return SABER_STAR;
        }
        if Q_stricmp(name, b"SABER_TRIDENT\0".as_ptr() as *const c_char) == 0 {
            return SABER_TRIDENT;
        }
        if Q_stricmp(name, b"SABER_SITH_SWORD\0".as_ptr() as *const c_char) == 0 {
            return SABER_SITH_SWORD;
        }
    }
    SABER_SINGLE
}

pub fn UI_SaberDrawBlade(item: *mut itemDef_t, saberName: *const c_char, saberModel: c_int, saberType: saberType_t, origin: *const vec3_t, angles: *const vec3_t, bladeNum: c_int) {

    let mut bladeColorString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut bladeColor: saber_colors_t;
    let mut bladeLength: f32;
    let mut bladeRadius: f32;
    let mut bladeOrigin: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    //	vec3_t	angles={0};
    let mut boltMatrix: mdxaBone_t;
    let mut tagHack: qboolean = 0; // qfalse
    let mut tagName: *const c_char;
    let mut bolt: c_int;
    let mut scale: f32;

    unsafe {
        if (((*item).flags & ITF_ISSABER) != 0) && saberModel < 2 {
            trap_Cvar_VariableStringBuffer(b"ui_saber_color\0".as_ptr() as *const c_char, &mut bladeColorString[0], MAX_QPATH as c_int);
        } else {
            //if ( item->flags&ITF_ISSABER2 ) - presumed
            trap_Cvar_VariableStringBuffer(b"ui_saber2_color\0".as_ptr() as *const c_char, &mut bladeColorString[0], MAX_QPATH as c_int);
        }

        if trap_G2API_HasGhoul2ModelOnIndex(&(&mut (*item).ghoul2 as *mut *mut c_void), saberModel) == 0 {
            //invalid index!
            return;
        }

        bladeColor = TranslateSaberColor(&bladeColorString[0]);

        bladeLength = UI_SaberBladeLengthForSaber(saberName, bladeNum);
        bladeRadius = UI_SaberBladeRadiusForSaber(saberName, bladeNum);

        tagName = va(b"*blade%d\0".as_ptr() as *const c_char, bladeNum + 1);
        bolt = trap_G2API_AddBolt(&(&(*item).ghoul2 as *const *mut c_void), saberModel, tagName);

        if bolt == -1 {
            tagHack = 1; // qtrue
            //hmm, just fall back to the most basic tag (this will also make it work with pre-JKA saber models
            bolt = trap_G2API_AddBolt(&(&(*item).ghoul2 as *const *mut c_void), saberModel, b"*flash\0".as_ptr() as *const c_char);
            if bolt == -1 {
                //no tag_flash either?!!
                bolt = 0;
            }
        }

        //	angles[PITCH] = curYaw;
        //	angles[ROLL] = 0;

        boltMatrix = core::mem::zeroed();
        trap_G2API_GetBoltMatrix(&(&(*item).ghoul2 as *const *mut c_void), saberModel, bolt, &mut boltMatrix, angles, origin, uiInfo.uiDC.realTime, core::ptr::null_mut(), &[0.0; 3]);

        // work the matrix axis stuff into the original axis and origins used.
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut bladeOrigin);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut axis[0]);//front (was NEGATIVE_Y, but the md3->glm exporter screws up this tag somethin' awful)
                                                        //		...changed this back to NEGATIVE_Y
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut axis[1]);//right ... and changed this to NEGATIVE_X
        BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_Z, &mut axis[2]);//up

        // Where do I get scale from?
        //	scale = DC->xscale;
        scale = 1.0;

        if tagHack != 0 {
            match saberType {
                SABER_SINGLE => {
                    VectorMA(&bladeOrigin, scale, &axis[0], &mut bladeOrigin);
                }
                SABER_DAGGER | SABER_LANCE => {
                }
                SABER_STAFF => {
                    if bladeNum == 0 {
                        VectorMA(&bladeOrigin, 12.0 * scale, &axis[0], &mut bladeOrigin);
                    }
                    if bladeNum == 1 {
                        VectorScale(&axis[0], -1.0, &mut axis[0]);
                        VectorMA(&bladeOrigin, 12.0 * scale, &axis[0], &mut bladeOrigin);
                    }
                }
                SABER_BROAD => {
                    if bladeNum == 0 {
                        VectorMA(&bladeOrigin, -1.0 * scale, &axis[1], &mut bladeOrigin);
                    } else if bladeNum == 1 {
                        VectorMA(&bladeOrigin, 1.0 * scale, &axis[1], &mut bladeOrigin);
                    }
                }
                SABER_PRONG => {
                    if bladeNum == 0 {
                        VectorMA(&bladeOrigin, -3.0 * scale, &axis[1], &mut bladeOrigin);
                    } else if bladeNum == 1 {
                        VectorMA(&bladeOrigin, 3.0 * scale, &axis[1], &mut bladeOrigin);
                    }
                }
                SABER_ARC => {
                    VectorSubtract(&axis[1], &axis[2], &mut axis[1]);
                    VectorNormalize(&mut axis[1]);
                    match bladeNum {
                        0 => {
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorScale(&axis[0], 0.75, &mut axis[0]);
                            VectorScale(&axis[1], 0.25, &mut axis[1]);
                            VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                        }
                        1 => {
                            VectorScale(&axis[0], 0.25, &mut axis[0]);
                            VectorScale(&axis[1], 0.75, &mut axis[1]);
                            VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                        }
                        2 => {
                            VectorMA(&bladeOrigin, -8.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorScale(&axis[0], -0.25, &mut axis[0]);
                            VectorScale(&axis[1], 0.75, &mut axis[1]);
                            VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                        }
                        3 => {
                            VectorMA(&bladeOrigin, -16.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorScale(&axis[0], -0.75, &mut axis[0]);
                            VectorScale(&axis[1], 0.25, &mut axis[1]);
                            VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                        }
                        _ => {}
                    }
                }
                SABER_SAI => {
                    if bladeNum == 1 {
                        VectorMA(&bladeOrigin, -3.0 * scale, &axis[1], &mut bladeOrigin);
                    } else if bladeNum == 2 {
                        VectorMA(&bladeOrigin, 3.0 * scale, &axis[1], &mut bladeOrigin);
                    }
                }
                SABER_CLAW => {
                    match bladeNum {
                        0 => {
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[2], &mut bladeOrigin);
                        }
                        1 => {
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[2], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[1], &mut bladeOrigin);
                        }
                        2 => {
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 2.0 * scale, &axis[2], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, -2.0 * scale, &axis[1], &mut bladeOrigin);
                        }
                        _ => {}
                    }
                }
                SABER_STAR => {
                    match bladeNum {
                        0 => {
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        1 => {
                            VectorScale(&axis[0], 0.33, &mut axis[0]);
                            VectorScale(&axis[2], 0.67, &mut axis[2]);
                            VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        2 => {
                            VectorScale(&axis[0], -0.33, &mut axis[0]);
                            VectorScale(&axis[2], 0.67, &mut axis[2]);
                            VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        3 => {
                            VectorScale(&axis[0], -1.0, &mut axis[0]);
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        4 => {
                            VectorScale(&axis[0], -0.33, &mut axis[0]);
                            VectorScale(&axis[2], -0.67, &mut axis[2]);
                            VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        5 => {
                            VectorScale(&axis[0], 0.33, &mut axis[0]);
                            VectorScale(&axis[2], -0.67, &mut axis[2]);
                            VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                            VectorMA(&bladeOrigin, 8.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        _ => {}
                    }
                }
                SABER_TRIDENT => {
                    match bladeNum {
                        0 => {
                            VectorMA(&bladeOrigin, 24.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        1 => {
                            VectorMA(&bladeOrigin, -6.0 * scale, &axis[1], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 24.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        2 => {
                            VectorMA(&bladeOrigin, 6.0 * scale, &axis[1], &mut bladeOrigin);
                            VectorMA(&bladeOrigin, 24.0 * scale, &axis[0], &mut bladeOrigin);
                        }
                        3 => {
                            VectorMA(&bladeOrigin, -32.0 * scale, &axis[0], &mut bladeOrigin);
                            VectorScale(&axis[0], -1.0, &mut axis[0]);
                        }
                        _ => {}
                    }
                }
                SABER_SITH_SWORD => {
                    //no blade
                }
                _ => {}
            }
        }
        if saberType == SABER_SITH_SWORD {
            //draw no blade
            return;
        }

        UI_DoSaber(&bladeOrigin, &axis[0], bladeLength, bladeLength, bladeRadius, bladeColor);
    }
}

/*
void UI_SaberDrawBlades( itemDef_t *item, vec3_t origin, vec3_t angles )
{
    //NOTE: only allows one saber type in view at a time
    char saber[MAX_QPATH];
    if ( item->flags&ITF_ISSABER )
    {
        trap_Cvar_VariableStringBuffer("ui_saber", saber, sizeof(saber) );
        if ( !UI_SaberValidForPlayerInMP( saber ) )
        {
            trap_Cvar_Set( "ui_saber", "kyle" );
            trap_Cvar_VariableStringBuffer("ui_saber", saber, sizeof(saber) );
        }
    }
    else if ( item->flags&ITF_ISSABER2 )
    {
        trap_Cvar_VariableStringBuffer("ui_saber2", saber, sizeof(saber) );
        if ( !UI_SaberValidForPlayerInMP( saber ) )
        {
            trap_Cvar_Set( "ui_saber2", "kyle" );
            trap_Cvar_VariableStringBuffer("ui_saber2", saber, sizeof(saber) );
        }
    }
    else
    {
        return;
    }
    if ( saber[0] )
    {
        saberType_t saberType;
        int curBlade;
        int numBlades = UI_SaberNumBladesForSaber( saber );
        if ( numBlades )
        {//okay, here we go, time to draw each blade...
            char	saberTypeString[MAX_QPATH]={0};
            UI_SaberTypeForSaber( saber, saberTypeString );
            saberType = TranslateSaberType( saberTypeString );
            for ( curBlade = 0; curBlade < numBlades; curBlade++ )
            {
                UI_SaberDrawBlade( item, saber, saberType, origin, angles, curBlade );
            }
        }
    }
}
*/

pub fn UI_GetSaberForMenu(saber: *mut c_char, saberNum: c_int) {
    let mut saberTypeString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut saberType: saberType_t = SABER_NONE;

    unsafe {
        if saberNum == 0 {
            trap_Cvar_VariableStringBuffer(b"ui_saber\0".as_ptr() as *const c_char, saber, MAX_QPATH as c_int);
            if UI_SaberValidForPlayerInMP(saber) == 0 {
                trap_Cvar_Set(b"ui_saber\0".as_ptr() as *const c_char, b"kyle\0".as_ptr() as *const c_char);
                trap_Cvar_VariableStringBuffer(b"ui_saber\0".as_ptr() as *const c_char, saber, MAX_QPATH as c_int);
            }
        } else {
            trap_Cvar_VariableStringBuffer(b"ui_saber2\0".as_ptr() as *const c_char, saber, MAX_QPATH as c_int);
            if UI_SaberValidForPlayerInMP(saber) == 0 {
                trap_Cvar_Set(b"ui_saber2\0".as_ptr() as *const c_char, b"kyle\0".as_ptr() as *const c_char);
                trap_Cvar_VariableStringBuffer(b"ui_saber2\0".as_ptr() as *const c_char, saber, MAX_QPATH as c_int);
            }
        }
        //read this from the sabers.cfg
        UI_SaberTypeForSaber(saber, &mut saberTypeString[0]);
        if saberTypeString[0] as u8 != 0 {
            saberType = TranslateSaberType(&saberTypeString[0]);
        }

        match uiInfo.movesTitleIndex {
            0 => {
                //MD_ACROBATICS:
            }
            1 | 2 | 3 => {
                //MD_SINGLE_FAST:
                //MD_SINGLE_MEDIUM:
                //MD_SINGLE_STRONG:
                if saberType != SABER_SINGLE {
                    Q_strncpyz(saber, b"single_1\0".as_ptr() as *const c_char, MAX_QPATH as c_int);
                }
            }
            4 => {
                //MD_DUAL_SABERS:
                if saberType != SABER_SINGLE {
                    Q_strncpyz(saber, b"single_1\0".as_ptr() as *const c_char, MAX_QPATH as c_int);
                }
            }
            5 => {
                //MD_SABER_STAFF:
                if saberType == SABER_SINGLE || saberType == SABER_NONE {
                    Q_strncpyz(saber, b"dual_1\0".as_ptr() as *const c_char, MAX_QPATH as c_int);
                }
            }
            _ => {}
        }
    }
}

pub fn UI_SaberDrawBlades(item: *mut itemDef_t, origin: *const vec3_t, angles: *const vec3_t) {
    //NOTE: only allows one saber type in view at a time
    let mut saber: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut saberNum: c_int = 0;
    let mut saberModel: c_int = 0;
    let mut numSabers: c_int = 1;

    unsafe {
        if (((*item).flags & ITF_ISCHARACTER) != 0)//hacked sabermoves sabers in character's hand
            && uiInfo.movesTitleIndex == 4 /*MD_DUAL_SABERS*/ {
            numSabers = 2;
        }

        saberNum = 0;
        while saberNum < numSabers {
            if ((*item).flags & ITF_ISCHARACTER) != 0 {
                //hacked sabermoves sabers in character's hand
                UI_GetSaberForMenu(&mut saber[0], saberNum);
                saberModel = saberNum + 1;
            } else if ((*item).flags & ITF_ISSABER) != 0 {
                trap_Cvar_VariableStringBuffer(b"ui_saber\0".as_ptr() as *const c_char, &mut saber[0], MAX_QPATH as c_int);
                if UI_SaberValidForPlayerInMP(&saber[0]) == 0 {
                    trap_Cvar_Set(b"ui_saber\0".as_ptr() as *const c_char, b"kyle\0".as_ptr() as *const c_char);
                    trap_Cvar_VariableStringBuffer(b"ui_saber\0".as_ptr() as *const c_char, &mut saber[0], MAX_QPATH as c_int);
                }
                saberModel = 0;
            } else if ((*item).flags & ITF_ISSABER2) != 0 {
                trap_Cvar_VariableStringBuffer(b"ui_saber2\0".as_ptr() as *const c_char, &mut saber[0], MAX_QPATH as c_int);
                if UI_SaberValidForPlayerInMP(&saber[0]) == 0 {
                    trap_Cvar_Set(b"ui_saber2\0".as_ptr() as *const c_char, b"kyle\0".as_ptr() as *const c_char);
                    trap_Cvar_VariableStringBuffer(b"ui_saber2\0".as_ptr() as *const c_char, &mut saber[0], MAX_QPATH as c_int);
                }
                saberModel = 0;
            } else {
                return;
            }
            if saber[0] as u8 != 0 {
                let mut saberType: saberType_t;
                let mut curBlade: c_int = 0;
                let numBlades: c_int = UI_SaberNumBladesForSaber(&saber[0]);
                if numBlades != 0 {
                    //okay, here we go, time to draw each blade...
                    let mut saberTypeString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    UI_SaberTypeForSaber(&saber[0], &mut saberTypeString[0]);
                    saberType = TranslateSaberType(&saberTypeString[0]);
                    while curBlade < numBlades {
                        if UI_SaberShouldDrawBlade(&saber[0], curBlade) != 0 {
                            UI_SaberDrawBlade(item, &saber[0], saberModel, saberType, origin, angles, curBlade);
                        }
                        curBlade += 1;
                    }
                }
            }
            saberNum += 1;
        }
    }
}

pub fn UI_SaberAttachToChar(item: *mut itemDef_t) {
    let mut numSabers: c_int = 1;
    let mut saberNum: c_int = 0;

    unsafe {
        if trap_G2API_HasGhoul2ModelOnIndex(&(&mut (*item).ghoul2 as *mut *mut c_void), 2) != 0 {
            //remove any extra models
            trap_G2API_RemoveGhoul2Model(&mut (&mut (*item).ghoul2 as *mut *mut c_void), 2);
        }
        if trap_G2API_HasGhoul2ModelOnIndex(&(&mut (*item).ghoul2 as *mut *mut c_void), 1) != 0 {
            //remove any extra models
            trap_G2API_RemoveGhoul2Model(&mut (&mut (*item).ghoul2 as *mut *mut c_void), 1);
        }

        if uiInfo.movesTitleIndex == 4 /*MD_DUAL_SABERS*/ {
            numSabers = 2;
        }

        saberNum = 0;
        while saberNum < numSabers {
            //bolt sabers
            let mut modelPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut skinPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut saber: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            UI_GetSaberForMenu(&mut saber[0], saberNum);

            if UI_SaberModelForSaber(&saber[0], &mut modelPath[0]) != 0 {
                //successfully found a model
                let g2Saber: c_int = trap_G2API_InitGhoul2Model(&mut (&mut (*item).ghoul2 as *mut *mut c_void), &modelPath[0], 0, 0, 0, 0, 0); //add the model
                if g2Saber != 0 {
                    let boltNum: c_int;
                    //get the customSkin, if any
                    if UI_SaberSkinForSaber(&saber[0], &mut skinPath[0]) != 0 {
                        let g2skin: qhandle_t = trap_R_RegisterSkin(&skinPath[0]);
                        trap_G2API_SetSkin(&mut (&mut (*item).ghoul2 as *mut *mut c_void), g2Saber, 0, g2skin);//this is going to set the surfs on/off matching the skin file
                    } else {
                        trap_G2API_SetSkin(&mut (&mut (*item).ghoul2 as *mut *mut c_void), g2Saber, 0, 0);//turn off custom skin
                    }
                    if saberNum == 0 {
                        boltNum = trap_G2API_AddBolt(&(&(*item).ghoul2 as *const *mut c_void), 0, b"*r_hand\0".as_ptr() as *const c_char);
                    } else {
                        boltNum = trap_G2API_AddBolt(&(&(*item).ghoul2 as *const *mut c_void), 0, b"*l_hand\0".as_ptr() as *const c_char);
                    }
                    trap_G2API_AttachG2Model(&mut (&mut (*item).ghoul2 as *mut *mut c_void), g2Saber, &mut (&mut (*item).ghoul2 as *mut *mut c_void), boltNum, 0);
                }
            }
            saberNum += 1;
        }
    }
}

const MAX_SABER_HILTS: c_int = 64;

// Fill in with saber hilts
pub fn UI_SaberGetHiltInfo(singleHilts: *mut *const c_char, staffHilts: *mut *const c_char) {
    let mut numSingleHilts: c_int = 0;
    let mut numStaffHilts: c_int = 0;
    let mut saberName: *const c_char;
    let mut token: *const c_char;
    let mut p: *const c_char;

    unsafe {
        //go through all the loaded sabers and put the valid ones in the proper list
        p = core::ptr::addr_of!(SaberParms[0]) as *const c_char;
        COM_BeginParseSession(b"saberlist\0".as_ptr() as *const c_char);

        // look for a saber
        loop {
            if p.is_null() {
                break;
            }
            token = COM_ParseExt(&mut (p as *mut c_char), 1); // qtrue
            if (*token as u8) == 0 {
                //invalid name
                continue;
            }
            saberName = String_Alloc(token);
            //see if there's a "{" on the next line
            SkipRestOfLine(&mut (p as *mut c_char));

            if UI_ParseLiteralSilent(&mut (p as *mut c_char), b"{\0".as_ptr() as *const c_char) != 0 {
                //nope, not a name, keep looking
                continue;
            }

            //this is a saber name
            if UI_SaberValidForPlayerInMP(saberName) == 0 {
                SkipBracedSection(&mut (p as *mut c_char));
                continue;
            }

            if UI_IsSaberTwoHanded(saberName) != 0 {
                if numStaffHilts < MAX_SABER_HILTS - 1 {
                    //-1 because we have to NULL terminate the list
                    *staffHilts.offset(numStaffHilts as isize) = saberName;
                    numStaffHilts += 1;
                } else {
                    Com_Printf(b"WARNING: too many two-handed sabers, ignoring saber '%s'\n\0".as_ptr() as *const c_char, saberName);
                }
            } else {
                if numSingleHilts < MAX_SABER_HILTS - 1 {
                    //-1 because we have to NULL terminate the list
                    *singleHilts.offset(numSingleHilts as isize) = saberName;
                    numSingleHilts += 1;
                } else {
                    Com_Printf(b"WARNING: too many one-handed sabers, ignoring saber '%s'\n\0".as_ptr() as *const c_char, saberName);
                }
            }
            //skip the whole braced section and move on to the next entry
            SkipBracedSection(&mut (p as *mut c_char));
        }
        //null terminate the list so the UI code knows where to stop listing them
        *singleHilts.offset(numSingleHilts as isize) = core::ptr::null();
        *staffHilts.offset(numStaffHilts as isize) = core::ptr::null();
    }
}
