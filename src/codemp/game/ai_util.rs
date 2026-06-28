//! Ported `codemp/game/ai_util.c` — bot AI utility helpers.
//!
//! This file holds the bot memory-arena wrappers (`B_*Alloc`/`B_*Free`,
//! `B_InitAlloc`/`B_CleanupAlloc`), the personality-file key/value parsers
//! (`GetValueGroup`/`GetPairedValue`), and the personality loader
//! (`BotUtilizePersonality` + `ParseEmotionalAttachments`).
//!
//! **`BOT_ZMALLOC` is never defined in the build**, so every `#ifdef BOT_ZMALLOC`
//! block (the `BAllocList[MAX_BALLOC]` arena, `trap_BotGetMemoryGame`/
//! `trap_BotFreeMemoryGame` bookkeeping, the `BOTMEMTRACK` counters) compiles out.
//! The active code reduces `B_Alloc`→`BG_Alloc`, `B_TempAlloc`→`BG_TempAlloc`,
//! `B_TempFree`→`BG_TempFree`, and `B_Free` to a no-op. The `BAllocList` file-scope
//! global is `#ifdef`-gated out and not emitted; the `gBotChatBuffer` global is live in
//! the PC source (it was `/* */`-dead in the Xbox tree) and is emitted here.
//!
//! No oracle — these are stateful engine-trap / file-I/O and pointer-walking
//! string parsers; they are faithful ports verified by inspection against the C.

#[allow(unused_imports)]
// each import lands with the fn that first uses it (one commit per fn)
use core::ffi::{c_char, c_int, c_void};

use crate::codemp::game::ai_wpnav::gWPArray;
use crate::codemp::game::bg_misc::{BG_Alloc, BG_TempAlloc, BG_TempFree};

use crate::codemp::game::ai_main_h::{
    bot_state_t, DEFAULT_FORCEPOWERS, MAX_CHAT_BUFFER_SIZE, MAX_CHAT_LINE_SIZE,
    MAX_FORCE_INFO_SIZE, MAX_LOVED_ONES,
};
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR, WP_FLECHETTE,
    WP_MELEE, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE,
};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_main::G_Printf;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared::{Com_sprintf, Sz};
use crate::codemp::game::q_shared_h::{FS_READ, MAX_CLIENTS};
use crate::trap;

/// `S_COLOR_RED` (q_shared.h) — console color escape.
const S_COLOR_RED: &str = "^1";

// C `strstr`/`strlen`/`atoi`/`atof` (libc): the `_JK2MP`/non-`Q3_VM` build links
// these from the CRT rather than the `Q3_VM` bg_lib shims, so they come in via
// `extern "C"` (mirrors `bg_vehicleLoad.rs`/`g_saga.rs`) instead of the
// `vm`-gated copies.
extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
}

pub fn B_TempAlloc(size: c_int) -> *mut c_void {
    BG_TempAlloc(size)
}

pub fn B_TempFree(size: c_int) {
    BG_TempFree(size);
}

pub fn B_Alloc(size: c_int) -> *mut c_void {
    // BOT_ZMALLOC is undefined, so the `#else` branch is the live one.
    BG_Alloc(size)
}

// BOT_ZMALLOC is undefined, so the whole body compiles out — `B_Free` is a no-op
// that still takes (and ignores) the pointer, matching the C signature.
pub fn B_Free(_ptr: *mut c_void) {}

pub fn B_InitAlloc() {
    // The `#ifdef BOT_ZMALLOC memset(BAllocList, ...)` is compiled out; only the
    // `memset(gWPArray, 0, sizeof(gWPArray))` remains — zero the waypoint array.
    unsafe {
        for slot in (*core::ptr::addr_of_mut!(gWPArray)).iter_mut() {
            *slot = core::ptr::null_mut();
        }
    }
}

pub fn B_CleanupAlloc() {
    // PC body is only the `#ifdef BOT_ZMALLOC` `BAllocList` sweep, which is
    // undefined-out, leaving an empty function. (The Xbox source additionally cleared
    // every live `gWPArray` slot here; PC removed that loop entirely.)
}

