//! Safe Rust wrappers over the engine syscalls — the `trap_*` functions.
//!
//! Each corresponds 1:1 to a `trap_X` in JKA's `refs/raven-jediacademy/codemp/game/g_syscalls.c`
//! and is named `X` here, so call sites read `trap::X(..)`. Signatures were
//! verified against the original-JKA source. Only the primitive / string / buffer
//! traps needed to bring the module up are implemented so far; more get added as
//! the port needs them.

use core::ffi::{c_char, c_int};
use std::ffi::CString;

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::{
    fsMode_t, mdxaBone_t, playerState_t, qhandle_t, sharedIKMoveParams_t,
    sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t, trace_t, usercmd_t, vec3_t,
    CollisionRecord_t,
};
use crate::ffi::syscalls::pass_float;
use crate::ffi::types::{fileHandle_t, qboolean, vmCvar_t};
use crate::ffi::GameImport::*;
use core::ffi::c_void;

// Deferred trap-layer subsystems, partitioned by `GameImport` enum range. Each
// submodule's wrappers land in Phase B; the foundation only scaffolds them so the
// parallel workers never touch a shared file. Re-exported flat so call sites stay
// `trap::X(..)` regardless of which submodule X lives in.
pub mod aas;
pub mod botlib_ai;
pub mod botlib_core;
pub mod ea;
pub mod ghoul2_ext;
pub mod misc;
pub mod nav;
// `allow(unused_imports)`: the submodules are empty until Phase B populates them, so
// the glob re-exports have nothing to re-export yet — drop the allow once filled.
#[allow(unused_imports)]
pub use aas::*;
#[allow(unused_imports)]
pub use botlib_ai::*;
#[allow(unused_imports)]
pub use botlib_core::*;
#[allow(unused_imports)]
pub use ghoul2_ext::*;
#[allow(unused_imports)]
pub use misc::*;
#[allow(unused_imports)]
pub use nav::*;

/// Default scratch size for engine→module string outputs (`MAX_STRING_CHARS`).
const STRING_BUF: usize = 8192;

/// Build a NUL-terminated C string, stripping any interior NULs rather than
/// failing (engine strings are best-effort and must never panic the module).
fn cstr(s: &str) -> CString {
    match CString::new(s) {
        Ok(c) => c,
        Err(_) => CString::new(s.replace('\0', "")).unwrap_or_default(),
    }
}

