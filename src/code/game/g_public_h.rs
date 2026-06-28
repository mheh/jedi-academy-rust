// g_public.h -- game module information visible to server

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void, c_float};

// Type stubs for external types defined in other modules
pub type entityState_t = c_void;
pub type playerState_s = c_void;
pub type trace_t = c_void;
pub type vec3_t = [c_float; 3];
pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type fileHandle_t = c_int;
pub type fsMode_t = c_int;
pub type memtag_t = c_int;
pub type usercmd_t = c_void;
pub type cvar_t = c_void;
pub type EG2_Collision = c_int;
pub type mdxaBone_t = c_void;
pub type Eorientations = c_int;
pub type CGhoul2Info = c_void;
pub type CGhoul2Info_v = c_void;
pub type CCollisionRecord = c_void;
pub type CMiniHeap = c_void;
pub type IGhoul2InfoArray = c_void;
pub type CRagDollParams = c_void;
pub type CRagDollUpdateParams = c_void;
pub type SSkinGoreData = c_void;
pub type sharedSetBoneIKStateParams_t = c_void;
pub type sharedIKMoveParams_t = c_void;

pub const GAME_API_VERSION: c_int = 8;

// entity->svFlags
// the server does not know how to interpret most of the values
// in entityStates (level eType), so the game must explicitly flag
// special server behaviors
pub const SVF_NOCLIENT: c_int = 0x00000001;	// don't send entity to clients, even if it has effects
pub const SVF_INACTIVE: c_int = 0x00000002;	// Can't be used when this flag is on
pub const SVF_NPC: c_int = 0x00000004;
pub const SVF_BOT: c_int = 0x00000008;
pub const SVF_PLAYER_USABLE: c_int = 0x00000010;	// player can use this with the use button
pub const SVF_BROADCAST: c_int = 0x00000020;	// send to all connected clients
pub const SVF_PORTAL: c_int = 0x00000040;	// merge a second pvs at origin2 into snapshots
pub const SVF_USE_CURRENT_ORIGIN: c_int = 0x00000080;	// entity->currentOrigin instead of entity->s.origin
													// for link position (missiles and movers)
pub const SVF_TRIMODEL: c_int = 0x00000100;	// Use a three piece model make up like a player does
pub const SVF_OBJECTIVE: c_int = 0x00000200;	// Draw it's name if crosshair comes across it
pub const SVF_ANIMATING: c_int = 0x00000400;	// Currently animating from startFrame to endFrame
pub const SVF_NPC_PRECACHE: c_int = 0x00000800;	// Tell cgame to precache this spawner's NPC stuff
pub const SVF_KILLED_SELF: c_int = 0x00001000;	// ent killed itself in a script, so don't do ICARUS_FreeEnt on it... or else!
pub const SVF_NAVGOAL: c_int = 0x00002000;	// Navgoal
pub const SVF_NOPUSH: c_int = 0x00004000;	// Can't be pushed around
pub const SVF_ICARUS_FREEZE: c_int = 0x00008000;	// NPCs are frozen, ents don't execute ICARUS commands
pub const SVF_PLAT: c_int = 0x00010000;	// A func_plat or door acting like one
pub const SVF_BBRUSH: c_int = 0x00020000;	// breakable brush
pub const SVF_LOCKEDENEMY: c_int = 0x00040000;	// keep current enemy until dead
pub const SVF_IGNORE_ENEMIES: c_int = 0x00080000;	// Ignore all enemies
pub const SVF_BEAMING: c_int = 0x00100000;	// Being transported
pub const SVF_PLAYING_SOUND: c_int = 0x00200000;	// In the middle of playing a sound
pub const SVF_CUSTOM_GRAVITY: c_int = 0x00400000;	// Use personal gravity
pub const SVF_BROKEN: c_int = 0x00800000;	// A broken misc_model_breakable
pub const SVF_NO_TELEPORT: c_int = 0x01000000;	// Will not be teleported
pub const SVF_NONNPC_ENEMY: c_int = 0x02000000;	// Not a client/NPC, but can still be considered a hostile lifeform
pub const SVF_SELF_ANIMATING: c_int = 0x04000000;	// Ent will control it's animation itself in it's think func
pub const SVF_GLASS_BRUSH: c_int = 0x08000000;	// Ent is a glass brush
pub const SVF_NO_BASIC_SOUNDS: c_int = 0x10000000;	// Don't load basic custom sound set
pub const SVF_NO_COMBAT_SOUNDS: c_int = 0x20000000;	// Don't load combat custom sound set
pub const SVF_NO_EXTRA_SOUNDS: c_int = 0x40000000;	// Don't load extra custom sound set
pub const SVF_MOVER_ADJ_AREA_PORTALS: c_int = 0x80000000;	// For scripted movers only- must *explicitly instruct* them to affect area portals
//===============================================================

