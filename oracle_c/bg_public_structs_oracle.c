/*
 * Oracle TU for the bg_public.h structs. Carries verbatim copies of the struct
 * definitions over a minimal prelude (the dependent int-typedef enums + the
 * array-sizing limits), and exposes the C compiler's `sizeof`/`offsetof` so the
 * Rust port can assert its layout bit-for-bit. Copy rather than #include because
 * bg_public.h drags in the clang-hostile include tree.
 *
 * `animation_t` is `#pragma pack(1)` + pointer-free (arch-independent). The other
 * structs carry raw pointers, so these accessors validate the HOST (64-bit)
 * layout only -- matching the `#[cfg(target_pointer_width = "64")]` asserts in the
 * Rust port (see DEVIATIONS.md). Built only under the `oracle` cargo feature.
 */

#include <stddef.h>

/* --- prelude: dependent types/consts (enums are int-width) --- */
typedef int qboolean;
typedef int animEventType_t;
typedef int itemType_t;
typedef int fieldtype_t;
typedef int saberMoveName_t;
#define MAX_QPATH 64
#define MAX_RANDOM_ANIM_SOUNDS 4
#define AED_ARRAY_SIZE (MAX_RANDOM_ANIM_SOUNDS + 3)
#define MAX_ANIM_EVENTS 300
#define MAX_ITEM_MODELS 4

/* --- verbatim struct definitions from bg_public.h --- */
#pragma pack(push, 1)
typedef struct animation_s {
	unsigned short		firstFrame;
	unsigned short		numFrames;
	short				frameLerp;			/* msec between frames */
	signed char			loopFrames;			/* 0 to numFrames */
} animation_t;
#pragma pack(pop)

typedef struct animevent_s {
	animEventType_t	eventType;
	unsigned short	keyFrame;
	signed short	eventData[AED_ARRAY_SIZE];
	char			*stringData;
} animevent_t;

typedef struct {
	char			filename[MAX_QPATH];
	animation_t		*anims;
} bgLoadedAnim_t;

typedef struct {
	char			filename[MAX_QPATH];
	animevent_t		torsoAnimEvents[MAX_ANIM_EVENTS];
	animevent_t		legsAnimEvents[MAX_ANIM_EVENTS];
	qboolean		eventsParsed;
} bgLoadedEvents_t;

typedef struct gitem_s {
	char		*classname;
	char		*pickup_sound;
	char		*world_model[MAX_ITEM_MODELS];
	char		*view_model;
	char		*icon;
	int			quantity;
	itemType_t  giType;
	int			giTag;
	char		*precaches;
	char		*sounds;
	char		*description;
} gitem_t;

typedef struct {
	char *name;
	int animToUse;
	int	startQuad;
	int	endQuad;
	unsigned animSetFlags;
	int blendTime;
	int blocking;
	saberMoveName_t chain_idle;
	saberMoveName_t chain_attack;
	qboolean trailLength;
} saberMoveData_t;

typedef struct {
	char	*name;
	int		ofs;
	fieldtype_t	type;
	int		flags;
} BG_field_t;

/* --- sizeof + offsetof accessors --- */
size_t jka_bgs_sizeof_animation_t(void) { return sizeof(animation_t); }
size_t jka_bgs_off_animation_loopFrames(void) { return offsetof(animation_t, loopFrames); }

size_t jka_bgs_sizeof_animevent_t(void) { return sizeof(animevent_t); }
size_t jka_bgs_off_animevent_eventData(void) { return offsetof(animevent_t, eventData); }
size_t jka_bgs_off_animevent_stringData(void) { return offsetof(animevent_t, stringData); }

size_t jka_bgs_sizeof_bgLoadedAnim_t(void) { return sizeof(bgLoadedAnim_t); }
size_t jka_bgs_off_bgLoadedAnim_anims(void) { return offsetof(bgLoadedAnim_t, anims); }

size_t jka_bgs_sizeof_bgLoadedEvents_t(void) { return sizeof(bgLoadedEvents_t); }
size_t jka_bgs_off_bgLoadedEvents_legsAnimEvents(void) { return offsetof(bgLoadedEvents_t, legsAnimEvents); }
size_t jka_bgs_off_bgLoadedEvents_eventsParsed(void) { return offsetof(bgLoadedEvents_t, eventsParsed); }

size_t jka_bgs_sizeof_gitem_t(void) { return sizeof(gitem_t); }
size_t jka_bgs_off_gitem_world_model(void) { return offsetof(gitem_t, world_model); }
size_t jka_bgs_off_gitem_quantity(void) { return offsetof(gitem_t, quantity); }
size_t jka_bgs_off_gitem_precaches(void) { return offsetof(gitem_t, precaches); }

size_t jka_bgs_sizeof_saberMoveData_t(void) { return sizeof(saberMoveData_t); }
size_t jka_bgs_off_saberMoveData_chain_idle(void) { return offsetof(saberMoveData_t, chain_idle); }
size_t jka_bgs_off_saberMoveData_trailLength(void) { return offsetof(saberMoveData_t, trailLength); }

size_t jka_bgs_sizeof_BG_field_t(void) { return sizeof(BG_field_t); }
size_t jka_bgs_off_BG_field_type(void) { return offsetof(BG_field_t, type); }