/// # Safety
/// `buf`/`group` must be NUL-terminated; `outbuf` must point to a writable buffer
/// large enough for the located group's contents.
pub unsafe fn GetValueGroup(buf: *mut c_char, group: *mut c_char, outbuf: *mut c_char) -> c_int {
    const LB: c_char = b'{' as c_char;
    const RB: c_char = b'}' as c_char;
    const NL: c_char = b'\n' as c_char;

    let mut i: c_int;
    // C declares `int iplace = 0;` — never read; dropped to avoid a dead-store.
    let mut failure: c_int;
    let mut startpoint: c_int;
    let mut startletter: c_int;
    let mut subg: c_int = 0;

    i = 0;

    let mut place: *mut c_char = strstr(buf, group);

    if place.is_null() {
        return 0;
    }

    startpoint = (place as isize - buf as isize) as c_int + strlen(group) as c_int + 1;
    startletter = (place as isize - buf as isize) as c_int - 1;

    failure = 0;

    while *buf.offset((startpoint + 1) as isize) != LB || *buf.offset(startletter as isize) != NL {
        let placesecond: *mut c_char = strstr(place.offset(1), group);

        if !placesecond.is_null() {
            startpoint += (placesecond as isize - place as isize) as c_int;
            startletter += (placesecond as isize - place as isize) as c_int;
            place = placesecond;
        } else {
            failure = 1;
            break;
        }
    }

    if failure != 0 {
        return 0;
    }

    //we have found the proper group name if we made it here, so find the opening brace and read into the outbuf
    //until hitting the end brace

    while *buf.offset(startpoint as isize) != LB {
        startpoint += 1;
    }

    startpoint += 1;

    while *buf.offset(startpoint as isize) != RB || subg != 0 {
        if *buf.offset(startpoint as isize) == LB {
            subg += 1;
        } else if *buf.offset(startpoint as isize) == RB {
            subg -= 1;
        }
        *outbuf.offset(i as isize) = *buf.offset(startpoint as isize);
        i += 1;
        startpoint += 1;
    }
    *outbuf.offset(i as isize) = b'\0' as c_char;

    1
}

/// # Safety
/// `buf`/`key`/`outbuf` may be null (checked); when non-null `buf`/`key` must be
/// NUL-terminated and `outbuf` must point to a writable buffer large enough for
/// the located value. `buf` is mutated in place (comment lines are blanked out).
pub unsafe fn GetPairedValue(buf: *mut c_char, key: *mut c_char, outbuf: *mut c_char) -> c_int {
    const SP: c_char = b' ' as c_char;
    const NL: c_char = b'\n' as c_char;
    const SL: c_char = b'/' as c_char;
    const TAB: c_char = 9; //tab == 9

    let mut startpoint: c_int;
    let mut startletter: c_int;
    let mut i: c_int;
    let mut found: c_int;

    if buf.is_null() || key.is_null() || outbuf.is_null() {
        return 0;
    }

    i = 0;

    while *buf.offset(i as isize) != 0 && *buf.offset(i as isize) != b'\0' as c_char {
        if *buf.offset(i as isize) == SL {
            if *buf.offset((i + 1) as isize) != 0
                && *buf.offset((i + 1) as isize) != b'\0' as c_char
                && *buf.offset((i + 1) as isize) == SL
            {
                while *buf.offset(i as isize) != NL {
                    *buf.offset(i as isize) = SL;
                    i += 1;
                }
            }
        }
        i += 1;
    }

    let mut place: *mut c_char = strstr(buf, key);

    if place.is_null() {
        return 0;
    }
    //tab == 9
    startpoint = (place as isize - buf as isize) as c_int + strlen(key) as c_int;
    startletter = (place as isize - buf as isize) as c_int - 1;

    found = 0;

    while found == 0 {
        if startletter == 0
            || *buf.offset(startletter as isize) == 0
            || *buf.offset(startletter as isize) == b'\0' as c_char
            || *buf.offset(startletter as isize) == TAB
            || *buf.offset(startletter as isize) == SP
            || *buf.offset(startletter as isize) == NL
        {
            if *buf.offset(startpoint as isize) == b'\0' as c_char
                || *buf.offset(startpoint as isize) == TAB
                || *buf.offset(startpoint as isize) == SP
                || *buf.offset(startpoint as isize) == NL
            {
                found = 1;
                break;
            }
        }

        let placesecond: *mut c_char = strstr(place.offset(1), key);

        if !placesecond.is_null() {
            startpoint += (placesecond as isize - place as isize) as c_int;
            startletter += (placesecond as isize - place as isize) as c_int;
            place = placesecond;
        } else {
            place = core::ptr::null_mut();
            break;
        }
    }

    if found == 0
        || place.is_null()
        || *buf.offset(startpoint as isize) == 0
        || *buf.offset(startpoint as isize) == b'\0' as c_char
    {
        return 0;
    }

    while *buf.offset(startpoint as isize) == SP
        || *buf.offset(startpoint as isize) == TAB
        || *buf.offset(startpoint as isize) == NL
    {
        startpoint += 1;
    }

    i = 0;

    while *buf.offset(startpoint as isize) != 0
        && *buf.offset(startpoint as isize) != b'\0' as c_char
        && *buf.offset(startpoint as isize) != NL
    {
        *outbuf.offset(i as isize) = *buf.offset(startpoint as isize);
        i += 1;
        startpoint += 1;
    }

    *outbuf.offset(i as isize) = b'\0' as c_char;

    1
}

