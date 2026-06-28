//! `g_utils.c` — game-side utility helpers.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_utils.c`, incrementally as consumers
//! need it (the same lazy strategy used for the `trap_*` surface). This first
//! slice is the **configstring index family**: `G_FindConfigstringIndex` and its
//! thin `G_*Index` wrappers, pulled in by `G_InitGame`'s `G_SoundIndex` calls.
//!
//! These bridge to the engine through the already-ported `trap_GetConfigstring`/
//! `trap_SetConfigstring` wrappers, so — like the trap layer itself — they take
//! Rust `&str` rather than raw `const char *` and have no C oracle (their
//! behaviour is engine-call control flow, not a computable data table). See
//! DEVIATIONS.md.
//!
//! **This is a lazy / on-demand partial port — NOT a top-to-bottom slice.** Ported so
//! far: the shader-remap subsystem (`AddRemap` / `BuildShaderStateConfig`), the
//! configstring-index family, the entity finder `G_Find`, the target pick/use family
//! (`G_PickTarget` / `GlobalUse` / `G_UseTargets2` / `G_UseTargets`), the pure
//! vector/bounds helpers (`G_SetMovedir` / `G_PointInBounds` / `G_BoxInBounds`), the
//! entity-allocator core (`G_InitGentity` / `G_Spawn` /
//! `G_EntitiesFree` plus `G_Spawn`'s static `G_SpewEntList` diagnostic), the
//! position/angle setters (`G_SetOrigin` / `G_SetAngles`), `G_TempEntity`, the
//! event/sound emitters (`G_AddEvent` / `G_SoundIndex`), and `G_FreeEntity` with its
//! free-path helpers (`G_KillG2Queue` / `G_FreeVehicleObject` / `G_FreeFakeClient`). The
//! rest of g_utils.c (the `G_Sound` emitter / link/trace setters / `TryUse` / `G_KillBox`
//! / … plus the vehicle-pool allocator
//! `G_AllocateVehicleObject`) is unported and lands as its consumers do —
//! several funnel into not-yet-ported Ghoul2/vehicle/NPC/ICARUS layers.
//! (`G_BSPIndex` — commented-out in C — is reconstructed here for the live `SP_misc_bsp` call.)

#![allow(non_snake_case)] // C function names (`G_SoundIndex`, ...) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`remapCount`, ...) kept verbatim
#![allow(non_camel_case_types)] // C type names (`shaderRemap_t`) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use core::sync::atomic::{AtomicUsize, Ordering};

extern "C" {
    /// libc `char *strcpy( char *dest, const char *src )` — the unbounded copy
    /// `AddRemap` uses into its fixed `MAX_QPATH` buffers, verbatim from the C source.
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

use crate::codemp::game::anims::{BOTH_BUTTON_HOLD, BOTH_CONSOLE1};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_misc::{BG_AddPredictableEventToPlayerstate, BG_Alloc};
use crate::codemp::game::bg_panimate::{bgAllAnims, BG_SetAnim};
use crate::codemp::game::bg_public::team_t;
use crate::codemp::game::bg_public::{
    bgEntity_t, CS_AMBIENT_SET, CS_BSP_MODELS, CS_EFFECTS, CS_G2BONES, CS_ICONS, CS_MODELS,
    CS_SHADERSTATE, CS_SOUNDS, DEFAULT_MAXS_2, DEFAULT_MINS_2, EF_SOUNDTRACKER, ET_EVENTS,
    ET_MISSILE, ET_NPC, EV_ENTITY_SOUND, EV_EVENT_BIT1, EV_EVENT_BITS, EV_GENERAL_SOUND,
    EV_MUTE_SOUND, EV_PLAY_EFFECT, EV_PLAY_EFFECT_ID, EV_SCREENSHAKE, EV_USE_ITEM0, GT_TEAM,
    HANDEXTEND_DRAGGING, HANDEXTEND_NONE, HI_AMMODISP, HI_JETPACK, MASK_OPAQUE, MAX_SUB_BSP,
    MOD_TELEFRAG, PMF_FOLLOW, PMF_TIME_KNOCKBACK, STAT_HOLDABLE_ITEMS, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_public::{
    GT_SIEGE, HI_HEALTHDISP, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO, STAT_HEALTH,
    STAT_MAX_HEALTH,
};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::siegeClass_t;
use crate::codemp::game::bg_vehicles_h::Vehicle_t;
use crate::codemp::game::bg_weapons::{ammoData, weaponData};
use crate::codemp::game::bg_weapons_h::{LAST_USEABLE_WEAPON, WP_NONE};
use crate::codemp::game::g_combat::{gSiegeRoundBegun, G_Damage};
use crate::codemp::game::g_items::{ItemUse_Jetpack, ItemUse_UseDisp};
use crate::codemp::game::g_local::{
    gclient_t, gentity_s, gentity_t, CON_CONNECTED, DAMAGE_NO_PROTECTION, FL_INACTIVE,
};
use crate::codemp::game::g_main::{
    g_entities, g_ff_objectives, g_gametype, g_gravity, g_knockback, level, Com_Error, Com_Printf,
    G_Error, G_Printf,
};
use crate::codemp::game::g_mover::Touch_Button;
use crate::codemp::game::g_public_h::{Q3_INFINITE, SVF_BROADCAST, SVF_PLAYER_USABLE};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleVectors, CrossProduct, Distance, DotProduct,
    G_FindClosestPointOnLineSegment, VectorAdd, VectorClear, VectorCompare, VectorCopy,
    VectorLength, VectorMA, VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{va, Com_sprintf, Q_strcat, Q_stricmp, Sz};
use crate::codemp::game::q_shared_h::{
    snap_vector, trace_t, usercmd_t, vec3_t, vec_t, BUTTON_USE, CHAN_AUTO, CHAN_VOICE,
    ENTITYNUM_MAX_NORMAL, ENTITYNUM_NONE, ERR_DROP, MAX_AMBIENT_SETS, MAX_CLIENTS, MAX_FX,
    MAX_G2BONES, MAX_GENTITIES, MAX_ICONS, MAX_MODELS, MAX_QPATH, MAX_SABERS, MAX_SOUNDS,
    MAX_STRING_CHARS, NUM_TRACK_CHANNELS, SOLID_BMODEL, TRACK_CHANNEL_NONE, TR_LINEAR_STOP,
    TR_NONLINEAR_STOP, TR_STATIONARY, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_CORPSE, CONTENTS_ITEM, CONTENTS_SOLID,
};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `typedef struct { char oldShader[MAX_QPATH]; char newShader[MAX_QPATH];
/// float timeOffset; } shaderRemap_t` (g_utils.c:9) — one server-driven shader
/// remap. File-local in C; private here.
#[derive(Clone, Copy)]
#[repr(C)]
struct shaderRemap_t {
    oldShader: [c_char; MAX_QPATH],
    newShader: [c_char; MAX_QPATH],
    timeOffset: f32,
}

const SHADER_REMAP_ZERO: shaderRemap_t = shaderRemap_t {
    oldShader: [0; MAX_QPATH],
    newShader: [0; MAX_QPATH],
    timeOffset: 0.0,
};

const MAX_SHADER_REMAPS: usize = 128;

/// `int remapCount` / `shaderRemap_t remappedShaders[MAX_SHADER_REMAPS]` (g_utils.c:17).
/// External-linkage globals in C, but only `AddRemap`/`BuildShaderStateConfig` (also
/// in this file) ever touch them, so kept module-private.
static mut remapCount: c_int = 0;
static mut remappedShaders: [shaderRemap_t; MAX_SHADER_REMAPS] =
    [SHADER_REMAP_ZERO; MAX_SHADER_REMAPS];

/// `void AddRemap( const char *oldShader, const char *newShader, float timeOffset )`
/// (g_utils.c:20).
///
/// Records (or updates) a `oldShader -> newShader` render remap with a `timeOffset`.
/// Updates the existing entry if `oldShader` already maps; otherwise appends, capped
/// at `MAX_SHADER_REMAPS` (silently dropped past the cap, as in C). The `strcpy`s into
/// the fixed `MAX_QPATH` buffers are carried verbatim (unbounded, like the original).
pub unsafe fn AddRemap(oldShader: *const c_char, newShader: *const c_char, timeOffset: f32) {
    let table = addr_of_mut!(remappedShaders) as *mut shaderRemap_t;

    let mut i: c_int = 0;
    while i < *addr_of!(remapCount) {
        let entry = table.add(i as usize);
        if Q_stricmp(oldShader, (*entry).oldShader.as_ptr()) == 0 {
            // found it, just update this one
            strcpy((*entry).newShader.as_mut_ptr(), newShader);
            (*entry).timeOffset = timeOffset;
            return;
        }
        i += 1;
    }
    if *addr_of!(remapCount) < MAX_SHADER_REMAPS as c_int {
        let entry = table.add(*addr_of!(remapCount) as usize);
        strcpy((*entry).newShader.as_mut_ptr(), newShader);
        strcpy((*entry).oldShader.as_mut_ptr(), oldShader);
        (*entry).timeOffset = timeOffset;
        *addr_of_mut!(remapCount) += 1;
    }
}

/// `const char *BuildShaderStateConfig( void )` (g_utils.c:39).
///
/// Serialises the remap table into one `old=new:offset@`-delimited configstring,
/// returning a pointer to the static `buff`. Carried verbatim including the
/// `memset(buff, 0, MAX_STRING_CHARS)` quirk — it zeroes only the first quarter of the
/// `MAX_STRING_CHARS*4` buffer, which is harmless because the following `Q_strcat`
/// restarts from `strlen(buff) == 0`.
pub unsafe fn BuildShaderStateConfig() -> *const c_char {
    static mut buff: [c_char; MAX_STRING_CHARS * 4] = [0; MAX_STRING_CHARS * 4];
    // char out[(MAX_QPATH * 2) + 5];
    let mut out: [c_char; (MAX_QPATH * 2) + 5] = [0; (MAX_QPATH * 2) + 5];

    let buffp = addr_of_mut!(buff) as *mut c_char;
    let table = addr_of!(remappedShaders) as *const shaderRemap_t;

    // memset(buff, 0, MAX_STRING_CHARS) -- only the first quarter, as in C
    core::ptr::write_bytes(buffp, 0, MAX_STRING_CHARS);
    let mut i: c_int = 0;
    while i < *addr_of!(remapCount) {
        let entry = table.add(i as usize);
        Com_sprintf(
            out.as_mut_ptr(),
            ((MAX_QPATH * 2) + 5) as c_int,
            format_args!(
                "{}={}:{:5.2}@",
                Sz((*entry).oldShader.as_ptr()),
                Sz((*entry).newShader.as_ptr()),
                (*entry).timeOffset
            ),
        );
        Q_strcat(buffp, (MAX_STRING_CHARS * 4) as c_int, out.as_ptr());
        i += 1;
    }
    buffp
}

/// `static int G_FindConfigstringIndex( const char *name, int start, int max,
/// qboolean create )` — find `name` in the `[start, start+max)` configstring
/// block, returning its 1-based slot; when `create`, register it in the first
/// free slot (erroring on overflow).
///
/// File-local (`static`) in C → private here. The C reads each configstring into
/// a `char s[MAX_STRING_CHARS]` scratch buffer and `strcmp`s it; the trap wrapper
/// returns an owned `String`, so the buffer is gone and the compare is `==`. The
/// `!name` (NULL `char *`) guard is subsumed by `&str` never being null.
fn G_FindConfigstringIndex(name: &str, start: c_int, max: c_int, create: qboolean) -> c_int {
    if name.is_empty() {
        return 0;
    }

    let mut i: c_int = 1;
    while i < max {
        let s = trap::GetConfigstring(start + i);
        if s.is_empty() {
            break;
        }
        if s == name {
            return i;
        }
        i += 1;
    }

    if create == QFALSE {
        return 0;
    }

    if i == max {
        G_Error("G_FindConfigstringIndex: overflow");
    }

    trap::SetConfigstring(start + i, name);

    i
}

/*
Ghoul2 Insert Start
*/

/// `int G_BoneIndex( const char *name )`
pub fn G_BoneIndex(name: &str) -> c_int {
    G_FindConfigstringIndex(name, CS_G2BONES, MAX_G2BONES as c_int, QTRUE)
}
/*
Ghoul2 Insert End
*/

/// `int G_ModelIndex( const char *name )`
///
/// The `#ifdef _DEBUG_MODEL_PATH_ON_SERVER` existence-check block (which opens the
/// model file via `trap_FS_FOpenFile` to warn about missing models) is omitted —
/// it is a debug-only diagnostic, undefined in the retail build (see DEVIATIONS.md
/// "Build configuration").
pub fn G_ModelIndex(name: &str) -> c_int {
    G_FindConfigstringIndex(name, CS_MODELS, MAX_MODELS as c_int, QTRUE)
}

/// `int G_IconIndex( const char* name )`
pub fn G_IconIndex(name: &str) -> c_int {
    // C: assert(name && name[0]) — compiled out in the retail build (NDEBUG).
    G_FindConfigstringIndex(name, CS_ICONS, MAX_ICONS as c_int, QTRUE)
}

/// `int G_SoundIndex( const char *name )`
pub fn G_SoundIndex(name: &str) -> c_int {
    // C: assert(name && name[0]) — compiled out in the retail build (NDEBUG).
    G_FindConfigstringIndex(name, CS_SOUNDS, MAX_SOUNDS as c_int, QTRUE)
}

/// `int G_SoundSetIndex( const char *name )`
pub fn G_SoundSetIndex(name: &str) -> c_int {
    G_FindConfigstringIndex(name, CS_AMBIENT_SET, MAX_AMBIENT_SETS as c_int, QTRUE)
}

/// `int G_EffectIndex( const char *name )`
pub fn G_EffectIndex(name: &str) -> c_int {
    G_FindConfigstringIndex(name, CS_EFFECTS, MAX_FX as c_int, QTRUE)
}

/// `int G_BSPIndex( const char *name )` (g_utils.c:154).
///
/// Interns a sub-BSP model `name` into the `CS_BSP_MODELS` configstring range and returns
/// its index (the `G_ModelIndex`/`G_SoundIndex` sibling for the `misc_bsp` instancing path).
/// **Reconstructed:** the C definition is commented-out at g_utils.c:153-158 (its `MAX_SUB_BSP`
/// was likewise commented in q_shared.h), yet it is *called* live by `SP_misc_bsp`
/// (g_misc.c:419) — so it is ported faithfully from the commented body.
pub fn G_BSPIndex(name: &str) -> c_int {
    G_FindConfigstringIndex(name, CS_BSP_MODELS, MAX_SUB_BSP, QTRUE)
}

/// `void G_TeamCommand( team_t team, char *cmd )` (g_utils.c:199).
///
/// Reliably sends the server command `cmd` to every `CON_CONNECTED` client whose
/// `sess.sessionTeam` matches `team`. The C wraps the text in `va("%s", cmd)` — a
/// redundant format-buffer copy that yields `cmd` verbatim; here it collapses into the
/// `&str` the trap layer already marshals back into a C string. No oracle (walks the
/// global `level.clients` array and calls `trap_SendServerCommand`).
pub unsafe fn G_TeamCommand(team: team_t, cmd: *const c_char) {
    let lvl = addr_of!(level);
    let mut i: c_int = 0;
    while i < (*lvl).maxclients {
        let client = (*lvl).clients.offset(i as isize);
        if (*client).pers.connected == CON_CONNECTED && (*client).sess.sessionTeam == team {
            trap::SendServerCommand(i, &CStr::from_ptr(cmd).to_string_lossy());
        }
        i += 1;
    }
}

/// `gentity_t *G_Find( gentity_t *from, int fieldofs, const char *match )`
/// (g_utils.c:224).
///
/// Scans active entities for the next one whose `char *` field at byte offset
/// `fieldofs` case-insensitively equals `match`. `fieldofs` is the C `FOFS(field)`
/// (`offsetof`) — callers pass `core::mem::offset_of!(gentity_s, classname)` etc.
/// `from == NULL` starts at `g_entities[0]`; otherwise the search resumes at the
/// entity *after* `from`. Returns null at the end of the list.
///
/// Faithful raw-pointer port: the C reads the field with `*(char **)((byte *)from +
/// fieldofs)`, reproduced here verbatim. The trailing `from++` of the `for`-loop runs
/// even on the `continue` paths (not-in-use / null field), so the inner skips are a
/// labelled `break` that falls through to the shared increment.
pub unsafe fn G_Find(
    mut from: *mut gentity_t,
    fieldofs: usize,
    match_: *const c_char,
) -> *mut gentity_t {
    if from.is_null() {
        from = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    } else {
        from = from.add(1);
    }

    let end = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*addr_of!(level)).num_entities as usize);
    while from < end {
        'cont: {
            if (*from).inuse == QFALSE {
                break 'cont;
            }
            let s = *((from as *const u8).add(fieldofs) as *const *const c_char);
            if s.is_null() {
                break 'cont;
            }
            if Q_stricmp(s, match_) == 0 {
                return from;
            }
        }
        from = from.add(1);
    }

    null_mut()
}

