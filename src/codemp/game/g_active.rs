//! Port of `g_active.c` — per-frame client active-state processing (movement,
//! damage feedback, inactivity, view, weapon/think hooks).
//!
//! Landed incrementally: only the leaves whose deps are already ported. Pure
//! classifiers ([`G_StandingAnim`], [`G_ActionButtonPressed`]) and the computational
//! [`ClientTimerActions`]/[`G_AddPushVecToUcmd`] (extracted-scalar oracles) are
//! oracle-tested; the entity/client-state mutators ([`P_SetTwitchInfo`],
//! [`P_DamageFeedback`], [`G_SetClientSound`], [`ClientInactivityTimer`]) touch the
//! process-global `level`/cvar statics + traps, so they follow the no-oracle
//! convention (same as the rest of the `g_*` client code). The ghoul2 vehicle hook
//! [`G_VehicleAttachDroidUnit`] (g_active.c:830) likewise follows the no-oracle convention.

#![allow(non_snake_case)] // C function names (`G_StandingAnim`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::anims::{
    animNumber_t, BOTH_BOW, BOTH_DUAL_TAUNT, BOTH_ENGAGETAUNT, BOTH_GESTURE1, BOTH_MEDITATE,
    BOTH_SHOWOFF_DUAL, BOTH_SHOWOFF_FAST, BOTH_SHOWOFF_MEDIUM, BOTH_SHOWOFF_STAFF,
    BOTH_SHOWOFF_STRONG, BOTH_STAFF_TAUNT, BOTH_STAND1, BOTH_STAND1IDLE1, BOTH_STAND2,
    BOTH_STAND2IDLE1, BOTH_STAND2IDLE2, BOTH_STAND3, BOTH_STAND3IDLE1, BOTH_STAND4, BOTH_STAND5,
    BOTH_STAND5IDLE1, BOTH_VICTORY_DUAL, BOTH_VICTORY_FAST, BOTH_VICTORY_MEDIUM,
    BOTH_VICTORY_STAFF, BOTH_VICTORY_STRONG, MAX_ANIMATIONS, TORSO_RAISEWEAP1,
};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_vehicles_h::{VH_SPEEDER, VH_WALKER};
use crate::codemp::game::bg_misc::{
    vectoyaw, BG_GiveMeVectorFromMatrix, BG_PlayerStateToEntityState,
    BG_PlayerStateToEntityStateExtraPolate, BG_PlayerTouchesItem,
};
use crate::codemp::game::bg_panimate::{
    bgAllAnims, BG_AnimLength, BG_InKnockDownOnly, BG_SaberInAttack, PM_SaberInReturn,
    PM_SaberInStart, PM_SaberInTransition,
};
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_GONK, CLASS_MARK1, CLASS_MARK2, CLASS_MOUSE, CLASS_PROBE, CLASS_PROTOCOL,
    CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REMOTE, CLASS_SEEKER, CLASS_VEHICLE, NPCTEAM_PLAYER,
};
use crate::codemp::game::bg_pmove::Pmove;
use crate::codemp::game::bg_public::{
    bgEntity_t, pmove_t, EF_DEAD, EF_FIRING, EF_INVULNERABLE, EF_PLAYER_EVENT, EF_TALK, ET_EVENTS,
    ET_ITEM, ET_NPC, ET_PUSH_TRIGGER, ET_TELEPORT_TRIGGER, EV_ALT_FIRE, EV_FIRE_WEAPON, EV_PAIN,
    EV_POWERUP_BATTLESUIT, EV_TAUNT, GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, HANDEXTEND_NONE,
    HANDEXTEND_TAUNT, LS_READY, MASK_PLAYERSOLID, MOD_LAVA, MOD_SLIME, MOD_WATER, PMF_FOLLOW,
    PM_DEAD, PM_SPECTATOR, PW_BATTLESUIT, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    PMF_SCOREBOARD, SETANIM_FLAG_OVERRIDE, STAT_ARMOR, STAT_HEALTH, STAT_MAX_HEALTH,
    TEAM_SPECTATOR, WEAPON_CHARGING, WEAPON_CHARGING_ALT, WEAPON_READY, MOD_CRUSH, MOD_FALLING,
    DF_NO_FOOTSTEPS,
    DF_NO_FALLING, ET_PLAYER, EV_FALL, EV_ROLL, EV_SABER_ATTACK, EV_USE_ITEM0, EV_USE_ITEM1,
    EV_USE_ITEM2, EV_USE_ITEM3, EV_USE_ITEM4, EV_USE_ITEM5, EV_USE_ITEM6, EV_USE_ITEM7,
    EV_USE_ITEM8, EV_USE_ITEM9, EV_USE_ITEM10, EV_USE_ITEM11, GT_SIEGE,
    EF_BODYPUSH, EF_CONNECTION, EF_DISINTEGRATION, EF_JETPACK, EF_JETPACK_ACTIVE,
    EF_JETPACK_FLAMING, EF_NODRAW,
    EF2_GENERIC_NPC_FLAG, EF2_HELD_BY_MONSTER, EF2_SHIP_DEATH, ET_BODY, EV_PRIVATE_DUEL,
    HANDEXTEND_DRAGGING, HANDEXTEND_KNOCKDOWN, HANDEXTEND_POSTTHROW, HANDEXTEND_POSTTHROWN,
    HANDEXTEND_WEAPONREADY, HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP,
    HI_JETPACK, HI_MEDPAC, HI_MEDPAC_BIG, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD, MOD_MELEE, MOD_SABER,
    PDSOUND_FORCEJUMP, PM_JETPACK, PM_NOCLIP, PM_NORMAL, PW_CLOAKED, STAT_HOLDABLE_ITEMS,
};
use crate::codemp::game::bg_weapons_h::{WP_MELEE, WP_SABER};
use crate::codemp::game::g_client::{ClientBegin, SetClientViewAngle};
use crate::codemp::game::g_cmds::{
    Cmd_EngageDuel_f, Cmd_FollowCycle_f, Cmd_SaberAttackCycle_f, Cmd_ToggleSaber_f, G_ItemUsable,
    SetTeam, StopFollowing,
};
use crate::codemp::game::ai_main::InFieldOfVision;
use crate::codemp::game::g_combat::{
    gGAvoidDismember, G_ApplyKnockback, G_CheckForDismemberment, G_Damage,
};
use crate::codemp::game::g_local::{
    clientPersistant_t, gclient_t, gentity_t, CON_CONNECTED, DAMAGE_NO_ARMOR, DAMAGE_NO_PROTECTION,
    FL_BBRUSH, FL_FORCE_GESTURE, FL_GODMODE, SPECTATOR_FOLLOW, SPECTATOR_FREE, SPECTATOR_SCOREBOARD,
};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::w_force::{
    ForceAbsorb, ForceHeal, ForceProtect, ForceRage, ForceSeeing, ForceSpeed, ForceTeamForceReplenish,
    ForceTeamHeal, ForceThrow, G_PreDefSound,
};
use crate::codemp::game::g_weapon::FireWeapon;
use crate::codemp::game::g_items::{
    ItemUse_Binoculars, ItemUse_Jetpack, ItemUse_MedPack, ItemUse_MedPack_Big, ItemUse_Seeker,
    ItemUse_Shield, ItemUse_UseEWeb,
};
use crate::codemp::game::g_main::{
    g_debugMelee, g_debugMove, g_dmflags, g_entities, g_forcerespawn, g_friendlyFire, g_gametype,
    g_gravity, g_inactivity, g_noSpecMove, g_saberLockRandomNess, g_siegeRespawn, g_smoothClients,
    g_spawnInvulnerability, g_speed, g_stepSlideFix, g_synchronousClients, g_timeouttospec,
    gDoSlowMoDuel, level, pmove_fixed, pmove_msec, G_GetStringEdString,
};
use crate::codemp::game::g_client::respawn;
use crate::codemp::game::g_utils::{G_EntitySound, G_MuteSound};
use crate::codemp::game::q_shared::{va, Sz};
use crate::codemp::game::g_mover::Touch_DoorTrigger;
use crate::codemp::game::g_public_h::{SVF_BOT, SVF_GLASS_BRUSH, SVF_NOTSINGLECLIENT};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_SetAngles, G_SetAnim, G_SetOrigin, G_Sound, G_SoundIndex, G_TempEntity,
};
use crate::codemp::game::bg_g2_utils::BG_AttachToRancor;
use crate::codemp::game::g_nav::FlyingCreature;
use crate::codemp::game::npc::NPC_SetAnim;
use crate::codemp::game::q_math::{
    vectoangles, AngleVectors, DotProduct, VectorAdd, VectorClear, VectorCompare, VectorCopy,
    VectorLength, VectorLengthSquared, VectorMA, VectorNormalize, VectorNormalize2, VectorScale,
    VectorSet, VectorSubtract, vec3_origin,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, trace_t, usercmd_t, vec3_t, vec_t, BLOCKED_NONE, BUTTON_ALT_ATTACK,
    BUTTON_ATTACK, BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING, BUTTON_FORCEGRIP, BUTTON_FORCEPOWER,
    BUTTON_GESTURE, BUTTON_USE, BUTTON_USE_HOLDABLE, CHAN_AUTO, CHAN_VOICE, CHAN_WEAPON,
    ENTITYNUM_NONE, FP_SEE,
    MAX_CLIENTS, MAX_GENTITIES, MAX_PS_EVENTS, PITCH, SS_DESANN, SS_DUAL, SS_FAST, SS_MEDIUM,
    SS_STAFF, SS_STRONG, SS_TAVION, YAW, QFALSE, QTRUE, ENTITYNUM_WORLD, TR_GRAVITY, MAT_GLASS,
    MAT_GLASS_METAL, MAT_GRATE1, MAX_POWERUPS,
    mdxaBone_t, BUTTON_TALK, FORCE_LEVEL_3, FP_RAGE,
    GENCMD_BOW, GENCMD_ENGAGE_DUEL, GENCMD_FLOURISH, GENCMD_FORCE_ABSORB, GENCMD_FORCE_DISTRACT,
    GENCMD_FORCE_FORCEPOWEROTHER, GENCMD_FORCE_HEAL, GENCMD_FORCE_HEALOTHER, GENCMD_FORCE_PROTECT,
    GENCMD_FORCE_PULL, GENCMD_FORCE_RAGE, GENCMD_FORCE_SEEING, GENCMD_FORCE_SPEED,
    GENCMD_FORCE_THROW, GENCMD_GLOAT, GENCMD_MEDITATE, GENCMD_SABERATTACKCYCLE, GENCMD_SABERSWITCH,
    GENCMD_TAUNT, GENCMD_USE_AMMODISP, GENCMD_USE_BACTA, GENCMD_USE_BACTABIG, GENCMD_USE_CLOAK,
    GENCMD_USE_ELECTROBINOCULARS, GENCMD_USE_EWEB, GENCMD_USE_FIELD, GENCMD_USE_HEALTHDISP,
    GENCMD_USE_JETPACK, GENCMD_USE_SEEKER, GENCMD_USE_SENTRY, GENCMD_ZOOM, NEGATIVE_Y, ORIGIN,
    ROLL, SS_NONE, SS_NUM_SABER_STYLES,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_TRIGGER, CONTENTS_WATER,
};

// `taunt` selector for [`G_SetTauntAnim`]. In C this is an anonymous enum duplicated
// (un-named) in cg_event.c; the values are the implicit 0..4 of declaration order.
// Kept file-local and faithful to that ordering.
// `FALL_FADE_TIME` (q_shared.h:2148, retail-PC raven-jediacademy) — fade-out window before a
// fall-to-death finalises. Defined file-locally (the full q_shared.h macro set is ported
// incrementally) and used by [`ClientThink_real`]'s falling-to-death check. The Xbox/grayj tree
// had 1200; retail PC raised it to 3000.
const FALL_FADE_TIME: c_int = 3000;

const TAUNT_TAUNT: c_int = 0;
const TAUNT_BOW: c_int = 1;
const TAUNT_MEDITATE: c_int = 2;
const TAUNT_FLOURISH: c_int = 3;
const TAUNT_GLOAT: c_int = 4;

/// `void P_SetTwitchInfo(gclient_t *client)` (g_active.c:24). Stamp the pain twitch
/// info onto the client's player-state: record the time of the hit and toggle the twitch
/// direction. No oracle — mutates the `gclient_t` and reads the process-global `level.time`.
///
/// # Safety
/// `client` must point to a valid `gclient_t`; the `level` global must be initialised.
pub unsafe fn P_SetTwitchInfo(client: *mut gclient_t) {
    (*client).ps.painTime = (*addr_of!(level)).time;
    (*client).ps.painDirection ^= 1;
}

/// `void P_DamageFeedback( gentity_t *player )` (g_active.c:40). Called just before a snapshot
/// is sent to the given player. Totals up all damage and generates both the `player_state_t`
/// damage values to that client for pain blends and kicks, and global pain sound events for all
/// clients. No oracle — entity-state side-effecting (mutates the `gclient_t`/`gentity_t`, reads
/// the process-global `level.time`, emits a `G_AddEvent`).
///
/// # Safety
/// `player` must point to a valid `gentity_t` whose `client` is non-NULL; the `level` global must
/// be initialised.
pub unsafe fn P_DamageFeedback(player: *mut gentity_t) {
    let client: *mut gclient_t;
    let mut count: f32;
    let mut angles: vec3_t = [0.0; 3];

    client = (*player).client;
    if (*client).ps.pm_type == PM_DEAD {
        return;
    }

    // total points of damage shot at the player this frame
    count = ((*client).damage_blood + (*client).damage_armor) as f32;
    if count == 0.0 {
        return; // didn't take any damage
    }

    if count > 255.0 {
        count = 255.0;
    }

    // send the information to the client

    // world damage (falling, slime, etc) uses a special code
    // to make the blend blob centered instead of positional
    if (*client).damage_fromWorld != QFALSE {
        (*client).ps.damagePitch = 255;
        (*client).ps.damageYaw = 255;

        (*client).damage_fromWorld = QFALSE;
    } else {
        vectoangles(&(*client).damage_from, &mut angles);
        (*client).ps.damagePitch = (angles[PITCH] / 360.0 * 256.0) as c_int;
        (*client).ps.damageYaw = (angles[YAW] / 360.0 * 256.0) as c_int;

        //cap them since we can't send negative values in here across the net
        if (*client).ps.damagePitch < 0 {
            (*client).ps.damagePitch = 0;
        }
        if (*client).ps.damageYaw < 0 {
            (*client).ps.damageYaw = 0;
        }
    }

    // play an apropriate pain sound
    if ((*addr_of!(level)).time > (*player).pain_debounce_time)
        && ((*player).flags & FL_GODMODE) == 0
        && ((*player).s.eFlags & EF_DEAD) == 0
    {
        // don't do more than two pain sounds a second
        // nmckenzie: also don't make him loud and whiny if he's only getting nicked.
        if (*addr_of!(level)).time - (*client).ps.painTime < 500 || count < 10.0 {
            return;
        }
        P_SetTwitchInfo(client);
        (*player).pain_debounce_time = (*addr_of!(level)).time + 700;

        G_AddEvent(player, EV_PAIN, (*player).health);
        (*client).ps.damageEvent += 1;

        if (*client).damage_armor != 0 && (*client).damage_blood == 0 {
            (*client).ps.damageType = 1; //pure shields
        } else if (*client).damage_armor != 0 {
            (*client).ps.damageType = 2; //shields and health
        } else {
            (*client).ps.damageType = 0; //pure health
        }
    }

    (*client).ps.damageCount = count as c_int;

    //
    // clear totals
    //
    (*client).damage_blood = 0;
    (*client).damage_armor = 0;
    (*client).damage_knockback = 0;
}

//==============================================================
/// `void DoImpact( gentity_t *self, gentity_t *other, qboolean damageSelf )` (g_active.c:217).
/// Resolve a physics impact between `self` and `other`: compute the impact magnitude from
/// `self`'s velocity and mass, then (subject to easy-break-brush leniency and ground-contact
/// timing) damage/knock-back `other` and optionally apply falling damage to `self`. No oracle —
/// dereferences `gentity_t`/`gclient_t` fields, calls `G_Damage`/`G_ApplyKnockback`/
/// `trap_PointContents`, and reads the process-global `level`/`g_gravity`.
///
/// # Safety
/// `self` and `other` must point to valid `gentity_t`; the `level` global and `g_gravity` cvar
/// must be initialised.
pub unsafe fn DoImpact(self_: *mut gentity_t, other: *mut gentity_t, damageSelf: qboolean) {
    let mut magnitude: f32;
    let my_mass: f32;
    let mut velocity: vec3_t = [0.0; 3];
    let cont: c_int;
    let mut easyBreakBrush: qboolean = QTRUE;

    if !(*self_).client.is_null() {
        VectorCopy(&(*(*self_).client).ps.velocity, &mut velocity);
        if (*self_).mass == 0.0 {
            my_mass = 10.0;
        } else {
            my_mass = (*self_).mass;
        }
    } else {
        VectorCopy(&(*self_).s.pos.trDelta, &mut velocity);
        if (*self_).s.pos.trType == TR_GRAVITY {
            velocity[2] -= 0.25f32 * (*addr_of!(g_gravity)).value;
        }
        if (*self_).mass == 0.0 {
            my_mass = 1.0;
        } else if (*self_).mass <= 10.0 {
            my_mass = 10.0;
        } else {
            my_mass = (*self_).mass; //10;
        }
    }

    magnitude = VectorLength(&velocity) * my_mass / 10.0;

    /*
    if(pointcontents(self.absmax)==CONTENT_WATER)//FIXME: or other watertypes
        magnitude/=3;							//water absorbs 2/3 velocity

    if(self.classname=="barrel"&&self.aflag)//rolling barrels are made for impacts!
        magnitude*=3;

    if(self.frozen>0&&magnitude<300&&self.flags&FL_ONGROUND&&loser==world&&self.velocity_z<-20&&self.last_onground+0.3<time)
        magnitude=300;
    */
    if (*other).material == MAT_GLASS
        || (*other).material == MAT_GLASS_METAL
        || (*other).material == MAT_GRATE1
        || ((*other).flags & FL_BBRUSH) != 0 && ((*other).spawnflags & 8/*THIN*/) != 0
        || ((*other).r.svFlags & SVF_GLASS_BRUSH) != 0
    {
        easyBreakBrush = QTRUE;
    }

    if (*self_).client.is_null()
        || (*(*self_).client).ps.lastOnGround + 300 < (*addr_of!(level)).time
        || ((*(*self_).client).ps.lastOnGround + 100 < (*addr_of!(level)).time
            && easyBreakBrush != QFALSE)
    {
        let mut dir1: vec3_t = [0.0; 3];
        let mut dir2: vec3_t = [0.0; 3];
        let mut force: f32 = 0.0;
        let dot: f32;

        if easyBreakBrush != QFALSE {
            magnitude *= 2.0;
        }

        //damage them
        if magnitude >= 100.0 && (*other).s.number < ENTITYNUM_WORLD {
            VectorCopy(&velocity, &mut dir1);
            VectorNormalize(&mut dir1);
            if VectorCompare(&(*other).r.currentOrigin, &vec3_origin) != QFALSE {
                //a brush with no origin
                VectorCopy(&dir1, &mut dir2);
            } else {
                VectorSubtract(&(*other).r.currentOrigin, &(*self_).r.currentOrigin, &mut dir2);
                VectorNormalize(&mut dir2);
            }

            dot = DotProduct(&dir1, &dir2);

            if dot >= 0.2 {
                force = dot;
            } else {
                force = 0.0;
            }

            force *= magnitude / 50.0;

            cont = crate::trap::PointContents(&(*other).r.absmax, (*other).s.number);
            if (cont & CONTENTS_WATER) != 0
            //|| (self.classname=="barrel"&&self.aflag))//FIXME: or other watertypes
            {
                force /= 3.0; //water absorbs 2/3 velocity
            }

            /*
            if(self.frozen>0&&force>10)
                force=10;
            */

            if (force >= 1.0 && (*other).s.number != 0) || force >= 10.0 {
                /*
                dprint("Damage other (");
                dprint(loser.classname);
                dprint("): ");
                dprint(ftos(force));
                dprint("\n");
                */
                if (*other).r.svFlags & SVF_GLASS_BRUSH != 0 {
                    (*other).splashRadius =
                        (((*self_).r.maxs[0] - (*self_).r.mins[0]) / 4.0f32) as c_int;
                }
                if (*other).takedamage != QFALSE {
                    G_Damage(
                        other,
                        self_,
                        self_,
                        velocity.as_mut_ptr() as *mut vec3_t,
                        addr_of_mut!((*self_).r.currentOrigin),
                        force as c_int,
                        DAMAGE_NO_ARMOR,
                        MOD_CRUSH,
                    ); //FIXME: MOD_IMPACT
                } else {
                    G_ApplyKnockback(other, &dir2, force);
                }
            }
        }

        if damageSelf != QFALSE && (*self_).takedamage != QFALSE {
            //Now damage me
            //FIXME: more lenient falling damage, especially for when driving a vehicle
            if !(*self_).client.is_null() && (*(*self_).client).ps.fd.forceJumpZStart != 0.0 {
                //we were force-jumping
                if (*self_).r.currentOrigin[2] >= (*(*self_).client).ps.fd.forceJumpZStart {
                    //we landed at same height or higher than we landed
                    magnitude = 0.0;
                } else {
                    //FIXME: take off some of it, at least?
                    magnitude = ((*(*self_).client).ps.fd.forceJumpZStart
                        - (*self_).r.currentOrigin[2])
                        / 3.0;
                }
            }
            //if(self.classname!="monster_mezzoman"&&self.netname!="spider")//Cats always land on their feet
            if (magnitude >= 100.0 + (*self_).health as f32
                && (*self_).s.number != 0
                && (*self_).s.weapon != WP_SABER)
                || (magnitude >= 700.0)
            //&& self.safe_time < level.time ))//health here is used to simulate structural integrity
            {
                if ((*self_).s.weapon == WP_SABER || (*self_).s.number == 0)
                    && !(*self_).client.is_null()
                    && (*(*self_).client).ps.groundEntityNum < ENTITYNUM_NONE
                    && magnitude < 1000.0
                {
                    //players and jedi take less impact damage
                    //allow for some lenience on high falls
                    magnitude /= 2.0;
                    /*
                    if ( self.absorb_time >= time )//crouching on impact absorbs 1/2 the damage
                    {
                        magnitude/=2;
                    }
                    */
                }
                magnitude /= 40.0;
                magnitude = magnitude - force / 2.0; //If damage other, subtract half of that damage off of own injury
                if magnitude >= 1.0 {
                    //FIXME: Put in a thingtype impact sound function
                    /*
                    dprint("Damage self (");
                    dprint(self.classname);
                    dprint("): ");
                    dprint(ftos(magnitude));
                    dprint("\n");
                    */
                    /*
                    if ( self.classname=="player_sheep "&& self.flags&FL_ONGROUND && self.velocity_z > -50 )
                        return;
                    */
                    G_Damage(
                        self_,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        addr_of_mut!((*self_).r.currentOrigin),
                        (magnitude / 2.0) as c_int,
                        DAMAGE_NO_ARMOR,
                        MOD_FALLING,
                    ); //FIXME: MOD_IMPACT
                }
            }
        }

        //FIXME: slow my velocity some?

        // NOTENOTE We don't use lastimpact as of yet
        //		self->lastImpact = level.time;

        /*
        if(self.flags&FL_ONGROUND)
            self.last_onground=time;
        */
    }
}

