// this line must stay at top so the whole PCH thing works...

// #include "cg_headers.h"

// #include "cg_local.h"

// this file is only included when building a dll

// #ifdef _IMMERSION
// #include "../ff/ff.h"
// #else
// /////////////////////  this is a bit kludgy, but it only gives access to one
// //							enum table because of the #define. May get changed.
// #define CGAME_ONLY
// #include "../client/fffx.h"
// //
// /////////////////////
// #endif // _IMMERSION

use core::ffi::{c_int, c_char, c_void};

// Types referenced below are defined in imported modules (cg_headers, etc.)
// vmCvar_t, vec3_t, trace_t, fileHandle_t, clipHandle_t, sfxHandle_t, qhandle_t, etc.

// Syscall command enum values (referenced from other modules)
// CG_PRINT, CG_ERROR, CG_MILLISECONDS, etc.

// prototypes
extern "C" {
    fn CG_PreInit();
}

// This static function pointer will be set by dllEntry()
// In the original C code it was initialized to (int (*)( int, ...))-1 as a sentinel
// We use Option to represent the uninitialized/initialized states
static mut syscall: Option<unsafe extern "C" fn(c_int, ...) -> c_int> = None;

#[cfg(target_os = "xbox")]
pub extern "C" fn cg_dllEntry(syscallptr: unsafe extern "C" fn(c_int, ...) -> c_int) {
    unsafe {
        syscall = Some(syscallptr);
        CG_PreInit();
    }
}

#[cfg(not(target_os = "xbox"))]
pub extern "C" fn dllEntry(syscallptr: unsafe extern "C" fn(c_int, ...) -> c_int) {
    unsafe {
        syscall = Some(syscallptr);
        CG_PreInit();
    }
}

#[inline]
fn PASSFLOAT(x: f32) -> c_int {
    let floatTemp = x;
    unsafe { core::mem::transmute::<f32, c_int>(floatTemp) }
}

pub extern "C" fn cgi_Printf(fmt: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_PRINT as c_int, fmt);
        }
    }
}

pub extern "C" fn cgi_Error(fmt: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ERROR as c_int, fmt);
        }
    }
}

pub extern "C" fn cgi_Milliseconds() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_MILLISECONDS as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_Cvar_Register(vmCvar: *mut c_void, varName: *const c_char, defaultValue: *const c_char, flags: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CVAR_REGISTER as c_int, vmCvar, varName, defaultValue, flags);
        }
    }
}

pub extern "C" fn cgi_Cvar_Update(vmCvar: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CVAR_UPDATE as c_int, vmCvar);
        }
    }
}

pub extern "C" fn cgi_Cvar_Set(var_name: *const c_char, value: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CVAR_SET as c_int, var_name, value);
        }
    }
}

pub extern "C" fn cgi_Argc() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ARGC as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_Argv(n: c_int, buffer: *mut c_char, bufferLength: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ARGV as c_int, n, buffer, bufferLength);
        }
    }
}

pub extern "C" fn cgi_Args(buffer: *mut c_char, bufferLength: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ARGS as c_int, buffer, bufferLength);
        }
    }
}

pub extern "C" fn cgi_FS_FOpenFile(qpath: *const c_char, f: *mut c_void, mode: c_int) -> c_int {
    unsafe {
        if let Some(func) = syscall {
            func(CG_FS_FOPENFILE as c_int, qpath, f, mode)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_FS_Read(buffer: *mut c_void, len: c_int, f: c_int) -> c_int {
    unsafe {
        if let Some(func) = syscall {
            func(CG_FS_READ as c_int, buffer, len, f)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_FS_Write(buffer: *const c_void, len: c_int, f: c_int) -> c_int {
    unsafe {
        if let Some(func) = syscall {
            func(CG_FS_WRITE as c_int, buffer, len, f)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_FS_FCloseFile(f: c_int) {
    unsafe {
        if let Some(func) = syscall {
            func(CG_FS_FCLOSEFILE as c_int, f);
        }
    }
}

pub extern "C" fn cgi_SendConsoleCommand(text: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SENDCONSOLECOMMAND as c_int, text);
        }
    }
}

pub extern "C" fn cgi_AddCommand(cmdName: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ADDCOMMAND as c_int, cmdName);
        }
    }
}

pub extern "C" fn cgi_SendClientCommand(s: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SENDCLIENTCOMMAND as c_int, s);
        }
    }
}