/// `int G_RadiusList( vec3_t origin, float radius, gentity_t *ignore, qboolean
/// takeDamage, gentity_t *ent_list[MAX_GENTITIES] )` (g_utils.c:254).
///
/// Fills `ent_list` with every in-use entity within `radius` of `origin` whose
/// `takedamage` flag equals `takeDamage`, skipping `ignore`. Distance is measured from
/// the *edge* of each entity's bounding box (`r.absmin`/`r.absmax`), so an entity
/// straddling `origin` on an axis contributes 0 on that axis. `radius` is clamped up to
/// 1 first. Returns the count placed in `ent_list`. No oracle (queries the engine via
/// `trap_EntitiesInBox` and walks the global `g_entities` array).
pub unsafe fn G_RadiusList(
    origin: &vec3_t,
    mut radius: f32,
    ignore: *const gentity_t,
    takeDamage: qboolean,
    ent_list: &mut [*mut gentity_t; MAX_GENTITIES],
) -> c_int {
    let mut entityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut ent_count: c_int = 0;

    if radius < 1.0 {
        radius = 1.0;
    }

    for i in 0..3 {
        mins[i] = origin[i] - radius;
        maxs[i] = origin[i] + radius;
    }

    let numListedEntities = trap::EntitiesInBox(&mins, &maxs, &mut entityList);

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    for e in 0..numListedEntities as usize {
        let ent = base.add(entityList[e] as usize);

        if ent as *const gentity_t == ignore
            || (*ent).inuse == QFALSE
            || (*ent).takedamage != takeDamage
        {
            continue;
        }

        // find the distance from the edge of the bounding box
        let mut v: vec3_t = [0.0; 3];
        for i in 0..3 {
            if origin[i] < (*ent).r.absmin[i] {
                v[i] = (*ent).r.absmin[i] - origin[i];
            } else if origin[i] > (*ent).r.absmax[i] {
                v[i] = origin[i] - (*ent).r.absmax[i];
            } else {
                v[i] = 0.0;
            }
        }

        let dist = VectorLength(&v);
        if dist >= radius {
            continue;
        }

        // ok, we are within the radius, add us to the incoming list
        ent_list[ent_count as usize] = ent;
        ent_count += 1;
    }

    // we are done, return how many we found
    ent_count
}

/// `void G_Throw( gentity_t *targ, vec3_t newDir, float push )` (g_utils.c:317).
///
/// Apply a knockback impulse `push` along `newDir` to `targ`, scaled by `g_knockback`
/// and `targ`'s effective mass (its `physicsBounce` if positive, else a default 200). With
/// gravity on (`g_gravity.value > 0`) the vertical component gets the larger `1.5` multiplier
/// and the horizontal an `0.8` one; with gravity off it is a flat scale. The impulse adds into
/// the client playerstate velocity (clients) or the trajectory `trDelta` (non-stopping movers),
/// restamping `trBase`/`trTime`. For clients with no live `pm_time` it also arms a
/// 50..=200 ms `PMF_TIME_KNOCKBACK` timer so the recipient can't instantly cancel the move.
///
/// No oracle: mutates a `gentity_t`/`gclient_t` and reads the module-global `g_gravity`/
/// `g_knockback` cvars and `level.time`. The scale expressions are inlined (not routed through
/// [`VectorScale`]) to preserve the C macro's `double` intermediate (`... * 0.8`/`* 1.5`).
///
/// # Safety
/// `targ` must be a valid, live `gentity_t`; its `client` pointer is read only when non-null.
pub unsafe fn G_Throw(targ: *mut gentity_t, newDir: &vec3_t, push: f32) {
    let mut kvel: vec3_t = [0.0; 3];
    let mass: f32;

    if (*targ).physicsBounce > 0.0 {
        // overide the mass
        mass = (*targ).physicsBounce;
    } else {
        mass = 200.0;
    }

    if (*addr_of!(g_gravity)).value > 0.0 {
        // VectorScale( newDir, g_knockback.value * (float)push / mass * 0.8, kvel );
        // The C macro substitutes the scale expression inline; the trailing `* 0.8`
        // promotes it to `double`, so `newDir[i] * scale` evaluates in double precision
        // before truncating into the float `kvel[i]`. Mirror that exactly.
        let scale: f64 = ((*addr_of!(g_knockback)).value * push / mass) as f64 * 0.8_f64;
        kvel[0] = (newDir[0] as f64 * scale) as f32;
        kvel[1] = (newDir[1] as f64 * scale) as f32;
        kvel[2] = (newDir[2] as f64 * scale) as f32;
        // kvel[2] = newDir[2] * g_knockback.value * (float)push / mass * 1.5;
        // C evaluates left-to-right: the three float multiplies/divide each round to
        // `float`, then the trailing `* 1.5` promotes to `double`. Reproduce per-op.
        let kb = (*addr_of!(g_knockback)).value;
        let z_f: f32 = newDir[2] * kb * push / mass;
        kvel[2] = (z_f as f64 * 1.5_f64) as f32;
    } else {
        // VectorScale( newDir, g_knockback.value * (float)push / mass, kvel );
        let scale: f32 = (*addr_of!(g_knockback)).value * push / mass;
        kvel[0] = newDir[0] * scale;
        kvel[1] = newDir[1] * scale;
        kvel[2] = newDir[2] * scale;
    }

    if !(*targ).client.is_null() {
        let vel = addr_of_mut!((*(*targ).client).ps.velocity);
        VectorAdd(&*vel, &kvel, &mut *vel);
    } else if (*targ).s.pos.trType != TR_STATIONARY
        && (*targ).s.pos.trType != TR_LINEAR_STOP
        && (*targ).s.pos.trType != TR_NONLINEAR_STOP
    {
        let delta = addr_of_mut!((*targ).s.pos.trDelta);
        VectorAdd(&*delta, &kvel, &mut *delta);
        VectorCopy(&(*targ).r.currentOrigin, &mut (*targ).s.pos.trBase);
        (*targ).s.pos.trTime = (*addr_of!(level)).time;
    }

    // set the timer so that the other client can't cancel
    // out the movement immediately
    if !(*targ).client.is_null() && (*(*targ).client).ps.pm_time == 0 {
        let mut t: c_int = (push * 2.0) as c_int;

        if t < 50 {
            t = 50;
        }
        if t > 200 {
            t = 200;
        }
        (*(*targ).client).ps.pm_time = t;
        (*(*targ).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
    }
}

/// `void G_SetAnim( gentity_t *ent, usercmd_t *ucmd, int setAnimParts, int anim,
/// int setAnimFlags, int blendTime )` (g_utils.c:488).
///
/// Drives an animation outside of pmove by handing `ent`'s playerstate and its skeleton's
/// anim table (`bgAllAnims[ent->localAnimIndex]`) to [`BG_SetAnim`]. The `ucmd` parameter
/// is dead in the retail build — only the disabled `#if 0` legacy pmove path read it — so
/// it is kept for signature parity and ignored. No oracle (mutates a `gclient_t`).
pub unsafe fn G_SetAnim(
    ent: *mut gentity_t,
    _ucmd: *mut usercmd_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    blendTime: c_int,
) {
    debug_assert!(!(*ent).client.is_null());
    BG_SetAnim(
        addr_of_mut!((*(*ent).client).ps),
        (*addr_of!(bgAllAnims))[(*ent).localAnimIndex as usize].anims,
        setAnimParts,
        anim,
        setAnimFlags,
        blendTime,
    );
}

/// `gentity_t *G_PickTarget( char *targetname )` (g_utils.c:530).
///
/// Collects up to `MAXCHOICES` (32) active entities whose `targetname` matches and
/// returns one at random — `rand()` is the game's own LCG via [`rand`], not libc.
/// Warns through `G_Printf` and returns null on a NULL `targetname` or no match.
pub unsafe fn G_PickTarget(targetname: *mut c_char) -> *mut gentity_t {
    const MAXCHOICES: usize = 32;

    let mut ent: *mut gentity_t = null_mut();
    let mut num_choices: c_int = 0;
    let mut choice: [*mut gentity_t; MAXCHOICES] = [null_mut(); MAXCHOICES];

    if targetname.is_null() {
        G_Printf("G_PickTarget called with NULL targetname\n");
        return null_mut();
    }

    loop {
        ent = G_Find(ent, offset_of!(gentity_s, targetname), targetname);
        if ent.is_null() {
            break;
        }
        choice[num_choices as usize] = ent;
        num_choices += 1;
        if num_choices as usize == MAXCHOICES {
            break;
        }
    }

    if num_choices == 0 {
        G_Printf(&format!(
            "G_PickTarget: target {} not found\n",
            Sz(targetname)
        ));
        return null_mut();
    }

    choice[(rand() % num_choices) as usize]
}

/// `void GlobalUse( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_utils.c:561) — fire `self`'s `use` callback, skipping if `self` is null, flagged
/// `FL_INACTIVE`, or has no `use` handler.
pub unsafe fn GlobalUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if self_.is_null() || ((*self_).flags & FL_INACTIVE) != 0 {
        return;
    }

    match (*self_).r#use {
        None => (),
        Some(use_fn) => use_fn(self_, other, activator),
    }
}

/// `void G_UseTargets2( gentity_t *ent, gentity_t *activator, const char *string )`
/// (g_utils.c:575).
///
/// Applies `ent`'s optional shader remap (pushing the rebuilt shader-state configstring),
/// then fires the `use` of every entity whose `targetname` equals `string`. Self-targeting
/// warns instead of recursing, and the loop bails out if `ent` is freed mid-iteration.
pub unsafe fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, string: *const c_char) {
    if ent.is_null() {
        return;
    }

    if !(*ent).targetShaderName.is_null() && !(*ent).targetShaderNewName.is_null() {
        let f = ((*addr_of!(level)).time as f64 * 0.001) as f32;
        AddRemap((*ent).targetShaderName, (*ent).targetShaderNewName, f);
        let cfg = BuildShaderStateConfig();
        trap::SetConfigstring(
            CS_SHADERSTATE,
            CStr::from_ptr(cfg).to_str().unwrap_or_default(),
        );
    }

    if string.is_null() || *string == 0 {
        return;
    }

    let mut t: *mut gentity_t = null_mut();
    loop {
        t = G_Find(t, offset_of!(gentity_s, targetname), string);
        if t.is_null() {
            break;
        }
        if t == ent {
            G_Printf("WARNING: Entity used itself.\n");
        } else if (*t).r#use.is_some() {
            GlobalUse(t, ent, activator);
        }
        if (*ent).inuse == QFALSE {
            G_Printf("entity was removed while using targets\n");
            return;
        }
    }
}

/// `void G_UseTargets( gentity_t *ent, gentity_t *activator )` (g_utils.c:618) —
/// fire the `use` of everything matching `ent`'s own `target` string.
pub unsafe fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t) {
    if ent.is_null() {
        return;
    }
    G_UseTargets2(ent, activator, (*ent).target);
}

/// `float *tv( float x, float y, float z )` (g_utils.c:636) — "temp vector": pack three
/// floats into one of eight rotating `static vec3_t` so several temporaries can be passed to
/// one call without colliding. The returned pointer aliases shared static storage and is only
/// valid until `tv` has been called eight more times — faithful to the C. No oracle (rotating
/// static state; follows the [`crate::codemp::game::q_shared::va`] precedent).
///
/// # Safety
/// The returned pointer aliases a rotating static buffer (see above).
pub unsafe fn tv(x: vec_t, y: vec_t, z: vec_t) -> *mut vec_t {
    // static int index; static vec3_t vecs[8];
    static mut VECS: [vec3_t; 8] = [[0.0; 3]; 8];
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    // use an array so that multiple tempvectors won't collide for a while
    let idx = INDEX.fetch_add(1, Ordering::Relaxed) & 7;
    let v = (core::ptr::addr_of_mut!(VECS) as *mut vec3_t).add(idx) as *mut vec_t;

    *v.add(0) = x;
    *v.add(1) = y;
    *v.add(2) = z;

    v
}

/// `char *vtos( const vec3_t v )` (g_utils.c:671) — "vector to string": format `v` as
/// `(%i %i %i)` (truncated to ints) into one of eight rotating `static char[32]` for printing.
/// Same aliasing lifetime as [`tv`]. No oracle (rotating static state).
///
/// # Safety
/// The returned pointer aliases a rotating static buffer (see above).
pub unsafe fn vtos(v: &vec3_t) -> *mut c_char {
    // static int index; static char str[8][32];
    static mut STR: [[c_char; 32]; 8] = [[0; 32]; 8];
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    // use an array so that multiple vtos won't collide
    let idx = INDEX.fetch_add(1, Ordering::Relaxed) & 7;
    let s = (core::ptr::addr_of_mut!(STR) as *mut c_char).add(idx * 32);

    Com_sprintf(
        s,
        32,
        format_args!("({} {} {})", v[0] as c_int, v[1] as c_int, v[2] as c_int),
    );

    s
}

