/*
 * Oracle TU for the bg_public.h shared enums. Each enum body is copied
 * VERBATIM (preserving every `= N` assignment) so the C compiler computes the
 * enumerator values independently of the Rust port; the accessors expose a
 * spread of checkpoints (first, last/terminal, every explicit `=`, plus a few
 * interior points for the large enums) for the parity test to assert bit-exact.
 * Generated from bg_public.h. Built only under the `oracle` cargo feature.
 *
 * All enumerators across these enums are globally unique (they coexist in the
 * single real bg_public.h translation unit), so dumping the bodies as separate
 * anonymous enums here is collision-free.
 */

/* g2ModelParts_t */
enum {

	G2_MODELPART_HEAD = 10,
	G2_MODELPART_WAIST,
	G2_MODELPART_LARM,
	G2_MODELPART_RARM,
	G2_MODELPART_RHAND,
	G2_MODELPART_LLEG,
	G2_MODELPART_RLEG
};
int jka_bge_G2_MODELPART_HEAD(void) { return G2_MODELPART_HEAD; }
int jka_bge_G2_MODELPART_RLEG(void) { return G2_MODELPART_RLEG; }

/* forceHandAnims_t */
enum {

	HANDEXTEND_NONE = 0,
	HANDEXTEND_FORCEPUSH,
	HANDEXTEND_FORCEPULL,
	HANDEXTEND_FORCE_HOLD,
	HANDEXTEND_SABERPULL,
	HANDEXTEND_CHOKE, //use handextend priorities to choke someone being gripped
	HANDEXTEND_WEAPONREADY,
	HANDEXTEND_DODGE,
	HANDEXTEND_KNOCKDOWN,
	HANDEXTEND_DUELCHALLENGE,
	HANDEXTEND_TAUNT,

	HANDEXTEND_PRETHROW,
	HANDEXTEND_POSTTHROW,
	HANDEXTEND_PRETHROWN,
	HANDEXTEND_POSTTHROWN,

	HANDEXTEND_DRAGGING,

	HANDEXTEND_JEDITAUNT,
};
int jka_bge_HANDEXTEND_NONE(void) { return HANDEXTEND_NONE; }
int jka_bge_HANDEXTEND_JEDITAUNT(void) { return HANDEXTEND_JEDITAUNT; }

/* brokenLimb_t */
enum {

	BROKENLIMB_NONE = 0,
	BROKENLIMB_LARM,
	BROKENLIMB_RARM,
	NUM_BROKENLIMBS
};
int jka_bge_BROKENLIMB_NONE(void) { return BROKENLIMB_NONE; }
int jka_bge_NUM_BROKENLIMBS(void) { return NUM_BROKENLIMBS; }

/* gametype_t */
enum {

	GT_FFA,				// free for all
	GT_HOLOCRON,		// holocron ffa
	GT_JEDIMASTER,		// jedi master
	GT_DUEL,		// one on one tournament
	GT_POWERDUEL,
	GT_SINGLE_PLAYER,	// single player ffa

	//-- team games go after this --

	GT_TEAM,			// team deathmatch
	GT_SIEGE,			// siege
	GT_CTF,				// capture the flag
	GT_CTY,
	GT_MAX_GAME_TYPE
};
int jka_bge_GT_FFA(void) { return GT_FFA; }
int jka_bge_GT_MAX_GAME_TYPE(void) { return GT_MAX_GAME_TYPE; }

/* gender_t */
enum {
 GENDER_MALE, GENDER_FEMALE, GENDER_NEUTER 
};
int jka_bge_GENDER_MALE(void) { return GENDER_MALE; }
int jka_bge_GENDER_NEUTER(void) { return GENDER_NEUTER; }

/* anon_L211 */
enum {

	SABERLOCK_TOP,
	SABERLOCK_SIDE,
	SABERLOCK_LOCK,
	SABERLOCK_BREAK,
	SABERLOCK_SUPERBREAK,
	SABERLOCK_WIN,
	SABERLOCK_LOSE
};
int jka_bge_SABERLOCK_TOP(void) { return SABERLOCK_TOP; }
int jka_bge_SABERLOCK_LOSE(void) { return SABERLOCK_LOSE; }

/* anon_L222 */
enum {

	DIR_RIGHT,
	DIR_LEFT,
	DIR_FRONT,
	DIR_BACK
};
int jka_bge_DIR_RIGHT(void) { return DIR_RIGHT; }
int jka_bge_DIR_BACK(void) { return DIR_BACK; }

/* footstepType_t */
enum {

