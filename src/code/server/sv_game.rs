// sv_game.c -- interface to the game dll

use core::ffi::{c_int, c_char, c_void};

// External type declarations (stubs for types defined in other modules)
#[repr(C)]
pub struct gentity_t {
    // Placeholder - full definition in game module
    pub s: sharedEntity_t,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub bmodel: c_int,
    pub contents: c_int,
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
}

#[repr(C)]
pub struct sharedEntity_t {
    pub number: c_int,
    // Other fields...
}

#[repr(C)]
pub struct svEntity_t {
    pub areanum: c_int,
    pub areanum2: c_int,
    // Other fields...
}

pub type qboolean = c_int;
pub type clipHandle_t = c_int;

#[repr(C)]
pub struct trace_t {
    pub startsolid: qboolean,
    pub fraction: f32,
    pub endpos: [f32; 3],
    // Other fields...
}

#[repr(C)]
pub struct game_import_t {
    pub Printf: *const c_void,
    pub WriteCam: *const c_void,
    pub FlushCamFile: *const c_void,
    pub Error: *const c_void,
    pub Milliseconds: *const c_void,
    pub DropClient: *const c_void,
    pub SendServerCommand: *const c_void,
    pub linkentity: *const c_void,
    pub unlinkentity: *const c_void,
    pub EntitiesInBox: *const c_void,
    pub EntityContact: *const c_void,
    pub trace: *const c_void,
    pub pointcontents: *const c_void,
    pub totalMapContents: *const c_void,
    pub SetBrushModel: *const c_void,
    pub inPVS: *const c_void,
    pub inPVSIgnorePortals: *const c_void,
    pub SetConfigstring: *const c_void,
    pub GetConfigstring: *const c_void,
    pub SetUserinfo: *const c_void,
    pub GetUserinfo: *const c_void,
    pub GetServerinfo: *const c_void,
    pub cvar: *const c_void,
    pub cvar_set: *const c_void,
    pub Cvar_VariableIntegerValue: *const c_void,
    pub Cvar_VariableStringBuffer: *const c_void,
    pub argc: *const c_void,
    pub argv: *const c_void,
    pub SendConsoleCommand: *const c_void,
    pub FS_FOpenFile: *const c_void,
    pub FS_Read: *const c_void,
    pub FS_Write: *const c_void,
    pub FS_FCloseFile: *const c_void,
    pub FS_ReadFile: *const c_void,
    pub FS_FreeFile: *const c_void,
    pub FS_GetFileList: *const c_void,
    pub AppendToSaveGame: *const c_void,
    pub ReadFromSaveGame: *const c_void,
    pub ReadFromSaveGameOptional: *const c_void,
    pub AdjustAreaPortalState: *const c_void,
    pub AreasConnected: *const c_void,
    pub VoiceVolume: *const c_void,
    pub Malloc: *const c_void,
    pub Free: *const c_void,
    pub bIsFromZone: *const c_void,
    pub G2API_AddBolt: *const c_void,
    pub G2API_AttachEnt: *const c_void,
    pub G2API_AttachG2Model: *const c_void,
    pub G2API_CollisionDetect: *const c_void,
    pub G2API_DetachEnt: *const c_void,
    pub G2API_DetachG2Model: *const c_void,
    pub G2API_GetAnimFileName: *const c_void,
    pub G2API_GetBoltMatrix: *const c_void,
    pub G2API_GetBoneAnim: *const c_void,
    pub G2API_GetBoneAnimIndex: *const c_void,
    pub G2API_AddSurface: *const c_void,
    pub G2API_HaveWeGhoul2Models: *const c_void,
    pub G2API_InitGhoul2Model: *const c_void,
    pub G2API_SetBoneAngles: *const c_void,
    pub G2API_SetBoneAnglesMatrix: *const c_void,
    pub G2API_SetBoneAnim: *const c_void,
    pub G2API_SetSkin: *const c_void,
    pub G2API_CopyGhoul2Instance: *const c_void,
    pub G2API_SetBoneAnglesIndex: *const c_void,
    pub G2API_SetBoneAnimIndex: *const c_void,
    pub G2API_IsPaused: *const c_void,
    pub G2API_ListBones: *const c_void,
    pub G2API_ListSurfaces: *const c_void,
    pub G2API_PauseBoneAnim: *const c_void,
    pub G2API_PauseBoneAnimIndex: *const c_void,
    pub G2API_PrecacheGhoul2Model: *const c_void,
    pub G2API_RemoveBolt: *const c_void,
    pub G2API_RemoveBone: *const c_void,
    pub G2API_RemoveGhoul2Model: *const c_void,
    pub G2API_SetLodBias: *const c_void,
    pub G2API_SetRootSurface: *const c_void,
    pub G2API_SetShader: *const c_void,
    pub G2API_SetSurfaceOnOff: *const c_void,
    pub G2API_StopBoneAngles: *const c_void,
    pub G2API_StopBoneAnim: *const c_void,
    pub G2API_SetGhoul2ModelFlags: *const c_void,
    pub G2API_AddBoltSurfNum: *const c_void,
    pub G2API_RemoveSurface: *const c_void,
    pub G2API_GetAnimRange: *const c_void,
    pub G2API_GetAnimRangeIndex: *const c_void,
    pub G2API_GiveMeVectorFromMatrix: *const c_void,
    pub G2API_GetGhoul2ModelFlags: *const c_void,
    pub G2API_CleanGhoul2Models: *const c_void,
    pub TheGhoul2InfoArray: *const c_void,
    pub G2API_GetParentSurface: *const c_void,
    pub G2API_GetSurfaceIndex: *const c_void,
    pub G2API_GetSurfaceName: *const c_void,
    pub G2API_GetGLAName: *const c_void,
    pub G2API_SetNewOrigin: *const c_void,
    pub G2API_GetBoneIndex: *const c_void,
    pub G2API_StopBoneAnglesIndex: *const c_void,
    pub G2API_StopBoneAnimIndex: *const c_void,
    pub G2API_SetBoneAnglesMatrixIndex: *const c_void,
    pub G2API_SetAnimIndex: *const c_void,
    pub G2API_GetAnimIndex: *const c_void,
    pub G2API_SaveGhoul2Models: *const c_void,
    pub G2API_LoadGhoul2Models: *const c_void,
    pub G2API_LoadSaveCodeDestructGhoul2Info: *const c_void,
    pub G2API_GetAnimFileNameIndex: *const c_void,
    pub G2API_GetAnimFileInternalNameIndex: *const c_void,
    pub G2API_GetSurfaceRenderStatus: *const c_void,
    pub G2API_SetRagDoll: *const c_void,
    pub G2API_AnimateG2Models: *const c_void,
    pub G2API_RagPCJConstraint: *const c_void,
    pub G2API_RagPCJGradientSpeed: *const c_void,
    pub G2API_RagEffectorGoal: *const c_void,
    pub G2API_GetRagBonePos: *const c_void,
    pub G2API_RagEffectorKick: *const c_void,
    pub G2API_RagForceSolve: *const c_void,
    pub G2API_SetBoneIKState: *const c_void,
    pub G2API_IKMove: *const c_void,
    pub G2API_AddSkinGore: *const c_void,
    pub G2API_ClearSkinGore: *const c_void,
    pub RMG_Init: *const c_void,
    pub CM_RegisterTerrain: *const c_void,
    pub SetActiveSubBSP: *const c_void,
    pub RE_RegisterSkin: *const c_void,
    pub RE_GetAnimationCFG: *const c_void,
    pub WE_GetWindVector: *const c_void,
    pub WE_GetWindGusting: *const c_void,
    pub WE_IsOutside: *const c_void,
    pub WE_IsOutsideCausingPain: *const c_void,
    pub WE_GetChanceOfSaberFizz: *const c_void,
    pub WE_IsShaking: *const c_void,
    pub WE_AddWeatherZone: *const c_void,
    pub WE_SetTempGlobalFogColor: *const c_void,
}

