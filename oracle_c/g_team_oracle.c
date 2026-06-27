/*
 * Oracle TU for g_team.c's pure team-name / team-query helpers, transcribed
 * VERBATIM from raven-jediacademy/codemp/game/g_team.c:
 *   OtherTeam, TeamName, OtherTeamName, TeamColorString.
 *
 * These are dependency-free integer-in / value-out functions (the string ones
 * return immutable `^N` color-code string literals), so `src/codemp/game/
 * g_team.rs` can assert its ports match the real Raven C bit-exact.
 *
 * The TEAM_* and S_COLOR_* constants are transcribed below from
 * q_shared.h (the same numeric/string values the Rust port uses).
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

/* team_t values (verbatim from q_shared.h) */
#define TEAM_FREE       0
#define TEAM_RED        1
#define TEAM_BLUE       2
#define TEAM_SPECTATOR  3

/* color-code string literals (verbatim from q_shared.h) */
#define S_COLOR_RED     "^1"
#define S_COLOR_YELLOW  "^3"
#define S_COLOR_BLUE    "^4"
#define S_COLOR_WHITE   "^7"

int jka_OtherTeam(int team) {
	if (team==TEAM_RED)
		return TEAM_BLUE;
	else if (team==TEAM_BLUE)
		return TEAM_RED;
	return team;
}

const char *jka_TeamName(int team)  {
	if (team==TEAM_RED)
		return "RED";
	else if (team==TEAM_BLUE)
		return "BLUE";
	else if (team==TEAM_SPECTATOR)
		return "SPECTATOR";
	return "FREE";
}

const char *jka_OtherTeamName(int team) {
	if (team==TEAM_RED)
		return "BLUE";
	else if (team==TEAM_BLUE)
		return "RED";
	else if (team==TEAM_SPECTATOR)
		return "SPECTATOR";
	return "FREE";
}

const char *jka_TeamColorString(int team) {
	if (team==TEAM_RED)
		return S_COLOR_RED;
	else if (team==TEAM_BLUE)
		return S_COLOR_BLUE;
	else if (team==TEAM_SPECTATOR)
		return S_COLOR_YELLOW;
	return S_COLOR_WHITE;
}

/* SortClients — verbatim ascending int-index comparator (g_team.c:1080).
 * Dependency-free: subtracts the two int values the void* args point at. */
int jka_SortClients(const void *a, const void *b) {
	return *(int *)a - *(int *)b;
}