//rww - RAGDOLL_BEGIN
// class CRagDollUpdateParams;
// class CRagDollParams;
//rww - RAGDOLL_END

// typedef struct gentity_s gentity_t;
// typedef struct gclient_s gclient_t;

#[repr(C)]
pub enum SavedGameJustLoaded_e {
	eNO = 0,
	eFULL = 1,
	eAUTO = 2,
}

// Forward declaration stubs - the full definitions are in other modules
pub type gentity_t = gentity_s;

#[repr(C)]
pub struct gentity_s {
	pub s: entityState_t,				// communicated by server to clients
	pub client: *mut playerState_s,
	pub inuse: qboolean,
	pub linked: qboolean,				// qfalse if not in any good cluster

	pub svFlags: c_int,			// SVF_NOCLIENT, SVF_BROADCAST, etc

	pub bmodel: qboolean,				// if false, assume an explicit mins / maxs bounding box
									// only set by gi.SetBrushModel
	pub mins: vec3_t,
	pub maxs: vec3_t,
	pub contents: c_int,			// CONTENTS_TRIGGER, CONTENTS_SOLID, CONTENTS_BODY, etc
									// a non-solid entity should set to 0

	pub absmin: vec3_t,
	pub absmax: vec3_t,		// derived from mins/maxs and origin + rotation

	// currentOrigin will be used for all collision detection and world linking.
	// it will not necessarily be the same as the trajectory evaluation for the current
	// time, because each entity must be moved one at a time after time is advanced
	// to avoid simultanious collision issues
	pub currentOrigin: vec3_t,
	pub currentAngles: vec3_t,

	pub owner: *mut gentity_s,				// objects never interact with their owners, to
									// prevent player missiles from immediately
									// colliding with their owner
	/*
	Ghoul2 Insert Start
	*/
	// this marker thing of Jake's is used for memcpy() length calcs, so don't put any ordinary fields (like above)
	//	below this point or they won't work, and will mess up all sorts of stuff.
	//
	pub ghoul2: CGhoul2Info_v,
	/*
	Ghoul2 Insert End
	*/
	// the game dll can add anything it wants after
	// this point in the structure
}

//===============================================================

// #if defined(_XBOX) && !defined(_TRACE_FUNCTOR_T_DEFINED_)
// Function objects to replace the function pointers used for trace
// We can't have default arguments on function pointers, but this allows us to
// do the same thing with minimal impact elsewhere.
#[repr(C)]
pub struct Trace_Functor_t {
	pub trace_func: Option<unsafe extern "C" fn(
		*mut trace_t,
		*const vec3_t,
		*const vec3_t,
		*const vec3_t,
		*const vec3_t,
		c_int,
		c_int,
		EG2_Collision,
		c_int,
	)>,
}

// Always create this class exactly once
// #define _TRACE_FUNCTOR_T_DEFINED_
// #endif

