//! Port of `g_items.c` — item pickup, respawn, and drop/launch logic.
//! Landed incrementally: only the helpers whose already-ported callers (or the
//! frontier enablers) reach. The pure respawn-time scaler `adjustRespawnTime`
//! (`g_items.c:47`) lands first — it is a self-contained leaf (plain arithmetic over
//! the item type/tag and the two read globals `g_adaptRespawn`/`level.numPlayingClients`)
//! and is oracle-checked bit-exact. `G_ItemDisabled` (`g_items.c:3074`) follows — a
//! self-contained `disable_<classname>` cvar probe (`Com_sprintf` + a `trap_Cvar_*`
//! read, so no oracle), the first leaf on the `G_SpawnItem` path. The `RegisterItem`
//! family lands alongside: the `itemRegistered[MAX_ITEMS]` global (`g_items.c:2979`),
//! `RegisterItem` (`g_items.c:3033` — marks an item registered by its `bg_itemlist`
//! offset; NULL→`G_Error`), and `SaveRegisteredItems` (`g_items.c:3049` — serializes the
//! flags to the `CS_ITEMS` config string). Both are no-oracle (G_Error/trap-bound).
//! `ClearRegisteredItems` still waits on `G_PrecacheDispensers`. `G_BounceItem`
//! (`g_items.c:3143`) lands next — the trajectory-reflect math feeding `G_RunItem`'s
//! physics step (twin of the already-ported `G_BounceObject`/`G_BounceMissile`); no
//! oracle (mutates a `gentity_t` and fires the `ent->touch` fn-pointer). `Add_Ammo`
//! (`g_items.c:2131`) follows — a `Pickup_Ammo`/`MedPackGive` leaf that clamps a
//! client's `ps.ammo[]` slot to its [`ammoData`] max; no oracle (mutates the
//! `gentity_t`→`client`→`ps` graph). `Pickup_Ammo` (`g_items.c:2143`) lands on top —
//! awards ammo (the `ammo_all` giTag==-1 branch tops up the base types, with siege
//! bonuses gated on the `STAT_WEAPONS` ownership bits) via [`Add_Ammo`], returning the
//! [`adjustRespawnTime`] delay; no oracle (same entity/cvar graph). `Pickup_Weapon`
//! (`g_items.c:2193`) grants the weapon bit + tops up its ammo slot with the
//! "respawning rules" taper for non-dropped non-team-DM items, reading `g_gametype` /
//! `g_weaponRespawn`; no oracle (same entity/cvar graph). `Pickup_Holdable`
//! (`g_items.c:2117`) sets the `STAT_HOLDABLE_ITEM` index (the `ent->item - bg_itemlist`
//! offset) + the `STAT_HOLDABLE_ITEMS` ownership bit; no oracle (same graph). The drop/launch
//! enablers `LaunchItem`/`Drop_Item`
//! (the cross-cutting unblockers for g_combat's `TossClientItems`/`TossClientWeapon`
//! and g_target's `target_give`) now have their g_team flag callbacks
//! (`Team_DroppedFlagThink`/`Team_CheckDroppedItem`, `Team_FreeEntity`) as real Rust
//! symbols; `LaunchItem` remains blocked only on `Touch_Item` (whose `gentity_t::touch`
//! slot it assigns), which in turn waits on `CheckItemCanBePickedUpByNPC` + `Pickup_Team`.
//! `G_RunItem` (`g_items.c:3196`) lands independently of that chain — the per-frame item
//! physics step (gravity-snap + trace + bounce/free), all of whose callees
//! (`G_RunThink`/`BG_EvaluateTrajectory`/`G_BounceItem`/`Team_FreeEntity`/`G_FreeEntity`)
//! and the core trace/link/contents traps were already ported; no oracle.
//!
//! The E-Web (emplaced-gun) Ghoul2 cluster lands next: `EWeb_SetBoneAngles`
//! (`g_items.c:1508` — server/client bone-angle override, walking the entity-state's two
//! bone slots and applying `BONE_ANGLES_POSTMULT` via `trap_G2API_SetBoneAngles`),
//! `EWeb_SetBoneAnim` (`g_items.c:1614` — `model_root` g2 anim via `trap_G2API_SetBoneAnim`),
//! `EWebFire` (`g_items.c:1637` — reads the muzzle bolt, launches a `WP_TURRET` projectile via
//! `CreateMissile`, plays the muzzle flash) and `EWebPositionUser` (`g_items.c:1680` — locks the
//! owner to the cannon stand point via a trace + strafe `G_SetAnim`). All four are no-oracle
//! (G2 traps + entity/client mutation). `EWeb_Create` (`g_items.c:1872`) stays blocked: its
//! `think` slot assigns the unported `EWebThink` (which in turn waits on `EWebUpdateBoneAngles`).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_misc::{
    bg_itemlist, bg_numItems, BG_AddPredictableEventToPlayerstate, BG_CanItemBeGrabbed,
    BG_CycleInven, BG_EmplacedView, BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta, BG_FindItem,
    BG_FindItemForHoldable, BG_FindItemForWeapon, BG_GiveMeVectorFromMatrix,
};
use crate::codemp::game::bg_vehicles_h::VH_WALKER;
use crate::codemp::game::bg_public::{
    gitem_t, itemType_t, CS_ITEMS, EFFECT_EXPLOSION_DETPACK, EFFECT_EXPLOSION_PAS, EF_CLIENTSMOOTH,
    EF_DEAD, EF_DROPPEDWEAPON, EF_G2ANIMATING, EF_ITEMPLACEHOLDER, EF_NODRAW,
    MOD_TURBLAST, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_LEGS,
    EF_SEEKERDRONE, ET_GENERAL, ET_HOLOCRON, ET_ITEM, ET_NPC, ET_SPECIAL,
    EV_GENERAL_SOUND, EV_GLOBAL_ITEM_PICKUP, EV_GLOBAL_SOUND, EV_ITEM_PICKUP, EV_ITEM_RESPAWN,
    EV_LOCALTIMER, GT_CTF, GT_CTY, GT_DUEL, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_TEAM, HANDEXTEND_NONE, HI_EWEB,
    HI_HEALTHDISP, HI_MEDPAC, HI_MEDPAC_BIG, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD, IT_AMMO, IT_ARMOR, IT_HEALTH,
    IT_HOLDABLE, IT_POWERUP, IT_TEAM, ITEM_RADIUS,
    IT_WEAPON, MASK_PLAYERSOLID, MASK_SHOT, MASK_SOLID, MAX_ITEMS, MOD_SENTRY, MOD_SUICIDE, MOD_UNKNOWN, PERS_PLAYEREVENTS, PLAYEREVENT_DENIEDREWARD,
    PM_DEAD,
    PW_BLUEFLAG, PW_CLOAKED, PW_FORCE_BOON, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT,
    PW_NEUTRALFLAG, PW_REDFLAG, PW_YSALAMIRI, STAT_ARMOR,
    STAT_HEALTH, STAT_HOLDABLE_ITEM, STAT_HOLDABLE_ITEMS, STAT_MAX_HEALTH, STAT_WEAPONS,
    TOSS_DEBOUNCE_TIME, WEAPON_READY,
};
use crate::codemp::game::bg_weapons::{ammoData, weaponData};
use crate::codemp::game::bg_weapons_h::{
    weapon_t, AMMO_BLASTER, AMMO_DETPACK, AMMO_METAL_BOLTS, AMMO_POWERCELL, AMMO_ROCKETS,
    AMMO_THERMAL, AMMO_TRIPMINE, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DET_PACK, WP_EMPLACED_GUN,
    WP_MELEE, WP_NONE,
    WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::ai_h::SQUAD_STAND_AND_SHOOT;
use crate::codemp::game::b_public_h::SCF_FORCED_MARCH;
use crate::codemp::game::g_combat::G_RadiusDamage;
use crate::codemp::game::g_exphysics::G_RunExPhys;
use crate::codemp::game::g_weapon::WP_FireTurretMissile;
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_ai_jedi::{Jedi_Cloak, Jedi_Decloak};
use crate::codemp::game::g_log::{G_LogWeaponItem, G_LogWeaponPickup, G_LogWeaponPowerup};
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, CON_CONNECTED, CON_DISCONNECTED, DAMAGE_DEATH_KNOCKBACK, FL_BOUNCE_HALF,
    FL_DROPPED_ITEM,
    FL_NOTARGET, FL_TEAMSLAVE, FRAMETIME,
};
use crate::codemp::game::g_main::{
    g_adaptRespawn, g_duelWeaponDisable, g_entities, g_forcePowerDisable, g_gametype,
    g_weaponDisable, g_weaponRespawn, level, Com_Printf, G_Error, G_LogPrintf, G_Printf, G_RunThink,
};
use crate::codemp::game::anims::{BOTH_STRAFE_LEFT1, BOTH_STRAFE_RIGHT1};
use crate::codemp::game::g_spawn::G_SpawnFloat;
use crate::codemp::game::g_public_h::{SVF_BROADCAST, SVF_NOCLIENT, SVF_SINGLECLIENT};
use crate::codemp::game::g_team::{
    OnSameTeam, Pickup_Team, Team_CheckDroppedItem, Team_DroppedFlagThink, Team_FreeEntity,
    Team_InitGame,
};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_GONK, CLASS_MARK1, CLASS_MARK2, CLASS_MOUSE, CLASS_PROBE, CLASS_PROTOCOL,
    CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY, CLASS_UGNAUGHT,
    CLASS_VEHICLE, CLASS_WAMPA,
};
use crate::codemp::game::g_utils::{
    vtos, G_AddEvent, G_AddPredictableEvent, G_BoneIndex, G_EffectIndex, G_FreeEntity,
    G_ModelIndex, G_PlayEffect, G_PlayEffectID, G_RadiusList, G_ScaleNetHealth, G_SetAnim,
    G_SetOrigin, G_Sound, G_SoundIndex, G_Spawn, G_TempEntity, G_UseTargets,
};
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::ghoul2::g2_h::{
    BONE_ANGLES_POSTMULT, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE_FREEZE,
};
use crate::codemp::game::w_saber::HasSetSaberOnly;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize360, AngleSubtract, AngleVectors, DotProduct,
    VectorAdd, VectorCopy, VectorLength, VectorLengthSquared, VectorMA, VectorNormalize,
    VectorClear, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{crandom, random, Com_sprintf, Sz};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, qhandle_t, trace_t, vec3_t, BUTTON_ATTACK, CHAN_AUTO, CHAN_BODY, ENTITYNUM_NONE,
    ENTITYNUM_WORLD, FORCE_DARKSIDE,
    FORCE_LIGHTSIDE, MAX_CLIENTS, MAX_GENTITIES, NEGATIVE_X, NEGATIVE_Y, NEGATIVE_Z, ORIGIN, PITCH,
    POSITIVE_Y, ROLL, TR_GRAVITY, TR_STATIONARY, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_LIGHTSABER, CONTENTS_NODROP, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP,
    CONTENTS_SOLID, CONTENTS_TRIGGER,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap::{self, SnapVector};

// The retail (non-`Q3_VM`) build links the C library's `strcmp` (the `g_main.rs`/`g_team.rs`
// precedent) — `LaunchItem` uses it to recognize the CTF flag classnames.
extern "C" {
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

// g_object.c:72 — `G_RunObject` is the per-frame physics `think` for ROFF/movable objects;
// the sentry-gun cluster (`pas_fire`/`SP_PAS`) installs/runs it. Now ported in g_object.rs,
// so it is imported directly (the prior `extern "C"` forward-decl is dropped).
use crate::codemp::game::g_object::G_RunObject;

/// `g_items.c:2979` — `qboolean itemRegistered[MAX_ITEMS]`. Marks which entries of
/// [`bg_itemlist`] have been registered (and so must be precached on the client). Written
/// by [`RegisterItem`]; serialized to the `CS_ITEMS` config string by [`SaveRegisteredItems`].
pub static mut itemRegistered: [qboolean; MAX_ITEMS as usize] = [QFALSE; MAX_ITEMS as usize];

// g_items.c:101-105 — the personal-shield sound handles. File-static `qhandle_t`s, all 0
// until `PlaceShield` (g_items.c:389) lazily registers them on first use; each is then read
// by the shield think/touch/pain/remove callbacks. `ShieldRemove` reads
// `shieldDeactivateSound`; the rest land with the remaining shield cluster.
pub static mut shieldLoopSound: qhandle_t = 0;
pub static mut shieldAttachSound: qhandle_t = 0;
pub static mut shieldActivateSound: qhandle_t = 0;
pub static mut shieldDeactivateSound: qhandle_t = 0;
pub static mut shieldDamageSound: qhandle_t = 0;

// g_items.c:91-99 — personal-shield health defines. `SHIELD_HEALTH_DEC` (=10) and
// `SHIELD_SIEGE_HEALTH_DEC` (= SHIELD_SIEGE_HEALTH/25 = 2000/25 = 80) are the per-second
// health drain `ShieldThink` subtracts; both give the shield a 25-second lifetime.
const SHIELD_HEALTH: c_int = 250;
const SHIELD_HEALTH_DEC: c_int = 10;
const MAX_SHIELD_HEIGHT: c_int = 254;
const MAX_SHIELD_HALFWIDTH: c_int = 255;
const SHIELD_HALFTHICKNESS: c_int = 4;
const SHIELD_PLACEDIST: c_int = 64;
const SHIELD_SIEGE_HEALTH: c_int = 2000;
const SHIELD_SIEGE_HEALTH_DEC: c_int = SHIELD_SIEGE_HEALTH / 25;

// g_items.c:21-27 — item respawn-time defines. Each lands with its first reader:
// RESPAWN_AMMO via adjustRespawnTime / Pickup_Ammo; RESPAWN_ARMOR via Pickup_Armor;
// RESPAWN_HEALTH + RESPAWN_MEGAHEALTH via Pickup_Health;
// RESPAWN_TEAM_WEAPON via Pickup_Weapon; RESPAWN_POWERUP via Pickup_Powerup.
const RESPAWN_TEAM_WEAPON: c_int = 30;
const RESPAWN_HOLDABLE: c_int = 60;
const RESPAWN_ARMOR: c_int = 20;
const RESPAWN_HEALTH: c_int = 30;
const RESPAWN_AMMO: c_int = 40;
const RESPAWN_POWERUP: c_int = 120;
const RESPAWN_MEGAHEALTH: c_int = 120;

// g_items.c:30 — item spawnflag bit: the item is suspended in the air (don't drop to floor).
const ITMSF_SUSPEND: c_int = 1;

// g_items.c:32 — item spawnflag bit read by Touch_Item: set on an item entity to allow
// NPCs (non-players) to pick it up.
const ITMSF_ALLOWNPC: c_int = 4;

// g_items.c:1274-1275 — tossed-special-item lifetime/owner-no-touch windows (#define, ms).
const TOSSED_ITEM_STAY_PERIOD: c_int = 20000;
const TOSSED_ITEM_OWNER_NOTOUCH_DUR: c_int = 1000;

// g_items.c:42 — medpack heal amounts (#define).
const MAX_MEDPACK_HEAL_AMOUNT: c_int = 25;
const MAX_MEDPACK_BIG_HEAL_AMOUNT: c_int = 50;

/// Port of `g_items.c:47` — `adjustRespawnTime`. Scales an item's base respawn time by
/// the number of playing clients (when `g_adaptRespawn` is set), with a special case
/// that forces the thermal/trip-mine/det-pack weapons onto the ammo respawn rate.
///
/// Oracle-checked bit-exact via [`tests`] against the extracted C: the body reads only
/// `g_adaptRespawn.integer` and `level.numPlayingClients`, which the oracle wrapper
/// lifts to parameters. The C `respawnTime *= 20.0 / (float)(…)` mixes a `double`
/// literal with a `float` divisor, so the multiply promotes `respawnTime` (float) to
/// `double` and truncates back on store — reproduced here via explicit `f64` casts.
pub fn adjustRespawnTime(preRespawnTime: f32, itemType: itemType_t, itemTag: weapon_t) -> c_int {
    let mut respawnTime: f32 = preRespawnTime;

    if itemType == IT_WEAPON
        && (itemTag == WP_THERMAL || itemTag == WP_TRIP_MINE || itemTag == WP_DET_PACK)
    {
        // special case for these, use ammo respawn rate
        respawnTime = RESPAWN_AMMO as f32;
    }

    let g_adapt = unsafe { (*addr_of!(g_adaptRespawn)).integer };
    if g_adapt == 0 {
        return respawnTime as c_int;
    }

    let numPlayingClients = unsafe { (*addr_of!(level)).numPlayingClients };
    if numPlayingClients > 4 {
        // Start scaling the respawn times.
        if numPlayingClients > 32 {
            // 1/4 time minimum.
            respawnTime = (respawnTime as f64 * 0.25) as f32;
        } else if numPlayingClients > 12 {
            // From 12-32, scale from 0.5 to 0.25;
            respawnTime =
                (respawnTime as f64 * (20.0f64 / (numPlayingClients + 8) as f32 as f64)) as f32;
        } else {
            // From 4-12, scale from 1.0 to 0.5;
            respawnTime =
                (respawnTime as f64 * (8.0f64 / (numPlayingClients + 4) as f32 as f64)) as f32;
        }
    }

    if respawnTime < 1.0 {
        // No matter what, don't go lower than 1 second, or the pickups become very noisy!
        respawnTime = 1.0;
    }

    respawnTime as c_int
}

/// Port of `g_items.c:3033` — `RegisterItem`. Mark `item` as registered so it gets added
/// to the precache list (later serialized by [`SaveRegisteredItems`]). Computes the item's
/// index as the pointer offset into [`bg_itemlist`] (faithful to the C `item - bg_itemlist`).
///
/// No oracle: a NULL guard that calls the diverging [`G_Error`] plus a write to the
/// `itemRegistered` process-global (no extractable computation) — same no-oracle precedent
/// as the other trap/control-flow ports.
///
/// # Safety
/// `item`, when non-NULL, must point into the live [`bg_itemlist`] table so the offset is
/// a valid in-bounds index.
pub unsafe fn RegisterItem(item: *mut gitem_t) {
    if item.is_null() {
        G_Error("RegisterItem: NULL");
    }
    let idx = item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as usize;
    itemRegistered[idx] = QTRUE;
}

/// Port of `g_items.c:3049` — `SaveRegisteredItems`. Build a `bg_numItems`-long string of
/// `'1'`/`'0'` flags (one per item in [`bg_itemlist`], from [`itemRegistered`]) and write it
/// to the `CS_ITEMS` config string so the client knows which items to precache.
///
/// No oracle: ends in a `trap_SetConfigstring` engine syscall (unavailable in the harness)
/// and reads the `itemRegistered` process-global — same no-oracle precedent as the other
/// trap-bound ports.
pub unsafe fn SaveRegisteredItems() {
    let mut string = [0u8; MAX_ITEMS as usize + 1];
    // C also tracks `count`, but it is read only by a commented-out `G_Printf`, so it is dropped.

    for i in 0..bg_numItems as usize {
        if itemRegistered[i] != QFALSE {
            string[i] = b'1';
        } else {
            string[i] = b'0';
        }
    }
    string[bg_numItems as usize] = 0;

    //	G_Printf( "%i items registered\n", count );
    let s = core::str::from_utf8_unchecked(&string[..bg_numItems as usize]);
    trap::SetConfigstring(CS_ITEMS, s);
}

/// `void G_CheckTeamItems( void )` (g_items.c:3093) — round-start team-item setup: call
/// [`Team_InitGame`], then for CTF/CTY verify the map provides both base flags
/// (`team_CTF_redflag` / `team_CTF_blueflag`), warning via [`G_Printf`] if either is absent
/// or unregistered. The registration check is the C `itemRegistered[ item - bg_itemlist ]`
/// pointer-offset index.
///
/// No oracle: drives the `Team_InitGame` / `g_gametype` globals, [`BG_FindItem`], the
/// `itemRegistered` process-global, and `G_Printf` — same no-oracle precedent as the other
/// global/trap-bound ports.
///
/// # Safety
/// Touches the `g_gametype` / `itemRegistered` / `teamgame` file globals; call only on the
/// game thread.
pub unsafe fn G_CheckTeamItems() {
    // Set up team stuff
    Team_InitGame();

    if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY {
        // check for the two flags
        let mut item = BG_FindItem(c"team_CTF_redflag".as_ptr());
        if item.is_null()
            || itemRegistered[item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as usize]
                == QFALSE
        {
            G_Printf("^3WARNING: No team_CTF_redflag in map");
        }
        item = BG_FindItem(c"team_CTF_blueflag".as_ptr());
        if item.is_null()
            || itemRegistered[item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as usize]
                == QFALSE
        {
            G_Printf("^3WARNING: No team_CTF_blueflag in map");
        }
    }
}

/// Port of `g_items.c:3074` — `G_ItemDisabled`. Returns the value of the
/// `disable_<classname>` cvar for this item (nonzero = the item is disabled on this
/// server). Builds the cvar name into a 128-byte buffer (matching C's `char name[128]`),
/// then reads it through `trap_Cvar_VariableIntegerValue`.
///
/// No oracle: the body is a `trap_Cvar_*` call, which has no extractable C oracle (the
/// engine syscall is unavailable in the test harness) — same no-oracle precedent as the
/// other trap-bound control-flow ports.
pub unsafe fn G_ItemDisabled(item: *mut gitem_t) -> c_int {
    let mut name = [0 as c_char; 128];

    Com_sprintf(
        name.as_mut_ptr(),
        name.len() as c_int,
        format_args!("disable_{}", Sz((*item).classname)),
    );
    trap::Cvar_VariableIntegerValue(&Sz(name.as_ptr()).to_string())
}

/// Port of `g_items.c:3143` — `G_BounceItem`. Reflect a dropped item's velocity off the
/// surface it just hit, scaling the new `s.pos.trDelta` by `physicsBounce` so it bleeds
/// speed instead of bouncing forever. Det-packs fire their `touch` callback and bail;
/// once the item settles onto a near-flat surface with little vertical speed it is snapped
/// to rest on the ground; holocrons and sentry guns re-fire `touch` after coming to rest.
/// Twin of `G_BounceObject` (g_object.c) / `G_BounceMissile` (g_missile.c).
///
/// No oracle: mutates a `gentity_t` and invokes the `ent->touch` function pointer
/// (control-flow over engine entity state) — same no-oracle precedent as `G_BounceObject`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `trace` to a valid `trace_t`; and
/// `g_entities` must point to the live entity array (indexed by `trace->entityNum`).
pub unsafe fn G_BounceItem(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut velocity: vec3_t = [0.0; 3];

    // reflect the velocity on the trace plane
    let hitTime = ((*addr_of!(level)).previousTime as f32
        + ((*addr_of!(level)).time - (*addr_of!(level)).previousTime) as f32 * (*trace).fraction)
        as c_int;
    BG_EvaluateTrajectoryDelta(&(*ent).s.pos, hitTime, &mut velocity);
    let dot = DotProduct(&velocity, &(*trace).plane.normal);
    VectorMA(
        &velocity,
        -2.0 * dot,
        &(*trace).plane.normal,
        &mut (*ent).s.pos.trDelta,
    );

    // cut the velocity to keep from bouncing forever
    let trDelta = (*ent).s.pos.trDelta;
    VectorScale(&trDelta, (*ent).physicsBounce, &mut (*ent).s.pos.trDelta);

    if (*ent).s.weapon == WP_DET_PACK
        && (*ent).s.eType == ET_GENERAL
        && (*ent).physicsObject != 0
    {
        // detpacks only
        if let Some(touch) = (*ent).touch {
            touch(ent, core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*trace).entityNum as usize), trace);
            return;
        }
    }

    // check for stop
    if (*trace).plane.normal[2] > 0.0 && (*ent).s.pos.trDelta[2] < 40.0 {
        (*trace).endpos[2] += 1.0; // make sure it is off ground
        SnapVector(&mut (*trace).endpos);
        G_SetOrigin(ent, &(*trace).endpos);
        (*ent).s.groundEntityNum = (*trace).entityNum as c_int;
        return;
    }

    let currentOrigin = (*ent).r.currentOrigin;
    VectorAdd(
        &currentOrigin,
        &(*trace).plane.normal,
        &mut (*ent).r.currentOrigin,
    );
    let currentOrigin = (*ent).r.currentOrigin;
    VectorCopy(&currentOrigin, &mut (*ent).s.pos.trBase);
    (*ent).s.pos.trTime = (*addr_of!(level)).time;

    if (*ent).s.eType == ET_HOLOCRON
        || ((*ent).s.shouldtarget != 0 && (*ent).s.eType == ET_GENERAL && (*ent).physicsObject != 0)
    {
        // holocrons and sentry guns
        if let Some(touch) = (*ent).touch {
            touch(ent, core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*trace).entityNum as usize), trace);
        }
    }
}

