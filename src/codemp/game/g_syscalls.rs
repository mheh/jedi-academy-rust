// Copyright (C) 1999-2000 Id Software, Inc.
//
use core::ffi::{c_int, c_char, c_void};

// this file is only included when building a dll
// g_syscalls.asm is included instead when building a qvm

// Syscall function pointer - initialized by dllEntry
static mut syscall: unsafe extern "C" fn(c_int, ...) -> c_int =
    unsafe { core::mem::transmute(-1isize) };

// extern "C" block for Linux compatibility
#[cfg(target_os = "linux")]
extern "C" {
    pub unsafe fn dllEntry(syscallptr: unsafe extern "C" fn(c_int, ...) -> c_int) {
        syscall = syscallptr;
    }
}

#[cfg(not(target_os = "linux"))]
pub unsafe extern "C" fn dllEntry(syscallptr: unsafe extern "C" fn(c_int, ...) -> c_int) {
    syscall = syscallptr;
}

#[inline]
fn PASSFLOAT(x: f32) -> c_int {
    let float_temp: f32 = x;
    unsafe { *((&float_temp as *const f32) as *const c_int) }
}

pub unsafe fn trap_Printf(fmt: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_PRINT, fmt as *const c_void);
}

pub unsafe fn trap_Error(fmt: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_ERROR, fmt as *const c_void);
}

pub unsafe fn trap_Milliseconds() -> c_int {
    syscall(crate::game::g_syscalls_h::G_MILLISECONDS)
}


//rww - precision timer funcs... -ALWAYS- call end after start with supplied ptr, or you'll get a nasty memory leak.
//not that you should be using these outside of debug anyway.. because you shouldn't be. So don't.

//Start should be suppled with a pointer to an empty pointer (e.g. void *blah; trap_PrecisionTimer_Start(&blah);),
//the empty pointer will be filled with an exe address to our timer (this address means nothing in vm land however).
//You must pass this pointer back unmodified to the timer end func.
pub unsafe fn trap_PrecisionTimer_Start(theNewTimer: *mut *mut c_void) {
    syscall(crate::game::g_syscalls_h::G_PRECISIONTIMER_START, theNewTimer as *const c_void);
}

//If you're using the above example, the appropriate call for this is int result = trap_PrecisionTimer_End(blah);
pub unsafe fn trap_PrecisionTimer_End(theTimer: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::G_PRECISIONTIMER_END, theTimer as *const c_void)
}

pub unsafe fn trap_Cvar_Register(cvar: *mut crate::q_shared_h::vmCvar_t, var_name: *const c_char, value: *const c_char, flags: c_int) {
    syscall(crate::game::g_syscalls_h::G_CVAR_REGISTER, cvar as *const c_void, var_name as *const c_void, value as *const c_void, flags);
}

pub unsafe fn trap_Cvar_Update(cvar: *mut crate::q_shared_h::vmCvar_t) {
    syscall(crate::game::g_syscalls_h::G_CVAR_UPDATE, cvar as *const c_void);
}

pub unsafe fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_CVAR_SET, var_name as *const c_void, value as *const c_void);
}

pub unsafe fn trap_Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_CVAR_VARIABLE_INTEGER_VALUE, var_name as *const c_void)
}

pub unsafe fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) {
    syscall(crate::game::g_syscalls_h::G_CVAR_VARIABLE_STRING_BUFFER, var_name as *const c_void, buffer as *const c_void, bufsize);
}

pub unsafe fn trap_Argc() -> c_int {
    syscall(crate::game::g_syscalls_h::G_ARGC)
}

pub unsafe fn trap_Argv(n: c_int, buffer: *mut c_char, bufferLength: c_int) {
    syscall(crate::game::g_syscalls_h::G_ARGV, n, buffer as *const c_void, bufferLength);
}

pub unsafe fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut crate::qcommon_h::fileHandle_t, mode: crate::qcommon_h::fsMode_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_FS_FOPEN_FILE, qpath as *const c_void, f as *const c_void, mode)
}

pub unsafe fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: crate::qcommon_h::fileHandle_t) {
    syscall(crate::game::g_syscalls_h::G_FS_READ, buffer, len, f);
}

pub unsafe fn trap_FS_Write(buffer: *const c_void, len: c_int, f: crate::qcommon_h::fileHandle_t) {
    syscall(crate::game::g_syscalls_h::G_FS_WRITE, buffer, len, f);
}

pub unsafe fn trap_FS_FCloseFile(f: crate::qcommon_h::fileHandle_t) {
    syscall(crate::game::g_syscalls_h::G_FS_FCLOSE_FILE, f);
}

pub unsafe fn trap_SendConsoleCommand(exec_when: c_int, text: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_SEND_CONSOLE_COMMAND, exec_when, text as *const c_void);
}

pub unsafe fn trap_LocateGameData(gEnts: *mut crate::g_local_h::gentity_t, numGEntities: c_int, sizeofGEntity_t: c_int,
                         clients: *mut crate::q_shared_h::playerState_t, sizeofGClient: c_int) {
    syscall(crate::game::g_syscalls_h::G_LOCATE_GAME_DATA, gEnts as *const c_void, numGEntities, sizeofGEntity_t, clients as *const c_void, sizeofGClient);
}

pub unsafe fn trap_DropClient(clientNum: c_int, reason: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_DROP_CLIENT, clientNum, reason as *const c_void);
}

pub unsafe fn trap_SendServerCommand(clientNum: c_int, text: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_SEND_SERVER_COMMAND, clientNum, text as *const c_void);
}

pub unsafe fn trap_SetConfigstring(num: c_int, string: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_SET_CONFIGSTRING, num, string as *const c_void);
}

pub unsafe fn trap_GetConfigstring(num: c_int, buffer: *mut c_char, bufferSize: c_int) {
    syscall(crate::game::g_syscalls_h::G_GET_CONFIGSTRING, num, buffer as *const c_void, bufferSize);
}

pub unsafe fn trap_GetUserinfo(num: c_int, buffer: *mut c_char, bufferSize: c_int) {
    syscall(crate::game::g_syscalls_h::G_GET_USERINFO, num, buffer as *const c_void, bufferSize);
}

pub unsafe fn trap_SetUserinfo(num: c_int, buffer: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_SET_USERINFO, num, buffer as *const c_void);
}

pub unsafe fn trap_GetServerinfo(buffer: *mut c_char, bufferSize: c_int) {
    syscall(crate::game::g_syscalls_h::G_GET_SERVERINFO, buffer as *const c_void, bufferSize);
}

//server culling to reduce traffic on open maps -rww
pub unsafe fn trap_SetServerCull(cullDistance: f32) {
    syscall(crate::game::g_syscalls_h::G_SET_SERVER_CULL, PASSFLOAT(cullDistance));
}

pub unsafe fn trap_SetBrushModel(ent: *mut crate::g_local_h::gentity_t, name: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_SET_BRUSH_MODEL, ent as *const c_void, name as *const c_void);
}

pub unsafe fn trap_Trace(results: *mut crate::q_shared_h::trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passEntityNum: c_int, contentmask: c_int) {
    syscall(crate::game::g_syscalls_h::G_TRACE, results as *const c_void, start as *const c_void, mins as *const c_void, maxs as *const c_void, end as *const c_void, passEntityNum, contentmask, 0, 10);
}

//g2TraceType 0 is no g2 col, 1 is collision against anything not EF_DEAD, 2 is collision against all.
pub unsafe fn trap_G2Trace(results: *mut crate::q_shared_h::trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passEntityNum: c_int, contentmask: c_int, g2TraceType: c_int, traceLod: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2TRACE, results as *const c_void, start as *const c_void, mins as *const c_void, maxs as *const c_void, end as *const c_void, passEntityNum, contentmask, g2TraceType, traceLod);
}

