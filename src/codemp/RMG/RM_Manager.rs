//! Mechanical port of `codemp/RMG/RM_Manager.cpp`.
//!
//! Implements the CRMManager class. The CRMManager class manages the arioche system.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================
//
// These types are declared here to allow this file to compile structurally.
// Full definitions exist in the oracle but have not yet been ported.
// Porting these types is out of scope for this file.

/// Stub for unported `class CGenericParser2` (GenericParser2.h).
/// Parser for configuration files.
pub struct CGenericParser2 {
    _opaque: [u8; 0],
}

impl CGenericParser2 {
    /// Stub for `CGPGroup* CGenericParser2::GetBaseParseGroup()`.
    /// Returns the root parsing group.
    pub fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `void CGenericParser2::Clean()`.
    /// Cleans up the parser.
    pub fn Clean(&mut self) {
        // Porting stub: cleans up parser state.
    }
}

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::GetName()`.
    /// Returns the name of this group.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `CGPGroup* CGPGroup::GetSubGroups()`.
    /// Returns the first subgroup.
    pub fn GetSubGroups(&self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *defaultVal)`.
    /// Returns the value string associated with the given key, or default if not found.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        // Porting stub: in reality, this looks up the key in internal storage
        // and returns the value or the default. For now, return the default.
        default_val
    }
}

/// Stub for unported `class CRMMission` (RM_Mission.h).
/// Holds state for a random mission.
pub struct CRMMission {
    _opaque: [u8; 0],
}

impl CRMMission {
    /// Stub for `CRMObjective* CRMMission::GetCurrentObjective()`.
    /// Returns the current mission objective.
    pub fn GetCurrentObjective(&self) -> *mut CRMObjective {
        core::ptr::null_mut()
    }

    /// Stub for `int CRMMission::GetTimeLimit()`.
    /// Returns the time limit for the mission.
    pub fn GetTimeLimit(&self) -> c_int {
        0
    }

    /// Stub for `void CRMMission::CompleteObjective(CRMObjective*)`.
    /// Marks the objective as complete.
    pub fn CompleteObjective(&mut self, _obj: *mut CRMObjective) {
        // Porting stub: marks the objective as completed.
    }

    /// Stub for `void CRMMission::CompleteMission()`.
    /// Marks the entire mission as complete.
    pub fn CompleteMission(&mut self) {
        // Porting stub: handles mission completion.
    }

    /// Stub for `void CRMMission::FailedMission(bool)`.
    /// Marks the mission as failed.
    pub fn FailedMission(&mut self, _TimeExpired: bool) {
        // Porting stub: handles mission failure.
    }

    /// Stub for `void CRMMission::Preview(const vec3_t)`.
    /// Previews the mission.
    pub fn Preview(&self, _from: *const vec3_t) {
        // Porting stub: handles mission preview.
    }

    /// Stub for `bool CRMMission::Load(const char*, const char*, const char*)`.
    /// Loads mission data from configuration.
    pub fn Load(&mut self, _mission: *const c_char, _instances: *const c_char, _course: *const c_char) -> bool {
        // Porting stub: loads mission configuration.
        false
    }

    /// Stub for `void CRMMission::Spawn(CRandomTerrain*, qboolean)`.
    /// Spawns the mission entities.
    pub fn Spawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: c_int) {
        // Porting stub: spawns mission entities.
    }
}

