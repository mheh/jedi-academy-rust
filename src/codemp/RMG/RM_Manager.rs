// Anything above this #include will be ignored by the compiler

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::exe_headers_h::*;

/************************************************************************************************
 *
 * RM_Manager.cpp
 *
 * Implements the CRMManager class.  The CRMManager class manages the arioche system.
 *
 ************************************************************************************************/

use crate::codemp::RMG::RM_Headers_h::*;
use crate::codemp::server::server_h::*;
use crate::codemp::qcommon::qcommon_h::*;

use core::ffi::{c_char, c_int};

extern "C" {
    fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_ParseTextFile(filename: *const c_char, parser: *mut CGenericParser2) -> bool;
    fn va(format: *const c_char, ...) -> *mut c_char;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn CM_TM_Free();
    fn CM_TM_AddBuilding(x: f32, y: f32, side: c_int);
    fn CM_TM_AddObjective(x: f32, y: f32, side: c_int);
    fn CM_TM_AddStart(x: f32, y: f32, side: c_int);
    fn CM_TM_AddEnd(x: f32, y: f32, side: c_int);
    fn CM_TM_AddWallRect(x: f32, y: f32, side: c_int);
}

pub static mut mCurObjective: *mut CRMObjective = core::ptr::null_mut();

/************************************************************************************************
 * TheRandomMissionManager
 *	Pointer to only active CRMManager class
 *
 ************************************************************************************************/
