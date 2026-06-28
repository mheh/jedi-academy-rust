//! Port of `NPC_reactions.c` — the NPC pain/touch/respond reaction layer.
//!
//! Opened cycle 64 from the verified-portable leaves: `NPC_CheckAttacker`
//! (NPC_reactions.c:42), `NPC_TempLookTarget` (:663) and `NPC_GetPainChance` (:157).
//! These read only ported callees (`G_SetEnemy`/`G_ClearEnemy` from npc_combat.rs,
//! `NPC_CheckLookTarget`/`NPC_SetLookTarget` from npc_utils.rs) plus pure `g_spskill`
//! arithmetic, so they land without touching the not-yet-ported NPC-AI / ICARUS core.
//!
//! `NPC_SetPainEvent` (NPC_reactions.c:133) landed cycle 65 — its callee
//! `trap_ICARUS_TaskIDPending` is now wrapped and `G_AddEvent` is ported.
//!
//! Cycle 74 drained the reaction handlers: `NPC_ChoosePainAnimation` (:207),
//! `NPC_Pain` (:365), `NPC_Touch` (:539), `NPC_Respond` (:692) and `NPC_UseResponse`
//! (:952) — their callees (Save/Set/RestoreNPCGlobals, G_AddVoiceEvent,
//! G_ActivateBehavior, G_UseTargets2, BG_PickAnim/PM_* anim predicates, NPC_SetAnim,
//! G_Sound/G_SoundIndex) are all ported.
//!
//! `NPC_Use` (:1010) landed cycle 106 — its `Vehicle_t`/`m_pVehicle` board/eject vtable
//! and `Jedi_WaitingAmbush`/`Jedi_Ambush` callees are all ported now; the MEDIC-heal and
//! GONK-battery paths are commented out in the C original. Replaces the `npc.rs` dispatch stub.

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // `killPlayerTimer` kept verbatim from C

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::anims::{
    BOTH_PAIN1, BOTH_PAIN18, BOTH_PAIN2, BOTH_PAIN3,
};
use crate::codemp::game::b_public_h::{
    NPCAI_DIE_ON_IMPACT, NPCAI_TOUCHED_GOAL, RANK_CAPTAIN, SCF_CHASE_ENEMIES, SCF_CROUCHED,
    SCF_DONT_FIRE, SCF_FORCED_MARCH, SCF_NO_COMBAT_TALK, SCF_NO_MIND_TRICK, SCF_NO_RESPONSE,
    SCF_WALKING, BS_DEFAULT, BS_HUNT_AND_KILL,
};
use crate::codemp::game::bg_panimate::{
    bgAllAnims, bgHumanoidAnimations, BG_CrouchAnim, BG_FlippingAnim, BG_PickAnim,
    BG_SaberInSpecialAttack, PM_InCartwheel, PM_InKnockDown, PM_SpinningAnim,
};
use crate::codemp::game::bg_pmove::PM_RollingAnim;
use crate::codemp::game::bg_public::{
    bgEntity_t, EF2_HELD_BY_MONSTER, EV_ANGER1, EV_ANGER3, EV_CHASE1, EV_CHASE3, EV_CHOKE1, EV_CHOKE3, EV_CONFUSE1, EV_COVER1,
    EV_COVER5, EV_DETECTED1, EV_DETECTED5, EV_ESCAPING2, EV_FFTURN, EV_FFWARN, EV_GIVEUP3, EV_GIVEUP4,
    EV_JDETECTED1, EV_JDETECTED2, EV_LOST1, EV_OUTFLANK1, EV_OUTFLANK2, EV_PAIN, EV_SIGHT1,
    EV_SIGHT2, EV_SIGHT3, EV_SOUND1, EV_SOUND3, EV_SUSPICIOUS4, EV_TAUNT1, EV_TAUNT2, MOD_CRUSH,
    MOD_MELEE, MOD_SABER, PM_DEAD, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
    SETANIM_LEGS, STAT_MAX_HEALTH, TEAM_FREE,
};
use crate::codemp::game::bg_public::LS_READY;
use crate::codemp::game::bg_weapons_h::{WP_SABER, WP_THERMAL};
use crate::codemp::game::g_local::{gentity_t, FL_NOTARGET, HL_GENERIC1};
use crate::codemp::game::g_main::{g_entities, g_spskill, level};
use crate::codemp::game::g_public_h::{
    BSET_FFIRE, BSET_FLEE, BSET_PAIN, BSET_USE, SVF_ICARUS_FREEZE, TID_CHAN_VOICE,
};
use crate::codemp::game::g_utils::{G_Sound, G_SoundIndex, G_UseTargets2, G_AddEvent};
use crate::codemp::game::npc::{
    NPCInfo, NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC,
};
use crate::codemp::game::npc_ai_jedi::{Jedi_Ambush, Jedi_WaitingAmbush};
use crate::codemp::game::npc_combat::{G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy};
use crate::codemp::game::npc_utils::{G_ActivateBehavior, NPC_CheckLookTarget, NPC_SetLookTarget};
use crate::codemp::game::q_math::VectorCopy;
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{trace_t, CHAN_AUTO, FORCE_LEVEL_1, MAX_CLIENTS};
use crate::codemp::game::teams_h::{
    npcteam_t, CLASS_BESPIN_COP, CLASS_DESANN, CLASS_GALAKMECH, CLASS_GONK, CLASS_JAN, CLASS_JEDI,
    CLASS_LANDO, CLASS_LUKE, CLASS_MOUSE, CLASS_PRISONER, CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2,
    CLASS_REBEL, CLASS_VEHICLE, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER,
};
use crate::codemp::game::g_combat::{gPainHitLoc, gPainMOD, gPainPoint};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap::ICARUS_TaskIDPending;

