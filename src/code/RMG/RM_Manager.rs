/************************************************************************************************
 *
 * RM_Manager.cpp
 *
 * Implements the CRMManager class.  The CRMManager class manages the arioche system.
 *
 ************************************************************************************************/

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, dead_code, unused_variables, unused_mut, unused_unsafe)]

use crate::code::server::exe_headers_h::*;
use crate::code::RMG::rm_headers_h::*;
use crate::code::server::server_h::*;

use core::ffi::{c_char, c_int};
use core::ptr;

// sprintf is from <stdio.h> (system/libc include, not a ported module)
extern "C" {
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
}

// CRMManager is defined in RM_Manager.h and imported via rm_headers_h glob.
// CCMLandScape, CRandomTerrain, CRMMission, CRMObjective, CGenericParser2, CGPGroup are
// trusted to be imported from their respective header modules via the globs above.
// Do NOT define or stub any of these types here.

// Static member definition: CRMManager::mCurObjective = 0
pub static mut CRMManager_mCurObjective: *mut CRMObjective = ptr::null_mut();

/************************************************************************************************
 * TheRandomMissionManager
 *	Pointer to only active CRMManager class
 *
 ************************************************************************************************/
pub static mut TheRandomMissionManager: *mut CRMManager = ptr::null_mut();

impl CRMManager {
    /************************************************************************************************
     * CRMManager::CRMManager
     *	constructor
     *
     * inputs:
     *
     * return:
     *
     ************************************************************************************************/
    pub fn new() -> Self {
        CRMManager {
            mLandScape: ptr::null_mut(),
            mTerrain:   ptr::null_mut(),
            mMission: ptr::null_mut(),
            mCurPriority: 1,
            mUseTimeLimit: false,
        }
    }

    /************************************************************************************************
     * CRMManager::~CRMManager
     *	destructor
     *
     * inputs:
     *
     * return:
     *
     ************************************************************************************************/
    pub unsafe fn destructor(&mut self) {
        #[cfg(not(feature = "final_build"))]
        Com_Printf(b"... Shutting down TheRandomMissionManager\n\0".as_ptr() as *const c_char);
        #[cfg(not(feature = "dedicated"))]
        CM_TM_Free();
        if !self.mMission.is_null() {
            drop(Box::from_raw(self.mMission));
            self.mMission = ptr::null_mut();
        }
    }

