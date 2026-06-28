// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_float, c_void};
use std::ffi::CStr;

// extern declarations from b_local.h and game engine
extern "C" {
    pub static mut NPC: *mut gentity_t;
    pub static mut player: *mut gentity_t;
    pub static mut level: level_t;
    pub static mut ucmd: usercmd_t;
    pub static mut gi: gameImport_t;
    pub static mut g_spskill: cvar_t;

    pub fn G_Knockdown(
        self_: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: c_float,
        breakSaberLock: c_int,
    );

    pub fn G_EffectIndex(str: *const core::ffi::c_char) -> c_int;
    pub fn G_SoundIndex(str: *const core::ffi::c_char) -> c_int;
    pub fn va(fmt: *const core::ffi::c_char, ...) -> *const core::ffi::c_char;
    pub fn TIMER_Set(ent: *mut gentity_t, timer: *const core::ffi::c_char, duration: c_int);
    pub fn TIMER_Done(ent: *mut gentity_t, timer: *const core::ffi::c_char) -> c_int;
    pub fn NPC_SetAnim(
        ent: *mut gentity_t,
        type_: c_int,
        anim: c_int,
        flags: c_int,
    );
    pub fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int);
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn Distance(a: *const vec3_t, b: *const vec3_t) -> c_float;
    pub fn CGCam_Shake(intensity: c_float, duration: c_int);
    pub fn VectorCopy(src: *const vec3_t, dest: *mut vec3_t);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(vec: *mut vec3_t) -> c_float;
    pub fn VectorMA(
        veca: *const vec3_t,
        scale: c_float,
        vecb: *const vec3_t,
        vecc: *mut vec3_t,
    );
    pub fn DistanceHorizontal(a: *const vec3_t, b: *const vec3_t) -> c_float;
    pub fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> c_float;
    pub fn VectorLengthSquared(vec: *const vec3_t) -> c_float;
    pub fn VectorClear(vec: *mut vec3_t);
    pub fn G_PlayEffect(index: c_int, org: *const vec3_t, up: *const vec3_t);
    pub fn G_Throw(ent: *mut gentity_t, dir: *const vec3_t, strength: c_float);
    pub fn NPC_MoveToGoal(retreat: c_int) -> c_int;
    pub fn NPC_SetMoveGoal(
        ent: *mut gentity_t,
        point: *const vec3_t,
        radius: c_float,
        update: c_int,
    );
    pub fn NPC_CheckAlertEvents(
        checkSight: c_int,
        checkSound: c_int,
        lastAlertID: c_int,
        checkTouchEvents: c_int,
        alertLevel: c_int,
        ignorePlayer: c_int,
    ) -> c_int;
    pub fn vectoangles(vec: *const vec3_t, angles: *mut vec3_t);
    pub fn AngleNormalize180(angle: c_float) -> c_float;
    pub fn SetClientViewAngle(ent: *mut gentity_t, angles: *const vec3_t);
    pub fn TossClientItems(ent: *mut gentity_t);
    pub fn NPC_UpdateAngles(doPitch: c_int, doYaw: c_int);
    pub fn NPC_ReachedGoal();
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundname: *const core::ffi::c_char);
    pub fn GEntity_DieFunc(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        damage: c_int,
        meansOfDeath: c_int,
        d_flags: c_int,
        hitLoc: c_int,
    );
    pub fn G_FreeEntity(ent: *mut gentity_t);
    pub fn VectorCompare(v1: *const vec3_t, v2: *const vec3_t) -> c_int;
    pub fn Q_flrand(min: c_float, max: c_float) -> c_float;
}

// Local stubs for engine types - these should match their actual definitions from g_headers.h/b_local.h
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub inuse: c_int,
    pub linkcount: c_int,
    pub activator: *mut gentity_t,
    pub client: *mut gclient_t,
    pub NPC: *mut gNPC_t,
    pub enemy: *mut gentity_t,
    pub currentOrigin: [c_float; 3],
    pub s_origin: [c_float; 3],
    pub absmin: [c_float; 3],
    pub maxs: [c_float; 3],
    pub mins: [c_float; 3],
    pub pos1: [c_float; 3],
    pub pos2: [c_float; 3],
    pub clipmask: c_int,
    pub contents: c_int,
    pub svFlags: c_int,
    pub flags: c_int,
    pub radius: c_float,
    pub health: c_int,
    pub takedamage: c_int,
    // ... other fields
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub origin: [c_float; 3],
    pub trDelta: [c_float; 3],
    pub loopSound: c_int,
    pub weapon: c_int,
    // ... other fields
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub NPC_class: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub legsAnim: c_int,
    pub legsAnimTimer: c_int,
    pub viewangles: [c_float; 3],
    pub velocity: [c_float; 3],
    pub moveDir: [c_float; 3],
    pub speed: c_float,
    pub groundEntityNum: c_int,
    pub eFlags: c_int,
    pub lastStationary: c_int,
}

#[repr(C)]
pub struct gNPC_t {
    pub goalEntity: *mut gentity_t,
    pub nextBStateThink: c_int,
    pub enemyLastSeenTime: c_int,
    pub enemyLastSeenLocation: [c_float; 3],
    pub lastAlertID: c_int,
    pub scriptFlags: c_int,
    pub stats: npcStats_t,
}

#[repr(C)]
pub struct npcStats_t {
    pub earshot: c_float,
    pub walkSpeed: c_float,
    pub runSpeed: c_float,
}

#[repr(C)]
pub struct usercmd_t {
    pub buttons: c_int,
    pub forwardmove: c_int,
    pub rightmove: c_int,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: c_float,
    pub endpos: [c_float; 3],
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: [c_float; 3],
    pub dist: c_float,
    pub type_: c_int,
    pub signbits: c_int,
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
    pub alertEvents: [alertEvent_t; 256],
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    pub value: c_float,
}

