// Created 2/3/03 by Brian Osman - split Zone code from common.cpp

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::mem;
use std::ptr::{self, addr_of, addr_of_mut};

#[allow(non_camel_case_types)]
pub type qboolean = c_int;
pub const QFALSE: c_int = 0;
pub const QTRUE: c_int = 1;

// memtag_t is used as an enum discriminant but is typedef'd as char
// The TAG_* constants are defined via the tags.h inclusion macro pattern
#[allow(non_camel_case_types)]
pub type memtag_t = c_char;

// Tag constants from tags.h
pub const TAG_ALL: c_char = 0;
pub const TAG_HUNKALLOC: c_char = 1;
pub const TAG_HUNKMISCMODELS: c_char = 2;
pub const TAG_FILESYS: c_char = 3;
pub const TAG_EVENT: c_char = 4;
pub const TAG_CLIPBOARD: c_char = 5;
pub const TAG_LISTFILES: c_char = 6;
pub const TAG_AMBIENTSET: c_char = 7;
pub const TAG_G_ALLOC: c_char = 8;
pub const TAG_CLIENTS: c_char = 9;
pub const TAG_STATIC: c_char = 10;
pub const TAG_SMALL: c_char = 11;
pub const TAG_MODEL: c_char = 12;
pub const TAG_MODEL_MD3: c_char = 13;
pub const TAG_MODEL_GLM: c_char = 14;
pub const TAG_MODEL_GLA: c_char = 15;
pub const TAG_ICARUS: c_char = 16;
pub const TAG_IMAGE_T: c_char = 17;
pub const TAG_TEMP_WORKSPACE: c_char = 18;
pub const TAG_TEMP_TGA: c_char = 19;
pub const TAG_TEMP_JPG: c_char = 20;
pub const TAG_TEMP_PNG: c_char = 21;
pub const TAG_SND_MP3STREAMHDR: c_char = 22;
pub const TAG_SND_DYNAMICMUSIC: c_char = 23;
pub const TAG_SND_RAWDATA: c_char = 24;
pub const TAG_GHOUL2: c_char = 25;
pub const TAG_BSP: c_char = 26;
pub const TAG_BSP_DISKIMAGE: c_char = 27;
pub const TAG_GP2: c_char = 28;
pub const TAG_SPECIAL_MEM_TEST: c_char = 29;
pub const TAG_ANIMATION_CFG: c_char = 30;
pub const TAG_SAVEGAME: c_char = 31;
pub const TAG_SHADERTEXT: c_char = 32;
pub const TAG_CM_TERRAIN: c_char = 33;
pub const TAG_R_TERRAIN: c_char = 34;
pub const TAG_INFLATE: c_char = 35;
pub const TAG_DEFLATE: c_char = 36;
pub const TAG_POINTCACHE: c_char = 37;
pub const TAG_NEWDEL: c_char = 38;
pub const TAG_COUNT: c_char = 39;

const MAX_QPATH: usize = 256;

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
const DEBUG_ZONE_ALLOC_OPTIONAL_LABEL_SIZE: usize = 256;

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
static mut giZoneSnaphotNum: c_int = 0;

// define a string table of all mem tags...
// This would normally be generated from the tags.h include with TAGDEF macro
// For now we provide string references that match the tag constants
static psTagStrings: [*const c_char; 41] = unsafe {
    // These are stub pointers - the actual implementation would need to populate these
    [ptr::null(); 41]
};

// This handles zone memory allocation.
// It is a wrapper around malloc with a tag id and a magic number at the start

const ZONE_MAGIC: c_int = 0x21436587i32;

// if you change ANYTHING in this structure, be sure to update the tables below using DEF_STATIC...
#[repr(C)]
#[derive(Clone, Copy)]
pub struct zoneHeader_s {
    pub iMagic: c_int,
    pub eTag: memtag_t,
    pub iSize: c_int,
    pub pNext: *mut zoneHeader_s,
    pub pPrev: *mut zoneHeader_s,

    #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
    pub sSrcFileBaseName: [c_char; MAX_QPATH],
    #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
    pub iSrcFileLineNum: c_int,
    #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
    pub sOptionalLabel: [c_char; DEBUG_ZONE_ALLOC_OPTIONAL_LABEL_SIZE],
    #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
    pub iSnapshotNumber: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct zoneTail_t {
    pub iMagic: c_int,
}

#[inline]
unsafe fn ZoneTailFromHeader(pHeader: *mut zoneHeader_s) -> *mut zoneTail_t {
    (pHeader as *mut c_char).add(
        mem::size_of::<zoneHeader_s>() + (*pHeader).iSize as usize
    ) as *mut zoneTail_t
}

#[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
use std::collections::HashMap;

#[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
static mut mapAllocatedZones: Option<HashMap<*mut c_void, c_int>> = None;

#[repr(C)]
pub struct zoneStats_s {
    pub iCount: c_int,
    pub iCurrent: c_int,
    pub iPeak: c_int,

