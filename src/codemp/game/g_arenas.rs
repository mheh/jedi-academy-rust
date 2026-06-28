// Copyright (C) 1999-2000 Id Software, Inc.
//
//
// g_arenas.c
//

#![allow(non_snake_case)]

use crate::codemp::game::g_local::{gentity_t, level_locals_t, gclient_t};
use crate::codemp::game::g_public_h::SVF_BOT;
use crate::codemp::game::bg_public::{
    TEAM_SPECTATOR, TEAM_RED, TEAM_BLUE, GT_CTF,
    PERS_SCORE, PERS_RANK, PERS_KILLED,
    PERS_IMPRESSIVE_COUNT, PERS_EXCELLENT_COUNT, PERS_DEFEND_COUNT,
    PERS_ASSIST_COUNT, PERS_GAUNTLET_FRAG_COUNT, PERS_CAPTURES,
};
use crate::codemp::game::q_shared_h::MAX_STRING_CHARS;
use crate::codemp::game::g_main::CalculateRanks;
use crate::codemp::game::g_syscalls::trap_SendConsoleCommand;
use crate::codemp::game::q_shared::Com_sprintf;
use core::ffi::{c_int, c_char};
use core::ptr::addr_of_mut;

extern "C" {
    pub static mut g_entities: [gentity_t; 0];
    pub static mut level: level_locals_t;
    pub static g_gametype: crate::ffi::types::vmCvar_t;

    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

pub static mut podium1: *mut gentity_t = core::ptr::null_mut();
pub static mut podium2: *mut gentity_t = core::ptr::null_mut();
pub static mut podium3: *mut gentity_t = core::ptr::null_mut();


/*
==================
UpdateTournamentInfo
==================
*/
pub unsafe fn UpdateTournamentInfo() {
    let mut i: c_int;
    let mut player: *mut gentity_t;
    let mut playerClientNum: c_int;
    let mut n: c_int;
    let mut accuracy: c_int;
    let mut perfect: c_int;
    let mut msglen: c_int;
    let mut buflen: c_int;
    let mut score1: c_int;
    let mut score2: c_int;
    let mut won: c_int;
    let mut buf: [c_char; 32] = [0; 32];
    let mut msg: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    // find the real player
    player = core::ptr::null_mut();
    i = 0;
    while i < (*addr_of_mut!(level)).maxclients {
        player = addr_of_mut!(g_entities[i as usize]) as *mut gentity_t;
        if (*player).inuse == 0 {
            i += 1;
            continue;
        }
        if ((*player).r.svFlags & SVF_BOT) == 0 {
            break;
        }
        i += 1;
    }
    // this should never happen!
    if player.is_null() || i == (*addr_of_mut!(level)).maxclients {
        return;
    }
    playerClientNum = i;

    CalculateRanks();

    if (*addr_of_mut!(level)).clients[playerClientNum as usize].sess.sessionTeam == TEAM_SPECTATOR {
        Com_sprintf(
            addr_of_mut!(msg[0]),
            core::mem::size_of_val(&msg) as c_int,
            format_args!("postgame {} {} 0 0 0 0 0 0 0 0 0 0 0", (*addr_of_mut!(level)).numNonSpectatorClients, playerClientNum),
        );
    } else {
        if (*(*player).client).accuracy_shots != 0 {
            accuracy = (*(*player).client).accuracy_hits * 100 / (*(*player).client).accuracy_shots;
        } else {
            accuracy = 0;
        }
        won = 0;
        if g_gametype.integer >= GT_CTF {
            score1 = (*addr_of_mut!(level)).teamScores[TEAM_RED as usize];
            score2 = (*addr_of_mut!(level)).teamScores[TEAM_BLUE as usize];
            if (*addr_of_mut!(level)).clients[playerClientNum as usize].sess.sessionTeam == TEAM_RED {
                won = ((*addr_of_mut!(level)).teamScores[TEAM_RED as usize] > (*addr_of_mut!(level)).teamScores[TEAM_BLUE as usize]) as c_int;
            } else {
                won = ((*addr_of_mut!(level)).teamScores[TEAM_BLUE as usize] > (*addr_of_mut!(level)).teamScores[TEAM_RED as usize]) as c_int;
            }
        } else {
            if &(*addr_of_mut!(level)).clients[playerClientNum as usize] as *const _ ==
                &(*addr_of_mut!(level)).clients[(*addr_of_mut!(level)).sortedClients[0] as usize] as *const _ {
                won = 1;
                score1 = (*addr_of_mut!(level)).clients[(*addr_of_mut!(level)).sortedClients[0] as usize].ps.persistant[PERS_SCORE as usize];
                score2 = (*addr_of_mut!(level)).clients[(*addr_of_mut!(level)).sortedClients[1] as usize].ps.persistant[PERS_SCORE as usize];
            } else {
                score2 = (*addr_of_mut!(level)).clients[(*addr_of_mut!(level)).sortedClients[0] as usize].ps.persistant[PERS_SCORE as usize];
                score1 = (*addr_of_mut!(level)).clients[(*addr_of_mut!(level)).sortedClients[1] as usize].ps.persistant[PERS_SCORE as usize];
            }
        }
        if won != 0 && (*(*player).client).ps.persistant[PERS_KILLED as usize] == 0 {
            perfect = 1;
        } else {
            perfect = 0;
        }
        Com_sprintf(
            addr_of_mut!(msg[0]),
            core::mem::size_of_val(&msg) as c_int,
            format_args!("postgame {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                (*addr_of_mut!(level)).numNonSpectatorClients,
                playerClientNum,
                accuracy,
                (*(*player).client).ps.persistant[PERS_IMPRESSIVE_COUNT as usize],
                (*(*player).client).ps.persistant[PERS_EXCELLENT_COUNT as usize],
                (*(*player).client).ps.persistant[PERS_DEFEND_COUNT as usize],
                (*(*player).client).ps.persistant[PERS_ASSIST_COUNT as usize],
                (*(*player).client).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize],
                (*(*player).client).ps.persistant[PERS_SCORE as usize],
                perfect,
                score1,
                score2,
                (*addr_of_mut!(level)).time,
                (*(*player).client).ps.persistant[PERS_CAPTURES as usize]
            ),
        );
    }

    msglen = strlen(addr_of_mut!(msg[0])) as c_int;
    i = 0;
    while i < (*addr_of_mut!(level)).numNonSpectatorClients {
        n = (*addr_of_mut!(level)).sortedClients[i as usize];
        Com_sprintf(
            addr_of_mut!(buf[0]),
            core::mem::size_of_val(&buf) as c_int,
            format_args!(" {} {} {}",
                n,
                (*addr_of_mut!(level)).clients[n as usize].ps.persistant[PERS_RANK as usize],
                (*addr_of_mut!(level)).clients[n as usize].ps.persistant[PERS_SCORE as usize]
            ),
        );
        buflen = strlen(addr_of_mut!(buf[0])) as c_int;
        if msglen + buflen + 1 >= core::mem::size_of_val(&msg) as c_int {
            break;
        }
        strcat(addr_of_mut!(msg[0]), addr_of_mut!(buf[0]));
        i += 1;
    }
    trap_SendConsoleCommand(2, addr_of_mut!(msg[0])); // EXEC_APPEND = 2
}