#[repr(C)]
pub struct alertEvent_t {
    pub position: [c_float; 3],
    pub ID: c_int,
}

pub type TraceFn = extern "C" fn(
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    passent: c_int,
    contentmask: c_int,
);

pub type EntitiesInBoxFn = extern "C" fn(
    mins: *const vec3_t,
    maxs: *const vec3_t,
    list: *mut *mut gentity_t,
    maxcount: c_int,
) -> c_int;

#[repr(C)]
pub struct gameImport_t {
    pub trace: TraceFn,
    pub EntitiesInBox: EntitiesInBoxFn,
}

pub type vec3_t = [c_float; 3];

// Constants
const MIN_ATTACK_DIST_SQ: c_int = 128;
const MIN_MISS_DIST: c_int = 100;
const MIN_MISS_DIST_SQ: c_int = MIN_MISS_DIST * MIN_MISS_DIST;
const MAX_MISS_DIST: c_int = 500;
const MAX_MISS_DIST_SQ: c_int = MAX_MISS_DIST * MAX_MISS_DIST;
const MIN_SCORE: c_float = -37500.0; // speed of (50*50) - dist of (200*200)

// Engine constants (stubs)
const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

// Placeholder constants from game engine
const BOTH_ATTACK1: c_int = 0;
const BOTH_ATTACK2: c_int = 1;
const BOTH_WALK2: c_int = 2;
const BOTH_SWIM_IDLE1: c_int = 3;
const BOTH_FALLDEATH1: c_int = 4;

const SETANIM_LEGS: c_int = 0;
const SETANIM_TORSO: c_int = 1;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;
const SETANIM_FLAG_RESTART: c_int = 4;

const ENTITYNUM_NONE: c_int = 1024;
const ENTITYNUM_WORLD: c_int = 0;

const CONTENTS_SOLID: c_int = 1;
const CONTENTS_BODY: c_int = 32;
const CONTENTS_BOTCLIP: c_int = 0x4000;
const CONTENTS_MONSTERCLIP: c_int = 0x2000;

const MASK_NPCSOLID: c_int = CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_MONSTERCLIP;

const EV_PAIN: c_int = 5;
const EV_DEATH1: c_int = 10;
const EV_DEATH3: c_int = 12;
const EV_MISSILE: c_int = 4;

const EF_HELD_BY_RANCOR: c_int = 0x00000400;
const EF_HELD_BY_WAMPA: c_int = 0x00000800;
const EF_HELD_BY_SAND_CREATURE: c_int = 0x04000000;
const EF_NODRAW: c_int = 0x00000080;

const FL_NOTARGET: c_int = 0x00000040;

const BUTTON_WALKING: c_int = 1;

const SVF_LOCKEDENEMY: c_int = 0x00000800;

const SCF_IGNORE_ALERTS: c_int = 0x00000020;

const PITCH: c_int = 0;
const YAW: c_int = 1;

const CLASS_SAND_CREATURE: c_int = 18;
const CLASS_RANCOR: c_int = 13;
const CLASS_ATST: c_int = 19;

const ET_MISSILE: c_int = 2;
const WP_THERMAL: c_int = 14;

const CHAN_VOICE: c_int = 2;

const MAX_CLIENTS: c_int = 64;

const Q3_INFINITE: c_float = 1e10;

const AEL_MINOR: c_int = 0;

const MOD_MELEE: c_int = 3;
const HL_NONE: c_int = 0;

const MIN_WALK_NORMAL: c_float = 0.7071;

pub fn SandCreature_Precache() {
    let mut i: c_int;
    unsafe {
        G_EffectIndex(b"env/sand_dive\0".as_ptr() as *const core::ffi::c_char);
        G_EffectIndex(b"env/sand_spray\0".as_ptr() as *const core::ffi::c_char);
        G_EffectIndex(b"env/sand_move\0".as_ptr() as *const core::ffi::c_char);
        G_EffectIndex(b"env/sand_move_breach\0".as_ptr() as *const core::ffi::c_char);
        // G_EffectIndex( "env/sand_attack_breach" );
        i = 1;
        while i < 4 {
            G_SoundIndex(va(b"sound/chars/sand_creature/voice%d.mp3\0".as_ptr() as *const core::ffi::c_char, i));
            i += 1;
        }
        G_SoundIndex(b"sound/chars/sand_creature/slither.wav\0".as_ptr() as *const core::ffi::c_char);
    }
}

pub fn SandCreature_ClearTimers(ent: *mut gentity_t) {
    unsafe {
        TIMER_Set(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char, -level.time);
        TIMER_Set(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char, -level.time);
        TIMER_Set(NPC, b"breachDebounce\0".as_ptr() as *const core::ffi::c_char, -level.time);
        TIMER_Set(NPC, b"pain\0".as_ptr() as *const core::ffi::c_char, -level.time);
        TIMER_Set(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char, -level.time);
        TIMER_Set(NPC, b"missDebounce\0".as_ptr() as *const core::ffi::c_char, -level.time);
    }
}

pub fn NPC_SandCreature_Die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    // FIXME: somehow make him solid when he dies?
}