    // I'm keeping these updated on the fly, since it's quicker for cache-pool
    // purposes rather than recalculating each time...
    pub iSizesPerTag: [c_int; 41],
    pub iCountsPerTag: [c_int; 41],
}

#[repr(C)]
pub struct zone_s {
    pub Stats: zoneStats_s,
    pub Header: zoneHeader_s,
}

// Opaque cvar_t struct - only the integer field is accessed
#[repr(C)]
pub struct cvar_s {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_s,
}

pub type cvar_t = cvar_s;

static mut com_validateZone: *mut cvar_t = ptr::null_mut();

// We can't use struct initialization directly for static mut, so we use mem::zeroed
static mut TheZone: zone_s = unsafe { mem::zeroed() };

// static mem blocks to reduce a lot of small zone overhead
#[repr(C)]
pub struct StaticZeroMem_t {
    pub Header: zoneHeader_s,
    // byte mem[0];
    pub Tail: zoneTail_t,
}

#[repr(C)]
pub struct StaticMem_t {
    pub Header: zoneHeader_s,
    pub mem: [u8; 2],
    pub Tail: zoneTail_t,
}

static gEmptyString: StaticMem_t = StaticMem_t {
    Header: unsafe {
        mem::zeroed()
    },
    mem: [0, 0],
    Tail: zoneTail_t { iMagic: ZONE_MAGIC },
};

static gNumberString: [StaticMem_t; 10] = [
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'0', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'1', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'2', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'3', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'4', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'5', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'6', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'7', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'8', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: unsafe { mem::zeroed() }, mem: [b'9', 0], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
];

static mut gbMemFreeupOccured: qboolean = QFALSE;

extern "C" {
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn CM_DeleteCachedMap(bDeleteAll: qboolean) -> qboolean;
    fn SND_RegisterAudio_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: qboolean) -> qboolean;
    fn RE_RegisterImages_LevelLoadEnd() -> qboolean;
    fn RE_RegisterModels_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: qboolean) -> qboolean;
    static gbInsideLoadSound: qboolean;
    fn SND_FreeOldestSound() -> c_int;
    fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
    fn Cmd_RemoveCommand(cmd_name: *const c_char);
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Sys_Milliseconds() -> c_int;
    fn atoi(s: *const c_char) -> c_int;
}

const ERR_FATAL: c_int = 3;

// Scans through the linked list of mallocs and makes sure no data has been overwritten
pub fn Z_Validate() -> c_int {
    let mut ret: c_int = 0;
    unsafe {
        if ptr::is_null(com_validateZone) || (*com_validateZone).integer == 0 {
            return ret;
        }

        let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
        while !ptr::is_null(pMemory) {
            #[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
            {
                // this won't happen here, but wtf?
                let iAllocCount = if let Some(ref map) = mapAllocatedZones {
                    map.get(&(pMemory as *mut c_void)).copied().unwrap_or(0)
                } else {
                    0
                };
                if iAllocCount <= 0 {
                    Com_Error(ERR_FATAL, b"Z_Validate(): Bad block allocation count!\0".as_ptr() as *const c_char);
                    return ret;
                }
            }

            if (*pMemory).iMagic != ZONE_MAGIC {
                Com_Error(ERR_FATAL, b"Z_Validate(): Corrupt zone header!\0".as_ptr() as *const c_char);
                return ret;
            }

            // this block of code is intended to make sure all of the data is paged in
            if (*pMemory).eTag != TAG_IMAGE_T
                && (*pMemory).eTag != TAG_MODEL_MD3
                && (*pMemory).eTag != TAG_MODEL_GLM
                && (*pMemory).eTag != TAG_MODEL_GLA
            {
                // don't bother with disk caches as they've already been hit or will be thrown out next
                let mut memstart = pMemory as *mut u8;
                let mut totalSize = (*pMemory).iSize;
                while totalSize > 4096 {
                    memstart = memstart.add(4096);
                    ret += *memstart as c_int; // this fools the optimizer
                    totalSize -= 4096;
                }
            }

            if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
                Com_Error(ERR_FATAL, b"Z_Validate(): Corrupt zone tail!\0".as_ptr() as *const c_char);
                return ret;
            }

            pMemory = (*pMemory).pNext;
        }
    }
    ret
}