/*
static gentity_t *SpawnModelOnVictoryPad( gentity_t *pad, vec3_t offset, gentity_t *ent, int place ) {
	gentity_t	*body;
	vec3_t		vec;
	vec3_t		f, r, u;

	body = G_Spawn();
	if ( !body ) {
		G_Printf( S_COLOR_RED "ERROR: out of gentities\n" );
		return NULL;
	}

	body->classname = ent->client->pers.netname;
	body->client = ent->client;
	body->s = ent->s;
	body->s.eType = ET_PLAYER;		// could be ET_INVISIBLE
	body->s.eFlags = 0;				// clear EF_TALK, etc
	body->s.powerups = 0;			// clear powerups
	body->s.loopSound = 0;			// clear lava burning
	body->s.number = body - g_entities;
	body->timestamp = level.time;
	body->physicsObject = qtrue;
	body->physicsBounce = 0;		// don't bounce
	body->s.event = 0;
	body->s.pos.trType = TR_STATIONARY;
	body->s.groundEntityNum = ENTITYNUM_WORLD;
	body->s.legsAnim = WeaponReadyAnim[ent->s.weapon];
	body->s.torsoAnim = WeaponReadyAnim[ent->s.weapon];
	if( body->s.weapon == WP_NONE ) {
		body->s.weapon = WP_BRYAR_PISTOL;
	}
	if( body->s.weapon == WP_SABER) {
		body->s.torsoAnim = BOTH_STAND2;
	}
	body->s.event = 0;
	body->r.svFlags = ent->r.svFlags;
	VectorCopy (ent->r.mins, body->r.mins);
	VectorCopy (ent->r.maxs, body->r.maxs);
	VectorCopy (ent->r.absmin, body->r.absmin);
	VectorCopy (ent->r.absmax, body->r.absmax);
	body->clipmask = CONTENTS_SOLID | CONTENTS_PLAYERCLIP;
	body->r.contents = CONTENTS_BODY;
	body->r.ownerNum = ent->r.ownerNum;
	body->takedamage = qfalse;

	VectorSubtract( level.intermission_origin, pad->r.currentOrigin, vec );
	vectoangles( vec, body->s.apos.trBase );
	body->s.apos.trBase[PITCH] = 0;
	body->s.apos.trBase[ROLL] = 0;

	AngleVectors( body->s.apos.trBase, f, r, u );
	VectorMA( pad->r.currentOrigin, offset[0], f, vec );
	VectorMA( vec, offset[1], r, vec );
	VectorMA( vec, offset[2], u, vec );

	G_SetOrigin( body, vec );

	trap_LinkEntity (body);

	body->count = place;

	return body;
}


static void CelebrateStop( gentity_t *player ) {
	int		anim;

	if( player->s.weapon == WP_SABER) {
		anim = BOTH_STAND2;
	}
	else {
		anim = WeaponReadyAnim[player->s.weapon];
	}
	player->s.torsoAnim = ( ( player->s.torsoAnim & ANIM_TOGGLEBIT ) ^ ANIM_TOGGLEBIT ) | anim;
}


#define	TIMER_GESTURE	(34*66+50)
static void CelebrateStart( gentity_t *player ) {
	player->s.torsoAnim = ( ( player->s.torsoAnim & ANIM_TOGGLEBIT ) ^ ANIM_TOGGLEBIT ) | BOTH_TALKGESTURE1;
	player->nextthink = level.time + TIMER_GESTURE;
	player->think = CelebrateStop;


//	player->client->ps.events[player->client->ps.eventSequence & (MAX_PS_EVENTS-1)] = EV_TAUNT;
//	player->client->ps.eventParms[player->client->ps.eventSequence & (MAX_PS_EVENTS-1)] = 0;
//	player->client->ps.eventSequence++;

	G_AddEvent(player, EV_TAUNT, 0);
}


static vec3_t	offsetFirst  = {0, 0, 74};
static vec3_t	offsetSecond = {-10, 60, 54};
static vec3_t	offsetThird  = {-19, -60, 45};

static void PodiumPlacementThink( gentity_t *podium ) {
	vec3_t		vec;
	vec3_t		origin;
	vec3_t		f, r, u;

	podium->nextthink = level.time + 100;

	AngleVectors( level.intermission_angle, vec, NULL, NULL );
	VectorMA( level.intermission_origin, trap_Cvar_VariableIntegerValue( "g_podiumDist" ), vec, origin );
	origin[2] -= trap_Cvar_VariableIntegerValue( "g_podiumDrop" );
	G_SetOrigin( podium, origin );

	if( podium1 ) {
		VectorSubtract( level.intermission_origin, podium->r.currentOrigin, vec );
		vectoangles( vec, podium1->s.apos.trBase );
		podium1->s.apos.trBase[PITCH] = 0;
		podium1->s.apos.trBase[ROLL] = 0;

		AngleVectors( podium1->s.apos.trBase, f, r, u );
		VectorMA( podium->r.currentOrigin, offsetFirst[0], f, vec );
		VectorMA( vec, offsetFirst[1], r, vec );
		VectorMA( vec, offsetFirst[2], u, vec );

		G_SetOrigin( podium1, vec );
	}

	if( podium2 ) {
		VectorSubtract( level.intermission_origin, podium->r.currentOrigin, vec );
		vectoangles( vec, podium2->s.apos.trBase );
		podium2->s.apos.trBase[PITCH] = 0;
		podium2->s.apos.trBase[ROLL] = 0;

		AngleVectors( podium2->s.apos.trBase, f, r, u );
		VectorMA( podium->r.currentOrigin, offsetSecond[0], f, vec );
		VectorMA( vec, offsetSecond[1], r, vec );
		VectorMA( vec, offsetSecond[2], u, vec );

		G_SetOrigin( podium2, vec );
	}

	if( podium3 ) {
		VectorSubtract( level.intermission_origin, podium->r.currentOrigin, vec );
		vectoangles( vec, podium3->s.apos.trBase );
		podium3->s.apos.trBase[PITCH] = 0;
		podium3->s.apos.trBase[ROLL] = 0;

		AngleVectors( podium3->s.apos.trBase, f, r, u );
		VectorMA( podium->r.currentOrigin, offsetThird[0], f, vec );
		VectorMA( vec, offsetThird[1], r, vec );
		VectorMA( vec, offsetThird[2], u, vec );

		G_SetOrigin( podium3, vec );
	}
}


static gentity_t *SpawnPodium( void ) {
	gentity_t	*podium;
	vec3_t		vec;
	vec3_t		origin;

	podium = G_Spawn();
	if ( !podium ) {
		return NULL;
	}

	podium->classname = "podium";
	podium->s.eType = ET_GENERAL;
	podium->s.number = podium - g_entities;
	podium->clipmask = CONTENTS_SOLID;
	podium->r.contents = CONTENTS_SOLID;
	podium->s.modelindex = G_ModelIndex( SP_PODIUM_MODEL );

	AngleVectors( level.intermission_angle, vec, NULL, NULL );
	VectorMA( level.intermission_origin, trap_Cvar_VariableIntegerValue( "g_podiumDist" ), vec, origin );
	origin[2] -= trap_Cvar_VariableIntegerValue( "g_podiumDrop" );
	G_SetOrigin( podium, origin );

	VectorSubtract( level.intermission_origin, podium->r.currentOrigin, vec );
	podium->s.apos.trBase[YAW] = vectoyaw( vec );
	trap_LinkEntity (podium);

	podium->think = PodiumPlacementThink;
	podium->nextthink = level.time + 100;
	return podium;
}



//==================
//SpawnModelsOnVictoryPads
//==================

void SpawnModelsOnVictoryPads( void ) {
	gentity_t	*player;
	gentity_t	*podium;

	podium1 = NULL;
	podium2 = NULL;
	podium3 = NULL;

	podium = SpawnPodium();

	player = SpawnModelOnVictoryPad( podium, offsetFirst, &g_entities[level.sortedClients[0]],
				level.clients[ level.sortedClients[0] ].ps.persistant[PERS_RANK] &~ RANK_TIED_FLAG );
	if ( player ) {
		player->nextthink = level.time + 2000;
		player->think = CelebrateStart;
		podium1 = player;
	}

	player = SpawnModelOnVictoryPad( podium, offsetSecond, &g_entities[level.sortedClients[1]],
				level.clients[ level.sortedClients[1] ].ps.persistant[PERS_RANK] &~ RANK_TIED_FLAG );
	if ( player ) {
		podium2 = player;
	}

	if ( level.numNonSpectatorClients > 2 ) {
		player = SpawnModelOnVictoryPad( podium, offsetThird, &g_entities[level.sortedClients[2]],
				level.clients[ level.sortedClients[2] ].ps.persistant[PERS_RANK] &~ RANK_TIED_FLAG );
		if ( player ) {
			podium3 = player;
		}
	}
}



//===============
//Svcmd_AbortPodium_f
//===============

void Svcmd_AbortPodium_f( void ) {
	if( g_gametype.integer != GT_SINGLE_PLAYER ) {
		return;
	}

	if( podium1 ) {
		podium1->nextthink = level.time;
		podium1->think = CelebrateStop;
	}
}
*/
