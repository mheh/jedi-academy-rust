//! Port of `g_misc.c` — assorted `info_*`/`misc_*`/`target_*` point entities: positional
//! targets, switchable lights, decorative models, the screen-shake trigger, and the
//! `TeleportPlayer` relocation helper that backs the teleporter classes.
//!
//! Each class is an `SP_*` spawner (an `unsafe extern "C" fn(*mut gentity_t)` matching the
//! still-gated `G_CallSpawn` registry's `void (*spawn)(gentity_t*)` slot) plus any
//! `use`/`think` callbacks it installs. Landed incrementally: only the classes whose full
//! dep-set is already ported. Not yet ported — the Ghoul2 g2animents, the holocron/ammo/health
//! dispensers, and `misc_portal_*`. All callbacks here are No-oracle (engine-syscall /
//! global level/entity plumbing).

#![allow(non_snake_case)] // C function names (`SP_info_camp`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::bg_lib::atof;
use crate::codemp::game::bg_misc::forcePowerDarkLight;
use crate::codemp::game::bg_misc::BG_EvaluateTrajectory;
use crate::codemp::game::bg_misc::BG_FindItemForWeapon;
use crate::codemp::game::bg_misc::BG_PlayerStateToEntityState;
use crate::codemp::game::bg_public::EF_DOUBLE_AMMO;
use crate::codemp::game::bg_public::EF_NODRAW;
use crate::codemp::game::bg_public::EV_FIRE_WEAPON;
use crate::codemp::game::bg_public::MASK_SHOT;
use crate::codemp::game::bg_public::{
    CS_LIGHT_STYLES, CS_SKYBOXORG, CS_TERRAINS, EF_PERMANENT, EF_TELEPORT_BIT, ET_GENERAL, ET_NPC,
    ET_PORTAL, ET_TERRAIN, EV_PLAYER_TELEPORT_IN, EV_PLAYER_TELEPORT_OUT, GT_SIEGE,
    GT_SINGLE_PLAYER, PMF_FOLLOW, PMF_TIME_KNOCKBACK, STAT_ARMOR, STAT_MAX_HEALTH, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_public::{
    DEFAULT_MAXS_2, DEFAULT_MINS_2, EF_CLIENTSMOOTH, EF_RAG, ET_HOLOCRON, GT_CTF, GT_CTY,
    GT_HOLOCRON, MASK_PLAYERSOLID,
};
use crate::codemp::game::bg_public::{
    EF_RADAROBJECT, ET_FX, EV_BMODEL_SOUND, FX_STATE_CONTINUOUS, FX_STATE_OFF, FX_STATE_ONE_SHOT,
    FX_STATE_ONE_SHOT_LIMIT, MASK_SOLID, MOD_UNKNOWN,
};
use crate::codemp::game::bg_public::{EV_ITEM_PICKUP, EV_NOAMMO};
use crate::codemp::game::bg_saga::{bgSiegeClasses, WPTable};
use crate::codemp::game::bg_weapons::ammoData;
use crate::codemp::game::bg_weapons_h::{AMMO_BLASTER, AMMO_MAX, AMMO_ROCKETS};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_NONE};
use crate::codemp::game::g_client::SetClientViewAngle;
use crate::codemp::game::g_combat::{AddScore, G_RadiusDamage};
use crate::codemp::game::g_exphysics::G_RunExPhys;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, reference_tag_t, DAMAGEREDIRECT_HEAD, DAMAGEREDIRECT_LLEG,
    DAMAGEREDIRECT_RLEG, FL_BOUNCE_HALF, FL_INACTIVE, FL_SHIELDED, FRAMETIME, MAX_REFNAME,
    START_TIME_FIND_LINKS, START_TIME_LINK_ENTS,
};
use crate::codemp::game::g_main::{
    g_MaxHolocronCarry, g_RMG, g_entities, g_gametype, level, Com_Error, Com_Printf, G_Printf,
    LogExit,
};
use crate::codemp::game::g_mover::{G_FindDoorTrigger, BMS_END, BMS_MID, BMS_START};
use crate::codemp::game::g_object::G_RunObject;
use crate::codemp::game::g_public_h::{
    BSET_USE, SVF_BROADCAST, SVF_NOCLIENT, SVF_PLAYER_USABLE, SVF_PORTAL, SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString, G_SpawnVector};
use crate::codemp::game::g_utils::{
    vtos, G_AddEvent, G_EffectIndex, G_EntitySound, G_Find, G_FreeEntity, G_IconIndex, G_KillBox,
    G_ModelIndex, G_PickTarget, G_ScreenShake, G_SetAngles, G_SetMovedir, G_SetOrigin, G_Sound,
    G_SoundIndex, G_SoundSetIndex, G_Spawn, G_TempEntity, G_UseTargets, G_UseTargets2,
};
use crate::codemp::game::g_weapon::{FireWeapon, WP_FireBlasterMissile};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleVectors, CrossProduct, DirToByte, PerpendicularVector,
    VectorClear, VectorCopy, VectorMA, VectorNormalize, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{
    crandom, random, va, GetIDForString, Info_SetValueForKey, Q_stricmp, Q_strlwr, Q_strncpyz,
};
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, BUTTON_USE, CHAN_AUTO, CHAN_VOICE, ENTITYNUM_NONE, ENTITYNUM_WORLD, ERR_DROP,
    FORCE_DARKSIDE, FORCE_LIGHTSIDE, FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE,
    FP_SABER_OFFENSE, MAX_CLIENTS, MAX_GENTITIES, MAX_INFO_STRING, M_PI, NUM_FORCE_POWERS,
    TR_GRAVITY,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_CORPSE, CONTENTS_SOLID, CONTENTS_TERRAIN, CONTENTS_TRIGGER,
};
use crate::codemp::game::w_saber::HasSetSaberOnly;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `#define HOLOCRON_RESPAWN_TIME 30000` (g_misc.c:10).
const HOLOCRON_RESPAWN_TIME: c_int = 30000;

/// `void HolocronRespawn( gentity_t *self )` (g_misc.c:760). Makes the holocron's icon
/// visible again (the `count - 128` encodes the force-power model index, mirroring
/// `SP_misc_holocron`).
unsafe fn HolocronRespawn(self_: *mut gentity_t) {
    (*self_).s.modelindex = (*self_).count - 128;
}

/// `void HolocronPopOut( gentity_t *self )` (g_misc.c:765). Gives the holocron a random
/// horizontal velocity (±) and an upward kick so it pops away from a dropping carrier.
unsafe fn HolocronPopOut(self_: *mut gentity_t) {
    if Q_irand(1, 10) < 5 {
        (*self_).s.pos.trDelta[0] = 150.0 + Q_irand(1, 100) as f32;
    } else {
        (*self_).s.pos.trDelta[0] = -150.0 - Q_irand(1, 100) as f32;
    }
    if Q_irand(1, 10) < 5 {
        (*self_).s.pos.trDelta[1] = 150.0 + Q_irand(1, 100) as f32;
    } else {
        (*self_).s.pos.trDelta[1] = -150.0 - Q_irand(1, 100) as f32;
    }
    (*self_).s.pos.trDelta[2] = 150.0 + Q_irand(1, 100) as f32;
}

/// `void HolocronTouch( gentity_t *self, gentity_t *other, trace_t *trace )` (g_misc.c:786).
/// Player pickup callback (installed by `SP_misc_holocron` as `ent->touch`): validates the
/// toucher, bails if the holocron is hidden/carried/on-cooldown, tracks the oldest carried
/// power for max-carry eviction (`g_MaxHolocronCarry`), auto-selects this power if the player
/// isn't using their selected one, then records the pickup and marks the holocron carried.
/// The two saber-eviction blocks are carried as comments verbatim (disabled in the C — saber
/// attack is now always force-level 1 via holocron). No oracle (entity/player-state + events).
///
/// # Safety
/// `self_`/`other` must be valid `gentity_t`; `trace` may be null.
pub unsafe extern "C" fn HolocronTouch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    let mut i: c_int = 0;
    let mut othercarrying: c_int = 0;
    let mut time_lowest: f32 = 0.0;
    let mut index_lowest: c_int = -1;
    let mut hasall: c_int = 1;
    let forceReselect: c_int = WP_NONE;

    if !trace.is_null() {
        (*self_).s.groundEntityNum = (*trace).entityNum as c_int;
    }

    if other.is_null() || (*other).client.is_null() || (*other).health < 1 {
        return;
    }

    if (*self_).s.modelindex == 0 {
        return;
    }

    if !(*self_).enemy.is_null() {
        return;
    }

    if (*(*other).client).ps.holocronsCarried[(*self_).count as usize] != 0.0 {
        return;
    }

    if (*(*other).client).ps.holocronCantTouch == (*self_).s.number
        && (*(*other).client).ps.holocronCantTouchTime > (*addr_of!(level)).time as f32
    {
        return;
    }

    while i < NUM_FORCE_POWERS as c_int {
        if (*(*other).client).ps.holocronsCarried[i as usize] != 0.0 {
            othercarrying += 1;

            if index_lowest == -1
                || (*(*other).client).ps.holocronsCarried[i as usize] < time_lowest
            {
                index_lowest = i;
                time_lowest = (*(*other).client).ps.holocronsCarried[i as usize];
            }
        } else if i != (*self_).count {
            hasall = 0;
        }
        i += 1;
    }

    if hasall != 0 {
        //once we pick up this holocron we'll have all of them, so give us super special best prize!
        //G_Printf("You deserve a pat on the back.\n");
    }

    if ((*(*other).client).ps.fd.forcePowersActive
        & (1 << (*(*other).client).ps.fd.forcePowerSelected))
        == 0
    {
        //If the player isn't using his currently selected force power, select this one
        if (*self_).count != FP_SABER_OFFENSE
            && (*self_).count != FP_SABER_DEFENSE
            && (*self_).count != FP_SABERTHROW
            && (*self_).count != FP_LEVITATION
        {
            (*(*other).client).ps.fd.forcePowerSelected = (*self_).count;
        }
    }

    if (*addr_of!(g_MaxHolocronCarry)).integer != 0
        && othercarrying >= (*addr_of!(g_MaxHolocronCarry)).integer
    {
        //make the oldest holocron carried by the player pop out to make room for this one
        (*(*other).client).ps.holocronsCarried[index_lowest as usize] = 0.0;

        /*
        if (index_lowest == FP_SABER_OFFENSE && !HasSetSaberOnly())
        { //you lost your saberattack holocron, so no more saber for you
            other->client->ps.stats[STAT_WEAPONS] |= (1 << WP_STUN_BATON);
            other->client->ps.stats[STAT_WEAPONS] &= ~(1 << WP_SABER);

            if (other->client->ps.weapon == WP_SABER)
            {
                forceReselect = WP_SABER;
            }
        }
        */
        //NOTE: No longer valid as we are now always giving a force level 1 saber attack level in holocron
    }

    //G_Sound(other, CHAN_AUTO, G_SoundIndex("sound/weapons/w_pkup.wav"));
    G_AddEvent(other, EV_ITEM_PICKUP, (*self_).s.number);

    (*(*other).client).ps.holocronsCarried[(*self_).count as usize] =
        (*addr_of!(level)).time as f32;
    (*self_).s.modelindex = 0;
    (*self_).enemy = other;

    (*self_).pos2[0] = 1.0;
    (*self_).pos2[1] = ((*addr_of!(level)).time + HOLOCRON_RESPAWN_TIME) as f32;

    /*
    if (self->count == FP_SABER_OFFENSE && !HasSetSaberOnly())
    { //player gets a saber
        other->client->ps.stats[STAT_WEAPONS] |= (1 << WP_SABER);
        other->client->ps.stats[STAT_WEAPONS] &= ~(1 << WP_STUN_BATON);

        if (other->client->ps.weapon == WP_STUN_BATON)
        {
            forceReselect = WP_STUN_BATON;
        }
    }
    */

    if forceReselect != WP_NONE {
        G_AddEvent(other, EV_NOAMMO, forceReselect);
    }

    //G_Printf("DON'T TOUCH ME\n");
}

/// `void HolocronThink( gentity_t *ent )` (g_misc.c:907). Per-frame callback (installed as
/// `ent->think`): if the carrier died/disconnected/fell-to-death it pops the holocron out at
/// the carrier's origin (and applies a no-touch cooldown so it doesn't re-stick), refreshes
/// the respawn timer while carried, and once it has sat out of place for
/// `HOLOCRON_RESPAWN_TIME` it returns to its spawn spot; finally runs object physics if it has
/// any velocity. The C `goto justthink` is modeled as a labeled-block `break 'justthink`.
/// No oracle (entity/player-state + `G_RunObject`/`trap_LinkEntity`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn HolocronThink(ent: *mut gentity_t) {
    'justthink: {
        if (*ent).pos2[0] != 0.0
            && ((*ent).enemy.is_null()
                || (*(*ent).enemy).client.is_null()
                || (*(*ent).enemy).health < 1)
        {
            if !(*ent).enemy.is_null() && !(*(*ent).enemy).client.is_null() {
                HolocronRespawn(ent);
                let org = (*(*(*ent).enemy).client).ps.origin;
                VectorCopy(&org, &mut (*ent).s.pos.trBase);
                VectorCopy(&org, &mut (*ent).s.origin);
                VectorCopy(&org, &mut (*ent).r.currentOrigin);
                //copy to person carrying's origin before popping out of them
                HolocronPopOut(ent);
                (*(*(*ent).enemy).client).ps.holocronsCarried[(*ent).count as usize] = 0.0;
                (*ent).enemy = core::ptr::null_mut();

                break 'justthink;
            }
        } else if (*ent).pos2[0] != 0.0
            && !(*ent).enemy.is_null()
            && !(*(*ent).enemy).client.is_null()
        {
            (*ent).pos2[1] = ((*addr_of!(level)).time + HOLOCRON_RESPAWN_TIME) as f32;
        }

        if !(*ent).enemy.is_null() && !(*(*ent).enemy).client.is_null() {
            if (*(*(*ent).enemy).client).ps.holocronsCarried[(*ent).count as usize] == 0.0 {
                (*(*(*ent).enemy).client).ps.holocronCantTouch = (*ent).s.number;
                (*(*(*ent).enemy).client).ps.holocronCantTouchTime =
                    ((*addr_of!(level)).time + 5000) as f32;

                HolocronRespawn(ent);
                let org = (*(*(*ent).enemy).client).ps.origin;
                VectorCopy(&org, &mut (*ent).s.pos.trBase);
                VectorCopy(&org, &mut (*ent).s.origin);
                VectorCopy(&org, &mut (*ent).r.currentOrigin);
                //copy to person carrying's origin before popping out of them
                HolocronPopOut(ent);
                (*ent).enemy = core::ptr::null_mut();

                break 'justthink;
            }

            if (*(*ent).enemy).inuse == 0
                || (!(*(*ent).enemy).client.is_null()
                    && (*(*(*ent).enemy).client).ps.fallingToDeath != 0)
            {
                if (*(*ent).enemy).inuse != 0 && !(*(*ent).enemy).client.is_null() {
                    (*(*(*ent).enemy).client).ps.holocronBits &= !(1 << (*ent).count);
                    (*(*(*ent).enemy).client).ps.holocronsCarried[(*ent).count as usize] = 0.0;
                }
                (*ent).enemy = core::ptr::null_mut();
                HolocronRespawn(ent);
                let org2 = (*ent).s.origin2;
                VectorCopy(&org2, &mut (*ent).s.pos.trBase);
                VectorCopy(&org2, &mut (*ent).s.origin);
                VectorCopy(&org2, &mut (*ent).r.currentOrigin);

                (*ent).s.pos.trTime = (*addr_of!(level)).time;

                (*ent).pos2[0] = 0.0;

                trap::LinkEntity(ent);

                break 'justthink;
            }
        }

        if (*ent).pos2[0] != 0.0 && (*ent).pos2[1] < (*addr_of!(level)).time as f32 {
            //isn't in original place and has been there for (HOLOCRON_RESPAWN_TIME) seconds without being picked up, so respawn
            let org2 = (*ent).s.origin2;
            VectorCopy(&org2, &mut (*ent).s.pos.trBase);
            VectorCopy(&org2, &mut (*ent).s.origin);
            VectorCopy(&org2, &mut (*ent).r.currentOrigin);

            (*ent).s.pos.trTime = (*addr_of!(level)).time;

            (*ent).pos2[0] = 0.0;

            trap::LinkEntity(ent);
        }
    }

    // justthink:
    (*ent).nextthink = (*addr_of!(level)).time + 50;

    if (*ent).s.pos.trDelta[0] != 0.0
        || (*ent).s.pos.trDelta[1] != 0.0
        || (*ent).s.pos.trDelta[2] != 0.0
    {
        G_RunObject(ent);
    }
}

// Native libc `atoi` — the non-`vm` build resolves the C `atoi((char*)...)` calls in
// `SP_terrain` against libc (the bg_lib Rust `atoi` is gated behind the `vm` feature).
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

/// `void SP_misc_holocron( gentity_t *ent )` (g_misc.c:762). Spawn function for a
/// `misc_holocron` force-power pickup (Holocron gametype only). Drops the holocron to the
/// floor via a downward trace, frees it when out of place (non-holocron gametype, or a saber
/// holocron in saber-only mode, or starting in solid), clamps its `count` into the valid
/// force-power range, then wires it up as a bouncing physics trigger that displays the
/// dark/light/neutral icon for its power and installs the `HolocronTouch`/`HolocronThink`
/// callbacks. The retail PC oracle has no `assert( 0 )` here (that was Xbox-build residue)
/// and actively sets `ent->s.isJediMaster = qtrue`. No oracle test (entity-state spawn:
/// `trap_Trace`/`G_FreeEntity`/`G_SetOrigin`/`trap_LinkEntity` side-effects).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_holocron(ent: *mut gentity_t) {
    let mut dest: vec3_t = [0.0; 3];

    if (*addr_of!(g_gametype)).integer != GT_HOLOCRON {
        G_FreeEntity(ent);
        return;
    }

    if HasSetSaberOnly() == QTRUE
        && ((*ent).count == FP_SABER_OFFENSE
            || (*ent).count == FP_SABER_DEFENSE
            || (*ent).count == FP_SABERTHROW)
    {
        //having saber holocrons in saber only mode is pointless
        G_FreeEntity(ent);
        return;
    }

    (*ent).s.isJediMaster = QTRUE;

    VectorSet(&mut (*ent).r.maxs, 8.0, 8.0, 8.0);
    VectorSet(&mut (*ent).r.mins, -8.0, -8.0, -8.0);

    (*ent).s.origin[2] += 0.1;
    (*ent).r.maxs[2] -= 0.1;

    VectorSet(
        &mut dest,
        (*ent).s.origin[0],
        (*ent).s.origin[1],
        (*ent).s.origin[2] - 4096.0,
    );
    let tr = trap::Trace(
        &(*ent).s.origin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &dest,
        (*ent).s.number,
        MASK_SOLID,
    );
    if tr.startsolid != 0 {
        G_Printf(&format!(
            "SP_misc_holocron: misc_holocron startsolid at {}\n",
            CStr::from_ptr(vtos(&(*ent).s.origin)).to_string_lossy()
        ));
        G_FreeEntity(ent);
        return;
    }

    //add the 0.1 back after the trace
    (*ent).r.maxs[2] += 0.1;

    // allow to ride movers
    //	ent->s.groundEntityNum = tr.entityNum;

    G_SetOrigin(ent, &tr.endpos);

    if (*ent).count < 0 {
        (*ent).count = 0;
    }

    if (*ent).count >= NUM_FORCE_POWERS as c_int {
        (*ent).count = NUM_FORCE_POWERS as c_int - 1;
    }
    /*
        if (g_forcePowerDisable.integer &&
            (g_forcePowerDisable.integer & (1 << ent->count)))
        {
            G_FreeEntity(ent);
            return;
        }
    */
    //No longer doing this, causing too many complaints about accidentally setting no force powers at all
    //and starting a holocron game (making it basically just FFA)

    (*ent).enemy = core::ptr::null_mut();

    (*ent).flags = FL_BOUNCE_HALF;

    (*ent).s.modelindex = (*ent).count - 128; //G_ModelIndex(holocronTypeModels[ent->count]);
    (*ent).s.eType = ET_HOLOCRON;
    (*ent).s.pos.trType = TR_GRAVITY;
    (*ent).s.pos.trTime = (*addr_of!(level)).time;

    (*ent).r.contents = CONTENTS_TRIGGER;
    (*ent).clipmask = MASK_SOLID;

    (*ent).s.trickedentindex4 = (*ent).count;

    if forcePowerDarkLight[(*ent).count as usize] == FORCE_DARKSIDE {
        (*ent).s.trickedentindex3 = 1;
    } else if forcePowerDarkLight[(*ent).count as usize] == FORCE_LIGHTSIDE {
        (*ent).s.trickedentindex3 = 2;
    } else {
        (*ent).s.trickedentindex3 = 3;
    }

    (*ent).physicsObject = QTRUE;

    let trBase = (*ent).s.pos.trBase;
    VectorCopy(&trBase, &mut (*ent).s.origin2); //remember the spawn spot

    (*ent).touch = Some(HolocronTouch);

    trap::LinkEntity(ent);

    (*ent).think = Some(HolocronThink);
    (*ent).nextthink = (*addr_of!(level)).time + 50;
}

