//! Fundamental shared types and math constants from `q_shared.h`.
//!
//! `vec_t`/`vec3_t` are plain float arrays exactly as in C, so geometry code ports
//! mechanically and any FFI-visible struct embedding them keeps identical layout.

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

/// `vec_t` — the scalar type for all geometry (C `float`).
pub type vec_t = f32;
pub type vec2_t = [vec_t; 2];
pub type vec3_t = [vec_t; 3];
pub type vec4_t = [vec_t; 4];
pub type vec5_t = [vec_t; 5];
pub type vec3pair_t = [vec3_t; 2];

pub type ivec3_t = [i32; 3];
pub type ivec4_t = [i32; 4];
pub type ivec5_t = [i32; 5];

/// `byte` — unsigned char.
pub type byte = u8;

/// `qint64` (q_shared.h) — a 64-bit integer for the global-rankings interface,
/// implemented as 8 explicit bytes "for qvm compatibility". Only ever consumed by
/// `Long64Swap`/`Long64NoSwap`; pointer-free, so identical layout on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct qint64 {
    pub b0: byte,
    pub b1: byte,
    pub b2: byte,
    pub b3: byte,
    pub b4: byte,
    pub b5: byte,
    pub b6: byte,
    pub b7: byte,
}

const _: () = assert!(core::mem::size_of::<qint64>() == 8);
const _: () = assert!(core::mem::align_of::<qint64>() == 1);

/// `stringID_table_t` (q_shared.h) — a `{ name, id }` row for the `GetIDForString`
/// / `GetStringForID` lookup tables (each table is terminated by a `{ NULL, 0 }`
/// or `{ "", ... }` entry). Holds a raw `char *`, so its layout is arch-dependent
/// (no fixed-size assert).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct stringID_table_t {
    pub name: *const c_char,
    pub id: c_int,
}

/// `NUMVERTEXNORMALS` (q_shared.h) — size of the `bytedirs` quantized-normal table.
pub const NUMVERTEXNORMALS: usize = 162;

/// String-size limits from q_shared.h.
pub const MAX_STRING_CHARS: usize = 1024; // max length of a string passed to Cmd_TokenizeString
pub const MAX_TOKEN_CHARS: usize = 1024; // max length of an individual token
pub const MAX_INFO_STRING: usize = 1024;
pub const MAX_INFO_KEY: usize = 1024;
pub const MAX_INFO_VALUE: usize = 1024;
pub const BIG_INFO_STRING: usize = 8192; // used for system info key only
pub const BIG_INFO_KEY: usize = 8192;
pub const BIG_INFO_VALUE: usize = 8192;
pub const MAX_QPATH: usize = 64; // max length of a quake game pathname
pub const MAX_NAME_LENGTH: usize = 32; // max length of a client name (q_shared.h:404)

/// World-coordinate bounds (q_shared.h:18-20). Faithful to the C macro forms; the
/// `(float)WORLD_SIZE*(float)WORLD_SIZE` distance sentinel in `NPC_FindSquadPoint`
/// is the first consumer.
pub const MAX_WORLD_COORD: c_int = 64 * 1024;
pub const MIN_WORLD_COORD: c_int = -64 * 1024;
pub const WORLD_SIZE: c_int = MAX_WORLD_COORD - MIN_WORLD_COORD;

/// `#define Q3_SCRIPT_DIR "scripts"` (q_shared.h:10) — the script-directory prefix
/// joined onto a behaviorSet name to form an ICARUS script path. Consumed by
/// `G_ActivateBehavior` (npc_utils) and `target_scriptrunner` (g_target).
pub const Q3_SCRIPT_DIR: &str = "scripts";

// `qboolean` lives at the FFI boundary; re-export so the foundation has one definition.
pub use crate::ffi::types::{qboolean, QFALSE, QTRUE};

/// `M_PI` — matches the literal baked into the original C (gcc v2 math.h value).
pub const M_PI: vec_t = 3.14159265358979323846;

/// Euler angle indices into a `vec3_t` of angles (from q_shared.h).
pub const PITCH: usize = 0; // up / down
pub const YAW: usize = 1; // left / right
pub const ROLL: usize = 2; // fall over

/// `errorParm_t` (q_shared.h) — parameters to the main `Com_Error`/`G_Error`
/// routine. (`ERR_FATAL` is `#undef`'d then re-defined in the original because
/// `malloc.h` also defines it.)
pub const ERR_FATAL: c_int = 0; // exit the entire game with a popup window
pub const ERR_DROP: c_int = 1; // print to console and disconnect from game
pub const ERR_SERVERDISCONNECT: c_int = 2; // don't kill server
pub const ERR_DISCONNECT: c_int = 3; // client disconnected from the server
pub const ERR_NEED_CD: c_int = 4; // pop up the need-cd dialog

/// `Q_COLOR_ESCAPE` (q_shared.h) — the `^` that introduces a `^N` color code.
pub const Q_COLOR_ESCAPE: c_char = b'^' as c_char;

pub const COLOR_BLACK: c_char = b'0' as c_char;
pub const COLOR_RED: c_char = b'1' as c_char;
pub const COLOR_GREEN: c_char = b'2' as c_char;
pub const COLOR_YELLOW: c_char = b'3' as c_char;
pub const COLOR_BLUE: c_char = b'4' as c_char;
pub const COLOR_CYAN: c_char = b'5' as c_char;
pub const COLOR_MAGENTA: c_char = b'6' as c_char;
pub const COLOR_WHITE: c_char = b'7' as c_char;

// say modes (q_shared.h:2972)
pub const SAY_ALL: c_int = 0;
pub const SAY_TEAM: c_int = 1;
pub const SAY_TELL: c_int = 2;

/// `Q_IsColorString(p)` (q_shared.h) — true if `p` points at a `^N` color code
/// with `N` in `0..=7`. Faithful translation of the C macro (the digit-range
/// comparisons are done against `c_char`, which orders identically to the C
/// `int`-promoted comparisons for every value a `char` can hold).
///
/// # Safety
/// `p` must be null or point to a NUL-terminated buffer with at least the byte at
/// `p` readable (and `p+1` when `*p == '^'`).
#[inline]
pub unsafe fn Q_IsColorString(p: *const c_char) -> bool {
    !p.is_null()
        && *p == Q_COLOR_ESCAPE
        && *p.add(1) != 0
        && *p.add(1) != Q_COLOR_ESCAPE
        && *p.add(1) <= b'7' as c_char
        && *p.add(1) >= b'0' as c_char
}

/// Plane `type` values (q_shared.h): 0,1,2 = axial (X/Y/Z), 3 = nonaxial.
pub const PLANE_X: byte = 0;
pub const PLANE_Y: byte = 1;
pub const PLANE_Z: byte = 2;
pub const PLANE_NON_AXIAL: byte = 3;

/// `cplane_t` (q_shared.h) — a plane plus the precomputed fields used by the fast
/// box-on-plane-side test. Pointer-free, so identical layout on 32- and 64-bit.
///
/// `type` is a Rust keyword, hence `r#type` (the C field name is `type`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub r#type: byte,   // for fast side tests: 0,1,2 = axial, 3 = nonaxial
    pub signbits: byte, // signx + (signy<<1) + (signz<<2), used as lookup during collision
    pub pad: [byte; 2],
}

// sizeof(cplane_t) is 20 on the original (12 + 4 + 1 + 1 + 2), align 4.
const _: () = assert!(core::mem::size_of::<cplane_t>() == 20);
const _: () = assert!(core::mem::align_of::<cplane_t>() == 4);

// ---------------------------------------------------------------------------
// CollisionRecord_t / G2Trace_t (q_shared.h, "Ghoul2 Insert Start") — one entry
// per ghoul2 part hit by a `trap_G2API_CollisionDetect` per-poly trace.
// ---------------------------------------------------------------------------

/// `CollisionRecord_t` (q_shared.h) — a single ghoul2 per-poly collision hit: which
/// model/poly/surface was struck, the world-space hit position/normal, and the
/// barycentric coordinates of the hit point on the triangle. Pointer-free, `repr(C)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CollisionRecord_t {
    pub mDistance: f32,
    pub mEntityNum: c_int,
    pub mModelIndex: c_int,
    pub mPolyIndex: c_int,
    pub mSurfaceIndex: c_int,
    pub mCollisionPosition: vec3_t,
    pub mCollisionNormal: vec3_t,
    pub mFlags: c_int,
    pub mMaterial: c_int,
    pub mLocation: c_int,
    pub mBarycentricI: f32, // two barycentic coodinates for the hit point
    pub mBarycentricJ: f32, // K = 1-I-J
}

/// `MAX_G2_COLLISIONS` (q_shared.h) — the fixed size of a [`G2Trace_t`] collision map.
pub const MAX_G2_COLLISIONS: usize = 16;

/// `G2Trace_t` (q_shared.h) — `CollisionRecord_t[MAX_G2_COLLISIONS]`: the full map that
/// describes all of the parts of ghoul2 models that got hit by a per-poly trace.
pub type G2Trace_t = [CollisionRecord_t; MAX_G2_COLLISIONS];

/// `DEG2RAD(a)` from q_shared.h.
#[inline]
pub fn DEG2RAD(a: vec_t) -> vec_t {
    (a * M_PI) / 180.0
}

/// `RAD2DEG(a)` from q_shared.h.
#[inline]
pub fn RAD2DEG(a: vec_t) -> vec_t {
    (a * 180.0) / M_PI
}

/// `ANGLE2SHORT(x)` (q_shared.h) — `((int)((x)*65536/360) & 65535)`. The arithmetic
/// stays in `float` (`65536`/`360` are int literals converted to float), then truncates
/// to `int` and masks to 16 bits.
#[inline]
pub fn ANGLE2SHORT(x: vec_t) -> c_int {
    ((x * 65536.0 / 360.0) as c_int) & 65535
}

/// `SHORT2ANGLE(x)` (q_shared.h) — `((x)*(360.0/65536))`. The `360.0` literal promotes
/// the multiply to `double`, so the constant is computed in `f64` before narrowing.
#[inline]
pub fn SHORT2ANGLE(x: c_int) -> vec_t {
    (x as f64 * (360.0 / 65536.0)) as vec_t
}

/// Inline `SnapVector` (q_shared.h:1310/1331) — the *macro/inline* form compiled into the
/// game module, **not** `trap_SnapVector` (the engine syscall that [`crate::trap::SnapVector`]
/// wraps and that `bg_pmove.c` calls). Both `bg_misc.c` (`BG_PlayerStateToEntityState`) and
/// `g_utils.c` (`G_TempEntity`/`G_SoundTempEntity`) invoke this local form, so it lives in the
/// `q_shared.h` module they share. The shipped retail (Win32) build uses the `__asm fistp`
/// version, which rounds each component to the nearest integer with ties to even under the
/// default x87 rounding mode — [`f32::round_ties_even`] is the exact match. (The LCC/VM build's
/// `(int)`-truncation `#define` variant is *not* the ABI target; see `DEVIATIONS.md`.)
pub(crate) fn snap_vector(v: &mut vec3_t) {
    v[0] = v[0].round_ties_even();
    v[1] = v[1].round_ties_even();
    v[2] = v[2].round_ties_even();
}

// ===========================================================================
// Per-level limits (q_shared.h "per-level limits" block).
// ===========================================================================

/// `MAX_CLIENTS` — absolute client limit (non-Xbox value; the `_XBOX` build uses 10).
pub const MAX_CLIENTS: usize = 32;

/// `GENTITYNUM_BITS` — entitynums are communicated with this many bits.
pub const GENTITYNUM_BITS: c_int = 10;
/// `MAX_GENTITIES` = `1 << GENTITYNUM_BITS`.
pub const MAX_GENTITIES: usize = 1 << GENTITYNUM_BITS;

// entitynums are communicated with GENTITY_BITS, so any reserved values that
// are going to be communicated over the net need to also be in this range
/// `ENTITYNUM_NONE` = `MAX_GENTITIES - 1`.
pub const ENTITYNUM_NONE: c_int = MAX_GENTITIES as c_int - 1;
/// `ENTITYNUM_WORLD` = `MAX_GENTITIES - 2`.
pub const ENTITYNUM_WORLD: c_int = MAX_GENTITIES as c_int - 2;
/// `ENTITYNUM_MAX_NORMAL` = `MAX_GENTITIES - 2`.
pub const ENTITYNUM_MAX_NORMAL: c_int = MAX_GENTITIES as c_int - 2;

/// `MAX_RADAR_ENTITIES` = `MAX_GENTITIES`.
pub const MAX_RADAR_ENTITIES: usize = MAX_GENTITIES;
/// `MAX_TERRAINS` (rwwRMG inserted) — the C value is 1 (`1//32`).
pub const MAX_TERRAINS: usize = 1;
/// `MAX_LOCATIONS`.
pub const MAX_LOCATIONS: usize = 64;

