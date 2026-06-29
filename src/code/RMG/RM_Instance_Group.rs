/************************************************************************************************
 *
 * RM_Instance_Group.cpp
 *
 * Implements the CRMGroupInstance class.  This class is reponsible for parsing a
 * group instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

// #include "../server/exe_headers.h"
use crate::code::server::exe_headers_h::*;
// #include "rm_headers.h"
use crate::code::RMG::RM_Headers_h::*;
// #include "rm_instance_group.h"
use crate::code::RMG::RM_Instance_Group_h::*;

use core::ffi::{c_char, c_double, c_int};

// C stdlib functions used in this file
extern "C" {
    fn atof(s: *const c_char) -> c_double;
    fn atoi(s: *const c_char) -> c_int;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
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
    // C++: CRMGroupInstance( CGPGroup *instGroup, CRMInstanceFile& instFile )
    //        : CRMInstance ( instGroup, instFile )
    // porting note: Rust lacks C++ inheritance; the base CRMInstance sub-object occupies the
    //   start of CRMGroupInstance's memory in C++ single-inheritance layout.  Base class init
    //   and protected field access are performed through *mut CRMInstance casts.  When the
    //   CRMGroupInstance Rust struct is updated to embed CRMInstance as its first field, the
    //   base init line below can be replaced with:
    //     core::ptr::write(self_ as *mut CRMInstance,
    //                      CRMInstance::new(instGroup, &mut *instFile));
    // porting note: mAutomapSymbol is a private field of CRMInstance accessed here via cast
    //   because C++ treats it as protected in the derived class.  The field must be pub (or
    //   a SetAutomapSymbol setter must be added to CRMInstance) for this to compile.
    pub unsafe fn ctor(
        self_: *mut CRMGroupInstance,
        instGroup: *mut CGPGroup,
        instFile: *mut CRMInstanceFile,
    ) {
        // : CRMInstance ( instGroup, instFile )
        // porting note: invoke base class constructor; see note above regarding layout
        CRMInstance::new(instGroup, &mut *instFile);

        let base: *mut CRMInstance = self_ as *mut CRMInstance;

        // Grab the padding and confine radius
        (*self_).mPaddingSize   = atof ( (*instGroup).FindPairValue ( b"padding\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*TheRandomMissionManager).GetMission()).GetDefaultPadding() ) ) ) as f32;
        (*self_).mConfineRadius = atof ( (*instGroup).FindPairValue ( b"confine\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char ) ) as f32;

        let automapSymName: *const c_char = (*instGroup).FindPairValue ( b"automap_symbol\0".as_ptr() as *const c_char, b"none\0".as_ptr() as *const c_char );
        if 0 == strcmpi(automapSymName, b"none\0".as_ptr() as *const c_char)	   	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_NONE   as c_int; }
        else if 0 == strcmpi(automapSymName, b"building\0".as_ptr() as *const c_char)  	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_BLD    as c_int; }
        else if 0 == strcmpi(automapSymName, b"objective\0".as_ptr() as *const c_char) 	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_OBJ    as c_int; }
        else if 0 == strcmpi(automapSymName, b"start\0".as_ptr() as *const c_char)	   	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_START  as c_int; }
        else if 0 == strcmpi(automapSymName, b"end\0".as_ptr() as *const c_char)	   	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_END    as c_int; }
        else if 0 == strcmpi(automapSymName, b"enemy\0".as_ptr() as *const c_char)	   	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_ENEMY  as c_int; }
        else if 0 == strcmpi(automapSymName, b"friend\0".as_ptr() as *const c_char)	   	{ (*base).mAutomapSymbol = CRMAutomapSymbol::AUTOMAP_FRIEND as c_int; }
        else 	                                                                                { (*base).mAutomapSymbol = atoi( automapSymName ); }

        // optional instance objective strings
        (*base).SetMessage((*instGroup).FindPairValue(b"objective_message\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char));
        (*base).SetDescription((*instGroup).FindPairValue(b"objective_description\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char));
        (*base).SetInfo((*instGroup).FindPairValue(b"objective_info\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char));

        // Iterate through the sub groups to determine the instances which make up the group
        let mut instGroup: *mut CGPGroup = (*instGroup).GetSubGroups ( );

        while !instGroup.is_null() {
            let name: *const c_char;
            let mincount: c_int;
            let maxcount: c_int;
            let mut count: c_int;
            let _minrange: f32;
            let _maxrange: f32;

            // Make sure only instances are specified as sub groups
            debug_assert!( 0 == stricmp ( (*instGroup).GetName ( ), b"instance\0".as_ptr() as *const c_char ) );

            // Grab the name
            name     = (*instGroup).FindPairValue ( b"name\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char );

            // Grab the range information
            _minrange = atof((*instGroup).FindPairValue ( b"minrange\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char ) ) as f32;
            _maxrange = atof((*instGroup).FindPairValue ( b"maxrange\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char ) ) as f32;

            // Grab the count information and randomly generate a count value
            mincount = atoi((*instGroup).FindPairValue ( b"mincount\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char ) );
            maxcount = atoi((*instGroup).FindPairValue ( b"maxcount\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char ) );
            count	 = mincount;

            if maxcount > mincount {
                count += (*(*TheRandomMissionManager).GetLandScape()).irand(0, maxcount-mincount);
            }

            // For each count create and add the instance
            // C++: for ( ; count ; count -- )
            while count != 0 {
                // Create the instance
                let instance: *mut CRMInstance = (*instFile).CreateInstance ( name );

                // mirrors the C++ for's count-- post-decrement, which runs even when continuing
                count -= 1;

                // Skip this instance if it couldnt be created for some reason.  The CreateInstance
                // method will report an error so no need to do so here.
                if instance.is_null() {
                    continue;
                }

                // Set the min and max range for the instance
                (*instance).SetFilter((*base).GetFilter());
                (*instance).SetTeamFilter((*base).GetTeamFilter());

                // Add the instance to the list
                (*self_).mInstances.push_back ( instance );
            }

            // Next sub group
            instGroup = (*instGroup).GetNext ( );
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
    // C++: ~CRMGroupInstance(void)
    pub unsafe fn dtor(self_: *mut CRMGroupInstance) {
        // Cleanup
        (*self_).RemoveInstances ( );
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
    pub unsafe fn SetFilter(&mut self, filter: *const c_char) {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        CRMInstance::SetFilter(&mut *base, filter);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        for &instance in self.mInstances.iter() {
            (*instance).SetFilter(filter);
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
    pub unsafe fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        CRMInstance::SetTeamFilter(&mut *base, teamFilter);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        for &instance in self.mInstances.iter() {
            (*instance).SetTeamFilter(teamFilter);
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
    pub unsafe fn SetMirror(&mut self, mirror: c_int) {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        CRMInstance::SetMirror(&mut *base, mirror);
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        for &instance in self.mInstances.iter() {
            (*instance).SetMirror(mirror);
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
    pub unsafe fn RemoveInstances(&mut self) {
        // rmInstanceIter_t it;
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     delete *it;
        // }
        // Safety: instances were allocated via C++ operator new through CRMInstanceFile::CreateInstance.
        // drop(Box::from_raw(instance)) approximates `delete *it`; a real C++/Rust interop would
        // call the C++ destructor through a shim instead.
        for &instance in self.mInstances.iter() {
            // delete *it;
            drop(Box::from_raw(instance));
        }

        self.mInstances.clear();
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
    pub unsafe fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        // for(it = mInstances.begin(); it != mInstances.end(); it++ )
        for &instance in self.mInstances.iter() {
            (*instance).SetFlattenHeight ( (*base).GetFlattenHeight() );

            // Add the instance to the landscape now
            (*instance).PreSpawn ( terrain, IsServer );
        }

        CRMInstance::PreSpawn(&mut *base, terrain, IsServer)
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
    pub unsafe fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        // Spawn all the instances associated with this group
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        for &instance in self.mInstances.iter() {
            (*instance).SetSide((*base).GetSide()); // which side owns it?

            // Add the instance to the landscape now
            (*instance).Spawn ( terrain, IsServer );
        }
        #[cfg(not(feature = "DEDICATED"))]
        {
            (*base).DrawAutomapSymbol();
        }
        true
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
    pub unsafe fn Preview(&self, from: *const vec3_t) {
        // rmInstanceIter_t it;
        let base: *const CRMInstance = self as *const CRMGroupInstance as *const CRMInstance;

        CRMInstance::Preview(&*base, from);

        // Render all the instances
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        for &instance in self.mInstances.iter() {
            (*instance).Preview ( from );
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
    pub unsafe fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        // rmInstanceIter_t it;
        let base: *mut CRMInstance = self as *mut CRMGroupInstance as *mut CRMInstance;

        let collide: bool = (*area).IsCollisionEnabled ( );

        // Disable collision
        (*area).EnableCollision ( false );

        // Do what really needs to get done
        CRMInstance::SetArea(&mut *base, amanager, area);

        // Prepare for spawn by calculating all the positions of the sub instances
        // and flattening the ground below them.
        // for(it = mInstances.begin(); it != mInstances.end(); it++ )
        for &instance in self.mInstances.iter() {
            let go: *mut vec_t = (*base).GetOrigin();

            // Drop it in the center of the group for now
            let origin: vec3_t = [*go, *go.add(1), 2500.0_f32];

            // Set the area of position
            let newarea: *mut CRMArea = (*amanager).CreateArea (
                origin,
                (*instance).GetSpacingRadius(),
                (*instance).GetSpacingLine(),
                self.mPaddingSize,
                self.mConfineRadius,
                *(go as *const vec3_t),    // GetOrigin()
                *(go as *const vec3_t),    // GetOrigin()
                (*instance).GetFlattenRadius() != 0.0_f32,
                collide,
                (*instance).GetLockOrigin(),
                (*area).GetSymmetric ( ),
            );
            (*instance).SetArea ( amanager, newarea );
        }
    }
}