/// `qboolean gEscaping = qfalse;` (g_misc.c:2315). Set by the `target_escapetrigger`
/// machinery (not yet ported) to mark that an "escape"-style round end-condition is
/// active; read in `CheckExitRules` (g_main.c) to decide whether to end the level when
/// the escape timer elapses or all live players die. A single-threaded game-module
/// global; always accessed through `addr_of!`/`addr_of_mut!`.
pub static mut gEscaping: qboolean = QFALSE;

/// `int gEscapeTime = 0;` (g_misc.c:2316). `level.time` deadline by which players must
/// escape; once `gEscapeTime < level.time`, `CheckExitRules` ends the round. Companion
/// to [`gEscaping`].
pub static mut gEscapeTime: c_int = 0;

/// `#define STATION_RECHARGE_TIME 100` (g_misc.c:12). Default recharge interval (ms) for the
/// energy/shield station when no `chargerate` spawn key is supplied.
const STATION_RECHARGE_TIME: c_int = 100;

/// `#define MAX_AMMO_GIVE 2` (g_misc.c:11). Per-tick cap on shield/health/ammo dispensed by the
/// power-converter `use` callbacks.
const MAX_AMMO_GIVE: c_int = 2;

/*QUAKED info_camp (0 0.5 0) (-4 -4 -4) (4 4 4)
Used as a positional target for calculations in the utilities (spotlights, etc), but removed during gameplay.
*/
/// `void SP_info_camp( gentity_t *self )` (g_misc.c:24). Just pins the origin so other
/// entities can target it; survives into gameplay. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_camp(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);
}

/*QUAKED info_null (0 0.5 0) (-4 -4 -4) (4 4 4)
Used as a positional target for calculations in the utilities (spotlights, etc), but removed during gameplay.
*/
/// `void SP_info_null( gentity_t *self )` (g_misc.c:32). Editor-only positional target; the
/// utilities bake it in, so it is freed at spawn. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_null(self_: *mut gentity_t) {
    G_FreeEntity(self_);
}

/*QUAKED info_notnull (0 0.5 0) (-4 -4 -4) (4 4 4)
Used as a positional target for in-game calculation, like jumppad targets.
target_position does the same thing
*/
/// `void SP_info_notnull( gentity_t *self )` (g_misc.c:41). Like `info_null` but kept at
/// runtime (jumppad targets, etc). No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_notnull(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);
}

/// `static void misc_lightstyle_set( gentity_t *ent )` (g_misc.c:87). Drives a switchable
/// light by rewriting its three light-style configstrings (RGB): when off, copy from the
/// `style_off` style if one is set, else go dark (`"a"`); when on, copy from the
/// `switch_style` style if set, else go full bright (`"z"`). No oracle (engine configstring
/// syscalls).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
unsafe fn misc_lightstyle_set(ent: *mut gentity_t) {
    let mLightStyle: c_int = (*ent).count;
    let mLightSwitchStyle: c_int = (*ent).bounceCount;
    let mLightOffStyle: c_int = (*ent).fly_sound_debounce_time;
    if (*ent).alt_fire == QFALSE {
        // turn off
        if mLightOffStyle != 0 {
            // i have a light style i'd like to use when off
            for i in 0..3 {
                let lightstyle = trap::GetConfigstring(CS_LIGHT_STYLES + (mLightOffStyle * 3) + i);
                trap::SetConfigstring(CS_LIGHT_STYLES + (mLightStyle * 3) + i, &lightstyle);
            }
        } else {
            for i in 0..3 {
                trap::SetConfigstring(CS_LIGHT_STYLES + (mLightStyle * 3) + i, "a");
            }
        }
    } else {
        // Turn myself on now
        if mLightSwitchStyle != 0 {
            // i have a light style i'd like to use when on
            for i in 0..3 {
                let lightstyle =
                    trap::GetConfigstring(CS_LIGHT_STYLES + (mLightSwitchStyle * 3) + i);
                trap::SetConfigstring(CS_LIGHT_STYLES + (mLightStyle * 3) + i, &lightstyle);
            }
        } else {
            for i in 0..3 {
                trap::SetConfigstring(CS_LIGHT_STYLES + (mLightStyle * 3) + i, "z");
            }
        }
    }
}

/// `void misc_dlight_use( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:135). Toggles the switchable light's on/off state and re-applies its styles.
/// No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn misc_dlight_use(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(ent, BSET_USE);

    (*ent).alt_fire = if (*ent).alt_fire != QFALSE {
        QFALSE
    } else {
        QTRUE
    }; // toggle
    misc_lightstyle_set(ent);
}

/*QUAKED light (0 1 0) (-8 -8 -8) (8 8 8) linear noIncidence START_OFF
Non-displayed light. See g_misc.c for the full key/style documentation.
*/
/// `void SP_light( gentity_t *self )` (g_misc.c:143). Only switchable lights (those with a
/// `targetname`) survive into the running game; the rest are baked into the lightmap and
/// freed. Reads the on/off/switch style indices, installs the toggle `use`, and applies the
/// initial state (on unless the `START_OFF` spawnflag bit 4 is set). No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_light(self_: *mut gentity_t) {
    if (*self_).targetname.is_null() {
        // if i don't have a light style switch, the i go away
        G_FreeEntity(self_);
        return;
    }

    G_SpawnInt(c"style".as_ptr(), c"0".as_ptr(), &mut (*self_).count);
    G_SpawnInt(
        c"switch_style".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).bounceCount,
    );
    G_SpawnInt(
        c"style_off".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).fly_sound_debounce_time,
    );
    G_SetOrigin(self_, &(*self_).s.origin);
    trap::LinkEntity(self_);

    (*self_).r#use = Some(misc_dlight_use);

    (*self_).s.eType = ET_GENERAL;
    (*self_).alt_fire = QFALSE;
    (*self_).r.svFlags |= SVF_NOCLIENT;

    if (*self_).spawnflags & 4 == 0 {
        // turn myself on now
        (*self_).alt_fire = QTRUE;
    }
    misc_lightstyle_set(self_);
}

/*
=================================================================================
TELEPORTERS
=================================================================================
*/

/// `void TeleportPlayer( gentity_t *player, vec3_t origin, vec3_t angles )` (g_misc.c:177).
/// Relocates a player (or NPC) to `origin`/`angles`: fires teleport-out/in temp events
/// (skipped for spectators), unlinks to clear the way for `G_KillBox`, snaps the
/// playerstate origin (+1 unit), spits the player out along `angles` at speed 400 with a
/// 160 ms knockback hold, toggles `EF_TELEPORT_BIT` so the client stops lerping, sets the
/// view, kills anything at the destination, then folds the playerstate back into the entity
/// state and relinks. The NPC eType is preserved across `BG_PlayerStateToEntityState`. No
/// oracle (drives engine link/temp-entity syscalls and `G_KillBox`).
///
/// # Safety
/// `player` must point to a valid `gentity_t` whose `client` is non-NULL.
pub unsafe fn TeleportPlayer(player: *mut gentity_t, origin: &vec3_t, angles: &vec3_t) {
    let isNPC = (*player).s.eType == ET_NPC;

    // use temp events at source and destination to prevent the effect
    // from getting dropped by a second player event
    if (*(*player).client).sess.sessionTeam != TEAM_SPECTATOR {
        let tent = G_TempEntity(&(*(*player).client).ps.origin, EV_PLAYER_TELEPORT_OUT);
        (*tent).s.clientNum = (*player).s.clientNum;

        let tent = G_TempEntity(origin, EV_PLAYER_TELEPORT_IN);
        (*tent).s.clientNum = (*player).s.clientNum;
    }

    // unlink to make sure it can't possibly interfere with G_KillBox
    trap::UnlinkEntity(player);

    VectorCopy(origin, &mut (*(*player).client).ps.origin);
    (*(*player).client).ps.origin[2] += 1.0;

    // spit the player out
    AngleVectors(
        angles,
        Some(&mut (*(*player).client).ps.velocity),
        None,
        None,
    );
    let vel = (*(*player).client).ps.velocity;
    VectorScale(&vel, 400.0, &mut (*(*player).client).ps.velocity);
    (*(*player).client).ps.pm_time = 160; // hold time
    (*(*player).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;

    // toggle the teleport bit so the client knows to not lerp
    (*(*player).client).ps.eFlags ^= EF_TELEPORT_BIT;

    // set angles
    SetClientViewAngle(player, angles);

    // kill anything at the destination
    if (*(*player).client).sess.sessionTeam != TEAM_SPECTATOR {
        G_KillBox(player);
    }

    // save results of pmove
    BG_PlayerStateToEntityState(&mut (*(*player).client).ps, &mut (*player).s, QTRUE);
    if isNPC {
        (*player).s.eType = ET_NPC;
    }

    // use the precise origin for linking
    VectorCopy(
        &(*(*player).client).ps.origin,
        &mut (*player).r.currentOrigin,
    );

    if (*(*player).client).sess.sessionTeam != TEAM_SPECTATOR {
        trap::LinkEntity(player);
    }
}

/*QUAKED misc_teleporter_dest (1 0 0) (-32 -32 -24) (32 32 -16)
Point teleporters at these.
Now that we don't have teleport destination pads, this is just an info_notnull
*/
/// `void SP_misc_teleporter_dest( gentity_t *ent )` (g_misc.c:238). A no-op marker the
/// teleporters aim at — the entity is left exactly as spawned. No oracle.
///
/// # Safety
/// `_ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_teleporter_dest(_ent: *mut gentity_t) {}

/*QUAKED misc_model (1 0 0) (-16 -16 -16) (16 16 16)
"model"		arbitrary .md3 or .ase file to display — turns into map triangles, not solid
*/
/// `void SP_misc_model( gentity_t *ent )` (g_misc.c:249). The in-engine model spawn is
/// `#if 0`'d out upstream (the geometry is baked into the BSP by the compiler), so the live
/// path just frees the entity. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_model(ent: *mut gentity_t) {
    G_FreeEntity(ent);
}

/*QUAKED misc_model_static (1 0 0) (-16 -16 0) (16 16 16)
"model"		arbitrary .md3 file to display — loaded as a model in the renderer, not BSP space
*/
/// `void SP_misc_model_static( gentity_t *ent )` (g_misc.c:277). Renderer-side static model;
/// the game module just frees the spawn entity. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_model_static(ent: *mut gentity_t) {
    G_FreeEntity(ent);
}

/*QUAKED misc_G2model (1 0 0) (-16 -16 -16) (16 16 16)
"model"		arbitrary .glm file to display
*/
/// `void SP_misc_G2model( gentity_t *ent )` (g_misc.c:285). The Ghoul2 model spawn is
/// `#if 0`'d out upstream, so the live path just frees the entity. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_G2model(ent: *mut gentity_t) {
    G_FreeEntity(ent);
}

//===========================================================

/// `void locateCamera( gentity_t *ent )` (g_misc.c:305). Think for a `misc_portal_surface`:
/// resolves its targeted `misc_portal_camera` ("owner"), copies the rotate speed/offset and
/// swing flags from the owner's spawnflags into this entity's state, and points the portal
/// view either at the camera's own target or along the camera's facing. Frees itself if the
/// owner can't be found. No oracle (`G_PickTarget` target-name lookup + entity-state
/// plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn locateCamera(ent: *mut gentity_t) {
    let mut dir: vec3_t = [0.0; 3];
    let target: *mut gentity_t;
    let owner: *mut gentity_t;

    owner = G_PickTarget((*ent).target);
    if owner.is_null() {
        G_Printf("Couldn't find target for misc_partal_surface\n");
        G_FreeEntity(ent);
        return;
    }
    (*ent).r.ownerNum = (*owner).s.number;

    // frame holds the rotate speed
    if (*owner).spawnflags & 1 != 0 {
        (*ent).s.frame = 25;
    } else if (*owner).spawnflags & 2 != 0 {
        (*ent).s.frame = 75;
    }

    // swing camera ?
    if (*owner).spawnflags & 4 != 0 {
        // set to 0 for no rotation at all
        (*ent).s.powerups = 0;
    } else {
        (*ent).s.powerups = 1;
    }

    // clientNum holds the rotate offset
    (*ent).s.clientNum = (*owner).s.clientNum;

    VectorCopy(&(*owner).s.origin, &mut (*ent).s.origin2);

    // see if the portal_camera has a target
    target = G_PickTarget((*owner).target);
    if !target.is_null() {
        VectorSubtract(&(*target).s.origin, &(*owner).s.origin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        G_SetMovedir(&mut (*owner).s.angles, &mut dir);
    }

    (*ent).s.eventParm = DirToByte(&dir);
}

/*QUAKED fx_snow (1 0 0) (-16 -16 -16) (16 16 16)
This world effect will spawn snow globally into the level.

"count" the number of snow particles (default of 1000)
*/
/// `void SP_CreateSnow( gentity_t *ent )` (g_misc.c:2297). Precaches the global snow world
/// effect plus its companion fog and constant-wind effects. `ent` is unused (the snow takes
/// no parameters). No oracle (`G_EffectIndex` configstring syscalls).
///
/// # Safety
/// `_ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_CreateSnow(_ent: *mut gentity_t) {
    G_EffectIndex("*snow");
    G_EffectIndex("*fog");
    G_EffectIndex("*constantwind (100 100 -100)");
}

/*QUAKED fx_rain (1 0 0) (-16 -16 -16) (16 16 16)
This world effect will spawn rain globally into the level.

"count" the number of rain particles (default of 500)
*/
/// `void SP_CreateRain( gentity_t *ent )` (g_misc.c:2310). Precaches the global rain world
/// effect, parameterised by the `count` particle count. The C `va("*rain init %i",
/// ent->count)` format-buffer collapses into the `&str` the index helper already marshals
/// back into a C string. No oracle (`G_EffectIndex` configstring syscall).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_CreateRain(ent: *mut gentity_t) {
    G_EffectIndex(&format!("*rain init {}", (*ent).count));
}

/// `void Use_Target_Screenshake( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:2318). Fires a screen shake of the configured intensity/duration, either
/// globally (all clients) or only within the PVS of its origin. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn Use_Target_Screenshake(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    let mut bGlobal = QFALSE;

    if (*ent).genericValue6 != 0 {
        bGlobal = QTRUE;
    }

    G_ScreenShake(
        &(*ent).s.origin,
        core::ptr::null_mut(),
        (*ent).speed,
        (*ent).genericValue5,
        bGlobal,
    );
}

/*QUAKED target_screenshake (1 0 0) (-4 -4 -4) (4 4 4) GLOBAL
Shakes the screen of nearby (or all) clients when used.
*/
/// `void SP_target_screenshake( gentity_t *ent )` (g_misc.c:2330). Reads the shake
/// intensity/duration/global keys and installs the `use` callback. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_screenshake(ent: *mut gentity_t) {
    G_SpawnFloat(c"intensity".as_ptr(), c"10".as_ptr(), &mut (*ent).speed);
    // intensity of the shake
    G_SpawnInt(
        c"duration".as_ptr(),
        c"800".as_ptr(),
        &mut (*ent).genericValue5,
    );
    // duration of the shake
    G_SpawnInt(
        c"globalshake".as_ptr(),
        c"1".as_ptr(),
        &mut (*ent).genericValue6,
    );
    // non-0 if shake should be global (all clients). Otherwise, only in the PVS.

    (*ent).r#use = Some(Use_Target_Screenshake);
}

/// `void check_recharge( gentity_t *ent )` (g_misc.c:951). Think callback for the
/// energy/shield recharge station: while an activator is holding `USE` (within the debounce
/// window) it stays "in use" and does not recharge; otherwise it plays the done sound,
/// drops the loop sound, clears the activator, and ticks `count` up toward `genericValue4`
/// on the `genericValue5` recharge interval. Mirrors the fill level into `s.health` (the
/// health bar) and reschedules itself for the next frame. No oracle (level/entity plumbing
/// + `G_Sound` syscall).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn check_recharge(ent: *mut gentity_t) {
    if (*ent).fly_sound_debounce_time < (*addr_of!(level)).time
        || (*ent).activator.is_null()
        || (*(*ent).activator).client.is_null()
        || (*(*(*ent).activator).client).pers.cmd.buttons & BUTTON_USE == 0
    {
        if !(*ent).activator.is_null() {
            G_Sound(ent, CHAN_AUTO, (*ent).genericValue7);
        }
        (*ent).s.loopSound = 0;
        (*ent).s.loopIsSoundset = QFALSE;
        (*ent).activator = core::ptr::null_mut();
        (*ent).fly_sound_debounce_time = 0;
    }

    if (*ent).activator.is_null() {
        // don't recharge during use
        if (*ent).genericValue8 < (*addr_of!(level)).time {
            if (*ent).count < (*ent).genericValue4 {
                (*ent).count += 1;
            }
            (*ent).genericValue8 = (*addr_of!(level)).time + (*ent).genericValue5;
        }
    }
    (*ent).s.health = (*ent).count; // the "health bar" is gonna be how full we are
    (*ent).nextthink = (*addr_of!(level)).time;
}

/// `void EnergyShieldStationSettings( gentity_t *ent )` (g_misc.c:988). Reads the
/// energy/shield station's spawn keys: `count` (capacity, default 200) and `chargerate`
/// (recharge interval, default 0). When no `chargerate` was supplied, falls back to
/// `STATION_RECHARGE_TIME`. No oracle (`G_SpawnInt` spawn-var read + entity mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn EnergyShieldStationSettings(ent: *mut gentity_t) {
    G_SpawnInt(c"count".as_ptr(), c"200".as_ptr(), &mut (*ent).count);

    G_SpawnInt(
        c"chargerate".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue5,
    );

    if (*ent).genericValue5 == 0 {
        (*ent).genericValue5 = STATION_RECHARGE_TIME;
    }
}

/*
================
EnergyAmmoShieldStationSettings
================
*/
/// `void EnergyAmmoStationSettings( gentity_t *ent )` (g_misc.c:1518). Reads the energy/ammo
/// power-converter's `count` capacity spawn key (default 200). No oracle (`G_SpawnInt`
/// spawn-var read + entity mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn EnergyAmmoStationSettings(ent: *mut gentity_t) {
    G_SpawnInt(c"count".as_ptr(), c"200".as_ptr(), &mut (*ent).count);
}

/*
================
EnergyHealthStationSettings
================
*/
/// `void EnergyHealthStationSettings( gentity_t *ent )` (g_misc.c:1686). Reads the
/// energy/health power-converter's `count` capacity spawn key (default 200). No oracle
/// (`G_SpawnInt` spawn-var read + entity mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn EnergyHealthStationSettings(ent: *mut gentity_t) {
    G_SpawnInt(c"count".as_ptr(), c"200".as_ptr(), &mut (*ent).count);
}

//rww - Called by skyportal entities. This will check through entities and flag them
//as portal ents if they are in the same pvs as a skyportal entity and pass
//a direct point trace check between origins. I really wanted to use an eFlag for
//flagging portal entities, but too many entities like to reset their eFlags.
//Note that this was not part of the original wolf sky portal stuff.
/// `void G_PortalifyEntities( gentity_t *ent )` (g_misc.c:640). Skyportal `think`: walks the
/// whole `g_entities` array and flags every in-use entity (other than the portal itself)
/// that shares the portal's PVS and is reachable by a direct point trace as a portal ent
/// (`s.isPortalEnt`), so the server forwards it to all clients regardless of where they are.
/// Clients are skipped unless they are NPCs (flagging a real client would be bad). Finally
/// the portal entity frees itself next frame — its data now lives in a configstring. No
/// oracle (`trap_InPVS`/`trap_Trace` engine syscalls + `g_entities`/`level` plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn G_PortalifyEntities(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    while (i as usize) < MAX_GENTITIES {
        let scan: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);

        if !scan.is_null()
            && (*scan).inuse != QFALSE
            && (*scan).s.number != (*ent).s.number
            && trap::InPVS(&(*ent).s.origin, &(*scan).r.currentOrigin) != QFALSE
        {
            let tr = trap::Trace(
                &(*ent).s.origin,
                &vec3_origin,
                &vec3_origin,
                &(*scan).r.currentOrigin,
                (*ent).s.number,
                CONTENTS_SOLID,
            );

            if tr.fraction == 1.0
                || (tr.entityNum as c_int == (*scan).s.number
                    && tr.entityNum as c_int != ENTITYNUM_NONE
                    && tr.entityNum as c_int != ENTITYNUM_WORLD)
            {
                if (*scan).client.is_null() || (*scan).s.eType == ET_NPC {
                    //making a client a portal entity would be bad.
                    (*scan).s.isPortalEnt = QTRUE; //he's flagged now
                }
            }
        }

        i += 1;
    }

    (*ent).think = Some(G_FreeEntity); //the portal entity is no longer needed because its information is stored in a config string.
    (*ent).nextthink = (*addr_of!(level)).time;
}

/*
======================================================================

  SHOOTERS

======================================================================
*/

