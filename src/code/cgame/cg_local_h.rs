// Mirror of oracle/code/cgame/cg_local.h
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void, c_float};
use crate::code::game::q_shared_h::*;
use crate::code::game::g_shared_h::*;
use crate::code::cgame::cg_camera_h::*;

// The entire cgame module is unloaded and reloaded on each level change,
// so there is NO persistant data between levels on the client side.
// If you absolutely need something stored, it can either be kept
// by the server in the server stored userinfos, or stashed in a cvar.

pub const POWERUP_BLINKS: c_int = 5;
pub const POWERUP_BLINK_TIME: c_int = 1000;
pub const FADE_TIME: c_int = 200;
pub const PULSE_TIME: c_int = 200;
pub const DAMAGE_DEFLECT_TIME: c_int = 100;
pub const DAMAGE_RETURN_TIME: c_int = 400;
pub const DAMAGE_TIME: c_int = 500;
pub const LAND_DEFLECT_TIME: c_int = 150;
pub const LAND_RETURN_TIME: c_int = 300;
pub const STEP_TIME: c_int = 200;
pub const DUCK_TIME: c_int = 100;
pub const PAIN_TWITCH_TIME: c_int = 200;
pub const WEAPON_SELECT_TIME: c_int = 1400;
pub const ITEM_SCALEUP_TIME: c_int = 1000;
// Zoom vars
pub const ZOOM_TIME: c_int = 150;		// not currently used?
pub const MAX_ZOOM_FOV: c_float = 3.0;
pub const ZOOM_IN_TIME: c_float = 1500.0;
pub const ZOOM_OUT_TIME: c_float = 100.0;
pub const ZOOM_START_PERCENT: c_float = 0.3;

pub const ITEM_BLOB_TIME: c_int = 200;
pub const MUZZLE_FLASH_TIME: c_int = 20;
pub const FRAG_FADE_TIME: c_int = 1000;		// time for fragments to fade away

pub const PULSE_SCALE: c_float = 1.5;			// amount to scale up the icons when activating

pub const MAX_STEP_CHANGE: c_int = 32;

pub const MAX_VERTS_ON_POLY: c_int = 10;
pub const MAX_MARK_POLYS: c_int = 256;

pub const STAT_MINUS: c_int = 10;	// num frame for '-' stats digit

pub const ICON_SIZE: c_int = 48;
pub const CHAR_WIDTH: c_int = 32;
pub const CHAR_HEIGHT: c_int = 48;
pub const TEXT_ICON_SPACE: c_int = 4;

pub const CHARSMALL_WIDTH: c_int = 16;
pub const CHARSMALL_HEIGHT: c_int = 32;

// very large characters
pub const GIANT_WIDTH: c_int = 32;
pub const GIANT_HEIGHT: c_int = 48;

pub const MAX_PRINTTEXT: c_int = 128;
pub const MAX_CAPTIONTEXT: c_int = 32;	// we don't need 64 now since we don't use this for scroll text, and I needed to change a hardwired 128 to 256, so...
pub const MAX_LCARSTEXT: c_int = 128;


pub const NUM_FONT_BIG: c_int = 1;
pub const NUM_FONT_SMALL: c_int = 2;
pub const NUM_FONT_CHUNKY: c_int = 3;

pub const CS_BASIC: c_int = 0;
pub const CS_COMBAT: c_int = 1;
pub const CS_EXTRA: c_int = 2;
pub const CS_JEDI: c_int = 3;
pub const CS_TRY_ALL: c_int = 4;

pub const WAVE_AMPLITUDE: c_float = 1.0;
pub const WAVE_FREQUENCY: c_float = 0.4;

//=================================================

// player entities need to track more information
// than any other type of entity.

// note that not every player entity is a client entity,
// because corpses after respawn are outside the normal
// client numbering range

// when changing animation, set animationTime to frameTime + lerping time
// The current lerp will finish out, then it will lerp to the new animation
#[repr(C)]
pub struct lerpFrame_t {
	pub oldFrame: c_int,
	pub oldFrameTime: c_int,		// time when ->oldFrame was exactly on

	pub frame: c_int,
	pub frameTime: c_int,			// time when ->frame will be exactly on

	pub backlerp: c_float,

	pub yawAngle: c_float,
	pub yawing: qboolean,
	pub pitchAngle: c_float,
	pub pitching: qboolean,

	pub animationNumber: c_int,
	pub animation: *mut animation_t,
	pub animationTime: c_int,		// time when the first frame of the animation will be exact
}

#[repr(C)]
pub struct playerEntity_t {
	pub legs: lerpFrame_t,
	pub torso: lerpFrame_t,
	pub painTime: c_int,
	pub painDirection: c_int,	// flip from 0 to 1

	// For persistent beam weapons, so they don't play their start sound more than once
	pub lightningFiring: qboolean,

	// machinegun spinning
	//	float			barrelAngle;
	//	int				barrelTime;
	//	qboolean		barrelSpinning;
}

//=================================================

// centity_t have a direct corespondence with gentity_t in the game, but
// only the entityState_t is directly communicated to the cgame
#[repr(C)]
pub struct centity_s
{
	pub currentState: entityState_t,	// from cg.frame
	pub nextState: *const entityState_t,		// from cg.nextFrame, if available
	pub interpolate: qboolean,	// true if next is valid to interpolate to
	pub currentValid: qboolean,	// true if cg.frame holds this entity

	pub muzzleFlashTime: c_int,	// move to playerEntity?
	pub altFire: qboolean,			// move to playerEntity?

	pub previousEvent: c_int,
	//	int				teleportFlag;

	//	int				trailTime;		// so missile trails can handle dropped initial packets
	pub miscTime: c_int,

	pub pe: playerEntity_t,

	//	int				errorTime;		// decay the error from this time
	//	vec3_t			errorOrigin;
	//	vec3_t			errorAngles;

	//	qboolean		extrapolated;	// false if origin / angles is an interpolation
	//	vec3_t			rawOrigin;
	//	vec3_t			rawAngles;

	//	vec3_t			beamEnd;

	// exact interpolated position of entity on this frame
	pub lerpOrigin: vec3_t,
	pub lerpAngles: vec3_t,
	pub renderAngles: vec3_t,	//for ET_PLAYERS, the actual angles it was rendered at- should be used by any getboltmatrix calls after CG_Player

	pub rotValue: c_float, //rotation increment for repeater effect

	pub snapShotTime: c_int,

	//Pointer to corresponding gentity
	pub gent: *mut gentity_t,
}

pub type centity_t = centity_s;


//======================================================================

// local entities are created as a result of events or predicted actions,
// and live independently from all server transmitted entities

#[repr(C)]
pub struct markPoly_s {
	pub prevMark: *mut markPoly_s,
	pub nextMark: *mut markPoly_s,
	pub time: c_int,
	pub markShader: qhandle_t,
	pub alphaFade: qboolean,		// fade alpha instead of rgb
	pub color: [c_float; 4],
	pub poly: poly_t,
	pub verts: [polyVert_t; MAX_VERTS_ON_POLY as usize],
}

pub type markPoly_t = markPoly_s;