// these are also in be_aas_def.h - argh (rjr)
/// `MAX_MODELS` — these are sent over the net as -12 bits.
pub const MAX_MODELS: usize = 512;
/// `MAX_SOUNDS` — so they cannot be blindly increased.
pub const MAX_SOUNDS: usize = 256;
/// `MAX_ICONS` — max registered icons you can have per map.
pub const MAX_ICONS: usize = 64;
/// `MAX_FX` — max effects strings.
pub const MAX_FX: usize = 64;
// (`MAX_SUB_BSP` 32 is commented out in q_shared.h; the CS_MAX chain uses the
// non-SUB_BSP form, so it is not ported.)

/// `MAX_G2BONES` (Ghoul2; rww: was MAX_CHARSKINS, value still equal).
pub const MAX_G2BONES: usize = 64;

/// `MAX_AMBIENT_SETS` — ambient soundsets, sent over in config strings.
pub const MAX_AMBIENT_SETS: usize = 256;

/// `MAX_CONFIGSTRINGS` — "this is getting pretty high. Try not to raise it."
pub const MAX_CONFIGSTRINGS: usize = 1700;

/// `MAX_LIGHT_STYLES` (q_shared.h, defined near the light-style block).
pub const MAX_LIGHT_STYLES: usize = 64;

// The only configstrings the system reserves; all others are strictly for
// servergame->clientgame communication.
/// `CS_SERVERINFO` — an info string with all the serverinfo cvars.
pub const CS_SERVERINFO: c_int = 0;
/// `CS_SYSTEMINFO` — an info string for server system to client system config.
pub const CS_SYSTEMINFO: c_int = 1;

// ===========================================================================
// bit field limits (q_shared.h).
// ===========================================================================

pub const MAX_STATS: usize = 16;
pub const MAX_PERSISTANT: usize = 16;
pub const MAX_POWERUPS: usize = 16;
pub const MAX_WEAPONS: usize = 19;

pub const MAX_PS_EVENTS: usize = 2;

/// `PS_PMOVEFRAMECOUNTBITS` (q_shared.h:2047) — width of the `pmove_framecount`
/// field that is transmitted in the player state; `Pmove` masks the counter to
/// this many bits each frame.
pub const PS_PMOVEFRAMECOUNTBITS: c_int = 6;

/// `FORCE_LIGHTSIDE` (q_shared.h:2049) / `FORCE_DARKSIDE` (q_shared.h:2050) — the two
/// force alignments. `0` means neutral (used in `forcePowerDarkLight[]`). These index
/// nothing; they tag a force power's side and a client's chosen side.
pub const FORCE_LIGHTSIDE: c_int = 1;
pub const FORCE_DARKSIDE: c_int = 2;

// ===========================================================================
// Force powers (q_shared.h `forcePowers_t`) — gives `NUM_FORCE_POWERS`, which
// sizes several `forcedata_t` arrays. `FP_FIRST`/`FP_HEAL` are both 0.
// ===========================================================================

pub const FP_FIRST: c_int = 0; // marker
pub const FP_HEAL: c_int = 0; // instant
pub const FP_LEVITATION: c_int = 1; // hold/duration
pub const FP_SPEED: c_int = 2; // duration
pub const FP_PUSH: c_int = 3; // hold/duration
pub const FP_PULL: c_int = 4; // hold/duration
pub const FP_TELEPATHY: c_int = 5; // instant
pub const FP_GRIP: c_int = 6; // hold/duration
pub const FP_LIGHTNING: c_int = 7; // hold/duration
pub const FP_RAGE: c_int = 8; // duration
pub const FP_PROTECT: c_int = 9;
pub const FP_ABSORB: c_int = 10;
pub const FP_TEAM_HEAL: c_int = 11;
pub const FP_TEAM_FORCE: c_int = 12;
pub const FP_DRAIN: c_int = 13;
pub const FP_SEE: c_int = 14;
pub const FP_SABER_OFFENSE: c_int = 15;
pub const FP_SABER_DEFENSE: c_int = 16;
pub const FP_SABERTHROW: c_int = 17;
/// `NUM_FORCE_POWERS` — count of the `FP_*` powers (18).
pub const NUM_FORCE_POWERS: usize = 18;

// ===========================================================================
// Sound channels (q_shared.h `soundChannel_t`). Channel 0 (`CHAN_AUTO`) never
// willingly overrides; every other channel overrides a playing sound on it.
// ===========================================================================

pub const CHAN_AUTO: c_int = 0;
pub const CHAN_LOCAL: c_int = 1;
pub const CHAN_WEAPON: c_int = 2;
pub const CHAN_VOICE: c_int = 3;
pub const CHAN_VOICE_ATTEN: c_int = 4;
pub const CHAN_ITEM: c_int = 5;
pub const CHAN_BODY: c_int = 6;
pub const CHAN_AMBIENT: c_int = 7;
pub const CHAN_LOCAL_SOUND: c_int = 8;
pub const CHAN_ANNOUNCER: c_int = 9;
pub const CHAN_LESS_ATTEN: c_int = 10;
pub const CHAN_MENU1: c_int = 11;
pub const CHAN_VOICE_GLOBAL: c_int = 12;
pub const CHAN_MUSIC: c_int = 13;

// ===========================================================================
// Tracking channels (q_shared.h `trackchan_t`) — `TRACK_CHANNEL_MAX` sizes the
// `forcedata_t::killSoundEntIndex` array. The enum starts at 50.
// ===========================================================================

pub const TRACK_CHANNEL_NONE: c_int = 50;
pub const TRACK_CHANNEL_1: c_int = 51;
pub const TRACK_CHANNEL_2: c_int = 52;
pub const TRACK_CHANNEL_3: c_int = 53;
pub const TRACK_CHANNEL_4: c_int = 54;
pub const TRACK_CHANNEL_5: c_int = 55;
pub const NUM_TRACK_CHANNELS: c_int = 56;
/// `TRACK_CHANNEL_MAX` = `NUM_TRACK_CHANNELS - 50` (6).
pub const TRACK_CHANNEL_MAX: usize = (NUM_TRACK_CHANNELS - 50) as usize;

// ===========================================================================
// Networked master structs (q_shared.h). These six are pointer-free, so they
// have identical layout on 32- and 64-bit; mirrored field-for-field as faithful
// `#[repr(C)]`. Size asserts below match the C `sizeof` on the build arch
// (verified against an oracle TU; see `oracle/q_shared_h_oracle.c`).
// ===========================================================================

/// `SOLID_BMODEL` (q_shared.h) — if `entityState->solid == SOLID_BMODEL`,
/// modelindex is an inline model number.
pub const SOLID_BMODEL: c_int = 0xffffff;

/// `trType_t` (q_shared.h) — how a `trajectory_t` evolves over time.
pub const TR_STATIONARY: c_int = 0;
pub const TR_INTERPOLATE: c_int = 1; // non-parametric, but interpolate between snapshots
pub const TR_LINEAR: c_int = 2;
pub const TR_LINEAR_STOP: c_int = 3;
pub const TR_NONLINEAR_STOP: c_int = 4;
pub const TR_SINE: c_int = 5; // value = base + sin( time / duration ) * delta
pub const TR_GRAVITY: c_int = 6;
/// `trType_t` is a C enum, transmitted/stored as a plain `int`.
pub type trType_t = c_int;

/// `trajectory_t` (q_shared.h) — parametric description of a moving value
/// (position or angles). Pointer-free; identical layout on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct trajectory_t {
    pub trType: trType_t,
    pub trTime: c_int,
    pub trDuration: c_int, // if non 0, trTime + trDuration = stop time
    pub trBase: vec3_t,
    pub trDelta: vec3_t, // velocity, etc
}

const _: () = assert!(core::mem::size_of::<trajectory_t>() == 36);
const _: () = assert!(core::mem::align_of::<trajectory_t>() == 4);

// `connstate_t` (q_shared.h) — the *client-side* connection state. Only the two values that
// ai_main.c's `GetIdealDestination` compares against are needed here.
pub type connstate_t = c_int;
pub const CA_UNINITIALIZED: connstate_t = 0;
pub const CA_DISCONNECTED: connstate_t = 1;
pub const CA_AUTHORIZING: connstate_t = 2;
pub const CA_CONNECTING: connstate_t = 3;
pub const CA_CHALLENGING: connstate_t = 4;
pub const CA_CONNECTED: connstate_t = 5;
pub const CA_LOADING: connstate_t = 6;
pub const CA_PRIMED: connstate_t = 7;
pub const CA_ACTIVE: connstate_t = 8;
pub const CA_CINEMATIC: connstate_t = 9;

// ---------------------------------------------------------------------------
// usercmd_t (q_shared.h).
// ---------------------------------------------------------------------------

// usercmd_t->button bits, many of which are generated by the client system,
// so they aren't game/cgame only definitions
pub const BUTTON_ATTACK: c_int = 1;
pub const BUTTON_TALK: c_int = 2; // displays talk balloon and disables actions
pub const BUTTON_USE_HOLDABLE: c_int = 4;
pub const BUTTON_GESTURE: c_int = 8;
pub const BUTTON_WALKING: c_int = 16; // walking can't just be infered from MOVE_RUN
                                      // because a key pressed late in the frame will
                                      // only generate a small move value for that frame
                                      // walking will use different animations and
                                      // won't generate footsteps
pub const BUTTON_USE: c_int = 32; // the ol' use key returns!
pub const BUTTON_FORCEGRIP: c_int = 64;
pub const BUTTON_ALT_ATTACK: c_int = 128;
pub const BUTTON_ANY: c_int = 256; // any key whatsoever
pub const BUTTON_FORCEPOWER: c_int = 512; // use the "active" force power
pub const BUTTON_FORCE_LIGHTNING: c_int = 1024;
pub const BUTTON_FORCE_DRAIN: c_int = 2048;

/// `MOVE_RUN` — if forwardmove or rightmove are >= MOVE_RUN, then
/// BUTTON_WALKING should be set.
pub const MOVE_RUN: c_int = 120;

/// `genCmds_t` (q_shared.h) — generic command ids carried in the `byte`
/// `usercmd_t::generic_cmd` field. Starts at 1.
pub const GENCMD_SABERSWITCH: c_int = 1;
pub const GENCMD_ENGAGE_DUEL: c_int = 2;
pub const GENCMD_FORCE_HEAL: c_int = 3;
pub const GENCMD_FORCE_SPEED: c_int = 4;
pub const GENCMD_FORCE_THROW: c_int = 5;
pub const GENCMD_FORCE_PULL: c_int = 6;
pub const GENCMD_FORCE_DISTRACT: c_int = 7;
pub const GENCMD_FORCE_RAGE: c_int = 8;
pub const GENCMD_FORCE_PROTECT: c_int = 9;
pub const GENCMD_FORCE_ABSORB: c_int = 10;
pub const GENCMD_FORCE_HEALOTHER: c_int = 11;
pub const GENCMD_FORCE_FORCEPOWEROTHER: c_int = 12;
pub const GENCMD_FORCE_SEEING: c_int = 13;
pub const GENCMD_USE_SEEKER: c_int = 14;
pub const GENCMD_USE_FIELD: c_int = 15;
pub const GENCMD_USE_BACTA: c_int = 16;
pub const GENCMD_USE_ELECTROBINOCULARS: c_int = 17;
pub const GENCMD_ZOOM: c_int = 18;
pub const GENCMD_USE_SENTRY: c_int = 19;
pub const GENCMD_USE_JETPACK: c_int = 20;
pub const GENCMD_USE_BACTABIG: c_int = 21;
pub const GENCMD_USE_HEALTHDISP: c_int = 22;
pub const GENCMD_USE_AMMODISP: c_int = 23;
pub const GENCMD_USE_EWEB: c_int = 24;
pub const GENCMD_USE_CLOAK: c_int = 25;
pub const GENCMD_SABERATTACKCYCLE: c_int = 26;
pub const GENCMD_TAUNT: c_int = 27;
pub const GENCMD_BOW: c_int = 28;
pub const GENCMD_MEDITATE: c_int = 29;
pub const GENCMD_FLOURISH: c_int = 30;
pub const GENCMD_GLOAT: c_int = 31;
// (The C `genCmds_t` enum itself is not aliased: its only storage here is the
// `byte` `usercmd_t::generic_cmd` field, not an int-width enum type.)

/// `usercmd_t` (q_shared.h) — sent to the server each client frame.
/// Pointer-free; identical layout on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct usercmd_t {
    pub serverTime: c_int,
    pub angles: [c_int; 3],
    pub buttons: c_int,
    pub weapon: byte, // weapon
    pub forcesel: byte,
    pub invensel: byte,
    pub generic_cmd: byte,
    pub forwardmove: i8, // signed char
    pub rightmove: i8,
    pub upmove: i8,
}

