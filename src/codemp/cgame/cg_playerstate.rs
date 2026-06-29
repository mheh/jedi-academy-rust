// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_playerstate.c -- this file acts on changes in a new playerState_t
// With normal play, this will be done after local prediction, but when
// following another player or playing back a demo, it will be checked
// when the snapshot transitions like all the other entities

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// CG_CheckAmmo
//
// If the ammo has gone low enough to generate the warning, play a sound
// ============================================================================
pub fn CG_CheckAmmo() {
    // #if 0
    // 	int		i;
    // 	int		total;
    // 	int		previous;
    // 	int		weapons;
    //
    // 	// see about how many seconds of ammo we have remaining
    // 	weapons = cg.snap->ps.stats[ STAT_WEAPONS ];
    // 	total = 0;
    // 	for ( i = WP_BRYAR_PISTOL; i < WP_NUM_WEAPONS ; i++ ) {
    // 		if ( ! ( weapons & ( 1 << i ) ) ) {
    // 			continue;
    // 		}
    // 		switch ( i )
    // 		{
    // 		case WP_BRYAR_PISTOL:
    // 		case WP_CONCUSSION:
    // 		case WP_BRYAR_OLD:
    // 		case WP_BLASTER:
    // 		case WP_DISRUPTOR:
    // 		case WP_BOWCASTER:
    // 		case WP_REPEATER:
    // 		case WP_DEMP2:
    // 		case WP_FLECHETTE:
    // 		case WP_ROCKET_LAUNCHER:
    // 		case WP_THERMAL:
    // 		case WP_TRIP_MINE:
    // 		case WP_DET_PACK:
    // 		case WP_EMPLACED_GUN:
    // 			total += cg.snap->ps.ammo[weaponData[i].ammoIndex] * 1000;
    // 			break;
    // 		default:
    // 			total += cg.snap->ps.ammo[weaponData[i].ammoIndex] * 200;
    // 			break;
    // 		}
    // 		if ( total >= 5000 ) {
    // 			cg.lowAmmoWarning = 0;
    // 			return;
    // 		}
    // 	}
    //
    // 	previous = cg.lowAmmoWarning;
    //
    // 	if ( total == 0 ) {
    // 		cg.lowAmmoWarning = 2;
    // 	} else {
    // 		cg.lowAmmoWarning = 1;
    // 	}
    //
    // 	if (cg.snap->ps.weapon == WP_SABER)
    // 	{
    // 		cg.lowAmmoWarning = 0;
    // 	}
    //
    // 	// play a sound on transitions
    // 	if ( cg.lowAmmoWarning != previous ) {
    // 		trap_S_StartLocalSound( cgs.media.noAmmoSound, CHAN_LOCAL_SOUND );
    // 	}
    // #endif
    // disabled silly ammo warning stuff for now
}

