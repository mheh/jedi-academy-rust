//! `g_spawn.c` — entity spawning + the spawn-string parser support.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_spawn.c`. **Partial port:** the
//! string-allocation helper `G_NewString`, the spawn-string *readers*
//! `G_SpawnString`/`G_SpawnFloat`/`G_SpawnInt`/`G_SpawnVector`, and the spawn-var
//! *parser* chain `G_ParseSpawnVars` + its writers `G_AddSpawnVarToken`/
//! `AddSpawnField`/`HandleEntityAdjustment` have landed — the foundation every `SP_*`
//! spawn function and the field parsers (`BG_ParseField`, the vehicle/saber loaders)
//! build their key/value lookups on. The class dispatch table (`spawns[]`) and
//! [`G_CallSpawn`] have landed too, and now the `fields[]` field-offset table
//! (g_spawn.c:54) plus the full entity spawner proper
//! (`G_SpawnGEntityFromSpawnVars`, `SP_worldspawn`, `G_SpawnEntitiesFromString`).

#![allow(non_snake_case)] // C function names (`G_NewString`) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::mem::{offset_of, size_of};
use core::ptr::{addr_of, addr_of_mut, copy_nonoverlapping, null_mut};

use crate::codemp::game::bg_misc::{bg_itemlist, BG_FindItem, BG_ParseField};
use crate::codemp::game::bg_panimate::{bgHumanoidAnimations, BGPAFtextLoaded, BG_ParseAnimationFile};
use crate::codemp::game::bg_public::{
    gitem_t, BG_field_t, CS_GAME_VERSION, CS_GLOBAL_AMBIENT_SET, CS_LEVEL_START_TIME,
    CS_LIGHT_STYLES, CS_MESSAGE, CS_MOTD, CS_MUSIC, CS_WARMUP, EF_PERMANENT, ET_MOVER, F_ANGLEHACK,
    F_FLOAT, F_IGNORE,
    F_INT, F_LSTRING, F_PARM1, F_PARM10, F_PARM11, F_PARM12, F_PARM13, F_PARM14, F_PARM15, F_PARM16,
    F_PARM2, F_PARM3, F_PARM4, F_PARM5, F_PARM6, F_PARM7, F_PARM8, F_PARM9, F_VECTOR, GAME_VERSION,
    GT_FFA, GT_MAX_GAME_TYPE, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM, MAX_SPAWN_VARS,
    MAX_SPAWN_VARS_CHARS, TEAM_BLUE, TEAM_RED,
};
use crate::codemp::game::g_client::g2SaberInstance;
use crate::codemp::game::g_items::{EWebPrecache, G_SpawnItem};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_public_h::{
    BSET_ANGER, BSET_ATTACK, BSET_AWAKE, BSET_BLOCKED, BSET_DEATH, BSET_DELAYED, BSET_FFDEATH,
    BSET_FFIRE, BSET_FLEE, BSET_LOSTENEMY, BSET_MINDTRICK, BSET_PAIN, BSET_SPAWN, BSET_USE,
    BSET_VICTORY,
};
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_motd, g_restarted, level, Com_Error, G_Error, G_Printf,
};
use crate::codemp::game::g_mem::G_Alloc;
use crate::codemp::game::g_target::scriptrunner_run;
use crate::codemp::game::g_utils::{
    G_BSPIndex, G_FreeEntity, G_SetOrigin, G_SoundIndex, G_SoundSetIndex, G_Spawn,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::q_math::{VectorAdd, VectorCopy};
use crate::codemp::game::q_shared::{Com_sprintf, Q_stricmp, Q_strncmp, Sz};
use crate::codemp::game::q_shared_h::{
    vec3_t, DEG2RAD, ENTITYNUM_WORLD, ERR_DROP, MAX_AMBIENT_SETS, MAX_GENTITIES, MAX_QPATH,
    MAX_TOKEN_CHARS,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// The spawn-var readers below convert the matched value string the same way the C
// game module does — through libc `atof`/`atoi` and `sscanf` (the game build pulls
// these from <stdlib.h>/<stdio.h>). Calling the same functions keeps the parse
// byte-for-byte with the original's locale-default behavior. `strlen` likewise
// backs `G_AddSpawnVarToken`'s length math.
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    // `SP_gametype_item` keys off substrings of `targetname` via libc `strstr`
    // (<string.h>) exactly as the C game module does.
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    // `G_CallSpawn` matches classnames against `bg_itemlist`/`spawns[]` with libc
    // `strcmp` (<string.h>), exactly as the C game module does.
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// ===========================================================================
// fields[] field-offset table (g_spawn.c:54) — the worldspawn KEYSTONE.
//
// C is `BG_field_t fields[]` of `{name, FOFS(field), type}` triples, NULL-name
// terminated. `FOFS(x)` is `offsetof(gentity_t, x)`, reproduced here with
// `core::mem::offset_of!`. The C initializers leave the 4th member (`flags`) at 0,
// so every entry below sets `flags: 0`. `BG_ParseField` (bg_misc.rs) consumes this
// table; the terminator is a NULL `name`. Comments carried over verbatim.
// ===========================================================================

/// Convenience constructor for one [`BG_field_t`] row — keeps the table readable
/// and mirrors the C `{name, ofs, type}` braces (C leaves `flags` zero).
const fn field(name: &CStr, ofs: usize, ty: c_int) -> BG_field_t {
    BG_field_t {
        name: name.as_ptr() as *mut c_char,
        ofs: ofs as c_int,
        r#type: ty,
        flags: 0,
    }
}

// `BG_field_t` holds a raw `char *name`; the table is module-private and only read
// on the game thread, mirroring the C file-scope array.
unsafe impl Sync for FieldsTable {}
#[repr(transparent)]
struct FieldsTable([BG_field_t; 86]);

/// `BG_field_t fields[]` (g_spawn.c:54) — the spawn-key → gentity-field-offset map
/// scanned by [`BG_ParseField`]. Faithful 1:1 with the C table including its
/// `//`-comments and the trailing NULL-name terminator.
#[allow(non_upper_case_globals)] // C global name `fields` kept verbatim
static fields: FieldsTable = FieldsTable([
    field(c"classname", offset_of!(gentity_t, classname), F_LSTRING),
    field(c"teamnodmg", offset_of!(gentity_t, teamnodmg), F_INT),
    field(c"teamowner", offset_of!(gentity_t, s.teamowner), F_INT),
    field(c"teamuser", offset_of!(gentity_t, alliedTeam), F_INT),
    field(c"alliedTeam", offset_of!(gentity_t, alliedTeam), F_INT), //for misc_turrets
    field(c"roffname", offset_of!(gentity_t, roffname), F_LSTRING),
    field(c"rofftarget", offset_of!(gentity_t, rofftarget), F_LSTRING),
    field(c"healingclass", offset_of!(gentity_t, healingclass), F_LSTRING),
    field(c"healingsound", offset_of!(gentity_t, healingsound), F_LSTRING),
    field(c"healingrate", offset_of!(gentity_t, healingrate), F_INT),
    field(c"ownername", offset_of!(gentity_t, ownername), F_LSTRING),
    field(c"origin", offset_of!(gentity_t, s.origin), F_VECTOR),
    field(c"model", offset_of!(gentity_t, model), F_LSTRING),
    field(c"model2", offset_of!(gentity_t, model2), F_LSTRING),
    field(c"spawnflags", offset_of!(gentity_t, spawnflags), F_INT),
    field(c"speed", offset_of!(gentity_t, speed), F_FLOAT),
    field(c"target", offset_of!(gentity_t, target), F_LSTRING),
    field(c"target2", offset_of!(gentity_t, target2), F_LSTRING),
    field(c"target3", offset_of!(gentity_t, target3), F_LSTRING),
    field(c"target4", offset_of!(gentity_t, target4), F_LSTRING),
    field(c"target5", offset_of!(gentity_t, target5), F_LSTRING),
    field(c"target6", offset_of!(gentity_t, target6), F_LSTRING),
    field(c"NPC_targetname", offset_of!(gentity_t, NPC_targetname), F_LSTRING),
    field(c"NPC_target", offset_of!(gentity_t, NPC_target), F_LSTRING),
    field(c"NPC_target2", offset_of!(gentity_t, target2), F_LSTRING), //NPC_spawner only
    field(c"NPC_target4", offset_of!(gentity_t, target4), F_LSTRING), //NPC_spawner only
    field(c"NPC_type", offset_of!(gentity_t, NPC_type), F_LSTRING),
    field(c"targetname", offset_of!(gentity_t, targetname), F_LSTRING),
    field(c"message", offset_of!(gentity_t, message), F_LSTRING),
    field(c"team", offset_of!(gentity_t, team), F_LSTRING),
    field(c"wait", offset_of!(gentity_t, wait), F_FLOAT),
    field(c"delay", offset_of!(gentity_t, delay), F_INT),
    field(c"random", offset_of!(gentity_t, random), F_FLOAT),
    field(c"count", offset_of!(gentity_t, count), F_INT),
    field(c"health", offset_of!(gentity_t, health), F_INT),
    field(c"light", 0, F_IGNORE),
    field(c"dmg", offset_of!(gentity_t, damage), F_INT),
    field(c"angles", offset_of!(gentity_t, s.angles), F_VECTOR),
    field(c"angle", offset_of!(gentity_t, s.angles), F_ANGLEHACK),
    field(c"targetShaderName", offset_of!(gentity_t, targetShaderName), F_LSTRING),
    field(c"targetShaderNewName", offset_of!(gentity_t, targetShaderNewName), F_LSTRING),
    field(c"linear", offset_of!(gentity_t, alt_fire), F_INT), //for movers to use linear movement
    field(c"closetarget", offset_of!(gentity_t, closetarget), F_LSTRING), //for doors
    field(c"opentarget", offset_of!(gentity_t, opentarget), F_LSTRING), //for doors
    field(c"paintarget", offset_of!(gentity_t, paintarget), F_LSTRING), //for doors
    field(c"goaltarget", offset_of!(gentity_t, goaltarget), F_LSTRING), //for siege
    field(c"idealclass", offset_of!(gentity_t, idealclass), F_LSTRING), //for siege spawnpoints
    //rww - icarus stuff:
    field(c"spawnscript", offset_of!(gentity_t, behaviorSet) + BSET_SPAWN as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"usescript", offset_of!(gentity_t, behaviorSet) + BSET_USE as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"awakescript", offset_of!(gentity_t, behaviorSet) + BSET_AWAKE as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"angerscript", offset_of!(gentity_t, behaviorSet) + BSET_ANGER as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"attackscript", offset_of!(gentity_t, behaviorSet) + BSET_ATTACK as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"victoryscript", offset_of!(gentity_t, behaviorSet) + BSET_VICTORY as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"lostenemyscript", offset_of!(gentity_t, behaviorSet) + BSET_LOSTENEMY as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"painscript", offset_of!(gentity_t, behaviorSet) + BSET_PAIN as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"fleescript", offset_of!(gentity_t, behaviorSet) + BSET_FLEE as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"deathscript", offset_of!(gentity_t, behaviorSet) + BSET_DEATH as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"delayscript", offset_of!(gentity_t, behaviorSet) + BSET_DELAYED as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"delayscripttime", offset_of!(gentity_t, delayScriptTime), F_INT), //name of script to run
    field(c"blockedscript", offset_of!(gentity_t, behaviorSet) + BSET_BLOCKED as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"ffirescript", offset_of!(gentity_t, behaviorSet) + BSET_FFIRE as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"ffdeathscript", offset_of!(gentity_t, behaviorSet) + BSET_FFDEATH as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"mindtrickscript", offset_of!(gentity_t, behaviorSet) + BSET_MINDTRICK as usize * size_of::<*mut c_char>(), F_LSTRING), //name of script to run
    field(c"script_targetname", offset_of!(gentity_t, script_targetname), F_LSTRING), //scripts look for this when "affecting"
    field(c"fullName", offset_of!(gentity_t, fullName), F_LSTRING),
    field(c"soundSet", offset_of!(gentity_t, soundSet), F_LSTRING),
    field(c"radius", offset_of!(gentity_t, radius), F_FLOAT),
    field(c"numchunks", offset_of!(gentity_t, radius), F_FLOAT), //for func_breakables
    field(c"chunksize", offset_of!(gentity_t, mass), F_FLOAT), //for func_breakables
    //Script parms - will this handle clamping to 16 or whatever length of parm[0] is?
    field(c"parm1", 0, F_PARM1),
    field(c"parm2", 0, F_PARM2),
    field(c"parm3", 0, F_PARM3),
    field(c"parm4", 0, F_PARM4),
    field(c"parm5", 0, F_PARM5),
    field(c"parm6", 0, F_PARM6),
    field(c"parm7", 0, F_PARM7),
    field(c"parm8", 0, F_PARM8),
    field(c"parm9", 0, F_PARM9),
    field(c"parm10", 0, F_PARM10),
    field(c"parm11", 0, F_PARM11),
    field(c"parm12", 0, F_PARM12),
    field(c"parm13", 0, F_PARM13),
    field(c"parm14", 0, F_PARM14),
    field(c"parm15", 0, F_PARM15),
    field(c"parm16", 0, F_PARM16),
    // {NULL} terminator
    BG_field_t {
        name: null_mut(),
        ofs: 0,
        r#type: 0,
        flags: 0,
    },
]);

/// `qboolean G_SpawnString( const char *key, const char *defaultString, char **out )`
/// (g_spawn.c:6).
///
/// Looks `key` up in the global `level.spawnVars[]` key/value table (filled by
/// [`G_ParseSpawnVars`]). On a case-insensitive [`Q_stricmp`] match,
/// points `*out` at the stored value and returns `qtrue`; otherwise writes
/// `defaultString` and returns `qfalse`.
///
/// Faithful quirk carried over: when `!level.spawning` the original pre-seeds
/// `*out = defaultString` but *still* scans the table (it does not early-return),
/// so a stale entry can win. No oracle — it reads the mutable global `level`.
///
/// # Safety
/// `key`/`defaultString` must be NUL-terminated; `out` must be a valid writable
/// `char **`. Must run on the game thread that owns `level`.
pub unsafe fn G_SpawnString(
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut *mut c_char,
) -> qboolean {
    let lvl = addr_of!(level);

    if (*lvl).spawning == QFALSE {
        *out = defaultString as *mut c_char;
    }

    let mut i: c_int = 0;
    while i < (*lvl).numSpawnVars {
        if Q_stricmp(key, (*lvl).spawnVars[i as usize][0]) == 0 {
            *out = (*lvl).spawnVars[i as usize][1];
            return QTRUE;
        }
        i += 1;
    }

    *out = defaultString as *mut c_char;
    QFALSE
}

/// `qboolean G_SpawnFloat( const char *key, const char *defaultString, float *out )`
/// (g_spawn.c:25).
///
/// [`G_SpawnString`] then `atof`. Returns the present flag from the lookup; `*out`
/// is the parsed float either way (from the matched value or the default).
///
/// # Safety
/// As [`G_SpawnString`]; `out` must be a valid writable `float *`.
pub unsafe fn G_SpawnFloat(
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut f32,
) -> qboolean {
    let mut s: *mut c_char = null_mut();
    let present = G_SpawnString(key, defaultString, &mut s);
    *out = atof(s) as f32;
    present
}

/// `qboolean G_SpawnInt( const char *key, const char *defaultString, int *out )`
/// (g_spawn.c:34).
///
/// [`G_SpawnString`] then `atoi`. Returns the present flag; `*out` is the parsed int.
///
/// # Safety
/// As [`G_SpawnString`]; `out` must be a valid writable `int *`.
pub unsafe fn G_SpawnInt(
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut c_int,
) -> qboolean {
    let mut s: *mut c_char = null_mut();
    let present = G_SpawnString(key, defaultString, &mut s);
    *out = atoi(s);
    present
}

/// `qboolean G_SpawnVector( const char *key, const char *defaultString, float *out )`
/// (g_spawn.c:43).
///
/// [`G_SpawnString`] then `sscanf( s, "%f %f %f", &out[0..2] )`. Returns the present
/// flag. Faithful to the original: only the components `sscanf` successfully parses
/// are written, so a short value string leaves the trailing `out[]` slots untouched.
///
/// # Safety
/// As [`G_SpawnString`]; `out` must point to a writable `float[3]`.
pub unsafe fn G_SpawnVector(
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut f32,
) -> qboolean {
    let mut s: *mut c_char = null_mut();
    let present = G_SpawnString(key, defaultString, &mut s);
    sscanf(
        s,
        c"%f %f %f".as_ptr(),
        out,
        out.add(1),
        out.add(2),
    );
    present
}

/// `char *G_NewString( const char *string )` (g_spawn.c:716).
///
/// Builds a copy of `string` in the game module's [`G_Alloc`] zone, translating the
/// two-character escape `\n` into a real linefeed (so spawn message texts can be
/// multi-line) and `\<other>` into a bare backslash. Carried over verbatim, including
/// the `i < l-1` guard's quirk: a trailing backslash (the last real char) still enters
/// the escape branch, reads the NUL as its second byte, and emits a lone `\\`.
///
/// `l = strlen+1` so the loop's final iteration copies the NUL terminator.
pub unsafe fn G_NewString(string: *const c_char) -> *mut c_char {
    // l = strlen(string) + 1
    let mut len: usize = 0;
    while *string.add(len) != 0 {
        len += 1;
    }
    let l: i32 = (len + 1) as i32;

    let newb = G_Alloc(l) as *mut c_char;
    let mut new_p = newb;

    // turn \n into a real linefeed
    let mut i: i32 = 0;
    while i < l {
        let ci = *string.add(i as usize);
        if ci == b'\\' as c_char && i < l - 1 {
            i += 1;
            let cj = *string.add(i as usize);
            *new_p = if cj == b'n' as c_char {
                b'\n' as c_char
            } else {
                b'\\' as c_char
            };
            new_p = new_p.add(1);
        } else {
            *new_p = ci;
            new_p = new_p.add(1);
        }
        i += 1;
    }

    newb
}

/// `char *G_AddSpawnVarToken( const char *string )` (g_spawn.c:843).
///
/// Appends `string` (plus its NUL) to the global `level.spawnVarChars[]` arena and
/// returns a pointer to the stored copy — the backing store the `spawnVars[][]`
/// key/value pointers alias. [`G_Error`]s (fatal) if the arena would overflow
/// `MAX_SPAWN_VARS_CHARS`. No oracle — it mutates the global `level`.
///
/// # Safety
/// `string` must be NUL-terminated; must run on the game thread that owns `level`.
pub unsafe fn G_AddSpawnVarToken(string: *const c_char) -> *mut c_char {
    let l = strlen(string) as c_int;
    let lvl = addr_of_mut!(level);

    if (*lvl).numSpawnVarChars + l + 1 > MAX_SPAWN_VARS_CHARS {
        G_Error("G_AddSpawnVarToken: MAX_SPAWN_CHARS");
    }

    let dest = (*lvl)
        .spawnVarChars
        .as_mut_ptr()
        .add((*lvl).numSpawnVarChars as usize);
    copy_nonoverlapping(string, dest, (l + 1) as usize);

    (*lvl).numSpawnVarChars += l + 1;

    dest
}

/// `void AddSpawnField( char *field, char *value )` (g_spawn.c:860).
///
/// Sets the spawn var `field` to `value`: on a case-insensitive [`Q_stricmp`] match
/// it overwrites the existing value pointer (re-interning `value` via
/// [`G_AddSpawnVarToken`]); otherwise it appends a fresh key/value pair and bumps
/// `level.numSpawnVars`. Used by [`HandleEntityAdjustment`] to rewrite the BSP-instance
/// origin/angles/target fields. No oracle — it mutates the global `level`.
///
/// Faithful quirk: unlike [`G_ParseSpawnVars`], the append path does **not** guard
/// against `MAX_SPAWN_VARS`, matching the original.
///
/// # Safety
/// `field`/`value` must be NUL-terminated; must run on the game thread that owns `level`.
pub unsafe fn AddSpawnField(field: *const c_char, value: *const c_char) {
    let lvl = addr_of_mut!(level);

    let mut i: c_int = 0;
    while i < (*lvl).numSpawnVars {
        if Q_stricmp((*lvl).spawnVars[i as usize][0], field) == 0 {
            (*lvl).spawnVars[i as usize][1] = G_AddSpawnVarToken(value);
            return;
        }
        i += 1;
    }

    let n = (*lvl).numSpawnVars as usize;
    (*lvl).spawnVars[n][0] = G_AddSpawnVarToken(field);
    (*lvl).spawnVars[n][1] = G_AddSpawnVarToken(value);
    (*lvl).numSpawnVars += 1;
}

/// `static void HandleEntityAdjustment( void )` (g_spawn.c:882).
///
/// Rewrites the just-parsed spawn vars of a sub-BSP-instance entity into world
/// space before it is spawned: rotates `origin` by `level.mRotationAdjust` and
/// translates by `level.mOriginAdjust`; folds the rotation into `angles`/`angle`
/// and `direction`; stamps `BSPInstanceID`; and prefixes the target-linkage fields
/// (`targetname`/`target`/`killtarget`/`brushparent`/`brushchild`/`enemy`/
/// `ICARUSname`) with `level.mTargetAdjust` so each instance's links stay private to
/// that instance. Each rewrite goes through [`AddSpawnField`]. No oracle — it reads
/// and mutates the global `level`.
///
/// DEVIATIONS carried as faithfully as the host build allows:
/// - `Com_sprintf("%0.0f", …)` → `format_args!("{:.0}", …)`: the printf float
///   formatting becomes Rust's (the codebase-wide [`Com_sprintf`] translation). Both
///   round half-to-even, so they agree except on rare exact-half f32 values.
/// - The locals (`origin`/`angles`) are zero-initialised; the original leaves them
///   uninitialised and relies on `sscanf` filling them, so a value string with fewer
///   than the expected floats reads as `0.0` here vs. garbage there (upstream UB).
///
/// # Safety
/// Must run on the game thread that owns `level`, after [`G_ParseSpawnVars`] filled it.
unsafe fn HandleEntityAdjustment() {
    let lvl = addr_of_mut!(level);
    let novalue = c"novalue".as_ptr(); // #define NOVALUE "novalue"

    let mut value: *mut c_char = null_mut();
    let mut origin: vec3_t = [0.0; 3];
    let mut new_origin: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut temp = [0 as c_char; MAX_QPATH];

    // Snapshot the per-instance adjustments (AddSpawnField never touches them).
    let rot_adjust = (*lvl).mRotationAdjust;
    let origin_adjust = (*lvl).mOriginAdjust;
    let mtarget = (*lvl).mTargetAdjust;

    // ---- origin: rotate then translate ----
    G_SpawnString(c"origin".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        sscanf(
            value,
            c"%f %f %f".as_ptr(),
            origin.as_mut_ptr(),
            origin.as_mut_ptr().add(1),
            origin.as_mut_ptr().add(2),
        );
    } else {
        origin[0] = 0.0;
        origin[1] = 0.0;
        origin[2] = 0.0;
    }

    let rotation = DEG2RAD(rot_adjust);
    let cosr = (rotation as f64).cos();
    let sinr = (rotation as f64).sin();
    new_origin[0] = (origin[0] as f64 * cosr - origin[1] as f64 * sinr) as f32;
    new_origin[1] = (origin[0] as f64 * sinr + origin[1] as f64 * cosr) as f32;
    new_origin[2] = origin[2];
    let pre_translate = new_origin;
    VectorAdd(&pre_translate, &origin_adjust, &mut new_origin);
    Com_sprintf(
        temp.as_mut_ptr(),
        MAX_QPATH as c_int,
        format_args!("{:.0} {:.0} {:.0}", new_origin[0], new_origin[1], new_origin[2]),
    );
    AddSpawnField(c"origin".as_ptr(), temp.as_ptr());

    // ---- angles / angle: fold the rotation into yaw ----
    G_SpawnString(c"angles".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        sscanf(
            value,
            c"%f %f %f".as_ptr(),
            angles.as_mut_ptr(),
            angles.as_mut_ptr().add(1),
            angles.as_mut_ptr().add(2),
        );
        angles[1] = ((angles[1] + rot_adjust) as f64 % 360.0_f64) as f32;
        Com_sprintf(
            temp.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{:.0} {:.0} {:.0}", angles[0], angles[1], angles[2]),
        );
        AddSpawnField(c"angles".as_ptr(), temp.as_ptr());
    } else {
        G_SpawnString(c"angle".as_ptr(), novalue, &mut value);
        if Q_stricmp(value, novalue) != 0 {
            sscanf(value, c"%f".as_ptr(), angles.as_mut_ptr().add(1));
        } else {
            angles[1] = 0.0;
        }
        angles[1] = ((angles[1] + rot_adjust) as f64 % 360.0_f64) as f32;
        Com_sprintf(
            temp.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{:.0}", angles[1]),
        );
        AddSpawnField(c"angle".as_ptr(), temp.as_ptr());
    }

    // RJR experimental code for handling "direction" field of breakable brushes
    // though direction is rarely ever used.
    G_SpawnString(c"direction".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        sscanf(
            value,
            c"%f %f %f".as_ptr(),
            angles.as_mut_ptr(),
            angles.as_mut_ptr().add(1),
            angles.as_mut_ptr().add(2),
        );
    } else {
        angles[0] = 0.0;
        angles[1] = 0.0;
        angles[2] = 0.0;
    }
    angles[1] = ((angles[1] + rot_adjust) as f64 % 360.0_f64) as f32;
    Com_sprintf(
        temp.as_mut_ptr(),
        MAX_QPATH as c_int,
        format_args!("{:.0} {:.0} {:.0}", angles[0], angles[1], angles[2]),
    );
    AddSpawnField(c"direction".as_ptr(), temp.as_ptr());

    AddSpawnField(c"BSPInstanceID".as_ptr(), mtarget);

    // ---- prefix the target-linkage fields with mTargetAdjust ----
    G_SpawnString(c"targetname".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"targetname".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"target".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"target".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"killtarget".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"killtarget".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"brushparent".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"brushparent".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"brushchild".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"brushchild".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"enemy".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"enemy".as_ptr(), temp.as_ptr());
    }

    G_SpawnString(c"ICARUSname".as_ptr(), novalue, &mut value);
    if Q_stricmp(value, novalue) != 0 {
        Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("{}{}", Sz(mtarget), Sz(value)));
        AddSpawnField(c"ICARUSname".as_ptr(), temp.as_ptr());
    }
}

