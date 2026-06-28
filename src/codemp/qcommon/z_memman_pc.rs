// Created 3/13/03 by Brian Osman (VV) - Split Zone/Hunk from common

// Anything above this #include will be ignored by the compiler
// (oracle/codemp/qcommon/exe_headers.h would be included)

// (oracle/codemp/qcommon/platform.h would be included)

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::{size_of, zeroed};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// ============================================================================
// Type definitions for memory tags (from tags.h expanded via TAGDEF macro)
// ============================================================================

pub type memtag_t = c_int;

// These match the enum expansion from oracle/codemp/qcommon/tags.h
pub const TAG_ALL: memtag_t = 0;
pub const TAG_BOTLIB: memtag_t = 1;
pub const TAG_CLIENTS: memtag_t = 2;
pub const TAG_BOTGAME: memtag_t = 3;
pub const TAG_DOWNLOAD: memtag_t = 4;
pub const TAG_GENERAL: memtag_t = 5;
pub const TAG_CLIPBOARD: memtag_t = 6;
pub const TAG_SND_MP3STREAMHDR: memtag_t = 7;
pub const TAG_SND_DYNAMICMUSIC: memtag_t = 8;
pub const TAG_BSP_DISKIMAGE: memtag_t = 9;
pub const TAG_VM: memtag_t = 10;
pub const TAG_SPECIAL_MEM_TEST: memtag_t = 11;
pub const TAG_HUNK_MARK1: memtag_t = 12;
pub const TAG_HUNK_MARK2: memtag_t = 13;
pub const TAG_EVENT: memtag_t = 14;
pub const TAG_FILESYS: memtag_t = 15;
pub const TAG_GHOUL2: memtag_t = 16;
pub const TAG_GHOUL2_GORE: memtag_t = 17;
pub const TAG_LISTFILES: memtag_t = 18;
pub const TAG_AMBIENTSET: memtag_t = 19;
pub const TAG_STATIC: memtag_t = 20;
pub const TAG_SMALL: memtag_t = 21;
pub const TAG_MODEL_MD3: memtag_t = 22;
pub const TAG_MODEL_GLM: memtag_t = 23;
pub const TAG_MODEL_GLA: memtag_t = 24;
pub const TAG_ICARUS: memtag_t = 25;
pub const TAG_ICARUS2: memtag_t = 26;
pub const TAG_ICARUS3: memtag_t = 27;
pub const TAG_ICARUS4: memtag_t = 28;
pub const TAG_ICARUS5: memtag_t = 29;
pub const TAG_SHADERTEXT: memtag_t = 30;
pub const TAG_SND_RAWDATA: memtag_t = 31;
pub const TAG_TEMP_WORKSPACE: memtag_t = 32;
pub const TAG_TEMP_PNG: memtag_t = 33;
pub const TAG_TEXTPOOL: memtag_t = 34;
pub const TAG_IMAGE_T: memtag_t = 35;
pub const TAG_INFLATE: memtag_t = 36;
pub const TAG_DEFLATE: memtag_t = 37;
pub const TAG_BSP: memtag_t = 38;
pub const TAG_GRIDMESH: memtag_t = 39;
pub const TAG_POINTCACHE: memtag_t = 40;
pub const TAG_TERRAIN: memtag_t = 41;
pub const TAG_R_TERRAIN: memtag_t = 42;
pub const TAG_RESAMPLE: memtag_t = 43;
pub const TAG_CM_TERRAIN: memtag_t = 44;
pub const TAG_CM_TERRAIN_TEMP: memtag_t = 45;
pub const TAG_TEMP_IMAGE: memtag_t = 46;
pub const TAG_VM_ALLOCATED: memtag_t = 47;
pub const TAG_TEMP_HUNKALLOC: memtag_t = 48;
pub const TAG_NEWDEL: memtag_t = 49;
pub const TAG_UI_ALLOC: memtag_t = 50;
pub const TAG_CG_UI_ALLOC: memtag_t = 51;
pub const TAG_BG_ALLOC: memtag_t = 52;
pub const TAG_BINK: memtag_t = 53;
pub const TAG_XBL_FRIENDS: memtag_t = 54;
pub const TAG_COUNT: memtag_t = 55;

