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

  FILE:		ImmCondition.h

  PURPOSE:	Immersion Foundation Classes Base Condition Effect

  STARTED:	Oct.10.97

  NOTES/REVISIONS:
     Mar.02.99 jrm (Jeff Mallett): Force-->Feel renaming
     Mar.02.99 jrm: Added GetIsCompatibleGUID
     Mar.15.99 jrm: __declspec(dllimport/dllexport) the whole class
     Nov.15.99 efw (Evan Wies): Converted to IFC

**********************************************************************/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char};

use crate::code::ff::IFC::ImmBaseTypes_h::*;   // ImmBaseTypes.h
use crate::code::ff::IFC::ImmEffect_h::*;       // ImmEffect.h  (DWORD, LONG, BOOL, POINT, GUID, CImmEffect, CImmDevice, LPDIEFFECT, IMM_EFFECT_* consts)
use crate::code::ff::IFC::FeelitAPI_h::*;        // real leaf for FEELIT_CONDITION / GUID_Feelit_*

// Windows stand-in (no crate mirror)
pub type TCHAR = c_char;
// Windows stand-in (no crate mirror)
pub const INFINITE: DWORD = 0xFFFFFFFF;

pub type LPCIMM_CONDITION = *const IMM_CONDITION;

//================================================================
// Constants
//================================================================

pub const IMM_CONDITION_PT_NULL: POINT = POINT { x: 0, y: 0 };

pub const IMM_CONDITION_DEFAULT_COEFFICIENT: LONG = 2500;
pub const IMM_CONDITION_DEFAULT_SATURATION: DWORD = 10000;
pub const IMM_CONDITION_DEFAULT_DEADBAND: LONG = 100;
pub const IMM_CONDITION_DEFAULT_CENTER_POINT: POINT = IMM_EFFECT_MOUSE_POS_AT_START;
pub const IMM_CONDITION_DEFAULT_DURATION: DWORD = INFINITE;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IC_ArgumentType {
    IC_NULL = 0,
    IC_POSITIVE_COEFFICIENT = 1,
    IC_NEGATIVE_COEFFICIENT = 2,
    IC_POSITIVE_SATURATION = 3,
    IC_NEGATIVE_SATURATION = 4,
    IC_DEAD_BAND = 5,
    IC_AXIS = 6,
    IC_CENTER = 7,
    IC_DIRECTION_X = 8,
    IC_DIRECTION_Y = 9,
    IC_ANGLE = 10,
    IC_CONDITION_X = 11,
    IC_CONDITION_Y = 12,
}

pub const IC_CONDITION: IC_ArgumentType = IC_ArgumentType::IC_CONDITION_X;

//================================================================
// CImmCondition
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmCondition {
    // Base class (CImmEffect) embedded
    // (Opaque for this faithful header port)

    //
    // INTERNAL DATA
    //
    pub m_aCondition: [IMM_CONDITION; 2],
    pub m_dwfAxis: DWORD,
    pub m_bUseMousePosAtStart: BOOL,

    //
    // PROTECTED DATA
    //
    pub m_bUseDeviceCoordinates: BOOL,
}

// Method declarations
// Note: These are declarations mirroring the C++ class interface.
// Actual implementations would be in a separate implementation file.