const _: () = assert!(core::mem::size_of::<usercmd_t>() == 28);
const _: () = assert!(core::mem::align_of::<usercmd_t>() == 4);

// ---------------------------------------------------------------------------
// trace_t (q_shared.h).
// ---------------------------------------------------------------------------

/// `trace_t` (q_shared.h) — returned when a box is swept through the world.
/// Pointer-free; identical layout on 32/64-bit. (The Ghoul2 `G2CollisionMap`
/// member is commented out in the original "to avoid wasting space in the trace
/// structure", so it is absent here too.)
///
/// `trace->entityNum` can also be 0 to (MAX_GENTITIES-1) or ENTITYNUM_NONE,
/// ENTITYNUM_WORLD.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct trace_t {
    pub allsolid: byte,      // if true, plane is not valid
    pub startsolid: byte,    // if true, the initial point was in a solid area
    pub entityNum: i16,      // entity the contacted sirface is a part of
    pub fraction: f32,       // time completed, 1.0 = didn't hit anything
    pub endpos: vec3_t,      // final position
    pub plane: cplane_t,     // surface normal at impact, transformed to world space
    pub surfaceFlags: c_int, // surface hit
    pub contents: c_int,     // contents on other side of surface hit
}

const _: () = assert!(core::mem::size_of::<trace_t>() == 48);
const _: () = assert!(core::mem::align_of::<trace_t>() == 4);
const _: () = assert!(core::mem::offset_of!(trace_t, plane) == 20);

// ---------------------------------------------------------------------------
// forcedata_t (q_shared.h) — embedded inside playerState_t as `fd`.
// ---------------------------------------------------------------------------

/// `forcedata_t` (q_shared.h) — a player's force-power state. Pointer-free;
/// arrays sized by `NUM_FORCE_POWERS` (18) and `TRACK_CHANNEL_MAX` (6).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct forcedata_t {
    pub forcePowerDebounce: [c_int; NUM_FORCE_POWERS], // for effects that must have an interval
    pub forcePowersKnown: c_int,
    pub forcePowersActive: c_int,
    pub forcePowerSelected: c_int,
    pub forceButtonNeedRelease: c_int,
    pub forcePowerDuration: [c_int; NUM_FORCE_POWERS],
    pub forcePower: c_int,
    pub forcePowerMax: c_int,
    pub forcePowerRegenDebounceTime: c_int,
    pub forcePowerLevel: [c_int; NUM_FORCE_POWERS], // so we know the max forceJump power you have
    pub forcePowerBaseLevel: [c_int; NUM_FORCE_POWERS],
    pub forceUsingAdded: c_int,
    pub forceJumpZStart: f32, // So when you land, you don't get hurt as much
    pub forceJumpCharge: f32, // you're current forceJump charge-up level, increases the longer you hold the force jump button down
    pub forceJumpSound: c_int,
    pub forceJumpAddTime: c_int,
    pub forceGripEntityNum: c_int,          // what entity I'm gripping
    pub forceGripDamageDebounceTime: c_int, // debounce for grip damage
    pub forceGripBeingGripped: f32,         // if > level.time then client is in someone's grip
    pub forceGripCripple: c_int, // if != 0 then make it so this client can't move quickly (he's being gripped)
    pub forceGripUseTime: c_int, // can't use if > level.time
    pub forceGripSoundTime: f32,
    pub forceGripStarted: f32, // level.time when the grip was activated
    pub forceHealTime: c_int,
    pub forceHealAmount: c_int,
    // This hurts me somewhat to do, but there's no other real way to allow completely "dynamic" mindtricking.
    pub forceMindtrickTargetIndex: c_int,  // 0-15
    pub forceMindtrickTargetIndex2: c_int, // 16-32
    pub forceMindtrickTargetIndex3: c_int, // 33-48
    pub forceMindtrickTargetIndex4: c_int, // 49-64
    pub forceRageRecoveryTime: c_int,
    pub forceDrainEntNum: c_int,
    pub forceDrainTime: f32,
    pub forceDoInit: c_int,
    pub forceSide: c_int,
    pub forceRank: c_int,
    pub forceDeactivateAll: c_int,
    pub killSoundEntIndex: [c_int; TRACK_CHANNEL_MAX], // this goes here so it doesn't get wiped over respawn
    pub sentryDeployed: qboolean,
    pub saberAnimLevelBase: c_int, // sigh...
    pub saberAnimLevel: c_int,
    pub saberDrawAnimLevel: c_int,
    pub suicides: c_int,
    pub privateDuelTime: c_int,
}

const _: () = assert!(core::mem::size_of::<forcedata_t>() == 464);
const _: () = assert!(core::mem::align_of::<forcedata_t>() == 4);

/// `itemUseFail_t` (q_shared.h) — reason codes sent as the parm of an
/// `EV_ITEMUSEFAIL` event (read by `PM_ItemUsable`). Starts at 1.
pub const SENTRY_NOROOM: c_int = 1;
pub const SENTRY_ALREADYPLACED: c_int = 2;
pub const SHIELD_NOROOM: c_int = 3;
pub const SEEKER_ALREADYDEPLOYED: c_int = 4;
/// `itemUseFail_t` is a C enum, transmitted/stored as a plain `int`.
pub type itemUseFail_t = c_int;

// ---------------------------------------------------------------------------
// entityState_t (q_shared.h).
// ---------------------------------------------------------------------------

/// `entityState_t` (q_shared.h) — the information conveyed from the server in
/// an update message about entities that the client will need to render in some
/// way. The messages are delta compressed, so it doesn't really matter if the
/// structure size is fairly large.
///
/// This is the PC version (the `#ifndef _XBOX` branch, with all members
/// 32-bit); the tightly-packed `_XBOX` variant is not ported. Pointer-free;
/// identical layout on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct entityState_t {
    pub number: c_int, // entity index
    pub eType: c_int,  // entityType_t
    pub eFlags: c_int,
    pub eFlags2: c_int, // EF2_??? used much less frequently

    pub pos: trajectory_t,  // for calculating position
    pub apos: trajectory_t, // for calculating angles

    pub time: c_int,
    pub time2: c_int,

    pub origin: vec3_t,
    pub origin2: vec3_t,

    pub angles: vec3_t,
    pub angles2: vec3_t,

    // rww - these were originally because we shared g2 info client and server side.
    // Now they just get used as generic values everywhere.
    pub bolt1: c_int,
    pub bolt2: c_int,

    // rww - this is necessary for determining player visibility during a jedi mindtrick
    pub trickedentindex: c_int,  // 0-15
    pub trickedentindex2: c_int, // 16-32
    pub trickedentindex3: c_int, // 33-48
    pub trickedentindex4: c_int, // 49-64

    pub speed: f32,

    pub fireflag: c_int,

    pub genericenemyindex: c_int,

    pub activeForcePass: c_int,

    pub emplacedOwner: c_int,

    pub otherEntityNum: c_int, // shotgun sources, etc
    pub otherEntityNum2: c_int,

    pub groundEntityNum: c_int, // -1 = in air

    pub constantLight: c_int,     // r + (g<<8) + (b<<16) + (intensity<<24)
    pub loopSound: c_int,         // constantly loop this sound
    pub loopIsSoundset: qboolean, // qtrue if the loopSound index is actually a soundset index

    pub soundSetIndex: c_int,

    pub modelGhoul2: c_int,
    pub g2radius: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub clientNum: c_int, // 0 to (MAX_CLIENTS - 1), for players and corpses
    pub frame: c_int,

    pub saberInFlight: qboolean,
    pub saberEntityNum: c_int,
    pub saberMove: c_int,
    pub forcePowersActive: c_int,
    pub saberHolstered: c_int, // sent in only only 2 bits - should be 0, 1 or 2

    pub isJediMaster: qboolean, // retail-PC: active in the `#ifndef _XBOX` 32-bit entityState (Xbox commented it out)

    pub isPortalEnt: qboolean, // this needs to be seperate for all entities I guess, which is why I couldn't reuse another value.

    pub solid: c_int, // for client side prediction, trap_linkentity sets this properly

    pub event: c_int, // impulse events -- muzzle flashes, footsteps, etc
    pub eventParm: c_int,

    // so crosshair knows what it's looking at
    pub owner: c_int,
    pub teamowner: c_int,
    pub shouldtarget: qboolean,

    // for players
    pub powerups: c_int, // bit flags
    pub weapon: c_int,   // determines weapon and flash model, etc
    pub legsAnim: c_int,
    pub torsoAnim: c_int,

    pub legsFlip: qboolean, // set to opposite when the same anim needs restarting, sent over in only 1 bit. Cleaner and makes porting easier than having that god forsaken ANIM_TOGGLEBIT.
    pub torsoFlip: qboolean,

    pub forceFrame: c_int, // if non-zero, force the anim frame

    pub generic1: c_int,

    pub heldByClient: c_int, // can only be a client index - this client should be holding onto my arm using IK stuff.

    pub ragAttach: c_int, // attach to ent while ragging

    pub iModelScale: c_int, // rww - transfer a percentage of the normal scale in a single int instead of 3 x-y-z scale values

    pub brokenLimbs: c_int,

    pub boltToPlayer: c_int, // set to index of a real client+1 to bolt the ent to that client. Must be a real client, NOT an NPC.

    // for looking at an entity's origin (NPCs and players)
    pub hasLookTarget: qboolean,
    pub lookTarget: c_int,

    pub customRGBA: [c_int; 4],

    // I didn't want to do this, but I.. have no choice. However, we aren't setting this for all ents or anything,
    // only ones we want health knowledge about on cgame (like siege objective breakables) -rww
    pub health: c_int,
    pub maxhealth: c_int, // so I know how to draw the stupid health bar

    // NPC-SPECIFIC FIELDS
    //------------------------------------------------------------
    pub npcSaber1: c_int,
    pub npcSaber2: c_int,

    // index values for each type of sound, gets the folder the sounds
    // are in. I wish there were a better way to do this,
    pub csSounds_Std: c_int,
    pub csSounds_Combat: c_int,
    pub csSounds_Extra: c_int,
    pub csSounds_Jedi: c_int,

    pub surfacesOn: c_int, // a bitflag of corresponding surfaces from a lookup table. These surfaces will be forced on.
    pub surfacesOff: c_int, // same as above, but forced off instead.

    // Allow up to 4 PCJ lookup values to be stored here.
    // The resolve to configstrings which contain the name of the
    // desired bone.
    pub boneIndex1: c_int,
    pub boneIndex2: c_int,
    pub boneIndex3: c_int,
    pub boneIndex4: c_int,

    // packed with x, y, z orientations for bone angles
    pub boneOrient: c_int,

    // I.. feel bad for doing this, but NPCs really just need to
    // be able to control this sort of thing from the server sometimes.
    // At least it's at the end so this stuff is never going to get sent
    // over for anything that isn't an NPC.
    pub boneAngles1: vec3_t, // angles of boneIndex1
    pub boneAngles2: vec3_t, // angles of boneIndex2
    pub boneAngles3: vec3_t, // angles of boneIndex3
    pub boneAngles4: vec3_t, // angles of boneIndex4

    pub NPC_class: c_int, // we need to see what it is on the client for a few effects.

    // If non-0, this is the index of the vehicle a player/NPC is riding.
    pub m_iVehicleNum: c_int,

    // rww - spare values specifically for use by mod authors.
    // See netf_overrides.txt if you want to increase the send
    // amount of any of these above 1 bit.
    pub userInt1: c_int,
    pub userInt2: c_int,
    pub userInt3: c_int,
    pub userFloat1: f32,
    pub userFloat2: f32,
    pub userFloat3: f32,
    pub userVec1: vec3_t,
    pub userVec2: vec3_t,
}

const _: () = assert!(core::mem::size_of::<entityState_t>() == 532);
const _: () = assert!(core::mem::align_of::<entityState_t>() == 4);
const _: () = assert!(core::mem::offset_of!(entityState_t, pos) == 16);
const _: () = assert!(core::mem::offset_of!(entityState_t, userVec2) == 520);

// ---------------------------------------------------------------------------
// playerState_t (q_shared.h).
// ---------------------------------------------------------------------------