    /************************************************************************************************
     * CRMManager::SetLandscape
     *	Sets the landscape and terrain object used to load a mission
     *
     * inputs:
     *	landscape - landscape object
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn SetLandScape(&mut self, landscape: *mut CCMLandScape) {
        self.mLandScape = landscape;
        self.mTerrain = (*landscape).GetRandomTerrain();
    }

    /************************************************************************************************
     * CRMManager::LoadMission
     *	Loads the mission using the mission name stored in the ar_mission cvar
     *
     * inputs:
     *	none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn LoadMission(&mut self, IsServer: qboolean) -> bool {
        #[cfg(not(feature = "pre_release_demo"))]
        {
            let mut instances: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut mission:   [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut course:    [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut map:       [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut temp:      [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

            #[cfg(not(feature = "final_build"))]
            {
                Com_Printf(b"--------- Random Mission Manager ---------\n\n\0".as_ptr() as *const c_char);
                Com_Printf(b"RMG version : 0.01\n\n\0".as_ptr() as *const c_char);
            }

            if self.mTerrain.is_null() {
                return false;
            }

            // Grab the arioche variables
            Cvar_VariableStringBuffer(b"rmg_usetimelimit\0".as_ptr() as *const c_char, temp.as_mut_ptr(), MAX_QPATH);
            if strcmpi(temp.as_ptr(), b"yes\0".as_ptr() as *const c_char) == 0 {
                self.mUseTimeLimit = true;
            }
            Cvar_VariableStringBuffer(b"rmg_instances\0".as_ptr() as *const c_char, instances.as_mut_ptr(), MAX_QPATH);
            Cvar_VariableStringBuffer(b"RMG_mission\0".as_ptr() as *const c_char, temp.as_mut_ptr(), MAX_QPATH);
            Cvar_VariableStringBuffer(b"rmg_map\0".as_ptr() as *const c_char, map.as_mut_ptr(), MAX_QPATH);
            sprintf(mission.as_mut_ptr(), b"%s_%s\0".as_ptr() as *const c_char, temp.as_ptr(), map.as_ptr());
            Cvar_VariableStringBuffer(b"rmg_course\0".as_ptr() as *const c_char, course.as_mut_ptr(), MAX_QPATH);

            // dump existing mission, if any
            if !self.mMission.is_null() {
                drop(Box::from_raw(self.mMission));
                self.mMission = ptr::null_mut();
            }

            // Create a new mission file
            self.mMission = Box::into_raw(Box::new(CRMMission::new(self.mTerrain)));

            // Load the mission using the arioche variables
            if !(*self.mMission).Load(mission.as_ptr(), instances.as_ptr(), course.as_ptr()) {
                return false;
            }

            if self.mUseTimeLimit {
                Cvar_Set(b"rmg_timelimit\0".as_ptr() as *const c_char, va(b"%d\0".as_ptr() as *const c_char, (*self.mMission).GetTimeLimit()));
            } else {
                Cvar_Set(b"rmg_timelimit\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
            }

            if IsServer != 0 {
                // set the names of the teams
                let mut parser: CGenericParser2 = core::mem::zeroed();
                //CGPGroup*			root;

                Cvar_VariableStringBuffer(b"RMG_terrain\0".as_ptr() as *const c_char, temp.as_mut_ptr(), MAX_QPATH);

                /*
                // Create the parser for the mission file
                if(Com_ParseTextFile(va("ext_data/rmg/%s.teams", temp), parser))
                {
                    root = parser.GetBaseParseGroup()->GetSubGroups();
                    if (0 == stricmp(root->GetName(), "teams"))
                    {
                        SV_SetConfigstring( CS_GAMETYPE_REDTEAM, root->FindPairValue ( "red", "marine" ));
                        SV_SetConfigstring( CS_GAMETYPE_BLUETEAM, root->FindPairValue ( "blue", "thug" ));
                    }
                    parser.Clean();
                }
                */
                //rww - This is single player, no such thing.
            }

            // Must have a valid landscape before we can spawn the mission
            assert!(!self.mLandScape.is_null());

            #[cfg(not(feature = "final_build"))]
            Com_Printf(b"------------------------------------------\n\0".as_ptr() as *const c_char);

            return true;
        }
        #[cfg(feature = "pre_release_demo")]
        return false;
        // Port note: trailing false satisfies the type-checker when exactly one cfg branch is
        // inactive; unreachable at runtime since the two cfg guards are complementary.
        #[allow(unreachable_code)]
        false
    }

    /************************************************************************************************
     * CRMManager::IsMissionComplete
     *	Determines whether or not all the arioche objectives have been met
     *
     * inputs:
     *  none
     *
     * return:
     *	true: all objectives have been completed
     *  false: one or more of the objectives has not been met
     *
     ************************************************************************************************/
    pub unsafe fn IsMissionComplete(&mut self) -> bool {
        if ptr::null_mut() == (*self.mMission).GetCurrentObjective() {
            return true;
        }

        return false;
    }

    /************************************************************************************************
     * CRMManager::HasTimeExpired
     *	Determines whether or not the time limit (if one) has expired
     *
     * inputs:
     *  none
     *
     * return:
     *	true: time limit has expired
     *  false: time limit has not expired
     *
     ************************************************************************************************/
    pub fn HasTimeExpired(&self) -> bool {
        /*	if (mMission->GetTimeLimit() == 0 || !mUseTimeLimit)
        	{	// no time limit set
        		return false;
        	}

        	if (mMission->GetTimeLimit() * 1000 * 60 > level.time - level.startTime)
        	{	// we are still under our time limit
        		return false;
        	}

        	// over our time limit!
        	return true;*/

        return false;
    }

    /************************************************************************************************
     * CRMManager::UpdateStatisticCvars
     *	Updates the statistic cvars with data from the game
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    fn UpdateStatisticCvars(&self) {
        /*	// No player set then nothing more to do
        	if ( mPlayer )
        	{
        		float accuracy;

        		// Calculate the accuracy
        		accuracy  = (float)mPlayer->client->ps.persistant[PERS_SHOTS_HIT];
        		accuracy /= (float)mPlayer->client->ps.persistant[PERS_SHOTS];
        		accuracy *= 100.0f;

        		// set the accuracy cvar
        		gi.Cvar_Set ( "ar_pl_accuracy", va("%d%%",(int)accuracy) );

        		// Set the # of kills cvar
        		gi.Cvar_Set ( "ar_kills", va("%d", mPlayer->client->ps.persistant[PERS_SCORE] ) );

        		int hours;
        		int mins;
        		int seconds;
        		int tens;
        		int millisec = (level.time - level.startTime);

        		seconds = millisec / 1000;
        		hours = seconds / (60 * 60);
        		seconds -= (hours * 60 * 60);
        		mins = seconds / 60;
        		seconds -= mins * 60;
        		tens = seconds / 10;
        		seconds -= tens * 10;

        		gi.Cvar_Set ( "ar_duration", va("%dhr %dmin %dsec", hours, mins, seconds ) );

        		WpnID wpnID = TheWpnSysMgr().GetFavoriteWeapon ( );
        		gi.Cvar_Set ( "ar_fav_wp", CWeaponSystem::GetWpnName ( wpnID ) );

        		// show difficulty
        		char difficulty[MAX_QPATH];
        		gi.Cvar_VariableStringBuffer("g_skill", difficulty, MAX_QPATH);
        		strupr(difficulty);
        		gi.Cvar_Set ( "ar_diff", va("&GENERIC_%s&",difficulty) );

        		// compute rank
        		float compositeRank = 1;
        		int rankMax = 3;  // max rank less 1
        		float timeRank = mUseTimeLimit ? (1.0f - (mins / (float)mMission->GetTimeLimit())) : 0;
        		float killRank = mPlayer->client->ps.persistant[PERS_SCORE] / (float)GetCharacterManager().GetAllSize();
        		killRank = (killRank > 0) ? killRank : 1.0f;
        		float accuRank = (accuracy > 0) ? accuracy*0.01f : 1.0f;
        		float weapRank = 1.0f - CWeaponSystem::GetRank(wpnID);

        		compositeRank = ((timeRank + killRank + accuRank + weapRank) / 3.0f) * rankMax + 1;

        		if (compositeRank > 4)
        			compositeRank = 4;

        		gi.Cvar_Set ( "ar_rank", va("&RMG_RANK%d&",((int)compositeRank)) );
        	}*/
    }

    /************************************************************************************************
     * CRMManager::CompleteMission
     *	Does end-of-mission stuff (pause game, end screen, return to menu)
     *    <Description>                                                                             *
     * Input                                                                                        *
     *    <Variable>: <Description>                                                                 *
     * Output / Return                                                                              *
     *    <Variable>: <Description>                                                                 *
     ************************************************************************************************/
    pub unsafe fn CompleteMission(&mut self) {
        self.UpdateStatisticCvars();

        (*self.mMission).CompleteMission();
    }

    /************************************************************************************************
     * CRMManager::FailedMission
     *	Does end-of-mission stuff (pause game, end screen, return to menu)
     *    <Description>                                                                             *
     * Input                                                                                        *
     *    TimeExpired: indicates if the reason failed was because of time
     * Output / Return                                                                              *
     *    <Variable>: <Description>                                                                 *
     ************************************************************************************************/
    pub unsafe fn FailedMission(&mut self, TimeExpired: bool) {
        self.UpdateStatisticCvars();

        (*self.mMission).FailedMission(TimeExpired);
    }

    /************************************************************************************************
     * CRMManager::CompleteObjective
     *	Marks the given objective as completed
     *
     * inputs:
     *  obj:  objective to set as completed
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn CompleteObjective(&mut self, obj: *mut CRMObjective) {
        assert!(!obj.is_null());

        (*self.mMission).CompleteObjective(obj);
    }

    /************************************************************************************************
     * CRMManager::Preview
     *	previews the random mission genration information
     *
     * inputs:
     *  from:  origin being previed from
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Preview(&self, from: vec3_t) {
        // Dont bother if we havent reached our timer yet
        /*	if ( level.time < mPreviewTimer )
        	{
        		return;
        	}

        	// Let the mission do all the previewing
        	mMission->Preview ( from );

        	// Another second
        	mPreviewTimer = level.time + 1000;*/
    }

    /************************************************************************************************
     * CRMManager::Preview
     *	previews the random mission genration information
     *
     * inputs:
     *  from:  origin being previed from
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn SpawnMission(&mut self, IsServer: qboolean) -> bool {
        // Spawn the mission
        (*self.mMission).Spawn(self.mTerrain, IsServer);

        return true;
    }
}
