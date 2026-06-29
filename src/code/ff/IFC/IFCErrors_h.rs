/*
 * Copyright (c) 1999 - 2000 Immersion Corporation
 *
 * Permission to use, copy, modify, distribute, and sell this
 * software and its documentation may be granted without fee;
 * interested parties are encouraged to request permission from
 *     Immersion Corporation
 *     801 Fox Lane
 *     San Jose, CA 95131
 *     408-467-1900
 *
 * IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
 * INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
 * IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
 * CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
 * NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF THE
 * CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 *
 * FILE:		IFCErrors.h
 *
 * PURPOSE:	Error codes returned in IFC; Error handling in IFC
 *
 * STARTED:	2/28/99 by Jeff Mallett
 *
 * NOTES/REVISIONS:
 *    3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
 *    3/6/99 jrm: Added user error handling control
 *    3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
 *
 */

use core::ffi::{c_char, c_int, c_long, c_ulong};

/// Error codes returned in IFC
/// Original: typedef definition from <winerror.h>
pub type HRESULT = c_long;

/****************************************************************************
 *
 *      Error Codes
 *
 ****************************************************************************/

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IFC_ERROR_CODE {
    IFC_ERR_OK = 0,

    IFC_ERR_UNKNOWN_ERROR = 1,

    IFC_ERR_ALLOCATION_FAILED = 2,
    IFC_ERR_INVALID_PARAMETER = 3,
    IFC_ERR_NULL_PARAMETER = 4,
    IFC_ERR_WRONG_FORM = 5,

    IFC_ERR_DEVICE_IS_NULL = 6,
    IFC_ERR_INVALID_GUID = 7,
    IFC_ERR_EFFECT_NOT_INITIALIZED = 8,

    IFC_ERR_CANT_INITIALIZE_DEVICE = 9,

    IFC_ERR_CANT_CREATE_EFFECT = 10,
    IFC_ERR_CANT_CREATE_EFFECT_FROM_IFR = 11,
    IFC_ERR_NO_EFFECTS_FOUND = 12,
    IFC_ERR_EFFECT_IS_COMPOUND = 13,

    IFC_ERR_PROJECT_ALREADY_OPEN = 14,
    IFC_ERR_PROJECT_NOT_OPEN = 15,

    IFC_ERR_NO_DX7_DEVICE = 16,
    IFC_ERR_CANT_WRITE_IFR = 17,

    IFC_ERR_DINPUT_NOT_FOUND = 18,
    IFC_ERR_IMMAPI_NOT_FOUND = 19,

    IFC_ERR_FILE_NOT_FOUND = 20,
    IFC_ERR_NO_VERSION_INFO = 21,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IFC_ERROR_HANDLING_FLAGS {
    IFC_OUTPUT_ERR_TO_DEBUG = 0x0001,
    IFC_OUTPUT_ERR_TO_DIALOG = 0x0002,
}

/****************************************************************************
 *
 *      Macros
 *
 ****************************************************************************/

//
// ------ PUBLIC MACROS ------
//

// Original macros (translated to functions below):
// #define IFC_GET_LAST_ERROR			CIFCErrors::GetLastErrorCode()
// #define IFC_SET_ERROR_HANDLING		CIFCErrors::SetErrorHandling

//
// ------ PRIVATE MACROS ------
//

// Original macros:
// #if (IFC_VERSION >= 0x0110)
//  #define IFC_SET_ERROR(err)			CIFCErrors::SetErrorCode(err, __FILE__, __LINE__)
// #else
//  #define IFC_SET_ERROR(err)			CIFCErrors::SetErrorCode(err)
// #endif
// #define IFC_CLEAR_ERROR				IFC_SET_ERROR(IFC_ERR_OK)

/****************************************************************************
 *
 *      CIFCErrors
 *
 ****************************************************************************/
// All members are static.  Don't bother instantiating an object of this class.

//
// ------ PUBLIC INTERFACE ------
//

//
// ATTRIBUTES
//

static mut m_Err: HRESULT = 0;
static mut m_dwErrHandlingFlags: c_ulong = 0;

#[allow(non_snake_case)]
pub fn GetLastErrorCode() -> HRESULT {
    unsafe { m_Err }
}

#[allow(non_snake_case)]
pub fn SetErrorHandling(dwFlags: c_ulong) {
    unsafe {
        m_dwErrHandlingFlags = dwFlags;
    }
}

//
// ------ PRIVATE INTERFACE ------
//

// Internally used by IFC classes
#[allow(non_snake_case)]
pub fn SetErrorCode(err: HRESULT, _sFile: *const c_char, _nLine: c_int) {
    unsafe {
        m_Err = err;
    }
}

//
// HELPERS
//

//
// INTERNAL DATA
//