/// `playerState_t` (q_shared.h) — the information needed by both the client and
/// server to predict player motion and actions. Nothing outside of pmove should
/// modify these, or some degree of prediction error will occur. You can't add
/// anything to this without modifying the code in msg.c.
///
/// playerState_t is a full superset of entityState_t as it is used by players,
/// so if a playerState_t is transmitted, the entityState_t can be fully derived
/// from it.
///
/// Ported with `_XBOX` undefined (so the `userInt*`/`userFloat*`/`userVec*`
/// block is present) and `_ONEBIT_COMBO` undefined (so the trailing
/// `deltaOneBits`/`deltaNumBits` are absent), matching the PC build.
/// Pointer-free; identical layout on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct playerState_t {
    pub commandTime: c_int, // cmd->serverTime of last executed command
    pub pm_type: c_int,
    pub bobCycle: c_int, // for view bobbing and footstep generation
    pub pm_flags: c_int, // ducked, jump_held, etc
    pub pm_time: c_int,

    pub origin: vec3_t,
    pub velocity: vec3_t,

    pub moveDir: vec3_t, // NOT sent over the net - nor should it be.

    pub weaponTime: c_int,
    pub weaponChargeTime: c_int,
    pub weaponChargeSubtractTime: c_int,
    pub gravity: c_int,
    pub speed: f32,
    pub basespeed: c_int, // used in prediction to know base server g_speed value when modifying speed between updates
    pub delta_angles: [c_int; 3], // add to command angles to get view direction
    // changed by spawns, rotating objects, and teleporters
    pub slopeRecalcTime: c_int, // this is NOT sent across the net and is maintained seperately on game and cgame in pmove code.

    pub useTime: c_int,

    pub groundEntityNum: c_int, // ENTITYNUM_NONE = in air

    pub legsTimer: c_int, // don't change low priority animations until this runs out
    pub legsAnim: c_int,

    pub torsoTimer: c_int, // don't change low priority animations until this runs out
    pub torsoAnim: c_int,

    pub legsFlip: qboolean, // set to opposite when the same anim needs restarting, sent over in only 1 bit. Cleaner and makes porting easier than having that god forsaken ANIM_TOGGLEBIT.
    pub torsoFlip: qboolean,

    pub movementDir: c_int, // a number 0 to 7 that represents the reletive angle
    // of movement to the view angle (axial and diagonals)
    // when at rest, the value will remain unchanged
    // used to twist the legs during strafing
    pub eFlags: c_int,  // copied to entityState_t->eFlags
    pub eFlags2: c_int, // copied to entityState_t->eFlags2, EF2_??? used much less frequently

    pub eventSequence: c_int, // pmove generated events
    pub events: [c_int; MAX_PS_EVENTS],
    pub eventParms: [c_int; MAX_PS_EVENTS],

    pub externalEvent: c_int, // events set on player from another source
    pub externalEventParm: c_int,
    pub externalEventTime: c_int,

    pub clientNum: c_int, // ranges from 0 to MAX_CLIENTS-1
    pub weapon: c_int,    // copied to entityState_t->weapon
    pub weaponstate: c_int,

    pub viewangles: vec3_t, // for fixed views
    pub viewheight: c_int,

    // damage feedback
    pub damageEvent: c_int, // when it changes, latch the other parms
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,
    pub damageType: c_int,

    pub painTime: c_int, // used for both game and client side to process the pain twitch - NOT sent across the network
    pub painDirection: c_int, // NOT sent across the network
    pub yawAngle: f32,   // NOT sent across the network
    pub yawing: qboolean, // NOT sent across the network
    pub pitchAngle: f32, // NOT sent across the network
    pub pitching: qboolean, // NOT sent across the network

    pub stats: [c_int; MAX_STATS],
    pub persistant: [c_int; MAX_PERSISTANT], // stats that aren't cleared on death
    pub powerups: [c_int; MAX_POWERUPS],     // level.time that the powerup runs out
    pub ammo: [c_int; MAX_WEAPONS],

    pub generic1: c_int,
    pub loopSound: c_int,
    pub jumppad_ent: c_int, // jumppad entity hit this frame

    // not communicated over the net at all
    pub ping: c_int,             // server to game info for scoreboard
    pub pmove_framecount: c_int, // FIXME: don't transmit over the network
    pub jumppad_frame: c_int,
    pub entityEventSequence: c_int,

    pub lastOnGround: c_int, // last time you were on the ground

    pub saberInFlight: qboolean,

    pub saberMove: c_int,
    pub saberBlocking: c_int,
    pub saberBlocked: c_int,

    pub saberLockTime: c_int,
    pub saberLockEnemy: c_int,
    pub saberLockFrame: c_int, // since we don't actually have the ability to get the current anim frame
    pub saberLockHits: c_int, // every x number of buttons hits, allow one push forward in a saber lock (server only)
    pub saberLockHitCheckTime: c_int, // so we don't allow more than 1 push per server frame
    pub saberLockHitIncrementTime: c_int, // so we don't add a hit per attack button press more than once per server frame
    pub saberLockAdvance: qboolean,       // do an advance (sent across net as 1 bit)

    pub saberEntityNum: c_int,
    pub saberEntityDist: f32,
    pub saberEntityState: c_int,
    pub saberThrowDelay: c_int,
    pub saberCanThrow: qboolean,
    pub saberDidThrowTime: c_int,
    pub saberDamageDebounceTime: c_int,
    pub saberHitWallSoundDebounceTime: c_int,
    pub saberEventFlags: c_int,

    pub rocketLockIndex: c_int,
    pub rocketLastValidTime: f32,
    pub rocketLockTime: f32,
    pub rocketTargetTime: f32,

    pub emplacedIndex: c_int,
    pub emplacedTime: f32,

    pub isJediMaster: qboolean, // retail-PC: active (Xbox commented it out)
    pub forceRestricted: qboolean,
    pub trueJedi: qboolean,
    pub trueNonJedi: qboolean,
    pub saberIndex: c_int,

    pub genericEnemyIndex: c_int,
    pub droneFireTime: f32,
    pub droneExistTime: f32,

    pub activeForcePass: c_int,

    pub hasDetPackPlanted: qboolean, // better than taking up an eFlag isn't it?

    // retail-PC: this holocron block is active (Xbox wrapped it in /* */)
    pub holocronsCarried: [f32; NUM_FORCE_POWERS],
    pub holocronCantTouch: c_int,
    pub holocronCantTouchTime: f32, // for keeping track of the last holocron that just popped out of me (if any)
    pub holocronBits: c_int,

    pub electrifyTime: c_int,

    pub saberAttackSequence: c_int,
    pub saberIdleWound: c_int,
    pub saberAttackWound: c_int,
    pub saberBlockTime: c_int,

    pub otherKiller: c_int,
    pub otherKillerTime: c_int,
    pub otherKillerDebounceTime: c_int,

    pub fd: forcedata_t,
    pub forceJumpFlip: qboolean,
    pub forceHandExtend: c_int,
    pub forceHandExtendTime: c_int,

    pub forceRageDrainTime: c_int,

    pub forceDodgeAnim: c_int,
    pub quickerGetup: qboolean,

    pub groundTime: c_int, // time when first left ground

    pub footstepTime: c_int,

    pub otherSoundTime: c_int,
    pub otherSoundLen: f32,

    pub forceGripMoveInterval: c_int,
    pub forceGripChangeMovetype: c_int,

    pub forceKickFlip: c_int,

    pub duelIndex: c_int,
    pub duelTime: c_int,
    pub duelInProgress: qboolean,

    pub saberAttackChainCount: c_int,

    pub saberHolstered: c_int,

    pub forceAllowDeactivateTime: c_int,

    // zoom key
    pub zoomMode: c_int, // 0 - not zoomed, 1 - disruptor weapon
    pub zoomTime: c_int,
    pub zoomLocked: qboolean,
    pub zoomFov: f32,
    pub zoomLockTime: c_int,

    pub fallingToDeath: c_int,

    pub useDelay: c_int,

    pub inAirAnim: qboolean,

    pub lastHitLoc: vec3_t,

    pub heldByClient: c_int, // can only be a client index - this client should be holding onto my arm using IK stuff.

    pub ragAttach: c_int, // attach to ent while ragging

    pub iModelScale: c_int,

    pub brokenLimbs: c_int,

    // for looking at an entity's origin (NPCs and players)
    pub hasLookTarget: qboolean,
    pub lookTarget: c_int,

    pub customRGBA: [c_int; 4],

    pub standheight: c_int,
    pub crouchheight: c_int,

    // If non-0, this is the index of the vehicle a player/NPC is riding.
    pub m_iVehicleNum: c_int,

    // lovely hack for keeping vehicle orientation in sync with prediction
    pub vehOrientation: vec3_t,
    pub vehBoarding: qboolean,
    pub vehSurfaces: c_int,

    // vehicle turnaround stuff (need this in ps so it doesn't jerk too much in prediction)
    pub vehTurnaroundIndex: c_int,
    pub vehTurnaroundTime: c_int,

    // vehicle has weapons linked
    pub vehWeaponsLinked: qboolean,

    // when hyperspacing, you just go forward really fast for HYPERSPACE_TIME
    pub hyperSpaceTime: c_int,
    pub hyperSpaceAngles: vec3_t,

    // hacking when > time
    pub hackingTime: c_int,
    // actual hack amount - only for the proper percentage display when
    // drawing progress bar (is there a less bandwidth-eating way to do
    // this without a lot of hassle?)
    pub hackingBaseTime: c_int,

    // keeps track of jetpack fuel
    pub jetpackFuel: c_int,

    // keeps track of cloak fuel
    pub cloakFuel: c_int,

    // rww - spare values specifically for use by mod authors.
    // See psf_overrides.txt if you want to increase the send
    // amount of any of these above 1 bit.
    // (`#ifndef _XBOX`)
    pub userInt1: c_int,
    pub userInt2: c_int,
    pub userInt3: c_int,
    pub userFloat1: f32,
    pub userFloat2: f32,
    pub userFloat3: f32,
    pub userVec1: vec3_t,
    pub userVec2: vec3_t,
    // (`#ifdef _ONEBIT_COMBO` deltaOneBits/deltaNumBits omitted — undefined.)
}

const _: () = assert!(core::mem::size_of::<playerState_t>() == 1552);
const _: () = assert!(core::mem::align_of::<playerState_t>() == 4);
const _: () = assert!(core::mem::offset_of!(playerState_t, fd) == 804);
const _: () = assert!(core::mem::offset_of!(playerState_t, userVec2) == 1540);

// ===========================================================================
// Saber data (q_shared.h). `saberInfo_t` is embedded by value as the two-element
// `gclient_t::saber` array, so its layout is load-bearing for that master struct
// (the `g_local.h` unit). Everything here is pointer-free, so identical layout on
// 32- and 64-bit; sizes/offsets are oracle-verified (`oracle/q_shared_h_oracle.c`).
// ===========================================================================

/// `qhandle_t` (q_shared.h) — opaque renderer/sound handle (C `typedef int`).
pub type qhandle_t = c_int;

/// `mdxaBone_t` (q_shared.h) — a ghoul2 bone/bolt transform: a 3×4 row-major matrix
/// (`float matrix[3][4]`, the upper three rows of a 4×4 with translation in column 3).
/// Filled by `trap_G2API_GetBoltMatrix`; the bolt world position is column 3
/// (`matrix[*][3]`). Pointer-free, identical layout on 32- and 64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

// sizeof(mdxaBone_t) is 48 on the original (12 floats), align 4.
const _: () = assert!(core::mem::size_of::<mdxaBone_t>() == 48);
const _: () = assert!(core::mem::align_of::<mdxaBone_t>() == 4);

// For ghoul2 axis use — `enum Eorientations` (q_shared.h). NOTE the non-obvious
// declaration order X, Z, Y (not X, Y, Z), so the numeric values are interleaved;
// these are passed as bare ints to `trap_G2API_SetBoneAngles` / matrix accessors.
pub const ORIGIN: c_int = 0;
pub const POSITIVE_X: c_int = 1;
pub const POSITIVE_Z: c_int = 2;
pub const POSITIVE_Y: c_int = 3;
pub const NEGATIVE_X: c_int = 4;
pub const NEGATIVE_Z: c_int = 5;
pub const NEGATIVE_Y: c_int = 6;

/// `sharedRagDollUpdateParams_t` (q_shared.h) — parameters handed to
/// `trap_G2API_AnimateG2Models` to step a ghoul2 instance's animation (and ragdoll)
/// for a frame: where/how the model sits in the world plus its owner and a settle
/// frame. Pointer-free; crosses the syscall ABI by pointer, so layout is load-bearing.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct sharedRagDollUpdateParams_t {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    pub velocity: vec3_t,
    pub me: c_int,
    pub settleFrame: c_int,
}
// 4×vec3_t (48) + 2×int (8) = 56, align 4.
const _: () = assert!(core::mem::size_of::<sharedRagDollUpdateParams_t>() == 56);
const _: () = assert!(core::mem::align_of::<sharedRagDollUpdateParams_t>() == 4);