pub extern "C" fn cgi_UpdateScreen() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UPDATESCREEN as c_int);
        }
    }
}

// RMG BEGIN
pub extern "C" fn cgi_RMG_Init(terrainID: c_int, terrainInfo: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_RMG_INIT as c_int, terrainID, terrainInfo);
        }
    }
}

pub extern "C" fn cgi_CM_RegisterTerrain(terrainInfo: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_REGISTER_TERRAIN as c_int, terrainInfo)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_RE_InitRendererTerrain(terrainInfo: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_RE_INIT_RENDERER_TERRAIN as c_int, terrainInfo);
        }
    }
}
// RMG END

pub extern "C" fn cgi_CM_LoadMap(mapname: *const c_char, subBSP: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_LOADMAP as c_int, mapname, subBSP);
        }
    }
}

pub extern "C" fn cgi_CM_NumInlineModels() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_NUMINLINEMODELS as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_InlineModel(index: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_INLINEMODEL as c_int, index)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_TempBoxModel(mins: *const c_void, maxs: *const c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_TEMPBOXMODEL as c_int, mins, maxs)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_PointContents(p: *const c_void, model: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_POINTCONTENTS as c_int, p, model)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_TransformedPointContents(p: *const c_void, model: c_int, origin: *const c_void, angles: *const c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_TRANSFORMEDPOINTCONTENTS as c_int, p, model, origin, angles)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_BoxTrace(results: *mut c_void, start: *const c_void, end: *const c_void,
                                   mins: *const c_void, maxs: *const c_void,
                                   model: c_int, brushmask: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_BOXTRACE as c_int, results, start, end, mins, maxs, model, brushmask);
        }
    }
}

pub extern "C" fn cgi_CM_TransformedBoxTrace(results: *mut c_void, start: *const c_void, end: *const c_void,
                                              mins: *const c_void, maxs: *const c_void,
                                              model: c_int, brushmask: c_int,
                                              origin: *const c_void, angles: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_TRANSFORMEDBOXTRACE as c_int, results, start, end, mins, maxs, model, brushmask, origin, angles);
        }
    }
}

pub extern "C" fn cgi_CM_MarkFragments(numPoints: c_int, points: *const *const c_void,
                 projection: *const c_void,
                 maxPoints: c_int, pointBuffer: *mut c_void,
                 maxFragments: c_int, fragmentBuffer: *mut c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_MARKFRAGMENTS as c_int, numPoints, points, projection, maxPoints, pointBuffer, maxFragments, fragmentBuffer)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_CM_SnapPVS(origin: *const c_void, buffer: *mut u8) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CM_SNAPPVS as c_int, origin, buffer);
        }
    }
}

pub extern "C" fn cgi_S_StopSounds() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_STOPSOUNDS as c_int);
        }
    }
}

pub extern "C" fn cgi_S_StartSound(origin: *const c_void, entityNum: c_int, entchannel: c_int, sfx: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_STARTSOUND as c_int, origin, entityNum, entchannel, sfx);
        }
    }
}

pub extern "C" fn cgi_AS_ParseSets() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_AS_PARSESETS as c_int);
        }
    }
}

pub extern "C" fn cgi_AS_AddPrecacheEntry(name: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_AS_ADDENTRY as c_int, name);
        }
    }
}

pub extern "C" fn cgi_S_UpdateAmbientSet(name: *const c_char, origin: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_UPDATEAMBIENTSET as c_int, name, origin);
        }
    }
}

pub extern "C" fn cgi_S_AddLocalSet(name: *const c_char, listener_origin: *const c_void, origin: *const c_void, entID: c_int, time: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_ADDLOCALSET as c_int, name, listener_origin, origin, entID, time)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_AS_GetBModelSound(name: *const c_char, stage: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_AS_GETBMODELSOUND as c_int, name, stage)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_S_StartLocalSound(sfx: c_int, channelNum: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_STARTLOCALSOUND as c_int, sfx, channelNum);
        }
    }
}