// Frees a block of memory...
fn Zone_FreeBlock(pMemory: *mut zoneHeader_s) -> c_int {
    unsafe {
        let iSize = (*pMemory).iSize;
        if (*pMemory).eTag != TAG_STATIC {
            // belt and braces, should never hit this though

            // Update stats...
            (*addr_of_mut!(TheZone)).Stats.iCount -= 1;
            (*addr_of_mut!(TheZone)).Stats.iCurrent -= (*pMemory).iSize;
            (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
            (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

            // Sanity checks...
            assert!((*(*pMemory).pPrev).pNext == pMemory);
            assert!(ptr::is_null((*pMemory).pNext) || ((*(*pMemory).pNext).pPrev == pMemory));

            // Unlink and free...
            (*(*pMemory).pPrev).pNext = (*pMemory).pNext;
            if !ptr::is_null((*pMemory).pNext) {
                (*(*pMemory).pNext).pPrev = (*pMemory).pPrev;
            }

            // debugging double frees
            (*pMemory).iMagic = 0x45455246i32; // 'FREE' in little-endian

            let layout = std::alloc::Layout::from_size_align_unchecked(
                mem::size_of::<zoneHeader_s>() + iSize as usize + mem::size_of::<zoneTail_t>(),
                mem::align_of::<zoneHeader_s>(),
            );
            std::alloc::dealloc(pMemory as *mut u8, layout);

            #[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
            {
                // this has already been checked for in execution order, but wtf?
                let iAllocCount = if let Some(ref mut map) = mapAllocatedZones {
                    map.get(&(pMemory as *mut c_void)).copied().unwrap_or(0)
                } else {
                    0
                };
                if iAllocCount == 0 {
                    Com_Error(ERR_FATAL, b"Zone_FreeBlock(): Double-freeing block!\0".as_ptr() as *const c_char);
                    return -1;
                }
                if let Some(ref mut map) = mapAllocatedZones {
                    *map.entry(pMemory as *mut c_void).or_insert(0) -= 1;
                }
            }
        }
        iSize
    }
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
pub fn _D_Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean, psFile: *const c_char, iLine: c_int) -> *mut c_void {
    unsafe {
        *addr_of_mut!(gbMemFreeupOccured) = QFALSE;

        if iSize == 0 {
            let pMemory = &gEmptyString as *const StaticMem_t as *const zoneHeader_s;
            return pMemory.add(1) as *mut c_void;
        }

        // Add in tracking info and round to a longword...  (ignore longword aligning now we're not using contiguous blocks)
        let iRealSize = (iSize as usize + mem::size_of::<zoneHeader_s>() + mem::size_of::<zoneTail_t>()) as c_int;

        // Allocate a chunk...
        let mut pMemory: *mut zoneHeader_s = ptr::null_mut();
        loop {
            #[cfg(target_os = "windows")]
            {
                if *addr_of!(gbMemFreeupOccured) != QFALSE {
                    std::thread::sleep(std::time::Duration::from_secs(1)); // sleep for a second, so Windows has a chance to shuffle mem to de-swiss-cheese it
                }
            }

            if bZeroit != QFALSE {
                let layout = std::alloc::Layout::from_size_align_unchecked(
                    iRealSize as usize,
                    mem::align_of::<zoneHeader_s>(),
                );
                let allocated = std::alloc::alloc_zeroed(layout);
                pMemory = allocated as *mut zoneHeader_s;
            } else {
                let layout = std::alloc::Layout::from_size_align_unchecked(
                    iRealSize as usize,
                    mem::align_of::<zoneHeader_s>(),
                );
                let allocated = std::alloc::alloc(layout);
                pMemory = allocated as *mut zoneHeader_s;
            }

            if !ptr::is_null(pMemory) {
                break;
            }

            // new bit, if we fail to malloc memory, try dumping some of the cached stuff that's non-vital and try again...

            // ditch the BSP cache...
            if CM_DeleteCachedMap(QFALSE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've just ditched a whole load of memory, so try again with the malloc
            }

            // ditch any sounds not used on this level...
            if SND_RegisterAudio_LevelLoadEnd(QTRUE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've dropped at least one sound, so try again with the malloc
            }

            // ditch any image_t's (and associated GL texture mem) not used on this level...
            if RE_RegisterImages_LevelLoadEnd() != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've dropped at least one image, so try again with the malloc
            }

            // ditch the model-binaries cache...  (must be getting desperate here!)
            if RE_RegisterModels_LevelLoadEnd(QTRUE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue;
            }

            // as a last panic measure, dump all the audio memory, but not if we're in the audio loader
            // (which is annoying, but I'm not sure how to ensure we're not dumping any memory needed by the sound
            // currently being loaded if that was the case)...
            //
            // note that this keeps querying until it's freed up as many bytes as the requested size, but freeing
            // several small blocks might not mean that one larger one is satisfiable after freeup, however that'll
            // just make it go round again and try for freeing up another bunch of blocks until the total is satisfied
            // again (though this will have freed twice the requested amount in that case), so it'll either work
            // eventually or not free up enough and drop through to the final ERR_DROP. No worries...
            if gbInsideLoadSound == QFALSE {
                let iBytesFreed = SND_FreeOldestSound();
                if iBytesFreed != 0 {
                    let mut iTheseBytesFreed = 0;
                    loop {
                        iTheseBytesFreed = SND_FreeOldestSound();
                        if iTheseBytesFreed == 0 {
                            break;
                        }
                        if iBytesFreed + iTheseBytesFreed >= iRealSize {
                            break; // early opt-out since we've managed to recover enough (mem-contiguity issues aside)
                        }
                    }
                    *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                    continue;
                }
            }

            // sigh, dunno what else to try, I guess we'll have to give up and report this as an out-of-mem error...
            // findlabel:  "recovermem"

            Com_Printf(
                b"^1Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n\0".as_ptr() as *const c_char,
                iSize,
                psTagStrings[eTag as usize],
            );
            Z_Details_f();
            Com_Error(
                ERR_FATAL,
                b"(Repeat): Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n\0".as_ptr() as *const c_char,
                iSize,
                psTagStrings[eTag as usize],
            );
            return ptr::null_mut();
        }

        #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
        {
            extern "C" {
                fn Filename_WithoutPath(psFilename: *const c_char) -> *mut c_char;
            }
            Q_strncpyz(
                (*pMemory).sSrcFileBaseName.as_mut_ptr(),
                Filename_WithoutPath(psFile),
                mem::size_of_val(&(*pMemory).sSrcFileBaseName),
            );
            (*pMemory).iSrcFileLineNum = iLine;
            (*pMemory).sOptionalLabel[0] = 0;
            (*pMemory).iSnapshotNumber = *addr_of!(giZoneSnaphotNum);
        }

        // Link in
        (*pMemory).iMagic = ZONE_MAGIC;
        (*pMemory).eTag = eTag;
        (*pMemory).iSize = iSize;
        (*pMemory).pNext = (*addr_of_mut!(TheZone)).Header.pNext;
        (*addr_of_mut!(TheZone)).Header.pNext = pMemory;
        if !ptr::is_null((*pMemory).pNext) {
            (*(*pMemory).pNext).pPrev = pMemory;
        }
        (*pMemory).pPrev = &mut (*addr_of_mut!(TheZone)).Header;
        //
        // add tail...
        //
        (*ZoneTailFromHeader(pMemory)).iMagic = ZONE_MAGIC;

        // Update stats...
        (*addr_of_mut!(TheZone)).Stats.iCurrent += iSize;
        (*addr_of_mut!(TheZone)).Stats.iCount += 1;
        (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[eTag as usize] += iSize;
        (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[eTag as usize] += 1;

        if (*addr_of_mut!(TheZone)).Stats.iCurrent > (*addr_of_mut!(TheZone)).Stats.iPeak {
            (*addr_of_mut!(TheZone)).Stats.iPeak = (*addr_of_mut!(TheZone)).Stats.iCurrent;
        }

        #[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
        {
            if mapAllocatedZones.is_none() {
                mapAllocatedZones = Some(HashMap::new());
            }
            if let Some(ref mut map) = mapAllocatedZones {
                *map.entry(pMemory as *mut c_void).or_insert(0) += 1;
            }
        }

        Z_Validate(); // check for corruption

        pMemory.add(1) as *mut c_void
    }
}

#[cfg(not(feature = "DEBUG_ZONE_ALLOCS"))]
pub fn Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean, _unusedAlign: c_int) -> *mut c_void {
    unsafe {
        *addr_of_mut!(gbMemFreeupOccured) = QFALSE;

        if iSize == 0 {
            let pMemory = &gEmptyString as *const StaticMem_t as *const zoneHeader_s;
            return pMemory.add(1) as *mut c_void;
        }

        // Add in tracking info and round to a longword...  (ignore longword aligning now we're not using contiguous blocks)
        let iRealSize = (iSize as usize + mem::size_of::<zoneHeader_s>() + mem::size_of::<zoneTail_t>()) as c_int;

        // Allocate a chunk...
        let mut pMemory: *mut zoneHeader_s = ptr::null_mut();
        loop {
            #[cfg(target_os = "windows")]
            {
                if *addr_of!(gbMemFreeupOccured) != QFALSE {
                    std::thread::sleep(std::time::Duration::from_secs(1)); // sleep for a second, so Windows has a chance to shuffle mem to de-swiss-cheese it
                }
            }

            if bZeroit != QFALSE {
                let layout = std::alloc::Layout::from_size_align_unchecked(
                    iRealSize as usize,
                    mem::align_of::<zoneHeader_s>(),
                );
                let allocated = std::alloc::alloc_zeroed(layout);
                pMemory = allocated as *mut zoneHeader_s;
            } else {
                let layout = std::alloc::Layout::from_size_align_unchecked(
                    iRealSize as usize,
                    mem::align_of::<zoneHeader_s>(),
                );
                let allocated = std::alloc::alloc(layout);
                pMemory = allocated as *mut zoneHeader_s;
            }

            if !ptr::is_null(pMemory) {
                break;
            }

            // new bit, if we fail to malloc memory, try dumping some of the cached stuff that's non-vital and try again...

            // ditch the BSP cache...
            if CM_DeleteCachedMap(QFALSE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've just ditched a whole load of memory, so try again with the malloc
            }

            // ditch any sounds not used on this level...
            if SND_RegisterAudio_LevelLoadEnd(QTRUE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've dropped at least one sound, so try again with the malloc
            }

            // ditch any image_t's (and associated GL texture mem) not used on this level...
            if RE_RegisterImages_LevelLoadEnd() != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue; // we've dropped at least one image, so try again with the malloc
            }

            // ditch the model-binaries cache...  (must be getting desperate here!)
            if RE_RegisterModels_LevelLoadEnd(QTRUE) != QFALSE {
                *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                continue;
            }

            // as a last panic measure, dump all the audio memory, but not if we're in the audio loader
            if gbInsideLoadSound == QFALSE {
                let iBytesFreed = SND_FreeOldestSound();
                if iBytesFreed != 0 {
                    let mut iTheseBytesFreed = 0;
                    loop {
                        iTheseBytesFreed = SND_FreeOldestSound();
                        if iTheseBytesFreed == 0 {
                            break;
                        }
                        if iBytesFreed + iTheseBytesFreed >= iRealSize {
                            break; // early opt-out since we've managed to recover enough (mem-contiguity issues aside)
                        }
                    }
                    *addr_of_mut!(gbMemFreeupOccured) = QTRUE;
                    continue;
                }
            }

            // sigh, dunno what else to try, I guess we'll have to give up and report this as an out-of-mem error...

            Com_Printf(
                b"^1Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n\0".as_ptr() as *const c_char,
                iSize,
                psTagStrings[eTag as usize],
            );
            Z_Details_f();
            Com_Error(
                ERR_FATAL,
                b"(Repeat): Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n\0".as_ptr() as *const c_char,
                iSize,
                psTagStrings[eTag as usize],
            );
            return ptr::null_mut();
        }

        // Link in
        (*pMemory).iMagic = ZONE_MAGIC;
        (*pMemory).eTag = eTag;
        (*pMemory).iSize = iSize;
        (*pMemory).pNext = (*addr_of_mut!(TheZone)).Header.pNext;
        (*addr_of_mut!(TheZone)).Header.pNext = pMemory;
        if !ptr::is_null((*pMemory).pNext) {
            (*(*pMemory).pNext).pPrev = pMemory;
        }
        (*pMemory).pPrev = &mut (*addr_of_mut!(TheZone)).Header;
        //
        // add tail...
        //
        (*ZoneTailFromHeader(pMemory)).iMagic = ZONE_MAGIC;

        // Update stats...
        (*addr_of_mut!(TheZone)).Stats.iCurrent += iSize;
        (*addr_of_mut!(TheZone)).Stats.iCount += 1;
        (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[eTag as usize] += iSize;
        (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[eTag as usize] += 1;

        if (*addr_of_mut!(TheZone)).Stats.iCurrent > (*addr_of_mut!(TheZone)).Stats.iPeak {
            (*addr_of_mut!(TheZone)).Stats.iPeak = (*addr_of_mut!(TheZone)).Stats.iCurrent;
        }

        Z_Validate(); // check for corruption

        pMemory.add(1) as *mut c_void
    }
}

// used during model cacheing to save an extra malloc, lets us morph the disk-load buffer then
// just not fs_freefile() it afterwards.
pub fn Z_MorphMallocTag(pvAddress: *mut c_void, eDesiredTag: memtag_t) {
    unsafe {
        let pMemory = (pvAddress as *mut zoneHeader_s).offset(-1);

        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, b"Z_MorphMallocTag(): Not a valid zone header!\0".as_ptr() as *const c_char);
            return; // won't get here
        }

        // DEC existing tag stats...
        // TheZone.Stats.iCurrent	- unchanged
        // TheZone.Stats.iCount	- unchanged
        (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
        (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

        // morph...
        (*pMemory).eTag = eDesiredTag;

        // INC new tag stats...
        // TheZone.Stats.iCurrent	- unchanged
        // TheZone.Stats.iCount	- unchanged
        (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[(*pMemory).eTag as usize] += (*pMemory).iSize;
        (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[(*pMemory).eTag as usize] += 1;
    }
}

// stats-query function to to see if it's our malloc
// returns block size if so
pub fn Z_IsFromZone(pvAddress: *mut c_void, eTag: memtag_t) -> qboolean {
    unsafe {
        let pMemory = (pvAddress as *mut zoneHeader_s).offset(-1);

        if (*pMemory).iMagic == 0x45455246i32 { // debugging double free ('FREE' in little-endian)
            Com_Printf(b"Z_IsFromZone(%x): Ptr has been freed already!(%9s)\n\0".as_ptr() as *const c_char, pvAddress as c_int, pvAddress);
            return QFALSE;
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            return QFALSE;
        }

        // looks like it is from our zone, let's double check the tag
        if (*pMemory).eTag != eTag {
            return QFALSE;
        }

        (*pMemory).iSize
    }
}

// stats-query function to ask how big a malloc is...
pub fn Z_Size(pvAddress: *mut c_void) -> c_int {
    unsafe {
        let pMemory = (pvAddress as *mut zoneHeader_s).offset(-1);

        if (*pMemory).eTag == TAG_STATIC {
            return 0; // kind of
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, b"Z_Size(): Not a valid zone header!\0".as_ptr() as *const c_char);
            return 0; // won't get here
        }

        (*pMemory).iSize
    }
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
pub fn _D_Z_Label(pvAddress: *const c_void, psLabel: *const c_char) {
    unsafe {
        let pMemory = (pvAddress as *mut zoneHeader_s).offset(-1);

        if (*pMemory).eTag == TAG_STATIC {
            return;
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, b"_D_Z_Label(): Not a valid zone header!\0".as_ptr() as *const c_char);
        }

        Q_strncpyz(
            (*pMemory).sOptionalLabel.as_mut_ptr(),
            psLabel,
            mem::size_of_val(&(*pMemory).sOptionalLabel),
        );
        (*pMemory).sOptionalLabel[mem::size_of_val(&(*pMemory).sOptionalLabel) - 1] = 0;
    }
}

// Frees a block of memory...
pub fn Z_Free(pvAddress: *mut c_void) -> c_int {
    unsafe {
        if (*addr_of_mut!(TheZone)).Stats.iCount == 0 {
            // Com_Error(ERR_FATAL, "Z_Free(): Zone has been cleard already!");
            Com_Printf(b"Z_Free(%x): Zone has been cleard already!\n\0".as_ptr() as *const c_char, pvAddress as c_int);
            return -1;
        }

        let pMemory = (pvAddress as *mut zoneHeader_s).offset(-1);

        if (*pMemory).iMagic == 0x45455246i32 { // debugging double free
            Com_Error(ERR_FATAL, b"Z_Free(%s): Block already-freed, or not allocated through Z_Malloc!\0".as_ptr() as *const c_char, pvAddress);
            return -1;
        }

        if (*pMemory).eTag == TAG_STATIC {
            return 0;
        }

        #[cfg(feature = "DETAILED_ZONE_DEBUG_CODE")]
        {
            let iAllocCount = if let Some(ref map) = mapAllocatedZones {
                map.get(&(pMemory as *mut c_void)).copied().unwrap_or(0)
            } else {
                0
            };
            if iAllocCount <= 0 {
                Com_Error(ERR_FATAL, b"Z_Free(): Block already-freed, or not allocated through Z_Malloc!\0".as_ptr() as *const c_char);
                return -1;
            }
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, b"Z_Free(): Corrupt zone header!\0".as_ptr() as *const c_char);
            return -1;
        }
        if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, b"Z_Free(): Corrupt zone tail!\0".as_ptr() as *const c_char);
            return -1;
        }

        Zone_FreeBlock(pMemory)
    }
}

pub fn Z_MemSize(eTag: memtag_t) -> c_int {
    unsafe { (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[eTag as usize] }
}

// Frees all blocks with the specified tag...
pub fn Z_TagFree(eTag: memtag_t) {
    // #ifdef _DEBUG
    // int iZoneBlocks = TheZone.Stats.iCount;
    // #endif

    unsafe {
        let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
        while !ptr::is_null(pMemory) {
            let pNext = (*pMemory).pNext;
            if (eTag == TAG_ALL) || ((*pMemory).eTag == eTag) {
                Zone_FreeBlock(pMemory);
            }
            pMemory = pNext;
        }
    }

    // these stupid pragmas don't work here???!?!?!
    // #ifdef _DEBUG
    // #pragma warning( disable : 4189)
    // int iBlocksFreed = iZoneBlocks - TheZone.Stats.iCount;
    // #pragma warning( default : 4189)
    // #endif
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
pub fn _D_S_Malloc(iSize: c_int, psFile: *const c_char, iLine: c_int) -> *mut c_void {
    _D_Z_Malloc(iSize, TAG_SMALL, QFALSE, psFile, iLine)
}

#[cfg(not(feature = "DEBUG_ZONE_ALLOCS"))]
pub fn S_Malloc(iSize: c_int) -> *mut c_void {
    Z_Malloc(iSize, TAG_SMALL, QFALSE, 0)
}

#[cfg(feature = "_DEBUG")]
fn Z_MemRecoverTest_f() {
    // needs to be in _DEBUG only, not good for final game!
    unsafe {
        if Cmd_Argc() != 2 {
            Com_Printf(b"Usage: zone_memrecovertest max2alloc\n\0".as_ptr() as *const c_char);
            return;
        }

        let iMaxAlloc = 1024 * 1024 * atoi(Cmd_Argv(1));
        let mut iTotalMalloc = 0;
        loop {
            let iThisMalloc = 5 * (1024 * 1024);
            Z_Malloc(iThisMalloc, TAG_SPECIAL_MEM_TEST, QFALSE, 0); // and lose, just to consume memory
            iTotalMalloc += iThisMalloc;

            if *addr_of!(gbMemFreeupOccured) != QFALSE || (iTotalMalloc >= iMaxAlloc) {
                break;
            }
        }

        Z_TagFree(TAG_SPECIAL_MEM_TEST);
    }
}

// Gives a summary of the zone memory usage
fn Z_Stats_f() {
    unsafe {
        Com_Printf(
            b"\nThe zone is using %d bytes (%.2fMB) in %d memory blocks\n\0".as_ptr() as *const c_char,
            (*addr_of_mut!(TheZone)).Stats.iCurrent,
            (*addr_of_mut!(TheZone)).Stats.iCurrent as f32 / 1024.0 / 1024.0,
            (*addr_of_mut!(TheZone)).Stats.iCount,
        );

        Com_Printf(
            b"The zone peaked at %d bytes (%.2fMB)\n\0".as_ptr() as *const c_char,
            (*addr_of_mut!(TheZone)).Stats.iPeak,
            (*addr_of_mut!(TheZone)).Stats.iPeak as f32 / 1024.0 / 1024.0,
        );
    }
}

// Gives a detailed breakdown of the memory blocks in the zone
fn Z_Details_f() {
    unsafe {
        Com_Printf(b"---------------------------------------------------------------------------\n\0".as_ptr() as *const c_char);
        Com_Printf(b"%20s %9s\n\0".as_ptr() as *const c_char, b"Zone Tag\0".as_ptr() as *const c_char, b"Bytes\0".as_ptr() as *const c_char);
        Com_Printf(b"%20s %9s\n\0".as_ptr() as *const c_char, b"--------\0".as_ptr() as *const c_char, b"-----\0".as_ptr() as *const c_char);
        for i in 0..(TAG_COUNT as usize) {
            let iThisCount = (*addr_of_mut!(TheZone)).Stats.iCountsPerTag[i];
            let iThisSize = (*addr_of_mut!(TheZone)).Stats.iSizesPerTag[i];

            if iThisCount != 0 {
                // can you believe that using %2.2f as a format specifier doesn't bloody work?
                // It ignores the left-hand specifier. Sigh, now I've got to do shit like this...
                let fSize = (iThisSize as f32) / 1024.0 / 1024.0;
                let iSize = fSize as c_int;
                let iRemainder = (100.0 * (fSize - fSize.floor())) as c_int;
                Com_Printf(
                    b"%20s %9d (%2d.%02dMB) in %6d blocks (%9d Bytes/block)\n\0".as_ptr() as *const c_char,
                    psTagStrings[i],
                    iThisSize,
                    iSize,
                    iRemainder,
                    iThisCount,
                    iThisSize / iThisCount,
                );
            }
        }
        Com_Printf(b"---------------------------------------------------------------------------\n\0".as_ptr() as *const c_char);

        Z_Stats_f();
    }
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
use std::collections::BTreeMap;

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
static mut AllTagBlockLabels: Option<BTreeMap<String, BTreeMap<String, c_int>>> = None;

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
fn Z_Snapshot_f() {
    unsafe {
        if let Some(ref mut labels) = AllTagBlockLabels {
            labels.clear();
        } else {
            AllTagBlockLabels = Some(BTreeMap::new());
        }

        let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
        while !ptr::is_null(pMemory) {
            if let Some(ref mut labels) = AllTagBlockLabels {
                let tag_str = std::ffi::CStr::from_ptr(psTagStrings[(*pMemory).eTag as usize])
                    .to_string_lossy()
                    .into_owned();
                let label_str = std::ffi::CStr::from_ptr((*pMemory).sOptionalLabel.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                *labels
                    .entry(tag_str)
                    .or_insert_with(BTreeMap::new)
                    .entry(label_str)
                    .or_insert(0) += 1;
            }
            pMemory = (*pMemory).pNext;
        }

        *addr_of_mut!(giZoneSnaphotNum) += 1;
        Com_Printf(b"Ok.    ( Current snapshot num is now %d )\n\0".as_ptr() as *const c_char, *addr_of!(giZoneSnaphotNum));
    }
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
fn Z_TagDebug_f() {
    unsafe {
        let mut AllTagBlockLabels_Local: Option<BTreeMap<String, BTreeMap<String, c_int>>> = None;
        let mut bSnapShotTestActive = QFALSE;

        let mut eTag: memtag_t = TAG_ALL;

        let psTAGName = Cmd_Argv(1);
        if *psTAGName as c_int != 0 {
            // check optional arg...
            if Q_stricmp(psTAGName, b"#snap\0".as_ptr() as *const c_char) == 0 {
                bSnapShotTestActive = QTRUE;

                if let Some(ref labels) = AllTagBlockLabels {
                    AllTagBlockLabels_Local = Some(labels.clone());
                }
            }

            let mut psTAGName = Cmd_Argv(2);
            if *psTAGName as c_int != 0 {
                // skip over "tag_" if user supplied it...
                if Q_stricmpn(psTAGName, b"TAG_\0".as_ptr() as *const c_char, 4) == 0 {
                    psTAGName = psTAGName.add(4);
                }

                // see if the user specified a valid tag...
                for i in 0..(TAG_COUNT as usize) {
                    if Q_stricmp(psTAGName, psTagStrings[i]) == 0 {
                        eTag = i as memtag_t;
                        break;
                    }
                }
            }
        } else {
            Com_Printf(b"Usage: 'zone_tagdebug [#snap] <tag>', e.g. TAG_GHOUL2, TAG_ALL (careful!)\n\0".as_ptr() as *const c_char);
            return;
        }

        let snapshot_str = if bSnapShotTestActive != QFALSE { b"( since snapshot only )" } else { b"" };
        Com_Printf(
            b"Dumping debug data for tag \"%s\"...%s\n\n\0".as_ptr() as *const c_char,
            psTagStrings[eTag as usize],
            snapshot_str.as_ptr() as *const c_char,
        );

        Com_Printf(b"%8s\0".as_ptr() as *const c_char, b" \0".as_ptr() as *const c_char); // to compensate for code further down:   Com_Printf("(%5d) ",iBlocksListed);
        if eTag == TAG_ALL {
            Com_Printf(b"%20s \0".as_ptr() as *const c_char, b"Zone Tag\0".as_ptr() as *const c_char);
        }
        Com_Printf(b"%9s\n\0".as_ptr() as *const c_char, b"Bytes\0".as_ptr() as *const c_char);
        Com_Printf(b"%8s\0".as_ptr() as *const c_char, b" \0".as_ptr() as *const c_char);
        if eTag == TAG_ALL {
            Com_Printf(b"%20s \0".as_ptr() as *const c_char, b"--------\0".as_ptr() as *const c_char);
        }
        Com_Printf(b"%9s\n\0".as_ptr() as *const c_char, b"-----\0".as_ptr() as *const c_char);

        if bSnapShotTestActive != QFALSE {
            // dec ref counts in last snapshot for all current blocks (which will make new stuff go negative)
            let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
            while !ptr::is_null(pMemory) {
                if (*pMemory).eTag == eTag || eTag == TAG_ALL {
                    if let Some(ref mut labels) = AllTagBlockLabels_Local {
                        let tag_str = std::ffi::CStr::from_ptr(psTagStrings[(*pMemory).eTag as usize])
                            .to_string_lossy()
                            .into_owned();
                        let label_str = std::ffi::CStr::from_ptr((*pMemory).sOptionalLabel.as_ptr())
                            .to_string_lossy()
                            .into_owned();
                        *labels
                            .entry(tag_str)
                            .or_insert_with(BTreeMap::new)
                            .entry(label_str)
                            .or_insert(0) -= 1;
                    }
                }
                pMemory = (*pMemory).pNext;
            }
        }

        // now dump them out...
        let mut iBlocksListed = 0;
        let mut iTotalSize = 0;
        let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
        while !ptr::is_null(pMemory) {
            let should_print = if bSnapShotTestActive != QFALSE {
                (*pMemory).eTag == eTag || eTag == TAG_ALL
            } else {
                ((*pMemory).eTag == eTag || eTag == TAG_ALL)
            };

            if should_print {
                let fSize = ((*pMemory).iSize as f32) / 1024.0 / 1024.0;
                let iSize = fSize as c_int;
                let iRemainder = (100.0 * (fSize - fSize.floor())) as c_int;

                Com_Printf(b"(%5d) \0".as_ptr() as *const c_char, iBlocksListed);

                if eTag == TAG_ALL {
                    Com_Printf(b"%20s\0".as_ptr() as *const c_char, psTagStrings[(*pMemory).eTag as usize]);
                }

                Com_Printf(
                    b" %9d (%2d.%02dMB) File: \"%s\", Line: %d\n\0".as_ptr() as *const c_char,
                    (*pMemory).iSize,
                    iSize,
                    iRemainder,
                    (*pMemory).sSrcFileBaseName.as_ptr(),
                    (*pMemory).iSrcFileLineNum,
                );
                if (*pMemory).sOptionalLabel[0] != 0 {
                    Com_Printf(b"( Label: \"%s\" )\n\0".as_ptr() as *const c_char, (*pMemory).sOptionalLabel.as_ptr());
                }
                iBlocksListed += 1;
                iTotalSize += (*pMemory).iSize;

                if bSnapShotTestActive != QFALSE {
                    // bump ref count so we only 1 warning per new string, not for every one sharing that label...
                    if let Some(ref mut labels) = AllTagBlockLabels_Local {
                        let tag_str = std::ffi::CStr::from_ptr(psTagStrings[(*pMemory).eTag as usize])
                            .to_string_lossy()
                            .into_owned();
                        let label_str = std::ffi::CStr::from_ptr((*pMemory).sOptionalLabel.as_ptr())
                            .to_string_lossy()
                            .into_owned();
                        *labels
                            .entry(tag_str)
                            .or_insert_with(BTreeMap::new)
                            .entry(label_str)
                            .or_insert(0) += 1;
                    }
                }
            }
            pMemory = (*pMemory).pNext;
        }

        Com_Printf(
            b"( %d blocks listed, %d bytes (%.2fMB) total )\n\0".as_ptr() as *const c_char,
            iBlocksListed,
            iTotalSize,
            (iTotalSize as f32) / 1024.0 / 1024.0,
        );
    }
}

// Shuts down the zone memory system and frees up all memory
pub fn Com_ShutdownZoneMemory() {
    unsafe {
        Cmd_RemoveCommand(b"zone_stats\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"zone_details\0".as_ptr() as *const c_char);

        #[cfg(feature = "_DEBUG")]
        {
            Cmd_RemoveCommand(b"zone_memrecovertest\0".as_ptr() as *const c_char);
        }

        #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
        {
            Cmd_RemoveCommand(b"zone_tagdebug\0".as_ptr() as *const c_char);
            Cmd_RemoveCommand(b"zone_snapshot\0".as_ptr() as *const c_char);
        }

        if (*addr_of_mut!(TheZone)).Stats.iCount != 0 {
            // Com_Printf("Automatically freeing %d blocks making up %d bytes\n", TheZone.Stats.iCount, TheZone.Stats.iCurrent);
            Z_TagFree(TAG_ALL);

            assert!((*addr_of_mut!(TheZone)).Stats.iCount == 0);
            assert!((*addr_of_mut!(TheZone)).Stats.iCurrent == 0);
        }
    }
}

// Initialises the zone memory system
pub fn Com_InitZoneMemory() {
    unsafe {
        Com_Printf(b"Initialising zone memory .....\n\0".as_ptr() as *const c_char);

        // Zero initialize TheZone by writing a zeroed struct
        (*addr_of_mut!(TheZone)) = mem::zeroed();
        (*addr_of_mut!(TheZone)).Header.iMagic = ZONE_MAGIC;

        com_validateZone = Cvar_Get(
            b"com_validateZone\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        );

        Cmd_AddCommand(
            b"zone_stats\0".as_ptr() as *const c_char,
            Z_Stats_f as *const c_void,
        );
        Cmd_AddCommand(
            b"zone_details\0".as_ptr() as *const c_char,
            Z_Details_f as *const c_void,
        );

        #[cfg(feature = "_DEBUG")]
        {
            Cmd_AddCommand(
                b"zone_memrecovertest\0".as_ptr() as *const c_char,
                Z_MemRecoverTest_f as *const c_void,
            );
        }

        #[cfg(feature = "DEBUG_ZONE_ALLOCS")]
        {
            Cmd_AddCommand(
                b"zone_tagdebug\0".as_ptr() as *const c_char,
                Z_TagDebug_f as *const c_void,
            );
            Cmd_AddCommand(
                b"zone_snapshot\0".as_ptr() as *const c_char,
                Z_Snapshot_f as *const c_void,
            );
        }
    }
}

/*
========================
CopyString

 NOTE:	never write over the memory CopyString returns because
		memory from a memstatic_t might be returned
========================
*/
pub fn CopyString(in_: *const c_char) -> *mut c_char {
    unsafe {
        if *in_ as c_int == 0 {
            return ((&gEmptyString as *const StaticMem_t as *const zoneHeader_s).add(1)) as *mut c_char;
        } else if *in_.add(1) as c_int == 0 {
            if *in_ as c_int >= b'0' as c_int && *in_ as c_int <= b'9' as c_int {
                return ((&gNumberString[(*in_ as c_int - b'0' as c_int) as usize] as *const StaticMem_t as *const zoneHeader_s).add(1)) as *mut c_char;
            }
        }

        let out = S_Malloc((strlen(in_) + 1) as c_int) as *mut c_char;
        strcpy(out, in_);

        Z_Label(out, in_);

        out
    }
}

#[cfg(feature = "DEBUG_ZONE_ALLOCS")]
#[inline]
fn Z_Label(ptr: *const c_void, label: *const c_char) {
    _D_Z_Label(ptr, label);
}

#[cfg(not(feature = "DEBUG_ZONE_ALLOCS"))]
#[inline]
fn Z_Label(_ptr: *const c_void, _label: *const c_char) {
    // no-op
}

/*
===============
Com_TouchMemory

Touch all known used data to make sure it is paged in
===============
*/
pub fn Com_TouchMemory() {
    unsafe {
        Z_Validate();

        let start = Sys_Milliseconds();

        let mut sum: c_int = 0;
        let mut totalTouched: c_int = 0;

        let mut pMemory = (*addr_of_mut!(TheZone)).Header.pNext;
        while !ptr::is_null(pMemory) {
            let pMem = (&(*pMemory).add(1) as *const zoneHeader_s) as *const u8;
            let j = (*pMemory).iSize >> 2;
            for i in (0..j).step_by(64) {
                sum += (*(pMem.add(i as usize * 4) as *const c_int));
            }
            totalTouched += (*pMemory).iSize;
            pMemory = (*pMemory).pNext;
        }

        let end = Sys_Milliseconds();

        // Com_Printf( "Com_TouchMemory: %i bytes, %i msec\n", totalTouched, end - start );
    }
}
