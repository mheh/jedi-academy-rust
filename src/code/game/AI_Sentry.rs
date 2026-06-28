// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

use core::ffi::{c_int, c_char, c_void};

// Placeholder types and externs - actual definitions should be in imported modules
#[repr(C)]
pub struct gentity_t {
    // Placeholder - full structure defined elsewhere
}

#[repr(C)]
pub struct mdxaBone_t {
    // Placeholder - full structure defined elsewhere
}

#[repr(C)]
pub struct trace_t {
    // Placeholder - full structure defined elsewhere
}

#[repr(C)]
pub struct gitem_t {
    // Placeholder - full structure defined elsewhere
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

extern "C" {
    pub fn CreateMissile(org: *const vec3_t, dir: *const vec3_t, vel: f32, life: c_int, owner: *mut gentity_t, altFire: qboolean) -> *mut gentity_t;
    pub fn FindItemForAmmo(ammo: c_int) -> *mut gitem_t;
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn RegisterItem(item: *mut gitem_t);
    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    pub fn NPC_SetAnim(ent: *mut gentity_t, setanim_type: c_int, anim: c_int, flags: c_int);
    pub fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int);
    pub fn TIMER_Set(ent: *mut gentity_t, label: *const c_char, time: c_int);
    pub fn TIMER_Done(ent: *mut gentity_t, label: *const c_char) -> qboolean;
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundIndex: *const c_char);
    pub fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean);
    pub fn NPC_FaceEnemy(doPitch: qboolean);
    pub fn NPC_MoveToGoal(allowRun: qboolean);
    pub fn NPC_CheckEnemyExt() -> qboolean;
    pub fn NPC_ClearLOS(ent: *mut gentity_t) -> qboolean;
    pub fn NPC_CheckPlayerTeamStealth() -> qboolean;
    pub fn UpdateGoal() -> qboolean;
    pub fn NPC_BSIdle();
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn G_PlayEffect(effectName: *const c_char, org: *const vec3_t, dir: *const vec3_t);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn Q_irand(lower: c_int, upper: c_int) -> c_int;
    pub fn VectorMA(vec: *const vec3_t, scale: f32, dir: *const vec3_t, out: *mut vec3_t);
    pub fn VectorSubtract(vec1: *const vec3_t, vec2: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(vec: *mut vec3_t) -> f32;
    pub fn DistanceHorizontalSquared(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    pub fn random() -> f32;
    pub fn fabs(x: f32) -> f32;
    pub fn rand() -> c_int;
}

// Global NPC pointer
extern "C" {
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut c_void;
    pub static mut ucmd: c_void;
}

// External globals
extern "C" {
    pub static mut level: LevelLocals;
    pub static mut cg: CGameState;
    pub static mut g_spskill: *mut cvar_t;
}

#[repr(C)]
pub struct LevelLocals {
    pub time: c_int,
    // Placeholder - more fields elsewhere
}

#[repr(C)]
pub struct CGameState {
    pub time: c_int,
    // Placeholder - more fields elsewhere
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // Placeholder - more fields elsewhere
}

// Local state enums
pub const LSTATE_NONE: c_int = 0;
pub const LSTATE_ASLEEP: c_int = 1;
pub const LSTATE_WAKEUP: c_int = 2;
pub const LSTATE_ACTIVE: c_int = 3;
pub const LSTATE_POWERING_UP: c_int = 4;
pub const LSTATE_ATTACKING: c_int = 5;

const MIN_DISTANCE: c_int = 256;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const SENTRY_FORWARD_BASE_SPEED: c_int = 10;
const SENTRY_FORWARD_MULTIPLIER: c_int = 5;

const SENTRY_VELOCITY_DECAY: f32 = 0.85f32;
const SENTRY_STRAFE_VEL: c_int = 256;
const SENTRY_STRAFE_DIS: c_int = 200;
const SENTRY_UPWARD_PUSH: c_int = 32;
const SENTRY_HOVER_HEIGHT: c_int = 24;

// Flags and constants
const FL_SHIELDED: c_int = 0x00000200;
const BOTH_POWERUP1: c_int = 0; // Placeholder - actual animation indices elsewhere
const BOTH_FLY_SHIELDED: c_int = 1; // Placeholder
const BOTH_ATTACK1: c_int = 2; // Placeholder
const BOTH_SLEEP1: c_int = 3; // Placeholder

const SETANIM_BOTH: c_int = 0; // Placeholder
const SETANIM_FLAG_OVERRIDE: c_int = 1; // Placeholder
const SETANIM_FLAG_HOLD: c_int = 2; // Placeholder

const CHAN_AUTO: c_int = 0; // Placeholder

const AMMO_BLASTER: c_int = 0; // Placeholder
const WP_BRYAR_PISTOL: c_int = 0; // Placeholder
const MOD_DEMP2: c_int = 0; // Placeholder
const MOD_DEMP2_ALT: c_int = 0; // Placeholder
const MOD_ENERGY: c_int = 0; // Placeholder
const DAMAGE_DEATH_KNOCKBACK: c_int = 0; // Placeholder
const MASK_SHOT: c_int = 0; // Placeholder
const CONTENTS_LIGHTSABER: c_int = 0; // Placeholder
const MASK_SOLID: c_int = 0; // Placeholder

const SCF_LOOK_FOR_ENEMIES: c_int = 0x00000001; // Placeholder
const SCF_CHASE_ENEMIES: c_int = 0x00000002; // Placeholder

const BUTTON_WALKING: c_int = 0x00000008; // Placeholder

const ORIGIN: c_int = 0; // Placeholder for G2API_GiveMeVectorFromMatrix

const useF_sentry_use: c_int = 0; // Placeholder for e_UseFunc

/*
-------------------------
NPC_Sentry_Precache
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_Sentry_Precache() {
    unsafe {
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_explo\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_pain\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_shield_open\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_shield_close\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_hover_1_lp\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/sentry/misc/sentry_hover_2_lp\0".as_ptr() as *const c_char);

        for i in 1..4 {
            G_SoundIndex(va(b"sound/chars/sentry/misc/talk%d\0".as_ptr() as *const c_char, i));
        }

        G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);
        G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);

        RegisterItem(FindItemForAmmo(AMMO_BLASTER));
    }
}

/*
================
sentry_use
================
*/
#[no_mangle]
pub extern "C" fn sentry_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    unsafe {
        G_ActivateBehavior(self_, 0); // BSET_USE placeholder

        (*self_).flags &= !FL_SHIELDED;
        NPC_SetAnim(self_, SETANIM_BOTH, BOTH_POWERUP1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
        //	self->NPC->localState = LSTATE_WAKEUP;
        // (*(*self_).NPC).localState = LSTATE_ACTIVE;
    }
}

/*
-------------------------
NPC_Sentry_Pain
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_Sentry_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int) {
    unsafe {
        NPC_Pain(self_, inflictor, other, point, damage, mod_);

        if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
            // (*(*self_).NPC).burstCount = 0;
            TIMER_Set(self_, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(9000, 12000));
            (*self_).flags |= FL_SHIELDED;
            NPC_SetAnim(self_, SETANIM_BOTH, BOTH_FLY_SHIELDED, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
            G_SoundOnEnt(self_, CHAN_AUTO, b"sound/chars/sentry/misc/sentry_pain\0".as_ptr() as *const c_char);

            // (*(*self_).NPC).localState = LSTATE_ACTIVE;
        }

        // You got hit, go after the enemy
        //	if (self->NPC->localState == LSTATE_ASLEEP)
        //	{
        //		G_Sound( self, G_SoundIndex("sound/chars/sentry/misc/shieldsopen.wav"));
        //
        //		self->flags &= ~FL_SHIELDED;
        //		NPC_SetAnim( self, SETANIM_BOTH, BOTH_POWERUP1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
        //		self->NPC->localState = LSTATE_WAKEUP;
        //	}
    }
}

/*
-------------------------
Sentry_Fire
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_Fire() {
    unsafe {
        let mut muzzle: vec3_t = [0.0f32; 3];
        let mut forward: vec3_t = [0.0f32; 3];
        let mut vright: vec3_t = [0.0f32; 3];
        let mut up: vec3_t = [0.0f32; 3];
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut bolt: c_int = 0;

        (*NPC).flags &= !FL_SHIELDED;

        // let npc_info = &mut *NPCInfo;
        // if npc_info.localState == LSTATE_POWERING_UP {
        //     if TIMER_Done(NPC, b"powerup\0".as_ptr() as *const c_char) != 0 {
        //         npc_info.localState = LSTATE_ATTACKING;
        //         NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_ATTACK1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
        //     } else {
        //         // can't do anything right now
        //         return;
        //     }
        // } else if npc_info.localState == LSTATE_ACTIVE {
        //     npc_info.localState = LSTATE_POWERING_UP;
        //
        //     G_SoundOnEnt(NPC, CHAN_AUTO, b"sound/chars/sentry/misc/sentry_shield_open\0".as_ptr() as *const c_char);
        //     NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_POWERUP1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
        //     TIMER_Set(NPC, b"powerup\0".as_ptr() as *const c_char, 250);
        //     return;
        // } else if npc_info.localState != LSTATE_ATTACKING {
        //     // bad because we are uninitialized
        //     npc_info.localState = LSTATE_ACTIVE;
        //     return;
        // }

        // Which muzzle to fire from?
        // let which = npc_info.burstCount % 3;
        // match which {
        //     0 => bolt = (*NPC).genericBolt1,
        //     1 => bolt = (*NPC).genericBolt2,
        //     2 | _ => bolt = (*NPC).genericBolt3,
        // }

        // gi.G2API_GetBoltMatrix( NPC->ghoul2, NPC->playerModel,
        //             bolt,
        //             &boltMatrix, NPC->currentAngles, NPC->currentOrigin, (cg.time?cg.time:level.time),
        //             NULL, NPC->s.modelScale );
        //
        // gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, muzzle );
        //
        // AngleVectors( NPC->currentAngles, forward, vright, up );
        // //	G_Sound( NPC, G_SoundIndex("sound/chars/sentry/misc/shoot.wav"));
        //
        // G_PlayEffect( "bryar/muzzle_flash", muzzle, forward );
        //
        // missile = CreateMissile( muzzle, forward, 1600, 10000, NPC );
        //
        // missile->classname = "bryar_proj";
        // missile->s.weapon = WP_BRYAR_PISTOL;
        //
        // missile->dflags = DAMAGE_DEATH_KNOCKBACK;
        // missile->methodOfDeath = MOD_ENERGY;
        // missile->clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
        //
        // NPCInfo->burstCount++;
        // NPC->attackDebounceTime = level.time + 50;
        // missile->damage = 5;
        //
        // // now scale for difficulty
        // if ( g_spskill->integer == 0 )
        // {
        //     NPC->attackDebounceTime += 200;
        //     missile->damage = 1;
        // }
        // else if ( g_spskill->integer == 1 )
        // {
        //     NPC->attackDebounceTime += 100;
        //     missile->damage = 3;
        // }
    }
}

/*
-------------------------
Sentry_MaintainHeight
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_MaintainHeight() {
    unsafe {
        let mut dif: f32 = 0.0f32;

        (*NPC).s.loopSound = G_SoundIndex(b"sound/chars/sentry/misc/sentry_hover_1_lp\0".as_ptr() as *const c_char);

        // Update our angles regardless
        NPC_UpdateAngles(1, 1);

        // If we have an enemy, we should try to hover at about enemy eye level
        if !(*NPC).enemy.is_null() {
            // Find the height difference
            dif = ((*(*NPC).enemy).currentOrigin[2] + (*(*NPC).enemy).maxs[2]) - (*NPC).currentOrigin[2];

            // cap to prevent dramatic height shifts
            if fabs(dif) > 8.0f32 {
                if fabs(dif) > SENTRY_HOVER_HEIGHT as f32 {
                    dif = if dif < 0.0f32 { -24.0f32 } else { 24.0f32 };
                }

                (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0f32;
            }
        } else {
            let mut goal: *mut gentity_t = core::ptr::null_mut();

            // if NPCInfo->goalEntity {	// Is there a goal?
            //     goal = NPCInfo->goalEntity;
            // } else {
            //     goal = NPCInfo->lastGoalEntity;
            // }

            if !goal.is_null() {
                dif = (*goal).currentOrigin[2] - (*NPC).currentOrigin[2];

                if fabs(dif) > SENTRY_HOVER_HEIGHT as f32 {
                    // ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if (*(*NPC).client).ps.velocity[2] != 0.0f32 {
                        (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                        if fabs((*(*NPC).client).ps.velocity[2]) < 2.0f32 {
                            (*(*NPC).client).ps.velocity[2] = 0.0f32;
                        }
                    }
                }
            }
            // Apply friction to Z
            else if (*(*NPC).client).ps.velocity[2] != 0.0f32 {
                (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                if fabs((*(*NPC).client).ps.velocity[2]) < 1.0f32 {
                    (*(*NPC).client).ps.velocity[2] = 0.0f32;
                }
            }
        }

        // Apply friction
        if (*(*NPC).client).ps.velocity[0] != 0.0f32 {
            (*(*NPC).client).ps.velocity[0] *= SENTRY_VELOCITY_DECAY;

            if fabs((*(*NPC).client).ps.velocity[0]) < 1.0f32 {
                (*(*NPC).client).ps.velocity[0] = 0.0f32;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0f32 {
            (*(*NPC).client).ps.velocity[1] *= SENTRY_VELOCITY_DECAY;

            if fabs((*(*NPC).client).ps.velocity[1]) < 1.0f32 {
                (*(*NPC).client).ps.velocity[1] = 0.0f32;
            }
        }

        NPC_FaceEnemy(1);
    }
}

/*
-------------------------
Sentry_Idle
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_Idle() {
    unsafe {
        Sentry_MaintainHeight();

        // Is he waking up?
        // if (*NPCInfo).localState == LSTATE_WAKEUP {
        //     if (*(*NPC).client).ps.torsoAnimTimer <= 0 {
        //         (*NPCInfo).scriptFlags |= SCF_LOOK_FOR_ENEMIES;
        //         (*NPCInfo).burstCount = 0;
        //     }
        // } else {
            NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_SLEEP1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
            (*NPC).flags |= FL_SHIELDED;

            NPC_BSIdle();
        // }
    }
}

/*
-------------------------
Sentry_Strafe
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_Strafe() {
    unsafe {
        let mut dir: c_int = 0;
        let mut end: vec3_t = [0.0f32; 3];
        let mut right: vec3_t = [0.0f32; 3];
        let mut tr: trace_t = core::mem::zeroed();

        AngleVectors(&(*(*NPC).client).renderInfo.eyeAngles, core::ptr::null_mut(), &mut right, core::ptr::null_mut());

        // Pick a random strafe direction, then check to see if doing a strafe would be
        //	reasonable valid
        dir = if (rand() & 1) != 0 { -1 } else { 1 };
        VectorMA(&(*NPC).currentOrigin, (SENTRY_STRAFE_DIS * dir) as f32, &right, &mut end);

        // gi.trace(&mut tr, &(*NPC).currentOrigin, core::ptr::null(), core::ptr::null(), &end, (*NPC).s.number, MASK_SOLID);

        // Close enough
        // if tr.fraction > 0.9f32 {
        //     VectorMA(&(*(*NPC).client).ps.velocity, (SENTRY_STRAFE_VEL * dir) as f32, &right, &mut (*(*NPC).client).ps.velocity);
        //
        //     // Add a slight upward push
        //     (*(*NPC).client).ps.velocity[2] += SENTRY_UPWARD_PUSH as f32;
        //
        //     // Set the strafe start time so we can do a controlled roll
        //     (*NPC).fx_time = level.time;
        //     (*NPCInfo).standTime = level.time + 3000 + (random() * 500.0f32) as c_int;
        // }
    }
}

/*
-------------------------
Sentry_Hunt
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_Hunt(visible: qboolean, advance: qboolean) {
    unsafe {
        let mut distance: f32 = 0.0f32;
        let mut speed: f32 = 0.0f32;
        let mut forward: vec3_t = [0.0f32; 3];

        //If we're not supposed to stand still, pursue the player
        // if (*NPCInfo).standTime < level.time {
        //     // Only strafe when we can see the player
        //     if visible != 0 {
        //         Sentry_Strafe();
        //         return;
        //     }
        // }

        //If we don't want to advance, stop here
        if advance == 0 && visible != 0 {
            return;
        }

        //Only try and navigate if the player is visible
        if visible == 0 {
            // Move towards our goal
            // (*NPCInfo).goalEntity = (*NPC).enemy;
            // (*NPCInfo).goalRadius = 12;

            NPC_MoveToGoal(1);
            return;
        } else {
            VectorSubtract(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &mut forward);
            distance = VectorNormalize(&mut forward);
        }

        speed = SENTRY_FORWARD_BASE_SPEED as f32 + SENTRY_FORWARD_MULTIPLIER as f32 * (*g_spskill).integer as f32;
        VectorMA(&(*(*NPC).client).ps.velocity, speed, &forward, &mut (*(*NPC).client).ps.velocity);
    }
}

/*
-------------------------
Sentry_RangedAttack
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_RangedAttack(visible: qboolean, advance: qboolean) {
    unsafe {
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 && (*NPC).attackDebounceTime < level.time && visible != 0 {
            // Attack?
            // if (*NPCInfo).burstCount > 6 {
            //     if (*NPC).fly_sound_debounce_time == 0 {
            //         //delay closing down to give the player an opening
            //         (*NPC).fly_sound_debounce_time = level.time + Q_irand(500, 2000);
            //     } else if (*NPC).fly_sound_debounce_time < level.time {
            //         (*NPCInfo).localState = LSTATE_ACTIVE;
            //         (*NPC).fly_sound_debounce_time = 0;
            //         (*NPCInfo).burstCount = 0;
            //         TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(2000, 3500));
            //         (*NPC).flags |= FL_SHIELDED;
            //         NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_FLY_SHIELDED, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
            //         G_SoundOnEnt(NPC, CHAN_AUTO, b"sound/chars/sentry/misc/sentry_shield_close\0".as_ptr() as *const c_char);
            //     }
            // } else {
                Sentry_Fire();
            // }
        }

        // if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        //     Sentry_Hunt(visible, advance);
        // }
    }
}

/*
-------------------------
Sentry_AttackDecision
-------------------------
*/
#[no_mangle]
pub extern "C" fn Sentry_AttackDecision() {
    unsafe {
        // Always keep a good height off the ground
        Sentry_MaintainHeight();

        (*NPC).s.loopSound = G_SoundIndex(b"sound/chars/sentry/misc/sentry_hover_2_lp\0".as_ptr() as *const c_char);

        //randomly talk
        if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
            if TIMER_Done(NPC, b"angerNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt(NPC, CHAN_AUTO, va(b"sound/chars/sentry/misc/talk%d\0".as_ptr() as *const c_char, Q_irand(1, 3)));

                TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(4000, 10000));
            }
        }

        // He's dead.
        if (*(*NPC).enemy).health < 1 {
            (*NPC).enemy = core::ptr::null_mut();
            Sentry_Idle();
            return;
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt() == 0 {
            Sentry_Idle();
            return;
        }

        // Rate our distance to the target and visibilty
        let distance: f32 = DistanceHorizontalSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);
        let visible: qboolean = NPC_ClearLOS((*NPC).enemy);
        let advance: qboolean = if distance > MIN_DISTANCE_SQR as f32 { 1 } else { 0 };

        // If we cannot see our target, move to see it
        if visible == 0 {
            // if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            //     Sentry_Hunt(visible, advance);
            //     return;
            // }
        }

        NPC_FaceEnemy(1);

        Sentry_RangedAttack(visible, advance);
    }
}