/// Port of `g_items.c:3196` — `G_RunItem`. Per-frame physics step for a dropped/launched
/// item: snaps a free-falling item back onto a gravity trajectory, traces from its
/// previous to its current position, relinks + runs its think, and (when it impacts)
/// either frees it in a nodrop volume (team items go through [`Team_FreeEntity`], the rest
/// through [`G_FreeEntity`]) or bounces it via [`G_BounceItem`].
///
/// No oracle: drives the world via [`trap::Trace`]/[`trap::LinkEntity`]/
/// [`trap::PointContents`] and mutates a `gentity_t` — same precedent as the
/// already-ported `G_RunMissile`/`G_BounceItem`.
pub unsafe fn G_RunItem(ent: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];
    let mask: c_int;

    // if groundentity has been set to -1, it may have been pushed off an edge
    if (*ent).s.groundEntityNum == -1 {
        if (*ent).s.pos.trType != TR_GRAVITY {
            (*ent).s.pos.trType = TR_GRAVITY;
            (*ent).s.pos.trTime = (*addr_of!(level)).time;
        }
    }

    if (*ent).s.pos.trType == TR_STATIONARY {
        // check think function
        G_RunThink(ent);
        return;
    }

    // get current position
    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);

    // trace a line from the previous position to the current position
    if (*ent).clipmask != 0 {
        mask = (*ent).clipmask;
    } else {
        mask = MASK_PLAYERSOLID & !CONTENTS_BODY; //MASK_SOLID;
    }
    let mut tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &origin,
        (*ent).r.ownerNum,
        mask,
    );

    VectorCopy(&tr.endpos, &mut (*ent).r.currentOrigin);

    if tr.startsolid != 0 {
        tr.fraction = 0.0;
    }

    trap::LinkEntity(ent); // FIXME: avoid this for stationary?

    // check think function
    G_RunThink(ent);

    if tr.fraction == 1.0 {
        return;
    }

    // if it is in a nodrop volume, remove it
    let contents = trap::PointContents(&(*ent).r.currentOrigin, -1);
    if contents & CONTENTS_NODROP != 0 {
        if !(*ent).item.is_null() && (*(*ent).item).giType == IT_TEAM {
            Team_FreeEntity(ent);
        } else {
            G_FreeEntity(ent);
        }
        return;
    }

    G_BounceItem(ent, &mut tr);
}

/// Port of `g_items.c:2131` — `Add_Ammo`. Gives `count` rounds of the given ammo
/// type to `ent`'s client, clamped to the per-type maximum from [`ammoData`]. The C
/// guards the whole add on the current count being below max, so an already-full slot
/// is left untouched (a negative `count` can never push it negative through this path).
///
/// No oracle: mutates a `gentity_t`'s `client->ps.ammo[]` through the (non-extractable)
/// entity/client graph — same precedent as the other entity-mutating g_items ports.
pub unsafe fn Add_Ammo(ent: *mut gentity_t, weapon: c_int, count: c_int) {
    let ps = &mut (*(*ent).client).ps;
    if ps.ammo[weapon as usize] < ammoData[weapon as usize].max {
        ps.ammo[weapon as usize] += count;
        if ps.ammo[weapon as usize] > ammoData[weapon as usize].max {
            ps.ammo[weapon as usize] = ammoData[weapon as usize].max;
        }
    }
}

/// Port of `g_items.c:2143` — `Pickup_Ammo`. Awards ammo to `other`'s client when it
/// touches the ammo item `ent`, then returns the respawn delay from [`adjustRespawnTime`].
///
/// A `giTag == -1` item is an `ammo_all` pickup: it tops up the four base ammo types
/// (siege gives larger amounts plus thermal/trip-mine/det-pack rounds if the player owns
/// those weapons, via the [`STAT_WEAPONS`] bitfield); otherwise the item's `giTag` ammo
/// type gets `ent.count` rounds (falling back to the item's base `quantity`). All adds go
/// through [`Add_Ammo`], which clamps to the per-type max.
///
/// No oracle: walks the `gentity_t`→`client`→`ps` graph and reads the `g_gametype`
/// cvar — same precedent as [`Add_Ammo`] and the other entity-mutating g_items ports.
pub unsafe fn Pickup_Ammo(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let quantity = if (*ent).count != 0 {
        (*ent).count
    } else {
        (*(*ent).item).quantity
    };

    if (*(*ent).item).giTag == -1 {
        // an ammo_all, give them a bit of everything
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            // complaints that siege tech's not giving enough ammo.  Does anything else use ammo all?
            Add_Ammo(other, AMMO_BLASTER, 100);
            Add_Ammo(other, AMMO_POWERCELL, 100);
            Add_Ammo(other, AMMO_METAL_BOLTS, 100);
            Add_Ammo(other, AMMO_ROCKETS, 5);
            if (*(*other).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_DET_PACK) != 0 {
                Add_Ammo(other, AMMO_DETPACK, 2);
            }
            if (*(*other).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_THERMAL) != 0 {
                Add_Ammo(other, AMMO_THERMAL, 2);
            }
            if (*(*other).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_TRIP_MINE) != 0 {
                Add_Ammo(other, AMMO_TRIPMINE, 2);
            }
        } else {
            Add_Ammo(other, AMMO_BLASTER, 50);
            Add_Ammo(other, AMMO_POWERCELL, 50);
            Add_Ammo(other, AMMO_METAL_BOLTS, 50);
            Add_Ammo(other, AMMO_ROCKETS, 2);
        }
    } else {
        Add_Ammo(other, (*(*ent).item).giTag, quantity);
    }

    adjustRespawnTime(RESPAWN_AMMO as f32, (*(*ent).item).giType, (*(*ent).item).giTag)
}

//======================================================================

/// Port of `g_items.c:2037` — `Pickup_Powerup`. Activates the item's powerup on `other`
/// (seconds-rounded start time if not already active), adds `count`-or-`quantity` seconds
/// of duration, and (for `PW_YSALAMIRI`) clears the three force-aura powerups. Then it
/// gives every nearby, alive, opposing client a "denied" anti-reward — a line-of-sight
/// `trap_Trace`-confirmed, range-(192)- and facing-(`DotProduct >= 0.4`)-gated toggle of
/// `PLAYEREVENT_DENIEDREWARD` in their `PERS_PLAYEREVENTS`. Returns [`RESPAWN_POWERUP`].
///
/// No oracle: walks `level.clients`, fires [`trap::Trace`], and mutates the
/// `gentity_t`→`client`→`ps` graph — same precedent as the rest of the pickup family.
pub unsafe fn Pickup_Powerup(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let quantity: c_int;
    let mut i: c_int;
    let mut client: *mut gclient_t;

    let lvl = addr_of!(level);

    if (*(*other).client).ps.powerups[(*(*ent).item).giTag as usize] == 0 {
        // round timing to seconds to make multiple powerup timers
        // count in sync
        (*(*other).client).ps.powerups[(*(*ent).item).giTag as usize] =
            (*lvl).time - ((*lvl).time % 1000);

        G_LogWeaponPowerup((*other).s.number, (*(*ent).item).giTag);
    }

    if (*ent).count != 0 {
        quantity = (*ent).count;
    } else {
        quantity = (*(*ent).item).quantity;
    }

    (*(*other).client).ps.powerups[(*(*ent).item).giTag as usize] += quantity * 1000;

    if (*(*ent).item).giTag == PW_YSALAMIRI {
        (*(*other).client).ps.powerups[PW_FORCE_ENLIGHTENED_LIGHT as usize] = 0;
        (*(*other).client).ps.powerups[PW_FORCE_ENLIGHTENED_DARK as usize] = 0;
        (*(*other).client).ps.powerups[PW_FORCE_BOON as usize] = 0;
    }

    // give any nearby players a "denied" anti-reward
    i = 0;
    while i < (*lvl).maxclients {
        let mut delta: vec3_t = [0.0; 3];
        let len: f32;
        let mut forward: vec3_t = [0.0; 3];
        let tr: trace_t;

        client = (*lvl).clients.offset(i as isize);
        if client == (*other).client {
            i += 1;
            continue;
        }
        if (*client).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if (*client).ps.stats[STAT_HEALTH as usize] <= 0 {
            i += 1;
            continue;
        }

        // if same team in team game, no sound
        // cannot use OnSameTeam as it expects to g_entities, not clients
        if (*addr_of!(g_gametype)).integer >= GT_TEAM
            && (*(*other).client).sess.sessionTeam == (*client).sess.sessionTeam
        {
            i += 1;
            continue;
        }

        // if too far away, no sound
        VectorSubtract(&(*ent).s.pos.trBase, &(*client).ps.origin, &mut delta);
        len = VectorNormalize(&mut delta);
        if len > 192.0 {
            i += 1;
            continue;
        }

        // if not facing, no sound
        AngleVectors(&(*client).ps.viewangles, Some(&mut forward), None, None);
        if DotProduct(&delta, &forward) < 0.4 {
            i += 1;
            continue;
        }

        // if not line of sight, no sound
        tr = trap::Trace(
            &(*client).ps.origin,
            &vec3_origin,
            &vec3_origin,
            &(*ent).s.pos.trBase,
            ENTITYNUM_NONE,
            CONTENTS_SOLID,
        );
        if tr.fraction != 1.0 {
            i += 1;
            continue;
        }

        // anti-reward
        (*client).ps.persistant[PERS_PLAYEREVENTS as usize] ^= PLAYEREVENT_DENIEDREWARD;
        i += 1;
    }
    RESPAWN_POWERUP
}

//======================================================================

/// Port of `g_items.c:2117` — `Pickup_Holdable`. Sets `other`'s `STAT_HOLDABLE_ITEM` to
/// the item's index in [`bg_itemlist`] (the C `ent->item - bg_itemlist` pointer offset),
/// marks the item owned in the `STAT_HOLDABLE_ITEMS` bitfield, then returns the scaled
/// [`RESPAWN_HOLDABLE`] delay.
///
/// No oracle: walks the `gentity_t`→`client`→`ps` graph and reads `bg_itemlist` — same
/// precedent as [`Pickup_Ammo`] / [`RegisterItem`].
pub unsafe fn Pickup_Holdable(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let idx = ((*ent).item as *const gitem_t)
        .offset_from(addr_of!(bg_itemlist) as *const gitem_t) as c_int;
    (*(*other).client).ps.stats[STAT_HOLDABLE_ITEM as usize] = idx;

    (*(*other).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] |= 1 << (*(*ent).item).giTag;

    G_LogWeaponItem((*other).s.number, (*(*ent).item).giTag);

    adjustRespawnTime(
        RESPAWN_HOLDABLE as f32,
        (*(*ent).item).giType,
        (*(*ent).item).giTag,
    )
}

//======================================================================

/// Port of `g_items.c:2193` — `Pickup_Weapon`. Grants the weapon `ent`'s `giTag` to
/// `other` (setting its [`STAT_WEAPONS`] ownership bit) and tops up the matching ammo
/// slot, then returns the respawn delay. A negative `ent.count` grants the weapon with no
/// ammo; otherwise the award is `ent.count` rounds (falling back to the item's base
/// `quantity`). For non-dropped items outside team deathmatch the "respawning rules"
/// taper the award: if the player holds less than half the minimum they are topped up to
/// the minimum, else they get only half. Ammo is added via [`Add_Ammo`] to
/// `weaponData[giTag].ammoIndex`. Team DM uses [`RESPAWN_TEAM_WEAPON`]; otherwise the
/// `g_weaponRespawn` cvar value drives [`adjustRespawnTime`].
///
/// No oracle: walks the `gentity_t`→`client`→`ps` graph and reads the `g_gametype` /
/// `g_weaponRespawn` cvars — same precedent as [`Pickup_Ammo`] and the other
/// entity-mutating g_items ports. The C `quantity*0.5` mixes the `int` quantity with a
/// `double` literal, so the compare/store promote to `double` and truncate back on the
/// `int` assignment — reproduced here via explicit `f64` casts.
pub unsafe fn Pickup_Weapon(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut quantity: c_int;

    if (*ent).count < 0 {
        quantity = 0; // None for you, sir!
    } else {
        if (*ent).count != 0 {
            quantity = (*ent).count;
        } else {
            quantity = (*(*ent).item).quantity;
        }

        // dropped items and teamplay weapons always have full ammo
        if (*ent).flags & FL_DROPPED_ITEM == 0 && (*addr_of!(g_gametype)).integer != GT_TEAM {
            // respawning rules

            // New method:  If the player has less than half the minimum, give them the minimum, else add 1/2 the min.

            // drop the quantity if the already have over the minimum
            if ((*(*other).client).ps.ammo[(*(*ent).item).giTag as usize] as f64)
                < quantity as f64 * 0.5
            {
                quantity -= (*(*other).client).ps.ammo[(*(*ent).item).giTag as usize];
            } else {
                quantity = (quantity as f64 * 0.5) as c_int; // only add half the value.
            }
        }
    }

    // add the weapon
    (*(*other).client).ps.stats[STAT_WEAPONS as usize] |= 1 << (*(*ent).item).giTag;

    Add_Ammo(other, weaponData[(*(*ent).item).giTag as usize].ammoIndex, quantity);

    G_LogWeaponPickup((*other).s.number, (*(*ent).item).giTag);

    // team deathmatch has slow weapon respawns
    if (*addr_of!(g_gametype)).integer == GT_TEAM {
        return adjustRespawnTime(
            RESPAWN_TEAM_WEAPON as f32,
            (*(*ent).item).giType,
            (*(*ent).item).giTag,
        );
    }

    adjustRespawnTime(
        (*addr_of!(g_weaponRespawn)).integer as f32,
        (*(*ent).item).giType,
        (*(*ent).item).giTag,
    )
}

//======================================================================

/// Port of `g_items.c:2250` — `Pickup_Health`. Heals `other` when it touches the health
/// item `ent`, then returns the respawn delay. Small (5) and mega (100) healths overheal to
/// twice `STAT_MAX_HEALTH`; all other healths cap at `STAT_MAX_HEALTH`. The award is
/// `ent.count` rounds, falling back to the item's base `quantity`. Mega health (quantity
/// 100) returns the fixed slow [`RESPAWN_MEGAHEALTH`]; everything else scales via
/// [`adjustRespawnTime`].
///
/// No oracle: walks the `gentity_t`→`client`→`ps` graph and mutates `other.health` — same
/// precedent as [`Pickup_Ammo`] and the other entity-mutating g_items ports.
pub unsafe fn Pickup_Health(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let max: c_int;
    let quantity: c_int;

    // small and mega healths will go over the max
    if (*(*ent).item).quantity != 5 && (*(*ent).item).quantity != 100 {
        max = (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize];
    } else {
        max = (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize] * 2;
    }

    if (*ent).count != 0 {
        quantity = (*ent).count;
    } else {
        quantity = (*(*ent).item).quantity;
    }

    (*other).health += quantity;

    if (*other).health > max {
        (*other).health = max;
    }
    (*(*other).client).ps.stats[STAT_HEALTH as usize] = (*other).health;

    if (*(*ent).item).quantity == 100 {
        // mega health respawns slow
        return RESPAWN_MEGAHEALTH;
    }

    adjustRespawnTime(RESPAWN_HEALTH as f32, (*(*ent).item).giType, (*(*ent).item).giTag)
}

//======================================================================

/// Port of `g_items.c:2283` — `Pickup_Armor`. Adds the armor item's `quantity` to
/// `other`'s `STAT_ARMOR`, capped at `STAT_MAX_HEALTH * giTag` (the item's giTag is the
/// armor multiplier), then returns the scaled [`RESPAWN_ARMOR`] delay.
///
/// No oracle: walks the `gentity_t`→`client`→`ps` graph and mutates the armor stat — same
/// precedent as [`Pickup_Health`] / [`Pickup_Ammo`].
pub unsafe fn Pickup_Armor(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    (*(*other).client).ps.stats[STAT_ARMOR as usize] += (*(*ent).item).quantity;
    if (*(*other).client).ps.stats[STAT_ARMOR as usize]
        > (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize] * (*(*ent).item).giTag
    {
        (*(*other).client).ps.stats[STAT_ARMOR as usize] =
            (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize] * (*(*ent).item).giTag;
    }

    adjustRespawnTime(RESPAWN_ARMOR as f32, (*(*ent).item).giType, (*(*ent).item).giTag)
}

//======================================================================

