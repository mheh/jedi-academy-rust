//! Faithful port of `q_shared.c` — stateless string, parsing, and info-string
//! support routines included in every game/cgame/ui module.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/q_shared.c`. These functions operate on
//! raw C `char` buffers (`*const c_char` / `*mut c_char`) exactly as the originals
//! do, so the byte-for-byte semantics other ported code relies on — fixed-size
//! buffers, NUL termination, in-place edits, truncation on overflow — are
//! preserved. Idiomatic `&str`/`String` redesign is deferred to Stage 3
//! (`roadmap/stage-3-rewrites-entity-state/04-errors-and-strings.md`).
//!
//! Names mirror the C originals (hence the `non_snake_case` allow) so the two can
//! be diffed during parity checks. Every function is verified bit-/byte-exact
//! against the extracted real C in `oracle/q_shared_oracle.c`.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)] // C type names (`flagStatus_t`, ...) kept verbatim

use core::ffi::{c_char, c_int};
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use super::g_main::{Com_Error, Com_Printf};
use super::q_shared_h::{
    byte, qboolean, qint64, stringID_table_t, vec4_t, Q_IsColorString, BIG_INFO_KEY,
    BIG_INFO_STRING, BIG_INFO_VALUE, ERR_DROP, ERR_FATAL, MAX_INFO_KEY, MAX_INFO_STRING,
    MAX_INFO_VALUE, MAX_QPATH, MAX_TOKEN_CHARS, QFALSE, QTRUE,
};

// Native libc ctype helpers used by the case-folding string routines. The native
// game build pulls these from <ctype.h>; calling the same libc functions the
// oracle uses keeps `Q_strlwr`/`Q_strupr` byte-exact (including the platform's
// behavior for the out-of-range args a signed `char` produces).
extern "C" {
    fn tolower(c: c_int) -> c_int;
    fn toupper(c: c_int) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
}

/// `strlen` over a raw C string, faithful to the original's `strlen(...)` calls.
///
/// # Safety
/// `s` must point to a NUL-terminated buffer.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// `strchr` — first occurrence of `c`, or null. Matches libc (with `c == 0`
/// returning the pointer to the terminating NUL).
///
/// # Safety
/// `s` must point to a NUL-terminated buffer.
unsafe fn c_strchr(s: *const c_char, c: c_int) -> *const c_char {
    let cc = c as c_char;
    let mut p = s;
    while *p != 0 {
        if *p == cc {
            return p;
        }
        p = p.add(1);
    }
    if cc == 0 {
        p
    } else {
        core::ptr::null()
    }
}

