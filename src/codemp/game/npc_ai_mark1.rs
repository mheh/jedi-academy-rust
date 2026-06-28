//! Port of `NPC_AI_Mark1.c` — the Mark I droid's behavior state.
//!
//! The whole file is **live** in the PC MP module (the Xbox tree `#if 0`s these
//! NPC behaviors out — here on PC they ship). Faithful-first transcription; every
//! callee was already present in the Rust tree, so the entire file is ported.
//!
//! Ported: `NPC_Mark1_Precache` (NPC_AI_Mark1.c:50),
//! `NPC_Mark1_Part_Explode` (:81), `Mark1_Idle` (:109),
//! `Mark1Dead_FireRocket` (:123), `Mark1Dead_FireBlaster` (:171),
//! `Mark1_die` (:209), `Mark1_dying` (:250), `NPC_Mark1_Pain` (:320),
//! `Mark1_Hunt` (:404), `Mark1_FireBlaster` (:424), `Mark1_BlasterAttack` (:495),
//! `Mark1_FireRocket` (:555), `Mark1_RocketAttack` (:606),
//! `Mark1_AttackDecision` (:625), `Mark1_Patrol` (:711),
//! `NPC_BSMark1_Default` (:747).

#![allow(non_snake_case)] // C function names (`NPC_BSMark1_Default`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::addr_of;

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_DEATH1, BOTH_DEATH2, BOTH_PAIN1, BOTH_SLEEP1,
};
use crate::codemp::game::b_public_h::{SCF_LOOK_FOR_ENEMIES, SPOT_HEAD};
use crate::codemp::game::bg_misc::{
    BG_FindItemForAmmo, BG_FindItemForWeapon, BG_GiveMeVectorFromMatrix,
};
use crate::codemp::game::bg_public::{
    MASK_SHOT, MOD_BRYAR_PISTOL, MOD_ROCKET, MOD_UNKNOWN, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO, STAT_HEALTH,
};
use crate::codemp::game::bg_weapons_h::{
    AMMO_BLASTER, AMMO_METAL_BOLTS, WP_BOWCASTER, WP_BRYAR_PISTOL,
};
use crate::codemp::game::g_combat::{gPainHitLoc, G_Damage};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{
    gentity_t, DAMAGE_DEATH_KNOCKBACK, HL_ARM_LT, HL_ARM_RT, HL_CHEST, HL_GENERIC1,
};
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::codemp::game::npc::{
    ucmd, NPCInfo, NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC,
};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_SetSurfaceOnOff,
    NPC_UpdateAngles,
};
use crate::codemp::game::q_math::{
    vectoangles, AngleVectors, DistanceHorizontalSquared, Q_irand, VectorScale, VectorSet,
    VectorSubtract,
};
use crate::codemp::game::q_shared::va;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, NEGATIVE_Y, ORIGIN,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const TURN_OFF: c_int = 0x00000100;

const LEFT_ARM_HEALTH: c_int = 40;
const RIGHT_ARM_HEALTH: c_int = 40;
const AMMO_POD_HEALTH: c_int = 40;

const BOWCASTER_VELOCITY: f32 = 1300.0;
const BOWCASTER_SIZE: f32 = 2.0;
const BOWCASTER_SPLASH_DAMAGE: c_int = 0;
const BOWCASTER_SPLASH_RADIUS: c_int = 0;

// Local state enums (`ai.h` distance_e is not yet ported; the two values are kept
// verbatim as local consts).
const DIST_MELEE: c_int = 0;
const DIST_LONG: c_int = 1;

// Local state enums
const LSTATE_FIRED0: c_int = 3;
const LSTATE_FIRED1: c_int = 4;
const LSTATE_FIRED2: c_int = 5;
const LSTATE_FIRED3: c_int = 6;
const LSTATE_FIRED4: c_int = 7;