/// Port of `g_items.c:2301` — `RespawnItem`. Brings a respawned item back: for a
/// teamed item it randomly re-selects one member of the spawn team (a `rand() % count`
/// pick walked off `teammaster`; a missing master is fatal via [`G_Error`]), then
/// re-enables it — `CONTENTS_TRIGGER`, clear `EF_NODRAW`/`EF_ITEMPLACEHOLDER` and
/// `SVF_NOCLIENT`, relink — and plays the respawn sounds (a global/general broadcast
/// `sound/items/respawn1` temp-entity for powerups, plus the nearby-only
/// `EV_ITEM_RESPAWN` event), finally clearing `nextthink`.
///
/// No oracle: re-selects/relinks live `gentity_t`s, fires [`G_Error`]/[`trap::LinkEntity`]/
/// [`G_TempEntity`]/[`G_AddEvent`], and draws the shared LCG via [`rand`] — same
/// trap/entity-graph precedent as the rest of the g_items pickup family.
pub unsafe extern "C" fn RespawnItem(mut ent: *mut gentity_t) {
    // randomly select from teamed entities
    if !(*ent).team.is_null() {
        let master: *mut gentity_t;
        let mut count: c_int;
        let choice: c_int;

        if (*ent).teammaster.is_null() {
            G_Error("RespawnItem: bad teammaster");
        }
        master = (*ent).teammaster;

        count = 0;
        ent = master;
        while !ent.is_null() {
            ent = (*ent).teamchain;
            count += 1;
        }

        choice = rand() % count;

        count = 0;
        ent = master;
        while count < choice {
            ent = (*ent).teamchain;
            count += 1;
        }
    }

    (*ent).r.contents = CONTENTS_TRIGGER;
    //ent->s.eFlags &= ~EF_NODRAW;
    (*ent).s.eFlags &= !(EF_NODRAW | EF_ITEMPLACEHOLDER);
    (*ent).r.svFlags &= !SVF_NOCLIENT;
    trap::LinkEntity(ent);

    if (*(*ent).item).giType == IT_POWERUP {
        // play powerup spawn sound to all clients
        let te: *mut gentity_t;

        // if the powerup respawn sound should Not be global
        if (*ent).speed != 0.0 {
            te = G_TempEntity(&(*ent).s.pos.trBase, EV_GENERAL_SOUND);
        } else {
            te = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_SOUND);
        }
        (*te).s.eventParm = G_SoundIndex("sound/items/respawn1");
        (*te).r.svFlags |= SVF_BROADCAST;
    }

    // play the normal respawn sound only to nearby clients
    G_AddEvent(ent, EV_ITEM_RESPAWN, 0);

    (*ent).nextthink = 0;
}

/// Port of `g_items.c:2778` — `Use_Item`. The item entity's `use` callback (installed by
/// `FinishSpawningItem`): respawning the item, which just defers to [`RespawnItem`].
///
/// `extern "C"`: stored in the `use` function-pointer slot of a `gentity_t`.
///
/// No oracle: a one-line delegation to [`RespawnItem`] (itself a trap/entity-graph fn).
pub unsafe extern "C" fn Use_Item(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    RespawnItem(ent);
}

/// Port of `g_items.c:472` — `ItemUse_Binoculars`. Toggles the binocular zoom mode on
/// the using client: when not zoomed (`zoomMode == 0`) it switches into binocular zoom
/// (`zoomMode = 2`, unlocked, `zoomFov = 40`); when already in binocular zoom it returns
/// to normal and stamps `zoomTime = level.time`. Null/no-client guards and a
/// `weaponstate != WEAPON_READY` guard (so it can't be re-fired mid weapon-switch).
///
/// No oracle: pure `gentity_t`→`client`→`ps` mutation reading `level.time`.
pub unsafe fn ItemUse_Binoculars(ent: *mut gentity_t) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    if (*(*ent).client).ps.weaponstate != WEAPON_READY {
        //So we can't fool it and reactivate while switching to the saber or something.
        return;
    }

    /*
    if (ent->client->ps.weapon == WP_SABER)
    { //No.
        return;
    }
    */

    if (*(*ent).client).ps.zoomMode == 0 {
        // not zoomed or currently zoomed with the disruptor
        (*(*ent).client).ps.zoomMode = 2;
        (*(*ent).client).ps.zoomLocked = QFALSE;
        (*(*ent).client).ps.zoomFov = 40.0;
    } else if (*(*ent).client).ps.zoomMode == 2 {
        (*(*ent).client).ps.zoomMode = 0;
        (*(*ent).client).ps.zoomTime = (*addr_of!(level)).time;
    }
}

/// Port of `g_items.c:1096` — `ItemUse_Seeker`. Activates the seeker drone holdable:
/// sets `EF_SEEKERDRONE`, an existence timeout (`level.time + 30000`) and a first-fire
/// time (`level.time + 1500`). The original's `GT_SIEGE`+`d_siegeSeekerNPC` branch spawns
/// a real `remote` NPC via `NPC_SpawnType` (not-yet-ported NPC subsystem); since that path is
/// only reachable in SIEGE — a gametype not yet wired — only the standard drone branch is
/// ported here. See DEVIATIONS.
///
/// No oracle: `gentity_t`→`client`→`ps` mutation reading `level.time`.
pub unsafe fn ItemUse_Seeker(ent: *mut gentity_t) {
    // NOTE: the GT_SIEGE / d_siegeSeekerNPC branch (NPC_SpawnType "remote") is not yet
    // ported, pending the NPC subsystem; only the standard drone activation is ported.
    (*(*ent).client).ps.eFlags |= EF_SEEKERDRONE;
    (*(*ent).client).ps.droneExistTime = ((*addr_of!(level)).time + 30000) as f32;
    (*(*ent).client).ps.droneFireTime = ((*addr_of!(level)).time + 1500) as f32;
}

/// Port of `g_items.c:1127` — `MedPackGive` (file-static helper). Heals `ent` by `amount`,
/// capped at `STAT_MAX_HEALTH`. No-ops on a null/no-client entity, a dead entity
/// (`health <= 0` / `STAT_HEALTH <= 0` / `EF_DEAD`), or one already at max health.
///
/// No oracle: `gentity_t`→`client`→`ps` mutation (entity-graph precedent).
unsafe fn MedPackGive(ent: *mut gentity_t, amount: c_int) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    if (*ent).health <= 0
        || (*(*ent).client).ps.stats[STAT_HEALTH as usize] <= 0
        || ((*(*ent).client).ps.eFlags & EF_DEAD) != 0
    {
        return;
    }

    if (*ent).health >= (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
        return;
    }

    (*ent).health += amount;

    if (*ent).health > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
        (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
    }
}

/// Port of `g_items.c:1154` — `ItemUse_MedPack_Big`. Heals via [`MedPackGive`] for
/// `MAX_MEDPACK_BIG_HEAL_AMOUNT` (50).
pub unsafe fn ItemUse_MedPack_Big(ent: *mut gentity_t) {
    MedPackGive(ent, MAX_MEDPACK_BIG_HEAL_AMOUNT);
}

/// Port of `g_items.c:1159` — `ItemUse_MedPack`. Heals via [`MedPackGive`] for
/// `MAX_MEDPACK_HEAL_AMOUNT` (25).
pub unsafe fn ItemUse_MedPack(ent: *mut gentity_t) {
    MedPackGive(ent, MAX_MEDPACK_HEAL_AMOUNT);
}

/// Port of `g_items.c:1165` — `Jetpack_Off`. Turns the client's jetpack off (clears
/// `jetPackOn`); no-ops if already off. The C `assert(ent && ent->client)` is omitted
/// (Rust debug-asserts add no behavior at the ABI boundary; callers pass valid clients).
///
/// No oracle: `gentity_t`→`client` flag mutation.
pub unsafe fn Jetpack_Off(ent: *mut gentity_t) {
    //create effects?
    if (*(*ent).client).jetPackOn == QFALSE {
        //aready off
        return;
    }

    (*(*ent).client).jetPackOn = QFALSE;
}

/// Port of `g_items.c:1177` — `Jetpack_On`. Turns the client's jetpack on (sets
/// `jetPackOn`) and plays the `boba/JETON` sound; no-ops if already on, if the client is
/// being force-gripped (`forceGripBeingGripped >= level.time`), or if falling to death.
/// The C `assert(ent && ent->client)` is omitted (see [`Jetpack_Off`]).
///
/// No oracle: `gentity_t`→`client` mutation plus [`G_Sound`]/[`G_SoundIndex`].
pub unsafe fn Jetpack_On(ent: *mut gentity_t) {
    //create effects?
    if (*(*ent).client).jetPackOn == QTRUE {
        //aready on
        return;
    }

    if (*(*ent).client).ps.fd.forceGripBeingGripped >= (*addr_of!(level)).time as f32 {
        //can't turn on during grip interval
        return;
    }

    if (*(*ent).client).ps.fallingToDeath != 0 {
        //too late!
        return;
    }

    G_Sound(ent, CHAN_AUTO, G_SoundIndex("sound/boba/JETON"));

    (*(*ent).client).jetPackOn = QTRUE;
}

/// Port of `g_items.c:1201` — `ItemUse_Jetpack`. Toggles the jetpack on a debounce
/// (`jetPackToggleTime`): no-ops while the toggle timer is still pending; refuses while
/// dead (`health <= 0` / `STAT_HEALTH <= 0` / `EF_DEAD` / `PM_DEAD`); refuses to start
/// when fuel `< 5`; otherwise calls [`Jetpack_Off`] or [`Jetpack_On`] and re-arms the
/// toggle timer (`level.time + JETPACK_TOGGLE_TIME`). The C `assert` is omitted (see
/// [`Jetpack_Off`]).
///
/// No oracle: `gentity_t`→`client`→`ps` mutation reading `level.time`.
pub unsafe fn ItemUse_Jetpack(ent: *mut gentity_t) {
    const JETPACK_TOGGLE_TIME: c_int = 1000;

    if (*(*ent).client).jetPackToggleTime >= (*addr_of!(level)).time {
        return;
    }

    if (*ent).health <= 0
        || (*(*ent).client).ps.stats[STAT_HEALTH as usize] <= 0
        || ((*(*ent).client).ps.eFlags & EF_DEAD) != 0
        || (*(*ent).client).ps.pm_type == PM_DEAD
    {
        //can't use it when dead under any circumstances.
        return;
    }

    if (*(*ent).client).jetPackOn == QFALSE && (*(*ent).client).ps.jetpackFuel < 5 {
        //too low on fuel to start it up
        return;
    }

    if (*(*ent).client).jetPackOn == QTRUE {
        Jetpack_Off(ent);
    } else {
        Jetpack_On(ent);
    }

    (*(*ent).client).jetPackToggleTime = (*addr_of!(level)).time + JETPACK_TOGGLE_TIME;
}

/// Port of `g_items.c:1239` — `ItemUse_UseCloak`. Toggles personal cloak on a debounce
/// (`cloakToggleTime`): no-ops while the toggle timer is still pending; refuses while dead
/// (`health <= 0` / `STAT_HEALTH <= 0` / `EF_DEAD` / `PM_DEAD`); refuses to start when
/// `cloakFuel < 5`; otherwise calls [`Jedi_Decloak`] (if `PW_CLOAKED` is set) or
/// [`Jedi_Cloak`] and re-arms the toggle timer (`level.time + CLOAK_TOGGLE_TIME`). The C
/// `assert(ent && ent->client)` is omitted (the [`ItemUse_Jetpack`] precedent).
///
/// No oracle: `gentity_t`→`client`→`ps` mutation reading `level.time` and calling
/// trap-backed cloak fns.
pub unsafe fn ItemUse_UseCloak(ent: *mut gentity_t) {
    const CLOAK_TOGGLE_TIME: c_int = 1000;

    if (*(*ent).client).cloakToggleTime >= (*addr_of!(level)).time {
        return;
    }

    if (*ent).health <= 0
        || (*(*ent).client).ps.stats[STAT_HEALTH as usize] <= 0
        || ((*(*ent).client).ps.eFlags & EF_DEAD) != 0
        || (*(*ent).client).ps.pm_type == PM_DEAD
    {
        //can't use it when dead under any circumstances.
        return;
    }

    if (*(*ent).client).ps.powerups[PW_CLOAKED as usize] == 0
        && (*(*ent).client).ps.cloakFuel < 5
    {
        //too low on fuel to start it up
        return;
    }

    if (*(*ent).client).ps.powerups[PW_CLOAKED as usize] != 0 {
        //decloak
        Jedi_Decloak(ent);
    } else {
        //cloak
        Jedi_Cloak(ent);
    }

    (*(*ent).client).cloakToggleTime = (*addr_of!(level)).time + CLOAK_TOGGLE_TIME;
}

/// `void SentryTouch( gentity_t *ent, gentity_t *other, trace_t *trace )`
/// (g_items.c:515) — the portable-sentry-gun `touch` callback. The C body is a bare
/// `return;` (a deliberate no-op placeholder — the sentry has no touch behaviour),
/// so this is the empty stub. `extern "C"` so it slots into the `gentity_t::touch`
/// fn-pointer ABI (the `Use_Item` callback precedent). Clean leaf: no callees.
/// No oracle (empty body).
pub unsafe extern "C" fn SentryTouch(
    _ent: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
}

/// `void turret_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod )`
/// (g_items.c:940) — the portable-sentry-gun base `die` callback. Clears its
/// think/use/die fn-pointer slots, fires its targets, then (if the deploying client
/// is gone) frees and bails; otherwise it explodes (effect + radius damage), clears
/// the owner's `sentryDeployed` flag, and frees. `extern "C"` so it slots into the
/// `gentity_t::die` fn-pointer ABI. Clean leaf: every callee
/// (`G_UseTargets`/`G_FreeEntity`/`VectorSet`/`G_PlayEffect`/`G_RadiusDamage`) is
/// ported and all three fn-pointer slots are nulled. No oracle (entity-graph mutation).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn turret_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    // Turn off the thinking of the base & use it's targets
    (*self_).think = None;
    (*self_).r#use = None;

    if !(*self_).target.is_null() {
        G_UseTargets(self_, attacker);
    }

    let owner = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*self_).genericValue3 as usize);
    if (*owner).inuse == QFALSE || (*owner).client.is_null() {
        G_FreeEntity(self_);
        return;
    }

    // clear my data
    (*self_).die = None;
    (*self_).takedamage = QFALSE;
    (*self_).health = 0;

    // hack the effect angle so that explode death can orient the effect properly
    VectorSet(&mut (*self_).s.angles, 0.0, 0.0, 1.0);

    G_PlayEffect(
        EFFECT_EXPLOSION_PAS,
        &(*self_).s.pos.trBase,
        &(*self_).s.angles,
    );
    G_RadiusDamage(
        &(*self_).s.pos.trBase,
        owner,
        30.0,
        256.0,
        self_,
        self_,
        MOD_UNKNOWN,
    );

    (*(*owner).client).ps.fd.sentryDeployed = QFALSE;

    //ExplodeDeath( self );
    G_FreeEntity(self_);
}

/// `void sentryExpire( gentity_t *self )` (g_items.c:702) — the portable-sentry-gun
/// lifetime-expiry `think` callback: simply forwards to [`turret_die`] with the entity
/// as its own inflictor/attacker, 1000 damage, `MOD_UNKNOWN`. `extern "C"` for the
/// `gentity_t::think` fn-pointer ABI. Clean leaf now that `turret_die` has landed.
/// No oracle (entity-graph mutation via the callee).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn sentryExpire(self_: *mut gentity_t) {
    turret_die(self_, self_, self_, 1000, MOD_UNKNOWN);
}

/// `void SpecialItemThink( gentity_t *ent )` (g_items.c:1277) — the per-frame `think`
/// callback for a tossed special item: once its `genericValue5` lifetime expires it
/// re-points its think slot at `G_FreeEntity` and schedules immediate cleanup; until
/// then it runs the extended-physics step and snaps `s.origin` to the traced origin.
/// `extern "C"` for the `gentity_t::think` fn-pointer ABI. Clean leaf: callees
/// (`G_RunExPhys`/`VectorCopy`) ported, the slot points at the ported `G_FreeEntity`.
/// No oracle (entity-graph mutation + physics).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn SpecialItemThink(ent: *mut gentity_t) {
    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    if (*ent).genericValue5 < (*addr_of!(level)).time {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    G_RunExPhys(ent, gravity, mass, bounce, QFALSE as i32, core::ptr::null_mut(), 0);
    VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.origin);
    (*ent).nextthink = (*addr_of!(level)).time + 50;
}

/// `void G_SpecialSpawnItem( gentity_t *ent, gitem_t *item )` (g_items.c:1295) — sets up a
/// tossed "special" item entity (a dispenser-spawned medpack/ammo): registers the item,
/// arms its [`SpecialItemThink`] lifetime/physics think, sets the bouncy item bounds + trigger
/// contents + [`Touch_Item`], records the owner / owner-no-touch window and the pickup-removes
/// (not respawns) flag, and marks it for server-only smoothed physics. Clean cluster leaf:
/// all callees ([`RegisterItem`], [`VectorSet`], [`SpecialItemThink`], [`Touch_Item`]) ported.
/// No oracle (entity-state spawn: think/touch fn-pointers + `s.eType`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `item` must point into the live [`bg_itemlist`]
/// table so the `item - bg_itemlist` modelindex offset is valid.
pub unsafe fn G_SpecialSpawnItem(ent: *mut gentity_t, item: *mut gitem_t) {
    RegisterItem(item);
    (*ent).item = item;

    //go away if no one wants me
    (*ent).genericValue5 = (*addr_of!(level)).time + TOSSED_ITEM_STAY_PERIOD;
    (*ent).think = Some(SpecialItemThink);
    (*ent).nextthink = (*addr_of!(level)).time + 50;
    (*ent).clipmask = MASK_SOLID;

    (*ent).physicsBounce = 0.50; // items are bouncy
    VectorSet(&mut (*ent).r.mins, -8.0, -8.0, -0.0);
    VectorSet(&mut (*ent).r.maxs, 8.0, 8.0, 16.0);

    (*ent).s.eType = ET_ITEM;
    (*ent).s.modelindex =
        item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as c_int; // store item number in modelindex

    (*ent).r.contents = CONTENTS_TRIGGER;
    (*ent).touch = Some(Touch_Item);

    //can't touch owner for x seconds
    (*ent).genericValue11 = (*ent).r.ownerNum;
    (*ent).genericValue10 = (*addr_of!(level)).time + TOSSED_ITEM_OWNER_NOTOUCH_DUR;

    //so we know to remove when picked up, not respawn
    (*ent).genericValue9 = 1;

    //kind of a lame value to use, but oh well. This means don't
    //pick up this item clientside with prediction, because we
    //aren't sending over all the data necessary for the player
    //to know if he can.
    (*ent).s.brokenLimbs = 1;

    //since it uses my server-only physics
    (*ent).s.eFlags |= EF_CLIENTSMOOTH;
}