/// Copy a fixed C `char` buffer (NUL-terminated) into an owned `String`.
fn cbuf_to_string(buf: &[c_char]) -> String {
    // SAFETY: `buf` is a valid `&[c_char]`; reinterpreting as bytes is sound.
    let bytes = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// `trap_Printf` — print to the server console.
pub fn Printf(text: &str) {
    let c = cstr(text);
    unsafe {
        syscall!(G_PRINT, c.as_ptr());
    }
}

/// `trap_Error` — abort the game with a message. The engine does not return.
pub fn Error(text: &str) -> ! {
    let c = cstr(text);
    unsafe {
        syscall!(G_ERROR, c.as_ptr());
    }
    unreachable!("engine G_ERROR returned")
}

/// `trap_Milliseconds` — engine millisecond clock (profiling only).
pub fn Milliseconds() -> i32 {
    unsafe { syscall!(G_MILLISECONDS) as i32 }
}

/// `trap_Cvar_Register` — register/attach a cvar mirror. Pass `None` to register
/// a cvar without keeping a module-side mirror.
pub fn Cvar_Register(cvar: Option<&mut vmCvar_t>, var_name: &str, value: &str, flags: i32) {
    let name = cstr(var_name);
    let val = cstr(value);
    let ptr = cvar.map_or(core::ptr::null_mut(), |c| c as *mut vmCvar_t);
    unsafe {
        syscall!(G_CVAR_REGISTER, ptr, name.as_ptr(), val.as_ptr(), flags);
    }
}

/// `trap_Cvar_Update` — refresh a previously registered cvar mirror.
pub fn Cvar_Update(cvar: &mut vmCvar_t) {
    unsafe {
        syscall!(G_CVAR_UPDATE, cvar as *mut vmCvar_t);
    }
}

/// `trap_Cvar_Set` — set a cvar by name.
pub fn Cvar_Set(var_name: &str, value: &str) {
    let name = cstr(var_name);
    let val = cstr(value);
    unsafe {
        syscall!(G_CVAR_SET, name.as_ptr(), val.as_ptr());
    }
}

/// `trap_Cvar_VariableIntegerValue`.
pub fn Cvar_VariableIntegerValue(var_name: &str) -> i32 {
    let name = cstr(var_name);
    unsafe { syscall!(G_CVAR_VARIABLE_INTEGER_VALUE, name.as_ptr()) as i32 }
}

/// `trap_Cvar_VariableStringBuffer` — read a cvar's string value.
pub fn Cvar_VariableString(var_name: &str) -> String {
    let name = cstr(var_name);
    let mut buf = [0 as c_char; STRING_BUF];
    unsafe {
        syscall!(
            G_CVAR_VARIABLE_STRING_BUFFER,
            name.as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as i32
        );
    }
    cbuf_to_string(&buf)
}

/// `trap_Cvar_VariableStringBuffer( const char *var_name, char *buffer, int bufsize )`
/// (g_syscalls.c:67) — read a cvar's string value into a caller-provided buffer. The
/// C-faithful counterpart to [`Cvar_VariableString`] for call sites that keep the raw
/// `char[]` (e.g. to `atof` it). `buffer.len()` supplies `bufsize`.
pub fn Cvar_VariableStringBuffer(var_name: &str, buffer: &mut [c_char]) {
    let name = cstr(var_name);
    unsafe {
        syscall!(
            G_CVAR_VARIABLE_STRING_BUFFER,
            name.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as i32
        );
    }
}

/// `trap_Argc` — number of tokens in the current command.
pub fn Argc() -> i32 {
    unsafe { syscall!(G_ARGC) as i32 }
}

/// `trap_Argv` — token `n` of the current command.
pub fn Argv(n: i32) -> String {
    let mut buf = [0 as c_char; STRING_BUF];
    unsafe {
        syscall!(G_ARGV, n, buf.as_mut_ptr(), buf.len() as i32);
    }
    cbuf_to_string(&buf)
}

/// `trap_FS_FOpenFile` — open `qpath` in `mode`, returning `(length, handle)`.
/// The C out-param `fileHandle_t *f` becomes the second tuple element; the `int`
/// return (the file length, meaningful for `FS_READ`) is the first. A zero
/// handle means the open failed.
pub fn FS_FOpenFile(qpath: &str, mode: fsMode_t) -> (i32, fileHandle_t) {
    let path = cstr(qpath);
    let mut f: fileHandle_t = 0;
    let len = unsafe {
        syscall!(
            G_FS_FOPEN_FILE,
            path.as_ptr(),
            &mut f as *mut fileHandle_t,
            mode
        ) as i32
    };
    (len, f)
}

/// `trap_FS_Read` — read into `buffer` from the open file handle `f`. The C
/// `( void *buffer, int len, ... )` pair collapses to a mutable byte slice, whose
/// length supplies `len` (the symmetric inverse of [`FS_Write`]).
pub fn FS_Read(buffer: &mut [u8], f: fileHandle_t) {
    unsafe {
        syscall!(G_FS_READ, buffer.as_mut_ptr(), buffer.len() as i32, f);
    }
}

/// `trap_FS_Write` — write `buffer` to the open file handle `f`. The C
/// `( const void *buffer, int len, ... )` pair collapses to a byte slice, whose
/// length supplies `len`.
pub fn FS_Write(buffer: &[u8], f: fileHandle_t) {
    unsafe {
        syscall!(G_FS_WRITE, buffer.as_ptr(), buffer.len() as i32, f);
    }
}

/// `trap_FS_FCloseFile` — close a file handle opened with [`FS_FOpenFile`].
pub fn FS_FCloseFile(f: fileHandle_t) {
    unsafe {
        syscall!(G_FS_FCLOSE_FILE, f);
    }
}

/// `trap_FS_GetFileList` — list files under `path` matching `extension`,
/// packing their names into `listbuf` as a run of NUL-terminated strings and
/// returning the count. The C `( ..., char *listbuf, int bufsize )` pair
/// collapses to a mutable `c_char` slice (its length supplies `bufsize`); the
/// caller walks the packed names with `strlen`/pointer arithmetic, so the raw
/// element type is kept rather than `&str`.
pub fn FS_GetFileList(path: &str, extension: &str, listbuf: &mut [c_char]) -> i32 {
    let p = cstr(path);
    let ext = cstr(extension);
    unsafe {
        syscall!(
            G_FS_GETFILELIST,
            p.as_ptr(),
            ext.as_ptr(),
            listbuf.as_mut_ptr(),
            listbuf.len() as i32
        ) as i32
    }
}

/// `trap_SendConsoleCommand` — queue a console command. `exec_when` is an
/// engine `EXEC_*` value (0 = now, 1 = insert, 2 = append).
pub fn SendConsoleCommand(exec_when: i32, text: &str) {
    let c = cstr(text);
    unsafe {
        syscall!(G_SEND_CONSOLE_COMMAND, exec_when, c.as_ptr());
    }
}

/// `trap_LocateGameData` — tell the server where the entity and client arrays
/// live (and their element strides), so it can address them directly without
/// going through an interface. `gents`/`clients` are the array bases;
/// `sizeof_gentity`/`sizeof_gclient` are the strides the engine walks by.
///
/// The `clients` pointer the caller passes is `&gclient[0].ps` (a
/// `playerState_t*` into the first client), not the client base — the engine
/// strides it by `sizeof_gclient` to reach each client's `ps`. This is the
/// pointer-prefix-critical handoff: `sizeof_gentity`/`sizeof_gclient` must equal
/// the module's real `gentity_t`/`gclient_t` sizes (both oracle-pinned at
/// 64-bit), or the engine mis-addresses every entity/client. The caller must
/// keep the arrays alive for as long as the engine holds these pointers.
pub fn LocateGameData(
    gents: *mut gentity_t,
    num_gentities: i32,
    sizeof_gentity: i32,
    clients: *mut playerState_t,
    sizeof_gclient: i32,
) {
    unsafe {
        syscall!(
            G_LOCATE_GAME_DATA,
            gents,
            num_gentities,
            sizeof_gentity,
            clients,
            sizeof_gclient
        );
    }
}

/// `trap_SendServerCommand` — reliably send a command to a client
/// (`client_num == -1` broadcasts to all clients).
pub fn SendServerCommand(client_num: i32, text: &str) {
    let c = cstr(text);
    unsafe {
        syscall!(G_SEND_SERVER_COMMAND, client_num, c.as_ptr());
    }
}

/// `trap_DropClient` — kick a client with a reason message.
pub fn DropClient(client_num: i32, reason: &str) {
    let c = cstr(reason);
    unsafe {
        syscall!(G_DROP_CLIENT, client_num, c.as_ptr());
    }
}

/// `trap_SetConfigstring`.
pub fn SetConfigstring(num: i32, string: &str) {
    let c = cstr(string);
    unsafe {
        syscall!(G_SET_CONFIGSTRING, num, c.as_ptr());
    }
}

/// `trap_GetConfigstring`.
pub fn GetConfigstring(num: i32) -> String {
    let mut buf = [0 as c_char; STRING_BUF];
    unsafe {
        syscall!(G_GET_CONFIGSTRING, num, buf.as_mut_ptr(), buf.len() as i32);
    }
    cbuf_to_string(&buf)
}

/// `trap_GetUserinfo` — read a client's userinfo string.
pub fn GetUserinfo(num: i32) -> String {
    let mut buf = [0 as c_char; STRING_BUF];
    unsafe {
        syscall!(G_GET_USERINFO, num, buf.as_mut_ptr(), buf.len() as i32);
    }
    cbuf_to_string(&buf)
}

/// `trap_SetUserinfo` — overwrite a client's userinfo string.
pub fn SetUserinfo(num: i32, info: &str) {
    let c = cstr(info);
    unsafe {
        syscall!(G_SET_USERINFO, num, c.as_ptr());
    }
}

/// `trap_GetUsercmd` — copy the engine's stored usercmd for `clientNum` into `cmd`
/// (the C `( int clientNum, usercmd_t *cmd )`; out-param kept verbatim as a raw
/// pointer because the caller owns the storage — `pers.cmd` inside the gclient).
pub fn GetUsercmd(client_num: i32, cmd: *mut usercmd_t) {
    unsafe {
        syscall!(G_GET_USERCMD, client_num, cmd);
    }
}

/// `trap_GetServerinfo` — read the serverinfo string (all the cvars visible to
/// server browsers).
pub fn GetServerinfo() -> String {
    let mut buf = [0 as c_char; STRING_BUF];
    unsafe {
        syscall!(G_GET_SERVERINFO, buf.as_mut_ptr(), buf.len() as i32);
    }
    cbuf_to_string(&buf)
}

/// `trap_GetEntityToken` — pull the next token from the BSP entity string the
/// engine cached at load, writing it into `buffer` and returning `qtrue` while
/// tokens remain (`qfalse` at end of string). `G_ParseSpawnVars` drives this in a
/// loop to walk the brace-delimited key/value blocks.
///
/// Kept 1:1 with the C `( char *buffer, int bufferSize )`: the caller owns the
/// scratch buffer and its length supplies `bufferSize`, so the qboolean return is
/// preserved verbatim (unlike the string-returning traps above, which hide their
/// buffer).
pub fn GetEntityToken(buffer: &mut [c_char]) -> qboolean {
    unsafe { syscall!(G_GET_ENTITY_TOKEN, buffer.as_mut_ptr(), buffer.len() as i32) as qboolean }
}

// ===========================================================================
// World interaction — entity linking, collision tracing, PVS, area portals.
// These mirror the "server specific functionality" block of g_syscalls.c. The
// `qboolean`-returning traps keep that C return type verbatim (faithful 1:1 with
// the engine API); structure out-params (`trace_t`) become return values, as
// with the string/tuple-returning traps above.
// ===========================================================================

/// `trap_LinkEntity` — make `ent` visible to collision and to clients. An entity
/// is never sent to a client or used for collision until it is linked; if its
/// size, position, or solidity changes it must be relinked.
pub fn LinkEntity(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_LINKENTITY, ent);
    }
}