/*
-------------------------
NPC_Mark1_Precache
-------------------------
*/
pub unsafe fn NPC_Mark1_Precache() {
    G_SoundIndex("sound/chars/mark1/misc/mark1_wakeup");
    G_SoundIndex("sound/chars/mark1/misc/shutdown");
    G_SoundIndex("sound/chars/mark1/misc/walk");
    G_SoundIndex("sound/chars/mark1/misc/run");
    G_SoundIndex("sound/chars/mark1/misc/death1");
    G_SoundIndex("sound/chars/mark1/misc/death2");
    G_SoundIndex("sound/chars/mark1/misc/anger");
    G_SoundIndex("sound/chars/mark1/misc/mark1_fire");
    G_SoundIndex("sound/chars/mark1/misc/mark1_pain");
    G_SoundIndex("sound/chars/mark1/misc/mark1_explo");

    //	G_EffectIndex( "small_chunks");
    G_EffectIndex("env/med_explode2");
    G_EffectIndex("explosions/probeexplosion1");
    G_EffectIndex("blaster/smoke_bolton");
    G_EffectIndex("bryar/muzzle_flash");
    G_EffectIndex("explosions/droidexplosion1");

    RegisterItem(BG_FindItemForAmmo(AMMO_METAL_BOLTS));
    RegisterItem(BG_FindItemForAmmo(AMMO_BLASTER));
    RegisterItem(BG_FindItemForWeapon(WP_BOWCASTER));
    RegisterItem(BG_FindItemForWeapon(WP_BRYAR_PISTOL));
}

/*
-------------------------
NPC_Mark1_Part_Explode
-------------------------
*/
pub unsafe fn NPC_Mark1_Part_Explode(self_: *mut gentity_t, bolt: c_int) {
    if bolt >= 0 {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &(*self_).r.currentAngles,
            &(*self_).r.currentOrigin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*self_).modelScale,
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut dir);

        G_PlayEffectID(G_EffectIndex("env/med_explode2"), &org, &dir);

        G_PlayEffectID(G_EffectIndex("blaster/smoke_bolton"), &org, &dir);
    }

    //G_PlayEffectID( G_EffectIndex("blaster/smoke_bolton"), self->playerModel, bolt, self->s.number );
}

/*
-------------------------
Mark1_Idle
-------------------------
*/
pub unsafe fn Mark1_Idle() {
    NPC_BSIdle();

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_SLEEP1, SETANIM_FLAG_NORMAL);
}