/// `void FinishSpawningItem( gentity_t *ent )` (g_items.c:2792) — the deferred item-spawn
/// think (installed by [`G_SpawnItem`]): culls items that don't belong in the active gametype
/// (siege/JM powerups, saber-only ammo/holdables, holocron-mode light/dark, force-disabled
/// powerups, duel armor/health/medpacks, non-CTF flags), then sets up the survivors —
/// item bounds, `ET_ITEM`, the `item - bg_itemlist` modelindex, trigger contents, and the
/// [`Touch_Item`]/[`Use_Item`] callbacks. Suspended items keep their origin; the rest
/// drop-to-floor via a downward [`trap::Trace`] (bailing on startsolid). Team-slave/targeted
/// items spawn hidden+non-solid; everything else links into the world.
/// No oracle (entity-state spawn: traces, link, fn-pointer callbacks).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-NULL `item` pointing into the live
/// [`bg_itemlist`] table.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn FinishSpawningItem(ent: *mut gentity_t) {
    let tr: trace_t;
    let mut dest: vec3_t = [0.0; 3];
    //	gitem_t		*item;

    //	VectorSet( ent->r.mins, -ITEM_RADIUS, -ITEM_RADIUS, -ITEM_RADIUS );
    //	VectorSet( ent->r.maxs, ITEM_RADIUS, ITEM_RADIUS, ITEM_RADIUS );

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //in siege remove all powerups
        if (*(*ent).item).giType == IT_POWERUP {
            G_FreeEntity(ent);
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer != GT_JEDIMASTER {
        if HasSetSaberOnly() != 0 {
            if (*(*ent).item).giType == IT_AMMO {
                G_FreeEntity(ent);
                return;
            }

            if (*(*ent).item).giType == IT_HOLDABLE
                && ((*(*ent).item).giTag == HI_SEEKER
                    || (*(*ent).item).giTag == HI_SHIELD
                    || (*(*ent).item).giTag == HI_SENTRY_GUN)
            {
                G_FreeEntity(ent);
                return;
            }
        }
    } else {
        //no powerups in jedi master
        if (*(*ent).item).giType == IT_POWERUP {
            G_FreeEntity(ent);
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_HOLOCRON {
        if (*(*ent).item).giType == IT_POWERUP
            && ((*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_LIGHT
                || (*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_DARK)
        {
            G_FreeEntity(ent);
            return;
        }
    }

    if (*addr_of!(g_forcePowerDisable)).integer != 0 {
        //if force powers disabled, don't add force powerups
        if (*(*ent).item).giType == IT_POWERUP
            && ((*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_LIGHT
                || (*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_DARK
                || (*(*ent).item).giTag == PW_FORCE_BOON)
        {
            G_FreeEntity(ent);
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        if (*(*ent).item).giType == IT_ARMOR
            || (*(*ent).item).giType == IT_HEALTH
            || ((*(*ent).item).giType == IT_HOLDABLE
                && ((*(*ent).item).giTag == HI_MEDPAC || (*(*ent).item).giTag == HI_MEDPAC_BIG))
        {
            G_FreeEntity(ent);
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer != GT_CTF
        && (*addr_of!(g_gametype)).integer != GT_CTY
        && (*(*ent).item).giType == IT_TEAM
    {
        let mut killMe = 0;

        match (*(*ent).item).giTag {
            x if x == PW_REDFLAG => {
                killMe = 1;
            }
            x if x == PW_BLUEFLAG => {
                killMe = 1;
            }
            x if x == PW_NEUTRALFLAG => {
                killMe = 1;
            }
            _ => {}
        }

        if killMe != 0 {
            G_FreeEntity(ent);
            return;
        }
    }

    VectorSet(&mut (*ent).r.mins, -8.0, -8.0, -0.0);
    VectorSet(&mut (*ent).r.maxs, 8.0, 8.0, 16.0);

    (*ent).s.eType = ET_ITEM;
    (*ent).s.modelindex =
        (*ent).item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as c_int; // store item number in modelindex
    (*ent).s.modelindex2 = 0; // zero indicates this isn't a dropped item

    (*ent).r.contents = CONTENTS_TRIGGER;
    (*ent).touch = Some(Touch_Item);
    // useing an item causes it to respawn
    (*ent).r#use = Some(Use_Item);

    // create a Ghoul2 model if the world model is a glm
    /*	item = &bg_itemlist[ ent->s.modelindex ];
        if (!stricmp(&item->world_model[0][strlen(item->world_model[0]) - 4], ".glm"))
        {
            trap_G2API_InitGhoul2Model(&ent->s, item->world_model[0], G_ModelIndex(item->world_model[0] ), 0, 0, 0, 0);
            ent->s.radius = 60;
        }
    */
    if (*ent).spawnflags & ITMSF_SUSPEND != 0 {
        // suspended
        G_SetOrigin(ent, &(*ent).s.origin);
    } else {
        // drop to floor

        //if it is directly even with the floor it will return startsolid, so raise up by 0.1
        //and temporarily subtract 0.1 from the z maxs so that going up doesn't push into the ceiling
        (*ent).s.origin[2] += 0.1;
        (*ent).r.maxs[2] -= 0.1;

        VectorSet(
            &mut dest,
            (*ent).s.origin[0],
            (*ent).s.origin[1],
            (*ent).s.origin[2] - 4096.0,
        );
        tr = trap::Trace(
            &(*ent).s.origin,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &dest,
            (*ent).s.number,
            MASK_SOLID,
        );
        if tr.startsolid != 0 {
            G_Printf(&format!(
                "FinishSpawningItem: {} startsolid at {}\n",
                Sz((*ent).classname),
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
    }

    // team slaves and targeted items aren't present at start
    if ((*ent).flags & FL_TEAMSLAVE) != 0 || !(*ent).targetname.is_null() {
        (*ent).s.eFlags |= EF_NODRAW;
        (*ent).r.contents = 0;
        return;
    }

    // powerups don't spawn in for a while
    /*
    if ( ent->item->giType == IT_POWERUP ) {
        float	respawn;

        respawn = 45 + crandom() * 15;
        ent->s.eFlags |= EF_NODRAW;
        ent->r.contents = 0;
        ent->nextthink = level.time + respawn * 1000;
        ent->think = RespawnItem;
        return;
    }
    */

    trap::LinkEntity(ent);
}

/// `void G_SpawnItem( gentity_t *ent, gitem_t *item )` (g_items.c:3092) — map-spawn entry for
/// an item entity: reads its `random`/`wait` spawn keys, then in duel modes drops weapons
/// disabled by [`g_duelWeaponDisable`] (else [`g_weaponDisable`]) — except in Jedi Master.
/// Surviving items are registered and (unless [`G_ItemDisabled`]) wired to defer real spawn
/// to [`FinishSpawningItem`] on the third frame (so they can ride trains), with the bouncy
/// item physics; powerups also precache the respawn sound and read `noglobalsound`.
/// No oracle (entity-state spawn: think fn-pointer + spawn-key reads).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `item` must point into the live [`bg_itemlist`]
/// table.
pub unsafe fn G_SpawnItem(ent: *mut gentity_t, item: *mut gitem_t) {
    let wDisable: c_int;

    G_SpawnFloat(c"random".as_ptr(), c"0".as_ptr(), &mut (*ent).random);
    G_SpawnFloat(c"wait".as_ptr(), c"0".as_ptr(), &mut (*ent).wait);

    if (*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        wDisable = (*addr_of!(g_duelWeaponDisable)).integer;
    } else {
        wDisable = (*addr_of!(g_weaponDisable)).integer;
    }

    if (*item).giType == IT_WEAPON && wDisable != 0 && (wDisable & (1 << (*item).giTag)) != 0 {
        if (*addr_of!(g_gametype)).integer != GT_JEDIMASTER {
            G_FreeEntity(ent);
            return;
        }
    }

    RegisterItem(item);
    if G_ItemDisabled(item) != 0 {
        return;
    }

    (*ent).item = item;
    // some movers spawn on the second frame, so delay item
    // spawns until the third frame so they can ride trains
    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME * 2;
    (*ent).think = Some(FinishSpawningItem);

    (*ent).physicsBounce = 0.50; // items are bouncy

    if (*item).giType == IT_POWERUP {
        G_SoundIndex("sound/items/respawn1");
        G_SpawnFloat(c"noglobalsound".as_ptr(), c"0".as_ptr(), &mut (*ent).speed);
    }
}

/// `void ClearRegisteredItems( void )` (g_items.c:3011) — resets the [`itemRegistered`]
/// precache table to all-off, then re-registers the four weapons every player always starts
/// with (Bryar pistol, stun baton, melee, saber), and in Siege also precaches the dispenser
/// items via [`G_PrecacheDispensers`]. Clean leaf now that `G_PrecacheDispensers` is ported.
///
/// # Safety
/// Writes the `itemRegistered` global and registers items through ported callees.
pub unsafe fn ClearRegisteredItems() {
    // C: memset( itemRegistered, 0, sizeof( itemRegistered ) );
    for slot in (*addr_of_mut!(itemRegistered)).iter_mut() {
        *slot = QFALSE;
    }

    // players always start with the base weapon
    RegisterItem(BG_FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(BG_FindItemForWeapon(WP_STUN_BATON));
    RegisterItem(BG_FindItemForWeapon(WP_MELEE));
    RegisterItem(BG_FindItemForWeapon(WP_SABER));

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        // kind of cheesy, maybe check if siege class with disp's is gonna be on this map too
        G_PrecacheDispensers();
    }
}

/// `void G_PrecacheDispensers(void)` (g_items.c:1336) — registers the two item
/// dispensers' dispensed items (the instant-medpack and the all-ammo pickup) so the engine
/// precaches their assets. Each `BG_FindItem` lookup is NULL-guarded before
/// [`RegisterItem`], faithful to the C. Clean leaf.
///
/// # Safety
/// Reads/writes the `bg_itemlist` + `itemRegistered` globals through ported callees.
pub unsafe fn G_PrecacheDispensers() {
    // g_items.c:1333-1334 — DISP_HEALTH_ITEM / DISP_AMMO_ITEM (local #defines).
    let item = BG_FindItem(c"item_medpak_instant".as_ptr());
    if !item.is_null() {
        RegisterItem(item);
    }

    let item = BG_FindItem(c"ammo_all".as_ptr());
    if !item.is_null() {
        RegisterItem(item);
    }
}

/// `void EWebPrecache(void)` (g_items.c:1434) — precaches the E-Web turret's assets: its
/// weapon item ([`BG_FindItemForWeapon`]`(WP_TURRET)` → [`RegisterItem`]) plus its two
/// muzzle/explosion effects via [`G_EffectIndex`]. Clean leaf.
///
/// # Safety
/// Registers items and effect indices through ported callees that touch globals.
pub unsafe fn EWebPrecache() {
    RegisterItem(BG_FindItemForWeapon(WP_TURRET));
    G_EffectIndex("detpack/explosion.efx");
    G_EffectIndex("turret/muzzle_flash.efx");
}

/// `void EWebDisattach(gentity_t *owner, gentity_t *eweb)` (g_items.c:1417) — puts the
/// portable E-Web away: clears the owner's `ewebIndex`/`emplacedIndex`, restores the saved
/// weapon bitfield (`eweb->genericValue11`) into `STAT_WEAPONS` if the owner is still alive
/// (else zeroes it), then schedules the eweb entity for immediate cleanup
/// ([`G_FreeEntity`] this frame). Clean leaf — pure field writes + a ported think slot.
///
/// # Safety
/// `owner` must point to a valid `gentity_t` with a non-null `client`; `eweb` must point to
/// a valid `gentity_t`.
pub unsafe fn EWebDisattach(owner: *mut gentity_t, eweb: *mut gentity_t) {
    (*(*owner).client).ewebIndex = 0;
    (*(*owner).client).ps.emplacedIndex = 0;
    if (*owner).health > 0 {
        (*(*owner).client).ps.stats[STAT_WEAPONS as usize] = (*eweb).genericValue11;
    } else {
        (*(*owner).client).ps.stats[STAT_WEAPONS as usize] = 0;
    }
    (*eweb).think = Some(G_FreeEntity);
    (*eweb).nextthink = (*addr_of!(level)).time;
}

// g_items.c:1442-1443 — e-web death blast.
const EWEB_DEATH_RADIUS: c_int = 128;
const EWEB_DEATH_DMG: c_int = 90;

/// `void EWebDie(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod)`
/// (g_items.c:1449) — the E-Web turret's `die` callback. Detonates a radius blast at the
/// turret's origin ([`G_RadiusDamage`], `MOD_SUICIDE`), plays the detpack explosion effect
/// ([`G_PlayEffect`]), and — if the turret still has a live client owner — puts the weapon
/// away ([`EWebDisattach`]), resets `ewebHealth`, strips `HI_EWEB` from the owner's holdable
/// bitfield, and (if the owner had the E-Web holdable selected) deselects it and cycles to the
/// first available item ([`BG_CycleInven`]). Clean cluster leaf — all callees ported; entity-state
/// mutation + radius damage, so no oracle. `extern "C"` for the `gentity_t::die` fn-ptr slot.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other args are unused. When the owner slot
/// is live it must index a valid `gentity_t` with a non-null `client`.
pub unsafe extern "C" fn EWebDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    let mut fx_dir: vec3_t = [0.0; 3];

    G_RadiusDamage(
        &(*self_).r.currentOrigin,
        self_,
        EWEB_DEATH_DMG as f32,
        EWEB_DEATH_RADIUS as f32,
        self_,
        self_,
        MOD_SUICIDE,
    );

    VectorSet(&mut fx_dir, 1.0, 0.0, 0.0);
    G_PlayEffect(EFFECT_EXPLOSION_DETPACK, &(*self_).r.currentOrigin, &fx_dir);

    if (*self_).r.ownerNum != ENTITYNUM_NONE {
        let owner = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*self_).r.ownerNum as usize);

        if (*owner).inuse == QTRUE && !(*owner).client.is_null() {
            EWebDisattach(owner, self_);

            //make sure it resets next time we spawn one in case we someone obtain one before death
            (*(*owner).client).ewebHealth = -1;

            //take it away from him, it is gone forever.
            (*(*owner).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_EWEB);

            let list = addr_of!(bg_itemlist) as *const gitem_t;
            let sel = (*(*owner).client).ps.stats[STAT_HOLDABLE_ITEM as usize];
            if sel > 0
                && (*list.add(sel as usize)).giType == IT_HOLDABLE
                && (*list.add(sel as usize)).giTag == HI_EWEB
            {
                //he has it selected so deselect it and select the first thing available
                (*(*owner).client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
                BG_CycleInven(&mut (*(*owner).client).ps, 1);
            }
        }
    }
}

/// `void ShieldRemove(gentity_t *self)` (g_items.c:108) — tears down a personal-shield
/// entity: re-points its think slot at [`G_FreeEntity`] (scheduled `+100ms`), plays the
/// deactivate sound ([`shieldDeactivateSound`]) via [`G_AddEvent`], and clears the looping
/// force-field hum. Clean leaf — a ported think slot, a ported event helper, and field
/// writes; the sound handle is the shared shield static (0 until `PlaceShield` registers it).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe fn ShieldRemove(self_: *mut gentity_t) {
    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time + 100;

    // Play kill sound...
    G_AddEvent(self_, EV_GENERAL_SOUND, *addr_of!(shieldDeactivateSound));
    (*self_).s.loopSound = 0;
    (*self_).s.loopIsSoundset = QFALSE;
}

/// `void ShieldThink(gentity_t *self)` (g_items.c:123) — the personal-shield's per-second
/// health countdown. Clears the `trickedentindex` pain flag, drains `health` by the
/// gametype-appropriate decrement ([`SHIELD_SIEGE_HEALTH_DEC`] in `GT_SIEGE`, else
/// [`SHIELD_HEALTH_DEC`]), reschedules itself for `+1000ms`, and tears the shield down via
/// [`ShieldRemove`] once health hits zero. Clean leaf — field writes, a ported gametype
/// global, and a ported teardown helper.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn ShieldThink(self_: *mut gentity_t) {
    (*self_).s.trickedentindex = 0;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        (*self_).health -= SHIELD_SIEGE_HEALTH_DEC;
    } else {
        (*self_).health -= SHIELD_HEALTH_DEC;
    }
    (*self_).nextthink = (*addr_of!(level)).time + 1000;
    if (*self_).health <= 0 {
        ShieldRemove(self_);
    }
}

/// `void ShieldDie(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod)`
/// (g_items.c:145) — the personal-shield's `die` callback: the shield took damage below zero
/// health. Plays the damage sound ([`shieldDamageSound`]) via [`G_AddEvent`] and tears the
/// shield down through [`ShieldRemove`]. Clean leaf — a ported event helper plus a ported
/// teardown helper. `extern "C"` for the `gentity_t::die` fn-ptr slot.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other args are unused.
pub unsafe extern "C" fn ShieldDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    // Play damaging sound...
    G_AddEvent(self_, EV_GENERAL_SOUND, *addr_of!(shieldDamageSound));

    ShieldRemove(self_);
}

/// `void ShieldPain(gentity_t *self, gentity_t *attacker, int damage)` (g_items.c:155) — the
/// personal-shield's `pain` callback: the shield was hit. Re-points its think slot at
/// [`ShieldThink`] (scheduled `+400ms`), plays the damage sound ([`shieldDamageSound`]) via
/// [`G_AddEvent`], and raises the `trickedentindex` flag so the client draws the shield-pain
/// flicker. Clean leaf — a ported think slot, a ported event helper, and field writes.
/// `extern "C"` for the `gentity_t::pain` fn-ptr slot.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; the other args are unused.
pub unsafe extern "C" fn ShieldPain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
) {
    // Set the itemplaceholder flag to indicate the the shield drawing that the shield pain should be drawn.
    (*self_).think = Some(ShieldThink);
    (*self_).nextthink = (*addr_of!(level)).time + 400;

    // Play damaging sound...
    G_AddEvent(self_, EV_GENERAL_SOUND, *addr_of!(shieldDamageSound));

    (*self_).s.trickedentindex = 1;
}

/// `void ShieldGoSolid(gentity_t *self)` (g_items.c:171) — the shield's `think` callback that
/// tries to re-activate after a friendly pass-through. Decrements `health` (tearing down via
/// [`ShieldRemove`] if it hits zero), then [`trap::Trace`]s its own bounds against
/// `CONTENTS_BODY`: if something is in the way it reschedules itself `+200ms`, otherwise it
/// goes solid — clears `EF_NODRAW`, restores `CONTENTS_SOLID`/`takedamage`, schedules
/// [`ShieldThink`] `+1000ms`, plays the activate sound ([`shieldActivateSound`]), and restores
/// the looping hum ([`shieldLoopSound`]). Re-links each branch via [`trap::LinkEntity`].
/// `extern "C"` for the `gentity_t::think` fn-ptr slot.
///
/// No oracle: drives the world via [`trap::Trace`]/[`trap::LinkEntity`] and mutates the
/// `gentity_t` graph — the shield-cluster / item-world precedent.
///
/// # Safety
/// `self_` must point to a valid, linked `gentity_t`.
pub unsafe extern "C" fn ShieldGoSolid(self_: *mut gentity_t) {
    // see if we're valid
    (*self_).health -= 1;
    if (*self_).health <= 0 {
        ShieldRemove(self_);
        return;
    }

    let tr: trace_t = trap::Trace(
        &(*self_).r.currentOrigin,
        &(*self_).r.mins,
        &(*self_).r.maxs,
        &(*self_).r.currentOrigin,
        (*self_).s.number,
        CONTENTS_BODY,
    );
    if tr.startsolid != 0 {
        // gah, we can't activate yet
        (*self_).nextthink = (*addr_of!(level)).time + 200;
        (*self_).think = Some(ShieldGoSolid);
        trap::LinkEntity(self_);
    } else {
        // get hard... huh-huh...
        (*self_).s.eFlags &= !EF_NODRAW;

        (*self_).r.contents = CONTENTS_SOLID;
        (*self_).nextthink = (*addr_of!(level)).time + 1000;
        (*self_).think = Some(ShieldThink);
        (*self_).takedamage = QTRUE;
        trap::LinkEntity(self_);

        // Play raising sound...
        G_AddEvent(self_, EV_GENERAL_SOUND, *addr_of!(shieldActivateSound));
        (*self_).s.loopSound = *addr_of!(shieldLoopSound);
        (*self_).s.loopIsSoundset = QFALSE;
    }
}

/// `void ShieldGoNotSolid(gentity_t *self)` (g_items.c:211) — briefly drops the shield to
/// non-solid so a friend can pass through: clears `r.contents`, sets `EF_NODRAW`, schedules
/// [`ShieldGoSolid`] `+200ms` (a deliberately coarse interval to avoid Activate-message
/// pileup), disables `takedamage`, relinks via [`trap::LinkEntity`], plays the deactivate
/// sound ([`shieldDeactivateSound`]) and silences the hum. Clean leaf — a ported think slot,
/// a ported event helper, and field writes.
///
/// No oracle: relinks via [`trap::LinkEntity`] and mutates the `gentity_t` graph — the
/// shield-cluster precedent.
///
/// # Safety
/// `self_` must point to a valid, linked `gentity_t`.
pub unsafe extern "C" fn ShieldGoNotSolid(self_: *mut gentity_t) {
    // make the shield non-solid very briefly
    (*self_).r.contents = 0;
    (*self_).s.eFlags |= EF_NODRAW;
    // nextthink needs to have a large enough interval to avoid excess accumulation of Activate messages
    (*self_).nextthink = (*addr_of!(level)).time + 200;
    (*self_).think = Some(ShieldGoSolid);
    (*self_).takedamage = QFALSE;
    trap::LinkEntity(self_);

    // Play kill sound...
    G_AddEvent(self_, EV_GENERAL_SOUND, *addr_of!(shieldDeactivateSound));
    (*self_).s.loopSound = 0;
    (*self_).s.loopIsSoundset = QFALSE;
}

/// `void ShieldTouch(gentity_t *self, gentity_t *other, trace_t *trace)` (g_items.c:230) — the
/// shield's `touch` callback: a player touched the shield. In a team game it lets teammates
/// through (parent vs. `other` via the ported [`OnSameTeam`]); otherwise it only lets the
/// dropper (matching `s.number`) through — either way by dropping to non-solid via
/// [`ShieldGoNotSolid`]. Clean leaf — a ported team predicate and a ported sibling. `extern "C"`
/// for the `gentity_t::touch` fn-ptr slot.
///
/// No oracle: dereferences the `gentity_t`→`client` graph and the gametype global — the
/// shield-cluster / touch-callback precedent.
///
/// # Safety
/// `self_`/`other` must point to valid `gentity_t`s; `_trace` to a valid `trace_t`.
pub unsafe extern "C" fn ShieldTouch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        // let teammates through
        // compare the parent's team to the "other's" team
        if !(*self_).parent.is_null()
            && !(*(*self_).parent).client.is_null()
            && !(*other).client.is_null()
        {
            if OnSameTeam((*self_).parent, other) != QFALSE {
                ShieldGoNotSolid(self_);
            }
        }
    } else {
        // let the person who dropped the shield through
        if !(*self_).parent.is_null() && (*(*self_).parent).s.number == (*other).s.number {
            ShieldGoNotSolid(self_);
        }
    }
}

/// `void CreateShield(gentity_t *ent)` (g_items.c:254) — the deferred `think` callback
/// [`PlaceShield`] arms 500ms after spawn: expands the dropped shield in all directions.
/// Traces upward for the height (capped at `MAX_SHIELD_HEIGHT`), picks the alignment axis
/// from `s.angles[YAW]`, traces sideways both ways for the half-widths, recenters the
/// origin, sets `r.mins`/`r.maxs`, packs `xaxis|height|posWidth|negWidth` into `s.time2`
/// for client-side rendering, sets `health` (siege vs. normal), wires the
/// pain/die/touch slots, then probes its own bounds against `CONTENTS_BODY` — if blocked it
/// goes `EF_NODRAW`/non-solid and reschedules [`ShieldGoSolid`] `+200ms`, else it goes
/// solid (`CONTENTS_PLAYERCLIP|CONTENTS_SHOTCLIP`), schedules [`ShieldThink`], enables
/// `takedamage`, and plays the raise sound + hum. Finally calls [`ShieldGoSolid`].
///
/// No oracle: world traces ([`trap::Trace`]), relinks ([`trap::LinkEntity`]), and mutates
/// the `gentity_t` graph — the shield-cluster precedent. (C's unused `static int shieldID`
/// is omitted — it is declared but never read.)
///
/// # Safety
/// `ent` must point to a valid, spawned `gentity_t`.
pub unsafe extern "C" fn CreateShield(ent: *mut gentity_t) {
    let mut tr: trace_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut pos_trace_end: vec3_t = [0.0; 3];
    let mut neg_trace_end: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let height: c_int;
    let pos_width: c_int;
    let neg_width: c_int;
    let half_width: c_int;
    let xaxis: qboolean;
    let param_data: c_int;

    // trace upward to find height of shield
    VectorCopy(&(*ent).r.currentOrigin, &mut end);
    end[2] += MAX_SHIELD_HEIGHT as f32;
    tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*ent).s.number,
        MASK_SHOT,
    );
    height = (MAX_SHIELD_HEIGHT as f32 * tr.fraction) as c_int;

    // use angles to find the proper axis along which to align the shield
    VectorSet(
        &mut mins,
        -(SHIELD_HALFTHICKNESS as f32),
        -(SHIELD_HALFTHICKNESS as f32),
        0.0,
    );
    VectorSet(
        &mut maxs,
        SHIELD_HALFTHICKNESS as f32,
        SHIELD_HALFTHICKNESS as f32,
        height as f32,
    );
    VectorCopy(&(*ent).r.currentOrigin, &mut pos_trace_end);
    VectorCopy(&(*ent).r.currentOrigin, &mut neg_trace_end);

    if (*ent).s.angles[YAW] as c_int == 0 {
        // shield runs along y-axis
        pos_trace_end[1] += MAX_SHIELD_HALFWIDTH as f32;
        neg_trace_end[1] -= MAX_SHIELD_HALFWIDTH as f32;
        xaxis = QFALSE;
    } else {
        // shield runs along x-axis
        pos_trace_end[0] += MAX_SHIELD_HALFWIDTH as f32;
        neg_trace_end[0] -= MAX_SHIELD_HALFWIDTH as f32;
        xaxis = QTRUE;
    }

    // trace horizontally to find extend of shield
    // positive trace
    VectorCopy(&(*ent).r.currentOrigin, &mut start);
    start[2] += (height >> 1) as f32;
    tr = trap::Trace(
        &start,
        &vec3_origin,
        &vec3_origin,
        &pos_trace_end,
        (*ent).s.number,
        MASK_SHOT,
    );
    pos_width = (MAX_SHIELD_HALFWIDTH as f32 * tr.fraction) as c_int;
    // negative trace
    tr = trap::Trace(
        &start,
        &vec3_origin,
        &vec3_origin,
        &neg_trace_end,
        (*ent).s.number,
        MASK_SHOT,
    );
    neg_width = (MAX_SHIELD_HALFWIDTH as f32 * tr.fraction) as c_int;

    // kef -- monkey with dimensions and place origin in center
    half_width = (pos_width + neg_width) >> 1;
    if xaxis != QFALSE {
        (*ent).r.currentOrigin[0] = (*ent).r.currentOrigin[0] - neg_width as f32 + half_width as f32;
    } else {
        (*ent).r.currentOrigin[1] = (*ent).r.currentOrigin[1] - neg_width as f32 + half_width as f32;
    }
    (*ent).r.currentOrigin[2] += (height >> 1) as f32;

    // set entity's mins and maxs to new values, make it solid, and link it
    if xaxis != QFALSE {
        VectorSet(
            &mut (*ent).r.mins,
            -(half_width as f32),
            -(SHIELD_HALFTHICKNESS as f32),
            -((height >> 1) as f32),
        );
        VectorSet(
            &mut (*ent).r.maxs,
            half_width as f32,
            SHIELD_HALFTHICKNESS as f32,
            (height >> 1) as f32,
        );
    } else {
        VectorSet(
            &mut (*ent).r.mins,
            -(SHIELD_HALFTHICKNESS as f32),
            -(half_width as f32),
            -((height >> 1) as f32),
        );
        VectorSet(
            &mut (*ent).r.maxs,
            SHIELD_HALFTHICKNESS as f32,
            half_width as f32,
            height as f32,
        );
    }
    (*ent).clipmask = MASK_SHOT;

    // Information for shield rendering.
    //	xaxis - 1 bit
    //	height - 0-254 8 bits
    //	posWidth - 0-255 8 bits
    //  negWidth - 0 - 255 8 bits
    param_data = ((xaxis as c_int) << 24) | (height << 16) | (pos_width << 8) | neg_width;
    (*ent).s.time2 = param_data;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        (*ent).health = ((SHIELD_SIEGE_HEALTH * 1) as f32).ceil() as c_int;
    } else {
        (*ent).health = ((SHIELD_HEALTH * 1) as f32).ceil() as c_int;
    }

    (*ent).s.time = (*ent).health; //???
    (*ent).pain = Some(ShieldPain);
    (*ent).die = Some(ShieldDie);
    (*ent).touch = Some(ShieldTouch);

    // see if we're valid
    tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &(*ent).r.currentOrigin,
        (*ent).s.number,
        CONTENTS_BODY,
    );

    if tr.startsolid != 0 {
        // Something in the way!
        // make the shield non-solid very briefly
        (*ent).r.contents = 0;
        (*ent).s.eFlags |= EF_NODRAW;
        // nextthink needs to have a large enough interval to avoid excess accumulation of Activate messages
        (*ent).nextthink = (*addr_of!(level)).time + 200;
        (*ent).think = Some(ShieldGoSolid);
        (*ent).takedamage = QFALSE;
        trap::LinkEntity(ent);
    } else {
        // Get solid.
        (*ent).r.contents = CONTENTS_PLAYERCLIP | CONTENTS_SHOTCLIP; //CONTENTS_SOLID;

        (*ent).nextthink = (*addr_of!(level)).time;
        (*ent).think = Some(ShieldThink);

        (*ent).takedamage = QTRUE;
        trap::LinkEntity(ent);

        // Play raising sound...
        G_AddEvent(ent, EV_GENERAL_SOUND, *addr_of!(shieldActivateSound));
        (*ent).s.loopSound = *addr_of!(shieldLoopSound);
        (*ent).s.loopIsSoundset = QFALSE;
    }

    ShieldGoSolid(ent);
}