/// `void G_SetMovedir( vec3_t angles, vec3_t movedir )` (g_utils.c:687).
///
/// The editor specifies a single yaw value for a direction; the two sentinel angles
/// `{0,-1,0}` / `{0,-2,0}` mean straight up / straight down, otherwise the yaw is
/// turned into a forward vector via [`AngleVectors`]. `angles` is then cleared because
/// it is being repurposed as a direction, not an orientation. Mutates both arguments.
pub fn G_SetMovedir(angles: &mut vec3_t, movedir: &mut vec3_t) {
    const VEC_UP: vec3_t = [0.0, -1.0, 0.0];
    const MOVEDIR_UP: vec3_t = [0.0, 0.0, 1.0];
    const VEC_DOWN: vec3_t = [0.0, -2.0, 0.0];
    const MOVEDIR_DOWN: vec3_t = [0.0, 0.0, -1.0];

    if VectorCompare(angles, &VEC_UP) != 0 {
        VectorCopy(&MOVEDIR_UP, movedir);
    } else if VectorCompare(angles, &VEC_DOWN) != 0 {
        VectorCopy(&MOVEDIR_DOWN, movedir);
    } else {
        AngleVectors(angles, Some(movedir), None, None);
    }
    VectorClear(angles);
}

/// `void G_InitGentity( gentity_t *e )` (g_utils.c:703).
///
/// Brings a slot into use: mark it `inuse`, reset to the `"noclass"` classname, stamp
/// its entity number (its index in `g_entities`), clear the owner, and assume no Ghoul2
/// model, and free any ICARUS info on the slot. Called by `G_Spawn`
/// on every allocation.
pub unsafe fn G_InitGentity(e: *mut gentity_t) {
    (*e).inuse = QTRUE;
    (*e).classname = c"noclass".as_ptr() as *mut c_char;
    (*e).s.number = e.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
    (*e).r.ownerNum = ENTITYNUM_NONE;
    (*e).s.modelGhoul2 = 0; // assume not

    trap::ICARUS_FreeEnt(e); // ICARUS information must be added after this point
}

/// `static void G_SpewEntList( void )` (g_utils.c:714) — last-gasp diagnostic dumped
/// just before the fatal `G_Error` when the entity array is exhausted: every in-use
/// entity's number+classname (and a running tally of NPCs, projectiles, temp-entities,
/// and sound-tracker temp-entities) to the console. File-local (`static`) in C → private
/// here. No oracle (walks the global `g_entities` array, prints, and writes a file).
///
/// The `#ifndef VM_OR_FINAL_BUILD` `entspew.txt` file output is mirrored with
/// `#[cfg(not(feature = "vm"))]` (the crate's `vm` feature == C `Q3_VM`); there is no
/// `FINAL_BUILD` feature here. The C `va()`+`Com_Printf`+`strcpy(className,...)` is
/// rendered with Rust `format!`, printing the classname directly (equivalent output).
unsafe fn G_SpewEntList() {
    let mut num_npc: c_int = 0;
    let mut num_projectile: c_int = 0;
    let mut num_temp_ent: c_int = 0;
    let mut num_temp_ent_st: c_int = 0;

    #[cfg(not(feature = "vm"))]
    let fh = trap::FS_FOpenFile("entspew.txt", crate::codemp::game::q_shared_h::FS_WRITE).1;

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    let mut i: c_int = 0;
    while i < ENTITYNUM_MAX_NORMAL {
        let ent = base.add(i as usize);
        if (*ent).inuse != QFALSE {
            if (*ent).s.eType == ET_MISSILE {
                num_projectile += 1;
            } else if (*ent).s.eType == ET_NPC {
                num_npc += 1;
            } else if (*ent).freeAfterEvent != QFALSE {
                num_temp_ent += 1;
                if ((*ent).s.eFlags & EF_SOUNDTRACKER) != 0 {
                    num_temp_ent_st += 1;
                }

                let s = format!(
                    "TEMPENT {:4}: EV {}\n",
                    (*ent).s.number,
                    (*ent).s.eType - ET_EVENTS
                );
                Com_Printf(&s);
                #[cfg(not(feature = "vm"))]
                if fh != 0 {
                    trap::FS_Write(s.as_bytes(), fh);
                }
            }

            let s = if !(*ent).classname.is_null() && *(*ent).classname != 0 {
                format!(
                    "ENT {:4}: Classname {}\n",
                    (*ent).s.number,
                    Sz((*ent).classname)
                )
            } else {
                format!("ENT {:4}: Classname Unknown\n", (*ent).s.number)
            };
            Com_Printf(&s);
            #[cfg(not(feature = "vm"))]
            if fh != 0 {
                trap::FS_Write(s.as_bytes(), fh);
            }
        }

        i += 1;
    }

    let s = format!(
        "TempEnt count: {}\nTempEnt ST: {}\nNPC count: {}\nProjectile count: {}\n",
        num_temp_ent, num_temp_ent_st, num_npc, num_projectile
    );
    Com_Printf(&s);
    #[cfg(not(feature = "vm"))]
    if fh != 0 {
        trap::FS_Write(s.as_bytes(), fh);
        trap::FS_FCloseFile(fh);
    }
}

/// `gentity_t *G_Spawn( void )` (g_utils.c:813) — find a free entity slot or open a new
/// one, returning it freshly `G_InitGentity`'d.
///
/// The first `MAX_CLIENTS` slots are reserved for clients, so the search starts past them.
/// The two-pass `force` loop relaxes the "don't reuse a slot
/// freed in the last second" replacement policy on the second pass (which is otherwise
/// rarely reached). On exhaustion it spews the entity list and `G_Error`s; otherwise it
/// grows `level.num_entities`, re-publishes the array to the engine via `trap_LocateGameData`,
/// and inits the new slot. No oracle (entity-array global control flow).
///
/// The C `for`-loop's `i++, e++` runs on every `continue`, so the inner skips here
/// increment-then-`continue` to mirror that.
pub unsafe fn G_Spawn() -> *mut gentity_t {
    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    let lvl = addr_of_mut!(level);

    let mut e: *mut gentity_t = null_mut(); // shut up warning
    let mut i: c_int = 0; // shut up warning
    for force in 0..2 {
        // if we go through all entities and can't find one to free,
        // override the normal minimum times before use
        e = base.add(MAX_CLIENTS);
        i = MAX_CLIENTS as c_int;
        while i < (*lvl).num_entities {
            if (*e).inuse != QFALSE {
                i += 1;
                e = e.add(1);
                continue;
            }

            // the first couple seconds of server time can involve a lot of
            // freeing and allocating, so relax the replacement policy
            if force == 0
                && (*e).freetime > (*lvl).startTime + 2000
                && (*lvl).time - (*e).freetime < 1000
            {
                i += 1;
                e = e.add(1);
                continue;
            }

            // reuse this slot
            G_InitGentity(e);
            return e;
        }
        if i != MAX_GENTITIES as c_int {
            break;
        }
    }
    if i == ENTITYNUM_MAX_NORMAL {
        G_SpewEntList();
        G_Error("G_Spawn: no free entities");
    }

    // open up a new slot
    (*lvl).num_entities += 1;

    // let the server system know that there are more entities
    trap::LocateGameData(
        (*lvl).gentities,
        (*lvl).num_entities,
        core::mem::size_of::<gentity_t>() as c_int,
        addr_of_mut!((*(*lvl).clients).ps),
        core::mem::size_of::<gclient_t>() as c_int,
    );

    G_InitGentity(e);
    e
}

// G_SpawnVehicle (Xbox g_utils.c:866) is not present in the PC tree; the PC
// NPC_Spawn_Do path always uses G_Spawn(). Omitted for Xbox->PC fidelity.

/// `#define MAX_VEHICLES_AT_A_TIME 128` (g_utils.c:384) — size of the static vehicle pool.
const MAX_VEHICLES_AT_A_TIME: usize = 128;

/// `static Vehicle_t g_vehiclePool[MAX_VEHICLES_AT_A_TIME]` / `static qboolean
/// g_vehiclePoolOccupied[MAX_VEHICLES_AT_A_TIME]` (g_utils.c:388) — the fixed pool of vehicle
/// instances and their occupancy flags. Zero-initialised like [`crate::codemp::game::g_main`]'s
/// `g_clients`. Slots are handed out by [`G_AllocateVehicleObject`], returned by
/// [`G_FreeVehicleObject`], and the occupancy array is cleared one-shot on the first
/// [`G_AllocateVehicleObject`] call (gated by [`g_vehiclePoolInit`]); PC `G_InitGame` does not
/// re-init it. The pool's callers are the NPC-vehicle spawn track (Animal/Fighter/Speeder/Walker NPC).
static mut g_vehiclePool: [Vehicle_t; MAX_VEHICLES_AT_A_TIME] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
static mut g_vehiclePoolOccupied: [qboolean; MAX_VEHICLES_AT_A_TIME] =
    [QFALSE; MAX_VEHICLES_AT_A_TIME];

/// `static qboolean g_vehiclePoolInit = qfalse` (g_utils.c:387) — one-shot flag tracking whether
/// the occupancy array has been cleared yet. Set on the first [`G_AllocateVehicleObject`] call
/// (which then zeroes [`g_vehiclePoolOccupied`]). The PC source makes this file-local `static` and
/// drops `G_ClearVehicles`; PC `G_InitGame` never re-inits the pool, so this stays a private
/// file-local `static mut` with no cross-file users.
static mut g_vehiclePoolInit: qboolean = QFALSE;

/// `void G_AllocateVehicleObject( Vehicle_t **pVeh )` (g_utils.c:391) — allocate a vehicle out of
/// the static [`g_vehiclePool`]. On the first call it sets [`g_vehiclePoolInit`] and zeroes the
/// occupancy array; then it linearly scans for a free slot, marks it occupied, zeroes the
/// `Vehicle_t`, and hands its address back through `*pVeh`. If every slot is taken it calls
/// `Com_Error(ERR_DROP, ...)`, which does not return.
///
/// No oracle — mutates the module-static pool and may `Com_Error`.
///
/// # Safety
/// `pVeh` must be a valid writable `*mut Vehicle_t` slot.
pub unsafe fn G_AllocateVehicleObject(pVeh: *mut *mut Vehicle_t) {
    let mut i = 0;

    if g_vehiclePoolInit == QFALSE {
        g_vehiclePoolInit = QTRUE;
        // memset(g_vehiclePoolOccupied, 0, sizeof(g_vehiclePoolOccupied))
        *addr_of_mut!(g_vehiclePoolOccupied) = [QFALSE; MAX_VEHICLES_AT_A_TIME];
    }

    let pool = addr_of_mut!(g_vehiclePool) as *mut Vehicle_t;
    let occupied = addr_of_mut!(g_vehiclePoolOccupied) as *mut qboolean;
    while i < MAX_VEHICLES_AT_A_TIME {
        // iterate through and try to find a free one
        if *occupied.add(i) == QFALSE {
            *occupied.add(i) = QTRUE;
            // memset(&g_vehiclePool[i], 0, sizeof(Vehicle_t))
            core::ptr::write_bytes(pool.add(i), 0, 1);
            *pVeh = pool.add(i);
            return;
        }
        i += 1;
    }
    Com_Error(ERR_DROP, "Ran out of vehicle pool slots.");
}

/// `void G_FreeVehicleObject( Vehicle_t *pVeh )` (g_utils.c:422) — return `pVeh`'s slot to the
/// vehicle pool by clearing its occupancy flag (a "sort of lame" linear scan, per the C). Called
/// from [`G_FreeEntity`] when an NPC vehicle entity is freed. No oracle (scans the module-static
/// pool).
///
/// # Safety
/// `pVeh` may be any pointer — a value that is not a live pool slot is simply a no-op.
pub unsafe fn G_FreeVehicleObject(pVeh: *mut Vehicle_t) {
    let pool = addr_of!(g_vehiclePool) as *const Vehicle_t;
    let occupied = addr_of_mut!(g_vehiclePoolOccupied) as *mut qboolean;
    let mut i = 0;
    while i < MAX_VEHICLES_AT_A_TIME {
        if *occupied.add(i) != QFALSE && pool.add(i) == (pVeh as *const Vehicle_t) {
            // guess this is it
            *occupied.add(i) = QFALSE;
            break;
        }
        i += 1;
    }
}

/// `void G_FreeFakeClient( gclient_t **cl )` (g_utils.c:378) — release a dynamically-allocated
/// fake-client `gclient_t`. Faithfully **empty**: the original's sole `trap_TrueFree` call is
/// commented out (the C comment notes the dynamic free "is busted somehow at the moment", so the
/// memory is left pooled via `gClPtrs` rather than freed). No oracle.
///
/// # Safety
/// `_cl` is unused (matches the C no-op body).
pub unsafe fn G_FreeFakeClient(_cl: *mut *mut gclient_t) {
    //trap_TrueFree((void **)cl);
}

/// `gclient_t *gClPtrs[MAX_GENTITIES]` (g_utils.c:437) — the per-entity pool of fake-client
/// `gclient_t` allocations, indexed by entity number. External-linkage in C, but only
/// [`G_CreateFakeClient`] (and the `_XBOX`-only `G_ClPtrClear`, not built here) touch it, so kept
/// module-private. Each slot is lazily filled by `BG_Alloc` on first use and then re-handed out
/// (never freed — see [`G_FreeFakeClient`]).
static mut gClPtrs: [*mut gclient_t; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];

/// `void G_CreateFakeClient( int entNum, gclient_t **cl )` (g_utils.c:439).
///
/// Hand back a fake-client `gclient_t` for entity `entNum`, allocating one out of the
/// engine-side `BG_Alloc` heap on first use and caching it in `gClPtrs[entNum]` so a second
/// call for the same slot reuses it. The original's dynamic `trap_TrueMalloc` path is commented
/// out (its companion `trap_TrueFree` is "busted somehow at the moment"); the pooled `BG_Alloc`
/// path is what actually runs.
///
/// No oracle: writes through `*cl` and the module-static `gClPtrs` pool via `BG_Alloc`.
///
/// # Safety
/// `entNum` must be a valid entity number (`0..MAX_GENTITIES`); `cl` must be a valid writable
/// `*mut gclient_t` slot.
pub unsafe fn G_CreateFakeClient(entNum: c_int, cl: *mut *mut gclient_t) {
    //trap_TrueMalloc((void **)cl, sizeof(gclient_t));
    let pool = addr_of_mut!(gClPtrs) as *mut *mut gclient_t;
    if (*pool.add(entNum as usize)).is_null() {
        *pool.add(entNum as usize) =
            BG_Alloc(core::mem::size_of::<gclient_t>() as c_int) as *mut gclient_t;
    }
    *cl = *pool.add(entNum as usize);
}

