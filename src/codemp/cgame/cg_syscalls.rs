// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_syscalls.c -- this file is only included when building a dll
// cg_syscalls.asm is included instead when building a qvm

// Translate of #include "cg_local.h" - external to this module
// See cg_local module for shared definitions

use core::ffi::{c_int, c_char, c_void};

// Syscall function pointer type: returns int, takes int and variadic arguments
// Initialized to -1 (cast as pointer), set via dllEntry
static mut syscall: Option<extern "C" fn(c_int, ...) -> c_int> = None;

// Precondition: syscall will be set by dllEntry before any trap functions are called
// The -1 initialization in original C is an unsafe pattern; we use None to mark uninitialized.
// In actual use, dllEntry replaces this with the real function pointer.

pub fn dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int) {
	unsafe {
		syscall = Some(syscallptr);
	}
}

// PASSFLOAT: reinterpret float bits as int for syscall marshaling
#[inline]
fn PASSFLOAT(x: f32) -> c_int {
	let floatTemp: f32 = x;
	unsafe { *((&floatTemp as *const f32) as *const c_int) }
}

pub fn trap_Print(fmt: *const c_char) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_PRINT, fmt as *const c_void);
		}
	}
}

pub fn trap_Error(fmt: *const c_char) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_ERROR, fmt as *const c_void);
		}
	}
}

pub fn trap_Milliseconds() -> c_int {
	unsafe {
		if let Some(f) = syscall {
			f(CG_MILLISECONDS)
		} else {
			0
		}
	}
}

//rww - precision timer funcs... -ALWAYS- call end after start with supplied ptr, or you'll get a nasty memory leak.
//not that you should be using these outside of debug anyway.. because you shouldn't be. So don't.

//Start should be suppled with a pointer to an empty pointer (e.g. void *blah; trap_PrecisionTimer_Start(&blah);),
//the empty pointer will be filled with an exe address to our timer (this address means nothing in vm land however).
//You must pass this pointer back unmodified to the timer end func.
pub fn trap_PrecisionTimer_Start(theNewTimer: *mut *mut c_void) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_PRECISIONTIMER_START, theNewTimer as *mut c_void);
		}
	}
}

//If you're using the above example, the appropriate call for this is int result = trap_PrecisionTimer_End(blah);
pub fn trap_PrecisionTimer_End(theTimer: *mut c_void) -> c_int {
	unsafe {
		if let Some(f) = syscall {
			f(CG_PRECISIONTIMER_END, theTimer)
		} else {
			0
		}
	}
}

pub fn trap_Cvar_Register(vmCvar: *mut c_void, varName: *const c_char, defaultValue: *const c_char, flags: c_int) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_CVAR_REGISTER, vmCvar, varName as *mut c_void, defaultValue as *mut c_void, flags);
		}
	}
}

pub fn trap_Cvar_Update(vmCvar: *mut c_void) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_CVAR_UPDATE, vmCvar);
		}
	}
}

pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_CVAR_SET, var_name as *mut c_void, value as *mut c_void);
		}
	}
}

pub fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_CVAR_VARIABLESTRINGBUFFER, var_name as *mut c_void, buffer as *mut c_void, bufsize);
		}
	}
}

pub fn trap_Cvar_GetHiddenVarValue(name: *const c_char) -> c_int {
	unsafe {
		if let Some(f) = syscall {
			f(CG_CVAR_GETHIDDENVALUE, name as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_Argc() -> c_int {
	unsafe {
		if let Some(f) = syscall {
			f(CG_ARGC)
		} else {
			0
		}
	}
}

pub fn trap_Argv(n: c_int, buffer: *mut c_char, bufferLength: c_int) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_ARGV, n, buffer as *mut c_void, bufferLength);
		}
	}
}

pub fn trap_Args(buffer: *mut c_char, bufferLength: c_int) {
	unsafe {
		if let Some(f) = syscall {
			f(CG_ARGS, buffer as *mut c_void, bufferLength);
		}
	}
}

pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut c_void, mode: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FS_FOPENFILE, qpath as *mut c_void, f, mode)
		} else {
			0
		}
	}
}

pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FS_READ, buffer, len, f);
		}
	}
}

pub fn trap_FS_Write(buffer: *const c_void, len: c_int, f: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FS_WRITE, buffer as *mut c_void, len, f);
		}
	}
}

pub fn trap_FS_FCloseFile(f: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FS_FCLOSEFILE, f);
		}
	}
}

pub fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FS_GETFILELIST, path as *mut c_void, extension as *mut c_void, listbuf as *mut c_void, bufsize)
		} else {
			0
		}
	}
}

pub fn trap_SendConsoleCommand(text: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SENDCONSOLECOMMAND, text as *mut c_void);
		}
	}
}

pub fn trap_AddCommand(cmdName: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ADDCOMMAND, cmdName as *mut c_void);
		}
	}
}

pub fn trap_RemoveCommand(cmdName: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_REMOVECOMMAND, cmdName as *mut c_void);
		}
	}
}

pub fn trap_SendClientCommand(s: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SENDCLIENTCOMMAND, s as *mut c_void);
		}
	}
}

pub fn trap_UpdateScreen() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_UPDATESCREEN);
		}
	}
}

pub fn trap_CM_LoadMap(mapname: *const c_char, SubBSP: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_LOADMAP, mapname as *mut c_void, SubBSP);
		}
	}
}

pub fn trap_CM_NumInlineModels() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_NUMINLINEMODELS)
		} else {
			0
		}
	}
}

pub fn trap_CM_InlineModel(index: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_INLINEMODEL, index)
		} else {
			0
		}
	}
}

pub fn trap_CM_TempBoxModel(mins: *const [f32; 3], maxs: *const [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_TEMPBOXMODEL, mins as *mut c_void, maxs as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_CM_TempCapsuleModel(mins: *const [f32; 3], maxs: *const [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_TEMPCAPSULEMODEL, mins as *mut c_void, maxs as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_CM_PointContents(p: *const [f32; 3], model: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_POINTCONTENTS, p as *mut c_void, model)
		} else {
			0
		}
	}
}

pub fn trap_CM_TransformedPointContents(p: *const [f32; 3], model: c_int, origin: *const [f32; 3], angles: *const [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_TRANSFORMEDPOINTCONTENTS, p as *mut c_void, model, origin as *mut c_void, angles as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_CM_BoxTrace(results: *mut c_void, start: *const [f32; 3], end: *const [f32; 3],
						  mins: *const [f32; 3], maxs: *const [f32; 3],
						  model: c_int, brushmask: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_BOXTRACE, results, start as *mut c_void, end as *mut c_void, mins as *mut c_void, maxs as *mut c_void, model, brushmask);
		}
	}
}

pub fn trap_CM_CapsuleTrace(results: *mut c_void, start: *const [f32; 3], end: *const [f32; 3],
						  mins: *const [f32; 3], maxs: *const [f32; 3],
						  model: c_int, brushmask: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_CAPSULETRACE, results, start as *mut c_void, end as *mut c_void, mins as *mut c_void, maxs as *mut c_void, model, brushmask);
		}
	}
}

