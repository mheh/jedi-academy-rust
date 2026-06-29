// Copyright (C) 1999-2000 Id Software, Inc.
//
// RoffSystem.cpp

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use std::collections::HashMap;

// Re-export types from q_shared
pub type qboolean = c_int;
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub const QTRUE: c_int = 1;
pub const QFALSE: c_int = 0;
pub const MAX_QPATH: usize = 64;

// ============================================================================
// ROFF Defines
// ============================================================================
pub const ROFF_VERSION: c_int = 1;
pub const ROFF_NEW_VERSION: c_int = 2;
pub const ROFF_STRING: &[u8] = b"ROFF";
pub const ROFF_SAMPLE_RATE: c_int = 10; // 10hz

extern "C" {
    // String functions
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // File system
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);

    // Misc
    fn Com_Printf(msg: *const c_char, ...);
    fn COM_StripExtension(input: *const c_char, output: *mut c_char);
    fn va(format: *const c_char, ...) -> *const c_char;

    // Vector math
    fn VectorCopy(in_: *const vec3_t, out: *mut vec3_t);
    fn VectorScale(in_: *const vec3_t, scale: vec_t, out: *mut vec3_t);
    fn VectorMA(veca: *const vec3_t, scale: vec_t, vecb: *const vec3_t, vecc: *mut vec3_t);
    fn VectorClear(v: *mut vec3_t);
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);

    // Server
    fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;

    // VM
    fn VM_Call(vm: *mut c_void, callnum: c_int, ...) -> c_int;

    // Server state (svs)
    // NOTE: This is a partial declaration - svs is a complex struct, only used for svs.time
    pub static svs: serverStatic_t;
    pub static cgvm: *mut c_void; // client game virtual machine
    pub static gvm: *mut c_void; // server game virtual machine
}

#[repr(C)]
pub struct serverStatic_t {
    pub time: c_int,
    // ... other fields omitted
}