/*
-------------------------
Mark1Dead_FireRocket
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub unsafe fn Mark1Dead_FireRocket() {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut muzzle_dir: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;

    let damage: c_int = 50;
    let bolt: c_int = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash5");

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        bolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut muzzle_dir);

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle1, &muzzle_dir);

    G_Sound(
        NPC,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark1/misc/mark1_fire"),
    );

    missile = CreateMissile(
        &mut muzzle1,
        &muzzle_dir,
        BOWCASTER_VELOCITY,
        10000,
        NPC,
        QFALSE,
    );

    (*missile).classname = c"bowcaster_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BOWCASTER;

    VectorSet(
        &mut (*missile).r.maxs,
        BOWCASTER_SIZE,
        BOWCASTER_SIZE,
        BOWCASTER_SIZE,
    );
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    //missile->methodOfDeath = MOD_ENERGY;
    (*missile).methodOfDeath = MOD_ROCKET;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    (*missile).splashDamage = BOWCASTER_SPLASH_DAMAGE;
    (*missile).splashRadius = BOWCASTER_SPLASH_RADIUS;

    // we don't want it to bounce
    (*missile).bounceCount = 0;
}

/*
-------------------------
Mark1Dead_FireBlaster
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub unsafe fn Mark1Dead_FireBlaster() {
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut muzzle_dir: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let bolt: c_int;

    bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash1");

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        bolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut muzzle_dir);

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle1, &muzzle_dir);

    missile = CreateMissile(&mut muzzle1, &muzzle_dir, 1600.0, 10000, NPC, QFALSE);

    G_Sound(
        NPC,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark1/misc/mark1_fire"),
    );

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = 1;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Mark1_die
-------------------------
*/
// The C die-func enum assignment (`NPC->e_DieFunc = dieF_Mark1_die;`) is commented
// out in the PC source, so this is compiled but not wired through the death pointer.
// Kept extern "C" / faithful 7-param signature.
pub unsafe extern "C" fn Mark1_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
    _dFlags: c_int,
    _hitLoc: c_int,
) {
    /*
    int	anim;

    // Is he dead already?
    anim = self->client->ps.legsAnim;
    if (((anim==BOTH_DEATH1) || (anim==BOTH_DEATH2)) && (self->client->ps.torsoTimer<=0))
    {	// This is because self->health keeps getting zeroed out. HL_NONE acts as health in this case.
        self->locationDamage[HL_NONE] += damage;
        if (self->locationDamage[HL_NONE] > 50)
        {
            DeathFX(self);
            self->client->ps.eFlags |= EF_NODRAW;
            self->contents = CONTENTS_CORPSE;
            // G_FreeEntity( self ); // Is this safe?  I can't see why we'd mark it nodraw and then just leave it around??
            self->e_ThinkFunc = thinkF_G_FreeEntity;
            self->nextthink = level.time + FRAMETIME;
        }
        return;
    }
    */

    G_Sound(
        self_,
        CHAN_AUTO,
        G_SoundIndex(&format!(
            "sound/chars/mark1/misc/death{}.wav",
            Q_irand(1, 2)
        )),
    );

    // Choose a death anim
    if Q_irand(1, 10) > 5 {
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_DEATH2,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    } else {
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_DEATH1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
}

/*
-------------------------
Mark1_dying
-------------------------
*/
pub unsafe fn Mark1_dying(self_: *mut gentity_t) {
    let mut num: c_int;
    let newBolt: c_int;

    if (*(*self_).client).ps.torsoTimer > 0 {
        if TIMER_Done(self_, c"dyingExplosion".as_ptr()) != QFALSE {
            num = Q_irand(1, 3);

            // Find place to generate explosion
            if num == 1 {
                num = Q_irand(8, 10);
                newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, &format!("*flash{}", num));
                NPC_Mark1_Part_Explode(self_, newBolt);
            } else {
                num = Q_irand(1, 6);
                newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, &format!("*torso_tube{}", num));
                NPC_Mark1_Part_Explode(self_, newBolt);
                NPC_SetSurfaceOnOff(self_, va(format_args!("torso_tube{}", num)), TURN_OFF);
            }

            TIMER_Set(self_, c"dyingExplosion".as_ptr(), Q_irand(300, 1000));
        }

        //		int		dir;
        //		vec3_t	right;

        // Shove to the side
        //		AngleVectors( self->client->renderInfo.eyeAngles, NULL, right, NULL );
        //		VectorMA( self->client->ps.velocity, -80, right, self->client->ps.velocity );

        // See which weapons are there
        // Randomly fire blaster
        if trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"l_arm".as_ptr()) == 0 {
            // Is the blaster still on the model?
            if Q_irand(1, 5) == 1 {
                SaveNPCGlobals();
                SetNPCGlobals(self_);
                Mark1Dead_FireBlaster();
                RestoreNPCGlobals();
            }
        }

        // Randomly fire rocket
        if trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"r_arm".as_ptr()) == 0 {
            // Is the rocket still on the model?
            if Q_irand(1, 10) == 1 {
                SaveNPCGlobals();
                SetNPCGlobals(self_);
                Mark1Dead_FireRocket();
                RestoreNPCGlobals();
            }
        }
    }
}