/// `void Client_CheckImpactBBrush( gentity_t *self, gentity_t *other )` (g_active.c:411).
/// Thin guard around [`DoImpact`]: clients only do impact damage against easy-break breakables,
/// and never as spectators. No oracle — entity-state guard delegating to `DoImpact`.
///
/// # Safety
/// `self` and `other` must point to valid `gentity_t` (or be null); the `level` global must be
/// initialised.
pub unsafe fn Client_CheckImpactBBrush(self_: *mut gentity_t, other: *mut gentity_t) {
    if other.is_null() || (*other).inuse == QFALSE {
        return;
    }
    if self_.is_null()
        || (*self_).inuse == QFALSE
        || (*self_).client.is_null()
        || (*(*self_).client).tempSpectate >= (*addr_of!(level)).time
        || (*(*self_).client).sess.sessionTeam == TEAM_SPECTATOR
    {
        //hmm.. let's not let spectators ram into breakables.
        return;
    }

    /*
    if (BG_InSpecialJump(self->client->ps.legsAnim))
    { //don't do this either, qa says it creates "balance issues"
        return;
    }
    */

    if (*other).material == MAT_GLASS
        || (*other).material == MAT_GLASS_METAL
        || (*other).material == MAT_GRATE1
        || ((*other).flags & FL_BBRUSH) != 0 && ((*other).spawnflags & 8/*THIN*/) != 0
        || ((*other).flags & FL_BBRUSH) != 0 && ((*other).health <= 10)
        || ((*other).r.svFlags & SVF_GLASS_BRUSH) != 0
    {
        //clients only do impact damage against easy-break breakables
        DoImpact(self_, other, QFALSE);
    }
}

/// `void G_SetClientSound( gentity_t *ent )` (g_active.c:448). Pick the client's looping sound:
/// hacking, med-heal, med-supply, or lava/slime fry, else silence. No oracle — pure entity/client
/// field assignments reading the process-global `level` sound indices.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the `level` global must be initialised.
pub unsafe fn G_SetClientSound(ent: *mut gentity_t) {
    if !(*ent).client.is_null() && (*(*ent).client).isHacking != 0 {
        //loop hacking sound
        (*(*ent).client).ps.loopSound = (*addr_of!(level)).snd_hack;
        (*ent).s.loopIsSoundset = QFALSE;
    } else if !(*ent).client.is_null() && (*(*ent).client).isMedHealed > (*addr_of!(level)).time {
        //loop healing sound
        (*(*ent).client).ps.loopSound = (*addr_of!(level)).snd_medHealed;
        (*ent).s.loopIsSoundset = QFALSE;
    } else if !(*ent).client.is_null() && (*(*ent).client).isMedSupplied > (*addr_of!(level)).time {
        //loop supplying sound
        (*(*ent).client).ps.loopSound = (*addr_of!(level)).snd_medSupplied;
        (*ent).s.loopIsSoundset = QFALSE;
    } else if (*ent).waterlevel != 0 && ((*ent).watertype & (CONTENTS_LAVA | CONTENTS_SLIME)) != 0 {
        (*(*ent).client).ps.loopSound = (*addr_of!(level)).snd_fry;
        (*ent).s.loopIsSoundset = QFALSE;
    } else {
        (*(*ent).client).ps.loopSound = 0;
        (*ent).s.loopIsSoundset = QFALSE;
    }
}

/// `qboolean ClientInactivityTimer( gclient_t *client )` (g_active.c:755). Returns
/// `qfalse` if the client has been kicked for inactivity, `qtrue` otherwise. No oracle —
/// reads the `g_inactivity` cvar + `level.time`, mutates the `gclient_t`, and calls
/// `trap_DropClient`/`trap_SendServerCommand`.
///
/// # Safety
/// `client` must point into the `level.clients[]` array; the `level` global and the
/// `g_inactivity` cvar must be initialised.
pub unsafe fn ClientInactivityTimer(client: *mut gclient_t) -> qboolean {
    if g_inactivity.integer == 0 {
        // give everyone some time, so if the operator sets g_inactivity during
        // gameplay, everyone isn't kicked
        (*client).inactivityTime = (*addr_of!(level)).time + 60 * 1000;
        (*client).inactivityWarning = QFALSE;
    } else if (*client).pers.cmd.forwardmove != 0
        || (*client).pers.cmd.rightmove != 0
        || (*client).pers.cmd.upmove != 0
        || ((*client).pers.cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0
    {
        (*client).inactivityTime = (*addr_of!(level)).time + g_inactivity.integer * 1000;
        (*client).inactivityWarning = QFALSE;
    } else if (*client).pers.localClient == QFALSE {
        // client - level.clients == client index
        let client_num = client.offset_from((*addr_of!(level)).clients) as c_int;
        if (*addr_of!(level)).time > (*client).inactivityTime {
            crate::trap::DropClient(client_num, "Dropped due to inactivity");
            return QFALSE;
        }
        if (*addr_of!(level)).time > (*client).inactivityTime - 10000
            && (*client).inactivityWarning == QFALSE
        {
            (*client).inactivityWarning = QTRUE;
            crate::trap::SendServerCommand(
                client_num,
                "cp \"Ten seconds until inactivity drop!\n\"",
            );
        }
    }
    QTRUE
}

/// `void ClientTimerActions( gentity_t *ent, int msec )` (g_active.c:787). Accumulate `msec`
/// into the client's `timeResidual`; each whole second, bleed off one point of over-max health
/// and one point of over-max armor.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL.
pub unsafe fn ClientTimerActions(ent: *mut gentity_t, msec: c_int) {
    let client: *mut gclient_t;

    client = (*ent).client;
    (*client).timeResidual += msec;

    while (*client).timeResidual >= 1000 {
        (*client).timeResidual -= 1000;

        // count down health when over max
        if (*ent).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*ent).health -= 1;
        }

        // count down armor when over max
        if (*client).ps.stats[STAT_ARMOR as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*client).ps.stats[STAT_ARMOR as usize] -= 1;
        }
    }
}

/// `void G_AddPushVecToUcmd( gentity_t *self, usercmd_t *ucmd )` (g_active.c:1216). Bend the
/// client's move command toward its active push vector (`pushVec`): rebuild the intended move
/// velocity from the ucmd, add the push, renormalize into `ps.speed`, and project back onto the
/// forward/right axes as new `forwardmove`/`rightmove`. Clears `pushVec` once `pushVecTime`
/// expires. No-op when the client has no push (`VectorLengthSquared == 0`).
///
/// # Safety
/// `self` must point to a valid `gentity_t` (its `client` may be NULL — handled) and `ucmd` to a
/// valid `usercmd_t`; the `level` global must be initialised.
pub unsafe fn G_AddPushVecToUcmd(self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut moveDir: vec3_t = [0.0; 3];
    let pushSpeed: f32;

    if (*self_).client.is_null() {
        return;
    }
    pushSpeed = VectorLengthSquared(&(*(*self_).client).pushVec);
    if pushSpeed == 0.0 {
        //not being pushed
        return;
    }

    AngleVectors(
        &(*(*self_).client).ps.viewangles,
        Some(&mut forward),
        Some(&mut right),
        None,
    );
    let tmp_forward = forward;
    VectorScale(
        &tmp_forward,
        (*ucmd).forwardmove as f32 / 127.0f32 * (*(*self_).client).ps.speed,
        &mut moveDir,
    );
    let tmp_moveDir = moveDir;
    VectorMA(
        &tmp_moveDir,
        (*ucmd).rightmove as f32 / 127.0f32 * (*(*self_).client).ps.speed,
        &right,
        &mut moveDir,
    );
    //moveDir is now our intended move velocity

    let tmp_moveDir = moveDir;
    VectorAdd(&tmp_moveDir, &(*(*self_).client).pushVec, &mut moveDir);
    (*(*self_).client).ps.speed = VectorNormalize(&mut moveDir);
    //moveDir is now our intended move velocity plus our push Vector

    let fMove: f32 = 127.0 * DotProduct(&forward, &moveDir);
    let rMove: f32 = 127.0 * DotProduct(&right, &moveDir);
    (*ucmd).forwardmove = (fMove as f64).floor() as i8; //If in the same dir , will be positive
    (*ucmd).rightmove = (rMove as f64).floor() as i8; //If in the same dir , will be positive

    if (*(*self_).client).pushVecTime < (*addr_of!(level)).time {
        VectorClear(&mut (*(*self_).client).pushVec);
    }
}

/// `qboolean G_StandingAnim( int anim )` (g_active.c:1251). Whether `anim` is one of the
/// plain standing idles (NOTE: does not check idles or special (cinematic) stands).
pub fn G_StandingAnim(anim: c_int) -> qboolean {
    //NOTE: does not check idles or special (cinematic) stands
    match anim {
        BOTH_STAND1 | BOTH_STAND2 | BOTH_STAND3 | BOTH_STAND4 => return QTRUE,
        _ => {}
    }
    QFALSE
}

/// `qboolean G_ActionButtonPressed(int buttons)` (g_active.c:1265). Whether any "action"
/// button (attack/use/gesture/force) is held in the `buttons` bitmask.
pub fn G_ActionButtonPressed(buttons: c_int) -> qboolean {
    if buttons & BUTTON_ATTACK != 0 {
        return QTRUE;
    } else if buttons & BUTTON_USE_HOLDABLE != 0 {
        return QTRUE;
    } else if buttons & BUTTON_GESTURE != 0 {
        return QTRUE;
    } else if buttons & BUTTON_USE != 0 {
        return QTRUE;
    } else if buttons & BUTTON_FORCEGRIP != 0 {
        return QTRUE;
    } else if buttons & BUTTON_ALT_ATTACK != 0 {
        return QTRUE;
    } else if buttons & BUTTON_FORCEPOWER != 0 {
        return QTRUE;
    } else if buttons & BUTTON_FORCE_LIGHTNING != 0 {
        return QTRUE;
    } else if buttons & BUTTON_FORCE_DRAIN != 0 {
        return QTRUE;
    }

    QFALSE
}

/// `void ClientIntermissionThink( gclient_t *client )` (g_active.c:814). During the
/// intermission, swap and latch the button actions so that once a player presses
/// attack/use-holdable they are marked ready to exit. No oracle — pure `gclient_t`
/// field mutation.
///
/// # Safety
/// `client` must point to a valid `gclient_t`.
pub unsafe fn ClientIntermissionThink(client: *mut gclient_t) {
    (*client).ps.eFlags &= !EF_TALK;
    (*client).ps.eFlags &= !EF_FIRING;

    // the level will exit when everyone wants to or after timeouts

    // swap and latch button actions
    (*client).oldbuttons = (*client).buttons;
    (*client).buttons = (*client).pers.cmd.buttons;
    if (*client).buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) & ((*client).oldbuttons ^ (*client).buttons) != 0 {
        // this used to be an ^1 but once a player says ready, it should stick
        (*client).readyToExit = 1;
    }
}

/// `void SendPendingPredictableEvents( playerState_t *ps )` (g_active.c:1063). If there
/// are still events pending, create a temporary entity for the event which is sent to
/// everyone except the client who generated the event. No oracle — temp-entity +
/// playerState side-effects.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`; the entity system must be initialised.
pub unsafe fn SendPendingPredictableEvents(ps: *mut playerState_t) {
    let t: *mut gentity_t;
    let event: c_int;
    let seq: c_int;
    let ext_event: c_int;
    let number: c_int;

    // if there are still events pending
    if (*ps).entityEventSequence < (*ps).eventSequence {
        // create a temporary entity for this event which is sent to everyone
        // except the client who generated the event
        seq = (*ps).entityEventSequence & (MAX_PS_EVENTS as c_int - 1);
        event = (*ps).events[seq as usize] | (((*ps).entityEventSequence & 3) << 8);
        // set external event to zero before calling BG_PlayerStateToEntityState
        ext_event = (*ps).externalEvent;
        (*ps).externalEvent = 0;
        // create temporary entity for event
        t = G_TempEntity(&(*ps).origin, event);
        number = (*t).s.number;
        BG_PlayerStateToEntityState(&mut *ps, &mut (*t).s, QTRUE);
        (*t).s.number = number;
        (*t).s.eType = ET_EVENTS + event;
        (*t).s.eFlags |= EF_PLAYER_EVENT;
        (*t).s.otherEntityNum = (*ps).clientNum;
        // send to everyone except the client who generated the event
        (*t).r.svFlags |= SVF_NOTSINGLECLIENT;
        (*t).r.singleClient = (*ps).clientNum;
        // set back external event
        (*ps).externalEvent = ext_event;
    }
}

/// `void G_SetTauntAnim( gentity_t *ent, int taunt )` (g_active.c:1667). Play the
/// requested taunt/bow/meditate/flourish/gloat animation, toggling saber sounds and
/// holstered state per saber style, and (for non-meditate/bow taunts) fire the taunt
/// event. No oracle — entity/saber-state + animation side-effects.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL; the `level` and
/// `g_gametype` globals must be initialised.
pub unsafe fn G_SetTauntAnim(ent: *mut gentity_t, taunt: c_int) {
    let client = (*ent).client;
    if (*client).pers.cmd.upmove != 0
        || (*client).pers.cmd.forwardmove != 0
        || (*client).pers.cmd.rightmove != 0
    {
        // hack, don't do while moving
        return;
    }
    if taunt != TAUNT_TAUNT {
        // normal taunt always allowed
        if (*addr_of!(g_gametype)).integer != GT_DUEL
            && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
        {
            // no taunts unless in Duel
            return;
        }
    }
    if (*client).ps.torsoTimer < 1
        && (*client).ps.forceHandExtend == HANDEXTEND_NONE
        && (*client).ps.legsTimer < 1
        && (*client).ps.weaponTime < 1
        && (*client).ps.saberLockTime < (*addr_of!(level)).time
    {
        let mut anim: c_int = -1;
        match taunt {
            TAUNT_TAUNT => {
                if (*client).ps.weapon != WP_SABER {
                    anim = BOTH_ENGAGETAUNT;
                } else if (*client).saber[0].tauntAnim != -1 {
                    anim = (*client).saber[0].tauntAnim;
                } else if (*client).saber[1].model[0] != 0 && (*client).saber[1].tauntAnim != -1 {
                    anim = (*client).saber[1].tauntAnim;
                } else {
                    match (*client).ps.fd.saberAnimLevel {
                        SS_FAST | SS_TAVION => {
                            if (*client).ps.saberHolstered == 1
                                && (*client).saber[1].model[0] != 0
                            {
                                // turn off second saber
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOff);
                            } else if (*client).ps.saberHolstered == 0 {
                                // turn off first
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOff);
                            }
                            (*client).ps.saberHolstered = 2;
                            anim = BOTH_GESTURE1;
                        }
                        SS_MEDIUM | SS_STRONG | SS_DESANN => {
                            anim = BOTH_ENGAGETAUNT;
                        }
                        SS_DUAL => {
                            if (*client).ps.saberHolstered == 1
                                && (*client).saber[1].model[0] != 0
                            {
                                // turn on second saber
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOn);
                            } else if (*client).ps.saberHolstered == 2 {
                                // turn on first
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                            }
                            (*client).ps.saberHolstered = 0;
                            anim = BOTH_DUAL_TAUNT;
                        }
                        SS_STAFF => {
                            if (*client).ps.saberHolstered > 0 {
                                // turn on all blades
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                            }
                            (*client).ps.saberHolstered = 0;
                            anim = BOTH_STAFF_TAUNT;
                        }
                        _ => {}
                    }
                }
            }
            TAUNT_BOW => {
                if (*client).saber[0].bowAnim != -1 {
                    anim = (*client).saber[0].bowAnim;
                } else if (*client).saber[1].model[0] != 0 && (*client).saber[1].bowAnim != -1 {
                    anim = (*client).saber[1].bowAnim;
                } else {
                    anim = BOTH_BOW;
                }
                if (*client).ps.saberHolstered == 1 && (*client).saber[1].model[0] != 0 {
                    // turn off second saber
                    G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOff);
                } else if (*client).ps.saberHolstered == 0 {
                    // turn off first
                    G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOff);
                }
                (*client).ps.saberHolstered = 2;
            }
            TAUNT_MEDITATE => {
                if (*client).saber[0].meditateAnim != -1 {
                    anim = (*client).saber[0].meditateAnim;
                } else if (*client).saber[1].model[0] != 0 && (*client).saber[1].meditateAnim != -1
                {
                    anim = (*client).saber[1].meditateAnim;
                } else {
                    anim = BOTH_MEDITATE;
                }
                if (*client).ps.saberHolstered == 1 && (*client).saber[1].model[0] != 0 {
                    // turn off second saber
                    G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOff);
                } else if (*client).ps.saberHolstered == 0 {
                    // turn off first
                    G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOff);
                }
                (*client).ps.saberHolstered = 2;
            }
            TAUNT_FLOURISH => {
                if (*client).ps.weapon == WP_SABER {
                    if (*client).ps.saberHolstered == 1 && (*client).saber[1].model[0] != 0 {
                        // turn on second saber
                        G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOn);
                    } else if (*client).ps.saberHolstered == 2 {
                        // turn on first
                        G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                    }
                    (*client).ps.saberHolstered = 0;
                    if (*client).saber[0].flourishAnim != -1 {
                        anim = (*client).saber[0].flourishAnim;
                    } else if (*client).saber[1].model[0] != 0
                        && (*client).saber[1].flourishAnim != -1
                    {
                        anim = (*client).saber[1].flourishAnim;
                    } else {
                        match (*client).ps.fd.saberAnimLevel {
                            SS_FAST | SS_TAVION => {
                                anim = BOTH_SHOWOFF_FAST;
                            }
                            SS_MEDIUM => {
                                anim = BOTH_SHOWOFF_MEDIUM;
                            }
                            SS_STRONG | SS_DESANN => {
                                anim = BOTH_SHOWOFF_STRONG;
                            }
                            SS_DUAL => {
                                anim = BOTH_SHOWOFF_DUAL;
                            }
                            SS_STAFF => {
                                anim = BOTH_SHOWOFF_STAFF;
                            }
                            _ => {}
                        }
                    }
                }
            }
            TAUNT_GLOAT => {
                if (*client).saber[0].gloatAnim != -1 {
                    anim = (*client).saber[0].gloatAnim;
                } else if (*client).saber[1].model[0] != 0 && (*client).saber[1].gloatAnim != -1 {
                    anim = (*client).saber[1].gloatAnim;
                } else {
                    match (*client).ps.fd.saberAnimLevel {
                        SS_FAST | SS_TAVION => {
                            anim = BOTH_VICTORY_FAST;
                        }
                        SS_MEDIUM => {
                            anim = BOTH_VICTORY_MEDIUM;
                        }
                        SS_STRONG | SS_DESANN => {
                            if (*client).ps.saberHolstered != 0 {
                                // turn on first
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                            }
                            (*client).ps.saberHolstered = 0;
                            anim = BOTH_VICTORY_STRONG;
                        }
                        SS_DUAL => {
                            if (*client).ps.saberHolstered == 1 && (*client).saber[1].model[0] != 0
                            {
                                // turn on second saber
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[1].soundOn);
                            } else if (*client).ps.saberHolstered == 2 {
                                // turn on first
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                            }
                            (*client).ps.saberHolstered = 0;
                            anim = BOTH_VICTORY_DUAL;
                        }
                        SS_STAFF => {
                            if (*client).ps.saberHolstered != 0 {
                                // turn on first
                                G_Sound(ent, CHAN_WEAPON, (*client).saber[0].soundOn);
                            }
                            (*client).ps.saberHolstered = 0;
                            anim = BOTH_VICTORY_STAFF;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if anim != -1 {
            if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                (*client).ps.forceHandExtend = HANDEXTEND_TAUNT;
                (*client).ps.forceDodgeAnim = anim;
                (*client).ps.forceHandExtendTime =
                    (*addr_of!(level)).time + BG_AnimLength((*ent).localAnimIndex, anim as animNumber_t);
            }
            if taunt != TAUNT_MEDITATE && taunt != TAUNT_BOW {
                // no sound for meditate or bow
                G_AddEvent(ent, EV_TAUNT, taunt);
            }
        }
    }
}

/// `void G_CheckClientTimeouts ( gentity_t *ent )` (g_active.c:3529). The only timeout
/// supported right now is the timeout to spectator mode: if `g_timeouttospec` is set and
/// the client has not sent a command within that window, force it to spectator. No oracle —
/// reads the `level`/`g_timeouttospec` globals and calls [`SetTeam`].
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL; the `level` and
/// `g_timeouttospec` globals must be initialised.
pub unsafe fn G_CheckClientTimeouts(ent: *mut gentity_t) {
    // Only timeout supported right now is the timeout to spectator mode
    if (*addr_of!(g_timeouttospec)).integer == 0 {
        return;
    }

    // Already a spectator, no need to boot them to spectator
    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        return;
    }

    // See how long its been since a command was received by the client and if its
    // longer than the timeout to spectator then force this client into spectator mode
    if (*addr_of!(level)).time - (*(*ent).client).pers.cmd.serverTime
        > (*addr_of!(g_timeouttospec)).integer * 1000
    {
        SetTeam(ent, c"spectator".as_ptr() as *mut _);
    }
}