pub unsafe fn trap_PointContents(point: *const [f32; 3], passEntityNum: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_POINT_CONTENTS, point as *const c_void, passEntityNum)
}


pub unsafe fn trap_InPVS(p1: *const [f32; 3], p2: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_IN_PVS, p1 as *const c_void, p2 as *const c_void)
}

pub unsafe fn trap_InPVSIgnorePortals(p1: *const [f32; 3], p2: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_IN_PVS_IGNORE_PORTALS, p1 as *const c_void, p2 as *const c_void)
}

pub unsafe fn trap_AdjustAreaPortalState(ent: *mut crate::g_local_h::gentity_t, open: c_int) {
    syscall(crate::game::g_syscalls_h::G_ADJUST_AREA_PORTAL_STATE, ent as *const c_void, open);
}

pub unsafe fn trap_AreasConnected(area1: c_int, area2: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_AREAS_CONNECTED, area1, area2)
}

pub unsafe fn trap_LinkEntity(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_LINKENTITY, ent as *const c_void);
}

pub unsafe fn trap_UnlinkEntity(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_UNLINKENTITY, ent as *const c_void);
}

pub unsafe fn trap_EntitiesInBox(mins: *const [f32; 3], maxs: *const [f32; 3], list: *mut c_int, maxcount: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ENTITIES_IN_BOX, mins as *const c_void, maxs as *const c_void, list as *const c_void, maxcount)
}

pub unsafe fn trap_EntityContact(mins: *const [f32; 3], maxs: *const [f32; 3], ent: *const crate::g_local_h::gentity_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ENTITY_CONTACT, mins as *const c_void, maxs as *const c_void, ent as *const c_void)
}

pub unsafe fn trap_BotAllocateClient() -> c_int {
    syscall(crate::game::g_syscalls_h::G_BOT_ALLOCATE_CLIENT)
}

pub unsafe fn trap_BotFreeClient(clientNum: c_int) {
    syscall(crate::game::g_syscalls_h::G_BOT_FREE_CLIENT, clientNum);
}

pub unsafe fn trap_GetUsercmd(clientNum: c_int, cmd: *mut crate::q_shared_h::usercmd_t) {
    syscall(crate::game::g_syscalls_h::G_GET_USERCMD, clientNum, cmd as *const c_void);
}

pub unsafe fn trap_GetEntityToken(buffer: *mut c_char, bufferSize: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_GET_ENTITY_TOKEN, buffer as *const c_void, bufferSize)
}

pub unsafe fn trap_SiegePersSet(pers: *mut crate::g_local_h::siegePers_t) {
    syscall(crate::game::g_syscalls_h::G_SIEGEPERSSET, pers as *const c_void);
}
pub unsafe fn trap_SiegePersGet(pers: *mut crate::g_local_h::siegePers_t) {
    syscall(crate::game::g_syscalls_h::G_SIEGEPERSGET, pers as *const c_void);
}

pub unsafe fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_FS_GETFILELIST, path as *const c_void, extension as *const c_void, listbuf as *const c_void, bufsize)
}

pub unsafe fn trap_DebugPolygonCreate(color: c_int, numPoints: c_int, points: *mut [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_DEBUG_POLYGON_CREATE, color, numPoints, points as *const c_void)
}

pub unsafe fn trap_DebugPolygonDelete(id: c_int) {
    syscall(crate::game::g_syscalls_h::G_DEBUG_POLYGON_DELETE, id);
}

pub unsafe fn trap_RealTime(qtime: *mut crate::q_shared_h::qtime_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_REAL_TIME, qtime as *const c_void)
}

pub unsafe fn trap_SnapVector(v: *mut f32) {
    syscall(crate::game::g_syscalls_h::G_SNAPVECTOR, v as *const c_void);
}

pub unsafe fn trap_TraceCapsule(results: *mut crate::q_shared_h::trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passEntityNum: c_int, contentmask: c_int) {
    syscall(crate::game::g_syscalls_h::G_TRACECAPSULE, results as *const c_void, start as *const c_void, mins as *const c_void, maxs as *const c_void, end as *const c_void, passEntityNum, contentmask, 0, 10);
}

pub unsafe fn trap_EntityContactCapsule(mins: *const [f32; 3], maxs: *const [f32; 3], ent: *const crate::g_local_h::gentity_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ENTITY_CONTACTCAPSULE, mins as *const c_void, maxs as *const c_void, ent as *const c_void)
}

//qboolean trap_SP_RegisterServer( const char *package )
//{
//	return syscall( SP_REGISTER_SERVER_CMD, package );
//}

pub unsafe fn trap_SP_GetStringTextString(text: *const c_char, buffer: *mut c_char, bufferLength: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::SP_GETSTRINGTEXTSTRING, text as *const c_void, buffer as *const c_void, bufferLength)
}

pub unsafe fn trap_ROFF_Clean() -> c_int {
    syscall(crate::game::g_syscalls_h::G_ROFF_CLEAN)
}

pub unsafe fn trap_ROFF_UpdateEntities() {
    syscall(crate::game::g_syscalls_h::G_ROFF_UPDATE_ENTITIES);
}

pub unsafe fn trap_ROFF_Cache(file: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ROFF_CACHE, file as *const c_void)
}

pub unsafe fn trap_ROFF_Play(entID: c_int, roffID: c_int, doTranslation: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ROFF_PLAY, entID, roffID, doTranslation)
}

pub unsafe fn trap_ROFF_Purge_Ent(entID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ROFF_PURGE_ENT, entID)
}

//rww - dynamic vm memory allocation!
pub unsafe fn trap_TrueMalloc(ptr: *mut *mut c_void, size: c_int) {
    syscall(crate::game::g_syscalls_h::G_TRUEMALLOC, ptr as *const c_void, size);
}

pub unsafe fn trap_TrueFree(ptr: *mut *mut c_void) {
    syscall(crate::game::g_syscalls_h::G_TRUEFREE, ptr as *const c_void);
}

//rww - icarus traps
pub unsafe fn trap_ICARUS_RunScript(ent: *mut crate::g_local_h::gentity_t, name: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_RUNSCRIPT, ent as *const c_void, name as *const c_void)
}

pub unsafe fn trap_ICARUS_RegisterScript(name: *const c_char, bCalledDuringInterrogate: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_REGISTERSCRIPT, name as *const c_void, bCalledDuringInterrogate)
}

pub unsafe fn trap_ICARUS_Init() {
    syscall(crate::game::g_syscalls_h::G_ICARUS_INIT);
}

pub unsafe fn trap_ICARUS_ValidEnt(ent: *mut crate::g_local_h::gentity_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_VALIDENT, ent as *const c_void)
}

pub unsafe fn trap_ICARUS_IsInitialized(entID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_ISINITIALIZED, entID)
}

pub unsafe fn trap_ICARUS_MaintainTaskManager(entID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_MAINTAINTASKMANAGER, entID)
}

pub unsafe fn trap_ICARUS_IsRunning(entID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_ISRUNNING, entID)
}

pub unsafe fn trap_ICARUS_TaskIDPending(ent: *mut crate::g_local_h::gentity_t, taskID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_TASKIDPENDING, ent as *const c_void, taskID)
}

pub unsafe fn trap_ICARUS_InitEnt(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_INITENT, ent as *const c_void);
}

pub unsafe fn trap_ICARUS_FreeEnt(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_FREEENT, ent as *const c_void);
}

pub unsafe fn trap_ICARUS_AssociateEnt(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_ASSOCIATEENT, ent as *const c_void);
}

pub unsafe fn trap_ICARUS_Shutdown() {
    syscall(crate::game::g_syscalls_h::G_ICARUS_SHUTDOWN);
}

pub unsafe fn trap_ICARUS_TaskIDSet(ent: *mut crate::g_local_h::gentity_t, taskType: c_int, taskID: c_int) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_TASKIDSET, ent as *const c_void, taskType, taskID);
}

