/**********************************************************************
    Copyright (c) 1997 - 2000 Immersion Corporation

    Permission to use, copy, modify, distribute, and sell this
    software and its documentation may be granted without fee;
    interested parties are encouraged to request permission from
        Immersion Corporation
        801 Fox Lane
        San Jose, CA 95131
        408-467-1900

    IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
    INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
    IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
    CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
    LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
    NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
    CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

  FILE:        ImmBox.h

  PURPOSE:    Box Class for Immersion Foundation Classes

  STARTED:    11/04/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
     3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
     11/15/99 sdr (Steve Rank): Converted to IFC

**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_uint};

// ================================================================
// Stub types for unported dependencies
// ================================================================

/// Represents a Windows POINT structure (x, y coordinates)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}

/// Represents a Windows RECT structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RECT {
    pub left: c_long,
    pub top: c_long,
    pub right: c_long,
    pub bottom: c_long,
}

/// Type alias for pointer to RECT
pub type LPCRECT = *const RECT;

/// Boolean type (Windows BOOL)
pub type BOOL = c_int;

/// Windows DWORD
pub type DWORD = c_uint;

/// Windows LONG
pub type LONG = c_long;

/// Forward declaration for CImmEffect (from ImmEffect.h)
#[repr(C)]
pub struct CImmEffect {
    // Stub: actual fields defined in ImmEffect_h.rs
    _private: [u8; 0],
}

/// Forward declaration for CImmDevice (from ImmDevice)
#[repr(C)]
pub struct CImmDevice {
    // Stub: actual fields defined elsewhere
    _private: [u8; 0],
}

/// Forward declaration for CImmEnclosure (base class, from ImmEnclosure.h)
#[repr(C)]
pub struct CImmEnclosure {
    // Stub: actual fields defined in ImmEnclosure_h.rs
    _private: [u8; 0],
}

/// Stub for ImmBaseTypes constants
pub const MAXLONG: LONG = 2147483647i32;

pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;
pub const IMM_ENCLOSURE_HEIGHT_AUTO: DWORD = 0xFFFFFFFF;
pub const IMM_ENCLOSURE_WALL_WIDTH_AUTO: DWORD = 0xFFFFFFFF;
pub const IMM_ENCLOSURE_DEFAULT_WIDTH: DWORD = 0;
pub const IMM_ENCLOSURE_DEFAULT_HEIGHT: DWORD = 0;

// ================================================================
// Constants
// ================================================================

pub const IMM_BOX_MOUSE_POS_AT_START: POINT = POINT {
    x: MAXLONG,
    y: MAXLONG,
};

pub const IMM_BOX_DEFAULT_STIFFNESS: LONG = 5000;
pub const IMM_BOX_DEFAULT_WIDTH: DWORD = 10;
pub const IMM_BOX_DEFAULT_HEIGHT: DWORD = IMM_ENCLOSURE_HEIGHT_AUTO;
pub const IMM_BOX_DEFAULT_WALL_WIDTH: DWORD = IMM_ENCLOSURE_WALL_WIDTH_AUTO;

pub const IMM_BOX_DEFAULT_CENTER_POINT: POINT = IMM_BOX_MOUSE_POS_AT_START;

// ================================================================
// CImmBox
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Box Class for Immersion Foundation Classes
#[repr(C)]
pub struct CImmBox {
    // Base class: CImmEnclosure
    pub base: CImmEnclosure,
}

impl CImmBox {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    /// Constructor
    pub fn new() -> Self {
        CImmBox {
            base: CImmEnclosure { _private: [0; 0] },
        }
    }

    //
    // ATTRIBUTES
    //

    /// Change parameters using center point
    ///
    /// # Arguments
    /// * `pntCenter` - Center point of the box
    /// * `lStiffness` - Stiffness value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `dwWidth` - Width value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `dwHeight` - Height value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `dwWallWidth` - Wall width value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `pInsideEffect` - Inside effect pointer (default: null)
    pub fn ChangeParameters(
        &mut self,
        pntCenter: POINT,
        lStiffness: LONG,
        dwWidth: DWORD,
        dwHeight: DWORD,
        dwWallWidth: DWORD,
        pInsideEffect: *mut CImmEffect,
    ) -> BOOL {
        // Stub: to be implemented
        0
    }

    /// Change parameters using rectangle
    ///
    /// # Arguments
    /// * `pRectOutside` - Rectangle defining the box bounds
    /// * `lStiffness` - Stiffness value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `dwWallWidth` - Wall width value (default: IMM_EFFECT_DONT_CHANGE)
    /// * `pInsideEffect` - Inside effect pointer (default: null)
    pub fn ChangeParameters_Rect(
        &mut self,
        pRectOutside: LPCRECT,
        lStiffness: LONG,
        dwWallWidth: DWORD,
        pInsideEffect: *mut CImmEffect,
    ) -> BOOL {
        // Stub: to be implemented
        0
    }

    //
    // OPERATIONS
    //

    /// Initialize the box with given parameters using center point
    ///
    /// # Arguments
    /// * `pDevice` - Device pointer
    /// * `dwWidth` - Width (default: IMM_ENCLOSURE_DEFAULT_WIDTH)
    /// * `dwHeight` - Height (default: IMM_ENCLOSURE_DEFAULT_HEIGHT)
    /// * `lStiffness` - Stiffness (default: IMM_BOX_DEFAULT_STIFFNESS)
    /// * `dwWallWidth` - Wall width (default: IMM_BOX_DEFAULT_WALL_WIDTH)
    /// * `pntCenter` - Center point (default: IMM_BOX_DEFAULT_CENTER_POINT)
    /// * `pInsideEffect` - Inside effect (default: null)
    /// * `dwNoDownload` - No download flag (default: 0)
    pub fn Initialize(
        &mut self,
        pDevice: *mut CImmDevice,
        dwWidth: DWORD,
        dwHeight: DWORD,
        lStiffness: LONG,
        dwWallWidth: DWORD,
        pntCenter: POINT,
        pInsideEffect: *mut CImmEffect,
        dwNoDownload: DWORD,
    ) -> BOOL {
        // Stub: to be implemented
        0
    }

    /// Initialize the box with given parameters using rectangle
    ///
    /// # Arguments
    /// * `pDevice` - Device pointer
    /// * `pRectOutside` - Rectangle defining the box bounds
    /// * `lStiffness` - Stiffness (default: IMM_BOX_DEFAULT_STIFFNESS)
    /// * `dwWallWidth` - Wall width (default: IMM_BOX_DEFAULT_WALL_WIDTH)
    /// * `pInsideEffect` - Inside effect (default: null)
    /// * `dwNoDownload` - No download flag (default: 0)
    pub fn Initialize_Rect(
        &mut self,
        pDevice: *mut CImmDevice,
        pRectOutside: LPCRECT,
        lStiffness: LONG,
        dwWallWidth: DWORD,
        pInsideEffect: *mut CImmEffect,
        dwNoDownload: DWORD,
    ) -> BOOL {
        // Stub: to be implemented
        0
    }
}

//
// ------ PRIVATE INTERFACE ------
//
//
// HELPERS
//
// (none defined in the header)

//
// INTERNAL DATA
//
// (inherited from CImmEnclosure)
