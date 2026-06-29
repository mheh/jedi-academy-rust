// Copyright (C) 1999-2000 Id Software, Inc.
//

use core::ffi::{c_int, c_char, c_void};
use core::mem::transmute;

#[allow(non_snake_case)]

// this file is only included when building a dll
// syscalls.asm is included instead when building a qvm

type SyscallFn = unsafe extern "C" fn(c_int, ...) -> c_int;

// Faithfully initialized to -1; actual function set by dllEntry before use
// SAFETY: This represents an invalid function pointer; calling before dllEntry is UB
static mut syscall: SyscallFn = unsafe {
    transmute::<isize, SyscallFn>(-1)
};

pub unsafe fn dllEntry(syscallptr: SyscallFn) {
    syscall = syscallptr;
}

fn PASSFLOAT(x: f32) -> c_int {
    let floatTemp: f32 = x;
    unsafe { *(addr_of!(floatTemp) as *const c_int) }
}

use core::ptr::addr_of;

pub fn trap_Print(string: *const c_char) {
    unsafe {
        syscall(UI_PRINT as c_int, string);
    }
}

pub fn trap_Error(string: *const c_char) {
    unsafe {
        syscall(UI_ERROR as c_int, string);
    }
}

pub fn trap_Milliseconds() -> c_int {
    unsafe {
        syscall(UI_MILLISECONDS as c_int)
    }
}

pub fn trap_Cvar_Register(cvar: *mut c_void, var_name: *const c_char, value: *const c_char, flags: c_int) {
    unsafe {
        syscall(UI_CVAR_REGISTER as c_int, cvar, var_name, value, flags);
    }
}

pub fn trap_Cvar_Update(cvar: *mut c_void) {
    unsafe {
        syscall(UI_CVAR_UPDATE as c_int, cvar);
    }
}

pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char) {
    unsafe {
        syscall(UI_CVAR_SET as c_int, var_name, value);
    }
}

pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32 {
    unsafe {
        let temp: c_int = syscall(UI_CVAR_VARIABLEVALUE as c_int, var_name);
        *(addr_of!(temp) as *const f32)
    }
}

pub fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) {
    unsafe {
        syscall(UI_CVAR_VARIABLESTRINGBUFFER as c_int, var_name, buffer, bufsize);
    }
}

pub fn trap_Cvar_SetValue(var_name: *const c_char, value: f32) {
    unsafe {
        syscall(UI_CVAR_SETVALUE as c_int, var_name, PASSFLOAT(value));
    }
}

pub fn trap_Cvar_Reset(name: *const c_char) {
    unsafe {
        syscall(UI_CVAR_RESET as c_int, name);
    }
}

pub fn trap_Cvar_Create(var_name: *const c_char, var_value: *const c_char, flags: c_int) {
    unsafe {
        syscall(UI_CVAR_CREATE as c_int, var_name, var_value, flags);
    }
}

pub fn trap_Cvar_InfoStringBuffer(bit: c_int, buffer: *mut c_char, bufsize: c_int) {
    unsafe {
        syscall(UI_CVAR_INFOSTRINGBUFFER as c_int, bit, buffer, bufsize);
    }
}

pub fn trap_Argc() -> c_int {
    unsafe {
        syscall(UI_ARGC as c_int)
    }
}

pub fn trap_Argv(n: c_int, buffer: *mut c_char, bufferLength: c_int) {
    unsafe {
        syscall(UI_ARGV as c_int, n, buffer, bufferLength);
    }
}

pub fn trap_Cmd_ExecuteText(exec_when: c_int, text: *const c_char) {
    unsafe {
        syscall(UI_CMD_EXECUTETEXT as c_int, exec_when, text);
    }
}

pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut c_void, mode: c_int) -> c_int {
    unsafe {
        syscall(UI_FS_FOPENFILE as c_int, qpath, f, mode)
    }
}

pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: c_int) {
    unsafe {
        syscall(UI_FS_READ as c_int, buffer, len, f);
    }
}

pub fn trap_FS_Write(buffer: *const c_void, len: c_int, f: c_int) {
    unsafe {
        syscall(UI_FS_WRITE as c_int, buffer, len, f);
    }
}

pub fn trap_FS_FCloseFile(f: c_int) {
    unsafe {
        syscall(UI_FS_FCLOSEFILE as c_int, f);
    }
}

pub fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int {
    unsafe {
        syscall(UI_FS_GETFILELIST as c_int, path, extension, listbuf, bufsize)
    }
}

pub fn trap_R_RegisterModel(name: *const c_char) -> c_int {
    unsafe {
        syscall(UI_R_REGISTERMODEL as c_int, name)
    }
}

pub fn trap_R_RegisterSkin(name: *const c_char) -> c_int {
    unsafe {
        syscall(UI_R_REGISTERSKIN as c_int, name)
    }
}

