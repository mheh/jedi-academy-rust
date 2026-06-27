/*
 * Oracle TU for g_main.c's cvar registration table. Holds the cvar mirror
 * globals and `gameCvarTable[]` transcribed VERBATIM from
 * raven-jediacademy/codemp/game/g_main.c (the PC source of truth; build-config
 * #ifdefs kept intact, so clang -- compiled with no -D_DEBUG / -DFINAL_BUILD /
 * -DDEBUG_SABER_BOX -- resolves exactly the retail dedicated-server set the Rust
 * port targets). Exposes the real C table's size and each row's data fields so
 * `src/codemp/game/g_main.rs` can assert its hand-built `gameCvarTable` matches.
 *
 * The `vmCvar` pointer column is NOT parity-checked (Rust and C point at
 * different globals); only the count + name/default/flags/modificationCount/
 * trackChange/teamShader data is. The minimal vmCvar_t below exists only so the
 * `&global` initializers type-check; its layout is irrelevant here.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

typedef int qboolean;
#define qfalse 0
#define qtrue 1

/* cvar flags (verbatim subset from q_shared.h) referenced by the table */
#define CVAR_ARCHIVE     0x00000001
#define CVAR_USERINFO    0x00000002
#define CVAR_SERVERINFO  0x00000004
#define CVAR_SYSTEMINFO  0x00000008
#define CVAR_LATCH       0x00000020
#define CVAR_ROM         0x00000040
#define CVAR_CHEAT       0x00000200
#define CVAR_NORESTART   0x00000400
#define CVAR_INTERNAL    0x00000800

#define GAMEVERSION "basejka"

typedef struct { int handle; int modificationCount; float value; int integer; char string[256]; } vmCvar_t;

/* cvarTable_t -- verbatim from g_main.c */
typedef struct {
	vmCvar_t	*vmCvar;
	char		*cvarName;
	char		*defaultString;
	int			cvarFlags;
	int			modificationCount;  // for tracking changes
	qboolean	trackChange;	    // track this variable, and announce if changed
  qboolean teamShader;        // track and if changed, update shader state
} cvarTable_t;

/* --- cvar mirror globals, verbatim from g_main.c (namespace_begin/end #includes
   dropped; the non-vmCvar g_entities/g_clients/gDuelExit globals are not needed) --- */
vmCvar_t	g_trueJedi;

vmCvar_t	g_gametype;
vmCvar_t	g_MaxHolocronCarry;
vmCvar_t	g_ff_objectives;
vmCvar_t	g_autoMapCycle;
vmCvar_t	g_dmflags;
vmCvar_t	g_maxForceRank;
vmCvar_t	g_forceBasedTeams;
vmCvar_t	g_privateDuel;

vmCvar_t	g_allowNPC;

vmCvar_t	g_armBreakage;

vmCvar_t	g_saberLocking;
vmCvar_t	g_saberLockFactor;
vmCvar_t	g_saberTraceSaberFirst;

vmCvar_t	d_saberKickTweak;

vmCvar_t	d_powerDuelPrint;

vmCvar_t	d_saberGhoul2Collision;
vmCvar_t	g_saberBladeFaces;
vmCvar_t	d_saberAlwaysBoxTrace;
vmCvar_t	d_saberBoxTraceSize;

vmCvar_t	d_siegeSeekerNPC;

vmCvar_t	g_debugMelee;
vmCvar_t	g_stepSlideFix;

vmCvar_t	g_noSpecMove;

#ifdef _DEBUG
vmCvar_t	g_disableServerG2;
#endif

vmCvar_t	d_perPlayerGhoul2;

vmCvar_t	d_projectileGhoul2Collision;

vmCvar_t	g_g2TraceLod;

vmCvar_t	g_optvehtrace;

vmCvar_t	g_locationBasedDamage;

vmCvar_t	g_allowHighPingDuelist;

vmCvar_t	g_logClientInfo;

vmCvar_t	g_slowmoDuelEnd;