// ============================================================================
// Memory zone structures
// ============================================================================

// This handles zone memory allocation.
// It is a wrapper around malloc with a tag id and a magic number at the start

const ZONE_MAGIC: c_int = 0x21436587;

#[repr(C)]
pub struct zoneHeader_s {
    iMagic: c_int,
    eTag: memtag_t,
    iSize: c_int,
    pNext: *mut zoneHeader_s,
    pPrev: *mut zoneHeader_s,
}

pub type zoneHeader_t = zoneHeader_s;

#[repr(C)]
pub struct zoneTail_t {
    iMagic: c_int,
}

#[inline]
fn ZoneTailFromHeader(pHeader: *mut zoneHeader_t) -> *mut zoneTail_t {
    unsafe {
        ((*pHeader as *mut c_char).add(size_of::<zoneHeader_t>() + (*pHeader).iSize as usize))
            as *mut zoneTail_t
    }
}

#[repr(C)]
struct zoneStats_s {
    iCount: c_int,
    iCurrent: c_int,
    iPeak: c_int,
    // I'm keeping these updated on the fly, since it's quicker for cache-pool
    // purposes rather than recalculating each time...
    //
    iSizesPerTag: [c_int; TAG_COUNT as usize],
    iCountsPerTag: [c_int; TAG_COUNT as usize],
}

pub type zoneStats_t = zoneStats_s;

#[repr(C)]
struct zone_s {
    Stats: zoneStats_t,
    Header: zoneHeader_t,
}

pub type zone_t = zone_s;

// Stubs for external types
#[repr(C)]
pub struct cvar_s {
    // Stub - actual definition lives in oracle
    _unused: [u8; 0],
}
pub type cvar_t = cvar_s;

// External function declarations
extern "C" {
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Cmd_RemoveCommand(cmd_name: *const c_char);
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: unsafe extern "C" fn());
    pub fn Cvar_Get(
        var_name: *const c_char,
        value: *const c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    pub fn CM_DeleteCachedMap(bGuaranteedOkToDelete: c_int) -> c_int;
    pub fn SND_RegisterAudio_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: c_int) -> c_int;
    pub fn RE_RegisterImages_LevelLoadEnd() -> c_int;
    pub fn RE_RegisterModels_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: c_int) -> c_int;
    pub fn SND_FreeOldestSound() -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(count: usize, size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    #[cfg(target_os = "windows")]
    pub fn Sleep(dwMilliseconds: u32);

    pub fn CIN_CloseAllVideos();
    pub fn CL_ShutdownCGame();
    pub fn CL_ShutdownUI();
    pub fn SV_ShutdownGameProgs();
    pub fn R_HunkClearCrap();
    pub fn VM_Clear();
}

#[cfg(not(target_os = "windows"))]
unsafe fn Sleep(_ms: u32) {
    // No-op on non-Windows
}

// Global variables

static mut com_validateZone: *mut cvar_t = null_mut();

