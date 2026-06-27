//! `bg_lib.c` — standard C library replacement routines used by code compiled for
//! the virtual machine.
//!
//! Faithful port of `refs/raven-jediacademy/codemp/game/bg_lib.c`. The C file is **not one
//! uniform thing** — its preprocessor guards split it three ways, and we mirror that
//! split 1:1 (see `roadmap/CONVENTIONS.md`, "Conditional compilation"):
//!
//! - **Group A — always compiled (ungated here).** `qsort` (+ `med3`/`swapfunc`),
//!   `srand`/`rand`, `atof`/`_atof`, `memmove`. These compile in *both* the native
//!   and VM builds, and on native they **override libc** — so they are the game's
//!   real behavior on every target. Ported faithfully, oracle-verified.
//! - **Group B — `#[cfg(feature = "vm")]`** (the Rust mirror of C `#if defined(Q3_VM)`).
//!   The no-libc shims a bytecode/WASM build needs: `strlen`/`strcat`/`strcpy`/`strcmp`/
//!   `strchr`/`strstr`, `tolower`/`toupper`/`abs`/`fabs`/`tan`, `atoi`/`_atoi`, and the
//!   printf helpers `AddInt`/`AddFloat`/`AddString`. On the default native build these
//!   resolve to std/libc instead, so they are gated off. `vsprintf` and `sscanf`, which the C
//!   builds by walking a `va_list` as an `int*` array, are both **now ported** via an explicit
//!   typed-slice deviation (the int*-walk is broken on a 64-bit ABI anyway) — see each fn and
//!   `DEVIATIONS.md`. (`sscanf` additionally has a native libc binding for its variadic native
//!   callers; `vsprintf` needs none — `Com_sprintf`/`va` are ported via `fmt::Arguments`.)
//!   Group B parity tests are gated on `feature = "vm"` too, so run them with
//!   **`cargo test --features "oracle vm"`** (plain `--features oracle` skips them).
//! - **Group C — `#if 0` dead code (skipped).** Never compiled in any JKA config:
//!   `floor`, `memset`, `memcpy`, `strncpy`, `sqrt`, `sin`, `cos`, `acos`, `atan2`
//!   and the `sintable`/`acostable`. Listed here for the record; not ported.
//!
//! Why keep Group B at all when OpenJK deleted the whole VM build: the user wants to
//! retain and expand JKA's sandboxed bytecode-module capability (realistic modern
//! target: WebAssembly). See memory `project-vm-capability`. The `vm` cargo feature
//! *is* the `Q3_VM` define.
//!
//! Function names and the original comments are carried over so the Rust can be
//! diffed against the C. Group A is parity-tested against the extracted C oracle
//! (`oracle/bg_lib_oracle.c`; libc-colliding names are `jka_`-prefixed there):
//! `cargo test --features oracle`.

#![allow(non_snake_case)] // AddInt/AddFloat/AddString (Group B) keep their C names.
#![allow(non_camel_case_types)] // `cmp_t` keeps its C name for 1:1 traceability.

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::sync::atomic::{AtomicI32, Ordering};

// =============================================================================
// Group A — always compiled (ungated); overrides libc on native, so this is the
// game's real behavior on BOTH the native and VM targets.
// =============================================================================

/// `static int randSeed` — the seed for the game's own `rand()`.
///
/// A plain `static int` in C; an [`AtomicI32`] here so the module needs no `unsafe`
/// for the mutable global (cf. `COM_LINES` in `q_shared`). The game is
/// single-threaded, so `Relaxed` is the correct ordering — there is no cross-thread
/// happens-before to establish, only interior mutability.
static RAND_SEED: AtomicI32 = AtomicI32::new(0);

/// `void srand( unsigned seed )`.
pub fn srand(seed: c_uint) {
    // randSeed = seed;  (the unsigned->int assignment just reinterprets the bits)
    RAND_SEED.store(seed as i32, Ordering::Relaxed);
}

/// `int rand( void )` — the game RNG: a 15-bit Quake3 LCG.
///
/// Compiled ungated in `bg_lib.c`, so it **overrides libc `rand()`** even on native
/// builds — this, not the platform RNG, is the game's `rand()` everywhere. The
/// `& 0x7fff` makes `RAND_MAX` 0x7fff (15 bits), which is why callers such as
/// `random()`/`crandom()` mask the draw with `& 0x7fff`.
pub fn rand() -> c_int {
    // randSeed = (69069 * randSeed + 1);
    // Signed-int overflow is UB in C but wraps in practice (and the oracle is built
    // -fwrapv to make that explicit); `wrapping_*` reproduces it bit-for-bit.
    let next = RAND_SEED
        .load(Ordering::Relaxed)
        .wrapping_mul(69069)
        .wrapping_add(1);
    RAND_SEED.store(next, Ordering::Relaxed);
    next & 0x7fff
}

// -----------------------------------------------------------------------------
// qsort -- Bentley & McIlroy, "Engineering a Sort Function". Ungated in bg_lib.c,
// so this is the `qsort` the game uses on every target (it overrides libc's on
// native). The original carries a copy-width optimization (the swaptype/SWAPINIT/
// swap/vecswap/swapcode apparatus) which we collapse to a single byte-wise swap;
// see `swapfunc` for why that is output-identical. Everything else -- pivot
// selection, partition, tie-break order, the insertion-sort fallbacks, and the
// recurse-small / iterate-large tail -- is reproduced exactly. Parity-checked
// against the C (with its full machinery intact) in `qsort_matches_oracle`.
// -----------------------------------------------------------------------------

/// `typedef int cmp_t(const void *, const void *);`
///
/// In C the typedef names the *function* type and `qsort` takes a `cmp_t *`; a Rust
/// `extern "C" fn` value already **is** that function pointer, so this alias stands in
/// for C's `cmp_t *`. (bk001127: the C typedef sits under `#if !defined( Q3_VM )`,
/// "needed for DLL's"; the Rust fn-pointer type is identical on every target, so it is
/// unconditional here.) Modeled as a *safe* fn pointer -- invoking the comparator just
/// hands control to one of the game's own C functions; the unsafety is in the raw
/// element pointers passed to it, which is why [`qsort`] itself is `unsafe`.
pub type cmp_t = extern "C" fn(*const c_void, *const c_void) -> c_int;

/// Exchange `n` bytes between `a` and `b`, one byte at a time.
///
/// DEVIATION (output-invariant). The C original threads a `swaptype` through
/// `SWAPINIT`/`swap`/`vecswap`/`swapcode`/`swapfunc` to pick a copy *width*: a single
/// `long` (`swaptype 0`), a run of `long`s (`swaptype 1`), or bytes (`swaptype 2`).
/// `SWAPINIT` only chooses a `long` width when the base is `long`-aligned **and**
/// `es % sizeof(long) == 0`, so a wide copy always moves exactly the bytes a byte-wise
/// copy would -- the width is a speed trick that never changes which bytes land where.
/// Collapsing it to one byte loop is therefore **bit-identical** in result (verified in
/// `qsort_matches_oracle` against the unmodified C, which keeps the whole apparatus).
/// `n` is always `>= 1` at every call site (`swap` passes `es >= 1`; `vecswap` is
/// guarded `n > 0`), matching C `swapcode`'s `do { } while`, which runs at least once.
unsafe fn swapfunc(a: *mut u8, b: *mut u8, n: usize) {
    let mut i = 0;
    while i < n {
        let t = *a.add(i);
        *a.add(i) = *b.add(i);
        *b.add(i) = t;
        i += 1;
    }
}

/// ```c
/// static char *med3(char* a, char* b, char* c, cmp_t* cmp)
/// ```
/// Median-of-three: returns whichever of `a`/`b`/`c` addresses the median element. The C
/// is one nested `?:`; expanded to `if`/`else` here, branch-for-branch identical. It
/// takes no unsafe action itself (only calls `cmp` and selects a pointer) but is `unsafe`
/// because the pointers it forwards to `cmp` must be valid -- the same contract as
/// [`qsort`], its only caller.
unsafe fn med3(a: *mut u8, b: *mut u8, c: *mut u8, cmp: cmp_t) -> *mut u8 {
    if cmp(a as *const c_void, b as *const c_void) < 0 {
        if cmp(b as *const c_void, c as *const c_void) < 0 {
            b
        } else if cmp(a as *const c_void, c as *const c_void) < 0 {
            c
        } else {
            a
        }
    } else if cmp(b as *const c_void, c as *const c_void) > 0 {
        b
    } else if cmp(a as *const c_void, c as *const c_void) < 0 {
        a
    } else {
        c
    }
}

/// Byte distance `p - q` as a `usize`. Every call below subtracts a lower address from a
/// higher one (each `>= 0` by the partition invariants noted at the call), so this never
/// wraps. Stands in for the C `char *` subtraction, whose signed `ptrdiff_t` the original
/// stores in an `int`.
#[inline]
fn ptr_diff(p: *const u8, q: *const u8) -> usize {
    (p as usize) - (q as usize)
}