/// `qboolean G_EntitiesFree( void )` (g_utils.c:923) — is there at least one free
/// (`!inuse`) non-client slot in `[MAX_CLIENTS, level.num_entities)`? Scans from the
/// first non-client slot and returns `qtrue` on the first free one, `qfalse` if none.
///
/// No oracle — it reads the global `g_entities`/`level` arrays.
pub unsafe fn G_EntitiesFree() -> qboolean {
    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    let mut e = base.add(MAX_CLIENTS);
    let mut i: c_int = MAX_CLIENTS as c_int;
    while i < (*addr_of!(level)).num_entities {
        if (*e).inuse != QFALSE {
            i += 1;
            e = e.add(1);
            continue;
        }
        // slot available
        return QTRUE;
    }
    QFALSE
}

/// `#define MAX_G2_KILL_QUEUE 256` (g_utils.c:938) — capacity of the per-frame ghoul2-kill queue.
const MAX_G2_KILL_QUEUE: usize = 256;

/// `int gG2KillIndex[MAX_G2_KILL_QUEUE]` / `int gG2KillNum` (g_utils.c:940) — the queue of entity
/// numbers whose **client-side** ghoul2 instances must be destroyed (the server has no ghoul2
/// access), flushed as a batched `"kg2"` server command by [`G_SendG2KillQueue`] (g_utils.c:943).
/// That flush's only caller is the per-frame `G_RunFrame` (g_main.c:4187), which is not yet ported;
/// until it lands the queue also drains via the full-queue fast path in [`G_KillG2Queue`].
static mut gG2KillIndex: [c_int; MAX_G2_KILL_QUEUE] = [0; MAX_G2_KILL_QUEUE];
static mut gG2KillNum: c_int = 0;

/// `void G_SendG2KillQueue( void )` (g_utils.c:943) — flush the pending ghoul2-kill queue to all
/// clients as a single batched `"kg2 <ent> <ent> ..."` server command, up to 64 entity numbers per
/// send. A zero count is a no-op. After sending it decrements `gG2KillNum` by however many it drained;
/// the negative-underflow guard (`assert(0)`) should be impossible but is kept defensively. Called
/// once per frame from `G_RunFrame`. No oracle (drives `trap_SendServerCommand` over the
/// module-static queue).
pub unsafe fn G_SendG2KillQueue() {
    let mut g2KillString: [c_char; 1024] = [0; 1024];
    let mut i: c_int = 0;

    if *addr_of!(gG2KillNum) == 0 {
        return;
    }

    Com_sprintf(g2KillString.as_mut_ptr(), 1024, format_args!("kg2"));

    let queue = addr_of!(gG2KillIndex) as *const c_int;
    while i < *addr_of!(gG2KillNum) && i < 64 {
        // send 64 at once, max...
        Q_strcat(
            g2KillString.as_mut_ptr(),
            1024,
            va(format_args!(" {}", *queue.add(i as usize))),
        );
        i += 1;
    }

    trap::SendServerCommand(-1, &CStr::from_ptr(g2KillString.as_ptr()).to_string_lossy());

    // Clear the count because we just sent off the whole queue
    *addr_of_mut!(gG2KillNum) -= i;
    if *addr_of!(gG2KillNum) < 0 {
        // hmm, should be impossible, but I'm paranoid as we're far past beta.
        debug_assert!(false);
        *addr_of_mut!(gG2KillNum) = 0;
    }
}

/// `void G_KillG2Queue( int entNum )` (g_utils.c:972) — enqueue `entNum` for client-side ghoul2
/// teardown. If the queue is already full it is sent immediately as its own `"kg2"` command (eats
/// more bandwidth, but there is no choice). No oracle (drives `trap_SendServerCommand` over the
/// module-static queue).
pub unsafe fn G_KillG2Queue(entNum: c_int) {
    if *addr_of!(gG2KillNum) >= MAX_G2_KILL_QUEUE as c_int {
        // This would be considered a Bad Thing.
        // (#ifdef _DEBUG "Exceeded the MAX_G2_KILL_QUEUE count" warning omitted — retail build.)
        // Since we're out of queue slots, just send it now as a seperate command
        // (eats more bandwidth, but we have no choice)
        trap::SendServerCommand(-1, &format!("kg2 {}", entNum));
        return;
    }

    let queue = addr_of_mut!(gG2KillIndex) as *mut c_int;
    *queue.add(*addr_of!(gG2KillNum) as usize) = entNum;
    *addr_of_mut!(gG2KillNum) += 1;
}

/// `void G_FreeEntity( gentity_t *ed )` (g_utils.c:995) — mark `ed` as free. Unlinks it from the
/// world and tears down its ICARUS instance, then (unless `neverFree`) cleans up its ghoul2 state:
/// queue the client-side g2 kill, free the server g2 instance, return any vehicle-object slot, and
/// for NPC clients free the linked saber entity, the per-saber `weaponGhoul2` models, and the fake
/// client. Sound-tracker entities clear their index off every client and kill the looping sound.
/// Finally the slot is zeroed and stamped `"freed"`. No oracle (engine calls + global entity-array
/// mutation).
///
/// `extern "C"` because the original assigns `G_FreeEntity` directly to `ent->think`
/// (e.g. `G_RunExPhys`'s autoKill path, `target_relay`), so it must be ABI-compatible with
/// the `think` function-pointer slot.
///
/// # Safety
/// `ed` must point to a valid `gentity_t` in the `g_entities` array.
pub unsafe extern "C" fn G_FreeEntity(ed: *mut gentity_t) {
    if (*ed).isSaberEntity != QFALSE {
        // #ifdef _DEBUG Com_Printf("Tried to remove JM saber!\n") — debug-only, omitted (retail).
        return;
    }

    trap::UnlinkEntity(ed); // unlink from world

    trap::ICARUS_FreeEnt(ed); // ICARUS information must be added after this point

    if (*ed).neverFree != QFALSE {
        return;
    }

    // rww - this may seem a bit hackish, but unfortunately we have no access to anything
    // ghoul2-related on the server and thus must send a message to let the client know he needs
    // to clean up all the g2 stuff for this now-removed entity
    if (*ed).s.modelGhoul2 != 0 {
        // force all clients to accept an event to destroy this instance, right now.
        // (the G_TempEntity EV_DESTROY_GHOUL2_INSTANCE path is commented out in C — events can be
        // dropped, which would be a bad thing — so the kill goes through the reliable queue.)
        G_KillG2Queue((*ed).s.number);
    }

    // And, free the server instance too, if there is one.
    if !(*ed).ghoul2.is_null() {
        trap::G2API_CleanGhoul2Models(addr_of_mut!((*ed).ghoul2));
    }

    if (*ed).s.eType == ET_NPC && !(*ed).m_pVehicle.is_null() {
        // tell the "vehicle pool" that this one is now free
        G_FreeVehicleObject((*ed).m_pVehicle);
    }

    if (*ed).s.eType == ET_NPC && !(*ed).client.is_null() {
        // this "client" structure is one of our dynamically allocated ones, so free the memory
        let mut saberEntNum: c_int = -1;
        let mut i: c_int = 0;
        if (*(*ed).client).ps.saberEntityNum != 0 {
            saberEntNum = (*(*ed).client).ps.saberEntityNum;
        } else if (*(*ed).client).saberStoredIndex != 0 {
            saberEntNum = (*(*ed).client).saberStoredIndex;
        }

        let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        if saberEntNum > 0 && (*base.add(saberEntNum as usize)).inuse != QFALSE {
            (*base.add(saberEntNum as usize)).neverFree = QFALSE;
            G_FreeEntity(base.add(saberEntNum as usize));
        }

        while i < MAX_SABERS as c_int {
            let g2 = (*(*ed).client).weaponGhoul2[i as usize];
            if !g2.is_null() && trap::G2_HaveWeGhoul2Models(g2) != QFALSE {
                trap::G2API_CleanGhoul2Models(addr_of_mut!(
                    (*(*ed).client).weaponGhoul2[i as usize]
                ));
            }
            i += 1;
        }

        G_FreeFakeClient(addr_of_mut!((*ed).client));
    }

    if (*ed).s.eFlags & EF_SOUNDTRACKER != 0 {
        let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        let mut i: c_int = 0;
        while i < MAX_CLIENTS as c_int {
            let ent = base.add(i as usize);
            if (*ent).inuse != QFALSE && !(*ent).client.is_null() {
                let mut ch: c_int = TRACK_CHANNEL_NONE - 50;
                while ch < NUM_TRACK_CHANNELS - 50 {
                    if (*(*ent).client).ps.fd.killSoundEntIndex[ch as usize] == (*ed).s.number {
                        (*(*ent).client).ps.fd.killSoundEntIndex[ch as usize] = 0;
                    }
                    ch += 1;
                }
            }
            i += 1;
        }

        // make sure clientside loop sounds are killed on the tracker and client
        trap::SendServerCommand(
            -1,
            &format!("kls {} {}", (*ed).s.trickedentindex, (*ed).s.number),
        );
    }

    core::ptr::write_bytes(ed, 0, 1);
    (*ed).classname = c"freed".as_ptr() as *mut c_char;
    (*ed).freetime = (*addr_of!(level)).time;
    (*ed).inuse = QFALSE;
}

/// `gentity_t *G_TempEntity( vec3_t origin, int event )` (g_utils.c:1117) — spawn a
/// transient event entity that the engine auto-removes after the event fires.
///
/// `eType` is `ET_EVENTS + event` (the freestanding-event encoding), `freeAfterEvent` is
/// set, and the origin is `SnapVector`'d (inline, not `trap_SnapVector`) before
/// [`G_SetOrigin`] to save network bandwidth — so callers must snap toward the start vector
/// first if the origin sits right on a surface. The entity is linked for PVS clustering.
/// No oracle (allocates from the `g_entities`/`level` globals via [`G_Spawn`] and calls
/// `trap_LinkEntity`).
pub unsafe fn G_TempEntity(origin: &vec3_t, event: c_int) -> *mut gentity_t {
    let e = G_Spawn();
    (*e).s.eType = ET_EVENTS + event;

    (*e).classname = c"tempEntity".as_ptr() as *mut c_char;
    (*e).eventTime = (*addr_of!(level)).time;
    (*e).freeAfterEvent = QTRUE;

    let mut snapped: vec3_t = [0.0; 3];
    VectorCopy(origin, &mut snapped);
    snap_vector(&mut snapped); // save network bandwidth
    G_SetOrigin(e, &snapped);

    // find cluster for PVS
    trap::LinkEntity(e);

    e
}

/// `void G_KillBox( gentity_t *ent )` (g_utils.c:1225). Telefrags every *other* client
/// whose bounding box overlaps `ent`'s proposed new position (`ent->client->ps.origin` +
/// box). Skips `ent` itself and its owner (vehicle). The C comment notes `ent` should be
/// unlinked before calling. No oracle (`trap_EntitiesInBox` + the `G_Damage` chain over
/// global `g_entities`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`.
pub unsafe fn G_KillBox(ent: *mut gentity_t) {
    let mut touch: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    VectorAdd(&(*(*ent).client).ps.origin, &(*ent).r.mins, &mut mins);
    VectorAdd(&(*(*ent).client).ps.origin, &(*ent).r.maxs, &mut maxs);
    let num = trap::EntitiesInBox(&mins, &maxs, &mut touch);

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    for i in 0..num as usize {
        let hit = base.add(touch[i] as usize);
        if (*hit).client.is_null() {
            continue;
        }

        if (*hit).s.number == (*ent).s.number {
            //don't telefrag yourself!
            continue;
        }

        if (*ent).r.ownerNum == (*hit).s.number {
            //don't telefrag your vehicle!
            continue;
        }

        // nail it
        G_Damage(
            hit,
            ent,
            ent,
            null_mut(),
            null_mut(),
            100000,
            DAMAGE_NO_PROTECTION,
            MOD_TELEFRAG,
        );
    }
}

/// `void G_AddPredictableEvent( gentity_t *ent, int event, int eventParm )` (g_utils.c:1269)
/// — for non-pmove events that are also client-predicted (jumppads, item pickups). Clients
/// only: defers to [`BG_AddPredictableEventToPlayerstate`] on `ent->client->ps`; a no-op for
/// non-clients. No oracle (mutates a `gclient_t`).
pub unsafe fn G_AddPredictableEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int) {
    if (*ent).client.is_null() {
        return;
    }
    BG_AddPredictableEventToPlayerstate(event, eventParm, &mut (*(*ent).client).ps);
}

/// `void G_AddEvent( gentity_t *ent, int event, int eventParm )` (g_utils.c:1284) — queue
/// an entity event for transmission. A zero `event` is a no-op (warned). Clients carry the
/// event in `playerState_t::externalEvent` (with the rolling 2-bit `EV_EVENT_BIT1` sequence
/// counter so repeats re-trigger); non-clients use `entityState_t::event`. `eventTime` is
/// stamped either way. No oracle (mutates a `gentity_t`/`gclient_t` and reads `level.time`).
pub unsafe fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int) {
    if event == 0 {
        G_Printf(&format!(
            "G_AddEvent: zero event added for entity {}\n",
            (*ent).s.number
        ));
        return;
    }

    // clients need to add the event in playerState_t instead of entityState_t
    if !(*ent).client.is_null() {
        let mut bits = (*(*ent).client).ps.externalEvent & EV_EVENT_BITS;
        bits = (bits + EV_EVENT_BIT1) & EV_EVENT_BITS;
        (*(*ent).client).ps.externalEvent = event | bits;
        (*(*ent).client).ps.externalEventParm = eventParm;
        (*(*ent).client).ps.externalEventTime = (*addr_of!(level)).time;
    } else {
        let mut bits = (*ent).s.event & EV_EVENT_BITS;
        bits = (bits + EV_EVENT_BIT1) & EV_EVENT_BITS;
        (*ent).s.event = event | bits;
        (*ent).s.eventParm = eventParm;
    }
    (*ent).eventTime = (*addr_of!(level)).time;
}

/// `gentity_t *G_PlayEffect( int fxID, vec3_t org, vec3_t ang )` (g_utils.c:1313) — spawn an
/// `EV_PLAY_EFFECT` temp-entity at `org` carrying the predefined effect id in `eventParm`.
/// No oracle (allocates via [`G_TempEntity`]).
pub unsafe fn G_PlayEffect(fxID: c_int, org: &vec3_t, ang: &vec3_t) -> *mut gentity_t {
    let te = G_TempEntity(org, EV_PLAY_EFFECT);
    VectorCopy(ang, &mut (*te).s.angles);
    VectorCopy(org, &mut (*te).s.origin);
    (*te).s.eventParm = fxID;

    te
}