pub fn trap_CM_TransformedBoxTrace(results: *mut c_void, start: *const [f32; 3], end: *const [f32; 3],
						  mins: *const [f32; 3], maxs: *const [f32; 3],
						  model: c_int, brushmask: c_int,
						  origin: *const [f32; 3], angles: *const [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_TRANSFORMEDBOXTRACE, results, start as *mut c_void, end as *mut c_void, mins as *mut c_void, maxs as *mut c_void, model, brushmask, origin as *mut c_void, angles as *mut c_void);
		}
	}
}

pub fn trap_CM_TransformedCapsuleTrace(results: *mut c_void, start: *const [f32; 3], end: *const [f32; 3],
						  mins: *const [f32; 3], maxs: *const [f32; 3],
						  model: c_int, brushmask: c_int,
						  origin: *const [f32; 3], angles: *const [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_TRANSFORMEDCAPSULETRACE, results, start as *mut c_void, end as *mut c_void, mins as *mut c_void, maxs as *mut c_void, model, brushmask, origin as *mut c_void, angles as *mut c_void);
		}
	}
}

pub fn trap_CM_MarkFragments(numPoints: c_int, points: *const *const [f32; 3],
				projection: *const [f32; 3],
				maxPoints: c_int, pointBuffer: *mut [f32; 3],
				maxFragments: c_int, fragmentBuffer: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_MARKFRAGMENTS, numPoints, points as *mut c_void, projection as *mut c_void, maxPoints, pointBuffer as *mut c_void, maxFragments, fragmentBuffer)
		} else {
			0
		}
	}
}

pub fn trap_S_GetVoiceVolume(entityNum: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_GETVOICEVOLUME, entityNum)
		} else {
			0
		}
	}
}

pub fn trap_S_MuteSound(entityNum: c_int, entchannel: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_MUTESOUND, entityNum, entchannel);
		}
	}
}

pub fn trap_S_StartSound(origin: *mut [f32; 3], entityNum: c_int, entchannel: c_int, sfx: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_STARTSOUND, origin as *mut c_void, entityNum, entchannel, sfx);
		}
	}
}

pub fn trap_S_StartLocalSound(sfx: c_int, channelNum: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_STARTLOCALSOUND, sfx, channelNum);
		}
	}
}

pub fn trap_S_ClearLoopingSounds() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_CLEARLOOPINGSOUNDS);
		}
	}
}

pub fn trap_S_AddLoopingSound(entityNum: c_int, origin: *const [f32; 3], velocity: *const [f32; 3], sfx: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_ADDLOOPINGSOUND, entityNum, origin as *mut c_void, velocity as *mut c_void, sfx);
		}
	}
}

pub fn trap_S_UpdateEntityPosition(entityNum: c_int, origin: *const [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_UPDATEENTITYPOSITION, entityNum, origin as *mut c_void);
		}
	}
}

pub fn trap_S_AddRealLoopingSound(entityNum: c_int, origin: *const [f32; 3], velocity: *const [f32; 3], sfx: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_ADDREALLOOPINGSOUND, entityNum, origin as *mut c_void, velocity as *mut c_void, sfx);
		}
	}
}

pub fn trap_S_StopLoopingSound(entityNum: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_STOPLOOPINGSOUND, entityNum);
		}
	}
}

pub fn trap_S_Respatialize(entityNum: c_int, origin: *const [f32; 3], axis: *mut [[f32; 3]; 3], inwater: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_RESPATIALIZE, entityNum, origin as *mut c_void, axis as *mut c_void, inwater);
		}
	}
}

pub fn trap_S_ShutUp(shutUpFactor: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_SHUTUP, shutUpFactor);
		}
	}
}

pub fn trap_S_RegisterSound(sample: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_REGISTERSOUND, sample as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_S_StartBackgroundTrack(intro: *const c_char, loop_track: *const c_char, bReturnWithoutStarting: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_STARTBACKGROUNDTRACK, intro as *mut c_void, loop_track as *mut c_void, bReturnWithoutStarting);
		}
	}
}

pub fn trap_S_UpdateAmbientSet(name: *const c_char, origin: *mut [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_UPDATEAMBIENTSET, name as *mut c_void, origin as *mut c_void);
		}
	}
}

pub fn trap_AS_ParseSets() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_AS_PARSESETS);
		}
	}
}

pub fn trap_AS_AddPrecacheEntry(name: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_AS_ADDPRECACHEENTRY, name as *mut c_void);
		}
	}
}

pub fn trap_S_AddLocalSet(name: *const c_char, listener_origin: *mut [f32; 3], origin: *mut [f32; 3], entID: c_int, time: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_ADDLOCALSET, name as *mut c_void, listener_origin as *mut c_void, origin as *mut c_void, entID, time)
		} else {
			0
		}
	}
}

pub fn trap_AS_GetBModelSound(name: *const c_char, stage: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_AS_GETBMODELSOUND, name as *mut c_void, stage)
		} else {
			0
		}
	}
}

pub fn trap_R_LoadWorldMap(mapname: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_LOADWORLDMAP, mapname as *mut c_void);
		}
	}
}

pub fn trap_R_RegisterModel(name: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REGISTERMODEL, name as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_RegisterSkin(name: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REGISTERSKIN, name as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_RegisterShader(name: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REGISTERSHADER, name as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REGISTERSHADERNOMIP, name as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_RegisterFont(fontName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REGISTERFONT, fontName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_FONT_STRLENPIXELS, text as *mut c_void, iFontIndex, PASSFLOAT(scale))
		} else {
			0
		}
	}
}

pub fn trap_R_Font_StrLenChars(text: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_FONT_STRLENCHARS, text as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_Font_HeightPixels(iFontIndex: c_int, scale: f32) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_FONT_STRHEIGHTPIXELS, iFontIndex, PASSFLOAT(scale))
		} else {
			0
		}
	}
}

pub fn trap_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const f32, setIndex: c_int, iCharLimit: c_int, scale: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_FONT_DRAWSTRING, ox, oy, text as *mut c_void, rgba as *mut c_void, setIndex, iCharLimit, PASSFLOAT(scale));
		}
	}
}

pub fn trap_Language_IsAsian() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_LANGUAGE_ISASIAN)
		} else {
			0
		}
	}
}

pub fn trap_Language_UsesSpaces() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_LANGUAGE_USESSPACES)
		} else {
			0
		}
	}
}

pub fn trap_AnyLanguage_ReadCharFromString(psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ANYLANGUAGE_READCHARFROMSTRING, psText as *mut c_void, piAdvanceCount as *mut c_void, pbIsTrailingPunctuation as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_ClearScene() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_CLEARSCENE);
		}
	}
}

pub fn trap_R_ClearDecals() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_CLEARDECALS);
		}
	}
}

pub fn trap_R_AddRefEntityToScene(re: *const c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDREFENTITYTOSCENE, re as *mut c_void);
		}
	}
}

pub fn trap_R_AddPolyToScene(hShader: c_int, numVerts: c_int, verts: *const c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDPOLYTOSCENE, hShader, numVerts, verts as *mut c_void);
		}
	}
}

pub fn trap_R_AddPolysToScene(hShader: c_int, numVerts: c_int, verts: *const c_void, num: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDPOLYSTOSCENE, hShader, numVerts, verts as *mut c_void, num);
		}
	}
}