/// ```c
/// void qsort( void* a, size_t n, size_t es, cmp_t* cmp)
/// ```
/// Sort `n` elements of `es` bytes each, in place, by `cmp`. Faithful port: the C `goto
/// loop` tail becomes `continue 'qloop`; the side-effecting `while (.. && (r = cmp(..))
/// <op> 0)` conditions become inner `loop`s that break on the negated test; the `min`
/// macro becomes [`core::cmp::min`]; `swap`/`vecswap` become byte-wise [`swapfunc`] calls
/// (`vecswap`'s `n > 0` guard preserved). All computed pointers stay within
/// `[a, a + n*es]` for `n >= 1`.
pub unsafe fn qsort(a: *mut c_void, mut n: usize, es: usize, cmp: cmp_t) {
    let mut a = a as *mut u8;
    'qloop: loop {
        // SWAPINIT(a, es): selects the (collapsed-away) swap width -- nothing to do.
        let mut swap_cnt = 0;
        if n < 7 {
            let mut pm = a.add(es);
            while pm < a.add(n * es) {
                let mut pl = pm;
                while pl > a && cmp(pl.sub(es) as *const c_void, pl as *const c_void) > 0 {
                    swapfunc(pl, pl.sub(es), es);
                    pl = pl.sub(es);
                }
                pm = pm.add(es);
            }
            return;
        }
        let mut pm = a.add((n / 2) * es);
        if n > 7 {
            let mut pl = a;
            let mut pn = a.add((n - 1) * es);
            if n > 40 {
                let d = (n / 8) * es;
                pl = med3(pl, pl.add(d), pl.add(2 * d), cmp);
                pm = med3(pm.sub(d), pm, pm.add(d), cmp);
                pn = med3(pn.sub(2 * d), pn.sub(d), pn, cmp);
            }
            pm = med3(pl, pm, pn, cmp);
        }
        swapfunc(a, pm, es);
        let mut pa = a.add(es);
        let mut pb = a.add(es);
        let mut pc = a.add((n - 1) * es);
        let mut pd = pc;
        loop {
            // while (pb <= pc && (r = cmp(pb, a)) <= 0) { if (r==0) {..} pb += es; }
            loop {
                if pb > pc {
                    break;
                }
                let r = cmp(pb as *const c_void, a as *const c_void);
                if r > 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = 1;
                    swapfunc(pa, pb, es);
                    pa = pa.add(es);
                }
                pb = pb.add(es);
            }
            // while (pb <= pc && (r = cmp(pc, a)) >= 0) { if (r==0) {..} pc -= es; }
            loop {
                if pb > pc {
                    break;
                }
                let r = cmp(pc as *const c_void, a as *const c_void);
                if r < 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = 1;
                    swapfunc(pc, pd, es);
                    pd = pd.sub(es);
                }
                pc = pc.sub(es);
            }
            if pb > pc {
                break;
            }
            swapfunc(pb, pc, es);
            swap_cnt = 1;
            pb = pb.add(es);
            pc = pc.sub(es);
        }
        if swap_cnt == 0 {
            // Switch to insertion sort
            let mut pm = a.add(es);
            while pm < a.add(n * es) {
                let mut pl = pm;
                while pl > a && cmp(pl.sub(es) as *const c_void, pl as *const c_void) > 0 {
                    swapfunc(pl, pl.sub(es), es);
                    pl = pl.sub(es);
                }
                pm = pm.add(es);
            }
            return;
        }
        let pn = a.add(n * es);
        let mut r = core::cmp::min(ptr_diff(pa, a), ptr_diff(pb, pa));
        if r > 0 {
            swapfunc(a, pb.sub(r), r);
        }
        // pn - pd >= es here (pd <= a + (n-1)*es, pn == a + n*es), so this does not wrap.
        r = core::cmp::min(ptr_diff(pd, pc), ptr_diff(pn, pd) - es);
        if r > 0 {
            swapfunc(pb, pn.sub(r), r);
        }
        r = ptr_diff(pb, pa);
        if r > es {
            qsort(a as *mut c_void, r / es, es, cmp);
        }
        r = ptr_diff(pd, pc);
        if r > es {
            // Iterate rather than recurse to save stack space
            a = pn.sub(r);
            n = r / es;
            continue 'qloop;
        }
        return;
    }
}

// -----------------------------------------------------------------------------
// atof / _atof -- JKA's own string->float parser. Ungated in bg_lib.c, so this
// (NOT libc strtod/atof) is what the game uses to read floats on every target. It
// is deliberately minimal: "// not handling 10e10 notation..." -- no exponent
// form, no inf/nan, no locale. The float/double widths are load-bearing: the
// integer part accumulates in `float` (FLT_EVAL_METHOD 0 -> done in float), while
// the fractional part accumulates `value += c * fraction` with `fraction` a
// `double`, so each fractional add rounds through double then truncates back to
// float. Reproduced step-for-step; parity-checked bit-exact against the C oracle
// (`-ffp-contract=off` on both sides, and Rust never auto-fuses, so no FMA skew).
// -----------------------------------------------------------------------------

/// ```c
/// double atof( const char *string )
/// ```
/// Parse a float from a NUL-terminated string. `unsafe` because `string` is a raw
/// C pointer walked to the terminator. Whitespace is `*string <= ' '`, read through
/// the platform's `char` signedness (the oracle is built with the same compiler on
/// the same target, so both agree on whether high-bit bytes count as whitespace).
pub unsafe fn atof(mut string: *const c_char) -> f64 {
    let sign: f32;
    let mut value: f32;
    let mut c: c_int;

    // skip whitespace
    while (*string as c_int) <= (' ' as c_int) {
        if *string == 0 {
            return 0.0;
        }
        string = string.add(1);
    }

    // check sign
    let ch = *string as c_int;
    if ch == ('+' as c_int) {
        string = string.add(1);
        sign = 1.0;
    } else if ch == ('-' as c_int) {
        string = string.add(1);
        sign = -1.0;
    } else {
        sign = 1.0;
    }

    // read digits
    value = 0.0;
    c = *string as c_int; // c = string[0];
    if c != ('.' as c_int) {
        loop {
            c = *string as c_int; // c = *string++;
            string = string.add(1);
            if c < ('0' as c_int) || c > ('9' as c_int) {
                break;
            }
            c -= '0' as c_int;
            value = value * 10.0 + c as f32;
        }
    } else {
        string = string.add(1);
    }

    // check for decimal point
    if c == ('.' as c_int) {
        let mut fraction: f64 = 0.1;
        loop {
            c = *string as c_int; // c = *string++;
            string = string.add(1);
            if c < ('0' as c_int) || c > ('9' as c_int) {
                break;
            }
            c -= '0' as c_int;
            // value += c * fraction; -- value promotes to double for the add, then
            // the store truncates back to float (faithful to C's mixed widths).
            value = (value as f64 + c as f64 * fraction) as f32;
            fraction *= 0.1;
        }
    }

    // not handling 10e10 notation...

    (value * sign) as f64
}

/// ```c
/// double _atof( const char **stringPtr )
/// ```
/// Like [`atof`], but advances `*stringPtr` past the consumed text (used by
/// `sscanf`). Two faithful quirks vs [`atof`]: `c` is pre-seeded to `'0'`
/// ("bk001211 - uninitialized use possible") and the integer part is gated on
/// `string[0] != '.'` rather than peeking `c` -- so a leading-dot input like ".5"
/// leaves `c == '0'`, the `c == '.'` fraction branch is skipped, and the function
/// returns 0 with `*stringPtr` left pointing AT the dot (where [`atof`] would parse
/// it as 0.5). Carried over verbatim.
pub unsafe fn _atof(string_ptr: *mut *const c_char) -> f64 {
    let mut string: *const c_char = *string_ptr;
    let sign: f32;
    let mut value: f32;
    let mut c: c_int = '0' as c_int; // bk001211 - uninitialized use possible

    // skip whitespace
    while (*string as c_int) <= (' ' as c_int) {
        if *string == 0 {
            *string_ptr = string;
            return 0.0;
        }
        string = string.add(1);
    }

    // check sign
    let ch = *string as c_int;
    if ch == ('+' as c_int) {
        string = string.add(1);
        sign = 1.0;
    } else if ch == ('-' as c_int) {
        string = string.add(1);
        sign = -1.0;
    } else {
        sign = 1.0;
    }

    // read digits
    value = 0.0;
    if (*string as c_int) != ('.' as c_int) {
        loop {
            c = *string as c_int; // c = *string++;
            string = string.add(1);
            if c < ('0' as c_int) || c > ('9' as c_int) {
                break;
            }
            c -= '0' as c_int;
            value = value * 10.0 + c as f32;
        }
    }

    // check for decimal point
    if c == ('.' as c_int) {
        let mut fraction: f64 = 0.1;
        loop {
            c = *string as c_int; // c = *string++;
            string = string.add(1);
            if c < ('0' as c_int) || c > ('9' as c_int) {
                break;
            }
            c -= '0' as c_int;
            value = (value as f64 + c as f64 * fraction) as f32;
            fraction *= 0.1;
        }
    }

    // not handling 10e10 notation...
    *string_ptr = string;

    (value * sign) as f64
}

