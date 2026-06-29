// leave this line at the top of all AI_xxxx files for PCH reasons...
// #include "g_headers.h"

// #include "b_local.h"

use core::ffi::{c_int, c_void};

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

const SPF_RANCOR_MUTANT: c_int = 1;
const SPF_RANCOR_FASTKILL: c_int = 2;

extern "C" {
    fn G_EntIsBreakable(entityNum: c_int, breaker: *mut gentity_t) -> qboolean;
    static mut g_dismemberment: *mut cvar_t;
    static mut g_bobaDebug: *mut cvar_t;

    // External functions and types from other modules
    fn G_SoundIndex(sound: *const c_char) -> c_int;
    fn G_EffectIndex(effect: *const c_char) -> c_int;
    fn va(format: *const c_char, ...) -> *const c_char;
    fn gi_trace(
        trace: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passent: c_int,
        contentmask: c_int,
    );
    fn NPC_SetAnim(
        ent: *mut gentity_t,
        setAnimType: c_int,
        animNum: c_int,
        flags: c_int,
    );
    fn TIMER_Set(ent: *mut gentity_t, label: *const c_char, duration: c_int);
    fn TIMER_Remove(ent: *mut gentity_t, label: *const c_char);
    fn TIMER_Done(ent: *mut gentity_t, label: *const c_char) -> qboolean;
    fn TIMER_Done2(ent: *mut gentity_t, label: *const c_char, remove: qboolean) -> qboolean;
    fn TIMER_Exists(ent: *mut gentity_t, label: *const c_char) -> qboolean;
    fn UpdateGoal() -> qboolean;
    fn NPC_MoveToGoal(override_: qboolean) -> qboolean;
    fn NPC_CheckEnemyExt(checkAlerts: qboolean) -> qboolean;
    fn NPC_ClearLOS(ent: *mut gentity_t) -> qboolean;
    fn NPC_FaceEnemy(doPitch: qboolean);
    fn NPC_FacePosition(position: *const vec3_t, doPitch: qboolean);
    fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean);
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorCompare(a: *const vec3_t, b: *const vec3_t) -> qboolean;
    fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorScale(a: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorMA(a: *const vec3_t, scale: f32, b: *const vec3_t, out: *mut vec3_t);
    fn VectorNormalize(v: *mut vec3_t) -> f32;
    fn Distance(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    fn DistanceSquared(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    fn DistanceHorizontal(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    static vec3_origin: vec3_t;
    fn G_Sound(ent: *mut gentity_t, index: c_int);
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, sound: *const c_char);
    fn G_Damage(
        target: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );
    fn G_Knockdown(
        self_: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: f32,
        breakSaberLock: qboolean,
    );
    fn G_DoDismemberment(
        self_: *mut gentity_t,
        point: *const vec3_t,
        mod_: c_int,
        damage: c_int,
        hitLoc: c_int,
        force: qboolean,
    ) -> qboolean;
    fn NPC_EntRangeFromBolt(targEnt: *mut gentity_t, boltIndex: c_int) -> f32;
    fn NPC_GetEntsNearBolt(
        radiusEnts: *mut *mut gentity_t,
        radius: f32,
        boltIndex: c_int,
        boltOrg: *mut vec3_t,
    ) -> c_int;
    fn G_Throw(ent: *mut gentity_t, dir: *const vec3_t, speed: f32);
    fn TossClientItems(self_: *mut gentity_t) -> *mut gentity_t;
    fn AddSoundEvent(
        ent: *mut gentity_t,
        origin: *const vec3_t,
        radius: f32,
        level: c_int,
        needLOS: qboolean,
        needLOSto: qboolean,
    );
    fn gi_inPVS(p1: *const vec3_t, p2: *const vec3_t) -> qboolean;
    fn G_SetEnemy(ent: *mut gentity_t, enemy: *mut gentity_t);
    fn NPC_CheckEnemy(
        ignoreTeam: qboolean,
        ignoreAI: qboolean,
        ignoreAlert: qboolean,
    ) -> *mut gentity_t;
    fn NPC_ValidEnemy(ent: *mut gentity_t) -> qboolean;
    fn GEntity_PainFunc(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        point: *const vec3_t,
        damage: c_int,
        mod_: c_int,
    );
    fn InFOV(
        point: *const vec3_t,
        org: *const vec3_t,
        angles: *const vec3_t,
        fovX: f32,
        fovY: f32,
    ) -> qboolean;
    fn AngleVectors(
        angles: *const vec3_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
    );
    fn Q_irand(min: c_int, max: c_int) -> c_int;
    fn Q_flrand(min: f32, max: f32) -> f32;
    fn AddSightEvent(
        ent: *mut gentity_t,
        origin: *const vec3_t,
        radius: f32,
        level: c_int,
        addLight: c_int,
    );
    fn G_FreeEntity(ent: *mut gentity_t);
    fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int);
    fn G_StopEffect(effectIndex: c_int, modelIndex: c_int, boltIndex: c_int, entNum: c_int);
    fn G_PlayEffect(
        effectIndex: c_int,
        modelIndex: c_int,
        boltIndex: c_int,
        entNum: c_int,
        org: *const vec3_t,
        duration: c_int,
        isRelative: qboolean,
    );
    fn CGCam_Shake(intensity: f32, duration: c_int);
    fn G_RadiusDamage(
        origin: *const vec3_t,
        attacker: *mut gentity_t,
        damage: c_int,
        radius: f32,
        ignore: *mut gentity_t,
        mod_: c_int,
    );
    static mut player: *mut gentity_t;
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gNPC_t;
    static mut ucmd: usercmd_t;
    static mut level: level_locals_t;
    fn G_DebugLine(
        start: *const vec3_t,
        end: *const vec3_t,
        duration: c_int,
        color: u32,
        clearPrevious: qboolean,
    );
    fn gi_G2API_GetBoltMatrix(
        ghoul2: *mut c_void,
        modelindex: c_int,
        boltIndex: c_int,
        boltMatrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        origin: *const vec3_t,
        time: c_int,
        modelScale: *const vec3_t,
        scale: *const vec3_t,
    );
    fn gi_G2API_GiveMeVectorFromMatrix(
        boltMatrix: *const mdxaBone_t,
        flags: c_int,
        vec: *mut vec3_t,
    );
    fn CG_DrawEdge(start: *const vec3_t, end: *const vec3_t, edge: c_int);
    static NAVDEBUG_showCollision: qboolean;
}

// Type stubs for external structures
#[repr(C)]
pub struct gentity_t {
    // Stub - full definition elsewhere
    pub s: entityState_t,
    pub client: *mut gclient_t,
    pub NPC: *mut gNPC_t,
    pub ghoul2: *mut c_void,
    pub health: c_int,
    pub maxHealth: c_int,
    pub targetname: *const c_char,
    pub classname: *const c_char,
    pub spawnflags: c_int,
    pub count: c_int,
    pub wait: f32,
    pub flags: c_int,
    pub activator: *mut gentity_t,
    pub enemy: *mut gentity_t,
    pub lastEnemy: *mut gentity_t,
    pub currentOrigin: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub s_angles: vec3_t,
    pub currentAngles: vec3_t,
    pub inuse: qboolean,
    pub clipmask: c_int,
    pub contents: c_int,
    pub target: *const c_char,
    pub pos3: vec3_t,
    pub useDebounceTime: c_int,
    pub takedamage: qboolean,
    pub playerModel: c_int,
    pub gutBolt: c_int,
    pub handLBolt: c_int,
    pub handRBolt: c_int,
    pub painDebounceTime: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eFlags: c_int,
    pub modelIndex: c_int,
    pub modelScale: [f32; 3],
    pub origin: vec3_t,
    pub time: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub NPC_class: c_int,
    pub dismembered: bool,
}

#[repr(C)]
pub struct playerState_t {
    pub legsAnim: c_int,
    pub legsAnimTimer: c_int,
    pub torsoAnimTimer: c_int,
    pub groundEntityNum: c_int,
    pub eFlags: c_int,
    pub velocity: vec3_t,
    pub moveDir: vec3_t,
    pub viewangles: vec3_t,
    pub speed: f32,
}

#[repr(C)]
pub struct gNPC_t {
    pub localState: c_int,
    pub enemyLastSeenTime: c_int,
    pub combatMove: qboolean,
    pub goalEntity: *mut gentity_t,
    pub goalRadius: f32,
    pub desiredYaw: f32,
    pub lockedDesiredYaw: f32,
    pub blockedEntity: *mut gentity_t,
    pub confusionTime: c_int,
    pub stats: npcStats_t,
    pub nextBStateThink: c_int,
    pub lastPathAngles: vec3_t,
    pub ignorePain: qboolean,
    pub scriptFlags: c_int,
}

#[repr(C)]
pub struct npcStats_t {
    pub walkSpeed: f32,
    pub runSpeed: f32,
}

#[repr(C)]
pub struct cvar_t {
    // Stub
    pub integer: c_int,
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type trace_t = [u8; 512]; // Stub - real structure elsewhere
pub type mdxaBone_t = [[f32; 4]; 3]; // Stub - matrix structure

#[repr(C)]
pub struct usercmd_t {
    pub buttons: c_int,
    pub forwardmove: c_int,
    pub rightmove: c_int,
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
}

// Stub for STEER namespace - these are called as methods
extern "C" {
    mod STEER {
        use super::*;
        pub fn Activate(ent: *mut gentity_t);
        pub fn Seek(ent: *mut gentity_t, goal: *const vec3_t);
        pub fn AvoidCollisions(ent: *mut gentity_t);
        pub fn DeActivate(ent: *mut gentity_t, ucmd: *mut usercmd_t);
    }
}

const BUTTON_WALKING: c_int = 0x80;
const CONTENTS_BOTCLIP: c_int = 0x2000000;
const CONTENTS_SOLID: c_int = 0x1;
const CONTENTS_BODY: c_int = 0x2;
const ENTITYNUM_WORLD: c_int = 0x7FFF;
const ENTITYNUM_NONE: c_int = 0x7FFE;
const MAX_CLIENTS: c_int = 64;
const Q3_INFINITE: c_int = 0x7FFFFFFF;
const EF_HELD_BY_RANCOR: c_int = 0x80;
const EF_HELD_BY_WAMPA: c_int = 0x100;
const EF_NODRAW: c_int = 0x80;
const FL_NOTARGET: c_int = 0x4;
const FL_NO_KNOCKBACK: c_int = 0x100;
const MASK_SHOT: c_int = 0x7FF;
const CONTENTS_BOTCLIP: c_int = 0x2000000;
const DAMAGE_NO_KNOCKBACK: c_int = 0x1;
const DAMAGE_NO_PROTECTION: c_int = 0x2;
const DAMAGE_NO_ARMOR: c_int = 0x8;
const DAMAGE_NO_HIT_LOC: c_int = 0x20;
const DAMAGE_IGNORE_TEAM: c_int = 0x100;
const MOD_MELEE: c_int = 13;
const MOD_CRUSH: c_int = 16;
const MOD_SABER: c_int = 19;
const MOD_LAVA: c_int = 13;
const EV_DEATH1: c_int = 40;
const EV_DEATH3: c_int = 42;
const EV_JUMP: c_int = 14;
const CHAN_AUTO: c_int = 2;
const CHAN_WEAPON: c_int = 3;
const CLASS_RANCOR: c_int = 11;
const CLASS_GALAKMECH: c_int = 15;
const CLASS_ATST: c_int = 16;
const CLASS_GONK: c_int = 17;
const CLASS_R2D2: c_int = 18;
const CLASS_R5D2: c_int = 19;
const CLASS_MARK1: c_int = 20;
const CLASS_MARK2: c_int = 21;
const CLASS_MOUSE: c_int = 22;
const CLASS_PROBE: c_int = 23;
const CLASS_SEEKER: c_int = 24;
const CLASS_REMOTE: c_int = 25;
const CLASS_SENTRY: c_int = 26;
const CLASS_INTERROGATOR: c_int = 27;
const CLASS_VEHICLE: c_int = 28;
const CLASS_JAWA: c_int = 8;
const CLASS_UGNAUGHT: c_int = 9;
const BOTH_STAND1TO2: c_int = 0;
const BOTH_ATTACK1: c_int = 1;
const BOTH_ATTACK2: c_int = 2;
const BOTH_ATTACK3: c_int = 3;
const BOTH_ATTACK4: c_int = 4;
const BOTH_ATTACK5: c_int = 5;
const BOTH_ATTACK6: c_int = 6;
const BOTH_ATTACK7: c_int = 7;
const BOTH_ATTACK10: c_int = 10;
const BOTH_ATTACK11: c_int = 11;
const BOTH_MELEE1: c_int = 12;
const BOTH_MELEE2: c_int = 13;
const BOTH_HOLD_DROP: c_int = 14;
const BOTH_HOLD_SNIFF: c_int = 15;
const BOTH_PAIN1: c_int = 16;
const BOTH_PAIN2: c_int = 17;
const BOTH_SWIM_IDLE1: c_int = 18;
const BOTH_DEATH17: c_int = 19;
const BOTH_DEATHBACKWARD2: c_int = 20;
const BOTH_GUARD_IDLE1: c_int = 21;
const BOTH_GUARD_LOOKAROUND1: c_int = 22;
const SETANIM_BOTH: c_int = 0;
const SETANIM_TORSO: c_int = 1;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;
const HL_WAIST: c_int = 2;
const HL_HEAD: c_int = 0;
const HL_BACK_RT: c_int = 1;
const HL_HAND_LT: c_int = 3;
const YAW: usize = 1;
const PITCH: usize = 0;
const ORIGIN: c_int = 0;
const NEGATIVE_Z: c_int = 1;
const FOFS: c_int = 0;
const AEL_DANGER: c_int = 1;
const AEL_DANGER_GREAT: c_int = 2;
const SCF_LOOK_FOR_ENEMIES: c_int = 0x1;
const EDGE_IMPACT_POSSIBLE: c_int = 1;

pub fn Rancor_Attack(distance: f32, doCharge: qboolean, aimAtBlockedEntity: qboolean);

/*
-------------------------
NPC_Rancor_Precache
-------------------------
*/
pub fn NPC_Rancor_Precache() {
    let mut i: c_int;
    i = 1;
    while i < 5 {
        G_SoundIndex(va("sound/chars/rancor/snort_%d.wav", i));
        i += 1;
    }
    G_SoundIndex("sound/chars/rancor/swipehit.wav" as *const c_char);
    G_SoundIndex("sound/chars/rancor/chomp.wav" as *const c_char);
}

pub fn NPC_MutantRancor_Precache() {
    G_SoundIndex("sound/chars/rancor/breath_start.wav" as *const c_char);
    G_SoundIndex("sound/chars/rancor/breath_loop.wav" as *const c_char);
    G_EffectIndex("mrancor/breath" as *const c_char);
}
//FIXME: initialize all my timers

pub fn Rancor_CheckAhead(mut end: *mut vec3_t) -> qboolean {
    let mut trace: trace_t;
    let clipmask: c_int = unsafe { (*NPC).clipmask } | CONTENTS_BOTCLIP;

    //make sure our goal isn't underground (else the trace will fail)
    let mut bottom: vec3_t = [
        unsafe { (*end)[0] },
        unsafe { (*end)[1] },
        unsafe { (*end)[2] + (*NPC).mins[2] },
    ];
    unsafe {
        gi_trace(&mut trace, end, &vec3_origin, &vec3_origin, &bottom, unsafe { (*NPC).s.number }, unsafe { (*NPC).clipmask });
    }
    if unsafe { *((&trace as *const trace_t) as *const f32) } < 1.0_f32 {
        //in the ground, raise it up
        unsafe {
            (*end)[2] -= unsafe { (*NPC).mins[2] } * (1.0_f32 - unsafe { *((&trace as *const trace_t) as *const f32) }) - 0.125_f32;
        }
    }

    unsafe {
        gi_trace(&mut trace, &unsafe { (*NPC).currentOrigin }, &unsafe { (*NPC).mins }, &unsafe { (*NPC).maxs }, end, unsafe { (*NPC).s.number }, clipmask);
    }

    unsafe {
        if *((&trace as *const trace_t) as *const u8) == 0 as u8 && *((&trace as *const trace_t) as *const u8) == 0 as u8 && *((&trace as *const trace_t) as *const f32) == 1.0_f32 {
            return 1;
        }
    }

    if unsafe { *((&trace as *const trace_t) as *const c_int) } < ENTITYNUM_WORLD
        && G_EntIsBreakable(unsafe { *((&trace as *const trace_t) as *const c_int) }, unsafe { NPC })
    {
        //breakable brush in our way, break it
        //	NPCInfo->blockedEntity = &g_entities[trace.entityNum];
        return 1;
    }

    //Aw screw it, always try to go straight at him if we can at all
    if unsafe { *((&trace as *const trace_t) as *const f32) } >= 0.25_f32 {
        return 1;
    }

    //FIXME: if something in the way that's not the world, set blocked ent
    return 0;
}

/*
-------------------------
Rancor_Idle
-------------------------
*/
pub fn Rancor_Idle() {
    unsafe {
        (*NPCInfo).localState = LSTATE_CLEAR;
    }

    //If we have somewhere to go, then do that
    if UpdateGoal() != 0 {
        unsafe {
            ucmd.buttons &= !BUTTON_WALKING;
        }
        NPC_MoveToGoal(1);
    }
}


pub fn Rancor_CheckRoar(self_: *mut gentity_t) -> qboolean {
    if unsafe { (*self_).wait } == 0.0_f32 {
        //haven't ever gotten mad yet
        unsafe {
            (*self_).wait = 1.0_f32; //do this only once
        }
        NPC_SetAnim(self_, SETANIM_BOTH, BOTH_STAND1TO2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
        TIMER_Set(self_, "rageTime" as *const c_char, unsafe { (*(*self_).client).ps.legsAnimTimer });
        return 1;
    }
    return 0;
}

/*
-------------------------
Rancor_Patrol
-------------------------
*/
pub fn Rancor_Patrol() {
    unsafe {
        (*NPCInfo).localState = LSTATE_CLEAR;
    }

    //If we have somewhere to go, then do that
    if UpdateGoal() != 0 {
        unsafe {
            ucmd.buttons &= !BUTTON_WALKING;
        }
        NPC_MoveToGoal(1);
    }

    if NPC_CheckEnemyExt(1) == 0 {
        Rancor_Idle();
        return;
    }
    Rancor_CheckRoar(unsafe { NPC });
    TIMER_Set(unsafe { NPC }, "lookForNewEnemy" as *const c_char, Q_irand(5000, 15000));
}

/*
-------------------------
Rancor_Move
-------------------------
*/
pub fn Rancor_Move(visible: qboolean) {
    if unsafe { (*NPCInfo).localState } != LSTATE_WAITING {
        unsafe {
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = (*NPC).maxs[0] + (MIN_DISTANCE as f32 * (*NPC).s.modelScale[0]);
            // just get us within combat range
            //FIXME: for some reason, if NPC_MoveToGoal fails, it sets my angles to my lastPathAngles, which I don't want
            let savYaw = (*NPCInfo).desiredYaw;
            let savWalking = (ucmd.buttons & BUTTON_WALKING) != 0;
            if !NPC_MoveToGoal(1) != 0 {
                //can't macro-nav, just head right for him
                //FIXME: if something in the way that's not the world, set blocked ent
                let mut dest: vec3_t = [0.0; 3];
                VectorCopy(&(*(*NPCInfo).goalEntity).currentOrigin, &mut dest);
                if Rancor_CheckAhead(&mut dest) != 0 {
                    //use our temp move straight to goal check
                    if !savWalking {
                        ucmd.buttons &= !BUTTON_WALKING; // Unset from MoveToGoal()
                    }
                    STEER::Activate(NPC);
                    STEER::Seek(NPC, &dest);
                    STEER::AvoidCollisions(NPC);
                    STEER::DeActivate(NPC, &mut ucmd);
                } else {
                    //all else fails, look at him
                    //	gi.Printf("Fail\n");
                    (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw = savYaw;
                    if !(*NPCInfo).blockedEntity.is_null() && (*NPC).enemy != core::ptr::null_mut() && gi_inPVS(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin) != 0 {
                        //nothing to destroy?  just go straight at goal dest
                        let mut horzClose: qboolean = 0;
                        if !savWalking {
                            ucmd.buttons &= !BUTTON_WALKING; // Unset from MoveToGoal()
                        }

                        if DistanceHorizontal(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin)
                            < (*NPC).maxs[0] + (MIN_DISTANCE as f32 * (*NPC).s.modelScale[0])
                        {
                            //close, just look at him
                            horzClose = 1;
                            NPC_FaceEnemy(1);
                        } else {
                            //try to move  towards him
                            STEER::Activate(NPC);
                            STEER::Seek(NPC, &dest);
                            STEER::AvoidCollisions(NPC);
                            STEER::DeActivate(NPC, &mut ucmd);
                        }
                        //let him know he should attack at random out of frustration?
                        if (*NPCInfo).goalEntity == (*NPC).enemy {
                            if TIMER_Done(NPC, "attacking" as *const c_char) != 0
                                && TIMER_Done(NPC, "frustrationAttack" as *const c_char) != 0
                            {
                                let enemyDist = Distance(&dest, &(*NPC).currentOrigin);
                                if (!horzClose != 0 || Q_irand(0, 5) != 0) && Q_irand(0, 1) != 0 {
                                    Rancor_Attack(enemyDist, 1, 0);
                                } else {
                                    Rancor_Attack(enemyDist, 0, 0);
                                }
                                if horzClose != 0 {
                                    TIMER_Set(NPC, "frustrationAttack" as *const c_char, Q_irand(2000, 5000));
                                } else {
                                    TIMER_Set(NPC, "frustrationAttack" as *const c_char, Q_irand(5000, 15000));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

//---------------------------------------------------------
extern "C" {
    fn G_Knockdown(
        self_: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: f32,
        breakSaberLock: qboolean,
    );
    fn G_DoDismemberment(
        self_: *mut gentity_t,
        point: *const vec3_t,
        mod_: c_int,
        damage: c_int,
        hitLoc: c_int,
        force: qboolean,
    ) -> qboolean;
    fn NPC_EntRangeFromBolt(targEnt: *mut gentity_t, boltIndex: c_int) -> f32;
    fn NPC_GetEntsNearBolt(
        radiusEnts: *mut *mut gentity_t,
        radius: f32,
        boltIndex: c_int,
        boltOrg: *mut vec3_t,
    ) -> c_int;
}

pub fn Rancor_DropVictim(self_: *mut gentity_t) {
    //FIXME: if Rancor dies, it should drop its victim.
    //FIXME: if Rancor is removed, it must remove its victim.
    //FIXME: if in BOTH_HOLD_DROP, throw them a little, too?
    if !unsafe { (*self_).activator }.is_null() {
        if !unsafe { (*(*self_).activator).client }.is_null() {
            unsafe {
                (*(*(*self_).activator).client).ps.eFlags &= !EF_HELD_BY_RANCOR;
            }
        }
        unsafe {
            (*(*self_).activator).activator = core::ptr::null_mut();
        }
        if unsafe { (*(*self_).activator).health } <= 0 {
            if unsafe { (*(*self_).activator).s.number } != 0 {
                //never free player
                if unsafe { (*self_).count } == 1 {
                    //in my hand, just drop them
                    if !unsafe { (*(*self_).activator).client }.is_null() {
                        unsafe {
                            (*(*(*self_).activator).client).ps.legsAnimTimer = 0;
                            (*(*(*self_).activator).client).ps.torsoAnimTimer = 0;
                        }
                        //FIXME: ragdoll?
                    }
                } else {
                    G_FreeEntity(unsafe { (*self_).activator });
                }
            } else {
                unsafe {
                    (*(*self_).activator).s.eFlags |= EF_NODRAW;
                    if !(*(*self_).activator).client.is_null() {
                        (*(*(*self_).activator).client).ps.eFlags |= EF_NODRAW;
                    }
                    (*(*self_).activator).clipmask &= !CONTENTS_BODY;
                }
            }
        } else {
            if !unsafe { (*(*self_).activator).NPC }.is_null() {
                //start thinking again
                unsafe {
                    (*(*(*self_).activator).NPC).nextBStateThink = level.time;
                }
            }
            //clear their anim and let them fall
            unsafe {
                (*(*(*self_).activator).client).ps.legsAnimTimer = 0;
                (*(*(*self_).activator).client).ps.torsoAnimTimer = 0;
            }
        }
        if unsafe { (*(*self_).enemy) == (*(*self_).activator) } {
            unsafe {
                (*self_).enemy = core::ptr::null_mut();
            }
        }
        if unsafe { (*(*self_).activator).s.number } == 0 {
            //don't attack the player again for a bit
            unsafe {
                TIMER_Set(self_, "attackDebounce" as *const c_char, Q_irand(2000, 4000 + ((2 - (*g_spskill).integer) * 2000)));
            }
        }
        unsafe {
            (*self_).activator = core::ptr::null_mut();
        }
    }
    unsafe {
        (*self_).count = 0; //drop him
    }
}

pub fn Rancor_Swing(boltIndex: c_int, tryGrab: qboolean) {
    let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
    let mut numEnts: c_int;
    let radius: f32 = if unsafe { (*NPC).spawnflags & SPF_RANCOR_MUTANT } != 0 {
        200.0_f32
    } else {
        88.0_f32
    };
    let radiusSquared: f32 = radius * radius;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];
    let mut originUp: vec3_t = [0.0; 3];

    unsafe {
        VectorCopy(&(*NPC).currentOrigin, &mut originUp);
        originUp[2] += (*NPC).maxs[2] * 0.75_f32;

        numEnts = NPC_GetEntsNearBolt(&mut radiusEnts[0] as *mut *mut gentity_t, radius, boltIndex, &mut boltOrg);

        //if ( NPCInfo->blockedEntity && G_EntIsBreakable( NPCInfo->blockedEntity->s.number, NPC ) )
        {
            //attacking a breakable brush
            //HMM... maybe always do this?
            //if boltOrg inside a breakable brush, damage it
            let mut trace: trace_t = [0; 512];
            gi_trace(
                &mut trace,
                &(*NPC).pos3,
                &vec3_origin,
                &vec3_origin,
                &boltOrg,
                (*NPC).s.number,
                CONTENTS_SOLID | CONTENTS_BODY,
            );
            if (*g_bobaDebug).integer > 0 {
                G_DebugLine(&(*NPC).pos3, &boltOrg, 1000, 0x000000ff, 1);
            }
            //remember pos3 for the trace from last hand pos to current hand pos next time
            VectorCopy(&boltOrg, &mut (*NPC).pos3);
            //FIXME: also do a trace TO the bolt from where we are...?
            if G_EntIsBreakable(*((&trace as *const trace_t) as *const c_int), NPC) != 0 {
                G_Damage(
                    &mut g_entities[*((&trace as *const trace_t) as *const c_int) as usize],
                    NPC,
                    NPC,
                    &vec3_origin,
                    &boltOrg,
                    100,
                    0,
                    MOD_MELEE,
                );
            } else {
                //fuck, do an actual line trace, I guess...
                gi_trace(
                    &mut trace,
                    &originUp,
                    &vec3_origin,
                    &vec3_origin,
                    &boltOrg,
                    (*NPC).s.number,
                    CONTENTS_SOLID | CONTENTS_BODY,
                );
                if (*g_bobaDebug).integer > 0 {
                    G_DebugLine(&originUp, &boltOrg, 1000, 0x000000ff, 1);
                }
                if G_EntIsBreakable(*((&trace as *const trace_t) as *const c_int), NPC) != 0 {
                    G_Damage(
                        &mut g_entities[*((&trace as *const trace_t) as *const c_int) as usize],
                        NPC,
                        NPC,
                        &vec3_origin,
                        &boltOrg,
                        200,
                        0,
                        MOD_MELEE,
                    );
                }
            }
        }

        i = 0;
        while i < numEnts {
            if radiusEnts[i as usize].is_null() || (*radiusEnts[i as usize]).inuse == 0 {
                i += 1;
                continue;
            }

            if radiusEnts[i as usize] == NPC {
                //Skip the rancor ent
                i += 1;
                continue;
            }

            if (*radiusEnts[i as usize]).client.is_null() {
                //must be a client
                i += 1;
                continue;
            }

            if (((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0)
                || (((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_WAMPA) != 0)
            {
                //can't be one already being held
                i += 1;
                continue;
            }

            if ((*radiusEnts[i as usize]).s.eFlags & EF_NODRAW) != 0 {
                //not if invisible
                i += 1;
                continue;
            }

            if DistanceSquared(&(*radiusEnts[i as usize]).currentOrigin, &boltOrg) <= radiusSquared {
                if gi_inPVS(&(*radiusEnts[i as usize]).currentOrigin, &(*NPC).currentOrigin) == 0 {
                    //don't grab anything that's in another PVS
                    i += 1;
                    continue;
                }

                if tryGrab != 0
                    && (*NPC).count != 1 //don't have one in hand or in mouth already - FIXME: allow one in hand and any number in mouth!
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_GALAKMECH
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_ATST
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_GONK
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_R2D2
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_R5D2
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_MARK1
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_MARK2
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_MOUSE
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_PROBE
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_SEEKER
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_REMOTE
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_SENTRY
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_INTERROGATOR
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_VEHICLE
                {
                    //grab
                    if (*NPC).count == 2 {
                        //have one in my mouth, remove him
                        TIMER_Remove(NPC, "clearGrabbed" as *const c_char);
                        Rancor_DropVictim(NPC);
                    }
                    (*NPC).enemy = radiusEnts[i as usize]; //make him my new best friend
                    (*(*radiusEnts[i as usize]).client).ps.eFlags |= EF_HELD_BY_RANCOR;
                    //FIXME: this makes it so that the victim can't hit us with shots!  Just use activator or something
                    (*radiusEnts[i as usize]).activator = NPC; // kind of dumb, but when we are locked to the Rancor, we are owned by it.
                    (*NPC).activator = radiusEnts[i as usize]; //remember him
                    (*NPC).count = 1; //in my hand
                    //wait to attack
                    TIMER_Set(
                        NPC,
                        "attacking" as *const c_char,
                        (*(*NPC).client).ps.legsAnimTimer + Q_irand(500, 2500),
                    );
                    if (*radiusEnts[i as usize]).health > 0 {
                        //do pain on enemy
                        GEntity_PainFunc(radiusEnts[i as usize], NPC, NPC, &(*radiusEnts[i as usize]).currentOrigin, 0, MOD_CRUSH);
                    } else if !(*radiusEnts[i as usize]).client.is_null() {
                        NPC_SetAnim(radiusEnts[i as usize], SETANIM_BOTH, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    }
                } else {
                    //smack
                    G_Sound(radiusEnts[i as usize], G_SoundIndex("sound/chars/rancor/swipehit.wav" as *const c_char));
                    //actually push the enemy
                    let mut angs: vec3_t = [0.0; 3];
                    VectorCopy(&(*(*NPC).client).ps.viewangles, &mut angs);
                    angs[YAW] += Q_flrand(25.0_f32, 50.0_f32);
                    angs[PITCH] = Q_flrand(-25.0_f32, -15.0_f32);
                    let mut pushDir: vec3_t = [0.0; 3];
                    AngleVectors(&angs, &mut pushDir, core::ptr::null_mut(), core::ptr::null_mut());
                    if (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_RANCOR
                        && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_ATST
                        && ((*radiusEnts[i as usize]).flags & FL_NO_KNOCKBACK) == 0
                    {
                        G_Throw(radiusEnts[i as usize], &pushDir, 250.0_f32);
                        if (*radiusEnts[i as usize]).health > 0 {
                            //do pain on enemy
                            G_Knockdown(radiusEnts[i as usize], NPC, &pushDir, 100.0_f32, 1);
                        }
                    }
                }
            }
            i += 1;
        }
    }
}

pub fn Rancor_Smash() {
    let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
    let mut numEnts: c_int;
    let radius: f32 = if unsafe { (*NPC).spawnflags & SPF_RANCOR_MUTANT } != 0 {
        256.0_f32
    } else {
        128.0_f32
    };
    let halfRadSquared: f32 = (radius / 2.0_f32) * (radius / 2.0_f32);
    let radiusSquared: f32 = radius * radius;
    let mut distSq: f32;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];

    unsafe {
        AddSoundEvent(NPC, &(*NPC).currentOrigin, 512.0_f32, AEL_DANGER, 0, 1);

        numEnts = NPC_GetEntsNearBolt(&mut radiusEnts[0] as *mut *mut gentity_t, radius, (*NPC).handLBolt, &mut boltOrg);

        //if ( NPCInfo->blockedEntity && G_EntIsBreakable( NPCInfo->blockedEntity->s.number, NPC ) )
        {
            //attacking a breakable brush
            //HMM... maybe always do this?
            //if boltOrg inside a breakable brush, damage it
            let mut trace: trace_t = [0; 512];
            gi_trace(
                &mut trace,
                &boltOrg,
                &vec3_origin,
                &vec3_origin,
                &(*NPC).pos3,
                (*NPC).s.number,
                CONTENTS_SOLID | CONTENTS_BODY,
            );
            if (*g_bobaDebug).integer > 0 {
                G_DebugLine(&(*NPC).pos3, &boltOrg, 1000, 0x000000ff, 1);
            }
            //remember pos3 for the trace from last hand pos to current hand pos next time
            VectorCopy(&boltOrg, &mut (*NPC).pos3);
            //FIXME: also do a trace TO the bolt from where we are...?
            if G_EntIsBreakable(*((&trace as *const trace_t) as *const c_int), NPC) != 0 {
                G_Damage(
                    &mut g_entities[*((&trace as *const trace_t) as *const c_int) as usize],
                    NPC,
                    NPC,
                    &vec3_origin,
                    &boltOrg,
                    200,
                    0,
                    MOD_MELEE,
                );
            } else {
                //fuck, do an actual line trace, I guess...
                gi_trace(
                    &mut trace,
                    &(*NPC).currentOrigin,
                    &vec3_origin,
                    &vec3_origin,
                    &boltOrg,
                    (*NPC).s.number,
                    CONTENTS_SOLID | CONTENTS_BODY,
                );
                if (*g_bobaDebug).integer > 0 {
                    G_DebugLine(&(*NPC).currentOrigin, &boltOrg, 1000, 0x000000ff, 1);
                }
                if G_EntIsBreakable(*((&trace as *const trace_t) as *const c_int), NPC) != 0 {
                    G_Damage(
                        &mut g_entities[*((&trace as *const trace_t) as *const c_int) as usize],
                        NPC,
                        NPC,
                        &vec3_origin,
                        &boltOrg,
                        200,
                        0,
                        MOD_MELEE,
                    );
                }
            }
        }

        i = 0;
        while i < numEnts {
            if radiusEnts[i as usize].is_null() || (*radiusEnts[i as usize]).inuse == 0 {
                i += 1;
                continue;
            }

            if radiusEnts[i as usize] == NPC {
                //Skip the rancor ent
                i += 1;
                continue;
            }

            if (*radiusEnts[i as usize]).client.is_null() {
                //must be a client
                if G_EntIsBreakable((*radiusEnts[i as usize]).s.number, NPC) != 0 {
                    //damage breakables within range, but not as much
                    if Q_irand(0, 1) == 0 {
                        G_Damage(
                            radiusEnts[i as usize],
                            NPC,
                            NPC,
                            &vec3_origin,
                            &(*radiusEnts[i as usize]).currentOrigin,
                            100,
                            0,
                            MOD_MELEE,
                        );
                    }
                }
                i += 1;
                continue;
            }

            if (((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0) {
                //can't be one being held
                i += 1;
                continue;
            }

            if ((*radiusEnts[i as usize]).s.eFlags & EF_NODRAW) != 0 {
                //not if invisible
                i += 1;
                continue;
            }

            distSq = DistanceSquared(&(*radiusEnts[i as usize]).currentOrigin, &boltOrg);
            if distSq <= radiusSquared {
                if distSq < halfRadSquared {
                    //close enough to do damage, too
                    G_Sound(radiusEnts[i as usize], G_SoundIndex("sound/chars/rancor/swipehit.wav" as *const c_char));
                    if ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0 && (*radiusEnts[i as usize]).s.number >= MAX_CLIENTS {
                        G_Damage(
                            radiusEnts[i as usize],
                            NPC,
                            NPC,
                            &vec3_origin,
                            &boltOrg,
                            (*radiusEnts[i as usize]).health + 1000,
                            DAMAGE_NO_KNOCKBACK | DAMAGE_NO_PROTECTION,
                            MOD_MELEE,
                        );
                    } else if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                        //more damage
                        G_Damage(
                            radiusEnts[i as usize],
                            NPC,
                            NPC,
                            &vec3_origin,
                            &(*radiusEnts[i as usize]).currentOrigin,
                            Q_irand(40, 55),
                            DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE,
                        );
                    } else {
                        G_Damage(
                            radiusEnts[i as usize],
                            NPC,
                            NPC,
                            &vec3_origin,
                            &(*radiusEnts[i as usize]).currentOrigin,
                            Q_irand(10, 25),
                            DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE,
                        );
                    }
                }
                if (*radiusEnts[i as usize]).health > 0
                    && !(*radiusEnts[i as usize]).client.is_null()
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnts[i as usize]).client).NPC_class != CLASS_ATST
                {
                    if distSq < halfRadSquared
                        || (*(*radiusEnts[i as usize]).client).ps.groundEntityNum != ENTITYNUM_NONE
                    {
                        //within range of my fist or withing ground-shaking range and not in the air
                        if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                            G_Knockdown(radiusEnts[i as usize], NPC, &vec3_origin, 500.0_f32, 1);
                        } else {
                            G_Knockdown(
                                radiusEnts[i as usize],
                                NPC,
                                &vec3_origin,
                                Q_irand(200, 350) as f32,
                                1,
                            );
                        }
                    }
                }
            }
            i += 1;
        }
    }
}

pub fn Rancor_Bite() {
    let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
    let mut numEnts: c_int;
    let radius: f32 = 100.0_f32;
    let radiusSquared: f32 = radius * radius;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];

    unsafe {
        numEnts = NPC_GetEntsNearBolt(&mut radiusEnts[0] as *mut *mut gentity_t, radius, (*NPC).gutBolt, &mut boltOrg);

        i = 0;
        while i < numEnts {
            if radiusEnts[i as usize].is_null() || (*radiusEnts[i as usize]).inuse == 0 {
                i += 1;
                continue;
            }

            if radiusEnts[i as usize] == NPC {
                //Skip the rancor ent
                i += 1;
                continue;
            }

            if (*radiusEnts[i as usize]).client.is_null() {
                //must be a client
                i += 1;
                continue;
            }

            if (((*(*radiusEnts[i as usize]).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0) {
                //can't be one already being held
                i += 1;
                continue;
            }

            if ((*radiusEnts[i as usize]).s.eFlags & EF_NODRAW) != 0 {
                //not if invisible
                i += 1;
                continue;
            }

            if DistanceSquared(&(*radiusEnts[i as usize]).currentOrigin, &boltOrg) <= radiusSquared {
                if ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0 && (*radiusEnts[i as usize]).s.number >= MAX_CLIENTS {
                    G_Damage(
                        radiusEnts[i as usize],
                        NPC,
                        NPC,
                        &vec3_origin,
                        &(*radiusEnts[i as usize]).currentOrigin,
                        (*radiusEnts[i as usize]).health + 1000,
                        DAMAGE_NO_KNOCKBACK | DAMAGE_NO_PROTECTION,
                        MOD_MELEE,
                    );
                } else if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                    //more damage
                    G_Damage(
                        radiusEnts[i as usize],
                        NPC,
                        NPC,
                        &vec3_origin,
                        &(*radiusEnts[i as usize]).currentOrigin,
                        Q_irand(35, 50),
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                } else {
                    G_Damage(
                        radiusEnts[i as usize],
                        NPC,
                        NPC,
                        &vec3_origin,
                        &(*radiusEnts[i as usize]).currentOrigin,
                        Q_irand(15, 30),
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                }
                if (*radiusEnts[i as usize]).health <= 0 && !(*radiusEnts[i as usize]).client.is_null() {
                    //killed them, chance of dismembering
                    if Q_irand(0, 1) == 0 {
                        //bite something off
                        let mut hitLoc: c_int = HL_WAIST;
                        if (*g_dismemberment).integer < 3 {
                            hitLoc = Q_irand(HL_BACK_RT, HL_HAND_LT);
                        } else {
                            hitLoc = Q_irand(HL_WAIST, HL_HEAD);
                        }
                        if hitLoc == HL_HEAD {
                            NPC_SetAnim(
                                radiusEnts[i as usize],
                                SETANIM_BOTH,
                                BOTH_DEATH17,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else if hitLoc == HL_WAIST {
                            NPC_SetAnim(
                                radiusEnts[i as usize],
                                SETANIM_BOTH,
                                BOTH_DEATHBACKWARD2,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        (*(*radiusEnts[i as usize]).client).dismembered = false;
                        //FIXME: the limb should just disappear, cuz I ate it
                        G_DoDismemberment(
                            radiusEnts[i as usize],
                            &(*radiusEnts[i as usize]).currentOrigin,
                            MOD_SABER,
                            1000,
                            hitLoc,
                            1,
                        );
                    }
                }
                G_Sound(radiusEnts[i as usize], G_SoundIndex("sound/chars/rancor/chomp.wav" as *const c_char));
            }
            i += 1;
        }
    }
}

//------------------------------
extern "C" {
    fn TossClientItems(self_: *mut gentity_t) -> *mut gentity_t;
}

pub fn Rancor_Attack(distance: f32, doCharge: qboolean, aimAtBlockedEntity: qboolean) {
    unsafe {
        if TIMER_Exists(NPC, "attacking" as *const c_char) == 0
            && TIMER_Done(NPC, "attackDebounce" as *const c_char) != 0
        {
            if (*NPC).count == 2 && !(*NPC).activator.is_null() {
            } else if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                //holding enemy
                if (((*NPC).spawnflags & SPF_RANCOR_FASTKILL) == 0
                    || (*(*NPC).activator).s.number < MAX_CLIENTS)
                    && (*(*NPC).activator).health > 0
                    && Q_irand(0, 1) != 0
                {
                    //quick bite
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_ATTACK1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TIMER_Set(NPC, "attack_dmg" as *const c_char, 450);
                } else {
                    //full eat
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_ATTACK3, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TIMER_Set(NPC, "attack_dmg" as *const c_char, 900);
                    //Make victim scream in fright
                    if (*(*NPC).activator).health > 0 && !(*(*NPC).activator).client.is_null() {
                        G_AddEvent((*NPC).activator, Q_irand(EV_DEATH1, EV_DEATH3), 0);
                        NPC_SetAnim(
                            (*NPC).activator,
                            SETANIM_TORSO,
                            BOTH_FALLDEATH1,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        if !(*(*NPC).activator).NPC.is_null() {
                            //no more thinking for you
                            TossClientItems(NPC);
                            (*(*(*NPC).activator).NPC).nextBStateThink = Q3_INFINITE;
                        }
                    }
                }
            } else if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health > 0 && doCharge != 0 {
                //charge
                if Q_irand(0, 3) == 0 {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_ATTACK5, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TIMER_Set(NPC, "attack_dmg" as *const c_char, 1250);
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).s.number == 0 {
                        //don't attack the player again for a bit
                        TIMER_Set(
                            NPC,
                            "attackDebounce" as *const c_char,
                            (*(*NPC).client).ps.legsAnimTimer
                                + Q_irand(2000, 4000 + ((2 - (*g_spskill).integer) * 2000)),
                        );
                    }
                } else if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                    //breath attack
                    let mut breathAnim: c_int = BOTH_ATTACK4;
                    let mut checkEnt: *mut gentity_t = core::ptr::null_mut();
                    let mut center: vec3_t = [0.0; 3];
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).inuse != 0 {
                        checkEnt = (*NPC).enemy;
                        VectorCopy(&(*(*NPC).enemy).currentOrigin, &mut center);
                    } else if !(*NPCInfo).blockedEntity.is_null() && (*(*NPCInfo).blockedEntity).inuse != 0 {
                        checkEnt = (*NPCInfo).blockedEntity;
                        //if it has an origin brush, use it...
                        if VectorCompare(&(*(*NPCInfo).blockedEntity).s.origin, &vec3_origin) != 0 {
                            //no origin brush, calc center
                            VectorAdd(&(*(*NPCInfo).blockedEntity).mins, &(*(*NPCInfo).blockedEntity).maxs, &mut center);
                            VectorScale(&center, 0.5_f32, &mut center);
                        } else {
                            //use origin brush as center
                            VectorCopy(&(*(*NPCInfo).blockedEntity).s.origin, &mut center);
                        }
                    }
                    if !checkEnt.is_null() {
                        let zHeightRelative: f32 = center[2] - (*NPC).currentOrigin[2];
                        if zHeightRelative >= 128.0_f32 * (*NPC).s.modelScale[2] {
                            breathAnim = BOTH_ATTACK7;
                        } else if zHeightRelative >= 64.0_f32 * (*NPC).s.modelScale[2] {
                            breathAnim = BOTH_ATTACK6;
                        }
                    }
                    NPC_SetAnim(NPC, SETANIM_BOTH, breathAnim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    //start effect here
                    G_PlayEffect(
                        G_EffectIndex("mrancor/breath" as *const c_char),
                        (*NPC).playerModel,
                        (*NPC).gutBolt,
                        (*NPC).s.number,
                        &(*NPC).currentOrigin,
                        (*(*NPC).client).ps.legsAnimTimer - 500,
                        0,
                    );
                    TIMER_Set(NPC, "breathAttack" as *const c_char, (*(*NPC).client).ps.legsAnimTimer - 500);
                    G_SoundOnEnt(NPC, CHAN_WEAPON, "sound/chars/rancor/breath_start.wav" as *const c_char);
                    (*NPC).s.loopSound = G_SoundIndex("sound/chars/rancor/breath_loop.wav" as *const c_char);
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).s.number == 0 {
                        //don't attack the player again for a bit
                        TIMER_Set(
                            NPC,
                            "attackDebounce" as *const c_char,
                            (*(*NPC).client).ps.legsAnimTimer
                                + Q_irand(2000, 4000 + ((2 - (*g_spskill).integer) * 2000)),
                        );
                    }
                } else {
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_MELEE2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TIMER_Set(NPC, "attack_dmg" as *const c_char, 1250);
                    let yawAng: vec3_t = [0.0_f32, (*(*NPC).client).ps.viewangles[YAW], 0.0_f32];
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&yawAng, &mut fwd, core::ptr::null_mut(), core::ptr::null_mut());
                    VectorScale(&fwd, distance * 1.5_f32, &mut (*(*NPC).client).ps.velocity);
                    (*(*NPC).client).ps.velocity[2] = 150.0_f32;
                    (*(*NPC).client).ps.groundEntityNum = ENTITYNUM_NONE;
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).s.number == 0 {
                        //don't attack the player again for a bit
                        TIMER_Set(
                            NPC,
                            "attackDebounce" as *const c_char,
                            (*(*NPC).client).ps.legsAnimTimer
                                + Q_irand(2000, 4000 + ((2 - (*g_spskill).integer) * 2000)),
                        );
                    }
                }
            } else if Q_irand(0, 1) == 0 {
                //mutant rancor can smash
                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_MELEE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                TIMER_Set(NPC, "attack_dmg" as *const c_char, 900);
                //init pos3 for the trace from last hand pos to current hand pos
                VectorCopy(&(*NPC).currentOrigin, &mut (*NPC).pos3);
            } else if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0
                || distance >= (*NPC).maxs[0] + (MIN_DISTANCE as f32 * (*NPC).s.modelScale[0]) - 64.0_f32
            {
                //try to grab
                let mut grabAnim: c_int = BOTH_ATTACK2;
                let mut checkEnt: *mut gentity_t = core::ptr::null_mut();
                let mut center: vec3_t = [0.0; 3];
                if (aimAtBlockedEntity == 0 || (*NPCInfo).blockedEntity.is_null()) && !(*NPC).enemy.is_null() && (*(*NPC).enemy).inuse != 0 {
                    checkEnt = (*NPC).enemy;
                    VectorCopy(&(*(*NPC).enemy).currentOrigin, &mut center);
                } else if !(*NPCInfo).blockedEntity.is_null() && (*(*NPCInfo).blockedEntity).inuse != 0 {
                    checkEnt = (*NPCInfo).blockedEntity;
                    //if it has an origin brush, use it...
                    if VectorCompare(&(*(*NPCInfo).blockedEntity).s.origin, &vec3_origin) != 0 {
                        //no origin brush, calc center
                        VectorAdd(&(*(*NPCInfo).blockedEntity).mins, &(*(*NPCInfo).blockedEntity).maxs, &mut center);
                        VectorScale(&center, 0.5_f32, &mut center);
                    } else {
                        //use origin brush as center
                        VectorCopy(&(*(*NPCInfo).blockedEntity).s.origin, &mut center);
                    }
                }
                if !checkEnt.is_null() {
                    let zHeightRelative: f32 = center[2] - (*NPC).currentOrigin[2];
                    if zHeightRelative >= 128.0_f32 * (*NPC).s.modelScale[2] {
                        grabAnim = BOTH_ATTACK11;
                    } else if zHeightRelative >= 64.0_f32 * (*NPC).s.modelScale[2] {
                        grabAnim = BOTH_ATTACK10;
                    }
                }
                NPC_SetAnim(NPC, SETANIM_BOTH, grabAnim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                TIMER_Set(NPC, "attack_dmg" as *const c_char, 800);
                if !(*NPC).enemy.is_null() && (*(*NPC).enemy).s.number == 0 {
                    //don't attack the player again for a bit
                    TIMER_Set(
                        NPC,
                        "attackDebounce" as *const c_char,
                        (*(*NPC).client).ps.legsAnimTimer
                            + Q_irand(2000, 4000 + ((2 - (*g_spskill).integer) * 2000)),
                    );
                }
                //init pos3 for the trace from last hand pos to current hand pos
                VectorCopy(&(*NPC).currentOrigin, &mut (*NPC).pos3);
            } else {
                //FIXME: back up?
                ucmd.forwardmove = -64;
                //FIXME: check for walls/ledges?
                return;
            }

            TIMER_Set(NPC, "attacking" as *const c_char, (*(*NPC).client).ps.legsAnimTimer + (random() * 200.0_f32) as c_int);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
        if TIMER_Done2(NPC, "attack_dmg" as *const c_char, 1) != 0 {
            match (*(*NPC).client).ps.legsAnim {
                BOTH_MELEE1 => {
                    Rancor_Smash();
                    let playerDist = NPC_EntRangeFromBolt(player, (*NPC).handLBolt);
                    if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                        if playerDist < 512.0_f32 {
                            CGCam_Shake(1.0_f32 * playerDist / 256.0_f32, 1000);
                        }
                    } else {
                        if playerDist < 256.0_f32 {
                            CGCam_Shake(1.0_f32 * playerDist / 128.0_f32, 1000);
                        }
                    }
                }
                BOTH_MELEE2 => {
                    Rancor_Bite();
                    TIMER_Set(NPC, "attack_dmg2" as *const c_char, 450);
                }
                BOTH_ATTACK1 => {
                    if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                        if ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0
                            && (*(*NPC).activator).s.number >= MAX_CLIENTS
                        {
                            G_Damage(
                                (*NPC).activator,
                                NPC,
                                NPC,
                                &vec3_origin,
                                &(*(*NPC).activator).currentOrigin,
                                (*(*NPC).activator).health + 1000,
                                DAMAGE_NO_KNOCKBACK | DAMAGE_NO_PROTECTION,
                                MOD_MELEE,
                            );
                        } else if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                            //more damage
                            G_Damage(
                                (*NPC).activator,
                                NPC,
                                NPC,
                                &vec3_origin,
                                &(*(*NPC).activator).currentOrigin,
                                Q_irand(55, 70),
                                DAMAGE_NO_KNOCKBACK,
                                MOD_MELEE,
                            );
                        } else {
                            G_Damage(
                                (*NPC).activator,
                                NPC,
                                NPC,
                                &vec3_origin,
                                &(*(*NPC).activator).currentOrigin,
                                Q_irand(25, 40),
                                DAMAGE_NO_KNOCKBACK,
                                MOD_MELEE,
                            );
                        }
                        if (*(*NPC).activator).health <= 0 {
                            //killed him
                            if (*g_dismemberment).integer >= 3 {
                                //make it look like we bit his head off
                                (*(*(*NPC).activator).client).dismembered = false;
                                G_DoDismemberment(
                                    (*NPC).activator,
                                    &(*(*NPC).activator).currentOrigin,
                                    MOD_SABER,
                                    1000,
                                    HL_HEAD,
                                    1,
                                );
                            }
                            NPC_SetAnim(
                                (*NPC).activator,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        G_Sound((*NPC).activator, G_SoundIndex("sound/chars/rancor/chomp.wav" as *const c_char));
                    }
                }
                BOTH_ATTACK2 | BOTH_ATTACK10 | BOTH_ATTACK11 => {
                    //try to grab
                    Rancor_Swing((*NPC).handRBolt, 1);
                }
                BOTH_ATTACK3 => {
                    if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                        //cut in half
                        if !(*(*NPC).activator).client.is_null() {
                            (*(*(*NPC).activator).client).dismembered = false;
                            G_DoDismemberment(
                                (*NPC).activator,
                                &(*(*NPC).enemy).currentOrigin,
                                MOD_SABER,
                                1000,
                                HL_WAIST,
                                1,
                            );
                        }
                        //KILL
                        G_Damage(
                            (*NPC).activator,
                            NPC,
                            NPC,
                            &vec3_origin,
                            &(*(*NPC).activator).currentOrigin,
                            (*(*NPC).enemy).health + 1000,
                            DAMAGE_NO_PROTECTION | DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                            MOD_MELEE,
                        );
                        if !(*(*NPC).activator).client.is_null() {
                            NPC_SetAnim(
                                (*NPC).activator,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        TIMER_Set(NPC, "attack_dmg2" as *const c_char, 1350);
                        G_Sound((*NPC).activator, G_SoundIndex("sound/chars/rancor/swipehit.wav" as *const c_char));
                        G_AddEvent((*NPC).activator, EV_JUMP, (*(*NPC).activator).health);
                    }
                }
                _ => {}
            }
        } else if TIMER_Done2(NPC, "attack_dmg2" as *const c_char, 1) != 0 {
            match (*(*NPC).client).ps.legsAnim {
                BOTH_MELEE1 => {}
                BOTH_MELEE2 => {
                    Rancor_Bite();
                }
                BOTH_ATTACK1 => {}
                BOTH_ATTACK2 => {}
                BOTH_ATTACK3 => {
                    if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                        //swallow victim
                        G_Sound((*NPC).activator, G_SoundIndex("sound/chars/rancor/chomp.wav" as *const c_char));
                        //FIXME: sometimes end up with a live one in our mouths?
                        //just make sure they're dead
                        if (*(*NPC).activator).health > 0 {
                            //cut in half
                            (*(*(*NPC).activator).client).dismembered = false;
                            G_DoDismemberment(
                                (*NPC).activator,
                                &(*(*NPC).enemy).currentOrigin,
                                MOD_SABER,
                                1000,
                                HL_WAIST,
                                1,
                            );
                            //KILL
                            G_Damage(
                                (*NPC).activator,
                                NPC,
                                NPC,
                                &vec3_origin,
                                &(*(*NPC).activator).currentOrigin,
                                (*(*NPC).enemy).health + 1000,
                                DAMAGE_NO_PROTECTION | DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                                MOD_MELEE,
                            );
                            NPC_SetAnim(
                                (*NPC).activator,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                            G_AddEvent((*NPC).activator, EV_JUMP, (*(*NPC).activator).health);
                        }
                        (*NPC).count = 2;
                        TIMER_Set(NPC, "clearGrabbed" as *const c_char, 2600);
                    }
                }
                _ => {}
            }
        }

        // Just using this to remove the attacking flag at the right time
        TIMER_Done2(NPC, "attacking" as *const c_char, 1);
    }
}

//----------------------------------
pub fn Rancor_Combat() {
    unsafe {
        if (*NPC).count != 0 {
            //holding my enemy
            (*NPCInfo).enemyLastSeenTime = level.time;
            if TIMER_Done2(NPC, "takingPain" as *const c_char, 1) != 0 {
                (*NPCInfo).localState = LSTATE_CLEAR;
            } else if ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0 && !(*NPC).activator.is_null() && (*(*NPC).activator).s.number >= MAX_CLIENTS {
                Rancor_Attack(0.0_f32, 0, 0);
            } else if (*NPC).useDebounceTime >= level.time && !(*NPC).activator.is_null() {
                //just sniffing the guy
                if (*NPC).useDebounceTime <= level.time + 100 && (*(*NPC).client).ps.legsAnim != BOTH_HOLD_DROP {
                    //just about done, drop him
                    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_HOLD_DROP, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    TIMER_Set(
                        NPC,
                        "attacking" as *const c_char,
                        (*(*NPC).client).ps.legsAnimTimer + (Q_irand(500, 1000) * (3 - (*g_spskill).integer)),
                    );
                }
            } else {
                if (*NPC).useDebounceTime == 0 && !(*NPC).activator.is_null() && (*(*NPC).activator).s.number < MAX_CLIENTS {
                    //first time I pick the player, just sniff them
                    if TIMER_Done(NPC, "attacking" as *const c_char) != 0 {
                        //ready to attack
                        NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_HOLD_SNIFF, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                        (*NPC).useDebounceTime = level.time + (*(*NPC).client).ps.legsAnimTimer + Q_irand(500, 2000);
                    }
                } else {
                    Rancor_Attack(0.0_f32, 0, 0);
                }
            }
            NPC_UpdateAngles(1, 1);
            return;
        }

        (*NPCInfo).goalRadius = (*NPC).maxs[0] + (MAX_DISTANCE as f32 * (*NPC).s.modelScale[0]);
        // just get us within combat range

        // If we cannot see our target or we have somewhere to go, then do that
        if NPC_ClearLOS((*NPC).enemy) == 0 || UpdateGoal() != 0 {
            (*NPCInfo).combatMove = 1;
            (*NPCInfo).goalEntity = (*NPC).enemy;

            Rancor_Move(0);
            return;
        }

        (*NPCInfo).enemyLastSeenTime = level.time;
        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        NPC_FaceEnemy(1);

        let distance: f32 = Distance(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);

        let advance: qboolean = if distance > (*NPC).maxs[0] + (MIN_DISTANCE as f32 * (*NPC).s.modelScale[0]) {
            1
        } else {
            0
        };
        let mut doCharge: qboolean = 0;

        if advance != 0 {
            //have to get closer
            if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 && ((*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null()) {
                //don't do breath attack vs. bbrushes
            } else {
                let yawOnlyAngles: vec3_t = [0.0_f32, (*NPC).currentAngles[YAW], 0.0_f32];
                if (*(*NPC).enemy).health > 0
                    && ((distance - (250.0_f32 * (*NPC).s.modelScale[0])).abs()) <= (80.0_f32 * (*NPC).s.modelScale[0])
                    && InFOV(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &yawOnlyAngles, 30.0_f32, 30.0_f32) != 0
                {
                    let mut chance: c_int 9;
                    if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 {
                        //higher chance of doing breath attack
                        chance = 5 - (*g_spskill).integer;
                    }
                    if Q_irand(0, chance) == 0 {
                        //go for the charge
                        doCharge = 1;
                    }
                }
            }
        }

        if ((advance != 0 || (*NPCInfo).localState == LSTATE_WAITING) && TIMER_Done(NPC, "attacking" as *const c_char) != 0) {
            // waiting monsters can't attack
            if TIMER_Done2(NPC, "takingPain" as *const c_char, 1) != 0 {
                (*NPCInfo).localState = LSTATE_CLEAR;
            } else {
                Rancor_Move(1);
            }
        } else {
            Rancor_Attack(distance, doCharge, 0);
        }
    }
}

/*
-------------------------
NPC_Rancor_Pain
-------------------------
*/
pub fn NPC_Rancor_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    let mut hitByRancor: qboolean = 0;

    unsafe {
        if !self_.is_null() && !(*self_).NPC.is_null() && (*(*self_).NPC).ignorePain != 0 {
            return;
        }
        if TIMER_Done(self_, "breathAttack" as *const c_char) == 0 {
            //nothing interrupts breath attack
            return;
        }

        TIMER_Remove(self_, "confusionTime" as *const c_char);

        if !other.is_null() && !(*other).client.is_null() && (*(*other).client).NPC_class == CLASS_RANCOR {
            hitByRancor = 1;
        }
        if !other.is_null()
            && (*other).inuse != 0
            && other != (*self_).enemy
            && ((*other).flags & FL_NOTARGET) == 0
        {
            if (*self_).count == 0 {
                if ((*other).s.number == 0 && Q_irand(0, 3) == 0)
                    || (*self_).enemy.is_null()
                    || (*(*self_).enemy).health == 0
                    || (!(*(*self_).enemy).client.is_null() && (*(*(*self_).enemy).client).NPC_class == CLASS_RANCOR)
                    || (Q_irand(0, 4) == 0
                        && DistanceSquared(&(*other).currentOrigin, &(*self_).currentOrigin)
                            < DistanceSquared(&(*(*self_).enemy).currentOrigin, &(*self_).currentOrigin))
                {
                    //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                    //FIXME: if can't nav to my enemy, take this guy if I can nav to him
                    (*self_).lastEnemy = (*self_).enemy;
                    G_SetEnemy(self_, other);
                    if (*self_).enemy != (*self_).lastEnemy {
                        //clear this so that we only sniff the player the first time we pick them up
                        (*self_).useDebounceTime = 0;
                    }
                    TIMER_Set(self_, "lookForNewEnemy" as *const c_char, Q_irand(5000, 15000));
                    if hitByRancor != 0 {
                        //stay mad at this Rancor for 2-5 secs before looking for other enemies
                        TIMER_Set(self_, "rancorInfight" as *const c_char, Q_irand(2000, 5000));
                    }
                }
            }
        }
        if (hitByRancor != 0
            || ((*self_).count == 1 && !(*self_).activator.is_null() && Q_irand(0, 4) == 0)
            || Q_irand(0, 200) < damage)
            //hit by rancor, hit while holding live victim, or took a lot of damage
            && (*(*self_).client).ps.legsAnim != BOTH_STAND1TO2
            && TIMER_Done(self_, "takingPain" as *const c_char) != 0
        {
            if Rancor_CheckRoar(self_) == 0 {
                if (*(*self_).client).ps.legsAnim != BOTH_MELEE1
                    && (*(*self_).client).ps.legsAnim != BOTH_MELEE2
                    && (*(*self_).client).ps.legsAnim != BOTH_ATTACK2
                    && (*(*self_).client).ps.legsAnim != BOTH_ATTACK10
                    && (*(*self_).client).ps.legsAnim != BOTH_ATTACK11
                {
                    //cant interrupt one of the big attack anims
                    {
                        //if going to bite our victim, only victim can interrupt that anim
                        if (*self_).health > 100 || hitByRancor != 0 {
                            TIMER_Remove(self_, "attacking" as *const c_char);

                            VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s_angles);

                            if (*self_).count == 1 {
                                NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            } else {
                                NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            }
                            TIMER_Set(
                                self_,
                                "takingPain" as *const c_char,
                                (*(*self_).client).ps.legsAnimTimer + Q_irand(0, 500 * (2 - (*g_spskill).integer)),
                            );

                            if !(*self_).NPC.is_null() {
                                (*(*self_).NPC).localState = LSTATE_WAITING;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn Rancor_CheckDropVictim() {
    unsafe {
        if ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0 && (*(*NPC).activator).s.number >= MAX_CLIENTS {
            return;
        }
        let mins: vec3_t = [
            (*(*NPC).activator).mins[0] - 1.0_f32,
            (*(*NPC).activator).mins[1] - 1.0_f32,
            0.0_f32,
        ];
        let maxs: vec3_t = [
            (*(*NPC).activator).maxs[0] + 1.0_f32,
            (*(*NPC).activator).maxs[1] + 1.0_f32,
            1.0_f32,
        ];
        let start: vec3_t = [
            (*(*NPC).activator).currentOrigin[0],
            (*(*NPC).activator).currentOrigin[1],
            (*(*NPC).activator).absmin[2],
        ];
        let end: vec3_t = [
            (*(*NPC).activator).currentOrigin[0],
            (*(*NPC).activator).currentOrigin[1],
            (*(*NPC).activator).absmax[2] - 1.0_f32,
        ];
        let mut trace: trace_t = [0; 512];
        gi_trace(&mut trace, &start, &mins, &maxs, &end, (*(*NPC).activator).s.number, (*(*NPC).activator).clipmask);
        if *((&trace as *const trace_t) as *const u8) == 0
            && *((&trace as *const trace_t) as *const u8) == 0
            && *((&trace as *const trace_t) as *const f32) >= 1.0_f32
        {
            Rancor_DropVictim(NPC);
        }
    }
}

pub fn Rancor_AttackBBrush() -> qboolean {
    unsafe {
        let mut trace: trace_t = [0; 512];
        let mut center: vec3_t = [0.0; 3];
        let mut dir2Brush: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let checkDist: f32 = 64.0_f32;

        if VectorCompare(&(*(*NPCInfo).blockedEntity).s.origin, &vec3_origin) != 0 {
            //no origin brush, calc center
            VectorAdd(&(*(*NPCInfo).blockedEntity).mins, &(*(*NPCInfo).blockedEntity).maxs, &mut center);
            VectorScale(&center, 0.5_f32, &mut center);
        } else {
            VectorCopy(&(*(*NPCInfo).blockedEntity).s.origin, &mut center);
        }
        if NAVDEBUG_showCollision != 0 {
            CG_DrawEdge(&(*NPC).currentOrigin, &center, EDGE_IMPACT_POSSIBLE);
        }
        center[2] = (*NPC).currentOrigin[2]; //we can't fly, so let's ignore z diff
        NPC_FacePosition(&center, 0);
        //see if we're close to it
        VectorSubtract(&center, &(*NPC).currentOrigin, &mut dir2Brush);
        let brushSize: f32 = (((*(*NPCInfo).blockedEntity).maxs[0] - (*(*NPCInfo).blockedEntity).mins[0]) * 0.5_f32
            + ((*(*NPCInfo).blockedEntity).maxs[1] - (*(*NPCInfo).blockedEntity).mins[1]) * 0.5_f32)
            * 0.5_f32;
        let dist2Brush: f32 = VectorNormalize(&mut dir2Brush) - (*NPC).maxs[0] - brushSize;
        if dist2Brush < (MIN_DISTANCE as f32 * (*NPC).s.modelScale[0]) {
            //close enough to just hit it
            *((&mut trace as *mut trace_t) as *mut f32) = 0.0_f32;
            *((&mut trace as *mut trace_t) as *mut c_int) = (*(*NPCInfo).blockedEntity).s.number;
        } else {
            VectorMA(&(*NPC).currentOrigin, checkDist, &dir2Brush, &mut end);
            gi_trace(
                &mut trace,
                &(*NPC).currentOrigin,
                &(*NPC).mins,
                &(*NPC).maxs,
                &end,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if *((&trace as *const trace_t) as *const u8) != 0
                || *((&trace as *const trace_t) as *const u8) != 0
            {
                //wtf?
                (*NPCInfo).blockedEntity = core::ptr::null_mut();
                return 0;
            }
        }
        if *((&trace as *const trace_t) as *const f32) >= 1.0_f32
            //too far away
            || *((&trace as *const trace_t) as *const c_int) != (*(*NPCInfo).blockedEntity).s.number
        //OR blocked by something else
        {
            //keep moving towards it
            ucmd.buttons &= !BUTTON_WALKING;
            STEER::Activate(NPC);
            STEER::Seek(NPC, &center);
            STEER::AvoidCollisions(NPC);
            STEER::DeActivate(NPC, &mut ucmd);
        } else if *((&trace as *const trace_t) as *const c_int) == (*(*NPCInfo).blockedEntity).s.number {
            //close enough, smash it!
            Rancor_Attack((*((&trace as *const trace_t) as *const f32) * checkDist), 0, 1);
            TIMER_Remove(NPC, "attackDebounce" as *const c_char);
            (*NPCInfo).enemyLastSeenTime = level.time;
        } else {
            if G_EntIsBreakable(*((&trace as *const trace_t) as *const c_int), NPC) != 0 {
                //oh, well, smash that, then
                let prevblockedEnt: *mut gentity_t = (*NPCInfo).blockedEntity;
                (*NPCInfo).blockedEntity = &mut g_entities[*((&trace as *const trace_t) as *const c_int) as usize];
                Rancor_Attack((*((&trace as *const trace_t) as *const f32) * checkDist), 0, 1);
                TIMER_Remove(NPC, "attackDebounce" as *const c_char);
                (*NPCInfo).enemyLastSeenTime = level.time;
                (*NPCInfo).blockedEntity = prevblockedEnt;
            } else {
                (*NPCInfo).blockedEntity = core::ptr::null_mut();
                return 0;
            }
        }
        return 1;
    }
}

pub fn Rancor_FireBreathAttack() {
    unsafe {
        let damage: c_int = Q_irand(10, 15);
        let mut tr: trace_t = [0; 512];
        let mut traceEnt: *mut gentity_t = core::ptr::null_mut();
        let mut boltMatrix: mdxaBone_t = [[0.0; 4]; 3];
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let traceMins: vec3_t = [-4.0_f32, -4.0_f32, -4.0_f32];
        let traceMaxs: vec3_t = [4.0_f32, 4.0_f32, 4.0_f32];
        let rancAngles: vec3_t = [0.0_f32, (*(*NPC).client).ps.viewangles[YAW], 0.0_f32];

        gi_G2API_GetBoltMatrix(
            (*NPC).ghoul2,
            (*NPC).playerModel,
            (*NPC).gutBolt,
            &mut boltMatrix,
            &rancAngles,
            &(*NPC).currentOrigin,
            if level.time != 0 { level.time } else { level.time },
            &(*NPC).s.modelScale,
        );

        gi_G2API_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut start);
        gi_G2API_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Z, &mut dir);
        VectorMA(&start, 512.0_f32, &dir, &mut end);

        gi_trace(&mut tr, &start, &traceMins, &traceMaxs, &end, (*NPC).s.number, MASK_SHOT);

        traceEnt = &mut g_entities[*((&tr as *const trace_t) as *const c_int) as usize];
        if *((&tr as *const trace_t) as *const c_int) < ENTITYNUM_WORLD
            && (*traceEnt).takedamage != 0
            && !(*traceEnt).client.is_null()
        {
            //breath attack only does damage to living things
            G_Damage(
                traceEnt,
                NPC,
                NPC,
                &dir,
                &*((&tr as *const trace_t) as *const vec3_t),
                damage * 2,
                DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC | DAMAGE_IGNORE_TEAM,
                MOD_LAVA,
            );
        }
        if *((&tr as *const trace_t) as *const f32) < 1.0_f32 {
            //hit something, do radius damage
            G_RadiusDamage(&*((&tr as *const trace_t) as *const vec3_t), NPC, damage, 250.0_f32, NPC, MOD_LAVA);
        }
    }
}

pub fn Rancor_CheckAnimDamage() {
    unsafe {
        if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK2
            || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK10
            || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK11
        {
            if (*(*NPC).client).ps.legsAnimTimer >= 1200 && (*(*NPC).client).ps.legsAnimTimer <= 1350 {
                if Q_irand(0, 2) != 0 {
                    Rancor_Swing((*NPC).handRBolt, 0);
                } else {
                    Rancor_Swing((*NPC).handRBolt, 1);
                }
            } else if (*(*NPC).client).ps.legsAnimTimer >= 1100 && (*(*NPC).client).ps.legsAnimTimer <= 1550 {
                Rancor_Swing((*NPC).handRBolt, 1);
            }
        } else if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK5 {
            if (*(*NPC).client).ps.legsAnimTimer >= 750 && (*(*NPC).client).ps.legsAnimTimer <= 1300 {
                Rancor_Swing((*NPC).handLBolt, 0);
            } else if (*(*NPC).client).ps.legsAnimTimer >= 1700 && (*(*NPC).client).ps.legsAnimTimer <= 2300 {
                Rancor_Swing((*NPC).handRBolt, 0);
            }
        }
    }
}

/*
-------------------------
NPC_BSRancor_Default
-------------------------
*/
pub fn NPC_BSRancor_Default() {
    unsafe {
        AddSightEvent(NPC, &(*NPC).currentOrigin, 1024.0_f32, AEL_DANGER_GREAT, 50);

        if !(*NPCInfo).blockedEntity.is_null() && TIMER_Done(NPC, "blockedEntityIgnore" as *const c_char) != 0 {
            if TIMER_Exists(NPC, "blockedEntityTimeOut" as *const c_char) == 0 {
                TIMER_Set(NPC, "blockedEntityTimeOut" as *const c_char, 5000);
            } else if TIMER_Done(NPC, "blockedEntityTimeOut" as *const c_char) != 0 {
                TIMER_Remove(NPC, "blockedEntityTimeOut" as *const c_char);
                TIMER_Set(NPC, "blockedEntityIgnore" as *const c_char, 25000);
                (*NPCInfo).blockedEntity = core::ptr::null_mut();
            }
        } else {
            TIMER_Remove(NPC, "blockedEntityTimeOut" as *const c_char);
            TIMER_Remove(NPC, "blockedEntityIgnore" as *const c_char);
        }

        Rancor_CheckAnimDamage();

        if TIMER_Done(NPC, "breathAttack" as *const c_char) == 0 {
            //doing breath attack, just do damage
            Rancor_FireBreathAttack();
            NPC_UpdateAngles(1, 1);
            return;
        } else if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK4
            || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK6
            || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK7
        {
            G_StopEffect(G_EffectIndex("mrancor/breath" as *const c_char), (*NPC).playerModel, (*NPC).gutBolt, (*NPC).s.number);
            (*NPC).s.loopSound = 0;
        }

        if TIMER_Done2(NPC, "clearGrabbed" as *const c_char, 1) != 0 {
            Rancor_DropVictim(NPC);
        } else if ((*(*NPC).client).ps.legsAnim == BOTH_PAIN2 || (*(*NPC).client).ps.legsAnim == BOTH_HOLD_DROP)
            && (*NPC).count == 1
            && !(*NPC).activator.is_null()
        {
            Rancor_CheckDropVictim();
        }
        if TIMER_Done(NPC, "rageTime" as *const c_char) == 0 {
            //do nothing but roar first time we see an enemy
            AddSoundEvent(NPC, &(*NPC).currentOrigin, 1024.0_f32, AEL_DANGER_GREAT, 0, 0);
            NPC_FaceEnemy(1);
            return;
        }

        if (*NPCInfo).localState == LSTATE_WAITING && TIMER_Done2(NPC, "takingPain" as *const c_char, 1) != 0 {
            //was not doing anything because we were taking pain, but pain is done now, so clear it...
            (*NPCInfo).localState = LSTATE_CLEAR;
        }

        if TIMER_Done(NPC, "confusionTime" as *const c_char) == 0 {
            NPC_UpdateAngles(1, 1);
            return;
        }

        if !(*NPC).enemy.is_null() {
            if !(*(*NPC).enemy).client.is_null()
                && ((*(*(*NPC).enemy).client).NPC_class == CLASS_UGNAUGHT
                    || (*(*(*NPC).enemy).client).NPC_class == CLASS_JAWA)
                && (*(*NPC).enemy).enemy != NPC
                && ((*(*NPC).enemy).enemy.is_null()
                    || (*(*(*NPC).enemy).enemy).client.is_null()
                    || (*(*(*(*NPC).enemy).enemy).client).NPC_class != CLASS_RANCOR)
            {
                //they should be scared of ME and no-one else
                G_SetEnemy((*NPC).enemy, NPC);
            }
            if TIMER_Done(NPC, "angrynoise" as *const c_char) != 0 {
                G_SoundOnEnt(NPC, CHAN_AUTO, va("sound/chars/rancor/anger%d.wav" as *const c_char, Q_irand(1, 3)));

                TIMER_Set(NPC, "angrynoise" as *const c_char, Q_irand(5000, 10000));
            } else {
                AddSoundEvent(NPC, &(*NPC).currentOrigin, 512.0_f32, AEL_DANGER_GREAT, 0, 0);
            }
            if (*NPC).count == 2 && (*(*NPC).client).ps.legsAnim == BOTH_ATTACK3 {
                //we're still chewing our enemy up
                NPC_UpdateAngles(1, 1);
                return;
            }
            //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
            if !(*(*NPC).enemy).client.is_null() && (*(*(*NPC).enemy).client).NPC_class == CLASS_RANCOR {
                //got mad at another Rancor, look for a valid enemy
                if TIMER_Done(NPC, "rancorInfight" as *const c_char) != 0 {
                    NPC_CheckEnemyExt(1);
                }
            } else if (*NPC).count == 0 {
                if !(*NPCInfo).blockedEntity.is_null() {
                    //something in our way
                    if (*(*NPCInfo).blockedEntity).inuse == 0 {
                        //was destroyed
                        (*NPCInfo).blockedEntity = core::ptr::null_mut();
                    } else {
                        //a breakable?
                        if G_EntIsBreakable((*(*NPCInfo).blockedEntity).s.number, NPC) != 0 {
                            //breakable brush
                            if Rancor_AttackBBrush() == 0 {
                                //didn't move inside that func, so call move here...?
                                Rancor_Move(1);
                            }
                            NPC_UpdateAngles(1, 1);
                            return;
                        } else {
                            //if it's a client and in our way, get mad at it!
                            if (*NPCInfo).blockedEntity != (*NPC).enemy
                                && !(*(*NPCInfo).blockedEntity).client.is_null()
                                && NPC_ValidEnemy((*NPCInfo).blockedEntity) != 0
                                && Q_irand(0, 9) == 0
                            {
                                G_SetEnemy(NPC, (*NPCInfo).blockedEntity);
                                //look again in 2-5 secs
                                TIMER_Set(NPC, "lookForNewEnemy" as *const c_char, Q_irand(2000, 5000));
                                (*NPCInfo).blockedEntity = core::ptr::null_mut();
                            }
                        }
                    }
                }
                if NPC_ValidEnemy((*NPC).enemy) == 0 {
                    TIMER_Remove(NPC, "lookForNewEnemy" as *const c_char); //make them look again right now
                    if (*(*NPC).enemy).inuse == 0
                        || level.time - (*(*NPC).enemy).s.time > Q_irand(10000, 15000)
                        || ((*NPC).spawnflags & SPF_RANCOR_FASTKILL) != 0
                    {
                        //don't linger on dead bodies
                        //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                        if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 && !player.is_null() && (*player).health >= 0 {
                            //all else failing, always go after the player
                            (*NPC).lastEnemy = (*NPC).enemy;
                            G_SetEnemy(NPC, player);
                            if (*NPC).enemy != (*NPC).lastEnemy {
                                //clear this so that we only sniff the player the first time we pick them up
                                (*NPC).useDebounceTime = 0;
                            }
                        } else {
                            (*NPC).enemy = core::ptr::null_mut();
                            Rancor_Patrol();
                            NPC_UpdateAngles(1, 1);
                            return;
                        }
                    }
                }
                if TIMER_Done(NPC, "lookForNewEnemy" as *const c_char) != 0 {
                    let sav_enemy: *mut gentity_t = (*NPC).enemy;
                    (*NPC).enemy = core::ptr::null_mut();
                    let newEnemy: *mut gentity_t = NPC_CheckEnemy(if (*NPCInfo).confusionTime < level.time { 1 } else { 0 }, 0, 0);
                    (*NPC).enemy = sav_enemy;
                    if !newEnemy.is_null() && newEnemy != sav_enemy {
                        //picked up a new enemy!
                        (*NPC).lastEnemy = (*NPC).enemy;
                        G_SetEnemy(NPC, newEnemy);
                        if (*NPC).enemy != (*NPC).lastEnemy {
                            //clear this so that we only sniff the player the first time we pick them up
                            (*NPC).useDebounceTime = 0;
                        }
                        //hold this one for at least 5-15 seconds
                        TIMER_Set(NPC, "lookForNewEnemy" as *const c_char, Q_irand(5000, 15000));
                    } else {
                        //look again in 2-5 secs
                        TIMER_Set(NPC, "lookForNewEnemy" as *const c_char, Q_irand(2000, 5000));
                    }
                }
            }
            Rancor_Combat();
            if TIMER_Done(NPC, "attacking" as *const c_char) != 0
                && TIMER_Done(NPC, "takingpain" as *const c_char) != 0
                && TIMER_Done(NPC, "confusionDebounce" as *const c_char) != 0
                && (*NPCInfo).localState == LSTATE_CLEAR
                && (*NPC).count == 0
            {
                //not busy
                if ucmd.forwardmove == 0 && ucmd.rightmove == 0 && VectorCompare(&(*(*NPC).client).ps.moveDir, &vec3_origin) != 0 {
                    //not moving
                    if level.time - (*NPCInfo).enemyLastSeenTime > 5000 {
                        //haven't seen an enemy in a while
                        if Q_irand(0, 20) == 0 {
                            if Q_irand(0, 1) != 0 {
                                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            } else {
                                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_LOOKAROUND1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            }
                            TIMER_Set(NPC, "confusionTime" as *const c_char, (*(*NPC).client).ps.legsAnimTimer);
                            TIMER_Set(
                                NPC,
                                "confusionDebounce" as *const c_char,
                                (*(*NPC).client).ps.legsAnimTimer + Q_irand(4000, 8000),
                            );
                        }
                    }
                }
            }
        } else {
            if TIMER_Done(NPC, "idlenoise" as *const c_char) != 0 {
                G_SoundOnEnt(NPC, CHAN_AUTO, va("sound/chars/rancor/snort_%d.wav" as *const c_char, Q_irand(1, 4)));

                TIMER_Set(NPC, "idlenoise" as *const c_char, Q_irand(2000, 4000));
                AddSoundEvent(NPC, &(*NPC).currentOrigin, 384.0_f32, AEL_DANGER, 0, 0);
            }
            if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                Rancor_Patrol();
                if (*NPC).enemy.is_null() && (*NPC).wait != 0.0_f32 {
                    //we've been mad before and can't find an enemy
                    if ((*NPC).spawnflags & SPF_RANCOR_MUTANT) != 0 && !player.is_null() && (*player).health >= 0 {
                        //all else failing, always go after the player
                        (*NPC).lastEnemy = (*NPC).enemy;
                        G_SetEnemy(NPC, player);
                        if (*NPC).enemy != (*NPC).lastEnemy {
                            //clear this so that we only sniff the player the first time we pick them up
                            (*NPC).useDebounceTime = 0;
                        }
                    }
                }
            } else {
                Rancor_Idle();
            }
        }

        NPC_UpdateAngles(1, 1);
    }
}

extern "C" {
    static mut g_entities: [gentity_t; 2048];
    static mut g_spskill: *mut cvar_t;
    fn random() -> f32;
}
