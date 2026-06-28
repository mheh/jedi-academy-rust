//
/*
=======================================================================

USER INTERFACE SABER LOADING & DISPLAY CODE

=======================================================================
*/

// leave this at the top of all UI_xxxx files for PCH reasons...
//

use core::ffi::{c_int, c_char};

// Stubs for external types and functions we depend on
// These would normally come from ui_local.h and ui_shared.h headers
pub type qhandle_t = c_int;
pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub type vec3_t = [f32; 3];

// YAW, PITCH, ROLL indices for angles
pub const YAW: usize = 0;
pub const PITCH: usize = 1;
pub const ROLL: usize = 2;

// Saber color enum
#[derive(Clone, Copy)]
pub enum saber_colors_t {
    SABER_RED = 0,
    SABER_ORANGE = 1,
    SABER_YELLOW = 2,
    SABER_GREEN = 3,
    SABER_BLUE = 4,
    SABER_PURPLE = 5,
}

// Saber type enum
#[derive(Clone, Copy, PartialEq)]
pub enum saberType_t {
    SABER_NONE = 0,
    SABER_SINGLE = 1,
    SABER_STAFF = 2,
    SABER_BROAD = 3,
    SABER_PRONG = 4,
    SABER_DAGGER = 5,
    SABER_ARC = 6,
    SABER_SAI = 7,
    SABER_CLAW = 8,
    SABER_LANCE = 9,
    SABER_STAR = 10,
    SABER_TRIDENT = 11,
    SABER_SITH_SWORD = 12,
}

// Render entity type
pub const RT_SABER_GLOW: c_int = 6;
pub const RT_LINE: c_int = 5;

// Render effect flags
pub const ITF_ISANYSABER: c_int = 0x40000;
pub const ITF_ISCHARACTER: c_int = 0x80000;
pub const ITF_ISSABER: c_int = 0x100000;
pub const ITF_ISSABER2: c_int = 0x200000;

// Vector matrix constants for g2 functions
pub const ORIGIN: c_int = 0;
pub const NEGATIVE_X: c_int = 1;
pub const NEGATIVE_Y: c_int = 2;
pub const POSITIVE_Z: c_int = 3;

pub const S_COLOR_RED: &[u8] = b"^1";
pub const MAX_QPATH: usize = 256;
pub const ERR_FATAL: c_int = 3;

#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub scale: f32,
    pub backlerp: f32,
    pub oldorigin: vec3_t,
    pub oldaxis: [vec3_t; 3],
    pub skinNum: c_int,
    pub customSkin: qhandle_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [u8; 4],
    pub radius: f32,
    pub rotation: f32,
    pub saberLength: f32,
    pub ghoul2: u64, // void *
    pub frame: c_int,
    pub oldframe: c_int,
    pub backlerp2: f32,
}

#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct itemDef_t {
    pub ghoul2: std::vec::Vec<u8>, // Placeholder for G2 model vector
    pub flags: c_int,
}

// External structs and functions - these are stubs for porting purposes
pub struct uiInfo_t {
    pub uiDC: uiDC_t,
    pub movesTitleIndex: c_int,
}

pub struct uiDC_t {
    pub realTime: c_int,
}

// Placeholder for accessing renderer and other systems
pub struct displayContext_t;

// These would be provided by the engine
extern "C" {
    pub static mut uiInfo: uiInfo_t;
    pub static mut DC: *mut displayContext_t;

    pub fn COM_ParseExt(data: *mut *const c_char, allowLineBreak: qboolean) -> *const c_char;
    pub fn COM_BeginParseSession();
    pub fn COM_Compress(data: *mut c_char) -> c_int;
    pub fn COM_ParseString(data: *mut *const c_char, value: *mut *const c_char) -> qboolean;
    pub fn SkipBracedSection(data: *mut *const c_char);
    pub fn SkipRestOfLine(data: *mut *const c_char);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize, makeNullTerminator: qboolean);
    pub fn Q_irand(minVal: c_int, maxVal: c_int) -> c_int;
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn atof(s: *const c_char) -> f32;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8;
    pub fn crandom() -> f32;
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;

    // Renderer functions
    pub fn registerShader(name: *const c_char) -> qhandle_t;
    pub fn registerSkin(name: *const c_char) -> qhandle_t;
    pub fn addRefEntityToScene(entity: *const refEntity_t);
    pub fn getCVarString(name: *const c_char, buffer: *mut c_char, size: usize);
    pub fn g2_AddBolt(ghoul2: *const u8, name: *const c_char) -> c_int;
    pub fn g2_GetBoltMatrix(ghoul2: *const u8, modelindex: c_int, bolt: c_int, matrix: *mut mdxaBone_t,
                           angles: *const vec3_t, origin: *const vec3_t, time: c_int,
                           models: *const u8, blendmatrix: *const vec3_t);
    pub fn g2_GiveMeVectorFromMatrix(matrix: mdxaBone_t, flags: c_int, vec: *mut vec3_t);
    pub fn g2_RemoveGhoul2Model(ghoul2: *mut u8, index: c_int);
    pub fn g2_InitGhoul2Model(ghoul2: *mut u8, modelname: *const c_char, a: c_int, b: c_int,
                             c: c_int, d: c_int, e: c_int) -> c_int;
    pub fn g2_SetSkin(ghoul2: *const u8, surface: c_int, skin: qhandle_t);
    pub fn G2API_AddBolt(ghoul2: *const u8, name: *const c_char) -> c_int;
    pub fn G2API_AttachG2Model(ghoul2a: *const u8, ghoul2b: *const u8, bolt: c_int, entnum: c_int);
    pub fn G2API_RemoveGhoul2Model(ghoul2: *mut u8, index: c_int);
    pub fn FS_GetFileList(path: *const c_char, ext: *const c_char, list: *mut c_char, size: c_int) -> c_int;
    pub fn FS_ReadFile(name: *const c_char, buffer: *mut *mut c_char) -> c_int;
    pub fn FS_FreeFile(buffer: *mut c_char);
}

