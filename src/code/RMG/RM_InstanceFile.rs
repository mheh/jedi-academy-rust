// Translated from oracle/code/RMG/RM_InstanceFile.cpp

// Preserved includes documentation:
// #include "../server/exe_headers.h"
// #include "rm_headers.h"
// #include "rm_instance_npc.h"
// #include "rm_instance_bsp.h"
// #include "rm_instance_random.h"
// #include "rm_instance_group.h"
// #include "rm_instance_void.h"

use core::ffi::{c_int, c_char};

// LOCAL STUB: Declarations for C types used in this file
// These are defined in the original headers but included here for structural coherence
// Porting note: Full implementations of CGPGroup and CRMInstance are in their respective modules

extern "C" {
    // Stubs for C library functions
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...) -> c_int;
    fn Com_ParseTextFile(filename: *const c_char, parser: *mut CGPGroup) -> bool;
    fn Com_Printf(fmt: *const c_char, ...);
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
}

// LOCAL STUB: Forward declarations for types from other modules
pub struct CGPGroup {
    _opaque: [u8; 0],
}

pub struct CRMInstance {
    _opaque: [u8; 0],
}

pub struct CRMBSPInstance {
    _opaque: [u8; 0],
}

pub struct CRMGroupInstance {
    _opaque: [u8; 0],
}

pub struct CRMRandomInstance {
    _opaque: [u8; 0],
}

pub struct CRMVoidInstance {
    _opaque: [u8; 0],
}

extern "C" {
    fn CRMBSPInstance_new(group: *mut CGPGroup, instance_file: *mut CRMInstanceFile) -> *mut CRMBSPInstance;
    fn CRMGroupInstance_new(group: *mut CGPGroup, instance_file: *mut CRMInstanceFile) -> *mut CRMGroupInstance;
    fn CRMRandomInstance_new(group: *mut CGPGroup, instance_file: *mut CRMInstanceFile) -> *mut CRMRandomInstance;
    fn CRMVoidInstance_new(group: *mut CGPGroup, instance_file: *mut CRMInstanceFile) -> *mut CRMVoidInstance;
}

impl CGPGroup {
    fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        unimplemented!()
    }

    fn GetSubGroups(&mut self) -> *mut CGPGroup {
        unimplemented!()
    }

    fn GetNext(&self) -> *mut CGPGroup {
        unimplemented!()
    }

    fn FindPairValue(&self, key: *const c_char, default: *const c_char) -> *const c_char {
        unimplemented!()
    }

    fn GetName(&self) -> *const c_char {
        unimplemented!()
    }

    fn Clean(&mut self) {
        unimplemented!()
    }
}

impl CRMInstance {
    fn IsValid(&self) -> bool {
        unimplemented!()
    }
}

// MAX_QPATH constant - standard Q3 engine constant
const MAX_QPATH: usize = 256;

/************************************************************************************************
 *
 * RM_InstanceFile.cpp
 *
 * implements the CRMInstanceFile class.  This class provides functionality to load
 * and create instances from an instance file.  First call Open to open the instance file and
 * then use CreateInstance to create new instances.  When finished call Close to cleanup.
 *
 ************************************************************************************************/

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
pub struct CRMInstanceFile {
    mInstances: *mut CGPGroup,
    mParser: CGPGroup,
}

impl CRMInstanceFile {
    pub fn new() -> Self {
        CRMInstanceFile {
            mInstances: core::ptr::null_mut(),
            mParser: CGPGroup { _opaque: [] },
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
    // Rust note: Drop trait replaces C++ destructor
    pub fn close(&mut self) {
        // If not open then dont close  it
        if self.mInstances.is_null() {
            return;
        }
        self.mParser.Clean();

        self.mInstances = core::ptr::null_mut();
    }

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
    pub fn Open(&mut self, instance: *const c_char) -> bool {
        let mut instanceDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];

        // Build the filename
        unsafe {
            Com_sprintf(
                instanceDef.as_mut_ptr(),
                MAX_QPATH as c_int,
                b"ext_data/rmg/%s.instance\0".as_ptr() as *const c_char,
                instance,
            );
        }

        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            // Debug message
            unsafe {
                Com_Printf(
                    b"CM_Terrain: Loading and parsing instanceDef %s.....\n\0".as_ptr() as *const c_char,
                    instance,
                );
            }
        }

        // Parse the text file using the generic parser
        unsafe {
            if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser as *mut CGPGroup) {
                Com_sprintf(
                    instanceDef.as_mut_ptr(),
                    MAX_QPATH as c_int,
                    b"ext_data/arioche/%s.instance\0".as_ptr() as *const c_char,
                    instance,
                );
                if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser as *mut CGPGroup) {
                    Com_Printf(
                        va(
                            b"CM_Terrain: Could not open instance file '%s'\n\0".as_ptr() as *const c_char,
                            instanceDef.as_ptr(),
                        ),
                    );
                    return false;
                }
            }
        }

        // The whole file....
        let basegroup = unsafe { self.mParser.GetBaseParseGroup() };

        // The root { } struct
        self.mInstances = unsafe { (*basegroup).GetSubGroups() };

        // The "instances" { } structure
        self.mInstances = unsafe { (*self.mInstances).GetSubGroups() };

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
    pub fn Close(&mut self) {
        // If not open then dont close  it
        if self.mInstances.is_null() {
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
    pub fn CreateInstance(&mut self, name: *const c_char) -> *mut CRMInstance {
        static mut instanceID: c_int = 0;

        let mut group: *mut CGPGroup;
        let mut instance: *mut CRMInstance;

        // Make sure we were loaded
        assert!(!self.mInstances.is_null());

        // Search through the instances for the one with the given name
        group = self.mInstances;
        while !group.is_null() {
            unsafe {
                // Skip it if the name doesnt match
                if stricmp(name, (*group).FindPairValue(b"name\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char)) != 0 {
                    group = (*group).GetNext();
                    continue;
                }

                // Handle the various forms of instance types
                if stricmp((*group).GetName(), b"bsp\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMBSPInstance_new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
                }
                else if stricmp((*group).GetName(), b"npc\0".as_ptr() as *const c_char) == 0 {
                    //			instance = new CRMNPCInstance ( group, *this );
                    group = (*group).GetNext();
                    continue;
                }
                else if stricmp((*group).GetName(), b"group\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMGroupInstance_new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
                }
                else if stricmp((*group).GetName(), b"random\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMRandomInstance_new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
                }
                else if stricmp((*group).GetName(), b"void\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMVoidInstance_new(group, self as *mut CRMInstanceFile) as *mut CRMInstance;
                }
                else {
                    group = (*group).GetNext();
                    continue;
                }

                // If the instance isnt valid after being created then delete it
                if !(*instance).IsValid() {
                    // Porting note: C++ delete becomes explicit deallocation
                    // This assumes instance is heap-allocated
                    Box::from_raw(instance);
                    return core::ptr::null_mut();
                }

                // The instance was successfully created so return it
                return instance;
            }
        }

        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            // The instance wasnt found in the file so report it
            unsafe {
                Com_Printf(
                    va(
                        b"WARNING:  Instance '%s' was not found in the active instance file\n\0".as_ptr() as *const c_char,
                        name,
                    ),
                );
            }
        }

        core::ptr::null_mut()
    }
}

impl Drop for CRMInstanceFile {
    fn drop(&mut self) {
        self.close();
    }
}
