//! Partial port of `g_ICARUScb.c` — the ICARUS callback file ("all that can be handled
//! within vm's is handled in here"). The ICARUS scripting bridge itself (the `Q3_*` task
//! interface, register get/set, the `Interpreter_*` glue) is not yet ported, pending the
//! interpreter-side traps land; this file currently holds only the self-contained leaf
//! helpers whose callees are all already ported. `G_DebugPrint` lands first as the
//! developer-only diagnostic printer the rest of the callback layer uses.

#![allow(non_upper_case_globals)] // C module-global names (`textcolor_caption`, …) kept verbatim

use core::ffi::{c_char, c_int, CStr};

use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::b_public_h::{
    bState_t, BS_ADVANCE_FIGHT, BS_DEFAULT, BS_JUMP, BS_NOCLIP, BS_SEARCH, BS_WANDER, JS_FACING,
    NPCAI_CUSTOM_GRAVITY, NPCAI_NO_COLL_AVOID, NPCAI_TOUCHED_GOAL, SCF_WALKING,
};
use crate::codemp::game::bg_public::{
    EF_NODRAW, EF_TELEPORT_BIT, ET_ITEM, ET_MOVER, ET_NPC, EV_GLOBAL_SOUND, MOD_CRUSH,
    MOD_FALLING, MOD_UNKNOWN, PMF_TIME_KNOCKBACK, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
    SETANIM_FLAG_RESTART, SETANIM_LEGS, SETANIM_TORSO, STAT_ARMOR, STAT_HEALTH, STAT_MAX_HEALTH,
    STAT_WEAPONS, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_saga::WPTable;
use crate::codemp::game::g_misc::{TAG_GetAngles, TAG_GetOrigin, TAG_GetOrigin2, TAG_GetRadius};
use crate::codemp::game::npc_combat::{ChangeWeapon, G_ClearEnemy, G_SetEnemy};
use crate::codemp::game::g_client::{SetClientViewAngle, SpotWouldTelefrag2};
use crate::codemp::game::g_combat::{player_die, G_Damage};
use crate::codemp::game::g_mover::{
    G_PlayDoorLoopSound, G_PlayDoorSound, InitMoverTrData, LockDoors, MatchTeam, UnLockDoors,
    BMS_END, BMS_START,
};
use crate::codemp::game::g_local::{
    gentity_s, gentity_t, FL_GODMODE, FL_INACTIVE, FL_NOTARGET, FL_NO_KNOCKBACK, FRAMETIME,
    MOVER_1TO2, MOVER_2TO1, MOVER_POS1, MOVER_POS2,
};
use crate::codemp::game::g_main::{g_developer, g_entities, g_gravity, level as g_level, Com_Printf};
use crate::codemp::game::g_mem::G_Alloc;
use crate::codemp::game::g_nav::{NAV_FindClosestWaypointForEnt, NPC_SetMoveGoal, WAYPOINT_NONE};
use crate::codemp::game::g_public_h::{
    bSet_t, parms_t, BSET_ANGER, BSET_ATTACK, BSET_AWAKE, BSET_BLOCKED, BSET_DEATH, BSET_DELAYED,
    BSET_FFDEATH, BSET_FFIRE, BSET_FLEE, BSET_INVALID, BSET_LOSTENEMY, BSET_MINDTRICK, BSET_PAIN,
    BSET_SPAWN, BSET_USE, BSET_VICTORY, MAX_PARMS, NUM_BSETS, SVF_BROADCAST, SVF_ICARUS_FREEZE,
    SVF_NOCLIENT, SVF_PLAYER_USABLE, TID_ANGLE_FACE, TID_ANIM_BOTH, TID_ANIM_LOWER,
    TID_ANIM_UPPER, TID_BSTATE, TID_CHAN_VOICE, TID_LOCATION, TID_MOVE_NAV, TID_RESIZE,
};
use crate::codemp::game::bg_pmove::BG_SabersOff;
use crate::codemp::game::npc_behavior::NPC_BSSearchStart;
use crate::codemp::game::npc_stats::BSTable;

// Native libc `atoi`/`atof` (the C uses both directly: `atoi((char*)data)`,
// `atof(num_string)`). Bound locally as `extern "C"` — the bg_lib re-implementations are
// `#[cfg(feature = "vm")]`-gated, and the oracle links the same libc, so byte-exactness
// holds (the bg_misc.rs precedent for the spawn-field parsers).
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
}
use crate::codemp::game::g_cmds::Cmd_ToggleSaber_f;
use crate::codemp::game::g_spawn::G_NewString;
use crate::codemp::game::g_utils::{
    G_Find, G_FreeEntity, G_SetAnim, G_SetOrigin, G_Sound, G_SoundIndex, G_Spawn, G_TempEntity,
    G_UseTargets2,
};
use crate::codemp::game::q_math::{AngleDelta, AngleSubtract, VectorClear, VectorCopy, VectorMA};
use crate::codemp::game::q_shared::{
    Com_sprintf, GetIDForString, Q_stricmp, Q_strncmp, Q_strncpyz, Q_strupr, COM_StripExtension,
};
use crate::codemp::game::q_shared_h::{
    qboolean, stringID_table_t, vec3_t, vec4_t, CHAN_AUTO, CHAN_VOICE, MAX_GENTITIES, MAX_QPATH,
    QFALSE, QTRUE, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_STATIONARY,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_CORPSE};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::trap;
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

// For system-wide prints (q_shared.h `enum WL_e`). Defined here rather than in `q_shared_h.rs`
// because — as the C notes — the enum lives in a header the interpreter TU can't include, yet
// `G_DebugPrint` branches on it. `WL_ERROR` is explicitly `=1`; the rest follow.
const WL_ERROR: c_int = 1;
const WL_WARNING: c_int = 2;
const WL_VERBOSE: c_int = 3; // handled by `G_DebugPrint`'s `default`/catch-all arm (C: `default: case WL_VERBOSE:`).
pub const WL_DEBUG: c_int = 4;

// q_shared.h color escapes (S_COLOR_*): "^1" red, "^2" green, "^3" yellow, "^4" blue.
const S_COLOR_RED: &str = "^1";
const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_BLUE: &str = "^4";
const S_COLOR_GREEN: &str = "^2";

/// `void G_DebugPrint( int level, const char *format, ... )` (g_ICARUScb.c:274) — the
/// developer-only ICARUS diagnostic printer. Gated entirely on `g_developer == 2` (the
/// `g_ICARUSDebug` check is `#if 0`'d in the C); otherwise colorizes the message by `level`
/// (ERROR red / WARNING yellow / DEBUG blue / VERBOSE-or-default green) and `Com_Printf`s it.
///
/// As with the rest of the printf family, the C varargs + `vsprintf`-into-`char text[1024]`
/// collapse to a pre-rendered `&str` (callers format with Rust's `format!`) — see
/// `G_Printf`/`Com_Printf`. The `WL_DEBUG` case still does real work on that rendered text:
/// it `sscanf`s the leading entity number, skips the fixed 5-char prefix, and prints the
/// owning entity's `script_targetname`.
///
/// # Safety
/// The `WL_DEBUG` path indexes `g_entities`; the array must be initialised.
pub unsafe fn G_DebugPrint(level: c_int, text: &str) {
    //Don't print messages they don't want to see
    //if ( g_ICARUSDebug->integer < level )
    if (*addr_of!(g_developer)).integer != 2 {
        return;
    }

    // (The C renders the varargs into `char text[1024]` via vsprintf here; the caller
    // pre-renders, so `text` arrives already formatted.)

    //Add the color formatting
    match level {
        WL_ERROR => {
            Com_Printf(&format!("{S_COLOR_RED}ERROR: {text}"));
        }

        WL_WARNING => {
            Com_Printf(&format!("{S_COLOR_YELLOW}WARNING: {text}"));
        }

        WL_DEBUG => {
            let mut entNum: c_int;

            // sscanf( text, "%d", &entNum ) — parse the leading integer.
            entNum = text
                .trim_start()
                .split(|c: char| !(c.is_ascii_digit() || c == '-' || c == '+'))
                .next()
                .and_then(|s| s.parse::<c_int>().ok())
                .unwrap_or(0);

            //if ( ( ICARUS_entFilter >= 0 ) && ( ICARUS_entFilter != entNum ) )
            //	return;

            // buffer = (char *) text; buffer += 5;
            let buffer: &str = text.get(5..).unwrap_or("");

            if entNum < 0 || entNum > MAX_GENTITIES as c_int {
                entNum = 0;
            }

            let script_targetname =
                (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entNum as usize)).script_targetname;
            // glibc printf renders a NULL `%s` as "(null)"; reproduce that for the C's
            // unconditional `printf("%s", ...script_targetname)`.
            let name = if script_targetname.is_null() {
                "(null)".to_string()
            } else {
                core::ffi::CStr::from_ptr(script_targetname)
                    .to_string_lossy()
                    .into_owned()
            };
            Com_Printf(&format!("{S_COLOR_BLUE}DEBUG: {name}({entNum}): {buffer}\n"));
        }

        // default / WL_VERBOSE
        _ => {
            Com_Printf(&format!("{S_COLOR_GREEN}INFO: {text}"));
        }
    }
}

/// `void Q3_TaskIDClear( int *taskID )` (g_ICARUScb.c:269) — resets a task-ID out-param to -1.
///
/// # Safety
/// `taskID` must point to a valid, writable `c_int`.
pub unsafe fn Q3_TaskIDClear(taskID: *mut c_int) {
    *taskID = -1;
}

/// `static char *Q3_GetAnimLower( gentity_t *ent )` (g_ICARUScb.c:330) — returns the name of
/// the entity's current legs animation (NULL if the entity is not a client).
///
/// # Safety
/// `ent` must be a valid entity pointer.
pub unsafe fn Q3_GetAnimLower(ent: *mut gentity_t) -> *mut c_char {
    let anim: c_int;

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_WARNING,
            "Q3_GetAnimLower: attempted to read animation state off non-client!\n",
        );
        return null_mut();
    }

    anim = (*(*ent).client).ps.legsAnim;

    (*addr_of!(animTable))[anim as usize].name as *mut c_char
}

/// `static char *Q3_GetAnimUpper( gentity_t *ent )` (g_ICARUScb.c:350) — returns the name of
/// the entity's current torso animation (NULL if the entity is not a client).
///
/// # Safety
/// `ent` must be a valid entity pointer.
pub unsafe fn Q3_GetAnimUpper(ent: *mut gentity_t) -> *mut c_char {
    let anim: c_int;

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_WARNING,
            "Q3_GetAnimUpper: attempted to read animation state off non-client!\n",
        );
        return null_mut();
    }

    anim = (*(*ent).client).ps.torsoAnim;

    (*addr_of!(animTable))[anim as usize].name as *mut c_char
}

/// `static char *Q3_GetAnimBoth( gentity_t *ent )` (g_ICARUScb.c:370) — returns the legs
/// animation name, warning if either the legs or torso name is NULL (and, under `_DEBUG`, if
/// they disagree). MP is not a `_DEBUG` build, so the mismatch warning is compiled out.
///
/// # Safety
/// `ent` must be a valid entity pointer.
pub unsafe fn Q3_GetAnimBoth(ent: *mut gentity_t) -> *mut c_char {
    let lowerName: *mut c_char = Q3_GetAnimLower(ent);
    let upperName: *mut c_char = Q3_GetAnimUpper(ent);

    if lowerName.is_null() || *lowerName == 0 {
        G_DebugPrint(WL_WARNING, "Q3_GetAnimBoth: NULL legs animation string found!\n");
        return null_mut();
    }

    if upperName.is_null() || *upperName == 0 {
        G_DebugPrint(WL_WARNING, "Q3_GetAnimBoth: NULL torso animation string found!\n");
        return null_mut();
    }

    if Q_stricmp(lowerName, upperName) != 0 {
        // #ifdef _DEBUG	// sigh, cut down on tester reports that aren't important
        //		G_DebugPrint( WL_WARNING, "Q3_GetAnimBoth: legs and torso animations did not match : returning legs\n" );
        // #endif  — MP is not a _DEBUG build, so this print is compiled out.
    }

    lowerName
}

/// `int Q3_PlaySound( int taskID, int entID, const char *name, const char *channel )`
/// (g_ICARUScb.c:399) — plays a sound on an entity over the named channel.
///
/// Uppercases + strips the extension off the requested name, resolves it to a sound index,
/// then: announcer / `target_scriptrunner` ents broadcast; voice channels go through `G_Sound`
/// (skipped entirely if `timescale > 1`) and register a `TID_CHAN_VOICE` task; global-voice
/// broadcasts; everything else plays on `CHAN_AUTO`. The whole subtitle (`ct`/`SV_GentityNum`)
/// block is `/* */`'d out in the C, so it is omitted here. Returns `qtrue` except on the two
/// voice-channel paths (`qtrue` on timescale-skip, `qfalse` otherwise).
///
/// # Safety
/// `g_entities` must be initialised; `name`/`channel` must be valid C strings.
pub unsafe fn Q3_PlaySound(
    taskID: c_int,
    entID: c_int,
    name: *const c_char,
    channel: *const c_char,
) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut finalName = [0 as c_char; MAX_QPATH];
    // set a default so the compiler doesn't bitch
    let mut voice_chan: c_int = CHAN_VOICE;
    let mut type_voice: qboolean = QFALSE;
    let soundHandle: c_int;
    let mut bBroadcast: qboolean;

    Q_strncpyz(finalName.as_mut_ptr(), name, MAX_QPATH as c_int);
    Q_strupr(finalName.as_mut_ptr());
    //G_AddSexToMunroString( finalName, qtrue );

    COM_StripExtension(finalName.as_ptr(), finalName.as_mut_ptr());

    soundHandle = G_SoundIndex(&CStr::from_ptr(finalName.as_ptr()).to_string_lossy());
    bBroadcast = QFALSE;

    if Q_stricmp(channel, c"CHAN_ANNOUNCER".as_ptr()) == 0
        || (!(*ent).classname.is_null()
            && Q_stricmp(c"target_scriptrunner".as_ptr(), (*ent).classname) == 0)
    {
        bBroadcast = QTRUE;
    }

    // moved here from further down so I can easily check channel-type without code dup...
    //
    if Q_stricmp(channel, c"CHAN_VOICE".as_ptr()) == 0 {
        voice_chan = CHAN_VOICE;
        type_voice = QTRUE;
    } else if Q_stricmp(channel, c"CHAN_VOICE_ATTEN".as_ptr()) == 0 {
        voice_chan = CHAN_AUTO; //CHAN_VOICE_ATTEN;
        type_voice = QTRUE;
    } else if Q_stricmp(channel, c"CHAN_VOICE_GLOBAL".as_ptr()) == 0 {
        // this should broadcast to everyone, put only casue animation on G_SoundOnEnt...
        voice_chan = CHAN_AUTO; //CHAN_VOICE_GLOBAL;
        type_voice = QTRUE;
        bBroadcast = QTRUE;
    }

    // (The subtitle `ct`/SV_GentityNum block here is `/* */`'d out in the C — omitted.)

    if type_voice != QFALSE {
        // trap_Cvar_VariableStringBuffer("timescale", buf, sizeof(buf)); tFVal = atof(buf);
        let buf = std::ffi::CString::new(trap::Cvar_VariableString("timescale")).unwrap();
        let tFVal: f32 = atof(buf.as_ptr()) as f32;

        if tFVal > 1.0 {
            //Skip the damn sound!
            return QTRUE;
        } else {
            //This the voice channel
            G_Sound(
                ent,
                voice_chan,
                G_SoundIndex(&CStr::from_ptr(finalName.as_ptr()).to_string_lossy()),
            );
        }
        //Remember we're waiting for this
        trap::ICARUS_TaskIDSet(ent, TID_CHAN_VOICE, taskID);

        return QFALSE;
    }

    if bBroadcast != QFALSE {
        //Broadcast the sound
        let te: *mut gentity_t = G_TempEntity(&(*ent).r.currentOrigin, EV_GLOBAL_SOUND);
        (*te).s.eventParm = soundHandle;
        (*te).r.svFlags |= SVF_BROADCAST;
    } else {
        G_Sound(ent, CHAN_AUTO, soundHandle);
    }

    QTRUE
}

/// `void Q3_Play( int taskID, int entID, const char *type, const char *name )`
/// (g_ICARUScb.c:526) — plays a ROFF on an entity. Only the `"PLAY_ROFF"` type is handled:
/// caches the ROFF, interns its name, snapshots origin/angles into `origin2`/`angles2`, links
/// the entity, registers a `TID_MOVE_NAV` task and starts the ROFF playing.
///
/// # Safety
/// `g_entities` must be initialised; `type`/`name` must be valid C strings.
pub unsafe fn Q3_Play(taskID: c_int, entID: c_int, type_: *const c_char, name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if Q_stricmp(type_, c"PLAY_ROFF".as_ptr()) == 0 {
        // Try to load the requested ROFF
        (*ent).roffid = trap::ROFF_Cache(&CStr::from_ptr(name).to_string_lossy());
        if (*ent).roffid != 0 {
            (*ent).roffname = G_NewString(name);

            // Start the roff from the beginning
            //ent->roff_ctr = 0;

            //Save this off for later
            trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);

            // Let the ROFF playing start.
            //ent->next_roff_time = level.time;

            //rww - Maybe use pos1 and pos2? I don't think we need to care if these values are sent across the net.
            // These need to be initialised up front...
            //VectorCopy( ent->r.currentOrigin, ent->pos1 );
            //VectorCopy( ent->r.currentAngles, ent->pos2 );
            VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.origin2);
            VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.angles2);

            trap::LinkEntity(ent);

            trap::ROFF_Play((*ent).s.number, (*ent).roffid, QTRUE);
        }
    }
}


// =====================================================================================
// "NOT SUPPORTED IN MP" stub family
//
// These ICARUS Q3_Set*/Q3_Camera*/etc. callbacks exist in the SP code path but are
// no-ops in the MP module: each merely emits a developer warning via `G_DebugPrint`
// (the only side effect, gated on `g_developer == 2`) and returns a constant. Ported
// faithfully — same signatures, same warning literals, same return values — so the
// register-set dispatch can call them once it lands. (`Q3_LCARSText`'s warning string
// reads "Q3_ScrollText" verbatim, a copy-paste in the original Raven source.)
// =====================================================================================

/// `void Q3_SetDPitch( int entID, float data )` (g_ICARUScb.c:2607) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDPitch(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDPitch: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDYaw( int entID, float data )` (g_ICARUScb.c:2623) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDYaw(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDYaw: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetShootDist( int entID, float data )` (g_ICARUScb.c:2639) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetShootDist(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetShootDist: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetVisrange( int entID, float data )` (g_ICARUScb.c:2655) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetVisrange(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetVisrange: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetEarshot( int entID, float data )` (g_ICARUScb.c:2671) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetEarshot(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetEarshot: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetVigilance( int entID, float data )` (g_ICARUScb.c:2687) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetVigilance(_entID: c_int, _data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetVigilance: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetVFOV( int entID, int data )` (g_ICARUScb.c:2703) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetVFOV(_entID: c_int, _data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetVFOV: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetHFOV( int entID, int data )` (g_ICARUScb.c:2719) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetHFOV(_entID: c_int, _data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetHFOV: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetWidth( int entID, int data )` (g_ICARUScb.c:2735) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetWidth(_entID: c_int, _data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetWidth: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetVampire( int entID, qboolean vampire )` (g_ICARUScb.c:2803) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetVampire(_entID: c_int, _vampire: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetVampire: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetGreetAllies( int entID, qboolean greet )` (g_ICARUScb.c:2817) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetGreetAllies(_entID: c_int, _greet: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetGreetAllies: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetViewTarget (int entID, const char *name)` (g_ICARUScb.c:2833) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetViewTarget(_entID: c_int, _name: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetViewTarget: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetWatchTarget (int entID, const char *name)` (g_ICARUScb.c:2849) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetWatchTarget(_entID: c_int, _name: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetWatchTarget: NOT SUPPORTED IN MP\n"); }
}

/// `static void Q3_SetWalkSpeed (int entID, int int_data)` (g_ICARUScb.c:3139) — sets the NPC's
/// walk speed (and the client's `ps.speed`). `int_data == 0` is first clamped to 1, then
/// overwritten with `int_data` anyway (faithful to the C).
///
/// # Safety
/// `g_entities` must be initialised; an NPC entity has a non-null `client`.
pub unsafe fn Q3_SetWalkSpeed(entID: c_int, int_data: c_int) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetWalkSpeed: invalid entID {entID}\n"));
        return;
    }

    if (*self_).NPC.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!("Q3_SetWalkSpeed: '{}' is not an NPC!\n", cstr_or_null((*self_).targetname)),
        );
        return;
    }

    if int_data == 0 {
        (*(*self_).NPC).stats.walkSpeed = 1;
        (*(*self_).client).ps.speed = 1.0;
    }

    (*(*self_).NPC).stats.walkSpeed = int_data;
    (*(*self_).client).ps.speed = int_data as f32;
}

/// `static void Q3_SetRunSpeed (int entID, int int_data)` (g_ICARUScb.c:3173) — sets the NPC's
/// run speed (and the client's `ps.speed`). `int_data == 0` is first clamped to 1, then
/// overwritten with `int_data` anyway (faithful to the C).
///
/// # Safety
/// `g_entities` must be initialised; an NPC entity has a non-null `client`.
pub unsafe fn Q3_SetRunSpeed(entID: c_int, int_data: c_int) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetRunSpeed: invalid entID {entID}\n"));
        return;
    }

    if (*self_).NPC.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!("Q3_SetRunSpeed: '{}' is not an NPC!\n", cstr_or_null((*self_).targetname)),
        );
        return;
    }

    if int_data == 0 {
        (*(*self_).NPC).stats.runSpeed = 1;
        (*(*self_).client).ps.speed = 1.0;
    }

    (*(*self_).NPC).stats.runSpeed = int_data;
    (*(*self_).client).ps.speed = int_data as f32;
}

/// `void Q3_SetYawSpeed (int entID, float float_data)` (g_ICARUScb.c:2990) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetYawSpeed(_entID: c_int, _float_data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetYawSpeed: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetAggression(int entID, int int_data)` (g_ICARUScb.c:3006) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetAggression(_entID: c_int, _int_data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAggression: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetAim(int entID, int int_data)` (g_ICARUScb.c:3022) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetAim(_entID: c_int, _int_data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAim: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetShotSpacing(int entID, int int_data)` (g_ICARUScb.c:3116) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetShotSpacing(_entID: c_int, _int_data: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetShotSpacing: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetFollowDist(int entID, float float_data)` (g_ICARUScb.c:3131) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetFollowDist(_entID: c_int, _float_data: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetFollowDist: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetRemoveTarget (int entID, const char *target)` (g_ICARUScb.c:3331) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetRemoveTarget(_entID: c_int, _target: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetRemoveTarget: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetMusicState( const char *dms )` (g_ICARUScb.c:3399) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetMusicState(_dms: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetMusicState: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetForcePowerLevel ( int entID, int forcePower, int forceLevel )` (g_ICARUScb.c:3405) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetForcePowerLevel(_entID: c_int, _forcePower: c_int, _forceLevel: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetForcePowerLevel: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetCaptureGoal( int entID, const char *name )` (g_ICARUScb.c:3470) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetCaptureGoal(_entID: c_int, _name: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetCaptureGoal: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetIgnorePain( int entID, qboolean data)` (g_ICARUScb.c:3496) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetIgnorePain(_entID: c_int, _data: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetIgnorePain: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetIgnoreEnemies( int entID, qboolean data)` (g_ICARUScb.c:3509) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetIgnoreEnemies(_entID: c_int, _data: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetIgnoreEnemies: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetIgnoreAlerts( int entID, qboolean data)` (g_ICARUScb.c:3522) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetIgnoreAlerts(_entID: c_int, _data: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetIgnoreAlerts: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDontShoot( int entID, qboolean add)` (g_ICARUScb.c:3559) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDontShoot(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDontShoot: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDontFire( int entID, qboolean add)` (g_ICARUScb.c:3572) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDontFire(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDontFire: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetFireWeapon(int entID, qboolean add)` (g_ICARUScb.c:3585) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetFireWeapon(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetFireWeapon: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetLockedEnemy ( int entID, qboolean locked)` (g_ICARUScb.c:3658) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetLockedEnemy(_entID: c_int, _locked: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetLockedEnemy: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetCinematicSkipScript( char *scriptname )` (g_ICARUScb.c:3672) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetCinematicSkipScript(_scriptname: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetCinematicSkipScript: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoMindTrick( int entID, qboolean add)` (g_ICARUScb.c:3685) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoMindTrick(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoMindTrick: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetCrouched( int entID, qboolean add)` (g_ICARUScb.c:3698) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetCrouched(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetCrouched: NOT SUPPORTED IN MP\n"); }
}

