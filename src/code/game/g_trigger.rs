// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "g_functions.h"
// #include "b_local.h"
// #include "anims.h"

use core::ffi::{c_int, c_char};

// Forward declarations of external types and functions
extern "C" {
    fn G_PointInBounds(point: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3]) -> core::ffi::c_int;
    fn G_ClearTrace(start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], ignore: c_int, clipmask: c_int) -> core::ffi::c_int;
    fn SpotWouldTelefrag2(mover: *mut gentity_t, dest: *const [f32; 3]) -> core::ffi::c_int;
    fn PM_CrouchAnim(anim: c_int) -> core::ffi::c_int;
    fn Boba_FlyStart(self_: *mut gentity_t);
    fn Boba_Flying(self_: *mut gentity_t) -> core::ffi::c_int;
    fn Pilot_ActivePilotCount() -> c_int;
    fn G_SetMovedir(angles: *const [f32; 3], movedir: *mut [f32; 3]);
    fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t);
    fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, target: *const c_char);
    fn G_Sound(ent: *mut gentity_t, index: c_int);
    fn G_Find(from: *mut gentity_t, fieldofs: usize, match_: *const c_char) -> *mut gentity_t;
    fn G_PickTarget(targetname: *const c_char) -> *mut gentity_t;
    fn G_FreeEntity(ent: *mut gentity_t);
    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_SpawnString(key: *const c_char, default_value: *const c_char, out: *mut *const c_char) -> c_int;
    fn G_SpawnFloat(key: *const c_char, default_value: *const c_char, out: *mut f32) -> c_int;
    fn G_SpawnInt(key: *const c_char, default_value: *const c_char, out: *mut c_int) -> c_int;
    fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const [f32; 3], point: *const [f32; 3], damage: c_int, dflags: c_int, mod_: c_int);
    fn G_Error(fmt: *const c_char, ...);
    fn G_SetOrigin(ent: *mut gentity_t, origin: *const [f32; 3]);
    fn TeleportMover(mover: *mut gentity_t, origin: *const [f32; 3], diffAngles: *const [f32; 3], snapAngle: c_int);
    fn TeleportPlayer(ent: *mut gentity_t, origin: *const [f32; 3], angles: *const [f32; 3]);
    fn JET_FlyStart(actor: *mut gentity_t);
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> c_int;
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorAdd(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorLength(v: *const [f32; 3]) -> f32;
    fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
    fn DotProduct(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strncpyz(dst: *mut c_char, src: *const c_char, size: usize);
    fn COM_Parse(data_p: *mut *const c_char) -> *const c_char;
    fn COM_DefaultExtension(path: *mut c_char, maxsize: usize, extension: *const c_char);
    fn vtos(v: *const [f32; 3]) -> *const c_char;
    fn GetIDForString(table: *const (), str: *const c_char) -> c_int;
    fn NPC_SetAnim(ent: *mut gentity_t, setanim_type: c_int, anim: c_int, flags: c_int);
    fn crandom() -> f32;
    fn sqrt(x: f32) -> f32;
    fn floor(x: f32) -> f32;

    // Global objects
    static mut gi: gameImport_t;
    static mut level: level_locals_t;
    static mut g_entities: [gentity_t; 2048];  // MAX_GENTITIES
    static mut cg: cg_t;
    static mut g_gravity: *mut cvar_t;
    static TeamTable: ();  // Opaque reference to team table
}

// Placeholder types for external dependencies (stubs for structural coherence)
#[repr(C)]
pub struct gentity_t {
    // This is a placeholder; the full definition is in the oracle
    pub s: entityState_t,
    pub ownername: *const c_char,
    pub model: *const c_char,
    pub classname: *const c_char,
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub s_origin: [f32; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub absmin: [f32; 3],
    pub absmax: [f32; 3],
    pub movedir: [f32; 3],
    pub activator: *mut gentity_t,
    pub lastEnemy: *mut gentity_t,
    pub target: *const c_char,
    pub target2: *const c_char,
    pub targetname: *const c_char,
    pub message: *const c_char,
    pub NPC_targetname: *const c_char,
    pub NPC_target: *const c_char,
    pub script_targetname: *const c_char,
    pub team: *const c_char,
    pub soundSet: *const c_char,
    pub svFlags: c_int,
    pub contents: c_int,
    pub spawnflags: c_int,
    pub nextthink: c_int,
    pub e_ThinkFunc: c_int,
    pub e_TouchFunc: c_int,
    pub e_UseFunc: c_int,
    pub painDebounceTime: c_int,
    pub aimDebounceTime: c_int,
    pub attackDebounceTime: c_int,
    pub wait: f32,
    pub delay: f32,
    pub random: f32,
    pub speed: f32,
    pub radius: f32,
    pub damage: c_int,
    pub count: c_int,
    pub lastInAirTime: c_int,
    pub noDamageTeam: c_int,
    pub noise_index: c_int,
    pub linked: c_int,
    pub inuse: c_int,
    pub takedamage: c_int,
    pub health: c_int,
    pub NPC: *mut (),
    pub client: *mut gclient_t,
    pub m_pVehicle: *mut (),
    pub inSpaceIndex: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub origin: [f32; 3],
    pub origin2: [f32; 3],
    pub angles: [f32; 3],
    pub pos: trBase_t,
    pub trType: c_int,
    pub trDelta: [f32; 3],
    pub groundEntityNum: c_int,
    pub powerups: c_int,
    pub m_iVehicleNum: c_int,
}

#[repr(C)]
pub struct trBase_t {
    pub trType: c_int,
    pub trBase: [f32; 3],
    pub trDelta: [f32; 3],
    pub trTime: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub playerTeam: c_int,
    pub usercmd: usercmd_t,
    pub renderInfo: renderInfo_t,
    pub hiddenDist: f32,
    pub hiddenDir: [f32; 3],
    pub inSpaceIndex: c_int,
    pub inSpaceSuffocation: c_int,
    pub NPC_class: c_int,
    pub fly_sound_debounce_time: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub viewangles: [f32; 3],
    pub viewheight: c_int,
    pub pm_type: c_int,
    pub pm_flags: c_int,
    pub eFlags: c_int,
    pub weapon: c_int,
    pub weaponTime: c_int,
    pub torsoAnimTimer: c_int,
    pub legsAnim: c_int,
    pub forcePowersActive: c_int,
    pub powerups: [c_int; 16],
    pub vehTurnaroundIndex: c_int,
    pub vehTurnaroundTime: c_int,
}

#[repr(C)]
pub struct usercmd_t {
    pub buttons: c_int,
}

#[repr(C)]
pub struct renderInfo_t {
    pub eyePoint: [f32; 3],
    pub eyeAngles: [f32; 3],
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
}

#[repr(C)]
pub struct gameImport_t {
    pub Printf: extern "C" fn(*const c_char, ...) -> (),
    pub SetBrushModel: extern "C" fn(*mut gentity_t, *const c_char) -> (),
    pub SetConfigstring: extern "C" fn(c_int, *const c_char) -> (),
    pub linkentity: extern "C" fn(*mut gentity_t) -> (),
    pub unlinkentity: extern "C" fn(*mut gentity_t) -> (),
    pub trace: extern "C" fn(*mut trace_t, *const [f32; 3], *const [f32; 3], *const [f32; 3], *const [f32; 3], c_int, c_int) -> (),
    pub EntityContact: extern "C" fn(*const [f32; 3], *const [f32; 3], *mut gentity_t) -> c_int,
    pub inPVS: extern "C" fn(*const [f32; 3], *const [f32; 3]) -> c_int,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct cg_t {
    pub overrides: cg_overrides_t,
}

#[repr(C)]
pub struct cg_overrides_t {
    pub active: c_int,
    pub thirdPersonCameraDamp: f32,
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
}

const ENTDIST_PLAYER: c_int = 1;
const ENTDIST_NPC: c_int = 2;

// The wait time has passed, so set back up for another activation
pub unsafe extern "C" fn multi_wait(ent: *mut gentity_t) {
    (*ent).nextthink = 0;
}

// the trigger was just activated
// ent->activator should be set to the activator so it can be held through a delay
// so wait for the delay time before firing
pub unsafe extern "C" fn multi_trigger_run(ent: *mut gentity_t) {
    (*ent).e_ThinkFunc = 0; // thinkF_NULL

    G_ActivateBehavior(ent, 32); // BSET_USE = 32

    if !(*ent).soundSet.is_null() && *(*ent).soundSet as u8 != 0 {
        gi.SetConfigstring(2, (*ent).soundSet); // CS_AMBIENT_SET = 2
    }

    G_UseTargets(ent, (*ent).activator);
    if (*ent).noise_index != 0 {
        G_Sound((*ent).activator, (*ent).noise_index);
    }

    if !(*ent).target2.is_null() && *(*ent).target2 as u8 != 0 && (*ent).wait >= 0.0 {
        (*ent).e_ThinkFunc = 5; // thinkF_trigger_cleared_fire = 5
        (*ent).nextthink = level.time + (*ent).speed as c_int;
    } else if (*ent).wait > 0.0 {
        if (*ent).painDebounceTime != level.time {
            // first ent to touch it this frame
            // (*ent).e_ThinkFunc = thinkF_multi_wait;
            (*ent).nextthink = level.time + (((*ent).wait + (*ent).random * crandom()) * 1000.0) as c_int;
            (*ent).painDebounceTime = level.time;
        }
    } else if (*ent).wait < 0.0 {
        // we can't just remove (self) here, because this is a touch function
        // called while looping through area links...
        (*ent).contents &= !1; // !CONTENTS_TRIGGER
        (*ent).e_TouchFunc = 0; // touchF_NULL
        (*ent).e_UseFunc = 0; // useF_NULL
        // Don't remove, Icarus may barf?
        // (*ent).nextthink = level.time + FRAMETIME;
        // (*ent).think = G_FreeEntity;
    }
    if !(*ent).activator.is_null() && (*(*ent).activator).s.number == 0 {
        // mark the trigger as being touched by the player
        (*ent).aimDebounceTime = level.time;
    }
}

pub unsafe extern "C" fn multi_trigger(ent: *mut gentity_t, activator: *mut gentity_t) {
    if (*ent).e_ThinkFunc == 3 {
        // thinkF_multi_trigger_run = 3
        // already triggered, just waiting to run
        return;
    }

    if (*ent).nextthink > level.time {
        if (*ent).spawnflags & 2048 != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in a single frame
            if (*ent).painDebounceTime != 0 && (*ent).painDebounceTime != level.time {
                // this should still allow subsequent ents to fire this trigger in the current frame
                return; // can't retrigger until the wait is over
            }
        } else {
            return;
        }
    }
    if (*ent).spawnflags & 32 != 0 {
        (*ent).nextthink = level.time + (*ent).delay as c_int;

        // trace_t	viewTrace;
        // gi.trace(&viewTrace, ent->currentOrigin, 0, 0, activator->currentOrigin, ent->s.number, MASK_SHOT);
        // if ((viewTrace.allsolid) || (viewTrace.startsolid) || 	(viewTrace.entityNum!=activator->s.number))
        // {
        //	return;
        // }
    }

    // if the player has already activated this trigger this frame
    if !activator.is_null() && (*activator).s.number == 0 && (*ent).aimDebounceTime == level.time {
        return;
    }

    if (*ent).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // Not active at this time
        return;
    }

    (*ent).activator = activator;

    if (*ent).delay != 0.0 && (*ent).painDebounceTime < (level.time as f32 + (*ent).delay) as c_int {
        // delay before firing trigger
        (*ent).e_ThinkFunc = 3; // thinkF_multi_trigger_run = 3
        (*ent).nextthink = level.time + (*ent).delay as c_int;
        (*ent).painDebounceTime = level.time;
    } else {
        multi_trigger_run(ent);
    }
}

pub unsafe extern "C" fn Use_Multi(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    multi_trigger(ent, activator);
}

pub unsafe extern "C" fn Touch_Multi(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    if (*other).client.is_null() {
        return;
    }

    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // set by target_deactivate
        return;
    }

    if (*self_).noDamageTeam != 0 {
        if (*(*other).client).playerTeam != (*self_).noDamageTeam {
            return;
        }
    }

    // moved to just above multi_trigger because up here it just checks if the trigger is not being touched
    // we want it to check any conditions set on the trigger, if one of those isn't met, the trigger is considered to be "cleared"
    // if ( (*self_).e_ThinkFunc == thinkF_trigger_cleared_fire )
    // {
    //	// We're waiting to fire our target2 first
    //	(*self_).nextthink = level.time + (*self_).speed;
    //	return;
    // }

    if (*self_).spawnflags & 1 != 0 {
        if (*other).s.number != 0 {
            return;
        }
    } else {
        if (*self_).spawnflags & 16 != 0 {
            // NPCONLY
            if (*other).NPC.is_null() {
                return;
            }
        }

        if !(*self_).NPC_targetname.is_null() && *(*self_).NPC_targetname as u8 != 0 {
            if !(*other).script_targetname.is_null() && *(*other).script_targetname as u8 != 0 {
                if Q_stricmp((*self_).NPC_targetname, (*other).script_targetname) != 0 {
                    // not the right guy to fire me off
                    return;
                }
            } else {
                return;
            }
        }
    }

    if (*self_).spawnflags & 4 != 0 {
        // USE_BUTTON
        if (*other).client.is_null() {
            return;
        }

        if (*(*other).client).usercmd.buttons & 1 == 0 {
            // BUTTON_USE = 1
            // not pressing use button
            return;
        }
    }

    if (*self_).spawnflags & 2 != 0 {
        // FACING
        let mut forward: [f32; 3] = [0.0; 3];

        if !(*other).client.is_null() {
            AngleVectors(
                core::ptr::addr_of!((*(*other).client).ps.viewangles),
                core::ptr::addr_of_mut!(forward),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        } else {
            AngleVectors(
                core::ptr::addr_of!((*other).currentAngles),
                core::ptr::addr_of_mut!(forward),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if DotProduct(core::ptr::addr_of!((*self_).movedir), core::ptr::addr_of!(forward)) < 0.5 {
            // Not Within 45 degrees
            return;
        }
    }

    if (*self_).spawnflags & 8 != 0 {
        // FIRE_BUTTON
        if (*other).client.is_null() {
            return;
        }

        if ((*(*other).client).ps.eFlags & 4 == 0) // EF_FIRING = 4
            && ((*(*other).client).ps.eFlags & 8 == 0)
        {
            // EF_ALT_FIRING = 8
            // not pressing fire button or altfire button
            return;
        }

        // FIXME: do we care about the sniper rifle or not?

        if (*other).s.number == 0 && ((*(*other).client).ps.weapon > 16 || (*(*other).client).ps.weapon <= 0) {
            // MAX_PLAYER_WEAPONS = 16, WP_NONE = 0
            // don't care about non-player weapons if this is the player
            return;
        }
    }

    if !(*other).client.is_null() && (*self_).radius != 0.0 {
        let mut eyeSpot: [f32; 3] = [0.0; 3];

        // Only works if your head is in it, but we allow leaning out
        // NOTE: We don't use CalcEntitySpot SPOT_HEAD because we don't want this
        // to be reliant on the physical model the player uses.
        VectorCopy(
            core::ptr::addr_of!((*other).currentOrigin),
            core::ptr::addr_of_mut!(eyeSpot),
        );
        eyeSpot[2] += (*(*other).client).ps.viewheight as f32;

        if G_PointInBounds(
            core::ptr::addr_of!(eyeSpot),
            core::ptr::addr_of!((*self_).absmin),
            core::ptr::addr_of!((*self_).absmax),
        ) != 0
        {
            if ((*(*other).client).ps.eFlags & 4 == 0) && ((*(*other).client).ps.eFlags & 8 == 0) {
                // not attacking, so hiding bonus
                // FIXME:  should really have sound events clear the hiddenDist
                (*(*other).client).hiddenDist = (*self_).radius;
                // NOTE: movedir HAS to be normalized!
                if VectorLength(core::ptr::addr_of!((*self_).movedir)) > 0.0 {
                    // They can only be hidden from enemies looking in this direction
                    VectorCopy(
                        core::ptr::addr_of!((*self_).movedir),
                        core::ptr::addr_of_mut!((*(*other).client).hiddenDir),
                    );
                } else {
                    VectorClear(core::ptr::addr_of_mut!((*(*other).client).hiddenDir));
                }
            }
        }
    }

    if (*self_).spawnflags & 4 != 0 {
        // USE_BUTTON
        NPC_SetAnim(other, 0, 24, 5); // SETANIM_TORSO = 0, BOTH_BUTTON_HOLD = 24, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD = 5
                                      /*
                                      if ( !VectorLengthSquared( other->client->ps.velocity ) && !PM_CrouchAnim( other->client->ps.legsAnim ) )
                                      {
                                          NPC_SetAnim( other, SETANIM_LEGS, BOTH_BUTTON_HOLD, SETANIM_FLAG_NORMAL|SETANIM_FLAG_HOLD );
                                      }
                                      */
                                      // (*other).client->ps.weaponTime = (*other).client->ps.torsoAnimTimer;
    }

    if (*self_).e_ThinkFunc == 5 {
        // thinkF_trigger_cleared_fire = 5
        // We're waiting to fire our target2 first
        (*self_).nextthink = level.time + (*self_).speed as c_int;
        return;
    }

    if (*self_).spawnflags & 32 != 0 {
        if Pilot_ActivePilotCount() >= (*self_).lastInAirTime {
            return;
        }
    }

    multi_trigger(self_, other);
}

pub unsafe extern "C" fn trigger_cleared_fire(self_: *mut gentity_t) {
    G_UseTargets2(self_, (*self_).activator, (*self_).target2);
    (*self_).e_ThinkFunc = 0; // thinkF_NULL

    // should start the wait timer now, because the trigger's just been cleared, so we must "wait" from this point
    if (*self_).wait > 0.0 {
        (*self_).nextthink = level.time + (((*self_).wait + (*self_).random * crandom()) * 1000.0) as c_int;
    }
}

pub unsafe extern "C" fn G_TriggerActive(self_: *mut gentity_t) -> c_int {
    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // set by target_deactivate
        return 0; // qfalse
    }

    if (*self_).spawnflags & 1 != 0 {
        // player only
        return 0; // qfalse
    }

    /*
    ???
    if ( (*self_).spawnflags & 4 )
    {
        // USE_BUTTON
        return qfalse;
    }
    */

    /*
    ???
    if ( (*self_).spawnflags & 8 )
    {
        // FIRE_BUTTON
        return qfalse;
    }
    */

    /*
    if ( (*self_).radius )
    {
        // Only works if your head is in it, but we allow leaning out
        // NOTE: We don't use CalcEntitySpot SPOT_HEAD because we don't want this
        // to be reliant on the physical model the player uses.
        return qfalse;
    }
    */
    1 // qtrue
}

pub fn InitTrigger(self_: *mut gentity_t) {
    unsafe {
        if VectorCompare(
            core::ptr::addr_of!((*self_).s.angles),
            core::ptr::addr_of!([0.0f32; 3]),
        ) == 0
        {
            G_SetMovedir(
                core::ptr::addr_of!((*self_).s.angles),
                core::ptr::addr_of_mut!((*self_).movedir),
            );
        }

        gi.SetBrushModel(self_, (*self_).model);
        (*self_).contents = 1; // CONTENTS_TRIGGER
        (*self_).svFlags = 16; // SVF_NOCLIENT

        if (*self_).spawnflags & 128 != 0 {
            (*self_).svFlags |= 8; // SVF_INACTIVE
        }
    }
}

/*QUAKED trigger_multiple (.1 .5 .1) ? PLAYERONLY FACING USE_BUTTON FIRE_BUTTON NPCONLY LIMITED_PILOT x INACTIVE MULTIPLE
PLAYERONLY - only a player can trigger this by touch
FACING - Won't fire unless triggering ent's view angles are within 45 degrees of trigger's angles (in addition to any other conditions)
USE_BUTTON - Won't fire unless player is in it and pressing use button (in addition to any other conditions)
FIRE_BUTTON - Won't fire unless player/NPC is in it and pressing fire button (in addition to any other conditions)
NPCONLY - only non-player NPCs can trigger this by touch
LIMITED_PILOT - only spawn if there are open pilot slots
INACTIVE - Start off, has to be activated to be touchable/usable
MULTIPLE - multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

"wait"		Seconds between triggerings, 0 default, number < 0 means one time only.
"random"	wait variance, default is 0
"delay"		how many seconds to wait to fire targets after tripped
"hiderange" As long as NPC's head is in this trigger, NPCs out of this hiderange cannot see him.  If you set an angle on the trigger, they're only hidden from enemies looking in that direction.  the player's crouch viewheight is 36, his standing viewheight is 54.  So a trigger thast should hide you when crouched but not standing should be 48 tall.
"target2"	The trigger will fire this only when the trigger has been activated and subsequently 'cleared'( once any of the conditions on the trigger have not been satisfied).  This will not fire the "target" more than once until the "target2" is fired (trigger field is 'cleared')
"speed"		How many seconds to wait to fire the target2, default is 1
"noise"		Sound to play when the trigger fires (plays at activator's origin)
"max_pilots"	Number of pilots this spawner will allow

Variable sized repeatable trigger.  Must be targeted at one or more entities.
so, the basic time between firing is a random time between
(wait - random) and (wait + random)

"NPC_targetname" - If set, only an NPC with a matching NPC_targetname will trip this trigger
"team" - If set, only this team can trip this trigger
	player
	enemy
	neutral

"soundSet"	Ambient sound set to play when this trigger is activated
*/
pub unsafe extern "C" fn SP_trigger_multiple(ent: *mut gentity_t) {
    let mut buffer: [c_char; 256] = [0; 256]; // MAX_QPATH = 256
    let mut s: *const c_char = core::ptr::null();

    if G_SpawnString(
        b"noise\0".as_ptr() as *const c_char,
        b"*NOSOUND*\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!(s),
    ) != 0
    {
        Q_strncpyz(
            buffer.as_mut_ptr(),
            s,
            core::mem::size_of::<[c_char; 256]>(),
        );
        COM_DefaultExtension(
            buffer.as_mut_ptr(),
            core::mem::size_of::<[c_char; 256]>(),
            b".wav\0".as_ptr() as *const c_char,
        );
        (*ent).noise_index = G_SoundIndex(buffer.as_ptr());
    }

    G_SpawnFloat(b"wait\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, core::ptr::addr_of_mut!((*ent).wait));
    G_SpawnFloat(
        b"random\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*ent).random),
    );
    G_SpawnInt(
        b"max_pilots\0".as_ptr() as *const c_char,
        b"2\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*ent).lastInAirTime),
    );

    if ((*ent).wait > 0.0) && ((*ent).random >= (*ent).wait) {
        (*ent).random = (*ent).wait - 0.1; // FRAMETIME = 0.1 in seconds
        gi.Printf(
            b"^3trigger_multiple has random >= wait\n\0".as_ptr() as *const c_char,
        );
    }

    (*ent).delay *= 1000.0; // 1 = 1 msec, 1000 = 1 sec
    if ((*ent).speed == 0.0) && !(*ent).target2.is_null() && *(*ent).target2 as u8 != 0 {
        (*ent).speed = 1000.0;
    } else {
        (*ent).speed *= 1000.0;
    }

    (*ent).e_TouchFunc = 1; // touchF_Touch_Multi = 1
    (*ent).e_UseFunc = 1; // useF_Use_Multi = 1

    if !(*ent).team.is_null() && *(*ent).team as u8 != 0 {
        (*ent).noDamageTeam = GetIDForString(
            core::ptr::addr_of!(TeamTable) as *const (),
            (*ent).team,
        );
        (*ent).team = core::ptr::null();
    }

    InitTrigger(ent);
    gi.linkentity(ent);
}

/*QUAKED trigger_once (.5 1 .5) ? PLAYERONLY FACING USE_BUTTON FIRE_BUTTON NPCONLY x x INACTIVE MULTIPLE
PLAYERONLY - only a player can trigger this by touch
FACING - Won't fire unless triggering ent's view angles are within 45 degrees of trigger's angles (in addition to any other conditions)
USE_BUTTON - Won't fire unless player is in it and pressing use button (in addition to any other conditions)
FIRE_BUTTON - Won't fire unless player/NPC is in it and pressing fire button (in addition to any other conditions)
NPCONLY - only non-player NPCs can trigger this by touch
INACTIVE - Start off, has to be activated to be touchable/usable
MULTIPLE - multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

"random"	wait variance, default is 0
"delay"		how many seconds to wait to fire targets after tripped
Variable sized repeatable trigger.  Must be targeted at one or more entities.
so, the basic time between firing is a random time between
(wait - random) and (wait + random)
"noise"		Sound to play when the trigger fires (plays at activator's origin)

"NPC_targetname" - If set, only an NPC with a matching NPC_targetname will trip this trigger
"team" - If set, only this team can trip this trigger
	player
	enemy
	neutral

"soundSet"	Ambient sound set to play when this trigger is activated
*/
pub unsafe extern "C" fn SP_trigger_once(ent: *mut gentity_t) {
    let mut buffer: [c_char; 256] = [0; 256]; // MAX_QPATH = 256
    let mut s: *const c_char = core::ptr::null();

    if G_SpawnString(
        b"noise\0".as_ptr() as *const c_char,
        b"*NOSOUND*\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!(s),
    ) != 0
    {
        Q_strncpyz(
            buffer.as_mut_ptr(),
            s,
            core::mem::size_of::<[c_char; 256]>(),
        );
        COM_DefaultExtension(
            buffer.as_mut_ptr(),
            core::mem::size_of::<[c_char; 256]>(),
            b".wav\0".as_ptr() as *const c_char,
        );
        (*ent).noise_index = G_SoundIndex(buffer.as_ptr());
    }

    (*ent).wait = -1.0;

    (*ent).e_TouchFunc = 1; // touchF_Touch_Multi = 1
    (*ent).e_UseFunc = 1; // useF_Use_Multi = 1

    if !(*ent).team.is_null() && *(*ent).team as u8 != 0 {
        (*ent).noDamageTeam = GetIDForString(
            core::ptr::addr_of!(TeamTable) as *const (),
            (*ent).team,
        );
        (*ent).team = core::ptr::null();
    }

    (*ent).delay *= 1000.0; // 1 = 1 msec, 1000 = 1 sec

    InitTrigger(ent);
    gi.linkentity(ent);
}

/*QUAKED trigger_bidirectional (.1 .5 .1) ? PLAYER_ONLY x x x x x x INACTIVE
NOT IMPLEMENTED
INACTIVE - Start off, has to be activated to be touchable/usable

set "angle" for forward direction
Fires "target" when someone moves through it in direction of angle
Fires "backwardstarget" when someone moves through it in the opposite direction of angle

"NPC_targetname" - If set, only an NPC with a matching NPC_targetname will trip this trigger

"wait" - how long to wait between triggerings

  TODO:
	count
*/
pub unsafe extern "C" fn SP_trigger_bidirectional(ent: *mut gentity_t) {
    G_FreeEntity(ent);
    // FIXME: Implement
    /*	if(!(*ent).wait)
    {
        (*ent).wait = -1;
    }

    (*ent).touch = Touch_Multi;
    (*ent).use = Use_Multi;

    InitTrigger( ent );
    gi.linkentity (ent);
    */
}

/*QUAKED trigger_location (.1 .5 .1) ?
When an ent is asked for it's location, it will return this ent's "message" field if it is in it.
  "message" - location name

  NOTE: always rectangular
*/
pub unsafe extern "C" fn G_GetLocationForEnt(ent: *mut gentity_t) -> *const c_char {
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut found: *mut gentity_t = core::ptr::null_mut();

    VectorAdd(
        core::ptr::addr_of!((*ent).currentOrigin),
        core::ptr::addr_of!((*ent).mins),
        core::ptr::addr_of_mut!(mins),
    );
    VectorAdd(
        core::ptr::addr_of!((*ent).currentOrigin),
        core::ptr::addr_of!((*ent).maxs),
        core::ptr::addr_of_mut!(maxs),
    );

    loop {
        found = G_Find(found, 0, b"trigger_location\0".as_ptr() as *const c_char);
        if found.is_null() {
            break;
        }
        if gi.EntityContact(
            core::ptr::addr_of!(mins),
            core::ptr::addr_of!(maxs),
            found,
        ) != 0
        {
            return (*found).message;
        }
    }

    core::ptr::null()
}

pub unsafe extern "C" fn SP_trigger_location(ent: *mut gentity_t) {
    if (*ent).message.is_null() || *(*ent).message as u8 == 0 {
        gi.Printf(b"WARNING: trigger_location with no message!\n\0".as_ptr() as *const c_char);
        G_FreeEntity(ent);
        return;
    }

    gi.SetBrushModel(ent, (*ent).model);
    (*ent).contents = 0;
    (*ent).svFlags = 16; // SVF_NOCLIENT

    gi.linkentity(ent);
}

/*
==============================================================================

trigger_always

==============================================================================
*/

pub unsafe extern "C" fn trigger_always_think(ent: *mut gentity_t) {
    G_UseTargets(ent, ent);
    G_FreeEntity(ent);
}

/*QUAKED trigger_always (.1 .5 .1) (-8 -8 -8) (8 8 8)
This trigger will always fire.  It is activated by the world.
*/
pub unsafe extern "C" fn SP_trigger_always(ent: *mut gentity_t) {
    // we must have some delay to make sure our use targets are present
    (*ent).nextthink = level.time + 300;
    (*ent).e_ThinkFunc = 2; // thinkF_trigger_always_think = 2
}

/*
==============================================================================

trigger_push

==============================================================================
*/
const PUSH_CONVEYOR: c_int = 32;

pub unsafe extern "C" fn trigger_push_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // set by target_deactivate
        return;
    }

    if level.time < (*self_).painDebounceTime + (*self_).wait as c_int {
        // normal 'wait' check
        if (*self_).spawnflags & 2048 != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in one frame
            if (*self_).painDebounceTime != 0 && level.time > (*self_).painDebounceTime {
                // if we haven't reached the next frame continue to let ents touch the trigger
                return;
            }
        } else {
            // only allowing one ent per frame to touch trigger
            return;
        }
    }

    // if the player has already activated this trigger this frame
    if !other.is_null() && (*other).s.number == 0 && (*self_).aimDebounceTime == level.time {
        return;
    }

    if (*self_).spawnflags & PUSH_CONVEYOR != 0 {
        // only push player if he's on the ground
        if (*other).s.groundEntityNum == -1 {
            // ENTITYNUM_NONE = -1
            return;
        }
    }

    if (*self_).spawnflags & 1 != 0 {
        // PLAYERONLY
        if (*other).s.number != 0 {
            return;
        }
    } else {
        if (*self_).spawnflags & 8 != 0 {
            // NPCONLY
            if (*other).NPC.is_null() {
                return;
            }
        }
    }

    if (*other).client.is_null() {
        if (*other).s.pos.trType != 0 && (*other).s.pos.trType != 1 && (*other).s.pos.trType != 2
            && VectorLengthSquared(core::ptr::addr_of!((*other).s.pos.trDelta)) > 0.0
        {
            // already moving (TR_STATIONARY, TR_LINEAR_STOP, TR_NONLINEAR_STOP)
            VectorCopy(
                core::ptr::addr_of!((*other).currentOrigin),
                core::ptr::addr_of_mut!((*other).s.pos.trBase),
            );
            VectorCopy(
                core::ptr::addr_of!((*self_).s.origin2),
                core::ptr::addr_of_mut!((*other).s.pos.trDelta),
            );
            (*other).s.pos.trTime = level.time;
        }
        return;
    }

    if (*(*other).client).ps.pm_type != 0 {
        // PM_NORMAL = 0
        return;
    }

    if (*self_).spawnflags & 16 != 0 {
        // relative, dir to it * speed
        let mut dir: [f32; 3] = [0.0; 3];
        VectorSubtract(
            core::ptr::addr_of!((*self_).s.origin2),
            core::ptr::addr_of!((*other).currentOrigin),
            core::ptr::addr_of_mut!(dir),
        );
        if (*self_).speed != 0.0 {
            VectorNormalize(core::ptr::addr_of_mut!(dir));
            VectorScale(
                core::ptr::addr_of!(dir),
                (*self_).speed,
                core::ptr::addr_of_mut!(dir),
            );
        }
        VectorCopy(
            core::ptr::addr_of!(dir),
            core::ptr::addr_of_mut!((*(*other).client).ps.velocity),
        );
    } else if (*self_).spawnflags & 4 != 0 {
        // linear dir * speed
        VectorScale(
            core::ptr::addr_of!((*self_).s.origin2),
            (*self_).speed,
            core::ptr::addr_of_mut!((*(*other).client).ps.velocity),
        );
    } else {
        VectorCopy(
            core::ptr::addr_of!((*self_).s.origin2),
            core::ptr::addr_of_mut!((*(*other).client).ps.velocity),
        );
    }
    // so we don't take damage unless we land lower than we start here...
    (*(*other).client).ps.forceJumpZStart = 0;
    (*(*other).client).ps.pm_flags |= 8; // PMF_TRIGGER_PUSHED
    (*(*other).client).ps.jumpZStart = (*(*other).client).ps.origin[2] as c_int;

    if (*self_).wait == -1.0 {
        (*self_).e_TouchFunc = 0; // touchF_NULL
    } else if (*self_).wait > 0.0 {
        (*self_).painDebounceTime = level.time;
    }
    if !other.is_null() && (*other).s.number == 0 {
        // mark that the player has activated this trigger this frame
        (*self_).aimDebounceTime = level.time;
    }
}

