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

  FILE:		ImmGrid.h

  PURPOSE:	Immersion Foundation Classes Grid Effect

  STARTED:	Dec.11.97

  NOTES/REVISIONS:
     Mar.02.99 jrm (Jeff Mallett): Force-->Feel renaming
     Mar.02.99 jrm: Added GetIsCompatibleGUID
     Mar.02.99 jrm: __declspec(dllimport/dllexport) the whole class
     Nov.15.99 efw (Evan Wies): Converted to IFC

**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_ulong};

// Windows type stubs for Immersion SDK header compatibility
pub type BOOL = c_int;
pub type DWORD = c_ulong;
pub type LONG = c_int;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GUID {
    pub Data1: c_ulong,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

// ================================================================
// Constants
// ================================================================

pub const IMM_GRID_DEFAULT_HORIZ_OFFSET: DWORD = 0;
pub const IMM_GRID_DEFAULT_VERT_OFFSET: DWORD = 0;
pub const IMM_GRID_DEFAULT_HORIZ_SPACING: DWORD = 100;
pub const IMM_GRID_DEFAULT_VERT_SPACING: DWORD = 100;
pub const IMM_GRID_DEFAULT_NODE_STRENGTH: LONG = 5000;
pub const IMM_GRID_DEFAULT_NODE_SATURATION: DWORD = 10000;

// ================================================================
// CImmGrid
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

// Forward declaration stubs for dependencies
// Note: These are stub forward declarations. In the full port,
// they would link to the actual ImmCondition.h and device structures.
pub struct CImmCondition;
pub struct CImmDevice;

// Note: GUID_Imm_Grid is a constant from the Immersion SDK.
// It is declared as an extern symbol for linking with the SDK.
extern "C" {
    pub static GUID_Imm_Grid: GUID;
}

// CImmGrid struct mirrors the C++ class CImmGrid : public CImmCondition
#[repr(C)]
pub struct CImmGrid {
    // Inherited from CImmCondition
    // Base class data would be laid out here in the actual implementation
}

//
// CONSTRUCTOR/DESTRUCTOR
//

// Constructor - Note: C++ constructors are typically called implicitly
// The implementation is external (in the .cpp file)

// Destructor - Note: marked as virtual in the original C++ class
// The implementation is external (in the .cpp file)

//
// ATTRIBUTES
//

impl CImmGrid {
    /// virtual BOOL GetIsCompatibleGUID(GUID &guid)
    /// Inline implementation: return IsEqualGUID(guid, GUID_Imm_Grid)
    #[inline]
    pub unsafe fn GetIsCompatibleGUID(&self, guid: &GUID) -> BOOL {
        Self::IsEqualGUID(guid, &GUID_Imm_Grid)
    }

    /// Helper: Check if two GUIDs are equal
    /// Implements: IsEqualGUID(guid1, guid2) from Windows SDK
    #[inline]
    fn IsEqualGUID(guid1: &GUID, guid2: &GUID) -> BOOL {
        if guid1.Data1 == guid2.Data1
            && guid1.Data2 == guid2.Data2
            && guid1.Data3 == guid2.Data3
            && guid1.Data4 == guid2.Data4
        {
            1 // TRUE
        } else {
            0 // FALSE
        }
    }
}

// External function declarations for CImmGrid methods
// These are C++ method implementations that would be in the .cpp file
extern "C" {
    /// BOOL ChangeParameters(
    ///     DWORD dwHorizSpacing,
    ///     DWORD dwVertSpacing = IMM_EFFECT_DONT_CHANGE,
    ///     LONG  lHorizNodeStrength = IMM_EFFECT_DONT_CHANGE,
    ///     LONG  lVertNodeStrength = IMM_EFFECT_DONT_CHANGE,
    ///     LONG  lHorizOffset = IMM_EFFECT_DONT_CHANGE,
    ///     LONG  lVertOffset = IMM_EFFECT_DONT_CHANGE,
    ///     DWORD dwHorizNodeSaturation = IMM_EFFECT_DONT_CHANGE,
    ///     DWORD dwVertNodeSaturation = IMM_EFFECT_DONT_CHANGE)
    /// Note: Default parameter values IMM_EFFECT_DONT_CHANGE are defined in ImmCondition.h
    pub fn CImmGrid_ChangeParameters(
        this: *mut CImmGrid,
        dwHorizSpacing: DWORD,
        dwVertSpacing: DWORD,
        lHorizNodeStrength: LONG,
        lVertNodeStrength: LONG,
        lHorizOffset: LONG,
        lVertOffset: LONG,
        dwHorizNodeSaturation: DWORD,
        dwVertNodeSaturation: DWORD,
    ) -> BOOL;

    /// BOOL ChangeHSpacing(DWORD dwHorizSpacing)
    pub fn CImmGrid_ChangeHSpacing(this: *mut CImmGrid, dwHorizSpacing: DWORD) -> BOOL;

    /// BOOL ChangeVSpacing(DWORD dwVertSpacing)
    pub fn CImmGrid_ChangeVSpacing(this: *mut CImmGrid, dwVertSpacing: DWORD) -> BOOL;

    /// BOOL ChangeHNodeStrength(LONG lHorizNodeStrength)
    pub fn CImmGrid_ChangeHNodeStrength(this: *mut CImmGrid, lHorizNodeStrength: LONG) -> BOOL;

    /// BOOL ChangeVNodeStrength(LONG lVertNodeStrength)
    pub fn CImmGrid_ChangeVNodeStrength(this: *mut CImmGrid, lVertNodeStrength: LONG) -> BOOL;

    /// BOOL ChangeOffset(POINT pntOffset)
    pub fn CImmGrid_ChangeOffset(this: *mut CImmGrid, pntOffset: POINT) -> BOOL;

    /// BOOL ChangeHNodeSaturation(DWORD dwHorizNodeSaturation)
    pub fn CImmGrid_ChangeHNodeSaturation(this: *mut CImmGrid, dwHorizNodeSaturation: DWORD) -> BOOL;

    /// BOOL ChangeVNodeSaturation(DWORD dwVertNodeSaturation)
    pub fn CImmGrid_ChangeVNodeSaturation(this: *mut CImmGrid, dwVertNodeSaturation: DWORD) -> BOOL;

    /// BOOL GetHSpacing(DWORD &dwHorizSpacing)
    pub fn CImmGrid_GetHSpacing(this: *const CImmGrid, dwHorizSpacing: *mut DWORD) -> BOOL;

    /// BOOL GetVSpacing(DWORD &dwVertSpacing)
    pub fn CImmGrid_GetVSpacing(this: *const CImmGrid, dwVertSpacing: *mut DWORD) -> BOOL;

    /// BOOL GetHNodeStrength(LONG &lHorizNodeStrength)
    pub fn CImmGrid_GetHNodeStrength(this: *const CImmGrid, lHorizNodeStrength: *mut LONG) -> BOOL;

    /// BOOL GetVNodeStrength(LONG &lVertNodeStrength)
    pub fn CImmGrid_GetVNodeStrength(this: *const CImmGrid, lVertNodeStrength: *mut LONG) -> BOOL;

    /// BOOL GetOffset(POINT &pntOffset)
    pub fn CImmGrid_GetOffset(this: *const CImmGrid, pntOffset: *mut POINT) -> BOOL;

    /// BOOL GetHNodeSaturation(DWORD &dwHorizNodeSaturation)
    pub fn CImmGrid_GetHNodeSaturation(this: *const CImmGrid, dwHorizNodeSaturation: *mut DWORD) -> BOOL;

    /// BOOL GetVNodeSaturation(DWORD &dwVertNodeSaturation)
    pub fn CImmGrid_GetVNodeSaturation(this: *const CImmGrid, dwVertNodeSaturation: *mut DWORD) -> BOOL;
}

//
// OPERATIONS
//

extern "C" {
    /// virtual BOOL Initialize(
    ///     CImmDevice* pDevice,
    ///     DWORD dwHorizSpacing = IMM_GRID_DEFAULT_HORIZ_SPACING,
    ///     DWORD dwVertSpacing = IMM_GRID_DEFAULT_VERT_SPACING,
    ///     LONG  lHorizNodeStrength = IMM_GRID_DEFAULT_NODE_STRENGTH,
    ///     LONG  lVertNodeStrength = IMM_GRID_DEFAULT_NODE_STRENGTH,
    ///     DWORD dwHorizOffset = IMM_GRID_DEFAULT_HORIZ_OFFSET,
    ///     DWORD dwVertOffset = IMM_GRID_DEFAULT_VERT_OFFSET,
    ///     DWORD dwHorizNodeSaturation = IMM_GRID_DEFAULT_NODE_SATURATION,
    ///     DWORD dwVertNodeSaturation = IMM_GRID_DEFAULT_NODE_SATURATION,
    ///     DWORD dwNoDownload = 0)
    /// Note: Default parameter values are shown in comments; Rust functions cannot have default parameters
    pub fn CImmGrid_Initialize(
        this: *mut CImmGrid,
        pDevice: *mut CImmDevice,
        dwHorizSpacing: DWORD,
        dwVertSpacing: DWORD,
        lHorizNodeStrength: LONG,
        lVertNodeStrength: LONG,
        dwHorizOffset: DWORD,
        dwVertOffset: DWORD,
        dwHorizNodeSaturation: DWORD,
        dwVertNodeSaturation: DWORD,
        dwNoDownload: DWORD,
    ) -> BOOL;
}

//
// INLINES
//

/// inline BOOL CImmGrid::GetIsCompatibleGUID(GUID &guid)
/// { return IsEqualGUID(guid, GUID_Imm_Grid); }
#[inline]
pub unsafe fn CImmGrid_GetIsCompatibleGUID(this: *const CImmGrid, guid: &GUID) -> BOOL {
    (*this).GetIsCompatibleGUID(guid)
}