/// `static void Q3_SetWalking( int entID, qboolean add)` (g_ICARUScb.c:3942) — toggles the
/// `SCF_WALKING` script flag on the NPC.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetWalking(entID: c_int, add: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetWalking: invalid entID {entID}\n"));
        return;
    }

    if (*ent).NPC.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!("Q3_SetWalking: '{}' is not an NPC!\n", cstr_or_null((*ent).targetname)),
        );
        return;
    }

    if add != QFALSE {
        (*(*ent).NPC).scriptFlags |= SCF_WALKING;
    } else {
        (*(*ent).NPC).scriptFlags &= !SCF_WALKING;
    }
}

/// `void Q3_SetRunning( int entID, qboolean add)` (g_ICARUScb.c:3724) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetRunning(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetRunning: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetForcedMarch( int entID, qboolean add)` (g_ICARUScb.c:3737) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetForcedMarch(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetForcedMarch: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetChaseEnemies( int entID, qboolean add)` (g_ICARUScb.c:3749) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetChaseEnemies(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetChaseEnemies: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetLookForEnemies( int entID, qboolean add)` (g_ICARUScb.c:3763) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetLookForEnemies(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetLookForEnemies: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetFaceMoveDir( int entID, qboolean add)` (g_ICARUScb.c:3775) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetFaceMoveDir(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetFaceMoveDir: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetAltFire( int entID, qboolean add)` (g_ICARUScb.c:3788) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetAltFire(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAltFire: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDontFlee( int entID, qboolean add)` (g_ICARUScb.c:3801) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDontFlee(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDontFlee: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoResponse( int entID, qboolean add)` (g_ICARUScb.c:3814) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoResponse(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoResponse: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetCombatTalk( int entID, qboolean add)` (g_ICARUScb.c:3827) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetCombatTalk(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetCombatTalk: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetAlertTalk( int entID, qboolean add)` (g_ICARUScb.c:3840) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetAlertTalk(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAlertTalk: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetUseCpNearest( int entID, qboolean add)` (g_ICARUScb.c:3853) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetUseCpNearest(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetUseCpNearest: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoForce( int entID, qboolean add)` (g_ICARUScb.c:3866) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoForce(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoForce: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoAcrobatics( int entID, qboolean add)` (g_ICARUScb.c:3879) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoAcrobatics(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoAcrobatics: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetUseSubtitles( int entID, qboolean add)` (g_ICARUScb.c:3892) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetUseSubtitles(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetUseSubtitles: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoFallToDeath( int entID, qboolean add)` (g_ICARUScb.c:3905) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoFallToDeath(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoFallToDeath: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDismemberable( int entID, qboolean dismemberable)` (g_ICARUScb.c:3918) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDismemberable(_entID: c_int, _dismemberable: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDismemberable: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetMoreLight( int entID, qboolean add )` (g_ICARUScb.c:3932) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetMoreLight(_entID: c_int, _add: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetMoreLight: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetUndying( int entID, qboolean undying)` (g_ICARUScb.c:3945) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetUndying(_entID: c_int, _undying: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetUndying: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetInvincible( int entID, qboolean invincible)` (g_ICARUScb.c:3958) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetInvincible(_entID: c_int, _invincible: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetInvicible: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetForceInvincible( int entID, qboolean forceInv )` (g_ICARUScb.c:3972) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetForceInvincible(_entID: c_int, _forceInv: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetForceInvicible: NOT SUPPORTED IN MP\n"); }
}

/// `static void Q3_SetNoAvoid( int entID, qboolean noAvoid)` (g_ICARUScb.c:4237) — toggles the
/// `NPCAI_NO_COLL_AVOID` AI flag on the NPC.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetNoAvoid(entID: c_int, noAvoid: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetNoAvoid: invalid entID {entID}\n"));
        return;
    }

    if (*ent).NPC.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!("Q3_SetNoAvoid: '{}' is not an NPC!\n", cstr_or_null((*ent).targetname)),
        );
        return;
    }

    if noAvoid != QFALSE {
        (*(*ent).NPC).aiFlags |= NPCAI_NO_COLL_AVOID;
    } else {
        (*(*ent).NPC).aiFlags &= !NPCAI_NO_COLL_AVOID;
    }
}

/// `void Q3_CameraGroup( int entID, char *camG)` (g_ICARUScb.c:4183) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_CameraGroup(_entID: c_int, _camG: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_CameraGroup: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_CameraGroupZOfs( float camGZOfs )` (g_ICARUScb.c:4196) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_CameraGroupZOfs(_camGZOfs: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_CameraGroupZOfs: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_CameraGroupTag( char *camGTag )` (g_ICARUScb.c:4208) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_CameraGroupTag(_camGTag: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_CameraGroupTag: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_RemoveRHandModel( int entID, char *addModel)` (g_ICARUScb.c:4219) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_RemoveRHandModel(_entID: c_int, _addModel: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_RemoveRHandModel: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_AddRHandModel( int entID, char *addModel)` (g_ICARUScb.c:4229) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_AddRHandModel(_entID: c_int, _addModel: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_AddRHandModel: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_AddLHandModel( int entID, char *addModel)` (g_ICARUScb.c:4239) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_AddLHandModel(_entID: c_int, _addModel: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_AddLHandModel: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_RemoveLHandModel( int entID, char *addModel)` (g_ICARUScb.c:4249) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_RemoveLHandModel(_entID: c_int, _addModel: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_RemoveLHandModel: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_LookTarget( int entID, char *targetName)` (g_ICARUScb.c:4261) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_LookTarget(_entID: c_int, _targetName: *mut c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_LookTarget: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_Face( int entID,int expression, float holdtime)` (g_ICARUScb.c:4274) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_Face(_entID: c_int, _expression: c_int, _holdtime: f32) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_Face: NOT SUPPORTED IN MP\n"); }
}

/// `qboolean Q3_SetLocation( int entID, const char *location )` (g_ICARUScb.c:4288) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetLocation(_entID: c_int, _location: *const c_char) -> qboolean {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetLocation: NOT SUPPORTED IN MP\n"); }
    QTRUE
}

/// `void Q3_SetPlayerLocked( int entID, qboolean locked )` (g_ICARUScb.c:4304) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetPlayerLocked(_entID: c_int, _locked: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetPlayerLocked: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetLockPlayerWeapons( int entID, qboolean locked )` (g_ICARUScb.c:4318) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetLockPlayerWeapons(_entID: c_int, _locked: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetLockPlayerWeapons: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetNoImpactDamage( int entID, qboolean noImp )` (g_ICARUScb.c:4333) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetNoImpactDamage(_entID: c_int, _noImp: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetNoImpactDamage: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDelayScriptTime(int entID, int delayTime)` (g_ICARUScb.c:4445) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDelayScriptTime(_entID: c_int, _delayTime: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDelayScriptTime: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetDisableShaderAnims( int entID, int disabled )` (g_ICARUScb.c:4491) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetDisableShaderAnims(_entID: c_int, _disabled: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetDisableShaderAnims: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetShaderAnim( int entID, int disabled )` (g_ICARUScb.c:4506) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetShaderAnim(_entID: c_int, _disabled: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetShaderAnim: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetStartFrame( int entID, int startFrame )` (g_ICARUScb.c:4521) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetStartFrame(_entID: c_int, _startFrame: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetStartFrame: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetEndFrame( int entID, int endFrame )` (g_ICARUScb.c:4536) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetEndFrame(_entID: c_int, _endFrame: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetEndFrame: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetAnimFrame( int entID, int animFrame )` (g_ICARUScb.c:4550) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetAnimFrame(_entID: c_int, _animFrame: c_int) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAnimFrame: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetLoopAnim( int entID, qboolean loopAnim )` (g_ICARUScb.c:4564) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetLoopAnim(_entID: c_int, _loopAnim: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetLoopAnim: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetShields( int entID, qboolean shields )` (g_ICARUScb.c:4579) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetShields(_entID: c_int, _shields: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetShields: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetCleanDamagingEnts( void )` (g_ICARUScb.c:4655) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetCleanDamagingEnts() {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetCleanDamagingEnts: NOT SUPPORTED IN MP\n"); }
}

// `vec4_t textcolor_caption; vec4_t textcolor_center; vec4_t textcolor_scroll;`
// (g_ICARUScb.c:4661) — module-global text colours passed to the no-op `SetTextColor`.
// They are never read in MP (`SetTextColor` is a NOT-SUPPORTED warning), but exist so the
// Q3_Set*TextColor callbacks below mirror the C exactly.
static mut textcolor_caption: vec4_t = [0.0; 4];
static mut textcolor_center: vec4_t = [0.0; 4];
static mut textcolor_scroll: vec4_t = [0.0; 4];

/// `void SetTextColor ( vec4_t textcolor,const char *color)` (g_ICARUScb.c:4670) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn SetTextColor(_textcolor: vec4_t, _color: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "SetTextColor: NOT SUPPORTED IN MP\n"); }
}

/// `static void Q3_SetCaptionTextColor ( const char *color)` (g_ICARUScb.c:4683) — change the
/// colour caption text prints in (forwards to the no-op [`SetTextColor`]).
///
/// # Safety
/// `color` must be a valid C string.
pub unsafe fn Q3_SetCaptionTextColor(color: *const c_char) {
    SetTextColor(*addr_of!(textcolor_caption), color);
}

/// `static void Q3_SetCenterTextColor ( const char *color)` (g_ICARUScb.c:4695) — change the
/// colour center text prints in (forwards to the no-op [`SetTextColor`]).
///
/// # Safety
/// `color` must be a valid C string.
pub unsafe fn Q3_SetCenterTextColor(color: *const c_char) {
    SetTextColor(*addr_of!(textcolor_center), color);
}

/// `static void Q3_SetScrollTextColor ( const char *color)` (g_ICARUScb.c:4707) — change the
/// colour scroll text prints in (forwards to the no-op [`SetTextColor`]).
///
/// # Safety
/// `color` must be a valid C string.
pub unsafe fn Q3_SetScrollTextColor(color: *const c_char) {
    SetTextColor(*addr_of!(textcolor_scroll), color);
}

/// `void Q3_ScrollText ( const char *id)` (g_ICARUScb.c:4719) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_ScrollText(_id: *const c_char) {
    //trap_SendServerCommand( -1, va("st \"%s\"", id));
    unsafe { G_DebugPrint(WL_WARNING, "Q3_ScrollText: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_LCARSText ( const char *id)` (g_ICARUScb.c:4734) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_LCARSText(_id: *const c_char) {
    //trap_SendServerCommand( -1, va("lt \"%s\"", id));
    unsafe { G_DebugPrint(WL_WARNING, "Q3_ScrollText: NOT SUPPORTED IN MP\n"); }
}


// =====================================================================================
// Entity-field setters / leaf callbacks — the portable vein of the ICARUS callback layer.
//
// These mirror the C control flow verbatim (`gentity_t *self = &g_entities[entID]` then a
// field write), calling only already-ported helpers (G_Find/G_DebugPrint/Q_stricmp/
// G_NewString/G_SetOrigin/…). All entity-state mutators, hence no-oracle.  The dispatch
// (`Q3_Set`/`Q3_Get*`) and the mover-lerp/think-callback families stay unported (they need
// the ICARUS `setTable`/`SET_*` register enum and mover-trajectory state respectively).
// =====================================================================================


/// `void Q3_Use( int entID, const char *target )` (g_ICARUScb.c:984) — uses an entity by
/// firing its targets.
///
/// # Safety
/// `g_entities` must be initialised; `target` may be NULL (handled).
pub unsafe fn Q3_Use(entID: c_int, target: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    // C: if ( !ent ) — &g_entities[entID] is never NULL, but mirror the check faithfully.
    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Use: invalid entID {entID}\n"));
        return;
    }

    if target.is_null() || *target == 0 {
        G_DebugPrint(WL_WARNING, "Q3_Use: string is NULL!\n");
        return;
    }

    G_UseTargets2(ent, ent, target);
}

/// `void Q3_Kill( int entID, const char *name )` (g_ICARUScb.c:1012) — zeroes a target's
/// health and invokes its die callback.
///
/// # Safety
/// `g_entities` must be initialised; calls the target's `die` fn-pointer.
pub unsafe fn Q3_Kill(entID: c_int, name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let victim: *mut gentity_t;

    if Q_stricmp(name, c"self".as_ptr()) == 0 {
        victim = ent;
    } else if Q_stricmp(name, c"enemy".as_ptr()) == 0 {
        victim = (*ent).enemy;
    } else {
        victim = G_Find(null_mut(), core::mem::offset_of!(gentity_t, targetname), name);
    }

    if victim.is_null() {
        G_DebugPrint(
            WL_WARNING,
            &format!("Q3_Kill: can't find {}\n", CStr::from_ptr(name).to_string_lossy()),
        );
        return;
    }

    //rww - I guess this would only apply to NPCs anyway. I'm not going to bother.
    //if ( victim == ent )
    //{//don't ICARUS_FreeEnt me, I'm in the middle of a script!  (FIXME: shouldn't ICARUS handle this internally?)
    //	victim->svFlags |= SVF_KILLED_SELF;
    //}

    let o_health = (*victim).health;
    (*victim).health = 0;
    if !(*victim).client.is_null() {
        (*victim).flags |= FL_NO_KNOCKBACK;
    }
    //G_SetEnemy(victim, ent);
    if let Some(die) = (*victim).die {
        // check can be omitted
        //GEntity_DieFunc( victim, NULL, NULL, o_health, MOD_UNKNOWN );
        die(victim, victim, victim, o_health, MOD_UNKNOWN);
    }
}

/// `void Q3_RemoveEnt( gentity_t *victim )` (g_ICARUScb.c:1065) — schedules an entity for
/// removal (clients can't be removed in MP — only NPCs, by re-thinking into `G_FreeEntity`).
///
/// # Safety
/// `victim` must be a valid entity pointer; `level` must be initialised.
pub unsafe fn Q3_RemoveEnt(victim: *mut gentity_t) {
    if !(*victim).client.is_null() {
        if (*victim).s.eType != ET_NPC {
            G_DebugPrint(WL_WARNING, "Q3_RemoveEnt: You can't remove clients in MP!\n");
            debug_assert!(false); //can't remove clients in MP
        } else {
            //remove the NPC
            if (*(*victim).client).NPC_class == CLASS_VEHICLE {
                //eject everyone out of a vehicle that's about to remove itself
                let pVeh = (*victim).m_pVehicle;
                if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
                    if let Some(eject_all) = (*(*pVeh).m_pVehicleInfo).EjectAll {
                        eject_all(pVeh);
                    }
                }
            }
            (*victim).think = Some(G_FreeEntity);
            (*victim).nextthink = (*addr_of!(g_level)).time + 100;
        }
        /*
        //ClientDisconnect(ent);
        ...
        */
    } else {
        (*victim).think = Some(G_FreeEntity);
        (*victim).nextthink = (*addr_of!(g_level)).time + 100;
    }
}

/// `void Q3_Remove( int entID, const char *name )` (g_ICARUScb.c:1131) — removes the entity(s)
/// named `name` (`self`/`enemy`/all matching targetname) via [`Q3_RemoveEnt`].
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_Remove(entID: c_int, name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut victim: *mut gentity_t;

    if Q_stricmp(c"self".as_ptr(), name) == 0 {
        victim = ent;
        if victim.is_null() {
            G_DebugPrint(
                WL_WARNING,
                &format!("Q3_Remove: can't find {}\n", CStr::from_ptr(name).to_string_lossy()),
            );
            return;
        }
        Q3_RemoveEnt(victim);
    } else if Q_stricmp(c"enemy".as_ptr(), name) == 0 {
        victim = (*ent).enemy;
        if victim.is_null() {
            G_DebugPrint(
                WL_WARNING,
                &format!("Q3_Remove: can't find {}\n", CStr::from_ptr(name).to_string_lossy()),
            );
            return;
        }
        Q3_RemoveEnt(victim);
    } else {
        victim = G_Find(null_mut(), core::mem::offset_of!(gentity_t, targetname), name);
        if victim.is_null() {
            G_DebugPrint(
                WL_WARNING,
                &format!("Q3_Remove: can't find {}\n", CStr::from_ptr(name).to_string_lossy()),
            );
            return;
        }

        while !victim.is_null() {
            Q3_RemoveEnt(victim);
            victim = G_Find(victim, core::mem::offset_of!(gentity_t, targetname), name);
        }
    }
}

/// `void Q3_SetLoopSound(int entID, const char *name)` (g_ICARUScb.c:2855) — sets/clears the
/// entity's looping sound from a sound index.
///
/// # Safety
/// `g_entities` must be initialised; `name` must be a valid C string.
pub unsafe fn Q3_SetLoopSound(entID: c_int, name: *const c_char) {
    let index: c_int;
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if Q_stricmp(c"NULL".as_ptr(), name) == 0 || Q_stricmp(c"NONE".as_ptr(), name) == 0 {
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
        return;
    }

    index = G_SoundIndex(&CStr::from_ptr(name).to_string_lossy());

    if index != 0 {
        (*self_).s.loopSound = index;
        (*self_).s.loopIsSoundset = QFALSE;
    } else {
        G_DebugPrint(
            WL_WARNING,
            &format!(
                "Q3_SetLoopSound: can't find sound file: '{}'\n",
                CStr::from_ptr(name).to_string_lossy()
            ),
        );
    }
}

/// `void Q3_SetICARUSFreeze( int entID, const char *name, qboolean freeze )`
/// (g_ICARUScb.c:2880) — sets/clears the SVF_ICARUS_FREEZE flag on the named entity.
///
/// # Safety
/// `g_entities` must be initialised; `name` must be a valid C string.
pub unsafe fn Q3_SetICARUSFreeze(_entID: c_int, name: *const c_char, freeze: qboolean) {
    let mut self_: *mut gentity_t =
        G_Find(null_mut(), core::mem::offset_of!(gentity_t, targetname), name);
    if self_.is_null() {
        //hmm, targetname failed, try script_targetname?
        self_ = G_Find(null_mut(), core::mem::offset_of!(gentity_t, script_targetname), name);
    }

    if self_.is_null() {
        G_DebugPrint(
            WL_WARNING,
            &format!(
                "Q3_SetICARUSFreeze: invalid ent {}\n",
                CStr::from_ptr(name).to_string_lossy()
            ),
        );
        return;
    }

    if freeze != QFALSE {
        (*self_).r.svFlags |= SVF_ICARUS_FREEZE;
    } else {
        (*self_).r.svFlags &= !SVF_ICARUS_FREEZE;
    }
}

/// `void Q3_SetViewEntity(int entID, const char *name)` (g_ICARUScb.c:2913) — currently
/// unsupported in MP: warns and no-ops.
pub fn Q3_SetViewEntity(_entID: c_int, _name: *const c_char) {
    unsafe {
        G_DebugPrint(WL_WARNING, "Q3_SetViewEntity currently unsupported in MP, ask if you need it.\n");
    }
}

/// `static void Q3_SetWeapon (int entID, const char *wp_name)` (g_ICARUScb.c:3104) — resolves
/// the weapon name through [`WPTable`], grants it (`ps.stats[STAT_WEAPONS] = 1<<wp`), and
/// switches to it via [`ChangeWeapon`].
///
/// # Safety
/// `g_entities` must be initialised; the entity must be a client.
pub unsafe fn Q3_SetWeapon(entID: c_int, wp_name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let wp: c_int = GetIDForString(addr_of!(WPTable) as *const stringID_table_t, wp_name);

    (*(*ent).client).ps.stats[STAT_WEAPONS as usize] = 1 << wp;
    ChangeWeapon(ent, wp);
}

/// `void Q3_SetItem (int entID, const char *item_name)` (g_ICARUScb.c:2941) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetItem(_entID: c_int, _item_name: *const c_char) {
    //rww - unused in mp
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetItem: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetTarget2 (int entID, const char *target2)` (g_ICARUScb.c:3300) — does not exist in MP: warns and no-ops.
pub fn Q3_SetTarget2(_entID: c_int, _target2: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetTarget2 does not exist in MP\n"); }
}

/// `void Q3_SetPainTarget (int entID, const char *targetname)` (g_ICARUScb.c:3347) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetPainTarget(_entID: c_int, _targetname: *const c_char) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetPainTarget: NOT SUPPORTED IN MP\n"); }
}

/// `void Q3_SetEvent( int entID, const char *event_name )` (g_ICARUScb.c:3483) — NOT SUPPORTED IN MP: warns and no-ops.
pub fn Q3_SetEvent(_entID: c_int, _event_name: *const c_char) {
    //rwwFIXMEFIXME: Use in MP?
    unsafe {
        G_DebugPrint(WL_WARNING, "Q3_SetEvent: NOT SUPPORTED IN MP (may be in future, ask if needed)\n");
    }
}

/// `void Q3_SetAnimHoldTime( int entID, int int_data, qboolean lower )` (g_ICARUScb.c:2273) —
/// not currently supported in MP: warns and no-ops (body `#if`'d out in the C).
pub fn Q3_SetAnimHoldTime(_entID: c_int, _int_data: c_int, _lower: qboolean) {
    unsafe { G_DebugPrint(WL_WARNING, "Q3_SetAnimHoldTime is not currently supported in MP\n"); }
}

/// `static qboolean Q3_SetTeleportDest( int entID, vec3_t org )` (g_ICARUScb.c:1898) — copies
/// `org` onto the script-running entity once nothing is blocking the spot. If the spot is
/// occupied it spawns a [`MoveOwner`] helper to keep retrying and returns `qfalse`; otherwise
/// it places the entity immediately and returns `qtrue`.
///
/// # Safety
/// `g_entities`/`level` must be initialised; may spawn an entity.
pub unsafe fn Q3_SetTeleportDest(entID: c_int, org: &vec3_t) -> qboolean {
    let teleEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if !teleEnt.is_null() {
        if SpotWouldTelefrag2(teleEnt, org) != QFALSE {
            let teleporter: *mut gentity_t = G_Spawn();

            G_SetOrigin(teleporter, org);
            (*teleporter).r.ownerNum = (*teleEnt).s.number;

            (*teleporter).think = Some(MoveOwner);
            (*teleporter).nextthink = (*addr_of!(g_level)).time + FRAMETIME;

            return QFALSE;
        } else {
            G_SetOrigin(teleEnt, org);
        }
    }

    QTRUE
}

/// `void Q3_SetOrigin( int entID, vec3_t origin )` (g_ICARUScb.c:1932) — sets the origin of an
/// entity directly (clients get teleported with a velocity clear; others via `G_SetOrigin`).
///
/// # Safety
/// `g_entities` must be initialised; relinks the entity.
pub unsafe fn Q3_SetOrigin(entID: c_int, origin: &vec3_t) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetOrigin: bad ent {entID}\n"));
        return;
    }

    trap::UnlinkEntity(ent);

    if !(*ent).client.is_null() {
        VectorCopy(origin, &mut (*(*ent).client).ps.origin);
        VectorCopy(origin, &mut (*ent).r.currentOrigin);
        (*(*ent).client).ps.origin[2] += 1.0;

        VectorClear(&mut (*(*ent).client).ps.velocity);
        (*(*ent).client).ps.pm_time = 160; // hold time
        (*(*ent).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;

        (*(*ent).client).ps.eFlags ^= EF_TELEPORT_BIT;

    //		G_KillBox (ent);
    } else {
        G_SetOrigin(ent, origin);
    }

    trap::LinkEntity(ent);
}