const PUSH_CONSTANT: c_int = 2;

/*
=================
AimAtTarget

Calculate origin2 so the target apogee will be hit
=================
*/
pub unsafe extern "C" fn AimAtTarget(self_: *mut gentity_t) {
    let mut ent: *mut gentity_t;
    let mut origin: [f32; 3] = [0.0; 3];
    let mut height: f32;
    let mut gravity: f32;
    let mut time: f32;
    let mut forward: f32;
    let mut dist: f32;

    VectorAdd(
        core::ptr::addr_of!((*self_).absmin),
        core::ptr::addr_of!((*self_).absmax),
        core::ptr::addr_of_mut!(origin),
    );
    VectorScale(
        core::ptr::addr_of!(origin),
        0.5,
        core::ptr::addr_of_mut!(origin),
    );

    ent = G_PickTarget((*self_).target);
    if ent.is_null() {
        G_FreeEntity(self_);
        return;
    }

    if !(*self_).classname.is_null() && Q_stricmp((*self_).classname, b"trigger_push\0".as_ptr() as *const c_char) == 0 {
        if (*self_).spawnflags & 2 != 0 {
            // check once a second to see if we should activate or deactivate ourselves
            (*self_).e_ThinkFunc = 4; // thinkF_trigger_push_checkclear = 4
            (*self_).nextthink = level.time + 100; // FRAMETIME = 100
        }

        if (*self_).spawnflags & 16 != 0 {
            // relative, not an arc or linear
            VectorCopy(
                core::ptr::addr_of!((*ent).currentOrigin),
                core::ptr::addr_of_mut!((*self_).s.origin2),
            );
            return;
        } else if (*self_).spawnflags & 4 != 0 {
            // linear, not an arc
            VectorSubtract(
                core::ptr::addr_of!((*ent).currentOrigin),
                core::ptr::addr_of!(origin),
                core::ptr::addr_of_mut!((*self_).s.origin2),
            );
            VectorNormalize(core::ptr::addr_of_mut!((*self_).s.origin2));
            return;
        }
    }

    if !(*self_).classname.is_null() && Q_stricmp((*self_).classname, b"target_push\0".as_ptr() as *const c_char) == 0 {
        if (*self_).spawnflags & PUSH_CONSTANT != 0 {
            VectorSubtract(
                core::ptr::addr_of!((*ent).s.origin),
                core::ptr::addr_of!((*self_).s.origin),
                core::ptr::addr_of_mut!((*self_).s.origin2),
            );
            VectorNormalize(core::ptr::addr_of_mut!((*self_).s.origin2));
            VectorScale(
                core::ptr::addr_of!((*self_).s.origin2),
                (*self_).speed,
                core::ptr::addr_of_mut!((*self_).s.origin2),
            );
            return;
        }
    }
    height = (*ent).s.origin[2] - origin[2];
    if height < 0.0 {
        // sqrt of negative is bad!
        height = 0.0;
    }
    gravity = (*g_gravity).value;
    if gravity < 0.0 {
        gravity = 0.0;
    }
    time = sqrt(height / (0.5 * gravity));
    if time == 0.0 {
        G_FreeEntity(self_);
        return;
    }

    // set s.origin2 to the push velocity
    VectorSubtract(
        core::ptr::addr_of!((*ent).s.origin),
        core::ptr::addr_of!(origin),
        core::ptr::addr_of_mut!((*self_).s.origin2),
    );
    (*self_).s.origin2[2] = 0.0;
    dist = VectorNormalize(core::ptr::addr_of_mut!((*self_).s.origin2));

    forward = dist / time;
    VectorScale(
        core::ptr::addr_of!((*self_).s.origin2),
        forward,
        core::ptr::addr_of_mut!((*self_).s.origin2),
    );

    (*self_).s.origin2[2] = time * gravity;
}