/// ```c
/// void *memmove( void *dest, const void *src, size_t count )
/// ```
/// Copy `count` bytes from `src` to `dest`, handling overlap. Ungated in bg_lib.c
/// (under `//#ifndef _MSC_VER`), so it overrides libc `memmove` on native too. The
/// overlap rule is decided purely by the pointer ordering: `dest > src` copies
/// back-to-front, otherwise front-to-back -- byte by byte. Behaviorally identical to
/// [`core::ptr::copy`], but ported faithfully because the C is always compiled.
///
/// The loop counter is a plain `int i` in C: for `count == 0` the descending branch's
/// `i = count - 1` underflows the unsigned `count` and narrows to `-1`, so the
/// `i >= 0` test fails immediately (no copy) -- reproduced via the wrapping subtract.
pub unsafe fn memmove(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void {
    let d = dest as *mut u8;
    let s = src as *const u8;
    if (dest as usize) > (src as usize) {
        // for ( i = count-1 ; i >= 0 ; i-- )
        let mut i: c_int = (count as c_int).wrapping_sub(1);
        while i >= 0 {
            *d.add(i as usize) = *s.add(i as usize);
            i -= 1;
        }
    } else {
        // for ( i = 0 ; i < count ; i++ )
        let mut i: c_int = 0;
        while (i as usize) < count {
            *d.add(i as usize) = *s.add(i as usize);
            i += 1;
        }
    }
    dest
}

// =============================================================================
// Group B — `#[cfg(feature = "vm")]` (the Rust mirror of C `#if defined(Q3_VM)`).
// These are the no-libc shims a bytecode/WASM build needs. On the default native
// build they resolve to std/libc instead, so they are gated OFF — only a `--features
// vm` build compiles them. Their oracle bodies (`oracle/bg_lib_oracle.c`) are extracted
// UNGATED and `jka_`-prefixed, so `cargo test --features "oracle vm"` can call both the
// Rust shim and the authentic C side-by-side. See module header + roadmap CONVENTIONS
// "Conditional compilation".
//
// `vsprintf` and `sscanf` -- both of which the C drives by walking a `va_list` reinterpreted
// as an `int*` array -- are PORTED below via an explicit typed-slice deviation (that int*-walk
// cannot be expressed on stable Rust and is *broken on any 64-bit ABI*; the typed slice is the
// 64-bit-correct stand-in, one slot per C `arg++`). The concrete-signature printf helpers
// `AddInt`/`AddFloat`/`AddString` that `vsprintf` drives are ported + oracle-verified below.
// `sscanf` additionally has a native variadic libc binding (its native callers are genuinely
// variadic); `vsprintf` needs no native binding -- the C copy is `Q3_VM`-only and its callers
// `Com_sprintf`/`va` are ported via the varargs->`fmt::Arguments` deviation. See `DEVIATIONS.md`.
// =============================================================================

// -----------------------------------------------------------------------------
// String routines. `bk001211 - gcc errors on compiling strcpy: parse error before
// `__extension__`` -- so the whole block sits under `#if defined( Q3_VM )`.
// -----------------------------------------------------------------------------

/// ```c
/// size_t strlen( const char *string )
/// ```
/// Length up to (not including) the NUL. `unsafe`: `string` is a raw pointer walked to
/// the terminator. The C `s - string` pointer difference is reproduced as a byte count.
#[cfg(feature = "vm")]
pub unsafe fn strlen(string: *const c_char) -> usize {
    let mut s = string;
    while *s != 0 {
        s = s.add(1);
    }
    // return s - string;
    (s as usize) - (string as usize)
}

/// ```c
/// char *strcat( char *strDestination, const char *strSource )
/// ```
/// Append `strSource` onto the end of `strDestination` (which must have room), NUL-
/// terminate, and return `strDestination`. No bounds check -- faithful to the C.
#[cfg(feature = "vm")]
pub unsafe fn strcat(str_destination: *mut c_char, mut str_source: *const c_char) -> *mut c_char {
    let mut s = str_destination;
    while *s != 0 {
        s = s.add(1);
    }
    while *str_source != 0 {
        *s = *str_source;
        s = s.add(1);
        str_source = str_source.add(1);
    }
    *s = 0;
    str_destination
}

/// ```c
/// char *strcpy( char *strDestination, const char *strSource )
/// ```
/// Copy `strSource` (incl. its NUL) into `strDestination` and return it. No bounds
/// check -- faithful to the C.
#[cfg(feature = "vm")]
pub unsafe fn strcpy(str_destination: *mut c_char, mut str_source: *const c_char) -> *mut c_char {
    let mut s = str_destination;
    while *str_source != 0 {
        *s = *str_source;
        s = s.add(1);
        str_source = str_source.add(1);
    }
    *s = 0;
    str_destination
}

/// ```c
/// int strcmp( const char *string1, const char *string2 )
/// ```
/// Lexicographic compare, returning `*string1 - *string2` at the first difference (or at
/// the shared terminator). The chars promote to `int` through the platform's `char`
/// signedness (`c_char` here, matching the oracle on the same target), so a `*p as c_int`
/// subtraction reproduces C's promotion exactly.
#[cfg(feature = "vm")]
pub unsafe fn strcmp(mut string1: *const c_char, mut string2: *const c_char) -> c_int {
    while *string1 == *string2 && *string1 != 0 && *string2 != 0 {
        string1 = string1.add(1);
        string2 = string2.add(1);
    }
    (*string1 as c_int) - (*string2 as c_int)
}

/// ```c
/// char *strchr( const char *string, int c )
/// ```
/// First occurrence of `c` in `string`, or NULL. `*string == c` compares the char
/// (promoted to `int`) against the `int` argument, so `*string as c_int == c` is faithful.
/// Does NOT match the trailing NUL (the loop stops before it) -- this differs from libc
/// `strchr`, which finds `c == '\0'`; carried verbatim.
#[cfg(feature = "vm")]
pub unsafe fn strchr(mut string: *const c_char, c: c_int) -> *mut c_char {
    while *string != 0 {
        if *string as c_int == c {
            return string as *mut c_char;
        }
        string = string.add(1);
    }
    core::ptr::null_mut() // return (char *)0;
}

/// ```c
/// char *strstr( const char *string, const char *strCharSet )
/// ```
/// First occurrence of the substring `strCharSet` in `string`, or NULL. Naive O(n*m)
/// scan, byte by byte; an empty `strCharSet` matches at `string` (the inner `for` exits
/// immediately on `strCharSet[0] == 0`). Faithful.
#[cfg(feature = "vm")]
pub unsafe fn strstr(mut string: *const c_char, str_char_set: *const c_char) -> *mut c_char {
    while *string != 0 {
        let mut i: isize = 0;
        while *str_char_set.offset(i) != 0 {
            if *string.offset(i) != *str_char_set.offset(i) {
                break;
            }
            i += 1;
        }
        if *str_char_set.offset(i) == 0 {
            return string as *mut c_char;
        }
        string = string.add(1);
    }
    core::ptr::null_mut() // return (char *)0;
}

// -----------------------------------------------------------------------------
// ctype. `bk001127 - undid undo` -- these sit under their own `#if defined ( Q3_VM )`
// in bg_lib.c (the comment about a Mac/`__linux__` guard is dead). ASCII-only case
// folding, no locale.
// -----------------------------------------------------------------------------

/// ```c
/// int tolower( int c )
/// ```
/// ASCII lowercase: `A`..`Z` shift by `'a' - 'A'`, everything else passes through. Safe
/// -- it touches no memory.
#[cfg(feature = "vm")]
pub fn tolower(mut c: c_int) -> c_int {
    if c >= ('A' as c_int) && c <= ('Z' as c_int) {
        c += ('a' as c_int) - ('A' as c_int);
    }
    c
}

/// ```c
/// int toupper( int c )
/// ```
/// ASCII uppercase: `a`..`z` shift by `'A' - 'a'`, everything else passes through. Safe.
#[cfg(feature = "vm")]
pub fn toupper(mut c: c_int) -> c_int {
    if c >= ('a' as c_int) && c <= ('z' as c_int) {
        c += ('A' as c_int) - ('a' as c_int);
    }
    c
}

// -----------------------------------------------------------------------------
// abs / fabs / tan -- the math shims. abs/fabs sit just after the printf family in
// bg_lib.c; tan is its own `#ifdef Q3_VM` block (`bk001127 - guarded this tan
// replacement` to dodge `ld: undefined versioned symbol name tan@@GLIBC_2.0`).
// -----------------------------------------------------------------------------

/// ```c
/// int abs( int n )
/// ```
/// `n < 0 ? -n : n`. Carried faithful UB: `abs(INT_MIN)` negates `INT_MIN`, which
/// overflows -- the oracle TU is built `-fwrapv` so it wraps back to `INT_MIN`, and
/// [`i32::wrapping_neg`] reproduces that exactly (a plain `-n` would panic in a debug
/// build). Safe.
#[cfg(feature = "vm")]
pub fn abs(n: c_int) -> c_int {
    if n < 0 {
        n.wrapping_neg()
    } else {
        n
    }
}

/// ```c
/// double fabs( double x )
/// ```
/// `x < 0 ? -x : x`. NOT libc `fabs`: because the test is `x < 0` (strict) rather than a
/// sign-bit clear, `fabs(-0.0)` returns `-0.0` (`-0.0 < 0` is false), and any NaN passes
/// through unchanged. Carried verbatim. Safe.
#[cfg(feature = "vm")]
pub fn fabs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// In a real VM/WASM build `sin`/`cos` are provided by the runtime -- they are NOT in
// bg_lib (its `sin`/`cos` are `#if 0` dead code, Group C). Declared `extern "C"` so a
// native `--features vm` build (and the parity test) resolves them to libm, matching the
// oracle's `jka_tan`, which also calls libm `sin`/`cos`. With `-ffp-contract=off` on the
// C side and no auto-fusion in Rust, the single divide is bit-identical.
#[cfg(feature = "vm")]
extern "C" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
}

/// ```c
/// double tan( double x )
/// ```
/// `sin(x) / cos(x)`. `bk001127 - guarded this tan replacement` to dodge a glibc
/// versioned-symbol clash (`tan@@GLIBC_2.0`). The inner `unsafe` is only the calls to the
/// `extern "C"` math fns above (sound); the arithmetic is the same single IEEE divide as C.
#[cfg(feature = "vm")]
pub fn tan(x: f64) -> f64 {
    unsafe { sin(x) / cos(x) }
}

// -----------------------------------------------------------------------------
// int parse. The integer twins of atof/_atof (bg_lib.c ~915-1008): same whitespace +
// sign handling, but accumulate in `int`. "not handling 10e10 notation..." -- no hex,
// no overflow guard. `value*10+c` and `value*sign` overflow is UB in C, defined to wrap
// by the oracle's `-fwrapv`; `wrapping_mul`/`wrapping_add` reproduce it.
// -----------------------------------------------------------------------------

/// ```c
/// int atoi( const char *string )
/// ```
/// Parse a base-10 int from a NUL-terminated string. `unsafe`: `string` is a raw pointer
/// walked to a non-digit or the terminator.
#[cfg(feature = "vm")]
pub unsafe fn atoi(mut string: *const c_char) -> c_int {
    let sign: c_int;
    let mut value: c_int;
    let mut c: c_int;

    // skip whitespace
    while (*string as c_int) <= (' ' as c_int) {
        if *string == 0 {
            return 0;
        }
        string = string.add(1);
    }

    // check sign
    let ch = *string as c_int;
    if ch == ('+' as c_int) {
        string = string.add(1);
        sign = 1;
    } else if ch == ('-' as c_int) {
        string = string.add(1);
        sign = -1;
    } else {
        sign = 1;
    }

    // read digits
    value = 0;
    loop {
        c = *string as c_int; // c = *string++;
        string = string.add(1);
        if c < ('0' as c_int) || c > ('9' as c_int) {
            break;
        }
        c -= '0' as c_int;
        value = value.wrapping_mul(10).wrapping_add(c);
    }

    // not handling 10e10 notation...

    value.wrapping_mul(sign)
}

/// ```c
/// int _atoi( const char **stringPtr )
/// ```
/// Like [`atoi`], but advances `*stringPtr` past the consumed text (used by `sscanf`).
/// FAITHFUL ASYMMETRY vs [`_atof`]: on an empty / all-whitespace input `_atoi` returns 0
/// **without** writing `*stringPtr` (the early `return 0` skips the write), whereas
/// `_atof` sets it to the terminator first. Carried verbatim.
#[cfg(feature = "vm")]
pub unsafe fn _atoi(string_ptr: *mut *const c_char) -> c_int {
    let mut string: *const c_char = *string_ptr;
    let sign: c_int;
    let mut value: c_int;
    let mut c: c_int;

    // skip whitespace
    while (*string as c_int) <= (' ' as c_int) {
        if *string == 0 {
            // NOTE: unlike _atof, _atoi does NOT write *string_ptr back here.
            return 0;
        }
        string = string.add(1);
    }

    // check sign
    let ch = *string as c_int;
    if ch == ('+' as c_int) {
        string = string.add(1);
        sign = 1;
    } else if ch == ('-' as c_int) {
        string = string.add(1);
        sign = -1;
    } else {
        sign = 1;
    }

    // read digits
    value = 0;
    loop {
        c = *string as c_int; // c = *string++;
        string = string.add(1);
        if c < ('0' as c_int) || c > ('9' as c_int) {
            break;
        }
        c -= '0' as c_int;
        value = value.wrapping_mul(10).wrapping_add(c);
    }

    // not handling 10e10 notation...
    *string_ptr = string;

    value.wrapping_mul(sign)
}

// -----------------------------------------------------------------------------
// printf helpers (bg_lib.c ~1037-1173). These three append a formatted field into a
// `char**` cursor and advance it. They are the concrete-signature core that
// `vsprintf` (ported below) dispatches to. The format-flag bitmask is `#define`d in
// bg_lib.c (~1023-1031); only the two `vsprintf` actually consults -- LADJUST/ZEROPAD --
// are defined here (the rest, ALT/HEXPREFIX/LONGINT/..., are unused by the ported code).
// -----------------------------------------------------------------------------