//
// functions provided by the main engine
//
/*
Ghoul2 Insert Start
*/
// class CMiniHeap;
/*
Ghoul2 Insert End
*/
#[repr(C)]
pub struct game_import_t {
	//============== general Quake services ==================

	// print message on the local console
	pub Printf: Option<unsafe extern "C" fn(*const c_char, ...)>,

	// Write a camera ref_tag to cameras.map
	pub WriteCam: Option<unsafe extern "C" fn(*const c_char)>,
	pub FlushCamFile: Option<unsafe extern "C" fn()>,

	// abort the game
	pub Error: Option<unsafe extern "C" fn(c_int, *const c_char, ...)>,

	// get current time for profiling reasons
	// this should NOT be used for any game related tasks,
	// because it is not journaled
	pub Milliseconds: Option<unsafe extern "C" fn() -> c_int>,

	// console variable interaction
	pub cvar: Option<unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> *mut cvar_t>,
	pub cvar_set: Option<unsafe extern "C" fn(*const c_char, *const c_char)>,
	pub Cvar_VariableIntegerValue: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
	pub Cvar_VariableStringBuffer: Option<unsafe extern "C" fn(*const c_char, *mut c_char, c_int)>,

	// ClientCommand and ServerCommand parameter access
	pub argc: Option<unsafe extern "C" fn() -> c_int>,
	pub argv: Option<unsafe extern "C" fn(c_int) -> *mut c_char>,

	pub FS_FOpenFile: Option<unsafe extern "C" fn(*const c_char, *mut fileHandle_t, fsMode_t) -> c_int>,
	pub FS_Read: Option<unsafe extern "C" fn(*mut c_void, c_int, fileHandle_t) -> c_int>,
	pub FS_Write: Option<unsafe extern "C" fn(*const c_void, c_int, fileHandle_t) -> c_int>,
	pub FS_FCloseFile: Option<unsafe extern "C" fn(fileHandle_t)>,
	pub FS_ReadFile: Option<unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> c_int>,
	pub FS_FreeFile: Option<unsafe extern "C" fn(*mut c_void)>,
	pub FS_GetFileList: Option<unsafe extern "C" fn(*const c_char, *const c_char, *mut c_char, c_int) -> c_int>,

	// Savegame handling
	//
	pub AppendToSaveGame: Option<unsafe extern "C" fn(c_ulong, *const c_void, c_int) -> qboolean>,
	// #ifdef _XBOX	// No default arguments through function pointers
	// 	int			ReadFromSaveGame(unsigned long chid, void *pvAddress, int iLength, void **ppvAddressPtr = NULL)
	// 	{
	// 		return SG_Read(chid, pvAddress, iLength, ppvAddressPtr);
	// 	}
	//
	// 	int			ReadFromSaveGameOptional(unsigned long chid, void *pvAddress, int iLength, void **ppvAddressPtr = NULL)
	// 	{
	// 		return SG_ReadOptional(chid, pvAddress, iLength, ppvAddressPtr);
	// 	}
	// #else
	pub ReadFromSaveGame: Option<unsafe extern "C" fn(c_ulong, *mut c_void, c_int, *mut *mut c_void) -> c_int>,
	pub ReadFromSaveGameOptional: Option<unsafe extern "C" fn(c_ulong, *mut c_void, c_int, *mut *mut c_void) -> c_int>,
	// #endif
	// add commands to the console as if they were typed in
	// for map changing, etc
	pub SendConsoleCommand: Option<unsafe extern "C" fn(*const c_char)>,


	//=========== server specific functionality =============

	// kick a client off the server with a message
	pub DropClient: Option<unsafe extern "C" fn(c_int, *const c_char)>,

	// reliably sends a command string to be interpreted by the given
	// client.  If clientNum is -1, it will be sent to all clients
	pub SendServerCommand: Option<unsafe extern "C" fn(c_int, *const c_char, ...)>,

