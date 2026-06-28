// sv_game.c -- interface to the game dll
//Anything above this #include will be ignored by the compiler

use core::ffi::{c_int, c_char, c_void};

// Stubs for external dependencies - these would be declared in other modules
extern "C" {
    pub static mut sv: server_t;
    pub static mut svs: server_static_t;
    pub static sv_maxclients: *mut cvar_t;
    pub static com_RMG: *mut cvar_t;
    pub static com_dedicated: *mut cvar_t;
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Memset(dest: *mut c_void, c: c_int, count: usize);
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    pub fn Com_RealTime(qtime: *mut qtime_s) -> c_int;
    pub fn Com_Milliseconds() -> c_int;
    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_Register(vm_cvar: *mut vmCvar_t, var_name: *const c_char, value: *const c_char, flags: c_int);
    pub fn Cvar_Update(vm_cvar: *mut vmCvar_t);
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
    pub fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    pub fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn Cvar_InfoString(bit: c_int) -> *const c_char;
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufsize: c_int);
    pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn FS_FOpenFileByMode(qpath: *const c_char, f: *mut c_int, mode: fsMode_t) -> c_int;
    pub fn FS_Read2(buffer: *mut c_void, len: c_int, f: c_int);
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: c_int);
    pub fn FS_FCloseFile(f: c_int);
    pub fn FS_GetFileList(path: *const c_char, extension: *const c_char, list_buf: *mut c_char, bufsize: c_int) -> c_int;
    pub fn SV_LinkEntity(ent: *mut sharedEntity_t);
    pub fn SV_UnlinkEntity(ent: *mut sharedEntity_t);
    pub fn SV_AreaEntities(mins: *const f32, maxs: *const f32, entity_list: *mut c_int, max_entities: c_int) -> c_int;
    pub fn SV_Trace(results: *mut trace_t, start: *const f32, mins: *const f32, maxs: *const f32, end: *const f32,
                    passent: c_int, content_mask: c_int, capsule: c_int, g2trace: c_int, skel_lod: c_int);
    pub fn SV_PointContents(p: *const f32, passent: c_int) -> c_int;
    pub fn SV_SetConfigstring(index: c_int, val: *const c_char);
    pub fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufsize: c_int);
    pub fn SV_SetUserinfo(client_num: c_int, val: *const c_char);
    pub fn SV_GetUserinfo(client_num: c_int, buffer: *mut c_char, bufsize: c_int);
    pub fn SV_SendServerCommand(client: *mut client_t, fmt: *const c_char, ...);
    pub fn SV_DropClient(drop: *mut client_t, reason: *const c_char);
    pub fn SV_ClipHandleForEntity(ent: *const sharedEntity_t) -> clipHandle_t;
    pub fn SV_BotAllocateClient() -> c_int;
    pub fn SV_BotFreeClient(client_num: c_int);
    pub fn SV_ClientThink(client: *mut client_t, cmd: *mut usercmd_s);
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t;
    pub fn CM_ModelBounds(model: clipHandle_t, mins: *mut f32, maxs: *mut f32);
    pub fn CM_PointLeafnum(p: *const f32) -> c_int;
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_ClusterPVS(cluster: c_int) -> *const u8;
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> c_int;
    pub fn CM_AdjustAreaPortalState(area1: c_int, area2: c_int, open: c_int);
    pub fn CM_EntityString() -> *mut c_char;
    pub fn CM_ModelContents(model: clipHandle_t, subbsp_index: c_int) -> c_int;
    pub fn CM_LoadSubBSP(name: *const c_char, clientload: c_int) -> c_int;
    pub fn CM_FindSubBSP(model_index: c_int) -> c_int;
    pub fn CM_SubBSPEntityString(index: c_int) -> *mut c_char;
    pub fn CM_RegisterTerrain(name: *const c_char, server_side: bool) -> *mut c_void;
    pub fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char;
    pub fn Sys_SnapVector(v: *mut f32);
    pub fn Sys_Milliseconds() -> c_int;
    pub fn Sys_CheckCD() -> c_int;
    pub fn SE_GetString(label: *const c_char) -> *const c_char;
    pub fn MatrixMultiply(a: *mut f32, b: *mut f32, c: *mut f32);
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn PerpendicularVector(dst: *mut f32, src: *const f32);
    pub fn Q_acos(c: f32) -> f32;
    pub fn Q_asin(c: f32) -> f32;
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn VM_ArgPtr(ptr: c_int) -> *mut c_void;
    pub fn VM_Shifted_Alloc(ptr: *mut *mut c_void, size: usize);
    pub fn VM_Shifted_Free(ptr: *mut *mut c_void);
    pub fn VM_Call(vm: *mut c_void, command: c_int, ...) -> c_int;
    pub fn VM_Create(module: *const c_char, syscalls: extern "C" fn(*mut c_int) -> c_int, interpret: vmInterpret_t) -> *mut c_void;
    pub fn VM_Free(vm: *mut c_void);
    pub fn VM_Restart(vm: *mut c_void) -> *mut c_void;
    pub fn BotImport_DebugPolygonCreate(color: c_int, num_points: c_int, points: *mut f32) -> c_int;
    pub fn BotImport_DebugPolygonDelete(id: c_int);
    pub fn SV_BotLibSetup() -> c_int;
    pub fn SV_BotLibShutdown() -> c_int;
    pub fn SV_BotGetSnapshotEntity(client_num: c_int, ent_num: c_int) -> c_int;
    pub fn SV_BotGetConsoleMessage(client_num: c_int, message: *mut c_char, size: c_int) -> c_int;
    pub fn SV_BotWaypointReception(wpnum: c_int, wps: *mut *mut wpobject_t);
    pub fn SV_BotCalculatePaths(rmg: c_int);
    pub fn ICARUS_RunScript(ent: *mut sharedEntity_t, name: *const c_char) -> c_int;
    pub fn ICARUS_RegisterScript(name: *const c_char, qtest: c_int) -> c_int;
    pub fn ICARUS_Init();
    pub fn ICARUS_ValidEnt(ent: *mut sharedEntity_t) -> c_int;
    pub fn ICARUS_InitEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_FreeEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_AssociateEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_Shutdown();
    pub fn Q3_TaskIDPending(ent: *mut sharedEntity_t, task_type: taskID_t) -> c_int;
    pub fn Q3_TaskIDSet(ent: *mut sharedEntity_t, task_type: taskID_t, task_id: c_int);
    pub fn Q3_TaskIDComplete(ent: *mut sharedEntity_t, task_type: taskID_t);
    pub fn Q3_SetVar(task_id: c_int, ent_id: c_int, type_name: *const c_char, data: *const c_char);
    pub fn Q3_VariableDeclared(name: *const c_char) -> c_int;
    pub fn Q3_GetFloatVariable(name: *const c_char, value: *mut f32) -> c_int;
    pub fn Q3_GetStringVariable(name: *const c_char, value: *mut *const c_char) -> c_int;
    pub fn Q3_GetVectorVariable(name: *const c_char, value: *mut f32) -> c_int;
    pub fn RE_RegisterServerSkin(name: *const c_char) -> qhandle_t;
    pub fn G2API_ListBones(ghoul2: *mut CGhoul2Info, model_index: c_int);
    pub fn G2API_ListSurfaces(ghoul2: *mut CGhoul2Info);
    pub fn G2API_HaveWeGhoul2Models(ghoul2: CGhoul2Info_v) -> c_int;
    pub fn G2API_SetGhoul2ModelIndexes(ghoul2: CGhoul2Info_v, model_handles: *mut qhandle_t, skin_handles: *mut qhandle_t);
    pub fn G2API_GetBoltMatrix(ghoul2: CGhoul2Info_v, model_index: c_int, bolt_index: c_int, matrix: *mut mdxaBone_t,
                               angles: *const f32, position: *const f32, frame_num: c_int, model_handle: *mut qhandle_t, scale: *mut f32) -> c_int;
    pub fn G2API_SetSkin(ghoul2: *mut CGhoul2Info, model_index: c_int, skin_handle: qhandle_t);
    pub fn G2API_Ghoul2Size(ghoul2: CGhoul2Info_v) -> c_int;
    pub fn G2API_AddBolt(ghoul2: CGhoul2Info_v, model_index: c_int, bone_name: *const c_char) -> c_int;
    pub fn G2API_SetBoltInfo(ghoul2: CGhoul2Info_v, model_index: c_int, bolt_index: c_int);
    pub fn G2API_SetBoneAngles(ghoul2: CGhoul2Info_v, model_index: c_int, bone_name: *const c_char, angles: *mut f32, flags: c_int,
                               up: Eorientations, right: Eorientations, forward: Eorientations, model_handles: *mut qhandle_t, use_skel: c_int, blend_time: c_int) -> c_int;
    pub fn G2API_SetBoneAnim(ghoul2: CGhoul2Info_v, model_index: c_int, bone_name: *const c_char, start_frame: c_int, end_frame: c_int,
                             flags: c_int, anim_speed: f32, current_time: c_int, set_frame: f32, blend_time: c_int) -> c_int;
    pub fn G2API_GetBoneAnim(ghoul2: *mut CGhoul2Info, bone_name: *const c_char, current_time: c_int, frame: *mut f32, start_frame: *mut c_int,
                             end_frame: *mut c_int, flags: *mut c_int, retAnim: *mut f32, last_time: *mut c_int) -> c_int;
    pub fn G2API_GetGLAName(ghoul2: CGhoul2Info_v, model_index: c_int) -> *mut c_char;
    pub fn G2API_CopyGhoul2Instance(ghoul2_from: CGhoul2Info_v, ghoul2_to: CGhoul2Info_v, model_index: c_int) -> c_int;
    pub fn G2API_CopySpecificG2Model(from: CGhoul2Info_v, model_index: c_int, to: CGhoul2Info_v, to_model_index: c_int);
    pub fn G2API_DuplicateGhoul2Instance(ghoul2: CGhoul2Info_v, ghoul2_ptr: *mut *mut CGhoul2Info_v);
    pub fn G2API_HasGhoul2ModelOnIndex(ghoul2_ptr: *mut *mut CGhoul2Info_v, model_index: c_int) -> c_int;
    pub fn G2API_RemoveGhoul2Model(ghoul2_ptr: *mut *mut CGhoul2Info_v, model_index: c_int) -> c_int;
    pub fn G2API_RemoveGhoul2Models(ghoul2_ptr: *mut *mut CGhoul2Info_v) -> c_int;
    pub fn G2API_CleanGhoul2Models(ghoul2_ptr: *mut *mut CGhoul2Info_v);
    pub fn G2API_CollisionDetect(collisions: *mut CollisionRecord_t, ghoul2: CGhoul2Info_v, user_scale: *const f32, angles: *const f32,
                                 position: *const f32, trace_start: *const f32, trace_end: *const f32, scale: *const f32, g2vert_space: *mut c_void,
                                 use_lod: c_int, frame: c_int, radius: f32);
    pub fn G2API_CollisionDetectCache(collisions: *mut CollisionRecord_t, ghoul2: CGhoul2Info_v, user_scale: *const f32, angles: *const f32,
                                      position: *const f32, trace_start: *const f32, trace_end: *const f32, scale: *const f32, g2vert_space: *mut c_void,
                                      use_lod: c_int, frame: c_int, radius: f32);
    pub fn G2API_SetRootSurface(ghoul2: CGhoul2Info_v, model_index: c_int, surface_name: *const c_char) -> c_int;
    pub fn G2API_SetSurfaceOnOff(ghoul2: CGhoul2Info_v, surface_name: *const c_char, flags: c_int) -> c_int;
    pub fn G2API_SetNewOrigin(ghoul2: CGhoul2Info_v, bolt_index: c_int) -> c_int;
    pub fn G2API_DoesBoneExist(ghoul2: *mut CGhoul2Info, bone_name: *const c_char) -> c_int;
    pub fn G2API_GetSurfaceRenderStatus(ghoul2: *mut CGhoul2Info, surface_name: *const c_char) -> c_int;
    pub fn G2API_AbsurdSmoothing(ghoul2: CGhoul2Info_v, smooth: c_int);
    pub fn G2API_SetRagDoll(ghoul2: CGhoul2Info_v, params: *mut c_void);
    pub fn G2API_ResetRagDoll(ghoul2: CGhoul2Info_v);
    pub fn G2API_AnimateG2Models(ghoul2: CGhoul2Info_v, anim_speed: c_int, params: *mut c_void);
    pub fn G2API_RagPCJConstraint(ghoul2: CGhoul2Info_v, bone_name: *const c_char, min: *mut f32, max: *mut f32) -> c_int;
    pub fn G2API_RagPCJGradientSpeed(ghoul2: CGhoul2Info_v, bone_name: *const c_char, speed: f32) -> c_int;
    pub fn G2API_RagEffectorGoal(ghoul2: CGhoul2Info_v, bone_name: *const c_char, goal: *mut f32) -> c_int;
    pub fn G2API_GetRagBonePos(ghoul2: CGhoul2Info_v, bone_name: *const c_char, pos: *mut f32, scale: *mut f32, angles: *mut f32, matrix: *mut f32) -> c_int;
    pub fn G2API_RagEffectorKick(ghoul2: CGhoul2Info_v, bone_name: *const c_char, force: *mut f32) -> c_int;
    pub fn G2API_RagForceSolve(ghoul2: CGhoul2Info_v, do_solve: c_int) -> c_int;
    pub fn G2API_SetBoneIKState(ghoul2: CGhoul2Info_v, bone_index: c_int, bone_name: *const c_char, ik_flags: c_int, params: *mut sharedSetBoneIKStateParams_t) -> c_int;
    pub fn G2API_IKMove(ghoul2: CGhoul2Info_v, bone_index: c_int, params: *mut sharedIKMoveParams_t) -> c_int;
    pub fn G2API_RemoveBone(ghoul2: *mut CGhoul2Info, bone_name: *const c_char) -> c_int;
    pub fn G2API_AttachInstanceToEntNum(ghoul2: CGhoul2Info_v, ent_num: c_int, leave_attached: c_int);
    pub fn G2API_ClearAttachedInstance(ent_num: c_int);
    pub fn G2API_CleanEntAttachments();
    pub fn G2API_OverrideServerWithClientData(ghoul2: *mut CGhoul2Info) -> c_int;
    pub fn G2API_GetSurfaceName(ghoul2: *mut CGhoul2Info, surface_num: c_int) -> *mut c_char;
    pub fn TheRandomMissionManager_new() -> *mut c_void;
    pub fn gSequencers(index: c_int) -> *mut c_void;
    pub fn gTaskManagers(index: c_int) -> *mut c_void;
    pub static mut g_svCullDist: f32;
    pub static mut gG2_GBMNoReconstruct: c_int;
    pub static mut gG2_GBMUseSPMethod: c_int;
    pub static mut G2VertSpaceServer: *mut CMiniHeap;
    pub static mut TheRandomMissionManager: *mut c_void;
}