/// `#define LADJUST 0x00000004` -- left adjustment (the `-` flag).
#[cfg(feature = "vm")]
const LADJUST: c_int = 0x0000_0004;
/// `#define ZEROPAD 0x00000080` -- zero (as opposed to blank) pad.
#[cfg(feature = "vm")]
const ZEROPAD: c_int = 0x0000_0080;

/// ```c
/// void AddInt( char **buf_p, int val, int width, int flags )
/// ```
/// Append `val` as base-10 with `width`/`flags` padding, advancing `*buf_p`. No bounds
/// check. Faithful carries: `val = -val` overflows for `INT_MIN` (`wrapping_neg`, matching
/// the oracle's `-fwrapv`), after which the digit loop emits the C-defined negative
/// `val % 10` results; the `'0' + val % 10` store narrows int->char via `as c_char`
/// (mod 256, as C does); and `while (digits--)` / `while (width--)` are reproduced as
/// post-decrement loops (the latter loops forever on a negative `width`, exactly as the C
/// does -- callers/tests keep `width` sane). Rust `%`/`/` truncate toward zero like C.
#[cfg(feature = "vm")]
pub unsafe fn AddInt(buf_p: *mut *mut c_char, mut val: c_int, mut width: c_int, flags: c_int) {
    let mut text = [0 as c_char; 32];
    let mut digits: c_int;
    let signed_val: c_int;
    let mut buf: *mut c_char;

    digits = 0;
    signed_val = val;
    if val < 0 {
        val = val.wrapping_neg(); // val = -val;  (INT_MIN wraps)
    }
    loop {
        // text[digits++] = '0' + val % 10;
        text[digits as usize] = (('0' as c_int).wrapping_add(val % 10)) as c_char;
        digits += 1;
        val /= 10;
        if val == 0 {
            break;
        }
    }

    if signed_val < 0 {
        text[digits as usize] = '-' as i32 as c_char;
        digits += 1;
    }

    buf = *buf_p;

    if (flags & LADJUST) == 0 {
        while digits < width {
            *buf = if (flags & ZEROPAD) != 0 {
                '0' as i32 as c_char
            } else {
                ' ' as i32 as c_char
            };
            buf = buf.add(1);
            width -= 1;
        }
    }

    // while ( digits-- ) { *buf++ = text[digits]; width--; }
    loop {
        let old = digits;
        digits -= 1;
        if old == 0 {
            break;
        }
        *buf = text[digits as usize];
        buf = buf.add(1);
        width -= 1;
    }

    if (flags & LADJUST) != 0 {
        // while ( width-- ) { *buf++ = ... ; }
        loop {
            let old = width;
            width = width.wrapping_sub(1);
            if old == 0 {
                break;
            }
            *buf = if (flags & ZEROPAD) != 0 {
                '0' as i32 as c_char
            } else {
                ' ' as i32 as c_char
            };
            buf = buf.add(1);
        }
    }

    *buf_p = buf;
}

/// ```c
/// void AddFloat( char **buf_p, float fval, int width, int prec )
/// ```
/// Append `fval` with `width` integer-field padding (spaces only -- no flags) and `prec`
/// fraction digits (default 6 when `prec < 0`), advancing `*buf_p`. No rounding of the
/// last fraction digit and no `width` accounting for the fraction -- faithful to the C.
///
/// The float/double widths are load-bearing (cf. `atof`): `fval *= 10.0` multiplies a
/// **double** literal, so the step computes in `f64` and stores back to `f32`; `fval -=
/// (int)fval` truncates through `int` then subtracts in `float`. Reproduced step-for-step
/// (`-ffp-contract=off` on the C side, no auto-fusion in Rust -> bit-identical). Rust's
/// saturating float->int `as` agrees with C's truncation for in-range magnitudes (callers
/// pass sane floats); the fraction's `(int)fval` is always in `0..10`.
#[cfg(feature = "vm")]
pub unsafe fn AddFloat(buf_p: *mut *mut c_char, mut fval: f32, mut width: c_int, mut prec: c_int) {
    let mut text = [0 as c_char; 32];
    let mut digits: c_int;
    let signed_val: f32;
    let mut buf: *mut c_char;
    let mut val: c_int;

    // get the sign
    signed_val = fval;
    if fval < 0.0 {
        fval = -fval;
    }

    // write the float number
    digits = 0;
    val = fval as c_int; // val = (int)fval;
    loop {
        text[digits as usize] = (('0' as c_int).wrapping_add(val % 10)) as c_char;
        digits += 1;
        val /= 10;
        if val == 0 {
            break;
        }
    }

    if signed_val < 0.0 {
        text[digits as usize] = '-' as i32 as c_char;
        digits += 1;
    }

    buf = *buf_p;

    while digits < width {
        *buf = ' ' as i32 as c_char;
        buf = buf.add(1);
        width -= 1;
    }

    // while ( digits-- ) { *buf++ = text[digits]; }
    loop {
        let old = digits;
        digits -= 1;
        if old == 0 {
            break;
        }
        *buf = text[digits as usize];
        buf = buf.add(1);
    }

    *buf_p = buf;

    if prec < 0 {
        prec = 6;
    }
    // write the fraction
    digits = 0;
    while digits < prec {
        fval -= (fval as c_int) as f32; // fval -= (int) fval;
        fval = (fval as f64 * 10.0) as f32; // fval *= 10.0;  (double literal)
        val = fval as c_int; // val = (int) fval;
        text[digits as usize] = (('0' as c_int).wrapping_add(val % 10)) as c_char;
        digits += 1;
    }

    if digits > 0 {
        buf = *buf_p;
        *buf = '.' as i32 as c_char;
        buf = buf.add(1);
        // for (prec = 0; prec < digits; prec++) *buf++ = text[prec];
        prec = 0;
        while prec < digits {
            *buf = text[prec as usize];
            buf = buf.add(1);
            prec += 1;
        }
        *buf_p = buf;
    }
}

/// ```c
/// void AddString( char **buf_p, char *string, int width, int prec )
/// ```
/// Append `string` (NULL prints `"(null)"`), truncated to `prec` chars when `prec >= 0`,
/// right-padded with spaces to `width`, advancing `*buf_p`. Length comes from the Group B
/// [`strlen`] (so the `vm` build's own, not libc). The two trailing loops differ: `while
/// (size--)` is a bare post-decrement (copy), while `while (width-- > 0)` only pads while
/// `width` is positive (so a negative `width`, i.e. a field already wider than requested,
/// pads nothing) -- both carried verbatim.
#[cfg(feature = "vm")]
pub unsafe fn AddString(
    buf_p: *mut *mut c_char,
    mut string: *const c_char,
    mut width: c_int,
    mut prec: c_int,
) {
    let mut size: c_int;
    let mut buf: *mut c_char;

    buf = *buf_p;

    if string.is_null() {
        string = b"(null)\0".as_ptr() as *const c_char;
        prec = -1;
    }

    if prec >= 0 {
        size = 0;
        while size < prec {
            if *string.offset(size as isize) == 0 {
                break;
            }
            size += 1;
        }
    } else {
        size = strlen(string) as c_int;
    }

    width -= size;

    // while ( size-- ) { *buf++ = *string++; }
    loop {
        let old = size;
        size -= 1;
        if old == 0 {
            break;
        }
        *buf = *string;
        buf = buf.add(1);
        string = string.add(1);
    }

    // while ( width-- > 0 ) { *buf++ = ' '; }
    loop {
        let old = width;
        width -= 1;
        if old <= 0 {
            break;
        }
        *buf = ' ' as i32 as c_char;
        buf = buf.add(1);
    }

    *buf_p = buf;
}

/// ```c
/// #define to_digit(c)  ((c) - '0')
/// #define is_digit(c)  ((unsigned)to_digit(c) <= 9)
/// ```
/// The `vsprintf` format-scanner's digit test (bg_lib.c ~1033). Cast-to-`unsigned` folds the
/// `< '0'` and `> '9'` cases into one `<= 9` compare; carried verbatim. Safe (no memory).
#[cfg(feature = "vm")]
#[inline]
fn is_digit(c: c_char) -> bool {
    ((c as c_int - '0' as c_int) as c_uint) <= 9
}

/// One argument for [`vsprintf`]. The C walks a `va_list` reinterpreted as `int *arg` and
/// reads `*arg` (int), `*(double *)arg` (a `float` promoted to `double` through the
/// varargs), or `(char *)*arg` (a string) per conversion. That stack-walk is both
/// unrepresentable on stable Rust *and incorrect on any 64-bit ABI* (varargs are not packed
/// after `fmt`), so — exactly as [`sscanf`] does for its output pointers — the arguments are
/// taken as an explicit typed slice instead, one `VsArg` standing in for each C `arg++`.
#[cfg(feature = "vm")]
pub enum VsArg {
    /// `%c`, `%d`, `%i`, and the `default` conversion — the C reads `int` off `arg`.
    Int(c_int),
    /// `%f` — the C reads `*(double *)arg` (the `float` arg promoted to `double` by the
    /// varargs call), which `AddFloat` then narrows back to `float` (`as f32` here).
    Double(f64),
    /// `%s` — the C reads `(char *)*arg`.
    Str(*const c_char),
}

#[cfg(feature = "vm")]
#[inline]
fn vs_int(a: &VsArg) -> c_int {
    match a {
        VsArg::Int(v) => *v,
        _ => panic!("vsprintf: %c/%d/%i (or unknown conversion) needs a VsArg::Int slot"),
    }
}
#[cfg(feature = "vm")]
#[inline]
fn vs_double(a: &VsArg) -> f64 {
    match a {
        VsArg::Double(v) => *v,
        _ => panic!("vsprintf: %f needs a VsArg::Double slot"),
    }
}
#[cfg(feature = "vm")]
#[inline]
fn vs_str(a: &VsArg) -> *const c_char {
    match a {
        VsArg::Str(v) => *v,
        _ => panic!("vsprintf: %s needs a VsArg::Str slot"),
    }
}