/// `qboolean G_ParseSpawnVars( qboolean inSubBSP )` (g_spawn.c:1010).
///
/// Parses one brace-bounded `{ "key" "value" … }` block from the engine's cached
/// entity string (pulled token by token through [`trap::GetEntityToken`]) into the
/// global `level.spawnVars[]`/`spawnVarChars[]` tables. Returns `qfalse` at end of
/// string (no opening brace left), `qtrue` after a block is parsed. Does **not**
/// spawn anything — `G_SpawnGEntityFromSpawnVars` consumes the table afterwards.
/// Fatal [`G_Error`] on a malformed block (missing `{`, EOF before `}`, empty
/// `}`, or `MAX_SPAWN_VARS` overflow). When `inSubBSP`, the freshly parsed vars are
/// run through [`HandleEntityAdjustment`] to relocate the instance into world space.
///
/// # Safety
/// Must run on the game thread that owns `level`.
pub unsafe fn G_ParseSpawnVars(inSubBSP: qboolean) -> qboolean {
    let mut keyname = [0 as c_char; MAX_TOKEN_CHARS];
    let mut com_token = [0 as c_char; MAX_TOKEN_CHARS];

    let lvl = addr_of_mut!(level);
    (*lvl).numSpawnVars = 0;
    (*lvl).numSpawnVarChars = 0;

    // parse the opening brace
    if trap::GetEntityToken(&mut com_token) == QFALSE {
        // end of spawn string
        return QFALSE;
    }
    if com_token[0] != b'{' as c_char {
        G_Error(&format!(
            "G_ParseSpawnVars: found {} when expecting {{",
            Sz(com_token.as_ptr())
        ));
    }

    // go through all the key / value pairs
    loop {
        // parse key
        if trap::GetEntityToken(&mut keyname) == QFALSE {
            G_Error("G_ParseSpawnVars: EOF without closing brace");
        }

        if keyname[0] == b'}' as c_char {
            break;
        }

        // parse value
        if trap::GetEntityToken(&mut com_token) == QFALSE {
            G_Error("G_ParseSpawnVars: EOF without closing brace");
        }

        if com_token[0] == b'}' as c_char {
            G_Error("G_ParseSpawnVars: closing brace without data");
        }
        if (*lvl).numSpawnVars == MAX_SPAWN_VARS {
            G_Error("G_ParseSpawnVars: MAX_SPAWN_VARS");
        }
        let n = (*lvl).numSpawnVars as usize;
        (*lvl).spawnVars[n][0] = G_AddSpawnVarToken(keyname.as_ptr());
        (*lvl).spawnVars[n][1] = G_AddSpawnVarToken(com_token.as_ptr());
        (*lvl).numSpawnVars += 1;
    }

    if inSubBSP != QFALSE {
        HandleEntityAdjustment();
    }

    QTRUE
}