#[repr(C)]
pub struct game_export_t {
    pub apiversion: c_int,
    // Other fields...
}

// External declarations
extern "C" {
    pub static mut ge: *const game_export_t;
    pub static mut sv: server_t;
    pub static mut svs: serverStatic_t;
    pub static mut com_speeds: *const cvar_t;
    pub static mut timeInPVSCheck: c_int;
    pub static mut cmg: clipMap_t;

    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_WriteCam(text: *const c_char);
    pub fn Com_FlushCamFile();
    pub fn Sys_Milliseconds() -> c_int;
    pub fn Sys_UnloadGame();
    pub fn Sys_GetGameAPI(parms: *const c_void) -> *const game_export_t;
    pub fn Sys_CheckCD() -> c_int;
    pub fn SV_SendServerCommand(client: *const c_void, fmt: *const c_char, ...);
    pub fn SV_DropClient(client: *const c_void, reason: *const c_char);
    pub fn SV_LinkEntity(ent: *const c_void);
    pub fn SV_UnlinkEntity(ent: *const c_void);
    pub fn SV_AreaEntities(mins: *const f32, maxs: *const f32, list: *const c_int, maxcount: c_int) -> c_int;
    pub fn SV_Trace(results: *const c_void, start: *const f32, mins: *const f32, maxs: *const f32, end: *const f32, passent: c_int, contentmask: c_int);
    pub fn SV_PointContents(p: *const f32, passent: c_int) -> c_int;
    pub fn SV_ClipHandleForEntity(ent: *const gentity_t) -> clipHandle_t;
    pub fn CM_TotalMapContents() -> c_int;
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t;
    pub fn CM_ModelBounds(cmg: *const clipMap_t, model: clipHandle_t, mins: *const f32, maxs: *const f32);
    pub fn CM_ModelContents(model: clipHandle_t, subBspIndex: c_int) -> c_int;
    pub fn CM_LoadSubBSP(name: *const c_char, clientload: qboolean) -> c_int;
    pub fn CM_FindSubBSP(modelindex: c_int) -> c_int;
    pub fn CM_SubBSPEntityString(subBspIndex: c_int) -> *const c_char;
    pub fn CM_PointLeafnum(p: *const f32) -> c_int;
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_ClusterPVS(cluster: c_int) -> *const u8;
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> c_int;
    pub fn CM_AdjustAreaPortalState(area1: c_int, area2: c_int, open: qboolean);
    pub fn CM_EntityString() -> *const c_char;
    pub fn CM_TransformedBoxTrace(results: *const c_void, start: *const f32, end: *const f32, mins: *const f32, maxs: *const f32, model: clipHandle_t, passent: c_int, origin: *const f32, angles: *const f32);
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *const cvar_t;
    pub fn Cvar_Set(name: *const c_char, value: *const c_char);
    pub fn Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    pub fn Cvar_VariableStringBuffer(name: *const c_char, buffer: *const c_char, bufsize: c_int);
    pub fn Cvar_InfoString(bit: c_int) -> *const c_char;
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cbuf_AddText(text: *const c_char);
    pub fn FS_FOpenFileByMode(qpath: *const c_char, f: *const c_void, mode: c_int) -> c_int;
    pub fn FS_Read(buffer: *const c_void, len: c_int, f: *const c_void) -> c_int;
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: *const c_void) -> c_int;
    pub fn FS_FCloseFile(f: *const c_void);
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *const *const c_void) -> c_int;
    pub fn FS_FreeFile(buffer: *const c_void);
    pub fn FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *const c_char, bufsize: c_int) -> c_int;
    pub fn SG_Append(data: *const c_void, size: c_int);
    pub fn SG_Read(data: *const c_void, size: c_int);
    pub fn SG_ReadOptional(data: *const c_void, size: c_int);
    pub fn SV_SetConfigstring(index: c_int, val: *const c_char);
    pub fn SV_GetConfigstring(index: c_int, buffer: *const c_char, bufferSize: c_int);
    pub fn SV_SetUserinfo(index: c_int, val: *const c_char);
    pub fn SV_GetUserinfo(index: c_int, buffer: *const c_char, bufferSize: c_int);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn COM_Parse(data_p: *const *const c_char) -> *const c_char;
    pub fn atoi(str: *const c_char) -> c_int;
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn VectorCopy(src: *const f32, dest: *const f32);
    pub fn Z_Malloc(size: c_int, tag: c_int, zeroed: qboolean) -> *const c_void;
    pub fn Z_Free(ptr: *const c_void);
    pub fn Z_IsFromZone(ptr: *const c_void) -> qboolean;
    pub fn Z_TagFree(tag: c_int);
    pub fn SCR_StopCinematic();
    pub fn CL_ShutdownCGame();
    pub fn VM_Create(moduleName: *const c_char) -> c_int;
    pub fn SE_GetString(reference: *const c_char) -> *const c_char;
    pub fn RE_RegisterSkin(name: *const c_char) -> c_int;
    pub fn RE_GetAnimationCFG(name: *const c_char) -> *const c_char;
    pub fn R_GetWindVector(wind: *const f32);
    pub fn R_GetWindGusting() -> f32;
    pub fn R_IsOutside(pos: *const f32) -> qboolean;
    pub fn R_IsOutsideCausingPain(pos: *const f32) -> qboolean;
    pub fn R_GetChanceOfSaberFizz() -> f32;
    pub fn R_IsShaking(pos: *const f32) -> qboolean;
    pub fn R_AddWeatherZone(mins: *const f32, maxs: *const f32);
    pub fn R_SetTempGlobalFogColor(color: *const f32);
    pub fn vsprintf(dest: *const c_char, fmt: *const c_char, ap: *const c_void) -> c_int;
}