pub extern "C" fn cgi_S_ClearLoopingSounds() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_CLEARLOOPINGSOUNDS as c_int);
        }
    }
}

pub extern "C" fn cgi_S_AddLoopingSound(entityNum: c_int, origin: *const c_void, velocity: *const c_void, sfx: c_int, chan: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_ADDLOOPINGSOUND as c_int, entityNum, origin, velocity, sfx, chan);
        }
    }
}

pub extern "C" fn cgi_S_UpdateEntityPosition(entityNum: c_int, origin: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_UPDATEENTITYPOSITION as c_int, entityNum, origin);
        }
    }
}

pub extern "C" fn cgi_S_Respatialize(entityNum: c_int, origin: *const c_void, axis: *const *const c_void, inwater: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_RESPATIALIZE as c_int, entityNum, origin, axis, inwater);
        }
    }
}

pub extern "C" fn cgi_S_RegisterSound(sample: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_REGISTERSOUND as c_int, sample)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_S_StartBackgroundTrack(intro: *const c_char, loop_music: *const c_char, bForceStart: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_S_STARTBACKGROUNDTRACK as c_int, intro, loop_music, bForceStart);
        }
    }
}

pub extern "C" fn cgi_S_GetSampleLength(sfx: c_int) -> f32 {
    unsafe {
        if let Some(f) = syscall {
            let result = f(CG_S_GETSAMPLELENGTH as c_int, sfx);
            core::mem::transmute::<c_int, f32>(result)
        } else {
            0.0
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_Start(ff: c_int, clientNum: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_START as c_int, ff, clientNum);
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_Stop(ff: c_int, clientNum: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_STOP as c_int, ff, clientNum);
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_StopAll() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_STOPALL as c_int);
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_Shake(intensity: c_int, duration: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_SHAKE as c_int, intensity, duration);
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_Register(name: *const c_char, channel: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_REGISTER as c_int, name, channel)
        } else {
            0
        }
    }
}

#[cfg(feature = "_IMMERSION")]
pub extern "C" fn cgi_FF_AddLoopingForce(handle: c_int, entNum: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_ADDLOOPINGFORCE as c_int, handle, entNum);
        }
    }
}

#[cfg(not(feature = "_IMMERSION"))]
pub extern "C" fn cgi_FF_StartFX(iFX: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_STARTFX as c_int, iFX);
        }
    }
}

#[cfg(not(feature = "_IMMERSION"))]
pub extern "C" fn cgi_FF_EnsureFX(iFX: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_ENSUREFX as c_int, iFX);
        }
    }
}

#[cfg(not(feature = "_IMMERSION"))]
pub extern "C" fn cgi_FF_StopFX(iFX: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_STOPFX as c_int, iFX);
        }
    }
}

#[cfg(not(feature = "_IMMERSION"))]
pub extern "C" fn cgi_FF_StopAllFX() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_STOPALLFX as c_int);
        }
    }
}

#[cfg(target_os = "xbox")]
pub extern "C" fn cgi_FF_Xbox_Shake(intensity: f32, duration: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_XBOX_SHAKE as c_int, PASSFLOAT(intensity), duration);
        }
    }
}

#[cfg(target_os = "xbox")]
pub extern "C" fn cgi_FF_Xbox_Damage(damage: c_int, xpos: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_FF_XBOX_DAMAGE as c_int, damage, PASSFLOAT(xpos));
        }
    }
}

pub extern "C" fn cgi_R_LoadWorldMap(mapname: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_LOADWORLDMAP as c_int, mapname);
        }
    }
}

pub extern "C" fn cgi_R_RegisterModel(name: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_REGISTERMODEL as c_int, name)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_RegisterSkin(name: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_REGISTERSKIN as c_int, name)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_RegisterShader(name: *const c_char) -> c_int {
    unsafe {
        let hShader = if let Some(f) = syscall {
            f(CG_R_REGISTERSHADER as c_int, name)
        } else {
            0
        };
        assert!(hShader != 0);
        hShader
    }
}