// Type stubs for external dependencies
#[repr(C)]
pub struct server_t {
    pub gentities: *mut sharedEntity_t,
    pub gentity_size: c_int,
    pub num_entities: c_int,
    pub game_clients: *mut playerState_t,
    pub game_client_size: c_int,
    pub sv_entities: [svEntity_t; 4096],
    pub entity_parse_point: *mut c_char,
    pub m_local_sub_bsp_index: c_int,
    pub m_local_sub_bsp_model_offset: c_int,
    pub m_local_sub_bsp_entity_parse_point: *mut c_char,
    pub m_shared_memory: *mut c_char,
    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *mut c_char,
    pub mSharedMemory: *mut c_char,
    pub state: c_int,
}

#[repr(C)]
pub struct server_static_t {
    pub clients: [client_t; 64],
    pub time: c_int,
}

#[repr(C)]
pub struct client_t {
    pub gentity: *mut sharedEntity_t,
    pub last_usercmd: usercmd_t,
}

#[repr(C)]
pub struct sharedEntity_t {
    pub s: entityState_t,
    pub r: entityShared_t,
    pub task_id: [c_int; 16],
    pub parms: *mut parms_t,
    pub behavior_set: [*mut c_char; 3],
    pub script_targetname: *mut c_char,
    pub delay_script_time: c_int,
    pub full_name: *mut c_char,
    pub targetname: *mut c_char,
    pub classname: *mut c_char,
    pub ghoul2: *mut c_void,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub modelindex: c_int,
}