pub fn trap_R_AddDecalToScene(shader: c_int, origin: *const [f32; 3], dir: *const [f32; 3], orientation: f32, r: f32, g: f32, b: f32, a: f32, alphaFade: c_int, radius: f32, temporary: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDDECALTOSCENE, shader, origin as *mut c_void, dir as *mut c_void, PASSFLOAT(orientation), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b), PASSFLOAT(a), alphaFade, PASSFLOAT(radius), temporary);
		}
	}
}

pub fn trap_R_LightForPoint(point: *mut [f32; 3], ambientLight: *mut [f32; 3], directedLight: *mut [f32; 3], lightDir: *mut [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_LIGHTFORPOINT, point as *mut c_void, ambientLight as *mut c_void, directedLight as *mut c_void, lightDir as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_AddLightToScene(org: *const [f32; 3], intensity: f32, r: f32, g: f32, b: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDLIGHTTOSCENE, org as *mut c_void, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b));
		}
	}
}

pub fn trap_R_AddAdditiveLightToScene(org: *const [f32; 3], intensity: f32, r: f32, g: f32, b: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_ADDADDITIVELIGHTTOSCENE, org as *mut c_void, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b));
		}
	}
}

pub fn trap_R_RenderScene(fd: *const c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_RENDERSCENE, fd as *mut c_void);
		}
	}
}

pub fn trap_R_SetColor(rgba: *const f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_SETCOLOR, rgba as *mut c_void);
		}
	}
}

pub fn trap_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32,
							   s1: f32, t1: f32, s2: f32, t2: f32, hShader: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_DRAWSTRETCHPIC, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), hShader);
		}
	}
}

pub fn trap_R_ModelBounds(model: c_int, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_MODELBOUNDS, model, mins as *mut c_void, maxs as *mut c_void);
		}
	}
}

pub fn trap_R_LerpTag(tag: *mut c_void, mod_handle: c_int, startFrame: c_int, endFrame: c_int,
					   frac: f32, tagName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_LERPTAG, tag, mod_handle, startFrame, endFrame, PASSFLOAT(frac), tagName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_R_DrawRotatePic(x: f32, y: f32, w: f32, h: f32,
				   s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_DRAWROTATEPIC, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), PASSFLOAT(a), hShader);
		}
	}
}

pub fn trap_R_DrawRotatePic2(x: f32, y: f32, w: f32, h: f32,
				   s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_DRAWROTATEPIC2, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), PASSFLOAT(a), hShader);
		}
	}
}

//linear fogging, with settable range -rww
pub fn trap_R_SetRangeFog(range: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_SETRANGEFOG, PASSFLOAT(range));
		}
	}
}

//set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
pub fn trap_R_SetRefractProp(alpha: f32, stretch: f32, prepost: c_int, negate: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_SETREFRACTIONPROP, PASSFLOAT(alpha), PASSFLOAT(stretch), prepost, negate);
		}
	}
}

pub fn trap_R_RemapShader(oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_REMAP_SHADER, oldShader as *mut c_void, newShader as *mut c_void, timeOffset as *mut c_void);
		}
	}
}

pub fn trap_R_GetLightStyle(style: c_int, color: *mut c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_GET_LIGHT_STYLE, style, color as *mut c_void);
		}
	}
}

pub fn trap_R_SetLightStyle(style: c_int, color: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_SET_LIGHT_STYLE, style, color);
		}
	}
}

pub fn trap_R_GetBModelVerts(bmodelIndex: c_int, verts: *mut *mut [f32; 3], normal: *mut [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_GET_BMODEL_VERTS, bmodelIndex, verts as *mut c_void, normal as *mut c_void);
		}
	}
}

pub fn trap_R_GetDistanceCull(f: *mut f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_GETDISTANCECULL, f as *mut c_void);
		}
	}
}

//get screen resolution -rww
pub fn trap_R_GetRealRes(w: *mut c_int, h: *mut c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_GETREALRES, w as *mut c_void, h as *mut c_void);
		}
	}
}


//automap elevation setting -rww
pub fn trap_R_AutomapElevAdj(newHeight: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_AUTOMAPELEVADJ, PASSFLOAT(newHeight));
		}
	}
}

//initialize automap -rww
pub fn trap_R_InitWireframeAutomap() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_INITWIREFRAMEAUTO)
		} else {
			0
		}
	}
}

pub fn trap_FX_AddLine(start: *const [f32; 3], end: *const [f32; 3], size1: f32, size2: f32, sizeParm: f32,
									alpha1: f32, alpha2: f32, alphaParm: f32,
									sRGB: *const [f32; 3], eRGB: *const [f32; 3], rgbParm: f32,
									killTime: c_int, shader: c_int, flags: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDLINE, start as *mut c_void, end as *mut c_void, PASSFLOAT(size1), PASSFLOAT(size2), PASSFLOAT(sizeParm),
									PASSFLOAT(alpha1), PASSFLOAT(alpha2), PASSFLOAT(alphaParm),
									sRGB as *mut c_void, eRGB as *mut c_void, PASSFLOAT(rgbParm),
									killTime, shader, flags);
		}
	}
}

pub fn trap_GetGlconfig(glconfig: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETGLCONFIG, glconfig);
		}
	}
}

pub fn trap_GetGameState(gamestate: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETGAMESTATE, gamestate);
		}
	}
}

pub fn trap_GetCurrentSnapshotNumber(snapshotNumber: *mut c_int, serverTime: *mut c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETCURRENTSNAPSHOTNUMBER, snapshotNumber as *mut c_void, serverTime as *mut c_void);
		}
	}
}

pub fn trap_GetSnapshot(snapshotNumber: c_int, snapshot: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETSNAPSHOT, snapshotNumber, snapshot)
		} else {
			0
		}
	}
}

pub fn trap_GetDefaultState(entityIndex: c_int, state: *mut c_void) -> c_int {
	//rwwRMG - added [NEWTRAP]
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETDEFAULTSTATE, entityIndex, state)
		} else {
			0
		}
	}
}

pub fn trap_GetServerCommand(serverCommandNumber: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETSERVERCOMMAND, serverCommandNumber)
		} else {
			0
		}
	}
}

pub fn trap_GetCurrentCmdNumber() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETCURRENTCMDNUMBER)
		} else {
			0
		}
	}
}

pub fn trap_GetUserCmd(cmdNumber: c_int, ucmd: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GETUSERCMD, cmdNumber, ucmd)
		} else {
			0
		}
	}
}

pub fn trap_SetUserCmdValue(stateValue: c_int, sensitivityScale: f32, mPitchOverride: f32, mYawOverride: f32, mSensitivityOverride: f32, fpSel: c_int, invenSel: c_int, fighterControls: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SETUSERCMDVALUE, stateValue, PASSFLOAT(sensitivityScale), PASSFLOAT(mPitchOverride), PASSFLOAT(mYawOverride), PASSFLOAT(mSensitivityOverride), fpSel, invenSel, fighterControls);
		}
	}
}

pub fn trap_SetClientForceAngle(time: c_int, angle: *mut [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SETCLIENTFORCEANGLE, time, angle as *mut c_void);
		}
	}
}

pub fn trap_SetClientTurnExtent(turnAdd: f32, turnSub: f32, turnTime: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SETCLIENTTURNEXTENT, PASSFLOAT(turnAdd), PASSFLOAT(turnSub), turnTime);
		}
	}
}

pub fn trap_OpenUIMenu(menuID: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_OPENUIMENU, menuID);
		}
	}
}

