//! Faithful port of `NPC_AI_GalakMech.c` — the GALAK Mech boss NPC's behavior
//! state. (Ported from the PC tree `refs/raven-jediacademy`, where this file is
//! live; the Xbox `refs/raven-jediacademy` copy is `#if 0` dead code.)
//!
//! The whole MP-compiled file is landed here: the precache/init entry points,
//! the explosion/death helpers, the pain handler, the hold-position / move
//! helpers, the patrol entry, the check-move / check-fire state helpers, the
//! laser-start and gloat helpers, and the attack / default behavior-state entry
//! points. Several large blocks inside these functions are `#if 0`/`if (0)` in
//! the C source (shield powerups, victory anims, melee smackdown) and are
//! carried over verbatim as comments / dead branches.

#![allow(non_snake_case)] // C function names (`NPC_GalakMech_Init`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK4, BOTH_KNOCKDOWN1, BOTH_KNOCKDOWN4, BOTH_KNOCKDOWN5,
    BOTH_STAND2TO1,
};
// BOTH_ATTACK5 only appears in `#if 0` commented melee code below; not imported.
use crate::codemp::game::b_public_h::{
    BS_CINEMATIC, SCF_ALT_FIRE, SCF_CHASE_ENEMIES, SCF_DONT_FIRE, SCF_FIRE_WEAPON, SPOT_HEAD,
    SPOT_WEAPON,
};
use crate::codemp::game::bg_panimate::BG_CrouchAnim;
use crate::codemp::game::bg_public::{
    BG_GiveMeVectorFromMatrix, EV_ANGER1, EV_ANGER2, EV_ANGER3, EV_CHASE1, EV_COVER1, EV_DETECTED1,
    EV_ESCAPING1, EV_PUSHED1, EV_PUSHED2, EV_PUSHED3, MASK_SHOT, MOD_CRUSH, MOD_REPEATER,
    MOD_REPEATER_ALT, MOD_UNKNOWN, PW_BATTLESUIT, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    SETANIM_FLAG_OVERRIDE, STAT_ARMOR,
};
use crate::codemp::game::bg_weapons_h::{WP_NONE, WP_REPEATER, WP_SABER, WP_TURRET};
use crate::codemp::game::g_combat::{gPainMOD, gPainPoint, G_Damage};
use crate::codemp::game::g_local::{
    gentity_t, DAMAGE_NO_ARMOR, DAMAGE_NO_KNOCKBACK, FL_NO_KNOCKBACK, FL_SHIELDED, FRAMETIME,
    HL_GENERIC1,
};
use crate::codemp::game::g_main::{g_entities, g_spskill, level};
use crate::codemp::game::g_nav::{navInfo_t, NAV_HitNavGoal, NIF_COLLISION};
use crate::codemp::game::g_public_h::{SVF_BROADCAST, TID_MOVE_NAV};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_FreeEntity, G_PlayEffectID, G_SetOrigin, G_Sound, G_SoundAtLoc, G_SoundIndex,
    G_SoundOnEnt, G_Spawn, G_Throw,
};
use crate::codemp::game::g_weapon::WP_LobFire;
use crate::codemp::game::w_saber_h::ARMOR_EFFECT_TIME;
use crate::codemp::game::npc::{ucmd, NPC_SetAnim, NPC, NPCInfo};
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_combat::{
    NPC_AimAdjust, NPC_ChangeWeapon, NPC_FreeCombatPoint, NPC_ShotEntity, WeaponThink,
};
use crate::codemp::game::npc_goal::{G_BoundsOverlap, NPC_ReachedGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NAV_GetLastMove, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::{NPC_Pain, NPC_SetPainEvent};
use crate::codemp::game::npc_senses::InFront;
use crate::codemp::game::npc_sounds::G_AddVoiceEvent;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_SetSurfaceOnOff,
    NPC_UpdateAngles,
};
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleNormalize360, AngleVectors, DistanceSquared,
    VectorClear, VectorCompare, VectorCopy, VectorMA, VectorNormalize, VectorSet, VectorSubtract,
    Q_irand,
};
use crate::codemp::game::q_shared::{crandom, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, trace_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, NEGATIVE_Y, ORIGIN, PITCH, YAW,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// Per-frame attack working set, shared between `NPC_BSGM_Attack` and its
// `GM_CheckMoveState` / `GM_CheckFireState` helpers (C file-scope statics).
static mut enemyLOS4: c_int = 0;
static mut enemyCS4: c_int = 0;
static mut hitAlly4: c_int = 0;
static mut faceEnemy4: c_int = 0;
static mut move4: c_int = 0;
static mut shoot4: c_int = 0;
static mut enemyDist4: f32 = 0.0;
static mut impactPos4: vec3_t = [0.0, 0.0, 0.0];

static mut shieldMins: vec3_t = [-60.0, -60.0, -24.0];
static mut shieldMaxs: vec3_t = [60.0, 60.0, 80.0];

// #define MELEE_DIST_SQUARED 6400//80*80
const MELEE_DIST_SQUARED: f32 = 6400.0;
// #define MIN_LOB_DIST_SQUARED 65536//256*256
const MIN_LOB_DIST_SQUARED: f32 = 65536.0;
// #define MAX_LOB_DIST_SQUARED 200704//448*448
const MAX_LOB_DIST_SQUARED: f32 = 200704.0;
// #define REPEATER_ALT_SIZE 3 // half of bbox size
const REPEATER_ALT_SIZE: f32 = 3.0;
// #define GENERATOR_HEALTH 25
const GENERATOR_HEALTH: c_int = 25;
// #define TURN_ON 0x00000000
const TURN_ON: c_int = 0x00000000;
// #define TURN_OFF 0x00000100
const TURN_OFF: c_int = 0x00000100;
// #define GALAK_SHIELD_HEALTH 500
const GALAK_SHIELD_HEALTH: c_int = 500;

pub unsafe fn NPC_GalakMech_Precache() {
    G_SoundIndex("sound/weapons/galak/skewerhit.wav");
    G_SoundIndex("sound/weapons/galak/lasercharge.wav");
    G_SoundIndex("sound/weapons/galak/lasercutting.wav");
    G_SoundIndex("sound/weapons/galak/laserdamage.wav");

    G_EffectIndex("galak/trace_beam");
    G_EffectIndex("galak/beam_warmup");
    //	G_EffectIndex( "small_chunks");
    G_EffectIndex("env/med_explode2");
    G_EffectIndex("env/small_explode2");
    G_EffectIndex("galak/explode");
    G_EffectIndex("blaster/smoke_bolton");
    //	G_EffectIndex( "env/exp_trail_comp");
}

pub unsafe fn NPC_GalakMech_Init(ent: *mut gentity_t) {
    if (*(*ent).NPC).behaviorState != BS_CINEMATIC {
        (*(*ent).client).ps.stats[STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
        (*(*ent).NPC).investigateCount = 0;
        (*(*ent).NPC).investigateDebounceTime = 0;
        (*ent).flags |= FL_SHIELDED; //reflect normal shots
        //rwwFIXMEFIXME: Support PW_GALAK_SHIELD
        //ent->client->ps.powerups[PW_GALAK_SHIELD] = Q3_INFINITE;//temp, for effect
        //ent->fx_time = level.time;
        VectorSet(&mut (*ent).r.mins, -60.0, -60.0, -24.0);
        VectorSet(&mut (*ent).r.maxs, 60.0, 60.0, 80.0);
        (*ent).flags |= FL_NO_KNOCKBACK; //don't get pushed
        TIMER_Set(ent, c"attackDelay".as_ptr(), 0); //FIXME: Slant for difficulty levels
        TIMER_Set(ent, c"flee".as_ptr(), 0);
        TIMER_Set(ent, c"smackTime".as_ptr(), 0);
        TIMER_Set(ent, c"beamDelay".as_ptr(), 0);
        TIMER_Set(ent, c"noLob".as_ptr(), 0);
        TIMER_Set(ent, c"noRapid".as_ptr(), 0);
        TIMER_Set(ent, c"talkDebounce".as_ptr(), 0);

        NPC_SetSurfaceOnOff(ent, c"torso_shield".as_ptr(), TURN_ON);
        NPC_SetSurfaceOnOff(ent, c"torso_galakface".as_ptr(), TURN_OFF);
        NPC_SetSurfaceOnOff(ent, c"torso_galakhead".as_ptr(), TURN_OFF);
        NPC_SetSurfaceOnOff(ent, c"torso_eyes_mouth".as_ptr(), TURN_OFF);
        NPC_SetSurfaceOnOff(ent, c"torso_collar".as_ptr(), TURN_OFF);
        NPC_SetSurfaceOnOff(ent, c"torso_galaktorso".as_ptr(), TURN_OFF);
    } else {
        //		NPC_SetSurfaceOnOff( ent, "helmet", TURN_OFF );
        NPC_SetSurfaceOnOff(ent, c"torso_shield".as_ptr(), TURN_OFF);
        NPC_SetSurfaceOnOff(ent, c"torso_galakface".as_ptr(), TURN_ON);
        NPC_SetSurfaceOnOff(ent, c"torso_galakhead".as_ptr(), TURN_ON);
        NPC_SetSurfaceOnOff(ent, c"torso_eyes_mouth".as_ptr(), TURN_ON);
        NPC_SetSurfaceOnOff(ent, c"torso_collar".as_ptr(), TURN_ON);
        NPC_SetSurfaceOnOff(ent, c"torso_galaktorso".as_ptr(), TURN_ON);
    }
}

//-----------------------------------------------------------------
unsafe fn GM_CreateExplosion(self_: *mut gentity_t, boltID: c_int, doSmall: qboolean) {
    //doSmall = qfalse
    if boltID >= 0 {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            boltID,
            &mut boltMatrix,
            &(*self_).r.currentAngles,
            &(*self_).r.currentOrigin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*self_).modelScale,
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut dir);

        if doSmall != 0 {
            G_PlayEffectID(G_EffectIndex("env/small_explode2"), &org, &dir);
        } else {
            G_PlayEffectID(G_EffectIndex("env/med_explode2"), &org, &dir);
        }
    }
}