pub unsafe extern "C" fn trigger_push_checkclear(self_: *mut gentity_t) {
    let mut trace: trace_t = core::mem::zeroed();
    let mut center: [f32; 3] = [0.0; 3];

    (*self_).nextthink = level.time + 500;

    VectorAdd(
        core::ptr::addr_of!((*self_).absmin),
        core::ptr::addr_of!((*self_).absmax),
        core::ptr::addr_of_mut!(center),
    );
    VectorScale(core::ptr::addr_of!(center), 0.5, core::ptr::addr_of_mut!(center));

    let mut target: *mut gentity_t = G_Find(core::ptr::null_mut(), 0, (*self_).target);
    gi.trace(
        core::ptr::addr_of_mut!(trace),
        core::ptr::addr_of!(center),
        core::ptr::addr_of!([0.0f32; 3]),
        core::ptr::addr_of!([0.0f32; 3]),
        core::ptr::addr_of!((*target).currentOrigin),
        -1, // ENTITYNUM_NONE
        1,  // CONTENTS_SOLID
    );

    if trace.fraction >= 1.0 {
        // can trace, turn on
        (*self_).contents |= 1; // CONTENTS_TRIGGER
        (*self_).e_TouchFunc = 1; // touchF_trigger_push_touch = 1
        gi.linkentity(self_);
    } else {
        // no trace, turn off
        (*self_).contents &= !1; // CONTENTS_TRIGGER
        (*self_).e_TouchFunc = 0; // touchF_NULL
        gi.unlinkentity(self_);
    }
}