	// config strings hold all the index strings, and various other information
	// that is reliably communicated to all clients
	// All of the current configstrings are sent to clients when
	// they connect, and changes are sent to all connected clients.
	// All confgstrings are cleared at each level start.
	pub SetConfigstring: Option<unsafe extern "C" fn(c_int, *const c_char)>,
	pub GetConfigstring: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,

	// userinfo strings are maintained by the server system, so they
	// are persistant across level loads, while all other game visible
	// data is completely reset
	pub GetUserinfo: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
	pub SetUserinfo: Option<unsafe extern "C" fn(c_int, *const c_char)>,

	// the serverinfo info string has all the cvars visible to server browsers
	pub GetServerinfo: Option<unsafe extern "C" fn(*mut c_char, c_int)>,

	// sets mins and maxs based on the brushmodel name
	pub SetBrushModel: Option<unsafe extern "C" fn(*mut gentity_t, *const c_char)>,

	// collision detection against all linked entities
	// #ifdef _XBOX
	// 	Trace_Functor_t trace;
	// #else
	pub trace: Option<unsafe extern "C" fn(
		*mut trace_t,
		*const vec3_t,
		*const vec3_t,
		*const vec3_t,
		*const vec3_t,
		c_int,
		c_int,
		EG2_Collision,
		c_int,
	)>,
	// #endif

	// point contents against all linked entities
	pub pointcontents: Option<unsafe extern "C" fn(*const vec3_t, c_int) -> c_int>,
	// what contents are on the map?
	pub totalMapContents: Option<unsafe extern "C" fn() -> c_int>,

	pub inPVS: Option<unsafe extern "C" fn(*const vec3_t, *const vec3_t) -> qboolean>,
	pub inPVSIgnorePortals: Option<unsafe extern "C" fn(*const vec3_t, *const vec3_t) -> qboolean>,
	pub AdjustAreaPortalState: Option<unsafe extern "C" fn(*mut gentity_t, qboolean)>,
	pub AreasConnected: Option<unsafe extern "C" fn(c_int, c_int) -> qboolean>,

	// an entity will never be sent to a client or used for collision
	// if it is not passed to linkentity.  If the size, position, or
	// solidity changes, it must be relinked.
	pub linkentity: Option<unsafe extern "C" fn(*mut gentity_t)>,
	pub unlinkentity: Option<unsafe extern "C" fn(*mut gentity_t)>,		// call before removing an interactive entity

	// EntitiesInBox will return brush models based on their bounding box,
	// so exact determination must still be done with EntityContact
	pub EntitiesInBox: Option<unsafe extern "C" fn(*const vec3_t, *const vec3_t, *mut *mut gentity_t, c_int) -> c_int>,

	// perform an exact check against inline brush models of non-square shape
	pub EntityContact: Option<unsafe extern "C" fn(*const vec3_t, *const vec3_t, *const gentity_t) -> qboolean>,

	// sound volume values
	pub VoiceVolume: *mut c_int,

	// dynamic memory allocator for things that need to be freed
	pub Malloc: Option<unsafe extern "C" fn(c_int, memtag_t, qboolean) -> *mut c_void>,	// see qcommon/tags.h for choices
	pub Free: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
	pub bIsFromZone: Option<unsafe extern "C" fn(*mut c_void, memtag_t) -> qboolean>,	// see qcommon/tags.h for choices

	/*
	Ghoul2 Insert Start
	*/
	pub G2API_PrecacheGhoul2Model: Option<unsafe extern "C" fn(*const c_char) -> qhandle_t>,