/// `void SP_item_botroam( gentity_t *ent )` (g_spawn.c:363).
///
/// Empty body — a stub `SP_*` spawn handler so the `item_botroam` class registers in
/// the spawn table without doing anything (bot-roam waypoints are handled elsewhere).
/// No oracle (empty body; takes an opaque `gentity_t *`).
///
/// # Safety
/// `ent` is unused; the pointer need not be valid.
pub unsafe extern "C" fn SP_item_botroam(_ent: *mut gentity_t) {}

/// `void SP_gametype_item ( gentity_t* ent )` (g_spawn.c:367).
///
/// Spawn handler for `gametype_item` entities — the CTF flag spawns. Reads the optional
/// `teamfilter` spawn key (purely to consume it; the value is unused), fixes the entity
/// origin, then resolves which flag item to drop. When `level.mTeamFilter` is set it
/// overrides any per-entity team and only honours a `targetname` containing `"flag"`;
/// otherwise it keys off `"red_flag"`/`"blue_flag"` in the `targetname`. A resolved item
/// clears `targetname`, adopts the item's `classname`, and is spawned via [`G_SpawnItem`].
/// No oracle (entity-state spawn side effects; calls [`G_SetOrigin`]/[`G_SpawnItem`]).
///
/// # Safety
/// `ent` must be a valid, in-use `gentity_t *`; `level` must be initialized.
pub unsafe extern "C" fn SP_gametype_item(ent: *mut gentity_t) {
    let mut item: *mut gitem_t = null_mut();
    let mut value: *mut c_char = null_mut();
    let mut team: c_int = -1;

    G_SpawnString(c"teamfilter".as_ptr(), c"".as_ptr(), &mut value);

    G_SetOrigin(ent, &(*ent).s.origin);

    let lvl = addr_of!(level);

    // If a team filter is set then override any team settings for the spawns
    if (*lvl).mTeamFilter[0] != 0 {
        if Q_stricmp((*lvl).mTeamFilter.as_ptr(), c"red".as_ptr()) == 0 {
            team = TEAM_RED as c_int;
        } else if Q_stricmp((*lvl).mTeamFilter.as_ptr(), c"blue".as_ptr()) == 0 {
            team = TEAM_BLUE as c_int;
        }
    }

    if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
        if team != -1 {
            if !strstr((*ent).targetname, c"flag".as_ptr()).is_null() {
                if team == TEAM_RED as c_int {
                    item = BG_FindItem(c"team_CTF_redflag".as_ptr());
                } else {
                    //blue
                    item = BG_FindItem(c"team_CTF_blueflag".as_ptr());
                }
            }
        } else if !strstr((*ent).targetname, c"red_flag".as_ptr()).is_null() {
            item = BG_FindItem(c"team_CTF_redflag".as_ptr());
        } else if !strstr((*ent).targetname, c"blue_flag".as_ptr()).is_null() {
            item = BG_FindItem(c"team_CTF_blueflag".as_ptr());
        } else {
            item = null_mut();
        }

        if !item.is_null() {
            (*ent).targetname = null_mut();
            (*ent).classname = (*item).classname;
            G_SpawnItem(ent, item);
        }
    }
}

/// `qboolean SP_bsp_worldspawn( void )` (g_spawn.c:1384).
///
/// Stub worldspawn handler for sub-BSP instances — currently just returns `qtrue`.
/// Carried-over comment: `//rww - Planning on having something here?`. No oracle
/// (trivial constant return).
pub unsafe fn SP_bsp_worldspawn() -> qboolean {
    //rww - Planning on having something here?
    QTRUE
}

/// `void G_PrecacheSoundsets( void )` (g_spawn.c:1389).
///
/// Walks the whole `g_entities[]` array; for every in-use entity carrying a non-empty
/// `soundSet` string, registers it via [`G_SoundSetIndex`] (a `CS_AMBIENT_SET`
/// configstring) and stamps the resulting index into `ent->s.soundSetIndex`. Fatal
/// [`Com_Error`]`(ERR_DROP)` once more than `MAX_AMBIENT_SETS` distinct sets have been
/// counted. No oracle — it walks the global `g_entities` array and calls the
/// trap-backed [`G_SoundSetIndex`] (configstring registration).
///
/// `soundSet` is a `*mut c_char`; bridged to the `&str` [`G_SoundSetIndex`] expects
/// through `CStr::from_ptr(...).to_string_lossy()`, the codebase idiom for char-pointer
/// fields feeding the configstring index helpers.
///
/// # Safety
/// Must run on the game thread that owns `g_entities` / `level`, after `g_entities`
/// has been allocated by `GAME_INIT`.
pub unsafe fn G_PrecacheSoundsets() {
    // C: `gentity_t *ent = NULL;` — the loop cursor; bound per-iteration below.
    let mut countedSets: c_int = 0;

    let mut i: usize = 0;
    while i < MAX_GENTITIES {
        let ent: *mut gentity_t = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(i);

        if (*ent).inuse != QFALSE && !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
            if countedSets >= MAX_AMBIENT_SETS as c_int {
                Com_Error(
                    ERR_DROP,
                    "MAX_AMBIENT_SETS was exceeded! (too many soundsets)\n",
                );
            }

            (*ent).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*ent).soundSet).to_string_lossy());
            countedSets += 1;
        }
        i += 1;
    }
}

// ===========================================================================
// spawns[] dispatch table (g_spawn.c:430) — KEYSTONE.
//
// C type is `void (*)(gentity_t *)`, so the Rust entries are
// `unsafe extern "C" fn(*mut gentity_t)`. Handlers already ported elsewhere are
// referenced directly; handlers ported with the *Rust* ABI (`unsafe fn`, not
// `extern "C"`) are bridged through one-line `extern "C"` shims below (Rust will
// not coerce between the two ABIs, and the ported sources are not `#[no_mangle]`
// so an `extern` forward-decl cannot reach them). Deep handlers whose
// subsystems are not yet ported (NPC / NAV-waypoint / Siege / turret / RMG-terrain)
// are present as no-op `extern "C"` stubs so the table compiles and mirrors C
// exactly; each carries a `REVISIT` note for when its subsystem lands.
// ===========================================================================