/// `strcmp` — byte compare as `unsigned char` (libc semantics), 0 iff equal.
///
/// # Safety
/// `a` and `b` must point to NUL-terminated buffers.
unsafe fn c_strcmp(mut a: *const c_char, mut b: *const c_char) -> c_int {
    loop {
        let ca = *a as u8 as c_int;
        let cb = *b as u8 as c_int;
        if ca != cb {
            return ca - cb;
        }
        if ca == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

/// `strcpy` — copy `src` (including the NUL) to `dst`. Forward byte copy, which is
/// what the original relies on for the `strcpy(start, s)` "remove this part" move
/// where `start` precedes `s` in the same buffer.
///
/// # Safety
/// `dst` must be writable for `strlen(src)+1` bytes; `src` NUL-terminated.
unsafe fn c_strcpy(dst: *mut c_char, src: *const c_char) {
    let mut d = dst;
    let mut s = src;
    loop {
        *d = *s;
        if *s == 0 {
            break;
        }
        d = d.add(1);
        s = s.add(1);
    }
}

/// `strcat` — append `src` to `dst` (libc semantics).
///
/// # Safety
/// `dst` must be NUL-terminated and writable for `strlen(dst)+strlen(src)+1`
/// bytes; `src` NUL-terminated.
unsafe fn c_strcat(dst: *mut c_char, src: *const c_char) {
    let mut d = dst.add(c_strlen(dst));
    let mut s = src;
    loop {
        *d = *s;
        if *s == 0 {
            break;
        }
        d = d.add(1);
        s = s.add(1);
    }
}

/// `int GetIDForString ( stringID_table_t *table, const char *string )` — id for
/// the row whose name matches `string` (case-insensitive), or -1. Scans until a
/// row with a null/empty name.
///
/// # Safety
/// `table` must point to a `{ NULL, _ }`/`{ "", _ }`-terminated array; `string`
/// NUL-terminated.
pub unsafe fn GetIDForString(table: *const stringID_table_t, string: *const c_char) -> c_int {
    let mut index = 0isize;

    while !(*table.offset(index)).name.is_null() && *(*table.offset(index)).name != 0 {
        if Q_stricmp((*table.offset(index)).name, string) == 0 {
            return (*table.offset(index)).id;
        }
        index += 1;
    }

    -1
}

/// `const char *GetStringForID( stringID_table_t *table, int id )` — name for the
/// row with `id`, or null.
///
/// # Safety
/// `table` must point to a `{ NULL, _ }`/`{ "", _ }`-terminated array.
pub unsafe fn GetStringForID(table: *const stringID_table_t, id: c_int) -> *const c_char {
    let mut index = 0isize;

    while !(*table.offset(index)).name.is_null() && *(*table.offset(index)).name != 0 {
        if (*table.offset(index)).id == id {
            return (*table.offset(index)).name;
        }
        index += 1;
    }

    core::ptr::null()
}

/// `int Com_Clampi( int min, int max, int value )` — clamp an int to `[min,max]`.
pub fn Com_Clampi(min: c_int, max: c_int, value: c_int) -> c_int {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

/// `float Com_Clamp( float min, float max, float value )`.
pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

/// `char *COM_SkipPath (char *pathname)` — pointer to the basename (past the
/// last `/`).
///
/// # Safety
/// `pathname` must point to a NUL-terminated buffer.
pub unsafe fn COM_SkipPath(pathname: *mut c_char) -> *mut c_char {
    let mut pathname = pathname;
    let mut last = pathname;
    while *pathname != 0 {
        if *pathname == b'/' as c_char {
            last = pathname.add(1);
        }
        pathname = pathname.add(1);
    }
    last
}

/// `void COM_StripExtension( const char *in, char *out )` — copy `in` into `out`
/// up to the first `.`.
///
/// # Safety
/// `in_` must be NUL-terminated; `out` must be writable for the copied length + 1.
pub unsafe fn COM_StripExtension(mut in_: *const c_char, mut out: *mut c_char) {
    while *in_ != 0 && *in_ != b'.' as c_char {
        *out = *in_;
        out = out.add(1);
        in_ = in_.add(1);
    }
    *out = 0;
}

/// `void COM_DefaultExtension (char *path, int maxSize, const char *extension )`
/// — append `extension` (which must include the `.`) unless `path` already has
/// one.
///
/// # Safety
/// `path` must be a writable, NUL-terminated buffer of `maxSize` bytes. As in the
/// C, an empty `path` reads one byte before the buffer (UB) — callers pass
/// non-empty paths.
pub unsafe fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    let mut old_path = [0 as c_char; MAX_QPATH];

    //
    // if path doesn't have a .EXT, append extension
    // (extension should include the .)
    //
    // src = path + strlen(path) - 1;
    let len = c_strlen(path);
    let mut src = path.add(len).sub(1);

    while *src != b'/' as c_char && src != path {
        if *src == b'.' as c_char {
            return; // it has an extension
        }
        src = src.sub(1);
    }

    Q_strncpyz(old_path.as_mut_ptr(), path, old_path.len() as c_int);
    Com_sprintf(
        path,
        maxSize,
        format_args!("{}{}", Sz(old_path.as_ptr()), Sz(extension)),
    );
}

// ============================================================================
//
//                     BYTE ORDER FUNCTIONS
//
// ============================================================================
//
// (The function-pointer dispatch table — `_BigShort`/`_LittleShort`/... and
// `Swap_Init` — is commented out in the original q_shared.c, so only these
// concrete swap/no-swap routines are compiled. Each intermediate is computed in
// `c_int` to mirror C's integer promotion of `short`/`byte` operands; the final
// reconstruction uses `wrapping_add` because the original's signed `int`
// additions overflow by design and wrap in practice.)

/// `short ShortSwap (short l)`.
pub fn ShortSwap(l: i16) -> i16 {
    let b1: byte = ((l as c_int) & 255) as byte;
    let b2: byte = (((l as c_int) >> 8) & 255) as byte;

    (((b1 as c_int) << 8) + b2 as c_int) as i16
}

/// `short ShortNoSwap (short l)`.
pub fn ShortNoSwap(l: i16) -> i16 {
    l
}

/// `int LongSwap (int l)`.
pub fn LongSwap(l: c_int) -> c_int {
    let b1: byte = (l & 255) as byte;
    let b2: byte = ((l >> 8) & 255) as byte;
    let b3: byte = ((l >> 16) & 255) as byte;
    let b4: byte = ((l >> 24) & 255) as byte;

    ((b1 as c_int) << 24)
        .wrapping_add((b2 as c_int) << 16)
        .wrapping_add((b3 as c_int) << 8)
        .wrapping_add(b4 as c_int)
}

/// `int LongNoSwap (int l)`.
pub fn LongNoSwap(l: c_int) -> c_int {
    l
}

/// `qint64 Long64Swap (qint64 ll)` — reverse the 8 bytes.
pub fn Long64Swap(ll: qint64) -> qint64 {
    qint64 {
        b0: ll.b7,
        b1: ll.b6,
        b2: ll.b5,
        b3: ll.b4,
        b4: ll.b3,
        b5: ll.b2,
        b6: ll.b1,
        b7: ll.b0,
    }
}

/// `qint64 Long64NoSwap (qint64 ll)`.
pub fn Long64NoSwap(ll: qint64) -> qint64 {
    ll
}

/// `float FloatSwap (const float *f)` — byte-swap a float via the `_FloatByteUnion`
/// (`{ float f; unsigned int i; }`): read the bits, `LongSwap` them, reinterpret.
///
/// # Safety
/// `f` must point to a readable `f32`.
pub unsafe fn FloatSwap(f: *const f32) -> f32 {
    // out.i = LongSwap(in->i); the union's `i` is `unsigned int`.
    let in_i = (*f).to_bits();
    let out_i = LongSwap(in_i as c_int) as u32;
    f32::from_bits(out_i)
}

/// `float FloatNoSwap (const float *f)`.
///
/// # Safety
/// `f` must point to a readable `f32`.
pub unsafe fn FloatNoSwap(f: *const f32) -> f32 {
    *f
}

// ============================================================================
//
//                     PARSING
//
// ============================================================================
//
// The original's file-scope parser state. `com_token` is the scratch the parse
// routines fill and return a pointer to; `com_lines` is the running line count;
// `com_parsename` is the session name used in diagnostics. Single-threaded game,
// so direct static storage matches the C (atomics for the integer counter, and
// `addr_of_mut!` for the buffers, avoid the `static_mut_refs` lint).

// static char com_token[MAX_TOKEN_CHARS];
static mut COM_TOKEN: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
// static char com_parsename[MAX_TOKEN_CHARS];
static mut COM_PARSENAME: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
// static int com_lines;
static COM_LINES: AtomicI32 = AtomicI32::new(0);

#[inline]
fn com_token_ptr() -> *mut c_char {
    core::ptr::addr_of_mut!(COM_TOKEN) as *mut c_char
}

/// `void COM_BeginParseSession( const char *name )` — reset the line counter and
/// record the session name for diagnostics.
///
/// # Safety
/// `name` must be NUL-terminated.
pub unsafe fn COM_BeginParseSession(name: *const c_char) {
    COM_LINES.store(0, Ordering::Relaxed);
    let parsename = core::ptr::addr_of_mut!(COM_PARSENAME) as *mut c_char;
    Com_sprintf(
        parsename,
        MAX_TOKEN_CHARS as c_int,
        format_args!("{}", Sz(name)),
    );
}

/// `int COM_GetCurrentParseLine( void )`.
pub fn COM_GetCurrentParseLine() -> c_int {
    COM_LINES.load(Ordering::Relaxed)
}

/// `char *COM_Parse( const char **data_p )` — parse one token, allowing line
/// breaks. Never returns null; returns an empty token at end of data.
///
/// # Safety
/// `*data_p` must be null or NUL-terminated.
pub unsafe fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char {
    COM_ParseExt(data_p, QTRUE)
}

/// `int COM_ParseInfos( char *buf, int max, char infos[][MAX_INFO_STRING] )` —
/// parse a sequence of `{ key value ... }` info blocks from `buf` into `infos`,
/// one `MAX_INFO_STRING`-byte infostring per block, returning the count parsed.
///
/// In the original this is wrapped in `#if 0 // no longer used`; ported faithfully
/// anyway. `infos` is the decayed `char (*)[MAX_INFO_STRING]` (a pointer to rows);
/// `infos[count]` selects one writable `MAX_INFO_STRING` row.
///
/// # Safety
/// `buf` must be null or NUL-terminated. `infos` must point to at least `max`
/// writable `MAX_INFO_STRING`-byte rows.
pub unsafe fn COM_ParseInfos(
    buf: *const c_char,
    max: c_int,
    infos: *mut [c_char; MAX_INFO_STRING],
) -> c_int {
    let mut token: *mut c_char;
    let mut count: c_int;
    let mut key = [0 as c_char; MAX_TOKEN_CHARS];

    // COM_Parse takes `const char **`; mirror C's `&buf` by tracking a local.
    let mut buf = buf;

    count = 0;

    loop {
        token = COM_Parse(&mut buf);
        if *token == 0 {
            // if ( !token[0] )
            break;
        }
        if c_strcmp(token, c"{".as_ptr()) != 0 {
            Com_Printf("Missing { in info file\n");
            break;
        }

        if count == max {
            Com_Printf("Max infos exceeded\n");
            break;
        }

        let info = infos.offset(count as isize) as *mut c_char; // infos[count]
        *info = 0; // infos[count][0] = 0
        loop {
            token = COM_ParseExt(&mut buf, QTRUE);
            if *token == 0 {
                // if ( !token[0] )
                Com_Printf("Unexpected end of info file\n");
                break;
            }
            if c_strcmp(token, c"}".as_ptr()) == 0 {
                break;
            }
            Q_strncpyz(
                key.as_mut_ptr(),
                token,
                core::mem::size_of_val(&key) as c_int,
            );

            token = COM_ParseExt(&mut buf, QFALSE);
            if *token == 0 {
                // if ( !token[0] )
                c_strcpy(token, c"<NULL>".as_ptr());
            }
            Info_SetValueForKey(info, key.as_ptr(), token);
        }
        count += 1;
    }

    count
}

/// `void COM_ParseError( char *format, ... )` — print a parse error tagged with
/// the session name and current line. Varargs → `fmt::Arguments` (see [`Sz`] /
/// [`Com_sprintf`]). Output-only; no oracle parity test.
///
/// # Safety
/// Reads the shared parser state; safe to call after `COM_BeginParseSession`.
pub unsafe fn COM_ParseError(args: fmt::Arguments) {
    let string = args.to_string();
    let parsename = core::ptr::addr_of!(COM_PARSENAME) as *const c_char;
    Com_Printf(&format!(
        "ERROR: {}, line {}: {}\n",
        Sz(parsename),
        COM_LINES.load(Ordering::Relaxed),
        string
    ));
}

/// `void COM_ParseWarning( char *format, ... )` — as `COM_ParseError` but tagged
/// `WARNING`.
///
/// # Safety
/// Reads the shared parser state; safe to call after `COM_BeginParseSession`.
pub unsafe fn COM_ParseWarning(args: fmt::Arguments) {
    let string = args.to_string();
    let parsename = core::ptr::addr_of!(COM_PARSENAME) as *const c_char;
    Com_Printf(&format!(
        "WARNING: {}, line {}: {}\n",
        Sz(parsename),
        COM_LINES.load(Ordering::Relaxed),
        string
    ));
}

/// `const char *SkipWhitespace( const char *data, qboolean *hasNewLines )` —
/// advance past bytes `<= ' '` (which, with sign-extended `char`, includes
/// high-bit bytes), counting newlines. Returns null at end of data.
///
/// # Safety
/// `data` must be NUL-terminated; `hasNewLines` writable.
pub unsafe fn SkipWhitespace(mut data: *const c_char, hasNewLines: *mut qboolean) -> *const c_char {
    loop {
        let c = *data as c_int; // while( (c = *data) <= ' ')
        if c > b' ' as c_int {
            break;
        }
        if c == 0 {
            return core::ptr::null();
        }
        if c == b'\n' as c_int {
            COM_LINES.fetch_add(1, Ordering::Relaxed);
            *hasNewLines = QTRUE;
        }
        data = data.add(1);
    }
    data
}

/// `char *COM_ParseExt( const char **data_p, qboolean allowLineBreaks )` — parse
/// one token (skipping `//` and `/* */` comments), optionally stopping at a line
/// break. Returns a pointer into the shared `com_token`.
///
/// NOTE: a quoted token of exactly `MAX_TOKEN_CHARS` chars makes the original
/// write `com_token[MAX_TOKEN_CHARS]` (one past the array) — a latent overflow
/// carried over faithfully; callers/tests stay below the limit.
///
/// # Safety
/// `*data_p` must be null or NUL-terminated. The returned pointer aliases shared
/// static storage, valid until the next parse call.
pub unsafe fn COM_ParseExt(data_p: *mut *const c_char, allowLineBreaks: qboolean) -> *mut c_char {
    // (C: `int c = 0, len;` — the `= 0` is dead, c is always set before its first
    // read; left uninitialized here to keep the build warning-free.)
    let mut c: c_int;
    let mut len: c_int;
    let mut hasNewLines: qboolean = QFALSE;
    let com_token = com_token_ptr();

    let mut data = *data_p;
    len = 0;
    *com_token = 0; // com_token[0] = 0

    // make sure incoming data is valid
    if data.is_null() {
        *data_p = core::ptr::null();
        return com_token;
    }

    loop {
        // skip whitespace
        data = SkipWhitespace(data, &mut hasNewLines);
        if data.is_null() {
            *data_p = core::ptr::null();
            return com_token;
        }
        if hasNewLines != QFALSE && allowLineBreaks == QFALSE {
            *data_p = data;
            return com_token;
        }

        c = *data as c_int;

        // skip double slash comments
        if c == b'/' as c_int && *data.add(1) == b'/' as c_char {
            data = data.add(2);
            while *data != 0 && *data != b'\n' as c_char {
                data = data.add(1);
            }
        }
        // skip /* */ comments
        else if c == b'/' as c_int && *data.add(1) == b'*' as c_char {
            data = data.add(2);
            while *data != 0 && (*data != b'*' as c_char || *data.add(1) != b'/' as c_char) {
                data = data.add(1);
            }
            if *data != 0 {
                data = data.add(2);
            }
        } else {
            break;
        }
    }

    // handle quoted strings
    if c == b'"' as c_int {
        data = data.add(1);
        loop {
            c = *data as c_int;
            data = data.add(1);
            if c == b'"' as c_int || c == 0 {
                *com_token.add(len as usize) = 0;
                *data_p = data;
                return com_token;
            }
            if len < MAX_TOKEN_CHARS as c_int {
                *com_token.add(len as usize) = c as c_char;
                len += 1;
            }
        }
    }

    // parse a regular word
    loop {
        if len < MAX_TOKEN_CHARS as c_int {
            *com_token.add(len as usize) = c as c_char;
            len += 1;
        }
        data = data.add(1);
        c = *data as c_int;
        if c == b'\n' as c_int {
            COM_LINES.fetch_add(1, Ordering::Relaxed);
        }
        if !(c > 32) {
            break;
        }
    }

    if len == MAX_TOKEN_CHARS as c_int {
        len = 0;
    }
    *com_token.add(len as usize) = 0;

    *data_p = data;
    com_token
}

/// `int COM_Compress( char *data_p )` — strip comments and collapse runs of
/// whitespace in place; returns the compressed length.
///
/// # Safety
/// `data_p` must point to a writable NUL-terminated buffer (the original writes
/// the final NUL unconditionally, so a null pointer would fault — callers don't
/// pass one).
pub unsafe fn COM_Compress(data_p: *mut c_char) -> c_int {
    let mut inp = data_p;
    let mut out = data_p;
    let mut newline = QFALSE;
    let mut whitespace = QFALSE;

    if !inp.is_null() {
        loop {
            let mut c = *inp as c_int;
            if c == 0 {
                break;
            }
            // skip double slash comments
            if c == b'/' as c_int && *inp.add(1) == b'/' as c_char {
                while *inp != 0 && *inp != b'\n' as c_char {
                    inp = inp.add(1);
                }
            // skip /* */ comments
            } else if c == b'/' as c_int && *inp.add(1) == b'*' as c_char {
                while *inp != 0 && (*inp != b'*' as c_char || *inp.add(1) != b'/' as c_char) {
                    inp = inp.add(1);
                }
                if *inp != 0 {
                    inp = inp.add(2);
                }
            // record when we hit a newline
            } else if c == b'\n' as c_int || c == b'\r' as c_int {
                newline = QTRUE;
                inp = inp.add(1);
            // record when we hit whitespace
            } else if c == b' ' as c_int || c == b'\t' as c_int {
                whitespace = QTRUE;
                inp = inp.add(1);
            // an actual token
            } else {
                // if we have a pending newline, emit it (and it counts as whitespace)
                if newline != QFALSE {
                    *out = b'\n' as c_char;
                    out = out.add(1);
                    newline = QFALSE;
                    whitespace = QFALSE;
                }
                if whitespace != QFALSE {
                    *out = b' ' as c_char;
                    out = out.add(1);
                    whitespace = QFALSE;
                }

                // copy quoted strings unmolested
                if c == b'"' as c_int {
                    *out = c as c_char;
                    out = out.add(1);
                    inp = inp.add(1);
                    loop {
                        c = *inp as c_int;
                        if c != 0 && c != b'"' as c_int {
                            *out = c as c_char;
                            out = out.add(1);
                            inp = inp.add(1);
                        } else {
                            break;
                        }
                    }
                    if c == b'"' as c_int {
                        *out = c as c_char;
                        out = out.add(1);
                        inp = inp.add(1);
                    }
                } else {
                    *out = c as c_char;
                    out = out.add(1);
                    inp = inp.add(1);
                }
            }
        }
    }
    *out = 0;
    out.offset_from(data_p) as c_int
}

/// `qboolean COM_ParseString( const char **data, const char **s )`.
///
/// NOTE (carried-over bug): the original tests `s[0] == 0`, i.e. whether the
/// *pointer* returned by `COM_ParseExt` is null — which it never is (it returns
/// `com_token`). So the "unexpected EOF" branch is dead and the function always
/// returns `qfalse`. Faithful to the original.
///
/// # Safety
/// `*data` must be null or NUL-terminated; `s` writable.
pub unsafe fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> qboolean {
    *s = COM_ParseExt(data, QFALSE) as *const c_char;
    if (*s).is_null() {
        Com_Printf("unexpected EOF\n");
        return QTRUE;
    }
    QFALSE
}

/// `qboolean COM_ParseInt( const char **data, int *i )` — parse one token as an
/// int via `atoi`; returns `qtrue` (after a diagnostic) at end of data.
///
/// # Safety
/// `*data` must be null or NUL-terminated; `i` writable.
pub unsafe fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> qboolean {
    let token = COM_ParseExt(data, QFALSE);
    if *token == 0 {
        Com_Printf("unexpected EOF\n");
        return QTRUE;
    }

    *i = atoi(token);
    QFALSE
}

/// `qboolean COM_ParseFloat( const char **data, float *f )` — parse one token as a
/// float via `atof`; returns `qtrue` (after a diagnostic) at end of data.
///
/// # Safety
/// `*data` must be null or NUL-terminated; `f` writable.
pub unsafe fn COM_ParseFloat(data: *mut *const c_char, f: *mut f32) -> qboolean {
    let token = COM_ParseExt(data, QFALSE);
    if *token == 0 {
        Com_Printf("unexpected EOF\n");
        return QTRUE;
    }

    *f = atof(token) as f32;
    QFALSE
}

/// `qboolean COM_ParseVec4( const char **buffer, vec4_t *c)` — parse four floats.
///
/// # Safety
/// `*buffer` must be null or NUL-terminated; `c` writable.
pub unsafe fn COM_ParseVec4(buffer: *mut *const c_char, c: *mut vec4_t) -> qboolean {
    let mut f: f32 = 0.0;
    for i in 0..4 {
        if COM_ParseFloat(buffer, &mut f) != QFALSE {
            return QTRUE;
        }
        (*c)[i] = f;
    }
    QFALSE
}

/// `void COM_MatchToken( const char **buf_p, char *match )` — parse a token and
/// `Com_Error` if it doesn't equal `match`.
///
/// # Safety
/// `*buf_p` must be null or NUL-terminated; `match_` NUL-terminated.
pub unsafe fn COM_MatchToken(buf_p: *mut *const c_char, match_: *const c_char) {
    let token = COM_Parse(buf_p);
    if c_strcmp(token, match_) != 0 {
        Com_Error(
            ERR_DROP,
            &format!("MatchToken: {} != {}", Sz(token), Sz(match_)),
        );
    }
}

/// `void SkipBracedSection (const char **program)` — the next token must be `{`;
/// skip to the matching `}` (nested braces handled).
///
/// # Safety
/// `*program` must be null or NUL-terminated.
pub unsafe fn SkipBracedSection(program: *mut *const c_char) {
    let mut depth = 0;
    loop {
        let token = COM_ParseExt(program, QTRUE);
        if *token.add(1) == 0 {
            if *token == b'{' as c_char {
                depth += 1;
            } else if *token == b'}' as c_char {
                depth -= 1;
            }
        }
        if !(depth != 0 && !(*program).is_null()) {
            break;
        }
    }
}

/// `void SkipRestOfLine ( const char **data )` — advance past the next newline
/// (or to end of data), counting the line.
///
/// # Safety
/// `*data` must be NUL-terminated.
pub unsafe fn SkipRestOfLine(data: *mut *const c_char) {
    let mut p = *data;
    loop {
        let c = *p as c_int; // c = *p++
        p = p.add(1);
        if c == 0 {
            break;
        }
        if c == b'\n' as c_int {
            COM_LINES.fetch_add(1, Ordering::Relaxed);
            break;
        }
    }
    *data = p;
}

/// `void Parse1DMatrix (const char **buf_p, int x, float *m)` — parse `( f f .. )`
/// into `m[0..x]`.
///
/// # Safety
/// `*buf_p` must be null or NUL-terminated; `m` writable for `x` floats.
pub unsafe fn Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f32) {
    let open = b"(\0".as_ptr() as *const c_char;
    let close = b")\0".as_ptr() as *const c_char;

    COM_MatchToken(buf_p, open);

    for i in 0..x {
        let token = COM_Parse(buf_p);
        *m.add(i as usize) = atof(token) as f32;
    }

    COM_MatchToken(buf_p, close);
}

/// `void Parse2DMatrix (const char **buf_p, int y, int x, float *m)`.
///
/// # Safety
/// `*buf_p` must be null or NUL-terminated; `m` writable for `y*x` floats.
pub unsafe fn Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f32) {
    let open = b"(\0".as_ptr() as *const c_char;
    let close = b")\0".as_ptr() as *const c_char;

    COM_MatchToken(buf_p, open);

    for i in 0..y {
        Parse1DMatrix(buf_p, x, m.add((i * x) as usize));
    }

    COM_MatchToken(buf_p, close);
}