/// `sharedIKMoveParams_t` (q_shared.h) — "rww - update parms for ik bone stuff":
/// args to `trap_G2API_IKMove` asking the named bone to reach toward `desiredOrigin`
/// at `movementSpeed`. Pointer-free; crosses the syscall ABI by pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct sharedIKMoveParams_t {
    /// name of bone
    pub boneName: [c_char; 512],
    /// world coordinate that this bone should be attempting to reach
    pub desiredOrigin: vec3_t,
    /// world coordinate of the entity who owns the g2 instance that owns the bone
    pub origin: vec3_t,
    /// how fast the bone should move toward the destination
    pub movementSpeed: f32,
}
// char[512] (512) + 2×vec3_t (24) + float (4) = 540, align 4.
const _: () = assert!(core::mem::size_of::<sharedIKMoveParams_t>() == 540);
const _: () = assert!(core::mem::align_of::<sharedIKMoveParams_t>() == 4);

/// `sharedSetBoneIKStateParams_t` (q_shared.h) — args to `trap_G2API_SetBoneIKState`
/// that configure an IK (physics-controlled joint) bone: joint limits, the caller's
/// placement, the bone's base-pose frames, and blend/override controls. Pointer-free;
/// crosses the syscall ABI by pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct sharedSetBoneIKStateParams_t {
    /// ik joint limit
    pub pcjMins: vec3_t,
    /// ik joint limit
    pub pcjMaxs: vec3_t,
    /// origin of caller
    pub origin: vec3_t,
    /// angles of caller
    pub angles: vec3_t,
    /// scale of caller
    pub scale: vec3_t,
    /// bone rad
    pub radius: f32,
    /// bone blend time
    pub blendTime: c_int,
    /// override ik bone flags
    pub pcjOverrides: c_int,
    /// base pose start
    pub startFrame: c_int,
    /// base pose end
    pub endFrame: c_int,
    /// normally if the bone has specified start/end frames already it will leave it
    /// alone.. if this is true, then the animation will be restarted on the bone with
    /// the specified frames anyway.
    pub forceAnimOnBone: qboolean,
}
// 5×vec3_t (60) + float (4) + 5×int (20) = 84, align 4.
const _: () = assert!(core::mem::size_of::<sharedSetBoneIKStateParams_t>() == 84);
const _: () = assert!(core::mem::align_of::<sharedSetBoneIKStateParams_t>() == 4);

/// `enum sharedEIKMoveState` (q_shared.h) — the IK state passed to
/// `trap_G2API_SetBoneIKState`. Plain sequential C enum, passed as `int`.
pub type sharedEIKMoveState = c_int;
pub const IKS_NONE: sharedEIKMoveState = 0;
pub const IKS_DYNAMIC: sharedEIKMoveState = 1;

/// `sharedRagDollParams_t` (q_shared.h) — parameters handed to `trap_G2API_SetRagDoll`
/// to drive ragdoll physics on a ghoul2 instance: world placement, the pelvis
/// offsets the engine writes back, impact/shot strengths, the owning entity, the
/// animation frame range, the collision source, the begin/phase return controls, and
/// the effector-disable mask. Several fields are in/out (the engine writes the
/// `pelvis*Offset` and `CallRagDollBegin` back). Pointer-free; crosses the syscall ABI
/// by pointer, so layout is load-bearing.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct sharedRagDollParams_t {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    /// always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisAnglesOffset: vec3_t,
    /// always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisPositionOffset: vec3_t,
    /// should be applicable when RagPhase is RP_DEATH_COLLISION
    pub fImpactStrength: f32,
    /// should be applicable for setting velocity of corpse on shot (probably only on RP_CORPSE_SHOT)
    pub fShotStrength: f32,
    /// index of entity giving this update
    pub me: c_int,
    // rww - we have convenient animation/frame access in the game, so just send this info over from there.
    pub startFrame: c_int,
    pub endFrame: c_int,
    /// 1 = from a fall, 0 from effectors, this will be going away soon, hence no enum
    pub collisionType: c_int,
    /// a return value, means that we are now beginning ragdoll and the NPC stuff needs to happen
    pub CallRagDollBegin: qboolean,
    pub RagPhase: c_int,
    // effector control, used for RP_DISABLE_EFFECTORS call
    /// set this to an | of the RE_* flags for a RP_DISABLE_EFFECTORS
    pub effectorsToTurnOff: c_int,
}
// 5×vec3_t (60) + 2×float (8) + 7×int (28) = 96, align 4.
const _: () = assert!(core::mem::size_of::<sharedRagDollParams_t>() == 96);
const _: () = assert!(core::mem::align_of::<sharedRagDollParams_t>() == 4);

/// `MAX_TOKENLENGTH` (q_shared.h) — fixed buffer size for a parsed token string.
pub const MAX_TOKENLENGTH: usize = 1024;

/// `pc_token_t` (q_shared.h) — a parsed-out token returned by the engine-side parser
/// through `trap_PC_ReadToken` (an in/out parm). Pointer-free; crosses the syscall ABI
/// by pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct pc_token_t {
    pub r#type: c_int,
    pub subtype: c_int,
    pub intvalue: c_int,
    pub floatvalue: f32,
    pub string: [c_char; MAX_TOKENLENGTH],
}
// 4×int/float (16) + char[1024] (1024) = 1040, align 4.
const _: () = assert!(core::mem::size_of::<pc_token_t>() == 1040);
const _: () = assert!(core::mem::align_of::<pc_token_t>() == 4);

/// `qtime_t` (q_shared.h) — broken-down calendar time returned by `trap_RealTime`
/// (mirrors C `struct tm`). Pointer-free; crosses the syscall ABI by pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct qtime_t {
    /// seconds after the minute - [0,59]
    pub tm_sec: c_int,
    /// minutes after the hour - [0,59]
    pub tm_min: c_int,
    /// hours since midnight - [0,23]
    pub tm_hour: c_int,
    /// day of the month - [1,31]
    pub tm_mday: c_int,
    /// months since January - [0,11]
    pub tm_mon: c_int,
    /// years since 1900
    pub tm_year: c_int,
    /// days since Sunday - [0,6]
    pub tm_wday: c_int,
    /// days since January 1 - [0,365]
    pub tm_yday: c_int,
    /// daylight savings time flag
    pub tm_isdst: c_int,
}
const _: () = assert!(core::mem::size_of::<qtime_t>() == 36);

/// `siegePers_t` (q_shared.h) — siege persistent state carried across map loads in a
/// siege round, exchanged through `trap_SiegePersSet`/`trap_SiegePersGet`. Pointer-free;
/// crosses the syscall ABI by pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct siegePers_t {
    pub beatingTime: qboolean,
    pub lastTeam: c_int,
    pub lastTime: c_int,
}
const _: () = assert!(core::mem::size_of::<siegePers_t>() == 12);

/// `saberBlockType_t` (q_shared.h) — how wide an arc a block covers (anonymous
/// enum + `typedef int`).
pub type saberBlockType_t = c_int;
pub const BLK_NO: saberBlockType_t = 0;
/// Block only attacks and shots around the saber itself, a bbox of around 12x12x12
pub const BLK_TIGHT: saberBlockType_t = 1;
/// Block all attacks in an area around the player in a rough arc of 180 degrees
pub const BLK_WIDE: saberBlockType_t = 2;

/// `saberBlockedType_t` (q_shared.h) — which direction/quadrant a block came
/// from (anonymous enum + `typedef int`). Consumed by `BG_KnockawayForParry`
/// (bg_panimate.c).
pub type saberBlockedType_t = c_int;
pub const BLOCKED_NONE: saberBlockedType_t = 0;
pub const BLOCKED_BOUNCE_MOVE: saberBlockedType_t = 1;
pub const BLOCKED_PARRY_BROKEN: saberBlockedType_t = 2;
pub const BLOCKED_ATK_BOUNCE: saberBlockedType_t = 3;
pub const BLOCKED_UPPER_RIGHT: saberBlockedType_t = 4;
pub const BLOCKED_UPPER_LEFT: saberBlockedType_t = 5;
pub const BLOCKED_LOWER_RIGHT: saberBlockedType_t = 6;
pub const BLOCKED_LOWER_LEFT: saberBlockedType_t = 7;
pub const BLOCKED_TOP: saberBlockedType_t = 8;
pub const BLOCKED_UPPER_RIGHT_PROJ: saberBlockedType_t = 9;
pub const BLOCKED_UPPER_LEFT_PROJ: saberBlockedType_t = 10;
pub const BLOCKED_LOWER_RIGHT_PROJ: saberBlockedType_t = 11;
pub const BLOCKED_LOWER_LEFT_PROJ: saberBlockedType_t = 12;
pub const BLOCKED_TOP_PROJ: saberBlockedType_t = 13;

/// `saber_colors_t` (q_shared.h) — blade color (anonymous enum + `typedef int`).
pub type saber_colors_t = c_int;
pub const SABER_RED: saber_colors_t = 0;
pub const SABER_ORANGE: saber_colors_t = 1;
pub const SABER_YELLOW: saber_colors_t = 2;
pub const SABER_GREEN: saber_colors_t = 3;
pub const SABER_BLUE: saber_colors_t = 4;
pub const SABER_PURPLE: saber_colors_t = 5;
pub const NUM_SABER_COLORS: saber_colors_t = 6;

/// `saberType_t` (q_shared.h) — hilt type.
pub type saberType_t = c_int;
pub const SABER_NONE: saberType_t = 0;
pub const SABER_SINGLE: saberType_t = 1;
pub const SABER_STAFF: saberType_t = 2;
pub const SABER_DAGGER: saberType_t = 3;
pub const SABER_BROAD: saberType_t = 4;
pub const SABER_PRONG: saberType_t = 5;
pub const SABER_ARC: saberType_t = 6;
pub const SABER_SAI: saberType_t = 7;
pub const SABER_CLAW: saberType_t = 8;
pub const SABER_LANCE: saberType_t = 9;
pub const SABER_STAR: saberType_t = 10;
pub const SABER_TRIDENT: saberType_t = 11;
pub const SABER_SITH_SWORD: saberType_t = 12;
pub const NUM_SABERS: saberType_t = 13;

/// `saber_styles_t` (q_shared.h) — locked attack style.
pub type saber_styles_t = c_int;
pub const SS_NONE: saber_styles_t = 0;
pub const SS_FAST: saber_styles_t = 1;
pub const SS_MEDIUM: saber_styles_t = 2;
pub const SS_STRONG: saber_styles_t = 3;
pub const SS_DESANN: saber_styles_t = 4;
pub const SS_TAVION: saber_styles_t = 5;
pub const SS_DUAL: saber_styles_t = 6;
pub const SS_STAFF: saber_styles_t = 7;
pub const SS_NUM_SABER_STYLES: saber_styles_t = 8;

// ===========================================================================
// SABER FLAGS (q_shared.h) — old `qboolean` saber bools converted to a flag
// bitfield (`saberInfo_t::saberFlags`). The first 7 are the PC migration of the
// Xbox bool fields; the rest are PC-new move/behavior restrictions.
// ===========================================================================
pub const SFL_NOT_LOCKABLE: c_int = 1 << 0; // can't get into a saberlock
pub const SFL_NOT_THROWABLE: c_int = 1 << 1; // can't be thrown
pub const SFL_NOT_DISARMABLE: c_int = 1 << 2; // can't be dropped
pub const SFL_NOT_ACTIVE_BLOCKING: c_int = 1 << 3; // don't try to block incoming shots with this saber
pub const SFL_TWO_HANDED: c_int = 1 << 4; // uses both hands
pub const SFL_SINGLE_BLADE_THROWABLE: c_int = 1 << 5; // can throw if only the first blade is on
pub const SFL_RETURN_DAMAGE: c_int = 1 << 6; // keeps spinning and doing damage on return
                                             // NEW FLAGS
pub const SFL_ON_IN_WATER: c_int = 1 << 7; // weapon stays active even in water
pub const SFL_BOUNCE_ON_WALLS: c_int = 1 << 8; // saber bounces back when it hits solid architecture
pub const SFL_BOLT_TO_WRIST: c_int = 1 << 9; // saber model is bolted to wrist, not in hand
                                             // Move Restrictions
pub const SFL_NO_PULL_ATTACK: c_int = 1 << 10; // cannot do pull+attack move
pub const SFL_NO_BACK_ATTACK: c_int = 1 << 11; // cannot do back-stab moves
pub const SFL_NO_STABDOWN: c_int = 1 << 12; // cannot do stabdown move
pub const SFL_NO_WALL_RUNS: c_int = 1 << 13; // cannot side-run or forward-run on walls
pub const SFL_NO_WALL_FLIPS: c_int = 1 << 14; // cannot do backflip/side-flips off walls
pub const SFL_NO_WALL_GRAB: c_int = 1 << 15; // cannot grab wall & jump off
pub const SFL_NO_ROLLS: c_int = 1 << 16; // cannot roll
pub const SFL_NO_FLIPS: c_int = 1 << 17; // cannot do flips
pub const SFL_NO_CARTWHEELS: c_int = 1 << 18; // cannot do cartwheels
pub const SFL_NO_KICKS: c_int = 1 << 19; // cannot do kicks
pub const SFL_NO_MIRROR_ATTACKS: c_int = 1 << 20; // cannot do simultaneous attack left/right moves
pub const SFL_NO_ROLL_STAB: c_int = 1 << 21; // cannot do roll-stab move at end of roll

