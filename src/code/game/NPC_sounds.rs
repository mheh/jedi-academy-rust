//NPC_sounds.rs

// leave this line at the top for all NPC_xxxx.cpp files...
// (C header: g_headers.h)
// (C header: b_local.h)
// (C header: Q3_Interface.h)

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char};

// ============================================================================
// Types - Minimal stubs with only fields accessed by this file
// ============================================================================

/// Forward declarations and partial definitions for types from b_public.h, g_local.h
#[repr(C)]
pub struct playerState_t {
    pub pm_type: c_int,
    pub powerups: [c_int; 16],
    // ... other fields not used in this file
}

#[repr(C)]
pub struct renderInfo_t {
    pub lookTarget: c_int,
    // ... other fields not used in this file
}

/// Partial definition of gNPC_t - only fields used in this file.
/// For full definition, see codemp/game/b_public_h.rs
#[repr(C)]
pub struct gNPC_t {
    pub timeOfDeath: c_int,
    pub touchedByPlayer: *mut gentity_t,
    pub enemyLastVisibility: c_int,
    pub aimTime: c_int,
    pub desiredYaw: f32,
    pub desiredPitch: f32,
    pub lockedDesiredYaw: f32,
    pub lockedDesiredPitch: f32,
    pub aimingBeam: *mut gentity_t,
    pub enemyLastSeenLocation: [f32; 3],
    pub enemyLastSeenTime: c_int,
    pub enemyLastHeardLocation: [f32; 3],
    pub enemyLastHeardTime: c_int,
    pub lastAlertID: c_int,
    pub eFlags: c_int,
    pub aiFlags: c_int,
    pub currentAmmo: c_int,
    pub shotTime: c_int,
    pub burstCount: c_int,
    pub burstMin: c_int,
    pub burstMean: c_int,
    pub burstMax: c_int,
    pub burstSpacing: c_int,
    pub attackHold: c_int,
    pub attackHoldTime: c_int,
    pub shootAngles: [f32; 3],
    pub rank: c_int,
    pub behaviorState: c_int,
    pub defaultBehavior: c_int,
    pub tempBehavior: c_int,
    pub ignorePain: c_int,
    pub duckDebounceTime: c_int,
    pub walkDebounceTime: c_int,
    pub enemyCheckDebounceTime: c_int,
    pub investigateDebounceTime: c_int,
    pub investigateCount: c_int,
    pub investigateGoal: [f32; 3],
    pub investigateSoundDebounceTime: c_int,
    pub greetingDebounceTime: c_int,
    pub eventOwner: *mut gentity_t,
    pub coverTarg: *mut gentity_t,
    pub jumpState: c_int,
    pub followDist: f32,
    pub tempGoal: *mut gentity_t,
    pub goalEntity: *mut gentity_t,
    pub lastGoalEntity: *mut gentity_t,
    pub eventualGoal: *mut gentity_t,
    pub captureGoal: *mut gentity_t,
    pub defendEnt: *mut gentity_t,
    pub greetEnt: *mut gentity_t,
    pub goalTime: c_int,
    pub straightToGoal: c_int,
    pub distToGoal: f32,
    pub navTime: c_int,
    pub blockingEntNum: c_int,
    pub blockedSpeechDebounceTime: c_int,
    pub lastSideStepSide: c_int,
    pub sideStepHoldTime: c_int,
    pub homeWp: c_int,
    pub group: *mut c_int, // AIGroupInfo_t*
    // ... remaining fields truncated; full definition in b_public_h.rs
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub NPC_class: c_int,
    pub renderInfo: renderInfo_t,
    // ... other fields not used in this file
}

#[repr(C)]
pub struct gentity_t {
    pub NPC: *mut gNPC_t,
    pub client: *mut gclient_t,
    pub health: c_int,
    pub enemy: *mut gentity_t,
    // ... other fields not used in this file
}

#[repr(C)]
pub struct level_s {
    pub time: c_int,
    // ... other fields not needed
}

// ============================================================================
// Constants
// ============================================================================

const PM_DEAD: c_int = 4;
const CLASS_SABOTEUR: c_int = 6; // CLASS_SABOTEUR
const PW_CLOAKED: usize = 15;
const PW_UNCLOAKING: usize = 16;
const EV_ANGER1: c_int = 3;
const EV_VICTORY3: c_int = 8;
const EV_CHASE1: c_int = 9;
const EV_SUSPICIOUS5: c_int = 16;
const EV_GIVEUP1: c_int = 19;
const EV_CONFUSE1: c_int = 25;
const EV_CONFUSE2: c_int = 26;
const EV_CONFUSE3: c_int = 27;
const SCF_NO_COMBAT_TALK: c_int = 0x00000200;
const SCF_NO_ALERT_TALK: c_int = 0x02000000;
const BS_DEFAULT: c_int = 0;
const TID_CHAN_VOICE: c_int = 4;