/// `gentity_t *G_PlayEffectID( const int fxID, vec3_t org, vec3_t ang )` (g_utils.c:1330) —
/// like [`G_PlayEffect`] but `fxID` is a `G_EffectIndex`'d configstring id (`EV_PLAY_EFFECT_ID`).
/// A zero direction defaults to `+Y` so the effect has a play axis. No oracle.
pub unsafe fn G_PlayEffectID(fxID: c_int, org: &vec3_t, ang: &vec3_t) -> *mut gentity_t {
    let te = G_TempEntity(org, EV_PLAY_EFFECT_ID);
    VectorCopy(ang, &mut (*te).s.angles);
    VectorCopy(org, &mut (*te).s.origin);
    (*te).s.eventParm = fxID;

    if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
        // play off this dir by default then.
        (*te).s.angles[1] = 1.0;
    }

    te
}

/// `gentity_t *G_ScreenShake( vec3_t org, gentity_t *target, float intensity, int duration,
/// qboolean global )` (g_utils.c:1353) — spawn an `EV_SCREENSHAKE` temp-entity. `intensity`
/// rides in `s.angles[0]`, `duration` in `s.time`; `target` (1-based, 0 = none) selects whose
/// view shakes via `s.modelindex`, and `global` broadcasts via `SVF_BROADCAST`. No oracle.
pub unsafe fn G_ScreenShake(
    org: &vec3_t,
    target: *mut gentity_t,
    intensity: f32,
    duration: c_int,
    global: qboolean,
) -> *mut gentity_t {
    let te = G_TempEntity(org, EV_SCREENSHAKE);
    VectorCopy(org, &mut (*te).s.origin);
    (*te).s.angles[0] = intensity;
    (*te).s.time = duration;

    if !target.is_null() {
        (*te).s.modelindex = (*target).s.number + 1;
    } else {
        (*te).s.modelindex = 0;
    }

    if global != QFALSE {
        (*te).r.svFlags |= SVF_BROADCAST;
    }

    te
}

/// `gentity_t *G_SoundTempEntity( vec3_t origin, int event, int channel )` (g_utils.c:1148)
/// — like [`G_TempEntity`] but a *sound-tracker-aware* event entity: it additionally stamps
/// `s.clientNum` with the active client and explicitly sets `inuse`. The PC build's
/// `ClientManager::ActiveClientNum()` collapses to the single active client `0` (see
/// `DEVIATIONS.md` / [`BG_BLADE_ActivateTrail`]). `channel` is unused here — [`G_Sound`]
/// writes it into `s.saberEntityNum` on the returned entity. No oracle (allocates via
/// [`G_Spawn`] + traps).
pub unsafe fn G_SoundTempEntity(origin: &vec3_t, event: c_int, _channel: c_int) -> *mut gentity_t {
    let e = G_Spawn();

    (*e).s.eType = ET_EVENTS + event;
    (*e).inuse = QTRUE;

    (*e).classname = c"tempEntity".as_ptr() as *mut c_char;
    (*e).eventTime = (*addr_of!(level)).time;
    (*e).freeAfterEvent = QTRUE;

    let mut snapped: vec3_t = [0.0; 3];
    VectorCopy(origin, &mut snapped);
    snap_vector(&mut snapped); // save network bandwidth
    G_SetOrigin(e, &snapped);

    // find cluster for PVS
    trap::LinkEntity(e);

    e
}

/// `void G_MuteSound( int entnum, int channel )` (g_utils.c:1385) — broadcast an
/// `EV_MUTE_SOUND` temp-entity telling clients to silence the looping sound on `channel` of
/// entity `entnum` (carried in `s.trickedentindex2`/`s.trickedentindex`), then free the
/// `g_entities[entnum]` sound-tracker entity if it still carries `EF_SOUNDTRACKER`. No oracle
/// (broadcasts via [`G_TempEntity`] and frees from the `g_entities` global).
pub unsafe fn G_MuteSound(entnum: c_int, channel: c_int) {
    let te = G_TempEntity(&vec3_origin, EV_MUTE_SOUND);
    (*te).r.svFlags = SVF_BROADCAST;
    (*te).s.trickedentindex2 = entnum;
    (*te).s.trickedentindex = channel;

    let e = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entnum as usize);

    if ((*e).s.eFlags & EF_SOUNDTRACKER) != 0 {
        G_FreeEntity(e);
        (*e).s.eFlags = 0;
    }
}

/// `void G_Sound( gentity_t *ent, int channel, int soundIndex )` (g_utils.c:1408) — fire an
/// `EV_GENERAL_SOUND` for `ent` on `channel` via [`G_SoundTempEntity`]. When `ent` is a
/// client and `channel` is an explicit tracking channel (`> TRACK_CHANNEL_NONE`), it first
/// mutes/frees that client's previously kill-tracked sound on the channel
/// (`ps.fd.killSoundEntIndex[channel-50]`, re-read after [`G_MuteSound`] because freeing the
/// tracker can zero it), then records this temp-entity's number there and tags it
/// `EF_SOUNDTRACKER` so a later [`G_MuteSound`] can silence it. No oracle (allocates from /
/// frees the `g_entities` global).
pub unsafe fn G_Sound(ent: *mut gentity_t, channel: c_int, soundIndex: c_int) {
    debug_assert!(soundIndex != 0);

    let te = G_SoundTempEntity(&(*ent).r.currentOrigin, EV_GENERAL_SOUND, channel);
    (*te).s.eventParm = soundIndex;
    (*te).s.saberEntityNum = channel;

    if !ent.is_null() && !(*ent).client.is_null() && channel > TRACK_CHANNEL_NONE {
        // let the client remember the index of the player entity so he can kill the most
        // recent sound on request
        let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        let idx = (*(*ent).client).ps.fd.killSoundEntIndex[(channel - 50) as usize];
        if (*base.add(idx as usize)).inuse != QFALSE && idx > MAX_CLIENTS as c_int {
            G_MuteSound(idx, CHAN_VOICE);
            let idx2 = (*(*ent).client).ps.fd.killSoundEntIndex[(channel - 50) as usize];
            if idx2 > MAX_CLIENTS as c_int && (*base.add(idx2 as usize)).inuse != QFALSE {
                G_FreeEntity(base.add(idx2 as usize));
            }
            (*(*ent).client).ps.fd.killSoundEntIndex[(channel - 50) as usize] = 0;
        }

        (*(*ent).client).ps.fd.killSoundEntIndex[(channel - 50) as usize] = (*te).s.number;
        (*te).s.trickedentindex = (*ent).s.number;
        (*te).s.eFlags = EF_SOUNDTRACKER;
        //te->freeAfterEvent = qfalse;
    }
}

/// `void G_SoundAtLoc( vec3_t loc, int channel, int soundIndex )` (g_utils.c:1442) — fire a
/// world-positioned `EV_GENERAL_SOUND` at `loc` (no source entity). No oracle.
pub unsafe fn G_SoundAtLoc(loc: &vec3_t, channel: c_int, soundIndex: c_int) {
    let te = G_TempEntity(loc, EV_GENERAL_SOUND);
    (*te).s.eventParm = soundIndex;
    (*te).s.saberEntityNum = channel;
}

/// `void G_EntitySound( gentity_t *ent, int channel, int soundIndex )` (g_utils.c:1455) —
/// fire an `EV_ENTITY_SOUND` that follows `ent` (the client renders it attached to
/// `ent->s.number` on `channel`). No oracle.
pub unsafe fn G_EntitySound(ent: *mut gentity_t, channel: c_int, soundIndex: c_int) {
    let te = G_TempEntity(&(*ent).r.currentOrigin, EV_ENTITY_SOUND);
    (*te).s.eventParm = soundIndex;
    (*te).s.clientNum = (*ent).s.number;
    (*te).s.trickedentindex = channel;
}

/// `void G_SoundOnEnt( gentity_t *ent, int channel, const char *soundPath )` (g_utils.c:1465)
/// — SP-porting convenience: like [`G_EntitySound`] but interns `soundPath` via
/// [`G_SoundIndex`] first. The `const char *` path is taken as `&str` to match the sibling
/// configstring-intern helpers. No oracle.
pub unsafe fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundPath: &str) {
    let te = G_TempEntity(&(*ent).r.currentOrigin, EV_ENTITY_SOUND);
    (*te).s.eventParm = G_SoundIndex(soundPath);
    (*te).s.clientNum = (*ent).s.number;
    (*te).s.trickedentindex = channel;
}

/// `void G_ScaleNetHealth( gentity_t *self )` (g_utils.c:1175).
///
/// Mirrors the entity's `maxHealth`/`health` into the networked `s.maxhealth`/`s.health`
/// the client draws the health bar from. Values under 1000 pass through verbatim;
/// 1000+ are scaled down by 100 to fit the netfield. Negative results clamp to 0, and a
/// still-living entity (`health > 0`) never scales to a "dead" `s.health <= 0` — it
/// floors at 1. No oracle (mutates a `gentity_t`).
pub unsafe fn G_ScaleNetHealth(self_: *mut gentity_t) {
    let maxHealth = (*self_).maxHealth;

    if maxHealth < 1000 {
        // it's good then
        (*self_).s.maxhealth = maxHealth;
        (*self_).s.health = (*self_).health;

        if (*self_).s.health < 0 {
            // don't let it wrap around
            (*self_).s.health = 0;
        }
        return;
    }

    // otherwise, scale it down
    (*self_).s.maxhealth = maxHealth / 100;
    (*self_).s.health = (*self_).health / 100;

    if (*self_).s.health < 0 {
        // don't let it wrap around
        (*self_).s.health = 0;
    }

    if (*self_).health > 0 && (*self_).s.health <= 0 {
        // don't let it scale to 0 if the thing is still not "dead"
        (*self_).s.health = 1;
    }
}

/// `void G_SetAngles( gentity_t *ent, vec3_t angles )` (g_utils.c:1990) — fix an entity's
/// orientation by stamping `angles` into the shared `r.currentAngles`, the networked
/// `s.angles`, and the angle-trajectory base `s.apos.trBase`. No oracle (mutates a
/// `gentity_t`, which the isolated-C oracle can't construct).
pub unsafe fn G_SetAngles(ent: *mut gentity_t, angles: &vec3_t) {
    VectorCopy(angles, &mut (*ent).r.currentAngles);
    VectorCopy(angles, &mut (*ent).s.angles);
    VectorCopy(angles, &mut (*ent).s.apos.trBase);
}

/// `qboolean G_ClearTrace( vec3_t start, vec3_t mins, vec3_t maxs, vec3_t end, int ignore,
/// int clipmask )` (g_utils.c:1997).
///
/// Is the swept box from `start` to `end` (ignoring entity `ignore`, against `clipmask`)
/// completely clear? Returns false if the trace starts solid, is all solid, or stops
/// short (`fraction < 1.0`). The C parks the result in a `static trace_t`; the Rust trap
/// returns it by value, so there is no static here. No oracle (calls `trap_Trace`).
pub fn G_ClearTrace(
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    let tr = trap::Trace(start, mins, maxs, end, ignore, clipmask);

    if tr.allsolid != 0 || tr.startsolid != 0 || tr.fraction < 1.0 {
        return QFALSE;
    }

    QTRUE
}

/// `void G_SetOrigin( gentity_t *ent, vec3_t origin )` (g_utils.c:2018) — set the position
/// trajectory for a fixed (stationary) position: stamp `origin` into `s.pos.trBase`, mark
/// the trajectory `TR_STATIONARY` with zeroed time/duration/delta, and copy `origin` into
/// the shared `r.currentOrigin`. No oracle (mutates a `gentity_t`).
pub unsafe fn G_SetOrigin(ent: *mut gentity_t, origin: &vec3_t) {
    VectorCopy(origin, &mut (*ent).s.pos.trBase);
    (*ent).s.pos.trType = TR_STATIONARY;
    (*ent).s.pos.trTime = 0;
    (*ent).s.pos.trDuration = 0;
    VectorClear(&mut (*ent).s.pos.trDelta);

    VectorCopy(origin, &mut (*ent).r.currentOrigin);
}

/// `qboolean G_CheckInSolid( gentity_t *self, qboolean fix )` (g_utils.c:2028).
///
/// Traces straight down from `self`'s origin (a zero-floored copy of its bounding box,
/// dropped by `r.mins[2]`) to detect the entity standing in solid. Returns true when it
/// is. When `fix` is set and the downward trace is merely obstructed (not start/all
/// solid), it relocates `self` to the trace endpoint via [`G_SetOrigin`] + `trap_LinkEntity`
/// and re-checks once with `fix = qfalse`. No oracle (calls `trap_Trace`/`trap_LinkEntity`).
pub unsafe fn G_CheckInSolid(self_: *mut gentity_t, fix: qboolean) -> qboolean {
    let mut end: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];

    VectorCopy(&(*self_).r.currentOrigin, &mut end);
    end[2] += (*self_).r.mins[2];
    VectorCopy(&(*self_).r.mins, &mut mins);
    mins[2] = 0.0;

    let trace = trap::Trace(
        &(*self_).r.currentOrigin,
        &mins,
        &(*self_).r.maxs,
        &end,
        (*self_).s.number,
        (*self_).clipmask,
    );
    if trace.allsolid != 0 || trace.startsolid != 0 {
        return QTRUE;
    }

    if trace.fraction < 1.0 {
        if fix != QFALSE {
            // Put them at end of trace and check again
            let mut neworg: vec3_t = [0.0; 3];

            VectorCopy(&trace.endpos, &mut neworg);
            neworg[2] -= (*self_).r.mins[2];
            G_SetOrigin(self_, &neworg);
            trap::LinkEntity(self_);

            return G_CheckInSolid(self_, QFALSE);
        } else {
            return QTRUE;
        }
    }

    QFALSE
}

/// `void G_SpeechEvent( gentity_t *self, int event )` (g_utils.c:2145) — thin wrapper that
/// fires `event` on `self` with parm 0 via [`G_AddEvent`]. No oracle.
pub unsafe fn G_SpeechEvent(self_: *mut gentity_t, event: c_int) {
    G_AddEvent(self_, event, 0);
}

/// `qboolean G_PointInBounds( vec3_t point, vec3_t mins, vec3_t maxs )`
/// (g_utils.c:1940) — is `point` inside the `[mins, maxs]` AABB (inclusive)?
pub fn G_PointInBounds(point: &vec3_t, mins: &vec3_t, maxs: &vec3_t) -> qboolean {
    for i in 0..3 {
        if point[i] < mins[i] {
            return QFALSE;
        }
        if point[i] > maxs[i] {
            return QFALSE;
        }
    }

    QTRUE
}