pub fn NPC_SandCreature_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    unsafe {
        if TIMER_Done(self_, b"pain\0".as_ptr() as *const core::ffi::c_char) != 0 {
            // FIXME: effect and sound
            // FIXME: shootable during this anim?
            NPC_SetAnim(
                self_,
                SETANIM_LEGS,
                Q_irand(BOTH_ATTACK1, BOTH_ATTACK2),
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
            );
            G_AddEvent(self_, EV_PAIN, Q_irand(0, 100));
            TIMER_Set(
                self_,
                b"pain\0".as_ptr() as *const core::ffi::c_char,
                (*(*self_).client).ps.legsAnimTimer + Q_irand(500, 2000),
            );
            let playerDist: c_float = Distance((*player).currentOrigin.as_ptr(), (*self_).currentOrigin.as_ptr());
            if playerDist < 256.0 {
                CGCam_Shake(1.0 * playerDist / 128.0, (*(*self_).client).ps.legsAnimTimer);
            }
        }
        (*self_).enemy = std::ptr::null_mut();
        (*(*self_).NPC).goalEntity = std::ptr::null_mut();
    }
}

pub fn SandCreature_MoveEffect() {
    unsafe {
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut org: vec3_t = [(*NPC).currentOrigin[0], (*NPC).currentOrigin[1], (*NPC).absmin[2] + 2.0];

        let playerDist: c_float = Distance((*player).currentOrigin.as_ptr(), (*NPC).currentOrigin.as_ptr());
        if playerDist < 256.0 {
            CGCam_Shake(0.75 * playerDist / 256.0, 250);
        }

        if level.time - (*(*NPC).client).ps.lastStationary > 2000 {
            // first time moving for at least 2 seconds
            // clear speakingtime
            TIMER_Set(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char, -level.time);
        }

        if TIMER_Done(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char) != 0
            && TIMER_Done(NPC, b"breachDebounce\0".as_ptr() as *const core::ffi::c_char) != 0
            && TIMER_Done(NPC, b"pain\0".as_ptr() as *const core::ffi::c_char) != 0
            && TIMER_Done(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) != 0
            && Q_irand(0, 10) == 0
        {
            // Breach!
            // FIXME: only do this while moving forward?
            let mut trace: trace_t = std::mem::zeroed();
            // make him solid here so he can be hit/gets blocked on stuff. Check clear first.
            (gi.trace)(
                &mut trace,
                (*NPC).currentOrigin.as_ptr(),
                (*NPC).mins.as_ptr(),
                (*NPC).maxs.as_ptr(),
                (*NPC).currentOrigin.as_ptr(),
                (*NPC).s.number,
                MASK_NPCSOLID,
            );
            if trace.allsolid == 0 && trace.startsolid == 0 {
                (*NPC).clipmask = MASK_NPCSOLID; // turn solid for a little bit
                (*NPC).contents = CONTENTS_BODY;
                // NPC->takedamage = qtrue;//can be shot?

                // FIXME: Breach sound?
                // FIXME: Breach effect?
                NPC_SetAnim(
                    NPC,
                    SETANIM_LEGS,
                    BOTH_WALK2,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                );
                TIMER_Set(
                    NPC,
                    b"breaching\0".as_ptr() as *const core::ffi::c_char,
                    (*(*NPC).client).ps.legsAnimTimer,
                );
                TIMER_Set(
                    NPC,
                    b"breachDebounce\0".as_ptr() as *const core::ffi::c_char,
                    (*(*NPC).client).ps.legsAnimTimer + Q_irand(0, 10000),
                );
            }
        }
        if TIMER_Done(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char) == 0 {
            // different effect when breaching
            // FIXME: make effect
            G_PlayEffect(G_EffectIndex(b"env/sand_move_breach\0".as_ptr() as *const core::ffi::c_char), org.as_ptr(), up.as_ptr());
        } else {
            G_PlayEffect(G_EffectIndex(b"env/sand_move\0".as_ptr() as *const core::ffi::c_char), org.as_ptr(), up.as_ptr());
        }
        (*NPC).s.loopSound = G_SoundIndex(b"sound/chars/sand_creature/slither.wav\0".as_ptr() as *const core::ffi::c_char);
    }
}

pub fn SandCreature_CheckAhead(end: *mut vec3_t) -> c_int {
    unsafe {
        let mut trace: trace_t = std::mem::zeroed();
        let clipmask: c_int = (*NPC).clipmask | CONTENTS_BOTCLIP;

        // make sure our goal isn't underground (else the trace will fail)
        let mut bottom: vec3_t = [(*end)[0], (*end)[1], (*end)[2] + (*NPC).mins[2]];
        (gi.trace)(
            &mut trace,
            end as *const vec3_t,
            vec3_origin.as_ptr(),
            vec3_origin.as_ptr(),
            bottom.as_ptr() as *const vec3_t,
            (*NPC).s.number,
            (*NPC).clipmask,
        );
        if trace.fraction < 1.0 {
            // in the ground, raise it up
            (*end)[2] -= (*NPC).mins[2] * (1.0 - trace.fraction) - 0.125;
        }

        (gi.trace)(
            &mut trace,
            (*NPC).currentOrigin.as_ptr(),
            (*NPC).mins.as_ptr(),
            (*NPC).maxs.as_ptr(),
            end as *const vec3_t,
            (*NPC).s.number,
            clipmask,
        );

        if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
            // started inside do not enter, so ignore them
            let new_clipmask = clipmask & !CONTENTS_BOTCLIP;
            (gi.trace)(
                &mut trace,
                (*NPC).currentOrigin.as_ptr(),
                (*NPC).mins.as_ptr(),
                (*NPC).maxs.as_ptr(),
                end as *const vec3_t,
                (*NPC).s.number,
                new_clipmask,
            );
        }
        // Do a simple check
        if (trace.allsolid == 0) && (trace.startsolid == 0) && (trace.fraction == 1.0) {
            return 1;
        }

        if trace.plane.normal[2] >= MIN_WALK_NORMAL {
            return 1;
        }

        // This is a work around
        let radius: c_float = if (*NPC).maxs[0] > (*NPC).maxs[1] {
            (*NPC).maxs[0]
        } else {
            (*NPC).maxs[1]
        };
        let dist: c_float = Distance((*NPC).currentOrigin.as_ptr(), end as *const vec3_t);
        let tFrac: c_float = 1.0 - (radius / dist);

        if trace.fraction >= tFrac {
            return 1;
        }

        return 0;
    }
}