/*
-------------------------
NPC_Mark1_Pain
- look at what was hit and see if it should be removed from the model.
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark1_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    let newBolt: c_int;
    let hitLoc: c_int = *addr_of!(gPainHitLoc);

    NPC_Pain(self_, attacker, damage);

    G_Sound(
        self_,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark1/misc/mark1_pain"),
    );

    // Hit in the CHEST???
    if hitLoc == HL_CHEST {
        let chance: c_int = Q_irand(1, 4);

        if (chance == 1) && (damage > 5) {
            NPC_SetAnim(
                self_,
                SETANIM_BOTH,
                BOTH_PAIN1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
    }
    // Hit in the left arm?
    else if (hitLoc == HL_ARM_LT)
        && ((*self_).locationDamage[HL_ARM_LT as usize] > LEFT_ARM_HEALTH)
    {
        if (*self_).locationDamage[hitLoc as usize] >= LEFT_ARM_HEALTH {
            // Blow it up?
            newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*flash3");
            if newBolt != -1 {
                NPC_Mark1_Part_Explode(self_, newBolt);
            }

            NPC_SetSurfaceOnOff(self_, c"l_arm".as_ptr(), TURN_OFF);
        }
    }
    // Hit in the right arm?
    else if (hitLoc == HL_ARM_RT)
        && ((*self_).locationDamage[HL_ARM_RT as usize] > RIGHT_ARM_HEALTH)
    {
        // Blow it up?
        if (*self_).locationDamage[hitLoc as usize] >= RIGHT_ARM_HEALTH {
            newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*flash4");
            if newBolt != -1 {
                //				G_PlayEffect( "small_chunks", self->playerModel, self->genericBolt2, self->s.number);
                NPC_Mark1_Part_Explode(self_, newBolt);
            }

            NPC_SetSurfaceOnOff(self_, c"r_arm".as_ptr(), TURN_OFF);
        }
    }
    // Check ammo pods
    else {
        for i in 0..6 {
            if (hitLoc == HL_GENERIC1 + i)
                && ((*self_).locationDamage[(HL_GENERIC1 + i) as usize] > AMMO_POD_HEALTH)
            {
                // Blow it up?
                if (*self_).locationDamage[hitLoc as usize] >= AMMO_POD_HEALTH {
                    let newBolt =
                        trap::G2API_AddBolt((*self_).ghoul2, 0, &format!("*torso_tube{}", i + 1));
                    if newBolt != -1 {
                        NPC_Mark1_Part_Explode(self_, newBolt);
                    }
                    NPC_SetSurfaceOnOff(self_, va(format_args!("torso_tube{}", i + 1)), TURN_OFF);
                    NPC_SetAnim(
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN1,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    break;
                }
            }
        }
    }

    // Are both guns shot off?
    if (trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"l_arm".as_ptr()) > 0)
        && (trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"r_arm".as_ptr()) > 0)
    {
        G_Damage(
            self_,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (*self_).health,
            0,
            MOD_UNKNOWN,
        );
    }
}

/*
-------------------------
Mark1_Hunt
- look for enemy.
-------------------------
*/
pub unsafe fn Mark1_Hunt() {
    if (*NPCInfo).goalEntity.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
    }

    NPC_FaceEnemy(QTRUE);

    (*NPCInfo).combatMove = QTRUE;
    NPC_MoveToGoal(QTRUE);
}