/// `void Use_Shooter( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_misc.c:882).
/// `use` callback for a `shooter_*` entity: aims at its `enemy` (if any) or its fixed
/// `movedir`, randomly perturbs the direction within `random` radians off two perpendicular
/// axes, and fires the configured weapon (only `WP_BLASTER` is implemented), then plays the
/// fire event. No oracle (`crandom` shared-LCG RNG + `WP_FireBlasterMissile` projectile
/// spawn / entity-event syscalls).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn Use_Shooter(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    let mut dir: vec3_t = [0.0; 3];
    let mut deg: f32;
    let mut up: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];

    // see if we have a target
    if !(*ent).enemy.is_null() {
        VectorSubtract(&(*(*ent).enemy).r.currentOrigin, &(*ent).s.origin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        VectorCopy(&(*ent).movedir, &mut dir);
    }

    // randomize a bit
    PerpendicularVector(&mut up, &dir);
    CrossProduct(&up, &dir, &mut right);

    deg = crandom() as f32 * (*ent).random;
    let dir_copy = dir;
    VectorMA(&dir_copy, deg, &up, &mut dir);

    deg = crandom() as f32 * (*ent).random;
    let dir_copy = dir;
    VectorMA(&dir_copy, deg, &right, &mut dir);

    VectorNormalize(&mut dir);

    match (*ent).s.weapon {
        WP_BLASTER => {
            WP_FireBlasterMissile(ent, &mut (*ent).s.origin, &dir, QFALSE);
        }
        _ => {}
    }

    G_AddEvent(ent, EV_FIRE_WEAPON, 0);
}

/// `static void InitShooter_Finish( gentity_t *ent )` (g_misc.c:917). Deferred think for a
/// `shooter_*` entity that targets a (possibly moving) object: resolves the target name into
/// an `enemy` pointer once the level has spawned, then clears the think so it never runs
/// again. No oracle (`G_PickTarget` target-name lookup + entity-state plumbing). Ported `pub`
/// though the C is `static` (the landed-leaf visibility precedent — avoids a `dead_code`
/// warning until its sole installer `InitShooter` lands).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn InitShooter_Finish(ent: *mut gentity_t) {
    (*ent).enemy = G_PickTarget((*ent).target);
    (*ent).think = None;
    (*ent).nextthink = 0;
}

/// `void InitShooter( gentity_t *ent, int weapon )` (g_misc.c:923). Spawn-time setup for a
/// `shooter_*` entity: installs `Use_Shooter`, registers the weapon's item, folds the
/// designer `random` (degrees of deviance) into a `sin`-scaled radian factor, schedules the
/// deferred `InitShooter_Finish` target resolve if it has a (possibly moving) target, and
/// links the entity. No oracle (`trap_LinkEntity` plus item-registration / entity-state
/// plumbing; `M_PI`/`sin` are exercised bit-exact elsewhere).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn InitShooter(ent: *mut gentity_t, weapon: c_int) {
    (*ent).r#use = Some(Use_Shooter);
    (*ent).s.weapon = weapon;

    RegisterItem(BG_FindItemForWeapon(weapon));

    G_SetMovedir(&mut (*ent).s.angles, &mut (*ent).movedir);

    if (*ent).random == 0.0 {
        (*ent).random = 1.0;
    }
    (*ent).random = ((M_PI * (*ent).random / 180.0) as f64).sin() as f32;
    // target might be a moving object, so we can't set movedir for it
    if !(*ent).target.is_null() {
        (*ent).think = Some(InitShooter_Finish);
        (*ent).nextthink = (*addr_of!(level)).time + 500;
    }
    trap::LinkEntity(ent);
}

/*QUAKED shooter_blaster (1 0 0) (-16 -16 -16) (16 16 16)
Fires at either the target or the current direction.
"random" is the number of degrees of deviance from the taget. (1.0 default)
*/
/// `void SP_shooter_blaster( gentity_t *ent )` (g_misc.c:947). The `shooter_blaster` spawner:
/// a `WP_BLASTER` automated turret/trap. Delegates to `InitShooter`. No oracle (spawn plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_shooter_blaster(ent: *mut gentity_t) {
    InitShooter(ent, WP_BLASTER);
}

/// `void Use_Target_Escapetrig( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:2344). Use callback for a `target_escapetrig`: if this is the start trigger
/// (`!genericValue6`) it arms the SP-style "escape" round (`gEscaping`/`gEscapeTime`);
/// otherwise, while an escape is ongoing, it ends it — awarding 100 points to every living,
/// non-spectating, non-following survivor and 500 to the activator who escaped — then logs
/// the exit. No oracle (`AddScore`/`LogExit` + global `gEscaping`/`g_entities` plumbing).
///
/// # Safety
/// `ent`/`activator` must be valid entity pointers (or `activator` null); `other` is unused.
pub unsafe extern "C" fn Use_Target_Escapetrig(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*ent).genericValue6 == 0 {
        gEscaping = QTRUE;
        gEscapeTime = (*addr_of!(level)).time + (*ent).genericValue5;
    } else if gEscaping != QFALSE {
        let mut i: c_int = 0;
        gEscaping = QFALSE;
        while i < MAX_CLIENTS as c_int {
            //all of the survivors get 100 points!
            let ent_i = core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>()
                .add(i as usize);
            if (*ent_i).inuse != QFALSE
                && !(*ent_i).client.is_null()
                && (*ent_i).health > 0
                && (*(*ent_i).client).sess.sessionTeam != TEAM_SPECTATOR
                && ((*(*ent_i).client).ps.pm_flags & PMF_FOLLOW) == 0
            {
                let origin = (*(*ent_i).client).ps.origin;
                AddScore(ent_i, &origin, 100);
            }
            i += 1;
        }
        if !activator.is_null() && (*activator).inuse != QFALSE && !(*activator).client.is_null() {
            //the one who escaped gets 500
            let origin = (*(*activator).client).ps.origin;
            AddScore(activator, &origin, 500);
        }

        LogExit("Escaped!");
    }
}

/// `void SP_target_escapetrig( gentity_t *ent )` (g_misc.c:2374). Spawn function for a
/// `target_escapetrig`. This is a single-player-only mechanic, so it frees itself in any
/// non-`GT_SINGLE_PLAYER` gametype; otherwise it reads the `escapetime` (escape timer, ms)
/// and `escapegoal` (non-0 = this trigger *ends* an escape rather than starting one) spawn
/// keys and installs [`Use_Target_Escapetrig`]. No oracle (spawn plumbing:
/// `G_FreeEntity`/`G_SpawnInt`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_escapetrig(ent: *mut gentity_t) {
    if (*addr_of!(g_gametype)).integer != GT_SINGLE_PLAYER {
        G_FreeEntity(ent);
        return;
    }

    G_SpawnInt(
        c"escapetime".as_ptr(),
        c"60000".as_ptr(),
        &mut (*ent).genericValue5,
    );
    //time given (in ms) for the escape
    G_SpawnInt(
        c"escapegoal".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue6,
    );
    //if non-0, when used, will end an ongoing escape instead of start it

    (*ent).r#use = Some(Use_Target_Escapetrig);
}

/// `void maglock_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int mod )` (g_misc.c:2398). Death callback for a `misc_maglock` door lock: when this was
/// the last lock pointed at the door, decrements its `lockCount` and clears `FL_INACTIVE` so
/// the door works again, then fires the lock's targets. The upstream `WP_Explode( self )`
/// call is `//`-commented out in C (an `rwwFIXMEFIXME`) and is preserved commented here. No
/// oracle (entity-state plumbing + `G_UseTargets`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other parameters are unused.
pub unsafe extern "C" fn maglock_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    //unlock our door if we're the last lock pointed at the door
    if !(*self_).activator.is_null() {
        (*(*self_).activator).lockCount -= 1;
        if (*(*self_).activator).lockCount == 0 {
            (*(*self_).activator).flags &= !FL_INACTIVE;
        }
    }

    //use targets
    G_UseTargets(self_, attacker);
    //die
    //rwwFIXMEFIXME - weap expl func
    //	WP_Explode( self );
}

/*QUAKED misc_maglock (0 .5 .8) (-8 -8 -8) (8 8 8) x x x x x x x x
Place facing a door (using the angle, not a targetname) and it will lock that door.  Can only be destroyed by lightsaber and will automatically unlock the door it's attached to

NOTE: place these half-way in the door to make it flush with the door's surface.

"target"	thing to use when destoryed (not doors - it automatically unlocks the door it was angled at)
"health"	default is 10
*/
/// `void SP_misc_maglock( gentity_t *self )` (g_misc.c:2420). Spawn function for a
/// `misc_maglock` door lock: sets its `door_lock.md3` model and `maglock/explosion` effect
/// index, freezes its origin, then defers linking to its door via [`maglock_link`] — armed an
/// extra 200 ms past `START_TIME_FIND_LINKS` so the doors have linked and spawned their
/// triggers first. C declares the `maglock_link` think and `G_FindDoorTrigger` helper via its
/// own forward-decls just above; both are already ported here so no shim is needed. No oracle
/// (spawn plumbing: `G_ModelIndex`/`G_EffectIndex`/`G_SetOrigin` + entity-state).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_maglock(self_: *mut gentity_t) {
    //NOTE: May have to make these only work on doors that are either untargeted
    //		or are targeted by a trigger, not doors fired off by scripts, counters
    //		or other such things?
    (*self_).s.modelindex = G_ModelIndex("models/map_objects/imp_detention/door_lock.md3");
    (*self_).genericValue1 = G_EffectIndex("maglock/explosion");

    G_SetOrigin(self_, &(*self_).s.origin);

    (*self_).think = Some(maglock_link);
    //FIXME: for some reason, when you re-load a level, these fail to find their doors...?  Random?  Testing an additional 200ms after the START_TIME_FIND_LINKS
    (*self_).nextthink = (*addr_of!(level)).time + START_TIME_FIND_LINKS + 200; //START_TIME_FIND_LINKS;//because we need to let the doors link up and spawn their triggers first!
}

/// `void maglock_link( gentity_t *self )` (g_misc.c:2434). Deferred think for a
/// `misc_maglock`: traces forward to find the `func_door` it locks. While it can't find a
/// surface or a door it re-arms itself for another 100 ms; once it hits a door it bumps the
/// door's (or its trigger's) `lockCount` and marks it `FL_INACTIVE`, then positions/orients
/// itself flush against the surface, gives itself a small hittable corpse box, and makes
/// itself lightsaber-only destroyable (`FL_SHIELDED`, 10 health, `maglock_die`). The
/// in-solid case `Com_Error`s (`-> !`), so the C `G_FreeEntity(self); return;` after it is
/// unreachable. No oracle (`trap_Trace` collision + entity-state plumbing).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn maglock_link(self_: *mut gentity_t) {
    //find what we're supposed to be attached to
    let mut forward: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let trace: trace_t;
    let traceEnt: *mut gentity_t;

    AngleVectors(&(*self_).s.angles, Some(&mut forward), None, None);
    VectorMA(&(*self_).s.origin, 128.0, &forward, &mut end);
    VectorMA(&(*self_).s.origin, -4.0, &forward, &mut start);

    trace = trap::Trace(
        &start,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*self_).s.number,
        MASK_SHOT,
    );

    if trace.allsolid != 0 || trace.startsolid != 0 {
        Com_Error(
            ERR_DROP,
            &format!(
                "misc_maglock at {} in solid\n",
                CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy()
            ),
        );
        // Com_Error (-> !) does not return; the C trailing statements are unreachable:
        //	G_FreeEntity( self );
        //	return;
    }
    if trace.fraction == 1.0 {
        (*self_).think = Some(maglock_link);
        (*self_).nextthink = (*addr_of!(level)).time + 100;
        /*
        Com_Error( ERR_DROP,"misc_maglock at %s pointed at no surface\n", vtos(self->s.origin) );
        G_FreeEntity( self );
        */
        return;
    }
    traceEnt = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add(trace.entityNum as usize);
    if trace.entityNum as c_int >= ENTITYNUM_WORLD
        || traceEnt.is_null()
        || Q_stricmp(c"func_door".as_ptr(), (*traceEnt).classname) != 0
    {
        (*self_).think = Some(maglock_link);
        (*self_).nextthink = (*addr_of!(level)).time + 100;
        //Com_Error( ERR_DROP,"misc_maglock at %s not pointed at a door\n", vtos(self->s.origin) );
        //G_FreeEntity( self );
        return;
    }

    //check the traceEnt, make sure it's a door and give it a lockCount and deactivate it
    //find the trigger for the door
    (*self_).activator = G_FindDoorTrigger(traceEnt);
    if (*self_).activator.is_null() {
        (*self_).activator = traceEnt;
    }
    (*(*self_).activator).lockCount += 1;
    (*(*self_).activator).flags |= FL_INACTIVE;

    //now position and orient it
    vectoangles(&trace.plane.normal, &mut end);
    G_SetOrigin(self_, &trace.endpos);
    G_SetAngles(self_, &end);

    //make it hittable
    //FIXME: if rotated/inclined this bbox may be off... but okay if we're a ghoul model?
    //self->s.modelindex = G_ModelIndex( "models/map_objects/imp_detention/door_lock.md3" );
    VectorSet(&mut (*self_).r.mins, -8.0, -8.0, -8.0);
    VectorSet(&mut (*self_).r.maxs, 8.0, 8.0, 8.0);
    (*self_).r.contents = CONTENTS_CORPSE;

    //make it destroyable
    (*self_).flags |= FL_SHIELDED; //only damagable by lightsabers
    (*self_).takedamage = QTRUE;
    (*self_).health = 10;
    (*self_).die = Some(maglock_die);
    //self->fxID = G_EffectIndex( "maglock/explosion" );

    trap::LinkEntity(self_);
}

/// `void faller_touch( gentity_t *self, gentity_t *other, trace_t *trace )` (g_misc.c:2505).
/// Touch callback for a "faller" (a ragdolling falling body): while it is dropping fast
/// (`epVelocity[2] < -100`) and the pain-debounce window has elapsed, picks one of three
/// stormtrooper pain sounds at random, plays it on the voice channel plus an impact sound on
/// the auto channel, then arms a 3s self-destruct timer and a 200ms pain debounce. No oracle
/// (`Q_irand`/sound-index/entity-sound syscalls + level/entity plumbing).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `_other`/`_trace` are unused.
pub unsafe extern "C" fn faller_touch(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*self_).epVelocity[2] < -100.0 && (*self_).genericValue7 < (*addr_of!(level)).time {
        let r = Q_irand(1, 3);

        if r == 1 {
            (*self_).genericValue11 = G_SoundIndex("sound/chars/stofficer1/misc/pain25");
        } else if r == 2 {
            (*self_).genericValue11 = G_SoundIndex("sound/chars/stofficer1/misc/pain50");
        } else {
            (*self_).genericValue11 = G_SoundIndex("sound/chars/stofficer1/misc/pain75");
        }

        G_EntitySound(self_, CHAN_VOICE, (*self_).genericValue11);
        G_EntitySound(self_, CHAN_AUTO, (*self_).genericValue10);

        (*self_).genericValue6 = (*addr_of!(level)).time + 3000;

        (*self_).genericValue7 = (*addr_of!(level)).time + 200;
    }
}

/// `void faller_think( gentity_t *ent )` (g_misc.c:2533). Think for a "faller" body: once its
/// 15 s lifetime (`genericValue6`) expires it schedules itself for removal via `G_FreeEntity`;
/// otherwise, the first frame it begins dropping fast plays the falling-scream sound (latched
/// by `genericValue8`), runs the extended-physics step, mirrors the resulting velocity into
/// the entity-state trajectory delta (scaled by 10), and reschedules itself 25 ms out. No
/// oracle (`G_EntitySound` syscall + `G_RunExPhys` physics integration / entity-state
/// plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn faller_think(ent: *mut gentity_t) {
    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    if (*ent).genericValue6 < (*addr_of!(level)).time {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*ent).epVelocity[2] < -100.0 {
        if (*ent).genericValue8 == 0 {
            G_EntitySound(ent, CHAN_VOICE, (*ent).genericValue9);
            (*ent).genericValue8 = 1;
        }
    } else {
        (*ent).genericValue8 = 0;
    }

    G_RunExPhys(ent, gravity, mass, bounce, QTRUE, core::ptr::null_mut(), 0);
    let epVelocity = (*ent).epVelocity;
    VectorScale(&epVelocity, 10.0, &mut (*ent).s.pos.trDelta);
    (*ent).nextthink = (*addr_of!(level)).time + 25;
}

/// `void misc_faller_create( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:2564). Spawns one falling-stormtrooper ragdoll at `ent`'s origin: a Ghoul2
/// stormtrooper model with random per-instance RGBA tint, player-solid bounds/clipmask, the
/// `EF_RAG|EF_CLIENTSMOOTH` ragdoll flags, the `faller_think` extended-physics tick and
/// `faller_touch` ground splat, and a random initial horizontal `epVelocity`, then links it.
/// Also doubles as the `use` callback for a `targetname`'d `misc_faller`. No oracle (entity-state
/// spawn: `G_Spawn`/`G_SoundIndex`/`G_ModelIndex`/`G_SetOrigin`/`trap_LinkEntity` side-effects).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn misc_faller_create(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    let faller = G_Spawn();

    (*faller).genericValue10 = G_SoundIndex("sound/player/fallsplat");
    (*faller).genericValue9 = G_SoundIndex("sound/chars/stofficer1/misc/falling1");
    (*faller).genericValue8 = 0;
    (*faller).genericValue7 = 0;

    (*faller).genericValue6 = (*addr_of!(level)).time + 15000;

    G_SetOrigin(faller, &(*ent).s.origin);

    (*faller).s.modelGhoul2 = 1;
    (*faller).s.modelindex = G_ModelIndex("models/players/stormtrooper/model.glm");
    (*faller).s.g2radius = 100;

    (*faller).s.customRGBA[0] = Q_irand(1, 255);
    (*faller).s.customRGBA[1] = Q_irand(1, 255);
    (*faller).s.customRGBA[2] = Q_irand(1, 255);
    (*faller).s.customRGBA[3] = 255;

    VectorSet(&mut (*faller).r.mins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
    VectorSet(&mut (*faller).r.maxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

    (*faller).clipmask = MASK_PLAYERSOLID;
    (*faller).r.contents = MASK_PLAYERSOLID;

    (*faller).s.eFlags = EF_RAG | EF_CLIENTSMOOTH;

    (*faller).think = Some(faller_think);
    (*faller).nextthink = (*addr_of!(level)).time;

    (*faller).touch = Some(faller_touch);

    (*faller).epVelocity[0] = flrand(-256.0, 256.0);
    (*faller).epVelocity[1] = flrand(-256.0, 256.0);

    trap::LinkEntity(faller);
}

/// `void misc_faller_think( gentity_t *ent )` (g_misc.c:2605). Per-interval spawner `think`:
/// drops one faller via [`misc_faller_create`], then re-arms `nextthink` for `genericValue1`
/// (the `interval`) plus a random fudge up to `genericValue2` (the `fudgefactor`). No oracle
/// (delegates to the entity-state spawner).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn misc_faller_think(ent: *mut gentity_t) {
    misc_faller_create(ent, ent, ent);
    (*ent).nextthink =
        (*addr_of!(level)).time + (*ent).genericValue1 + Q_irand(0, (*ent).genericValue2);
}

/*QUAKED misc_faller (1 0 0) (-8 -8 -8) (8 8 8)
Falling stormtrooper - spawned every interval+random fudgefactor,
or if specified, when used.

targetname	- if specified, will only spawn when used
interval	- spawn every so often (milliseconds)
fudgefactor	- milliseconds between 0 and this number randomly added to interval
*/
/// `void SP_misc_faller( gentity_t *ent )` (g_misc.c:2619). Spawn function for `misc_faller`:
/// precaches the stormtrooper model and its pain/fall/splat sounds, reads the `interval`
/// (`genericValue1`, default 500) and `fudgefactor` (`genericValue2`, default 0) spawn keys,
/// then — if it has no `targetname` — arms the periodic [`misc_faller_think`] spawner;
/// otherwise wires [`misc_faller_create`] as its `use` callback. No oracle (entity-state spawn
/// fn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_faller(ent: *mut gentity_t) {
    G_ModelIndex("models/players/stormtrooper/model.glm");
    G_SoundIndex("sound/chars/stofficer1/misc/pain25");
    G_SoundIndex("sound/chars/stofficer1/misc/pain50");
    G_SoundIndex("sound/chars/stofficer1/misc/pain75");
    G_SoundIndex("sound/chars/stofficer1/misc/falling1");
    G_SoundIndex("sound/player/fallsplat");

    G_SpawnInt(
        c"interval".as_ptr(),
        c"500".as_ptr(),
        &mut (*ent).genericValue1,
    );
    G_SpawnInt(
        c"fudgefactor".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue2,
    );

    if (*ent).targetname.is_null() || *(*ent).targetname == 0 {
        (*ent).think = Some(misc_faller_think);
        (*ent).nextthink =
            (*addr_of!(level)).time + (*ent).genericValue1 + Q_irand(0, (*ent).genericValue2);
    } else {
        (*ent).r#use = Some(misc_faller_create);
    }
}

