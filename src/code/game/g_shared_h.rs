// Faithful translation of oracle/code/game/g_shared.h
// Preserve original C behavior and layout

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_char, c_void};

// Include dependencies from other modules (local stubs as needed for structural coherence)
// Note: These types are defined elsewhere in the Rust translation
use crate::code::game::bg_public::*;
use crate::code::game::g_public::*;
use crate::code::game::b_public::*;
use crate::code::Icarus::Stdafx::*;
use crate::code::renderer::tr_types::*;
use crate::code::cgame::cg_public::*;
use crate::code::game::G_Vehicles::*;
use crate::code::game::hitlocs::*;
use crate::code::game::bset::*;

// FOFS(x) macro - calculates field offset from gentity_t pointer cast
// #define FOFS(x) ((int)&(((gentity_t *)0)->x))
// In Rust, we use a function-like macro via const
pub const fn FOFS<T>() -> usize {
    // This is a compile-time stub; actual offset calculations would use offset_of! or memoffset
    0
}

// #ifdef _XBOX
// #define MAX_NPC_WATER_UPDATE_PER_FRAME	2	// maxmum number of NPCs that will get updated water infromation per frame
// #endif
#[cfg(target_os = "xbox")]
pub const MAX_NPC_WATER_UPDATE_PER_FRAME: c_int = 2; // maxmum number of NPCs that will get updated water infromation per frame

/// taskID_e enum - used to track task timers
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum taskID_e {
    TID_CHAN_VOICE = 0, // Waiting for a voice sound to complete
    TID_ANIM_UPPER,     // Waiting to finish a lower anim holdtime
    TID_ANIM_LOWER,     // Waiting to finish a lower anim holdtime
    TID_ANIM_BOTH,      // Waiting to finish lower and upper anim holdtimes or normal md3 animating
    TID_MOVE_NAV,       // Trying to get to a navgoal or For ET_MOVERS
    TID_ANGLE_FACE,     // Turning to an angle or facing
    TID_BSTATE,         // Waiting for a certain bState to finish
    TID_LOCATION,       // Waiting for ent to enter a specific trigger_location
    // TID_MISSIONSTATUS,	// Waiting for player to finish reading MISSION STATUS SCREEN
    TID_RESIZE,  // Waiting for clear bbox to inflate size
    TID_SHOOT,   // Waiting for fire event
    NUM_TIDS,    // for def of taskID array
}

pub type taskID_t = taskID_e;

/// material_e enum - material types for chunk effects
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum material_e {
    MAT_METAL = 0,      // scorched blue-grey metal
    MAT_GLASS,          // not a real chunk type, just plays an effect with glass sprites
    MAT_ELECTRICAL,     // sparks only
    MAT_ELEC_METAL,     // sparks/electrical type metal
    MAT_DRK_STONE,      // brown
    MAT_LT_STONE,       // tan
    MAT_GLASS_METAL,    // glass sprites and METAl chunk
    MAT_METAL2,         // electrical metal type
    MAT_NONE,           // no chunks
    MAT_GREY_STONE,     // grey
    MAT_METAL3,         // METAL and METAL2 chunks
    MAT_CRATE1,         // yellow multi-colored crate chunks
    MAT_GRATE1,         // grate chunks
    MAT_ROPE,           // for yavin trial...no chunks, just wispy bits
    MAT_CRATE2,         // read multi-colored crate chunks
    MAT_WHITE_METAL,    // white angular chunks

    NUM_MATERIALS,
}

pub type material_t = material_e;

//===From cg_local.h================================================
pub const DEFAULT_HEADMODEL: &[u8] = b"";
pub const DEFAULT_TORSOMODEL: &[u8] = b"";
pub const DEFAULT_LEGSMODEL: &[u8] = b"mouse";

// each client has an associated clientInfo_t
// that contains media references necessary to present the
// client model and other color coded effects
// this is regenerated each time a userinfo configstring changes

pub const MAX_CUSTOM_BASIC_SOUNDS: usize = 14;
pub const MAX_CUSTOM_COMBAT_SOUNDS: usize = 17;
pub const MAX_CUSTOM_EXTRA_SOUNDS: usize = 36;
pub const MAX_CUSTOM_JEDI_SOUNDS: usize = 22;
pub const MAX_CUSTOM_SOUNDS: usize = MAX_CUSTOM_JEDI_SOUNDS
    + MAX_CUSTOM_EXTRA_SOUNDS
    + MAX_CUSTOM_COMBAT_SOUNDS
    + MAX_CUSTOM_BASIC_SOUNDS;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct clientInfo_t {
    pub infoValid: qboolean,

    pub name: [c_char; MAX_QPATH],
    pub team: team_t,

    pub score: c_int,      // updated by score servercmds
    pub handicap: c_int,

    pub legsModel: qhandle_t,
    pub legsSkin: qhandle_t,

    pub torsoModel: qhandle_t,
    pub torsoSkin: qhandle_t,

    pub headModel: qhandle_t,
    pub headSkin: qhandle_t,

    pub animFileIndex: c_int,

    pub sounds: [sfxHandle_t; MAX_CUSTOM_SOUNDS],

    pub customBasicSoundDir: *mut c_char,
    pub customCombatSoundDir: *mut c_char,
    pub customExtraSoundDir: *mut c_char,
    pub customJediSoundDir: *mut c_char,
}