/*
-------------------------
Mark1_FireBlaster
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub unsafe fn Mark1_FireBlaster() {
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    // C declares `static vec3_t forward, vright, up;` but the static storage is
    // dead (recomputed via AngleVectors every call) — plain locals, matching the
    // `Sentry_Fire`/`Remote_Fire` precedent (avoids static_mut_refs).
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let bolt: c_int;

    // Which muzzle to fire from?
    if ((*NPCInfo).localState <= LSTATE_FIRED0) || ((*NPCInfo).localState == LSTATE_FIRED4) {
        (*NPCInfo).localState = LSTATE_FIRED1;
        bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash1");
    } else if (*NPCInfo).localState == LSTATE_FIRED1 {
        (*NPCInfo).localState = LSTATE_FIRED2;
        bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash2");
    } else if (*NPCInfo).localState == LSTATE_FIRED2 {
        (*NPCInfo).localState = LSTATE_FIRED3;
        bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash3");
    } else {
        (*NPCInfo).localState = LSTATE_FIRED4;
        bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash4");
    }

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        bolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);

    if (*NPC).health != 0 {
        CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org1);
        VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
        vectoangles(&delta1, &mut angleToEnemy1);
        AngleVectors(
            &angleToEnemy1,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    } else {
        AngleVectors(
            &(*NPC).r.currentAngles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    }

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle1, &forward);

    G_Sound(
        NPC,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark1/misc/mark1_fire"),
    );

    missile = CreateMissile(&mut muzzle1, &forward, 1600.0, 10000, NPC, QFALSE);

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = 1;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Mark1_BlasterAttack
-------------------------
*/
pub unsafe fn Mark1_BlasterAttack(advance: qboolean) {
    let mut chance: c_int;

    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
        // Attack?
        chance = Q_irand(1, 5);

        (*NPCInfo).burstCount += 1;

        if (*NPCInfo).burstCount < 3 {
            // Too few shots this burst?
            chance = 2; // Force it to keep firing.
        } else if (*NPCInfo).burstCount > 12 {
            // Too many shots fired this burst?
            (*NPCInfo).burstCount = 0;
            chance = 1; // Force it to stop firing.
        }

        // Stop firing.
        if chance == 1 {
            (*NPCInfo).burstCount = 0;
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(1000, 3000));
            (*(*NPC).client).ps.torsoTimer = 0; // Just in case the firing anim is running.
        } else {
            if TIMER_Done(NPC, c"attackDelay2".as_ptr()) != QFALSE {
                // Can't be shooting every frame.
                TIMER_Set(NPC, c"attackDelay2".as_ptr(), Q_irand(50, 50));
                Mark1_FireBlaster();
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
            return;
        }
    } else if advance != QFALSE {
        if (*(*NPC).client).ps.torsoAnim == BOTH_ATTACK1 {
            (*(*NPC).client).ps.torsoTimer = 0; // Just in case the firing anim is running.
        }
        Mark1_Hunt();
    } else {
        // Make sure he's not firing.
        if (*(*NPC).client).ps.torsoAnim == BOTH_ATTACK1 {
            (*(*NPC).client).ps.torsoTimer = 0; // Just in case the firing anim is running.
        }
    }
}

/*
-------------------------
Mark1_FireRocket
-------------------------
*/
pub unsafe fn Mark1_FireRocket() {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    // C declares `static vec3_t forward, vright, up;` — dead static storage, plain
    // locals (recomputed every call via AngleVectors).
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let bolt: c_int = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash5");
    let missile: *mut gentity_t;

    let damage: c_int = 50;

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        bolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);

    //	G_PlayEffect( "blaster/muzzle_flash", muzzle1 );

    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org1);
    VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
    vectoangles(&delta1, &mut angleToEnemy1);
    AngleVectors(
        &angleToEnemy1,
        Some(&mut forward),
        Some(&mut vright),
        Some(&mut up),
    );

    G_Sound(
        NPC,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark1/misc/mark1_fire"),
    );

    missile = CreateMissile(
        &mut muzzle1,
        &forward,
        BOWCASTER_VELOCITY,
        10000,
        NPC,
        QFALSE,
    );

    (*missile).classname = c"bowcaster_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BOWCASTER;

    VectorSet(
        &mut (*missile).r.maxs,
        BOWCASTER_SIZE,
        BOWCASTER_SIZE,
        BOWCASTER_SIZE,
    );
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_ROCKET;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    (*missile).splashDamage = BOWCASTER_SPLASH_DAMAGE;
    (*missile).splashRadius = BOWCASTER_SPLASH_RADIUS;

    // we don't want it to bounce
    (*missile).bounceCount = 0;
}

