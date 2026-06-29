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

  FILE:		FEELitIFR.h

  PURPOSE:	Input/Output for IFR Files, FEELit version

  STARTED:

  NOTES/REVISIONS:

**********************************************************************/

use core::ffi::{c_char, c_int, c_uint, c_void};

// IFRAPI was __stdcall (Windows calling convention)
// DLLAPI was __declspec(dllimport) or __declspec(dllexport) depending on _IFCDLL_

/*
**  CONSTANTS
*/

/*
**  RT_IMMERSION - Resource type for IFR projects stored as resources.
**   This is the resource type looked for by IFLoadProjectResource().
*/
pub const RT_IMMERSION: *const c_char = b"IMMERSION\0".as_ptr() as *const c_char;

/*
**  TYPES/STRUCTURES
*/

/*
**  HIFRPROJECT - used to identify a loaded project as a whole.
**   individual objects within a project are uniquely referenced by name.
**   Created by the IFLoadProject*() functions and released by IFReleaseProject().
*/
pub type HIFRPROJECT = *mut c_void;

// Forward declarations for external types from DirectInput/dinput.h
pub type HMODULE = *mut c_void;
pub type LPIIMM_DEVICE = *mut c_void;
pub type LPIMM_EFFECT = *mut c_void;

/*
**  IFREffect - contains the information needed to create a DI effect
**   using IDirectInputEffect::CreateEffect(). An array of pointers to these
**	 structures is allocated and returned by IFCreateEffectStructs().
*/
#[repr(C)]
pub struct IFREffect {
    pub guid: [c_char; 16], // GUID is 16 bytes (4 DWORD + 2 WORD + 8 BYTE)
    pub dwIterations: c_uint,
    pub effectName: *mut c_char,
    pub lpDIEffect: LPIMM_EFFECT,
}

/*
**  FUNCTION DECLARATIONS
*/

/*
**  IFLoadProjectResource() - Load a project from a resource.
**   hRsrcModule - handle of the module containing the project definition resource.
**   pRsrcName - name or MAKEINTRESOURCE(id) identifier of resource to load.
**   pDevice - device for which the project is being loaded. If NULL,
**     effects will be created generically, and IFCreateEffects() will fail.
**   Returns an identifier for the loaded project, or NULL if unsuccessful.
*/
extern "C" {
    pub fn IFRLoadProjectResource(
        hRsrcModule: HMODULE,
        pRsrcName: *const c_char,
        pDevice: LPIIMM_DEVICE,
    ) -> HIFRPROJECT;
}

/*
**  IFLoadProjectPointer() - Load a project from a pointer.
**   pProject - points to a project definition.
**   pDevice - device for which the project is being loaded. If NULL,
**     effects will be created generically, and IFCreateEffects() will fail.
**   Returns an identifier for the loaded project, or NULL if unsuccessful.
*/
extern "C" {
    pub fn IFRLoadProjectPointer(
        pProject: *mut c_void,
        pDevice: LPIIMM_DEVICE,
    ) -> HIFRPROJECT;
}

/*
**  IFLoadProjectFile() - Load a project from a file.
**    pProjectFileName - points to a project file name.
**    pDevice - device for which the project is being loaded. If NULL,
**       effects will be created generically, and IFCreateEffects() will fail.
**    Returns an identifier for the loaded project, or NULL if unsuccessful.
*/
extern "C" {
    pub fn IFRLoadProjectFile(
        pProjectFileName: *const c_char,
        pDevice: LPIIMM_DEVICE,
    ) -> HIFRPROJECT;
}

/*
**  IFRLoadProjectFromMemory() - Load a project from memory.
**
**    In cases where a file or resource is readily accessible, it may
**	  be necessary to pass IFR formated information through memory.
**
**    pProjectDef - memory addres that contains information from an IFR file.
**    pDevice - device for which the project is being loaded. If NULL,
**       effects will be created generically, and IFRCreateEffects() will fail.
**    Returns an identifier for the loaded project, or NULL if unsuccessful.
*/
extern "C" {
    pub fn IFRLoadProjectFromMemory(
        pProjectDef: *mut c_void,
        pDevice: LPIIMM_DEVICE,
    ) -> HIFRPROJECT;
}

/*
**  IFLoadProjectObjectPointer() - Load a project from a pointer to a single
**     object definition (usually used only by the editor).
**   pObject - points to an object definition.
**   pDevice - device for which the project is being loaded. If NULL,
**     effects will be created generically, and IFCreateEffects() will fail.
**   Returns an identifier for the loaded project, or NULL if unsuccessful.
*/
extern "C" {
    pub fn IFRLoadProjectObjectPointer(
        pObject: *mut c_void,
        pDevice: LPIIMM_DEVICE,
    ) -> HIFRPROJECT;
}