/// `void Parse3DMatrix (const char **buf_p, int z, int y, int x, float *m)`.
///
/// # Safety
/// `*buf_p` must be null or NUL-terminated; `m` writable for `z*y*x` floats.
pub unsafe fn Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f32) {
    let open = b"(\0".as_ptr() as *const c_char;
    let close = b")\0".as_ptr() as *const c_char;

    COM_MatchToken(buf_p, open);

    for i in 0..z {
        Parse2DMatrix(buf_p, y, x, m.add((i * x * y) as usize));
    }

    COM_MatchToken(buf_p, close);
}

// ============================================================================
//
//                     LIBRARY REPLACEMENT FUNCTIONS
//
// ============================================================================

/// `int Q_isprint( int c )` — printable ASCII (space through `~`).
pub fn Q_isprint(c: c_int) -> c_int {
    if c >= 0x20 && c <= 0x7E {
        return 1;
    }
    0
}

/// `int Q_islower( int c )`.
pub fn Q_islower(c: c_int) -> c_int {
    if c >= 'a' as c_int && c <= 'z' as c_int {
        return 1;
    }
    0
}

/// `int Q_isupper( int c )`.
pub fn Q_isupper(c: c_int) -> c_int {
    if c >= 'A' as c_int && c <= 'Z' as c_int {
        return 1;
    }
    0
}

/// `int Q_isalpha( int c )`.
pub fn Q_isalpha(c: c_int) -> c_int {
    if (c >= 'a' as c_int && c <= 'z' as c_int) || (c >= 'A' as c_int && c <= 'Z' as c_int) {
        return 1;
    }
    0
}

/// `char* Q_strrchr( const char* string, int c )` — last occurrence of `c`.
/// Returns a pointer to the terminating NUL when `c == 0` (faithful quirk), else
/// null if not found.
///
/// # Safety
/// `string` must point to a NUL-terminated buffer.
pub unsafe fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char {
    let cc = c as c_char; // char cc = c;  (truncates to char)
    let mut s = string as *mut c_char;
    let mut sp: *mut c_char = core::ptr::null_mut();

    while *s != 0 {
        if *s == cc {
            sp = s;
        }
        s = s.add(1);
    }
    if cc == 0 {
        sp = s;
    }

    sp
}

/// `void Q_strncpyz( char *dest, const char *src, int destsize )` — safe strncpy
/// that always NUL-terminates.
///
/// Replicates `strncpy(dest, src, destsize-1)` faithfully, **including** its
/// zero-padding of the remaining bytes when `src` is shorter, then forces the
/// final byte to 0.
///
/// # Safety
/// `dest` must be writable for `destsize` bytes and `src` NUL-terminated (the C
/// `Com_Error`-aborts on NULL/`destsize < 1`, which this preserves).
pub unsafe fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int) {
    // bk001129 - also NULL dest
    if dest.is_null() {
        Com_Error(ERR_FATAL, "Q_strncpyz: NULL dest");
    }
    if src.is_null() {
        Com_Error(ERR_FATAL, "Q_strncpyz: NULL src");
    }
    if destsize < 1 {
        Com_Error(ERR_FATAL, "Q_strncpyz: destsize < 1");
    }

    // strncpy( dest, src, destsize-1 ): copy until NUL or n bytes, zero-padding
    // the rest of the n bytes; then dest[destsize-1] = 0.
    let n = (destsize - 1) as usize;
    let mut i = 0usize;
    while i < n && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dest.add(i) = 0;
        i += 1;
    }
    *dest.add((destsize - 1) as usize) = 0;
}

/// `int Q_stricmpn (const char *s1, const char *s2, int n)` — case-insensitive
/// compare of up to `n` chars. Returns -1/0/1.
///
/// # Safety
/// Each non-null pointer must reference a NUL-terminated buffer.
pub unsafe fn Q_stricmpn(mut s1: *const c_char, mut s2: *const c_char, mut n: c_int) -> c_int {
    // bk001129 - moved in 1.17 fix not in id codebase
    if s1.is_null() {
        if s2.is_null() {
            return 0;
        } else {
            return -1;
        }
    } else if s2.is_null() {
        return 1;
    }

    loop {
        let mut c1 = *s1 as c_int;
        s1 = s1.add(1);
        let mut c2 = *s2 as c_int;
        s2 = s2.add(1);

        let old_n = n; // if (!n--)  — tests n, then decrements
        n = n.wrapping_sub(1);
        if old_n == 0 {
            return 0; // strings are equal until end point
        }

        if c1 != c2 {
            if c1 >= 'a' as c_int && c1 <= 'z' as c_int {
                c1 -= 'a' as c_int - 'A' as c_int;
            }
            if c2 >= 'a' as c_int && c2 <= 'z' as c_int {
                c2 -= 'a' as c_int - 'A' as c_int;
            }
            if c1 != c2 {
                return if c1 < c2 { -1 } else { 1 };
            }
        }

        if c1 == 0 {
            break;
        }
    }

    0 // strings are equal
}

/// `int Q_strncmp (const char *s1, const char *s2, int n)` — case-sensitive
/// compare of up to `n` chars. Returns -1/0/1. (No NULL guard, matching the C.)
///
/// # Safety
/// `s1` and `s2` must reference NUL-terminated buffers.
pub unsafe fn Q_strncmp(mut s1: *const c_char, mut s2: *const c_char, mut n: c_int) -> c_int {
    loop {
        let c1 = *s1 as c_int;
        s1 = s1.add(1);
        let c2 = *s2 as c_int;
        s2 = s2.add(1);

        let old_n = n;
        n = n.wrapping_sub(1);
        if old_n == 0 {
            return 0; // strings are equal until end point
        }

        if c1 != c2 {
            return if c1 < c2 { -1 } else { 1 };
        }

        if c1 == 0 {
            break;
        }
    }

    0 // strings are equal
}

/// `int Q_stricmp (const char *s1, const char *s2)` — case-insensitive full
/// compare (`Q_stricmpn` with a huge bound), or -1 if either pointer is null.
///
/// # Safety
/// Each non-null pointer must reference a NUL-terminated buffer.
pub unsafe fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if !s1.is_null() && !s2.is_null() {
        Q_stricmpn(s1, s2, 99999)
    } else {
        -1
    }
}

/// `char *Q_strlwr( char *s1 )` — lowercase in place, returns `s1`.
///
/// # Safety
/// `s1` must point to a writable NUL-terminated buffer.
pub unsafe fn Q_strlwr(s1: *mut c_char) -> *mut c_char {
    let mut s = s1;
    while *s != 0 {
        *s = tolower(*s as c_int) as c_char;
        s = s.add(1);
    }
    s1
}

/// `char *Q_strupr( char *s1 )` — uppercase in place, returns `s1`.
///
/// # Safety
/// `s1` must point to a writable NUL-terminated buffer.
pub unsafe fn Q_strupr(s1: *mut c_char) -> *mut c_char {
    let mut s = s1;
    while *s != 0 {
        *s = toupper(*s as c_int) as c_char;
        s = s.add(1);
    }
    s1
}

/// `void Q_strcat( char *dest, int size, const char *src )` — append `src` to
/// `dest`, never exceeding `size` and always NUL-terminating.
///
/// # Safety
/// `dest` must be a writable NUL-terminated buffer of `size` bytes; `src`
/// NUL-terminated. `Com_Error`-aborts if `dest` is already overflowed.
pub unsafe fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char) {
    let l1 = c_strlen(dest) as c_int;
    if l1 >= size {
        Com_Error(ERR_FATAL, "Q_strcat: already overflowed");
    }
    Q_strncpyz(dest.add(l1 as usize), src, size - l1);
}

/// `int Q_PrintStrlen( const char *string )` — visible length, skipping `^N`
/// color codes.
///
/// # Safety
/// `string` must be null or a NUL-terminated buffer.
pub unsafe fn Q_PrintStrlen(string: *const c_char) -> c_int {
    if string.is_null() {
        return 0;
    }

    let mut len = 0;
    let mut p = string;
    while *p != 0 {
        if Q_IsColorString(p) {
            p = p.add(2);
            continue;
        }
        p = p.add(1);
        len += 1;
    }

    len
}