/*
-------------------------
GM_Dying
-------------------------
*/

pub unsafe fn GM_Dying(self_: *mut gentity_t) {
    if (*addr_of!(level)).time - (*self_).s.time < 4000 {
        //FIXME: need a real effect
        //self->s.powerups |= ( 1 << PW_SHOCKED );
        //self->client->ps.powerups[PW_SHOCKED] = level.time + 1000;
        (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + 1000;
        if TIMER_Done(self_, c"dyingExplosion".as_ptr()) != QFALSE {
            // Find place to generate explosion
            match Q_irand(1, 14) {
                1 => {
                    if trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"r_hand".as_ptr())
                        == 0
                    {
                        //r_hand still there
                        GM_CreateExplosion(
                            self_,
                            trap::G2API_AddBolt((*self_).ghoul2, 0, "*flasha"),
                            QTRUE,
                        );
                        NPC_SetSurfaceOnOff(self_, c"r_hand".as_ptr(), TURN_OFF);
                    } else if trap::G2API_GetSurfaceRenderStatus(
                        (*self_).ghoul2,
                        0,
                        c"r_arm_middle".as_ptr(),
                    ) == 0
                    {
                        //r_arm_middle still there
                        let _newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_arm_elbow");
                        NPC_SetSurfaceOnOff(self_, c"r_arm_middle".as_ptr(), TURN_OFF);
                    }
                }
                2 => {
                    //FIXME: do only once?
                    if trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"l_hand".as_ptr())
                        == 0
                    {
                        //l_hand still there
                        GM_CreateExplosion(
                            self_,
                            trap::G2API_AddBolt((*self_).ghoul2, 0, "*flashc"),
                            QFALSE,
                        );
                        NPC_SetSurfaceOnOff(self_, c"l_hand".as_ptr(), TURN_OFF);
                    } else if trap::G2API_GetSurfaceRenderStatus(
                        (*self_).ghoul2,
                        0,
                        c"l_arm_wrist".as_ptr(),
                    ) == 0
                    {
                        //l_arm_wrist still there
                        let _newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_arm_cap_l_hand");
                        NPC_SetSurfaceOnOff(self_, c"l_arm_wrist".as_ptr(), TURN_OFF);
                    } else if trap::G2API_GetSurfaceRenderStatus(
                        (*self_).ghoul2,
                        0,
                        c"l_arm_middle".as_ptr(),
                    ) == 0
                    {
                        //l_arm_middle still there
                        let _newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_arm_cap_l_hand");
                        NPC_SetSurfaceOnOff(self_, c"l_arm_middle".as_ptr(), TURN_OFF);
                    } else if trap::G2API_GetSurfaceRenderStatus(
                        (*self_).ghoul2,
                        0,
                        c"l_arm_augment".as_ptr(),
                    ) == 0
                    {
                        //l_arm_augment still there
                        let _newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_arm_elbow");
                        NPC_SetSurfaceOnOff(self_, c"l_arm_augment".as_ptr(), TURN_OFF);
                    }
                }
                3 | 4 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*hip_fr");
                    GM_CreateExplosion(self_, newBolt, QFALSE);
                }
                5 | 6 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*shldr_l");
                    GM_CreateExplosion(self_, newBolt, QFALSE);
                }
                7 | 8 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*uchest_r");
                    GM_CreateExplosion(self_, newBolt, QFALSE);
                }
                9 | 10 => {
                    GM_CreateExplosion(self_, (*(*self_).client).renderInfo.headBolt, QFALSE);
                }
                11 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_leg_knee");
                    GM_CreateExplosion(self_, newBolt, QTRUE);
                }
                12 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_leg_knee");
                    GM_CreateExplosion(self_, newBolt, QTRUE);
                }
                13 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_leg_foot");
                    GM_CreateExplosion(self_, newBolt, QTRUE);
                }
                14 => {
                    let newBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_leg_foot");
                    GM_CreateExplosion(self_, newBolt, QTRUE);
                }
                _ => {}
            }

            TIMER_Set(self_, c"dyingExplosion".as_ptr(), Q_irand(300, 1100));
        }
    } else {
        //one final, huge explosion
        G_PlayEffectID(
            G_EffectIndex("galak/explode"),
            &(*self_).r.currentOrigin,
            &vec3_origin,
        );
        //		G_PlayEffect( "small_chunks", self->r.currentOrigin );
        //		G_PlayEffect( "env/exp_trail_comp", self->r.currentOrigin, self->currentAngles );
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
        (*self_).think = Some(G_FreeEntity);
    }
}

