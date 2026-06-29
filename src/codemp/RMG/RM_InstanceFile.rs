//! Mechanical port of `codemp/RMG/RM_InstanceFile.cpp`.
//!
//! Implements the CRMInstanceFile class. This class provides functionality to load
//! and create instances from an instance file. First call Open to open the instance file and
//! then use CreateInstance to create new instances. When finished call Close to cleanup.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================
//
// These types are declared here to allow this file to compile structurally.
// Full definitions exist in the oracle but have not yet been ported.
// Porting these types is out of scope for this file.

/// Stub for unported `class CGenericParser2` (GenericParser2.h).
/// Parser for configuration files.
pub struct CGenericParser2 {
    _opaque: [u8; 0],
}

impl CGenericParser2 {
    /// Stub for `CGPGroup* CGenericParser2::GetBaseParseGroup()`.
    /// Returns the root parsing group.
    pub fn GetBaseParseGroup(&mut self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `void CGenericParser2::Clean()`.
    /// Cleans up the parser.
    pub fn Clean(&mut self) {
        // Porting stub: cleans up parser state.
    }
}

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::GetName()`.
    /// Returns the name of this group.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `CGPGroup* CGPGroup::GetSubGroups()`.
    /// Returns the first subgroup.
    pub fn GetSubGroups(&mut self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `CGPGroup* CGPGroup::GetNext()`.
    /// Returns the next sibling group.
    pub fn GetNext(&mut self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *defaultVal)`.
    /// Returns the value string associated with the given key, or default if not found.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        // Porting stub: in reality, this looks up the key in internal storage
        // and returns the value or the default. For now, return the default.
        default_val
    }
}

/// Stub for unported `class CRMInstance` (RM_Instance.h).
/// Base class for all instances in the random map generation system.
pub struct CRMInstance {
    _opaque: [u8; 0],
}

impl CRMInstance {
    /// Stub for `bool CRMInstance::IsValid()`.
    /// Returns true if the instance is valid.
    pub fn IsValid(&self) -> bool {
        true // Porting stub: default implementation returns true
    }
}

/// Stub for unported `class CRMBSPInstance` (RM_Instance_BSP.h).
/// Implements a BSP instance as part of the random map generation system.
pub struct CRMBSPInstance {
    _opaque: [u8; 0],
}

impl CRMBSPInstance {
    /// Stub for `CRMBSPInstance::CRMBSPInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Constructor for BSP instance.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMNPCInstance` (RM_Instance_NPC.h).
/// Implements an NPC instance as part of the random map generation system.
pub struct CRMNPCInstance {
    _opaque: [u8; 0],
}

impl CRMNPCInstance {
    /// Stub for `CRMNPCInstance::CRMNPCInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Constructor for NPC instance.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMGroupInstance` (RM_Instance_Group.h).
/// Implements a group instance as part of the random map generation system.
pub struct CRMGroupInstance {
    _opaque: [u8; 0],
}

impl CRMGroupInstance {
    /// Stub for `CRMGroupInstance::CRMGroupInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Constructor for group instance.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMRandomInstance` (RM_Instance_Random.h).
/// Implements a random instance as part of the random map generation system.
pub struct CRMRandomInstance {
    _opaque: [u8; 0],
}

impl CRMRandomInstance {
    /// Stub for `CRMRandomInstance::CRMRandomInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Constructor for random instance.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CRMVoidInstance` (RM_Instance_Void.h).
/// Implements a void instance as part of the random map generation system.
pub struct CRMVoidInstance {
    _opaque: [u8; 0],
}

impl CRMVoidInstance {
    /// Stub for `CRMVoidInstance::CRMVoidInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Constructor for void instance.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> *mut CRMInstance {
        core::ptr::null_mut()
    }
}

// ============================================================================
// Constants and types from qcommon
// ============================================================================

/// Maximum path length (from q_shared.h).
pub const MAX_QPATH: usize = 256;

// ============================================================================
// extern "C" function declarations
// ============================================================================

extern "C" {
    /// Quake engine function to format a string with variadic arguments.
    /// Stores up to size bytes of the formatted result into dest.
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);

    /// Quake engine function to print formatted messages to console.
    fn Com_Printf(fmt: *const c_char, ...);

    /// Quake engine function to parse a text file into a parser structure.
    /// Returns true if successful.
    fn Com_ParseTextFile(file: *const c_char, parser: *mut CGenericParser2) -> bool;

    /// Quake engine function to format a string with variadic arguments.
    /// Returns a pointer to a static buffer containing the formatted result.
    pub fn va(format: *const c_char, ...) -> *mut c_char;