// SABER FLAGS2 (q_shared.h) — `saberInfo_t::saberFlags2`. Primary-blade style
// flags (bits 0-8) plus their `*2` secondary-blade variants (bits 9-17).
// Primary Blade Style
pub const SFL2_NO_WALL_MARKS: c_int = 1 << 0; // stops the saber from drawing marks on the world
pub const SFL2_NO_DLIGHT: c_int = 1 << 1; // stops the saber from drawing a dynamic light
pub const SFL2_NO_BLADE: c_int = 1 << 2; // stops the saber from drawing a blade
pub const SFL2_NO_CLASH_FLARE: c_int = 1 << 3; // no big white clash flare with other sabers
pub const SFL2_NO_DISMEMBERMENT: c_int = 1 << 4; // never does dismemberment
pub const SFL2_NO_IDLE_EFFECT: c_int = 1 << 5; // no damage/effects when idle
pub const SFL2_ALWAYS_BLOCK: c_int = 1 << 6; // blades will always be blocking
pub const SFL2_NO_MANUAL_DEACTIVATE: c_int = 1 << 7; // blades cannot be manually toggled on/off
pub const SFL2_TRANSITION_DAMAGE: c_int = 1 << 8; // blade does damage in start/transition/return anims
                                                  // Secondary Blade Style
pub const SFL2_NO_WALL_MARKS2: c_int = 1 << 9;
pub const SFL2_NO_DLIGHT2: c_int = 1 << 10;
pub const SFL2_NO_BLADE2: c_int = 1 << 11;
pub const SFL2_NO_CLASH_FLARE2: c_int = 1 << 12;
pub const SFL2_NO_DISMEMBERMENT2: c_int = 1 << 13;
pub const SFL2_NO_IDLE_EFFECT2: c_int = 1 << 14;
pub const SFL2_ALWAYS_BLOCK2: c_int = 1 << 15;
pub const SFL2_NO_MANUAL_DEACTIVATE2: c_int = 1 << 16;
pub const SFL2_TRANSITION_DAMAGE2: c_int = 1 << 17;

/// `MAX_BLADES` (q_shared.h) — blades per `saberInfo_t`.
pub const MAX_BLADES: usize = 8;
/// `MAX_SABERS` (q_shared.h) — sabers carried per client.
pub const MAX_SABERS: usize = 2;

// ===========================================================================
// Force-power levels (q_shared.h, the anonymous enum immediately after
// `MAX_SABERS`). These index the `forceJumpHeight`/`forceJumpStrength`/
// `forcePushPullRadius`/`forcePowerNeeded` tables (defined in bg_pmove.c,
// declared in w_saber.h / bg_local.h) and `playerState_t::forcePowerLevel[]`.
// The C `typedef enum {…};` has no tag/typedef name — so, like `forcePowers_t`
// (FP_*) above, the enumerators are plain `int` constants. `NUM_FORCE_POWER_LEVELS`
// is `usize` because it dimensions those arrays (matching `NUM_FORCE_POWERS`).
// `FORCE_LEVEL_4`/`_5` are the two `#define`s that follow (one/two past the max).
// ===========================================================================

pub const FORCE_LEVEL_0: c_int = 0;
pub const FORCE_LEVEL_1: c_int = 1;
pub const FORCE_LEVEL_2: c_int = 2;
pub const FORCE_LEVEL_3: c_int = 3;
/// `NUM_FORCE_POWER_LEVELS` — count of the base `FORCE_LEVEL_*` levels (4).
pub const NUM_FORCE_POWER_LEVELS: usize = 4;

/// `FORCE_LEVEL_4` = `FORCE_LEVEL_3 + 1` (4) — one past the normal max.
pub const FORCE_LEVEL_4: c_int = FORCE_LEVEL_3 + 1;
/// `FORCE_LEVEL_5` = `FORCE_LEVEL_4 + 1` (5).
pub const FORCE_LEVEL_5: c_int = FORCE_LEVEL_4 + 1;

/// `saberTrail_t` (q_shared.h) — a blade's motion trail + wall-mark state.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct saberTrail_t {
    // Actual trail stuff
    pub inAction: c_int, // controls whether should we even consider starting one
    pub duration: c_int, // how long each trail seg stays in existence
    pub lastTime: c_int, // time a saber segement was last stored
    pub base: vec3_t,
    pub tip: vec3_t,

    pub dualbase: vec3_t,
    pub dualtip: vec3_t,

    // Marks stuff
    pub haveOldPos: [qboolean; 2],
    pub oldPos: [vec3_t; 2],
    pub oldNormal: [vec3_t; 2], // store this in case we don't have a connect-the-dots situation
                                //	..then we'll need the normal to project a mark blob onto the impact point
}
const _: () = assert!(core::mem::size_of::<saberTrail_t>() == 116);
const _: () = assert!(core::mem::align_of::<saberTrail_t>() == 4);
const _: () = assert!(core::mem::offset_of!(saberTrail_t, oldPos) == 68);
const _: () = assert!(core::mem::offset_of!(saberTrail_t, oldNormal) == 92);

/// `bladeInfo_t` (q_shared.h) — per-blade runtime state. Embeds a single
/// `saberTrail_t` (PC layout; the Xbox build had `saberTrail_t trail[2]`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct bladeInfo_t {
    pub active: qboolean,
    pub color: saber_colors_t,
    pub radius: f32,
    pub length: f32,
    pub lengthMax: f32,
    pub lengthOld: f32,
    pub desiredLength: f32,
    pub muzzlePoint: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzleDirOld: vec3_t,
    pub trail: saberTrail_t,
    pub hitWallDebounceTime: c_int,
    pub storageTime: c_int,
    pub extendDebounce: c_int,
}
const _: () = assert!(core::mem::size_of::<bladeInfo_t>() == 204);
const _: () = assert!(core::mem::align_of::<bladeInfo_t>() == 4);
const _: () = assert!(core::mem::offset_of!(bladeInfo_t, trail) == 76);
const _: () = assert!(core::mem::offset_of!(bladeInfo_t, hitWallDebounceTime) == 192);

/// `saberInfo_t` (q_shared.h) — a full saber definition (one entry in sabers.cfg).
/// Embedded by value as `gclient_t::saber[MAX_SABERS]`, so its layout is
/// load-bearing. Pointer-free; identical on 32/64-bit. `type` is a Rust keyword,
/// hence `r#type` (the C field name is `type`).
///
/// PC layout: the Xbox `qboolean` bools (lockable/throwable/twoHanded/… and the
/// MP-only noWallMarks/noIdleEffect block) collapse into the two `saberFlags`/
/// `saberFlags2` bitfields (see SFL_/SFL2_ above), `style` becomes the
/// `stylesLearned`/`stylesForbidden` pair, and a large block of PC move/anim
/// overrides + a full secondary-blade style section is appended.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct saberInfo_t {
    pub name: [c_char; 64],         // entry in sabers.cfg, if any
    pub fullName: [c_char; 64],     // the "Proper Name" of the saber, shown in UI
    pub r#type: saberType_t,        // none, single or staff
    pub model: [c_char; MAX_QPATH], // hilt model
    pub skin: qhandle_t,            // registered skin id
    pub soundOn: c_int,             // game soundindex for turning on sound
    pub soundLoop: c_int,           // game soundindex for hum/loop sound
    pub soundOff: c_int,            // game soundindex for turning off sound
    pub numBlades: c_int,
    pub blade: [bladeInfo_t; MAX_BLADES], // blade info - like length, trail, origin, dir, etc.
    pub stylesLearned: c_int,             // styles you get when you get this saber, if any
    pub stylesForbidden: c_int,           // styles you cannot use with this saber, if any
    pub maxChain: c_int, // how many moves can be chained in a row with this weapon (-1 is infinite, 0 is use default behavior)
    pub forceRestrictions: c_int, // force powers that cannot be used while this saber is on (bitfield) - FIXME: maybe make this a limit on the max level, per force power, that can be used with this type?
    pub lockBonus: c_int,         // in saberlocks, this type of saber pushes harder or weaker
    pub parryBonus: c_int,        // added to strength of parry with this saber
    pub breakParryBonus: c_int,   // added to strength when hit a parry
    pub breakParryBonus2: c_int,  // for bladeStyle2 (see bladeStyle2Start below)
    pub disarmBonus: c_int, // added to disarm chance when win saberlock or have a good parry (knockaway)
    pub disarmBonus2: c_int, // for bladeStyle2 (see bladeStyle2Start below)
    pub singleBladeStyle: saber_styles_t, // makes it so that you use a different style if you only have the first blade active
    //===NEW========================================================================================
    // these values are global to the saber, like all of the ones above
    pub saberFlags: c_int,  // from SFL_ list above
    pub saberFlags2: c_int, // from SFL2_ list above

    // done in cgame (client-side code)
    pub spinSound: qhandle_t, // none - if set, plays this sound as it spins when thrown
    pub swingSound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when swung during an attack - NOTE: must provide all 3!!!

    // done in game (server-side code)
    pub moveSpeedScale: f32, // 1.0 - you move faster/slower when using this saber
    pub animSpeedScale: f32, // 1.0 - plays normal attack animations faster/slower

    // done in both cgame and game (BG code)
    pub kataMove: c_int, // LS_INVALID - if set, player will execute this move when they press both attack buttons at the same time
    pub lungeAtkMove: c_int, // LS_INVALID - if set, player will execute this move when they crouch+fwd+attack
    pub jumpAtkUpMove: c_int, // LS_INVALID - if set, player will execute this move when they jump+attack
    pub jumpAtkFwdMove: c_int, // LS_INVALID - if set, player will execute this move when they jump+fwd+attack
    pub jumpAtkBackMove: c_int, // LS_INVALID - if set, player will execute this move when they jump+back+attack
    pub jumpAtkRightMove: c_int, // LS_INVALID - if set, player will execute this move when they jump+rightattack
    pub jumpAtkLeftMove: c_int, // LS_INVALID - if set, player will execute this move when they jump+left+attack
    pub readyAnim: c_int,       // -1 - anim to use when standing idle
    pub drawAnim: c_int,        // -1 - anim to use when drawing weapon
    pub putawayAnim: c_int,     // -1 - anim to use when putting weapon away
    pub tauntAnim: c_int,       // -1 - anim to use when hit "taunt"
    pub bowAnim: c_int,         // -1 - anim to use when hit "bow"
    pub meditateAnim: c_int,    // -1 - anim to use when hit "meditate"
    pub flourishAnim: c_int,    // -1 - anim to use when hit "flourish"
    pub gloatAnim: c_int,       // -1 - anim to use when hit "gloat"

    //***NOTE: you can only have a maximum of 2 "styles" of blades, so this next value, "bladeStyle2Start" is the number of the first blade to use these value on... all blades before this use the normal values above, all blades at and after this number use the secondary values below***
    pub bladeStyle2Start: c_int, // 0 - if set, blades from this number and higher use the following values

    //***The following can be different for the extra blades - not setting them individually defaults them to the value for the whole saber (and first blade)***

    //===PRIMARY BLADES=====================
    // done in cgame (client-side code)
    pub trailStyle: c_int, // 0 - default (0) is normal, 1 is a motion blur and 2 is no trail at all
    pub g2MarksShader: c_int, // none - if set, the game will use this shader for marks on enemies
    pub g2WeaponMarkShader: c_int, // none - if set, projects this shader onto the weapon when it damages a person
    pub hitSound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber hits a person - NOTE: must provide all 3!!!
    pub blockSound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber/sword hits another saber/sword - NOTE: must provide all 3!!!
    pub bounceSound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber/sword hits a wall and bounces off - NOTE: must provide all 3!!!
    pub blockEffect: c_int, // none - if set, plays this effect when the saber/sword hits another saber/sword
    pub hitPersonEffect: c_int, // none - if set, plays this effect when the saber/sword hits a person
    pub hitOtherEffect: c_int, // none - if set, plays this effect when the saber/sword hits something else damagable
    pub bladeEffect: c_int,    // none - if set, plays this effect at the blade tag

    // done in game (server-side code)
    pub knockbackScale: f32, // 0 - if non-zero, uses damage done to calculate an appropriate amount of knockback
    pub damageScale: f32,    // 1 - scale up or down the damage done by the saber
    pub splashRadius: f32,   // 0 - radius of splashDamage
    pub splashDamage: c_int, // 0 - amount of splashDamage
    pub splashKnockback: f32, // 0 - amount of splashKnockback

    //===SECONDARY BLADES===================
    // done in cgame (client-side code)
    pub trailStyle2: c_int, // 0 - default (0) is normal, 1 is a motion blur and 2 is no trail at all
    pub g2MarksShader2: c_int, // none - if set, the game will use this shader for marks on enemies
    pub g2WeaponMarkShader2: c_int, // none - if set, projects this shader onto the weapon when it damages a person
    pub hit2Sound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber hits a person - NOTE: must provide all 3!!!
    pub block2Sound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber/sword hits another saber/sword - NOTE: must provide all 3!!!
    pub bounce2Sound: [qhandle_t; 3], // none - if set, plays one of these 3 sounds when saber/sword hits a wall and bounces off - NOTE: must provide all 3!!!
    pub blockEffect2: c_int, // none - if set, plays this effect when the saber/sword hits another saber/sword
    pub hitPersonEffect2: c_int, // none - if set, plays this effect when the saber/sword hits a person
    pub hitOtherEffect2: c_int, // none - if set, plays this effect when the saber/sword hits something else damagable
    pub bladeEffect2: c_int,    // none - if set, plays this effect at the blade tag

    // done in game (server-side code)
    pub knockbackScale2: f32, // 0 - if non-zero, uses damage done to calculate an appropriate amount of knockback
    pub damageScale2: f32,    // 1 - scale up or down the damage done by the saber
    pub splashRadius2: f32,   // 0 - radius of splashDamage
    pub splashDamage2: c_int, // 0 - amount of splashDamage
    pub splashKnockback2: f32, // 0 - amount of splashKnockback
}
const _: () = assert!(core::mem::size_of::<saberInfo_t>() == 2156);
const _: () = assert!(core::mem::align_of::<saberInfo_t>() == 4);
const _: () = assert!(core::mem::offset_of!(saberInfo_t, blade) == 216);
const _: () = assert!(core::mem::offset_of!(saberInfo_t, saberFlags) == 1892);
const _: () = assert!(core::mem::offset_of!(saberInfo_t, swingSound) == 1904);
const _: () = assert!(core::mem::offset_of!(saberInfo_t, knockbackScale) == 2052);
const _: () = assert!(core::mem::offset_of!(saberInfo_t, splashKnockback2) == 2152);