/// `trap_UnlinkEntity` — call before removing an interactive entity.
pub fn UnlinkEntity(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_UNLINKENTITY, ent);
    }
}

/// `trap_SetBrushModel` — set `ent`'s `mins`/`maxs` (and bmodel) from the named
/// inline brush model (e.g. `"*3"`).
pub fn SetBrushModel(ent: *mut gentity_t, name: &str) {
    let c = cstr(name);
    unsafe {
        syscall!(G_SET_BRUSH_MODEL, ent, c.as_ptr());
    }
}

/// `trap_Trace` — sweep the box (`mins`..`maxs`) from `start` to `end` against all
/// linked entities, returning the resulting [`trace_t`] (the C `trace_t *results`
/// out-param). `pass_entity_num` is skipped; `contentmask` selects which surfaces
/// block. The C wrapper hard-codes the two trailing ghoul2 args (`g2TraceType` 0,
/// `traceLod` 10) — see [`G2Trace`] to set them.
pub fn Trace(
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    pass_entity_num: i32,
    contentmask: i32,
) -> trace_t {
    let mut results = trace_t::default();
    unsafe {
        syscall!(
            G_TRACE,
            &mut results as *mut trace_t,
            start.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            end.as_ptr(),
            pass_entity_num,
            contentmask,
            0,
            10
        );
    }
    results
}

/// `trap_G2Trace` — like [`Trace`] but with explicit ghoul2 collision control.
/// `g2_trace_type`: 0 = no g2 collision, 1 = collide vs anything not `EF_DEAD`,
/// 2 = collide vs all. `trace_lod` is the ghoul2 level of detail.
pub fn G2Trace(
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    pass_entity_num: i32,
    contentmask: i32,
    g2_trace_type: i32,
    trace_lod: i32,
) -> trace_t {
    let mut results = trace_t::default();
    unsafe {
        syscall!(
            G_G2TRACE,
            &mut results as *mut trace_t,
            start.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            end.as_ptr(),
            pass_entity_num,
            contentmask,
            g2_trace_type,
            trace_lod
        );
    }
    results
}

/// `trap_TraceCapsule` — as [`Trace`] but sweeps a capsule rather than a box.
/// Like `Trace`, the C wrapper hard-codes the trailing ghoul2 args (0, 10).
pub fn TraceCapsule(
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    pass_entity_num: i32,
    contentmask: i32,
) -> trace_t {
    let mut results = trace_t::default();
    unsafe {
        syscall!(
            G_TRACECAPSULE,
            &mut results as *mut trace_t,
            start.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            end.as_ptr(),
            pass_entity_num,
            contentmask,
            0,
            10
        );
    }
    results
}

/// `trap_PointContents` — content flags at `point`, tested against all linked
/// entities. `pass_entity_num` is excluded from the test.
pub fn PointContents(point: &vec3_t, pass_entity_num: i32) -> i32 {
    unsafe { syscall!(G_POINT_CONTENTS, point.as_ptr(), pass_entity_num) as i32 }
}

/// `trap_InPVS` — whether `p2` is in `p1`'s PVS (potentially visible set). Closed
/// area portals (e.g. doors) break visibility.
pub fn InPVS(p1: &vec3_t, p2: &vec3_t) -> qboolean {
    unsafe { syscall!(G_IN_PVS, p1.as_ptr(), p2.as_ptr()) as qboolean }
}