static mut TheZone: zone_t = unsafe {
    zone_t {
        Stats: zoneStats_s {
            iCount: 0,
            iCurrent: 0,
            iPeak: 0,
            iSizesPerTag: [0; TAG_COUNT as usize],
            iCountsPerTag: [0; TAG_COUNT as usize],
        },
        Header: zoneHeader_s {
            iMagic: 0,
            eTag: 0,
            iSize: 0,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
    }
};

// static mem blocks to reduce a lot of small zone overhead
#[repr(C, packed)]
struct StaticZeroMem_t {
    Header: zoneHeader_t,
    Tail: zoneTail_t,
}

#[repr(C, packed)]
struct StaticMem_t {
    Header: zoneHeader_t,
    mem: [c_char; 2],
    Tail: zoneTail_t,
}

static mut gZeroMalloc: StaticZeroMem_t = StaticZeroMem_t {
    Header: zoneHeader_s {
        iMagic: ZONE_MAGIC,
        eTag: TAG_STATIC,
        iSize: 0,
        pNext: null_mut(),
        pPrev: null_mut(),
    },
    Tail: zoneTail_t {
        iMagic: ZONE_MAGIC,
    },
};

static mut gEmptyString: StaticMem_t = StaticMem_t {
    Header: zoneHeader_s {
        iMagic: ZONE_MAGIC,
        eTag: TAG_STATIC,
        iSize: 2,
        pNext: null_mut(),
        pPrev: null_mut(),
    },
    mem: [b'\0' as c_char, b'\0' as c_char],
    Tail: zoneTail_t {
        iMagic: ZONE_MAGIC,
    },
};

static mut gNumberString: [StaticMem_t; 10] = [
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'0' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'1' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'2' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'3' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'4' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'5' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'6' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'7' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'8' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
    StaticMem_t {
        Header: zoneHeader_s {
            iMagic: ZONE_MAGIC,
            eTag: TAG_STATIC,
            iSize: 2,
            pNext: null_mut(),
            pPrev: null_mut(),
        },
        mem: [b'9' as c_char, b'\0' as c_char],
        Tail: zoneTail_t {
            iMagic: ZONE_MAGIC,
        },
    },
];

static mut gbMemFreeupOccured: c_int = 0;

// String table for tags (would be generated from tags.h include)
// Stub implementation - actual strings would match the TAGDEF tags
static PS_TAG_STRINGS: &[&str] = &[
    "ALL",
    "BOTLIB",
    "CLIENTS",
    "BOTGAME",
    "DOWNLOAD",
    "GENERAL",
    "CLIPBOARD",
    "SND_MP3STREAMHDR",
    "SND_DYNAMICMUSIC",
    "BSP_DISKIMAGE",
    "VM",
    "SPECIAL_MEM_TEST",
    "HUNK_MARK1",
    "HUNK_MARK2",
    "EVENT",
    "FILESYS",
    "GHOUL2",
    "GHOUL2_GORE",
    "LISTFILES",
    "AMBIENTSET",
    "STATIC",
    "SMALL",
    "MODEL_MD3",
    "MODEL_GLM",
    "MODEL_GLA",
    "ICARUS",
    "ICARUS2",
    "ICARUS3",
    "ICARUS4",
    "ICARUS5",
    "SHADERTEXT",
    "SND_RAWDATA",
    "TEMP_WORKSPACE",
    "TEMP_PNG",
    "TEXTPOOL",
    "IMAGE_T",
    "INFLATE",
    "DEFLATE",
    "BSP",
    "GRIDMESH",
    "POINTCACHE",
    "TERRAIN",
    "R_TERRAIN",
    "RESAMPLE",
    "CM_TERRAIN",
    "CM_TERRAIN_TEMP",
    "TEMP_IMAGE",
    "VM_ALLOCATED",
    "TEMP_HUNKALLOC",
    "NEWDEL",
    "UI_ALLOC",
    "CG_UI_ALLOC",
    "BG_ALLOC",
    "BINK",
    "XBL_FRIENDS",
    "COUNT",
];

static mut Z_Details_f_ref: fn() = Z_Details_f;

// Scans through the linked list of mallocs and makes sure no data has been overwritten

pub unsafe fn Z_Validate() {
    if com_validateZone.is_null() || (*com_validateZone).integer == 0 {
        return;
    }

    let mut pMemory = TheZone.Header.pNext;
    while !pMemory.is_null() {
        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(
                1, // ERR_FATAL
                c"Z_Validate(): Corrupt zone header!".as_ptr() as *const c_char,
            );
            return;
        }

        if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
            Com_Error(
                1, // ERR_FATAL
                c"Z_Validate(): Corrupt zone tail!".as_ptr() as *const c_char,
            );
            return;
        }

        pMemory = (*pMemory).pNext;
    }
}