pub unsafe fn trap_ICARUS_TaskIDComplete(ent: *mut crate::g_local_h::gentity_t, taskType: c_int) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_TASKIDCOMPLETE, ent as *const c_void, taskType);
}

pub unsafe fn trap_ICARUS_SetVar(taskID: c_int, entID: c_int, type_name: *const c_char, data: *const c_char) {
    syscall(crate::game::g_syscalls_h::G_ICARUS_SETVAR, taskID, entID, type_name as *const c_void, data as *const c_void);
}

pub unsafe fn trap_ICARUS_VariableDeclared(type_name: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_VARIABLEDECLARED, type_name as *const c_void)
}

pub unsafe fn trap_ICARUS_GetFloatVariable(name: *const c_char, value: *mut f32) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_GETFLOATVARIABLE, name as *const c_void, value as *const c_void)
}

pub unsafe fn trap_ICARUS_GetStringVariable(name: *const c_char, value: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_GETSTRINGVARIABLE, name as *const c_void, value as *const c_void)
}

pub unsafe fn trap_ICARUS_GetVectorVariable(name: *const c_char, value: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_ICARUS_GETVECTORVARIABLE, name as *const c_void, value as *const c_void)
}

//rww - BEGIN NPC NAV TRAPS
pub unsafe fn trap_Nav_Init() {
    syscall(crate::game::g_syscalls_h::G_NAV_INIT);
}

pub unsafe fn trap_Nav_Free() {
    syscall(crate::game::g_syscalls_h::G_NAV_FREE);
}

pub unsafe fn trap_Nav_Load(filename: *const c_char, checksum: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_LOAD, filename as *const c_void, checksum)
}

pub unsafe fn trap_Nav_Save(filename: *const c_char, checksum: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_SAVE, filename as *const c_void, checksum)
}

pub unsafe fn trap_Nav_AddRawPoint(point: [f32; 3], flags: c_int, radius: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_ADDRAWPOINT, &point as *const _ as *const c_void, flags, radius)
}

pub unsafe fn trap_Nav_CalculatePaths(recalc: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_CALCULATEPATHS, recalc);
}

pub unsafe fn trap_Nav_HardConnect(first: c_int, second: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_HARDCONNECT, first, second);
}

pub unsafe fn trap_Nav_ShowNodes() {
    syscall(crate::game::g_syscalls_h::G_NAV_SHOWNODES);
}

pub unsafe fn trap_Nav_ShowEdges() {
    syscall(crate::game::g_syscalls_h::G_NAV_SHOWEDGES);
}

pub unsafe fn trap_Nav_ShowPath(start: c_int, end: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_SHOWPATH, start, end);
}

pub unsafe fn trap_Nav_GetNearestNode(ent: *mut crate::g_local_h::gentity_t, lastID: c_int, flags: c_int, targetID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNEARESTNODE, ent as *const c_void, lastID, flags, targetID)
}

pub unsafe fn trap_Nav_GetBestNode(startID: c_int, endID: c_int, rejectID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETBESTNODE, startID, endID, rejectID)
}

pub unsafe fn trap_Nav_GetNodePosition(nodeID: c_int, out: *mut [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNODEPOSITION, nodeID, out as *const c_void)
}

pub unsafe fn trap_Nav_GetNodeNumEdges(nodeID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNODENUMEDGES, nodeID)
}

pub unsafe fn trap_Nav_GetNodeEdge(nodeID: c_int, edge: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNODEEDGE, nodeID, edge)
}

pub unsafe fn trap_Nav_GetNumNodes() -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNUMNODES)
}

pub unsafe fn trap_Nav_Connected(startID: c_int, endID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_CONNECTED, startID, endID)
}

pub unsafe fn trap_Nav_GetPathCost(startID: c_int, endID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETPATHCOST, startID, endID)
}

pub unsafe fn trap_Nav_GetEdgeCost(startID: c_int, endID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETEDGECOST, startID, endID)
}

pub unsafe fn trap_Nav_GetProjectedNode(origin: [f32; 3], nodeID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETPROJECTEDNODE, &origin as *const _ as *const c_void, nodeID)
}

pub unsafe fn trap_Nav_CheckFailedNodes(ent: *mut crate::g_local_h::gentity_t) {
    syscall(crate::game::g_syscalls_h::G_NAV_CHECKFAILEDNODES, ent as *const c_void);
}

pub unsafe fn trap_Nav_AddFailedNode(ent: *mut crate::g_local_h::gentity_t, nodeID: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_ADDFAILEDNODE, ent as *const c_void, nodeID);
}

pub unsafe fn trap_Nav_NodeFailed(ent: *mut crate::g_local_h::gentity_t, nodeID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_NODEFAILED, ent as *const c_void, nodeID)
}

pub unsafe fn trap_Nav_NodesAreNeighbors(startID: c_int, endID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_NODESARENEIGHBORS, startID, endID)
}

pub unsafe fn trap_Nav_ClearFailedEdge(failedEdge: *mut crate::g_local_h::failedEdge_t) {
    syscall(crate::game::g_syscalls_h::G_NAV_CLEARFAILEDEDGE, failedEdge as *const c_void);
}

pub unsafe fn trap_Nav_ClearAllFailedEdges() {
    syscall(crate::game::g_syscalls_h::G_NAV_CLEARALLFAILEDEDGES);
}

pub unsafe fn trap_Nav_EdgeFailed(startID: c_int, endID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_EDGEFAILED, startID, endID)
}

pub unsafe fn trap_Nav_AddFailedEdge(entID: c_int, startID: c_int, endID: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_ADDFAILEDEDGE, entID, startID, endID);
}

pub unsafe fn trap_Nav_CheckFailedEdge(failedEdge: *mut crate::g_local_h::failedEdge_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_CHECKFAILEDEDGE, failedEdge as *const c_void)
}

pub unsafe fn trap_Nav_CheckAllFailedEdges() {
    syscall(crate::game::g_syscalls_h::G_NAV_CHECKALLFAILEDEDGES);
}

pub unsafe fn trap_Nav_RouteBlocked(startID: c_int, testEdgeID: c_int, endID: c_int, rejectRank: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_ROUTEBLOCKED, startID, testEdgeID, endID, rejectRank)
}

pub unsafe fn trap_Nav_GetBestNodeAltRoute(startID: c_int, endID: c_int, pathCost: *mut c_int, rejectID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETBESTNODEALTROUTE, startID, endID, pathCost as *const c_void, rejectID)
}

pub unsafe fn trap_Nav_GetBestNodeAltRoute2(startID: c_int, endID: c_int, rejectID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETBESTNODEALT2, startID, endID, rejectID)
}

pub unsafe fn trap_Nav_GetBestPathBetweenEnts(ent: *mut crate::g_local_h::gentity_t, goal: *mut crate::g_local_h::gentity_t, flags: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETBESTPATHBETWEENENTS, ent as *const c_void, goal as *const c_void, flags)
}

pub unsafe fn trap_Nav_GetNodeRadius(nodeID: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETNODERADIUS, nodeID)
}

pub unsafe fn trap_Nav_CheckBlockedEdges() {
    syscall(crate::game::g_syscalls_h::G_NAV_CHECKBLOCKEDEDGES);
}

pub unsafe fn trap_Nav_ClearCheckedNodes() {
    syscall(crate::game::g_syscalls_h::G_NAV_CLEARCHECKEDNODES);
}

pub unsafe fn trap_Nav_CheckedNode(wayPoint: c_int, ent: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_CHECKEDNODE, wayPoint, ent)
}

pub unsafe fn trap_Nav_SetCheckedNode(wayPoint: c_int, ent: c_int, value: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_SETCHECKEDNODE, wayPoint, ent, value);
}