	FOOTSTEP_R,
	FOOTSTEP_L,
	FOOTSTEP_HEAVY_R,
	FOOTSTEP_HEAVY_L,
	NUM_FOOTSTEP_TYPES
};
int jka_bge_FOOTSTEP_R(void) { return FOOTSTEP_R; }
int jka_bge_NUM_FOOTSTEP_TYPES(void) { return NUM_FOOTSTEP_TYPES; }

/* animEventType_t */
enum {
//NOTENOTE:  Be sure to update animEventTypeTable and ParseAnimationEvtBlock(...) if you change this enum list!
	AEV_NONE,
	AEV_SOUND,		//# animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
	AEV_FOOTSTEP,	//# animID AEV_FOOTSTEP framenum footstepType chancetoplay
	AEV_EFFECT,		//# animID AEV_EFFECT framenum effectpath boltName chancetoplay
	AEV_FIRE,		//# animID AEV_FIRE framenum altfire chancetofire
	AEV_MOVE,		//# animID AEV_MOVE framenum forwardpush rightpush uppush
	AEV_SOUNDCHAN,  //# animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
	AEV_SABER_SWING,  //# animID AEV_SABER_SWING framenum CHANNEL randomlow randomhi chancetoplay
	AEV_SABER_SPIN,  //# animID AEV_SABER_SPIN framenum CHANNEL chancetoplay
	AEV_NUM_AEV
};
int jka_bge_AEV_NONE(void) { return AEV_NONE; }
int jka_bge_AEV_NUM_AEV(void) { return AEV_NUM_AEV; }

/* pmtype_t */
enum {

	PM_NORMAL,		// can accelerate and turn
	PM_JETPACK,		// special jetpack movement
	PM_FLOAT,		// float with no gravity in general direction of velocity (intended for gripping)
	PM_NOCLIP,		// noclip movement
	PM_SPECTATOR,	// still run into walls
	PM_DEAD,		// no acceleration or turning, but free falling
	PM_FREEZE,		// stuck in place with no control
	PM_INTERMISSION,	// no movement or status bar
	PM_SPINTERMISSION	// no movement or status bar
};
int jka_bge_PM_NORMAL(void) { return PM_NORMAL; }
int jka_bge_PM_SPINTERMISSION(void) { return PM_SPINTERMISSION; }

/* weaponstate_t */
enum {

	WEAPON_READY, 
	WEAPON_RAISING,
	WEAPON_DROPPING,
	WEAPON_FIRING,
	WEAPON_CHARGING,
	WEAPON_CHARGING_ALT,
	WEAPON_IDLE, //lowered		// NOTENOTE Added with saber
};
int jka_bge_WEAPON_READY(void) { return WEAPON_READY; }
int jka_bge_WEAPON_IDLE(void) { return WEAPON_IDLE; }

/* anon_L376 */
enum {

	FORCE_MASTERY_UNINITIATED,
	FORCE_MASTERY_INITIATE,
	FORCE_MASTERY_PADAWAN,
	FORCE_MASTERY_JEDI,
	FORCE_MASTERY_JEDI_GUARDIAN,
	FORCE_MASTERY_JEDI_ADEPT,
	FORCE_MASTERY_JEDI_KNIGHT,
	FORCE_MASTERY_JEDI_MASTER,
	NUM_FORCE_MASTERY_LEVELS
};
int jka_bge_FORCE_MASTERY_UNINITIATED(void) { return FORCE_MASTERY_UNINITIATED; }
int jka_bge_NUM_FORCE_MASTERY_LEVELS(void) { return NUM_FORCE_MASTERY_LEVELS; }

/* statIndex_t */
enum {

	STAT_HEALTH,
	STAT_HOLDABLE_ITEM,
	STAT_HOLDABLE_ITEMS,
	STAT_PERSISTANT_POWERUP,
	//MAKE SURE STAT_WEAPONS REMAINS 4!!!!
	//There is a hardcoded reference in msg.cpp to send it in 32 bits -rww
	STAT_WEAPONS = 4,					// 16 bit fields
	STAT_ARMOR,				
	STAT_DEAD_YAW,					// look this direction when dead (FIXME: get rid of?)
	STAT_CLIENTS_READY,				// bit mask of clients wishing to exit the intermission (FIXME: configstring?)
	STAT_MAX_HEALTH					// health / armor limit, changable by handicap
};
int jka_bge_STAT_HEALTH(void) { return STAT_HEALTH; }
int jka_bge_STAT_WEAPONS(void) { return STAT_WEAPONS; }
int jka_bge_STAT_MAX_HEALTH(void) { return STAT_MAX_HEALTH; }

/* persEnum_t */
enum {