#[repr(C)]
pub struct server_t {
    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *const c_char,
    pub entityParsePoint: *const c_char,
    pub svEntities: [svEntity_t; 2048], // MAX_GENTITIES
    pub time: c_int,
    pub state: c_int,
}

pub struct serverStatic_t {
    pub clients: [clientData_t; 64],
}

pub struct clientData_t {
    pub gentity: *const gentity_t,
}

pub struct clipMap_t {
    // Placeholder
}

pub struct cvar_t {
    pub string: *const c_char,
    pub integer: c_int,
}

pub const MAX_GENTITIES: usize = 2048;
pub const ERR_DROP: c_int = 0;
pub const ERR_NEED_CD: c_int = 1;
pub const CONTENTS_OPAQUE: c_int = 1;
pub const CVAR_SERVERINFO: c_int = 1;

pub const vec3_origin: [f32; 3] = [0.0, 0.0, 0.0];

pub static mut SubBSP: [*const c_void; 10] = [core::ptr::null(); 10];

// these functions must be used instead of pointer arithmetic, because
// the game allocates gentities with private information after the server shared part
/*
int	SV_NumForGentity( gentity_t *ent ) {
	int		num;

	num = ( (byte *)ent - (byte *)ge->gentities ) / ge->gentitySize;

	return num;
}
*/
pub fn SV_GentityNum(num: c_int) -> *const gentity_t {
    unsafe {
        assert!(num >= 0);
        let ent: *const gentity_t = ((*ge).gentities as *const u8)
            .add((num as usize) * ((*ge).gentitySize as usize)) as *const gentity_t;

        ent
    }
}

pub fn SV_SvEntityForGentity(gEnt: *const gentity_t) -> *const svEntity_t {
    unsafe {
        if gEnt.is_null() || (*gEnt).s.number < 0 || (*gEnt).s.number >= MAX_GENTITIES as c_int {
            Com_Error(ERR_DROP, b"SV_SvEntityForGentity: bad gEnt\0".as_ptr() as *const c_char);
        }
        &sv.svEntities[(*gEnt).s.number as usize] as *const svEntity_t
    }
}

pub fn SV_GEntityForSvEntity(svEnt: *const svEntity_t) -> *const gentity_t {
    let num: c_int;

    unsafe {
        num = ((svEnt as usize - sv.svEntities.as_ptr() as usize) / core::mem::size_of::<svEntity_t>()) as c_int;
    }
    SV_GentityNum(num)
}

/*
===============
SV_GameSendServerCommand

Sends a command string to a client
===============
*/
pub fn SV_GameSendServerCommand(clientNum: c_int, fmt: *const c_char) {
    let mut msg: [c_char; 8192] = [0; 8192];

    unsafe {
        // vsprintf would be called with variadic args from C
        // In Rust, we'd need unsafe FFI handling for va_list, so we keep the pattern but note the limitation
        // vsprintf(msg.as_mut_ptr(), fmt, argptr);

        if clientNum == -1 {
            SV_SendServerCommand(core::ptr::null(), b"%s\0".as_ptr() as *const c_char);
        } else {
            if clientNum < 0 || clientNum >= 1 {
                return;
            }
            SV_SendServerCommand(
                (&svs.clients as *const clientData_t).add(clientNum as usize) as *const c_void,
                b"%s\0".as_ptr() as *const c_char
            );
        }
    }
}


/*
===============
SV_GameDropClient

Disconnects the client with a message
===============
*/
pub fn SV_GameDropClient(clientNum: c_int, reason: *const c_char) {
    if clientNum < 0 || clientNum >= 1 {
        return;
    }
    unsafe {
        SV_DropClient(
            (&svs.clients as *const clientData_t).add(clientNum as usize) as *const c_void,
            reason
        );
    }
}


/*
=================
SV_SetBrushModel

sets mins and maxs for inline bmodels
=================
*/
pub fn SV_SetBrushModel(ent: *const gentity_t, name: *const c_char) {
    let h: clipHandle_t;
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];

    if name.is_null() {
        unsafe {
            Com_Error(
                ERR_DROP,
                b"SV_SetBrushModel: NULL model for ent number %d\0".as_ptr() as *const c_char,
                (*ent).s.number
            );
        }
    }

    unsafe {
        if *name as u8 as c_char == b'*' as c_char {
            let ent_mut = ent as *mut gentity_t;
            (*ent_mut).s.modelindex = atoi(name.add(1));

            if sv.mLocalSubBSPIndex != -1 {
                (*ent_mut).s.modelindex += sv.mLocalSubBSPModelOffset;
            }

            h = CM_InlineModel((*ent_mut).s.modelindex);

            if sv.mLocalSubBSPIndex != -1 {
                CM_ModelBounds(SubBSP[sv.mLocalSubBSPIndex as usize], h, mins.as_mut_ptr(), maxs.as_mut_ptr());
            } else {
                CM_ModelBounds(&cmg, h, mins.as_mut_ptr(), maxs.as_mut_ptr());
            }

            //CM_ModelBounds( h, mins, maxs );

            VectorCopy(mins.as_ptr(), (*ent_mut).mins.as_mut_ptr());
            VectorCopy(maxs.as_ptr(), (*ent_mut).maxs.as_mut_ptr());
            (*ent_mut).bmodel = 1;

            if false { //com_RMG && com_RMG->integer //fixme: this test really should be do we have bsp instances
                (*ent_mut).contents = CM_ModelContents(h, sv.mLocalSubBSPIndex);
            } else {
                (*ent_mut).contents = CM_ModelContents(h, -1);
            }
        } else if *name as u8 as c_char == b'#' as c_char {
            let ent_mut = ent as *mut gentity_t;
            (*ent_mut).s.modelindex = CM_LoadSubBSP(va(b"maps/%s.bsp\0".as_ptr() as *const c_char, name.add(1)), 0);
            CM_ModelBounds(SubBSP[CM_FindSubBSP((*ent_mut).s.modelindex) as usize], (*ent_mut).s.modelindex, mins.as_mut_ptr(), maxs.as_mut_ptr());

            VectorCopy(mins.as_ptr(), (*ent_mut).mins.as_mut_ptr());
            VectorCopy(maxs.as_ptr(), (*ent_mut).maxs.as_mut_ptr());
            (*ent_mut).bmodel = 1;

            //rwwNOTE: We don't ever want to set contents -1, it includes CONTENTS_LIGHTSABER.
            //Lots of stuff will explode if there's a brush with CONTENTS_LIGHTSABER that isn't attached to a client owner.
            //ent->contents = -1;		// we don't know exactly what is in the brushes
            h = CM_InlineModel((*ent_mut).s.modelindex);
            (*ent_mut).contents = CM_ModelContents(h, CM_FindSubBSP((*ent_mut).s.modelindex));
            //	ent->contents = CONTENTS_SOLID;
        } else {
            Com_Error(
                ERR_DROP,
                b"SV_SetBrushModel: %s isn't a brush model (ent %d)\0".as_ptr() as *const c_char,
                name,
                (*ent).s.number
            );
        }
    }
}

