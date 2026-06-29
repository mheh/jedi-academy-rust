#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// Anything above this #include will be ignored by the compiler
use crate::codemp::qcommon::exe_headers_h::*;

/************************************************************************************************
 *
 * RM_Instance_Random.cpp
 *
 * Implements the CRMRandomInstance class.  This class is reponsible for parsing a
 * random instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

use crate::codemp::RMG::RM_Headers_h::*;

use crate::codemp::RMG::RM_Instance_Random_h::*;

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
impl CRMRandomInstance {
    pub unsafe fn new(mut instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self
    // : CRMInstance ( instGroup, instFile )
    {
        // Porting note: C++ base-class init CRMInstance(instGroup, instFile) cannot be expressed
        // directly in Rust without knowing the full struct layout; self_ is zero-initialized as the
        // closest faithful equivalent.  Full struct layout is trusted from RM_Instance_Random_h.
        let mut self_: Self = core::mem::zeroed();

        let mut group: *mut CGPGroup;
        let mut groups: [*mut CGPGroup; MAX_RANDOM_INSTANCES as usize] =
            [core::ptr::null_mut(); MAX_RANDOM_INSTANCES as usize];
        let mut numGroups: core::ffi::c_int;

        // Build a list of the groups one can be chosen
        numGroups = 0;
        group = (*instGroup).GetSubGroups();
        while !group.is_null() {
            // If this isnt an instance group then skip it
            if stricmp(
                (*group).GetName(),
                b"instance\0".as_ptr() as *const core::ffi::c_char,
            ) != 0
            {
                group = (*group).GetNext();
                continue;
            }

            let mut multiplier: core::ffi::c_int = atoi((*group).FindPairValue(
                b"multiplier\0".as_ptr() as *const core::ffi::c_char,
                b"1\0".as_ptr() as *const core::ffi::c_char,
            ));
            while multiplier > 0 && numGroups < MAX_RANDOM_INSTANCES as core::ffi::c_int {
                groups[numGroups as usize] = group;
                numGroups += 1;
                multiplier -= 1;
            }

            group = (*group).GetNext();
        }

        // No groups, no instance
        if numGroups == 0 {
            // Initialize this now
            self_.mInstance = core::ptr::null_mut();

            // Faithful null-dereference of `group` preserved from C++ original (group == NULL here)
            Com_Printf(
                b"WARNING: No sub instances specified for random instance '%s'\n\0".as_ptr()
                    as *const core::ffi::c_char,
                (*group).FindPairValue(
                    b"name\0".as_ptr() as *const core::ffi::c_char,
                    b"unknown\0".as_ptr() as *const core::ffi::c_char,
                ),
            );
            return self_;
        }

        // Now choose a group to parse
        {
            let tmm = *core::ptr::addr_of!(TheRandomMissionManager);
            let landscape = (*tmm).GetLandScape();
            instGroup = groups[(*landscape).irand(0, numGroups - 1) as usize];
        }

        // Create the child instance now.  If the instance create fails then the
        // IsValid routine will return false and this instance wont be added
        self_.mInstance = (*instFile).CreateInstance((*instGroup).FindPairValue(
            b"name\0".as_ptr() as *const core::ffi::c_char,
            b"\0".as_ptr() as *const core::ffi::c_char,
        ));
        (*self_.mInstance).SetFilter(self_.mFilter);
        (*self_.mInstance).SetTeamFilter(self_.mTeamFilter);

        self_.mAutomapSymbol = (*self_.mInstance).GetAutomapSymbol();

        self_.SetMessage((*self_.mInstance).GetMessage());
        self_.SetDescription((*self_.mInstance).GetDescription());
        self_.SetInfo((*self_.mInstance).GetInfo());

        self_
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
    pub unsafe fn dtor(&mut self) {
        if !self.mInstance.is_null() {
            // delete mInstance;
            // Porting note: C++ `delete` requires C++ allocator; translated as Box::from_raw for
            // Rust ownership semantics.  Caller must ensure mInstance was Rust-allocated or handle
            // deallocation through C++ interop instead.
            drop(Box::from_raw(self.mInstance));
        }
    }

    pub unsafe fn SetMirror(&mut self, mirror: core::ffi::c_int) {
        self.base_.SetMirror(mirror); // CRMInstance::SetMirror(mirror)
        if !self.mInstance.is_null() {
            (*self.mInstance).SetMirror(mirror);
        }
    }

    pub unsafe fn SetFilter(&mut self, filter: *const core::ffi::c_char) {
        self.base_.SetFilter(filter); // CRMInstance::SetFilter(filter)
        if !self.mInstance.is_null() {
            (*self.mInstance).SetFilter(filter);
        }
    }

    pub unsafe fn SetTeamFilter(&mut self, teamFilter: *const core::ffi::c_char) {
        self.base_.SetTeamFilter(teamFilter); // CRMInstance::SetTeamFilter(teamFilter)
        if !self.mInstance.is_null() {
            (*self.mInstance).SetTeamFilter(teamFilter);
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
    pub unsafe fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        assert!(!self.mInstance.is_null()); // assert ( mInstance );

        (*self.mInstance).SetFlattenHeight(self.GetFlattenHeight());

        (*self.mInstance).PreSpawn(terrain, IsServer)
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
    pub unsafe fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        (*self.mInstance).SetObjective(self.GetObjective());
        (*self.mInstance).SetSide(self.GetSide());

        if !(*self.mInstance).Spawn(terrain, IsServer) {
            return false;
        }

        true
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
    pub unsafe fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        self.base_.SetArea(amanager, area); // CRMInstance::SetArea ( amanager, area )

        (*self.mInstance).SetArea(amanager, self.mArea);
    }
}