pub unsafe fn Z_Malloc(
    iSize: c_int,
    eTag: memtag_t,
    bZeroit: c_int,
    _iUnusedAlign: c_int,
) -> *mut c_void {
    gbMemFreeupOccured = 0;

    if iSize == 0 {
        let pMemory = addr_of_mut!(gZeroMalloc) as *mut zoneHeader_t;
        return pMemory.add(1) as *mut c_void;
    }

    // Add in tracking info
    //
    let iRealSize: c_int = (iSize + size_of::<zoneHeader_t>() as c_int + size_of::<zoneTail_t>() as c_int);

    // Allocate a chunk...
    //
    let mut pMemory: *mut zoneHeader_t = null_mut();
    loop {
        if pMemory.is_null() {
            #[cfg(target_os = "windows")]
            {
                if gbMemFreeupOccured != 0 {
                    Sleep(1000); // sleep for a second, so Windows has a chance to shuffle mem to de-swiss-cheese it
                }
            }

            if bZeroit != 0 {
                pMemory = calloc(iRealSize as usize, 1) as *mut zoneHeader_t;
            } else {
                pMemory = malloc(iRealSize as usize) as *mut zoneHeader_t;
            }
        }

        if !pMemory.is_null() {
            break;
        }

        // new bit, if we fail to malloc memory, try dumping some of the cached stuff that's non-vital and try again...
        //

        // ditch the BSP cache...
        //
        if CM_DeleteCachedMap(0) != 0 {
            gbMemFreeupOccured = 1;
            continue; // we've just ditched a whole load of memory, so try again with the malloc
        }

        // ditch any sounds not used on this level...
        //
        if SND_RegisterAudio_LevelLoadEnd(1) != 0 {
            gbMemFreeupOccured = 1;
            continue; // we've dropped at least one sound, so try again with the malloc
        }

        // ditch any image_t's (and associated GL memory) not used on this level...
        //
        if RE_RegisterImages_LevelLoadEnd() != 0 {
            gbMemFreeupOccured = 1;
            continue; // we've dropped at least one image, so try again with the malloc
        }

        // ditch the model-binaries cache...  (must be getting desperate here!)
        //
        if RE_RegisterModels_LevelLoadEnd(1) != 0 {
            gbMemFreeupOccured = 1;
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
        //
        extern "C" {
            pub static mut gbInsideLoadSound: c_int;
        }

        if gbInsideLoadSound == 0 {
            let iBytesFreed: c_int = SND_FreeOldestSound();
            if iBytesFreed != 0 {
                let mut iTheseBytesFreed: c_int;
                let mut total_freed = iBytesFreed;
                loop {
                    iTheseBytesFreed = SND_FreeOldestSound();
                    if iTheseBytesFreed == 0 {
                        break;
                    }
                    total_freed += iTheseBytesFreed;
                    if total_freed >= iRealSize {
                        break; // early opt-out since we've managed to recover enough (mem-contiguity issues aside)
                    }
                }
                gbMemFreeupOccured = 1;
                continue;
            }
        }

        // sigh, dunno what else to try, I guess we'll have to give up and report this as an out-of-mem error...
        //
        // findlabel:  "recovermem"

        let tag_str = if (eTag as usize) < PS_TAG_STRINGS.len() {
            PS_TAG_STRINGS[eTag as usize]
        } else {
            "UNKNOWN"
        };

        Com_Printf(
            c"^1Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n".as_ptr() as *const c_char,
            iSize,
            tag_str.as_ptr() as *const c_char,
        );
        Z_Details_f();
        Com_Error(
            1, // ERR_FATAL
            c"(Repeat): Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n".as_ptr() as *const c_char,
            iSize,
            tag_str.as_ptr() as *const c_char,
        );
        return null_mut();
    }

    // Link in
    (*pMemory).iMagic = ZONE_MAGIC;
    (*pMemory).eTag = eTag;
    (*pMemory).iSize = iSize;
    (*pMemory).pNext = TheZone.Header.pNext;
    TheZone.Header.pNext = pMemory;
    if !(*pMemory).pNext.is_null() {
        (*(*pMemory).pNext).pPrev = pMemory;
    }
    (*pMemory).pPrev = addr_of_mut!(TheZone.Header);
    //
    // add tail...
    //
    (*ZoneTailFromHeader(pMemory)).iMagic = ZONE_MAGIC;

    // Update stats...
    //
    TheZone.Stats.iCurrent += iSize;
    TheZone.Stats.iCount += 1;
    TheZone.Stats.iSizesPerTag[eTag as usize] += iSize;
    TheZone.Stats.iCountsPerTag[eTag as usize] += 1;

    if TheZone.Stats.iCurrent > TheZone.Stats.iPeak {
        TheZone.Stats.iPeak = TheZone.Stats.iCurrent;
    }

    Z_Validate(); // check for corruption

    let pvReturnMem: *mut c_void = pMemory.add(1) as *mut c_void;
    pvReturnMem
}

// used during model cacheing to save an extra malloc, lets us morph the disk-load buffer then
// just not fs_freefile() it afterwards.
//
pub unsafe fn Z_MorphMallocTag(pvAddress: *mut c_void, eDesiredTag: memtag_t) {
    let pMemory: *mut zoneHeader_t =
        ((pvAddress as *mut zoneHeader_t).offset(-1)) as *mut zoneHeader_t;

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(
            1, // ERR_FATAL
            c"Z_MorphMallocTag(): Not a valid zone header!".as_ptr() as *const c_char,
        );
        return; // won't get here
    }

    // DEC existing tag stats...
    //
    // TheZone.Stats.iCurrent  - unchanged
    // TheZone.Stats.iCount    - unchanged
    TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
    TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

    // morph...
    //
    (*pMemory).eTag = eDesiredTag;

    // INC new tag stats...
    //
    // TheZone.Stats.iCurrent  - unchanged
    // TheZone.Stats.iCount    - unchanged
    TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] += (*pMemory).iSize;
    TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] += 1;
}

