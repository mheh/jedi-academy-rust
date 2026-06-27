//! Port of `ai_wpnav.c` — bot waypoint/node navigation. This file owns the
//! waypoint array (`gWPArray`/`gWPNum`), the node-table scratch grid
//! (`nodetable`/`nodenum`) used by the trail-connection machinery, and the
//! render/edit globals (`gBotEdit`, `gWPRenderTime`, ...). The globals are
//! *declared* `extern` in `ai_main.h` but *defined* here, matching upstream.
//!
//! Opened at the portable leaves: `G_TestLine` (the keystone — only
//! `G_TempEntity`+`VectorCopy`), the pure node-table helpers
//! (`NodeHere`/`G_NodeClear*`/`G_NodeMatchingXY*`/`G_NearestNodeToPoint`), the
//! reachability/visibility traces (`CanGetToVector`/`CanGetToVectorTravel`/
//! `OrgVisibleCurve`/`CanForceJumpTo`), and the trap-driven WP-management +
//! spawn helpers whose callees are already landed.
//!
//! The deep path machinery (`ConnectTrail`, `RepairPaths`, `CalculatePaths`,
//! `LoadPathData`, `BeginAutoPathRoutine`, ...) and the `B_*` memory layer
//! (`ai_util.c`) plus the `bot_wp_*` cvars are not yet ported, so functions that
//! reach them are deferred for a later cycle.

#![allow(non_snake_case)] // C function names (`G_TestLine`) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`gWPArray`) kept verbatim
#![allow(dead_code)] // globals/fns consumed by ai_main + later-cycle path machinery
#![allow(unused_assignments)] // faithful C dead-stores / redundant re-inits (e.g. `i = 0;`, `tent = NULL;`)
use core::ffi::{c_char, c_int};
use core::mem::{offset_of, size_of};
use core::ptr::{addr_of, addr_of_mut, null_mut, write_bytes};

use crate::codemp::game::ai_main::{
    bot_normgpath, bot_wp_clearweight, bot_wp_distconnect, bot_wp_edit, bot_wp_info,
    bot_wp_visconnect, eFlagBlue, eFlagRed, flagBlue, flagRed, oFlagBlue, oFlagRed,
    GetNearestVisibleWP, OrgVisible, OrgVisibleBox,
};
use crate::codemp::game::ai_main_h::{
    nodeobject_t, MAX_NODETABLE_SIZE, TABLE_BRANCH_DISTANCE, WPFLAG_BLUE_FLAG, WPFLAG_CALCULATED,
    WPFLAG_DUCK, WPFLAG_GOALPOINT, WPFLAG_JUMP, WPFLAG_NEVERONEWAY, WPFLAG_NOMOVEFUNC, WPFLAG_NOVIS,
    WPFLAG_ONEWAY_BACK, WPFLAG_ONEWAY_FWD, WPFLAG_RED_FLAG, WPFLAG_SIEGE_IMPERIALOBJ,
    WPFLAG_SIEGE_REBELOBJ, WPFLAG_SNIPEORCAMP, WPFLAG_SNIPEORCAMPSTAND, WPFLAG_WAITFORFUNC,
    LEVELFLAG_NOPOINTPREDICTION,
};
use crate::codemp::game::ai_util::{B_Alloc, B_TempAlloc, B_TempFree};
use crate::codemp::game::bg_public::{
    DEFAULT_MAXS_2, DEFAULT_MINS_2, ET_TERRAIN, EV_SCOREPLUM, EV_TESTLINE, GT_SIEGE, IT_AMMO,
    IT_TEAM, MASK_PLAYERSOLID, MASK_SOLID, PW_BLUEFLAG, PW_REDFLAG,
};
use crate::codemp::game::bg_saga_h::SIEGETEAM_TEAM1;
use crate::codemp::game::bg_weapons_h::WP_NUM_WEAPONS;
use crate::codemp::game::g_cmds::ConcatArgs;
use crate::codemp::game::g_local::{gentity_s, gentity_t};
use crate::codemp::game::g_main::{g_entities, g_gametype, g_RMG, level, G_Printf};
use crate::codemp::game::g_public_h::SVF_BROADCAST;
use crate::codemp::game::g_utils::{G_Find, G_TempEntity};
use crate::codemp::game::q_math::{
    vec3_origin, VectorClear, VectorCopy, VectorLength, VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{va, Com_sprintf, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    vec2_t, vec3_t, wpneighbor_t, wpobject_t, CVAR_CHEAT, CVAR_ROM, CVAR_SERVERINFO,
    DEFAULT_GRID_SPACING, ENTITYNUM_NONE, ENTITYNUM_WORLD, FS_READ, FS_WRITE,
    MAX_NEIGHBOR_LINK_DISTANCE, MAX_NEIGHBOR_SIZE, MAX_WPARRAY_SIZE,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_SOLID;
use crate::ffi::types::{fileHandle_t, qboolean, QFALSE, QTRUE};
use crate::trap;

unsafe extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_RED: &str = "^1";


// ===========================================================================
// File-scope globals (ai_wpnav.c:6-23). Declared `extern` in ai_main.h, defined
// here. Marked `pub(crate)` so the parallel ai_main work can read them.
// ===========================================================================

/// `float gWPRenderTime = 0;` (ai_wpnav.c:6)
pub(crate) static mut gWPRenderTime: f32 = 0.0;
/// `float gDeactivated = 0;` (ai_wpnav.c:7)
pub(crate) static mut gDeactivated: f32 = 0.0;
/// `float gBotEdit = 0;` (ai_wpnav.c:8)
pub(crate) static mut gBotEdit: f32 = 0.0;
/// `int gWPRenderedFrame = 0;` (ai_wpnav.c:9)
pub(crate) static mut gWPRenderedFrame: c_int = 0;

/// `wpobject_t *gWPArray[MAX_WPARRAY_SIZE];` (ai_wpnav.c:12)
pub(crate) static mut gWPArray: [*mut wpobject_t; MAX_WPARRAY_SIZE] =
    [null_mut(); MAX_WPARRAY_SIZE];
/// `int gWPNum = 0;` (ai_wpnav.c:13)
pub(crate) static mut gWPNum: c_int = 0;

/// `int gLastPrintedIndex = -1;` (ai_wpnav.c:16)
pub(crate) static mut gLastPrintedIndex: c_int = -1;

/// `nodeobject_t nodetable[MAX_NODETABLE_SIZE];` (ai_wpnav.c:19)
pub(crate) static mut nodetable: [nodeobject_t; MAX_NODETABLE_SIZE] = [nodeobject_t {
    origin: [0.0; 3],
    weight: 0.0,
    flags: 0,
    neighbornum: 0,
    inuse: 0,
}; MAX_NODETABLE_SIZE];
/// `int nodenum;` (ai_wpnav.c:20) — so we can connect broken trails
pub(crate) static mut nodenum: c_int = 0;

/// `int gLevelFlags = 0;` (ai_wpnav.c:23)
pub(crate) static mut gLevelFlags: c_int = 0;

/// `void G_TestLine(vec3_t start, vec3_t end, int color, int time)` (ai_wpnav.c:212) —
/// keystone debug primitive: spawn an `EV_TESTLINE` temp-entity drawing a coloured line from
/// `start` to `end` for `time` ms, broadcast to all clients.
pub unsafe fn G_TestLine(start: &vec3_t, end: &vec3_t, color: c_int, time: c_int) {
    let te: *mut gentity_t = G_TempEntity(start, EV_TESTLINE);
    VectorCopy(start, &mut (*te).s.origin);
    VectorCopy(end, &mut (*te).s.origin2);
    (*te).s.time2 = time;
    (*te).s.weapon = color;
    (*te).r.svFlags |= SVF_BROADCAST;
}

/// `void RemoveWP(void)` (ai_wpnav.c:456) — pop the last waypoint off the array.
///
/// NOTE: faithfully replicates the upstream `memset( gWPArray[gWPNum], 0,
/// sizeof(gWPArray[gWPNum]) )` — `sizeof` of a *pointer* (not the struct), so this clears only
/// the first `size_of::<*mut wpobject_t>()` bytes of the object, exactly as the C does.
pub unsafe fn RemoveWP() {
    if gWPNum <= 0 {
        return;
    }

    gWPNum -= 1;

    if gWPArray[gWPNum as usize].is_null() || (*gWPArray[gWPNum as usize]).inuse == 0 {
        return;
    }

    //B_Free((wpobject_t *)gWPArray[gWPNum]);
    if !gWPArray[gWPNum as usize].is_null() {
        write_bytes(
            gWPArray[gWPNum as usize] as *mut u8,
            0,
            size_of::<*mut wpobject_t>(),
        );
    }

    //gWPArray[gWPNum] = NULL;

    if !gWPArray[gWPNum as usize].is_null() {
        (*gWPArray[gWPNum as usize]).inuse = 0;
    }
}

/// `void RemoveAllWP(void)` (ai_wpnav.c:484) — pop every waypoint.
pub unsafe fn RemoveAllWP() {
    while gWPNum != 0 {
        RemoveWP();
    }
}

/// `void TeleportToWP(gentity_t *pl, int afterindex)` (ai_wpnav.c:716) — warp `pl` to the
/// origin of the waypoint whose `index` is `afterindex`.
pub unsafe fn TeleportToWP(pl: *mut gentity_t, afterindex: c_int) {
    let foundindex: c_int;
    let mut foundanindex: c_int;
    let mut i: c_int;

    if pl.is_null() || (*pl).client.is_null() {
        return;
    }

    foundanindex = 0;
    i = 0;

    if afterindex < 0 || afterindex >= gWPNum {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint number {afterindex} does not exist\n"
        ));
        return;
    }

    let mut tmpfound = 0;
    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == afterindex
        {
            tmpfound = i;
            foundanindex = 1;
            break;
        }

        i += 1;
    }
    foundindex = tmpfound;

    if foundanindex == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint index {afterindex} should exist, but does not (?)\n"
        ));
        return;
    }

    let origin = (*gWPArray[foundindex as usize]).origin;
    VectorCopy(&origin, &mut (*(*pl).client).ps.origin);
}

/// `void WPFlagsModify(int wpnum, int flags)` (ai_wpnav.c:760) — overwrite a waypoint's flags.
pub unsafe fn WPFlagsModify(wpnum: c_int, flags: c_int) {
    if wpnum < 0
        || wpnum >= gWPNum
        || gWPArray[wpnum as usize].is_null()
        || (*gWPArray[wpnum as usize]).inuse == 0
    {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}WPFlagsModify: Waypoint {wpnum} does not exist\n"
        ));
        return;
    }

    (*gWPArray[wpnum as usize]).flags = flags;
}

/// `void TransferWPData(int from, int to)` (ai_wpnav.c:349) — copy waypoint `from`'s data into
/// slot `to`, allocating the destination if needed. Used to shuffle the array during
/// insert/remove operations.
pub unsafe fn TransferWPData(from: c_int, to: c_int) {
    if gWPArray[to as usize].is_null() {
        gWPArray[to as usize] = B_Alloc(size_of::<wpobject_t>() as c_int) as *mut wpobject_t;
    }

    if gWPArray[to as usize].is_null() {
        G_Printf(&format!(
            "{S_COLOR_RED}FATAL ERROR: Could not allocated memory for waypoint\n"
        ));
    }

    (*gWPArray[to as usize]).flags = (*gWPArray[from as usize]).flags;
    (*gWPArray[to as usize]).weight = (*gWPArray[from as usize]).weight;
    (*gWPArray[to as usize]).associated_entity = (*gWPArray[from as usize]).associated_entity;
    (*gWPArray[to as usize]).disttonext = (*gWPArray[from as usize]).disttonext;
    (*gWPArray[to as usize]).forceJumpTo = (*gWPArray[from as usize]).forceJumpTo;
    (*gWPArray[to as usize]).index = to;
    (*gWPArray[to as usize]).inuse = (*gWPArray[from as usize]).inuse;
    let origin = (*gWPArray[from as usize]).origin;
    VectorCopy(&origin, &mut (*gWPArray[to as usize]).origin);
}

/// `void CreateNewWP(vec3_t origin, int flags)` (ai_wpnav.c:371) — append a fresh waypoint at
/// `origin` with the given flags to the end of the array.
pub unsafe fn CreateNewWP(origin: &vec3_t, flags: c_int) {
    if gWPNum >= MAX_WPARRAY_SIZE as c_int {
        if g_RMG.integer == 0 {
            G_Printf(&format!(
                "{S_COLOR_YELLOW}Warning: Waypoint limit hit ({})\n",
                MAX_WPARRAY_SIZE
            ));
        }
        return;
    }

    if gWPArray[gWPNum as usize].is_null() {
        gWPArray[gWPNum as usize] = B_Alloc(size_of::<wpobject_t>() as c_int) as *mut wpobject_t;
    }

    if gWPArray[gWPNum as usize].is_null() {
        G_Printf(&format!(
            "{S_COLOR_RED}ERROR: Could not allocated memory for waypoint\n"
        ));
    }

    (*gWPArray[gWPNum as usize]).flags = flags;
    (*gWPArray[gWPNum as usize]).weight = 0.0; //calculated elsewhere
    (*gWPArray[gWPNum as usize]).associated_entity = ENTITYNUM_NONE as c_int; //set elsewhere
    (*gWPArray[gWPNum as usize]).forceJumpTo = 0;
    (*gWPArray[gWPNum as usize]).disttonext = 0.0; //calculated elsewhere
    (*gWPArray[gWPNum as usize]).index = gWPNum;
    (*gWPArray[gWPNum as usize]).inuse = 1;
    VectorCopy(origin, &mut (*gWPArray[gWPNum as usize]).origin);
    gWPNum += 1;
}

/// `void CreateNewWP_FromObject(wpobject_t *wp)` (ai_wpnav.c:403) — append a waypoint cloned
/// from an existing `wpobject_t` (including its neighbor links); used when reloading a saved
/// path. Tracks the CTF flag waypoints (`flagRed`/`flagBlue` and their originals) by flag bit.
pub unsafe fn CreateNewWP_FromObject(wp: *mut wpobject_t) {
    let mut i: c_int;

    if gWPNum >= MAX_WPARRAY_SIZE as c_int {
        return;
    }

    if gWPArray[gWPNum as usize].is_null() {
        gWPArray[gWPNum as usize] = B_Alloc(size_of::<wpobject_t>() as c_int) as *mut wpobject_t;
    }

    if gWPArray[gWPNum as usize].is_null() {
        G_Printf(&format!(
            "{S_COLOR_RED}ERROR: Could not allocated memory for waypoint\n"
        ));
    }

    (*gWPArray[gWPNum as usize]).flags = (*wp).flags;
    (*gWPArray[gWPNum as usize]).weight = (*wp).weight;
    (*gWPArray[gWPNum as usize]).associated_entity = (*wp).associated_entity;
    (*gWPArray[gWPNum as usize]).disttonext = (*wp).disttonext;
    (*gWPArray[gWPNum as usize]).forceJumpTo = (*wp).forceJumpTo;
    (*gWPArray[gWPNum as usize]).index = gWPNum;
    (*gWPArray[gWPNum as usize]).inuse = 1;
    let origin = (*wp).origin;
    VectorCopy(&origin, &mut (*gWPArray[gWPNum as usize]).origin);
    (*gWPArray[gWPNum as usize]).neighbornum = (*wp).neighbornum;

    i = (*wp).neighbornum;

    while i >= 0 {
        (*gWPArray[gWPNum as usize]).neighbors[i as usize].num = (*wp).neighbors[i as usize].num;
        (*gWPArray[gWPNum as usize]).neighbors[i as usize].forceJumpTo =
            (*wp).neighbors[i as usize].forceJumpTo;

        i -= 1;
    }

    if (*gWPArray[gWPNum as usize]).flags & WPFLAG_RED_FLAG != 0 {
        flagRed = gWPArray[gWPNum as usize];
        oFlagRed = flagRed;
    } else if (*gWPArray[gWPNum as usize]).flags & WPFLAG_BLUE_FLAG != 0 {
        flagBlue = gWPArray[gWPNum as usize];
        oFlagBlue = flagBlue;
    }

    gWPNum += 1;
}