	PERS_SCORE,						// !!! MUST NOT CHANGE, SERVER AND GAME BOTH REFERENCE !!!
	PERS_HITS,						// total points damage inflicted so damage beeps can sound on change
	PERS_RANK,						// player rank or team rank
	PERS_TEAM,						// player team
	PERS_SPAWN_COUNT,				// incremented every respawn
	PERS_PLAYEREVENTS,				// 16 bits that can be flipped for events
	PERS_ATTACKER,					// clientnum of last damage inflicter
	PERS_ATTACKEE_ARMOR,			// health/armor of last person we attacked
	PERS_KILLED,					// count of the number of times you died
	// player awards tracking
	PERS_IMPRESSIVE_COUNT,			// two railgun hits in a row
	PERS_EXCELLENT_COUNT,			// two successive kills in a short amount of time
	PERS_DEFEND_COUNT,				// defend awards
	PERS_ASSIST_COUNT,				// assist awards
	PERS_GAUNTLET_FRAG_COUNT,		// kills with the guantlet
	PERS_CAPTURES					// captures
};
int jka_bge_PERS_SCORE(void) { return PERS_SCORE; }
int jka_bge_PERS_CAPTURES(void) { return PERS_CAPTURES; }

/* effectTypes_t */
enum {

	EFFECT_NONE = 0,
	EFFECT_SMOKE,
	EFFECT_EXPLOSION,
	EFFECT_EXPLOSION_PAS,
	EFFECT_SPARK_EXPLOSION,
	EFFECT_EXPLOSION_TRIPMINE,
	EFFECT_EXPLOSION_DETPACK,
	EFFECT_EXPLOSION_FLECHETTE,
	EFFECT_STUNHIT,
	EFFECT_EXPLOSION_DEMP2ALT,
	EFFECT_EXPLOSION_TURRET,
	EFFECT_SPARKS,
	EFFECT_WATER_SPLASH,
	EFFECT_ACID_SPLASH,
	EFFECT_LAVA_SPLASH,
	EFFECT_LANDING_MUD,
	EFFECT_LANDING_SAND,
	EFFECT_LANDING_DIRT,
	EFFECT_LANDING_SNOW,
	EFFECT_LANDING_GRAVEL,
	EFFECT_MAX
};
int jka_bge_EFFECT_NONE(void) { return EFFECT_NONE; }
int jka_bge_EFFECT_MAX(void) { return EFFECT_MAX; }

/* powerup_t */
enum {

	PW_NONE,

	PW_QUAD,
	PW_BATTLESUIT,
	PW_PULL,
	//PW_INVIS, //rww - removed
	//PW_REGEN, //rww - removed
	//PW_FLIGHT, //rww - removed

	PW_REDFLAG,
	PW_BLUEFLAG,
	PW_NEUTRALFLAG,

	PW_SHIELDHIT,

	//PW_SCOUT, //rww - removed
	//PW_GUARD, //rww - removed
	//PW_DOUBLER, //rww - removed
	//PW_AMMOREGEN, //rww - removed
	PW_SPEEDBURST,
	PW_DISINT_4,
	PW_SPEED,
	PW_CLOAKED,
	PW_FORCE_ENLIGHTENED_LIGHT,
	PW_FORCE_ENLIGHTENED_DARK,
	PW_FORCE_BOON,
	PW_YSALAMIRI,

	PW_NUM_POWERUPS
};
int jka_bge_PW_NONE(void) { return PW_NONE; }
int jka_bge_PW_NUM_POWERUPS(void) { return PW_NUM_POWERUPS; }

/* holdable_t */
enum {

	HI_NONE,

	HI_SEEKER,
	HI_SHIELD,
	HI_MEDPAC,
	HI_MEDPAC_BIG,
	HI_BINOCULARS,
	HI_SENTRY_GUN,
	HI_JETPACK,

	HI_HEALTHDISP,
	HI_AMMODISP,
	HI_EWEB,
	HI_CLOAK,

	HI_NUM_HOLDABLE
};
int jka_bge_HI_NONE(void) { return HI_NONE; }
int jka_bge_HI_NUM_HOLDABLE(void) { return HI_NUM_HOLDABLE; }

/* ctfMsg_t */
enum {

	CTFMESSAGE_FRAGGED_FLAG_CARRIER,
	CTFMESSAGE_FLAG_RETURNED,
	CTFMESSAGE_PLAYER_RETURNED_FLAG,
	CTFMESSAGE_PLAYER_CAPTURED_FLAG,
	CTFMESSAGE_PLAYER_GOT_FLAG
};
int jka_bge_CTFMESSAGE_FRAGGED_FLAG_CARRIER(void) { return CTFMESSAGE_FRAGGED_FLAG_CARRIER; }
int jka_bge_CTFMESSAGE_PLAYER_GOT_FLAG(void) { return CTFMESSAGE_PLAYER_GOT_FLAG; }