/// `void Q3_SetCopyOrigin( int entID, const char *name )` (g_ICARUScb.c:1973) — copies the
/// found entity's origin (and view angle) onto the script-running entity.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetCopyOrigin(entID: c_int, name: *const c_char) {
    let found: *mut gentity_t =
        G_Find(null_mut(), core::mem::offset_of!(gentity_t, targetname), name);

    if !found.is_null() {
        Q3_SetOrigin(entID, &(*found).r.currentOrigin);
        SetClientViewAngle((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize), &(*found).s.angles);
    } else {
        G_DebugPrint(
            WL_WARNING,
            &format!(
                "Q3_SetCopyOrigin: ent {} not found!\n",
                CStr::from_ptr(name).to_string_lossy()
            ),
        );
    }
}

/// `void Q3_SetVelocity( int entID, int axis, float speed )` (g_ICARUScb.c:1995) — adds to a
/// client's velocity along an axis.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetVelocity(entID: c_int, axis: c_int, speed: f32) {
    let found: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    //FIXME: Not supported
    if found.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetVelocity invalid entID {entID}\n"));
        return;
    }

    if (*found).client.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetVelocity: not a client {entID}\n"));
        return;
    }

    //FIXME: add or set?
    (*(*found).client).ps.velocity[axis as usize] += speed;

    (*(*found).client).ps.pm_time = 500;
    (*(*found).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
}

/// `void Q3_SetAngles( int entID, vec3_t angles )` (g_ICARUScb.c:2025) — sets the angles of an
/// entity directly.
///
/// # Safety
/// `g_entities` must be initialised; relinks the entity.
pub unsafe fn Q3_SetAngles(entID: c_int, angles: &vec3_t) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetAngles: bad ent {entID}\n"));
        return;
    }

    if !(*ent).client.is_null() {
        SetClientViewAngle(ent, angles);
    } else {
        VectorCopy(angles, &mut (*ent).s.angles);
    }
    trap::LinkEntity(ent);
}

/// `static void Q3_SetOriginOffset( int entID, int axis, float offset )` (g_ICARUScb.c:2117) —
/// offsets a mover's origin along `axis` by `offset`, lerping there over a duration derived from
/// the entity's `speed` (via [`Q3_Lerp2Origin`]).
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn Q3_SetOriginOffset(entID: c_int, axis: c_int, offset: f32) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut origin: vec3_t = [0.0; 3];
    let mut duration: f32;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetOriginOffset: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_SetOriginOffset: ent {entID} is NOT a mover!\n"));
        return;
    }

    VectorCopy(&(*ent).s.origin, &mut origin);
    origin[axis as usize] += offset;
    duration = 0.0;
    if (*ent).speed != 0.0 {
        // C: duration = fabs(offset)/fabs(ent->speed)*1000.0f;  (computed in double)
        duration = ((offset as f64).abs() / ((*ent).speed as f64).abs() * 1000.0) as f32;
    }
    Q3_Lerp2Origin(-1, entID, &origin, duration);
}

/// `static void SetLowerAnim( int entID, int animID)` (g_ICARUScb.c:2154) — sets the legs anim.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn SetLowerAnim(entID: c_int, animID: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("SetLowerAnim: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(WL_ERROR, &format!("SetLowerAnim: ent {entID} is NOT a player or NPC!\n"));
        return;
    }

    G_SetAnim(
        ent,
        null_mut(),
        SETANIM_LEGS,
        animID,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        0,
    );
}

/// `static void SetUpperAnim ( int entID, int animID)` (g_ICARUScb.c:2182) — sets the torso anim.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn SetUpperAnim(entID: c_int, animID: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("SetUpperAnim: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        // (C: the warning literal reads "SetLowerAnim" here — a copy-paste in the Raven source.)
        G_DebugPrint(WL_ERROR, &format!("SetLowerAnim: ent {entID} is NOT a player or NPC!\n"));
        return;
    }

    G_SetAnim(
        ent,
        null_mut(),
        SETANIM_TORSO,
        animID,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        0,
    );
}

/// `static qboolean Q3_SetAnimUpper( int entID, const char *anim_name )` (g_ICARUScb.c:2208) —
/// sets the upper (torso) animation of an entity by name.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetAnimUpper(entID: c_int, anim_name: *const c_char) -> qboolean {
    let animID: c_int = GetIDForString(addr_of!(animTable) as *const stringID_table_t, anim_name);

    if animID == -1 {
        G_DebugPrint(
            WL_WARNING,
            &format!(
                "Q3_SetAnimUpper: unknown animation sequence '{}'\n",
                CStr::from_ptr(anim_name).to_string_lossy()
            ),
        );
        return QFALSE;
    }

    /*
    if ( !PM_HasAnimation( SV_GentityNum(entID), animID ) )
    {
        return qfalse;
    }
    */

    SetUpperAnim(entID, animID);
    QTRUE
}

/// `static qboolean Q3_SetAnimLower( int entID, const char *anim_name )` (g_ICARUScb.c:2238) —
/// sets the lower (legs) animation of an entity by name.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetAnimLower(entID: c_int, anim_name: *const c_char) -> qboolean {
    //FIXME: Setting duck anim does not actually duck!

    let animID: c_int = GetIDForString(addr_of!(animTable) as *const stringID_table_t, anim_name);

    if animID == -1 {
        G_DebugPrint(
            WL_WARNING,
            &format!(
                "Q3_SetAnimLower: unknown animation sequence '{}'\n",
                CStr::from_ptr(anim_name).to_string_lossy()
            ),
        );
        return QFALSE;
    }

    /*
    if ( !PM_HasAnimation( SV_GentityNum(entID), animID ) )
    {
        return qfalse;
    }
    */

    SetLowerAnim(entID, animID);
    QTRUE
}

/// `static void Q3_SetHealth( int entID, int data )` (g_ICARUScb.c:2311) — sets an entity's
/// health (and client STAT_HEALTH, with a `player_die` when set to 0).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetHealth(entID: c_int, mut data: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetHealth: invalid entID {entID}\n"));
        return;
    }

    if data < 0 {
        data = 0;
    }

    (*ent).health = data;

    if (*ent).client.is_null() {
        return;
    }

    (*(*ent).client).ps.stats[STAT_HEALTH as usize] = data;

    if (*(*ent).client).ps.stats[STAT_HEALTH as usize]
        > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize]
    {
        let mh = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
        (*(*ent).client).ps.stats[STAT_HEALTH as usize] = mh;
        (*ent).health = mh;
    }
    if data == 0 {
        (*ent).health = 1;
        if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
            //this would be silly
            return;
        }

        (*ent).flags &= !FL_GODMODE;
        (*(*ent).client).ps.stats[STAT_HEALTH as usize] = -999;
        (*ent).health = -999;
        player_die(ent, ent, ent, 100000, MOD_FALLING);
    }
}

/// `static void Q3_SetArmor( int entID, int data )` (g_ICARUScb.c:2363) — sets a client's
/// STAT_ARMOR (clamped to STAT_MAX_HEALTH).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetArmor(entID: c_int, data: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetArmor: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        return;
    }

    (*(*ent).client).ps.stats[STAT_ARMOR as usize] = data;
    if (*(*ent).client).ps.stats[STAT_ARMOR as usize]
        > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize]
    {
        (*(*ent).client).ps.stats[STAT_ARMOR as usize] =
            (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
    }
}

/// `static void Q3_SetFriction(int entID, int int_data)` (g_ICARUScb.c:3038) — validates a
/// client then warns (the body is `#if`'d out in the C — unsupported in MP).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetFriction(entID: c_int, _int_data: c_int) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetFriction: invalid entID {entID}\n"));
        return;
    }

    if (*self_).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetFriction: '{}' is not an NPC/player!\n",
                cstr_or_null((*self_).targetname)
            ),
        );
        return;
    }

    G_DebugPrint(WL_WARNING, "Q3_SetFriction currently unsupported in MP\n");
    //	self->client->ps.friction = int_data;
}

/// `static void Q3_SetGravity(int entID, float float_data)` (g_ICARUScb.c:3068) — sets a
/// client's gravity.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetGravity(entID: c_int, float_data: f32) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetGravity: invalid entID {entID}\n"));
        return;
    }

    if (*self_).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetGravity: '{}' is not an NPC/player!\n",
                cstr_or_null((*self_).targetname)
            ),
        );
        return;
    }

    //FIXME: what if we want to return them to normal global gravity?
    if !(*self_).NPC.is_null() {
        (*(*self_).NPC).aiFlags |= NPCAI_CUSTOM_GRAVITY;
    }
    (*(*self_).client).ps.gravity = float_data as c_int;
}

/// `static void Q3_SetWait(int entID, float float_data)` (g_ICARUScb.c:3102) — sets `ent->wait`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetWait(entID: c_int, float_data: f32) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetWait: invalid entID {entID}\n"));
        return;
    }

    (*self_).wait = float_data;
}

/// `static void Q3_SetScale(int entID, float float_data)` (g_ICARUScb.c:3364) — sets the
/// model-scale on a client or entity. A negative `float_data` is stored raw (sentinel);
/// otherwise it is the percentage-int `float_data*100`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetScale(entID: c_int, float_data: f32) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetScale: invalid entID {entID}\n"));
        return;
    }

    if !(*self_).client.is_null() {
        if float_data < 0.0 {
            (*(*self_).client).ps.iModelScale = float_data as c_int;
        } else {
            (*(*self_).client).ps.iModelScale = (float_data * 100.0) as c_int;
        }
    } else if float_data < 0.0 {
        (*self_).s.iModelScale = float_data as c_int;
    } else {
        (*self_).s.iModelScale = (float_data * 100.0) as c_int;
    }
}

/// `static float Q3_GameSideCheckStringCounterIncrement( const char *string )`
/// (g_ICARUScb.c:3175) — parses a leading `+N`/`-N` into a (signed) float increment, else 0.
///
/// # Safety
/// `string` must be a valid C string.
pub unsafe fn Q3_GameSideCheckStringCounterIncrement(string: *const c_char) -> f32 {
    let mut val: f32 = 0.0;

    if *string == b'+' as c_char {
        //We want to increment whatever the value is by whatever follows the +
        if *string.add(1) != 0 {
            let num_string = string.add(1);
            val = atof(num_string) as f32;
        }
    } else if *string == b'-' as c_char {
        //we want to decrement
        if *string.add(1) != 0 {
            let num_string = string.add(1);
            val = atof(num_string) as f32 * -1.0;
        }
    }

    val
}

/// `static void Q3_SetCount(int entID, const char *data)` (g_ICARUScb.c:3209) — sets/increments
/// `ent->count`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetCount(entID: c_int, data: *const c_char) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let val: f32;

    //FIXME: use FOFS() stuff here to make a generic entity field setting?
    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetCount: invalid entID {entID}\n"));
        return;
    }

    val = Q3_GameSideCheckStringCounterIncrement(data);
    if val != 0.0 {
        (*self_).count += val as c_int;
    } else {
        (*self_).count = atoi(data);
    }
}

/// `static void Q3_SetTargetName (int entID, const char *targetname)` (g_ICARUScb.c:3241) —
/// sets/clears `ent->targetname` (interning via `G_NewString`).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetTargetName(entID: c_int, targetname: *const c_char) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetTargetName: invalid entID {entID}\n"));
        return;
    }

    if Q_stricmp(c"NULL".as_ptr(), targetname) == 0 {
        (*self_).targetname = null_mut();
    } else {
        (*self_).targetname = G_NewString(targetname);
    }
}

/// `static void Q3_SetTarget (int entID, const char *target)` (g_ICARUScb.c:3271) — sets/clears
/// `ent->target` (interning via `G_NewString`).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetTarget(entID: c_int, target: *const c_char) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetTarget: invalid entID {entID}\n"));
        return;
    }

    if Q_stricmp(c"NULL".as_ptr(), target) == 0 {
        (*self_).target = null_mut();
    } else {
        (*self_).target = G_NewString(target);
    }
}

/// `static void Q3_SetFullName (int entID, const char *fullName)` (g_ICARUScb.c:3379) —
/// sets/clears `ent->fullName` (interning via `G_NewString`).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetFullName(entID: c_int, fullName: *const c_char) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetFullName: invalid entID {entID}\n"));
        return;
    }

    if Q_stricmp(c"NULL".as_ptr(), fullName) == 0 {
        (*self_).fullName = null_mut();
    } else {
        (*self_).fullName = G_NewString(fullName);
    }
}

/// `void Q3_SetParm (int entID, int parmNum, const char *parmValue)` (g_ICARUScb.c:3420) —
/// sets/increments one of an entity's ICARUS parms (allocating the parms block if needed).
///
/// # Safety
/// `g_entities` must be initialised; allocates via `G_Alloc`.
pub unsafe fn Q3_SetParm(entID: c_int, parmNum: c_int, parmValue: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let val: f32;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetParm: invalid entID {entID}\n"));
        return;
    }

    if parmNum < 0 || parmNum >= MAX_PARMS as c_int {
        G_DebugPrint(WL_WARNING, &format!("SET_PARM: parmNum {parmNum} out of range!\n"));
        return;
    }

    if (*ent).parms.is_null() {
        (*ent).parms = G_Alloc(core::mem::size_of::<parms_t>() as c_int) as *mut parms_t;
        core::ptr::write_bytes((*ent).parms as *mut u8, 0, core::mem::size_of::<parms_t>());
    }

    val = Q3_GameSideCheckStringCounterIncrement(parmValue);
    if val != 0.0 {
        let cur = atof((*(*ent).parms).parm[parmNum as usize].as_ptr()) as f32 + val;
        // C: Com_sprintf( ent->parms->parm[parmNum], sizeof(ent->parms->parm), "%f", val );
        Com_sprintf(
            (*(*ent).parms).parm[parmNum as usize].as_mut_ptr(),
            core::mem::size_of_val(&(*(*ent).parms).parm) as c_int,
            format_args!("{cur:.6}"),
        );
    } else {
        //Just copy the string
        //copy only 16 characters
        let dst = &mut (*(*ent).parms).parm[parmNum as usize];
        let cap = dst.len();
        strncpy_field(dst.as_mut_ptr(), parmValue, cap);
        //set the last charcter to null in case we had to truncate their passed string
        if dst[cap - 1] != 0 {
            //Tried to set a string that is too long
            dst[cap - 1] = 0;
            G_DebugPrint(
                WL_WARNING,
                &format!(
                    "SET_PARM: parm{} string too long, truncated to '{}'!\n",
                    parmNum,
                    CStr::from_ptr(dst.as_ptr()).to_string_lossy()
                ),
            );
        }
    }
}

/// `static void Q3_SetNoTarget( int entID, qboolean data)` (g_ICARUScb.c:3536) — toggles
/// `FL_NOTARGET`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetNoTarget(entID: c_int, data: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetNoTarget: invalid entID {entID}\n"));
        return;
    }

    if data != QFALSE {
        (*ent).flags |= FL_NOTARGET;
    } else {
        (*ent).flags &= !FL_NOTARGET;
    }
}

/// `static void Q3_SetInactive(int entID, qboolean add)` (g_ICARUScb.c:3599) — toggles
/// `FL_INACTIVE`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetInactive(entID: c_int, add: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetInactive: invalid entID {entID}\n"));
        return;
    }

    if add != QFALSE {
        (*ent).flags |= FL_INACTIVE;
    } else {
        (*ent).flags &= !FL_INACTIVE;
    }
}

/// `static void Q3_SetFuncUsableVisible(int entID, qboolean visible )` (g_ICARUScb.c:3626) —
/// toggles a func_usable's draw/contents via `SVF_NOCLIENT` + `EF_NODRAW`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetFuncUsableVisible(entID: c_int, visible: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetFuncUsableVisible: invalid entID {entID}\n"));
        return;
    }

    // Yeah, I know that this doesn't even do half of what the func_usable use code does, but if I've got two things on top of each other...and only
    //	one is visible at a time....and neither can ever be used......and finally, the shader on it has the shader_anim stuff going on....It doesn't seem
    //	like I can easily use the other version without nasty side effects.
    if visible != QFALSE {
        (*ent).r.svFlags &= !SVF_NOCLIENT;
        (*ent).s.eFlags &= !EF_NODRAW;
    } else {
        (*ent).r.svFlags |= SVF_NOCLIENT;
        (*ent).s.eFlags |= EF_NODRAW;
    }
}

/// `static void Q3_SetInvisible( int entID, qboolean invisible )` (g_ICARUScb.c:2765) —
/// toggles `EF_NODRAW` (and clears contents when made invisible).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetInvisible(entID: c_int, invisible: qboolean) {
    let self_: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if self_.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetInvisible: invalid entID {entID}\n"));
        return;
    }

    if invisible != QFALSE {
        (*self_).s.eFlags |= EF_NODRAW;
        if !(*self_).client.is_null() {
            (*(*self_).client).ps.eFlags |= EF_NODRAW;
        }
        (*self_).r.contents = 0;
    } else {
        (*self_).s.eFlags &= !EF_NODRAW;
        if !(*self_).client.is_null() {
            (*(*self_).client).ps.eFlags &= !EF_NODRAW;
        }
    }
}

/// `static void Q3_SetForwardMove( int entID, int fmoveVal)` (g_ICARUScb.c:4082) — validates a
/// client then warns (the body is `#if`'d out in the C — unsupported in MP).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetForwardMove(entID: c_int, _fmoveVal: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetForwardMove: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetForwardMove: '{}' is not an NPC/player!\n",
                cstr_or_null((*ent).targetname)
            ),
        );
        return;
    }

    G_DebugPrint(WL_WARNING, "Q3_SetForwardMove: NOT SUPPORTED IN MP\n");
    //ent->client->forced_forwardmove = fmoveVal;
}

/// `static void Q3_SetRightMove( int entID, int rmoveVal)` (g_ICARUScb.c:4109) — validates a
/// client then warns (the body is `#if`'d out in the C — unsupported in MP).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetRightMove(entID: c_int, _rmoveVal: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetRightMove: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetRightMove: '{}' is not an NPC/player!\n",
                cstr_or_null((*ent).targetname)
            ),
        );
        return;
    }

    G_DebugPrint(WL_WARNING, "Q3_SetRightMove: NOT SUPPORTED IN MP\n");
    //ent->client->forced_rightmove = rmoveVal;
}

/// `static void Q3_SetLockAngle( int entID, const char *lockAngle)` (g_ICARUScb.c:4136) —
/// validates a client then warns (the body is `#if`'d out in the C — unsupported in MP).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetLockAngle(entID: c_int, _lockAngle: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetLockAngle: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetLockAngle: '{}' is not an NPC/player!\n",
                cstr_or_null((*ent).targetname)
            ),
        );
        return;
    }

    G_DebugPrint(
        WL_WARNING,
        "Q3_SetLockAngle is not currently available. Ask if you really need it.\n",
    );
}

/// `static void Q3_SetPlayerUsable( int entID, qboolean usable )` (g_ICARUScb.c:4462) — toggles
/// `SVF_PLAYER_USABLE`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetPlayerUsable(entID: c_int, usable: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetPlayerUsable: invalid entID {entID}\n"));
        return;
    }

    if usable != QFALSE {
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    } else {
        (*ent).r.svFlags &= !SVF_PLAYER_USABLE;
    }
}

/// `static void Q3_SetNoKnockback( int entID, qboolean noKnockback )` (g_ICARUScb.c:4628) —
/// toggles `FL_NO_KNOCKBACK`.
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_SetNoKnockback(entID: c_int, noKnockback: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetNoKnockback: invalid entID {entID}\n"));
        return;
    }

    if noKnockback != QFALSE {
        (*ent).flags |= FL_NO_KNOCKBACK;
    } else {
        (*ent).flags &= !FL_NO_KNOCKBACK;
    }
}

/// `static void Q3_SetTimeScale( int entID, const char *data )` (g_ICARUScb.c:2750) — sets the
/// `timescale` cvar.
///
/// # Safety
/// `data` must be a valid C string.
pub unsafe fn Q3_SetTimeScale(_entID: c_int, data: *const c_char) {
    trap::Cvar_Set("timescale", &CStr::from_ptr(data).to_string_lossy());
}

/// `void anglerCallback( gentity_t *ent )` (g_ICARUScb.c:568) — utility think-fn that
/// completes a `TID_ANGLE_FACE` task: snaps `currentAngles` to the trajectory's end, clears all
/// angular movement to `TR_STATIONARY`, stops thinking (if still pointed at itself) and relinks.
/// Assigned to `ent->think`, so `extern "C"`.
///
/// # Safety
/// `ent` must be a valid entity pointer; `level` must be initialised.
pub unsafe extern "C" fn anglerCallback(ent: *mut gentity_t) {
    //Complete the task
    trap::ICARUS_TaskIDComplete(ent, TID_ANGLE_FACE);

    //Set the currentAngles, clear all movement
    VectorMA(
        &(*ent).s.apos.trBase,
        (*ent).s.apos.trDuration as f32 * 0.001,
        &(*ent).s.apos.trDelta,
        &mut (*ent).r.currentAngles,
    );
    VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
    VectorClear(&mut (*ent).s.apos.trDelta);
    (*ent).s.apos.trDuration = 1;
    (*ent).s.apos.trType = TR_STATIONARY;
    (*ent).s.apos.trTime = (*addr_of!(g_level)).time;

    //Stop thinking
    (*ent).reached = None;
    if let Some(think) = (*ent).think {
        if core::ptr::fn_addr_eq(
            think,
            anglerCallback as unsafe extern "C" fn(*mut gentity_t),
        ) {
            (*ent).think = None;
        }
    }

    //link
    trap::LinkEntity(ent);
}

/// `void moverCallback( gentity_t *ent )` (g_ICARUScb.c:602) — utility reached-fn that completes
/// a `TID_MOVE_NAV` task: stops the looping sound, plays the end sound, matches the mover team
/// to its destination pos, and clears the `Blocked_Mover` callback. Assigned to `ent->reached`,
/// so `extern "C"`.
///
/// # Safety
/// `ent` must be a valid entity pointer; `level` must be initialised.
pub unsafe extern "C" fn moverCallback(ent: *mut gentity_t) {
    //complete the task
    trap::ICARUS_TaskIDComplete(ent, TID_MOVE_NAV);

    // play sound
    (*ent).s.loopSound = 0; //stop looping sound
    (*ent).s.loopIsSoundset = QFALSE;
    G_PlayDoorSound(ent, BMS_END); //play end sound

    if (*ent).moverState == MOVER_1TO2 {
        //reached open
        // reached pos2
        MatchTeam(ent, MOVER_POS2, (*addr_of!(g_level)).time);
        //SetMoverState( ent, MOVER_POS2, level.time );
    } else if (*ent).moverState == MOVER_2TO1 {
        //reached closed
        MatchTeam(ent, MOVER_POS1, (*addr_of!(g_level)).time);
        //SetMoverState( ent, MOVER_POS1, level.time );
    }

    if let Some(blocked) = (*ent).blocked {
        if core::ptr::fn_addr_eq(
            blocked,
            Blocked_Mover as unsafe extern "C" fn(*mut gentity_t, *mut gentity_t),
        ) {
            (*ent).blocked = None;
        }
    }

    //	if ( !Q_stricmp( "misc_model_breakable", ent->classname ) && ent->physicsBounce )
    //	{//a gravity-affected model
    //		misc_model_breakable_gravity_init( ent, qfalse );
    //	}
}

/// `void Blocked_Mover( gentity_t *ent, gentity_t *other )` (g_ICARUScb.c:634) — a mover's
/// blocked-callback: frees a non-client (or dead-client corpse) that blocks it, and crushes
/// whatever blocked it for `ent->damage`. Assigned to `ent->blocked`, so `extern "C"`.
///
/// # Safety
/// `ent`/`other` must be valid entity pointers.
pub unsafe extern "C" fn Blocked_Mover(ent: *mut gentity_t, other: *mut gentity_t) {
    // remove anything other than a client -- no longer the case

    // don't remove security keys or goodie keys
    if (*other).s.eType == ET_ITEM {
        // should we be doing anything special if a key blocks it... move it somehow..?
    }
    // if your not a client, or your a dead client remove yourself...
    else if (*other).s.number != 0
        && ((*other).client.is_null()
            || (!(*other).client.is_null()
                && (*other).health <= 0
                && (*other).r.contents == CONTENTS_CORPSE
                && (*other).message.is_null()))
    {
        //if ( !other->taskManager || !other->taskManager->IsRunning() )
        {
            // if an item or weapon can we do a little explosion..?
            G_FreeEntity(other);
            return;
        }
    }

    if (*ent).damage != 0 {
        G_Damage(
            other,
            ent,
            ent,
            null_mut(),
            null_mut(),
            (*ent).damage,
            0,
            MOD_CRUSH,
        );
    }
}

