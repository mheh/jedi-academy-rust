//! FFI bindings to the extracted original Raven C functions — a **test-only oracle**.
//!
//! Compiled and linked only under the `oracle` cargo feature (see `build.rs` and
//! `oracle/*.c`). Calling these runs the real C so tests can assert the Rust port
//! in [`crate::codemp::game`] reproduces it bit-for-bit. Never used by the shipped module.
//!
//! C `vec3_t` is `float[3]`, which decays to `float*` as a parameter — hence the
//! `*mut f32` / `*const f32` pointer arguments here.

#![allow(non_snake_case)]

use crate::codemp::game::bg_lib::cmp_t;
use crate::codemp::game::bg_public::{gitem_t, saberMoveData_t, BG_field_t};
use crate::codemp::game::bg_saga_h::{siegeClass_t, siegeTeam_t};
use crate::codemp::game::bg_weapons_h::{ammoData_t, weaponData_t};
use crate::codemp::game::q_shared_h::{bladeInfo_t, cplane_t, qint64, saberInfo_t, stringID_table_t};
use core::ffi::{c_char, c_int, c_short, c_uint, c_ulong, c_void};

/// Mirror of the oracle's `jka_saberFace_t` (`{ float v1[3]; v2[3]; v3[3]; }`, w_saber.c:2309) —
/// one triangular face of a saber's collision hull. Matches the Rust port's `saberFace_t` and the
/// C oracle struct so `jka_G_BuildSaberFaces`/`jka_G_SaberFaceCollisionCheck` share a buffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OracleSaberFace {
    pub v1: [f32; 3],
    pub v2: [f32; 3],
    pub v3: [f32; 3],
}

extern "C" {
    /// `float Q_rsqrt( float number )` — fast inverse square root.
    pub fn Q_rsqrt(number: f32) -> f32;
    /// `float Q_fabs( float f )`.
    pub fn Q_fabs(f: f32) -> f32;
    /// `vec_t VectorNormalize( vec3_t v )` — normalizes in place, returns old length.
    pub fn VectorNormalize(v: *mut f32) -> f32;
    /// `vec_t VectorNormalize2( const vec3_t v, vec3_t out )`.
    pub fn VectorNormalize2(v: *const f32, out: *mut f32) -> f32;
    /// `int Q_rand( int *seed )`.
    pub fn Q_rand(seed: *mut c_int) -> c_int;
    /// `float Q_random( int *seed )`.
    pub fn Q_random(seed: *mut c_int) -> f32;
    /// `float Q_crandom( int *seed )`.
    pub fn Q_crandom(seed: *mut c_int) -> f32;

    /// `vec_t _DotProduct( const vec3_t v1, const vec3_t v2 )`.
    pub fn _DotProduct(v1: *const f32, v2: *const f32) -> f32;
    /// `void _VectorSubtract( const vec3_t a, const vec3_t b, vec3_t out )`.
    pub fn _VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    /// `void _VectorAdd( const vec3_t a, const vec3_t b, vec3_t out )`.
    pub fn _VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    /// `void _VectorScale( const vec3_t in, vec_t scale, vec3_t out )`.
    pub fn _VectorScale(in_: *const f32, scale: f32, out: *mut f32);
    /// `void _VectorMA( const vec3_t veca, float scale, const vec3_t vecb, vec3_t vecc )`.
    pub fn _VectorMA(veca: *const f32, scale: f32, vecb: *const f32, vecc: *mut f32);
    /// `void _VectorCopy( const vec3_t in, vec3_t out )`.
    pub fn _VectorCopy(in_: *const f32, out: *mut f32);
    /// `void CrossProduct( const vec3_t v1, const vec3_t v2, vec3_t cross )`.
    pub fn CrossProduct(v1: *const f32, v2: *const f32, cross: *mut f32);
    /// `void VectorInverse( vec3_t v )`.
    pub fn VectorInverse(v: *mut f32);
    /// `vec_t VectorLength( const vec3_t v )`.
    pub fn VectorLength(v: *const f32) -> f32;
    /// `vec_t VectorLengthSquared( const vec3_t v )`.
    pub fn VectorLengthSquared(v: *const f32) -> f32;
    /// `int VectorCompare( const vec3_t v1, const vec3_t v2 )`.
    pub fn VectorCompare(v1: *const f32, v2: *const f32) -> c_int;
    /// `vec_t Distance( const vec3_t p1, const vec3_t p2 )`.
    pub fn Distance(p1: *const f32, p2: *const f32) -> f32;
    /// `vec_t DistanceSquared( const vec3_t p1, const vec3_t p2 )`.
    pub fn DistanceSquared(p1: *const f32, p2: *const f32) -> f32;
    /// `void VectorNormalizeFast( vec3_t v )`.
    pub fn VectorNormalizeFast(v: *mut f32);
    /// `float LerpAngle (float from, float to, float frac)`.
    pub fn LerpAngle(from: f32, to: f32, frac: f32) -> f32;
    /// `float AngleSubtract( float a1, float a2 )`.
    pub fn AngleSubtract(a1: f32, a2: f32) -> f32;
    /// `void AnglesSubtract( vec3_t v1, vec3_t v2, vec3_t v3 )`.
    pub fn AnglesSubtract(v1: *const f32, v2: *const f32, v3: *mut f32);
    /// `float AngleMod(float a)`.
    pub fn AngleMod(a: f32) -> f32;
    /// `float AngleNormalize360 ( float angle )`.
    pub fn AngleNormalize360(angle: f32) -> f32;
    /// `float AngleNormalize180 ( float angle )`.
    pub fn AngleNormalize180(angle: f32) -> f32;
    /// `float AngleDelta ( float angle1, float angle2 )`.
    pub fn AngleDelta(angle1: f32, angle2: f32) -> f32;
    /// `void vectoangles( const vec3_t value1, vec3_t angles )`.
    pub fn vectoangles(value1: *const f32, angles: *mut f32);
    /// `void AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up )`.
    /// Any of forward/right/up may be NULL.
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    /// `void ProjectPointOnPlane( vec3_t dst, const vec3_t p, const vec3_t normal )`.
    pub fn ProjectPointOnPlane(dst: *mut f32, p: *const f32, normal: *const f32);
    /// `void AnglesToAxis( const vec3_t angles, vec3_t axis[3] )` — axis is 9 contiguous floats.
    pub fn AnglesToAxis(angles: *const f32, axis: *mut f32);
    /// `void AxisClear( vec3_t axis[3] )`.
    pub fn AxisClear(axis: *mut f32);
    /// `void AxisCopy( vec3_t in[3], vec3_t out[3] )`.
    pub fn AxisCopy(in_: *const f32, out: *mut f32);
    /// `void MakeNormalVectors( const vec3_t forward, vec3_t right, vec3_t up )`.
    pub fn MakeNormalVectors(forward: *const f32, right: *mut f32, up: *mut f32);
    /// `void VectorRotate( vec3_t in, vec3_t matrix[3], vec3_t out )` — matrix is 9 floats.
    pub fn VectorRotate(in_: *const f32, matrix: *const f32, out: *mut f32);
    /// `void PerpendicularVector( vec3_t dst, const vec3_t src )`.
    pub fn PerpendicularVector(dst: *mut f32, src: *const f32);

    /// `float RadiusFromBounds( const vec3_t mins, const vec3_t maxs )`.
    pub fn RadiusFromBounds(mins: *const f32, maxs: *const f32) -> f32;
    /// `void ClearBounds( vec3_t mins, vec3_t maxs )`.
    pub fn ClearBounds(mins: *mut f32, maxs: *mut f32);
    /// `vec_t DistanceHorizontal( const vec3_t p1, const vec3_t p2 )`.
    pub fn DistanceHorizontal(p1: *const f32, p2: *const f32) -> f32;
    /// `vec_t DistanceHorizontalSquared( const vec3_t p1, const vec3_t p2 )`.
    pub fn DistanceHorizontalSquared(p1: *const f32, p2: *const f32) -> f32;
    /// `void AddPointToBounds( const vec3_t v, vec3_t mins, vec3_t maxs )`.
    pub fn AddPointToBounds(v: *const f32, mins: *mut f32, maxs: *mut f32);

    /// `unsigned ColorBytes3 (float r, float g, float b)` — top byte is indeterminate.
    pub fn ColorBytes3(r: f32, g: f32, b: f32) -> c_uint;
    /// `unsigned ColorBytes4 (float r, float g, float b, float a)`.
    pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> c_uint;
    /// `float NormalizeColor( const vec3_t in, vec3_t out )`.
    pub fn NormalizeColor(in_: *const f32, out: *mut f32) -> f32;

    /// `signed char ClampChar( int i )` (`signed char` is always i8).
    pub fn ClampChar(i: c_int) -> i8;
    /// `signed short ClampShort( int i )`.
    pub fn ClampShort(i: c_int) -> i16;
    /// `void Vector4Scale( const vec4_t in, vec_t scale, vec4_t out )`.
    pub fn Vector4Scale(in_: *const f32, scale: f32, out: *mut f32);
    /// `int Q_log2( int val )`.
    pub fn Q_log2(val: c_int) -> c_int;

    /// `int DirToByte( vec3_t dir )` — closest quantized normal index.
    pub fn DirToByte(dir: *const f32) -> c_int;
    /// `void ByteToDir( int b, vec3_t dir )`.
    pub fn ByteToDir(b: c_int, dir: *mut f32);

    /// `void MatrixMultiply(float in1[3][3], float in2[3][3], float out[3][3])` —
    /// all three are 9 contiguous floats.
    pub fn MatrixMultiply(in1: *const f32, in2: *const f32, out: *mut f32);
    /// `qboolean PlaneFromPoints( vec4_t plane, const vec3_t a, const vec3_t b, const vec3_t c )`.
    pub fn PlaneFromPoints(plane: *mut f32, a: *const f32, b: *const f32, c: *const f32) -> c_int;
    /// `void RotatePointAroundVector( vec3_t dst, const vec3_t dir, const vec3_t point, float degrees )`.
    pub fn RotatePointAroundVector(dst: *mut f32, dir: *const f32, point: *const f32, degrees: f32);
    /// `void RotateAroundDirection( vec3_t axis[3], float yaw )` — axis is 9 contiguous floats.
    pub fn RotateAroundDirection(axis: *mut f32, yaw: f32);

    /// `void NormalToLatLong( const vec3_t normal, byte bytes[2] )`.
    pub fn NormalToLatLong(normal: *const f32, bytes: *mut u8);

    /// `void Rand_Init(int seed)` — seed the shared `holdrand` state.
    pub fn Rand_Init(seed: c_int);
    /// `float flrand(float min, float max)`.
    pub fn flrand(min: f32, max: f32) -> f32;
    /// `int irand(int min, int max)`.
    pub fn irand(min: c_int, max: c_int) -> c_int;
    /// `int Q_irand(int value1, int value2)` — PC wrapper over `irand` (lives in q_math.c).
    pub fn Q_irand(value1: c_int, value2: c_int) -> c_int;
    /// `float Q_flrand(float min, float max)` — rww wrapper over `flrand` (lives in q_math.c).
    pub fn Q_flrand(min: f32, max: f32) -> f32;

    /// `float powf ( float x, int y )` — JKA's integer-power (renamed `jka_powf` in the
    /// oracle to avoid the libm `powf(float,float)` collision).
    pub fn jka_powf(x: f32, y: c_int) -> f32;
    /// `double fmod( double x, double y )` — JKA's `#ifdef Q3_VM` libc shim (truncating
    /// quotient), renamed `jka_fmod` in the oracle to avoid the libm `fmod` collision.
    pub fn jka_fmod(x: f64, y: f64) -> f64;

    /// `float DotProductNormalize( const vec3_t inVec1, const vec3_t inVec2 )`.
    pub fn DotProductNormalize(in_vec1: *const f32, in_vec2: *const f32) -> f32;
    /// `qboolean G_FindClosestPointOnLineSegment( const vec3_t start, const vec3_t end, const vec3_t from, vec3_t result )`.
    pub fn G_FindClosestPointOnLineSegment(
        start: *const f32,
        end: *const f32,
        from: *const f32,
        result: *mut f32,
    ) -> c_int;
    /// `float G_PointDistFromLineSegment( const vec3_t start, const vec3_t end, const vec3_t from )`.
    pub fn G_PointDistFromLineSegment(start: *const f32, end: *const f32, from: *const f32) -> f32;

    /// `void SetPlaneSignbits (cplane_t *out)`.
    pub fn SetPlaneSignbits(out: *mut cplane_t);
    /// `int BoxOnPlaneSide (vec3_t emins, vec3_t emaxs, struct cplane_s *p)` — portable C version.
    pub fn BoxOnPlaneSide(emins: *const f32, emaxs: *const f32, p: *const cplane_t) -> c_int;
}