pub fn SV_SetActiveSubBSP(index: c_int) -> *const c_char {
    if index >= 0 {
        unsafe {
            sv.mLocalSubBSPIndex = CM_FindSubBSP(index);
            sv.mLocalSubBSPModelOffset = index;
            sv.mLocalSubBSPEntityParsePoint = CM_SubBSPEntityString(sv.mLocalSubBSPIndex);
            return sv.mLocalSubBSPEntityParsePoint;
        }
    } else {
        unsafe {
            sv.mLocalSubBSPIndex = -1;
        }
    }

    core::ptr::null()
}

/*
=================
SV_inPVS

Also checks portalareas so that doors block sight
=================
*/
pub fn SV_inPVS(p1: *const f32, p2: *const f32) -> qboolean {
    let mut leafnum: c_int;
    let mut cluster: c_int;
    let mut area1: c_int;
    let mut area2: c_int;
    let mask: *const u8;
    let mut start: c_int = 0;

    unsafe {
        if (*com_speeds).integer != 0 {
            start = Sys_Milliseconds();
        }
        leafnum = CM_PointLeafnum(p1);
        cluster = CM_LeafCluster(leafnum);
        area1 = CM_LeafArea(leafnum);
        mask = CM_ClusterPVS(cluster);

        leafnum = CM_PointLeafnum(p2);
        cluster = CM_LeafCluster(leafnum);
        area2 = CM_LeafArea(leafnum);
        if !mask.is_null() && ((*mask.add((cluster >> 3) as usize) & (1 << (cluster & 7))) == 0) {
            if (*com_speeds).integer != 0 {
                timeInPVSCheck += Sys_Milliseconds() - start;
            }
            return 0;
        }

        if CM_AreasConnected(area1, area2) == 0 {
            timeInPVSCheck += Sys_Milliseconds() - start;
            return 0; // a door blocks sight
        }

        if (*com_speeds).integer != 0 {
            timeInPVSCheck += Sys_Milliseconds() - start;
        }
        return 1;
    }
}


/*
=================
SV_inPVSIgnorePortals

Does NOT check portalareas
=================
*/
pub fn SV_inPVSIgnorePortals(p1: *const f32, p2: *const f32) -> qboolean {
    let mut leafnum: c_int;
    let mut cluster: c_int;
    let mut area1: c_int;
    let mut area2: c_int;
    let mask: *const u8;
    let mut start: c_int = 0;

    unsafe {
        if (*com_speeds).integer != 0 {
            start = Sys_Milliseconds();
        }

        leafnum = CM_PointLeafnum(p1);
        cluster = CM_LeafCluster(leafnum);
        area1 = CM_LeafArea(leafnum);
        mask = CM_ClusterPVS(cluster);

        leafnum = CM_PointLeafnum(p2);
        cluster = CM_LeafCluster(leafnum);
        area2 = CM_LeafArea(leafnum);

        if !mask.is_null() && ((*mask.add((cluster >> 3) as usize) & (1 << (cluster & 7))) == 0) {
            if (*com_speeds).integer != 0 {
                timeInPVSCheck += Sys_Milliseconds() - start;
            }
            return 0;
        }

        if (*com_speeds).integer != 0 {
            timeInPVSCheck += Sys_Milliseconds() - start;
        }
        return 1;
    }
}


/*
========================
SV_AdjustAreaPortalState
========================
*/
pub fn SV_AdjustAreaPortalState(ent: *const gentity_t, open: qboolean) {
    unsafe {
        if ((*ent).contents & CONTENTS_OPAQUE) == 0 {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                //		Com_Printf( "INFO: entity number %d not opaque: not affecting area portal!\n", ent->s.number );
            }
            return;
        }

        let svEnt: *const svEntity_t;

        svEnt = SV_SvEntityForGentity(ent);
        if (*svEnt).areanum2 == -1 {
            return;
        }
        CM_AdjustAreaPortalState((*svEnt).areanum, (*svEnt).areanum2, open);
    }
}


/*
==================
SV_EntityContact
==================
*/
pub fn SV_EntityContact(mins: *const f32, maxs: *const f32, gEnt: *const gentity_t) -> qboolean {
    let origin: *const f32;
    let angles: *const f32;
    let ch: clipHandle_t;
    let mut trace: trace_t = unsafe { core::mem::zeroed() };

    // check for exact collision
    unsafe {
        origin = (*gEnt).currentOrigin.as_ptr();
        angles = (*gEnt).currentAngles.as_ptr();

        ch = SV_ClipHandleForEntity(gEnt);
        CM_TransformedBoxTrace(&mut trace as *mut trace_t as *const c_void, vec3_origin.as_ptr(), vec3_origin.as_ptr(), mins, maxs,
            ch, -1, origin, angles);

        return trace.startsolid;
    }
}


/*
===============
SV_GetServerinfo

===============
*/
pub fn SV_GetServerinfo(buffer: *mut c_char, bufferSize: c_int) {
    if bufferSize < 1 {
        unsafe {
            Com_Error(ERR_DROP, b"SV_GetServerinfo: bufferSize == %i\0".as_ptr() as *const c_char, bufferSize);
        }
    }
    unsafe {
        Q_strncpyz(buffer, Cvar_InfoString(CVAR_SERVERINFO), bufferSize);
    }
}

pub fn SV_GetEntityToken(buffer: *mut c_char, bufferSize: c_int) -> qboolean {
    let s: *const c_char;

    unsafe {
        if sv.mLocalSubBSPIndex == -1 {
            s = COM_Parse(&mut sv.entityParsePoint as *mut _ as *mut *const c_char);
            Q_strncpyz(buffer, s, bufferSize);
            if sv.entityParsePoint.is_null() && *s as u8 as c_char == 0 {
                return 0;
            } else {
                return 1;
            }
        } else {
            s = COM_Parse(&mut sv.mLocalSubBSPEntityParsePoint as *mut _ as *mut *const c_char);
            Q_strncpyz(buffer, s, bufferSize);
            if sv.mLocalSubBSPEntityParsePoint.is_null() && *s as u8 as c_char == 0 {
                return 0;
            } else {
                return 1;
            }
        }
    }
}