#[repr(C)]
#[repr(C)]
pub enum leType_t {
	LE_MARK,
	LE_FADE_MODEL,
	LE_FADE_SCALE_MODEL, // currently only for Demp2 shock sphere
	LE_FRAGMENT,
	LE_PUFF,
	LE_FADE_RGB,
	LE_LIGHT,
	LE_LINE,
	LE_QUAD,
	LE_SPRITE,
}

#[repr(C)]
pub enum leFlag_t {
	LEF_PUFF_DONT_SCALE = 0x0001,			// do not scale size over time
	LEF_TUMBLE			= 0x0002,			// tumble over time, used for ejecting shells
	LEF_FADE_RGB		= 0x0004,			// explicitly fade
	LEF_NO_RANDOM_ROTATE = 0x0008			// MakeExplosion adds random rotate which could be bad in some cases
}

#[repr(C)]
pub enum leBounceSound_t {
	LEBS_NONE,
	LEBS_METAL,
	LEBS_ROCK
} // fragment local entities can make sounds on impacts

#[repr(C)]
pub struct localEntity_s {
	pub prev: *mut localEntity_s,
	pub next: *mut localEntity_s,
	pub leType: leType_t,
	pub leFlags: c_int,

	pub startTime: c_int,
	pub endTime: c_int,

	pub lifeRate: c_float,			// 1.0 / (endTime - startTime)

	pub pos: trajectory_t,
	pub angles: trajectory_t,

	pub bounceFactor: c_float,		// 0.0 = no bounce, 1.0 = perfect

	pub color: [c_float; 4],

	pub radius: c_float,

	pub light: c_float,
	pub lightColor: vec3_t,

	pub leBounceSoundType: leBounceSound_t,

	pub refEntity: refEntity_t,
	pub ownerGentNum: c_int,
}

pub type localEntity_t = localEntity_s;

//======================================================================


// each IT_* item has an associated itemInfo_t
// that constains media references necessary to present the
// item and its effects
#[repr(C)]
pub struct itemInfo_t {
	pub registered: qboolean,
	pub models: qhandle_t,
	pub icon: qhandle_t,
}


#[repr(C)]
pub struct powerupInfo_t {
	pub itemNum: c_int,
}


pub const CG_OVERRIDE_3RD_PERSON_ENT: c_int = 0x00000001;
pub const CG_OVERRIDE_3RD_PERSON_RNG: c_int = 0x00000002;
pub const CG_OVERRIDE_3RD_PERSON_ANG: c_int = 0x00000004;
pub const CG_OVERRIDE_3RD_PERSON_VOF: c_int = 0x00000008;
pub const CG_OVERRIDE_3RD_PERSON_POF: c_int = 0x00000010;
pub const CG_OVERRIDE_3RD_PERSON_CDP: c_int = 0x00000020;
pub const CG_OVERRIDE_3RD_PERSON_APH: c_int = 0x00000040;
pub const CG_OVERRIDE_FOV: c_int = 0x00000080;

#[repr(C)]
pub struct overrides_t {
	//NOTE: these probably get cleared in save/load!!!
	pub active: c_int,	//bit-flag field of which overrides are active
	pub thirdPersonEntity: c_int,	//who to center on
	pub thirdPersonRange: c_float,	//how far to be from them
	pub thirdPersonAngle: c_float,	//what angle to look at them from
	pub thirdPersonVertOffset: c_float,	//how high to be above them
	pub thirdPersonPitchOffset: c_float,	//what offset pitch to apply the the camera view
	pub thirdPersonCameraDamp: c_float,	//how tightly to move the camera pos behind the player
	pub thirdPersonAlpha: c_float,	//how tightly to move the camera pos behind the player
	pub fov: c_float,				//what fov to use
	//NOTE: could put Alpha and HorzOffset and the target & camera damps, but no-one is trying to override those, so...
}

//======================================================================


// all cg.stepTime, cg.duckTime, cg.landTime, etc are set to cg.time when the action
// occurs, and they will have visible effects for #define STEP_TIME or whatever msec after

#[repr(C)]
pub struct cg_t {
	pub clientFrame: c_int,		// incremented each frame

	pub levelShot: qboolean,			// taking a level menu screenshot

	// there are only one or two snapshot_t that are relevent at a time
	pub latestSnapshotNum: c_int,	// the number of snapshots the client system has received
	pub latestSnapshotTime: c_int,	// the time from latestSnapshotNum, so we don't need to read the snapshot yet
	pub processedSnapshotNum: c_int,// the number of snapshots cgame has requested
	pub snap: *mut snapshot_t,				// cg.snap->serverTime <= cg.time
	pub nextSnap: *mut snapshot_t,			// cg.nextSnap->serverTime > cg.time, or NULL

	pub frameInterpolation: c_float,	// (float)( cg.time - cg.frame->serverTime ) / (cg.nextFrame->serverTime - cg.frame->serverTime)

	pub thisFrameTeleport: qboolean,
	pub nextFrameTeleport: qboolean,

	pub frametime: c_int,		// cg.time - cg.oldTime

	pub time: c_int,			// this is the time value that the client
								// is rendering at.
	pub oldTime: c_int,		// time at last frame, used for missile trails and prediction checking

	pub timelimitWarnings: c_int,	// 5 min, 1 min, overtime

	pub renderingThirdPerson: qboolean,		// during deaths, chasecams, etc

	// prediction state
	pub hyperspace: qboolean,				// true if prediction has hit a trigger_teleport
	pub predicted_player_state: playerState_t,
	pub validPPS: qboolean,				// clear until the first call to CG_PredictPlayerState
	pub predictedErrorTime: c_int,
	pub predictedError: vec3_t,

	pub stepChange: c_float,				// for stair up smoothing
	pub stepTime: c_int,

	pub duckChange: c_float,				// for duck viewheight smoothing
	pub duckTime: c_int,

	pub landChange: c_float,				// for landing hard
	pub landTime: c_int,

	// input state sent to server
	pub weaponSelect: c_int,
	pub saberAnimLevelPending: c_int,

	// auto rotating items
	pub autoAngles: vec3_t,
	pub autoAxis: [vec3_t; 3],
	pub autoAnglesFast: vec3_t,
	pub autoAxisFast: [vec3_t; 3],

	// view rendering
	pub refdef: refdef_t,
	pub refdefViewAngles: vec3_t,		// will be converted to refdef.viewaxis

	// zoom key
	pub zoomMode: c_int,		// 0 - not zoomed, 1 - binoculars, 2 - disruptor weapon
	pub zoomDir: c_int,		// -1, 1
	pub zoomTime: c_int,
	pub zoomLocked: qboolean,

	// gonk use
	pub batteryChargeTime: c_int,

	// FIXME:
	pub forceCrosshairStartTime: c_int,
	pub forceCrosshairEndTime: c_int,

	// information screen text during loading
	pub infoScreenText: [c_char; MAX_STRING_CHARS as usize],

	// centerprinting
	pub centerPrintTime: c_int,
	pub centerPrintY: c_int,
	pub centerPrint: [c_char; 1024],
	pub centerPrintLines: c_int,