pub fn trap_R_RegisterFont(fontName: *const c_char) -> c_int {
    unsafe {
        syscall(UI_R_REGISTERFONT as c_int, fontName)
    }
}

pub fn trap_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int {
    unsafe {
        syscall(UI_R_FONT_STRLENPIXELS as c_int, text, iFontIndex, PASSFLOAT(scale))
    }
}

pub fn trap_R_Font_StrLenChars(text: *const c_char) -> c_int {
    unsafe {
        syscall(UI_R_FONT_STRLENCHARS as c_int, text)
    }
}

pub fn trap_R_Font_HeightPixels(iFontIndex: c_int, scale: f32) -> c_int {
    unsafe {
        syscall(UI_R_FONT_STRHEIGHTPIXELS as c_int, iFontIndex, PASSFLOAT(scale))
    }
}

pub fn trap_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const f32, setIndex: c_int, iCharLimit: c_int, scale: f32) {
    unsafe {
        syscall(UI_R_FONT_DRAWSTRING as c_int, ox, oy, text, rgba, setIndex, iCharLimit, PASSFLOAT(scale));
    }
}

pub fn trap_Language_IsAsian() -> c_int {
    unsafe {
        syscall(UI_LANGUAGE_ISASIAN as c_int)
    }
}

pub fn trap_Language_UsesSpaces() -> c_int {
    unsafe {
        syscall(UI_LANGUAGE_USESSPACES as c_int)
    }
}

pub fn trap_AnyLanguage_ReadCharFromString(psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut c_int) -> c_int {
    unsafe {
        syscall(UI_ANYLANGUAGE_READCHARFROMSTRING as c_int, psText, piAdvanceCount, pbIsTrailingPunctuation)
    }
}

pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int {
    let mut buf: [c_char; 1024] = [0; 1024];

    unsafe {
        if *name == b'*' as c_char {
            trap_Cvar_VariableStringBuffer(name.offset(1), buf.as_mut_ptr(), 1024);
            if buf[0] != 0 {
                return syscall(UI_R_REGISTERSHADERNOMIP as c_int, buf.as_ptr());
            }
        }
        syscall(UI_R_REGISTERSHADERNOMIP as c_int, name)
    }
}

// added so I don't have to store a string containing the path of
// the shader icon for a class -rww
pub fn trap_R_ShaderNameFromIndex(name: *mut c_char, index: c_int) {
    unsafe {
        syscall(UI_R_SHADERNAMEFROMINDEX as c_int, name, index);
    }
}

pub fn trap_R_ClearScene() {
    unsafe {
        syscall(UI_R_CLEARSCENE as c_int);
    }
}

pub fn trap_R_AddRefEntityToScene(re: *const c_void) {
    unsafe {
        syscall(UI_R_ADDREFENTITYTOSCENE as c_int, re);
    }
}

pub fn trap_R_AddPolyToScene(hShader: c_int, numVerts: c_int, verts: *const c_void) {
    unsafe {
        syscall(UI_R_ADDPOLYTOSCENE as c_int, hShader, numVerts, verts);
    }
}

pub fn trap_R_AddLightToScene(org: *const c_void, intensity: f32, r: f32, g: f32, b: f32) {
    unsafe {
        syscall(UI_R_ADDLIGHTTOSCENE as c_int, org, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b));
    }
}

pub fn trap_R_RenderScene(fd: *const c_void) {
    unsafe {
        syscall(UI_R_RENDERSCENE as c_int, fd);
    }
}

pub fn trap_R_SetColor(rgba: *const f32) {
    unsafe {
        syscall(UI_R_SETCOLOR as c_int, rgba);
    }
}

pub fn trap_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: c_int) {
    unsafe {
        syscall(UI_R_DRAWSTRETCHPIC as c_int, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), hShader);
    }
}

pub fn trap_R_ModelBounds(model: c_int, mins: *mut c_void, maxs: *mut c_void) {
    unsafe {
        syscall(UI_R_MODELBOUNDS as c_int, model, mins, maxs);
    }
}

pub fn trap_UpdateScreen() {
    unsafe {
        syscall(UI_UPDATESCREEN as c_int);
    }
}

pub fn trap_CM_LerpTag(tag: *mut c_void, mod_: c_int, startFrame: c_int, endFrame: c_int, frac: f32, tagName: *const c_char) -> c_int {
    unsafe {
        syscall(UI_CM_LERPTAG as c_int, tag, mod_, startFrame, endFrame, PASSFLOAT(frac), tagName)
    }
}

pub fn trap_S_StartLocalSound(sfx: c_int, channelNum: c_int) {
    unsafe {
        syscall(UI_S_STARTLOCALSOUND as c_int, sfx, channelNum);
    }
}