pub unsafe fn trap_Nav_FlagAllNodes(newFlag: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_FLAGALLNODES, newFlag);
}

pub unsafe fn trap_Nav_GetPathsCalculated() -> c_int {
    syscall(crate::game::g_syscalls_h::G_NAV_GETPATHSCALCULATED)
}

pub unsafe fn trap_Nav_SetPathsCalculated(newVal: c_int) {
    syscall(crate::game::g_syscalls_h::G_NAV_SETPATHSCALCULATED, newVal);
}
//rww - END NPC NAV TRAPS

pub unsafe fn trap_SV_RegisterSharedMemory(memory: *mut c_char) {
    syscall(crate::game::g_syscalls_h::G_SET_SHARED_BUFFER, memory as *const c_void);
}

// BotLib traps start here
pub unsafe fn trap_BotLibSetup() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_SETUP)
}

pub unsafe fn trap_BotLibShutdown() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_SHUTDOWN)
}

pub unsafe fn trap_BotLibVarSet(var_name: *mut c_char, value: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_LIBVAR_SET, var_name as *const c_void, value as *const c_void)
}

pub unsafe fn trap_BotLibVarGet(var_name: *mut c_char, value: *mut c_char, size: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_LIBVAR_GET, var_name as *const c_void, value as *const c_void, size)
}

pub unsafe fn trap_BotLibDefine(string: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_PC_ADD_GLOBAL_DEFINE, string as *const c_void)
}

pub unsafe fn trap_BotLibStartFrame(time: f32) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_START_FRAME, PASSFLOAT(time))
}

pub unsafe fn trap_BotLibLoadMap(mapname: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_LOAD_MAP, mapname as *const c_void)
}

pub unsafe fn trap_BotLibUpdateEntity(ent: c_int, bue: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_UPDATENTITY, ent, bue)
}

pub unsafe fn trap_BotLibTest(parm0: c_int, parm1: *mut c_char, parm2: [f32; 3], parm3: [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_TEST, parm0, parm1 as *const c_void, &parm2 as *const _ as *const c_void, &parm3 as *const _ as *const c_void)
}

pub unsafe fn trap_BotGetSnapshotEntity(clientNum: c_int, sequence: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_GET_SNAPSHOT_ENTITY, clientNum, sequence)
}

pub unsafe fn trap_BotGetServerCommand(clientNum: c_int, message: *mut c_char, size: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_GET_CONSOLE_MESSAGE, clientNum, message as *const c_void, size)
}

pub unsafe fn trap_BotUserCommand(clientNum: c_int, ucmd: *mut crate::q_shared_h::usercmd_t) {
    syscall(crate::game::g_syscalls_h::BOTLIB_USER_COMMAND, clientNum, ucmd as *const c_void);
}

pub unsafe fn trap_AAS_EntityInfo(entnum: c_int, info: *mut c_void) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_ENTITY_INFO, entnum, info);
}

pub unsafe fn trap_AAS_Initialized() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_INITIALIZED)
}

pub unsafe fn trap_AAS_PresenceTypeBoundingBox(presencetype: c_int, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX, presencetype, mins as *const c_void, maxs as *const c_void);
}

pub unsafe fn trap_AAS_Time() -> f32 {
    let temp: c_int;
    temp = syscall(crate::game::g_syscalls_h::BOTLIB_AAS_TIME);
    (*((&temp as *const c_int) as *const f32))
}

pub unsafe fn trap_AAS_PointAreaNum(point: [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_POINT_AREA_NUM, &point as *const _ as *const c_void)
}

pub unsafe fn trap_AAS_PointReachabilityAreaIndex(point: [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX, &point as *const _ as *const c_void)
}

pub unsafe fn trap_AAS_TraceAreas(start: [f32; 3], end: [f32; 3], areas: *mut c_int, points: *mut [f32; 3], maxareas: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_TRACE_AREAS, &start as *const _ as *const c_void, &end as *const _ as *const c_void, areas as *const c_void, points as *const c_void, maxareas)
}

pub unsafe fn trap_AAS_BBoxAreas(absmins: [f32; 3], absmaxs: [f32; 3], areas: *mut c_int, maxareas: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_BBOX_AREAS, &absmins as *const _ as *const c_void, &absmaxs as *const _ as *const c_void, areas as *const c_void, maxareas)
}

pub unsafe fn trap_AAS_AreaInfo(areanum: c_int, info: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_AREA_INFO, areanum, info)
}

pub unsafe fn trap_AAS_PointContents(point: [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_POINT_CONTENTS, &point as *const _ as *const c_void)
}

pub unsafe fn trap_AAS_NextBSPEntity(ent: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_NEXT_BSP_ENTITY, ent)
}

pub unsafe fn trap_AAS_ValueForBSPEpairKey(ent: c_int, key: *mut c_char, value: *mut c_char, size: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY, ent, key as *const c_void, value as *const c_void, size)
}

pub unsafe fn trap_AAS_VectorForBSPEpairKey(ent: c_int, key: *mut c_char, v: *mut [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY, ent, key as *const c_void, v as *const c_void)
}

pub unsafe fn trap_AAS_FloatForBSPEpairKey(ent: c_int, key: *mut c_char, value: *mut f32) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY, ent, key as *const c_void, value as *const c_void)
}

pub unsafe fn trap_AAS_IntForBSPEpairKey(ent: c_int, key: *mut c_char, value: *mut c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY, ent, key as *const c_void, value as *const c_void)
}

pub unsafe fn trap_AAS_AreaReachability(areanum: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_AREA_REACHABILITY, areanum)
}

pub unsafe fn trap_AAS_AreaTravelTimeToGoalArea(areanum: c_int, origin: [f32; 3], goalareanum: c_int, travelflags: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA, areanum, &origin as *const _ as *const c_void, goalareanum, travelflags)
}

pub unsafe fn trap_AAS_EnableRoutingArea(areanum: c_int, enable: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_ENABLE_ROUTING_AREA, areanum, enable)
}

pub unsafe fn trap_AAS_PredictRoute(route: *mut c_void, areanum: c_int, origin: [f32; 3],
                            goalareanum: c_int, travelflags: c_int, maxareas: c_int, maxtime: c_int,
                            stopevent: c_int, stopcontents: c_int, stoptfl: c_int, stopareanum: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_PREDICT_ROUTE, route, areanum, &origin as *const _ as *const c_void, goalareanum, travelflags, maxareas, maxtime, stopevent, stopcontents, stoptfl, stopareanum)
}

pub unsafe fn trap_AAS_AlternativeRouteGoals(start: [f32; 3], startareanum: c_int, goal: [f32; 3], goalareanum: c_int, travelflags: c_int,
                                        altroutegoals: *mut c_void, maxaltroutegoals: c_int,
                                        type_: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL, &start as *const _ as *const c_void, startareanum, &goal as *const _ as *const c_void, goalareanum, travelflags, altroutegoals, maxaltroutegoals, type_)
}

pub unsafe fn trap_AAS_Swimming(origin: [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_SWIMMING, &origin as *const _ as *const c_void)
}

pub unsafe fn trap_AAS_PredictClientMovement(move_: *mut c_void, entnum: c_int, origin: [f32; 3], presencetype: c_int, onground: c_int, velocity: [f32; 3], cmdmove: [f32; 3], cmdframes: c_int, maxframes: c_int, frametime: f32, stopevent: c_int, stopareanum: c_int, visualize: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT, move_, entnum, &origin as *const _ as *const c_void, presencetype, onground, &velocity as *const _ as *const c_void, &cmdmove as *const _ as *const c_void, cmdframes, maxframes, PASSFLOAT(frametime), stopevent, stopareanum, visualize)
}

pub unsafe fn trap_EA_Say(client: c_int, str_: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_SAY, client, str_ as *const c_void);
}