/// `trap_InPVSIgnorePortals` — as [`InPVS`], but area portals never break the
/// visibility test.
pub fn InPVSIgnorePortals(p1: &vec3_t, p2: &vec3_t) -> qboolean {
    unsafe { syscall!(G_IN_PVS_IGNORE_PORTALS, p1.as_ptr(), p2.as_ptr()) as qboolean }
}

/// `trap_AdjustAreaPortalState` — open or close the area portal that `ent` (a
/// door) straddles, updating PVS and area connectivity. `open` is a `qboolean`.
pub fn AdjustAreaPortalState(ent: *mut gentity_t, open: qboolean) {
    unsafe {
        syscall!(G_ADJUST_AREA_PORTAL_STATE, ent, open);
    }
}

/// `trap_AreasConnected` — whether `area1` and `area2` are connected (i.e. not
/// separated by a closed area portal).
pub fn AreasConnected(area1: i32, area2: i32) -> qboolean {
    unsafe { syscall!(G_AREAS_CONNECTED, area1, area2) as qboolean }
}

/// `trap_EntitiesInBox` — fill `list` with the entity numbers whose bounding
/// boxes overlap the box `mins`..`maxs`, returning the count found. The C
/// `( int *list, int maxcount )` pair collapses to a `&mut [i32]` whose length
/// supplies `maxcount`. This is a broad-phase bbox test, so exact overlap must
/// still be confirmed with [`EntityContact`].
pub fn EntitiesInBox(mins: &vec3_t, maxs: &vec3_t, list: &mut [i32]) -> i32 {
    unsafe {
        syscall!(
            G_ENTITIES_IN_BOX,
            mins.as_ptr(),
            maxs.as_ptr(),
            list.as_mut_ptr(),
            list.len() as i32
        ) as i32
    }
}

/// `trap_EntityContact` — exact overlap test of the box `mins`..`maxs` against
/// `ent`'s (possibly non-axial) inline brush model.
pub fn EntityContact(mins: &vec3_t, maxs: &vec3_t, ent: *const gentity_t) -> qboolean {
    unsafe { syscall!(G_ENTITY_CONTACT, mins.as_ptr(), maxs.as_ptr(), ent) as qboolean }
}

/// `trap_EntityContactCapsule` — as [`EntityContact`], but the moving volume is a
/// capsule rather than a box.
pub fn EntityContactCapsule(mins: &vec3_t, maxs: &vec3_t, ent: *const gentity_t) -> qboolean {
    unsafe { syscall!(G_ENTITY_CONTACTCAPSULE, mins.as_ptr(), maxs.as_ptr(), ent) as qboolean }
}

// ===========================================================================
// Engine-subsystem registration / Ghoul2.
// ===========================================================================

/// `trap_SV_RegisterSharedMemory` — hand the engine the address of the module's
/// shared-memory buffer (`gSharedBuffer`), which the engine fills for certain
/// callbacks (e.g. the ICARUS bridge) for the module to read back. The caller
/// must keep the buffer alive for as long as the engine holds the pointer.
pub fn SV_RegisterSharedMemory(memory: *mut c_char) {
    unsafe {
        syscall!(G_SET_SHARED_BUFFER, memory);
    }
}

/// `trap_G2API_CleanEntAttachments` — clear any engine-side ghoul2 instance↔entity
/// attachments (used at game init).
pub fn G2API_CleanEntAttachments() {
    unsafe {
        syscall!(G_G2_CLEANENTATTACHMENTS);
    }
}

/// `trap_G2API_GetBoltMatrix` — ask the engine for the world-space transform of a
/// ghoul2 bolt/bone. The engine reconstructs the model's skeleton at `frame_num` with
/// the model placed at `position`/`angles`/`scale`, then writes the bolt `bolt_index`'s
/// 3×4 matrix into `matrix` (bolt origin is its column 3). `model_list` may be null.
/// Returns `qtrue` on success. (A third `_NoReconstruct` variant exists but is unused by
/// the game module; the `_NoRecNoRot` variant below is used by `BG_G2ClientSpineAngles`.)
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix(
    ghoul2: *mut c_void,
    model_index: i32,
    bolt_index: i32,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frame_num: i32,
    model_list: *mut qhandle_t,
    scale: &vec3_t,
) -> qboolean {
    unsafe {
        syscall!(
            G_G2_GETBOLT,
            ghoul2,
            model_index,
            bolt_index,
            matrix as *mut mdxaBone_t,
            angles.as_ptr(),
            position.as_ptr(),
            frame_num,
            model_list,
            scale.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_GetBoltMatrix_NoRecNoRot` — like `G2API_GetBoltMatrix` but forces the
/// engine to skip reconstructing (and rotating) the skeleton before reading the bolt,
/// using whatever pose is already cached. `model_list` may be null. Returns `qtrue` on
/// success.
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix_NoRecNoRot(
    ghoul2: *mut c_void,
    model_index: i32,
    bolt_index: i32,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frame_num: i32,
    model_list: *mut qhandle_t,
    scale: &vec3_t,
) -> qboolean {
    unsafe {
        syscall!(
            G_G2_GETBOLT_NOREC_NOROT,
            ghoul2,
            model_index,
            bolt_index,
            matrix as *mut mdxaBone_t,
            angles.as_ptr(),
            position.as_ptr(),
            frame_num,
            model_list,
            scale.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_SetBoneAngles` — override the bone `bone_name`'s angles on the ghoul2
/// instance, telling the engine how to compose them (`flags`) and which axis order the
/// three orientation slots (`up`/`right`/`forward`) map to. `model_list` may be null.
/// Returns `qtrue` on success.
#[allow(clippy::too_many_arguments)]
pub fn G2API_SetBoneAngles(
    ghoul2: *mut c_void,
    model_index: i32,
    bone_name: &str,
    angles: &vec3_t,
    flags: i32,
    up: i32,
    right: i32,
    forward: i32,
    model_list: *mut qhandle_t,
    blend_time: i32,
    current_time: i32,
) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_ANGLEOVERRIDE,
            ghoul2,
            model_index,
            bone.as_ptr(),
            angles.as_ptr(),
            flags,
            up,
            right,
            forward,
            model_list,
            blend_time,
            current_time
        ) as qboolean
    }
}