// --- already-ported handlers, C ABI: referenced directly ------------------
use crate::codemp::game::g_client::{
    SP_info_player_deathmatch, SP_info_player_duel, SP_info_player_duel1, SP_info_player_duel2,
    SP_info_player_intermission, SP_info_player_siegeteam1, SP_info_player_siegeteam2,
    SP_info_player_start,
};
use crate::codemp::game::g_misc::{
    SP_CreateRain, SP_CreateSnow, SP_CreateSpaceDust, SP_fx_runner, SP_info_camp, SP_info_notnull,
    SP_info_null, SP_light, SP_misc_G2model, SP_misc_ammo_floor_unit, SP_misc_faller,
    SP_misc_holocron, SP_misc_maglock, SP_misc_model, SP_misc_model_ammo_power_converter,
    SP_misc_model_health_power_converter, SP_misc_model_shield_power_converter, SP_misc_model_static,
    SP_misc_portal_camera, SP_misc_portal_surface, SP_misc_shield_floor_unit, SP_misc_skyportal,
    SP_misc_skyportal_orient, SP_misc_teleporter_dest, SP_misc_weapon_shooter, SP_misc_weather_zone,
    SP_reference_tag, SP_shooter_blaster, SP_target_escapetrig, SP_target_screenshake,
};
use crate::codemp::game::g_target::{
    SP_target_activate, SP_target_counter, SP_target_deactivate, SP_target_delay, SP_target_give,
    SP_target_kill, SP_target_laser, SP_target_level_change, SP_target_location, SP_target_play_music,
    SP_target_position, SP_target_print, SP_target_random, SP_target_relay,
    SP_target_remove_powerups, SP_target_score, SP_target_scriptrunner, SP_target_speaker,
    SP_target_teleporter,
};
use crate::codemp::game::g_team::{
    SP_team_CTF_blueplayer, SP_team_CTF_bluespawn, SP_team_CTF_redplayer, SP_team_CTF_redspawn,
};
use crate::codemp::game::g_trigger::{
    SP_func_timer, SP_target_push, SP_trigger_always, SP_trigger_hurt, SP_trigger_hyperspace,
    SP_trigger_lightningstrike, SP_trigger_multiple, SP_trigger_once, SP_trigger_push,
    SP_trigger_shipboundary, SP_trigger_space, SP_trigger_teleport,
};

// --- already-ported handlers, Rust ABI: bridged through extern "C" shims ----
// Each shim is a faithful trampoline to the ported `unsafe fn` of the same name.
macro_rules! spawn_shim {
    ($shim:ident => $path:path) => {
        unsafe extern "C" fn $shim(ent: *mut gentity_t) {
            $path(ent)
        }
    };
}
spawn_shim!(sp_emplaced_gun => crate::codemp::game::g_weapon::SP_emplaced_gun);
spawn_shim!(sp_func_bobbing => crate::codemp::game::g_mover::SP_func_bobbing);
spawn_shim!(sp_func_breakable => crate::codemp::game::g_mover::SP_func_breakable);
spawn_shim!(sp_func_button => crate::codemp::game::g_mover::SP_func_button);
spawn_shim!(sp_func_door => crate::codemp::game::g_mover::SP_func_door);
spawn_shim!(sp_func_glass => crate::codemp::game::g_mover::SP_func_glass);
spawn_shim!(sp_func_pendulum => crate::codemp::game::g_mover::SP_func_pendulum);
spawn_shim!(sp_func_plat => crate::codemp::game::g_mover::SP_func_plat);
spawn_shim!(sp_func_rotating => crate::codemp::game::g_mover::SP_func_rotating);
spawn_shim!(sp_func_static => crate::codemp::game::g_mover::SP_func_static);
spawn_shim!(sp_func_train => crate::codemp::game::g_mover::SP_func_train);
spawn_shim!(sp_func_usable => crate::codemp::game::g_mover::SP_func_usable);
spawn_shim!(sp_func_wall => crate::codemp::game::g_mover::SP_func_wall);
spawn_shim!(sp_path_corner => crate::codemp::game::g_mover::SP_path_corner);
spawn_shim!(sp_info_siege_decomplete => crate::codemp::game::g_saga::SP_info_siege_decomplete);
spawn_shim!(sp_info_siege_radaricon => crate::codemp::game::g_saga::SP_info_siege_radaricon);
spawn_shim!(sp_misc_siege_item => crate::codemp::game::g_saga::SP_misc_siege_item);

// --- not-yet-ported-subsystem handlers: no-op stubs (mirror C entries) -----------
macro_rules! spawn_stub {
    ($($name:ident),+ $(,)?) => {
        $( unsafe extern "C" fn $name(_ent: *mut gentity_t) {} )+
    };
}
// REVISIT: stub — un-stub when the NPC subsystem lands.
spawn_stub!(
    SP_NPC_spawner, SP_NPC_Vehicle, SP_NPC_Kyle, SP_NPC_Lando, SP_NPC_Jan, SP_NPC_Luke,
    SP_NPC_MonMothma, SP_NPC_Tavion, SP_NPC_Tavion_New, SP_NPC_Alora, SP_NPC_Reelo, SP_NPC_Galak,
    SP_NPC_Desann, SP_NPC_Bartender, SP_NPC_MorganKatarn, SP_NPC_Jedi, SP_NPC_Prisoner, SP_NPC_Rebel,
    SP_NPC_Stormtrooper, SP_NPC_StormtrooperOfficer, SP_NPC_Snowtrooper, SP_NPC_Tie_Pilot,
    SP_NPC_Ugnaught, SP_NPC_Jawa, SP_NPC_Gran, SP_NPC_Rodian, SP_NPC_Weequay, SP_NPC_Trandoshan,
    SP_NPC_Tusken, SP_NPC_Noghri, SP_NPC_SwampTrooper, SP_NPC_Imperial, SP_NPC_ImpWorker,
    SP_NPC_BespinCop, SP_NPC_Reborn, SP_NPC_ShadowTrooper, SP_NPC_Monster_Murjj,
    SP_NPC_Monster_Swamp, SP_NPC_Monster_Howler, SP_NPC_MineMonster, SP_NPC_Monster_Claw,
    SP_NPC_Monster_Glider, SP_NPC_Monster_Flier2, SP_NPC_Monster_Lizard, SP_NPC_Monster_Fish,
    SP_NPC_Monster_Wampa, SP_NPC_Monster_Rancor, SP_NPC_Droid_Interrogator, SP_NPC_Droid_Probe,
    SP_NPC_Droid_Mark1, SP_NPC_Droid_Mark2, SP_NPC_Droid_ATST, SP_NPC_Droid_Seeker,
    SP_NPC_Droid_Remote, SP_NPC_Droid_Sentry, SP_NPC_Droid_Gonk, SP_NPC_Droid_Mouse,
    SP_NPC_Droid_R2D2, SP_NPC_Droid_R5D2, SP_NPC_Droid_Protocol, SP_NPC_Reborn_New, SP_NPC_Cultist,
    SP_NPC_Cultist_Saber, SP_NPC_Cultist_Saber_Powers, SP_NPC_Cultist_Destroyer,
    SP_NPC_Cultist_Commando,
);
// REVISIT: stub — un-stub when the NAV/waypoint subsystem lands.
spawn_stub!(
    SP_waypoint, SP_waypoint_small, SP_waypoint_navgoal, SP_waypoint_navgoal_8,
    SP_waypoint_navgoal_4, SP_waypoint_navgoal_2, SP_waypoint_navgoal_1, SP_point_combat,
);
// REVISIT: stub — un-stub when the Siege subsystem lands.
spawn_stub!(SP_info_siege_objective, SP_target_siege_end);
// REVISIT: stub — un-stub `SP_misc_turret` when the non-G2 turret lands.
// `SP_misc_turretG2` is the real port (g_turret_g2.rs), imported below.
spawn_stub!(SP_misc_turret);
use crate::codemp::game::g_turret_g2::SP_misc_turretG2;
// REVISIT: stub — un-stub when the relevant subsystem lands.
//   SP_info_jedimaster_start (g_main.c) / SP_terrain (RMG, blocked) /
//   SP_target_interest (NPC AI).
spawn_stub!(SP_info_jedimaster_start, SP_terrain, SP_target_interest);
/// `void SP_misc_bsp( gentity_t *ent )` (g_misc.c:387) — spawns a `misc_bsp` sub-BSP
/// instance: a separate `.bsp` brush model stamped into the level at runtime. Sets the
/// per-instance origin/rotation/target adjustments on `level`, brush-models the entity,
/// then recursively spawns the instanced entities (with the active sub-BSP selected) and
/// pops the instance depth. No oracle (configstring/level/brush-model trap side effects);
/// verified by review against the C.
///
/// `G_BSPIndex` (g_utils.c) is reconstructed (commented-out in JKA); `strcpy(mTeamFilter,…)`
/// uses the project's `Com_sprintf("%s")` faithful-copy idiom; the commented-out
/// `g_debugRMG` debug-cylinder tail and `mFilter` lines are dropped as in the C.
///
/// # Safety
/// `ent` must be a valid `*mut gentity_t`; `level` must be initialised; spawn-var parsing
/// state must be live (called during `G_SpawnEntitiesFromString`).
unsafe extern "C" fn SP_misc_bsp(ent: *mut gentity_t) {
    let lvl = addr_of_mut!(level);
    let mut temp = [0 as c_char; MAX_QPATH];
    let mut out: *mut c_char = null_mut();
    let mut newAngle: f32 = 0.0;
    let mut tempint: c_int = 0;

    G_SpawnFloat(c"angle".as_ptr(), c"0".as_ptr(), &mut newAngle);
    if newAngle != 0.0 {
        (*ent).s.angles[1] = newAngle;
    }
    // don't support rotation any other way
    (*ent).s.angles[0] = 0.0;
    (*ent).s.angles[2] = 0.0;

    G_SpawnString(c"bspmodel".as_ptr(), c"".as_ptr(), &mut out);

    (*ent).s.eFlags = EF_PERMANENT;

    // Mainly for debugging
    G_SpawnInt(c"spacing".as_ptr(), c"0".as_ptr(), &mut tempint);
    (*ent).s.time2 = tempint;
    G_SpawnInt(c"flatten".as_ptr(), c"0".as_ptr(), &mut tempint);
    (*ent).s.time = tempint;

    Com_sprintf(temp.as_mut_ptr(), MAX_QPATH as c_int, format_args!("#{}", Sz(out)));
    trap::SetBrushModel(ent, &CStr::from_ptr(temp.as_ptr()).to_string_lossy()); // SV_SetBrushModel -- sets mins and maxs
    G_BSPIndex(&CStr::from_ptr(temp.as_ptr()).to_string_lossy());

    (*lvl).mNumBSPInstances += 1;
    Com_sprintf(
        temp.as_mut_ptr(),
        MAX_QPATH as c_int,
        format_args!("{}-", (*lvl).mNumBSPInstances),
    );
    let ent_origin = (*ent).s.origin;
    let ent_angles = (*ent).s.angles;
    VectorCopy(&ent_origin, &mut (*lvl).mOriginAdjust);
    (*lvl).mRotationAdjust = (*ent).s.angles[1];
    // faithful to C `level.mTargetAdjust = temp;` — stores a pointer to this fn's `temp`
    // stack buffer; valid through the `G_SpawnEntitiesFromString` call below (where the
    // instanced ents read it via `HandleEntityAdjustment`), dangling after return as in C.
    (*lvl).mTargetAdjust = temp.as_mut_ptr();
    //level.hasBspInstances = qtrue; //rww - also not referenced anywhere.
    (*lvl).mBSPInstanceDepth += 1;

    //G_SpawnString("filter", "", &out);
    //strcpy(level.mFilter, out);

    G_SpawnString(c"teamfilter".as_ptr(), c"".as_ptr(), &mut out);
    // C: `strcpy(level.mTeamFilter, out)` — Com_sprintf("%s") is the project's faithful strcpy idiom.
    Com_sprintf(
        (*lvl).mTeamFilter.as_mut_ptr(),
        MAX_QPATH as c_int,
        format_args!("{}", Sz(out)),
    );

    VectorCopy(&ent_origin, &mut (*ent).s.pos.trBase);
    VectorCopy(&ent_origin, &mut (*ent).r.currentOrigin);
    VectorCopy(&ent_angles, &mut (*ent).s.apos.trBase);
    VectorCopy(&ent_angles, &mut (*ent).r.currentAngles);

    (*ent).s.eType = ET_MOVER;

    trap::LinkEntity(ent);

    trap::SetActiveSubBSP((*ent).s.modelindex);
    G_SpawnEntitiesFromString(QTRUE);
    trap::SetActiveSubBSP(-1);

    (*lvl).mBSPInstanceDepth -= 1;
    //level.mFilter[0] = level.mTeamFilter[0] = 0;
    (*lvl).mTeamFilter[0] = 0;

    // (the commented-out `g_debugRMG` debug-cylinder tail is debug-only — dropped as in C)
}