unsafe fn Zone_FreeBlock(pMemory: *mut zoneHeader_t) {
    if (*pMemory).eTag != TAG_STATIC {
        // belt and braces, should never hit this though
        // Update stats...
        //
        TheZone.Stats.iCount -= 1;
        TheZone.Stats.iCurrent -= (*pMemory).iSize;
        TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
        TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

        // Sanity checks...
        //
        assert_eq!(
            (*(*pMemory).pPrev).pNext, pMemory,
            "Zone_FreeBlock: pPrev->pNext != pMemory"
        );
        assert!(
            (*pMemory).pNext.is_null()
                || (*(*pMemory).pNext).pPrev == pMemory,
            "Zone_FreeBlock: pNext->pPrev != pMemory"
        );

        // Unlink and free...
        //
        (*(*pMemory).pPrev).pNext = (*pMemory).pNext;
        if !(*pMemory).pNext.is_null() {
            (*(*pMemory).pNext).pPrev = (*pMemory).pPrev;
        }
        free(pMemory as *mut c_void);
    }
}

// stats-query function to ask how big a malloc is...
//
pub unsafe fn Z_Size(pvAddress: *mut c_void) -> c_int {
    let pMemory: *mut zoneHeader_t =
        ((pvAddress as *mut zoneHeader_t).offset(-1)) as *mut zoneHeader_t;

    if (*pMemory).eTag == TAG_STATIC {
        return 0; // kind of
    }

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(
            1, // ERR_FATAL
            c"Z_Size(): Not a valid zone header!".as_ptr() as *const c_char,
        );
        return 0; // won't get here
    }

    (*pMemory).iSize
}

// Frees a block of memory...
//
pub unsafe fn Z_Free(pvAddress: *mut c_void) {
    if pvAddress.is_null() {
        // I've put this in as a safety measure because of some bits of #ifdef BSPC stuff  -Ste.
        // Com_Error(ERR_FATAL, "Z_Free(): NULL arg");
        return;
    }

    let pMemory: *mut zoneHeader_t =
        ((pvAddress as *mut zoneHeader_t).offset(-1)) as *mut zoneHeader_t;

    if (*pMemory).eTag == TAG_STATIC {
        return;
    }

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(
            1, // ERR_FATAL
            c"Z_Free(): Corrupt zone header!".as_ptr() as *const c_char,
        );
        return;
    }
    if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
        Com_Error(
            1, // ERR_FATAL
            c"Z_Free(): Corrupt zone tail!".as_ptr() as *const c_char,
        );
        return;
    }

    Zone_FreeBlock(pMemory);
}

pub unsafe fn Z_MemSize(eTag: memtag_t) -> c_int {
    TheZone.Stats.iSizesPerTag[eTag as usize]
}