	// Scrolling text, caption text and LCARS text use this
	pub printText: [[c_char; 128]; MAX_PRINTTEXT as usize],
	pub printTextY: c_int,

	pub captionText: [[c_char; 256]; MAX_CAPTIONTEXT as usize],	// bosted for taiwanese squealy radio static speech in kejim post
	pub captionTextY: c_int,

	pub scrollTextLines: c_int,	// Number of lines being printed
	pub scrollTextTime: c_int,

	pub captionNextTextTime: c_int,
	pub captionTextCurrentLine: c_int,
	pub captionTextTime: c_int,
	pub captionLetterTime: c_int,

	// For flashing health armor counter
	pub oldhealth: c_int,
	pub oldHealthTime: c_int,
	pub oldarmor: c_int,
	pub oldArmorTime: c_int,
	pub oldammo: c_int,
	pub oldAmmoTime: c_int,

	// low ammo warning state
	pub lowAmmoWarning: c_int,		// 1 = low, 2 = empty

	// crosshair client ID
	pub crosshairClientNum: c_int,		//who you're looking at
	pub crosshairClientTime: c_int,	//last time you looked at them

	// powerup active flashing
	pub powerupActive: c_int,
	pub powerupTime: c_int,

	//==========================
	pub creditsStart: c_int,

	pub itemPickup: c_int,
	pub itemPickupTime: c_int,
	pub itemPickupBlendTime: c_int,	// the pulse around the crosshair is timed seperately

	pub iconHUDPercent: c_float,			// How far into opening sequence the icon HUD is
	pub iconSelectTime: c_int,			// How long the Icon HUD has been active
	pub iconHUDActive: qboolean,

	pub DataPadInventorySelect: c_int,		// Current inventory item chosen on Data Pad
	pub DataPadWeaponSelect: c_int,		// Current weapon item chosen on Data Pad
	pub DataPadforcepowerSelect: c_int,	// Current force power chosen on Data Pad

	pub messageLitActive: qboolean,			// Flag to show of message lite is active

	pub weaponSelectTime: c_int,
	pub weaponAnimation: c_int,
	pub weaponAnimationTime: c_int,

	pub inventorySelect: c_int,		// Current inventory item chosen
	pub inventorySelectTime: c_int,

	pub forcepowerSelect: c_int,		// Current force power chosen
	pub forcepowerSelectTime: c_int,

	// blend blobs
	pub damageTime: c_float,
	pub damageX: c_float,
	pub damageY: c_float,
	pub damageValue: c_float,

	// status bar head
	pub headYaw: c_float,
	pub headEndPitch: c_float,
	pub headEndYaw: c_float,
	pub headEndTime: c_int,
	pub headStartPitch: c_float,
	pub headStartYaw: c_float,
	pub headStartTime: c_int,

	pub loadLCARSStage: c_int,

	pub missionInfoFlashTime: c_int,
	pub missionStatusShow: qboolean,
	pub missionStatusDeadTime: c_int,

	pub forceHUDTotalFlashTime: c_int,
	pub forceHUDNextFlashTime: c_int,
	pub forceHUDActive: qboolean,				// Flag to show force hud is off/on

	pub missionFailedScreen: qboolean,	// qtrue if opened

	pub weaponPickupTextTime: c_int,

	pub VHUDFlashTime: c_int,
	pub VHUDTurboFlag: qboolean,
	pub HUDTickFlashTime: c_int,
	pub HUDArmorFlag: qboolean,
	pub HUDHealthFlag: qboolean,

	// view movement
	pub v_dmg_time: c_float,
	pub v_dmg_pitch: c_float,
	pub v_dmg_roll: c_float,

	pub wonkyTime: c_int,		// when interrogator gets you, wonky time controls "drugged" camera view.

	pub kick_angles: vec3_t,	// weapon kicks
	pub kick_time: c_int,		// when the kick happened, so it gets reduced over time

	// temp working variables for player view
	pub bobfracsin: c_float,
	pub bobcycle: c_int,
	pub xyspeed: c_float,

	// development tool
	pub testModelName: [c_char; MAX_QPATH as usize],
	/*
	Ghoul2 Insert Start
	*/
	pub testModel: c_int,
	// had to be moved so we wouldn't wipe these out with the memset - these have STL in them and shouldn't be cleared that way
	pub activeSnapshots: [snapshot_t; 2],
	pub testModelEntity: refEntity_t,
	/*
	Ghoul2 Insert End
	*/
	pub overrides: overrides_t,	//for overriding certain third-person camera properties
}


pub const MAX_SHOWPOWERS: c_int = 12;
pub extern "C" {
	pub static mut showPowers: [c_int; MAX_SHOWPOWERS as usize];
	pub static mut showPowersName: [*mut c_char; MAX_SHOWPOWERS as usize];
	pub static mut force_icons: [c_int; NUM_FORCE_POWERS as usize];
}
pub const MAX_DPSHOWPOWERS: c_int = 16;

//==============================================================================


pub const SG_OFF: c_int = 0;
pub const SG_STRING: c_int = 1;
pub const SG_GRAPHIC: c_int = 2;
pub const SG_NUMBER: c_int = 3;
pub const SG_VAR: c_int = 4;

#[repr(C)]
pub struct screengraphics_s
{
	pub type_: c_int,		// STRING or GRAPHIC
	pub timer: c_float,		// When it changes
	pub x: c_int,			// X position
	pub y: c_int,			// Y positon
	pub width: c_int,		// Graphic width
	pub height: c_int,		// Graphic height
	pub file: *mut c_char,		// File name of graphic/ text if STRING
	pub ingameEnum: c_int,	// Index to ingame_text[]
	pub graphic: qhandle_t,	// Handle of graphic if GRAPHIC
	pub min: c_int,		//
	pub max: c_int,
	pub target: c_int,		// Final value
	pub inc: c_int,
	pub style: c_int,
	pub color: c_int,		// Normal color
	pub pointer: *mut c_void,		// To an address
}