pub fn trap_S_RegisterSound(sample: *const c_char) -> c_int {
    unsafe {
        syscall(UI_S_REGISTERSOUND as c_int, sample)
    }
}

pub fn trap_Key_KeynumToStringBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_KEY_KEYNUMTOSTRINGBUF as c_int, keynum, buf, buflen);
    }
}

pub fn trap_Key_GetBindingBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_KEY_GETBINDINGBUF as c_int, keynum, buf, buflen);
    }
}

pub fn trap_Key_SetBinding(keynum: c_int, binding: *const c_char) {
    unsafe {
        syscall(UI_KEY_SETBINDING as c_int, keynum, binding);
    }
}

pub fn trap_Key_IsDown(keynum: c_int) -> c_int {
    unsafe {
        syscall(UI_KEY_ISDOWN as c_int, keynum)
    }
}

pub fn trap_Key_GetOverstrikeMode() -> c_int {
    unsafe {
        syscall(UI_KEY_GETOVERSTRIKEMODE as c_int)
    }
}

pub fn trap_Key_SetOverstrikeMode(state: c_int) {
    unsafe {
        syscall(UI_KEY_SETOVERSTRIKEMODE as c_int, state);
    }
}

pub fn trap_Key_ClearStates() {
    unsafe {
        syscall(UI_KEY_CLEARSTATES as c_int);
    }
}

pub fn trap_Key_GetCatcher() -> c_int {
    unsafe {
        syscall(UI_KEY_GETCATCHER as c_int)
    }
}

pub fn trap_Key_SetCatcher(catcher: c_int) {
    unsafe {
        syscall(UI_KEY_SETCATCHER as c_int, catcher);
    }
}

pub fn trap_GetClipboardData(buf: *mut c_char, bufsize: c_int) {
    unsafe {
        syscall(UI_GETCLIPBOARDDATA as c_int, buf, bufsize);
    }
}

pub fn trap_GetClientState(state: *mut c_void) {
    unsafe {
        syscall(UI_GETCLIENTSTATE as c_int, state);
    }
}

pub fn trap_GetGlconfig(glconfig: *mut c_void) {
    unsafe {
        syscall(UI_GETGLCONFIG as c_int, glconfig);
    }
}

pub fn trap_GetConfigString(index: c_int, buff: *mut c_char, buffsize: c_int) -> c_int {
    unsafe {
        syscall(UI_GETCONFIGSTRING as c_int, index, buff, buffsize)
    }
}

pub fn trap_LAN_GetServerCount(source: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_GETSERVERCOUNT as c_int, source)
    }
}

pub fn trap_LAN_GetServerAddressString(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_LAN_GETSERVERADDRESSSTRING as c_int, source, n, buf, buflen);
    }
}

pub fn trap_LAN_GetServerInfo(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_LAN_GETSERVERINFO as c_int, source, n, buf, buflen);
    }
}

pub fn trap_LAN_GetServerPing(source: c_int, n: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_GETSERVERPING as c_int, source, n)
    }
}

pub fn trap_LAN_GetPingQueueCount() -> c_int {
    unsafe {
        syscall(UI_LAN_GETPINGQUEUECOUNT as c_int)
    }
}

pub fn trap_LAN_ServerStatus(serverAddress: *const c_char, serverStatus: *mut c_char, maxLen: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_SERVERSTATUS as c_int, serverAddress, serverStatus, maxLen)
    }
}

pub fn trap_LAN_SaveCachedServers() {
    unsafe {
        syscall(UI_LAN_SAVECACHEDSERVERS as c_int);
    }
}

pub fn trap_LAN_LoadCachedServers() {
    unsafe {
        syscall(UI_LAN_LOADCACHEDSERVERS as c_int);
    }
}

pub fn trap_LAN_ResetPings(n: c_int) {
    unsafe {
        syscall(UI_LAN_RESETPINGS as c_int, n);
    }
}

pub fn trap_LAN_ClearPing(n: c_int) {
    unsafe {
        syscall(UI_LAN_CLEARPING as c_int, n);
    }
}

pub fn trap_LAN_GetPing(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int) {
    unsafe {
        syscall(UI_LAN_GETPING as c_int, n, buf, buflen, pingtime);
    }
}

pub fn trap_LAN_GetPingInfo(n: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_LAN_GETPINGINFO as c_int, n, buf, buflen);
    }
}

pub fn trap_LAN_MarkServerVisible(source: c_int, n: c_int, visible: c_int) {
    unsafe {
        syscall(UI_LAN_MARKSERVERVISIBLE as c_int, source, n, visible);
    }
}

pub fn trap_LAN_ServerIsVisible(source: c_int, n: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_SERVERISVISIBLE as c_int, source, n)
    }
}

pub fn trap_LAN_UpdateVisiblePings(source: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_UPDATEVISIBLEPINGS as c_int, source)
    }
}

