#![allow(non_snake_case)]

// Preserved includes documentation from oracle/code/RMG/RM_Instance_Random.cpp:
// #include "../server/exe_headers.h"
// #include "rm_headers.h"
// #include "rm_instance_random.h"

/************************************************************************************************
 *
 * RM_Instance_Random.cpp
 *
 * Implements the CRMRandomInstance class.  This class is reponsible for parsing a
 * random instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

use core::ffi::{c_char, c_int};
use std::ptr;

use super::RM_Instance_Random_h::*;

// LOCAL STUB: Forward declarations for types used in this file
pub struct CRMInstance;
pub struct CRandomTerrain;
pub struct CRMAreaManager;
pub struct CRMArea;

extern "C" {
    /// stricmp - Case-insensitive string comparison
    /// int stricmp(const char *s1, const char *s2);
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// atoi - Convert C string to integer
    /// int atoi(const char *s);
    fn atoi(s: *const c_char) -> c_int;

    /// Com_Printf - Print to console
    /// void Com_Printf(const char *fmt, ...);
    fn Com_Printf(fmt: *const c_char, ...) -> ();

    /// CGPGroup::GetSubGroups
    fn CGPGroup_GetSubGroups(this: *mut CGPGroup) -> *mut CGPGroup;

    /// CGPGroup::GetNext
    fn CGPGroup_GetNext(this: *mut CGPGroup) -> *mut CGPGroup;

    /// CGPGroup::GetName
    fn CGPGroup_GetName(this: *mut CGPGroup) -> *const c_char;

    /// CGPGroup::FindPairValue
    fn CGPGroup_FindPairValue(
        this: *mut CGPGroup,
        key: *const c_char,
        default: *const c_char,
    ) -> *const c_char;

    /// CRMInstanceFile::CreateInstance
    fn CRMInstanceFile_CreateInstance(
        this: *mut CRMInstanceFile,
        name: *const c_char,
    ) -> *mut CRMInstance;

    /// CRMInstance::SetFilter
    fn CRMInstance_SetFilter(this: *mut CRMInstance, filter: *const c_char);

    /// CRMInstance::SetTeamFilter
    fn CRMInstance_SetTeamFilter(this: *mut CRMInstance, teamFilter: *const c_char);

    /// CRMInstance::GetAutomapSymbol
    fn CRMInstance_GetAutomapSymbol(this: *mut CRMInstance);

    /// CRMInstance::SetMirror
    fn CRMInstance_SetMirror(this: *mut CRMInstance, mirror: c_int);

    /// CRMInstance::SetFilter (parent class)
    fn CRMInstance_SetFilter_parent(this: *mut CRMInstance, filter: *const c_char);

    /// CRMInstance::SetTeamFilter (parent class)
    fn CRMInstance_SetTeamFilter_parent(this: *mut CRMInstance, teamFilter: *const c_char);

    /// CRMInstance::SetMirror (parent class)
    fn CRMInstance_SetMirror_parent(this: *mut CRMInstance, mirror: c_int);

    /// CRMInstance::SetFlattenHeight
    fn CRMInstance_SetFlattenHeight(this: *mut CRMInstance, height: f32);

    /// CRMInstance::PreSpawn
    fn CRMInstance_PreSpawn(
        this: *mut CRMInstance,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool;

    /// CRMInstance::GetFlattenHeight
    fn CRMInstance_GetFlattenHeight(this: *mut CRMInstance) -> f32;

    /// CRMInstance::SetObjective
    fn CRMInstance_SetObjective(this: *mut CRMInstance);

    /// CRMInstance::SetSide
    fn CRMInstance_SetSide(this: *mut CRMInstance);

    /// CRMInstance::GetObjective
    fn CRMInstance_GetObjective(this: *mut CRMInstance);

    /// CRMInstance::GetSide
    fn CRMInstance_GetSide(this: *mut CRMInstance);

    /// CRMInstance::Spawn
    fn CRMInstance_Spawn(
        this: *mut CRMInstance,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool;

    /// CRMInstance::SetArea
    fn CRMInstance_SetArea(
        this: *mut CRMInstance,
        amanager: *mut CRMAreaManager,
        area: *mut CRMArea,
    );

    /// CRandomTerrain::irand
    fn CRandomTerrain_irand(this: *mut CRandomTerrain, min: c_int, max: c_int) -> c_int;

    /// TheRandomMissionManager global
    static TheRandomMissionManager: *const ();

    /// TheRandomMissionManager::GetLandScape
    fn TheRandomMissionManager_GetLandScape(this: *const ()) -> *mut CRandomTerrain;
}

impl CRMRandomInstance {
    /************************************************************************************************
     * CRMRandomInstance::CRMRandomInstance
     *	constructs a random instance by choosing one of the sub instances and creating it
     *
     * inputs:
     *  instGroup:  parser group containing infromation about this instance
     *  instFile:   reference to an open instance file for creating sub instances
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        unsafe {
            let mut groups: [*mut CGPGroup; MAX_RANDOM_INSTANCES] =
                [ptr::null_mut(); MAX_RANDOM_INSTANCES];
            let mut numGroups: c_int = 0;
            let mut group: *mut CGPGroup;
            let mut instGroup_mut = instGroup;

            // Build a list of the groups one can be chosen
            group = CGPGroup_GetSubGroups(instGroup);
            loop {
                if group.is_null() {
                    break;
                }

                // If this isnt an instance group then skip it
                if stricmp(
                    CGPGroup_GetName(group),
                    b"instance\0".as_ptr() as *const c_char,
                ) != 0
                {
                    group = CGPGroup_GetNext(group);
                    continue;
                }

                let mut multiplier = atoi(CGPGroup_FindPairValue(
                    group,
                    b"multiplier\0".as_ptr() as *const c_char,
                    b"1\0".as_ptr() as *const c_char,
                ));
                while multiplier > 0 && numGroups < MAX_RANDOM_INSTANCES as c_int {
                    groups[numGroups as usize] = group;
                    numGroups += 1;
                    multiplier -= 1;
                }

                group = CGPGroup_GetNext(group);
            }

            // No groups, no instance
            if numGroups == 0 {
                // Initialize this now
                Com_Printf(
                    b"WARNING: No sub instances specified for random instance '%s'\n\0"
                        .as_ptr() as *const c_char,
                    CGPGroup_FindPairValue(
                        group,
                        b"name\0".as_ptr() as *const c_char,
                        b"unknown\0".as_ptr() as *const c_char,
                    ),
                );
                return CRMRandomInstance {
                    mInstance: ptr::null_mut(),
                };
            }

            // Now choose a group to parse
            let landscape = TheRandomMissionManager_GetLandScape(TheRandomMissionManager);
            let random_index =
                CRandomTerrain_irand(landscape, 0, numGroups - 1) as usize;
            instGroup_mut = groups[random_index];

            // Create the child instance now.  If the instance create fails then the
            // IsValid routine will return false and this instance wont be added
            let mInstance = CRMInstanceFile_CreateInstance(
                instFile,
                CGPGroup_FindPairValue(
                    instGroup_mut,
                    b"name\0".as_ptr() as *const c_char,
                    b"\0".as_ptr() as *const c_char,
                ),
            );
            CRMInstance_SetFilter(mInstance, ptr::null());
            CRMInstance_SetTeamFilter(mInstance, ptr::null());

            CRMInstance_GetAutomapSymbol(mInstance);

            CRMRandomInstance { mInstance }
        }
    }
}

/************************************************************************************************
 * CRMRandomInstance::~CRMRandomInstance
 *	Deletes the sub instance
 *
 * inputs:
 *  none
 *
 * return:
 *	none
 *
 ************************************************************************************************/