/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; `section` must be a NUL-terminated
/// chat-section name. Reads the module statics `gBotChatBuffer` and `level`.
///
/// **No oracle** — drives the per-client `gBotChatBuffer` global, the `level` time
/// global, the `se_language` cvar trap, and `Q_irand`; not isolatable off-engine.
pub unsafe fn BotDoChat(bs: *mut bot_state_t, section: *mut c_char, always: c_int) -> c_int {
    const NL: c_char = b'\n' as c_char;
    const CR: c_char = 13;
    const TAB: c_char = 9;

    let mut inc_1: c_int;
    let mut inc_2: c_int;
    let lines: c_int;
    let mut checkedline: c_int;
    let mut getthisline: c_int;

    if (*bs).canChat == 0 {
        return 0;
    }

    if (*bs).doChat != 0 {
        //already have a chat scheduled
        return 0;
    }

    if trap::Cvar_VariableIntegerValue("se_language") != 0 {
        //no chatting unless English.
        return 0;
    }

    if Q_irand(1, 10) > (*bs).chatFrequency && always == 0 {
        return 0;
    }

    (*bs).chatTeam = 0;

    let chatgroup: *mut c_char = B_TempAlloc(MAX_CHAT_BUFFER_SIZE) as *mut c_char;

    let rval: c_int = GetValueGroup(
        (*core::ptr::addr_of_mut!(gBotChatBuffer))[(*bs).client as usize].as_mut_ptr(),
        section,
        chatgroup,
    );

    if rval == 0 {
        //the bot has no group defined for the specified chat event
        B_TempFree(MAX_CHAT_BUFFER_SIZE); //chatgroup
        return 0;
    }

    inc_1 = 0;
    inc_2 = 2;

    while *chatgroup.offset(inc_2 as isize) != 0
        && *chatgroup.offset(inc_2 as isize) != b'\0' as c_char
    {
        if *chatgroup.offset(inc_2 as isize) != CR && *chatgroup.offset(inc_2 as isize) != TAB {
            *chatgroup.offset(inc_1 as isize) = *chatgroup.offset(inc_2 as isize);
            inc_1 += 1;
        }
        inc_2 += 1;
    }
    *chatgroup.offset(inc_1 as isize) = b'\0' as c_char;

    inc_1 = 0;

    lines = {
        let mut count = 0;
        while *chatgroup.offset(inc_1 as isize) != 0
            && *chatgroup.offset(inc_1 as isize) != b'\0' as c_char
        {
            if *chatgroup.offset(inc_1 as isize) == NL {
                count += 1;
            }
            inc_1 += 1;
        }
        count
    };

    if lines == 0 {
        B_TempFree(MAX_CHAT_BUFFER_SIZE); //chatgroup
        return 0;
    }

    getthisline = Q_irand(0, lines + 1);

    if getthisline < 1 {
        getthisline = 1;
    }
    if getthisline > lines {
        getthisline = lines;
    }

    checkedline = 1;

    inc_1 = 0;

    while checkedline != getthisline {
        if *chatgroup.offset(inc_1 as isize) != 0
            && *chatgroup.offset(inc_1 as isize) != b'\0' as c_char
        {
            if *chatgroup.offset(inc_1 as isize) == NL {
                inc_1 += 1;
                checkedline += 1;
            }
        }

        if checkedline == getthisline {
            break;
        }

        inc_1 += 1;
    }

    //we're at the starting position of the desired line here
    inc_2 = 0;

    while *chatgroup.offset(inc_1 as isize) != NL {
        *chatgroup.offset(inc_2 as isize) = *chatgroup.offset(inc_1 as isize);
        inc_2 += 1;
        inc_1 += 1;
    }
    *chatgroup.offset(inc_2 as isize) = b'\0' as c_char;

    //trap_EA_Say(bs->client, chatgroup);
    inc_1 = 0;
    inc_2 = 0;

    if strlen(chatgroup) > MAX_CHAT_LINE_SIZE as usize {
        B_TempFree(MAX_CHAT_BUFFER_SIZE); //chatgroup
        return 0;
    }

    while *chatgroup.offset(inc_1 as isize) != 0 {
        if *chatgroup.offset(inc_1 as isize) == b'%' as c_char
            && *chatgroup.offset((inc_1 + 1) as isize) != b'%' as c_char
        {
            inc_1 += 1;

            let cobject: *mut gentity_t = if *chatgroup.offset(inc_1 as isize) == b's' as c_char
                && !(*bs).chatObject.is_null()
            {
                (*bs).chatObject
            } else if *chatgroup.offset(inc_1 as isize) == b'a' as c_char
                && !(*bs).chatAltObject.is_null()
            {
                (*bs).chatAltObject
            } else {
                core::ptr::null_mut()
            };

            if !cobject.is_null() && !(*cobject).client.is_null() {
                let mut inc_n: c_int = 0;

                while (*(*cobject).client).pers.netname[inc_n as usize] != 0 {
                    (*bs).currentChat[inc_2 as usize] =
                        (*(*cobject).client).pers.netname[inc_n as usize];
                    inc_2 += 1;
                    inc_n += 1;
                }
                inc_2 -= 1; //to make up for the auto-increment below
            }
        } else {
            (*bs).currentChat[inc_2 as usize] = *chatgroup.offset(inc_1 as isize);
        }
        inc_2 += 1;
        inc_1 += 1;
    }
    (*bs).currentChat[inc_2 as usize] = b'\0' as c_char;

    if strcmp(section, c"GeneralGreetings".as_ptr()) == 0 {
        (*bs).doChat = 2;
    } else {
        (*bs).doChat = 1;
    }
    (*bs).chatTime_stored =
        (strlen((*bs).currentChat.as_ptr()) as c_int * 45 + Q_irand(1300, 1500)) as f32;
    (*bs).chatTime = level.time as f32 + (*bs).chatTime_stored;

    B_TempFree(MAX_CHAT_BUFFER_SIZE); //chatgroup

    1
}