pub struct ui_t {
    Printf: fn(*const c_char, ...),
    FS_GetFileList: fn(*const c_char, *const c_char, *mut c_char, c_int) -> c_int,
    FS_ReadFile: fn(*const c_char, *mut *mut c_char) -> c_int,
    FS_FreeFile: fn(*mut c_char),
}

extern "C" {
    pub static mut ui: ui_t;
}

#[derive(Clone, Copy)]
pub struct refDrawInterface_t {
    pub RegisterShader: fn(*const c_char) -> qhandle_t,
    pub AddRefEntityToScene: fn(*const refEntity_t),
}

pub const MAX_SABER_DATA_SIZE: usize = 0x80000;

// On Xbox, static linking lets us steal the buffer from wp_saberLoad
// Just make sure that the saber data size is the same
#[cfg(target_os = "xbox")]
extern "C" {
    pub static mut SaberParms: [c_char; MAX_SABER_DATA_SIZE];
}

#[cfg(not(target_os = "xbox"))]
pub static mut SaberParms: [c_char; MAX_SABER_DATA_SIZE] = [0; MAX_SABER_DATA_SIZE];

pub static mut ui_saber_parms_parsed: qboolean = qfalse;

static mut redSaberGlowShader: qhandle_t = 0;
static mut redSaberCoreShader: qhandle_t = 0;
static mut orangeSaberGlowShader: qhandle_t = 0;
static mut orangeSaberCoreShader: qhandle_t = 0;
static mut yellowSaberGlowShader: qhandle_t = 0;
static mut yellowSaberCoreShader: qhandle_t = 0;
static mut greenSaberGlowShader: qhandle_t = 0;
static mut greenSaberCoreShader: qhandle_t = 0;
static mut blueSaberGlowShader: qhandle_t = 0;
static mut blueSaberCoreShader: qhandle_t = 0;
static mut purpleSaberGlowShader: qhandle_t = 0;
static mut purpleSaberCoreShader: qhandle_t = 0;

pub fn UI_CacheSaberGlowGraphics() {
    //FIXME: these get fucked by vid_restarts
    unsafe {
        redSaberGlowShader = registerShader(b"gfx/effects/sabers/red_glow\0".as_ptr() as *const c_char);
        redSaberCoreShader = registerShader(b"gfx/effects/sabers/red_line\0".as_ptr() as *const c_char);
        orangeSaberGlowShader = registerShader(b"gfx/effects/sabers/orange_glow\0".as_ptr() as *const c_char);
        orangeSaberCoreShader = registerShader(b"gfx/effects/sabers/orange_line\0".as_ptr() as *const c_char);
        yellowSaberGlowShader = registerShader(b"gfx/effects/sabers/yellow_glow\0".as_ptr() as *const c_char);
        yellowSaberCoreShader = registerShader(b"gfx/effects/sabers/yellow_line\0".as_ptr() as *const c_char);
        greenSaberGlowShader = registerShader(b"gfx/effects/sabers/green_glow\0".as_ptr() as *const c_char);
        greenSaberCoreShader = registerShader(b"gfx/effects/sabers/green_line\0".as_ptr() as *const c_char);
        blueSaberGlowShader = registerShader(b"gfx/effects/sabers/blue_glow\0".as_ptr() as *const c_char);
        blueSaberCoreShader = registerShader(b"gfx/effects/sabers/blue_line\0".as_ptr() as *const c_char);
        purpleSaberGlowShader = registerShader(b"gfx/effects/sabers/purple_glow\0".as_ptr() as *const c_char);
        purpleSaberCoreShader = registerShader(b"gfx/effects/sabers/purple_line\0".as_ptr() as *const c_char);
    }
}

pub fn UI_ParseLiteral(data: &mut *const c_char, string: *const c_char) -> qboolean {
    let mut token: *const c_char;

    token = COM_ParseExt(data as *mut *const c_char, qtrue);
    unsafe {
        if *token == 0 {
            (ui.Printf)(b"unexpected EOF\n\0".as_ptr() as *const c_char);
            return qtrue;
        }

        if Q_stricmp(token, string) != 0 {
            (ui.Printf)(b"required string '%s' missing\n\0".as_ptr() as *const c_char, string);
            return qtrue;
        }
    }

    qfalse
}