/*QUAKED fx_spacedust (1 0 0) (-16 -16 -16) (16 16 16)
This world effect will spawn space dust globally into the level.

"count" the number of snow particles (default of 1000)
*/
//----------------------------------------------------------
/// `void SP_CreateSpaceDust( gentity_t *ent )` (g_misc.c:2284). Precaches the global
/// space-dust world effect, parameterised by the `count` particle count. The companion
/// `G_EffectIndex("*constantwind ( 10 -10 0 )")` is `//`-commented out in C and is preserved
/// commented here. No oracle (`G_EffectIndex` configstring syscall).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_CreateSpaceDust(ent: *mut gentity_t) {
    G_EffectIndex(&format!("*spacedust {}", (*ent).count));
    //G_EffectIndex("*constantwind ( 10 -10 0 )");
}

/// `void shield_power_converter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:1005). `use` callback for the energy/shield power converter: while it has charge
/// and the activator isn't already at full armor, tops them up by up to `MAX_AMMO_GIVE` per
/// 100 ms tick (clamped to the remaining `count`), draining `count` unless `nodrain`
/// (`genericValue12`); plays the run loop while charging and the done/empty sound when it
/// stops. In siege, a class with no `maxarmor` can't use it at all, and the armor cap comes
/// from `bgSiegeClasses` instead of the player's `STAT_MAX_HEALTH`. No oracle (entity/client
/// stat mutation + `G_Sound`/`G_SoundIndex` syscalls and the `bgSiegeClasses` table).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are checked before use.
pub unsafe extern "C" fn shield_power_converter_use(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let dif: c_int;
    let mut add: c_int;
    let mut stop: c_int = 1;

    if activator.is_null() || (*activator).client.is_null() {
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !other.is_null()
        && !(*other).client.is_null()
        && (*(*other).client).siegeClass != 0
    {
        if bgSiegeClasses[(*(*other).client).siegeClass as usize].maxarmor == 0 {
            //can't use it!
            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex("sound/interface/shieldcon_empty"),
            );
            return;
        }
    }

    if (*self_).setTime < (*addr_of!(level)).time {
        let maxArmor: c_int;
        if (*self_).s.loopSound == 0 {
            (*self_).s.loopSound = G_SoundIndex("sound/interface/shieldcon_run");
            (*self_).s.loopIsSoundset = QFALSE;
        }
        (*self_).setTime = (*addr_of!(level)).time + 100;

        if (*addr_of!(g_gametype)).integer == GT_SIEGE
            && !other.is_null()
            && !(*other).client.is_null()
            && (*(*other).client).siegeClass != -1
        {
            maxArmor = bgSiegeClasses[(*(*other).client).siegeClass as usize].maxarmor;
        } else {
            maxArmor = (*(*activator).client).ps.stats[STAT_MAX_HEALTH as usize];
        }
        dif = maxArmor - (*(*activator).client).ps.stats[STAT_ARMOR as usize];

        if dif > 0 {
            // Already at full armor?
            if dif > MAX_AMMO_GIVE {
                add = MAX_AMMO_GIVE;
            } else {
                add = dif;
            }

            if (*self_).count < add {
                add = (*self_).count;
            }

            if (*self_).genericValue12 == 0 {
                (*self_).count -= add;
            }
            if (*self_).count <= 0 {
                (*self_).setTime = 0;
            }
            stop = 0;

            (*self_).fly_sound_debounce_time = (*addr_of!(level)).time + 500;
            (*self_).activator = activator;

            (*(*activator).client).ps.stats[STAT_ARMOR as usize] += add;
        }
    }

    if stop != 0 || (*self_).count <= 0 {
        if (*self_).s.loopSound != 0 && (*self_).setTime < (*addr_of!(level)).time {
            if (*self_).count <= 0 {
                G_Sound(
                    self_,
                    CHAN_AUTO,
                    G_SoundIndex("sound/interface/shieldcon_empty"),
                );
            } else {
                G_Sound(self_, CHAN_AUTO, (*self_).genericValue7);
            }
        }
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
        if (*self_).setTime < (*addr_of!(level)).time {
            (*self_).setTime = (*addr_of!(level)).time + (*self_).genericValue5 + 100;
        }
    }
}

//dispense generic ammo
/// `void ammo_generic_power_converter_use( gentity_t *self, gentity_t *other, gentity_t
/// *activator )` (g_misc.c:1106). `use` callback for the `misc_ammo_floor_unit` generic-ammo
/// dispenser: while it has charge, tops up every ammo type from `AMMO_BLASTER` to `AMMO_MAX`
/// by 5% of that type's `max` (at least 1), honouring `EF_DOUBLE_AMMO` (cap at 2x) and the
/// siege rocket clamp of 10, then drains `count` by 20% of the last `add` (at least 1) unless
/// `nodrain` (`genericValue12`). Plays the run loop while charging and the done/empty sound
/// when it stops. The large commented-out per-type variant in the C source is preserved
/// verbatim below. No oracle (entity/client mutation + `G_Sound`/`G_SoundIndex` and the
/// `ammoData` table).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are checked before use.
pub unsafe extern "C" fn ammo_generic_power_converter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut add: c_int; //int /*dif,*/ add;
                        //int ammoType;
    let mut stop: c_int = 1;

    if activator.is_null() || (*activator).client.is_null() {
        return;
    }

    if (*self_).setTime < (*addr_of!(level)).time {
        let mut gaveSome = QFALSE;
        /*
        while (i < 3)
        {
            if (!self->s.loopSound)
            {
                self->s.loopSound = G_SoundIndex("sound/interface/ammocon_run");
                self->s.loopIsSoundset = qfalse;
            }
            self->setTime = level.time + 100;

            //dif = activator->client->ps.stats[STAT_MAX_HEALTH] - activator->client->ps.stats[STAT_ARMOR];
            switch (i)
            { //don't give rockets I guess
            case 0:
                ammoType = AMMO_BLASTER;
                break;
            case 1:
                ammoType = AMMO_POWERCELL;
                break;
            case 2:
                ammoType = AMMO_METAL_BOLTS;
                break;
            default:
                ammoType = -1;
                break;
            }

            if (ammoType != -1)
            {
                dif = ammoData[ammoType].max - activator->client->ps.ammo[ammoType];
            }
            else
            {
                dif = 0;
            }

            if (dif > 0)
            { //only give if not full
                if (dif > MAX_AMMO_GIVE)
                {
                    add = MAX_AMMO_GIVE;
                }
                else
                {
                    add = dif;
                }

                if (self->count<add)
                {
                    add = self->count;
                }

                self->count -= add;
                if (self->count <= 0)
                {
                    self->setTime = 0;
                    break;
                }
                stop = 0;

                self->fly_sound_debounce_time = level.time + 500;
                self->activator = activator;

                activator->client->ps.ammo[ammoType] += add;
            }

            i++;
        }
        */
        let mut i: c_int = AMMO_BLASTER;
        if (*self_).s.loopSound == 0 {
            (*self_).s.loopSound = G_SoundIndex("sound/interface/ammocon_run");
            (*self_).s.loopIsSoundset = QFALSE;
        }
        //self->setTime = level.time + 100;
        (*self_).fly_sound_debounce_time = (*addr_of!(level)).time + 500;
        (*self_).activator = activator;
        while i < AMMO_MAX {
            add = (ammoData[i as usize].max as f64 * 0.05) as c_int;
            if add < 1 {
                add = 1;
            }
            if ((*(*activator).client).ps.eFlags & EF_DOUBLE_AMMO != 0
                && (*(*activator).client).ps.ammo[i as usize] < ammoData[i as usize].max * 2)
                || ((*(*activator).client).ps.ammo[i as usize] < ammoData[i as usize].max)
            {
                gaveSome = QTRUE;
                if (*addr_of!(g_gametype)).integer == GT_SIEGE
                    && i == AMMO_ROCKETS
                    && (*(*activator).client).ps.ammo[i as usize] >= 10
                {
                    //this stuff is already a freaking mess, so..
                    gaveSome = QFALSE;
                }
                (*(*activator).client).ps.ammo[i as usize] += add;
                if (*addr_of!(g_gametype)).integer == GT_SIEGE
                    && i == AMMO_ROCKETS
                    && (*(*activator).client).ps.ammo[i as usize] >= 10
                {
                    // fixme - this should SERIOUSLY be externed.
                    (*(*activator).client).ps.ammo[i as usize] = 10;
                } else if (*(*activator).client).ps.eFlags & EF_DOUBLE_AMMO != 0 {
                    if (*(*activator).client).ps.ammo[i as usize] >= ammoData[i as usize].max * 2 {
                        // yuck.
                        (*(*activator).client).ps.ammo[i as usize] = ammoData[i as usize].max * 2;
                    } else {
                        stop = 0;
                    }
                } else {
                    if (*(*activator).client).ps.ammo[i as usize] >= ammoData[i as usize].max {
                        (*(*activator).client).ps.ammo[i as usize] = ammoData[i as usize].max;
                    } else {
                        stop = 0;
                    }
                }
            }
            i += 1;
            if (*self_).genericValue12 == 0 && gaveSome != QFALSE {
                let mut sub: c_int = (add as f64 * 0.2) as c_int;
                if sub < 1 {
                    sub = 1;
                }
                (*self_).count -= sub;
                if (*self_).count <= 0 {
                    (*self_).count = 0;
                    stop = 1;
                    break;
                }
            }
        }
    }

    if stop != 0 || (*self_).count <= 0 {
        if (*self_).s.loopSound != 0 && (*self_).setTime < (*addr_of!(level)).time {
            if (*self_).count <= 0 {
                G_Sound(
                    self_,
                    CHAN_AUTO,
                    G_SoundIndex("sound/interface/ammocon_empty"),
                );
            } else {
                G_Sound(self_, CHAN_AUTO, (*self_).genericValue7);
            }
        }
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
        if (*self_).setTime < (*addr_of!(level)).time {
            (*self_).setTime = (*addr_of!(level)).time + (*self_).genericValue5 + 100;
        }
    }
}

/// `void ammo_power_converter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:1528). `use` callback for the `misc_model_ammo_power_converter`: while it has any
/// `count` left and the tick window has elapsed, tops up every ammo type from `AMMO_BLASTER`
/// to `AMMO_MAX` by 10% of that type's `max` (at least 1, clamped to `max`), draining `count`
/// by the last `add` unless `nodrain` (`genericValue12`). Plays the pickup-shield loop while
/// charging and drops it when it stops. The dead `overcharge` flag and the large commented-out
/// per-type/`highest` block from the C source are preserved verbatim. No oracle (entity/client
/// mutation + `G_SoundIndex` and the `ammoData` table).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are checked before use.
pub unsafe extern "C" fn ammo_power_converter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut add: c_int = 0; //int add = 0.0f;//,highest;
    let _overcharge: c_int; //qboolean overcharge;
                            //	int			difBlaster,difPowerCell,difMetalBolts;
    let mut stop: c_int = 1;

    if activator.is_null() || (*activator).client.is_null() {
        return;
    }

    if (*self_).setTime < (*addr_of!(level)).time {
        _overcharge = QFALSE;

        if (*self_).s.loopSound == 0 {
            (*self_).s.loopSound = G_SoundIndex("sound/player/pickupshield.wav");
        }

        (*self_).setTime = (*addr_of!(level)).time + 100;

        if (*self_).count != 0 {
            // Has it got any power left?
            let mut i: c_int = AMMO_BLASTER;
            while i < AMMO_MAX {
                add = (ammoData[i as usize].max as f64 * 0.1) as c_int;
                if add < 1 {
                    add = 1;
                }
                if (*(*activator).client).ps.ammo[i as usize] < ammoData[i as usize].max {
                    (*(*activator).client).ps.ammo[i as usize] += add;
                    if (*(*activator).client).ps.ammo[i as usize] > ammoData[i as usize].max {
                        (*(*activator).client).ps.ammo[i as usize] = ammoData[i as usize].max;
                    }
                }
                i += 1;
            }
            if (*self_).genericValue12 == 0 {
                (*self_).count -= add;
            }
            stop = 0;

            (*self_).fly_sound_debounce_time = (*addr_of!(level)).time + 500;
            (*self_).activator = activator;

            /*
            if (self->count > MAX_AMMO_GIVE)
            {
                add = MAX_AMMO_GIVE;
            }
            else if (self->count<0)
            {
                add = 0;
            }
            else
            {
                add = self->count;
            }

            activator->client->ps.ammo[AMMO_BLASTER] += add;
            activator->client->ps.ammo[AMMO_POWERCELL] += add;
            activator->client->ps.ammo[AMMO_METAL_BOLTS] += add;

            self->count -= add;
            stop = 0;

            self->fly_sound_debounce_time = level.time + 500;
            self->activator = activator;

            difBlaster = activator->client->ps.ammo[AMMO_BLASTER] - ammoData[AMMO_BLASTER].max;
            difPowerCell = activator->client->ps.ammo[AMMO_POWERCELL] - ammoData[AMMO_POWERCELL].max;
            difMetalBolts = activator->client->ps.ammo[AMMO_METAL_BOLTS] - ammoData[AMMO_METAL_BOLTS].max;

            // Find the highest one
            highest = difBlaster;
            if (difPowerCell>difBlaster)
            {
                highest = difPowerCell;
            }

            if (difMetalBolts > highest)
            {
                highest = difMetalBolts;
            }
            */
        }
    }

    if stop != 0 {
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
    }
}

/// `void health_power_converter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:1696). `use` callback for the `misc_model_health_power_converter`: while the tick
/// window has elapsed and the activator isn't already at full health, adds up to 5 HP
/// (clamped to the remaining `count`) to the activator's *entity* `health` per tick. Plays the
/// pickup-health loop while charging and drops it when it stops. The `MAX_AMMO_GIVE` macro is
/// `//`-replaced by the literal `5` in C, and the `self->count -= add` drain is itself
/// commented out upstream — both preserved here. No oracle (entity/client mutation +
/// `G_SoundIndex`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are checked before use.
pub unsafe extern "C" fn health_power_converter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let dif: c_int;
    let mut add: c_int;
    let mut stop: c_int = 1;

    if activator.is_null() || (*activator).client.is_null() {
        return;
    }

    if (*self_).setTime < (*addr_of!(level)).time {
        if (*self_).s.loopSound == 0 {
            (*self_).s.loopSound = G_SoundIndex("sound/player/pickuphealth.wav");
        }
        (*self_).setTime = (*addr_of!(level)).time + 100;

        dif = (*(*activator).client).ps.stats[STAT_MAX_HEALTH as usize] - (*activator).health;

        if dif > 0 {
            // Already at full armor?
            if dif > /*MAX_AMMO_GIVE*/ 5 {
                add = 5; //MAX_AMMO_GIVE;
            } else {
                add = dif;
            }

            if (*self_).count < add {
                add = (*self_).count;
            }

            //self->count -= add;
            stop = 0;

            (*self_).fly_sound_debounce_time = (*addr_of!(level)).time + 500;
            (*self_).activator = activator;

            (*activator).health += add;
        }
    }

    if stop != 0 {
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
    }
}

/*QUAKED misc_model_shield_power_converter (1 0 0) (-16 -16 -16) (16 16 16)
model="models/items/psd_big.md3"
Gives shield energy when used.

"count" - the amount of ammo given when used (default 200)
*/
//------------------------------------------------------------
/// `void SP_misc_model_shield_power_converter( gentity_t *ent )` (g_misc.c:1472). Spawns the
/// shield-recharge station: sizes its bounds, indexes the model, makes it player-usable solid,
/// reads its `count` via [`EnergyShieldStationSettings`], wires `check_recharge` as the think and
/// `shield_power_converter_use` as the use callback, seeds the cgame health bar, precaches the
/// run/done/empty sounds, links it, and (in Siege) broadcasts a radar icon. No oracle
/// (entity-state spawn fn — sets `gentity`/state fields + fn-ptrs, calls `trap_LinkEntity`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_model_shield_power_converter(ent: *mut gentity_t) {
    if (*ent).health == 0 {
        (*ent).health = 60;
    }

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, -16.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 16.0);

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).s.eFlags = 0;
    (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).clipmask = MASK_SOLID;

    EnergyShieldStationSettings(ent);

    (*ent).genericValue4 = (*ent).count; //initial value
    (*ent).think = Some(check_recharge);

    (*ent).s.maxhealth = (*ent).count;
    (*ent).s.health = (*ent).count;
    (*ent).s.shouldtarget = QTRUE;
    (*ent).s.teamowner = 0;
    (*ent).s.owner = ENTITYNUM_NONE;

    (*ent).nextthink = (*addr_of!(level)).time + 200; // + STATION_RECHARGE_TIME;

    (*ent).r#use = Some(shield_power_converter_use);

    G_SetOrigin(ent, &(*ent).s.origin);
    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);
    trap::LinkEntity(ent);

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");

    (*ent).s.modelindex2 = G_ModelIndex("/models/items/psd_big.md3"); // Precache model
}

/*QUAKED misc_model_ammo_power_converter (1 0 0) (-16 -16 -16) (16 16 16)
model="models/items/power_converter.md3"
Gives ammo energy when used.

"count" - the amount of ammo given when used (default 200)
"nodrain" - don't drain power from me
*/
//------------------------------------------------------------
/// `void SP_misc_model_ammo_power_converter( gentity_t *ent )` (g_misc.c:1639). Spawns the ammo
/// dispenser: sizes its bounds, indexes the model, makes it player-usable solid, reads the
/// `nodrain` spawn key into `genericValue12` and its `count` via [`EnergyAmmoStationSettings`],
/// wires `ammo_power_converter_use`/`check_recharge`, seeds the cgame health bar only when it
/// drains, then sets origin and links it. No oracle (entity-state spawn fn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_model_ammo_power_converter(ent: *mut gentity_t) {
    if (*ent).health == 0 {
        (*ent).health = 60;
    }

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, -16.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 16.0);

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).s.eFlags = 0;
    (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).clipmask = MASK_SOLID;

    G_SpawnInt(
        c"nodrain".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue12,
    );
    (*ent).r#use = Some(ammo_power_converter_use);

    EnergyAmmoStationSettings(ent);

    (*ent).genericValue4 = (*ent).count; //initial value
    (*ent).think = Some(check_recharge);

    if (*ent).genericValue12 == 0 {
        (*ent).s.maxhealth = (*ent).count;
        (*ent).s.health = (*ent).count;
    }
    (*ent).s.shouldtarget = QTRUE;
    (*ent).s.teamowner = 0;
    (*ent).s.owner = ENTITYNUM_NONE;

    (*ent).nextthink = (*addr_of!(level)).time + 200; // + STATION_RECHARGE_TIME;

    G_SetOrigin(ent, &(*ent).s.origin);
    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);
    trap::LinkEntity(ent);

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
}

/*QUAKED misc_model_health_power_converter (1 0 0) (-16 -16 -16) (16 16 16)
model="models/items/power_converter.md3"
Gives ammo energy when used.

"count" - the amount of ammo given when used (default 200)
*/
//------------------------------------------------------------
/// `void SP_misc_model_health_power_converter( gentity_t *ent )` (g_misc.c:1757). Spawns the
/// bacta/health station: sizes its bounds, indexes the model, makes it player-usable solid,
/// wires `health_power_converter_use`/`check_recharge`, reads `count` via
/// [`EnergyHealthStationSettings`], sets origin and links it, precaches the pickup/done sounds,
/// and (in Siege) broadcasts a radar icon. No oracle (entity-state spawn fn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_model_health_power_converter(ent: *mut gentity_t) {
    if (*ent).health == 0 {
        (*ent).health = 60;
    }

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, -16.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 16.0);

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).s.eFlags = 0;
    (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).clipmask = MASK_SOLID;

    (*ent).r#use = Some(health_power_converter_use);

    EnergyHealthStationSettings(ent);

    (*ent).genericValue4 = (*ent).count; //initial value
    (*ent).think = Some(check_recharge);

    //ent->s.maxhealth = ent->s.health = ent->count;
    (*ent).s.shouldtarget = QTRUE;
    (*ent).s.teamowner = 0;
    (*ent).s.owner = ENTITYNUM_NONE;

    (*ent).nextthink = (*addr_of!(level)).time + 200; // + STATION_RECHARGE_TIME;

    G_SetOrigin(ent, &(*ent).s.origin);
    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);
    trap::LinkEntity(ent);

    //G_SoundIndex("sound/movers/objects/useshieldstation.wav");
    G_SoundIndex("sound/player/pickuphealth.wav");
    (*ent).genericValue7 = G_SoundIndex("sound/interface/shieldcon_done");

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //show on radar from everywhere
        (*ent).r.svFlags |= SVF_BROADCAST;
        (*ent).s.eFlags |= EF_RADAROBJECT;
        (*ent).s.genericenemyindex = G_IconIndex("gfx/mp/siegeicons/desert/bacta");
    }
}