//==================================================================
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum moverState_e {
    MOVER_POS1,
    MOVER_POS2,
    MOVER_1TO2,
    MOVER_2TO1,
}

pub type moverState_t = moverState_e;

// Rendering information structure

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum targetModel_e {
    MODEL_LEGS = 0,
    MODEL_TORSO,
    MODEL_HEAD,
    MODEL_WEAPON1,
    MODEL_WEAPON2,
    MODEL_WEAPON3,
    MODEL_EXTRA1,
    MODEL_EXTRA2,
    NUM_TARGET_MODELS,
}

pub type targetModel_t = targetModel_e;

//renderFlags
pub const RF_LOCKEDANGLE: c_int = 1;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct renderInfo_s {
    // Legs model, or full model on one piece entities
    pub legsModelName: [c_char; 32], // -slc[]
    // (unnamed union field, use legsModelName or access as modelName overlay)

    pub torsoModelName: [c_char; 32], // -slc[]
    pub headModelName: [c_char; 32],  // -slc[]

    //In whole degrees, How far to let the different model parts yaw and pitch
    pub headYawRangeLeft: c_int,
    pub headYawRangeRight: c_int,
    pub headPitchRangeUp: c_int,
    pub headPitchRangeDown: c_int,

    pub torsoYawRangeLeft: c_int,
    pub torsoYawRangeRight: c_int,
    pub torsoPitchRangeUp: c_int,
    pub torsoPitchRangeDown: c_int,

    pub legsFrame: c_int,
    pub torsoFrame: c_int,

    pub legsFpsMod: f32,
    pub torsoFpsMod: f32,

    //Fields to apply to entire model set, individual model's equivalents will modify this value
    pub customRGBA: [u8; 4], //Red Green Blue, 0 = don't apply

    //Allow up to 4 PCJ lookup values to be stored here.
    //The resolve to configstrings which contain the name of the
    //desired bone.
    pub boneIndex1: c_int,
    pub boneIndex2: c_int,
    pub boneIndex3: c_int,
    pub boneIndex4: c_int,

    //packed with x, y, z orientations for bone angles
    pub boneOrient: c_int,

    //I.. feel bad for doing this, but NPCs really just need to
    //be able to control this sort of thing from the server sometimes.
    //At least it's at the end so this stuff is never going to get sent
    //over for anything that isn't an NPC.
    pub boneAngles1: vec3_t, //angles of boneIndex1
    pub boneAngles2: vec3_t, //angles of boneIndex2
    pub boneAngles3: vec3_t, //angles of boneIndex3
    pub boneAngles4: vec3_t, //angles of boneIndex4

    //RF?
    pub renderFlags: c_int,

    //
    pub muzzlePoint: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDirOld: vec3_t,
    //pub muzzlePointNext: vec3_t,	// Muzzle point one server frame in the future!
    //pub muzzleDirNext: vec3_t,
    pub mPCalcTime: c_int, //Last time muzzle point was calced

    //
    pub lockYaw: f32, //

    //
    pub headPoint: vec3_t,    //Where your tag_head is
    pub headAngles: vec3_t,   //where the tag_head in the torso is pointing
    pub handRPoint: vec3_t,   //where your right hand is
    pub handLPoint: vec3_t,   //where your left hand is
    pub crotchPoint: vec3_t,  //Where your crotch is
    pub footRPoint: vec3_t,   //where your right hand is
    pub footLPoint: vec3_t,   //where your left hand is
    pub torsoPoint: vec3_t,   //Where your chest is
    pub torsoAngles: vec3_t,  //Where the chest is pointing
    pub eyePoint: vec3_t,     //Where your eyes are
    pub eyeAngles: vec3_t,    //Where your eyes face
    pub lookTarget: c_int,    //Which ent to look at with lookAngles
    pub lookMode: lookMode_t,
    pub lookTargetClearTime: c_int, //Time to clear the lookTarget
    pub lastVoiceVolume: c_int,     //Last frame's voice volume
    pub lastHeadAngles: vec3_t,     //Last headAngles, NOT actual facing of head model
    pub headBobAngles: vec3_t,      //headAngle offsets
    pub targetHeadBobAngles: vec3_t, //head bob angles will try to get to targetHeadBobAngles
    pub lookingDebounceTime: c_int, //When we can stop using head looking angle behavior
    pub legsYaw: f32,               //yaw angle your legs are actually rendering at
}