/// `void RemoveWP_InTrail(int afterindex)` (ai_wpnav.c:491) — delete the waypoint whose `index`
/// is `afterindex`, sliding every later waypoint down one slot to close the gap.
pub unsafe fn RemoveWP_InTrail(afterindex: c_int) {
    let foundindex: c_int;
    let mut foundanindex: c_int;
    let mut didchange: c_int;
    let mut i: c_int;

    foundanindex = 0;
    didchange = 0;
    i = 0;

    if afterindex < 0 || afterindex >= gWPNum {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint number {afterindex} does not exist\n"
        ));
        return;
    }

    let mut tmpfound = 0;
    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == afterindex
        {
            tmpfound = i;
            foundanindex = 1;
            break;
        }

        i += 1;
    }
    foundindex = tmpfound;

    if foundanindex == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint index {afterindex} should exist, but does not (?)\n"
        ));
        return;
    }

    i = 0;

    while i <= gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).index == foundindex {
            //B_Free(gWPArray[i]);
            //Keep reusing the memory
            write_bytes(
                gWPArray[i as usize] as *mut u8,
                0,
                size_of::<*mut wpobject_t>(),
            );

            //gWPArray[i] = NULL;
            (*gWPArray[i as usize]).inuse = 0;
            didchange = 1;
        } else if !gWPArray[i as usize].is_null() && didchange != 0 {
            TransferWPData(i, i - 1);
            //B_Free(gWPArray[i]);

            //Keep reusing the memory
            write_bytes(
                gWPArray[i as usize] as *mut u8,
                0,
                size_of::<*mut wpobject_t>(),
            );

            //gWPArray[i] = NULL;
            (*gWPArray[i as usize]).inuse = 0;
        }

        i += 1;
    }
    gWPNum -= 1;
}

/// `int CreateNewWP_InTrail(vec3_t origin, int flags, int afterindex)` (ai_wpnav.c:559) — insert
/// a new waypoint immediately *after* the one with `index == afterindex`, sliding later points up.
pub unsafe fn CreateNewWP_InTrail(origin: &vec3_t, flags: c_int, afterindex: c_int) -> c_int {
    let foundindex: c_int;
    let mut foundanindex: c_int;
    let mut i: c_int;

    foundanindex = 0;
    i = 0;

    if gWPNum >= MAX_WPARRAY_SIZE as c_int {
        if g_RMG.integer == 0 {
            G_Printf(&format!(
                "{S_COLOR_YELLOW}Warning: Waypoint limit hit ({})\n",
                MAX_WPARRAY_SIZE
            ));
        }
        return 0;
    }

    if afterindex < 0 || afterindex >= gWPNum {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint number {afterindex} does not exist\n"
        ));
        return 0;
    }

    let mut tmpfound = 0;
    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == afterindex
        {
            tmpfound = i;
            foundanindex = 1;
            break;
        }

        i += 1;
    }
    foundindex = tmpfound;

    if foundanindex == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint index {afterindex} should exist, but does not (?)\n"
        ));
        return 0;
    }

    i = gWPNum;

    while i >= 0 {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index != foundindex
        {
            TransferWPData(i, i + 1);
        } else if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == foundindex
        {
            i += 1;

            if gWPArray[i as usize].is_null() {
                gWPArray[i as usize] =
                    B_Alloc(size_of::<wpobject_t>() as c_int) as *mut wpobject_t;
            }

            (*gWPArray[i as usize]).flags = flags;
            (*gWPArray[i as usize]).weight = 0.0; //calculated elsewhere
            (*gWPArray[i as usize]).associated_entity = ENTITYNUM_NONE as c_int; //set elsewhere
            (*gWPArray[i as usize]).disttonext = 0.0; //calculated elsewhere
            (*gWPArray[i as usize]).forceJumpTo = 0;
            (*gWPArray[i as usize]).index = i;
            (*gWPArray[i as usize]).inuse = 1;
            VectorCopy(origin, &mut (*gWPArray[i as usize]).origin);
            gWPNum += 1;
            break;
        }

        i -= 1;
    }

    1
}

/// `int CreateNewWP_InsertUnder(vec3_t origin, int flags, int afterindex)` (ai_wpnav.c:637) —
/// like `CreateNewWP_InTrail` but inserts *at* the matched slot (shifting the matched point up
/// too), placing the new point "under" it in the trail.
pub unsafe fn CreateNewWP_InsertUnder(origin: &vec3_t, flags: c_int, afterindex: c_int) -> c_int {
    let foundindex: c_int;
    let mut foundanindex: c_int;
    let mut i: c_int;

    foundanindex = 0;
    i = 0;

    if gWPNum >= MAX_WPARRAY_SIZE as c_int {
        if g_RMG.integer == 0 {
            G_Printf(&format!(
                "{S_COLOR_YELLOW}Warning: Waypoint limit hit ({})\n",
                MAX_WPARRAY_SIZE
            ));
        }
        return 0;
    }

    if afterindex < 0 || afterindex >= gWPNum {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint number {afterindex} does not exist\n"
        ));
        return 0;
    }

    let mut tmpfound = 0;
    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == afterindex
        {
            tmpfound = i;
            foundanindex = 1;
            break;
        }

        i += 1;
    }
    foundindex = tmpfound;

    if foundanindex == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint index {afterindex} should exist, but does not (?)\n"
        ));
        return 0;
    }

    i = gWPNum;

    while i >= 0 {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index != foundindex
        {
            TransferWPData(i, i + 1);
        } else if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).index == foundindex
        {
            //i++;
            TransferWPData(i, i + 1);

            if gWPArray[i as usize].is_null() {
                gWPArray[i as usize] =
                    B_Alloc(size_of::<wpobject_t>() as c_int) as *mut wpobject_t;
            }

            (*gWPArray[i as usize]).flags = flags;
            (*gWPArray[i as usize]).weight = 0.0; //calculated elsewhere
            (*gWPArray[i as usize]).associated_entity = ENTITYNUM_NONE as c_int; //set elsewhere
            (*gWPArray[i as usize]).disttonext = 0.0; //calculated elsewhere
            (*gWPArray[i as usize]).forceJumpTo = 0;
            (*gWPArray[i as usize]).index = i;
            (*gWPArray[i as usize]).inuse = 1;
            VectorCopy(origin, &mut (*gWPArray[i as usize]).origin);
            gWPNum += 1;
            break;
        }

        i -= 1;
    }

    1
}

/// `char *GetFlagStr(int flags)` (ai_wpnav.c:25) — render a waypoint's flag bitmask into a
/// short human-readable string in a 128-byte `B_TempAlloc` buffer (e.g. `"jd red flag"`). The
/// caller must `B_TempFree(128)` it. Returns a pointer into the temp arena.
pub unsafe fn GetFlagStr(flags: c_int) -> *mut c_char {
    let flagstr = B_TempAlloc(128) as *mut c_char;
    let mut i: isize = 0;

    // faithful to `flagstr[i] = c; i++;`
    let put = |i: &mut isize, c: u8| unsafe {
        *flagstr.offset(*i) = c as c_char;
        *i += 1;
    };
    // faithful to `strcpy(flagstr, s)` for the literal-with-NUL cases
    let put_str = |s: &[u8]| unsafe {
        core::ptr::copy_nonoverlapping(s.as_ptr() as *const c_char, flagstr, s.len());
    };

    if flags == 0 {
        put_str(b"none\0");
        return flagstr;
    }

    if flags & WPFLAG_JUMP != 0 {
        put(&mut i, b'j');
    }
    if flags & WPFLAG_DUCK != 0 {
        put(&mut i, b'd');
    }
    if flags & WPFLAG_SNIPEORCAMPSTAND != 0 {
        put(&mut i, b'c');
    }
    if flags & WPFLAG_WAITFORFUNC != 0 {
        put(&mut i, b'f');
    }
    if flags & WPFLAG_SNIPEORCAMP != 0 {
        put(&mut i, b's');
    }
    if flags & WPFLAG_ONEWAY_FWD != 0 {
        put(&mut i, b'x');
    }
    if flags & WPFLAG_ONEWAY_BACK != 0 {
        put(&mut i, b'y');
    }
    if flags & WPFLAG_GOALPOINT != 0 {
        put(&mut i, b'g');
    }
    if flags & WPFLAG_NOVIS != 0 {
        put(&mut i, b'n');
    }
    if flags & WPFLAG_NOMOVEFUNC != 0 {
        put(&mut i, b'm');
    }

    if flags & WPFLAG_RED_FLAG != 0 {
        if i != 0 {
            put(&mut i, b' ');
        }
        for &c in b"red flag" {
            put(&mut i, c);
        }
    }

    if flags & WPFLAG_BLUE_FLAG != 0 {
        if i != 0 {
            put(&mut i, b' ');
        }
        for &c in b"blue flag" {
            put(&mut i, c);
        }
    }

    if flags & WPFLAG_SIEGE_IMPERIALOBJ != 0 {
        if i != 0 {
            put(&mut i, b' ');
        }
        for &c in b"saga_imp" {
            put(&mut i, c);
        }
    }

    if flags & WPFLAG_SIEGE_REBELOBJ != 0 {
        if i != 0 {
            put(&mut i, b' ');
        }
        for &c in b"saga_reb" {
            put(&mut i, c);
        }
    }

    put(&mut i, b'\0');

    if i == 0 {
        put_str(b"unknown\0");
    }

    flagstr
}

/// `void BotWaypointRender(void)` (ai_wpnav.c:224) — debug visualization (`gBotEdit`): draws a
/// score-plum at each waypoint plus jump-link test lines, and prints flag info for the waypoint
/// nearest the first client. Throttled via `gWPRenderTime`/`gWPRenderedFrame`.
pub unsafe fn BotWaypointRender() {
    let mut i: c_int;
    let mut n: c_int;
    let inc_checker: c_int;
    let mut bestindex: c_int;
    let mut gotbestindex: c_int;
    let mut bestdist: f32;
    let mut checkdist: f32;
    let plum: *mut gentity_t;
    let viewent: *mut gentity_t;
    let flagstr: *mut c_char;
    let mut a: vec3_t = [0.0; 3];

    if gBotEdit == 0.0 {
        return;
    }

    bestindex = 0;

    if gWPRenderTime <= level.time as f32 {
        gWPRenderTime = (level.time + 100) as f32;

        i = gWPRenderedFrame;
        inc_checker = gWPRenderedFrame;

        while i < gWPNum {
            if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
                let porigin = (*gWPArray[i as usize]).origin;
                let plum = G_TempEntity(&porigin, EV_SCOREPLUM);
                (*plum).r.svFlags |= SVF_BROADCAST;
                (*plum).s.time = i;

                n = 0;

                while n < (*gWPArray[i as usize]).neighbornum {
                    if (*gWPArray[i as usize]).neighbors[n as usize].forceJumpTo != 0
                        && !gWPArray[(*gWPArray[i as usize]).neighbors[n as usize].num as usize]
                            .is_null()
                    {
                        let start = (*gWPArray[i as usize]).origin;
                        let end = (*gWPArray
                            [(*gWPArray[i as usize]).neighbors[n as usize].num as usize])
                            .origin;
                        G_TestLine(&start, &end, 0x0000ff, 5000);
                    }
                    n += 1;
                }

                gWPRenderedFrame += 1;
            } else {
                gWPRenderedFrame = 0;
                break;
            }

            if (i - inc_checker) > 4 {
                break; //don't render too many at once
            }
            i += 1;
        }

        if i >= gWPNum {
            gWPRenderTime = (level.time + 1500) as f32; //wait a bit after we finish the whole trail
            gWPRenderedFrame = 0;
        }
    }

    // checkprint:
    if bot_wp_info.value == 0.0 {
        return;
    }

    viewent = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(); //&g_entities[0] — only show info to the first client

    if viewent.is_null() || (*viewent).client.is_null() {
        //client isn't in the game yet?
        return;
    }

    bestdist = 256.0; //max distance for showing point info
    gotbestindex = 0;

    i = 0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            let org = (*gWPArray[i as usize]).origin;
            VectorSubtract(&(*(*viewent).client).ps.origin, &org, &mut a);

            checkdist = VectorLength(&a);

            if checkdist < bestdist {
                bestdist = checkdist;
                bestindex = i;
                gotbestindex = 1;
            }
        }
        i += 1;
    }

    if gotbestindex != 0 && bestindex != gLastPrintedIndex {
        flagstr = GetFlagStr((*gWPArray[bestindex as usize]).flags);
        gLastPrintedIndex = bestindex;
        let wp = gWPArray[bestindex as usize];
        let fstr = core::ffi::CStr::from_ptr(flagstr).to_string_lossy().into_owned();
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Waypoint {}\nFlags - {} ({}) (w{:.6})\nOrigin - ({} {} {})\n",
            (*wp).index,
            (*wp).flags,
            fstr,
            (*wp).weight,
            (*wp).origin[0] as c_int,
            (*wp).origin[1] as c_int,
            (*wp).origin[2] as c_int,
        ));
        //GetFlagStr allocates 128 bytes for this, if it's changed then obviously this must be as well
        B_TempFree(128); //flagstr

        let porigin = (*wp).origin;
        plum = G_TempEntity(&porigin, EV_SCOREPLUM);
        (*plum).r.svFlags |= SVF_BROADCAST;
        (*plum).s.time = bestindex; //render it once
    } else if gotbestindex == 0 {
        gLastPrintedIndex = -1;
    }
}

/// `static int NotWithinRange(int base, int extent)` (ai_wpnav.c:771) — true unless `extent`
/// is within ±5 of `base` (used to skip near-adjacent waypoints in path calc).
fn NotWithinRange(base: c_int, extent: c_int) -> c_int {
    if extent > base && base + 5 >= extent {
        return 0;
    }

    if extent < base && base - 5 <= extent {
        return 0;
    }

    1
}

/// `int NodeHere(vec3_t spot)` (ai_wpnav.c:787) — true if a node already exists at `spot`
/// (integer x/y match, z within ±5).
pub unsafe fn NodeHere(spot: &vec3_t) -> c_int {
    let mut i: c_int = 0;

    while i < nodenum {
        if nodetable[i as usize].origin[0] as c_int == spot[0] as c_int
            && nodetable[i as usize].origin[1] as c_int == spot[1] as c_int
        {
            if nodetable[i as usize].origin[2] as c_int == spot[2] as c_int
                || (nodetable[i as usize].origin[2] as c_int) < spot[2] as c_int
                    && nodetable[i as usize].origin[2] as c_int + 5 > spot[2] as c_int
                || nodetable[i as usize].origin[2] as c_int > spot[2] as c_int
                    && nodetable[i as usize].origin[2] as c_int - 5 < spot[2] as c_int
            {
                return 1;
            }
        }
        i += 1;
    }

    0
}

/// `int CanGetToVector(vec3_t org1, vec3_t org2, vec3_t mins, vec3_t maxs)` (ai_wpnav.c:812) —
/// straight-line traceable from `org1` to `org2` (clear, not in solid).
pub fn CanGetToVector(org1: &vec3_t, org2: &vec3_t, mins: &vec3_t, maxs: &vec3_t) -> c_int {
    let tr = trap::Trace(org1, mins, maxs, org2, ENTITYNUM_NONE, MASK_SOLID);

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        return 1;
    }

    0
}

/// `int OrgVisibleCurve(vec3_t org1, vec3_t mins, vec3_t maxs, vec3_t org2, int ignore)`
/// (ai_wpnav.c:1526) — visible along an L-shaped path: level out `org1` to `org2`'s height,
/// then check both legs of the bend trace clear.
pub fn OrgVisibleCurve(
    org1: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    org2: &vec3_t,
    ignore: c_int,
) -> c_int {
    let mut evenorg1: vec3_t = [0.0; 3];

    VectorCopy(org1, &mut evenorg1);
    evenorg1[2] = org2[2];

    let mut tr = trap::Trace(&evenorg1, mins, maxs, org2, ignore, MASK_SOLID);

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        tr = trap::Trace(&evenorg1, mins, maxs, org1, ignore, MASK_SOLID);

        if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
            return 1;
        }
    }

    0
}

/// `int CanForceJumpTo(int baseindex, int testingindex, float distance)` (ai_wpnav.c:1549) —
/// classify whether a force-jump can carry from waypoint `baseindex` up to `testingindex`,
/// returning the force-jump tier (1/2/3) or `0` if not jumpable.
pub unsafe fn CanForceJumpTo(baseindex: c_int, testingindex: c_int, distance: f32) -> c_int {
    let heightdif: f32;
    let mut xy_base: vec3_t = [0.0; 3];
    let mut xy_test: vec3_t = [0.0; 3];
    let mut v: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let wpBase: *mut wpobject_t = gWPArray[baseindex as usize];
    let wpTest: *mut wpobject_t = gWPArray[testingindex as usize];

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -15.0; //-1
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 15.0; //1

    if wpBase.is_null() || (*wpBase).inuse == 0 || wpTest.is_null() || (*wpTest).inuse == 0 {
        return 0;
    }

    if distance > 400.0 {
        return 0;
    }

    VectorCopy(&(*wpBase).origin, &mut xy_base);
    VectorCopy(&(*wpTest).origin, &mut xy_test);

    xy_base[2] = xy_test[2];

    VectorSubtract(&xy_base, &xy_test, &mut v);

    if VectorLength(&v) > MAX_NEIGHBOR_LINK_DISTANCE as f32 {
        return 0;
    }

    if ((*wpBase).origin[2] as c_int) < (*wpTest).origin[2] as c_int {
        heightdif = (*wpTest).origin[2] - (*wpBase).origin[2];
    } else {
        return 0; //err..
    }

    if heightdif < 128.0 {
        //don't bother..
        return 0;
    }

    if heightdif > 512.0 {
        //too high
        return 0;
    }

    if OrgVisibleCurve(&(*wpBase).origin, &mins, &maxs, &(*wpTest).origin, ENTITYNUM_NONE) == 0 {
        return 0;
    }

    if heightdif > 400.0 {
        3
    } else if heightdif > 256.0 {
        2
    } else {
        1
    }
}

