/* Oracle extraction of NPC_spawn.c's pure weapon-bitmask lookup:
 *   NPC_WeaponsForTeam (refs/raven-jediacademy/codemp/game/NPC_spawn.c:516)
 *
 * Pure `switch(team)` of Q_stricmp/Q_strncmp returning (1 << WP_*) starting-weapon
 * bitmasks. The two string callees are reused from q_shared_oracle.c (same static lib);
 * declared extern here. The `weapon_t` (bg_weapons.h:7) and team (teams.h) enums are
 * reproduced with their authentic ordinals so returned masks match the Rust constants;
 * the SFB_* spawnflag defines come from b_local.h. Renamed `jka_` for linkage. */

typedef enum  // bg_weapons.h:7 weapon_t (only the ordinals this fn references matter)
{
	WP_NONE,            /* 0 */
	WP_STUN_BATON,      /* 1 */
	WP_MELEE,           /* 2 */
	WP_SABER,           /* 3 */
	WP_BRYAR_PISTOL,    /* 4 */
	WP_BLASTER,         /* 5 */
	WP_DISRUPTOR,       /* 6 */
	WP_BOWCASTER,       /* 7 */
	WP_REPEATER,        /* 8 */
	WP_DEMP2,           /* 9 */
	WP_FLECHETTE,       /* 10 */
	WP_ROCKET_LAUNCHER, /* 11 */
	WP_THERMAL,         /* 12 */
	WP_NUM_WEAPONS_PLACEHOLDER
} weapon_t;

typedef int npcteam_t;       // teams.h:14
typedef npcteam_t team_t;    // NPC_spawn.c uses team_t for npcteam_t
#define NPCTEAM_FREE     0
#define NPCTEAM_ENEMY    1
#define NPCTEAM_PLAYER   2
#define NPCTEAM_NEUTRAL  3

#define SFB_RIFLEMAN  2   // b_local.h:139
#define SFB_PHASER    4   // b_local.h:141

int Q_stricmp (const char *s1, const char *s2);
int Q_strncmp (const char *string1, const char *string2, int count);

int jka_NPC_WeaponsForTeam( team_t team, int spawnflags, const char *NPC_type )
{
	//*** not sure how to handle this, should I pass in class instead of team and go from there? - dmv
	switch(team)
	{
	case NPCTEAM_ENEMY:
		if ( Q_stricmp( "tavion", NPC_type ) == 0 ||
			Q_strncmp( "reborn", NPC_type, 6 ) == 0 ||
			Q_stricmp( "desann", NPC_type ) == 0 ||
			Q_strncmp( "shadowtrooper", NPC_type, 13 ) == 0 )
			return ( 1 << WP_SABER);

		if ( Q_strncmp( "stofficer", NPC_type, 9 ) == 0 )
		{
			return ( 1 << WP_FLECHETTE);
		}
		if ( Q_stricmp( "stcommander", NPC_type ) == 0 )
		{
			return ( 1 << WP_REPEATER);
		}
		if ( Q_stricmp( "swamptrooper", NPC_type ) == 0 )
		{
			return ( 1 << WP_FLECHETTE);
		}
		if ( Q_stricmp( "swamptrooper2", NPC_type ) == 0 )
		{
			return ( 1 << WP_REPEATER);
		}
		if ( Q_stricmp( "rockettrooper", NPC_type ) == 0 )
		{
			return ( 1 << WP_ROCKET_LAUNCHER);
		}
		if ( Q_strncmp( "shadowtrooper", NPC_type, 13 ) == 0 )
		{
			return ( 1 << WP_SABER);
		}
		if ( Q_stricmp( "imperial", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_strncmp( "impworker", NPC_type, 9 ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_stricmp( "stormpilot", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_stricmp( "galak", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_stricmp( "galak_mech", NPC_type ) == 0 )
		{
			return ( 1 << WP_REPEATER);
		}
		if ( Q_strncmp( "ugnaught", NPC_type, 8 ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_stricmp( "granshooter", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_stricmp( "granboxer", NPC_type ) == 0 )
		{
			return ( 1 << WP_STUN_BATON);
		}
		if ( Q_strncmp( "gran", NPC_type, 4 ) == 0 )
		{
			return (( 1 << WP_THERMAL)|( 1 << WP_STUN_BATON));
		}
		if ( Q_stricmp( "rodian", NPC_type ) == 0 )
		{
			return ( 1 << WP_DISRUPTOR);
		}
		if ( Q_stricmp( "rodian2", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}

		if (( Q_stricmp( "interrogator",NPC_type) == 0) || ( Q_stricmp( "sentry",NPC_type) == 0) || (Q_strncmp( "protocol",NPC_type,8) == 0) )
		{
			return WP_NONE;
		}

		if ( Q_strncmp( "weequay", NPC_type, 7 ) == 0 )
		{
			return ( 1 << WP_BOWCASTER);
		}
		if ( Q_stricmp( "impofficer", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if ( Q_stricmp( "impcommander", NPC_type ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}
		if (( Q_stricmp( "probe", NPC_type ) == 0 ) || ( Q_stricmp( "seeker", NPC_type ) == 0 ))
		{
			return 0;
		}
		if ( Q_stricmp( "remote", NPC_type ) == 0 )
		{
			return 0;
		}
		if ( Q_stricmp( "trandoshan", NPC_type ) == 0 )
		{
			return (1<<WP_REPEATER);
		}
		if ( Q_stricmp( "atst", NPC_type ) == 0 )
		{
			return 0;
		}
		if ( Q_stricmp( "mark1", NPC_type ) == 0 )
		{
			return 0;
		}
		if ( Q_stricmp( "mark2", NPC_type ) == 0 )
		{
			return 0;
		}
		if ( Q_stricmp( "minemonster", NPC_type ) == 0 )
		{
			return (( 1 << WP_STUN_BATON));
		}
		if ( Q_stricmp( "howler", NPC_type ) == 0 )
		{
			return (( 1 << WP_STUN_BATON));
		}
		//Stormtroopers, etc.
		return ( 1 << WP_BLASTER);
		break;

	case NPCTEAM_PLAYER:
		if(spawnflags & SFB_RIFLEMAN)
			return ( 1 << WP_REPEATER);

		if(spawnflags & SFB_PHASER)
			return ( 1 << WP_BLASTER);

		if ( Q_strncmp( "jedi", NPC_type, 4 ) == 0 || Q_stricmp( "luke", NPC_type ) == 0 )
			return ( 1 << WP_SABER);

		if ( Q_strncmp( "prisoner", NPC_type, 8 ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_strncmp( "bespincop", NPC_type, 9 ) == 0 )
		{
			return ( 1 << WP_BLASTER);
		}

		if ( Q_stricmp( "MonMothma", NPC_type ) == 0 )
		{
			return WP_NONE;
		}

		//rebel
		return ( 1 << WP_BLASTER);
		break;

	case NPCTEAM_NEUTRAL:

		if ( Q_stricmp( "mark1", NPC_type ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_stricmp( "mark2", NPC_type ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_strncmp( "ugnaught", NPC_type, 8 ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_stricmp( "bartender", NPC_type ) == 0 )
		{
			return WP_NONE;
		}
		if ( Q_stricmp( "morgankatarn", NPC_type ) == 0 )
		{
			return WP_NONE;
		}

		break;

	default:
		break;
	}

	return WP_NONE;
}