// ============================================================================
// CG_DamageFeedback
// ============================================================================
pub fn CG_DamageFeedback(yawByte: c_int, pitchByte: c_int, damage: c_int) {
    let mut left: f32;
    let mut front: f32;
    let mut up: f32;
    let mut kick: f32;
    let health: c_int;
    let mut scale: f32;
    let mut dir: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let mut dist: f32;
    let mut yaw: f32;
    let mut pitch: f32;

    // show the attacking player's head and name in corner
    unsafe {
        cg.attackerTime = cg.time;
    }

    // the lower on health you are, the greater the view kick will be
    unsafe {
        health = (*cg.snap).ps.stats[STAT_HEALTH];
        if health < 40 {
            scale = 1.0;
        } else {
            scale = 40.0 / health as f32;
        }
    }
    kick = damage as f32 * scale;

    if kick < 5.0 {
        kick = 5.0;
    }
    if kick > 10.0 {
        kick = 10.0;
    }

    // if yaw and pitch are both 255, make the damage always centered (falling, etc)
    if yawByte == 255 && pitchByte == 255 {
        unsafe {
            cg.damageX = 0.0;
            cg.damageY = 0.0;
            cg.v_dmg_roll = 0.0;
            cg.v_dmg_pitch = -kick;
        }
    } else {
        // positional
        pitch = pitchByte as f32 / 255.0 * 360.0;
        yaw = yawByte as f32 / 255.0 * 360.0;

        angles[PITCH] = pitch;
        angles[YAW] = yaw;
        angles[ROLL] = 0.0;

        AngleVectors(angles.as_ptr(), dir.as_mut_ptr(), std::ptr::null_mut(), std::ptr::null_mut());
        VectorSubtract(vec3_origin.as_ptr(), dir.as_ptr(), dir.as_mut_ptr());

        unsafe {
            front = DotProduct(dir.as_ptr(), (*cg.refdef).viewaxis[0].as_ptr());
            left = DotProduct(dir.as_ptr(), (*cg.refdef).viewaxis[1].as_ptr());
            up = DotProduct(dir.as_ptr(), (*cg.refdef).viewaxis[2].as_ptr());
        }

        dir[0] = front;
        dir[1] = left;
        dir[2] = 0.0;
        dist = VectorLength(dir.as_ptr());
        if dist < 0.1 {
            dist = 0.1;
        }

        unsafe {
            cg.v_dmg_roll = kick * left;

            cg.v_dmg_pitch = -kick * front;

            if front <= 0.1 {
                front = 0.1;
            }
            cg.damageX = -left / front;
            cg.damageY = up / dist;
        }
    }

    // clamp the position
    unsafe {
        if cg.damageX > 1.0 {
            cg.damageX = 1.0;
        }
        if cg.damageX < -1.0 {
            cg.damageX = -1.0;
        }

        if cg.damageY > 1.0 {
            cg.damageY = 1.0;
        }
        if cg.damageY < -1.0 {
            cg.damageY = -1.0;
        }

        // don't let the screen flashes vary as much
        if kick > 10.0 {
            kick = 10.0;
        }
        cg.damageValue = kick;
        cg.v_dmg_time = cg.time + DAMAGE_TIME;
        cg.damageTime = (*cg.snap).serverTime;
    }

    // JLFRUMBLE
    #[cfg(target_os = "xbox")]
    {
        extern "C" {
            fn FF_XboxShake(intensity: f32, duration: c_int);
            fn FF_XboxDamage(damage: c_int, xpos: f32);
        }
        // FF_XboxShake(kick, 500);
        unsafe {
            FF_XboxDamage(damage, -left);
        }
    }
}

// ================
// CG_Respawn
//
// A respawn happened this snapshot
// ================
pub fn CG_Respawn() {
    // no error decay on player movement
    unsafe {
        cg.thisFrameTeleport = qtrue;

        // display weapons available
        cg.weaponSelectTime = cg.time;

        // select the weapon the server says we are using
        cg.weaponSelect = (*cg.snap).ps.weapon;
    }
}

extern "C" {
    pub static eventnames: *const *const c_char;
}