pub fn UI_SaberParseParm(saberName: *const c_char, parmname: *const c_char, saberData: *mut c_char) -> qboolean {
    let mut token: *const c_char;
    let mut value: *const c_char;
    let mut p: *const c_char;

    unsafe {
        if saberName.is_null() || *saberName == 0 {
            return qfalse;
        }

        //try to parse it out
        p = core::ptr::addr_of!(SaberParms) as *const c_char;
        COM_BeginParseSession();

        // look for the right saber
        while !p.is_null() {
            token = COM_ParseExt(&mut (p as *mut c_char) as *mut *const c_char, qtrue);
            if *token == 0 {
                return qfalse;
            }

            if Q_stricmp(token, saberName) == 0 {
                break;
            }

            SkipBracedSection(&mut (p as *mut c_char) as *mut *const c_char);
        }
        if p.is_null() {
            return qfalse;
        }

        if UI_ParseLiteral(&mut (p as *mut c_char) as *mut *const c_char, b"{\0".as_ptr() as *const c_char) != qfalse {
            return qfalse;
        }

        // parse the saber info block
        loop {
            token = COM_ParseExt(&mut (p as *mut c_char) as *mut *const c_char, qtrue);
            if *token == 0 {
                (ui.Printf)(b"%s%s\0".as_ptr() as *const c_char, S_COLOR_RED.as_ptr() as *const c_char,
                           b"ERROR: unexpected EOF while parsing '%s'\n\0".as_ptr() as *const c_char, saberName);
                return qfalse;
            }

            if Q_stricmp(token, b"}\0".as_ptr() as *const c_char) == 0 {
                break;
            }

            if Q_stricmp(token, parmname) == 0 {
                if COM_ParseString(&mut (p as *mut c_char) as *mut *const c_char, &mut value as *mut *const c_char) != qfalse {
                    continue;
                }
                strcpy(saberData, value);
                return qtrue;
            }

            SkipRestOfLine(&mut (p as *mut c_char) as *mut *const c_char);
            continue;
        }

        qfalse
    }
}

pub fn UI_SaberProperNameForSaber(saberName: *const c_char, saberProperName: *mut c_char) -> qboolean {
    UI_SaberParseParm(saberName, b"name\0".as_ptr() as *const c_char, saberProperName)
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
    let mut numBladesString: [c_char; 8] = [0; 8];
    UI_SaberParseParm(saberName, b"numBlades\0".as_ptr() as *const c_char, numBladesString.as_mut_ptr());
    let mut numBlades = unsafe { atoi(numBladesString.as_ptr()) };
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
    UI_SaberParseParm(saberName, b"bladeStyle2Start\0".as_ptr() as *const c_char, bladeStyle2StartString.as_mut_ptr());
    unsafe {
        if !bladeStyle2StartString.as_ptr().is_null() && *bladeStyle2StartString.as_ptr() != 0 {
            bladeStyle2Start = atoi(bladeStyle2StartString.as_ptr());
        }
        if bladeStyle2Start != 0 && bladeNum >= bladeStyle2Start {
            //use second blade style
            UI_SaberParseParm(saberName, b"noBlade2\0".as_ptr() as *const c_char, noBladeString.as_mut_ptr());
            if !noBladeString.as_ptr().is_null() && *noBladeString.as_ptr() != 0 {
                noBlade = atoi(noBladeString.as_ptr());
            }
        } else {
            //use first blade style
            UI_SaberParseParm(saberName, b"noBlade\0".as_ptr() as *const c_char, noBladeString.as_mut_ptr());
            if !noBladeString.as_ptr().is_null() && *noBladeString.as_ptr() != 0 {
                noBlade = atoi(noBladeString.as_ptr());
            }
        }
    }
    if noBlade == 0 { qtrue } else { qfalse }
}

pub fn UI_SaberBladeLengthForSaber(saberName: *const c_char, bladeNum: c_int) -> f32 {
    let mut lengthString: [c_char; 8] = [0; 8];
    let mut length: f32 = 40.0f32;
    UI_SaberParseParm(saberName, b"saberLength\0".as_ptr() as *const c_char, lengthString.as_mut_ptr());
    unsafe {
        if lengthString[0] != 0 {
            length = atof(lengthString.as_ptr());
            if length < 0.0f32 {
                length = 0.0f32;
            }
        }

        let mut parmname_buf: [c_char; 32] = [0; 32];
        core::ptr::write_bytes(parmname_buf.as_mut_ptr(), 0, 32);
        // va("saberLength%d", bladeNum+1)
        // For now using a simpler approach
        UI_SaberParseParm(saberName, b"saberLength1\0".as_ptr() as *const c_char, lengthString.as_mut_ptr()); // Would need va() replacement
        if lengthString[0] != 0 {
            length = atof(lengthString.as_ptr());
            if length < 0.0f32 {
                length = 0.0f32;
            }
        }
    }

    length
}

pub fn UI_SaberBladeRadiusForSaber(saberName: *const c_char, bladeNum: c_int) -> f32 {
    let mut radiusString: [c_char; 8] = [0; 8];
    let mut radius: f32 = 3.0f32;
    UI_SaberParseParm(saberName, b"saberRadius\0".as_ptr() as *const c_char, radiusString.as_mut_ptr());
    unsafe {
        if radiusString[0] != 0 {
            radius = atof(radiusString.as_ptr());
            if radius < 0.0f32 {
                radius = 0.0f32;
            }
        }

        UI_SaberParseParm(saberName, b"saberRadius1\0".as_ptr() as *const c_char, radiusString.as_mut_ptr()); // Would need va() replacement
        if radiusString[0] != 0 {
            radius = atof(radiusString.as_ptr());
            if radius < 0.0f32 {
                radius = 0.0f32;
            }
        }
    }

    radius
}