/* pdSounds_t */
enum {

	PDSOUND_NONE,
	PDSOUND_PROTECTHIT,
	PDSOUND_PROTECT,
	PDSOUND_ABSORBHIT,
	PDSOUND_ABSORB,
	PDSOUND_FORCEJUMP,
	PDSOUND_FORCEGRIP
};
int jka_bge_PDSOUND_NONE(void) { return PDSOUND_NONE; }
int jka_bge_PDSOUND_FORCEGRIP(void) { return PDSOUND_FORCEGRIP; }

/* entity_event_t */
enum {

	EV_NONE,

	EV_CLIENTJOIN,

	EV_FOOTSTEP,
	EV_FOOTSTEP_METAL,
	EV_FOOTSPLASH,
	EV_FOOTWADE,
	EV_SWIM,

	EV_STEP_4,
	EV_STEP_8,
	EV_STEP_12,
	EV_STEP_16,

	EV_FALL,

	EV_JUMP_PAD,			// boing sound at origin, jump sound on player

	EV_GHOUL2_MARK,			//create a projectile impact mark on something with a client-side g2 instance.

	EV_GLOBAL_DUEL,
	EV_PRIVATE_DUEL,

	EV_JUMP,
	EV_ROLL,
	EV_WATER_TOUCH,	// foot touches
	EV_WATER_LEAVE,	// foot leaves
	EV_WATER_UNDER,	// head touches
	EV_WATER_CLEAR,	// head leaves

	EV_ITEM_PICKUP,			// normal item pickups are predictable
	EV_GLOBAL_ITEM_PICKUP,	// powerup / team sounds are broadcast to everyone

	EV_VEH_FIRE,

	EV_NOAMMO,
	EV_CHANGE_WEAPON,
	EV_FIRE_WEAPON,
	EV_ALT_FIRE,
	EV_SABER_ATTACK,
	EV_SABER_HIT,
	EV_SABER_BLOCK,
	EV_SABER_CLASHFLARE,
	EV_SABER_UNHOLSTER,
	EV_BECOME_JEDIMASTER,
	EV_DISRUPTOR_MAIN_SHOT,
	EV_DISRUPTOR_SNIPER_SHOT,
	EV_DISRUPTOR_SNIPER_MISS,
	EV_DISRUPTOR_HIT,
	EV_DISRUPTOR_ZOOMSOUND,

	EV_PREDEFSOUND,

	EV_TEAM_POWER,

	EV_SCREENSHAKE,

	EV_LOCALTIMER,

	EV_USE,			// +Use key

	EV_USE_ITEM0,
	EV_USE_ITEM1,
	EV_USE_ITEM2,
	EV_USE_ITEM3,
	EV_USE_ITEM4,
	EV_USE_ITEM5,
	EV_USE_ITEM6,
	EV_USE_ITEM7,
	EV_USE_ITEM8,
	EV_USE_ITEM9,
	EV_USE_ITEM10,
	EV_USE_ITEM11,
	EV_USE_ITEM12,
	EV_USE_ITEM13,
	EV_USE_ITEM14,
	EV_USE_ITEM15,

	EV_ITEMUSEFAIL,

	EV_ITEM_RESPAWN,
	EV_ITEM_POP,
	EV_PLAYER_TELEPORT_IN,
	EV_PLAYER_TELEPORT_OUT,

	EV_GRENADE_BOUNCE,		// eventParm will be the soundindex
	EV_MISSILE_STICK,		// eventParm will be the soundindex

	EV_PLAY_EFFECT,
	EV_PLAY_EFFECT_ID,
	EV_PLAY_PORTAL_EFFECT_ID,

	EV_PLAYDOORSOUND,
	EV_PLAYDOORLOOPSOUND,
	EV_BMODEL_SOUND,

	EV_MUTE_SOUND,
	EV_VOICECMD_SOUND,
	EV_GENERAL_SOUND,
	EV_GLOBAL_SOUND,		// no attenuation
	EV_GLOBAL_TEAM_SOUND,
	EV_ENTITY_SOUND,

	EV_PLAY_ROFF,

	EV_GLASS_SHATTER,
	EV_DEBRIS,
	EV_MISC_MODEL_EXP,

	EV_CONC_ALT_IMPACT,

	EV_MISSILE_HIT,
	EV_MISSILE_MISS,
	EV_MISSILE_MISS_METAL,
	EV_BULLET,				// otherEntity is the shooter

	EV_PAIN,
	EV_DEATH1,
	EV_DEATH2,
	EV_DEATH3,
	EV_OBITUARY,