// ==============
// CG_CheckPlayerstateEvents
// ==============
pub fn CG_CheckPlayerstateEvents(ps: *const playerState_t, ops: *const playerState_t) {
    let mut i: c_int;
    let mut event: c_int;
    let mut cent: *mut centity_t;

    unsafe {
        if (*ps).externalEvent != 0 && (*ps).externalEvent != (*ops).externalEvent {
            cent = &mut cg_entities[(*ps).clientNum as usize];
            (*cent).currentState.event = (*ps).externalEvent;
            (*cent).currentState.eventParm = (*ps).externalEventParm;
            CG_EntityEvent(cent, (*cent).lerpOrigin.as_ptr());
        }

        cent = &mut cg_entities[(*ps).clientNum as usize];
        // go through the predictable events buffer
        i = (*ps).eventSequence - MAX_PS_EVENTS;
        while i < (*ps).eventSequence {
            // if we have a new predictable event
            if i >= (*ops).eventSequence
                // or the server told us to play another event instead of a predicted event we already issued
                // or something the server told us changed our prediction causing a different event
                || (i > (*ops).eventSequence - MAX_PS_EVENTS
                    && (*ps).events[(i & (MAX_PS_EVENTS - 1)) as usize]
                        != (*ops).events[(i & (MAX_PS_EVENTS - 1)) as usize])
            {
                event = (*ps).events[(i & (MAX_PS_EVENTS - 1)) as usize];
                (*cent).currentState.event = event;
                (*cent).currentState.eventParm =
                    (*ps).eventParms[(i & (MAX_PS_EVENTS - 1)) as usize];
                // JLF ADDED to hopefully mark events as player event
                (*cent).playerState = ps;
                CG_EntityEvent(cent, (*cent).lerpOrigin.as_ptr());

                cg.predictableEvents[(i & (MAX_PREDICTED_EVENTS - 1)) as usize] = event;

                cg.eventSequence += 1;
            }
            i += 1;
        }
    }
}

// ==================
// CG_CheckChangedPredictableEvents
// ==================
pub fn CG_CheckChangedPredictableEvents(ps: *const playerState_t) {
    let mut i: c_int;
    let mut event: c_int;
    let mut cent: *mut centity_t;

    unsafe {
        cent = &mut cg_entities[(*ps).clientNum as usize];
        i = (*ps).eventSequence - MAX_PS_EVENTS;
        while i < (*ps).eventSequence {
            //
            if i >= cg.eventSequence {
                i += 1;
                continue;
            }
            // if this event is not further back in than the maximum predictable events we remember
            if i > cg.eventSequence - MAX_PREDICTED_EVENTS {
                // if the new playerstate event is different from a previously predicted one
                if (*ps).events[(i & (MAX_PS_EVENTS - 1)) as usize]
                    != cg.predictableEvents[(i & (MAX_PREDICTED_EVENTS - 1)) as usize]
                {
                    event = (*ps).events[(i & (MAX_PS_EVENTS - 1)) as usize];
                    (*cent).currentState.event = event;
                    (*cent).currentState.eventParm =
                        (*ps).eventParms[(i & (MAX_PS_EVENTS - 1)) as usize];
                    CG_EntityEvent(cent, (*cent).lerpOrigin.as_ptr());

                    cg.predictableEvents[(i & (MAX_PREDICTED_EVENTS - 1)) as usize] = event;

                    if cg_showmiss.integer != 0 {
                        CG_Printf(b"WARNING: changed predicted event\n" as *const u8 as *const c_char);
                    }
                }
            }
            i += 1;
        }
    }
}

// ==================
// pushReward
// ==================
#[cfg(feature = "JK2AWARDS")]
unsafe fn pushReward(sfx: sfxHandle_t, shader: qhandle_t, rewardCount: c_int) {
    if cg.rewardStack < (MAX_REWARDSTACK - 1) {
        cg.rewardStack += 1;
        cg.rewardSound[cg.rewardStack as usize] = sfx;
        cg.rewardShader[cg.rewardStack as usize] = shader;
        cg.rewardCount[cg.rewardStack as usize] = rewardCount;
    }
}

pub static mut cgAnnouncerTime: c_int = 0; // to prevent announce sounds from playing on top of each other