pub type renderInfo_t = renderInfo_s;

// Movement information structure

/*
pub struct moveInfo_s {	// !!!!!!!!!! LOADSAVE-affecting struct !!!!!!!!
    pub desiredAngles: vec3_t,	// Desired facing angles
    pub speed: f32,	// Speed of movement
    pub aspeed: f32,	// Speed of angular movement
    pub moveDir: vec3_t,	// Direction of movement
    pub velocity: vec3_t,	// movement velocity
    pub flags: c_int,			// Special state flags
}

pub type moveInfo_t = moveInfo_s;
*/

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum clientConnected_e {
    CON_DISCONNECTED,
    CON_CONNECTING,
    CON_CONNECTED,
}

pub type clientConnected_t = clientConnected_e;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum playerTeamStateState_e {
    TEAM_BEGIN,  // Beginning a team game, spawn at base
    TEAM_ACTIVE, // Now actively playing
}

pub type playerTeamStateState_t = playerTeamStateState_e;

/*
pub enum race_e {
    RACE_NONE = 0,
    RACE_HUMAN,
    RACE_BORG,
    RACE_KLINGON,
    RACE_HIROGEN,
    RACE_MALON,
    RACE_STASIS,
    RACE_8472,
    RACE_BOT,
    RACE_HARVESTER,
    RACE_REAVER,
    RACE_AVATAR,
    RACE_PARASITE,
    RACE_VULCAN,
    RACE_BETAZOID,
    RACE_BOLIAN,
    RACE_TALAXIAN,
    RACE_BAJORAN,
    RACE_HOLOGRAM,
}

pub type race_t = race_e;
*/

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct playerTeamState_t {
    pub state: playerTeamStateState_t,

    pub captures: c_int,
    pub basedefense: c_int,
    pub carrierdefense: c_int,
    pub flagrecovery: c_int,
    pub fragcarrier: c_int,
    pub assists: c_int,

    pub lasthurtcarrier: f32,
    pub lastreturnedflag: f32,
    pub flagsince: f32,
    pub lastfraggedcarrier: f32,
}

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct objectives_s {
    pub display: qboolean, // A displayable objective?
    pub status: c_int,     // Succeed or fail or pending
}

pub type objectives_t = objectives_s;

// NOTE: This is an arbitrary number greater than our current number of objectives with
// some fluff just in case we add more in the future.
pub const MAX_MISSION_OBJ: usize = 100;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct missionStats_s {
    pub secretsFound: c_int,               // # of secret areas found
    pub totalSecrets: c_int,               // # of secret areas that could have been found
    pub shotsFired: c_int,                 // total number of shots fired
    pub hits: c_int,                       // Shots that did damage
    pub enemiesSpawned: c_int,             // # of enemies spawned
    pub enemiesKilled: c_int,              // # of enemies killed
    pub saberThrownCnt: c_int,             // # of times saber was thrown
    pub saberBlocksCnt: c_int,             // # of times saber was used to block
    pub legAttacksCnt: c_int,              // # of times legs were hit with saber
    pub armAttacksCnt: c_int,              // # of times arm were hit with saber
    pub torsoAttacksCnt: c_int,            // # of times torso was hit with saber
    pub otherAttacksCnt: c_int,            // # of times anything else on a monster was hit with saber
    pub forceUsed: [c_int; NUM_FORCE_POWERS], // # of times each force power was used
    pub weaponUsed: [c_int; WP_NUM_WEAPONS],  // # of times each weapon was used
}

pub type missionStats_t = missionStats_s;

// the auto following clients don't follow a specific client
// number, but instead follow the first two active players
pub const FOLLOW_ACTIVE1: c_int = -1;
pub const FOLLOW_ACTIVE2: c_int = -2;

// client data that stays across multiple levels or tournament restarts
// this is achieved by writing all the data to cvar strings at game shutdown
// time and reading them back at connection time.  Anything added here
// MUST be dealt with in G_InitSessionData() / G_ReadSessionData() / G_WriteSessionData()
//
// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct clientSession_t {
    pub missionObjectivesShown: c_int,     // Number of times mission objectives have been updated
    pub sessionTeam: team_t,
    pub mission_objectives: [objectives_t; MAX_MISSION_OBJ],
    pub missionStats: missionStats_t, // Various totals while on a mission
}