	// #ifdef _XBOX	// No default arguments through function pointers
	// 	int			G2API_InitGhoul2Model(CGhoul2Info_v &ghoul2, const char *fileName, int modelIndex, qhandle_t customSkin = NULL,
	// 								  qhandle_t customShader = NULL, int modelFlags = 0, int lodBias = 0)
	// 	{
	// 		return ::G2API_InitGhoul2Model(ghoul2, fileName, modelIndex, customSkin, customShader, modelFlags, lodBias);
	// 	}
	//
	// 	qboolean	G2API_SetSkin(CGhoul2Info *ghlInfo, qhandle_t customSkin, qhandle_t renderSkin = 0 )
	// 	{
	// 		return ::G2API_SetSkin(ghlInfo, customSkin, renderSkin);
	// 	}
	//
	// 	qboolean	G2API_SetBoneAnim(CGhoul2Info *ghlInfo, const char *boneName, const int startFrame, const int endFrame,
	// 							  const int flags, const float animSpeed, const int currentTime, const float setFrame = -1, const int blendTime = -1)
	// 	{
	// 		return ::G2API_SetBoneAnim(ghlInfo, boneName, startFrame, endFrame, flags, animSpeed, currentTime, setFrame, blendTime);
	// 	}
	//
	// 	qboolean	G2API_SetBoneAngles(CGhoul2Info *ghlInfo, const char *boneName, const vec3_t angles,
	// 								   const int flags, const Eorientations up, const Eorientations right, const Eorientations forward,
	// 								   qhandle_t *modelList, int blendTime = 0, int blendStart = 0)
	// 	{
	// 		return ::G2API_SetBoneAngles(ghlInfo, boneName, angles, flags, up, right, forward, modelList, blendTime, blendStart);
	// 	}
	//
	// 	qboolean	G2API_SetBoneAnglesMatrix(CGhoul2Info *ghlInfo, const char *boneName, const mdxaBone_t &matrix, const int flags,
	// 									  qhandle_t *modelList, int blendTime = 0, int currentTime = 0)
	// 	{
	// 		return ::G2API_SetBoneAnglesMatrix(ghlInfo, boneName, matrix, flags, modelList, blendTime, currentTime);
	// 	}
	//
	// 	void		G2API_CopyGhoul2Instance(CGhoul2Info_v &ghoul2From, CGhoul2Info_v &ghoul2To, int modelIndex = -1)
	// 	{
	// 		::G2API_CopyGhoul2Instance(ghoul2From, ghoul2To, modelIndex);
	// 	}
	//
	// 	qboolean	G2API_SetBoneAnglesIndex(CGhoul2Info *ghlInfo, const int index, const vec3_t angles, const int flags,
	// 							 const Eorientations yaw, const Eorientations pitch, const Eorientations roll,
	// 							 qhandle_t *modelList, int blendTime = 0, int currentTime = 0)
	// 	{
	// 		return ::G2API_SetBoneAnglesIndex(ghlInfo, index, angles, flags, yaw, pitch, roll, modelList, blendTime, currentTime);
	// 	}
	//
	// 	qboolean	G2API_SetBoneAnimIndex(CGhoul2Info *ghlInfo, const int index, const int startFrame, const int endFrame, const int flags, const float animSpeed, const int currentTime, const float setFrame = -1, const int blendTime = -1)
	// 	{
	// 		return ::G2API_SetBoneAnimIndex(ghlInfo, index, startFrame, endFrame, flags, animSpeed, currentTime, setFrame, blendTime);
	// 	}
	//
	// #else