pub static mut TheRandomMissionManager: *mut CRMManager = core::ptr::null_mut();

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
        // Port note: mPreviewTimer is not set by the C++ constructor; zero-initialized here.
        CRMManager {
            mLandScape: core::ptr::null_mut(),
            mTerrain: core::ptr::null_mut(),
            mMission: core::ptr::null_mut(),
            mPreviewTimer: 0,
            mCurPriority: 1,
            mUseTimeLimit: false,
            mAutomapSymbolCount: 0,
            mAutomapSymbols: [rmAutomapSymbol_t { mType: 0, mSide: 0, mOrigin: [0.0, 0.0, 0.0] };
                MAX_AUTOMAP_SYMBOLS],
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
            let mut mission: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut course: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut map: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut temp: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

            #[cfg(not(feature = "final_build"))]
            {
                Com_Printf(
                    b"--------- Random Mission Manager ---------\n\n\0".as_ptr() as *const c_char,
                );
                Com_Printf(b"RMG version : 1.01\n\n\0".as_ptr() as *const c_char);
            }

            if self.mTerrain.is_null() {
                return false;
            }

            // Grab the arioche variables
            Cvar_VariableStringBuffer(
                b"rmg_usetimelimit\0".as_ptr() as *const c_char,
                temp.as_mut_ptr(),
                MAX_QPATH as c_int,
            );
            if Q_stricmp(temp.as_ptr(), b"yes\0".as_ptr() as *const c_char) == 0 {
                self.mUseTimeLimit = true;
            }
            Cvar_VariableStringBuffer(
                b"rmg_instances\0".as_ptr() as *const c_char,
                instances.as_mut_ptr(),
                MAX_QPATH as c_int,
            );
            Cvar_VariableStringBuffer(
                b"RMG_mission\0".as_ptr() as *const c_char,
                temp.as_mut_ptr(),
                MAX_QPATH as c_int,
            );
            Cvar_VariableStringBuffer(
                b"rmg_map\0".as_ptr() as *const c_char,
                map.as_mut_ptr(),
                MAX_QPATH as c_int,
            );
            sprintf(
                mission.as_mut_ptr(),
                b"%s_%s\0".as_ptr() as *const c_char,
                temp.as_ptr(),
                map.as_ptr(),
            );
            Cvar_VariableStringBuffer(
                b"rmg_course\0".as_ptr() as *const c_char,
                course.as_mut_ptr(),
                MAX_QPATH as c_int,
            );

            // dump existing mission, if any
            if !self.mMission.is_null() {
                drop(Box::from_raw(self.mMission));
                self.mMission = core::ptr::null_mut();
            }

            // Create a new mission file
            self.mMission = Box::into_raw(Box::new(CRMMission::new(self.mTerrain)));

            if IsServer != 0 {
                // Load the mission using the arioche variables
                if !(*self.mMission).Load(
                    mission.as_ptr(),
                    instances.as_ptr(),
                    course.as_ptr(),
                ) {
                    return false;
                }

                // set the names of the teams
                let mut parser: CGenericParser2 = CGenericParser2::new();

                Cvar_VariableStringBuffer(
                    b"RMG_terrain\0".as_ptr() as *const c_char,
                    temp.as_mut_ptr(),
                    MAX_QPATH as c_int,
                );

                // Create the parser for the mission file
                if Com_ParseTextFile(
                    va(
                        b"ext_data/rmg/%s.teams\0".as_ptr() as *const c_char,
                        temp.as_ptr(),
                    ),
                    &mut parser as *mut CGenericParser2,
                ) {
                    let root: *mut CGPGroup =
                        (*parser.GetBaseParseGroup()).GetSubGroups();
                    if 0 == stricmp(
                        (*root).GetName(),
                        b"teams\0".as_ptr() as *const c_char,
                    ) {
                        /*
                        SV_SetConfigstring( CS_GAMETYPE_REDTEAM, root->FindPairValue ( "red", "marine" ));
                        SV_SetConfigstring( CS_GAMETYPE_BLUETEAM, root->FindPairValue ( "blue", "thug" ));
                        */
                        //rwwFIXMEFIXME: Do we care about this?
                    }
                    parser.Clean();
                }
            }

            // Must have a valid landscape before we can spawn the mission
            assert!(!self.mLandScape.is_null());

            #[cfg(not(feature = "final_build"))]
            {
                Com_Printf(
                    b"------------------------------------------\n\0".as_ptr() as *const c_char,
                );
            }

            return true;
        }
        // #else
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
    pub unsafe fn IsMissionComplete(&self) -> bool {
        if core::ptr::null_mut::<CRMObjective>() == (*self.mMission).GetCurrentObjective() {
            return true;
        }

        false
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

        false
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
    fn UpdateStatisticCvars(&mut self) {
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
    pub fn Preview(&self, _from: vec3_t) {
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

        true
    }

    pub unsafe fn AddAutomapSymbol(&mut self, type_: c_int, origin: vec3_t, side: c_int) {
        if type_ == 0 {
            return;
        }

        self.mAutomapSymbols[self.mAutomapSymbolCount as usize].mType = type_;
        self.mAutomapSymbols[self.mAutomapSymbolCount as usize].mSide = side;
        // VectorCopy ( origin, mAutomapSymbols[mAutomapSymbolCount].mOrigin )
        self.mAutomapSymbols[self.mAutomapSymbolCount as usize].mOrigin = origin;
        self.mAutomapSymbolCount += 1;
    }

    pub fn GetAutomapSymbolCount(&self) -> c_int {
        self.mAutomapSymbolCount
    }

    pub fn GetAutomapSymbol(&mut self, index: c_int) -> *mut rmAutomapSymbol_t {
        unsafe { self.mAutomapSymbols.as_mut_ptr().add(index as usize) }
    }

    /*
    void CRMManager::WriteAutomapSymbols ( msg_t* msg )
    {
        rmAutomapSymbolIter_t it;

        MSG_WriteShort ( msg, (unsigned long)mAutomapSymbols.size() );

        for(it = mAutomapSymbols.begin(); it != mAutomapSymbols.end(); it++)
        {
            CRMAutomapSymbol* symbol = (CRMAutomapSymbol*) *it;

            MSG_WriteByte ( msg, (unsigned char) symbol->mType );
            MSG_WriteByte ( msg, (unsigned char) symbol->mSide );
            MSG_WriteLong ( msg, (long) symbol->mOrigin[0] );
            MSG_WriteLong ( msg, (long) symbol->mOrigin[1] );
        }
    }
    */

    pub unsafe fn ProcessAutomapSymbols(count: c_int, symbols: *mut rmAutomapSymbol_t) {
        #[cfg(not(feature = "dedicated"))]
        {
            let mut i: c_int = 0;

            while i < count {
                // draw proper symbol on map for instance
                match (*symbols.add(i as usize)).mType {
                    x if x == AUTOMAP_BLD => {
                        CM_TM_AddBuilding(
                            (*symbols.add(i as usize)).mOrigin[0],
                            (*symbols.add(i as usize)).mOrigin[1],
                            (*symbols.add(i as usize)).mSide,
                        );
                    }
                    x if x == AUTOMAP_OBJ => {
                        CM_TM_AddObjective(
                            (*symbols.add(i as usize)).mOrigin[0],
                            (*symbols.add(i as usize)).mOrigin[1],
                            (*symbols.add(i as usize)).mSide,
                        );
                    }
                    x if x == AUTOMAP_START => {
                        CM_TM_AddStart(
                            (*symbols.add(i as usize)).mOrigin[0],
                            (*symbols.add(i as usize)).mOrigin[1],
                            (*symbols.add(i as usize)).mSide,
                        );
                    }
                    x if x == AUTOMAP_END => {
                        CM_TM_AddEnd(
                            (*symbols.add(i as usize)).mOrigin[0],
                            (*symbols.add(i as usize)).mOrigin[1],
                            (*symbols.add(i as usize)).mSide,
                        );
                    }
                    x if x == AUTOMAP_ENEMY => {}
                    x if x == AUTOMAP_FRIEND => {}
                    x if x == AUTOMAP_WALL => {
                        CM_TM_AddWallRect(
                            (*symbols.add(i as usize)).mOrigin[0],
                            (*symbols.add(i as usize)).mOrigin[1],
                            (*symbols.add(i as usize)).mSide,
                        );
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }
}

impl Drop for CRMManager {
    /************************************************************************************************
     * CRMManager::~CRMManager
     *	destructor
     *
     * inputs:
     *
     * return:
     *
     ************************************************************************************************/
    fn drop(&mut self) {
        #[cfg(not(feature = "final_build"))]
        unsafe {
            Com_Printf(
                b"... Shutting down TheRandomMissionManager\n\0".as_ptr() as *const c_char,
            );
        }
        #[cfg(not(feature = "dedicated"))]
        unsafe {
            CM_TM_Free();
        }
        if !self.mMission.is_null() {
            unsafe {
                drop(Box::from_raw(self.mMission));
            }
            self.mMission = core::ptr::null_mut();
        }
    }
}

impl Default for CRMManager {
    fn default() -> Self {
        Self::new()
    }
}