// ---- q_shared.c (string / parsing / info-string support) ----
extern "C" {
    /// `int GetIDForString ( stringID_table_t *table, const char *string )`.
    pub fn GetIDForString(table: *const stringID_table_t, string: *const c_char) -> c_int;
    /// `const char *GetStringForID( stringID_table_t *table, int id )`.
    pub fn GetStringForID(table: *const stringID_table_t, id: c_int) -> *const c_char;

    /// `void COM_BeginParseSession( const char *name )`.
    pub fn COM_BeginParseSession(name: *const c_char);
    /// `int COM_GetCurrentParseLine( void )`.
    pub fn COM_GetCurrentParseLine() -> c_int;
    /// `char *COM_Parse( const char **data_p )`.
    pub fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char;
    /// `int COM_Compress( char *data_p )`.
    pub fn COM_Compress(data_p: *mut c_char) -> c_int;
    /// `qboolean COM_ParseString( const char **data, const char **s )`.
    pub fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> c_int;
    /// `qboolean COM_ParseInt( const char **data, int *i )`.
    pub fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> c_int;
    /// `qboolean COM_ParseFloat( const char **data, float *f )`.
    pub fn COM_ParseFloat(data: *mut *const c_char, f: *mut f32) -> c_int;
    /// `qboolean COM_ParseVec4( const char **buffer, vec4_t *c)`.
    pub fn COM_ParseVec4(buffer: *mut *const c_char, c: *mut [f32; 4]) -> c_int;
    /// `void COM_MatchToken( const char **buf_p, char *match )`.
    pub fn COM_MatchToken(buf_p: *mut *const c_char, match_: *const c_char);
    /// `void SkipBracedSection (const char **program)`.
    pub fn SkipBracedSection(program: *mut *const c_char);
    /// `void SkipRestOfLine ( const char **data )`.
    pub fn SkipRestOfLine(data: *mut *const c_char);
    /// `void Parse1DMatrix (const char **buf_p, int x, float *m)`.
    pub fn Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f32);
    /// `void Parse2DMatrix (const char **buf_p, int y, int x, float *m)`.
    pub fn Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f32);
    /// `void Parse3DMatrix (const char **buf_p, int z, int y, int x, float *m)`.
    pub fn Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f32);
    /// `const char *SkipWhitespace( const char *data, qboolean *hasNewLines )`.
    pub fn SkipWhitespace(data: *const c_char, hasNewLines: *mut c_int) -> *const c_char;
    /// `char *COM_ParseExt( const char **data_p, qboolean allowLineBreaks )`.
    pub fn COM_ParseExt(data_p: *mut *const c_char, allowLineBreaks: c_int) -> *mut c_char;
    /// `int COM_ParseInfos( char *buf, int max, char infos[][MAX_INFO_STRING] )`.
    pub fn COM_ParseInfos(
        buf: *const c_char,
        max: c_int,
        infos: *mut [c_char; crate::codemp::game::q_shared_h::MAX_INFO_STRING],
    ) -> c_int;

    /// `char *COM_SkipPath (char *pathname)`.
    pub fn COM_SkipPath(pathname: *mut c_char) -> *mut c_char;
    /// `void COM_StripExtension( const char *in, char *out )`.
    pub fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    /// `void COM_DefaultExtension (char *path, int maxSize, const char *extension )`.
    pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char);

    /// `int Com_Clampi( int min, int max, int value )`.
    pub fn Com_Clampi(min: c_int, max: c_int, value: c_int) -> c_int;
    /// `float Com_Clamp( float min, float max, float value )`.
    pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32;

    /// `short ShortSwap (short l)`.
    pub fn ShortSwap(l: i16) -> i16;
    /// `short ShortNoSwap (short l)`.
    pub fn ShortNoSwap(l: i16) -> i16;
    /// `int LongSwap (int l)`.
    pub fn LongSwap(l: c_int) -> c_int;
    /// `int LongNoSwap (int l)`.
    pub fn LongNoSwap(l: c_int) -> c_int;
    /// `qint64 Long64Swap (qint64 ll)`.
    pub fn Long64Swap(ll: qint64) -> qint64;
    /// `qint64 Long64NoSwap (qint64 ll)`.
    pub fn Long64NoSwap(ll: qint64) -> qint64;
    /// `float FloatSwap (const float *f)`.
    pub fn FloatSwap(f: *const f32) -> f32;
    /// `float FloatNoSwap (const float *f)`.
    pub fn FloatNoSwap(f: *const f32) -> f32;

    /// `int Q_isprint( int c )`.
    pub fn Q_isprint(c: c_int) -> c_int;
    /// `int Q_islower( int c )`.
    pub fn Q_islower(c: c_int) -> c_int;
    /// `int Q_isupper( int c )`.
    pub fn Q_isupper(c: c_int) -> c_int;
    /// `int Q_isalpha( int c )`.
    pub fn Q_isalpha(c: c_int) -> c_int;

    /// `char* Q_strrchr( const char* string, int c )`.
    pub fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char;
    /// `void Q_strncpyz( char *dest, const char *src, int destsize )`.
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    /// `int Q_stricmpn (const char *s1, const char *s2, int n)`.
    pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    /// `int Q_strncmp (const char *s1, const char *s2, int n)`.
    pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    /// `int Q_stricmp (const char *s1, const char *s2)`.
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    /// `char *Q_strlwr( char *s1 )`.
    pub fn Q_strlwr(s1: *mut c_char) -> *mut c_char;
    /// `char *Q_strupr( char *s1 )`.
    pub fn Q_strupr(s1: *mut c_char) -> *mut c_char;
    /// `void Q_strcat( char *dest, int size, const char *src )`.
    pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char);
    /// `void QDECL Com_sprintf( char *dest, int size, const char *fmt, ...)` —
    /// declared variadic so parity tests can drive the real C with concrete args.
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    /// `char * QDECL va( const char *format, ... )`.
    pub fn va(format: *const c_char, ...) -> *mut c_char;

    /// `int Q_PrintStrlen( const char *string )`.
    pub fn Q_PrintStrlen(string: *const c_char) -> c_int;
    /// `char *Q_CleanStr( char *string )`.
    pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char;

    /// `char *Info_ValueForKey( const char *s, const char *key )`.
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    /// `void Info_NextPair( const char **head, char *key, char *value )`.
    pub fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char);
    /// `qboolean Info_Validate( const char *s )`.
    pub fn Info_Validate(s: *const c_char) -> c_int;
    /// `void Info_RemoveKey( char *s, const char *key )`.
    pub fn Info_RemoveKey(s: *mut c_char, key: *const c_char);
    /// `void Info_RemoveKey_Big( char *s, const char *key )`.
    pub fn Info_RemoveKey_Big(s: *mut c_char, key: *const c_char);
    /// `void Info_SetValueForKey( char *s, const char *key, const char *value )`.
    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
    /// `void Info_SetValueForKey_Big( char *s, const char *key, const char *value )`.
    pub fn Info_SetValueForKey_Big(s: *mut c_char, key: *const c_char, value: *const c_char);

    /// `#define random()` (q_shared.h:1492; oracle: `jka_q_random`, renamed to dodge
    /// POSIX `random()`).
    pub fn jka_q_random() -> f32;
    /// `#define crandom()` (q_shared.h:1493; oracle: `jka_crandom`). Returns `double`.
    pub fn jka_crandom() -> f64;
}

