/*
 * Oracle TU for bg_local.h's pml_t struct and the movement-tunable #defines.
 *
 * bg_local.h is NOT self-contained (it uses vec3_t/trace_t/qboolean plus the
 * namespace macros and references playerState_t etc. in its prototypes), so it
 * can't be #include'd directly. Instead the pml_t by-value deps (qboolean, vec3_t,
 * cplane_t, trace_t) are transcribed verbatim -- same as bg_vehicles_h_oracle.c /
 * g_local_oracle.c -- and then pml_t and the #defines are copied verbatim from the
 * authentic Raven bg_local.h. The C compiler yields the real sizeof/offsetof, which
 * the Rust port asserts against.
 *
 * pml_t is pointer-free => arch-independent; built only under the `oracle` feature.
 */

#include <stddef.h>

/* ---- base types (verbatim, same as g_local_oracle.c) ---- */
typedef int qboolean;
typedef unsigned char byte;
typedef float vec_t;
typedef vec_t vec3_t[3];

typedef struct cplane_s { vec3_t normal; float dist; byte type; byte signbits; byte pad[2]; } cplane_t;
typedef struct {
	byte allsolid; byte startsolid; short entityNum; float fraction;
	vec3_t endpos; cplane_t plane; int surfaceFlags; int contents;
} trace_t;

/* ---- #defines (verbatim from bg_local.h) ---- */
#define	MIN_WALK_NORMAL	0.7f
#define	TIMER_LAND		130
#define	TIMER_GESTURE	(34*66+50)
#define	OVERCLIP		1.001f

/* ---- pml_t (verbatim from bg_local.h) ---- */
typedef struct
{
	vec3_t		forward, right, up;
	float		frametime;

	int			msec;

	qboolean	walking;
	qboolean	groundPlane;
	trace_t		groundTrace;

	float		impactSpeed;

	vec3_t		previous_origin;
	vec3_t		previous_velocity;
	int			previous_waterlevel;
} pml_t;

size_t jka_bl_sizeof_pml_t(void) { return sizeof(pml_t); }
size_t jka_bl_alignof_pml_t(void) { return _Alignof(pml_t); }

size_t jka_bl_off_forward(void) { return offsetof(pml_t, forward); }
size_t jka_bl_off_frametime(void) { return offsetof(pml_t, frametime); }
size_t jka_bl_off_msec(void) { return offsetof(pml_t, msec); }
size_t jka_bl_off_walking(void) { return offsetof(pml_t, walking); }
size_t jka_bl_off_groundPlane(void) { return offsetof(pml_t, groundPlane); }
size_t jka_bl_off_groundTrace(void) { return offsetof(pml_t, groundTrace); }
size_t jka_bl_off_impactSpeed(void) { return offsetof(pml_t, impactSpeed); }
size_t jka_bl_off_previous_origin(void) { return offsetof(pml_t, previous_origin); }
size_t jka_bl_off_previous_velocity(void) { return offsetof(pml_t, previous_velocity); }
size_t jka_bl_off_previous_waterlevel(void) { return offsetof(pml_t, previous_waterlevel); }

float jka_bl_MIN_WALK_NORMAL(void) { return MIN_WALK_NORMAL; }
int jka_bl_TIMER_LAND(void) { return TIMER_LAND; }
int jka_bl_TIMER_GESTURE(void) { return TIMER_GESTURE; }
float jka_bl_OVERCLIP(void) { return OVERCLIP; }