// client data that stays across multiple respawns, but is cleared
// on each level change or team change at ClientBegin()
// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct clientPersistant_t {
    pub connected: clientConnected_t,
    pub lastCommand: usercmd_t,
    pub netname: [c_char; 34],
    pub maxHealth: c_int,      // for handicapping
    pub enterTime: c_int,      // level.time the client entered the game
    pub cmd_angles: [i16; 3],  // angles sent over in the last command

    pub teamState: playerTeamState_t, // status in teamplay games
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum saberBlockType_e {
    BLK_NO,
    BLK_TIGHT, // Block only attacks and shots around the saber itself, a bbox of around 12x12x12
    BLK_WIDE,  // Block all attacks in an area around the player in a rough arc of 180 degrees
}

pub type saberBlockType_t = saberBlockType_e;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum saberBlockedType_e {
    BLOCKED_NONE,
    BLOCKED_PARRY_BROKEN,
    BLOCKED_ATK_BOUNCE,
    BLOCKED_UPPER_RIGHT,
    BLOCKED_UPPER_LEFT,
    BLOCKED_LOWER_RIGHT,
    BLOCKED_LOWER_LEFT,
    BLOCKED_TOP,
    BLOCKED_UPPER_RIGHT_PROJ,
    BLOCKED_UPPER_LEFT_PROJ,
    BLOCKED_LOWER_RIGHT_PROJ,
    BLOCKED_LOWER_LEFT_PROJ,
    BLOCKED_TOP_PROJ,
}

pub type saberBlockedType_t = saberBlockedType_e;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum movetype_e {
    MT_STATIC = 0,
    MT_WALK,
    MT_RUNJUMP,
    MT_FLYSWIM,
    NUM_MOVETYPES,
}

pub type movetype_t = movetype_e;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!

// this structure is cleared on each ClientSpawn(),
// except for 'client->pers' and 'client->sess'
#[repr(C)]
pub struct gclient_s {
    // ps MUST be the first element, because the server expects it
    pub ps: playerState_t, // communicated by server to clients

    // private to game
    pub pers: clientPersistant_t,
    pub sess: clientSession_t,

    pub lastCmdTime: c_int, // level.time of last usercmd_t, for EF_CONNECTION

    pub usercmd: usercmd_t, // most recent usercmd

    pub buttons: c_int,
    pub oldbuttons: c_int,
    pub latched_buttons: c_int,

    // sum up damage over an entire frame, so
    // shotgun blasts give a single big kick
    pub damage_armor: c_int,      // damage absorbed by armor
    pub damage_blood: c_int,      // damage taken out of health
    pub damage_from: vec3_t,      // origin for vector calculation
    pub damage_fromWorld: bool,   // if true, don't use the damage_from vector
    pub noclip: bool,
    //icarus forced moving.  is this still used?
    pub forced_forwardmove: i8,
    pub forced_rightmove: i8,

    // timers
    pub respawnTime: c_int, // can respawn when time > this, force after g_forcerespwan
    pub idleTime: c_int,    // for playing idleAnims

    pub airOutTime: c_int,

    // timeResidual is used to handle events that happen every second
    // like health / armor countdowns and regeneration
    pub timeResidual: c_int,

    // Facial Expression Timers

    pub facial_blink: f32,  // time before next blink. If a minus value, we are in blink mode
    pub facial_timer: f32,  // time before next alert, frown or smile. If a minus value, we are in anim mode
    pub facial_anim: c_int, // anim to show in anim mode

    //Client info - updated when ClientInfoChanged is called, instead of using configstrings
    pub clientInfo: clientInfo_t,
    pub moveType: movetype_t,
    pub jetPackTime: c_int,
    pub fireDelay: c_int, //msec to delay calling G_FireWeapon after EV_FIREWEAPON event is called

    // The time at which a breath should be triggered. -Aurelio
    pub breathPuffTime: c_int,

    //Used to be in gentity_t, now here.. mostly formation stuff
    pub playerTeam: team_t,
    pub enemyTeam: team_t,
    pub leader: *mut gentity_s,
    pub NPC_class: class_t,

    //FIXME: could combine these
    pub hiddenDist: f32,    //How close ents have to be to pick you up as an enemy
    pub hiddenDir: vec3_t,  //Normalized direction in which NPCs can't see you (you are hidden)

    pub renderInfo: renderInfo_t,

    //dismember tracker
    pub dismembered: bool,
    pub dismemberProbLegs: c_char,   // probability of the legs being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbHead: c_char,   // probability of the head being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbArms: c_char,   // probability of the arms being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbHands: c_char,  // probability of the hands being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbWaist: c_char,  // probability of the waist being dismembered (located in NPC.cfg, 0 = never, 100 = always)

    pub standheight: c_int,
    pub crouchheight: c_int,
    pub poisonDamage: c_int,             // Amount of poison damage to be given
    pub poisonTime: c_int,               // When to apply poison damage
    pub slopeRecalcTime: c_int,          // debouncer for slope-foot-height-diff calcing