pub extern "C" {
	pub static mut cg: cg_t;
	pub static mut cg_entities: [centity_t; MAX_GENTITIES as usize];

	pub static mut cg_permanents: [*mut centity_t; MAX_GENTITIES as usize];
	pub static mut cg_numpermanents: c_int;

	pub static mut cg_weapons: [weaponInfo_t; MAX_WEAPONS as usize];
	pub static mut cg_items: [itemInfo_t; MAX_ITEMS as usize];
	pub static mut cg_markPolys: [markPoly_t; MAX_MARK_POLYS as usize];


	pub static mut cg_runpitch: vmCvar_t;
	pub static mut cg_runroll: vmCvar_t;
	pub static mut cg_bobup: vmCvar_t;
	pub static mut cg_bobpitch: vmCvar_t;
	pub static mut cg_bobroll: vmCvar_t;
	pub static mut cg_shadows: vmCvar_t;
	pub static mut cg_renderToTextureFX: vmCvar_t;
	pub static mut cg_shadowCullDistance: vmCvar_t;
	pub static mut cg_paused: vmCvar_t;
	pub static mut cg_drawTimer: vmCvar_t;
	pub static mut cg_drawFPS: vmCvar_t;
	pub static mut cg_drawSnapshot: vmCvar_t;
	pub static mut cg_drawAmmoWarning: vmCvar_t;
	pub static mut cg_drawCrosshair: vmCvar_t;
	pub static mut cg_crosshairForceHint: vmCvar_t;
	pub static mut cg_crosshairIdentifyTarget: vmCvar_t;
	pub static mut cg_crosshairX: vmCvar_t;
	pub static mut cg_crosshairY: vmCvar_t;
	pub static mut cg_crosshairSize: vmCvar_t;
	pub static mut cg_drawStatus: vmCvar_t;
	pub static mut cg_drawHUD: vmCvar_t;
	pub static mut cg_draw2D: vmCvar_t;
	pub static mut cg_debugAnim: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_debugAnimTarget: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_gun_frame: vmCvar_t;
	pub static mut cg_debugSaber: vmCvar_t;
	pub static mut cg_debugEvents: vmCvar_t;
	pub static mut cg_errorDecay: vmCvar_t;
	pub static mut cg_footsteps: vmCvar_t;
	pub static mut cg_addMarks: vmCvar_t;
	pub static mut cg_drawGun: vmCvar_t;
	pub static mut cg_autoswitch: vmCvar_t;
	pub static mut cg_simpleItems: vmCvar_t;
	pub static mut cg_fov: vmCvar_t;
	pub static mut cg_endcredits: vmCvar_t;
	pub static mut cg_updatedDataPadForcePower1: vmCvar_t;
	pub static mut cg_updatedDataPadForcePower2: vmCvar_t;
	pub static mut cg_updatedDataPadForcePower3: vmCvar_t;
	pub static mut cg_updatedDataPadObjective: vmCvar_t;
	pub static mut cg_drawBreath: vmCvar_t;
	pub static mut cg_roffdebug: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_roffval1: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_roffval2: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_roffval3: vmCvar_t;
	#[cfg(not(feature = "FINAL_BUILD"))]
	pub static mut cg_roffval4: vmCvar_t;
	pub static mut cg_thirdPerson: vmCvar_t;
	pub static mut cg_thirdPersonRange: vmCvar_t;
	pub static mut cg_thirdPersonMaxRange: vmCvar_t;
	pub static mut cg_thirdPersonAngle: vmCvar_t;
	pub static mut cg_thirdPersonPitchOffset: vmCvar_t;
	pub static mut cg_thirdPersonVertOffset: vmCvar_t;
	pub static mut cg_thirdPersonCameraDamp: vmCvar_t;
	pub static mut cg_thirdPersonTargetDamp: vmCvar_t;
	pub static mut cg_gunAutoFirst: vmCvar_t;

	pub static mut cg_stereoSeparation: vmCvar_t;
	pub static mut cg_developer: vmCvar_t;
	pub static mut cg_timescale: vmCvar_t;
	pub static mut cg_skippingcin: vmCvar_t;

	pub static mut cg_pano: vmCvar_t;
	pub static mut cg_panoNumShots: vmCvar_t;

	pub static mut fx_freeze: vmCvar_t;
	pub static mut fx_debug: vmCvar_t;

	pub static mut cg_missionInfoFlashTime: vmCvar_t;
	pub static mut cg_hudFiles: vmCvar_t;

	pub static mut cg_turnAnims: vmCvar_t;
	pub static mut cg_motionBoneComp: vmCvar_t;
	pub static mut cg_reliableAnimSounds: vmCvar_t;

	pub static mut cg_smoothPlayerPos: vmCvar_t;
	pub static mut cg_smoothPlayerPlat: vmCvar_t;
	pub static mut cg_smoothPlayerPlatAccel: vmCvar_t;
}