/*
-------------------------
Mark1_RocketAttack
-------------------------
*/
pub unsafe fn Mark1_RocketAttack(advance: qboolean) {
    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
        // Attack?
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(1000, 3000));
        NPC_SetAnim(
            NPC,
            SETANIM_TORSO,
            BOTH_ATTACK2,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        Mark1_FireRocket();
    } else if advance != QFALSE {
        Mark1_Hunt();
    }
}

/*
-------------------------
Mark1_AttackDecision
-------------------------
*/
pub unsafe fn Mark1_AttackDecision() {
    let blasterTest: c_int;
    let rocketTest: c_int;
    let distance: f32;
    let mut distRate: c_int;
    let visible: qboolean;
    let advance: qboolean;

    //randomly talk
    if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
        if TIMER_Done(NPC, c"angerNoise".as_ptr()) != QFALSE {
            //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));
            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(4000, 10000));
        }
    }

    // Enemy is dead or he has no enemy.
    if ((*(*NPC).enemy).health < 1) || (NPC_CheckEnemyExt(QFALSE) == QFALSE) {
        (*NPC).enemy = core::ptr::null_mut();
        return;
    }

    // Rate our distance to the target and visibility
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    distRate = if distance > MIN_MELEE_RANGE_SQR as f32 {
        DIST_LONG
    } else {
        DIST_MELEE
    };
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    // If we cannot see our target, move to see it
    if (visible == QFALSE) || (NPC_FaceEnemy(QTRUE) == QFALSE) {
        Mark1_Hunt();
        return;
    }

    // See if the side weapons are there
    blasterTest = trap::G2API_GetSurfaceRenderStatus((*NPC).ghoul2, 0, c"l_arm".as_ptr());
    rocketTest = trap::G2API_GetSurfaceRenderStatus((*NPC).ghoul2, 0, c"r_arm".as_ptr());

    // It has both side weapons
    if blasterTest == 0 && rocketTest == 0 {
        // So do nothing.
    } else if blasterTest != -1 && blasterTest != 0 {
        distRate = DIST_LONG;
    } else if rocketTest != -1 && rocketTest != 0 {
        distRate = DIST_MELEE;
    } else {
        // It should never get here, but just in case
        (*NPC).health = 0;
        (*(*NPC).client).ps.stats[STAT_HEALTH as usize] = 0;
        //GEntity_DieFunc(NPC, NPC, NPC, 100, MOD_UNKNOWN);
        if let Some(die) = (*NPC).die {
            die(NPC, NPC, NPC, 100, MOD_UNKNOWN);
        }
    }

    // We can see enemy so shoot him if timers let you.
    NPC_FaceEnemy(QTRUE);

    if distRate == DIST_MELEE {
        Mark1_BlasterAttack(advance);
    } else if distRate == DIST_LONG {
        Mark1_RocketAttack(advance);
    }
}

/*
-------------------------
Mark1_Patrol
-------------------------
*/
pub unsafe fn Mark1_Patrol() {
    if NPC_CheckPlayerTeamStealth() != QFALSE {
        G_Sound(
            NPC,
            CHAN_AUTO,
            G_SoundIndex("sound/chars/mark1/misc/mark1_wakeup"),
        );
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if !UpdateGoal().is_null() {
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
            NPC_UpdateAngles(QTRUE, QTRUE);
        }

        //randomly talk
        //		if (TIMER_Done(NPC,"patrolNoise"))
        //		{
        //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));
        //
        //			TIMER_Set( NPC, "patrolNoise", Q_irand( 2000, 4000 ) );
        //		}
    }
}

/*
-------------------------
NPC_BSMark1_Default
-------------------------
*/
pub unsafe fn NPC_BSMark1_Default() {
    //NPC->e_DieFunc = dieF_Mark1_die;

    if !(*NPC).enemy.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        Mark1_AttackDecision();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Mark1_Patrol();
    } else {
        Mark1_Idle();
    }
}