/// `void moveAndRotateCallback( gentity_t *ent )` (g_ICARUScb.c:666) — utility callback that
/// stops both turning ([`anglerCallback`]) and moving ([`moverCallback`]).
///
/// # Safety
/// `ent` must be a valid entity pointer; `level` must be initialised.
pub unsafe extern "C" fn moveAndRotateCallback(ent: *mut gentity_t) {
    //stop turning
    anglerCallback(ent);
    //stop moving
    moverCallback(ent);
}

/// `void Q3_Lerp2Start( int entID, int taskID, float duration )` (g_ICARUScb.c:681) — lerps the
/// origin of an entity to its starting position (`MOVER_2TO1`), wiring the move/blocked
/// callbacks, registering a `TID_MOVE_NAV` task and playing the door sounds.
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn Q3_Lerp2Start(entID: c_int, taskID: c_int, duration: f32) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Lerp2Start: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_Lerp2Start: ent {entID} is NOT a mover!\n"));
        return;
    }

    if (*ent).s.eType != ET_MOVER {
        (*ent).s.eType = ET_MOVER;
    }

    //FIXME: set up correctly!!!
    (*ent).moverState = MOVER_2TO1;
    (*ent).s.eType = ET_MOVER;
    (*ent).reached = Some(moverCallback); //Callsback the the completion of the move
    if (*ent).damage != 0 {
        (*ent).blocked = Some(Blocked_Mover);
    }

    (*ent).s.pos.trDuration = (duration * 10.0) as c_int; //In seconds
    (*ent).s.pos.trTime = (*addr_of!(g_level)).time;

    trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
    // starting sound
    G_PlayDoorLoopSound(ent);
    G_PlayDoorSound(ent, BMS_START); //??

    trap::LinkEntity(ent);
}

/// `void Q3_Lerp2End( int entID, int taskID, float duration )` (g_ICARUScb.c:729) — lerps the
/// origin of an entity to its ending position (`MOVER_1TO2`), wiring the move/blocked callbacks,
/// registering a `TID_MOVE_NAV` task and playing the door sounds.
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn Q3_Lerp2End(entID: c_int, taskID: c_int, duration: f32) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Lerp2End: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_Lerp2End: ent {entID} is NOT a mover!\n"));
        return;
    }

    if (*ent).s.eType != ET_MOVER {
        (*ent).s.eType = ET_MOVER;
    }

    //FIXME: set up correctly!!!
    (*ent).moverState = MOVER_1TO2;
    (*ent).s.eType = ET_MOVER;
    (*ent).reached = Some(moverCallback); //Callsback the the completion of the move
    if (*ent).damage != 0 {
        (*ent).blocked = Some(Blocked_Mover);
    }

    (*ent).s.pos.trDuration = (duration * 10.0) as c_int; //In seconds
    (*ent).s.time = (*addr_of!(g_level)).time;

    trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
    // starting sound
    G_PlayDoorLoopSound(ent);
    G_PlayDoorSound(ent, BMS_START); //??

    trap::LinkEntity(ent);
}

/// `void Q3_Lerp2Pos( int taskID, int entID, vec3_t origin, vec3_t angles, float duration )`
/// (g_ICARUScb.c:780) — lerps both the origin and (optionally) the angles of a mover to the
/// destination values, choosing the proper mover direction and rotation trajectory.
///
/// # Safety
/// `g_entities`/`level` must be initialised; `angles` may be NULL (handled).
pub unsafe fn Q3_Lerp2Pos(
    taskID: c_int,
    entID: c_int,
    origin: &vec3_t,
    angles: *const vec3_t,
    mut duration: f32,
) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut ang: vec3_t = [0.0; 3];
    let moverState;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Lerp2Pos: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_Lerp2Pos: ent {entID} is NOT a mover!\n"));
        return;
    }

    if (*ent).s.eType != ET_MOVER {
        (*ent).s.eType = ET_MOVER;
    }

    //Don't allow a zero duration
    if duration == 0.0 {
        duration = 1.0;
    }

    //
    // Movement

    let cur_state = (*ent).moverState;

    if cur_state == MOVER_POS1 || cur_state == MOVER_2TO1 {
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).pos1);
        VectorCopy(origin, &mut (*ent).pos2);

        moverState = MOVER_1TO2;
    } else {
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).pos2);
        VectorCopy(origin, &mut (*ent).pos1);

        moverState = MOVER_2TO1;
    }

    InitMoverTrData(ent);

    (*ent).s.pos.trDuration = duration as c_int;

    // start it going
    MatchTeam(ent, moverState, (*addr_of!(g_level)).time);
    //SetMoverState( ent, moverState, level.time );

    //Only do the angles if specified
    if !angles.is_null() {
        //
        // Rotation

        for i in 0..3 {
            ang[i] = AngleDelta((*angles)[i], (*ent).r.currentAngles[i]);
            (*ent).s.apos.trDelta[i] = ang[i] / (duration * 0.001);
        }

        VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);

        if (*ent).alt_fire != QFALSE {
            (*ent).s.apos.trType = TR_LINEAR_STOP;
        } else {
            (*ent).s.apos.trType = TR_NONLINEAR_STOP;
        }
        (*ent).s.apos.trDuration = duration as c_int;

        (*ent).s.apos.trTime = (*addr_of!(g_level)).time;

        (*ent).reached = Some(moveAndRotateCallback);
        trap::ICARUS_TaskIDSet(ent, TID_ANGLE_FACE, taskID);
    } else {
        //Setup the last bits of information
        (*ent).reached = Some(moverCallback);
    }

    if (*ent).damage != 0 {
        (*ent).blocked = Some(Blocked_Mover);
    }

    trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
    // starting sound
    G_PlayDoorLoopSound(ent);
    G_PlayDoorSound(ent, BMS_START); //??

    trap::LinkEntity(ent);
}

/// `void Q3_Lerp2Angles( int taskID, int entID, vec3_t angles, float duration )`
/// (g_ICARUScb.c:891) — lerps a mover's angles to the destination value, thinking via
/// [`anglerCallback`] when the move completes.
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn Q3_Lerp2Angles(taskID: c_int, entID: c_int, angles: &vec3_t, duration: f32) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut ang: vec3_t = [0.0; 3];

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Lerp2Angles: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_Lerp2Angles: ent {entID} is NOT a mover!\n"));
        return;
    }

    //If we want an instant move, don't send 0...
    (*ent).s.apos.trDuration = if duration > 0.0 { duration as c_int } else { 1 };

    for i in 0..3 {
        ang[i] = AngleSubtract(angles[i], (*ent).r.currentAngles[i]);
        (*ent).s.apos.trDelta[i] = ang[i] / ((*ent).s.apos.trDuration as f32 * 0.001);
    }

    VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);

    if (*ent).alt_fire != QFALSE {
        (*ent).s.apos.trType = TR_LINEAR_STOP;
    } else {
        (*ent).s.apos.trType = TR_NONLINEAR_STOP;
    }

    (*ent).s.apos.trTime = (*addr_of!(g_level)).time;

    trap::ICARUS_TaskIDSet(ent, TID_ANGLE_FACE, taskID);

    //ent->e_ReachedFunc = reachedF_NULL;
    (*ent).think = Some(anglerCallback);
    (*ent).nextthink = (*addr_of!(g_level)).time + duration as c_int;

    trap::LinkEntity(ent);
}

/// `void Q3_Lerp2Origin( int taskID, int entID, vec3_t origin, float duration )`
/// (g_ICARUScb.c:2054) — lerps a mover's origin to the destination value (origin only).
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn Q3_Lerp2Origin(taskID: c_int, entID: c_int, origin: &vec3_t, duration: f32) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let moverState;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_Lerp2Origin: invalid entID {entID}\n"));
        return;
    }

    if !(*ent).client.is_null()
        || Q_stricmp((*ent).classname, c"target_scriptrunner".as_ptr()) == 0
    {
        G_DebugPrint(WL_ERROR, &format!("Q3_Lerp2Origin: ent {entID} is NOT a mover!\n"));
        return;
    }

    if (*ent).s.eType != ET_MOVER {
        (*ent).s.eType = ET_MOVER;
    }

    let cur_state = (*ent).moverState;

    if cur_state == MOVER_POS1 || cur_state == MOVER_2TO1 {
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).pos1);
        VectorCopy(origin, &mut (*ent).pos2);

        moverState = MOVER_1TO2;
    } else if cur_state == MOVER_POS2 || cur_state == MOVER_1TO2 {
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).pos2);
        VectorCopy(origin, &mut (*ent).pos1);

        moverState = MOVER_2TO1;
    } else {
        // C leaves `moverState` uninitialised in the (unreachable) fall-through; mirror by
        // retaining the entity's current state so MatchTeam below is still well-defined.
        moverState = cur_state;
    }

    InitMoverTrData(ent); //FIXME: This will probably break normal things that are being moved...

    (*ent).s.pos.trDuration = duration as c_int;

    // start it going
    MatchTeam(ent, moverState, (*addr_of!(g_level)).time);
    //SetMoverState( ent, moverState, level.time );

    (*ent).reached = Some(moverCallback);
    if (*ent).damage != 0 {
        (*ent).blocked = Some(Blocked_Mover);
    }
    if taskID != -1 {
        trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
    }
    // starting sound
    G_PlayDoorLoopSound(ent); //start looping sound
    G_PlayDoorSound(ent, BMS_START); //play start sound

    trap::LinkEntity(ent);
}

/// `void MoveOwner( gentity_t *self )` (g_ICARUScb.c:1868) — think-fn for the teleport helper
/// spawned by `Q3_SetTeleportDest`: frees itself next frame, but if the destination would still
/// telefrag the owner it re-thinks itself; otherwise it places the owner and completes the move
/// task. Assigned to `self->think`, so `extern "C"`.
///
/// # Safety
/// `g_entities`/`level` must be initialised; `self` must be a valid entity pointer.
pub unsafe extern "C" fn MoveOwner(self_: *mut gentity_t) {
    let owner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*self_).r.ownerNum as usize);

    (*self_).nextthink = (*addr_of!(g_level)).time + FRAMETIME;
    (*self_).think = Some(G_FreeEntity);

    if owner.is_null() || (*owner).inuse == QFALSE {
        return;
    }

    if SpotWouldTelefrag2(owner, &(*self_).r.currentOrigin) != QFALSE {
        (*self_).think = Some(MoveOwner);
    } else {
        G_SetOrigin(owner, &(*self_).r.currentOrigin);
        trap::ICARUS_TaskIDComplete(owner, TID_MOVE_NAV);
    }
}

/// `void SolidifyOwner( gentity_t *self )` (g_ICARUScb.c:3999) — think-fn that re-solidifies a
/// resized owner once the spot is clear: temporarily forces `CONTENTS_BODY` and, if that would
/// telefrag, restores the old contents and re-thinks; otherwise completes the resize task.
/// Assigned to `self->think`, so `extern "C"`.
///
/// # Safety
/// `g_entities`/`level` must be initialised; `self` must be a valid entity pointer.
pub unsafe extern "C" fn SolidifyOwner(self_: *mut gentity_t) {
    let oldContents: c_int;
    let owner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*self_).r.ownerNum as usize);

    (*self_).nextthink = (*addr_of!(g_level)).time + FRAMETIME;
    (*self_).think = Some(G_FreeEntity);

    if owner.is_null() || (*owner).inuse == QFALSE {
        return;
    }

    oldContents = (*owner).r.contents;
    (*owner).r.contents = CONTENTS_BODY;
    if SpotWouldTelefrag2(owner, &(*owner).r.currentOrigin) != QFALSE {
        (*owner).r.contents = oldContents;
        (*self_).think = Some(SolidifyOwner);
    } else {
        trap::ICARUS_TaskIDComplete(owner, TID_RESIZE);
    }
}

/// `static qboolean Q3_SetSolid( int entID, qboolean solid)` (g_ICARUScb.c:4033) —
/// solidifies/de-solidifies an entity. When solidifying, if the spot is occupied it spawns a
/// [`SolidifyOwner`] helper to keep retrying (and returns `qfalse`); otherwise it ORs
/// `CONTENTS_BODY` into the clipmask. When de-solidifying it drops contents to none (if also
/// `EF_NODRAW`) or to `CONTENTS_CORPSE`.
///
/// # Safety
/// `g_entities`/`level` must be initialised; may spawn an entity.
pub unsafe fn Q3_SetSolid(entID: c_int, solid: qboolean) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() || (*ent).inuse == QFALSE {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetSolid: invalid entID {entID}\n"));
        return QTRUE;
    }

    if solid != QFALSE {
        //FIXME: Presumption
        let oldContents: c_int = (*ent).r.contents;
        (*ent).r.contents = CONTENTS_BODY;
        if SpotWouldTelefrag2(ent, &(*ent).r.currentOrigin) != QFALSE {
            let solidifier: *mut gentity_t = G_Spawn();

            (*solidifier).r.ownerNum = (*ent).s.number;

            (*solidifier).think = Some(SolidifyOwner);
            (*solidifier).nextthink = (*addr_of!(g_level)).time + FRAMETIME;

            (*ent).r.contents = oldContents;
            return QFALSE;
        }
        (*ent).clipmask |= CONTENTS_BODY;
    } else {
        //FIXME: Presumption
        if (*ent).s.eFlags & EF_NODRAW != 0 {
            //We're invisible too, so set contents to none
            (*ent).r.contents = 0;
        } else {
            (*ent).r.contents = CONTENTS_CORPSE;
        }
    }
    QTRUE
}

/// `static void Q3_SetSaberActive( int entID, qboolean active )` (g_ICARUScb.c:4594) — toggles a
/// client's saber on/off by funnelling through `Cmd_ToggleSaber_f` when the requested state
/// differs from the current holster state.
///
/// # Safety
/// `g_entities` must be initialised; `entID` must index a valid slot.
pub unsafe fn Q3_SetSaberActive(entID: c_int, active: qboolean) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() || (*ent).inuse == QFALSE {
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetSaberActive: {entID} is not a client\n"));
    }

    //fixme: Take into account player being in state where saber won't toggle? For now we simply won't care.
    if (*(*ent).client).ps.saberHolstered == 0 && active != QFALSE {
        Cmd_ToggleSaber_f(ent);
    } else if BG_SabersOff(&mut (*(*ent).client).ps) != QFALSE && active == QFALSE {
        Cmd_ToggleSaber_f(ent);
    }
}

// Helper: `sscanf( data, "%f %f %f", &v[0], &v[1], &v[2] )`. Parses up to three
// whitespace-separated leading floats off the C string `data`, each via the same
// stop-at-first-non-float rule sscanf("%f") uses (see `parse_leading_f32`). Slots that
// have no parseable token are left untouched (the C leaves them as the caller's prior
// stack value; callers here read the vector immediately, so well-formed data fills all 3).
unsafe fn sscanf_vec3(data: *const c_char, v: &mut vec3_t) {
    if data.is_null() {
        return;
    }
    let s = CStr::from_ptr(data).to_string_lossy();
    for (i, tok) in s.split_ascii_whitespace().take(3).enumerate() {
        if let Ok(f) = parse_leading_f32(tok) {
            v[i] = f;
        }
    }
}

/// `static void Q3_SetEnemy( int entID, const char *name )` (g_ICARUScb.c:2149) — sets the
/// entity's enemy by targetname (clearing it on "NONE"/"NULL").
///
/// # Safety
/// `g_entities` must be initialised; `name` must be a valid C string.
pub unsafe fn Q3_SetEnemy(entID: c_int, name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetEnemy: invalid entID {entID}\n"));
        return;
    }

    if Q_stricmp(c"NONE".as_ptr(), name) == 0 || Q_stricmp(c"NULL".as_ptr(), name) == 0 {
        if !(*ent).NPC.is_null() {
            G_ClearEnemy(ent);
        } else {
            (*ent).enemy = null_mut();
        }
    } else {
        let enemy: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, targetname), name);

        if enemy.is_null() {
            G_DebugPrint(
                WL_ERROR,
                &format!("Q3_SetEnemy: no such enemy: '{}'\n", cstr_or_null(name)),
            );
            // return;
        }
        /*else if(enemy->health <= 0)
        {
            //G_DebugPrint( WL_ERROR, "Q3_SetEnemy: ERROR - desired enemy has health %d\n", enemy->health );
            return;
        }*/
        else if !(*ent).NPC.is_null() {
            G_SetEnemy(ent, enemy);
            (*ent).cantHitEnemyCounter = 0;
        } else {
            G_SetEnemy(ent, enemy);
        }
    }
}

/// `static void Q3_SetLeader( int entID, const char *name )` (g_ICARUScb.c:2207) — sets the
/// client's `leader` by targetname (clearing it on "NONE"/"NULL"; a dead leader is ignored).
///
/// # Safety
/// `g_entities` must be initialised; `name` must be a valid C string.
pub unsafe fn Q3_SetLeader(entID: c_int, name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetLeader: invalid entID {entID}\n"));
        return;
    }

    if (*ent).client.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!("Q3_SetLeader: ent {entID} is NOT a player or NPC!\n"),
        );
        return;
    }

    if Q_stricmp(c"NONE".as_ptr(), name) == 0 || Q_stricmp(c"NULL".as_ptr(), name) == 0 {
        (*(*ent).client).leader = null_mut();
    } else {
        let leader: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, targetname), name);

        if leader.is_null() {
            //G_DebugPrint( WL_ERROR,"Q3_SetEnemy: unable to locate enemy: '%s'\n", name );
            return;
        } else if (*leader).health <= 0 {
            //G_DebugPrint( WL_ERROR,"Q3_SetEnemy: ERROR - desired enemy has health %d\n", enemy->health );
            return;
        } else {
            (*(*ent).client).leader = leader;
        }
    }
}

/// `static qboolean Q3_SetNavGoal( int entID, const char *name )` (g_ICARUScb.c:2255) — sets the
/// NPC's navigational goal: clears it on "null"/"NULL", else resolves `name` first as a tag
/// (`TAG_GetOrigin2`→`NPC_SetMoveGoal`) and otherwise as an entity targetname. Returns `qtrue`
/// only on the successful tag-origin path (the caller then stashes a wait task).
///
/// # Safety
/// `g_entities` must be initialised; `name` must be a valid C string.
pub unsafe fn Q3_SetNavGoal(entID: c_int, name: *const c_char) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut goalPos: vec3_t = [0.0; 3];

    if (*ent).health == 0 {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a corpse! \"{}\"\n",
                cstr_or_null(name),
                cstr_or_null((*ent).script_targetname)
            ),
        );
        return QFALSE;
    }
    if (*ent).NPC.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a non-NPC: \"{}\"\n",
                cstr_or_null(name),
                cstr_or_null((*ent).script_targetname)
            ),
        );
        return QFALSE;
    }
    if (*(*ent).NPC).tempGoal.is_null() {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a dead NPC: \"{}\"\n",
                cstr_or_null(name),
                cstr_or_null((*ent).script_targetname)
            ),
        );
        return QFALSE;
    }
    if (*(*(*ent).NPC).tempGoal).inuse == QFALSE {
        G_DebugPrint(
            WL_ERROR,
            &format!(
                "Q3_SetNavGoal: NPC's (\"{}\") navgoal is freed: \"{}\"\n",
                cstr_or_null(name),
                cstr_or_null((*ent).script_targetname)
            ),
        );
        return QFALSE;
    }
    if Q_stricmp(c"null".as_ptr(), name) == 0 || Q_stricmp(c"NULL".as_ptr(), name) == 0 {
        (*(*ent).NPC).goalEntity = null_mut();
        trap::ICARUS_TaskIDComplete(ent, TID_MOVE_NAV);
        return QFALSE;
    } else {
        //Get the position of the goal
        if TAG_GetOrigin2(null_mut(), name, &mut goalPos) == QFALSE {
            let targ: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, targetname), name);
            if targ.is_null() {
                G_DebugPrint(
                    WL_ERROR,
                    &format!("Q3_SetNavGoal: can't find NAVGOAL \"{}\"\n", cstr_or_null(name)),
                );
                return QFALSE;
            } else {
                (*(*ent).NPC).goalEntity = targ;
                (*(*ent).NPC).goalRadius = ((((*ent).r.maxs[0] + (*ent).r.maxs[0]) as f64).sqrt()
                    + (((*targ).r.maxs[0] + (*targ).r.maxs[0]) as f64).sqrt())
                    as c_int;
                (*(*ent).NPC).aiFlags &= !NPCAI_TOUCHED_GOAL;
            }
        } else {
            let goalRadius: c_int = TAG_GetRadius(null_mut(), name);
            NPC_SetMoveGoal(ent, &goalPos, goalRadius, QTRUE, -1, null_mut());
            //We know we want to clear the lastWaypoint here
            (*(*(*ent).NPC).goalEntity).lastWaypoint = WAYPOINT_NONE;
            (*(*ent).NPC).aiFlags &= !NPCAI_TOUCHED_GOAL;
            // #ifdef _DEBUG: ent->NPC->tempGoal->target = G_NewString( name ); — debug-only, excluded.
            return QTRUE;
        }
    }
    QFALSE
}