	pub G2API_InitGhoul2Model: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, c_int, qhandle_t, qhandle_t, c_int, c_int) -> c_int>,
	pub G2API_SetSkin: Option<unsafe extern "C" fn(*mut CGhoul2Info, qhandle_t, qhandle_t) -> qboolean>,
	pub G2API_SetBoneAnim: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, c_int, c_int, c_int, c_float, c_int, c_float, c_int) -> qboolean>,
	pub G2API_SetBoneAngles: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, *const vec3_t, c_int, Eorientations, Eorientations, Eorientations, *mut qhandle_t, c_int, c_int) -> qboolean>,
	pub G2API_SetBoneAnglesIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, *const vec3_t, c_int, Eorientations, Eorientations, Eorientations, *mut qhandle_t, c_int, c_int) -> qboolean>,
	pub G2API_SetBoneAnglesMatrix: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, *const mdxaBone_t, c_int, *mut qhandle_t, c_int, c_int) -> qboolean>,
	pub G2API_CopyGhoul2Instance: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *mut CGhoul2Info_v, c_int)>,
	pub G2API_SetBoneAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, c_int, c_int, c_int, c_float, c_int, c_float, c_int) -> qboolean>,
	// #endif

	pub G2API_SetLodBias: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_SetShader: Option<unsafe extern "C" fn(*mut CGhoul2Info, qhandle_t) -> qboolean>,
	pub G2API_RemoveGhoul2Model: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int) -> qboolean>,
	pub G2API_SetSurfaceOnOff: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, c_int) -> qboolean>,
	pub G2API_SetRootSurface: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int, *const c_char) -> qboolean>,
	pub G2API_RemoveSurface: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_AddSurface: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, c_int, c_float, c_float, c_int) -> c_int>,
	pub G2API_GetBoneAnim: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, c_int, *mut c_float, *mut c_int, *mut c_int, *mut c_int, *mut c_float, *mut c_int) -> qboolean>,
	pub G2API_GetBoneAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, c_int, *mut c_float, *mut c_int, *mut c_int, *mut c_int, *mut c_float, *mut c_int) -> qboolean>,
	pub G2API_GetAnimRange: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, *mut c_int, *mut c_int) -> qboolean>,
	pub G2API_GetAnimRangeIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, *mut c_int, *mut c_int) -> qboolean>,

	pub G2API_PauseBoneAnim: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, c_int) -> qboolean>,
	pub G2API_PauseBoneAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, c_int) -> qboolean>,
	pub G2API_IsPaused: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> qboolean>,
	pub G2API_StopBoneAnim: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> qboolean>,
	pub G2API_StopBoneAngles: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> qboolean>,
	pub G2API_RemoveBone: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> qboolean>,
	pub G2API_RemoveBolt: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_AddBolt: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> c_int>,
	pub G2API_AddBoltSurfNum: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> c_int>,
	pub G2API_AttachG2Model: Option<unsafe extern "C" fn(*mut CGhoul2Info, *mut CGhoul2Info, c_int, c_int) -> qboolean>,
	pub G2API_DetachG2Model: Option<unsafe extern "C" fn(*mut CGhoul2Info) -> qboolean>,
	pub G2API_AttachEnt: Option<unsafe extern "C" fn(*mut c_int, *mut CGhoul2Info, c_int, c_int, c_int) -> qboolean>,
	pub G2API_DetachEnt: Option<unsafe extern "C" fn(*mut c_int)>,

	pub G2API_GetBoltMatrix: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int, c_int, *mut mdxaBone_t, *const vec3_t, *const vec3_t, c_int, *mut qhandle_t, *const vec3_t) -> qboolean>,

	pub G2API_ListSurfaces: Option<unsafe extern "C" fn(*mut CGhoul2Info)>,
	pub G2API_ListBones: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int)>,
	pub G2API_HaveWeGhoul2Models: Option<unsafe extern "C" fn(*mut CGhoul2Info_v) -> qboolean>,
	pub G2API_SetGhoul2ModelFlags: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_GetGhoul2ModelFlags: Option<unsafe extern "C" fn(*mut CGhoul2Info) -> c_int>,

	pub G2API_GetAnimFileName: Option<unsafe extern "C" fn(*mut CGhoul2Info, *mut *mut c_char) -> qboolean>,
	pub G2API_CollisionDetect: Option<unsafe extern "C" fn(*mut CCollisionRecord, *mut CGhoul2Info_v, *const vec3_t, *const vec3_t, c_int, c_int, *mut vec3_t, *mut vec3_t, *mut vec3_t, *mut CMiniHeap, EG2_Collision, c_int, c_float)>,
	pub G2API_GiveMeVectorFromMatrix: Option<unsafe extern "C" fn(*mut mdxaBone_t, Eorientations, *mut vec3_t)>,
	pub G2API_CleanGhoul2Models: Option<unsafe extern "C" fn(*mut CGhoul2Info_v)>,
	pub TheGhoul2InfoArray: Option<unsafe extern "C" fn() -> *mut IGhoul2InfoArray>,
	pub G2API_GetParentSurface: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> c_int>,
	pub G2API_GetSurfaceIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> c_int>,
	pub G2API_GetSurfaceName: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> *mut c_char>,
	pub G2API_GetGLAName: Option<unsafe extern "C" fn(*mut CGhoul2Info) -> *mut c_char>,
	pub G2API_SetNewOrigin: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_GetBoneIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char, qboolean) -> c_int>,
	pub G2API_StopBoneAnglesIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_StopBoneAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_SetBoneAnglesMatrixIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int, *const mdxaBone_t, c_int, *mut qhandle_t, c_int, c_int) -> qboolean>,
	pub G2API_SetAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info, c_int) -> qboolean>,
	pub G2API_GetAnimIndex: Option<unsafe extern "C" fn(*mut CGhoul2Info) -> c_int>,
	pub G2API_SaveGhoul2Models: Option<unsafe extern "C" fn(*mut CGhoul2Info_v)>,
	pub G2API_LoadGhoul2Models: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *mut c_char)>,
	pub G2API_LoadSaveCodeDestructGhoul2Info: Option<unsafe extern "C" fn(*mut CGhoul2Info_v)>,
	pub G2API_GetAnimFileNameIndex: Option<unsafe extern "C" fn(qhandle_t) -> *mut c_char>,
	pub G2API_GetAnimFileInternalNameIndex: Option<unsafe extern "C" fn(qhandle_t) -> *mut c_char>,
	pub G2API_GetSurfaceRenderStatus: Option<unsafe extern "C" fn(*mut CGhoul2Info, *const c_char) -> c_int>,

	//rww - RAGDOLL_BEGIN
	pub G2API_SetRagDoll: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *mut CRagDollParams)>,
	pub G2API_AnimateG2Models: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int, *mut CRagDollUpdateParams)>,

	pub G2API_RagPCJConstraint: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, *mut vec3_t, *mut vec3_t) -> qboolean>,
	pub G2API_RagPCJGradientSpeed: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, c_float) -> qboolean>,
	pub G2API_RagEffectorGoal: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, *mut vec3_t) -> qboolean>,
	pub G2API_GetRagBonePos: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, *mut vec3_t, *mut vec3_t, *mut vec3_t, *mut vec3_t) -> qboolean>,
	pub G2API_RagEffectorKick: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *const c_char, *mut vec3_t) -> qboolean>,
	pub G2API_RagForceSolve: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, qboolean) -> qboolean>,

	pub G2API_SetBoneIKState: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int, *const c_char, c_int, *mut sharedSetBoneIKStateParams_t) -> qboolean>,
	pub G2API_IKMove: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, c_int, *mut sharedIKMoveParams_t) -> qboolean>,
	//rww - RAGDOLL_END

	pub G2API_AddSkinGore: Option<unsafe extern "C" fn(*mut CGhoul2Info_v, *mut SSkinGoreData)>,
	pub G2API_ClearSkinGore: Option<unsafe extern "C" fn(*mut CGhoul2Info_v)>,

	pub RMG_Init: Option<unsafe extern "C" fn(c_int)>,
	// #ifndef _XBOX
	pub CM_RegisterTerrain: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
	// #endif
	pub SetActiveSubBSP: Option<unsafe extern "C" fn(c_int) -> *const c_char>,


	pub RE_RegisterSkin: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
	pub RE_GetAnimationCFG: Option<unsafe extern "C" fn(*const c_char, *mut c_char, c_int) -> c_int>,

	pub WE_GetWindVector: Option<unsafe extern "C" fn(*mut vec3_t, *mut vec3_t) -> bool>,
	pub WE_GetWindGusting: Option<unsafe extern "C" fn(*mut vec3_t) -> bool>,
	pub WE_IsOutside: Option<unsafe extern "C" fn(*mut vec3_t) -> bool>,
	pub WE_IsOutsideCausingPain: Option<unsafe extern "C" fn(*mut vec3_t) -> c_float>,
	pub WE_GetChanceOfSaberFizz: Option<unsafe extern "C" fn() -> c_float>,
	pub WE_IsShaking: Option<unsafe extern "C" fn(*mut vec3_t) -> bool>,
	pub WE_AddWeatherZone: Option<unsafe extern "C" fn(*mut vec3_t, *mut vec3_t)>,
	pub WE_SetTempGlobalFogColor: Option<unsafe extern "C" fn(*mut vec3_t) -> bool>,


	/*
	Ghoul2 Insert End
	*/
}