pub fn trap_LAN_AddServer(source: c_int, name: *const c_char, addr: *const c_char) -> c_int {
    unsafe {
        syscall(UI_LAN_ADDSERVER as c_int, source, name, addr)
    }
}

pub fn trap_LAN_RemoveServer(source: c_int, addr: *const c_char) {
    unsafe {
        syscall(UI_LAN_REMOVESERVER as c_int, source, addr);
    }
}

pub fn trap_LAN_CompareServers(source: c_int, sortKey: c_int, sortDir: c_int, s1: c_int, s2: c_int) -> c_int {
    unsafe {
        syscall(UI_LAN_COMPARESERVERS as c_int, source, sortKey, sortDir, s1, s2)
    }
}

pub fn trap_MemoryRemaining() -> c_int {
    unsafe {
        syscall(UI_MEMORY_REMAINING as c_int)
    }
}

#[cfg(feature = "USE_CD_KEY")]
pub fn trap_GetCDKey(buf: *mut c_char, buflen: c_int) {
    unsafe {
        syscall(UI_GET_CDKEY as c_int, buf, buflen);
    }
}

#[cfg(feature = "USE_CD_KEY")]
pub fn trap_SetCDKey(buf: *mut c_char) {
    unsafe {
        syscall(UI_SET_CDKEY as c_int, buf);
    }
}

#[cfg(feature = "USE_CD_KEY")]
pub fn trap_VerifyCDKey(key: *const c_char, chksum: *const c_char) -> c_int {
    unsafe {
        syscall(UI_VERIFY_CDKEY as c_int, key, chksum)
    }
}

pub fn trap_PC_AddGlobalDefine(define: *mut c_char) -> c_int {
    unsafe {
        syscall(UI_PC_ADD_GLOBAL_DEFINE as c_int, define)
    }
}

pub fn trap_PC_LoadSource(filename: *const c_char) -> c_int {
    unsafe {
        syscall(UI_PC_LOAD_SOURCE as c_int, filename)
    }
}

pub fn trap_PC_FreeSource(handle: c_int) -> c_int {
    unsafe {
        syscall(UI_PC_FREE_SOURCE as c_int, handle)
    }
}

pub fn trap_PC_ReadToken(handle: c_int, pc_token: *mut c_void) -> c_int {
    unsafe {
        syscall(UI_PC_READ_TOKEN as c_int, handle, pc_token)
    }
}

pub fn trap_PC_SourceFileAndLine(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int {
    unsafe {
        syscall(UI_PC_SOURCE_FILE_AND_LINE as c_int, handle, filename, line)
    }
}

pub fn trap_PC_LoadGlobalDefines(filename: *const c_char) -> c_int {
    unsafe {
        syscall(UI_PC_LOAD_GLOBAL_DEFINES as c_int, filename)
    }
}

pub fn trap_PC_RemoveAllGlobalDefines() {
    unsafe {
        syscall(UI_PC_REMOVE_ALL_GLOBAL_DEFINES as c_int);
    }
}

pub fn trap_S_StopBackgroundTrack() {
    unsafe {
        syscall(UI_S_STOPBACKGROUNDTRACK as c_int);
    }
}

pub fn trap_S_StartBackgroundTrack(intro: *const c_char, loop_: *const c_char, bReturnWithoutStarting: c_int) {
    unsafe {
        syscall(UI_S_STARTBACKGROUNDTRACK as c_int, intro, loop_, bReturnWithoutStarting);
    }
}

pub fn trap_RealTime(qtime: *mut c_void) -> c_int {
    unsafe {
        syscall(UI_REAL_TIME as c_int, qtime)
    }
}

// this returns a handle.  arg0 is the name in the format "idlogo.roq", set arg1 to NULL, alteredstates to qfalse (do not alter gamestate)
pub fn trap_CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int) -> c_int {
    unsafe {
        syscall(UI_CIN_PLAYCINEMATIC as c_int, arg0, xpos, ypos, width, height, bits)
    }
}

// stops playing the cinematic and ends it.  should always return FMV_EOF
// cinematics must be stopped in reverse order of when they are started
pub fn trap_CIN_StopCinematic(handle: c_int) -> c_int {
    unsafe {
        syscall(UI_CIN_STOPCINEMATIC as c_int, handle)
    }
}


// will run a frame of the cinematic but will not draw it.  Will return FMV_EOF if the end of the cinematic has been reached.
pub fn trap_CIN_RunCinematic(handle: c_int) -> c_int {
    unsafe {
        syscall(UI_CIN_RUNCINEMATIC as c_int, handle)
    }
}


// draws the current frame
pub fn trap_CIN_DrawCinematic(handle: c_int) {
    unsafe {
        syscall(UI_CIN_DRAWCINEMATIC as c_int, handle);
    }
}


