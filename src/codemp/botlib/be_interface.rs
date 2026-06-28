
/*****************************************************************************
 * name:		be_interface.c // bk010221 - FIXME - DEAD code elimination
 *
 * desc:		bot library interface
 *
 * $Archive: /MissionPack/code/botlib/be_interface.c $
 * $Author: Zaphod $
 * $Revision: 16 $
 * $Modtime: 5/16/01 2:36p $
 * $Date: 5/16/01 2:41p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr::{addr_of_mut, null_mut};

// Type aliases and stubs for external dependencies
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Stub types for structures from included headers
#[repr(C)]
pub struct bot_entitystate_t {
    pub type_: c_int,
    pub flags: c_int,
    pub old_origin: vec3_t,
    pub solid: c_int,
    pub groundent: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub frame: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub angles: vec3_t,
    pub origin: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
}

#[repr(C)]
pub struct bot_goal_t {
    pub areanum: c_int,
    pub origin: vec3_t,
}

#[repr(C)]
pub struct bot_avoidspot_s {
    pub origin: vec3_t,
    pub radius: f32,
    pub type_: c_int,
}

#[repr(C)]
pub struct aas_reachability_t {
    pub areanum: c_int,
    pub start: vec3_t,
    pub end: vec3_t,
    pub traveltype: c_int,
}

#[repr(C)]
pub struct aas_export_t {
    pub AAS_EntityInfo: unsafe extern "C" fn(c_int, *mut c_void),
    pub AAS_Initialized: unsafe extern "C" fn() -> c_int,
    pub AAS_PresenceTypeBoundingBox: unsafe extern "C" fn(c_int, *mut f32, *mut f32),
    pub AAS_Time: unsafe extern "C" fn() -> i32,
    pub AAS_PointAreaNum: unsafe extern "C" fn(*const f32) -> c_int,
    pub AAS_PointReachabilityAreaIndex: unsafe extern "C" fn(*const f32) -> c_int,
    pub AAS_TraceAreas: unsafe extern "C" fn(*const f32, *const f32, *mut c_int, *mut vec3_t, c_int) -> c_int,
    pub AAS_BBoxAreas: unsafe extern "C" fn(*const f32, *const f32, *mut c_int, c_int) -> c_int,
    pub AAS_AreaInfo: unsafe extern "C" fn(c_int, *mut c_void),
    pub AAS_PointContents: unsafe extern "C" fn(*const f32) -> c_int,
    pub AAS_NextBSPEntity: unsafe extern "C" fn(c_int) -> c_int,
    pub AAS_ValueForBSPEpairKey: unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char,
    pub AAS_VectorForBSPEpairKey: unsafe extern "C" fn(c_int, *const c_char, *mut f32),
    pub AAS_FloatForBSPEpairKey: unsafe extern "C" fn(c_int, *const c_char) -> f32,
    pub AAS_IntForBSPEpairKey: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub AAS_AreaReachability: unsafe extern "C" fn(c_int) -> c_int,
    pub AAS_AreaTravelTimeToGoalArea: unsafe extern "C" fn(c_int, *const f32, c_int, c_int) -> c_int,
    pub AAS_EnableRoutingArea: unsafe extern "C" fn(c_int, c_int),
    pub AAS_PredictRoute: unsafe extern "C" fn(*mut c_void, c_int, *const f32, c_int, c_int, c_int, c_int, c_int, c_int),
    pub AAS_AlternativeRouteGoals: unsafe extern "C" fn(*const f32, c_int, *mut c_void, c_int, c_int, *mut *mut c_void, c_int, c_int) -> c_int,
    pub AAS_Swimming: unsafe extern "C" fn(*const f32) -> c_int,
    pub AAS_PredictClientMovement: unsafe extern "C" fn(*mut c_void, c_int, *const f32, c_int, c_int, *const f32, *const f32, c_int, c_int, f32, c_int),
}

#[repr(C)]
pub struct ea_export_t {
    pub EA_Command: unsafe extern "C" fn(c_int, *const c_char),
    pub EA_Say: unsafe extern "C" fn(c_int, *const c_char),
    pub EA_SayTeam: unsafe extern "C" fn(c_int, *const c_char),
    pub EA_Action: unsafe extern "C" fn(c_int, c_int),
    pub EA_Gesture: unsafe extern "C" fn(c_int),
    pub EA_Talk: unsafe extern "C" fn(c_int),
    pub EA_Attack: unsafe extern "C" fn(c_int),
    pub EA_Alt_Attack: unsafe extern "C" fn(c_int),
    pub EA_ForcePower: unsafe extern "C" fn(c_int, c_int),
    pub EA_Use: unsafe extern "C" fn(c_int),
    pub EA_Respawn: unsafe extern "C" fn(c_int),
    pub EA_Crouch: unsafe extern "C" fn(c_int),
    pub EA_MoveUp: unsafe extern "C" fn(c_int),
    pub EA_MoveDown: unsafe extern "C" fn(c_int),
    pub EA_MoveForward: unsafe extern "C" fn(c_int),
    pub EA_MoveBack: unsafe extern "C" fn(c_int),
    pub EA_MoveLeft: unsafe extern "C" fn(c_int),
    pub EA_MoveRight: unsafe extern "C" fn(c_int),
    pub EA_SelectWeapon: unsafe extern "C" fn(c_int, c_int),
    pub EA_Jump: unsafe extern "C" fn(c_int),
    pub EA_DelayedJump: unsafe extern "C" fn(c_int),
    pub EA_Move: unsafe extern "C" fn(c_int, *const f32, c_int),
    pub EA_View: unsafe extern "C" fn(c_int, *const f32),
    pub EA_GetInput: unsafe extern "C" fn(c_int, f32, *mut c_void),
    pub EA_EndRegular: unsafe extern "C" fn(c_int, f32),
    pub EA_ResetInput: unsafe extern "C" fn(c_int),
}

#[repr(C)]
pub struct ai_export_t {
    pub BotLoadCharacter: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub BotFreeCharacter: unsafe extern "C" fn(c_int),
    pub Characteristic_Float: unsafe extern "C" fn(c_int, c_int) -> f32,
    pub Characteristic_BFloat: unsafe extern "C" fn(c_int, c_int, f32, f32) -> f32,
    pub Characteristic_Integer: unsafe extern "C" fn(c_int, c_int) -> c_int,
    pub Characteristic_BInteger: unsafe extern "C" fn(c_int, c_int, c_int, c_int) -> c_int,
    pub Characteristic_String: unsafe extern "C" fn(c_int, c_int) -> *mut c_char,
    pub BotAllocChatState: unsafe extern "C" fn() -> c_int,
    pub BotFreeChatState: unsafe extern "C" fn(c_int),
    pub BotQueueConsoleMessage: unsafe extern "C" fn(c_int, c_int, *const c_char),
    pub BotRemoveConsoleMessage: unsafe extern "C" fn(c_int, c_int),
    pub BotNextConsoleMessage: unsafe extern "C" fn(c_int, *mut c_void),
    pub BotNumConsoleMessages: unsafe extern "C" fn(c_int) -> c_int,
    pub BotInitialChat: unsafe extern "C" fn(c_int, *const c_char, c_int, ...),
    pub BotNumInitialChats: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub BotReplyChat: unsafe extern "C" fn(c_int, *const c_char, c_int, c_int, ...) -> c_int,
    pub BotChatLength: unsafe extern "C" fn(c_int) -> c_int,
    pub BotEnterChat: unsafe extern "C" fn(c_int, c_int, c_int),
    pub BotGetChatMessage: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> c_int,
    pub StringContains: unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> c_int,
    pub BotFindMatch: unsafe extern "C" fn(*const c_char, *mut c_void, c_int, *mut c_void) -> c_int,
    pub BotMatchVariable: unsafe extern "C" fn(*mut c_void, c_int, *mut c_char, c_int),
    pub UnifyWhiteSpaces: unsafe extern "C" fn(*mut c_char),
    pub BotReplaceSynonyms: unsafe extern "C" fn(*mut c_char, c_int),
    pub BotLoadChatFile: unsafe extern "C" fn(c_int, *const c_char, *const c_char) -> c_int,
    pub BotSetChatGender: unsafe extern "C" fn(c_int, c_int),
    pub BotSetChatName: unsafe extern "C" fn(c_int, *const c_char),
    pub BotResetGoalState: unsafe extern "C" fn(c_int),
    pub BotResetAvoidGoals: unsafe extern "C" fn(c_int),
    pub BotRemoveFromAvoidGoals: unsafe extern "C" fn(c_int, c_int),
    pub BotPushGoal: unsafe extern "C" fn(c_int, *mut bot_goal_t),
    pub BotPopGoal: unsafe extern "C" fn(c_int),
    pub BotEmptyGoalStack: unsafe extern "C" fn(c_int),
    pub BotDumpAvoidGoals: unsafe extern "C" fn(c_int),
    pub BotDumpGoalStack: unsafe extern "C" fn(c_int),
    pub BotGoalName: unsafe extern "C" fn(c_int, *mut c_char, c_int),
    pub BotGetTopGoal: unsafe extern "C" fn(c_int, *mut bot_goal_t) -> c_int,
    pub BotGetSecondGoal: unsafe extern "C" fn(c_int, *mut bot_goal_t) -> c_int,
    pub BotChooseLTGItem: unsafe extern "C" fn(c_int, *const f32, *mut c_int) -> c_int,
    pub BotChooseNBGItem: unsafe extern "C" fn(c_int, *const f32, *mut c_int, *mut c_int, *mut c_int) -> c_int,
    pub BotTouchingGoal: unsafe extern "C" fn(*const f32, *mut bot_goal_t) -> c_int,
    pub BotItemGoalInVisButNotVisible: unsafe extern "C" fn(c_int, *const f32, *mut bot_goal_t) -> c_int,
    pub BotGetLevelItemGoal: unsafe extern "C" fn(c_int, c_int, *mut bot_goal_t) -> c_int,
    pub BotGetNextCampSpotGoal: unsafe extern "C" fn(c_int, *mut bot_goal_t) -> c_int,
    pub BotGetMapLocationGoal: unsafe extern "C" fn(*const c_char, *mut bot_goal_t) -> c_int,
    pub BotAvoidGoalTime: unsafe extern "C" fn(c_int, c_int) -> f32,
    pub BotSetAvoidGoalTime: unsafe extern "C" fn(c_int, c_int, f32),
    pub BotInitLevelItems: unsafe extern "C" fn(),
    pub BotUpdateEntityItems: unsafe extern "C" fn(),
    pub BotLoadItemWeights: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub BotFreeItemWeights: unsafe extern "C" fn(c_int),
    pub BotInterbreedGoalFuzzyLogic: unsafe extern "C" fn(*mut c_int, *mut c_int, *mut c_int) -> c_int,
    pub BotSaveGoalFuzzyLogic: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub BotMutateGoalFuzzyLogic: unsafe extern "C" fn(*mut c_int, c_int),
    pub BotAllocGoalState: unsafe extern "C" fn(c_int) -> c_int,
    pub BotFreeGoalState: unsafe extern "C" fn(c_int),
    pub BotResetMoveState: unsafe extern "C" fn(c_int),
    pub BotMoveToGoal: unsafe extern "C" fn(c_int, *mut c_void, c_int, c_int),
    pub BotMoveInDirection: unsafe extern "C" fn(c_int, *const f32, f32, c_int),
    pub BotResetAvoidReach: unsafe extern "C" fn(c_int),
    pub BotResetLastAvoidReach: unsafe extern "C" fn(c_int),
    pub BotReachabilityArea: unsafe extern "C" fn(*const f32, c_int) -> c_int,
    pub BotMovementViewTarget: unsafe extern "C" fn(c_int, *mut bot_goal_t, c_int, f32, *mut f32),
    pub BotPredictVisiblePosition: unsafe extern "C" fn(*const f32, c_int, *mut bot_goal_t, c_int, *mut f32),
    pub BotAllocMoveState: unsafe extern "C" fn() -> c_int,
    pub BotFreeMoveState: unsafe extern "C" fn(c_int),
    pub BotInitMoveState: unsafe extern "C" fn(c_int, *mut c_void),
    pub BotAddAvoidSpot: unsafe extern "C" fn(c_int, *const f32, f32, c_int),
    pub BotChooseBestFightWeapon: unsafe extern "C" fn(c_int, *mut c_int) -> c_int,
    pub BotGetWeaponInfo: unsafe extern "C" fn(c_int, c_int, *mut c_void),
    pub BotLoadWeaponWeights: unsafe extern "C" fn(c_int, *const c_char) -> c_int,
    pub BotAllocWeaponState: unsafe extern "C" fn() -> c_int,
    pub BotFreeWeaponState: unsafe extern "C" fn(c_int),
    pub BotResetWeaponState: unsafe extern "C" fn(c_int),
    pub GeneticParentsAndChildSelection: unsafe extern "C" fn(c_int, *mut c_int, *mut c_int, *mut c_int) -> c_int,
}

#[repr(C)]
pub struct botlib_export_t {
    pub aas: aas_export_t,
    pub ea: ea_export_t,
    pub ai: ai_export_t,
    pub BotLibSetup: unsafe extern "C" fn() -> c_int,
    pub BotLibShutdown: unsafe extern "C" fn() -> c_int,
    pub BotLibVarSet: unsafe extern "C" fn(*const c_char, *const c_char) -> c_int,
    pub BotLibVarGet: unsafe extern "C" fn(*const c_char, *mut c_char, c_int) -> c_int,
    pub PC_AddGlobalDefine: unsafe extern "C" fn(*const c_char) -> c_int,
    pub PC_LoadSourceHandle: unsafe extern "C" fn(*const c_char) -> c_int,
    pub PC_FreeSourceHandle: unsafe extern "C" fn(c_int) -> c_int,
    pub PC_ReadTokenHandle: unsafe extern "C" fn(c_int, *mut c_void) -> c_int,
    pub PC_SourceFileAndLine: unsafe extern "C" fn(c_int, *mut c_char, *mut c_int) -> c_int,
    pub PC_LoadGlobalDefines: unsafe extern "C" fn(*const c_char) -> c_int,
    pub PC_RemoveAllGlobalDefines: unsafe extern "C" fn() -> c_int,
    pub BotLibStartFrame: unsafe extern "C" fn(f32) -> c_int,
    pub BotLibLoadMap: unsafe extern "C" fn(*const c_char) -> c_int,
    pub BotLibUpdateEntity: unsafe extern "C" fn(c_int, *mut bot_entitystate_t) -> c_int,
    pub Test: unsafe extern "C" fn(c_int, *mut c_char, vec3_t, vec3_t) -> c_int,
}

#[repr(C)]
pub struct botlib_import_t {
    pub Print: unsafe extern "C" fn(c_int, *const c_char, ...),
    pub Trace: unsafe extern "C" fn(*mut c_void, *const f32, *const f32, *const f32, *const f32, c_int, c_int),
    pub EntityTrace: unsafe extern "C" fn(*mut c_void, *const f32, *const f32, *const f32, *const f32, c_int, c_int),
    pub PointContents: unsafe extern "C" fn(*const f32) -> c_int,
    pub inPVS: unsafe extern "C" fn(*const f32, *const f32) -> c_int,
    pub BSPEntityData: unsafe extern "C" fn() -> *mut c_char,
    pub BSPModelMinsMaxsOrigin: unsafe extern "C" fn(c_int, *const f32, *mut f32, *mut f32, *mut f32),
    pub BotClientCommand: unsafe extern "C" fn(c_int, *const c_char),
    pub GetMemory: unsafe extern "C" fn(c_int) -> *mut c_void,
    pub FreeMemory: unsafe extern "C" fn(*mut c_void),
    pub AvailableMemory: unsafe extern "C" fn() -> c_int,
    pub HunkAlloc: unsafe extern "C" fn(c_int) -> *mut c_void,
    pub FS_FOpenFile: unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int) -> c_int,
    pub FS_Read: unsafe extern "C" fn(*mut c_void, c_int, *mut c_void) -> c_int,
    pub FS_Write: unsafe extern "C" fn(*const c_void, c_int, *mut c_void) -> c_int,
    pub FS_FCloseFile: unsafe extern "C" fn(*mut c_void),
    pub FS_Seek: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int,
    pub DebugLineCreate: unsafe extern "C" fn() -> c_int,
    pub DebugLineDelete: unsafe extern "C" fn(c_int),
    pub DebugLineShow: unsafe extern "C" fn(c_int, *const f32, *const f32, c_int),
    pub DebugPolygonCreate: unsafe extern "C" fn(c_int, c_int, *mut f32) -> c_int,
    pub DebugPolygonDelete: unsafe extern "C" fn(c_int),
}

#[repr(C)]
pub struct botlib_globals_t {
    pub botlibsetup: c_int,
    pub maxclients: c_int,
    pub maxentities: c_int,
    pub goalareanum: c_int,
    pub goalorigin: vec3_t,
}

//library globals in a structure
pub static mut botlibglobals: botlib_globals_t = botlib_globals_t {
    botlibsetup: 0,
    maxclients: 0,
    maxentities: 0,
    goalareanum: 0,
    goalorigin: [0.0; 3],
};

pub static mut be_botlib_export: botlib_export_t = botlib_export_t {
    aas: aas_export_t {
        AAS_EntityInfo: dummy_fn_void,
        AAS_Initialized: dummy_fn_int,
        AAS_PresenceTypeBoundingBox: dummy_fn_void,
        AAS_Time: dummy_fn_i32,
        AAS_PointAreaNum: dummy_fn_int,
        AAS_PointReachabilityAreaIndex: dummy_fn_int,
        AAS_TraceAreas: dummy_fn_int,
        AAS_BBoxAreas: dummy_fn_int,
        AAS_AreaInfo: dummy_fn_void,
        AAS_PointContents: dummy_fn_int,
        AAS_NextBSPEntity: dummy_fn_int,
        AAS_ValueForBSPEpairKey: dummy_fn_ptr,
        AAS_VectorForBSPEpairKey: dummy_fn_void,
        AAS_FloatForBSPEpairKey: dummy_fn_f32,
        AAS_IntForBSPEpairKey: dummy_fn_int,
        AAS_AreaReachability: dummy_fn_int,
        AAS_AreaTravelTimeToGoalArea: dummy_fn_int,
        AAS_EnableRoutingArea: dummy_fn_void,
        AAS_PredictRoute: dummy_fn_void,
        AAS_AlternativeRouteGoals: dummy_fn_int,
        AAS_Swimming: dummy_fn_int,
        AAS_PredictClientMovement: dummy_fn_void,
    },
    ea: ea_export_t {
        EA_Command: dummy_fn_void,
        EA_Say: dummy_fn_void,
        EA_SayTeam: dummy_fn_void,
        EA_Action: dummy_fn_void,
        EA_Gesture: dummy_fn_void,
        EA_Talk: dummy_fn_void,
        EA_Attack: dummy_fn_void,
        EA_Alt_Attack: dummy_fn_void,
        EA_ForcePower: dummy_fn_void,
        EA_Use: dummy_fn_void,
        EA_Respawn: dummy_fn_void,
        EA_Crouch: dummy_fn_void,
        EA_MoveUp: dummy_fn_void,
        EA_MoveDown: dummy_fn_void,
        EA_MoveForward: dummy_fn_void,
        EA_MoveBack: dummy_fn_void,
        EA_MoveLeft: dummy_fn_void,
        EA_MoveRight: dummy_fn_void,
        EA_SelectWeapon: dummy_fn_void,
        EA_Jump: dummy_fn_void,
        EA_DelayedJump: dummy_fn_void,
        EA_Move: dummy_fn_void,
        EA_View: dummy_fn_void,
        EA_GetInput: dummy_fn_void,
        EA_EndRegular: dummy_fn_void,
        EA_ResetInput: dummy_fn_void,
    },
    ai: ai_export_t {
        BotLoadCharacter: dummy_fn_int,
        BotFreeCharacter: dummy_fn_void,
        Characteristic_Float: dummy_fn_f32,
        Characteristic_BFloat: dummy_fn_f32,
        Characteristic_Integer: dummy_fn_int,
        Characteristic_BInteger: dummy_fn_int,
        Characteristic_String: dummy_fn_ptr,
        BotAllocChatState: dummy_fn_int,
        BotFreeChatState: dummy_fn_void,
        BotQueueConsoleMessage: dummy_fn_void,
        BotRemoveConsoleMessage: dummy_fn_void,
        BotNextConsoleMessage: dummy_fn_void,
        BotNumConsoleMessages: dummy_fn_int,
        BotInitialChat: dummy_fn_void,
        BotNumInitialChats: dummy_fn_int,
        BotReplyChat: dummy_fn_int,
        BotChatLength: dummy_fn_int,
        BotEnterChat: dummy_fn_void,
        BotGetChatMessage: dummy_fn_int,
        StringContains: dummy_fn_int,
        BotFindMatch: dummy_fn_int,
        BotMatchVariable: dummy_fn_void,
        UnifyWhiteSpaces: dummy_fn_void,
        BotReplaceSynonyms: dummy_fn_void,
        BotLoadChatFile: dummy_fn_int,
        BotSetChatGender: dummy_fn_void,
        BotSetChatName: dummy_fn_void,
        BotResetGoalState: dummy_fn_void,
        BotResetAvoidGoals: dummy_fn_void,
        BotRemoveFromAvoidGoals: dummy_fn_void,
        BotPushGoal: dummy_fn_void,
        BotPopGoal: dummy_fn_void,
        BotEmptyGoalStack: dummy_fn_void,
        BotDumpAvoidGoals: dummy_fn_void,
        BotDumpGoalStack: dummy_fn_void,
        BotGoalName: dummy_fn_void,
        BotGetTopGoal: dummy_fn_int,
        BotGetSecondGoal: dummy_fn_int,
        BotChooseLTGItem: dummy_fn_int,
        BotChooseNBGItem: dummy_fn_int,
        BotTouchingGoal: dummy_fn_int,
        BotItemGoalInVisButNotVisible: dummy_fn_int,
        BotGetLevelItemGoal: dummy_fn_int,
        BotGetNextCampSpotGoal: dummy_fn_int,
        BotGetMapLocationGoal: dummy_fn_int,
        BotAvoidGoalTime: dummy_fn_f32,
        BotSetAvoidGoalTime: dummy_fn_void,
        BotInitLevelItems: dummy_fn_void,
        BotUpdateEntityItems: dummy_fn_void,
        BotLoadItemWeights: dummy_fn_int,
        BotFreeItemWeights: dummy_fn_void,
        BotInterbreedGoalFuzzyLogic: dummy_fn_int,
        BotSaveGoalFuzzyLogic: dummy_fn_int,
        BotMutateGoalFuzzyLogic: dummy_fn_void,
        BotAllocGoalState: dummy_fn_int,
        BotFreeGoalState: dummy_fn_void,
        BotResetMoveState: dummy_fn_void,
        BotMoveToGoal: dummy_fn_void,
        BotMoveInDirection: dummy_fn_void,
        BotResetAvoidReach: dummy_fn_void,
        BotResetLastAvoidReach: dummy_fn_void,
        BotReachabilityArea: dummy_fn_int,
        BotMovementViewTarget: dummy_fn_void,
        BotPredictVisiblePosition: dummy_fn_void,
        BotAllocMoveState: dummy_fn_int,
        BotFreeMoveState: dummy_fn_void,
        BotInitMoveState: dummy_fn_void,
        BotAddAvoidSpot: dummy_fn_void,
        BotChooseBestFightWeapon: dummy_fn_int,
        BotGetWeaponInfo: dummy_fn_void,
        BotLoadWeaponWeights: dummy_fn_int,
        BotAllocWeaponState: dummy_fn_int,
        BotFreeWeaponState: dummy_fn_void,
        BotResetWeaponState: dummy_fn_void,
        GeneticParentsAndChildSelection: dummy_fn_int,
    },
    BotLibSetup: Export_BotLibSetup,
    BotLibShutdown: Export_BotLibShutdown,
    BotLibVarSet: Export_BotLibVarSet,
    BotLibVarGet: Export_BotLibVarGet,
    PC_AddGlobalDefine: dummy_fn_int,
    PC_LoadSourceHandle: dummy_fn_int,
    PC_FreeSourceHandle: dummy_fn_int,
    PC_ReadTokenHandle: dummy_fn_int,
    PC_SourceFileAndLine: dummy_fn_int,
    PC_LoadGlobalDefines: dummy_fn_int,
    PC_RemoveAllGlobalDefines: dummy_fn_int,
    BotLibStartFrame: Export_BotLibStartFrame,
    BotLibLoadMap: Export_BotLibLoadMap,
    BotLibUpdateEntity: Export_BotLibUpdateEntity,
    Test: BotExportTest,
};

pub static mut botimport: botlib_import_t = botlib_import_t {
    Print: dummy_fn_print,
    Trace: dummy_fn_void,
    EntityTrace: dummy_fn_void,
    PointContents: dummy_fn_int,
    inPVS: dummy_fn_int,
    BSPEntityData: dummy_fn_ptr,
    BSPModelMinsMaxsOrigin: dummy_fn_void,
    BotClientCommand: dummy_fn_void,
    GetMemory: dummy_fn_ptr,
    FreeMemory: dummy_fn_void,
    AvailableMemory: dummy_fn_int,
    HunkAlloc: dummy_fn_ptr,
    FS_FOpenFile: dummy_fn_int,
    FS_Read: dummy_fn_int,
    FS_Write: dummy_fn_int,
    FS_FCloseFile: dummy_fn_void,
    FS_Seek: dummy_fn_int,
    DebugLineCreate: dummy_fn_int,
    DebugLineDelete: dummy_fn_void,
    DebugLineShow: dummy_fn_void,
    DebugPolygonCreate: dummy_fn_int,
    DebugPolygonDelete: dummy_fn_void,
};
//
pub static mut bot_developer: c_int = 0;
//qtrue if the library is setup
pub static mut botlibsetup: c_int = 0;

// Dummy function implementations
unsafe extern "C" fn dummy_fn_void(_: c_int) {}
unsafe extern "C" fn dummy_fn_print(_: c_int, _: *const c_char) {}
unsafe extern "C" fn dummy_fn_int(_: c_int) -> c_int { 0 }
unsafe extern "C" fn dummy_fn_i32(_: c_int) -> i32 { 0 }
unsafe extern "C" fn dummy_fn_f32(_: c_int) -> f32 { 0.0 }
unsafe extern "C" fn dummy_fn_ptr(_: c_int) -> *mut c_void { null_mut() }

//===========================================================================
//
// several functions used by the exported functions
//
//===========================================================================

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn Sys_MilliSeconds() -> c_int
{
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    (duration.as_millis() as c_int) / 1000
} //end of the function Sys_MilliSeconds
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ValidClientNumber(num: c_int, str_: *mut c_char) -> qboolean
{
    if num < 0 || num > unsafe { (*addr_of_mut!(botlibglobals)).maxclients }
    {
        //weird: the disabled stuff results in a crash
        (botimport.Print)(
            PRT_ERROR,
            "%s: invalid client number %d, [0, %d]\n\0".as_ptr() as *const c_char,
            str_,
            num,
            unsafe { (*addr_of_mut!(botlibglobals)).maxclients },
        );
        return qfalse;
    } //end if
    qtrue
} //end of the function BotValidateClientNumber
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ValidEntityNumber(num: c_int, str_: *mut c_char) -> qboolean
{
    if num < 0 || num > unsafe { (*addr_of_mut!(botlibglobals)).maxentities }
    {
        (botimport.Print)(
            PRT_ERROR,
            "%s: invalid entity number %d, [0, %d]\n\0".as_ptr() as *const c_char,
            str_,
            num,
            unsafe { (*addr_of_mut!(botlibglobals)).maxentities },
        );
        return qfalse;
    } //end if
    qtrue
} //end of the function BotValidateClientNumber
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn BotLibSetup(str_: *mut c_char) -> qboolean
{
    if unsafe { (*addr_of_mut!(botlibglobals)).botlibsetup == 0 }
    {
        (botimport.Print)(
            PRT_ERROR,
            "%s: bot library used before being setup\n\0".as_ptr() as *const c_char,
            str_,
        );
        return qfalse;
    } //end if
    qtrue
} //end of the function BotLibSetup

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibSetup() -> c_int
{
    let errnum: c_int;

    bot_developer = LibVarGetValue(b"bot_developer\0".as_ptr() as *const c_char) as c_int;
    Com_Memset(
        addr_of_mut!(botlibglobals) as *mut c_void,
        0,
        std::mem::size_of::<botlib_globals_t>(),
    ); // bk001207 - init
    //initialize byte swapping (litte endian etc.)
    //	Swap_Init();
    Log_Open(b"botlib.log\0".as_ptr() as *mut c_char);
    //
    //	botimport.Print(PRT_MESSAGE, "------- BotLib Initialization -------\n");
    //
    (*addr_of_mut!(botlibglobals)).maxclients = LibVarValue(
        b"maxclients\0".as_ptr() as *const c_char,
        b"128\0".as_ptr() as *const c_char,
    ) as c_int;
    (*addr_of_mut!(botlibglobals)).maxentities = LibVarValue(
        b"maxentities\0".as_ptr() as *const c_char,
        b"1024\0".as_ptr() as *const c_char,
    ) as c_int;

    errnum = AAS_Setup(); //be_aas_main.c
    if errnum != BLERR_NOERROR {
        return errnum;
    }
    errnum = EA_Setup(); //be_ea.c
    if errnum != BLERR_NOERROR {
        return errnum;
    }
    /*
    errnum = BotSetupWeaponAI();	//be_ai_weap.c
    if (errnum != BLERR_NOERROR)return errnum;
    errnum = BotSetupGoalAI();		//be_ai_goal.c
    if (errnum != BLERR_NOERROR) return errnum;
    errnum = BotSetupChatAI();		//be_ai_chat.c
    if (errnum != BLERR_NOERROR) return errnum;
    errnum = BotSetupMoveAI();		//be_ai_move.c
    if (errnum != BLERR_NOERROR) return errnum;
    */
    botlibsetup = qtrue;
    (*addr_of_mut!(botlibglobals)).botlibsetup = qtrue;

    BLERR_NOERROR
} //end of the function Export_BotLibSetup
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibShutdown() -> c_int
{
    if BotLibSetup(b"BotLibShutdown\0".as_ptr() as *mut c_char) == 0 {
        return BLERR_LIBRARYNOTSETUP;
    }
    #[cfg(not(feature = "demo"))]
    {
        //DumpFileCRCs();
    }
    //
    BotShutdownChatAI(); //be_ai_chat.c
    BotShutdownMoveAI(); //be_ai_move.c
    BotShutdownGoalAI(); //be_ai_goal.c
    BotShutdownWeaponAI(); //be_ai_weap.c
    BotShutdownWeights(); //be_ai_weight.c
    BotShutdownCharacters(); //be_ai_char.c
    //shud down aas
    AAS_Shutdown();
    //shut down bot elemantary actions
    EA_Shutdown();
    //free all libvars
    LibVarDeAllocAll();
    //remove all global defines from the pre compiler
    PC_RemoveAllGlobalDefines();

    //dump all allocated memory
    //	DumpMemory();
    #[cfg(feature = "debug")]
    {
        PrintMemoryLabels();
    }
    //shut down library log file
    Log_Shutdown();
    //
    botlibsetup = qfalse;
    (*addr_of_mut!(botlibglobals)).botlibsetup = qfalse;
    // print any files still open
    PC_CheckOpenSourceHandles();
    //
    BLERR_NOERROR
} //end of the function Export_BotLibShutdown
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibVarSet(var_name: *const c_char, value: *const c_char) -> c_int
{
    LibVarSet(var_name, value);
    BLERR_NOERROR
} //end of the function Export_BotLibVarSet
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibVarGet(var_name: *const c_char, value: *mut c_char, size: c_int) -> c_int
{
    let varvalue: *mut c_char;

    varvalue = LibVarGetString(var_name);
    strncpy(value, varvalue, (size - 1) as usize);
    *value.offset((size - 1) as isize) = 0;
    BLERR_NOERROR
} //end of the function Export_BotLibVarGet
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibStartFrame(time: f32) -> c_int
{
    if BotLibSetup(b"BotStartFrame\0".as_ptr() as *mut c_char) == 0 {
        return BLERR_LIBRARYNOTSETUP;
    }
    AAS_StartFrame(time)
} //end of the function Export_BotLibStartFrame
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibLoadMap(mapname: *const c_char) -> c_int
{
    #[cfg(feature = "debug")]
    let starttime: c_int = Sys_MilliSeconds();

    let errnum: c_int;

    if BotLibSetup(b"BotLoadMap\0".as_ptr() as *mut c_char) == 0 {
        return BLERR_LIBRARYNOTSETUP;
    }
    //
    (botimport.Print)(
        PRT_MESSAGE,
        b"------------ Map Loading ------------\n\0".as_ptr() as *const c_char,
    );
    //startup AAS for the current map, model and sound index
    errnum = AAS_LoadMap(mapname);
    if errnum != BLERR_NOERROR {
        return errnum;
    }
    //initialize the items in the level
    BotInitLevelItems(); //be_ai_goal.h
    BotSetBrushModelTypes(); //be_ai_move.h
    //
    (botimport.Print)(
        PRT_MESSAGE,
        b"-------------------------------------\n\0".as_ptr() as *const c_char,
    );
    #[cfg(feature = "debug")]
    {
        (botimport.Print)(
            PRT_MESSAGE,
            b"map loaded in %d msec\n\0".as_ptr() as *const c_char,
            Sys_MilliSeconds() - starttime,
        );
    }
    //
    BLERR_NOERROR
} //end of the function Export_BotLibLoadMap
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe extern "C" fn Export_BotLibUpdateEntity(ent: c_int, state: *mut bot_entitystate_t) -> c_int
{
    if BotLibSetup(b"BotUpdateEntity\0".as_ptr() as *mut c_char) == 0 {
        return BLERR_LIBRARYNOTSETUP;
    }
    if ValidEntityNumber(ent, b"BotUpdateEntity\0".as_ptr() as *mut c_char) == 0 {
        return BLERR_INVALIDENTITYNUMBER;
    }

    AAS_UpdateEntity(ent, state)
} //end of the function Export_BotLibUpdateEntity
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
extern "C" {
    pub fn AAS_TestMovementPrediction(entnum: c_int, origin: *const f32, dir: *const f32);
    pub fn ElevatorBottomCenter(reach: *const c_void, bottomcenter: *mut f32);
    pub fn BotGetReachabilityToGoal(
        origin: *const f32,
        areanum: c_int,
        lastgoalareanum: c_int,
        lastareanum: c_int,
        avoidreach: *mut c_int,
        avoidreachtimes: *mut f32,
        avoidreachtries: *mut c_int,
        goal: *mut bot_goal_t,
        travelflags: c_int,
        movetravelflags: c_int,
        avoidspots: *mut bot_avoidspot_s,
        numavoidspots: c_int,
        flags: *mut c_int,
    ) -> c_int;

    pub fn AAS_PointLight(origin: *const f32, red: *mut c_int, green: *mut c_int, blue: *mut c_int) -> c_int;

    pub fn AAS_TraceAreas(
        start: *const f32,
        end: *const f32,
        areas: *mut c_int,
        points: *mut vec3_t,
        maxareas: c_int,
    ) -> c_int;

    pub fn AAS_Reachability_WeaponJump(area1num: c_int, area2num: c_int) -> c_int;

    pub fn BotFuzzyPointReachabilityArea(origin: *const f32) -> c_int;

    pub fn BotGapDistance(origin: *const f32, hordir: *const f32, entnum: c_int) -> f32;

    pub fn AAS_FloodAreas(origin: *const f32);

    pub fn AAS_AreaCluster(areanum: c_int) -> c_int;
    pub fn AAS_PointPresenceType(origin: *const f32) -> c_int;
    pub fn AAS_AreaTravelTimeToGoalArea(from: c_int, from_pos: *const f32, to: c_int, travelflags: c_int) -> c_int;
    pub fn AAS_ClearShownPolygons();
    pub fn AAS_ClearShownDebugLines();
    pub fn AAS_ShowAreaPolygons(areanum: c_int, numsectors: c_int, showflags: c_int);
    pub fn AAS_ShowReachableAreas(areanum: c_int);
    pub fn AAS_ShowReachability(reach: *const c_void);
    pub fn AAS_ReachabilityFromNum(reachnum: c_int, reach: *mut c_void);
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn VectorMA(v: *const f32, scale: f32, dir: *const f32, out: *mut f32);
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn VectorClear(v: *mut f32);

    pub static mut aasworld: c_int;

    pub fn LibVarGetValue(name: *const c_char) -> f32;
    pub fn LibVarValue(name: *const c_char, default: *const c_char) -> f32;
    pub fn LibVarGetString(name: *const c_char) -> *mut c_char;
    pub fn LibVarSet(name: *const c_char, value: *const c_char);
    pub fn Com_Memset(ptr: *mut c_void, val: c_int, size: usize);
    pub fn Com_Memcpy(dst: *mut c_void, src: *const c_void, size: usize);
    pub fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn Log_Open(filename: *mut c_char);
    pub fn Log_Shutdown();
    pub fn AAS_Setup() -> c_int;
    pub fn EA_Setup() -> c_int;
    pub fn BotShutdownChatAI();
    pub fn BotShutdownMoveAI();
    pub fn BotShutdownGoalAI();
    pub fn BotShutdownWeaponAI();
    pub fn BotShutdownWeights();
    pub fn BotShutdownCharacters();
    pub fn AAS_Shutdown();
    pub fn EA_Shutdown();
    pub fn LibVarDeAllocAll();
    pub fn PC_RemoveAllGlobalDefines() -> c_int;
    pub fn PrintMemoryLabels();
    pub fn PC_CheckOpenSourceHandles();
    pub fn AAS_StartFrame(time: f32) -> c_int;
    pub fn AAS_LoadMap(mapname: *const c_char) -> c_int;
    pub fn BotInitLevelItems();
    pub fn BotSetBrushModelTypes();
    pub fn AAS_UpdateEntity(ent: c_int, state: *mut c_void) -> c_int;
}