    pub pushVec: vec3_t,
    pub pushVecTime: c_int,

    pub noRagTime: c_int,           //don't do ragdoll stuff if > level.time
    pub isRagging: qboolean,
    pub overridingBones: c_int,     //dragging body or doing something else to override one or more ragdoll effector's/pcj's

    pub ragLastOrigin: vec3_t,      //keeping track of positions between rags while dragging corpses
    pub ragLastOriginTime: c_int,

    //push refraction effect vars
    pub pushEffectFadeTime: c_int,
    pub pushEffectOrigin: vec3_t,

    //Rocket locking vars for non-player clients (only Vehicles use these right now...)
    pub rocketLockIndex: c_int,
    pub rocketLastValidTime: f32,
    pub rocketLockTime: f32,
    pub rocketTargetTime: f32,

    //for trigger_space brushes
    pub inSpaceSuffocation: c_int,
    pub inSpaceIndex: c_int,
}

pub type gclient_t = gclient_s;

pub const MAX_PARMS: usize = 16;
pub const MAX_PARM_STRING_LENGTH: usize = MAX_QPATH; //was 16, had to lengthen it so they could take a valid file path

#[repr(C)]
pub struct parms_t {
    pub parm: [[c_char; MAX_PARM_STRING_LENGTH]; MAX_PARMS],
}

#[cfg(feature = "GAME_INCLUDE")]
pub mod game_include {
    use super::*;

    //these hold the place for the enums in functions.h so i don't have to recompile everytime it changes
    pub type thinkFunc_t = c_int;
    pub type clThinkFunc_t = c_int;
    pub type reachedFunc_t = c_int;
    pub type blockedFunc_t = c_int;
    pub type touchFunc_t = c_int;
    pub type useFunc_t = c_int;
    pub type painFunc_t = c_int;
    pub type dieFunc_t = c_int;

    pub const MAX_FAILED_NODES: usize = 8;
    pub const MAX_INHAND_WEAPONS: usize = 2;

    pub struct centity_s {
        _placeholder: [u8; 0],
    }
    pub type centity_t = centity_s;

    // !!!!!!!!!!! LOADSAVE-affecting struct !!!!!!!!!!!!!
    #[repr(C)]
    pub struct gentity_s {
        pub s: entityState_t, // communicated by server to clients
        pub client: *mut super::gclient_s, // NULL if not a player (unless it's NPC ( if (this->NPC != NULL)  )  <sigh>... -slc)
        pub inuse: qboolean,
        pub linked: qboolean, // qfalse if not in any good cluster

        pub svFlags: c_int, // SVF_NOCLIENT, SVF_BROADCAST, etc

        pub bmodel: qboolean, // if false, assume an explicit mins / maxs bounding box
                              // only set by gi.SetBrushModel
        pub mins: vec3_t,
        pub maxs: vec3_t,
        pub contents: c_int, // CONTENTS_TRIGGER, CONTENTS_SOLID, CONTENTS_BODY, etc
                             // a non-solid entity should set to 0

        pub absmin: vec3_t,
        pub absmax: vec3_t, // derived from mins/maxs and origin + rotation

        // currentOrigin will be used for all collision detection and world linking.
        // it will not necessarily be the same as the trajectory evaluation for the current
        // time, because each entity must be moved one at a time after time is advanced
        // to avoid simultanious collision issues
        pub currentOrigin: vec3_t,
        pub currentAngles: vec3_t,

        pub owner: *mut gentity_s, // objects never interact with their owners, to
                                    // prevent player missiles from immediately
                                    // colliding with their owner
        /*
        Ghoul2 Insert Start
        */
        // this marker thing of Jake's is used for memcpy() length calcs, so don't put any ordinary fields (like above)
        //	below this point or they won't work, and will mess up all sorts of stuff.
        //
        pub ghoul2: CGhoul2Info_v,

        pub modelScale: vec3_t, //needed for g2 collision
        /*
        Ghoul2 Insert End
        */

        // DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER
        // EXPECTS THE FIELDS IN THAT ORDER!

        //==========================================================================================

        //Essential entity fields
        // note: all the char* fields from here on should be left as ptrs, not declared, because of the way that ent-parsing
        //	works by forcing field offset ptrs as char* and using G_NewString()!! (see G_ParseField() in gmae/g_spawn.cpp -slc
        //
        pub classname: *mut c_char,  // set in QuakeEd
        pub spawnflags: c_int,       // set in QuakeEd

        pub flags: c_int, // FL_* variables

        pub model: *mut c_char,  // Normal model, or legs model on tri-models
        pub model2: *mut c_char, // Torso model

        pub freetime: c_int, // sv.time when the object was freed

        pub eventTime: c_int, // events will be cleared EVENT_VALID_MSEC after set
        pub freeAfterEvent: qboolean,
        //	pub unlinkAfterEvent: qboolean,