	EV_POWERUP_QUAD,
	EV_POWERUP_BATTLESUIT,
	//EV_POWERUP_REGEN,

	EV_FORCE_DRAINED,

	EV_GIB_PLAYER,			// gib a previously living player
	EV_SCOREPLUM,			// score plum

	EV_CTFMESSAGE,

	EV_BODYFADE,

	EV_SIEGE_ROUNDOVER,
	EV_SIEGE_OBJECTIVECOMPLETE,

	EV_DESTROY_GHOUL2_INSTANCE,

	EV_DESTROY_WEAPON_MODEL,

	EV_GIVE_NEW_RANK,
	EV_SET_FREE_SABER,
	EV_SET_FORCE_DISABLE,

	EV_WEAPON_CHARGE,
	EV_WEAPON_CHARGE_ALT,

	EV_SHIELD_HIT,

	EV_DEBUG_LINE,
	EV_TESTLINE,
	EV_STOPLOOPINGSOUND,
	EV_STARTLOOPINGSOUND,
	EV_TAUNT,

	//rww - Begin NPC sound events
	EV_ANGER1,	//Say when acquire an enemy when didn't have one before
	EV_ANGER2,
	EV_ANGER3,

	EV_VICTORY1,	//Say when killed an enemy
	EV_VICTORY2,
	EV_VICTORY3,

	EV_CONFUSE1,	//Say when confused
	EV_CONFUSE2,
	EV_CONFUSE3,

	EV_PUSHED1,		//Say when pushed
	EV_PUSHED2,
	EV_PUSHED3,

	EV_CHOKE1,		//Say when choking
	EV_CHOKE2,
	EV_CHOKE3,

	EV_FFWARN,		//ffire founds
	EV_FFTURN,
	//extra sounds for ST
	EV_CHASE1,
	EV_CHASE2,
	EV_CHASE3,
	EV_COVER1,
	EV_COVER2,
	EV_COVER3,
	EV_COVER4,
	EV_COVER5,
	EV_DETECTED1,
	EV_DETECTED2,
	EV_DETECTED3,
	EV_DETECTED4,
	EV_DETECTED5,
	EV_LOST1,
	EV_OUTFLANK1,
	EV_OUTFLANK2,
	EV_ESCAPING1,
	EV_ESCAPING2,
	EV_ESCAPING3,
	EV_GIVEUP1,
	EV_GIVEUP2,
	EV_GIVEUP3,
	EV_GIVEUP4,
	EV_LOOK1,
	EV_LOOK2,
	EV_SIGHT1,
	EV_SIGHT2,
	EV_SIGHT3,
	EV_SOUND1,
	EV_SOUND2,
	EV_SOUND3,
	EV_SUSPICIOUS1,
	EV_SUSPICIOUS2,
	EV_SUSPICIOUS3,
	EV_SUSPICIOUS4,
	EV_SUSPICIOUS5,
	//extra sounds for Jedi
	EV_COMBAT1,
	EV_COMBAT2,
	EV_COMBAT3,
	EV_JDETECTED1,
	EV_JDETECTED2,
	EV_JDETECTED3,
	EV_TAUNT1,
	EV_TAUNT2,
	EV_TAUNT3,
	EV_JCHASE1,
	EV_JCHASE2,
	EV_JCHASE3,
	EV_JLOST1,
	EV_JLOST2,
	EV_JLOST3,
	EV_DEFLECT1,
	EV_DEFLECT2,
	EV_DEFLECT3,
	EV_GLOAT1,
	EV_GLOAT2,
	EV_GLOAT3,
	EV_PUSHFAIL,

	EV_SIEGESPEC,
	
};
int jka_bge_EV_NONE(void) { return EV_NONE; }
int jka_bge_EV_USE_ITEM5(void) { return EV_USE_ITEM5; }
int jka_bge_EV_BODYFADE(void) { return EV_BODYFADE; }
int jka_bge_EV_ESCAPING2(void) { return EV_ESCAPING2; }
int jka_bge_EV_SIEGESPEC(void) { return EV_SIEGESPEC; }

/* global_team_sound_t */
enum {

	GTS_RED_CAPTURE,
	GTS_BLUE_CAPTURE,
	GTS_RED_RETURN,
	GTS_BLUE_RETURN,
	GTS_RED_TAKEN,
	GTS_BLUE_TAKEN,
	GTS_REDTEAM_SCORED,
	GTS_BLUETEAM_SCORED,
	GTS_REDTEAM_TOOK_LEAD,
	GTS_BLUETEAM_TOOK_LEAD,
	GTS_TEAMS_ARE_TIED
};
int jka_bge_GTS_RED_CAPTURE(void) { return GTS_RED_CAPTURE; }
int jka_bge_GTS_TEAMS_ARE_TIED(void) { return GTS_TEAMS_ARE_TIED; }