//==============================================

/*
===============
SV_ShutdownGameProgs

Called when either the entire server is being killed, or
it is changing to a different game directory.
===============
*/
pub fn SV_ShutdownGameProgs(shutdownCin: qboolean) {
    unsafe {
        if ge.is_null() {
            return;
        }
        (*ge).Shutdown();

        #[cfg(target_os = "xbox")]
        {
            if shutdownCin != 0 {
                SCR_StopCinematic();
            }
        }
        #[cfg(not(target_os = "xbox"))]
        {
            SCR_StopCinematic();
        }
        CL_ShutdownCGame(); //we have cgame burried in here.

        Sys_UnloadGame(); //this kills cgame as well.

        ge = core::ptr::null();
        cgvm.entryPoint = 0;
    }
}

// this is a compile-helper function since Z_Malloc can now become a macro with __LINE__ etc
//
pub fn G_ZMalloc_Helper(iSize: c_int, eTag: c_int, bZeroit: qboolean) -> *const c_void {
    unsafe {
        Z_Malloc(iSize, eTag, bZeroit)
    }
}

//rww - RAGDOLL_BEGIN
extern "C" {
    pub fn G2API_SetRagDoll(ghoul2: *const c_void, parms: *const c_void);
    pub fn G2API_AnimateG2Models(ghoul2: *const c_void, AcurrentTime: c_int, params: *const c_void);

    pub fn G2API_RagPCJConstraint(ghoul2: *const c_void, boneName: *const c_char, min: *const f32, max: *const f32) -> qboolean;
    pub fn G2API_RagPCJGradientSpeed(ghoul2: *const c_void, boneName: *const c_char, speed: f32) -> qboolean;
    pub fn G2API_RagEffectorGoal(ghoul2: *const c_void, boneName: *const c_char, pos: *const f32) -> qboolean;
    pub fn G2API_GetRagBonePos(ghoul2: *const c_void, boneName: *const c_char, pos: *const f32, entAngles: *const f32, entPos: *const f32, entScale: *const f32) -> qboolean;
    pub fn G2API_RagEffectorKick(ghoul2: *const c_void, boneName: *const c_char, velocity: *const f32) -> qboolean;
    pub fn G2API_RagForceSolve(ghoul2: *const c_void, force: qboolean) -> qboolean;

    pub fn G2API_SetBoneIKState(ghoul2: *const c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *const c_void) -> qboolean;
    pub fn G2API_IKMove(ghoul2: *const c_void, time: c_int, params: *const c_void) -> qboolean;
}
//rww - RAGDOLL_END

//This is as good a place as any I guess.
#[cfg(not(target_os = "xbox"))]
pub fn RMG_Init(terrainID: c_int) {
    unsafe {
        if core::ptr::null::<c_void>() as usize == core::ptr::addr_of!(TheRandomMissionManager) as usize {
            // TheRandomMissionManager = new CRMManager;
        }
        // TheRandomMissionManager->SetLandScape(cmg.landScape);
        // if (TheRandomMissionManager->LoadMission(qtrue))
        // {
        //     TheRandomMissionManager->SpawnMission(qtrue);
        // }
        //		cmg.landScapes[args[1]]->UpdatePatches();
        //sv.mRMGChecksum = cm.landScapes[terrainID]->get_rand_seed();
    }
}

extern "C" {
    pub fn CM_RegisterTerrain(config: *const c_char, server: c_int) -> *const c_void;
}

#[cfg(not(target_os = "xbox"))]
pub fn InterfaceCM_RegisterTerrain(info: *const c_char) -> c_int {
    unsafe {
        // return CM_RegisterTerrain(info, false)->GetTerrainId();
        0
    }
}

extern "C" {
    pub static mut TheRandomMissionManager: *const c_void;
}

#[repr(C)]
pub struct cgvm_t {
    pub entryPoint: c_int,
}

pub static mut cgvm: cgvm_t = cgvm_t {
    entryPoint: 0,
};