/// `void SP_misc_ammo_floor_unit( gentity_t *ent )` (g_misc.c:1290). Spawn function for the
/// floor-standing ammo recharge station: drops it to the floor via a downward trace (freeing
/// it if it starts in solid), defaults its `health`/`model`, makes it a player-usable solid,
/// applies the [`EnergyShieldStationSettings`] charge defaults, reads the `nodrain` spawn key,
/// wires `check_recharge`/`ammo_generic_power_converter_use`, seeds the cgame health bar unless
/// it doesn't drain, links it, precaches its run/done/empty sounds, and (in Siege) broadcasts a
/// radar icon. No oracle (entity-state spawn fn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_ammo_floor_unit(ent: *mut gentity_t) {
    let mut dest: vec3_t = [0.0; 3];

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, 0.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 40.0);

    (*ent).s.origin[2] += 0.1;
    (*ent).r.maxs[2] -= 0.1;

    VectorSet(
        &mut dest,
        (*ent).s.origin[0],
        (*ent).s.origin[1],
        (*ent).s.origin[2] - 4096.0,
    );
    let tr = trap::Trace(
        &(*ent).s.origin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &dest,
        (*ent).s.number,
        MASK_SOLID,
    );
    if tr.startsolid != 0 {
        G_Printf(&format!(
            "SP_misc_ammo_floor_unit: misc_ammo_floor_unit startsolid at {}\n",
            CStr::from_ptr(vtos(&(*ent).s.origin)).to_string_lossy()
        ));
        G_FreeEntity(ent);
        return;
    }

    //add the 0.1 back after the trace
    (*ent).r.maxs[2] += 0.1;

    // allow to ride movers
    (*ent).s.groundEntityNum = tr.entityNum as c_int;

    G_SetOrigin(ent, &tr.endpos);

    if (*ent).health == 0 {
        (*ent).health = 60;
    }

    if (*ent).model.is_null() || *(*ent).model == 0 {
        (*ent).model = c"/models/items/a_pwr_converter.md3".as_ptr() as *mut c_char;
    }

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).s.eFlags = 0;
    (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).clipmask = MASK_SOLID;

    EnergyShieldStationSettings(ent);

    (*ent).genericValue4 = (*ent).count; //initial value
    (*ent).think = Some(check_recharge);

    G_SpawnInt(
        c"nodrain".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue12,
    );

    if (*ent).genericValue12 == 0 {
        (*ent).s.maxhealth = (*ent).count;
        (*ent).s.health = (*ent).count;
    }
    (*ent).s.shouldtarget = QTRUE;
    (*ent).s.teamowner = 0;
    (*ent).s.owner = ENTITYNUM_NONE;

    (*ent).nextthink = (*addr_of!(level)).time + 200; // + STATION_RECHARGE_TIME;

    (*ent).r#use = Some(ammo_generic_power_converter_use);

    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);
    trap::LinkEntity(ent);

    G_SoundIndex("sound/interface/ammocon_run");
    (*ent).genericValue7 = G_SoundIndex("sound/interface/ammocon_done");
    G_SoundIndex("sound/interface/ammocon_empty");

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //show on radar from everywhere
        (*ent).r.svFlags |= SVF_BROADCAST;
        (*ent).s.eFlags |= EF_RADAROBJECT;
        (*ent).s.genericenemyindex = G_IconIndex("gfx/mp/siegeicons/desert/weapon_recharge");
    }
}

/*QUAKED misc_shield_floor_unit (1 0 0) (-16 -16 0) (16 16 40)
model="/models/items/a_shield_converter.md3"
Gives shield energy when used.

"count" - max charge value (default 50)
"chargerate" - rechage 1 point every this many milliseconds (default 3000)
"nodrain" - don't drain power from me
*/
/// `void SP_misc_shield_floor_unit( gentity_t *ent )` (g_misc.c:1377). Spawn function for the
/// floor-standing shield recharge station: only valid in CTF/CTY/Siege (freed otherwise), drops
/// it to the floor via a downward trace (freeing it if it starts in solid), defaults its
/// `health`/`model`, makes it a player-usable solid, applies the [`EnergyShieldStationSettings`]
/// charge defaults, reads the `nodrain` spawn key, wires `check_recharge`/`shield_power_converter_use`,
/// seeds the cgame health bar unless it doesn't drain, links it, precaches its run/done/empty
/// sounds, and (in Siege) broadcasts a radar icon. No oracle (entity-state spawn fn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_shield_floor_unit(ent: *mut gentity_t) {
    let mut dest: vec3_t = [0.0; 3];

    if (*addr_of!(g_gametype)).integer != GT_CTF
        && (*addr_of!(g_gametype)).integer != GT_CTY
        && (*addr_of!(g_gametype)).integer != GT_SIEGE
    {
        G_FreeEntity(ent);
        return;
    }

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, 0.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 40.0);

    (*ent).s.origin[2] += 0.1;
    (*ent).r.maxs[2] -= 0.1;

    VectorSet(
        &mut dest,
        (*ent).s.origin[0],
        (*ent).s.origin[1],
        (*ent).s.origin[2] - 4096.0,
    );
    let tr = trap::Trace(
        &(*ent).s.origin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &dest,
        (*ent).s.number,
        MASK_SOLID,
    );
    if tr.startsolid != 0 {
        G_Printf(&format!(
            "SP_misc_shield_floor_unit: misc_shield_floor_unit startsolid at {}\n",
            CStr::from_ptr(vtos(&(*ent).s.origin)).to_string_lossy()
        ));
        G_FreeEntity(ent);
        return;
    }

    //add the 0.1 back after the trace
    (*ent).r.maxs[2] += 0.1;

    // allow to ride movers
    (*ent).s.groundEntityNum = tr.entityNum as c_int;

    G_SetOrigin(ent, &tr.endpos);

    if (*ent).health == 0 {
        (*ent).health = 60;
    }

    if (*ent).model.is_null() || *(*ent).model == 0 {
        (*ent).model = c"/models/items/a_shield_converter.md3".as_ptr() as *mut c_char;
    }

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).s.eFlags = 0;
    (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).clipmask = MASK_SOLID;

    EnergyShieldStationSettings(ent);

    (*ent).genericValue4 = (*ent).count; //initial value
    (*ent).think = Some(check_recharge);

    G_SpawnInt(
        c"nodrain".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue12,
    );

    if (*ent).genericValue12 == 0 {
        (*ent).s.maxhealth = (*ent).count;
        (*ent).s.health = (*ent).count;
    }
    (*ent).s.shouldtarget = QTRUE;
    (*ent).s.teamowner = 0;
    (*ent).s.owner = ENTITYNUM_NONE;

    (*ent).nextthink = (*addr_of!(level)).time + 200; // + STATION_RECHARGE_TIME;

    (*ent).r#use = Some(shield_power_converter_use);

    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);
    trap::LinkEntity(ent);

    G_SoundIndex("sound/interface/shieldcon_run");
    (*ent).genericValue7 = G_SoundIndex("sound/interface/shieldcon_done");
    G_SoundIndex("sound/interface/shieldcon_empty");

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //show on radar from everywhere
        (*ent).r.svFlags |= SVF_BROADCAST;
        (*ent).s.eFlags |= EF_RADAROBJECT;
        (*ent).s.genericenemyindex = G_IconIndex("gfx/mp/siegeicons/desert/shield_recharge");
    }
}

/*QUAKED misc_portal_surface (0 0 1) (-8 -8 -8) (8 8 8)
The portal surface nearest this entity will show a view from the targeted misc_portal_camera, or a mirror view if untargeted.
This must be within 64 world units of the surface!
*/
/// `void SP_misc_portal_surface(gentity_t *ent)` (g_misc.c:355). The portal-surface spawner:
/// clears the bounds, links the entity, and tags it `SVF_PORTAL`/`ET_PORTAL`. If untargeted it
/// becomes a plain mirror (`origin2 = origin`); otherwise it schedules `locateCamera` to bind
/// to the targeted `misc_portal_camera`. No oracle (engine link / spawn plumbing + think-ptr).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_portal_surface(ent: *mut gentity_t) {
    VectorClear(&mut (*ent).r.mins);
    VectorClear(&mut (*ent).r.maxs);
    trap::LinkEntity(ent);

    (*ent).r.svFlags = SVF_PORTAL;
    (*ent).s.eType = ET_PORTAL;

    if (*ent).target.is_null() {
        VectorCopy(&(*ent).s.origin, &mut (*ent).s.origin2);
    } else {
        (*ent).think = Some(locateCamera);
        (*ent).nextthink = (*addr_of!(level)).time + 100;
    }
}

/*QUAKED misc_portal_camera (0 0 1) (-8 -8 -8) (8 8 8) slowrotate fastrotate noswing
The target for a misc_portal_director.  You can set either angles or target another entity to determine the direction of view.
"roll" an angle modifier to orient the camera around the target vector;
*/
/// `void SP_misc_portal_camera(gentity_t *ent)` (g_misc.c:375). The view target for a
/// `misc_portal_director`: clears the bounds, links the entity, and packs the `roll` spawn
/// key into `s.clientNum` as a 0..256 angle byte. No oracle (engine link / spawn plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_portal_camera(ent: *mut gentity_t) {
    let mut roll: f32 = 0.0;

    VectorClear(&mut (*ent).r.mins);
    VectorClear(&mut (*ent).r.maxs);
    trap::LinkEntity(ent);

    G_SpawnFloat(c"roll".as_ptr(), c"0".as_ptr(), &mut roll);

    (*ent).s.clientNum = (roll as f64 / 360.0 * 256.0) as c_int;
}

/*QUAKED misc_skyportal_orient (.6 .7 .7) (-8 -8 0) (8 8 16)
point from which to orient the sky portal cam in relation
to the regular view position.

"modelscale"			the scale at which to scale positions
*/
/// `void SP_misc_skyportal_orient( gentity_t *ent )` (g_misc.c:677). The orientation hint
/// for the sky portal cam is engine-side; the game module just frees the spawn entity.
/// No oracle (entity free).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_skyportal_orient(ent: *mut gentity_t) {
    G_FreeEntity(ent);
}

// void AddSpawnField(char *field, char *value);
const MAX_INSTANCE_TYPES: c_int = 16;

/// `void SP_terrain( gentity_t *ent )` (g_misc.c:486). RMG (Random Map Generator) terrain
/// spawner: forces the `RMG` cvar on, brush-models the entity, packs all the terrain
/// generation parameters (heightmap, patch counts, bounds, terrainDef, instances, etc.) into
/// an info string, registers it with the collision model (`trap_CM_RegisterTerrain`), pushes
/// the info string down to clients via the `CS_TERRAINS+terrainID` configstring, sets the
/// terrain contents/eFlags/eType, links the entity, and (when RMG is active) initialises the
/// terrain via `trap_RMG_Init`. The commented-out cvar-override and team-skin blocks are
/// preserved as comments exactly as in the upstream C. No oracle (all effects are trap
/// side-effects on engine state — cvars, configstrings, collision model — with no scalar
/// return; verified by inspection/compile like the other `SP_*` spawn fns).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_terrain(ent: *mut gentity_t) {
    let mut temp: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
    #[allow(unused_assignments)]
    let mut seed: String = String::new();
    #[allow(unused_assignments)]
    let mut missionType: String = String::new();
    let mut value: *mut c_char = core::ptr::null_mut();

    //Force it to 1 when there is terrain on the level.
    trap::Cvar_Set("RMG", "1");
    g_RMG.integer = 1;

    VectorClear(&mut (*ent).s.angles);
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    // Get the shader from the top of the brush
    //	shaderNum = gi.CM_GetShaderNum(s.modelindex);
    let shaderNum: c_int = 0;

    if g_RMG.integer != 0 {
        /*
        // Grab the default terrain file from the RMG cvar
        trap_Cvar_VariableStringBuffer("RMG_terrain", temp, MAX_QPATH);
        Com_sprintf(final, MAX_QPATH, "%s", temp);
        AddSpawnField("terrainDef", temp);

        trap_Cvar_VariableStringBuffer("RMG_instances", temp, MAX_QPATH);
        Com_sprintf(final, MAX_QPATH, "%s", temp);
        AddSpawnField("instanceDef", temp);

        trap_Cvar_VariableStringBuffer("RMG_miscents", temp, MAX_QPATH);
        Com_sprintf(final, MAX_QPATH, "%s", temp);
        AddSpawnField("miscentDef", temp);
        */
        //rww - disabled for now, don't want cvar overrides.

        seed = trap::Cvar_VariableString("RMG_seed");
        missionType = trap::Cvar_VariableString("RMG_mission");

        //rww - May want to implement these at some point.
        //trap_Cvar_VariableStringBuffer("RMG_soundset", soundSet, MAX_QPATH);
        //trap_SetConfigstring(CS_AMBIENT_SOUNDSETS, soundSet );
    }

    // Get info required for the common init
    temp[0] = 0;
    G_SpawnString(c"heightmap".as_ptr(), c"".as_ptr(), &mut value);
    Info_SetValueForKey(temp.as_mut_ptr(), c"heightMap".as_ptr(), value);

    G_SpawnString(c"numpatches".as_ptr(), c"400".as_ptr(), &mut value);
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"numPatches".as_ptr(),
        va(format_args!("{}", atoi(value))),
    );

    G_SpawnString(c"terxels".as_ptr(), c"4".as_ptr(), &mut value);
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"terxels".as_ptr(),
        va(format_args!("{}", atoi(value))),
    );

    let seedc = std::ffi::CString::new(seed).unwrap_or_default();
    Info_SetValueForKey(temp.as_mut_ptr(), c"seed".as_ptr(), seedc.as_ptr());
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"minx".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.mins[0])),
    );
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"miny".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.mins[1])),
    );
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"minz".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.mins[2])),
    );
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"maxx".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.maxs[0])),
    );
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"maxy".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.maxs[1])),
    );
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"maxz".as_ptr(),
        va(format_args!("{:.6}", (*ent).r.maxs[2])),
    );

    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"modelIndex".as_ptr(),
        va(format_args!("{}", (*ent).s.modelindex)),
    );

    G_SpawnString(c"terraindef".as_ptr(), c"grassyhills".as_ptr(), &mut value);
    Info_SetValueForKey(temp.as_mut_ptr(), c"terrainDef".as_ptr(), value);

    G_SpawnString(c"instancedef".as_ptr(), c"".as_ptr(), &mut value);
    Info_SetValueForKey(temp.as_mut_ptr(), c"instanceDef".as_ptr(), value);

    G_SpawnString(c"miscentdef".as_ptr(), c"".as_ptr(), &mut value);
    Info_SetValueForKey(temp.as_mut_ptr(), c"miscentDef".as_ptr(), value);

    let missionTypec = std::ffi::CString::new(missionType).unwrap_or_default();
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"missionType".as_ptr(),
        missionTypec.as_ptr(),
    );

    let mut i: c_int = 0;
    while i < MAX_INSTANCE_TYPES {
        let r#final = trap::Cvar_VariableString(&format!("RMG_instance{}", i));
        if !r#final.is_empty() {
            let finalc = std::ffi::CString::new(r#final).unwrap_or_default();
            Info_SetValueForKey(
                temp.as_mut_ptr(),
                va(format_args!("inst{}", i)),
                finalc.as_ptr(),
            );
        }
        i += 1;
    }

    // Set additional data required on the client only
    G_SpawnString(c"densitymap".as_ptr(), c"".as_ptr(), &mut value);
    Info_SetValueForKey(temp.as_mut_ptr(), c"densityMap".as_ptr(), value);

    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"shader".as_ptr(),
        va(format_args!("{}", shaderNum)),
    );
    G_SpawnString(c"texturescale".as_ptr(), c"0.005".as_ptr(), &mut value);
    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"texturescale".as_ptr(),
        va(format_args!("{:.6}", atof(value))),
    );

    // Initialise the common aspects of the terrain
    let terrainID: c_int =
        trap::CM_RegisterTerrain(&CStr::from_ptr(temp.as_ptr()).to_string_lossy());
    //	SetCommon(common);

    Info_SetValueForKey(
        temp.as_mut_ptr(),
        c"terrainId".as_ptr(),
        va(format_args!("{}", terrainID)),
    );

    // Let the entity know if it is random generated or not
    //	SetIsRandom(common->GetIsRandom());

    // Let the game remember everything
    //level.landScapes[terrainID] = ent; //rww - also not referenced

    // Send all the data down to the client
    trap::SetConfigstring(
        CS_TERRAINS + terrainID,
        &CStr::from_ptr(temp.as_ptr()).to_string_lossy(),
    );

    // Make sure the contents are properly set
    (*ent).r.contents = CONTENTS_TERRAIN;
    (*ent).r.svFlags = SVF_NOCLIENT;
    (*ent).s.eFlags = EF_PERMANENT;
    (*ent).s.eType = ET_TERRAIN;

    // Hook into the world so physics will work
    trap::LinkEntity(ent);

    // If running RMG then initialize the terrain and handle team skins
    if g_RMG.integer != 0 {
        trap::RMG_Init(terrainID);

        /*
        if ( level.gametypeData->teams )
        {
            char temp[MAX_QPATH];

            // Red team change from RMG ?
            trap_GetConfigstring ( CS_GAMETYPE_REDTEAM, temp, MAX_QPATH );
            if ( Q_stricmp ( temp, level.gametypeTeam[TEAM_RED] ) )
            {
                level.gametypeTeam[TEAM_RED] = trap_VM_LocalStringAlloc ( temp );
            }

            // Blue team change from RMG ?
            trap_GetConfigstring ( CS_GAMETYPE_BLUETEAM, temp, MAX_QPATH );
            if ( Q_stricmp ( temp, level.gametypeTeam[TEAM_BLUE] ) )
            {
                level.gametypeTeam[TEAM_BLUE] = trap_VM_LocalStringAlloc ( temp );
            }
        }
        */
    }
}

/*QUAKED misc_skyportal (.6 .7 .7) (-8 -8 0) (8 8 16)
"fov" for the skybox default is 80
To have the portal sky fogged, enter any of the following values:
"onlyfoghere" if non-0 allows you to set a global fog, but will only use that fog within this sky portal.

Also note that entities in the same PVS and visible (via point trace) from this
object will be flagged as portal entities. This means they will be sent and
updated from the server for every client every update regardless of where
they are, and they will essentially be added to the scene twice if the client
is in the same PVS as them (only once otherwise, but still once no matter
where the client is). In other words, don't go overboard with it or everything
will explode.
*/
/// `void SP_misc_skyportal( gentity_t *ent )` (g_misc.c:696). The skybox-portal spawner: reads
/// the `fov`/fog spawn keys, pushes the packed skybox origin string to the `CS_SKYBOXORG`
/// configstring, then schedules `G_PortalifyEntities` (after a delay so all other entities have
/// spawned) to flag in-PVS entities as portal entities. No oracle (spawn plumbing +
/// configstring + think-ptr).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_skyportal(ent: *mut gentity_t) {
    let mut fov: *mut c_char = core::ptr::null_mut();
    let mut fogv: vec3_t = [0.0; 3]; //----(SA)
    let mut fogn: c_int = 0; //----(SA)
    let mut fogf: c_int = 0; //----(SA)
    let mut isfog: c_int = 0; // (SA)

    let fov_x: f32;

    G_SpawnString(c"fov".as_ptr(), c"80".as_ptr(), &mut fov);
    fov_x = atof(fov) as f32;

    isfog += G_SpawnVector(c"fogcolor".as_ptr(), c"0 0 0".as_ptr(), fogv.as_mut_ptr());
    isfog += G_SpawnInt(c"fognear".as_ptr(), c"0".as_ptr(), &mut fogn);
    isfog += G_SpawnInt(c"fogfar".as_ptr(), c"300".as_ptr(), &mut fogf);

    trap::SetConfigstring(
        CS_SKYBOXORG,
        &format!(
            "{:.2} {:.2} {:.2} {:.1} {} {:.2} {:.2} {:.2} {} {}",
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2],
            fov_x,
            isfog,
            fogv[0],
            fogv[1],
            fogv[2],
            fogn,
            fogf
        ),
    );

    (*ent).think = Some(G_PortalifyEntities);
    (*ent).nextthink = (*addr_of!(level)).time + 1050; //give it some time first so that all other entities are spawned.
}

/// `void fx_runner_think( gentity_t *ent )` (g_misc.c:2041). Per-frame think for an
/// `fx_runner` (env_effect): re-evaluates the entity's position/orientation trajectories,
/// flags the client-side effect as continuous, schedules the next think, and optionally
/// deals radius damage, fires `target2`, and (re)arms the looping soundset. The two
/// `G_AddEvent`/`EV_PLAY_*EFFECT_ID` calls are commented out in the upstream C — preserved
/// here as comments. No oracle (mutates the entity, schedules thinks, deals damage).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn fx_runner_think(ent: *mut gentity_t) {
    BG_EvaluateTrajectory(
        &(*ent).s.pos,
        (*addr_of!(level)).time,
        &mut (*ent).r.currentOrigin,
    );
    BG_EvaluateTrajectory(
        &(*ent).s.apos,
        (*addr_of!(level)).time,
        &mut (*ent).r.currentAngles,
    );

    // call the effect with the desired position and orientation
    if (*ent).s.isPortalEnt != QFALSE {
        //		G_AddEvent( ent, EV_PLAY_PORTAL_EFFECT_ID, ent->genericValue5 );
    } else {
        //		G_AddEvent( ent, EV_PLAY_EFFECT_ID, ent->genericValue5 );
    }

    // start the fx on the client (continuous)
    (*ent).s.modelindex2 = FX_STATE_CONTINUOUS;

    VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.angles);
    VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.origin);

    // nextthink = level.time + ent->delay + random() * ent->random;
    // (Faithful to C: the int sum is promoted to float by random()*ent->random,
    //  then truncated back to int on store.)
    (*ent).nextthink =
        (((*addr_of!(level)).time + (*ent).delay) as f32 + random() * (*ent).random) as c_int;

    if (*ent).spawnflags & 4 != 0
    // damage
    {
        G_RadiusDamage(
            &(*ent).r.currentOrigin,
            ent,
            (*ent).splashDamage as f32,
            (*ent).splashRadius as f32,
            ent,
            ent,
            MOD_UNKNOWN,
        );
    }

    if !(*ent).target2.is_null() && *(*ent).target2 != 0 {
        // let our target know that we have spawned an effect
        G_UseTargets2(ent, ent, (*ent).target2);
    }

    if (*ent).spawnflags & 2 == 0 && (*ent).s.loopSound == 0 {
        // NOT ONESHOT...this is an assy thing to do
        if !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
            (*ent).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*ent).soundSet).to_string_lossy());
            (*ent).s.loopIsSoundset = QTRUE;
            (*ent).s.loopSound = BMS_MID;
        }
    }
}