pub fn testPrintInt(string: *mut c_char, i: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_TESTPRINTINT, string as *mut c_void, i);
		}
	}
}

pub fn testPrintFloat(string: *mut c_char, f: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_TESTPRINTFLOAT, string as *mut c_void, PASSFLOAT(f));
		}
	}
}

pub fn trap_MemoryRemaining() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_MEMORY_REMAINING)
		} else {
			0
		}
	}
}

pub fn trap_Key_IsDown(keynum: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_KEY_ISDOWN, keynum)
		} else {
			0
		}
	}
}

pub fn trap_Key_GetCatcher() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_KEY_GETCATCHER)
		} else {
			0
		}
	}
}

pub fn trap_Key_SetCatcher(catcher: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_KEY_SETCATCHER, catcher);
		}
	}
}

pub fn trap_Key_GetKey(binding: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_KEY_GETKEY, binding as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_PC_AddGlobalDefine(define: *mut c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_ADD_GLOBAL_DEFINE, define as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_PC_LoadSource(filename: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_LOAD_SOURCE, filename as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_PC_FreeSource(handle: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_FREE_SOURCE, handle)
		} else {
			0
		}
	}
}

pub fn trap_PC_ReadToken(handle: c_int, pc_token: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_READ_TOKEN, handle, pc_token)
		} else {
			0
		}
	}
}

pub fn trap_PC_SourceFileAndLine(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_SOURCE_FILE_AND_LINE, handle, filename as *mut c_void, line as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_PC_LoadGlobalDefines(filename: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_LOAD_GLOBAL_DEFINES, filename as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_PC_RemoveAllGlobalDefines() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_PC_REMOVE_ALL_GLOBAL_DEFINES);
		}
	}
}

pub fn trap_S_StopBackgroundTrack() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_S_STOPBACKGROUNDTRACK);
		}
	}
}

pub fn trap_RealTime(qtime: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_REAL_TIME, qtime)
		} else {
			0
		}
	}
}

pub fn trap_SnapVector(v: *mut f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SNAPVECTOR, v as *mut c_void);
		}
	}
}

// this returns a handle.  arg0 is the name in the format "idlogo.roq", set arg1 to NULL, alteredstates to qfalse (do not alter gamestate)
pub fn trap_CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CIN_PLAYCINEMATIC, arg0 as *mut c_void, xpos, ypos, width, height, bits)
		} else {
			0
		}
	}
}

// stops playing the cinematic and ends it.  should always return FMV_EOF
// cinematics must be stopped in reverse order of when they are started
pub fn trap_CIN_StopCinematic(handle: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CIN_STOPCINEMATIC, handle)
		} else {
			0
		}
	}
}


// will run a frame of the cinematic but will not draw it.  Will return FMV_EOF if the end of the cinematic has been reached.
pub fn trap_CIN_RunCinematic(handle: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CIN_RUNCINEMATIC, handle)
		} else {
			0
		}
	}
}


// draws the current frame
pub fn trap_CIN_DrawCinematic(handle: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CIN_DRAWCINEMATIC, handle);
		}
	}
}


// allows you to resize the animation dynamically
pub fn trap_CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CIN_SETEXTENTS, handle, x, y, w, h);
		}
	}
}

pub fn trap_GetEntityToken(buffer: *mut c_char, bufferSize: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_GET_ENTITY_TOKEN, buffer as *mut c_void, bufferSize)
		} else {
			0
		}
	}
}

pub fn trap_R_inPVS(p1: *const [f32; 3], p2: *const [f32; 3], mask: *mut u8) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_INPVS, p1 as *mut c_void, p2 as *mut c_void, mask as *mut c_void)
		} else {
			0
		}
	}
}


pub fn trap_FX_RegisterEffect(file: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_REGISTER_EFFECT, file as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_FX_PlayEffect(file: *const c_char, org: *mut [f32; 3], fwd: *mut [f32; 3], vol: c_int, rad: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_EFFECT, file as *mut c_void, org as *mut c_void, fwd as *mut c_void, vol, rad);
		}
	}
}

pub fn trap_FX_PlayEntityEffect(file: *const c_char, org: *mut [f32; 3],
						vec3_t_axis: *mut [[f32; 3]; 3], boltInfo: c_int, entNum: c_int, vol: c_int, rad: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_ENTITY_EFFECT, file as *mut c_void, org as *mut c_void, vec3_t_axis as *mut c_void, boltInfo, entNum, vol, rad);
		}
	}
}

pub fn trap_FX_PlayEffectID(id: c_int, org: *mut [f32; 3], fwd: *mut [f32; 3], vol: c_int, rad: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_EFFECT_ID, id, org as *mut c_void, fwd as *mut c_void, vol, rad);
		}
	}
}

pub fn trap_FX_PlayPortalEffectID(id: c_int, org: *mut [f32; 3], fwd: *mut [f32; 3], vol: c_int, rad: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_PORTAL_EFFECT_ID, id, org as *mut c_void, fwd as *mut c_void);
		}
	}
}

pub fn trap_FX_PlayEntityEffectID(id: c_int, org: *mut [f32; 3],
						vec3_t_axis: *mut [[f32; 3]; 3], boltInfo: c_int, entNum: c_int, vol: c_int, rad: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_ENTITY_EFFECT_ID, id, org as *mut c_void, vec3_t_axis as *mut c_void, boltInfo, entNum, vol, rad);
		}
	}
}

pub fn trap_FX_PlayBoltedEffectID(id: c_int, org: *mut [f32; 3],
						ghoul2: *mut c_void, boltNum: c_int, entNum: c_int, modelNum: c_int, iLooptime: c_int, isRelative: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_PLAY_BOLTED_EFFECT_ID, id, org as *mut c_void, ghoul2, boltNum, entNum, modelNum, iLooptime, isRelative);
		}
	}
}

pub fn trap_FX_AddScheduledEffects(skyPortal: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADD_SCHEDULED_EFFECTS, skyPortal);
		}
	}
}

pub fn trap_FX_Draw2DEffects(screenXScale: f32, screenYScale: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_DRAW_2D_EFFECTS, PASSFLOAT(screenXScale), PASSFLOAT(screenYScale));
		}
	}
}

pub fn trap_FX_InitSystem(refdef: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_INIT_SYSTEM, refdef)
		} else {
			0
		}
	}
}

pub fn trap_FX_SetRefDef(refdef: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_SET_REFDEF, refdef);
		}
	}
}

pub fn trap_FX_FreeSystem() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_FREE_SYSTEM)
		} else {
			0
		}
	}
}

pub fn trap_FX_Reset() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_RESET);
		}
	}
}

pub fn trap_FX_AdjustTime(time: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADJUST_TIME, time);
		}
	}
}


pub fn trap_FX_AddPoly(p: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDPOLY, p);
		}
	}
}

pub fn trap_FX_AddBezier(p: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDBEZIER, p);
		}
	}
}

pub fn trap_FX_AddPrimitive(p: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDPRIMITIVE, p);
		}
	}
}

pub fn trap_FX_AddSprite(p: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDSPRITE, p);
		}
	}
}

pub fn trap_FX_AddElectricity(p: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_FX_ADDELECTRICITY, p);
		}
	}
}

//void trap_SP_Print(const unsigned ID, byte *Data)
//{
//	syscall( CG_SP_PRINT, ID, Data);
//}