pub fn UI_SaberLoadParms() {
    unsafe {
        let mut len: c_int;
        let mut totallen: c_int;
        let mut saberExtFNLen: c_int;
        let mut fileCnt: c_int;
        let mut i: c_int;
        let mut buffer: *mut c_char;
        let mut holdChar: *mut c_char;
        let mut marker: *mut c_char;
        let saberExtensionListBuf: [c_char; 2048] = [0; 2048];  //	The list of file names read in

        //ui.Printf( "UI Parsing *.sab saber definitions\n" );

        ui_saber_parms_parsed = qtrue;
        UI_CacheSaberGlowGraphics();

        //set where to store the first one
        totallen = 0;
        marker = core::ptr::addr_of_mut!(SaberParms) as *mut c_char;
        *marker = '\0' as c_char;

        //now load in the sabers
        fileCnt = (ui.FS_GetFileList)(b"ext_data/sabers\0".as_ptr() as *const c_char, b".sab\0".as_ptr() as *const c_char, saberExtensionListBuf.as_ptr() as *mut c_char, 2048);

        holdChar = saberExtensionListBuf.as_ptr() as *mut c_char;
        i = 0;
        while i < fileCnt {
            saberExtFNLen = strlen(holdChar) as c_int;

            len = (ui.FS_ReadFile)(va(b"ext_data/sabers/%s\0".as_ptr() as *const c_char, holdChar), &mut buffer as *mut *mut c_char);

            if len == -1 {
                (ui.Printf)(b"UI_SaberLoadParms: error reading %s\n\0".as_ptr() as *const c_char, holdChar);
            } else {
                if totallen != 0 && *(marker.offset(-1)) == '}' as c_char {
                    //don't let it end on a } because that should be a stand-alone token
                    strcat(marker, b" \0".as_ptr() as *const c_char);
                    totallen += 1;
                    marker = marker.offset(1);
                }
                len = COM_Compress(buffer);

                if totallen + len >= MAX_SABER_DATA_SIZE as c_int {
                    Com_Error(ERR_FATAL, b"UI_SaberLoadParms: ran out of space before reading %s\n(you must make the .npc files smaller)\0".as_ptr() as *const c_char, holdChar);
                }
                strcat(marker, buffer);
                (ui.FS_FreeFile)(buffer);

                totallen += len;
                marker = marker.offset(len as isize);
            }
            i += 1;
            holdChar = holdChar.offset((saberExtFNLen + 1) as isize);
        }
    }
}

// Vector operations - these are inlined/macros in C
fn VectorMA(v1: &vec3_t, scale: f32, v2: &vec3_t, out: &mut vec3_t) {
    out[0] = v1[0] + scale * v2[0];
    out[1] = v1[1] + scale * v2[1];
    out[2] = v1[2] + scale * v2[2];
}

fn VectorSet(out: &mut vec3_t, x: f32, y: f32, z: f32) {
    out[0] = x;
    out[1] = y;
    out[2] = z;
}

fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

fn VectorScale(v: &vec3_t, scale: f32, out: &mut vec3_t) {
    out[0] = v[0] * scale;
    out[1] = v[1] * scale;
    out[2] = v[2] * scale;
}

fn VectorSubtract(v1: &vec3_t, v2: &vec3_t, out: &mut vec3_t) {
    out[0] = v1[0] - v2[0];
    out[1] = v1[1] - v2[1];
    out[2] = v1[2] - v2[2];
}