pub unsafe fn trap_EA_SayTeam(client: c_int, str_: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_SAY_TEAM, client, str_ as *const c_void);
}

pub unsafe fn trap_EA_Command(client: c_int, command: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_COMMAND, client, command as *const c_void);
}

pub unsafe fn trap_EA_Action(client: c_int, action: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_ACTION, client, action);
}

pub unsafe fn trap_EA_Gesture(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_GESTURE, client);
}

pub unsafe fn trap_EA_Talk(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_TALK, client);
}

pub unsafe fn trap_EA_Attack(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_ATTACK, client);
}

pub unsafe fn trap_EA_Alt_Attack(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_ALT_ATTACK, client);
}

pub unsafe fn trap_EA_ForcePower(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_FORCEPOWER, client);
}

pub unsafe fn trap_EA_Use(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_USE, client);
}

pub unsafe fn trap_EA_Respawn(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_RESPAWN, client);
}

pub unsafe fn trap_EA_Crouch(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_CROUCH, client);
}

pub unsafe fn trap_EA_MoveUp(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_UP, client);
}

pub unsafe fn trap_EA_MoveDown(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_DOWN, client);
}

pub unsafe fn trap_EA_MoveForward(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_FORWARD, client);
}

pub unsafe fn trap_EA_MoveBack(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_BACK, client);
}

pub unsafe fn trap_EA_MoveLeft(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_LEFT, client);
}

pub unsafe fn trap_EA_MoveRight(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE_RIGHT, client);
}

pub unsafe fn trap_EA_SelectWeapon(client: c_int, weapon: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_SELECT_WEAPON, client, weapon);
}

pub unsafe fn trap_EA_Jump(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_JUMP, client);
}

pub unsafe fn trap_EA_DelayedJump(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_DELAYED_JUMP, client);
}

pub unsafe fn trap_EA_Move(client: c_int, dir: [f32; 3], speed: f32) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_MOVE, client, &dir as *const _ as *const c_void, PASSFLOAT(speed));
}

pub unsafe fn trap_EA_View(client: c_int, viewangles: [f32; 3]) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_VIEW, client, &viewangles as *const _ as *const c_void);
}

pub unsafe fn trap_EA_EndRegular(client: c_int, thinktime: f32) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_END_REGULAR, client, PASSFLOAT(thinktime));
}

pub unsafe fn trap_EA_GetInput(client: c_int, thinktime: f32, input: *mut c_void) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_GET_INPUT, client, PASSFLOAT(thinktime), input);
}

pub unsafe fn trap_EA_ResetInput(client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_EA_RESET_INPUT, client);
}

pub unsafe fn trap_BotLoadCharacter(charfile: *mut c_char, skill: f32) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_LOAD_CHARACTER, charfile as *const c_void, PASSFLOAT(skill))
}

pub unsafe fn trap_BotFreeCharacter(character: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_CHARACTER, character);
}

pub unsafe fn trap_Characteristic_Float(character: c_int, index: c_int) -> f32 {
    let temp: c_int;
    temp = syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHARACTERISTIC_FLOAT, character, index);
    (*((&temp as *const c_int) as *const f32))
}

pub unsafe fn trap_Characteristic_BFloat(character: c_int, index: c_int, min: f32, max: f32) -> f32 {
    let temp: c_int;
    temp = syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHARACTERISTIC_BFLOAT, character, index, PASSFLOAT(min), PASSFLOAT(max));
    (*((&temp as *const c_int) as *const f32))
}

pub unsafe fn trap_Characteristic_Integer(character: c_int, index: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHARACTERISTIC_INTEGER, character, index)
}

pub unsafe fn trap_Characteristic_BInteger(character: c_int, index: c_int, min: c_int, max: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHARACTERISTIC_BINTEGER, character, index, min, max)
}

pub unsafe fn trap_Characteristic_String(character: c_int, index: c_int, buf: *mut c_char, size: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHARACTERISTIC_STRING, character, index, buf as *const c_void, size);
}

pub unsafe fn trap_BotAllocChatState() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ALLOC_CHAT_STATE)
}

pub unsafe fn trap_BotFreeChatState(handle: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_CHAT_STATE, handle);
}

pub unsafe fn trap_BotQueueConsoleMessage(chatstate: c_int, type_: c_int, message: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_QUEUE_CONSOLE_MESSAGE, chatstate, type_, message as *const c_void);
}

pub unsafe fn trap_BotRemoveConsoleMessage(chatstate: c_int, handle: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_REMOVE_CONSOLE_MESSAGE, chatstate, handle);
}

pub unsafe fn trap_BotNextConsoleMessage(chatstate: c_int, cm: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_NEXT_CONSOLE_MESSAGE, chatstate, cm)
}

pub unsafe fn trap_BotNumConsoleMessages(chatstate: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_NUM_CONSOLE_MESSAGE, chatstate)
}

pub unsafe fn trap_BotInitialChat(chatstate: c_int, type_: *mut c_char, mcontext: c_int, var0: *mut c_char, var1: *mut c_char, var2: *mut c_char, var3: *mut c_char, var4: *mut c_char, var5: *mut c_char, var6: *mut c_char, var7: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_INITIAL_CHAT, chatstate, type_ as *const c_void, mcontext, var0 as *const c_void, var1 as *const c_void, var2 as *const c_void, var3 as *const c_void, var4 as *const c_void, var5 as *const c_void, var6 as *const c_void, var7 as *const c_void);
}

pub unsafe fn trap_BotNumInitialChats(chatstate: c_int, type_: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_NUM_INITIAL_CHATS, chatstate, type_ as *const c_void)
}

pub unsafe fn trap_BotReplyChat(chatstate: c_int, message: *mut c_char, mcontext: c_int, vcontext: c_int, var0: *mut c_char, var1: *mut c_char, var2: *mut c_char, var3: *mut c_char, var4: *mut c_char, var5: *mut c_char, var6: *mut c_char, var7: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_REPLY_CHAT, chatstate, message as *const c_void, mcontext, vcontext, var0 as *const c_void, var1 as *const c_void, var2 as *const c_void, var3 as *const c_void, var4 as *const c_void, var5 as *const c_void, var6 as *const c_void, var7 as *const c_void)
}

pub unsafe fn trap_BotChatLength(chatstate: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHAT_LENGTH, chatstate)
}

pub unsafe fn trap_BotEnterChat(chatstate: c_int, client: c_int, sendto: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ENTER_CHAT, chatstate, client, sendto);
}

pub unsafe fn trap_BotGetChatMessage(chatstate: c_int, buf: *mut c_char, size: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_CHAT_MESSAGE, chatstate, buf as *const c_void, size);
}

pub unsafe fn trap_StringContains(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_STRING_CONTAINS, str1 as *const c_void, str2 as *const c_void, casesensitive)
}

pub unsafe fn trap_BotFindMatch(str_: *mut c_char, match_: *mut c_void, context: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FIND_MATCH, str_ as *const c_void, match_, context)
}

pub unsafe fn trap_BotMatchVariable(match_: *mut c_void, variable: c_int, buf: *mut c_char, size: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_MATCH_VARIABLE, match_, variable, buf as *const c_void, size);
}

pub unsafe fn trap_UnifyWhiteSpaces(string: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_UNIFY_WHITE_SPACES, string as *const c_void);
}

pub unsafe fn trap_BotReplaceSynonyms(string: *mut c_char, context: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_REPLACE_SYNONYMS, string as *const c_void, context);
}

pub unsafe fn trap_BotLoadChatFile(chatstate: c_int, chatfile: *mut c_char, chatname: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_LOAD_CHAT_FILE, chatstate, chatfile as *const c_void, chatname as *const c_void)
}

pub unsafe fn trap_BotSetChatGender(chatstate: c_int, gender: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_SET_CHAT_GENDER, chatstate, gender);
}