pub extern "C" fn cgi_R_RegisterShaderNoMip(name: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_REGISTERSHADERNOMIP as c_int, name)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_RegisterFont(name: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_REGISTERFONT as c_int, name)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_FONTSTRLENPIXELS as c_int, text, iFontIndex, PASSFLOAT(scale))
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_Font_StrLenChars(text: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_FONTSTRLENCHARS as c_int, text)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_Font_HeightPixels(iFontIndex: c_int, scale: f32) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_FONTHEIGHTPIXELS as c_int, iFontIndex, PASSFLOAT(scale))
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_Language_IsAsian() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_LANGUAGE_ISASIAN as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_Language_UsesSpaces() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_LANGUAGE_USESSPACES as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_AnyLanguage_ReadCharFromString(psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut c_int) -> u32 {
    unsafe {
        if let Some(f) = syscall {
            f(CG_ANYLANGUAGE_READFROMSTRING as c_int, psText, piAdvanceCount, pbIsTrailingPunctuation) as u32
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const f32, setIndex: c_int, iMaxPixelWidth: c_int, scale: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_FONTDRAWSTRING as c_int, ox, oy, text, rgba, setIndex, iMaxPixelWidth, PASSFLOAT(scale));
        }
    }
}

// set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
pub extern "C" fn cgi_R_SetRefractProp(alpha: f32, stretch: f32, prepost: c_int, negate: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_SETREFRACTIONPROP as c_int, PASSFLOAT(alpha), PASSFLOAT(stretch), prepost, negate);
        }
    }
}

pub extern "C" fn cgi_R_ClearScene() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_CLEARSCENE as c_int);
        }
    }
}

pub extern "C" fn cgi_R_AddRefEntityToScene(re: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_ADDREFENTITYTOSCENE as c_int, re);
        }
    }
}

pub extern "C" fn cgi_R_inPVS(p1: *const c_void, p2: *const c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_INPVS as c_int, p1, p2)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_R_GetLighting(origin: *const c_void, ambientLight: *mut c_void, directedLight: *mut c_void, ligthDir: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_GETLIGHTING as c_int, origin, ambientLight, directedLight, ligthDir);
        }
    }
}

pub extern "C" fn cgi_R_AddPolyToScene(hShader: c_int, numVerts: c_int, verts: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_ADDPOLYTOSCENE as c_int, hShader, numVerts, verts);
        }
    }
}

pub extern "C" fn cgi_R_AddLightToScene(org: *const c_void, intensity: f32, r: f32, g: f32, b: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_ADDLIGHTTOSCENE as c_int, org, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b));
        }
    }
}

pub extern "C" fn cgi_R_RenderScene(fd: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_RENDERSCENE as c_int, fd);
        }
    }
}

pub extern "C" fn cgi_R_SetColor(rgba: *const f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_SETCOLOR as c_int, rgba);
        }
    }
}

pub extern "C" fn cgi_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32,
                                        s1: f32, t1: f32, s2: f32, t2: f32, hShader: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_DRAWSTRETCHPIC as c_int, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), hShader);
        }
    }
}

// void	cgi_R_DrawScreenShot( float x, float y, float w, float h){
// 	syscall( CG_R_DRAWSCREENSHOT, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h) );
// }

pub extern "C" fn cgi_R_ModelBounds(model: c_int, mins: *mut c_void, maxs: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_MODELBOUNDS as c_int, model, mins, maxs);
        }
    }
}

pub extern "C" fn cgi_R_LerpTag(tag: *mut c_void, mod_handle: c_int, startFrame: c_int, endFrame: c_int,
                                 frac: f32, tagName: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_LERPTAG as c_int, tag, mod_handle, startFrame, endFrame, PASSFLOAT(frac), tagName);
        }
    }
}

pub extern "C" fn cgi_R_DrawRotatePic(x: f32, y: f32, w: f32, h: f32,
                   s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_DRAWROTATEPIC as c_int, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), PASSFLOAT(a), hShader);
        }
    }
}

pub extern "C" fn cgi_R_DrawRotatePic2(x: f32, y: f32, w: f32, h: f32,
                   s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_DRAWROTATEPIC2 as c_int, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), PASSFLOAT(a), hShader);
        }
    }
}