/// `trap_G2API_SetBoneAnim` — play the frame range `start_frame..end_frame` on bone
/// `bone_name` of the ghoul2 instance, at `anim_speed`, blending over `blend_time`.
/// `set_frame` seeds the current frame (`-1` to leave it). Returns `qtrue` on success.
/// The two float args ride the integer syscall ABI via `pass_float` (C `PASSFLOAT`).
#[allow(clippy::too_many_arguments)]
pub fn G2API_SetBoneAnim(
    ghoul2: *mut c_void,
    model_index: i32,
    bone_name: &str,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
    current_time: i32,
    set_frame: f32,
    blend_time: i32,
) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_PLAYANIM,
            ghoul2,
            model_index,
            bone.as_ptr(),
            start_frame,
            end_frame,
            flags,
            pass_float(anim_speed),
            current_time,
            pass_float(set_frame),
            blend_time
        ) as qboolean
    }
}

/// `trap_G2API_GetBoneAnim` — read back the animation state of bone `bone_name` at
/// `current_time`: the current/`start`/`end` frames, `flags`, and `anim_speed`.
/// `model_list` may be null. Returns `qtrue` on success.
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoneAnim(
    ghoul2: *mut c_void,
    bone_name: &str,
    current_time: i32,
    current_frame: &mut f32,
    start_frame: &mut i32,
    end_frame: &mut i32,
    flags: &mut i32,
    anim_speed: &mut f32,
    model_list: *mut i32,
    model_index: i32,
) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_GETBONEANIM,
            ghoul2,
            bone.as_ptr(),
            current_time,
            current_frame as *mut f32,
            start_frame as *mut i32,
            end_frame as *mut i32,
            flags as *mut i32,
            anim_speed as *mut f32,
            model_list,
            model_index
        ) as qboolean
    }
}

/// `trap_G2API_AnimateG2Models` — step the ghoul2 instance's animation (and ragdoll)
/// for time `time`, using `params` (the model's world placement, owner, settle frame).
pub fn G2API_AnimateG2Models(ghoul2: *mut c_void, time: i32, params: &sharedRagDollUpdateParams_t) {
    unsafe {
        syscall!(
            G_G2_ANIMATEG2MODELS,
            ghoul2,
            time,
            params as *const sharedRagDollUpdateParams_t
        );
    }
}

/// `trap_G2API_SetBoneIKState` — turn IK on/off for bone `bone_name` of the ghoul2
/// instance. `bone_name` may be `None` (a null name initialises or tears down the
/// instance-wide IK/ragdoll effector state); `params` may be `None` (required when
/// enabling, ignored when disabling). Returns `qtrue` on success.
pub fn G2API_SetBoneIKState(
    ghoul2: *mut c_void,
    time: i32,
    bone_name: Option<&str>,
    ik_state: i32,
    params: Option<&sharedSetBoneIKStateParams_t>,
) -> qboolean {
    let bone = bone_name.map(cstr);
    let bone_ptr = bone.as_ref().map_or(core::ptr::null(), |c| c.as_ptr());
    let params_ptr = params.map_or(core::ptr::null(), |p| {
        p as *const sharedSetBoneIKStateParams_t
    });
    unsafe {
        syscall!(
            G_G2_SETBONEIKSTATE,
            ghoul2,
            time,
            bone_ptr,
            ik_state,
            params_ptr
        ) as qboolean
    }
}

/// `trap_G2API_IKMove` — advance an active IK bone one step toward its
/// `desiredOrigin` (carried in `params`) at time `time`. Returns `qtrue` on success.
pub fn G2API_IKMove(ghoul2: *mut c_void, time: i32, params: &sharedIKMoveParams_t) -> qboolean {
    unsafe {
        syscall!(
            G_G2_IKMOVE,
            ghoul2,
            time,
            params as *const sharedIKMoveParams_t
        ) as qboolean
    }
}

/// `trap_R_RegisterSkin` — register the `.skin` file `name` with the engine renderer and
/// return its handle (precaching it). Used by [`crate::codemp::game::bg_misc::BG_ModelCache`]
/// to warm a model's skin before instancing its ghoul2 model.
pub fn R_RegisterSkin(name: &str) -> qhandle_t {
    let n = cstr(name);
    unsafe { syscall!(G_R_REGISTERSKIN, n.as_ptr()) as qhandle_t }
}

// ===========================================================================
// ICARUS scripting — the game→engine half of the ICARUS bridge.
//
// The ICARUS *interpreter* (tokenizer/sequencer/taskmanager — the
// `refs/raven-jediacademy/codemp/icarus/` C++ library) lives **engine-side**; the
// module reaches it through these syscalls and the engine calls *back* into the
// module via the `GAME_ICARUS_*` vmMain commands (the `g_ICARUScb.c` bridge).
// SP maps run in MP and ship scripts, so this surface is required, not stubbed.
//
// Unlike the `&str` traps above (called from Rust), these are driven by the
// C-ported bridge / `G_ActivateBehavior`, which already hold raw C pointers, so
// the wrappers keep the verbatim `g_syscalls.c` pointer signatures.
// ===========================================================================

/// `trap_ICARUS_RunScript` — queue script `name` to run on `ent` (engine-side
/// interpreter). Returns the engine's int result.
pub fn ICARUS_RunScript(ent: *mut gentity_t, name: *const c_char) -> c_int {
    unsafe { syscall!(G_ICARUS_RUNSCRIPT, ent, name) as c_int }
}