/* team_t */
enum {

	TEAM_FREE,
	TEAM_RED,
	TEAM_BLUE,
	TEAM_SPECTATOR,

	TEAM_NUM_TEAMS
};
int jka_bge_TEAM_FREE(void) { return TEAM_FREE; }
int jka_bge_TEAM_NUM_TEAMS(void) { return TEAM_NUM_TEAMS; }

/* duelTeam_t */
enum {

	DUELTEAM_FREE,
	DUELTEAM_LONE,
	DUELTEAM_DOUBLE,

	DUELTEAM_SINGLE,		// for regular duel matches (not power duel)
};
int jka_bge_DUELTEAM_FREE(void) { return DUELTEAM_FREE; }
int jka_bge_DUELTEAM_SINGLE(void) { return DUELTEAM_SINGLE; }

/* teamtask_t */
enum {

	TEAMTASK_NONE,
	TEAMTASK_OFFENSE, 
	TEAMTASK_DEFENSE,
	TEAMTASK_PATROL,
	TEAMTASK_FOLLOW,
	TEAMTASK_RETRIEVE,
	TEAMTASK_ESCORT,
	TEAMTASK_CAMP
};
int jka_bge_TEAMTASK_NONE(void) { return TEAMTASK_NONE; }
int jka_bge_TEAMTASK_CAMP(void) { return TEAMTASK_CAMP; }

/* meansOfDeath_t */
enum {

	MOD_UNKNOWN,
	MOD_STUN_BATON,
	MOD_MELEE,
	MOD_SABER,
	MOD_BRYAR_PISTOL,
	MOD_BRYAR_PISTOL_ALT,
	MOD_BLASTER,
	MOD_TURBLAST,
	MOD_DISRUPTOR,
	MOD_DISRUPTOR_SPLASH,
	MOD_DISRUPTOR_SNIPER,
	MOD_BOWCASTER,
	MOD_REPEATER,
	MOD_REPEATER_ALT,
	MOD_REPEATER_ALT_SPLASH,
	MOD_DEMP2,
	MOD_DEMP2_ALT,
	MOD_FLECHETTE,
	MOD_FLECHETTE_ALT_SPLASH,
	MOD_ROCKET,
	MOD_ROCKET_SPLASH,
	MOD_ROCKET_HOMING,
	MOD_ROCKET_HOMING_SPLASH,
	MOD_THERMAL,
	MOD_THERMAL_SPLASH,
	MOD_TRIP_MINE_SPLASH,
	MOD_TIMED_MINE_SPLASH,
	MOD_DET_PACK_SPLASH,
	MOD_VEHICLE,
	MOD_CONC,
	MOD_CONC_ALT,
	MOD_FORCE_DARK,
	MOD_SENTRY,
	MOD_WATER,
	MOD_SLIME,
	MOD_LAVA,
	MOD_CRUSH,
	MOD_TELEFRAG,
	MOD_FALLING,
	MOD_COLLISION,
	MOD_VEH_EXPLOSION,
	MOD_SUICIDE,
	MOD_TARGET_LASER,
	MOD_TRIGGER_HURT,
	MOD_TEAM_CHANGE,
	//AURELIO: when/if you put this back in, remember to make a case for it in all the other places where
	//mod's are checked. Also, it probably isn't the most elegant solution for what you want - just add
	//a frag back to the player after you call the player_die (and keep a local of his pre-death score to
	//make sure he actually lost points, there may be cases where you don't lose points on changing teams
	//or suiciding, and so you would actually be giving him a point) -Rich
	// I put it back in for now, if it becomes a problem we'll work around it later (it shouldn't though)...
	MOD_MAX
};
int jka_bge_MOD_UNKNOWN(void) { return MOD_UNKNOWN; }
int jka_bge_MOD_MAX(void) { return MOD_MAX; }

/* itemType_t */
enum {

	IT_BAD,
	IT_WEAPON,				// EFX: rotate + upscale + minlight
	IT_AMMO,				// EFX: rotate
	IT_ARMOR,				// EFX: rotate + minlight
	IT_HEALTH,				// EFX: static external sphere + rotating internal
	IT_POWERUP,				// instant on, timer based
							// EFX: rotate + external ring that rotates
	IT_HOLDABLE,			// single use, holdable item
							// EFX: rotate + bob
	IT_PERSISTANT_POWERUP,
	IT_TEAM
};
int jka_bge_IT_BAD(void) { return IT_BAD; }
int jka_bge_IT_TEAM(void) { return IT_TEAM; }