/// Render a raw C string operand inside `format_args!`, emitting its bytes up to
/// the NUL the way the original `%s` consumed a `char*` in `Com_sprintf`/`va`.
///
/// Each byte is emitted as a `char` (Latin-1), so the rendering is byte-exact for
/// ASCII content (the dominant case: paths, keys, ASCII names). A `%s` operand
/// carrying bytes ≥ 0x80 would be re-encoded as multi-byte UTF-8 by Rust's
/// formatter — a known stage-1 limitation of the varargs→`format_args!`
/// deviation, flagged for the stage-3 errors-and-strings redesign. A null pointer
/// renders as empty.
pub struct Sz(pub *const c_char);

impl fmt::Display for Sz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_null() {
            let mut p = self.0;
            // SAFETY: caller guarantees a NUL-terminated buffer (the same contract
            // the C `%s` relied on).
            unsafe {
                while *p != 0 {
                    f.write_char(*p as u8 as char)?;
                    p = p.add(1);
                }
            }
        }
        Ok(())
    }
}

/// `void QDECL Com_sprintf( char *dest, int size, const char *fmt, ...)`.
///
/// Varargs deviation: the C `fmt, ...` + `vsprintf` become a pre-rendered
/// `fmt::Arguments` (callers pass `format_args!`, translating the printf format to
/// Rust's; C-string operands go through [`Sz`]). The faithful tail is preserved:
/// render into the equivalent of the 32000-byte `bigbuffer`, `Com_Error` if it
/// overflowed that, `Com_Printf` a warning if it overflows `size`, then
/// `Q_strncpyz` into `dest`.
///
/// # Safety
/// `dest` must be writable for `size` bytes.
pub unsafe fn Com_sprintf(dest: *mut c_char, size: c_int, args: fmt::Arguments) {
    // char bigbuffer[32000]; len = vsprintf(bigbuffer, fmt, argptr);
    let bigbuffer = args.to_string();
    let len = bigbuffer.len() as c_int;
    if len as usize >= 32000 {
        Com_Error(ERR_FATAL, "Com_sprintf: overflowed bigbuffer");
    }
    if len >= size {
        Com_Printf(&format!("Com_sprintf: overflow of {len} in {size}\n"));
    }
    // Q_strncpyz(dest, bigbuffer, size): copy the rendered bytes (NUL-terminated)
    // bounded by size.
    let mut src: Vec<c_char> = bigbuffer.bytes().map(|b| b as c_char).collect();
    src.push(0);
    Q_strncpyz(dest, src.as_ptr(), size);
}

/// `char * QDECL va( const char *format, ... )` — varargs printf into one of two
/// rotating static buffers, so the result survives until the next-but-one call.
///
/// Varargs deviation as for [`Com_sprintf`]. DEVIATION: the original does an
/// **unbounded** `vsprintf` into a 32000-byte buffer; this bounds the copy to
/// 31999 + a NUL to avoid UB — identical output for the sub-32000-byte inputs
/// that occur in practice. `INDEX` is an atomic mirroring the C `static int
/// index`.
///
/// # Safety
/// The returned pointer is valid only until `va` has been called twice more (it
/// aliases shared static storage), exactly like the original.
pub unsafe fn va(args: fmt::Arguments) -> *mut c_char {
    // static char string[2][32000]; static int index = 0;
    static mut STRING: [[c_char; 32000]; 2] = [[0; 32000]; 2];
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    // buf = string[index & 1]; index++;
    let idx = INDEX.fetch_add(1, Ordering::Relaxed) & 1;
    let rows = core::ptr::addr_of_mut!(STRING) as *mut c_char;
    let buf = rows.add(idx * 32000);

    let rendered = args.to_string();
    let bytes = rendered.as_bytes();
    let n = core::cmp::min(bytes.len(), 31999);
    for i in 0..n {
        *buf.add(i) = bytes[i] as c_char;
    }
    *buf.add(n) = 0;

    buf
}

/// `char *Q_CleanStr( char *string )` — strip color codes and non-printable
/// bytes in place, returns `string`.
///
/// # Safety
/// `string` must point to a writable NUL-terminated buffer.
pub unsafe fn Q_CleanStr(string: *mut c_char) -> *mut c_char {
    let mut s = string;
    let mut d = string;
    loop {
        let c = *s as c_int;
        if c == 0 {
            break;
        }
        if Q_IsColorString(s) {
            s = s.add(1);
        } else if c >= 0x20 && c <= 0x7E {
            *d = c as c_char;
            d = d.add(1);
        }
        s = s.add(1);
    }
    *d = b'\0' as c_char;

    string
}

// ============================================================================
//
//                     INFO STRINGS
//
// ============================================================================