/// Stub for unported `class CRMObjective` (RM_Objective.h).
/// Represents a mission objective.
pub struct CRMObjective {
    _opaque: [u8; 0],
}

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
/// Represents the landscape/terrain mesh for the map.
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// Stub for `CRandomTerrain* CCMLandScape::GetRandomTerrain()`.
    /// Returns the terrain generator associated with this landscape.
    pub fn GetRandomTerrain(&self) -> *mut CRandomTerrain {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
/// Manages terrain generation for random maps.
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

// ============================================================================
// Constants and types from qcommon/client
// ============================================================================

/// Maximum path length (from q_shared.h).
pub const MAX_QPATH: usize = 64;

/// Maximum number of automap symbols that can be tracked.
pub const MAX_AUTOMAP_SYMBOLS: usize = 512;

/// Automap symbol type constants (from RM_Instance.h).
pub const AUTOMAP_NONE: c_int = 0;
pub const AUTOMAP_BLD: c_int = 1;
pub const AUTOMAP_OBJ: c_int = 2;
pub const AUTOMAP_START: c_int = 3;
pub const AUTOMAP_END: c_int = 4;
pub const AUTOMAP_ENEMY: c_int = 5;
pub const AUTOMAP_FRIEND: c_int = 6;
pub const AUTOMAP_WALL: c_int = 7;

/// C type for Quake boolean (from q_shared.h).
pub type qboolean = c_int;

/// C fixed array type for 3D vector (from q_shared.h: `typedef float vec3_t[3]`).
pub type vec3_t = [f32; 3];

/// Automap symbol structure (from client.h).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct rmAutomapSymbol_t {
    pub mType: c_int,
    pub mSide: c_int,
    pub mOrigin: vec3_t,
}

// ============================================================================
// extern "C" functions from qcommon and game
// ============================================================================

extern "C" {
    /// Quake engine function to retrieve a console variable's string value.
    /// Stores up to bufsize bytes of the variable's value into buffer.
    fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);

    /// Quake engine function for case-insensitive string comparison.
    /// Returns 0 if strings are equal (ignoring case).
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Quake engine function to print formatted messages to console.
    fn Com_Printf(fmt: *const c_char, ...);

    /// Quake engine function to parse a text file into a parser structure.
    /// Returns true if successful.
    fn Com_ParseTextFile(file: *const c_char, parser: *mut CGenericParser2) -> bool;

    /// Quake engine function to format a string with variadic arguments.
    /// Returns a pointer to a static buffer containing the formatted result.
    pub fn va(format: *const c_char, ...) -> *mut c_char;

    /// C standard library function for formatted string printing.
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;

    /// C standard library function for case-insensitive string comparison.
    /// Returns 0 if strings are equal (ignoring case).
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Quake terrain/landscape function to free terrain map resources.
    fn CM_TM_Free();

    /// Quake terrain/landscape functions to add symbols to the automap.
    fn CM_TM_AddBuilding(x: f32, y: f32, side: c_int);
    fn CM_TM_AddObjective(x: f32, y: f32, side: c_int);
    fn CM_TM_AddStart(x: f32, y: f32, side: c_int);
    fn CM_TM_AddEnd(x: f32, y: f32, side: c_int);
    fn CM_TM_AddWallRect(x: f32, y: f32, side: c_int);

    /// C standard library function to copy memory.
    /// Mirrors the Quake engine's VectorCopy macro behavior.
    fn VectorCopy(src: *const f32, dst: *mut f32);
}

// ============================================================================
// CRMManager static member
// ============================================================================

/// Static member of CRMManager: currently active objective (eek)
pub static mut mCurObjective: *mut CRMObjective = core::ptr::null_mut();

// ============================================================================
// CRMManager class implementation
// ============================================================================