/*
===============
SV_InitGameProgs

Init the game subsystem for a new map
===============
*/
pub fn SV_InitGameProgs() {
    let mut import: game_import_t = unsafe { core::mem::zeroed() };
    let mut i: c_int;

    unsafe {
        // unload anything we have now
        if !ge.is_null() {
            SV_ShutdownGameProgs(1);
        }

        #[cfg(not(target_os = "xbox"))]
        {
            if Cvar_VariableIntegerValue(b"fs_restrict\0".as_ptr() as *const c_char) == 0 && Sys_CheckCD() == 0 {
                Com_Error(ERR_NEED_CD, SE_GetString(b"CON_TEXT_NEED_CD\0".as_ptr() as *const c_char)); //"Game CD not in drive" );
            }
        }

        // load a new game dll
        import.Printf = Com_Printf as *const c_void;
        import.WriteCam = Com_WriteCam as *const c_void;
        import.FlushCamFile = Com_FlushCamFile as *const c_void;
        import.Error = Com_Error as *const c_void;

        import.Milliseconds = Sys_Milliseconds as *const c_void;

        import.DropClient = SV_GameDropClient as *const c_void;

        import.SendServerCommand = SV_GameSendServerCommand as *const c_void;


        import.linkentity = SV_LinkEntity as *const c_void;
        import.unlinkentity = SV_UnlinkEntity as *const c_void;
        import.EntitiesInBox = SV_AreaEntities as *const c_void;
        import.EntityContact = SV_EntityContact as *const c_void;
        import.trace = SV_Trace as *const c_void;
        import.pointcontents = SV_PointContents as *const c_void;
        import.totalMapContents = CM_TotalMapContents as *const c_void;
        import.SetBrushModel = SV_SetBrushModel as *const c_void;

        import.inPVS = SV_inPVS as *const c_void;
        import.inPVSIgnorePortals = SV_inPVSIgnorePortals as *const c_void;

        import.SetConfigstring = SV_SetConfigstring as *const c_void;
        import.GetConfigstring = SV_GetConfigstring as *const c_void;

        import.SetUserinfo = SV_SetUserinfo as *const c_void;
        import.GetUserinfo = SV_GetUserinfo as *const c_void;

        import.GetServerinfo = SV_GetServerinfo as *const c_void;

        import.cvar = Cvar_Get as *const c_void;
        import.cvar_set = Cvar_Set as *const c_void;
        import.Cvar_VariableIntegerValue = Cvar_VariableIntegerValue as *const c_void;
        import.Cvar_VariableStringBuffer = Cvar_VariableStringBuffer as *const c_void;

        import.argc = Cmd_Argc as *const c_void;
        import.argv = Cmd_Argv as *const c_void;
        import.SendConsoleCommand = Cbuf_AddText as *const c_void;

        import.FS_FOpenFile = FS_FOpenFileByMode as *const c_void;
        import.FS_Read = FS_Read as *const c_void;
        import.FS_Write = FS_Write as *const c_void;
        import.FS_FCloseFile = FS_FCloseFile as *const c_void;
        import.FS_ReadFile = FS_ReadFile as *const c_void;
        import.FS_FreeFile = FS_FreeFile as *const c_void;
        import.FS_GetFileList = FS_GetFileList as *const c_void;

        import.AppendToSaveGame = SG_Append as *const c_void;
        #[cfg(not(target_os = "xbox"))]
        {
            import.ReadFromSaveGame = SG_Read as *const c_void;
            import.ReadFromSaveGameOptional = SG_ReadOptional as *const c_void;
        }

        import.AdjustAreaPortalState = SV_AdjustAreaPortalState as *const c_void;
        import.AreasConnected = CM_AreasConnected as *const c_void;

        import.VoiceVolume = core::ptr::addr_of!(s_entityWavVol) as *const c_void;

        import.Malloc = G_ZMalloc_Helper as *const c_void;
        import.Free = Z_Free as *const c_void;
        import.bIsFromZone = Z_IsFromZone as *const c_void;
        /*
        Ghoul2 Insert Start
        */

        import.G2API_AddBolt = G2API_AddBolt as *const c_void;
        import.G2API_AttachEnt = G2API_AttachEnt as *const c_void;
        import.G2API_AttachG2Model = G2API_AttachG2Model as *const c_void;
        import.G2API_CollisionDetect = G2API_CollisionDetect as *const c_void;
        import.G2API_DetachEnt = G2API_DetachEnt as *const c_void;
        import.G2API_DetachG2Model = G2API_DetachG2Model as *const c_void;
        import.G2API_GetAnimFileName = G2API_GetAnimFileName as *const c_void;
        import.G2API_GetBoltMatrix = G2API_GetBoltMatrix as *const c_void;
        import.G2API_GetBoneAnim = G2API_GetBoneAnim as *const c_void;
        import.G2API_GetBoneAnimIndex = G2API_GetBoneAnimIndex as *const c_void;
        import.G2API_AddSurface = G2API_AddSurface as *const c_void;
        import.G2API_HaveWeGhoul2Models = G2API_HaveWeGhoul2Models as *const c_void;
        #[cfg(not(target_os = "xbox"))]
        {
            import.G2API_InitGhoul2Model = G2API_InitGhoul2Model as *const c_void;
            import.G2API_SetBoneAngles = G2API_SetBoneAngles as *const c_void;
            import.G2API_SetBoneAnglesMatrix = G2API_SetBoneAnglesMatrix as *const c_void;
            import.G2API_SetBoneAnim = G2API_SetBoneAnim as *const c_void;
            import.G2API_SetSkin = G2API_SetSkin as *const c_void;
            import.G2API_CopyGhoul2Instance = G2API_CopyGhoul2Instance as *const c_void;
            import.G2API_SetBoneAnglesIndex = G2API_SetBoneAnglesIndex as *const c_void;
            import.G2API_SetBoneAnimIndex = G2API_SetBoneAnimIndex as *const c_void;
        }
        import.G2API_IsPaused = G2API_IsPaused as *const c_void;
        import.G2API_ListBones = G2API_ListBones as *const c_void;
        import.G2API_ListSurfaces = G2API_ListSurfaces as *const c_void;
        import.G2API_PauseBoneAnim = G2API_PauseBoneAnim as *const c_void;
        import.G2API_PauseBoneAnimIndex = G2API_PauseBoneAnimIndex as *const c_void;
        import.G2API_PrecacheGhoul2Model = G2API_PrecacheGhoul2Model as *const c_void;
        import.G2API_RemoveBolt = G2API_RemoveBolt as *const c_void;
        import.G2API_RemoveBone = G2API_RemoveBone as *const c_void;
        import.G2API_RemoveGhoul2Model = G2API_RemoveGhoul2Model as *const c_void;
        import.G2API_SetLodBias = G2API_SetLodBias as *const c_void;
        import.G2API_SetRootSurface = G2API_SetRootSurface as *const c_void;
        import.G2API_SetShader = G2API_SetShader as *const c_void;
        import.G2API_SetSurfaceOnOff = G2API_SetSurfaceOnOff as *const c_void;
        import.G2API_StopBoneAngles = G2API_StopBoneAngles as *const c_void;
        import.G2API_StopBoneAnim = G2API_StopBoneAnim as *const c_void;
        import.G2API_SetGhoul2ModelFlags = G2API_SetGhoul2ModelFlags as *const c_void;
        import.G2API_AddBoltSurfNum = G2API_AddBoltSurfNum as *const c_void;
        import.G2API_RemoveSurface = G2API_RemoveSurface as *const c_void;
        import.G2API_GetAnimRange = G2API_GetAnimRange as *const c_void;
        import.G2API_GetAnimRangeIndex = G2API_GetAnimRangeIndex as *const c_void;
        import.G2API_GiveMeVectorFromMatrix = G2API_GiveMeVectorFromMatrix as *const c_void;
        import.G2API_GetGhoul2ModelFlags = G2API_GetGhoul2ModelFlags as *const c_void;
        import.G2API_CleanGhoul2Models = G2API_CleanGhoul2Models as *const c_void;
        import.TheGhoul2InfoArray = TheGhoul2InfoArray as *const c_void;
        import.G2API_GetParentSurface = G2API_GetParentSurface as *const c_void;
        import.G2API_GetSurfaceIndex = G2API_GetSurfaceIndex as *const c_void;
        import.G2API_GetSurfaceName = G2API_GetSurfaceName as *const c_void;
        import.G2API_GetGLAName = G2API_GetGLAName as *const c_void;
        import.G2API_SetNewOrigin = G2API_SetNewOrigin as *const c_void;
        import.G2API_GetBoneIndex = G2API_GetBoneIndex as *const c_void;
        import.G2API_StopBoneAnglesIndex = G2API_StopBoneAnglesIndex as *const c_void;
        import.G2API_StopBoneAnimIndex = G2API_StopBoneAnimIndex as *const c_void;
        import.G2API_SetBoneAnglesMatrixIndex = G2API_SetBoneAnglesMatrixIndex as *const c_void;
        import.G2API_SetAnimIndex = G2API_SetAnimIndex as *const c_void;
        import.G2API_GetAnimIndex = G2API_GetAnimIndex as *const c_void;

        import.G2API_SaveGhoul2Models = G2API_SaveGhoul2Models as *const c_void;
        import.G2API_LoadGhoul2Models = G2API_LoadGhoul2Models as *const c_void;
        import.G2API_LoadSaveCodeDestructGhoul2Info = G2API_LoadSaveCodeDestructGhoul2Info as *const c_void;
        import.G2API_GetAnimFileNameIndex = G2API_GetAnimFileNameIndex as *const c_void;
        import.G2API_GetAnimFileInternalNameIndex = G2API_GetAnimFileInternalNameIndex as *const c_void;
        import.G2API_GetSurfaceRenderStatus = G2API_GetSurfaceRenderStatus as *const c_void;

        //rww - RAGDOLL_BEGIN
        import.G2API_SetRagDoll = G2API_SetRagDoll as *const c_void;
        import.G2API_AnimateG2Models = G2API_AnimateG2Models as *const c_void;

        import.G2API_RagPCJConstraint = G2API_RagPCJConstraint as *const c_void;
        import.G2API_RagPCJGradientSpeed = G2API_RagPCJGradientSpeed as *const c_void;
        import.G2API_RagEffectorGoal = G2API_RagEffectorGoal as *const c_void;
        import.G2API_GetRagBonePos = G2API_GetRagBonePos as *const c_void;
        import.G2API_RagEffectorKick = G2API_RagEffectorKick as *const c_void;
        import.G2API_RagForceSolve = G2API_RagForceSolve as *const c_void;

        import.G2API_SetBoneIKState = G2API_SetBoneIKState as *const c_void;
        import.G2API_IKMove = G2API_IKMove as *const c_void;
        //rww - RAGDOLL_END

        import.G2API_AddSkinGore = G2API_AddSkinGore as *const c_void;
        import.G2API_ClearSkinGore = G2API_ClearSkinGore as *const c_void;

        #[cfg(not(target_os = "xbox"))]
        {
            import.RMG_Init = RMG_Init as *const c_void;
            import.CM_RegisterTerrain = InterfaceCM_RegisterTerrain as *const c_void;
        }
        import.SetActiveSubBSP = SV_SetActiveSubBSP as *const c_void;

        import.RE_RegisterSkin = RE_RegisterSkin as *const c_void;
        import.RE_GetAnimationCFG = RE_GetAnimationCFG as *const c_void;


        import.WE_GetWindVector = R_GetWindVector as *const c_void;
        import.WE_GetWindGusting = R_GetWindGusting as *const c_void;
        import.WE_IsOutside = R_IsOutside as *const c_void;
        import.WE_IsOutsideCausingPain = R_IsOutsideCausingPain as *const c_void;
        import.WE_GetChanceOfSaberFizz = R_GetChanceOfSaberFizz as *const c_void;
        import.WE_IsShaking = R_IsShaking as *const c_void;
        import.WE_AddWeatherZone = R_AddWeatherZone as *const c_void;
        import.WE_SetTempGlobalFogColor = R_SetTempGlobalFogColor as *const c_void;


        /*
        Ghoul2 Insert End
        */

        ge = Sys_GetGameAPI(&import as *const game_import_t as *const c_void);

        if ge.is_null() {
            Com_Error(ERR_DROP, b"failed to load game DLL\0".as_ptr() as *const c_char);
        }

        //hook up the client while we're here
        #[cfg(target_os = "xbox")]
        {
            VM_Create(b"cl\0".as_ptr() as *const c_char);
        }
        #[cfg(not(target_os = "xbox"))]
        {
            if VM_Create(b"cl\0".as_ptr() as *const c_char) == 0 {
                Com_Error(ERR_DROP, b"failed to attach to the client DLL\0".as_ptr() as *const c_char);
            }
        }

        if (*ge).apiversion != GAME_API_VERSION {
            Com_Error(ERR_DROP, b"game is version %i, not %i\0".as_ptr() as *const c_char, (*ge).apiversion,
                GAME_API_VERSION);
        }

        sv.entityParsePoint = CM_EntityString();

        // use the current msec count for a random seed
        Z_TagFree(TAG_G_ALLOC);
        (*ge).Init(
            sv_mapname.string,
            sv_spawntarget.string,
            sv_mapChecksum.integer,
            CM_EntityString(),
            sv.time,
            com_frameTime,
            Com_Milliseconds(),
            eSavedGameJustLoaded,
            qbLoadTransition
        );

        // clear all gentity pointers that might still be set from
        // a previous level
        for i in 0..1 {
            svs.clients[i].gentity = core::ptr::null();
        }
    }
}