/// `gentity_t *GetObjectThatTargets(gentity_t *ent)` (ai_wpnav.c:1715) — find the first entity
/// whose `target` matches `ent`'s `targetname`.
pub unsafe fn GetObjectThatTargets(ent: *mut gentity_t) -> *mut gentity_t {
    let next: *mut gentity_t;

    if (*ent).targetname.is_null() {
        return null_mut();
    }

    next = G_Find(
        null_mut(),
        offset_of!(gentity_s, target),
        (*ent).targetname,
    );

    if !next.is_null() {
        return next;
    }

    null_mut()
}

/// `int GetNearestVisibleWPToItem(vec3_t org, int ignore)` (ai_wpnav.c:1817) — nearest in-PVS,
/// box-visible waypoint within 64 units (and matching z ±15) of `org`; `-1` if none.
pub unsafe fn GetNearestVisibleWPToItem(org: &vec3_t, ignore: c_int) -> c_int {
    let mut i: c_int;
    let mut bestdist: f32;
    let mut flLen: f32;
    let mut bestindex: c_int;
    let mut a: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    i = 0;
    bestdist = 64.0; //has to be less than 64 units to the item or it isn't safe enough
    bestindex = -1;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = 0.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 0.0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).origin[2] - 15.0 < org[2]
            && (*gWPArray[i as usize]).origin[2] + 15.0 > org[2]
        {
            VectorSubtract(org, &(*gWPArray[i as usize]).origin, &mut a);
            flLen = VectorLength(&a);

            if flLen < bestdist
                && trap::InPVS(org, &(*gWPArray[i as usize]).origin) != 0
                && OrgVisibleBox(org, &mins, &maxs, &(*gWPArray[i as usize]).origin, ignore) != 0
            {
                bestdist = flLen;
                bestindex = i;
            }
        }

        i += 1;
    }

    bestindex
}

/// `int G_NearestNodeToPoint(vec3_t point)` (ai_wpnav.c:2510) — index of the node on the grid
/// closest to `point`; `-1` if the node table is empty.
pub unsafe fn G_NearestNodeToPoint(point: &vec3_t) -> c_int {
    let mut vSub: vec3_t = [0.0; 3];
    let mut bestIndex: c_int = -1;
    let mut i: c_int = 0;
    let mut bestDist: f32 = 0.0;
    let mut testDist: f32;

    while i < nodenum {
        VectorSubtract(&nodetable[i as usize].origin, point, &mut vSub);
        testDist = VectorLength(&vSub);

        if bestIndex == -1 {
            bestIndex = i;
            bestDist = testDist;

            i += 1;
            continue;
        }

        if testDist < bestDist {
            bestIndex = i;
            bestDist = testDist;
        }
        i += 1;
    }

    bestIndex
}

/// `void G_NodeClearForNext(void)` (ai_wpnav.c:2545) — reset nodes for the next trail
/// connection (clear flags, reset weights to a large sentinel).
pub unsafe fn G_NodeClearForNext() {
    let mut i: c_int = 0;

    while i < nodenum {
        nodetable[i as usize].flags = 0;
        nodetable[i as usize].weight = 99999.0;

        i += 1;
    }
}

/// `void G_NodeClearFlags(void)` (ai_wpnav.c:2558) — clear node flags only so nodes can be
/// reused.
pub unsafe fn G_NodeClearFlags() {
    let mut i: c_int = 0;

    while i < nodenum {
        nodetable[i as usize].flags = 0;

        i += 1;
    }
}

/// `int G_NodeMatchingXY(float x, float y)` (ai_wpnav.c:2570) — first unflagged node with the
/// matching x,y coordinates; `-1` if none.
pub unsafe fn G_NodeMatchingXY(x: f32, y: f32) -> c_int {
    let mut i: c_int = 0;

    while i < nodenum {
        if nodetable[i as usize].origin[0] == x
            && nodetable[i as usize].origin[1] == y
            && nodetable[i as usize].flags == 0
        {
            return i;
        }

        i += 1;
    }

    -1
}

/// `int G_NodeMatchingXY_BA(int x, int y, int final)` (ai_wpnav.c:2589) — the lowest-weight
/// node matching the specified x,y coordinates (or `final` if it matches first).
pub unsafe fn G_NodeMatchingXY_BA(x: c_int, y: c_int, r#final: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut bestindex: c_int = -1;
    let mut bestWeight: f32 = 9999.0;

    while i < nodenum {
        if nodetable[i as usize].origin[0] as c_int == x
            && nodetable[i as usize].origin[1] as c_int == y
            && nodetable[i as usize].flags == 0
            && (nodetable[i as usize].weight < bestWeight || i == r#final)
        {
            if i == r#final {
                return i;
            }
            bestindex = i;
            bestWeight = nodetable[i as usize].weight;
        }

        i += 1;
    }

    bestindex
}

/// `gentity_t *GetClosestSpawn(gentity_t *ent)` (ai_wpnav.c:3425) — nearest player-start /
/// deathmatch spawn entity to `ent`.
pub unsafe fn GetClosestSpawn(ent: *mut gentity_t) -> *mut gentity_t {
    let mut spawn: *mut gentity_t;
    let mut closestSpawn: *mut gentity_t = null_mut();
    let mut closestDist: f32 = -1.0;
    let mut i: c_int = crate::codemp::game::q_shared_h::MAX_CLIENTS as c_int;

    while i < (*addr_of!(level)).num_entities {
        spawn = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !spawn.is_null()
            && (*spawn).inuse != 0
            && (Q_stricmp((*spawn).classname, c"info_player_start".as_ptr()) == 0
                || Q_stricmp((*spawn).classname, c"info_player_deathmatch".as_ptr()) == 0)
        {
            let checkDist: f32;
            let mut vSub: vec3_t = [0.0; 3];

            VectorSubtract(
                &(*(*ent).client).ps.origin,
                &(*spawn).r.currentOrigin,
                &mut vSub,
            );
            checkDist = VectorLength(&vSub);

            if closestDist == -1.0 || checkDist < closestDist {
                closestSpawn = spawn;
                closestDist = checkDist;
            }
        }

        i += 1;
    }

    closestSpawn
}

/// `gentity_t *GetNextSpawnInIndex(gentity_t *currentSpawn)` (ai_wpnav.c:3459) — the next
/// player-start / deathmatch spawn after `currentSpawn` in entity order, looping back to the
/// start of the client range if needed.
pub unsafe fn GetNextSpawnInIndex(currentSpawn: *mut gentity_t) -> *mut gentity_t {
    let mut spawn: *mut gentity_t;
    let mut nextSpawn: *mut gentity_t = null_mut();
    let mut i: c_int = (*currentSpawn).s.number + 1;

    while i < (*addr_of!(level)).num_entities {
        spawn = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !spawn.is_null()
            && (*spawn).inuse != 0
            && (Q_stricmp((*spawn).classname, c"info_player_start".as_ptr()) == 0
                || Q_stricmp((*spawn).classname, c"info_player_deathmatch".as_ptr()) == 0)
        {
            nextSpawn = spawn;
            break;
        }

        i += 1;
    }

    if nextSpawn.is_null() {
        //loop back around to 0
        i = crate::codemp::game::q_shared_h::MAX_CLIENTS as c_int;

        while i < (*addr_of!(level)).num_entities {
            spawn = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if !spawn.is_null()
                && (*spawn).inuse != 0
                && (Q_stricmp((*spawn).classname, c"info_player_start".as_ptr()) == 0
                    || Q_stricmp((*spawn).classname, c"info_player_deathmatch".as_ptr()) == 0)
            {
                nextSpawn = spawn;
                break;
            }

            i += 1;
        }
    }

    nextSpawn
}

/// `int CanGetToVectorTravel(vec3_t org1, vec3_t moveTo, vec3_t mins, vec3_t maxs)`
/// (ai_wpnav.c:883) — walk a player-sized box from `org1` toward `moveTo` (ignoring Z), stepping
/// over stairs, and report whether any forward progress was actually made. This is the active
/// `#else` branch (the `#if 0` ramp-slope variant is dead in upstream).
pub fn CanGetToVectorTravel(org1: &vec3_t, moveTo: &vec3_t, mins: &vec3_t, maxs: &vec3_t) -> c_int {
    let mut tr;
    let mut stepTo: vec3_t = [0.0; 3];
    let mut stepSub: vec3_t = [0.0; 3];
    let mut stepGoal: vec3_t = [0.0; 3];
    let mut workingOrg: vec3_t = [0.0; 3];
    let mut lastIncrement: vec3_t = [0.0; 3];
    let mut finalMeasure: vec3_t = [0.0; 3];
    let mut stepSize: f32;
    let mut measureLength: f32;
    let mut didMove: c_int = 0;
    let traceMask: c_int = MASK_PLAYERSOLID;
    let mut initialDone: qboolean = QFALSE;

    VectorCopy(org1, &mut workingOrg);
    VectorCopy(org1, &mut lastIncrement);

    VectorCopy(moveTo, &mut stepTo);
    stepTo[2] = workingOrg[2];

    VectorSubtract(&stepTo, &workingOrg, &mut stepSub);
    stepSize = VectorLength(&stepSub); //make the step size the length of the original positions without Z

    VectorNormalize(&mut stepSub);

    while initialDone == QFALSE || didMove != 0 {
        initialDone = QTRUE;
        didMove = 0;

        stepGoal[0] = workingOrg[0] + stepSub[0] * stepSize;
        stepGoal[1] = workingOrg[1] + stepSub[1] * stepSize;
        stepGoal[2] = workingOrg[2] + stepSub[2] * stepSize;

        tr = trap::Trace(&workingOrg, mins, maxs, &stepGoal, ENTITYNUM_NONE, traceMask);

        if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction != 0.0 {
            let mut vecSub: vec3_t = [0.0; 3];
            VectorSubtract(&workingOrg, &tr.endpos, &mut vecSub);

            if VectorLength(&vecSub) > (stepSize / 2.0) {
                workingOrg[0] = tr.endpos[0];
                workingOrg[1] = tr.endpos[1];
                //trap_LinkEntity(self);
                didMove = 1;
            }
        }

        if didMove != 1 {
            //stair check
            let mut trFrom: vec3_t = [0.0; 3];
            let mut trTo: vec3_t = [0.0; 3];
            let mut trDir: vec3_t = [0.0; 3];
            let mut vecMeasure: vec3_t = [0.0; 3];

            VectorCopy(&tr.endpos, &mut trFrom);
            trFrom[2] += 16.0;

            VectorSubtract(/*tr.endpos*/ &stepGoal, &workingOrg, &mut trDir);
            VectorNormalize(&mut trDir);
            trTo[0] = tr.endpos[0] + trDir[0] * 2.0;
            trTo[1] = tr.endpos[1] + trDir[1] * 2.0;
            trTo[2] = tr.endpos[2] + trDir[2] * 2.0;
            trTo[2] += 16.0;

            VectorSubtract(&trFrom, &trTo, &mut vecMeasure);

            if VectorLength(&vecMeasure) > 1.0 {
                tr = trap::Trace(&trFrom, mins, maxs, &trTo, ENTITYNUM_NONE, traceMask);

                if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction == 1.0 {
                    //clear trace here, probably up a step
                    let mut trUp: vec3_t = [0.0; 3];
                    let mut trDown: vec3_t = [0.0; 3];
                    VectorCopy(&tr.endpos, &mut trUp);
                    VectorCopy(&tr.endpos, &mut trDown);
                    trDown[2] -= 16.0;

                    tr = trap::Trace(&trFrom, mins, maxs, &trTo, ENTITYNUM_NONE, traceMask);

                    if tr.startsolid == 0 && tr.allsolid == 0 {
                        //plop us down on the step after moving up
                        VectorCopy(&tr.endpos, &mut workingOrg);
                        //trap_LinkEntity(self);
                        didMove = 1;
                    }
                }
            }
        }

        VectorSubtract(&lastIncrement, &workingOrg, &mut finalMeasure);
        measureLength = VectorLength(&finalMeasure);

        if measureLength == 0.0 {
            //no progress, break out. If last movement was a sucess didMove will equal 1.
            break;
        }

        stepSize -= measureLength; //subtract the progress distance from the step size so we don't overshoot the mark.
        if stepSize <= 0.0 {
            break;
        }

        VectorCopy(&workingOrg, &mut lastIncrement);
    }

    didMove
}

/// `int OpposingEnds(int start, int end)` (ai_wpnav.c:1402) — true if `start` is flagged
/// forward-only and `end` is flagged backward-only (a one-way pair that must not be connected).
pub unsafe fn OpposingEnds(start: c_int, end: c_int) -> c_int {
    if gWPArray[start as usize].is_null()
        || (*gWPArray[start as usize]).inuse == 0
        || gWPArray[end as usize].is_null()
        || (*gWPArray[end as usize]).inuse == 0
    {
        return 0;
    }

    if (*gWPArray[start as usize]).flags & WPFLAG_ONEWAY_FWD != 0
        && (*gWPArray[end as usize]).flags & WPFLAG_ONEWAY_BACK != 0
    {
        return 1;
    }

    0
}

/// `int DoorBlockingSection(int start, int end)` (ai_wpnav.c:1418) — if a `func_` door blocks the
/// trail between `start` and `end` (and blocks it symmetrically from both directions), assume the
/// two points are in visibility when it opens, so return 1.
pub unsafe fn DoorBlockingSection(start: c_int, end: c_int) -> c_int {
    //if a door blocks the trail, we'll just have to assume the points on each side are in visibility when it's open
    let mut tr;
    let testdoor: *mut gentity_t;
    let start_trace_index: c_int;

    if gWPArray[start as usize].is_null()
        || (*gWPArray[start as usize]).inuse == 0
        || gWPArray[end as usize].is_null()
        || (*gWPArray[end as usize]).inuse == 0
    {
        return 0;
    }

    tr = trap::Trace(
        &(*gWPArray[start as usize]).origin,
        &vec3_origin,
        &vec3_origin,
        &(*gWPArray[end as usize]).origin,
        ENTITYNUM_NONE,
        MASK_SOLID,
    );

    if tr.fraction == 1.0 {
        return 0;
    }

    testdoor = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

    if testdoor.is_null() {
        return 0;
    }

    if strstr((*testdoor).classname, c"func_".as_ptr()).is_null() {
        return 0;
    }

    start_trace_index = tr.entityNum as c_int;

    tr = trap::Trace(
        &(*gWPArray[end as usize]).origin,
        &vec3_origin,
        &vec3_origin,
        &(*gWPArray[start as usize]).origin,
        ENTITYNUM_NONE,
        MASK_SOLID,
    );

    if tr.fraction == 1.0 {
        return 0;
    }

    if start_trace_index == tr.entityNum as c_int {
        return 1;
    }

    0
}