pub unsafe fn trap_BotSetChatName(chatstate: c_int, name: *mut c_char, client: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_SET_CHAT_NAME, chatstate, name as *const c_void, client);
}

pub unsafe fn trap_BotResetGoalState(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_GOAL_STATE, goalstate);
}

pub unsafe fn trap_BotResetAvoidGoals(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_AVOID_GOALS, goalstate);
}

pub unsafe fn trap_BotRemoveFromAvoidGoals(goalstate: c_int, number: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_REMOVE_FROM_AVOID_GOALS, goalstate, number);
}

pub unsafe fn trap_BotPushGoal(goalstate: c_int, goal: *mut c_void) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_PUSH_GOAL, goalstate, goal);
}

pub unsafe fn trap_BotPopGoal(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_POP_GOAL, goalstate);
}

pub unsafe fn trap_BotEmptyGoalStack(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_EMPTY_GOAL_STACK, goalstate);
}

pub unsafe fn trap_BotDumpAvoidGoals(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_DUMP_AVOID_GOALS, goalstate);
}

pub unsafe fn trap_BotDumpGoalStack(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_DUMP_GOAL_STACK, goalstate);
}

pub unsafe fn trap_BotGoalName(number: c_int, name: *mut c_char, size: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GOAL_NAME, number, name as *const c_void, size);
}

pub unsafe fn trap_BotGetTopGoal(goalstate: c_int, goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_TOP_GOAL, goalstate, goal)
}

pub unsafe fn trap_BotGetSecondGoal(goalstate: c_int, goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_SECOND_GOAL, goalstate, goal)
}

pub unsafe fn trap_BotChooseLTGItem(goalstate: c_int, origin: [f32; 3], inventory: *mut c_int, travelflags: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHOOSE_LTG_ITEM, goalstate, &origin as *const _ as *const c_void, inventory as *const c_void, travelflags)
}

pub unsafe fn trap_BotChooseNBGItem(goalstate: c_int, origin: [f32; 3], inventory: *mut c_int, travelflags: c_int, ltg: *mut c_void, maxtime: f32) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHOOSE_NBG_ITEM, goalstate, &origin as *const _ as *const c_void, inventory as *const c_void, travelflags, ltg, PASSFLOAT(maxtime))
}

pub unsafe fn trap_BotTouchingGoal(origin: [f32; 3], goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_TOUCHING_GOAL, &origin as *const _ as *const c_void, goal)
}

pub unsafe fn trap_BotItemGoalInVisButNotVisible(viewer: c_int, eye: [f32; 3], viewangles: [f32; 3], goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE, viewer, &eye as *const _ as *const c_void, &viewangles as *const _ as *const c_void, goal)
}

pub unsafe fn trap_BotGetLevelItemGoal(index: c_int, classname: *mut c_char, goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_LEVEL_ITEM_GOAL, index, classname as *const c_void, goal)
}

pub unsafe fn trap_BotGetNextCampSpotGoal(num: c_int, goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL, num, goal)
}

pub unsafe fn trap_BotGetMapLocationGoal(name: *mut c_char, goal: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_MAP_LOCATION_GOAL, name as *const c_void, goal)
}

pub unsafe fn trap_BotAvoidGoalTime(goalstate: c_int, number: c_int) -> f32 {
    let temp: c_int;
    temp = syscall(crate::game::g_syscalls_h::BOTLIB_AI_AVOID_GOAL_TIME, goalstate, number);
    (*((&temp as *const c_int) as *const f32))
}

pub unsafe fn trap_BotSetAvoidGoalTime(goalstate: c_int, number: c_int, avoidtime: f32) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_SET_AVOID_GOAL_TIME, goalstate, number, PASSFLOAT(avoidtime));
}

pub unsafe fn trap_BotInitLevelItems() {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_INIT_LEVEL_ITEMS);
}

pub unsafe fn trap_BotUpdateEntityItems() {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_UPDATE_ENTITY_ITEMS);
}

pub unsafe fn trap_BotLoadItemWeights(goalstate: c_int, filename: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_LOAD_ITEM_WEIGHTS, goalstate, filename as *const c_void)
}

pub unsafe fn trap_BotFreeItemWeights(goalstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_ITEM_WEIGHTS, goalstate);
}

pub unsafe fn trap_BotInterbreedGoalFuzzyLogic(parent1: c_int, parent2: c_int, child: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC, parent1, parent2, child);
}

pub unsafe fn trap_BotSaveGoalFuzzyLogic(goalstate: c_int, filename: *mut c_char) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC, goalstate, filename as *const c_void);
}

pub unsafe fn trap_BotMutateGoalFuzzyLogic(goalstate: c_int, range: f32) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC, goalstate, PASSFLOAT(range));
}

pub unsafe fn trap_BotAllocGoalState(state: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ALLOC_GOAL_STATE, state)
}

pub unsafe fn trap_BotFreeGoalState(handle: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_GOAL_STATE, handle);
}

pub unsafe fn trap_BotResetMoveState(movestate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_MOVE_STATE, movestate);
}

pub unsafe fn trap_BotAddAvoidSpot(movestate: c_int, origin: [f32; 3], radius: f32, type_: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ADD_AVOID_SPOT, movestate, &origin as *const _ as *const c_void, PASSFLOAT(radius), type_);
}

pub unsafe fn trap_BotMoveToGoal(result: *mut c_void, movestate: c_int, goal: *mut c_void, travelflags: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_MOVE_TO_GOAL, result, movestate, goal, travelflags);
}

pub unsafe fn trap_BotMoveInDirection(movestate: c_int, dir: [f32; 3], speed: f32, type_: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_MOVE_IN_DIRECTION, movestate, &dir as *const _ as *const c_void, PASSFLOAT(speed), type_)
}

pub unsafe fn trap_BotResetAvoidReach(movestate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_AVOID_REACH, movestate);
}

pub unsafe fn trap_BotResetLastAvoidReach(movestate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_LAST_AVOID_REACH, movestate);
}

pub unsafe fn trap_BotReachabilityArea(origin: [f32; 3], testground: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_REACHABILITY_AREA, &origin as *const _ as *const c_void, testground)
}

pub unsafe fn trap_BotMovementViewTarget(movestate: c_int, goal: *mut c_void, travelflags: c_int, lookahead: f32, target: *mut [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_MOVEMENT_VIEW_TARGET, movestate, goal, travelflags, PASSFLOAT(lookahead), target as *const c_void)
}

pub unsafe fn trap_BotPredictVisiblePosition(origin: [f32; 3], areanum: c_int, goal: *mut c_void, travelflags: c_int, target: *mut [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_PREDICT_VISIBLE_POSITION, &origin as *const _ as *const c_void, areanum, goal, travelflags, target as *const c_void)
}

pub unsafe fn trap_BotAllocMoveState() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ALLOC_MOVE_STATE)
}

pub unsafe fn trap_BotFreeMoveState(handle: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_MOVE_STATE, handle);
}

pub unsafe fn trap_BotInitMoveState(handle: c_int, initmove: *mut c_void) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_INIT_MOVE_STATE, handle, initmove);
}

pub unsafe fn trap_BotChooseBestFightWeapon(weaponstate: c_int, inventory: *mut c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON, weaponstate, inventory as *const c_void)
}

pub unsafe fn trap_BotGetWeaponInfo(weaponstate: c_int, weapon: c_int, weaponinfo: *mut c_void) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GET_WEAPON_INFO, weaponstate, weapon, weaponinfo);
}

pub unsafe fn trap_BotLoadWeaponWeights(weaponstate: c_int, filename: *mut c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_LOAD_WEAPON_WEIGHTS, weaponstate, filename as *const c_void)
}

pub unsafe fn trap_BotAllocWeaponState() -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_ALLOC_WEAPON_STATE)
}

pub unsafe fn trap_BotFreeWeaponState(weaponstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_FREE_WEAPON_STATE, weaponstate);
}