/// `qboolean G_BoxInBounds( vec3_t point, vec3_t mins, vec3_t maxs, vec3_t boundsMins,
/// vec3_t boundsMaxs )` (g_utils.c:1959) — is the box `point+[mins,maxs]` completely
/// contained within `[boundsMins, boundsMaxs]`?
pub fn G_BoxInBounds(
    point: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    boundsMins: &vec3_t,
    boundsMaxs: &vec3_t,
) -> qboolean {
    let mut boxMins: vec3_t = [0.0; 3];
    let mut boxMaxs: vec3_t = [0.0; 3];

    VectorAdd(point, mins, &mut boxMins);
    VectorAdd(point, maxs, &mut boxMaxs);

    if boxMaxs[0] > boundsMaxs[0] {
        return QFALSE;
    }

    if boxMaxs[1] > boundsMaxs[1] {
        return QFALSE;
    }

    if boxMaxs[2] > boundsMaxs[2] {
        return QFALSE;
    }

    if boxMins[0] < boundsMins[0] {
        return QFALSE;
    }

    if boxMins[1] < boundsMins[1] {
        return QFALSE;
    }

    if boxMins[2] < boundsMins[2] {
        return QFALSE;
    }

    // box is completely contained within bounds
    QTRUE
}

/// `void G_EntityPosition( int i, vec3_t ret )` (g_utils.c:1478).
///
/// Writes the world position of entity `i` into `ret`: its `r.currentOrigin` if the index
/// is valid and the entity is in use, else the zero vector. The bmodel-centroid branch is
/// `#if 0`'d out in the original C ("Do we really care about doing this? It's slow and
/// unnecessary") so only the live `VectorCopy` / zero-fill branches are ported. No oracle
/// (reads the global `g_entities` array).
pub unsafe fn G_EntityPosition(i: c_int, ret: &mut vec3_t) {
    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    if
    /*g_entities &&*/
    i >= 0 && i < MAX_GENTITIES as c_int && (*base.add(i as usize)).inuse != QFALSE {
        // #if 0	// VVFIXME - Do we really care about doing this? It's slow and unnecessary
        //   (bmodel CM_InlineModel/CM_ModelBounds centroid branch — disabled in C)
        // #endif
        VectorCopy(&(*base.add(i as usize)).r.currentOrigin, ret);
    } else {
        ret[0] = 0.0;
        ret[1] = 0.0;
        ret[2] = 0.0;
    }
}

/// `qboolean ValidUseTarget( gentity_t *ent )` (g_utils.c:1516).
///
/// Is `ent` usable by a player pressing BUTTON_USE? It must have a `use` callback, must
/// not be deactivated (`FL_INACTIVE`, set by `target_deactivate`), and must carry the
/// `SVF_PLAYER_USABLE` shared flag. No oracle (reads an entity's `use` fn-pointer and
/// flag fields).
pub unsafe fn ValidUseTarget(ent: *mut gentity_t) -> qboolean {
    if (*ent).r#use.is_none() {
        return QFALSE;
    }

    if (*ent).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return QFALSE;
    }

    if (*ent).r.svFlags & SVF_PLAYER_USABLE == 0 {
        // Check for flag that denotes BUTTON_USE useability
        return QFALSE;
    }

    QTRUE
}

/// `qboolean G_ExpandPointToBBox( vec3_t point, const vec3_t mins, const vec3_t maxs,
/// int ignore, int clipmask )` (g_utils.c:2150).
///
/// Tries to seat a `mins`/`maxs` box around `point` without it intersecting solid,
/// nudging `point` along each axis as needed: per axis, trace toward `mins[i]`; if
/// blocked, slide the start forward and trace the remaining `maxs[i] - mins[i]*fraction`
/// the other way. Returns false the moment any trace starts solid or stays obstructed.
/// On success the adjusted position is written back into `point` and it returns true.
/// No oracle (calls `trap_Trace`).
pub fn G_ExpandPointToBBox(
    point: &mut vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    VectorCopy(point, &mut start);

    for i in 0..3 {
        VectorCopy(&start, &mut end);
        end[i] += mins[i];
        let mut tr = trap::Trace(&start, &vec3_origin, &vec3_origin, &end, ignore, clipmask);
        if tr.allsolid != 0 || tr.startsolid != 0 {
            return QFALSE;
        }
        if tr.fraction < 1.0 {
            VectorCopy(&start, &mut end);
            end[i] += maxs[i] - (mins[i] * tr.fraction);
            tr = trap::Trace(&start, &vec3_origin, &vec3_origin, &end, ignore, clipmask);
            if tr.allsolid != 0 || tr.startsolid != 0 {
                return QFALSE;
            }
            if tr.fraction < 1.0 {
                return QFALSE;
            }
            VectorCopy(&end, &mut start);
        }
    }

    // expanded it, now see if it's all clear
    let tr = trap::Trace(&start, mins, maxs, &start, ignore, clipmask);
    if tr.allsolid != 0 || tr.startsolid != 0 {
        return QFALSE;
    }

    VectorCopy(&start, point);
    QTRUE
}

/// `float ShortestLineSegBewteen2LineSegs( vec3_t start1, vec3_t end1, vec3_t start2,
/// vec3_t end2, vec3_t close_pnt1, vec3_t close_pnt2 )` (g_utils.c:2194).
///
/// Closest distance between two line segments (`start1`→`end1` and `start2`→`end2`),
/// writing the nearest point on each into `close_pnt1`/`close_pnt2`. Solves the
/// parametric `(s, t)` minimum first; if either parameter falls outside `[0, 1]` (or the
/// segments are near-parallel, `|denom| <= 0.001`), it falls back to an exhaustive sweep
/// of the four endpoint pairs and four endpoint-vs-segment projections, keeping the
/// shortest. Misspelling `Bewteen` is carried from the original. Pure float math — the
/// `fabs` test runs in double like the C; everything else is single-precision.
pub fn ShortestLineSegBewteen2LineSegs(
    start1: &vec3_t,
    end1: &vec3_t,
    start2: &vec3_t,
    end2: &vec3_t,
    close_pnt1: &mut vec3_t,
    close_pnt2: &mut vec3_t,
) -> f32 {
    let mut new_pnt: vec3_t = [0.0; 3];

    // compute some temporaries
    let mut start_dif: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    VectorSubtract(start2, start1, &mut start_dif);
    VectorSubtract(end1, start1, &mut v1);
    VectorSubtract(end2, start2, &mut v2);

    let v1v1 = DotProduct(&v1, &v1);
    let v2v2 = DotProduct(&v2, &v2);
    let v1v2 = DotProduct(&v1, &v2);

    // the main computation
    let denom = (v1v2 * v1v2) - (v1v1 * v2v2);

    let mut current_dist: f32;

    // if denom is small, then skip all this and jump to the section marked below
    if (denom as f64).abs() > 0.001f32 as f64 {
        let mut s =
            -((v2v2 * DotProduct(&v1, &start_dif)) - (v1v2 * DotProduct(&v2, &start_dif))) / denom;
        let mut t =
            ((v1v1 * DotProduct(&v2, &start_dif)) - (v1v2 * DotProduct(&v1, &start_dif))) / denom;
        let mut done = QTRUE;

        if s < 0.0 {
            done = QFALSE;
            s = 0.0; // and see note below
        }

        if s > 1.0 {
            done = QFALSE;
            s = 1.0; // and see note below
        }

        if t < 0.0 {
            done = QFALSE;
            t = 0.0; // and see note below
        }

        if t > 1.0 {
            done = QFALSE;
            t = 1.0; // and see note below
        }

        // vec close_pnt1 = start1 + s * v1
        VectorMA(start1, s, &v1, close_pnt1);
        // vec close_pnt2 = start2 + t * v2
        VectorMA(start2, t, &v2, close_pnt2);

        current_dist = Distance(close_pnt1, close_pnt2);
        // now, if none of those if's fired, you are done.
        if done == QTRUE {
            return current_dist;
        }
        // If they did fire, then we need to do some additional tests.
        //
        // What we are gonna do is see if we can find a shorter distance than the above
        // involving the endpoints.
    } else {
        // start here for parallel lines with current_dist = infinity
        current_dist = Q3_INFINITE as f32;
    }

    // test all the endpoints
    let mut new_dist = Distance(start1, start2);
    if new_dist < current_dist {
        VectorCopy(start1, close_pnt1);
        VectorCopy(start2, close_pnt2);
        current_dist = new_dist;
    }

    new_dist = Distance(start1, end2);
    if new_dist < current_dist {
        VectorCopy(start1, close_pnt1);
        VectorCopy(end2, close_pnt2);
        current_dist = new_dist;
    }

    new_dist = Distance(end1, start2);
    if new_dist < current_dist {
        VectorCopy(end1, close_pnt1);
        VectorCopy(start2, close_pnt2);
        current_dist = new_dist;
    }

    new_dist = Distance(end1, end2);
    if new_dist < current_dist {
        VectorCopy(end1, close_pnt1);
        VectorCopy(end2, close_pnt2);
        current_dist = new_dist;
    }

    // Then we have 4 more point / segment tests
    G_FindClosestPointOnLineSegment(start2, end2, start1, &mut new_pnt);
    new_dist = Distance(start1, &new_pnt);
    if new_dist < current_dist {
        VectorCopy(start1, close_pnt1);
        VectorCopy(&new_pnt, close_pnt2);
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start2, end2, end1, &mut new_pnt);
    new_dist = Distance(end1, &new_pnt);
    if new_dist < current_dist {
        VectorCopy(end1, close_pnt1);
        VectorCopy(&new_pnt, close_pnt2);
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start1, end1, start2, &mut new_pnt);
    new_dist = Distance(start2, &new_pnt);
    if new_dist < current_dist {
        VectorCopy(&new_pnt, close_pnt1);
        VectorCopy(start2, close_pnt2);
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start1, end1, end2, &mut new_pnt);
    new_dist = Distance(end2, &new_pnt);
    if new_dist < current_dist {
        VectorCopy(&new_pnt, close_pnt1);
        VectorCopy(end2, close_pnt2);
        current_dist = new_dist;
    }

    current_dist
}

/// `void GetAnglesForDirection( const vec3_t p1, const vec3_t p2, vec3_t out )`
/// (g_utils.c:2372).
///
/// Euler angles of the direction from `p1` to `p2`: subtract, then [`vectoangles`].
/// Pure float math (both helpers are already bit-exact-verified against C).
pub fn GetAnglesForDirection(p1: &vec3_t, p2: &vec3_t, out: &mut vec3_t) {
    let mut v: vec3_t = [0.0; 3];

    VectorSubtract(p2, p1, &mut v);
    vectoangles(&v, out);
}

/// `qboolean G_PlayerHasCustomSkeleton( gentity_t *ent )` (g_utils.c:164).
///
/// Predicate for whether a player is using a class with a custom skeleton (Siege
/// `CFL_CUSTOMSKEL`). In original JKA the entire body is commented out and the function
/// unconditionally returns `qfalse` — faithfully carried here: the `#if 0`-style block
/// (`bgSiegeClasses[ent->client->siegeClass]` / `classflags & (1<<CFL_CUSTOMSKEL)`) is
/// kept as a doc note, the live body is `return qfalse`. No oracle (constant return).
///
/// # Safety
/// `ent` is unused in the live body; the signature matches the C caller contract.
pub unsafe fn G_PlayerHasCustomSkeleton(_ent: *mut gentity_t) -> qboolean {
    /*
    siegeClass_t *scl;

    if (g_gametype.integer != GT_SIEGE)
    { //only in siege
        return qfalse;
    }

    if (ent->s.number >= MAX_CLIENTS ||
        !ent->client ||
        ent->client->siegeClass == -1)
    { //invalid class
        return qfalse;
    }

    scl = &bgSiegeClasses[ent->client->siegeClass];
    if (!(scl->classflags & (1<<CFL_CUSTOMSKEL)))
    { //class is not flagged for this
        return qfalse;
    }

    return qtrue;
    */
    QFALSE
}

/// `qboolean TryHeal( gentity_t *ent, gentity_t *target )` (g_utils.c:1609).
///
/// Siege-only object healing: if `ent`'s active siege class name matches the
/// `target`'s `healingclass`, top the target's health up by 10 (capped at
/// `maxHealth`) on a `healingrate` debounce, play the optional `healingsound`,
/// rescale the crosshair net-health bar (and mirror it onto a linked
/// `target_ent`), and keep `ent` in the button-hold healing torso anim.
///
/// No oracle: pure entity/`gclient` health-and-anim mutation with engine-call
/// side-effects (`G_Sound`/`G_SetAnim`), not a computable data table.
///
/// `target->healingsound` is a raw `char *`; [`G_SoundIndex`] takes a Rust `&str`
/// (the configstring-family deviation), so it is bridged through
/// `CStr::from_ptr(...).to_str()` like the other char-pointer call sites here.
pub unsafe fn TryHeal(ent: *mut gentity_t, target: *mut gentity_t) -> qboolean {
    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*(*ent).client).siegeClass != -1
        && !target.is_null()
        && (*target).inuse != QFALSE
        && (*target).maxHealth != 0
        && !(*target).healingclass.is_null()
        && *(*target).healingclass != 0
        && (*target).health > 0
        && (*target).health < (*target).maxHealth
    {
        //it's not dead yet...
        let scl = (addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t)
            .add((*(*ent).client).siegeClass as usize);

        if Q_stricmp((*scl).name.as_ptr(), (*target).healingclass) == 0 {
            //this thing can be healed by the class this player is using
            if (*target).healingDebounce < (*addr_of!(level)).time {
                //do the actual heal
                (*target).health += 10;
                if (*target).health > (*target).maxHealth {
                    //don't go too high
                    (*target).health = (*target).maxHealth;
                }
                (*target).healingDebounce = (*addr_of!(level)).time + (*target).healingrate;
                if !(*target).healingsound.is_null() && *(*target).healingsound != 0 {
                    //play it
                    let sound = CStr::from_ptr((*target).healingsound)
                        .to_str()
                        .unwrap_or_default();
                    if (*target).s.solid == SOLID_BMODEL {
                        //ok, well, just play it on the client then.
                        G_Sound(ent, CHAN_AUTO, G_SoundIndex(sound));
                    } else {
                        G_Sound(target, CHAN_AUTO, G_SoundIndex(sound));
                    }
                }

                //update net health for bar
                G_ScaleNetHealth(target);
                if !(*target).target_ent.is_null() && (*(*target).target_ent).maxHealth != 0 {
                    (*(*target).target_ent).health = (*target).health;
                    G_ScaleNetHealth((*target).target_ent);
                }
            }

            //keep them in the healing anim even when the healing debounce is not yet expired
            if (*(*ent).client).ps.torsoAnim == BOTH_BUTTON_HOLD
                || (*(*ent).client).ps.torsoAnim == BOTH_CONSOLE1
            {
                //extend the time
                (*(*ent).client).ps.torsoTimer = 500;
            } else {
                G_SetAnim(
                    ent,
                    null_mut(),
                    SETANIM_TORSO,
                    BOTH_BUTTON_HOLD,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }

            return QTRUE;
        }
    }

    QFALSE
}

/// `#define USE_DISTANCE 64.0f` (g_utils.c:1675) — reach of the [`TryUse`] forward trace.
const USE_DISTANCE: f32 = 64.0f32;

