/* Oracle extraction from refs/raven-jediacademy/codemp/game/NPC_stats.c.
 *
 * TranslateRankName (NPC_stats.c:292) — pure rank-name string -> rank_t enum
 * match. Verbatim body; the `rank_t` enum (ai.h:31 //# rank_e) is reproduced here
 * with its authentic ordinals so the returned ints match the Rust b_public_h.rs
 * RANK_* constants. Q_stricmp lives in q_shared_oracle.c (same static lib); declared
 * extern and reused. Renamed jka_TranslateRankName and made non-static for linkage. */

typedef enum /*# rank_e*/
{
	RANK_CIVILIAN,	/* 0 */
	RANK_CREWMAN,	/* 1 */
	RANK_ENSIGN,	/* 2 */
	RANK_LT_JG,		/* 3 */
	RANK_LT,		/* 4 */
	RANK_LT_COMM,	/* 5 */
	RANK_COMMANDER,	/* 6 */
	RANK_CAPTAIN	/* 7 */
} rank_t;

int Q_stricmp (const char *s1, const char *s2);

rank_t jka_TranslateRankName( const char *name )
{
	if ( !Q_stricmp( name, "civilian" ) )
	{
		return RANK_CIVILIAN;
	}

	if ( !Q_stricmp( name, "crewman" ) )
	{
		return RANK_CREWMAN;
	}

	if ( !Q_stricmp( name, "ensign" ) )
	{
		return RANK_ENSIGN;
	}

	if ( !Q_stricmp( name, "ltjg" ) )
	{
		return RANK_LT_JG;
	}

	if ( !Q_stricmp( name, "lt" ) )
	{
		return RANK_LT;
	}

	if ( !Q_stricmp( name, "ltcomm" ) )
	{
		return RANK_LT_COMM;
	}

	if ( !Q_stricmp( name, "commander" ) )
	{
		return RANK_COMMANDER;
	}

	if ( !Q_stricmp( name, "captain" ) )
	{
		return RANK_CAPTAIN;
	}

	return RANK_CIVILIAN;
}