// ===========================================================================
// Material (q_shared.h `material_e` + `typedef int material_t`). Embedded by
// value in `gentity_t::material`. "material stuff needs to be shared."
// ===========================================================================

/// `material_t` (q_shared.h) — chunk/impact material type (anonymous enum +
/// `typedef int`).
pub type material_t = c_int;
pub const MAT_METAL: material_t = 0; // scorched blue-grey metal
pub const MAT_GLASS: material_t = 1; // not a real chunk type, just plays an effect with glass sprites
pub const MAT_ELECTRICAL: material_t = 2; // sparks only
pub const MAT_ELEC_METAL: material_t = 3; // sparks/electrical type metal
pub const MAT_DRK_STONE: material_t = 4; // brown
pub const MAT_LT_STONE: material_t = 5; // tan
pub const MAT_GLASS_METAL: material_t = 6; // glass sprites and METAl chunk
pub const MAT_METAL2: material_t = 7; // electrical metal type
pub const MAT_NONE: material_t = 8; // no chunks
pub const MAT_GREY_STONE: material_t = 9; // grey
pub const MAT_METAL3: material_t = 10; // METAL and METAL2 chunks
pub const MAT_CRATE1: material_t = 11; // yellow multi-colored crate chunks
pub const MAT_GRATE1: material_t = 12; // grate chunks
pub const MAT_ROPE: material_t = 13; // for yavin trial...no chunks, just wispy bits
pub const MAT_CRATE2: material_t = 14; // read multi-colored crate chunks
pub const MAT_WHITE_METAL: material_t = 15; // white angular chunks
pub const MAT_SNOWY_ROCK: material_t = 16; // gray & brown chunks
pub const NUM_MATERIALS: material_t = 17;
const _: () = assert!(core::mem::size_of::<material_t>() == 4);

// ===========================================================================
// Cvar flags (q_shared.h) — bit flags OR'd into the engine cvar system's `int`
// flags field (and into `cvarTable_t::cvarFlags`). Plain explicit-value
// `#define`s, carried verbatim with their comments; like the other `#define`
// integer constants in this file they need no C-oracle (no auto-numbering or
// layout to verify — the value is the literal).
// ===========================================================================

/// `CVAR_ARCHIVE` — set to cause it to be saved to vars.rc. Used for system
/// variables, not for player specific configurations.
pub const CVAR_ARCHIVE: c_int = 0x00000001;
/// `CVAR_USERINFO` — sent to server on connect or change.
pub const CVAR_USERINFO: c_int = 0x00000002;
/// `CVAR_SERVERINFO` — sent in response to front end requests.
pub const CVAR_SERVERINFO: c_int = 0x00000004;
/// `CVAR_SYSTEMINFO` — these cvars will be duplicated on all clients.
pub const CVAR_SYSTEMINFO: c_int = 0x00000008;
/// `CVAR_INIT` — don't allow change from console at all, but can be set from
/// the command line.
pub const CVAR_INIT: c_int = 0x00000010;
/// `CVAR_LATCH` — will only change when C code next does a Cvar_Get(), so it
/// can't be changed without proper initialization. modified will be set, even
/// though the value hasn't changed yet.
pub const CVAR_LATCH: c_int = 0x00000020;
/// `CVAR_ROM` — display only, cannot be set by user at all (can be set by code).
pub const CVAR_ROM: c_int = 0x00000040;
/// `CVAR_USER_CREATED` — created by a set command.
pub const CVAR_USER_CREATED: c_int = 0x00000080;
/// `CVAR_TEMP` — can be set even when cheats are disabled, but is not archived.
pub const CVAR_TEMP: c_int = 0x00000100;
/// `CVAR_CHEAT` — can not be changed if cheats are disabled.
pub const CVAR_CHEAT: c_int = 0x00000200;
/// `CVAR_NORESTART` — do not clear when a cvar_restart is issued.
pub const CVAR_NORESTART: c_int = 0x00000400;
/// `CVAR_INTERNAL` — cvar won't be displayed, ever (for passwords and such).
pub const CVAR_INTERNAL: c_int = 0x00000800;
/// `CVAR_PARENTAL` — lets cvar system know that parental stuff needs to be updated.
pub const CVAR_PARENTAL: c_int = 0x00001000;
//JLF
/// `CVAR_PROFILE` — used to mark profile cvars.
pub const CVAR_PROFILE: c_int = 0x00002000;

// ===========================================================================
// fsMode_t (q_shared.h) — the file-open mode passed to `trap_FS_FOpenFile`. A
// plain sequential C enum; like the other q_shared.h enums it is aliased to
// `c_int` with explicit-value consts (the values are the literal enumerator
// order, so there is nothing to oracle-verify).
// ===========================================================================

pub const FS_READ: c_int = 0;
pub const FS_WRITE: c_int = 1;
pub const FS_APPEND: c_int = 2;
pub const FS_APPEND_SYNC: c_int = 3;
/// `fsMode_t` is a C enum, passed as a plain `int` across the syscall ABI.
pub type fsMode_t = c_int;

// ===========================================================================
// Waypoint navigation objects (q_shared.h:897-925) — the bot-AI waypoint graph
// node (`wpobject_t`) and its neighbor link (`wpneighbor_t`). Used by the bot
// state (`bot_state_t`) and the waypoint nav globals (`gWPArray`, ...).
// ===========================================================================

pub const MAX_WPARRAY_SIZE: usize = 4096;
pub const MAX_NEIGHBOR_SIZE: usize = 32;

pub const MAX_NEIGHBOR_LINK_DISTANCE: i32 = 128;
pub const MAX_NEIGHBOR_FORCEJUMP_LINK_DISTANCE: i32 = 400;

pub const DEFAULT_GRID_SPACING: i32 = 400;

/// `wpneighbor_t` (q_shared.h:905) — a link from one waypoint to a neighbor.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct wpneighbor_t {
    pub num: c_int,
    pub forceJumpTo: c_int,
}

/// `wpobject_t` (q_shared.h:911) — a single waypoint navigation node.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct wpobject_t {
    pub origin: vec3_t,
    pub inuse: c_int,
    pub index: c_int,
    pub weight: f32,
    pub disttonext: f32,
    pub flags: c_int,
    pub associated_entity: c_int,

    pub forceJumpTo: c_int,

    pub neighbornum: c_int,
    pub neighbors: [wpneighbor_t; MAX_NEIGHBOR_SIZE],
}