/*
-------------------------
NPC_GM_Pain
-------------------------
*/

pub unsafe extern "C" fn NPC_GM_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let mut point: vec3_t = [0.0; 3];
    let inflictor: *mut gentity_t = attacker;
    let hitLoc: c_int = 1;
    let mod_: c_int = *addr_of!(gPainMOD);

    VectorCopy(&*addr_of!(gPainPoint), &mut point);

    //if ( self->client->ps.powerups[PW_GALAK_SHIELD] == 0 )
    if false
    //rwwFIXMEFIXME: do all of this
    {
        //shield is currently down
        //FIXME: allow for radius damage?
        /*
        if ( (hitLoc==HL_GENERIC1) && (self->locationDamage[HL_GENERIC1] > GENERATOR_HEALTH) )
        {
            int newBolt = gi.G2API_AddBolt( &self->ghoul2[self->playerModel], "*antenna_base" );
            if ( newBolt != -1 )
            {
                GM_CreateExplosion( self, newBolt, qfalse );
            }

            NPC_SetSurfaceOnOff( self, "torso_shield", TURN_OFF );
            NPC_SetSurfaceOnOff( self, "torso_antenna", TURN_OFF );
            NPC_SetSurfaceOnOff( self, "torso_antenna_base_cap", TURN_ON );
            self->client->ps.powerups[PW_GALAK_SHIELD] = 0;//temp, for effect
            self->client->ps.stats[STAT_ARMOR] = 0;//no more armor
            self->NPC->investigateDebounceTime = 0;//stop recharging

            NPC_SetAnim( self, SETANIM_BOTH, BOTH_ALERT1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            TIMER_Set( self, "attackDelay", self->client->ps.torsoTimer );
            G_AddEvent( self, Q_irand( EV_DEATH1, EV_DEATH3 ), self->health );
        }
        */
    } else {
        //store the point for shield impact
        //if ( point )
        //{
        //	VectorCopy( point, self->pos4 );
        //	self->client->poisonTime = level.time;
        //rwwFIXMEFIXME: ..do this is as well.
        //}
    }

    if (*self_).lockCount == 0 && (*(*self_).client).ps.torsoTimer <= 0 {
        //don't interrupt laser sweep attack or other special attacks/moves
        if (*self_).count < 4 && (*self_).health > 100 && hitLoc != HL_GENERIC1 {
            if (*self_).delay < (*addr_of!(level)).time {
                let speech: c_int;
                match (*self_).count {
                    1 => {
                        speech = EV_PUSHED2;
                    }
                    2 => {
                        speech = EV_PUSHED3;
                    }
                    3 => {
                        speech = EV_DETECTED1;
                    }
                    //default & 0
                    _ => {
                        speech = EV_PUSHED1;
                    }
                }
                (*self_).count += 1;
                (*(*self_).NPC).blockedSpeechDebounceTime = 0;
                G_AddVoiceEvent(self_, speech, Q_irand(3000, 5000));
                (*self_).delay = (*addr_of!(level)).time + Q_irand(5000, 7000);
            }
        } else {
            NPC_Pain(self_, attacker, damage);
        }
    } else if hitLoc == HL_GENERIC1 {
        NPC_SetPainEvent(self_);
        //self->s.powerups |= ( 1 << PW_SHOCKED );
        //self->client->ps.powerups[PW_SHOCKED] = level.time + Q_irand( 500, 2500 );
        (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + Q_irand(500, 2500);
    }

    if !inflictor.is_null() && (*inflictor).lastEnemy == self_ {
        //He force-pushed my own lobfires back at me
        if mod_ == MOD_REPEATER_ALT && Q_irand(0, 2) == 0 {
            if TIMER_Done(self_, c"noRapid".as_ptr()) != QFALSE {
                (*(*self_).NPC).scriptFlags &= !SCF_ALT_FIRE;
                (*self_).alt_fire = QFALSE;
                TIMER_Set(self_, c"noLob".as_ptr(), Q_irand(2000, 6000));
            } else {
                //hopefully this will make us fire the laser
                TIMER_Set(self_, c"noLob".as_ptr(), Q_irand(1000, 2000));
            }
        } else if mod_ == MOD_REPEATER && Q_irand(0, 5) == 0 {
            if TIMER_Done(self_, c"noLob".as_ptr()) != QFALSE {
                (*(*self_).NPC).scriptFlags |= SCF_ALT_FIRE;
                (*self_).alt_fire = QTRUE;
                TIMER_Set(self_, c"noRapid".as_ptr(), Q_irand(2000, 6000));
            } else {
                //hopefully this will make us fire the laser
                TIMER_Set(self_, c"noRapid".as_ptr(), Q_irand(1000, 2000));
            }
        }
    }
}

/*
-------------------------
GM_HoldPosition
-------------------------
*/

unsafe fn GM_HoldPosition() {
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, QTRUE);
    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE {
        //don't have a script waiting for me to get to my point, okay to stop trying and stand
        (*NPCInfo).goalEntity = core::ptr::null_mut();
    }
}

/*
-------------------------
GM_Move
-------------------------
*/
unsafe fn GM_Move() -> qboolean {
    let moved: qboolean;
    let mut info: navInfo_t = core::mem::zeroed();

    (*NPCInfo).combatMove = QTRUE; //always move straight toward our goal

    moved = NPC_MoveToGoal(QTRUE);

    //Get the move info
    NAV_GetLastMove(&mut info);

    //FIXME: if we bump into another one of our guys and can't get around him, just stop!
    //If we hit our target, then stop and fire!
    if info.flags & NIF_COLLISION != 0 {
        if info.blocker == (*NPC).enemy {
            GM_HoldPosition();
        }
    }

    //If our move failed, then reset
    if moved == QFALSE {
        //FIXME: if we're going to a combat point, need to pick a different one
        if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE {
            //can't transfer movegoal or stop when a script we're running is waiting to complete
            GM_HoldPosition();
        }
    }

    moved
}