// ============================================================================
// External Functions and Globals
// ============================================================================

extern "C" {
    pub static mut level: level_s;

    pub fn G_SpeechEvent(self_: *mut gentity_t, event: c_int);
    pub fn Q3_TaskIDPending(self_: *mut gentity_t, task_id: c_int) -> c_int;
    pub fn G_ClearEnemy(self_: *mut gentity_t);
    pub fn TIMER_Done(self_: *mut gentity_t, name: *const c_char) -> c_int;
    pub fn TIMER_Set(self_: *mut gentity_t, name: *const c_char, duration: c_int);
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
}

// ============================================================================
// Functions
// ============================================================================

/*
void NPC_AngerSound (void)
{
	if(NPCInfo->investigateSoundDebounceTime)
		return;

	NPCInfo->investigateSoundDebounceTime = 1;

//	switch((int)NPC->client->race)
//	{
//	case RACE_KLINGON:
		//G_Sound(NPC, G_SoundIndex(va("sound/mgtest/klingon/talk%d.wav",	Q_irand(1, 4))));
//		break;
//	}
}
*/

pub unsafe fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int) {
    if (*self_).NPC.is_null() {
        return;
    }

    if (*self_).client.is_null() || (*(*self_).client).ps.pm_type >= PM_DEAD {
        return;
    }

    if (*(*self_).NPC).blockedSpeechDebounceTime > level.time {
        return;
    }

    if Q3_TaskIDPending(self_, TID_CHAN_VOICE) != 0 {
        return;
    }

    if !(*self_).client.is_null() && (*(*self_).client).NPC_class == CLASS_SABOTEUR {
        if ((*(*self_).client).ps.powerups[PW_CLOAKED] != 0)
            || ((*(*self_).client).ps.powerups[PW_UNCLOAKING] > level.time)
        {
            //I'm cloaked (or still decloaking), so don't talk and give away my position...
            //don't make any combat voice noises, but still make pain and death sounds
            if ((event >= EV_ANGER1 && event <= EV_VICTORY3)
                || (event >= EV_CHASE1 && event <= EV_SUSPICIOUS5))
            {
                return;
            }

            if event >= EV_GIVEUP1 && event <= EV_SUSPICIOUS5 {
                return;
            }
        }
    }

    if ((*(*self_).NPC).scriptFlags & SCF_NO_COMBAT_TALK) != 0
        && ((event >= EV_ANGER1 && event <= EV_VICTORY3) || (event >= EV_CHASE1 && event <= EV_SUSPICIOUS5))
    {
        return;
    }

    if ((*(*self_).NPC).scriptFlags & SCF_NO_ALERT_TALK) != 0
        && (event >= EV_GIVEUP1 && event <= EV_SUSPICIOUS5)
    {
        return;
    }
    //FIXME: Also needs to check for teammates. Don't want
    //		everyone babbling at once

    //NOTE: was losing too many speech events, so we do it directly now, screw networking!
    //G_AddEvent( self, event, 0 );
    G_SpeechEvent(self_, event);

    //won't speak again for 5 seconds (unless otherwise specified)
    (*(*self_).NPC).blockedSpeechDebounceTime =
        level.time + if speakDebounceTime == 0 { 5000 } else { speakDebounceTime };
}

pub unsafe fn NPC_PlayConfusionSound(self_: *mut gentity_t) {
    if (*self_).health > 0 {
        if (!(*self_).enemy.is_null()) //was mad
            || (TIMER_Done(self_, b"enemyLastVisible\0".as_ptr() as *const c_char) != 0) //saw something suspicious
            || ((*(*self_).client).renderInfo.lookTarget == 0) //was looking at player
        {
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //make sure we say this
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE2, EV_CONFUSE3), 2000);
        } else if !(*self_).NPC.is_null()
            && ((*(*self_).NPC).investigateDebounceTime + (*(*self_).NPC).pauseTime > level.time)
        {
            //was checking something out
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //make sure we say this
            G_AddVoiceEvent(self_, EV_CONFUSE1, 2000);
        }
        //G_AddVoiceEvent( self, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000 );
    }
    //reset him to be totally unaware again
    TIMER_Set(self_, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
    (*(*self_).NPC).tempBehavior = BS_DEFAULT;

    //self->NPC->behaviorState = BS_PATROL;
    G_ClearEnemy(self_); //FIXME: or just self->enemy = NULL;?

    (*(*self_).NPC).investigateCount = 0;
}