// allows you to resize the animation dynamically
pub fn trap_CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    unsafe {
        syscall(UI_CIN_SETEXTENTS as c_int, handle, x, y, w, h);
    }
}


pub fn trap_R_RemapShader(oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char) {
    unsafe {
        syscall(UI_R_REMAP_SHADER as c_int, oldShader, newShader, timeOffset);
    }
}

pub fn trap_SP_GetNumLanguages() -> c_int {
    unsafe {
        syscall(UI_SP_GETNUMLANGUAGES as c_int)
    }
}

pub fn trap_GetLanguageName(languageIndex: c_int, buffer: *mut c_char) {
    unsafe {
        syscall(UI_SP_GETLANGUAGENAME as c_int, languageIndex, buffer);
    }
}

pub fn trap_SP_GetStringTextString(text: *const c_char, buffer: *mut c_char, bufferLength: c_int) -> c_int {
    unsafe {
        syscall(UI_SP_GETSTRINGTEXTSTRING as c_int, text, buffer, bufferLength)
    }
}

// Ghoul2 Insert Start

pub fn trap_G2_ListModelSurfaces(ghlInfo: *mut c_void) {
    unsafe {
        syscall(UI_G2_LISTSURFACES as c_int, ghlInfo);
    }
}

pub fn trap_G2_ListModelBones(ghlInfo: *mut c_void, frame: c_int) {
    unsafe {
        syscall(UI_G2_LISTBONES as c_int, ghlInfo, frame);
    }
}

pub fn trap_G2_SetGhoul2ModelIndexes(ghoul2: *mut c_void, modelList: *mut c_int, skinList: *mut c_int) {
    unsafe {
        syscall(UI_G2_SETMODELS as c_int, ghoul2, modelList, skinList);
    }
}

pub fn trap_G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> c_int {
    unsafe {
        syscall(UI_G2_HAVEWEGHOULMODELS as c_int, ghoul2) as c_int
    }
}

pub fn trap_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                angles: *const c_void, position: *const c_void, frameNum: c_int, modelList: *mut c_int, scale: *const c_void) -> c_int {
    unsafe {
        syscall(UI_G2_GETBOLT as c_int, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale) as c_int
    }
}

pub fn trap_G2API_GetBoltMatrix_NoReconstruct(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                angles: *const c_void, position: *const c_void, frameNum: c_int, modelList: *mut c_int, scale: *const c_void) -> c_int {
    // Same as above but force it to not reconstruct the skeleton before getting the bolt position
    unsafe {
        syscall(UI_G2_GETBOLT_NOREC as c_int, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale) as c_int
    }
}

pub fn trap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                angles: *const c_void, position: *const c_void, frameNum: c_int, modelList: *mut c_int, scale: *const c_void) -> c_int {
    // Same as above but force it to not reconstruct the skeleton before getting the bolt position
    unsafe {
        syscall(UI_G2_GETBOLT_NOREC_NOROT as c_int, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale) as c_int
    }
}

pub fn trap_G2API_InitGhoul2Model(ghoul2Ptr: *mut *mut c_void, fileName: *const c_char, modelIndex: c_int, customSkin: c_int,
                          customShader: c_int, modelFlags: c_int, lodBias: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_INITGHOUL2MODEL as c_int, ghoul2Ptr, fileName, modelIndex, customSkin, customShader, modelFlags, lodBias)
    }
}

pub fn trap_G2API_SetSkin(ghoul2: *mut c_void, modelIndex: c_int, customSkin: c_int, renderSkin: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_SETSKIN as c_int, ghoul2, modelIndex, customSkin, renderSkin)
    }
}

pub fn trap_G2API_CollisionDetect(
    collRecMap: *mut c_void,
    ghoul2: *mut c_void,
    angles: *const c_void,
    position: *const c_void,
    frameNumber: c_int,
    entNum: c_int,
    rayStart: *const c_void,
    rayEnd: *const c_void,
    scale: *const c_void,
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32
) {
    unsafe {
        syscall(UI_G2_COLLISIONDETECT as c_int, collRecMap, ghoul2, angles, position, frameNumber, entNum, rayStart, rayEnd, scale, traceFlags, useLod, PASSFLOAT(fRadius));
    }
}

pub fn trap_G2API_CollisionDetectCache(
    collRecMap: *mut c_void,
    ghoul2: *mut c_void,
    angles: *const c_void,
    position: *const c_void,
    frameNumber: c_int,
    entNum: c_int,
    rayStart: *const c_void,
    rayEnd: *const c_void,
    scale: *const c_void,
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32
) {
    unsafe {
        syscall(UI_G2_COLLISIONDETECTCACHE as c_int, collRecMap, ghoul2, angles, position, frameNumber, entNum, rayStart, rayEnd, scale, traceFlags, useLod, PASSFLOAT(fRadius));
    }
}