#[repr(C)]
pub struct entityShared_t {
    pub current_origin: [f32; 3],
    pub current_angles: [f32; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub b_model: c_int,
    pub contents: c_int,
}

#[repr(C)]
pub struct svEntity_t {
    pub areanum: c_int,
    pub areanum2: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct usercmd_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct usercmd_s {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct vmCvar_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

#[repr(C)]
pub struct trace_t {
    pub start_solid: c_int,
}

pub type clipHandle_t = c_int;
pub type qhandle_t = c_int;
pub type fsMode_t = c_int;
pub type taskID_t = c_int;
pub type vmInterpret_t = c_int;

#[repr(C)]
pub struct parms_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct qtime_s {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct botlib_export_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct wpobject_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct CGhoul2Info {
    pub _placeholder: u8,
}

pub type CGhoul2Info_v = *mut c_void;

#[repr(C)]
pub struct mdxaBone_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub enum Eorientations {
    ORIGIN = 0,
}

#[repr(C)]
pub struct CollisionRecord_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct sharedSetBoneIKStateParams_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct sharedIKMoveParams_t {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct CMiniHeap {
    pub _placeholder: u8,
}

#[repr(C)]
pub struct siegePers_t {
    pub data1: c_int,
    pub data2: c_int,
    pub data3: c_int,
}

pub static mut botlib_export: *mut botlib_export_t = core::ptr::null_mut();
pub static mut g_local_modifier: sharedEntity_t = sharedEntity_t {
    s: entityState_t { number: 0, modelindex: 0 },
    r: entityShared_t {
        current_origin: [0.0; 3],
        current_angles: [0.0; 3],
        mins: [0.0; 3],
        maxs: [0.0; 3],
        b_model: 0,
        contents: 0,
    },
    task_id: [0; 16],
    parms: core::ptr::null_mut(),
    behavior_set: [core::ptr::null_mut(); 3],
    script_targetname: core::ptr::null_mut(),
    delay_script_time: 0,
    full_name: core::ptr::null_mut(),
    targetname: core::ptr::null_mut(),
    classname: core::ptr::null_mut(),
    ghoul2: core::ptr::null_mut(),
};
pub static mut sv_siege_pers_data: siegePers_t = siegePers_t { data1: 0, data2: 0, data3: 0 };

extern "C" {
    pub fn SV_GameError(string: *const c_char) {
        Com_Error(3, b"%s\0".as_ptr() as *const c_char, string);
    }

    pub fn SV_GamePrint(string: *const c_char) {
        Com_Printf(b"%s\0".as_ptr() as *const c_char, string);
    }
}

// these functions must be used instead of pointer arithmetic, because
// the game allocates gentities with private information after the server shared part
pub unsafe extern "C" fn SV_NumForGentity(ent: *mut sharedEntity_t) -> c_int {
    let num = ((ent as *mut u8).offset_from(sv.gentities as *mut u8)) / sv.gentitySize as isize;
    num as c_int
}

pub unsafe extern "C" fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t {
    ((sv.gentities as *mut u8).add((sv.gentitySize as isize * num as isize) as usize) as *mut sharedEntity_t)
}

pub unsafe extern "C" fn SV_GameClientNum(num: c_int) -> *mut playerState_t {
    ((sv.gameClients as *mut u8).add((sv.gameClientSize as isize * num as isize) as usize) as *mut playerState_t)
}

pub unsafe extern "C" fn SV_SvEntityForGentity(g_ent: *mut sharedEntity_t) -> *mut svEntity_t {
    if g_ent.is_null() || (*g_ent).s.number < 0 || (*g_ent).s.number >= 4096 {
        Com_Error(3, b"SV_SvEntityForGentity: bad gEnt\0".as_ptr() as *const c_char);
    }
    &mut sv.sv_entities[(*g_ent).s.number as usize] as *mut svEntity_t
}

pub unsafe extern "C" fn SV_GEntityForSvEntity(sv_ent: *mut svEntity_t) -> *mut sharedEntity_t {
    let num = (sv_ent as *mut u8).offset_from(sv.sv_entities.as_mut_ptr() as *mut u8) / core::mem::size_of::<svEntity_t>() as isize;
    SV_GentityNum(num as c_int)
}

/*
===============
SV_GameSendServerCommand

Sends a command string to a client
===============
*/
pub unsafe extern "C" fn SV_GameSendServerCommand(client_num: c_int, text: *const c_char) {
    if client_num == -1 {
        SV_SendServerCommand(core::ptr::null_mut(), b"%s\0".as_ptr() as *const c_char, text);
    } else {
        if client_num < 0 || client_num >= (*sv_maxclients).integer {
            return;
        }
        SV_SendServerCommand(svs.clients.as_mut_ptr().add(client_num as usize), b"%s\0".as_ptr() as *const c_char, text);
    }
}


/*
===============
SV_GameDropClient

Disconnects the client with a message
===============
*/
pub unsafe extern "C" fn SV_GameDropClient(client_num: c_int, reason: *const c_char) {
    if client_num < 0 || client_num >= (*sv_maxclients).integer {
        return;
    }
    SV_DropClient(svs.clients.as_mut_ptr().add(client_num as usize), reason);
}

pub unsafe extern "C" fn CM_ModelContents(model: clipHandle_t, sub_bsp_index: c_int) -> c_int;
pub unsafe extern "C" fn CM_LoadSubBSP(name: *const c_char, clientload: c_int) -> c_int;
pub unsafe extern "C" fn CM_FindSubBSP(model_index: c_int) -> c_int;
pub unsafe extern "C" fn CM_SubBSPEntityString(index: c_int) -> *mut c_char;

/*
=================
SV_SetBrushModel

sets mins and maxs for inline bmodels
=================
*/
pub unsafe extern "C" fn SV_SetBrushModel(ent: *mut sharedEntity_t, name: *const c_char) {
    let mut h: clipHandle_t;
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];

    if name.is_null() {
        Com_Error(3, b"SV_SetBrushModel: NULL\0".as_ptr() as *const c_char);
    }

    if *name as i8 == '*' as i8 {
        (*ent).s.modelindex = libc::atoi(name.add(1));

        if sv.mLocalSubBSPIndex != -1 {
            (*ent).s.modelindex += sv.mLocalSubBSPModelOffset;
        }

        h = CM_InlineModel((*ent).s.modelindex);

        CM_ModelBounds(h, mins.as_mut_ptr(), maxs.as_mut_ptr());

        VectorCopy(mins.as_ptr(), (*ent).r.mins.as_mut_ptr());
        VectorCopy(maxs.as_ptr(), (*ent).r.maxs.as_mut_ptr());
        (*ent).r.b_model = 1;

        if !com_RMG.is_null() && (*com_RMG).integer != 0 {
            (*ent).r.contents = CM_ModelContents(h, sv.mLocalSubBSPIndex);
        } else {
            (*ent).r.contents = CM_ModelContents(h, -1);
        }
    } else if *name as i8 == '#' as i8 {
        (*ent).s.modelindex = CM_LoadSubBSP(
            va(b"maps/%s.bsp\0".as_ptr() as *const c_char, name.add(1)),
            0
        );
        CM_ModelBounds((*ent).s.modelindex, mins.as_mut_ptr(), maxs.as_mut_ptr());

        VectorCopy(mins.as_ptr(), (*ent).r.mins.as_mut_ptr());
        VectorCopy(maxs.as_ptr(), (*ent).r.maxs.as_mut_ptr());
        (*ent).r.b_model = 1;

        //rwwNOTE: We don't ever want to set contents -1, it includes CONTENTS_LIGHTSABER.
        //Lots of stuff will explode if there's a brush with CONTENTS_LIGHTSABER that isn't attached to a client owner.
        //ent->contents = -1;		// we don't know exactly what is in the brushes
        h = CM_InlineModel((*ent).s.modelindex);
        (*ent).r.contents = CM_ModelContents(h, CM_FindSubBSP((*ent).s.modelindex));
    } else {
        Com_Error(3, b"SV_SetBrushModel: %s isn't a brush model\0".as_ptr() as *const c_char, name);
    }
}

pub unsafe extern "C" fn SV_SetActiveSubBSP(index: c_int) -> *const c_char {
    if index >= 0 {
        sv.mLocalSubBSPIndex = CM_FindSubBSP(index);
        sv.mLocalSubBSPModelOffset = index;
        sv.mLocalSubBSPEntityParsePoint = CM_SubBSPEntityString(sv.mLocalSubBSPIndex);
        return sv.mLocalSubBSPEntityParsePoint;
    } else {
        sv.mLocalSubBSPIndex = -1;
    }

    core::ptr::null()
}

/*
=================
SV_inPVS

Also checks portalareas so that doors block sight
=================
*/
pub unsafe extern "C" fn SV_inPVS(p1: *const f32, p2: *const f32) -> c_int {
    let mut leafnum: c_int;
    let mut cluster: c_int;
    let mut area1: c_int;
    let mut area2: c_int;
    let mask: *const u8;

    leafnum = CM_PointLeafnum(p1);
    cluster = CM_LeafCluster(leafnum);
    area1 = CM_LeafArea(leafnum);
    mask = CM_ClusterPVS(cluster);

    leafnum = CM_PointLeafnum(p2);
    cluster = CM_LeafCluster(leafnum);
    area2 = CM_LeafArea(leafnum);
    if !mask.is_null() && (((*mask.add(cluster as usize >> 3)) & (1u8 << (cluster & 7))) == 0) {
        return 0;
    }
    if CM_AreasConnected(area1, area2) == 0 {
        return 0;		// a door blocks sight
    }
    return 1;
}


/*
=================
SV_inPVSIgnorePortals

Does NOT check portalareas
=================
*/
pub unsafe extern "C" fn SV_inPVSIgnorePortals(p1: *const f32, p2: *const f32) -> c_int {
    let mut leafnum: c_int;
    let mut cluster: c_int;
    let mut area1: c_int;
    let mut area2: c_int;
    let mask: *const u8;

    leafnum = CM_PointLeafnum(p1);
    cluster = CM_LeafCluster(leafnum);
    area1 = CM_LeafArea(leafnum);
    mask = CM_ClusterPVS(cluster);

    leafnum = CM_PointLeafnum(p2);
    cluster = CM_LeafCluster(leafnum);
    area2 = CM_LeafArea(leafnum);

    if !mask.is_null() && (((*mask.add(cluster as usize >> 3)) & (1u8 << (cluster & 7))) == 0) {
        return 0;
    }

    return 1;
}


/*
========================
SV_AdjustAreaPortalState
========================
*/
pub unsafe extern "C" fn SV_AdjustAreaPortalState(ent: *mut sharedEntity_t, open: c_int) {
    let sv_ent: *mut svEntity_t;

    sv_ent = SV_SvEntityForGentity(ent);
    if (*sv_ent).areanum2 == -1 {
        return;
    }
    CM_AdjustAreaPortalState((*sv_ent).areanum, (*sv_ent).areanum2, open);
}


/*
==================
SV_GameAreaEntities
==================
*/
pub unsafe extern "C" fn SV_EntityContact(mins: *const f32, maxs: *const f32, g_ent: *const sharedEntity_t, capsule: c_int) -> c_int {
    let origin: *const f32;
    let angles: *const f32;
    let mut ch: clipHandle_t;
    let mut trace: trace_t = core::mem::zeroed();

    // check for exact collision
    origin = (*g_ent).r.current_origin.as_ptr();
    angles = (*g_ent).r.current_angles.as_ptr();

    ch = SV_ClipHandleForEntity(g_ent);
    let vec3_origin: [f32; 3] = [0.0; 3];
    SV_Trace(&mut trace, vec3_origin.as_ptr(), vec3_origin.as_ptr(), mins, maxs,
        ch, -1, origin, angles, capsule, 0, 0);

    return trace.start_solid;
}


/*
===============
SV_GetServerinfo

===============
*/
pub unsafe extern "C" fn SV_GetServerinfo(buffer: *mut c_char, buffer_size: c_int) {
    if buffer_size < 1 {
        Com_Error(3, b"SV_GetServerinfo: bufferSize == %i\0".as_ptr() as *const c_char, buffer_size);
    }
    Q_strncpyz(buffer, Cvar_InfoString(1 << 0), buffer_size);
}

/*
===============
SV_LocateGameData

===============
*/
pub unsafe extern "C" fn SV_LocateGameData(g_ents: *mut sharedEntity_t, num_g_entities: c_int, size_of_g_entity_t: c_int,
                       clients: *mut playerState_t, size_of_game_client: c_int) {
    sv.gentities = g_ents;
    sv.gentity_size = size_of_g_entity_t;
    sv.num_entities = num_g_entities;

    sv.game_clients = clients;
    sv.game_client_size = size_of_game_client;
}

pub unsafe extern "C" fn SV_GetEntityToken(buffer: *mut c_char, buffer_size: c_int) -> c_int {
    let s: *mut c_char;

    if sv.mLocalSubBSPIndex == -1 {
        s = COM_Parse(&mut sv.entityParsePoint as *mut *mut c_char as *mut *const c_char);
        Q_strncpyz(buffer, s, buffer_size);
        if sv.entityParsePoint.is_null() && (*s as i8) == 0 {
            return 0;
        } else {
            return 1;
        }
    } else {
        s = COM_Parse(&mut sv.mLocalSubBSPEntityParsePoint as *mut *mut c_char as *mut *const c_char);
        Q_strncpyz(buffer, s, buffer_size);
        if sv.mLocalSubBSPEntityParsePoint.is_null() && (*s as i8) == 0 {
            return 0;
        } else {
            return 1;
        }
    }
}

/*
===============
SV_GetUsercmd

===============
*/
pub unsafe extern "C" fn SV_GetUsercmd(client_num: c_int, cmd: *mut usercmd_t) {
    if client_num < 0 || client_num >= (*sv_maxclients).integer {
        Com_Error(3, b"SV_GetUsercmd: bad clientNum:%i\0".as_ptr() as *const c_char, client_num);
    }
    *cmd = svs.clients[client_num as usize].last_usercmd;
}

//==============================================

fn float_as_int(f: f32) -> c_int {
    let temp: c_int;
    unsafe {
        *((&temp) as *mut c_int as *mut f32) = f;
    }
    temp
}

/*
====================
SV_GameSystemCalls

The module is making a system call
====================
*/
//rcg010207 - see my comments in VM_DllSyscall(), in qcommon/vm.c ...

const TRAP_MEMSET: c_int = 100;
const TRAP_MEMCPY: c_int = 101;
const TRAP_STRNCPY: c_int = 102;
const TRAP_SIN: c_int = 103;
const TRAP_COS: c_int = 104;
const TRAP_ATAN2: c_int = 105;
const TRAP_SQRT: c_int = 106;
const TRAP_MATRIXMULTIPLY: c_int = 107;
const TRAP_ANGLEVECTORS: c_int = 108;
const TRAP_PERPENDICULARVECTOR: c_int = 109;
const TRAP_FLOOR: c_int = 110;
const TRAP_CEIL: c_int = 111;
const TRAP_TESTPRINTINT: c_int = 112;
const TRAP_TESTPRINTFLOAT: c_int = 113;
const TRAP_ACOS: c_int = 114;
const TRAP_ASIN: c_int = 115;

const G_PRINT: c_int = 1;
const G_ERROR: c_int = 2;
const G_MILLISECONDS: c_int = 3;
const CG_PRECISIONTIMER_START: c_int = 4;
const CG_PRECISIONTIMER_END: c_int = 5;
const G_CVAR_REGISTER: c_int = 6;
const G_CVAR_UPDATE: c_int = 7;
const G_CVAR_SET: c_int = 8;
const G_CVAR_VARIABLE_INTEGER_VALUE: c_int = 9;
const G_CVAR_VARIABLE_STRING_BUFFER: c_int = 10;
const G_ARGC: c_int = 11;
const G_ARGV: c_int = 12;
const G_SEND_CONSOLE_COMMAND: c_int = 13;

const G_FS_FOPEN_FILE: c_int = 14;
const G_FS_READ: c_int = 15;
const G_FS_WRITE: c_int = 16;
const G_FS_FCLOSE_FILE: c_int = 17;
const G_FS_GETFILELIST: c_int = 18;

const G_LOCATE_GAME_DATA: c_int = 19;
const G_DROP_CLIENT: c_int = 20;
const G_SEND_SERVER_COMMAND: c_int = 21;
const G_LINKENTITY: c_int = 22;
const G_UNLINKENTITY: c_int = 23;
const G_ENTITIES_IN_BOX: c_int = 24;
const G_ENTITY_CONTACT: c_int = 25;
const G_ENTITY_CONTACTCAPSULE: c_int = 26;
const G_TRACE: c_int = 27;
const G_G2TRACE: c_int = 28;
const G_TRACECAPSULE: c_int = 29;
const G_POINT_CONTENTS: c_int = 30;
const G_SET_SERVER_CULL: c_int = 31;
const G_SET_BRUSH_MODEL: c_int = 32;
const G_IN_PVS: c_int = 33;
const G_IN_PVS_IGNORE_PORTALS: c_int = 34;

const G_SET_CONFIGSTRING: c_int = 35;
const G_GET_CONFIGSTRING: c_int = 36;
const G_SET_USERINFO: c_int = 37;
const G_GET_USERINFO: c_int = 38;
const G_GET_SERVERINFO: c_int = 39;
const G_ADJUST_AREA_PORTAL_STATE: c_int = 40;
const G_AREAS_CONNECTED: c_int = 41;

const G_BOT_ALLOCATE_CLIENT: c_int = 42;
const G_BOT_FREE_CLIENT: c_int = 43;

const G_GET_USERCMD: c_int = 44;

const G_SIEGEPERSSET: c_int = 45;
const G_SIEGEPERSGET: c_int = 46;

const G_DEBUG_POLYGON_CREATE: c_int = 47;
const G_DEBUG_POLYGON_DELETE: c_int = 48;
const G_REAL_TIME: c_int = 49;
const G_SNAPVECTOR: c_int = 50;

const SP_GETSTRINGTEXTSTRING: c_int = 51;

const G_ROFF_CLEAN: c_int = 52;
const G_ROFF_UPDATE_ENTITIES: c_int = 53;
const G_ROFF_CACHE: c_int = 54;
const G_ROFF_PLAY: c_int = 55;
const G_ROFF_PURGE_ENT: c_int = 56;

const G_TRUEMALLOC: c_int = 57;
const G_TRUEFREE: c_int = 58;

const G_ICARUS_RUNSCRIPT: c_int = 59;
const G_ICARUS_REGISTERSCRIPT: c_int = 60;
const G_ICARUS_INIT: c_int = 61;
const G_ICARUS_VALIDENT: c_int = 62;
const G_ICARUS_ISINITIALIZED: c_int = 63;
const G_ICARUS_MAINTAINTASKMANAGER: c_int = 64;
const G_ICARUS_ISRUNNING: c_int = 65;
const G_ICARUS_TASKIDPENDING: c_int = 66;
const G_ICARUS_INITENT: c_int = 67;
const G_ICARUS_FREEENT: c_int = 68;
const G_ICARUS_ASSOCIATEENT: c_int = 69;
const G_ICARUS_SHUTDOWN: c_int = 70;
const G_ICARUS_TASKIDSET: c_int = 71;
const G_ICARUS_TASKIDCOMPLETE: c_int = 72;
const G_ICARUS_SETVAR: c_int = 73;
const G_ICARUS_VARIABLEDECLARED: c_int = 74;
const G_ICARUS_GETFLOATVARIABLE: c_int = 75;
const G_ICARUS_GETSTRINGVARIABLE: c_int = 76;
const G_ICARUS_GETVECTORVARIABLE: c_int = 77;

const G_NAV_INIT: c_int = 78;
const G_NAV_FREE: c_int = 79;
const G_NAV_LOAD: c_int = 80;
const G_NAV_SAVE: c_int = 81;
const G_NAV_ADDRAWPOINT: c_int = 82;
const G_NAV_CALCULATEPATHS: c_int = 83;
const G_NAV_HARDCONNECT: c_int = 84;
const G_NAV_SHOWNODES: c_int = 85;
const G_NAV_SHOWEDGES: c_int = 86;
const G_NAV_SHOWPATH: c_int = 87;
const G_NAV_GETNEARESTNODE: c_int = 88;
const G_NAV_GETBESTNODE: c_int = 89;
const G_NAV_GETNODEPOSITION: c_int = 90;
const G_NAV_GETNODENUMEDGES: c_int = 91;
const G_NAV_GETNODEEDGE: c_int = 92;
const G_NAV_GETNUMNODES: c_int = 93;
const G_NAV_CONNECTED: c_int = 94;
const G_NAV_GETPATHCOST: c_int = 95;
const G_NAV_GETEDGECOST: c_int = 96;
const G_NAV_GETPROJECTEDNODE: c_int = 97;
const G_NAV_CHECKFAILEDNODES: c_int = 98;
const G_NAV_ADDFAILEDNODE: c_int = 99;
const G_NAV_NODEFAILED: c_int = 100;
const G_NAV_NODESARENEIGHBORS: c_int = 101;
const G_NAV_CLEARFAILEDEDGE: c_int = 102;
const G_NAV_CLEARALLFAILEDEDGES: c_int = 103;
const G_NAV_EDGEFAILED: c_int = 104;
const G_NAV_ADDFAILEDEDGE: c_int = 105;
const G_NAV_CHECKFAILEDEDGE: c_int = 106;
const G_NAV_CHECKALLFAILEDEDGES: c_int = 107;
const G_NAV_ROUTEBLOCKED: c_int = 108;
const G_NAV_GETBESTNODEALTROUTE: c_int = 109;
const G_NAV_GETBESTNODEALT2: c_int = 110;
const G_NAV_GETBESTPATHBETWEENENTS: c_int = 111;
const G_NAV_GETNODERADIUS: c_int = 112;
const G_NAV_CHECKBLOCKEDEDGES: c_int = 113;
const G_NAV_CLEARCHECKEDNODES: c_int = 114;
const G_NAV_CHECKEDNODE: c_int = 115;
const G_NAV_SETCHECKEDNODE: c_int = 116;
const G_NAV_FLAGALLNODES: c_int = 117;
const G_NAV_GETPATHSCALCULATED: c_int = 118;
const G_NAV_SETPATHSCALCULATED: c_int = 119;

const G_SET_SHARED_BUFFER: c_int = 120;

const BOTLIB_SETUP: c_int = 121;
const BOTLIB_SHUTDOWN: c_int = 122;

pub unsafe extern "C" fn SV_BotWaypointReception(wpnum: c_int, wps: *mut *mut wpobject_t);
pub unsafe extern "C" fn SV_BotCalculatePaths(rmg: c_int);

pub unsafe extern "C" fn ConvertedEntity(ent: *mut sharedEntity_t) -> *mut sharedEntity_t {
 //Return an entity with the memory shifted around to allow reading/modifying VM memory
    let mut i: c_int = 0;

    assert!(!ent.is_null());

    g_local_modifier.s = (*ent).s;
    g_local_modifier.r = (*ent).r;
    while i < 16 {
        g_local_modifier.task_id[i as usize] = (*ent).task_id[i as usize];
        i += 1;
    }
    i = 0;
    g_local_modifier.parms = VM_ArgPtr((*ent).parms as c_int) as *mut parms_t;
    while i < 3 {
        g_local_modifier.behavior_set[i as usize] = VM_ArgPtr((*ent).behavior_set[i as usize] as c_int) as *mut c_char;
        i += 1;
    }
    i = 0;
    g_local_modifier.script_targetname = VM_ArgPtr((*ent).script_targetname as c_int) as *mut c_char;
    g_local_modifier.delay_script_time = (*ent).delay_script_time;
    g_local_modifier.full_name = VM_ArgPtr((*ent).full_name as c_int) as *mut c_char;
    g_local_modifier.targetname = VM_ArgPtr((*ent).targetname as c_int) as *mut c_char;
    g_local_modifier.classname = VM_ArgPtr((*ent).classname as c_int) as *mut c_char;

    g_local_modifier.ghoul2 = (*ent).ghoul2;

    &mut g_local_modifier as *mut sharedEntity_t
}

pub unsafe extern "C" fn SV_GameSystemCalls(args: *mut c_int) -> c_int {
    match *args {

    //rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap to match, and
    //all of these traps must be shared and have cases in sv_game, cl_cgame, and cl_ui. They must also
    //all be in the same order, and start at 100.
    100 => { // TRAP_MEMSET
        Com_Memset(VM_ArgPtr(*args.add(1)), *args.add(2), *args.add(3) as usize);
        0
    }
    101 => { // TRAP_MEMCPY
        Com_Memcpy(VM_ArgPtr(*args.add(1)), VM_ArgPtr(*args.add(2)), *args.add(3) as usize);
        0
    }
    102 => { // TRAP_STRNCPY
        libc::strncpy(VM_ArgPtr(*args.add(1)) as *mut c_char, VM_ArgPtr(*args.add(2)) as *const c_char, *args.add(3) as usize) as c_int
    }
    103 => { // TRAP_SIN
        float_as_int(libc::sinf((*(args.add(1) as *const f32)))
    }
    104 => { // TRAP_COS
        float_as_int(libc::cosf((*(args.add(1) as *const f32))))
    }
    105 => { // TRAP_ATAN2
        float_as_int(libc::atan2f((*(args.add(1) as *const f32)), (*(args.add(2) as *const f32))))
    }
    106 => { // TRAP_SQRT
        float_as_int(libc::sqrtf((*(args.add(1) as *const f32))))
    }
    107 => { // TRAP_MATRIXMULTIPLY
        MatrixMultiply(VM_ArgPtr(*args.add(1)) as *mut f32, VM_ArgPtr(*args.add(2)) as *mut f32, VM_ArgPtr(*args.add(3)) as *mut f32);
        0
    }
    108 => { // TRAP_ANGLEVECTORS
        AngleVectors(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *mut f32, VM_ArgPtr(*args.add(3)) as *mut f32, VM_ArgPtr(*args.add(4)) as *mut f32);
        0
    }
    109 => { // TRAP_PERPENDICULARVECTOR
        PerpendicularVector(VM_ArgPtr(*args.add(1)) as *mut f32, VM_ArgPtr(*args.add(2)) as *const f32);
        0
    }
    110 => { // TRAP_FLOOR
        float_as_int(libc::floorf((*(args.add(1) as *const f32))))
    }
    111 => { // TRAP_CEIL
        float_as_int(libc::ceilf((*(args.add(1) as *const f32))))
    }
    112 => 0, // TRAP_TESTPRINTINT
    113 => 0, // TRAP_TESTPRINTFLOAT
    114 => { // TRAP_ACOS
        float_as_int(Q_acos((*(args.add(1) as *const f32))))
    }
    115 => { // TRAP_ASIN
        float_as_int(Q_asin((*(args.add(1) as *const f32))))
    }

    1 => { // G_PRINT
        Com_Printf(b"%s\0".as_ptr() as *const c_char, VM_ArgPtr(*args.add(1)));
        0
    }
    2 => { // G_ERROR
        Com_Error(3, b"%s\0".as_ptr() as *const c_char, VM_ArgPtr(*args.add(1)));
        0
    }
    3 => Sys_Milliseconds(), // G_MILLISECONDS
    //rww - precision timer funcs... -ALWAYS- call end after start with supplied ptr, or you'll get a nasty memory leak.
    //not that you should be using these outside of debug anyway.. because you shouldn't be. So don't.
    4 => { // CG_PRECISIONTIMER_START
        0 // Placeholder for complex C++ code
    }
    5 => { // CG_PRECISIONTIMER_END
        0 // Placeholder for complex C++ code
    }
    6 => { // G_CVAR_REGISTER
        Cvar_Register(VM_ArgPtr(*args.add(1)) as *mut vmCvar_t, VM_ArgPtr(*args.add(2)) as *const c_char, VM_ArgPtr(*args.add(3)) as *const c_char, *args.add(4));
        0
    }
    7 => { // G_CVAR_UPDATE
        Cvar_Update(VM_ArgPtr(*args.add(1)) as *mut vmCvar_t);
        0
    }
    8 => { // G_CVAR_SET
        Cvar_Set(VM_ArgPtr(*args.add(1)) as *const c_char, VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    9 => Cvar_VariableIntegerValue(VM_ArgPtr(*args.add(1)) as *const c_char), // G_CVAR_VARIABLE_INTEGER_VALUE
    10 => { // G_CVAR_VARIABLE_STRING_BUFFER
        Cvar_VariableStringBuffer(VM_ArgPtr(*args.add(1)) as *const c_char, VM_ArgPtr(*args.add(2)) as *mut c_char, *args.add(3));
        0
    }
    11 => Cmd_Argc(), // G_ARGC
    12 => { // G_ARGV
        Cmd_ArgvBuffer(*args.add(1), VM_ArgPtr(*args.add(2)) as *mut c_char, *args.add(3));
        0
    }
    13 => { // G_SEND_CONSOLE_COMMAND
        Cbuf_ExecuteText(*args.add(1), VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }

    14 => FS_FOpenFileByMode(VM_ArgPtr(*args.add(1)) as *const c_char, VM_ArgPtr(*args.add(2)) as *mut c_int, *args.add(3) as fsMode_t), // G_FS_FOPEN_FILE
    15 => { // G_FS_READ
        FS_Read2(VM_ArgPtr(*args.add(1)), *args.add(2), *args.add(3));
        0
    }
    16 => { // G_FS_WRITE
        FS_Write(VM_ArgPtr(*args.add(1)), *args.add(2), *args.add(3));
        0
    }
    17 => { // G_FS_FCLOSE_FILE
        FS_FCloseFile(*args.add(1));
        0
    }
    18 => FS_GetFileList(VM_ArgPtr(*args.add(1)) as *const c_char, VM_ArgPtr(*args.add(2)) as *const c_char, VM_ArgPtr(*args.add(3)) as *mut c_char, *args.add(4)), // G_FS_GETFILELIST

    19 => { // G_LOCATE_GAME_DATA
        SV_LocateGameData(VM_ArgPtr(*args.add(1)) as *mut sharedEntity_t, *args.add(2), *args.add(3), VM_ArgPtr(*args.add(4)) as *mut playerState_t, *args.add(5));
        0
    }
    20 => { // G_DROP_CLIENT
        SV_GameDropClient(*args.add(1), VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    21 => { // G_SEND_SERVER_COMMAND
        SV_GameSendServerCommand(*args.add(1), VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    22 => { // G_LINKENTITY
        SV_LinkEntity(VM_ArgPtr(*args.add(1)) as *mut sharedEntity_t);
        0
    }
    23 => { // G_UNLINKENTITY
        SV_UnlinkEntity(VM_ArgPtr(*args.add(1)) as *mut sharedEntity_t);
        0
    }
    24 => SV_AreaEntities(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *mut c_int, *args.add(4)), // G_ENTITIES_IN_BOX
    25 => SV_EntityContact(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *const sharedEntity_t, 0) as c_int, // G_ENTITY_CONTACT (capsule=false)
    26 => SV_EntityContact(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *const sharedEntity_t, 1) as c_int, // G_ENTITY_CONTACTCAPSULE (capsule=true)
    27 => { // G_TRACE
        SV_Trace(VM_ArgPtr(*args.add(1)) as *mut trace_t, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *const f32, VM_ArgPtr(*args.add(4)) as *const f32, VM_ArgPtr(*args.add(5)) as *const f32, *args.add(6), *args.add(7), 0, 0, *args.add(9));
        0
    }
    28 => { // G_G2TRACE
        SV_Trace(VM_ArgPtr(*args.add(1)) as *mut trace_t, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *const f32, VM_ArgPtr(*args.add(4)) as *const f32, VM_ArgPtr(*args.add(5)) as *const f32, *args.add(6), *args.add(7), 0, *args.add(8), *args.add(9));
        0
    }
    29 => { // G_TRACECAPSULE
        SV_Trace(VM_ArgPtr(*args.add(1)) as *mut trace_t, VM_ArgPtr(*args.add(2)) as *const f32, VM_ArgPtr(*args.add(3)) as *const f32, VM_ArgPtr(*args.add(4)) as *const f32, VM_ArgPtr(*args.add(5)) as *const f32, *args.add(6), *args.add(7), 1, *args.add(8), *args.add(9));
        0
    }
    30 => SV_PointContents(VM_ArgPtr(*args.add(1)) as *const f32, *args.add(2)), // G_POINT_CONTENTS
    31 => { // G_SET_SERVER_CULL
        g_svCullDist = (*(args.add(1) as *const f32));
        0
    }
    32 => { // G_SET_BRUSH_MODEL
        SV_SetBrushModel(VM_ArgPtr(*args.add(1)) as *mut sharedEntity_t, VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    33 => SV_inPVS(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *const f32), // G_IN_PVS
    34 => SV_inPVSIgnorePortals(VM_ArgPtr(*args.add(1)) as *const f32, VM_ArgPtr(*args.add(2)) as *const f32), // G_IN_PVS_IGNORE_PORTALS

    35 => { // G_SET_CONFIGSTRING
        SV_SetConfigstring(*args.add(1), VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    36 => { // G_GET_CONFIGSTRING
        SV_GetConfigstring(*args.add(1), VM_ArgPtr(*args.add(2)) as *mut c_char, *args.add(3));
        0
    }
    37 => { // G_SET_USERINFO
        SV_SetUserinfo(*args.add(1), VM_ArgPtr(*args.add(2)) as *const c_char);
        0
    }
    38 => { // G_GET_USERINFO
        SV_GetUserinfo(*args.add(1), VM_ArgPtr(*args.add(2)) as *mut c_char, *args.add(3));
        0
    }
    39 => { // G_GET_SERVERINFO
        SV_GetServerinfo(VM_ArgPtr(*args.add(1)) as *mut c_char, *args.add(2));
        0
    }
    40 => { // G_ADJUST_AREA_PORTAL_STATE
        SV_AdjustAreaPortalState(VM_ArgPtr(*args.add(1)) as *mut sharedEntity_t, *args.add(2));
        0
    }
    41 => CM_AreasConnected(*args.add(1), *args.add(2)), // G_AREAS_CONNECTED

    42 => SV_BotAllocateClient(), // G_BOT_ALLOCATE_CLIENT
    43 => { // G_BOT_FREE_CLIENT
        SV_BotFreeClient(*args.add(1));
        0
    }

    44 => { // G_GET_USERCMD
        SV_GetUsercmd(*args.add(1), VM_ArgPtr(*args.add(2)) as *mut usercmd_t);
        0
    }

    45 => { // G_SIEGEPERSSET
        sv_siege_pers_data = *((VM_ArgPtr(*args.add(1)) as *const siegePers_t));
        0
    }

    46 => { // G_SIEGEPERSGET
        *((VM_ArgPtr(*args.add(1)) as *mut siegePers_t)) = sv_siege_pers_data;
        0
    }

    47 => BotImport_DebugPolygonCreate(*args.add(1), *args.add(2), VM_ArgPtr(*args.add(3)) as *mut f32), // G_DEBUG_POLYGON_CREATE
    48 => { // G_DEBUG_POLYGON_DELETE
        BotImport_DebugPolygonDelete(*args.add(1));
        0
    }
    49 => Com_RealTime(VM_ArgPtr(*args.add(1)) as *mut qtime_s), // G_REAL_TIME
    50 => { // G_SNAPVECTOR
        Sys_SnapVector(VM_ArgPtr(*args.add(1)) as *mut f32);
        0
    }

    51 => { // SP_GETSTRINGTEXTSTRING
        let text: *const c_char;
        assert!(!VM_ArgPtr(*args.add(1)).is_null());
        assert!(!VM_ArgPtr(*args.add(2)).is_null());
        text = SE_GetString(VM_ArgPtr(*args.add(1)) as *const c_char);

        if *text as i8 != 0 {
            Q_strncpyz(VM_ArgPtr(*args.add(2)) as *mut c_char, text, *args.add(3));
            return 1;
        } else {
            Q_strncpyz(VM_ArgPtr(*args.add(2)) as *mut c_char, b"??\0".as_ptr() as *const c_char, *args.add(3));
            return 0;
        }
    }

    _ => {
        Com_Error(3, b"Bad game system trap: %i\0".as_ptr() as *const c_char, *args);
        -1
    }
    }
}

/*
===============
SV_ShutdownGameProgs

Called every time a map changes
===============
*/
pub unsafe extern "C" fn SV_ShutdownGameProgs() {
    if core::ptr::null_mut::<c_void>() as *mut c_void == core::ptr::null_mut() {
        return;
    }
    // VM_Call( gvm, GAME_SHUTDOWN, qfalse );
    // VM_Free( gvm );
    // gvm = NULL;
}

/*
==================
SV_InitGameVM

Called for both a full init and a restart
==================
*/
unsafe extern "C" fn SV_InitGameVM(restart: c_int) {
    let mut i: c_int;

    // start the entity parsing at the beginning
    // sv.entityParsePoint = CM_EntityString();

    // use the current msec count for a random seed
    // init for this gamestate
    // VM_Call( gvm, GAME_INIT, svs.time, Com_Milliseconds(), restart );

    // clear all gentity pointers that might still be set from
    // a previous level
    for i in 0..(*sv_maxclients).integer {
        svs.clients[i as usize].gentity = core::ptr::null_mut();
    }
}



/*
===================
SV_RestartGameProgs

Called on a map_restart, but not on a normal map change
===================
*/
pub unsafe extern "C" fn SV_RestartGameProgs() {
    // if ( !gvm ) {
    //     return;
    // }
    // VM_Call( gvm, GAME_SHUTDOWN, qtrue );

    // do a restart instead of a free
    // gvm = VM_Restart( gvm );
    // if ( !gvm ) { // bk001212 - as done below
    //     Com_Error( ERR_FATAL, "VM_Restart on game failed" );
    // }

    // SV_InitGameVM( qtrue );
}


/*
===============
SV_InitGameProgs

Called on a normal map change, not on a map_restart
===============
*/
pub unsafe extern "C" fn SV_InitGameProgs() {
    let var: *mut cvar_t;
    //FIXME these are temp while I make bots run in vm
    extern "C" {
        pub static mut bot_enable: c_int;
    }

    var = Cvar_Get(b"bot_enable\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 1 << 4);
    if !var.is_null() {
        bot_enable = (*var).integer;
    } else {
        bot_enable = 0;
    }

    if (Cvar_VariableValue(b"fs_restrict\0".as_ptr() as *const c_char) == 0.0) && ((*com_dedicated).integer == 0) && (Sys_CheckCD() == 0) {
        Com_Error(4, SE_GetString(b"CON_TEXT_NEED_CD\0".as_ptr() as *const c_char));
    }

    // load the dll or bytecode
    // gvm = VM_Create( "jampgame", SV_GameSystemCalls, (vmInterpret_t)(int)Cvar_VariableValue( "vm_game" ) );
    // if ( !gvm ) {
    //     Com_Error( ERR_FATAL, "VM_Create on game failed" );
    // }

    // SV_InitGameVM( qfalse );
}


/*
====================
SV_GameCommand

See if the current console command is claimed by the game
====================
*/
pub unsafe extern "C" fn SV_GameCommand() -> c_int {
    // if ( sv.state != SS_GAME ) {
    //     return qfalse;
    // }

    // return (qboolean)VM_Call( gvm, GAME_CONSOLE_COMMAND );
    0
}

// Stub implementations for missing math functions
fn va(_fmt: *const c_char, ...) -> *const c_char {
    core::ptr::null()
}

extern "C" {
    pub fn gSequencers(index: c_int) -> *mut c_void;
    pub fn gTaskManagers(index: c_int) -> *mut c_void;
}

// Needed for g++ to compile
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// These are large switch cases from the original (botlib, nav, g2 calls) that are stubbed out
// They would require full definitions of many external systems.
// The translation preserves the structure but depends on external C linkage for actual calls.