// Frees all blocks with the specified tag...
//
pub unsafe fn Z_TagFree(eTag: memtag_t) {
    let mut pMemory: *mut zoneHeader_t = TheZone.Header.pNext;
    while !pMemory.is_null() {
        let pNext: *mut zoneHeader_t = (*pMemory).pNext;
        if (eTag == TAG_ALL) || ((*pMemory).eTag == eTag) {
            Zone_FreeBlock(pMemory);
        }
        pMemory = pNext;
    }
}

pub unsafe fn S_Malloc(iSize: c_int) -> *mut c_void {
    Z_Malloc(iSize, TAG_SMALL, 0, 4)
}

#[cfg(debug_assertions)]
pub extern "C" fn Z_MemRecoverTest_f() {
    // needs to be in _DEBUG only, not good for final game!
    // fixme: findmeste: Remove this sometime
    //
    unsafe {
        let mut iTotalMalloc: c_int = 0;
        loop {
            let iThisMalloc: c_int = 5 * (1024 * 1024);
            Z_Malloc(iThisMalloc, TAG_SPECIAL_MEM_TEST, 0, 4); // and lose, just to consume memory
            iTotalMalloc += iThisMalloc;

            if gbMemFreeupOccured != 0 {
                break;
            }
        }

        Z_TagFree(TAG_SPECIAL_MEM_TEST);
    }
}

// Gives a summary of the zone memory usage

pub extern "C" fn Z_Stats_f() {
    unsafe {
        Com_Printf(
            c"\nThe zone is using %d bytes (%.2fMB) in %d memory blocks\n".as_ptr() as *const c_char,
            TheZone.Stats.iCurrent,
            (TheZone.Stats.iCurrent as f32) / 1024.0 / 1024.0,
            TheZone.Stats.iCount,
        );

        Com_Printf(
            c"The zone peaked at %d bytes (%.2fMB)\n".as_ptr() as *const c_char,
            TheZone.Stats.iPeak,
            (TheZone.Stats.iPeak as f32) / 1024.0 / 1024.0,
        );
    }
}

// Gives a detailed breakdown of the memory blocks in the zone

pub extern "C" fn Z_Details_f() {
    unsafe {
        Com_Printf(
            c"---------------------------------------------------------------------------\n".as_ptr() as *const c_char,
        );
        Com_Printf(
            c"%20s %9s\n".as_ptr() as *const c_char,
            c"Zone Tag".as_ptr() as *const c_char,
            c"Bytes".as_ptr() as *const c_char,
        );
        Com_Printf(
            c"%20s %9s\n".as_ptr() as *const c_char,
            c"--------".as_ptr() as *const c_char,
            c"-----".as_ptr() as *const c_char,
        );
        for i in 0..(TAG_COUNT as usize) {
            let iThisCount: c_int = TheZone.Stats.iCountsPerTag[i];
            let iThisSize: c_int = TheZone.Stats.iSizesPerTag[i];

            if iThisCount != 0 {
                // can you believe that using %2.2f as a format specifier doesn't bloody work?
                // It ignores the left-hand specifier. Sigh, now I've got to do shit like this...
                //
                let fSize: f32 = (iThisSize as f32) / 1024.0 / 1024.0;
                let iSize: c_int = fSize as c_int;
                let iRemainder: c_int = (100.0 * (fSize - (fSize.floor()))) as c_int;
                let tag_str = if i < PS_TAG_STRINGS.len() {
                    PS_TAG_STRINGS[i]
                } else {
                    "UNKNOWN"
                };
                Com_Printf(
                    c"%20s %9d (%2d.%02dMB) in %6d blocks (%9d average)\n".as_ptr() as *const c_char,
                    tag_str.as_ptr() as *const c_char,
                    iThisSize,
                    iSize,
                    iRemainder,
                    iThisCount,
                    iThisSize / iThisCount,
                );
            }
        }
        Com_Printf(
            c"---------------------------------------------------------------------------\n".as_ptr() as *const c_char,
        );

        Z_Stats_f();
    }
}