/*QUAKED trigger_push (.1 .5 .1) ? PLAYERONLY CHECKCLEAR LINEAR NPCONLY RELATIVE CONVEYOR x INACTIVE MULTIPLE
Must point at a target_position, which will be the apex of the leap.
This will be client side predicted, unlike target_push
PLAYERONLY - only the player is affected
LINEAR - Instead of tossing the client at the target_position, it will push them towards it.  Must set a "speed" (see below)
CHECKCLEAR - Every 1 second, it will check to see if it can trace to the target_position, if it can, the trigger is touchable, if it can't, the trigger is not touchable
NPCONLY - only NPCs are affected
RELATIVE - instead of pushing you in a direction that is always from the center of the trigger to the target_position, it pushes *you* toward the target position, relative to your current location (can use with "speed"... if don't set a speed, it will use the distance from you to the target_position)
CONVEYOR - acts like a conveyor belt, will only push if player is on the ground ( should probably use RELATIVE also, if you want a true conveyor belt )
INACTIVE - not active until targeted by a target_activate
MULTIPLE - multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

wait - how long to wait between pushes: -1 = push only once
speed - when used with the LINEAR spawnflag, pushes the client toward the position at a constant speed (default is 1000)
*/
pub unsafe extern "C" fn SP_trigger_push(self_: *mut gentity_t) {
    InitTrigger(self_);

    if (*self_).wait > 0.0 {
        (*self_).wait *= 1000.0;
    }

    // unlike other triggers, we need to send this one to the client
    (*self_).svFlags &= !16; // SVF_NOCLIENT

    (*self_).s.eType = 21; // ET_PUSH_TRIGGER

    if (*self_).spawnflags & 2 == 0 {
        // start on
        (*self_).e_TouchFunc = 1; // touchF_trigger_push_touch = 1
    }
    if (*self_).spawnflags & 4 != 0 {
        // linear
        (*self_).speed = 1000.0;
    }
    (*self_).e_ThinkFunc = 6; // thinkF_AimAtTarget = 6
    (*self_).nextthink = level.time + 200; // START_TIME_LINK_ENTS = 200
    gi.linkentity(self_);
}