/// ```c
/// int vsprintf( char *buffer, const char *fmt, va_list argptr )
/// ```
/// Faithful-**behavior** port of bg_lib.c's `vsprintf` (~1183) — the VM build's minimal
/// `*printf` engine. It supports `%c %d %i %f %s %%` plus left-adjust (`-`), zero-pad (`0`),
/// a numeric field `width`, and a `.prec` precision, parsing-and-ignoring anything else
/// (`*` and `$` are unsupported, as the original comment notes). The formatting itself is
/// the already-oracle-verified [`AddInt`]/[`AddFloat`]/[`AddString`]; this is the
/// format-string state machine that drives them.
///
/// MECHANISM DEVIATION (notated; behavior-identical) — see [`VsArg`]: the C does
/// `arg = (int *)argptr; … *arg … arg++`, walking the `va_list` as a packed `int` array.
/// That is unportable on stable Rust *and broken on a 64-bit ABI*, so the arguments arrive
/// as an explicit `args` slice (slot `n` ⇒ the C's `n`-th `arg++`). The C's `%f` slot
/// accounting (`arg += 2` non-LCC / `arg += 1` under `__LCC__`) is a 32-bit-VM memory-layout
/// artifact with no meaning for a typed slice, so every consumed conversion advances `argi`
/// by one — matching which conversions do (and `%%` does **not**) consume an argument.
///
/// `goto` deviation: the C `rflag:`/`reswitch:` labels become an inner loop — `goto rflag`
/// reads the next format byte then `continue`s; `goto reswitch` re-dispatches the current
/// byte via `continue`; a completed conversion `break`s back to the outer literal-copy loop.
/// The width/precision accumulators use `wrapping_*` to match the oracle TU's `-fwrapv`.
///
/// Stays `#[cfg(feature = "vm")]`: it mirrors C's `#if defined( Q3_VM )` guard and calls the
/// `vm`-gated `AddInt`/`AddFloat`/`AddString`. The native build has **no** in-module
/// `vsprintf` (the C copy is `Q3_VM`-only) and needs none — its only callers, `Com_sprintf`
/// and `va`, are ported via the varargs→`fmt::Arguments` deviation (see `DEVIATIONS.md`),
/// so unlike `sscanf` there is no native libc binding to add.
///
/// `unsafe`: `buffer`/`fmt` are raw pointers walked to their bounds with no overflow check —
/// the caller supplies a buffer large enough for the formatted result and exactly one
/// correctly-typed `args` slot per consuming conversion, faithful to the C's stack contract.
#[cfg(feature = "vm")]
pub unsafe fn vsprintf(buffer: *mut c_char, mut fmt: *const c_char, args: &[VsArg]) -> c_int {
    let mut buf_p = buffer;
    let mut argi: usize = 0; // replaces the C `arg++` int*-walk
    let mut ch: c_char;

    loop {
        // run through the format string until we hit a '%' or '\0'
        loop {
            ch = *fmt;
            if ch == 0 || ch == b'%' as c_char {
                break;
            }
            *buf_p = ch; // *buf_p++ = ch;
            buf_p = buf_p.add(1);
            fmt = fmt.add(1);
        }
        if ch == 0 {
            break; // goto done
        }

        // skip over the '%'
        fmt = fmt.add(1);

        // reset formatting state
        let mut flags: c_int = 0;
        let mut width: c_int = 0;
        let mut prec: c_int = -1;
        // (C declares `char sign` here and zeroes it each pass but never reads it — omitted.)

        // rflag: ch = *fmt++;
        ch = *fmt;
        fmt = fmt.add(1);

        // reswitch:
        loop {
            match ch as u8 {
                b'-' => {
                    flags |= LADJUST;
                    // goto rflag
                    ch = *fmt;
                    fmt = fmt.add(1);
                    continue;
                }
                b'.' => {
                    // n = 0; while( is_digit( ch = *fmt++ ) ) n = 10*n + (ch-'0');
                    let mut n: c_int = 0;
                    loop {
                        ch = *fmt;
                        fmt = fmt.add(1);
                        if !is_digit(ch) {
                            break;
                        }
                        n = n.wrapping_mul(10).wrapping_add(ch as c_int - '0' as c_int);
                    }
                    prec = if n < 0 { -1 } else { n };
                    continue; // goto reswitch (ch holds the non-digit terminator)
                }
                b'0' => {
                    flags |= ZEROPAD;
                    // goto rflag
                    ch = *fmt;
                    fmt = fmt.add(1);
                    continue;
                }
                b'1'..=b'9' => {
                    // n = 0; do { n = 10*n + (ch-'0'); ch = *fmt++; } while( is_digit(ch) );
                    let mut n: c_int = 0;
                    loop {
                        n = n.wrapping_mul(10).wrapping_add(ch as c_int - '0' as c_int);
                        ch = *fmt;
                        fmt = fmt.add(1);
                        if !is_digit(ch) {
                            break;
                        }
                    }
                    width = n;
                    continue; // goto reswitch
                }
                b'c' => {
                    // *buf_p++ = (char)*arg; arg++;
                    *buf_p = vs_int(&args[argi]) as c_char;
                    buf_p = buf_p.add(1);
                    argi += 1;
                    break;
                }
                b'd' | b'i' => {
                    // AddInt( &buf_p, *arg, width, flags ); arg++;
                    AddInt(&mut buf_p, vs_int(&args[argi]), width, flags);
                    argi += 1;
                    break;
                }
                b'f' => {
                    // AddFloat( &buf_p, *(double *)arg, width, prec ); arg += 2 (1 on LCC);
                    AddFloat(&mut buf_p, vs_double(&args[argi]) as f32, width, prec);
                    argi += 1;
                    break;
                }
                b's' => {
                    // AddString( &buf_p, (char *)*arg, width, prec ); arg++;
                    AddString(&mut buf_p, vs_str(&args[argi]), width, prec);
                    argi += 1;
                    break;
                }
                b'%' => {
                    // *buf_p++ = ch;  (consumes NO argument)
                    *buf_p = ch;
                    buf_p = buf_p.add(1);
                    break;
                }
                _ => {
                    // default: *buf_p++ = (char)*arg; arg++;
                    *buf_p = vs_int(&args[argi]) as c_char;
                    buf_p = buf_p.add(1);
                    argi += 1;
                    break;
                }
            }
        }
    }

    // done:
    *buf_p = 0;
    (buf_p as usize - buffer as usize) as c_int // return buf_p - buffer;
}

/// ```c
/// int sscanf( const char *buffer, const char *fmt, ... )
/// ```
/// Faithful-**behavior** port of bg_lib.c's `sscanf` (~1285) — the VM build's minimal
/// scanf shim. It handles only `%i`/`%d`/`%u` (via [`_atoi`]) and `%f` (via [`_atof`]),
/// silently ignores any other conversion, and — a genuine original quirk — never
/// increments its match count, so it **always returns 0**.
///
/// MECHANISM DEVIATION (notated; behavior-identical): the C walks its varargs with the
/// 32-bit trick `arg = (int **)&fmt + 1; … arg++`, reading each output pointer off the
/// stack just past `fmt`. That is both unrepresentable in stable Rust and *incorrect on
/// any 64-bit ABI* (varargs arrive in registers, not packed after `fmt`), so the literal
/// mechanism is unportable rather than merely inconvenient. We take the output pointers
/// as an explicit `args` slice instead: `args[n]` stands in for the C's `n`-th `arg++`,
/// and since each conversion writes an `int` or `float` through that pointer, the slot is
/// a type-erased `*mut c_void` reinterpreted per the spec (`%i/%d/%u` → `*mut c_int`,
/// `%f` → `*mut f32`). `buffer` is advanced by `_atoi`/`_atof` just as the C's `&buffer` is.
///
/// `_atof` returns `f64` here (its `f32` computation widened, oracle-verified bit-exact to
/// the C `_atof`); the `as f32` narrowing recovers the exact float the C stores, losslessly.
///
/// Carried-from-C contract (hence `unsafe`): no arg-count checking — a `%`-spec consumes
/// the next `args` slot whether or not it is recognized (matching C's unconditional
/// `arg++`), so the caller must supply exactly one slot per `%`-conversion, just as the C
/// relies on exactly one stack arg per conversion.
///
/// Stays `#[cfg(feature = "vm")]`: it mirrors C's `#if defined( Q3_VM )` guard and calls
/// the `vm`-gated [`_atoi`]. The native C build links full libc `sscanf` instead; a
/// libc-equivalent for the eventual native game build is tracked in `DEVIATIONS.md`.
#[cfg(feature = "vm")]
pub unsafe fn sscanf(buffer: *const c_char, fmt: *const c_char, args: &[*mut c_void]) -> c_int {
    let mut buffer = buffer; // advanced by _atoi/_atof, mirroring C's `&buffer`
    let mut fmt = fmt;
    let mut argi: usize = 0; // replaces the C `arg++` pointer walk
    let count: c_int = 0; // C never increments `count` -> always 0 (carried verbatim)

    while *fmt != 0 {
        if *fmt != b'%' as c_char {
            fmt = fmt.add(1);
            continue;
        }

        let cmd = *fmt.add(1);
        fmt = fmt.add(2);

        match cmd as u8 {
            b'i' | b'd' | b'u' => {
                *(args[argi] as *mut c_int) = _atoi(&mut buffer);
            }
            b'f' => {
                *(args[argi] as *mut f32) = _atof(&mut buffer) as f32;
            }
            _ => {}
        }
        argi += 1; // C does `arg++` unconditionally after the switch
    }

    count
}

// Native-build `sscanf`: the complement of the `vm`-gated shim above. In the original C,
// `bg_lib.c`'s hand-rolled `sscanf` is wrapped in `#if defined( Q3_VM )`; the ordinary
// native `.so`/`.dll` build has no in-module copy and simply links the platform's libc
// `sscanf`. We mirror that exactly with an FFI binding (same approach as the `sin`/`cos`
// declaration above): no reimplementation, just resolve the real C symbol at link time.
#[cfg(not(feature = "vm"))]
extern "C" {
    /// ```c
    /// int sscanf( const char *buffer, const char *fmt, ... )
    /// ```
    /// A true C **variadic** function. Declaring/calling a variadic `extern "C"` is stable
    /// Rust (only *defining* one is nightly), so callers write the faithful C call shape
    /// directly — e.g. `BG_ParseField`'s `F_VECTOR`:
    /// `sscanf(value, c"%f %f %f".as_ptr(), &mut v[0], &mut v[1], &mut v[2])`. Unlike the
    /// VM shim, this is full libc: it honors every conversion and returns the real match
    /// count.
    ///
    /// `unsafe` for the usual variadic-FFI reasons: the format string must match the number
    /// and type of the output pointers, each of which must point to writable storage of the
    /// matching type — exactly the contract the C call sites already satisfy.
    pub fn sscanf(buffer: *const c_char, fmt: *const c_char, ...) -> c_int;
}