/// `void P_WorldEffects( gentity_t *ent )` (g_active.c:133). Check for lava / slime contents
/// and drowning, applying the appropriate damage and gurp/pain sound events. No oracle —
/// entity/client side-effects + `rand()` + `G_Sound`/`G_AddEvent`/`G_Damage`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL; the `level` global must
/// be initialised.
pub unsafe fn P_WorldEffects(ent: *mut gentity_t) {
    let envirosuit: qboolean;
    let waterlevel: c_int;

    if (*(*ent).client).noclip != 0 {
        (*(*ent).client).airOutTime = (*addr_of!(level)).time + 12000; // don't need air
        return;
    }

    waterlevel = (*ent).waterlevel;

    envirosuit =
        ((*(*ent).client).ps.powerups[PW_BATTLESUIT as usize] > (*addr_of!(level)).time) as qboolean;

    //
    // check for drowning
    //
    if waterlevel == 3 {
        // envirosuit give air
        if envirosuit != QFALSE {
            (*(*ent).client).airOutTime = (*addr_of!(level)).time + 10000;
        }

        // if out of air, start drowning
        if (*(*ent).client).airOutTime < (*addr_of!(level)).time {
            // drown!
            (*(*ent).client).airOutTime += 1000;
            if (*ent).health > 0 {
                // take more damage the longer underwater
                (*ent).damage += 2;
                if (*ent).damage > 15 {
                    (*ent).damage = 15;
                }

                // play a gurp sound instead of a normal pain sound
                if (*ent).health <= (*ent).damage {
                    G_Sound(ent, CHAN_VOICE, G_SoundIndex(/*"*drown.wav"*/ "sound/player/gurp1.wav"));
                } else if rand() & 1 != 0 {
                    G_Sound(ent, CHAN_VOICE, G_SoundIndex("sound/player/gurp1.wav"));
                } else {
                    G_Sound(ent, CHAN_VOICE, G_SoundIndex("sound/player/gurp2.wav"));
                }

                // don't play a normal pain sound
                (*ent).pain_debounce_time = (*addr_of!(level)).time + 200;

                G_Damage(
                    ent,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*ent).damage,
                    DAMAGE_NO_ARMOR,
                    MOD_WATER,
                );
            }
        }
    } else {
        (*(*ent).client).airOutTime = (*addr_of!(level)).time + 12000;
        (*ent).damage = 2;
    }

    //
    // check for sizzle damage (move to pmove?)
    //
    if waterlevel != 0 && ((*ent).watertype & (CONTENTS_LAVA | CONTENTS_SLIME)) != 0 {
        if (*ent).health > 0 && (*ent).pain_debounce_time <= (*addr_of!(level)).time {
            if envirosuit != QFALSE {
                G_AddEvent(ent, EV_POWERUP_BATTLESUIT, 0);
            } else {
                if (*ent).watertype & CONTENTS_LAVA != 0 {
                    G_Damage(
                        ent,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        30 * waterlevel,
                        0,
                        MOD_LAVA,
                    );
                }

                if (*ent).watertype & CONTENTS_SLIME != 0 {
                    G_Damage(
                        ent,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        10 * waterlevel,
                        0,
                        MOD_SLIME,
                    );
                }
            }
        }
    }
}

/// `void ClientImpacts( gentity_t *ent, pmove_t *pm )` (g_active.c:482). Dispatch the touch
/// callback of every entity the pmove flagged this frame (de-duplicating the touch list), and
/// for bots also fire `ent`'s own touch on each. No oracle — entity touch fn-ptr dispatch.
///
/// # Safety
/// `ent` and `pm` must be valid; the `g_entities` array must be initialised.
pub unsafe fn ClientImpacts(ent: *mut gentity_t, pm: *mut pmove_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut trace: trace_t = core::mem::zeroed();

    // memset( &trace, 0, sizeof( trace ) ); — handled by zeroed() above
    i = 0;
    while i < (*pm).numtouch {
        j = 0;
        while j < i {
            if (*pm).touchents[j as usize] == (*pm).touchents[i as usize] {
                break;
            }
            j += 1;
        }
        if j != i {
            i += 1;
            continue; // duplicated
        }
        let other: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*pm).touchents[i as usize] as usize);

        if ((*ent).r.svFlags & SVF_BOT) != 0 && (*ent).touch.is_some() {
            ((*ent).touch.unwrap())(ent, other, &mut trace);
        }

        if (*other).touch.is_none() {
            i += 1;
            continue;
        }

        ((*other).touch.unwrap())(other, ent, &mut trace);
        i += 1;
    }
}

/// `void G_TouchTriggers( gentity_t *ent )` (g_active.c:520). Find all trigger entities that
/// `ent`'s current position touches and fire their touch callbacks. Spectators only interact
/// with teleporters. No oracle — `trap_EntitiesInBox`/`trap_EntityContact` + touch fn-ptr
/// dispatch.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the `g_entities` array and `level` global must be
/// initialised.
pub unsafe fn G_TouchTriggers(ent: *mut gentity_t) {
    let mut i: c_int;
    let num: c_int;
    let mut touch: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut trace: trace_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    // static vec3_t range = { 40, 40, 52 };
    static RANGE: vec3_t = [40.0, 40.0, 52.0];

    if (*ent).client.is_null() {
        return;
    }

    // dead clients don't activate triggers!
    if (*(*ent).client).ps.stats[STAT_HEALTH as usize] <= 0 {
        return;
    }

    let origin = (*(*ent).client).ps.origin;
    VectorSubtract(&origin, &RANGE, &mut mins);
    VectorAdd(&origin, &RANGE, &mut maxs);

    num = crate::trap::EntitiesInBox(&mins, &maxs, &mut touch);

    // can't use ent->r.absmin, because that has a one unit pad
    let origin = (*(*ent).client).ps.origin;
    let r_mins = (*ent).r.mins;
    let r_maxs = (*ent).r.maxs;
    VectorAdd(&origin, &r_mins, &mut mins);
    VectorAdd(&origin, &r_maxs, &mut maxs);

    i = 0;
    while i < num {
        let hit: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(touch[i as usize] as usize);

        if (*hit).touch.is_none() && (*ent).touch.is_none() {
            i += 1;
            continue;
        }
        if ((*hit).r.contents & CONTENTS_TRIGGER) == 0 {
            i += 1;
            continue;
        }

        // ignore most entities if a spectator
        if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
            // hit->touch != Touch_DoorTrigger — fn-pointer identity via fn_addr_eq
            let hit_is_doortrigger = match (*hit).touch {
                Some(t) => core::ptr::fn_addr_eq(
                    t,
                    Touch_DoorTrigger
                        as unsafe extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t),
                ),
                None => false,
            };
            if (*hit).s.eType != ET_TELEPORT_TRIGGER
                // this is ugly but adding a new ET_? type will
                // most likely cause network incompatibilities
                && !hit_is_doortrigger
            {
                i += 1;
                continue;
            }
        }

        // use seperate code for determining if an item is picked up
        // so you don't have to actually contact its bounding box
        if (*hit).s.eType == ET_ITEM {
            if BG_PlayerTouchesItem(&(*(*ent).client).ps, &(*hit).s, (*addr_of!(level)).time)
                == QFALSE
            {
                i += 1;
                continue;
            }
        } else if crate::trap::EntityContact(&mins, &maxs, hit) == QFALSE {
            i += 1;
            continue;
        }

        trace = core::mem::zeroed();

        if (*hit).touch.is_some() {
            ((*hit).touch.unwrap())(hit, ent, &mut trace);
        }

        if ((*ent).r.svFlags & SVF_BOT) != 0 && (*ent).touch.is_some() {
            ((*ent).touch.unwrap())(ent, hit, &mut trace);
        }
        i += 1;
    }

    // if we didn't touch a jump pad this pmove frame
    if (*(*ent).client).ps.jumppad_frame != (*(*ent).client).ps.pmove_framecount {
        (*(*ent).client).ps.jumppad_frame = 0;
        (*(*ent).client).ps.jumppad_ent = 0;
    }
}

/// `void G_MoverTouchPushTriggers( gentity_t *ent, vec3_t oldOrg )` (g_active.c:605). Sweep the
/// path the mover traversed this frame and fire the touch callbacks of any push-triggers it
/// crossed. No oracle — `trap_EntitiesInBox`/`trap_EntityContact` + touch fn-ptr dispatch.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `old_org` must be valid; the `g_entities` array must
/// be initialised.
pub unsafe fn G_MoverTouchPushTriggers(ent: *mut gentity_t, old_org: &vec3_t) {
    let mut i: c_int;
    let mut num: c_int;
    let mut step: f32;
    let mut stepSize: f32;
    let dist: f32;
    let mut touch: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut trace: trace_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut size: vec3_t = [0.0; 3];
    let mut checkSpot: vec3_t = [0.0; 3];
    const RANGE: vec3_t = [40.0, 40.0, 52.0];

    // non-moving movers don't hit triggers!
    if VectorLengthSquared(&(*ent).s.pos.trDelta) == 0.0 {
        return;
    }

    let r_mins = (*ent).r.mins;
    let r_maxs = (*ent).r.maxs;
    VectorSubtract(&r_mins, &r_maxs, &mut size);
    stepSize = VectorLength(&size);
    if stepSize < 1.0 {
        stepSize = 1.0;
    }

    let currentOrigin = (*ent).r.currentOrigin;
    VectorSubtract(&currentOrigin, old_org, &mut dir);
    dist = VectorNormalize(&mut dir);
    step = 0.0;
    while step <= dist {
        let currentOrigin = (*ent).r.currentOrigin;
        VectorMA(&currentOrigin, step, &dir, &mut checkSpot);
        VectorSubtract(&checkSpot, &RANGE, &mut mins);
        VectorAdd(&checkSpot, &RANGE, &mut maxs);

        num = crate::trap::EntitiesInBox(&mins, &maxs, &mut touch);

        // can't use ent->r.absmin, because that has a one unit pad
        let r_mins = (*ent).r.mins;
        let r_maxs = (*ent).r.maxs;
        let checkSpot_snap = checkSpot;
        VectorAdd(&checkSpot_snap, &r_mins, &mut mins);
        VectorAdd(&checkSpot_snap, &r_maxs, &mut maxs);

        i = 0;
        while i < num {
            let hit: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(touch[i as usize] as usize);

            if (*hit).s.eType != ET_PUSH_TRIGGER {
                i += 1;
                continue;
            }

            if (*hit).touch.is_none() {
                i += 1;
                continue;
            }

            if ((*hit).r.contents & CONTENTS_TRIGGER) == 0 {
                i += 1;
                continue;
            }

            if crate::trap::EntityContact(&mins, &maxs, hit) == QFALSE {
                i += 1;
                continue;
            }

            trace = core::mem::zeroed();

            if (*hit).touch.is_some() {
                ((*hit).touch.unwrap())(hit, ent, &mut trace);
            }
            i += 1;
        }
        step += stepSize;
    }
}

/// `extern "C"` shim mirroring C's `trap_Trace` (g_syscalls.c:142) so it can be assigned to the
/// `pmove_t::trace` engine-callback fn-pointer (which has the raw `vec3_t`-pointer signature).
/// The Rust [`crate::trap::Trace`] wrapper has a different, idiomatic shape; this is the verbatim
/// callback form Pmove invokes through `pm->trace`.
unsafe extern "C" fn trap_Trace_pm(
    results: *mut trace_t,
    start: *const vec_t,
    mins: *const vec_t,
    maxs: *const vec_t,
    end: *const vec_t,
    pass_entity_num: c_int,
    contentmask: c_int,
) {
    syscall!(
        crate::ffi::GameImport::G_TRACE,
        results,
        start,
        mins,
        maxs,
        end,
        pass_entity_num,
        contentmask,
        0,
        10
    );
}

/// `extern "C"` shim mirroring C's `trap_PointContents` (g_syscalls.c:151) for the
/// `pmove_t::pointcontents` engine-callback fn-pointer.
unsafe extern "C" fn trap_PointContents_pm(point: *const vec_t, pass_entity_num: c_int) -> c_int {
    syscall!(
        crate::ffi::GameImport::G_POINT_CONTENTS,
        point,
        pass_entity_num
    ) as c_int
}

/// `void SpectatorThink( gentity_t *ent, usercmd_t *ucmd )` (g_active.c:682). Run a spectator
/// pmove (unless following another player), touch triggers, and cycle/leave follow mode on the
/// attack/jump buttons. No oracle — `Pmove` + `trap_Trace`/`trap_PointContents`/`trap_UnlinkEntity`
/// + entity-state side effects.
///
/// # Safety
/// `ent` and `ucmd` must be valid; the `level`/`g_entities`/`g_noSpecMove` globals must be
/// initialised.
pub unsafe fn SpectatorThink(ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut pm: pmove_t;
    let client: *mut gclient_t;

    client = (*ent).client;

    if (*client).sess.spectatorState != SPECTATOR_FOLLOW {
        (*client).ps.pm_type = PM_SPECTATOR;
        (*client).ps.speed = 400.0; // faster than normal
        (*client).ps.basespeed = 400;

        //hmm, shouldn't have an anim if you're a spectator, make sure
        //it gets cleared.
        (*client).ps.legsAnim = 0;
        (*client).ps.legsTimer = 0;
        (*client).ps.torsoAnim = 0;
        (*client).ps.torsoTimer = 0;

        // set up for pmove
        pm = core::mem::zeroed();
        pm.ps = &mut (*client).ps;
        pm.cmd = *ucmd;
        pm.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY; // spectators can fly through bodies
        pm.trace = Some(trap_Trace_pm);
        pm.pointcontents = Some(trap_PointContents_pm);

        pm.noSpecMove = g_noSpecMove.integer;

        pm.animations = core::ptr::null_mut();
        pm.nonHumanoid = QFALSE;

        //Set up bg entity data
        pm.baseEnt = core::ptr::addr_of_mut!(g_entities).cast::<bgEntity_t>();
        pm.entSize = core::mem::size_of::<gentity_t>() as c_int;

        // perform a pmove
        Pmove(&mut pm);
        // save results of pmove
        let psorigin = (*client).ps.origin;
        crate::codemp::game::q_math::VectorCopy(&psorigin, &mut (*ent).s.origin);

        if (*(*ent).client).tempSpectate < (*addr_of!(level)).time {
            G_TouchTriggers(ent);
        }
        crate::trap::UnlinkEntity(ent);
    }

    (*client).oldbuttons = (*client).buttons;
    (*client).buttons = (*ucmd).buttons;

    if (*client).tempSpectate < (*addr_of!(level)).time {
        // attack button cycles through spectators
        if ((*client).buttons & BUTTON_ATTACK) != 0 && ((*client).oldbuttons & BUTTON_ATTACK) == 0 {
            Cmd_FollowCycle_f(ent, 1);
        }

        if (*client).sess.spectatorState == SPECTATOR_FOLLOW && (*ucmd).upmove > 0 {
            //jump now removes you from follow mode
            StopFollowing(ent);
        }
    }
}

/// `void G_VehicleAttachDroidUnit( gentity_t *vehEnt )` (g_active.c:830). Snaps the
/// vehicle's attached droid (astromech) unit onto the vehicle's `*droid` bolt tag: reads
/// the bolt matrix off the vehicle's ghoul2 instance, pulls the origin and forward
/// (`NEGATIVE_Y`) vectors out of it, faces the droid down `fwd`, syncs the client/NPC
/// player-state, relinks it, and (for NPCs) forces the `BOTH_STAND2` anim. No oracle —
/// reads the `level` global and calls ghoul2/link traps + entity-state mutators.
///
/// # Safety
/// `vehEnt` may be null (guarded). When non-null its `m_pVehicle`/`m_pDroidUnit` and the
/// droid's `client`/`NPC` pointers must be valid; the `level` global must be initialised.
pub unsafe fn G_VehicleAttachDroidUnit(vehEnt: *mut gentity_t) {
    if !vehEnt.is_null()
        && !(*vehEnt).m_pVehicle.is_null()
        && !(*(*vehEnt).m_pVehicle).m_pDroidUnit.is_null()
    {
        let droidEnt: *mut gentity_t = (*(*vehEnt).m_pVehicle).m_pDroidUnit as *mut gentity_t;
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut fwd: vec3_t = [0.0; 3];

        crate::trap::G2API_GetBoltMatrix(
            (*vehEnt).ghoul2,
            0,
            (*(*vehEnt).m_pVehicle).m_iDroidUnitTag,
            &mut boltMatrix,
            &(*vehEnt).r.currentAngles,
            &(*vehEnt).r.currentOrigin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*vehEnt).modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut (*droidEnt).r.currentOrigin);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut fwd);
        vectoangles(&fwd, &mut (*droidEnt).r.currentAngles);

        if !(*droidEnt).client.is_null() {
            VectorCopy(
                &(*droidEnt).r.currentAngles,
                &mut (*(*droidEnt).client).ps.viewangles,
            );
            VectorCopy(
                &(*droidEnt).r.currentOrigin,
                &mut (*(*droidEnt).client).ps.origin,
            );
        }

        G_SetOrigin(droidEnt, &(*droidEnt).r.currentOrigin);
        crate::trap::LinkEntity(droidEnt);

        if !(*droidEnt).NPC.is_null() {
            NPC_SetAnim(
                droidEnt,
                SETANIM_BOTH,
                BOTH_STAND2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
    }
}

//called gameside only from pmove code (convenience)
/// `void G_CheapWeaponFire(int entNum, int ev)` (g_active.c:861). Fire a client's
/// weapon directly from pmove (used by the speeder/melee predicted-fire path). No oracle
/// — reads the `g_entities`/`level` globals + vehicle structs and calls `FireWeapon`.
///
/// # Safety
/// `entNum` must index `g_entities[]`; the `g_entities`/`level` globals must be initialised.
pub unsafe fn G_CheapWeaponFire(entNum: c_int, ev: c_int) {
    let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entNum as usize);

    if (*ent).inuse == QFALSE || (*ent).client.is_null() {
        return;
    }

    match ev {
        EV_FIRE_WEAPON => {
            if !(*ent).m_pVehicle.is_null()
                && (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                && !(*ent).client.is_null()
                && (*(*ent).client).ps.m_iVehicleNum != 0
            {
                //a speeder with a pilot
                let rider: *mut gentity_t =
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(((*(*ent).client).ps.m_iVehicleNum - 1) as usize);
                if (*rider).inuse != QFALSE && !(*rider).client.is_null() {
                    //pilot is valid...
                    if (*(*rider).client).ps.weapon != WP_MELEE
                        && ((*(*rider).client).ps.weapon != WP_SABER
                            || (*(*rider).client).ps.saberHolstered == 0)
                    {
                        //can only attack on speeder when using melee or when saber is holstered
                        return;
                    }
                }
            }

            FireWeapon(ent, QFALSE);
            (*(*ent).client).dangerTime = (*addr_of!(level)).time;
            (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
            (*(*ent).client).invulnerableTimer = 0;
        }
        EV_ALT_FIRE => {
            FireWeapon(ent, QTRUE);
            (*(*ent).client).dangerTime = (*addr_of!(level)).time;
            (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
            (*(*ent).client).invulnerableTimer = 0;
        }
        _ => {}
    }
}

// G_UpdateClientBroadcasts local #defines (g_active.c:1101-1105). Only the MAX_SIGHT_* pair is
// actually used (by G_UpdateForceSightBroadcasts); the MAX_JEDIMASTER_* pair belongs to
// G_UpdateJediMasterBroadcasts whose body is #if 0/commented out upstream, so it is unused here.
const MAX_JEDIMASTER_DISTANCE: c_int = 2500;
#[allow(dead_code)] // only referenced from the commented-out G_UpdateJediMasterBroadcasts body
const MAX_JEDIMASTER_FOV: c_int = 100;

const MAX_SIGHT_DISTANCE: c_int = 1500;
const MAX_SIGHT_FOV: c_int = 100;

/// `static void G_UpdateForceSightBroadcasts ( gentity_t *self )` (g_active.c:1107). Turn on the
/// broadcast bit for any client currently using force sight (`FP_SEE`) that is within
/// `MAX_SIGHT_DISTANCE` and `MAX_SIGHT_FOV` of `self`. No oracle — pure entity-state: iterates
/// `level.numConnectedClients`/`level.sortedClients`, reads `g_entities`, writes the
/// `r.broadcastClients` bitfield, and calls the ported `InFieldOfVision`.
///
/// # Safety
/// `self` must be valid; the `level`/`g_entities` globals must be initialised.
unsafe fn G_UpdateForceSightBroadcasts(self_: *mut gentity_t) {
    // int i;

    // Any clients with force sight on should see this client
    let mut i: c_int = 0;
    while i < (*addr_of!(level)).numConnectedClients {
        let ent: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*addr_of!(level)).sortedClients[i as usize] as usize);
        let mut angles: vec3_t = [0.0; 3];

        if ent == self_ {
            i += 1;
            continue;
        }

        // Not using force sight so we shouldnt broadcast to this one
        if (*(*ent).client).ps.fd.forcePowersActive & (1 << FP_SEE) == 0 {
            i += 1;
            continue;
        }

        VectorSubtract(
            &(*(*self_).client).ps.origin,
            &(*(*ent).client).ps.origin,
            &mut angles,
        );
        let dist: f32 = VectorLengthSquared(&angles);
        let angles_in = angles;
        vectoangles(&angles_in, &mut angles);

        // Too far away then just forget it
        if dist > (MAX_SIGHT_DISTANCE * MAX_SIGHT_DISTANCE) as f32 {
            i += 1;
            continue;
        }

        // If not within the field of view then forget it
        if InFieldOfVision(&(*(*ent).client).ps.viewangles, MAX_SIGHT_FOV as f32, &mut angles)
            == QFALSE as c_int
        {
            break;
        }

        // Turn on the broadcast bit for the master and since there is only one
        // master we are done
        (*self_).r.broadcastClients[((*ent).s.clientNum / 32) as usize] |=
            1 << ((*ent).s.clientNum % 32);

        break;
    }
}