pub fn trap_SP_GetStringTextString(text: *const c_char, buffer: *mut c_char, bufferLength: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SP_GETSTRINGTEXTSTRING, text as *mut c_void, buffer as *mut c_void, bufferLength)
		} else {
			0
		}
	}
}

pub fn trap_ROFF_Clean() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ROFF_CLEAN)
		} else {
			0
		}
	}
}

pub fn trap_ROFF_UpdateEntities() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ROFF_UPDATE_ENTITIES);
		}
	}
}

pub fn trap_ROFF_Cache(file: *mut c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ROFF_CACHE, file as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_ROFF_Play(entID: c_int, roffID: c_int, doTranslation: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ROFF_PLAY, entID, roffID, doTranslation)
		} else {
			0
		}
	}
}

pub fn trap_ROFF_Purge_Ent(entID: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_ROFF_PURGE_ENT, entID)
		} else {
			0
		}
	}
}


//rww - dynamic vm memory allocation!
pub fn trap_TrueMalloc(ptr: *mut *mut c_void, size: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_TRUEMALLOC, ptr as *mut c_void, size);
		}
	}
}

pub fn trap_TrueFree(ptr: *mut *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_TRUEFREE, ptr as *mut c_void);
		}
	}
}

/*
Ghoul2 Insert Start
*/
// CG Specific API calls
pub fn trap_G2_ListModelSurfaces(ghlInfo: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_LISTSURFACES, ghlInfo);
		}
	}
}

pub fn trap_G2_ListModelBones(ghlInfo: *mut c_void, frame: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_LISTBONES, ghlInfo, frame);
		}
	}
}

pub fn trap_G2_SetGhoul2ModelIndexes(ghoul2: *mut c_void, modelList: *mut c_int, skinList: *mut c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETMODELS, ghoul2, modelList as *mut c_void, skinList as *mut c_void);
		}
	}
}

pub fn trap_G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_HAVEWEGHOULMODELS, ghoul2)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
								const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETBOLT, ghoul2, modelIndex, boltIndex, matrix, const_angles as *mut c_void, const_position as *mut c_void, frameNum, modelList as *mut c_void, scale as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetBoltMatrix_NoReconstruct(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
								const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
	//Same as above but force it to not reconstruct the skeleton before getting the bolt position
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETBOLT_NOREC, ghoul2, modelIndex, boltIndex, matrix, const_angles as *mut c_void, const_position as *mut c_void, frameNum, modelList as *mut c_void, scale as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
								const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
	//Same as above but force it to not reconstruct the skeleton before getting the bolt position
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETBOLT_NOREC_NOROT, ghoul2, modelIndex, boltIndex, matrix, const_angles as *mut c_void, const_position as *mut c_void, frameNum, modelList as *mut c_void, scale as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_InitGhoul2Model(ghoul2Ptr: *mut *mut c_void, fileName: *const c_char, modelIndex: c_int, customSkin: c_int,
						  customShader: c_int, modelFlags: c_int, lodBias: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_INITGHOUL2MODEL, ghoul2Ptr as *mut c_void, fileName as *mut c_void, modelIndex, customSkin, customShader, modelFlags, lodBias)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetSkin(ghoul2: *mut c_void, modelIndex: c_int, customSkin: c_int, renderSkin: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETSKIN, ghoul2, modelIndex, customSkin, renderSkin)
		} else {
			0
		}
	}
}

pub fn trap_G2API_CollisionDetect(
	collRecMap: *mut c_void,
	ghoul2: *mut c_void,
	const_angles: *const [f32; 3],
	const_position: *const [f32; 3],
	frameNumber: c_int,
	entNum: c_int,
	const_rayStart: *const [f32; 3],
	const_rayEnd: *const [f32; 3],
	const_scale: *const [f32; 3],
	traceFlags: c_int,
	useLod: c_int,
	fRadius: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_COLLISIONDETECT, collRecMap, ghoul2, const_angles as *mut c_void, const_position as *mut c_void, frameNumber, entNum, const_rayStart as *mut c_void, const_rayEnd as *mut c_void, const_scale as *mut c_void, traceFlags, useLod, PASSFLOAT(fRadius));
		}
	}
}

pub fn trap_G2API_CollisionDetectCache(
	collRecMap: *mut c_void,
	ghoul2: *mut c_void,
	const_angles: *const [f32; 3],
	const_position: *const [f32; 3],
	frameNumber: c_int,
	entNum: c_int,
	const_rayStart: *const [f32; 3],
	const_rayEnd: *const [f32; 3],
	const_scale: *const [f32; 3],
	traceFlags: c_int,
	useLod: c_int,
	fRadius: f32) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_COLLISIONDETECTCACHE, collRecMap, ghoul2, const_angles as *mut c_void, const_position as *mut c_void, frameNumber, entNum, const_rayStart as *mut c_void, const_rayEnd as *mut c_void, const_scale as *mut c_void, traceFlags, useLod, PASSFLOAT(fRadius));
		}
	}
}

pub fn trap_G2API_CleanGhoul2Models(ghoul2Ptr: *mut *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_CLEANMODELS, ghoul2Ptr as *mut c_void);
		}
	}
}

pub fn trap_G2API_SetBoneAngles(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, const_angles: *const [f32; 3], flags: c_int,
								const_up: c_int, const_right: c_int, const_forward: c_int, modelList: *mut c_int,
								blendTime: c_int, currentTime: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ANGLEOVERRIDE, ghoul2, modelIndex, boneName as *mut c_void, const_angles as *mut c_void, flags, const_up, const_right, const_forward, modelList as *mut c_void, blendTime, currentTime)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetBoneAnim(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int,
							  flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_PLAYANIM, ghoul2, modelIndex, boneName as *mut c_void, startFrame, endFrame, flags, PASSFLOAT(animSpeed), currentTime, PASSFLOAT(setFrame), blendTime)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32,
						   startFrame: *mut c_int, endFrame: *mut c_int, flags: *mut c_int, animSpeed: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETBONEANIM, ghoul2, boneName as *mut c_void, currentTime, currentFrame as *mut c_void, startFrame as *mut c_void, endFrame as *mut c_void, flags as *mut c_void, animSpeed as *mut c_void, modelList as *mut c_void, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetBoneFrame(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETBONEFRAME, ghoul2, boneName as *mut c_void, currentTime, currentFrame as *mut c_void, modelList as *mut c_void, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetGLAName(ghoul2: *mut c_void, modelIndex: c_int, fillBuf: *mut c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETGLANAME, ghoul2, modelIndex, fillBuf as *mut c_void);
		}
	}
}

pub fn trap_G2API_CopyGhoul2Instance(g2From: *mut c_void, g2To: *mut c_void, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_COPYGHOUL2INSTANCE, g2From, g2To, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_CopySpecificGhoul2Model(g2From: *mut c_void, modelFrom: c_int, g2To: *mut c_void, modelTo: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_COPYSPECIFICGHOUL2MODEL, g2From, modelFrom, g2To, modelTo);
		}
	}
}

pub fn trap_G2API_DuplicateGhoul2Instance(g2From: *mut c_void, g2To: *mut *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_DUPLICATEGHOUL2INSTANCE, g2From, g2To as *mut c_void);
		}
	}
}