/// `qboolean PlaceShield(gentity_t *playerent)` (g_items.c:387) — drops a portable personal
/// shield in front of the player. On first call it lazily registers the five `shield*Sound`
/// handles ([`shieldLoopSound`]/[`shieldAttachSound`]/[`shieldActivateSound`]/
/// [`shieldDeactivateSound`]/[`shieldDamageSound`]) and caches the `HI_SHIELD` item. Traces
/// `SHIELD_PLACEDIST` ahead (then down to the floor) for room; if clear it [`G_Spawn`]s the
/// shield, orients it by `fwd`, wires the `CreateShield` think `+500ms`, parents it to the
/// player, stamps the team/owner/render fields, links it, and plays the attach sound —
/// returning `QTRUE`. `QFALSE` if there is no room.
///
/// No oracle: world traces, [`G_Spawn`]/[`trap::LinkEntity`], sound registration, and the
/// `gentity_t`→`client` graph — the shield-cluster precedent.
///
/// # Safety
/// `playerent` must point to a valid `gentity_t` whose `client` is non-NULL.
pub unsafe fn PlaceShield(playerent: *mut gentity_t) -> qboolean {
    // C: static const gitem_t *shieldItem = NULL; — retained across calls.
    static mut shieldItem: *const gitem_t = core::ptr::null();

    let shield: *mut gentity_t;
    let mut tr: trace_t;
    let mut fwd: vec3_t = [0.0; 3];
    let mut pos: vec3_t = [0.0; 3];
    let mut dest: vec3_t = [0.0; 3];
    let mins: vec3_t = [-4.0, -4.0, 0.0];
    let maxs: vec3_t = [4.0, 4.0, 4.0];

    if *addr_of!(shieldAttachSound) == 0 {
        shieldLoopSound = G_SoundIndex("sound/movers/doors/forcefield_lp.wav");
        shieldAttachSound = G_SoundIndex("sound/weapons/detpack/stick.wav");
        shieldActivateSound = G_SoundIndex("sound/movers/doors/forcefield_on.wav");
        shieldDeactivateSound = G_SoundIndex("sound/movers/doors/forcefield_off.wav");
        shieldDamageSound = G_SoundIndex("sound/effects/bumpfield.wav");
        shieldItem = BG_FindItemForHoldable(HI_SHIELD);
    }

    // can we place this in front of us?
    AngleVectors(
        &(*(*playerent).client).ps.viewangles,
        Some(&mut fwd),
        None,
        None,
    );
    fwd[2] = 0.0;
    VectorMA(
        &(*(*playerent).client).ps.origin,
        SHIELD_PLACEDIST as f32,
        &fwd,
        &mut dest,
    );
    tr = trap::Trace(
        &(*(*playerent).client).ps.origin,
        &mins,
        &maxs,
        &dest,
        (*playerent).s.number,
        MASK_SHOT,
    );
    if tr.fraction > 0.9 {
        // room in front
        VectorCopy(&tr.endpos, &mut pos);
        // drop to floor
        VectorSet(&mut dest, pos[0], pos[1], pos[2] - 4096.0);
        tr = trap::Trace(
            &pos,
            &mins,
            &maxs,
            &dest,
            (*playerent).s.number,
            MASK_SOLID,
        );
        if tr.startsolid == 0 && tr.allsolid == 0 {
            // got enough room so place the portable shield
            shield = G_Spawn();

            // Figure out what direction the shield is facing.
            if (fwd[0]).abs() > (fwd[1]).abs() {
                // shield is north/south, facing east.
                (*shield).s.angles[YAW] = 0.0;
            } else {
                // shield is along the east/west axis, facing north
                (*shield).s.angles[YAW] = 90.0;
            }
            (*shield).think = Some(CreateShield);
            (*shield).nextthink = (*addr_of!(level)).time + 500; // power up after .5 seconds
            (*shield).parent = playerent;

            // Set team number.
            (*shield).s.otherEntityNum2 = (*(*playerent).client).sess.sessionTeam;

            (*shield).s.eType = ET_SPECIAL;
            (*shield).s.modelindex = HI_SHIELD; // this'll be used in CG_Useable() for rendering.
            (*shield).classname = (*shieldItem).classname;

            (*shield).r.contents = CONTENTS_TRIGGER;

            (*shield).touch = None;
            // using an item causes it to respawn
            (*shield).r#use = None; //Use_Item;

            // allow to ride movers
            (*shield).s.groundEntityNum = tr.entityNum as c_int;

            G_SetOrigin(shield, &tr.endpos);

            (*shield).s.eFlags &= !EF_NODRAW;
            (*shield).r.svFlags &= !SVF_NOCLIENT;

            trap::LinkEntity(shield);

            (*shield).s.owner = (*playerent).s.number;
            (*shield).s.shouldtarget = QTRUE;
            if (*addr_of!(g_gametype)).integer >= GT_TEAM {
                (*shield).s.teamowner = (*(*playerent).client).sess.sessionTeam;
            } else {
                (*shield).s.teamowner = 16;
            }

            // Play placing sound...
            G_AddEvent(shield, EV_GENERAL_SOUND, *addr_of!(shieldAttachSound));

            return QTRUE;
        }
    }
    // no room
    QFALSE
}

/// `void ItemUse_Shield(gentity_t *ent)` (g_items.c:504) — the `HI_SHIELD` holdable's use
/// callback: a thin wrapper that drops a shield via [`PlaceShield`] (discarding its result).
///
/// No oracle: forwards to the trap/spawn-bound [`PlaceShield`].
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL.
pub unsafe fn ItemUse_Shield(ent: *mut gentity_t) {
    PlaceShield(ent);
}

/// Port of `g_items.c:1493` — `EWebPain`. The e-web gun's `pain` callback: it syncs the
/// gun's current `health` back onto its owning client's `ewebHealth` so the value persists
/// across re-deployments. No-op when the gun has no owner (`ownerNum == ENTITYNUM_NONE`) or
/// the owner is gone / not a client.
///
/// No oracle: pure `gentity_t` → `g_entities[ownerNum]` → `client` field sync (no
/// extractable computation) — same entity-graph precedent as the other no-oracle g_items
/// ports.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; when its `r.ownerNum` is a real entity index,
/// `g_entities` must point to the live entity array.
pub unsafe extern "C" fn EWebPain(self_: *mut gentity_t, _attacker: *mut gentity_t, _damage: c_int) {
    //update the owner's health status of me
    if (*self_).r.ownerNum != ENTITYNUM_NONE {
        let owner = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*self_).r.ownerNum as usize);

        if (*owner).inuse != QFALSE && !(*owner).client.is_null() {
            (*(*owner).client).ewebHealth = (*self_).health;
        }
    }
}

/// `void EWeb_SetBoneAngles(gentity_t *ent, char *bone, vec3_t angles)` (g_items.c:1508) —
/// "special routine for tracking angles between client and server". Walks the entity-state's
/// two bone slots (`boneIndex1`/`boneIndex2` + `boneAngles1`/`boneAngles2`) looking for `bone`'s
/// [`G_BoneIndex`]; if absent it claims the first free slot (warning out if both are taken).
/// Copies `angles` into the matching `boneAngles*` (so the client can replay the override by
/// index), then — if the server holds a ghoul2 instance — applies the override directly via
/// [`crate::trap::G2API_SetBoneAngles`] with `BONE_ANGLES_POSTMULT` and the `POSITIVE_Y` /
/// `NEGATIVE_Z` / `NEGATIVE_X` axis map packed into `s.boneOrient`. The `_XBOX` byte-index
/// branch is dropped (non-`_XBOX` `int` path). No oracle — entity-state mutation + trap. Note:
/// the C trailing slot (`thebone == &boneIndex2`, `i == 0` case) advances to a NULL terminator on
/// the next iteration via the `default` arm.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `level` must be initialised.
pub unsafe fn EWeb_SetBoneAngles(ent: *mut gentity_t, bone: &str, angles: &vec3_t) {
    let mut thebone: *mut c_int = addr_of_mut!((*ent).s.boneIndex1);
    let mut first_free: *mut c_int = core::ptr::null_mut();
    let mut i: c_int = 0;
    let boneIndex = G_BoneIndex(bone);
    let mut boneVector: *mut vec3_t = addr_of_mut!((*ent).s.boneAngles1);
    let mut freeBoneVec: *mut vec3_t = core::ptr::null_mut();

    while !thebone.is_null() {
        if *thebone == 0 && first_free.is_null() {
            //if the value is 0 then this index is clear, we can use it if we don't find the bone we want already existing.
            first_free = thebone;
            freeBoneVec = boneVector;
        } else if *thebone != 0 {
            if *thebone == boneIndex {
                //this is it
                break;
            }
        }

        match i {
            0 => {
                thebone = addr_of_mut!((*ent).s.boneIndex2);
                boneVector = addr_of_mut!((*ent).s.boneAngles2);
            }
            /*
            1 => {
                thebone = addr_of_mut!((*ent).s.boneIndex3);
                boneVector = addr_of_mut!((*ent).s.boneAngles3);
            }
            2 => {
                thebone = addr_of_mut!((*ent).s.boneIndex4);
                boneVector = addr_of_mut!((*ent).s.boneAngles4);
            }
            */
            _ => {
                thebone = core::ptr::null_mut();
                boneVector = core::ptr::null_mut();
            }
        }

        i += 1;
    }

    if thebone.is_null() {
        //didn't find it, create it
        if first_free.is_null() {
            //no free bones.. can't do a thing then.
            // #ifndef FINAL_BUILD (non-final build: warning kept)
            Com_Printf("WARNING: E-Web has no free bone indexes\n");
            return;
        }

        thebone = first_free;

        *thebone = boneIndex;
        boneVector = freeBoneVec;
    }

    //If we got here then we have a vector and an index.

    //Copy the angles over the vector in the entitystate, so we can use the corresponding index
    //to set the bone angles on the client.
    VectorCopy(angles, &mut *boneVector);

    //Now set the angles on our server instance if we have one.

    if (*ent).ghoul2.is_null() {
        return;
    }

    let flags = BONE_ANGLES_POSTMULT;
    let up = POSITIVE_Y;
    let right = NEGATIVE_Z;
    let forward = NEGATIVE_X;

    //first 3 bits is forward, second 3 bits is right, third 3 bits is up
    (*ent).s.boneOrient = forward | (right << 3) | (up << 6);

    trap::G2API_SetBoneAngles(
        (*ent).ghoul2,
        0,
        bone,
        angles,
        flags,
        up,
        right,
        forward,
        core::ptr::null_mut(),
        100,
        (*addr_of!(level)).time,
    );
}

/// `void EWeb_SetBoneAnim(gentity_t *eweb, int startFrame, int endFrame)` (g_items.c:1614) —
/// "start an animation on model_root both server side and client side". Flags the entity for
/// client-side g2 animation (`EF_G2ANIMATING`) and stashes the frame range in `s.torsoAnim` /
/// `s.legsAnim` (toggling `s.torsoFlip` to force a restart when the same anim is already
/// playing), then drives the server ghoul2 instance via
/// [`crate::trap::G2API_SetBoneAnim`] on `"model_root"` with
/// `BONE_ANIM_OVERRIDE_FREEZE|BONE_ANIM_BLEND`. No oracle — entity-state mutation + trap.
///
/// # Safety
/// `eweb` must point to a valid `gentity_t` with a live `ghoul2` instance; `level` must be
/// initialised.
pub unsafe fn EWeb_SetBoneAnim(eweb: *mut gentity_t, startFrame: c_int, endFrame: c_int) {
    //set info on the entity so it knows to start the anim on the client next snapshot.
    (*eweb).s.eFlags |= EF_G2ANIMATING;

    if (*eweb).s.torsoAnim == startFrame && (*eweb).s.legsAnim == endFrame {
        //already playing this anim, let's flag it to restart
        (*eweb).s.torsoFlip = if (*eweb).s.torsoFlip != QFALSE {
            QFALSE
        } else {
            QTRUE
        };
    } else {
        (*eweb).s.torsoAnim = startFrame;
        (*eweb).s.legsAnim = endFrame;
    }

    //now set the animation on the server ghoul2 instance.
    debug_assert!(!(*eweb).ghoul2.is_null());
    trap::G2API_SetBoneAnim(
        (*eweb).ghoul2,
        0,
        "model_root",
        startFrame,
        endFrame,
        BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
        1.0,
        (*addr_of!(level)).time,
        -1.0,
        100,
    );
}

/// g_items.c:1636 — E-Web missile damage.
const EWEB_MISSILE_DAMAGE: c_int = 20;

/// `void EWebFire(gentity_t *owner, gentity_t *eweb)` (g_items.c:1637) — "fire a shot off".
/// Reads the muzzle bolt matrix ([`crate::trap::G2API_GetBoltMatrix`] at `genericValue10`),
/// pulls the origin/forward out via [`BG_GiveMeVectorFromMatrix`], backs the spawn point 16
/// units into the bounding box, then launches a `WP_TURRET` projectile via [`CreateMissile`]
/// (`EWEB_MISSILE_DAMAGE`, `DAMAGE_DEATH_KNOCKBACK`, `MOD_TURBLAST`, bounce 8, ignoring the
/// E-Web itself) and plays the muzzle-flash effect. Bad-bolt guard (`genericValue10 == -1`)
/// asserts and bails. No oracle — trap reads + entity spawn/mutation + effect.
///
/// # Safety
/// `owner` and `eweb` must point to valid `gentity_t`s; `eweb` must hold a live `ghoul2`
/// instance with a valid muzzle bolt; `level` must be initialised.
pub unsafe fn EWebFire(owner: *mut gentity_t, eweb: *mut gentity_t) {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut p: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut bPoint: vec3_t = [0.0; 3];

    if (*eweb).genericValue10 == -1 {
        //oh no
        debug_assert!(false, "Bad e-web bolt");
        return;
    }

    //get the muzzle point
    trap::G2API_GetBoltMatrix(
        (*eweb).ghoul2,
        0,
        (*eweb).genericValue10,
        &mut boltMatrix,
        &(*eweb).s.apos.trBase,
        &(*eweb).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*eweb).modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut p);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut d);

    //Start the thing backwards into the bounding box so it can't start inside other solid things
    VectorMA(&p, -16.0, &d, &mut bPoint);

    //create the missile
    let missile = CreateMissile(&mut bPoint, &d, 1200.0, 10000, owner, QFALSE);

    (*missile).classname = c"generic_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_TURRET;

    (*missile).damage = EWEB_MISSILE_DAMAGE;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_TURBLAST;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    //ignore the e-web entity
    (*missile).passThroughNum = (*eweb).s.number + 1;

    //times it can bounce before it dies
    (*missile).bounceCount = 8;

    //play the muzzle flash
    let mut d2 = d;
    vectoangles(&d, &mut d2);
    G_PlayEffectID(G_EffectIndex("turret/muzzle_flash.efx"), &p, &d2);
}

