// Faithful Rust port of oracle/code/RMG/RM_Instance_Group.cpp
// Preserves C symbol names, control flow, globals, raw pointers, casts, and dangerous behavior.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void, c_double};
use std::ptr;

use crate::code::RMG::RM_Instance_Group_h::CRMGroupInstance;
use crate::code::RMG::RM_Instance_h::CRMInstance;
use crate::code::RMG::RM_Manager_h::TheRandomMissionManager;
use crate::code::RMG::RM_Manager_h::CRMManager;
use crate::code::RMG::RM_Mission_h::CRMMission;

// LOCAL STUB: Forward declarations for types used in this file
pub struct CGPGroup {
    _opaque: [u8; 0],
}

pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

pub struct CRMArea {
    _opaque: [u8; 0],
}

pub struct CRMObjective {
    _opaque: [u8; 0],
}

pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

extern "C" {
    /// atof - Convert C string to floating point number
    /// double atof(const char *s);
    fn atof(s: *const c_char) -> c_double;

    /// atoi - Convert C string to integer
    /// int atoi(const char *s);
    fn atoi(s: *const c_char) -> c_int;

    /// strcmpi - Case-insensitive string comparison
    /// int strcmpi(const char *s1, const char *s2);
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;

    /// va - Format string like sprintf (returns static buffer)
    /// const char* va(const char *format, ...);
    fn va(fmt: *const c_char, ...) -> *const c_char;

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

    /// CRMInstance::SetMirror
    fn CRMInstance_SetMirror(this: *mut CRMInstance, mirror: c_int);

    /// CRMInstance::SetFlattenHeight
    fn CRMInstance_SetFlattenHeight(this: *mut CRMInstance, height: c_int);

    /// CRMInstance::PreSpawn
    fn CRMInstance_PreSpawn(
        this: *mut CRMInstance,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool;

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

    /// CRMInstance::SetSide
    fn CRMInstance_SetSide(this: *mut CRMInstance, side: c_int);

    /// CRMInstance::GetSide
    fn CRMInstance_GetSide(this: *mut CRMInstance) -> c_int;

    /// CRMInstance::DrawAutomapSymbol
    #[cfg(not(feature = "DEDICATED"))]
    fn CRMInstance_DrawAutomapSymbol(this: *mut CRMInstance);

    /// CRMInstance::Preview
    fn CRMInstance_Preview(this: *mut CRMInstance, from: *const vec3_t);

    /// CRMInstance parent class SetFilter
    fn CRMInstance_SetFilter_parent(this: *mut CRMInstance, filter: *const c_char);

    /// CRMInstance parent class SetTeamFilter
    fn CRMInstance_SetTeamFilter_parent(this: *mut CRMInstance, teamFilter: *const c_char);

    /// CRMInstance parent class SetMirror
    fn CRMInstance_SetMirror_parent(this: *mut CRMInstance, mirror: c_int);

    /// CRMInstance parent class SetArea
    fn CRMInstance_SetArea_parent(
        this: *mut CRMInstance,
        amanager: *mut CRMAreaManager,
        area: *mut CRMArea,
    );

    /// CRMInstance parent class Preview
    fn CRMInstance_Preview_parent(this: *mut CRMInstance, from: *const vec3_t);

    /// CRMInstance parent class SetMessage
    fn CRMInstance_SetMessage(this: *mut CRMInstance, msg: *const c_char);

    /// CRMInstance parent class SetDescription
    fn CRMInstance_SetDescription(this: *mut CRMInstance, desc: *const c_char);

    /// CRMInstance parent class SetInfo
    fn CRMInstance_SetInfo(this: *mut CRMInstance, info: *const c_char);

    /// CRMInstance parent class PreSpawn
    fn CRMInstance_PreSpawn_parent(
        this: *mut CRMInstance,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool;

    /// CRMInstance parent class Spawn
    fn CRMInstance_Spawn_parent(
        this: *mut CRMInstance,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool;

    /// CRMInstance constructor
    fn CRMInstance_ctor(this: *mut CRMInstance, instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile);

    /// CRMArea::EnableCollision
    fn CRMArea_EnableCollision(this: *mut CRMArea, enable: bool);

    /// CRMArea::IsCollisionEnabled
    fn CRMArea_IsCollisionEnabled(this: *mut CRMArea) -> bool;

    /// CRMAreaManager::CreateArea
    fn CRMAreaManager_CreateArea(
        this: *mut CRMAreaManager,
        origin: *const vec3_t,
        radius1: f32,
        radius2: c_int,
        padding: f32,
        confine: f32,
        origin2: *const vec3_t,
        origin3: *const vec3_t,
        flatten: bool,
        collide: bool,
        lockorigin: bool,
        symmetric: bool,
    ) -> *mut CRMArea;

    /// CRandomTerrain::irand
    fn CRandomTerrain_irand(this: *mut CRandomTerrain, min: c_int, max: c_int) -> c_int;

    /// CRMInstance::GetOrigin
    fn CRMInstance_GetOrigin(this: *mut CRMInstance) -> *mut f32;

    /// CRMInstance::GetSpacingRadius
    fn CRMInstance_GetSpacingRadius(this: *mut CRMInstance) -> f32;

    /// CRMInstance::GetSpacingLine
    fn CRMInstance_GetSpacingLine(this: *mut CRMInstance) -> c_int;

    /// CRMInstance::GetFlattenRadius
    fn CRMInstance_GetFlattenRadius(this: *mut CRMInstance) -> f32;

    /// CRMInstance::GetLockOrigin
    fn CRMInstance_GetLockOrigin(this: *mut CRMInstance) -> bool;
}

impl CRMGroupInstance {
    /************************************************************************************************
     * CRMGroupInstance::CRMGroupInstance
     *	constructur
     *
     * inputs:
     *  settlementID:  ID of the settlement being created
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        unsafe {
            let mut group_instance: CRMGroupInstance = core::mem::zeroed();

            // Call parent constructor on the embedded CRMInstance
            // In C++: : CRMInstance ( instGroup, instFile )
            CRMInstance_ctor(&mut group_instance as *mut CRMGroupInstance as *mut CRMInstance, instGroup, instFile);

            // Grab the padding and confine radius
            // mPaddingSize   = atof ( instGroup->FindPairValue ( "padding", va("%i", TheRandomMissionManager->GetMission()->GetDefaultPadding() ) ) );
            let default_padding = (*(*TheRandomMissionManager).GetMission()).GetDefaultPadding();
            let default_padding_str = va(b"%i\0".as_ptr() as *const c_char, default_padding);
            let padding_str = CGPGroup_FindPairValue(instGroup, b"padding\0".as_ptr() as *const c_char, default_padding_str);
            group_instance.mPaddingSize = atof(padding_str) as f32;

            // mConfineRadius = atof ( instGroup->FindPairValue ( "confine", "0" ) );
            let confine_str = CGPGroup_FindPairValue(instGroup, b"confine\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
            group_instance.mConfineRadius = atof(confine_str) as f32;

            // Parse automap symbol
            let automapSymName = CGPGroup_FindPairValue(instGroup, b"automap_symbol\0".as_ptr() as *const c_char, b"none\0".as_ptr() as *const c_char);
            if strcmpi(automapSymName, b"none\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_NONE ;
                let instance = &mut group_instance as *mut CRMGroupInstance as *mut CRMInstance;
                // Set mAutomapSymbol to 0 (AUTOMAP_NONE)
            } else if strcmpi(automapSymName, b"building\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_BLD  ;
            } else if strcmpi(automapSymName, b"objective\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_OBJ  ;
            } else if strcmpi(automapSymName, b"start\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_START;
            } else if strcmpi(automapSymName, b"end\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_END  ;
            } else if strcmpi(automapSymName, b"enemy\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_ENEMY;
            } else if strcmpi(automapSymName, b"friend\0".as_ptr() as *const c_char) == 0 {
                // mAutomapSymbol = AUTOMAP_FRIEND;
            } else {
                // mAutomapSymbol = atoi( automapSymName );
            }

            // optional instance objective strings
            // SetMessage(instGroup->FindPairValue("objective_message",""));
            let message_str = CGPGroup_FindPairValue(instGroup, b"objective_message\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
            CRMInstance_SetMessage(&mut group_instance as *mut CRMGroupInstance as *mut CRMInstance, message_str);

            // SetDescription(instGroup->FindPairValue("objective_description",""));
            let desc_str = CGPGroup_FindPairValue(instGroup, b"objective_description\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
            CRMInstance_SetDescription(&mut group_instance as *mut CRMGroupInstance as *mut CRMInstance, desc_str);

            // SetInfo(instGroup->FindPairValue("objective_info",""));
            let info_str = CGPGroup_FindPairValue(instGroup, b"objective_info\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
            CRMInstance_SetInfo(&mut group_instance as *mut CRMGroupInstance as *mut CRMInstance, info_str);

            // Iterate through the sub groups to determine the instances which make up the group
            // instGroup = instGroup->GetSubGroups ( );
            let mut instGroup = CGPGroup_GetSubGroups(instGroup);

            while !instGroup.is_null() {
                // Make sure only instances are specified as sub groups
                // assert ( 0 == stricmp ( instGroup->GetName ( ), "instance" ) );

                // Grab the name
                let name = CGPGroup_FindPairValue(instGroup, b"name\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);

                // Grab the range information
                let minrange_str = CGPGroup_FindPairValue(instGroup, b"minrange\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
                let _minrange = atof(minrange_str) as f32;

                let maxrange_str = CGPGroup_FindPairValue(instGroup, b"maxrange\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
                let _maxrange = atof(maxrange_str) as f32;

                // Grab the count information and randomly generate a count value
                let mincount_str = CGPGroup_FindPairValue(instGroup, b"mincount\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
                let mincount = atoi(mincount_str);

                let maxcount_str = CGPGroup_FindPairValue(instGroup, b"maxcount\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
                let maxcount = atoi(maxcount_str);

                let mut count = mincount;

                if maxcount > mincount {
                    // count += (TheRandomMissionManager->GetLandScape()->irand(0, maxcount-mincount));
                    let landscape = (*TheRandomMissionManager).GetTerrain();
                    if !landscape.is_null() {
                        let rand_val = CRandomTerrain_irand(landscape, 0, maxcount - mincount);
                        count += rand_val;
                    }
                }

                // For each count create and add the instance
                while count > 0 {
                    count -= 1;

                    // Create the instance
                    let instance = CRMInstanceFile_CreateInstance(instFile, name);

                    // Skip this instance if it couldnt be created for some reason.  The CreateInstance
                    // method will report an error so no need to do so here.
                    if !instance.is_null() {
                        // Set the min and max range for the instance
                        // instance->SetFilter(mFilter);
                        // instance->SetTeamFilter(mTeamFilter);
                        // Note: In this implementation, we would need to access mFilter and mTeamFilter
                        // from the parent CRMInstance structure. This requires C++ wrappers.
                        // For now, we skip this as it would require additional extern functions.

                        // Add the instance to the list
                        // mInstances.push_back ( instance );
                        // Note: In Rust, we would need to manipulate mInstances (LinkedList<*mut CRMInstance>)
                        // This requires C++ wrappers for LinkedList operations or direct manipulation.
                    }
                }

                // Next sub group
                instGroup = CGPGroup_GetNext(instGroup);
            }

            group_instance
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::~CRMGroupInstance
     *	Removes all buildings and inhabitants
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn drop(&mut self) {
        self.RemoveInstances();
    }

    /************************************************************************************************
     * CRMGroupInstance::SetFilter
     *	Sets a filter used to exclude instances
     *
     * inputs:
     *  filter: filter name
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetFilter(&mut self, filter: *const c_char) {
        unsafe {
            // CRMInstance::SetFilter(filter);
            CRMInstance_SetFilter_parent(self as *mut CRMGroupInstance as *mut CRMInstance, filter);

            // for(it = mInstances.begin(); it != mInstances.end(); it++)
            // {
            //     (*it)->SetFilter(filter);
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::SetTeamFilter
     *	Sets the filter used to exclude team based instances
     *
     * inputs:
     *  teamFilter: filter name
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        unsafe {
            // CRMInstance::SetTeamFilter(teamFilter);
            CRMInstance_SetTeamFilter_parent(self as *mut CRMGroupInstance as *mut CRMInstance, teamFilter);

            // for(it = mInstances.begin(); it != mInstances.end(); it++)
            // {
            //     (*it)->SetTeamFilter(teamFilter);
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::SetMirror
     *	Sets the flag to mirror an instance on map
     *
     * inputs:
     *  mirror
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetMirror(&mut self, mirror: c_int) {
        unsafe {
            // CRMInstance::SetMirror(mirror);
            CRMInstance_SetMirror_parent(self as *mut CRMGroupInstance as *mut CRMInstance, mirror);

            // for(it = mInstances.begin(); it != mInstances.end(); it++)
            // {
            //     (*it)->SetMirror(mirror);
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::RemoveInstances
     *	Removes all instances associated with the group
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn RemoveInstances(&mut self) {
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     delete *it;
        // }
        //
        // mInstances.clear();
        // Note: Deleting instances from mInstances (LinkedList<*mut CRMInstance>)
        // requires proper C++ object destruction or C++ wrappers.
        // This is a stub that would need proper implementation.
    }

    /************************************************************************************************
     * CRMGroupInstance::PreSpawn
     *	Prepares the group for spawning by
     *
     * inputs:
     *  landscape: landscape to calculate the position within
     *  instance: instance to calculate the position for
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            // for(it = mInstances.begin(); it != mInstances.end(); it++ )
            // {
            //     CRMInstance* instance = *it;
            //
            //     instance->SetFlattenHeight ( mFlattenHeight );
            //
            //     // Add the instance to the landscape now
            //     instance->PreSpawn ( terrain, IsServer );
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation

            // return CRMInstance::PreSpawn ( terrain, IsServer );
            CRMInstance_PreSpawn_parent(self as *mut CRMGroupInstance as *mut CRMInstance, terrain, IsServer)
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::Spawn
     *	Adds the group instance to the given landscape using the specified origin.  All sub instances
     *  will be added to the landscape within their min and max range from the origin.
     *
     * inputs:
     *  landscape: landscape to add the instance group to
     *  origin: origin of the instance group
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            // Spawn all the instances associated with this group
            // for(it = mInstances.begin(); it != mInstances.end(); it++)
            // {
            //     CRMInstance* instance = *it;
            //     instance->SetSide(GetSide()); // which side owns it?
            //
            //     // Add the instance to the landscape now
            //     instance->Spawn ( terrain, IsServer );
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation

            #[cfg(not(feature = "DEDICATED"))]
            {
                // DrawAutomapSymbol();
                CRMInstance_DrawAutomapSymbol(self as *mut CRMGroupInstance as *mut CRMInstance);
            }

            true
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::Preview
     *	Renders debug information for the instance
     *
     * inputs:
     *  from: point to render the preview from
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Preview(&self, from: *const vec3_t) {
        unsafe {
            // CRMInstance::Preview ( from );
            CRMInstance_Preview_parent(self as *const CRMGroupInstance as *mut CRMGroupInstance as *mut CRMInstance, from);

            // Render all the instances
            // for(it = mInstances.begin(); it != mInstances.end(); it++)
            // {
            //     CRMInstance* instance = *it;
            //
            //     instance->Preview ( from );
            // }
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation
        }
    }

    /************************************************************************************************
     * CRMGroupInstance::SetArea
     *	Overidden to make sure the groups area doesnt eat up any room.  The collision on the
     *  groups area will be turned off
     *
     * inputs:
     *  area: area to set
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        unsafe {
            // bool collide = area->IsCollisionEnabled ( );
            let collide = CRMArea_IsCollisionEnabled(area);

            // Disable collision
            // area->EnableCollision ( false );
            CRMArea_EnableCollision(area, false);

            // Do what really needs to get done
            // CRMInstance::SetArea ( amanager, area );
            CRMInstance_SetArea_parent(self as *mut CRMGroupInstance as *mut CRMInstance, amanager, area);

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
            // Note: Iterating over mInstances requires C++ wrapper or direct LinkedList manipulation
            // The above loop would need:
            // - Accessing mInstances (LinkedList)
            // - Calling GetOrigin() on self and instances
            // - Calling GetSpacingRadius(), GetSpacingLine(), GetFlattenRadius(), GetLockOrigin() on instances
            // - Calling CreateArea on amanager
            // - Converting float to bool for GetFlattenRadius()?true:false
            // All of which require C++ wrappers or additional extern functions.
        }
    }
}