// Shuts down the zone memory system and frees up all memory
pub unsafe fn Com_ShutdownZoneMemory() {
    //  Com_Printf("Shutting down zone memory .....\n");

    Cmd_RemoveCommand(c"zone_stats".as_ptr() as *const c_char);
    Cmd_RemoveCommand(c"zone_details".as_ptr() as *const c_char);

    if TheZone.Stats.iCount != 0 {
        Com_Printf(
            c"Automatically freeing %d blocks making up %d bytes\n".as_ptr() as *const c_char,
            TheZone.Stats.iCount,
            TheZone.Stats.iCurrent,
        );
        Z_TagFree(TAG_ALL);

        assert_eq!(
            TheZone.Stats.iCount,
            0,
            "Zone still has blocks after TAG_ALL free"
        );
        assert_eq!(
            TheZone.Stats.iCurrent,
            0,
            "Zone still has bytes after TAG_ALL free"
        );
    }
}

// Initialises the zone memory system

pub unsafe fn Com_InitZoneMemory() {
    memset(
        addr_of_mut!(TheZone) as *mut c_void,
        0,
        size_of::<zone_t>(),
    );
    TheZone.Header.iMagic = ZONE_MAGIC;

    // #ifdef _DEBUG
    //    com_validateZone = Cvar_Get("com_validateZone", "1", 0);
    // #else
    com_validateZone = Cvar_Get(
        c"com_validateZone".as_ptr() as *const c_char,
        c"0".as_ptr() as *const c_char,
        0,
    );
    // #endif

    Cmd_AddCommand(
        c"zone_stats".as_ptr() as *const c_char,
        Z_Stats_f,
    );
    Cmd_AddCommand(
        c"zone_details".as_ptr() as *const c_char,
        Z_Details_f,
    );

    #[cfg(debug_assertions)]
    {
        Cmd_AddCommand(
            c"zone_memrecovertest".as_ptr() as *const c_char,
            Z_MemRecoverTest_f,
        );
    }
}

/*
========================
CopyString

 NOTE:  never write over the memory CopyString returns because
        memory from a memstatic_t might be returned
========================
*/
pub unsafe fn CopyString(in_: *const c_char) -> *mut c_char {
    if *in_ == 0 {
        return ((addr_of_mut!(gEmptyString) as *mut c_char)
            .add(size_of::<zoneHeader_t>()))
            as *mut c_char;
    } else if *in_.add(1) == 0 {
        let first_char = *in_;
        if first_char >= b'0' as c_char && first_char <= b'9' as c_char {
            return ((addr_of_mut!(gNumberString)[(first_char - b'0' as c_char) as usize]
                as *mut c_char)
                .add(size_of::<zoneHeader_t>()))
                as *mut c_char;
        }
    }

    let out: *mut c_char = S_Malloc((strlen(in_) + 1) as c_int) as *mut c_char;
    strcpy(out, in_);
    out
}

static mut hunk_tag: memtag_t = 0;

/*
===============
Com_TouchMemory

Touch all known used data to make sure it is paged in
===============
*/
pub unsafe fn Com_TouchMemory() {
    // int        start, end;
    let mut i: c_int;
    let mut j: c_int;
    let mut sum: c_int = 0;

    // start = Sys_Milliseconds();
    Z_Validate();

    sum = 0;

    let mut pMemory: *mut zoneHeader_t = TheZone.Header.pNext;
    while !pMemory.is_null() {
        let pMem: *mut u8 = (&mut (*pMemory)[1]) as *mut zoneHeader_t as *mut u8;
        j = (*pMemory).iSize >> 2;
        i = 0;
        while i < j {
            sum += (*(pMem.add(i as usize * 4) as *mut c_int));
            i += 64;
        }

        pMemory = (*pMemory).pNext;
    }

    // end = Sys_Milliseconds();
    // Com_Printf( "Com_TouchMemory: %i msec\n", end - start );
}

pub unsafe fn Com_TheHunkMarkHasBeenMade() -> c_int {
    if hunk_tag == TAG_HUNK_MARK2 {
        return 1; // qtrue
    }
    0 // qfalse
}