/// `spawn_t` (g_spawn.c:153) — one row of the [`spawns`] dispatch table: a class
/// `name` and the `void (*)(gentity_t *)` handler that spawns it.
#[repr(C)]
pub struct spawn_t {
    pub name: *const c_char,
    pub spawn: Option<unsafe extern "C" fn(*mut gentity_t)>,
}

// `spawn_t` holds raw pointers; the table is module-private and only read on the
// game thread, mirroring the C file-scope array.
unsafe impl Sync for spawn_t {}

/// `spawn_t spawns[]` (g_spawn.c:430) — the class-name → spawn-function registry
/// scanned by [`G_CallSpawn`]. Faithful 1:1 with the C table, including the
/// commented-out `misc_bsp` line and the trailing `{0, 0}` terminator (here a
/// NUL name + `None`). Comments carried over verbatim.
#[allow(non_upper_case_globals)] // C global name `spawns` kept verbatim
static spawns: &[spawn_t] = &[
    // info entities don't do anything at all, but provide positional
    // information for things controlled by other processes
    spawn_t { name: c"info_player_start".as_ptr(), spawn: Some(SP_info_player_start) },
    spawn_t { name: c"info_player_duel".as_ptr(), spawn: Some(SP_info_player_duel) },
    spawn_t { name: c"info_player_duel1".as_ptr(), spawn: Some(SP_info_player_duel1) },
    spawn_t { name: c"info_player_duel2".as_ptr(), spawn: Some(SP_info_player_duel2) },
    spawn_t { name: c"info_player_deathmatch".as_ptr(), spawn: Some(SP_info_player_deathmatch) },
    spawn_t { name: c"info_player_siegeteam1".as_ptr(), spawn: Some(SP_info_player_siegeteam1) },
    spawn_t { name: c"info_player_siegeteam2".as_ptr(), spawn: Some(SP_info_player_siegeteam2) },
    spawn_t { name: c"info_player_intermission".as_ptr(), spawn: Some(SP_info_player_intermission) },
    spawn_t { name: c"info_jedimaster_start".as_ptr(), spawn: Some(SP_info_jedimaster_start) },
    spawn_t { name: c"info_null".as_ptr(), spawn: Some(SP_info_null) },
    spawn_t { name: c"info_notnull".as_ptr(), spawn: Some(SP_info_notnull) }, // use target_position instead
    spawn_t { name: c"info_camp".as_ptr(), spawn: Some(SP_info_camp) },

    spawn_t { name: c"info_siege_objective".as_ptr(), spawn: Some(SP_info_siege_objective) },
    spawn_t { name: c"info_siege_radaricon".as_ptr(), spawn: Some(sp_info_siege_radaricon) },
    spawn_t { name: c"info_siege_decomplete".as_ptr(), spawn: Some(sp_info_siege_decomplete) },
    spawn_t { name: c"target_siege_end".as_ptr(), spawn: Some(SP_target_siege_end) },
    spawn_t { name: c"misc_siege_item".as_ptr(), spawn: Some(sp_misc_siege_item) },

    spawn_t { name: c"func_plat".as_ptr(), spawn: Some(sp_func_plat) },
    spawn_t { name: c"func_button".as_ptr(), spawn: Some(sp_func_button) },
    spawn_t { name: c"func_door".as_ptr(), spawn: Some(sp_func_door) },
    spawn_t { name: c"func_static".as_ptr(), spawn: Some(sp_func_static) },
    spawn_t { name: c"func_rotating".as_ptr(), spawn: Some(sp_func_rotating) },
    spawn_t { name: c"func_bobbing".as_ptr(), spawn: Some(sp_func_bobbing) },
    spawn_t { name: c"func_pendulum".as_ptr(), spawn: Some(sp_func_pendulum) },
    spawn_t { name: c"func_train".as_ptr(), spawn: Some(sp_func_train) },
    spawn_t { name: c"func_group".as_ptr(), spawn: Some(SP_info_null) },
    spawn_t { name: c"func_timer".as_ptr(), spawn: Some(SP_func_timer) }, // rename trigger_timer?
    spawn_t { name: c"func_breakable".as_ptr(), spawn: Some(sp_func_breakable) },
    spawn_t { name: c"func_glass".as_ptr(), spawn: Some(sp_func_glass) },
    spawn_t { name: c"func_usable".as_ptr(), spawn: Some(sp_func_usable) },
    spawn_t { name: c"func_wall".as_ptr(), spawn: Some(sp_func_wall) },

    // Triggers are brush objects that cause an effect when contacted
    // by a living player, usually involving firing targets.
    // While almost everything could be done with
    // a single trigger class and different targets, triggered effects
    // could not be client side predicted (push and teleport).
    spawn_t { name: c"trigger_lightningstrike".as_ptr(), spawn: Some(SP_trigger_lightningstrike) },

    spawn_t { name: c"trigger_always".as_ptr(), spawn: Some(SP_trigger_always) },
    spawn_t { name: c"trigger_multiple".as_ptr(), spawn: Some(SP_trigger_multiple) },
    spawn_t { name: c"trigger_once".as_ptr(), spawn: Some(SP_trigger_once) },
    spawn_t { name: c"trigger_push".as_ptr(), spawn: Some(SP_trigger_push) },
    spawn_t { name: c"trigger_teleport".as_ptr(), spawn: Some(SP_trigger_teleport) },
    spawn_t { name: c"trigger_hurt".as_ptr(), spawn: Some(SP_trigger_hurt) },
    spawn_t { name: c"trigger_space".as_ptr(), spawn: Some(SP_trigger_space) },
    spawn_t { name: c"trigger_shipboundary".as_ptr(), spawn: Some(SP_trigger_shipboundary) },
    spawn_t { name: c"trigger_hyperspace".as_ptr(), spawn: Some(SP_trigger_hyperspace) },

    // targets perform no action by themselves, but must be triggered
    // by another entity
    spawn_t { name: c"target_give".as_ptr(), spawn: Some(SP_target_give) },
    spawn_t { name: c"target_remove_powerups".as_ptr(), spawn: Some(SP_target_remove_powerups) },
    spawn_t { name: c"target_delay".as_ptr(), spawn: Some(SP_target_delay) },
    spawn_t { name: c"target_speaker".as_ptr(), spawn: Some(SP_target_speaker) },
    spawn_t { name: c"target_print".as_ptr(), spawn: Some(SP_target_print) },
    spawn_t { name: c"target_laser".as_ptr(), spawn: Some(SP_target_laser) },
    spawn_t { name: c"target_score".as_ptr(), spawn: Some(SP_target_score) },
    spawn_t { name: c"target_teleporter".as_ptr(), spawn: Some(SP_target_teleporter) },
    spawn_t { name: c"target_relay".as_ptr(), spawn: Some(SP_target_relay) },
    spawn_t { name: c"target_kill".as_ptr(), spawn: Some(SP_target_kill) },
    spawn_t { name: c"target_position".as_ptr(), spawn: Some(SP_target_position) },
    spawn_t { name: c"target_location".as_ptr(), spawn: Some(SP_target_location) },
    spawn_t { name: c"target_counter".as_ptr(), spawn: Some(SP_target_counter) },
    spawn_t { name: c"target_random".as_ptr(), spawn: Some(SP_target_random) },
    spawn_t { name: c"target_scriptrunner".as_ptr(), spawn: Some(SP_target_scriptrunner) },
    spawn_t { name: c"target_interest".as_ptr(), spawn: Some(SP_target_interest) },
    spawn_t { name: c"target_activate".as_ptr(), spawn: Some(SP_target_activate) },
    spawn_t { name: c"target_deactivate".as_ptr(), spawn: Some(SP_target_deactivate) },
    spawn_t { name: c"target_level_change".as_ptr(), spawn: Some(SP_target_level_change) },
    spawn_t { name: c"target_play_music".as_ptr(), spawn: Some(SP_target_play_music) },
    spawn_t { name: c"target_push".as_ptr(), spawn: Some(SP_target_push) },

    spawn_t { name: c"light".as_ptr(), spawn: Some(SP_light) },
    spawn_t { name: c"path_corner".as_ptr(), spawn: Some(sp_path_corner) },

    spawn_t { name: c"misc_teleporter_dest".as_ptr(), spawn: Some(SP_misc_teleporter_dest) },
    spawn_t { name: c"misc_model".as_ptr(), spawn: Some(SP_misc_model) },
    spawn_t { name: c"misc_model_static".as_ptr(), spawn: Some(SP_misc_model_static) },
    spawn_t { name: c"misc_G2model".as_ptr(), spawn: Some(SP_misc_G2model) },
    spawn_t { name: c"misc_portal_surface".as_ptr(), spawn: Some(SP_misc_portal_surface) },
    spawn_t { name: c"misc_portal_camera".as_ptr(), spawn: Some(SP_misc_portal_camera) },
    spawn_t { name: c"misc_weather_zone".as_ptr(), spawn: Some(SP_misc_weather_zone) },

    spawn_t { name: c"misc_bsp".as_ptr(), spawn: Some(SP_misc_bsp) },
    spawn_t { name: c"terrain".as_ptr(), spawn: Some(SP_terrain) },
    spawn_t { name: c"misc_skyportal_orient".as_ptr(), spawn: Some(SP_misc_skyportal_orient) },
    spawn_t { name: c"misc_skyportal".as_ptr(), spawn: Some(SP_misc_skyportal) },

    //rwwFIXMEFIXME: only for testing rmg team stuff
    spawn_t { name: c"gametype_item".as_ptr(), spawn: Some(SP_gametype_item) },

    spawn_t { name: c"misc_ammo_floor_unit".as_ptr(), spawn: Some(SP_misc_ammo_floor_unit) },
    spawn_t { name: c"misc_shield_floor_unit".as_ptr(), spawn: Some(SP_misc_shield_floor_unit) },
    spawn_t { name: c"misc_model_shield_power_converter".as_ptr(), spawn: Some(SP_misc_model_shield_power_converter) },
    spawn_t { name: c"misc_model_ammo_power_converter".as_ptr(), spawn: Some(SP_misc_model_ammo_power_converter) },
    spawn_t { name: c"misc_model_health_power_converter".as_ptr(), spawn: Some(SP_misc_model_health_power_converter) },

    spawn_t { name: c"fx_runner".as_ptr(), spawn: Some(SP_fx_runner) },

    spawn_t { name: c"target_screenshake".as_ptr(), spawn: Some(SP_target_screenshake) },
    spawn_t { name: c"target_escapetrig".as_ptr(), spawn: Some(SP_target_escapetrig) },

    spawn_t { name: c"misc_maglock".as_ptr(), spawn: Some(SP_misc_maglock) },

    spawn_t { name: c"misc_faller".as_ptr(), spawn: Some(SP_misc_faller) },

    spawn_t { name: c"ref_tag".as_ptr(), spawn: Some(SP_reference_tag) },
    spawn_t { name: c"ref_tag_huge".as_ptr(), spawn: Some(SP_reference_tag) },

    spawn_t { name: c"misc_weapon_shooter".as_ptr(), spawn: Some(SP_misc_weapon_shooter) },

    //new NPC ents
    spawn_t { name: c"NPC_spawner".as_ptr(), spawn: Some(SP_NPC_spawner) },

    spawn_t { name: c"NPC_Vehicle".as_ptr(), spawn: Some(SP_NPC_Vehicle) },
    spawn_t { name: c"NPC_Kyle".as_ptr(), spawn: Some(SP_NPC_Kyle) },
    spawn_t { name: c"NPC_Lando".as_ptr(), spawn: Some(SP_NPC_Lando) },
    spawn_t { name: c"NPC_Jan".as_ptr(), spawn: Some(SP_NPC_Jan) },
    spawn_t { name: c"NPC_Luke".as_ptr(), spawn: Some(SP_NPC_Luke) },
    spawn_t { name: c"NPC_MonMothma".as_ptr(), spawn: Some(SP_NPC_MonMothma) },
    spawn_t { name: c"NPC_Tavion".as_ptr(), spawn: Some(SP_NPC_Tavion) },

    //new tavion
    spawn_t { name: c"NPC_Tavion_New".as_ptr(), spawn: Some(SP_NPC_Tavion_New) },

    //new alora
    spawn_t { name: c"NPC_Alora".as_ptr(), spawn: Some(SP_NPC_Alora) },

    spawn_t { name: c"NPC_Reelo".as_ptr(), spawn: Some(SP_NPC_Reelo) },
    spawn_t { name: c"NPC_Galak".as_ptr(), spawn: Some(SP_NPC_Galak) },
    spawn_t { name: c"NPC_Desann".as_ptr(), spawn: Some(SP_NPC_Desann) },
    spawn_t { name: c"NPC_Bartender".as_ptr(), spawn: Some(SP_NPC_Bartender) },
    spawn_t { name: c"NPC_MorganKatarn".as_ptr(), spawn: Some(SP_NPC_MorganKatarn) },
    spawn_t { name: c"NPC_Jedi".as_ptr(), spawn: Some(SP_NPC_Jedi) },
    spawn_t { name: c"NPC_Prisoner".as_ptr(), spawn: Some(SP_NPC_Prisoner) },
    spawn_t { name: c"NPC_Rebel".as_ptr(), spawn: Some(SP_NPC_Rebel) },
    spawn_t { name: c"NPC_Stormtrooper".as_ptr(), spawn: Some(SP_NPC_Stormtrooper) },
    spawn_t { name: c"NPC_StormtrooperOfficer".as_ptr(), spawn: Some(SP_NPC_StormtrooperOfficer) },
    spawn_t { name: c"NPC_Snowtrooper".as_ptr(), spawn: Some(SP_NPC_Snowtrooper) },
    spawn_t { name: c"NPC_Tie_Pilot".as_ptr(), spawn: Some(SP_NPC_Tie_Pilot) },
    spawn_t { name: c"NPC_Ugnaught".as_ptr(), spawn: Some(SP_NPC_Ugnaught) },
    spawn_t { name: c"NPC_Jawa".as_ptr(), spawn: Some(SP_NPC_Jawa) },
    spawn_t { name: c"NPC_Gran".as_ptr(), spawn: Some(SP_NPC_Gran) },
    spawn_t { name: c"NPC_Rodian".as_ptr(), spawn: Some(SP_NPC_Rodian) },
    spawn_t { name: c"NPC_Weequay".as_ptr(), spawn: Some(SP_NPC_Weequay) },
    spawn_t { name: c"NPC_Trandoshan".as_ptr(), spawn: Some(SP_NPC_Trandoshan) },
    spawn_t { name: c"NPC_Tusken".as_ptr(), spawn: Some(SP_NPC_Tusken) },
    spawn_t { name: c"NPC_Noghri".as_ptr(), spawn: Some(SP_NPC_Noghri) },
    spawn_t { name: c"NPC_SwampTrooper".as_ptr(), spawn: Some(SP_NPC_SwampTrooper) },
    spawn_t { name: c"NPC_Imperial".as_ptr(), spawn: Some(SP_NPC_Imperial) },
    spawn_t { name: c"NPC_ImpWorker".as_ptr(), spawn: Some(SP_NPC_ImpWorker) },
    spawn_t { name: c"NPC_BespinCop".as_ptr(), spawn: Some(SP_NPC_BespinCop) },
    spawn_t { name: c"NPC_Reborn".as_ptr(), spawn: Some(SP_NPC_Reborn) },
    spawn_t { name: c"NPC_ShadowTrooper".as_ptr(), spawn: Some(SP_NPC_ShadowTrooper) },
    spawn_t { name: c"NPC_Monster_Murjj".as_ptr(), spawn: Some(SP_NPC_Monster_Murjj) },
    spawn_t { name: c"NPC_Monster_Swamp".as_ptr(), spawn: Some(SP_NPC_Monster_Swamp) },
    spawn_t { name: c"NPC_Monster_Howler".as_ptr(), spawn: Some(SP_NPC_Monster_Howler) },
    spawn_t { name: c"NPC_MineMonster".as_ptr(), spawn: Some(SP_NPC_MineMonster) },
    spawn_t { name: c"NPC_Monster_Claw".as_ptr(), spawn: Some(SP_NPC_Monster_Claw) },
    spawn_t { name: c"NPC_Monster_Glider".as_ptr(), spawn: Some(SP_NPC_Monster_Glider) },
    spawn_t { name: c"NPC_Monster_Flier2".as_ptr(), spawn: Some(SP_NPC_Monster_Flier2) },
    spawn_t { name: c"NPC_Monster_Lizard".as_ptr(), spawn: Some(SP_NPC_Monster_Lizard) },
    spawn_t { name: c"NPC_Monster_Fish".as_ptr(), spawn: Some(SP_NPC_Monster_Fish) },
    spawn_t { name: c"NPC_Monster_Wampa".as_ptr(), spawn: Some(SP_NPC_Monster_Wampa) },
    spawn_t { name: c"NPC_Monster_Rancor".as_ptr(), spawn: Some(SP_NPC_Monster_Rancor) },
    spawn_t { name: c"NPC_Droid_Interrogator".as_ptr(), spawn: Some(SP_NPC_Droid_Interrogator) },
    spawn_t { name: c"NPC_Droid_Probe".as_ptr(), spawn: Some(SP_NPC_Droid_Probe) },
    spawn_t { name: c"NPC_Droid_Mark1".as_ptr(), spawn: Some(SP_NPC_Droid_Mark1) },
    spawn_t { name: c"NPC_Droid_Mark2".as_ptr(), spawn: Some(SP_NPC_Droid_Mark2) },
    spawn_t { name: c"NPC_Droid_ATST".as_ptr(), spawn: Some(SP_NPC_Droid_ATST) },
    spawn_t { name: c"NPC_Droid_Seeker".as_ptr(), spawn: Some(SP_NPC_Droid_Seeker) },
    spawn_t { name: c"NPC_Droid_Remote".as_ptr(), spawn: Some(SP_NPC_Droid_Remote) },
    spawn_t { name: c"NPC_Droid_Sentry".as_ptr(), spawn: Some(SP_NPC_Droid_Sentry) },
    spawn_t { name: c"NPC_Droid_Gonk".as_ptr(), spawn: Some(SP_NPC_Droid_Gonk) },
    spawn_t { name: c"NPC_Droid_Mouse".as_ptr(), spawn: Some(SP_NPC_Droid_Mouse) },
    spawn_t { name: c"NPC_Droid_R2D2".as_ptr(), spawn: Some(SP_NPC_Droid_R2D2) },
    spawn_t { name: c"NPC_Droid_R5D2".as_ptr(), spawn: Some(SP_NPC_Droid_R5D2) },
    spawn_t { name: c"NPC_Droid_Protocol".as_ptr(), spawn: Some(SP_NPC_Droid_Protocol) },

    //maybe put these guys in some day, for now just spawn reborns in their place.
    spawn_t { name: c"NPC_Reborn_New".as_ptr(), spawn: Some(SP_NPC_Reborn_New) },
    spawn_t { name: c"NPC_Cultist".as_ptr(), spawn: Some(SP_NPC_Cultist) },
    spawn_t { name: c"NPC_Cultist_Saber".as_ptr(), spawn: Some(SP_NPC_Cultist_Saber) },
    spawn_t { name: c"NPC_Cultist_Saber_Powers".as_ptr(), spawn: Some(SP_NPC_Cultist_Saber_Powers) },
    spawn_t { name: c"NPC_Cultist_Destroyer".as_ptr(), spawn: Some(SP_NPC_Cultist_Destroyer) },
    spawn_t { name: c"NPC_Cultist_Commando".as_ptr(), spawn: Some(SP_NPC_Cultist_Commando) },

    //rwwFIXMEFIXME: Faked for testing NPCs (another other things) in RMG with sof2 assets
    spawn_t { name: c"NPC_Colombian_Soldier".as_ptr(), spawn: Some(SP_NPC_Reborn) },
    spawn_t { name: c"NPC_Colombian_Rebel".as_ptr(), spawn: Some(SP_NPC_Reborn) },
    spawn_t { name: c"NPC_Colombian_EmplacedGunner".as_ptr(), spawn: Some(SP_NPC_ShadowTrooper) },
    spawn_t { name: c"NPC_Manuel_Vergara_RMG".as_ptr(), spawn: Some(SP_NPC_Desann) },
    //	{"info_NPCnav", SP_waypoint},

    spawn_t { name: c"waypoint".as_ptr(), spawn: Some(SP_waypoint) },
    spawn_t { name: c"waypoint_small".as_ptr(), spawn: Some(SP_waypoint_small) },
    spawn_t { name: c"waypoint_navgoal".as_ptr(), spawn: Some(SP_waypoint_navgoal) },
    spawn_t { name: c"waypoint_navgoal_8".as_ptr(), spawn: Some(SP_waypoint_navgoal_8) },
    spawn_t { name: c"waypoint_navgoal_4".as_ptr(), spawn: Some(SP_waypoint_navgoal_4) },
    spawn_t { name: c"waypoint_navgoal_2".as_ptr(), spawn: Some(SP_waypoint_navgoal_2) },
    spawn_t { name: c"waypoint_navgoal_1".as_ptr(), spawn: Some(SP_waypoint_navgoal_1) },

    spawn_t { name: c"fx_spacedust".as_ptr(), spawn: Some(SP_CreateSpaceDust) },
    spawn_t { name: c"fx_rain".as_ptr(), spawn: Some(SP_CreateRain) },
    spawn_t { name: c"fx_snow".as_ptr(), spawn: Some(SP_CreateSnow) },

    spawn_t { name: c"point_combat".as_ptr(), spawn: Some(SP_point_combat) },

    spawn_t { name: c"misc_holocron".as_ptr(), spawn: Some(SP_misc_holocron) },

    spawn_t { name: c"shooter_blaster".as_ptr(), spawn: Some(SP_shooter_blaster) },

    spawn_t { name: c"team_CTF_redplayer".as_ptr(), spawn: Some(SP_team_CTF_redplayer) },
    spawn_t { name: c"team_CTF_blueplayer".as_ptr(), spawn: Some(SP_team_CTF_blueplayer) },

    spawn_t { name: c"team_CTF_redspawn".as_ptr(), spawn: Some(SP_team_CTF_redspawn) },
    spawn_t { name: c"team_CTF_bluespawn".as_ptr(), spawn: Some(SP_team_CTF_bluespawn) },

    spawn_t { name: c"item_botroam".as_ptr(), spawn: Some(SP_item_botroam) },

    spawn_t { name: c"emplaced_gun".as_ptr(), spawn: Some(sp_emplaced_gun) },

    spawn_t { name: c"misc_turret".as_ptr(), spawn: Some(SP_misc_turret) },
    spawn_t { name: c"misc_turretG2".as_ptr(), spawn: Some(SP_misc_turretG2) },

    spawn_t { name: null_mut(), spawn: None },
];