/// `static void G_UpdateJediMasterBroadcasts ( gentity_t *self )` (g_active.c:1149). Broadcast
/// the Jedi Master to all clients within range and field of view. No oracle — pure entity-state:
/// iterates `level.numConnectedClients`/`level.sortedClients`, reads `g_entities`, writes the
/// `r.broadcastClients` bitfield, and calls the ported `InFieldOfVision`. Reads the `g_gametype`
/// cvar global.
///
/// # Safety
/// `self` must be valid; the `level`/`g_entities`/`g_gametype` globals must be initialised.
unsafe fn G_UpdateJediMasterBroadcasts(self_: *mut gentity_t) {
    // int i;

    // Not jedi master mode then nothing to do
    if g_gametype.integer != GT_JEDIMASTER {
        return;
    }

    // This client isnt the jedi master so it shouldnt broadcast
    if (*(*self_).client).ps.isJediMaster == QFALSE as c_int {
        return;
    }

    // Broadcast ourself to all clients within range
    let mut i: c_int = 0;
    while i < (*addr_of!(level)).numConnectedClients {
        let ent: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*addr_of!(level)).sortedClients[i as usize] as usize);
        let mut angles: vec3_t = [0.0; 3];

        if ent == self_ {
            i += 1;
            continue;
        }

        VectorSubtract(
            &(*(*self_).client).ps.origin,
            &(*(*ent).client).ps.origin,
            &mut angles,
        );
        let dist: f32 = VectorLengthSquared(&angles);
        let angles_in = angles;
        vectoangles(&angles_in, &mut angles);

        // Too far away then just forget it
        if dist > (MAX_JEDIMASTER_DISTANCE * MAX_JEDIMASTER_DISTANCE) as f32 {
            i += 1;
            continue;
        }

        // If not within the field of view then forget it
        if InFieldOfVision(
            &(*(*ent).client).ps.viewangles,
            MAX_JEDIMASTER_FOV as f32,
            &mut angles,
        ) == QFALSE as c_int
        {
            i += 1;
            continue;
        }

        // Turn on the broadcast bit for the master and since there is only one
        // master we are done
        (*self_).r.broadcastClients[((*ent).s.clientNum / 32) as usize] |=
            1 << ((*ent).s.clientNum % 32);

        i += 1;
    }
}

/// `void G_UpdateClientBroadcasts ( gentity_t *self )` (g_active.c:1204). Determines whether this
/// client should be broadcast to any other clients: clears the `r.broadcastClients` bitfield then
/// lets the Jedi-Master and force-sight helpers set the relevant bits. No oracle — entity-state.
///
/// # Safety
/// `self` must be valid; the `level`/`g_entities`/`g_gametype` globals must be initialised.
pub unsafe fn G_UpdateClientBroadcasts(self_: *mut gentity_t) {
    // Clear all the broadcast bits for this client
    (*self_).r.broadcastClients = [0; 2]; // memset( ..., 0, sizeof( broadcastClients ) )

    // The jedi master is broadcast to everyone in range
    G_UpdateJediMasterBroadcasts(self_);

    // Anyone with force sight on should see this client
    G_UpdateForceSightBroadcasts(self_);
}