fn VectorNormalize(v: &mut vec3_t) {
    let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

fn VectorAdd(v1: &vec3_t, v2: &vec3_t, out: &mut vec3_t) {
    out[0] = v1[0] + v2[0];
    out[1] = v1[1] + v2[1];
    out[2] = v1[2] + v2[2];
}

pub fn UI_DoSaber(origin: &vec3_t, dir: &vec3_t, length: f32, lengthMax: f32, radius: f32, color: saber_colors_t) {
    let mut mid: vec3_t = [0.0; 3];
    let mut rgb: vec3_t = [1.0, 1.0, 1.0];
    let mut blade: qhandle_t = 0;
    let mut glow: qhandle_t = 0;
    let mut saber: refEntity_t;
    let mut radiusmult: f32;

    if length < 0.5f32 {
        // if the thing is so short, just forget even adding me.
        return;
    }

    // Find the midpoint of the saber for lighting purposes
    VectorMA(origin, length * 0.5f32, dir, &mut mid);

    match color {
        saber_colors_t::SABER_RED => {
            unsafe {
                glow = redSaberGlowShader;
                blade = redSaberCoreShader;
            }
            VectorSet(&mut rgb, 1.0f32, 0.2f32, 0.2f32);
        }
        saber_colors_t::SABER_ORANGE => {
            unsafe {
                glow = orangeSaberGlowShader;
                blade = orangeSaberCoreShader;
            }
            VectorSet(&mut rgb, 1.0f32, 0.5f32, 0.1f32);
        }
        saber_colors_t::SABER_YELLOW => {
            unsafe {
                glow = yellowSaberGlowShader;
                blade = yellowSaberCoreShader;
            }
            VectorSet(&mut rgb, 1.0f32, 1.0f32, 0.2f32);
        }
        saber_colors_t::SABER_GREEN => {
            unsafe {
                glow = greenSaberGlowShader;
                blade = greenSaberCoreShader;
            }
            VectorSet(&mut rgb, 0.2f32, 1.0f32, 0.2f32);
        }
        saber_colors_t::SABER_BLUE => {
            unsafe {
                glow = blueSaberGlowShader;
                blade = blueSaberCoreShader;
            }
            VectorSet(&mut rgb, 0.2f32, 0.4f32, 1.0f32);
        }
        saber_colors_t::SABER_PURPLE => {
            unsafe {
                glow = purpleSaberGlowShader;
                blade = purpleSaberCoreShader;
            }
            VectorSet(&mut rgb, 0.9f32, 0.2f32, 1.0f32);
        }
    }

    // always add a light because sabers cast a nice glow before they slice you in half!!  or something...
    /*
    if ( doLight )
    {//FIXME: RGB combine all the colors of the sabers you're using into one averaged color!
        cgi_R_AddLightToScene( mid, (length*2.0f) + (random()*8.0f), rgb[0], rgb[1], rgb[2] );
    }
    */

    saber = refEntity_t {
        reType: 0,
        renderfx: 0,
        hModel: 0,
        lightingOrigin: [0.0; 3],
        shadowPlane: 0.0,
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
        nonNormalizedAxes: 0,
        scale: 0.0,
        backlerp: 0.0,
        oldorigin: [0.0; 3],
        oldaxis: [[0.0; 3]; 3],
        skinNum: 0,
        customSkin: 0,
        customShader: 0,
        shaderRGBA: [0; 4],
        radius: 0.0,
        rotation: 0.0,
        saberLength: 0.0,
        ghoul2: 0,
        frame: 0,
        oldframe: 0,
        backlerp2: 0.0,
    };

    // Saber glow is it's own ref type because it uses a ton of sprites, otherwise it would eat up too many
    //	refEnts to do each glow blob individually
    saber.saberLength = length;

    // Jeff, I did this because I foolishly wished to have a bright halo as the saber is unleashed.
    // It's not quite what I'd hoped tho.  If you have any ideas, go for it!  --Pat
    if length < lengthMax {
        radiusmult = 1.0 + (2.0 / length); // Note this creates a curve, and length cannot be < 0.5.
    } else {
        radiusmult = 1.0;
    }

    let radiusRange = radius * 0.075f32;
    let radiusStart = radius - radiusRange;

    saber.radius = (radiusStart + unsafe { crandom() } * radiusRange) * radiusmult;
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

    unsafe {
        addRefEntityToScene(&saber);
    }

    // Do the hot core
    let mut tmpOrigin: vec3_t = [0.0; 3];
    let mut tmpOldOrigin: vec3_t = [0.0; 3];
    VectorMA(origin, length, dir, &mut tmpOrigin);
    VectorMA(origin, -1.0f32, dir, &mut tmpOldOrigin);
    VectorCopy(&tmpOrigin, &mut saber.origin);
    VectorCopy(&tmpOldOrigin, &mut saber.oldorigin);
    saber.customShader = blade;
    saber.reType = RT_LINE;
    let radiusStart2 = radius / 3.0f32;
    saber.radius = (radiusStart2 + unsafe { crandom() } * radiusRange) * radiusmult;
    //	saber.radius = (1.0 + crandom() * 0.2f)*radiusmult;

    unsafe {
        addRefEntityToScene(&saber);
    }
}

pub fn TranslateSaberColor(name: *const c_char) -> saber_colors_t {
    unsafe {
        if Q_stricmp(name, b"red\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_RED;
        }
        if Q_stricmp(name, b"orange\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_ORANGE;
        }
        if Q_stricmp(name, b"yellow\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_YELLOW;
        }
        if Q_stricmp(name, b"green\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_GREEN;
        }
        if Q_stricmp(name, b"blue\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_BLUE;
        }
        if Q_stricmp(name, b"purple\0".as_ptr() as *const c_char) == 0 {
            return saber_colors_t::SABER_PURPLE;
        }
        if Q_stricmp(name, b"random\0".as_ptr() as *const c_char) == 0 {
            return std::mem::transmute::<c_int, saber_colors_t>(Q_irand(saber_colors_t::SABER_ORANGE as c_int, saber_colors_t::SABER_PURPLE as c_int));
        }
    }
    saber_colors_t::SABER_BLUE
}

pub fn TranslateSaberType(name: *const c_char) -> saberType_t {
    unsafe {
        if Q_stricmp(name, b"SABER_SINGLE\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_SINGLE;
        }
        if Q_stricmp(name, b"SABER_STAFF\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_STAFF;
        }
        if Q_stricmp(name, b"SABER_BROAD\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_BROAD;
        }
        if Q_stricmp(name, b"SABER_PRONG\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_PRONG;
        }
        if Q_stricmp(name, b"SABER_DAGGER\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_DAGGER;
        }
        if Q_stricmp(name, b"SABER_ARC\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_ARC;
        }
        if Q_stricmp(name, b"SABER_SAI\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_SAI;
        }
        if Q_stricmp(name, b"SABER_CLAW\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_CLAW;
        }
        if Q_stricmp(name, b"SABER_LANCE\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_LANCE;
        }
        if Q_stricmp(name, b"SABER_STAR\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_STAR;
        }
        if Q_stricmp(name, b"SABER_TRIDENT\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_TRIDENT;
        }
        if Q_stricmp(name, b"SABER_SITH_SWORD\0".as_ptr() as *const c_char) == 0 {
            return saberType_t::SABER_SITH_SWORD;
        }
    }
    saberType_t::SABER_SINGLE
}

pub fn UI_SaberDrawBlade(item: *mut itemDef_t, saberName: *const c_char, saberModel: c_int, saberType: saberType_t, origin: &vec3_t, curYaw: f32, bladeNum: c_int) {
    let mut bladeColorString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut angles: vec3_t = [0.0; 3];

    unsafe {
        if (*item).flags & ITF_ISANYSABER != 0 && (*item).flags & ITF_ISCHARACTER != 0 {
            //it's bolted to a dude!
            angles[YAW] = curYaw;
        } else {
            angles[PITCH] = curYaw;
            angles[ROLL] = 90.0f32;
        }

        if saberModel >= 0 {
            // TODO: check item->ghoul2.size() - this requires proper G2 vector implementation
            // if ( saberModel >= item->ghoul2.size() )
            //{//uhh... invalid index!
            //	return;
            //}
        }

        if (*item).flags & ITF_ISSABER != 0 && saberModel < 2 {
            getCVarString(b"ui_saber_color\0".as_ptr() as *const c_char, bladeColorString.as_mut_ptr(), MAX_QPATH);
        } else {
            //if ( item->flags&ITF_ISSABER2 ) - presumed
            getCVarString(b"ui_saber2_color\0".as_ptr() as *const c_char, bladeColorString.as_mut_ptr(), MAX_QPATH);
        }
    }
    let bladeColor = TranslateSaberColor(bladeColorString.as_ptr());

    let bladeLength = UI_SaberBladeLengthForSaber(saberName, bladeNum);
    let bladeRadius = UI_SaberBladeRadiusForSaber(saberName, bladeNum);
    let mut bladeOrigin: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut boltMatrix: mdxaBone_t = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut tagHack: qboolean = qfalse;

    let tagName = va(b"*blade%d\0".as_ptr() as *const c_char, bladeNum + 1);
    let bolt = unsafe { g2_AddBolt((*item).ghoul2 as *const u8, tagName) };

    if bolt == -1 {
        tagHack = qtrue;
        //hmm, just fall back to the most basic tag (this will also make it work with pre-JKA saber models
        let bolt2 = unsafe { g2_AddBolt((*item).ghoul2 as *const u8, b"*flash\0".as_ptr() as *const c_char) };
        let bolt = if bolt2 == -1 {
            //no tag_flash either?!!
            0
        } else {
            bolt2
        };

        unsafe {
            let vec3_origin: vec3_t = [0.0; 3];
            g2_GetBoltMatrix((*item).ghoul2 as *const u8, saberModel, bolt, &mut boltMatrix,
                           angles.as_ptr(), origin, uiInfo.uiDC.realTime, std::ptr::null(), vec3_origin.as_ptr());
        }
    } else {
        unsafe {
            let vec3_origin: vec3_t = [0.0; 3];
            g2_GetBoltMatrix((*item).ghoul2 as *const u8, saberModel, bolt, &mut boltMatrix,
                           angles.as_ptr(), origin, uiInfo.uiDC.realTime, std::ptr::null(), vec3_origin.as_ptr());
        }
    }

    // work the matrix axis stuff into the original axis and origins used.
    unsafe {
        g2_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, bladeOrigin.as_mut_ptr());
        g2_GiveMeVectorFromMatrix(boltMatrix, NEGATIVE_X, axis[0].as_mut_ptr()); //front (was NEGATIVE_Y, but the md3->glm exporter screws up this tag somethin' awful)
        g2_GiveMeVectorFromMatrix(boltMatrix, NEGATIVE_Y, axis[1].as_mut_ptr()); //right
        g2_GiveMeVectorFromMatrix(boltMatrix, POSITIVE_Z, axis[2].as_mut_ptr()); //up
    }

    // TODO: fetch xscale from DC
    let scale = 1.0f32;

    if tagHack != qfalse {
        match saberType {
            saberType_t::SABER_SINGLE | saberType_t::SABER_DAGGER | saberType_t::SABER_LANCE => {}
            saberType_t::SABER_STAFF => {
                if bladeNum == 1 {
                    VectorScale(&axis[0], -1.0f32, &mut axis[0]);
                    VectorMA(&bladeOrigin, 16.0f32 * scale, &axis[0], &mut bladeOrigin);
                }
            }
            saberType_t::SABER_BROAD => {
                if bladeNum == 0 {
                    VectorMA(&bladeOrigin, -1.0f32 * scale, &axis[1], &mut bladeOrigin);
                } else if bladeNum == 1 {
                    VectorMA(&bladeOrigin, 1.0f32 * scale, &axis[1], &mut bladeOrigin);
                }
            }
            saberType_t::SABER_PRONG => {
                if bladeNum == 0 {
                    VectorMA(&bladeOrigin, -3.0f32 * scale, &axis[1], &mut bladeOrigin);
                } else if bladeNum == 1 {
                    VectorMA(&bladeOrigin, 3.0f32 * scale, &axis[1], &mut bladeOrigin);
                }
            }
            saberType_t::SABER_ARC => {
                VectorSubtract(&axis[1], &axis[2], &mut axis[1]);
                VectorNormalize(&mut axis[1]);
                match bladeNum {
                    0 => {
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorScale(&axis[0], 0.75f32, &mut axis[0]);
                        VectorScale(&axis[1], 0.25f32, &mut axis[1]);
                        VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                    }
                    1 => {
                        VectorScale(&axis[0], 0.25f32, &mut axis[0]);
                        VectorScale(&axis[1], 0.75f32, &mut axis[1]);
                        VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                    }
                    2 => {
                        VectorMA(&bladeOrigin, -8.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorScale(&axis[0], -0.25f32, &mut axis[0]);
                        VectorScale(&axis[1], 0.75f32, &mut axis[1]);
                        VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                    }
                    3 => {
                        VectorMA(&bladeOrigin, -16.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorScale(&axis[0], -0.75f32, &mut axis[0]);
                        VectorScale(&axis[1], 0.25f32, &mut axis[1]);
                        VectorAdd(&axis[0], &axis[1], &mut axis[0]);
                    }
                    _ => {}
                }
            }
            saberType_t::SABER_SAI => {
                if bladeNum == 1 {
                    VectorMA(&bladeOrigin, -3.0f32 * scale, &axis[1], &mut bladeOrigin);
                } else if bladeNum == 2 {
                    VectorMA(&bladeOrigin, 3.0f32 * scale, &axis[1], &mut bladeOrigin);
                }
            }
            saberType_t::SABER_CLAW => {
                match bladeNum {
                    0 => {
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[2], &mut bladeOrigin);
                    }
                    1 => {
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[2], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[1], &mut bladeOrigin);
                    }
                    2 => {
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 2.0f32 * scale, &axis[2], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, -2.0f32 * scale, &axis[1], &mut bladeOrigin);
                    }
                    _ => {}
                }
            }
            saberType_t::SABER_STAR => {
                match bladeNum {
                    0 => {
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    1 => {
                        VectorScale(&axis[0], 0.33f32, &mut axis[0]);
                        VectorScale(&axis[2], 0.67f32, &mut axis[2]);
                        VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    2 => {
                        VectorScale(&axis[0], -0.33f32, &mut axis[0]);
                        VectorScale(&axis[2], 0.67f32, &mut axis[2]);
                        VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    3 => {
                        VectorScale(&axis[0], -1.0f32, &mut axis[0]);
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    4 => {
                        VectorScale(&axis[0], -0.33f32, &mut axis[0]);
                        VectorScale(&axis[2], -0.67f32, &mut axis[2]);
                        VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    5 => {
                        VectorScale(&axis[0], 0.33f32, &mut axis[0]);
                        VectorScale(&axis[2], -0.67f32, &mut axis[2]);
                        VectorAdd(&axis[0], &axis[2], &mut axis[0]);
                        VectorMA(&bladeOrigin, 8.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    _ => {}
                }
            }
            saberType_t::SABER_TRIDENT => {
                match bladeNum {
                    0 => {
                        VectorMA(&bladeOrigin, 24.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    1 => {
                        VectorMA(&bladeOrigin, -6.0f32 * scale, &axis[1], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 24.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    2 => {
                        VectorMA(&bladeOrigin, 6.0f32 * scale, &axis[1], &mut bladeOrigin);
                        VectorMA(&bladeOrigin, 24.0f32 * scale, &axis[0], &mut bladeOrigin);
                    }
                    3 => {
                        VectorMA(&bladeOrigin, -32.0f32 * scale, &axis[0], &mut bladeOrigin);
                        VectorScale(&axis[0], -1.0f32, &mut axis[0]);
                    }
                    _ => {}
                }
            }
            saberType_t::SABER_SITH_SWORD => {
                //no blade
            }
            _ => {}
        }
    }
    if saberType == saberType_t::SABER_SITH_SWORD {
        //draw no blade
        return;
    }

    UI_DoSaber(&bladeOrigin, &axis[0], bladeLength, bladeLength, bladeRadius, bladeColor);
}

extern "C" {
    pub fn ItemParse_asset_model_go(item: *mut itemDef_t, name: *const c_char) -> qboolean;
    pub fn ItemParse_model_g2skin_go(item: *mut itemDef_t, skinName: *const c_char) -> qboolean;
}

pub fn UI_GetSaberForMenu(saber: *mut c_char, saberNum: c_int) {
    let mut saberTypeString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut saberType: saberType_t = saberType_t::SABER_NONE;

    unsafe {
        if saberNum == 0 {
            getCVarString(b"g_saber\0".as_ptr() as *const c_char, saber, MAX_QPATH);
        } else {
            getCVarString(b"g_saber2\0".as_ptr() as *const c_char, saber, MAX_QPATH);
        }
        //read this from the sabers.cfg
        UI_SaberTypeForSaber(saber, saberTypeString.as_mut_ptr());
        if saberTypeString[0] != 0 {
            saberType = TranslateSaberType(saberTypeString.as_ptr());
        }

        match uiInfo.movesTitleIndex {
            0 => {
                //MD_ACROBATICS:
            }
            1 | 2 | 3 => {
                //MD_SINGLE_FAST, MD_SINGLE_MEDIUM, MD_SINGLE_STRONG:
                if saberType != saberType_t::SABER_SINGLE {
                    Q_strncpyz(saber, b"single_1\0".as_ptr() as *const c_char, MAX_QPATH, qtrue);
                }
            }
            4 => {
                //MD_DUAL_SABERS:
                if saberType != saberType_t::SABER_SINGLE {
                    Q_strncpyz(saber, b"single_1\0".as_ptr() as *const c_char, MAX_QPATH, qtrue);
                }
            }
            5 => {
                //MD_SABER_STAFF:
                if saberType == saberType_t::SABER_SINGLE || saberType == saberType_t::SABER_NONE {
                    Q_strncpyz(saber, b"dual_1\0".as_ptr() as *const c_char, MAX_QPATH, qtrue);
                }
            }
            _ => {}
        }
    }
}

pub fn UI_SaberDrawBlades(item: *mut itemDef_t, origin: &vec3_t, curYaw: f32) {
    //NOTE: only allows one saber type in view at a time
    let mut saber: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut saberNum: c_int = 0;
    let mut saberModel: c_int = 0;
    let mut numSabers: c_int = 1;

    unsafe {
        if (*item).flags & ITF_ISCHARACTER != 0 {
            //hacked sabermoves sabers in character's hand
            if uiInfo.movesTitleIndex == 4 {
                //MD_DUAL_SABERS
                numSabers = 2;
            }
        }

        while saberNum < numSabers {
            if (*item).flags & ITF_ISCHARACTER != 0 {
                //hacked sabermoves sabers in character's hand
                UI_GetSaberForMenu(saber.as_mut_ptr(), saberNum);
                saberModel = saberNum + 1;
            } else if (*item).flags & ITF_ISSABER != 0 {
                getCVarString(b"ui_saber\0".as_ptr() as *const c_char, saber.as_mut_ptr(), MAX_QPATH);
                saberModel = 0;
            } else if (*item).flags & ITF_ISSABER2 != 0 {
                getCVarString(b"ui_saber2\0".as_ptr() as *const c_char, saber.as_mut_ptr(), MAX_QPATH);
                saberModel = 0;
            } else {
                return;
            }
            if saber[0] != 0 {
                let numBlades = UI_SaberNumBladesForSaber(saber.as_ptr());
                if numBlades != 0 {
                    //okay, here we go, time to draw each blade...
                    let mut saberTypeString: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    UI_SaberTypeForSaber(saber.as_ptr(), saberTypeString.as_mut_ptr());
                    let saberType = TranslateSaberType(saberTypeString.as_ptr());
                    let mut curBlade: c_int = 0;
                    while curBlade < numBlades {
                        if UI_SaberShouldDrawBlade(saber.as_ptr(), curBlade) != qfalse {
                            UI_SaberDrawBlade(item, saber.as_ptr(), saberModel, saberType, origin, curYaw, curBlade);
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
    unsafe {
        let mut numSabers: c_int = 1;
        let mut saberNum: c_int = 0;

        // TODO: These size/mModelindex checks need proper ghoul2 vector implementation
        // if ( item->ghoul2.size() > 2 && item->ghoul2[2].mModelindex >=0 )
        //{//remove any extra models
        //	DC->g2_RemoveGhoul2Model(item->ghoul2, 2);
        //}
        // if ( item->ghoul2.size() > 1 && item->ghoul2[1].mModelindex >=0)
        //{//remove any extra models
        //	DC->g2_RemoveGhoul2Model(item->ghoul2, 1);
        //}

        if uiInfo.movesTitleIndex == 4 {
            //MD_DUAL_SABERS
            numSabers = 2;
        }

        while saberNum < numSabers {
            //bolt sabers
            let mut modelPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut skinPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut saber: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            UI_GetSaberForMenu(saber.as_mut_ptr(), saberNum);

            if UI_SaberModelForSaber(saber.as_ptr(), modelPath.as_mut_ptr()) != qfalse {
                //successfully found a model
                let g2Saber = g2_InitGhoul2Model((*item).ghoul2 as *mut u8, modelPath.as_ptr(), 0, 0, 0, 0, 0); //add the model
                if g2Saber != 0 {
                    //get the customSkin, if any
                    if UI_SaberSkinForSaber(saber.as_ptr(), skinPath.as_mut_ptr()) != qfalse {
                        let g2skin = registerSkin(skinPath.as_ptr());
                        g2_SetSkin(&((*item).ghoul2 as *const u8).offset(g2Saber as isize), 0, g2skin); //this is going to set the surfs on/off matching the skin file
                    } else {
                        g2_SetSkin(&((*item).ghoul2 as *const u8).offset(g2Saber as isize), -1, 0); //turn off custom skin
                    }
                    let boltNum: c_int;
                    if saberNum == 0 {
                        boltNum = G2API_AddBolt(&((*item).ghoul2 as *const u8), b"*r_hand\0".as_ptr() as *const c_char);
                    } else {
                        boltNum = G2API_AddBolt(&((*item).ghoul2 as *const u8), b"*l_hand\0".as_ptr() as *const c_char);
                    }
                    G2API_AttachG2Model(&((*item).ghoul2 as *const u8).offset(g2Saber as isize), &((*item).ghoul2 as *const u8), boltNum, 0);
                }
            }
            saberNum += 1;
        }
    }
}