        //Physics and movement fields
        pub physicsBounce: f32, // 1.0 = continuous bounce, 0.0 = no bounce
        pub clipmask: c_int,    // brushes with this content value will be collided against
                                // when moving.  items and corpses do not collide against
                                // players, for instance
        //	pub moveInfo: moveInfo_t,		//FIXME: use this more?
        pub speed: f32,
        pub resultspeed: f32,
        pub lastMoveTime: c_int,
        pub movedir: vec3_t,
        pub lastOrigin: vec3_t,  //Where you were last frame
        pub lastAngles: vec3_t,  //Where you were looking last frame
        pub mass: f32,           //How heavy you are
        pub lastImpact: c_int,   //Last time you impacted something

        //Variables reflecting environment
        pub watertype: c_int,
        pub waterlevel: c_int,
        pub wupdate: i16,
        pub prev_waterlevel: i16,

        //Targeting/linking fields
        pub angle: f32,           // set in editor, -1 = up, -2 = down
        pub target: *mut c_char,
        pub target2: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
        pub target3: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
        pub target4: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
        pub targetJump: *mut c_char,
        pub targetname: *mut c_char,
        pub team: *mut c_char,

        pub roff: *mut c_char, // the roff file to use, if there is one
        // fxFile overlay: name of the external effect file

        pub roff_ctr: c_int, // current roff frame we are playing

        pub next_roff_time: c_int,
        pub fx_time: c_int, // timer for beam in/out effects.

        //Think Functions
        pub nextthink: c_int,                      //Used to determine if it's time to call e_ThinkFunc again
        pub e_ThinkFunc: game_include::thinkFunc_t,    //Called once every game frame for every ent
        pub e_clThinkFunc: game_include::clThinkFunc_t, //Think func for equivalent centity
        pub e_ReachedFunc: game_include::reachedFunc_t, // movers call this when hitting endpoint
        pub e_BlockedFunc: game_include::blockedFunc_t,
        pub e_TouchFunc: game_include::touchFunc_t,
        pub e_UseFunc: game_include::useFunc_t, //Called by G_UseTargets
        pub e_PainFunc: game_include::painFunc_t,  //Called by G_Damage when damage is taken
        pub e_DieFunc: game_include::dieFunc_t,   //Called by G_Damage when health reaches <= 0

        //Health and damage fields
        pub health: c_int,
        pub max_health: c_int,
        pub takedamage: qboolean,
        pub material: material_t,
        pub damage: c_int,
        pub dflags: c_int,
        //explosives, breakable brushes
        pub splashDamage: c_int, // quad will increase this without increasing radius
        pub splashRadius: c_int,
        pub methodOfDeath: c_int,
        pub splashMethodOfDeath: c_int,
        //pub hitLoc: c_int,//where you were last hit
        pub locationDamage: [c_int; HL_MAX], // Damage accumulated on different body locations

        //Entity pointers
        pub chain: *mut gentity_s,
        pub enemy: *mut gentity_s,
        pub activator: *mut gentity_s,
        pub teamchain: *mut gentity_s,     // next entity in team
        pub teammaster: *mut gentity_s,    // master of the team
        pub lastEnemy: *mut gentity_s,

        //Timing variables, counters and debounce times
        pub wait: f32,
        pub random: f32,
        pub delay: c_int,
        pub alt_fire: qboolean,
        pub count: c_int,
        pub bounceCount: c_int,
        pub fly_sound_debounce_time: c_int, // wind tunnel
        pub painDebounceTime: c_int,
        pub disconnectDebounceTime: c_int,
        pub attackDebounceTime: c_int,
        pub pushDebounceTime: c_int,
        pub aimDebounceTime: c_int,
        pub useDebounceTime: c_int,

        //Unions for miscellaneous fields used under very specific circumstances
        pub trigger_formation: qboolean,
        // misc_dlight_active overlay
        // has_bounced overlay: for thermal Det.  we force at least one bounce to happen before it can do proximity checks

        //Navigation
        pub spawnContents: c_int,        // store contents of ents on spawn so nav system can restore them
        pub waypoint: c_int,             //Set once per frame, if you've moved, and if someone asks
        pub wayedge: c_int,              //Used by doors and breakable things to know what edge goes through them
        pub lastWaypoint: c_int,         //To make sure you don't double-back
        pub lastInAirTime: c_int,
        pub noWaypointTime: c_int,       //Debouncer - so don't keep checking every waypoint in existance every frame that you can't find one
        pub combatPoint: c_int,
        pub followPos: vec3_t,
        pub followPosRecalcTime: c_int,
        pub followPosWaypoint: c_int,

        //Animation
        pub loopAnim: qboolean,
        pub startFrame: c_int,
        pub endFrame: c_int,

