//Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::RMG::RM_Headers_h::*;
use crate::codemp::RMG::RM_Instance_Void_h::*;

use core::ffi::c_char;

/************************************************************************************************
 *
 * RM_Instance_Void.cpp
 *
 * Implements the CRMVoidInstance class.  This class just adds a void into the
 * area manager to help space things out.
 *
 ************************************************************************************************/

extern "C" {
    fn atof(s: *const c_char) -> f64;
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
    pub unsafe fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        // C++: : CRMInstance ( instGroup, instFile )
        let mut self_ = Self {
            base: CRMInstance::new(instGroup, instFile),
            mSpacingRadius: 0.0,
            mFlattenRadius: 0.0,
        };
        self_.mSpacingRadius = atof( (*instGroup).FindPairValue ( c"spacing".as_ptr(), c"0".as_ptr() ) );
        self_.mFlattenRadius = atof( (*instGroup).FindPairValue ( c"flatten".as_ptr(), c"0".as_ptr() ) );
        self_
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
        (*area).EnableCollision ( false );

        // Do what really needs to get done
        self.base.SetArea ( amanager, area );
    }
}