/*
==============
TryUse

Try and use an entity in the world, directly ahead of us
==============
*/
/// `void TryUse( gentity_t *ent )` (g_utils.c:1681).
///
/// The use-key handler: trace `USE_DISTANCE` ahead of `ent`'s view and act on whatever it
/// finds — eject/board a vehicle, fire a held dispenser at a teammate, trigger a `func_button`
/// or other usable, heal, or (falling through the C `tryJetPack` label) toggle the jetpack /
/// spew ammo. No oracle: branches on entity-state, the global `g_entities`/`level` arrays, traps,
/// and vehicle fn-pointers.
///
/// The C `goto tryJetPack` is rendered as `break 'try_use` out of the labeled `'try_use` block,
/// which falls through to the jetpack tail just as the C label does.
///
/// # Safety
/// `ent` may be NULL or any pointer; pointers it dereferences must be valid when non-NULL.
pub unsafe fn TryUse(ent: *mut gentity_t) {
    let target: *mut gentity_t;
    let mut src: vec3_t = [0.0; 3];
    let mut dest: vec3_t = [0.0; 3];
    let mut vf: vec3_t = [0.0; 3];
    let mut viewspot: vec3_t = [0.0; 3];

    // static vec3_t playerMins = {-15, -15, DEFAULT_MINS_2};
    // static vec3_t playerMaxs = {15, 15, DEFAULT_MAXS_2};
    let player_mins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2 as vec_t];
    let player_maxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as vec_t];

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && *addr_of!(gSiegeRoundBegun) == QFALSE {
        //nothing can be used til the round starts.
        return;
    }

    if ent.is_null()
        || (*ent).client.is_null()
        || ((*(*ent).client).ps.weaponTime > 0
            && (*(*ent).client).ps.torsoAnim != BOTH_BUTTON_HOLD
            && (*(*ent).client).ps.torsoAnim != BOTH_CONSOLE1)
        || (*ent).health < 1
        || ((*(*ent).client).ps.pm_flags & PMF_FOLLOW) != 0
        || (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR
        || ((*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
            && (*(*ent).client).ps.forceHandExtend != HANDEXTEND_DRAGGING)
    {
        return;
    }

    if (*(*ent).client).ps.emplacedIndex != 0 {
        //on an emplaced gun or using a vehicle, don't do anything when hitting use key
        return;
    }

    if (*ent).s.number < MAX_CLIENTS as c_int
        && !(*ent).client.is_null()
        && (*(*ent).client).ps.m_iVehicleNum != 0
    {
        let currentVeh = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*(*ent).client).ps.m_iVehicleNum as usize);
        if (*currentVeh).inuse != QFALSE && !(*currentVeh).m_pVehicle.is_null() {
            let pVeh = (*currentVeh).m_pVehicle;
            if (*pVeh).m_iBoarding == 0 {
                ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, ent as *mut bgEntity_t, QFALSE);
            }
            return;
        }
    }

    'try_use: {
        if (*(*ent).client).jetPackOn != QFALSE {
            //can't use anything else to jp is off
            break 'try_use;
        }

        if (*(*ent).client).bodyGrabIndex != ENTITYNUM_NONE {
            //then hitting the use key just means let go
            if (*(*ent).client).bodyGrabTime < (*addr_of!(level)).time {
                let grabbed = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).client).bodyGrabIndex as usize);

                if (*grabbed).inuse != QFALSE {
                    if !(*grabbed).client.is_null() {
                        (*(*grabbed).client).ps.ragAttach = 0;
                    } else {
                        (*grabbed).s.ragAttach = 0;
                    }
                }
                (*(*ent).client).bodyGrabIndex = ENTITYNUM_NONE;
                (*(*ent).client).bodyGrabTime = (*addr_of!(level)).time + 1000;
            }
            return;
        }

        VectorCopy(&(*(*ent).client).ps.origin, &mut viewspot);
        viewspot[2] += (*(*ent).client).ps.viewheight as vec_t;

        VectorCopy(&viewspot, &mut src);
        AngleVectors(&(*(*ent).client).ps.viewangles, Some(&mut vf), None, None);

        VectorMA(&src, USE_DISTANCE, &vf, &mut dest);

        //Trace ahead to find a valid target
        let trace = trap::Trace(
            &src,
            &vec3_origin,
            &vec3_origin,
            &dest,
            (*ent).s.number,
            MASK_OPAQUE | CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_ITEM | CONTENTS_CORPSE,
        );

        if trace.fraction == 1.0f32 || (trace.entityNum as c_int) < 1 {
            break 'try_use;
        }

        target =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);

        //Enable for corpse dragging
        // (C #if 0 block omitted)

        if !target.is_null()
            && !(*target).m_pVehicle.is_null()
            && !(*target).client.is_null()
            && (*target).s.NPC_class == CLASS_VEHICLE
            && (*(*ent).client).ps.zoomMode == 0
        {
            //if target is a vehicle then perform appropriate checks
            let pVeh = (*target).m_pVehicle;

            if !(*pVeh).m_pVehicleInfo.is_null() {
                if (*ent).r.ownerNum == (*target).s.number {
                    //user is already on this vehicle so eject him
                    ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(
                        pVeh,
                        ent as *mut bgEntity_t,
                        QFALSE,
                    );
                } else {
                    // Otherwise board this vehicle.
                    if (*addr_of!(g_gametype)).integer < GT_TEAM
                        || (*target).alliedTeam == 0
                        || (*target).alliedTeam == (*(*ent).client).sess.sessionTeam
                    {
                        //not belonging to a team, or client is on same team
                        ((*(*pVeh).m_pVehicleInfo).Board.unwrap())(pVeh, ent as *mut bgEntity_t);
                    }
                }
                //clear the damn button!
                (*(*ent).client).pers.cmd.buttons &= !BUTTON_USE;
                return;
            }
        }

        // (C #if 0 "ye olde method" block omitted)
        if (((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_HEALTHDISP)) != 0
            || ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_AMMODISP)) != 0)
            && !target.is_null()
            && (*target).inuse != QFALSE
            && !(*target).client.is_null()
            && (*target).health > 0
            && OnSameTeam(ent, target) != QFALSE
            && (G_CanUseDispOn(target, HI_HEALTHDISP) != 0
                || G_CanUseDispOn(target, HI_AMMODISP) != 0)
        {
            //a live target that's on my team, we can use him
            if G_CanUseDispOn(target, HI_HEALTHDISP) != 0 {
                G_UseDispenserOn(ent, HI_HEALTHDISP, target);
            }
            if G_CanUseDispOn(target, HI_AMMODISP) != 0 {
                G_UseDispenserOn(ent, HI_AMMODISP, target);
            }

            //for now, we will use the standard use anim
            if (*(*ent).client).ps.torsoAnim == BOTH_BUTTON_HOLD {
                //extend the time
                (*(*ent).client).ps.torsoTimer = 500;
            } else {
                G_SetAnim(
                    ent,
                    null_mut(),
                    SETANIM_TORSO,
                    BOTH_BUTTON_HOLD,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
            (*(*ent).client).ps.weaponTime = (*(*ent).client).ps.torsoTimer;
            return;
        }

        //Check for a use command
        if ValidUseTarget(target) != QFALSE
            && ((*addr_of!(g_gametype)).integer != GT_SIEGE
                || (*target).alliedTeam == 0
                || (*target).alliedTeam != (*(*ent).client).sess.sessionTeam
                || (*addr_of!(g_ff_objectives)).integer != 0)
        {
            if (*(*ent).client).ps.torsoAnim == BOTH_BUTTON_HOLD
                || (*(*ent).client).ps.torsoAnim == BOTH_CONSOLE1
            {
                //extend the time
                (*(*ent).client).ps.torsoTimer = 500;
            } else {
                G_SetAnim(
                    ent,
                    null_mut(),
                    SETANIM_TORSO,
                    BOTH_BUTTON_HOLD,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
            (*(*ent).client).ps.weaponTime = (*(*ent).client).ps.torsoTimer;
            /*
            NPC_SetAnim( ent, SETANIM_TORSO, BOTH_FORCEPUSH, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            if ( !VectorLengthSquared( ent->client->ps.velocity ) )
            {
                NPC_SetAnim( ent, SETANIM_LEGS, BOTH_FORCEPUSH, SETANIM_FLAG_NORMAL|SETANIM_FLAG_HOLD );
            }
            */
            // target->touch == Touch_Button — fn-pointer identity via fn_addr_eq
            let touch_is_button = match (*target).touch {
                Some(t) => core::ptr::fn_addr_eq(
                    t,
                    Touch_Button
                        as unsafe extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t),
                ),
                None => false,
            };
            if touch_is_button {
                //pretend we touched it
                ((*target).touch.unwrap())(target, ent, null_mut());
            } else {
                GlobalUse(target, ent, ent);
            }
            return;
        }

        if TryHeal(ent, target) != QFALSE {
            return;
        }
    } // tryJetPack:

    //if we got here, we didn't actually use anything else, so try to toggle jetpack if we are in the air, or if it is already on
    if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_JETPACK)) != 0 {
        if (*(*ent).client).jetPackOn != QFALSE
            || (*(*ent).client).ps.groundEntityNum == ENTITYNUM_NONE
        {
            ItemUse_Jetpack(ent);
            return;
        }
    }

    if ((*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_AMMODISP)) != 0
    /*&&
    G_ItemUsable(&ent->client->ps, HI_AMMODISP)*/
    {
        //if you used nothing, then try spewing out some ammo
        let mut fAng: vec3_t = [0.0; 3];
        let mut fwd: vec3_t = [0.0; 3];

        VectorSet(
            &mut fAng,
            0.0f32,
            (*(*ent).client).ps.viewangles[YAW],
            0.0f32,
        );
        AngleVectors(&fAng, Some(&mut fwd), None, None);

        let origin = (*(*ent).client).ps.origin;
        // VectorMA(ent->client->ps.origin, 64.0f, fwd, fwd) — in/out alias `fwd`; via a temp.
        let fwd_in = fwd;
        VectorMA(&origin, 64.0f32, &fwd_in, &mut fwd);
        let trToss = trap::Trace(
            &origin,
            &player_mins,
            &player_maxs,
            &fwd,
            (*ent).s.number,
            (*ent).clipmask,
        );
        if trToss.fraction == 1.0f32 && trToss.allsolid == 0 && trToss.startsolid == 0 {
            ItemUse_UseDisp(ent, HI_AMMODISP);
            G_AddEvent(ent, EV_USE_ITEM0 + HI_AMMODISP, 0);
            return;
        }
    }
}

/// `int G_CanUseDispOn( gentity_t *ent, int dispType )` (g_utils.c:1571) — see if
/// this guy needs servicing from a specific type of dispenser.
///
/// Returns 0 for a dead/invalid `ent`. For `HI_HEALTHDISP`, 1 iff hurt
/// (`STAT_HEALTH < STAT_MAX_HEALTH`). For `HI_AMMODISP`, 0 if the current weapon
/// is not a player-useable weapon, else 1 iff the weapon's ammo slot is below the
/// `ammoData` cap. Any other `dispType` returns 0.
///
/// No oracle: branches on `gclient` playerstate and the `weaponData`/`ammoData`
/// tables, not a pure computation over plain inputs.
pub unsafe fn G_CanUseDispOn(ent: *mut gentity_t, dispType: c_int) -> c_int {
    if (*ent).client.is_null()
        || (*ent).inuse == QFALSE
        || (*ent).health < 1
        || (*(*ent).client).ps.stats[STAT_HEALTH as usize] < 1
    {
        //dead or invalid
        return 0;
    }

    if dispType == HI_HEALTHDISP {
        if (*(*ent).client).ps.stats[STAT_HEALTH as usize]
            < (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize]
        {
            //he's hurt
            return 1;
        }

        //otherwise no
        return 0;
    } else if dispType == HI_AMMODISP {
        if (*(*ent).client).ps.weapon <= WP_NONE || (*(*ent).client).ps.weapon > LAST_USEABLE_WEAPON
        {
            //not a player-useable weapon
            return 0;
        }

        let weapon = (*(*ent).client).ps.weapon as usize;
        let ammoIdx = weaponData[weapon].ammoIndex as usize;
        if (*(*ent).client).ps.ammo[ammoIdx] < ammoData[ammoIdx].max {
            //needs more ammo for current weapon
            return 1;
        }

        //needs none
        return 0;
    }

    //invalid type?
    0
}

/// `void G_UseDispenserOn( gentity_t *ent, int dispType, gentity_t *target )`
/// (g_utils.c:1537) — use an ammo/health dispenser on another client.
///
/// For `HI_HEALTHDISP`, bump the target's health by 4 (capped at its
/// `STAT_MAX_HEALTH`), flag `isMedHealed`, and mirror the stat back onto the
/// entity `health`. For `HI_AMMODISP`, on the dispensing `ent`'s
/// `medSupplyDebounce`, add the current weapon's `energyPerShot` worth of ammo
/// (capped at the ammo table `max`) and rearm the debounce by the weapon's
/// `fireTime`; always refresh the target's `isMedSupplied` flag.
///
/// No oracle: pure entity/`gclient` playerstate mutation reading the engine's
/// `level.time` plus the `weaponData`/`ammoData` tables — not a computable pure
/// function.
pub unsafe fn G_UseDispenserOn(ent: *mut gentity_t, dispType: c_int, target: *mut gentity_t) {
    if dispType == HI_HEALTHDISP {
        (*(*target).client).ps.stats[STAT_HEALTH as usize] += 4;

        if (*(*target).client).ps.stats[STAT_HEALTH as usize]
            > (*(*target).client).ps.stats[STAT_MAX_HEALTH as usize]
        {
            (*(*target).client).ps.stats[STAT_HEALTH as usize] =
                (*(*target).client).ps.stats[STAT_MAX_HEALTH as usize];
        }

        (*(*target).client).isMedHealed = (*addr_of!(level)).time + 500;
        (*target).health = (*(*target).client).ps.stats[STAT_HEALTH as usize];
    } else if dispType == HI_AMMODISP {
        if (*(*ent).client).medSupplyDebounce < (*addr_of!(level)).time {
            //do the next increment
            //increment based on the amount of ammo used per normal shot.
            let weapon = (*(*target).client).ps.weapon as usize;
            let ammoIdx = weaponData[weapon].ammoIndex as usize;
            (*(*target).client).ps.ammo[ammoIdx] += weaponData[weapon].energyPerShot;

            if (*(*target).client).ps.ammo[ammoIdx] > ammoData[ammoIdx].max {
                //cap it off
                (*(*target).client).ps.ammo[ammoIdx] = ammoData[ammoIdx].max;
            }

            //base the next supply time on how long the weapon takes to fire. Seems fair enough.
            (*(*ent).client).medSupplyDebounce =
                (*addr_of!(level)).time + weaponData[weapon].fireTime;
        }
        (*(*target).client).isMedSupplied = (*addr_of!(level)).time + 500;
    }
}