// Trajectory types
#[repr(C)]
#[derive(Clone, Copy)]
pub enum trType_t {
    TR_STATIONARY = 0,
    TR_INTERPOLATE = 1,
    TR_LINEAR = 2,
    TR_LINEAR_STOP = 3,
    TR_NONLINEAR_STOP = 4,
    TR_SINE = 5,
    TR_GRAVITY = 6,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct trajectory_t {
    pub trType: trType_t,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

// Entity state
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    // ... rest of fields not needed for this port
}

// Entity shared
#[repr(C)]
pub struct entityShared_t {
    pub linked: qboolean,
    pub linkcount: c_int,
    pub svFlags: c_int,
    pub singleClient: c_int,
    pub bmodel: qboolean,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub contents: c_int,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    pub mIsRoffing: qboolean,
    pub ownerNum: c_int,
    pub broadcastClients: [c_int; 2],
}

// Shared entity
#[repr(C)]
pub struct sharedEntity_t {
    pub s: entityState_t,
    pub playerState: *mut c_void,
    pub m_pVehicle: *mut c_void,
    pub ghoul2: *mut c_void,
    pub localAnimIndex: c_int,
    pub modelScale: vec3_t,
    pub r: entityShared_t,
    pub next_roff_time: c_int, //rww - npc's need to know when they're getting roff'd
    // ... other fields not needed for this port
}

// ============================================================================
// ROFF Types
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFFHeader {
    pub mHeader: [c_char; 4],
    pub mVersion: c_int,
    pub mCount: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFFEntry {
    pub mOriginOffset: vec3_t,
    pub mRotateOffset: vec3_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFF2Header {
    pub mHeader: [c_char; 4],
    pub mVersion: c_int,
    pub mCount: c_int,
    pub mFrameRate: c_int,
    pub mNumNotes: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFF2Entry {
    pub mOriginOffset: vec3_t,
    pub mRotateOffset: vec3_t,
    pub mStartNote: c_int,
    pub mNumNotes: c_int,
}

// ============================================================================
// SROFFEntity
// ============================================================================

pub struct SROFFEntity {
    pub mEntID: c_int,
    pub mROFFID: c_int,
    pub mNextROFFTime: c_int,
    pub mROFFFrame: c_int,
    pub mKill: qboolean,
    pub mSignal: qboolean,
    pub mTranslated: qboolean,
    pub mIsClient: qboolean,
    pub mStartAngles: vec3_t,
}

// ============================================================================
// CROFF Class
// ============================================================================

pub struct CROFF {
    pub mID: c_int,
    pub mROFFFilePath: [c_char; MAX_QPATH],
    pub mROFFEntries: c_int,
    pub mFrameTime: c_int,
    pub mLerp: c_int,
    pub mMoveRotateList: *mut TROFF2Entry,
    pub mNumNoteTracks: c_int,
    pub mNoteTrackIndexes: *mut *mut c_char,
    pub mUsedByClient: qboolean,
    pub mUsedByServer: qboolean,
}

impl CROFF {
    //---------------------------------------------------------------------------
    // CROFFSystem::CROFF::CROFF
    //	Simple constructor for CROFF object
    //
    // INPUTS:
    //	pass in the filepath and the id of the roff object to create
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn new(file: *const c_char, id: c_int) -> Self {
        let mut croff = CROFF {
            mID: id,
            mROFFFilePath: [0; MAX_QPATH],
            mROFFEntries: 0,
            mFrameTime: 0,
            mLerp: 0,
            mMoveRotateList: core::ptr::null_mut(),
            mNumNoteTracks: 0,
            mNoteTrackIndexes: core::ptr::null_mut(),
            mUsedByClient: QFALSE,
            mUsedByServer: QFALSE,
        };

        unsafe {
            strcpy(croff.mROFFFilePath.as_mut_ptr(), file);
        }

        croff
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::CROFF::~CROFF()
    //	Frees any resources when the CROFF object dies
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn drop(&mut self) {
        unsafe {
            if !self.mMoveRotateList.is_null() {
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(
                    self.mMoveRotateList,
                    self.mROFFEntries as usize,
                ));
            }

            if !self.mNoteTrackIndexes.is_null() {
                if !(*self.mNoteTrackIndexes).is_null() {
                    let _ = Box::from_raw(*self.mNoteTrackIndexes);
                }
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(
                    self.mNoteTrackIndexes,
                    self.mNumNoteTracks as usize,
                ));
            }
        }
    }
}

impl Drop for CROFF {
    fn drop(&mut self) {
        unsafe {
            if !self.mMoveRotateList.is_null() {
                let _ = Vec::from_raw_parts(
                    self.mMoveRotateList,
                    self.mROFFEntries as usize,
                    self.mROFFEntries as usize,
                );
            }

            if !self.mNoteTrackIndexes.is_null() {
                if !(*self.mNoteTrackIndexes).is_null() {
                    let _ = Vec::from_raw_parts(
                        *self.mNoteTrackIndexes as *mut u8,
                        0, // We don't track the size of data allocation
                        0,
                    );
                }
                let _ = Vec::from_raw_parts(
                    self.mNoteTrackIndexes,
                    self.mNumNoteTracks as usize,
                    self.mNumNoteTracks as usize,
                );
            }
        }
    }
}

// ============================================================================
// CROFFSystem
// ============================================================================

pub struct CROFFSystem {
    mROFFList: HashMap<c_int, Box<CROFF>>,
    mID: c_int,
    mROFFEntList: Vec<Box<SROFFEntity>>,
}

impl CROFFSystem {
    pub fn new() -> Self {
        CROFFSystem {
            mROFFList: HashMap::new(),
            mID: 0,
            mROFFEntList: Vec::new(),
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::Restart
    //	Cleans up the roff system, not sure how useful this really is
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	success or failure
    //---------------------------------------------------------------------------
    pub fn Restart(&mut self) -> qboolean {
        // remove everything from the list
        self.mROFFList.clear();

        // clear CROFFSystem unique ID counter
        self.mID = 0;

        return QTRUE;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::IsROFF
    //	Makes sure that the requested file is actually a ROFF
    //
    // INPUTS:
    //	pass in the file data
    //
    // RETURN:
    //	returns test success or failure
    //---------------------------------------------------------------------------
    pub fn IsROFF(&self, data: *mut c_void) -> qboolean {
        let hdr = data as *mut TROFFHeader;
        let hdr2 = data as *mut TROFF2Header;

        unsafe {
            let header_bytes = (*hdr).mHeader;
            let roff_string = ROFF_STRING;

            if strcmp(header_bytes.as_ptr(), roff_string.as_ptr() as *const c_char) == 0 {
                // bad header
                return QFALSE;
            }

            if (*hdr).mVersion != ROFF_VERSION && (*hdr).mVersion != ROFF_NEW_VERSION {
                // bad version
                return QFALSE;
            }

            if (*hdr).mVersion == ROFF_VERSION && (*hdr).mCount <= 0.0 {
                // bad count
                return QFALSE;
            }

            if (*hdr).mVersion == ROFF_NEW_VERSION && (*hdr2).mCount <= 0 {
                // bad count
                return QFALSE;
            }

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::InitROFF
    //	Handles stuffing the roff data in the CROFF object
    //
    // INPUTS:
    //	pass in the file data and the object to stuff the data into.
    //
    // RETURN:
    //	returns initialization success or failure
    //---------------------------------------------------------------------------
    pub fn InitROFF(&mut self, data: *mut c_void, obj: &mut CROFF) -> qboolean {
        let hdr = data as *mut TROFFHeader;

        unsafe {
            if (*hdr).mVersion == ROFF_NEW_VERSION {
                return self.InitROFF2(data, obj);
            }

            obj.mROFFEntries = (*hdr).mCount as c_int;
            let count = obj.mROFFEntries as usize;
            let mut vec = vec![TROFF2Entry {
                mOriginOffset: [0.0; 3],
                mRotateOffset: [0.0; 3],
                mStartNote: 0,
                mNumNotes: 0,
            }; count];
            obj.mMoveRotateList = vec.as_mut_ptr();
            core::mem::forget(vec);

            obj.mFrameTime = 1000 / ROFF_SAMPLE_RATE;
            obj.mLerp = ROFF_SAMPLE_RATE;
            obj.mNumNoteTracks = 0;
            obj.mNoteTrackIndexes = core::ptr::null_mut();

            if !obj.mMoveRotateList.is_null() {
                // Step past the header to get to the goods
                let roff_data = (hdr as *mut c_void as *mut TROFFEntry).add(1);

                // Copy all of the goods into our ROFF cache
                for i in 0..(*hdr).mCount as usize {
                    VectorCopy(&(*roff_data.add(i)).mOriginOffset, &mut (*obj.mMoveRotateList.add(i)).mOriginOffset);
                    VectorCopy(&(*roff_data.add(i)).mRotateOffset, &mut (*obj.mMoveRotateList.add(i)).mRotateOffset);
                    (*obj.mMoveRotateList.add(i)).mStartNote = -1;
                    (*obj.mMoveRotateList.add(i)).mNumNotes = 0;
                }

                self.FixBadAngles(obj);
            } else {
                return QFALSE;
            }

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::InitROFF2
    //	Handles stuffing the roff data in the CROFF object for version 2
    //
    // INPUTS:
    //	pass in the file data and the object to stuff the data into.
    //
    // RETURN:
    //	returns initialization success or failure
    //---------------------------------------------------------------------------
    pub fn InitROFF2(&mut self, data: *mut c_void, obj: &mut CROFF) -> qboolean {
        let hdr = data as *mut TROFF2Header;

        unsafe {
            obj.mROFFEntries = (*hdr).mCount;
            let count = obj.mROFFEntries as usize;
            let mut vec = vec![TROFF2Entry {
                mOriginOffset: [0.0; 3],
                mRotateOffset: [0.0; 3],
                mStartNote: 0,
                mNumNotes: 0,
            }; count];
            obj.mMoveRotateList = vec.as_mut_ptr();
            core::mem::forget(vec);

            obj.mFrameTime = (*hdr).mFrameRate;
            obj.mLerp = 1000 / (*hdr).mFrameRate;
            obj.mNumNoteTracks = (*hdr).mNumNotes;

            if !obj.mMoveRotateList.is_null() {
                // Step past the header to get to the goods
                let roff_data = (hdr as *mut c_void as *mut TROFF2Entry).add(1);

                // Copy all of the goods into our ROFF cache
                for i in 0..(*hdr).mCount as usize {
                    VectorCopy(&(*roff_data.add(i)).mOriginOffset, &mut (*obj.mMoveRotateList.add(i)).mOriginOffset);
                    VectorCopy(&(*roff_data.add(i)).mRotateOffset, &mut (*obj.mMoveRotateList.add(i)).mRotateOffset);
                    (*obj.mMoveRotateList.add(i)).mStartNote = (*roff_data.add(i)).mStartNote;
                    (*obj.mMoveRotateList.add(i)).mNumNotes = (*roff_data.add(i)).mNumNotes;
                }

                self.FixBadAngles(obj);

                if obj.mNumNoteTracks != 0 {
                    let mut size: usize = 0;
                    let mut ptr = (roff_data as *mut c_void).add((*hdr).mCount as usize * core::mem::size_of::<TROFF2Entry>()) as *mut c_char;
                    let start = ptr;

                    for _i in 0..obj.mNumNoteTracks {
                        size += strlen(ptr) + 1;
                        ptr = ptr.add(strlen(ptr) + 1);
                    }

                    let mut index_vec = vec![core::ptr::null_mut::<c_char>(); obj.mNumNoteTracks as usize];
                    obj.mNoteTrackIndexes = index_vec.as_mut_ptr();
                    core::mem::forget(index_vec);

                    let mut data_vec = vec![0u8; size];
                    ptr = data_vec.as_mut_ptr() as *mut c_char;
                    *obj.mNoteTrackIndexes = ptr;
                    core::mem::forget(data_vec);
                    memcpy(ptr as *mut c_void, start as *const c_void, size);

                    for i in 1..obj.mNumNoteTracks {
                        ptr = ptr.add(strlen(ptr) + 1);
                        *obj.mNoteTrackIndexes.add(i as usize) = ptr;
                    }
                }
            } else {
                return QFALSE;
            }

            return QTRUE;
        }
    }

    /************************************************************************************************
     * CROFFSystem::FixBadAngles                                                                    *
     *    This function will attempt to fix bad angles (large) that come in from the exporter.      *
     *                                                                                              *
     * Input                                                                                        *
     *    obj: the ROFF object                                                                      *
     *                                                                                              *
     * Output / Return                                                                              *
     *    none                                                                                      *
     *                                                                                              *
     ************************************************************************************************/
    pub fn FixBadAngles(&self, obj: &mut CROFF) {
        // Ideally we would fix the ROFF exporter, if that doesn't happen, this may be an adequate solution
        #[cfg(feature = "ROFF_AUTO_FIX_BAD_ANGLES")]
        unsafe {
            // Attempt to fix bad angles

            for index in 0..obj.mROFFEntries {
                for t in 0..3 {
                    if (*obj.mMoveRotateList.add(index as usize)).mRotateOffset[t] > 180.0f32 {
                        // found a bad angle
                        //	Com_Printf( S_COLOR_YELLOW"Fixing bad roff angle\n <%6.2f> changed to <%6.2f>.\n",
                        //				roff_data[i].mRotateOffset[t], roff_data[i].mRotateOffset[t] - 360.0f );
                        (*obj.mMoveRotateList.add(index as usize)).mRotateOffset[t] -= 360.0f32;
                    } else if (*obj.mMoveRotateList.add(index as usize)).mRotateOffset[t] < -180.0f32 {
                        // found a bad angle
                        //	Com_Printf( S_COLOR_YELLOW"Fixing bad roff angle\n <%6.2f> changed to <%6.2f>.\n",
                        //				roff_data[i].mRotateOffset[t], roff_data[i].mRotateOffset[t] + 360.0f );
                        (*obj.mMoveRotateList.add(index as usize)).mRotateOffset[t] += 360.0f32;
                    }
                }
            }
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::Cache
    //	Pre-caches roff data to avoid file hits during gameplay.  Disallows
    //		repeated caches of existing roffs.
    //
    // INPUTS:
    //	pass in the filepath of the roff to cache
    //
    // RETURN:
    //	returns ID of the roff, whether its an existing one or new one.
    //---------------------------------------------------------------------------
    pub fn Cache(&mut self, file: *const c_char, isClient: qboolean) -> c_int {
        // See if this item is already cached
        let mut id = self.GetID(file);

        if id != 0 {
            #[cfg(debug_assertions)]
            unsafe {
                Com_Printf(
                    b"Ignoring. File '%s' already cached.\n\0".as_ptr() as *const c_char,
                    file,
                );
            }
        } else {
            unsafe {
                // Read the file in one fell swoop
                let mut data: *mut c_void = core::ptr::null_mut();
                let mut len = FS_ReadFile(file, &mut data);

                if len <= 0 {
                    let mut otherPath: [c_char; 1024] = [0; 1024];
                    COM_StripExtension(file, otherPath.as_mut_ptr());
                    let rof_path = va(
                        b"scripts/%s.rof\0".as_ptr() as *const c_char,
                        otherPath.as_ptr(),
                    );
                    len = FS_ReadFile(rof_path, &mut data);
                    if len <= 0 {
                        Com_Printf(
                            b"Could not open .ROF file '%s'\n\0".as_ptr() as *const c_char,
                            file,
                        );
                        return 0;
                    }
                }

                // Make sure that the file is roff
                if self.IsROFF(data) == QFALSE {
                    Com_Printf(
                        b"cache failed: roff <%s> does not exist or is not a valid roff\n\0".as_ptr() as *const c_char,
                        file,
                    );
                    FS_FreeFile(data);

                    return 0;
                }

                // Things are looking good so far, so create a new CROFF object
                id = self.NewID();

                let mut c_roff = Box::new(CROFF::new(file, id));

                if self.InitROFF(data, &mut *c_roff) == QFALSE {
                    // something failed, so get rid of the object
                    self.Unload(id);
                    FS_FreeFile(data);
                    return 0;
                }

                self.mROFFList.insert(id, c_roff);

                FS_FreeFile(data);
            }
        }

        // Access the CROFF object and set client/server flags
        if let Some(c_roff) = self.mROFFList.get_mut(&id) {
            if isClient != QFALSE {
                c_roff.mUsedByClient = QTRUE;
            } else {
                c_roff.mUsedByServer = QTRUE;
            }
        }

        // If we haven't requested a new ID, we'll just be returning the ID of the existing roff
        return id;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::GetID
    //	Finds the associated (internal) ID of the specified roff file
    //
    // INPUTS:
    //	pass in the roff file path
    //
    // RETURN:
    //	returns ID if there is one, zero if nothing was found
    //---------------------------------------------------------------------------
    pub fn GetID(&self, file: *const c_char) -> c_int {
        unsafe {
            // Attempt to find the requested roff
            for (id, croff) in self.mROFFList.iter() {
                if strcmp(croff.mROFFFilePath.as_ptr(), file) == 0 {
                    // return the ID to this roff
                    return *id;
                }
            }
        }

        // Not found
        return 0;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::Unload
    //	Removes the roff from the list, deleting it to free up any used resources
    //
    // INPUTS:
    //	pass in the id of the roff to delete, use GetID if you only know the roff
    //		filepath
    //
    // RETURN:
    //	qtrue if item was in the list, qfalse otherwise
    //---------------------------------------------------------------------------
    pub fn Unload(&mut self, id: c_int) -> qboolean {
        if self.mROFFList.contains_key(&id) {
            // requested item found in the list, free mem, then remove from list
            self.mROFFList.remove(&id);

            #[cfg(debug_assertions)]
            unsafe {
                Com_Printf(b"roff unloaded\n\0".as_ptr() as *const c_char);
            }

            return QTRUE;
        } else {
            // not found

            #[cfg(debug_assertions)]
            unsafe {
                Com_Printf(
                    b"unload failed: roff <%i> does not exist\n\0".as_ptr() as *const c_char,
                    id,
                );
            }

            return QFALSE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::Clean
    //	Cleans out all Roffs, freeing up any used resources
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	success of operation
    //---------------------------------------------------------------------------
    pub fn Clean(&mut self, isClient: qboolean) -> qboolean {
        #[allow(unreachable_code)]
        {
            // Implementation using current approach
            let ids: Vec<c_int> = self.mROFFList.keys().cloned().collect();
            for id in ids {
                self.Unload(id);
            }

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::List
    //	Dumps the file path to the current set of cached roffs, for debug purposes
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn List(&self) {
        unsafe {
            Com_Printf(b"\n--Cached ROFF files--\n\0".as_ptr() as *const c_char);
            Com_Printf(b"ID   FILE\n\0".as_ptr() as *const c_char);

            for (id, croff) in self.mROFFList.iter() {
                Com_Printf(
                    b"%2i - %s\n\0".as_ptr() as *const c_char,
                    *id,
                    croff.mROFFFilePath.as_ptr(),
                );
            }

            Com_Printf(
                b"\nFiles: %i\n\0".as_ptr() as *const c_char,
                self.mROFFList.len() as c_int,
            );
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::List
    //	Overloaded version of List, dumps the specified roff data to the console
    //
    // INPUTS:
    //	id of roff to display
    //
    // RETURN:
    //	success or failure of operation
    //---------------------------------------------------------------------------
    pub fn ListById(&self, id: c_int) -> qboolean {
        if let Some(obj) = self.mROFFList.get(&id) {
            unsafe {
                let dat = obj.mMoveRotateList;

                Com_Printf(b"File: %s\n\0".as_ptr() as *const c_char, obj.mROFFFilePath.as_ptr());
                Com_Printf(b"ID: %i\n\0".as_ptr() as *const c_char, id);
                Com_Printf(
                    b"Entries: %i\n\n\0".as_ptr() as *const c_char,
                    obj.mROFFEntries,
                );

                Com_Printf(b"MOVE                 ROTATE\n\0".as_ptr() as *const c_char);

                for i in 0..obj.mROFFEntries {
                    Com_Printf(
                        b"%6.2f %6.2f %6.2f   %6.2f %6.2f %6.2f\n\0".as_ptr() as *const c_char,
                        (*dat.add(i as usize)).mOriginOffset[0],
                        (*dat.add(i as usize)).mOriginOffset[1],
                        (*dat.add(i as usize)).mOriginOffset[2],
                        (*dat.add(i as usize)).mRotateOffset[0],
                        (*dat.add(i as usize)).mRotateOffset[1],
                        (*dat.add(i as usize)).mRotateOffset[2],
                    );
                }

                return QTRUE;
            }
        }

        unsafe {
            Com_Printf(
                b"ROFF not found: id <%d>\n\0".as_ptr() as *const c_char,
                id,
            );
        }

        return QFALSE;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::Play
    //	Start roff playback on an entity
    //
    // INPUTS:
    //	the id of the entity that will be roffed
    //	the id of the roff to play
    //
    // RETURN:
    //	success or failure of add operation
    //---------------------------------------------------------------------------
    pub fn Play(&mut self, entID: c_int, id: c_int, doTranslation: qboolean, isClient: qboolean) -> qboolean {
        unsafe {
            let ent = SV_GentityNum(entID);

            (*ent).r.mIsRoffing = QTRUE;
            /*rjr	if(ent->GetPhysics() == PHYSICS_TYPE_NONE)
            {
                ent->SetPhysics(PHYSICS_TYPE_BRUSHMODEL);
            }*/
            //bjg TODO: reset this latter?

            if ent.is_null() {
                // shame on you..
                return QFALSE;
            }

            let mut roffing_ent = Box::new(SROFFEntity {
                mEntID: entID,
                mROFFID: id,
                mNextROFFTime: svs.time,
                mROFFFrame: 0,
                mKill: QFALSE,
                mSignal: QTRUE, // TODO: hook up the real signal code
                mTranslated: doTranslation,
                mIsClient: isClient,
                mStartAngles: (*ent).s.apos.trBase,
            });

            self.mROFFEntList.push(roffing_ent);

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::ListEnts
    //	List all of the ents in the roff system
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn ListEnts(&self) {
        /*	char	*name, *file;
        int		id;

        TROFFEntList::iterator itr = mROFFEntList.begin();
        TROFFList::iterator itrRoff;

        Com_Printf( S_COLOR_GREEN"\n--ROFFing Entities--\n" );
        Com_Printf( S_COLOR_GREEN"EntID EntName       RoffFile\n" );

        // display everything in the end list
        for ( itr = mROFFEntList.begin(); itr != mROFFEntList.end(); ++itr )
        {
            // Entity ID
            id = ((SROFFEntity *)(*itr))->mEntID;
            // Entity Name
            name = entitySystem->GetEntityFromID( id )->GetName();
            // ROFF object that will contain the roff file name
            itrRoff = mROFFList.find( ((SROFFEntity *)(*itr))->mROFFID );

            if ( itrRoff != mROFFList.end() )
            { // grab our filename
                file = ((CROFF *)((*itrRoff).second ))->mROFFFilePath;
            }
            else
            { // roff filename not found == bad
                file = "Error:  Unknown";
            }

            Com_Printf( S_COLOR_GREEN"%3i  %s    %s\n", id, name, file );
        }

        Com_Printf( S_COLOR_GREEN"\nEntities: %i\n", mROFFEntList.size() );*/
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::PurgeEnt
    //	Prematurely purge an entity from the roff system
    //
    // INPUTS:
    //	the id of the entity to purge
    //
    // RETURN:
    //	success or failure of purge operation
    //---------------------------------------------------------------------------
    pub fn PurgeEnt(&mut self, entID: c_int, isClient: qboolean) -> qboolean {
        for i in 0..self.mROFFEntList.len() {
            if self.mROFFEntList[i].mIsClient == isClient && self.mROFFEntList[i].mEntID == entID {
                // Make sure it won't stay lerping
                self.ClearLerp(&mut self.mROFFEntList[i]);

                self.mROFFEntList.remove(i);
                return QTRUE;
            }
        }

        unsafe {
            Com_Printf(
                b"Purge failed:  Entity <%i> not found\n\0".as_ptr() as *const c_char,
                entID,
            );
        }

        return QFALSE;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::PurgeEnt
    //	Prematurely purge an entity from the roff system
    //
    // INPUTS:
    //	the name fo the entity to purge
    //
    // RETURN:
    //	success or failure of purge operation
    //---------------------------------------------------------------------------
    pub fn PurgeEntByName(&mut self, name: *mut c_char) -> qboolean {
        /* rjr	CEntity *ent = entitySystem->GetEntityFromName( NULL, name );

        if ( ent && ent->GetInUse() == qtrue )
        {
            return PurgeEnt( ent->GetID() );
        }
        else
        {
            Com_Printf( S_COLOR_RED"Entity <%s> not found or not in use\n", name );
            return qfalse;
        }*/

        return QFALSE;
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::UpdateEntities
    //	Update all of the entities in the system
    //
    // INPUTS:
    //	none
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn UpdateEntities(&mut self, isClient: qboolean) {
        // display everything in the entity list
        for i in 0..self.mROFFEntList.len() {
            if self.mROFFEntList[i].mIsClient != isClient {
                continue;
            }

            // Get this entities ROFF object
            if let Some(roff) = self.mROFFList.get(&self.mROFFEntList[i].mROFFID) {
                // roff that baby!
                if self.ApplyROFF(&mut self.mROFFEntList[i], roff) == QFALSE {
                    // done roffing, mark for death
                    self.mROFFEntList[i].mKill = QTRUE;
                }
            } else {
                // roff not found == bad, dump an error message and purge this ent
                unsafe {
                    Com_Printf(b"ROFF System Error:\n\0".as_ptr() as *const c_char);
                    //			Com_Printf( S_COLOR_RED" -ROFF not found for entity <%s>\n",
                    //					entitySystem->GetEntityFromID(((SROFFEntity *)(*itr))->mEntID)->GetName() );
                }

                self.mROFFEntList[i].mKill = QTRUE;

                self.ClearLerp(&mut self.mROFFEntList[i]);
            }
        }

        // Delete killed ROFFers from the list
        // Man, there just has to be a better way to do this
        let mut i = 0;
        while i < self.mROFFEntList.len() {
            if self.mROFFEntList[i].mIsClient != isClient {
                i += 1;
                continue;
            }

            if self.mROFFEntList[i].mKill == QTRUE {
                //make sure ICARUS knows ROFF is stopped
                //			CICARUSGameInterface::TaskIDComplete(
                //				entitySystem->GetEntityFromID(((SROFFEntity *)(*itr))->mEntID), TID_MOVE);
                // trash this guy from the list
                self.mROFFEntList.remove(i);
                i = 0;
            } else {
                i += 1;
            }
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::ApplyROFF
    //	Does the dirty work of applying the raw ROFF data
    //
    // INPUTS:
    //	The the roff_entity struct and the raw roff data
    //
    // RETURN:
    //	True == success;  False == roff playback complete or failure
    //---------------------------------------------------------------------------
    pub fn ApplyROFF(&self, roff_ent: &mut SROFFEntity, roff: &CROFF) -> qboolean {
        let mut f: vec3_t = [0.0; 3];
        let mut r: vec3_t = [0.0; 3];
        let mut u: vec3_t = [0.0; 3];
        let mut result: vec3_t = [0.0; 3];
        let mut ent: *mut sharedEntity_t = core::ptr::null_mut();
        let mut originTrajectory: *mut trajectory_t = core::ptr::null_mut();
        let mut angleTrajectory: *mut trajectory_t = core::ptr::null_mut();
        let mut origin: *mut vec3_t = core::ptr::null_mut();
        let mut angle: *mut vec3_t = core::ptr::null_mut();

        unsafe {
            if svs.time < roff_ent.mNextROFFTime {
                // Not time to roff yet
                return QTRUE;
            }

            if roff_ent.mIsClient != QFALSE {
                #[cfg(not(feature = "DEDICATED"))]
                {
                    let mut originTemp: vec3_t = [0.0; 3];
                    let mut angleTemp: vec3_t = [0.0; 3];
                    originTrajectory = VM_Call(
                        cgvm,
                        0, // CG_GET_ORIGIN_TRAJECTORY
                        roff_ent.mEntID,
                    ) as *mut trajectory_t;
                    angleTrajectory = VM_Call(
                        cgvm,
                        1, // CG_GET_ANGLE_TRAJECTORY
                        roff_ent.mEntID,
                    ) as *mut trajectory_t;
                    VM_Call(
                        cgvm,
                        2, // CG_GET_ORIGIN
                        roff_ent.mEntID,
                        &mut originTemp as *mut vec3_t,
                    );
                    origin = &mut originTemp;
                    VM_Call(
                        cgvm,
                        3, // CG_GET_ANGLES
                        roff_ent.mEntID,
                        &mut angleTemp as *mut vec3_t,
                    );
                    angle = &mut angleTemp;
                }
            } else {
                // Find the entity to apply the roff to
                ent = SV_GentityNum(roff_ent.mEntID);

                if ent.is_null() {
                    // bad stuff
                    return QFALSE;
                }

                originTrajectory = &mut (*ent).s.pos as *mut trajectory_t;
                angleTrajectory = &mut (*ent).s.apos as *mut trajectory_t;
                origin = &mut (*ent).r.currentOrigin as *mut vec3_t;
                angle = &mut (*ent).r.currentAngles as *mut vec3_t;
            }

            if roff_ent.mROFFFrame >= roff.mROFFEntries {
                // we are done roffing, so stop moving and flag this ent to be removed
                self.SetLerp(
                    &mut *originTrajectory,
                    trType_t::TR_STATIONARY,
                    origin,
                    core::ptr::null(),
                    svs.time,
                    roff.mLerp,
                );
                self.SetLerp(
                    &mut *angleTrajectory,
                    trType_t::TR_STATIONARY,
                    angle,
                    core::ptr::null(),
                    svs.time,
                    roff.mLerp,
                );
                if roff_ent.mIsClient == QFALSE {
                    (*ent).r.mIsRoffing = QFALSE;
                }
                return QFALSE;
            }

            if roff_ent.mTranslated != QFALSE {
                AngleVectors(&roff_ent.mStartAngles, &mut f, &mut r, &mut u);
                VectorScale(
                    &f,
                    (*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mOriginOffset[0],
                    &mut result,
                );
                VectorMA(
                    &result,
                    -(*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mOriginOffset[1],
                    &r,
                    &mut result,
                );
                VectorMA(
                    &result,
                    (*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mOriginOffset[2],
                    &u,
                    &mut result,
                );
            } else {
                VectorCopy(
                    &(*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mOriginOffset,
                    &mut result,
                );
            }

            // Set up our origin interpolation
            self.SetLerp(
                &mut *originTrajectory,
                trType_t::TR_LINEAR,
                origin,
                &result,
                svs.time,
                roff.mLerp,
            );

            // Set up our angle interpolation
            self.SetLerp(
                &mut *angleTrajectory,
                trType_t::TR_LINEAR,
                angle,
                &(*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mRotateOffset,
                svs.time,
                roff.mLerp,
            );

            if (*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mStartNote >= 0 {
                for i in 0..(*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mNumNotes {
                    self.ProcessNote(
                        roff_ent,
                        *roff.mNoteTrackIndexes.add(
                            ((*roff.mMoveRotateList.add(roff_ent.mROFFFrame as usize)).mStartNote + i)
                                as usize,
                        ),
                    );
                }
            }

            // Advance ROFF frames and lock to a 10hz cycle
            roff_ent.mROFFFrame += 1;
            roff_ent.mNextROFFTime = svs.time + roff.mFrameTime;

            //rww - npcs need to know when they're getting roff'd
            if !ent.is_null() {
                (*ent).next_roff_time = roff_ent.mNextROFFTime;
            }

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::ProcessNote
    //	Helper function to process note track data
    //
    // INPUTS:
    //	The roff entity and note string
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn ProcessNote(&self, roff_ent: &SROFFEntity, note: *mut c_char) {
        unsafe {
            let mut temp: [c_char; 1024] = [0; 1024];
            let mut pos: usize = 0;

            while *note.add(pos) != 0 {
                let mut size: usize = 0;
                while *note.add(pos) != 0 && (*note.add(pos) as u8) < b' ' {
                    pos += 1;
                }

                while *note.add(pos) != 0 && (*note.add(pos) as u8) >= b' ' {
                    temp[size] = *note.add(pos);
                    size += 1;
                    pos += 1;
                }
                temp[size] = 0;

                if size != 0 {
                    if roff_ent.mIsClient != QFALSE {
                        #[cfg(not(feature = "DEDICATED"))]
                        {
                            // VM_Call( cgvm, CG_ROFF_NOTETRACK_CALLBACK, roff_ent->mEntID, temp );
                        }
                    } else {
                        // VM_Call( gvm, GAME_ROFF_NOTETRACK_CALLBACK, roff_ent->mEntID, temp );
                    }
                }
            }
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::ClearLerp
    //	Helper function to clear a given entities lerp fields
    //
    // INPUTS:
    //	The ID of the entity to clear
    //
    // RETURN:
    //	success or failure of the operation
    //---------------------------------------------------------------------------
    pub fn ClearLerp(&self, roff_ent: &SROFFEntity) -> qboolean {
        let mut ent: *mut sharedEntity_t = core::ptr::null_mut();
        let mut originTrajectory: *mut trajectory_t = core::ptr::null_mut();
        let mut angleTrajectory: *mut trajectory_t = core::ptr::null_mut();
        let mut origin: *mut vec3_t = core::ptr::null_mut();
        let mut angle: *mut vec3_t = core::ptr::null_mut();

        unsafe {
            if roff_ent.mIsClient != QFALSE {
                #[cfg(not(feature = "DEDICATED"))]
                {
                    let mut originTemp: vec3_t = [0.0; 3];
                    let mut angleTemp: vec3_t = [0.0; 3];
                    originTrajectory = VM_Call(
                        cgvm,
                        0, // CG_GET_ORIGIN_TRAJECTORY
                        roff_ent.mEntID,
                    ) as *mut trajectory_t;
                    angleTrajectory = VM_Call(
                        cgvm,
                        1, // CG_GET_ANGLE_TRAJECTORY
                        roff_ent.mEntID,
                    ) as *mut trajectory_t;
                    VM_Call(
                        cgvm,
                        2, // CG_GET_ORIGIN
                        roff_ent.mEntID,
                        &mut originTemp as *mut vec3_t,
                    );
                    origin = &mut originTemp;
                    VM_Call(
                        cgvm,
                        3, // CG_GET_ANGLES
                        roff_ent.mEntID,
                        &mut angleTemp as *mut vec3_t,
                    );
                    angle = &mut angleTemp;
                }
            } else {
                // Find the entity to apply the roff to
                ent = SV_GentityNum(roff_ent.mEntID);

                if ent.is_null() {
                    // bad stuff
                    return QFALSE;
                }

                originTrajectory = &mut (*ent).s.pos as *mut trajectory_t;
                angleTrajectory = &mut (*ent).s.apos as *mut trajectory_t;
                origin = &mut (*ent).r.currentOrigin as *mut vec3_t;
                angle = &mut (*ent).r.currentAngles as *mut vec3_t;
            }

            self.SetLerp(
                &mut *originTrajectory,
                trType_t::TR_STATIONARY,
                origin,
                core::ptr::null(),
                svs.time,
                ROFF_SAMPLE_RATE,
            );
            self.SetLerp(
                &mut *angleTrajectory,
                trType_t::TR_STATIONARY,
                angle,
                core::ptr::null(),
                svs.time,
                ROFF_SAMPLE_RATE,
            );

            return QTRUE;
        }
    }

    //---------------------------------------------------------------------------
    // CROFFSystem::SetLerp
    //	Helper function to set up a positional or angular interpolation
    //
    // INPUTS:
    //	The entity trajectory field to modify, the interpolation type, the base origin,
    //		and the interpolation start time
    //
    // RETURN:
    //	none
    //---------------------------------------------------------------------------
    pub fn SetLerp(&self, tr: &mut trajectory_t, type_: trType_t, origin: *const vec3_t, delta: *const vec3_t, time: c_int, rate: c_int) {
        tr.trType = type_;
        tr.trTime = time;
        unsafe {
            VectorCopy(origin, &mut tr.trBase);

            // Check for a NULL delta
            if !delta.is_null() {
                VectorScale(delta, rate as vec_t, &mut tr.trDelta);
            } else {
                VectorClear(&mut tr.trDelta);
            }
        }
    }

    fn NewID(&mut self) -> c_int {
        self.mID += 1;
        self.mID
    }
}

impl Default for CROFFSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CROFFSystem {
    fn drop(&mut self) {
        self.Restart();
    }
}

// ============================================================================
// Global instance
// ============================================================================

// The one and only instance...
pub static mut theROFFSystem: Option<CROFFSystem> = None;