/// `extern int killPlayerTimer;` (NPC_reactions.c:29). The storage lives in
/// `g_main.c` (`int killPlayerTimer = 0;`), which has not landed yet; until it does
/// this is the module's single definition. `NPC_Pain` is the sole current consumer
/// (it arms the timer when an NPC turns on the friendly-firing player). REVISIT:
/// re-home into g_main.rs when that file ports its globals.
pub static mut killPlayerTimer: c_int = 0;

/*
-------------------------
NPC_CheckAttacker
-------------------------
*/
// TODO: Port-Bug - Missing return after player interaction block (should return after line 177)
unsafe fn NPC_CheckAttacker(other: *mut gentity_t, mod_: c_int) {
    //FIXME: I don't see anything in here that would stop teammates from taking a teammate
    //			as an enemy.  Ideally, there would be code before this to prevent that from
    //			happening, but that is presumptuous.

    //valid ent - FIXME: a VALIDENT macro would be nice here
    if other.is_null() {
        return;
    }

    if other == NPC {
        return;
    }

    if (*other).inuse == QFALSE {
        return;
    }

    //Don't take a target that doesn't want to be
    if (*other).flags & FL_NOTARGET != 0 {
        return;
    }

    //	if ( NPC->svFlags & SVF_LOCKEDENEMY )
    //	{//IF LOCKED, CANNOT CHANGE ENEMY!!!!!
    //		return;
    //	}
    //rwwFIXMEFIXME: support this

    //If we haven't taken a target, just get mad
    if (*NPC).enemy.is_null()
    //was using "other", fixed to NPC
    {
        G_SetEnemy(NPC, other);
        return;
    }

    //we have an enemy, see if he's dead
    if (*(*NPC).enemy).health <= 0 {
        G_ClearEnemy(NPC);
        G_SetEnemy(NPC, other);
        return;
    }

    //Don't take the same enemy again
    if other == (*NPC).enemy {
        return;
    }

    if (*(*NPC).client).ps.weapon == WP_SABER {
        //I'm a jedi
        if mod_ == MOD_SABER {
            //I was hit by a saber  FIXME: what if this was a thrown saber?
            //always switch to this enemy if I'm a jedi and hit by another saber
            G_ClearEnemy(NPC);
            G_SetEnemy(NPC, other);
            return;
        }
    }
    //Special case player interactions
    if other == core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()
    //&g_entities[0]
    {
        //Account for the skill level to skew the results
        let luckThreshold: f32;

        match (*addr_of!(g_spskill)).integer {
            //Easiest difficulty, mild chance of picking up the player
            0 => {
                luckThreshold = 0.9f32;
            }

            //Medium difficulty, half-half chance of picking up the player
            1 => {
                luckThreshold = 0.5f32;
            }

            //Hardest difficulty, always turn on attacking player
            _ => {
                luckThreshold = 0.0f32;
            }
        }

        //Randomly pick up the target
        if random() > luckThreshold {
            G_ClearEnemy(other);
            (*other).enemy = NPC;
        }
    }
}

pub unsafe fn NPC_SetPainEvent(self_: *mut gentity_t) {
    if (*self_).NPC.is_null() || (*(*self_).NPC).aiFlags & NPCAI_DIE_ON_IMPACT == 0 {
        // no more borg
        //	if( self->client->playerTeam != TEAM_BORG )
        //	{
        //if ( !Q3_TaskIDPending( self, TID_CHAN_VOICE ) )
        if ICARUS_TaskIDPending(self_, TID_CHAN_VOICE) != QTRUE && !(*self_).client.is_null() {
            //G_AddEvent( self, EV_PAIN, floor((float)self->health/self->max_health*100.0f) );
            G_AddEvent(
                self_,
                EV_PAIN,
                (((*self_).health as f32
                    / (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] as f32
                    * 100.0f32) as f64)
                    .floor() as c_int,
            );
            //rwwFIXMEFIXME: Do this properly?
        }
        //	}
    }
}

/*
-------------------------
NPC_GetPainChance
-------------------------
*/

pub unsafe fn NPC_GetPainChance(self_: *mut gentity_t, damage: c_int) -> f32 {
    let mut pain_chance: f32;
    if (*self_).enemy.is_null() {
        //surprised, always take pain
        return 1.0f32;
    }

    if (*self_).client.is_null() {
        return 1.0f32;
    }

    //if ( damage > self->max_health/2.0f )
    if damage as f32 > (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] as f32 / 2.0f32 {
        return 1.0f32;
    }

    pain_chance = ((*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] - (*self_).health) as f32
        / ((*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] as f32 * 2.0f32)
        + damage as f32 / ((*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] as f32 / 2.0f32);
    match (*addr_of!(g_spskill)).integer {
        0 => {
            //easy
            //return 0.75f;
        }

        1 => {
            //med
            pain_chance *= 0.5f32;
            //return 0.35f;
        }

        _ => {
            //hard
            pain_chance *= 0.1f32;
            //return 0.05f;
        }
    }
    //Com_Printf( "%s: %4.2f\n", self->NPC_type, pain_chance );
    pain_chance
}

/*
-------------------------
NPC_ChoosePainAnimation
-------------------------
*/

const MIN_PAIN_TIME: c_int = 200;