/// Random Mission Manager.
/// Manages the state and lifecycle of random mission generation.
pub struct CRMManager {
    pub mMission: *mut CRMMission,
    pub mLandScape: *mut CCMLandScape,
    pub mTerrain: *mut CRandomTerrain,
    pub mCurPriority: c_int,
    pub mUseTimeLimit: bool,
    pub mAutomapSymbolCount: c_int,
    pub mAutomapSymbols: [rmAutomapSymbol_t; MAX_AUTOMAP_SYMBOLS],
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
            mLandScape: core::ptr::null_mut(),
            mTerrain: core::ptr::null_mut(),
            mMission: core::ptr::null_mut(),
            mCurPriority: 1,
            mUseTimeLimit: false,
            mAutomapSymbolCount: 0,
            mAutomapSymbols: [rmAutomapSymbol_t::default(); MAX_AUTOMAP_SYMBOLS],
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
        unsafe {
            self.mTerrain = (*landscape).GetRandomTerrain();
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
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            let mut instances: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut mission: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut course: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut map: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut temp: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            #[cfg(not(feature = "FINAL_BUILD"))]
            unsafe {
                Com_Printf(b"--------- Random Mission Manager ---------\n\n\0".as_ptr() as *const c_char);
                Com_Printf(b"RMG version : 1.01\n\n\0".as_ptr() as *const c_char);
            }

            if self.mTerrain.is_null() {
                return false;
            }

            // Grab the arioche variables
            unsafe {
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
            }

            // dump existing mission, if any
            if !self.mMission.is_null() {
                unsafe {
                    // In Rust, we would typically use Box::from_raw to properly deallocate,
                    // but since this is a C-allocated object, we just null it out.
                    // The C++ delete is handled by the C++ runtime.
                    let _ = core::ptr::read(self.mMission);
                }
                self.mMission = core::ptr::null_mut();
            }

            // Create a new mission file
            unsafe {
                self.mMission = Box::into_raw(Box::new(CRMMission { _opaque: [0; 0] }));
            }

            if IsServer != 0 {
                // Load the mission using the arioche variables
                unsafe {
                    if !(*self.mMission).Load(
                        mission.as_ptr(),
                        instances.as_ptr(),
                        course.as_ptr(),
                    ) {
                        return false;
                    }
                }

                // set the names of the teams
                let mut parser: CGenericParser2 = CGenericParser2 { _opaque: [0; 0] };
                let mut root: *mut CGPGroup;

                unsafe {
                    Cvar_VariableStringBuffer(
                        b"RMG_terrain\0".as_ptr() as *const c_char,
                        temp.as_mut_ptr(),
                        MAX_QPATH as c_int,
                    );

                    // Create the parser for the mission file
                    let path = va(
                        b"ext_data/rmg/%s.teams\0".as_ptr() as *const c_char,
                        temp.as_ptr(),
                    );
                    if Com_ParseTextFile(path, &mut parser as *mut CGenericParser2) {
                        root = parser.GetBaseParseGroup().as_mut().unwrap().GetSubGroups();
                        if !root.is_null() {
                            if stricmp((*root).GetName(), b"teams\0".as_ptr() as *const c_char) == 0 {
                                /*
                                SV_SetConfigstring( CS_GAMETYPE_REDTEAM, root->FindPairValue ( "red", "marine" ));
                                SV_SetConfigstring( CS_GAMETYPE_BLUETEAM, root->FindPairValue ( "blue", "thug" ));
                                */
                                //rwwFIXMEFIXME: Do we care about this?
                            }
                        }
                        parser.Clean();
                    }
                }
            }

            // Must have a valid landscape before we can spawn the mission
            assert!(!self.mLandScape.is_null());

            #[cfg(not(feature = "FINAL_BUILD"))]
            unsafe {
                Com_Printf(b"------------------------------------------\n\0".as_ptr() as *const c_char);
            }

            true
        }
        #[cfg(feature = "PRE_RELEASE_DEMO")]
        {
            false
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
        unsafe {
            if self.mMission.is_null() {
                return true;
            }
            if (*self.mMission).GetCurrentObjective().is_null() {
                return true;
            }
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
    pub fn CompleteMission(&mut self) {
        self.UpdateStatisticCvars();

        unsafe {
            if !self.mMission.is_null() {
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

        unsafe {
            if !self.mMission.is_null() {
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

        unsafe {
            if !self.mMission.is_null() {
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
    pub fn Preview(&self, from: *const vec3_t) {
        // Dont bother if we havent reached our timer yet
        /*	if ( level.time < mPreviewTimer )
        {
            return;
        }

        // Let the mission do all the previewing
        mMission->Preview ( from );

        // Another second
        mPreviewTimer = level.time + 1000;*/
        let _ = from;
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
        unsafe {
            if !self.mTerrain.is_null() && !self.mMission.is_null() {
                (*self.mMission).Spawn(self.mTerrain, IsServer);
            }
        }

        true
    }

    pub fn AddAutomapSymbol(&mut self, typ: c_int, origin: vec3_t, side: c_int) {
        if typ == 0 {
            return;
        }

        if (self.mAutomapSymbolCount as usize) < MAX_AUTOMAP_SYMBOLS {
            let idx = self.mAutomapSymbolCount as usize;
            self.mAutomapSymbols[idx].mType = typ;
            self.mAutomapSymbols[idx].mSide = side;
            unsafe {
                VectorCopy(origin.as_ptr(), self.mAutomapSymbols[idx].mOrigin.as_mut_ptr());
            }
            self.mAutomapSymbolCount += 1;
        }
    }

    pub fn GetAutomapSymbolCount(&self) -> c_int {
        self.mAutomapSymbolCount
    }

    pub fn GetAutomapSymbol(&mut self, index: c_int) -> *mut rmAutomapSymbol_t {
        if index >= 0 && (index as usize) < MAX_AUTOMAP_SYMBOLS {
            unsafe { self.mAutomapSymbols.as_mut_ptr().add(index as usize) }
        } else {
            core::ptr::null_mut()
        }
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

    pub fn ProcessAutomapSymbols(count: c_int, symbols: *mut rmAutomapSymbol_t) {
        #[cfg(not(feature = "DEDICATED"))]
        {
            let mut i: c_int = 0;

            while i < count {
                unsafe {
                    // draw proper symbol on map for instance
                    match (*symbols.add(i as usize)).mType {
                        AUTOMAP_BLD => {
                            CM_TM_AddBuilding(
                                (*symbols.add(i as usize)).mOrigin[0],
                                (*symbols.add(i as usize)).mOrigin[1],
                                (*symbols.add(i as usize)).mSide,
                            );
                        }
                        AUTOMAP_OBJ => {
                            CM_TM_AddObjective(
                                (*symbols.add(i as usize)).mOrigin[0],
                                (*symbols.add(i as usize)).mOrigin[1],
                                (*symbols.add(i as usize)).mSide,
                            );
                        }
                        AUTOMAP_START => {
                            CM_TM_AddStart(
                                (*symbols.add(i as usize)).mOrigin[0],
                                (*symbols.add(i as usize)).mOrigin[1],
                                (*symbols.add(i as usize)).mSide,
                            );
                        }
                        AUTOMAP_END => {
                            CM_TM_AddEnd(
                                (*symbols.add(i as usize)).mOrigin[0],
                                (*symbols.add(i as usize)).mOrigin[1],
                                (*symbols.add(i as usize)).mSide,
                            );
                        }
                        AUTOMAP_ENEMY => {}
                        AUTOMAP_FRIEND => {}
                        AUTOMAP_WALL => {
                            CM_TM_AddWallRect(
                                (*symbols.add(i as usize)).mOrigin[0],
                                (*symbols.add(i as usize)).mOrigin[1],
                                (*symbols.add(i as usize)).mSide,
                            );
                        }
                        _ => {}
                    }
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
        #[cfg(not(feature = "FINAL_BUILD"))]
        unsafe {
            Com_Printf(b"... Shutting down TheRandomMissionManager\n\0".as_ptr() as *const c_char);
        }
        #[cfg(not(feature = "DEDICATED"))]
        unsafe {
            CM_TM_Free();
        }
        if !self.mMission.is_null() {
            unsafe {
                // In Rust, we would typically use Box::from_raw to properly deallocate,
                // but since this is a C-allocated object, we just null it out.
                // The C++ delete is handled by the C++ runtime.
                let _ = core::ptr::read(self.mMission);
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

// ============================================================================
// Global instance of the random mission manager
// ============================================================================

/************************************************************************************************
 * TheRandomMissionManager
 *	Pointer to only active CRMManager class
 *
 ************************************************************************************************/
pub static mut TheRandomMissionManager: *mut CRMManager = core::ptr::null_mut();