pub unsafe fn trap_BotResetWeaponState(weaponstate: c_int) {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_RESET_WEAPON_STATE, weaponstate);
}

pub unsafe fn trap_GeneticParentsAndChildSelection(numranks: c_int, ranks: *mut f32, parent1: *mut c_int, parent2: *mut c_int, child: *mut c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION, numranks, ranks as *const c_void, parent1 as *const c_void, parent2 as *const c_void, child as *const c_void)
}

pub unsafe fn trap_PC_LoadSource(filename: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_PC_LOAD_SOURCE, filename as *const c_void)
}

pub unsafe fn trap_PC_FreeSource(handle: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_PC_FREE_SOURCE, handle)
}

pub unsafe fn trap_PC_ReadToken(handle: c_int, pc_token: *mut crate::q_shared_h::pc_token_t) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_PC_READ_TOKEN, handle, pc_token as *const c_void)
}

pub unsafe fn trap_PC_SourceFileAndLine(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::BOTLIB_PC_SOURCE_FILE_AND_LINE, handle, filename as *const c_void, line as *const c_void)
}


/*
Ghoul2 Insert Start
*/
pub unsafe fn trap_R_RegisterSkin(name: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_R_REGISTERSKIN, name as *const c_void)
}

// CG Specific API calls
pub unsafe fn trap_G2_ListModelBones(ghlInfo: *mut c_void, frame: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_LISTBONES, ghlInfo, frame);
}

pub unsafe fn trap_G2_ListModelSurfaces(ghlInfo: *mut c_void) {
    syscall(crate::game::g_syscalls_h::G_G2_LISTSURFACES, ghlInfo);
}

pub unsafe fn trap_G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_HAVEWEGHOULMODELS, ghoul2)
}

pub unsafe fn trap_G2_SetGhoul2ModelIndexes(ghoul2: *mut c_void, modelList: *mut c_int, skinList: *mut c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_SETMODELS, ghoul2, modelList as *const c_void, skinList as *const c_void);
}

pub unsafe fn trap_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut crate::g_local_h::mdxaBone_t,
                                angles: *const [f32; 3], position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_GETBOLT, ghoul2, modelIndex, boltIndex, matrix as *const c_void, angles as *const c_void, position as *const c_void, frameNum, modelList as *const c_void, scale as *const c_void)
}

pub unsafe fn trap_G2API_GetBoltMatrix_NoReconstruct(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut crate::g_local_h::mdxaBone_t,
                                angles: *const [f32; 3], position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
    //Same as above but force it to not reconstruct the skeleton before getting the bolt position
    syscall(crate::game::g_syscalls_h::G_G2_GETBOLT_NOREC, ghoul2, modelIndex, boltIndex, matrix as *const c_void, angles as *const c_void, position as *const c_void, frameNum, modelList as *const c_void, scale as *const c_void)
}

pub unsafe fn trap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut crate::g_local_h::mdxaBone_t,
                                angles: *const [f32; 3], position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int {
    //Same as above but force it to not reconstruct the skeleton before getting the bolt position
    syscall(crate::game::g_syscalls_h::G_G2_GETBOLT_NOREC_NOROT, ghoul2, modelIndex, boltIndex, matrix as *const c_void, angles as *const c_void, position as *const c_void, frameNum, modelList as *const c_void, scale as *const c_void)
}

pub unsafe fn trap_G2API_InitGhoul2Model(ghoul2Ptr: *mut *mut c_void, fileName: *const c_char, modelIndex: c_int, customSkin: c_int,
                          customShader: c_int, modelFlags: c_int, lodBias: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_INITGHOUL2MODEL, ghoul2Ptr as *const c_void, fileName as *const c_void, modelIndex, customSkin, customShader, modelFlags, lodBias)
}

pub unsafe fn trap_G2API_SetSkin(ghoul2: *mut c_void, modelIndex: c_int, customSkin: c_int, renderSkin: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SETSKIN, ghoul2, modelIndex, customSkin, renderSkin)
}

pub unsafe fn trap_G2API_Ghoul2Size(ghlInfo: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SIZE, ghlInfo)
}

pub unsafe fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_ADDBOLT, ghoul2, modelIndex, boneName as *const c_void)
}

pub unsafe fn trap_G2API_SetBoltInfo(ghoul2: *mut c_void, modelIndex: c_int, boltInfo: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_SETBOLTINFO, ghoul2, modelIndex, boltInfo);
}

pub unsafe fn trap_G2API_SetBoneAngles(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, angles: *const [f32; 3], flags: c_int,
                                up: c_int, right: c_int, forward: c_int, modelList: *mut c_int,
                                blendTime: c_int, currentTime: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_ANGLEOVERRIDE, ghoul2, modelIndex, boneName as *const c_void, angles as *const c_void, flags, up, right, forward, modelList as *const c_void, blendTime, currentTime)
}

pub unsafe fn trap_G2API_SetBoneAnim(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int,
                          flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_PLAYANIM, ghoul2, modelIndex, boneName as *const c_void, startFrame, endFrame, flags, PASSFLOAT(animSpeed), currentTime, PASSFLOAT(setFrame), blendTime)
}

pub unsafe fn trap_G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32,
                           startFrame: *mut c_int, endFrame: *mut c_int, flags: *mut c_int, animSpeed: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_GETBONEANIM, ghoul2, boneName as *const c_void, currentTime, currentFrame as *const c_void, startFrame as *const c_void, endFrame as *const c_void, flags as *const c_void, animSpeed as *const c_void, modelList as *const c_void, modelIndex)
}

pub unsafe fn trap_G2API_GetGLAName(ghoul2: *mut c_void, modelIndex: c_int, fillBuf: *mut c_char) {
    syscall(crate::game::g_syscalls_h::G_G2_GETGLANAME, ghoul2, modelIndex, fillBuf as *const c_void);
}

pub unsafe fn trap_G2API_CopyGhoul2Instance(g2From: *mut c_void, g2To: *mut c_void, modelIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_COPYGHOUL2INSTANCE, g2From, g2To, modelIndex)
}

pub unsafe fn trap_G2API_CopySpecificGhoul2Model(g2From: *mut c_void, modelFrom: c_int, g2To: *mut c_void, modelTo: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_COPYSPECIFICGHOUL2MODEL, g2From, modelFrom, g2To, modelTo);
}

pub unsafe fn trap_G2API_DuplicateGhoul2Instance(g2From: *mut c_void, g2To: *mut *mut c_void) {
    syscall(crate::game::g_syscalls_h::G_G2_DUPLICATEGHOUL2INSTANCE, g2From, g2To as *const c_void);
}

pub unsafe fn trap_G2API_HasGhoul2ModelOnIndex(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_HASGHOUL2MODELONINDEX, ghlInfo, modelIndex)
}

pub unsafe fn trap_G2API_RemoveGhoul2Model(ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_REMOVEGHOUL2MODEL, ghlInfo, modelIndex)
}

pub unsafe fn trap_G2API_RemoveGhoul2Models(ghlInfo: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_REMOVEGHOUL2MODELS, ghlInfo)
}

pub unsafe fn trap_G2API_CleanGhoul2Models(ghoul2Ptr: *mut *mut c_void) {
    syscall(crate::game::g_syscalls_h::G_G2_CLEANMODELS, ghoul2Ptr as *const c_void);
}

pub unsafe fn trap_G2API_CollisionDetect(
    collRecMap: *mut crate::g_local_h::CollisionRecord_t,
    ghoul2: *mut c_void,
    angles: *const [f32; 3],
    position: *const [f32; 3],
    frameNumber: c_int,
    entNum: c_int,
    rayStart: *const [f32; 3],
    rayEnd: *const [f32; 3],
    scale: *const [f32; 3],
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32
) {
    syscall(crate::game::g_syscalls_h::G_G2_COLLISIONDETECT, collRecMap as *const c_void, ghoul2, angles as *const c_void, position as *const c_void, frameNumber, entNum, rayStart as *const c_void, rayEnd as *const c_void, scale as *const c_void, traceFlags, useLod, PASSFLOAT(fRadius));
}