/*
-------------------------
NPC_BSGM_Patrol
-------------------------
*/

pub unsafe fn NPC_BSGM_Patrol() {
    if NPC_CheckPlayerTeamStealth() != QFALSE {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
GM_CheckMoveState
-------------------------
*/

unsafe fn GM_CheckMoveState() {
    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) != QFALSE {
        //moving toward a goal that a script is waiting on, so don't stop for anything!
        move4 = QTRUE;
    }

    //See if we're moving towards a goal, not the enemy
    if (*NPCInfo).goalEntity != (*NPC).enemy && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if NAV_HitNavGoal(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            16,
            QFALSE,
        ) != QFALSE
            || (trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
                && enemyLOS4 != 0
                && enemyDist4 <= 10000.0)
        {
            //either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a combat point) and we're scouting and found our enemy
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(250, 500)); //FIXME: Slant for difficulty levels
        }
    }
}

/*
-------------------------
GM_CheckFireState
-------------------------
*/

unsafe fn GM_CheckFireState() {
    if enemyCS4 != 0 {
        //if have a clear shot, always try
        return;
    }

    if VectorCompare(&(*(*NPC).client).ps.velocity, &vec3_origin) == 0 {
        //if moving at all, don't do this
        return;
    }

    //See if we should continue to fire on their last position
    if hitAlly4 == 0 && (*NPCInfo).enemyLastSeenTime > 0 {
        if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime < 10000 {
            if Q_irand(0, 10) == 0 {
                //Fire on the last known position
                let mut muzzle: vec3_t = [0.0; 3];
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let mut tooClose: qboolean = QFALSE;
                let mut tooFar: qboolean = QFALSE;
                let mut distThreshold: f32;
                let mut dist: f32;

                CalcEntitySpot(NPC, SPOT_HEAD, &mut muzzle);
                if VectorCompare(&*addr_of!(impactPos4), &vec3_origin) != 0 {
                    //never checked ShotEntity this frame, so must do a trace...
                    //vec3_t	mins = {-2,-2,-2}, maxs = {2,2,2};
                    let mut forward: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    AngleVectors(&(*(*NPC).client).ps.viewangles, Some(&mut forward), None, None);
                    VectorMA(&muzzle, 8192.0, &forward, &mut end);
                    let tr = trap::Trace(
                        &muzzle,
                        &vec3_origin,
                        &vec3_origin,
                        &end,
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    VectorCopy(&tr.endpos, &mut *addr_of_mut!(impactPos4));
                }

                //see if impact would be too close to me
                distThreshold = 16384.0; /*128*128*/ //default
                if (*NPC).s.weapon == WP_REPEATER {
                    if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                        distThreshold = 65536.0; /*256*256*/
                    }
                }

                dist = DistanceSquared(&*addr_of!(impactPos4), &muzzle);

                if dist < distThreshold {
                    //impact would be too close to me
                    tooClose = QTRUE;
                } else if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime > 5000 {
                    //we've haven't seen them in the last 5 seconds
                    //see if it's too far from where he is
                    distThreshold = 65536.0; /*256*256*/ //default
                    if (*NPC).s.weapon == WP_REPEATER {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                            distThreshold = 262144.0; /*512*512*/
                        }
                    }
                    dist =
                        DistanceSquared(&*addr_of!(impactPos4), &(*NPCInfo).enemyLastSeenLocation);
                    if dist > distThreshold {
                        //impact would be too far from enemy
                        tooFar = QTRUE;
                    }
                }

                if tooClose == 0 && tooFar == 0 {
                    //okay too shoot at last pos
                    VectorSubtract(&(*NPCInfo).enemyLastSeenLocation, &muzzle, &mut dir);
                    VectorNormalize(&mut dir);
                    vectoangles(&dir, &mut angles);

                    (*NPCInfo).desiredYaw = angles[YAW as usize];
                    (*NPCInfo).desiredPitch = angles[PITCH as usize];

                    shoot4 = QTRUE;
                    faceEnemy4 = QFALSE;
                }
            }
        }
    }
}

pub unsafe fn NPC_GM_StartLaser() {
    if (*NPC).lockCount == 0 {
        //haven't already started a laser attack
        //warm up for the beam attack
        //#if 0
        //		NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_RAISEWEAP2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
        //#endif
        TIMER_Set(NPC, c"beamDelay".as_ptr(), (*(*NPC).client).ps.torsoTimer);
        TIMER_Set(
            NPC,
            c"attackDelay".as_ptr(),
            (*(*NPC).client).ps.torsoTimer + 3000,
        );
        (*NPC).lockCount = 1;
        //turn on warmup effect
        G_PlayEffectID(
            G_EffectIndex("galak/beam_warmup"),
            &(*NPC).r.currentOrigin,
            &vec3_origin,
        );
        G_SoundOnEnt(NPC, CHAN_AUTO, "sound/weapons/galak/lasercharge.wav");
    }
}