// ==================
// CG_CheckLocalSounds
// ==================
pub fn CG_CheckLocalSounds(ps: *const playerState_t, ops: *const playerState_t) {
    let mut highScore: c_int;
    let mut health: c_int;
    let mut armor: c_int;
    let mut reward: c_int;
    #[cfg(feature = "JK2AWARDS")]
    let mut sfx: sfxHandle_t;

    unsafe {
        // don't play the sounds if the player just changed teams
        if (*ps).persistant[PERS_TEAM as usize] != (*ops).persistant[PERS_TEAM as usize] {
            return;
        }

        // hit changes
        if (*ps).persistant[PERS_HITS as usize] > (*ops).persistant[PERS_HITS as usize] {
            armor = ((*ps).persistant[PERS_ATTACKEE_ARMOR as usize] & 0xff) as c_int;
            health = ((*ps).persistant[PERS_ATTACKEE_ARMOR as usize] >> 8) as c_int;

            if armor > health / 2 {
                // We also hit shields along the way, so consider them "pierced".
                //			trap_S_StartLocalSound( cgs.media.shieldPierceSound, CHAN_LOCAL_SOUND );
            } else {
                // Shields didn't really stand in our way.
                //			trap_S_StartLocalSound( cgs.media.hitSound, CHAN_LOCAL_SOUND );
            }

            //FIXME: Hit sounds?
            /*
            if (armor > 50 ) {
                trap_S_StartLocalSound( cgs.media.hitSoundHighArmor, CHAN_LOCAL_SOUND );
            } else if (armor || health > 100) {
                trap_S_StartLocalSound( cgs.media.hitSoundLowArmor, CHAN_LOCAL_SOUND );
            } else {
                trap_S_StartLocalSound( cgs.media.hitSound, CHAN_LOCAL_SOUND );
            }
            */
        } else if (*ps).persistant[PERS_HITS as usize] < (*ops).persistant[PERS_HITS as usize] {
            //trap_S_StartLocalSound( cgs.media.hitTeamSound, CHAN_LOCAL_SOUND );
        }

        // health changes of more than -3 should make pain sounds
        if cg_oldPainSounds.integer != 0 {
            if (*ps).stats[STAT_HEALTH as usize] < ((*ops).stats[STAT_HEALTH as usize] - 3)
                as c_int
            {
                if (*ps).stats[STAT_HEALTH as usize] > 0 {
                    CG_PainEvent(
                        &mut cg_entities[cg.predictedPlayerState.clientNum as usize],
                        (*ps).stats[STAT_HEALTH as usize],
                    );
                }
            }
        }

        // if we are going into the intermission, don't start any voices
        if cg.intermissionStarted != 0 {
            return;
        }

        #[cfg(feature = "JK2AWARDS")]
        {
            // reward sounds
            reward = qfalse;
            if (*ps).persistant[PERS_CAPTURES as usize]
                != (*ops).persistant[PERS_CAPTURES as usize]
            {
                pushReward(
                    cgs.media.captureAwardSound,
                    cgs.media.medalCapture,
                    (*ps).persistant[PERS_CAPTURES as usize] as c_int,
                );
                reward = qtrue;
                //Com_Printf("capture\n");
            }
            if (*ps).persistant[PERS_IMPRESSIVE_COUNT as usize]
                != (*ops).persistant[PERS_IMPRESSIVE_COUNT as usize]
            {
                sfx = cgs.media.impressiveSound;

                pushReward(
                    sfx,
                    cgs.media.medalImpressive,
                    (*ps).persistant[PERS_IMPRESSIVE_COUNT as usize] as c_int,
                );
                reward = qtrue;
                //Com_Printf("impressive\n");
            }
            if (*ps).persistant[PERS_EXCELLENT_COUNT as usize]
                != (*ops).persistant[PERS_EXCELLENT_COUNT as usize]
            {
                sfx = cgs.media.excellentSound;
                pushReward(
                    sfx,
                    cgs.media.medalExcellent,
                    (*ps).persistant[PERS_EXCELLENT_COUNT as usize] as c_int,
                );
                reward = qtrue;
                //Com_Printf("excellent\n");
            }
            if (*ps).persistant[PERS_GAUNTLET_FRAG_COUNT as usize]
                != (*ops).persistant[PERS_GAUNTLET_FRAG_COUNT as usize]
            {
                sfx = cgs.media.humiliationSound;
                pushReward(
                    sfx,
                    cgs.media.medalGauntlet,
                    (*ps).persistant[PERS_GAUNTLET_FRAG_COUNT as usize] as c_int,
                );
                reward = qtrue;
                //Com_Printf("guantlet frag\n");
            }
            if (*ps).persistant[PERS_DEFEND_COUNT as usize]
                != (*ops).persistant[PERS_DEFEND_COUNT as usize]
            {
                pushReward(
                    cgs.media.defendSound,
                    cgs.media.medalDefend,
                    (*ps).persistant[PERS_DEFEND_COUNT as usize] as c_int,
                );
                reward = qtrue;
                //Com_Printf("defend\n");
            }
            if (*ps).persistant[PERS_ASSIST_COUNT as usize]
                != (*ops).persistant[PERS_ASSIST_COUNT as usize]
            {
                //pushReward(cgs.media.assistSound, cgs.media.medalAssist, ps->persistant[PERS_ASSIST_COUNT]);
                //reward = qtrue;
                //Com_Printf("assist\n");
            }
            // if any of the player event bits changed
            if (*ps).persistant[PERS_PLAYEREVENTS as usize]
                != (*ops).persistant[PERS_PLAYEREVENTS as usize]
            {
                if ((*ps).persistant[PERS_PLAYEREVENTS as usize] & PLAYEREVENT_DENIEDREWARD)
                    != ((*ops).persistant[PERS_PLAYEREVENTS as usize] & PLAYEREVENT_DENIEDREWARD)
                {
                    trap_S_StartLocalSound(cgs.media.deniedSound, CHAN_ANNOUNCER);
                } else if ((*ps).persistant[PERS_PLAYEREVENTS as usize]
                    & PLAYEREVENT_GAUNTLETREWARD)
                    != ((*ops).persistant[PERS_PLAYEREVENTS as usize]
                        & PLAYEREVENT_GAUNTLETREWARD)
                {
                    trap_S_StartLocalSound(cgs.media.humiliationSound, CHAN_ANNOUNCER);
                }
                reward = qtrue;
            }
        }

        #[cfg(not(feature = "JK2AWARDS"))]
        {
            reward = qfalse;
        }

        // lead changes
        if reward == 0 && cgAnnouncerTime < cg.time {
            //
            if cg.warmup == 0 && cgs.gametype != GT_POWERDUEL {
                // never play lead changes during warmup and powerduel
                if (*ps).persistant[PERS_RANK as usize] != (*ops).persistant[PERS_RANK as usize] {
                    if cgs.gametype < GT_TEAM {
                        /*
                        if (  ps->persistant[PERS_RANK] == 0 ) {
                            CG_AddBufferedSound(cgs.media.takenLeadSound);
                            cgAnnouncerTime = cg.time + 3000;
                        } else if ( ps->persistant[PERS_RANK] == RANK_TIED_FLAG ) {
                            //CG_AddBufferedSound(cgs.media.tiedLeadSound);
                        } else if ( ( ops->persistant[PERS_RANK] & ~RANK_TIED_FLAG ) == 0 ) {
                            //rww - only bother saying this if you have more than 1 kill already.
                            //joining the server and hearing "the force is not with you" is silly.
                            if (ps->persistant[PERS_SCORE] > 0)
                            {
                                CG_AddBufferedSound(cgs.media.lostLeadSound);
                                cgAnnouncerTime = cg.time + 3000;
                            }
                        }
                        */
                    }
                }
            }
        }

        // timelimit warnings
        if cgs.timelimit > 0 && cgAnnouncerTime < cg.time {
            let mut msec: c_int;

            msec = cg.time - cgs.levelStartTime;
            if (cg.timelimitWarnings & 4) == 0
                && msec > ((cgs.timelimit * 60 + 2) * 1000) as c_int
            {
                cg.timelimitWarnings |= 1 | 2 | 4;
                //trap_S_StartLocalSound( cgs.media.suddenDeathSound, CHAN_ANNOUNCER );
            } else if (cg.timelimitWarnings & 2) == 0
                && msec > ((cgs.timelimit - 1) * 60 * 1000) as c_int
            {
                cg.timelimitWarnings |= 1 | 2;
                trap_S_StartLocalSound(cgs.media.oneMinuteSound, CHAN_ANNOUNCER);
                cgAnnouncerTime = cg.time + 3000;
            } else if cgs.timelimit > 5
                && (cg.timelimitWarnings & 1) == 0
                && msec > ((cgs.timelimit - 5) * 60 * 1000) as c_int
            {
                cg.timelimitWarnings |= 1;
                trap_S_StartLocalSound(cgs.media.fiveMinuteSound, CHAN_ANNOUNCER);
                cgAnnouncerTime = cg.time + 3000;
            }
        }

        // fraglimit warnings
        if cgs.fraglimit > 0
            && cgs.gametype < GT_CTF
            && cgs.gametype != GT_DUEL
            && cgs.gametype != GT_POWERDUEL
            && cgs.gametype != GT_SIEGE
            && cgAnnouncerTime < cg.time
        {
            highScore = cgs.scores1;
            if (cg.fraglimitWarnings & 4) == 0 && highScore == (cgs.fraglimit - 1) {
                cg.fraglimitWarnings |= 1 | 2 | 4;
                CG_AddBufferedSound(cgs.media.oneFragSound);
                cgAnnouncerTime = cg.time + 3000;
            } else if cgs.fraglimit > 2
                && (cg.fraglimitWarnings & 2) == 0
                && highScore == (cgs.fraglimit - 2)
            {
                cg.fraglimitWarnings |= 1 | 2;
                CG_AddBufferedSound(cgs.media.twoFragSound);
                cgAnnouncerTime = cg.time + 3000;
            } else if cgs.fraglimit > 3
                && (cg.fraglimitWarnings & 1) == 0
                && highScore == (cgs.fraglimit - 3)
            {
                cg.fraglimitWarnings |= 1;
                CG_AddBufferedSound(cgs.media.threeFragSound);
                cgAnnouncerTime = cg.time + 3000;
            }
        }
    }
}