// extern int G_PickPainAnim( gentity_t *self, vec3_t point, int damage, int hitLoc );
#[allow(unused_variables)] // `point` feeds the commented-out G_PickPainAnim path only
pub unsafe fn NPC_ChoosePainAnimation(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    point: *mut [f32; 3], // vec3_t point
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
    voiceEvent: c_int,
) {
    let _ = MIN_PAIN_TIME; // #define MIN_PAIN_TIME 200 — declared by C, not referenced in this build
    let mut pain_anim: c_int = -1;
    let mut pain_chance: f32;

    //If we've already taken pain, then don't take it again
    if (*addr_of!(level)).time < (*self_).painDebounceTime
        && /*mod != MOD_ELECTROCUTE &&*/ mod_ != MOD_MELEE
    {
        //rwwFIXMEFIXME: MOD_ELECTROCUTE
        //FIXME: if hit while recoving from losing a saber lock, we should still play a pain anim?
        return;
    }

    if (*self_).s.weapon == WP_THERMAL && (*(*self_).client).ps.weaponTime > 0 {
        //don't interrupt thermal throwing anim
        return;
    }
    else if (*(*self_).client).NPC_class == CLASS_GALAKMECH {
        if hitLoc == HL_GENERIC1 {
            //hit the antenna!
            pain_chance = 1.0f32;
            //	self->s.powerups |= ( 1 << PW_SHOCKED );
            //	self->client->ps.powerups[PW_SHOCKED] = level.time + Q_irand( 500, 2500 );
            //rwwFIXMEFIXME: support for this
        }
        //	else if ( self->client->ps.powerups[PW_GALAK_SHIELD] )
        //	{//shield up
        //		return;
        //	}
        //rwwFIXMEFIXME: and this
        else if (*self_).health > 200 && damage < 100 {
            //have a *lot* of health
            pain_chance = 0.05f32;
        } else {
            //the lower my health and greater the damage, the more likely I am to play a pain anim
            pain_chance = (200.0f32 - (*self_).health as f32) / 100.0f32 + damage as f32 / 50.0f32;
        }
    } else if !(*self_).client.is_null()
        && (*(*self_).client).playerTeam == NPCTEAM_PLAYER
        && !other.is_null()
        && (*other).s.number == 0
    {
        //ally shot by player always complains
        pain_chance = 1.1f32;
    } else {
        if !other.is_null() && (*other).s.weapon == WP_SABER
            || /*mod == MOD_ELECTROCUTE ||*/ mod_ == MOD_CRUSH
        /*FIXME:MOD_FORCE_GRIP*/
        {
            pain_chance = 1.0f32; //always take pain from saber
        } else if mod_ == MOD_MELEE {
            //higher in rank (skill) we are, less likely we are to be fazed by a punch
            pain_chance =
                1.0f32 - ((RANK_CAPTAIN - (*(*self_).NPC).rank) as f32 / RANK_CAPTAIN as f32);
        } else if (*(*self_).client).NPC_class == CLASS_PROTOCOL {
            pain_chance = 1.0f32;
        } else {
            pain_chance = NPC_GetPainChance(self_, damage);
        }
        if (*(*self_).client).NPC_class == CLASS_DESANN {
            pain_chance *= 0.5f32;
        }
    }

    //See if we're going to flinch
    if random() < pain_chance {
        // int animLength; (declared here in C; assigned unconditionally below before use)

        //Pick and play our animation
        if ((*(*self_).client).ps.fd.forceGripBeingGripped as f32) < (*addr_of!(level)).time as f32 {
            //not being force-gripped or force-drained
            if /*G_CheckForStrongAttackMomentum( self ) //rwwFIXMEFIXME: Is this needed?
                ||*/ PM_SpinningAnim( (*(*self_).client).ps.legsAnim ) == QTRUE
                || BG_SaberInSpecialAttack( (*(*self_).client).ps.torsoAnim ) == QTRUE
                || PM_InKnockDown( &mut (*(*self_).client).ps ) == QTRUE
                || PM_RollingAnim( (*(*self_).client).ps.legsAnim ) == QTRUE
                || (BG_FlippingAnim( (*(*self_).client).ps.legsAnim ) == QTRUE
                    && PM_InCartwheel( (*(*self_).client).ps.legsAnim ) != QTRUE)
            {
                //strong attacks, rolls, knockdowns, flips and spins cannot be interrupted by pain
            } else {
                //play an anim
                let mut parts: c_int;

                if (*(*self_).client).NPC_class == CLASS_GALAKMECH {
                    //only has 1 for now
                    //FIXME: never plays this, it seems...
                    pain_anim = BOTH_PAIN1;
                } else if mod_ == MOD_MELEE {
                    pain_anim = BG_PickAnim((*self_).localAnimIndex, BOTH_PAIN2, BOTH_PAIN3);
                } else if (*self_).s.weapon == WP_SABER {
                    //temp HACK: these are the only 2 pain anims that look good when holding a saber
                    pain_anim = BG_PickAnim((*self_).localAnimIndex, BOTH_PAIN2, BOTH_PAIN3);
                }
                /*
                else if ( mod != MOD_ELECTROCUTE )
                {
                    pain_anim = G_PickPainAnim( self, point, damage, hitLoc );
                }
                */

                if pain_anim == -1 {
                    pain_anim = BG_PickAnim((*self_).localAnimIndex, BOTH_PAIN1, BOTH_PAIN18);
                }
                (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_1; //next attack must be a quick attack
                (*(*self_).client).ps.saberMove = LS_READY; //don't finish whatever saber move you may have been in
                parts = SETANIM_BOTH;
                if BG_CrouchAnim((*(*self_).client).ps.legsAnim) == QTRUE
                    || PM_InCartwheel((*(*self_).client).ps.legsAnim) == QTRUE
                {
                    parts = SETANIM_LEGS;
                }

                if pain_anim != -1 {
                    NPC_SetAnim(
                        self_,
                        parts,
                        pain_anim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
            }
            if voiceEvent != -1 {
                G_AddVoiceEvent(self_, voiceEvent, Q_irand(2000, 4000));
            } else {
                NPC_SetPainEvent(self_);
            }
        } else {
            G_AddVoiceEvent(self_, Q_irand(EV_CHOKE1, EV_CHOKE3), 0);
        }

        //Setup the timing for it
        /*
        if ( mod == MOD_ELECTROCUTE )
        {
            self->painDebounceTime = level.time + 4000;
        }
        */
        // (RUST-FIX) C reads bgAllAnims/bgHumanoidAnimations[pain_anim==-1] (OOB, silently tolerated); Rust bounds-checks the slice, so guard the -1 sentinel (no anim → animLength 0).
        // let animLength: c_int = ((*(*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
        //     .anims
        //     .add(pain_anim as usize))
        // .numFrames as f32
        //     * ((*addr_of!(bgHumanoidAnimations))[pain_anim as usize].frameLerp as f32).abs())
        //     as c_int;
        let animLength: c_int = if pain_anim != -1 {
            ((*(*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
                .anims
                .add(pain_anim as usize))
            .numFrames as f32
                * ((*addr_of!(bgHumanoidAnimations))[pain_anim as usize].frameLerp as f32).abs())
                as c_int
        } else {
            0
        };

        (*self_).painDebounceTime = (*addr_of!(level)).time + animLength;
        (*(*self_).client).ps.weaponTime = 0;
    }
}

/*
===============
NPC_Pain
===============
*/
pub unsafe extern "C" fn NPC_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let mut otherTeam: npcteam_t = TEAM_FREE;
    let mut voiceEvent: c_int = -1;
    let other: *mut gentity_t = attacker;
    let mod_: c_int = *addr_of!(gPainMOD);
    let hitLoc: c_int = *addr_of!(gPainHitLoc);
    let mut point: [f32; 3] = [0.0; 3];

    VectorCopy(&*addr_of!(gPainPoint), &mut point);

    if (*self_).NPC.is_null() {
        return;
    }

    if other.is_null() {
        return;
    }

    //or just remove ->pain in player_die?
    if (*(*self_).client).ps.pm_type == PM_DEAD {
        return;
    }

    if other == self_ {
        return;
    }

    //MCG: Ignore damage from your own team for now
    if !(*other).client.is_null() {
        otherTeam = (*(*other).client).playerTeam;
        //	if ( otherTeam == TEAM_DISGUISE )
        //	{
        //		otherTeam = TEAM_PLAYER;
        //	}
    }

    if (*(*self_).client).playerTeam != 0
        && !(*other).client.is_null()
        && otherTeam == (*(*self_).client).playerTeam
    /*	&& (!player->client->ps.viewEntity || other->s.number != player->client->ps.viewEntity)*/
    {
        //rwwFIXMEFIXME: Will need modification when player controllable npcs are done
        //hit by a teammate
        if other != (*self_).enemy && self_ != (*other).enemy {
            //we weren't already enemies
            if !(*self_).enemy.is_null() || !(*other).enemy.is_null()
            //|| (other->s.number&&other->s.number!=player->client->ps.viewEntity)
            //rwwFIXMEFIXME: same

            /*|| (!other->s.number&&Q_irand( 0, 3 ))*/
            {
                //if one of us actually has an enemy already, it's okay, just an accident OR wasn't hit by player or someone controlled by player OR player hit ally and didn't get 25% chance of getting mad (FIXME:accumulate anger+base on diff?)
                //FIXME: player should have to do a certain amount of damage to ally or hit them several times to make them mad
                //Still run pain and flee scripts
                if !(*self_).client.is_null() && !(*self_).NPC.is_null() {
                    //Run any pain instructions
                    if (*self_).health
                        <= ((*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] / 3)
                        && G_ActivateBehavior(self_, BSET_FLEE) == QTRUE
                    {
                    } else
                    // if( VALIDSTRING( self->behaviorSet[BSET_PAIN] ) )
                    {
                        G_ActivateBehavior(self_, BSET_PAIN);
                    }
                }
                if damage != -1 {
                    //-1 == don't play pain anim
                    //Set our proper pain animation
                    if Q_irand(0, 1) != 0 {
                        NPC_ChoosePainAnimation(
                            self_, other, &mut point, damage, mod_, hitLoc, EV_FFWARN,
                        );
                    } else {
                        NPC_ChoosePainAnimation(self_, other, &mut point, damage, mod_, hitLoc, -1);
                    }
                }
                return;
            } else if !(*self_).NPC.is_null() && (*other).s.number == 0
            //should be assumed, but...
            {
                //dammit, stop that!
                if (*(*self_).NPC).charmedTime != 0 {
                    //mindtricked
                    return;
                } else if (*(*self_).NPC).ffireCount
                    < 3 + ((2 - (*addr_of!(g_spskill)).integer) * 2)
                {
                    //not mad enough yet
                    //Com_Printf( "chck: %d < %d\n", self->NPC->ffireCount, 3+((2-g_spskill.integer)*2) );
                    if damage != -1 {
                        //-1 == don't play pain anim
                        //Set our proper pain animation
                        if Q_irand(0, 1) != 0 {
                            NPC_ChoosePainAnimation(
                                self_, other, &mut point, damage, mod_, hitLoc, EV_FFWARN,
                            );
                        } else {
                            NPC_ChoosePainAnimation(
                                self_, other, &mut point, damage, mod_, hitLoc, -1,
                            );
                        }
                    }
                    return;
                } else if G_ActivateBehavior(self_, BSET_FFIRE) == QTRUE {
                    //we have a specific script to run, so do that instead
                    return;
                } else {
                    //okay, we're going to turn on our ally, we need to set and lock our enemy and put ourselves in a bstate that lets us attack him (and clear any flags that would stop us)
                    (*(*self_).NPC).blockedSpeechDebounceTime = 0;
                    voiceEvent = EV_FFTURN;
                    (*(*self_).NPC).behaviorState = BS_DEFAULT;
                    (*(*self_).NPC).tempBehavior = BS_DEFAULT;
                    (*(*self_).NPC).defaultBehavior = BS_DEFAULT;
                    (*other).flags &= !FL_NOTARGET;
                    //self->svFlags &= ~(SVF_IGNORE_ENEMIES|SVF_ICARUS_FREEZE|SVF_NO_COMBAT_SOUNDS);
                    (*self_).r.svFlags &= !SVF_ICARUS_FREEZE;
                    G_SetEnemy(self_, other);
                    //self->svFlags |= SVF_LOCKEDENEMY; //rwwFIXMEFIXME: proper support for these flags.
                    (*(*self_).NPC).scriptFlags &= !(SCF_DONT_FIRE
                        | SCF_CROUCHED
                        | SCF_WALKING
                        | SCF_NO_COMBAT_TALK
                        | SCF_FORCED_MARCH);
                    (*(*self_).NPC).scriptFlags |= SCF_CHASE_ENEMIES | SCF_NO_MIND_TRICK;
                    //NOTE: we also stop ICARUS altogether
                    //stop_icarus = qtrue;
                    //rwwFIXMEFIXME: stop icarus?
                    if *addr_of!(killPlayerTimer) == 0 {
                        *addr_of_mut!(killPlayerTimer) = (*addr_of!(level)).time + 10000;
                    }
                }
            }
        }
    }

    SaveNPCGlobals();
    SetNPCGlobals(self_);

    //Do extra bits
    if (*NPCInfo).ignorePain == QFALSE {
        (*NPCInfo).confusionTime = 0; //clear any charm or confusion, regardless
        if damage != -1 {
            //-1 == don't play pain anim
            //Set our proper pain animation
            NPC_ChoosePainAnimation(self_, other, &mut point, damage, mod_, hitLoc, voiceEvent);
        }
        //Check to take a new enemy
        if (*NPC).enemy != other && NPC != other {
            //not already mad at them
            NPC_CheckAttacker(other, mod_);
        }
    }

    //Attempt to run any pain instructions
    if !(*self_).client.is_null() && !(*self_).NPC.is_null() {
        //FIXME: This needs better heuristics perhaps
        if (*self_).health <= ((*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] / 3)
            && G_ActivateBehavior(self_, BSET_FLEE) == QTRUE
        {
        } else
        //if( VALIDSTRING( self->behaviorSet[BSET_PAIN] ) )
        {
            G_ActivateBehavior(self_, BSET_PAIN);
        }
    }

    //Attempt to fire any paintargets we might have
    if !(*self_).paintarget.is_null() && *(*self_).paintarget != 0 {
        G_UseTargets2(self_, other, (*self_).paintarget);
    }

    RestoreNPCGlobals();
}

/*
-------------------------
NPC_Touch
-------------------------
*/
// extern qboolean INV_SecurityKeyGive( gentity_t *target, const char *keyname );
pub unsafe extern "C" fn NPC_Touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*self_).NPC.is_null() {
        return;
    }

    SaveNPCGlobals();
    SetNPCGlobals(self_);

    if !(*self_).message.is_null() && (*self_).health <= 0 {
        //I am dead and carrying a key
        //if ( other && player && player->health > 0 && other == player )
        if !other.is_null()
            && !(*other).client.is_null()
            && ((*other).s.number as usize) < MAX_CLIENTS
        {
            //player touched me
            /*
            char *text;
            qboolean	keyTaken;
            //give him my key
            if ( Q_stricmp( "goodie", self->message ) == 0 )
            {//a goodie key
                if ( (keyTaken = INV_GoodieKeyGive( other )) == qtrue )
                {
                    text = "cp @SP_INGAME_TOOK_IMPERIAL_GOODIE_KEY";
                    G_AddEvent( other, EV_ITEM_PICKUP, (FindItemForInventory( INV_GOODIE_KEY )-bg_itemlist) );
                }
                else
                {
                    text = "cp @SP_INGAME_CANT_CARRY_GOODIE_KEY";
                }
            }
            else
            {//a named security key
                if ( (keyTaken = INV_SecurityKeyGive( player, self->message )) == qtrue )
                {
                    text = "cp @SP_INGAME_TOOK_IMPERIAL_SECURITY_KEY";
                    G_AddEvent( other, EV_ITEM_PICKUP, (FindItemForInventory( INV_SECURITY_KEY )-bg_itemlist) );
                }
                else
                {
                    text = "cp @SP_INGAME_CANT_CARRY_SECURITY_KEY";
                }
            }
            */
            //rwwFIXMEFIXME: support for goodie/security keys?
            /*
            if ( keyTaken )
            {//remove my key
                NPC_SetSurfaceOnOff( self, "l_arm_key", 0x00000002 );
                self->message = NULL;
                //FIXME: temp pickup sound
                G_Sound( player, G_SoundIndex( "sound/weapons/key_pkup.wav" ) );
                //FIXME: need some event to pass to cgame for sound/graphic/message?
            }
            //FIXME: temp message
            gi.SendServerCommand( NULL, text );
            */
        }
    }

    if !(*other).client.is_null() {
        //FIXME:  if pushing against another bot, both ucmd.rightmove = 127???
        //Except if not facing one another...
        if (*other).health > 0 {
            (*NPCInfo).touchedByPlayer = other;
        }

        if other == (*NPCInfo).goalEntity {
            (*NPCInfo).aiFlags |= NPCAI_TOUCHED_GOAL;
        }

        // !(self->svFlags&SVF_LOCKEDENEMY) && !(self->svFlags&SVF_IGNORE_ENEMIES) &&
        if (*other).flags & FL_NOTARGET == 0 {
            if (*(*self_).client).enemyTeam != 0 {
                //See if we bumped into an enemy
                if (*(*other).client).playerTeam == (*(*self_).client).enemyTeam {
                    //bumped into an enemy
                    if (*NPCInfo).behaviorState != BS_HUNT_AND_KILL && (*NPCInfo).tempBehavior == 0 {
                        //MCG - Begin: checking specific BS mode here, this is bad, a HACK
                        //FIXME: not medics?
                        if (*NPC).enemy != other {
                            //not already mad at them
                            G_SetEnemy(NPC, other);
                        }
                        //				NPCInfo->tempBehavior = BS_HUNT_AND_KILL;
                    }
                }
            }
        }

        //FIXME: do this if player is moving toward me and with a certain dist?
        /*
        if ( other->s.number == 0 && self->client->playerTeam == other->client->playerTeam )
        {
            VectorAdd( self->client->pushVec, other->client->ps.velocity, self->client->pushVec );
        }
        */
    } else {
        //FIXME: check for SVF_NONNPC_ENEMY flag here?
        if (*other).health > 0 {
            //if ( NPC->enemy == other && (other->svFlags&SVF_NONNPC_ENEMY) )
            if false
            //rwwFIXMEFIXME: Can probably just check if num < MAX_CLIENTS for non-npc enemy stuff
            {
                (*NPCInfo).touchedByPlayer = other;
            }
        }

        if other == (*NPCInfo).goalEntity {
            (*NPCInfo).aiFlags |= NPCAI_TOUCHED_GOAL;
        }
    }

    RestoreNPCGlobals();
}

/*
-------------------------
NPC_TempLookTarget
-------------------------
*/

pub unsafe fn NPC_TempLookTarget(
    self_: *mut gentity_t,
    lookEntNum: c_int,
    mut minLookTime: c_int,
    mut maxLookTime: c_int,
) {
    if (*self_).client.is_null() {
        return;
    }

    if (*(*self_).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
        //lookTarget is set by and to the monster that's holding you, no other operations can change that
        return;
    }

    if minLookTime == 0 {
        minLookTime = 1000;
    }

    if maxLookTime == 0 {
        maxLookTime = 1000;
    }

    if NPC_CheckLookTarget(self_) == QFALSE {
        //Not already looking at something else
        //Look at him for 1 to 3 seconds
        NPC_SetLookTarget(
            self_,
            lookEntNum,
            (*addr_of!(level)).time + Q_irand(minLookTime, maxLookTime),
        );
    }
}

pub unsafe fn NPC_Respond(self_: *mut gentity_t, userNum: c_int) {
    let mut event: c_int = -1;
    /*

    if ( Q_irand( 0, 1 ) )
    {
        event = Q_irand(EV_RESPOND1, EV_RESPOND3);
    }
    else
    {
        event = Q_irand(EV_BUSY1, EV_BUSY3);
    }
    */

    if Q_irand(0, 1) == 0 {
        //set looktarget to them for a second or two
        NPC_TempLookTarget(self_, userNum, 1000, 3000);
    }

    //some last-minute hacked in responses
    match (*(*self_).client).NPC_class {
        x if x == CLASS_JAN => {
            if !(*self_).enemy.is_null() {
                if Q_irand(0, 2) == 0 {
                    event = Q_irand(EV_CHASE1, EV_CHASE3);
                } else if Q_irand(0, 1) != 0 {
                    event = Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                } else {
                    event = Q_irand(EV_COVER1, EV_COVER5);
                }
            } else if Q_irand(0, 2) == 0 {
                event = EV_SUSPICIOUS4;
            } else if Q_irand(0, 1) == 0 {
                event = EV_SOUND1;
            } else {
                event = EV_CONFUSE1;
            }
        }
        x if x == CLASS_LANDO => {
            if !(*self_).enemy.is_null() {
                if Q_irand(0, 2) == 0 {
                    event = Q_irand(EV_CHASE1, EV_CHASE3);
                } else if Q_irand(0, 1) != 0 {
                    event = Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                } else {
                    event = Q_irand(EV_COVER1, EV_COVER5);
                }
            } else if Q_irand(0, 6) == 0 {
                event = EV_SIGHT2;
            } else if Q_irand(0, 5) == 0 {
                event = EV_GIVEUP4;
            } else if Q_irand(0, 4) > 1 {
                event = Q_irand(EV_SOUND1, EV_SOUND3);
            } else {
                event = Q_irand(EV_JDETECTED1, EV_JDETECTED2);
            }
        }
        x if x == CLASS_LUKE => {
            if !(*self_).enemy.is_null() {
                event = EV_COVER1;
            } else {
                event = Q_irand(EV_SOUND1, EV_SOUND3);
            }
        }
        x if x == CLASS_JEDI => {
            if (*self_).enemy.is_null() {
                /*
                if ( !(self->svFlags&SVF_IGNORE_ENEMIES)
                    && (self->NPC->scriptFlags&SCF_LOOK_FOR_ENEMIES)
                    && self->client->enemyTeam == TEAM_ENEMY )
                    */
                if false
                //rwwFIXMEFIXME: support flags!
                {
                    event = Q_irand(EV_ANGER1, EV_ANGER3);
                } else {
                    event = Q_irand(EV_TAUNT1, EV_TAUNT2);
                }
            }
        }
        x if x == CLASS_PRISONER => {
            if !(*self_).enemy.is_null() {
                if Q_irand(0, 1) != 0 {
                    event = Q_irand(EV_CHASE1, EV_CHASE3);
                } else {
                    event = Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                }
            } else {
                event = Q_irand(EV_SOUND1, EV_SOUND3);
            }
        }
        x if x == CLASS_REBEL => {
            if !(*self_).enemy.is_null() {
                if Q_irand(0, 2) == 0 {
                    event = Q_irand(EV_CHASE1, EV_CHASE3);
                } else {
                    event = Q_irand(EV_DETECTED1, EV_DETECTED5);
                }
            } else {
                event = Q_irand(EV_SOUND1, EV_SOUND3);
            }
        }
        x if x == CLASS_BESPIN_COP => {
            if Q_stricmp(c"bespincop".as_ptr(), (*self_).NPC_type) == 0 {
                //variant 1
                if !(*self_).enemy.is_null() {
                    if Q_irand(0, 9) > 6 {
                        event = Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if Q_irand(0, 6) > 4 {
                        event = Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if Q_irand(0, 3) == 0 {
                    event = Q_irand(EV_SIGHT2, EV_SIGHT3);
                } else if Q_irand(0, 1) == 0 {
                    event = Q_irand(EV_SOUND1, EV_SOUND3);
                } else if Q_irand(0, 2) == 0 {
                    event = EV_LOST1;
                } else if Q_irand(0, 1) == 0 {
                    event = EV_ESCAPING2;
                } else {
                    event = EV_GIVEUP4;
                }
            } else {
                //variant2
                if !(*self_).enemy.is_null() {
                    if Q_irand(0, 9) > 6 {
                        event = Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if Q_irand(0, 6) > 4 {
                        event = Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if Q_irand(0, 3) == 0 {
                    event = Q_irand(EV_SIGHT1, EV_SIGHT2);
                } else if Q_irand(0, 1) == 0 {
                    event = Q_irand(EV_SOUND1, EV_SOUND3);
                } else if Q_irand(0, 2) == 0 {
                    event = EV_LOST1;
                } else if Q_irand(0, 1) == 0 {
                    event = EV_GIVEUP3;
                } else {
                    event = EV_CONFUSE1;
                }
            }
        }
        x if x == CLASS_R2D2 => {
            // droid
            // C: G_Sound(self, CHAN_AUTO, G_SoundIndex(va("sound/chars/r2d2/misc/r2d2talk0%d.wav",Q_irand(1, 3))));
            // The va() format-buffer collapses into the &str G_SoundIndex marshals (cf. g_missile.rs).
            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex(&format!(
                    "sound/chars/r2d2/misc/r2d2talk0{}.wav",
                    Q_irand(1, 3)
                )),
            );
        }
        x if x == CLASS_R5D2 => {
            // droid
            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex(&format!("sound/chars/r5d2/misc/r5talk{}.wav", Q_irand(1, 4))),
            );
        }
        x if x == CLASS_MOUSE => {
            // droid
            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex(&format!(
                    "sound/chars/mouse/misc/mousego{}.wav",
                    Q_irand(1, 3)
                )),
            );
        }
        x if x == CLASS_GONK => {
            // droid
            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex(&format!(
                    "sound/chars/gonk/misc/gonktalk{}.wav",
                    Q_irand(1, 2)
                )),
            );
        }
        _ => {}
    }

    if event != -1 {
        //hack here because we reuse some "combat" and "extra" sounds
        let addFlag: qboolean = ((*(*self_).NPC).scriptFlags & SCF_NO_COMBAT_TALK != 0) as qboolean;
        (*(*self_).NPC).scriptFlags &= !SCF_NO_COMBAT_TALK;

        G_AddVoiceEvent(self_, event, 3000);

        if addFlag != 0 {
            (*(*self_).NPC).scriptFlags |= SCF_NO_COMBAT_TALK;
        }
    }
}

/*
-------------------------
NPC_UseResponse
-------------------------
*/

pub unsafe fn NPC_UseResponse(self_: *mut gentity_t, user: *mut gentity_t, useWhenDone: qboolean) {
    if (*self_).NPC.is_null() || (*self_).client.is_null() {
        return;
    }

    if (*user).s.number != 0 {
        //not used by the player
        if useWhenDone != QFALSE {
            G_ActivateBehavior(self_, BSET_USE);
        }
        return;
    }

    if !(*user).client.is_null()
        && (*(*self_).client).playerTeam != (*(*user).client).playerTeam
        && (*(*self_).client).playerTeam != NPCTEAM_NEUTRAL
    {
        //only those on the same team react
        if useWhenDone != QFALSE {
            G_ActivateBehavior(self_, BSET_USE);
        }
        return;
    }

    if (*(*self_).NPC).blockedSpeechDebounceTime > (*addr_of!(level)).time {
        //I'm not responding right now
        return;
    }

    /*
    if ( gi.VoiceVolume[self->s.number] )
    {//I'm talking already
        if ( !useWhenDone )
        {//you're not trying to use me
            return;
        }
    }
    */
    //rwwFIXMEFIXME: Support for this?

    if useWhenDone != QFALSE {
        G_ActivateBehavior(self_, BSET_USE);
    } else {
        NPC_Respond(self_, (*user).s.number);
    }
}

/// `void NPC_Use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (NPC_reactions.c:1010) — the NPC `use` callback. Dead NPCs ignore it; otherwise
/// (under Save/Set/RestoreNPCGlobals) a vehicle NPC boards/ejects `other` via its
/// vehicle vtable, a waiting-ambush Jedi springs the ambush, and any NPC with a
/// `BSET_USE` script runs it — falling back to a verbal [`NPC_UseResponse`] when the
/// player uses an idle, responsive NPC. The MEDIC-heal path and the GONK battery
/// transfer are commented out in the C original (kept verbatim below).
///
/// No oracle: drives NPC/vehicle/script globals and indirect vtable calls.
///
/// # Safety
/// `self`/`other`/`activator` must be valid `gentity_t*`; for a `CLASS_VEHICLE` self,
/// `m_pVehicle`/`m_pVehicleInfo` and the invoked vtable thunks must be valid.
pub unsafe extern "C" fn NPC_Use(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*(*self_).client).ps.pm_type == PM_DEAD {
        // or just remove ->pain in player_die?
        return;
    }

    SaveNPCGlobals();
    SetNPCGlobals(self_);

    if !(*self_).client.is_null() && !(*self_).NPC.is_null() {
        // If this is a vehicle, let the other guy board it. Added 12/14/02 by AReis.
        if (*(*self_).client).NPC_class == CLASS_VEHICLE {
            let pVeh = (*self_).m_pVehicle;

            if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
                if other == self_ {
                    // if I used myself, eject everyone on me
                    ((*(*pVeh).m_pVehicleInfo).EjectAll.unwrap())(pVeh);
                } else if (*other).s.owner == (*self_).s.number {
                    // If other is already riding this vehicle (self), eject him.
                    ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(
                        pVeh,
                        other as *mut bgEntity_t,
                        QFALSE,
                    );
                } else {
                    // Otherwise board this vehicle.
                    ((*(*pVeh).m_pVehicleInfo).Board.unwrap())(pVeh, other as *mut bgEntity_t);
                }
            }
        } else if Jedi_WaitingAmbush(NPC) != QFALSE {
            Jedi_Ambush(NPC);
        }
        // Run any use instructions
        if !activator.is_null()
            && (*activator).s.number == 0
            && (*(*self_).client).NPC_class == CLASS_GONK
        {
            // must be using the gonk, so attempt to give battery power.
            // NOTE: this will steal up to MAX_BATTERIES for the activator, leaving the
            // residual on the gonk for potential later use.
            //			Add_Batteries( activator, &self->client->ps.batteryCharge );
            //rwwFIXMEFIXME: support for this?
        }
        // Not using MEDICs anymore
        /*
                if ( self->NPC->behaviorState == BS_MEDIC_HIDE && activator->client )
                {//Heal me NOW, dammit!
                    if ( activator->health < activator->client->ps.stats[STAT_MAX_HEALTH] )
                    {//person needs help
                        if ( self->NPC->eventualGoal != activator )
                        {//not my current patient already
                            NPC_TakePatient( activator );
                            G_ActivateBehavior( self, BSET_USE );
                        }
                    }
                    else if ( !self->enemy && activator->s.number == 0 && !gi.VoiceVolume[self->s.number] && !(self->NPC->scriptFlags&SCF_NO_RESPONSE) )
                    {//I don't have an enemy and I'm not talking and I was used by the player
                        NPC_UseResponse( self, other, qfalse );
                    }
                }
        */
        //		else if ( self->behaviorSet[BSET_USE] )
        if !(*self_).behaviorSet[BSET_USE as usize].is_null() {
            NPC_UseResponse(self_, other, QTRUE);
        }
        //		else if ( isMedic( self ) )
        //		{//Heal me NOW, dammit!
        //			NPC_TakePatient( activator );
        //		}
        else if (*self_).enemy.is_null()
            && (*activator).s.number == 0
            // !gi.VoiceVolume[self->s.number] &&  rwwFIXMEFIXME: voice volume support?
            && ((*(*self_).NPC).scriptFlags & SCF_NO_RESPONSE) == 0
        {
            // I don't have an enemy and I'm not talking and I was used by the player
            NPC_UseResponse(self_, other, QFALSE);
        }
    }

    RestoreNPCGlobals();
}

pub fn NPC_CheckPlayerAim() {
    //FIXME: need appropriate dialogue
    /*
    gentity_t *player = &g_entities[0];

    if ( player && player->client && player->client->ps.weapon > (int)(WP_NONE) && player->client->ps.weapon < (int)(WP_TRICORDER) )
    {//player has a weapon ready
        if ( g_crosshairEntNum == NPC->s.number && level.time - g_crosshairEntTime < 200
            && g_crosshairSameEntTime >= 3000 && g_crosshairEntDist < 256 )
        {//if the player holds the crosshair on you for a few seconds
            //ask them what the fuck they're doing
            G_AddVoiceEvent( NPC, Q_irand( EV_FF_1A, EV_FF_1C ), 0 );
        }
    }
    */
}

pub fn NPC_CheckAllClear() {
    //FIXME: need to make this happen only once after losing enemies, not over and over again
    /*
    if ( NPC->client && !NPC->enemy && level.time - teamLastEnemyTime[NPC->client->playerTeam] > 10000 )
    {//Team hasn't seen an enemy in 10 seconds
        if ( !Q_irand( 0, 2 ) )
        {
            G_AddVoiceEvent( NPC, Q_irand(EV_SETTLE1, EV_SETTLE3), 3000 );
        }
    }
    */
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::codemp::game::g_local::{gclient_t, gentity_t};
    use crate::codemp::game::bg_public::STAT_MAX_HEALTH;
    use core::ptr::addr_of_mut;

    extern "C" {
        // NPC_reactions.c:157 — scalar-marshaled (see npc_reactions_oracle.c).
        fn jka_npc_getpainchance(
            has_enemy: i32,
            has_client: i32,
            health: i32,
            max_health: i32,
            damage: i32,
            spskill: i32,
        ) -> f32;
    }

    #[test]
    fn npc_getpainchance_matches_oracle() {
        // A spread covering: no-enemy short-circuit, no-client short-circuit, the
        // damage > max_health/2 short-circuit, and the three g_spskill branches with
        // varied health/max_health/damage.
        let cases: &[(bool, bool, i32, i32, i32)] = &[
            (false, true, 50, 100, 10),  // no enemy -> 1.0
            (true, false, 50, 100, 10),  // no client -> 1.0
            (true, true, 50, 100, 60),   // damage > max/2 -> 1.0
            (true, true, 100, 100, 10),  // full health
            (true, true, 50, 100, 10),
            (true, true, 1, 100, 49),
            (true, true, 75, 200, 30),
            (true, true, 33, 66, 5),
        ];

        for &spskill in &[0i32, 1, 2, 3] {
            unsafe {
                (*addr_of_mut!(g_spskill)).integer = spskill;
            }
            for &(has_enemy, has_client, health, max_health, damage) in cases {
                let mut enemy: gentity_t = unsafe { core::mem::zeroed() };
                let mut client: gclient_t = unsafe { core::mem::zeroed() };
                client.ps.stats[STAT_MAX_HEALTH as usize] = max_health;

                let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                ent.health = health;
                if has_enemy {
                    ent.enemy = &mut enemy;
                }
                if has_client {
                    ent.client = &mut client;
                }

                let rust = unsafe { NPC_GetPainChance(&mut ent, damage) };
                let c = unsafe {
                    jka_npc_getpainchance(
                        has_enemy as i32,
                        has_client as i32,
                        health,
                        max_health,
                        damage,
                        spskill,
                    )
                };
                assert_eq!(
                    rust.to_bits(),
                    c.to_bits(),
                    "NPC_GetPainChance enemy={has_enemy} client={has_client} h={health} max={max_health} dmg={damage} sk={spskill}"
                );
            }
        }
    }
}