pub unsafe fn GM_StartGloat() {
    (*NPC).wait = 0.0;
    NPC_SetSurfaceOnOff(NPC, c"torso_galakface".as_ptr(), TURN_ON);
    NPC_SetSurfaceOnOff(NPC, c"torso_galakhead".as_ptr(), TURN_ON);
    NPC_SetSurfaceOnOff(NPC, c"torso_eyes_mouth".as_ptr(), TURN_ON);
    NPC_SetSurfaceOnOff(NPC, c"torso_collar".as_ptr(), TURN_ON);
    NPC_SetSurfaceOnOff(NPC, c"torso_galaktorso".as_ptr(), TURN_ON);

    NPC_SetAnim(
        NPC,
        SETANIM_BOTH,
        BOTH_STAND2TO1,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    (*(*NPC).client).ps.legsTimer += 500;
    (*(*NPC).client).ps.torsoTimer += 500;
}

/*
-------------------------
NPC_BSGM_Attack
-------------------------
*/

pub unsafe fn NPC_BSGM_Attack() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    // #if 0 — FIXME victory-anim block (BOTH_STAND2TO1 / BOTH_TRIUMPHANT* / GM_StartGloat)
    //         is excluded from the MP build; out of scope.

    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE || (*NPC).enemy.is_null() {
        (*NPC).enemy = core::ptr::null_mut();
        NPC_BSGM_Patrol();
        return;
    }

    enemyLOS4 = QFALSE;
    enemyCS4 = QFALSE;
    move4 = QTRUE;
    faceEnemy4 = QFALSE;
    shoot4 = QFALSE;
    hitAlly4 = QFALSE;
    VectorClear(&mut *addr_of_mut!(impactPos4));
    enemyDist4 = DistanceSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);

    //if ( NPC->client->ps.torsoAnim == BOTH_ATTACK4 ||
    //	NPC->client->ps.torsoAnim == BOTH_ATTACK5 )
    if false {
        shoot4 = QFALSE;
        if TIMER_Done(NPC, c"smackTime".as_ptr()) != QFALSE && (*NPCInfo).blockedDebounceTime == 0 {
            //time to smack
            //recheck enemyDist4 and InFront
            if enemyDist4 < MELEE_DIST_SQUARED
                && InFront(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*NPC).client).ps.viewangles,
                    0.3,
                ) != QFALSE
            {
                let mut smackDir: vec3_t = [0.0; 3];
                VectorSubtract(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &mut smackDir,
                );
                smackDir[2] += 30.0;
                VectorNormalize(&mut smackDir);
                //hurt them
                G_Sound(
                    (*NPC).enemy,
                    CHAN_AUTO,
                    G_SoundIndex("sound/weapons/galak/skewerhit.wav"),
                );
                G_Damage(
                    (*NPC).enemy,
                    NPC,
                    NPC,
                    addr_of_mut!(smackDir),
                    addr_of_mut!((*NPC).r.currentOrigin),
                    ((*addr_of!(g_spskill)).integer + 1) * Q_irand(5, 10),
                    DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                    MOD_CRUSH,
                );
                if (*(*NPC).client).ps.torsoAnim == BOTH_ATTACK4 {
                    //smackdown
                    let mut knockAnim: c_int = BOTH_KNOCKDOWN1;
                    if BG_CrouchAnim((*(*(*NPC).enemy).client).ps.legsAnim) != QFALSE {
                        //knockdown from crouch
                        knockAnim = BOTH_KNOCKDOWN4;
                    }
                    //throw them
                    smackDir[2] = 1.0;
                    VectorNormalize(&mut smackDir);
                    G_Throw((*NPC).enemy, &smackDir, 50.0);
                    NPC_SetAnim(
                        (*NPC).enemy,
                        SETANIM_BOTH,
                        knockAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                } else {
                    //uppercut
                    //throw them
                    G_Throw((*NPC).enemy, &smackDir, 100.0);
                    //make them backflip
                    NPC_SetAnim(
                        (*NPC).enemy,
                        SETANIM_BOTH,
                        BOTH_KNOCKDOWN5,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
                //done with the damage
                (*NPCInfo).blockedDebounceTime = 1;
            }
        }
    } else if (*NPC).lockCount != 0 {
        //already shooting laser
        //sometimes use the laser beam attack, but only after he's taken down our generator
        shoot4 = QFALSE;
        if (*NPC).lockCount == 1 {
            //charging up
            if TIMER_Done(NPC, c"beamDelay".as_ptr()) != QFALSE {
                //time to start the beam
                //if ( Q_irand( 0, 1 ) )
                //if (1)
                let laserAnim: c_int = BOTH_ATTACK2;
                /*
                else
                {
                    laserAnim = BOTH_ATTACK7;
                }
                */
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    laserAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*(*NPC).client).ps.torsoTimer + Q_irand(1000, 3000),
                );
                //turn on beam effect
                (*NPC).lockCount = 2;
                G_PlayEffectID(
                    G_EffectIndex("galak/trace_beam"),
                    &(*NPC).r.currentOrigin,
                    &vec3_origin,
                );
                (*NPC).s.loopSound = G_SoundIndex("sound/weapons/galak/lasercutting.wav");
                if (*NPCInfo).coverTarg.is_null() {
                    //for moving looping sound at end of trace
                    (*NPCInfo).coverTarg = G_Spawn();
                    if !(*NPCInfo).coverTarg.is_null() {
                        G_SetOrigin(
                            (*NPCInfo).coverTarg,
                            &(*(*NPC).client).renderInfo.muzzlePoint,
                        );
                        (*(*NPCInfo).coverTarg).r.svFlags |= SVF_BROADCAST;
                        (*(*NPCInfo).coverTarg).s.loopSound =
                            G_SoundIndex("sound/weapons/galak/lasercutting.wav");
                    }
                }
            }
        } else {
            //in the actual attack now
            if (*(*NPC).client).ps.torsoTimer <= 0 {
                //attack done!
                (*NPC).lockCount = 0;
                G_FreeEntity((*NPCInfo).coverTarg);
                (*NPC).s.loopSound = 0;
                //#if 0
                //				NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_DROPWEAP2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
                //#endif
                TIMER_Set(NPC, c"attackDelay".as_ptr(), (*(*NPC).client).ps.torsoTimer);
            } else {
                //attack still going
                //do the trace and damage
                let mins: vec3_t = [-3.0, -3.0, -3.0];
                let maxs: vec3_t = [3.0, 3.0, 3.0];
                let mut end: vec3_t = [0.0; 3];
                VectorMA(
                    &(*(*NPC).client).renderInfo.muzzlePoint,
                    1024.0,
                    &(*(*NPC).client).renderInfo.muzzleDir,
                    &mut end,
                );
                let mut trace = trap::Trace(
                    &(*(*NPC).client).renderInfo.muzzlePoint,
                    &mins,
                    &maxs,
                    &end,
                    (*NPC).s.number,
                    MASK_SHOT,
                );
                if trace.allsolid != 0 || trace.startsolid != 0 {
                    //oops, in a wall
                    if !(*NPCInfo).coverTarg.is_null() {
                        G_SetOrigin(
                            (*NPCInfo).coverTarg,
                            &(*(*NPC).client).renderInfo.muzzlePoint,
                        );
                    }
                } else {
                    //clear
                    if trace.fraction < 1.0 {
                        //hit something
                        let traceEnt: *mut gentity_t = (addr_of_mut!(g_entities)
                            .cast::<gentity_t>())
                        .offset(trace.entityNum as isize);
                        if !traceEnt.is_null() && (*traceEnt).takedamage != QFALSE {
                            //damage it
                            G_SoundAtLoc(
                                &trace.endpos,
                                CHAN_AUTO,
                                G_SoundIndex("sound/weapons/galak/laserdamage.wav"),
                            );
                            G_Damage(
                                traceEnt,
                                NPC,
                                NPC,
                                addr_of_mut!((*(*NPC).client).renderInfo.muzzleDir),
                                addr_of_mut!(trace.endpos),
                                10,
                                0,
                                MOD_UNKNOWN,
                            );
                        }
                    }
                    if !(*NPCInfo).coverTarg.is_null() {
                        G_SetOrigin((*NPCInfo).coverTarg, &trace.endpos);
                    }
                    if Q_irand(0, 5) == 0 {
                        G_SoundAtLoc(
                            &trace.endpos,
                            CHAN_AUTO,
                            G_SoundIndex("sound/weapons/galak/laserdamage.wav"),
                        );
                    }
                }
            }
        }
    } else {
        //Okay, we're not in a special attack, see if we should switch weapons or start a special attack
        /*
        if ( NPC->s.weapon == WP_REPEATER
            && !(NPCInfo->scriptFlags & SCF_ALT_FIRE)//using rapid-fire
            && NPC->enemy->s.weapon == WP_SABER //enemy using saber
            && NPC->client && (NPC->client->ps.saberEventFlags&SEF_DEFLECTED)
            && !Q_irand( 0, 50 ) )
        {//he's deflecting my shots, switch to the laser or the lob fire for a while
            TIMER_Set( NPC, "noRapid", Q_irand( 2000, 6000 ) );
            NPCInfo->scriptFlags |= SCF_ALT_FIRE;
            NPC->alt_fire = qtrue;
            if ( NPC->locationDamage[HL_GENERIC1] > GENERATOR_HEALTH && (Q_irand( 0, 1 )||enemyDist4 < MAX_LOB_DIST_SQUARED) )
            {//shield down, use laser
                NPC_GM_StartLaser();
            }
        }
        else*/
        if
        // !NPC->client->ps.powerups[PW_GALAK_SHIELD]
        true //rwwFIXMEFIXME: just act like the shield is down til the effects and stuff are done
            && enemyDist4 < MELEE_DIST_SQUARED
            && InFront(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &(*(*NPC).client).ps.viewangles,
                0.3,
            ) != QFALSE
            && (*(*NPC).enemy).localAnimIndex <= 1
        {
            //within 80 and in front
            //our shield is down, and enemy within 80, if very close, use melee attack to slap away
            if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
                //animate me
                let swingAnim: c_int = BOTH_ATTACK1;
                //#if 0
                //				if ( NPC->locationDamage[HL_GENERIC1] > GENERATOR_HEALTH )
                //				{//generator down, use random melee
                //					swingAnim = Q_irand( BOTH_ATTACK4, BOTH_ATTACK5 );//smackdown or uppercut
                //				}
                //				else
                //				{//always knock-away
                //					swingAnim = BOTH_ATTACK5;//uppercut
                //				}
                //#endif
                //FIXME: swing sound
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    swingAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*(*NPC).client).ps.torsoTimer + Q_irand(1000, 3000),
                );
                //delay the hurt until the proper point in the anim
                TIMER_Set(NPC, c"smackTime".as_ptr(), 600);
                (*NPCInfo).blockedDebounceTime = 0;
                //FIXME: say something?
            }
        } else if (*NPC).lockCount == 0
            && (*NPC).locationDamage[HL_GENERIC1 as usize] > GENERATOR_HEALTH
            && TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
            && InFront(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &(*(*NPC).client).ps.viewangles,
                0.3,
            ) != QFALSE
            && ((Q_irand(0, 10 * (2 - (*addr_of!(g_spskill)).integer)) == 0
                && enemyDist4 > MIN_LOB_DIST_SQUARED
                && enemyDist4 < MAX_LOB_DIST_SQUARED)
                || (TIMER_Done(NPC, c"noLob".as_ptr()) == QFALSE
                    && TIMER_Done(NPC, c"noRapid".as_ptr()) == QFALSE))
            && (*(*NPC).enemy).s.weapon != WP_TURRET
        {
            //sometimes use the laser beam attack, but only after he's taken down our generator
            shoot4 = QFALSE;
            NPC_GM_StartLaser();
        } else if enemyDist4 < MIN_LOB_DIST_SQUARED
            && ((*(*NPC).enemy).s.weapon != WP_TURRET
                || Q_stricmp(c"PAS".as_ptr(), (*(*NPC).enemy).classname) != 0)
            && TIMER_Done(NPC, c"noRapid".as_ptr()) != QFALSE
        {
            //256
            //enemy within 256
            if (*(*NPC).client).ps.weapon == WP_REPEATER
                && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0
            {
                //shooting an explosive, but enemy too close, switch to primary fire
                (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
                (*NPC).alt_fire = QFALSE;
                //FIXME: use weap raise & lower anims
                NPC_ChangeWeapon(WP_REPEATER);
            }
        } else if (enemyDist4 > MAX_LOB_DIST_SQUARED
            || ((*(*NPC).enemy).s.weapon == WP_TURRET
                && Q_stricmp(c"PAS".as_ptr(), (*(*NPC).enemy).classname) == 0))
            && TIMER_Done(NPC, c"noLob".as_ptr()) != QFALSE
        {
            //448
            //enemy more than 448 away and we are ready to try lob fire again
            if (*(*NPC).client).ps.weapon == WP_REPEATER
                && (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0
            {
                //enemy far enough away to use lobby explosives
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                (*NPC).alt_fire = QTRUE;
                //FIXME: use weap raise & lower anims
                NPC_ChangeWeapon(WP_REPEATER);
            }
        }
    }

    //can we see our target?
    if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time; //used here for aim debouncing, not always a clear LOS
        enemyLOS4 = QTRUE;

        if (*(*NPC).client).ps.weapon == WP_NONE {
            enemyCS4 = QFALSE; //not true, but should stop us from firing
            NPC_AimAdjust(-1); //adjust aim worse longer we have no weapon
        } else {
            //can we shoot our target?
            if (*(*NPC).client).ps.weapon == WP_REPEATER
                && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0
                && enemyDist4 < MIN_LOB_DIST_SQUARED
            {
                //256
                enemyCS4 = QFALSE; //not true, but should stop us from firing
                hitAlly4 = QTRUE; //us!
                                  //FIXME: if too close, run away!
            } else {
                let hit: c_int = NPC_ShotEntity((*NPC).enemy, addr_of_mut!(impactPos4));
                let hitEnt: *mut gentity_t =
                    (addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize);
                if hit == (*(*NPC).enemy).s.number
                    || (!hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
                    || (!hitEnt.is_null() && (*hitEnt).takedamage != QFALSE)
                {
                    //can hit enemy or will hit glass or other breakable, so shoot anyway
                    enemyCS4 = QTRUE;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                    VectorCopy(
                        &(*(*NPC).enemy).r.currentOrigin,
                        &mut (*NPCInfo).enemyLastSeenLocation,
                    );
                } else {
                    //Hmm, have to get around this bastard
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                    if !hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).playerTeam
                    {
                        //would hit an ally, don't fire!!!
                        hitAlly4 = QTRUE;
                    } else {
                        //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                    }
                }
            }
        }
    } else if trap::InPVS(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin) != QFALSE {
        let hit: c_int;
        let hitEnt: *mut gentity_t;

        if TIMER_Done(NPC, c"talkDebounce".as_ptr()) != QFALSE && Q_irand(0, 10) == 0 {
            if (*NPCInfo).enemyCheckDebounceTime < 8 {
                let mut speech: c_int = -1;
                match (*NPCInfo).enemyCheckDebounceTime {
                    0 | 1 | 2 => {
                        speech = EV_CHASE1 + (*NPCInfo).enemyCheckDebounceTime;
                    }
                    3 | 4 | 5 => {
                        speech = EV_COVER1 + (*NPCInfo).enemyCheckDebounceTime - 3;
                    }
                    6 | 7 => {
                        speech = EV_ESCAPING1 + (*NPCInfo).enemyCheckDebounceTime - 6;
                    }
                    _ => {}
                }
                (*NPCInfo).enemyCheckDebounceTime += 1;
                if speech != -1 {
                    G_AddVoiceEvent(NPC, speech, Q_irand(3000, 5000));
                    TIMER_Set(NPC, c"talkDebounce".as_ptr(), Q_irand(5000, 7000));
                }
            }
        }

        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;

        hit = NPC_ShotEntity((*NPC).enemy, addr_of_mut!(impactPos4));
        hitEnt = (addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize);
        if hit == (*(*NPC).enemy).s.number
            || (!hitEnt.is_null()
                && !(*hitEnt).client.is_null()
                && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
            || (!hitEnt.is_null() && (*hitEnt).takedamage != QFALSE)
        {
            //can hit enemy or will hit glass or other breakable, so shoot anyway
            enemyCS4 = QTRUE;
        } else {
            faceEnemy4 = QTRUE;
            NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
        }
    }

    if enemyLOS4 != 0 {
        faceEnemy4 = QTRUE;
    } else {
        if (*NPCInfo).goalEntity.is_null() {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //for now, always chase the enemy
            move4 = QTRUE;
        }
    }
    if enemyCS4 != 0 {
        shoot4 = QTRUE;
        //NPCInfo->enemyCheckDebounceTime = level.time;//actually used here as a last actual LOS
    } else {
        if (*NPCInfo).goalEntity.is_null() {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //for now, always chase the enemy
            move4 = QTRUE;
        }
    }

    //Check for movement to take care of
    GM_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    GM_CheckFireState();

    if (*(*NPC).client).ps.weapon == WP_REPEATER
        && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0
        && shoot4 != 0
        && TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
    {
        let mut muzzle: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let mut target: vec3_t = [0.0; 3];
        let mut velocity: vec3_t = [0.0, 0.0, 0.0];
        let mins: vec3_t = [-REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE];
        let maxs: vec3_t = [REPEATER_ALT_SIZE, REPEATER_ALT_SIZE, REPEATER_ALT_SIZE];
        let clearshot: qboolean;

        CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);

        VectorCopy(&(*(*NPC).enemy).r.currentOrigin, &mut target);

        target[0] +=
            flrand(-5.0, 5.0) + (crandom() as f32 * (6 - (*NPCInfo).currentAim) as f32 * 2.0);
        target[1] +=
            flrand(-5.0, 5.0) + (crandom() as f32 * (6 - (*NPCInfo).currentAim) as f32 * 2.0);
        target[2] +=
            flrand(-5.0, 5.0) + (crandom() as f32 * (6 - (*NPCInfo).currentAim) as f32 * 2.0);

        //Find the desired angles
        clearshot = WP_LobFire(
            NPC,
            &muzzle,
            &target,
            &mins,
            &maxs,
            MASK_SHOT | CONTENTS_LIGHTSABER,
            &mut velocity,
            QTRUE,
            (*NPC).s.number,
            (*(*NPC).enemy).s.number,
            300.0,
            1100.0,
            1500.0,
            QTRUE,
        );
        if VectorCompare(&vec3_origin, &velocity) != 0
            || (clearshot == QFALSE && enemyLOS4 != 0 && enemyCS4 != 0)
        {
            //no clear lob shot and no lob shot that will hit something breakable
            if enemyLOS4 != 0 && enemyCS4 != 0 && TIMER_Done(NPC, c"noRapid".as_ptr()) != QFALSE {
                //have a clear straight shot, so switch to primary
                (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
                (*NPC).alt_fire = QFALSE;
                NPC_ChangeWeapon(WP_REPEATER);
                //keep this weap for a bit
                TIMER_Set(NPC, c"noLob".as_ptr(), Q_irand(500, 1000));
            } else {
                shoot4 = QFALSE;
            }
        } else {
            vectoangles(&velocity, &mut angles);

            (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW as usize]);
            (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH as usize]);

            VectorCopy(&velocity, &mut (*(*NPC).client).hiddenDir);
            (*(*NPC).client).hiddenDist = VectorNormalize(&mut (*(*NPC).client).hiddenDir);
        }
    } else if faceEnemy4 != 0 {
        //face the enemy
        NPC_FaceEnemy(QTRUE);
    }

    if TIMER_Done(NPC, c"standTime".as_ptr()) == QFALSE {
        move4 = QFALSE;
    }
    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
        //not supposed to chase my enemies
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //goal is my entity, so don't move
            move4 = QFALSE;
        }
    }

    if move4 != 0 && (*NPC).lockCount == 0 {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null()
        /*&& NPC->client->ps.legsAnim != BOTH_ALERT1
        && NPC->client->ps.legsAnim != BOTH_ATTACK2
        && NPC->client->ps.legsAnim != BOTH_ATTACK4
        && NPC->client->ps.legsAnim != BOTH_ATTACK5
        && NPC->client->ps.legsAnim != BOTH_ATTACK7*/
        {
            move4 = GM_Move();
        } else {
            move4 = QFALSE;
        }
    }

    if TIMER_Done(NPC, c"flee".as_ptr()) == QFALSE {
        //running away
        faceEnemy4 = QFALSE;
    }

    //FIXME: check scf_face_move_dir here?

    if faceEnemy4 == 0 {
        //we want to face in the dir we're running
        if move4 == 0 {
            //if we haven't moved, we should look in the direction we last looked?
            VectorCopy(&(*(*NPC).client).ps.viewangles, &mut (*NPCInfo).lastPathAngles);
        }
        if move4 != 0 {
            //don't run away and shoot
            (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
            (*NPCInfo).desiredPitch = 0.0;
            shoot4 = QFALSE;
        }
    }
    NPC_UpdateAngles(QTRUE, QTRUE);

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot4 = QFALSE;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER {
            //don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot4 = QFALSE;
        }
    }
    //FIXME: don't shoot right away!
    if shoot4 != 0 {
        //try to shoot if it's time
        if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
            if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
                // we've already fired, no need to do it again here
                WeaponThink(QTRUE);
            }
        }
    }

    //also:
    if (*(*NPC).enemy).s.weapon == WP_TURRET
        && Q_stricmp(c"PAS".as_ptr(), (*(*NPC).enemy).classname) == 0
    {
        //crush turrets
        if G_BoundsOverlap(
            &(*NPC).r.absmin,
            &(*NPC).r.absmax,
            &(*(*NPC).enemy).r.absmin,
            &(*(*NPC).enemy).r.absmax,
        ) != QFALSE
        {
            //have to do this test because placed turrets are not solid to NPCs (so they don't obstruct navigation)
            //if ( NPC->client->ps.powerups[PW_GALAK_SHIELD] > 0 )
            if false {
                (*(*NPC).client).ps.powerups[PW_BATTLESUIT as usize] =
                    (*addr_of!(level)).time + ARMOR_EFFECT_TIME;
                G_Damage(
                    (*NPC).enemy,
                    NPC,
                    NPC,
                    core::ptr::null_mut(),
                    addr_of_mut!((*NPC).r.currentOrigin),
                    100,
                    DAMAGE_NO_KNOCKBACK,
                    MOD_UNKNOWN,
                );
            } else {
                G_Damage(
                    (*NPC).enemy,
                    NPC,
                    NPC,
                    core::ptr::null_mut(),
                    addr_of_mut!((*NPC).r.currentOrigin),
                    100,
                    DAMAGE_NO_KNOCKBACK,
                    MOD_CRUSH,
                );
            }
        }
    } else if !(*NPCInfo).touchedByPlayer.is_null() && (*NPCInfo).touchedByPlayer == (*NPC).enemy {
        //touched enemy
        //if ( NPC->client->ps.powerups[PW_GALAK_SHIELD] > 0 )
        if false {
            //zap him!
            let mut smackDir: vec3_t = [0.0; 3];

            //animate me
            //#if 0
            //			NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_ATTACK6, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            //#endif
            TIMER_Set(NPC, c"attackDelay".as_ptr(), (*(*NPC).client).ps.torsoTimer);
            TIMER_Set(NPC, c"standTime".as_ptr(), (*(*NPC).client).ps.legsTimer);
            //FIXME: debounce this?
            (*NPCInfo).touchedByPlayer = core::ptr::null_mut();
            //FIXME: some shield effect?
            (*(*NPC).client).ps.powerups[PW_BATTLESUIT as usize] =
                (*addr_of!(level)).time + ARMOR_EFFECT_TIME;

            VectorSubtract(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &mut smackDir,
            );
            smackDir[2] += 30.0;
            VectorNormalize(&mut smackDir);
            G_Damage(
                (*NPC).enemy,
                NPC,
                NPC,
                addr_of_mut!(smackDir),
                addr_of_mut!((*NPC).r.currentOrigin),
                ((*addr_of!(g_spskill)).integer + 1) * Q_irand(5, 10),
                DAMAGE_NO_KNOCKBACK,
                MOD_UNKNOWN,
            );
            //throw them
            G_Throw((*NPC).enemy, &smackDir, 100.0);
            //NPC->enemy->s.powerups |= ( 1 << PW_SHOCKED );
            if !(*(*NPC).enemy).client.is_null() {
                //	NPC->enemy->client->ps.powerups[PW_SHOCKED] = level.time + 1000;
                (*(*(*NPC).enemy).client).ps.electrifyTime = (*addr_of!(level)).time + 1000;
            }
            //stop any attacks
            ucmd.buttons = 0;
        }
    }

    if (*NPCInfo).movementSpeech < 3 && (*NPCInfo).blockedSpeechDebounceTime <= (*addr_of!(level)).time {
        if !(*NPC).enemy.is_null()
            && (*(*NPC).enemy).health > 0
            && (*(*NPC).enemy).painDebounceTime > (*addr_of!(level)).time
        {
            if (*(*NPC).enemy).health < 50 && (*NPCInfo).movementSpeech == 2 {
                G_AddVoiceEvent(NPC, EV_ANGER2, Q_irand(2000, 4000));
                (*NPCInfo).movementSpeech = 3;
            } else if (*(*NPC).enemy).health < 75 && (*NPCInfo).movementSpeech == 1 {
                G_AddVoiceEvent(NPC, EV_ANGER1, Q_irand(2000, 4000));
                (*NPCInfo).movementSpeech = 2;
            } else if (*(*NPC).enemy).health < 100 && (*NPCInfo).movementSpeech == 0 {
                G_AddVoiceEvent(NPC, EV_ANGER3, Q_irand(2000, 4000));
                (*NPCInfo).movementSpeech = 1;
            }
        }
    }
}

