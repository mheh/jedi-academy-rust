//! Mechanical port of `codemp/RMG/RM_Instance_Random.cpp`.
//!
//! Implements the CRMRandomInstance class. This class is responsible for parsing a
//! random instance as well as spawning it into a landscape.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs used during instance construction.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::FindPairValue(const char*, const char*)`.
    /// Finds a configuration value by key, returning default if not found.
    pub fn FindPairValue(&self, _key: *const c_char, _default: *const c_char) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `CGPGroup* CGPGroup::GetSubGroups()`.
    /// Returns the first sub-group.
    pub fn GetSubGroups(&mut self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `CGPGroup* CGPGroup::GetNext()`.
    /// Returns the next sibling group.
    pub fn GetNext(&mut self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `const char* CGPGroup::GetName()`.
    /// Returns the name of this group.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }
}

/// Stub for unported `class CRMInstanceFile` (RM_InstanceFile.h).
/// Reference to an open instance file for creating sub-instances.
pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

impl CRMInstanceFile {
    /// Stub for `CRMInstance* CRMInstanceFile::CreateInstance(const char*)`.
    /// Creates a new instance of the given name.
    pub fn CreateInstance(&mut self, _name: *const c_char) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMInstance` (RM_Instance.h).
/// Base class for all procedural instances.
pub struct CRMInstance {
    _opaque: [u8; 0],
}

impl CRMInstance {
    /// Stub for `void CRMInstance::SetFilter(const char*)`.
    /// Sets the filter for this instance.
    pub fn SetFilter(&mut self, _filter: *const c_char) {
        // Porting stub
    }

    /// Stub for `void CRMInstance::SetTeamFilter(const char*)`.
    /// Sets the team filter for this instance.
    pub fn SetTeamFilter(&mut self, _teamFilter: *const c_char) {
        // Porting stub
    }

    /// Stub for `int CRMInstance::GetAutomapSymbol()`.
    /// Returns the automap symbol for this instance.
    pub fn GetAutomapSymbol(&self) -> c_int {
        0
    }

    /// Stub for `const char* CRMInstance::GetMessage()`.
    /// Returns the message for this instance.
    pub fn GetMessage(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `const char* CRMInstance::GetDescription()`.
    /// Returns the description for this instance.
    pub fn GetDescription(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `const char* CRMInstance::GetInfo()`.
    /// Returns the info for this instance.
    pub fn GetInfo(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `void CRMInstance::SetMirror(int)`.
    /// Sets the mirror flag for this instance.
    pub fn SetMirror(&mut self, _mirror: c_int) {
        // Porting stub
    }

    /// Stub for `void CRMInstance::SetFlattenHeight(int)`.
    /// Sets the flatten height for this instance.
    pub fn SetFlattenHeight(&mut self, _height: c_int) {
        // Porting stub
    }

    /// Stub for `bool CRMInstance::PreSpawn(CRandomTerrain*, qboolean)`.
    /// Prepares the instance for spawning.
    pub fn PreSpawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: c_int) -> bool {
        false
    }

    /// Stub for `void CRMInstance::SetObjective(CRMObjective*)`.
    /// Sets the objective for this instance.
    pub fn SetObjective(&mut self, _obj: *mut CRMObjective) {
        // Porting stub
    }

    /// Stub for `CRMObjective* CRMInstance::GetObjective()`.
    /// Gets the objective for this instance.
    pub fn GetObjective(&self) -> *mut CRMObjective {
        core::ptr::null_mut()
    }

    /// Stub for `int CRMInstance::GetSide()`.
    /// Returns the side this instance belongs to.
    pub fn GetSide(&self) -> c_int {
        0
    }

    /// Stub for `void CRMInstance::SetSide(int)`.
    /// Sets the side for this instance.
    pub fn SetSide(&mut self, _side: c_int) {
        // Porting stub
    }

    /// Stub for `bool CRMInstance::Spawn(CRandomTerrain*, qboolean)`.
    /// Spawns the instance into the landscape.
    pub fn Spawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: c_int) -> bool {
        false
    }

    /// Stub for `void CRMInstance::SetArea(CRMAreaManager*, CRMArea*)`.
    /// Sets the area for this instance.
    pub fn SetArea(&mut self, _amanager: *mut CRMAreaManager, _area: *mut CRMArea) {
        // Porting stub
    }
}

/// Stub for unported `class CRMAreaManager` (RM_Area.h).
/// Manages multiple areas in the map.
pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMArea` (RM_Area.h).
/// Represents an area in the map.
pub struct CRMArea {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
/// Represents the random terrain generation system.
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
/// Represents the terrain landscape with height map and flattening.
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// Stub for `int CCMLandScape::irand(int, int)`.
    /// Returns a random integer between min and max.
    pub fn irand(&self, _min: c_int, _max: c_int) -> c_int {
        0
    }
}

/// Stub for unported `class CRandomMissionManager` (RM_Manager.h).
/// Global manager for random mission generation.
pub struct CRandomMissionManager {
    _opaque: [u8; 0],
}

impl CRandomMissionManager {
    /// Stub for `CCMLandScape* CRandomMissionManager::GetLandScape()`.
    /// Returns the landscape being managed.
    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMObjective` (RM_Objective.h).
/// Represents a mission objective.
pub struct CRMObjective {
    _opaque: [u8; 0],
}

// ============================================================================
// CONSTANTS
// ============================================================================

pub const MAX_RANDOM_INSTANCES: usize = 64;

// ============================================================================
// GLOBAL REFERENCES
// ============================================================================

/// Global pointer to the random mission manager.
/// Original C declaration: `extern CRandomMissionManager* TheRandomMissionManager;`
extern "C" {
    pub static mut TheRandomMissionManager: *mut CRandomMissionManager;
}

// ============================================================================
// extern "C" functions from libc and engine
// ============================================================================

extern "C" {
    /// C standard library function to convert string to integer.
    fn atoi(nptr: *const c_char) -> c_int;

    /// Quake engine function for case-insensitive string comparison.
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Quake engine function for formatted output.
    fn Com_Printf(fmt: *const c_char, ...);
}

// ============================================================================
// CRMRandomInstance - Random Instance class
// ============================================================================

/// Represents a random instance in the random map generation system.
#[repr(C)]
pub struct CRMRandomInstance {
    pub mInstance: *mut CRMInstance,
}

impl CRMRandomInstance {
    /// CRMRandomInstance::CRMRandomInstance
    /// constructs a random instance by choosing one of the sub instances and creating it
    ///
    /// inputs:
    ///  instGroup:  parser group containing information about this instance
    ///  instFile:   reference to an open instance file for creating sub instances
    ///
    /// return:
    ///  none
    pub fn new(instGroup: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self {
        let mut result = CRMRandomInstance {
            mInstance: core::ptr::null_mut(),
        };

        if instGroup.is_null() {
            return result;
        }

        // Safety: instGroup is checked for null above
        unsafe {
            let mut groups: [*mut CGPGroup; MAX_RANDOM_INSTANCES] = [core::ptr::null_mut(); MAX_RANDOM_INSTANCES];
            let mut numGroups: usize = 0;

            // Build a list of the groups one can be chosen
            let mut group = (*instGroup).GetSubGroups();

            while !group.is_null() {
                // If this isn't an instance group then skip it
                if 0 != stricmp((*group).GetName(), b"instance\0".as_ptr() as *const c_char) {
                    group = (*group).GetNext();
                    continue;
                }

                let multiplier_str = (*group).FindPairValue(b"multiplier\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
                let mut multiplier = atoi(multiplier_str);

                while multiplier > 0 && numGroups < MAX_RANDOM_INSTANCES {
                    groups[numGroups] = group;
                    numGroups += 1;
                    multiplier -= 1;
                }

                group = (*group).GetNext();
            }

            // No groups, no instance
            if numGroups == 0 {
                // Initialize this now
                result.mInstance = core::ptr::null_mut();

                Com_Printf(
                    b"WARNING: No sub instances specified for random instance '%s'\n\0".as_ptr() as *const c_char,
                    (*instGroup).FindPairValue(b"name\0".as_ptr() as *const c_char, b"unknown\0".as_ptr() as *const c_char),
                );

                return result;
            }

            // Now choose a group to parse
            let chosen_group = if !TheRandomMissionManager.is_null() {
                let mgr = &*TheRandomMissionManager;
                let landscape = mgr.GetLandScape();
                if !landscape.is_null() {
                    let rand_index = (*landscape).irand(0, numGroups as c_int - 1) as usize;
                    groups[rand_index]
                } else {
                    groups[0]
                }
            } else {
                groups[0]
            };

            // Create the child instance now. If the instance create fails then the
            // IsValid routine will return false and this instance won't be added
            let instance_name = (*chosen_group).FindPairValue(b"name\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
            result.mInstance = instFile.CreateInstance(instance_name);

            if !result.mInstance.is_null() {
                let instance = &mut *result.mInstance;
                instance.SetFilter(b"\0".as_ptr() as *const c_char);
                instance.SetTeamFilter(b"\0".as_ptr() as *const c_char);
            }
        }

        result
    }

    /// CRMRandomInstance::~CRMRandomInstance
    /// Deletes the sub instance
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///  none
    pub fn delete(&mut self) {
        // Note: In Rust, we don't manually delete. The struct will be dropped automatically.
        // If mInstance was allocated with C++ new, it needs to be deleted through C++ code.
        // This is a porting stub - actual deletion would happen through C++ interop.
        if !self.mInstance.is_null() {
            // C++ delete would happen here
            self.mInstance = core::ptr::null_mut();
        }
    }

    /// CRMRandomInstance::SetMirror
    pub fn SetMirror(&mut self, mirror: c_int) {
        // Call parent SetMirror
        // CRMInstance::SetMirror(mirror);

        if !self.mInstance.is_null() {
            unsafe {
                (*self.mInstance).SetMirror(mirror);
            }
        }
    }

    /// CRMRandomInstance::SetFilter
    pub fn SetFilter(&mut self, filter: *const c_char) {
        // Call parent SetFilter
        // CRMInstance::SetFilter(filter);

        if !self.mInstance.is_null() {
            unsafe {
                (*self.mInstance).SetFilter(filter);
            }
        }
    }

    /// CRMRandomInstance::SetTeamFilter
    pub fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        // Call parent SetTeamFilter
        // CRMInstance::SetTeamFilter(teamFilter);

        if !self.mInstance.is_null() {
            unsafe {
                (*self.mInstance).SetTeamFilter(teamFilter);
            }
        }
    }

    /// CRMRandomInstance::PreSpawn
    /// Prepares for the spawn of the random instance
    ///
    /// inputs:
    ///  terrain: terrain object this instance will be spawned on
    ///
    /// return:
    ///  true: preparation successful
    ///  false: preparation failed
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        // assert ( mInstance );

        if self.mInstance.is_null() {
            return false;
        }

        // Safety: mInstance is checked for null above
        unsafe {
            let instance = &mut *self.mInstance;

            // Note: GetFlattenHeight() is a method on the parent class CRMInstance
            // which would need to be accessible through inheritance or composition.
            // For now, we assume a flatten height of 0 as a stub.
            instance.SetFlattenHeight(0);

            return instance.PreSpawn(terrain, IsServer);
        }
    }

    /// CRMRandomInstance::Spawn
    /// Spawns the instance onto the terrain
    ///
    /// inputs:
    ///  terrain: terrain object this instance will be spawned on
    ///
    /// return:
    ///  true: spawn successful
    ///  false: spawn failed
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        if self.mInstance.is_null() {
            return false;
        }

        // Safety: mInstance is checked for null above
        unsafe {
            let instance = &mut *self.mInstance;

            // Note: GetObjective() and GetSide() are methods on the parent class CRMInstance.
            // For now, we use stub implementations.
            instance.SetObjective(core::ptr::null_mut());
            instance.SetSide(0);

            if !instance.Spawn(terrain, IsServer) {
                return false;
            }
        }

        true
    }

    /// CRMRandomInstance::SetArea
    /// Forwards the given area off to the internal instance
    ///
    /// inputs:
    ///  amanager: area manager
    ///  area: area to be set
    ///
    /// return:
    ///  none
    pub fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        // Call parent SetArea
        // CRMInstance::SetArea ( amanager, area );

        if !self.mInstance.is_null() {
            unsafe {
                (*self.mInstance).SetArea(amanager, area);
            }
        }
    }
}