/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; `buf` must be NUL-terminated.
pub unsafe fn ParseEmotionalAttachments(bs: *mut bot_state_t, buf: *mut c_char) {
    const SP: c_char = b' ' as c_char;
    const LB: c_char = b'{' as c_char;
    const RB: c_char = b'}' as c_char;
    const NL: c_char = b'\n' as c_char;
    const TAB: c_char = 9;
    const CR: c_char = 13;

    let mut i: c_int = 0;
    let mut i_c: c_int;
    let mut tbuf: [c_char; 16] = [0; 16];

    while *buf.offset(i as isize) != 0 && *buf.offset(i as isize) != RB {
        while *buf.offset(i as isize) == SP
            || *buf.offset(i as isize) == LB
            || *buf.offset(i as isize) == TAB
            || *buf.offset(i as isize) == CR
            || *buf.offset(i as isize) == NL
        {
            i += 1;
        }

        if *buf.offset(i as isize) != 0 && *buf.offset(i as isize) != RB {
            i_c = 0;
            while *buf.offset(i as isize) != LB
                && *buf.offset(i as isize) != TAB
                && *buf.offset(i as isize) != CR
                && *buf.offset(i as isize) != NL
            {
                (*bs).loved[(*bs).lovednum as usize].name[i_c as usize] = *buf.offset(i as isize);
                i_c += 1;
                i += 1;
            }
            (*bs).loved[(*bs).lovednum as usize].name[i_c as usize] = b'\0' as c_char;

            while *buf.offset(i as isize) == SP
                || *buf.offset(i as isize) == LB
                || *buf.offset(i as isize) == TAB
                || *buf.offset(i as isize) == CR
                || *buf.offset(i as isize) == NL
            {
                i += 1;
            }

            i_c = 0;

            while *buf.offset(i as isize) != LB
                && *buf.offset(i as isize) != TAB
                && *buf.offset(i as isize) != CR
                && *buf.offset(i as isize) != NL
            {
                tbuf[i_c as usize] = *buf.offset(i as isize);
                i_c += 1;
                i += 1;
            }
            tbuf[i_c as usize] = b'\0' as c_char;

            (*bs).loved[(*bs).lovednum as usize].level = atoi(tbuf.as_ptr());

            (*bs).lovednum += 1;
        } else {
            break;
        }

        if (*bs).lovednum >= MAX_LOVED_ONES as c_int {
            return;
        }

        i += 1;
    }
}