impl Drop for CRMRandomInstance {
    fn drop(&mut self) {
        if !self.mInstance.is_null() {
            unsafe {
                // delete mInstance
                let _ = Box::from_raw(self.mInstance);
            }
        }
    }
}

impl CRMRandomInstance {
    pub fn SetMirror(&mut self, mirror: c_int) {
        unsafe {
            CRMInstance_SetMirror_parent(
                self as *mut CRMRandomInstance as *mut CRMInstance,
                mirror,
            );
            if !self.mInstance.is_null() {
                CRMInstance_SetMirror(self.mInstance, mirror);
            }
        }
    }

    pub fn SetFilter(&mut self, filter: *const c_char) {
        unsafe {
            CRMInstance_SetFilter_parent(
                self as *mut CRMRandomInstance as *mut CRMInstance,
                filter,
            );
            if !self.mInstance.is_null() {
                CRMInstance_SetFilter(self.mInstance, filter);
            }
        }
    }

    pub fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        unsafe {
            CRMInstance_SetTeamFilter_parent(
                self as *mut CRMRandomInstance as *mut CRMInstance,
                teamFilter,
            );
            if !self.mInstance.is_null() {
                CRMInstance_SetTeamFilter(self.mInstance, teamFilter);
            }
        }
    }

    /************************************************************************************************
     * CRMRandomInstance::PreSpawn
     *	Prepares for the spawn of the random instance
     *
     * inputs:
     *  landscape: landscape object this instance will be spawned on
     *
     * return:
     *	true: preparation successful
     *  false: preparation failed
     *
     ************************************************************************************************/
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            assert!(!self.mInstance.is_null());

            let flatten_height = CRMInstance_GetFlattenHeight(
                self as *mut CRMRandomInstance as *mut CRMInstance
            );
            CRMInstance_SetFlattenHeight(self.mInstance, flatten_height);

            CRMInstance_PreSpawn(self.mInstance, terrain, IsServer)
        }
    }

    /************************************************************************************************
     * CRMRandomInstance::Spawn
     *	Spawns the instance onto the landscape
     *
     * inputs:
     *  landscape: landscape object this instance will be spawned on
     *
     * return:
     *	true: spawn successful
     *  false: spawn failed
     *
     ************************************************************************************************/
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            CRMInstance_SetObjective(self.mInstance);
            CRMInstance_SetSide(self.mInstance);

            if !CRMInstance_Spawn(self.mInstance, terrain, IsServer) {
                return false;
            }

            true
        }
    }

    /************************************************************************************************
     * CRMRandomInstance::SetArea
     *	Forwards the given area off to the internal instance
     *
     * inputs:
     *  area: area to be set
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        unsafe {
            CRMInstance_SetArea(
                self as *mut CRMRandomInstance as *mut CRMInstance,
                amanager,
                area,
            );

            CRMInstance_SetArea(self.mInstance, amanager, area);
        }
    }
}
