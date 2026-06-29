// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 128;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

use core::ffi::{c_int, c_void};

// External declarations for engine functions and globals
extern "C" {
    fn G_SoundIndex(s: *const c_int) -> c_int;
    fn va(fmt: *const c_int, ...) -> *const c_int;
    fn UpdateGoal() -> c_int;
    fn NPC_MoveToGoal(stop: c_int) -> c_void;
    fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]) -> c_void;
    fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
    fn G_SetEnemy(ent: *mut gentity_t, enemy: *mut gentity_t) -> c_void;
    fn NPC_CheckEnemyExt(checkAll: c_int) -> c_int;
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]) -> c_void;
    fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]) -> c_void;
    fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const [f32; 3], point: *const [f32; 3], damage: c_int, dflags: c_int, mod_: c_int) -> c_void;
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundpath: *const c_int) -> c_void;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn TIMER_Exists(ent: *mut gentity_t, name: *const c_int) -> c_int;
    fn random() -> f32;
    fn TIMER_Set(ent: *mut gentity_t, name: *const c_int, duration: c_int) -> c_void;
    fn NPC_SetAnim(ent: *mut gentity_t, setanim_level: c_int, anim: c_int, flags: c_int) -> c_void;
    fn TIMER_Done2(ent: *mut gentity_t, name: *const c_int, remove: c_int) -> c_int;
    fn TIMER_Done(ent: *mut gentity_t, name: *const c_int) -> c_int;
    fn NPC_ClearLOS(ent: *mut gentity_t) -> c_int;
    fn DistanceHorizontalSquared(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;
    fn NPC_FaceEnemy(doPitch: c_int) -> c_void;
    fn TIMER_Remove(ent: *mut gentity_t, name: *const c_int) -> c_void;
    fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]) -> c_void;
    fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int) -> c_void;
    fn NPC_UpdateAngles(doPitch: c_int, doYaw: c_int) -> c_void;

    // Globals
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gclient_t;
    static mut g_entities: *mut gentity_t;
    static mut ucmd: usercmd_t;
    static vec3_origin: [f32; 3];

    // Constants / enums
    static BUTTON_WALKING: c_int;
    static ENTITYNUM_NONE: c_int;
    static MASK_SHOT: c_int;
    static MOD_MELEE: c_int;
    static DAMAGE_NO_KNOCKBACK: c_int;
    static CHAN_VOICE_ATTEN: c_int;
    static SETANIM_BOTH: c_int;
    static BOTH_ATTACK4: c_int;
    static BOTH_ATTACK3: c_int;
    static BOTH_ATTACK1: c_int;
    static BOTH_ATTACK2: c_int;
    static BOTH_PAIN1: c_int;
    static SETANIM_FLAG_OVERRIDE: c_int;
    static SETANIM_FLAG_HOLD: c_int;
    static EV_PAIN: c_int;
    static SCF_LOOK_FOR_ENEMIES: c_int;
}

// Type stubs for structural coherence
#[repr(C)]
pub struct gentity_t {
    // Placeholder for full type definition elsewhere
}

#[repr(C)]
pub struct gclient_t {
    // Placeholder for full type definition elsewhere
}

#[repr(C)]
pub struct usercmd_t {
    // Placeholder for full type definition elsewhere
}

#[repr(C)]
pub struct trace_t {
    endpos: [f32; 3],
    entityNum: c_int,
    // Additional fields omitted for brevity
}

extern "C" {
    fn gi_trace(tr: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passent: c_int, contentmask: c_int) -> c_void;
}

/*
-------------------------
NPC_MineMonster_Precache
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_MineMonster_Precache() {
    unsafe {
        for i in 0..4 {
            G_SoundIndex(va(
                "sound/chars/mine/misc/bite%i.wav\0".as_ptr() as *const c_int,
                i + 1,
            ));
            G_SoundIndex(va(
                "sound/chars/mine/misc/miss%i.wav\0".as_ptr() as *const c_int,
                i + 1,
            ));
        }
    }
}


/*
-------------------------
MineMonster_Idle
-------------------------
*/
#[no_mangle]
pub extern "C" fn MineMonster_Idle() {
    unsafe {
        if UpdateGoal() != 0 {
            (*addr_of_mut!(ucmd)).buttons &= !BUTTON_WALKING;
            NPC_MoveToGoal(1);
        }
    }
}