/// `void EWebPositionUser(gentity_t *owner, gentity_t *eweb)` (g_items.c:1680) — "lock the
/// owner into place relative to the cannon pos". Reads the `cannon_Yrot` bolt
/// ([`crate::trap::G2API_GetBoltMatrix`] at `genericValue9`), projects a stand point 32 units
/// out and 4 up, then traces the owner there ([`crate::trap::Trace`]). On a clear move it
/// traces down to the floor, plays a strafe-left/right anim ([`G_SetAnim`]) toward the new spot
/// (resetting it when reversing direction, or clearing the leg timer when stationary), and
/// snaps the owner ([`G_SetOrigin`] + `ps.origin`). If the stand point is blocked it puts the
/// gun away ([`EWebDisattach`]). No oracle — trace + entity/client mutation + anim.
///
/// # Safety
/// `owner` and `eweb` must point to valid `gentity_t`s with non-null `client`s; `eweb` must
/// hold a live `ghoul2` instance; `level` must be initialised.
pub unsafe fn EWebPositionUser(owner: *mut gentity_t, eweb: *mut gentity_t) {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut p: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut tr: trace_t;

    trap::G2API_GetBoltMatrix(
        (*eweb).ghoul2,
        0,
        (*eweb).genericValue9,
        &mut boltMatrix,
        &(*eweb).s.apos.trBase,
        &(*eweb).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*eweb).modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut p);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut d);

    let p_copy = p;
    VectorMA(&p_copy, 32.0, &d, &mut p);
    p[2] = (*eweb).r.currentOrigin[2];

    p[2] += 4.0;

    tr = trap::Trace(
        &(*(*owner).client).ps.origin,
        &(*owner).r.mins,
        &(*owner).r.maxs,
        &p,
        (*owner).s.number,
        MASK_PLAYERSOLID,
    );

    if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction == 1.0 {
        //all clear, we can move there
        let mut pDown: vec3_t = [0.0; 3];

        VectorCopy(&p, &mut pDown);
        pDown[2] -= 7.0;
        tr = trap::Trace(
            &p,
            &(*owner).r.mins,
            &(*owner).r.maxs,
            &pDown,
            (*owner).s.number,
            MASK_PLAYERSOLID,
        );

        if tr.startsolid == 0 && tr.allsolid == 0 {
            VectorSubtract(&(*(*owner).client).ps.origin, &tr.endpos, &mut d);
            if VectorLength(&d) > 1.0 {
                //we moved, do some animating
                let mut dAng: vec3_t = [0.0; 3];
                let mut aFlags = SETANIM_FLAG_HOLD;

                vectoangles(&d, &mut dAng);
                dAng[YAW as usize] = AngleSubtract(
                    (*(*owner).client).ps.viewangles[YAW as usize],
                    dAng[YAW as usize],
                );
                if dAng[YAW as usize] > 0.0 {
                    if (*(*owner).client).ps.legsAnim == BOTH_STRAFE_RIGHT1 {
                        //reset to change direction
                        aFlags |= SETANIM_FLAG_OVERRIDE;
                    }
                    G_SetAnim(
                        owner,
                        addr_of_mut!((*(*owner).client).pers.cmd),
                        SETANIM_LEGS,
                        BOTH_STRAFE_LEFT1,
                        aFlags,
                        0,
                    );
                } else {
                    if (*(*owner).client).ps.legsAnim == BOTH_STRAFE_LEFT1 {
                        //reset to change direction
                        aFlags |= SETANIM_FLAG_OVERRIDE;
                    }
                    G_SetAnim(
                        owner,
                        addr_of_mut!((*(*owner).client).pers.cmd),
                        SETANIM_LEGS,
                        BOTH_STRAFE_RIGHT1,
                        aFlags,
                        0,
                    );
                }
            } else if (*(*owner).client).ps.legsAnim == BOTH_STRAFE_RIGHT1
                || (*(*owner).client).ps.legsAnim == BOTH_STRAFE_LEFT1
            {
                //don't keep animating in place
                (*(*owner).client).ps.legsTimer = 0;
            }

            G_SetOrigin(owner, &tr.endpos);
            VectorCopy(&tr.endpos, &mut (*(*owner).client).ps.origin);
        }
    } else {
        //can't move here.. stop using the thing I guess
        EWebDisattach(owner, eweb);
    }
}

const EWEB_HEALTH: c_int = 200;
const EWEB_USE_DEBOUNCE: c_int = 1000;

/// Port of `g_items.c:1748` — `EWebUpdateBoneAngles`. Per-think rotation of the e-web's
/// `cannon_Yrot`/`cannon_Xrot` bones to track the owner's view. Yaw is slewed toward the
/// owner's view yaw at a capped 4°/update; then the owner is repositioned ([`EWebPositionUser`],
/// which may stow the gun — hence the `ewebIndex == 0` early-out), and pitch is set directly
/// (scaled 0.8).
///
/// No oracle: drives the Ghoul2 bone-angle traps via [`EWeb_SetBoneAngles`] / [`EWebPositionUser`].
///
/// # Safety
/// `owner`/`eweb` must be live entities; `owner` must have a client.
pub unsafe fn EWebUpdateBoneAngles(owner: *mut gentity_t, eweb: *mut gentity_t) {
    let mut yAng: vec3_t = [0.0; 3];
    let ideal: f32;
    let mut incr: f32;
    let turnCap: f32 = 4.0; //max degrees we can turn per update

    VectorClear(&mut yAng);
    ideal = AngleSubtract(
        (*(*owner).client).ps.viewangles[YAW as usize],
        (*eweb).s.angles[YAW as usize],
    );
    incr = AngleSubtract(ideal, (*eweb).angle);

    if incr > turnCap {
        incr = turnCap;
    } else if incr < -turnCap {
        incr = -turnCap;
    }

    (*eweb).angle += incr;

    yAng[0] = (*eweb).angle;
    EWeb_SetBoneAngles(eweb, "cannon_Yrot", &yAng);

    EWebPositionUser(owner, eweb);
    if (*(*owner).client).ewebIndex == 0 {
        //was removed during position function
        return;
    }

    VectorClear(&mut yAng);
    yAng[2] = AngleSubtract(
        (*(*owner).client).ps.viewangles[PITCH as usize],
        (*eweb).s.angles[PITCH as usize],
    ) * 0.8;
    EWeb_SetBoneAngles(eweb, "cannon_Xrot", &yAng);
}

/// Port of `g_items.c:1789` — `EWebThink`. The deployed e-web's per-frame `think`: validates
/// the owner (alive, connected, still our e-web, still emplaced), otherwise either stows it
/// ([`EWebDisattach`]) or self-destructs ([`EWebDie`], `MOD_SUICIDE`). While valid it forces the
/// owner onto `WP_EMPLACED_GUN`, constrains the view yaw ([`BG_EmplacedView`]), updates bone
/// angles, and fires on `BUTTON_ATTACK` (100ms debounce) with the cheap fire/idle bone anims.
/// Always runs the falling physics ([`G_RunExPhys`]) and re-arms `nextthink`.
///
/// No oracle: trap/entity-graph think callback (fires [`EWebFire`]/[`EWebDie`]/[`G_RunExPhys`]).
///
/// # Safety
/// `self_` must be a live e-web entity; `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn EWebThink(self_: *mut gentity_t) {
    let mut killMe = false;
    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    if (*self_).r.ownerNum == ENTITYNUM_NONE {
        killMe = true;
    } else {
        let owner = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*self_).r.ownerNum as usize);

        if (*owner).inuse == QFALSE
            || (*owner).client.is_null()
            || (*(*owner).client).pers.connected != CON_CONNECTED
            || (*(*owner).client).ewebIndex != (*self_).s.number
            || (*owner).health < 1
        {
            killMe = true;
        } else if (*(*owner).client).ps.emplacedIndex != (*self_).s.number {
            //just go back to the inventory then
            EWebDisattach(owner, self_);
            return;
        }

        if !killMe {
            let mut yaw: f32 = 0.0;

            if BG_EmplacedView(
                &(*(*owner).client).ps.viewangles,
                &(*self_).s.angles,
                &mut yaw,
                (*self_).s.origin2[0],
            ) != 0
            {
                (*(*owner).client).ps.viewangles[YAW as usize] = yaw;
            }
            (*(*owner).client).ps.weapon = WP_EMPLACED_GUN;
            (*(*owner).client).ps.stats[STAT_WEAPONS as usize] = WP_EMPLACED_GUN;

            if (*self_).genericValue8 < (*addr_of!(level)).time {
                //make sure the anim timer is done
                EWebUpdateBoneAngles(owner, self_);
                if (*(*owner).client).ewebIndex == 0 {
                    //was removed during position function
                    return;
                }

                if (*(*owner).client).pers.cmd.buttons & BUTTON_ATTACK != 0 {
                    if (*self_).genericValue5 < (*addr_of!(level)).time {
                        //we can fire another shot off
                        EWebFire(owner, self_);

                        //cheap firing anim
                        EWeb_SetBoneAnim(self_, 2, 4);
                        (*self_).genericValue3 = 1;

                        //set fire debounce time
                        (*self_).genericValue5 = (*addr_of!(level)).time + 100;
                    }
                } else if (*self_).genericValue5 < (*addr_of!(level)).time && (*self_).genericValue3 != 0 {
                    //reset the anim back to non-firing
                    EWeb_SetBoneAnim(self_, 0, 1);
                    (*self_).genericValue3 = 0;
                }
            }
        }
    }

    if killMe {
        //something happened to the owner, let's explode
        EWebDie(self_, self_, self_, 999, MOD_SUICIDE);
        return;
    }

    //run some physics on it real quick so it falls and stuff properly
    G_RunExPhys(self_, gravity, mass, bounce, QFALSE as i32, core::ptr::null_mut(), 0);

    (*self_).nextthink = (*addr_of!(level)).time;
}

/// Port of `g_items.c:1872` — `EWeb_Create`. Spawns and sets up a deployed e-web entity in
/// front of `spawner`: traces for a clear box and ground, spawns the entity, restores the
/// owner's persistent e-web health, loads the Ghoul2 model + muzzle/yaw bolts, sets the
/// emplaced constraint, links it, and starts the unfolding bone anim. Returns the new entity,
/// or `NULL` when there is no room (plays the "empty" fail sound).
///
/// No oracle: spawns an entity, runs traps ([`trap::Trace`]/Ghoul2 init/bolt/[`trap::LinkEntity`])
/// and mutates the entity graph.
///
/// # Safety
/// `spawner` must be a live client entity.
pub unsafe fn EWeb_Create(spawner: *mut gentity_t) -> *mut gentity_t {
    let modelName: &str = "models/map_objects/hoth/eweb_model.glm";
    let failSound = G_SoundIndex("sound/interface/shieldcon_empty");
    let ent: *mut gentity_t;
    let mut tr: trace_t;
    let mut fAng: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut pos: vec3_t = [0.0; 3];
    let mut downPos: vec3_t = [0.0; 3];
    let mut s: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    VectorSet(&mut mins, -32.0, -32.0, -24.0);
    VectorSet(&mut maxs, 32.0, 32.0, 24.0);

    VectorSet(&mut fAng, 0.0, (*(*spawner).client).ps.viewangles[1], 0.0);
    AngleVectors(&fAng, Some(&mut fwd), None, None);

    VectorCopy(&(*(*spawner).client).ps.origin, &mut s);
    //allow some fudge
    s[2] += 12.0;

    let s_copy = s;
    VectorMA(&s_copy, 48.0, &fwd, &mut pos);

    tr = trap::Trace(&s, &mins, &maxs, &pos, (*spawner).s.number, MASK_PLAYERSOLID);

    if tr.allsolid != 0 || tr.startsolid != 0 || tr.fraction != 1.0 {
        //can't spawn here, we are in solid
        G_Sound(spawner, CHAN_AUTO, failSound);
        return core::ptr::null_mut();
    }

    ent = G_Spawn();

    (*ent).clipmask = MASK_PLAYERSOLID;
    (*ent).r.contents = MASK_PLAYERSOLID;

    (*ent).physicsObject = QTRUE;

    //for the sake of being able to differentiate client-side between this and an emplaced gun
    (*ent).s.weapon = WP_NONE;

    VectorCopy(&pos, &mut downPos);
    downPos[2] -= 18.0;
    tr = trap::Trace(&pos, &mins, &maxs, &downPos, (*spawner).s.number, MASK_PLAYERSOLID);

    if tr.startsolid != 0 || tr.allsolid != 0 || tr.fraction == 1.0 || (tr.entityNum as c_int) < ENTITYNUM_WORLD {
        //didn't hit ground.
        G_FreeEntity(ent);
        G_Sound(spawner, CHAN_AUTO, failSound);
        return core::ptr::null_mut();
    }

    VectorCopy(&tr.endpos, &mut pos);

    G_SetOrigin(ent, &pos);

    VectorCopy(&fAng, &mut (*ent).s.apos.trBase);
    VectorCopy(&fAng, &mut (*ent).r.currentAngles);

    (*ent).s.owner = (*spawner).s.number;
    (*ent).s.teamowner = (*(*spawner).client).sess.sessionTeam;

    (*ent).takedamage = QTRUE;

    if (*(*spawner).client).ewebHealth <= 0 {
        //refresh the owner's e-web health if its last e-web did not exist or was killed
        (*(*spawner).client).ewebHealth = EWEB_HEALTH;
    }

    //resume health of last deployment
    (*ent).maxHealth = EWEB_HEALTH;
    (*ent).health = (*(*spawner).client).ewebHealth;
    G_ScaleNetHealth(ent);

    (*ent).die = Some(EWebDie);
    (*ent).pain = Some(EWebPain);

    (*ent).think = Some(EWebThink);
    (*ent).nextthink = (*addr_of!(level)).time;

    //set up the g2 model info
    (*ent).s.modelGhoul2 = 1;
    (*ent).s.g2radius = 128;
    (*ent).s.modelindex = G_ModelIndex(modelName);

    trap::G2API_InitGhoul2Model(
        addr_of_mut!((*ent).ghoul2),
        c"models/map_objects/hoth/eweb_model.glm".as_ptr(),
        0,
        0,
        0,
        0,
        0,
    );

    if (*ent).ghoul2.is_null() {
        //should not happen, but just to be safe.
        G_FreeEntity(ent);
        return core::ptr::null_mut();
    }

    //initialize bone angles
    EWeb_SetBoneAngles(ent, "cannon_Yrot", &vec3_origin);
    EWeb_SetBoneAngles(ent, "cannon_Xrot", &vec3_origin);

    (*ent).genericValue10 = trap::G2API_AddBolt((*ent).ghoul2, 0, "*cannonflash"); //muzzle bolt
    (*ent).genericValue9 = trap::G2API_AddBolt((*ent).ghoul2, 0, "cannon_Yrot"); //for placing the owner relative to rotation

    //set the constraints for this guy as an emplaced weapon, and his constraint angles
    (*ent).s.origin2[0] = 360.0; //360 degrees in either direction

    VectorCopy(&fAng, &mut (*ent).s.angles); //consider "angle 0" for constraint

    //angle of y rot bone
    (*ent).angle = 0.0;

    (*ent).r.ownerNum = (*spawner).s.number;
    trap::LinkEntity(ent);

    //store off the owner's current weapons, we will be forcing him to use the "emplaced" weapon
    (*ent).genericValue11 = (*(*spawner).client).ps.stats[STAT_WEAPONS as usize];

    //start the "unfolding" anim
    EWeb_SetBoneAnim(ent, 4, 20);
    //don't allow use until the anim is done playing (rough time estimate)
    (*ent).genericValue8 = (*addr_of!(level)).time + 500;

    VectorCopy(&mins, &mut (*ent).r.mins);
    VectorCopy(&maxs, &mut (*ent).r.maxs);

    ent
}

/// Port of `g_items.c:1997` — `ItemUse_UseEWeb`. The holdable-item handler that toggles the
/// e-web: debounced ([`ewebTime`]), refuses while busy (`weaponTime`/`forceHandExtend`) or
/// while riding someone else's emplaced gun; otherwise stows the existing e-web
/// ([`EWebDisattach`]) or creates a new one ([`EWeb_Create`]), wiring `ewebIndex`/`emplacedIndex`.
///
/// No oracle: entity-graph mutation through [`EWeb_Create`]/[`EWebDisattach`].
///
/// # Safety
/// `ent` must be a live client entity.
pub unsafe fn ItemUse_UseEWeb(ent: *mut gentity_t) {
    if (*(*ent).client).ewebTime > (*addr_of!(level)).time {
        //can't use again yet
        return;
    }

    if (*(*ent).client).ps.weaponTime > 0
        || (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
    {
        //busy doing something else
        return;
    }

    if (*(*ent).client).ps.emplacedIndex != 0 && (*(*ent).client).ewebIndex == 0 {
        //using an emplaced gun already that isn't our own e-web
        return;
    }

    if (*(*ent).client).ewebIndex != 0 {
        //put it away
        EWebDisattach(ent, core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*(*ent).client).ewebIndex as usize));
    } else {
        //create it
        let eweb = EWeb_Create(ent);

        if !eweb.is_null() {
            //if it's null the thing couldn't spawn (probably no room)
            (*(*ent).client).ewebIndex = (*eweb).s.number;
            (*(*ent).client).ps.emplacedIndex = (*eweb).s.number;
        }
    }

    (*(*ent).client).ewebTime = (*addr_of!(level)).time + EWEB_USE_DEBOUNCE;
}

//---------------------------------
/// Port of `g_items.c:642` — `pas_adjust_enemy`. The portable-sentry-gun's per-think
/// enemy-validation step: drops its current `enemy` (and shuts the gun's sound down) when
/// the target is dead or no longer in line-of-sight. The LOS test traces from the gun's
/// trajectory base toward the enemy's eye/origin (clients drop 15 units off their
/// `ps.origin`; non-clients use `r.currentOrigin`) and fails if the trace hits something
/// solid short of the enemy. On failure — gated on the `bounceCount` debounce so it can't
/// ping-pong on and off — it nulls `enemy`, plays the turret shutdown sound, re-arms the
/// debounce, and sets `aimDebounceTime` so the turret pings for 5 seconds.
///
/// No oracle: fires [`trap::Trace`] / [`G_Sound`] / [`G_SoundIndex`] and mutates the
/// `gentity_t` graph (and draws the shared LCG via [`random`]) — same trap/entity-graph
/// precedent as the rest of the sentry-gun cluster (`turret_die` / `sentryExpire`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `enemy` is non-NULL (the C unconditionally
/// dereferences `ent->enemy`); when the enemy is a client, its `client` must be valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe fn pas_adjust_enemy(ent: *mut gentity_t) {
    let tr: trace_t;
    let mut keep = QTRUE;

    let lvl = addr_of!(level);

    if (*(*ent).enemy).health <= 0 {
        keep = QFALSE;
    } else {
        let mut org: vec3_t = [0.0; 3];
        let mut org2: vec3_t = [0.0; 3];

        VectorCopy(&(*ent).s.pos.trBase, &mut org2);

        if !(*(*ent).enemy).client.is_null() {
            VectorCopy(&(*(*(*ent).enemy).client).ps.origin, &mut org);
            org[2] -= 15.0;
        } else {
            VectorCopy(&(*(*ent).enemy).r.currentOrigin, &mut org);
        }

        tr = trap::Trace(
            &org2,
            &vec3_origin,
            &vec3_origin,
            &org,
            (*ent).s.number,
            MASK_SHOT,
        );

        if tr.allsolid != 0
            || tr.startsolid != 0
            || tr.fraction < 0.9
            || tr.entityNum as c_int == (*ent).s.number
        {
            if tr.entityNum as c_int != (*(*ent).enemy).s.number {
                // trace failed
                keep = QFALSE;
            }
        }
    }

    if keep != QFALSE {
        //ent->bounceCount = level.time + 500 + random() * 150;
    } else if (*ent).bounceCount < (*lvl).time && !(*ent).enemy.is_null() {
        // don't ping pong on and off
        (*ent).enemy = core::ptr::null_mut();
        // shut-down sound
        G_Sound(ent, CHAN_BODY, G_SoundIndex("sound/chars/turret/shutdown.wav"));

        // C `level.time + 500 + random() * 150` evaluates the whole sum in `float` (the
        // `random()*150` term promotes the int part to `float`) and truncates once on the
        // `int` store — reproduced here as a single `f32` sum cast to `c_int`.
        (*ent).bounceCount = (((*lvl).time + 500) as f32 + random() * 150.0) as c_int;

        // make turret play ping sound for 5 seconds
        (*ent).aimDebounceTime = (*lvl).time + 5000;
    }
}

// g_items.c:540 — `#define TURRET_RADIUS 800` (sentry-gun acquisition radius).
const TURRET_RADIUS: f32 = 800.0;

// g_items.c:704-705 — sentry-gun death-delay / lifetime windows (#define, ms).
const TURRET_DEATH_DELAY: c_int = 2000;
const TURRET_LIFETIME: c_int = 60000;

// g_items.c:973 — `#define TURRET_AMMO_COUNT 40` (sentry-gun starting shots).
const TURRET_AMMO_COUNT: c_int = 40;