/// `char gBotChatBuffer[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE];` (ai_util.c:12) — per-client
/// chat-section scratch, live in the PC build (the Xbox source `/* */`-commented every
/// consumer). Filled by `ReadChatGroups` from the personality file's `BEGIN_CHAT_GROUPS`
/// section; consumed by `BotDoChat`.
#[allow(non_upper_case_globals)]
pub(crate) static mut gBotChatBuffer: [[c_char; MAX_CHAT_BUFFER_SIZE as usize]; MAX_CLIENTS] =
    [[0; MAX_CHAT_BUFFER_SIZE as usize]; MAX_CLIENTS];

/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; `buf` must be NUL-terminated.
pub unsafe fn ReadChatGroups(bs: *mut bot_state_t, buf: *mut c_char) -> c_int {
    const NL: c_char = b'\n' as c_char;

    let cgroupbegin: *mut c_char = strstr(buf, c"BEGIN_CHAT_GROUPS".as_ptr());

    if cgroupbegin.is_null() {
        return 0;
    }

    if strlen(cgroupbegin) >= MAX_CHAT_BUFFER_SIZE as usize {
        G_Printf(&format!(
            "{S_COLOR_RED}Error: Personality chat section exceeds max size\n"
        ));
        return 0;
    }

    let mut cgbplace: c_int = (cgroupbegin as isize - buf as isize) as c_int + 1;

    while *buf.offset(cgbplace as isize) != NL {
        cgbplace += 1;
    }

    let mut i: c_int = 0;

    while *buf.offset(cgbplace as isize) != 0 && *buf.offset(cgbplace as isize) != b'\0' as c_char {
        (*core::ptr::addr_of_mut!(gBotChatBuffer))[(*bs).client as usize][i as usize] =
            *buf.offset(cgbplace as isize);
        i += 1;
        cgbplace += 1;
    }

    (*core::ptr::addr_of_mut!(gBotChatBuffer))[(*bs).client as usize][i as usize] = b'\0' as c_char;

    1
}