/* entityType_t */
enum {

	ET_GENERAL,
	ET_PLAYER,
	ET_ITEM,
	ET_MISSILE,
	ET_SPECIAL,				// rww - force fields
	ET_HOLOCRON,			// rww - holocron icon displays
	ET_MOVER,
	ET_BEAM,
	ET_PORTAL,
	ET_SPEAKER,
	ET_PUSH_TRIGGER,
	ET_TELEPORT_TRIGGER,
	ET_INVISIBLE,
	ET_NPC,					// ghoul2 player-like entity
	ET_TEAM,
	ET_BODY,
	ET_TERRAIN,
	ET_FX,

	ET_EVENTS				// any of the EV_* events can be added freestanding
							// by setting eType to ET_EVENTS + eventNum
							// this avoids having to set eFlags and eventNum
};
int jka_bge_ET_GENERAL(void) { return ET_GENERAL; }
int jka_bge_ET_EVENTS(void) { return ET_EVENTS; }

/* fieldtype_t */
enum {

	F_INT, 
	F_FLOAT,
	F_LSTRING,			// string on disk, pointer in memory, TAG_LEVEL
	F_GSTRING,			// string on disk, pointer in memory, TAG_GAME
	F_VECTOR,
	F_ANGLEHACK,
	F_ENTITY,			// index on disk, pointer in memory
	F_ITEM,				// index on disk, pointer in memory
	F_CLIENT,			// index on disk, pointer in memory
	F_PARM1,			// Special case for parms
	F_PARM2,			// Special case for parms
	F_PARM3,			// Special case for parms
	F_PARM4,			// Special case for parms
	F_PARM5,			// Special case for parms
	F_PARM6,			// Special case for parms
	F_PARM7,			// Special case for parms
	F_PARM8,			// Special case for parms
	F_PARM9,			// Special case for parms
	F_PARM10,			// Special case for parms
	F_PARM11,			// Special case for parms
	F_PARM12,			// Special case for parms
	F_PARM13,			// Special case for parms
	F_PARM14,			// Special case for parms
	F_PARM15,			// Special case for parms
	F_PARM16,			// Special case for parms
	F_IGNORE
};
int jka_bge_F_INT(void) { return F_INT; }
int jka_bge_F_IGNORE(void) { return F_IGNORE; }

/* saberMoveName_t */
enum {

	// Invalid, or saber not armed
	LS_NONE		= 0,

	// General movements with saber
	LS_READY,
	LS_DRAW,
	LS_PUTAWAY,

	// Attacks
	LS_A_TL2BR,//4
	LS_A_L2R,
	LS_A_BL2TR,
	LS_A_BR2TL,
	LS_A_R2L,
	LS_A_TR2BL,
	LS_A_T2B,
	LS_A_BACKSTAB,
	LS_A_BACK,
	LS_A_BACK_CR,
	LS_ROLL_STAB,
	LS_A_LUNGE,
	LS_A_JUMP_T__B_,
	LS_A_FLIP_STAB,
	LS_A_FLIP_SLASH,
	LS_JUMPATTACK_DUAL,
	LS_JUMPATTACK_ARIAL_LEFT,
	LS_JUMPATTACK_ARIAL_RIGHT,
	LS_JUMPATTACK_CART_LEFT,
	LS_JUMPATTACK_CART_RIGHT,
	LS_JUMPATTACK_STAFF_LEFT,
	LS_JUMPATTACK_STAFF_RIGHT,
	LS_BUTTERFLY_LEFT,
	LS_BUTTERFLY_RIGHT,
	LS_A_BACKFLIP_ATK,
	LS_SPINATTACK_DUAL,
	LS_SPINATTACK,
	LS_LEAP_ATTACK,
	LS_SWOOP_ATTACK_RIGHT,
	LS_SWOOP_ATTACK_LEFT,
	LS_TAUNTAUN_ATTACK_RIGHT,
	LS_TAUNTAUN_ATTACK_LEFT,
	LS_KICK_F,
	LS_KICK_B,
	LS_KICK_R,
	LS_KICK_L,
	LS_KICK_S,
	LS_KICK_BF,
	LS_KICK_RL,
	LS_KICK_F_AIR,
	LS_KICK_B_AIR,
	LS_KICK_R_AIR,
	LS_KICK_L_AIR,
	LS_STABDOWN,
	LS_STABDOWN_STAFF,
	LS_STABDOWN_DUAL,
	LS_DUAL_SPIN_PROTECT,
	LS_STAFF_SOULCAL,
	LS_A1_SPECIAL,
	LS_A2_SPECIAL,
	LS_A3_SPECIAL,
	LS_UPSIDE_DOWN_ATTACK,
	LS_PULL_ATTACK_STAB,
	LS_PULL_ATTACK_SWING,
	LS_SPINATTACK_ALORA,
	LS_DUAL_FB,
	LS_DUAL_LR,
	LS_HILT_BASH,