/// Port of `g_items.c:521` — `pas_fire`. The portable-sentry-gun's fire step: aims a turret
/// missile from 24 units above the gun, offset 16 units along the (normalized) direction to
/// the enemy's eye, then advances the gun's object physics one frame. No oracle (spawns a
/// missile via [`WP_FireTurretMissile`] + runs object physics — full entity-graph mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `enemy` is a non-NULL client (the C
/// unconditionally dereferences `ent->enemy->client`).
pub unsafe fn pas_fire(ent: *mut gentity_t) {
    let mut fwd: vec3_t = [0.0; 3];
    let mut myOrg: vec3_t = [0.0; 3];
    let mut enOrg: vec3_t = [0.0; 3];

    VectorCopy(&(*ent).r.currentOrigin, &mut myOrg);
    myOrg[2] += 24.0;

    VectorCopy(&(*(*(*ent).enemy).client).ps.origin, &mut enOrg);
    enOrg[2] += 24.0;

    VectorSubtract(&enOrg, &myOrg, &mut fwd);
    VectorNormalize(&mut fwd);

    myOrg[0] += fwd[0] * 16.0;
    myOrg[1] += fwd[1] * 16.0;
    myOrg[2] += fwd[2] * 16.0;

    WP_FireTurretMissile(
        core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*ent).genericValue3 as usize),
        &mut myOrg,
        &fwd,
        QFALSE,
        10,
        2300,
        MOD_SENTRY,
        ent,
    );

    G_RunObject(ent);
}

/// Port of `g_items.c:547` — `pas_find_enemies`. The portable-sentry-gun's target-acquisition
/// sweep: collects everything within `TURRET_RADIUS` (via [`G_RadiusList`]), skips non-clients,
/// itself, the dead/non-damageable/notarget, allies, its owner, anything not in PVS, and
/// vehicles, then line-of-sight-traces to each survivor and locks onto the nearest one with a
/// clear shot (playing the wind-up sound on first acquisition). Returns whether an enemy was
/// found. No oracle (trap traces/PVS + entity-graph mutation + the shared LCG via [`random`]).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
unsafe fn pas_find_enemies(self_: *mut gentity_t) -> qboolean {
    let mut found = QFALSE;
    let count: c_int;
    let mut bestDist: f32 = TURRET_RADIUS * TURRET_RADIUS;
    let mut enemyDist: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut entity_list: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];

    let lvl = addr_of!(level);

    if (*self_).aimDebounceTime > (*lvl).time {
        // time since we've been shut off
        // We were active and alert, i.e. had an enemy in the last 3 secs
        if (*self_).painDebounceTime < (*lvl).time {
            G_Sound(self_, CHAN_BODY, G_SoundIndex("sound/chars/turret/ping.wav"));
            (*self_).painDebounceTime = (*lvl).time + 1000;
        }
    }

    VectorCopy(&(*self_).s.pos.trBase, &mut org2);

    count = G_RadiusList(&org2, TURRET_RADIUS, self_, QTRUE, &mut entity_list);

    for i in 0..count {
        let target = entity_list[i as usize];

        if (*target).client.is_null() {
            continue;
        }
        if target == self_
            || (*target).takedamage == QFALSE
            || (*target).health <= 0
            || ((*target).flags & FL_NOTARGET) != 0
        {
            continue;
        }
        if (*self_).alliedTeam != 0
            && (*(*target).client).sess.sessionTeam == (*self_).alliedTeam
        {
            continue;
        }
        if (*self_).genericValue3 == (*target).s.number {
            continue;
        }
        if trap::InPVS(&org2, &(*target).r.currentOrigin) == QFALSE {
            continue;
        }

        if (*target).s.eType == ET_NPC && (*target).s.NPC_class == CLASS_VEHICLE {
            //don't get mad at vehicles, silly.
            continue;
        }

        if !(*target).client.is_null() {
            VectorCopy(&(*(*target).client).ps.origin, &mut org);
        } else {
            VectorCopy(&(*target).r.currentOrigin, &mut org);
        }

        let tr = trap::Trace(
            &org2,
            &vec3_origin,
            &vec3_origin,
            &org,
            (*self_).s.number,
            MASK_SHOT,
        );

        if tr.allsolid == 0
            && tr.startsolid == 0
            && (tr.fraction == 1.0 || tr.entityNum as c_int == (*target).s.number)
        {
            // Only acquire if have a clear shot, Is it in range and closer than our best?
            VectorSubtract(&(*target).r.currentOrigin, &(*self_).r.currentOrigin, &mut enemyDir);
            enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < bestDist {
                // all things equal, keep current
                if (*self_).attackDebounceTime + 100 < (*lvl).time {
                    // We haven't fired or acquired an enemy in the last 2 seconds-start-up sound
                    G_Sound(
                        self_,
                        CHAN_BODY,
                        G_SoundIndex("sound/chars/turret/startup.wav"),
                    );

                    // Wind up turrets for a bit
                    (*self_).attackDebounceTime = (*lvl).time + 900 + (random() * 200.0) as c_int;
                }

                G_SetEnemy(self_, target);
                bestDist = enemyDist;
                found = QTRUE;
            }
        }
    }

    found
}

/// Port of `g_items.c:708` — `pas_think`. The portable-sentry-gun's per-frame `think`: goes
/// non-solid if a client is stuck inside it, self-destructs if its owner left/changed teams,
/// arms its start-up frame, retires after `TURRET_LIFETIME`, validates/acquires an enemy
/// ([`pas_adjust_enemy`] / [`pas_find_enemies`]), sweep-aims its yaw/pitch toward the target
/// (or idles in a sine sweep), and fires ([`pas_fire`]) while ammo (`count`) lasts — shutting
/// down (→ [`sentryExpire`]) when it runs dry. `extern "C"` for the `gentity_t::think` slot.
/// No oracle (trap box query + entity-graph mutation + `sin`/`random`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn pas_think(ent: *mut gentity_t) {
    let mut moved: qboolean;
    let diffYaw: f32;
    let mut diffPitch: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut frontAngles: vec3_t = [0.0; 3];
    let mut backAngles: vec3_t = [0.0; 3];
    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut iEntityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut numListedEntities: c_int;
    let mut i: c_int = 0;
    let mut clTrapped = QFALSE;
    let mut testMins: vec3_t = [0.0; 3];
    let mut testMaxs: vec3_t = [0.0; 3];

    let lvl = addr_of!(level);

    testMins[0] = (*ent).r.currentOrigin[0] + (*ent).r.mins[0] + 4.0;
    testMins[1] = (*ent).r.currentOrigin[1] + (*ent).r.mins[1] + 4.0;
    testMins[2] = (*ent).r.currentOrigin[2] + (*ent).r.mins[2] + 4.0;

    testMaxs[0] = (*ent).r.currentOrigin[0] + (*ent).r.maxs[0] - 4.0;
    testMaxs[1] = (*ent).r.currentOrigin[1] + (*ent).r.maxs[1] - 4.0;
    testMaxs[2] = (*ent).r.currentOrigin[2] + (*ent).r.maxs[2] - 4.0;

    numListedEntities = trap::EntitiesInBox(&testMins, &testMaxs, &mut iEntityList);

    while i < numListedEntities {
        if iEntityList[i as usize] < MAX_CLIENTS as c_int {
            //client stuck inside me. go nonsolid.
            let clNum = iEntityList[i as usize];

            numListedEntities = trap::EntitiesInBox(
                &(*core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(clNum as usize)).r.absmin,
                &(*core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(clNum as usize)).r.absmax,
                &mut iEntityList,
            );

            i = 0;
            while i < numListedEntities {
                if iEntityList[i as usize] == (*ent).s.number {
                    clTrapped = QTRUE;
                    break;
                }
                i += 1;
            }
            break;
        }

        i += 1;
    }

    if clTrapped != QFALSE {
        (*ent).r.contents = 0;
        (*ent).s.fireflag = 0;
        (*ent).nextthink = (*lvl).time + FRAMETIME;
        return;
    } else {
        (*ent).r.contents = CONTENTS_SOLID;
    }

    let owner = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*ent).genericValue3 as usize);
    if (*owner).inuse == QFALSE
        || (*owner).client.is_null()
        || (*(*owner).client).sess.sessionTeam != (*ent).genericValue2
    {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*lvl).time;
        return;
    }

    //	G_RunObject(ent);

    if (*ent).damage == 0 {
        (*ent).damage = 1;
        (*ent).nextthink = (*lvl).time + FRAMETIME;
        return;
    }

    if ((*ent).genericValue8 + TURRET_LIFETIME) < (*lvl).time {
        G_Sound(
            ent,
            CHAN_BODY,
            G_SoundIndex("sound/chars/turret/shutdown.wav"),
        );
        (*ent).s.bolt2 = ENTITYNUM_NONE;
        (*ent).s.fireflag = 2;

        (*ent).think = Some(sentryExpire);
        (*ent).nextthink = (*lvl).time + TURRET_DEATH_DELAY;
        return;
    }

    (*ent).nextthink = (*lvl).time + FRAMETIME;

    if !(*ent).enemy.is_null() {
        // make sure that the enemy is still valid
        pas_adjust_enemy(ent);
    }

    if !(*ent).enemy.is_null() {
        if (*(*ent).enemy).client.is_null() {
            (*ent).enemy = core::ptr::null_mut();
        } else if (*(*ent).enemy).s.number == (*ent).s.number {
            (*ent).enemy = core::ptr::null_mut();
        } else if (*(*ent).enemy).health < 1 {
            (*ent).enemy = core::ptr::null_mut();
        }
    }

    if (*ent).enemy.is_null() {
        pas_find_enemies(ent);
    }

    if !(*ent).enemy.is_null() {
        (*ent).s.bolt2 = (*(*ent).enemy).s.number;
    } else {
        (*ent).s.bolt2 = ENTITYNUM_NONE;
    }

    moved = QFALSE;
    // C: `diffYaw = 0.0f; diffPitch = 0.0f;` — the `diffYaw` init is dead (the following
    // if/else unconditionally re-assigns it), so it is dropped to avoid a dead-store warning;
    // `diffPitch` keeps its init because the no-enemy branch leaves it untouched.
    diffPitch = 0.0;

    (*ent).speed = AngleNormalize360((*ent).speed);
    (*ent).random = AngleNormalize360((*ent).random);

    if !(*ent).enemy.is_null() {
        // ...then we'll calculate what new aim adjustments we should attempt to make this frame
        // Aim at enemy
        if !(*(*ent).enemy).client.is_null() {
            VectorCopy(&(*(*(*ent).enemy).client).ps.origin, &mut org);
        } else {
            VectorCopy(&(*(*ent).enemy).r.currentOrigin, &mut org);
        }

        VectorSubtract(&org, &(*ent).r.currentOrigin, &mut enemyDir);
        vectoangles(&enemyDir, &mut desiredAngles);

        diffYaw = AngleSubtract((*ent).speed, desiredAngles[YAW]);
        diffPitch = AngleSubtract((*ent).random, desiredAngles[PITCH]);
    } else {
        // no enemy, so make us slowly sweep back and forth as if searching for a new one
        // C: `sin( level.time * 0.0001f + ent->count ) * 2.0f` — the `float` arg promotes to
        // `double` through `sin`, the `* 2.0f` term promotes too, and the result truncates to
        // the `float` store. Mirror that: compute the arg in `f32`, do the sin/mul in `f64`.
        let sweep_arg = (*lvl).time as f32 * 0.0001 + (*ent).count as f32;
        diffYaw = ((sweep_arg as f64).sin() * 2.0) as f32;
    }

    if diffYaw.abs() > 0.25 {
        moved = QTRUE;

        if diffYaw.abs() > 10.0 {
            // cap max speed
            (*ent).speed += if diffYaw > 0.0 { -10.0 } else { 10.0 };
        } else {
            // small enough
            (*ent).speed -= diffYaw;
        }
    }

    if diffPitch.abs() > 0.25 {
        moved = QTRUE;

        if diffPitch.abs() > 4.0 {
            // cap max speed
            (*ent).random += if diffPitch > 0.0 { -4.0 } else { 4.0 };
        } else {
            // small enough
            (*ent).random -= diffPitch;
        }
    }

    // the bone axes are messed up, so hence some dumbness here
    VectorSet(&mut frontAngles, -(*ent).random, 0.0, 0.0);
    VectorSet(&mut backAngles, 0.0, 0.0, (*ent).speed);

    if moved != QFALSE {
        //ent->s.loopSound = G_SoundIndex( "sound/chars/turret/move.wav" );
    } else {
        (*ent).s.loopSound = 0;
        (*ent).s.loopIsSoundset = QFALSE;
    }

    if !(*ent).enemy.is_null() && (*ent).attackDebounceTime < (*lvl).time {
        (*ent).count -= 1;

        if (*ent).count != 0 {
            pas_fire(ent);
            (*ent).s.fireflag = 1;
            (*ent).attackDebounceTime = (*lvl).time + 200;
        } else {
            //ent->nextthink = 0;
            G_Sound(
                ent,
                CHAN_BODY,
                G_SoundIndex("sound/chars/turret/shutdown.wav"),
            );
            (*ent).s.bolt2 = ENTITYNUM_NONE;
            (*ent).s.fireflag = 2;

            (*ent).think = Some(sentryExpire);
            (*ent).nextthink = (*lvl).time + TURRET_DEATH_DELAY;
        }
    } else {
        (*ent).s.fireflag = 0;
    }

    // silence "assigned but never read" — these vectors mirror the C locals exactly (the
    // C also computes frontAngles/backAngles purely for documentation; they are unused).
    let _ = frontAngles;
    let _ = backAngles;
}

/// Port of `g_items.c:978` — `SP_PAS`. Spawn/setup of the portable-sentry-gun base: grants
/// starting ammo, sets the model's special-turret bolt hints + bounds, kicks the object
/// physics once, installs the [`pas_think`] think and the [`turret_die`] death callback, and
/// plays the start-up sound. No oracle (entity-state spawn: think/die fn-pointers + bounds).
///
/// # Safety
/// `base` must point to a valid `gentity_t`.
pub unsafe fn SP_PAS(base: *mut gentity_t) {
    let lvl = addr_of!(level);

    if (*base).count == 0 {
        // give ammo
        (*base).count = TURRET_AMMO_COUNT;
    }

    (*base).s.bolt1 = 1; //This is a sort of hack to indicate that this model needs special turret things done to it
    (*base).s.bolt2 = ENTITYNUM_NONE; //store our current enemy index

    (*base).damage = 0; // start animation flag

    VectorSet(&mut (*base).r.mins, -8.0, -8.0, 0.0);
    VectorSet(&mut (*base).r.maxs, 8.0, 8.0, 24.0);

    G_RunObject(base);

    (*base).think = Some(pas_think);
    (*base).nextthink = (*lvl).time + FRAMETIME;

    if (*base).health == 0 {
        (*base).health = 50;
    }

    (*base).takedamage = QTRUE;
    (*base).die = Some(turret_die);

    (*base).physicsObject = QTRUE;

    G_Sound(
        base,
        CHAN_BODY,
        G_SoundIndex("sound/chars/turret/startup.wav"),
    );
}

/// Port of `g_items.c:1014` — `ItemUse_Sentry`. Deploys a portable-sentry-gun 64 units in
/// front of the using client: spawns the entity, sets its model/ghoul2/bounds/contents/owner
/// fields, links it, marks the owner's `sentryDeployed`, and hands off to [`SP_PAS`]. No oracle
/// (entity spawn + link: fn-pointers, contents, owner bookkeeping).
///
/// # Safety
/// `ent` may be NULL (guarded); when non-NULL it must be a valid `gentity_t` whose `client`
/// (also guarded) is valid.
pub unsafe fn ItemUse_Sentry(ent: *mut gentity_t) {
    let mut fwd: vec3_t = [0.0; 3];
    let mut fwdorg: vec3_t = [0.0; 3];
    let mut yawonly: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    let lvl = addr_of!(level);

    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    VectorSet(&mut mins, -8.0, -8.0, 0.0);
    VectorSet(&mut maxs, 8.0, 8.0, 24.0);

    yawonly[ROLL] = 0.0;
    yawonly[PITCH] = 0.0;
    yawonly[YAW] = (*(*ent).client).ps.viewangles[YAW];

    AngleVectors(&yawonly, Some(&mut fwd), None, None);

    fwdorg[0] = (*(*ent).client).ps.origin[0] + fwd[0] * 64.0;
    fwdorg[1] = (*(*ent).client).ps.origin[1] + fwd[1] * 64.0;
    fwdorg[2] = (*(*ent).client).ps.origin[2] + fwd[2] * 64.0;

    let sentry = G_Spawn();

    (*sentry).classname = c"sentryGun".as_ptr() as *mut c_char;
    (*sentry).s.modelindex = G_ModelIndex("models/items/psgun.glm"); //replace ASAP

    (*sentry).s.g2radius = 30; // C: `= 30.0f` truncated to the int `g2radius` field
    (*sentry).s.modelGhoul2 = 1;

    G_SetOrigin(sentry, &fwdorg);
    (*sentry).parent = ent;
    (*sentry).r.contents = CONTENTS_SOLID;
    (*sentry).s.solid = 2;
    (*sentry).clipmask = MASK_SOLID;
    VectorCopy(&mins, &mut (*sentry).r.mins);
    VectorCopy(&maxs, &mut (*sentry).r.maxs);
    (*sentry).genericValue3 = (*ent).s.number;
    (*sentry).genericValue2 = (*(*ent).client).sess.sessionTeam; //so we can remove ourself if our owner changes teams
    (*sentry).r.absmin[0] = (*sentry).s.pos.trBase[0] + (*sentry).r.mins[0];
    (*sentry).r.absmin[1] = (*sentry).s.pos.trBase[1] + (*sentry).r.mins[1];
    (*sentry).r.absmin[2] = (*sentry).s.pos.trBase[2] + (*sentry).r.mins[2];
    (*sentry).r.absmax[0] = (*sentry).s.pos.trBase[0] + (*sentry).r.maxs[0];
    (*sentry).r.absmax[1] = (*sentry).s.pos.trBase[1] + (*sentry).r.maxs[1];
    (*sentry).r.absmax[2] = (*sentry).s.pos.trBase[2] + (*sentry).r.maxs[2];
    (*sentry).s.eType = ET_GENERAL;
    (*sentry).s.pos.trType = TR_GRAVITY; //STATIONARY;
    (*sentry).s.pos.trTime = (*lvl).time;
    (*sentry).touch = Some(SentryTouch);
    (*sentry).nextthink = (*lvl).time;
    (*sentry).genericValue4 = ENTITYNUM_NONE; //genericValue4 used as enemy index

    (*sentry).genericValue5 = 1000;

    (*sentry).genericValue8 = (*lvl).time;

    (*sentry).alliedTeam = (*(*ent).client).sess.sessionTeam;

    (*(*ent).client).ps.fd.sentryDeployed = QTRUE;

    trap::LinkEntity(sentry);

    (*sentry).s.owner = (*ent).s.number;
    (*sentry).s.shouldtarget = QTRUE;
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        (*sentry).s.teamowner = (*(*ent).client).sess.sessionTeam;
    } else {
        (*sentry).s.teamowner = 16;
    }

    SP_PAS(sentry);
}