vmCvar_t	g_saberDamageScale;

vmCvar_t	g_useWhileThrowing;

vmCvar_t	g_RMG;

vmCvar_t	g_svfps;

vmCvar_t	g_forceRegenTime;
vmCvar_t	g_spawnInvulnerability;
vmCvar_t	g_forcePowerDisable;
vmCvar_t	g_weaponDisable;
vmCvar_t	g_duelWeaponDisable;
vmCvar_t	g_allowDuelSuicide;
vmCvar_t	g_fraglimitVoteCorrection;
vmCvar_t	g_fraglimit;
vmCvar_t	g_duel_fraglimit;
vmCvar_t	g_timelimit;
vmCvar_t	g_capturelimit;
vmCvar_t	d_saberInterpolate;
vmCvar_t	g_friendlyFire;
vmCvar_t	g_friendlySaber;
vmCvar_t	g_password;
vmCvar_t	g_needpass;
vmCvar_t	g_maxclients;
vmCvar_t	g_maxGameClients;
vmCvar_t	g_dedicated;
vmCvar_t	g_developer;
vmCvar_t	g_speed;
vmCvar_t	g_gravity;
vmCvar_t	g_cheats;
vmCvar_t	g_knockback;
vmCvar_t	g_quadfactor;
vmCvar_t	g_forcerespawn;
vmCvar_t	g_siegeRespawn;
vmCvar_t	g_inactivity;
vmCvar_t	g_debugMove;
#ifndef FINAL_BUILD
vmCvar_t	g_debugDamage;
#endif
vmCvar_t	g_debugAlloc;
vmCvar_t	g_debugServerSkel;
vmCvar_t	g_weaponRespawn;
vmCvar_t	g_weaponTeamRespawn;
vmCvar_t	g_adaptRespawn;
vmCvar_t	g_motd;
vmCvar_t	g_synchronousClients;
vmCvar_t	g_warmup;
vmCvar_t	g_doWarmup;
vmCvar_t	g_restarted;
vmCvar_t	g_log;
vmCvar_t	g_logSync;
vmCvar_t	g_statLog;
vmCvar_t	g_statLogFile;
vmCvar_t	g_blood;
vmCvar_t	g_podiumDist;
vmCvar_t	g_podiumDrop;
vmCvar_t	g_allowVote;
vmCvar_t	g_allowTeamVote;
vmCvar_t	g_teamAutoJoin;
vmCvar_t	g_teamForceBalance;
vmCvar_t	g_banIPs;
vmCvar_t	g_filterBan;
vmCvar_t	g_debugForward;
vmCvar_t	g_debugRight;
vmCvar_t	g_debugUp;
vmCvar_t	g_smoothClients;

vmCvar_t	pmove_fixed;
vmCvar_t	pmove_msec;

vmCvar_t	g_listEntity;
//vmCvar_t	g_redteam;
//vmCvar_t	g_blueteam;
vmCvar_t	g_singlePlayer;
vmCvar_t	g_enableBreath;
vmCvar_t	g_dismember;
vmCvar_t	g_forceDodge;
vmCvar_t	g_timeouttospec;

vmCvar_t	g_saberDmgVelocityScale;
vmCvar_t	g_saberDmgDelay_Idle;
vmCvar_t	g_saberDmgDelay_Wound;

vmCvar_t	g_saberDebugPrint;

vmCvar_t	g_siegeTeamSwitch;

vmCvar_t	bg_fighterAltControl;
vmCvar_t	g_vehAutoAimLead;
vmCvar_t	g_autoKickKillSpammers;
vmCvar_t	g_autoBanKillSpammers;
vmCvar_t	g_autoKickTKSpammers;
vmCvar_t	g_autoBanTKSpammers;

#ifdef DEBUG_SABER_BOX
vmCvar_t	g_saberDebugBox;
#endif

vmCvar_t	d_altRoutes;
vmCvar_t	d_patched;