pub fn trap_G2API_HasGhoul2ModelOnIndex(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_HASGHOUL2MODELONINDEX, ghlInfo, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_RemoveGhoul2Model(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_REMOVEGHOUL2MODEL, ghlInfo, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SkinlessModel(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SKINLESSMODEL, ghlInfo, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetNumGoreMarks(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETNUMGOREMARKS, ghlInfo, modelIndex)
		} else {
			0
		}
	}
}

pub fn trap_G2API_AddSkinGore(ghlInfo: *mut c_void, gore: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ADDSKINGORE, ghlInfo, gore);
		}
	}
}

pub fn trap_G2API_ClearSkinGore(ghlInfo: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_CLEARSKINGORE, ghlInfo);
		}
	}
}

pub fn trap_G2API_Ghoul2Size(ghlInfo: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SIZE, ghlInfo)
		} else {
			0
		}
	}
}

pub fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ADDBOLT, ghoul2, modelIndex, boneName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_AttachEnt(boltInfo: *mut c_int, ghlInfoTo: *mut c_void, toBoltIndex: c_int, entNum: c_int, toModelNum: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ATTACHENT, boltInfo as *mut c_void, ghlInfoTo, toBoltIndex, entNum, toModelNum)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetBoltInfo(ghoul2: *mut c_void, modelIndex: c_int, boltInfo: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETBOLTON, ghoul2, modelIndex, boltInfo);
		}
	}
}

pub fn trap_G2API_SetRootSurface(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETROOTSURFACE, ghoul2, modelIndex, surfaceName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetSurfaceOnOff(ghoul2: *mut c_void, surfaceName: *const c_char, flags: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETSURFACEONOFF, ghoul2, surfaceName as *mut c_void, flags)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetNewOrigin(ghoul2: *mut c_void, boltIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETNEWORIGIN, ghoul2, boltIndex)
		} else {
			0
		}
	}
}

//check if a bone exists on skeleton without actually adding to the bone list -rww
pub fn trap_G2API_DoesBoneExist(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_DOESBONEEXIST, ghoul2, modelIndex, boneName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetSurfaceRenderStatus(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETSURFACERENDERSTATUS, ghoul2, modelIndex, surfaceName as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetTime() -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETTIME)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetTime(time: c_int, clock: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETTIME, time, clock);
		}
	}
}

//hack for smoothing during ugly situations. forgive me.
pub fn trap_G2API_AbsurdSmoothing(ghoul2: *mut c_void, status: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ABSURDSMOOTHING, ghoul2, status);
		}
	}
}

//rww - RAGDOLL_BEGIN
pub fn trap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETRAGDOLL, ghoul2, params);
		}
	}
}

pub fn trap_G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, params: *mut c_void) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ANIMATEG2MODELS, ghoul2, time, params);
		}
	}
}
//rww - RAGDOLL_END

//additional ragdoll options -rww
pub fn trap_G2API_RagPCJConstraint(ghoul2: *mut c_void, boneName: *const c_char, min: *mut [f32; 3], max: *mut [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_RAGPCJCONSTRAINT, ghoul2, boneName as *mut c_void, min as *mut c_void, max as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_RagPCJGradientSpeed(ghoul2: *mut c_void, boneName: *const c_char, speed: f32) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_RAGPCJGRADIENTSPEED, ghoul2, boneName as *mut c_void, PASSFLOAT(speed))
		} else {
			0
		}
	}
}

pub fn trap_G2API_RagEffectorGoal(ghoul2: *mut c_void, boneName: *const c_char, pos: *mut [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_RAGEFFECTORGOAL, ghoul2, boneName as *mut c_void, pos as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetRagBonePos(ghoul2: *mut c_void, boneName: *const c_char, pos: *mut [f32; 3], entAngles: *mut [f32; 3], entPos: *mut [f32; 3], entScale: *mut [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETRAGBONEPOS, ghoul2, boneName as *mut c_void, pos as *mut c_void, entAngles as *mut c_void, entPos as *mut c_void, entScale as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_RagEffectorKick(ghoul2: *mut c_void, boneName: *const c_char, velocity: *mut [f32; 3]) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_RAGEFFECTORKICK, ghoul2, boneName as *mut c_void, velocity as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_G2API_RagForceSolve(ghoul2: *mut c_void, force: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_RAGFORCESOLVE, ghoul2, force)
		} else {
			0
		}
	}
}

pub fn trap_G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_SETBONEIKSTATE, ghoul2, time, boneName as *mut c_void, ikState, params)
		} else {
			0
		}
	}
}

pub fn trap_G2API_IKMove(ghoul2: *mut c_void, time: c_int, params: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_IKMOVE, ghoul2, time, params)
		} else {
			0
		}
	}
}

pub fn trap_G2API_RemoveBone(ghoul2: *mut c_void, boneName: *const c_char, modelIndex: c_int) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_REMOVEBONE, ghoul2, boneName as *mut c_void, modelIndex)
		} else {
			0
		}
	}
}

//rww - Stuff to allow association of ghoul2 instances to entity numbers.
//This way, on listen servers when both the client and server are doing
//ghoul2 operations, we can copy relevant data off the client instance
//directly onto the server instance and slash the transforms and whatnot
//right in half.
pub fn trap_G2API_AttachInstanceToEntNum(ghoul2: *mut c_void, entityNum: c_int, server: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_ATTACHINSTANCETOENTNUM, ghoul2, entityNum, server);
		}
	}
}

pub fn trap_G2API_ClearAttachedInstance(entityNum: c_int) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_CLEARATTACHEDINSTANCE, entityNum);
		}
	}
}

pub fn trap_G2API_CleanEntAttachments() {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_CLEANENTATTACHMENTS);
		}
	}
}

pub fn trap_G2API_OverrideServer(serverInstance: *mut c_void) -> c_int {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_OVERRIDESERVER, serverInstance)
		} else {
			0
		}
	}
}

pub fn trap_G2API_GetSurfaceName(ghoul2: *mut c_void, surfNumber: c_int, modelIndex: c_int, fillBuf: *mut c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_G2_GETSURFACENAME, ghoul2, surfNumber, modelIndex, fillBuf as *mut c_void);
		}
	}
}

pub fn trap_CG_RegisterSharedMemory(memory: *mut c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_SET_SHARED_BUFFER, memory as *mut c_void);
		}
	}
}

pub fn trap_CM_RegisterTerrain(config: *const c_char) -> c_int {
	//rwwRMG - added [NEWTRAP]
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_CM_REGISTER_TERRAIN, config as *mut c_void)
		} else {
			0
		}
	}
}

pub fn trap_RMG_Init(terrainID: c_int, terrainInfo: *const c_char) {
	//rwwRMG - added [NEWTRAP]
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_RMG_INIT, terrainID, terrainInfo as *mut c_void);
		}
	}
}

pub fn trap_RE_InitRendererTerrain(info: *const c_char) {
	//rwwRMG - added [NEWTRAP]
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_RE_INIT_RENDERER_TERRAIN, info as *mut c_void);
		}
	}
}

pub fn trap_R_WeatherContentsOverride(contents: c_int) {
	//rwwRMG - added [NEWTRAP]
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_WEATHER_CONTENTS_OVERRIDE, contents);
		}
	}
}

pub fn trap_R_WorldEffectCommand(cmd: *const c_char) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_R_WORLDEFFECTCOMMAND, cmd as *mut c_void);
		}
	}
}