pub fn SandCreature_Move() -> c_int {
    unsafe {
        let mut moved: c_int = 0;
        // FIXME should ignore doors..?
        let mut dest: vec3_t = std::mem::zeroed();
        VectorCopy((*(*(*NPC).NPC).goalEntity).currentOrigin.as_ptr(), dest.as_mut_ptr());
        // Sand Creatures look silly using waypoints when they can go straight to the goal
        if SandCreature_CheckAhead(dest.as_mut_ptr()) != 0 {
            // use our temp move straight to goal check
            VectorSubtract(
                dest.as_ptr(),
                (*NPC).currentOrigin.as_ptr(),
                (*(*NPC).client).ps.moveDir.as_mut_ptr(),
            );
            (*(*NPC).client).ps.speed = VectorNormalize((*(*NPC).client).ps.moveDir.as_mut_ptr());
            if (ucmd.buttons & BUTTON_WALKING) != 0 && (*(*NPC).client).ps.speed > (*(*NPC).NPC).stats.walkSpeed {
                (*(*NPC).client).ps.speed = (*(*NPC).NPC).stats.walkSpeed;
            } else {
                if (*(*NPC).client).ps.speed < (*(*NPC).NPC).stats.walkSpeed {
                    (*(*NPC).client).ps.speed = (*(*NPC).NPC).stats.walkSpeed;
                }
                if (ucmd.buttons & BUTTON_WALKING) == 0 && (*(*NPC).client).ps.speed < (*(*NPC).NPC).stats.runSpeed {
                    (*(*NPC).client).ps.speed = (*(*NPC).NPC).stats.runSpeed;
                } else if (*(*NPC).client).ps.speed > (*(*NPC).NPC).stats.runSpeed {
                    (*(*NPC).client).ps.speed = (*(*NPC).NPC).stats.runSpeed;
                }
            }
            moved = 1;
        } else {
            moved = NPC_MoveToGoal(1);
        }
        if moved != 0 && (*NPC).radius != 0.0 {
            let mut newPos: vec3_t = std::mem::zeroed();
            let curTurfRange: c_float;
            let newTurfRange: c_float;
            curTurfRange = DistanceHorizontal((*NPC).currentOrigin.as_ptr(), (*NPC).s_origin.as_ptr());
            VectorMA(
                (*NPC).currentOrigin.as_ptr(),
                (*(*NPC).client).ps.speed / 100.0,
                (*(*NPC).client).ps.moveDir.as_ptr(),
                newPos.as_mut_ptr(),
            );
            newTurfRange = DistanceHorizontal(newPos.as_ptr(), (*NPC).s_origin.as_ptr());
            if newTurfRange > (*NPC).radius && newTurfRange > curTurfRange {
                // would leave our range
                // stop
                (*(*NPC).client).ps.speed = 0.0;
                VectorClear((*(*NPC).client).ps.moveDir.as_mut_ptr());
                ucmd.forwardmove = 0;
                ucmd.rightmove = 0;
                moved = 0;
            }
        }
        return moved;
        // often erroneously returns false ???  something wrong with NAV...?
    }
}