        //Script/ICARUS-related fields
        pub m_iIcarusID: c_int,
        pub taskID: [c_int; NUM_TIDS],
        pub parms: *mut parms_t,
        pub behaviorSet: [*mut c_char; NUM_BSETS],
        pub script_targetname: *mut c_char,
        pub delayScriptTime: c_int,

        // Ambient sound info
        pub soundSet: *mut c_char, //Only used for local sets
        pub setTime: c_int,

        //Used by cameras to locate subjects
        pub cameraGroup: *mut c_char,

        //For damage
        pub noDamageTeam: team_t,

        // Ghoul2 Animation info
        pub playerModel: i16,
        pub weaponModel: [i16; MAX_INHAND_WEAPONS],
        pub handRBolt: i16,
        pub handLBolt: i16,
        pub headBolt: i16,
        pub cervicalBolt: i16,
        pub chestBolt: i16,
        pub gutBolt: i16,
        pub torsoBolt: i16,
        pub crotchBolt: i16,
        pub motionBolt: i16,
        pub kneeLBolt: i16,
        pub kneeRBolt: i16,
        pub elbowLBolt: i16,
        pub elbowRBolt: i16,
        pub footLBolt: i16,
        pub footRBolt: i16,
        pub faceBone: i16,
        pub craniumBone: i16,
        pub cervicalBone: i16,
        pub thoracicBone: i16,
        pub upperLumbarBone: i16,
        pub lowerLumbarBone: i16,
        pub hipsBone: i16,
        pub motionBone: i16,
        pub rootBone: i16,
        pub footLBone: i16,
        pub footRBone: i16,
        pub humerusRBone: i16,

        pub genericBone1: i16,     // For bones special to an entity
        pub genericBone2: i16,
        pub genericBone3: i16,

        pub genericBolt1: i16,     // For bolts special to an entity
        pub genericBolt2: i16,
        pub genericBolt3: i16,
        pub genericBolt4: i16,
        pub genericBolt5: i16,

        pub cinematicModel: qhandle_t,

        //==========================================================================================

        //FIELDS USED EXCLUSIVELY BY SPECIFIC CLASSES OF ENTITIES
        // Vehicle information.
        // The vehicle object.
        pub m_pVehicle: *mut Vehicle_t,

        //NPC/Player entity fields
        //FIXME: Make these client only?
        pub NPC: *mut gNPC_t, //Only allocated if the entity becomes an NPC

        //Other NPC/Player-related entity fields
        pub ownername: *mut c_char, //Used by squadpaths to locate owning NPC

        //FIXME: Only used by NPCs, move it to gNPC_t
        pub cantHitEnemyCounter: c_int, //HACK - Makes them look for another enemy on the same team if the one they're after can't be hit

        //Only used by NPC_spawners
        pub NPC_type: *mut c_char,
        pub NPC_targetname: *mut c_char,
        pub NPC_target: *mut c_char,

        //Variables used by movers (most likely exclusively by them)
        pub moverState: moverState_t,
        pub soundPos1: c_int,
        pub sound1to2: c_int,
        pub sound2to1: c_int,
        pub soundPos2: c_int,
        pub soundLoop: c_int,
        pub nextTrain: *mut gentity_s,
        pub prevTrain: *mut gentity_s,
        pub pos1: vec3_t,
        pub pos2: vec3_t,
        pub pos3: vec3_t,
        pub sounds: c_int,
        pub closetarget: *mut c_char,
        pub opentarget: *mut c_char,
        pub paintarget: *mut c_char,
        pub lockCount: c_int, //for maglocks- actually get put on the trigger for the door

        //Variables used only by waypoints (for the most part)
        pub radius: f32,

        pub wpIndex: c_int,
        // fxID overlay: id of the external effect file

        pub noise_index: c_int,

        pub startRGBA: vec4_t,

        pub finalRGBA: vec4_t,
        // pos4 overlay: vec3_t
        // modelAngles overlay: vec3_t	//for brush entities with an attached md3 model, as an offset to the brush's angles

        //FIXME: Are these being used anymore?
        pub item: *mut gitem_t,    // for bonus items -
        pub message: *mut c_char,  //Used by triggers to print a message when activated

        pub lightLevel: f32,

        //FIXME: can these be removed/condensed/absorbed?
        //Rendering info
        //pub color: c_int,

        //Force effects
        pub forcePushTime: c_int,
        pub forcePuller: c_int, //who force-pulled me (so we don't damage them if we hit them)
    }

    pub type gentity_t = gentity_s;
}

// External declarations
#[cfg(not(feature = "GAME_INCLUDE"))]
extern "C" {
    pub static mut g_entities: [gentity_s; MAX_GENTITIES];
}

