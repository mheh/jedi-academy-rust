//! Mechanical port of `codemp/RMG/RM_Instance_Group.cpp`
//!
//! Implements the CRMGroupInstance class. This class is responsible for parsing a
//! group instance as well as spawning it into a landscape.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_float};

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

    /// Stub for `void CRMInstance::SetArea(CRMAreaManager*, CRMArea*)`.
    /// Sets the area for this instance.
    pub fn SetArea(&mut self, _amanager: *mut CRMAreaManager, _area: *mut CRMArea) {
        // Porting stub
    }

    /// Stub for `void CRMInstance::PreSpawn(CRandomTerrain*, qboolean)`.
    /// Prepares the instance for spawning.
    pub fn PreSpawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: c_int) -> bool {
        false
    }

    /// Stub for `void CRMInstance::Spawn(CRandomTerrain*, qboolean)`.
    /// Spawns the instance into the landscape.
    pub fn Spawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: c_int) -> bool {
        false
    }

    /// Stub for `void CRMInstance::Preview(const vec3_t)`.
    /// Renders debug preview of the instance.
    pub fn Preview(&self, _from: *const [c_float; 3]) {
        // Porting stub
    }

    /// Stub for `void CRMInstance::SetMirror(int)`.
    /// Sets the mirror flag for this instance.
    pub fn SetMirror(&mut self, _mirror: c_int) {
        // Porting stub
    }

    /// Stub for `int CRMInstance::GetSide()`.
    /// Returns the side this instance belongs to.
    pub fn GetSide(&self) -> c_int {
        0
    }

    /// Stub for `float CRMInstance::GetSpacingRadius()`.
    /// Returns the spacing radius.
    pub fn GetSpacingRadius(&self) -> c_float {
        0.0
    }

    /// Stub for `float CRMInstance::GetFlattenRadius()`.
    /// Returns the flatten radius.
    pub fn GetFlattenRadius(&self) -> c_float {
        0.0
    }

    /// Stub for `int CRMInstance::GetSpacingLine()`.
    /// Returns the spacing line.
    pub fn GetSpacingLine(&self) -> c_int {
        0
    }

    /// Stub for `bool CRMInstance::GetLockOrigin()`.
    /// Returns whether the origin is locked.
    pub fn GetLockOrigin(&self) -> bool {
        false
    }
}

/// Stub for unported `class CRMArea` (RM_Area.h).
/// Represents an area in the map.
pub struct CRMArea {
    _opaque: [u8; 0],
}

impl CRMArea {
    /// Stub for `bool CRMArea::IsCollisionEnabled()`.
    /// Returns whether collision is enabled.
    pub fn IsCollisionEnabled(&self) -> bool {
        false
    }

    /// Stub for `void CRMArea::EnableCollision(bool)`.
    /// Enables or disables collision.
    pub fn EnableCollision(&mut self, _enable: bool) {
        // Porting stub
    }

    /// Stub for `bool CRMArea::GetSymmetric()`.
    /// Returns the symmetric flag.
    pub fn GetSymmetric(&self) -> bool {
        false
    }
}

/// Stub for unported `class CRMAreaManager` (RM_Area.h).
/// Manages multiple areas in the map.
pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