/// `trap_ICARUS_RegisterScript` — precache/validate script `name`. `b_called_during_interrogate`
/// distinguishes the interrogation precache pass.
pub fn ICARUS_RegisterScript(
    name: *const c_char,
    b_called_during_interrogate: qboolean,
) -> qboolean {
    unsafe { syscall!(G_ICARUS_REGISTERSCRIPT, name, b_called_during_interrogate) as qboolean }
}

/// `trap_ICARUS_Init` — initialise the engine-side ICARUS instance (at `GAME_INIT`).
pub fn ICARUS_Init() {
    unsafe {
        syscall!(G_ICARUS_INIT);
    }
}

/// `trap_ICARUS_ValidEnt` — does `ent` have a live ICARUS instance?
pub fn ICARUS_ValidEnt(ent: *mut gentity_t) -> qboolean {
    unsafe { syscall!(G_ICARUS_VALIDENT, ent) as qboolean }
}

/// `trap_ICARUS_IsInitialized` — is entity `ent_id`'s ICARUS instance initialised?
pub fn ICARUS_IsInitialized(ent_id: c_int) -> qboolean {
    unsafe { syscall!(G_ICARUS_ISINITIALIZED, ent_id) as qboolean }
}

/// `trap_ICARUS_MaintainTaskManager` — pump entity `ent_id`'s task manager one step.
pub fn ICARUS_MaintainTaskManager(ent_id: c_int) -> qboolean {
    unsafe { syscall!(G_ICARUS_MAINTAINTASKMANAGER, ent_id) as qboolean }
}

/// `trap_ICARUS_IsRunning` — is a script currently running on entity `ent_id`?
pub fn ICARUS_IsRunning(ent_id: c_int) -> qboolean {
    unsafe { syscall!(G_ICARUS_ISRUNNING, ent_id) as qboolean }
}

/// `trap_ICARUS_TaskIDPending` — is task `task_id` still pending on `ent`?
pub fn ICARUS_TaskIDPending(ent: *mut gentity_t, task_id: c_int) -> qboolean {
    unsafe { syscall!(G_ICARUS_TASKIDPENDING, ent, task_id) as qboolean }
}

/// `trap_ICARUS_InitEnt` — create an ICARUS instance for `ent`.
pub fn ICARUS_InitEnt(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_ICARUS_INITENT, ent);
    }
}

/// `trap_ICARUS_FreeEnt` — tear down `ent`'s ICARUS instance (on entity free).
pub fn ICARUS_FreeEnt(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_ICARUS_FREEENT, ent);
    }
}

/// `trap_ICARUS_AssociateEnt` — bind `ent` to its ICARUS instance after spawn.
pub fn ICARUS_AssociateEnt(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_ICARUS_ASSOCIATEENT, ent);
    }
}

/// `trap_ICARUS_Shutdown` — shut down the engine-side ICARUS instance.
pub fn ICARUS_Shutdown() {
    unsafe {
        syscall!(G_ICARUS_SHUTDOWN);
    }
}

/// `trap_ICARUS_TaskIDSet` — record that `ent` is waiting on a `task_type` task `task_id`.
pub fn ICARUS_TaskIDSet(ent: *mut gentity_t, task_type: c_int, task_id: c_int) {
    unsafe {
        syscall!(G_ICARUS_TASKIDSET, ent, task_type, task_id);
    }
}

/// `trap_ICARUS_TaskIDComplete` — signal the engine that `ent`'s `task_type` task finished.
pub fn ICARUS_TaskIDComplete(ent: *mut gentity_t, task_type: c_int) {
    unsafe {
        syscall!(G_ICARUS_TASKIDCOMPLETE, ent, task_type);
    }
}

/// `trap_ICARUS_SetVar` — set ICARUS variable `type_name` to `data` (for task `task_id`,
/// entity `ent_id`).
pub fn ICARUS_SetVar(task_id: c_int, ent_id: c_int, type_name: *const c_char, data: *const c_char) {
    unsafe {
        syscall!(G_ICARUS_SETVAR, task_id, ent_id, type_name, data);
    }
}

/// `trap_ICARUS_VariableDeclared` — has variable `type_name` been declared? (returns its type)
pub fn ICARUS_VariableDeclared(type_name: *const c_char) -> c_int {
    unsafe { syscall!(G_ICARUS_VARIABLEDECLARED, type_name) as c_int }
}

/// `trap_ICARUS_GetFloatVariable` — read float variable `name` into `*value`.
pub fn ICARUS_GetFloatVariable(name: *const c_char, value: *mut f32) -> c_int {
    unsafe { syscall!(G_ICARUS_GETFLOATVARIABLE, name, value) as c_int }
}

/// `trap_ICARUS_GetStringVariable` — read string variable `name` into the caller's `value`
/// buffer. The C signature types the out-buffer `const char *` (a faithful quirk — the engine
/// writes through it); kept verbatim.
pub fn ICARUS_GetStringVariable(name: *const c_char, value: *const c_char) -> c_int {
    unsafe { syscall!(G_ICARUS_GETSTRINGVARIABLE, name, value) as c_int }
}

/// `trap_ICARUS_GetVectorVariable` — read vector variable `name` into `value`.
pub fn ICARUS_GetVectorVariable(name: *const c_char, value: *mut vec3_t) -> c_int {
    unsafe { syscall!(G_ICARUS_GETVECTORVARIABLE, name, value) as c_int }
}