/// `qboolean G_CallSpawn( gentity_t *ent )` (g_spawn.c:675).
///
/// Finds and calls the spawn function for `ent`'s `classname`, returning `qtrue`
/// if found. First scans `bg_itemlist` (skipping element 0) and, on a match,
/// spawns the item via [`G_SpawnItem`]; otherwise scans the [`spawns`] table.
/// Before invoking a table handler, a non-empty `healingsound` is precached via
/// [`G_SoundIndex`] (the carried-over `//yeah...this can be used for anything`
/// quirk). Prints a diagnostic and returns `qfalse` when nothing matches.
/// No oracle — dispatches into entity-state spawn handlers and reads the global
/// item list.
///
/// # Safety
/// `ent` must be a valid `gentity_t *`; `bg_itemlist` must be initialized.
pub unsafe fn G_CallSpawn(ent: *mut gentity_t) -> qboolean {
    if (*ent).classname.is_null() {
        G_Printf("G_CallSpawn: NULL classname\n");
        return QFALSE;
    }

    // check item spawn functions
    // C: `for ( item=bg_itemlist+1 ; item->classname ; item++ )`
    let mut idx: usize = 1;
    let itemlist = addr_of_mut!(bg_itemlist);
    while !(*itemlist)[idx].classname.is_null() {
        let item: *mut gitem_t = addr_of_mut!((*itemlist)[idx]);
        if strcmp((*item).classname, (*ent).classname) == 0 {
            G_SpawnItem(ent, item);
            return QTRUE;
        }
        idx += 1;
    }

    // check normal spawn functions
    let mut si: usize = 0;
    while !spawns[si].name.is_null() {
        let s = &spawns[si];
        if strcmp(s.name, (*ent).classname) == 0 {
            // found it
            if !(*ent).healingsound.is_null() && *(*ent).healingsound != 0 {
                //yeah...this can be used for anything, so.. precache it if it's there
                G_SoundIndex(&CStr::from_ptr((*ent).healingsound).to_string_lossy());
            }
            (s.spawn.unwrap())(ent);
            return QTRUE;
        }
        si += 1;
    }
    G_Printf(&format!(
        "{} doesn't have a spawn function\n",
        Sz((*ent).classname)
    ));
    QFALSE
}