pub unsafe extern "C" fn Use_target_push(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if (*activator).client.is_null() {
        return;
    }

    if (*(*activator).client).ps.pm_type != 0 {
        // PM_NORMAL = 0
        return;
    }

    G_ActivateBehavior(self_, 32); // BSET_USE = 32

    VectorCopy(
        core::ptr::addr_of!((*self_).s.origin2),
        core::ptr::addr_of_mut!((*(*activator).client).ps.velocity),
    );

    if (*self_).spawnflags & 4 != 0 {
        // lower
        // reset this so I don't take falling damage when I land
        (*(*activator).client).ps.jumpZStart = (*activator).currentOrigin[2] as c_int;
    }

    // so we don't take damage unless we land lower than we start here...
    (*(*activator).client).ps.forceJumpZStart = 0;
    (*(*activator).client).ps.pm_flags |= 8; // PMF_TRIGGER_PUSHED

    // play fly sound every 1.5 seconds
    if (*self_).noise_index != 0 && (*activator).fly_sound_debounce_time < level.time {
        (*activator).fly_sound_debounce_time = level.time + 1500;
        G_Sound(activator, (*self_).noise_index);
    }
}

/*QUAKED target_push (.5 .5 .5) (-8 -8 -8) (8 8 8) ENERGYNOISE CONSTANT NO_DAMAGE
When triggered, pushes the activator in the direction of angles
"speed"		defaults to 1000
ENERGYNOISE plays energy noise
CONSTANT will push activator in direction of 'target' at constant 'speed'
NO_DAMAGE the activator won't take falling damage after being pushed
*/
pub unsafe extern "C" fn SP_target_push(self_: *mut gentity_t) {
    if (*self_).speed == 0.0 {
        (*self_).speed = 1000.0;
    }
    G_SetMovedir(
        core::ptr::addr_of!((*self_).s.angles),
        core::ptr::addr_of_mut!((*self_).s.origin2),
    );
    VectorScale(
        core::ptr::addr_of!((*self_).s.origin2),
        (*self_).speed,
        core::ptr::addr_of_mut!((*self_).s.origin2),
    );

    if (*self_).spawnflags & 1 != 0 {
        // (*self_).noise_index = G_SoundIndex("sound/ambience/forge/antigrav.wav");
    }
    if !(*self_).target.is_null() {
        VectorCopy(
            core::ptr::addr_of!((*self_).s.origin),
            core::ptr::addr_of_mut!((*self_).absmin),
        );
        VectorCopy(
            core::ptr::addr_of!((*self_).s.origin),
            core::ptr::addr_of_mut!((*self_).absmax),
        );
        (*self_).e_ThinkFunc = 6; // thinkF_AimAtTarget = 6
        (*self_).nextthink = level.time + 200; // START_TIME_LINK_ENTS = 200
    }
    (*self_).e_UseFunc = 1; // useF_Use_target_push = 1
}

/*
==============================================================================

trigger_teleport

==============================================================================
*/
const SNAP_ANGLES: c_int = 1;
const NO_MISSILES: c_int = 2;
const NO_NPCS: c_int = 4;
const TTSF_STASIS: c_int = 8;
const TTSF_DEAD_OK: c_int = 16;