/// `int ConnectTrail(int startindex, int endindex, qboolean behindTheScenes)` (ai_wpnav.c:1000) —
/// the node-branching trail repair: flood a scratch node grid out from waypoint `startindex` in
/// `branchDistance` steps until it reaches near `endindex`, then walk the success chain back and
/// drop intermediate waypoints via `CreateNewWP_InTrail`. On failure it flags the pair one-way.
/// For `g_RMG` it short-circuits to just flagging the pair one-way.
pub unsafe fn ConnectTrail(startindex: c_int, endindex: c_int, behindTheScenes: qboolean) -> c_int {
    let mut foundit: c_int;
    let mut cancontinue: c_int;
    let mut i: c_int;
    let mut failsafe: c_int;
    let mut successnodeindex: c_int;
    let insertindex: c_int;
    let mut prenodestart: c_int;
    let mut extendednodes: [u8; MAX_NODETABLE_SIZE] = [0; MAX_NODETABLE_SIZE]; //for storing checked nodes and not trying to extend them each a bazillion times
    let mut fvecmeas: f32;
    let baseheight: f32;
    let mut branchDistance: f32;
    let mut maxDistFactor: f32 = 256.0;
    let mut a: vec3_t = [0.0; 3];
    let mut startplace: vec3_t = [0.0; 3];
    let mut starttrace: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut testspot: vec3_t = [0.0; 3];
    let mut validspotpos: vec3_t = [0.0; 3];
    let mut tr;

    if g_RMG.integer != 0 {
        //this might be temporary. Or not.
        if (*gWPArray[startindex as usize]).flags & WPFLAG_NEVERONEWAY == 0
            && (*gWPArray[endindex as usize]).flags & WPFLAG_NEVERONEWAY == 0
        {
            (*gWPArray[startindex as usize]).flags |= WPFLAG_ONEWAY_FWD;
            (*gWPArray[endindex as usize]).flags |= WPFLAG_ONEWAY_BACK;
        }
        return 0;
    }

    if g_RMG.integer == 0 {
        branchDistance = TABLE_BRANCH_DISTANCE as f32;
    } else {
        branchDistance = 512.0; //be less precise here, terrain is fairly broad, and we don't want to take an hour precalculating
    }

    if g_RMG.integer != 0 {
        maxDistFactor = 700.0;
    }

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = 0.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 0.0;

    nodenum = 0;
    foundit = 0;

    i = 0;

    successnodeindex = 0;

    while i < MAX_NODETABLE_SIZE as c_int {
        //clear it out before using it
        nodetable[i as usize].flags = 0;
        //		nodetable[i].index = 0;
        nodetable[i as usize].inuse = 0;
        nodetable[i as usize].neighbornum = 0;
        nodetable[i as usize].origin[0] = 0.0;
        nodetable[i as usize].origin[1] = 0.0;
        nodetable[i as usize].origin[2] = 0.0;
        nodetable[i as usize].weight = 0.0;

        extendednodes[i as usize] = 0;

        i += 1;
    }

    i = 0;

    if behindTheScenes == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Point {startindex} is not connected to {endindex} - Repairing...\n"
        ));
    }

    VectorCopy(&(*gWPArray[startindex as usize]).origin, &mut startplace);

    VectorCopy(&startplace, &mut starttrace);

    starttrace[2] -= 4096.0;

    tr = trap::Trace(
        &startplace,
        &vec3_origin,
        &vec3_origin,
        &starttrace,
        ENTITYNUM_NONE,
        MASK_SOLID,
    );

    baseheight = startplace[2] - tr.endpos[2];

    cancontinue = 1;

    VectorCopy(&startplace, &mut nodetable[nodenum as usize].origin);
    nodetable[nodenum as usize].weight = 1.0;
    nodetable[nodenum as usize].inuse = 1;
    //	nodetable[nodenum].index = nodenum;
    nodenum += 1;

    while nodenum < MAX_NODETABLE_SIZE as c_int && foundit == 0 && cancontinue != 0 {
        if g_RMG.integer != 0 {
            //adjust the branch distance dynamically depending on the distance from the start and end points.
            let mut startDist: vec3_t = [0.0; 3];
            let mut endDist: vec3_t = [0.0; 3];
            let startDistf: f32;
            let endDistf: f32;

            VectorSubtract(
                &nodetable[(nodenum - 1) as usize].origin,
                &(*gWPArray[startindex as usize]).origin,
                &mut startDist,
            );
            VectorSubtract(
                &nodetable[(nodenum - 1) as usize].origin,
                &(*gWPArray[endindex as usize]).origin,
                &mut endDist,
            );

            startDistf = VectorLength(&startDist);
            endDistf = VectorLength(&endDist);

            if startDistf < 64.0 || endDistf < 64.0 {
                branchDistance = 64.0;
            } else if startDistf < 128.0 || endDistf < 128.0 {
                branchDistance = 128.0;
            } else if startDistf < 256.0 || endDistf < 256.0 {
                branchDistance = 256.0;
            } else if startDistf < 512.0 || endDistf < 512.0 {
                branchDistance = 512.0;
            } else {
                branchDistance = 800.0;
            }
        }
        cancontinue = 0;
        i = 0;
        prenodestart = nodenum;

        while i < prenodestart {
            if extendednodes[i as usize] != 1 {
                VectorSubtract(
                    &(*gWPArray[endindex as usize]).origin,
                    &nodetable[i as usize].origin,
                    &mut a,
                );
                fvecmeas = VectorLength(&a);

                if fvecmeas < 128.0
                    && CanGetToVector(
                        &(*gWPArray[endindex as usize]).origin,
                        &nodetable[i as usize].origin,
                        &mins,
                        &maxs,
                    ) != 0
                {
                    foundit = 1;
                    successnodeindex = i;
                    break;
                }

                // ---- +X branch ----
                VectorCopy(&nodetable[i as usize].origin, &mut testspot);
                testspot[0] += branchDistance;
                VectorCopy(&testspot, &mut starttrace);
                starttrace[2] -= 4096.0;
                tr = trap::Trace(&testspot, &vec3_origin, &vec3_origin, &starttrace, ENTITYNUM_NONE, MASK_SOLID);
                testspot[2] = tr.endpos[2] + baseheight;
                if NodeHere(&testspot) == 0
                    && tr.startsolid == 0
                    && tr.allsolid == 0
                    && CanGetToVector(&nodetable[i as usize].origin, &testspot, &mins, &maxs) != 0
                {
                    VectorCopy(&testspot, &mut nodetable[nodenum as usize].origin);
                    nodetable[nodenum as usize].inuse = 1;
                    nodetable[nodenum as usize].weight = nodetable[i as usize].weight + 1.0;
                    nodetable[nodenum as usize].neighbornum = i;
                    if (nodetable[i as usize].origin[2] - nodetable[nodenum as usize].origin[2]) > 50.0 {
                        //if there's a big drop, make sure we know we can't just magically fly back up
                        nodetable[nodenum as usize].flags = WPFLAG_ONEWAY_FWD;
                    }
                    nodenum += 1;
                    cancontinue = 1;
                }
                if nodenum >= MAX_NODETABLE_SIZE as c_int {
                    break; //failure
                }

                // ---- -X branch ----
                VectorCopy(&nodetable[i as usize].origin, &mut testspot);
                testspot[0] -= branchDistance;
                VectorCopy(&testspot, &mut starttrace);
                starttrace[2] -= 4096.0;
                tr = trap::Trace(&testspot, &vec3_origin, &vec3_origin, &starttrace, ENTITYNUM_NONE, MASK_SOLID);
                testspot[2] = tr.endpos[2] + baseheight;
                if NodeHere(&testspot) == 0
                    && tr.startsolid == 0
                    && tr.allsolid == 0
                    && CanGetToVector(&nodetable[i as usize].origin, &testspot, &mins, &maxs) != 0
                {
                    VectorCopy(&testspot, &mut nodetable[nodenum as usize].origin);
                    nodetable[nodenum as usize].inuse = 1;
                    nodetable[nodenum as usize].weight = nodetable[i as usize].weight + 1.0;
                    nodetable[nodenum as usize].neighbornum = i;
                    if (nodetable[i as usize].origin[2] - nodetable[nodenum as usize].origin[2]) > 50.0 {
                        nodetable[nodenum as usize].flags = WPFLAG_ONEWAY_FWD;
                    }
                    nodenum += 1;
                    cancontinue = 1;
                }
                if nodenum >= MAX_NODETABLE_SIZE as c_int {
                    break; //failure
                }

                // ---- +Y branch ----
                VectorCopy(&nodetable[i as usize].origin, &mut testspot);
                testspot[1] += branchDistance;
                VectorCopy(&testspot, &mut starttrace);
                starttrace[2] -= 4096.0;
                tr = trap::Trace(&testspot, &vec3_origin, &vec3_origin, &starttrace, ENTITYNUM_NONE, MASK_SOLID);
                testspot[2] = tr.endpos[2] + baseheight;
                if NodeHere(&testspot) == 0
                    && tr.startsolid == 0
                    && tr.allsolid == 0
                    && CanGetToVector(&nodetable[i as usize].origin, &testspot, &mins, &maxs) != 0
                {
                    VectorCopy(&testspot, &mut nodetable[nodenum as usize].origin);
                    nodetable[nodenum as usize].inuse = 1;
                    nodetable[nodenum as usize].weight = nodetable[i as usize].weight + 1.0;
                    nodetable[nodenum as usize].neighbornum = i;
                    if (nodetable[i as usize].origin[2] - nodetable[nodenum as usize].origin[2]) > 50.0 {
                        nodetable[nodenum as usize].flags = WPFLAG_ONEWAY_FWD;
                    }
                    nodenum += 1;
                    cancontinue = 1;
                }
                if nodenum >= MAX_NODETABLE_SIZE as c_int {
                    break; //failure
                }

                // ---- -Y branch ----
                VectorCopy(&nodetable[i as usize].origin, &mut testspot);
                testspot[1] -= branchDistance;
                VectorCopy(&testspot, &mut starttrace);
                starttrace[2] -= 4096.0;
                tr = trap::Trace(&testspot, &vec3_origin, &vec3_origin, &starttrace, ENTITYNUM_NONE, MASK_SOLID);
                testspot[2] = tr.endpos[2] + baseheight;
                if NodeHere(&testspot) == 0
                    && tr.startsolid == 0
                    && tr.allsolid == 0
                    && CanGetToVector(&nodetable[i as usize].origin, &testspot, &mins, &maxs) != 0
                {
                    VectorCopy(&testspot, &mut nodetable[nodenum as usize].origin);
                    nodetable[nodenum as usize].inuse = 1;
                    nodetable[nodenum as usize].weight = nodetable[i as usize].weight + 1.0;
                    nodetable[nodenum as usize].neighbornum = i;
                    if (nodetable[i as usize].origin[2] - nodetable[nodenum as usize].origin[2]) > 50.0 {
                        nodetable[nodenum as usize].flags = WPFLAG_ONEWAY_FWD;
                    }
                    nodenum += 1;
                    cancontinue = 1;
                }
                if nodenum >= MAX_NODETABLE_SIZE as c_int {
                    break; //failure
                }

                extendednodes[i as usize] = 1;
            }

            i += 1;
        }
    }

    if foundit == 0 {
        // _DEBUG would always print this; release only when !behindTheScenes
        if behindTheScenes == 0 {
            G_Printf(&format!(
                "{S_COLOR_RED}Could not link {startindex} to {endindex}, unreachable by node branching.\n"
            ));
        }
        (*gWPArray[startindex as usize]).flags |= WPFLAG_ONEWAY_FWD;
        (*gWPArray[endindex as usize]).flags |= WPFLAG_ONEWAY_BACK;
        if behindTheScenes == 0 {
            G_Printf(&format!(
                "{S_COLOR_YELLOW}Since points cannot be connected, point {startindex} has been flagged as only-forward and point {endindex} has been flagged as only-backward.\n"
            ));
        }

        //The above (commented) code transfers nodes into the "rendered" waypoint array. Strictly for debugging.

        if behindTheScenes == 0 {
            //just use what we have if we're auto-pathing the level
            return 0;
        } else {
            let mut endDist: vec3_t = [0.0; 3];
            let mut nCount: c_int = 0;
            let mut idealNode: c_int = -1;
            let mut bestDist: f32 = 0.0;
            let mut testDist: f32;

            if nodenum <= 10 {
                //not enough to even really bother.
                return 0;
            }

            //Since it failed, find whichever node is closest to the desired end.
            while nCount < nodenum {
                VectorSubtract(
                    &nodetable[nCount as usize].origin,
                    &(*gWPArray[endindex as usize]).origin,
                    &mut endDist,
                );
                testDist = VectorLength(&endDist);
                if idealNode == -1 {
                    idealNode = nCount;
                    bestDist = testDist;
                    nCount += 1;
                    continue;
                }

                if testDist < bestDist {
                    idealNode = nCount;
                    bestDist = testDist;
                }

                nCount += 1;
            }

            if idealNode == -1 {
                return 0;
            }

            successnodeindex = idealNode;
        }
    }

    i = successnodeindex;
    insertindex = startindex;
    failsafe = 0;
    VectorCopy(&(*gWPArray[startindex as usize]).origin, &mut validspotpos);

    while failsafe < MAX_NODETABLE_SIZE as c_int && i < MAX_NODETABLE_SIZE as c_int && i >= 0 {
        VectorSubtract(&validspotpos, &nodetable[i as usize].origin, &mut a);
        if nodetable[nodetable[i as usize].neighbornum as usize].inuse == 0
            || CanGetToVectorTravel(
                &validspotpos,
                /*nodetable[nodetable[i].neighbornum].origin*/ &nodetable[i as usize].origin,
                &mins,
                &maxs,
            ) == 0
            || VectorLength(&a) > maxDistFactor
            || (CanGetToVectorTravel(
                &validspotpos,
                &(*gWPArray[endindex as usize]).origin,
                &mins,
                &maxs,
            ) == 0
                && CanGetToVectorTravel(
                    &nodetable[i as usize].origin,
                    &(*gWPArray[endindex as usize]).origin,
                    &mins,
                    &maxs,
                ) != 0)
        {
            nodetable[i as usize].flags |= WPFLAG_CALCULATED;
            let node_origin = nodetable[i as usize].origin;
            if CreateNewWP_InTrail(&node_origin, nodetable[i as usize].flags, insertindex) == 0 {
                if behindTheScenes == 0 {
                    G_Printf(&format!(
                        "{S_COLOR_RED}Could not link {startindex} to {endindex}, waypoint limit hit.\n"
                    ));
                }
                return 0;
            }

            VectorCopy(&nodetable[i as usize].origin, &mut validspotpos);
        }

        if i == 0 {
            break;
        }

        i = nodetable[i as usize].neighbornum;

        failsafe += 1;
    }

    if behindTheScenes == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Finished connecting {startindex} to {endindex}.\n"
        ));
    }

    1
}

/// `int RepairPaths(qboolean behindTheScenes)` (ai_wpnav.c:1466) — for each adjacent waypoint
/// pair that is too far apart or not mutually visible (and not already one-way / jump / door-
/// blocked / calculated), call [`ConnectTrail`] to branch a node trail between them.
pub unsafe fn RepairPaths(behindTheScenes: qboolean) -> c_int {
    let mut i: c_int;
    #[allow(unused_assignments)]
    let mut preAmount: c_int = 0;
    #[allow(unused_variables, unused_assignments)]
    let mut ctRet: c_int = 0;
    let mut a: vec3_t = [0.0; 3];
    let mut maxDistFactor: f32 = 400.0;

    if gWPNum == 0 {
        return 0;
    }

    if g_RMG.integer != 0 {
        maxDistFactor = 800.0; //higher tolerance here.
    }

    i = 0;

    preAmount = gWPNum;
    let _ = preAmount;

    trap::Cvar_Update(&mut *addr_of_mut!(bot_wp_distconnect));
    trap::Cvar_Update(&mut *addr_of_mut!(bot_wp_visconnect));

    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && !gWPArray[(i + 1) as usize].is_null()
            && (*gWPArray[(i + 1) as usize]).inuse != 0
        {
            VectorSubtract(
                &(*gWPArray[i as usize]).origin,
                &(*gWPArray[(i + 1) as usize]).origin,
                &mut a,
            );

            if (*gWPArray[(i + 1) as usize]).flags & WPFLAG_NOVIS == 0
                && (*gWPArray[(i + 1) as usize]).flags & WPFLAG_JUMP == 0 //don't calculate on jump points because they might not always want to be visible (in cases of force jumping)
                && (*gWPArray[i as usize]).flags & WPFLAG_CALCULATED == 0 //don't calculate it again
                && OpposingEnds(i, i + 1) == 0
                && ((bot_wp_distconnect.value != 0.0 && VectorLength(&a) > maxDistFactor)
                    || (OrgVisible(
                        &(*gWPArray[i as usize]).origin,
                        &(*gWPArray[(i + 1) as usize]).origin,
                        ENTITYNUM_NONE,
                    ) == 0
                        && bot_wp_visconnect.value != 0.0))
                && DoorBlockingSection(i, i + 1) == 0
            {
                ctRet = ConnectTrail(i, i + 1, behindTheScenes);
                let _ = ctRet;

                if gWPNum >= MAX_WPARRAY_SIZE as c_int {
                    //Bad!
                    gWPNum = MAX_WPARRAY_SIZE as c_int;
                    break;
                }

                /*if (!ctRet)
                {
                    return 0;
                }*/ //we still want to write it..
            }
        }

        i += 1;
    }

    1
}