/// `char *Info_ValueForKey( const char *s, const char *key )` — value for `key`
/// in the `\k\v\k\v` info string `s`, or `""`. The result aliases one of two
/// rotating static buffers (so two consecutive calls can be compared).
///
/// # Safety
/// `s` and `key` must be null or NUL-terminated. The returned pointer is read-only
/// and valid only until the next-but-one call (it aliases shared static storage).
pub unsafe fn Info_ValueForKey(mut s: *const c_char, key: *const c_char) -> *mut c_char {
    let mut pkey = [0 as c_char; BIG_INFO_KEY];
    // static char value[2][BIG_INFO_VALUE]; static int valueindex = 0;
    static mut VALUE: [[c_char; BIG_INFO_VALUE]; 2] = [[0; BIG_INFO_VALUE]; 2];
    static VALUEINDEX: AtomicUsize = AtomicUsize::new(0);
    // empty-string literal returned by reference, mirroring the C `return ""`.
    static EMPTY: c_char = 0;
    let empty = || (&EMPTY as *const c_char) as *mut c_char;

    if s.is_null() || key.is_null() {
        return empty();
    }

    if c_strlen(s) >= BIG_INFO_STRING {
        Com_Error(ERR_DROP, "Info_ValueForKey: oversize infostring");
    }

    // valueindex ^= 1;  (fetch_xor returns the prior value)
    let vi = VALUEINDEX.fetch_xor(1, Ordering::Relaxed) ^ 1;
    let value_row = (core::ptr::addr_of_mut!(VALUE) as *mut c_char).add(vi * BIG_INFO_VALUE);

    if *s == b'\\' as c_char {
        s = s.add(1);
    }
    loop {
        let mut o = pkey.as_mut_ptr();
        while *s != b'\\' as c_char {
            if *s == 0 {
                return empty();
            }
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;
        s = s.add(1);

        let mut o = value_row;
        while *s != b'\\' as c_char && *s != 0 {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        if Q_stricmp(key, pkey.as_ptr()) == 0 {
            return value_row;
        }

        if *s == 0 {
            break;
        }
        s = s.add(1);
    }

    empty()
}

/// `void Info_NextPair( const char **head, char *key, char *value )` — iterate
/// key/value pairs in an info string, advancing `*head`.
///
/// # Safety
/// `*head` must be NUL-terminated; `key`/`value` must be writable for the longest
/// token + 1.
pub unsafe fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char) {
    let mut s = *head;

    if *s == b'\\' as c_char {
        s = s.add(1);
    }
    *key = 0;
    *value = 0;

    let mut o = key;
    while *s != b'\\' as c_char {
        if *s == 0 {
            *o = 0;
            *head = s;
            return;
        }
        *o = *s;
        o = o.add(1);
        s = s.add(1);
    }
    *o = 0;
    s = s.add(1);

    let mut o = value;
    while *s != b'\\' as c_char && *s != 0 {
        *o = *s;
        o = o.add(1);
        s = s.add(1);
    }
    *o = 0;

    *head = s;
}

/// `qboolean Info_Validate( const char *s )` — reject info strings containing `"`
/// or `;` (which would break the server's parsing).
///
/// # Safety
/// `s` must point to a NUL-terminated buffer.
pub unsafe fn Info_Validate(s: *const c_char) -> qboolean {
    if !c_strchr(s, b'"' as c_int).is_null() {
        return QFALSE;
    }
    if !c_strchr(s, b';' as c_int).is_null() {
        return QFALSE;
    }
    QTRUE
}

/// `void Info_RemoveKey( char *s, const char *key )` — delete `key`'s pair from
/// the info string `s` in place.
///
/// # Safety
/// `s` must be a writable NUL-terminated buffer; `key` NUL-terminated.
pub unsafe fn Info_RemoveKey(mut s: *mut c_char, key: *const c_char) {
    let mut pkey = [0 as c_char; MAX_INFO_KEY];
    let mut value = [0 as c_char; MAX_INFO_VALUE];

    if c_strlen(s) >= MAX_INFO_STRING {
        Com_Error(ERR_DROP, "Info_RemoveKey: oversize infostring");
    }

    if !c_strchr(key, b'\\' as c_int).is_null() {
        return;
    }

    loop {
        let start = s;
        if *s == b'\\' as c_char {
            s = s.add(1);
        }
        let mut o = pkey.as_mut_ptr();
        while *s != b'\\' as c_char {
            if *s == 0 {
                return;
            }
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;
        s = s.add(1);

        let mut o = value.as_mut_ptr();
        // (the C repeats a `if(!*s) return;` here that the loop condition makes
        // dead; omitted as it can never fire)
        while *s != b'\\' as c_char && *s != 0 {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        if c_strcmp(key, pkey.as_ptr()) == 0 {
            c_strcpy(start, s); // remove this part
            return;
        }

        if *s == 0 {
            return;
        }
    }
}

/// `void Info_RemoveKey_Big( char *s, const char *key )` — as `Info_RemoveKey`
/// but for `BIG_INFO_STRING`-sized info strings.
///
/// # Safety
/// `s` must be a writable NUL-terminated buffer; `key` NUL-terminated.
pub unsafe fn Info_RemoveKey_Big(mut s: *mut c_char, key: *const c_char) {
    let mut pkey = [0 as c_char; BIG_INFO_KEY];
    let mut value = [0 as c_char; BIG_INFO_VALUE];

    if c_strlen(s) >= BIG_INFO_STRING {
        Com_Error(ERR_DROP, "Info_RemoveKey_Big: oversize infostring");
    }

    if !c_strchr(key, b'\\' as c_int).is_null() {
        return;
    }

    loop {
        let start = s;
        if *s == b'\\' as c_char {
            s = s.add(1);
        }
        let mut o = pkey.as_mut_ptr();
        while *s != b'\\' as c_char {
            if *s == 0 {
                return;
            }
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;
        s = s.add(1);

        let mut o = value.as_mut_ptr();
        while *s != b'\\' as c_char && *s != 0 {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        if c_strcmp(key, pkey.as_ptr()) == 0 {
            c_strcpy(start, s); // remove this part
            return;
        }

        if *s == 0 {
            return;
        }
    }
}

/// `void Info_SetValueForKey( char *s, const char *key, const char *value )` —
/// add or replace a key/value pair (the new pair is **prepended**).
///
/// # Safety
/// `s` must be a writable NUL-terminated `MAX_INFO_STRING` buffer; `key`/`value`
/// NUL-terminated.
pub unsafe fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char) {
    let mut newi = [0 as c_char; MAX_INFO_STRING];

    if c_strlen(s) >= MAX_INFO_STRING {
        Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring");
    }

    if !c_strchr(key, b'\\' as c_int).is_null() || !c_strchr(value, b'\\' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a \\\n");
        return;
    }
    if !c_strchr(key, b';' as c_int).is_null() || !c_strchr(value, b';' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a semicolon\n");
        return;
    }
    if !c_strchr(key, b'"' as c_int).is_null() || !c_strchr(value, b'"' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a \"\n");
        return;
    }

    Info_RemoveKey(s, key);
    if value.is_null() || c_strlen(value) == 0 {
        return;
    }

    Com_sprintf(
        newi.as_mut_ptr(),
        newi.len() as c_int,
        format_args!("\\{}\\{}", Sz(key), Sz(value)),
    );

    // NOTE: carried-over off-by-one — the original uses `>` (not `>=`) here, so an
    // exact MAX_INFO_STRING total writes one byte past `newi`. Faithful; callers
    // stay below the limit.
    if c_strlen(newi.as_ptr()) + c_strlen(s) > MAX_INFO_STRING {
        Com_Printf("Info string length exceeded\n");
        return;
    }

    c_strcat(newi.as_mut_ptr(), s); // append old info after the new pair
    c_strcpy(s, newi.as_ptr()); // ...then write it all back (new pair first)
}

/// `void Info_SetValueForKey_Big( char *s, const char *key, const char *value )`
/// — as `Info_SetValueForKey` but for `BIG_INFO_STRING` buffers, and the new pair
/// is **appended** (`strcat(s, newi)`) rather than prepended.
///
/// # Safety
/// `s` must be a writable NUL-terminated `BIG_INFO_STRING` buffer; `key`/`value`
/// NUL-terminated.
pub unsafe fn Info_SetValueForKey_Big(s: *mut c_char, key: *const c_char, value: *const c_char) {
    let mut newi = [0 as c_char; BIG_INFO_STRING];

    if c_strlen(s) >= BIG_INFO_STRING {
        // (the original's message says "Info_SetValueForKey", not "_Big" — kept)
        Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring");
    }

    if !c_strchr(key, b'\\' as c_int).is_null() || !c_strchr(value, b'\\' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a \\\n");
        return;
    }
    if !c_strchr(key, b';' as c_int).is_null() || !c_strchr(value, b';' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a semicolon\n");
        return;
    }
    if !c_strchr(key, b'"' as c_int).is_null() || !c_strchr(value, b'"' as c_int).is_null() {
        Com_Printf("Can't use keys or values with a \"\n");
        return;
    }

    Info_RemoveKey_Big(s, key);
    if value.is_null() || c_strlen(value) == 0 {
        return;
    }

    Com_sprintf(
        newi.as_mut_ptr(),
        newi.len() as c_int,
        format_args!("\\{}\\{}", Sz(key), Sz(value)),
    );

    if c_strlen(newi.as_ptr()) + c_strlen(s) > BIG_INFO_STRING {
        Com_Printf("BIG Info string length exceeded\n");
        return;
    }

    c_strcat(s, newi.as_ptr());
}

//====================================================================

/// `#define random()  ((rand () & 0x7fff) / ((float)0x7fff))` — q_shared.h:1492.
///
/// A function here, but a macro in C. The `rand()` underneath is
/// the game's own 15-bit LCG in [`bg_lib`](super::bg_lib) (which overrides libc on
/// every target), **not** the platform RNG — so the result is the same deterministic
/// stream everywhere. Returns `f32`: the C divides by `(float)0x7fff`, so the whole
/// expression is single-precision (no engine/syscall involvement).
pub fn random() -> f32 {
    (super::bg_lib::rand() & 0x7fff) as f32 / 0x7fff as f32
}

/// `#define crandom()  (2.0 * (random() - 0.5))` — q_shared.h:1493.
///
/// A function here, but a macro in C. Returns **`f64`**, not `f32`: the macro's `2.0`
/// and `0.5` are `double` literals, so the `float` [`random`] is promoted and the
/// product is double-precision. This matters at the call sites — e.g.
/// `Use_Target_Delay` computes `ent->random * crandom()` in `double` before
/// truncating to the `int` `nextthink`, so dropping to `f32` here would shift the
/// result. Draws from the same game LCG as [`random`]; no engine call.
pub fn crandom() -> f64 {
    2.0 * (random() as f64 - 0.5)
}

/// `flagStatus_t` (q_shared.h:2958) — CTF / One-Flag-CTF flag state, an `int` alias for the
/// anonymous `_flag_status` enum below.
pub type flagStatus_t = c_int;

/// `_flag_status` enum (q_shared.h:2951) — flag carrier/base state used by the team-flag
/// subsystem in `g_team.c`.
pub const FLAG_ATBASE: flagStatus_t = 0;
/// CTF: flag has been taken.
pub const FLAG_TAKEN: flagStatus_t = 1;
/// One Flag CTF: neutral flag taken by red.
pub const FLAG_TAKEN_RED: flagStatus_t = 2;
/// One Flag CTF: neutral flag taken by blue.
pub const FLAG_TAKEN_BLUE: flagStatus_t = 3;
/// Flag has been dropped (carrier died / dropped it).
pub const FLAG_DROPPED: flagStatus_t = 4;

/// Test-only: serialize tests that touch the shared parser globals (`com_token` /
/// `com_lines`, on both the Rust and oracle sides). `cargo test` runs tests on
/// parallel threads, so without this they race on that static state. `pub(crate)`
/// so sibling modules (e.g. `bg_saberLoad`'s `BG_ParseLiteral` test, which drives
/// the same Rust `COM_ParseExt` globals) can take it too — the `rand_lock`
/// precedent. Poison is ignored (a failing test shouldn't cascade into the rest).
#[cfg(all(test, feature = "oracle"))]
pub(crate) fn parse_lock() -> std::sync::MutexGuard<'static, ()> {
    static PARSE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    PARSE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// Walk every plausible `int` arg to the ctype-style helpers, including the
    /// negative values a signed `char` can produce and the EOF sentinel.
    fn char_inputs() -> impl Iterator<Item = c_int> {
        -260..=260
    }

    /// Build a NUL-terminated C buffer (`Vec<c_char>`) from raw bytes.
    fn cbuf_b(bytes: &[u8]) -> Vec<c_char> {
        let mut v: Vec<c_char> = bytes.iter().map(|&b| b as c_char).collect();
        v.push(0);
        v
    }

    /// Build a NUL-terminated C buffer (`Vec<c_char>`) from a Rust string.
    fn cbuf(s: &str) -> Vec<c_char> {
        cbuf_b(s.as_bytes())
    }

    #[test]
    fn Q_isprint_matches_oracle() {
        for c in char_inputs() {
            assert_eq!(
                Q_isprint(c),
                unsafe { oracle::Q_isprint(c) },
                "Q_isprint({c})"
            );
        }
    }

    #[test]
    fn Q_islower_matches_oracle() {
        for c in char_inputs() {
            assert_eq!(
                Q_islower(c),
                unsafe { oracle::Q_islower(c) },
                "Q_islower({c})"
            );
        }
    }

    #[test]
    fn Q_isupper_matches_oracle() {
        for c in char_inputs() {
            assert_eq!(
                Q_isupper(c),
                unsafe { oracle::Q_isupper(c) },
                "Q_isupper({c})"
            );
        }
    }

    #[test]
    fn Q_isalpha_matches_oracle() {
        for c in char_inputs() {
            assert_eq!(
                Q_isalpha(c),
                unsafe { oracle::Q_isalpha(c) },
                "Q_isalpha({c})"
            );
        }
    }

    #[test]
    fn Com_Clampi_matches_oracle() {
        let vals = [-100, -1, 0, 1, 5, 10, 50, 100];
        for &min in &vals {
            for &max in &vals {
                for &v in &vals {
                    assert_eq!(
                        Com_Clampi(min, max, v),
                        unsafe { oracle::Com_Clampi(min, max, v) },
                        "Com_Clampi({min},{max},{v})"
                    );
                }
            }
        }
    }

    #[test]
    fn Com_Clamp_matches_oracle() {
        let vals = [
            -100.0f32,
            -1.5,
            0.0,
            1.5,
            5.25,
            100.0,
            f32::NAN,
            f32::INFINITY,
        ];
        for &min in &vals {
            for &max in &vals {
                for &v in &vals {
                    let rust = Com_Clamp(min, max, v);
                    let c = unsafe { oracle::Com_Clamp(min, max, v) };
                    assert_eq!(rust.to_bits(), c.to_bits(), "Com_Clamp({min},{max},{v})");
                }
            }
        }
    }

    #[test]
    fn ShortSwap_matches_oracle_exhaustive() {
        for l in i16::MIN..=i16::MAX {
            assert_eq!(
                ShortSwap(l),
                unsafe { oracle::ShortSwap(l) },
                "ShortSwap({l})"
            );
            assert_eq!(
                ShortNoSwap(l),
                unsafe { oracle::ShortNoSwap(l) },
                "ShortNoSwap({l})"
            );
        }
    }

    #[test]
    fn LongSwap_matches_oracle() {
        let samples = [
            0i32,
            1,
            -1,
            i32::MIN,
            i32::MAX,
            0x1234_5678,
            0x0000_00FF,
            0xFF00_0000u32 as i32,
            0x00FF_00FFu32 as i32,
            0xDEAD_BEEFu32 as i32,
            42,
            -42,
            0x7F80_0000,
        ];
        for &l in &samples {
            assert_eq!(LongSwap(l), unsafe { oracle::LongSwap(l) }, "LongSwap({l})");
            assert_eq!(
                LongNoSwap(l),
                unsafe { oracle::LongNoSwap(l) },
                "LongNoSwap({l})"
            );
        }
    }

    #[test]
    fn Long64Swap_matches_oracle() {
        let samples = [
            qint64::default(),
            qint64 {
                b0: 1,
                b1: 2,
                b2: 3,
                b3: 4,
                b4: 5,
                b5: 6,
                b6: 7,
                b7: 8,
            },
            qint64 {
                b0: 0xFF,
                b1: 0,
                b2: 0xAA,
                b3: 0x55,
                b4: 0xDE,
                b5: 0xAD,
                b6: 0xBE,
                b7: 0xEF,
            },
        ];
        for &ll in &samples {
            assert_eq!(
                Long64Swap(ll),
                unsafe { oracle::Long64Swap(ll) },
                "Long64Swap({ll:?})"
            );
            assert_eq!(
                Long64NoSwap(ll),
                unsafe { oracle::Long64NoSwap(ll) },
                "Long64NoSwap({ll:?})"
            );
        }
    }

    #[test]
    fn Q_strrchr_matches_oracle() {
        let cases = [
            "",
            "a",
            "hello",
            "abracadabra",
            "a/b/c/d",
            "trailing/",
            "^1color",
        ];
        let chars = [
            b'a' as c_int,
            b'/' as c_int,
            b'z' as c_int,
            0,
            b'r' as c_int,
            256 + b'a' as c_int,
        ];
        for s in cases {
            let buf = cbuf(s);
            for &c in &chars {
                let r = unsafe { Q_strrchr(buf.as_ptr(), c) };
                let o = unsafe { oracle::Q_strrchr(buf.as_ptr(), c) };
                assert_eq!(r, o, "Q_strrchr({s:?}, {c})");
            }
        }
    }

    #[test]
    fn Q_strncpyz_matches_oracle() {
        let srcs = ["", "hi", "exactfit", "longerthanthebuffer.....", "abc"];
        let sizes = [1i32, 2, 4, 8, 16];
        for s in srcs {
            let src = cbuf(s);
            for &size in &sizes {
                // pre-fill with a sentinel so strncpy's zero-padding is observable
                let mut d_rust = vec![0xAAu8 as c_char; 32];
                let mut d_c = vec![0xAAu8 as c_char; 32];
                unsafe {
                    Q_strncpyz(d_rust.as_mut_ptr(), src.as_ptr(), size);
                    oracle::Q_strncpyz(d_c.as_mut_ptr(), src.as_ptr(), size);
                }
                assert_eq!(d_rust, d_c, "Q_strncpyz({s:?}, size={size})");
            }
        }
    }

    /// Pairs spanning equal/case-diff/prefix/high-bit-byte combinations.
    fn cmp_pairs() -> [(&'static [u8], &'static [u8]); 16] {
        [
            (b"", b""),
            (b"a", b"a"),
            (b"a", b"A"),
            (b"abc", b"abd"),
            (b"abc", b"abc"),
            (b"Hello", b"hello"),
            (b"hello", b"HELLO"),
            (b"abc", b"ab"),
            (b"ab", b"abc"),
            (b"zoo", b"zoon"),
            (b"Test123", b"test123"),
            (b"\x80x", b"\x80x"),
            (b"a\x80", b"a\x7f"),
            (b"[bracket]", b"{brace}"),
            (b"A", b"a"),
            (b"MixedCase", b"mixedcase"),
        ]
    }

    #[test]
    fn Q_stricmpn_matches_oracle() {
        for (a, b) in cmp_pairs() {
            let (ba, bb) = (cbuf_b(a), cbuf_b(b));
            for n in [0i32, 1, 2, 3, 5, 99999] {
                let r = unsafe { Q_stricmpn(ba.as_ptr(), bb.as_ptr(), n) };
                let o = unsafe { oracle::Q_stricmpn(ba.as_ptr(), bb.as_ptr(), n) };
                assert_eq!(r, o, "Q_stricmpn({a:?},{b:?},{n})");
            }
        }
        // NULL handling
        let s = cbuf("x");
        let nul = core::ptr::null();
        unsafe {
            assert_eq!(Q_stricmpn(nul, nul, 5), oracle::Q_stricmpn(nul, nul, 5));
            assert_eq!(
                Q_stricmpn(nul, s.as_ptr(), 5),
                oracle::Q_stricmpn(nul, s.as_ptr(), 5)
            );
            assert_eq!(
                Q_stricmpn(s.as_ptr(), nul, 5),
                oracle::Q_stricmpn(s.as_ptr(), nul, 5)
            );
        }
    }

    #[test]
    fn Q_strncmp_matches_oracle() {
        for (a, b) in cmp_pairs() {
            let (ba, bb) = (cbuf_b(a), cbuf_b(b));
            for n in [0i32, 1, 2, 3, 5, 99999] {
                let r = unsafe { Q_strncmp(ba.as_ptr(), bb.as_ptr(), n) };
                let o = unsafe { oracle::Q_strncmp(ba.as_ptr(), bb.as_ptr(), n) };
                assert_eq!(r, o, "Q_strncmp({a:?},{b:?},{n})");
            }
        }
    }

    #[test]
    fn Q_stricmp_matches_oracle() {
        for (a, b) in cmp_pairs() {
            let (ba, bb) = (cbuf_b(a), cbuf_b(b));
            let r = unsafe { Q_stricmp(ba.as_ptr(), bb.as_ptr()) };
            let o = unsafe { oracle::Q_stricmp(ba.as_ptr(), bb.as_ptr()) };
            assert_eq!(r, o, "Q_stricmp({a:?},{b:?})");
        }
        let s = cbuf("x");
        let nul = core::ptr::null();
        unsafe {
            assert_eq!(
                Q_stricmp(nul, s.as_ptr()),
                oracle::Q_stricmp(nul, s.as_ptr())
            );
            assert_eq!(
                Q_stricmp(s.as_ptr(), nul),
                oracle::Q_stricmp(s.as_ptr(), nul)
            );
        }
    }

    #[test]
    fn Q_strlwr_matches_oracle() {
        // ASCII only: tolower/toupper on the negative args a signed char yields for
        // high-bit bytes is UB in libc; the port calls the same libc as the oracle,
        // so that region is identical by construction and need not be exercised.
        let cases: [&[u8]; 5] = [
            b"",
            b"Hello World",
            b"ALLCAPS",
            b"already lower",
            b"MiXeD123!@#",
        ];
        for c in cases {
            let mut a = cbuf_b(c);
            let mut b = cbuf_b(c);
            let ra = unsafe { Q_strlwr(a.as_mut_ptr()) };
            assert_eq!(ra, a.as_mut_ptr(), "Q_strlwr returns s1");
            unsafe { oracle::Q_strlwr(b.as_mut_ptr()) };
            assert_eq!(a, b, "Q_strlwr({c:?})");
        }
    }

    #[test]
    fn Q_strupr_matches_oracle() {
        let cases: [&[u8]; 5] = [
            b"",
            b"Hello World",
            b"ALLCAPS",
            b"already lower",
            b"MiXeD123!@#",
        ];
        for c in cases {
            let mut a = cbuf_b(c);
            let mut b = cbuf_b(c);
            let ra = unsafe { Q_strupr(a.as_mut_ptr()) };
            assert_eq!(ra, a.as_mut_ptr(), "Q_strupr returns s1");
            unsafe { oracle::Q_strupr(b.as_mut_ptr()) };
            assert_eq!(a, b, "Q_strupr({c:?})");
        }
    }

    #[test]
    fn Q_strcat_matches_oracle() {
        let initials = ["", "foo", "12345"];
        let srcs = ["", "bar", "appendmelong"];
        let sizes = [4i32, 8, 16, 32];
        for init in initials {
            for s in srcs {
                for &size in &sizes {
                    // skip combos the C would Com_Error-abort on (already overflowed)
                    if init.len() as c_int >= size {
                        continue;
                    }
                    let mut da = vec![0xAAu8 as c_char; 32];
                    for (i, b) in init.bytes().enumerate() {
                        da[i] = b as c_char;
                    }
                    da[init.len()] = 0;
                    let mut db = da.clone();
                    let src = cbuf(s);
                    unsafe {
                        Q_strcat(da.as_mut_ptr(), size, src.as_ptr());
                        oracle::Q_strcat(db.as_mut_ptr(), size, src.as_ptr());
                    }
                    assert_eq!(da, db, "Q_strcat(init={init:?}, src={s:?}, size={size})");
                }
            }
        }
    }

    /// Color/control-byte strings for the color-aware routines.
    fn color_cases() -> [&'static [u8]; 11] {
        [
            b"",
            b"hello",
            b"^1red",
            b"^1r^2g^3b",
            b"^^literal",
            b"^8notcolor",
            b"trailing^",
            b"^",
            b"a\x01b\x1fc\x7fd\x80e",
            b"^7white^0black^",
            b"plain text 123",
        ]
    }

    #[test]
    fn Q_PrintStrlen_matches_oracle() {
        for c in color_cases() {
            let buf = cbuf_b(c);
            let r = unsafe { Q_PrintStrlen(buf.as_ptr()) };
            let o = unsafe { oracle::Q_PrintStrlen(buf.as_ptr()) };
            assert_eq!(r, o, "Q_PrintStrlen({c:?})");
        }
        // NULL
        unsafe {
            assert_eq!(
                Q_PrintStrlen(core::ptr::null()),
                oracle::Q_PrintStrlen(core::ptr::null())
            );
        }
    }

    #[test]
    fn Q_CleanStr_matches_oracle() {
        for c in color_cases() {
            let mut a = cbuf_b(c);
            let mut b = cbuf_b(c);
            let ra = unsafe { Q_CleanStr(a.as_mut_ptr()) };
            assert_eq!(ra, a.as_mut_ptr(), "Q_CleanStr returns string");
            unsafe { oracle::Q_CleanStr(b.as_mut_ptr()) };
            assert_eq!(a, b, "Q_CleanStr({c:?})");
        }
    }

    /// Read a raw C string into bytes (excluding the NUL).
    unsafe fn read_cstr(p: *const c_char) -> Vec<u8> {
        let mut v = Vec::new();
        let mut q = p;
        while *q != 0 {
            v.push(*q as u8);
            q = q.add(1);
        }
        v
    }

    #[test]
    fn Com_sprintf_matches_oracle() {
        // %s%s concatenation (ASCII = the byte-exact domain for the
        // varargs->format_args! deviation). size kept above the rendered length so
        // the overflow path (Com_Printf, which needs the engine) is not hit.
        let fmt2 = cbuf("%s%s");
        let pairs = [
            ("", ""),
            ("foo", "bar"),
            ("path/file", ".ext"),
            ("a", "bcdef"),
        ];
        for (a, b) in pairs {
            let ca = cbuf(a);
            let cb = cbuf(b);
            let mut da = vec![0xAAu8 as c_char; 80];
            let mut db = vec![0xAAu8 as c_char; 80];
            unsafe {
                Com_sprintf(
                    da.as_mut_ptr(),
                    64,
                    format_args!("{}{}", Sz(ca.as_ptr()), Sz(cb.as_ptr())),
                );
                oracle::Com_sprintf(db.as_mut_ptr(), 64, fmt2.as_ptr(), ca.as_ptr(), cb.as_ptr());
            }
            assert_eq!(da, db, "Com_sprintf(\"%s%s\", {a:?}, {b:?})");
        }

        // int + string mix
        let name = cbuf("hero");
        let fmt = cbuf("hp=%i name=%s");
        let mut da = vec![0xAAu8 as c_char; 64];
        let mut db = vec![0xAAu8 as c_char; 64];
        unsafe {
            Com_sprintf(
                da.as_mut_ptr(),
                64,
                format_args!("hp={} name={}", 100i32, Sz(name.as_ptr())),
            );
            oracle::Com_sprintf(db.as_mut_ptr(), 64, fmt.as_ptr(), 100i32, name.as_ptr());
        }
        assert_eq!(da, db, "Com_sprintf int+str");
    }

    #[test]
    fn va_matches_oracle() {
        let name = cbuf("bob");
        let fmt = cbuf("count=%d name=%s");
        let r = unsafe { va(format_args!("count={} name={}", 7i32, Sz(name.as_ptr()))) };
        let rr = unsafe { read_cstr(r) };
        let o = unsafe { oracle::va(fmt.as_ptr(), 7i32, name.as_ptr()) };
        let oo = unsafe { read_cstr(o) };
        assert_eq!(rr, oo, "va count/name");

        let fmt2 = cbuf("just text 42");
        let r2 = unsafe { va(format_args!("just text {}", 42i32)) };
        let rr2 = unsafe { read_cstr(r2) };
        let o2 = unsafe { oracle::va(fmt2.as_ptr()) };
        let oo2 = unsafe { read_cstr(o2) };
        assert_eq!(rr2, oo2, "va plain");
    }

    #[test]
    fn GetIDForString_and_GetStringForID_match_oracle() {
        let names = ["alpha", "beta", "gamma", "delta"];
        let bufs: Vec<Vec<c_char>> = names.iter().map(|s| cbuf(s)).collect();
        let ids = [10i32, 20, 30, 40];
        let mut table: Vec<stringID_table_t> = Vec::new();
        for (buf, &id) in bufs.iter().zip(ids.iter()) {
            table.push(stringID_table_t {
                name: buf.as_ptr(),
                id,
            });
        }
        table.push(stringID_table_t {
            name: core::ptr::null(),
            id: 0,
        }); // terminator

        for q in ["alpha", "Beta", "gamma", "missing", "DELTA", ""] {
            let cq = cbuf(q);
            let r = unsafe { GetIDForString(table.as_ptr(), cq.as_ptr()) };
            let o = unsafe { oracle::GetIDForString(table.as_ptr(), cq.as_ptr()) };
            assert_eq!(r, o, "GetIDForString({q:?})");
        }

        for id in [10i32, 25, 30, 40, -1, 0] {
            let r = unsafe { GetStringForID(table.as_ptr(), id) };
            let o = unsafe { oracle::GetStringForID(table.as_ptr(), id) };
            assert_eq!(r, o, "GetStringForID({id})"); // both index the same table -> same ptr/null
        }
    }

    #[test]
    fn random_matches_oracle() {
        // random()/crandom() draw from the game LCG (bg_lib::rand / oracle jka_rand) —
        // the same process-global seed bg_lib's own RNG tests use; take the lock.
        let _guard = crate::codemp::game::bg_lib::rand_lock();
        for seed in [0u32, 1, 42, 69069, 0x8000_0000, 0xffff_ffff] {
            crate::codemp::game::bg_lib::srand(seed);
            unsafe { oracle::jka_srand(seed) };
            for i in 0..100_000 {
                let r = random();
                let o = unsafe { oracle::jka_q_random() };
                assert_eq!(r.to_bits(), o.to_bits(), "random() seed={seed:#x} iter={i}");
            }
        }
    }

    #[test]
    fn crandom_matches_oracle() {
        let _guard = crate::codemp::game::bg_lib::rand_lock();
        for seed in [0u32, 1, 42, 69069, 0x8000_0000, 0xffff_ffff] {
            crate::codemp::game::bg_lib::srand(seed);
            unsafe { oracle::jka_srand(seed) };
            for i in 0..100_000 {
                let r = crandom();
                let o = unsafe { oracle::jka_crandom() };
                assert_eq!(
                    r.to_bits(),
                    o.to_bits(),
                    "crandom() seed={seed:#x} iter={i}"
                );
            }
        }
    }

    #[test]
    fn Info_ValueForKey_matches_oracle() {
        let infos = [
            "",
            "\\name\\bob\\team\\red",
            "name\\bob\\team\\red",
            "\\key1\\val1\\key2\\val2\\key3\\val3",
            "\\empty\\\\next\\x",
            "\\onlykey",
        ];
        let keys = [
            "name", "team", "key2", "missing", "empty", "onlykey", "Name", "",
        ];
        for info in infos {
            let ci = cbuf(info);
            for k in keys {
                let ck = cbuf(k);
                let r = unsafe { Info_ValueForKey(ci.as_ptr(), ck.as_ptr()) };
                let o = unsafe { oracle::Info_ValueForKey(ci.as_ptr(), ck.as_ptr()) };
                assert_eq!(
                    unsafe { read_cstr(r) },
                    unsafe { read_cstr(o) },
                    "Info_ValueForKey({info:?},{k:?})"
                );
            }
        }
    }

    #[test]
    fn Info_NextPair_matches_oracle() {
        let infos = [
            "\\a\\1\\b\\2\\c\\3",
            "a\\1\\b\\2",
            "",
            "\\solo\\val",
            "\\k\\\\next\\v",
        ];
        for info in infos {
            let ci = cbuf(info);
            let mut head_r: *const c_char = ci.as_ptr();
            let mut head_o: *const c_char = ci.as_ptr();
            for _ in 0..6 {
                let mut key_r = vec![0xAAu8 as c_char; 64];
                let mut val_r = vec![0xAAu8 as c_char; 64];
                let mut key_o = vec![0xAAu8 as c_char; 64];
                let mut val_o = vec![0xAAu8 as c_char; 64];
                unsafe {
                    Info_NextPair(&mut head_r, key_r.as_mut_ptr(), val_r.as_mut_ptr());
                    oracle::Info_NextPair(&mut head_o, key_o.as_mut_ptr(), val_o.as_mut_ptr());
                }
                assert_eq!(key_r, key_o, "Info_NextPair key info={info:?}");
                assert_eq!(val_r, val_o, "Info_NextPair val info={info:?}");
                let off_r = unsafe { head_r.offset_from(ci.as_ptr()) };
                let off_o = unsafe { head_o.offset_from(ci.as_ptr()) };
                assert_eq!(off_r, off_o, "Info_NextPair head info={info:?}");
                if unsafe { *head_r } == 0 {
                    break;
                }
            }
        }
    }

    #[test]
    fn Info_Validate_matches_oracle() {
        let cases = [
            "",
            "\\name\\bob",
            "has\"quote",
            "has;semi",
            "both\";",
            "clean-123",
        ];
        for s in cases {
            let cs = cbuf(s);
            let r = unsafe { Info_Validate(cs.as_ptr()) };
            let o = unsafe { oracle::Info_Validate(cs.as_ptr()) };
            assert_eq!(r, o, "Info_Validate({s:?})");
        }
    }

    /// Two identical info buffers (one for Rust, one for C), `info` + NUL at the
    /// front of a `cap`-byte sentinel-filled buffer.
    fn info_pair(info: &str, cap: usize) -> (Vec<c_char>, Vec<c_char>) {
        let mut a = vec![0xAAu8 as c_char; cap];
        for (i, by) in info.bytes().enumerate() {
            a[i] = by as c_char;
        }
        a[info.len()] = 0;
        let b = a.clone();
        (a, b)
    }

    #[test]
    fn Info_RemoveKey_matches_oracle() {
        let infos = [
            "\\name\\bob\\team\\red",
            "\\name\\bob",
            "name\\bob\\team\\red",
            "\\a\\1\\b\\2\\c\\3",
            "",
        ];
        let keys = ["name", "team", "a", "c", "missing", "bad\\key"];
        for info in infos {
            for k in keys {
                let (mut a, mut b) = info_pair(info, 1100);
                let ck = cbuf(k);
                unsafe {
                    Info_RemoveKey(a.as_mut_ptr(), ck.as_ptr());
                    oracle::Info_RemoveKey(b.as_mut_ptr(), ck.as_ptr());
                }
                assert_eq!(a, b, "Info_RemoveKey(info={info:?}, key={k:?})");
            }
        }
    }

    #[test]
    fn Info_RemoveKey_Big_matches_oracle() {
        let infos = [
            "\\name\\bob\\team\\red",
            "\\name\\bob",
            "name\\bob\\team\\red",
            "\\a\\1\\b\\2\\c\\3",
            "",
        ];
        let keys = ["name", "team", "a", "c", "missing", "bad\\key"];
        for info in infos {
            for k in keys {
                let (mut a, mut b) = info_pair(info, 1100);
                let ck = cbuf(k);
                unsafe {
                    Info_RemoveKey_Big(a.as_mut_ptr(), ck.as_ptr());
                    oracle::Info_RemoveKey_Big(b.as_mut_ptr(), ck.as_ptr());
                }
                assert_eq!(a, b, "Info_RemoveKey_Big(info={info:?}, key={k:?})");
            }
        }
    }

    #[test]
    fn Info_SetValueForKey_matches_oracle() {
        let infos = ["", "\\name\\bob", "\\name\\bob\\team\\red", "\\a\\1\\b\\2"];
        let kvs = [
            ("name", "alice"),
            ("team", "blue"),
            ("new", "val"),
            ("name", ""),
            ("a", "99"),
            ("x", "y"),
        ];
        for info in infos {
            for (k, v) in kvs {
                let (mut a, mut b) = info_pair(info, 1100);
                let ck = cbuf(k);
                let cv = cbuf(v);
                unsafe {
                    Info_SetValueForKey(a.as_mut_ptr(), ck.as_ptr(), cv.as_ptr());
                    oracle::Info_SetValueForKey(b.as_mut_ptr(), ck.as_ptr(), cv.as_ptr());
                }
                assert_eq!(a, b, "Info_SetValueForKey(info={info:?}, k={k:?}, v={v:?})");
            }
        }
    }

    #[test]
    fn Info_SetValueForKey_Big_matches_oracle() {
        let infos = ["", "\\name\\bob", "\\name\\bob\\team\\red", "\\a\\1\\b\\2"];
        let kvs = [
            ("name", "alice"),
            ("team", "blue"),
            ("new", "val"),
            ("name", ""),
            ("a", "99"),
            ("x", "y"),
        ];
        for info in infos {
            for (k, v) in kvs {
                let (mut a, mut b) = info_pair(info, 1100);
                let ck = cbuf(k);
                let cv = cbuf(v);
                unsafe {
                    Info_SetValueForKey_Big(a.as_mut_ptr(), ck.as_ptr(), cv.as_ptr());
                    oracle::Info_SetValueForKey_Big(b.as_mut_ptr(), ck.as_ptr(), cv.as_ptr());
                }
                assert_eq!(
                    a, b,
                    "Info_SetValueForKey_Big(info={info:?}, k={k:?}, v={v:?})"
                );
            }
        }
    }

    #[test]
    fn COM_ParseExt_matches_oracle() {
        let _g = parse_lock();
        let inputs = [
            "hello world",
            "  token1\ttoken2  token3 ",
            "line1\nline2\nline3",
            "\"quoted string\" next",
            "a // comment\nb",
            "x /* block\ncomment */ y",
            "{ nested } [ stuff ]",
            "trailing\n\n\n",
            "",
            "\"unterminated",
            "key1\nkey2 val2\n",
        ];
        for allow in [QTRUE, QFALSE] {
            for input in inputs {
                let buf = cbuf(input);
                let name = cbuf("test");
                unsafe {
                    COM_BeginParseSession(name.as_ptr());
                    oracle::COM_BeginParseSession(name.as_ptr());
                }
                let mut cur_r: *const c_char = buf.as_ptr();
                let mut cur_o: *const c_char = buf.as_ptr();
                for _ in 0..20 {
                    let tr = unsafe { COM_ParseExt(&mut cur_r, allow) };
                    let to = unsafe { oracle::COM_ParseExt(&mut cur_o, allow) };
                    assert_eq!(
                        unsafe { read_cstr(tr) },
                        unsafe { read_cstr(to) },
                        "token input={input:?} allow={allow}"
                    );
                    let off_r = if cur_r.is_null() {
                        -1
                    } else {
                        unsafe { cur_r.offset_from(buf.as_ptr()) }
                    };
                    let off_o = if cur_o.is_null() {
                        -1
                    } else {
                        unsafe { cur_o.offset_from(buf.as_ptr()) }
                    };
                    assert_eq!(off_r, off_o, "cursor input={input:?} allow={allow}");
                    assert_eq!(
                        COM_GetCurrentParseLine(),
                        unsafe { oracle::COM_GetCurrentParseLine() },
                        "lines input={input:?} allow={allow}"
                    );
                    if cur_r.is_null() || unsafe { read_cstr(tr) }.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn COM_ParseInfos_matches_oracle() {
        use crate::codemp::game::q_shared_h::MAX_INFO_STRING;
        let _g = parse_lock();
        let inputs = [
            // normal: two blocks of key/value pairs
            "{ name jedi level 5 } { name sith level 9 }",
            // single block
            "{ a 1 b 2 c 3 }",
            // empty input
            "",
            // missing opening brace -> "Missing { in info file"
            "name jedi",
            // value-less trailing key -> "<NULL>" substitution
            "{ lonelykey }",
            // unexpected EOF inside a block
            "{ key val",
            // empty block
            "{ }",
        ];
        let max: c_int = 4;
        for input in inputs {
            // Each side gets its own `max`-row infos array.
            let mut infos_r = vec![[0 as c_char; MAX_INFO_STRING]; max as usize];
            let mut infos_o = vec![[0 as c_char; MAX_INFO_STRING]; max as usize];
            let buf = cbuf(input);
            let name = cbuf("test");
            unsafe {
                COM_BeginParseSession(name.as_ptr());
                oracle::COM_BeginParseSession(name.as_ptr());
            }
            let cr = unsafe { COM_ParseInfos(buf.as_ptr(), max, infos_r.as_mut_ptr()) };
            let co = unsafe { oracle::COM_ParseInfos(buf.as_ptr(), max, infos_o.as_mut_ptr()) };
            assert_eq!(cr, co, "count input={input:?}");
            for i in 0..(cr as usize) {
                assert_eq!(
                    unsafe { read_cstr(infos_r[i].as_ptr()) },
                    unsafe { read_cstr(infos_o[i].as_ptr()) },
                    "infos[{i}] input={input:?}"
                );
            }
        }
    }

    #[test]
    fn COM_ParseString_matches_oracle() {
        let _g = parse_lock();
        let buf = cbuf("hello world \"quoted tok\" last");
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        for _ in 0..6 {
            // ParseString's EOF branch is the dead bug, so it never calls Com_Printf
            let mut sr: *const c_char = core::ptr::null();
            let mut so: *const c_char = core::ptr::null();
            let rr = unsafe { COM_ParseString(&mut cur_r, &mut sr) };
            let ro = unsafe { oracle::COM_ParseString(&mut cur_o, &mut so) };
            assert_eq!(rr, ro, "COM_ParseString ret");
            assert_eq!(
                unsafe { read_cstr(sr) },
                unsafe { read_cstr(so) },
                "COM_ParseString tok"
            );
            assert_eq!(
                cur_r.is_null(),
                cur_o.is_null(),
                "COM_ParseString cursor null"
            );
            if cur_r.is_null() {
                break;
            }
        }
    }

    #[test]
    fn COM_ParseInt_matches_oracle() {
        let _g = parse_lock();
        let buf = cbuf("42 -7 100 0 999 65535 -32768"); // 7 tokens
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        for _ in 0..7 {
            let (mut ir, mut io) = (0i32, 0i32);
            let rr = unsafe { COM_ParseInt(&mut cur_r, &mut ir) };
            let ro = unsafe { oracle::COM_ParseInt(&mut cur_o, &mut io) };
            assert_eq!(rr, ro, "COM_ParseInt ret");
            assert_eq!(ir, io, "COM_ParseInt val");
            assert_eq!(
                unsafe { cur_r.offset_from(buf.as_ptr()) },
                unsafe { cur_o.offset_from(buf.as_ptr()) },
                "COM_ParseInt cursor"
            );
        }
    }

    #[test]
    fn COM_ParseFloat_matches_oracle() {
        let _g = parse_lock();
        let buf = cbuf("1.5 -2.25 3.14159 0.0 100 -1e3 2.5e-2"); // 7 tokens
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        for _ in 0..7 {
            let (mut fr, mut fo) = (0f32, 0f32);
            let rr = unsafe { COM_ParseFloat(&mut cur_r, &mut fr) };
            let ro = unsafe { oracle::COM_ParseFloat(&mut cur_o, &mut fo) };
            assert_eq!(rr, ro, "COM_ParseFloat ret");
            assert_eq!(fr.to_bits(), fo.to_bits(), "COM_ParseFloat val");
        }
    }

    #[test]
    fn COM_ParseVec4_matches_oracle() {
        let _g = parse_lock();
        let buf = cbuf("1.0 2.0 3.0 4.0 0.5 -0.5 1e3 -1e-3"); // two vec4s
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        for _ in 0..2 {
            let mut vr = [0f32; 4];
            let mut vo = [0f32; 4];
            let rr = unsafe { COM_ParseVec4(&mut cur_r, &mut vr) };
            let ro = unsafe { oracle::COM_ParseVec4(&mut cur_o, &mut vo) };
            assert_eq!(rr, ro, "COM_ParseVec4 ret");
            for k in 0..4 {
                assert_eq!(vr[k].to_bits(), vo[k].to_bits(), "COM_ParseVec4 [{k}]");
            }
        }
    }

    #[test]
    fn COM_MatchToken_matches_oracle() {
        let _g = parse_lock();
        // matching case only (a mismatch Com_Errors -> aborts/panics)
        let name = cbuf("t");
        let open = cbuf("(");
        let buf = cbuf("( next )");
        unsafe {
            COM_BeginParseSession(name.as_ptr());
            oracle::COM_BeginParseSession(name.as_ptr());
        }
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        unsafe {
            COM_MatchToken(&mut cur_r, open.as_ptr());
            oracle::COM_MatchToken(&mut cur_o, open.as_ptr());
        }
        assert_eq!(
            unsafe { cur_r.offset_from(buf.as_ptr()) },
            unsafe { cur_o.offset_from(buf.as_ptr()) },
            "COM_MatchToken cursor"
        );
        assert_eq!(COM_GetCurrentParseLine(), unsafe {
            oracle::COM_GetCurrentParseLine()
        });
    }

    #[test]
    fn SkipBracedSection_matches_oracle() {
        let _g = parse_lock();
        let name = cbuf("t");
        let cases = [
            "{ a { b } c } after",
            "{ } trailing",
            "{nested{deeper{x}}} end",
        ];
        for input in cases {
            let buf = cbuf(input);
            unsafe {
                COM_BeginParseSession(name.as_ptr());
                oracle::COM_BeginParseSession(name.as_ptr());
            }
            let mut cur_r = buf.as_ptr();
            let mut cur_o = buf.as_ptr();
            unsafe {
                SkipBracedSection(&mut cur_r);
                oracle::SkipBracedSection(&mut cur_o);
            }
            let off_r = if cur_r.is_null() {
                -1
            } else {
                unsafe { cur_r.offset_from(buf.as_ptr()) }
            };
            let off_o = if cur_o.is_null() {
                -1
            } else {
                unsafe { cur_o.offset_from(buf.as_ptr()) }
            };
            assert_eq!(off_r, off_o, "SkipBracedSection cursor input={input:?}");
            assert_eq!(
                COM_GetCurrentParseLine(),
                unsafe { oracle::COM_GetCurrentParseLine() },
                "SkipBracedSection lines input={input:?}"
            );
        }
    }

    #[test]
    fn SkipRestOfLine_matches_oracle() {
        let _g = parse_lock();
        let name = cbuf("t");
        let cases = [
            "rest of line\nnext line",
            "no newline here",
            "\nimmediate",
            "a\nb\nc",
        ];
        for input in cases {
            let buf = cbuf(input);
            unsafe {
                COM_BeginParseSession(name.as_ptr());
                oracle::COM_BeginParseSession(name.as_ptr());
            }
            let mut cur_r = buf.as_ptr();
            let mut cur_o = buf.as_ptr();
            unsafe {
                SkipRestOfLine(&mut cur_r);
                oracle::SkipRestOfLine(&mut cur_o);
            }
            assert_eq!(
                unsafe { cur_r.offset_from(buf.as_ptr()) },
                unsafe { cur_o.offset_from(buf.as_ptr()) },
                "SkipRestOfLine cursor input={input:?}"
            );
            assert_eq!(
                COM_GetCurrentParseLine(),
                unsafe { oracle::COM_GetCurrentParseLine() },
                "SkipRestOfLine lines input={input:?}"
            );
        }
    }

    #[test]
    fn Parse1DMatrix_matches_oracle() {
        let _g = parse_lock();
        let name = cbuf("t");
        let buf = cbuf("( 1.0 2.5 -3.0 )");
        unsafe {
            COM_BeginParseSession(name.as_ptr());
            oracle::COM_BeginParseSession(name.as_ptr());
        }
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        let mut mr = [0f32; 3];
        let mut mo = [0f32; 3];
        unsafe {
            Parse1DMatrix(&mut cur_r, 3, mr.as_mut_ptr());
            oracle::Parse1DMatrix(&mut cur_o, 3, mo.as_mut_ptr());
        }
        for k in 0..3 {
            assert_eq!(mr[k].to_bits(), mo[k].to_bits(), "Parse1DMatrix[{k}]");
        }
        assert_eq!(unsafe { cur_r.offset_from(buf.as_ptr()) }, unsafe {
            cur_o.offset_from(buf.as_ptr())
        });
    }

    #[test]
    fn Parse2DMatrix_matches_oracle() {
        let _g = parse_lock();
        let name = cbuf("t");
        let buf = cbuf("( ( 1 2 ) ( 3 4 ) ( 5 6 ) )");
        unsafe {
            COM_BeginParseSession(name.as_ptr());
            oracle::COM_BeginParseSession(name.as_ptr());
        }
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        let mut mr = [0f32; 6];
        let mut mo = [0f32; 6];
        unsafe {
            Parse2DMatrix(&mut cur_r, 3, 2, mr.as_mut_ptr());
            oracle::Parse2DMatrix(&mut cur_o, 3, 2, mo.as_mut_ptr());
        }
        for k in 0..6 {
            assert_eq!(mr[k].to_bits(), mo[k].to_bits(), "Parse2DMatrix[{k}]");
        }
        assert_eq!(unsafe { cur_r.offset_from(buf.as_ptr()) }, unsafe {
            cur_o.offset_from(buf.as_ptr())
        });
    }

    #[test]
    fn Parse3DMatrix_matches_oracle() {
        let _g = parse_lock();
        let name = cbuf("t");
        let buf = cbuf("( ( ( 1 2 ) ( 3 4 ) ) ( ( 5 6 ) ( 7 8 ) ) )");
        unsafe {
            COM_BeginParseSession(name.as_ptr());
            oracle::COM_BeginParseSession(name.as_ptr());
        }
        let mut cur_r = buf.as_ptr();
        let mut cur_o = buf.as_ptr();
        let mut mr = [0f32; 8];
        let mut mo = [0f32; 8];
        unsafe {
            Parse3DMatrix(&mut cur_r, 2, 2, 2, mr.as_mut_ptr());
            oracle::Parse3DMatrix(&mut cur_o, 2, 2, 2, mo.as_mut_ptr());
        }
        for k in 0..8 {
            assert_eq!(mr[k].to_bits(), mo[k].to_bits(), "Parse3DMatrix[{k}]");
        }
        assert_eq!(unsafe { cur_r.offset_from(buf.as_ptr()) }, unsafe {
            cur_o.offset_from(buf.as_ptr())
        });
    }

    #[test]
    fn COM_Compress_matches_oracle() {
        let inputs = [
            "  hello   world  ",
            "a // line comment\nb",
            "x /* block */ y",
            "\"quoted   spaces\"  out",
            "line1\n\nline2\r\nline3",
            "\ttabs\tand spaces ",
            "no_compress",
            "",
            "/* unterminated",
            "trailing/* */",
        ];
        for input in inputs {
            let mut a = cbuf(input);
            let mut b = cbuf(input);
            let ra = unsafe { COM_Compress(a.as_mut_ptr()) };
            let rb = unsafe { oracle::COM_Compress(b.as_mut_ptr()) };
            assert_eq!(ra, rb, "COM_Compress return input={input:?}");
            assert_eq!(a, b, "COM_Compress buffer input={input:?}");
        }
    }

    #[test]
    fn COM_SkipPath_matches_oracle() {
        let cases = ["", "file", "/file", "a/b/c", "path/", "/", "x/y/z/name.ext"];
        for s in cases {
            let mut buf = cbuf(s);
            let r = unsafe { COM_SkipPath(buf.as_mut_ptr()) };
            let o = unsafe { oracle::COM_SkipPath(buf.as_mut_ptr()) };
            assert_eq!(r, o, "COM_SkipPath({s:?})");
        }
    }

    #[test]
    fn COM_StripExtension_matches_oracle() {
        let cases = [
            "",
            "file",
            "file.ext",
            "a.b.c",
            ".hidden",
            "no_dot",
            "path/to/file.tga",
        ];
        for s in cases {
            let in_buf = cbuf(s);
            let mut a = vec![0xAAu8 as c_char; 64];
            let mut b = vec![0xAAu8 as c_char; 64];
            unsafe {
                COM_StripExtension(in_buf.as_ptr(), a.as_mut_ptr());
                oracle::COM_StripExtension(in_buf.as_ptr(), b.as_mut_ptr());
            }
            assert_eq!(a, b, "COM_StripExtension({s:?})");
        }
    }

    #[test]
    fn COM_DefaultExtension_matches_oracle() {
        // non-empty paths only (the C reads path[-1] when path is empty: UB)
        let cases = [
            ("file", ".tga"),
            ("file.bmp", ".tga"),
            ("path/to/file", ".md3"),
            ("path/to/file.ext", ".md3"),
            ("noext", ".cfg"),
            ("dir.x/name", ".wav"),
        ];
        for (p, ext) in cases {
            let cext = cbuf(ext);
            let mut a = vec![0xAAu8 as c_char; 128];
            let mut b = vec![0xAAu8 as c_char; 128];
            for (i, by) in p.bytes().enumerate() {
                a[i] = by as c_char;
                b[i] = by as c_char;
            }
            a[p.len()] = 0;
            b[p.len()] = 0;
            unsafe {
                COM_DefaultExtension(a.as_mut_ptr(), 128, cext.as_ptr());
                oracle::COM_DefaultExtension(b.as_mut_ptr(), 128, cext.as_ptr());
            }
            assert_eq!(a, b, "COM_DefaultExtension({p:?},{ext:?})");
        }
    }

    #[test]
    fn FloatSwap_matches_oracle() {
        let samples = [
            0.0f32,
            -0.0,
            1.0,
            -1.0,
            3.14159,
            1.0e30,
            1.0e-30,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            1234.5678,
        ];
        for &f in &samples {
            let rust = unsafe { FloatSwap(&f) };
            let c = unsafe { oracle::FloatSwap(&f) };
            assert_eq!(rust.to_bits(), c.to_bits(), "FloatSwap({f})");
            let rust_n = unsafe { FloatNoSwap(&f) };
            let c_n = unsafe { oracle::FloatNoSwap(&f) };
            assert_eq!(rust_n.to_bits(), c_n.to_bits(), "FloatNoSwap({f})");
        }
    }
}