/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer with a NUL-terminated
/// `settings.personalityfile`.
///
/// **No oracle** — pure engine-trap file I/O (`trap_FS_*`), which the off-engine
/// oracle harness cannot satisfy (the `BG_SiegeParseClassFile` precedent). The
/// `trap` wrappers are Rust-idiomatic: `FS_FOpenFile` takes a `&str` path and
/// returns `(len, handle)`, `FS_Read` takes a `&mut [u8]` over the raw read buffer.
///
/// PC re-port (Xbox→PC): `buf` is a fixed `B_TempAlloc(131072)` allocated up front with
/// an `if (len >= 131072)` length guard and a post-read space-fill loop; `group` is a
/// fixed `B_TempAlloc(65536)`. The `chatability`/`chatfrequency` parse block and the
/// `gBotChatBuffer` clear loop (both `/* */`-dead in Xbox) are now live, as is the
/// `if (bs->canChat) ReadChatGroups(...)` block.
pub unsafe fn BotUtilizePersonality(bs: *mut bot_state_t) {
    let mut len: c_int;
    // C saves `rlen = len` and restores `len = rlen` around the space-fill loop, but PC
    // never reads `len` again (the trailing allocs/frees are fixed 131072/65536), so the
    // save/restore is a dead store and is dropped here (cf. GetValueGroup's `iplace`).
    let mut failed: c_int;
    let readbuf: *mut c_char;
    let group: *mut c_char;

    let buf: *mut c_char = B_TempAlloc(131072) as *mut c_char;

    let pfile =
        core::ffi::CStr::from_ptr((*bs).settings.personalityfile.as_ptr()).to_string_lossy();
    let (l, f) = trap::FS_FOpenFile(&pfile, FS_READ);
    len = l;

    failed = 0;

    if f == 0 {
        G_Printf(&format!(
            "{S_COLOR_RED}Error: Specified personality not found\n"
        ));
        B_TempFree(131072); //buf
        return;
    }

    if len >= 131072 {
        G_Printf(&format!(
            "{S_COLOR_RED}Personality file exceeds maximum length\n"
        ));
        B_TempFree(131072); //buf
        return;
    }

    {
        let rbuf = core::slice::from_raw_parts_mut(buf as *mut u8, len as usize);
        trap::FS_Read(rbuf, f);
    }

    while len < 131072 {
        //kill all characters after the file length, since sometimes FS_Read doesn't do that entirely (or so it seems)
        *buf.offset(len as isize) = b'\0' as c_char;
        len += 1;
    }

    readbuf = B_TempAlloc(1024) as *mut c_char;
    group = B_TempAlloc(65536) as *mut c_char;

    if GetValueGroup(buf, c"GeneralBotInfo".as_ptr() as *mut c_char, group) == 0 {
        G_Printf(&format!(
            "{S_COLOR_RED}Personality file contains no GeneralBotInfo group\n"
        ));
        failed = 1; //set failed so we know to set everything to default values
    }

    if failed == 0 && GetPairedValue(group, c"reflex".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).skills.reflex = atoi(readbuf);
    } else {
        (*bs).skills.reflex = 100; //default
    }

    if failed == 0 && GetPairedValue(group, c"accuracy".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).skills.accuracy = atof(readbuf) as f32;
    } else {
        (*bs).skills.accuracy = 10.0; //default
    }

    if failed == 0 && GetPairedValue(group, c"turnspeed".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).skills.turnspeed = atof(readbuf) as f32;
    } else {
        (*bs).skills.turnspeed = 0.01; //default
    }

    if failed == 0
        && GetPairedValue(group, c"turnspeed_combat".as_ptr() as *mut c_char, readbuf) != 0
    {
        (*bs).skills.turnspeed_combat = atof(readbuf) as f32;
    } else {
        (*bs).skills.turnspeed_combat = 0.05; //default
    }

    if failed == 0 && GetPairedValue(group, c"maxturn".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).skills.maxturn = atof(readbuf) as f32;
    } else {
        (*bs).skills.maxturn = 360.0; //default
    }

    if failed == 0 && GetPairedValue(group, c"perfectaim".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).skills.perfectaim = atoi(readbuf);
    } else {
        (*bs).skills.perfectaim = 0; //default
    }
    if failed == 0 && GetPairedValue(group, c"chatability".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).canChat = atoi(readbuf);
    } else {
        (*bs).canChat = 0; //default
    }

    if failed == 0 && GetPairedValue(group, c"chatfrequency".as_ptr() as *mut c_char, readbuf) != 0
    {
        (*bs).chatFrequency = atoi(readbuf);
    } else {
        (*bs).chatFrequency = 5; //default
    }

    if failed == 0 && GetPairedValue(group, c"hatelevel".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).loved_death_thresh = atoi(readbuf);
    } else {
        (*bs).loved_death_thresh = 3; //default
    }

    if failed == 0 && GetPairedValue(group, c"camper".as_ptr() as *mut c_char, readbuf) != 0 {
        (*bs).isCamper = atoi(readbuf);
    } else {
        (*bs).isCamper = 0; //default
    }

    if failed == 0
        && GetPairedValue(group, c"saberspecialist".as_ptr() as *mut c_char, readbuf) != 0
    {
        (*bs).saberSpecialist = atoi(readbuf);
    } else {
        (*bs).saberSpecialist = 0; //default
    }

    if failed == 0 && GetPairedValue(group, c"forceinfo".as_ptr() as *mut c_char, readbuf) != 0 {
        Com_sprintf(
            (*bs).forceinfo.as_mut_ptr(),
            MAX_FORCE_INFO_SIZE as c_int,
            format_args!("{}\0", Sz(readbuf)),
        );
    } else {
        Com_sprintf(
            (*bs).forceinfo.as_mut_ptr(),
            MAX_FORCE_INFO_SIZE as c_int,
            format_args!("{}\0", DEFAULT_FORCEPOWERS),
        );
    }

    let mut i: c_int = 0;

    while i < MAX_CHAT_BUFFER_SIZE {
        //clear out the chat buffer for this bot
        (*core::ptr::addr_of_mut!(gBotChatBuffer))[(*bs).client as usize][i as usize] =
            b'\0' as c_char;
        i += 1;
    }

    if (*bs).canChat != 0 {
        if ReadChatGroups(bs, buf) == 0 {
            (*bs).canChat = 0;
        }
    }

    if GetValueGroup(buf, c"BotWeaponWeights".as_ptr() as *mut c_char, group) != 0 {
        if GetPairedValue(group, c"WP_STUN_BATON".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_STUN_BATON as usize] = atoi(readbuf) as f32;
            (*bs).botWeaponWeights[WP_MELEE as usize] =
                (*bs).botWeaponWeights[WP_STUN_BATON as usize];
        }

        if GetPairedValue(group, c"WP_SABER".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_SABER as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_BRYAR_PISTOL".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_BRYAR_PISTOL as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_BLASTER".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_BLASTER as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_DISRUPTOR".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_DISRUPTOR as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_BOWCASTER".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_BOWCASTER as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_REPEATER".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_REPEATER as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_DEMP2".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_DEMP2 as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_FLECHETTE".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_FLECHETTE as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(
            group,
            c"WP_ROCKET_LAUNCHER".as_ptr() as *mut c_char,
            readbuf,
        ) != 0
        {
            (*bs).botWeaponWeights[WP_ROCKET_LAUNCHER as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_THERMAL".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_THERMAL as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_TRIP_MINE".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_TRIP_MINE as usize] = atoi(readbuf) as f32;
        }

        if GetPairedValue(group, c"WP_DET_PACK".as_ptr() as *mut c_char, readbuf) != 0 {
            (*bs).botWeaponWeights[WP_DET_PACK as usize] = atoi(readbuf) as f32;
        }
    }

    (*bs).lovednum = 0;

    if GetValueGroup(buf, c"EmotionalAttachments".as_ptr() as *mut c_char, group) != 0 {
        ParseEmotionalAttachments(bs, group);
    }

    B_TempFree(131072); //buf
    B_TempFree(1024); //readbuf
    B_TempFree(65536); //group
    trap::FS_FCloseFile(f);
}