/// `void CalculatePaths(void)` (ai_wpnav.c:1623) — rebuild the neighbor lists for every
/// waypoint: clear old neighbors, then for each pair in range / force-jumpable / box-visible,
/// register a bidirectional-ish neighbor link (marking force-jump links with `999`).
pub unsafe fn CalculatePaths() {
    let mut i: c_int;
    let mut c: c_int;
    let mut forceJumpable: c_int;
    let mut maxNeighborDist: c_int = MAX_NEIGHBOR_LINK_DISTANCE;
    let mut nLDist: f32;
    let mut a: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    if gWPNum == 0 {
        return;
    }

    if g_RMG.integer != 0 {
        maxNeighborDist = (DEFAULT_GRID_SPACING as f32 + (DEFAULT_GRID_SPACING as f32 * 0.5)) as c_int;
    }

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -15.0; //-1
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 15.0; //1

    //now clear out all the neighbor data before we recalculate
    i = 0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null()
            && (*gWPArray[i as usize]).inuse != 0
            && (*gWPArray[i as usize]).neighbornum != 0
        {
            while (*gWPArray[i as usize]).neighbornum >= 0 {
                let nn = (*gWPArray[i as usize]).neighbornum as usize;
                (*gWPArray[i as usize]).neighbors[nn].num = 0;
                (*gWPArray[i as usize]).neighbors[nn].forceJumpTo = 0;
                (*gWPArray[i as usize]).neighbornum -= 1;
            }
            (*gWPArray[i as usize]).neighbornum = 0;
        }

        i += 1;
    }

    i = 0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            c = 0;

            while c < gWPNum {
                if !gWPArray[c as usize].is_null()
                    && (*gWPArray[c as usize]).inuse != 0
                    && i != c
                    && NotWithinRange(i, c) != 0
                {
                    VectorSubtract(
                        &(*gWPArray[i as usize]).origin,
                        &(*gWPArray[c as usize]).origin,
                        &mut a,
                    );

                    nLDist = VectorLength(&a);
                    forceJumpable = CanForceJumpTo(i, c, nLDist);

                    if (nLDist < maxNeighborDist as f32 || forceJumpable != 0)
                        && ((*gWPArray[i as usize]).origin[2] as c_int
                            == (*gWPArray[c as usize]).origin[2] as c_int
                            || forceJumpable != 0)
                        && (OrgVisibleBox(
                            &(*gWPArray[i as usize]).origin,
                            &mins,
                            &maxs,
                            &(*gWPArray[c as usize]).origin,
                            ENTITYNUM_NONE,
                        ) != 0
                            || forceJumpable != 0)
                    {
                        let nn = (*gWPArray[i as usize]).neighbornum as usize;
                        (*gWPArray[i as usize]).neighbors[nn].num = c;
                        if forceJumpable != 0
                            && ((*gWPArray[i as usize]).origin[2] as c_int
                                != (*gWPArray[c as usize]).origin[2] as c_int
                                || nLDist < maxNeighborDist as f32)
                        {
                            (*gWPArray[i as usize]).neighbors[nn].forceJumpTo = 999; //forceJumpable; //FJSR
                        } else {
                            (*gWPArray[i as usize]).neighbors[nn].forceJumpTo = 0;
                        }
                        (*gWPArray[i as usize]).neighbornum += 1;
                    }

                    if (*gWPArray[i as usize]).neighbornum >= MAX_NEIGHBOR_SIZE as c_int {
                        break;
                    }
                }
                c += 1;
            }
        }
        i += 1;
    }
}

/// `void CalculateJumpRoutes(void)` (ai_wpnav.c:1953) — for each jump-flagged waypoint, look at
/// the height drop to its neighbors in the trail and set its `forceJumpTo` tier accordingly.
pub unsafe fn CalculateJumpRoutes() {
    let mut i: c_int = 0;
    let mut nheightdif: f32;
    let mut pheightdif: f32;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            if (*gWPArray[i as usize]).flags & WPFLAG_JUMP != 0 {
                nheightdif = 0.0;
                pheightdif = 0.0;

                (*gWPArray[i as usize]).forceJumpTo = 0;

                if !gWPArray[(i - 1) as usize].is_null()
                    && (*gWPArray[(i - 1) as usize]).inuse != 0
                    && ((*gWPArray[(i - 1) as usize]).origin[2] + 16.0)
                        < (*gWPArray[i as usize]).origin[2]
                {
                    nheightdif =
                        (*gWPArray[i as usize]).origin[2] - (*gWPArray[(i - 1) as usize]).origin[2];
                }

                if !gWPArray[(i + 1) as usize].is_null()
                    && (*gWPArray[(i + 1) as usize]).inuse != 0
                    && ((*gWPArray[(i + 1) as usize]).origin[2] + 16.0)
                        < (*gWPArray[i as usize]).origin[2]
                {
                    pheightdif =
                        (*gWPArray[i as usize]).origin[2] - (*gWPArray[(i + 1) as usize]).origin[2];
                }

                if nheightdif > pheightdif {
                    pheightdif = nheightdif;
                }

                if pheightdif != 0.0 {
                    if pheightdif > 500.0 {
                        (*gWPArray[i as usize]).forceJumpTo = 999; //FORCE_LEVEL_3; //FJSR
                    } else if pheightdif > 256.0 {
                        (*gWPArray[i as usize]).forceJumpTo = 999; //FORCE_LEVEL_2; //FJSR
                    } else if pheightdif > 128.0 {
                        (*gWPArray[i as usize]).forceJumpTo = 999; //FORCE_LEVEL_1; //FJSR
                    }
                }
            }
        }

        i += 1;
    }
}

/// `float botGlobalNavWeaponWeights[WP_NUM_WEAPONS]` (ai_wpnav.c:1796) — per-weapon goal weights
/// used by [`CalculateWeightGoals`] to score waypoints near dropped weapons. The C initializer
/// lists 16 entries (WP_NONE..WP_EMPLACED_GUN); the remaining `WP_NUM_WEAPONS - 16` slots default
/// to zero.
static botGlobalNavWeaponWeights: [f32; WP_NUM_WEAPONS as usize] = {
    let mut w = [0.0f32; WP_NUM_WEAPONS as usize];
    w[0] = 0.0; //WP_NONE,
    w[1] = 0.0; //WP_STUN_BATON,
    w[2] = 0.0; //WP_MELEE
    w[3] = 0.0; //WP_SABER,
    w[4] = 0.0; //WP_BRYAR_PISTOL,
    w[5] = 3.0; //WP_BLASTER,
    w[6] = 5.0; //WP_DISRUPTOR,
    w[7] = 4.0; //WP_BOWCASTER,
    w[8] = 6.0; //WP_REPEATER,
    w[9] = 7.0; //WP_DEMP2,
    w[10] = 8.0; //WP_FLECHETTE,
    w[11] = 9.0; //WP_ROCKET_LAUNCHER,
    w[12] = 3.0; //WP_THERMAL,
    w[13] = 3.0; //WP_TRIP_MINE,
    w[14] = 3.0; //WP_DET_PACK,
    w[15] = 0.0; //WP_EMPLACED_GUN,
    w
};

/// `void CalculateSiegeGoals(void)` (ai_wpnav.c:1734) — for each `info_siege_objective`, chase
/// the target chain to the actual objective entity, find the nearest visible waypoint to its
/// center, and flag that waypoint as an imperial/rebel siege objective.
pub unsafe fn CalculateSiegeGoals() {
    let mut i: c_int = 0;
    let mut looptracker: c_int;
    let mut wpindex: c_int;
    let mut dif: vec3_t = [0.0; 3];
    let mut ent: *mut gentity_t;
    let mut tent: *mut gentity_t;
    let mut t2ent: *mut gentity_t;

    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        tent = null_mut();

        if !ent.is_null()
            && !(*ent).classname.is_null()
            && strcmp((*ent).classname, c"info_siege_objective".as_ptr()) == 0
        {
            tent = ent;
            t2ent = GetObjectThatTargets(tent);
            looptracker = 0;

            while !t2ent.is_null() && looptracker < 2048 {
                //looptracker keeps us from getting stuck in case something is set up weird on this map
                tent = t2ent;
                t2ent = GetObjectThatTargets(tent);
                looptracker += 1;
            }

            if looptracker >= 2048 {
                //something unpleasent has happened
                tent = null_mut();
                break;
            }
        }

        if !tent.is_null() && !ent.is_null() && tent != ent {
            //tent should now be the object attached to the mission objective
            dif[0] = ((*tent).r.absmax[0] + (*tent).r.absmin[0]) / 2.0;
            dif[1] = ((*tent).r.absmax[1] + (*tent).r.absmin[1]) / 2.0;
            dif[2] = ((*tent).r.absmax[2] + (*tent).r.absmin[2]) / 2.0;

            wpindex = GetNearestVisibleWP(&dif, (*tent).s.number);

            if wpindex != -1
                && !gWPArray[wpindex as usize].is_null()
                && (*gWPArray[wpindex as usize]).inuse != 0
            {
                //found the waypoint nearest the center of this objective-related object
                if (*ent).side == SIEGETEAM_TEAM1 {
                    (*gWPArray[wpindex as usize]).flags |= WPFLAG_SIEGE_IMPERIALOBJ;
                } else {
                    (*gWPArray[wpindex as usize]).flags |= WPFLAG_SIEGE_REBELOBJ;
                }

                (*gWPArray[wpindex as usize]).associated_entity = (*tent).s.number;
            }
        }

        i += 1;
    }
}

/// `void CalculateWeightGoals(void)` (ai_wpnav.c:1858) — set waypoint weights based on item /
/// weapon placement: walk all entities, score them by classname/type, and stamp the weight +
/// `WPFLAG_GOALPOINT` onto the nearest visible waypoint. Optionally clears all weights first.
pub unsafe fn CalculateWeightGoals() {
    //set waypoint weights depending on weapon and item placement
    let mut i: c_int = 0;
    let mut wpindex: c_int;
    let mut ent: *mut gentity_t;
    let mut weight: f32;

    trap::Cvar_Update(&mut *addr_of_mut!(bot_wp_clearweight));

    if bot_wp_clearweight.integer != 0 {
        //if set then flush out all weight/goal values before calculating them again
        while i < gWPNum {
            if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
                (*gWPArray[i as usize]).weight = 0.0;

                if (*gWPArray[i as usize]).flags & WPFLAG_GOALPOINT != 0 {
                    (*gWPArray[i as usize]).flags -= WPFLAG_GOALPOINT;
                }
            }

            i += 1;
        }
    }

    i = 0;

    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        weight = 0.0;

        if !ent.is_null() && !(*ent).classname.is_null() {
            if strcmp((*ent).classname, c"item_seeker".as_ptr()) == 0 {
                weight = 2.0;
            } else if strcmp((*ent).classname, c"item_shield".as_ptr()) == 0 {
                weight = 2.0;
            } else if strcmp((*ent).classname, c"item_medpac".as_ptr()) == 0 {
                weight = 2.0;
            } else if strcmp((*ent).classname, c"item_sentry_gun".as_ptr()) == 0 {
                weight = 2.0;
            } else if strcmp((*ent).classname, c"item_force_enlighten_dark".as_ptr()) == 0 {
                weight = 5.0;
            } else if strcmp((*ent).classname, c"item_force_enlighten_light".as_ptr()) == 0 {
                weight = 5.0;
            } else if strcmp((*ent).classname, c"item_force_boon".as_ptr()) == 0 {
                weight = 5.0;
            } else if strcmp((*ent).classname, c"item_ysalimari".as_ptr()) == 0 {
                weight = 2.0;
            } else if !strstr((*ent).classname, c"weapon_".as_ptr()).is_null() && !(*ent).item.is_null() {
                weight = botGlobalNavWeaponWeights[(*(*ent).item).giTag as usize];
            } else if !(*ent).item.is_null() && (*(*ent).item).giType == IT_AMMO {
                weight = 3.0;
            }
        }

        if !ent.is_null() && weight != 0.0 {
            wpindex = GetNearestVisibleWPToItem(&(*ent).s.pos.trBase, (*ent).s.number);

            if wpindex != -1
                && !gWPArray[wpindex as usize].is_null()
                && (*gWPArray[wpindex as usize]).inuse != 0
            {
                //found the waypoint nearest the center of this object
                (*gWPArray[wpindex as usize]).weight = weight;
                (*gWPArray[wpindex as usize]).flags |= WPFLAG_GOALPOINT;
                (*gWPArray[wpindex as usize]).associated_entity = (*ent).s.number;
            }
        }

        i += 1;
    }
}

/// `void FlagObjects(void)` (ai_wpnav.c:2252) — find the CTF red/blue flag entities and flag the
/// nearest reachable waypoint to each (`WPFLAG_RED_FLAG` / `WPFLAG_BLUE_FLAG`), recording the
/// waypoint + entity in the `flagRed`/`oFlagRed`/`eFlagRed` (and blue) globals.
pub unsafe fn FlagObjects() {
    let mut i: c_int = 0;
    let mut bestindex: c_int = 0;
    let mut found: c_int = 0;
    let mut bestdist: f32 = 999999.0;
    let mut tlen: f32;
    let mut flag_red: *mut gentity_t;
    let mut flag_blue: *mut gentity_t;
    let mut ent: *mut gentity_t;
    let mut a: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut tr;

    flag_red = null_mut();
    flag_blue = null_mut();

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -5.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 5.0;

    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && (*ent).inuse != 0 && !(*ent).classname.is_null() {
            if flag_red.is_null() && strcmp((*ent).classname, c"team_CTF_redflag".as_ptr()) == 0 {
                flag_red = ent;
            } else if flag_blue.is_null()
                && strcmp((*ent).classname, c"team_CTF_blueflag".as_ptr()) == 0
            {
                flag_blue = ent;
            }

            if !flag_red.is_null() && !flag_blue.is_null() {
                break;
            }
        }

        i += 1;
    }

    i = 0;

    if flag_red.is_null() || flag_blue.is_null() {
        return;
    }

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            VectorSubtract(
                &(*flag_red).s.pos.trBase,
                &(*gWPArray[i as usize]).origin,
                &mut a,
            );
            tlen = VectorLength(&a);

            if tlen < bestdist {
                tr = trap::Trace(
                    &(*flag_red).s.pos.trBase,
                    &mins,
                    &maxs,
                    &(*gWPArray[i as usize]).origin,
                    (*flag_red).s.number,
                    MASK_SOLID,
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*flag_red).s.number {
                    bestdist = tlen;
                    bestindex = i;
                    found = 1;
                }
            }
        }

        i += 1;
    }

    if found != 0 {
        (*gWPArray[bestindex as usize]).flags |= WPFLAG_RED_FLAG;
        flagRed = gWPArray[bestindex as usize];
        oFlagRed = flagRed;
        eFlagRed = flag_red;
    }

    bestdist = 999999.0;
    bestindex = 0;
    found = 0;
    i = 0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            VectorSubtract(
                &(*flag_blue).s.pos.trBase,
                &(*gWPArray[i as usize]).origin,
                &mut a,
            );
            tlen = VectorLength(&a);

            if tlen < bestdist {
                tr = trap::Trace(
                    &(*flag_blue).s.pos.trBase,
                    &mins,
                    &maxs,
                    &(*gWPArray[i as usize]).origin,
                    (*flag_blue).s.number,
                    MASK_SOLID,
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*flag_blue).s.number {
                    bestdist = tlen;
                    bestindex = i;
                    found = 1;
                }
            }
        }

        i += 1;
    }

    if found != 0 {
        (*gWPArray[bestindex as usize]).flags |= WPFLAG_BLUE_FLAG;
        flagBlue = gWPArray[bestindex as usize];
        oFlagBlue = flagBlue;
        eFlagBlue = flag_blue;
    }
}