pub fn trap_G2API_CleanGhoul2Models(ghoul2Ptr: *mut *mut c_void) {
    unsafe {
        syscall(UI_G2_CLEANMODELS as c_int, ghoul2Ptr);
    }
}

pub fn trap_G2API_SetBoneAngles(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, angles: *const c_void, flags: c_int,
                                up: c_int, right: c_int, forward: c_int, modelList: *mut c_int,
                                blendTime: c_int, currentTime: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_ANGLEOVERRIDE as c_int, ghoul2, modelIndex, boneName, angles, flags, up, right, forward, modelList, blendTime, currentTime) as c_int
    }
}

pub fn trap_G2API_SetBoneAnim(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int,
                              flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_PLAYANIM as c_int, ghoul2, modelIndex, boneName, startFrame, endFrame, flags, PASSFLOAT(animSpeed), currentTime, PASSFLOAT(setFrame), blendTime) as c_int
    }
}

pub fn trap_G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32,
                           startFrame: *mut c_int, endFrame: *mut c_int, flags: *mut c_int, animSpeed: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_GETBONEANIM as c_int, ghoul2, boneName, currentTime, currentFrame, startFrame, endFrame, flags, animSpeed, modelList, modelIndex) as c_int
    }
}

pub fn trap_G2API_GetBoneFrame(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_GETBONEFRAME as c_int, ghoul2, boneName, currentTime, currentFrame, modelList, modelIndex) as c_int
    }
}

pub fn trap_G2API_GetGLAName(ghoul2: *mut c_void, modelIndex: c_int, fillBuf: *mut c_char) {
    unsafe {
        syscall(UI_G2_GETGLANAME as c_int, ghoul2, modelIndex, fillBuf);
    }
}

pub fn trap_G2API_CopyGhoul2Instance(g2From: *mut c_void, g2To: *mut c_void, modelIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_COPYGHOUL2INSTANCE as c_int, g2From, g2To, modelIndex)
    }
}

pub fn trap_G2API_CopySpecificGhoul2Model(g2From: *mut c_void, modelFrom: c_int, g2To: *mut c_void, modelTo: c_int) {
    unsafe {
        syscall(UI_G2_COPYSPECIFICGHOUL2MODEL as c_int, g2From, modelFrom, g2To, modelTo);
    }
}

pub fn trap_G2API_DuplicateGhoul2Instance(g2From: *mut c_void, g2To: *mut *mut c_void) {
    unsafe {
        syscall(UI_G2_DUPLICATEGHOUL2INSTANCE as c_int, g2From, g2To);
    }
}

pub fn trap_G2API_HasGhoul2ModelOnIndex(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_HASGHOUL2MODELONINDEX as c_int, ghlInfo, modelIndex)
    }
}

pub fn trap_G2API_RemoveGhoul2Model(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_REMOVEGHOUL2MODEL as c_int, ghlInfo, modelIndex)
    }
}

pub fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
    unsafe {
        syscall(UI_G2_ADDBOLT as c_int, ghoul2, modelIndex, boneName)
    }
}

pub fn trap_G2API_SetBoltInfo(ghoul2: *mut c_void, modelIndex: c_int, boltInfo: c_int) {
    unsafe {
        syscall(UI_G2_SETBOLTON as c_int, ghoul2, modelIndex, boltInfo);
    }
}

pub fn trap_G2API_SetRootSurface(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> c_int {
    unsafe {
        syscall(UI_G2_SETROOTSURFACE as c_int, ghoul2, modelIndex, surfaceName)
    }
}

pub fn trap_G2API_SetSurfaceOnOff(ghoul2: *mut c_void, surfaceName: *const c_char, flags: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_SETSURFACEONOFF as c_int, ghoul2, surfaceName, flags)
    }
}

pub fn trap_G2API_SetNewOrigin(ghoul2: *mut c_void, boltIndex: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_SETNEWORIGIN as c_int, ghoul2, boltIndex)
    }
}

pub fn trap_G2API_GetTime() -> c_int {
    unsafe {
        syscall(UI_G2_GETTIME as c_int)
    }
}

pub fn trap_G2API_SetTime(time: c_int, clock: c_int) {
    unsafe {
        syscall(UI_G2_SETTIME as c_int, time, clock);
    }
}

// rww - RAGDOLL_BEGIN
pub fn trap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut c_void) {
    unsafe {
        syscall(UI_G2_SETRAGDOLL as c_int, ghoul2, params);
    }
}

pub fn trap_G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, params: *mut c_void) {
    unsafe {
        syscall(UI_G2_ANIMATEG2MODELS as c_int, ghoul2, time, params);
    }
}
// rww - RAGDOLL_END