// ===============
// CG_TransitionPlayerState
//
// ===============
pub fn CG_TransitionPlayerState(ps: *mut playerState_t, ops: *mut playerState_t) {
    unsafe {
        // check for changing follow mode
        if (*ps).clientNum != (*ops).clientNum {
            cg.thisFrameTeleport = qtrue;
            // make sure we don't get any unwanted transition effects
            *ops = *ps;
        }

        // damage events (player is getting wounded)
        if (*ps).damageEvent != (*ops).damageEvent && (*ps).damageCount != 0 {
            CG_DamageFeedback((*ps).damageYaw, (*ps).damagePitch, (*ps).damageCount);
        }

        // respawning
        if (*ps).persistant[PERS_SPAWN_COUNT as usize]
            != (*ops).persistant[PERS_SPAWN_COUNT as usize]
        {
            CG_Respawn();
        }

        if cg.mapRestart != 0 {
            CG_Respawn();
            cg.mapRestart = qfalse;
        }

        if (*cg.snap).ps.pm_type != PM_INTERMISSION
            && (*ps).persistant[PERS_TEAM as usize] != TEAM_SPECTATOR
        {
            CG_CheckLocalSounds(ps, ops);
        }

        // check for going low on ammo
        CG_CheckAmmo();

        // run events
        CG_CheckPlayerstateEvents(ps, ops);

        // smooth the ducking viewheight change
        if (*ps).viewheight != (*ops).viewheight {
            cg.duckChange = (*ps).viewheight - (*ops).viewheight;
            cg.duckTime = cg.time;
        }
    }
}

