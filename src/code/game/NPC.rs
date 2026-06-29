#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types,
         unused_mut, unused_variables, dead_code, unused_imports, clippy::all)]

//
// NPC.cpp - generic functions
//

// leave this line at the top for all NPC_xxxx.cpp files...
use crate::code::game::g_headers_h::*;
use crate::code::game::b_local_h::*;
use crate::code::game::anims_h::*;
use crate::code::game::g_functions_h::*;
use crate::code::game::say_h::*;
use crate::code::game::Q3_Interface_h::*;
use crate::code::game::g_vehicles_h::*;

use core::ffi::{c_int, c_float, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut, null};

extern "C" {
    pub static mut playerMins: vec3_t;
    pub static mut playerMaxs: vec3_t;
    // extern void PM_SetAnimFinal(int *torsoAnim,int *legsAnim,int type,int anim,int priority,int *torsoAnimTimer,int *legsAnimTimer,gentity_t *gent);
    pub fn PM_SetTorsoAnimTimer(ent: *mut gentity_t, torsoAnimTimer: *mut c_int, time: c_int);
    pub fn PM_SetLegsAnimTimer(ent: *mut gentity_t, legsAnimTimer: *mut c_int, time: c_int);
    pub fn NPC_BSNoClip();
    pub fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    pub fn NPC_ApplyRoff();
    pub fn NPC_TempLookTarget(self_: *mut gentity_t, lookEntNum: c_int, minLookTime: c_int, maxLookTime: c_int);
    pub fn NPC_CheckPlayerAim();
    pub fn NPC_CheckAllClear();
    pub fn NPC_CheckLookTarget(self_: *mut gentity_t) -> qboolean;
    pub fn NPC_SetLookTarget(self_: *mut gentity_t, entNum: c_int, clearTime: c_int);
    pub fn Mark1_dying(self_: *mut gentity_t);
    pub fn NPC_BSCinematic();
    pub fn GetTime(lastTime: c_int) -> c_int;
    pub fn G_CheckCharmed(self_: *mut gentity_t);
    pub fn Boba_Flying(self_: *mut gentity_t) -> qboolean;
    pub fn RT_Flying(self_: *mut gentity_t) -> qboolean;
    pub fn Jedi_CultistDestroyer(self_: *mut gentity_t) -> qboolean;
    pub fn Boba_Update();
    pub fn Boba_Flee() -> bool;
    pub fn Boba_Tactics() -> bool;
    pub fn BubbleShield_Update();
    pub fn PM_LockedAnim(anim: c_int) -> qboolean;
    pub static mut g_dismemberment: *mut cvar_t;
    pub static mut g_saberRealisticCombat: *mut cvar_t;
    pub static mut g_corpseRemovalTime: *mut cvar_t;
    pub static mut debug_subdivision: *mut cvar_t;
    pub static mut stop_icarus: qboolean;
    pub static mut eventClearTime: c_int;
    pub static mut showBBoxes: qboolean;
    pub fn InFOVFromPlayerView(ent: *mut gentity_t, hFOV: c_int, vFOV: c_int) -> qboolean;
    pub fn CG_Cube(mins: *mut f32, maxs: *mut f32, color: *mut f32, alpha: f32);
    pub fn CG_Line(start: *mut f32, end: *mut f32, color: *mut f32, alpha: f32);
    pub fn CG_Cylinder(start: *mut f32, end: *mut f32, radius: f32, color: *mut f32);
    pub fn JET_Flying(self_: *mut gentity_t) -> qboolean;
    pub fn JET_FlyStart(self_: *mut gentity_t);
    pub fn JET_FlyStop(self_: *mut gentity_t);
    pub fn NPC_BSEmplaced();
    pub fn NPC_CheckSurrender() -> qboolean;
    pub fn NPC_BSRT_Default();
    pub fn NPC_BSCivilian_Default(bState: c_int);
    pub fn NPC_BSSD_Default();
    pub fn NPC_BehaviorSet_Trooper(bState: c_int);
    pub fn NPC_IsTrooper(ent: *mut gentity_t) -> bool;
    pub fn Pilot_MasterUpdate() -> bool;
    pub fn G_ParseAnimFileSet(skeletonName: *const c_char, modelName: *const c_char) -> c_int;
    pub fn NPC_MaxDistSquaredForWeapon() -> f32;
    pub fn PM_SetAnimFinal(
        torsoAnim: *mut c_int,
        legsAnim: *mut c_int,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        torsoAnimTimer: *mut c_int,
        legsAnimTimer: *mut c_int,
        ent: *mut gentity_t,
        iBlend: c_int,
    );
    pub fn NPC_BSImperialProbe_Attack();
    pub fn NPC_BSImperialProbe_Patrol();
    pub fn NPC_BSImperialProbe_Wait();
    pub fn NPC_BSSeeker_Default();
    pub fn NPC_BSRemote_Default();
    pub fn NPC_BSSentry_Default();
    #[cfg(feature = "ai_timers")]
    pub static mut AITime: c_int;
}

//Local Variables
// ai debug cvars
pub static mut debugNPCAI: *mut cvar_t = null_mut();           // used to print out debug info about the bot AI
pub static mut debugNPCFreeze: *mut cvar_t = null_mut();       // set to disable bot ai and temporarily freeze them in place
pub static mut debugNPCName: *mut cvar_t = null_mut();
pub static mut d_saberCombat: *mut cvar_t = null_mut();
pub static mut d_JediAI: *mut cvar_t = null_mut();
pub static mut d_noGroupAI: *mut cvar_t = null_mut();
pub static mut d_asynchronousGroupAI: *mut cvar_t = null_mut();
pub static mut d_slowmodeath: *mut cvar_t = null_mut();

pub static mut NPC: *mut gentity_t = null_mut();
pub static mut NPCInfo: *mut gNPC_t = null_mut();
pub static mut client: *mut gclient_t = null_mut();
pub static mut ucmd: usercmd_t = unsafe { core::mem::zeroed() };
pub static mut enemyVisibility: visibility_t = unsafe { core::mem::zeroed() };

pub static mut _saved_NPC: *mut gentity_t = null_mut();
pub static mut _saved_NPCInfo: *mut gNPC_t = null_mut();
pub static mut _saved_client: *mut gclient_t = null_mut();
pub static mut _saved_ucmd: usercmd_t = unsafe { core::mem::zeroed() };

pub static mut NPCDEBUG_RED: vec3_t = [1.0, 0.0, 0.0];
pub static mut NPCDEBUG_GREEN: vec3_t = [0.0, 1.0, 0.0];
pub static mut NPCDEBUG_BLUE: vec3_t = [0.0, 0.0, 1.0];
pub static mut NPCDEBUG_LIGHT_BLUE: vec3_t = [0.3, 0.7, 1.0];

const REMOVE_DISTANCE: c_int = 128;
const REMOVE_DISTANCE_SQR: c_int = REMOVE_DISTANCE * REMOVE_DISTANCE;

pub unsafe fn CorpsePhysics(self_: *mut gentity_t) {
    // run the bot through the server like it was a real client
    addr_of_mut!(ucmd).write(core::mem::zeroed());
    ClientThink((*self_).s.number, addr_of_mut!(ucmd));
    VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());

    //FIXME: match my pitch and roll for the slope of my groundPlane
    if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE && ((*self_).flags & FL_DISINTEGRATED) == 0 {
        //on the ground
        //FIXME: check 4 corners
        pitch_roll_for_slope(self_, null_mut(), null_mut(), qfalse);
    }

    if eventClearTime == level.time + ALERT_CLEAR_TIME {
        //events were just cleared out so add me again
        if ((*(*self_).client).ps.eFlags & EF_NODRAW) == 0 {
            AddSightEvent((*self_).enemy, (*self_).currentOrigin.as_ptr(), 384, AEL_DISCOVERED);
        }
    }

    if level.time - (*self_).s.time > 3000 {
        //been dead for 3 seconds
        if (*debug_subdivision).integer == 0 && (*g_saberRealisticCombat).integer == 0 {
            //can't be dismembered once dead
            if (*(*self_).client).NPC_class != CLASS_PROTOCOL {
                (*(*self_).client).dismembered = true;
            }
        }
    }

    if level.time - (*self_).s.time > 500 {
        //don't turn "nonsolid" until about 1 second after actual death

        if ((*(*self_).client).NPC_class != CLASS_MARK1) && ((*(*self_).client).NPC_class != CLASS_INTERROGATOR) {
            // The Mark1 & Interrogator stays solid.
            (*self_).contents = CONTENTS_CORPSE;
        }

        if !(*self_).message.is_null() {
            (*self_).contents |= CONTENTS_TRIGGER;
        }
    }
}