/*
-------------------------
MineMonster_Patrol
-------------------------
*/
#[no_mangle]
pub extern "C" fn MineMonster_Patrol() {
    unsafe {
        (*NPCInfo).localState = LSTATE_CLEAR;

        // If we have somewhere to go, then do that
        if UpdateGoal() != 0 {
            (*addr_of_mut!(ucmd)).buttons &= !BUTTON_WALKING;
            NPC_MoveToGoal(1);
        }

        let mut dif: [f32; 3] = [0.0; 3];
        VectorSubtract(
            &(*g_entities.offset(0)).currentOrigin,
            &(*NPC).currentOrigin,
            &mut dif,
        );

        if VectorLengthSquared(&dif) < 256.0 * 256.0 {
            G_SetEnemy(NPC, &mut *g_entities.offset(0));
        }

        if NPC_CheckEnemyExt(1) == 0 {
            MineMonster_Idle();
            return;
        }
    }
}

/*
-------------------------
MineMonster_Move
-------------------------
*/
#[no_mangle]
pub extern "C" fn MineMonster_Move(_visible: c_int) {
    unsafe {
        if (*NPCInfo).localState != LSTATE_WAITING {
            (*NPCInfo).goalEntity = (*NPC).enemy;
            NPC_MoveToGoal(1);
            (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range
        }
    }
}

/*---------------------------------------------------------*/
#[no_mangle]
pub extern "C" fn MineMonster_TryDamage(enemy: *mut gentity_t, damage: c_int) {
    unsafe {
        let mut end: [f32; 3] = [0.0; 3];
        let mut dir: [f32; 3] = [0.0; 3];
        let mut tr: trace_t = core::mem::zeroed();

        if enemy.is_null() {
            return;
        }

        AngleVectors(
            &(*(*NPC).client).ps.viewangles,
            &mut dir,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        VectorMA(&(*NPC).currentOrigin, MIN_DISTANCE as f32, &dir, &mut end);

        // Should probably trace from the mouth, but, ah well.
        gi_trace(
            &mut tr,
            &(*NPC).currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*NPC).s.number,
            MASK_SHOT,
        );

        if tr.entityNum >= 0 && tr.entityNum < ENTITYNUM_NONE {
            G_Damage(
                &mut *g_entities.offset(tr.entityNum as isize),
                NPC,
                NPC,
                &dir,
                &tr.endpos,
                damage,
                DAMAGE_NO_KNOCKBACK,
                MOD_MELEE,
            );
            G_SoundOnEnt(
                NPC,
                CHAN_VOICE_ATTEN,
                va(
                    "sound/chars/mine/misc/bite%i.wav\0".as_ptr() as *const c_int,
                    Q_irand(1, 4),
                ),
            );
        } else {
            G_SoundOnEnt(
                NPC,
                CHAN_VOICE_ATTEN,
                va(
                    "sound/chars/mine/misc/miss%i.wav\0".as_ptr() as *const c_int,
                    Q_irand(1, 4),
                ),
            );
        }
    }
}

/*------------------------------*/
#[no_mangle]
pub extern "C" fn MineMonster_Attack() {
    unsafe {
        if TIMER_Exists(NPC, "attacking\0".as_ptr() as *const c_int) == 0 {
            // usually try and play a jump attack if the player somehow got above them....or just really rarely
            if !(*NPC).enemy.is_null()
                && (((*(*NPC).enemy).currentOrigin[2] - (*NPC).currentOrigin[2] > 10.0
                    && random() > 0.1)
                    || random() > 0.8)
            {
                // Going to do ATTACK4
                TIMER_Set(
                    NPC,
                    "attacking\0".as_ptr() as *const c_int,
                    (1750.0 + random() * 200.0) as c_int,
                );
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK4,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                TIMER_Set(
                    NPC,
                    "attack2_dmg\0".as_ptr() as *const c_int,
                    950,
                ); // level two damage
            } else if random() > 0.5 {
                if random() > 0.8 {
                    // Going to do ATTACK3, (rare)
                    TIMER_Set(
                        NPC,
                        "attacking\0".as_ptr() as *const c_int,
                        850,
                    );
                    NPC_SetAnim(
                        NPC,
                        SETANIM_BOTH,
                        BOTH_ATTACK3,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );

                    TIMER_Set(
                        NPC,
                        "attack2_dmg\0".as_ptr() as *const c_int,
                        400,
                    ); // level two damage
                } else {
                    // Going to do ATTACK1
                    TIMER_Set(
                        NPC,
                        "attacking\0".as_ptr() as *const c_int,
                        850,
                    );
                    NPC_SetAnim(
                        NPC,
                        SETANIM_BOTH,
                        BOTH_ATTACK1,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );

                    TIMER_Set(
                        NPC,
                        "attack1_dmg\0".as_ptr() as *const c_int,
                        450,
                    ); // level one damage
                }
            } else {
                // Going to do ATTACK2
                TIMER_Set(
                    NPC,
                    "attacking\0".as_ptr() as *const c_int,
                    1250,
                );
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK2,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                TIMER_Set(
                    NPC,
                    "attack1_dmg\0".as_ptr() as *const c_int,
                    700,
                ); // level one damage
            }
        } else {
            // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
            if TIMER_Done2(
                NPC,
                "attack1_dmg\0".as_ptr() as *const c_int,
                1,
            ) != 0
            {
                MineMonster_TryDamage((*NPC).enemy, 5);
            } else if TIMER_Done2(
                NPC,
                "attack2_dmg\0".as_ptr() as *const c_int,
                1,
            ) != 0
            {
                MineMonster_TryDamage((*NPC).enemy, 10);
            }
        }

        // Just using this to remove the attacking flag at the right time
        TIMER_Done2(NPC, "attacking\0".as_ptr() as *const c_int, 1);
    }
}

/*----------------------------------*/
#[no_mangle]
pub extern "C" fn MineMonster_Combat() {
    unsafe {
        // If we cannot see our target or we have somewhere to go, then do that
        if NPC_ClearLOS((*NPC).enemy) == 0 || UpdateGoal() != 0 {
            (*NPCInfo).combatMove = 1;
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range

            NPC_MoveToGoal(1);
            return;
        }

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        NPC_FaceEnemy(1);

        let distance: f32 =
            DistanceHorizontalSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);

        let advance: c_int = if distance > MIN_DISTANCE_SQR as f32 { 1 } else { 0 };

        if (advance != 0 || (*NPCInfo).localState == LSTATE_WAITING)
            && TIMER_Done(NPC, "attacking\0".as_ptr() as *const c_int) != 0
        {
            // waiting monsters can't attack
            if TIMER_Done2(
                NPC,
                "takingPain\0".as_ptr() as *const c_int,
                1,
            ) != 0
            {
                (*NPCInfo).localState = LSTATE_CLEAR;
            } else {
                MineMonster_Move(1);
            }
        } else {
            MineMonster_Attack();
        }
    }
}