pub unsafe fn trap_G2API_CollisionDetectCache(
    collRecMap: *mut crate::g_local_h::CollisionRecord_t,
    ghoul2: *mut c_void,
    angles: *const [f32; 3],
    position: *const [f32; 3],
    frameNumber: c_int,
    entNum: c_int,
    rayStart: *const [f32; 3],
    rayEnd: *const [f32; 3],
    scale: *const [f32; 3],
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32
) {
    syscall(crate::game::g_syscalls_h::G_G2_COLLISIONDETECTCACHE, collRecMap as *const c_void, ghoul2, angles as *const c_void, position as *const c_void, frameNumber, entNum, rayStart as *const c_void, rayEnd as *const c_void, scale as *const c_void, traceFlags, useLod, PASSFLOAT(fRadius));
}

pub unsafe fn trap_G2API_GetSurfaceName(ghoul2: *mut c_void, surfNumber: c_int, modelIndex: c_int, fillBuf: *mut c_char) {
    syscall(crate::game::g_syscalls_h::G_G2_GETSURFACENAME, ghoul2, surfNumber, modelIndex, fillBuf as *const c_void);
}

pub unsafe fn trap_G2API_SetRootSurface(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SETROOTSURFACE, ghoul2, modelIndex, surfaceName as *const c_void)
}

pub unsafe fn trap_G2API_SetSurfaceOnOff(ghoul2: *mut c_void, surfaceName: *const c_char, flags: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SETSURFACEONOFF, ghoul2, surfaceName as *const c_void, flags)
}

pub unsafe fn trap_G2API_SetNewOrigin(ghoul2: *mut c_void, boltIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SETNEWORIGIN, ghoul2, boltIndex)
}

//check if a bone exists on skeleton without actually adding to the bone list -rww
pub unsafe fn trap_G2API_DoesBoneExist(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_DOESBONEEXIST, ghoul2, modelIndex, boneName as *const c_void)
}

pub unsafe fn trap_G2API_GetSurfaceRenderStatus(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_GETSURFACERENDERSTATUS, ghoul2, modelIndex, surfaceName as *const c_void)
}

//hack for smoothing during ugly situations. forgive me.
pub unsafe fn trap_G2API_AbsurdSmoothing(ghoul2: *mut c_void, status: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_ABSURDSMOOTHING, ghoul2, status);
}

//rww - RAGDOLL_BEGIN
pub unsafe fn trap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut crate::g_local_h::sharedRagDollParams_t) {
    syscall(crate::game::g_syscalls_h::G_G2_SETRAGDOLL, ghoul2, params as *const c_void);
}

pub unsafe fn trap_G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, params: *mut crate::g_local_h::sharedRagDollUpdateParams_t) {
    syscall(crate::game::g_syscalls_h::G_G2_ANIMATEG2MODELS, ghoul2, time, params as *const c_void);
}
//rww - RAGDOLL_END

//additional ragdoll options -rww
pub unsafe fn trap_G2API_RagPCJConstraint(ghoul2: *mut c_void, boneName: *const c_char, min: *const [f32; 3], max: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_RAGPCJCONSTRAINT, ghoul2, boneName as *const c_void, min as *const c_void, max as *const c_void)
}

pub unsafe fn trap_G2API_RagPCJGradientSpeed(ghoul2: *mut c_void, boneName: *const c_char, speed: f32) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_RAGPCJGRADIENTSPEED, ghoul2, boneName as *const c_void, PASSFLOAT(speed))
}

pub unsafe fn trap_G2API_RagEffectorGoal(ghoul2: *mut c_void, boneName: *const c_char, pos: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_RAGEFFECTORGOAL, ghoul2, boneName as *const c_void, pos as *const c_void)
}

pub unsafe fn trap_G2API_GetRagBonePos(ghoul2: *mut c_void, boneName: *const c_char, pos: *mut [f32; 3], entAngles: *const [f32; 3], entPos: *const [f32; 3], entScale: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_GETRAGBONEPOS, ghoul2, boneName as *const c_void, pos as *const c_void, entAngles as *const c_void, entPos as *const c_void, entScale as *const c_void)
}

pub unsafe fn trap_G2API_RagEffectorKick(ghoul2: *mut c_void, boneName: *const c_char, velocity: *const [f32; 3]) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_RAGEFFECTORKICK, ghoul2, boneName as *const c_void, velocity as *const c_void)
}

pub unsafe fn trap_G2API_RagForceSolve(ghoul2: *mut c_void, force: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_RAGFORCESOLVE, ghoul2, force)
}

pub unsafe fn trap_G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *mut crate::g_local_h::sharedSetBoneIKStateParams_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_SETBONEIKSTATE, ghoul2, time, boneName as *const c_void, ikState, params as *const c_void)
}

pub unsafe fn trap_G2API_IKMove(ghoul2: *mut c_void, time: c_int, params: *mut crate::g_local_h::sharedIKMoveParams_t) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_IKMOVE, ghoul2, time, params as *const c_void)
}

pub unsafe fn trap_G2API_RemoveBone(ghoul2: *mut c_void, boneName: *const c_char, modelIndex: c_int) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_REMOVEBONE, ghoul2, boneName as *const c_void, modelIndex)
}

//rww - Stuff to allow association of ghoul2 instances to entity numbers.
//This way, on listen servers when both the client and server are doing
//ghoul2 operations, we can copy relevant data off the client instance
//directly onto the server instance and slash the transforms and whatnot
//right in half.
pub unsafe fn trap_G2API_AttachInstanceToEntNum(ghoul2: *mut c_void, entityNum: c_int, server: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_ATTACHINSTANCETOENTNUM, ghoul2, entityNum, server);
}

pub unsafe fn trap_G2API_ClearAttachedInstance(entityNum: c_int) {
    syscall(crate::game::g_syscalls_h::G_G2_CLEARATTACHEDINSTANCE, entityNum);
}

pub unsafe fn trap_G2API_CleanEntAttachments() {
    syscall(crate::game::g_syscalls_h::G_G2_CLEANENTATTACHMENTS);
}

pub unsafe fn trap_G2API_OverrideServer(serverInstance: *mut c_void) -> c_int {
    syscall(crate::game::g_syscalls_h::G_G2_OVERRIDESERVER, serverInstance)
}

/*
Ghoul2 Insert End
*/

pub unsafe fn trap_SetActiveSubBSP(index: c_int) {
    //rwwRMG - added [NEWTRAP]
    syscall(crate::game::g_syscalls_h::G_SET_ACTIVE_SUBBSP, index);
}

pub unsafe fn trap_CM_RegisterTerrain(config: *const c_char) -> c_int {
    //rwwRMG - added [NEWTRAP]
    syscall(crate::game::g_syscalls_h::G_CM_REGISTER_TERRAIN, config as *const c_void)
}

pub unsafe fn trap_RMG_Init(terrainID: c_int) {
    //rwwRMG - added [NEWTRAP]
    syscall(crate::game::g_syscalls_h::G_RMG_INIT, terrainID);
}

pub unsafe fn trap_Bot_UpdateWaypoints(wpnum: c_int, wps: *mut *mut crate::g_local_h::wpobject_t) {
    syscall(crate::game::g_syscalls_h::G_BOT_UPDATEWAYPOINTS, wpnum, wps as *const c_void);
}

pub unsafe fn trap_Bot_CalculatePaths(rmg: c_int) {
    syscall(crate::game::g_syscalls_h::G_BOT_CALCULATEPATHS, rmg);
}