pub unsafe extern "C" fn trigger_teleporter_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    let mut dest: *mut gentity_t;

    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // set by target_deactivate
        return;
    }

    dest = G_PickTarget((*self_).target);
    if dest.is_null() {
        gi.Printf(b"Couldn\'t find teleporter destination\n\0".as_ptr() as *const c_char);
        return;
    }

    if !(*other).client.is_null() {
        if (*(*other).client).ps.pm_type == 3 {
            // PM_DEAD = 3
            if (*self_).spawnflags & TTSF_DEAD_OK == 0 {
                // dead men can't teleport
                return;
            }
        }
        if !(*other).NPC.is_null() {
            if (*self_).spawnflags & NO_NPCS != 0 {
                return;
            }
        }

        if (*(*other).client).playerTeam != 0
            && SpotWouldTelefrag2(
                other,
                core::ptr::addr_of!((*dest).currentOrigin),
            ) != 0
        {
            // Don't go through if something blocking on the other side
            return;
        }

        TeleportPlayer(
            other,
            core::ptr::addr_of!((*dest).s.origin),
            core::ptr::addr_of!((*dest).s.angles),
        );
    } else if (*self_).svFlags & 128 == 0
        && (*self_).spawnflags & NO_MISSILES == 0
        && VectorLengthSquared(core::ptr::addr_of!((*other).s.pos.trDelta)) > 0.0
    {
        // SVF_NO_TELEPORT = 128
        // It's a mover of some sort and is currently moving
        let mut diffAngles: [f32; 3] = [0.0, 0.0, 0.0];
        let mut snap: c_int = 0; // qfalse

        if !(*self_).lastEnemy.is_null() {
            VectorSubtract(
                core::ptr::addr_of!((*dest).s.angles),
                core::ptr::addr_of!((*(*self_).lastEnemy).s.angles),
                core::ptr::addr_of_mut!(diffAngles),
            );
        } else {
            // snaps to angle
            VectorSubtract(
                core::ptr::addr_of!((*dest).s.angles),
                core::ptr::addr_of!((*other).currentAngles),
                core::ptr::addr_of_mut!(diffAngles),
            );
            snap = 1; // qtrue
        }

        TeleportMover(
            other,
            core::ptr::addr_of!((*dest).s.origin),
            core::ptr::addr_of!(diffAngles),
            snap,
        );
    }
}

pub unsafe extern "C" fn trigger_teleporter_find_closest_portal(self_: *mut gentity_t) {
    let mut found: *mut gentity_t = core::ptr::null_mut();
    let mut org: [f32; 3] = [0.0; 3];
    let mut vec: [f32; 3] = [0.0; 3];
    let mut dist: f32;
    let mut bestDist: f32 = 64.0 * 64.0;

    VectorAdd(
        core::ptr::addr_of!((*self_).mins),
        core::ptr::addr_of!((*self_).maxs),
        core::ptr::addr_of_mut!(org),
    );
    VectorScale(core::ptr::addr_of!(org), 0.5, core::ptr::addr_of_mut!(org));

    loop {
        found = G_Find(found, 0, b"misc_portal_surface\0".as_ptr() as *const c_char);
        if found.is_null() {
            break;
        }
        VectorSubtract(
            core::ptr::addr_of!((*found).currentOrigin),
            core::ptr::addr_of!(org),
            core::ptr::addr_of_mut!(vec),
        );
        dist = VectorLengthSquared(core::ptr::addr_of!(vec));
        if dist < bestDist {
            (*self_).lastEnemy = found;
            bestDist = dist;
        }
    }

    if !(*self_).lastEnemy.is_null() {
        gi.Printf(b"trigger_teleporter found misc_portal_surface\n\0".as_ptr() as *const c_char);
    }
    (*self_).e_ThinkFunc = 0; // thinkF_NULL
}

/*QUAKED trigger_teleport (.1 .5 .1) ? SNAP_ANGLES NO_MISSILES NO_NPCS STASIS DEAD_OK x x INACTIVE
Allows client side prediction of teleportation events.
Must point at a target_position, which will be the teleport destination.

SNAP_ANGLES - Make the entity that passes through snap to the target_position's angles
NO_MISSILES - Missiles and thrown objects cannot pass through
NO_NPCS - NPCs cannot pass through
STASIS - will play stasis teleport sound and fx instead of starfleet
DEAD_OK - even if dead, you will teleport
*/
pub unsafe extern "C" fn SP_trigger_teleport(self_: *mut gentity_t) {
    InitTrigger(self_);

    // unlike other triggers, we need to send this one to the client
    (*self_).svFlags &= !16; // SVF_NOCLIENT

    (*self_).s.eType = 20; // ET_TELEPORT_TRIGGER

    (*self_).e_TouchFunc = 1; // touchF_trigger_teleporter_touch = 1

    (*self_).e_ThinkFunc = 7; // thinkF_trigger_teleporter_find_closest_portal = 7
    (*self_).nextthink = level.time + 200; // START_TIME_LINK_ENTS = 200

    gi.linkentity(self_);
}

/*
==============================================================================

trigger_hurt

==============================================================================
*/

/*QUAKED trigger_hurt (.1 .5 .1) ? START_OFF PLAYERONLY SILENT NO_PROTECTION LOCKCAM FALLING ELECTRICAL INACTIVE MULTIPLE
Any entity that touches this will be hurt.
It does dmg points of damage each server frame

PLAYERONLY		only the player is hurt by it
SILENT			supresses playing the sound
NO_PROTECTION	*nothing* stops the damage
LOCKCAM			Falling death results in camera locking in place
FALLING			Forces a falling scream and anim
ELECTRICAL		does electrical damage
INACTIVE		Cannot be triggered until used by a target_activate
MULTIPLE        multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

"dmg"			default 5 (whole numbers only)
"delay"			How many seconds it takes to get from 0 to "dmg" (default is 0)
"wait"			Use in instead of "SLOW" - determines how often the player gets hurt, 0.1 is every frame, 1.0 is once per second.  -1 will stop after one use
"count"			If set, FALLING death causes a fade to black in this many milliseconds (default is 10000 = 10 seconds)
"NPC_targetname" - If set, only an NPC with a matching NPC_targetname will trip this trigger
"noise"         sound to play when it hurts something ( default: "sound/world/electro" )
*/
pub unsafe extern "C" fn hurt_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, 32); // BSET_USE = 32

    // FIXME: Targeting the trigger will toggle its on / off state???
    if (*self_).linked != 0 {
        gi.unlinkentity(self_);
    } else {
        gi.linkentity(self_);
    }
}

pub unsafe extern "C" fn trigger_hurt_reset(self_: *mut gentity_t) {
    (*self_).attackDebounceTime = 0;
    (*self_).e_ThinkFunc = 0; // thinkF_NULL
}

pub unsafe extern "C" fn hurt_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    let mut dflags: c_int;
    let mut actualDmg: c_int = (*self_).damage;

    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // set by target_deactivate
        return;
    }

    if (*other).takedamage == 0 {
        return;
    }

    if level.time < (*self_).painDebounceTime + (*self_).wait as c_int {
        // normal 'wait' check
        if (*self_).spawnflags & 2048 != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in one frame
            if (*self_).painDebounceTime != 0 && level.time > (*self_).painDebounceTime {
                // if we haven't reached the next frame continue to let ents touch the trigger
                return;
            }
        } else {
            // only allowing one ent per frame to touch trigger
            return;
        }
    }

    // if the player has already activated this trigger this frame
    if !other.is_null() && (*other).s.number == 0 && (*self_).aimDebounceTime == level.time {
        return;
    }

    if (*self_).spawnflags & 2 != 0 {
        // player only
        if (*other).s.number != 0 {
            return;
        }
    }

    if !(*self_).NPC_targetname.is_null() && *(*self_).NPC_targetname as u8 != 0 {
        // I am for you, Kirk
        if !(*other).script_targetname.is_null() && *(*other).script_targetname as u8 != 0 {
            // must have a name
            if Q_stricmp((*self_).NPC_targetname, (*other).script_targetname) != 0 {
                // not the right guy to fire me off
                return;
            }
        } else {
            // no name?  No trigger.
            return;
        }
    }

    // play sound
    if (*self_).spawnflags & 4 == 0 {
        G_Sound(other, (*self_).noise_index);
    }

    if (*self_).spawnflags & 8 != 0 {
        dflags = 1; // DAMAGE_NO_PROTECTION
    } else {
        dflags = 0;
    }

    if (*self_).delay != 0.0 {
        // Increase dmg over time
        if ((*self_).attackDebounceTime as f32) < (*self_).delay {
            // FIXME: this is for the entire trigger, not per person, so if someone else jumped in after you were in it for 5 seconds, they'd get damaged faster
            actualDmg = floor(((*self_).damage as f32 * (*self_).attackDebounceTime as f32 / (*self_).delay)) as c_int;
        }
        (*self_).attackDebounceTime += 100; // FRAMETIME = 100

        (*self_).e_ThinkFunc = 8; // thinkF_trigger_hurt_reset = 8
        (*self_).nextthink = level.time + 100 * 2; // FRAMETIME*2
    }

    if actualDmg != 0 {
        if ((*self_).spawnflags & 64 != 0) && !(*other).client.is_null() {
            // electrical damage
            // zap effect
            (*other).s.powerups |= 1 << 11; // PW_SHOCKED = 11
            (*(*other).client).ps.powerups[11] = level.time + 1000;
        }

        if (*self_).spawnflags & 32 != 0 {
            // falling death
            if !(*other).NPC.is_null() && !(*other).client.is_null()
                && ((*(*other).client).NPC_class == 5 || (*(*other).client).NPC_class == 6)
            {
                // CLASS_BOBAFETT = 5, CLASS_ROCKETTROOPER = 6
                // boba never falls to his death!
                // FIXME:  fall through if jetpack broken?
                JET_FlyStart(other);
            } else {
                G_Damage(
                    other,
                    self_,
                    self_,
                    core::ptr::null(),
                    core::ptr::null(),
                    actualDmg,
                    dflags | 4, // DAMAGE_NO_ARMOR = 4
                    9,          // MOD_FALLING = 9
                );
                // G_Damage will free this ent, which makes it s.number 0, so we must check inuse...
                if (*other).s.number == 0 && (*other).health <= 0 {
                    if (*self_).count != 0 {
                        // Placeholder for CGCam_Fade call
                        // extern void CGCam_Fade( vec4_t source, vec4_t dest, float duration );
                        // let src: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
                        // let dst: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
                        // CGCam_Fade(&src, &dst, (*self_).count as f32);
                    }
                    if (*self_).spawnflags & 16 != 0 {
                        // lock cam
                        cg.overrides.active |= 256; // CG_OVERRIDE_3RD_PERSON_CDP
                        cg.overrides.thirdPersonCameraDamp = 0.0;
                    }
                    if !(*other).client.is_null() {
                        (*(*other).client).ps.pm_flags |= 256; // PMF_SLOW_MO_FALL
                    }
                    // G_SoundOnEnt( other, CHAN_VOICE, "*falling1.wav" );//CHAN_VOICE_ATTEN?
                }
            }
        } else {
            G_Damage(
                other,
                self_,
                self_,
                core::ptr::null(),
                core::ptr::null(),
                actualDmg,
                dflags,
                10, // MOD_TRIGGER_HURT = 10
            );
        }
        if !other.is_null() && (*other).s.number == 0 {
            (*self_).aimDebounceTime = level.time;
        }
        if ((*self_).spawnflags & 64 != 0) && !(*other).client.is_null() && (*other).health <= 0 {
            // electrical damage
            // just killed them, make the effect last longer since dead clients don't touch triggers
            (*(*other).client).ps.powerups[11] = level.time + 10000;
        }
        (*self_).painDebounceTime = level.time;
    }

    if (*self_).wait < 0.0 {
        (*self_).e_TouchFunc = 0; // touchF_NULL
    }
}

