//Anything above this #include will be ignored by the compiler

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::ffi::{c_char, c_int};

/************************************************************************************************
 *
 * RM_InstanceFile.cpp
 *
 * implements the CRMInstanceFile class.  This class provides functionality to load
 * and create instances from an instance file.  First call Open to open the instance file and
 * then use CreateInstance to create new instances.  When finished call Close to cleanup.
 *
 ************************************************************************************************/

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::RMG::RM_Headers_h::*;

//#include "rm_instance_npc.h"
use crate::codemp::RMG::RM_Instance_BSP_h::*;
use crate::codemp::RMG::RM_Instance_Random_h::*;
use crate::codemp::RMG::RM_Instance_Group_h::*;
use crate::codemp::RMG::RM_Instance_Void_h::*;

// Triage: CRMInstanceFile is defined in RM_InstanceFile.h; import from its paired header module
// rather than redefining it here.
use crate::codemp::RMG::RM_InstanceFile_h::*;

/************************************************************************************************
 * CRMInstanceFile::CRMInstanceFile
 *	constructor
 *
 * inputs:
 *  none
 *
 * return:
 *	none
 *
 ************************************************************************************************/
impl CRMInstanceFile {
    pub unsafe fn new() -> Self {
        // C++ constructor body: mInstances = NULL.
        // zero-init covers the null pointer; mParser is zero-initialised (C++ default ctor).
        let mut s = core::mem::zeroed::<CRMInstanceFile>();
        s.mInstances = core::ptr::null_mut();
        s
    }
}

/************************************************************************************************
 * CRMInstanceFile::~CRMInstanceFile
 *	Destroys the instance file by freeing the parser
 *
 * inputs:
 *  none
 *
 * return:
 *	none
 *
 ************************************************************************************************/
impl Drop for CRMInstanceFile {
    fn drop(&mut self) {
        unsafe { self.Close(); }
    }
}

impl CRMInstanceFile {
    /************************************************************************************************
     * CRMInstanceFile::Open
     *	Opens the given instance file and prepares it for use in instance creation
     *
     * inputs:
     *  instance: Name of instance to open.  Note that the root path will be automatically
     *			  added and shouldnt be included in the given name
     *
     * return:
     *	true: instance file successfully loaded
     *  false: instance file could not be loaded for some reason
     *
     ************************************************************************************************/
    pub unsafe fn Open(&mut self, instance: *const c_char) -> bool {
        let mut instanceDef: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let basegroup: *mut CGPGroup;

        // Build the filename
        Com_sprintf(
            instanceDef.as_mut_ptr(),
            MAX_QPATH as c_int,
            b"ext_data/rmg/%s.instance\0".as_ptr() as *const c_char,
            instance,
        );

        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            // Debug message
            Com_Printf(
                b"CM_Terrain: Loading and parsing instanceDef %s.....\n\0".as_ptr() as *const c_char,
                instance,
            );
        }

        // Parse the text file using the generic parser
        if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser) {
            Com_sprintf(
                instanceDef.as_mut_ptr(),
                MAX_QPATH as c_int,
                b"ext_data/arioche/%s.instance\0".as_ptr() as *const c_char,
                instance,
            );
            if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser) {
                Com_Printf(va(
                    b"CM_Terrain: Could not open instance file '%s'\n\0".as_ptr() as *const c_char,
                    instanceDef.as_ptr(),
                ));
                return false;
            }
        }

        // The whole file....
        basegroup = self.mParser.GetBaseParseGroup();

        // The root { } struct
        self.mInstances = (*basegroup).GetSubGroups();

        // The "instances" { } structure
        self.mInstances = (*self.mInstances).GetSubGroups();

        true
    }

    /************************************************************************************************
     * CRMInstanceFile::Close
     *	Closes an open instance file
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn Close(&mut self) {
        // If not open then dont close  it
        if self.mInstances == core::ptr::null_mut() {
            return;
        }
        self.mParser.Clean();

        self.mInstances = core::ptr::null_mut();
    }

    /************************************************************************************************
     * CRMInstanceFile::CreateInstance
     *	Creates an instance (to be freed by caller) using the given instance name.
     *
     * inputs:
     *  name: Name of the instance to read from the instance file
     *
     * return:
     *	NULL: instance could not be read from the instance file
     *  NON-NULL: instance created and returned for further use
     *
     ************************************************************************************************/
    pub unsafe fn CreateInstance(&mut self, name: *const c_char) -> *mut CRMInstance {
        static mut instanceID: c_int = 0;

        let mut group: *mut CGPGroup;
        let mut instance: *mut CRMInstance = core::ptr::null_mut();

        // Make sure we were loaded
        assert!(!self.mInstances.is_null());

        // Search through the instances for the one with the given name
        group = self.mInstances;
        while !group.is_null() {
            // Skip it if the name doesnt match
            if stricmp(
                name,
                (*group).FindPairValue(
                    b"name\0".as_ptr() as *const c_char,
                    b"\0".as_ptr() as *const c_char,
                ),
            ) != 0
            {
                group = (*group).GetNext();
                continue;
            }

            // Handle the various forms of instance types
            if stricmp((*group).GetName(), b"bsp\0".as_ptr() as *const c_char) == 0 {
                instance = CRMBSPInstance::new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
            } else if stricmp((*group).GetName(), b"npc\0".as_ptr() as *const c_char) == 0 {
                //			instance = new CRMNPCInstance ( group, *this );
                group = (*group).GetNext();
                continue;
            } else if stricmp((*group).GetName(), b"group\0".as_ptr() as *const c_char) == 0 {
                instance = CRMGroupInstance::new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
            } else if stricmp((*group).GetName(), b"random\0".as_ptr() as *const c_char) == 0 {
                instance = CRMRandomInstance::new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
            } else if stricmp((*group).GetName(), b"void\0".as_ptr() as *const c_char) == 0 {
                instance = CRMVoidInstance::new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
            } else {
                group = (*group).GetNext();
                continue;
            }

            // If the instance isnt valid after being created then delete it
            if !(*instance).IsValid() {
                drop(Box::from_raw(instance));
                return core::ptr::null_mut();
            }

            // The instance was successfully created so return it
            return instance;
        }

        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            // The instance wasnt found in the file so report it
            Com_Printf(va(
                b"WARNING:  Instance '%s' was not found in the active instance file\n\0".as_ptr()
                    as *const c_char,
                name,
            ));
        }

        core::ptr::null_mut()
    }
}
