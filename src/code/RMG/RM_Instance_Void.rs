#![allow(non_snake_case)]

// Preserved includes documentation from oracle/code/RMG/RM_Instance_Void.cpp:
// #include "../server/exe_headers.h"
// #include "rm_headers.h"
// #include "rm_instance_void.h"

/************************************************************************************************
 *
 * RM_Instance_Void.cpp
 *
 * Implements the CRMVoidInstance class.  This class just adds a void into the
 * area manager to help space things out.
 *
 ************************************************************************************************/

use core::ffi::{c_char, c_double};

// LOCAL STUB: Forward declarations for types used in this file
// Full definitions are in their respective modules
pub struct CGPGroup;
pub struct CRMInstanceFile;
pub struct CRMAreaManager;
pub struct CRMArea;

extern "C" {
    /// atof - Convert C string to floating point number
    /// double atof(const char *s);
    fn atof(s: *const c_char) -> c_double;

    /// Stub for parent class method
    /// void CRMInstance::SetArea ( CRMAreaManager* amanager, CRMArea* area );
    fn CRMInstance_SetArea(amanager: *mut CRMAreaManager, area: *mut CRMArea);

    /// Stub for CRMArea method
    /// void CRMArea::EnableCollision ( bool enable );
    fn CRMArea_EnableCollision(this: *mut CRMArea, enable: bool);

    /// Stub for CGPGroup method
    /// const char* CGPGroup::FindPairValue ( const char* key, const char* default );
    fn CGPGroup_FindPairValue(
        this: *mut CGPGroup,
        key: *const c_char,
        default: *const c_char,
    ) -> *const c_char;

    /// Stub for parent constructor
    /// CRMInstance::CRMInstance ( CGPGroup *instGroup, CRMInstanceFile& instFile );
    fn CRMInstance_ctor(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile);
}

// Implementation module for CRMVoidInstance methods
pub struct CRMVoidInstance {
    // Inherits from CRMInstance
}

impl CRMVoidInstance {
    /************************************************************************************************
     * CRMVoidInstance::CRMVoidInstance
     *	constructs a void instance
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
            // Call parent constructor
            // In C++: : CRMInstance ( instGroup, instFile )
            CRMInstance_ctor(instGroup, instFile);

            // mSpacingRadius = atof( instGroup->FindPairValue ( "spacing", "0" ) );
            let spacing_str = CGPGroup_FindPairValue(
                instGroup,
                b"spacing\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            let _mSpacingRadius = atof(spacing_str);

            // mFlattenRadius = atof( instGroup->FindPairValue ( "flatten", "0" ) );
            let flatten_str = CGPGroup_FindPairValue(
                instGroup,
                b"flatten\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            let _mFlattenRadius = atof(flatten_str);

            CRMVoidInstance {}
        }
    }

    /************************************************************************************************
     * CRMVoidInstance::SetArea
     *	Overidden to make sure the void area doesnt continually.
     *
     * inputs:
     *  area: area to set
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        // Disable collision
        // area->EnableCollision ( false );
        CRMArea_EnableCollision(area, false);

        // Do what really needs to get done
        // CRMInstance::SetArea ( amanager, area );
        CRMInstance_SetArea(amanager, area);
    }
}
