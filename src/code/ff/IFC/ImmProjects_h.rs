/**********************************************************************
    Copyright (c) 1999 - 2000 Immersion Corporation

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

  FILE:		ImmProjects.h

  PURPOSE:	CImmProject
               Manages a set of forces in a project.
               There will be a project for each opened IFR file.
			CImmProjects
			   Manages a set of projects

  STARTED:	2/22/99 by Jeff Mallett


  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class

**********************************************************************/

use core::ffi::c_char;
use crate::code::ff::IFC::IFCErrors_h::*;
use crate::code::ff::IFC::ImmBaseTypes_h::*;
use crate::code::ff::IFC::ImmDevice_h::*;
use crate::code::ff::IFC::ImmCompoundEffect_h::*;

// ================================================================
// CImmProject
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmProject {
    // INTERNAL DATA
    // (accessed through public/protected interface below)
    m_hProj: HIFRPROJECT,
    m_dwProjectFileType: DWORD,
    m_pCreatedEffects: *mut CImmCompoundEffect,
    m_pDevice: *mut CImmDevice,
    m_piDI7: LPDIRECTINPUT,
    m_piDIDevice7: LPDIRECTINPUTDEVICE2,
    m_szProjectFileName: [TCHAR; MAX_PATH],
    m_nCreatedEffects: core::ffi::c_int,
    m_pNext: *mut CImmProject,
}

impl CImmProject {
    // CONSTRUCTOR/DESTRUCTOR

    pub fn new() -> Self {
        // Constructor initialization
        CImmProject {
            m_hProj: core::ptr::null_mut(),
            m_dwProjectFileType: 0,
            m_pCreatedEffects: core::ptr::null_mut(),
            m_pDevice: core::ptr::null_mut(),
            m_piDI7: core::ptr::null_mut(),
            m_piDIDevice7: core::ptr::null_mut(),
            m_szProjectFileName: [0; MAX_PATH],
            m_nCreatedEffects: 0,
            m_pNext: core::ptr::null_mut(),
        }
    }

    pub fn drop(&mut self) {
        // Destructor
    }

    pub fn Close(&mut self) {
    }

    // ATTRIBUTES

    pub fn GetDevice(&self) -> *mut CImmDevice {
        self.m_pDevice
    }

    pub fn GetIsOpen(&self) -> BOOL {
        if self.m_hProj.is_null() {
            0
        } else {
            1
        }
    }

    pub fn GetCreatedEffect(&self, lpszEffectName: *const c_char) -> *mut CImmCompoundEffect {
        // Method declaration
        core::ptr::null_mut()
    }

    pub fn GetCreatedEffectByIndex(&self, nIndex: core::ffi::c_int) -> *mut CImmCompoundEffect {
        // Method declaration (overload by index)
        core::ptr::null_mut()
    }

    pub fn GetNumEffectsFromIFR(&self) -> core::ffi::c_int {
        // Method declaration
        0
    }

    pub fn GetEffectNameFromIFRbyIndex(&self, nEffectIndex: core::ffi::c_int) -> *const c_char {
        // Method declaration
        core::ptr::null()
    }

    pub fn GetEffectSoundPathFromIFR(&self, lpszEffectName: *const c_char) -> *const c_char {
        // Method declaration
        core::ptr::null()
    }

    pub fn GetEffectType(&self, lpszEffectName: *const c_char) -> DWORD {
        // Method declaration
        0
    }

    pub fn GetEffectTypeFromIFRByName(&self, lpszEffectName: *const c_char) -> DWORD {
        // Method declaration (overload by name)
        0
    }

    pub fn GetEffectTypeFromIFRByIndex(&self, nEffectIndex: core::ffi::c_int) -> DWORD {
        // Method declaration (overload by index)
        0
    }

    pub fn GetNumCreatedEffects(&self) -> core::ffi::c_int {
        self.m_nCreatedEffects
    }

    // OPERATIONS

    pub fn Start(
        &mut self,
        lpszEffectName: *const c_char,
        dwIterations: DWORD,
        dwFlags: DWORD,
        pDevice: *mut CImmDevice,
    ) -> BOOL {
        // Method declaration
        0
    }

    pub fn Stop(&mut self, lpszEffectName: *const c_char) -> BOOL {
        // Method declaration
        0
    }

    pub fn OpenFile(&mut self, lpszFilePath: *const c_char, pDevice: *mut CImmDevice) -> BOOL {
        // Method declaration
        0
    }