/// `qboolean Q3_Set( int taskID, int entID, const char *type_name, const char *data )`
/// (g_ICARUScb.c:4746) — the giant ICARUS `set` dispatcher. Resolves `type_name` to a `SET_*`
/// id via `GetIDForString(setTable,…)`, parses `data` to the field's type, and applies it to
/// `g_entities[entID]`. Returns `qfalse` when the set starts an asynchronous task the script
/// must wait on (and stashes the taskID via `trap_ICARUS_TaskIDSet`); `qtrue` otherwise. The
/// `default` arm forwards to the declared ICARUS variable layer (`trap_ICARUS_SetVar`).
/// Many SET_* fields are explicitly NOT SUPPORTED IN MP and merely warn.
///
/// # Safety
/// `g_entities` must be initialised; `type_name`/`data` must be valid C strings.
pub unsafe fn Q3_Set(taskID: c_int, entID: c_int, type_name: *const c_char, data: *const c_char) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let float_data: f32;
    let int_data: c_int;
    let toSet: c_int;
    let mut vector_data: vec3_t = [0.0; 3];

    //Set this for callbacks
    toSet = GetIDForString(addr_of!(setTable) as *const stringID_table_t, type_name);

    //TODO: Throw in a showscript command that will list each command and what they're doing...
    //		maybe as simple as printing that line of the script to the console preceeded by the person's name?
    //		showscript can take any number of targetnames or "all"?  Groupname?
    match toSet {
        SET_ORIGIN => {
            sscanf_vec3(data, &mut vector_data);
            G_SetOrigin(ent, &vector_data);
            if Q_strncmp(c"NPC_".as_ptr(), (*ent).classname, 4) == 0 {
                //hack for moving spawners
                VectorCopy(&vector_data, &mut (*ent).s.origin);
            }
        }

        SET_TELEPORT_DEST => {
            sscanf_vec3(data, &mut vector_data);
            if Q3_SetTeleportDest(entID, &vector_data) == QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
                return QFALSE;
            }
        }

        SET_COPY_ORIGIN => {
            Q3_SetCopyOrigin(entID, data);
        }

        SET_ANGLES => {
            //Q3_SetAngles( entID, *(vec3_t *) data);
            sscanf_vec3(data, &mut vector_data);
            Q3_SetAngles(entID, &vector_data);
        }

        SET_XVELOCITY => {
            float_data = atof(data) as f32;
            Q3_SetVelocity(entID, 0, float_data);
        }

        SET_YVELOCITY => {
            float_data = atof(data) as f32;
            Q3_SetVelocity(entID, 1, float_data);
        }

        SET_ZVELOCITY => {
            float_data = atof(data) as f32;
            Q3_SetVelocity(entID, 2, float_data);
        }

        SET_Z_OFFSET => {
            float_data = atof(data) as f32;
            Q3_SetOriginOffset(entID, 2, float_data);
        }

        SET_ENEMY => {
            Q3_SetEnemy(entID, data);
        }

        SET_LEADER => {
            Q3_SetLeader(entID, data);
        }

        SET_NAVGOAL => {
            if Q3_SetNavGoal(entID, data) != QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_MOVE_NAV, taskID);
                return QFALSE; //Don't call it back
            }
        }

        SET_ANIM_UPPER => {
            if Q3_SetAnimUpper(entID, data) != QFALSE {
                Q3_TaskIDClear(addr_of_mut!((*ent).taskID[TID_ANIM_BOTH as usize])); //We only want to wait for the top
                trap::ICARUS_TaskIDSet(ent, TID_ANIM_UPPER, taskID);
                return QFALSE; //Don't call it back
            }
        }

        SET_ANIM_LOWER => {
            if Q3_SetAnimLower(entID, data) != QFALSE {
                Q3_TaskIDClear(addr_of_mut!((*ent).taskID[TID_ANIM_BOTH as usize])); //We only want to wait for the bottom
                trap::ICARUS_TaskIDSet(ent, TID_ANIM_LOWER, taskID);
                return QFALSE; //Don't call it back
            }
        }

        SET_ANIM_BOTH => {
            let mut both = 0;
            if Q3_SetAnimUpper(entID, data) != QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_ANIM_UPPER, taskID);
                both += 1;
            } else {
                G_DebugPrint(
                    WL_ERROR,
                    &format!(
                        "Q3_SetAnimUpper: {} does not have anim {}!\n",
                        cstr_or_null((*ent).targetname),
                        cstr_or_null(data)
                    ),
                );
            }
            if Q3_SetAnimLower(entID, data) != QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_ANIM_LOWER, taskID);
                both += 1;
            } else {
                G_DebugPrint(
                    WL_ERROR,
                    &format!(
                        "Q3_SetAnimLower: {} does not have anim {}!\n",
                        cstr_or_null((*ent).targetname),
                        cstr_or_null(data)
                    ),
                );
            }
            if both >= 2 {
                trap::ICARUS_TaskIDSet(ent, TID_ANIM_BOTH, taskID);
            }
            if both != 0 {
                return QFALSE; //Don't call it back
            }
        }

        SET_ANIM_HOLDTIME_LOWER => {
            int_data = atoi(data);
            Q3_SetAnimHoldTime(entID, int_data, QTRUE);
            Q3_TaskIDClear(addr_of_mut!((*ent).taskID[TID_ANIM_BOTH as usize])); //We only want to wait for the bottom
            trap::ICARUS_TaskIDSet(ent, TID_ANIM_LOWER, taskID);
            return QFALSE; //Don't call it back
        }

        SET_ANIM_HOLDTIME_UPPER => {
            int_data = atoi(data);
            Q3_SetAnimHoldTime(entID, int_data, QFALSE);
            Q3_TaskIDClear(addr_of_mut!((*ent).taskID[TID_ANIM_BOTH as usize])); //We only want to wait for the top
            trap::ICARUS_TaskIDSet(ent, TID_ANIM_UPPER, taskID);
            return QFALSE; //Don't call it back
        }

        SET_ANIM_HOLDTIME_BOTH => {
            int_data = atoi(data);
            Q3_SetAnimHoldTime(entID, int_data, QFALSE);
            Q3_SetAnimHoldTime(entID, int_data, QTRUE);
            trap::ICARUS_TaskIDSet(ent, TID_ANIM_BOTH, taskID);
            trap::ICARUS_TaskIDSet(ent, TID_ANIM_UPPER, taskID);
            trap::ICARUS_TaskIDSet(ent, TID_ANIM_LOWER, taskID);
            return QFALSE; //Don't call it back
        }

        SET_PLAYER_TEAM => {
            G_DebugPrint(WL_WARNING, "Q3_SetPlayerTeam: Not in MP ATM, let a programmer (ideally Rich) know if you need it\n");
        }

        SET_ENEMY_TEAM => {
            G_DebugPrint(WL_WARNING, "Q3_SetEnemyTeam: NOT SUPPORTED IN MP\n");
        }

        SET_HEALTH => {
            int_data = atoi(data);
            Q3_SetHealth(entID, int_data);
        }

        SET_ARMOR => {
            int_data = atoi(data);
            Q3_SetArmor(entID, int_data);
        }

        SET_BEHAVIOR_STATE => {
            if Q3_SetBState(entID, data) == QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_BSTATE, taskID);
                return QFALSE; //don't complete
            }
        }

        SET_DEFAULT_BSTATE => {
            Q3_SetDefaultBState(entID, data);
        }

        SET_TEMP_BSTATE => {
            if Q3_SetTempBState(entID, data) == QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_BSTATE, taskID);
                return QFALSE; //don't complete
            }
        }

        SET_CAPTURE => {
            Q3_SetCaptureGoal(entID, data);
        }

        SET_DPITCH => {
            //FIXME: make these set tempBehavior to BS_FACE and await completion?  Or set lockedDesiredPitch/Yaw and aimTime?
            float_data = atof(data) as f32;
            Q3_SetDPitch(entID, float_data);
            trap::ICARUS_TaskIDSet(ent, TID_ANGLE_FACE, taskID);
            return QFALSE;
        }

        SET_DYAW => {
            float_data = atof(data) as f32;
            Q3_SetDYaw(entID, float_data);
            trap::ICARUS_TaskIDSet(ent, TID_ANGLE_FACE, taskID);
            return QFALSE;
        }

        SET_EVENT => {
            Q3_SetEvent(entID, data);
        }

        SET_VIEWTARGET => {
            Q3_SetViewTarget(entID, data);
            trap::ICARUS_TaskIDSet(ent, TID_ANGLE_FACE, taskID);
            return QFALSE;
        }

        SET_WATCHTARGET => {
            Q3_SetWatchTarget(entID, data);
        }

        SET_VIEWENTITY => {
            Q3_SetViewEntity(entID, data);
        }

        SET_LOOPSOUND => {
            Q3_SetLoopSound(entID, data);
        }

        SET_ICARUS_FREEZE | SET_ICARUS_UNFREEZE => {
            Q3_SetICARUSFreeze(entID, data, (toSet == SET_ICARUS_FREEZE) as qboolean);
        }

        SET_WEAPON => {
            Q3_SetWeapon(entID, data);
        }

        SET_ITEM => {
            Q3_SetItem(entID, data);
        }

        SET_WALKSPEED => {
            int_data = atoi(data);
            Q3_SetWalkSpeed(entID, int_data);
        }

        SET_RUNSPEED => {
            int_data = atoi(data);
            Q3_SetRunSpeed(entID, int_data);
        }

        SET_WIDTH => {
            int_data = atoi(data);
            Q3_SetWidth(entID, int_data);
            return QFALSE;
        }

        SET_YAWSPEED => {
            float_data = atof(data) as f32;
            Q3_SetYawSpeed(entID, float_data);
        }

        SET_AGGRESSION => {
            int_data = atoi(data);
            Q3_SetAggression(entID, int_data);
        }

        SET_AIM => {
            int_data = atoi(data);
            Q3_SetAim(entID, int_data);
        }

        SET_FRICTION => {
            int_data = atoi(data);
            Q3_SetFriction(entID, int_data);
        }

        SET_GRAVITY => {
            float_data = atof(data) as f32;
            Q3_SetGravity(entID, float_data);
        }

        SET_WAIT => {
            float_data = atof(data) as f32;
            Q3_SetWait(entID, float_data);
        }

        SET_FOLLOWDIST => {
            float_data = atof(data) as f32;
            Q3_SetFollowDist(entID, float_data);
        }

        SET_SCALE => {
            float_data = atof(data) as f32;
            Q3_SetScale(entID, float_data);
        }

        SET_COUNT => {
            Q3_SetCount(entID, data);
        }

        SET_SHOT_SPACING => {
            int_data = atoi(data);
            Q3_SetShotSpacing(entID, int_data);
        }

        SET_IGNOREPAIN => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetIgnorePain(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetIgnorePain(entID, QFALSE);
            }
        }

        SET_IGNOREENEMIES => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetIgnoreEnemies(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetIgnoreEnemies(entID, QFALSE);
            }
        }

        SET_IGNOREALERTS => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetIgnoreAlerts(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetIgnoreAlerts(entID, QFALSE);
            }
        }

        SET_DONTSHOOT => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetDontShoot(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetDontShoot(entID, QFALSE);
            }
        }

        SET_DONTFIRE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetDontFire(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetDontFire(entID, QFALSE);
            }
        }

        SET_LOCKED_ENEMY => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetLockedEnemy(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetLockedEnemy(entID, QFALSE);
            }
        }

        SET_NOTARGET => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoTarget(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetNoTarget(entID, QFALSE);
            }
        }

        SET_LEAN => {
            G_DebugPrint(WL_WARNING, "SET_LEAN NOT SUPPORTED IN MP\n");
        }

        SET_SHOOTDIST => {
            float_data = atof(data) as f32;
            Q3_SetShootDist(entID, float_data);
        }

        SET_TIMESCALE => {
            Q3_SetTimeScale(entID, data);
        }

        SET_VISRANGE => {
            float_data = atof(data) as f32;
            Q3_SetVisrange(entID, float_data);
        }

        SET_EARSHOT => {
            float_data = atof(data) as f32;
            Q3_SetEarshot(entID, float_data);
        }

        SET_VIGILANCE => {
            float_data = atof(data) as f32;
            Q3_SetVigilance(entID, float_data);
        }

        SET_VFOV => {
            int_data = atoi(data);
            Q3_SetVFOV(entID, int_data);
        }

        SET_HFOV => {
            int_data = atoi(data);
            Q3_SetHFOV(entID, int_data);
        }

        SET_TARGETNAME => {
            Q3_SetTargetName(entID, data);
        }

        SET_TARGET => {
            Q3_SetTarget(entID, data);
        }

        SET_TARGET2 => {
            Q3_SetTarget2(entID, data);
        }

        SET_LOCATION => {
            if Q3_SetLocation(entID, data) == QFALSE {
                trap::ICARUS_TaskIDSet(ent, TID_LOCATION, taskID);
                return QFALSE;
            }
        }

        SET_PAINTARGET => {
            Q3_SetPainTarget(entID, data);
        }

        SET_DEFEND_TARGET => {
            G_DebugPrint(WL_WARNING, &format!("Q3_SetDefendTarget unimplemented\n"));
            //Q3_SetEnemy( entID, (char *) data);
        }

        SET_PARM1 | SET_PARM2 | SET_PARM3 | SET_PARM4 | SET_PARM5 | SET_PARM6 | SET_PARM7
        | SET_PARM8 | SET_PARM9 | SET_PARM10 | SET_PARM11 | SET_PARM12 | SET_PARM13
        | SET_PARM14 | SET_PARM15 | SET_PARM16 => {
            Q3_SetParm(entID, toSet - SET_PARM1, data);
        }

        SET_SPAWNSCRIPT | SET_USESCRIPT | SET_AWAKESCRIPT | SET_ANGERSCRIPT | SET_ATTACKSCRIPT
        | SET_VICTORYSCRIPT | SET_PAINSCRIPT | SET_FLEESCRIPT | SET_DEATHSCRIPT
        | SET_DELAYEDSCRIPT | SET_BLOCKEDSCRIPT | SET_FFIRESCRIPT | SET_FFDEATHSCRIPT
        | SET_MINDTRICKSCRIPT => {
            if Q3_SetBehaviorSet(entID, toSet, data) == QFALSE {
                G_DebugPrint(
                    WL_ERROR,
                    &format!("Q3_SetBehaviorSet: Invalid bSet {}\n", cstr_or_null(type_name)),
                );
            }
        }

        SET_NO_MINDTRICK => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoMindTrick(entID, QTRUE);
            } else {
                Q3_SetNoMindTrick(entID, QFALSE);
            }
        }

        SET_CINEMATIC_SKIPSCRIPT => {
            Q3_SetCinematicSkipScript(data as *mut c_char);
        }

        SET_DELAYSCRIPTTIME => {
            int_data = atoi(data);
            Q3_SetDelayScriptTime(entID, int_data);
        }

        SET_CROUCHED => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetCrouched(entID, QTRUE);
            } else {
                Q3_SetCrouched(entID, QFALSE);
            }
        }

        SET_WALKING => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetWalking(entID, QTRUE);
            } else {
                Q3_SetWalking(entID, QFALSE);
            }
        }

        SET_RUNNING => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetRunning(entID, QTRUE);
            } else {
                Q3_SetRunning(entID, QFALSE);
            }
        }

        SET_CHASE_ENEMIES => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetChaseEnemies(entID, QTRUE);
            } else {
                Q3_SetChaseEnemies(entID, QFALSE);
            }
        }

        SET_LOOK_FOR_ENEMIES => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetLookForEnemies(entID, QTRUE);
            } else {
                Q3_SetLookForEnemies(entID, QFALSE);
            }
        }

        SET_FACE_MOVE_DIR => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetFaceMoveDir(entID, QTRUE);
            } else {
                Q3_SetFaceMoveDir(entID, QFALSE);
            }
        }

        SET_ALT_FIRE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetAltFire(entID, QTRUE);
            } else {
                Q3_SetAltFire(entID, QFALSE);
            }
        }

        SET_DONT_FLEE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetDontFlee(entID, QTRUE);
            } else {
                Q3_SetDontFlee(entID, QFALSE);
            }
        }

        SET_FORCED_MARCH => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetForcedMarch(entID, QTRUE);
            } else {
                Q3_SetForcedMarch(entID, QFALSE);
            }
        }

        SET_NO_RESPONSE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoResponse(entID, QTRUE);
            } else {
                Q3_SetNoResponse(entID, QFALSE);
            }
        }

        SET_NO_COMBAT_TALK => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetCombatTalk(entID, QTRUE);
            } else {
                Q3_SetCombatTalk(entID, QFALSE);
            }
        }

        SET_NO_ALERT_TALK => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetAlertTalk(entID, QTRUE);
            } else {
                Q3_SetAlertTalk(entID, QFALSE);
            }
        }

        SET_USE_CP_NEAREST => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetUseCpNearest(entID, QTRUE);
            } else {
                Q3_SetUseCpNearest(entID, QFALSE);
            }
        }

        SET_NO_FORCE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoForce(entID, QTRUE);
            } else {
                Q3_SetNoForce(entID, QFALSE);
            }
        }

        SET_NO_ACROBATICS => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoAcrobatics(entID, QTRUE);
            } else {
                Q3_SetNoAcrobatics(entID, QFALSE);
            }
        }

        SET_USE_SUBTITLES => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetUseSubtitles(entID, QTRUE);
            } else {
                Q3_SetUseSubtitles(entID, QFALSE);
            }
        }

        SET_NO_FALLTODEATH => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoFallToDeath(entID, QTRUE);
            } else {
                Q3_SetNoFallToDeath(entID, QFALSE);
            }
        }

        SET_DISMEMBERABLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetDismemberable(entID, QTRUE);
            } else {
                Q3_SetDismemberable(entID, QFALSE);
            }
        }

        SET_MORELIGHT => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetMoreLight(entID, QTRUE);
            } else {
                Q3_SetMoreLight(entID, QFALSE);
            }
        }

        SET_TREASONED => {
            G_DebugPrint(WL_VERBOSE, "SET_TREASONED is disabled, do not use\n");
            /*
            G_TeamRetaliation( NULL, SV_GentityNum(0), qfalse );
            ffireLevel = FFIRE_LEVEL_RETALIATION;
            */
        }

        SET_UNDYING => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetUndying(entID, QTRUE);
            } else {
                Q3_SetUndying(entID, QFALSE);
            }
        }

        SET_INVINCIBLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetInvincible(entID, QTRUE);
            } else {
                Q3_SetInvincible(entID, QFALSE);
            }
        }

        SET_NOAVOID => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoAvoid(entID, QTRUE);
            } else {
                Q3_SetNoAvoid(entID, QFALSE);
            }
        }

        SET_SOLID => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                if Q3_SetSolid(entID, QTRUE) == QFALSE {
                    trap::ICARUS_TaskIDSet(ent, TID_RESIZE, taskID);
                    return QFALSE;
                }
            } else {
                Q3_SetSolid(entID, QFALSE);
            }
        }

        SET_INVISIBLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetInvisible(entID, QTRUE);
            } else {
                Q3_SetInvisible(entID, QFALSE);
            }
        }

        SET_VAMPIRE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetVampire(entID, QTRUE);
            } else {
                Q3_SetVampire(entID, QFALSE);
            }
        }

        SET_FORCE_INVINCIBLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetForceInvincible(entID, QTRUE);
            } else {
                Q3_SetForceInvincible(entID, QFALSE);
            }
        }

        SET_GREET_ALLIES => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetGreetAllies(entID, QTRUE);
            } else {
                Q3_SetGreetAllies(entID, QFALSE);
            }
        }

        SET_PLAYER_LOCKED => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetPlayerLocked(entID, QTRUE);
            } else {
                Q3_SetPlayerLocked(entID, QFALSE);
            }
        }

        SET_LOCK_PLAYER_WEAPONS => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetLockPlayerWeapons(entID, QTRUE);
            } else {
                Q3_SetLockPlayerWeapons(entID, QFALSE);
            }
        }

        SET_NO_IMPACT_DAMAGE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoImpactDamage(entID, QTRUE);
            } else {
                Q3_SetNoImpactDamage(entID, QFALSE);
            }
        }

        SET_FORWARDMOVE => {
            int_data = atoi(data);
            Q3_SetForwardMove(entID, int_data);
        }

        SET_RIGHTMOVE => {
            int_data = atoi(data);
            Q3_SetRightMove(entID, int_data);
        }

        SET_LOCKYAW => {
            Q3_SetLockAngle(entID, data);
        }

        SET_CAMERA_GROUP => {
            Q3_CameraGroup(entID, data as *mut c_char);
        }
        SET_CAMERA_GROUP_Z_OFS => {
            float_data = atof(data) as f32;
            Q3_CameraGroupZOfs(float_data);
        }
        SET_CAMERA_GROUP_TAG => {
            Q3_CameraGroupTag(data as *mut c_char);
        }

        //FIXME: put these into camera commands
        SET_LOOK_TARGET => {
            Q3_LookTarget(entID, data as *mut c_char);
        }

        SET_ADDRHANDBOLT_MODEL => {
            Q3_AddRHandModel(entID, data as *mut c_char);
        }

        SET_REMOVERHANDBOLT_MODEL => {
            Q3_RemoveRHandModel(entID, data as *mut c_char);
        }

        SET_ADDLHANDBOLT_MODEL => {
            Q3_AddLHandModel(entID, data as *mut c_char);
        }

        SET_REMOVELHANDBOLT_MODEL => {
            Q3_RemoveLHandModel(entID, data as *mut c_char);
        }

        SET_FACEEYESCLOSED | SET_FACEEYESOPENED | SET_FACEAUX | SET_FACEBLINK
        | SET_FACEBLINKFROWN | SET_FACEFROWN | SET_FACENORMAL => {
            float_data = atof(data) as f32;
            Q3_Face(entID, toSet, float_data);
        }

        SET_SCROLLTEXT => {
            Q3_ScrollText(data);
        }

        SET_LCARSTEXT => {
            Q3_LCARSText(data);
        }

        SET_CAPTIONTEXTCOLOR => {
            Q3_SetCaptionTextColor(data);
        }
        SET_CENTERTEXTCOLOR => {
            Q3_SetCenterTextColor(data);
        }
        SET_SCROLLTEXTCOLOR => {
            Q3_SetScrollTextColor(data);
        }

        SET_PLAYER_USABLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetPlayerUsable(entID, QTRUE);
            } else {
                Q3_SetPlayerUsable(entID, QFALSE);
            }
        }

        SET_STARTFRAME => {
            int_data = atoi(data);
            Q3_SetStartFrame(entID, int_data);
        }

        SET_ENDFRAME => {
            int_data = atoi(data);
            Q3_SetEndFrame(entID, int_data);

            trap::ICARUS_TaskIDSet(ent, TID_ANIM_BOTH, taskID);
            return QFALSE;
        }

        SET_ANIMFRAME => {
            int_data = atoi(data);
            Q3_SetAnimFrame(entID, int_data);
            return QFALSE;
        }

        SET_LOOP_ANIM => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetLoopAnim(entID, QTRUE);
            } else {
                Q3_SetLoopAnim(entID, QFALSE);
            }
        }

        SET_INTERFACE => {
            G_DebugPrint(WL_WARNING, "Q3_SetInterface: NOT SUPPORTED IN MP\n");
        }

        SET_SHIELDS => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetShields(entID, QTRUE);
            } else {
                Q3_SetShields(entID, QFALSE);
            }
        }

        SET_SABERACTIVE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetSaberActive(entID, QTRUE);
            } else {
                Q3_SetSaberActive(entID, QFALSE);
            }
        }

        SET_ADJUST_AREA_PORTALS => {
            G_DebugPrint(WL_WARNING, "Q3_SetAdjustAreaPortals: NOT SUPPORTED IN MP\n");
        }

        SET_DMG_BY_HEAVY_WEAP_ONLY => {
            G_DebugPrint(WL_WARNING, "Q3_SetDmgByHeavyWeapOnly: NOT SUPPORTED IN MP\n");
        }

        SET_SHIELDED => {
            G_DebugPrint(WL_WARNING, "Q3_SetShielded: NOT SUPPORTED IN MP\n");
        }

        SET_NO_GROUPS => {
            G_DebugPrint(WL_WARNING, "Q3_SetNoGroups: NOT SUPPORTED IN MP\n");
        }

        SET_FIRE_WEAPON => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetFireWeapon(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetFireWeapon(entID, QFALSE);
            }
        }

        SET_INACTIVE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetInactive(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetInactive(entID, QFALSE);
            } else if Q_stricmp(c"unlocked".as_ptr(), data) == 0 {
                UnLockDoors((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize));
            } else if Q_stricmp(c"locked".as_ptr(), data) == 0 {
                LockDoors((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize));
            }
        }
        SET_END_SCREENDISSOLVE => {
            G_DebugPrint(WL_WARNING, "SET_END_SCREENDISSOLVE: NOT SUPPORTED IN MP\n");
        }

        SET_MISSION_STATUS_SCREEN => {
            //Cvar_Set("cg_missionstatusscreen", "1");
            G_DebugPrint(WL_WARNING, "SET_MISSION_STATUS_SCREEN: NOT SUPPORTED IN MP\n");
        }

        SET_FUNC_USABLE_VISIBLE => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetFuncUsableVisible(entID, QTRUE);
            } else if Q_stricmp(c"false".as_ptr(), data) == 0 {
                Q3_SetFuncUsableVisible(entID, QFALSE);
            }
        }

        SET_NO_KNOCKBACK => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetNoKnockback(entID, QTRUE);
            } else {
                Q3_SetNoKnockback(entID, QFALSE);
            }
        }

        SET_VIDEO_PLAY => {
            // don't do this check now, James doesn't want a scripted cinematic to also skip any Video cinematics as well,
            //	the "timescale" and "skippingCinematic" cvars will be set back to normal in the Video code, so doing a
            //	skip will now only skip one section of a multiple-part story (eg VOY1 bridge sequence)
            //
            //		if ( g_timescale->value <= 1.0f )
            {
                G_DebugPrint(WL_WARNING, "SET_VIDEO_PLAY: NOT SUPPORTED IN MP\n");
                //SV_SendConsoleCommand( va("inGameCinematic %s\n", (char *)data) );
            }
        }

        SET_VIDEO_FADE_IN => {
            G_DebugPrint(WL_WARNING, "SET_VIDEO_FADE_IN: NOT SUPPORTED IN MP\n");
        }

        SET_VIDEO_FADE_OUT => {
            G_DebugPrint(WL_WARNING, "SET_VIDEO_FADE_OUT: NOT SUPPORTED IN MP\n");
        }
        SET_REMOVE_TARGET => {
            Q3_SetRemoveTarget(entID, data);
        }

        SET_LOADGAME => {
            //gi.SendConsoleCommand( va("load %s\n", (const char *) data ) );
            G_DebugPrint(WL_WARNING, "SET_LOADGAME: NOT SUPPORTED IN MP\n");
        }

        SET_MENU_SCREEN => {
            //UI_SetActiveMenu( (const char *) data );
        }

        SET_OBJECTIVE_SHOW => {
            G_DebugPrint(WL_WARNING, "SET_OBJECTIVE_SHOW: NOT SUPPORTED IN MP\n");
        }
        SET_OBJECTIVE_HIDE => {
            G_DebugPrint(WL_WARNING, "SET_OBJECTIVE_HIDE: NOT SUPPORTED IN MP\n");
        }
        SET_OBJECTIVE_SUCCEEDED => {
            G_DebugPrint(WL_WARNING, "SET_OBJECTIVE_SUCCEEDED: NOT SUPPORTED IN MP\n");
        }
        SET_OBJECTIVE_FAILED => {
            G_DebugPrint(WL_WARNING, "SET_OBJECTIVE_FAILED: NOT SUPPORTED IN MP\n");
        }

        SET_OBJECTIVE_CLEARALL => {
            G_DebugPrint(WL_WARNING, "SET_OBJECTIVE_CLEARALL: NOT SUPPORTED IN MP\n");
        }

        SET_MISSIONFAILED => {
            G_DebugPrint(WL_WARNING, "SET_MISSIONFAILED: NOT SUPPORTED IN MP\n");
        }

        SET_MISSIONSTATUSTEXT => {
            G_DebugPrint(WL_WARNING, "SET_MISSIONSTATUSTEXT: NOT SUPPORTED IN MP\n");
        }

        SET_MISSIONSTATUSTIME => {
            G_DebugPrint(WL_WARNING, "SET_MISSIONSTATUSTIME: NOT SUPPORTED IN MP\n");
        }

        SET_CLOSINGCREDITS => {
            G_DebugPrint(WL_WARNING, "SET_CLOSINGCREDITS: NOT SUPPORTED IN MP\n");
        }

        SET_SKILL => {
            //		//can never be set
        }

        SET_FULLNAME => {
            Q3_SetFullName(entID, data);
        }

        SET_DISABLE_SHADER_ANIM => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetDisableShaderAnims(entID, QTRUE);
            } else {
                Q3_SetDisableShaderAnims(entID, QFALSE);
            }
        }

        SET_SHADER_ANIM => {
            if Q_stricmp(c"true".as_ptr(), data) == 0 {
                Q3_SetShaderAnim(entID, QTRUE);
            } else {
                Q3_SetShaderAnim(entID, QFALSE);
            }
        }

        SET_MUSIC_STATE => {
            Q3_SetMusicState(data);
        }

        SET_CLEAN_DAMAGING_ENTS => {
            Q3_SetCleanDamagingEnts();
        }

        SET_HUD => {
            G_DebugPrint(WL_WARNING, "SET_HUD: NOT SUPPORTED IN MP\n");
        }

        SET_FORCE_HEAL_LEVEL | SET_FORCE_JUMP_LEVEL | SET_FORCE_SPEED_LEVEL
        | SET_FORCE_PUSH_LEVEL | SET_FORCE_PULL_LEVEL | SET_FORCE_MINDTRICK_LEVEL
        | SET_FORCE_GRIP_LEVEL | SET_FORCE_LIGHTNING_LEVEL | SET_SABER_THROW
        | SET_SABER_DEFENSE | SET_SABER_OFFENSE => {
            int_data = atoi(data);
            Q3_SetForcePowerLevel(entID, toSet - SET_FORCE_HEAL_LEVEL, int_data);
        }

        _ => {
            //G_DebugPrint( WL_ERROR, "Q3_Set: '%s' is not a valid set field\n", type_name );
            trap::ICARUS_SetVar(taskID, entID, type_name, data);
        }
    }

    QTRUE
}