// ============================================================================
// LOCAL STUBS AND EXTERNS - These are declared as stubs for structural coherence
// and should be provided by cg_local.h or related modules
// ============================================================================

extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cg_static_t;
    pub static mut cg_entities: [centity_t; MAX_GENTITIES];
    pub static mut cg_showmiss: cvar_t;
    pub static mut cg_oldPainSounds: cvar_t;

    pub fn trap_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn CG_EntityEvent(cent: *mut centity_t, origin: *const f32);
    pub fn CG_PainEvent(cent: *mut centity_t, health: c_int);
    pub fn CG_Printf(format: *const c_char);
    pub fn CG_AddBufferedSound(sfx: sfxHandle_t);
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn VectorSubtract(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn DotProduct(vec1: *const f32, vec2: *const f32) -> f32;
    pub fn VectorLength(v: *const f32) -> f32;
}

static vec3_origin: [f32; 3] = [0.0; 3];

// ============================================================================
// C ABI TYPE STUBS
// ============================================================================

pub type sfxHandle_t = c_int;
pub type qhandle_t = c_int;

#[repr(C)]
pub struct playerState_t {
    // Stub: minimal fields sufficient for type definition
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub weaponTime: c_int,
    pub weaponDelay: c_int,
    pub health: c_int,
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub clientNum: c_int,
    pub weapon: c_int,
    pub weaponstate: c_int,
    pub viewangles: [f32; 3],
    pub viewheight: c_int,
    pub damageEvent: c_int,
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,
    pub stats: [c_int; 16],
    pub persistant: [c_int; 16],
    pub eventSequence: c_int,
    pub events: [c_int; 8],
    pub eventParms: [c_int; 8],
    pub pm_time: c_int,
    pub serverTime: c_int,
}