/// `void G_CheckClientIdle( gentity_t *ent, usercmd_t *ucmd )` (g_active.c:1307). Idle-animation
/// state machine: break a player out of a standing idle when they act, otherwise (after 5s of
/// inactivity) start an idle animation. No oracle — entity/client animation state machine reading
/// the `level` global + calling `G_SetAnim`/`Q_irand`.
///
/// # Safety
/// `ent` and `ucmd` must be valid; the `level` global must be initialised.
pub unsafe fn G_CheckClientIdle(ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut viewChange: vec3_t = [0.0; 3];
    let actionPressed: qboolean;
    let mut buttons: c_int;

    if ent.is_null()
        || (*ent).client.is_null()
        || (*ent).health <= 0
        || (*(*ent).client).ps.stats[STAT_HEALTH as usize] <= 0
        || (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR
        || ((*(*ent).client).ps.pm_flags & PMF_FOLLOW) != 0
    {
        return;
    }

    buttons = (*ucmd).buttons;

    if ((*ent).r.svFlags & SVF_BOT) != 0 {
        //they press use all the time..
        buttons &= !BUTTON_USE;
    }
    actionPressed = G_ActionButtonPressed(buttons);

    let viewangles = (*(*ent).client).ps.viewangles;
    let idleViewAngles = (*(*ent).client).idleViewAngles;
    VectorSubtract(&viewangles, &idleViewAngles, &mut viewChange);
    if VectorCompare(&vec3_origin, &(*(*ent).client).ps.velocity) == 0
        || actionPressed != QFALSE
        || (*ucmd).forwardmove != 0
        || (*ucmd).rightmove != 0
        || (*ucmd).upmove != 0
        || G_StandingAnim((*(*ent).client).ps.legsAnim) == QFALSE
        || ((*ent).health + (*(*ent).client).ps.stats[STAT_ARMOR as usize])
            != (*(*ent).client).idleHealth
        || VectorLength(&viewChange) > 10.0
        || (*(*ent).client).ps.legsTimer > 0
        || (*(*ent).client).ps.torsoTimer > 0
        || (*(*ent).client).ps.weaponTime > 0
        || (*(*ent).client).ps.weaponstate == WEAPON_CHARGING
        || (*(*ent).client).ps.weaponstate == WEAPON_CHARGING_ALT
        || (*(*ent).client).ps.zoomMode != 0
        || ((*(*ent).client).ps.weaponstate != WEAPON_READY
            && (*(*ent).client).ps.weapon != WP_SABER)
        || (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
        || (*(*ent).client).ps.saberBlocked != BLOCKED_NONE
        || (*(*ent).client).ps.saberBlocking >= (*addr_of!(level)).time
        || (*(*ent).client).ps.weapon == WP_MELEE
        || ((*(*ent).client).ps.weapon != (*(*ent).client).pers.cmd.weapon as c_int
            && (*ent).s.eType != ET_NPC)
    {
        //FIXME: also check for turning?
        let mut brokeOut: qboolean = QFALSE;

        if VectorCompare(&vec3_origin, &(*(*ent).client).ps.velocity) == 0
            || actionPressed != QFALSE
            || (*ucmd).forwardmove != 0
            || (*ucmd).rightmove != 0
            || (*ucmd).upmove != 0
            || ((*ent).health + (*(*ent).client).ps.stats[STAT_ARMOR as usize])
                != (*(*ent).client).idleHealth
            || (*(*ent).client).ps.zoomMode != 0
            || ((*(*ent).client).ps.weaponstate != WEAPON_READY
                && (*(*ent).client).ps.weapon != WP_SABER)
            || ((*(*ent).client).ps.weaponTime > 0 && (*(*ent).client).ps.weapon == WP_SABER)
            || (*(*ent).client).ps.weaponstate == WEAPON_CHARGING
            || (*(*ent).client).ps.weaponstate == WEAPON_CHARGING_ALT
            || (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
            || (*(*ent).client).ps.saberBlocked != BLOCKED_NONE
            || (*(*ent).client).ps.saberBlocking >= (*addr_of!(level)).time
            || (*(*ent).client).ps.weapon == WP_MELEE
            || ((*(*ent).client).ps.weapon != (*(*ent).client).pers.cmd.weapon as c_int
                && (*ent).s.eType != ET_NPC)
        {
            //if in an idle, break out
            match (*(*ent).client).ps.legsAnim {
                BOTH_STAND1IDLE1 | BOTH_STAND2IDLE1 | BOTH_STAND2IDLE2 | BOTH_STAND3IDLE1
                | BOTH_STAND5IDLE1 => {
                    (*(*ent).client).ps.legsTimer = 0;
                    brokeOut = QTRUE;
                }
                _ => {}
            }
            match (*(*ent).client).ps.torsoAnim {
                BOTH_STAND1IDLE1 | BOTH_STAND2IDLE1 | BOTH_STAND2IDLE2 | BOTH_STAND3IDLE1
                | BOTH_STAND5IDLE1 => {
                    (*(*ent).client).ps.torsoTimer = 0;
                    (*(*ent).client).ps.weaponTime = 0;
                    (*(*ent).client).ps.saberMove = LS_READY;
                    brokeOut = QTRUE;
                }
                _ => {}
            }
        }
        //
        (*(*ent).client).idleHealth =
            (*ent).health + (*(*ent).client).ps.stats[STAT_ARMOR as usize];
        let viewangles = (*(*ent).client).ps.viewangles;
        crate::codemp::game::q_math::VectorCopy(
            &viewangles,
            &mut (*(*ent).client).idleViewAngles,
        );
        if (*(*ent).client).idleTime < (*addr_of!(level)).time {
            (*(*ent).client).idleTime = (*addr_of!(level)).time;
        }

        if brokeOut != QFALSE
            && ((*(*ent).client).ps.weaponstate == WEAPON_CHARGING
                || (*(*ent).client).ps.weaponstate == WEAPON_CHARGING_ALT)
        {
            (*(*ent).client).ps.torsoAnim = TORSO_RAISEWEAP1;
        }
    } else if (*addr_of!(level)).time - (*(*ent).client).idleTime > 5000 {
        //been idle for 5 seconds
        let mut idleAnim: c_int = -1;
        match (*(*ent).client).ps.legsAnim {
            BOTH_STAND1 => {
                idleAnim = BOTH_STAND1IDLE1;
            }
            BOTH_STAND2 => {
                idleAnim = BOTH_STAND2IDLE1; //Q_irand(BOTH_STAND2IDLE1,BOTH_STAND2IDLE2);
            }
            BOTH_STAND3 => {
                idleAnim = BOTH_STAND3IDLE1;
            }
            BOTH_STAND5 => {
                idleAnim = BOTH_STAND5IDLE1;
            }
            _ => {}
        }

        if idleAnim == BOTH_STAND2IDLE1 && Q_irand(1, 10) <= 5 {
            idleAnim = BOTH_STAND2IDLE2;
        }

        if idleAnim != -1 && /*PM_HasAnimation( ent, idleAnim )*/idleAnim > 0 && idleAnim < MAX_ANIMATIONS {
            G_SetAnim(
                ent,
                ucmd,
                SETANIM_BOTH,
                idleAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );

            //don't idle again after this anim for a while
            //ent->client->idleTime = level.time + PM_AnimLength( ent->client->clientInfo.animFileIndex, (animNumber_t)idleAnim ) + Q_irand( 0, 2000 );
            (*(*ent).client).idleTime =
                (*addr_of!(level)).time + (*(*ent).client).ps.legsTimer + Q_irand(0, 2000);
        }
    }
}

/*
==================
SpectatorClientEndFrame

==================
*/
/// `void SpectatorClientEndFrame( gentity_t *ent )` (g_active.c:3647). Per-frame
/// update for a spectator client — follows a chase target's playerstate, drops to free
/// spectate when the target leaves, and toggles the scoreboard pm_flag. No oracle —
/// client/session state over the `level` global, calling `ClientBegin`.
///
/// # Safety
/// `ent` and its `client` must be valid; the `level` global must be initialised.
pub unsafe fn SpectatorClientEndFrame(ent: *mut gentity_t) {
    let cl: *mut gclient_t;

    if (*ent).s.eType == ET_NPC {
        debug_assert!(false);
        return;
    }

    // if we are doing a chase cam or a remote view, grab the latest info
    if (*(*ent).client).sess.spectatorState == SPECTATOR_FOLLOW {
        let mut clientNum: c_int; //, flags;

        clientNum = (*(*ent).client).sess.spectatorClient;

        // team follow1 and team follow2 go to whatever clients are playing
        if clientNum == -1 {
            clientNum = (*addr_of!(level)).follow1;
        } else if clientNum == -2 {
            clientNum = (*addr_of!(level)).follow2;
        }
        if clientNum >= 0 {
            cl = (*addr_of!(level)).clients.add(clientNum as usize);
            if (*cl).pers.connected == CON_CONNECTED
                && (*cl).sess.sessionTeam != TEAM_SPECTATOR
            {
                //flags = (cl->mGameFlags & ~(PSG_VOTED | PSG_TEAMVOTED)) | (ent->client->mGameFlags & (PSG_VOTED | PSG_TEAMVOTED));
                //ent->client->mGameFlags = flags;
                (*(*ent).client).ps.eFlags = (*cl).ps.eFlags;
                (*(*ent).client).ps = (*cl).ps;
                (*(*ent).client).ps.pm_flags |= PMF_FOLLOW;
                return;
            } else {
                // drop them to free spectators unless they are dedicated camera followers
                if (*(*ent).client).sess.spectatorClient >= 0 {
                    (*(*ent).client).sess.spectatorState = SPECTATOR_FREE;
                    ClientBegin(
                        (*ent).client.offset_from((*addr_of!(level)).clients) as c_int,
                        QTRUE,
                    );
                }
            }
        }
    }

    if (*(*ent).client).sess.spectatorState == SPECTATOR_SCOREBOARD {
        (*(*ent).client).ps.pm_flags |= PMF_SCOREBOARD;
    } else {
        (*(*ent).client).ps.pm_flags &= !PMF_SCOREBOARD;
    }
}

// ---------------------------------------------------------------------------
// Guarded stubs for the not-yet-ported NPC item-use callees in ClientEvents.
// EV_USE_ITEM6 (sentry gun), EV_USE_ITEM10 (eweb) and EV_USE_ITEM11 (cloak)
// funnel into ItemUse_Sentry/ItemUse_UseEWeb/ItemUse_UseCloak, which depend on
// the not-yet-ported NPC/turret spawn paths. Faithful no-op placeholders so the
// dispatch table stays complete; un-stub when those item subsystems land.
// (Mirrors the FireVehicleWeapon guarded-stub precedent from cycle 31.)

// ItemUse_Sentry / ItemUse_UseCloak are ported in g_items.rs — imported here so the
// EV_USE_ITEM6 / EV_USE_ITEM11 events deploy the sentry / toggle the cloak as in C.
use crate::codemp::game::g_items::{ItemUse_Sentry, ItemUse_UseCloak};

/// `void ClientEvents( gentity_t *ent, int oldEventSequence )` (g_active.c:913). Walk the
/// player-state event ring from `oldEventSequence` to the current `eventSequence`, applying the
/// server-side effects of each event (fall/roll damage, weapon/saber fire bookkeeping, holdable
/// item use). Presentation is left to the client; only game effects are handled here. No oracle —
/// reads/writes `gentity_t`/`gclient_t` state, the `level`/cvar globals and the trap/item layer.
///
/// # Safety
/// `ent` and its `client` must be valid; the `level`/`g_dmflags`/`g_gametype` globals must be
/// initialised.
pub unsafe fn ClientEvents(ent: *mut gentity_t, mut oldEventSequence: c_int) {
    let mut i: c_int; //, j;
    let mut event: c_int;
    let client: *mut gclient_t;
    let mut damage: c_int;
    let mut dir: vec3_t = [0.0; 3];
    //	vec3_t	origin, angles;
    //	qboolean	fired;
    //	gitem_t *item;
    //	gentity_t *drop;

    client = (*ent).client;

    if oldEventSequence < (*client).ps.eventSequence - MAX_PS_EVENTS as c_int {
        oldEventSequence = (*client).ps.eventSequence - MAX_PS_EVENTS as c_int;
    }
    i = oldEventSequence;
    while i < (*client).ps.eventSequence {
        event = (*client).ps.events[(i & (MAX_PS_EVENTS as c_int - 1)) as usize];

        match event {
            _ if event == EV_FALL || event == EV_ROLL => {
                let delta: c_int = (*client).ps.eventParms[(i & (MAX_PS_EVENTS as c_int - 1)) as usize];
                let mut knockDownage: qboolean = QFALSE;

                if !(*ent).client.is_null() && (*(*ent).client).ps.fallingToDeath != 0 {
                    i += 1;
                    continue;
                }

                if (*ent).s.eType != ET_PLAYER {
                    i += 1;
                    continue; // not in the player model
                }

                if (*addr_of!(g_dmflags)).integer & DF_NO_FALLING != 0 {
                    i += 1;
                    continue;
                }

                if BG_InKnockDownOnly((*(*ent).client).ps.legsAnim) != QFALSE {
                    if delta <= 14 {
                        i += 1;
                        continue;
                    }
                    knockDownage = QTRUE;
                } else {
                    if delta <= 44 {
                        i += 1;
                        continue;
                    }
                }

                if knockDownage != QFALSE {
                    damage = delta * 1; //you suffer for falling unprepared. A lot. Makes throws and things useful, and more realistic I suppose.
                } else {
                    if (*addr_of!(g_gametype)).integer == GT_SIEGE && delta > 60 {
                        //longer falls hurt more
                        damage = delta * 1; //good enough for now, I guess
                    } else {
                        damage = (delta as f32 * 0.16) as c_int; //good enough for now, I guess
                    }
                }

                VectorSet(&mut dir, 0.0, 0.0, 1.0);
                (*ent).pain_debounce_time = (*addr_of!(level)).time + 200; // no normal pain sound
                G_Damage(
                    ent,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    damage,
                    DAMAGE_NO_ARMOR,
                    MOD_FALLING,
                );

                if (*ent).health < 1 {
                    G_Sound(ent, CHAN_AUTO, G_SoundIndex("sound/player/fallsplat.wav"));
                }
            }
            _ if event == EV_FIRE_WEAPON => {
                FireWeapon(ent, QFALSE);
                (*(*ent).client).dangerTime = (*addr_of!(level)).time;
                (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
                (*(*ent).client).invulnerableTimer = 0;
            }

            _ if event == EV_ALT_FIRE => {
                FireWeapon(ent, QTRUE);
                (*(*ent).client).dangerTime = (*addr_of!(level)).time;
                (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
                (*(*ent).client).invulnerableTimer = 0;
            }

            _ if event == EV_SABER_ATTACK => {
                (*(*ent).client).dangerTime = (*addr_of!(level)).time;
                (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
                (*(*ent).client).invulnerableTimer = 0;
            }

            //rww - Note that these must be in the same order (ITEM#-wise) as they are in holdable_t
            _ if event == EV_USE_ITEM1 => {
                //seeker droid
                ItemUse_Seeker(ent);
            }
            _ if event == EV_USE_ITEM2 => {
                //shield
                ItemUse_Shield(ent);
            }
            _ if event == EV_USE_ITEM3 => {
                //medpack
                ItemUse_MedPack(ent);
            }
            _ if event == EV_USE_ITEM4 => {
                //big medpack
                ItemUse_MedPack_Big(ent);
            }
            _ if event == EV_USE_ITEM5 => {
                //binoculars
                ItemUse_Binoculars(ent);
            }
            _ if event == EV_USE_ITEM6 => {
                //sentry gun
                ItemUse_Sentry(ent);
            }
            _ if event == EV_USE_ITEM7 => {
                //jetpack
                ItemUse_Jetpack(ent);
            }
            _ if event == EV_USE_ITEM8 => {
                //health disp
                //ItemUse_UseDisp(ent, HI_HEALTHDISP);
            }
            _ if event == EV_USE_ITEM9 => {
                //ammo disp
                //ItemUse_UseDisp(ent, HI_AMMODISP);
            }
            _ if event == EV_USE_ITEM10 => {
                //eweb
                ItemUse_UseEWeb(ent);
            }
            _ if event == EV_USE_ITEM11 => {
                //cloak
                ItemUse_UseCloak(ent);
            }
            _ => {}
        }

        i += 1;
    }
}

/// `void NPC_Accelerate( gentity_t *ent, qboolean fullWalkAcc, qboolean fullRunAcc )`
/// (g_active.c:1437). Adjust `ent->NPC->currentSpeed` toward `desiredSpeed` using the NPC's
/// acceleration, walking up to `walkSpeed` and running above it.
///
/// No oracle: pure entity-state arithmetic — dereferences `gentity_t->client`/`->NPC` and the
/// `gNPC_t` speed/stat fields (`currentSpeed`/`desiredSpeed`/`stats.{acceleration,walkSpeed}`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; its `client`/`NPC` may be NULL (handled).
pub unsafe fn NPC_Accelerate(ent: *mut gentity_t, fullWalkAcc: qboolean, fullRunAcc: qboolean) {
    if (*ent).client.is_null() || (*ent).NPC.is_null() {
        return;
    }

    if (*(*ent).NPC).stats.acceleration == 0 {
        //No acceleration means just start and stop
        (*(*ent).NPC).currentSpeed = (*(*ent).NPC).desiredSpeed;
    }
    //FIXME:  in cinematics always accel/decel?
    else if (*(*ent).NPC).desiredSpeed <= (*(*ent).NPC).stats.walkSpeed {
        //Only accelerate if at walkSpeeds
        if (*(*ent).NPC).desiredSpeed > (*(*ent).NPC).currentSpeed + (*(*ent).NPC).stats.acceleration
        {
            //ent->client->ps.friction = 0;
            (*(*ent).NPC).currentSpeed += (*(*ent).NPC).stats.acceleration;
        } else if (*(*ent).NPC).desiredSpeed > (*(*ent).NPC).currentSpeed {
            //ent->client->ps.friction = 0;
            (*(*ent).NPC).currentSpeed = (*(*ent).NPC).desiredSpeed;
        } else if fullWalkAcc != QFALSE
            && (*(*ent).NPC).desiredSpeed
                < (*(*ent).NPC).currentSpeed - (*(*ent).NPC).stats.acceleration
        {
            //decelerate even when walking
            (*(*ent).NPC).currentSpeed -= (*(*ent).NPC).stats.acceleration;
        } else if (*(*ent).NPC).desiredSpeed < (*(*ent).NPC).currentSpeed {
            //stop on a dime
            (*(*ent).NPC).currentSpeed = (*(*ent).NPC).desiredSpeed;
        }
    } else
    //  if ( ent->NPC->desiredSpeed > ent->NPC->stats.walkSpeed )
    {
        //Only decelerate if at runSpeeds
        if fullRunAcc != QFALSE
            && (*(*ent).NPC).desiredSpeed
                > (*(*ent).NPC).currentSpeed + (*(*ent).NPC).stats.acceleration
        {
            //Accelerate to runspeed
            //ent->client->ps.friction = 0;
            (*(*ent).NPC).currentSpeed += (*(*ent).NPC).stats.acceleration;
        } else if (*(*ent).NPC).desiredSpeed > (*(*ent).NPC).currentSpeed {
            //accelerate instantly
            //ent->client->ps.friction = 0;
            (*(*ent).NPC).currentSpeed = (*(*ent).NPC).desiredSpeed;
        } else if fullRunAcc != QFALSE
            && (*(*ent).NPC).desiredSpeed
                < (*(*ent).NPC).currentSpeed - (*(*ent).NPC).stats.acceleration
        {
            (*(*ent).NPC).currentSpeed -= (*(*ent).NPC).stats.acceleration;
        } else if (*(*ent).NPC).desiredSpeed < (*(*ent).NPC).currentSpeed {
            (*(*ent).NPC).currentSpeed = (*(*ent).NPC).desiredSpeed;
        }
    }
}

/*
-------------------------
NPC_GetWalkSpeed
-------------------------
*/
/// `static int NPC_GetWalkSpeed( gentity_t *ent )` (g_active.c:1499). The NPC's walk speed,
/// switched on `ent->client->playerTeam` (currently a stub switch with only the
/// `NPCTEAM_PLAYER`/default arm, returning `stats.walkSpeed`).
///
/// No oracle: entity-state — dereferences `gentity_t->client`/`->NPC` and reads
/// `playerTeam`/`stats.walkSpeed`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; its `client`/`NPC` may be NULL (handled).
unsafe fn NPC_GetWalkSpeed(ent: *mut gentity_t) -> c_int {
    #[allow(unused_assignments)]
    let mut walkSpeed: c_int = 0;

    if (*ent).client.is_null() || (*ent).NPC.is_null() {
        return 0;
    }

    match (*(*ent).client).playerTeam {
        NPCTEAM_PLAYER => {
            //To shutup compiler, will add entries later (this is stub code)
            walkSpeed = (*(*ent).NPC).stats.walkSpeed;
        }
        _ => {
            walkSpeed = (*(*ent).NPC).stats.walkSpeed;
        }
    }

    walkSpeed
}

/*
-------------------------
NPC_GetRunSpeed
-------------------------
*/
/// `static int NPC_GetRunSpeed( gentity_t *ent )` (g_active.c:1522). The NPC's run speed; the
/// old `playerTeam` switch is preserved verbatim as a comment (TEAM_BORG/TEAM_8472/…), and the
/// live code switches on `ent->client->NPC_class` to keep droid classes at base `runSpeed` while
/// everything else gets a 1.3x bump.
///
/// No oracle: entity-state — dereferences `gentity_t->client`/`->NPC` and reads
/// `NPC_class`/`stats.runSpeed`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; its `client`/`NPC` may be NULL (handled).
unsafe fn NPC_GetRunSpeed(ent: *mut gentity_t) -> c_int {
    #[allow(unused_assignments)]
    let mut runSpeed: c_int = 0;

    if (*ent).client.is_null() || (*ent).NPC.is_null() {
        return 0;
    }
    /*
        switch ( ent->client->playerTeam )
        {
        case TEAM_BORG:
            runSpeed = ent->NPC->stats.runSpeed;
            runSpeed += BORG_RUN_INCR * (g_spskill->integer%3);
            break;

        case TEAM_8472:
            runSpeed = ent->NPC->stats.runSpeed;
            runSpeed += SPECIES_RUN_INCR * (g_spskill->integer%3);
            break;

        case TEAM_STASIS:
            runSpeed = ent->NPC->stats.runSpeed;
            runSpeed += STASIS_RUN_INCR * (g_spskill->integer%3);
            break;

        case TEAM_BOTS:
            runSpeed = ent->NPC->stats.runSpeed;
            break;

        default:
            runSpeed = ent->NPC->stats.runSpeed;
            break;
        }
    */
    // team no longer indicates species/race.  Use NPC_class to adjust speed for specific npc types
    match (*(*ent).client).NPC_class {
        // droid cases here to shut-up compiler
        CLASS_PROBE | CLASS_GONK | CLASS_R2D2 | CLASS_R5D2 | CLASS_MARK1 | CLASS_MARK2
        | CLASS_PROTOCOL | CLASS_ATST | CLASS_MOUSE | CLASS_SEEKER | CLASS_REMOTE => {
            // CLASS_ATST: hmm, not really your average droid
            runSpeed = (*(*ent).NPC).stats.runSpeed;
        }

        _ => {
            runSpeed = ((*(*ent).NPC).stats.runSpeed as f32 * 1.3f32) as c_int; //rww - seems to slow in MP for some reason.
        }
    }

    runSpeed
}

// ---------------------------------------------------------------------------
// Guarded stub for one remaining not-yet-ported callee of ClientThink_real
// (`saberCheckKnockdown_DuelLoss`, below). The NPC speed- and gravity-management
// blocks are now fully ported inline (gNPC_t is a complete struct), so they are no
// longer stubbed here.

unsafe fn G_HeldByMonster(ent: *mut gentity_t, ucmd: *mut *mut usercmd_t) {
    if !ent.is_null()
        && !(*ent).client.is_null()
        && (*(*ent).client).ps.hasLookTarget != QFALSE
    //NOTE: lookTarget is an entity number, so this presumes that client 0 is NOT a Rancor...
    {
        let monster: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.lookTarget as usize);
        if !monster.is_null() && !(*monster).client.is_null() {
            //take the monster's waypoint as your own
            (*ent).waypoint = (*monster).waypoint;
            if (*monster).s.NPC_class == CLASS_RANCOR {
                //only possibility right now, may add Wampa and Sand Creature later
                BG_AttachToRancor(
                    (*monster).ghoul2, //ghoul2 info
                    (*monster).r.currentAngles[YAW],
                    &(*monster).r.currentOrigin,
                    (*addr_of!(level)).time,
                    core::ptr::null_mut(),
                    &(*monster).modelScale,
                    (*(*monster).client).ps.eFlags2 & EF2_GENERIC_NPC_FLAG,
                    addr_of_mut!((*(*ent).client).ps.origin),
                    addr_of_mut!((*(*ent).client).ps.viewangles),
                    core::ptr::null_mut(),
                );
            }
            VectorClear(&mut (*(*ent).client).ps.velocity);
            G_SetOrigin(ent, &(*(*ent).client).ps.origin);
            SetClientViewAngle(ent, &(*(*ent).client).ps.viewangles);
            G_SetAngles(ent, &(*(*ent).client).ps.viewangles);
            crate::trap::LinkEntity(ent); //redundant?
        }
    }
    // don't allow movement, weapon switching, and most kinds of button presses
    (*(*ucmd)).forwardmove = 0;
    (*(*ucmd)).rightmove = 0;
    (*(*ucmd)).upmove = 0;
}

//Seems like a slightly less than ideal method for this, could it be done on the client?
unsafe fn G_CheckMovingLoopingSounds(ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    if !(*ent).client.is_null() {
        if (!(*ent).NPC.is_null()
            && VectorCompare(&vec3_origin, &(*(*ent).client).ps.moveDir) == QFALSE)//moving using moveDir
            || (*ucmd).forwardmove != 0
            || (*ucmd).rightmove != 0//moving using ucmds
            || ((*ucmd).upmove != 0 && FlyingCreature(ent) != QFALSE)//flier using ucmds to move
            || (FlyingCreature(ent) != QFALSE
                && VectorCompare(&vec3_origin, &(*(*ent).client).ps.velocity) == QFALSE
                && (*ent).health > 0)
        //flier using velocity to move
        {
            match (*(*ent).client).NPC_class {
                CLASS_R2D2 => {
                    (*ent).s.loopSound = G_SoundIndex("sound/chars/r2d2/misc/r2_move_lp.wav");
                }
                CLASS_R5D2 => {
                    (*ent).s.loopSound = G_SoundIndex("sound/chars/r2d2/misc/r2_move_lp2.wav");
                }
                CLASS_MARK2 => {
                    (*ent).s.loopSound = G_SoundIndex("sound/chars/mark2/misc/mark2_move_lp");
                }
                CLASS_MOUSE => {
                    (*ent).s.loopSound = G_SoundIndex("sound/chars/mouse/misc/mouse_lp");
                }
                CLASS_PROBE => {
                    (*ent).s.loopSound = G_SoundIndex("sound/chars/probe/misc/probedroidloop");
                }
                _ => {}
            }
        } else {
            //not moving under your own control, stop loopSound
            if (*(*ent).client).NPC_class == CLASS_R2D2
                || (*(*ent).client).NPC_class == CLASS_R5D2
                || (*(*ent).client).NPC_class == CLASS_MARK2
                || (*(*ent).client).NPC_class == CLASS_MOUSE
                || (*(*ent).client).NPC_class == CLASS_PROBE
            {
                (*ent).s.loopSound = 0;
            }
        }
    }
}

/// TODO: un-stub when the saber-lock duel-loss knockdown lands (`saberCheckKnockdown_DuelLoss`, g_saber.c).
unsafe fn saberCheckKnockdown_DuelLoss(
    _saberent: *mut gentity_t,
    _saberOwner: *mut gentity_t,
    _other: *mut gentity_t,
) {
}

// ForceTelepathy (w_force.rs) and Jedi_Cloak (npc_ai_jedi.rs) are ported — imported
// here so mind-trick and the jedi cloak actually fire as in C.
use crate::codemp::game::npc_ai_jedi::Jedi_Cloak;
use crate::codemp::game::w_force::ForceTelepathy;

// `Jedi_Decloak` (NPC_misc.c) is ported in npc_ai_jedi.rs — imported here (the prior no-op
// stub is dropped, so the cloak actually drops as in C).
use crate::codemp::game::npc_ai_jedi::Jedi_Decloak;

// TryUse (the +use entity-interaction path) is ported in g_utils.rs — imported here.
use crate::codemp::game::g_utils::TryUse;

/// `void ClientThink_real( gentity_t *ent )` (g_active.c:1882). The giant per-frame
/// client think: builds and runs the `Pmove`, then applies all the resulting game-side
/// effects (intermission/spectator handling, vehicle/NPC speed management, gravity, private
/// duels, force-throw "being thrown" physics, saber lock, force-power/item generic commands,
/// respawn, broadcasts, idle anims, and the vehicle ClientThink recursion). No oracle —
/// pure per-frame entity/client-state mutation through `Pmove`, the `trap_*` layer and the
/// `level`/cvar globals.
///
/// Not-yet-ported subsystems are guarded-stubbed (see the helpers above): the whole `ent->NPC`
/// speed/gravity block (opaque `gNPC_t` fields) is collapsed to a `!(*ent).NPC.is_null()`
/// placeholder; the held-by-monster, looping-sound, saber-lock-knockdown, saber-switch,
/// telepathy, cloak and `+use` callees are no-op placeholders. The C++ `ClientManager`/
/// `FF_XboxSaberRumble` Xbox force-feedback tail is omitted (Xbox-only, not part of the PC
/// server module).
///
/// # Safety
/// `ent` and its `client` must be valid; the `level`/`g_entities`/cvar globals must be
/// initialised.
pub unsafe fn ClientThink_real(ent: *mut gentity_t) {
    let client: *mut gclient_t;
    let mut pm: pmove_t;
    let oldEventSequence: c_int;
    let mut msec: c_int;
    let ucmd: *mut usercmd_t;
    let mut isNPC: qboolean = QFALSE;
    let controlledByPlayer: qboolean = QFALSE;
    let mut killJetFlags: qboolean = QTRUE;

    client = (*ent).client;

    if (*ent).s.eType == ET_NPC {
        isNPC = QTRUE;
    }

    // don't think if the client is not yet connected (and thus not yet spawned in)
    if (*client).pers.connected != CON_CONNECTED && isNPC == QFALSE {
        return;
    }

    // This code was moved here from clientThink to fix a problem with g_synchronousClients
    // being set to 1 when in vehicles.
    if (*ent).s.number < MAX_CLIENTS as c_int && (*(*ent).client).ps.m_iVehicleNum != 0 {
        //driving a vehicle
        if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.m_iVehicleNum as usize))
            .client
            .is_null()
        {
            let veh: *mut gentity_t =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.m_iVehicleNum as usize);

            if !(*veh).m_pVehicle.is_null()
                && (*(*veh).m_pVehicle).m_pPilot == ent as *mut bgEntity_t
            {
                //only take input from the pilot...
                (*(*veh).client).ps.commandTime = (*(*ent).client).ps.commandTime;
                core::ptr::copy_nonoverlapping(
                    &(*(*ent).client).pers.cmd as *const usercmd_t,
                    &mut (*(*veh).m_pVehicle).m_ucmd as *mut usercmd_t,
                    1,
                );
                if (*(*veh).m_pVehicle).m_ucmd.buttons & BUTTON_TALK != 0 {
                    //forced input if "chat bubble" is up
                    (*(*veh).m_pVehicle).m_ucmd.buttons = BUTTON_TALK;
                    (*(*veh).m_pVehicle).m_ucmd.forwardmove = 0;
                    (*(*veh).m_pVehicle).m_ucmd.rightmove = 0;
                    (*(*veh).m_pVehicle).m_ucmd.upmove = 0;
                }
            }
        }
    }

    if (*client).ps.pm_flags & PMF_FOLLOW == 0 {
        if (*addr_of!(g_gametype)).integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].saberStance != 0
        {
            //the class says we have to use this stance set.
            if (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].saberStance
                & (1 << (*client).ps.fd.saberAnimLevel)
                == 0
            {
                //the current stance is not in the bitmask, so find the first one that is.
                let mut i = SS_FAST;

                while i < SS_NUM_SABER_STYLES {
                    if (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].saberStance
                        & (1 << i)
                        != 0
                    {
                        if i == SS_DUAL && (*client).ps.saberHolstered == 1 {
                            //one saber should be off, adjust saberAnimLevel accordinly
                            (*client).ps.fd.saberAnimLevelBase = i;
                            (*client).ps.fd.saberAnimLevel = SS_FAST;
                            (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                        } else if i == SS_STAFF
                            && (*client).ps.saberHolstered == 1
                            && (*client).saber[0].singleBladeStyle != SS_NONE
                        {
                            //one saber or blade should be off, adjust saberAnimLevel accordinly
                            (*client).ps.fd.saberAnimLevelBase = i;
                            (*client).ps.fd.saberAnimLevel = (*client).saber[0].singleBladeStyle;
                            (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                        } else {
                            (*client).ps.fd.saberAnimLevel = i;
                            (*client).ps.fd.saberAnimLevelBase = (*client).ps.fd.saberAnimLevel;
                            (*client).ps.fd.saberDrawAnimLevel = i;
                        }
                        break;
                    }

                    i += 1;
                }
            }
        } else if (*client).saber[0].model[0] != 0 && (*client).saber[1].model[0] != 0 {
            //with two sabs always use akimbo style
            (*client).ps.fd.saberAnimLevelBase = SS_DUAL;
            if (*client).ps.saberHolstered == 1 {
                //one saber should be off, adjust saberAnimLevel accordinly
                (*client).ps.fd.saberAnimLevel = SS_FAST;
                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
            } else {
                (*client).ps.fd.saberAnimLevel = SS_DUAL;
                (*client).ps.fd.saberAnimLevelBase = (*client).ps.fd.saberAnimLevel;
                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
            }
        } else if (*client).saber[0].stylesLearned == 1 << SS_STAFF {
            //then always use the staff style
            (*client).ps.fd.saberAnimLevelBase = SS_STAFF;
            if (*client).ps.saberHolstered == 1 && (*client).saber[0].singleBladeStyle != SS_NONE {
                //one blade should be off, adjust saberAnimLevel accordinly
                (*client).ps.fd.saberAnimLevel = (*client).saber[0].singleBladeStyle;
                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
            } else {
                (*client).ps.fd.saberAnimLevel = SS_STAFF;
                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
            }
        }
    }

    // mark the time, so the connection sprite can be removed
    ucmd = &mut (*(*ent).client).pers.cmd;

    if !client.is_null() && (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
        G_HeldByMonster(ent, &mut (ucmd as *mut usercmd_t));
    }

    // sanity check the command time to prevent speedup cheating
    if (*ucmd).serverTime > (*addr_of!(level)).time + 200 {
        (*ucmd).serverTime = (*addr_of!(level)).time + 200;
        //		G_Printf("serverTime <<<<<\n" );
    }
    if (*ucmd).serverTime < (*addr_of!(level)).time - 1000 {
        (*ucmd).serverTime = (*addr_of!(level)).time - 1000;
        //		G_Printf("serverTime >>>>>\n" );
    }

    if isNPC != QFALSE && ((*ucmd).serverTime - (*client).ps.commandTime) < 1 {
        (*ucmd).serverTime = (*client).ps.commandTime + 100;
    }

    msec = (*ucmd).serverTime - (*client).ps.commandTime;
    // following others may result in bad times, but we still want
    // to check for follow toggles
    if msec < 1 && (*client).sess.spectatorState != SPECTATOR_FOLLOW {
        return;
    }

    if msec > 200 {
        msec = 200;
    }

    if (*addr_of!(pmove_msec)).integer < 8 {
        crate::trap::Cvar_Set("pmove_msec", "8");
    } else if (*addr_of!(pmove_msec)).integer > 33 {
        crate::trap::Cvar_Set("pmove_msec", "33");
    }

    if (*addr_of!(pmove_fixed)).integer != 0 || (*client).pers.pmoveFixed != 0 {
        (*ucmd).serverTime = (((*ucmd).serverTime + (*addr_of!(pmove_msec)).integer - 1)
            / (*addr_of!(pmove_msec)).integer)
            * (*addr_of!(pmove_msec)).integer;
        //if (ucmd->serverTime - client->ps.commandTime <= 0)
        //	return;
    }

    //
    // check for exiting intermission
    //
    if (*addr_of!(level)).intermissiontime != 0 {
        ClientIntermissionThink(client);
        return;
    }

    // spectators don't do much
    if (*client).sess.sessionTeam == TEAM_SPECTATOR
        || (*client).tempSpectate > (*addr_of!(level)).time
    {
        if (*client).sess.spectatorState == SPECTATOR_SCOREBOARD {
            return;
        }
        SpectatorThink(ent, ucmd);
        return;
    }

    if !ent.is_null()
        && !(*ent).client.is_null()
        && (*(*ent).client).ps.eFlags & EF_INVULNERABLE != 0
    {
        if (*(*ent).client).invulnerableTimer <= (*addr_of!(level)).time {
            (*(*ent).client).ps.eFlags &= !EF_INVULNERABLE;
        }
    }

    if (*ent).s.eType != ET_NPC {
        // check for inactivity timer, but never drop the local client of a non-dedicated server
        if ClientInactivityTimer(client) == QFALSE {
            return;
        }
    }

    //Check if we should have a fullbody push effect around the player
    if (*client).pushEffectTime > (*addr_of!(level)).time {
        (*client).ps.eFlags |= EF_BODYPUSH;
    } else if (*client).pushEffectTime != 0 {
        (*client).pushEffectTime = 0;
        (*client).ps.eFlags &= !EF_BODYPUSH;
    }

    if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_JETPACK) != 0 {
        (*client).ps.eFlags |= EF_JETPACK;
    } else {
        (*client).ps.eFlags &= !EF_JETPACK;
    }

    if (*client).noclip != 0 {
        (*client).ps.pm_type = PM_NOCLIP;
    } else if (*client).ps.eFlags & EF_DISINTEGRATION != 0 {
        (*client).ps.pm_type = PM_NOCLIP;
    } else if (*client).ps.stats[STAT_HEALTH as usize] <= 0 {
        (*client).ps.pm_type = PM_DEAD;
    } else {
        if (*client).ps.forceGripChangeMovetype != 0 {
            (*client).ps.pm_type = (*client).ps.forceGripChangeMovetype;
        } else {
            if (*client).jetPackOn != 0 {
                (*client).ps.pm_type = PM_JETPACK;
                (*client).ps.eFlags |= EF_JETPACK_ACTIVE;
                killJetFlags = QFALSE;
            } else {
                (*client).ps.pm_type = PM_NORMAL;
            }
        }
    }

    if killJetFlags != QFALSE {
        (*client).ps.eFlags &= !EF_JETPACK_ACTIVE;
        (*client).ps.eFlags &= !EF_JETPACK_FLAMING;
    }

    // `#define SLOWDOWN_DIST 128.0f` / `#define MIN_NPC_SPEED 16.0f` (g_active.c:2203-2204) —
    // used by the NPC desiredSpeed / turn-slowdown block below.
    const SLOWDOWN_DIST: f32 = 128.0;
    const MIN_NPC_SPEED: c_int = 16;

    if (*client).bodyGrabIndex != ENTITYNUM_NONE {
        let grabbed: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*client).bodyGrabIndex as usize);

        if (*grabbed).inuse == QFALSE
            || (*grabbed).s.eType != ET_BODY
            || ((*grabbed).s.eFlags & EF_DISINTEGRATION != 0)
            || ((*grabbed).s.eFlags & EF_NODRAW != 0)
        {
            if (*grabbed).inuse != QFALSE && (*grabbed).s.eType == ET_BODY {
                (*grabbed).s.ragAttach = 0;
            }
            (*client).bodyGrabIndex = ENTITYNUM_NONE;
        } else {
            let mut rhMat: mdxaBone_t = core::mem::zeroed();
            let mut rhOrg: vec3_t = [0.0; 3];
            let mut tAng: vec3_t = [0.0; 3];
            let mut bodyDir: vec3_t = [0.0; 3];
            let bodyDist: f32;

            (*(*ent).client).ps.forceHandExtend = HANDEXTEND_DRAGGING;

            if (*(*ent).client).ps.forceHandExtendTime < (*addr_of!(level)).time + 500 {
                (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
            }

            VectorSet(&mut tAng, 0.0, (*(*ent).client).ps.viewangles[YAW as usize], 0.0);
            crate::trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                0,
                &mut rhMat,
                &tAng,
                &(*(*ent).client).ps.origin,
                (*addr_of!(level)).time,
                core::ptr::null_mut(),
                &(*ent).modelScale,
            ); //0 is always going to be right hand bolt
            BG_GiveMeVectorFromMatrix(&rhMat, ORIGIN, &mut rhOrg);

            VectorSubtract(&rhOrg, &(*grabbed).r.currentOrigin, &mut bodyDir);
            bodyDist = VectorLength(&bodyDir);

            if bodyDist > 40.0 {
                //can no longer reach
                (*grabbed).s.ragAttach = 0;
                (*client).bodyGrabIndex = ENTITYNUM_NONE;
            } else if bodyDist > 24.0 {
                bodyDir[2] = 0.0; //don't want it floating
                                  //VectorScale(bodyDir, 0.1f, bodyDir);
                let epv = (*grabbed).epVelocity;
                VectorAdd(&epv, &bodyDir, &mut (*grabbed).epVelocity);
                G_Sound(grabbed, CHAN_AUTO, G_SoundIndex("sound/player/roll1.wav"));
            }
        }
    } else if (*(*ent).client).ps.forceHandExtend == HANDEXTEND_DRAGGING {
        (*(*ent).client).ps.forceHandExtend = HANDEXTEND_WEAPONREADY;
    }

    if !(*ent).NPC.is_null() && (*ent).s.NPC_class != CLASS_VEHICLE {
        //vehicles manage their own speed
        // Port of g_active.c:2261-2420 — NPC desiredSpeed selection, distance/turn
        // slowdown and acceleration. For player clients (ent->NPC == NULL) this branch is
        // never taken.
        let npc = (*ent).NPC;
        use crate::codemp::game::b_public_h::NPCAI_NO_SLOWDOWN;
        use crate::codemp::game::bg_public::EF2_FLYING;
        use crate::codemp::game::q_math::AngleDelta;
        use crate::codemp::game::q_shared_h::BUTTON_WALKING;
        use crate::codemp::game::surfaceflags_h::CONTENTS_LADDER;
        //FIXME: swoop should keep turning (and moving forward?) for a little bit?
        if (*npc).combatMove == QFALSE {
            //if ( !(ucmd->buttons & BUTTON_USE) ) / if (1) -- Not leaning
            let flying = (*ucmd).upmove != 0 && ((*client).ps.eFlags2 & EF2_FLYING) != 0;
            let climbing = (*ucmd).upmove != 0 && ((*ent).watertype & CONTENTS_LADDER) != 0;

            if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 || flying {
                //In-Formation NPCs set their desiredSpeed themselves
                if ((*ucmd).buttons & BUTTON_WALKING) != 0 {
                    (*npc).desiredSpeed = NPC_GetWalkSpeed(ent);
                } else {
                    //running
                    (*npc).desiredSpeed = NPC_GetRunSpeed(ent);
                }

                if (*npc).currentSpeed >= 80 && controlledByPlayer == QFALSE {
                    //At higher speeds, need to slow down close to stuff; slow down as
                    //you approach your goal
                    if (*npc).distToGoal < SLOWDOWN_DIST
                        && ((*npc).aiFlags & NPCAI_NO_SLOWDOWN) == 0
                    {
                        if (*npc).desiredSpeed > MIN_NPC_SPEED {
                            let slowdown_speed =
                                (*npc).desiredSpeed as f32 * (*npc).distToGoal / SLOWDOWN_DIST;
                            (*npc).desiredSpeed = slowdown_speed.ceil() as c_int;
                            if (*npc).desiredSpeed < MIN_NPC_SPEED {
                                //don't slow down too much
                                (*npc).desiredSpeed = MIN_NPC_SPEED;
                            }
                        }
                    }
                }
            } else if climbing {
                (*npc).desiredSpeed = (*npc).stats.walkSpeed;
            } else {
                //We want to stop
                (*npc).desiredSpeed = 0;
            }

            NPC_Accelerate(ent, QFALSE, QFALSE);

            if (*npc).currentSpeed <= 24 && (*npc).desiredSpeed < (*npc).currentSpeed {
                //No-one walks this slow -- Full stop
                (*npc).currentSpeed = 0;
                (*client).ps.speed = 0.0;
                (*ucmd).forwardmove = 0;
                (*ucmd).rightmove = 0;
            } else {
                if (*npc).currentSpeed <= (*npc).stats.walkSpeed {
                    //Play the walkanim
                    (*ucmd).buttons |= BUTTON_WALKING;
                } else {
                    (*ucmd).buttons &= !BUTTON_WALKING;
                }

                if (*npc).currentSpeed > 0 {
                    //We should be moving
                    if climbing || flying {
                        if (*ucmd).upmove == 0 {
                            //force them to take a couple more steps until stopped
                            (*ucmd).upmove = (*npc).last_ucmd.upmove;
                        }
                    } else if (*ucmd).forwardmove == 0 && (*ucmd).rightmove == 0 {
                        //force them to take a couple more steps until stopped
                        (*ucmd).forwardmove = (*npc).last_ucmd.forwardmove;
                        (*ucmd).rightmove = (*npc).last_ucmd.rightmove;
                    }
                }

                (*client).ps.speed = (*npc).currentSpeed as f32;

                //rwwFIXMEFIXME: do this and also check for all real client / if (1)
                //Slow down on turns - don't orbit!!! (locked-yaw path `if (0)` not taken)
                let turndelta = (180.0
                    - AngleDelta((*ent).r.currentAngles[YAW as usize], (*npc).desiredYaw).abs())
                    / 180.0;

                if turndelta < 0.75 {
                    (*client).ps.speed = 0.0;
                } else if (*npc).distToGoal < 100.0 && turndelta < 1.0 {
                    //Turn is greater than 45 degrees or closer than 100 to goal
                    (*client).ps.speed = ((*client).ps.speed * turndelta).floor();
                }
            }
        } else {
            (*npc).desiredSpeed = if ((*ucmd).buttons & BUTTON_WALKING) != 0 {
                NPC_GetWalkSpeed(ent)
            } else {
                NPC_GetRunSpeed(ent)
            };

            (*client).ps.speed = (*npc).desiredSpeed as f32;
        }

        if ((*ucmd).buttons & BUTTON_WALKING) != 0 {
            //sort of a hack since MP handles walking differently from SP (has some proxy
            //cheat prevention methods)
            if (*ucmd).forwardmove > 64 {
                (*ucmd).forwardmove = 64;
            } else if (*ucmd).forwardmove < -64 {
                (*ucmd).forwardmove = -64;
            }

            if (*ucmd).rightmove > 64 {
                (*ucmd).rightmove = 64;
            } else if (*ucmd).rightmove < -64 {
                (*ucmd).rightmove = -64;
            }
        }
        (*client).ps.basespeed = (*client).ps.speed as c_int;
    } else if (*client).ps.m_iVehicleNum == 0
        && ((*ent).NPC.is_null() || (*ent).s.NPC_class != CLASS_VEHICLE)
    {
        //if riding a vehicle it will manage our speed and such
        // set speed
        (*client).ps.speed = (*addr_of!(g_speed)).value;

        //Check for a siege class speed multiplier
        if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1 {
            (*client).ps.speed *=
                (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].speed;
        }

        if (*client).bodyGrabIndex != ENTITYNUM_NONE {
            //can't go nearly as fast when dragging a body around
            (*client).ps.speed *= 0.2;
        }

        (*client).ps.basespeed = (*client).ps.speed as c_int;
    }

    // Port of g_active.c:2443-2472 — gravity selection.
    if (*ent).NPC.is_null()
        || ((*(*ent).NPC).aiFlags & crate::codemp::game::b_public_h::NPCAI_CUSTOM_GRAVITY) == 0
    {
        //use global gravity
        if !(*ent).NPC.is_null()
            && (*ent).s.NPC_class == CLASS_VEHICLE
            && !(*ent).m_pVehicle.is_null()
            && (*(*(*ent).m_pVehicle).m_pVehicleInfo).gravity != 0
        {
            //use custom veh gravity
            (*client).ps.gravity = (*(*(*ent).m_pVehicle).m_pVehicleInfo).gravity;
        } else if (*(*ent).client).inSpaceIndex != 0
            && (*(*ent).client).inSpaceIndex != ENTITYNUM_NONE
        {
            //in space, so no gravity...
            (*client).ps.gravity = 1;
            if (*ent).s.number < MAX_CLIENTS as c_int {
                let vel = (*client).ps.velocity;
                VectorScale(&vel, 0.8, &mut (*client).ps.velocity);
            }
        } else if (*client).ps.eFlags2 & EF2_SHIP_DEATH != 0 {
            //float there
            VectorClear(&mut (*client).ps.velocity);
            (*client).ps.gravity = 1;
        } else {
            (*client).ps.gravity = (*addr_of!(g_gravity)).value as c_int;
        }
    }

    if (*(*ent).client).ps.duelInProgress != 0 {
        let duelAgainst: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.duelIndex as usize);

        //Keep the time updated, so once this duel ends this player can't engage in a duel for another
        //10 seconds. This will give other people a chance to engage in duels in case this player wants
        //to engage again right after he's done fighting and someone else is waiting.
        (*(*ent).client).ps.fd.privateDuelTime = (*addr_of!(level)).time + 10000;

        if (*(*ent).client).ps.duelTime < (*addr_of!(level)).time {
            //Bring out the sabers
            if (*(*ent).client).ps.weapon == WP_SABER
                && (*(*ent).client).ps.saberHolstered != 0
                && (*(*ent).client).ps.duelTime != 0
            {
                (*(*ent).client).ps.saberHolstered = 0;

                if (*(*ent).client).saber[0].soundOn != 0 {
                    G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[0].soundOn);
                }
                if (*(*ent).client).saber[1].soundOn != 0 {
                    G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOn);
                }

                G_AddEvent(ent, EV_PRIVATE_DUEL, 2);

                (*(*ent).client).ps.duelTime = 0;
            }

            if !duelAgainst.is_null()
                && !(*duelAgainst).client.is_null()
                && (*duelAgainst).inuse != QFALSE
                && (*(*duelAgainst).client).ps.weapon == WP_SABER
                && (*(*duelAgainst).client).ps.saberHolstered != 0
                && (*(*duelAgainst).client).ps.duelTime != 0
            {
                (*(*duelAgainst).client).ps.saberHolstered = 0;

                if (*(*duelAgainst).client).saber[0].soundOn != 0 {
                    G_Sound(duelAgainst, CHAN_AUTO, (*(*duelAgainst).client).saber[0].soundOn);
                }
                if (*(*duelAgainst).client).saber[1].soundOn != 0 {
                    G_Sound(duelAgainst, CHAN_AUTO, (*(*duelAgainst).client).saber[1].soundOn);
                }

                G_AddEvent(duelAgainst, EV_PRIVATE_DUEL, 2);

                (*(*duelAgainst).client).ps.duelTime = 0;
            }
        } else {
            (*client).ps.speed = 0.0;
            (*client).ps.basespeed = 0;
            (*ucmd).forwardmove = 0;
            (*ucmd).rightmove = 0;
            (*ucmd).upmove = 0;
        }

        if duelAgainst.is_null()
            || (*duelAgainst).client.is_null()
            || (*duelAgainst).inuse == QFALSE
            || (*(*duelAgainst).client).ps.duelIndex != (*ent).s.number
        {
            (*(*ent).client).ps.duelInProgress = 0;
            G_AddEvent(ent, EV_PRIVATE_DUEL, 0);
        } else if (*duelAgainst).health < 1
            || (*(*duelAgainst).client).ps.stats[STAT_HEALTH as usize] < 1
        {
            (*(*ent).client).ps.duelInProgress = 0;
            (*(*duelAgainst).client).ps.duelInProgress = 0;

            G_AddEvent(ent, EV_PRIVATE_DUEL, 0);
            G_AddEvent(duelAgainst, EV_PRIVATE_DUEL, 0);

            //Winner gets full health.. providing he's still alive
            if (*ent).health > 0 && (*(*ent).client).ps.stats[STAT_HEALTH as usize] > 0 {
                if (*ent).health < (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
                    (*(*ent).client).ps.stats[STAT_HEALTH as usize] =
                        (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
                    (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
                }

                if (*addr_of!(g_spawnInvulnerability)).integer != 0 {
                    (*(*ent).client).ps.eFlags |= EF_INVULNERABLE;
                    (*(*ent).client).invulnerableTimer =
                        (*addr_of!(level)).time + (*addr_of!(g_spawnInvulnerability)).integer;
                }
            }

            /*
            trap_SendServerCommand( ent-g_entities, va("print \"%s %s\n\"", ent->client->pers.netname, G_GetStringEdString("MP_SVGAME", "PLDUELWINNER")) );
            trap_SendServerCommand( duelAgainst-g_entities, va("print \"%s %s\n\"", ent->client->pers.netname, G_GetStringEdString("MP_SVGAME", "PLDUELWINNER")) );
            */
            //Private duel announcements are now made globally because we only want one duel at a time.
            if (*ent).health > 0 && (*(*ent).client).ps.stats[STAT_HEALTH as usize] > 0 {
                crate::trap::SendServerCommand(
                    -1,
                    &format!(
                        "cp \"{} {} {}!\n\"",
                        Sz((*(*ent).client).pers.netname.as_ptr()),
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"PLDUELWINNER".as_ptr(),
                        )),
                        Sz((*(*duelAgainst).client).pers.netname.as_ptr()),
                    ),
                );
            } else {
                //it was a draw, because we both managed to die in the same frame
                crate::trap::SendServerCommand(
                    -1,
                    &format!(
                        "cp \"{}\n\"",
                        Sz(G_GetStringEdString(c"MP_SVGAME".as_ptr(), c"PLDUELTIE".as_ptr(),)),
                    ),
                );
            }
        } else {
            let mut vSub: vec3_t = [0.0; 3];
            let subLen: f32;

            VectorSubtract(
                &(*(*ent).client).ps.origin,
                &(*(*duelAgainst).client).ps.origin,
                &mut vSub,
            );
            subLen = VectorLength(&vSub);

            if subLen >= 1024.0 {
                (*(*ent).client).ps.duelInProgress = 0;
                (*(*duelAgainst).client).ps.duelInProgress = 0;

                G_AddEvent(ent, EV_PRIVATE_DUEL, 0);
                G_AddEvent(duelAgainst, EV_PRIVATE_DUEL, 0);

                crate::trap::SendServerCommand(
                    -1,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(c"MP_SVGAME".as_ptr(), c"PLDUELSTOP".as_ptr(),)),
                    ),
                );
            }
        }
    }

    if (*(*ent).client).doingThrow > (*addr_of!(level)).time {
        let throwee: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).throwingIndex as usize);

        if (*throwee).inuse == QFALSE
            || (*throwee).client.is_null()
            || (*throwee).health < 1
            || (*(*throwee).client).sess.sessionTeam == TEAM_SPECTATOR
            || ((*(*throwee).client).ps.pm_flags & PMF_FOLLOW != 0)
            || (*(*throwee).client).throwingIndex != (*ent).s.number
        {
            (*(*ent).client).doingThrow = 0;
            (*(*ent).client).ps.forceHandExtend = HANDEXTEND_NONE;

            if (*throwee).inuse != QFALSE && !(*throwee).client.is_null() {
                (*(*throwee).client).ps.heldByClient = 0;
                (*(*throwee).client).beingThrown = 0;

                if (*(*throwee).client).ps.forceHandExtend != HANDEXTEND_POSTTHROWN {
                    (*(*throwee).client).ps.forceHandExtend = HANDEXTEND_NONE;
                }
            }
        }
    }

    if (*(*ent).client).beingThrown > (*addr_of!(level)).time {
        let thrower: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).throwingIndex as usize);

        if (*thrower).inuse == QFALSE
            || (*thrower).client.is_null()
            || (*thrower).health < 1
            || (*(*thrower).client).sess.sessionTeam == TEAM_SPECTATOR
            || ((*(*thrower).client).ps.pm_flags & PMF_FOLLOW != 0)
            || (*(*thrower).client).throwingIndex != (*ent).s.number
        {
            (*(*ent).client).ps.heldByClient = 0;
            (*(*ent).client).beingThrown = 0;

            if (*(*ent).client).ps.forceHandExtend != HANDEXTEND_POSTTHROWN {
                (*(*ent).client).ps.forceHandExtend = HANDEXTEND_NONE;
            }

            if (*thrower).inuse != QFALSE && !(*thrower).client.is_null() {
                (*(*thrower).client).doingThrow = 0;
                (*(*thrower).client).ps.forceHandExtend = HANDEXTEND_NONE;
            }
        } else if (*thrower).inuse != QFALSE
            && !(*thrower).client.is_null()
            && !(*thrower).ghoul2.is_null()
            && crate::trap::G2_HaveWeGhoul2Models((*thrower).ghoul2) != QFALSE
        {
            // #if 0 lHandBolt/pelBolt block omitted (compiled out upstream)
            {
                let pDif: f32 = 40.0;
                let mut boltOrg: vec3_t = [0.0; 3];
                let mut pBoltOrg: vec3_t = [0.0; 3];
                let mut tAngles: vec3_t = [0.0; 3];
                let mut vDif: vec3_t = [0.0; 3];
                let mut entDir: vec3_t = [0.0; 3];
                let mut otherAngles: vec3_t = [0.0; 3];
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];

                //Always look at the thrower.
                VectorSubtract(
                    &(*(*thrower).client).ps.origin,
                    &(*(*ent).client).ps.origin,
                    &mut entDir,
                );
                VectorCopy(&(*(*ent).client).ps.viewangles, &mut otherAngles);
                otherAngles[YAW as usize] = vectoyaw(&entDir);
                SetClientViewAngle(ent, &otherAngles);

                VectorCopy(&(*(*thrower).client).ps.viewangles, &mut tAngles);
                tAngles[ROLL as usize] = 0.0;
                tAngles[PITCH as usize] = 0.0;

                //Get the direction between the pelvis and position of the hand
                // (#if 0 G2 boltMatrix path omitted; the #else fallback below is used)
                VectorCopy(&(*(*thrower).client).ps.origin, &mut pBoltOrg);
                AngleVectors(&tAngles, Some(&mut fwd), Some(&mut right), None);
                boltOrg[0] = pBoltOrg[0] + fwd[0] * 8.0 + right[0] * pDif;
                boltOrg[1] = pBoltOrg[1] + fwd[1] * 8.0 + right[1] * pDif;
                boltOrg[2] = pBoltOrg[2];

                //G_TestLine(boltOrg, pBoltOrg, 0x0000ff, 50);

                VectorSubtract(&(*(*ent).client).ps.origin, &boltOrg, &mut vDif);
                if VectorLength(&vDif) > 32.0
                    && ((*(*thrower).client).doingThrow - (*addr_of!(level)).time) < 4500
                {
                    //the hand is too far away, and can no longer hold onto us, so escape.
                    (*(*ent).client).ps.heldByClient = 0;
                    (*(*ent).client).beingThrown = 0;
                    (*(*thrower).client).doingThrow = 0;

                    (*(*thrower).client).ps.forceHandExtend = HANDEXTEND_NONE;
                    G_EntitySound(thrower, CHAN_VOICE, G_SoundIndex("*pain25.wav"));

                    (*(*ent).client).ps.forceDodgeAnim = 2;
                    (*(*ent).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                    (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 500;
                    (*(*ent).client).ps.velocity[2] = 400.0;
                    G_PreDefSound(&(*(*ent).client).ps.origin, PDSOUND_FORCEJUMP);
                } else if ((*client).beingThrown - (*addr_of!(level)).time) < 4000 {
                    //step into the next part of the throw, and go flying back
                    let vScale: f32 = 400.0;
                    (*(*ent).client).ps.forceHandExtend = HANDEXTEND_POSTTHROWN;
                    (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1200;
                    (*(*ent).client).ps.forceDodgeAnim = 0;

                    (*(*thrower).client).ps.forceHandExtend = HANDEXTEND_POSTTHROW;
                    (*(*thrower).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 200;

                    (*(*ent).client).ps.heldByClient = 0;

                    (*(*ent).client).ps.heldByClient = 0;
                    (*(*ent).client).beingThrown = 0;
                    (*(*thrower).client).doingThrow = 0;

                    AngleVectors(&(*(*thrower).client).ps.viewangles, Some(&mut vDif), None, None);
                    (*(*ent).client).ps.velocity[0] = vDif[0] * vScale;
                    (*(*ent).client).ps.velocity[1] = vDif[1] * vScale;
                    (*(*ent).client).ps.velocity[2] = 400.0;

                    G_EntitySound(ent, CHAN_VOICE, G_SoundIndex("*pain100.wav"));
                    G_EntitySound(thrower, CHAN_VOICE, G_SoundIndex("*jump1.wav"));

                    //Set the thrower as the "other killer", so if we die from fall/impact damage he is credited.
                    (*(*ent).client).ps.otherKiller = (*thrower).s.number;
                    (*(*ent).client).ps.otherKillerTime = (*addr_of!(level)).time + 8000;
                    (*(*ent).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 100;
                } else {
                    //see if we can move to be next to the hand.. if it's not clear, break the throw.
                    let mut intendedOrigin: vec3_t = [0.0; 3];
                    let tr: trace_t;
                    let tr2: trace_t;

                    VectorSubtract(&boltOrg, &pBoltOrg, &mut vDif);
                    VectorNormalize(&mut vDif);

                    VectorClear(&mut (*(*ent).client).ps.velocity);
                    intendedOrigin[0] = pBoltOrg[0] + vDif[0] * pDif;
                    intendedOrigin[1] = pBoltOrg[1] + vDif[1] * pDif;
                    intendedOrigin[2] = (*(*thrower).client).ps.origin[2];

                    tr = crate::trap::Trace(
                        &intendedOrigin,
                        &(*ent).r.mins,
                        &(*ent).r.maxs,
                        &intendedOrigin,
                        (*ent).s.number,
                        (*ent).clipmask,
                    );
                    tr2 = crate::trap::Trace(
                        &(*(*ent).client).ps.origin,
                        &(*ent).r.mins,
                        &(*ent).r.maxs,
                        &intendedOrigin,
                        (*ent).s.number,
                        CONTENTS_SOLID,
                    );

                    if tr.fraction == 1.0
                        && tr.startsolid == 0
                        && tr2.fraction == 1.0
                        && tr2.startsolid == 0
                    {
                        VectorCopy(&intendedOrigin, &mut (*(*ent).client).ps.origin);

                        if ((*client).beingThrown - (*addr_of!(level)).time) < 4800 {
                            (*(*ent).client).ps.heldByClient = (*thrower).s.number + 1;
                        }
                    } else {
                        //if the guy can't be put here then it's time to break the throw off.
                        (*(*ent).client).ps.heldByClient = 0;
                        (*(*ent).client).beingThrown = 0;
                        (*(*thrower).client).doingThrow = 0;

                        (*(*thrower).client).ps.forceHandExtend = HANDEXTEND_NONE;
                        G_EntitySound(thrower, CHAN_VOICE, G_SoundIndex("*pain25.wav"));

                        (*(*ent).client).ps.forceDodgeAnim = 2;
                        (*(*ent).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                        (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 500;
                        (*(*ent).client).ps.velocity[2] = 400.0;
                        G_PreDefSound(&(*(*ent).client).ps.origin, PDSOUND_FORCEJUMP);
                    }
                }
            }
        }
    } else if (*(*ent).client).ps.heldByClient != 0 {
        (*(*ent).client).ps.heldByClient = 0;
    }

    /*
    if ( client->ps.powerups[PW_HASTE] ) {
        client->ps.speed *= 1.3;
    }
    */

    //Will probably never need this again, since we have g2 properly serverside now.
    //But just in case.
    /* ATST damage-box block omitted (compiled out upstream) */

    //rww - moved this stuff into the pmove code so that it's predicted properly
    //BG_AdjustClientSpeed(&client->ps, &client->pers.cmd, level.time);

    // set up for pmove
    oldEventSequence = (*client).ps.eventSequence;

    pm = core::mem::zeroed();

    if (*ent).flags & FL_FORCE_GESTURE != 0 {
        (*ent).flags &= !FL_FORCE_GESTURE;
        (*(*ent).client).pers.cmd.buttons |= BUTTON_GESTURE;
    }

    if !(*ent).client.is_null()
        && (*(*ent).client).ps.fallingToDeath != 0
        && ((*addr_of!(level)).time - FALL_FADE_TIME) > (*(*ent).client).ps.fallingToDeath
    {
        //die!
        if (*ent).health > 0 {
            let mut otherKiller: *mut gentity_t = ent;
            if (*(*ent).client).ps.otherKillerTime > (*addr_of!(level)).time
                && (*(*ent).client).ps.otherKiller != ENTITYNUM_NONE
            {
                otherKiller =
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.otherKiller as usize);

                if (*otherKiller).inuse == QFALSE {
                    otherKiller = ent;
                }
            }
            G_Damage(
                ent,
                otherKiller,
                otherKiller,
                core::ptr::null_mut(),
                addr_of_mut!((*(*ent).client).ps.origin),
                9999,
                DAMAGE_NO_PROTECTION,
                MOD_FALLING,
            );
            //player_die(ent, ent, ent, 100000, MOD_FALLING);
            //		if (!ent->NPC)
            //		{
            //			respawn(ent);
            //		}
            //		ent->client->ps.fallingToDeath = 0;

            G_MuteSound((*ent).s.number, CHAN_VOICE); //stop screaming, because you are dead!
        }
    }

    if (*(*ent).client).ps.otherKillerTime > (*addr_of!(level)).time
        && (*(*ent).client).ps.groundEntityNum != ENTITYNUM_NONE
        && (*(*ent).client).ps.otherKillerDebounceTime < (*addr_of!(level)).time
    {
        (*(*ent).client).ps.otherKillerTime = 0;
        (*(*ent).client).ps.otherKiller = ENTITYNUM_NONE;
    } else if (*(*ent).client).ps.otherKillerTime > (*addr_of!(level)).time
        && (*(*ent).client).ps.groundEntityNum == ENTITYNUM_NONE
    {
        if (*(*ent).client).ps.otherKillerDebounceTime < ((*addr_of!(level)).time + 100) {
            (*(*ent).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 100;
        }
    }

    //		WP_ForcePowersUpdate( ent, ucmd); //update any active force powers
    //		WP_SaberPositionUpdate(ent, ucmd); //check the server-side saber point, do apprioriate server-side actions (effects are cs-only)
    //		WP_SaberStartMissileBlockCheck(ent, ucmd);

    //NOTE: can't put USE here *before* PMove!!
    if (*(*ent).client).ps.useDelay > (*addr_of!(level)).time
        && (*(*ent).client).ps.m_iVehicleNum != 0
    {
        //when in a vehicle, debounce the use...
        (*ucmd).buttons &= !BUTTON_USE;
    }

    //FIXME: need to do this before check to avoid walls and cliffs (or just cliffs?)
    G_AddPushVecToUcmd(ent, ucmd);

    //play/stop any looping sounds tied to controlled movement
    G_CheckMovingLoopingSounds(ent, ucmd);

    pm.ps = &mut (*client).ps;
    pm.cmd = *ucmd;
    if (*pm.ps).pm_type == PM_DEAD {
        pm.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY;
    } else if (*ent).r.svFlags & SVF_BOT != 0 {
        pm.tracemask = MASK_PLAYERSOLID | CONTENTS_MONSTERCLIP;
    } else {
        pm.tracemask = MASK_PLAYERSOLID;
    }
    pm.trace = Some(trap_Trace_pm);
    pm.pointcontents = Some(trap_PointContents_pm);
    pm.debugLevel = (*addr_of!(g_debugMove)).integer;
    pm.noFootsteps = (((*addr_of!(g_dmflags)).integer & DF_NO_FOOTSTEPS) > 0) as qboolean;

    pm.pmove_fixed = (*addr_of!(pmove_fixed)).integer | (*client).pers.pmoveFixed;
    pm.pmove_msec = (*addr_of!(pmove_msec)).integer;

    pm.animations = (*addr_of!(bgAllAnims))[(*ent).localAnimIndex as usize].anims; //NULL;

    //rww - bgghoul2
    pm.ghoul2 = core::ptr::null_mut();

    // #ifdef _DEBUG g_disableServerG2 path omitted (debug build only)
    if !(*ent).ghoul2.is_null() {
        if (*ent).localAnimIndex > 1 {
            //if it isn't humanoid then we will be having none of this.
            pm.ghoul2 = core::ptr::null_mut();
        } else {
            pm.ghoul2 = (*ent).ghoul2;
            pm.g2Bolts_LFoot = crate::trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_leg_foot");
            pm.g2Bolts_RFoot = crate::trap::G2API_AddBolt((*ent).ghoul2, 0, "*r_leg_foot");
        }
    }

    //point the saber data to the right place
    // (#if 0 pm.saber[] wiring omitted upstream)

    //I'll just do this every frame in case the scale changes in realtime (don't need to update the g2 inst for that)
    VectorCopy(&(*ent).modelScale, &mut pm.modelScale);
    //rww end bgghoul2

    pm.gametype = (*addr_of!(g_gametype)).integer;
    pm.debugMelee = (*addr_of!(g_debugMelee)).integer;
    pm.stepSlideFix = (*addr_of!(g_stepSlideFix)).integer;

    pm.noSpecMove = (*addr_of!(g_noSpecMove)).integer;

    pm.nonHumanoid = ((*ent).localAnimIndex > 0) as qboolean;

    VectorCopy(&(*client).ps.origin, &mut (*client).oldOrigin);

    /* g_singlePlayer intermission-queue block omitted (compiled out upstream) */

    //Set up bg entity data
    pm.baseEnt = core::ptr::addr_of_mut!(g_entities).cast::<bgEntity_t>();
    pm.entSize = core::mem::size_of::<gentity_t>() as c_int;

    if (*(*ent).client).ps.saberLockTime > (*addr_of!(level)).time {
        let blockOpp: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.saberLockEnemy as usize);

        if !blockOpp.is_null() && (*blockOpp).inuse != QFALSE && !(*blockOpp).client.is_null() {
            let mut lockDir: vec3_t = [0.0; 3];
            let mut lockAng: vec3_t = [0.0; 3];

            //VectorClear( ent->client->ps.velocity );
            VectorSubtract(
                &(*blockOpp).r.currentOrigin,
                &(*ent).r.currentOrigin,
                &mut lockDir,
            );
            //lockAng[YAW] = vectoyaw( defDir );
            vectoangles(&lockDir, &mut lockAng);
            SetClientViewAngle(ent, &lockAng);
        }

        if (*(*ent).client).ps.saberLockHitCheckTime < (*addr_of!(level)).time {
            //have moved to next frame since last lock push
            (*(*ent).client).ps.saberLockHitCheckTime = (*addr_of!(level)).time; //so we don't push more than once per server frame
            if ((*(*ent).client).buttons & BUTTON_ATTACK) != 0
                && ((*(*ent).client).oldbuttons & BUTTON_ATTACK) == 0
            {
                if (*(*ent).client).ps.saberLockHitIncrementTime < (*addr_of!(level)).time {
                    //have moved to next frame since last saberlock attack button press
                    let mut lockHits: c_int = 0;
                    (*(*ent).client).ps.saberLockHitIncrementTime = (*addr_of!(level)).time; //so we don't register an attack key press more than once per server frame
                    //NOTE: FP_SABER_OFFENSE level already taken into account in PM_SaberLocked
                    if (*(*ent).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                        //raging: push harder
                        lockHits = 1 + (*(*ent).client).ps.fd.forcePowerLevel[FP_RAGE as usize];
                    } else {
                        //normal attack
                        match (*(*ent).client).ps.fd.saberAnimLevel {
                            x if x == SS_FAST => {
                                lockHits = 1;
                            }
                            x if x == SS_MEDIUM
                                || x == SS_TAVION
                                || x == SS_DUAL
                                || x == SS_STAFF =>
                            {
                                lockHits = 2;
                            }
                            x if x == SS_STRONG || x == SS_DESANN => {
                                lockHits = 3;
                            }
                            _ => {}
                        }
                    }
                    if (*(*ent).client).ps.fd.forceRageRecoveryTime > (*addr_of!(level)).time
                        && Q_irand(0, 1) != 0
                    {
                        //finished raging: weak
                        lockHits -= 1;
                    }
                    lockHits += (*(*ent).client).saber[0].lockBonus;
                    if (*(*ent).client).saber[1].model[0] != 0
                        && (*(*ent).client).ps.saberHolstered == 0
                    {
                        lockHits += (*(*ent).client).saber[1].lockBonus;
                    }
                    (*(*ent).client).ps.saberLockHits += lockHits;
                    if (*addr_of!(g_saberLockRandomNess)).integer != 0 {
                        (*(*ent).client).ps.saberLockHits +=
                            Q_irand(0, (*addr_of!(g_saberLockRandomNess)).integer);
                        if (*(*ent).client).ps.saberLockHits < 0 {
                            (*(*ent).client).ps.saberLockHits = 0;
                        }
                    }
                }
            }
            if (*(*ent).client).ps.saberLockHits > 0 {
                if (*(*ent).client).ps.saberLockAdvance == 0 {
                    (*(*ent).client).ps.saberLockHits -= 1;
                }
                (*(*ent).client).ps.saberLockAdvance = QTRUE;
            }
        }
    } else {
        (*(*ent).client).ps.saberLockFrame = 0;
        //check for taunt
        if (pm.cmd.generic_cmd as c_int == GENCMD_ENGAGE_DUEL)
            && ((*addr_of!(g_gametype)).integer == GT_DUEL
                || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        {
            //already in a duel, make it a taunt command
            pm.cmd.buttons |= BUTTON_GESTURE;
        }
    }

    if (*ent).s.number >= MAX_CLIENTS as c_int {
        VectorCopy(&(*ent).r.mins, &mut pm.mins);
        VectorCopy(&(*ent).r.maxs, &mut pm.maxs);
        if (*ent).s.NPC_class == CLASS_VEHICLE && !(*ent).m_pVehicle.is_null() {
            if !(*(*ent).m_pVehicle).m_pPilot.is_null() {
                //vehicles want to use their last pilot ucmd I guess
                if ((*addr_of!(level)).time - (*(*ent).m_pVehicle).m_ucmd.serverTime) > 2000 {
                    //Previous owner disconnected, maybe
                    (*(*ent).m_pVehicle).m_ucmd.serverTime = (*addr_of!(level)).time;
                    (*(*ent).client).ps.commandTime = (*addr_of!(level)).time - 100;
                    msec = 100;
                }

                core::ptr::copy_nonoverlapping(
                    &(*(*ent).m_pVehicle).m_ucmd as *const usercmd_t,
                    &mut pm.cmd as *mut usercmd_t,
                    1,
                );

                //no veh can strafe
                pm.cmd.rightmove = 0;
                //no crouching or jumping!
                pm.cmd.upmove = 0;

                //NOTE: button presses were getting lost!
                pm.cmd.buttons = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*(*ent).m_pVehicle).m_pPilot).s.number as usize))
                .client)
                    .pers
                    .cmd
                    .buttons
                    & (BUTTON_ATTACK | BUTTON_ALT_ATTACK);
            }
            if (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER {
                if (*(*ent).client).ps.groundEntityNum != ENTITYNUM_NONE {
                    //ATST crushes anything underneath it
                    let under: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*ent).client).ps.groundEntityNum as usize);
                    if !under.is_null() && (*under).health != 0 && (*under).takedamage != QFALSE {
                        let mut down: vec3_t = [0.0, 0.0, -1.0];
                        //FIXME: we'll be doing traces down from each foot, so we'll have a real impact origin
                        G_Damage(
                            under,
                            ent,
                            ent,
                            addr_of_mut!(down),
                            addr_of_mut!((*under).r.currentOrigin),
                            100,
                            0,
                            MOD_CRUSH,
                        );
                    }
                }
            }
        }
    }

    Pmove(&mut pm);

    if (*(*ent).client).solidHack != 0 {
        if (*(*ent).client).solidHack > (*addr_of!(level)).time {
            //whee!
            (*ent).r.contents = 0;
        } else {
            (*ent).r.contents = CONTENTS_BODY;
            (*(*ent).client).solidHack = 0;
        }
    }

    if !(*ent).NPC.is_null() {
        VectorCopy(&(*(*ent).client).ps.viewangles, &mut (*ent).r.currentAngles);
    }

    if pm.checkDuelLoss != 0 {
        if pm.checkDuelLoss > 0
            && (pm.checkDuelLoss <= MAX_CLIENTS as c_int
                || (pm.checkDuelLoss < (MAX_GENTITIES as c_int - 1)
                    && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((pm.checkDuelLoss - 1) as usize)).s.eType
                        == ET_NPC))
        {
            let clientLost: *mut gentity_t =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((pm.checkDuelLoss - 1) as usize);

            if !clientLost.is_null()
                && (*clientLost).inuse != QFALSE
                && !(*clientLost).client.is_null()
                && Q_irand(0, 40) > (*clientLost).health
            {
                let mut attDir: vec3_t = [0.0; 3];
                VectorSubtract(
                    &(*(*ent).client).ps.origin,
                    &(*(*clientLost).client).ps.origin,
                    &mut attDir,
                );
                VectorNormalize(&mut attDir);

                VectorClear(&mut (*(*clientLost).client).ps.velocity);
                (*(*clientLost).client).ps.forceHandExtend = HANDEXTEND_NONE;
                (*(*clientLost).client).ps.forceHandExtendTime = 0;

                *addr_of_mut!(gGAvoidDismember) = 1;
                G_Damage(
                    clientLost,
                    ent,
                    ent,
                    addr_of_mut!(attDir),
                    addr_of_mut!((*(*clientLost).client).ps.origin),
                    9999,
                    DAMAGE_NO_PROTECTION,
                    MOD_SABER,
                );

                if (*clientLost).health < 1 {
                    *addr_of_mut!(gGAvoidDismember) = 2;
                    G_CheckForDismemberment(
                        clientLost,
                        ent,
                        &(*(*clientLost).client).ps.origin,
                        999,
                        (*(*clientLost).client).ps.legsAnim,
                        QFALSE,
                    );
                }

                *addr_of_mut!(gGAvoidDismember) = 0;
            } else if !clientLost.is_null()
                && (*clientLost).inuse != QFALSE
                && !(*clientLost).client.is_null()
                && (*(*clientLost).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN
                && (*(*clientLost).client).ps.saberEntityNum != 0
            {
                //if we didn't knock down it was a circle lock. So as punishment, make them lose their saber and go into a proper anim
                saberCheckKnockdown_DuelLoss(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*clientLost).client).ps.saberEntityNum as usize),
                    clientLost,
                    ent,
                );
            }
        }

        pm.checkDuelLoss = 0;
    }

    if pm.cmd.generic_cmd as c_int != 0
        && (pm.cmd.generic_cmd as c_int != (*(*ent).client).lastGenCmd
            || (*(*ent).client).lastGenCmdTime < (*addr_of!(level)).time)
    {
        (*(*ent).client).lastGenCmd = pm.cmd.generic_cmd as c_int;
        if pm.cmd.generic_cmd as c_int != GENCMD_FORCE_THROW
            && pm.cmd.generic_cmd as c_int != GENCMD_FORCE_PULL
        {
            //these are the only two where you wouldn't care about a delay between
            (*(*ent).client).lastGenCmdTime = (*addr_of!(level)).time + 300; //default 100ms debounce between issuing the same command.
        }

        match pm.cmd.generic_cmd as c_int {
            0 => {}
            x if x == GENCMD_SABERSWITCH => {
                Cmd_ToggleSaber_f(ent);
            }
            x if x == GENCMD_ENGAGE_DUEL => {
                if (*addr_of!(g_gametype)).integer == GT_DUEL
                    || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
                {
                    //already in a duel, made it a taunt command
                } else {
                    Cmd_EngageDuel_f(ent);
                }
            }
            x if x == GENCMD_FORCE_HEAL => {
                ForceHeal(ent);
            }
            x if x == GENCMD_FORCE_SPEED => {
                ForceSpeed(ent, 0);
            }
            x if x == GENCMD_FORCE_THROW => {
                ForceThrow(ent, QFALSE);
            }
            x if x == GENCMD_FORCE_PULL => {
                ForceThrow(ent, QTRUE);
            }
            x if x == GENCMD_FORCE_DISTRACT => {
                ForceTelepathy(ent);
            }
            x if x == GENCMD_FORCE_RAGE => {
                ForceRage(ent);
            }
            x if x == GENCMD_FORCE_PROTECT => {
                ForceProtect(ent);
            }
            x if x == GENCMD_FORCE_ABSORB => {
                ForceAbsorb(ent);
            }
            x if x == GENCMD_FORCE_HEALOTHER => {
                ForceTeamHeal(ent);
            }
            x if x == GENCMD_FORCE_FORCEPOWEROTHER => {
                ForceTeamForceReplenish(ent);
            }
            x if x == GENCMD_FORCE_SEEING => {
                ForceSeeing(ent);
            }
            x if x == GENCMD_USE_SEEKER => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SEEKER))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_SEEKER) != 0
                {
                    ItemUse_Seeker(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_SEEKER, 0);
                    (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_SEEKER);
                }
            }
            x if x == GENCMD_USE_FIELD => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SHIELD))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_SHIELD) != 0
                {
                    ItemUse_Shield(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_SHIELD, 0);
                    (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_SHIELD);
                }
            }
            x if x == GENCMD_USE_BACTA => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_MEDPAC))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_MEDPAC) != 0
                {
                    ItemUse_MedPack(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_MEDPAC, 0);
                    (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_MEDPAC);
                }
            }
            x if x == GENCMD_USE_BACTABIG => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize]
                    & (1 << HI_MEDPAC_BIG))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_MEDPAC_BIG) != 0
                {
                    ItemUse_MedPack_Big(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_MEDPAC_BIG, 0);
                    (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &=
                        !(1 << HI_MEDPAC_BIG);
                }
            }
            x if x == GENCMD_USE_ELECTROBINOCULARS => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize]
                    & (1 << HI_BINOCULARS))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_BINOCULARS) != 0
                {
                    ItemUse_Binoculars(ent);
                    if (*(*ent).client).ps.zoomMode == 0 {
                        G_AddEvent(ent, EV_USE_ITEM0 + HI_BINOCULARS, 1);
                    } else {
                        G_AddEvent(ent, EV_USE_ITEM0 + HI_BINOCULARS, 2);
                    }
                }
            }
            x if x == GENCMD_ZOOM => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize]
                    & (1 << HI_BINOCULARS))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_BINOCULARS) != 0
                {
                    ItemUse_Binoculars(ent);
                    if (*(*ent).client).ps.zoomMode == 0 {
                        G_AddEvent(ent, EV_USE_ITEM0 + HI_BINOCULARS, 1);
                    } else {
                        G_AddEvent(ent, EV_USE_ITEM0 + HI_BINOCULARS, 2);
                    }
                }
            }
            x if x == GENCMD_USE_SENTRY => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize]
                    & (1 << HI_SENTRY_GUN))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_SENTRY_GUN) != 0
                {
                    ItemUse_Sentry(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_SENTRY_GUN, 0);
                    (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &=
                        !(1 << HI_SENTRY_GUN);
                }
            }
            x if x == GENCMD_USE_JETPACK => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_JETPACK))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_JETPACK) != 0
                {
                    ItemUse_Jetpack(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_JETPACK, 0);
                    /* zoomMode HI_BINOCULARS event block commented out upstream */
                }
            }
            x if x == GENCMD_USE_HEALTHDISP => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize]
                    & (1 << HI_HEALTHDISP))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_HEALTHDISP) != 0
                {
                    //ItemUse_UseDisp(ent, HI_HEALTHDISP);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_HEALTHDISP, 0);
                }
            }
            x if x == GENCMD_USE_AMMODISP => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_AMMODISP))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_AMMODISP) != 0
                {
                    //ItemUse_UseDisp(ent, HI_AMMODISP);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_AMMODISP, 0);
                }
            }
            x if x == GENCMD_USE_EWEB => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_EWEB)) != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_EWEB) != 0
                {
                    ItemUse_UseEWeb(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 + HI_EWEB, 0);
                }
            }
            x if x == GENCMD_USE_CLOAK => {
                if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_CLOAK))
                    != 0
                    && G_ItemUsable(&mut (*(*ent).client).ps, HI_CLOAK) != 0
                {
                    if (*(*ent).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                        //decloak
                        Jedi_Decloak(ent);
                    } else {
                        //cloak
                        Jedi_Cloak(ent);
                    }
                }
            }
            x if x == GENCMD_SABERATTACKCYCLE => {
                Cmd_SaberAttackCycle_f(ent);
            }
            x if x == GENCMD_TAUNT => {
                G_SetTauntAnim(ent, TAUNT_TAUNT);
            }
            x if x == GENCMD_BOW => {
                G_SetTauntAnim(ent, TAUNT_BOW);
            }
            x if x == GENCMD_MEDITATE => {
                G_SetTauntAnim(ent, TAUNT_MEDITATE);
            }
            x if x == GENCMD_FLOURISH => {
                G_SetTauntAnim(ent, TAUNT_FLOURISH);
            }
            x if x == GENCMD_GLOAT => {
                G_SetTauntAnim(ent, TAUNT_GLOAT);
            }
            _ => {}
        }
    }

    // save results of pmove
    if (*(*ent).client).ps.eventSequence != oldEventSequence {
        (*ent).eventTime = (*addr_of!(level)).time;
    }
    if (*addr_of!(g_smoothClients)).integer != 0 {
        BG_PlayerStateToEntityStateExtraPolate(
            &mut (*(*ent).client).ps,
            &mut (*ent).s,
            (*(*ent).client).ps.commandTime,
            QFALSE,
        );
        //rww - 12-03-02 - Don't snap the origin of players! It screws prediction all up.
    } else {
        BG_PlayerStateToEntityState(&mut (*(*ent).client).ps, &mut (*ent).s, QFALSE);
    }

    if isNPC != QFALSE {
        (*ent).s.eType = ET_NPC;
    }

    SendPendingPredictableEvents(&mut (*(*ent).client).ps);

    if (*(*ent).client).ps.eFlags & EF_FIRING == 0 {
        (*client).fireHeld = QFALSE; // for grapple
    }

    // use the snapped origin for linking so it matches client predicted versions
    VectorCopy(&(*ent).s.pos.trBase, &mut (*ent).r.currentOrigin);

    if (*ent).s.eType != ET_NPC
        || (*ent).s.NPC_class != CLASS_VEHICLE
        || (*ent).m_pVehicle.is_null()
        || (*(*ent).m_pVehicle).m_iRemovedSurfaces == 0
    {
        //let vehicles that are getting broken apart do their own crazy sizing stuff
        VectorCopy(&pm.mins, &mut (*ent).r.mins);
        VectorCopy(&pm.maxs, &mut (*ent).r.maxs);
    }

    (*ent).waterlevel = pm.waterlevel;
    (*ent).watertype = pm.watertype;

    // execute client events
    ClientEvents(ent, oldEventSequence);

    if pm.useEvent != 0 {
        //TODO: Use
        //		TryUse( ent );
    }
    if ((*(*ent).client).pers.cmd.buttons & BUTTON_USE) != 0
        && (*(*ent).client).ps.useDelay < (*addr_of!(level)).time
    {
        TryUse(ent);
        (*(*ent).client).ps.useDelay = (*addr_of!(level)).time + 100;
    }

    // link entity now, after any personal teleporters have been used
    crate::trap::LinkEntity(ent);
    if (*(*ent).client).noclip == 0 {
        G_TouchTriggers(ent);
    }

    // NOTE: now copy the exact origin over otherwise clients can be snapped into solid
    VectorCopy(&(*(*ent).client).ps.origin, &mut (*ent).r.currentOrigin);

    //test for solid areas in the AAS file
    //	BotTestAAS(ent->r.currentOrigin);

    // touch other objects
    ClientImpacts(ent, &mut pm);

    // save results of triggers and client events
    if (*(*ent).client).ps.eventSequence != oldEventSequence {
        (*ent).eventTime = (*addr_of!(level)).time;
    }

    // swap and latch button actions
    (*client).oldbuttons = (*client).buttons;
    (*client).buttons = (*ucmd).buttons;
    (*client).latched_buttons |= (*client).buttons & !(*client).oldbuttons;

    //	G_VehicleAttachDroidUnit( ent );

    // Did we kick someone in our pmove sequence?
    if (*client).ps.forceKickFlip != 0 {
        let faceKicked: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(((*client).ps.forceKickFlip - 1) as usize);

        if !faceKicked.is_null()
            && !(*faceKicked).client.is_null()
            && (OnSameTeam(ent, faceKicked) == QFALSE
                || (*addr_of!(g_friendlyFire)).integer != 0)
            && ((*(*faceKicked).client).ps.duelInProgress == QFALSE
                || (*(*faceKicked).client).ps.duelIndex == (*ent).s.number)
            && ((*(*ent).client).ps.duelInProgress == QFALSE
                || (*(*ent).client).ps.duelIndex == (*faceKicked).s.number)
        {
            if !faceKicked.is_null()
                && !(*faceKicked).client.is_null()
                && (*faceKicked).health != 0
                && (*faceKicked).takedamage != QFALSE
            {
                //push them away and do pain
                let mut oppDir: vec3_t = [0.0; 3];
                let mut strength: c_int =
                    VectorNormalize2(&(*client).ps.velocity, &mut oppDir) as c_int;

                strength = (strength as f32 * 0.05) as c_int;

                VectorScale(&oppDir.clone(), -1.0, &mut oppDir);

                G_Damage(
                    faceKicked,
                    ent,
                    ent,
                    addr_of_mut!(oppDir),
                    addr_of_mut!((*client).ps.origin),
                    strength,
                    DAMAGE_NO_ARMOR,
                    MOD_MELEE,
                );

                if (*(*faceKicked).client).ps.weapon != WP_SABER
                    || (*(*faceKicked).client).ps.fd.saberAnimLevel != FORCE_LEVEL_3
                    || (BG_SaberInAttack((*(*faceKicked).client).ps.saberMove) == QFALSE
                        && PM_SaberInStart((*(*faceKicked).client).ps.saberMove) == QFALSE
                        && PM_SaberInReturn((*(*faceKicked).client).ps.saberMove) == QFALSE
                        && PM_SaberInTransition((*(*faceKicked).client).ps.saberMove) == QFALSE)
                {
                    if (*faceKicked).health > 0
                        && (*(*faceKicked).client).ps.stats[STAT_HEALTH as usize] > 0
                        && (*(*faceKicked).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN
                    {
                        if BG_KnockDownable(&mut (*(*faceKicked).client).ps) != QFALSE
                            && Q_irand(1, 10) <= 3
                        {
                            //only actually knock over sometimes, but always do velocity hit
                            (*(*faceKicked).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                            (*(*faceKicked).client).ps.forceHandExtendTime =
                                (*addr_of!(level)).time + 1100;
                            (*(*faceKicked).client).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                        }

                        (*(*faceKicked).client).ps.otherKiller = (*ent).s.number;
                        (*(*faceKicked).client).ps.otherKillerTime =
                            (*addr_of!(level)).time + 5000;
                        (*(*faceKicked).client).ps.otherKillerDebounceTime =
                            (*addr_of!(level)).time + 100;

                        (*(*faceKicked).client).ps.velocity[0] =
                            oppDir[0] * (strength as f32 * 40.0);
                        (*(*faceKicked).client).ps.velocity[1] =
                            oppDir[1] * (strength as f32 * 40.0);
                        (*(*faceKicked).client).ps.velocity[2] = 200.0;
                    }
                }

                G_Sound(
                    faceKicked,
                    CHAN_AUTO,
                    G_SoundIndex(
                        Sz(va(format_args!(
                            "sound/weapons/melee/punch{}",
                            Q_irand(1, 4)
                        )))
                        .to_string()
                        .as_str(),
                    ),
                );
            }
        }

        (*client).ps.forceKickFlip = 0;
    }

    // check for respawning
    if (*client).ps.stats[STAT_HEALTH as usize] <= 0
        && !((*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0) //can't respawn while being eaten
        && (*ent).s.eType != ET_NPC
    {
        // wait for the attack button to be pressed
        if (*addr_of!(level)).time > (*client).respawnTime && *addr_of!(gDoSlowMoDuel) == QFALSE {
            // forcerespawn is to prevent users from waiting out powerups
            let mut forceRes: c_int = (*addr_of!(g_forcerespawn)).integer;

            if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
                forceRes = 1;
            } else if (*addr_of!(g_gametype)).integer == GT_SIEGE
                && (*addr_of!(g_siegeRespawn)).integer != 0
            {
                //wave respawning on
                forceRes = 1;
            }

            if forceRes > 0
                && ((*addr_of!(level)).time - (*client).respawnTime) > forceRes * 1000
            {
                respawn(ent);
                return;
            }

            // pressing attack or use is the normal respawn method
            if (*ucmd).buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) != 0 {
                respawn(ent);
            }
        } else if *addr_of!(gDoSlowMoDuel) != QFALSE {
            (*client).respawnTime = (*addr_of!(level)).time + 1000;
        }
        return;
    }

    // perform once-a-second actions
    ClientTimerActions(ent, msec);

    G_UpdateClientBroadcasts(ent);

    //try some idle anims on ent if getting no input and not moving for some time
    G_CheckClientIdle(ent, ucmd);

    // This code was moved here from clientThink to fix a problem with g_synchronousClients
    // being set to 1 when in vehicles.
    if (*ent).s.number < MAX_CLIENTS as c_int && (*(*ent).client).ps.m_iVehicleNum != 0 {
        //driving a vehicle
        //run it
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.m_iVehicleNum as usize)).inuse
            != QFALSE
            && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.m_iVehicleNum as usize))
                .client
                .is_null()
        {
            ClientThink(
                (*(*ent).client).ps.m_iVehicleNum,
                &mut (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).client).ps.m_iVehicleNum as usize))
                .m_pVehicle)
                    .m_ucmd,
            );
        } else {
            //vehicle no longer valid?
            (*(*ent).client).ps.m_iVehicleNum = 0;
        }
    }

    // The C++ `ClientManager::ActiveClientNum()` / `FF_XboxSaberRumble()` Xbox
    // force-feedback tail is omitted — Xbox-only, not part of the PC server module.
}