/*
===================
G_SpawnGEntityFromSpawnVars

Spawn an entity and fill in all of the level fields from
level.spawnVars[], then call the class specfic spawn function
===================
*/
/// `void G_SpawnGEntityFromSpawnVars( qboolean inSubBSP )` (g_spawn.c:758).
///
/// Allocates a free [`gentity_t`] via [`G_Spawn`], applies every parsed spawn var
/// to it through [`BG_ParseField`] against the [`fields`] table, then enforces the
/// `notsingle`/`notteam`/`notfree`/`notta`/`gametype` filters (freeing the entity and
/// returning if it should not exist in this game type). Copies the editor origin into
/// the trajectory/shared origin, dispatches to the class spawn handler via
/// [`G_CallSpawn`] (freeing on failure), and finally tags ICARUS scripting onto valid
/// non-`NPC_` recipients ([`trap::ICARUS_ValidEnt`]/[`trap::ICARUS_InitEnt`] +
/// [`G_ActivateBehavior`]`(BSET_SPAWN)`). No oracle — drives entity-state allocation,
/// the global `level`/`g_gametype`, and the ICARUS traps.
///
/// # Safety
/// Must run on the game thread that owns `level`/`g_entities`, after [`G_ParseSpawnVars`].
pub unsafe fn G_SpawnGEntityFromSpawnVars(_inSubBSP: qboolean) {
    let mut i: c_int;
    // static char *gametypeNames[] = {...}
    static GAMETYPE_NAMES: [&CStr; 10] = [
        c"ffa", c"holocron", c"jedimaster", c"duel", c"powerduel", c"single", c"team", c"siege",
        c"ctf", c"cty",
    ];

    // get the next free entity
    let ent: *mut gentity_t = G_Spawn();

    let lvl = addr_of!(level);
    i = 0;
    while i < (*lvl).numSpawnVars {
        BG_ParseField(
            fields.0.as_ptr() as *mut BG_field_t,
            (*lvl).spawnVars[i as usize][0],
            (*lvl).spawnVars[i as usize][1],
            ent as *mut u8,
        );
        i += 1;
    }

    // check for "notsingle" flag
    if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER {
        G_SpawnInt(c"notsingle".as_ptr(), c"0".as_ptr(), &mut i);
        if i != 0 {
            G_FreeEntity(ent);
            return;
        }
    }
    // check for "notteam" flag (GT_FFA, GT_DUEL, GT_SINGLE_PLAYER)
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        G_SpawnInt(c"notteam".as_ptr(), c"0".as_ptr(), &mut i);
        if i != 0 {
            G_FreeEntity(ent);
            return;
        }
    } else {
        G_SpawnInt(c"notfree".as_ptr(), c"0".as_ptr(), &mut i);
        if i != 0 {
            G_FreeEntity(ent);
            return;
        }
    }

    G_SpawnInt(c"notta".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        G_FreeEntity(ent);
        return;
    }

    let mut value: *mut c_char = null_mut();
    if G_SpawnString(c"gametype".as_ptr(), null_mut(), &mut value) != QFALSE {
        let gt = (*addr_of!(g_gametype)).integer;
        if gt >= GT_FFA && gt < GT_MAX_GAME_TYPE {
            let gametype_name = GAMETYPE_NAMES[gt as usize].as_ptr();

            let s = strstr(value, gametype_name);
            if s.is_null() {
                G_FreeEntity(ent);
                return;
            }
        }
    }

    // move editor origin to pos
    let o = (*ent).s.origin;
    (*ent).s.pos.trBase = o;
    (*ent).r.currentOrigin = o;

    // if we didn't get a classname, don't bother spawning anything
    if G_CallSpawn(ent) == QFALSE {
        G_FreeEntity(ent);
    }

    //Tag on the ICARUS scripting information only to valid recipients
    if trap::ICARUS_ValidEnt(ent) != QFALSE {
        trap::ICARUS_InitEnt(ent);

        if !(*ent).classname.is_null() && *(*ent).classname != 0 {
            if Q_strncmp(c"NPC_".as_ptr(), (*ent).classname, 4) != 0 {
                //Not an NPC_spawner (rww - probably don't even care for MP, but whatever)
                G_ActivateBehavior(ent, BSET_SPAWN);
            }
        }
    }
}

// `static char *defaultStyles[32][3]` (g_spawn.c:1062) — the default light-style
// animation strings (one R/G/B triple per of the 32 base styles). C uses `char *`;
// here `*const c_char` from `c"…"` literals. Empty styles are `c""`.
static DEFAULT_STYLES: [[&CStr; 3]; 32] = [
    // 0 normal
    [c"z", c"z", c"z"],
    // 1 FLICKER (first variety)
    [c"mmnmmommommnonmmonqnmmo", c"mmnmmommommnonmmonqnmmo", c"mmnmmommommnonmmonqnmmo"],
    // 2 SLOW STRONG PULSE
    [
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb",
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb",
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb",
    ],
    // 3 CANDLE (first variety)
    [
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg",
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg",
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg",
    ],
    // 4 FAST STROBE
    [c"mamamamamama", c"mamamamamama", c"mamamamamama"],
    // 5 GENTLE PULSE 1
    [
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj",
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj",
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj",
    ],
    // 6 FLICKER (second variety)
    [c"nmonqnmomnmomomno", c"nmonqnmomnmomomno", c"nmonqnmomnmomomno"],
    // 7 CANDLE (second variety)
    [c"mmmaaaabcdefgmmmmaaaammmaamm", c"mmmaaaabcdefgmmmmaaaammmaamm", c"mmmaaaabcdefgmmmmaaaammmaamm"],
    // 8 CANDLE (third variety)
    [
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa",
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa",
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa",
    ],
    // 9 SLOW STROBE (fourth variety)
    [c"aaaaaaaazzzzzzzz", c"aaaaaaaazzzzzzzz", c"aaaaaaaazzzzzzzz"],
    // 10 FLUORESCENT FLICKER
    [c"mmamammmmammamamaaamammma", c"mmamammmmammamamaaamammma", c"mmamammmmammamamaaamammma"],
    // 11 SLOW PULSE NOT FADE TO BLACK
    [
        c"abcdefghijklmnopqrrqponmlkjihgfedcba",
        c"abcdefghijklmnopqrrqponmlkjihgfedcba",
        c"abcdefghijklmnopqrrqponmlkjihgfedcba",
    ],
    // 12 FAST PULSE FOR JEREMY
    [c"mkigegik", c"mkigegik", c"mkigegik"],
    // 13 Test Blending
    [c"abcdefghijklmqrstuvwxyz", c"zyxwvutsrqmlkjihgfedcba", c"aammbbzzccllcckkffyyggp"],
    // 14
    [c"", c"", c""],
    // 15
    [c"", c"", c""],
    // 16
    [c"", c"", c""],
    // 17
    [c"", c"", c""],
    // 18
    [c"", c"", c""],
    // 19
    [c"", c"", c""],
    // 20
    [c"", c"", c""],
    // 21
    [c"", c"", c""],
    // 22
    [c"", c"", c""],
    // 23
    [c"", c"", c""],
    // 24
    [c"", c"", c""],
    // 25
    [c"", c"", c""],
    // 26
    [c"", c"", c""],
    // 27
    [c"", c"", c""],
    // 28
    [c"", c"", c""],
    // 29
    [c"", c"", c""],
    // 30
    [c"", c"", c""],
    // 31
    [c"", c"", c""],
];

// `#define LS_STYLES_START 0` / `#define LS_NUM_STYLES 32` (q_shared.h:423) — not yet
// ported into q_shared_h.rs; defined here file-locally (matching the C `#include` path)
// until that header lands.
const LS_STYLES_START: c_int = 0;
const LS_NUM_STYLES: c_int = 32;

/// `void *precachedKyle = 0;` (g_spawn.c:1226) — the server's precached Kyle template
/// ghoul2 instance, initialised once in [`SP_worldspawn`]. `pub` so the ghoul2
/// cleanup in [`G_ShutdownGame`](super::g_main::G_ShutdownGame) can release it.
#[allow(non_upper_case_globals)]
pub static mut precachedKyle: *mut core::ffi::c_void = null_mut();

/// `float g_cullDistance;` (g_spawn.c:1250) — the per-map net-cull distance set from the
/// `distanceCull` worldspawn key and pushed to the engine via [`trap::SetServerCull`].
#[allow(non_upper_case_globals)]
pub(crate) static mut g_cullDistance: f32 = 0.0;