/// `void fx_runner_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:2088). Use callback for an `fx_runner`. If inside a sky portal area, marks the
/// entity broadcast on first use. For ONESHOT runners it fires the effect once (via
/// `fx_runner_think`), parks `nextthink` at -1, sets the one-shot client state, optionally
/// re-fires `target2` and plays the start soundset event. Otherwise it toggles the
/// continuous effect on/off, (re)arming or silencing the looping soundset accordingly. No
/// oracle (mutates the entity, schedules thinks, fires events/effects).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`. `other`/`activator` are unused but kept for the
/// `use` fn-pointer ABI.
pub unsafe extern "C" fn fx_runner_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    if (*self_).s.isPortalEnt != QFALSE {
        //rww - mark it as broadcast upon first use if it's within the area of a skyportal
        (*self_).r.svFlags |= SVF_BROADCAST;
    }

    if (*self_).spawnflags & 2 != 0
    // ONESHOT
    {
        // call the effect with the desired position and orientation, as a safety thing,
        //	make sure we aren't thinking at all.
        let saveState: c_int = (*self_).s.modelindex2 + 1;

        fx_runner_think(self_);
        (*self_).nextthink = -1;
        // one shot indicator
        (*self_).s.modelindex2 = saveState;
        if (*self_).s.modelindex2 > FX_STATE_ONE_SHOT_LIMIT {
            (*self_).s.modelindex2 = FX_STATE_ONE_SHOT;
        }

        if !(*self_).target2.is_null() {
            // let our target know that we have spawned an effect
            G_UseTargets2(self_, self_, (*self_).target2);
        }

        if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
            (*self_).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*self_).soundSet).to_string_lossy());
            G_AddEvent(self_, EV_BMODEL_SOUND, BMS_START);
        }
    } else {
        // ensure we are working with the right think function
        (*self_).think = Some(fx_runner_think);

        // toggle our state
        if (*self_).nextthink == -1 {
            // NOTE: we fire the effect immediately on use, the fx_runner_think func will set
            //	up the nextthink time.
            fx_runner_think(self_);

            if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
                (*self_).s.soundSetIndex =
                    G_SoundSetIndex(&CStr::from_ptr((*self_).soundSet).to_string_lossy());
                G_AddEvent(self_, EV_BMODEL_SOUND, BMS_START);
                (*self_).s.loopSound = BMS_MID;
                (*self_).s.loopIsSoundset = QTRUE;
            }
        } else {
            // turn off for now
            (*self_).nextthink = -1;

            // turn off fx on client
            (*self_).s.modelindex2 = FX_STATE_OFF;

            if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
                (*self_).s.soundSetIndex =
                    G_SoundSetIndex(&CStr::from_ptr((*self_).soundSet).to_string_lossy());
                G_AddEvent(self_, EV_BMODEL_SOUND, BMS_END);
                (*self_).s.loopSound = 0;
                (*self_).s.loopIsSoundset = QFALSE;
            }
        }
    }
}

/// `void fx_runner_link( gentity_t *ent )` (g_misc.c:2162). Deferred link think for an
/// `fx_runner`: if a `target` is set it tries to orient the effect toward that entity
/// (falling back to the default UP angle with a warning), validates `target2`, and applies
/// the angles. For STARTOFF/ONESHOT runners it parks `nextthink` at -1 (waits to be used);
/// otherwise it (re)arms the looping soundset and starts the continuous think shortly. If the
/// runner can be targeted it installs `fx_runner_use` as its use callback. No oracle (mutates
/// the entity, schedules thinks, walks the entity list via `G_Find`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn fx_runner_link(ent: *mut gentity_t) {
    let mut dir: vec3_t = [0.0; 3];

    if !(*ent).target.is_null() && *(*ent).target != 0 {
        // try to use the target to override the orientation
        let target: *mut gentity_t = G_Find(
            core::ptr::null_mut(),
            offset_of!(gentity_t, targetname),
            (*ent).target,
        );

        if target.is_null() {
            // Bah, no good, dump a warning, but continue on and use the UP vector
            Com_Printf(&format!(
                "fx_runner_link: target specified but not found: {}\n",
                CStr::from_ptr((*ent).target).to_string_lossy()
            ));
            Com_Printf("  -assuming UP orientation.\n");
        } else {
            // Our target is valid so let's override the default UP vector
            VectorSubtract(&(*target).s.origin, &(*ent).s.origin, &mut dir);
            VectorNormalize(&mut dir);
            vectoangles(&dir, &mut (*ent).s.angles);
        }
    }

    // don't really do anything with this right now other than do a check to warn the designers if the target2 is bogus
    if !(*ent).target2.is_null() && *(*ent).target2 != 0 {
        let target: *mut gentity_t = G_Find(
            core::ptr::null_mut(),
            offset_of!(gentity_t, targetname),
            (*ent).target2,
        );

        if target.is_null() {
            // Target2 is bogus, but we can still continue
            Com_Printf(&format!(
                "fx_runner_link: target2 was specified but is not valid: {}\n",
                CStr::from_ptr((*ent).target2).to_string_lossy()
            ));
        }
    }

    G_SetAngles(ent, &(*ent).s.angles);

    if (*ent).spawnflags & 1 != 0 || (*ent).spawnflags & 2 != 0
    // STARTOFF || ONESHOT
    {
        // We won't even consider thinking until we are used
        (*ent).nextthink = -1;
    } else {
        if !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
            (*ent).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*ent).soundSet).to_string_lossy());
            (*ent).s.loopSound = BMS_MID;
            (*ent).s.loopIsSoundset = QTRUE;
        }

        // Let's get to work right now!
        (*ent).think = Some(fx_runner_think);
        (*ent).nextthink = (*addr_of!(level)).time + 200; // wait a small bit, then start working
    }

    // make us useable if we can be targeted
    if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
        (*ent).r#use = Some(fx_runner_use);
    }
}

/// `#define FX_ENT_RADIUS 32` (g_misc.c:2036). Half-extent of an `fx_runner`'s bounding box.
const FX_ENT_RADIUS: c_int = 32;

/// `void SP_fx_runner( gentity_t *ent )` (g_misc.c:2231). Spawn function for `fx_runner`
/// (env_effect): parses the `fxFile`, timing (`delay`/`random`) and damage
/// (`splashRadius`/`splashDamage`) keys, defaults the orientation to UP when no angles were
/// given, registers the effect index, fills in the transmitted entity-state fields
/// (`ET_FX`/speed/time/off state), schedules the deferred `fx_runner_link` think, sets the
/// origin and a small bounding box, and links the entity. Frees the entity with an error if
/// no `fxFile` was specified. No oracle (spawn parsing, effect registration, entity link).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`. Must run during spawn so `G_Spawn*` reads the
/// active spawn vars.
pub unsafe extern "C" fn SP_fx_runner(ent: *mut gentity_t) {
    let mut fxFile: *mut c_char = core::ptr::null_mut();

    G_SpawnString(c"fxFile".as_ptr(), c"".as_ptr(), &mut fxFile);
    // Get our defaults
    G_SpawnInt(c"delay".as_ptr(), c"200".as_ptr(), &mut (*ent).delay);
    G_SpawnFloat(c"random".as_ptr(), c"0".as_ptr(), &mut (*ent).random);
    G_SpawnInt(
        c"splashRadius".as_ptr(),
        c"16".as_ptr(),
        &mut (*ent).splashRadius,
    );
    G_SpawnInt(
        c"splashDamage".as_ptr(),
        c"5".as_ptr(),
        &mut (*ent).splashDamage,
    );

    if (*ent).s.angles[0] == 0.0 && (*ent).s.angles[1] == 0.0 && (*ent).s.angles[2] == 0.0 {
        // didn't have angles, so give us the default of up
        VectorSet(&mut (*ent).s.angles, -90.0, 0.0, 0.0);
    }

    if fxFile.is_null() || *fxFile == 0 {
        Com_Printf(&format!(
            // S_COLOR_RED
            "^1ERROR: fx_runner {} at {} has no fxFile specified\n",
            if (*ent).targetname.is_null() {
                "(null)".into()
            } else {
                CStr::from_ptr((*ent).targetname).to_string_lossy()
            },
            CStr::from_ptr(vtos(&(*ent).s.origin)).to_string_lossy()
        ));
        G_FreeEntity(ent);
        return;
    }

    // Try and associate an effect file, unfortunately we won't know if this worked or not
    //	until the CGAME trys to register it...
    (*ent).s.modelindex = G_EffectIndex(&CStr::from_ptr(fxFile).to_string_lossy());

    // important info transmitted
    (*ent).s.eType = ET_FX;
    (*ent).s.speed = (*ent).delay as f32;
    (*ent).s.time = (*ent).random as c_int;
    (*ent).s.modelindex2 = FX_STATE_OFF;

    // Give us a bit of time to spawn in the other entities, since we may have to target one of 'em
    (*ent).think = Some(fx_runner_link);
    (*ent).nextthink = (*addr_of!(level)).time + 400;

    // Save our position and link us up!
    G_SetOrigin(ent, &(*ent).s.origin);

    VectorSet(
        &mut (*ent).r.maxs,
        FX_ENT_RADIUS as f32,
        FX_ENT_RADIUS as f32,
        FX_ENT_RADIUS as f32,
    );
    VectorScale(&(*ent).r.maxs, -1.0, &mut (*ent).r.mins);

    trap::LinkEntity(ent);
}

/// `void ref_link( gentity_t *ent )` (g_misc.c:3048). Deferred think for a `ref_tag`: if it
/// `target`s an entity, resolves that entity by `targetname` via `G_Find` and points the
/// tag's angles toward it (else prints a red error), registers the tag in the global tag pool
/// via [`TAG_Add`] (radius 16, no flags), then frees the entity immediately — once linked a
/// `ref_tag` can never be referred to as an entity again. This lives in the upstream
/// `#ifndef _XBOX` block, which the server build compiles. No oracle (`G_Find`/`TAG_Add`/
/// `G_FreeEntity` + entity-state plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn ref_link(ent: *mut gentity_t) {
    let tag: *mut reference_tag_t;

    if !(*ent).target.is_null() {
        //TODO: Find the target and set our angles to that direction
        let target: *mut gentity_t = G_Find(
            core::ptr::null_mut(),
            offset_of!(gentity_t, targetname),
            (*ent).target,
        );
        let mut dir: vec3_t = [0.0; 3];

        if !target.is_null() {
            //Find the direction to the target
            VectorSubtract(&(*target).s.origin, &(*ent).s.origin, &mut dir);
            VectorNormalize(&mut dir);
            vectoangles(&dir, &mut (*ent).s.angles);

            //FIXME: Does pitch get flipped?
        } else {
            Com_Printf(&format!(
                "{}ERROR: ref_tag ({}) has invalid target ({})",
                "^1", // S_COLOR_RED
                CStr::from_ptr((*ent).targetname).to_string_lossy(),
                CStr::from_ptr((*ent).target).to_string_lossy()
            ));
        }
    }

    //Add the tag
    tag = TAG_Add(
        (*ent).targetname,
        (*ent).ownername,
        &(*ent).s.origin,
        &(*ent).s.angles,
        16,
        0,
    );
    let _ = tag;

    //Delete immediately, cannot be refered to as an entity again
    //NOTE: this means if you wanted to link them in a chain for, say, a path, you can't
    G_FreeEntity(ent);
}

/// `void SP_reference_tag( gentity_t *ent )` (g_misc.c:3083). Spawner for a `reference_tag`:
/// if it `target`s another entity its linking must wait until all entities have spawned, so
/// it schedules [`ref_link`] as a deferred think (`START_TIME_LINK_ENTS` after map start);
/// otherwise it links immediately. Ported from the retail-PC source (the Xbox copy was an
/// `assert(0)` stub with the real body commented out). No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_reference_tag(ent: *mut gentity_t) {
    if !(*ent).target.is_null() {
        //Init cannot occur until all entities have been spawned
        (*ent).think = Some(ref_link);
        (*ent).nextthink = (*addr_of!(level)).time + START_TIME_LINK_ENTS;
    } else {
        ref_link(ent);
    }
}

/// `void misc_weapon_shooter_fire( gentity_t *self )` (g_misc.c:3180). Think/fire callback
/// for the (PC-only, `#ifndef _XBOX`) weapon-shooter: fires the shooter's weapon (alt-fire
/// when `spawnflags&1`), and if `spawnflags&2` (repeat) reschedules itself to fire again after
/// `wait` seconds.
///
/// No oracle: mutates entity state (`think`/`nextthink`) and fires a weapon via `FireWeapon`.
///
/// # Safety
/// `self` must point to a valid `gentity_t`.
pub unsafe extern "C" fn misc_weapon_shooter_fire(self_: *mut gentity_t) {
    FireWeapon(self_, (*self_).spawnflags & 1);
    if (*self_).spawnflags & 2 != 0 {
        //repeat
        (*self_).think = Some(misc_weapon_shooter_fire);
        (*self_).nextthink = ((*addr_of!(level)).time as f32 + (*self_).wait) as c_int;
    }
}

/// `void misc_weapon_shooter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_misc.c:3190). Use callback for the (PC-only, `#ifndef _XBOX`) weapon-shooter. If the
/// shooter is already in repeating-fire mode (its `think` is `misc_weapon_shooter_fire`),
/// stops it by clearing `nextthink`; otherwise it fires once.
///
/// No oracle: compares the `think` fn-pointer and mutates entity state / fires a weapon.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`. `other`/`activator` are unused but kept for the
/// `use` fn-pointer ABI.
pub unsafe extern "C" fn misc_weapon_shooter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    if let Some(think) = (*self_).think {
        if core::ptr::fn_addr_eq(
            think,
            misc_weapon_shooter_fire as unsafe extern "C" fn(*mut gentity_t),
        ) {
            //repeating fire, stop
            /*
            G_FreeClientForShooter(self->client);
            self->think = G_FreeEntity;
            self->nextthink = level.time;
            */
            (*self_).nextthink = 0;
            return;
        }
    }
    //otherwise, fire
    misc_weapon_shooter_fire(self_);
}

/// `void misc_weapon_shooter_aim( gentity_t *self )` (g_misc.c:3206). Think callback for the
/// (PC-only, `#ifndef _XBOX`) weapon-shooter: re-aims at its `target` each frame. Resolves
/// the target by `targetname` via `G_Find`, stores it as `enemy`, points the shooter's
/// view at the target's current origin (`vectoangles` → client viewangles →
/// `SetClientViewAngle`), and reschedules next frame. Drops the enemy if the target is gone.
///
/// No oracle: mutates entity/client state and reaches engine plumbing (`G_Find`,
/// `SetClientViewAngle`).
///
/// # Safety
/// `self` must point to a valid `gentity_t` with a non-NULL `client` when `target` is set.
pub unsafe extern "C" fn misc_weapon_shooter_aim(self_: *mut gentity_t) {
    //update my aim
    if !(*self_).target.is_null() {
        let targ = G_Find(
            core::ptr::null_mut(),
            offset_of!(gentity_t, targetname),
            (*self_).target,
        );
        if !targ.is_null() {
            (*self_).enemy = targ;
            let mut tmp: vec3_t = [0.0; 3];
            VectorSubtract(
                &(*targ).r.currentOrigin,
                &(*self_).r.currentOrigin,
                &mut tmp,
            );
            (*self_).pos1 = tmp;
            VectorCopy(&(*targ).r.currentOrigin, &mut (*self_).pos1);
            let mut viewangles = (*(*self_).client).ps.viewangles;
            vectoangles(&(*self_).pos1, &mut viewangles);
            (*(*self_).client).ps.viewangles = viewangles;
            SetClientViewAngle(self_, &(*(*self_).client).ps.viewangles);
            //FIXME: don't keep doing this unless target is a moving target?
            (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
        } else {
            (*self_).enemy = core::ptr::null_mut();
        }
    }
}

// kind of hacky, but we have to do this with no dynamic allocation
/// `#define MAX_SHOOTERS 16` (g_misc.c:3346) — size of the static shooter-client pool.
const MAX_SHOOTERS: usize = 16;

/// `typedef struct shooterClient_s { gclient_t cl; qboolean inuse; } shooterClient_t;`
/// (g_misc.c:3347) — a borrowed `gclient_t` slot the weapon-shooter code drives, since the
/// module has no dynamic allocation.
#[repr(C)]
struct shooterClient_t {
    cl: gclient_t,
    inuse: qboolean,
}

/// `static shooterClient_t g_shooterClients[MAX_SHOOTERS]` (g_misc.c:3351) + `static qboolean
/// g_shooterClientInit` (g_misc.c:3352). Zero-initialised file-local pool handed out by
/// [`G_ClientForShooter`]; `g_shooterClientInit` gates the one-shot `memset` clear.
static mut g_shooterClients: [shooterClient_t; MAX_SHOOTERS] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
static mut g_shooterClientInit: qboolean = QFALSE;

/// `gclient_t *G_ClientForShooter( void )` (g_misc.c:3354). Hands out the next free slot in the
/// static [`g_shooterClients`] pool (clearing it once on first use); `Com_Error(ERR_DROP)` on
/// exhaustion.
///
/// No oracle (file-static pool + `Com_Error` engine plumbing).
pub unsafe fn G_ClientForShooter() -> *mut gclient_t {
    if (*addr_of!(g_shooterClientInit)) == QFALSE {
        // in theory it should be initialized to 0 on the stack, but just in case.
        core::ptr::write_bytes(
            addr_of_mut!(g_shooterClients) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(g_shooterClients)),
        );
        *addr_of_mut!(g_shooterClientInit) = QTRUE;
    }

    let mut i = 0;
    while i < MAX_SHOOTERS {
        if (*addr_of!(g_shooterClients))[i].inuse == QFALSE {
            return addr_of_mut!((*addr_of_mut!(g_shooterClients))[i].cl);
        }
        i += 1;
    }

    Com_Error(ERR_DROP, "No free shooter clients - hit MAX_SHOOTERS");
}

/// `void G_FreeClientForShooter( gclient_t *cl )` (g_misc.c:3377) — releases the
/// [`g_shooterClients`] pool slot whose embedded `cl` matches, marking it `inuse = qfalse`.
///
/// No oracle (mutates the file-static shooter pool, not oracle-testable).
pub unsafe fn G_FreeClientForShooter(cl: *mut gclient_t) {
    let mut i = 0;
    while i < MAX_SHOOTERS {
        if addr_of_mut!((*addr_of_mut!(g_shooterClients))[i].cl) == cl {
            (*addr_of_mut!(g_shooterClients))[i].inuse = QFALSE;
            return;
        }
        i += 1;
    }
}