pub fn trap_G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *mut c_void) -> c_int {
    unsafe {
        syscall(UI_G2_SETBONEIKSTATE as c_int, ghoul2, time, boneName, ikState, params) as c_int
    }
}

pub fn trap_G2API_IKMove(ghoul2: *mut c_void, time: c_int, params: *mut c_void) -> c_int {
    unsafe {
        syscall(UI_G2_IKMOVE as c_int, ghoul2, time, params) as c_int
    }
}

pub fn trap_G2API_GetSurfaceName(ghoul2: *mut c_void, surfNumber: c_int, modelIndex: c_int, fillBuf: *mut c_char) {
    unsafe {
        syscall(UI_G2_GETSURFACENAME as c_int, ghoul2, surfNumber, modelIndex, fillBuf);
    }
}

pub fn trap_G2API_AttachG2Model(ghoul2From: *mut c_void, modelIndexFrom: c_int, ghoul2To: *mut c_void, toBoltIndex: c_int, toModel: c_int) -> c_int {
    unsafe {
        syscall(UI_G2_ATTACHG2MODEL as c_int, ghoul2From, modelIndexFrom, ghoul2To, toBoltIndex, toModel) as c_int
    }
}

// Ghoul2 Insert End

// Define syscall command constants as needed
// These would typically be defined in ui_local.h
const UI_PRINT: usize = 0;
const UI_ERROR: usize = 1;
const UI_MILLISECONDS: usize = 2;
const UI_CVAR_REGISTER: usize = 3;
const UI_CVAR_UPDATE: usize = 4;
const UI_CVAR_SET: usize = 5;
const UI_CVAR_VARIABLEVALUE: usize = 6;
const UI_CVAR_VARIABLESTRINGBUFFER: usize = 7;
const UI_CVAR_SETVALUE: usize = 8;
const UI_CVAR_RESET: usize = 9;
const UI_CVAR_CREATE: usize = 10;
const UI_CVAR_INFOSTRINGBUFFER: usize = 11;
const UI_ARGC: usize = 12;
const UI_ARGV: usize = 13;
const UI_CMD_EXECUTETEXT: usize = 14;
const UI_FS_FOPENFILE: usize = 15;
const UI_FS_READ: usize = 16;
const UI_FS_WRITE: usize = 17;
const UI_FS_FCLOSEFILE: usize = 18;
const UI_FS_GETFILELIST: usize = 19;
const UI_R_REGISTERMODEL: usize = 20;
const UI_R_REGISTERSKIN: usize = 21;
const UI_R_REGISTERFONT: usize = 22;
const UI_R_FONT_STRLENPIXELS: usize = 23;
const UI_R_FONT_STRLENCHARS: usize = 24;
const UI_R_FONT_STRHEIGHTPIXELS: usize = 25;
const UI_R_FONT_DRAWSTRING: usize = 26;
const UI_LANGUAGE_ISASIAN: usize = 27;
const UI_LANGUAGE_USESSPACES: usize = 28;
const UI_ANYLANGUAGE_READCHARFROMSTRING: usize = 29;
const UI_R_REGISTERSHADERNOMIP: usize = 30;
const UI_R_SHADERNAMEFROMINDEX: usize = 31;
const UI_R_CLEARSCENE: usize = 32;
const UI_R_ADDREFENTITYTOSCENE: usize = 33;
const UI_R_ADDPOLYTOSCENE: usize = 34;
const UI_R_ADDLIGHTTOSCENE: usize = 35;
const UI_R_RENDERSCENE: usize = 36;
const UI_R_SETCOLOR: usize = 37;
const UI_R_DRAWSTRETCHPIC: usize = 38;
const UI_R_MODELBOUNDS: usize = 39;
const UI_UPDATESCREEN: usize = 40;
const UI_CM_LERPTAG: usize = 41;
const UI_S_STARTLOCALSOUND: usize = 42;
const UI_S_REGISTERSOUND: usize = 43;
const UI_KEY_KEYNUMTOSTRINGBUF: usize = 44;
const UI_KEY_GETBINDINGBUF: usize = 45;
const UI_KEY_SETBINDING: usize = 46;
const UI_KEY_ISDOWN: usize = 47;
const UI_KEY_GETOVERSTRIKEMODE: usize = 48;
const UI_KEY_SETOVERSTRIKEMODE: usize = 49;
const UI_KEY_CLEARSTATES: usize = 50;
const UI_KEY_GETCATCHER: usize = 51;
const UI_KEY_SETCATCHER: usize = 52;
const UI_GETCLIPBOARDDATA: usize = 53;
const UI_GETCLIENTSTATE: usize = 54;
const UI_GETGLCONFIG: usize = 55;
const UI_GETCONFIGSTRING: usize = 56;
const UI_LAN_GETSERVERCOUNT: usize = 57;
const UI_LAN_GETSERVERADDRESSSTRING: usize = 58;
const UI_LAN_GETSERVERINFO: usize = 59;
const UI_LAN_GETSERVERPING: usize = 60;
const UI_LAN_GETPINGQUEUECOUNT: usize = 61;
const UI_LAN_SERVERSTATUS: usize = 62;
const UI_LAN_SAVECACHEDSERVERS: usize = 63;
const UI_LAN_LOADCACHEDSERVERS: usize = 64;
const UI_LAN_RESETPINGS: usize = 65;
const UI_LAN_CLEARPING: usize = 66;
const UI_LAN_GETPING: usize = 67;
const UI_LAN_GETPINGINFO: usize = 68;
const UI_LAN_MARKSERVERVISIBLE: usize = 69;
const UI_LAN_SERVERISVISIBLE: usize = 70;
const UI_LAN_UPDATEVISIBLEPINGS: usize = 71;
const UI_LAN_ADDSERVER: usize = 72;
const UI_LAN_REMOVESERVER: usize = 73;
const UI_LAN_COMPARESERVERS: usize = 74;
const UI_MEMORY_REMAINING: usize = 75;
const UI_GET_CDKEY: usize = 76;
const UI_SET_CDKEY: usize = 77;
const UI_VERIFY_CDKEY: usize = 78;
const UI_PC_ADD_GLOBAL_DEFINE: usize = 79;
const UI_PC_LOAD_SOURCE: usize = 80;
const UI_PC_FREE_SOURCE: usize = 81;
const UI_PC_READ_TOKEN: usize = 82;
const UI_PC_SOURCE_FILE_AND_LINE: usize = 83;
const UI_PC_LOAD_GLOBAL_DEFINES: usize = 84;
const UI_PC_REMOVE_ALL_GLOBAL_DEFINES: usize = 85;
const UI_S_STOPBACKGROUNDTRACK: usize = 86;
const UI_S_STARTBACKGROUNDTRACK: usize = 87;
const UI_REAL_TIME: usize = 88;
const UI_CIN_PLAYCINEMATIC: usize = 89;
const UI_CIN_STOPCINEMATIC: usize = 90;
const UI_CIN_RUNCINEMATIC: usize = 91;
const UI_CIN_DRAWCINEMATIC: usize = 92;
const UI_CIN_SETEXTENTS: usize = 93;
const UI_R_REMAP_SHADER: usize = 94;
const UI_SP_GETNUMLANGUAGES: usize = 95;
const UI_SP_GETLANGUAGENAME: usize = 96;
const UI_SP_GETSTRINGTEXTSTRING: usize = 97;
const UI_G2_LISTSURFACES: usize = 98;
const UI_G2_LISTBONES: usize = 99;
const UI_G2_SETMODELS: usize = 100;
const UI_G2_HAVEWEGHOULMODELS: usize = 101;
const UI_G2_GETBOLT: usize = 102;
const UI_G2_GETBOLT_NOREC: usize = 103;
const UI_G2_GETBOLT_NOREC_NOROT: usize = 104;
const UI_G2_INITGHOUL2MODEL: usize = 105;
const UI_G2_SETSKIN: usize = 106;
const UI_G2_COLLISIONDETECT: usize = 107;
const UI_G2_COLLISIONDETECTCACHE: usize = 108;
const UI_G2_CLEANMODELS: usize = 109;
const UI_G2_ANGLEOVERRIDE: usize = 110;
const UI_G2_PLAYANIM: usize = 111;
const UI_G2_GETBONEANIM: usize = 112;
const UI_G2_GETBONEFRAME: usize = 113;
const UI_G2_GETGLANAME: usize = 114;
const UI_G2_COPYGHOUL2INSTANCE: usize = 115;
const UI_G2_COPYSPECIFICGHOUL2MODEL: usize = 116;
const UI_G2_DUPLICATEGHOUL2INSTANCE: usize = 117;
const UI_G2_HASGHOUL2MODELONINDEX: usize = 118;
const UI_G2_REMOVEGHOUL2MODEL: usize = 119;
const UI_G2_ADDBOLT: usize = 120;
const UI_G2_SETBOLTON: usize = 121;
const UI_G2_SETROOTSURFACE: usize = 122;
const UI_G2_SETSURFACEONOFF: usize = 123;
const UI_G2_SETNEWORIGIN: usize = 124;
const UI_G2_GETTIME: usize = 125;
const UI_G2_SETTIME: usize = 126;
const UI_G2_SETRAGDOLL: usize = 127;
const UI_G2_ANIMATEG2MODELS: usize = 128;
const UI_G2_SETBONEIKSTATE: usize = 129;
const UI_G2_IKMOVE: usize = 130;
const UI_G2_GETSURFACENAME: usize = 131;
const UI_G2_ATTACHG2MODEL: usize = 132;