    pub fn LoadProjectFromResource(
        &mut self,
        hRsrcModule: HMODULE,
        pRsrcName: *const c_char,
        pDevice: *mut CImmDevice,
    ) -> BOOL {
        // Method declaration
        0
    }

    pub fn LoadProjectFromMemory(&mut self, pProjectDef: *mut core::ffi::c_void, pDevice: *mut CImmDevice) -> BOOL {
        // Method declaration
        0
    }

    pub fn LoadProjectObjectPointer(&mut self, pMem: *mut u8, pDevice: *mut CImmDevice) -> BOOL {
        // Method declaration
        0
    }

    pub fn WriteToFile(&self, lpszFilename: *const c_char) -> BOOL {
        // Method declaration
        0
    }

    pub fn CreateEffect(
        &mut self,
        lpszEffectName: *const c_char,
        pDevice: *mut CImmDevice,
        dwNoDownload: DWORD,
    ) -> *mut CImmCompoundEffect {
        // Method declaration
        core::ptr::null_mut()
    }

    pub fn CreateEffectByIndex(
        &mut self,
        nEffectIndex: core::ffi::c_int,
        pDevice: *mut CImmDevice,
        dwNoDownload: DWORD,
    ) -> *mut CImmCompoundEffect {
        // Method declaration
        core::ptr::null_mut()
    }

    pub fn AddEffect(
        &mut self,
        lpszEffectName: *const c_char,
        pObject: GENERIC_EFFECT_PTR,
    ) -> *mut CImmCompoundEffect {
        // Method declaration
        core::ptr::null_mut()
    }

    // Conditional on IFC_VERSION >= 0x0101
    pub fn DestroyEffect(&mut self, pCompoundEffect: *mut CImmCompoundEffect) {
        // Method declaration (IFC_VERSION >= 0x0101)
    }

    // PRIVATE INTERFACE

    // HELPERS

    fn set_next(&mut self, pNext: *mut CImmProject) {
        self.m_pNext = pNext;
    }

    fn get_next(&self) -> *mut CImmProject {
        self.m_pNext
    }

    fn append_effect_to_list(&mut self, pEffect: *mut CImmCompoundEffect) {
        // Helper method declaration
    }

    // Conditional on IFC_VERSION >= 0x0101
    fn remove_effect_from_list(&mut self, pEffect: *mut CImmCompoundEffect) -> BOOL {
        // Helper method declaration (IFC_VERSION >= 0x0101)
        0
    }

    fn create_effect_structs(&self, lpszEffectName: *const c_char, nEff: &mut core::ffi::c_int) -> *mut *mut IFREffect {
        // Helper method declaration
        core::ptr::null_mut()
    }

    fn create_effect_structs_by_index(&self, nEffectIndex: core::ffi::c_int, nEff: &mut core::ffi::c_int) -> *mut *mut IFREffect {
        // Helper method declaration
        core::ptr::null_mut()
    }

    fn release_effect_structs(&self, hEffects: *mut *mut IFREffect) -> BOOL {
        // Helper method declaration
        0
    }
}

// ================================================================
// CImmProjects
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmProjects {
    // INTERNAL DATA
    m_pProjects: *mut CImmProject,
}

impl CImmProjects {
    // CONSTRUCTOR/DESTRUCTOR

    pub fn new() -> Self {
        CImmProjects {
            m_pProjects: core::ptr::null_mut(),
        }
    }

    pub fn drop(&mut self) {
        // Destructor
    }

    pub fn Close(&mut self) {
    }

    // ATTRIBUTES

    pub fn GetProject(&self, index: core::ffi::c_int) -> *mut CImmProject {
        // Method declaration
        core::ptr::null_mut()
    }

    // OPERATIONS

    pub fn Stop(&mut self) -> BOOL {
        // Method declaration
        0
    }

    pub fn OpenFile(&mut self, lpszFilePath: *const c_char, pDevice: *mut CImmDevice) -> core::ffi::c_long {
        // Method declaration
        0
    }

    pub fn LoadProjectFromResource(
        &mut self,
        hRsrcModule: HMODULE,
        pRsrcName: *const c_char,
        pDevice: *mut CImmDevice,
    ) -> core::ffi::c_long {
        // Method declaration
        0
    }

    pub fn LoadProjectFromMemory(&mut self, pProjectDef: *mut core::ffi::c_void, pDevice: *mut CImmDevice) -> core::ffi::c_long {
        // Method declaration
        0
    }
}