// Helper: render a possibly-NULL `char*` as glibc printf would ("(null)").
// Mirrors the `%s` of a NULL `ent->targetname` in the C warning strings.
fn cstr_or_null(p: *const c_char) -> String {
    if p.is_null() {
        "(null)".into()
    } else {
        unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() }
    }
}

// `strncpy( dst, src, n )` semantics: copy up to `n` bytes from the NUL-terminated `src`,
// NUL-padding the remainder of `dst[0..n]` (matching C strncpy, which is what Q3_SetParm uses).
unsafe fn strncpy_field(dst: *mut c_char, src: *const c_char, n: usize) {
    let mut i = 0usize;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
}


// =====================================================================================
// ICARUS register set/get layer — the `setType_t` enum, the `setTable[]` name→ID table,
// and the `VTYPE_*` variable-type enum, plus the `Q3_Get*` register-read callbacks and the
// behavior-state (`Q3_Set*BState`) / behaviorSet setters that the `Q3_Set` dispatch funnels
// into. The enum + table live verbatim from `icarus/q3_interface.h` (`setType_t`) and the
// head of `g_ICARUScb.c` (`stringID_table_t setTable[]`); `GetIDForString(setTable,name)`
// resolves a script field name to its `SET_*` id. All entity-state reads/writes, hence
// no-oracle. (`Q3_Set`/`Q3_SetEnemy`/`Q3_SetSaberActive` stay unported — blocked.)
// =====================================================================================

// `enum //# setType_e` (icarus/q3_interface.h:6) — ICARUS register/field ids. `SET_PARM1=0`,
// the rest follow declaration order; the commented-out `SET_OBJECTIVEFOSTER` is omitted (as
// in the C) so the trailing ids match. The final `SET_` is the table terminator sentinel.
pub const SET_PARM1: c_int = 0;
pub const SET_PARM2: c_int = 1;
pub const SET_PARM3: c_int = 2;
pub const SET_PARM4: c_int = 3;
pub const SET_PARM5: c_int = 4;
pub const SET_PARM6: c_int = 5;
pub const SET_PARM7: c_int = 6;
pub const SET_PARM8: c_int = 7;
pub const SET_PARM9: c_int = 8;
pub const SET_PARM10: c_int = 9;
pub const SET_PARM11: c_int = 10;
pub const SET_PARM12: c_int = 11;
pub const SET_PARM13: c_int = 12;
pub const SET_PARM14: c_int = 13;
pub const SET_PARM15: c_int = 14;
pub const SET_PARM16: c_int = 15;
pub const SET_SPAWNSCRIPT: c_int = 16;
pub const SET_USESCRIPT: c_int = 17;
pub const SET_AWAKESCRIPT: c_int = 18;
pub const SET_ANGERSCRIPT: c_int = 19;
pub const SET_ATTACKSCRIPT: c_int = 20;
pub const SET_VICTORYSCRIPT: c_int = 21;
pub const SET_LOSTENEMYSCRIPT: c_int = 22;
pub const SET_PAINSCRIPT: c_int = 23;
pub const SET_FLEESCRIPT: c_int = 24;
pub const SET_DEATHSCRIPT: c_int = 25;
pub const SET_DELAYEDSCRIPT: c_int = 26;
pub const SET_BLOCKEDSCRIPT: c_int = 27;
pub const SET_FFIRESCRIPT: c_int = 28;
pub const SET_FFDEATHSCRIPT: c_int = 29;
pub const SET_MINDTRICKSCRIPT: c_int = 30;
pub const SET_VIDEO_PLAY: c_int = 31;
pub const SET_CINEMATIC_SKIPSCRIPT: c_int = 32;
pub const SET_ENEMY: c_int = 33;
pub const SET_LEADER: c_int = 34;
pub const SET_NAVGOAL: c_int = 35;
pub const SET_CAPTURE: c_int = 36;
pub const SET_VIEWTARGET: c_int = 37;
pub const SET_WATCHTARGET: c_int = 38;
pub const SET_TARGETNAME: c_int = 39;
pub const SET_PAINTARGET: c_int = 40;
pub const SET_CAMERA_GROUP: c_int = 41;
pub const SET_CAMERA_GROUP_TAG: c_int = 42;
pub const SET_LOOK_TARGET: c_int = 43;
pub const SET_ADDRHANDBOLT_MODEL: c_int = 44;
pub const SET_REMOVERHANDBOLT_MODEL: c_int = 45;
pub const SET_ADDLHANDBOLT_MODEL: c_int = 46;
pub const SET_REMOVELHANDBOLT_MODEL: c_int = 47;
pub const SET_CAPTIONTEXTCOLOR: c_int = 48;
pub const SET_CENTERTEXTCOLOR: c_int = 49;
pub const SET_SCROLLTEXTCOLOR: c_int = 50;
pub const SET_COPY_ORIGIN: c_int = 51;
pub const SET_DEFEND_TARGET: c_int = 52;
pub const SET_TARGET: c_int = 53;
pub const SET_TARGET2: c_int = 54;
pub const SET_LOCATION: c_int = 55;
pub const SET_REMOVE_TARGET: c_int = 56;
pub const SET_LOADGAME: c_int = 57;
pub const SET_LOCKYAW: c_int = 58;
pub const SET_FULLNAME: c_int = 59;
pub const SET_VIEWENTITY: c_int = 60;
pub const SET_LOOPSOUND: c_int = 61;
pub const SET_ICARUS_FREEZE: c_int = 62;
pub const SET_ICARUS_UNFREEZE: c_int = 63;
pub const SET_SCROLLTEXT: c_int = 64;
pub const SET_LCARSTEXT: c_int = 65;
pub const SET_ORIGIN: c_int = 66;
pub const SET_ANGLES: c_int = 67;
pub const SET_TELEPORT_DEST: c_int = 68;
pub const SET_XVELOCITY: c_int = 69;
pub const SET_YVELOCITY: c_int = 70;
pub const SET_ZVELOCITY: c_int = 71;
pub const SET_Z_OFFSET: c_int = 72;
pub const SET_DPITCH: c_int = 73;
pub const SET_DYAW: c_int = 74;
pub const SET_TIMESCALE: c_int = 75;
pub const SET_CAMERA_GROUP_Z_OFS: c_int = 76;
pub const SET_VISRANGE: c_int = 77;
pub const SET_EARSHOT: c_int = 78;
pub const SET_VIGILANCE: c_int = 79;
pub const SET_GRAVITY: c_int = 80;
pub const SET_FACEAUX: c_int = 81;
pub const SET_FACEBLINK: c_int = 82;
pub const SET_FACEBLINKFROWN: c_int = 83;
pub const SET_FACEFROWN: c_int = 84;
pub const SET_FACENORMAL: c_int = 85;
pub const SET_FACEEYESCLOSED: c_int = 86;
pub const SET_FACEEYESOPENED: c_int = 87;
pub const SET_WAIT: c_int = 88;
pub const SET_FOLLOWDIST: c_int = 89;
pub const SET_SCALE: c_int = 90;
pub const SET_ANIM_HOLDTIME_LOWER: c_int = 91;
pub const SET_ANIM_HOLDTIME_UPPER: c_int = 92;
pub const SET_ANIM_HOLDTIME_BOTH: c_int = 93;
pub const SET_HEALTH: c_int = 94;
pub const SET_ARMOR: c_int = 95;
pub const SET_WALKSPEED: c_int = 96;
pub const SET_RUNSPEED: c_int = 97;
pub const SET_YAWSPEED: c_int = 98;
pub const SET_AGGRESSION: c_int = 99;
pub const SET_AIM: c_int = 100;
pub const SET_FRICTION: c_int = 101;
pub const SET_SHOOTDIST: c_int = 102;
pub const SET_HFOV: c_int = 103;
pub const SET_VFOV: c_int = 104;
pub const SET_DELAYSCRIPTTIME: c_int = 105;
pub const SET_FORWARDMOVE: c_int = 106;
pub const SET_RIGHTMOVE: c_int = 107;
pub const SET_STARTFRAME: c_int = 108;
pub const SET_ENDFRAME: c_int = 109;
pub const SET_ANIMFRAME: c_int = 110;
pub const SET_COUNT: c_int = 111;
pub const SET_SHOT_SPACING: c_int = 112;
pub const SET_MISSIONSTATUSTIME: c_int = 113;
pub const SET_WIDTH: c_int = 114;
pub const SET_IGNOREPAIN: c_int = 115;
pub const SET_IGNOREENEMIES: c_int = 116;
pub const SET_IGNOREALERTS: c_int = 117;
pub const SET_DONTSHOOT: c_int = 118;
pub const SET_NOTARGET: c_int = 119;
pub const SET_DONTFIRE: c_int = 120;
pub const SET_LOCKED_ENEMY: c_int = 121;
pub const SET_CROUCHED: c_int = 122;
pub const SET_WALKING: c_int = 123;
pub const SET_RUNNING: c_int = 124;
pub const SET_CHASE_ENEMIES: c_int = 125;
pub const SET_LOOK_FOR_ENEMIES: c_int = 126;
pub const SET_FACE_MOVE_DIR: c_int = 127;
pub const SET_DONT_FLEE: c_int = 128;
pub const SET_FORCED_MARCH: c_int = 129;
pub const SET_UNDYING: c_int = 130;
pub const SET_NOAVOID: c_int = 131;
pub const SET_SOLID: c_int = 132;
pub const SET_PLAYER_USABLE: c_int = 133;
pub const SET_LOOP_ANIM: c_int = 134;
pub const SET_INTERFACE: c_int = 135;
pub const SET_SHIELDS: c_int = 136;
pub const SET_INVISIBLE: c_int = 137;
pub const SET_VAMPIRE: c_int = 138;
pub const SET_FORCE_INVINCIBLE: c_int = 139;
pub const SET_GREET_ALLIES: c_int = 140;
pub const SET_VIDEO_FADE_IN: c_int = 141;
pub const SET_VIDEO_FADE_OUT: c_int = 142;
pub const SET_PLAYER_LOCKED: c_int = 143;
pub const SET_LOCK_PLAYER_WEAPONS: c_int = 144;
pub const SET_NO_IMPACT_DAMAGE: c_int = 145;
pub const SET_NO_KNOCKBACK: c_int = 146;
pub const SET_ALT_FIRE: c_int = 147;
pub const SET_NO_RESPONSE: c_int = 148;
pub const SET_INVINCIBLE: c_int = 149;
pub const SET_MISSIONSTATUSACTIVE: c_int = 150;
pub const SET_NO_COMBAT_TALK: c_int = 151;
pub const SET_NO_ALERT_TALK: c_int = 152;
pub const SET_TREASONED: c_int = 153;
pub const SET_DISABLE_SHADER_ANIM: c_int = 154;
pub const SET_SHADER_ANIM: c_int = 155;
pub const SET_SABERACTIVE: c_int = 156;
pub const SET_ADJUST_AREA_PORTALS: c_int = 157;
pub const SET_DMG_BY_HEAVY_WEAP_ONLY: c_int = 158;
pub const SET_SHIELDED: c_int = 159;
pub const SET_NO_GROUPS: c_int = 160;
pub const SET_FIRE_WEAPON: c_int = 161;
pub const SET_NO_MINDTRICK: c_int = 162;
pub const SET_INACTIVE: c_int = 163;
pub const SET_FUNC_USABLE_VISIBLE: c_int = 164;
pub const SET_SECRET_AREA_FOUND: c_int = 165;
pub const SET_MISSION_STATUS_SCREEN: c_int = 166;
pub const SET_END_SCREENDISSOLVE: c_int = 167;
pub const SET_USE_CP_NEAREST: c_int = 168;
pub const SET_MORELIGHT: c_int = 169;
pub const SET_NO_FORCE: c_int = 170;
pub const SET_NO_FALLTODEATH: c_int = 171;
pub const SET_DISMEMBERABLE: c_int = 172;
pub const SET_NO_ACROBATICS: c_int = 173;
pub const SET_USE_SUBTITLES: c_int = 174;
pub const SET_CLEAN_DAMAGING_ENTS: c_int = 175;
pub const SET_HUD: c_int = 176;
pub const SET_SKILL: c_int = 177;
pub const SET_ANIM_UPPER: c_int = 178;
pub const SET_ANIM_LOWER: c_int = 179;
pub const SET_ANIM_BOTH: c_int = 180;
pub const SET_PLAYER_TEAM: c_int = 181;
pub const SET_ENEMY_TEAM: c_int = 182;
pub const SET_BEHAVIOR_STATE: c_int = 183;
pub const SET_DEFAULT_BSTATE: c_int = 184;
pub const SET_TEMP_BSTATE: c_int = 185;
pub const SET_EVENT: c_int = 186;
pub const SET_WEAPON: c_int = 187;
pub const SET_ITEM: c_int = 188;
pub const SET_MUSIC_STATE: c_int = 189;
pub const SET_FORCE_HEAL_LEVEL: c_int = 190;
pub const SET_FORCE_JUMP_LEVEL: c_int = 191;
pub const SET_FORCE_SPEED_LEVEL: c_int = 192;
pub const SET_FORCE_PUSH_LEVEL: c_int = 193;
pub const SET_FORCE_PULL_LEVEL: c_int = 194;
pub const SET_FORCE_MINDTRICK_LEVEL: c_int = 195;
pub const SET_FORCE_GRIP_LEVEL: c_int = 196;
pub const SET_FORCE_LIGHTNING_LEVEL: c_int = 197;
pub const SET_SABER_THROW: c_int = 198;
pub const SET_SABER_DEFENSE: c_int = 199;
pub const SET_SABER_OFFENSE: c_int = 200;
pub const SET_OBJECTIVE_SHOW: c_int = 201;
pub const SET_OBJECTIVE_HIDE: c_int = 202;
pub const SET_OBJECTIVE_SUCCEEDED: c_int = 203;
pub const SET_OBJECTIVE_FAILED: c_int = 204;
pub const SET_MISSIONFAILED: c_int = 205;
pub const SET_TACTICAL_SHOW: c_int = 206;
pub const SET_TACTICAL_HIDE: c_int = 207;
pub const SET_OBJECTIVE_CLEARALL: c_int = 208;
pub const SET_MISSIONSTATUSTEXT: c_int = 209;
pub const SET_MENU_SCREEN: c_int = 210;
pub const SET_CLOSINGCREDITS: c_int = 211;
pub const SET_LEAN: c_int = 212;
pub const SET_: c_int = 213;

// `enum { VTYPE_NONE=0, VTYPE_FLOAT, VTYPE_STRING, VTYPE_VECTOR }` (icarus/q3_registers.h:4) —
// declared-variable types returned by `trap_ICARUS_VariableDeclared`.
pub const VTYPE_FLOAT: c_int = 1;
pub const VTYPE_STRING: c_int = 2;
pub const VTYPE_VECTOR: c_int = 3;