/*QUAKED worldspawn (0 0 0) ?

Every map should have exactly one worldspawn.
"music"		music wav file
"gravity"	800 is default gravity
"message"	Text to print during connection process

BSP Options
"gridsize"     size of lighting grid to "X Y Z". default="64 64 128"
"ambient"      scale of global light (from _color)
"fog"          shader name of the global fog texture - must include the full path, such as "textures/rj/fog1"
"distancecull" value for vis for the maximum viewing distance
"chopsize"     value for bsp on the maximum polygon / portal size
"ls_Xr"	override lightstyle X with this pattern for Red.
"ls_Xg"	green (valid patterns are "a-z")
"ls_Xb"	blue (a is OFF, z is ON)

"fogstart"		override fog start distance and force linear
"radarrange" for Siege/Vehicle radar - default range is 2500
*/
/// `void SP_worldspawn( void )` (g_spawn.c:1251).
///
/// The global level setup driven by the `worldspawn` entity's spawn vars: sets the
/// net-cull distance, validates the first entity is `worldspawn`, copies only the
/// `spawnscript` field onto the world ent, precaches the humanoid animations and the
/// Kyle/saber ghoul2 templates, pushes the game-version/start-time/music/message/MOTD
/// configstrings and the gravity/dust/breath cvars, names the world ent, clears the
/// warmup, and installs the 32 default light styles (reading any `ls_Nr/g/b` overrides
/// and fatal-erroring on a R/G/B length mismatch). No oracle — drives the global
/// `level`/`g_entities`, the cvars, and the configstring/ghoul2 traps.
///
/// Deviations carried over: the commented-out `g_doWarmup` block is preserved as a
/// comment; the `lengthRed/Green/Blue` locals are read straight from the just-set
/// strings to mirror the original consistency check.
///
/// # Safety
/// Must run on the game thread that owns `level`/`g_entities`, after [`G_ParseSpawnVars`].
pub unsafe fn SP_worldspawn() {
    let mut text: *mut c_char = null_mut();
    let mut temp = [0 as c_char; 32];
    let mut i: c_int;

    //I want to "cull" entities out of net sends to clients to reduce
    //net traffic on our larger open maps -rww
    G_SpawnFloat(c"distanceCull".as_ptr(), c"6000.0".as_ptr(), addr_of_mut!(g_cullDistance));
    trap::SetServerCull(g_cullDistance);

    G_SpawnString(c"classname".as_ptr(), c"".as_ptr(), &mut text);
    if Q_stricmp(text, c"worldspawn".as_ptr()) != 0 {
        G_Error("SP_worldspawn: The first entity isn't 'worldspawn'");
    }

    let lvl = addr_of!(level);
    i = 0;
    while i < (*lvl).numSpawnVars {
        if Q_stricmp(c"spawnscript".as_ptr(), (*lvl).spawnVars[i as usize][0]) == 0 {
            //ONly let them set spawnscript, we don't want them setting an angle or something on the world.
            BG_ParseField(
                fields.0.as_ptr() as *mut BG_field_t,
                (*lvl).spawnVars[i as usize][0],
                (*lvl).spawnVars[i as usize][1],
                core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(ENTITYNUM_WORLD as usize) as *mut u8,
            );
        }
        i += 1;
    }
    //The server will precache the standard model and animations, so that there is no hit
    //when the first client connnects.
    if BGPAFtextLoaded == QFALSE {
        BG_ParseAnimationFile(
            c"models/players/_humanoid/animation.cfg".as_ptr(),
            addr_of_mut!(bgHumanoidAnimations) as *mut _,
            QTRUE,
        );
    }

    if precachedKyle.is_null() {
        trap::G2API_InitGhoul2Model(
            addr_of_mut!(precachedKyle),
            c"models/players/kyle/model.glm".as_ptr(),
            0,
            0,
            -20,
            0,
            0,
        );

        if !precachedKyle.is_null() {
            let def_skin = trap::R_RegisterSkin("models/players/kyle/model_default.skin");
            trap::G2API_SetSkin(precachedKyle, 0, def_skin, def_skin);
        }
    }

    if g2SaberInstance.is_null() {
        trap::G2API_InitGhoul2Model(
            addr_of_mut!(g2SaberInstance),
            c"models/weapons2/saber/saber_w.glm".as_ptr(),
            0,
            0,
            -20,
            0,
            0,
        );

        if !g2SaberInstance.is_null() {
            // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
            trap::G2API_SetBoltInfo(g2SaberInstance, 0, 0);
            // now set up the gun bolt on it
            trap::G2API_AddBolt(g2SaberInstance, 0, "*blade1");
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //a tad bit of a hack, but..
        EWebPrecache();
    }

    // make some data visible to connecting client
    trap::SetConfigstring(CS_GAME_VERSION, GAME_VERSION);

    trap::SetConfigstring(CS_LEVEL_START_TIME, &format!("{}", (*lvl).startTime));

    G_SpawnString(c"music".as_ptr(), c"".as_ptr(), &mut text);
    trap::SetConfigstring(CS_MUSIC, &CStr::from_ptr(text).to_string_lossy());

    G_SpawnString(c"message".as_ptr(), c"".as_ptr(), &mut text);
    trap::SetConfigstring(CS_MESSAGE, &CStr::from_ptr(text).to_string_lossy()); // map specific message

    trap::SetConfigstring(
        CS_MOTD,
        &CStr::from_ptr((*addr_of!(g_motd)).string.as_ptr()).to_string_lossy(),
    ); // message of the day

    G_SpawnString(c"gravity".as_ptr(), c"800".as_ptr(), &mut text);
    trap::Cvar_Set("g_gravity", &CStr::from_ptr(text).to_string_lossy());

    G_SpawnString(c"enableBreath".as_ptr(), c"0".as_ptr(), &mut text);
    trap::Cvar_Set("g_enableBreath", &CStr::from_ptr(text).to_string_lossy());

    G_SpawnString(c"soundSet".as_ptr(), c"default".as_ptr(), &mut text);
    trap::SetConfigstring(CS_GLOBAL_AMBIENT_SET, &CStr::from_ptr(text).to_string_lossy());

    let world = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(ENTITYNUM_WORLD as usize);
    (*world).s.number = ENTITYNUM_WORLD;
    (*world).classname = c"worldspawn".as_ptr() as *mut c_char;

    // see if we want a warmup time
    trap::SetConfigstring(CS_WARMUP, "");
    if (*addr_of!(g_restarted)).integer != 0 {
        trap::Cvar_Set("g_restarted", "0");
        (*addr_of_mut!(level)).warmupTime = 0;
    }
    /*
    else if ( g_doWarmup.integer && g_gametype.integer != GT_DUEL && g_gametype.integer != GT_POWERDUEL ) { // Turn it on
        level.warmupTime = -1;
        trap_SetConfigstring( CS_WARMUP, va("%i", level.warmupTime) );
        G_LogPrintf( "Warmup:\n" );
    }
    */

    trap::SetConfigstring(
        CS_LIGHT_STYLES + (LS_STYLES_START * 3) + 0,
        &CStr::from_ptr(DEFAULT_STYLES[0][0].as_ptr()).to_string_lossy(),
    );
    trap::SetConfigstring(
        CS_LIGHT_STYLES + (LS_STYLES_START * 3) + 1,
        &CStr::from_ptr(DEFAULT_STYLES[0][1].as_ptr()).to_string_lossy(),
    );
    trap::SetConfigstring(
        CS_LIGHT_STYLES + (LS_STYLES_START * 3) + 2,
        &CStr::from_ptr(DEFAULT_STYLES[0][2].as_ptr()).to_string_lossy(),
    );

    i = 1;
    while i < LS_NUM_STYLES {
        Com_sprintf(temp.as_mut_ptr(), 32, format_args!("ls_{}r", i));
        G_SpawnString(temp.as_ptr(), DEFAULT_STYLES[i as usize][0].as_ptr(), &mut text);
        let length_red = strlen(text) as c_int;
        trap::SetConfigstring(
            CS_LIGHT_STYLES + ((i + LS_STYLES_START) * 3) + 0,
            &CStr::from_ptr(text).to_string_lossy(),
        );

        Com_sprintf(temp.as_mut_ptr(), 32, format_args!("ls_{}g", i));
        G_SpawnString(temp.as_ptr(), DEFAULT_STYLES[i as usize][1].as_ptr(), &mut text);
        let length_green = strlen(text) as c_int;
        trap::SetConfigstring(
            CS_LIGHT_STYLES + ((i + LS_STYLES_START) * 3) + 1,
            &CStr::from_ptr(text).to_string_lossy(),
        );

        Com_sprintf(temp.as_mut_ptr(), 32, format_args!("ls_{}b", i));
        G_SpawnString(temp.as_ptr(), DEFAULT_STYLES[i as usize][2].as_ptr(), &mut text);
        let length_blue = strlen(text) as c_int;
        trap::SetConfigstring(
            CS_LIGHT_STYLES + ((i + LS_STYLES_START) * 3) + 2,
            &CStr::from_ptr(text).to_string_lossy(),
        );

        if length_red != length_green || length_green != length_blue {
            Com_Error(
                ERR_DROP,
                &format!(
                    "Style {} has inconsistent lengths: R {}, G {}, B {}",
                    i, length_red, length_green, length_blue
                ),
            );
        }
        i += 1;
    }
}

/*
==============
G_SpawnEntitiesFromString

Parses textual entity definitions out of an entstring and spawns gentities.
==============
*/
/// `void G_SpawnEntitiesFromString( qboolean inSubBSP )` (g_spawn.c:1419).
///
/// The top-level map spawn driver: enables `G_Spawn*()`, parses the first block and
/// runs the worldspawn ([`SP_worldspawn`], or [`SP_bsp_worldspawn`] for a sub-BSP —
/// bailing if that fails), then loops [`G_ParseSpawnVars`]/[`G_SpawnGEntityFromSpawnVars`]
/// over the remaining ents. If the world carries a `BSET_SPAWN` script it spins up a
/// hidden scriptrunner ([`scriptrunner_run`] think) to run it outside ICARUS. Finally
/// disables spawning (top-level only) and precaches soundsets. No oracle — drives the
/// whole global spawn pipeline and the entity-token / ICARUS traps.
///
/// # Safety
/// Must run on the game thread that owns `level`/`g_entities` during map init.
pub unsafe fn G_SpawnEntitiesFromString(inSubBSP: qboolean) {
    // allow calls to G_Spawn*()
    (*addr_of_mut!(level)).spawning = QTRUE;
    (*addr_of_mut!(level)).numSpawnVars = 0;

    // the worldspawn is not an actual entity, but it still
    // has a "spawn" function to perform any global setup
    // needed by a level (setting configstrings or cvars, etc)
    if G_ParseSpawnVars(QFALSE) == QFALSE {
        G_Error("SpawnEntities: no entities");
    }

    if inSubBSP == QFALSE {
        SP_worldspawn();
    } else {
        // Skip this guy if its worldspawn fails
        if SP_bsp_worldspawn() == QFALSE {
            return;
        }
    }

    // parse ents
    while G_ParseSpawnVars(inSubBSP) != QFALSE {
        G_SpawnGEntityFromSpawnVars(inSubBSP);
    }

    let world = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(ENTITYNUM_WORLD as usize);
    if !(*world).behaviorSet[BSET_SPAWN as usize].is_null()
        && *(*world).behaviorSet[BSET_SPAWN as usize] != 0
    {
        //World has a spawn script, but we don't want the world in ICARUS and running scripts,
        //so make a scriptrunner and start it going.
        let script_runner = G_Spawn();
        if !script_runner.is_null() {
            (*script_runner).behaviorSet[BSET_USE as usize] =
                (*world).behaviorSet[BSET_SPAWN as usize];
            (*script_runner).count = 1;
            (*script_runner).think = Some(scriptrunner_run);
            (*script_runner).nextthink = (*addr_of!(level)).time + 100;

            if (*script_runner).inuse != QFALSE {
                trap::ICARUS_InitEnt(script_runner);
            }
        }
    }

    if inSubBSP == QFALSE {
        (*addr_of_mut!(level)).spawning = QFALSE; // any future calls to G_Spawn*() will be errors
    }

    G_PrecacheSoundsets();
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::g_mem::{G_InitMemory, POOL_LOCK};
    use crate::oracle;
    use std::ffi::{CStr, CString};

    #[test]
    fn g_newstring_matches_oracle() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        G_InitMemory();

        // empty, no-escape, the `\n`->LF translation, `\<other>`->`\`, adjacent and
        // repeated escapes, an already-real newline, and the trailing-backslash quirk.
        let cases: &[&[u8]] = &[
            b"",
            b"hello world",
            b"line1\\nline2",
            b"a\\tb",
            b"\\n",
            b"\\\\",
            b"\\n\\n\\x",
            b"real\nnewline",
            b"trailing\\",
            b"mix \\n and \\q and text",
        ];
        for &bytes in cases {
            let cs = CString::new(bytes).unwrap();
            let got = unsafe { G_NewString(cs.as_ptr()) };
            let want = unsafe { oracle::jka_G_NewString(cs.as_ptr()) };
            let got_s = unsafe { CStr::from_ptr(got) };
            let want_s = unsafe { CStr::from_ptr(want) };
            assert_eq!(
                got_s.to_bytes(),
                want_s.to_bytes(),
                "G_NewString mismatch for {bytes:?}"
            );
        }
    }
}