// linear fogging, with settable range -rww
pub extern "C" fn cgi_R_SetRangeFog(range: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_SETRANGEFOG as c_int, PASSFLOAT(range));
        }
    }
}

pub extern "C" fn cgi_R_LAGoggles() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_LA_GOGGLES as c_int);
        }
    }
}

pub extern "C" fn cgi_R_Scissor(x: f32, y: f32, w: f32, h: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_SCISSOR as c_int, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h));
        }
    }
}

pub extern "C" fn cgi_GetGlconfig(glconfig: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETGLCONFIG as c_int, glconfig);
        }
    }
}

pub extern "C" fn cgi_GetGameState(gamestate: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETGAMESTATE as c_int, gamestate);
        }
    }
}

pub extern "C" fn cgi_GetCurrentSnapshotNumber(snapshotNumber: *mut c_int, serverTime: *mut c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETCURRENTSNAPSHOTNUMBER as c_int, snapshotNumber, serverTime);
        }
    }
}

pub extern "C" fn cgi_GetSnapshot(snapshotNumber: c_int, snapshot: *mut c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETSNAPSHOT as c_int, snapshotNumber, snapshot)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_GetDefaultState(entityIndex: c_int, state: *mut c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETDEFAULTSTATE as c_int, entityIndex, state)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_GetServerCommand(serverCommandNumber: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETSERVERCOMMAND as c_int, serverCommandNumber)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_GetCurrentCmdNumber() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETCURRENTCMDNUMBER as c_int)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_GetUserCmd(cmdNumber: c_int, ucmd: *mut c_void) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_GETUSERCMD as c_int, cmdNumber, ucmd)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_SetUserCmdValue(stateValue: c_int, sensitivityScale: f32, mPitchOverride: f32, mYawOverride: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SETUSERCMDVALUE as c_int, stateValue, PASSFLOAT(sensitivityScale), PASSFLOAT(mPitchOverride), PASSFLOAT(mYawOverride));
        }
    }
}

pub extern "C" fn cgi_SetUserCmdAngles(pitchOverride: f32, yawOverride: f32, rollOverride: f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SETUSERCMDANGLES as c_int, PASSFLOAT(pitchOverride), PASSFLOAT(yawOverride), PASSFLOAT(rollOverride));
        }
    }
}

// Ghoul2 Insert Start
// CG Specific API calls
pub extern "C" fn trap_G2_SetGhoul2ModelIndexes(ghoul2: *mut c_void, modelList: *mut c_void, skinList: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_G2_SETMODELS as c_int, ghoul2, modelList, skinList);
        }
    }
}
// Ghoul2 Insert End

pub extern "C" fn trap_Com_SetOrgAngles(org: *const c_void, angles: *const c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(COM_SETORGANGLES as c_int, org, angles);
        }
    }
}

pub extern "C" fn trap_R_GetLightStyle(style: c_int, color: *mut u8) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_GET_LIGHT_STYLE as c_int, style, color);
        }
    }
}

pub extern "C" fn trap_R_SetLightStyle(style: c_int, color: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_SET_LIGHT_STYLE as c_int, style, color);
        }
    }
}

pub extern "C" fn cgi_R_GetBModelVerts(bmodelIndex: c_int, verts: *mut *mut c_void, normal: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_GET_BMODEL_VERTS as c_int, bmodelIndex, verts, normal);
        }
    }
}

pub extern "C" fn cgi_R_WorldEffectCommand(command: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_R_WORLD_EFFECT_COMMAND as c_int, command);
        }
    }
}

// this returns a handle.  arg0 is the name in the format "idlogo.roq", set arg1 to NULL, alteredstates to qfalse (do not alter gamestate)
pub extern "C" fn trap_CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int, psAudioFile: *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CIN_PLAYCINEMATIC as c_int, arg0, xpos, ypos, width, height, bits, psAudioFile)
        } else {
            0
        }
    }
}

// stops playing the cinematic and ends it.  should always return FMV_EOF
// cinematics must be stopped in reverse order of when they are started
pub extern "C" fn trap_CIN_StopCinematic(handle: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CIN_STOPCINEMATIC as c_int, handle)
        } else {
            0
        }
    }
}

