/*
 * Oracle TU for teams.h. The two int-width enums (npcteam_t, class_t) are
 * embedded by value in gclient_t, so the g_local.h master needs them. The bodies
 * are copied verbatim and the C compiler numbers them; the accessors expose a
 * spread of checkpoints (anchor / interior / terminal) so the Rust consts in
 * `src/codemp/game/teams_h.rs` can be asserted bit-for-bit -- the long class_t
 * list is easy to miscount.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

enum {
	NPCTEAM_FREE,
	NPCTEAM_ENEMY,
	NPCTEAM_PLAYER,
	NPCTEAM_NEUTRAL,
	NPCTEAM_NUM_TEAMS
};

typedef enum {
	CLASS_NONE,
	CLASS_ATST,
	CLASS_BARTENDER,
	CLASS_BESPIN_COP,
	CLASS_CLAW,
	CLASS_COMMANDO,
	CLASS_DESANN,
	CLASS_FISH,
	CLASS_FLIER2,
	CLASS_GALAK,
	CLASS_GLIDER,
	CLASS_GONK,
	CLASS_GRAN,
	CLASS_HOWLER,
	CLASS_IMPERIAL,
	CLASS_IMPWORKER,
	CLASS_INTERROGATOR,
	CLASS_JAN,
	CLASS_JEDI,
	CLASS_KYLE,
	CLASS_LANDO,
	CLASS_LIZARD,
	CLASS_LUKE,
	CLASS_MARK1,
	CLASS_MARK2,
	CLASS_GALAKMECH,
	CLASS_MINEMONSTER,
	CLASS_MONMOTHA,
	CLASS_MORGANKATARN,
	CLASS_MOUSE,
	CLASS_MURJJ,
	CLASS_PRISONER,
	CLASS_PROBE,
	CLASS_PROTOCOL,
	CLASS_R2D2,
	CLASS_R5D2,
	CLASS_REBEL,
	CLASS_REBORN,
	CLASS_REELO,
	CLASS_REMOTE,
	CLASS_RODIAN,
	CLASS_SEEKER,
	CLASS_SENTRY,
	CLASS_SHADOWTROOPER,
	CLASS_STORMTROOPER,
	CLASS_SWAMP,
	CLASS_SWAMPTROOPER,
	CLASS_TAVION,
	CLASS_TRANDOSHAN,
	CLASS_UGNAUGHT,
	CLASS_JAWA,
	CLASS_WEEQUAY,
	CLASS_BOBAFETT,
	CLASS_VEHICLE,
	CLASS_RANCOR,
	CLASS_WAMPA,
	CLASS_NUM_CLASSES
} class_t;

int jka_teams_NPCTEAM_FREE(void) { return NPCTEAM_FREE; }
int jka_teams_NPCTEAM_NUM_TEAMS(void) { return NPCTEAM_NUM_TEAMS; }

int jka_teams_CLASS_NONE(void) { return CLASS_NONE; }
int jka_teams_CLASS_GONK(void) { return CLASS_GONK; }
int jka_teams_CLASS_GALAKMECH(void) { return CLASS_GALAKMECH; }
int jka_teams_CLASS_BOBAFETT(void) { return CLASS_BOBAFETT; }
int jka_teams_CLASS_WAMPA(void) { return CLASS_WAMPA; }
int jka_teams_CLASS_NUM_CLASSES(void) { return CLASS_NUM_CLASSES; }