    /// C standard library function for case-insensitive string comparison.
    /// Returns 0 if strings are equal (ignoring case).
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Assert macro: prints error if condition is false.
    fn assert(condition: c_int);
}

// ============================================================================
// CRMInstanceFile class implementation
// ============================================================================

/// Represents an open instance file for creating instances.
#[repr(C)]
pub struct CRMInstanceFile {
    /// Parser for the instance file
    pub mParser: CGenericParser2,
    /// Root instances group
    pub mInstances: *mut CGPGroup,
}

impl CRMInstanceFile {
    /// ========================================================================
    /// CRMInstanceFile::CRMInstanceFile
    ///     constructor
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn new() -> Self {
        Self {
            mParser: CGenericParser2 { _opaque: [] },
            mInstances: core::ptr::null_mut(),
        }
    }

    /// ========================================================================
    /// CRMInstanceFile::~CRMInstanceFile
    ///	Destroys the instance file by freeing the parser
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn delete(&mut self) {
        self.Close();
    }

    /// ========================================================================
    /// CRMInstanceFile::Open
    ///	Opens the given instance file and prepares it for use in instance creation
    ///
    /// inputs:
    ///  instance: Name of instance to open.  Note that the root path will be automatically
    ///			  added and shouldnt be included in the given name
    ///
    /// return:
    ///	true: instance file successfully loaded
    ///  false: instance file could not be loaded for some reason
    ///
    /// ========================================================================
    pub fn Open(&mut self, instance: *const c_char) -> bool {
        let mut instanceDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let basegroup: *mut CGPGroup;

        unsafe {
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
                Com_Printf(b"CM_Terrain: Loading and parsing instanceDef %s.....\n\0".as_ptr() as *const c_char, instance);
            }

            // Parse the text file using the generic parser
            if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser as *mut CGenericParser2) {
                Com_sprintf(
                    instanceDef.as_mut_ptr(),
                    MAX_QPATH as c_int,
                    b"ext_data/arioche/%s.instance\0".as_ptr() as *const c_char,
                    instance,
                );
                if !Com_ParseTextFile(instanceDef.as_ptr(), &mut self.mParser as *mut CGenericParser2) {
                    Com_Printf(
                        va(b"CM_Terrain: Could not open instance file '%s'\n\0".as_ptr() as *const c_char, instanceDef.as_ptr()),
                    );
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
    }

    /// ========================================================================
    /// CRMInstanceFile::Close
    ///	Closes an open instance file
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///	none
    ///
    /// ========================================================================
    pub fn Close(&mut self) {
        // If not open then dont close  it
        if self.mInstances == core::ptr::null_mut() {
            return;
        }
        self.mParser.Clean();

        self.mInstances = core::ptr::null_mut();
    }

    /// ========================================================================
    /// CRMInstanceFile::CreateInstance
    ///	Creates an instance (to be freed by caller) using the given instance name.
    ///
    /// inputs:
    ///  name: Name of the instance to read from the instance file
    ///
    /// return:
    ///	NULL: instance could not be read from the instance file
    ///  NON-NULL: instance created and returned for further use
    ///
    /// ========================================================================
    pub fn CreateInstance(&mut self, name: *const c_char) -> *mut CRMInstance {
        static mut instanceID: c_int = 0;

        let mut group: *mut CGPGroup;
        let mut instance: *mut CRMInstance;

        unsafe {
            // Make sure we were loaded
            assert(if self.mInstances == core::ptr::null_mut() { 0 } else { 1 });

            // Search through the instances for the one with the given name
            group = self.mInstances;
            while !group.is_null() {
                // Skip it if the name doesnt match
                if stricmp(name, (*group).FindPairValue(b"name\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char)) != 0 {
                    group = (*group).GetNext();
                    continue;
                }

                // Handle the various forms of instance types
                if stricmp((*group).GetName(), b"bsp\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMBSPInstance::new(group as *const CGPGroup, self as *const CRMInstanceFile);
                } else if stricmp((*group).GetName(), b"npc\0".as_ptr() as *const c_char) == 0 {
                    //			instance = new CRMNPCInstance ( group, *this );
                    group = (*group).GetNext();
                    continue;
                } else if stricmp((*group).GetName(), b"group\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMGroupInstance::new(group as *const CGPGroup, self as *const CRMInstanceFile);
                } else if stricmp((*group).GetName(), b"random\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMRandomInstance::new(group as *const CGPGroup, self as *const CRMInstanceFile);
                } else if stricmp((*group).GetName(), b"void\0".as_ptr() as *const c_char) == 0 {
                    instance = CRMVoidInstance::new(group as *const CGPGroup, self as *const CRMInstanceFile);
                } else {
                    group = (*group).GetNext();
                    continue;
                }

                // If the instance isnt valid after being created then delete it
                if !(*instance).IsValid() {
                    // delete instance;
                    return core::ptr::null_mut();
                }

                // The instance was successfully created so return it
                return instance;
            }

            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                // The instance wasnt found in the file so report it
                Com_Printf(
                    va(b"WARNING:  Instance '%s' was not found in the active instance file\n\0".as_ptr() as *const c_char, name),
                );
            }

            core::ptr::null_mut()
        }
    }
}