//
// functions exported by the game subsystem
//
#[repr(C)]
pub struct game_export_t {
	pub apiversion: c_int,

	// init and shutdown will be called every single level
	// levelTime will be near zero, while globalTime will be a large number
	// that can be used to track spectator entry times across restarts
	pub Init: Option<unsafe extern "C" fn(*const c_char, *const c_char, c_int, *const c_char, c_int, c_int, c_int, SavedGameJustLoaded_e, qboolean)>,
	pub Shutdown: Option<unsafe extern "C" fn()>,

	// ReadLevel is called after the default map information has been
	// loaded with SpawnEntities
	pub WriteLevel: Option<unsafe extern "C" fn(qboolean)>,
	pub ReadLevel: Option<unsafe extern "C" fn(qboolean, qboolean)>,
	pub GameAllowedToSaveHere: Option<unsafe extern "C" fn() -> qboolean>,

	// return NULL if the client is allowed to connect, otherwise return
	// a text string with the reason for denial
	pub ClientConnect: Option<unsafe extern "C" fn(c_int, qboolean, SavedGameJustLoaded_e) -> *mut c_char>,

	pub ClientBegin: Option<unsafe extern "C" fn(c_int, *mut usercmd_t, SavedGameJustLoaded_e)>,
	pub ClientUserinfoChanged: Option<unsafe extern "C" fn(c_int)>,
	pub ClientDisconnect: Option<unsafe extern "C" fn(c_int)>,
	pub ClientCommand: Option<unsafe extern "C" fn(c_int)>,
	pub ClientThink: Option<unsafe extern "C" fn(c_int, *mut usercmd_t)>,