// `stringID_table_t setTable[]` (g_ICARUScb.c:51) — maps each script field name to its
// `SET_*` id via the C `ENUM2STRING(x)` macro (`{ "x", x }`). Kept in the exact C order
// (note the duplicate `SET_BEHAVIOR_STATE` row, verbatim from the source) so
// `GetIDForString` resolves identically. Terminated by `{ "", SET_ }`.
const fn s(name: &'static CStr, id: c_int) -> stringID_table_t {
    stringID_table_t { name: name.as_ptr(), id }
}
pub static mut setTable: [stringID_table_t; 212] = [
    s(c"SET_SPAWNSCRIPT", SET_SPAWNSCRIPT),
    s(c"SET_USESCRIPT", SET_USESCRIPT),
    s(c"SET_AWAKESCRIPT", SET_AWAKESCRIPT),
    s(c"SET_ANGERSCRIPT", SET_ANGERSCRIPT),
    s(c"SET_ATTACKSCRIPT", SET_ATTACKSCRIPT),
    s(c"SET_VICTORYSCRIPT", SET_VICTORYSCRIPT),
    s(c"SET_PAINSCRIPT", SET_PAINSCRIPT),
    s(c"SET_FLEESCRIPT", SET_FLEESCRIPT),
    s(c"SET_DEATHSCRIPT", SET_DEATHSCRIPT),
    s(c"SET_DELAYEDSCRIPT", SET_DELAYEDSCRIPT),
    s(c"SET_BLOCKEDSCRIPT", SET_BLOCKEDSCRIPT),
    s(c"SET_FFIRESCRIPT", SET_FFIRESCRIPT),
    s(c"SET_FFDEATHSCRIPT", SET_FFDEATHSCRIPT),
    s(c"SET_MINDTRICKSCRIPT", SET_MINDTRICKSCRIPT),
    s(c"SET_NO_MINDTRICK", SET_NO_MINDTRICK),
    s(c"SET_ORIGIN", SET_ORIGIN),
    s(c"SET_TELEPORT_DEST", SET_TELEPORT_DEST),
    s(c"SET_ANGLES", SET_ANGLES),
    s(c"SET_XVELOCITY", SET_XVELOCITY),
    s(c"SET_YVELOCITY", SET_YVELOCITY),
    s(c"SET_ZVELOCITY", SET_ZVELOCITY),
    s(c"SET_Z_OFFSET", SET_Z_OFFSET),
    s(c"SET_ENEMY", SET_ENEMY),
    s(c"SET_LEADER", SET_LEADER),
    s(c"SET_NAVGOAL", SET_NAVGOAL),
    s(c"SET_ANIM_UPPER", SET_ANIM_UPPER),
    s(c"SET_ANIM_LOWER", SET_ANIM_LOWER),
    s(c"SET_ANIM_BOTH", SET_ANIM_BOTH),
    s(c"SET_ANIM_HOLDTIME_LOWER", SET_ANIM_HOLDTIME_LOWER),
    s(c"SET_ANIM_HOLDTIME_UPPER", SET_ANIM_HOLDTIME_UPPER),
    s(c"SET_ANIM_HOLDTIME_BOTH", SET_ANIM_HOLDTIME_BOTH),
    s(c"SET_PLAYER_TEAM", SET_PLAYER_TEAM),
    s(c"SET_ENEMY_TEAM", SET_ENEMY_TEAM),
    s(c"SET_BEHAVIOR_STATE", SET_BEHAVIOR_STATE),
    s(c"SET_BEHAVIOR_STATE", SET_BEHAVIOR_STATE),
    s(c"SET_HEALTH", SET_HEALTH),
    s(c"SET_ARMOR", SET_ARMOR),
    s(c"SET_DEFAULT_BSTATE", SET_DEFAULT_BSTATE),
    s(c"SET_CAPTURE", SET_CAPTURE),
    s(c"SET_DPITCH", SET_DPITCH),
    s(c"SET_DYAW", SET_DYAW),
    s(c"SET_EVENT", SET_EVENT),
    s(c"SET_TEMP_BSTATE", SET_TEMP_BSTATE),
    s(c"SET_COPY_ORIGIN", SET_COPY_ORIGIN),
    s(c"SET_VIEWTARGET", SET_VIEWTARGET),
    s(c"SET_WEAPON", SET_WEAPON),
    s(c"SET_ITEM", SET_ITEM),
    s(c"SET_WALKSPEED", SET_WALKSPEED),
    s(c"SET_RUNSPEED", SET_RUNSPEED),
    s(c"SET_YAWSPEED", SET_YAWSPEED),
    s(c"SET_AGGRESSION", SET_AGGRESSION),
    s(c"SET_AIM", SET_AIM),
    s(c"SET_FRICTION", SET_FRICTION),
    s(c"SET_GRAVITY", SET_GRAVITY),
    s(c"SET_IGNOREPAIN", SET_IGNOREPAIN),
    s(c"SET_IGNOREENEMIES", SET_IGNOREENEMIES),
    s(c"SET_IGNOREALERTS", SET_IGNOREALERTS),
    s(c"SET_DONTSHOOT", SET_DONTSHOOT),
    s(c"SET_DONTFIRE", SET_DONTFIRE),
    s(c"SET_LOCKED_ENEMY", SET_LOCKED_ENEMY),
    s(c"SET_NOTARGET", SET_NOTARGET),
    s(c"SET_LEAN", SET_LEAN),
    s(c"SET_CROUCHED", SET_CROUCHED),
    s(c"SET_WALKING", SET_WALKING),
    s(c"SET_RUNNING", SET_RUNNING),
    s(c"SET_CHASE_ENEMIES", SET_CHASE_ENEMIES),
    s(c"SET_LOOK_FOR_ENEMIES", SET_LOOK_FOR_ENEMIES),
    s(c"SET_FACE_MOVE_DIR", SET_FACE_MOVE_DIR),
    s(c"SET_ALT_FIRE", SET_ALT_FIRE),
    s(c"SET_DONT_FLEE", SET_DONT_FLEE),
    s(c"SET_FORCED_MARCH", SET_FORCED_MARCH),
    s(c"SET_NO_RESPONSE", SET_NO_RESPONSE),
    s(c"SET_NO_COMBAT_TALK", SET_NO_COMBAT_TALK),
    s(c"SET_NO_ALERT_TALK", SET_NO_ALERT_TALK),
    s(c"SET_UNDYING", SET_UNDYING),
    s(c"SET_TREASONED", SET_TREASONED),
    s(c"SET_DISABLE_SHADER_ANIM", SET_DISABLE_SHADER_ANIM),
    s(c"SET_SHADER_ANIM", SET_SHADER_ANIM),
    s(c"SET_INVINCIBLE", SET_INVINCIBLE),
    s(c"SET_NOAVOID", SET_NOAVOID),
    s(c"SET_SHOOTDIST", SET_SHOOTDIST),
    s(c"SET_TARGETNAME", SET_TARGETNAME),
    s(c"SET_TARGET", SET_TARGET),
    s(c"SET_TARGET2", SET_TARGET2),
    s(c"SET_LOCATION", SET_LOCATION),
    s(c"SET_PAINTARGET", SET_PAINTARGET),
    s(c"SET_TIMESCALE", SET_TIMESCALE),
    s(c"SET_VISRANGE", SET_VISRANGE),
    s(c"SET_EARSHOT", SET_EARSHOT),
    s(c"SET_VIGILANCE", SET_VIGILANCE),
    s(c"SET_HFOV", SET_HFOV),
    s(c"SET_VFOV", SET_VFOV),
    s(c"SET_DELAYSCRIPTTIME", SET_DELAYSCRIPTTIME),
    s(c"SET_FORWARDMOVE", SET_FORWARDMOVE),
    s(c"SET_RIGHTMOVE", SET_RIGHTMOVE),
    s(c"SET_LOCKYAW", SET_LOCKYAW),
    s(c"SET_SOLID", SET_SOLID),
    s(c"SET_CAMERA_GROUP", SET_CAMERA_GROUP),
    s(c"SET_CAMERA_GROUP_Z_OFS", SET_CAMERA_GROUP_Z_OFS),
    s(c"SET_CAMERA_GROUP_TAG", SET_CAMERA_GROUP_TAG),
    s(c"SET_LOOK_TARGET", SET_LOOK_TARGET),
    s(c"SET_ADDRHANDBOLT_MODEL", SET_ADDRHANDBOLT_MODEL),
    s(c"SET_REMOVERHANDBOLT_MODEL", SET_REMOVERHANDBOLT_MODEL),
    s(c"SET_ADDLHANDBOLT_MODEL", SET_ADDLHANDBOLT_MODEL),
    s(c"SET_REMOVELHANDBOLT_MODEL", SET_REMOVELHANDBOLT_MODEL),
    s(c"SET_FACEAUX", SET_FACEAUX),
    s(c"SET_FACEBLINK", SET_FACEBLINK),
    s(c"SET_FACEBLINKFROWN", SET_FACEBLINKFROWN),
    s(c"SET_FACEFROWN", SET_FACEFROWN),
    s(c"SET_FACENORMAL", SET_FACENORMAL),
    s(c"SET_FACEEYESCLOSED", SET_FACEEYESCLOSED),
    s(c"SET_FACEEYESOPENED", SET_FACEEYESOPENED),
    s(c"SET_SCROLLTEXT", SET_SCROLLTEXT),
    s(c"SET_LCARSTEXT", SET_LCARSTEXT),
    s(c"SET_SCROLLTEXTCOLOR", SET_SCROLLTEXTCOLOR),
    s(c"SET_CAPTIONTEXTCOLOR", SET_CAPTIONTEXTCOLOR),
    s(c"SET_CENTERTEXTCOLOR", SET_CENTERTEXTCOLOR),
    s(c"SET_PLAYER_USABLE", SET_PLAYER_USABLE),
    s(c"SET_STARTFRAME", SET_STARTFRAME),
    s(c"SET_ENDFRAME", SET_ENDFRAME),
    s(c"SET_ANIMFRAME", SET_ANIMFRAME),
    s(c"SET_LOOP_ANIM", SET_LOOP_ANIM),
    s(c"SET_INTERFACE", SET_INTERFACE),
    s(c"SET_SHIELDS", SET_SHIELDS),
    s(c"SET_NO_KNOCKBACK", SET_NO_KNOCKBACK),
    s(c"SET_INVISIBLE", SET_INVISIBLE),
    s(c"SET_VAMPIRE", SET_VAMPIRE),
    s(c"SET_FORCE_INVINCIBLE", SET_FORCE_INVINCIBLE),
    s(c"SET_GREET_ALLIES", SET_GREET_ALLIES),
    s(c"SET_PLAYER_LOCKED", SET_PLAYER_LOCKED),
    s(c"SET_LOCK_PLAYER_WEAPONS", SET_LOCK_PLAYER_WEAPONS),
    s(c"SET_NO_IMPACT_DAMAGE", SET_NO_IMPACT_DAMAGE),
    s(c"SET_PARM1", SET_PARM1),
    s(c"SET_PARM2", SET_PARM2),
    s(c"SET_PARM3", SET_PARM3),
    s(c"SET_PARM4", SET_PARM4),
    s(c"SET_PARM5", SET_PARM5),
    s(c"SET_PARM6", SET_PARM6),
    s(c"SET_PARM7", SET_PARM7),
    s(c"SET_PARM8", SET_PARM8),
    s(c"SET_PARM9", SET_PARM9),
    s(c"SET_PARM10", SET_PARM10),
    s(c"SET_PARM11", SET_PARM11),
    s(c"SET_PARM12", SET_PARM12),
    s(c"SET_PARM13", SET_PARM13),
    s(c"SET_PARM14", SET_PARM14),
    s(c"SET_PARM15", SET_PARM15),
    s(c"SET_PARM16", SET_PARM16),
    s(c"SET_DEFEND_TARGET", SET_DEFEND_TARGET),
    s(c"SET_WAIT", SET_WAIT),
    s(c"SET_COUNT", SET_COUNT),
    s(c"SET_SHOT_SPACING", SET_SHOT_SPACING),
    s(c"SET_VIDEO_PLAY", SET_VIDEO_PLAY),
    s(c"SET_VIDEO_FADE_IN", SET_VIDEO_FADE_IN),
    s(c"SET_VIDEO_FADE_OUT", SET_VIDEO_FADE_OUT),
    s(c"SET_REMOVE_TARGET", SET_REMOVE_TARGET),
    s(c"SET_LOADGAME", SET_LOADGAME),
    s(c"SET_MENU_SCREEN", SET_MENU_SCREEN),
    s(c"SET_OBJECTIVE_SHOW", SET_OBJECTIVE_SHOW),
    s(c"SET_OBJECTIVE_HIDE", SET_OBJECTIVE_HIDE),
    s(c"SET_OBJECTIVE_SUCCEEDED", SET_OBJECTIVE_SUCCEEDED),
    s(c"SET_OBJECTIVE_FAILED", SET_OBJECTIVE_FAILED),
    s(c"SET_MISSIONFAILED", SET_MISSIONFAILED),
    s(c"SET_TACTICAL_SHOW", SET_TACTICAL_SHOW),
    s(c"SET_TACTICAL_HIDE", SET_TACTICAL_HIDE),
    s(c"SET_FOLLOWDIST", SET_FOLLOWDIST),
    s(c"SET_SCALE", SET_SCALE),
    s(c"SET_OBJECTIVE_CLEARALL", SET_OBJECTIVE_CLEARALL),
    s(c"SET_MISSIONSTATUSTEXT", SET_MISSIONSTATUSTEXT),
    s(c"SET_WIDTH", SET_WIDTH),
    s(c"SET_CLOSINGCREDITS", SET_CLOSINGCREDITS),
    s(c"SET_SKILL", SET_SKILL),
    s(c"SET_MISSIONSTATUSTIME", SET_MISSIONSTATUSTIME),
    s(c"SET_FULLNAME", SET_FULLNAME),
    s(c"SET_FORCE_HEAL_LEVEL", SET_FORCE_HEAL_LEVEL),
    s(c"SET_FORCE_JUMP_LEVEL", SET_FORCE_JUMP_LEVEL),
    s(c"SET_FORCE_SPEED_LEVEL", SET_FORCE_SPEED_LEVEL),
    s(c"SET_FORCE_PUSH_LEVEL", SET_FORCE_PUSH_LEVEL),
    s(c"SET_FORCE_PULL_LEVEL", SET_FORCE_PULL_LEVEL),
    s(c"SET_FORCE_MINDTRICK_LEVEL", SET_FORCE_MINDTRICK_LEVEL),
    s(c"SET_FORCE_GRIP_LEVEL", SET_FORCE_GRIP_LEVEL),
    s(c"SET_FORCE_LIGHTNING_LEVEL", SET_FORCE_LIGHTNING_LEVEL),
    s(c"SET_SABER_THROW", SET_SABER_THROW),
    s(c"SET_SABER_DEFENSE", SET_SABER_DEFENSE),
    s(c"SET_SABER_OFFENSE", SET_SABER_OFFENSE),
    s(c"SET_VIEWENTITY", SET_VIEWENTITY),
    s(c"SET_WATCHTARGET", SET_WATCHTARGET),
    s(c"SET_SABERACTIVE", SET_SABERACTIVE),
    s(c"SET_ADJUST_AREA_PORTALS", SET_ADJUST_AREA_PORTALS),
    s(c"SET_DMG_BY_HEAVY_WEAP_ONLY", SET_DMG_BY_HEAVY_WEAP_ONLY),
    s(c"SET_SHIELDED", SET_SHIELDED),
    s(c"SET_NO_GROUPS", SET_NO_GROUPS),
    s(c"SET_FIRE_WEAPON", SET_FIRE_WEAPON),
    s(c"SET_INACTIVE", SET_INACTIVE),
    s(c"SET_FUNC_USABLE_VISIBLE", SET_FUNC_USABLE_VISIBLE),
    s(c"SET_MISSION_STATUS_SCREEN", SET_MISSION_STATUS_SCREEN),
    s(c"SET_END_SCREENDISSOLVE", SET_END_SCREENDISSOLVE),
    s(c"SET_LOOPSOUND", SET_LOOPSOUND),
    s(c"SET_ICARUS_FREEZE", SET_ICARUS_FREEZE),
    s(c"SET_ICARUS_UNFREEZE", SET_ICARUS_UNFREEZE),
    s(c"SET_USE_CP_NEAREST", SET_USE_CP_NEAREST),
    s(c"SET_MORELIGHT", SET_MORELIGHT),
    s(c"SET_CINEMATIC_SKIPSCRIPT", SET_CINEMATIC_SKIPSCRIPT),
    s(c"SET_NO_FORCE", SET_NO_FORCE),
    s(c"SET_NO_FALLTODEATH", SET_NO_FALLTODEATH),
    s(c"SET_DISMEMBERABLE", SET_DISMEMBERABLE),
    s(c"SET_NO_ACROBATICS", SET_NO_ACROBATICS),
    s(c"SET_MUSIC_STATE", SET_MUSIC_STATE),
    s(c"SET_USE_SUBTITLES", SET_USE_SUBTITLES),
    s(c"SET_CLEAN_DAMAGING_ENTS", SET_CLEAN_DAMAGING_ENTS),
    s(c"SET_HUD", SET_HUD),
    s(c"", SET_),
];

// interpreter.h `enum` type-IDs. Derived through the generated chain (tokenizer.h
// `TK_USERDEF`=8 → interpreter.h `NUM_USER_TOKENS`=19 → `ID_AFFECT`=19 … `NUM_IDS`=51 →
// `TYPE_WAIT_COMPLETE`=51, `TYPE_WAIT_TRIGGERED`=52, `TYPE_ANGLES`=53, `TYPE_ORIGIN`=54).
const TYPE_ANGLES: c_int = 53;
const TYPE_ORIGIN: c_int = 54;

/// `int Q3_GetTag( int entID, const char *name, int lookup, vec3_t info )`
/// (g_ICARUScb.c:948) — gets the value of a tag by name, dispatching the `lookup` type to
/// `TAG_GetOrigin`/`TAG_GetAngles` on the entity's `ownername`. (Active body in the PC source;
/// the Xbox build had it `assert(0); return 0;`.)
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn Q3_GetTag(entID: c_int, name: *const c_char, lookup: c_int, info: &mut vec3_t) -> c_int {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);

    if (*ent).inuse == QFALSE {
        debug_assert!(false); // assert(0);
        return 0;
    }

    match lookup {
        TYPE_ORIGIN => return TAG_GetOrigin((*ent).ownername, name, info),

        TYPE_ANGLES => return TAG_GetAngles((*ent).ownername, name, info),

        _ => {}
    }

    0
}