pub fn SandCreature_Attack(miss: c_int) {
    unsafe {
        // FIXME: make it able to grab a thermal detonator, take it down,
        //         then have it explode inside them, killing them
        //         (or, do damage, making them stick half out of the ground and
        //         screech for a bit, giving you a chance to run for it!)

        // FIXME: effect and sound
        // FIXME: shootable during this anim?
        if (*(*NPC).enemy).client == std::ptr::null_mut() {
            NPC_SetAnim(
                NPC,
                SETANIM_LEGS,
                BOTH_ATTACK1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
            );
        } else {
            NPC_SetAnim(
                NPC,
                SETANIM_LEGS,
                Q_irand(BOTH_ATTACK1, BOTH_ATTACK2),
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
            );
        }
        // don't do anything else while in this anim
        TIMER_Set(
            NPC,
            b"attacking\0".as_ptr() as *const core::ffi::c_char,
            (*(*NPC).client).ps.legsAnimTimer,
        );
        let playerDist: c_float = Distance((*player).currentOrigin.as_ptr(), (*NPC).currentOrigin.as_ptr());
        if playerDist < 256.0 {
            // FIXME: tone this down
            CGCam_Shake(0.75 * playerDist / 128.0, (*(*NPC).client).ps.legsAnimTimer);
        }

        if miss != 0 {
            // purposely missed him, chance of knocking him down
            // FIXME: if, during the attack anim, I do end up catching him close to my mouth, then snatch him anyway...
            if (*NPC).enemy != std::ptr::null_mut() && (*(*NPC).enemy).client != std::ptr::null_mut() {
                let mut dir2Enemy: vec3_t = std::mem::zeroed();
                VectorSubtract(
                    (*(*NPC).enemy).currentOrigin.as_ptr(),
                    (*NPC).currentOrigin.as_ptr(),
                    dir2Enemy.as_mut_ptr(),
                );
                if dir2Enemy[2] < 30.0 {
                    dir2Enemy[2] = 30.0;
                }
                if g_spskill.integer > 0 {
                    let enemyDist: c_float = VectorNormalize(dir2Enemy.as_mut_ptr());
                    // FIXME: tone this down, smaller radius
                    if enemyDist < 200.0 && (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE {
                        let mut throwStr: c_float = ((200.0 - enemyDist) * 0.4) + 20.0;
                        if throwStr > 45.0 {
                            throwStr = 45.0;
                        }
                        G_Throw((*NPC).enemy, dir2Enemy.as_ptr(), throwStr);
                        if g_spskill->integer > 1 {
                            // knock them down, too
                            if (*(*NPC).enemy).health > 0 && Q_flrand(50.0, 150.0) as c_int > enemyDist as c_int {
                                // knock them down
                                G_Knockdown((*NPC).enemy, NPC, dir2Enemy.as_ptr(), 300.0, 1);
                                if (*(*NPC).enemy).s.number < MAX_CLIENTS {
                                    // make the player look up at me
                                    let mut vAng: vec3_t = std::mem::zeroed();
                                    vectoangles(dir2Enemy.as_ptr(), vAng.as_mut_ptr());
                                    vAng[PITCH as usize] = AngleNormalize180(vAng[PITCH as usize]) * -1.0;
                                    vAng[1] = (*(*(*NPC).enemy).client).ps.viewangles[YAW as usize];
                                    vAng[2] = 0.0;
                                    SetClientViewAngle((*NPC).enemy, vAng.as_ptr());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            (*(*NPC).enemy).activator = NPC; // kind of dumb, but when we are locked to the Rancor, we are owned by it.
            (*NPC).activator = (*NPC).enemy; // remember him
            // this guy isn't going anywhere anymore
            (*(*NPC).enemy).contents = 0;
            (*(*NPC).enemy).clipmask = 0;

            if (*(*NPC).activator).client != std::ptr::null_mut() {
                (*(*(*NPC).activator).client).ps.SaberDeactivate();
                (*(*(*NPC).activator).client).ps.eFlags |= EF_HELD_BY_SAND_CREATURE;
                if (*(*NPC).activator).health > 0 && (*(*NPC).activator).client != std::ptr::null_mut() {
                    G_AddEvent((*NPC).activator, Q_irand(EV_DEATH1, EV_DEATH3), 0);
                    NPC_SetAnim((*NPC).activator, SETANIM_LEGS, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    NPC_SetAnim((*NPC).activator, SETANIM_TORSO, BOTH_FALLDEATH1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TossClientItems(NPC);
                    if (*(*NPC).activator).NPC != std::ptr::null_mut() {
                        // no more thinking for you
                        (*(*(*NPC).activator).NPC).nextBStateThink = (1e10) as c_int;
                    }
                }
                /*
                if ( !NPC->activator->s.number )
                {
                    cg.overrides.active |= (CG_OVERRIDE_3RD_PERSON_CDP|CG_OVERRIDE_3RD_PERSON_RNG);
                    cg.overrides.thirdPersonCameraDamp = 0;
                    cg.overrides.thirdPersonRange = 120;
                }
                */
            } else {
                (*(*NPC).activator).s.eFlags |= EF_HELD_BY_SAND_CREATURE;
            }
        }
    }
}

pub fn SandCreature_EntScore(ent: *mut gentity_t) -> c_float {
    unsafe {
        let moveSpeed: c_float;
        let dist: c_float;

        if (*ent).client != std::ptr::null_mut() {
            moveSpeed = VectorLengthSquared((*(*ent).client).ps.velocity.as_ptr());
        } else {
            moveSpeed = VectorLengthSquared((*ent).s.trDelta.as_ptr());
        }
        dist = DistanceSquared((*NPC).currentOrigin.as_ptr(), (*ent).currentOrigin.as_ptr());
        return moveSpeed - dist;
    }
}

pub fn SandCreature_SeekEnt(bestEnt: *mut gentity_t, score: c_float) {
    unsafe {
        (*(*NPC).NPC).enemyLastSeenTime = level.time;
        VectorCopy((*bestEnt).currentOrigin.as_ptr(), (*(*NPC).NPC).enemyLastSeenLocation.as_mut_ptr());
        NPC_SetMoveGoal(NPC, (*(*NPC).NPC).enemyLastSeenLocation.as_ptr(), 0.0, 0);
        if score > MIN_SCORE {
            (*NPC).enemy = bestEnt;
        }
    }
}

pub fn SandCreature_CheckMovingEnts() {
    unsafe {
        let mut radiusEnts: [*mut gentity_t; 128] = [std::ptr::null_mut(); 128];
        let radius: c_float = (*(*NPC).NPC).stats.earshot;
        let mut mins: vec3_t = std::mem::zeroed();
        let mut maxs: vec3_t = std::mem::zeroed();

        let mut i: c_int = 0;
        while i < 3 {
            mins[i as usize] = (*NPC).currentOrigin[i as usize] - radius;
            maxs[i as usize] = (*NPC).currentOrigin[i as usize] + radius;
            i += 1;
        }

        let numEnts: c_int = (gi.EntitiesInBox)(mins.as_ptr(), maxs.as_ptr(), radiusEnts.as_mut_ptr(), 128);
        let mut bestEnt: c_int = -1;
        let mut bestScore: c_float = 0.0;

        i = 0;
        while i < numEnts {
            if (*radiusEnts[i as usize]).inuse == 0 {
                i += 1;
                continue;
            }

            if radiusEnts[i as usize] == NPC {
                // Skip the rancor ent
                i += 1;
                continue;
            }

            if (*radiusEnts[i as usize]).client == std::ptr::null_mut() {
                // must be a client
                if (*radiusEnts[i as usize]).s.eType != ET_MISSILE || (*radiusEnts[i as usize]).s.weapon != WP_THERMAL {
                    // not a thermal detonator
                    i += 1;
                    continue;
                }
            } else {
                if ((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0 {
                    // can't be one being held
                    i += 1;
                    continue;
                }

                if ((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_WAMPA) != 0 {
                    // can't be one being held
                    i += 1;
                    continue;
                }

                if ((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_SAND_CREATURE) != 0 {
                    // can't be one being held
                    i += 1;
                    continue;
                }

                if ((*radiusEnts[i as usize]).s.eFlags & EF_NODRAW) != 0 {
                    // not if invisible
                    i += 1;
                    continue;
                }

                if (*(*radiusEnts[i as usize]).client).ps.groundEntityNum != ENTITYNUM_WORLD {
                    // not on the ground
                    i += 1;
                    continue;
                }

                if (*(*radiusEnts[i as usize]).client).NPC_class == CLASS_SAND_CREATURE {
                    i += 1;
                    continue;
                }
            }

            if ((*radiusEnts[i as usize]).flags & FL_NOTARGET) != 0 {
                i += 1;
                continue;
            }
            /*
            if ( radiusEnts[i]->client && (radiusEnts[i]->client->NPC_class == CLASS_RANCOR || radiusEnts[i]->client->NPC_class == CLASS_ATST ) )
            {//can't grab rancors or atst's
                continue;
            }
            */
            let checkScore: c_float = SandCreature_EntScore(radiusEnts[i as usize]);
            // FIXME: take mass into account too?  What else?
            if checkScore > bestScore {
                bestScore = checkScore;
                bestEnt = i;
            }

            i += 1;
        }
        if bestEnt != -1 {
            SandCreature_SeekEnt(radiusEnts[bestEnt as usize], bestScore);
        }
    }
}

pub fn SandCreature_SeekAlert(alertEvent: c_int) {
    unsafe {
        let alert: *mut alertEvent_t = &mut level.alertEvents[alertEvent as usize];

        // FIXME: check for higher alert status or closer than last location?
        (*(*NPC).NPC).enemyLastSeenTime = level.time;
        VectorCopy((*alert).position.as_ptr(), (*(*NPC).NPC).enemyLastSeenLocation.as_mut_ptr());
        NPC_SetMoveGoal(NPC, (*(*NPC).NPC).enemyLastSeenLocation.as_ptr(), 0.0, 0);
    }
}

pub fn SandCreature_CheckAlerts() {
    unsafe {
        if ((*(*NPC).NPC).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            let alertEvent: c_int = NPC_CheckAlertEvents(0, 1, (*(*NPC).NPC).lastAlertID, 0, AEL_MINOR, 1);

            // There is an event to look at
            if alertEvent >= 0 {
                // if ( level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
                {
                    SandCreature_SeekAlert(alertEvent);
                }
            }
        }
    }
}

pub fn SandCreature_DistSqToGoal(goalIsEnemy: c_int) -> c_float {
    unsafe {
        let goalDistSq: c_float;
        if (*(*NPC).NPC).goalEntity == std::ptr::null_mut() || goalIsEnemy != 0 {
            if (*NPC).enemy == std::ptr::null_mut() {
                return Q3_INFINITE;
            }
            (*(*NPC).NPC).goalEntity = (*NPC).enemy;
        }

        if (*(*(*NPC).NPC).goalEntity).client != std::ptr::null_mut() {
            goalDistSq = DistanceSquared((*NPC).currentOrigin.as_ptr(), (*(*(*NPC).NPC).goalEntity).currentOrigin.as_ptr());
        } else {
            let mut gOrg: vec3_t = std::mem::zeroed();
            VectorCopy((*(*(*NPC).NPC).goalEntity).currentOrigin.as_ptr(), gOrg.as_mut_ptr());
            gOrg[2] -= (*NPC).mins[2] - (*(*(*NPC).NPC).goalEntity).mins[2]; // moves the gOrg up/down to make it's origin seem at the proper height as if it had my mins
            goalDistSq = DistanceSquared((*NPC).currentOrigin.as_ptr(), gOrg.as_ptr());
        }
        return goalDistSq;
    }
}

pub fn SandCreature_Chase() {
    unsafe {
        if (*(*NPC).enemy).inuse == 0 {
            // freed
            (*NPC).enemy = std::ptr::null_mut();
            return;
        }

        if ((*NPC).svFlags & SVF_LOCKEDENEMY) != 0 {
            // always know where he is
            (*(*NPC).NPC).enemyLastSeenTime = level.time;
        }

        if ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
            if level.time - (*(*NPC).NPC).enemyLastSeenTime > 10000 {
                (*NPC).enemy = std::ptr::null_mut();
                return;
            }
        }

        if (*(*NPC).enemy).client != std::ptr::null_mut() {
            if ((*(*(*NPC).enemy).client).ps.eFlags & EF_HELD_BY_SAND_CREATURE) != 0
                || ((*(*(*NPC).enemy).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0
                || ((*(*(*NPC).enemy).client).ps.eFlags & EF_HELD_BY_WAMPA) != 0
            {
                // was picked up by another monster, forget about him
                (*NPC).enemy = std::ptr::null_mut();
                (*NPC).svFlags &= !SVF_LOCKEDENEMY;
                return;
            }
        }
        // chase the enemy
        if (*NPC).enemy != std::ptr::null_mut()
            && (*(*NPC).enemy).client != std::ptr::null_mut()
            && (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_WORLD
            && ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0
        {
            // off the ground!
            // FIXME: keep moving in the dir we were moving for a little bit...
        } else {
            let enemyScore: c_float = SandCreature_EntScore((*NPC).enemy);
            if enemyScore < MIN_SCORE && ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
                // too slow or too far away
            } else {
                let moveSpeed: c_float;
                if (*(*NPC).enemy).client != std::ptr::null_mut() {
                    moveSpeed = VectorLengthSquared((*(*(*NPC).enemy).client).ps.velocity.as_ptr());
                } else {
                    moveSpeed = VectorLengthSquared((*(*NPC).enemy).s.trDelta.as_ptr());
                }
                if moveSpeed != 0.0 {
                    // he's still moving, update my goalEntity's origin
                    SandCreature_SeekEnt((*NPC).enemy, 0.0);
                    (*(*NPC).NPC).enemyLastSeenTime = level.time;
                }
            }
        }

        if (level.time - (*(*NPC).NPC).enemyLastSeenTime) > 5000 && ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
            // enemy hasn't moved in about 5 seconds, see if there's anything else of interest
            SandCreature_CheckAlerts();
            SandCreature_CheckMovingEnts();
        }

        let enemyDistSq: c_float = SandCreature_DistSqToGoal(1);

        // FIXME: keeps chasing goalEntity even when it's already reached it...?
        if enemyDistSq >= MIN_ATTACK_DIST_SQ as c_float
            && (level.time - (*(*NPC).NPC).enemyLastSeenTime) <= 3000
        {
            // sensed enemy (or something) less than 3 seconds ago
            ucmd.buttons &= !BUTTON_WALKING;
            if SandCreature_Move() != 0 {
                SandCreature_MoveEffect();
            }
        } else if (level.time - (*(*NPC).NPC).enemyLastSeenTime) <= 5000 && ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
            // NOTE: this leaves a 2-second dead zone in which they'll just sit there unless their enemy moves
            // If there is an event we might be interested in if we weren't still interested in our enemy
            if NPC_CheckAlertEvents(0, 1, (*(*NPC).NPC).lastAlertID, 0, AEL_MINOR, 1) >= 0 {
                // just stir
                SandCreature_MoveEffect();
            }
        }

        if enemyDistSq < MIN_ATTACK_DIST_SQ as c_float {
            if (*(*NPC).enemy).client != std::ptr::null_mut() {
                (*(*NPC).client).ps.viewangles[YAW as usize] = (*(*(*NPC).enemy).client).ps.viewangles[YAW as usize];
            }
            if TIMER_Done(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char) != 0 {
                // okay to attack
                SandCreature_Attack(0);
            }
        } else if enemyDistSq < MAX_MISS_DIST_SQ as c_float
            && enemyDistSq > MIN_MISS_DIST_SQ as c_float
            && (*(*NPC).enemy).client != std::ptr::null_mut()
            && TIMER_Done(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char) != 0
            && TIMER_Done(NPC, b"missDebounce\0".as_ptr() as *const core::ffi::c_char) != 0
            && VectorCompare((*NPC).pos1.as_ptr(), (*NPC).currentOrigin.as_ptr()) == 0 // so we don't come up again in the same spot
            && Q_irand(0, 10) == 0
        {
            if ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
                // miss them
                SandCreature_Attack(1);
                VectorCopy((*NPC).currentOrigin.as_ptr(), (*NPC).pos1.as_mut_ptr());
                TIMER_Set(NPC, b"missDebounce\0".as_ptr() as *const core::ffi::c_char, Q_irand(3000, 10000));
            }
        }
    }
}

pub fn SandCreature_Hunt() {
    unsafe {
        SandCreature_CheckAlerts();
        SandCreature_CheckMovingEnts();
        // If we have somewhere to go, then do that
        // FIXME: keeps chasing goalEntity even when it's already reached it...?
        if (*(*NPC).NPC).goalEntity != std::ptr::null_mut() && SandCreature_DistSqToGoal(0) >= MIN_ATTACK_DIST_SQ as c_float {
            ucmd.buttons |= BUTTON_WALKING;
            if SandCreature_Move() != 0 {
                SandCreature_MoveEffect();
            }
        } else {
            NPC_ReachedGoal();
        }
    }
}

pub fn SandCreature_Sleep() {
    unsafe {
        SandCreature_CheckAlerts();
        SandCreature_CheckMovingEnts();
        // FIXME: keeps chasing goalEntity even when it's already reached it!
        if (*(*NPC).NPC).goalEntity != std::ptr::null_mut() && SandCreature_DistSqToGoal(0) >= MIN_ATTACK_DIST_SQ as c_float {
            ucmd.buttons |= BUTTON_WALKING;
            if SandCreature_Move() != 0 {
                SandCreature_MoveEffect();
            }
        } else {
            NPC_ReachedGoal();
        }
        /*
        if ( UpdateGoal() )
        {
            ucmd.buttons |= BUTTON_WALKING;
            //FIXME: Sand Creatures look silly using waypoints when they can go straight to the goal
            if ( SandCreature_Move() )
            {
                SandCreature_MoveEffect();
            }
        }
        */
    }
}

pub fn SandCreature_PushEnts() {
    unsafe {
        let mut radiusEnts: [*mut gentity_t; 128] = [std::ptr::null_mut(); 128];
        let radius: c_float = 70.0;
        let mut mins: vec3_t = std::mem::zeroed();
        let mut maxs: vec3_t = std::mem::zeroed();
        let mut smackDir: vec3_t = std::mem::zeroed();

        let mut i: c_int = 0;
        while i < 3 {
            mins[i as usize] = (*NPC).currentOrigin[i as usize] - radius;
            maxs[i as usize] = (*NPC).currentOrigin[i as usize] + radius;
            i += 1;
        }

        let numEnts: c_int = (gi.EntitiesInBox)(mins.as_ptr(), maxs.as_ptr(), radiusEnts.as_mut_ptr(), 128);
        let mut entIndex: c_int = 0;
        while entIndex < numEnts {
            // Only Clients
            // -----------
            if radiusEnts[entIndex as usize] == std::ptr::null_mut()
                || (*radiusEnts[entIndex as usize]).client == std::ptr::null_mut()
                || radiusEnts[entIndex as usize] == NPC
            {
                entIndex += 1;
                continue;
            }

            // Do The Vector Distance Test
            // ---------------------------
            VectorSubtract(
                (*radiusEnts[entIndex as usize]).currentOrigin.as_ptr(),
                (*NPC).currentOrigin.as_ptr(),
                smackDir.as_mut_ptr(),
            );
            let smackDist: c_float = VectorNormalize(smackDir.as_mut_ptr());
            if smackDist < radius {
                G_Throw(radiusEnts[entIndex as usize], smackDir.as_ptr(), 90.0);
            }

            entIndex += 1;
        }
    }
}

pub fn NPC_BSSandCreature_Default() {
    unsafe {
        let mut visible: c_int = 0;

        // clear it every frame, will be set if we actually move this frame...
        (*NPC).s.loopSound = 0;

        if (*NPC).health > 0 && TIMER_Done(NPC, b"breaching\0".as_ptr() as *const core::ffi::c_char) != 0 {
            // go back to non-solid mode
            if (*NPC).contents != 0 {
                (*NPC).contents = 0;
            }
            if (*NPC).clipmask == MASK_NPCSOLID {
                (*NPC).clipmask = CONTENTS_SOLID | CONTENTS_MONSTERCLIP;
            }
            if TIMER_Done(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char) != 0 {
                G_SoundOnEnt(
                    NPC,
                    CHAN_VOICE,
                    va(b"sound/chars/sand_creature/voice%d.mp3\0".as_ptr() as *const core::ffi::c_char, Q_irand(1, 3)),
                );
                TIMER_Set(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char, Q_irand(3000, 10000));
            }
        } else {
            // still in breaching anim
            visible = 1;
            // FIXME: maybe push things up/away and maybe knock people down when doing this?
            // FIXME: don't turn while breaching?
            // FIXME: move faster while breaching?
            // NOTE: shaking now done whenever he moves
        }

        // FIXME: when in start and end of attack/pain anims, need ground disturbance effect around him
        // NOTENOTE: someone stubbed this code in, so I figured I'd use it.  The timers are all weird, ie, magic numbers that sort of work,
        //  but maybe I'll try and figure out real values later if I have time.
        if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK1 || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK2 {
            // FIXME: get start and end frame numbers for this effect for each of these anims
            let up: vec3_t = [0.0, 0.0, 1.0];
            let mut org: vec3_t = std::mem::zeroed();
            VectorCopy((*NPC).currentOrigin.as_ptr(), org.as_mut_ptr());
            org[2] -= 40.0;
            if (*(*NPC).client).ps.legsAnimTimer > 3700 {
                // G_PlayEffect( G_EffectIndex( "env/sand_dive"  ), NPC->currentOrigin, up );
                G_PlayEffect(G_EffectIndex(b"env/sand_spray\0".as_ptr() as *const core::ffi::c_char), org.as_ptr(), up.as_ptr());
            } else if (*(*NPC).client).ps.legsAnimTimer > 1600 && (*(*NPC).client).ps.legsAnimTimer < 1900 {
                G_PlayEffect(G_EffectIndex(b"env/sand_spray\0".as_ptr() as *const core::ffi::c_char), org.as_ptr(), up.as_ptr());
            }
            // G_PlayEffect( G_EffectIndex( "env/sand_attack_breach" ), org, up );
        }

        if TIMER_Done(NPC, b"pain\0".as_ptr() as *const core::ffi::c_char) == 0 {
            visible = 1;
        } else if TIMER_Done(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) == 0 {
            visible = 1;
        } else {
            if (*NPC).activator != std::ptr::null_mut() {
                // kill and remove the guy we ate
                // FIXME: want to play ...?  What was I going to say?
                (*(*NPC).activator).health = 0;
                GEntity_DieFunc((*NPC).activator, NPC, NPC, 1000, MOD_MELEE, 0, HL_NONE);
                if (*(*NPC).activator).s.number != 0 {
                    G_FreeEntity((*NPC).activator);
                } else {
                    // can't remove the player, just make him invisible
                    (*(*NPC).client).ps.eFlags |= EF_NODRAW;
                }
                (*NPC).activator = std::ptr::null_mut();
                (*NPC).enemy = std::ptr::null_mut();
                (*(*NPC).NPC).goalEntity = std::ptr::null_mut();
            }

            if (*NPC).enemy != std::ptr::null_mut() {
                SandCreature_Chase();
            } else if (level.time - (*(*NPC).NPC).enemyLastSeenTime) < 5000 {
                // FIXME: should make this able to be variable
                // we were alerted recently, move towards there and look for footsteps, etc.
                SandCreature_Hunt();
            } else {
                // no alerts, sleep and wake up only by alerts
                // FIXME: keeps chasing goalEntity even when it's already reached it!
                SandCreature_Sleep();
            }
        }
        NPC_UpdateAngles(1, 1);
        if visible == 0 {
            (*(*NPC).client).ps.eFlags |= EF_NODRAW;
            (*NPC).s.eFlags |= EF_NODRAW;
        } else {
            (*(*NPC).client).ps.eFlags &= !EF_NODRAW;
            (*NPC).s.eFlags &= !EF_NODRAW;

            SandCreature_PushEnts();
        }
    }
}

// FIXME: need pain behavior of sticking up through ground, writhing and screaming
// FIXME: need death anim like pain, but flopping aside and staying above ground...