extern "C" {
    pub fn Com_Milliseconds() -> c_int;
    pub static sv_mapname: cvar_t;
    pub static sv_spawntarget: cvar_t;
    pub static sv_mapChecksum: cvar_t;
    pub static mut com_frameTime: c_int;
    pub static mut eSavedGameJustLoaded: c_int;
    pub static mut qbLoadTransition: qboolean;
}

pub const GAME_API_VERSION: c_int = 8;
pub const TAG_G_ALLOC: c_int = 1;



/*
====================
SV_GameCommand

See if the current console command is claimed by the game
====================
*/
pub fn SV_GameCommand() -> qboolean {
    unsafe {
        if sv.state != SS_GAME {
            return 0;
        }

        return (*ge).ConsoleCommand();
    }
}

pub const SS_GAME: c_int = 2;

extern "C" {
    pub fn G2API_AddBolt(ghoul2: *const c_void, boneName: *const c_char) -> c_int;
    pub fn G2API_AttachEnt(ghoul2: *const c_void, modelnum: *const c_void, boltnum: c_int, entnum: c_int, rotationAngle: c_int);
    pub fn G2API_AttachG2Model(ghoul2: *const c_void, toghoul2: *const c_void, toboltnum: c_int, fromModel: c_int, fromBoltnum: c_int, entnum: c_int) -> c_int;
    pub fn G2API_CollisionDetect(ghoul2: *const c_void);
    pub fn G2API_DetachEnt(ghoul2: *const c_void, boltnum: c_int);
    pub fn G2API_DetachG2Model(ghoul2: *const c_void, modelnum: c_int);
    pub fn G2API_GetAnimFileName(ghoul2: *const c_void, modelnum: c_int) -> *const c_char;
    pub fn G2API_GetBoltMatrix(ghoul2: *const c_void, modelnum: c_int, boltnum: c_int, matrix: *const c_void);
    pub fn G2API_GetBoneAnim(ghoul2: *const c_void, boneName: *const c_char, currentTime: c_int, animnum: *const c_int, startframe: *const c_int, endframe: *const c_int, flags: *const c_int, retAnim: *const c_int, modelnum: c_int) -> qboolean;
    pub fn G2API_GetBoneAnimIndex(ghoul2: *const c_void, boneindex: c_int, currentTime: c_int, animnum: *const c_int, startframe: *const c_int, endframe: *const c_int, flags: *const c_int, retAnim: *const c_int, modelnum: c_int) -> qboolean;
    pub fn G2API_AddSurface(ghoul2: *const c_void, surfName: *const c_char, polynum: c_int, modelnum: c_int) -> c_int;
    pub fn G2API_HaveWeGhoul2Models(ghoul2: *const c_void) -> qboolean;
    pub fn G2API_InitGhoul2Model(ghoul2: *const c_void, fileName: *const c_char, modelnum: c_int);
    pub fn G2API_SetBoneAngles(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char, angles: *const f32, flags: c_int, up: c_int, right: c_int, forward: c_int);
    pub fn G2API_SetBoneAnglesMatrix(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char, matrix: *const c_void, flags: c_int);
    pub fn G2API_SetBoneAnim(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int, flags: c_int, animSpeed: f32, currentTime: c_int) -> qboolean;
    pub fn G2API_SetSkin(ghoul2: *const c_void, modelnum: c_int, customSkin: c_int);
    pub fn G2API_CopyGhoul2Instance(ghoul2: *const c_void, to: *const c_void, modelnum: c_int);
    pub fn G2API_SetBoneAnglesIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int, angles: *const f32, flags: c_int, up: c_int, right: c_int, forward: c_int);
    pub fn G2API_SetBoneAnimIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int, startFrame: c_int, endFrame: c_int, flags: c_int, animSpeed: f32, currentTime: c_int) -> qboolean;
    pub fn G2API_IsPaused(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int) -> qboolean;
    pub fn G2API_ListBones(ghoul2: *const c_void, modelnum: c_int, boneList: *const c_char, bufferSize: c_int);
    pub fn G2API_ListSurfaces(ghoul2: *const c_void, modelnum: c_int, surfaceList: *const c_char, bufferSize: c_int);
    pub fn G2API_PauseBoneAnim(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char, pauseTime: c_int) -> qboolean;
    pub fn G2API_PauseBoneAnimIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int, pauseTime: c_int) -> qboolean;
    pub fn G2API_PrecacheGhoul2Model(fileName: *const c_char) -> c_int;
    pub fn G2API_RemoveBolt(ghoul2: *const c_void, modelnum: c_int, boltnum: c_int);
    pub fn G2API_RemoveBone(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char);
    pub fn G2API_RemoveGhoul2Model(ghoul2: *const c_void, modelnum: c_int);
    pub fn G2API_SetLodBias(ghoul2: *const c_void, modelnum: c_int, lodbias: c_int);
    pub fn G2API_SetRootSurface(ghoul2: *const c_void, modelnum: c_int, surfacenum: c_int);
    pub fn G2API_SetShader(ghoul2: *const c_void, modelnum: c_int, customShader: c_int);
    pub fn G2API_SetSurfaceOnOff(ghoul2: *const c_void, modelnum: c_int, surfaceName: *const c_char, flags: c_int);
    pub fn G2API_StopBoneAngles(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char);
    pub fn G2API_StopBoneAnim(ghoul2: *const c_void, modelnum: c_int, boneName: *const c_char);
    pub fn G2API_SetGhoul2ModelFlags(ghoul2: *const c_void, modelnum: c_int, flags: c_int);
    pub fn G2API_AddBoltSurfNum(ghoul2: *const c_void, modelnum: c_int, surfnum: c_int, boltnum: *const c_int) -> c_int;
    pub fn G2API_RemoveSurface(ghoul2: *const c_void, modelnum: c_int, surfacenum: c_int);
    pub fn G2API_GetAnimRange(ghoul2: *const c_void, boneName: *const c_char, startFrame: *const c_int, endFrame: *const c_int, modelnum: c_int) -> qboolean;
    pub fn G2API_GetAnimRangeIndex(ghoul2: *const c_void, boneindex: c_int, startFrame: *const c_int, endFrame: *const c_int, modelnum: c_int) -> qboolean;
    pub fn G2API_GiveMeVectorFromMatrix(matrix: *const c_void, flags: c_int) -> *const f32;
    pub fn G2API_GetGhoul2ModelFlags(ghoul2: *const c_void, modelnum: c_int) -> c_int;
    pub fn G2API_CleanGhoul2Models(ghoul2: *const c_void);
    pub fn TheGhoul2InfoArray() -> *const c_void;
    pub fn G2API_GetParentSurface(ghoul2: *const c_void, modelnum: c_int, surfacenum: c_int) -> c_int;
    pub fn G2API_GetSurfaceIndex(ghoul2: *const c_void, modelnum: c_int, surfaceName: *const c_char) -> c_int;
    pub fn G2API_GetSurfaceName(ghoul2: *const c_void, modelnum: c_int, surfacenum: c_int) -> *const c_char;
    pub fn G2API_GetGLAName(ghoul2: *const c_void, modelnum: c_int) -> *const c_char;
    pub fn G2API_SetNewOrigin(ghoul2: *const c_void, boltnum: c_int);
    pub fn G2API_GetBoneIndex(ghoul2: *const c_void, boneName: *const c_char, modelnum: c_int) -> c_int;
    pub fn G2API_StopBoneAnglesIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int);
    pub fn G2API_StopBoneAnimIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int);
    pub fn G2API_SetBoneAnglesMatrixIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int, matrix: *const c_void, flags: c_int);
    pub fn G2API_SetAnimIndex(ghoul2: *const c_void, modelnum: c_int, boneindex: c_int, startFrame: c_int, endFrame: c_int, flags: c_int, animSpeed: f32, currentTime: c_int) -> qboolean;
    pub fn G2API_GetAnimIndex(ghoul2: *const c_void, boneName: *const c_char, modelnum: c_int, startFrame: *const c_int, endFrame: *const c_int) -> qboolean;
    pub fn G2API_SaveGhoul2Models(ghoul2: *const c_void);
    pub fn G2API_LoadGhoul2Models(ghoul2: *const c_void);
    pub fn G2API_LoadSaveCodeDestructGhoul2Info(ghoul2: *const c_void);
    pub fn G2API_GetAnimFileNameIndex(ghoul2: *const c_void, modelnum: c_int, animFileIndex: c_int) -> *const c_char;
    pub fn G2API_GetAnimFileInternalNameIndex(index: c_int) -> *const c_char;
    pub fn G2API_GetSurfaceRenderStatus(ghoul2: *const c_void, modelnum: c_int, surfacenum: c_int) -> qboolean;
    pub fn G2API_AddSkinGore(ghoul2: *const c_void, gore: *const c_void, modelnum: c_int);
    pub fn G2API_ClearSkinGore(ghoul2: *const c_void, modelnum: c_int);
}