/*QUAKED misc_weapon_shooter (1 0 0) (-8 -8 -8) (8 8 8) ALTFIRE TOGGLE
ALTFIRE - fire the alt-fire of the chosen weapon
TOGGLE - keep firing until used again (fires at intervals of "wait")

"wait" - debounce time between refires (defaults to 500)

"target" - what to aim at (will update aim every frame if it's a moving target)

"weapon" - specify the weapon to use (default is WP_BLASTER)
    WP_BRYAR_PISTOL
    WP_BLASTER
    WP_DISRUPTOR
    WP_BOWCASTER
    WP_REPEATER
    WP_DEMP2
    WP_FLECHETTE
    WP_ROCKET_LAUNCHER
    WP_THERMAL
    WP_TRIP_MINE
    WP_DET_PACK
    WP_STUN_BATON
    WP_EMPLACED_GUN
    WP_BOT_LASER
    WP_TURRET
    WP_ATST_MAIN
    WP_ATST_SIDE
    WP_TIE_FIGHTER
    WP_RAPID_FIRE_CONC
    WP_BLASTER_PISTOL
*/
/// `void SP_misc_weapon_shooter( gentity_t *self )` (g_misc.c:3444). PC-tree body (the Xbox
/// tree `assert(0)`-stubbed it with "Removed by BTO - never used!?"; the PC retail source ships
/// the real implementation, so this is a PC-vs-Xbox reconciliation). Borrows a [`gclient_t`]
/// from the shooter pool, picks the weapon (default `WP_BLASTER`, or the `"weapon"` key via
/// [`GetIDForString`]/[`WPTable`]), registers its item, sets the muzzle point, and wires the
/// `think`/`use` callbacks (aim at `target` each frame if set, else a static aim angle).
/// No oracle (entity/client spawn plumbing).
///
/// # Safety
/// `self` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_weapon_shooter(self_: *mut gentity_t) {
    let mut s: *mut c_char = core::ptr::null_mut();

    //alloc a client just for the weapon code to use
    (*self_).client = G_ClientForShooter();

    G_SpawnString(c"weapon".as_ptr(), c"".as_ptr(), &mut s);

    //set weapon
    (*self_).s.weapon = WP_BLASTER;
    (*(*self_).client).ps.weapon = WP_BLASTER;
    if !s.is_null() && *s != 0 {
        //use a different weapon
        let w = GetIDForString(addr_of!(WPTable).cast(), s);
        (*self_).s.weapon = w;
        (*(*self_).client).ps.weapon = w;
    }

    RegisterItem(BG_FindItemForWeapon((*self_).s.weapon));

    //set where our muzzle is
    VectorCopy(
        &(*self_).s.origin,
        &mut (*(*self_).client).renderInfo.muzzlePoint,
    );
    //permanently updated (don't need for MP)
    //self->client->renderInfo.mPCalcTime = Q3_INFINITE;

    //set up to link
    if !(*self_).target.is_null() {
        (*self_).think = Some(misc_weapon_shooter_aim);
        (*self_).nextthink = (*addr_of!(level)).time + START_TIME_LINK_ENTS;
    } else {
        //just set aim angles
        VectorCopy(&(*self_).s.angles, &mut (*(*self_).client).ps.viewangles);
        AngleVectors(&(*self_).s.angles, Some(&mut (*self_).pos1), None, None);
    }

    //set up to fire when used
    (*self_).r#use = Some(misc_weapon_shooter_use);

    if (*self_).wait == 0.0 {
        (*self_).wait = 500.0;
    }
}

/*QUAKED misc_weather_zone (0 .5 .8) ?
Determines a region to check for weather contents - will significantly reduce load time
*/
/// `void SP_misc_weather_zone( gentity_t *ent )` (g_misc.c:3284). The weather-contents
/// region is consumed engine-side; the game module just frees the spawn entity. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_misc_weather_zone(ent: *mut gentity_t) {
    G_FreeEntity(ent);
}

// #ifndef _XBOX	// Removing unused ref_tag support!
//
// rww - ref tag stuff ported from SP (and C-ified)
/// `#define TAG_GENERIC_NAME "__WORLD__"` (g_misc.c). If a designer chooses this name, cut
/// a finger off as an example to the others. The fallback owner for `TAG_Find`.
const TAG_GENERIC_NAME: &CStr = c"__WORLD__";

// MAX_TAG_OWNERS is 16 for now in order to not use too much VM memory.
// Each tag owner has preallocated space for tags up to MAX_TAGS.
// As is this means 16*256 sizeof(reference_tag_t)'s in addition to name+inuse*16.
/// `#define MAX_TAGS 256` (g_misc.c:2650).
const MAX_TAGS: usize = 256;
/// `#define MAX_TAG_OWNERS 16` (g_misc.c:2651).
const MAX_TAG_OWNERS: usize = 16;

// Maybe I should use my trap_TrueMalloc/trap_TrueFree stuff with this.
// But I am not yet confident that it can be used without exploding at some point.

/// `typedef struct tagOwner_s { … } tagOwner_t;` (g_misc.c:2656). File-local ref-tag owner
/// record: a name, a preallocated `reference_tag_t[MAX_TAGS]` pool, and an in-use flag.
/// Pointer-free; not crossing the engine ABI.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct tagOwner_t {
    pub name: [c_char; MAX_REFNAME],
    pub tags: [reference_tag_t; MAX_TAGS],
    pub inuse: qboolean,
}

/// Zeroed `reference_tag_t` for the const array initializer (C global zero-init).
const REFERENCE_TAG_ZERO: reference_tag_t = reference_tag_t {
    name: [0; MAX_REFNAME],
    origin: [0.0; 3],
    angles: [0.0; 3],
    flags: 0,
    radius: 0,
    inuse: QFALSE,
};

/// `tagOwner_t refTagOwnerMap[MAX_TAG_OWNERS];` (g_misc.c:2663). File-scope owner pool,
/// zero-initialized like the C global (BSS).
static mut refTagOwnerMap: [tagOwner_t; MAX_TAG_OWNERS] = [tagOwner_t {
    name: [0; MAX_REFNAME],
    tags: [REFERENCE_TAG_ZERO; MAX_TAGS],
    inuse: QFALSE,
}; MAX_TAG_OWNERS];

/// `tagOwner_t *FirstFreeTagOwner(void)` (g_misc.c:2665). Returns the first not-in-use slot
/// of `refTagOwnerMap`, or NULL (with a non-final-build warning) if the pool is exhausted.
/// No oracle (returns a pointer into a mutable file-scope global).
///
/// # Safety
/// Returned pointer aliases the file-scope `refTagOwnerMap` global.
pub unsafe extern "C" fn FirstFreeTagOwner() -> *mut tagOwner_t {
    let mut i: usize = 0;

    while i < MAX_TAG_OWNERS {
        if (*addr_of!(refTagOwnerMap))[i].inuse == QFALSE {
            return addr_of_mut!((*addr_of_mut!(refTagOwnerMap))[i]);
        }
        i += 1;
    }

    // #ifndef FINAL_BUILD
    Com_Printf(&format!(
        "WARNING: MAX_TAG_OWNERS ({}) REF TAG LIMIT HIT\n",
        MAX_TAG_OWNERS
    ));
    // #endif
    core::ptr::null_mut()
}

/// `reference_tag_t *FirstFreeRefTag( tagOwner_t *tagOwner )` (g_misc.c:2684). Returns the
/// first not-in-use tag slot in `tagOwner`'s pool, or NULL (with a non-final-build warning)
/// when full. No oracle (pointer into a caller-supplied owner record).
///
/// # Safety
/// `tagOwner` must point to a valid `tagOwner_t`.
pub unsafe extern "C" fn FirstFreeRefTag(tagOwner: *mut tagOwner_t) -> *mut reference_tag_t {
    let mut i: usize = 0;

    debug_assert!(!tagOwner.is_null());

    while i < MAX_TAGS {
        if (*tagOwner).tags[i].inuse == QFALSE {
            return addr_of_mut!((*tagOwner).tags[i]);
        }
        i += 1;
    }

    // #ifndef FINAL_BUILD
    Com_Printf(&format!(
        "WARNING: MAX_TAGS ({}) REF TAG LIMIT HIT\n",
        MAX_TAGS
    ));
    // #endif
    core::ptr::null_mut()
}

/*
-------------------------
TAG_Init
-------------------------
*/

/// `void TAG_Init( void )` (g_misc.c:2711). Zeroes the file-scope `refTagOwnerMap` pool: each
/// owner's `tags[]` slots first, then the owner record itself (matching the C's two nested
/// `memset`s — the per-slot clears are redundant with the whole-record clear, but ported
/// faithfully). Oracle-tested over the file-static map (the C clears the same `refTagOwnerMap`
/// global; the test seeds it dirty and asserts a full zero afterward).
///
/// # Safety
/// Mutates the file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_Init() {
    let mut i: usize = 0;
    let mut x: usize = 0;

    while i < MAX_TAG_OWNERS {
        while x < MAX_TAGS {
            (*addr_of_mut!(refTagOwnerMap))[i].tags[x] = REFERENCE_TAG_ZERO;
            x += 1;
        }
        (*addr_of_mut!(refTagOwnerMap))[i].name = [0; MAX_REFNAME];
        (*addr_of_mut!(refTagOwnerMap))[i].tags = [REFERENCE_TAG_ZERO; MAX_TAGS];
        (*addr_of_mut!(refTagOwnerMap))[i].inuse = QFALSE;
        i += 1;
    }
}

/*
-------------------------
TAG_FindOwner
-------------------------
*/

/// `tagOwner_t *TAG_FindOwner( const char *owner )` (g_misc.c:2734). Linear-scans the
/// file-scope `refTagOwnerMap` for an in-use owner whose `name` case-insensitively matches
/// `owner`, returning a pointer to it or NULL. Oracle-tested over the file-static map (the C
/// scans the same `refTagOwnerMap` global; the test seeds named/in-use slots and asserts the
/// returned index matches).
///
/// # Safety
/// `owner` must be a valid NUL-terminated C string; the returned pointer aliases the
/// file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_FindOwner(owner: *const c_char) -> *mut tagOwner_t {
    let mut i: usize = 0;

    while i < MAX_TAG_OWNERS {
        if (*addr_of!(refTagOwnerMap))[i].inuse != QFALSE
            && Q_stricmp((*addr_of!(refTagOwnerMap))[i].name.as_ptr(), owner) == 0
        {
            return addr_of_mut!((*addr_of_mut!(refTagOwnerMap))[i]);
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/*
-------------------------
TAG_Find
-------------------------
*/

/// `reference_tag_t *TAG_Find( const char *owner, const char *name )` (g_misc.c:2756).
/// Resolves a named ref-tag for a given owner: looks up the owner (falling back to the
/// generic `TAG_GENERIC_NAME` owner when none is given or found), then linear-scans that
/// owner's in-use tags for one whose `name` case-insensitively matches `name`. If the
/// chosen owner has no such tag, it retries the scan against the generic owner. Returns a
/// pointer into the file-static `refTagOwnerMap` tag pool, or NULL.
///
/// Oracle-tested over the file-static map (the C scans the same `refTagOwnerMap` global;
/// the test seeds owner+tag slots and asserts the returned (owner,tag) index pair agrees).
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); the returned
/// pointer aliases the file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_Find(owner: *const c_char, name: *const c_char) -> *mut reference_tag_t {
    let mut tagOwner: *mut tagOwner_t = core::ptr::null_mut();
    let mut i: usize = 0;

    if !owner.is_null() && *owner != 0 {
        tagOwner = TAG_FindOwner(owner);
    }
    if tagOwner.is_null() {
        tagOwner = TAG_FindOwner(TAG_GENERIC_NAME.as_ptr());
    }

    //Not found...
    if tagOwner.is_null() {
        tagOwner = TAG_FindOwner(TAG_GENERIC_NAME.as_ptr());

        if tagOwner.is_null() {
            return core::ptr::null_mut();
        }
    }

    while i < MAX_TAGS {
        if (*tagOwner).tags[i].inuse != QFALSE
            && Q_stricmp((*tagOwner).tags[i].name.as_ptr(), name) == 0
        {
            return addr_of_mut!((*tagOwner).tags[i]);
        }
        i += 1;
    }

    //Try the generic owner instead
    tagOwner = TAG_FindOwner(TAG_GENERIC_NAME.as_ptr());

    if tagOwner.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while i < MAX_TAGS {
        if (*tagOwner).tags[i].inuse != QFALSE
            && Q_stricmp((*tagOwner).tags[i].name.as_ptr(), name) == 0
        {
            return addr_of_mut!((*tagOwner).tags[i]);
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/*
-------------------------
TAG_Add
-------------------------
*/

/// `reference_tag_t *TAG_Add( const char *name, const char *owner, vec3_t origin, vec3_t
/// angles, int radius, int flags )` (g_misc.c:2817). Inserts a named ref-tag under `owner`
/// (falling back to the generic `TAG_GENERIC_NAME` owner when none is given), allocating a
/// fresh owner slot via [`FirstFreeTagOwner`] if the owner doesn't yet exist and a fresh tag
/// slot via [`FirstFreeRefTag`]. Rejects a duplicate (owner,name) tag and a nameless tag.
/// Copies the origin/angles/radius/flags in, lower-cases both the owner and tag names (for
/// case-insensitive map searches), and marks both records in-use. Returns the new tag, or
/// NULL on duplicate/full/nameless. Oracle-tested over the file-static map (the C mutates the
/// same `refTagOwnerMap` global; the test seeds both maps identically, adds, and asserts the
/// returned tag's fields agree bit-for-bit).
///
/// # Safety
/// `name`/`owner` must be valid NUL-terminated C strings (or NULL); mutates and returns a
/// pointer into the file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_Add(
    name: *const c_char,
    owner: *const c_char,
    origin: &vec3_t,
    angles: &vec3_t,
    radius: c_int,
    flags: c_int,
) -> *mut reference_tag_t {
    let tag: *mut reference_tag_t;
    let mut tagOwner: *mut tagOwner_t;
    let mut owner = owner;

    //Make sure this tag's name isn't alread in use
    if !TAG_Find(owner, name).is_null() {
        Com_Printf(&format!(
            // S_COLOR_RED
            "^1Duplicate tag name \"{}\"\n",
            CStr::from_ptr(name).to_string_lossy()
        ));
        return core::ptr::null_mut();
    }

    //Attempt to add this to the owner's list
    if owner.is_null() || *owner == 0 {
        //If the owner isn't found, use the generic world name
        owner = TAG_GENERIC_NAME.as_ptr();
    }

    tagOwner = TAG_FindOwner(owner);

    if tagOwner.is_null() {
        //Create a new owner list
        tagOwner = FirstFreeTagOwner(); //new	tagOwner_t;

        if tagOwner.is_null() {
            debug_assert!(false);
            return core::ptr::null_mut();
        }
    }

    //This is actually reverse order of how SP does it because of the way we're storing/allocating.
    //Now that we have the owner, we want to get the first free reftag on the owner itself.
    tag = FirstFreeRefTag(tagOwner);

    if tag.is_null() {
        debug_assert!(false);
        return core::ptr::null_mut();
    }

    //Copy the information
    VectorCopy(origin, &mut (*tag).origin);
    VectorCopy(angles, &mut (*tag).angles);
    (*tag).radius = radius;
    (*tag).flags = flags;

    if name.is_null() || *name == 0 {
        Com_Printf(&format!(
            // S_COLOR_RED
            "^1ERROR: Nameless ref_tag found at ({} {} {})\n",
            origin[0] as c_int, origin[1] as c_int, origin[2] as c_int
        ));
        return core::ptr::null_mut();
    }

    //Copy the name
    Q_strncpyz((*tagOwner).name.as_mut_ptr(), owner, MAX_REFNAME as c_int);
    Q_strlwr((*tagOwner).name.as_mut_ptr()); //NOTENOTE: For case insensitive searches on a map

    //Copy the name
    Q_strncpyz((*tag).name.as_mut_ptr(), name, MAX_REFNAME as c_int);
    Q_strlwr((*tag).name.as_mut_ptr()); //NOTENOTE: For case insensitive searches on a map

    (*tagOwner).inuse = QTRUE;
    (*tag).inuse = QTRUE;

    tag
}

/*
-------------------------
TAG_GetOrigin
-------------------------
*/

/// `int TAG_GetOrigin( const char *owner, const char *name, vec3_t origin )` (g_misc.c:2893).
/// Looks up `(owner,name)` via [`TAG_Find`]; on miss, clears `origin` and returns 0; on hit,
/// copies the tag's origin out and returns 1. Oracle-tested over the file-static map (the C
/// reads the same `refTagOwnerMap` global; the test seeds both maps, queries, and asserts the
/// returned flag + origin agree bit-for-bit).
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); reads the
/// file-scope `refTagOwnerMap` global and writes `origin`.
pub unsafe fn TAG_GetOrigin(
    owner: *const c_char,
    name: *const c_char,
    origin: &mut vec3_t,
) -> c_int {
    let tag: *mut reference_tag_t = TAG_Find(owner, name);

    if tag.is_null() {
        VectorClear(origin);
        return 0;
    }

    VectorCopy(&(*tag).origin, origin);

    1
}

/*
-------------------------
TAG_GetOrigin2
Had to get rid of that damn assert for dev
-------------------------
*/

/// `int TAG_GetOrigin2( const char *owner, const char *name, vec3_t origin )` (g_misc.c:2915).
/// Looks up `(owner,name)` via [`TAG_Find`]; on miss, returns 0 (unlike [`TAG_GetOrigin`], the
/// C "Had to get rid of that damn assert for dev" comment notes the assert was removed, so the
/// NULL branch here neither asserts nor clears `origin`); on hit, copies the tag's origin out
/// and returns 1. No-oracle: reaches the opaque `reference_tag_t` pointer + level statics, same
/// as the [`TAG_GetAngles`]/[`TAG_GetRadius`] siblings.
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); reads the
/// file-scope `refTagOwnerMap` global and writes `origin`.
pub unsafe fn TAG_GetOrigin2(
    owner: *const c_char,
    name: *const c_char,
    origin: &mut vec3_t,
) -> c_int {
    let tag: *mut reference_tag_t = TAG_Find(owner, name);

    if tag.is_null() {
        return 0;
    }

    VectorCopy(&(*tag).origin, origin);

    1
}

/*
-------------------------
TAG_GetAngles
-------------------------
*/

/// `int TAG_GetAngles( const char *owner, const char *name, vec3_t angles )` (g_misc.c:2934).
/// Looks up `(owner,name)` via [`TAG_Find`]; on miss, asserts and returns 0; on hit, copies the
/// tag's angles out and returns 1. Oracle-tested over the file-static map (the C reads the same
/// `refTagOwnerMap` global; the test seeds both maps, queries, and asserts the returned flag +
/// angles agree bit-for-bit). (The C `assert(0)` is debug-only; the test queries only hits.)
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); reads the
/// file-scope `refTagOwnerMap` global and writes `angles`.
pub unsafe fn TAG_GetAngles(
    owner: *const c_char,
    name: *const c_char,
    angles: &mut vec3_t,
) -> c_int {
    let tag: *mut reference_tag_t = TAG_Find(owner, name);

    if tag.is_null() {
        debug_assert!(false);
        return 0;
    }

    VectorCopy(&(*tag).angles, angles);

    1
}

/*
-------------------------
TAG_GetRadius
-------------------------
*/

/// `int TAG_GetRadius( const char *owner, const char *name )` (g_misc.c:2955). Looks up
/// `(owner,name)` via [`TAG_Find`]; on miss, asserts and returns 0; on hit, returns the tag's
/// radius. Oracle-tested over the file-static map (the C reads the same `refTagOwnerMap`
/// global; the test seeds both maps, queries, and asserts the returned radius agrees
/// bit-for-bit). (The C `assert(0)` is debug-only; the test queries only hits.)
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); reads the
/// file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_GetRadius(owner: *const c_char, name: *const c_char) -> c_int {
    let tag: *mut reference_tag_t = TAG_Find(owner, name);

    if tag.is_null() {
        debug_assert!(false);
        return 0;
    }

    (*tag).radius
}

/*
-------------------------
TAG_GetFlags
-------------------------
*/

/// `int TAG_GetFlags( const char *owner, const char *name )` (g_misc.c:2974). Looks up
/// `(owner,name)` via [`TAG_Find`]; on miss, asserts and returns 0; on hit, returns the tag's
/// flags. No-oracle: reaches the opaque `reference_tag_t` pointer + level statics, same as the
/// [`TAG_GetRadius`]/[`TAG_GetAngles`] siblings. (The C `assert(0)` is debug-only.)
///
/// # Safety
/// `owner`/`name` must be valid NUL-terminated C strings (or `owner` NULL); reads the
/// file-scope `refTagOwnerMap` global.
pub unsafe fn TAG_GetFlags(owner: *const c_char, name: *const c_char) -> c_int {
    let tag: *mut reference_tag_t = TAG_Find(owner, name);

    if tag.is_null() {
        debug_assert!(false);
        return 0;
    }

    (*tag).flags
}

//----------------------------------------------------------
// ATST damage-box cluster (g_misc.c:1804-2015). The whole block is bracketed by
// `#if 0 //damage box stuff ... #endif` in C, so it never actually compiled in the MP
// module — yet the source is present and faithful, so the self-contained leaves are
// ported here. The two ATST-driver leaves (`ATST_ManageDamageBoxes`,
// `G_PlayerBecomeATST`) are NOT ported: they reference the `ATST_MINS0..ATST_HEADSIZE`
// macros and the `playerState_t.usingATST` field, which exist only in the single-player
// `code/game/` tree and are undefined in the MP `codemp/game/` headers (they would not
// compile even with `#if 0` removed). See the worker BLOCKED note.

/// `void DmgBoxHit( gentity_t *self, gentity_t *other, trace_t *trace )` (g_misc.c:1805).
/// `touch` callback for an ATST damage-redirect box: a no-op (`return;`). No oracle (empty
/// touch stub).
///
/// # Safety
/// Standard `touch` ABI; all parameters are unused.
pub unsafe extern "C" fn DmgBoxHit(
    _self_: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    // return;
}