/// `int LoadPathData(const char *filename)` (ai_wpnav.c:2007) — read `botroutes/<filename>.wnt`
/// and rebuild the waypoint array from it: parse the optional `levelflags` header, then each
/// space-delimited record (index, flags, weight, origin, neighbor list, disttonext) via
/// [`CreateNewWP_FromObject`]. Returns 1 on success, 2 if the file is missing, 0 on overflow.
pub unsafe fn LoadPathData(filename: *const c_char) -> c_int {
    let f: fileHandle_t;
    let fileString: *mut c_char;
    let currentVar: *mut c_char;
    let len: c_int;
    let mut i: c_int;
    let mut i_cv: c_int;
    #[allow(unused_assignments)]
    let mut nei_num: c_int;

    i = 0;
    #[allow(unused_assignments)]
    {
        i_cv = 0;
    }

    // Com_sprintf(routePath, 1024, "botroutes/%s.wnt\0", filename) — the trap wrapper takes &str.
    let fname = core::ffi::CStr::from_ptr(filename).to_string_lossy().into_owned();
    let routePath = format!("botroutes/{fname}.wnt");

    let (l, fh) = trap::FS_FOpenFile(&routePath, FS_READ);
    len = l;
    f = fh;

    if f == 0 {
        G_Printf(&format!(
            "{S_COLOR_YELLOW}Bot route data not found for {fname}\n"
        ));
        return 2;
    }

    if len >= 524288 {
        G_Printf(&format!("{S_COLOR_RED}Route file exceeds maximum length\n"));
        return 0;
    }

    fileString = B_TempAlloc(524288) as *mut c_char;
    currentVar = B_TempAlloc(2048) as *mut c_char;

    {
        let rbuf = core::slice::from_raw_parts_mut(fileString as *mut u8, len as usize);
        trap::FS_Read(rbuf, f);
    }

    macro_rules! fs {
        ($idx:expr) => {
            *fileString.offset($idx as isize)
        };
    }
    macro_rules! cv {
        ($idx:expr) => {
            *currentVar.offset($idx as isize)
        };
    }

    if fs!(i) == b'l' as c_char {
        //contains a "levelflags" entry..
        let mut readLFlags: [c_char; 64] = [0; 64];
        i_cv = 0;

        while fs!(i) != b' ' as c_char {
            i += 1;
        }
        i += 1;
        while fs!(i) != b'\n' as c_char {
            readLFlags[i_cv as usize] = fs!(i);
            i_cv += 1;
            i += 1;
        }
        readLFlags[i_cv as usize] = 0;
        i += 1;

        gLevelFlags = atoi(readLFlags.as_ptr());
    } else {
        gLevelFlags = 0;
    }

    while i < len {
        i_cv = 0;

        let mut thiswp: wpobject_t = wpobject_t {
            origin: [0.0; 3],
            inuse: 0,
            index: 0,
            weight: 0.0,
            disttonext: 0.0,
            flags: 0,
            associated_entity: ENTITYNUM_NONE as c_int,
            forceJumpTo: 0,
            neighbornum: 0,
            neighbors: [wpneighbor_t { num: 0, forceJumpTo: 0 }; MAX_NEIGHBOR_SIZE],
        };

        nei_num = 0;

        while nei_num < MAX_NEIGHBOR_SIZE as c_int {
            thiswp.neighbors[nei_num as usize].num = 0;
            thiswp.neighbors[nei_num as usize].forceJumpTo = 0;

            nei_num += 1;
        }

        while fs!(i) != b' ' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.index = atoi(currentVar);

        i_cv = 0;
        i += 1;

        while fs!(i) != b' ' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.flags = atoi(currentVar);

        i_cv = 0;
        i += 1;

        while fs!(i) != b' ' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.weight = atof(currentVar) as f32;

        i_cv = 0;
        i += 1;
        i += 1;

        while fs!(i) != b' ' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.origin[0] = atof(currentVar) as f32;

        i_cv = 0;
        i += 1;

        while fs!(i) != b' ' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.origin[1] = atof(currentVar) as f32;

        i_cv = 0;
        i += 1;

        while fs!(i) != b')' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.origin[2] = atof(currentVar) as f32;

        i += 4;

        while fs!(i) != b'}' as c_char {
            i_cv = 0;
            while fs!(i) != b' ' as c_char && fs!(i) != b'-' as c_char {
                cv!(i_cv) = fs!(i);
                i_cv += 1;
                i += 1;
            }
            cv!(i_cv) = b'\0' as c_char;

            thiswp.neighbors[thiswp.neighbornum as usize].num = atoi(currentVar);

            if fs!(i) == b'-' as c_char {
                i_cv = 0;
                i += 1;

                while fs!(i) != b' ' as c_char {
                    cv!(i_cv) = fs!(i);
                    i_cv += 1;
                    i += 1;
                }
                cv!(i_cv) = b'\0' as c_char;

                thiswp.neighbors[thiswp.neighbornum as usize].forceJumpTo = 999; //atoi(currentVar); //FJSR
            } else {
                thiswp.neighbors[thiswp.neighbornum as usize].forceJumpTo = 0;
            }

            thiswp.neighbornum += 1;

            i += 1;
        }

        i_cv = 0;
        i += 1;
        i += 1;

        while fs!(i) != b'\n' as c_char {
            cv!(i_cv) = fs!(i);
            i_cv += 1;
            i += 1;
        }
        cv!(i_cv) = b'\0' as c_char;

        thiswp.disttonext = atof(currentVar) as f32;

        CreateNewWP_FromObject(&mut thiswp);
        i += 1;
    }

    B_TempFree(524288); //fileString
    B_TempFree(2048); //currentVar

    trap::FS_FCloseFile(f);

    if g_gametype.integer == GT_SIEGE {
        CalculateSiegeGoals();
    }

    CalculateWeightGoals();
    //calculate weights for idle activity goals when
    //the bot has absolutely nothing else to do

    CalculateJumpRoutes();
    //Look at jump points and mark them as requiring
    //force jumping as needed

    1
}

/// Read the current NUL-terminated contents of a `B_TempAlloc` C buffer as an owned `String`,
/// for reproducing the C `Com_sprintf(buf, sz, "%s...", buf, ...)` self-concatenation idiom (the
/// rendered `%s` argument is the buffer's prior contents).
unsafe fn cbuf_str(buf: *const c_char) -> String {
    core::ffi::CStr::from_ptr(buf).to_string_lossy().into_owned()
}

/// `int SavePathData(const char *filename)` (ai_wpnav.c:2372) — repair, connect and flag the
/// current waypoint array, then serialize it to `botroutes/<filename>.wnt`. Returns 1 on success,
/// 0 on any failure (no waypoints, file open error, or unrepairable paths).
///
/// DEVIATION: the C reads `storeString` via `%s` before its first write (relying on whatever the
/// temp arena holds); we zero `storeString[0]` up front so the self-concat reads a valid empty
/// C string instead of uninitialized memory. Output is identical for well-formed runs.
pub unsafe fn SavePathData(filename: *const c_char) -> c_int {
    let f: fileHandle_t;
    let fileString: *mut c_char;
    let storeString: *mut c_char;
    let mut a: vec3_t = [0.0; 3];
    let mut flLen: f32;
    let mut i: c_int;
    let mut n: c_int;

    i = 0;

    if gWPNum == 0 {
        return 0;
    }

    // Com_sprintf(routePath, 1024, "botroutes/%s.wnt\0", filename)
    let fname = core::ffi::CStr::from_ptr(filename).to_string_lossy().into_owned();
    let routePath = format!("botroutes/{fname}.wnt");

    let (_l, fh) = trap::FS_FOpenFile(&routePath, FS_WRITE);
    f = fh;

    if f == 0 {
        G_Printf(&format!(
            "{S_COLOR_RED}ERROR: Could not open file to write path data\n"
        ));
        return 0;
    }

    if RepairPaths(QFALSE) == 0 {
        //check if we can see all waypoints from the last. If not, try to branch over.
        trap::FS_FCloseFile(f);
        return 0;
    }

    CalculatePaths(); //make everything nice and connected before saving

    FlagObjects(); //currently only used for flagging waypoints nearest CTF flags

    fileString = B_TempAlloc(524288) as *mut c_char;
    storeString = B_TempAlloc(4096) as *mut c_char;
    *storeString = 0; // see DEVIATION

    Com_sprintf(
        fileString,
        524288,
        format_args!(
            "{} {} {:.6} ({:.6} {:.6} {:.6}) {{ ",
            (*gWPArray[i as usize]).index,
            (*gWPArray[i as usize]).flags,
            (*gWPArray[i as usize]).weight,
            (*gWPArray[i as usize]).origin[0],
            (*gWPArray[i as usize]).origin[1],
            (*gWPArray[i as usize]).origin[2],
        ),
    );

    n = 0;

    while n < (*gWPArray[i as usize]).neighbornum {
        if (*gWPArray[i as usize]).neighbors[n as usize].forceJumpTo != 0 {
            let prev = cbuf_str(storeString);
            Com_sprintf(
                storeString,
                4096,
                format_args!(
                    "{}{}-{} ",
                    prev,
                    (*gWPArray[i as usize]).neighbors[n as usize].num,
                    (*gWPArray[i as usize]).neighbors[n as usize].forceJumpTo,
                ),
            );
        } else {
            let prev = cbuf_str(storeString);
            Com_sprintf(
                storeString,
                4096,
                format_args!("{}{} ", prev, (*gWPArray[i as usize]).neighbors[n as usize].num),
            );
        }
        n += 1;
    }

    if !gWPArray[(i + 1) as usize].is_null()
        && (*gWPArray[(i + 1) as usize]).inuse != 0
        && (*gWPArray[(i + 1) as usize]).index != 0
    {
        VectorSubtract(
            &(*gWPArray[i as usize]).origin,
            &(*gWPArray[(i + 1) as usize]).origin,
            &mut a,
        );
        flLen = VectorLength(&a);
    } else {
        flLen = 0.0;
    }

    (*gWPArray[i as usize]).disttonext = flLen;

    {
        let prev = cbuf_str(fileString);
        Com_sprintf(fileString, 524288, format_args!("{}}} {:.6}\n", prev, flLen));
    }

    i += 1;

    while i < gWPNum {
        Com_sprintf(
            storeString,
            4096,
            format_args!(
                "{} {} {:.6} ({:.6} {:.6} {:.6}) {{ ",
                (*gWPArray[i as usize]).index,
                (*gWPArray[i as usize]).flags,
                (*gWPArray[i as usize]).weight,
                (*gWPArray[i as usize]).origin[0],
                (*gWPArray[i as usize]).origin[1],
                (*gWPArray[i as usize]).origin[2],
            ),
        );

        n = 0;

        while n < (*gWPArray[i as usize]).neighbornum {
            if (*gWPArray[i as usize]).neighbors[n as usize].forceJumpTo != 0 {
                let prev = cbuf_str(storeString);
                Com_sprintf(
                    storeString,
                    4096,
                    format_args!(
                        "{}{}-{} ",
                        prev,
                        (*gWPArray[i as usize]).neighbors[n as usize].num,
                        (*gWPArray[i as usize]).neighbors[n as usize].forceJumpTo,
                    ),
                );
            } else {
                let prev = cbuf_str(storeString);
                Com_sprintf(
                    storeString,
                    4096,
                    format_args!("{}{} ", prev, (*gWPArray[i as usize]).neighbors[n as usize].num),
                );
            }
            n += 1;
        }

        if !gWPArray[(i + 1) as usize].is_null()
            && (*gWPArray[(i + 1) as usize]).inuse != 0
            && (*gWPArray[(i + 1) as usize]).index != 0
        {
            VectorSubtract(
                &(*gWPArray[i as usize]).origin,
                &(*gWPArray[(i + 1) as usize]).origin,
                &mut a,
            );
            flLen = VectorLength(&a);
        } else {
            flLen = 0.0;
        }

        (*gWPArray[i as usize]).disttonext = flLen;

        {
            let prev = cbuf_str(storeString);
            Com_sprintf(storeString, 4096, format_args!("{}}} {:.6}\n", prev, flLen));
        }

        strcat(fileString, storeString);

        i += 1;
    }

    {
        let wbuf = core::slice::from_raw_parts(fileString as *const u8, strlen(fileString));
        trap::FS_Write(wbuf, f);
    }

    B_TempFree(524288); //fileString
    B_TempFree(4096); //storeString

    trap::FS_FCloseFile(f);

    G_Printf("Path data has been saved and updated. You may need to restart the level for some things to be properly calculated.\n");

    1
}

/// `#define MAX_SPAWNPOINT_ARRAY 64` (ai_wpnav.c:2505)
const MAX_SPAWNPOINT_ARRAY: usize = 64;
/// `int gSpawnPointNum = 0;` (ai_wpnav.c:2506)
pub(crate) static mut gSpawnPointNum: c_int = 0;
/// `gentity_t *gSpawnPoints[MAX_SPAWNPOINT_ARRAY];` (ai_wpnav.c:2507)
pub(crate) static mut gSpawnPoints: [*mut gentity_t; MAX_SPAWNPOINT_ARRAY] =
    [null_mut(); MAX_SPAWNPOINT_ARRAY];

/// `int G_RecursiveConnection(int start, int end, int weight, qboolean traceCheck, float baseHeight)`
/// (ai_wpnav.c:2616) — depth-first flood across the RMG grid from node `start` toward `end`,
/// stamping increasing weights and (optionally) trace-checking inter-node visibility. Returns the
/// reached `end` index, or `-1` if unreachable from this branch.
pub unsafe fn G_RecursiveConnection(
    start: c_int,
    end: c_int,
    weight: c_int,
    traceCheck: qboolean,
    baseHeight: f32,
) -> c_int {
    let mut indexDirections: [c_int; 4] = [0; 4]; //0 == down, 1 == up, 2 == left, 3 == right
    let mut recursiveIndex: c_int = -1;
    let mut i: c_int;
    let mut passWeight: c_int = weight;
    let mut givenXY: vec2_t = [0.0; 2];
    let mut tr;

    passWeight += 1;
    nodetable[start as usize].weight = passWeight as f32;

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[0] -= DEFAULT_GRID_SPACING as f32;
    indexDirections[0] = G_NodeMatchingXY(givenXY[0], givenXY[1]);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[0] += DEFAULT_GRID_SPACING as f32;
    indexDirections[1] = G_NodeMatchingXY(givenXY[0], givenXY[1]);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[1] -= DEFAULT_GRID_SPACING as f32;
    indexDirections[2] = G_NodeMatchingXY(givenXY[0], givenXY[1]);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[1] += DEFAULT_GRID_SPACING as f32;
    indexDirections[3] = G_NodeMatchingXY(givenXY[0], givenXY[1]);

    i = 0;
    while i < 4 {
        if indexDirections[i as usize] == end {
            //we've connected all the way to the destination.
            return indexDirections[i as usize];
        }

        if indexDirections[i as usize] != -1 && nodetable[indexDirections[i as usize] as usize].flags != 0 {
            //this point is already used, so it's not valid.
            indexDirections[i as usize] = -1;
        } else if indexDirections[i as usize] != -1 {
            //otherwise mark it as used.
            nodetable[indexDirections[i as usize] as usize].flags = 1;
        }

        if indexDirections[i as usize] != -1 && traceCheck != 0 {
            //if we care about trace visibility between nodes, perform the check and mark as not valid if the trace isn't clear.
            tr = trap::Trace(
                &nodetable[start as usize].origin,
                &vec3_origin,
                &vec3_origin,
                &nodetable[indexDirections[i as usize] as usize].origin,
                ENTITYNUM_NONE,
                CONTENTS_SOLID,
            );

            if tr.fraction != 1.0 {
                indexDirections[i as usize] = -1;
            }
        }

        if indexDirections[i as usize] != -1 {
            //it's still valid, so keep connecting via this point.
            recursiveIndex =
                G_RecursiveConnection(indexDirections[i as usize], end, passWeight, traceCheck, baseHeight);
        }

        if recursiveIndex != -1 {
            //the result of the recursive check was valid, so return it.
            return recursiveIndex;
        }

        i += 1;
    }

    recursiveIndex
}

/// `qboolean G_BackwardAttachment(int start, int finalDestination, int insertAfter)`
/// (ai_wpnav.c:2981) — after [`G_RecursiveConnection`] weights the grid, walk back from `start`
/// toward `finalDestination` along the lowest-weight neighbors, dropping a waypoint at each step
/// (`CreateNewWP_InsertUnder`) until the original point is reached. Returns `qtrue` on success.
pub unsafe fn G_BackwardAttachment(
    start: c_int,
    finalDestination: c_int,
    insertAfter: c_int,
) -> qboolean {
    //After creating a node path between 2 points, this function links the 2 points with actual waypoint data.
    let mut indexDirections: [c_int; 4] = [0; 4]; //0 == down, 1 == up, 2 == left, 3 == right
    let mut i: c_int = 0;
    let mut lowestWeight: c_int = 9999;
    let mut desiredIndex: c_int = -1;
    let mut givenXY: vec2_t = [0.0; 2];

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[0] -= DEFAULT_GRID_SPACING as f32;
    indexDirections[0] = G_NodeMatchingXY_BA(givenXY[0] as c_int, givenXY[1] as c_int, finalDestination);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[0] += DEFAULT_GRID_SPACING as f32;
    indexDirections[1] = G_NodeMatchingXY_BA(givenXY[0] as c_int, givenXY[1] as c_int, finalDestination);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[1] -= DEFAULT_GRID_SPACING as f32;
    indexDirections[2] = G_NodeMatchingXY_BA(givenXY[0] as c_int, givenXY[1] as c_int, finalDestination);

    givenXY[0] = nodetable[start as usize].origin[0];
    givenXY[1] = nodetable[start as usize].origin[1];
    givenXY[1] += DEFAULT_GRID_SPACING as f32;
    indexDirections[3] = G_NodeMatchingXY_BA(givenXY[0] as c_int, givenXY[1] as c_int, finalDestination);

    while i < 4 {
        if indexDirections[i as usize] != -1 {
            if indexDirections[i as usize] == finalDestination {
                //hooray, we've found the original point and linked all the way back to it.
                let o_start = nodetable[start as usize].origin;
                CreateNewWP_InsertUnder(&o_start, 0, insertAfter);
                let o_dir = nodetable[indexDirections[i as usize] as usize].origin;
                CreateNewWP_InsertUnder(&o_dir, 0, insertAfter);
                return QTRUE;
            }

            if (nodetable[indexDirections[i as usize] as usize].weight as c_int) < lowestWeight
                && nodetable[indexDirections[i as usize] as usize].weight != 0.0
                && nodetable[indexDirections[i as usize] as usize].flags == 0
            /*&& (nodetable[indexDirections[i]].origin[2]-64 < nodetable[start].origin[2])*/
            {
                desiredIndex = indexDirections[i as usize];
                lowestWeight = nodetable[indexDirections[i as usize] as usize].weight as c_int;
            }
        }
        i += 1;
    }

    if desiredIndex != -1 {
        //Create a waypoint here, and then recursively call this function for the next neighbor with the lowest weight.
        if gWPNum < 3900 {
            let o_start = nodetable[start as usize].origin;
            CreateNewWP_InsertUnder(&o_start, 0, insertAfter);
        } else {
            return QFALSE;
        }

        nodetable[start as usize].flags = 1;
        return G_BackwardAttachment(desiredIndex, finalDestination, insertAfter);
    }

    QFALSE
}

