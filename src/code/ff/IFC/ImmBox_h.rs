/* *********************************************************************
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

  FILE:		ImmBox.h

  PURPOSE:	Box Class for Immersion Foundation Classes

  STARTED:	11/04/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
	 11/15/99 sdr (Steve Rank): Converted to IFC

**********************************************************************/

#![allow(non_snake_case)]

// #include "ImmBaseTypes.h"
use crate::code::ff::IFC::ImmBaseTypes_h::*;
// #include "ImmEffect.h"
use crate::code::ff::IFC::ImmEffect_h::*;
// #include "ImmEnclosure.h"
use crate::code::ff::IFC::ImmEnclosure_h::*;


//================================================================
// Constants
//================================================================

pub const IMM_BOX_MOUSE_POS_AT_START: POINT = POINT { x: MAXLONG, y: MAXLONG };

pub const IMM_BOX_DEFAULT_STIFFNESS: LONG = 5000;
pub const IMM_BOX_DEFAULT_WIDTH: DWORD = 10;
// Porting note: IMM_ENCLOSURE_HEIGHT_AUTO is c_long in the ported ImmEnclosure_h; cast to DWORD
// to match the DWORD dwHeight parameter type used throughout ImmBox.h.
pub const IMM_BOX_DEFAULT_HEIGHT: DWORD = IMM_ENCLOSURE_HEIGHT_AUTO as DWORD;
pub const IMM_BOX_DEFAULT_WALL_WIDTH: DWORD = IMM_ENCLOSURE_WALL_WIDTH_AUTO as DWORD;

pub const IMM_BOX_DEFAULT_CENTER_POINT: POINT = IMM_BOX_MOUSE_POS_AT_START;


//================================================================
// CImmBox
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

// class DLLIFC CImmBox : public CImmEnclosure
#[repr(C)]
#[allow(non_snake_case)]
pub struct CImmBox {
    // C++ public inheritance: CImmEnclosure embedded as first field
    pub base: CImmEnclosure,
}

impl CImmBox {
    //
    // CONSTRUCTOR/DESCTRUCTOR
    //

    // Constructor
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    // Destructor
    pub fn delete(&mut self) {
        // virtual ~CImmBox
    }


    //
    // ATTRIBUTES
    //

    pub fn ChangeParameters(
        &mut self,
        pntCenter: POINT,
        lStiffness: LONG,     // = IMM_EFFECT_DONT_CHANGE
        dwWidth: DWORD,       // = IMM_EFFECT_DONT_CHANGE
        dwHeight: DWORD,      // = IMM_EFFECT_DONT_CHANGE
        dwWallWidth: DWORD,   // = IMM_EFFECT_DONT_CHANGE
        pInsideEffect: *mut CImmEffect, // = (CImmEffect*) IMM_EFFECT_DONT_CHANGE
    ) -> BOOL {
        0
    }

    // C++ overload: ChangeParameters(LPCRECT, LONG, DWORD, CImmEffect*)
    pub fn ChangeParameters_rect(
        &mut self,
        pRectOutside: LPCRECT,
        lStiffness: LONG,     // = IMM_EFFECT_DONT_CHANGE
        dwWallWidth: DWORD,   // = IMM_EFFECT_DONT_CHANGE
        pInsideEffect: *mut CImmEffect, // = (CImmEffect*) IMM_EFFECT_DONT_CHANGE
    ) -> BOOL {
        0
    }


    //
    // OPERATIONS
    //

    pub fn Initialize(
        &mut self,
        pDevice: *mut CImmDevice,
        dwWidth: DWORD,       // = IMM_ENCLOSURE_DEFAULT_WIDTH
        dwHeight: DWORD,      // = IMM_ENCLOSURE_DEFAULT_HEIGHT
        lStiffness: LONG,     // = IMM_BOX_DEFAULT_STIFFNESS
        dwWallWidth: DWORD,   // = IMM_BOX_DEFAULT_WALL_WIDTH
        pntCenter: POINT,     // = IMM_BOX_DEFAULT_CENTER_POINT
        pInsideEffect: *mut CImmEffect, // = NULL
        dwNoDownload: DWORD,  // = 0
    ) -> BOOL {
        0
    }

    // C++ overload: Initialize(CImmDevice*, LPCRECT, LONG, DWORD, CImmEffect*, DWORD)
    pub fn Initialize_rect(
        &mut self,
        pDevice: *mut CImmDevice,
        pRectOutside: LPCRECT,
        lStiffness: LONG,     // = IMM_BOX_DEFAULT_STIFFNESS
        dwWallWidth: DWORD,   // = IMM_BOX_DEFAULT_WALL_WIDTH
        pInsideEffect: *mut CImmEffect, // = NULL
        dwNoDownload: DWORD,  // = 0
    ) -> BOOL {
        0
    }
}


//
// ------ PRIVATE INTERFACE ------
//

    //
    // HELPERS
    //

    //
    // INTERNAL DATA
    //