/*
----------------------------------------
NPC_RemoveBody

Determines when it's ok to ditch the corpse
----------------------------------------
*/
pub unsafe fn G_OkayToRemoveCorpse(self_: *mut gentity_t) -> qboolean {
    //if we're still on a vehicle, we won't remove ourselves until we get ejected
    if !(*self_).client.is_null() && (*(*self_).client).NPC_class != CLASS_VEHICLE && (*self_).s.m_iVehicleNum != 0 {
        let pVeh: *mut Vehicle_t = (*g_entities.as_mut_ptr().add((*self_).s.m_iVehicleNum as usize)).m_pVehicle;
        if !pVeh.is_null() {
            if ((*(*pVeh).m_pVehicleInfo).Eject)(pVeh, self_, qtrue) == qfalse {
                //dammit, still can't get off the vehicle...
                return qfalse;
            }
        } else {
            debug_assert!(false);
            #[cfg(not(feature = "final_build"))]
            {
                Com_Printf(
                    b"^1ERROR: Dead pilot's vehicle removed while corpse was riding it (pilot: %s)???\n\0"
                        .as_ptr() as *const c_char,
                    (*self_).targetname,
                );
            }
        }
    }

    if !(*self_).message.is_null() {
        //I still have a key
        return qfalse;
    }

    if IIcarusInterface::GetIcarus().IsRunning((*self_).m_iIcarusID) {
        //still running a script
        return qfalse;
    }

    if !(*self_).activator.is_null()
        && !(*(*self_).activator).client.is_null()
        && (((*(*(*self_).activator).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0
            || ((*(*(*self_).activator).client).ps.eFlags & EF_HELD_BY_SAND_CREATURE) != 0
            || ((*(*(*self_).activator).client).ps.eFlags & EF_HELD_BY_WAMPA) != 0)
    {
        //still holding a victim?
        return qfalse;
    }

    if !(*self_).client.is_null()
        && (((*(*self_).client).ps.eFlags & EF_HELD_BY_RANCOR) != 0
            || ((*(*self_).client).ps.eFlags & EF_HELD_BY_SAND_CREATURE) != 0
            || ((*(*self_).client).ps.eFlags & EF_HELD_BY_WAMPA) != 0)
    {
        //being held by a creature
        return qfalse;
    }

    if (*(*self_).client).ps.heldByClient < ENTITYNUM_WORLD {
        //being dragged
        return qfalse;
    }

    //okay, well okay to remove us...?
    qtrue
}

pub unsafe fn NPC_RemoveBody(self_: *mut gentity_t) {
    (*self_).nextthink = level.time + FRAMETIME / 2;

    //run physics at 20fps
    CorpsePhysics(self_);

    if (*(*self_).NPC).nextBStateThink <= level.time {
        //run logic at 10 fps
        if (*self_).m_iIcarusID != IIcarusInterface::ICARUS_INVALID && stop_icarus == qfalse {
            IIcarusInterface::GetIcarus().Update((*self_).m_iIcarusID);
        }
        (*(*self_).NPC).nextBStateThink = level.time + FRAMETIME;

        if G_OkayToRemoveCorpse(self_) == qfalse {
            return;
        }

        // I don't consider this a hack, it's creative coding . . .
        // I agree, very creative... need something like this for ATST and GALAKMECH too!
        if (*(*self_).client).NPC_class == CLASS_MARK1 {
            Mark1_dying(self_);
        }

        // Since these blow up, remove the bounding box.
        if (*(*self_).client).NPC_class == CLASS_REMOTE
            || (*(*self_).client).NPC_class == CLASS_SENTRY
            || (*(*self_).client).NPC_class == CLASS_PROBE
            || (*(*self_).client).NPC_class == CLASS_INTERROGATOR
            || (*(*self_).client).NPC_class == CLASS_PROBE
            || (*(*self_).client).NPC_class == CLASS_MARK2
        {
            G_FreeEntity(self_);
            return;
        }

        //FIXME: don't ever inflate back up?
        (*self_).maxs[2] = (*(*self_).client).renderInfo.eyePoint[2] - (*self_).currentOrigin[2] + 4.0;
        if (*self_).maxs[2] < -8.0 {
            (*self_).maxs[2] = -8.0;
        }

        if ((*(*self_).NPC).aiFlags & NPCAI_HEAL_ROSH) != 0 {
            //kothos twins' bodies are never removed
            return;
        }

        if (*(*self_).client).NPC_class == CLASS_GALAKMECH {
            //never disappears
            return;
        }

        if !(*self_).NPC.is_null() && (*(*self_).NPC).timeOfDeath <= level.time {
            (*(*self_).NPC).timeOfDeath = level.time + 1000;
            // Only do all of this nonsense for Scav boys ( and girls )
        ///	if ( self->client->playerTeam == TEAM_SCAVENGERS || self->client->playerTeam == TEAM_KLINGON
        //		|| self->client->playerTeam == TEAM_HIROGEN || self->client->playerTeam == TEAM_MALON )
            // should I check NPC_class here instead of TEAM ? - dmv
            if (*(*self_).client).playerTeam == TEAM_ENEMY || (*(*self_).client).NPC_class == CLASS_PROTOCOL {
                (*self_).nextthink = level.time + FRAMETIME; // try back in a second

                if DistanceSquared(
                    (*g_entities.as_ptr()).currentOrigin.as_ptr(),
                    (*self_).currentOrigin.as_ptr(),
                ) <= REMOVE_DISTANCE_SQR as f32
                {
                    return;
                }

                if InFOVFromPlayerView(self_, 110, 90) != qfalse {
                    // generous FOV check
                    if NPC_ClearLOS(g_entities.as_mut_ptr(), (*self_).currentOrigin.as_ptr()) != qfalse {
                        return;
                    }
                }
            }

            //FIXME: there are some conditions - such as heavy combat - in which we want
            //			to remove the bodies... but in other cases it's just weird, like
            //			when they're right behind you in a closed room and when they've been
            //			placed as dead NPCs by a designer...
            //			For now we just assume that a corpse with no enemy was
            //			placed in the map as a corpse
            if !(*self_).enemy.is_null() {
                if !(*self_).client.is_null()
                    && (*(*self_).client).ps.saberEntityNum > 0
                    && (*(*self_).client).ps.saberEntityNum < ENTITYNUM_WORLD
                {
                    let saberent: *mut gentity_t =
                        g_entities.as_mut_ptr().add((*(*self_).client).ps.saberEntityNum as usize);
                    if !saberent.is_null() {
                        G_FreeEntity(saberent);
                    }
                }
                G_FreeEntity(self_);
            }
        }
    }
}

/*
----------------------------------------
NPC_RemoveBody

Determines when it's ok to ditch the corpse
----------------------------------------
*/

pub unsafe fn BodyRemovalPadTime(ent: *mut gentity_t) -> c_int {
    let mut time: c_int;

    if ent.is_null() || (*ent).client.is_null() {
        return 0;
    }
/*
	switch ( ent->client->playerTeam )
	{
	case TEAM_KLINGON:	// no effect, we just remove them when the player isn't looking
	case TEAM_SCAVENGERS:
	case TEAM_HIROGEN:
	case TEAM_MALON:
	case TEAM_IMPERIAL:
	case TEAM_STARFLEET:
		time = 10000; // 15 secs.
		break;

	case TEAM_BORG:
		time = 2000;
		break;

	case TEAM_STASIS:
		return qtrue;
		break;

	case TEAM_FORGE:
		time = 1000;
		break;

	case TEAM_BOTS:
//		if (!Q_stricmp( ent->NPC_type, "mouse" ))
//		{
			time = 0;
//		}
//		else
//		{
//			time = 10000;
//		}
		break;

	case TEAM_8472:
		time = 2000;
		break;

	default:
		// never go away
		time = Q3_INFINITE;
		break;
	}
*/
    // team no longer indicates species/race, so in this case we'd use NPC_class, but
    if (*(*ent).client).NPC_class == CLASS_MOUSE
        || (*(*ent).client).NPC_class == CLASS_GONK
        || (*(*ent).client).NPC_class == CLASS_R2D2
        || (*(*ent).client).NPC_class == CLASS_R5D2
        //case CLASS_PROTOCOL:
        || (*(*ent).client).NPC_class == CLASS_MARK1
        || (*(*ent).client).NPC_class == CLASS_MARK2
        || (*(*ent).client).NPC_class == CLASS_PROBE
        || (*(*ent).client).NPC_class == CLASS_SEEKER
        || (*(*ent).client).NPC_class == CLASS_REMOTE
        || (*(*ent).client).NPC_class == CLASS_SENTRY
        || (*(*ent).client).NPC_class == CLASS_INTERROGATOR
    {
        time = 0;
    } else {
        // never go away
        if (*g_corpseRemovalTime).integer <= 0 {
            time = Q3_INFINITE;
        } else {
            time = (*g_corpseRemovalTime).integer * 1000;
        }
    }

    time
}


/*
----------------------------------------
NPC_RemoveBodyEffect

Effect to be applied when ditching the corpse
----------------------------------------
*/

unsafe fn NPC_RemoveBodyEffect() {
//	vec3_t		org;
//	gentity_t	*tent;

    if NPC.is_null() || (*NPC).client.is_null() || ((*NPC).s.eFlags & EF_NODRAW) != 0 {
        return;
    }
/*
	switch(NPC->client->playerTeam)
	{
	case TEAM_STARFLEET:
		//FIXME: Starfleet beam out
		break;

	case TEAM_BOTS:
//		VectorCopy( NPC->currentOrigin, org );
//		org[2] -= 16;
//		tent = G_TempEntity( org, EV_BOT_EXPLODE );
//		tent->owner = NPC;

		break;

	default:
		break;
	}
*/


    // team no longer indicates species/race, so in this case we'd use NPC_class, but

    // stub code
    if (*(*NPC).client).NPC_class == CLASS_PROBE
        || (*(*NPC).client).NPC_class == CLASS_SEEKER
        || (*(*NPC).client).NPC_class == CLASS_REMOTE
        || (*(*NPC).client).NPC_class == CLASS_SENTRY
        || (*(*NPC).client).NPC_class == CLASS_GONK
        || (*(*NPC).client).NPC_class == CLASS_R2D2
        || (*(*NPC).client).NPC_class == CLASS_R5D2
        //case CLASS_PROTOCOL:
        || (*(*NPC).client).NPC_class == CLASS_MARK1
        || (*(*NPC).client).NPC_class == CLASS_MARK2
        || (*(*NPC).client).NPC_class == CLASS_INTERROGATOR
        || (*(*NPC).client).NPC_class == CLASS_ATST // yeah, this is a little weird, but for now I'm listing all droids
    {
    //	VectorCopy( NPC->currentOrigin, org );
    //	org[2] -= 16;
    //	tent = G_TempEntity( org, EV_BOT_EXPLODE );
    //	tent->owner = NPC;
    } else {
    }


}


/*
====================================================================
void pitch_roll_for_slope (edict_t *forwhom, vec3_t *slope, vec3_t storeAngles )

MG

This will adjust the pitch and roll of a monster to match
a given slope - if a non-'0 0 0' slope is passed, it will
use that value, otherwise it will use the ground underneath
the monster.  If it doesn't find a surface, it does nothinh\g
and returns.
====================================================================
*/

pub unsafe fn pitch_roll_for_slope(forwhom: *mut gentity_t, pass_slope: *mut f32, storeAngles: *mut f32, keepPitch: qboolean) {
    let mut slope: vec3_t = [0.0; 3];
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut startspot: vec3_t = [0.0; 3];
    let mut endspot: vec3_t = [0.0; 3];
    let mut new_angles: vec3_t = [0.0, 0.0, 0.0];
    let mut pitch: f32;
    let mut mod_: f32;
    let mut dot: f32;

    //if we don't have a slope, get one
    if pass_slope.is_null() || VectorCompare(vec3_origin.as_ptr(), pass_slope) != qfalse {
        let mut trace: trace_t = core::mem::zeroed();

        VectorCopy((*forwhom).currentOrigin.as_ptr(), startspot.as_mut_ptr());
        startspot[2] += (*forwhom).mins[2] + 4.0;
        VectorCopy(startspot.as_ptr(), endspot.as_mut_ptr());
        endspot[2] -= 300.0;
        gi.trace(
            &mut trace,
            (*forwhom).currentOrigin.as_ptr(),
            vec3_origin.as_ptr(),
            vec3_origin.as_ptr(),
            endspot.as_ptr(),
            (*forwhom).s.number,
            MASK_SOLID,
        );
//		if(trace_fraction>0.05&&forwhom.movetype==MOVETYPE_STEP)
//			forwhom.flags(-)FL_ONGROUND;

        if trace.fraction >= 1.0 {
            return;
        }

        if addr_of!(trace.plane).is_null() {
            return;
        }

        if VectorCompare(vec3_origin.as_ptr(), trace.plane.normal.as_ptr()) != qfalse {
            return;
        }

        VectorCopy(trace.plane.normal.as_ptr(), slope.as_mut_ptr());
    } else {
        VectorCopy(pass_slope, slope.as_mut_ptr());
    }

    let mut oldPitch: f32 = 0.0;
    if !(*forwhom).client.is_null() && (*(*forwhom).client).NPC_class == CLASS_VEHICLE {
        //special code for vehicles
        let pVeh: *mut Vehicle_t = (*forwhom).m_pVehicle;

        let mut tempAngles: vec3_t = [0.0; 3];
        tempAngles[PITCH as usize] = 0.0;
        tempAngles[ROLL as usize] = 0.0;
        tempAngles[YAW as usize] = (*pVeh).m_vOrientation[YAW as usize];
        AngleVectors(tempAngles.as_ptr(), ovf.as_mut_ptr(), ovr.as_mut_ptr(), null_mut());
    } else {
        oldPitch = (*forwhom).currentAngles[PITCH as usize];
        AngleVectors((*forwhom).currentAngles.as_ptr(), ovf.as_mut_ptr(), ovr.as_mut_ptr(), null_mut());
    }

    vectoangles(slope.as_ptr(), new_angles.as_mut_ptr());
    pitch = new_angles[PITCH as usize] + 90.0;
    if keepPitch != qfalse {
        pitch += oldPitch;
    }
    new_angles[ROLL as usize] = 0.0;
    new_angles[PITCH as usize] = 0.0;

    AngleVectors(new_angles.as_ptr(), nvf.as_mut_ptr(), null_mut(), null_mut());

    mod_ = DotProduct(nvf.as_ptr(), ovr.as_ptr());

    if mod_ < 0.0 {
        mod_ = -1.0;
    } else {
        mod_ = 1.0;
    }

    dot = DotProduct(nvf.as_ptr(), ovf.as_ptr());

    if !storeAngles.is_null() {
        *storeAngles.add(PITCH as usize) = dot * pitch;
        *storeAngles.add(ROLL as usize) = (1.0 - Q_fabs(dot)) * pitch * mod_;
    } else if !(*forwhom).client.is_null() {
        (*(*forwhom).client).ps.viewangles[PITCH as usize] = dot * pitch;
        (*(*forwhom).client).ps.viewangles[ROLL as usize] = (1.0 - Q_fabs(dot)) * pitch * mod_;
        let oldmins2: f32 = (*forwhom).mins[2];
        (*forwhom).mins[2] = -24.0 + 12.0 * ((*(*forwhom).client).ps.viewangles[PITCH as usize]).abs() / 180.0;
        //FIXME: if it gets bigger, move up
        if oldmins2 > (*forwhom).mins[2] {
            //our mins is now lower, need to move up
            //FIXME: trace?
            (*(*forwhom).client).ps.origin[2] += oldmins2 - (*forwhom).mins[2];
            (*forwhom).currentOrigin[2] = (*(*forwhom).client).ps.origin[2];
            gi.linkentity(forwhom);
        }
    } else {
        (*forwhom).currentAngles[PITCH as usize] = dot * pitch;
        (*forwhom).currentAngles[ROLL as usize] = (1.0 - Q_fabs(dot)) * pitch * mod_;
    }
}

/*
void NPC_PostDeathThink( void )
{
	float	mostdist;
	trace_t trace1, trace2, trace3, trace4, movetrace;
	vec3_t	org, endpos, startpos, forward, right;
	int		whichtrace = 0;
	float	cornerdist[4];
	qboolean	frontbackbothclear = false;
	qboolean	rightleftbothclear = false;

	if( NPC->client->ps.groundEntityNum == ENTITYNUM_NONE || !VectorCompare( vec3_origin, NPC->client->ps.velocity ) )
	{
		if ( NPC->client->ps.groundEntityNum != ENTITYNUM_NONE && NPC->client->ps.friction == 1.0 )//check avelocity?
		{
			pitch_roll_for_slope( NPC );
		}

		return;
	}

	cornerdist[FRONT] = cornerdist[BACK] = cornerdist[RIGHT] = cornerdist[LEFT] = 0.0f;

	mostdist = MIN_DROP_DIST;

	AngleVectors( NPC->currentAngles, forward, right, NULL );
	VectorCopy( NPC->currentOrigin, org );
	org[2] += NPC->mins[2];

	VectorMA( org, NPC->dead_size, forward, startpos );
	VectorCopy( startpos, endpos );
	endpos[2] -= 128;
	gi.trace( &trace1, startpos, vec3_origin, vec3_origin, endpos, NPC->s.number, MASK_SOLID, );
	if( !trace1.allsolid && !trace1.startsolid )
	{
		cornerdist[FRONT] = trace1.fraction;
		if ( trace1.fraction > mostdist )
		{
			mostdist = trace1.fraction;
			whichtrace = 1;
		}
	}

	VectorMA( org, -NPC->dead_size, forward, startpos );
	VectorCopy( startpos, endpos );
	endpos[2] -= 128;
	gi.trace( &trace2, startpos, vec3_origin, vec3_origin, endpos, NPC->s.number, MASK_SOLID );
	if( !trace2.allsolid && !trace2.startsolid )
	{
		cornerdist[BACK] = trace2.fraction;
		if ( trace2.fraction > mostdist )
		{
			mostdist = trace2.fraction;
			whichtrace = 2;
		}
	}

	VectorMA( org, NPC->dead_size/2, right, startpos );
	VectorCopy( startpos, endpos );
	endpos[2] -= 128;
	gi.trace( &trace3, startpos, vec3_origin, vec3_origin, endpos, NPC->s.number, MASK_SOLID );
	if ( !trace3.allsolid && !trace3.startsolid )
	{
		cornerdist[RIGHT] = trace3.fraction;
		if ( trace3.fraction>mostdist )
		{
			mostdist = trace3.fraction;
			whichtrace = 3;
		}
	}

	VectorMA( org, -NPC->dead_size/2, right, startpos );
	VectorCopy( startpos, endpos );
	endpos[2] -= 128;
	gi.trace( &trace4, startpos, vec3_origin, vec3_origin, endpos, NPC->s.number, MASK_SOLID );
	if ( !trace4.allsolid && !trace4.startsolid )
	{
		cornerdist[LEFT] = trace4.fraction;
		if ( trace4.fraction > mostdist )
		{
			mostdist = trace4.fraction;
			whichtrace = 4;
		}
	}

	//OK!  Now if two opposite sides are hanging, use a third if any, else, do nothing
	if ( cornerdist[FRONT] > MIN_DROP_DIST && cornerdist[BACK] > MIN_DROP_DIST )
		frontbackbothclear = true;

	if ( cornerdist[RIGHT] > MIN_DROP_DIST && cornerdist[LEFT] > MIN_DROP_DIST )
		rightleftbothclear = true;

	if ( frontbackbothclear && rightleftbothclear )
		return;

	if ( frontbackbothclear )
	{
		if ( cornerdist[RIGHT] > MIN_DROP_DIST )
			whichtrace = 3;
		else if ( cornerdist[LEFT] > MIN_DROP_DIST )
			whichtrace = 4;
		else
			return;
	}

	if ( rightleftbothclear )
	{
		if ( cornerdist[FRONT] > MIN_DROP_DIST )
			whichtrace = 1;
		else if ( cornerdist[BACK] > MIN_DROP_DIST )
			whichtrace = 2;
		else
			return;
	}

	switch ( whichtrace )
	{//check for stuck
	case 1:
		VectorMA( NPC->currentOrigin, NPC->maxs[0], forward, endpos );
		gi.trace( &movetrace, NPC->currentOrigin, NPC->mins, NPC->maxs, endpos, NPC->s.number, MASK_MONSTERSOLID );
		if ( movetrace.allsolid || movetrace.startsolid || movetrace.fraction < 1.0 )
			if ( canmove( movetrace.ent ) )
				whichtrace = -1;
			else
				whichtrace = 0;
		break;
	case 2:
		VectorMA( NPC->currentOrigin, -NPC->maxs[0], forward, endpos );
		gi.trace( &movetrace, NPC->currentOrigin, NPC->mins, NPC->maxs, endpos, NPC->s.number, MASK_MONSTERSOLID );
		if ( movetrace.allsolid || movetrace.startsolid || movetrace.fraction < 1.0 )
			if ( canmove( movetrace.ent ) )
				whichtrace = -1;
			else
				whichtrace = 0;
		break;
	case 3:
		VectorMA( NPC->currentOrigin, NPC->maxs[0], right, endpos );
		gi.trace( &movetrace, NPC->currentOrigin, NPC->mins, NPC->maxs, endpos, NPC->s.number, MASK_MONSTERSOLID );
		if ( movetrace.allsolid || movetrace.startsolid || movetrace.fraction < 1.0 )
			if ( canmove( movetrace.ent ) )
				whichtrace = -1;
			else
				whichtrace = 0;
		break;
	case 4:
		VectorMA( NPC->currentOrigin, -NPC->maxs[0], right, endpos );
		gi.trace( &movetrace, NPC->currentOrigin, NPC->mins, NPC->maxs, endpos, NPC->s.number, MASK_MONSTERSOLID );
		if (movetrace.allsolid || movetrace.startsolid || movetrace.fraction < 1.0 )
			if ( canmove( movetrace.ent ) )
				whichtrace = -1;
			else
				whichtrace = 0;
		break;
	}

	switch ( whichtrace )
	{
	case 1:
		VectorMA( NPC->client->ps.velocity, 200, forward, NPC->client->ps.velocity );
		if ( trace1.fraction >= 0.9 )
		{
//can't anymore, origin not in center of deathframe!
//			NPC->avelocity[PITCH] = -300;
			NPC->client->ps.friction = 1.0;
		}
		else
		{
			pitch_roll_for_slope( NPC, &trace1.plane.normal );
			NPC->client->ps.friction = trace1.plane.normal[2] * 0.1;
		}
		return;
		break;

	case 2:
		VectorMA( NPC->client->ps.velocity, -200, forward, NPC->client->ps.velocity );
		if(trace2.fraction >= 0.9)
		{
//can't anymore, origin not in center of deathframe!
//			NPC->avelocity[PITCH] = 300;
			NPC->client->ps.friction = 1.0;
		}
		else
		{
			pitch_roll_for_slope( NPC, &trace2.plane.normal );
			NPC->client->ps.friction = trace2.plane.normal[2] * 0.1;
		}
		return;
		break;

	case 3:
		VectorMA( NPC->client->ps.velocity, 200, right, NPC->client->ps.velocity );
		if ( trace3.fraction >= 0.9 )
		{
//can't anymore, origin not in center of deathframe!
//			NPC->avelocity[ROLL] = -300;
			NPC->client->ps.friction = 1.0;
		}
		else
		{
			pitch_roll_for_slope( NPC, &trace3.plane.normal );
			NPC->client->ps.friction = trace3.plane.normal[2] * 0.1;
		}
		return;
		break;

	case 4:
		VectorMA( NPC->client->ps.velocity, -200, right, NPC->client->ps.velocity );
		if ( trace4.fraction >= 0.9 )
		{
//can't anymore, origin not in center of deathframe!
//			NPC->avelocity[ROLL] = 300;
			NPC->client->ps.friction = 1.0;
		}
		else
		{
			pitch_roll_for_slope( NPC, &trace4.plane.normal );
			NPC->client->ps.friction = trace4.plane.normal[2] * 0.1;
		}
		return;
		break;
	}

	//on solid ground
	if ( whichtrace == -1 )
	{
		return;
	}
	NPC->client->ps.friction = 1.0;

	//VectorClear( NPC->avelocity );
	pitch_roll_for_slope( NPC );

	//gi.linkentity (NPC);
}
*/

/*
----------------------------------------
DeadThink
----------------------------------------
*/
unsafe fn DeadThink() {
    let mut trace: trace_t = core::mem::zeroed();
    //HACKHACKHACKHACKHACK
    //We should really have a seperate G2 bounding box (seperate from the physics bbox) for G2 collisions only
    //FIXME: don't ever inflate back up?
    //GAH!  With Ragdoll, they get stuck in the ceiling
    let oldMaxs2: f32 = (*NPC).maxs[2];
    (*NPC).maxs[2] = (*(*NPC).client).renderInfo.eyePoint[2] - (*NPC).currentOrigin[2] + 4.0;
    if (*NPC).maxs[2] < -8.0 {
        (*NPC).maxs[2] = -8.0;
    }
    if (*NPC).maxs[2] > oldMaxs2 {
        //inflating maxs, make sure we're not inflating into solid
        gi.trace(
            &mut trace,
            (*NPC).currentOrigin.as_ptr(),
            (*NPC).mins.as_ptr(),
            (*NPC).maxs.as_ptr(),
            (*NPC).currentOrigin.as_ptr(),
            (*NPC).s.number,
            (*NPC).clipmask,
        );
        if trace.allsolid != qfalse {
            //must be inflating
            (*NPC).maxs[2] = oldMaxs2;
        }
    }
    /*
    {
        if ( VectorCompare( NPC->client->ps.velocity, vec3_origin ) )
        {//not flying through the air
            if ( NPC->mins[0] > -32 )
            {
                NPC->mins[0] -= 1;
                gi.trace (&trace, NPC->currentOrigin, NPC->mins, NPC->maxs, NPC->currentOrigin, NPC->s.number, NPC->clipmask );
                if ( trace.allsolid )
                {
                    NPC->mins[0] += 1;
                }
            }
            if ( NPC->maxs[0] < 32 )
            {
                NPC->maxs[0] += 1;
                gi.trace (&trace, NPC->currentOrigin, NPC->mins, NPC->maxs, NPC->currentOrigin, NPC->s.number, NPC->clipmask );
                if ( trace.allsolid )
                {
                    NPC->maxs[0] -= 1;
                }
            }
            if ( NPC->mins[1] > -32 )
            {
                NPC->mins[1] -= 1;
                gi.trace (&trace, NPC->currentOrigin, NPC->mins, NPC->maxs, NPC->currentOrigin, NPC->s.number, NPC->clipmask );
                if ( trace.allsolid )
                {
                    NPC->mins[1] += 1;
                }
            }
            if ( NPC->maxs[1] < 32 )
            {
                NPC->maxs[1] += 1;
                gi.trace (&trace, NPC->currentOrigin, NPC->mins, NPC->maxs, NPC->currentOrigin, NPC->s.number, NPC->clipmask );
                if ( trace.allsolid )
                {
                    NPC->maxs[1] -= 1;
                }
            }
        }
    }
    //HACKHACKHACKHACKHACK
    */


    //FIXME: tilt and fall off of ledges?
    //NPC_PostDeathThink();

    /*
    if ( !NPCInfo->timeOfDeath && NPC->client != NULL && NPCInfo != NULL )
    {
        //haven't finished death anim yet and were NOT given a specific amount of time to wait before removal
        int				legsAnim	= NPC->client->ps.legsAnim;
        animation_t		*animations	= knownAnimFileSets[NPC->client->clientInfo.animFileIndex].animations;

        NPC->bounceCount = -1; // This is a cheap hack for optimizing the pointcontents check below

        //ghoul doesn't tell us this anymore
        //if ( NPC->client->renderInfo.legsFrame == animations[legsAnim].firstFrame + (animations[legsAnim].numFrames - 1) )
        {
            //reached the end of the death anim
            NPCInfo->timeOfDeath = level.time + BodyRemovalPadTime( NPC );
        }
    }
    else
    */
    {
        //death anim done (or were given a specific amount of time to wait before removal), wait the requisite amount of time them remove
        if level.time >= (*NPCInfo).timeOfDeath + BodyRemovalPadTime(NPC) {
            if ((*(*NPC).client).ps.eFlags & EF_NODRAW) != 0 {
                if !IIcarusInterface::GetIcarus().IsRunning((*NPC).m_iIcarusID) {
                    (*NPC).e_ThinkFunc = thinkF_G_FreeEntity;
                    (*NPC).nextthink = level.time + FRAMETIME;
                }
            } else {
                // Start the body effect first, then delay 400ms before ditching the corpse
                NPC_RemoveBodyEffect();

                //FIXME: keep it running through physics somehow?
                (*NPC).e_ThinkFunc = thinkF_NPC_RemoveBody;
                (*NPC).nextthink = level.time + FRAMETIME / 2;
            //	if ( NPC->client->playerTeam == TEAM_FORGE )
            //		NPCInfo->timeOfDeath = level.time + FRAMETIME * 8;
            //	else if ( NPC->client->playerTeam == TEAM_BOTS )
                let npc_class: class_t = (*(*NPC).client).NPC_class;
                // check for droids
                if npc_class == CLASS_SEEKER
                    || npc_class == CLASS_REMOTE
                    || npc_class == CLASS_PROBE
                    || npc_class == CLASS_MOUSE
                    || npc_class == CLASS_GONK
                    || npc_class == CLASS_R2D2
                    || npc_class == CLASS_R5D2
                    || npc_class == CLASS_MARK2
                    || npc_class == CLASS_SENTRY
                //npc_class == CLASS_PROTOCOL ||
                {
                    (*(*NPC).client).ps.eFlags |= EF_NODRAW;
                    (*NPCInfo).timeOfDeath = level.time + FRAMETIME * 8;
                } else {
                    (*NPCInfo).timeOfDeath = level.time + FRAMETIME * 4;
                }
            }
            return;
        }
    }

    // If the player is on the ground and the resting position contents haven't been set yet...(BounceCount tracks the contents)
    if (*NPC).bounceCount < 0 && (*NPC).s.groundEntityNum >= 0 {
        // if client is in a nodrop area, make him/her nodraw
        let contents: c_int = gi.pointcontents((*NPC).currentOrigin.as_ptr(), -1);
        (*NPC).bounceCount = contents;

        if (contents & CONTENTS_NODROP) != 0 {
            (*(*NPC).client).ps.eFlags |= EF_NODRAW;
        }
    }

    CorpsePhysics(NPC);
}

/*
===============
SetNPCGlobals

local function to set globals used throughout the AI code
===============
*/
pub unsafe fn SetNPCGlobals(ent: *mut gentity_t) {
    NPC = ent;
    NPCInfo = (*ent).NPC;
    client = (*ent).client;
    addr_of_mut!(ucmd).write(core::mem::zeroed());
}

pub unsafe fn SaveNPCGlobals() {
    _saved_NPC = NPC;
    _saved_NPCInfo = NPCInfo;
    _saved_client = client;
    _saved_ucmd = ucmd;
}

pub unsafe fn RestoreNPCGlobals() {
    NPC = _saved_NPC;
    NPCInfo = _saved_NPCInfo;
    client = _saved_client;
    ucmd = _saved_ucmd;
}

//We MUST do this, other funcs were using NPC illegally when "self" wasn't the global NPC
pub unsafe fn ClearNPCGlobals() {
    NPC = null_mut();
    NPCInfo = null_mut();
    client = null_mut();
}
//===============

pub unsafe fn NPC_ShowDebugInfo() {
    if showBBoxes != qfalse {
        let mut found: *mut gentity_t = null_mut();
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];

        //do player, too
        VectorAdd((*player).currentOrigin.as_ptr(), (*player).mins.as_ptr(), mins.as_mut_ptr());
        VectorAdd((*player).currentOrigin.as_ptr(), (*player).maxs.as_ptr(), maxs.as_mut_ptr());
        CG_Cube(mins.as_mut_ptr(), maxs.as_mut_ptr(), NPCDEBUG_RED.as_mut_ptr(), 0.25);
        //do NPCs
        loop {
            found = G_Find(found, FOFS!(classname), b"NPC\0".as_ptr() as *const c_char);
            if found.is_null() { break; }
            if gi.inPVS((*found).currentOrigin.as_ptr(), (*g_entities.as_ptr()).currentOrigin.as_ptr()) != qfalse {
                VectorAdd((*found).currentOrigin.as_ptr(), (*found).mins.as_ptr(), mins.as_mut_ptr());
                VectorAdd((*found).currentOrigin.as_ptr(), (*found).maxs.as_ptr(), maxs.as_mut_ptr());
                CG_Cube(mins.as_mut_ptr(), maxs.as_mut_ptr(), NPCDEBUG_RED.as_mut_ptr(), 0.25);
            }
        }
    }
}

pub unsafe fn NPC_ApplyScriptFlags() {
    if ((*NPCInfo).scriptFlags & SCF_CROUCHED) != 0 {
        if (*NPCInfo).charmedTime > level.time && (ucmd.forwardmove != 0 || ucmd.rightmove != 0) {
            //ugh, if charmed and moving, ignore the crouched command
        } else {
            ucmd.upmove = -127;
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_RUNNING) != 0 {
        ucmd.buttons &= !BUTTON_WALKING;
    } else if ((*NPCInfo).scriptFlags & SCF_WALKING) != 0 {
        if (*NPCInfo).charmedTime > level.time && (ucmd.forwardmove != 0 || ucmd.rightmove != 0) {
            //ugh, if charmed and moving, ignore the walking command
        } else {
            ucmd.buttons |= BUTTON_WALKING;
        }
    }
/*
	if(NPCInfo->scriptFlags & SCF_CAREFUL)
	{
		ucmd.buttons |= BUTTON_CAREFUL;
	}
*/
    if ((*NPCInfo).scriptFlags & SCF_LEAN_RIGHT) != 0 {
        ucmd.buttons |= BUTTON_USE;
        ucmd.rightmove = 127;
        ucmd.forwardmove = 0;
        ucmd.upmove = 0;
    } else if ((*NPCInfo).scriptFlags & SCF_LEAN_LEFT) != 0 {
        ucmd.buttons |= BUTTON_USE;
        ucmd.rightmove = -127;
        ucmd.forwardmove = 0;
        ucmd.upmove = 0;
    }

    if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 && (ucmd.buttons & BUTTON_ATTACK) != 0 {
        //Use altfire instead
        ucmd.buttons |= BUTTON_ALT_ATTACK;
    }

    // only removes NPC when it's safe too (Player is out of PVS)
    if ((*NPCInfo).scriptFlags & SCF_SAFE_REMOVE) != 0 {
        // take from BSRemove
        if gi.inPVS((*NPC).currentOrigin.as_ptr(), (*g_entities.as_ptr()).currentOrigin.as_ptr()) == qfalse {
            //FIXME: use cg.vieworg?
            G_UseTargets2(NPC, NPC, (*NPC).target3);
            (*NPC).s.eFlags |= EF_NODRAW;
            (*NPC).svFlags &= !SVF_NPC;
            (*NPC).s.eType = ET_INVISIBLE;
            (*NPC).contents = 0;
            (*NPC).health = 0;
            (*NPC).targetname = null_mut();

            //Disappear in half a second
            (*NPC).e_ThinkFunc = thinkF_G_FreeEntity;
            (*NPC).nextthink = level.time + FRAMETIME;
        }//FIXME: else allow for out of FOV???

    }
}

pub unsafe fn NPC_HandleAIFlags() {
    // Update Guys With Jet Packs
    //----------------------------
    if ((*NPCInfo).scriptFlags & SCF_FLY_WITH_JET) != 0 {
        let mut ShouldFly: bool = ((*NPCInfo).aiFlags & NPCAI_FLY) != 0;
        let IsFlying: bool = JET_Flying(NPC) != qfalse;
        let mut IsInTheAir: bool = (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE;

        if IsFlying {
            // Don't Stop Flying Until Near The Ground
            //-----------------------------------------
            if IsInTheAir {
                let mut ground: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                VectorCopy((*NPC).currentOrigin.as_ptr(), ground.as_mut_ptr());
                ground[2] -= 60.0;
                gi.trace(
                    &mut trace,
                    (*NPC).currentOrigin.as_ptr(),
                    null(),
                    null(),
                    ground.as_ptr(),
                    (*NPC).s.number,
                    (*NPC).clipmask,
                );

                IsInTheAir = trace.allsolid == qfalse && trace.startsolid == qfalse && trace.fraction > 0.9;
            }

            // If Flying, Remember The Last Time
            //-----------------------------------
            if IsInTheAir {
                (*NPC).lastInAirTime = level.time;
                ShouldFly = true;
            }

            // Auto Turn Off Jet Pack After 1 Second On The Ground
            //-----------------------------------------------------
            else if !ShouldFly && (level.time - (*NPC).lastInAirTime) > 3000 {
                (*NPCInfo).aiFlags &= !NPCAI_FLY;
            }
        }


        // If We Should Be Flying And Are Not, Start Er Up
        //-------------------------------------------------
        if ShouldFly && !IsFlying {
            JET_FlyStart(NPC);              // EVENTUALLY, Remove All Other Calls
        }

        // Otherwise, If Needed, Shut It Off
        //-----------------------------------
        else if !ShouldFly && IsFlying {
            JET_FlyStop(NPC);               // EVENTUALLY, Remove All Other Calls
        }
    }

    //FIXME: make these flags checks a function call like NPC_CheckAIFlagsAndTimers
    if ((*NPCInfo).aiFlags & NPCAI_LOST) != 0 {
        //Print that you need help!
        //FIXME: shouldn't remove this just yet if cg_draw needs it
        (*NPCInfo).aiFlags &= !NPCAI_LOST;

        /*
        if ( showWaypoints )
        {
            Quake3Game()->DebugPrint(WL_WARNING, "%s can't navigate to target %s (my wp: %d, goal wp: %d)\n", NPC->targetname, NPCInfo->goalEntity->targetname, NPC->waypoint, NPCInfo->goalEntity->waypoint );
        }
        */

        if !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalEntity == (*NPC).enemy {
            //We can't nav to our enemy
            //Drop enemy and see if we should search for him
            NPC_LostEnemyDecideChase();
        }
    }

    //been told to play a victory sound after a delay
    if (*NPCInfo).greetingDebounceTime != 0 && (*NPCInfo).greetingDebounceTime < level.time {
        G_AddVoiceEvent(NPC, Q_irand(EV_VICTORY1, EV_VICTORY3), Q_irand(2000, 4000));
        (*NPCInfo).greetingDebounceTime = 0;
    }

    if (*NPCInfo).ffireCount > 0 {
        if (*NPCInfo).ffireFadeDebounce < level.time {
            (*NPCInfo).ffireCount -= 1;
            //Com_Printf( "drop: %d < %d\n", NPCInfo->ffireCount, 3+((2-g_spskill->integer)*2) );
            (*NPCInfo).ffireFadeDebounce = level.time + 3000;
        }
    }
}

pub unsafe fn NPC_AvoidWallsAndCliffs() {
/*
	vec3_t	forward, right, testPos, angles, mins;
	trace_t	trace;
	float	fwdDist, rtDist;
	//FIXME: set things like this forward dir once at the beginning
	//of a frame instead of over and over again
	if ( NPCInfo->aiFlags & NPCAI_NO_COLL_AVOID )
	{
		return;
	}

	if ( ucmd.upmove > 0 || NPC->client->ps.groundEntityNum == ENTITYNUM_NONE )
	{//Going to jump or in the air
		return;
	}

	if ( !ucmd.forwardmove && !ucmd.rightmove )
	{
		return;
	}

	if ( fabs( AngleDelta( NPC->currentAngles[YAW], NPCInfo->desiredYaw ) ) < 5.0 )//!ucmd.angles[YAW] )
	{//Not turning much, don't do this
		//NOTE: Should this not happen only if you're not turning AT ALL?
		//	You could be turning slowly but moving fast, so that would
		//	still let you walk right off a cliff...
		//NOTE: Or maybe it is a good idea to ALWAYS do this, regardless
		//	of whether ot not we're turning?  But why would we be walking
		//  straight into a wall or off	a cliff unless we really wanted to?
		return;
	}

	VectorCopy( NPC->mins, mins );
	mins[2] += STEPSIZE;
	angles[YAW] = NPC->client->ps.viewangles[YAW];//Add ucmd.angles[YAW]?
	AngleVectors( angles, forward, right, NULL );
	fwdDist = ((float)ucmd.forwardmove)/16.0f;
	rtDist = ((float)ucmd.rightmove)/16.0f;
	VectorMA( NPC->currentOrigin, fwdDist, forward, testPos );
	VectorMA( testPos, rtDist, right, testPos );
	gi.trace( &trace, NPC->currentOrigin, mins, NPC->maxs, testPos, NPC->s.number, NPC->clipmask );
	if ( trace.allsolid || trace.startsolid || trace.fraction < 1.0 )
	{//Going to bump into something, don't move, just turn
		ucmd.forwardmove = 0;
		ucmd.rightmove = 0;
		return;
	}

	VectorCopy(trace.endpos, testPos);
	testPos[2] -= 128;

	gi.trace( &trace, trace.endpos, mins, NPC->maxs, testPos, NPC->s.number, NPC->clipmask );
	if ( trace.allsolid || trace.startsolid || trace.fraction < 1.0 )
	{//Not going off a cliff
		return;
	}

	//going to fall at least 128, don't move, just turn... is this bad, though?  What if we want them to drop off?
	ucmd.forwardmove = 0;
	ucmd.rightmove = 0;
	return;
*/
}

pub unsafe fn NPC_CheckAttackScript() {
    if (ucmd.buttons & BUTTON_ATTACK) == 0 {
        return;
    }

    G_ActivateBehavior(NPC, BSET_ATTACK);
}

pub unsafe fn NPC_CheckAttackHold() {
    let mut vec: vec3_t = [0.0; 3];

    // If they don't have an enemy they shouldn't hold their attack anim.
    if (*NPC).enemy.is_null() {
        (*NPCInfo).attackHoldTime = 0;
        return;
    }

    //FIXME: need to tie this into AI somehow?
    VectorSubtract((*(*NPC).enemy).currentOrigin.as_ptr(), (*NPC).currentOrigin.as_ptr(), vec.as_mut_ptr());
    if VectorLengthSquared(vec.as_ptr()) > NPC_MaxDistSquaredForWeapon() {
        (*NPCInfo).attackHoldTime = 0;
    } else if (*NPCInfo).attackHoldTime != 0 && (*NPCInfo).attackHoldTime > level.time {
        ucmd.buttons |= BUTTON_ATTACK;
    } else if (*NPCInfo).attackHold != 0 && (ucmd.buttons & BUTTON_ATTACK) != 0 {
        (*NPCInfo).attackHoldTime = level.time + (*NPCInfo).attackHold;
    } else {
        (*NPCInfo).attackHoldTime = 0;
    }
}

/*
void NPC_KeepCurrentFacing(void)

Fills in a default ucmd to keep current angles facing
*/
pub unsafe fn NPC_KeepCurrentFacing() {
    if ucmd.angles[YAW as usize] == 0 {
        ucmd.angles[YAW as usize] = ANGLE2SHORT((*client).ps.viewangles[YAW as usize]) - (*client).ps.delta_angles[YAW as usize];
    }

    if ucmd.angles[PITCH as usize] == 0 {
        ucmd.angles[PITCH as usize] = ANGLE2SHORT((*client).ps.viewangles[PITCH as usize]) - (*client).ps.delta_angles[PITCH as usize];
    }
}

/*
-------------------------
NPC_BehaviorSet_Charmed
-------------------------
*/

pub unsafe fn NPC_BehaviorSet_Charmed(bState: c_int) {
    if bState == BS_FOLLOW_LEADER as c_int {
        //# 40: Follow your leader and shoot any enemies you come across
        NPC_BSFollowLeader();
    } else if bState == BS_REMOVE as c_int {
        NPC_BSRemove();
    } else if bState == BS_SEARCH as c_int {
        //# 43: Using current waypoint as a base, search the immediate branches of waypoints for enemies
        NPC_BSSearch();
    } else if bState == BS_WANDER as c_int {
        //# 46: Wander down random waypoint paths
        NPC_BSWander();
    } else if bState == BS_FLEE as c_int {
        NPC_BSFlee();
    } else {
        //default:
        //whatever
        NPC_BSDefault();
    }
}
/*
-------------------------
NPC_BehaviorSet_Default
-------------------------
*/

pub unsafe fn NPC_BehaviorSet_Default(bState: c_int) {
    if bState == BS_ADVANCE_FIGHT as c_int {
        //head toward captureGoal, shoot anything that gets in the way
        NPC_BSAdvanceFight();
    } else if bState == BS_SLEEP as c_int {
        //Follow a path, looking for enemies
        NPC_BSSleep();
    } else if bState == BS_FOLLOW_LEADER as c_int {
        //# 40: Follow your leader and shoot any enemies you come across
        NPC_BSFollowLeader();
    } else if bState == BS_JUMP as c_int {
        //41: Face navgoal and jump to it.
        NPC_BSJump();
    } else if bState == BS_REMOVE as c_int {
        NPC_BSRemove();
    } else if bState == BS_SEARCH as c_int {
        //# 43: Using current waypoint as a base, search the immediate branches of waypoints for enemies
        NPC_BSSearch();
    } else if bState == BS_NOCLIP as c_int {
        NPC_BSNoClip();
    } else if bState == BS_WANDER as c_int {
        //# 46: Wander down random waypoint paths
        NPC_BSWander();
    } else if bState == BS_FLEE as c_int {
        NPC_BSFlee();
    } else if bState == BS_WAIT as c_int {
        NPC_BSWait();
    } else if bState == BS_CINEMATIC as c_int {
        NPC_BSCinematic();
    } else {
        //default:
        //whatever
        NPC_BSDefault();
    }
}

/*
-------------------------
NPC_BehaviorSet_Interrogator
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Interrogator(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSInterrogator_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_ImperialProbe
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_ImperialProbe(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSImperialProbe_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}


/*
-------------------------
NPC_BehaviorSet_Seeker
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Seeker(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSSeeker_Default();
    } else {
        NPC_BSSeeker_Default();
    }
}

/*
-------------------------
NPC_BehaviorSet_Remote
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Remote(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSRemote_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Sentry
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Sentry(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSSentry_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Grenadier
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Grenadier(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSGrenadier_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Tusken
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Tusken(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSTusken_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Sniper
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Sniper(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSSniper_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}
/*
-------------------------
NPC_BehaviorSet_Stormtrooper
-------------------------
*/

pub unsafe fn NPC_BehaviorSet_Stormtrooper(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSST_Default();
    } else if bState == BS_INVESTIGATE as c_int {
        NPC_BSST_Investigate();
    } else if bState == BS_SLEEP as c_int {
        NPC_BSST_Sleep();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Jedi
-------------------------
*/

pub unsafe fn NPC_BehaviorSet_Jedi(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_INVESTIGATE as c_int //WTF???!!
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSJedi_Default();
    } else if bState == BS_FOLLOW_LEADER as c_int {
        NPC_BSJedi_FollowLeader();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

pub unsafe fn G_JediInNormalAI(ent: *mut gentity_t) -> qboolean {
    //NOTE: should match above func's switch!
    //check our bState
    let bState: bState_t = G_CurrentBState((*ent).NPC);
    if bState == BS_STAND_GUARD
        || bState == BS_PATROL
        || bState == BS_STAND_AND_SHOOT
        || bState == BS_HUNT_AND_KILL
        || bState == BS_INVESTIGATE
        || bState == BS_DEFAULT
        || bState == BS_FOLLOW_LEADER
    {
        return qtrue;
    }
    qfalse
}

/*
-------------------------
NPC_BehaviorSet_Droid
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Droid(bState: c_int) {
    if bState == BS_DEFAULT as c_int
        || bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
    {
        NPC_BSDroid_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Mark1
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Mark1(bState: c_int) {
    if bState == BS_DEFAULT as c_int
        || bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
    {
        NPC_BSMark1_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Mark2
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Mark2(bState: c_int) {
    if bState == BS_DEFAULT as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
    {
        NPC_BSMark2_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_ATST
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_ATST(bState: c_int) {
    if bState == BS_DEFAULT as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
    {
        NPC_BSATST_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_MineMonster
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_MineMonster(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSMineMonster_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Howler
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Howler(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSHowler_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Rancor
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Rancor(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSRancor_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Wampa
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_Wampa(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSWampa_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_SandCreature
-------------------------
*/
pub unsafe fn NPC_BehaviorSet_SandCreature(bState: c_int) {
    if bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
        || bState == BS_STAND_AND_SHOOT as c_int
        || bState == BS_HUNT_AND_KILL as c_int
        || bState == BS_DEFAULT as c_int
    {
        NPC_BSSandCreature_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_BehaviorSet_Droid
-------------------------
*/
// Added 01/21/03 by AReis.
pub unsafe fn NPC_BehaviorSet_Animal(bState: c_int) {
    if bState == BS_DEFAULT as c_int
        || bState == BS_STAND_GUARD as c_int
        || bState == BS_PATROL as c_int
    {
        NPC_BSAnimal_Default();

        //NPC_BSDroid_Default();
    } else {
        NPC_BehaviorSet_Default(bState);
    }
}

/*
-------------------------
NPC_RunBehavior
-------------------------
*/
pub unsafe fn NPC_RunBehavior(team: c_int, bState: c_int) {
    let mut dontSetAim: qboolean = qfalse;

    //
    if bState == BS_CINEMATIC as c_int {
        NPC_BSCinematic();
    } else if ((*NPCInfo).scriptFlags & SCF_PILOT) != 0 && Pilot_MasterUpdate() {
        return;
    } else if NPC_JumpBackingUp() != qfalse {
        return;
    } else if TIMER_Done(NPC, b"DEMP2_StunTime\0".as_ptr() as *const c_char) == qfalse {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    } else if (*(*NPC).client).ps.weapon == WP_EMPLACED_GUN {
        NPC_BSEmplaced();
        G_CheckCharmed(NPC);
        return;
    } else if (*(*NPC).client).NPC_class == CLASS_HOWLER {
        NPC_BehaviorSet_Howler(bState);
        return;
    } else if Jedi_CultistDestroyer(NPC) != qfalse {
        NPC_BSJedi_Default();
        dontSetAim = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_SABER_DROID {
        //saber droid
        NPC_BSSD_Default();
    } else if (*(*NPC).client).ps.weapon == WP_SABER {
        //jedi
        NPC_BehaviorSet_Jedi(bState);
        dontSetAim = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_REBORN && (*(*NPC).client).ps.weapon == WP_MELEE {
        //force-only reborn
        NPC_BehaviorSet_Jedi(bState);
        dontSetAim = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        Boba_Update();
        if (*NPCInfo).surrenderTime != 0 {
            Boba_Flee();
        } else {
            if !Boba_Tactics() {
                if Boba_Flying(NPC) != qfalse {
                    NPC_BehaviorSet_Seeker(bState);
                } else {
                    NPC_BehaviorSet_Jedi(bState);
                }
            }
        }
        dontSetAim = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
        //bounty hunter
        if RT_Flying(NPC) != qfalse || !(*NPC).enemy.is_null() {
            NPC_BSRT_Default();
        } else {
            NPC_BehaviorSet_Stormtrooper(bState);
        }
        G_CheckCharmed(NPC);
        dontSetAim = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_RANCOR {
        NPC_BehaviorSet_Rancor(bState);
    } else if (*(*NPC).client).NPC_class == CLASS_SAND_CREATURE {
        NPC_BehaviorSet_SandCreature(bState);
    } else if (*(*NPC).client).NPC_class == CLASS_WAMPA {
        NPC_BehaviorSet_Wampa(bState);
        G_CheckCharmed(NPC);
    } else if ((*NPCInfo).scriptFlags & SCF_FORCED_MARCH) != 0 {
        //being forced to march
        NPC_BSDefault();
    } else if (*(*NPC).client).ps.weapon == WP_TUSKEN_RIFLE {
        if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
            NPC_BehaviorSet_Sniper(bState);
            G_CheckCharmed(NPC);
            return;
        } else {
            NPC_BehaviorSet_Tusken(bState);
            G_CheckCharmed(NPC);
            return;
        }
    } else if (*(*NPC).client).ps.weapon == WP_TUSKEN_STAFF {
        NPC_BehaviorSet_Tusken(bState);
        G_CheckCharmed(NPC);
        return;
    } else if (*(*NPC).client).ps.weapon == WP_NOGHRI_STICK {
        NPC_BehaviorSet_Stormtrooper(bState);
        G_CheckCharmed(NPC);
    } else {
        if team == TEAM_ENEMY as c_int {
        //	case TEAM_SCAVENGERS:
        //	case TEAM_IMPERIAL:
        //	case TEAM_KLINGON:
        //	case TEAM_HIROGEN:
        //	case TEAM_MALON:
            // not sure if TEAM_ENEMY is appropriate here, I think I should be using NPC_class to check for behavior - dmv
            // special cases for enemy droids
            if (*(*NPC).client).NPC_class == CLASS_ATST {
                NPC_BehaviorSet_ATST(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_PROBE {
                NPC_BehaviorSet_ImperialProbe(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_REMOTE {
                NPC_BehaviorSet_Remote(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_SENTRY {
                NPC_BehaviorSet_Sentry(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_INTERROGATOR {
                NPC_BehaviorSet_Interrogator(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_MINEMONSTER {
                NPC_BehaviorSet_MineMonster(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_HOWLER {
                NPC_BehaviorSet_Howler(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_RANCOR {
                NPC_BehaviorSet_Rancor(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_SAND_CREATURE {
                NPC_BehaviorSet_SandCreature(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_MARK1 {
                NPC_BehaviorSet_Mark1(bState);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_MARK2 {
                NPC_BehaviorSet_Mark2(bState);
                return;
            }


            if (*(*NPC).client).NPC_class == CLASS_ASSASSIN_DROID {
                BubbleShield_Update();
            }

            if NPC_IsTrooper(NPC) {
                NPC_BehaviorSet_Trooper(bState);
                return;
            }

            if !(*NPC).enemy.is_null()
                && (*(*NPC).client).ps.weapon == WP_NONE
                && bState != BS_HUNT_AND_KILL as c_int
                && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == qfalse
            {
                //if in battle and have no weapon, run away, fixme: when in BS_HUNT_AND_KILL, they just stand there
                if bState != BS_FLEE as c_int {
                    NPC_StartFlee((*NPC).enemy, (*(*NPC).enemy).currentOrigin.as_ptr(), AEL_DANGER_GREAT, 5000, 10000);
                } else {
                    NPC_BSFlee();
                }
                return;
            }
            if (*(*NPC).client).ps.weapon == WP_SABER {
                //special melee exception
                NPC_BehaviorSet_Default(bState);
                return;
            }
            if (*(*NPC).client).ps.weapon == WP_DISRUPTOR && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                //a sniper
                NPC_BehaviorSet_Sniper(bState);
                return;
            }
            if (*(*NPC).client).ps.weapon == WP_THERMAL
                || (*(*NPC).client).ps.weapon == WP_MELEE //FIXME: separate AI for melee fighters
            {
                //a grenadier
                NPC_BehaviorSet_Grenadier(bState);
                return;
            }
            if NPC_CheckSurrender() != qfalse {
                return;
            }
            NPC_BehaviorSet_Stormtrooper(bState);
        } else if team == TEAM_NEUTRAL as c_int {
            // special cases for enemy droids
            if (*(*NPC).client).NPC_class == CLASS_PROTOCOL {
                NPC_BehaviorSet_Default(bState);
            } else if (*(*NPC).client).NPC_class == CLASS_UGNAUGHT
                || (*(*NPC).client).NPC_class == CLASS_JAWA
            {
                //others, too?
                NPC_BSCivilian_Default(bState);
                return;
            }
            // Add special vehicle behavior here.
            else if (*(*NPC).client).NPC_class == CLASS_VEHICLE {
                let pVehicle: *mut Vehicle_t = (*NPC).m_pVehicle;
                if (*pVehicle).m_pPilot.is_null() && (*pVehicle).m_iBoarding == 0 {
                    if (*(*pVehicle).m_pVehicleInfo).type_ == VH_ANIMAL {
                        NPC_BehaviorSet_Animal(bState);
                    }

                    // TODO: The only other case were we want a vehicle to do something specifically is
                    // perhaps in multiplayer where we want the shuttle to be able to lift off when not
                    // occupied and in a landing zone.
                }
            } else {
                // Just one of the average droids
                NPC_BehaviorSet_Droid(bState);
            }
        } else {
            //default:
            if (*(*NPC).client).NPC_class == CLASS_SEEKER {
                NPC_BehaviorSet_Seeker(bState);
            } else {
                if (*NPCInfo).charmedTime > level.time {
                    NPC_BehaviorSet_Charmed(bState);
                } else {
                    NPC_BehaviorSet_Default(bState);
                }
                G_CheckCharmed(NPC);
                dontSetAim = qtrue;
            }
        }
    }
}

unsafe fn G_CurrentBState(gNPC: *mut gNPC_t) -> bState_t {
    if (*gNPC).tempBehavior != BS_DEFAULT {
        //Overrides normal behavior until cleared
        return (*gNPC).tempBehavior;
    }

    if (*gNPC).behaviorState == BS_DEFAULT {
        (*gNPC).behaviorState = (*gNPC).defaultBehavior;
    }

    (*gNPC).behaviorState
}

/*
===============
NPC_ExecuteBState

  MCG

NPC Behavior state thinking

===============
*/
pub unsafe fn NPC_ExecuteBState(self_: *mut gentity_t) {
    let mut bState: bState_t;

    NPC_HandleAIFlags();

    //FIXME: these next three bits could be a function call, some sort of setup/cleanup func
    //Lookmode must be reset every think cycle
    if (*NPC).delayScriptTime != 0 && (*NPC).delayScriptTime <= level.time {
        G_ActivateBehavior(NPC, BSET_DELAYED);
        (*NPC).delayScriptTime = 0;
    }

    //Clear this and let bState set it itself, so it automatically handles changing bStates... but we need a set bState wrapper func
    (*NPCInfo).combatMove = qfalse;

    //Execute our bState
    bState = G_CurrentBState(NPCInfo);

    //Pick the proper bstate for us and run it
    NPC_RunBehavior((*(*self_).client).playerTeam, bState as c_int);


//	if(bState != BS_POINT_COMBAT && NPCInfo->combatPoint != -1)
//	{
        //level.combatPoints[NPCInfo->combatPoint].occupied = qfalse;
        //NPCInfo->combatPoint = -1;
//	}

    //Here we need to see what the scripted stuff told us to do
//Only process snapshot if independant and in combat mode- this would pick enemies and go after needed items
//	ProcessSnapshot();

//Ignore my needs if I'm under script control- this would set needs for items
//	CheckSelf();

    //Back to normal?  All decisions made?

    //FIXME: don't walk off ledges unless we can get to our goal faster that way, or that's our goal's surface
    //NPCPredict();

    if !(*NPC).enemy.is_null() {
        if (*(*NPC).enemy).inuse == qfalse {
            //just in case bState doesn't catch this
            G_ClearEnemy(NPC);
        }
    }

    if (*(*NPC).client).ps.saberLockTime != 0 && (*(*NPC).client).ps.saberLockEnemy != ENTITYNUM_NONE {
        NPC_SetLookTarget(NPC, (*(*NPC).client).ps.saberLockEnemy, level.time + 1000);
    } else if NPC_CheckLookTarget(NPC) == qfalse {
        if !(*NPC).enemy.is_null() {
            NPC_SetLookTarget(NPC, (*(*NPC).enemy).s.number, 0);
        }
    }

    if !(*NPC).enemy.is_null() {
        if ((*(*NPC).enemy).flags & FL_DONT_SHOOT) != 0 {
            ucmd.buttons &= !BUTTON_ATTACK;
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
        } else if (*(*NPC).client).playerTeam != TEAM_ENEMY //not an enemy
            && ((*(*NPC).client).playerTeam != TEAM_FREE
                || ((*(*NPC).client).NPC_class == CLASS_TUSKEN && Q_irand(0, 4) != 0)) //not a rampaging creature or I'm a tusken and I feel generous (temporarily)
            && !(*(*NPC).enemy).NPC.is_null()
            && ((*(*(*NPC).enemy).NPC).surrenderTime > level.time
                || ((*(*(*NPC).enemy).NPC).scriptFlags & SCF_FORCED_MARCH) != 0)
        {
            //don't shoot someone who's surrendering if you're a good guy
            ucmd.buttons &= !BUTTON_ATTACK;
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
        }

        if (*client).ps.weaponstate == WEAPON_IDLE {
            (*client).ps.weaponstate = WEAPON_READY;
        }
    } else {
        if (*client).ps.weaponstate == WEAPON_READY {
            (*client).ps.weaponstate = WEAPON_IDLE;
        }
    }

    if (ucmd.buttons & BUTTON_ATTACK) == 0 && (*NPC).attackDebounceTime > level.time {
        //We just shot but aren't still shooting, so hold the gun up for a while
        if (*client).ps.weapon == WP_SABER {
            //One-handed
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_WEAPONREADY1, SETANIM_FLAG_NORMAL, 0);
        } else if (*client).ps.weapon == WP_BRYAR_PISTOL {
            //Sniper pose
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_WEAPONREADY3, SETANIM_FLAG_NORMAL, 0);
        }
        /*//FIXME: What's the proper solution here?
        else
        {//heavy weapon
            NPC_SetAnim(NPC,SETANIM_TORSO,TORSO_WEAPONREADY3,SETANIM_FLAG_NORMAL);
        }
        */
    }

    NPC_CheckAttackHold();
    NPC_ApplyScriptFlags();

    //cliff and wall avoidance
    NPC_AvoidWallsAndCliffs();

    // run the bot through the server like it was a real client
//=== Save the ucmd for the second no-think Pmove ============================
    ucmd.serverTime = level.time - 50;
    (*NPCInfo).last_ucmd = ucmd;
    if (*NPCInfo).attackHoldTime == 0 {
        (*NPCInfo).last_ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK | BUTTON_FORCE_FOCUS); //so we don't fire twice in one think
    }
//============================================================================
    NPC_CheckAttackScript();
    NPC_KeepCurrentFacing();

    if (*NPC).next_roff_time == 0 || (*NPC).next_roff_time < level.time {
        //If we were following a roff, we don't do normal pmoves.
        ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
    } else {
        NPC_ApplyRoff();
    }

    // end of thinking cleanup
    (*NPCInfo).touchedByPlayer = null_mut();

    NPC_CheckPlayerAim();
    NPC_CheckAllClear();

    /*if( ucmd.forwardmove || ucmd.rightmove )
    {
        int	i, la = -1, ta = -1;

        for(i = 0; i < MAX_ANIMATIONS; i++)
        {
            if( NPC->client->ps.legsAnim == i )
            {
                la = i;
            }

            if( NPC->client->ps.torsoAnim == i )
            {
                ta = i;
            }

            if(la != -1 && ta != -1)
            {
                break;
            }
        }

        if(la != -1 && ta != -1)
        {//FIXME: should never play same frame twice or restart an anim before finishing it
            gi.Printf("LegsAnim: %s(%d) TorsoAnim: %s(%d)\n", animTable[la].name, NPC->renderInfo.legsFrame, animTable[ta].name, NPC->client->renderInfo.torsoFrame);
        }
    }*/
}

pub unsafe fn NPC_CheckInSolid() {
    let mut trace: trace_t = core::mem::zeroed();
    let mut point: vec3_t = [0.0; 3];
    VectorCopy((*NPC).currentOrigin.as_ptr(), point.as_mut_ptr());
    point[2] -= 0.25;

    gi.trace(
        &mut trace,
        (*NPC).currentOrigin.as_ptr(),
        (*NPC).mins.as_ptr(),
        (*NPC).maxs.as_ptr(),
        point.as_ptr(),
        (*NPC).s.number,
        (*NPC).clipmask,
    );
    if trace.startsolid == qfalse && trace.allsolid == qfalse {
        VectorCopy((*NPC).currentOrigin.as_ptr(), (*NPCInfo).lastClearOrigin.as_mut_ptr());
    } else {
        if VectorLengthSquared((*NPCInfo).lastClearOrigin.as_ptr()) != 0.0 {
//			gi.Printf("%s stuck in solid at %s: fixing...\n", NPC->script_targetname, vtos(NPC->currentOrigin));
            G_SetOrigin(NPC, (*NPCInfo).lastClearOrigin.as_ptr());
            gi.linkentity(NPC);
        }
    }
}

/*
===============
NPC_Think

Main NPC AI - called once per frame
===============
*/
#[cfg(feature = "ai_timers")]
pub unsafe fn NPC_Think(self_: *mut gentity_t) {
    let mut oldMoveDir: vec3_t = [0.0; 3];

    (*self_).nextthink = level.time + FRAMETIME / 2;

    SetNPCGlobals(self_);

    addr_of_mut!(ucmd).write(core::mem::zeroed());

    VectorCopy((*(*self_).client).ps.moveDir.as_ptr(), oldMoveDir.as_mut_ptr());
    VectorClear((*(*self_).client).ps.moveDir.as_mut_ptr());
    // see if NPC ai is frozen
    if (*debugNPCFreeze).integer != 0 || ((*NPC).svFlags & SVF_ICARUS_FREEZE) != 0 {
        NPC_UpdateAngles(qtrue, qtrue);
        ClientThink((*self_).s.number, addr_of_mut!(ucmd));
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
        return;
    }

    if self_.is_null() || (*self_).NPC.is_null() || (*self_).client.is_null() {
        return;
    }

    // dead NPCs have a special think, don't run scripts (for now)
    //FIXME: this breaks deathscripts
    if (*self_).health <= 0 {
        DeadThink();
        if (*NPCInfo).nextBStateThink <= level.time {
            if (*self_).m_iIcarusID != IIcarusInterface::ICARUS_INVALID && stop_icarus == qfalse {
                IIcarusInterface::GetIcarus().Update((*self_).m_iIcarusID);
            }
        }
        return;
    }

    // TODO! Tauntaun's (and other creature vehicles?) think, we'll need to make an exception here to allow that.

    if !(*self_).client.is_null()
        && (*(*self_).client).NPC_class == CLASS_VEHICLE
        && !(*self_).NPC_type.is_null()
        && !(*(*self_).m_pVehicle).m_pVehicleInfo.is_null()
        && ((*(*(*self_).m_pVehicle).m_pVehicleInfo).Inhabited)((*self_).m_pVehicle) == qfalse
    {
        //empty swoop logic
        if !(*self_).owner.is_null() {
            //still have attached owner, check and see if can forget him (so he can use me later)
            let mut dir2owner: vec3_t = [0.0; 3];
            VectorSubtract((*(*self_).owner).currentOrigin.as_ptr(), (*self_).currentOrigin.as_ptr(), dir2owner.as_mut_ptr());

            let oldOwner: *mut gentity_t = (*self_).owner;
            (*self_).owner = null_mut(); //clear here for that SpotWouldTelefrag check...?

            if VectorLengthSquared(dir2owner.as_ptr()) > (128 * 128) as f32
                || ((*self_).clipmask & (*oldOwner).clipmask) == 0
                || (DotProduct((*(*self_).client).ps.velocity.as_ptr(), (*(*oldOwner).client).ps.velocity.as_ptr()) < -200.0
                    && G_BoundsOverlap((*self_).absmin.as_ptr(), (*self_).absmin.as_ptr(), (*oldOwner).absmin.as_ptr(), (*oldOwner).absmax.as_ptr()) == qfalse)
            {
                //all clear, become solid to our owner now
                gi.linkentity(self_);
            } else {
                //blocked, retain owner
                (*self_).owner = oldOwner;
            }
        }
    }
    if (*player).client.is_null() == false && (*(*player).client).ps.viewEntity == (*self_).s.number {
        //being controlled by player
        if !(*self_).client.is_null() {
            //make the noises
            if TIMER_Done(self_, b"patrolNoise\0".as_ptr() as *const c_char) != qfalse && Q_irand(0, 20) == 0 {
                if (*(*self_).client).NPC_class == CLASS_R2D2 {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/r2d2/misc/r2d2talk0%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_R5D2 {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/r5d2/misc/r5talk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 4)));
                } else if (*(*self_).client).NPC_class == CLASS_PROBE {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/probe/misc/probetalk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_MOUSE {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/mouse/misc/mousego%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_GONK {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/gonk/misc/gonktalk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 2)));
                }
                TIMER_Set(self_, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
            }
        }
        //FIXME: might want to at least make sounds or something?
        //NPC_UpdateAngles(qtrue, qtrue);
        //Which ucmd should we send?  Does it matter, since it gets overridden anyway?
        (*NPCInfo).last_ucmd.serverTime = level.time - 50;
        ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
        return;
    }

    if (*NPCInfo).nextBStateThink <= level.time {
        let startTime: c_int = GetTime(0);
        if (*NPC).s.eType != ET_PLAYER {
            //Something drastic happened in our script
            return;
        }

        if (*NPC).s.weapon == WP_SABER && (*g_spskill).integer >= 2 && (*NPCInfo).rank > RANK_LT_JG {
            //Jedi think faster on hard difficulty, except low-rank (reborn)
            (*NPCInfo).nextBStateThink = level.time + FRAMETIME / 2;
        } else {
            //Maybe even 200 ms?
            (*NPCInfo).nextBStateThink = level.time + FRAMETIME;
        }

        //nextthink is set before this so something in here can override it
        NPC_ExecuteBState(self_);

        let addTime: c_int = GetTime(startTime);
        if addTime > 50 {
            gi.Printf(
                b"^1ERROR: NPC number %d, %s %s at %s, weaponnum: %d, using %d of AI time!!!\n\0".as_ptr() as *const c_char,
                (*NPC).s.number,
                (*NPC).NPC_type,
                (*NPC).targetname,
                vtos((*NPC).currentOrigin.as_ptr()),
                (*NPC).s.weapon,
                addTime,
            );
        }
        AITime += addTime;
    } else {
        if !(*NPC).client.is_null()
            && (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
            && ((*(*NPC).client).ps.eFlags & EF_FORCE_GRIPPED) != 0
            && (*(*NPC).client).moveType == MT_FLYSWIM
            && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
        {
            //reduce velocity
            VectorScale((*(*NPC).client).ps.velocity.as_ptr(), 0.75, (*(*NPC).client).ps.velocity.as_mut_ptr());
        }
        VectorCopy(oldMoveDir.as_ptr(), (*(*self_).client).ps.moveDir.as_mut_ptr());
        //or use client->pers.lastCommand?
        (*NPCInfo).last_ucmd.serverTime = level.time - 50;
        if (*NPC).next_roff_time == 0 || (*NPC).next_roff_time < level.time {
            //If we were following a roff, we don't do normal pmoves.
            //FIXME: firing angles (no aim offset) or regular angles?
            NPC_UpdateAngles(qtrue, qtrue);
            ucmd = (*NPCInfo).last_ucmd;
            ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
        } else {
            NPC_ApplyRoff();
        }
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
    }
    //must update icarus *every* frame because of certain animation completions in the pmove stuff that can leave a 50ms gap between ICARUS animation commands
    if (*self_).m_iIcarusID != IIcarusInterface::ICARUS_INVALID && stop_icarus == qfalse {
        IIcarusInterface::GetIcarus().Update((*self_).m_iIcarusID);
    }
}

#[cfg(not(feature = "ai_timers"))]
pub unsafe fn NPC_Think(self_: *mut gentity_t) {
    let mut oldMoveDir: vec3_t = [0.0; 3];

    (*self_).nextthink = level.time + FRAMETIME / 2;

    SetNPCGlobals(self_);

    addr_of_mut!(ucmd).write(core::mem::zeroed());

    VectorCopy((*(*self_).client).ps.moveDir.as_ptr(), oldMoveDir.as_mut_ptr());
    VectorClear((*(*self_).client).ps.moveDir.as_mut_ptr());
    // see if NPC ai is frozen
    if (*debugNPCFreeze).integer != 0 || ((*NPC).svFlags & SVF_ICARUS_FREEZE) != 0 {
        NPC_UpdateAngles(qtrue, qtrue);
        ClientThink((*self_).s.number, addr_of_mut!(ucmd));
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
        return;
    }

    if self_.is_null() || (*self_).NPC.is_null() || (*self_).client.is_null() {
        return;
    }

    // dead NPCs have a special think, don't run scripts (for now)
    //FIXME: this breaks deathscripts
    if (*self_).health <= 0 {
        DeadThink();
        if (*NPCInfo).nextBStateThink <= level.time {
            if (*self_).m_iIcarusID != IIcarusInterface::ICARUS_INVALID && stop_icarus == qfalse {
                IIcarusInterface::GetIcarus().Update((*self_).m_iIcarusID);
            }
        }
        return;
    }

    // TODO! Tauntaun's (and other creature vehicles?) think, we'll need to make an exception here to allow that.

    if !(*self_).client.is_null()
        && (*(*self_).client).NPC_class == CLASS_VEHICLE
        && !(*self_).NPC_type.is_null()
        && !(*(*self_).m_pVehicle).m_pVehicleInfo.is_null()
        && ((*(*(*self_).m_pVehicle).m_pVehicleInfo).Inhabited)((*self_).m_pVehicle) == qfalse
    {
        //empty swoop logic
        if !(*self_).owner.is_null() {
            //still have attached owner, check and see if can forget him (so he can use me later)
            let mut dir2owner: vec3_t = [0.0; 3];
            VectorSubtract((*(*self_).owner).currentOrigin.as_ptr(), (*self_).currentOrigin.as_ptr(), dir2owner.as_mut_ptr());

            let oldOwner: *mut gentity_t = (*self_).owner;
            (*self_).owner = null_mut(); //clear here for that SpotWouldTelefrag check...?

            if VectorLengthSquared(dir2owner.as_ptr()) > (128 * 128) as f32
                || ((*self_).clipmask & (*oldOwner).clipmask) == 0
                || (DotProduct((*(*self_).client).ps.velocity.as_ptr(), (*(*oldOwner).client).ps.velocity.as_ptr()) < -200.0
                    && G_BoundsOverlap((*self_).absmin.as_ptr(), (*self_).absmin.as_ptr(), (*oldOwner).absmin.as_ptr(), (*oldOwner).absmax.as_ptr()) == qfalse)
            {
                //all clear, become solid to our owner now
                gi.linkentity(self_);
            } else {
                //blocked, retain owner
                (*self_).owner = oldOwner;
            }
        }
    }
    if !(*player).client.is_null() && (*(*player).client).ps.viewEntity == (*self_).s.number {
        //being controlled by player
        if !(*self_).client.is_null() {
            //make the noises
            if TIMER_Done(self_, b"patrolNoise\0".as_ptr() as *const c_char) != qfalse && Q_irand(0, 20) == 0 {
                if (*(*self_).client).NPC_class == CLASS_R2D2 {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/r2d2/misc/r2d2talk0%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_R5D2 {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/r5d2/misc/r5talk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 4)));
                } else if (*(*self_).client).NPC_class == CLASS_PROBE {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/probe/misc/probetalk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_MOUSE {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/mouse/misc/mousego%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)));
                } else if (*(*self_).client).NPC_class == CLASS_GONK {
                    // droid
                    G_SoundOnEnt(self_, CHAN_AUTO, va(b"sound/chars/gonk/misc/gonktalk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 2)));
                }
                TIMER_Set(self_, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
            }
        }
        //FIXME: might want to at least make sounds or something?
        //NPC_UpdateAngles(qtrue, qtrue);
        //Which ucmd should we send?  Does it matter, since it gets overridden anyway?
        (*NPCInfo).last_ucmd.serverTime = level.time - 50;
        ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
        return;
    }

    if (*NPCInfo).nextBStateThink <= level.time {
        if (*NPC).s.eType != ET_PLAYER {
            //Something drastic happened in our script
            return;
        }

        if (*NPC).s.weapon == WP_SABER && (*g_spskill).integer >= 2 && (*NPCInfo).rank > RANK_LT_JG {
            //Jedi think faster on hard difficulty, except low-rank (reborn)
            (*NPCInfo).nextBStateThink = level.time + FRAMETIME / 2;
        } else {
            //Maybe even 200 ms?
            (*NPCInfo).nextBStateThink = level.time + FRAMETIME;
        }

        //nextthink is set before this so something in here can override it
        NPC_ExecuteBState(self_);
    } else {
        if !(*NPC).client.is_null()
            && (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
            && ((*(*NPC).client).ps.eFlags & EF_FORCE_GRIPPED) != 0
            && (*(*NPC).client).moveType == MT_FLYSWIM
            && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
        {
            //reduce velocity
            VectorScale((*(*NPC).client).ps.velocity.as_ptr(), 0.75, (*(*NPC).client).ps.velocity.as_mut_ptr());
        }
        VectorCopy(oldMoveDir.as_ptr(), (*(*self_).client).ps.moveDir.as_mut_ptr());
        //or use client->pers.lastCommand?
        (*NPCInfo).last_ucmd.serverTime = level.time - 50;
        if (*NPC).next_roff_time == 0 || (*NPC).next_roff_time < level.time {
            //If we were following a roff, we don't do normal pmoves.
            //FIXME: firing angles (no aim offset) or regular angles?
            NPC_UpdateAngles(qtrue, qtrue);
            ucmd = (*NPCInfo).last_ucmd;
            ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
        } else {
            NPC_ApplyRoff();
        }
        VectorCopy((*self_).s.origin.as_ptr(), (*self_).s.origin2.as_mut_ptr());
    }
    //must update icarus *every* frame because of certain animation completions in the pmove stuff that can leave a 50ms gap between ICARUS animation commands
    if (*self_).m_iIcarusID != IIcarusInterface::ICARUS_INVALID && stop_icarus == qfalse {
        IIcarusInterface::GetIcarus().Update((*self_).m_iIcarusID);
    }
}

pub unsafe fn NPC_InitAI() {
    debugNPCAI = gi.cvar(b"d_npcai\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    debugNPCFreeze = gi.cvar(b"d_npcfreeze\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    d_JediAI = gi.cvar(b"d_JediAI\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    d_noGroupAI = gi.cvar(b"d_noGroupAI\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    d_asynchronousGroupAI = gi.cvar(b"d_asynchronousGroupAI\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT);

    //0 = never (BORING)
    //1 = kyle only
    //2 = kyle and last enemy jedi
    //3 = kyle and any enemy jedi
    //4 = kyle and last enemy in a group, special kicks
    //5 = kyle and any enemy
    //6 = also when kyle takes pain or enemy jedi dodges player saber swing or does an acrobatic evasion
    // NOTE : I also create this in UI_Init()
    d_slowmodeath = gi.cvar(b"d_slowmodeath\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, CVAR_ARCHIVE); //save this setting

    d_saberCombat = gi.cvar(b"d_saberCombat\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
}

/*
==================================
void NPC_InitAnimTable( void )

  Need to initialize this table.
  If someone tried to play an anim
  before table is filled in with
  values, causes tasks that wait for
  anim completion to never finish.
  (frameLerp of 0 * numFrames of 0 = 0)
==================================
*/
pub unsafe fn NPC_InitAnimTable() {
    let mut i: c_int = 0;
    while i < MAX_ANIM_FILES {
        let mut j: c_int = 0;
        while j < MAX_ANIMATIONS {
            level.knownAnimFileSets[i as usize].animations[j as usize].firstFrame = 0;
            level.knownAnimFileSets[i as usize].animations[j as usize].frameLerp = 100;
//			level.knownAnimFileSets[i].animations[j].initialLerp = 100;
            level.knownAnimFileSets[i as usize].animations[j as usize].numFrames = 0;
            j += 1;
        }
        i += 1;
    }
}

pub unsafe fn NPC_InitGame() {
//	globals.NPCs = (gNPC_t *) gi.TagMalloc(game.maxclients * sizeof(game.bots[0]), TAG_GAME);
    debugNPCName = gi.cvar(b"d_npc\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, 0);
    NPC_LoadParms();
    NPC_InitAI();
    NPC_InitAnimTable();
    G_ParseAnimFileSet(b"_humanoid\0".as_ptr() as *const c_char, null()); //GET THIS CACHED NOW BEFORE CGAME STARTS
    /*
    ResetTeamCounters();
    for ( int team = TEAM_FREE; team < TEAM_NUM_TEAMS; team++ )
    {
        teamLastEnemyTime[team] = -10000;
    }
    */
}

pub unsafe fn NPC_SetAnim(ent: *mut gentity_t, mut setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, iBlend: c_int) {
    // FIXME : once torsoAnim and legsAnim are in the same structure for NCP and Players
    // rename PM_SETAnimFinal to PM_SetAnim and have both NCP and Players call PM_SetAnim

    if ent.is_null() {
        return;
    }

    if (*ent).health > 0 {
        //don't lock anims if the guy is dead
        if (*(*ent).client).ps.torsoAnimTimer != 0
            && PM_LockedAnim((*(*ent).client).ps.torsoAnim) != qfalse
            && PM_LockedAnim(anim) == qfalse
        {
            //nothing can override these special anims
            setAnimParts &= !SETANIM_TORSO;
        }

        if (*(*ent).client).ps.legsAnimTimer != 0
            && PM_LockedAnim((*(*ent).client).ps.legsAnim) != qfalse
            && PM_LockedAnim(anim) == qfalse
        {
            //nothing can override these special anims
            setAnimParts &= !SETANIM_LEGS;
        }
    }

    if setAnimParts == 0 {
        return;
    }

    if !(*ent).client.is_null() {
        //Players, NPCs
        if (setAnimFlags & SETANIM_FLAG_OVERRIDE) != 0 {
            if (setAnimParts & SETANIM_TORSO) != 0 {
                if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*(*ent).client).ps.torsoAnim != anim {
                    PM_SetTorsoAnimTimer(ent, &mut (*(*ent).client).ps.torsoAnimTimer, 0);
                }
            }
            if (setAnimParts & SETANIM_LEGS) != 0 {
                if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*(*ent).client).ps.legsAnim != anim {
                    PM_SetLegsAnimTimer(ent, &mut (*(*ent).client).ps.legsAnimTimer, 0);
                }
            }
        }

        PM_SetAnimFinal(
            &mut (*(*ent).client).ps.torsoAnim,
            &mut (*(*ent).client).ps.legsAnim,
            setAnimParts,
            anim,
            setAnimFlags,
            &mut (*(*ent).client).ps.torsoAnimTimer,
            &mut (*(*ent).client).ps.legsAnimTimer,
            ent,
            iBlend,
        );
    } else {
        //bodies, etc.
        if (setAnimFlags & SETANIM_FLAG_OVERRIDE) != 0 {
            if (setAnimParts & SETANIM_TORSO) != 0 {
                if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*ent).s.torsoAnim != anim {
                    PM_SetTorsoAnimTimer(ent, &mut (*ent).s.torsoAnimTimer, 0);
                }
            }
            if (setAnimParts & SETANIM_LEGS) != 0 {
                if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*ent).s.legsAnim != anim {
                    PM_SetLegsAnimTimer(ent, &mut (*ent).s.legsAnimTimer, 0);
                }
            }
        }

        PM_SetAnimFinal(
            &mut (*ent).s.torsoAnim,
            &mut (*ent).s.legsAnim,
            setAnimParts,
            anim,
            setAnimFlags,
            &mut (*ent).s.torsoAnimTimer,
            &mut (*ent).s.legsAnimTimer,
            ent,
            0, // iBlend not passed in the bodies branch (default param in C++)
        );
    }
}