/// `int Q3_GetFloat( int entID, int type, const char *name, float *value )`
/// (g_ICARUScb.c:1192) — reads a float-valued ICARUS register/field off an entity (resolved by
/// name through `setTable`). Returns 1 if the value was obtained, 0 otherwise; the `default`
/// arm falls through to the declared ICARUS float variables. (`//FIXME: May want to make a
/// "getTable" as well` — kept verbatim.)
///
/// # Safety
/// `g_entities` must be initialised; `name`/`value` must be valid.
pub unsafe fn Q3_GetFloat(entID: c_int, _type: c_int, name: *const c_char, value: *mut f32) -> c_int {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let toGet: c_int;

    if ent.is_null() {
        return 0;
    }

    toGet = GetIDForString(addr_of!(setTable) as *const stringID_table_t, name); //FIXME: May want to make a "getTable" as well
    //FIXME: I'm getting really sick of these huge switch statements!

    //NOTENOTE: return true if the value was correctly obtained
    match toGet {
        SET_PARM1 | SET_PARM2 | SET_PARM3 | SET_PARM4 | SET_PARM5 | SET_PARM6 | SET_PARM7
        | SET_PARM8 | SET_PARM9 | SET_PARM10 | SET_PARM11 | SET_PARM12 | SET_PARM13
        | SET_PARM14 | SET_PARM15 | SET_PARM16 => {
            if (*ent).parms.is_null() {
                G_DebugPrint(
                    WL_ERROR,
                    &format!(
                        "GET_PARM: {} {} did not have any parms set!\n",
                        cstr_or_null((*ent).classname),
                        cstr_or_null((*ent).targetname)
                    ),
                );
                return 0; // would prefer qfalse, but I'm fitting in with what's here <sigh>
            }
            *value = atof((*(*ent).parms).parm[(toGet - SET_PARM1) as usize].as_ptr()) as f32;
        }

        SET_COUNT => {
            *value = (*ent).count as f32;
        }

        SET_HEALTH => {
            *value = (*ent).health as f32;
        }

        SET_SKILL => {
            return 0;
        }

        SET_XVELOCITY => {
            //## %f="0.0" # Velocity along X axis
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_XVELOCITY, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.velocity[0];
        }

        SET_YVELOCITY => {
            //## %f="0.0" # Velocity along Y axis
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_YVELOCITY, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.velocity[1];
        }

        SET_ZVELOCITY => {
            //## %f="0.0" # Velocity along Z axis
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_ZVELOCITY, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.velocity[2];
        }

        SET_Z_OFFSET => {
            *value = (*ent).r.currentOrigin[2] - (*ent).s.origin[2];
        }

        SET_DPITCH => {
            //## %f="0.0" # Pitch for NPC to turn to
            return 0;
        }

        SET_DYAW => {
            //## %f="0.0" # Yaw for NPC to turn to
            return 0;
        }

        SET_WIDTH => {
            //## %f="0.0" # Width of NPC bounding box
            *value = (*ent).r.mins[0];
        }
        SET_TIMESCALE => {
            //## %f="0.0" # Speed-up slow down game (0 - 1.0)
            return 0;
        }
        SET_CAMERA_GROUP_Z_OFS => {
            //## %s="NULL" # all ents with this cameraGroup will be focused on
            return 0;
        }

        SET_VISRANGE => {
            //## %f="0.0" # How far away NPC can see
            return 0;
        }

        SET_EARSHOT => {
            //## %f="0.0" # How far an NPC can hear
            return 0;
        }

        SET_VIGILANCE => {
            //## %f="0.0" # How often to look for enemies (0 - 1.0)
            return 0;
        }

        SET_GRAVITY => {
            //## %f="0.0" # Change this ent's gravity - 800 default
            *value = (*addr_of!(g_gravity)).value;
        }

        SET_FACEEYESCLOSED | SET_FACEEYESOPENED | SET_FACEAUX | SET_FACEBLINK
        | SET_FACEBLINKFROWN | SET_FACEFROWN | SET_FACENORMAL => {
            G_DebugPrint(WL_WARNING, "Q3_GetFloat: SET_FACE___ not implemented\n");
            return 0;
        }
        SET_WAIT => {
            //## %f="0.0" # Change an entity's wait field
            *value = (*ent).wait;
        }
        SET_FOLLOWDIST => {
            //## %f="0.0" # How far away to stay from leader in BS_FOLLOW_LEADER
            return 0;
        }
        //# #sep ints
        SET_ANIM_HOLDTIME_LOWER => {
            //## %d="0" # Hold lower anim for number of milliseconds
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_ANIM_HOLDTIME_LOWER, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.legsTimer as f32;
        }
        SET_ANIM_HOLDTIME_UPPER => {
            //## %d="0" # Hold upper anim for number of milliseconds
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_ANIM_HOLDTIME_UPPER, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.torsoTimer as f32;
        }
        SET_ANIM_HOLDTIME_BOTH => {
            //## %d="0" # Hold lower and upper anims for number of milliseconds
            G_DebugPrint(WL_WARNING, "Q3_GetFloat: SET_ANIM_HOLDTIME_BOTH not implemented\n");
            return 0;
        }
        SET_ARMOR => {
            //## %d="0" # Change armor
            if (*ent).client.is_null() {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetFloat: SET_ARMOR, {} not a client\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
            *value = (*(*ent).client).ps.stats[STAT_ARMOR as usize] as f32;
        }
        SET_WALKSPEED => return 0,  //## %d="0" # Change walkSpeed
        SET_RUNSPEED => return 0,   //## %d="0" # Change runSpeed
        SET_YAWSPEED => return 0,   //## %d="0" # Change yawSpeed
        SET_AGGRESSION => return 0, //## %d="0" # Change aggression 1-5
        SET_AIM => return 0,        //## %d="0" # Change aim 1-5
        SET_FRICTION => return 0,   //## %d="0" # Change ent's friction - 6 default
        SET_SHOOTDIST => return 0,  //## %d="0" # How far the ent can shoot - 0 uses weapon
        SET_HFOV => return 0,       //## %d="0" # Horizontal field of view
        SET_VFOV => return 0,       //## %d="0" # Vertical field of view
        SET_DELAYSCRIPTTIME => return 0, //## %d="0" # How many seconds to wait before running delayscript
        SET_FORWARDMOVE => return 0, //## %d="0" # NPC move forward -127(back) to 127
        SET_RIGHTMOVE => return 0,  //## %d="0" # NPC move right -127(left) to 127
        SET_STARTFRAME => return 0, //## %d="0" # frame to start animation sequence on
        SET_ENDFRAME => return 0,   //## %d="0" # frame to end animation sequence on
        SET_ANIMFRAME => return 0,  //## %d="0" # of current frame

        SET_SHOT_SPACING => return 0, //## %d="1000" # Time between shots for an NPC - reset to defaults when changes weapon
        SET_MISSIONSTATUSTIME => return 0, //## %d="0" # Amount of time until Mission Status should be shown after death
        //# #sep booleans
        SET_IGNOREPAIN => return 0,    //## %t="BOOL_TYPES" # Do not react to pain
        SET_IGNOREENEMIES => return 0, //## %t="BOOL_TYPES" # Do not acquire enemies
        SET_IGNOREALERTS => return 0,  //## Do not get enemy set by allies in area(ambush)
        SET_DONTSHOOT => return 0,     //## %t="BOOL_TYPES" # Others won't shoot you
        SET_NOTARGET => {
            //## %t="BOOL_TYPES" # Others won't pick you as enemy
            *value = ((*ent).flags & FL_NOTARGET) as f32;
        }
        SET_DONTFIRE => return 0, //## %t="BOOL_TYPES" # Don't fire your weapon

        SET_LOCKED_ENEMY => return 0,     //## %t="BOOL_TYPES" # Keep current enemy until dead
        SET_CROUCHED => return 0,         //## %t="BOOL_TYPES" # Force NPC to crouch
        SET_WALKING => return 0,          //## %t="BOOL_TYPES" # Force NPC to move at walkSpeed
        SET_RUNNING => return 0,          //## %t="BOOL_TYPES" # Force NPC to move at runSpeed
        SET_CHASE_ENEMIES => return 0,    //## %t="BOOL_TYPES" # NPC will chase after enemies
        SET_LOOK_FOR_ENEMIES => return 0, //## %t="BOOL_TYPES" # NPC will be on the lookout for enemies
        SET_FACE_MOVE_DIR => return 0,    //## %t="BOOL_TYPES" # NPC will face in the direction it's moving
        SET_FORCED_MARCH => return 0,     //## %t="BOOL_TYPES" # Force NPC to move at runSpeed
        SET_UNDYING => return 0,          //## %t="BOOL_TYPES" # Can take damage down to 1 but not die
        SET_NOAVOID => return 0,          //## %t="BOOL_TYPES" # Will not avoid other NPCs or architecture

        SET_SOLID => {
            //## %t="BOOL_TYPES" # Make yourself notsolid or solid
            *value = (*ent).r.contents as f32;
        }
        SET_PLAYER_USABLE => {
            //## %t="BOOL_TYPES" # Can be activateby the player's "use" button
            *value = ((*ent).r.svFlags & SVF_PLAYER_USABLE) as f32;
        }
        SET_LOOP_ANIM => return 0, //## %t="BOOL_TYPES" # For non-NPCs: loop your animation sequence
        SET_INTERFACE => {
            //## %t="BOOL_TYPES" # Player interface on/off
            G_DebugPrint(WL_WARNING, "Q3_GetFloat: SET_INTERFACE not implemented\n");
            return 0;
        }
        SET_SHIELDS => return 0, //## %t="BOOL_TYPES" # NPC has no shields (Borg do not adapt)
        SET_INVISIBLE => {
            //## %t="BOOL_TYPES" # Makes an NPC not solid and not visible
            *value = ((*ent).s.eFlags & EF_NODRAW) as f32;
        }
        SET_VAMPIRE => return 0,          //## %t="BOOL_TYPES" # Makes an NPC not solid and not visible
        SET_FORCE_INVINCIBLE => return 0, //## %t="BOOL_TYPES" # Makes an NPC not solid and not visible
        SET_GREET_ALLIES => return 0,     //## %t="BOOL_TYPES" # Makes an NPC greet teammates
        SET_VIDEO_FADE_IN => {
            //## %t="BOOL_TYPES" # Makes video playback fade in
            G_DebugPrint(WL_WARNING, "Q3_GetFloat: SET_VIDEO_FADE_IN not implemented\n");
            return 0;
        }
        SET_VIDEO_FADE_OUT => {
            //## %t="BOOL_TYPES" # Makes video playback fade out
            G_DebugPrint(WL_WARNING, "Q3_GetFloat: SET_VIDEO_FADE_OUT not implemented\n");
            return 0;
        }
        SET_PLAYER_LOCKED => return 0,       //## %t="BOOL_TYPES" # Makes it so player cannot move
        SET_LOCK_PLAYER_WEAPONS => return 0, //## %t="BOOL_TYPES" # Makes it so player cannot switch weapons
        SET_NO_IMPACT_DAMAGE => return 0,    //## %t="BOOL_TYPES" # Makes it so player cannot switch weapons
        SET_NO_KNOCKBACK => {
            //## %t="BOOL_TYPES" # Stops this ent from taking knockback from weapons
            *value = ((*ent).flags & FL_NO_KNOCKBACK) as f32;
        }
        SET_ALT_FIRE => return 0,    //## %t="BOOL_TYPES" # Force NPC to use altfire when shooting
        SET_NO_RESPONSE => return 0, //## %t="BOOL_TYPES" # NPCs will do generic responses when this is on (usescripts override generic responses as well)
        SET_INVINCIBLE => {
            //## %t="BOOL_TYPES" # Completely unkillable
            *value = ((*ent).flags & FL_GODMODE) as f32;
        }
        SET_MISSIONSTATUSACTIVE => return 0, //# Turns on Mission Status Screen
        SET_NO_COMBAT_TALK => return 0,  //## %t="BOOL_TYPES" # NPCs will not do their combat talking noises when this is on
        SET_NO_ALERT_TALK => return 0,   //## %t="BOOL_TYPES" # NPCs will not do their combat talking noises when this is on
        SET_USE_CP_NEAREST => return 0,  //## %t="BOOL_TYPES" # NPCs will use their closest combat points, not try and find ones next to the player, or flank player
        SET_DISMEMBERABLE => return 0,   //## %t="BOOL_TYPES" # NPC will not be affected by force powers
        SET_NO_FORCE => return 0,
        SET_NO_ACROBATICS => return 0,
        SET_USE_SUBTITLES => return 0,
        SET_NO_FALLTODEATH => return 0,  //## %t="BOOL_TYPES" # NPC will not be affected by force powers
        SET_MORELIGHT => return 0,       //## %t="BOOL_TYPES" # NPCs will use their closest combat points, not try and find ones next to the player, or flank player
        SET_TREASONED => return 0,       //## %t="BOOL_TYPES" # Player has turned on his own- scripts will stop: NPCs will turn on him and level changes load the brig
        SET_DISABLE_SHADER_ANIM => return 0, //## %t="BOOL_TYPES" # Shaders won't animate
        SET_SHADER_ANIM => return 0,     //## %t="BOOL_TYPES" # Shader will be under frame control

        _ => {
            if trap::ICARUS_VariableDeclared(name) != VTYPE_FLOAT {
                return 0;
            }

            return trap::ICARUS_GetFloatVariable(name, value);
        }
    }

    1
}

/// `int Q3_GetVector( int entID, int type, const char *name, vec3_t value )`
/// (g_ICARUScb.c:1576) — reads a vector-valued ICARUS register/field off an entity. Returns 1
/// if obtained, else 0; the `default` arm falls through to the declared ICARUS vector
/// variables.
///
/// # Safety
/// `g_entities` must be initialised; `name`/`value` must be valid.
pub unsafe fn Q3_GetVector(entID: c_int, _type: c_int, name: *const c_char, value: &mut vec3_t) -> c_int {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let toGet: c_int;
    if ent.is_null() {
        return 0;
    }

    toGet = GetIDForString(addr_of!(setTable) as *const stringID_table_t, name); //FIXME: May want to make a "getTable" as well
    //FIXME: I'm getting really sick of these huge switch statements!

    //NOTENOTE: return true if the value was correctly obtained
    match toGet {
        SET_PARM1 | SET_PARM2 | SET_PARM3 | SET_PARM4 | SET_PARM5 | SET_PARM6 | SET_PARM7
        | SET_PARM8 | SET_PARM9 | SET_PARM10 | SET_PARM11 | SET_PARM12 | SET_PARM13
        | SET_PARM14 | SET_PARM15 | SET_PARM16 => {
            // sscanf( ent->parms->parm[toGet - SET_PARM1], "%f %f %f", &value[0], &value[1], &value[2] );
            let parm = &(*(*ent).parms).parm[(toGet - SET_PARM1) as usize];
            let txt = CStr::from_ptr(parm.as_ptr()).to_string_lossy();
            let mut it = txt
                .split(|c: char| c.is_ascii_whitespace())
                .filter(|s| !s.is_empty());
            // %f sscanf: parse leading floats; missing fields leave value[] unchanged.
            for slot in value.iter_mut() {
                if let Some(tok) = it.next() {
                    if let Ok(f) = parse_leading_f32(tok) {
                        *slot = f;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        SET_ORIGIN => {
            VectorCopy(&(*ent).r.currentOrigin, value);
        }

        SET_ANGLES => {
            VectorCopy(&(*ent).r.currentAngles, value);
        }

        SET_TELEPORT_DEST => {
            //## %v="0.0 0.0 0.0" # Set origin here as soon as the area is clear
            G_DebugPrint(WL_WARNING, "Q3_GetVector: SET_TELEPORT_DEST not implemented\n");
            return 0;
        }

        _ => {
            if trap::ICARUS_VariableDeclared(name) != VTYPE_VECTOR {
                return 0;
            }

            return trap::ICARUS_GetVectorVariable(name, value);
        }
    }

    1
}

// Parse the leading float of a token the way C `sscanf("%f")` does (stops at the first
// non-float character). Used by Q3_GetVector's SET_PARM* vector parse.
fn parse_leading_f32(tok: &str) -> Result<f32, ()> {
    let t = tok.trim_start();
    let bytes = t.as_bytes();
    let mut seen_dot = false;
    let mut seen_e = false;
    let mut i = 0usize;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_digit() {
            i += 1;
        } else if c == b'.' && !seen_dot && !seen_e {
            seen_dot = true;
            i += 1;
        } else if (c == b'e' || c == b'E') && !seen_e && i > 0 {
            seen_e = true;
            i += 1;
            if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                i += 1;
            }
        } else {
            break;
        }
    }
    t[..i].parse::<f32>().map_err(|_| ())
}

/// `int Q3_GetString( int entID, int type, const char *name, char **value )`
/// (g_ICARUScb.c:1645) — reads a string-valued ICARUS register/field off an entity (anim name,
/// parm, behaviorSet script path, targetname/target/fullName). Returns 1 if obtained, else 0;
/// the `default` arm falls through to the declared ICARUS string variables.
///
/// # Safety
/// `g_entities` must be initialised; `name`/`value` must be valid.
pub unsafe fn Q3_GetString(entID: c_int, _type: c_int, name: *const c_char, value: *mut *mut c_char) -> c_int {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let toGet: c_int;
    if ent.is_null() {
        return 0;
    }

    toGet = GetIDForString(addr_of!(setTable) as *const stringID_table_t, name); //FIXME: May want to make a "getTable" as well

    match toGet {
        SET_ANIM_BOTH => {
            *value = Q3_GetAnimBoth(ent);

            // C: if ( !value || !value[0] ) — `value` (the char**) is never NULL, and `value[0]`
            // re-reads the just-written string pointer; bail if it is NULL.
            if value.is_null() || (*value).is_null() {
                return 0;
            }
        }

        SET_PARM1 | SET_PARM2 | SET_PARM3 | SET_PARM4 | SET_PARM5 | SET_PARM6 | SET_PARM7
        | SET_PARM8 | SET_PARM9 | SET_PARM10 | SET_PARM11 | SET_PARM12 | SET_PARM13
        | SET_PARM14 | SET_PARM15 | SET_PARM16 => {
            if !(*ent).parms.is_null() {
                *value = (*(*ent).parms).parm[(toGet - SET_PARM1) as usize].as_ptr() as *mut c_char;
            } else {
                G_DebugPrint(
                    WL_WARNING,
                    &format!("Q3_GetString: invalid ent {} has no parms!\n", cstr_or_null((*ent).targetname)),
                );
                return 0;
            }
        }

        SET_TARGET => {
            *value = (*ent).target;
        }

        SET_LOCATION => {
            return 0;
        }

        //# #sep Scripts and other file paths
        SET_SPAWNSCRIPT => {
            //## %s="NULL" # Script to run when spawned //0 - do not change these, these are equal to BSET_SPAWN, etc
            *value = (*ent).behaviorSet[BSET_SPAWN as usize];
        }
        SET_USESCRIPT => {
            //## %s="NULL" # Script to run when used
            *value = (*ent).behaviorSet[BSET_USE as usize];
        }
        SET_AWAKESCRIPT => {
            //## %s="NULL" # Script to run when startled
            *value = (*ent).behaviorSet[BSET_AWAKE as usize];
        }
        SET_ANGERSCRIPT => {
            //## %s="NULL" # Script run when find an enemy for the first time
            *value = (*ent).behaviorSet[BSET_ANGER as usize];
        }
        SET_ATTACKSCRIPT => {
            //## %s="NULL" # Script to run when you shoot
            *value = (*ent).behaviorSet[BSET_ATTACK as usize];
        }
        SET_VICTORYSCRIPT => {
            //## %s="NULL" # Script to run when killed someone
            *value = (*ent).behaviorSet[BSET_VICTORY as usize];
        }
        SET_LOSTENEMYSCRIPT => {
            //## %s="NULL" # Script to run when you can't find your enemy
            *value = (*ent).behaviorSet[BSET_LOSTENEMY as usize];
        }
        SET_PAINSCRIPT => {
            //## %s="NULL" # Script to run when hit
            *value = (*ent).behaviorSet[BSET_PAIN as usize];
        }
        SET_FLEESCRIPT => {
            //## %s="NULL" # Script to run when hit and low health
            *value = (*ent).behaviorSet[BSET_FLEE as usize];
        }
        SET_DEATHSCRIPT => {
            //## %s="NULL" # Script to run when killed
            *value = (*ent).behaviorSet[BSET_DEATH as usize];
        }
        SET_DELAYEDSCRIPT => {
            //## %s="NULL" # Script to run after a delay
            *value = (*ent).behaviorSet[BSET_DELAYED as usize];
        }
        SET_BLOCKEDSCRIPT => {
            //## %s="NULL" # Script to run when blocked by teammate
            *value = (*ent).behaviorSet[BSET_BLOCKED as usize];
        }
        SET_FFIRESCRIPT => {
            //## %s="NULL" # Script to run when player has shot own team repeatedly
            *value = (*ent).behaviorSet[BSET_FFIRE as usize];
        }
        SET_FFDEATHSCRIPT => {
            //## %s="NULL" # Script to run when player kills a teammate
            *value = (*ent).behaviorSet[BSET_FFDEATH as usize];
        }

        //# #sep Standard strings
        SET_ENEMY => return 0,  //## %s="NULL" # Set enemy by targetname
        SET_LEADER => return 0, //## %s="NULL" # Set for BS_FOLLOW_LEADER
        SET_CAPTURE => return 0, //## %s="NULL" # Set captureGoal by targetname

        SET_TARGETNAME => {
            //## %s="NULL" # Set/change your targetname
            *value = (*ent).targetname;
        }
        SET_PAINTARGET => return 0,       //## %s="NULL" # Set/change what to use when hit
        SET_CAMERA_GROUP => return 0,     //## %s="NULL" # all ents with this cameraGroup will be focused on
        SET_CAMERA_GROUP_TAG => return 0, //## %s="NULL" # all ents with this cameraGroup will be focused on
        SET_LOOK_TARGET => {
            //## %s="NULL" # object for NPC to look at
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_LOOK_TARGET, NOT SUPPORTED IN MULTIPLAYER\n");
        }
        SET_TARGET2 => return 0, //## %s="NULL" # Set/change your target2: on NPC's: this fires when they're knocked out by the red hypo

        SET_REMOVE_TARGET => return 0, //## %s="NULL" # Target that is fired when someone completes the BS_REMOVE behaviorState
        SET_WEAPON => return 0,

        SET_ITEM => return 0,
        SET_MUSIC_STATE => return 0,
        //The below cannot be gotten
        SET_NAVGOAL => {
            //## %s="NULL" # *Move to this navgoal then continue script
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_NAVGOAL not implemented\n");
            return 0;
        }
        SET_VIEWTARGET => {
            //## %s="NULL" # Set angles toward ent by targetname
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_VIEWTARGET not implemented\n");
            return 0;
        }
        SET_WATCHTARGET => return 0, //## %s="NULL" # Set angles toward ent by targetname
        SET_VIEWENTITY => {
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_VIEWENTITY not implemented\n");
            return 0;
        }
        SET_CAPTIONTEXTCOLOR => {
            //## %s=""  # Color of text RED:WHITE:BLUE: YELLOW
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_CAPTIONTEXTCOLOR not implemented\n");
            return 0;
        }
        SET_CENTERTEXTCOLOR => {
            //## %s=""  # Color of text RED:WHITE:BLUE: YELLOW
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_CENTERTEXTCOLOR not implemented\n");
            return 0;
        }
        SET_SCROLLTEXTCOLOR => {
            //## %s=""  # Color of text RED:WHITE:BLUE: YELLOW
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_SCROLLTEXTCOLOR not implemented\n");
            return 0;
        }
        SET_COPY_ORIGIN => {
            //## %s="targetname"  # Copy the origin of the ent with targetname to your origin
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_COPY_ORIGIN not implemented\n");
            return 0;
        }
        SET_DEFEND_TARGET => {
            //## %s="targetname"  # This NPC will attack the target NPC's enemies
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_COPY_ORIGIN not implemented\n");
            return 0;
        }
        SET_VIDEO_PLAY => {
            //## %s="filename" # Play a Video (inGame)
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_VIDEO_PLAY not implemented\n");
            return 0;
        }
        SET_LOADGAME => {
            //## %s="exitholodeck" # Load the savegame that was auto-saved when you started the holodeck
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_LOADGAME not implemented\n");
            return 0;
        }
        SET_LOCKYAW => {
            //## %s="off"  # Lock legs to a certain yaw angle (or "off" or "auto" uses current)
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_LOCKYAW not implemented\n");
            return 0;
        }
        SET_SCROLLTEXT => {
            //## %s="" # key of text string to print
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_SCROLLTEXT not implemented\n");
            return 0;
        }
        SET_LCARSTEXT => {
            //## %s="" # key of text string to print in LCARS frame
            G_DebugPrint(WL_WARNING, "Q3_GetString: SET_LCARSTEXT not implemented\n");
            return 0;
        }

        SET_FULLNAME => {
            //## %s="NULL" # Set/change your targetname
            *value = (*ent).fullName;
        }
        _ => {
            if trap::ICARUS_VariableDeclared(name) != VTYPE_STRING {
                return 0;
            }

            return trap::ICARUS_GetStringVariable(name, *value);
        }
    }

    1
}

/// `static qboolean Q3_SetBState( int entID, const char *bs_name )` (g_ICARUScb.c:2397) —
/// changes an NPC's behavior state. Resolves `bs_name` to a `bState_t` through `BSTable`;
/// BS_SEARCH/BS_WANDER kick off a waypoint search, BS_NOCLIP toggles `client->noclip`, and
/// BS_ADVANCE_FIGHT/BS_JUMP return `qfalse` (wait for task-complete) or set jumpState. Returns
/// `qtrue` (ok to complete) except those wait cases.
///
/// # Safety
/// `g_entities` must be initialised; `ent->NPC`/`ent->client` are dereferenced when set.
pub unsafe fn Q3_SetBState(entID: c_int, bs_name: *const c_char) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let bSID: bState_t;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetBState: invalid entID {entID}\n"));
        return QTRUE;
    }

    if (*ent).NPC.is_null() {
        G_DebugPrint(WL_ERROR, &format!("Q3_SetBState: '{}' is not an NPC\n", cstr_or_null((*ent).targetname)));
        return QTRUE; //ok to complete
    }

    bSID = GetIDForString(addr_of!(BSTable) as *const stringID_table_t, bs_name);
    if bSID > -1 {
        if bSID == BS_SEARCH || bSID == BS_WANDER {
            //FIXME: Reimplement

            if (*ent).waypoint != WAYPOINT_NONE {
                NPC_BSSearchStart((*ent).waypoint, bSID);
            } else {
                (*ent).waypoint = NAV_FindClosestWaypointForEnt(ent, WAYPOINT_NONE);

                if (*ent).waypoint != WAYPOINT_NONE {
                    NPC_BSSearchStart((*ent).waypoint, bSID);
                }
                /*else if( ent->lastWaypoint >=0 && ent->lastWaypoint < num_waypoints )
                {
                    NPC_BSSearchStart( ent->lastWaypoint, bSID );
                }
                else if( ent->lastValidWaypoint >=0 && ent->lastValidWaypoint < num_waypoints )
                {
                    NPC_BSSearchStart( ent->lastValidWaypoint, bSID );
                }*/
                else {
                    G_DebugPrint(
                        WL_ERROR,
                        &format!("Q3_SetBState: '{}' is not in a valid waypoint to search from!\n", cstr_or_null((*ent).targetname)),
                    );
                    return QTRUE;
                }
            }
        }

        (*(*ent).NPC).tempBehavior = BS_DEFAULT; //need to clear any temp behaviour
        if (*(*ent).NPC).behaviorState == BS_NOCLIP && bSID != BS_NOCLIP {
            //need to rise up out of the floor after noclipping
            (*ent).r.currentOrigin[2] += 0.125;
            G_SetOrigin(ent, &(*ent).r.currentOrigin);
        }
        (*(*ent).NPC).behaviorState = bSID;
        if bSID == BS_DEFAULT {
            (*(*ent).NPC).defaultBehavior = bSID;
        }
    }

    (*(*ent).NPC).aiFlags &= !NPCAI_TOUCHED_GOAL;

    //	if ( bSID == BS_FLY )
    //	{//FIXME: need a set bState wrapper
    //		ent->client->moveType = MT_FLYSWIM;
    //	}
    //	else
    {
        //FIXME: these are presumptions!
        //Q3_SetGravity( entID, g_gravity->value );
        //ent->client->moveType = MT_RUNJUMP;
    }

    if bSID == BS_NOCLIP {
        (*(*ent).client).noclip = QTRUE;
    } else {
        (*(*ent).client).noclip = QFALSE;
    }

    /*
        if ( bSID == BS_FACE || bSID == BS_POINT_AND_SHOOT || bSID == BS_FACE_ENEMY )
        {
            ent->NPC->aimTime = level.time + 5 * 1000;//try for 5 seconds
            return qfalse;//need to wait for task complete message
        }
    */

    //	if ( bSID == BS_SNIPER || bSID == BS_ADVANCE_FIGHT )
    if bSID == BS_ADVANCE_FIGHT {
        return QFALSE; //need to wait for task complete message
    }

    /*
        if ( bSID == BS_SHOOT || bSID == BS_POINT_AND_SHOOT )
        {//Let them shoot right NOW
            ent->NPC->shotTime = ent->attackDebounceTime = level.time;
        }
    */
    if bSID == BS_JUMP {
        (*(*ent).NPC).jumpState = JS_FACING;
    }

    QTRUE //ok to complete
}

/// `static qboolean Q3_SetTempBState( int entID, const char *bs_name )` (g_ICARUScb.c:2523) —
/// sets an NPC's *temporary* behavior state (overrides the normal one while valid). Returns
/// `qtrue` (ok to complete).
///
/// # Safety
/// `g_entities` must be initialised; `ent->NPC` is dereferenced when set.
pub unsafe fn Q3_SetTempBState(entID: c_int, bs_name: *const c_char) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let bSID: bState_t;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetTempBState: invalid entID {entID}\n"));
        return QTRUE;
    }

    if (*ent).NPC.is_null() {
        G_DebugPrint(WL_ERROR, &format!("Q3_SetTempBState: '{}' is not an NPC\n", cstr_or_null((*ent).targetname)));
        return QTRUE; //ok to complete
    }

    bSID = GetIDForString(addr_of!(BSTable) as *const stringID_table_t, bs_name);
    if bSID > -1 {
        (*(*ent).NPC).tempBehavior = bSID;
    }

    /*
        if ( bSID == BS_FACE || bSID == BS_POINT_AND_SHOOT || bSID == BS_FACE_ENEMY )
        {
            ent->NPC->aimTime = level.time + 5 * 1000;//try for 5 seconds
            return qfalse;//need to wait for task complete message
        }
    */

    /*
        if ( bSID == BS_SHOOT || bSID == BS_POINT_AND_SHOOT )
        {//Let them shoot right NOW
            ent->NPC->shotTime = ent->attackDebounceTime = level.time;
        }
    */
    QTRUE //ok to complete
}

/// `static void Q3_SetDefaultBState( int entID, const char *bs_name )` (g_ICARUScb.c:2573) —
/// sets the NPC's fallback behavior state (used when no other is set).
///
/// # Safety
/// `g_entities` must be initialised; `ent->NPC` is dereferenced when set.
pub unsafe fn Q3_SetDefaultBState(entID: c_int, bs_name: *const c_char) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let bSID: bState_t;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetDefaultBState: invalid entID {entID}\n"));
        return;
    }

    if (*ent).NPC.is_null() {
        G_DebugPrint(WL_ERROR, &format!("Q3_SetDefaultBState: '{}' is not an NPC\n", cstr_or_null((*ent).targetname)));
        return;
    }

    bSID = GetIDForString(addr_of!(BSTable) as *const stringID_table_t, bs_name);
    if bSID > -1 {
        (*(*ent).NPC).defaultBehavior = bSID;
    }
}

/// `static qboolean Q3_SetBehaviorSet( int entID, int toSet, const char *scriptname)`
/// (g_ICARUScb.c:4345) — assigns one of the entity's `behaviorSet[]` script slots from a
/// `SET_*SCRIPT` id. `"NULL"` clears the slot; otherwise the name is interned via `G_NewString`
/// (the two `gi.TagFree` frees of the old string are commented out in the C). Returns `qtrue`
/// on a valid slot, `qfalse` otherwise.
///
/// # Safety
/// `g_entities` must be initialised; `scriptname` must be a valid C string.
pub unsafe fn Q3_SetBehaviorSet(entID: c_int, toSet: c_int, scriptname: *const c_char) -> qboolean {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entID as usize);
    let mut bSet: bSet_t = BSET_INVALID;

    if ent.is_null() {
        G_DebugPrint(WL_WARNING, &format!("Q3_SetBehaviorSet: invalid entID {entID}\n"));
        return QFALSE;
    }

    match toSet {
        SET_SPAWNSCRIPT => bSet = BSET_SPAWN,
        SET_USESCRIPT => bSet = BSET_USE,
        SET_AWAKESCRIPT => bSet = BSET_AWAKE,
        SET_ANGERSCRIPT => bSet = BSET_ANGER,
        SET_ATTACKSCRIPT => bSet = BSET_ATTACK,
        SET_VICTORYSCRIPT => bSet = BSET_VICTORY,
        SET_LOSTENEMYSCRIPT => bSet = BSET_LOSTENEMY,
        SET_PAINSCRIPT => bSet = BSET_PAIN,
        SET_FLEESCRIPT => bSet = BSET_FLEE,
        SET_DEATHSCRIPT => bSet = BSET_DEATH,
        SET_DELAYEDSCRIPT => bSet = BSET_DELAYED,
        SET_BLOCKEDSCRIPT => bSet = BSET_BLOCKED,
        SET_FFIRESCRIPT => bSet = BSET_FFIRE,
        SET_FFDEATHSCRIPT => bSet = BSET_FFDEATH,
        SET_MINDTRICKSCRIPT => bSet = BSET_MINDTRICK,
        _ => {}
    }

    if bSet < BSET_SPAWN || bSet >= NUM_BSETS as bSet_t {
        return QFALSE;
    }

    if Q_stricmp(c"NULL".as_ptr(), scriptname) == 0 {
        if !(*ent).behaviorSet[bSet as usize].is_null() {
            //			gi.TagFree( ent->behaviorSet[bSet] );
        }

        (*ent).behaviorSet[bSet as usize] = null_mut();
        //memset( &ent->behaviorSet[bSet], 0, sizeof(ent->behaviorSet[bSet]) );
    } else {
        if !scriptname.is_null() {
            if !(*ent).behaviorSet[bSet as usize].is_null() {
                //				gi.TagFree( ent->behaviorSet[bSet] );
            }

            (*ent).behaviorSet[bSet as usize] = G_NewString(scriptname); //FIXME: This really isn't good...
        }

        //ent->behaviorSet[bSet] = scriptname;
        //strncpy( (char *) &ent->behaviorSet[bSet], scriptname, MAX_BSET_LENGTH );
    }
    QTRUE
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::Q3_TaskIDClear;
    use crate::oracle;
    use core::ffi::c_int;

    /// `Q3_TaskIDClear` must drive its out-param to -1, matching the C oracle.
    #[test]
    fn Q3_TaskIDClear_matches_oracle() {
        for seed in [-5, -1, 0, 1, 42, i32::MAX, i32::MIN] {
            let mut rust: c_int = seed;
            let mut c: c_int = seed;
            unsafe {
                Q3_TaskIDClear(&mut rust);
                oracle::jka_Q3_TaskIDClear(&mut c);
            }
            assert_eq!(rust, c, "seed {seed}");
            assert_eq!(rust, -1, "seed {seed}");
        }
    }
}