extern "C" {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    // Constructor
    pub fn CImmCondition_new(
        rguidEffect: *const GUID,
    ) -> *mut CImmCondition;

    // Destructor
    pub fn CImmCondition_drop(this: *mut CImmCondition);

    //
    // ATTRIBUTES
    //

    pub fn CImmCondition_GetIsCompatibleGUID(
        this: *mut CImmCondition,
        guid: *mut GUID,
    ) -> BOOL;

    pub fn CImmCondition_GetEffectType(
        this: *mut CImmCondition,
    ) -> DWORD;

    // Use this form for single-axis and dual-axis effects
    pub fn CImmCondition_ChangeConditionParams_XY(
        this: *mut CImmCondition,
        pConditionX: LPCIMM_CONDITION,
        pConditionY: LPCIMM_CONDITION,
    ) -> BOOL;

    // Use this form for directional effects
    pub fn CImmCondition_ChangeConditionParams_Dir(
        this: *mut CImmCondition,
        pCondition: LPCIMM_CONDITION,
        lDirectionX: LONG,
        lDirectionY: LONG,
    ) -> BOOL;

    // Use this form for directional effects
    pub fn CImmCondition_ChangeConditionParamsPolar(
        this: *mut CImmCondition,
        pCondition: LPCIMM_CONDITION,
        lAngle: LONG,
    ) -> BOOL;

    // Use this form for single-axis, dual-axis, or directional effects
    pub fn CImmCondition_ChangeConditionParamsX(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lDirectionX: LONG,
        lDirectionY: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeConditionParamsPolarX(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lAngle: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeConditionParamsY(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lDirectionX: LONG,
        lDirectionY: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeConditionParamsPolarY(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lAngle: LONG,
    ) -> BOOL;

    // Use this form for single-axis, dual-axis symetrical, or directional effects
    pub fn CImmCondition_ChangeConditionParams(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lDirectionX: LONG,
        lDirectionY: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeConditionParamsPolar_Sym(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lAngle: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveCoefficientX(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeCoefficientX(
        this: *mut CImmCondition,
        lNegativeCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveSaturationX(
        this: *mut CImmCondition,
        dwPositiveSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeSaturationX(
        this: *mut CImmCondition,
        dwNegativeSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeDeadBandX(
        this: *mut CImmCondition,
        lDeadBand: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveCoefficientY(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeCoefficientY(
        this: *mut CImmCondition,
        lNegativeCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveSaturationY(
        this: *mut CImmCondition,
        dwPositiveSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeSaturationY(
        this: *mut CImmCondition,
        dwNegativeSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeDeadBandY(
        this: *mut CImmCondition,
        lDeadBand: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveCoefficient(
        this: *mut CImmCondition,
        lPositiveCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeCoefficient(
        this: *mut CImmCondition,
        lNegativeCoefficient: LONG,
    ) -> BOOL;

    pub fn CImmCondition_ChangePositiveSaturation(
        this: *mut CImmCondition,
        dwPositiveSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeNegativeSaturation(
        this: *mut CImmCondition,
        dwNegativeSaturation: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_ChangeDeadBand(
        this: *mut CImmCondition,
        lDeadBand: LONG,
    ) -> BOOL;

    pub fn CImmCondition_SetCenter(
        this: *mut CImmCondition,
        pntCenter: POINT,
    ) -> BOOL;

    pub fn CImmCondition_ChangeConditionParams2(
        this: *mut CImmCondition,
        type_: IC_ArgumentType,
        // ... variadic arguments would follow
    ) -> BOOL;

    pub fn CImmCondition_GetPositiveCoefficientX(
        this: *mut CImmCondition,
        lPositiveCoefficient: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetNegativeCoefficientX(
        this: *mut CImmCondition,
        lNegativeCoefficient: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetPositiveSaturationX(
        this: *mut CImmCondition,
        dwPositiveSaturation: *mut DWORD,
    ) -> BOOL;

    pub fn CImmCondition_GetNegativeSaturationX(
        this: *mut CImmCondition,
        dwNegativeSaturation: *mut DWORD,
    ) -> BOOL;

    pub fn CImmCondition_GetDeadBandX(
        this: *mut CImmCondition,
        lDeadBand: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetPositiveCoefficientY(
        this: *mut CImmCondition,
        lPositiveCoefficient: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetNegativeCoefficientY(
        this: *mut CImmCondition,
        lNegativeCoefficient: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetPositiveSaturationY(
        this: *mut CImmCondition,
        dwPositiveSaturation: *mut DWORD,
    ) -> BOOL;

    pub fn CImmCondition_GetNegativeSaturationY(
        this: *mut CImmCondition,
        dwNegativeSaturation: *mut DWORD,
    ) -> BOOL;

    pub fn CImmCondition_GetDeadBandY(
        this: *mut CImmCondition,
        lDeadBand: *mut LONG,
    ) -> BOOL;

    pub fn CImmCondition_GetAxis(
        this: *mut CImmCondition,
        dwfAxis: *mut DWORD,
    ) -> BOOL;

    pub fn CImmCondition_GetCenter(
        this: *mut CImmCondition,
        pntCenter: *mut POINT,
    ) -> BOOL;

    //
    // OPERATIONS
    //

    pub fn CImmCondition_Initialize(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        effect: *const IMM_CONDITION,
        dwNoDownload: DWORD,
    ) -> BOOL;

    // Use this form for single-axis and dual-axis effects
    pub fn CImmCondition_InitCondition_XY(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        pConditionX: LPCIMM_CONDITION,
        pConditionY: LPCIMM_CONDITION,
        bUseDeviceCoordinates: BOOL,
        dwNoDownload: DWORD,
    ) -> BOOL;

    // Use this form for directional effects
    pub fn CImmCondition_InitCondition_Dir(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        pCondition: LPCIMM_CONDITION,
        lDirectionX: LONG,
        lDirectionY: LONG,
        bUseDeviceCoordinates: BOOL,
        dwNoDownload: DWORD,
    ) -> BOOL;

    // Use this form for directional effects
    pub fn CImmCondition_InitConditionPolar(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        pCondition: LPCIMM_CONDITION,
        lAngle: LONG,
        bUseDeviceCoordinates: BOOL,
        dwNoDownload: DWORD,
    ) -> BOOL;

    // Use this form for single-axis, dual-axis symetrical, or directional effects
    pub fn CImmCondition_InitCondition(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        dwfAxis: DWORD,
        pntCenter: POINT,
        lDirectionX: LONG,
        lDirectionY: LONG,
        bUseDeviceCoordinates: BOOL,
        dwNoDownload: DWORD,
    ) -> BOOL;

    // Use this form for directional effects
    pub fn CImmCondition_InitConditionPolar_Sym(
        this: *mut CImmCondition,
        pDevice: *mut CImmDevice,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        lAngle: LONG,
        bUseDeviceCoordinates: BOOL,
        dwNoDownload: DWORD,
    ) -> BOOL;

    pub fn CImmCondition_Start(
        this: *mut CImmCondition,
        dwIterations: DWORD,
        dwFlags: DWORD,
        bAllowStartDelayEmulation: BOOL,
    ) -> BOOL;

    //
    // HELPERS
    //

    pub fn CImmCondition_convert_line_point_to_offset(
        this: *mut CImmCondition,
        pntOnLine: POINT,
    );

    pub fn CImmCondition_set_parameters_XY(
        this: *mut CImmCondition,
        dwfAxis: DWORD,
        dwfCoordinates: DWORD,
        lDirection0: LONG,
        lDirection1: LONG,
        pConditionX: LPCIMM_CONDITION,
        pConditionY: LPCIMM_CONDITION,
    ) -> BOOL;

    pub fn CImmCondition_set_parameters(
        this: *mut CImmCondition,
        dwfAxis: DWORD,
        dwfCoordinates: DWORD,
        lDirection0: LONG,
        lDirection1: LONG,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
    ) -> BOOL;

    pub fn CImmCondition_change_parameters_Values(
        this: *mut CImmCondition,
        lDirection0: LONG,
        lDirection1: LONG,
        lPositiveCoefficient: LONG,
        lNegativeCoefficient: LONG,
        dwPositiveSaturation: DWORD,
        dwNegativeSaturation: DWORD,
        lDeadBand: LONG,
        pntCenter: POINT,
        fAxis: c_int,
    ) -> DWORD;

    pub fn CImmCondition_change_parameters_XY(
        this: *mut CImmCondition,
        lDirection0: LONG,
        lDirection1: LONG,
        pConditionX: LPCIMM_CONDITION,
        pConditionY: LPCIMM_CONDITION,
    ) -> DWORD;

    pub fn CImmCondition_buffer_ifr_data(
        this: *mut CImmCondition,
        pData: *mut TCHAR,
    ) -> c_int;

    pub fn CImmCondition_get_ffe_data(
        this: *mut CImmCondition,
        pdiEffect: *mut LPDIEFFECT,
    ) -> BOOL;
}

//
// INLINES
//

// Inline implementation of GetIsCompatibleGUID
#[inline]
pub fn CImmCondition_GetIsCompatibleGUID_inline(guid: *const GUID) -> BOOL {
    unsafe {
        (IsEqualGUID(guid, &GUID_Imm_Spring) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_DeviceSpring) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_Damper) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_Inertia) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_Friction) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_Texture) != 0 ||
         IsEqualGUID(guid, &GUID_Imm_Grid) != 0) as BOOL
    }
}

extern "C" {
    // Stub for IsEqualGUID from Windows API
    fn IsEqualGUID(guid1: *const GUID, guid2: *const GUID) -> c_int;
}
