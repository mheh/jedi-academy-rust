#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]

// Anything above this #include will be ignored by the compiler
use crate::codemp::qcommon::exe_headers_h::*;

/************************************************************************************************
 *
 * RM_Instance_Group.cpp
 *
 * Implements the CRMGroupInstance class.  This class is reponsible for parsing a
 * group instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

use crate::codemp::RMG::RM_Headers_h::*;
use crate::codemp::RMG::RM_Instance_Group_h::*;

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
// Porting note: C++ base-class initializer `: CRMInstance(instGroup, instFile)` cannot be
// expressed in Rust; this `ctor` method covers the derived-class constructor body only.
impl CRMGroupInstance {
    pub unsafe fn ctor(
        &mut self,
        mut instGroup: *mut CGPGroup,
        instFile: *mut CRMInstanceFile,
    ) {
        use core::ffi::{c_char, c_float, c_int};
        use core::ptr::addr_of;

        // Grab the padding and confine radius
        self.mPaddingSize = atof((*instGroup).FindPairValue(
            b"padding\0".as_ptr() as *const c_char,
            va(
                b"%i\0".as_ptr() as *const c_char,
                (*(*addr_of!(TheRandomMissionManager)).GetMission()).GetDefaultPadding(),
            ),
        ));
        self.mConfineRadius = atof((*instGroup).FindPairValue(
            b"confine\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
        ));

        let automapSymName: *const c_char = (*instGroup).FindPairValue(
            b"automap_symbol\0".as_ptr() as *const c_char,
            b"none\0".as_ptr() as *const c_char,
        );
        if 0 == Q_stricmp(automapSymName, b"none\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_NONE;
        } else if 0 == Q_stricmp(automapSymName, b"building\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_BLD;
        } else if 0 == Q_stricmp(automapSymName, b"objective\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_OBJ;
        } else if 0 == Q_stricmp(automapSymName, b"start\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_START;
        } else if 0 == Q_stricmp(automapSymName, b"end\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_END;
        } else if 0 == Q_stricmp(automapSymName, b"enemy\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_ENEMY;
        } else if 0 == Q_stricmp(automapSymName, b"friend\0".as_ptr() as *const c_char) {
            self.mAutomapSymbol = AUTOMAP_FRIEND;
        } else {
            self.mAutomapSymbol = atoi(automapSymName);
        }

        // optional instance objective strings
        self.SetMessage((*instGroup).FindPairValue(
            b"objective_message\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
        ));
        self.SetDescription((*instGroup).FindPairValue(
            b"objective_description\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
        ));
        self.SetInfo((*instGroup).FindPairValue(
            b"objective_info\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
        ));

        // Iterate through the sub groups to determine the instances which make up the group
        instGroup = (*instGroup).GetSubGroups();

        while !instGroup.is_null() {
            let instance: *mut CRMInstance;
            let name: *const c_char;
            let mincount: c_int;
            let maxcount: c_int;
            let mut count: c_int;
            let minrange: c_float;
            let maxrange: c_float;

            // Make sure only instances are specified as sub groups
            assert!(0 == stricmp((*instGroup).GetName(), b"instance\0".as_ptr() as *const c_char));

            // Grab the name
            name = (*instGroup).FindPairValue(
                b"name\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );

            // Grab the range information
            minrange = atof((*instGroup).FindPairValue(
                b"minrange\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            ));
            maxrange = atof((*instGroup).FindPairValue(
                b"maxrange\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            ));

            // Grab the count information and randomly generate a count value
            mincount = atoi((*instGroup).FindPairValue(
                b"mincount\0".as_ptr() as *const c_char,
                b"1\0".as_ptr() as *const c_char,
            ));
            maxcount = atoi((*instGroup).FindPairValue(
                b"maxcount\0".as_ptr() as *const c_char,
                b"1\0".as_ptr() as *const c_char,
            ));
            count = mincount;

            if maxcount > mincount {
                count += (*(*addr_of!(TheRandomMissionManager)).GetLandScape())
                    .irand(0, maxcount - mincount);
            }

            // For each count create and add the instance
            // Porting note: C++ `for (; count; count--)` — decrement is the for-increment,
            // executed even on `continue`. Translated as while-loop with leading decrement so
            // `continue` still decrements before re-checking the condition.
            while count != 0 {
                count -= 1;
                // Create the instance
                instance = (*instFile).CreateInstance(name);

                // Skip this instance if it couldnt be created for some reason.  The CreateInstance
                // method will report an error so no need to do so here.
                if instance.is_null() {
                    continue;
                }

                // Set the min and max range for the instance
                (*instance).SetFilter(self.mFilter.as_ptr());
                (*instance).SetTeamFilter(self.mTeamFilter.as_ptr());

                // Add the instance to the list
                self.mInstances.push_back(instance);
            }

            // Next sub group
            instGroup = (*instGroup).GetNext();
        }
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
impl CRMGroupInstance {
    pub unsafe fn dtor(&mut self) {
        // Cleanup
        self.RemoveInstances();
    }
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
impl CRMGroupInstance {
    pub unsafe fn SetFilter(&mut self, filter: *const core::ffi::c_char) {
        let mut it: rmInstanceIter_t;

        // Porting note: C++ qualified base-class call `CRMInstance::SetFilter(filter)` — cast
        // self to the base-class pointer to invoke the non-virtual base implementation.
        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).SetFilter(filter);
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            (*(*it)).SetFilter(filter);
            it = it.next();
        }
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
impl CRMGroupInstance {
    pub unsafe fn SetTeamFilter(&mut self, teamFilter: *const core::ffi::c_char) {
        let mut it: rmInstanceIter_t;

        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).SetTeamFilter(teamFilter);
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            (*(*it)).SetTeamFilter(teamFilter);
            it = it.next();
        }
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
impl CRMGroupInstance {
    pub unsafe fn SetMirror(&mut self, mirror: core::ffi::c_int) {
        let mut it: rmInstanceIter_t;

        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).SetMirror(mirror);
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            (*(*it)).SetMirror(mirror);
            it = it.next();
        }
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
impl CRMGroupInstance {
    pub unsafe fn RemoveInstances(&mut self) {
        let mut it: rmInstanceIter_t;

        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            // Porting note: C++ `delete *it` — calls destructor and frees C++ new-allocated memory.
            // Translated as drop(Box::from_raw(*it)); actual cleanup requires C++ delete semantics.
            drop(Box::from_raw(*it));
            it = it.next();
        }

        self.mInstances.clear();
    }
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
impl CRMGroupInstance {
    pub unsafe fn PreSpawn(
        &mut self,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool {
        let mut it: rmInstanceIter_t;

        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            let instance: *mut CRMInstance = *it;

            (*instance).SetFlattenHeight(self.mFlattenHeight);

            // Add the instance to the landscape now
            (*instance).PreSpawn(terrain, IsServer);

            it = it.next();
        }

        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).PreSpawn(terrain, IsServer)
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
impl CRMGroupInstance {
    pub unsafe fn Spawn(
        &mut self,
        terrain: *mut CRandomTerrain,
        IsServer: qboolean,
    ) -> bool {
        let mut it: rmInstanceIter_t;

        // Spawn all the instances associated with this group
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            let instance: *mut CRMInstance = *it;
            (*instance).SetSide((*(self as *mut CRMGroupInstance as *mut CRMInstance)).GetSide()); // which side owns it?

            // Add the instance to the landscape now
            (*instance).Spawn(terrain, IsServer);

            it = it.next();
        }

        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).DrawAutomapSymbol();

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
impl CRMGroupInstance {
    pub unsafe fn Preview(&mut self, from: *const vec3_t) {
        let mut it: rmInstanceIter_t;

        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).Preview(from);

        // Render all the instances
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            let instance: *mut CRMInstance = *it;

            (*instance).Preview(from);

            it = it.next();
        }
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
impl CRMGroupInstance {
    pub unsafe fn SetArea(
        &mut self,
        amanager: *mut CRMAreaManager,
        area: *mut CRMArea,
    ) {
        let mut it: rmInstanceIter_t;

        let collide: bool = (*area).IsCollisionEnabled();

        // Disable collision
        (*area).EnableCollision(false);

        // Do what really needs to get done
        (*(self as *mut CRMGroupInstance as *mut CRMInstance)).SetArea(amanager, area);

        // Prepare for spawn by calculating all the positions of the sub instances
        // and flattening the ground below them.
        it = self.mInstances.begin();
        while it != self.mInstances.end() {
            let instance: *mut CRMInstance = *it;
            let newarea: *mut CRMArea;
            let mut origin: vec3_t = [0.0; 3];

            let base = self as *mut CRMGroupInstance as *mut CRMInstance;

            // Drop it in the center of the group for now
            origin[0] = *(*base).GetOrigin().add(0);
            origin[1] = *(*base).GetOrigin().add(1);
            origin[2] = 2500.0;

            // Set the area of position
            newarea = (*amanager).CreateArea(
                origin,
                (*instance).GetSpacingRadius(),
                (*instance).GetSpacingLine(),
                self.mPaddingSize,
                self.mConfineRadius,
                (*base).GetOrigin(),
                (*base).GetOrigin(),
                if (*instance).GetFlattenRadius() != 0.0 { true } else { false },
                collide,
                (*instance).GetLockOrigin(),
                (*area).GetSymmetric(),
            );
            (*instance).SetArea(amanager, newarea);

            it = it.next();
        }
    }
}