/// `void G_RMGPathing(void)` (ai_wpnav.c:3055) — generate waypoint information on-the-fly for a
/// random mission (RMG) terrain: drop a grid of nodes on the terrain, then for each consecutive
/// spawn point recursively connect their nearest nodes and lay down a real waypoint trail.
///
/// The `PATH_TIME_DEBUG` (`_DEBUG`), `PAINFULLY_DEBUGGING_THROUGH_VM`, `ASCII_ART_DEBUG` and
/// `DEBUG_NODE_FILE` blocks are compiled out in the release build and omitted here.
pub unsafe fn G_RMGPathing() {
    //Generate waypoint information on-the-fly for the random mission.
    let mut placeX: f32;
    let mut placeY: f32;
    let placeZ: f32;
    let mut i: c_int = 0;
    let gridSpacing: c_int = DEFAULT_GRID_SPACING;
    let mut nearestIndex: c_int;
    let mut nearestIndexForNext: c_int;
    let mut downVec: vec3_t = [0.0; 3];
    let mut trMins: vec3_t = [0.0; 3];
    let mut trMaxs: vec3_t = [0.0; 3];
    let mut tr;
    let terrain: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, classname), c"terrain".as_ptr());

    if terrain.is_null() || (*terrain).inuse == 0 || (*terrain).s.eType != ET_TERRAIN {
        G_Printf("Error: RMG with no terrain!\n");
        return;
    }

    nodenum = 0;
    write_bytes(addr_of_mut!(nodetable) as *mut u8, 0, size_of::<[nodeobject_t; MAX_NODETABLE_SIZE]>());

    VectorSet(&mut trMins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
    VectorSet(&mut trMaxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

    placeX = (*terrain).r.absmin[0];
    placeY = (*terrain).r.absmin[1];
    placeZ = (*terrain).r.absmax[2] - 400.0;

    //skim through the entirety of the terrain limits and drop nodes, removing
    //nodes that start in solid or fall too high on the terrain.
    while placeY < (*terrain).r.absmax[1] {
        if nodenum >= MAX_NODETABLE_SIZE as c_int {
            break;
        }

        while placeX < (*terrain).r.absmax[0] {
            if nodenum >= MAX_NODETABLE_SIZE as c_int {
                break;
            }

            nodetable[nodenum as usize].origin[0] = placeX;
            nodetable[nodenum as usize].origin[1] = placeY;
            nodetable[nodenum as usize].origin[2] = placeZ;

            VectorCopy(&nodetable[nodenum as usize].origin, &mut downVec);
            downVec[2] -= 3000.0;
            tr = trap::Trace(
                &nodetable[nodenum as usize].origin,
                &trMins,
                &trMaxs,
                &downVec,
                ENTITYNUM_NONE,
                MASK_SOLID,
            );

            if (tr.entityNum as c_int >= ENTITYNUM_WORLD
                || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).s.eType == ET_TERRAIN)
                && tr.endpos[2] < (*terrain).r.absmin[2] + 750.0
            {
                //only drop nodes on terrain directly
                VectorCopy(&tr.endpos, &mut nodetable[nodenum as usize].origin);
                nodenum += 1;
            } else {
                VectorClear(&mut nodetable[nodenum as usize].origin);
            }

            placeX += gridSpacing as f32;
        }

        placeX = (*terrain).r.absmin[0];
        placeY += gridSpacing as f32;
    }

    G_NodeClearForNext();

    //The grid has been placed down, now use it to connect the points in the level.
    while i < gSpawnPointNum - 1 {
        if gSpawnPoints[i as usize].is_null()
            || (*gSpawnPoints[i as usize]).inuse == 0
            || gSpawnPoints[(i + 1) as usize].is_null()
            || (*gSpawnPoints[(i + 1) as usize]).inuse == 0
        {
            i += 1;
            continue;
        }

        nearestIndex = G_NearestNodeToPoint(&(*gSpawnPoints[i as usize]).s.origin);
        nearestIndexForNext = G_NearestNodeToPoint(&(*gSpawnPoints[(i + 1) as usize]).s.origin);

        if nearestIndex == -1 || nearestIndexForNext == -1 {
            //Looks like there is no grid data near one of the points. Ideally, this will never happen.
            i += 1;
            continue;
        }

        if nearestIndex == nearestIndexForNext {
            //Two spawn points on top of each other? We don't need to do both points, keep going until the next differs.
            i += 1;
            continue;
        }

        //For now I am going to branch out mindlessly, but I will probably want to use some sort of A* algorithm
        //here to lessen the time taken.
        if G_RecursiveConnection(nearestIndex, nearestIndexForNext, 0, QTRUE, (*terrain).r.absmin[2])
            != nearestIndexForNext
        {
            //failed to branch to where we want. Oh well, try it without trace checks.
            G_NodeClearForNext();

            if G_RecursiveConnection(nearestIndex, nearestIndexForNext, 0, QFALSE, (*terrain).r.absmin[2])
                != nearestIndexForNext
            {
                //still failed somehow. Just disregard this point.
                G_NodeClearForNext();
                i += 1;
                continue;
            }
        }

        //Now our node array is set up so that highest reasonable weight is the destination node, and 2 is next to the original index,
        //so trace back to that point.
        G_NodeClearFlags();

        if G_BackwardAttachment(nearestIndexForNext, nearestIndex, gWPNum - 1) != 0 {
            //successfully connected the trail from nearestIndex to nearestIndexForNext
            if (*gSpawnPoints[(i + 1) as usize]).inuse != 0
                && !(*gSpawnPoints[(i + 1) as usize]).item.is_null()
                && (*(*gSpawnPoints[(i + 1) as usize]).item).giType == IT_TEAM
            {
                //This point is actually a CTF flag.
                if (*(*gSpawnPoints[(i + 1) as usize]).item).giTag == PW_REDFLAG
                    || (*(*gSpawnPoints[(i + 1) as usize]).item).giTag == PW_BLUEFLAG
                {
                    //Place a waypoint on the flag next in the trail, so the nearest grid point will link to it.
                    let o = (*gSpawnPoints[(i + 1) as usize]).s.origin;
                    CreateNewWP_InsertUnder(&o, WPFLAG_NEVERONEWAY, gWPNum - 1);
                }
            }
        } else {
            break;
        }

        G_NodeClearForNext();
        i += 1;
    }

    RepairPaths(QTRUE); //this has different behaviour for RMG and will just flag all points one way that don't trace to each other.
}

/// `void BeginAutoPathRoutine(void)` (ai_wpnav.c:3254) — entry point for RMG levels: collect the
/// deathmatch spawn points (and CTF flags) into `gSpawnPoints`, run [`G_RMGPathing`], push the
/// generated waypoints into the engine, flag objects, compute inter-waypoint distances, and drop
/// the dummy point used as an insert anchor.
pub unsafe fn BeginAutoPathRoutine() {
    //Called for RMG levels.
    let mut i: c_int = 0;
    let mut ent: *mut gentity_t;
    let mut v: vec3_t = [0.0; 3];

    gSpawnPointNum = 0;

    CreateNewWP(&vec3_origin, 0); //create a dummy waypoint to insert under

    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && (*ent).inuse != 0
            && !(*ent).classname.is_null()
            && *(*ent).classname != 0
            && Q_stricmp((*ent).classname, c"info_player_deathmatch".as_ptr()) == 0
        {
            if (*ent).s.origin[2] < 1280.0 {
                //h4x
                gSpawnPoints[gSpawnPointNum as usize] = ent;
                gSpawnPointNum += 1;
            }
        } else if !ent.is_null()
            && (*ent).inuse != 0
            && !(*ent).item.is_null()
            && (*(*ent).item).giType == IT_TEAM
            && ((*(*ent).item).giTag == PW_REDFLAG || (*(*ent).item).giTag == PW_BLUEFLAG)
        {
            //also make it path to flags in CTF.
            gSpawnPoints[gSpawnPointNum as usize] = ent;
            gSpawnPointNum += 1;
        }

        i += 1;
    }

    if gSpawnPointNum < 1 {
        return;
    }

    G_RMGPathing();

    //rww - Using a faster in-engine version because we're having to wait for this stuff to get done as opposed to just saving it once.
    trap::Bot_UpdateWaypoints(gWPNum, addr_of_mut!(gWPArray) as *mut *mut core::ffi::c_void);
    trap::Bot_CalculatePaths(g_RMG.integer);
    //CalculatePaths(); //make everything nice and connected

    FlagObjects(); //currently only used for flagging waypoints nearest CTF flags

    i = 0;

    while i < gWPNum - 1 {
        //disttonext is normally set on save, and when a file is loaded. For RMG we must do it after calc'ing.
        VectorSubtract(
            &(*gWPArray[i as usize]).origin,
            &(*gWPArray[(i + 1) as usize]).origin,
            &mut v,
        );
        (*gWPArray[i as usize]).disttonext = VectorLength(&v);
        i += 1;
    }

    RemoveWP(); //remove the dummy point at the end of the trail
}

/// `void LoadPath_ThisLevel(void)` (ai_wpnav.c:3347) — top-level bot-path bring-up for the current
/// map: for RMG generate or load nav data, otherwise [`LoadPathData`] the map's `.wnt`; set the
/// edit flag, and cache the CTF flag entities.
pub unsafe fn LoadPath_ThisLevel() {
    let mut mapname = crate::ffi::types::vmCvar_t::zeroed();
    let mut i: c_int = 0;
    let mut ent: *mut gentity_t;

    trap::Cvar_Register(
        Some(&mut mapname),
        "mapname",
        "",
        CVAR_SERVERINFO | CVAR_ROM,
    );

    if g_RMG.integer != 0 {
        //If RMG, generate the path on-the-fly
        trap::Cvar_Register(
            Some(&mut *addr_of_mut!(bot_normgpath)),
            "bot_normgpath",
            "1",
            CVAR_CHEAT,
        );
        //note: This is disabled for now as I'm using standard bot nav
        //on premade terrain levels.

        if bot_normgpath.integer == 0 {
            //autopath the random map
            BeginAutoPathRoutine();
        } else {
            //try loading standard nav data
            LoadPathData(mapname.string.as_ptr());
        }

        gLevelFlags |= LEVELFLAG_NOPOINTPREDICTION;
    } else if LoadPathData(mapname.string.as_ptr()) == 2 {
        //enter "edit" mode if cheats enabled?
    }

    trap::Cvar_Update(&mut *addr_of_mut!(bot_wp_edit));

    if bot_wp_edit.value != 0.0 {
        gBotEdit = 1.0;
    } else {
        gBotEdit = 0.0;
    }

    //set the flag entities
    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && (*ent).inuse != 0 && !(*ent).classname.is_null() {
            if eFlagRed.is_null() && strcmp((*ent).classname, c"team_CTF_redflag".as_ptr()) == 0 {
                eFlagRed = ent;
            } else if eFlagBlue.is_null()
                && strcmp((*ent).classname, c"team_CTF_blueflag".as_ptr()) == 0
            {
                eFlagBlue = ent;
            }

            if !eFlagRed.is_null() && !eFlagBlue.is_null() {
                break;
            }
        }

        i += 1;
    }
}

/// `int AcceptBotCommand(char *cmd, gentity_t *pl)` (ai_wpnav.c:3501) — the bot-waypoint editing
/// command dispatcher (only active when `gBotEdit`): `bot_wp_add`/`_rem`/`_tele`/`_addflagged`/
/// `_switchflags`/`_killoneways`/`_save` etc. Returns 1 if the command was handled. Most editing
/// commands set `gDeactivated` so bots pause until `bot_wp_save` recalculates and reactivates.
pub unsafe fn AcceptBotCommand(cmd: *mut c_char, pl: *mut gentity_t) -> c_int {
    let mut OptionalArgument: c_int;
    let mut i: c_int;
    let mut FlagsFromArgument: c_int;
    let mut OptionalSArgument: *mut c_char;
    let mut RequiredSArgument: *mut c_char;
    let mut mapname = crate::ffi::types::vmCvar_t::zeroed();

    if gBotEdit == 0.0 {
        return 0;
    }

    OptionalArgument = 0;
    i = 0;
    FlagsFromArgument = 0;
    OptionalSArgument = null_mut();
    RequiredSArgument = null_mut();

    //if a waypoint editing related command is issued, bots will deactivate.
    //once bot_wp_save is issued and the trail is recalculated, bots will activate again.

    if pl.is_null() || (*pl).client.is_null() {
        return 0;
    }

    if Q_stricmp(cmd, c"bot_wp_cmdlist".as_ptr()) == 0 {
        //lists all the bot waypoint commands.
        G_Printf("^3bot_wp_add^7 - Add a waypoint (optional int parameter will insert the point after the specified waypoint index in a trail)\n\n");
        G_Printf("^3bot_wp_rem^7 - Remove a waypoint (removes last unless waypoint index is specified as a parameter)\n\n");
        G_Printf("^3bot_wp_addflagged^7 - Same as wp_add, but adds a flagged point (type bot_wp_addflagged for help)\n\n");
        G_Printf("^3bot_wp_switchflags^7 - Switches flags on an existing waypoint (type bot_wp_switchflags for help)\n\n");
        G_Printf("^3bot_wp_tele^7 - Teleport yourself to the specified waypoint's location\n");
        G_Printf("^3bot_wp_killoneways^7 - Removes oneway (backward and forward) flags on all waypoints in the level\n\n");
        G_Printf("^3bot_wp_save^7 - Saves all waypoint data into a file for later use\n");

        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_add".as_ptr()) == 0 {
        gDeactivated = 1.0;
        OptionalSArgument = ConcatArgs(1);

        if !OptionalSArgument.is_null() {
            OptionalArgument = atoi(OptionalSArgument);
        }

        if !OptionalSArgument.is_null() && *OptionalSArgument != 0 {
            CreateNewWP_InTrail(&(*(*pl).client).ps.origin, 0, OptionalArgument);
        } else {
            CreateNewWP(&(*(*pl).client).ps.origin, 0);
        }
        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_rem".as_ptr()) == 0 {
        gDeactivated = 1.0;

        OptionalSArgument = ConcatArgs(1);

        if !OptionalSArgument.is_null() {
            OptionalArgument = atoi(OptionalSArgument);
        }

        if !OptionalSArgument.is_null() && *OptionalSArgument != 0 {
            RemoveWP_InTrail(OptionalArgument);
        } else {
            RemoveWP();
        }

        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_tele".as_ptr()) == 0 {
        gDeactivated = 1.0;
        OptionalSArgument = ConcatArgs(1);

        if !OptionalSArgument.is_null() {
            OptionalArgument = atoi(OptionalSArgument);
        }

        if !OptionalSArgument.is_null() && *OptionalSArgument != 0 {
            TeleportToWP(pl, OptionalArgument);
        } else {
            G_Printf("^3You didn't specify an index. Assuming last.\n");
            TeleportToWP(pl, gWPNum - 1);
        }
        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_spawntele".as_ptr()) == 0 {
        let mut closestSpawn = GetClosestSpawn(pl);

        if closestSpawn.is_null() {
            //There should always be a spawn point..
            return 1;
        }

        closestSpawn = GetNextSpawnInIndex(closestSpawn);

        if !closestSpawn.is_null() {
            VectorCopy(&(*closestSpawn).r.currentOrigin, &mut (*(*pl).client).ps.origin);
        }
        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_addflagged".as_ptr()) == 0 {
        gDeactivated = 1.0;

        RequiredSArgument = ConcatArgs(1);

        if RequiredSArgument.is_null() || *RequiredSArgument == 0 {
            G_Printf("^3Flag string needed for bot_wp_addflagged\nj - Jump point\nd - Duck point\nc - Snipe or camp standing\nf - Wait for func\nm - Do not move to when func is under\ns - Snipe or camp\nx - Oneway, forward\ny - Oneway, back\ng - Mission goal\nn - No visibility\nExample (for a point the bot would jump at, and reverse on when traveling a trail backwards):\nbot_wp_addflagged jx\n");
            return 1;
        }

        while *RequiredSArgument.offset(i as isize) != 0 {
            let ch = *RequiredSArgument.offset(i as isize) as u8;
            if ch == b'j' {
                FlagsFromArgument |= WPFLAG_JUMP;
            } else if ch == b'd' {
                FlagsFromArgument |= WPFLAG_DUCK;
            } else if ch == b'c' {
                FlagsFromArgument |= WPFLAG_SNIPEORCAMPSTAND;
            } else if ch == b'f' {
                FlagsFromArgument |= WPFLAG_WAITFORFUNC;
            } else if ch == b's' {
                FlagsFromArgument |= WPFLAG_SNIPEORCAMP;
            } else if ch == b'x' {
                FlagsFromArgument |= WPFLAG_ONEWAY_FWD;
            } else if ch == b'y' {
                FlagsFromArgument |= WPFLAG_ONEWAY_BACK;
            } else if ch == b'g' {
                FlagsFromArgument |= WPFLAG_GOALPOINT;
            } else if ch == b'n' {
                FlagsFromArgument |= WPFLAG_NOVIS;
            } else if ch == b'm' {
                FlagsFromArgument |= WPFLAG_NOMOVEFUNC;
            }

            i += 1;
        }

        OptionalSArgument = ConcatArgs(2);

        if !OptionalSArgument.is_null() {
            OptionalArgument = atoi(OptionalSArgument);
        }

        if !OptionalSArgument.is_null() && *OptionalSArgument != 0 {
            CreateNewWP_InTrail(&(*(*pl).client).ps.origin, FlagsFromArgument, OptionalArgument);
        } else {
            CreateNewWP(&(*(*pl).client).ps.origin, FlagsFromArgument);
        }
        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_switchflags".as_ptr()) == 0 {
        gDeactivated = 1.0;

        RequiredSArgument = ConcatArgs(1);

        if RequiredSArgument.is_null() || *RequiredSArgument == 0 {
            G_Printf("^3Flag string needed for bot_wp_switchflags\nType bot_wp_addflagged for a list of flags and their corresponding characters, or use 0 for no flags.\nSyntax: bot_wp_switchflags <flags> <n>\n");
            return 1;
        }

        while *RequiredSArgument.offset(i as isize) != 0 {
            let ch = *RequiredSArgument.offset(i as isize) as u8;
            if ch == b'j' {
                FlagsFromArgument |= WPFLAG_JUMP;
            } else if ch == b'd' {
                FlagsFromArgument |= WPFLAG_DUCK;
            } else if ch == b'c' {
                FlagsFromArgument |= WPFLAG_SNIPEORCAMPSTAND;
            } else if ch == b'f' {
                FlagsFromArgument |= WPFLAG_WAITFORFUNC;
            } else if ch == b's' {
                FlagsFromArgument |= WPFLAG_SNIPEORCAMP;
            } else if ch == b'x' {
                FlagsFromArgument |= WPFLAG_ONEWAY_FWD;
            } else if ch == b'y' {
                FlagsFromArgument |= WPFLAG_ONEWAY_BACK;
            } else if ch == b'g' {
                FlagsFromArgument |= WPFLAG_GOALPOINT;
            } else if ch == b'n' {
                FlagsFromArgument |= WPFLAG_NOVIS;
            } else if ch == b'm' {
                FlagsFromArgument |= WPFLAG_NOMOVEFUNC;
            }

            i += 1;
        }

        OptionalSArgument = ConcatArgs(2);

        if !OptionalSArgument.is_null() {
            OptionalArgument = atoi(OptionalSArgument);
        }

        if !OptionalSArgument.is_null() && *OptionalSArgument != 0 {
            WPFlagsModify(OptionalArgument, FlagsFromArgument);
        } else {
            G_Printf("^3Waypoint number (to modify) needed for bot_wp_switchflags\nSyntax: bot_wp_switchflags <flags> <n>\n");
        }
        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_killoneways".as_ptr()) == 0 {
        i = 0;

        while i < gWPNum {
            if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
                if (*gWPArray[i as usize]).flags & WPFLAG_ONEWAY_FWD != 0 {
                    (*gWPArray[i as usize]).flags -= WPFLAG_ONEWAY_FWD;
                }
                if (*gWPArray[i as usize]).flags & WPFLAG_ONEWAY_BACK != 0 {
                    (*gWPArray[i as usize]).flags -= WPFLAG_ONEWAY_BACK;
                }
            }

            i += 1;
        }

        return 1;
    }

    if Q_stricmp(cmd, c"bot_wp_save".as_ptr()) == 0 {
        gDeactivated = 0.0;
        trap::Cvar_Register(
            Some(&mut mapname),
            "mapname",
            "",
            CVAR_SERVERINFO | CVAR_ROM,
        );
        SavePathData(mapname.string.as_ptr());
        return 1;
    }

    0
}