// ===========================================================================
// 1:1-completeness ledger: q_shared.h static-inline / macro math.
//
// `ported_index.py` reports the following 28 q_shared.h symbols as "missing".
// They are FALSE POSITIVES: each is a header `static ID_INLINE` function, a
// preprocessor macro, or a platform-specific intrinsic. None is a standalone
// translation unit — in C they are header-inlined into every consumer — so the
// Rust port lands them as ordinary `fn`s, almost all already live in `q_math.rs`
// (a few byte-swap ones in `q_shared.rs`). This block documents what each C
// definition is, WHY the index flags it, and WHERE the live port lives. Every
// line below is a comment; this section adds ZERO live code.
//
// ---------------------------------------------------------------------------
// Byte-order swap wrappers (q_shared.h, per-platform `#ifdef` blocks).
//
// On a little-endian target (`__linux__ && !idppc`, q_shared.h:289-302 — the
// build ABI) the macros resolve so that `Big*` swap and `Little*` are no-ops:
//
//     inline static short BigShort( short l) { return ShortSwap(l); }   // :290
//     #define LittleShort                                               // :291
//     inline static int   BigLong(int l)     { return LongSwap(l); }    // :292
//     #define LittleLong                                                // :293
//     inline static float BigFloat(const float *l){ return FloatSwap(l); } // :294
//     #define LittleFloat                                               // :295
//     #define BigShort                                                  // :297
//     inline static short LittleShort(short l){ return ShortSwap(l); }  // :298
//     #define BigLong                                                   // :300
//     inline static int   LittleLong (int l) { return LongSwap(l); }    // :301
//     #define BigFloat                                                  // :302
//     inline static float LittleFloat (const float *l){ return FloatSwap(l); } // :302
//
// (The Win32/NDEBUG block q_shared.h:171-176 is the mirror image — there `Big*`
// is the no-op and `Little*` does the swap. We target the little-endian module,
// so the swapping half is `Big*`.) These wrappers are thin shims over the
// concrete `ShortSwap`/`LongSwap`/`FloatSwap` routines, which are ported live as:
//   - ShortSwap  -> q_shared.rs:265  pub fn ShortSwap(l: i16) -> i16
//   - LongSwap   -> q_shared.rs:278  pub fn LongSwap(l: c_int) -> c_int
//   - FloatSwap  -> q_shared.rs:319  pub unsafe fn FloatSwap(f: *const f32) -> f32
// so the `Big*`/`Little*` names need no separate definition.
//
//   BigShort  (swaps on LE)   ==  ShortSwap(l)
//   pub fn BigShort(l: i16) -> i16 { ShortSwap(l) }
//   BigLong   (swaps on LE)   ==  LongSwap(l)
//   pub fn BigLong(l: c_int) -> c_int { LongSwap(l) }
//   BigFloat  (swaps on LE)   ==  FloatSwap(l)
//   pub unsafe fn BigFloat(l: *const f32) -> f32 { FloatSwap(l) }
//   LittleShort (no-op on LE) ==  identity
//   pub fn LittleShort(l: i16) -> i16 { l }
//   LittleLong  (no-op on LE) ==  identity
//   pub fn LittleLong(l: c_int) -> c_int { l }
//   LittleFloat (no-op on LE) ==  identity (deref)
//   pub unsafe fn LittleFloat(l: *const f32) -> f32 { *l }
//
// ---------------------------------------------------------------------------
// Q_Cast{Short,UShort}2Float[Scale] (q_shared.h:1091-1109, `#ifdef _XBOX` only).
//
// These four exist only in the `_XBOX` build and are absent from the PC ABI we
// target, so there is no live port — they would be dead code. Faithful bodies:
//
//   inline void Q_CastShort2Float(float *f, const short *s) { *f = (float)*s; }
//   pub unsafe fn Q_CastShort2Float(f: *mut f32, s: *const i16) { *f = *s as f32; }
//
//   inline void Q_CastUShort2Float(float *f, const unsigned short *s) { *f = (float)*s; }
//   pub unsafe fn Q_CastUShort2Float(f: *mut f32, s: *const u16) { *f = *s as f32; }
//
//   inline void Q_CastShort2FloatScale(float *f, const short *s, float scale) { *f = (float)*s * scale; }
//   pub unsafe fn Q_CastShort2FloatScale(f: *mut f32, s: *const i16, scale: f32) { *f = (*s as f32) * scale; }
//
//   inline void Q_CastUShort2FloatScale(float *f, const unsigned short *s, float scale) { *f = (float)*s * scale; }
//   pub unsafe fn Q_CastUShort2FloatScale(f: *mut f32, s: *const u16, scale: f32) { *f = (*s as f32) * scale; }
//
// ---------------------------------------------------------------------------
// Q_fabs / Q_rsqrt (q_shared.h:1114-1139).
//
// On `idppc` these are PPC asm inlines; on the PC ABI (the `#else`, :1137-1138)
// they are forward-declared and DEFINED in q_math.c. Ported live as:
//   - Q_rsqrt -> q_math.rs:511  pub fn Q_rsqrt(number: f32) -> f32
//   - Q_fabs  -> q_math.rs:533  pub fn Q_fabs(f: f32) -> f32
// The `idppc` header-inline forms (q_shared.h:1114-1131) are:
//   static inline float Q_rsqrt(float number) {
//       float x = 0.5f * number, y;
//       asm("frsqrte %0,%1" : "=f"(y) : "f"(number));   // or __frsqrte(number)
//       return y * (1.5f - (x * y * y));
//   }
//   pub fn Q_rsqrt_idppc(number: f32) -> f32 {
//       let x = 0.5f32 * number;
//       let y = /* frsqrte estimate of 1/sqrt(number) */ 0.0f32;
//       y * (1.5f32 - (x * y * y))
//   }
//   static inline float Q_fabs(float x) { asm("fabs %0,%1" : "=f"(abs_x) : "f"(x)); return abs_x; }
//   pub fn Q_fabs_idppc(x: f32) -> f32 { x.abs() }
//
// ---------------------------------------------------------------------------
// Vector math: the `static ID_INLINE` family (q_shared.h:1354-1458) plus the
// `_XBOX` SSE DotProduct/VectorAdd/VectorSubtract/VectorScale (:1154-1253) and
// the non-XBOX macro forms (:1262-1267). All ported live in q_math.rs:
//
//   DotProduct        (#define :1262 / inline :1154) -> q_math.rs:157
//   pub fn DotProduct(x,y) -> vec_t { x[0]*y[0] + x[1]*y[1] + x[2]*y[2] }
//
//   VectorSubtract    (#define :1263 / inline :1184) -> q_math.rs:162
//   pub fn VectorSubtract(a,b,c) { c[i] = a[i] - b[i]; }
//
//   VectorAdd         (#define :1264 / inline :1208) -> q_math.rs:169
//   pub fn VectorAdd(a,b,c) { c[i] = a[i] + b[i]; }
//
//   VectorScale       (#define :1265 / inline :1232) -> q_math.rs:176
//   pub fn VectorScale(v,s,o) { o[i] = v[i] * s; }
//
//   CrossProduct      (inline :1454) -> q_math.rs:197
//   pub fn CrossProduct(v1,v2,cross) {
//       cross[0] = v1[1]*v2[2] - v1[2]*v2[1];
//       cross[1] = v1[2]*v2[0] - v1[0]*v2[2];
//       cross[2] = v1[0]*v2[1] - v1[1]*v2[0];
//   }
//
//   VectorInverse     (inline :1448) -> q_math.rs:204
//   pub fn VectorInverse(v) { v[i] = -v[i]; }
//
//   VectorLength      (inline :1361) -> q_math.rs:232
//   pub fn VectorLength(v) -> vec_t { sqrt(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]) }
//
//   VectorLengthSquared (inline :1392) -> q_math.rs:237
//   pub fn VectorLengthSquared(v) -> vec_t { v[0]*v[0] + v[1]*v[1] + v[2]*v[2] }
//
//   VectorCompare     (inline :1354) -> q_math.rs:242
//   pub fn VectorCompare(v1,v2) -> c_int {
//       if v1[0]!=v2[0] || v1[1]!=v2[1] || v1[2]!=v2[2] { return 0; } 1
//   }
//
//   Distance          (inline :1421) -> q_math.rs:250
//   pub fn Distance(p1,p2) -> vec_t { VectorSubtract(p2,p1,&mut v); VectorLength(&v) }
//
//   DistanceSquared   (inline :1428) -> q_math.rs:257
//   pub fn DistanceSquared(p1,p2) -> vec_t {
//       VectorSubtract(p2,p1,&mut v); v[0]*v[0] + v[1]*v[1] + v[2]*v[2]
//   }
//
//   VectorNormalizeFast (inline :1437) -> q_math.rs:265
//   pub fn VectorNormalizeFast(v) {
//       let ilength = Q_rsqrt(DotProduct(v,v)); v[i] *= ilength;
//   }
//
// ---------------------------------------------------------------------------
// SnapVector (q_shared.h:1310-1332). The retail `__asm fld/fistp` inline form is
// already ported live (the macro/inline local, NOT trap_SnapVector) as
//   q_shared_h.rs:228  pub(crate) fn snap_vector(v: &mut vec3_t)
// using `round_ties_even` to match x87 round-to-nearest. The LCC/VM truncating
// `#define SnapVector(v) {v[0]=((int)(v[0]));...}` variant (:1331) is not the
// ABI target. Faithful body of the inline (:1310):
//   static ID_INLINE void SnapVector(float *v) {
//       static int i; static float f;
//       f = *v; __asm fld f; __asm fistp i; *v = i; v++;   // x3
//   }
//
// ---------------------------------------------------------------------------
// PPC byte-swap / convert intrinsics (q_shared.h:205-225, `#if defined(MACOS_X)`
// && `__ppc__` only). Not in the PC ABI — no live port; documented faithfully.
//
//   static inline unsigned int __lwbrx(void *addr, int offset) {  // :205
//       unsigned int word; asm("lwbrx %0,%2,%1" : "=r"(word) : "r"(addr),"b"(offset)); return word;
//   }
//   // load word byte-reversed indexed: read the 32-bit word at addr+offset and
//   // return it with bytes reversed (big<->little).
//   pub unsafe fn __lwbrx(addr: *const u8, offset: i32) -> u32 {
//       u32::from_le_bytes(*(addr.offset(offset as isize) as *const [u8; 4]))
//   }
//
//   static inline unsigned short __lhbrx(void *addr, int offset) {  // :212
//       unsigned short halfword; asm("lhbrx %0,%2,%1" : "=r"(halfword) : "r"(addr),"b"(offset)); return halfword;
//   }
//   // load halfword byte-reversed indexed.
//   pub unsafe fn __lhbrx(addr: *const u8, offset: i32) -> u16 {
//       u16::from_le_bytes(*(addr.offset(offset as isize) as *const [u8; 2]))
//   }
//
//   static inline float __fctiw(float f) {  // :219
//       float fi; asm("fctiw %0,%1" : "=f"(fi) : "f"(f)); return fi;
//   }
//   // float convert to integer word (round to nearest, result kept in an fp reg).
//   pub fn __fctiw(f: f32) -> f32 { f.round_ties_even() }
//
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deg_rad_roundtrip() {
        let d = 90.0_f32;
        let back = RAD2DEG(DEG2RAD(d));
        assert!((back - d).abs() < 1e-3);
    }

    /// Parity: the Rust master networked structs match the authentic C
    /// `sizeof`/`offsetof` bit-for-bit on the build arch (oracle TU
    /// `q_shared_h_oracle.c`). The compile-time `const _` asserts above already
    /// pin total size + a couple of offsets; this ties those literals to the
    /// real C compiler and pins interior-field offsets too, catching any
    /// field-order/type transcription error that happens to preserve size.
    #[cfg(feature = "oracle")]
    #[test]
    fn struct_layout_matches_c() {
        use crate::oracle::*;
        use core::mem::{offset_of, size_of};

        unsafe {
            // sizes
            assert_eq!(size_of::<trajectory_t>(), jka_sizeof_trajectory_t());
            assert_eq!(size_of::<usercmd_t>(), jka_sizeof_usercmd_t());
            assert_eq!(size_of::<trace_t>(), jka_sizeof_trace_t());
            assert_eq!(size_of::<forcedata_t>(), jka_sizeof_forcedata_t());
            assert_eq!(size_of::<entityState_t>(), jka_sizeof_entityState_t());
            assert_eq!(size_of::<playerState_t>(), jka_sizeof_playerState_t());

            // trajectory_t
            assert_eq!(offset_of!(trajectory_t, trBase), jka_off_traj_trBase());
            assert_eq!(offset_of!(trajectory_t, trDelta), jka_off_traj_trDelta());

            // usercmd_t
            assert_eq!(offset_of!(usercmd_t, weapon), jka_off_cmd_weapon());
            assert_eq!(
                offset_of!(usercmd_t, forwardmove),
                jka_off_cmd_forwardmove()
            );

            // trace_t
            assert_eq!(offset_of!(trace_t, plane), jka_off_trace_plane());
            assert_eq!(offset_of!(trace_t, contents), jka_off_trace_contents());

            // forcedata_t
            assert_eq!(
                offset_of!(forcedata_t, forcePowerDuration),
                jka_off_fd_forcePowerDuration()
            );
            assert_eq!(
                offset_of!(forcedata_t, killSoundEntIndex),
                jka_off_fd_killSoundEntIndex()
            );
            assert_eq!(
                offset_of!(forcedata_t, privateDuelTime),
                jka_off_fd_privateDuelTime()
            );

            // entityState_t
            assert_eq!(offset_of!(entityState_t, pos), jka_off_es_pos());
            assert_eq!(offset_of!(entityState_t, speed), jka_off_es_speed());
            assert_eq!(
                offset_of!(entityState_t, customRGBA),
                jka_off_es_customRGBA()
            );
            assert_eq!(
                offset_of!(entityState_t, boneAngles1),
                jka_off_es_boneAngles1()
            );
            assert_eq!(offset_of!(entityState_t, userVec2), jka_off_es_userVec2());

            // playerState_t
            assert_eq!(offset_of!(playerState_t, origin), jka_off_ps_origin());
            assert_eq!(offset_of!(playerState_t, stats), jka_off_ps_stats());
            assert_eq!(offset_of!(playerState_t, fd), jka_off_ps_fd());
            assert_eq!(
                offset_of!(playerState_t, lastHitLoc),
                jka_off_ps_lastHitLoc()
            );
            assert_eq!(
                offset_of!(playerState_t, vehOrientation),
                jka_off_ps_vehOrientation()
            );
            assert_eq!(offset_of!(playerState_t, userVec2), jka_off_ps_userVec2());
        }
    }

    /// Parity: the saber structs (`saberInfo_t` is embedded by value in
    /// `gclient_t`) match the authentic C `sizeof`/`offsetof`. Pointer-free =>
    /// arch-independent, but we still pin interior offsets.
    #[cfg(feature = "oracle")]
    #[test]
    fn saber_struct_layout_matches_c() {
        use crate::oracle::*;
        use core::mem::{offset_of, size_of};

        unsafe {
            assert_eq!(size_of::<saberTrail_t>(), jka_sizeof_saberTrail_t());
            assert_eq!(
                offset_of!(saberTrail_t, oldPos),
                jka_off_saberTrail_oldPos()
            );
            assert_eq!(
                offset_of!(saberTrail_t, oldNormal),
                jka_off_saberTrail_oldNormal()
            );

            assert_eq!(size_of::<bladeInfo_t>(), jka_sizeof_bladeInfo_t());
            assert_eq!(offset_of!(bladeInfo_t, trail), jka_off_bladeInfo_trail());
            assert_eq!(
                offset_of!(bladeInfo_t, hitWallDebounceTime),
                jka_off_bladeInfo_hitWallDebounceTime()
            );

            assert_eq!(size_of::<saberInfo_t>(), jka_sizeof_saberInfo_t());
            assert_eq!(offset_of!(saberInfo_t, blade), jka_off_saberInfo_blade());
            assert_eq!(
                offset_of!(saberInfo_t, saberFlags),
                jka_off_saberInfo_saberFlags()
            );
            assert_eq!(
                offset_of!(saberInfo_t, swingSound),
                jka_off_saberInfo_swingSound()
            );
            assert_eq!(
                offset_of!(saberInfo_t, knockbackScale),
                jka_off_saberInfo_knockbackScale()
            );
            assert_eq!(
                offset_of!(saberInfo_t, splashKnockback2),
                jka_off_saberInfo_splashKnockback2()
            );
        }
    }

    /// Parity: the `material_e` enumerator values match the authentic C (anchor /
    /// interior / terminal count).
    #[cfg(feature = "oracle")]
    #[test]
    fn material_values_match_c() {
        use crate::oracle::*;
        unsafe {
            assert_eq!(MAT_METAL, jka_mat_MAT_METAL());
            assert_eq!(MAT_NONE, jka_mat_MAT_NONE());
            assert_eq!(MAT_SNOWY_ROCK, jka_mat_MAT_SNOWY_ROCK());
            assert_eq!(NUM_MATERIALS, jka_mat_NUM_MATERIALS());
        }
    }
}
