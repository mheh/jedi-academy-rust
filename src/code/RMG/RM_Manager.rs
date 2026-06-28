/************************************************************************************************
 *
 * RM_Manager.cpp
 *
 * Implements the CRMManager class.  The CRMManager class manages the arioche system.
 *
 ************************************************************************************************/

use core::ffi::c_int;
use std::ptr;

// porting stub: external types declared as opaque
#[repr(C)]
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CRMMission {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CRMObjective {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CGenericParser2 {
    _opaque: [u8; 0],
}

pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

#[repr(C)]
pub struct CRMManager {
    pub mMission: *mut CRMMission,
    pub mLandScape: *mut CCMLandScape,
    pub mTerrain: *mut CRandomTerrain,
    pub mPreviewTimer: c_int,
    pub mCurPriority: c_int,
    pub mUseTimeLimit: bool,
}

// eek
pub static mut mCurObjective: *mut CRMObjective = ptr::null_mut();

/************************************************************************************************
 * TheRandomMissionManager
 *	Pointer to only active CRMManager class
 *
 ************************************************************************************************/
pub static mut TheRandomMissionManager: *mut CRMManager = ptr::null_mut();

// porting stub: external functions
extern "C" {
    fn Com_Printf(format: *const u8, ...);
    fn Cvar_VariableStringBuffer(var_name: *const u8, buffer: *mut u8, bufsize: c_int);
    fn Cvar_Set(var_name: *const u8, value: *const u8);
    fn CM_TM_Free();
}

// porting stubs: libc functions
extern "C" {
    fn strcmpi(s1: *const u8, s2: *const u8) -> c_int;
    fn sprintf(s: *mut u8, format: *const u8, ...) -> c_int;
}

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
            mTerrain: ptr::null_mut(),
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
    pub fn drop(&mut self) {
        #[cfg(not(feature = "final_build"))]
        unsafe {
            Com_Printf(b"... Shutting down TheRandomMissionManager\n\0".as_ptr());
        }
        #[cfg(not(feature = "dedicated"))]
        unsafe {
            CM_TM_Free();
        }
        if !self.mMission.is_null() {
            unsafe {
                let _ = Box::from_raw(self.mMission);
            }
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
    pub fn SetLandScape(&mut self, landscape: *mut CCMLandScape) {
        self.mLandScape = landscape;
        if !landscape.is_null() {
            unsafe {
                self.mTerrain = (*landscape).GetRandomTerrain();
            }
        }
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
    pub fn LoadMission(&mut self, IsServer: qboolean) -> bool {
        #[cfg(not(feature = "pre_release_demo"))]
        {
            let mut instances: [u8; 260] = [0; 260];
            let mut mission: [u8; 260] = [0; 260];
            let mut course: [u8; 260] = [0; 260];
            let mut map: [u8; 260] = [0; 260];
            let mut temp: [u8; 260] = [0; 260];

            #[cfg(not(feature = "final_build"))]
            unsafe {
                Com_Printf(b"--------- Random Mission Manager ---------\n\n\0".as_ptr());
                Com_Printf(b"RMG version : 0.01\n\n\0".as_ptr());
            }

            if self.mTerrain.is_null() {
                return false;
            }

            // Grab the arioche variables
            unsafe {
                Cvar_VariableStringBuffer(
                    b"rmg_usetimelimit\0".as_ptr(),
                    temp.as_mut_ptr(),
                    260 as c_int,
                );
                if strcmpi(temp.as_ptr(), b"yes\0".as_ptr()) == 0 {
                    self.mUseTimeLimit = true;
                }
                Cvar_VariableStringBuffer(
                    b"rmg_instances\0".as_ptr(),
                    instances.as_mut_ptr(),
                    260 as c_int,
                );
                Cvar_VariableStringBuffer(
                    b"RMG_mission\0".as_ptr(),
                    temp.as_mut_ptr(),
                    260 as c_int,
                );
                Cvar_VariableStringBuffer(
                    b"rmg_map\0".as_ptr(),
                    map.as_mut_ptr(),
                    260 as c_int,
                );
                sprintf(
                    mission.as_mut_ptr(),
                    b"%s_%s\0".as_ptr(),
                    temp.as_ptr(),
                    map.as_ptr(),
                );
                Cvar_VariableStringBuffer(
                    b"rmg_course\0".as_ptr(),
                    course.as_mut_ptr(),
                    260 as c_int,
                );
            }

            // dump existing mission, if any
            if !self.mMission.is_null() {
                unsafe {
                    let _ = Box::from_raw(self.mMission);
                }
                self.mMission = ptr::null_mut();
            }

            // Create a new mission file
            self.mMission = Box::into_raw(Box::new(CRMMission { _opaque: [] }));

            // Load the mission using the arioche variables
            if !self.mMission.is_null() {
                unsafe {
                    if !(*self.mMission).Load(
                        mission.as_ptr(),
                        instances.as_ptr(),
                        course.as_ptr(),
                    ) {
                        return false;
                    }
                }
            } else {
                return false;
            }

            if self.mUseTimeLimit {
                unsafe {
                    let time_limit = (*self.mMission).GetTimeLimit();
                    let mut buffer: [u8; 260] = [0; 260];
                    sprintf(buffer.as_mut_ptr(), b"%d\0".as_ptr(), time_limit);
                    Cvar_Set(b"rmg_timelimit\0".as_ptr(), buffer.as_ptr());
                }
            } else {
                unsafe {
                    Cvar_Set(b"rmg_timelimit\0".as_ptr(), b"0\0".as_ptr());
                }
            }

            if IsServer != 0 {
                // set the names of the teams
                let _parser: CGenericParser2 = CGenericParser2 { _opaque: [] };
                //CGPGroup*			root;

                unsafe {
                    Cvar_VariableStringBuffer(
                        b"RMG_terrain\0".as_ptr(),
                        temp.as_mut_ptr(),
                        260 as c_int,
                    );
                }

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
            unsafe {
                Com_Printf(b"------------------------------------------\n\0".as_ptr());
            }

            return true;
        }
        #[cfg(feature = "pre_release_demo")]
        {
            return false;
        }
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
    pub fn IsMissionComplete(&self) -> bool {
        if !self.mMission.is_null() {
            unsafe {
                if (*self.mMission).GetCurrentObjective().is_null() {
                    return true;
                }
            }
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
    pub fn CompleteMission(&mut self) {
        self.UpdateStatisticCvars();

        if !self.mMission.is_null() {
            unsafe {
                (*self.mMission).CompleteMission();
            }
        }
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
    pub fn FailedMission(&mut self, TimeExpired: bool) {
        self.UpdateStatisticCvars();

        if !self.mMission.is_null() {
            unsafe {
                (*self.mMission).FailedMission(TimeExpired);
            }
        }
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
    pub fn CompleteObjective(&mut self, obj: *mut CRMObjective) {
        assert!(!obj.is_null());

        if !self.mMission.is_null() {
            unsafe {
                (*self.mMission).CompleteObjective(obj);
            }
        }
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
    pub fn Preview(&self, from: &vec3_t) {
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
    pub fn SpawnMission(&mut self, IsServer: qboolean) -> bool {
        // Spawn the mission
        if !self.mMission.is_null() && !self.mTerrain.is_null() {
            unsafe {
                (*self.mMission).Spawn(self.mTerrain, IsServer);
            }
        }

        return true;
    }

    // Inline accessor methods from header
    pub fn SetCurPriority(&mut self, priority: c_int) {
        self.mCurPriority = priority;
    }

    pub fn GetTerrain(&self) -> *mut CRandomTerrain {
        self.mTerrain
    }

    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        self.mLandScape
    }

    pub fn GetMission(&self) -> *mut CRMMission {
        self.mMission
    }

    pub fn GetCurPriority(&self) -> c_int {
        self.mCurPriority
    }
}

// porting stubs: external class methods
impl CCMLandScape {
    pub fn GetRandomTerrain(&self) -> *mut CRandomTerrain {
        // porting stub: unimplemented method
        ptr::null_mut()
    }
}

impl CRMMission {
    pub fn Load(
        &mut self,
        mission: *const u8,
        instances: *const u8,
        course: *const u8,
    ) -> bool {
        // porting stub: unimplemented method
        true
    }

    pub fn GetTimeLimit(&self) -> c_int {
        // porting stub: unimplemented method
        0
    }

    pub fn GetCurrentObjective(&self) -> *mut CRMObjective {
        // porting stub: unimplemented method
        ptr::null_mut()
    }

    pub fn CompleteMission(&mut self) {
        // porting stub: unimplemented method
    }

    pub fn FailedMission(&mut self, _TimeExpired: bool) {
        // porting stub: unimplemented method
    }

    pub fn CompleteObjective(&mut self, _obj: *mut CRMObjective) {
        // porting stub: unimplemented method
    }

    pub fn Spawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: qboolean) {
        // porting stub: unimplemented method
    }
}