/// Test-only: a single mutex serializing every test that drives the global RNG
/// seed. `rand()`/`srand()` here and `random()`/`crandom()` in [`q_shared`](super::q_shared)
/// advance the same `RAND_SEED` (and the oracle's `randSeed`), and `cargo test` runs
/// tests on parallel threads — so they all share this one lock to avoid interleaving
/// two sequences (cf. `parse_lock` in `q_shared`). `pub(crate)` so `q_shared`'s
/// oracle tests can take it too. Poison is ignored so one failing test does not
/// cascade into the rest.
#[cfg(all(test, feature = "oracle"))]
pub(crate) fn rand_lock() -> std::sync::MutexGuard<'static, ()> {
    static RAND_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    RAND_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;
    use core::ffi::c_void;

    #[test]
    fn rand_matches_oracle() {
        let _guard = rand_lock();
        // Cover seed-zero, small seeds, both sign-bit halves, max, and the multiplier
        // itself — exercising the wrap on the very first step for the large seeds.
        for &seed in &[0u32, 1, 2, 42, 69069, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff] {
            srand(seed);
            unsafe { oracle::jka_srand(seed) };
            for i in 0..100_000 {
                let got = rand();
                let want = unsafe { oracle::jka_rand() };
                assert_eq!(got, want, "rand mismatch at seed={seed:#x} iter={i}");
            }
        }
    }

    /// A comparator over the leading 4 bytes of each element, read as a host-endian
    /// `i32`. The SAME fn pointer is handed to both the Rust port and the C oracle, so
    /// any quirk in it is shared; the test checks only that both arrange the elements
    /// identically (and that the result is actually ordered).
    extern "C" fn cmp_key(a: *const c_void, b: *const c_void) -> c_int {
        let x = unsafe { core::ptr::read_unaligned(a as *const i32) };
        let y = unsafe { core::ptr::read_unaligned(b as *const i32) };
        (x > y) as c_int - (x < y) as c_int
    }

    /// Build an 8-aligned buffer of `n` elements of `es` bytes. Each element holds a
    /// 4-byte key (per `pattern`) and, when `es >= 8`, its original index as a 4-byte tag
    /// in bytes 4..8 -- so equal-key elements are distinguishable and the unstable sort's
    /// tie-break order is observable in the bytes. Backed by `Vec<u64>` (8-aligned) so the
    /// oracle's `SWAPINIT` can pick the wide `long` swap paths (swaptype 0/1) when
    /// `es % 8 == 0`, exercising exactly the copy-width logic our `swapfunc` collapses.
    /// Sized to at least `es` bytes so the `n == 0` scan's one-past `a + es` stays sound.
    fn build(n: usize, es: usize, pattern: u32) -> Vec<u64> {
        let nbytes = (n * es).max(es);
        let mut words = vec![0u64; (nbytes + 7) / 8];
        let p = words.as_mut_ptr() as *mut u8;
        let mut s = 0x1234_5678u32 ^ pattern.wrapping_mul(0x9E37_79B1);
        for i in 0..n {
            let key: i32 = match pattern {
                0 => i as i32,           // already ascending
                1 => (n - 1 - i) as i32, // descending
                2 => 7,                  // all equal
                3 => {
                    // tiny range -> many duplicates
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    (s >> 24) as i32 % 5
                }
                _ => {
                    // wide range -> few duplicates
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    (s >> 8) as i32
                }
            };
            unsafe {
                core::ptr::write_unaligned(p.add(i * es) as *mut i32, key);
                if es >= 8 {
                    core::ptr::write_unaligned(p.add(i * es + 4) as *mut i32, i as i32);
                }
            }
        }
        words
    }

    /// Inputs for the atof/_atof parity tests: empty, plain ints, signs, leading and
    /// embedded whitespace (incl. control bytes < ' '), the ".5" leading-dot case that
    /// splits atof from _atof, trailing garbage, lone signs, and a few high-bit byte
    /// sequences (their classification as whitespace depends on platform char sign --
    /// the oracle uses the same compiler/target, so parity holds either way).
    fn atof_cases() -> Vec<Vec<u8>> {
        let mut v: Vec<Vec<u8>> = Vec::new();
        for s in [
            "",
            "0",
            "7",
            "123",
            "-123",
            "+45",
            "+45.5",
            "3.14",
            "3.14159",
            "0.001",
            ".5",
            "-.5",
            "12.",
            "-0.0",
            "  3.14",
            "\t\n 42",
            "  -42",
            "999999999",
            "1234567.891011",
            "007",
            "abc",
            "   ",
            "-",
            "+",
            ".",
            "3.14xyz",
            "42 99",
            "  .25rest",
            "0.30000001",
            "100000000000",
        ] {
            v.push(s.as_bytes().to_vec());
        }
        // control / high-bit byte prefixes
        v.push(vec![0x01, 0x1f, b'-', b'4', b'2']);
        v.push(vec![0x80, 0x81, b'5']);
        v.push(vec![0xff, b'9', b'.', b'5']);
        v
    }

    #[test]
    fn atof_matches_oracle() {
        for bytes in atof_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            let got = unsafe { super::atof(cs.as_ptr()) };
            let want = unsafe { oracle::jka_atof(cs.as_ptr()) };
            assert_eq!(
                got.to_bits(),
                want.to_bits(),
                "atof mismatch for {bytes:?}: got {got} want {want}"
            );
        }
    }

    #[test]
    fn _atof_matches_oracle() {
        for bytes in atof_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            let base = cs.as_ptr();
            let mut p1 = base;
            let got = unsafe { super::_atof(&mut p1) };
            let mut p2 = base;
            let want = unsafe { oracle::jka__atof(&mut p2) };
            assert_eq!(
                got.to_bits(),
                want.to_bits(),
                "_atof value mismatch for {bytes:?}: got {got} want {want}"
            );
            let off1 = p1 as usize - base as usize;
            let off2 = p2 as usize - base as usize;
            assert_eq!(off1, off2, "_atof pointer-advance mismatch for {bytes:?}");
        }
    }

    #[test]
    fn memmove_matches_oracle() {
        // (dst_off, src_off, count) over a 32-byte buffer: forward overlap (dst>src),
        // backward overlap (dst<src), exact aliasing (dst==src), disjoint, and the
        // count==0 / count==1 edges. Each runs on an independent copy and the whole
        // buffer plus the returned pointer offset is compared.
        let cases: &[(usize, usize, usize)] = &[
            (0, 0, 0),
            (0, 0, 1),
            (4, 4, 8),   // dst == src
            (8, 4, 12),  // dst > src, overlapping -> back-to-front
            (4, 8, 12),  // dst < src, overlapping -> front-to-back
            (0, 16, 16), // disjoint, dst < src
            (16, 0, 16), // disjoint, dst > src
            (10, 9, 5),  // dst > src, 1-byte overlap stride
            (9, 10, 5),  // dst < src, 1-byte overlap stride
            (0, 0, 32),  // whole buffer onto itself
        ];
        let template: Vec<u8> = (0..32u8).map(|b| b.wrapping_mul(7).wrapping_add(3)).collect();
        for &(dst_off, src_off, count) in cases {
            let mut a = template.clone();
            let mut b = template.clone();
            let ra = unsafe {
                super::memmove(
                    a.as_mut_ptr().add(dst_off) as *mut c_void,
                    a.as_ptr().add(src_off) as *const c_void,
                    count,
                )
            };
            let rb = unsafe {
                oracle::jka_memmove(
                    b.as_mut_ptr().add(dst_off) as *mut c_void,
                    b.as_ptr().add(src_off) as *const c_void,
                    count,
                )
            };
            assert_eq!(a, b, "memmove buffer differs: {dst_off},{src_off},{count}");
            assert_eq!(
                ra as usize - a.as_ptr() as usize,
                rb as usize - b.as_ptr() as usize,
                "memmove return ptr differs: {dst_off},{src_off},{count}"
            );
        }
    }

    // ---------- Group B (`#[cfg(feature = "vm")]`) parity tests ----------
    // Compiled only under `cargo test --features "oracle vm"`; the Rust shims they drive
    // are themselves `vm`-gated, while the `jka_`-prefixed C oracle is always compiled.

    /// Strings for the string-routine parity tests: empty, single chars, words, spacing,
    /// digits, and a NUL-free control/high-bit byte sequence (its `char` signedness is the
    /// platform's, shared by the oracle on the same target).
    #[cfg(feature = "vm")]
    fn str_cases() -> Vec<Vec<u8>> {
        let mut v: Vec<Vec<u8>> = Vec::new();
        for s in [
            "",
            "a",
            "ab",
            "abc",
            "hello",
            "hello world",
            "AaBbCc",
            "  spaced  ",
            "1234567890",
            "the quick brown fox",
        ] {
            v.push(s.as_bytes().to_vec());
        }
        v.push(vec![0x01, 0x7f, 0x80, 0xff]);
        v
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strlen_matches_oracle() {
        for bytes in str_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            let got = unsafe { super::strlen(cs.as_ptr()) };
            let want = unsafe { oracle::jka_strlen(cs.as_ptr()) };
            assert_eq!(got, want, "strlen mismatch for {bytes:?}");
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strcmp_matches_oracle() {
        let cases = str_cases();
        for a in &cases {
            for b in &cases {
                let ca = std::ffi::CString::new(a.clone()).unwrap();
                let cb = std::ffi::CString::new(b.clone()).unwrap();
                let got = unsafe { super::strcmp(ca.as_ptr(), cb.as_ptr()) };
                let want = unsafe { oracle::jka_strcmp(ca.as_ptr(), cb.as_ptr()) };
                assert_eq!(got, want, "strcmp mismatch for {a:?} vs {b:?}");
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strchr_matches_oracle() {
        for bytes in str_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            // present chars, an absent char, high-bit bytes, and 0 (which strchr does NOT
            // match here -- the loop stops before the terminator).
            for &c in &[
                b'a' as c_int,
                b'o' as c_int,
                b' ' as c_int,
                b'z' as c_int,
                0x80,
                0xff,
                0,
            ] {
                let g = unsafe { super::strchr(cs.as_ptr(), c) };
                let w = unsafe { oracle::jka_strchr(cs.as_ptr(), c) };
                let goff = if g.is_null() {
                    -1
                } else {
                    g as isize - cs.as_ptr() as isize
                };
                let woff = if w.is_null() {
                    -1
                } else {
                    w as isize - cs.as_ptr() as isize
                };
                assert_eq!(goff, woff, "strchr mismatch for {bytes:?} c={c}");
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strstr_matches_oracle() {
        let needles: [&[u8]; 8] = [b"", b"o", b"lo", b"world", b"xyz", b"abc", b" ", b"fox"];
        for h in str_cases() {
            for n in needles {
                let ch = std::ffi::CString::new(h.clone()).unwrap();
                let cn = std::ffi::CString::new(n.to_vec()).unwrap();
                let g = unsafe { super::strstr(ch.as_ptr(), cn.as_ptr()) };
                let w = unsafe { oracle::jka_strstr(ch.as_ptr(), cn.as_ptr()) };
                let goff = if g.is_null() {
                    -1
                } else {
                    g as isize - ch.as_ptr() as isize
                };
                let woff = if w.is_null() {
                    -1
                } else {
                    w as isize - ch.as_ptr() as isize
                };
                assert_eq!(goff, woff, "strstr mismatch for {h:?} needle {n:?}");
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strcpy_matches_oracle() {
        for bytes in str_cases() {
            let src = std::ffi::CString::new(bytes.clone()).unwrap();
            // 0xAA-filled dest with padding past the NUL, to confirm only src+NUL is written.
            let cap = bytes.len() + 1 + 8;
            let mut a = vec![0xAAu8; cap];
            let mut b = vec![0xAAu8; cap];
            let ra = unsafe { super::strcpy(a.as_mut_ptr() as *mut c_char, src.as_ptr()) };
            let rb = unsafe { oracle::jka_strcpy(b.as_mut_ptr() as *mut c_char, src.as_ptr()) };
            assert_eq!(a, b, "strcpy buffer mismatch for {bytes:?}");
            assert_eq!(
                ra as usize - a.as_ptr() as usize,
                rb as usize - b.as_ptr() as usize,
                "strcpy return ptr mismatch for {bytes:?}"
            );
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn strcat_matches_oracle() {
        let prefixes: [&[u8]; 3] = [b"", b"pre-", b"existing string "];
        for pre in prefixes {
            for bytes in str_cases() {
                let src = std::ffi::CString::new(bytes.clone()).unwrap();
                let cap = pre.len() + bytes.len() + 1 + 8;
                let mut a = vec![0xAAu8; cap];
                let mut b = vec![0xAAu8; cap];
                a[..pre.len()].copy_from_slice(pre);
                a[pre.len()] = 0;
                b[..pre.len()].copy_from_slice(pre);
                b[pre.len()] = 0;
                let ra = unsafe { super::strcat(a.as_mut_ptr() as *mut c_char, src.as_ptr()) };
                let rb = unsafe { oracle::jka_strcat(b.as_mut_ptr() as *mut c_char, src.as_ptr()) };
                assert_eq!(a, b, "strcat buffer mismatch pre={pre:?} src={bytes:?}");
                assert_eq!(
                    ra as usize - a.as_ptr() as usize,
                    rb as usize - b.as_ptr() as usize,
                    "strcat return ptr mismatch pre={pre:?} src={bytes:?}"
                );
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn tolower_toupper_match_oracle() {
        // Cover the ASCII letters plus boundaries and out-of-range ints (the shims pass
        // those through unchanged); both sides take `int`, so signedness is moot.
        let mut cases: Vec<c_int> = (-1..=256).collect();
        cases.extend_from_slice(&[c_int::MIN, c_int::MAX, -128, 1000]);
        for c in cases {
            assert_eq!(
                super::tolower(c),
                unsafe { oracle::jka_tolower(c) },
                "tolower mismatch at c={c}"
            );
            assert_eq!(
                super::toupper(c),
                unsafe { oracle::jka_toupper(c) },
                "toupper mismatch at c={c}"
            );
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn abs_matches_oracle() {
        for n in [
            0,
            1,
            -1,
            42,
            -42,
            c_int::MAX,
            c_int::MIN, // the -fwrapv overflow case: -INT_MIN wraps to INT_MIN
            c_int::MIN + 1,
            -2_000_000_000,
            2_000_000_000,
        ] {
            assert_eq!(super::abs(n), unsafe { oracle::jka_abs(n) }, "abs mismatch at n={n}");
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn fabs_matches_oracle() {
        // bit-exact (to_bits) to pin the carried quirks: fabs(-0.0) == -0.0, NaN passes
        // through, +/-inf handled.
        for x in [
            0.0,
            -0.0,
            1.5,
            -1.5,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            -1e-300,
        ] {
            let got = super::fabs(x);
            let want = unsafe { oracle::jka_fabs(x) };
            assert_eq!(got.to_bits(), want.to_bits(), "fabs mismatch at x={x}");
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn tan_matches_oracle() {
        // bit-exact: both call the SAME libm sin/cos, so the single divide must agree.
        // Includes points near pi/2 where cos is tiny (large tan) but never exactly zero.
        for &x in &[
            0.0, 0.5, 1.0, -1.0, 0.785_398_163_4, 1.5, 1.57, -2.3, 3.0, -3.0, 100.0, -0.0,
        ] {
            let got = super::tan(x);
            let want = unsafe { oracle::jka_tan(x) };
            assert_eq!(got.to_bits(), want.to_bits(), "tan mismatch at x={x}");
        }
    }

    /// atof_cases plus int-boundary strings (the 32-bit limits and a few past them, where
    /// the carried `-fwrapv` overflow is observable).
    #[cfg(feature = "vm")]
    fn atoi_cases() -> Vec<Vec<u8>> {
        let mut v = atof_cases();
        for s in [
            "2147483647",
            "2147483648",
            "-2147483648",
            "-2147483649",
            "4294967296",
            "99999999999999",
            "+0",
            "-0",
            "  -2147483648rest",
        ] {
            v.push(s.as_bytes().to_vec());
        }
        v
    }

    #[cfg(feature = "vm")]
    #[test]
    fn atoi_matches_oracle() {
        for bytes in atoi_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            let got = unsafe { super::atoi(cs.as_ptr()) };
            let want = unsafe { oracle::jka_atoi(cs.as_ptr()) };
            assert_eq!(got, want, "atoi mismatch for {bytes:?}");
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn _atoi_matches_oracle() {
        for bytes in atoi_cases() {
            let cs = std::ffi::CString::new(bytes.clone()).unwrap();
            let base = cs.as_ptr();
            let mut p1 = base;
            let got = unsafe { super::_atoi(&mut p1) };
            let mut p2 = base;
            let want = unsafe { oracle::jka__atoi(&mut p2) };
            assert_eq!(got, want, "_atoi value mismatch for {bytes:?}");
            // Pointer advance, incl. the all-whitespace case where neither side writes back.
            assert_eq!(
                p1 as usize - base as usize,
                p2 as usize - base as usize,
                "_atoi pointer-advance mismatch for {bytes:?}"
            );
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn AddInt_matches_oracle() {
        const LADJUST: c_int = 0x0000_0004;
        const ZEROPAD: c_int = 0x0000_0080;
        // (width, flags). LADJUST widths are >= the max field (11: INT_MIN's 10 digits +
        // sign), so the faithful `while(width--)` final pad never sees a negative width
        // (which would hang both the Rust port and the C, identically).
        let combos: &[(c_int, c_int)] = &[
            (0, 0),
            (8, 0),
            (8, ZEROPAD),
            (12, LADJUST),
            (12, LADJUST | ZEROPAD),
        ];
        for &val in &[
            0,
            1,
            -1,
            7,
            42,
            -42,
            12345,
            -12345,
            1_000_000,
            -1_000_000,
            c_int::MAX,
            c_int::MIN,
        ] {
            for &(width, flags) in combos {
                let mut a = vec![0xAAu8; 64];
                let mut b = vec![0xAAu8; 64];
                let mut pa = a.as_mut_ptr() as *mut c_char;
                let mut pb = b.as_mut_ptr() as *mut c_char;
                unsafe {
                    super::AddInt(&mut pa, val, width, flags);
                    oracle::jka_AddInt(&mut pb, val, width, flags);
                }
                let adv_a = pa as usize - a.as_ptr() as usize;
                let adv_b = pb as usize - b.as_ptr() as usize;
                assert_eq!(a, b, "AddInt buffer mismatch val={val} width={width} flags={flags:#x}");
                assert_eq!(
                    adv_a, adv_b,
                    "AddInt advance mismatch val={val} width={width} flags={flags:#x}"
                );
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn AddFloat_matches_oracle() {
        // In-range magnitudes (Rust saturating float->int `as` == C truncation here).
        // Output is a decimal string, so the buffers are compared byte-for-byte.
        let fvals: &[f32] = &[
            0.0,
            -0.0,
            1.0,
            0.5,
            2.5,
            3.14159,
            -3.14159,
            123.456,
            -0.001,
            99.999,
            1000.0,
            0.30000001,
            -42.0,
            0.999999,
        ];
        // (width, prec): default prec (-1 -> 6), no fraction (0), and a few explicit precs.
        let combos: &[(c_int, c_int)] = &[(0, -1), (0, 0), (0, 2), (8, 3), (0, 6), (0, 10)];
        for &fval in fvals {
            for &(width, prec) in combos {
                let mut a = vec![0xAAu8; 64];
                let mut b = vec![0xAAu8; 64];
                let mut pa = a.as_mut_ptr() as *mut c_char;
                let mut pb = b.as_mut_ptr() as *mut c_char;
                unsafe {
                    super::AddFloat(&mut pa, fval, width, prec);
                    oracle::jka_AddFloat(&mut pb, fval, width, prec);
                }
                let adv_a = pa as usize - a.as_ptr() as usize;
                let adv_b = pb as usize - b.as_ptr() as usize;
                assert_eq!(
                    a, b,
                    "AddFloat buffer mismatch fval={fval} width={width} prec={prec}"
                );
                assert_eq!(
                    adv_a, adv_b,
                    "AddFloat advance mismatch fval={fval} width={width} prec={prec}"
                );
            }
        }
    }

    #[cfg(feature = "vm")]
    #[test]
    fn AddString_matches_oracle() {
        let strs: [&[u8]; 5] = [b"", b"a", b"hello", b"hello world", b"abc"];
        // (width, prec): full (-1), truncate to 0/3, prec past the end (100), padded width.
        let combos: &[(c_int, c_int)] = &[(0, -1), (0, 0), (0, 3), (0, 100), (10, -1), (10, 4)];
        // First over real strings, then the NULL -> "(null)" path.
        for s in strs {
            let cs = std::ffi::CString::new(s.to_vec()).unwrap();
            for &(width, prec) in combos {
                let mut a = vec![0xAAu8; 64];
                let mut b = vec![0xAAu8; 64];
                let mut pa = a.as_mut_ptr() as *mut c_char;
                let mut pb = b.as_mut_ptr() as *mut c_char;
                unsafe {
                    super::AddString(&mut pa, cs.as_ptr(), width, prec);
                    oracle::jka_AddString(&mut pb, cs.as_ptr(), width, prec);
                }
                assert_eq!(a, b, "AddString buffer mismatch s={s:?} width={width} prec={prec}");
                assert_eq!(
                    pa as usize - a.as_ptr() as usize,
                    pb as usize - b.as_ptr() as usize,
                    "AddString advance mismatch s={s:?} width={width} prec={prec}"
                );
            }
        }
        // NULL string -> "(null)", prec forced to -1.
        for &(width, prec) in combos {
            let mut a = vec![0xAAu8; 64];
            let mut b = vec![0xAAu8; 64];
            let mut pa = a.as_mut_ptr() as *mut c_char;
            let mut pb = b.as_mut_ptr() as *mut c_char;
            unsafe {
                super::AddString(&mut pa, core::ptr::null(), width, prec);
                oracle::jka_AddString(&mut pb, core::ptr::null(), width, prec);
            }
            assert_eq!(a, b, "AddString(NULL) buffer mismatch width={width} prec={prec}");
            assert_eq!(
                pa as usize - a.as_ptr() as usize,
                pb as usize - b.as_ptr() as usize,
                "AddString(NULL) advance mismatch width={width} prec={prec}"
            );
        }
    }

    /// `vsprintf` against the variadic C harness `jka_vsprintf` (same verbatim format-state
    /// machine + `AddInt`/`AddFloat`/`AddString` dispatch; the Rust typed `VsArg` slice and the
    /// C `va_arg` fetch are the two documented sides of the same mechanism deviation). For each
    /// format the Rust port writes via the typed slice and the C harness via matching native
    /// varargs; both the 128-byte buffer (incl. the trailing NUL + untouched `0xAA` tail) and
    /// the returned length must match byte-for-byte.
    #[cfg(feature = "vm")]
    #[test]
    fn vsprintf_matches_oracle() {
        use super::VsArg;
        use core::ffi::CStr;

        fn check(fmt: &CStr, rargs: &[VsArg], c_call: impl FnOnce(*mut c_char) -> c_int) {
            let mut a = vec![0xAAu8; 128];
            let mut b = vec![0xAAu8; 128];
            let ra = unsafe { super::vsprintf(a.as_mut_ptr() as *mut c_char, fmt.as_ptr(), rargs) };
            let rb = c_call(b.as_mut_ptr() as *mut c_char);
            assert_eq!(ra, rb, "vsprintf return mismatch for {fmt:?}");
            assert_eq!(a, b, "vsprintf buffer mismatch for {fmt:?}");
        }

        // Plain literal (no conversions) + the `%%` literal-percent (consumes no arg).
        let f = c"hello world";
        check(f, &[], |b| unsafe { oracle::jka_vsprintf(b, f.as_ptr()) });
        let f = c"100%% done, n=";
        check(f, &[], |b| unsafe { oracle::jka_vsprintf(b, f.as_ptr()) });

        // %d / %i, incl. the INT_MIN wrap path through AddInt. Right-justified widths over-pad
        // safely; the left-adjust (`-`) widths are kept >= 11 (INT_MIN is "-2147483648", 11
        // chars) because AddInt's faithful final `while(width--)` pad loop runs unbounded on a
        // negative width — i.e. when the field is *narrower* than the value (a UB the C shares;
        // cf. the AddInt parity test, which holds LADJUST widths >= the max field for the same
        // reason). So no `%-5d`-style narrow left-adjust here.
        let ints = [0, 1, -1, 7, -7, 42, -42, 12345, -12345, c_int::MAX, c_int::MIN];
        for f in [c"%d", c"%i", c"%5d", c"%05d", c"%1d", c"%12d", c"%-12d", c"%-012d", c"v=%d!"] {
            for &v in &ints {
                check(f, &[VsArg::Int(v)], |b| unsafe {
                    oracle::jka_vsprintf(b, f.as_ptr(), v)
                });
            }
        }

        // %f — C reads va_arg(double); both sides narrow the *same* f64 to f32 for AddFloat.
        // Magnitudes kept in range (saturating float->int `as` == C truncation), matching the
        // AddFloat parity test's domain.
        let floats = [0.0_f64, -0.0, 1.0, 0.5, 2.5, 3.14159, -3.14159, 123.456, -42.0, 99.999];
        for f in [c"%f", c"%.0f", c"%.2f", c"%8.3f", c"%-8.2f", c"%.10f"] {
            for &v in &floats {
                check(f, &[VsArg::Double(v)], |b| unsafe {
                    oracle::jka_vsprintf(b, f.as_ptr(), v)
                });
            }
        }

        // %s with width / precision, incl. the NULL -> "(null)" path.
        let strs = [c"", c"a", c"hello", c"hello world"];
        for f in [c"%s", c"%6s", c"%-6s", c"%.3s", c"[%s]"] {
            for s in strs {
                check(f, &[VsArg::Str(s.as_ptr())], |b| unsafe {
                    oracle::jka_vsprintf(b, f.as_ptr(), s.as_ptr())
                });
            }
            let nul = core::ptr::null::<c_char>();
            check(f, &[VsArg::Str(nul)], |b| unsafe {
                oracle::jka_vsprintf(b, f.as_ptr(), nul)
            });
        }

        // %c, and an unknown conversion (`%q`) — both take an Int slot and emit `(char)val`.
        for f in [c"%c", c"<%c>", c"%q", c"%z"] {
            for &v in &['A' as c_int, 'z' as c_int, '0' as c_int, '!' as c_int] {
                check(f, &[VsArg::Int(v)], |b| unsafe {
                    oracle::jka_vsprintf(b, f.as_ptr(), v)
                });
            }
        }

        // Mixed conversions in one format — exercises argument ordering across all types.
        let f = c"x=%d y=%.1f s=[%-4s] c=%c done%%";
        let s = c"hi";
        let (d, fl, c) = (5_i32, 2.5_f64, '*' as c_int);
        check(
            f,
            &[VsArg::Int(d), VsArg::Double(fl), VsArg::Str(s.as_ptr()), VsArg::Int(c)],
            |b| unsafe { oracle::jka_vsprintf(b, f.as_ptr(), d, fl, s.as_ptr(), c) },
        );
    }

    #[test]
    fn qsort_matches_oracle() {
        // es = 8  -> swaptype 0 (es == sizeof(long), single-long swap)
        // es = 16 -> swaptype 1 (es a multiple of sizeof(long), multi-long swap)
        // es = 4, 12 -> swaptype 2 (es % sizeof(long) != 0, byte swap)
        // Our port byte-swaps in every case; matching the oracle proves the collapse.
        for &es in &[4usize, 8, 12, 16] {
            for &n in &[0usize, 1, 2, 3, 5, 6, 7, 8, 13, 40, 41, 64, 100, 257, 1000] {
                for pattern in 0..5 {
                    let mut a = build(n, es, pattern);
                    let mut b = a.clone();
                    let pa = a.as_mut_ptr() as *mut c_void;
                    let pb = b.as_mut_ptr() as *mut c_void;
                    unsafe {
                        super::qsort(pa, n, es, cmp_key);
                        oracle::jka_qsort(pb, n, es, cmp_key);
                    }
                    let abytes =
                        unsafe { core::slice::from_raw_parts(a.as_ptr() as *const u8, n * es) };
                    let bbytes =
                        unsafe { core::slice::from_raw_parts(b.as_ptr() as *const u8, n * es) };
                    assert_eq!(
                        abytes, bbytes,
                        "qsort vs oracle differ: es={es} n={n} pattern={pattern}"
                    );
                    // Sanity: the port really did order the elements by key.
                    for w in 1..n {
                        let prev = unsafe {
                            core::ptr::read_unaligned(
                                abytes.as_ptr().add((w - 1) * es) as *const i32
                            )
                        };
                        let cur = unsafe {
                            core::ptr::read_unaligned(abytes.as_ptr().add(w * es) as *const i32)
                        };
                        assert!(
                            prev <= cur,
                            "not sorted: es={es} n={n} pattern={pattern} at index {w}"
                        );
                    }
                }
            }
        }
    }

    /// `sscanf` dispatch. The underlying `_atoi`/`_atof` parsers are themselves
    /// oracle-verified above, so this drives the format walk, the per-spec store width
    /// (`int` vs `float`), the unconditional arg-slot advance, and the always-0 return.
    /// There is no verbatim-C oracle for the walk itself: the original mechanism
    /// (`(int**)&fmt+1` stack walk) is 32-bit-VM-specific and broken on a 64-bit ABI.
    #[cfg(feature = "vm")]
    #[test]
    fn sscanf_behaves() {
        // %i/%d/%u all route to _atoi; space-separated ints.
        {
            let buf = std::ffi::CString::new("3 4 5").unwrap();
            let fmt = std::ffi::CString::new("%i %d %u").unwrap();
            let (mut a, mut b, mut c): (c_int, c_int, c_int) = (0, 0, 0);
            let n = unsafe {
                super::sscanf(
                    buf.as_ptr(),
                    fmt.as_ptr(),
                    &[
                        &mut a as *mut c_int as *mut c_void,
                        &mut b as *mut c_int as *mut c_void,
                        &mut c as *mut c_int as *mut c_void,
                    ],
                )
            };
            assert_eq!((a, b, c), (3, 4, 5));
            assert_eq!(n, 0, "sscanf always returns 0 (carried C quirk)");
        }

        // %f routes to _atof; "%f %f %f" is BG_ParseField's F_VECTOR call shape.
        {
            let buf = std::ffi::CString::new("1.5 -2.25 10").unwrap();
            let fmt = std::ffi::CString::new("%f %f %f").unwrap();
            let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
            unsafe {
                super::sscanf(
                    buf.as_ptr(),
                    fmt.as_ptr(),
                    &[
                        &mut x as *mut f32 as *mut c_void,
                        &mut y as *mut f32 as *mut c_void,
                        &mut z as *mut f32 as *mut c_void,
                    ],
                );
            }
            assert_eq!((x, y, z), (1.5f32, -2.25f32, 10.0f32));
        }

        // Literal (non-`%`) chars in fmt are skipped and NEVER matched against the
        // buffer (the shim only ever consumes the buffer through _atoi/_atof).
        {
            let buf = std::ffi::CString::new("42.0").unwrap();
            let fmt = std::ffi::CString::new("val=%f").unwrap();
            let mut v: f32 = 0.0;
            unsafe {
                super::sscanf(buf.as_ptr(), fmt.as_ptr(), &[&mut v as *mut f32 as *mut c_void]);
            }
            assert_eq!(v, 42.0f32);
        }
    }

    /// Native-build counterpart: smoke-test the libc `sscanf` binding. This is the
    /// platform's own `sscanf`, so there is nothing of ours to parity-check — the test
    /// exists to pin the variadic FFI declaration (that it links and that the C call shape
    /// used by `BG_ParseField`'s `F_VECTOR` reads three floats and returns the real match
    /// count, unlike the always-0 VM shim).
    #[cfg(not(feature = "vm"))]
    #[test]
    fn native_sscanf_binding_links() {
        let buf = std::ffi::CString::new("1.5 -2.25 10").unwrap();
        let fmt = std::ffi::CString::new("%f %f %f").unwrap();
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        let n = unsafe { super::sscanf(buf.as_ptr(), fmt.as_ptr(), &mut x, &mut y, &mut z) };
        assert_eq!((x, y, z), (1.5f32, -2.25f32, 10.0f32));
        assert_eq!(n, 3, "full libc sscanf returns the match count (not the VM shim's 0)");
    }
}