pub unsafe extern "C" fn SP_trigger_hurt(self_: *mut gentity_t) {
    let mut buffer: [c_char; 256] = [0; 256]; // MAX_QPATH = 256
    let mut s: *const c_char = core::ptr::null();

    InitTrigger(self_);

    if (*self_).spawnflags & 4 == 0 {
        G_SpawnString(
            b"noise\0".as_ptr() as *const c_char,
            b"sound/world/electro\0".as_ptr() as *const c_char,
            core::ptr::addr_of_mut!(s),
        );

        Q_strncpyz(
            buffer.as_mut_ptr(),
            s,
            core::mem::size_of::<[c_char; 256]>(),
        );
        (*self_).noise_index = G_SoundIndex(buffer.as_ptr());
    }

    (*self_).e_TouchFunc = 1; // touchF_hurt_touch = 1

    if (*self_).damage == 0 {
        (*self_).damage = 5;
    }

    (*self_).delay *= 1000.0;
    (*self_).wait *= 1000.0;

    (*self_).contents = 1; // CONTENTS_TRIGGER

    if !(*self_).targetname.is_null() {
        // NOTE: for some reason, this used to be: if((*self_).spawnflags&2)
        (*self_).e_UseFunc = 1; // useF_hurt_use = 1
    }

    // link in to the world if starting active
    if (*self_).spawnflags & 1 == 0 {
        gi.linkentity(self_);
    } else {
        // triggers automatically get linked into the world by SetBrushModel, so we have to unlink it here
        gi.unlinkentity(self_);
    }
}

const INITIAL_SUFFOCATION_DELAY: c_int = 5000; // 5 seconds

pub unsafe extern "C" fn space_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    if other.is_null() || (*other).inuse == 0 || (*other).client.is_null() {
        // NOTE: we need vehicles to know this, too...
        // || (*other).s.number >= MAX_CLIENTS)
        return;
    }

    if (*other).s.m_iVehicleNum != 0 && (*other).s.m_iVehicleNum <= 64 {
        // MAX_CLIENTS = 64
        // a player client inside a vehicle
        let veh: *mut gentity_t = core::ptr::addr_of_mut!(g_entities[(*other).s.m_iVehicleNum as usize]);

        if (*veh).inuse != 0 && !(*veh).client.is_null() && !(*veh).m_pVehicle.is_null() {
            // if they are "inside" a vehicle, then let that protect them from THE HORRORS OF SPACE.
            // Placeholder for vehicle info check
            return;
        }
    }

    if G_PointInBounds(
        core::ptr::addr_of!((*(*other).client).ps.origin) as *const [f32; 3],
        core::ptr::addr_of!((*self_).absmin),
        core::ptr::addr_of!((*self_).absmax),
    ) == 0
    {
        // his origin must be inside the trigger
        return;
    }

    if (*(*other).client).inSpaceIndex == 0 || (*(*other).client).inSpaceIndex == -1 {
        // ENTITYNUM_NONE = -1
        // freshly entering space
        (*(*other).client).inSpaceSuffocation = level.time + INITIAL_SUFFOCATION_DELAY;
    }

    (*(*other).client).inSpaceIndex = (*self_).s.number;
}

/*QUAKED trigger_space (.5 .5 .5) ?
causes human clients to suffocate and have no gravity.

*/
pub unsafe extern "C" fn SP_trigger_space(self_: *mut gentity_t) {
    InitTrigger(self_);
    (*self_).contents = 1; // CONTENTS_TRIGGER

    // FIXME: implement!!!
    // (*self_).e_TouchFunc = touchF_space_touch;

    gi.linkentity(self_);
}

pub unsafe extern "C" fn shipboundary_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    let mut ent: *mut gentity_t;

    if other.is_null()
        || (*other).inuse == 0
        || (*other).client.is_null()
        || (*other).s.number < 64
        || (*other).m_pVehicle.is_null()
    {
        // MAX_CLIENTS = 64, only let vehicles touch
        return;
    }

    ent = G_Find(core::ptr::null_mut(), 0, (*self_).target);
    if ent.is_null() || (*ent).inuse == 0 {
        // this is bad
        G_Error(
            b"trigger_shipboundary has invalid target \'%s\'\n\0".as_ptr() as *const c_char,
            (*self_).target,
        );
        return;
    }

    if (*other).s.m_iVehicleNum == 0 {
        // Placeholder for m_iRemovedSurfaces check
        // if a vehicle touches a boundary without a pilot in it or with parts missing, just blow the thing up
        G_Damage(
            other,
            other,
            other,
            core::ptr::null(),
            core::ptr::addr_of!((*(*other).client).ps.origin) as *const [f32; 3],
            99999,
            1, // DAMAGE_NO_PROTECTION
            21, // MOD_SUICIDE = 21
        );
        return;
    }

    (*(*other).client).ps.vehTurnaroundIndex = (*ent).s.number;
    (*(*other).client).ps.vehTurnaroundTime = level.time + (*self_).count;
}

/*QUAKED trigger_shipboundary (.5 .5 .5) ?
causes vehicle to turn toward target and travel in that direction for a set time when hit.

"target"		name of entity to turn toward (can be info_notnull, or whatever).
"traveltime"	time to travel in this direction

*/
pub unsafe extern "C" fn SP_trigger_shipboundary(self_: *mut gentity_t) {
    InitTrigger(self_);
    (*self_).contents = 1; // CONTENTS_TRIGGER

    if (*self_).target.is_null() || *(*self_).target as u8 == 0 {
        G_Error(b"trigger_shipboundary without a target.\0".as_ptr() as *const c_char);
    }
    G_SpawnInt(
        b"traveltime\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*self_).count),
    );

    if (*self_).count == 0 {
        G_Error(b"trigger_shipboundary without traveltime.\0".as_ptr() as *const c_char);
    }

    // FIXME: implement!
    // (*self_).e_TouchFunc = touchF_shipboundary_touch;

    gi.linkentity(self_);
}

/*
==============================================================================

timer

==============================================================================
*/

/*QUAKED func_timer (0.3 0.1 0.6) (-8 -8 -8) (8 8 8) START_ON
This should be renamed trigger_timer...
Repeatedly fires its targets.
Can be turned on or off by using.

"wait"			base time between triggering all targets, default is 1
"random"		wait variance, default is 0
so, the basic time between firing is a random time between
(wait - random) and (wait + random)

*/
pub unsafe extern "C" fn func_timer_think(self_: *mut gentity_t) {
    G_UseTargets(self_, (*self_).activator);
    // set time before next firing
    (*self_).nextthink = level.time + (1000.0 * ((*self_).wait + crandom() * (*self_).random)) as c_int;
}

pub unsafe extern "C" fn func_timer_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    (*self_).activator = activator;

    G_ActivateBehavior(self_, 32); // BSET_USE = 32

    // if on, turn it off
    if (*self_).nextthink != 0 {
        (*self_).nextthink = 0;
        return;
    }

    // turn it on
    func_timer_think(self_);
}