/*
=================
Com_InitHunkMemory
=================
*/
pub unsafe fn Com_InitHunkMemory() {
    hunk_tag = TAG_HUNK_MARK1;
    Hunk_Clear();
}

pub unsafe fn Com_ShutdownHunkMemory() {
    // Er, ok. Clear it then I guess.
    Z_TagFree(TAG_HUNK_MARK1);
    Z_TagFree(TAG_HUNK_MARK2);
}

/*
====================
Hunk_MemoryRemaining
====================
*/
pub unsafe fn Hunk_MemoryRemaining() -> c_int {
    (64 * 1024 * 1024) - (Z_MemSize(TAG_HUNK_MARK1) + Z_MemSize(TAG_HUNK_MARK2))
    // Yeah. Whatever. We've got no size now.
}

/*
===================
Hunk_SetMark

The server calls this after the level and game VM have been loaded
===================
*/
pub unsafe fn Hunk_SetMark() {
    hunk_tag = TAG_HUNK_MARK2;
}

/*
=================
Hunk_ClearToMark

The client calls this before starting a vid_restart or snd_restart
=================
*/
pub unsafe fn Hunk_ClearToMark() {
    assert_eq!(hunk_tag, TAG_HUNK_MARK2, "if this is not true then no mark has been made");
    Z_TagFree(TAG_HUNK_MARK2);
}

/*
=================
Hunk_CheckMark
=================
*/
pub unsafe fn Hunk_CheckMark() -> c_int {
    // if( hunk_low.mark || hunk_high.mark ) {
    if hunk_tag != TAG_HUNK_MARK1 {
        return 1; // qtrue
    }
    0 // qfalse
}

pub unsafe fn Hunk_Clear() {
    #[cfg(not(feature = "DEDICATED"))]
    {
        CL_ShutdownCGame();
        CL_ShutdownUI();
    }
    SV_ShutdownGameProgs();

    #[cfg(not(feature = "DEDICATED"))]
    {
        CIN_CloseAllVideos();
    }

    hunk_tag = TAG_HUNK_MARK1;
    Z_TagFree(TAG_HUNK_MARK1);
    Z_TagFree(TAG_HUNK_MARK2);

    R_HunkClearCrap();

    // Com_Printf( "Hunk_Clear: reset the hunk ok\n" );
    VM_Clear();

    // See if any ghoul2 stuff was leaked, at this point it should be all cleaned up.
    // #ifdef _FULL_G2_LEAK_CHECKING
    //    assert(g_Ghoul2Allocations == 0 && g_G2ClientAlloc == 0 && g_G2ServerAlloc == 0);
    //    if (g_Ghoul2Allocations)
    //    {
    //        Com_Printf("%i bytes leaked by ghoul2 routines (%i client, %i server)\n", g_Ghoul2Allocations, g_G2ClientAlloc, g_G2ServerAlloc);
    //        G2_DEBUG_ReportLeaks();
    //    }
    // #endif
}

/*
=================
Hunk_Alloc

Allocate permanent (until the hunk is cleared) memory
=================
*/
pub unsafe fn Hunk_Alloc(_size: c_int, _preference: c_int) -> *mut c_void {
    Z_Malloc(_size, hunk_tag, 1, 4)
}

/*
=================
Hunk_AllocateTempMemory

This is used by the file loading system.
Multiple files can be loaded in temporary memory.
When the files-in-use count reaches zero, all temp memory will be deleted
=================
*/
pub unsafe fn Hunk_AllocateTempMemory(size: c_int) -> *mut c_void {
    // don't bother clearing, because we are going to load a file over it
    Z_Malloc(size, TAG_TEMP_HUNKALLOC, 0, 4)
}

/*
==================
Hunk_FreeTempMemory
==================
*/
pub unsafe fn Hunk_FreeTempMemory(buf: *mut c_void) {
    Z_Free(buf);
}

/*
=================
Hunk_ClearTempMemory

The temp space is no longer needed.  If we have left more
touched but unused memory on this side, have future
permanent allocs use this side.
=================
*/
pub unsafe fn Hunk_ClearTempMemory() {
    Z_TagFree(TAG_TEMP_HUNKALLOC);
}