#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub nextState: entityState_t,
    pub playerState: *const playerState_t,
    pub lerpOrigin: [f32; 3],
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: trajectorys_t,
    pub apos: trajectorys_t,
    pub time: c_int,
    pub time2: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub origin2: [f32; 3],
    pub oldorigin: [f32; 3],
    pub otherEntityNum: c_int,
    pub otherEntityNum2: c_int,
    pub groundEntityNum: c_int,
    pub constantLight: c_int,
    pub loopSound: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub clientNum: c_int,
    pub frame: c_int,
    pub solid: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub generic1: c_int,
}

#[repr(C)]
pub struct trajectorys_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: [f32; 3],
    pub trDelta: [f32; 3],
}

#[repr(C)]
pub struct refdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: [f32; 3],
    pub viewaxis: [[f32; 3]; 3],
    pub time: c_int,
    pub rdflags: c_int,
}

#[repr(C)]
pub struct cg_media_t {
    pub noAmmoSound: sfxHandle_t,
    pub shieldPierceSound: sfxHandle_t,
    pub hitSound: sfxHandle_t,
    pub hitSoundHighArmor: sfxHandle_t,
    pub hitSoundLowArmor: sfxHandle_t,
    pub hitTeamSound: sfxHandle_t,
    pub captureAwardSound: sfxHandle_t,
    pub medalCapture: qhandle_t,
    pub impressiveSound: sfxHandle_t,
    pub medalImpressive: qhandle_t,
    pub excellentSound: sfxHandle_t,
    pub medalExcellent: qhandle_t,
    pub humiliationSound: sfxHandle_t,
    pub medalGauntlet: qhandle_t,
    pub defendSound: sfxHandle_t,
    pub medalDefend: qhandle_t,
    pub assistSound: sfxHandle_t,
    pub medalAssist: qhandle_t,
    pub deniedSound: sfxHandle_t,
    pub oneMinuteSound: sfxHandle_t,
    pub fiveMinuteSound: sfxHandle_t,
    pub suddenDeathSound: sfxHandle_t,
    pub takenLeadSound: sfxHandle_t,
    pub tiedLeadSound: sfxHandle_t,
    pub lostLeadSound: sfxHandle_t,
    pub oneFragSound: sfxHandle_t,
    pub twoFragSound: sfxHandle_t,
    pub threeFragSound: sfxHandle_t,
}