pub unsafe extern "C" fn SP_func_timer(self_: *mut gentity_t) {
    G_SpawnFloat(
        b"random\0".as_ptr() as *const c_char,
        b"1\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*self_).random),
    );
    G_SpawnFloat(
        b"wait\0".as_ptr() as *const c_char,
        b"1\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*self_).wait),
    );

    (*self_).e_UseFunc = 1; // useF_func_timer_use = 1
    (*self_).e_ThinkFunc = 9; // thinkF_func_timer_think = 9

    if (*self_).random >= (*self_).wait {
        (*self_).random = (*self_).wait - 1.0; // NOTE: was - FRAMETIME, but FRAMETIME is in msec (100) and these numbers are in *seconds*!
        gi.Printf(
            b"func_timer at %s has random >= wait\n\0".as_ptr() as *const c_char,
            vtos(core::ptr::addr_of!((*self_).s.origin)),
        );
    }

    if (*self_).spawnflags & 1 != 0 {
        (*self_).nextthink = level.time + 100; // FRAMETIME
        (*self_).activator = self_;
    }

    (*self_).svFlags = 16; // SVF_NOCLIENT
}

/*
==============================================================================

timer

==============================================================================
*/

/*QUAKED trigger_entdist (.1 .5 .1) (-8 -8 -8) (8 8 8) PLAYER NPC
fires if the given entity is within the given distance.  Sets itself inactive after one use.
----- KEYS -----
distance - radius entity can be away to fire trigger
target - fired if entity is within distance
target2 - fired if entity not within distance

NPC_target - NPC_types to look for
ownername - If any, who to calc the distance from- default is the trigger_entdist himself
example: target "biessman telsia" will look for the biessman and telsia NPC
if it finds either of these within distance it will fire.

  todo -
  add delay, count
  add monster classnames?????
  add LOS to it???
*/

pub unsafe extern "C" fn trigger_entdist_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut diff: [f32; 3] = [0.0; 3];
    let mut found: *mut gentity_t = core::ptr::null_mut();
    let mut owner: *mut gentity_t = core::ptr::null_mut();
    let mut useflag: c_int;
    let mut token: *const c_char;
    let mut holdString: *const c_char;

    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        // Don't use INACTIVE
        return;
    }

    G_ActivateBehavior(self_, 32); // BSET_USE = 32

    if !(*self_).ownername.is_null() && *(*self_).ownername as u8 != 0 {
        owner = G_Find(core::ptr::null_mut(), 0, (*self_).ownername);
    }

    if owner.is_null() {
        owner = self_;
    }

    (*self_).activator = activator;

    useflag = 0; // qfalse

    (*self_).svFlags |= 8; // SVF_INACTIVE
    // Make it inactive after one use

    if (*self_).spawnflags & ENTDIST_PLAYER != 0 {
        // Look for player???
        found = core::ptr::addr_of_mut!(g_entities[0]);

        if !found.is_null() {
            VectorSubtract(
                core::ptr::addr_of!((*owner).currentOrigin),
                core::ptr::addr_of!((*found).currentOrigin),
                core::ptr::addr_of_mut!(diff),
            );
            if VectorLength(core::ptr::addr_of!(diff)) < (*self_).count as f32 {
                useflag = 1; // qtrue
            }
        }
    }

    if ((*self_).spawnflags & ENTDIST_NPC != 0) && (useflag == 0) {
        holdString = (*self_).NPC_target;

        loop {
            if holdString.is_null() {
                break;
            }
            token = COM_Parse(core::ptr::addr_of_mut!(holdString));
            if token.is_null() || *token as u8 == 0 {
                // Nothing left to look at
                break;
            }

            found = G_Find(found, 0, token); // Look for the specified NPC
            if !found.is_null() {
                // Found???
                VectorSubtract(
                    core::ptr::addr_of!((*owner).currentOrigin),
                    core::ptr::addr_of!((*found).currentOrigin),
                    core::ptr::addr_of_mut!(diff),
                );
                if VectorLength(core::ptr::addr_of!(diff)) < (*self_).count as f32 {
                    // Within distance
                    useflag = 1; // qtrue
                    break;
                }
            }
        }
    }

    if useflag != 0 {
        G_UseTargets2(self_, (*self_).activator, (*self_).target);
    } else if !(*self_).target2.is_null() {
        // This is the negative target
        G_UseTargets2(self_, (*self_).activator, (*self_).target2);
    }
}

pub unsafe extern "C" fn SP_trigger_entdist(self_: *mut gentity_t) {
    G_SpawnInt(
        b"distance\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!((*self_).count),
    );

    (*self_).e_UseFunc = 1; // useF_trigger_entdist_use = 1
}

// spawnflag
const TRIGGERVISIBLE_FORCESIGHT: c_int = 2;

pub unsafe extern "C" fn trigger_visible_check_player_visibility(self_: *mut gentity_t) {
    // Check every FRAMETIME*2
    (*self_).nextthink = level.time + 100 * 2; // FRAMETIME*2

    if (*self_).svFlags & 8 != 0 {
        // SVF_INACTIVE
        return;
    }

    let mut dir: [f32; 3] = [0.0; 3];
    let mut dist: f32;
    let mut player: *mut gentity_t = core::ptr::addr_of_mut!(g_entities[0]);

    if player.is_null() || (*player).client.is_null() {
        return;
    }

    // Added 01/20/03 by AReis
    // If this trigger can only be used if the players force sight is on...
    if (*self_).spawnflags & TRIGGERVISIBLE_FORCESIGHT != 0 {
        // If their force sight is not on, leave...
        if ((*(*player).client).ps.forcePowersActive & (1 << 5)) == 0 {
            // FP_SEE = 5
            return;
        }
    }

    // 1: see if player is within 512*512 range
    VectorSubtract(
        core::ptr::addr_of!((*self_).currentOrigin),
        core::ptr::addr_of!((*(*player).client).renderInfo.eyePoint),
        core::ptr::addr_of_mut!(dir),
    );
    dist = VectorNormalize(core::ptr::addr_of_mut!(dir));
    if dist < (*self_).radius {
        // Within range
        let mut forward: [f32; 3] = [0.0; 3];
        let mut dot: f32;
        // 2: see if dot to us and player viewangles is > 0.7
        AngleVectors(
            core::ptr::addr_of!((*(*player).client).renderInfo.eyeAngles),
            core::ptr::addr_of_mut!(forward),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        dot = DotProduct(core::ptr::addr_of!(forward), core::ptr::addr_of!(dir));
        if dot > (*self_).random {
            // Within the desired FOV
            // 3: see if player is in PVS
            if gi.inPVS(
                core::ptr::addr_of!((*self_).currentOrigin),
                core::ptr::addr_of!((*(*player).client).renderInfo.eyePoint),
            ) != 0
            {
                let mut mins: [f32; 3] = [-1.0, -1.0, -1.0];
                let mut maxs: [f32; 3] = [1.0, 1.0, 1.0];
                // 4: If needbe, trace to see if there is clear LOS from player viewpos
                if ((*self_).spawnflags & 1 != 0)
                    || (G_ClearTrace(
                        core::ptr::addr_of!((*(*player).client).renderInfo.eyePoint),
                        core::ptr::addr_of!(mins),
                        core::ptr::addr_of!(maxs),
                        core::ptr::addr_of!((*self_).currentOrigin),
                        0,
                        1024, // MASK_OPAQUE
                    ) != 0)
                {
                    // 5: Fire!
                    G_UseTargets(self_, player);
                    // 6: Remove yourself
                    G_FreeEntity(self_);
                }
            }
        }
    }
}

/*QUAKED trigger_visible (.1 .5 .1) (-8 -8 -8) (8 8 8) NOTRACE FORCESIGHT x x x x x INACTIVE

  Only fires when player is looking at it, fires only once then removes itself.

  NOTRACE - Doesn't check to make sure the line of sight is completely clear (penetrates walls, forcefields, etc)
  FORCESIGHT - Only activates this trigger if force sight is on.
  INACTIVE - won't check for player visibility until activated

  radius - how far this ent can be from player's eyes, max, and still be considered "seen"
  FOV - how far off to the side of the player's field of view this can be, max, and still be considered "seen".  Player FOV is 80, so the default for this value is 30.

  "target" - What to use when it fires.
*/
pub unsafe extern "C" fn SP_trigger_visible(self_: *mut gentity_t) {
    if (*self_).radius <= 0.0 {
        (*self_).radius = 512.0;
    }

    if (*self_).random <= 0.0 {
        // about 30 degrees
        (*self_).random = 0.7;
    } else {
        // convert from FOV degrees to number meaningful for dot products
        (*self_).random = 1.0 - ((*self_).random / 90.0);
    }

    if (*self_).spawnflags & 128 != 0 {
        // Make it inactive
        (*self_).svFlags |= 8; // SVF_INACTIVE
    }

    G_SetOrigin(self_, core::ptr::addr_of!((*self_).s.origin));
    gi.linkentity(self_);

    (*self_).e_ThinkFunc = 10; // thinkF_trigger_visible_check_player_visibility = 10
    (*self_).nextthink = level.time + 100 * 2; // FRAMETIME*2
}