// =====================================================================================
// `void G_ClPtrClear(void)` — g_utils.c:450 (inside `#ifdef _XBOX` ... `#endif`)
// -------------------------------------------------------------------------------------
// FALSE-POSITIVE (#ifdef-excluded). `ported_index.py` flags `G_ClPtrClear` as missing,
// but the C definition sits under `#ifdef _XBOX`, so it is compiled out of the PC /
// dedicated-server ABI target this port reproduces — there is no live PC implementation
// to point at, and intentionally so. It would zero the whole `gClPtrs` fake-client pool
// (defined live just above as the module static `gClPtrs`, ~g_utils.rs:1031). Faithful
// translation of the C body, for 1:1-file self-documentation only (not built):
//
// // #[cfg(target_xbox)]  // no such target; _XBOX is never defined for this build
// pub unsafe fn G_ClPtrClear() {
//     for i in 0..MAX_GENTITIES {
//         gClPtrs[i] = null_mut();
//     }
// }
// =====================================================================================

/// `void G_CleanAllFakeClients( void )` (g_utils.c:459) — call this on game
/// shutdown to run through and get rid of all the lingering client pointers.
///
/// Walks `g_entities` from `MAX_CLIENTS` (all ents below have real client structs)
/// to `MAX_GENTITIES`, freeing the dynamically-allocated `client` of every in-use
/// `ET_NPC` via [`G_FreeFakeClient`].
///
/// No oracle: iterates the engine-owned `g_entities` array and frees module-owned
/// memory; pure side-effecting entity-state cleanup.
/// `int DebugLine( vec3_t start, vec3_t end, int color )` (g_utils.c:2074) — draw
/// a debug line as a thin quad via the engine's debug-polygon trap.
///
/// ```text
/// ================
/// DebugLine
///
///   debug polygons only work when running a local game
///   with r_debugSurface set to 2
/// ================
/// ```
pub fn DebugLine(start: &vec3_t, end: &vec3_t, color: c_int) -> c_int {
    let mut points: [vec3_t; 4] = [[0.0; 3]; 4];
    let mut dir: vec3_t = [0.0; 3];
    let mut cross: vec3_t = [0.0; 3];
    let up: vec3_t = [0.0, 0.0, 1.0];
    let dot: f32;

    VectorCopy(start, &mut points[0]);
    VectorCopy(start, &mut points[1]);
    //points[1][2] -= 2;
    VectorCopy(end, &mut points[2]);
    //points[2][2] -= 2;
    VectorCopy(end, &mut points[3]);

    VectorSubtract(end, start, &mut dir);
    VectorNormalize(&mut dir);
    dot = DotProduct(&dir, &up);
    if dot > 0.99 || dot < -0.99 {
        VectorSet(&mut cross, 1.0, 0.0, 0.0);
    } else {
        CrossProduct(&dir, &up, &mut cross);
    }

    VectorNormalize(&mut cross);

    let (p0, p1, p2, p3) = (points[0], points[1], points[2], points[3]);
    VectorMA(&p0, 2.0, &cross, &mut points[0]);
    VectorMA(&p1, -2.0, &cross, &mut points[1]);
    VectorMA(&p2, -2.0, &cross, &mut points[2]);
    VectorMA(&p3, 2.0, &cross, &mut points[3]);

    trap::DebugPolygonCreate(color, 4, points.as_mut_ptr())
}

/// `void G_ROFF_NotetrackCallback( gentity_t *cent, const char *notetrack )`
/// (g_utils.c:2102) — parse a ROFF notetrack command and act on it. Only the
/// `"loop"` command is handled here; an additional argument means reset to the
/// original position before looping.
pub unsafe fn G_ROFF_NotetrackCallback(cent: *mut gentity_t, notetrack: *const c_char) {
    let mut type_: [c_char; 256] = [0; 256];
    let mut i: usize = 0;
    let mut addlArg: c_int = 0;

    if cent.is_null() || notetrack.is_null() {
        return;
    }

    while *notetrack.add(i) != 0 && *notetrack.add(i) != b' ' as c_char {
        type_[i] = *notetrack.add(i);
        i += 1;
    }

    type_[i] = b'\0' as c_char;

    if i == 0 || type_[0] == 0 {
        return;
    }

    if *notetrack.add(i) == b' ' as c_char {
        addlArg = 1;
    }

    // strcmp(type, "loop") == 0
    if &type_[..4]
        == &[
            b'l' as c_char,
            b'o' as c_char,
            b'o' as c_char,
            b'p' as c_char,
        ]
        && type_[4] == 0
    {
        if addlArg != 0
        //including an additional argument means reset to original position before loop
        {
            let origin2 = (*cent).s.origin2;
            let angles2 = (*cent).s.angles2;
            VectorCopy(&origin2, &mut (*cent).s.pos.trBase);
            VectorCopy(&origin2, &mut (*cent).r.currentOrigin);
            VectorCopy(&angles2, &mut (*cent).s.apos.trBase);
            VectorCopy(&angles2, &mut (*cent).r.currentAngles);
        }

        trap::ROFF_Play((*cent).s.number, (*cent).roffid, QFALSE);
    }
}

pub unsafe fn G_CleanAllFakeClients() {
    let mut i: c_int = MAX_CLIENTS as c_int; //start off here since all ents below have real client structs.
    let mut ent: *mut gentity_t;

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    while i < MAX_GENTITIES as c_int {
        ent = base.add(i as usize);

        if (*ent).inuse != QFALSE && (*ent).s.eType == ET_NPC && !(*ent).client.is_null() {
            G_FreeFakeClient(addr_of_mut!((*ent).client));
        }
        i += 1;
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;

    /// `G_SetMovedir` over the two up/down sentinels and an assortment of generic
    /// yaw/pitch angles (the generic case delegates to the already-bit-exact-verified
    /// `AngleVectors`). Both the produced `movedir` and the cleared `angles` are
    /// checked against the real C.
    #[test]
    fn g_setmovedir_matches_oracle() {
        let cases: &[vec3_t] = &[
            [0.0, -1.0, 0.0], // VEC_UP sentinel
            [0.0, -2.0, 0.0], // VEC_DOWN sentinel
            [0.0, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [0.0, 45.0, 0.0],
            [30.0, 120.0, 10.0],
            [-15.0, -200.0, 0.0],
            [0.0, -1.0, 0.001], // near-but-not VEC_UP -> generic path
        ];
        for case in cases {
            let mut r_ang = *case;
            let mut r_dir: vec3_t = [0.0; 3];
            G_SetMovedir(&mut r_ang, &mut r_dir);

            let mut c_ang = *case;
            let mut c_dir: vec3_t = [0.0; 3];
            unsafe { oracle::jka_G_SetMovedir(c_ang.as_mut_ptr(), c_dir.as_mut_ptr()) };

            assert_eq!(r_dir, c_dir, "movedir mismatch for {case:?}");
            assert_eq!(r_ang, c_ang, "cleared angles mismatch for {case:?}");
        }
    }

    #[test]
    fn g_pointinbounds_matches_oracle() {
        let mins: vec3_t = [-10.0, -20.0, -30.0];
        let maxs: vec3_t = [10.0, 20.0, 30.0];
        let points: &[vec3_t] = &[
            [0.0, 0.0, 0.0],
            [-10.0, -20.0, -30.0], // on the min corner (inclusive)
            [10.0, 20.0, 30.0],    // on the max corner (inclusive)
            [-10.1, 0.0, 0.0],     // just outside x-min
            [0.0, 20.1, 0.0],      // just outside y-max
            [0.0, 0.0, 31.0],
            [5.0, -5.0, 15.0],
        ];
        for p in points {
            let got = G_PointInBounds(p, &mins, &maxs);
            let want =
                unsafe { oracle::jka_G_PointInBounds(p.as_ptr(), mins.as_ptr(), maxs.as_ptr()) };
            assert_eq!(got as c_int, want, "G_PointInBounds mismatch for {p:?}");
        }
    }

    #[test]
    fn g_boxinbounds_matches_oracle() {
        let bmins: vec3_t = [-100.0, -100.0, -100.0];
        let bmaxs: vec3_t = [100.0, 100.0, 100.0];
        let mins: vec3_t = [-5.0, -5.0, -5.0];
        let maxs: vec3_t = [5.0, 5.0, 5.0];
        let points: &[vec3_t] = &[
            [0.0, 0.0, 0.0],   // centred, fully contained
            [95.0, 0.0, 0.0],  // box max pokes past bounds max in x
            [-95.0, 0.0, 0.0], // box min pokes past bounds min in x
            [0.0, 96.0, 0.0],
            [0.0, 0.0, -96.0],
            [95.0, 95.0, 95.0], // corner, just contained
            [96.0, 0.0, 0.0],   // just over
        ];
        for p in points {
            let got = G_BoxInBounds(p, &mins, &maxs, &bmins, &bmaxs);
            let want = unsafe {
                oracle::jka_G_BoxInBounds(
                    p.as_ptr(),
                    mins.as_ptr(),
                    maxs.as_ptr(),
                    bmins.as_ptr(),
                    bmaxs.as_ptr(),
                )
            };
            assert_eq!(got as c_int, want, "G_BoxInBounds mismatch for {p:?}");
        }
    }

    /// `AddRemap` (incl. updating an existing entry) + `BuildShaderStateConfig`, driven
    /// in lockstep against the oracle from an empty table. Exercises the `%5.2f` render
    /// across sub-1, two-digit, zero, negative, and width-overflowing offsets. Only this
    /// test touches the remap globals, so no lock is needed.
    #[test]
    fn g_remap_matches_oracle() {
        use std::ffi::{CStr, CString};
        unsafe {
            // reset both tables to empty
            *addr_of_mut!(remapCount) = 0;
            core::ptr::write_bytes(
                addr_of_mut!(remappedShaders) as *mut shaderRemap_t,
                0,
                MAX_SHADER_REMAPS,
            );
            oracle::jka_ResetRemaps();

            let seq: &[(&str, &str, f32)] = &[
                ("tex/red01", "newred", 1.5),
                ("tex/blue01", "newblue", 12.34),
                ("tex/red01", "updatedred", 99.99), // updates the existing red01 entry
                ("tex/green", "g", 0.0),
                ("tex/neg", "n", -1.25),
                ("tex/big", "b", 123.456), // overflows the %5 field width
            ];
            for (old, new, off) in seq {
                let oc = CString::new(*old).unwrap();
                let nc = CString::new(*new).unwrap();
                AddRemap(oc.as_ptr(), nc.as_ptr(), *off);
                oracle::jka_AddRemap(oc.as_ptr(), nc.as_ptr(), *off);
            }

            let got = CStr::from_ptr(BuildShaderStateConfig());
            let want = CStr::from_ptr(oracle::jka_BuildShaderStateConfig());
            assert_eq!(
                got.to_bytes(),
                want.to_bytes(),
                "shader-state config mismatch"
            );
        }
    }

    /// `GetAnglesForDirection` over an assortment of direction vectors (axis-aligned,
    /// diagonal, zero, and steep pitch) — checks the produced Euler angles bit-for-bit
    /// against the real C (the `vectoangles` atan2 path included).
    #[test]
    fn g_getanglesfordirection_matches_oracle() {
        let cases: &[(vec3_t, vec3_t)] = &[
            ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
            ([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
            ([0.0, 0.0, 0.0], [0.0, 0.0, 10.0]),
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]), // degenerate
            ([5.0, 5.0, 5.0], [5.0, 5.0, 5.0]), // identical -> zero dir
            ([1.0, 2.0, 3.0], [-4.0, 7.0, -2.0]),
            ([100.0, -50.0, 25.0], [10.0, 10.0, 200.0]),
            ([0.0, 0.0, 0.0], [-3.0, -3.0, 0.0]),
        ];
        for (p1, p2) in cases {
            let mut r_out: vec3_t = [0.0; 3];
            GetAnglesForDirection(p1, p2, &mut r_out);

            let mut c_out: vec3_t = [0.0; 3];
            unsafe {
                oracle::jka_GetAnglesForDirection(p1.as_ptr(), p2.as_ptr(), c_out.as_mut_ptr())
            };

            assert_eq!(
                r_out, c_out,
                "GetAnglesForDirection mismatch for {p1:?}->{p2:?}"
            );
        }
    }

    /// `ShortestLineSegBewteen2LineSegs` across the branch space: skew segments (interior
    /// `s`/`t` solution), clamped `s`/`t` (endpoint fallback), parallel/degenerate
    /// segments (the `Q3_INFINITE` path), and a zero-length segment. Both the returned
    /// distance and the two closest points are checked bit-for-bit against the real C.
    #[test]
    fn g_shortestlineseg_matches_oracle() {
        // (start1, end1, start2, end2)
        let cases: &[(vec3_t, vec3_t, vec3_t, vec3_t)] = &[
            // skew lines crossing near the middle -> interior solution
            (
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, -5.0, 2.0],
                [5.0, 5.0, 2.0],
            ),
            // parallel segments -> denom ~ 0, infinity path then endpoint sweep
            (
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [10.0, 4.0, 0.0],
            ),
            // colinear, disjoint
            (
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [5.0, 0.0, 0.0],
            ),
            // clamped: closest pair is an endpoint vs interior
            (
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [20.0, 1.0, 0.0],
                [20.0, -1.0, 0.0],
            ),
            // second segment zero-length (a point)
            (
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 3.0, 0.0],
                [5.0, 3.0, 0.0],
            ),
            // general 3D skew
            (
                [1.0, 2.0, 3.0],
                [-4.0, 7.0, -2.0],
                [0.0, -1.0, 5.0],
                [6.0, 2.0, -3.0],
            ),
            // identical segments
            (
                [2.0, 2.0, 2.0],
                [8.0, 8.0, 8.0],
                [2.0, 2.0, 2.0],
                [8.0, 8.0, 8.0],
            ),
        ];
        for (s1, e1, s2, e2) in cases {
            let mut r_c1: vec3_t = [0.0; 3];
            let mut r_c2: vec3_t = [0.0; 3];
            let r_dist = ShortestLineSegBewteen2LineSegs(s1, e1, s2, e2, &mut r_c1, &mut r_c2);

            let mut c_c1: vec3_t = [0.0; 3];
            let mut c_c2: vec3_t = [0.0; 3];
            let c_dist = unsafe {
                oracle::jka_ShortestLineSegBewteen2LineSegs(
                    s1.as_ptr(),
                    e1.as_ptr(),
                    s2.as_ptr(),
                    e2.as_ptr(),
                    c_c1.as_mut_ptr(),
                    c_c2.as_mut_ptr(),
                )
            };

            assert_eq!(
                r_dist.to_bits(),
                c_dist.to_bits(),
                "dist mismatch for {s1:?}/{e1:?} {s2:?}/{e2:?}"
            );
            assert_eq!(
                r_c1, c_c1,
                "close_pnt1 mismatch for {s1:?}/{e1:?} {s2:?}/{e2:?}"
            );
            assert_eq!(
                r_c2, c_c2,
                "close_pnt2 mismatch for {s1:?}/{e1:?} {s2:?}/{e2:?}"
            );
        }
    }
}
