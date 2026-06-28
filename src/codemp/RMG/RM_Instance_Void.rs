//! Mechanical port of `codemp/RMG/RM_Instance_Void.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================
//
// These types are declared here to allow this file to compile structurally.
// Full definitions exist in the oracle but have not yet been ported.
// Porting these types is out of scope for this file.

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs used during instance construction.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *default_val)`.
    /// Returns the value string associated with the given key, or default if not found.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        // Porting stub: in reality, this looks up the key in internal storage
        // and returns the value or the default. For now, return the default.
        default_val
    }
}

/// Stub for unported `class CRMInstanceFile` (RM_InstanceFile.h).
/// Reference to an open instance file for creating sub-instances.
pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMArea` (RM_Area.h).
/// Represents an area in the map.
pub struct CRMArea {
    _opaque: [u8; 0],
}

impl CRMArea {
    /// Stub for `void CRMArea::EnableCollision(bool enabled)`.
    /// Enable or disable collision for this area.
    pub fn EnableCollision(&self, _enabled: bool) {
        // Porting stub: in reality, this toggles collision on the area.
    }
}

/// Stub for unported `class CRMAreaManager` (RM_Area.h).
/// Manages multiple areas in the map.
pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

// ============================================================================
// extern "C" functions from libc
// ============================================================================

extern "C" {
    /// C standard library function to convert a string to a double.
    fn atof(s: *const c_char) -> f64;
}

// ============================================================================
// CRMInstance stub
// ============================================================================
//
// The parent class `CRMInstance` is defined in RM_Instance.h.
// Its full Rust port is out of scope for this file; we provide a minimal
// interface here to allow the child class to call parent methods.

/// Stub for unported `class CRMInstance` (RM_Instance.h).
/// Base class for all instances in the random map generation system.
pub struct CRMInstance {
    /// Spacing radius for this instance
    pub mSpacingRadius: f64,
    /// Flatten radius for this instance
    pub mFlattenRadius: f64,
    // (other fields omitted for this stub)
}

impl CRMInstance {
    /// Stub for parent constructor `CRMInstance::CRMInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Initializes the base instance with default values.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> Self {
        CRMInstance {
            mSpacingRadius: 0.0,
            mFlattenRadius: 0.0,
        }
    }

    /// Stub for parent `void CRMInstance::SetArea(CRMAreaManager*, CRMArea*)`.
    /// Sets the area for this instance. Override in child classes as needed.
    pub fn SetArea(&mut self, _amanager: *const CRMAreaManager, _area: *const CRMArea) {
        // In the parent class, this just assigns mArea = area.
        // Child classes override to add additional behavior.
    }
}

// ============================================================================
// CRMVoidInstance
// ============================================================================

/// Implements the CRMVoidInstance class. This class just adds a void into the
/// area manager to help space things out.
pub struct CRMVoidInstance {
    /// Inherited fields from CRMInstance
    base: CRMInstance,
}

impl CRMVoidInstance {
    /// Constructs a void instance.
    ///
    /// # Arguments
    /// * `instGroup` - parser group containing information about this instance
    /// * `instFile` - reference to an open instance file for creating sub-instances
    pub fn new(instGroup: *const CGPGroup, instFile: *const CRMInstanceFile) -> Self {
        // Call parent constructor
        let mut instance = CRMVoidInstance {
            base: CRMInstance::new(instGroup, instFile),
        };

        // Safety: instGroup is assumed to be a valid pointer (passed from caller).
        // The FindPairValue call returns a pointer to a C string from the parser group,
        // which remains valid for the duration of this constructor call.
        unsafe {
            if !instGroup.is_null() {
                let instGroup_ref = &*instGroup;

                // Read spacing radius from config, convert via atof, defaulting to "0"
                let spacing_str = instGroup_ref.FindPairValue(
                    c"spacing".as_ptr(),
                    c"0".as_ptr(),
                );
                instance.base.mSpacingRadius = atof(spacing_str);

                // Read flatten radius from config, convert via atof, defaulting to "0"
                let flatten_str = instGroup_ref.FindPairValue(
                    c"flatten".as_ptr(),
                    c"0".as_ptr(),
                );
                instance.base.mFlattenRadius = atof(flatten_str);
            }
        }

        instance
    }

    /// Overridden to make sure the void area doesn't collide.
    /// Disables collision for the given area, then calls parent SetArea.
    ///
    /// # Arguments
    /// * `amanager` - area manager
    /// * `area` - area to set
    pub fn SetArea(&mut self, amanager: *const CRMAreaManager, area: *const CRMArea) {
        // Disable collision
        // Safety: area is assumed to be a valid pointer (passed from caller).
        // This mirrors the C++ call `area->EnableCollision(false)`.
        unsafe {
            if !area.is_null() {
                (*(area as *mut CRMArea)).EnableCollision(false);
            }
        }

        // Do what really needs to get done
        self.base.SetArea(amanager, area);
    }
}