/// `trap_G2API_InitGhoul2Model` — create a ghoul2 model instance for the `.glm` file
/// `file_name`, writing the new instance pointer through `ghoul2_ptr`. `model_index` is the
/// slot within that instance; `custom_skin`/`custom_shader` override the skin/shader (0 for
/// none); `model_flags`/`lod_bias` tune loading. Returns the model handle (the index it was
/// placed at). `ghoul2_ptr` must point to a valid `*mut c_void` slot (NULL to allocate).
///
/// `file_name` is a raw `*const c_char` to mirror the C wrapper's `const char *fileName` —
/// `WP_SaberAddG2Model` forwards a possibly-NULL `saberModel` straight through, which a `&str`
/// could not represent, and other call sites already hold a C string array they can pass
/// directly without a lossy `String` round-trip.
///
/// # Safety
/// `file_name` must be NULL or a valid NUL-terminated C string.
#[allow(clippy::too_many_arguments)]
pub unsafe fn G2API_InitGhoul2Model(
    ghoul2_ptr: *mut *mut c_void,
    file_name: *const c_char,
    model_index: i32,
    custom_skin: qhandle_t,
    custom_shader: qhandle_t,
    model_flags: i32,
    lod_bias: i32,
) -> i32 {
    unsafe {
        syscall!(
            G_G2_INITGHOUL2MODEL,
            ghoul2_ptr,
            file_name,
            model_index,
            custom_skin,
            custom_shader,
            model_flags,
            lod_bias
        ) as i32
    }
}

/// `trap_G2API_RemoveGhoul2Models` — free every model in the ghoul2 instance referenced by
/// `ghl_info` and release the instance. The C wrapper is declared `qboolean
/// trap_G2API_RemoveGhoul2Models(void *ghlInfo)` (a single `*`) even though its callers pass
/// `&ent->ghoul2` (a `void **`); the syscall just forwards the pointer value unchanged, so we
/// match the wrapper signature and the call sites cast `&ent.ghoul2` to `*mut c_void`. Used by
/// [`crate::codemp::game::w_saber::WP_SaberRemoveG2Model`] to tear down a saber render model.
pub fn G2API_RemoveGhoul2Models(ghl_info: *mut c_void) -> qboolean {
    unsafe { syscall!(G_G2_REMOVEGHOUL2MODELS, ghl_info) as qboolean }
}

/// `trap_G2API_CleanGhoul2Models` — free the ghoul2 instance pointed to by `ghoul2_ptr`
/// (releasing all its models) and NULL the slot. Safe to call on an allocated instance only.
pub fn G2API_CleanGhoul2Models(ghoul2_ptr: *mut *mut c_void) {
    unsafe {
        syscall!(G_G2_CLEANMODELS, ghoul2_ptr);
    }
}

/// `trap_G2_HaveWeGhoul2Models` — does `ghoul2` reference a live, non-empty ghoul2 instance?
/// Used by [`crate::codemp::game::g_utils::G_FreeEntity`] before tearing down a client's
/// per-saber `weaponGhoul2` models.
pub fn G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> qboolean {
    unsafe { syscall!(G_G2_HAVEWEGHOULMODELS, ghoul2) as qboolean }
}

/// `trap_G2API_OverrideServer` — make the server-side ghoul2 instance `server_instance` the
/// authoritative one (so subsequent G2 queries resolve against it). Returns `qtrue` on success.
pub fn G2API_OverrideServer(server_instance: *mut c_void) -> qboolean {
    unsafe { syscall!(G_G2_OVERRIDESERVER, server_instance) as qboolean }
}

/// `trap_G2API_AddBolt` — register a bolt (an attach point) on model `model_index` of the
/// ghoul2 instance, named by `bone_name` (a bone like `"jaw_bone"` or a tag surface like
/// `"*r_hand"`). Returns the bolt index (passed back to [`G2API_GetBoltMatrix`]), or `-1` on
/// failure. Used by [`crate::codemp::game::bg_g2_utils::BG_AttachToRancor`].
pub fn G2API_AddBolt(ghoul2: *mut c_void, model_index: i32, bone_name: &str) -> i32 {
    let n = cstr(bone_name);
    unsafe { syscall!(G_G2_ADDBOLT, ghoul2, model_index, n.as_ptr()) as i32 }
}

/// `trap_G2API_SetSkin` — override the skin on model `model_index` of the ghoul2 instance:
/// `custom_skin` is the registered `.skin` handle used for surface on/off (0 for none) and
/// `render_skin` is the handle the renderer draws with. Returns `qtrue` on success. Used by
/// [`crate::codemp::game::g_client::G_SaberModelSetup`] to apply a saber's custom skin.
pub fn G2API_SetSkin(
    ghoul2: *mut c_void,
    model_index: i32,
    custom_skin: qhandle_t,
    render_skin: qhandle_t,
) -> qboolean {
    unsafe { syscall!(G_G2_SETSKIN, ghoul2, model_index, custom_skin, render_skin) as qboolean }
}

/// `trap_G2API_SetBoltInfo` — set the bolt-info field on model `model_index` of the ghoul2
/// instance to `bolt_info` (which bolt this model is attached to on its parent). No return.
pub fn G2API_SetBoltInfo(ghoul2: *mut c_void, model_index: i32, bolt_info: i32) {
    unsafe {
        syscall!(G_G2_SETBOLTINFO, ghoul2, model_index, bolt_info);
    }
}

/// `trap_G2API_RemoveBone` — remove the bone named `bone_name` from the bone list of model
/// `model_index` on the ghoul2 instance. Returns `qtrue` on success. Used by
/// [`crate::codemp::game::g_client::G_UpdateClientAnims`] to drop a stale custom bone.
pub fn G2API_RemoveBone(ghoul2: *mut c_void, bone_name: &str, model_index: i32) -> qboolean {
    let n = cstr(bone_name);
    unsafe { syscall!(G_G2_REMOVEBONE, ghoul2, n.as_ptr(), model_index) as qboolean }
}

/// `trap_G2API_CopySpecificGhoul2Model` — copy a single model (`model_from`) from the ghoul2
/// instance `g2_from` into slot `model_to` of instance `g2_to`. No return. Used by
/// [`crate::codemp::game::g_client::G_SaberModelSetup`] to clone saber models onto a client.
pub fn G2API_CopySpecificGhoul2Model(
    g2_from: *mut c_void,
    model_from: i32,
    g2_to: *mut c_void,
    model_to: i32,
) {
    unsafe {
        syscall!(
            G_G2_COPYSPECIFICGHOUL2MODEL,
            g2_from,
            model_from,
            g2_to,
            model_to
        );
    }
}