/*
-------------------------
NPC_MineMonster_Pain
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_MineMonster_Pain(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _other: *mut gentity_t,
    _point: *const [f32; 3],
    damage: c_int,
    _mod: c_int,
    _hitLoc: c_int,
) {
    unsafe {
        G_AddEvent(
            self_,
            EV_PAIN,
            ((((*self_).health as f32) / ((*self_).max_health as f32) * 100.0).floor()) as c_int,
        );

        if damage >= 10 {
            TIMER_Remove(self_, "attacking\0".as_ptr() as *const c_int);
            TIMER_Remove(self_, "attacking1_dmg\0".as_ptr() as *const c_int);
            TIMER_Remove(self_, "attacking2_dmg\0".as_ptr() as *const c_int);
            TIMER_Set(self_, "takingPain\0".as_ptr() as *const c_int, 1350);

            VectorCopy(
                &(*(*self_).NPC).lastPathAngles,
                &mut (*self_).s.angles,
            );

            NPC_SetAnim(
                self_,
                SETANIM_BOTH,
                BOTH_PAIN1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            if !(*self_).NPC.is_null() {
                (*(*self_).NPC).localState = LSTATE_WAITING;
            }
        }
    }
}


/*
-------------------------
NPC_BSMineMonster_Default
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_BSMineMonster_Default() {
    unsafe {
        if !(*NPC).enemy.is_null() {
            MineMonster_Combat();
        } else if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            MineMonster_Patrol();
        } else {
            MineMonster_Idle();
        }

        NPC_UpdateAngles(1, 1);
    }
}