/*
-------------------------
NPC_Sentry_Patrol
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_Sentry_Patrol() {
    unsafe {
        Sentry_MaintainHeight();

        //If we have somewhere to go, then do that
        if (*NPC).enemy.is_null() {
            if NPC_CheckPlayerTeamStealth() != 0 {
                //NPC_AngerSound();
                NPC_UpdateAngles(1, 1);
                return;
            }

            if UpdateGoal() != 0 {
                //start loop sound once we move
                // ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(1);
            }

            //randomly talk
            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt(NPC, CHAN_AUTO, va(b"sound/chars/sentry/misc/talk%d\0".as_ptr() as *const c_char, Q_irand(1, 3)));

                TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
            }
        }

        NPC_UpdateAngles(1, 1);
    }
}

/*
-------------------------
NPC_BSSentry_Default
-------------------------
*/
#[no_mangle]
pub extern "C" fn NPC_BSSentry_Default() {
    unsafe {
        if !(*NPC).targetname.is_null() {
            (*NPC).e_UseFunc = useF_sentry_use;
        }

        if !(*NPC).enemy.is_null() {
            // && (*NPCInfo).localState != LSTATE_WAKEUP {
            // Don't attack if waking up or if no enemy
            Sentry_AttackDecision();
        } else if (*NPCInfo as *const c_void as usize) != 0 {
            // if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            NPC_Sentry_Patrol();
        } else {
            Sentry_Idle();
        }
    }
}