vmCvar_t		g_saberRealisticCombat;
vmCvar_t		g_saberRestrictForce;
vmCvar_t		d_saberSPStyleDamage;
vmCvar_t		g_debugSaberLocks;
vmCvar_t		g_saberLockRandomNess;
vmCvar_t		g_saberWallDamageScale;

vmCvar_t		d_saberStanceDebug;
vmCvar_t		debugNPCAI;
vmCvar_t		debugNPCFreeze;
vmCvar_t		debugNPCAimingBeam;
vmCvar_t		debugBreak;
vmCvar_t		debugNoRoam;
vmCvar_t		d_saberCombat;
vmCvar_t		d_JediAI;
vmCvar_t		d_noGroupAI;
vmCvar_t		d_asynchronousGroupAI;
vmCvar_t		d_slowmodeath;
vmCvar_t		d_noIntermissionWait;

vmCvar_t		g_spskill;

vmCvar_t		g_siegeTeam1;
vmCvar_t		g_siegeTeam2;

vmCvar_t	g_austrian;

vmCvar_t	g_powerDuelStartHealth;
vmCvar_t	g_powerDuelEndHealth;

vmCvar_t		g_showDuelHealths;

/* --- gameCvarTable, verbatim from g_main.c --- */
static cvarTable_t		gameCvarTable[] = {
	// don't override the cheat state set by the system
	{ &g_cheats, "sv_cheats", "", 0, 0, qfalse },

	{ &g_debugMelee, "g_debugMelee", "0", CVAR_SERVERINFO, 0, qtrue  },
	{ &g_stepSlideFix, "g_stepSlideFix", "1", CVAR_SERVERINFO, 0, qtrue  },

	{ &g_noSpecMove, "g_noSpecMove", "0", CVAR_SERVERINFO, 0, qtrue },

	// noset vars
	{ NULL, "gamename", GAMEVERSION , CVAR_SERVERINFO | CVAR_ROM, 0, qfalse  },
	{ NULL, "gamedate", __DATE__ , CVAR_ROM, 0, qfalse  },
	{ &g_restarted, "g_restarted", "0", CVAR_ROM, 0, qfalse  },
	{ NULL, "sv_mapname", "", CVAR_SERVERINFO | CVAR_ROM, 0, qfalse  },

	// latched vars
	{ &g_gametype, "g_gametype", "0", CVAR_SERVERINFO | CVAR_LATCH, 0, qfalse  },
	{ &g_MaxHolocronCarry, "g_MaxHolocronCarry", "3", CVAR_SERVERINFO | CVAR_LATCH, 0, qfalse  },

	{ &g_maxclients, "sv_maxclients", "8", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, qfalse  },
	{ &g_maxGameClients, "g_maxGameClients", "0", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, qfalse  },

	{ &g_trueJedi, "g_jediVmerc", "0", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, qtrue },

	// change anytime vars
	{ &g_ff_objectives, "g_ff_objectives", "0", /*CVAR_SERVERINFO |*/ CVAR_CHEAT | CVAR_NORESTART, 0, qtrue },

	{ &g_autoMapCycle, "g_autoMapCycle", "0", CVAR_ARCHIVE | CVAR_NORESTART, 0, qtrue },
	{ &g_dmflags, "dmflags", "0", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, qtrue  },

	{ &g_maxForceRank, "g_maxForceRank", "6", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, qfalse  },
	{ &g_forceBasedTeams, "g_forceBasedTeams", "0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, qfalse  },
	{ &g_privateDuel, "g_privateDuel", "1", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, qtrue  },

	{ &g_allowNPC, "g_allowNPC", "1", CVAR_SERVERINFO | CVAR_CHEAT, 0, qtrue  },

	{ &g_armBreakage, "g_armBreakage", "0", 0, 0, qtrue  },

	{ &g_saberLocking, "g_saberLocking", "1", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, qtrue  },
	{ &g_saberLockFactor, "g_saberLockFactor", "2", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_saberTraceSaberFirst, "g_saberTraceSaberFirst", "0", CVAR_ARCHIVE, 0, qtrue  },

	{ &d_saberKickTweak, "d_saberKickTweak", "1", 0, 0, qtrue  },

	{ &d_powerDuelPrint, "d_powerDuelPrint", "0", 0, qtrue },

	{ &d_saberGhoul2Collision, "d_saberGhoul2Collision", "1", CVAR_CHEAT, 0, qtrue  },
	{ &g_saberBladeFaces, "g_saberBladeFaces", "1", 0, 0, qtrue  },

	{ &d_saberAlwaysBoxTrace, "d_saberAlwaysBoxTrace", "0", CVAR_CHEAT, 0, qtrue  },
	{ &d_saberBoxTraceSize, "d_saberBoxTraceSize", "0", CVAR_CHEAT, 0, qtrue  },

	{ &d_siegeSeekerNPC, "d_siegeSeekerNPC", "0", CVAR_CHEAT, 0, qtrue },

#ifdef _DEBUG
	{ &g_disableServerG2, "g_disableServerG2", "0", 0, 0, qtrue },
#endif

	{ &d_perPlayerGhoul2, "d_perPlayerGhoul2", "0", CVAR_CHEAT, 0, qtrue },

	{ &d_projectileGhoul2Collision, "d_projectileGhoul2Collision", "1", CVAR_CHEAT, 0, qtrue  },

	{ &g_g2TraceLod, "g_g2TraceLod", "3", 0, 0, qtrue  },

	{ &g_optvehtrace, "com_optvehtrace", "0", 0, 0, qtrue  },

	{ &g_locationBasedDamage, "g_locationBasedDamage", "1", 0, 0, qtrue },

	{ &g_allowHighPingDuelist, "g_allowHighPingDuelist", "1", 0, 0, qtrue },

	{ &g_logClientInfo, "g_logClientInfo", "0", CVAR_ARCHIVE, 0, qtrue  },

	{ &g_slowmoDuelEnd, "g_slowmoDuelEnd", "0", CVAR_ARCHIVE, 0, qtrue  },

	{ &g_saberDamageScale, "g_saberDamageScale", "1", CVAR_ARCHIVE, 0, qtrue  },

	{ &g_useWhileThrowing, "g_useWhileThrowing", "1", 0, 0, qtrue  },

	{ &g_RMG, "RMG", "0", 0, 0, qtrue  },

	{ &g_svfps, "sv_fps", "20", 0, 0, qtrue },

	{ &g_forceRegenTime, "g_forceRegenTime", "200", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, qtrue  },

	{ &g_spawnInvulnerability, "g_spawnInvulnerability", "3000", CVAR_ARCHIVE, 0, qtrue  },

	{ &g_forcePowerDisable, "g_forcePowerDisable", "0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, qtrue  },
	{ &g_weaponDisable, "g_weaponDisable", "0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, qtrue  },
	{ &g_duelWeaponDisable, "g_duelWeaponDisable", "1", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, qtrue  },

	{ &g_allowDuelSuicide, "g_allowDuelSuicide", "1", CVAR_ARCHIVE, 0, qtrue },

	{ &g_fraglimitVoteCorrection, "g_fraglimitVoteCorrection", "1", CVAR_ARCHIVE, 0, qtrue },

	{ &g_fraglimit, "fraglimit", "20", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, qtrue },
	{ &g_duel_fraglimit, "duel_fraglimit", "10", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, qtrue },
	{ &g_timelimit, "timelimit", "0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, qtrue },
	{ &g_capturelimit, "capturelimit", "8", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, qtrue },

	{ &g_synchronousClients, "g_synchronousClients", "0", CVAR_SYSTEMINFO, 0, qfalse  },

	{ &d_saberInterpolate, "d_saberInterpolate", "0", CVAR_CHEAT, 0, qtrue },

	{ &g_friendlyFire, "g_friendlyFire", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_friendlySaber, "g_friendlySaber", "0", CVAR_ARCHIVE, 0, qtrue  },

	{ &g_teamAutoJoin, "g_teamAutoJoin", "0", CVAR_ARCHIVE  },
	{ &g_teamForceBalance, "g_teamForceBalance", "0", CVAR_ARCHIVE  },

	{ &g_warmup, "g_warmup", "20", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_doWarmup, "g_doWarmup", "0", 0, 0, qtrue  },
	{ &g_log, "g_log", "games.log", CVAR_ARCHIVE, 0, qfalse  },
	{ &g_logSync, "g_logSync", "0", CVAR_ARCHIVE, 0, qfalse  },

	{ &g_statLog, "g_statLog", "0", CVAR_ARCHIVE, 0, qfalse },
	{ &g_statLogFile, "g_statLogFile", "statlog.log", CVAR_ARCHIVE, 0, qfalse },

	{ &g_password, "g_password", "", CVAR_USERINFO, 0, qfalse  },

	{ &g_banIPs, "g_banIPs", "", CVAR_ARCHIVE, 0, qfalse  },
	{ &g_filterBan, "g_filterBan", "1", CVAR_ARCHIVE, 0, qfalse  },

	{ &g_needpass, "g_needpass", "0", CVAR_SERVERINFO | CVAR_ROM, 0, qfalse },

	{ &g_dedicated, "dedicated", "0", 0, 0, qfalse  },

	{ &g_developer, "developer", "0", 0, 0, qfalse },

	{ &g_speed, "g_speed", "250", 0, 0, qtrue  },
	{ &g_gravity, "g_gravity", "800", 0, 0, qtrue  },
	{ &g_knockback, "g_knockback", "1000", 0, 0, qtrue  },
	{ &g_quadfactor, "g_quadfactor", "3", 0, 0, qtrue  },
	{ &g_weaponRespawn, "g_weaponrespawn", "5", 0, 0, qtrue  },
	{ &g_weaponTeamRespawn, "g_weaponTeamRespawn", "5", 0, 0, qtrue },
	{ &g_adaptRespawn, "g_adaptrespawn", "1", 0, 0, qtrue  },		// Make weapons respawn faster with a lot of players.
	{ &g_forcerespawn, "g_forcerespawn", "60", 0, 0, qtrue },		// One minute force respawn.  Give a player enough time to reallocate force.
	{ &g_siegeRespawn, "g_siegeRespawn", "20", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, qtrue }, //siege respawn wave time
	{ &g_inactivity, "g_inactivity", "0", 0, 0, qtrue },
	{ &g_debugMove, "g_debugMove", "0", 0, 0, qfalse },
#ifndef FINAL_BUILD
	{ &g_debugDamage, "g_debugDamage", "0", 0, 0, qfalse },
#endif
	{ &g_debugAlloc, "g_debugAlloc", "0", 0, 0, qfalse },
	{ &g_debugServerSkel, "g_debugServerSkel", "0", CVAR_CHEAT, 0, qfalse },
	{ &g_motd, "g_motd", "", 0, 0, qfalse },
	{ &g_blood, "com_blood", "1", 0, 0, qfalse },

	{ &g_podiumDist, "g_podiumDist", "80", 0, 0, qfalse },
	{ &g_podiumDrop, "g_podiumDrop", "70", 0, 0, qfalse },

	{ &g_allowVote, "g_allowVote", "1", CVAR_ARCHIVE, 0, qfalse },
	{ &g_allowTeamVote, "g_allowTeamVote", "1", CVAR_ARCHIVE, 0, qfalse },
	{ &g_listEntity, "g_listEntity", "0", 0, 0, qfalse },

#if 0
	{ &g_debugForward, "g_debugForward", "0", 0, 0, qfalse },
	{ &g_debugRight, "g_debugRight", "0", 0, 0, qfalse },
	{ &g_debugUp, "g_debugUp", "0", 0, 0, qfalse },
#endif

//	{ &g_redteam, "g_redteam", "Empire", CVAR_ARCHIVE | CVAR_SERVERINFO | CVAR_USERINFO , 0, qtrue, qtrue },
//	{ &g_blueteam, "g_blueteam", "Rebellion", CVAR_ARCHIVE | CVAR_SERVERINFO | CVAR_USERINFO , 0, qtrue, qtrue  },
	{ &g_singlePlayer, "ui_singlePlayerActive", "", 0, 0, qfalse, qfalse  },

	{ &g_enableBreath, "g_enableBreath", "0", 0, 0, qtrue, qfalse },
	{ &g_smoothClients, "g_smoothClients", "1", 0, 0, qfalse},
	{ &pmove_fixed, "pmove_fixed", "0", CVAR_SYSTEMINFO, 0, qfalse},
	{ &pmove_msec, "pmove_msec", "8", CVAR_SYSTEMINFO, 0, qfalse},

	{ &g_dismember, "g_dismember", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_forceDodge, "g_forceDodge", "1", 0, 0, qtrue  },

	{ &g_timeouttospec, "g_timeouttospec", "70", CVAR_ARCHIVE, 0, qfalse },

	{ &g_saberDmgVelocityScale, "g_saberDmgVelocityScale", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_saberDmgDelay_Idle, "g_saberDmgDelay_Idle", "350", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_saberDmgDelay_Wound, "g_saberDmgDelay_Wound", "0", CVAR_ARCHIVE, 0, qtrue  },

#ifndef FINAL_BUILD
	{ &g_saberDebugPrint, "g_saberDebugPrint", "0", CVAR_CHEAT, 0, qfalse  },
#endif
	{ &g_debugSaberLocks, "g_debugSaberLocks", "0", CVAR_CHEAT, 0, qfalse },
	{ &g_saberLockRandomNess, "g_saberLockRandomNess", "2", CVAR_CHEAT, 0, qfalse },
// nmckenzie: SABER_DAMAGE_WALLS
	{ &g_saberWallDamageScale, "g_saberWallDamageScale", "0.4", CVAR_SERVERINFO, 0, qfalse },

	{ &d_saberStanceDebug, "d_saberStanceDebug", "0", 0, 0, qfalse },

	{ &g_siegeTeamSwitch, "g_siegeTeamSwitch", "1", CVAR_SERVERINFO|CVAR_ARCHIVE, qfalse },

	{ &bg_fighterAltControl, "bg_fighterAltControl", "0", CVAR_SERVERINFO, 0, qtrue },
	{ &g_vehAutoAimLead, "g_vehAutoAimLead", "0", CVAR_ARCHIVE },
	{ &g_autoKickKillSpammers, "g_autoKickKillSpammers", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_autoBanKillSpammers, "g_autoBanKillSpammers", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_autoKickTKSpammers, "g_autoKickTKSpammers", "0", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_autoBanTKSpammers, "g_autoBanTKSpammers", "0", CVAR_ARCHIVE, 0, qtrue  },

#ifdef DEBUG_SABER_BOX
	{ &g_saberDebugBox, "g_saberDebugBox", "0", CVAR_CHEAT, 0, qfalse },
#endif

	{ &d_altRoutes, "d_altRoutes", "0", CVAR_CHEAT, 0, qfalse },
	{ &d_patched, "d_patched", "0", CVAR_CHEAT, 0, qfalse },

	{ &g_saberRealisticCombat, "g_saberRealisticCombat", "0", CVAR_CHEAT },
	{ &g_saberRestrictForce, "g_saberRestrictForce", "0", CVAR_CHEAT },
	{ &d_saberSPStyleDamage, "d_saberSPStyleDamage", "1", CVAR_CHEAT },

	{ &debugNoRoam, "d_noroam", "0", CVAR_CHEAT },
	{ &debugNPCAimingBeam, "d_npcaiming", "0", CVAR_CHEAT },
	{ &debugBreak, "d_break", "0", CVAR_CHEAT },
	{ &debugNPCAI, "d_npcai", "0", CVAR_CHEAT },
	{ &debugNPCFreeze, "d_npcfreeze", "0", CVAR_CHEAT },
	{ &d_JediAI, "d_JediAI", "0", CVAR_CHEAT },
	{ &d_noGroupAI, "d_noGroupAI", "0", CVAR_CHEAT },
	{ &d_asynchronousGroupAI, "d_asynchronousGroupAI", "0", CVAR_CHEAT },

	//0 = never (BORING)
	//1 = kyle only
	//2 = kyle and last enemy jedi
	//3 = kyle and any enemy jedi
	//4 = kyle and last enemy in a group
	//5 = kyle and any enemy
	//6 = also when kyle takes pain or enemy jedi dodges player saber swing or does an acrobatic evasion

	{ &d_slowmodeath, "d_slowmodeath", "0", CVAR_CHEAT },

	{ &d_saberCombat, "d_saberCombat", "0", CVAR_CHEAT },

	{ &g_spskill, "g_npcspskill", "0", CVAR_ARCHIVE | CVAR_INTERNAL },

	//for overriding the level defaults
	{ &g_siegeTeam1, "g_siegeTeam1", "none", CVAR_ARCHIVE|CVAR_SERVERINFO, 0, qfalse  },
	{ &g_siegeTeam2, "g_siegeTeam2", "none", CVAR_ARCHIVE|CVAR_SERVERINFO, 0, qfalse  },

	//mainly for debugging with bots while I'm not around (want the server to
	//cycle through levels naturally)
	{ &d_noIntermissionWait, "d_noIntermissionWait", "0", CVAR_CHEAT, 0, qfalse  },

	{ &g_austrian, "g_austrian", "0", CVAR_ARCHIVE, 0, qfalse  },
// nmckenzie:
// DUEL_HEALTH
	{ &g_showDuelHealths, "g_showDuelHealths", "0", CVAR_SERVERINFO },
	{ &g_powerDuelStartHealth, "g_powerDuelStartHealth", "150", CVAR_ARCHIVE, 0, qtrue  },
	{ &g_powerDuelEndHealth, "g_powerDuelEndHealth", "90", CVAR_ARCHIVE, 0, qtrue  },
};

/* --- accessors: the real C table's size + each row's data fields --- */
int jka_gameCvarTableSize(void) { return (int)(sizeof(gameCvarTable) / sizeof(gameCvarTable[0])); }
const char *jka_cvar_name(int i) { return gameCvarTable[i].cvarName; }
const char *jka_cvar_default(int i) { return gameCvarTable[i].defaultString; }
int jka_cvar_flags(int i) { return gameCvarTable[i].cvarFlags; }
int jka_cvar_modcount(int i) { return gameCvarTable[i].modificationCount; }
int jka_cvar_track(int i) { return gameCvarTable[i].trackChange; }
int jka_cvar_team(int i) { return gameCvarTable[i].teamShader; }

/* --- G_GetStringEdString (g_main.c:4217) oracle ---
 * Verbatim function body. The engine's Com_sprintf renders into a 32000-byte
 * scratch via vsnprintf then Q_strncpyz's into dest bounded by size (NUL-term).
 * A vsnprintf straight into the size-bounded dest is byte-identical for the
 * "@@@%s" inputs that occur here, so this self-contained Com_sprintf replica
 * reproduces the engine exactly without dragging in q_shared.
 */
#include <stdio.h>
#include <string.h>
#include <stdarg.h>

static void jka_Com_sprintf_edstring(char *dest, int size, const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(dest, (size_t)size, fmt, ap); /* always NUL-terminates within size */
	va_end(ap);
}

const char *jka_G_GetStringEdString(char *refSection, char *refName) {
	static char text[1024] = {0};
	jka_Com_sprintf_edstring(text, sizeof(text), "@@@%s", refName);
	return text;
}