/// Port of `g_items.c:1353` — `ItemUse_UseDisp`. The health/ammo-dispenser holdable's `use`:
/// after the per-client toss-debounce / busy-hand gates pass, it tosses the matching item
/// ([`BG_FindItem`] for the medpack-instant or ammo-all classname) — spawning the item entity
/// at eye height, special-spawning it ([`G_SpecialSpawnItem`]), giving it a forward toss
/// velocity, and firing the `EV_LOCALTIMER` HUD-cooldown event. No oracle (entity spawn/link +
/// temp-entity event over the client→ps graph).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the C dereferences `ent->client` unguarded so it
/// must be non-NULL.
pub unsafe fn ItemUse_UseDisp(ent: *mut gentity_t, type_: c_int) {
    // g_items.c:1333-1334 — DISP_HEALTH_ITEM / DISP_AMMO_ITEM (local #defines).
    const DISP_HEALTH_ITEM: &CStr = c"item_medpak_instant";
    const DISP_AMMO_ITEM: &CStr = c"ammo_all";

    let item: *mut gitem_t;
    let eItem: *mut gentity_t;

    let lvl = addr_of!(level);

    if (*ent).client.is_null() || (*(*ent).client).tossableItemDebounce > (*lvl).time {
        //can't use it again yet
        return;
    }

    if (*(*ent).client).ps.weaponTime > 0
        || (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
    {
        //busy doing something else
        return;
    }

    (*(*ent).client).tossableItemDebounce = (*lvl).time + TOSS_DEBOUNCE_TIME;

    if type_ == HI_HEALTHDISP {
        item = BG_FindItem(DISP_HEALTH_ITEM.as_ptr());
    } else {
        item = BG_FindItem(DISP_AMMO_ITEM.as_ptr());
    }

    if !item.is_null() {
        let mut fwd: vec3_t = [0.0; 3];
        let mut pos: vec3_t = [0.0; 3];
        let te: *mut gentity_t;

        eItem = G_Spawn();
        (*eItem).r.ownerNum = (*ent).s.number;
        (*eItem).classname = (*item).classname;

        VectorCopy(&(*(*ent).client).ps.origin, &mut pos);
        pos[2] += (*(*ent).client).ps.viewheight as f32;

        G_SetOrigin(eItem, &pos);
        VectorCopy(&(*eItem).r.currentOrigin, &mut (*eItem).s.origin);
        trap::LinkEntity(eItem);

        G_SpecialSpawnItem(eItem, item);

        AngleVectors(&(*(*ent).client).ps.viewangles, Some(&mut fwd), None, None);
        VectorScale(&fwd, 128.0, &mut (*eItem).epVelocity);
        (*eItem).epVelocity[2] = 16.0;

        //	G_SetAnim( ent, NULL, SETANIM_TORSO, BOTH_THERMAL_THROW, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD, 0 );

        te = G_TempEntity(&(*(*ent).client).ps.origin, EV_LOCALTIMER);
        (*te).s.time = (*lvl).time;
        (*te).s.time2 = TOSS_DEBOUNCE_TIME;
        (*te).s.owner = (*(*ent).client).ps.clientNum;
    }
}

/// Port of `g_items.c:2349` — `CheckItemCanBePickedUpByNPC`.
///
/// Decides whether a non-player, in-combat NPC may grab a dropped item that does not belong
/// to the player. The commented-out `INV_SECURITY_KEY` exclusion is kept commented, as in the
/// C source.
///
/// # Safety
/// Dereferences `item`/`pickerupper` and `pickerupper->NPC` (`gNPC_t`); both must be valid
/// live entities (callers gate on `pickerupper->NPC` being non-null via the predicate itself).
pub unsafe fn CheckItemCanBePickedUpByNPC(
    item: *mut gentity_t,
    pickerupper: *mut gentity_t,
) -> qboolean {
    if ((*item).flags & FL_DROPPED_ITEM) != 0
        && (*item).activator != core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()
        && (*pickerupper).s.number != 0
        && (*pickerupper).s.weapon == WP_NONE
        && !(*pickerupper).enemy.is_null()
        && (*pickerupper).painDebounceTime < (*addr_of!(level)).time
        && !(*pickerupper).NPC.is_null()
        && (*(*pickerupper).NPC).surrenderTime < (*addr_of!(level)).time //not surrendering
        && ((*(*pickerupper).NPC).scriptFlags & SCF_FORCED_MARCH) == 0 //not being forced to march
        /*&& item->item->giTag != INV_SECURITY_KEY*/
    {
        //non-player, in combat, picking up a dropped item that does NOT belong to the player and it *not* a security key
        if (*addr_of!(level)).time - (*item).s.time < 3000
        //was 5000
        {
            return QFALSE;
        }
        return QTRUE;
    }
    QFALSE
}

/*
===============
Touch_Item
===============
*/
/// Port of `g_items.c:2375` — `Touch_Item`. The `touch` callback installed on item
/// entities: applies the shared (client+server) pickup rules, dispatches to the
/// item-specific `Pickup_*` handler, fires the pickup event/sound, runs the item's targets,
/// and arms the respawn (or frees a dropped item).
///
/// `extern "C"`: stored in the `touch` function-pointer slot of a `gentity_t` (installed by
/// [`LaunchItem`] and `FinishSpawningItem`).
///
/// No oracle: heavily side-effecting over the `gentity_t`→`client`→`ps` graph, the temp-entity
/// system, and `level.time`.
///
// TODO: Port-Bug
pub unsafe extern "C" fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, _trace: *mut trace_t) {
    let respawn: c_int;
    let mut predict: qboolean;

    if (*ent).genericValue10 > (*addr_of!(level)).time
        && !other.is_null()
        && (*other).s.number == (*ent).genericValue11
    {
        //this is the ent that we don't want to be able to touch us for x seconds
        return;
    }

    if (*ent).s.eFlags & EF_ITEMPLACEHOLDER != 0 {
        return;
    }

    if (*ent).s.eFlags & EF_NODRAW != 0 {
        return;
    }

    if (*(*ent).item).giType == IT_WEAPON
        && (*ent).s.powerups != 0
        && (*ent).s.powerups < (*addr_of!(level)).time
    {
        (*ent).s.generic1 = 0;
        (*ent).s.powerups = 0;
    }

    if (*other).client.is_null() {
        return;
    }
    if (*other).health < 1 {
        return; // dead people can't pickup
    }

    if (*(*ent).item).giType == IT_POWERUP
        && ((*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_LIGHT
            || (*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_DARK)
    {
        if (*(*ent).item).giTag == PW_FORCE_ENLIGHTENED_LIGHT {
            if (*(*other).client).ps.fd.forceSide != FORCE_LIGHTSIDE {
                return;
            }
        } else if (*(*other).client).ps.fd.forceSide != FORCE_DARKSIDE {
            return;
        }
    }

    // the same pickup rules are used for client side and server side
    if BG_CanItemBeGrabbed(
        (*addr_of!(g_gametype)).integer,
        &(*ent).s,
        &(*(*other).client).ps,
    ) == QFALSE
    {
        return;
    }

    if (*(*other).client).NPC_class == CLASS_ATST
        || (*(*other).client).NPC_class == CLASS_GONK
        || (*(*other).client).NPC_class == CLASS_MARK1
        || (*(*other).client).NPC_class == CLASS_MARK2
        || (*(*other).client).NPC_class == CLASS_MOUSE
        || (*(*other).client).NPC_class == CLASS_PROBE
        || (*(*other).client).NPC_class == CLASS_PROTOCOL
        || (*(*other).client).NPC_class == CLASS_R2D2
        || (*(*other).client).NPC_class == CLASS_R5D2
        || (*(*other).client).NPC_class == CLASS_SEEKER
        || (*(*other).client).NPC_class == CLASS_REMOTE
        || (*(*other).client).NPC_class == CLASS_RANCOR
        || (*(*other).client).NPC_class == CLASS_WAMPA
        //other->client->NPC_class == CLASS_JAWA || //FIXME: in some cases it's okay?
        || (*(*other).client).NPC_class == CLASS_UGNAUGHT //FIXME: in some cases it's okay?
        || (*(*other).client).NPC_class == CLASS_SENTRY
    {
        //FIXME: some flag would be better
        //droids can't pick up items/weapons!
        return;
    }

    if CheckItemCanBePickedUpByNPC(ent, other) != QFALSE {
        if !(*other).NPC.is_null()
            && !(*(*other).NPC).goalEntity.is_null()
            && (*(*(*other).NPC).goalEntity).enemy == ent
        {
            //they were running to pick me up, they did, so clear goal
            (*(*other).NPC).goalEntity = core::ptr::null_mut();
            (*(*other).NPC).squadState = SQUAD_STAND_AND_SHOOT;
        }
    } else if (*ent).spawnflags & ITMSF_ALLOWNPC == 0 {
        // NPCs cannot pick it up
        if (*ent).s.eType == ET_NPC {
            // Not the player?
            let mut dontGo: qboolean = QFALSE;
            if (*(*ent).item).giType == IT_AMMO
                && (*(*ent).item).giTag == -1
                && (*other).s.NPC_class == CLASS_VEHICLE
                && !(*other).m_pVehicle.is_null()
                && (*(*(*other).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
            {
                //yeah, uh, atst gets healed by these things
                if (*other).maxHealth != 0 && (*other).health < (*other).maxHealth {
                    (*other).health += 80;
                    if (*other).health > (*other).maxHealth {
                        (*other).health = (*other).maxHealth;
                    }
                    G_ScaleNetHealth(other);
                    dontGo = QTRUE;
                }
            }

            if dontGo == QFALSE {
                return;
            }
        }
    }

    G_LogPrintf(&format!(
        "Item: {} {}\n",
        (*other).s.number,
        Sz((*(*ent).item).classname)
    ));

    predict = (*(*other).client).pers.predictItemPickup;

    // call the item-specific pickup function
    match (*(*ent).item).giType {
        IT_WEAPON => {
            respawn = Pickup_Weapon(ent, other);
            //		predict = qfalse;
            predict = QTRUE;
        }
        IT_AMMO => {
            respawn = Pickup_Ammo(ent, other);
            if (*(*ent).item).giTag == AMMO_THERMAL
                || (*(*ent).item).giTag == AMMO_TRIPMINE
                || (*(*ent).item).giTag == AMMO_DETPACK
            {
                let weapForAmmo: c_int;

                if (*(*ent).item).giTag == AMMO_THERMAL {
                    weapForAmmo = WP_THERMAL;
                } else if (*(*ent).item).giTag == AMMO_TRIPMINE {
                    weapForAmmo = WP_TRIP_MINE;
                } else {
                    weapForAmmo = WP_DET_PACK;
                }

                if !other.is_null()
                    && !(*other).client.is_null()
                    && (*(*other).client).ps.ammo
                        [weaponData[weapForAmmo as usize].ammoIndex as usize]
                        > 0
                {
                    (*(*other).client).ps.stats[STAT_WEAPONS as usize] |= 1 << weapForAmmo;
                }
            }
            //		predict = qfalse;
            predict = QTRUE;
        }
        IT_ARMOR => {
            respawn = Pickup_Armor(ent, other);
            //		predict = qfalse;
            predict = QTRUE;
        }
        IT_HEALTH => {
            respawn = Pickup_Health(ent, other);
            //		predict = qfalse;
            predict = QTRUE;
        }
        IT_POWERUP => {
            respawn = Pickup_Powerup(ent, other);
            predict = QFALSE;
            //		predict = qtrue;
        }
        IT_TEAM => {
            respawn = Pickup_Team(ent, other);
        }
        IT_HOLDABLE => {
            respawn = Pickup_Holdable(ent, other);
        }
        _ => {
            return;
        }
    }

    if respawn == 0 {
        return;
    }

    // play the normal pickup sound
    if predict != QFALSE {
        if !(*other).client.is_null() {
            BG_AddPredictableEventToPlayerstate(
                EV_ITEM_PICKUP,
                (*ent).s.number,
                &mut (*(*other).client).ps,
            );
        } else {
            G_AddPredictableEvent(other, EV_ITEM_PICKUP, (*ent).s.number);
        }
    } else {
        G_AddEvent(other, EV_ITEM_PICKUP, (*ent).s.number);
    }

    // powerup pickups are global broadcasts
    if /*ent->item->giType == IT_POWERUP ||*/ (*(*ent).item).giType == IT_TEAM {
        // if we want the global sound to play
        if (*ent).speed == 0.0 {
            let te: *mut gentity_t = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_ITEM_PICKUP);
            (*te).s.eventParm = (*ent).s.modelindex;
            (*te).r.svFlags |= SVF_BROADCAST;
        } else {
            let te: *mut gentity_t = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_ITEM_PICKUP);
            (*te).s.eventParm = (*ent).s.modelindex;
            // only send this temp entity to a single client
            (*te).r.svFlags |= SVF_SINGLECLIENT;
            (*te).r.singleClient = (*other).s.number;
        }
    }

    // fire item targets
    G_UseTargets(ent, other);

    // wait of -1 will not respawn
    if (*ent).wait == -1.0 {
        (*ent).r.svFlags |= SVF_NOCLIENT;
        (*ent).s.eFlags |= EF_NODRAW;
        (*ent).r.contents = 0;
        (*ent).unlinkAfterEvent = QTRUE;
        return;
    }

    let mut respawn = respawn;

    // non zero wait overrides respawn time
    if (*ent).wait != 0.0 {
        respawn = (*ent).wait as c_int;
    }

    // random can be used to vary the respawn time
    if (*ent).random != 0.0 {
        // C: `respawn += crandom() * ent->random` — crandom() is a double, so the multiply
        // (and the int += promotion) evaluate in double before truncating back to int.
        respawn = (respawn as f64 + crandom() * (*ent).random as f64) as c_int;
        if respawn < 1 {
            respawn = 1;
        }
    }

    // dropped items will not respawn
    if (*ent).flags & FL_DROPPED_ITEM != 0 {
        (*ent).freeAfterEvent = QTRUE;
    }

    // picked up items still stay around, they just don't
    // draw anything.  This allows respawnable items
    // to be placed on movers.
    if (*ent).flags & FL_DROPPED_ITEM == 0
        && ((*(*ent).item).giType == IT_WEAPON || (*(*ent).item).giType == IT_POWERUP)
    {
        (*ent).s.eFlags |= EF_ITEMPLACEHOLDER;
        (*ent).s.eFlags &= !EF_NODRAW;
    } else {
        (*ent).s.eFlags |= EF_NODRAW;
        (*ent).r.svFlags |= SVF_NOCLIENT;
    }
    (*ent).r.contents = 0;

    if (*ent).genericValue9 != 0 {
        //dropped item, should be removed when picked up
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    // ZOID
    // A negative respawn times means to never respawn this item (but don't
    // delete it).  This is used by items that are respawned by third party
    // events such as ctf flags
    if respawn <= 0 {
        (*ent).nextthink = 0;
        (*ent).think = None;
    } else {
        (*ent).nextthink = (*addr_of!(level)).time + respawn * 1000;
        (*ent).think = Some(RespawnItem);
    }
    trap::LinkEntity(ent);
}

//======================================================================

// g_items.c:37-38 — `extern gentity_t *droppedRedFlag; extern gentity_t *droppedBlueFlag;`
// The current-dropped CTF flag entities. The authentic definitions live in `ai_main.c`
// (the bot subsystem, not yet ported); they are hosted here — alongside their only writer,
// [`LaunchItem`] — until `ai_main` lands and reclaims them. Read by the bot flag-routing
// logic (`ai_main.c`) once that subsystem is ported.
pub static mut droppedRedFlag: *mut gentity_t = core::ptr::null_mut();
pub static mut droppedBlueFlag: *mut gentity_t = core::ptr::null_mut();

// `extern "C"` shim so the Rust-ABI [`Team_DroppedFlagThink`] (g_team.rs) can be installed
// in the C-ABI `gentity_t::think` function-pointer slot. (g_team's port keeps the fn as a
// plain `unsafe fn`; this thunk lives with its only think-slot caller, [`LaunchItem`], until
// g_team retypes it `extern "C"`.)
unsafe extern "C" fn Team_DroppedFlagThink_thunk(ent: *mut gentity_t) {
    Team_DroppedFlagThink(ent);
}

/*
================
LaunchItem

Spawns an item and tosses it forward
================
*/
/// Port of `g_items.c:2671` — `LaunchItem`. Spawns a fresh item entity for `item`, places
/// it at `origin` with a gravity trajectory along `velocity`, installs [`Touch_Item`] and a
/// 30-second auto-remove think (or the CTF [`Team_DroppedFlagThink`] for a dropped flag), and
/// links it. The dropped-flag branch also caches the entity in [`droppedRedFlag`] /
/// [`droppedBlueFlag`] (matched by classname) for the bot flag-routing logic.
///
/// No oracle: entity-state — allocates a `gentity_t` and mutates its trajectory/flag graph
/// plus the dropped-flag process globals, ending in `trap_LinkEntity`.
///
/// # Safety
/// `item` must point into the live [`bg_itemlist`] table; `origin`/`velocity` are read-only.
pub unsafe fn LaunchItem(item: *mut gitem_t, origin: &vec3_t, velocity: &vec3_t) -> *mut gentity_t {
    let dropped: *mut gentity_t = G_Spawn();

    (*dropped).s.eType = ET_ITEM;
    // store item number in modelindex
    (*dropped).s.modelindex =
        item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as c_int;
    if (*dropped).s.modelindex < 0 {
        (*dropped).s.modelindex = 0;
    }
    (*dropped).s.modelindex2 = 1; // This is non-zero is it's a dropped item

    (*dropped).classname = (*item).classname;
    (*dropped).item = item;
    VectorSet(
        &mut (*dropped).r.mins,
        -(ITEM_RADIUS as f32),
        -(ITEM_RADIUS as f32),
        -(ITEM_RADIUS as f32),
    );
    VectorSet(
        &mut (*dropped).r.maxs,
        ITEM_RADIUS as f32,
        ITEM_RADIUS as f32,
        ITEM_RADIUS as f32,
    );

    (*dropped).r.contents = CONTENTS_TRIGGER;

    (*dropped).touch = Some(Touch_Item);

    G_SetOrigin(dropped, origin);
    (*dropped).s.pos.trType = TR_GRAVITY;
    (*dropped).s.pos.trTime = (*addr_of!(level)).time;
    VectorCopy(velocity, &mut (*dropped).s.pos.trDelta);

    (*dropped).flags |= FL_BOUNCE_HALF;
    if ((*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY)
        && (*item).giType == IT_TEAM
    {
        // Special case for CTF flags
        (*dropped).think = Some(Team_DroppedFlagThink_thunk);
        (*dropped).nextthink = (*addr_of!(level)).time + 30000;
        Team_CheckDroppedItem(dropped);

        //rww - so bots know
        if strcmp((*dropped).classname, c"team_CTF_redflag".as_ptr()) == 0 {
            droppedRedFlag = dropped;
        } else if strcmp((*dropped).classname, c"team_CTF_blueflag".as_ptr()) == 0 {
            droppedBlueFlag = dropped;
        }
    } else {
        // auto-remove after 30 seconds
        (*dropped).think = Some(G_FreeEntity);
        (*dropped).nextthink = (*addr_of!(level)).time + 30000;
    }

    (*dropped).flags = FL_DROPPED_ITEM;

    if (*item).giType == IT_WEAPON || (*item).giType == IT_POWERUP {
        (*dropped).s.eFlags |= EF_DROPPEDWEAPON;
    }

    vectoangles(velocity, &mut (*dropped).s.angles);
    (*dropped).s.angles[PITCH] = 0.0;

    if (*item).giTag == WP_TRIP_MINE || (*item).giTag == WP_DET_PACK {
        (*dropped).s.angles[PITCH] = -90.0;
    }

    if (*item).giTag != WP_BOWCASTER
        && (*item).giTag != WP_DET_PACK
        && (*item).giTag != WP_THERMAL
    {
        (*dropped).s.angles[ROLL] = -90.0;
    }

    (*dropped).physicsObject = QTRUE;

    trap::LinkEntity(dropped);

    dropped
}

/*
================
Drop_Item

Spawns an item and tosses it forward
================
*/
/// Port of `g_items.c:2755` — `Drop_Item`. Tosses `item` out in front of `ent`: builds a
/// forward (pitch-zeroed) launch velocity from `ent`'s yaw + `angle`, scaled to 150 with a
/// 200±50 upward component, then defers to [`LaunchItem`].
///
/// No oracle: entity-state — delegates to [`LaunchItem`] (which allocates + links a
/// `gentity_t`) and reads the random stream via `crandom`.
///
/// # Safety
/// `ent` must be a valid entity; `item` must point into the live [`bg_itemlist`] table.
pub unsafe fn Drop_Item(ent: *mut gentity_t, item: *mut gitem_t, angle: f32) -> *mut gentity_t {
    let mut velocity: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    VectorCopy(&(*ent).s.apos.trBase, &mut angles);
    angles[YAW] += angle;
    angles[PITCH] = 0.0; // always forward

    AngleVectors(&angles, Some(&mut velocity), None, None);
    // VectorScale aliases src==dst here; copy the input to satisfy the borrow checker.
    let velocity_in = velocity;
    VectorScale(&velocity_in, 150.0, &mut velocity);
    // C: `velocity[2] += 200 + crandom() * 50` — crandom() is `2.0 * (...)`, a double, so the
    // whole RHS evaluates in double before truncating back into the float component.
    velocity[2] = (velocity[2] as f64 + 200.0 + crandom() * 50.0) as f32;

    LaunchItem(item, &(*ent).s.pos.trBase, &velocity)
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;

    /// Serializes the tests that mutate the process-global `level` / `g_adaptRespawn`
    /// cvar so they don't race the parallel test runner. Re-exports the crate-wide shared
    /// lock (g_main) so it serializes cross-module, not just within this module.
    use crate::codemp::game::g_main::level_lock;

    /// `adjustRespawnTime` over a sweep of base times, item type/tag (covering the
    /// thermal/trip-mine/det-pack special case), the `g_adaptRespawn` toggle, and
    /// player counts straddling every scaling bucket (≤4, 5-12, 13-32, >32) plus the
    /// 1-second floor — checked bit-exact against the extracted C. Mutates the process
    /// globals it reads, under the lock.
    #[test]
    fn adjustrespawntime_matches_oracle() {
        use core::ptr::addr_of_mut;

        let _g = level_lock();

        let base_times = [20.0f32, 30.0, 40.0, 60.0, 120.0, 0.5, 3.7];
        // (itemType, itemTag) pairs: non-weapon, weapon-with-special-tag, weapon-other.
        let items: [(itemType_t, weapon_t); 6] = [
            (0, 0),            // IT_BAD / no tag (non-weapon path)
            (IT_WEAPON, WP_THERMAL),
            (IT_WEAPON, WP_TRIP_MINE),
            (IT_WEAPON, WP_DET_PACK),
            (IT_WEAPON, 5),    // a normal weapon — no special case
            (3, 12),           // non-weapon type that happens to share a tag value
        ];
        let adapt_flags = [0i32, 1];
        let player_counts = [0i32, 4, 5, 8, 12, 13, 20, 32, 33, 64, 200];

        for &pre in &base_times {
            for &(itype, itag) in &items {
                for &adapt in &adapt_flags {
                    for &npc in &player_counts {
                        unsafe {
                            (*addr_of_mut!(g_adaptRespawn)).integer = adapt;
                            (*addr_of_mut!(level)).numPlayingClients = npc;
                        }

                        let got = adjustRespawnTime(pre, itype, itag);
                        let want = unsafe {
                            oracle::jka_adjustRespawnTime(pre, itype, itag, adapt, npc)
                        };
                        assert_eq!(
                            got, want,
                            "pre={pre} itype={itype} itag={itag} adapt={adapt} npc={npc}"
                        );
                    }
                }
            }
        }
    }
}