unsafe extern "C" fn BotExportTest(parm0: c_int, parm1: *mut c_char, parm2: vec3_t, parm3: vec3_t) -> c_int
{

    //	return AAS_PointLight(parm2, NULL, NULL, NULL);

    #[cfg(feature = "debug")]
    {
        let mut area: c_int = -1;
        let mut line: [c_int; 2] = [0; 2];
        let mut newarea: c_int;
        let mut i: c_int;
        let mut highlightarea: c_int;
        let mut flood: c_int;
        //	int reachnum;
        let mut eye: vec3_t;
        let mut forward: vec3_t;
        let mut right: vec3_t;
        let mut end: vec3_t;
        let mut origin: vec3_t;
        //	vec3_t bottomcenter;
        //	aas_trace_t trace;
        //	aas_face_t *face;
        //	aas_entity_t *ent;
        //	bsp_trace_t bsptrace;
        //	aas_reachability_t reach;
        //	bot_goal_t goal;

        // clock_t start_time, end_time;
        let mins: vec3_t = [-16.0, -16.0, -24.0];
        let maxs: vec3_t = [16.0, 16.0, 32.0];

        //	int areas[10], numareas;


        //return 0;

        if aasworld == 0 {
            return 0;
        }

        /*
        if (parm0 & 1)
        {
            AAS_ClearShownPolygons();
            AAS_FloodAreas(parm2);
        } //end if
        return 0;
        */
        i = 0;
        while i < 2 {
            if line[i as usize] == 0 {
                line[i as usize] = (botimport.DebugLineCreate)();
            }
            i += 1;
        }

        //	AAS_ClearShownDebugLines();

        //if (AAS_AgainstLadder(parm2)) botimport.Print(PRT_MESSAGE, "against ladder\n");
        //BotOnGround(parm2, PRESENCE_NORMAL, 1, &newarea, &newarea);
        //botimport.Print(PRT_MESSAGE, "%f %f %f\n", parm2[0], parm2[1], parm2[2]);
        //*
        highlightarea = LibVarGetValue(b"bot_highlightarea\0".as_ptr() as *const c_char) as c_int;
        if highlightarea > 0 {
            newarea = highlightarea;
        } //end if
        else {
            VectorCopy(parm2.as_ptr(), origin.as_mut_ptr());
            origin[2] += 0.5;
            //newarea = AAS_PointAreaNum(origin);
            newarea = BotFuzzyPointReachabilityArea(origin.as_ptr());
        } //end else

        (botimport.Print)(
            PRT_MESSAGE,
            "\rtravel time to goal (%d) = %d  \0".as_ptr() as *const c_char,
            (*addr_of_mut!(botlibglobals)).goalareanum,
            AAS_AreaTravelTimeToGoalArea(
                newarea,
                origin.as_ptr(),
                (*addr_of_mut!(botlibglobals)).goalareanum,
                TFL_DEFAULT,
            ),
        );
        //newarea = BotReachabilityArea(origin, qtrue);
        if newarea != area {
            (botimport.Print)(
                PRT_MESSAGE,
                "origin = %f, %f, %f\n\0".as_ptr() as *const c_char,
                origin[0],
                origin[1],
                origin[2],
            );
            area = newarea;
            (botimport.Print)(
                PRT_MESSAGE,
                "new area %d, cluster %d, presence type %d\n\0".as_ptr() as *const c_char,
                area,
                AAS_AreaCluster(area),
                AAS_PointPresenceType(origin.as_ptr()),
            );
            (botimport.Print)(PRT_MESSAGE, "area contents: \0".as_ptr() as *const c_char);
            if aasworld != 0 {
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_WATER != 0
                {
                    (botimport.Print)(PRT_MESSAGE, "water &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_LAVA != 0 {
                    (botimport.Print)(PRT_MESSAGE, "lava &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_SLIME != 0
                {
                    (botimport.Print)(PRT_MESSAGE, "slime &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_JUMPPAD != 0
                {
                    (botimport.Print)(PRT_MESSAGE, "jump pad &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_CLUSTERPORTAL
                    != 0
                {
                    (botimport.Print)(PRT_MESSAGE, "cluster portal &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_VIEWPORTAL != 0 {
                    (botimport.Print)(PRT_MESSAGE, "view portal &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_DONOTENTER != 0
                {
                    (botimport.Print)(PRT_MESSAGE, "do not enter &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int & AREACONTENTS_MOVER != 0 {
                    (botimport.Print)(PRT_MESSAGE, "mover &\0".as_ptr() as *const c_char);
                } //end if
                if (*(aasworld as *mut c_int)).add(area as usize) as c_int == 0 {
                    (botimport.Print)(PRT_MESSAGE, "empty\0".as_ptr() as *const c_char);
                } //end if
            }
            (botimport.Print)(PRT_MESSAGE, "\n\0".as_ptr() as *const c_char);
            (botimport.Print)(
                PRT_MESSAGE,
                "travel time to goal (%d) = %d\n\0".as_ptr() as *const c_char,
                (*addr_of_mut!(botlibglobals)).goalareanum,
                AAS_AreaTravelTimeToGoalArea(
                    newarea,
                    origin.as_ptr(),
                    (*addr_of_mut!(botlibglobals)).goalareanum,
                    TFL_DEFAULT | TFL_ROCKETJUMP,
                ),
            );
            /*
            VectorCopy(origin, end);
            end[2] += 5;
            numareas = AAS_TraceAreas(origin, end, areas, NULL, 10);
            AAS_TraceClientBBox(origin, end, PRESENCE_CROUCH, -1);
            botimport.Print(PRT_MESSAGE, "num areas = %d, area = %d\n", numareas, areas[0]);
            */
            /*
            botlibglobals.goalareanum = newarea;
            VectorCopy(parm2, botlibglobals.goalorigin);
            botimport.Print(PRT_MESSAGE, "new goal %2.1f %2.1f %2.1f area %d\n",
                                origin[0], origin[1], origin[2], newarea);
            */
        } //end if
        //*
        flood = LibVarGetValue(b"bot_flood\0".as_ptr() as *const c_char) as c_int;
        if parm0 & 1 != 0 {
            if flood != 0 {
                AAS_ClearShownPolygons();
                AAS_ClearShownDebugLines();
                AAS_FloodAreas(parm2.as_ptr());
            } else {
                (*addr_of_mut!(botlibglobals)).goalareanum = newarea;
                VectorCopy(parm2.as_ptr(), (*addr_of_mut!(botlibglobals)).goalorigin.as_mut_ptr());
                (botimport.Print)(
                    PRT_MESSAGE,
                    "new goal %2.1f %2.1f %2.1f area %d\n\0".as_ptr() as *const c_char,
                    origin[0],
                    origin[1],
                    origin[2],
                    newarea,
                );
            }
        } //end if*/
        if flood != 0 {
            return 0;
        }
        //	if (parm0 & BUTTON_USE)
        //	{
        //		botlibglobals.runai = !botlibglobals.runai;
        //		if (botlibglobals.runai) botimport.Print(PRT_MESSAGE, "started AI\n");
        //		else botimport.Print(PRT_MESSAGE, "stopped AI\n");
        //		//* /
        /*
        goal.areanum = botlibglobals.goalareanum;
        reachnum = BotGetReachabilityToGoal(parm2, newarea, 1,
                                ms.avoidreach, ms.avoidreachtimes,
                                &goal, TFL_DEFAULT);
        if (!reachnum)
        {
            botimport.Print(PRT_MESSAGE, "goal not reachable\n");
        } //end if
        else
        {
            AAS_ReachabilityFromNum(reachnum, &reach);
            AAS_ClearShownDebugLines();
            AAS_ShowArea(area, qtrue);
            AAS_ShowArea(reach.areanum, qtrue);
            AAS_DrawCross(reach.start, 6, LINECOLOR_BLUE);
            AAS_DrawCross(reach.end, 6, LINECOLOR_RED);
            //
            if ((reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_ELEVATOR)
            {
                ElevatorBottomCenter(&reach, bottomcenter);
                AAS_DrawCross(bottomcenter, 10, LINECOLOR_GREEN);
            } //end if
        } //end else*/
        //		botimport.Print(PRT_MESSAGE, "travel time to goal = %d\n",
        //					AAS_AreaTravelTimeToGoalArea(area, origin, botlibglobals.goalareanum, TFL_DEFAULT));
        //		botimport.Print(PRT_MESSAGE, "test rj from 703 to 716\n");
        //		AAS_Reachability_WeaponJump(703, 716);
        //	} //end if*/

        /*	face = AAS_AreaGroundFace(newarea, parm2);
        if (face)
        {
            AAS_ShowFace(face - aasworld.faces);
        } //end if*/
        /*
        AAS_ClearShownDebugLines();
        AAS_ShowArea(newarea, parm0 & BUTTON_USE);
        AAS_ShowReachableAreas(area);
        */
        AAS_ClearShownPolygons();
        AAS_ClearShownDebugLines();
        AAS_ShowAreaPolygons(newarea, 1, parm0 & 4);
        if parm0 & 2 != 0 {
            AAS_ShowReachableAreas(area);
        } else {
            let mut lastgoalareanum: c_int = 0;
            let mut lastareanum: c_int = 0;
            let mut avoidreach: [c_int; MAX_AVOIDREACH as usize] = [0; MAX_AVOIDREACH as usize];
            let mut avoidreachtimes: [f32; MAX_AVOIDREACH as usize] = [0.0; MAX_AVOIDREACH as usize];
            let mut avoidreachtries: [c_int; MAX_AVOIDREACH as usize] = [0; MAX_AVOIDREACH as usize];
            let mut reachnum: c_int;
            let mut resultFlags: c_int = 0;
            let mut goal: bot_goal_t = bot_goal_t {
                areanum: 0,
                origin: [0.0; 3],
            };
            let mut reach: aas_reachability_t = aas_reachability_t {
                areanum: 0,
                start: [0.0; 3],
                end: [0.0; 3],
                traveltype: 0,
            };

            /*
            goal.areanum = botlibglobals.goalareanum;
            VectorCopy(botlibglobals.goalorigin, goal.origin);
            reachnum = BotGetReachabilityToGoal(origin, newarea,
                                          lastgoalareanum, lastareanum,
                                          avoidreach, avoidreachtimes, avoidreachtries,
                                          &goal, TFL_DEFAULT|TFL_FUNCBOB|TFL_ROCKETJUMP, TFL_DEFAULT|TFL_FUNCBOB|TFL_ROCKETJUMP,
                                          NULL, 0, &resultFlags);
            AAS_ReachabilityFromNum(reachnum, &reach);
            AAS_ShowReachability(&reach);
            */
            let mut curarea: c_int;
            let mut curorigin: vec3_t;

            goal.areanum = (*addr_of_mut!(botlibglobals)).goalareanum;
            VectorCopy(
                (*addr_of_mut!(botlibglobals)).goalorigin.as_ptr(),
                goal.origin.as_mut_ptr(),
            );
            VectorCopy(origin.as_ptr(), curorigin.as_mut_ptr());
            curarea = newarea;
            i = 0;
            while i < 100 {
                if curarea == goal.areanum {
                    break;
                }
                reachnum = BotGetReachabilityToGoal(
                    curorigin.as_ptr(),
                    curarea,
                    lastgoalareanum,
                    lastareanum,
                    avoidreach.as_mut_ptr(),
                    avoidreachtimes.as_mut_ptr(),
                    avoidreachtries.as_mut_ptr(),
                    &mut goal,
                    TFL_DEFAULT | TFL_FUNCBOB | TFL_ROCKETJUMP,
                    TFL_DEFAULT | TFL_FUNCBOB | TFL_ROCKETJUMP,
                    null_mut(),
                    0,
                    &mut resultFlags,
                );
                AAS_ReachabilityFromNum(reachnum, &mut reach as *mut _ as *mut c_void);
                AAS_ShowReachability(&reach as *const _ as *const c_void);
                VectorCopy(reach.end.as_ptr(), origin.as_mut_ptr());
                lastareanum = curarea;
                curarea = reach.areanum;
                i += 1;
            }
        } //end else
        VectorClear(forward.as_mut_ptr());
        //BotGapDistance(origin, forward, 0);
        /*
        if (parm0 & BUTTON_USE)
        {
            botimport.Print(PRT_MESSAGE, "test rj from 703 to 716\n");
            AAS_Reachability_WeaponJump(703, 716);
        } //end if*/

        AngleVectors(parm3.as_ptr(), forward.as_mut_ptr(), right.as_mut_ptr(), null_mut() as *mut f32);
        //get the eye 16 units to the right of the origin
        VectorMA(parm2.as_ptr(), 8.0, right.as_ptr(), eye.as_mut_ptr());
        //get the eye 24 units up
        eye[2] += 24.0;
        //get the end point for the line to be traced
        VectorMA(eye.as_ptr(), 800.0, forward.as_ptr(), end.as_mut_ptr());

        //	AAS_TestMovementPrediction(1, parm2, forward);
        /*
        //trace the line to find the hit point
        trace = AAS_TraceClientBBox(eye, end, PRESENCE_NORMAL, 1);
        if (!line[0]) line[0] = botimport.DebugLineCreate();
        botimport.DebugLineShow(line[0], eye, trace.endpos, LINECOLOR_BLUE);
        //
        AAS_ClearShownDebugLines();
        if (trace.ent)
        {
            ent = &aasworld.entities[trace.ent];
            AAS_ShowBoundingBox(ent->origin, ent->mins, ent->maxs);
        } //end if
    */

        /*
        start_time = clock();
        for (i = 0; i < 2000; i++)
        {
            AAS_Trace2(eye, mins, maxs, end, 1, MASK_PLAYERSOLID);
    //		AAS_TraceClientBBox(eye, end, PRESENCE_NORMAL, 1);
        } //end for
        end_time = clock();
        botimport.Print(PRT_MESSAGE, "me %lu clocks, %lu CLOCKS_PER_SEC\n", end_time - start_time, CLOCKS_PER_SEC);
        start_time = clock();
        for (i = 0; i < 2000; i++)
        {
            AAS_Trace(eye, mins, maxs, end, 1, MASK_PLAYERSOLID);
        } //end for
        end_time = clock();
        botimport.Print(PRT_MESSAGE, "id %lu clocks, %lu CLOCKS_PER_SEC\n", end_time - start_time, CLOCKS_PER_SEC);
    */

        // TTimo: nested comments are BAD for gcc -Werror, use #if 0 instead..
        #[cfg(feature = "bot_export_test_detailed")]
        {
            /*
            AAS_ClearShownDebugLines();
            //bsptrace = AAS_Trace(eye, NULL, NULL, end, 1, MASK_PLAYERSOLID);
            bsptrace = AAS_Trace(eye, mins, maxs, end, 1, MASK_PLAYERSOLID);
            if (!line[0]) line[0] = botimport.DebugLineCreate();
            botimport.DebugLineShow(line[0], eye, bsptrace.endpos, LINECOLOR_YELLOW);
            if (bsptrace.fraction < 1.0)
            {
                face = AAS_TraceEndFace(&trace);
                if (face)
                {
                    AAS_ShowFace(face - aasworld.faces);
                } //end if

                AAS_DrawPlaneCross(bsptrace.endpos,
                                    bsptrace.plane.normal,
                                    bsptrace.plane.dist + bsptrace.exp_dist,
                                    bsptrace.plane.type, LINECOLOR_GREEN);
                if (trace.ent)
                {
                    ent = &aasworld.entities[trace.ent];
                    AAS_ShowBoundingBox(ent->origin, ent->mins, ent->maxs);
                } //end if
            } //end if
            //bsptrace = AAS_Trace2(eye, NULL, NULL, end, 1, MASK_PLAYERSOLID);
            bsptrace = AAS_Trace2(eye, mins, maxs, end, 1, MASK_PLAYERSOLID);
            botimport.DebugLineShow(line[1], eye, bsptrace.endpos, LINECOLOR_BLUE);
            if (bsptrace.fraction < 1.0)
            {
                AAS_DrawPlaneCross(bsptrace.endpos,
                                    bsptrace.plane.normal,
                                    bsptrace.plane.dist,// + bsptrace.exp_dist,
                                    bsptrace.plane.type, LINECOLOR_RED);
                if (bsptrace.ent)
                {
                    ent = &aasworld.entities[bsptrace.ent];
                    AAS_ShowBoundingBox(ent->origin, ent->mins, ent->maxs);
                } //end if
            } //end if
            */
        }
    }

    0
} //end of the function BotExportTest


/*
============
Init_AAS_Export
============
*/
unsafe fn Init_AAS_Export(aas: *mut aas_export_t) {
    //--------------------------------------------
    // be_aas_entity.c
    //--------------------------------------------
    (*aas).AAS_EntityInfo = AAS_EntityInfo;
    //--------------------------------------------
    // be_aas_main.c
    //--------------------------------------------
    (*aas).AAS_Initialized = AAS_Initialized;
    (*aas).AAS_PresenceTypeBoundingBox = AAS_PresenceTypeBoundingBox;
    (*aas).AAS_Time = AAS_Time;
    //--------------------------------------------
    // be_aas_sample.c
    //--------------------------------------------
    (*aas).AAS_PointAreaNum = AAS_PointAreaNum;
    (*aas).AAS_PointReachabilityAreaIndex = AAS_PointReachabilityAreaIndex;
    (*aas).AAS_TraceAreas = AAS_TraceAreas;
    (*aas).AAS_BBoxAreas = AAS_BBoxAreas;
    (*aas).AAS_AreaInfo = AAS_AreaInfo;
    //--------------------------------------------
    // be_aas_bspq3.c
    //--------------------------------------------
    (*aas).AAS_PointContents = AAS_PointContents;
    (*aas).AAS_NextBSPEntity = AAS_NextBSPEntity;
    (*aas).AAS_ValueForBSPEpairKey = AAS_ValueForBSPEpairKey;
    (*aas).AAS_VectorForBSPEpairKey = AAS_VectorForBSPEpairKey;
    (*aas).AAS_FloatForBSPEpairKey = AAS_FloatForBSPEpairKey;
    (*aas).AAS_IntForBSPEpairKey = AAS_IntForBSPEpairKey;
    //--------------------------------------------
    // be_aas_reach.c
    //--------------------------------------------
    (*aas).AAS_AreaReachability = AAS_AreaReachability;
    //--------------------------------------------
    // be_aas_route.c
    //--------------------------------------------
    (*aas).AAS_AreaTravelTimeToGoalArea = AAS_AreaTravelTimeToGoalArea;
    (*aas).AAS_EnableRoutingArea = AAS_EnableRoutingArea;
    (*aas).AAS_PredictRoute = AAS_PredictRoute;
    //--------------------------------------------
    // be_aas_altroute.c
    //--------------------------------------------
    (*aas).AAS_AlternativeRouteGoals = AAS_AlternativeRouteGoals;
    //--------------------------------------------
    // be_aas_move.c
    //--------------------------------------------
    (*aas).AAS_Swimming = AAS_Swimming;
    (*aas).AAS_PredictClientMovement = AAS_PredictClientMovement;
}

extern "C" {
    pub unsafe fn AAS_EntityInfo();
    pub unsafe fn AAS_Initialized() -> c_int;
    pub unsafe fn AAS_PresenceTypeBoundingBox();
    pub unsafe fn AAS_Time() -> i32;
    pub unsafe fn AAS_PointAreaNum();
    pub unsafe fn AAS_PointReachabilityAreaIndex();
    pub unsafe fn AAS_TraceAreas();
    pub unsafe fn AAS_BBoxAreas();
    pub unsafe fn AAS_AreaInfo();
    pub unsafe fn AAS_PointContents();
    pub unsafe fn AAS_NextBSPEntity();
    pub unsafe fn AAS_ValueForBSPEpairKey();
    pub unsafe fn AAS_VectorForBSPEpairKey();
    pub unsafe fn AAS_FloatForBSPEpairKey();
    pub unsafe fn AAS_IntForBSPEpairKey();
    pub unsafe fn AAS_AreaReachability();
    pub unsafe fn AAS_AreaTravelTimeToGoalArea();
    pub unsafe fn AAS_EnableRoutingArea();
    pub unsafe fn AAS_PredictRoute();
    pub unsafe fn AAS_AlternativeRouteGoals();
    pub unsafe fn AAS_Swimming();
    pub unsafe fn AAS_PredictClientMovement();
    pub unsafe fn EA_Command();
    pub unsafe fn EA_Say();
    pub unsafe fn EA_SayTeam();
    pub unsafe fn EA_Action();
    pub unsafe fn EA_Gesture();
    pub unsafe fn EA_Talk();
    pub unsafe fn EA_Attack();
    pub unsafe fn EA_Alt_Attack();
    pub unsafe fn EA_ForcePower();
    pub unsafe fn EA_Use();
    pub unsafe fn EA_Respawn();
    pub unsafe fn EA_Crouch();
    pub unsafe fn EA_MoveUp();
    pub unsafe fn EA_MoveDown();
    pub unsafe fn EA_MoveForward();
    pub unsafe fn EA_MoveBack();
    pub unsafe fn EA_MoveLeft();
    pub unsafe fn EA_MoveRight();
    pub unsafe fn EA_SelectWeapon();
    pub unsafe fn EA_Jump();
    pub unsafe fn EA_DelayedJump();
    pub unsafe fn EA_Move();
    pub unsafe fn EA_View();
    pub unsafe fn EA_GetInput();
    pub unsafe fn EA_EndRegular();
    pub unsafe fn EA_ResetInput();
    pub unsafe fn BotLoadCharacter();
    pub unsafe fn BotFreeCharacter();
    pub unsafe fn Characteristic_Float();
    pub unsafe fn Characteristic_BFloat();
    pub unsafe fn Characteristic_Integer();
    pub unsafe fn Characteristic_BInteger();
    pub unsafe fn Characteristic_String();
    pub unsafe fn BotAllocChatState();
    pub unsafe fn BotFreeChatState();
    pub unsafe fn BotQueueConsoleMessage();
    pub unsafe fn BotRemoveConsoleMessage();
    pub unsafe fn BotNextConsoleMessage();
    pub unsafe fn BotNumConsoleMessages();
    pub unsafe fn BotInitialChat();
    pub unsafe fn BotNumInitialChats();
    pub unsafe fn BotReplyChat();
    pub unsafe fn BotChatLength();
    pub unsafe fn BotEnterChat();
    pub unsafe fn BotGetChatMessage();
    pub unsafe fn StringContains();
    pub unsafe fn BotFindMatch();
    pub unsafe fn BotMatchVariable();
    pub unsafe fn UnifyWhiteSpaces();
    pub unsafe fn BotReplaceSynonyms();
    pub unsafe fn BotLoadChatFile();
    pub unsafe fn BotSetChatGender();
    pub unsafe fn BotSetChatName();
    pub unsafe fn BotResetGoalState();
    pub unsafe fn BotResetAvoidGoals();
    pub unsafe fn BotRemoveFromAvoidGoals();
    pub unsafe fn BotPushGoal();
    pub unsafe fn BotPopGoal();
    pub unsafe fn BotEmptyGoalStack();
    pub unsafe fn BotDumpAvoidGoals();
    pub unsafe fn BotDumpGoalStack();
    pub unsafe fn BotGoalName();
    pub unsafe fn BotGetTopGoal();
    pub unsafe fn BotGetSecondGoal();
    pub unsafe fn BotChooseLTGItem();
    pub unsafe fn BotChooseNBGItem();
    pub unsafe fn BotTouchingGoal();
    pub unsafe fn BotItemGoalInVisButNotVisible();
    pub unsafe fn BotGetLevelItemGoal();
    pub unsafe fn BotGetNextCampSpotGoal();
    pub unsafe fn BotGetMapLocationGoal();
    pub unsafe fn BotAvoidGoalTime();
    pub unsafe fn BotSetAvoidGoalTime();
    pub unsafe fn BotInterbreedGoalFuzzyLogic();
    pub unsafe fn BotSaveGoalFuzzyLogic();
    pub unsafe fn BotMutateGoalFuzzyLogic();
    pub unsafe fn BotAllocGoalState();
    pub unsafe fn BotFreeGoalState();
    pub unsafe fn BotResetMoveState();
    pub unsafe fn BotMoveToGoal();
    pub unsafe fn BotMoveInDirection();
    pub unsafe fn BotResetAvoidReach();
    pub unsafe fn BotResetLastAvoidReach();
    pub unsafe fn BotReachabilityArea();
    pub unsafe fn BotMovementViewTarget();
    pub unsafe fn BotPredictVisiblePosition();
    pub unsafe fn BotAllocMoveState();
    pub unsafe fn BotFreeMoveState();
    pub unsafe fn BotInitMoveState();
    pub unsafe fn BotAddAvoidSpot();
    pub unsafe fn BotChooseBestFightWeapon();
    pub unsafe fn BotGetWeaponInfo();
    pub unsafe fn BotLoadWeaponWeights();
    pub unsafe fn BotAllocWeaponState();
    pub unsafe fn BotFreeWeaponState();
    pub unsafe fn BotResetWeaponState();
    pub unsafe fn GeneticParentsAndChildSelection();
    pub unsafe fn PC_AddGlobalDefine();
    pub unsafe fn PC_LoadSourceHandle();
    pub unsafe fn PC_FreeSourceHandle();
    pub unsafe fn PC_ReadTokenHandle();
    pub unsafe fn PC_SourceFileAndLine();
    pub unsafe fn PC_LoadGlobalDefines();
}


/*
============
Init_EA_Export
============
*/
unsafe fn Init_EA_Export(ea: *mut ea_export_t) {
    //ClientCommand elementary actions
    (*ea).EA_Command = EA_Command;
    (*ea).EA_Say = EA_Say;
    (*ea).EA_SayTeam = EA_SayTeam;

    (*ea).EA_Action = EA_Action;
    (*ea).EA_Gesture = EA_Gesture;
    (*ea).EA_Talk = EA_Talk;
    (*ea).EA_Attack = EA_Attack;
    (*ea).EA_Alt_Attack = EA_Alt_Attack;
    (*ea).EA_ForcePower = EA_ForcePower;
    (*ea).EA_Use = EA_Use;
    (*ea).EA_Respawn = EA_Respawn;
    (*ea).EA_Crouch = EA_Crouch;
    (*ea).EA_MoveUp = EA_MoveUp;
    (*ea).EA_MoveDown = EA_MoveDown;
    (*ea).EA_MoveForward = EA_MoveForward;
    (*ea).EA_MoveBack = EA_MoveBack;
    (*ea).EA_MoveLeft = EA_MoveLeft;
    (*ea).EA_MoveRight = EA_MoveRight;

    (*ea).EA_SelectWeapon = EA_SelectWeapon;
    (*ea).EA_Jump = EA_Jump;
    (*ea).EA_DelayedJump = EA_DelayedJump;
    (*ea).EA_Move = EA_Move;
    (*ea).EA_View = EA_View;
    (*ea).EA_GetInput = EA_GetInput;
    (*ea).EA_EndRegular = EA_EndRegular;
    (*ea).EA_ResetInput = EA_ResetInput;
}


/*
============
Init_AI_Export
============
*/
unsafe fn Init_AI_Export(ai: *mut ai_export_t) {
    //-----------------------------------
    // be_ai_char.h
    //-----------------------------------
    (*ai).BotLoadCharacter = BotLoadCharacter;
    (*ai).BotFreeCharacter = BotFreeCharacter;
    (*ai).Characteristic_Float = Characteristic_Float;
    (*ai).Characteristic_BFloat = Characteristic_BFloat;
    (*ai).Characteristic_Integer = Characteristic_Integer;
    (*ai).Characteristic_BInteger = Characteristic_BInteger;
    (*ai).Characteristic_String = Characteristic_String;
    //-----------------------------------
    // be_ai_chat.h
    //-----------------------------------
    (*ai).BotAllocChatState = BotAllocChatState;
    (*ai).BotFreeChatState = BotFreeChatState;
    (*ai).BotQueueConsoleMessage = BotQueueConsoleMessage;
    (*ai).BotRemoveConsoleMessage = BotRemoveConsoleMessage;
    (*ai).BotNextConsoleMessage = BotNextConsoleMessage;
    (*ai).BotNumConsoleMessages = BotNumConsoleMessages;
    (*ai).BotInitialChat = BotInitialChat;
    (*ai).BotNumInitialChats = BotNumInitialChats;
    (*ai).BotReplyChat = BotReplyChat;
    (*ai).BotChatLength = BotChatLength;
    (*ai).BotEnterChat = BotEnterChat;
    (*ai).BotGetChatMessage = BotGetChatMessage;
    (*ai).StringContains = StringContains;
    (*ai).BotFindMatch = BotFindMatch;
    (*ai).BotMatchVariable = BotMatchVariable;
    (*ai).UnifyWhiteSpaces = UnifyWhiteSpaces;
    (*ai).BotReplaceSynonyms = BotReplaceSynonyms;
    (*ai).BotLoadChatFile = BotLoadChatFile;
    (*ai).BotSetChatGender = BotSetChatGender;
    (*ai).BotSetChatName = BotSetChatName;
    //-----------------------------------
    // be_ai_goal.h
    //-----------------------------------
    (*ai).BotResetGoalState = BotResetGoalState;
    (*ai).BotResetAvoidGoals = BotResetAvoidGoals;
    (*ai).BotRemoveFromAvoidGoals = BotRemoveFromAvoidGoals;
    (*ai).BotPushGoal = BotPushGoal;
    (*ai).BotPopGoal = BotPopGoal;
    (*ai).BotEmptyGoalStack = BotEmptyGoalStack;
    (*ai).BotDumpAvoidGoals = BotDumpAvoidGoals;
    (*ai).BotDumpGoalStack = BotDumpGoalStack;
    (*ai).BotGoalName = BotGoalName;
    (*ai).BotGetTopGoal = BotGetTopGoal;
    (*ai).BotGetSecondGoal = BotGetSecondGoal;
    (*ai).BotChooseLTGItem = BotChooseLTGItem;
    (*ai).BotChooseNBGItem = BotChooseNBGItem;
    (*ai).BotTouchingGoal = BotTouchingGoal;
    (*ai).BotItemGoalInVisButNotVisible = BotItemGoalInVisButNotVisible;
    (*ai).BotGetLevelItemGoal = BotGetLevelItemGoal;
    (*ai).BotGetNextCampSpotGoal = BotGetNextCampSpotGoal;
    (*ai).BotGetMapLocationGoal = BotGetMapLocationGoal;
    (*ai).BotAvoidGoalTime = BotAvoidGoalTime;
    (*ai).BotSetAvoidGoalTime = BotSetAvoidGoalTime;
    (*ai).BotInitLevelItems = BotInitLevelItems;
    (*ai).BotUpdateEntityItems = BotUpdateEntityItems;
    (*ai).BotLoadItemWeights = BotLoadItemWeights;
    (*ai).BotFreeItemWeights = BotFreeItemWeights;
    (*ai).BotInterbreedGoalFuzzyLogic = BotInterbreedGoalFuzzyLogic;
    (*ai).BotSaveGoalFuzzyLogic = BotSaveGoalFuzzyLogic;
    (*ai).BotMutateGoalFuzzyLogic = BotMutateGoalFuzzyLogic;
    (*ai).BotAllocGoalState = BotAllocGoalState;
    (*ai).BotFreeGoalState = BotFreeGoalState;
    //-----------------------------------
    // be_ai_move.h
    //-----------------------------------
    (*ai).BotResetMoveState = BotResetMoveState;
    (*ai).BotMoveToGoal = BotMoveToGoal;
    (*ai).BotMoveInDirection = BotMoveInDirection;
    (*ai).BotResetAvoidReach = BotResetAvoidReach;
    (*ai).BotResetLastAvoidReach = BotResetLastAvoidReach;
    (*ai).BotReachabilityArea = BotReachabilityArea;
    (*ai).BotMovementViewTarget = BotMovementViewTarget;
    (*ai).BotPredictVisiblePosition = BotPredictVisiblePosition;
    (*ai).BotAllocMoveState = BotAllocMoveState;
    (*ai).BotFreeMoveState = BotFreeMoveState;
    (*ai).BotInitMoveState = BotInitMoveState;
    (*ai).BotAddAvoidSpot = BotAddAvoidSpot;
    //-----------------------------------
    // be_ai_weap.h
    //-----------------------------------
    (*ai).BotChooseBestFightWeapon = BotChooseBestFightWeapon;
    (*ai).BotGetWeaponInfo = BotGetWeaponInfo;
    (*ai).BotLoadWeaponWeights = BotLoadWeaponWeights;
    (*ai).BotAllocWeaponState = BotAllocWeaponState;
    (*ai).BotFreeWeaponState = BotFreeWeaponState;
    (*ai).BotResetWeaponState = BotResetWeaponState;
    //-----------------------------------
    // be_ai_gen.h
    //-----------------------------------
    (*ai).GeneticParentsAndChildSelection = GeneticParentsAndChildSelection;
}


/*
============
GetBotLibAPI
============
*/
pub unsafe extern "C" fn GetBotLibAPI(apiVersion: c_int, import: *mut botlib_import_t) -> *mut botlib_export_t {
    if import.is_null() {
        return null_mut();   // bk001129 - this wasn't set for base/
    }
    botimport = *import;
    if botimport.Print as *const c_void == null_mut() {
        return null_mut();   // bk001129 - pars pro toto
    }

    Com_Memset(
        addr_of_mut!(be_botlib_export) as *mut c_void,
        0,
        std::mem::size_of::<botlib_export_t>(),
    );

    if apiVersion != BOTLIB_API_VERSION {
        (botimport.Print)(
            PRT_ERROR,
            "Mismatched BOTLIB_API_VERSION: expected %i, got %i\n\0".as_ptr() as *const c_char,
            BOTLIB_API_VERSION,
            apiVersion,
        );
        return null_mut();
    }

    Init_AAS_Export(addr_of_mut!(be_botlib_export.aas));
    Init_EA_Export(addr_of_mut!(be_botlib_export.ea));
    Init_AI_Export(addr_of_mut!(be_botlib_export.ai));

    be_botlib_export.BotLibSetup = Export_BotLibSetup;
    be_botlib_export.BotLibShutdown = Export_BotLibShutdown;
    be_botlib_export.BotLibVarSet = Export_BotLibVarSet;
    be_botlib_export.BotLibVarGet = Export_BotLibVarGet;

    be_botlib_export.PC_AddGlobalDefine = PC_AddGlobalDefine;
    be_botlib_export.PC_LoadSourceHandle = PC_LoadSourceHandle;
    be_botlib_export.PC_FreeSourceHandle = PC_FreeSourceHandle;
    be_botlib_export.PC_ReadTokenHandle = PC_ReadTokenHandle;
    be_botlib_export.PC_SourceFileAndLine = PC_SourceFileAndLine;
    be_botlib_export.PC_LoadGlobalDefines = PC_LoadGlobalDefines;
    be_botlib_export.PC_RemoveAllGlobalDefines = PC_RemoveAllGlobalDefines;

    be_botlib_export.BotLibStartFrame = Export_BotLibStartFrame;
    be_botlib_export.BotLibLoadMap = Export_BotLibLoadMap;
    be_botlib_export.BotLibUpdateEntity = Export_BotLibUpdateEntity;
    be_botlib_export.Test = BotExportTest;

    addr_of_mut!(be_botlib_export)
}

// Constants
const qfalse: c_int = 0;
const qtrue: c_int = 1;
const BLERR_NOERROR: c_int = 0;
const BLERR_LIBRARYNOTSETUP: c_int = 1;
const BLERR_INVALIDENTITYNUMBER: c_int = 2;
const PRT_MESSAGE: c_int = 1;
const PRT_ERROR: c_int = 2;
const BOTLIB_API_VERSION: c_int = 2;
const MAX_AVOIDREACH: c_int = 32;
const TFL_DEFAULT: c_int = 1;
const TFL_ROCKETJUMP: c_int = 1024;
const TFL_FUNCBOB: c_int = 128;
const AREACONTENTS_WATER: c_int = 1;
const AREACONTENTS_LAVA: c_int = 2;
const AREACONTENTS_SLIME: c_int = 4;
const AREACONTENTS_JUMPPAD: c_int = 8;
const AREACONTENTS_CLUSTERPORTAL: c_int = 16;
const AREACONTENTS_VIEWPORTAL: c_int = 32;
const AREACONTENTS_DONOTENTER: c_int = 64;
const AREACONTENTS_MOVER: c_int = 128;