// will run a frame of the cinematic but will not draw it.  Will return FMV_EOF if the end of the cinematic has been reached.
pub extern "C" fn trap_CIN_RunCinematic(handle: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CIN_RUNCINEMATIC as c_int, handle)
        } else {
            0
        }
    }
}

// draws the current frame
pub extern "C" fn trap_CIN_DrawCinematic(handle: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CIN_DRAWCINEMATIC as c_int, handle);
        }
    }
}

// allows you to resize the animation dynamically
pub extern "C" fn trap_CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_CIN_SETEXTENTS as c_int, handle, x, y, w, h);
        }
    }
}

pub extern "C" fn cgi_Z_Malloc(size: c_int, tag: c_int) -> *mut c_void {
    unsafe {
        if let Some(f) = syscall {
            f(CG_Z_MALLOC as c_int, size, tag) as *mut c_void
        } else {
            core::ptr::null_mut()
        }
    }
}

pub extern "C" fn cgi_Z_Free(ptr: *mut c_void) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_Z_FREE as c_int, ptr);
        }
    }
}

pub extern "C" fn cgi_UI_SetActive_Menu(name: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_SETACTIVE_MENU as c_int, name);
        }
    }
}

pub extern "C" fn cgi_UI_Menu_OpenByName(buf: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_MENU_OPENBYNAME as c_int, buf);
        }
    }
}

pub extern "C" fn cgi_UI_Menu_Reset() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_MENU_RESET as c_int);
        }
    }
}

pub extern "C" fn cgi_UI_Menu_New(buf: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_MENU_NEW as c_int, buf);
        }
    }
}

pub extern "C" fn cgi_UI_Parse_Int(value: *mut c_int) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_PARSE_INT as c_int, value);
        }
    }
}

pub extern "C" fn cgi_UI_Parse_String(buf: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_PARSE_STRING as c_int, buf);
        }
    }
}

pub extern "C" fn cgi_UI_Parse_Float(value: *mut f32) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_PARSE_FLOAT as c_int, value);
        }
    }
}

pub extern "C" fn cgi_UI_StartParseSession(menuFile: *const c_char, buf: *mut *const c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_STARTPARSESESSION as c_int, menuFile, buf)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_UI_EndParseSession(buf: *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_ENDPARSESESSION as c_int, buf);
        }
    }
}

pub extern "C" fn cgi_UI_ParseExt(token: *mut *const c_char) {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_PARSEEXT as c_int, token);
        }
    }
}

pub extern "C" fn cgi_UI_MenuCloseAll() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_MENUCLOSE_ALL as c_int);
        }
    }
}

pub extern "C" fn cgi_UI_MenuPaintAll() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_MENUPAINT_ALL as c_int);
        }
    }
}

pub extern "C" fn cgi_UI_String_Init() {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_STRING_INIT as c_int);
        }
    }
}

pub extern "C" fn cgi_UI_GetMenuInfo(menuFile: *const c_char, x: *mut c_int, y: *mut c_int, w: *mut c_int, h: *mut c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_GETMENUINFO as c_int, menuFile, x, y, w, h)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_UI_GetMenuItemInfo(menuFile: *const c_char, itemName: *const c_char, x: *mut c_int, y: *mut c_int, w: *mut c_int, h: *mut c_int, color: *mut f32, background: *mut c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_GETITEMINFO as c_int, menuFile, itemName, x, y, w, h, color, background)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_UI_GetItemText(menuFile: *const c_char, itemName: *const c_char, text: *mut c_char) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_UI_GETITEMTEXT as c_int, menuFile, itemName, text)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_SP_GetStringTextString(text: *const c_char, buffer: *mut c_char, bufferLength: c_int) -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SP_GETSTRINGTEXTSTRING as c_int, text, buffer, bufferLength)
        } else {
            0
        }
    }
}

pub extern "C" fn cgi_EndGame() -> c_int {
    unsafe {
        if let Some(f) = syscall {
            f(CG_SENDCONSOLECOMMAND as c_int, b"cam_disable; disconnect\n".as_ptr() as *const c_char)
        } else {
            0
        }
    }
}