pub fn trap_WE_AddWeatherZone(mins: *const [f32; 3], maxs: *const [f32; 3]) {
	unsafe {
		if let Some(syscall_fn) = syscall {
			syscall_fn(CG_WE_ADDWEATHERZONE, mins as *mut c_void, maxs as *mut c_void);
		}
	}
}

/*
Ghoul2 Insert End
*/

// Syscall command IDs (from cg_public.h or similar)
// These are the trap handler enum values
const CG_PRINT: c_int = 0;
const CG_ERROR: c_int = 1;
const CG_MILLISECONDS: c_int = 2;
const CG_PRECISIONTIMER_START: c_int = 3;
const CG_PRECISIONTIMER_END: c_int = 4;
const CG_CVAR_REGISTER: c_int = 5;
const CG_CVAR_UPDATE: c_int = 6;
const CG_CVAR_SET: c_int = 7;
const CG_CVAR_VARIABLESTRINGBUFFER: c_int = 8;
const CG_CVAR_GETHIDDENVALUE: c_int = 9;
const CG_ARGC: c_int = 10;
const CG_ARGV: c_int = 11;
const CG_ARGS: c_int = 12;
const CG_FS_FOPENFILE: c_int = 13;
const CG_FS_READ: c_int = 14;
const CG_FS_WRITE: c_int = 15;
const CG_FS_FCLOSEFILE: c_int = 16;
const CG_FS_GETFILELIST: c_int = 17;
const CG_SENDCONSOLECOMMAND: c_int = 18;
const CG_ADDCOMMAND: c_int = 19;
const CG_REMOVECOMMAND: c_int = 20;
const CG_SENDCLIENTCOMMAND: c_int = 21;
const CG_UPDATESCREEN: c_int = 22;
const CG_CM_LOADMAP: c_int = 23;
const CG_CM_NUMINLINEMODELS: c_int = 24;
const CG_CM_INLINEMODEL: c_int = 25;
const CG_CM_TEMPBOXMODEL: c_int = 26;
const CG_CM_TEMPCAPSULEMODEL: c_int = 27;
const CG_CM_POINTCONTENTS: c_int = 28;
const CG_CM_TRANSFORMEDPOINTCONTENTS: c_int = 29;
const CG_CM_BOXTRACE: c_int = 30;
const CG_CM_CAPSULETRACE: c_int = 31;
const CG_CM_TRANSFORMEDBOXTRACE: c_int = 32;
const CG_CM_TRANSFORMEDCAPSULETRACE: c_int = 33;
const CG_CM_MARKFRAGMENTS: c_int = 34;
const CG_S_GETVOICEVOLUME: c_int = 35;
const CG_S_MUTESOUND: c_int = 36;
const CG_S_STARTSOUND: c_int = 37;
const CG_S_STARTLOCALSOUND: c_int = 38;
const CG_S_CLEARLOOPINGSOUNDS: c_int = 39;
const CG_S_ADDLOOPINGSOUND: c_int = 40;
const CG_S_UPDATEENTITYPOSITION: c_int = 41;
const CG_S_ADDREALLOOPINGSOUND: c_int = 42;
const CG_S_STOPLOOPINGSOUND: c_int = 43;
const CG_S_RESPATIALIZE: c_int = 44;
const CG_S_SHUTUP: c_int = 45;
const CG_S_REGISTERSOUND: c_int = 46;
const CG_S_STARTBACKGROUNDTRACK: c_int = 47;
const CG_S_UPDATEAMBIENTSET: c_int = 48;
const CG_AS_PARSESETS: c_int = 49;
const CG_AS_ADDPRECACHEENTRY: c_int = 50;
const CG_S_ADDLOCALSET: c_int = 51;
const CG_AS_GETBMODELSOUND: c_int = 52;
const CG_R_LOADWORLDMAP: c_int = 53;
const CG_R_REGISTERMODEL: c_int = 54;
const CG_R_REGISTERSKIN: c_int = 55;
const CG_R_REGISTERSHADER: c_int = 56;
const CG_R_REGISTERSHADERNOMIP: c_int = 57;
const CG_R_REGISTERFONT: c_int = 58;
const CG_R_FONT_STRLENPIXELS: c_int = 59;
const CG_R_FONT_STRLENCHARS: c_int = 60;
const CG_R_FONT_STRHEIGHTPIXELS: c_int = 61;
const CG_R_FONT_DRAWSTRING: c_int = 62;
const CG_LANGUAGE_ISASIAN: c_int = 63;
const CG_LANGUAGE_USESSPACES: c_int = 64;
const CG_ANYLANGUAGE_READCHARFROMSTRING: c_int = 65;
const CG_R_CLEARSCENE: c_int = 66;
const CG_R_CLEARDECALS: c_int = 67;
const CG_R_ADDREFENTITYTOSCENE: c_int = 68;
const CG_R_ADDPOLYTOSCENE: c_int = 69;
const CG_R_ADDPOLYSTOSCENE: c_int = 70;
const CG_R_ADDDECALTOSCENE: c_int = 71;
const CG_R_LIGHTFORPOINT: c_int = 72;
const CG_R_ADDLIGHTTOSCENE: c_int = 73;
const CG_R_ADDADDITIVELIGHTTOSCENE: c_int = 74;
const CG_R_RENDERSCENE: c_int = 75;
const CG_R_SETCOLOR: c_int = 76;
const CG_R_DRAWSTRETCHPIC: c_int = 77;
const CG_R_MODELBOUNDS: c_int = 78;
const CG_R_LERPTAG: c_int = 79;
const CG_R_DRAWROTATEPIC: c_int = 80;
const CG_R_DRAWROTATEPIC2: c_int = 81;
const CG_R_SETRANGEFOG: c_int = 82;
const CG_R_SETREFRACTIONPROP: c_int = 83;
const CG_R_REMAP_SHADER: c_int = 84;
const CG_R_GET_LIGHT_STYLE: c_int = 85;
const CG_R_SET_LIGHT_STYLE: c_int = 86;
const CG_R_GET_BMODEL_VERTS: c_int = 87;
const CG_R_GETDISTANCECULL: c_int = 88;
const CG_R_GETREALRES: c_int = 89;
const CG_R_AUTOMAPELEVADJ: c_int = 90;
const CG_R_INITWIREFRAMEAUTO: c_int = 91;
const CG_FX_ADDLINE: c_int = 92;
const CG_GETGLCONFIG: c_int = 93;
const CG_GETGAMESTATE: c_int = 94;
const CG_GETCURRENTSNAPSHOTNUMBER: c_int = 95;
const CG_GETSNAPSHOT: c_int = 96;
const CG_GETDEFAULTSTATE: c_int = 97;
const CG_GETSERVERCOMMAND: c_int = 98;
const CG_GETCURRENTCMDNUMBER: c_int = 99;
const CG_GETUSERCMD: c_int = 100;
const CG_SETUSERCMDVALUE: c_int = 101;
const CG_SETCLIENTFORCEANGLE: c_int = 102;
const CG_SETCLIENTTURNEXTENT: c_int = 103;
const CG_OPENUIMENU: c_int = 104;
const CG_TESTPRINTINT: c_int = 105;
const CG_TESTPRINTFLOAT: c_int = 106;
const CG_MEMORY_REMAINING: c_int = 107;
const CG_KEY_ISDOWN: c_int = 108;
const CG_KEY_GETCATCHER: c_int = 109;
const CG_KEY_SETCATCHER: c_int = 110;
const CG_KEY_GETKEY: c_int = 111;
const CG_PC_ADD_GLOBAL_DEFINE: c_int = 112;
const CG_PC_LOAD_SOURCE: c_int = 113;
const CG_PC_FREE_SOURCE: c_int = 114;
const CG_PC_READ_TOKEN: c_int = 115;
const CG_PC_SOURCE_FILE_AND_LINE: c_int = 116;
const CG_PC_LOAD_GLOBAL_DEFINES: c_int = 117;
const CG_PC_REMOVE_ALL_GLOBAL_DEFINES: c_int = 118;
const CG_S_STOPBACKGROUNDTRACK: c_int = 119;
const CG_REAL_TIME: c_int = 120;
const CG_SNAPVECTOR: c_int = 121;
const CG_CIN_PLAYCINEMATIC: c_int = 122;
const CG_CIN_STOPCINEMATIC: c_int = 123;
const CG_CIN_RUNCINEMATIC: c_int = 124;
const CG_CIN_DRAWCINEMATIC: c_int = 125;
const CG_CIN_SETEXTENTS: c_int = 126;
const CG_GET_ENTITY_TOKEN: c_int = 127;
const CG_R_INPVS: c_int = 128;
const CG_FX_REGISTER_EFFECT: c_int = 129;
const CG_FX_PLAY_EFFECT: c_int = 130;
const CG_FX_PLAY_ENTITY_EFFECT: c_int = 131;
const CG_FX_PLAY_EFFECT_ID: c_int = 132;
const CG_FX_PLAY_PORTAL_EFFECT_ID: c_int = 133;
const CG_FX_PLAY_ENTITY_EFFECT_ID: c_int = 134;
const CG_FX_PLAY_BOLTED_EFFECT_ID: c_int = 135;
const CG_FX_ADD_SCHEDULED_EFFECTS: c_int = 136;
const CG_FX_DRAW_2D_EFFECTS: c_int = 137;
const CG_FX_INIT_SYSTEM: c_int = 138;
const CG_FX_SET_REFDEF: c_int = 139;
const CG_FX_FREE_SYSTEM: c_int = 140;
const CG_FX_RESET: c_int = 141;
const CG_FX_ADJUST_TIME: c_int = 142;
const CG_FX_ADDPOLY: c_int = 143;
const CG_FX_ADDBEZIER: c_int = 144;
const CG_FX_ADDPRIMITIVE: c_int = 145;
const CG_FX_ADDSPRITE: c_int = 146;
const CG_FX_ADDELECTRICITY: c_int = 147;
const CG_SP_GETSTRINGTEXTSTRING: c_int = 148;
const CG_ROFF_CLEAN: c_int = 149;
const CG_ROFF_UPDATE_ENTITIES: c_int = 150;
const CG_ROFF_CACHE: c_int = 151;
const CG_ROFF_PLAY: c_int = 152;
const CG_ROFF_PURGE_ENT: c_int = 153;
const CG_TRUEMALLOC: c_int = 154;
const CG_TRUEFREE: c_int = 155;
const CG_G2_LISTSURFACES: c_int = 156;
const CG_G2_LISTBONES: c_int = 157;
const CG_G2_SETMODELS: c_int = 158;
const CG_G2_HAVEWEGHOULMODELS: c_int = 159;
const CG_G2_GETBOLT: c_int = 160;
const CG_G2_GETBOLT_NOREC: c_int = 161;
const CG_G2_GETBOLT_NOREC_NOROT: c_int = 162;
const CG_G2_INITGHOUL2MODEL: c_int = 163;
const CG_G2_SETSKIN: c_int = 164;
const CG_G2_COLLISIONDETECT: c_int = 165;
const CG_G2_COLLISIONDETECTCACHE: c_int = 166;
const CG_G2_CLEANMODELS: c_int = 167;
const CG_G2_ANGLEOVERRIDE: c_int = 168;
const CG_G2_PLAYANIM: c_int = 169;
const CG_G2_GETBONEANIM: c_int = 170;
const CG_G2_GETBONEFRAME: c_int = 171;
const CG_G2_GETGLANAME: c_int = 172;
const CG_G2_COPYGHOUL2INSTANCE: c_int = 173;
const CG_G2_COPYSPECIFICGHOUL2MODEL: c_int = 174;
const CG_G2_DUPLICATEGHOUL2INSTANCE: c_int = 175;
const CG_G2_HASGHOUL2MODELONINDEX: c_int = 176;
const CG_G2_REMOVEGHOUL2MODEL: c_int = 177;
const CG_G2_SKINLESSMODEL: c_int = 178;
const CG_G2_GETNUMGOREMARKS: c_int = 179;
const CG_G2_ADDSKINGORE: c_int = 180;
const CG_G2_CLEARSKINGORE: c_int = 181;
const CG_G2_SIZE: c_int = 182;
const CG_G2_ADDBOLT: c_int = 183;
const CG_G2_ATTACHENT: c_int = 184;
const CG_G2_SETBOLTON: c_int = 185;
const CG_G2_SETROOTSURFACE: c_int = 186;
const CG_G2_SETSURFACEONOFF: c_int = 187;
const CG_G2_SETNEWORIGIN: c_int = 188;
const CG_G2_DOESBONEEXIST: c_int = 189;
const CG_G2_GETSURFACERENDERSTATUS: c_int = 190;
const CG_G2_GETTIME: c_int = 191;
const CG_G2_SETTIME: c_int = 192;
const CG_G2_ABSURDSMOOTHING: c_int = 193;
const CG_G2_SETRAGDOLL: c_int = 194;
const CG_G2_ANIMATEG2MODELS: c_int = 195;
const CG_G2_RAGPCJCONSTRAINT: c_int = 196;
const CG_G2_RAGPCJGRADIENTSPEED: c_int = 197;
const CG_G2_RAGEFFECTORGOAL: c_int = 198;
const CG_G2_GETRAGBONEPOS: c_int = 199;
const CG_G2_RAGEFFECTORKICK: c_int = 200;
const CG_G2_RAGFORCESOLVE: c_int = 201;
const CG_G2_SETBONEIKSTATE: c_int = 202;
const CG_G2_IKMOVE: c_int = 203;
const CG_G2_REMOVEBONE: c_int = 204;
const CG_G2_ATTACHINSTANCETOENTNUM: c_int = 205;
const CG_G2_CLEARATTACHEDINSTANCE: c_int = 206;
const CG_G2_CLEANENTATTACHMENTS: c_int = 207;
const CG_G2_OVERRIDESERVER: c_int = 208;
const CG_G2_GETSURFACENAME: c_int = 209;
const CG_SET_SHARED_BUFFER: c_int = 210;
const CG_CM_REGISTER_TERRAIN: c_int = 211;
const CG_RMG_INIT: c_int = 212;
const CG_RE_INIT_RENDERER_TERRAIN: c_int = 213;
const CG_R_WEATHER_CONTENTS_OVERRIDE: c_int = 214;
const CG_R_WORLDEFFECTCOMMAND: c_int = 215;
const CG_WE_ADDWEATHERZONE: c_int = 216;