/// `void DmgBoxUpdateSelf( gentity_t *self )` (g_misc.c:1810). Per-frame `think` for an ATST
/// damage box: frees itself if its owner is gone/non-client/dead or if the owner's handle for
/// this body part no longer points at it; otherwise re-links and re-arms for the next frame.
/// No oracle (`g_entities`/`level` globals + `trap_LinkEntity`/`G_FreeEntity`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn DmgBoxUpdateSelf(self_: *mut gentity_t) {
    let owner: *mut gentity_t = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize);

    'killMe: {
        if owner.is_null() || (*owner).client.is_null() || (*owner).inuse == QFALSE {
            break 'killMe;
        }

        if (*self_).damageRedirect == DAMAGEREDIRECT_HEAD
            && (*(*owner).client).damageBoxHandle_Head != (*self_).s.number
        {
            break 'killMe;
        }

        if (*self_).damageRedirect == DAMAGEREDIRECT_RLEG
            && (*(*owner).client).damageBoxHandle_RLeg != (*self_).s.number
        {
            break 'killMe;
        }

        if (*self_).damageRedirect == DAMAGEREDIRECT_LLEG
            && (*(*owner).client).damageBoxHandle_LLeg != (*self_).s.number
        {
            break 'killMe;
        }

        if (*owner).health < 1 {
            break 'killMe;
        }

        //G_TestLine(self->r.currentOrigin, owner->client->ps.origin, 0x0000ff, 100);

        trap::LinkEntity(self_);

        (*self_).nextthink = (*addr_of!(level)).time;
        return;
    }

    // killMe:
    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time;
}

/// `void DmgBoxAbsorb_Die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int
/// damage, int mod )` (g_misc.c:1854). `die` callback for an ATST damage box: keeps itself
/// alive by resetting `health` to 1 (damage is redirected to the ATST pilot elsewhere). No
/// oracle (single field write).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other parameters are unused.
pub unsafe extern "C" fn DmgBoxAbsorb_Die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    (*self_).health = 1;
}

/// `void DmgBoxAbsorb_Pain( gentity_t *self, gentity_t *attacker, int damage )`
/// (g_misc.c:1859). `pain` callback for an ATST damage box: keeps itself alive by resetting
/// `health` to 1. No oracle (single field write).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other parameters are unused.
pub unsafe extern "C" fn DmgBoxAbsorb_Pain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
) {
    (*self_).health = 1;
}

/// `gentity_t *CreateNewDamageBox( gentity_t *ent )` (g_misc.c:1864). Spawns a server-only,
/// no-draw, player-solid damage-redirect box owned by `ent`, installs the
/// `DmgBoxHit`/`DmgBoxAbsorb_Pain`/`DmgBoxAbsorb_Die`/`DmgBoxUpdateSelf` callbacks, and arms
/// its first think. No oracle (`G_Spawn` + entity-state plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn CreateNewDamageBox(ent: *mut gentity_t) -> *mut gentity_t {
    let dmgBox: *mut gentity_t;

    //We do not want the client to have any real knowledge of the entity whatsoever. It will only
    //ever be used on the server.
    dmgBox = G_Spawn();
    (*dmgBox).classname = c"dmg_box".as_ptr() as *mut c_char;

    (*dmgBox).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    (*dmgBox).r.ownerNum = (*ent).s.number;

    (*dmgBox).clipmask = 0;
    (*dmgBox).r.contents = MASK_PLAYERSOLID;

    (*dmgBox).mass = 5000.0;

    (*dmgBox).s.eFlags |= EF_NODRAW;
    (*dmgBox).r.svFlags |= SVF_NOCLIENT;

    (*dmgBox).touch = Some(DmgBoxHit);

    (*dmgBox).takedamage = QTRUE;

    (*dmgBox).health = 1;

    (*dmgBox).pain = Some(DmgBoxAbsorb_Pain);
    (*dmgBox).die = Some(DmgBoxAbsorb_Die);

    (*dmgBox).think = Some(DmgBoxUpdateSelf);
    (*dmgBox).nextthink = (*addr_of!(level)).time + 50;

    dmgBox
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;

    /// `TAG_Init` over the file-static `refTagOwnerMap`. Seeds every slot dirty (in-use,
    /// non-zero name + every tag in-use), runs the port, and asserts the whole map is a
    /// flat-zero block — matching the real C, whose oracle wrapper does the same dirty/run
    /// and reports a `1` for a fully-zeroed pool.
    #[test]
    fn tag_init_matches_oracle() {
        unsafe {
            // dirty the Rust map the same way the oracle wrapper does
            for i in 0..MAX_TAG_OWNERS {
                (*addr_of_mut!(refTagOwnerMap))[i].inuse = QTRUE;
                (*addr_of_mut!(refTagOwnerMap))[i].name[0] = b'X' as c_char;
                for x in 0..MAX_TAGS {
                    (*addr_of_mut!(refTagOwnerMap))[i].tags[x].inuse = QTRUE;
                }
            }

            TAG_Init();

            // assert the entire map is a flat-zero block (faithful to C's memset behavior)
            let bytes = core::slice::from_raw_parts(
                addr_of!(refTagOwnerMap) as *const u8,
                core::mem::size_of::<[tagOwner_t; MAX_TAG_OWNERS]>(),
            );
            assert!(
                bytes.iter().all(|&b| b == 0),
                "refTagOwnerMap not fully zeroed"
            );

            // and the real C agrees that the same dirty/run leaves a fully-zeroed pool
            assert_eq!(
                oracle::jka_TAG_Init_zeroes(),
                1,
                "C TAG_Init did not zero the pool"
            );
        }
    }

    /// `TAG_FindOwner` over the file-static `refTagOwnerMap`. Seeds an identical set of
    /// owner slots (varying name + in-use flag) into both the Rust map and the C map, then
    /// queries the same owner names (exact, case-folded, absent, and the empty string),
    /// asserting the matched slot index agrees bit-for-bit with the real C.
    #[test]
    fn tag_findowner_matches_oracle() {
        unsafe {
            // identical seeding of both maps
            let seed: &[(usize, &core::ffi::CStr, qboolean)] = &[
                (0, c"alpha", QTRUE),
                (1, c"Bravo", QTRUE),
                (2, c"charlie", QFALSE), // present-by-name but not in use -> never matched
                (3, c"__WORLD__", QTRUE),
            ];

            // reset both pools to empty
            TAG_Init();
            oracle::jka_TAG_clear_map();

            for &(idx, name, inuse) in seed {
                let nbytes = name.to_bytes();
                let slot = &mut (*addr_of_mut!(refTagOwnerMap))[idx];
                for (k, &b) in nbytes.iter().enumerate() {
                    slot.name[k] = b as c_char;
                }
                slot.name[nbytes.len()] = 0;
                slot.inuse = inuse;

                oracle::jka_TAG_seed_owner(idx as c_int, name.as_ptr(), inuse as c_int);
            }

            let queries: &[&core::ffi::CStr] = &[
                c"alpha",     // exact
                c"ALPHA",     // case-folded match
                c"bravo",     // case-folded match (seeded "Bravo")
                c"charlie",   // seeded but not in use -> no match
                c"__WORLD__", // exact
                c"missing",   // absent
                c"",          // empty string
            ];

            for q in queries {
                let r = TAG_FindOwner(q.as_ptr());
                let r_idx = if r.is_null() {
                    -1
                } else {
                    ((r as usize - addr_of!(refTagOwnerMap) as *const _ as usize)
                        / core::mem::size_of::<tagOwner_t>()) as c_int
                };
                let c_idx = oracle::jka_TAG_FindOwner_index(q.as_ptr());
                assert_eq!(r_idx, c_idx, "TAG_FindOwner index mismatch for {:?}", q);
            }
        }
    }

    /// `TAG_Find` over the file-static `refTagOwnerMap`. Seeds an identical owner map plus
    /// per-owner tags into both the Rust map and the C map, then queries (owner, name)
    /// pairs that exercise every branch — owner+tag hit, owner-found-but-tag-only-in-generic
    /// fallback, NULL/empty owner falling through to the generic owner, and an absent tag —
    /// asserting the matched (owner,tag) index agrees bit-for-bit with the real C.
    #[test]
    fn tag_find_matches_oracle() {
        use core::ffi::CStr;

        unsafe {
            // reset both pools to empty
            TAG_Init();
            oracle::jka_TAG_clear_map();

            // owners: 0=alpha, 1=generic "__WORLD__"
            let owners: &[(usize, &CStr, qboolean)] =
                &[(0, c"alpha", QTRUE), (1, TAG_GENERIC_NAME, QTRUE)];
            for &(idx, name, inuse) in owners {
                let nbytes = name.to_bytes();
                let slot = &mut (*addr_of_mut!(refTagOwnerMap))[idx];
                for (k, &b) in nbytes.iter().enumerate() {
                    slot.name[k] = b as c_char;
                }
                slot.name[nbytes.len()] = 0;
                slot.inuse = inuse;
                oracle::jka_TAG_seed_owner(idx as c_int, name.as_ptr(), inuse as c_int);
            }

            // tags: (owner_idx, tag_idx, name, inuse)
            let tags: &[(usize, usize, &CStr, qboolean)] = &[
                (0, 0, c"hand", QTRUE),       // alpha/hand
                (0, 1, c"Foot", QFALSE),      // present-by-name but not in use
                (1, 0, c"world_spot", QTRUE), // generic/world_spot
                (1, 5, c"hand", QTRUE),       // generic also has a "hand"
            ];
            for &(oi, ti, name, inuse) in tags {
                let nbytes = name.to_bytes();
                let slot = &mut (*addr_of_mut!(refTagOwnerMap))[oi].tags[ti];
                for (k, &b) in nbytes.iter().enumerate() {
                    slot.name[k] = b as c_char;
                }
                slot.name[nbytes.len()] = 0;
                slot.inuse = inuse;
                oracle::jka_TAG_seed_tag(oi as c_int, ti as c_int, name.as_ptr(), inuse as c_int);
            }

            // (owner, name) queries exercising each branch
            let queries: &[(*const c_char, &CStr)] = &[
                (c"alpha".as_ptr(), c"hand"),       // owner+tag hit -> owner 0 tag 0
                (c"alpha".as_ptr(), c"HAND"),       // case-folded hit
                (c"alpha".as_ptr(), c"foot"), // tag present but not in use -> generic fallback (none) -> NULL
                (c"alpha".as_ptr(), c"world_spot"), // not on alpha -> generic owner has it
                (c"missing".as_ptr(), c"hand"), // unknown owner -> generic owner's hand
                (core::ptr::null(), c"world_spot"), // NULL owner -> generic
                (c"".as_ptr(), c"world_spot"), // empty owner -> generic
                (c"alpha".as_ptr(), c"nope"), // absent everywhere -> NULL
            ];

            for &(owner, name) in queries {
                let r = TAG_Find(owner, name.as_ptr());
                let r_idx = if r.is_null() {
                    -1
                } else {
                    // locate which owner/tag this pointer is
                    let mut found: c_int = -2;
                    for oi in 0..MAX_TAG_OWNERS {
                        let base = addr_of!((*addr_of!(refTagOwnerMap))[oi].tags[0]) as usize;
                        let end = base + MAX_TAGS * core::mem::size_of::<reference_tag_t>();
                        let rp = r as usize;
                        if rp >= base && rp < end {
                            let ti = (rp - base) / core::mem::size_of::<reference_tag_t>();
                            found = (oi * MAX_TAGS + ti) as c_int;
                            break;
                        }
                    }
                    found
                };
                let c_idx = oracle::jka_TAG_Find_index(owner, name.as_ptr());
                assert_eq!(r_idx, c_idx, "TAG_Find index mismatch for name {:?}", name);
            }
        }
    }

    /// `TAG_Add` over the file-static `refTagOwnerMap`. Adds an identical sequence of tags
    /// into both the Rust map and the C map — a fresh owner (allocating a new owner slot), a
    /// second tag under that same owner, a tag with a NULL owner (folding to the generic
    /// `__WORLD__` owner), and a duplicate (rejected) — and after each add asserts the packed
    /// returned (owner,tag) index plus the resulting tag's stored fields (origin, angles,
    /// radius, flags, inuse, lower-cased name) and owner record (inuse, lower-cased name) agree
    /// bit-for-bit with the real C.
    #[test]
    fn tag_add_matches_oracle() {
        use core::ffi::{c_char, c_int, CStr};

        unsafe {
            // reset both pools to empty
            TAG_Init();
            oracle::jka_TAG_clear_map();

            // (name, owner, origin, angles, radius, flags) sequence
            let adds: &[(&CStr, *const c_char, [f32; 3], [f32; 3], c_int, c_int)] = &[
                (
                    c"Hand",
                    c"Alpha".as_ptr(),
                    [1.0, 2.0, 3.0],
                    [10.0, 20.0, 30.0],
                    7,
                    0x1,
                ),
                (
                    c"Foot",
                    c"alpha".as_ptr(),
                    [4.0, 5.0, 6.0],
                    [40.0, 50.0, 60.0],
                    9,
                    0x2,
                ),
                (
                    c"WorldSpot",
                    core::ptr::null(),
                    [7.0, 8.0, 9.0],
                    [70.0, 80.0, 90.0],
                    11,
                    0x4,
                ),
                (
                    c"Hand",
                    c"Alpha".as_ptr(),
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    0,
                    0,
                ), // duplicate -> NULL
            ];

            for &(name, owner, origin, angles, radius, flags) in adds {
                let o: vec3_t = origin;
                let a: vec3_t = angles;
                let r = TAG_Add(name.as_ptr(), owner, &o, &a, radius, flags);
                let r_idx = if r.is_null() {
                    -1
                } else {
                    let mut found: c_int = -2;
                    for oi in 0..MAX_TAG_OWNERS {
                        let base = addr_of!((*addr_of!(refTagOwnerMap))[oi].tags[0]) as usize;
                        let end = base + MAX_TAGS * core::mem::size_of::<reference_tag_t>();
                        let rp = r as usize;
                        if rp >= base && rp < end {
                            let ti = (rp - base) / core::mem::size_of::<reference_tag_t>();
                            found = (oi * MAX_TAGS + ti) as c_int;
                            break;
                        }
                    }
                    found
                };

                let c_idx = oracle::jka_TAG_Add(
                    name.as_ptr(),
                    owner,
                    origin[0],
                    origin[1],
                    origin[2],
                    angles[0],
                    angles[1],
                    angles[2],
                    radius,
                    flags,
                );
                assert_eq!(r_idx, c_idx, "TAG_Add index mismatch for {:?}", name);

                // on a successful add, cross-check the stored fields slot-for-slot
                if c_idx >= 0 {
                    let oi = (c_idx as usize) / MAX_TAGS;
                    let ti = (c_idx as usize) % MAX_TAGS;
                    let tag = &(*addr_of!(refTagOwnerMap))[oi].tags[ti];

                    assert_eq!(
                        tag.inuse as c_int,
                        oracle::jka_TAG_get_inuse(oi as c_int, ti as c_int)
                    );
                    assert_eq!(
                        tag.radius,
                        oracle::jka_TAG_get_radius(oi as c_int, ti as c_int)
                    );
                    assert_eq!(
                        tag.flags,
                        oracle::jka_TAG_get_flags(oi as c_int, ti as c_int)
                    );
                    for c in 0..3 {
                        assert_eq!(
                            tag.origin[c],
                            oracle::jka_TAG_get_origin(oi as c_int, ti as c_int, c as c_int),
                            "origin[{}] mismatch",
                            c
                        );
                        assert_eq!(
                            tag.angles[c],
                            oracle::jka_TAG_get_angles(oi as c_int, ti as c_int, c as c_int),
                            "angles[{}] mismatch",
                            c
                        );
                    }

                    // lower-cased tag name agrees
                    let mut cname = [0u8; MAX_REFNAME];
                    oracle::jka_TAG_get_name(
                        oi as c_int,
                        ti as c_int,
                        cname.as_mut_ptr() as *mut c_char,
                    );
                    let c_tagname = CStr::from_ptr(cname.as_ptr() as *const c_char);
                    let r_tagname = CStr::from_ptr(tag.name.as_ptr());
                    assert_eq!(r_tagname, c_tagname, "tag name mismatch");

                    // owner record (inuse + lower-cased name) agrees
                    assert_eq!(
                        (*addr_of!(refTagOwnerMap))[oi].inuse as c_int,
                        oracle::jka_TAG_owner_inuse(oi as c_int),
                        "owner inuse mismatch"
                    );
                    let mut coname = [0u8; MAX_REFNAME];
                    oracle::jka_TAG_owner_name(oi as c_int, coname.as_mut_ptr() as *mut c_char);
                    let c_oname = CStr::from_ptr(coname.as_ptr() as *const c_char);
                    let r_oname = CStr::from_ptr((*addr_of!(refTagOwnerMap))[oi].name.as_ptr());
                    assert_eq!(r_oname, c_oname, "owner name mismatch");
                }
            }
        }
    }

    /// `TAG_GetOrigin` over the file-static `refTagOwnerMap`. Plants an identical tag (with a
    /// known origin) into both the Rust map and the C map via `TAG_Add`, then for each query (a
    /// hit and a miss) asserts the returned flag and the written origin agree bit-for-bit with
    /// the real C (the miss path `VectorClear`s the out vector, which is checked too).
    #[test]
    fn tag_getorigin_matches_oracle() {
        use core::ffi::{c_char, CStr};

        unsafe {
            // The seed wrappers don't set origin, so drive both maps through TAG_Add (already
            // oracle-verified) to plant a tag with a known origin, then query it.
            TAG_Init();
            oracle::jka_TAG_clear_map();
            let o: vec3_t = [1.5, -2.25, 3.0];
            let ang: vec3_t = [0.0, 0.0, 0.0];
            TAG_Add(c"hand".as_ptr(), c"alpha".as_ptr(), &o, &ang, 0, 0);
            oracle::jka_TAG_Add(
                c"hand".as_ptr(),
                c"alpha".as_ptr(),
                1.5,
                -2.25,
                3.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
            );

            let queries: &[(*const c_char, &CStr)] = &[
                (c"alpha".as_ptr(), c"hand"),    // hit
                (c"alpha".as_ptr(), c"missing"), // miss -> VectorClear, flag 0
            ];

            for &(owner, name) in queries {
                let mut r_out: vec3_t = [9.0, 9.0, 9.0];
                let r_flag = TAG_GetOrigin(owner, name.as_ptr(), &mut r_out);

                let mut c_out: [f32; 3] = [9.0, 9.0, 9.0];
                let c_flag = oracle::jka_TAG_GetOrigin(owner, name.as_ptr(), c_out.as_mut_ptr());

                assert_eq!(r_flag, c_flag, "TAG_GetOrigin flag mismatch for {:?}", name);
                assert_eq!(r_out, c_out, "TAG_GetOrigin out mismatch for {:?}", name);
            }
        }
    }

    /// `TAG_GetAngles` over the file-static `refTagOwnerMap`. Adds an identical tag (with known
    /// angles) into both maps via `TAG_Add`, then for the hit query asserts the returned flag
    /// and the written angles agree bit-for-bit with the real C. (Only the hit path is tested:
    /// a miss trips the C `assert(0)`.)
    #[test]
    fn tag_getangles_matches_oracle() {
        use core::ffi::{c_char, CStr};

        unsafe {
            TAG_Init();
            oracle::jka_TAG_clear_map();

            let o: vec3_t = [0.0, 0.0, 0.0];
            let ang: vec3_t = [11.5, -22.75, 33.0];
            TAG_Add(c"hand".as_ptr(), c"alpha".as_ptr(), &o, &ang, 0, 0);
            oracle::jka_TAG_Add(
                c"hand".as_ptr(),
                c"alpha".as_ptr(),
                0.0,
                0.0,
                0.0,
                11.5,
                -22.75,
                33.0,
                0,
                0,
            );

            let queries: &[(*const c_char, &CStr)] = &[(c"alpha".as_ptr(), c"hand")];

            for &(owner, name) in queries {
                let mut r_out: vec3_t = [9.0, 9.0, 9.0];
                let r_flag = TAG_GetAngles(owner, name.as_ptr(), &mut r_out);

                let mut c_out: [f32; 3] = [9.0, 9.0, 9.0];
                let c_flag = oracle::jka_TAG_GetAngles(owner, name.as_ptr(), c_out.as_mut_ptr());

                assert_eq!(r_flag, c_flag, "TAG_GetAngles flag mismatch for {:?}", name);
                assert_eq!(r_out, c_out, "TAG_GetAngles out mismatch for {:?}", name);
            }
        }
    }

    /// `TAG_GetRadius` over the file-static `refTagOwnerMap`. Adds an identical tag (with a
    /// known radius) into both maps via `TAG_Add`, then for the hit query asserts the returned
    /// radius agrees bit-for-bit with the real C. (Only the hit path is tested: a miss trips
    /// the C `assert(0)`.)
    #[test]
    fn tag_getradius_matches_oracle() {
        use core::ffi::{c_char, CStr};

        unsafe {
            TAG_Init();
            oracle::jka_TAG_clear_map();

            let o: vec3_t = [0.0, 0.0, 0.0];
            let ang: vec3_t = [0.0, 0.0, 0.0];
            TAG_Add(c"hand".as_ptr(), c"alpha".as_ptr(), &o, &ang, 42, 0);
            oracle::jka_TAG_Add(
                c"hand".as_ptr(),
                c"alpha".as_ptr(),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                42,
                0,
            );

            let queries: &[(*const c_char, &CStr)] = &[(c"alpha".as_ptr(), c"hand")];

            for &(owner, name) in queries {
                let r = TAG_GetRadius(owner, name.as_ptr());
                let c = oracle::jka_TAG_GetRadius(owner, name.as_ptr());
                assert_eq!(r, c, "TAG_GetRadius mismatch for {:?}", name);
            }
        }
    }
}