pub extern "C" {
	pub fn CG_NewClientinfo( clientNum: c_int );

	//
	// cg_main.c
	//
	pub fn CG_ConfigString( index: c_int ) -> *const c_char;
	pub fn CG_Argv( arg: c_int ) -> *const c_char;

	pub fn CG_Printf( msg: *const c_char, ... );
	pub fn CG_Error( msg: *const c_char, ... );

	pub fn CG_StartMusic( bForceStart: qboolean );

	pub fn CG_UpdateCvars( );

	pub fn CG_CrosshairPlayer( ) -> c_int;
	pub fn CG_LoadMenus(menuFile: *const c_char);

	//
	// cg_view.c
	//
	pub fn CG_TestModel_f ();
	pub fn CG_TestModelNextFrame_f ();
	pub fn CG_TestModelPrevFrame_f ();
	pub fn CG_TestModelNextSkin_f ();
	pub fn CG_TestModelPrevSkin_f ();

	pub fn CG_ZoomDown_f( );
	pub fn CG_ZoomUp_f( );

	pub fn CG_DrawActiveFrame( serverTime: c_int, stereoView: stereoFrame_t );
	/*
	Ghoul2 Insert Start
	*/

	pub fn CG_TestG2Model_f ();
	pub fn CG_TestModelSurfaceOnOff_f();
	pub fn CG_ListModelSurfaces_f ();
	pub fn CG_ListModelBones_f ();
	pub fn CG_TestModelSetAnglespre_f();
	pub fn CG_TestModelSetAnglespost_f();
	pub fn CG_TestModelAnimate_f();
	/*
	Ghoul2 Insert End
	*/


	//
	// cg_drawtools.c
	//

	pub fn CG_DrawRect( x: c_float, y: c_float, width: c_float, height: c_float, size: c_float, color: *const c_float );
	pub fn CG_FillRect( x: c_float, y: c_float, width: c_float, height: c_float, color: *const c_float );
	pub fn CG_Scissor( x: c_float, y: c_float, width: c_float, height: c_float);
	pub fn CG_DrawPic( x: c_float, y: c_float, width: c_float, height: c_float, hShader: qhandle_t );
	pub fn CG_DrawPic2( x: c_float, y: c_float, width: c_float, height: c_float, s1: c_float, t1: c_float, s2: c_float, t2: c_float, hShader: qhandle_t );
	pub fn CG_DrawRotatePic( x: c_float, y: c_float, width: c_float, height: c_float, angle: c_float, hShader: qhandle_t );
	pub fn CG_DrawRotatePic2( x: c_float, y: c_float, width: c_float, height: c_float, angle: c_float, hShader: qhandle_t );
	pub fn CG_DrawString( x: c_float, y: c_float, string: *const c_char,
				   charWidth: c_float, charHeight: c_float, modulate: *const c_float );
	pub fn CG_PrintInterfaceGraphics(min: c_int, max: c_int);
	pub fn CG_DrawNumField (x: c_int, y: c_int, width: c_int, value: c_int, charWidth: c_int, charHeight: c_int, style: c_int, zeroFill: qboolean);
	pub fn CG_DrawProportionalString( x: c_int, y: c_int, str: *const c_char, style: c_int, color: vec4_t );


	pub fn CG_DrawStringExt( x: c_int, y: c_int, string: *const c_char, setColor: *const c_float,
			forceColor: qboolean, shadow: qboolean, charWidth: c_int, charHeight: c_int );
	pub fn CG_DrawSmallStringColor( x: c_int, y: c_int, s: *const c_char, color: vec4_t );

	pub fn CG_DrawStrlen( str: *const c_char ) -> c_int;

	pub fn CG_FadeColor( startMsec: c_int, totalMsec: c_int ) -> *mut c_float;
	pub fn CG_TileClear( );


	//
	// cg_draw.c
	//
	pub fn CG_CenterPrint( str: *const c_char, y: c_int );
	pub fn CG_DrawActive( stereoView: stereoFrame_t );
	pub fn CG_ScrollText( str: *const c_char, iPixelWidth: c_int );
	pub fn CG_CaptionText( str: *const c_char, sound: c_int, y: c_int );
	pub fn CG_CaptionTextStop( );

	//
	// cg_text.c
	//
	pub fn CG_DrawScrollText( );
	pub fn CG_DrawCaptionText( );
	pub fn CG_DrawCenterString( );


	//
	// cg_player.c
	//
	pub fn CG_AddGhoul2Mark(type_: c_int, size: c_float, hitloc: vec3_t, hitdirection: vec3_t,
				entnum: c_int, entposition: vec3_t, entangle: c_float, ghoul2: *mut c_void, modelScale: vec3_t, lifeTime: c_int, firstModel: c_int, uaxis: vec3_t);
	pub fn CG_Player( cent: *mut centity_t );
	pub fn CG_ResetPlayerEntity( cent: *mut centity_t );
	pub fn CG_AddRefEntityWithPowerups( ent: *mut refEntity_t, powerups: c_int, cent: *mut centity_t );
	pub fn CG_GetTagWorldPosition( model: *mut refEntity_t, tag: *mut c_char, pos: vec3_t, axis: *mut [vec3_t; 3] );

	//
	// cg_predict.c
	//
	pub fn CG_PointContents( point: *const vec3_t, passEntityNum: c_int ) -> c_int;
	pub fn CG_Trace( result: *mut trace_t, start: *const vec3_t, mins: *const vec3_t, maxs: *const vec3_t, end: *const vec3_t,
					 skipNumber: c_int, mask: c_int, eG2TraceType: EG2_Collision, useLod: c_int );
	pub fn CG_PredictPlayerState( );

	//
	// cg_events.c
	//
	pub fn CG_CheckEvents( cent: *mut centity_t );
	pub fn CG_PlaceString( rank: c_int ) -> *const c_char;
	pub fn CG_EntityEvent( cent: *mut centity_t, position: vec3_t );


	//
	// cg_ents.c
	//
	pub fn CG_SetEntitySoundPosition( cent: *mut centity_t ) -> *mut vec3_t;
	pub fn CG_AddPacketEntities( isPortal: qboolean );
	pub fn CG_Beam( cent: *mut centity_t, color: c_int );
	pub fn CG_Cylinder( start: vec3_t, end: vec3_t, radius: c_float, color: vec3_t );
	pub fn CG_AdjustPositionForMover( in_: *const vec3_t, moverNum: c_int, atTime: c_int, out: vec3_t );

	pub fn CG_PositionEntityOnTag( entity: *mut refEntity_t, parent: *const refEntity_t,
							parentModel: qhandle_t, tagName: *mut c_char );
	pub fn CG_PositionRotatedEntityOnTag( entity: *mut refEntity_t, parent: *const refEntity_t,
							parentModel: qhandle_t, tagName: *mut c_char, tagOrient: *mut orientation_t );

	/*
	Ghoul2 Insert Start
	*/
	pub fn ScaleModelAxis(ent: *mut refEntity_t);
	/*
	Ghoul2 Insert End
	*/


	//
	// cg_weapons.c
	//
	pub fn CG_NextWeapon_f( );
	pub fn CG_PrevWeapon_f( );
	pub fn CG_Weapon_f( );
	pub fn CG_DPNextWeapon_f( );
	pub fn CG_DPPrevWeapon_f( );
	pub fn CG_DPNextInventory_f( );
	pub fn CG_DPPrevInventory_f( );
	pub fn CG_DPNextForcePower_f( );
	pub fn CG_DPPrevForcePower_f( );


	pub fn CG_RegisterWeapon( weaponNum: c_int );
	pub fn CG_RegisterItemVisuals( itemNum: c_int );
	pub fn CG_RegisterItemSounds( itemNum: c_int );

	pub fn CG_FireWeapon( cent: *mut centity_t, alt_fire: qboolean );
	//pub fn CG_ChargeWeapon( centity_t *cent );

	pub fn CG_AddViewWeapon (ps: *mut playerState_t);
	pub fn CG_DrawWeaponSelect( );

	pub fn CG_OutOfAmmoChange( );	// should this be in pmove?

	//
	// cg_marks.c
	//
	pub fn CG_InitMarkPolys( );
	pub fn CG_AddMarks( );
	pub fn CG_ImpactMark( markShader: qhandle_t,
				    origin: *const vec3_t, dir: *const vec3_t,
					orientation: c_float,
				    r: c_float, g: c_float, b: c_float, a: c_float,
					alphaFade: qboolean,
					radius: c_float, temporary: qboolean );

	//
	// cg_localents.c
	//
	pub fn CG_InitLocalEntities( );
	pub fn CG_AllocLocalEntity( ) -> *mut localEntity_t;
	pub fn CG_AddLocalEntities( );

	//
	// cg_effects.c
	//

	/*localEntity_t *CG_MakeExplosion( vec3_t origin, vec3_t dir,
								qhandle_t hModel, int numframes, qhandle_t shader, int msec,
								qboolean isSprite, float scale = 1.0f );// Overloaded

	localEntity_t *CG_MakeExplosion( vec3_t origin, vec3_t dir,
								qhandle_t hModel, int numframes, qhandle_t shader, int msec,
								qboolean isSprite, float scale, int flags );// Overloaded
	*/
	pub fn CG_AddTempLight( origin: vec3_t, scale: c_float, color: vec3_t, msec: c_int ) -> *mut localEntity_t;

	pub fn CG_TestLine( start: vec3_t, end: vec3_t, time: c_int, color: c_int, radius: c_int);

	//
	// cg_snapshot.c
	//
	pub fn CG_ProcessSnapshots( );

	//
	// cg_info.c
	//
	pub fn CG_DrawInformation( );

	//
	// cg_scoreboard.c
	//
	pub fn CG_DrawScoreboard( ) -> qboolean;
	pub fn CG_MissionCompletion();

	//
	// cg_consolecmds.c
	//
	pub fn CG_ConsoleCommand( ) -> qboolean;
	pub fn CG_InitConsoleCommands( );

	//
	// cg_servercmds.c
	//
	pub fn CG_ExecuteNewServerCommands( latestSequence: c_int );
	pub fn CG_ParseServerinfo( );

	//
	// cg_playerstate.c
	//
	pub fn CG_Respawn( );
	pub fn CG_TransitionPlayerState( ps: *mut playerState_t, ops: *mut playerState_t );

	// cg_credits.cpp
	//
	pub fn CG_Credits_Init( psStripReference: *const c_char, pv4Color: *mut vec4_t);
	pub fn CG_Credits_Running( ) -> qboolean;
	pub fn CG_Credits_Draw( ) -> qboolean;


	//===============================================

	//
	// system calls
	// These functions are how the cgame communicates with the main game system
	//

	// print message on the local console
	pub fn cgi_Printf( fmt: *const c_char );

	// abort the game
	pub fn cgi_Error( fmt: *const c_char );

	// milliseconds should only be used for performance tuning, never
	// for anything game related.  Get time from the CG_DrawActiveFrame parameter
	pub fn cgi_Milliseconds( ) -> c_int;

	// console variable interaction
	pub fn cgi_Cvar_Register( vmCvar: *mut vmCvar_t, varName: *const c_char, defaultValue: *const c_char, flags: c_int );
	pub fn cgi_Cvar_Update( vmCvar: *mut vmCvar_t );
	pub fn cgi_Cvar_Set( var_name: *const c_char, value: *const c_char );


	// ServerCommand and ConsoleCommand parameter access
	pub fn cgi_Argc( ) -> c_int;
	pub fn cgi_Argv( n: c_int, buffer: *mut c_char, bufferLength: c_int );
	pub fn cgi_Args( buffer: *mut c_char, bufferLength: c_int );

	// filesystem access
	// returns length of file
	pub fn cgi_FS_FOpenFile( qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t ) -> c_int;
	pub fn cgi_FS_Read( buffer: *mut c_void, len: c_int, f: fileHandle_t ) -> c_int;
	pub fn cgi_FS_Write( buffer: *const c_void, len: c_int, f: fileHandle_t ) -> c_int;
	pub fn cgi_FS_FCloseFile( f: fileHandle_t );

	// add commands to the local console as if they were typed in
	// for map changing, etc.  The command is not executed immediately,
	// but will be executed in order the next time console commands
	// are processed
	pub fn cgi_SendConsoleCommand( text: *const c_char );

	// register a command name so the console can perform command completion.
	// FIXME: replace this with a normal console command "defineCommand"?
	pub fn cgi_AddCommand( cmdName: *const c_char );

	// send a string to the server over the network
	pub fn cgi_SendClientCommand( s: *const c_char );

	// force a screen update, only used during gamestate load
	pub fn cgi_UpdateScreen( );

	//RMG
	pub fn cgi_RMG_Init(terrainID: c_int, terrainInfo: *const c_char);
	pub fn cgi_CM_RegisterTerrain(terrainInfo: *const c_char) -> c_int;
	pub fn cgi_RE_InitRendererTerrain( terrainInfo: *const c_char );

	// model collision
	pub fn cgi_CM_LoadMap( mapname: *const c_char, subBSP: qboolean );
	pub fn cgi_CM_NumInlineModels( ) -> c_int;
	pub fn cgi_CM_InlineModel( index: c_int ) -> clipHandle_t;		// 0 = world, 1+ = bmodels
	pub fn cgi_CM_TempBoxModel( mins: *const vec3_t, maxs: *const vec3_t ) -> clipHandle_t;//, const int contents );
	pub fn cgi_CM_PointContents( p: *const vec3_t, model: clipHandle_t ) -> c_int;
	pub fn cgi_CM_TransformedPointContents( p: *const vec3_t, model: clipHandle_t, origin: *const vec3_t, angles: *const vec3_t ) -> c_int;
	pub fn cgi_CM_BoxTrace( results: *mut trace_t, start: *const vec3_t, end: *const vec3_t,
						  mins: *const vec3_t, maxs: *const vec3_t,
						  model: clipHandle_t, brushmask: c_int );
	pub fn cgi_CM_TransformedBoxTrace( results: *mut trace_t, start: *const vec3_t, end: *const vec3_t,
						  mins: *const vec3_t, maxs: *const vec3_t,
						  model: clipHandle_t, brushmask: c_int,
						  origin: *const vec3_t, angles: *const vec3_t );

	// Returns the projection of a polygon onto the solid brushes in the world
	pub fn cgi_CM_MarkFragments( numPoints: c_int, points: *const *const vec3_t,
				projection: *const vec3_t,
				maxPoints: c_int, pointBuffer: *mut vec3_t,
				maxFragments: c_int, fragmentBuffer: *mut markFragment_t ) -> c_int;

	// normal sounds will have their volume dynamically changed as their entity
	// moves and the listener moves
	pub fn cgi_S_StartSound( origin: *const vec3_t, entityNum: c_int, entchannel: c_int, sfx: sfxHandle_t );
	pub fn cgi_S_StopSounds( );

	// a local sound is always played full volume
	pub fn cgi_S_StartLocalSound( sfx: sfxHandle_t, channelNum: c_int );
	pub fn cgi_S_ClearLoopingSounds( );
	pub fn cgi_S_AddLoopingSound(entityNum: c_int, origin: *const vec3_t, velocity: *const vec3_t, sfx: sfxHandle_t );
	pub fn cgi_S_UpdateEntityPosition( entityNum: c_int, origin: *const vec3_t );

	// repatialize recalculates the volumes of sound as they should be heard by the
	// given entityNum and position
	pub fn cgi_S_Respatialize( entityNum: c_int, origin: *const vec3_t, axis: *mut [vec3_t; 3], inwater: qboolean );
	pub fn cgi_S_RegisterSound( sample: *const c_char ) -> sfxHandle_t;		// returns buzz if not found
	pub fn cgi_S_StartBackgroundTrack( intro: *const c_char, loop_: *const c_char, bForceStart: qboolean );	// empty name stops music
	pub fn cgi_S_GetSampleLength( sfx: sfxHandle_t) -> c_float;


	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_Start( ff: ffHandle_t, clientNum: c_int );
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_Ensure( ff: ffHandle_t, clientNum: c_int );
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_Stop( ff: ffHandle_t, clientNum: c_int );
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_StopAll( );
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_Shake( intensity: c_int, duration: c_int );
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_Register( name: *const c_char, channel: c_int ) -> ffHandle_t;
	#[cfg(feature = "IMMERSION")]
	pub fn cgi_FF_AddLoopingForce( handle: ffHandle_t, entNum: c_int );
	#[cfg(not(feature = "IMMERSION"))]
	// I've made these into ints instead of original typedefs to cut down on rebuild time
	//	if I update the module they're in. No point in rebuilding all CGAME modules...
	//
	pub fn cgi_FF_StartFX( iFX: c_int );
	#[cfg(not(feature = "IMMERSION"))]
	pub fn cgi_FF_EnsureFX( iFX: c_int );
	#[cfg(not(feature = "IMMERSION"))]
	pub fn cgi_FF_StopFX( iFX: c_int );
	#[cfg(not(feature = "IMMERSION"))]
	pub fn cgi_FF_StopAllFX( );

	#[cfg(target_os = "xbox")]
	pub fn cgi_FF_Xbox_Shake( intensity: c_float, duration: c_int );
	#[cfg(target_os = "xbox")]
	pub fn cgi_FF_Xbox_Damage( damage: c_int, xpos: c_float );



	pub fn cgi_R_LoadWorldMap( mapname: *const c_char );

	// all media should be registered during level startup to prevent
	// hitches during gameplay
	pub fn cgi_R_RegisterModel( name: *const c_char ) -> qhandle_t;			// returns rgb axis if not found
	pub fn cgi_R_RegisterSkin( name: *const c_char ) -> qhandle_t;
	pub fn cgi_R_RegisterShader( name: *const c_char ) -> qhandle_t;			// returns default shader if not found
	pub fn cgi_R_RegisterShaderNoMip( name: *const c_char ) -> qhandle_t;			// returns all white if not found
	pub fn cgi_R_RegisterFont( name: *const c_char ) -> qhandle_t;
	pub fn cgi_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: c_float) -> c_int;
	pub fn cgi_R_Font_StrLenChars(text: *const c_char) -> c_int;
	pub fn cgi_R_Font_HeightPixels(iFontIndex: c_int, scale: c_float) -> c_int;
	pub fn cgi_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const c_float, setIndex: c_int, iMaxPixelWidth: c_int, scale: c_float);
	pub fn cgi_Language_IsAsian() -> qboolean;
	pub fn cgi_Language_UsesSpaces() -> qboolean;
	pub fn cgi_AnyLanguage_ReadCharFromString( psText: *const c_char, iAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut qboolean ) -> c_int;

	pub fn cgi_R_SetRefractProp(alpha: c_float, stretch: c_float, prepost: qboolean, negate: qboolean);

	// a scene is built up by calls to R_ClearScene and the various R_Add functions.
	// Nothing is drawn until R_RenderScene is called.
	pub fn cgi_R_ClearScene( );
	pub fn cgi_R_AddRefEntityToScene( re: *const refEntity_t );
	pub fn cgi_R_GetLighting( origin: *const vec3_t, ambientLight: vec3_t, directedLight: vec3_t, ligthDir: vec3_t );

	//used by miscents
	pub fn cgi_R_inPVS( p1: vec3_t, p2: vec3_t ) -> qboolean;

	// polys are intended for simple wall marks, not really for doing
	// significant construction
	pub fn cgi_R_AddPolyToScene( hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t );
	pub fn cgi_R_AddLightToScene( org: *const vec3_t, intensity: c_float, r: c_float, g: c_float, b: c_float );
	pub fn cgi_R_RenderScene( fd: *const refdef_t );
	pub fn cgi_R_SetColor( rgba: *const c_float );	// NULL = 1,1,1,1
	pub fn cgi_R_DrawStretchPic( x: c_float, y: c_float, w: c_float, h: c_float,
		s1: c_float, t1: c_float, s2: c_float, t2: c_float, hShader: qhandle_t );
	//pub fn cgi_R_DrawScreenShot( float x, float y, float w, float h);

	pub fn cgi_R_ModelBounds( model: qhandle_t, mins: vec3_t, maxs: vec3_t );
	pub fn cgi_R_LerpTag( tag: *mut orientation_t, mod_: qhandle_t, startFrame: c_int, endFrame: c_int,
					 frac: c_float, tagName: *const c_char );
	// Does weird, barely controllable rotation behaviour
	pub fn cgi_R_DrawRotatePic( x: c_float, y: c_float, w: c_float, h: c_float,
		s1: c_float, t1: c_float, s2: c_float, t2: c_float, a: c_float, hShader: qhandle_t );
	// rotates image around exact center point of passed in coords
	pub fn cgi_R_DrawRotatePic2( x: c_float, y: c_float, w: c_float, h: c_float,
		s1: c_float, t1: c_float, s2: c_float, t2: c_float, a: c_float, hShader: qhandle_t );
	pub fn cgi_R_SetRangeFog(range: c_float);
	pub fn cgi_R_LAGoggles( );
	pub fn cgi_R_Scissor( x: c_float, y: c_float, w: c_float, h: c_float);

	// The glconfig_t will not change during the life of a cgame.
	// If it needs to change, the entire cgame will be restarted, because
	// all the qhandle_t are then invalid.
	pub fn cgi_GetGlconfig( glconfig: *mut glconfig_t );

	// the gamestate should be grabbed at startup, and whenever a
	// configstring changes
	pub fn cgi_GetGameState( gamestate: *mut gameState_t );

	// cgame will poll each frame to see if a newer snapshot has arrived
	// that it is interested in.  The time is returned seperately so that
	// snapshot latency can be calculated.
	pub fn cgi_GetCurrentSnapshotNumber( snapshotNumber: *mut c_int, serverTime: *mut c_int );

	// a snapshot get can fail if the snapshot (or the entties it holds) is so
	// old that it has fallen out of the client system queue
	pub fn cgi_GetSnapshot( snapshotNumber: c_int, snapshot: *mut snapshot_t ) -> qboolean;

	pub fn cgi_GetDefaultState(entityIndex: c_int, state: *mut entityState_t ) -> qboolean;

	// retrieve a text command from the server stream
	// the current snapshot will hold the number of the most recent command
	// qfalse can be returned if the client system handled the command
	// argc() / argv() can be used to examine the parameters of the command
	pub fn cgi_GetServerCommand( serverCommandNumber: c_int ) -> qboolean;

	// returns the most recent command number that can be passed to GetUserCmd
	// this will always be at least one higher than the number in the current
	// snapshot, and it may be quite a few higher if it is a fast computer on
	// a lagged connection
	pub fn cgi_GetCurrentCmdNumber( ) -> c_int;
	pub fn cgi_GetUserCmd( cmdNumber: c_int, ucmd: *mut usercmd_t ) -> qboolean;

	// used for the weapon select and zoom
	pub fn cgi_SetUserCmdValue( stateValue: c_int, sensitivityScale: c_float, mPitchOverride: c_float, mYawOverride: c_float );
	pub fn cgi_SetUserCmdAngles( pitchOverride: c_float, yawOverride: c_float, rollOverride: c_float );

	pub fn cgi_S_UpdateAmbientSet( name: *const c_char, origin: vec3_t );
	pub fn cgi_AS_ParseSets( );
	pub fn cgi_AS_AddPrecacheEntry( name: *const c_char );
	pub fn cgi_S_AddLocalSet( name: *const c_char, listener_origin: vec3_t, origin: vec3_t, entID: c_int, time: c_int ) -> c_int;
	pub fn cgi_AS_GetBModelSound( name: *const c_char, stage: c_int ) -> sfxHandle_t;


	pub fn CG_DrawMiscEnts();


	//-----------------------------
	// Effects related prototypes
	//-----------------------------

	// Weapon prototypes
	pub fn FX_Saber( start: vec3_t, normal: vec3_t, height: c_float, radius: c_float, color: saber_colors_t );

	pub fn FX_BryarHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_BryarAltHitWall( origin: vec3_t, normal: vec3_t, power: c_int );
	pub fn FX_BryarHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );
	pub fn FX_BryarAltHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_BlasterProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_BlasterAltFireThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_BlasterWeaponHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_BlasterWeaponHitPlayer( hit: *mut gentity_t, origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_DisruptorMainShot( start: vec3_t, end: vec3_t );
	pub fn FX_DisruptorAltShot( start: vec3_t, end: vec3_t, full: qboolean );
	pub fn FX_DisruptorAltMiss( origin: vec3_t, normal: vec3_t );

	pub fn FX_BowcasterHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_BowcasterHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_RepeaterHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_RepeaterAltHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_RepeaterHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );
	pub fn FX_RepeaterAltHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_DEMP2_HitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_DEMP2_HitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );
	pub fn FX_DEMP2_AltDetonate( org: vec3_t, size: c_float );

	pub fn FX_FlechetteProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_FlechetteWeaponHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_FlechetteWeaponHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_RocketHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_RocketHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_ConcProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_ConcHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_ConcHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );
	pub fn FX_ConcAltShot( start: vec3_t, end: vec3_t );
	pub fn FX_ConcAltMiss( origin: vec3_t, normal: vec3_t );

	pub fn FX_EmplacedHitWall( origin: vec3_t, normal: vec3_t, eweb: qboolean );
	pub fn FX_EmplacedHitPlayer( origin: vec3_t, normal: vec3_t, eweb: qboolean );
	pub fn FX_EmplacedProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );

	pub fn FX_ATSTMainHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_ATSTMainHitPlayer( origin: vec3_t, normal: vec3_t, humanoid: qboolean );
	pub fn FX_ATSTMainProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );

	pub fn FX_TuskenShotProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_TuskenShotWeaponHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_TuskenShotWeaponHitPlayer( hit: *mut gentity_t, origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn FX_NoghriShotProjectileThink( cent: *mut centity_t, weapon: *const weaponInfo_s );
	pub fn FX_NoghriShotWeaponHitWall( origin: vec3_t, normal: vec3_t );
	pub fn FX_NoghriShotWeaponHitPlayer( hit: *mut gentity_t, origin: vec3_t, normal: vec3_t, humanoid: qboolean );

	pub fn CG_BounceEffect( cent: *mut centity_t, weapon: c_int, origin: vec3_t, normal: vec3_t );
	pub fn CG_MissileStick( cent: *mut centity_t, weapon: c_int, origin: vec3_t );

	pub fn CG_MissileHitPlayer( cent: *mut centity_t, weapon: c_int, origin: vec3_t, dir: vec3_t, altFire: qboolean );
	pub fn CG_MissileHitWall( cent: *mut centity_t, weapon: c_int, origin: vec3_t, dir: vec3_t, altFire: qboolean );

	pub fn CG_DrawTargetBeam( start: vec3_t, end: vec3_t, norm: vec3_t, beamFx: *const c_char, impactFx: *const c_char );


	/*
	Ghoul2 Insert Start
	*/
	// CG specific API access
	pub fn trap_G2_SetGhoul2ModelIndexes(ghoul2: &mut CGhoul2Info_v, modelList: *mut qhandle_t, skinList: *mut qhandle_t);
	pub fn CG_Init_CG();

	pub fn CG_SetGhoul2Info( ent: *mut refEntity_t, cent: *mut centity_t);

	/*
	Ghoul2 Insert End
	*/
	pub fn trap_Com_SetOrgAngles(org: vec3_t, angles: vec3_t);
	pub fn trap_R_GetLightStyle(style: c_int, color: color4ub_t);
	pub fn trap_R_SetLightStyle(style: c_int, color: c_int);

	pub fn trap_CIN_PlayCinematic( arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int, psAudioFile: *const c_char) -> c_int;
	pub fn trap_CIN_StopCinematic(handle: c_int) -> e_status;
	pub fn trap_CIN_RunCinematic (handle: c_int) -> e_status;
	pub fn trap_CIN_DrawCinematic (handle: c_int);
	pub fn trap_CIN_SetExtents (handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);
	pub fn cgi_Z_Malloc( size: c_int, tag: c_int ) -> *mut c_void;
	pub fn cgi_Z_Free( ptr: *mut c_void );

	pub fn cgi_SP_GetStringTextString(text: *const c_char, buf: *mut c_char, bufferlength: c_int) -> c_int;


	pub fn cgi_UI_Menu_Reset( );
	pub fn cgi_UI_Menu_New(buf: *mut c_char );
	pub fn cgi_UI_Menu_OpenByName(buf: *mut c_char);
	pub fn cgi_UI_SetActive_Menu(name: *mut c_char);
	pub fn cgi_UI_Parse_Int(value: *mut c_int);
	pub fn cgi_UI_Parse_String(buf: *mut c_char);
	pub fn cgi_UI_Parse_Float(value: *mut c_float);
	pub fn cgi_UI_StartParseSession(menuFile: *mut c_char, buf: *mut *mut c_char) -> c_int;
	pub fn cgi_UI_ParseExt(token: *mut *mut c_char);
	pub fn cgi_UI_MenuPaintAll();
	pub fn cgi_UI_MenuCloseAll();
	pub fn cgi_UI_String_Init();
	pub fn cgi_UI_GetMenuItemInfo(menuFile: *const c_char, itemName: *const c_char, x: *mut c_int, y: *mut c_int, w: *mut c_int, h: *mut c_int, color: vec4_t, background: *mut qhandle_t) -> c_int;
	pub fn cgi_UI_GetMenuInfo(menuFile: *mut c_char, x: *mut c_int, y: *mut c_int, w: *mut c_int, h: *mut c_int) -> c_int;

	pub fn SetWeaponSelectTime();

	pub fn CG_PlayEffectBolted( fxName: *const c_char, modelIndex: c_int, boltIndex: c_int, entNum: c_int, origin: vec3_t, iLoopTime: c_int, isRelative: bool );
	pub fn CG_PlayEffectIDBolted( fxID: c_int, modelIndex: c_int, boltIndex: c_int, entNum: c_int, origin: vec3_t, iLoopTime: c_int, isRelative: bool );
	pub fn CG_PlayEffectOnEnt( fxName: *const c_char, clientNum: c_int, origin: vec3_t, fwd: *const vec3_t );
	pub fn CG_PlayEffectIDOnEnt( fxID: c_int, clientNum: c_int, origin: vec3_t, fwd: *const vec3_t );
	pub fn CG_PlayEffect( fxName: *const c_char, origin: vec3_t, fwd: *const vec3_t );
	pub fn CG_PlayEffectID( fxID: c_int, origin: vec3_t, fwd: *const vec3_t );
}

#[allow(non_upper_case_globals)]
pub const CG_LEFT: c_int = 0x00000000;	// default
pub const CG_CENTER: c_int = 0x00000001;
pub const CG_RIGHT: c_int = 0x00000002;
pub const CG_FORMATMASK: c_int = 0x00000007;
pub const CG_SMALLFONT: c_int = 0x00000010;
pub const CG_BIGFONT: c_int = 0x00000020;	// default
pub const CG_GIANTFONT: c_int = 0x00000040;
pub const CG_DROPSHADOW: c_int = 0x00000800;
pub const CG_BLINK: c_int = 0x00001000;
pub const CG_INVERSE: c_int = 0x00002000;
pub const CG_PULSE: c_int = 0x00004000;
pub const CG_UNDERLINE: c_int = 0x00008000;
pub const CG_TINYFONT: c_int = 0x00010000;