/// `void ClientThink( int clientNum, usercmd_t *ucmd )` (g_active.c:3558). A new command has
/// arrived from the client: fetch it (real clients), stamp the command time, then funnel into
/// [`ClientThink_real`] for non-bot async clients and for the vehicle (>= MAX_CLIENTS) clients
/// even when running synchronously. The two large vehicle-handling comment blocks that were
/// moved into `ClientThink_real` are carried over verbatim as references. No oracle — entity-state
/// dispatcher.
///
/// # Safety
/// `ent`/`g_entities` for `clientNum` must be valid; the `level`/`g_synchronousClients` globals
/// must be initialised.
pub unsafe fn ClientThink(clientNum: c_int, ucmd: *mut usercmd_t) {
    let ent: *mut gentity_t;

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize);
    if clientNum < MAX_CLIENTS as c_int {
        crate::trap::GetUsercmd(clientNum, &mut (*(*ent).client).pers.cmd);
    }

    // mark the time we got info, so we can display the
    // phone jack if they don't get any for a while
    (*(*ent).client).lastCmdTime = (*addr_of!(level)).time;

    if !ucmd.is_null() {
        (*(*ent).client).pers.cmd = *ucmd;
    }

    /* 	This was moved to clientthink_real, but since its sort of a risky change i left it here for
        now as a more concrete reference - BSD
    ... (vehicle pilot-input block, see ClientThink_real) ... */

    if (*ent).r.svFlags & SVF_BOT == 0 && (*addr_of!(g_synchronousClients)).integer == 0 {
        ClientThink_real(ent);
    }
    // vehicles are clients and when running synchronous they still need to think here
    // so special case them.
    else if clientNum >= MAX_CLIENTS as c_int {
        ClientThink_real(ent);
    }

    /*	This was moved to clientthink_real, but since its sort of a risky change i left it here for
        now as a more concrete reference - BSD
    ... (vehicle ClientThink-recursion block, see ClientThink_real) ... */
}