/*
**  IFReleaseProject() - Release a loaded project.
**   hProject - identifies the project to be released.
**   Returns TRUE if the project is released, FALSE if it is an invalid project.
*/
extern "C" {
    pub fn IFRReleaseProject(hProject: HIFRPROJECT) -> c_int;
}

/*
**  IFCreateEffectStructs() - Create IFREffects for a named effect.
**   hProject - identifies the project containing the object.
**   pObjectName - name of the object for which to create structures.
**   pNumEffects - if not NULL will be set to a count of the IFREffect
**     structures in the array (not including the terminating NULL pointer.)
**   Returns a pointer to the allocated array of pointers to IFREffect
**     structures. The array is terminated with a NULL pointer. If the
**     function fails, a NULL pointer is returned.
*/
extern "C" {
    pub fn IFRCreateEffectStructs(
        hProject: HIFRPROJECT,
        pObjectName: *const c_char,
        pNumEffects: *mut c_int,
    ) -> *mut *mut IFREffect;
}

extern "C" {
    pub fn IFRCreateEffectStructsByIndex(
        hProject: HIFRPROJECT,
        nObjectIndex: c_int,
        pNumEffects: *mut c_int,
    ) -> *mut *mut IFREffect;
}

extern "C" {
    pub fn IFRGetNumEffects(hProject: HIFRPROJECT) -> c_int;
}

extern "C" {
    pub fn IFRGetObjectNameByIndex(
        hProject: HIFRPROJECT,
        nObjectIndex: c_int,
    ) -> *const c_char;
}

extern "C" {
    pub fn IFRGetObjectSoundPath(
        hProject: HIFRPROJECT,
        pObjectName: *const c_char,
    ) -> *const c_char;
}

extern "C" {
    pub fn IFRGetObjectType(
        hProject: HIFRPROJECT,
        pObjectName: *const c_char,
    ) -> c_uint;
}

extern "C" {
    pub fn IFRGetObjectTypeByIndex(
        hProject: HIFRPROJECT,
        nObjectIndex: c_int,
    ) -> c_uint;
}

extern "C" {
    pub fn IFRGetObjectNameByGUID(
        hProject: HIFRPROJECT,
        pGUID: *mut [c_char; 16],
    ) -> *const c_char;
}

extern "C" {
    pub fn IFRGetObjectID(
        hProject: HIFRPROJECT,
        pObjectName: *const c_char,
    ) -> [c_char; 16];
}

extern "C" {
    pub fn IFRGetContainedObjIDs(
        hProject: HIFRPROJECT,
        pCompoundObjName: *const c_char,
    ) -> *mut [c_char; 16];
}

/*
**  IFReleaseEffectStructs() - Release an array of IFREffects.
**   hProject - identifies the project for which the effects were created.
**   pEffects - points to the array of IFREffect pointers to be released.
**   Returns TRUE if the array is released, FALSE if it is an invalid array.
*/
extern "C" {
    pub fn IFRReleaseEffectStructs(
        hProject: HIFRPROJECT,
        pEffects: *mut *mut IFREffect,
    ) -> c_int;
}

/*
**  IFCreateEffects() - Creates the DirectInput effects using
**     IDirectInput::CreateEffect().
**   hProject - identifies the project containing the object.
**   pObjectName - name of the object for which to create effects.
**   pNumEffects - if not NULL will be set to a count of the IDirectInputEffect
**     pointers in the array (not including the terminating NULL pointer.)
**   Returns a pointer to the allocated array of pointers to IDirectInputEffects.
**     The array is terminated with a NULL pointer. If the function fails,
**     a NULL pointer is returned.
*/
extern "C" {
    pub fn IFRCreateEffects(
        hProject: HIFRPROJECT,
        pObjectName: *const c_char,
        pNumEffects: *mut c_int,
    ) -> *mut LPIMM_EFFECT;
}

/*
**  IFReleaseEffects() - Releases an array of IDirectInputEffect structures.
**   hProject - identifies the project for which the effects were created.
**   pEffects - points to the array if IDirectInputEffect pointers to be released.
**   Returns TRUE if the array is released, FALSE if it is an invalid array.
*/
extern "C" {
    pub fn IFRReleaseEffects(
        hProject: HIFRPROJECT,
        pEffects: *mut LPIMM_EFFECT,
    ) -> c_int;
}
