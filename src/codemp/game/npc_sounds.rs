//! Slice of `NPC_sounds.c` — NPC voice/confusion sound helpers.
//!
//! `G_AddVoiceEvent` (NPC_sounds.c:23) is defined here, its canonical C home (it
//! is a shared combat-talk gate consumed across the NPC subsystem; `npc_combat.rs`
//! imports it). `NPC_AngerSound` (:6) is entirely commented out in the C source and
//! has no real definition.
//!
//! Ported here: `G_AddVoiceEvent` (NPC_sounds.c:23), `NPC_PlayConfusionSound` (:66).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use crate::codemp::game::b_public_h::{BS_DEFAULT, SCF_NO_ALERT_TALK, SCF_NO_COMBAT_TALK};
use crate::codemp::game::bg_public::{
    EV_ANGER1, EV_CHASE1, EV_CONFUSE1, EV_CONFUSE2, EV_CONFUSE3, EV_GIVEUP1, EV_SUSPICIOUS5,
    EV_VICTORY3, PM_DEAD,
};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_public_h::TID_CHAN_VOICE;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::G_SpeechEvent;
use crate::codemp::game::npc_combat::G_ClearEnemy;
use crate::codemp::game::q_math::Q_irand;
use crate::ffi::types::QFALSE;
use crate::trap;

use core::ffi::c_int;
use core::ptr::addr_of;

/// `void G_AddVoiceEvent( gentity_t *self, int event, int speakDebounceTime )`
/// (NPC_sounds.c:23).
///
/// Queues an NPC combat/alert voice line. Bails when `self` has no NPC struct, no client
/// or is dead (`pm_type >= PM_DEAD`), is still inside its `blockedSpeechDebounceTime`, or
/// already has a `TID_CHAN_VOICE` ICARUS task pending. The `SCF_NO_COMBAT_TALK` and
/// `SCF_NO_ALERT_TALK` script flags suppress the matching event ranges. Per the C's note,
/// the line is emitted directly via `G_SpeechEvent` (not `G_AddEvent`) to avoid losing
/// speech events to networking, then a fresh `blockedSpeechDebounceTime` is set (default
/// 5000ms unless `speakDebounceTime` is non-zero). No oracle (entity-state mutation +
/// event emission); verified by review against the C.
///
/// # Safety
/// `self` must point to a valid `gentity_t`; `level` must be initialised. `self->client`/
/// `self->NPC` may be NULL (the early bail paths handle that).
pub(crate) unsafe fn G_AddVoiceEvent(
    self_: *mut gentity_t,
    event: c_int,
    speakDebounceTime: c_int,
) {
    if (*self_).NPC.is_null() {
        return;
    }

    if (*self_).client.is_null() || (*(*self_).client).ps.pm_type >= PM_DEAD {
        return;
    }

    if (*(*self_).NPC).blockedSpeechDebounceTime > (*addr_of!(level)).time {
        return;
    }

    if trap::ICARUS_TaskIDPending(self_, TID_CHAN_VOICE) != QFALSE {
        return;
    }

    if ((*(*self_).NPC).scriptFlags & SCF_NO_COMBAT_TALK) != 0
        && ((event >= EV_ANGER1 && event <= EV_VICTORY3)
            || (event >= EV_CHASE1 && event <= EV_SUSPICIOUS5))
    //(event < EV_FF_1A || event > EV_FF_3C) && (event < EV_RESPOND1 || event > EV_MISSION3) )
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
    (*(*self_).NPC).blockedSpeechDebounceTime = (*addr_of!(level)).time
        + if speakDebounceTime == 0 {
            5000
        } else {
            speakDebounceTime
        };
}

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

pub unsafe fn NPC_PlayConfusionSound(self_: *mut gentity_t) {
    if (*self_).health > 0 {
        if !(*self_).enemy.is_null() //was mad
            || TIMER_Done(self_, c"enemyLastVisible".as_ptr()) == QFALSE //saw something suspicious
            || (*(*self_).client).renderInfo.lookTarget == 0
        //was looking at player
        {
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //make sure we say this
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE2, EV_CONFUSE3), 2000);
        } else if !(*self_).NPC.is_null()
            && (*(*self_).NPC).investigateDebounceTime + (*(*self_).NPC).pauseTime
                > (*addr_of!(level)).time
        //was checking something out
        {
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //make sure we say this
            G_AddVoiceEvent(self_, EV_CONFUSE1, 2000);
        }
        //G_AddVoiceEvent( self, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000 );
    }
    //reset him to be totally unaware again
    TIMER_Set(self_, c"enemyLastVisible".as_ptr(), 0);
    (*(*self_).NPC).tempBehavior = BS_DEFAULT;

    //self->NPC->behaviorState = BS_PATROL;
    G_ClearEnemy(self_); //FIXME: or just self->enemy = NULL;?

    (*(*self_).NPC).investigateCount = 0;
}