pub unsafe fn NPC_BSGM_Default() {
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if (*(*NPC).client).ps.stats[STAT_ARMOR as usize] <= 0 {
        //armor gone
        //	if ( !NPCInfo->investigateDebounceTime )
        if false {
            //start regenerating the armor
            NPC_SetSurfaceOnOff(NPC, c"torso_shield".as_ptr(), TURN_OFF);
            (*NPC).flags &= !FL_SHIELDED; //no more reflections
            VectorSet(&mut (*NPC).r.mins, -20.0, -20.0, -24.0);
            VectorSet(&mut (*NPC).r.maxs, 20.0, 20.0, 64.0);
            (*(*NPC).client).ps.standheight = 64;
            (*(*NPC).client).ps.crouchheight = (*(*NPC).client).ps.standheight;
            if (*NPC).locationDamage[HL_GENERIC1 as usize] < GENERATOR_HEALTH {
                //still have the generator bolt-on
                if (*NPCInfo).investigateCount < 12 {
                    (*NPCInfo).investigateCount += 1;
                }
                (*NPCInfo).investigateDebounceTime =
                    (*addr_of!(level)).time + ((*NPCInfo).investigateCount * 5000);
            }
        } else if (*NPCInfo).investigateDebounceTime < (*addr_of!(level)).time {
            //armor regenerated, turn shield back on
            //do a trace and make sure we can turn this back on?
            let tr: trace_t = trap::Trace(
                &(*NPC).r.currentOrigin,
                &*addr_of!(shieldMins),
                &*addr_of!(shieldMaxs),
                &(*NPC).r.currentOrigin,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if tr.startsolid == 0 {
                VectorCopy(&*addr_of!(shieldMins), &mut (*NPC).r.mins);
                VectorCopy(&*addr_of!(shieldMaxs), &mut (*NPC).r.maxs);
                (*(*NPC).client).ps.standheight = (*addr_of!(shieldMaxs))[2] as c_int;
                (*(*NPC).client).ps.crouchheight = (*(*NPC).client).ps.standheight;
                (*(*NPC).client).ps.stats[STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
                (*NPCInfo).investigateDebounceTime = 0;
                (*NPC).flags |= FL_SHIELDED; //reflect normal shots
                                             //	NPC->fx_time = level.time;
                NPC_SetSurfaceOnOff(NPC, c"torso_shield".as_ptr(), TURN_ON);
            }
        }
    }
    /*
    if ( NPC->client->ps.stats[STAT_ARMOR] > 0 )
    {//armor present
        NPC->client->ps.powerups[PW_GALAK_SHIELD] = Q3_INFINITE;//temp, for effect
        NPC_SetSurfaceOnOff( NPC, "torso_shield", TURN_ON );
    }
    else
    {
        NPC_SetSurfaceOnOff( NPC, "torso_shield", TURN_OFF );
    }
    */
    //rwwFIXMEFIXME: Allow this stuff, and again, going to have to let the client know about it.
    //Maybe a surface-off bitflag of some sort in the entity state?

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSGM_Patrol();
    } else
    //if ( NPC->enemy )
    {
        //have an enemy
        NPC_BSGM_Attack();
    }
}
