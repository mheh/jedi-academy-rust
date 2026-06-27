//! Miscellaneous syscall wrappers — `trap_ROFF_*`, `trap_SiegePers*`,
//! `trap_SP_GetStringTextString`, `trap_CM_RegisterTerrain`, `trap_RMG_Init`,
//! `trap_SetActiveSubBSP`, `trap_SetServerCull`, `trap_TrueMalloc/Free`,
//! `trap_RealTime`, `trap_DebugPolygon*`, `trap_PrecisionTimer_*`. 1:1 with
//! `refs/raven-jediacademy/codemp/game/g_syscalls.c`.
//! Types: q_shared_h::{qtime_t, siegePers_t}.

use core::ffi::{c_char, c_void};

use crate::codemp::game::q_shared_h::{qtime_t, siegePers_t, vec3_t};
use crate::ffi::syscalls::pass_float;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport::*;

/// `trap_SetServerCull` — server culling to reduce traffic on open maps. The
/// float arg rides the integer syscall ABI via `pass_float` (C `PASSFLOAT`).
pub fn SetServerCull(cull_distance: f32) {
    unsafe {
        syscall!(G_SET_SERVER_CULL, pass_float(cull_distance));
    }
}

/// `trap_SiegePersSet` — store the siege persistent data the engine carries
/// across a map change.
pub fn SiegePersSet(pers: &siegePers_t) {
    unsafe {
        syscall!(G_SIEGEPERSSET, pers as *const siegePers_t);
    }
}

/// `trap_SiegePersGet` — read back the engine's siege persistent data into
/// `pers`.
pub fn SiegePersGet(pers: &mut siegePers_t) {
    unsafe {
        syscall!(G_SIEGEPERSGET, pers as *mut siegePers_t);
    }
}

/// `trap_DebugPolygonCreate` — register a debug polygon (`numPoints` of `points`)
/// in `color`, returning its id. `points` is a raw pointer to a `vec3_t` array
/// (the caller owns the storage), kept verbatim from the C `vec3_t *points`.
pub fn DebugPolygonCreate(color: i32, num_points: i32, points: *mut vec3_t) -> i32 {
    unsafe { syscall!(G_DEBUG_POLYGON_CREATE, color, num_points, points) as i32 }
}

/// `trap_DebugPolygonDelete` — remove the debug polygon `id`.
pub fn DebugPolygonDelete(id: i32) {
    unsafe {
        syscall!(G_DEBUG_POLYGON_DELETE, id);
    }
}

/// `trap_RealTime` — read the engine's wall-clock time into `qtime`, returning
/// the raw seconds-since-epoch value.
pub fn RealTime(qtime: &mut qtime_t) -> i32 {
    unsafe { syscall!(G_REAL_TIME, qtime as *mut qtime_t) as i32 }
}

/// `trap_PrecisionTimer_Start` — allocate a high-precision timer, writing its
/// handle through `the_new_timer` (a `void **`). Pair with [`PrecisionTimer_End`].
pub fn PrecisionTimer_Start(the_new_timer: *mut *mut c_void) {
    unsafe {
        syscall!(G_PRECISIONTIMER_START, the_new_timer);
    }
}

/// `trap_PrecisionTimer_End` — stop the timer `the_timer` (from
/// [`PrecisionTimer_Start`]) and return its elapsed measurement. If you're using
/// the above, the appropriate call is `let result = PrecisionTimer_End(blah)`.
pub fn PrecisionTimer_End(the_timer: *mut c_void) -> i32 {
    unsafe { syscall!(G_PRECISIONTIMER_END, the_timer) as i32 }
}

/// `trap_SP_GetStringTextString` — look up the string-package reference `text`
/// and copy its localized text into `buffer`. The C `( ..., char *buffer, int
/// bufferLength )` pair collapses to a `&mut [c_char]` whose length supplies
/// `bufferLength`; the int return is the engine's result.
pub fn SP_GetStringTextString(text: &str, buffer: &mut [c_char]) -> i32 {
    let t = super::cstr(text);
    unsafe {
        syscall!(
            SP_GETSTRINGTEXTSTRING,
            t.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as i32
        ) as i32
    }
}

/// `trap_ROFF_Clean` — flush all cached ROFF (rotation/origin animation) data.
pub fn ROFF_Clean() -> qboolean {
    unsafe { syscall!(G_ROFF_CLEAN) as qboolean }
}

/// `trap_ROFF_UpdateEntities` — advance every entity currently playing a ROFF.
pub fn ROFF_UpdateEntities() {
    unsafe {
        syscall!(G_ROFF_UPDATE_ENTITIES);
    }
}

/// `trap_ROFF_Cache` — load/cache the ROFF file `file`, returning its id (or 0
/// on failure).
pub fn ROFF_Cache(file: &str) -> i32 {
    let f = super::cstr(file);
    unsafe { syscall!(G_ROFF_CACHE, f.as_ptr()) as i32 }
}

/// `trap_ROFF_Play` — start ROFF `roff_id` on entity `ent_id`, optionally
/// translating its origin (`do_translation`). Returns `qtrue` on success.
pub fn ROFF_Play(ent_id: i32, roff_id: i32, do_translation: qboolean) -> qboolean {
    unsafe { syscall!(G_ROFF_PLAY, ent_id, roff_id, do_translation) as qboolean }
}

/// `trap_ROFF_Purge_Ent` — stop and clear the ROFF playing on entity `ent_id`.
pub fn ROFF_Purge_Ent(ent_id: i32) -> qboolean {
    unsafe { syscall!(G_ROFF_PURGE_ENT, ent_id) as qboolean }
}

/// `trap_TrueMalloc` — dynamic VM memory allocation: ask the engine for `size`
/// bytes, writing the pointer through `ptr` (a `void **`).
pub fn TrueMalloc(ptr: *mut *mut c_void, size: i32) {
    unsafe {
        syscall!(G_TRUEMALLOC, ptr, size);
    }
}

/// `trap_TrueFree` — release a block from [`TrueMalloc`] and NULL the slot `ptr`
/// (a `void **`).
pub fn TrueFree(ptr: *mut *mut c_void) {
    unsafe {
        syscall!(G_TRUEFREE, ptr);
    }
}

/// `trap_SetActiveSubBSP` — select the active sub-BSP `index` (RMG random-map
/// generation). `index < 0` clears it.
pub fn SetActiveSubBSP(index: i32) {
    unsafe {
        syscall!(G_SET_ACTIVE_SUBBSP, index);
    }
}

/// `trap_CM_RegisterTerrain` — register the terrain described by `config` with
/// the collision model (RMG), returning its terrain id.
pub fn CM_RegisterTerrain(config: &str) -> i32 {
    let c = super::cstr(config);
    unsafe { syscall!(G_CM_REGISTER_TERRAIN, c.as_ptr()) as i32 }
}

/// `trap_RMG_Init` — initialise the random-map generator against terrain
/// `terrain_id`.
pub fn RMG_Init(terrain_id: i32) {
    unsafe {
        syscall!(G_RMG_INIT, terrain_id);
    }
}