/// `void G_DebugNodeFile()` (ai_wpnav.c:2692) — dump the node grid weights to `ROUTEDEBUG.txt`,
/// laying them out in rows matching the terrain's X extent. Upstream this is gated behind
/// `#ifdef DEBUG_NODE_FILE` (never defined in the shipping build); ported here for completeness.
///
/// DEVIATION: the C uses a 131072-byte stack array `fileString`; we heap-allocate the same size
/// (`vec!`) to avoid a stack overflow — behavior is identical.
pub unsafe fn G_DebugNodeFile() {
    let f: fileHandle_t;
    let mut i: c_int = 0;
    let mut placeX: f32;
    let mut fileString: Vec<c_char> = vec![0; 131072];
    let terrain: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, classname), c"terrain".as_ptr());

    fileString[0] = 0;

    placeX = (*terrain).r.absmin[0];

    while i < nodenum {
        strcat(
            fileString.as_mut_ptr(),
            va(format_args!("{}-{:.6} ", i, nodetable[i as usize].weight)),
        );
        placeX += DEFAULT_GRID_SPACING as f32;

        if placeX >= (*terrain).r.absmax[0] {
            strcat(fileString.as_mut_ptr(), c"\n".as_ptr());
            placeX = (*terrain).r.absmin[0];
        }
        i += 1;
    }

    let (_l, fh) = trap::FS_FOpenFile("ROUTEDEBUG.txt", FS_WRITE);
    f = fh;
    {
        let wbuf = core::slice::from_raw_parts(fileString.as_ptr() as *const u8, strlen(fileString.as_ptr()));
        trap::FS_Write(wbuf, f);
    }
    trap::FS_FCloseFile(f);
}

/// `#define ALLOWABLE_DEBUG_FILE_SIZE 1048576` (ai_wpnav.c:2729)
const ALLOWABLE_DEBUG_FILE_SIZE: usize = 1048576;

/// `void CreateAsciiTableRepresentation()` (ai_wpnav.c:2731) — draw a text grid of the entire
/// waypoint array to `ROUTEDRAWN.txt` (each cell shows the waypoint index, prefixed by F/B for
/// one-way flags). Upstream gated behind `#ifdef ASCII_ART_DEBUG`; ported for completeness.
///
/// DEVIATION: the C `char fileString[ALLOWABLE_DEBUG_FILE_SIZE]` stack array is heap-allocated
/// here to avoid a 1 MB stack frame — behavior is identical.
pub unsafe fn CreateAsciiTableRepresentation() {
    //Draw a text grid of the entire waypoint array (useful for debugging final waypoint placement)
    let f: fileHandle_t;
    let mut i: c_int;
    let mut sP: c_int = 0;
    let mut placeX: c_int;
    let mut placeY: c_int;
    let mut oldX: c_int;
    let mut oldY: c_int;
    let mut fileString: Vec<c_char> = vec![0; ALLOWABLE_DEBUG_FILE_SIZE];
    let mut bChr: c_char;
    let terrain: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, classname), c"terrain".as_ptr());

    placeX = (*terrain).r.absmin[0] as c_int;
    placeY = (*terrain).r.absmin[1] as c_int;

    oldX = placeX - 1;
    oldY = placeY - 1;

    while placeY < (*terrain).r.absmax[1] as c_int {
        while placeX < (*terrain).r.absmax[0] as c_int {
            let mut gotit = false;

            i = 0;
            while i < gWPNum {
                if ((*gWPArray[i as usize]).origin[0] as c_int <= placeX
                    && (*gWPArray[i as usize]).origin[0] as c_int > oldX)
                    && ((*gWPArray[i as usize]).origin[1] as c_int <= placeY
                        && (*gWPArray[i as usize]).origin[1] as c_int > oldY)
                {
                    gotit = true;
                    break;
                }
                i += 1;
            }

            if gotit {
                if (*gWPArray[i as usize]).flags & WPFLAG_ONEWAY_FWD != 0 {
                    bChr = b'F' as c_char;
                } else if (*gWPArray[i as usize]).flags & WPFLAG_ONEWAY_BACK != 0 {
                    bChr = b'B' as c_char;
                } else {
                    bChr = b'+' as c_char;
                }

                if (*gWPArray[i as usize]).index < 10 {
                    fileString[sP as usize] = bChr;
                    fileString[(sP + 1) as usize] = b'0' as c_char;
                    fileString[(sP + 2) as usize] = b'0' as c_char;
                    fileString[(sP + 3) as usize] =
                        *va(format_args!("{}", (*gWPArray[i as usize]).index)).offset(0);
                } else if (*gWPArray[i as usize]).index < 100 {
                    let vastore = va(format_args!("{}", (*gWPArray[i as usize]).index));

                    fileString[sP as usize] = bChr;
                    fileString[(sP + 1) as usize] = b'0' as c_char;
                    fileString[(sP + 2) as usize] = *vastore.offset(0);
                    fileString[(sP + 3) as usize] = *vastore.offset(1);
                } else if (*gWPArray[i as usize]).index < 1000 {
                    let vastore = va(format_args!("{}", (*gWPArray[i as usize]).index));

                    fileString[sP as usize] = bChr;
                    fileString[(sP + 1) as usize] = *vastore.offset(0);
                    fileString[(sP + 2) as usize] = *vastore.offset(1);
                    fileString[(sP + 3) as usize] = *vastore.offset(2);
                } else {
                    fileString[sP as usize] = b'X' as c_char;
                    fileString[(sP + 1) as usize] = b'X' as c_char;
                    fileString[(sP + 2) as usize] = b'X' as c_char;
                    fileString[(sP + 3) as usize] = b'X' as c_char;
                }
            } else {
                fileString[sP as usize] = b'-' as c_char;
                fileString[(sP + 1) as usize] = b'-' as c_char;
                fileString[(sP + 2) as usize] = b'-' as c_char;
                fileString[(sP + 3) as usize] = b'-' as c_char;
            }

            sP += 4;

            if sP >= ALLOWABLE_DEBUG_FILE_SIZE as c_int - 16 {
                break;
            }
            oldX = placeX;
            placeX += DEFAULT_GRID_SPACING;
        }

        placeX = (*terrain).r.absmin[0] as c_int;
        oldX = placeX - 1;
        fileString[sP as usize] = b'\n' as c_char;
        sP += 1;

        if sP >= ALLOWABLE_DEBUG_FILE_SIZE as c_int - 16 {
            break;
        }

        oldY = placeY;
        placeY += DEFAULT_GRID_SPACING;
    }

    fileString[sP as usize] = 0;

    let (_l, fh) = trap::FS_FOpenFile("ROUTEDRAWN.txt", FS_WRITE);
    f = fh;
    {
        let wbuf = core::slice::from_raw_parts(fileString.as_ptr() as *const u8, strlen(fileString.as_ptr()));
        trap::FS_Write(wbuf, f);
    }
    trap::FS_FCloseFile(f);
}

/// `void CreateAsciiNodeTableRepresentation(int start, int end)` (ai_wpnav.c:2855) — draw a text
/// grid of a single node path (from `start` 'A' to `end` 'Z', other cells show the node weight)
/// to `ROUTEDRAWN.txt`. Upstream gated behind `#ifdef ASCII_ART_DEBUG`; ported for completeness.
///
/// DEVIATION: heap-allocated `fileString` (see [`CreateAsciiTableRepresentation`]).
pub unsafe fn CreateAsciiNodeTableRepresentation(start: c_int, end: c_int) {
    //draw a text grid of a single node path, from point A to Z.
    let f: fileHandle_t;
    let mut i: c_int;
    let mut sP: c_int = 0;
    let mut placeX: c_int;
    let mut placeY: c_int;
    let mut oldX: c_int;
    let mut oldY: c_int;
    let mut fileString: Vec<c_char> = vec![0; ALLOWABLE_DEBUG_FILE_SIZE];
    let terrain: *mut gentity_t = G_Find(null_mut(), offset_of!(gentity_s, classname), c"terrain".as_ptr());

    placeX = (*terrain).r.absmin[0] as c_int;
    placeY = (*terrain).r.absmin[1] as c_int;

    oldX = placeX - 1;
    oldY = placeY - 1;

    while placeY < (*terrain).r.absmax[1] as c_int {
        while placeX < (*terrain).r.absmax[0] as c_int {
            let mut gotit = false;

            i = 0;
            while i < nodenum {
                if ((nodetable[i as usize].origin[0] as c_int) <= placeX
                    && nodetable[i as usize].origin[0] as c_int > oldX)
                    && ((nodetable[i as usize].origin[1] as c_int) <= placeY
                        && nodetable[i as usize].origin[1] as c_int > oldY)
                {
                    gotit = true;
                    break;
                }
                i += 1;
            }

            if gotit {
                if i == start {
                    //beginning of the node trail
                    fileString[sP as usize] = b'A' as c_char;
                    fileString[(sP + 1) as usize] = b'A' as c_char;
                    fileString[(sP + 2) as usize] = b'A' as c_char;
                    fileString[(sP + 3) as usize] = b'A' as c_char;
                } else if i == end {
                    //destination of the node trail
                    fileString[sP as usize] = b'Z' as c_char;
                    fileString[(sP + 1) as usize] = b'Z' as c_char;
                    fileString[(sP + 2) as usize] = b'Z' as c_char;
                    fileString[(sP + 3) as usize] = b'Z' as c_char;
                } else if nodetable[i as usize].weight < 10.0 {
                    fileString[sP as usize] = b'+' as c_char;
                    fileString[(sP + 1) as usize] = b'0' as c_char;
                    fileString[(sP + 2) as usize] = b'0' as c_char;
                    fileString[(sP + 3) as usize] =
                        *va(format_args!("{:.6}", nodetable[i as usize].weight)).offset(0);
                } else if nodetable[i as usize].weight < 100.0 {
                    let vastore = va(format_args!("{:.6}", nodetable[i as usize].weight));

                    fileString[sP as usize] = b'+' as c_char;
                    fileString[(sP + 1) as usize] = b'0' as c_char;
                    fileString[(sP + 2) as usize] = *vastore.offset(0);
                    fileString[(sP + 3) as usize] = *vastore.offset(1);
                } else if nodetable[i as usize].weight < 1000.0 {
                    let vastore = va(format_args!("{:.6}", nodetable[i as usize].weight));

                    fileString[sP as usize] = b'+' as c_char;
                    fileString[(sP + 1) as usize] = *vastore.offset(0);
                    fileString[(sP + 2) as usize] = *vastore.offset(1);
                    fileString[(sP + 3) as usize] = *vastore.offset(2);
                } else {
                    fileString[sP as usize] = b'X' as c_char;
                    fileString[(sP + 1) as usize] = b'X' as c_char;
                    fileString[(sP + 2) as usize] = b'X' as c_char;
                    fileString[(sP + 3) as usize] = b'X' as c_char;
                }
            } else {
                fileString[sP as usize] = b'-' as c_char;
                fileString[(sP + 1) as usize] = b'-' as c_char;
                fileString[(sP + 2) as usize] = b'-' as c_char;
                fileString[(sP + 3) as usize] = b'-' as c_char;
            }

            sP += 4;

            if sP >= ALLOWABLE_DEBUG_FILE_SIZE as c_int - 16 {
                break;
            }
            oldX = placeX;
            placeX += DEFAULT_GRID_SPACING;
        }

        placeX = (*terrain).r.absmin[0] as c_int;
        oldX = placeX - 1;
        fileString[sP as usize] = b'\n' as c_char;
        sP += 1;

        if sP >= ALLOWABLE_DEBUG_FILE_SIZE as c_int - 16 {
            break;
        }

        oldY = placeY;
        placeY += DEFAULT_GRID_SPACING;
    }

    fileString[sP as usize] = 0;

    let (_l, fh) = trap::FS_FOpenFile("ROUTEDRAWN.txt", FS_WRITE);
    f = fh;
    {
        let wbuf = core::slice::from_raw_parts(fileString.as_ptr() as *const u8, strlen(fileString.as_ptr()));
        trap::FS_Write(wbuf, f);
    }
    trap::FS_FCloseFile(f);
}