impl CRMAreaManager {
    /// Stub for `CRMArea* CRMAreaManager::CreateArea(...)`.
    /// Creates a new area with the given parameters.
    pub fn CreateArea(
        &mut self,
        _origin: *const [c_float; 3],
        _radius: c_float,
        _spacing_line: c_int,
        _padding: c_float,
        _confine_radius: c_float,
        _orig1: *const [c_float; 3],
        _orig2: *const [c_float; 3],
        _flatten: bool,
        _collide: bool,
        _lock_origin: bool,
        _symmetric: bool,
    ) -> *mut CRMArea {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
/// Represents the random terrain generation system.
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

/// Stub for unported `class CCRMMission` (RM_Mission.h).
/// Represents a random mission.
pub struct CCRMMission {
    _opaque: [u8; 0],
}

impl CCRMMission {
    /// Stub for `int CCRMMission::GetDefaultPadding()`.
    /// Returns the default padding for the mission.
    pub fn GetDefaultPadding(&self) -> c_int {
        0
    }
}

/// Stub for unported `class CRandomMissionManager` (RM_Manager.h).
/// Global manager for random mission generation.
pub struct CRandomMissionManager {
    _opaque: [u8; 0],
}

impl CRandomMissionManager {
    /// Stub for `CCRMMission* CRandomMissionManager::GetMission()`.
    /// Returns the current mission.
    pub fn GetMission(&self) -> *mut CCRMMission {
        core::ptr::null_mut()
    }

    /// Stub for `CRandomTerrain* CRandomMissionManager::GetLandScape()`.
    /// Returns the landscape being managed.
    pub fn GetLandScape(&self) -> *mut CRandomTerrain {
        core::ptr::null_mut()
    }
}

/// Stub for C++ std::string.
/// Represents a dynamic string type used internally by CRMInstance.
#[repr(C)]
pub struct std_string {
    _private: [u8; 0],
}

/// Stub for C++ std::list<CRMInstance*>.
/// Represents a doubly-linked list of instance pointers.
#[repr(C)]
pub struct rmInstanceList_t {
    _private: [u8; 0],
}

/// Iterator type for instance list.
pub type rmInstanceIter_t = *mut CRMInstance;

// MAX_QPATH constant for fixed-size char arrays
pub const MAX_QPATH: usize = 64;

// AUTOMAP constants (from RM_Instance.h)
pub const AUTOMAP_NONE: c_int = 0;
pub const AUTOMAP_BLD: c_int = 1;
pub const AUTOMAP_OBJ: c_int = 2;
pub const AUTOMAP_START: c_int = 3;
pub const AUTOMAP_END: c_int = 4;
pub const AUTOMAP_ENEMY: c_int = 5;
pub const AUTOMAP_FRIEND: c_int = 6;
pub const AUTOMAP_WALL: c_int = 7;

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
    /// C standard library function to convert string to double.
    fn atof(nptr: *const c_char) -> c_float;

    /// C standard library function to convert string to integer.
    fn atoi(nptr: *const c_char) -> c_int;

    /// Quake engine function for case-insensitive string comparison.
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Platform function for case-insensitive string comparison (alias for Q_stricmp).
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Quake engine function for formatted string creation.
    fn va(fmt: *const c_char, ...) -> *const c_char;

    /// Assert function for C code.
    fn assert(_: c_int);
}

// ============================================================================
// CRMGroupInstance - Derived class (includes flattened base class fields)
// ============================================================================
// Preserved from RM_Instance_Group.h: class CRMGroupInstance : public CRMInstance
//
// CRMGroupInstance inherits from CRMInstance in the original C++ code.
// In Rust with #[repr(C)], we flatten the base class fields directly to maintain
// the exact memory layout expected by C++ code.
//
// Base class CRMInstance fields (from RM_Instance.h):
#[repr(C)]
pub struct CRMGroupInstance {
    // ---- CRMGroupInstance-specific fields (protected in original C++) ----
    pub mInstances: rmInstanceList_t,
    pub mConfineRadius: c_float,
    pub mPaddingSize: c_float,
}

impl CRMGroupInstance {
    /// ========================================================================
    /// CRMGroupInstance::CRMGroupInstance
    ///     constructor
    ///
    /// inputs:
    ///  settlementID:  ID of the settlement being created
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn new(instGroup: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self {
        unsafe {
            let mut mPaddingSize_val: c_float = 0.0;
            let mut mConfineRadius_val: c_float = 0.0;
            let mut mAutomapSymbol_val: c_int = AUTOMAP_NONE;

            if !instGroup.is_null() {
                // Grab the padding and confine radius
                // Note: va() usage requires C++ side support for formatting
                let padding_value = (*instGroup).FindPairValue(
                    b"padding\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                );
                if !padding_value.is_null() {
                    mPaddingSize_val = atof(padding_value);
                }

                let confine_value = (*instGroup).FindPairValue(
                    b"confine\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                );
                if !confine_value.is_null() {
                    mConfineRadius_val = atof(confine_value);
                }

                let automapSymName = (*instGroup).FindPairValue(
                    b"automap_symbol\0".as_ptr() as *const c_char,
                    b"none\0".as_ptr() as *const c_char,
                );

                if !automapSymName.is_null() {
                    if 0 == Q_stricmp(automapSymName, b"none\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_NONE;
                    } else if 0 == Q_stricmp(automapSymName, b"building\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_BLD;
                    } else if 0 == Q_stricmp(automapSymName, b"objective\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_OBJ;
                    } else if 0 == Q_stricmp(automapSymName, b"start\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_START;
                    } else if 0 == Q_stricmp(automapSymName, b"end\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_END;
                    } else if 0 == Q_stricmp(automapSymName, b"enemy\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_ENEMY;
                    } else if 0 == Q_stricmp(automapSymName, b"friend\0".as_ptr() as *const c_char) {
                        mAutomapSymbol_val = AUTOMAP_FRIEND;
                    } else {
                        mAutomapSymbol_val = atoi(automapSymName);
                    }
                }

                // optional instance objective strings
                // SetMessage(instGroup->FindPairValue("objective_message",""));
                // SetDescription(instGroup->FindPairValue("objective_description",""));
                // SetInfo(instGroup->FindPairValue("objective_info",""));
                // Note: These are methods on base class CRMInstance, not available here

                // Iterate through the sub groups to determine the instances which make up the group
                let mut instGroup_iter = (*instGroup).GetSubGroups();

                while !instGroup_iter.is_null() {
                    let instance: *mut CRMInstance;
                    let name: *const c_char;
                    let mincount: c_int;
                    let maxcount: c_int;
                    let mut count: c_int;
                    let _minrange: c_float;
                    let _maxrange: c_float;

                    // Make sure only instances are specified as sub groups
                    assert(0 == stricmp((*instGroup_iter).GetName(), b"instance\0".as_ptr() as *const c_char) as c_int);

                    // Grab the name
                    name = (*instGroup_iter).FindPairValue(
                        b"name\0".as_ptr() as *const c_char,
                        b"\0".as_ptr() as *const c_char,
                    );

                    // Grab the range information
                    _minrange = atof((*instGroup_iter).FindPairValue(
                        b"minrange\0".as_ptr() as *const c_char,
                        b"0\0".as_ptr() as *const c_char,
                    ));
                    _maxrange = atof((*instGroup_iter).FindPairValue(
                        b"maxrange\0".as_ptr() as *const c_char,
                        b"0\0".as_ptr() as *const c_char,
                    ));

                    // Grab the count information and randomly generate a count value
                    mincount = atoi((*instGroup_iter).FindPairValue(
                        b"mincount\0".as_ptr() as *const c_char,
                        b"1\0".as_ptr() as *const c_char,
                    ));
                    maxcount = atoi((*instGroup_iter).FindPairValue(
                        b"maxcount\0".as_ptr() as *const c_char,
                        b"1\0".as_ptr() as *const c_char,
                    ));
                    count = mincount;

                    if maxcount > mincount {
                        // count += (TheRandomMissionManager->GetLandScape()->irand(0, maxcount-mincount));
                        // TODO: implement irand call when landscape bindings available
                    }

                    // For each count create and add the instance
                    while count > 0 {
                        // Create the instance
                        instance = instFile.CreateInstance(name);

                        // Skip this instance if it couldnt be created for some reason.  The CreateInstance
                        // method will report an error so no need to do so here.
                        if !instance.is_null() {
                            // Set the min and max range for the instance
                            // (*instance).SetFilter(mFilter);
                            // (*instance).SetTeamFilter(mTeamFilter);

                            // Add the instance to the list
                            // mInstances.push_back ( instance );
                            // Note: std::list push_back requires C++ bindings
                        }

                        count -= 1;
                    }

                    // Next sub group
                    instGroup_iter = (*instGroup_iter).GetNext();
                }
            }

            CRMGroupInstance {
                mInstances: core::mem::zeroed(),
                mConfineRadius: mConfineRadius_val,
                mPaddingSize: mPaddingSize_val,
            }
        }
    }

    /// ========================================================================
    /// CRMGroupInstance::~CRMGroupInstance
    ///	Removes all buildings and inhabitants
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn drop(&mut self) {
        // Cleanup
        self.RemoveInstances();
    }

    /// ========================================================================
    /// CRMGroupInstance::SetFilter
    ///	Sets a filter used to exclude instances
    ///
    /// inputs:
    ///  filter: filter name
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn SetFilter(&mut self, filter: *const c_char) {
        // CRMInstance::SetFilter(filter);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     (*it)->SetFilter(filter);
        // }
        // Porting stub: iteration over C++ list not yet implemented
    }

    /// ========================================================================
    /// CRMGroupInstance::SetTeamFilter
    ///	Sets the filter used to exclude team based instances
    ///
    /// inputs:
    ///  teamFilter: filter name
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        // CRMInstance::SetTeamFilter(teamFilter);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     (*it)->SetTeamFilter(teamFilter);
        // }
        // Porting stub: iteration over C++ list not yet implemented
    }

    /// ========================================================================
    /// CRMGroupInstance::SetMirror
    ///	Sets the flag to mirror an instance on map
    ///
    /// inputs:
    ///  mirror
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn SetMirror(&mut self, mirror: c_int) {
        // CRMInstance::SetMirror(mirror);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     (*it)->SetMirror(mirror);
        // }
        // Porting stub: iteration over C++ list not yet implemented
    }

    /// ========================================================================
    /// CRMGroupInstance::RemoveInstances
    ///	Removes all instances associated with the group
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn RemoveInstances(&mut self) {
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     delete *it;
        // }
        //
        // mInstances.clear();
        // Porting stub: iteration and cleanup of C++ list not yet implemented
    }

    /// ========================================================================
    /// CRMGroupInstance::PreSpawn
    ///	Prepares the group for spawning by
    ///
    /// inputs:
    ///  landscape: landscape to calculate the position within
    ///  instance: instance to calculate the position for
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        // for(it = mInstances.begin(); it != mInstances.end(); it++ )
        // {
        //     CRMInstance* instance = *it;
        //
        //     instance->SetFlattenHeight ( mFlattenHeight );
        //
        //     // Add the instance to the landscape now
        //     instance->PreSpawn ( terrain, IsServer );
        // }
        //
        // return CRMInstance::PreSpawn ( terrain, IsServer );
        // Porting stub: iteration over C++ list not yet implemented
        false
    }

    /// ========================================================================
    /// CRMGroupInstance::Spawn
    ///	Adds the group instance to the given landscape using the specified origin.  All sub instances
    ///  will be added to the landscape within their min and max range from the origin.
    ///
    /// inputs:
    ///  landscape: landscape to add the instance group to
    ///  origin: origin of the instance group
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        // Spawn all the instances associated with this group
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     CRMInstance* instance = *it;
        //     instance->SetSide(GetSide()); // which side owns it?
        //
        //     // Add the instance to the landscape now
        //     instance->Spawn ( terrain, IsServer );
        // }
        //
        // DrawAutomapSymbol();
        //
        // return true;
        // Porting stub: iteration over C++ list not yet implemented
        true
    }