/// `trap_G2API_GetSurfaceRenderStatus` — query whether the surface `surface_name` on model
/// `model_index` of the ghoul2 instance is currently being rendered (the C returns an `int`:
/// nonzero = off/not-rendered flags, `0` = on). `surface_name` is passed through as a raw
/// engine string (callers hold a C buffer). Used by
/// [`crate::codemp::game::bg_g2_utils::BG_GetRootSurfNameWithVariant`].
pub fn G2API_GetSurfaceRenderStatus(
    ghoul2: *mut c_void,
    model_index: i32,
    surface_name: *const c_char,
) -> i32 {
    unsafe {
        syscall!(
            G_G2_GETSURFACERENDERSTATUS,
            ghoul2,
            model_index,
            surface_name
        ) as i32
    }
}

/// `trap_G2API_GetSurfaceName` — write the name of surface `surf_number` on model
/// `model_index` of the ghoul2 instance into the caller-owned `fill_buf` (a `char`
/// buffer, `MAX_QPATH`). Used by
/// [`crate::codemp::game::g_combat::G_LocationBasedDamageModifier`] to resolve the
/// last ghoul2-collision surface into a hit location.
pub fn G2API_GetSurfaceName(
    ghoul2: *mut c_void,
    surf_number: i32,
    model_index: i32,
    fill_buf: *mut c_char,
) {
    unsafe {
        syscall!(
            G_G2_GETSURFACENAME,
            ghoul2,
            surf_number,
            model_index,
            fill_buf
        );
    }
}

/// `trap_G2API_SetSurfaceOnOff` — turn the surface `surface_name` on model 0 of the
/// ghoul2 instance on or off by OR-ing `flags` into its render/descendant bits (e.g.
/// `0x00000100` = `G2SURFACEFLAG_NODESCENDANTS`, `0` = back on). Returns a `qboolean`
/// (success). `surface_name` is a raw engine string (callers hold a C buffer). Used by
/// [`crate::codemp::game::g_combat::G_Dismember`] to drop a severed limb's surfs on NPCs.
pub fn G2API_SetSurfaceOnOff(
    ghoul2: *mut c_void,
    surface_name: *const c_char,
    flags: i32,
) -> qboolean {
    unsafe { syscall!(G_G2_SETSURFACEONOFF, ghoul2, surface_name, flags) as qboolean }
}

/// `trap_G2API_CollisionDetect` — run a per-poly ghoul2 trace against the model
/// instance `ghoul2`, posed at `position`/`angles`/`scale` for frame `frame_number`,
/// along the ray `ray_start`→`ray_end`. The engine writes each part hit into
/// `coll_rec_map` (the caller's [`CollisionRecord_t`] array, i.e. the first element of a
/// `G2Trace_t`). `ent_num` is the model's entity number, `trace_flags`/`use_lod` tune the
/// trace, and `f_radius` is the trace's thickness. Used by
/// [`crate::codemp::game::w_saber::G_G2TraceCollide`] for saber-blade hit precision.
/// The trailing float arg rides the integer syscall ABI via `pass_float` (C `PASSFLOAT`).
#[allow(clippy::too_many_arguments)]
pub fn G2API_CollisionDetect(
    coll_rec_map: *mut CollisionRecord_t,
    ghoul2: *mut c_void,
    angles: &vec3_t,
    position: &vec3_t,
    frame_number: i32,
    ent_num: i32,
    ray_start: &vec3_t,
    ray_end: &vec3_t,
    scale: &vec3_t,
    trace_flags: i32,
    use_lod: i32,
    f_radius: f32,
) {
    unsafe {
        syscall!(
            G_G2_COLLISIONDETECT,
            coll_rec_map,
            ghoul2,
            angles.as_ptr(),
            position.as_ptr(),
            frame_number,
            ent_num,
            ray_start.as_ptr(),
            ray_end.as_ptr(),
            scale.as_ptr(),
            trace_flags,
            use_lod,
            pass_float(f_radius)
        );
    }
}

/// `trap_G2API_CollisionDetectCache` (g_syscalls.c:1326) — identical to
/// [`G2API_CollisionDetect`] but routes through `G_G2_COLLISIONDETECTCACHE`, the engine's
/// cached-result variant used for vehicle ghoul2 collision. Used by
/// [`crate::codemp::game::w_saber::G_G2TraceCollide`]'s `g_optvehtrace` vehicle path.
/// The trailing float arg rides the integer syscall ABI via `pass_float` (C `PASSFLOAT`).
#[allow(clippy::too_many_arguments)]
pub fn G2API_CollisionDetectCache(
    coll_rec_map: *mut CollisionRecord_t,
    ghoul2: *mut c_void,
    angles: &vec3_t,
    position: &vec3_t,
    frame_number: i32,
    ent_num: i32,
    ray_start: &vec3_t,
    ray_end: &vec3_t,
    scale: &vec3_t,
    trace_flags: i32,
    use_lod: i32,
    f_radius: f32,
) {
    unsafe {
        syscall!(
            G_G2_COLLISIONDETECTCACHE,
            coll_rec_map,
            ghoul2,
            angles.as_ptr(),
            position.as_ptr(),
            frame_number,
            ent_num,
            ray_start.as_ptr(),
            ray_end.as_ptr(),
            scale.as_ptr(),
            trace_flags,
            use_lod,
            pass_float(f_radius)
        );
    }
}

/// `trap_SnapVector` — round each component of `v` to the integer grid the engine
/// uses for the network snapshot, so client and server agree bit-for-bit. The C
/// wrapper takes a bare `float *v` (the 3-float velocity array).
pub fn SnapVector(v: &mut vec3_t) {
    unsafe {
        syscall!(G_SNAPVECTOR, v.as_mut_ptr());
    }
}