	pub RunFrame: Option<unsafe extern "C" fn(c_int)>,
	pub ConnectNavs: Option<unsafe extern "C" fn(*const c_char, c_int)>,

	// ConsoleCommand will be called when a command has been issued
	// that is not recognized as a builtin function.
	// The game can issue gi.argc() / gi.argv() commands to get the command
	// and parameters.  Return qfalse if the game doesn't recognize it as a command.
	pub ConsoleCommand: Option<unsafe extern "C" fn() -> qboolean>,

	//void		(*PrintEntClassname)( int clientNum );
	//int			(*ValidateAnimRange)( int startFrame, int endFrame, float animSpeed );

	pub GameSpawnRMGEntity: Option<unsafe extern "C" fn(*mut c_char)>,
	//
	// global variables shared between game and server
	//

	// The gentities array is allocated in the game dll so it
	// can vary in size from one game to another.
	//
	// The size will be fixed when ge->Init() is called
	// the server can't just use pointer arithmetic on gentities, because the
	// server's sizeof(struct gentity_s) doesn't equal gentitySize
	pub gentities: *mut gentity_s,
	pub gentitySize: c_int,
	pub num_entities: c_int,		// current number, <= MAX_GENTITIES
}

extern "C" {
	pub fn GetGameApi(import: *mut game_import_t) -> *mut game_export_t;
}

//#endif//#ifndef __G_PUBLIC_H__