/// `void G_RunClient( gentity_t *ent )` (g_active.c:3632). Per-frame client runner: for bots and
/// synchronous clients, stamp the command time and run [`ClientThink_real`]; async clients return
/// (they are driven by `ClientThink` from the engine usercmd path instead). No oracle — wrapper.
///
/// # Safety
/// `ent` and its `client` must be valid; the `level`/`g_synchronousClients` globals must be
/// initialised.
pub unsafe fn G_RunClient(ent: *mut gentity_t) {
    if (*ent).r.svFlags & SVF_BOT == 0 && (*addr_of!(g_synchronousClients)).integer == 0 {
        return;
    }
    (*(*ent).client).pers.cmd.serverTime = (*addr_of!(level)).time;
    ClientThink_real(ent);
}

/// `void ClientEndFrame( gentity_t *ent )` (g_active.c:3703). Called at the end of each server
/// frame for each connected client: expire powerups, set the `EF_CONNECTION` lag flag, sync
/// `STAT_HEALTH` from `ent->health`, run world effects / damage feedback / client sound, and snap
/// the entity-state from the player-state. Spectators short-circuit into
/// [`SpectatorClientEndFrame`]. No oracle — per-frame entity/client-state finalisation.
///
/// # Safety
/// `ent` and its `client` must be valid; the `level` global must be initialised.
pub unsafe fn ClientEndFrame(ent: *mut gentity_t) {
    let mut i: c_int;
    let _pers: *mut clientPersistant_t;
    let mut isNPC: qboolean = QFALSE;

    if (*ent).s.eType == ET_NPC {
        isNPC = QTRUE;
    }

    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        SpectatorClientEndFrame(ent);
        return;
    }

    _pers = &mut (*(*ent).client).pers;

    // turn off any expired powerups
    i = 0;
    while i < MAX_POWERUPS as c_int {
        if (*(*ent).client).ps.powerups[i as usize] < (*addr_of!(level)).time {
            (*(*ent).client).ps.powerups[i as usize] = 0;
        }
        i += 1;
    }

    // save network bandwidth
    // #if 0 (g_synchronousClients viewangles-clear block omitted upstream)

    //
    // If the end of unit layout is displayed, don't give
    // the player any normal movement attributes
    //
    if (*addr_of!(level)).intermissiontime != 0 {
        if (*ent).s.number < MAX_CLIENTS as c_int
            || (*(*ent).client).NPC_class == CLASS_VEHICLE
        {
            //players and vehicles do nothing in intermissions
            return;
        }
    }

    // burn from lava, etc
    P_WorldEffects(ent);

    // apply all the damage taken this frame
    P_DamageFeedback(ent);

    // add the EF_CONNECTION flag if we haven't gotten commands recently
    if (*addr_of!(level)).time - (*(*ent).client).lastCmdTime > 1000 {
        (*ent).s.eFlags |= EF_CONNECTION;
    } else {
        (*ent).s.eFlags &= !EF_CONNECTION;
    }

    (*(*ent).client).ps.stats[STAT_HEALTH as usize] = (*ent).health; // FIXME: get rid of ent->health...

    G_SetClientSound(ent);

    // set the latest infor
    if (*addr_of!(g_smoothClients)).integer != 0 {
        BG_PlayerStateToEntityStateExtraPolate(
            &mut (*(*ent).client).ps,
            &mut (*ent).s,
            (*(*ent).client).ps.commandTime,
            QFALSE,
        );
        //rww - 12-03-02 - Don't snap the origin of players! It screws prediction all up.
    } else {
        BG_PlayerStateToEntityState(&mut (*(*ent).client).ps, &mut (*ent).s, QFALSE);
    }

    if isNPC != QFALSE {
        (*ent).s.eType = ET_NPC;
    }

    SendPendingPredictableEvents(&mut (*(*ent).client).ps);

    // set the bit for the reachability area the client is currently in
    //	i = trap_AAS_PointReachabilityAreaIndex( ent->client->ps.origin );
    //	ent->client->areabits[i >> 3] |= 1 << (i & 7);
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::anims::{BOTH_STAND1, BOTH_STAND2, BOTH_STAND3, BOTH_STAND4};
    use crate::codemp::game::q_shared_h::{
        BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING,
        BUTTON_FORCEGRIP, BUTTON_FORCEPOWER, BUTTON_GESTURE, BUTTON_USE, BUTTON_USE_HOLDABLE,
    };
    use crate::oracle;

    /// `G_StandingAnim` over the four standing anims plus a spread of negatives — the
    /// standing-idle variants, adjacent enum values, and out-of-range ints — checked
    /// bit-exact against the extracted C.
    #[test]
    fn g_standinganim_matches_oracle() {
        let cases = [
            BOTH_STAND1,
            BOTH_STAND2,
            BOTH_STAND3,
            BOTH_STAND4,
            BOTH_STAND1 - 1,
            BOTH_STAND4 + 1,
            BOTH_STAND1 + 1, // BOTH_STAND1IDLE1 — must be qfalse
            0,
            -1,
            1,
            12345,
        ];
        for &anim in &cases {
            let got = G_StandingAnim(anim);
            let want = unsafe { oracle::jka_G_StandingAnim(anim) };
            assert_eq!(got, want, "anim={anim}");
        }
    }

    /// `G_ActionButtonPressed` over every action bit individually, none, combinations,
    /// and a non-action bit — checked bit-exact against the extracted C.
    #[test]
    fn g_actionbuttonpressed_matches_oracle() {
        let bits = [
            0,
            BUTTON_ATTACK,
            BUTTON_USE_HOLDABLE,
            BUTTON_GESTURE,
            BUTTON_USE,
            BUTTON_FORCEGRIP,
            BUTTON_ALT_ATTACK,
            BUTTON_FORCEPOWER,
            BUTTON_FORCE_LIGHTNING,
            BUTTON_FORCE_DRAIN,
            BUTTON_ATTACK | BUTTON_FORCE_DRAIN,
            BUTTON_GESTURE | BUTTON_USE,
            2,  // BUTTON_WALKING — not an action bit
            16, // non-action bit
            !0, // all bits
        ];
        for &b in &bits {
            let got = G_ActionButtonPressed(b);
            let want = unsafe { oracle::jka_G_ActionButtonPressed(b) };
            assert_eq!(got, want, "buttons={b}");
        }
    }

    /// `ClientTimerActions` over a spread of `(msec, timeResidual, health, max, armor)`
    /// inputs — sub-second accumulation, multi-second drains, over/under-max health &
    /// armor — checked bit-exact (timeResidual/health/armor outputs) against the C.
    #[test]
    fn clienttimeractions_matches_oracle() {
        use crate::codemp::game::bg_public::{STAT_ARMOR, STAT_MAX_HEALTH};
        use crate::codemp::game::g_local::{gclient_t, gentity_t};

        // (msec, timeResidual, health, max_health, armor)
        let cases: &[(c_int, c_int, c_int, c_int, c_int)] = &[
            (100, 0, 100, 100, 0),      // sub-second: no whole tick
            (500, 600, 100, 100, 0),    // crosses one second
            (3000, 200, 250, 100, 250), // multiple seconds, both over max
            (1000, 0, 100, 100, 50),    // exactly at max: no drain
            (1000, 0, 80, 100, 0),      // health under max
            (2500, 0, 150, 100, 120),   // 2 ticks, both drain twice
            (0, 999, 100, 100, 0),      // no msec, residual just under
            (1, 999, 200, 100, 200),    // tips over to one tick
        ];

        for &(msec, residual, health, max, armor) in cases {
            // Rust side: build a gentity + gclient and wire them together.
            unsafe {
                let mut client: gclient_t = core::mem::zeroed();
                let mut ent: gentity_t = core::mem::zeroed();
                client.timeResidual = residual;
                client.ps.stats[STAT_MAX_HEALTH as usize] = max;
                client.ps.stats[STAT_ARMOR as usize] = armor;
                ent.health = health;
                ent.client = &mut client;
                ClientTimerActions(&mut ent, msec);

                // C side: scalar mirror.
                let mut c_residual = residual;
                let mut c_health = health;
                let mut c_armor = armor;
                oracle::jka_ClientTimerActions(
                    msec,
                    &mut c_residual,
                    &mut c_health,
                    max,
                    &mut c_armor,
                );

                assert_eq!(client.timeResidual, c_residual, "timeResidual {msec},{residual}");
                assert_eq!(ent.health, c_health, "health {msec},{residual}");
                assert_eq!(
                    client.ps.stats[STAT_ARMOR as usize],
                    c_armor,
                    "armor {msec},{residual}"
                );
            }
        }
    }

    /// `G_AddPushVecToUcmd` over a spread of view angles / move commands / speeds / push
    /// vectors and the pushVecTime-vs-level.time clear branch — bit-exact (speed,
    /// forwardmove, rightmove, and the possibly-cleared pushVec) against the extracted C
    /// (which links the real AngleVectors/VectorNormalize/VectorLengthSquared).
    #[test]
    fn g_addpushvectoucmd_matches_oracle() {
        use core::ptr::addr_of_mut;

        use crate::codemp::game::g_local::{gclient_t, gentity_t};
        use crate::codemp::game::g_main::level;
        use crate::codemp::game::q_shared_h::{usercmd_t, vec3_t};

        // (viewangles, forwardmove, rightmove, speed, pushVec, pushVecTime, levelTime)
        struct Case {
            view: vec3_t,
            fwd: i8,
            rgt: i8,
            speed: f32,
            push: vec3_t,
            push_time: c_int,
            level_time: c_int,
        }
        let cases = [
            // no push: early return (pushVec stays, nothing changes)
            Case { view: [0.0, 0.0, 0.0], fwd: 127, rgt: 0, speed: 250.0,
                   push: [0.0, 0.0, 0.0], push_time: 0, level_time: 0 },
            // forward push, time not expired -> pushVec retained
            Case { view: [0.0, 90.0, 0.0], fwd: 127, rgt: 0, speed: 250.0,
                   push: [100.0, 0.0, 0.0], push_time: 5000, level_time: 1000 },
            // diagonal command + push, time expired -> pushVec cleared
            Case { view: [10.0, 45.0, 0.0], fwd: 64, rgt: -64, speed: 200.0,
                   push: [50.0, -30.0, 10.0], push_time: 500, level_time: 1000 },
            // backward, pushVecTime == levelTime (not <, so retained)
            Case { view: [-15.0, 180.0, 0.0], fwd: -127, rgt: 0, speed: 300.0,
                   push: [0.0, 200.0, 0.0], push_time: 2000, level_time: 2000 },
            // zero move command but active push
            Case { view: [0.0, 0.0, 0.0], fwd: 0, rgt: 0, speed: 150.0,
                   push: [0.0, 0.0, 100.0], push_time: 0, level_time: 5000 },
        ];

        for (i, c) in cases.iter().enumerate() {
            unsafe {
                // Rust side.
                let mut client: gclient_t = core::mem::zeroed();
                let mut ent: gentity_t = core::mem::zeroed();
                let mut ucmd: usercmd_t = core::mem::zeroed();
                client.ps.viewangles = c.view;
                client.ps.speed = c.speed;
                client.pushVec = c.push;
                client.pushVecTime = c.push_time;
                ent.client = &mut client;
                ucmd.forwardmove = c.fwd;
                ucmd.rightmove = c.rgt;
                (*addr_of_mut!(level)).time = c.level_time;
                G_AddPushVecToUcmd(&mut ent, &mut ucmd);

                // C side: scalar/vector mirror.
                let mut c_push = c.push;
                let mut c_speed = c.speed;
                let mut c_fwd = c.fwd;
                let mut c_rgt = c.rgt;
                let c_view = c.view;
                oracle::jka_G_AddPushVecToUcmd(
                    c_push.as_mut_ptr(),
                    c_view.as_ptr(),
                    &mut c_speed,
                    &mut c_fwd,
                    &mut c_rgt,
                    c.push_time,
                    c.level_time,
                );

                assert_eq!(client.ps.speed.to_bits(), c_speed.to_bits(), "speed case {i}");
                assert_eq!(ucmd.forwardmove, c_fwd, "forwardmove case {i}");
                assert_eq!(ucmd.rightmove, c_rgt, "rightmove case {i}");
                for k in 0..3 {
                    assert_eq!(
                        client.pushVec[k].to_bits(),
                        c_push[k].to_bits(),
                        "pushVec[{k}] case {i}"
                    );
                }
            }
        }
    }
}