	//starts
	LS_S_TL2BR,//26
	LS_S_L2R,
	LS_S_BL2TR,//# Start of attack chaining to SLASH LR2UL
	LS_S_BR2TL,//# Start of attack chaining to SLASH LR2UL
	LS_S_R2L,
	LS_S_TR2BL,
	LS_S_T2B,

	//returns
	LS_R_TL2BR,//33
	LS_R_L2R,
	LS_R_BL2TR,
	LS_R_BR2TL,
	LS_R_R2L,
	LS_R_TR2BL,
	LS_R_T2B,

	//transitions
	LS_T1_BR__R,//40
	LS_T1_BR_TR,
	LS_T1_BR_T_,
	LS_T1_BR_TL,
	LS_T1_BR__L,
	LS_T1_BR_BL,
	LS_T1__R_BR,//46
	LS_T1__R_TR,
	LS_T1__R_T_,
	LS_T1__R_TL,
	LS_T1__R__L,
	LS_T1__R_BL,
	LS_T1_TR_BR,//52
	LS_T1_TR__R,
	LS_T1_TR_T_,
	LS_T1_TR_TL,
	LS_T1_TR__L,
	LS_T1_TR_BL,
	LS_T1_T__BR,//58
	LS_T1_T___R,
	LS_T1_T__TR,
	LS_T1_T__TL,
	LS_T1_T___L,
	LS_T1_T__BL,
	LS_T1_TL_BR,//64
	LS_T1_TL__R,
	LS_T1_TL_TR,
	LS_T1_TL_T_,
	LS_T1_TL__L,
	LS_T1_TL_BL,
	LS_T1__L_BR,//70
	LS_T1__L__R,
	LS_T1__L_TR,
	LS_T1__L_T_,
	LS_T1__L_TL,
	LS_T1__L_BL,
	LS_T1_BL_BR,//76
	LS_T1_BL__R,
	LS_T1_BL_TR,
	LS_T1_BL_T_,
	LS_T1_BL_TL,
	LS_T1_BL__L,

	//Bounces
	LS_B1_BR,
	LS_B1__R,
	LS_B1_TR,
	LS_B1_T_,
	LS_B1_TL,
	LS_B1__L,
	LS_B1_BL,

	//Deflected attacks
	LS_D1_BR,
	LS_D1__R,
	LS_D1_TR,
	LS_D1_T_,
	LS_D1_TL,
	LS_D1__L,
	LS_D1_BL,
	LS_D1_B_,

	//Reflected attacks
	LS_V1_BR,
	LS_V1__R,
	LS_V1_TR,
	LS_V1_T_,
	LS_V1_TL,
	LS_V1__L,
	LS_V1_BL,
	LS_V1_B_,

	// Broken parries
	LS_H1_T_,//
	LS_H1_TR,
	LS_H1_TL,
	LS_H1_BR,
	LS_H1_B_,
	LS_H1_BL,

	// Knockaways
	LS_K1_T_,//
	LS_K1_TR,
	LS_K1_TL,
	LS_K1_BR,
	LS_K1_BL,

	// Parries
	LS_PARRY_UP,//
	LS_PARRY_UR,
	LS_PARRY_UL,
	LS_PARRY_LR,
	LS_PARRY_LL,

	// Projectile Reflections
	LS_REFLECT_UP,//
	LS_REFLECT_UR,
	LS_REFLECT_UL,
	LS_REFLECT_LR,
	LS_REFLECT_LL,

	LS_MOVE_MAX//
};
int jka_bge_LS_NONE(void) { return LS_NONE; }
int jka_bge_LS_DUAL_SPIN_PROTECT(void) { return LS_DUAL_SPIN_PROTECT; }
int jka_bge_LS_T1_TL_BR(void) { return LS_T1_TL_BR; }
int jka_bge_LS_K1_BR(void) { return LS_K1_BR; }
int jka_bge_LS_MOVE_MAX(void) { return LS_MOVE_MAX; }

/* saberQuadrant_t */
enum {

	Q_BR,
	Q_R,
	Q_TR,
	Q_T,
	Q_TL,
	Q_L,
	Q_BL,
	Q_B,
	Q_NUM_QUADS
};
int jka_bge_Q_BR(void) { return Q_BR; }
int jka_bge_Q_NUM_QUADS(void) { return Q_NUM_QUADS; }