#[cfg(feature = "GAME_INCLUDE")]
pub use game_include::gentity_s;

#[cfg(not(target_os = "windows"))]
extern "C" {
    pub static gi: game_import_t;
}

// each WP_* weapon enum has an associated weaponInfo_t
// that contains media references necessary to present the
// weapon and its effects
#[repr(C)]
pub struct weaponInfo_s {
    pub registered: qboolean,
    pub item: *mut gitem_t,

    pub handsModel: qhandle_t,        // the hands don't actually draw, they just position the weapon
    pub weaponModel: qhandle_t,       //for in view
    pub weaponWorldModel: qhandle_t,  //for in their hands
    pub barrelModel: [qhandle_t; 4],

    pub weaponMidpoint: vec3_t, // so it will rotate centered instead of by tag

    pub weaponIcon: qhandle_t,        // The version of the icon with a glowy background
    pub weaponIconNoAmmo: qhandle_t,  // The version of the icon with no ammo warning
    pub ammoIcon: qhandle_t,

    pub ammoModel: qhandle_t,

    pub missileModel: qhandle_t,
    pub missileSound: sfxHandle_t,
    pub missileTrailFunc: Option<unsafe extern "C" fn(*mut centity_t, *const weaponInfo_s)>,

    pub alt_missileModel: qhandle_t,
    pub alt_missileSound: sfxHandle_t,
    pub alt_missileTrailFunc: Option<unsafe extern "C" fn(*mut centity_t, *const weaponInfo_s)>,

    //	pub flashSound: sfxHandle_t,
    //	pub altFlashSound: sfxHandle_t,

    pub firingSound: sfxHandle_t,
    pub altFiringSound: sfxHandle_t,

    pub stopSound: sfxHandle_t,

    pub missileHitSound: sfxHandle_t,
    pub altmissileHitSound: sfxHandle_t,

    pub chargeSound: sfxHandle_t,
    pub altChargeSound: sfxHandle_t,

    pub selectSound: sfxHandle_t, // sound played when weapon is selected

    #[cfg(feature = "_IMMERSION")]
    pub firingForce: ffHandle_t,
    #[cfg(feature = "_IMMERSION")]
    pub altFiringForce: ffHandle_t,
    #[cfg(feature = "_IMMERSION")]
    pub stopForce: ffHandle_t,
    #[cfg(feature = "_IMMERSION")]
    pub chargeForce: ffHandle_t,
    #[cfg(feature = "_IMMERSION")]
    pub altChargeForce: ffHandle_t,
    #[cfg(feature = "_IMMERSION")]
    pub selectForce: ffHandle_t,
}

pub type weaponInfo_t = weaponInfo_s;

#[cfg(not(feature = "GAME_INCLUDE"))]
pub struct centity_s {
    _placeholder: [u8; 0],
}

#[cfg(not(feature = "GAME_INCLUDE"))]
pub type centity_t = centity_s;

extern "C" {
    pub fn CAS_GetBModelSound(name: *const c_char, stage: c_int) -> sfxHandle_t;
}

// Edge type constants
pub const EDGE_NORMAL: c_int = 0;
pub const EDGE_PATH: c_int = 1;
pub const EDGE_BLOCKED: c_int = 2;
pub const EDGE_FAILED: c_int = 3;
pub const EDGE_FLY: c_int = 4;
pub const EDGE_JUMP: c_int = 5;
pub const EDGE_LARGE: c_int = 6;
pub const EDGE_PATHBLOCKED: c_int = 7;
pub const EDGE_NEARESTVALID: c_int = 8;
pub const EDGE_NEARESTINVALID: c_int = 9;

pub const EDGE_NODE_FLOATING: c_int = 10;
pub const EDGE_NODE_NORMAL: c_int = 11;
pub const EDGE_NODE_GOAL: c_int = 12;
pub const EDGE_NODE_COMBAT: c_int = 13;

pub const EDGE_CELL: c_int = 14;
pub const EDGE_CELL_EMPTY: c_int = 15;
pub const EDGE_IMPACT_SAFE: c_int = 16;
pub const EDGE_IMPACT_POSSIBLE: c_int = 17;
pub const EDGE_THRUST: c_int = 18;
pub const EDGE_VELOCITY: c_int = 19;

pub const EDGE_FOLLOWPOS: c_int = 20;

pub const EDGE_WHITE_ONESECOND: c_int = 21;
pub const EDGE_WHITE_TWOSECOND: c_int = 22;
pub const EDGE_RED_ONESECOND: c_int = 23;
pub const EDGE_RED_TWOSECOND: c_int = 24;

// Node type constants
pub const NODE_NORMAL: c_int = 0;
pub const NODE_FLOATING: c_int = 1;
pub const NODE_GOAL: c_int = 2;
pub const NODE_NAVGOAL: c_int = 3;