// w_saber.c (oracle/w_saber_oracle.c): the saber subsystem's leaf helpers. `RandFloat`'s
// rand() is the game's bg_lib LCG (jka_rand, same static lib), not libc — see the file
// header. Renamed `jka_RandFloat` to dodge any host symbol clash.
extern "C" {
    /// `float RandFloat(float min, float max)` (w_saber.c:41; oracle: `jka_RandFloat`).
    pub fn jka_RandFloat(min: f32, max: f32) -> f32;
    /// `qboolean HasSetSaberOnly(void)` (w_saber.c:9006; oracle: `jka_HasSetSaberOnly`).
    /// The three cvar `.integer` values it reads are passed in as parameters so the oracle
    /// is a pure function; `qboolean` -> `c_int`.
    pub fn jka_HasSetSaberOnly(
        g_gametype: core::ffi::c_int,
        g_duelWeaponDisable: core::ffi::c_int,
        g_weaponDisable: core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `int VectorCompare2( const vec3_t v1, const vec3_t v2 )` (w_saber.c:4858;
    /// oracle: `jka_VectorCompare2`). `vec3_t` -> `*const f32`.
    pub fn jka_VectorCompare2(v1: *const f32, v2: *const f32) -> core::ffi::c_int;
    /// `int WPDEBUG_SaberColor( saber_colors_t saberColor )` (w_saber.c:2677; oracle:
    /// `jka_WPDEBUG_SaberColor`). `saber_colors_t` -> `c_int`.
    pub fn jka_WPDEBUG_SaberColor(saberColor: core::ffi::c_int) -> core::ffi::c_int;
    /// `qboolean G_PrettyCloseIGuess(float a, float b, float tolerance)` (w_saber.c:7529;
    /// oracle: `jka_G_PrettyCloseIGuess`). `qboolean` -> `c_int`.
    pub fn jka_G_PrettyCloseIGuess(a: f32, b: f32, tolerance: f32) -> core::ffi::c_int;
    /// `float WP_SaberBladeLength( saberInfo_t *saber )` (w_saber.c:2642; oracle:
    /// `jka_WP_SaberBladeLength`). Pass-the-read-fields: the blade `lengthMax` array and
    /// `numBlades` instead of the full `saberInfo_t`.
    pub fn jka_WP_SaberBladeLength(lengthMax: *const f32, numBlades: core::ffi::c_int) -> f32;
    /// `qboolean G_SaberInBackAttack(int move)` (w_saber.c:2291; oracle:
    /// `jka_G_SaberInBackAttack`). `qboolean` -> `c_int`.
    pub fn jka_G_SaberInBackAttack(r#move: core::ffi::c_int) -> core::ffi::c_int;
    /// `int WP_MissileBlockForBlock( int saberBlock )` (w_saber.c:8657; oracle:
    /// `jka_WP_MissileBlockForBlock`). Pure `saberBlockedType_t` -> `*_PROJ` switch.
    pub fn jka_WP_MissileBlockForBlock(saberBlock: core::ffi::c_int) -> core::ffi::c_int;

    /// `int G_GetParryForBlock(int block)` (w_saber.c:1764; oracle: `jka_G_GetParryForBlock`).
    /// Pure `saberBlockedType_t` -> parry/reflect `saberMoveName_t` switch.
    pub fn jka_G_GetParryForBlock(block: core::ffi::c_int) -> core::ffi::c_int;
    /// `int G_KnockawayForParry(int move)` (w_saber.c:2083; oracle: `jka_G_KnockawayForParry`).
    /// Pure `saberMoveName_t` parry -> `LS_K1_*` knockaway switch (default = `LS_K1_TR`).
    pub fn jka_G_KnockawayForParry(r#move: core::ffi::c_int) -> core::ffi::c_int;
    /// `int G_SaberLockAnim(int attackerSaberStyle, int defenderSaberStyle, int topOrSide,
    /// int lockOrBreakOrSuperBreak, int winOrLose)` (w_saber.c:967; oracle: `jka_G_SaberLockAnim`).
    /// Pure `saber_styles_t`×`SABERLOCK_*` -> `BOTH_LK_*` animNumber_t switch math.
    pub fn jka_G_SaberLockAnim(
        attackerSaberStyle: core::ffi::c_int,
        defenderSaberStyle: core::ffi::c_int,
        topOrSide: core::ffi::c_int,
        lockOrBreakOrSuperBreak: core::ffi::c_int,
        winOrLose: core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `int G_SaberAttackPower(gentity_t *ent, qboolean attacking)` (w_saber.c:120; oracle:
    /// `jka_G_SaberAttackPower`). Pass-the-read-fields: the `client`/`ps` fields and cvar globals
    /// the body reads are passed as scalars (`saberAnimLevel`, `lastSaberStorageTime`,
    /// `olderIsValid`, the two saber-base vec3s, `brokenLimbs`, `level.time`, `g_gametype`,
    /// `sess.duelTeam`). `qboolean` -> `c_int`.
    pub fn jka_G_SaberAttackPower(
        saberAnimLevel: core::ffi::c_int,
        attacking: core::ffi::c_int,
        lastSaberStorageTime: core::ffi::c_int,
        olderIsValid: core::ffi::c_int,
        lastSaberBase_Always: *const f32,
        olderSaberBase: *const f32,
        brokenLimbs: core::ffi::c_int,
        levelTime: core::ffi::c_int,
        g_gametype: core::ffi::c_int,
        duelTeam: core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `void G_SabCol_CalcPlaneEq(vec3_t x, vec3_t y, vec3_t z, float *planeEq)` (w_saber.c:2436;
    /// oracle: `jka_G_SabCol_CalcPlaneEq`). `vec3_t` -> `*const f32`, `float *planeEq` -> `*mut f32`.
    pub fn jka_G_SabCol_CalcPlaneEq(
        x: *const f32,
        y: *const f32,
        z: *const f32,
        planeEq: *mut f32,
    );
    /// `int G_SabCol_PointRelativeToPlane(vec3_t pos, float *side, float *planeEq)`
    /// (w_saber.c:2445; oracle: `jka_G_SabCol_PointRelativeToPlane`). `vec3_t` -> `*const f32`,
    /// the two `float *` params -> `*mut f32` / `*const f32`.
    pub fn jka_G_SabCol_PointRelativeToPlane(
        pos: *const f32,
        side: *mut f32,
        planeEq: *const f32,
    ) -> core::ffi::c_int;
    /// `static GAME_INLINE void G_BuildSaberFaces(...)` (w_saber.c:2317; oracle:
    /// `jka_G_BuildSaberFaces`). ORACLE DEVIATION: writes into a caller-provided `faces` buffer
    /// (>=12 entries) and returns `fNum` instead of the `*fNum`/`**fList` out-params over the
    /// file-static. `vec3_t` -> `*const f32`; `saberFace_t` -> [`OracleSaberFace`].
    pub fn jka_G_BuildSaberFaces(
        base: *const f32,
        tip: *const f32,
        radius: f32,
        fwd: *const f32,
        right: *const f32,
        faces: *mut OracleSaberFace,
    ) -> core::ffi::c_int;
    /// `static GAME_INLINE qboolean G_SaberFaceCollisionCheck(...)` (w_saber.c:2462; oracle:
    /// `jka_G_SaberFaceCollisionCheck`). `vec3_t` -> `*const f32` (in) / `*mut f32`
    /// (atkMins/atkMaxs/impactPoint, may be mutated); `saberFace_t` -> [`OracleSaberFace`];
    /// `qboolean` -> `c_int`. The `static` working-buffers become plain locals.
    pub fn jka_G_SaberFaceCollisionCheck(
        fNum: core::ffi::c_int,
        fList: *mut OracleSaberFace,
        atkStart: *const f32,
        atkEnd: *const f32,
        atkMins: *mut f32,
        atkMaxs: *mut f32,
        impactPoint: *mut f32,
    ) -> core::ffi::c_int;
    /// `static GAME_INLINE int G_PowerLevelForSaberAnim(gentity_t *ent, int saberNum,
    /// qboolean mySaberHit)` (w_saber.c:2859; oracle: `jka_G_PowerLevelForSaberAnim`).
    /// Pass-the-read-fields: the C reads `ps.torsoAnim`/`ps.torsoTimer`, derives
    /// `animTimeElapsed = BG_AnimLength(localAnimIndex, anim) - animTimer`, and reads
    /// `saber[saberNum].type`; those four scalars are passed in so the oracle is a pure int
    /// mapper. `qboolean` -> `c_int`. The `!ent`/`!client`/`saberNum >= MAX_SABERS` guard is
    /// checked Rust-side. Anim params use the real anims.h numeric values.
    pub fn jka_G_PowerLevelForSaberAnim(
        anim: core::ffi::c_int,
        animTimer: core::ffi::c_int,
        animTimeElapsed: core::ffi::c_int,
        saberType: core::ffi::c_int,
        saberNum: core::ffi::c_int,
        mySaberHit: core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `static GAME_INLINE void SetSaberBoxSize(gentity_t *saberent)` (w_saber.c:376; oracle:
    /// `jka_SetSaberBoxSize`). Pass-the-read-fields: the C derefs deep owner/blade state and the
    /// `level`/`g_entities` globals, so the read fields are passed as scalars/arrays and the
    /// resulting `r.mins`/`r.maxs` are written back. The two state predicates
    /// (`PM_SaberInBrokenParry` / `BG_SuperBreakLoseAnim`) collapse into the `inBrokenParryOrLose`
    /// flag; `blade00StorageTime` is `saber[0].blade[0].storageTime` (j==k==0 at that check in the
    /// tested forceBlock==0 cases); `modelPresent[j]` mirrors `saber[j].model[0] != 0` (and
    /// `modelPresent[1]` drives `dualSabers`). `saberFlags2[]`/`bladeStyle2Start[]` feed the PC
    /// broken-parry alwaysBlock/forceBlock pass. Owner-presence guards + the `#ifndef FINAL_BUILD`
    /// print are exercised/dropped Rust-side. `vec3_t` -> `[f32;3]`.
    pub fn jka_SetSaberBoxSize(
        r_mins: *mut f32,
        r_maxs: *mut f32,
        currentOrigin: *const f32,
        inBrokenParryOrLose: core::ffi::c_int,
        levelTime: core::ffi::c_int,
        lastSaberStorageTime: core::ffi::c_int,
        blade00StorageTime: core::ffi::c_int,
        saberHolstered: core::ffi::c_int,
        modelPresent: *const core::ffi::c_int,
        numBlades: *const core::ffi::c_int,
        saberFlags2: *const core::ffi::c_int,
        bladeStyle2Start: *const core::ffi::c_int,
        muzzlePoint: *const [[f32; 3]; 8],
        muzzleDir: *const [[f32; 3]; 8],
        lengthMax: *const [f32; 8],
    );
}

// w_force.c (oracle/w_force_oracle.c): the force subsystem's pure predicate leaves. Both
// take only the few fields their C bodies read (the `jka_HasSetSaberOnly` pass-the-read-fields
// precedent), so the oracle stays a pure function; the C NULL-guards (`if (ps)` / `if (!fd)`)
// are reproduced via a `has_*` int flag. Renamed `jka_*` to dodge any host symbol; `qboolean`
// -> `c_int`.
extern "C" {
    /// `qboolean WP_HasForcePowers( const playerState_t *ps )` (w_force.c:5015; oracle:
    /// `jka_WP_HasForcePowers`). `ps` reduced to its `fd.forcePowerLevel[18]` array + a
    /// `has_ps` flag (0 = NULL ps).
    pub fn jka_WP_HasForcePowers(
        has_ps: core::ffi::c_int,
        forcePowerLevel: *const core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `qboolean G_IsMindTricked( forcedata_t *fd, int client )` (w_force.c:4162; oracle:
    /// `jka_G_IsMindTricked`). `fd` reduced to its four `forceMindtrickTargetIndex*` scalars +
    /// a `has_fd` flag (0 = NULL fd).
    pub fn jka_G_IsMindTricked(
        has_fd: core::ffi::c_int,
        trickIndex1: core::ffi::c_int,
        trickIndex2: core::ffi::c_int,
        trickIndex3: core::ffi::c_int,
        trickIndex4: core::ffi::c_int,
        client: core::ffi::c_int,
    ) -> core::ffi::c_int;
    /// `void WP_AddToClientBitflags( gentity_t *ent, int entNum )` (w_force.c:1301; oracle:
    /// `jka_WP_AddToClientBitflags`). `ent` reduced to in/out pointers to its four
    /// `s.trickedentindex*` scalars + a `has_ent` flag (0 = NULL ent).
    pub fn jka_WP_AddToClientBitflags(
        has_ent: core::ffi::c_int,
        trickIndex1: *mut core::ffi::c_int,
        trickIndex2: *mut core::ffi::c_int,
        trickIndex3: *mut core::ffi::c_int,
        trickIndex4: *mut core::ffi::c_int,
        entNum: core::ffi::c_int,
    );
    /// `void WP_AddAsMindtricked( forcedata_t *fd, int entNum )` (w_force.c:2509; oracle:
    /// `jka_WP_AddAsMindtricked`). `fd` reduced to in/out pointers to its four
    /// `forceMindtrickTargetIndex*` scalars + a `has_fd` flag (0 = NULL fd).
    pub fn jka_WP_AddAsMindtricked(
        has_fd: core::ffi::c_int,
        targetIndex1: *mut core::ffi::c_int,
        targetIndex2: *mut core::ffi::c_int,
        targetIndex3: *mut core::ffi::c_int,
        targetIndex4: *mut core::ffi::c_int,
        entNum: core::ffi::c_int,
    );
    /// `static void RemoveTrickedEnt( forcedata_t *fd, int client )` (w_force.c:4206; oracle:
    /// `jka_RemoveTrickedEnt`). `fd` reduced to in/out pointers to its four
    /// `forceMindtrickTargetIndex*` scalars + a `has_fd` flag (0 = NULL fd). The clear-bit mirror
    /// of `jka_WP_AddAsMindtricked`.
    pub fn jka_RemoveTrickedEnt(
        has_fd: core::ffi::c_int,
        targetIndex1: *mut core::ffi::c_int,
        targetIndex2: *mut core::ffi::c_int,
        targetIndex3: *mut core::ffi::c_int,
        targetIndex4: *mut core::ffi::c_int,
        client: core::ffi::c_int,
    );
    /// `qboolean G_InGetUpAnim( playerState_t *ps )` (w_force.c:2977; oracle:
    /// `jka_G_InGetUpAnim`). `ps` reduced to its two read fields, `legsAnim` and `torsoAnim`
    /// (the pass-the-read-fields precedent).
    pub fn jka_G_InGetUpAnim(
        legsAnim: core::ffi::c_int,
        torsoAnim: core::ffi::c_int,
    ) -> core::ffi::c_int;
}

// tri_coll_test.c (oracle/tri_coll_test_oracle.c): Tomas Moller's triangle/triangle
// intersection test. Each `vec3_t` parameter is `float[3]`, which decays to `float*`; the
// C is non-const but read-only, so `*const f32` here. Both return `qboolean` (= `c_int`).
extern "C" {
    /// `qboolean coplanar_tri_tri( vec3_t N, vec3_t V0, vec3_t V1, vec3_t V2, vec3_t U0, vec3_t U1, vec3_t U2 )`.
    pub fn coplanar_tri_tri(
        N: *const f32,
        V0: *const f32,
        V1: *const f32,
        V2: *const f32,
        U0: *const f32,
        U1: *const f32,
        U2: *const f32,
    ) -> c_int;
    /// `qboolean tri_tri_intersect( vec3_t V0, vec3_t V1, vec3_t V2, vec3_t U0, vec3_t U1, vec3_t U2 )`.
    pub fn tri_tri_intersect(
        V0: *const f32,
        V1: *const f32,
        V2: *const f32,
        U0: *const f32,
        U1: *const f32,
        U2: *const f32,
    ) -> c_int;
}

// q_shared.h master networked structs (oracle/q_shared_h_oracle.c): the real C
// `sizeof`/`offsetof`, so the Rust port's layout can be asserted bit-for-bit.
extern "C" {
    pub fn jka_sizeof_trajectory_t() -> usize;
    pub fn jka_sizeof_usercmd_t() -> usize;
    pub fn jka_sizeof_trace_t() -> usize;
    pub fn jka_sizeof_forcedata_t() -> usize;
    pub fn jka_sizeof_entityState_t() -> usize;
    pub fn jka_sizeof_playerState_t() -> usize;

    pub fn jka_off_traj_trBase() -> usize;
    pub fn jka_off_traj_trDelta() -> usize;
    pub fn jka_off_cmd_weapon() -> usize;
    pub fn jka_off_cmd_forwardmove() -> usize;
    pub fn jka_off_trace_plane() -> usize;
    pub fn jka_off_trace_contents() -> usize;
    pub fn jka_off_fd_forcePowerDuration() -> usize;
    pub fn jka_off_fd_killSoundEntIndex() -> usize;
    pub fn jka_off_fd_privateDuelTime() -> usize;
    pub fn jka_off_es_pos() -> usize;
    pub fn jka_off_es_speed() -> usize;
    pub fn jka_off_es_customRGBA() -> usize;
    pub fn jka_off_es_boneAngles1() -> usize;
    pub fn jka_off_es_userVec2() -> usize;
    pub fn jka_off_ps_origin() -> usize;
    pub fn jka_off_ps_stats() -> usize;
    pub fn jka_off_ps_fd() -> usize;
    pub fn jka_off_ps_lastHitLoc() -> usize;
    pub fn jka_off_ps_vehOrientation() -> usize;
    pub fn jka_off_ps_userVec2() -> usize;

    // saber data (embedded by value in gclient_t via saberInfo_t)
    pub fn jka_sizeof_saberTrail_t() -> usize;
    pub fn jka_off_saberTrail_oldPos() -> usize;
    pub fn jka_off_saberTrail_oldNormal() -> usize;
    pub fn jka_sizeof_bladeInfo_t() -> usize;
    pub fn jka_off_bladeInfo_trail() -> usize;
    pub fn jka_off_bladeInfo_hitWallDebounceTime() -> usize;
    pub fn jka_sizeof_saberInfo_t() -> usize;
    pub fn jka_off_saberInfo_blade() -> usize;
    pub fn jka_off_saberInfo_saberFlags() -> usize;
    pub fn jka_off_saberInfo_swingSound() -> usize;
    pub fn jka_off_saberInfo_knockbackScale() -> usize;
    pub fn jka_off_saberInfo_splashKnockback2() -> usize;

    // material_e enumerator checkpoints
    pub fn jka_mat_MAT_METAL() -> c_int;
    pub fn jka_mat_MAT_NONE() -> c_int;
    pub fn jka_mat_MAT_SNOWY_ROCK() -> c_int;
    pub fn jka_mat_NUM_MATERIALS() -> c_int;
}

// bg_lib.c (oracle/bg_lib_oracle.c). Names that collide with the test binary's libc
// are `jka_`-prefixed there; the bodies are the authentic Raven bg_lib.c.
extern "C" {
    /// `void srand( unsigned seed )` (oracle: `jka_srand`).
    pub fn jka_srand(seed: c_uint);
    /// `int rand( void )` — the game's 15-bit LCG (oracle: `jka_rand`).
    pub fn jka_rand() -> c_int;
    /// `void qsort( void* a, size_t n, size_t es, cmp_t* cmp )` (oracle: `jka_qsort`).
    /// `cmp` is C's `cmp_t *`; the Rust [`cmp_t`] alias already denotes that fn pointer.
    pub fn jka_qsort(a: *mut c_void, n: usize, es: usize, cmp: cmp_t);
    /// `double atof( const char *string )` (oracle: `jka_atof`).
    pub fn jka_atof(string: *const c_char) -> f64;
    /// `double _atof( const char **stringPtr )` (oracle: `jka__atof`).
    pub fn jka__atof(string_ptr: *mut *const c_char) -> f64;
    /// `void *memmove( void *dest, const void *src, size_t count )` (oracle: `jka_memmove`).
    pub fn jka_memmove(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;

    // ---- Group B (`#if Q3_VM`): extracted ungated, `jka_`-prefixed, for parity tests
    // run under `--features "oracle vm"`. ----
    /// `size_t strlen( const char *string )` (oracle: `jka_strlen`).
    pub fn jka_strlen(string: *const c_char) -> usize;
    /// `char *strcat( char *dst, const char *src )` (oracle: `jka_strcat`).
    pub fn jka_strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    /// `char *strcpy( char *dst, const char *src )` (oracle: `jka_strcpy`).
    pub fn jka_strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    /// `int strcmp( const char *s1, const char *s2 )` (oracle: `jka_strcmp`).
    pub fn jka_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    /// `char *strchr( const char *string, int c )` (oracle: `jka_strchr`).
    pub fn jka_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    /// `char *strstr( const char *string, const char *strCharSet )` (oracle: `jka_strstr`).
    pub fn jka_strstr(string: *const c_char, str_char_set: *const c_char) -> *mut c_char;
    /// `int tolower( int c )` (oracle: `jka_tolower`).
    pub fn jka_tolower(c: c_int) -> c_int;
    /// `int toupper( int c )` (oracle: `jka_toupper`).
    pub fn jka_toupper(c: c_int) -> c_int;
    /// `int abs( int n )` (oracle: `jka_abs`; TU built `-fwrapv` so `abs(INT_MIN)` wraps).
    pub fn jka_abs(n: c_int) -> c_int;
    /// `double fabs( double x )` (oracle: `jka_fabs`).
    pub fn jka_fabs(x: f64) -> f64;
    /// `double tan( double x )` = `sin(x)/cos(x)` (oracle: `jka_tan`, via libm sin/cos).
    pub fn jka_tan(x: f64) -> f64;
    /// `int atoi( const char *string )` (oracle: `jka_atoi`; `-fwrapv` overflow).
    pub fn jka_atoi(string: *const c_char) -> c_int;
    /// `int _atoi( const char **stringPtr )` (oracle: `jka__atoi`).
    pub fn jka__atoi(string_ptr: *mut *const c_char) -> c_int;
    /// `void AddInt( char **buf_p, int val, int width, int flags )` (oracle: `jka_AddInt`).
    pub fn jka_AddInt(buf_p: *mut *mut c_char, val: c_int, width: c_int, flags: c_int);
    /// `void AddFloat( char **buf_p, float fval, int width, int prec )` (oracle: `jka_AddFloat`).
    pub fn jka_AddFloat(buf_p: *mut *mut c_char, fval: f32, width: c_int, prec: c_int);
    /// `void AddString( char **buf_p, char *string, int width, int prec )` (oracle:
    /// `jka_AddString`; its internal `strlen` is `jka_strlen`).
    pub fn jka_AddString(buf_p: *mut *mut c_char, string: *const c_char, width: c_int, prec: c_int);
    /// `int vsprintf( char *buffer, const char *fmt, va_list )` — oracle `jka_vsprintf` is a
    /// **variadic** test harness around the verbatim format-state machine; the C's broken-on-
    /// 64-bit `va_list`-as-`int*` walk is replaced by `va_arg`, mirroring the Rust port's typed
    /// `VsArg` slice. Dispatches to `jka_AddInt`/`jka_AddFloat`/`jka_AddString` (see
    /// `DEVIATIONS.md`). Declaring/calling a variadic `extern "C"` is stable Rust.
    pub fn jka_vsprintf(buffer: *mut c_char, fmt: *const c_char, ...) -> c_int;
}

// anims.h (oracle/anims_oracle.c): the authentic `animNumber_t` enum, read by the
// C compiler from the unmodified Raven header. Each accessor returns one enumerator
// value (a spread of checkpoints + the two terminal counts), plus the derived
// SABER_ANIM_GROUP_SIZE macro, so the generated Rust consts can be asserted bit-exact.
extern "C" {
    pub fn jka_anim_FACE_TALK0() -> c_int;
    pub fn jka_anim_BOTH_ATTACK10() -> c_int;
    pub fn jka_anim_BOTH_A1_T__B_() -> c_int;
    pub fn jka_anim_BOTH_A2_T__B_() -> c_int;
    pub fn jka_anim_BOTH_T2__R_T_() -> c_int;
    pub fn jka_anim_BOTH_A4_TL_BR() -> c_int;
    pub fn jka_anim_BOTH_T5__L_BL() -> c_int;
    pub fn jka_anim_BOTH_T7__R_TL() -> c_int;
    pub fn jka_anim_BOTH_K7_S7_T_() -> c_int;
    pub fn jka_anim_BOTH_BF1BREAK() -> c_int;
    pub fn jka_anim_BOTH_CONSOLE2HOLDCOMSTOP() -> c_int;
    pub fn jka_anim_BOTH_VT_IDLE_SR() -> c_int;
    pub fn jka_anim_BOTH_VICTORY_STAFF() -> c_int;
    pub fn jka_anim_BOTH_CHOKE1() -> c_int;
    pub fn jka_anim_LEGS_S1_RUP4() -> c_int;
    pub fn jka_anim_BOTH_CIN_50() -> c_int;
    pub fn jka_anim_MAX_ANIMATIONS() -> c_int;
    pub fn jka_anim_MAX_TOTALANIMATIONS() -> c_int;
    pub fn jka_SABER_ANIM_GROUP_SIZE() -> c_int;
}

// bg_public.h (oracle/bg_public_oracle.c): verbatim copies of bg_public.h
// definitions over the real external limit values, exposing the C compiler's
// computed results. First slice: the computed CS_* config-string chain.
extern "C" {
    pub fn jka_bgp_CS_SIEGE_STATE() -> c_int;
    pub fn jka_bgp_CS_MODELS() -> c_int;
    pub fn jka_bgp_CS_ICONS() -> c_int;
    pub fn jka_bgp_CS_LIGHT_STYLES() -> c_int;
    pub fn jka_bgp_CS_TERRAINS() -> c_int;
    pub fn jka_bgp_CS_BSP_MODELS() -> c_int;
    pub fn jka_bgp_CS_MAX() -> c_int;
    /// `BG_GiveMeVectorFromMatrix` driver: `m12` is the 3×4 bolt matrix in
    /// row-major order, `out` receives the selected `vec3_t`.
    pub fn jka_bgp_GiveMeVectorFromMatrix(m12: *const f32, flags: c_int, out: *mut f32);
}

// bg_public.h shared enums (oracle/bg_public_enums_oracle.c): each enum body is
// copied verbatim so the C compiler numbers it; these accessors return a spread of
// checkpoints (first / terminal / every explicit `= N` / interior points) per enum.
extern "C" {
    pub fn jka_bge_G2_MODELPART_HEAD() -> c_int;
    pub fn jka_bge_G2_MODELPART_RLEG() -> c_int;
    pub fn jka_bge_HANDEXTEND_NONE() -> c_int;
    pub fn jka_bge_HANDEXTEND_JEDITAUNT() -> c_int;
    pub fn jka_bge_BROKENLIMB_NONE() -> c_int;
    pub fn jka_bge_NUM_BROKENLIMBS() -> c_int;
    pub fn jka_bge_GT_FFA() -> c_int;
    pub fn jka_bge_GT_MAX_GAME_TYPE() -> c_int;
    pub fn jka_bge_GENDER_MALE() -> c_int;
    pub fn jka_bge_GENDER_NEUTER() -> c_int;
    pub fn jka_bge_SABERLOCK_TOP() -> c_int;
    pub fn jka_bge_SABERLOCK_LOSE() -> c_int;
    pub fn jka_bge_DIR_RIGHT() -> c_int;
    pub fn jka_bge_DIR_BACK() -> c_int;
    pub fn jka_bge_FOOTSTEP_R() -> c_int;
    pub fn jka_bge_NUM_FOOTSTEP_TYPES() -> c_int;
    pub fn jka_bge_AEV_NONE() -> c_int;
    pub fn jka_bge_AEV_NUM_AEV() -> c_int;
    pub fn jka_bge_PM_NORMAL() -> c_int;
    pub fn jka_bge_PM_SPINTERMISSION() -> c_int;
    pub fn jka_bge_WEAPON_READY() -> c_int;
    pub fn jka_bge_WEAPON_IDLE() -> c_int;
    pub fn jka_bge_FORCE_MASTERY_UNINITIATED() -> c_int;
    pub fn jka_bge_NUM_FORCE_MASTERY_LEVELS() -> c_int;
    pub fn jka_bge_STAT_HEALTH() -> c_int;
    pub fn jka_bge_STAT_WEAPONS() -> c_int;
    pub fn jka_bge_STAT_MAX_HEALTH() -> c_int;
    pub fn jka_bge_PERS_SCORE() -> c_int;
    pub fn jka_bge_PERS_CAPTURES() -> c_int;
    pub fn jka_bge_EFFECT_NONE() -> c_int;
    pub fn jka_bge_EFFECT_MAX() -> c_int;
    pub fn jka_bge_PW_NONE() -> c_int;
    pub fn jka_bge_PW_NUM_POWERUPS() -> c_int;
    pub fn jka_bge_HI_NONE() -> c_int;
    pub fn jka_bge_HI_NUM_HOLDABLE() -> c_int;
    pub fn jka_bge_CTFMESSAGE_FRAGGED_FLAG_CARRIER() -> c_int;
    pub fn jka_bge_CTFMESSAGE_PLAYER_GOT_FLAG() -> c_int;
    pub fn jka_bge_PDSOUND_NONE() -> c_int;
    pub fn jka_bge_PDSOUND_FORCEGRIP() -> c_int;
    pub fn jka_bge_EV_NONE() -> c_int;
    pub fn jka_bge_EV_USE_ITEM5() -> c_int;
    pub fn jka_bge_EV_BODYFADE() -> c_int;
    pub fn jka_bge_EV_ESCAPING2() -> c_int;
    pub fn jka_bge_EV_SIEGESPEC() -> c_int;
    pub fn jka_bge_GTS_RED_CAPTURE() -> c_int;
    pub fn jka_bge_GTS_TEAMS_ARE_TIED() -> c_int;
    pub fn jka_bge_TEAM_FREE() -> c_int;
    pub fn jka_bge_TEAM_NUM_TEAMS() -> c_int;
    pub fn jka_bge_DUELTEAM_FREE() -> c_int;
    pub fn jka_bge_DUELTEAM_SINGLE() -> c_int;
    pub fn jka_bge_TEAMTASK_NONE() -> c_int;
    pub fn jka_bge_TEAMTASK_CAMP() -> c_int;
    pub fn jka_bge_MOD_UNKNOWN() -> c_int;
    pub fn jka_bge_MOD_MAX() -> c_int;
    pub fn jka_bge_IT_BAD() -> c_int;
    pub fn jka_bge_IT_TEAM() -> c_int;
    pub fn jka_bge_ET_GENERAL() -> c_int;
    pub fn jka_bge_ET_EVENTS() -> c_int;
    pub fn jka_bge_F_INT() -> c_int;
    pub fn jka_bge_F_IGNORE() -> c_int;
    pub fn jka_bge_LS_NONE() -> c_int;
    pub fn jka_bge_LS_DUAL_SPIN_PROTECT() -> c_int;
    pub fn jka_bge_LS_T1_TL_BR() -> c_int;
    pub fn jka_bge_LS_K1_BR() -> c_int;
    pub fn jka_bge_LS_MOVE_MAX() -> c_int;
    pub fn jka_bge_Q_BR() -> c_int;
    pub fn jka_bge_Q_NUM_QUADS() -> c_int;
}

// bg_public.h structs (oracle/bg_public_structs_oracle.c): verbatim struct copies
// over the real dependent consts, exposing the C `sizeof`/`offsetof`. `animation_t`
// is arch-independent; the rest are validated at the host (64-bit) layout.
extern "C" {
    pub fn jka_bgs_sizeof_animation_t() -> usize;
    pub fn jka_bgs_off_animation_loopFrames() -> usize;
    pub fn jka_bgs_sizeof_animevent_t() -> usize;
    pub fn jka_bgs_off_animevent_eventData() -> usize;
    pub fn jka_bgs_off_animevent_stringData() -> usize;
    pub fn jka_bgs_sizeof_bgLoadedAnim_t() -> usize;
    pub fn jka_bgs_off_bgLoadedAnim_anims() -> usize;
    pub fn jka_bgs_sizeof_bgLoadedEvents_t() -> usize;
    pub fn jka_bgs_off_bgLoadedEvents_legsAnimEvents() -> usize;
    pub fn jka_bgs_off_bgLoadedEvents_eventsParsed() -> usize;
    pub fn jka_bgs_sizeof_gitem_t() -> usize;
    pub fn jka_bgs_off_gitem_world_model() -> usize;
    pub fn jka_bgs_off_gitem_quantity() -> usize;
    pub fn jka_bgs_off_gitem_precaches() -> usize;
    pub fn jka_bgs_sizeof_saberMoveData_t() -> usize;
    pub fn jka_bgs_off_saberMoveData_chain_idle() -> usize;
    pub fn jka_bgs_off_saberMoveData_trailLength() -> usize;
    pub fn jka_bgs_sizeof_BG_field_t() -> usize;
    pub fn jka_bgs_off_BG_field_type() -> usize;
}

// g_public.h data types (oracle/g_public_h_oracle.c): the data half of g_public.h
// (entityShared_t embedded by value in gentity_t, parms_t, failedEdge_t) plus the
// ICARUS NUM_TIDS/NUM_BSETS enum terminals that size gentity_t arrays.
extern "C" {
    pub fn jka_gp_sizeof_failedEdge_t() -> usize;
    pub fn jka_gp_sizeof_entityShared_t() -> usize;
    pub fn jka_gp_off_es_mins() -> usize;
    pub fn jka_gp_off_es_currentOrigin() -> usize;
    pub fn jka_gp_off_es_broadcastClients() -> usize;
    pub fn jka_gp_sizeof_parms_t() -> usize;
    pub fn jka_gp_NUM_TIDS() -> c_int;
    pub fn jka_gp_NUM_BSETS() -> c_int;
    pub fn jka_gp_BSET_INVALID() -> c_int;
    pub fn jka_gp_BSET_MINDTRICK() -> c_int;
}

// teams.h enums (oracle/teams_h_oracle.c): npcteam_t / class_t embedded by value
// in gclient_t. Verbatim enum bodies; checkpoint accessors.
extern "C" {
    pub fn jka_teams_NPCTEAM_FREE() -> c_int;
    pub fn jka_teams_NPCTEAM_NUM_TEAMS() -> c_int;
    pub fn jka_teams_CLASS_NONE() -> c_int;
    pub fn jka_teams_CLASS_GONK() -> c_int;
    pub fn jka_teams_CLASS_GALAKMECH() -> c_int;
    pub fn jka_teams_CLASS_BOBAFETT() -> c_int;
    pub fn jka_teams_CLASS_WAMPA() -> c_int;
    pub fn jka_teams_CLASS_NUM_CLASSES() -> c_int;
}

// b_public.h slice (oracle/b_public_h_oracle.c): lookMode_t (embedded by value in
// renderInfo_t) + the real gNPC_t/gNPCstats_t NPC runtime structs. gNPC_t carries
// gentity_t*/AIGroupInfo_t* pointers => arch-dependent, validated at host 64-bit.
extern "C" {
    pub fn jka_bp_LM_ENT() -> c_int;
    pub fn jka_bp_LM_INTEREST() -> c_int;
    pub fn jka_bp_sizeof_gNPCstats_t() -> usize;
    pub fn jka_bp_off_stats_aggression() -> usize;
    pub fn jka_bp_off_stats_runSpeed() -> usize;
    pub fn jka_bp_off_stats_acceleration() -> usize;
    pub fn jka_bp_sizeof_gNPC_t() -> usize;
    pub fn jka_bp_off_npc_timeOfDeath() -> usize;
    pub fn jka_bp_off_npc_stats() -> usize;
    pub fn jka_bp_off_npc_last_ucmd() -> usize;
    pub fn jka_bp_off_npc_ffireFadeDebounce() -> usize;
}

// g_local.h masters (oracle/g_local_oracle.c): gentity_t/gclient_t + the local
// structs they embed. Pointer-bearing => validated at the host 64-bit layout.
extern "C" {
    pub fn jka_gl_HL_MAX() -> c_int;
    pub fn jka_gl_sizeof_playerTeamState_t() -> usize;
    pub fn jka_gl_sizeof_clientSession_t() -> usize;
    pub fn jka_gl_sizeof_clientPersistant_t() -> usize;
    pub fn jka_gl_off_cp_netname() -> usize;
    pub fn jka_gl_off_cp_teamState() -> usize;
    pub fn jka_gl_sizeof_renderInfo_t() -> usize;
    pub fn jka_gl_off_ri_lookMode() -> usize;
    pub fn jka_gl_off_ri_lastG2() -> usize;
    pub fn jka_gl_sizeof_gentity_t() -> usize;
    pub fn jka_gl_off_ent_r() -> usize;
    pub fn jka_gl_off_ent_taskID() -> usize;
    pub fn jka_gl_off_ent_client() -> usize;
    pub fn jka_gl_off_ent_moverState() -> usize;
    pub fn jka_gl_off_ent_think() -> usize;
    pub fn jka_gl_off_ent_material() -> usize;
    pub fn jka_gl_off_ent_locationDamage() -> usize;
    pub fn jka_gl_off_ent_item() -> usize;
    pub fn jka_gl_sizeof_gclient_t() -> usize;
    pub fn jka_gl_off_cl_pers() -> usize;
    pub fn jka_gl_off_cl_sess() -> usize;
    pub fn jka_gl_off_cl_saber() -> usize;
    pub fn jka_gl_off_cl_renderInfo() -> usize;
    pub fn jka_gl_off_cl_NPC_class() -> usize;
    pub fn jka_gl_off_cl_lastGenCmdTime() -> usize;

    // level_locals_t + its support types
    pub fn jka_gl_sizeof_combatPoint_t() -> usize;
    pub fn jka_gl_sizeof_waypointData_t() -> usize;
    pub fn jka_gl_sizeof_reference_tag_t() -> usize;
    pub fn jka_gl_sizeof_bot_settings_t() -> usize;
    pub fn jka_gl_sizeof_interestPoint_t() -> usize;
    pub fn jka_gl_sizeof_alertEvent_t() -> usize;
    pub fn jka_gl_off_ae_owner() -> usize;
    pub fn jka_gl_sizeof_level_locals_t() -> usize;
    pub fn jka_gl_off_ll_groups() -> usize;
    pub fn jka_gl_off_ll_combatPoints() -> usize;
    pub fn jka_gl_off_ll_mTeamFilter() -> usize;
}

// g_spawn.c (oracle/g_spawn_oracle.c): the spawn-string allocation helper. The C body
// is verbatim except `G_Alloc` -> `malloc` (output bytes are allocator-independent).
extern "C" {
    /// `char *G_NewString( const char *string )` (oracle: `jka_G_NewString`).
    pub fn jka_G_NewString(string: *const c_char) -> *mut c_char;
}

// g_utils.c (oracle/g_utils_oracle.c): the pure vector/bounds helpers. Verbatim; the
// generic `G_SetMovedir` path links the real `VectorCompare`/`AngleVectors` from
// q_math_oracle.c. `vec3_t` is `float[3]`, decaying to `float*`.
extern "C" {
    /// `void G_SetMovedir( vec3_t angles, vec3_t movedir )` (oracle: `jka_G_SetMovedir`).
    pub fn jka_G_SetMovedir(angles: *mut f32, movedir: *mut f32);
    /// `qboolean G_PointInBounds( vec3_t point, vec3_t mins, vec3_t maxs )`.
    pub fn jka_G_PointInBounds(point: *const f32, mins: *const f32, maxs: *const f32) -> c_int;
    /// `qboolean G_BoxInBounds( vec3_t point, vec3_t mins, vec3_t maxs, vec3_t boundsMins,
    /// vec3_t boundsMaxs )`.
    pub fn jka_G_BoxInBounds(
        point: *const f32,
        mins: *const f32,
        maxs: *const f32,
        boundsMins: *const f32,
        boundsMaxs: *const f32,
    ) -> c_int;

    // The shader-remap subsystem over the oracle's own globals; the real Com_sprintf/
    // Q_strcat/Q_stricmp are linked from q_shared_oracle.c. jka_ResetRemaps clears the
    // table so the parity test can replay a deterministic sequence from empty.
    /// `void AddRemap( const char *oldShader, const char *newShader, float timeOffset )`.
    pub fn jka_AddRemap(oldShader: *const c_char, newShader: *const c_char, timeOffset: f32);
    /// `const char *BuildShaderStateConfig( void )`.
    pub fn jka_BuildShaderStateConfig() -> *const c_char;
    /// Test helper — zero `remapCount` / `remappedShaders` in the oracle TU.
    pub fn jka_ResetRemaps();

    /// `void GetAnglesForDirection( const vec3_t p1, const vec3_t p2, vec3_t out )` —
    /// links the real `vectoangles` from q_math_oracle.c.
    pub fn jka_GetAnglesForDirection(p1: *const f32, p2: *const f32, out: *mut f32);

    /// `float ShortestLineSegBewteen2LineSegs( vec3_t start1, vec3_t end1, vec3_t start2,
    /// vec3_t end2, vec3_t close_pnt1, vec3_t close_pnt2 )` — links the real `Distance` /
    /// `G_FindClosestPointOnLineSegment` from q_math_oracle.c.
    pub fn jka_ShortestLineSegBewteen2LineSegs(
        start1: *const f32,
        end1: *const f32,
        start2: *const f32,
        end2: *const f32,
        close_pnt1: *mut f32,
        close_pnt2: *mut f32,
    ) -> f32;
}

// bg_misc.c (oracle/bg_misc_parsefield_oracle.c): the spawn-field setter. Verbatim except
// G_Alloc -> malloc inside the F_LSTRING allocator and the deferred F_PARM* -> Q3_SetParm
// path omitted (both sides treat F_PARM* as no-ops).
extern "C" {
    /// `void BG_ParseField( BG_field_t *l_fields, const char *key, const char *value,
    /// byte *ent )` (oracle: `jka_BG_ParseField`).
    pub fn jka_BG_ParseField(
        l_fields: *mut BG_field_t,
        key: *const c_char,
        value: *const c_char,
        ent: *mut u8,
    );
}

// bg_vehicleLoad.c (oracle/bg_vehicleLoad_oracle.c): the .vwp field setter. Verbatim
// (QAGAME build) except BG_Alloc -> malloc inside the VF_LSTRING allocator (output bytes
// depend only on the strcpy, not the pool address) and the engine-configstring arms
// (VF_MODEL/EFFECT/SOUND/VEHTYPE/ANIM) omitted -- never reached by vehWeaponFields[].
extern "C" {
    /// `qboolean BG_ParseVehWeaponParm( vehWeaponInfo_t *vehWeapon, char *parmName,
    /// char *pValue )` (oracle: `jka_BG_ParseVehWeaponParm`). `vehWeapon` taken as a raw
    /// byte pointer (identical `vehWeaponInfo_t` layout both sides).
    pub fn jka_BG_ParseVehWeaponParm(
        vehWeapon: *mut u8,
        parmName: *const c_char,
        pValue: *const c_char,
    ) -> c_int;
}

// ai.h group-AI types (oracle/ai_h_oracle.c): AIGroupInfo_t embedded by value in
// level_locals_t. Pointer-bearing => validated at the host 64-bit layout.
extern "C" {
    pub fn jka_ai_NUM_SQUAD_STATES() -> c_int;
    pub fn jka_ai_sizeof_AIGroupMember_t() -> usize;
    pub fn jka_ai_sizeof_AIGroupInfo_t() -> usize;
    pub fn jka_ai_off_enemy() -> usize;
    pub fn jka_ai_off_commander() -> usize;
    pub fn jka_ai_off_numState() -> usize;
    pub fn jka_ai_off_member() -> usize;
}

// g_main.c cvar table (oracle/g_main_oracle.c): the real C gameCvarTable's size
// and each row's data fields, for parity against the Rust port's hand-built table.
extern "C" {
    pub fn jka_gameCvarTableSize() -> c_int;
    pub fn jka_cvar_name(i: c_int) -> *const c_char;
    pub fn jka_cvar_default(i: c_int) -> *const c_char;
    pub fn jka_cvar_flags(i: c_int) -> c_int;
    pub fn jka_cvar_modcount(i: c_int) -> c_int;
    pub fn jka_cvar_track(i: c_int) -> c_int;
    pub fn jka_cvar_team(i: c_int) -> c_int;
}

// g_main.c:4217 G_GetStringEdString (oracle/g_main_oracle.c): verbatim body that
// emits the "@@@<refName>" stringed-reference token into a static buffer.
extern "C" {
    pub fn jka_G_GetStringEdString(
        ref_section: *mut c_char,
        ref_name: *mut c_char,
    ) -> *const c_char;
}

// g_svcmds.c (oracle/g_svcmds_oracle.c): StringToFilter, the dotted-IP address
// parser for the packet / IP-ban filter list. The wrapper runs it and returns its
// mask / compare / return value for the Rust parity test.
extern "C" {
    pub fn jka_StringToFilter(s: *const c_char, mask: *mut c_uint, compare: *mut c_uint) -> c_int;
    // G_FilterPacket: the connect-time allow/deny check. The wrapper seeds the
    // filter list / filterBan, then runs the verbatim C body and returns its
    // qboolean for the Rust parity test.
    pub fn jka_G_FilterPacket(
        from: *const c_char,
        count: c_int,
        masks: *const c_uint,
        compares: *const c_uint,
        filter_ban: c_int,
    ) -> c_int;
}

// bg_vehicles.h structs (oracle/bg_vehicles_h_oracle.c): the vehicle data layer.
// Pointer-bearing (vehWeaponInfo_t/turretStats_t carry char*) => validated at the
// host 64-bit layout; the pointer-free status/stats structs are arch-independent.
extern "C" {
    pub fn jka_bv_sizeof_vehWeaponInfo_t() -> usize;
    pub fn jka_bv_off_vwi_bIsProjectile() -> usize;
    pub fn jka_bv_sizeof_turretStats_t() -> usize;
    pub fn jka_bv_off_ts_yawBone() -> usize;
    pub fn jka_bv_off_ts_iMuzzle() -> usize;
    pub fn jka_bv_off_ts_gunnerViewTag() -> usize;
    pub fn jka_bv_sizeof_vehWeaponStats_t() -> usize;
    pub fn jka_bv_sizeof_vehWeaponStatus_t() -> usize;
    pub fn jka_bv_sizeof_vehTurretStatus_t() -> usize;

    // The pointer cluster (vehicleInfo_t / Vehicle_t / bgEntity_t). bgEntity_t is a
    // bg_public.h type but shares the cluster's by-value deps, so its accessor lives
    // in this TU (the Rust test for it is in `bg_public`, calling these `jka_bv_*`).
    pub fn jka_bv_sizeof_vehicleInfo_t() -> usize;
    pub fn jka_bv_off_vi_type() -> usize;
    pub fn jka_bv_off_vi_model() -> usize;
    pub fn jka_bv_off_vi_weapon() -> usize;
    pub fn jka_bv_off_vi_turret() -> usize;
    pub fn jka_bv_off_vi_modelIndex() -> usize;
    pub fn jka_bv_off_vi_AnimateVehicle() -> usize;
    pub fn jka_bv_sizeof_bgEntity_t() -> usize;
    pub fn jka_bv_off_be_playerState() -> usize;
    pub fn jka_bv_off_be_m_pVehicle() -> usize;
    pub fn jka_bv_off_be_modelScale() -> usize;
    pub fn jka_bv_sizeof_Vehicle_t() -> usize;
    pub fn jka_bv_off_veh_m_ucmd() -> usize;
    pub fn jka_bv_off_veh_m_ulFlags() -> usize;
    pub fn jka_bv_off_veh_m_vOrientation() -> usize;
    pub fn jka_bv_off_veh_m_pVehicleInfo() -> usize;
    pub fn jka_bv_off_veh_m_LandTrace() -> usize;
    pub fn jka_bv_off_veh_weaponStatus() -> usize;
    pub fn jka_bv_off_veh_turretStatus() -> usize;
    pub fn jka_bv_off_veh_m_pOldPilot() -> usize;

    // pmove_t (bg_public.h) -- references bgEntity_t + shares usercmd_t/trace_t, so its
    // oracle is in this TU too (the Rust test is in `bg_public`).
    pub fn jka_bv_sizeof_pmove_t() -> usize;
    pub fn jka_bv_off_pm_cmd() -> usize;
    pub fn jka_bv_off_pm_mins() -> usize;
    pub fn jka_bv_off_pm_animations() -> usize;
    pub fn jka_bv_off_pm_trace() -> usize;
    pub fn jka_bv_off_pm_pointcontents() -> usize;
    pub fn jka_bv_off_pm_baseEnt() -> usize;
    pub fn jka_bv_off_pm_entSize() -> usize;

    // vehicleFields[] offset column (bg_vehicleLoad.c, _JK2MP config); independent C
    // transcription, compared element-wise by the `bg_vehicleLoad` Rust test.
    pub fn jka_vehicleFields_count() -> c_int;
    pub fn jka_vehicleFields_offsets() -> *const c_int;

    /// `BG_VehicleClampData( vehicleInfo_t *vehicle )` (bg_vehicleLoad.c:821), verbatim
    /// over the real `vehicleInfo_t`; taken as a raw byte pointer (identical layout both
    /// sides). The `bg_vehicleLoad` Rust test runs both sides on the same struct bytes.
    pub fn jka_BG_VehicleClampData(vehicle: *mut u8);
}

// bg_weapons.h (oracle/bg_weapons_h_oracle.c): `#include`s the authentic header, so
// the `weaponData_t`/`ammoData_t` `sizeof` and the `weapon_t`/`ammo_t` enum terminals
// are read from the real Raven source. Pointer-free => arch-independent.
extern "C" {
    pub fn jka_bw_sizeof_weaponData_t() -> usize;
    pub fn jka_bw_sizeof_ammoData_t() -> usize;
    pub fn jka_bw_WP_NONE() -> c_int;
    pub fn jka_bw_WP_NUM_WEAPONS() -> c_int;
    pub fn jka_bw_LAST_USEABLE_WEAPON() -> c_int;
    pub fn jka_bw_AMMO_NONE() -> c_int;
    pub fn jka_bw_AMMO_MAX() -> c_int;
}

// bg_weapons.c value tables (oracle/bg_weapons_oracle.c): the table DATA independently
// transcribed; the Rust test compares element-wise against the Rust port (catching a
// single-value typo on either side).
extern "C" {
    pub fn jka_bw_muzzle_ptr() -> *const [f32; 3];
    pub fn jka_bw_weaponData_ptr() -> *const weaponData_t;
    pub fn jka_bw_ammoData_ptr() -> *const ammoData_t;
}

// bg_misc.c logic (oracle/bg_misc_oracle.c): BG_AddPredictableEventToPlayerstate run
// over a minimal playerState_t; the wrapper loads/reads the event-ring state by value.
extern "C" {
    pub fn jka_bg_add_pred_event(
        newEvent: c_int,
        eventParm: c_int,
        io_eventSequence: *mut c_int,
        io_events: *mut c_int,
        io_eventParms: *mut c_int,
    );
    /// `float vectoyaw( const vec3_t vec )` — direction → yaw, driven by 3 floats.
    pub fn jka_bg_vectoyaw(x: f32, y: f32, z: f32) -> f32;
    /// `BG_EvaluateTrajectory` — trajectory fields marshalled in positionally, the
    /// sampled `vec3_t` written through `out`.
    pub fn jka_bg_eval_traj(
        trType: c_int,
        trTime: c_int,
        trDuration: c_int,
        bx: f32,
        by: f32,
        bz: f32,
        dx: f32,
        dy: f32,
        dz: f32,
        atTime: c_int,
        out: *mut f32,
    );
    /// `BG_GiveMeVectorFromMatrix` — the 12 `mdxaBone_t` matrix floats (row-major
    /// `matrix[3][4]`) + the axis flag marshalled in; the selected vec3 written through
    /// `out` (which the caller pre-seeds so the no-match untouched path is observable).
    pub fn jka_bg_give_me_vector_from_matrix(m12: *const f32, flags: c_int, out: *mut f32);
    /// `BG_EvaluateTrajectoryDelta` — same marshalling, velocity written through `out`.
    pub fn jka_bg_eval_traj_delta(
        trType: c_int,
        trTime: c_int,
        trDuration: c_int,
        bx: f32,
        by: f32,
        bz: f32,
        dx: f32,
        dy: f32,
        dz: f32,
        atTime: c_int,
        out: *mut f32,
    );
    /// `BG_PlayerTouchesItem` — player origin (`px..pz`) + the item's pos trajectory
    /// marshalled in positionally; returns the qboolean (1/0) pickup-proximity result.
    pub fn jka_bg_player_touches_item(
        px: f32,
        py: f32,
        pz: f32,
        trType: c_int,
        trTime: c_int,
        trDuration: c_int,
        bx: f32,
        by: f32,
        bz: f32,
        dx: f32,
        dy: f32,
        dz: f32,
        atTime: c_int,
    ) -> c_int;
    /// `BG_TouchJumpPad` — read ps/es fields marshalled in positionally (`jumppad_frame`
    /// seeded to 0 C-side); the three mutated outputs (`jumppad_ent`, `jumppad_frame`,
    /// and the 3-float `velocity`) are written back through the out-pointers.
    pub fn jka_bg_touch_jump_pad(
        pm_type: c_int,
        in_jumppad_ent: c_int,
        number: c_int,
        pmove_framecount: c_int,
        o2x: f32,
        o2y: f32,
        o2z: f32,
        out_jumppad_ent: *mut c_int,
        out_jumppad_frame: *mut c_int,
        out_velocity: *mut f32,
    );
    /// `BG_IsValidCharacterModel` — the two model/skin C strings passed straight through;
    /// returns the qboolean (1/0) validity result.
    pub fn jka_bg_is_valid_character_model(
        modelName: *const c_char,
        skinName: *const c_char,
    ) -> c_int;
    /// `BG_PlayerStateToEntityState` — ps→es field collapse. Inputs/outputs cross as flat
    /// arrays (never the full structs); the index maps in `bg_misc_oracle.c` MUST match the
    /// packing/reading in the Rust parity test. `out_entityEventSequence` returns the one
    /// mutated `ps` field (the event-latch advance).
    pub fn jka_bg_player_state_to_entity_state(
        in_i: *const c_int,
        in_f: *const f32,
        in_speed: f32,
        in_events: *const c_int,
        in_eventParms: *const c_int,
        in_powerups: *const c_int,
        in_customRGBA: *const c_int,
        snap: c_int,
        out_i: *mut c_int,
        out_f: *mut f32,
        out_customRGBA: *mut c_int,
        out_entityEventSequence: *mut c_int,
    );
    /// `BG_PlayerStateToEntityStateExtraPolate` — the extrapolating sibling. Same flat-array
    /// marshalling as the plain variant, plus a `time` input; `out_i[44]`/`out_i[45]` carry
    /// the extra `pos.trTime`/`pos.trDuration` it stamps, `out_i[46]` carries `isJediMaster`
    /// (and `out_i[2]` reports `TR_LINEAR_STOP`). Index maps in `bg_misc_oracle.c` MUST match
    /// the Rust parity test.
    pub fn jka_bg_player_state_to_entity_state_extrapolate(
        in_i: *const c_int,
        in_f: *const f32,
        in_speed: f32,
        in_events: *const c_int,
        in_eventParms: *const c_int,
        in_powerups: *const c_int,
        in_customRGBA: *const c_int,
        time: c_int,
        snap: c_int,
        out_i: *mut c_int,
        out_f: *mut f32,
        out_customRGBA: *mut c_int,
        out_entityEventSequence: *mut c_int,
    );
}

// bg_misc.c logic (oracle/bg_misc_oracle.c): the force-power cycle pair. ProperForceIndex
// returns the sorted-table slot for a power (or -1); CycleForce loads the player's
// forcePowersKnown/forcePowerSelected by value, runs the body, and returns the resulting
// forcePowerSelected.
extern "C" {
    pub fn jka_bg_proper_force_index(power: c_int) -> c_int;
    pub fn jka_bg_cycle_force(
        forcePowersKnown: c_int,
        forcePowerSelected: c_int,
        direction: c_int,
    ) -> c_int;
}

// bg_misc.c logic (oracle/bg_misc_oracle.c): BG_EmplacedView, the emplaced-gun yaw
// constriction. Only the `[YAW]` component of each angle is read, so the wrapper passes
// the two yaw scalars + constraint; it returns the int range code and writes the clamped
// yaw into `*out_newYaw` (untouched on a `0` return).
extern "C" {
    pub fn jka_bg_emplaced_view(
        baseYaw: f32,
        angleYaw: f32,
        constraint: f32,
        out_newYaw: *mut f32,
    ) -> c_int;
}

// bg_misc.c logic (oracle/bg_misc_oracle.c): the two force-power gate predicates
// BG_HasYsalamiri / BG_CanUseFPNow, run over a minimal playerState_t; each wrapper
// loads the read fields by value (positional ints) and returns the qboolean result.
extern "C" {
    pub fn jka_bg_has_ysalamiri(
        gametype: c_int,
        pw_red: c_int,
        pw_blue: c_int,
        pw_ysa: c_int,
    ) -> c_int;
    pub fn jka_bg_can_use_fp_now(
        gametype: c_int,
        time: c_int,
        power: c_int,
        pw_red: c_int,
        pw_blue: c_int,
        pw_ysa: c_int,
        forceRestricted: c_int,
        trueNonJedi: c_int,
        weapon: c_int,
        m_iVehicleNum: c_int,
        duelInProgress: c_int,
        saberLockFrame: c_int,
        saberLockTime: c_int,
        fallingToDeath: c_int,
        brokenLimbs: c_int,
    ) -> c_int;
}

// bg_misc.c logic (oracle/bg_misc_oracle.c): BG_LegalizedForcePowers, the force-config
// string legalizer. `io_powerOut` is the in/out config buffer (128 bytes); the wrapper
// rewrites it in place and returns the qboolean (1/0).
extern "C" {
    pub fn jka_bg_legalize_force_powers(
        io_powerOut: *mut c_char,
        maxRank: c_int,
        freeSaber: c_int,
        teamForce: c_int,
        gametype: c_int,
        fpDisabled: c_int,
    ) -> c_int;
}

// bg_misc.c data tables (oracle/bg_misc_oracle.c): verbatim copies of the four
// file-scope tables (bg_misc.c:34/78/113/150). Accessors hand back the base pointer
// (a `*const *const c_char` for the string tables, `*const c_int` for the debris ints);
// the Rust parity tests walk each index against the bg_misc.rs statics.
extern "C" {
    pub fn jka_bg_toggleable_surfaces_ptr() -> *const *const c_char;
    pub fn jka_bg_toggleable_surface_debris_ptr() -> *const c_int;
    pub fn jka_bg_custom_siege_sound_names_ptr() -> *const *const c_char;
    pub fn jka_bg_force_mastery_levels_ptr() -> *const *const c_char;
}

// bg_saga.c parsing helpers (oracle/bg_saga_oracle.c): verbatim copies of the
// self-contained, pointer-free text helpers. The Rust port in bg_saga.rs is asserted
// byte-for-byte against these.
extern "C" {
    pub fn jka_BG_SiegeStripTabs(buf: *mut c_char);
    pub fn jka_BG_SiegeGetValueGroup(
        buf: *mut c_char,
        group: *mut c_char,
        outbuf: *mut c_char,
    ) -> c_int;
    pub fn jka_BG_SiegeGetPairedValue(
        buf: *mut c_char,
        key: *mut c_char,
        outbuf: *mut c_char,
    ) -> c_int;
    pub fn jka_BG_SiegeTranslateForcePowers(buf: *mut c_char, siegeClass: *mut siegeClass_t);
    pub fn jka_BG_SiegeTranslateGenericTable(
        buf: *mut c_char,
        table: *const stringID_table_t,
        bitflag: c_int,
    ) -> c_int;
    pub fn jka_BG_SiegeFindThemeForTeam(
        team: c_int,
        t1: *mut siegeTeam_t,
        t2: *mut siegeTeam_t,
    ) -> *mut siegeTeam_t;
    pub fn jka_BG_SiegeCountBaseClass(stm: *mut siegeTeam_t, classIndex: c_short) -> c_int;
    pub fn jka_BG_GetUIPortraitFile(
        stm: *mut siegeTeam_t,
        classIndex: c_short,
        cntIndex: c_short,
    ) -> *mut c_char;
    pub fn jka_BG_GetUIPortrait(
        stm: *mut siegeTeam_t,
        classIndex: c_short,
        cntIndex: c_short,
    ) -> c_int;
    pub fn jka_BG_GetClassOnBaseClass(
        stm: *mut siegeTeam_t,
        classIndex: c_short,
        cntIndex: c_short,
    ) -> *mut siegeClass_t;
    pub fn jka_BG_SiegeFindClassByName(
        classname: *const c_char,
        classes: *mut siegeClass_t,
        num: c_int,
    ) -> *mut siegeClass_t;
    pub fn jka_BG_SiegeFindClassIndexByName(
        classname: *const c_char,
        classes: *mut siegeClass_t,
        num: c_int,
    ) -> c_int;
    pub fn jka_BG_SiegeFindTeamForTheme(
        themeName: *mut c_char,
        teams: *mut siegeTeam_t,
        num: c_int,
    ) -> *mut siegeTeam_t;
    pub fn jka_BG_SiegeCheckClassLegality(
        team: c_int,
        classname: *mut c_char,
        t1: *mut siegeTeam_t,
        t2: *mut siegeTeam_t,
    ) -> c_int;
}

// bg_misc.c item system (oracle/bg_misc_items_oracle.c): the verbatim bg_itemlist[]
// table + bg_numItems, plus the selectors/finders run over a minimal playerState_t.
// Finders return the bg_itemlist INDEX (not a pointer — the C table is a separate copy).
extern "C" {
    pub fn jka_bgitem_itemlist_ptr() -> *const gitem_t;
    pub fn jka_bgitem_numItems() -> c_int;
    pub fn jka_bgitem_GetItemIndexByTag(tag: c_int, r#type: c_int) -> c_int;
    pub fn jka_bgitem_IsItemSelectable(item: c_int) -> c_int;
    pub fn jka_bgitem_CycleInven(
        direction: c_int,
        io_holdableItem: *mut c_int,
        io_holdableItems: *mut c_int,
    );
    pub fn jka_bgitem_FindItemForPowerup(pw: c_int) -> c_int;
    pub fn jka_bgitem_FindItemForHoldable(pw: c_int) -> c_int;
    pub fn jka_bgitem_FindItemForWeapon(weapon: c_int) -> c_int;
    pub fn jka_bgitem_FindItemForAmmo(ammo: c_int) -> c_int;
    pub fn jka_bgitem_FindItem(classname: *const c_char) -> c_int;

    // BG_CanItemBeGrabbed: scalars + the four ps arrays (stats/persistant/ammo/
    // powerups) marshalled in; the item is bg_itemlist[modelindex]. `has_ps == 0`
    // exercises the NULL-ps safety return.
    pub fn jka_bgitem_can_item_be_grabbed(
        gametype: c_int,
        modelindex: c_int,
        has_ps: c_int,
        trueJedi: c_int,
        trueNonJedi: c_int,
        isJediMaster: c_int,
        duelInProgress: c_int,
        clientNum: c_int,
        forcePowersActive: c_int,
        generic1: c_int,
        es_powerups: c_int,
        eFlags: c_int,
        modelindex2: c_int,
        stats: *const c_int,
        persistant: *const c_int,
        ammo: *const c_int,
        powerups: *const c_int,
    ) -> c_int;
}

// bg_pmove.c data-layer tables (oracle/bg_pmove_oracle.c): the 13 `pm_*` movement
// params and the five force tables, independently transcribed; the Rust test compares
// element-wise against the Rust port (catching a single-value typo on either side).
extern "C" {
    pub fn jka_pm_params() -> *const f32;
    pub fn jka_pm_forceSpeedLevels() -> *const f32;
    pub fn jka_pm_forcePowerNeeded() -> *const c_int;
    pub fn jka_pm_forceJumpHeight() -> *const f32;
    pub fn jka_pm_forceJumpStrength() -> *const f32;
    pub fn jka_pm_forceJumpHeightMax() -> *const f32;
}

// bg_pmove.c stateless saber/anim/entity-index helpers + PM_AddTouchEnt logic
// (oracle/bg_pmove_oracle.c): each verbatim body run over minimal pm/ps contexts; the
// int-marshalling wrappers load state in and read the result back (no struct over FFI).
extern "C" {
    pub fn jka_pm_addtouchent(entityNum: c_int, io_numtouch: *mut c_int, io_touchents: *mut c_int);
    pub fn jka_PM_BGEntForNum(baseEnt: c_ulong, entSize: c_int, num: c_int) -> c_ulong;
    pub fn jka_BG_SabersOff(saberHolstered: c_int, saberAnimLevelBase: c_int) -> c_int;
    pub fn jka_BG_KnockDownable(is_null: c_int, m_iVehicleNum: c_int, emplacedIndex: c_int)
        -> c_int;
    pub fn jka_PM_GetSaberStance(
        saberEntityNum: c_int,
        saberHolstered: c_int,
        saberAnimLevelBase: c_int,
        saberAnimLevel: c_int,
    ) -> c_int;
    pub fn jka_PM_DoSlowFall(legsAnim: c_int, legsTimer: c_int) -> c_int;

    // bg_pmove.c movement primitives (932-1200). Floats/ints in, vec3s by pointer; no
    // struct crosses FFI.
    pub fn jka_PM_ClipVelocity(
        in_v: *const f32,
        normal_v: *const f32,
        overbounce: f32,
        pm_flags: c_int,
        stepSlideFix: c_int,
        clientNum: c_int,
        groundEntityNum: c_int,
        out_v: *mut f32,
    );
    pub fn jka_PM_Friction(
        io_velocity: *mut f32,
        walking: c_int,
        pm_type: c_int,
        clientNum: c_int,
        pm_flags: c_int,
        groundEntityNum: c_int,
        waterlevel: c_int,
        surfaceFlags: c_int,
        frametime: f32,
        pm_flying_in: c_int,
        hasEnt: c_int,
        NPC_class: c_int,
        hasVehicle: c_int,
        vehType: c_int,
        vehFriction: f32,
    );
    pub fn jka_PM_Accelerate(
        io_velocity: *mut f32,
        wishdir_v: *const f32,
        wishspeed: f32,
        accel: f32,
        frametime: f32,
        gametype: c_int,
        m_iVehicleNum: c_int,
        clientNum: c_int,
        pm_type: c_int,
    );
    pub fn jka_PM_CmdScale(forwardmove: c_int, rightmove: c_int, speed: f32) -> f32;
    pub fn jka_PM_SetMovementDir(
        forwardmove: c_int,
        rightmove: c_int,
        movementDir_in: c_int,
    ) -> c_int;
    pub fn jka_PM_SetPMViewAngle(
        angle: *const f32,
        ucmd_angles: *const c_int,
        out_delta_angles: *mut c_int,
        out_viewangles: *mut f32,
    );
    // 1442-3465 self-contained slice: BG_ForceWallJumpStrength (table read /2.5f),
    // PM_SetForceJumpZStart (the `-= 0.1` double-promotion, flattened to one float),
    // PM_DeadMove (velocity friction over pml.walking), PM_FootstepForSurface (the
    // surfaceFlags bit test). The other three slice fns are no-oracle (callback /
    // delegation / move-integrator).
    pub fn jka_BG_ForceWallJumpStrength() -> f32;
    pub fn jka_PM_SetForceJumpZStart(value: f32) -> f32;
    pub fn jka_PM_DeadMove(walking: c_int, io_velocity: *mut f32);
    pub fn jka_PM_FootstepForSurface(surfaceFlags: c_int) -> c_int;
    // 4431-4628 pure-switch anim classifiers: int in, qboolean out (full anim sweep).
    pub fn jka_PM_WalkingAnim(anim: c_int) -> c_int;
    pub fn jka_PM_RunningAnim(anim: c_int) -> c_int;
    pub fn jka_PM_SwimmingAnim(anim: c_int) -> c_int;
    pub fn jka_PM_RollingAnim(anim: c_int) -> c_int;
    // PM_AnglesForSlope (4501): yaw + slope[3] in, angles[3] out. Bit-exact — reuses
    // the q_math_oracle AngleVectors/vectoangles/Q_fabs symbols (same archive).
    pub fn jka_PM_AnglesForSlope(yaw: f32, slope_in: *const f32, angles_out: *mut f32);
    // BG_InSlopeAnim (4594): pure switch over the LEGS_* slope anims (full anim sweep).
    pub fn jka_BG_InSlopeAnim(anim: c_int) -> c_int;
    // PM_DropTimers (7536): the 4 ps timing fields + pml.msec marshalled by value.
    pub fn jka_PM_DropTimers(
        msec: c_int,
        io_pm_time: *mut c_int,
        io_pm_flags: *mut c_int,
        io_legsTimer: *mut c_int,
        io_torsoTimer: *mut c_int,
    );
    // BG_InRollAnim/BG_InKnockDown/BG_InRollES (8272/8285/8322): pure switch, int sweep.
    pub fn jka_BG_InRollAnim(legsAnim: c_int) -> c_int;
    pub fn jka_BG_InKnockDown(anim: c_int) -> c_int;
    pub fn jka_BG_InRollES(anim: c_int) -> c_int;
    // BG_UpdateLookAngles (8493) / BG_SwingAngles (8757): angle easing, vec3s by pointer.
    // Both reuse the q_math_oracle angle/length symbols (bit-exact vs the q_math port).
    #[allow(clippy::too_many_arguments)]
    pub fn jka_BG_UpdateLookAngles(
        looking_debounce_time: c_int,
        last_head_angles: *mut f32,
        time: c_int,
        look_angles: *mut f32,
        look_speed: f32,
        min_pitch: f32,
        max_pitch: f32,
        min_yaw: f32,
        max_yaw: f32,
        min_roll: f32,
        max_roll: f32,
    );
    pub fn jka_BG_SwingAngles(
        destination: f32,
        swing_tolerance: f32,
        clamp_tolerance: f32,
        speed: f32,
        angle: *mut f32,
        swinging: *mut c_int,
        frametime: c_int,
    ) -> f32;
    // BG_InRoll2 (8818): pure switch over legsAnim (full anim sweep).
    pub fn jka_BG_InRoll2(legsAnim: c_int) -> c_int;
    // PM_WeaponOkOnVehicle (9497) / PM_GetOkWeaponForVehicle (9514): the vehicle-legal
    // weapon predicate + the stats[STAT_WEAPONS] scan driven by value.
    pub fn jka_PM_WeaponOkOnVehicle(weapon: c_int) -> c_int;
    pub fn jka_PM_GetOkWeaponForVehicle(statWeapons: c_int) -> c_int;
}

// bg_local.h (oracle/bg_local_h_oracle.c): the `pml_t` by-value deps are transcribed
// verbatim (the header isn't self-contained), then `pml_t` + the movement-tunable
// `#define`s are copied verbatim. Pointer-free => arch-independent.
extern "C" {
    pub fn jka_bl_sizeof_pml_t() -> usize;
    pub fn jka_bl_alignof_pml_t() -> usize;
    pub fn jka_bl_off_forward() -> usize;
    pub fn jka_bl_off_frametime() -> usize;
    pub fn jka_bl_off_msec() -> usize;
    pub fn jka_bl_off_walking() -> usize;
    pub fn jka_bl_off_groundPlane() -> usize;
    pub fn jka_bl_off_groundTrace() -> usize;
    pub fn jka_bl_off_impactSpeed() -> usize;
    pub fn jka_bl_off_previous_origin() -> usize;
    pub fn jka_bl_off_previous_velocity() -> usize;
    pub fn jka_bl_off_previous_waterlevel() -> usize;
    pub fn jka_bl_MIN_WALK_NORMAL() -> f32;
    pub fn jka_bl_TIMER_LAND() -> c_int;
    pub fn jka_bl_TIMER_GESTURE() -> c_int;
    pub fn jka_bl_OVERCLIP() -> f32;
}

// bg_panimate.c (oracle/bg_panimate_oracle.c): the 50 stateless animation
// sequence-checking predicates, copied verbatim. Exposed under their real C
// names (the q_math/q_shared convention for ported functions). `qboolean`
// returns are `c_int`. The Rust tests sweep every input and assert equality.
extern "C" {
    pub fn BG_SaberStanceAnim(anim: c_int) -> c_int;
    pub fn BG_CrouchAnim(anim: c_int) -> c_int;
    pub fn BG_InSpecialJump(anim: c_int) -> c_int;
    pub fn BG_InSaberStandAnim(anim: c_int) -> c_int;
    pub fn BG_InReboundJump(anim: c_int) -> c_int;
    pub fn BG_InReboundHold(anim: c_int) -> c_int;
    pub fn BG_InReboundRelease(anim: c_int) -> c_int;
    pub fn BG_InBackFlip(anim: c_int) -> c_int;
    pub fn BG_DirectFlippingAnim(anim: c_int) -> c_int;
    pub fn BG_SaberInAttackPure(r#move: c_int) -> c_int;
    pub fn BG_SaberInAttack(r#move: c_int) -> c_int;
    pub fn BG_SaberInKata(saberMove: c_int) -> c_int;
    pub fn BG_InKataAnim(anim: c_int) -> c_int;
    pub fn BG_SaberInSpecial(r#move: c_int) -> c_int;
    pub fn BG_KickMove(r#move: c_int) -> c_int;
    pub fn BG_SaberInIdle(r#move: c_int) -> c_int;
    pub fn BG_InExtraDefenseSaberMove(r#move: c_int) -> c_int;
    pub fn BG_FlippingAnim(anim: c_int) -> c_int;
    pub fn BG_SpinningSaberAnim(anim: c_int) -> c_int;
    pub fn BG_SaberInSpecialAttack(anim: c_int) -> c_int;
    pub fn BG_KickingAnim(anim: c_int) -> c_int;
    pub fn BG_InGrappleMove(anim: c_int) -> c_int;
    pub fn BG_BrokenParryForAttack(r#move: c_int) -> c_int;
    pub fn BG_BrokenParryForParry(r#move: c_int) -> c_int;
    pub fn BG_KnockawayForParry(r#move: c_int) -> c_int;
    pub fn PM_SaberBounceForAttack(r#move: c_int) -> c_int;
    pub fn BG_InSpecialDeathAnim(anim: c_int) -> c_int;
    pub fn BG_InDeathAnim(anim: c_int) -> c_int;
    pub fn BG_InKnockDownOnly(anim: c_int) -> c_int;
    pub fn BG_InSaberLockOld(anim: c_int) -> c_int;
    pub fn BG_InSaberLock(anim: c_int) -> c_int;
    pub fn PM_InCartwheel(anim: c_int) -> c_int;
    pub fn BG_StabDownAnim(anim: c_int) -> c_int;
    pub fn PM_SaberDeflectionForQuad(quad: c_int) -> c_int;
    pub fn PM_SaberInDeflect(r#move: c_int) -> c_int;
    pub fn PM_SaberInParry(r#move: c_int) -> c_int;
    pub fn PM_SaberInKnockaway(r#move: c_int) -> c_int;
    pub fn PM_SaberInReflect(r#move: c_int) -> c_int;
    pub fn PM_SaberInStart(r#move: c_int) -> c_int;
    pub fn PM_SaberInReturn(r#move: c_int) -> c_int;
    pub fn BG_SaberInReturn(r#move: c_int) -> c_int;
    pub fn PM_InSaberAnim(anim: c_int) -> c_int;
    pub fn PM_PainAnim(anim: c_int) -> c_int;
    pub fn PM_JumpingAnim(anim: c_int) -> c_int;
    pub fn PM_LandingAnim(anim: c_int) -> c_int;
    pub fn PM_SpinningAnim(anim: c_int) -> c_int;
    pub fn PM_InOnGroundAnim(anim: c_int) -> c_int;
    pub fn BG_SuperBreakLoseAnim(anim: c_int) -> c_int;
    pub fn BG_SuperBreakWinAnim(anim: c_int) -> c_int;
    pub fn BG_SaberLockBreakAnim(anim: c_int) -> c_int;
    pub fn BG_FullBodyTauntAnim(anim: c_int) -> c_int;
    pub fn PM_SaberInTransition(r#move: c_int) -> c_int;
    pub fn BG_SaberInTransitionAny(r#move: c_int) -> c_int;
    // playerState_t* anim-state predicates: int-marshalling wrappers (the C side
    // reads only ps->legsTimer / ps->legsAnim from a minimal local struct).
    pub fn jka_BG_InRoll(legsTimer: c_int, legsAnim: c_int, anim: c_int) -> c_int;
    pub fn jka_PM_InKnockDown(legsTimer: c_int, legsAnim: c_int) -> c_int;
    pub fn jka_PM_InRollComplete(legsTimer: c_int, legsAnim: c_int, anim: c_int) -> c_int;
    pub fn jka_PM_CanRollFromSoulCal(legsTimer: c_int, legsAnim: c_int) -> c_int;
}

// bg_panimate.c animation-SETTER cluster (oracle/bg_panimate_setters_oracle.c):
// the verbatim BG_SetAnim / BG_SetAnimFinal / BG_SaberStartTransAnim call graph.
// `in`/`out` are flat int arrays (no struct crosses FFI); see the C TU header for
// the 19-int input / 6-int output layout.
extern "C" {
    pub fn jka_BG_SetAnimFinal(input: *const c_int, out: *mut c_int);
    pub fn jka_BG_SetAnim(input: *const c_int, out: *mut c_int);
    pub fn jka_BG_SaberStartTransAnim(
        clientNum: c_int,
        saberAnimLevel: c_int,
        weapon: c_int,
        anim: c_int,
        animSpeed: f32,
        broken: c_int,
    ) -> f32;
    pub fn jka_BG_HasAnimation(
        animIndex: c_int,
        animation: c_int,
        numAllAnims: c_int,
        numFramesAtSlot: c_int,
    ) -> c_int;
    pub fn jka_BG_AnimLength(anim: c_int, numFrames: c_int, frameLerp: c_int) -> c_int;
}

// w_saber.h (oracle/w_saber_h_oracle.c): the `#define` tunables + the FJ_* /
// evasionType_t enumerators, read from the AUTHENTIC header (`#include`d after
// predefining its two force-count deps). `c_int` accessors for the ints/flags/
// enum values, `f32` for the saber-radius/bounding-box floats. The Rust test
// asserts each against the corresponding `pub const`.
extern "C" {
    pub fn jka_ws_ARMOR_EFFECT_TIME() -> c_int;
    pub fn jka_ws_SEF_HITENEMY() -> c_int;
    pub fn jka_ws_SEF_HITOBJECT() -> c_int;
    pub fn jka_ws_SEF_HITWALL() -> c_int;
    pub fn jka_ws_SEF_PARRIED() -> c_int;
    pub fn jka_ws_SEF_DEFLECTED() -> c_int;
    pub fn jka_ws_SEF_BLOCKED() -> c_int;
    pub fn jka_ws_SEF_EVENTS() -> c_int;
    pub fn jka_ws_SEF_LOCKED() -> c_int;
    pub fn jka_ws_SEF_INWATER() -> c_int;
    pub fn jka_ws_SEF_LOCK_WON() -> c_int;
    pub fn jka_ws_SES_LEAVING() -> c_int;
    pub fn jka_ws_SES_HOVERING() -> c_int;
    pub fn jka_ws_SES_RETURNING() -> c_int;
    pub fn jka_ws_JSF_AMBUSH() -> c_int;
    pub fn jka_ws_SABER_RADIUS_STANDARD() -> f32;
    pub fn jka_ws_SABER_REFLECT_MISSILE_CONE() -> f32;
    pub fn jka_ws_FORCE_POWER_MAX() -> c_int;
    pub fn jka_ws_MAX_GRIP_DISTANCE() -> c_int;
    pub fn jka_ws_MAX_TRICK_DISTANCE() -> c_int;
    pub fn jka_ws_FORCE_JUMP_CHARGE_TIME() -> c_int;
    pub fn jka_ws_GRIP_DRAIN_AMOUNT() -> c_int;
    pub fn jka_ws_FORCE_LIGHTNING_RADIUS() -> c_int;
    pub fn jka_ws_MAX_DRAIN_DISTANCE() -> c_int;
    pub fn jka_ws_FJ_FORWARD() -> c_int;
    pub fn jka_ws_FJ_BACKWARD() -> c_int;
    pub fn jka_ws_FJ_RIGHT() -> c_int;
    pub fn jka_ws_FJ_LEFT() -> c_int;
    pub fn jka_ws_FJ_UP() -> c_int;
    pub fn jka_ws_EVASION_NONE() -> c_int;
    pub fn jka_ws_EVASION_PARRY() -> c_int;
    pub fn jka_ws_EVASION_DUCK_PARRY() -> c_int;
    pub fn jka_ws_EVASION_JUMP_PARRY() -> c_int;
    pub fn jka_ws_EVASION_DODGE() -> c_int;
    pub fn jka_ws_EVASION_JUMP() -> c_int;
    pub fn jka_ws_EVASION_DUCK() -> c_int;
    pub fn jka_ws_EVASION_FJUMP() -> c_int;
    pub fn jka_ws_EVASION_CARTWHEEL() -> c_int;
    pub fn jka_ws_EVASION_OTHER() -> c_int;
    pub fn jka_ws_NUM_EVASION_TYPES() -> c_int;
    pub fn jka_ws_SABERMINS_X() -> f32;
    pub fn jka_ws_SABERMINS_Y() -> f32;
    pub fn jka_ws_SABERMINS_Z() -> f32;
    pub fn jka_ws_SABERMAXS_X() -> f32;
    pub fn jka_ws_SABERMAXS_Y() -> f32;
    pub fn jka_ws_SABERMAXS_Z() -> f32;
    pub fn jka_ws_SABER_MIN_THROW_DIST() -> f32;
}

// bg_saber.c data tables (oracle/bg_saber_oracle.c): the four tables copied verbatim
// (`#include`s the authentic anims.h for BOTH_*, transcribes the LS_/Q_/BLK_ enums +
// the AFLAG_*/struct). The Rust test compares element-wise. `saberMoveData_t` is the
// repr(C) Rust struct (size 48, layout-verified), so the C base pointer reads directly.
extern "C" {
    pub fn jka_bgsab_saberMoveData_ptr() -> *const saberMoveData_t;
    pub fn jka_bgsab_transitionMove_ptr() -> *const c_int;
    pub fn jka_bgsab_saberMoveTransitionAngle_ptr() -> *const c_int;
    pub fn jka_bgsab_parryDebounce_ptr() -> *const c_int;
}

// bg_saber.c self-contained move helpers (oracle/bg_saber_oracle.c): verbatim
// bodies over the tables above / plain int args. `qboolean`/`saberMoveName_t`
// returns are `c_int`. `PM_SaberMoveQuadrantForMovement` takes a `usercmd_t *`,
// so it is exposed via an int-marshalling wrapper (no struct crosses the FFI).
extern "C" {
    pub fn PM_AttackMoveForQuad(quad: c_int) -> c_int;
    pub fn jka_PM_SaberMoveQuadrantForMovement(forwardmove: c_int, rightmove: c_int) -> c_int;
    pub fn PM_SaberInBounce(r#move: c_int) -> c_int;
    pub fn PM_SaberAttackChainAngle(move1: c_int, move2: c_int) -> c_int;
    pub fn PM_SaberInBrokenParry(r#move: c_int) -> c_int;
    pub fn PM_BrokenParryForParry(r#move: c_int) -> c_int;
    // `PM_SetAnimFrame` only assigns `ps->saberLockFrame`; exposed via an
    // int-marshalling wrapper over a minimal `playerState_t` (no struct crosses
    // the FFI). Returns the field value after the call.
    pub fn jka_PM_SetAnimFrame(frame: c_int) -> c_int;
    pub fn BG_CheckIncrementLockAnim(anim: c_int, winOrLose: c_int) -> c_int;
    // `BG_ForcePowerDrain` reads ps->fd.forcePowerLevel[]/forcePower + velocity[2]
    // and the (already-verified) forcePowerNeeded table; exposed via an
    // int-marshalling wrapper over a minimal `playerState_t` that returns the
    // resulting `forcePower` (no struct crosses the FFI).
    pub fn jka_BG_ForcePowerDrain(
        forcePower: c_int,
        overrideAmt: c_int,
        levelForPower: c_int,
        levelForLevitation: c_int,
        velZ: f32,
        startForcePower: c_int,
    ) -> c_int;

    // PM_CanDoRollStab (bg_saber.c): the pm-/g_entities-gated roll-stab veto, exposed
    // via an int-marshalling wrapper that models BG_MySaber's two resolved sabers as
    // (present, flags) pairs and runs the verbatim SFL_NO_ROLL_STAB decision.
    pub fn jka_PM_CanDoRollStab(
        weapon: c_int,
        s0_present: c_int,
        s0_flags: c_int,
        s1_present: c_int,
        s1_flags: c_int,
    ) -> c_int;

    // PM_SaberJumpAttackMove2 (bg_saber.c): the pm-/g_entities-gated jump+fwd+attack
    // move choice, exposed via an int-marshalling wrapper that models BG_MySaber's two
    // resolved sabers as (present, jumpAtkFwdMove) pairs and passes saberAnimLevel,
    // running the verbatim override/cancel/style decision.
    pub fn jka_PM_SaberJumpAttackMove2(
        s0_present: c_int,
        s0_move: c_int,
        s1_present: c_int,
        s1_move: c_int,
        saberAnimLevel: c_int,
    ) -> c_int;
}

// bg_saberLoad.c self-contained helpers (oracle/bg_saberLoad_oracle.c): verbatim
// bodies against verbatim transcriptions of their by-value q_shared.h dep types
// (layout verified in q_shared_h_oracle.c). Q_stricmp links to q_shared_oracle.c,
// Q_irand to q_math_oracle.c (PC moved it there).
extern "C" {
    /// `saber_colors_t TranslateSaberColor( const char *name )` (returns a `saber_colors_t`, i.e. `int`).
    pub fn TranslateSaberColor(name: *const c_char) -> c_int;
    /// `saber_styles_t TranslateSaberStyle( const char *name )` (returns a `saber_styles_t`, i.e. `int`).
    pub fn TranslateSaberStyle(name: *const c_char) -> c_int;
    /// `qboolean BG_ParseLiteral( const char **data, const char *string )`.
    pub fn BG_ParseLiteral(data: *mut *const c_char, string: *const c_char) -> c_int;

    // saber accessor family — verbatim bodies over the transcribed (layout-verified)
    // saberInfo_t/bladeInfo_t; the BG_BLADE_*Trail use trail index 0 (non-_XBOX).
    pub fn BG_BLADE_ActivateTrail(blade: *mut bladeInfo_t, duration: f32);
    pub fn BG_BLADE_DeactivateTrail(blade: *mut bladeInfo_t, duration: f32);
    pub fn BG_SI_Activate(saber: *mut saberInfo_t);
    pub fn BG_SI_Deactivate(saber: *mut saberInfo_t);
    pub fn BG_SI_BladeActivate(saber: *mut saberInfo_t, iBlade: c_int, bActive: c_int);
    pub fn BG_SI_Active(saber: *mut saberInfo_t) -> c_int;
    pub fn BG_SI_SetLength(saber: *mut saberInfo_t, length: f32);
    pub fn BG_SI_SetDesiredLength(saber: *mut saberInfo_t, len: f32, bladeNum: c_int);
    pub fn BG_SI_SetLengthGradual(saber: *mut saberInfo_t, time: c_int);
    pub fn BG_SI_Length(saber: *mut saberInfo_t) -> f32;
    pub fn BG_SI_LengthMax(saber: *mut saberInfo_t) -> f32;
    pub fn BG_SI_ActivateTrail(saber: *mut saberInfo_t, duration: f32);
    pub fn BG_SI_DeactivateTrail(saber: *mut saberInfo_t, duration: f32);
    /// `void WP_SaberSetColor( saberInfo_t *sabers, int saberNum, int bladeNum, char *colorName )`.
    pub fn WP_SaberSetColor(
        sabers: *mut saberInfo_t,
        saberNum: c_int,
        bladeNum: c_int,
        colorName: *const c_char,
    );

    // saber blade-style / valid-style predicates (bg_saberLoad.c) — verbatim bodies
    // over the transcribed (layout-verified) saberInfo_t.
    pub fn WP_SaberBladeUseSecondBladeStyle(saber: *mut saberInfo_t, bladeNum: c_int) -> c_int;
    pub fn WP_SaberBladeDoTransitionDamage(saber: *mut saberInfo_t, bladeNum: c_int) -> c_int;
    pub fn WP_UseFirstValidSaberStyle(
        saber1: *mut saberInfo_t,
        saber2: *mut saberInfo_t,
        saberHolstered: c_int,
        saberAnimLevel: *mut c_int,
    ) -> c_int;
    pub fn WP_SaberStyleValidForSaber(
        saber1: *mut saberInfo_t,
        saber2: *mut saberInfo_t,
        saberHolstered: c_int,
        saberAnimLevel: c_int,
    ) -> c_int;
    pub fn WP_SaberCanTurnOffSomeBlades(saber: *mut saberInfo_t) -> c_int;
}

// surfaceflags.h (oracle/surfaceflags_h_oracle.c): the authentic header is #include'd
// and every numeric #define exposed in declaration order; the Rust test compares
// element-wise.
extern "C" {
    pub fn jka_surfaceflags_values() -> *const c_int;
}

// g_combat.c (oracle/g_combat_oracle.c): the pure hit-location classifier
// G_GetHitLocation, called through a scalar/array-marshalling wrapper.
extern "C" {
    pub fn jka_G_GetHitLocation(
        has_client: c_int,
        yaw: f32,
        absmin: *const f32,
        absmax: *const f32,
        mins: *const f32,
        maxs: *const f32,
        ppoint: *const f32,
    ) -> c_int;
    pub fn jka_G_GetDismemberLoc(
        angles: *const f32,
        origin: *const f32,
        limb_type: c_int,
        out: *mut f32,
    );
}

// g_combat.c (oracle/g_combat_oracle.c): G_ApplyKnockback, transcribed onto
// marshalled scalars/arrays. All mutable target fields go in and come back out so the
// Rust gentity_t comparison covers velocity / trDelta / trBase / trTime / pm_time /
// pm_flags regardless of which branch fires.
extern "C" {
    pub fn jka_G_ApplyKnockback(
        physics_bounce: f32,
        has_client: c_int,
        tr_type: c_int,
        new_dir: *const f32,
        knockback: f32,
        gravity_value: f32,
        knockback_value: f32,
        level_time: c_int,
        in_velocity: *const f32,
        in_tr_delta: *const f32,
        in_tr_base: *const f32,
        in_current_origin: *const f32,
        in_tr_time: c_int,
        in_pm_time: c_int,
        in_pm_flags: c_int,
        out_velocity: *mut f32,
        out_tr_delta: *mut f32,
        out_tr_base: *mut f32,
        out_tr_time: *mut c_int,
        out_pm_time: *mut c_int,
        out_pm_flags: *mut c_int,
    );
}

// g_combat.c (oracle/g_combat_oracle.c): RaySphereIntersections, pure math. The wrapper
// marshals flat float inputs and copies the in-place-normalized dir plus both intersection
// slots back out, returning the hit count.
extern "C" {
    pub fn jka_RaySphereIntersections(
        origin: *const f32,
        radius: f32,
        point: *const f32,
        dir: *const f32,
        out_dir: *mut f32,
        out0: *mut f32,
        out1: *mut f32,
    ) -> c_int;
}

// g_combat.c (oracle/g_combat_oracle.c): G_InKnockDown, a pure switch over ps->legsAnim.
// The wrapper takes a bare legsAnim int and returns the qboolean result.
extern "C" {
    pub fn jka_G_InKnockDown(legs_anim: c_int) -> c_int;
}

// g_combat.c (oracle/g_combat_oracle.c): G_Knockdown, gated by BG_KnockDownable. The
// wrapper takes the two gate inputs (m_iVehicleNum, emplacedIndex) and level.time, plus
// in/out pointers for the four mutated ps fields (seeded by the caller so the
// not-knockdownable branch is observable).
extern "C" {
    pub fn jka_G_Knockdown(
        m_i_vehicle_num: c_int,
        emplaced_index: c_int,
        level_time: c_int,
        force_hand_extend: *mut c_int,
        force_dodge_anim: *mut c_int,
        force_hand_extend_time: *mut c_int,
        quicker_getup: *mut c_int,
    );
}

// g_combat.c (oracle/g_combat_oracle.c): G_CheckSpecialDeathAnim, the context-sensitive
// death-anim selector. The wrapper runs the verbatim body on a reduced playerState
// (legsAnim/legsTimer/viewangles/velocity) with the BG_InRoll/BG_FlippingAnim gates passed
// as flags and the anim's numFrames/frameLerp marshalled in (animLength = numFrames *
// fabs(frameLerp) reproduced verbatim).
extern "C" {
    pub fn jka_G_CheckSpecialDeathAnim(
        legs_anim: c_int,
        legs_timer: c_int,
        viewangles: *const f32,
        velocity: *const f32,
        in_roll: c_int,
        flipping: c_int,
        num_frames: u16,
        frame_lerp: i16,
    ) -> c_int;
}

// g_combat.c (oracle/g_combat_oracle.c): G_PickDeathAnim's hit-location selector. The
// wrapper runs the verbatim hitLoc switch with the resolved hitLoc, damage, mod,
// max_health, objVelocity, and the G_CheckSpecialDeathAnim result (-1 -> run switch)
// marshalled in. Its Q_irand forwards to irand (the holdrand MSVC LCG); the test seeds
// Rust's Rand_Init and the oracle's Rand_Init to the same value so the draws stay in lockstep.
extern "C" {
    pub fn jka_G_PickDeathAnim(
        hit_loc: c_int,
        damage: c_int,
        mod_: c_int,
        max_health: c_int,
        obj_velocity: *const f32,
        special_death_anim: c_int,
    ) -> c_int;
}

// g_client.c (oracle/g_client_oracle.c): SetClientViewAngle, pure per-axis delta-angle
// arithmetic plus two VectorCopy writes. The wrapper marshals the flat angle[3] and
// pers.cmd.angles[3] inputs and copies the three written arrays (ps.delta_angles,
// s.angles, ps.viewangles) back out.
extern "C" {
    pub fn jka_SetClientViewAngle(
        angle: *const f32,
        cmd_angles: *const c_int,
        out_delta_angles: *mut c_int,
        out_s_angles: *mut f32,
        out_viewangles: *mut f32,
    );
}

// g_client.c (oracle/g_client_oracle.c): ClientCleanName, the pure player-name sanitiser
// (leading-space strip, 3-space cap, ^0-black drop, ^N keep, "Padawan" fallback). The
// wrapper just forwards the in/out C strings and outSize.
extern "C" {
    pub fn jka_ClientCleanName(
        in_: *const core::ffi::c_char,
        out: *mut core::ffi::c_char,
        outSize: c_int,
    );
}

// g_mover.c (oracle/g_mover_oracle.c): the three pure rotation-matrix helpers
// G_CreateRotationMatrix / G_TransposeMatrix / G_RotatePoint. 3x3 matrices marshal as
// flat row-major float[9]; points/angles as float[3]. The extract bundles verbatim
// AngleVectors + VectorInverse so it is self-contained.
extern "C" {
    pub fn jka_G_CreateRotationMatrix(angles: *const f32, out_matrix: *mut f32);
    pub fn jka_G_TransposeMatrix(in_matrix: *const f32, out_transpose: *mut f32);
    pub fn jka_G_RotatePoint(in_point: *const f32, in_matrix: *const f32, out_point: *mut f32);
}

// g_weapon.c (oracle/g_weapon_oracle.c): the pure helpers whose behavior is plain
// arithmetic over vec3_t / scalars — WP_SpeedOfMissileForWeapon (constant),
// VectorNPos (componentwise abs), SnapVectorTowards (snap-toward-`to`). vec3_t cross
// the FFI as float pointers.
extern "C" {
    pub fn jka_WP_SpeedOfMissileForWeapon(wp: c_int, alt_fire: c_int) -> f32;
    pub fn jka_VectorNPos(in_: *const f32, out: *mut f32);
    pub fn jka_SnapVectorTowards(v: *mut f32, to: *const f32);
}

// g_missile.c (oracle/g_missile_oracle.c): the pure helpers whose behavior is plain
// arithmetic over vec3_t — G_BounceProjectile (reflect a ray off a plane and project
// it out 8192 units). vec3_t cross the FFI as float pointers.
extern "C" {
    pub fn jka_G_BounceProjectile(
        start: *const f32,
        impact: *const f32,
        dir: *const f32,
        endout: *mut f32,
    );
}

// g_items.c (oracle/g_items_oracle.c): adjustRespawnTime, the player-count respawn-time
// scaler. Pure arithmetic over the item type/tag plus the two read globals
// (g_adaptRespawn.integer, level.numPlayingClients), which the wrapper lifts to params.
extern "C" {
    pub fn jka_adjustRespawnTime(
        pre_respawn_time: f32,
        item_type: c_int,
        item_tag: c_int,
        g_adapt_respawn_integer: c_int,
        num_playing_clients: c_int,
    ) -> c_int;
}

// g_misc.c (oracle/g_misc_oracle.c): the file-static ref-tag pool ops TAG_Init and
// TAG_FindOwner, run over a layout-faithful copy of `refTagOwnerMap`. The map is opaque
// to Rust; the wrappers seed/clear it and report the result as an int (1 = fully zeroed
// for TAG_Init; the matched slot index, or -1, for TAG_FindOwner).
extern "C" {
    pub fn jka_TAG_Init_zeroes() -> c_int;
    pub fn jka_TAG_clear_map();
    pub fn jka_TAG_seed_owner(idx: c_int, name: *const c_char, inuse: c_int);
    pub fn jka_TAG_FindOwner_index(owner: *const c_char) -> c_int;
    pub fn jka_TAG_seed_tag(owner_idx: c_int, tag_idx: c_int, name: *const c_char, inuse: c_int);
    pub fn jka_TAG_Find_index(owner: *const c_char, name: *const c_char) -> c_int;
    #[allow(clippy::too_many_arguments)]
    pub fn jka_TAG_Add(
        name: *const c_char,
        owner: *const c_char,
        ox: f32,
        oy: f32,
        oz: f32,
        ax: f32,
        ay: f32,
        az: f32,
        radius: c_int,
        flags: c_int,
    ) -> c_int;
    pub fn jka_TAG_get_inuse(oi: c_int, ti: c_int) -> c_int;
    pub fn jka_TAG_get_radius(oi: c_int, ti: c_int) -> c_int;
    pub fn jka_TAG_get_flags(oi: c_int, ti: c_int) -> c_int;
    pub fn jka_TAG_get_origin(oi: c_int, ti: c_int, c: c_int) -> f32;
    pub fn jka_TAG_get_angles(oi: c_int, ti: c_int, c: c_int) -> f32;
    pub fn jka_TAG_get_name(oi: c_int, ti: c_int, out: *mut c_char);
    pub fn jka_TAG_owner_inuse(oi: c_int) -> c_int;
    pub fn jka_TAG_owner_name(oi: c_int, out: *mut c_char);
    pub fn jka_TAG_GetOrigin(owner: *const c_char, name: *const c_char, out: *mut f32) -> c_int;
    pub fn jka_TAG_GetAngles(owner: *const c_char, name: *const c_char, out: *mut f32) -> c_int;
    pub fn jka_TAG_GetRadius(owner: *const c_char, name: *const c_char) -> c_int;
}

// g_active.c (oracle/g_active_oracle.c): the two pure int->qboolean classifiers,
// plus the pure vector-math push-vector blend and the timer-residual decrementer.
extern "C" {
    pub fn jka_G_StandingAnim(anim: c_int) -> c_int;
    pub fn jka_G_ActionButtonPressed(buttons: c_int) -> c_int;
    /// `G_AddPushVecToUcmd` math extracted to scalar/vector in/out params.
    pub fn jka_G_AddPushVecToUcmd(
        pushVec: *mut f32,
        viewangles: *const f32,
        speed: *mut f32,
        forwardmove: *mut i8,
        rightmove: *mut i8,
        pushVecTime: c_int,
        levelTime: c_int,
    );
    /// `ClientTimerActions` math extracted to scalar in/out params.
    pub fn jka_ClientTimerActions(
        msec: c_int,
        timeResidual: *mut c_int,
        health: *mut c_int,
        stat_max_health: c_int,
        stat_armor: *mut c_int,
    );
}

// ai_main.c (oracle/ai_main_oracle.c): the pure field-of-vision check (verbatim body,
// with its lone AngleMod callee inlined `static` so the TU is self-contained).
extern "C" {
    /// `int InFieldOfVision(vec3_t viewangles, float fov, vec3_t angles)` — `angles` is
    /// mutated in place (each axis replaced by its `AngleMod`), hence `*mut f32`.
    pub fn jka_InFieldOfVision(viewangles: *const f32, fov: f32, angles: *mut f32) -> c_int;
    /// `float AngleDifference(float ang1, float ang2)` — pure wrap-corrected difference.
    pub fn jka_AngleDifference(ang1: f32, ang2: f32) -> f32;
    /// `float BotChangeViewAngle(float angle, float ideal_angle, float speed)` — pure
    /// `±speed`-clamped move toward `ideal_angle`, normalized via `AngleMod`.
    pub fn jka_BotChangeViewAngle(angle: f32, ideal_angle: f32, speed: f32) -> f32;
    /// `int BotWeaponBlockable(int weapon)` — pure `WP_*`-id classification.
    pub fn jka_BotWeaponBlockable(weapon: c_int) -> c_int;
}

// g_timer.c (oracle/g_timer_oracle.c): the string-keyed per-entity timer pool. TIMER_Set
// and TIMER_Done bodies are verbatim; the host `ent->s.number` key is passed as a bare int
// (`_idx` wrappers) and `level.time` lives in a file-local int set by the reset/set-time
// shims, so the TU is self-contained.
extern "C" {
    /// Reset the oracle's timer pool and set its clock (`jka_level_time`) to `level_time`.
    pub fn jka_TIMER_oracle_reset(level_time: c_int);
    /// Set the oracle's clock (`jka_level_time`) without clearing the pool.
    pub fn jka_TIMER_oracle_set_time(level_time: c_int);
    /// `TIMER_Set` keyed by entity index `num` (g_timer.c:129).
    pub fn jka_TIMER_Set_idx(num: c_int, identifier: *const c_char, duration: c_int);
    /// `TIMER_Done` keyed by entity index `num` (g_timer.c:165).
    pub fn jka_TIMER_Done_idx(num: c_int, identifier: *const c_char) -> c_int;
}

// g_ICARUScb.c (oracle/g_ICARUScb_oracle.c): the ICARUS callback layer. Q3_TaskIDClear is
// the one wholly self-contained leaf (body `*taskID = -1;`), reproduced verbatim.
extern "C" {
    /// `void Q3_TaskIDClear( int *taskID )` (g_ICARUScb.c:269) — clears a task ID to -1.
    pub fn jka_Q3_TaskIDClear(taskID: *mut c_int);
}

// g_vehicles.c (oracle/g_vehicles_oracle.c): the pure surface-name -> SHIPSURF_*
// predicate (its lone Q_strncmp callee inlined `static` so the TU is self-contained).
extern "C" {
    /// `int G_ShipSurfaceForSurfName( const char *surfaceName )` (g_vehicles.c:2650).
    pub fn G_ShipSurfaceForSurfName(surfaceName: *const c_char) -> c_int;
}

// FighterNPC.c (oracle/fighternpc_oracle.c): the pure scalar angle-decay helper.
extern "C" {
    /// `float PredictedAngularDecrement( float scale, float timeMod, float originalAngle )`
    /// (FighterNPC.c:234, file-`static`).
    pub fn jka_PredictedAngularDecrement(scale: f32, timeMod: f32, originalAngle: f32) -> f32;
}

// NPC_stats.c (oracle/npc_stats_oracle.c): the pure rank-name -> rank_t match.
// Its lone Q_stricmp callee is reused from q_shared_oracle.c (same static lib).
extern "C" {
    /// `static rank_t TranslateRankName( const char *name )` (NPC_stats.c:292).
    pub fn jka_TranslateRankName(name: *const c_char) -> c_int;
}

// NPC_spawn.c (oracle/npc_spawn_oracle.c): the pure team/NPC_type -> weapon-bitmask
// lookup. Its Q_stricmp/Q_strncmp callees are reused from q_shared_oracle.c.
extern "C" {
    /// `int NPC_WeaponsForTeam( team_t team, int spawnflags, const char *NPC_type )`
    /// (NPC_spawn.c:516).
    pub fn jka_NPC_WeaponsForTeam(team: c_int, spawnflags: c_int, NPC_type: *const c_char)
        -> c_int;
}

// NPC_goal.c (oracle/npc_goal_oracle.c): the pure AABB overlap test. `vec3_t` is
// `float[3]`, decaying to `float*`.
extern "C" {
    /// `qboolean G_BoundsOverlap( const vec3_t mins1, const vec3_t maxs1, const
    /// vec3_t mins2, const vec3_t maxs2 )` (NPC_goal.c:94).
    pub fn jka_G_BoundsOverlap(
        mins1: *const f32,
        maxs1: *const f32,
        mins2: *const f32,
        maxs2: *const f32,
    ) -> c_int;
}

// g_nav.c (oracle/g_nav_oracle.c): the pure navgoal-reached test. Its only callee,
// G_BoundsOverlap, is inlined in the oracle object. `vec3_t` is `float[3]`,
// decaying to `float*`.
extern "C" {
    /// `qboolean NAV_HitNavGoal( vec3_t point, vec3_t mins, vec3_t maxs, vec3_t
    /// dest, int radius, qboolean flying )` (g_nav.c:167).
    pub fn jka_NAV_HitNavGoal(
        point: *const f32,
        mins: *const f32,
        maxs: *const f32,
        dest: *const f32,
        radius: c_int,
        flying: c_int,
    ) -> c_int;
}