#[repr(C)]
pub struct cg_static_t {
    pub gametype: c_int,
    pub levelStartTime: c_int,
    pub timelimit: c_int,
    pub fraglimit: c_int,
    pub scores1: c_int,
    pub media: cg_media_t,
}

#[repr(C)]
pub struct snapshot_t {
    pub snapFlags: c_int,
    pub serverTime: c_int,
    pub ps: playerState_t,
}

#[repr(C)]
pub struct cg_t {
    pub clientFrame: c_int,
    pub clientNum: c_int,
    pub time: c_int,
    pub demoPlayback: c_int,
    pub levelStartTime: c_int,
    pub intermissionStarted: c_int,
    pub mapRestart: c_int,
    pub snap: *mut snapshot_t,
    pub nextSnap: *mut snapshot_t,
    pub thisFrameTeleport: c_int,
    pub refdef: *mut refdef_t,
    pub predictedPlayerState: playerState_t,
    pub damageValue: f32,
    pub damageX: f32,
    pub damageY: f32,
    pub v_dmg_time: c_int,
    pub v_dmg_pitch: f32,
    pub v_dmg_roll: f32,
    pub damageTime: c_int,
    pub attackerTime: c_int,
    pub lowAmmoWarning: c_int,
    pub weaponSelectTime: c_int,
    pub weaponSelect: c_int,
    pub duckChange: c_int,
    pub duckTime: c_int,
    pub eventSequence: c_int,
    pub predictableEvents: [c_int; 256],
    pub rewardStack: c_int,
    pub rewardSound: [sfxHandle_t; 10],
    pub rewardShader: [qhandle_t; 10],
    pub rewardCount: [c_int; 10],
    pub warmup: c_int,
    pub timelimitWarnings: c_int,
    pub fraglimitWarnings: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latchedString: *const c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

// ============================================================================
// CONSTANTS
// ============================================================================

const MAX_GENTITIES: usize = 2048;
const MAX_PS_EVENTS: c_int = 4;
const MAX_PREDICTED_EVENTS: c_int = 16;
const MAX_REWARDSTACK: c_int = 10;

const STAT_HEALTH: c_int = 0;
const STAT_WEAPONS: c_int = 4;

const PERS_TEAM: c_int = 0;
const PERS_HITS: c_int = 1;
const PERS_ATTACKEE_ARMOR: c_int = 2;
const PERS_RANK: c_int = 3;
const PERS_SCORE: c_int = 4;
const PERS_CAPTURES: c_int = 5;
const PERS_IMPRESSIVE_COUNT: c_int = 6;
const PERS_EXCELLENT_COUNT: c_int = 7;
const PERS_GAUNTLET_FRAG_COUNT: c_int = 8;
const PERS_DEFEND_COUNT: c_int = 9;
const PERS_ASSIST_COUNT: c_int = 10;
const PERS_PLAYEREVENTS: c_int = 11;
const PERS_SPAWN_COUNT: c_int = 12;

const PM_INTERMISSION: c_int = 4;

const TEAM_SPECTATOR: c_int = 3;

const CHAN_LOCAL_SOUND: c_int = 0;
const CHAN_ANNOUNCER: c_int = 4;

const DAMAGE_TIME: c_int = 500;

const RANK_TIED_FLAG: c_int = 0x4000;

const PLAYEREVENT_DENIEDREWARD: c_int = 1;
const PLAYEREVENT_GAUNTLETREWARD: c_int = 2;

const GT_TEAM: c_int = 3;
const GT_CTF: c_int = 4;
const GT_DUEL: c_int = 1;
const GT_POWERDUEL: c_int = 6;
const GT_SIEGE: c_int = 7;

const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

const PERS_SPAWN_COUNT: usize = 12;