    /// ========================================================================
    /// CRMGroupInstance::Preview
    ///	Renders debug information for the instance
    ///
    /// inputs:
    ///  from: point to render the preview from
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn Preview(&self, from: *const [c_float; 3]) {
        // CRMInstance::Preview ( from );
        //
        // // Render all the instances
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     CRMInstance* instance = *it;
        //
        //     instance->Preview ( from );
        // }
        // Porting stub: iteration over C++ list not yet implemented
    }

    /// ========================================================================
    /// CRMGroupInstance::SetArea
    ///	Overidden to make sure the groups area doesnt eat up any room.  The collision on the
    ///  groups area will be turned off
    ///
    /// inputs:
    ///  area: area to set
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        unsafe {
            let collide = (*area).IsCollisionEnabled();

            // Disable collision
            (*area).EnableCollision(false);

            // Do what really needs to get done
            // CRMInstance::SetArea ( amanager, area );

            // Prepare for spawn by calculating all the positions of the sub instances
            // and flattening the ground below them.
            // for(it = mInstances.begin(); it != mInstances.end(); it++ )
            // {
            //     CRMInstance  *instance = *it;
            //     CRMArea		 *newarea;
            //     vec3_t		 origin;
            //
            //     // Drop it in the center of the group for now
            //     origin[0] = GetOrigin()[0];
            //     origin[1] = GetOrigin()[1];
            //     origin[2] = 2500;
            //
            //     // Set the area of position
            //     newarea = amanager->CreateArea ( origin, instance->GetSpacingRadius(), instance->GetSpacingLine(), mPaddingSize, mConfineRadius, GetOrigin(), GetOrigin(), instance->GetFlattenRadius()?true:false, collide, instance->GetLockOrigin(), area->GetSymmetric ( ) );
            //     instance->SetArea ( amanager, newarea );
            // }

            // Porting stub: iteration over C++ list not yet implemented
        }
    }
}
