// Created 3/13/03 by Brian Osman (VV) - Split Zone/Hunk from common

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

//Anything above this #include will be ignored by the compiler
use crate::codemp::qcommon::exe_headers_h::*;

use crate::codemp::qcommon::platform_h::*;

use core::ffi::{c_char, c_int, c_void, CStr};
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

#[cfg(feature = "detailed_zone_debug_code")]
use std::collections::BTreeMap;

// External function declarations
// (hoisted from file-scope forward decls and inline `extern` decls inside function bodies)
extern "C" {
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Cmd_RemoveCommand(cmd_name: *const c_char);
    fn Cmd_AddCommand(cmd_name: *const c_char, function: unsafe extern "C" fn());
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    // from Z_Malloc body: extern qboolean CM_DeleteCachedMap(qboolean bGuaranteedOkToDelete)
    fn CM_DeleteCachedMap(bGuaranteedOkToDelete: qboolean) -> qboolean;
    // from Z_Malloc body: extern qboolean SND_RegisterAudio_LevelLoadEnd(qboolean)
    fn SND_RegisterAudio_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: qboolean) -> qboolean;
    // from Z_Malloc body: extern qboolean RE_RegisterModels_LevelLoadEnd(qboolean)
    fn RE_RegisterModels_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: qboolean) -> qboolean;
    // from Z_Malloc body: extern qboolean gbInsideLoadSound
    static mut gbInsideLoadSound: qboolean;
    // from Z_Malloc body: extern int SND_FreeOldestSound()
    fn SND_FreeOldestSound() -> c_int;
    // file-scope forward decl (line 22): void CIN_CloseAllVideos()
    fn CIN_CloseAllVideos();
    // file-scope forward decls (lines 736-738)
    fn CL_ShutdownCGame();
    fn CL_ShutdownUI();
    fn SV_ShutdownGameProgs();
    // file-scope forward decl (line 747): void R_HunkClearCrap(void)
    fn R_HunkClearCrap();
    fn VM_Clear();
    // C stdlib (from system includes transitively via exe_headers.h)
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// #ifndef DEDICATED — from Z_Malloc body: extern qboolean RE_RegisterImages_LevelLoadEnd(void)
#[cfg(not(feature = "dedicated"))]
extern "C" {
    fn RE_RegisterImages_LevelLoadEnd() -> qboolean;
}

// #ifdef _WIN32 — Sleep for memory-recovery delay
#[cfg(target_os = "windows")]
extern "C" {
    fn Sleep(dwMilliseconds: u32);
}

// #ifdef _FULL_G2_LEAK_CHECKING — Ghoul2 leak diagnostics (line 749)
#[cfg(feature = "full_g2_leak_checking")]
extern "C" {
    fn G2_DEBUG_ReportLeaks();
    static g_Ghoul2Allocations: c_int;
    static g_G2ClientAlloc: c_int;
    static g_G2ServerAlloc: c_int;
}

////////////////////////////////////////////////
//
// #ifdef TAGDEF  // itu?
// #undef TAGDEF
// #endif
// #define TAGDEF(blah) #blah
// const static char *psTagStrings[TAG_COUNT+1] = { #include "../qcommon/tags.h" };
// (TAGDEF macro stringifies each tag name; tags.h is expanded inline below)
// +1 because TAG_COUNT will itself become a string here. Oh well.
//
// Port note: [*const c_char; N] cannot be placed in a Rust static without unsafe Sync;
// translated as [&'static CStr; N] which is Sync and yields *const c_char via .as_ptr().
#[cfg(not(feature = "xbox"))]
static psTagStrings: [&'static CStr; 50] = [
    c"ALL",
    c"BOTLIB",
    c"CLIENTS",              // #ifndef _XBOX block begins:
    c"BOTGAME",
    c"DOWNLOAD",             // used by the downloading system
    c"GENERAL",
    c"CLIPBOARD",
    c"SND_MP3STREAMHDR",    // specific MP3 struct for decoding (about 18..22K each?), not the actual MP3 binary
    c"SND_DYNAMICMUSIC",    // in-mem MP3 files
    c"BSP_DISKIMAGE",       // temp during loading, to save both server and renderer fread()ing the same file.
    c"VM",                  // stuff for VM, may be zapped later?
    c"SPECIAL_MEM_TEST",    // special usage for testing z_malloc recover only
                            // #ifndef _XBOX block ends
    c"HUNK_MARK1",          //hunk allocations before the mark is set
    c"HUNK_MARK2",          //hunk allocations after the mark is set
    c"EVENT",
    c"FILESYS",             // general filesystem usage
    c"GHOUL2",              // Ghoul2 stuff
    c"GHOUL2_GORE",         // Ghoul2 gore stuff
    c"LISTFILES",           // for "*.blah" lists
    c"AMBIENTSET",
    c"STATIC",              // special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
    c"SMALL",               // used by S_Malloc, but probably more of a hint now. Will be dumped later
    c"MODEL_MD3",           // specific model types' disk images
    c"MODEL_GLM",           //     "
    c"MODEL_GLA",           //     "
    c"ICARUS",              // Memory used internally by the Icarus scripting system
    //sorry, I don't want to have to keep adding these and recompiling, so there may be more than I need
    c"ICARUS2",             //for debugging mem leaks in icarus -rww
    c"ICARUS3",             //for debugging mem leaks in icarus -rww
    c"ICARUS4",             //for debugging mem leaks in icarus -rww
    c"ICARUS5",             //for debugging mem leaks in icarus -rww
    c"SHADERTEXT",
    c"SND_RAWDATA",         // raw sound data, either MP3 or WAV
    c"TEMP_WORKSPACE",      // anything like file loading or image workspace that's only temporary
    c"TEMP_PNG",            // image workspace that's only temporary
    c"TEXTPOOL",            // for some special text-pool class thingy
    c"IMAGE_T",             // an image_t struct (no longer on the hunk because of cached texture stuff)
    c"INFLATE",             // Temp memory used by zlib32
    c"DEFLATE",             // Temp memory used by zlib32//	TAGDEF(SOUNDPOOL),					// pool of mem for the sound system
    c"BSP",                 // guess.
    c"GRIDMESH",            // some specific temp workspace that only seems to be in the MP codebase
    //rwwRMG - following:
    c"POINTCACHE",          // weather system
    c"TERRAIN",             // RMG terrain management
    c"R_TERRAIN",           // terrain renderer
    c"RESAMPLE",            // terrain heightmap resampling (I think)
    c"CM_TERRAIN",          // common terrain data management
    c"CM_TERRAIN_TEMP",     // temporary terrain allocations
    c"TEMP_IMAGE",          // temporary allocations for image manipulation
    c"VM_ALLOCATED",        // allocated by game or cgame via memory shifting
    c"TEMP_HUNKALLOC",
    c"COUNT",
];

#[cfg(feature = "xbox")]
static psTagStrings: [&'static CStr; 47] = [
    c"ALL",
    c"BOTLIB",
    c"CLIENTS",             // Memory used for client info
    // (no #ifndef _XBOX block on XBOX)
    c"HUNK_MARK1",          //hunk allocations before the mark is set
    c"HUNK_MARK2",          //hunk allocations after the mark is set
    c"EVENT",
    c"FILESYS",             // general filesystem usage
    c"GHOUL2",              // Ghoul2 stuff
    c"GHOUL2_GORE",         // Ghoul2 gore stuff
    c"LISTFILES",           // for "*.blah" lists
    c"AMBIENTSET",
    c"STATIC",              // special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
    c"SMALL",               // used by S_Malloc, but probably more of a hint now. Will be dumped later
    c"MODEL_MD3",           // specific model types' disk images
    c"MODEL_GLM",           //     "
    c"MODEL_GLA",           //     "
    c"ICARUS",              // Memory used internally by the Icarus scripting system
    //sorry, I don't want to have to keep adding these and recompiling, so there may be more than I need
    c"ICARUS2",             //for debugging mem leaks in icarus -rww
    c"ICARUS3",             //for debugging mem leaks in icarus -rww
    c"ICARUS4",             //for debugging mem leaks in icarus -rww
    c"ICARUS5",             //for debugging mem leaks in icarus -rww
    c"SHADERTEXT",
    c"SND_RAWDATA",         // raw sound data, either MP3 or WAV
    c"TEMP_WORKSPACE",      // anything like file loading or image workspace that's only temporary
    c"TEMP_PNG",            // image workspace that's only temporary
    c"TEXTPOOL",            // for some special text-pool class thingy
    c"IMAGE_T",             // an image_t struct (no longer on the hunk because of cached texture stuff)
    c"INFLATE",             // Temp memory used by zlib32
    c"DEFLATE",             // Temp memory used by zlib32
    c"BSP",                 // guess.
    c"GRIDMESH",            // some specific temp workspace that only seems to be in the MP codebase
    //rwwRMG - following:
    c"POINTCACHE",          // weather system
    c"TERRAIN",             // RMG terrain management
    c"R_TERRAIN",           // terrain renderer
    c"RESAMPLE",            // terrain heightmap resampling (I think)
    c"CM_TERRAIN",          // common terrain data management
    c"CM_TERRAIN_TEMP",     // temporary terrain allocations
    c"TEMP_IMAGE",          // temporary allocations for image manipulation
    c"VM_ALLOCATED",        // allocated by game or cgame via memory shifting
    c"TEMP_HUNKALLOC",
    c"NEWDEL",              // new / delete -> Z_Malloc on Xbox
    c"UI_ALLOC",            // UI DLL calls to UI_Alloc
    c"CG_UI_ALLOC",         // Cgame DLL calls to UI_Alloc
    c"BG_ALLOC",
    c"BINK",
    c"XBL_FRIENDS",         // friends list
    c"COUNT",
];
//
////////////////////////////////////////////////

// static void Z_Details_f(void); — file-local forward decl; defined later, no declaration needed in Rust
// void CIN_CloseAllVideos(); — hoisted to extern "C" block above


// This handles zone memory allocation.
// It is a wrapper around malloc with a tag id and a magic number at the start

const ZONE_MAGIC: c_int = 0x21436587;

#[repr(C)]
struct zoneHeader_s {
        iMagic: c_int,
        eTag:   memtag_t,
        iSize:  c_int,
    pNext: *mut zoneHeader_s,
    pPrev: *mut zoneHeader_s,
}

type zoneHeader_t = zoneHeader_s;

#[repr(C)]
struct zoneTail_t {
    iMagic: c_int,
}

#[inline]
unsafe fn ZoneTailFromHeader(pHeader: *mut zoneHeader_t) -> *mut zoneTail_t {
    ((pHeader as *mut c_char)
        .add(size_of::<zoneHeader_t>())
        .add((*pHeader).iSize as usize)) as *mut zoneTail_t
}

// #ifdef DETAILED_ZONE_DEBUG_CODE
// map <void*,int> mapAllocatedZones;
// #endif
#[cfg(feature = "detailed_zone_debug_code")]
static mut mapAllocatedZones: BTreeMap<*mut c_void, c_int> = BTreeMap::new();


#[repr(C)]
struct zoneStats_s {
    iCount:   c_int,
    iCurrent: c_int,
    iPeak:    c_int,

    // I'm keeping these updated on the fly, since it's quicker for cache-pool
    //	purposes rather than recalculating each time...
    //
    iSizesPerTag:  [c_int; TAG_COUNT as usize],
    iCountsPerTag: [c_int; TAG_COUNT as usize],
}

type zoneStats_t = zoneStats_s;

#[repr(C)]
struct zone_s {
    Stats:  zoneStats_t,
    Header: zoneHeader_t,
}

type zone_t = zone_s;

pub static mut com_validateZone: *mut cvar_t = null_mut();

// zone_t TheZone = {0};
// SAFETY: zone_t is a plain C struct; zero-initialization matches C's {0} semantics
pub static mut TheZone: zone_t = unsafe { core::mem::zeroed() };


// Scans through the linked list of mallocs and makes sure no data has been overwritten

pub unsafe fn Z_Validate() {
    if com_validateZone.is_null() || (*com_validateZone).integer == 0 {
        return;
    }

    let mut pMemory: *mut zoneHeader_t = TheZone.Header.pNext;
    while !pMemory.is_null() {
        // #ifdef DETAILED_ZONE_DEBUG_CODE
        // this won't happen here, but wtf?
        // int& iAllocCount = mapAllocatedZones[pMemory];
        // if (iAllocCount <= 0)
        // {
        //     Com_Error(ERR_FATAL, "Z_Validate(): Bad block allocation count!");
        //     return;
        // }
        // #endif
        #[cfg(feature = "detailed_zone_debug_code")]
        {
            // SAFETY: mapAllocatedZones is accessed only from single-threaded engine code
            let iAllocCount: &mut c_int = {
                let map = &mut *addr_of_mut!(mapAllocatedZones);
                map.entry(pMemory as *mut c_void).or_insert(0)
            };
            if *iAllocCount <= 0 {
                Com_Error(ERR_FATAL, c"Z_Validate(): Bad block allocation count!".as_ptr());
                return;
            }
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, c"Z_Validate(): Corrupt zone header!".as_ptr());
            return;
        }

        if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
            Com_Error(ERR_FATAL, c"Z_Validate(): Corrupt zone tail!".as_ptr());
            return;
        }

        pMemory = (*pMemory).pNext;
    }
}



// static mem blocks to reduce a lot of small zone overhead
//
// #pragma pack(push)
// #pragma pack(1)
#[repr(C, packed)]
struct StaticZeroMem_t {
    Header: zoneHeader_t,
//	byte mem[0];       // commented out in original; not translated (no zero-length arrays)
    Tail: zoneTail_t,
}

#[repr(C, packed)]
struct StaticMem_t {
    Header: zoneHeader_t,
    mem: [byte; 2],
    Tail: zoneTail_t,
}
// #pragma pack(pop)

static mut gZeroMalloc: StaticZeroMem_t = StaticZeroMem_t {
    Header: zoneHeader_s {
        iMagic: ZONE_MAGIC,
        eTag:   TAG_STATIC,
        iSize:  0,
        pNext:  null_mut(),
        pPrev:  null_mut(),
    },
    Tail: zoneTail_t { iMagic: ZONE_MAGIC },
};
static mut gEmptyString: StaticMem_t = StaticMem_t {
    Header: zoneHeader_s {
        iMagic: ZONE_MAGIC,
        eTag:   TAG_STATIC,
        iSize:  2,
        pNext:  null_mut(),
        pPrev:  null_mut(),
    },
    mem: [b'\0' as byte, b'\0' as byte],
    Tail: zoneTail_t { iMagic: ZONE_MAGIC },
};
static mut gNumberString: [StaticMem_t; 10] = [
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'0' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'1' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'2' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'3' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'4' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'5' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'6' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'7' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'8' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
    StaticMem_t { Header: zoneHeader_s { iMagic: ZONE_MAGIC, eTag: TAG_STATIC, iSize: 2, pNext: null_mut(), pPrev: null_mut() }, mem: [b'9' as byte, b'\0' as byte], Tail: zoneTail_t { iMagic: ZONE_MAGIC } },
];

pub static mut gbMemFreeupOccured: qboolean = qfalse;

pub unsafe fn Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean /* = qfalse */, iUnusedAlign: c_int /* = 4 */) -> *mut c_void {
    gbMemFreeupOccured = qfalse;

    if iSize == 0 {
        let pMemory: *mut zoneHeader_t = addr_of_mut!(gZeroMalloc) as *mut zoneHeader_t;
        return pMemory.add(1) as *mut c_void;
    }

    // Add in tracking info
    //
    let iRealSize: c_int = iSize + size_of::<zoneHeader_t>() as c_int + size_of::<zoneTail_t>() as c_int;

    // Allocate a chunk...
    //
    let mut pMemory: *mut zoneHeader_t = null_mut();
    while pMemory.is_null() {
        // #ifdef _WIN32
        #[cfg(target_os = "windows")]
        {
            if gbMemFreeupOccured != qfalse {
                Sleep(1000); // sleep for a second, so Windows has a chance to shuffle mem to de-swiss-cheese it
            }
        }
        // #endif

        if bZeroit != qfalse {
            pMemory = calloc(iRealSize as usize, 1) as *mut zoneHeader_t;
        } else {
            pMemory = malloc(iRealSize as usize) as *mut zoneHeader_t;
        }
        if pMemory.is_null() {
            // new bit, if we fail to malloc memory, try dumping some of the cached stuff that's non-vital and try again...
            //

            // ditch the BSP cache...
            //
            // extern qboolean CM_DeleteCachedMap(qboolean bGuaranteedOkToDelete); — hoisted above
            if CM_DeleteCachedMap(qfalse) != qfalse {
                gbMemFreeupOccured = qtrue;
                continue; // we've just ditched a whole load of memory, so try again with the malloc
            }


            // ditch any sounds not used on this level...
            //
            // extern qboolean SND_RegisterAudio_LevelLoadEnd(qboolean bDeleteEverythingNotUsedThisLevel); — hoisted above
            if SND_RegisterAudio_LevelLoadEnd(qtrue) != qfalse {
                gbMemFreeupOccured = qtrue;
                continue; // we've dropped at least one sound, so try again with the malloc
            }

            // #ifndef DEDICATED
            #[cfg(not(feature = "dedicated"))]
            {
                // ditch any image_t's (and associated GL memory) not used on this level...
                //
                // extern qboolean RE_RegisterImages_LevelLoadEnd(void); — hoisted above
                if RE_RegisterImages_LevelLoadEnd() != qfalse {
                    gbMemFreeupOccured = qtrue;
                    continue; // we've dropped at least one image, so try again with the malloc
                }
            }
            // #endif

            // ditch the model-binaries cache...  (must be getting desperate here!)
            //
            // extern qboolean RE_RegisterModels_LevelLoadEnd(qboolean bDeleteEverythingNotUsedThisLevel); — hoisted above
            if RE_RegisterModels_LevelLoadEnd(qtrue) != qfalse {
                gbMemFreeupOccured = qtrue;
                continue;
            }

            // as a last panic measure, dump all the audio memory, but not if we're in the audio loader
            //	(which is annoying, but I'm not sure how to ensure we're not dumping any memory needed by the sound
            //	currently being loaded if that was the case)...
            //
            // note that this keeps querying until it's freed up as many bytes as the requested size, but freeing
            //	several small blocks might not mean that one larger one is satisfiable after freeup, however that'll
            //	just make it go round again and try for freeing up another bunch of blocks until the total is satisfied
            //	again (though this will have freed twice the requested amount in that case), so it'll either work
            //	eventually or not free up enough and drop through to the final ERR_DROP. No worries...
            //
            // extern qboolean gbInsideLoadSound; — hoisted above
            // extern int SND_FreeOldestSound(); — hoisted above
            if gbInsideLoadSound == qfalse {
                let mut iBytesFreed: c_int = SND_FreeOldestSound();
                if iBytesFreed != 0 {
                    let mut iTheseBytesFreed: c_int;
                    loop {
                        iTheseBytesFreed = SND_FreeOldestSound();
                        if iTheseBytesFreed == 0 { break; }
                        iBytesFreed += iTheseBytesFreed;
                        if iBytesFreed >= iRealSize {
                            break; // early opt-out since we've managed to recover enough (mem-contiguity issues aside)
                        }
                    }
                    gbMemFreeupOccured = qtrue;
                    continue;
                }
            }

            // sigh, dunno what else to try, I guess we'll have to give up and report this as an out-of-mem error...
            //
            // findlabel:  "recovermem"

            // S_COLOR_RED expands to "^1"
            Com_Printf(c"^1Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n".as_ptr(),
                iSize, psTagStrings[eTag as usize].as_ptr());
            Z_Details_f();
            Com_Error(ERR_FATAL, c"(Repeat): Z_Malloc(): Failed to alloc %d bytes (TAG_%s) !!!!!\n".as_ptr(),
                iSize, psTagStrings[eTag as usize].as_ptr());
            return null_mut();
        }
    }

    // Link in
    (*pMemory).iMagic = ZONE_MAGIC;
    (*pMemory).eTag   = eTag;
    (*pMemory).iSize  = iSize;
    (*pMemory).pNext  = TheZone.Header.pNext;
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
    TheZone.Stats.iCount   += 1;
    TheZone.Stats.iSizesPerTag [eTag as usize] += iSize;
    TheZone.Stats.iCountsPerTag[eTag as usize] += 1;

    if TheZone.Stats.iCurrent > TheZone.Stats.iPeak {
        TheZone.Stats.iPeak = TheZone.Stats.iCurrent;
    }

    // #ifdef DETAILED_ZONE_DEBUG_CODE
    // mapAllocatedZones[pMemory]++;
    // #endif
    #[cfg(feature = "detailed_zone_debug_code")]
    {
        // SAFETY: mapAllocatedZones accessed only from single-threaded engine code
        let map = &mut *addr_of_mut!(mapAllocatedZones);
        *map.entry(pMemory as *mut c_void).or_insert(0) += 1;
    }

    Z_Validate(); // check for corruption

    let pvReturnMem: *mut c_void = pMemory.add(1) as *mut c_void;
    pvReturnMem
}

// used during model cacheing to save an extra malloc, lets us morph the disk-load buffer then
//	just not fs_freefile() it afterwards.
//
pub unsafe fn Z_MorphMallocTag(pvAddress: *mut c_void, eDesiredTag: memtag_t) {
    let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).offset(-1);

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(ERR_FATAL, c"Z_MorphMallocTag(): Not a valid zone header!".as_ptr());
        return; // won't get here
    }

    // DEC existing tag stats...
    //
//	TheZone.Stats.iCurrent	- unchanged
//	TheZone.Stats.iCount	- unchanged
    TheZone.Stats.iSizesPerTag [(*pMemory).eTag as usize] -= (*pMemory).iSize;
    TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

    // morph...
    //
    (*pMemory).eTag = eDesiredTag;

    // INC new tag stats...
    //
//	TheZone.Stats.iCurrent	- unchanged
//	TheZone.Stats.iCount	- unchanged
    TheZone.Stats.iSizesPerTag [(*pMemory).eTag as usize] += (*pMemory).iSize;
    TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] += 1;
}

unsafe fn Zone_FreeBlock(pMemory: *mut zoneHeader_t) {
    if (*pMemory).eTag != TAG_STATIC { // belt and braces, should never hit this though
        // Update stats...
        //
        TheZone.Stats.iCount   -= 1;
        TheZone.Stats.iCurrent -= (*pMemory).iSize;
        TheZone.Stats.iSizesPerTag [(*pMemory).eTag as usize] -= (*pMemory).iSize;
        TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

        // Sanity checks...
        //
        assert!((*(*pMemory).pPrev).pNext == pMemory);
        assert!((*pMemory).pNext.is_null() || ((*(*pMemory).pNext).pPrev == pMemory));

        // Unlink and free...
        //
        (*(*pMemory).pPrev).pNext = (*pMemory).pNext;
        if !(*pMemory).pNext.is_null() {
            (*(*pMemory).pNext).pPrev = (*pMemory).pPrev;
        }
        free(pMemory as *mut c_void);


        // #ifdef DETAILED_ZONE_DEBUG_CODE
        // this has already been checked for in execution order, but wtf?
        // int& iAllocCount = mapAllocatedZones[pMemory];
        // if (iAllocCount == 0)
        // {
        //     Com_Error(ERR_FATAL, "Zone_FreeBlock(): Double-freeing block!");
        //     return;
        // }
        // iAllocCount--;
        // #endif
        #[cfg(feature = "detailed_zone_debug_code")]
        {
            // SAFETY: mapAllocatedZones accessed only from single-threaded engine code
            let map = &mut *addr_of_mut!(mapAllocatedZones);
            let iAllocCount: &mut c_int = map.entry(pMemory as *mut c_void).or_insert(0);
            if *iAllocCount == 0 {
                Com_Error(ERR_FATAL, c"Zone_FreeBlock(): Double-freeing block!".as_ptr());
                return;
            }
            *iAllocCount -= 1;
        }
    }
}

// stats-query function to ask how big a malloc is...
//
pub unsafe fn Z_Size(pvAddress: *mut c_void) -> c_int {
    let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).offset(-1);

    if (*pMemory).eTag == TAG_STATIC {
        return 0; // kind of
    }

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(ERR_FATAL, c"Z_Size(): Not a valid zone header!".as_ptr());
        return 0; // won't get here
    }

    (*pMemory).iSize
}


// Frees a block of memory...
//
pub unsafe fn Z_Free(pvAddress: *mut c_void) {
    if pvAddress.is_null() { // I've put this in as a safety measure because of some bits of #ifdef BSPC stuff	-Ste.
        //Com_Error(ERR_FATAL, "Z_Free(): NULL arg");
        return;
    }

    let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).offset(-1);

    if (*pMemory).eTag == TAG_STATIC {
        return;
    }

    // #ifdef DETAILED_ZONE_DEBUG_CODE
    //
    // check this error *before* barfing on bad magics...
    //
    // int& iAllocCount = mapAllocatedZones[pMemory];
    // if (iAllocCount <= 0)
    // {
    //     Com_Error(ERR_FATAL, "Z_Free(): Block already-freed, or not allocated through Z_Malloc!");
    //     return;
    // }
    // #endif
    #[cfg(feature = "detailed_zone_debug_code")]
    {
        // SAFETY: mapAllocatedZones accessed only from single-threaded engine code
        let map = &mut *addr_of_mut!(mapAllocatedZones);
        let iAllocCount: &mut c_int = map.entry(pMemory as *mut c_void).or_insert(0);
        if *iAllocCount <= 0 {
            Com_Error(ERR_FATAL, c"Z_Free(): Block already-freed, or not allocated through Z_Malloc!".as_ptr());
            return;
        }
    }

    if (*pMemory).iMagic != ZONE_MAGIC {
        Com_Error(ERR_FATAL, c"Z_Free(): Corrupt zone header!".as_ptr());
        return;
    }
    if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
        Com_Error(ERR_FATAL, c"Z_Free(): Corrupt zone tail!".as_ptr());
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
//#ifdef _DEBUG
//	int iZoneBlocks = TheZone.Stats.iCount;
//#endif

    let mut pMemory: *mut zoneHeader_t = TheZone.Header.pNext;
    while !pMemory.is_null() {
        let pNext: *mut zoneHeader_t = (*pMemory).pNext;
        if (eTag == TAG_ALL) || ((*pMemory).eTag == eTag) {
            Zone_FreeBlock(pMemory);
        }
        pMemory = pNext;
    }

// these stupid pragmas don't work here???!?!?!
//
//#ifdef _DEBUG
//#pragma warning( disable : 4189)
//	int iBlocksFreed = iZoneBlocks - TheZone.Stats.iCount;
//#pragma warning( default : 4189)
//#endif
}


pub unsafe fn S_Malloc(iSize: c_int) -> *mut c_void {
    Z_Malloc(iSize, TAG_SMALL, qfalse, 4)
}


// #ifdef _DEBUG
#[cfg(debug_assertions)]
unsafe extern "C" fn Z_MemRecoverTest_f() {
    // needs to be in _DEBUG only, not good for final game!
    // fixme: findmeste: Remove this sometime
    //
    let mut iTotalMalloc: c_int = 0;
    loop {
        let iThisMalloc: c_int = 5 * (1024 * 1024);
        Z_Malloc(iThisMalloc, TAG_SPECIAL_MEM_TEST, qfalse, 4); // and lose, just to consume memory
        iTotalMalloc += iThisMalloc;

        if gbMemFreeupOccured != qfalse {
            break;
        }
    }

    Z_TagFree(TAG_SPECIAL_MEM_TEST);
}
// #endif



// Gives a summary of the zone memory usage

unsafe extern "C" fn Z_Stats_f() {
    Com_Printf(c"\nThe zone is using %d bytes (%.2fMB) in %d memory blocks\n".as_ptr(),
                              TheZone.Stats.iCurrent,
                                        (TheZone.Stats.iCurrent as f32) / 1024.0f32 / 1024.0f32,
                                                  TheZone.Stats.iCount
    );

    Com_Printf(c"The zone peaked at %d bytes (%.2fMB)\n".as_ptr(),
                            TheZone.Stats.iPeak,
                                     (TheZone.Stats.iPeak as f32) / 1024.0f32 / 1024.0f32
    );
}

// Gives a detailed breakdown of the memory blocks in the zone

unsafe extern "C" fn Z_Details_f() {
    Com_Printf(c"---------------------------------------------------------------------------\n".as_ptr());
    Com_Printf(c"%20s %9s\n".as_ptr(), c"Zone Tag".as_ptr(), c"Bytes".as_ptr());
    Com_Printf(c"%20s %9s\n".as_ptr(), c"--------".as_ptr(), c"-----".as_ptr());
    let mut i: c_int = 0;
    while i < TAG_COUNT {
        let iThisCount: c_int = TheZone.Stats.iCountsPerTag[i as usize];
        let iThisSize:  c_int = TheZone.Stats.iSizesPerTag [i as usize];

        if iThisCount != 0 {
            // can you believe that using %2.2f as a format specifier doesn't bloody work?
            //	It ignores the left-hand specifier. Sigh, now I've got to do shit like this...
            //
            let fSize:      f32 = (iThisSize as f32) / 1024.0f32 / 1024.0f32;
            let iSize:      c_int = fSize as c_int;
            let iRemainder: c_int = (100.0f32 * (fSize - fSize.floor())) as c_int;
            Com_Printf(c"%20s %9d (%2d.%02dMB) in %6d blocks (%9d average)\n".as_ptr(),
                    psTagStrings[i as usize].as_ptr(),
                          iThisSize,
                            iSize, iRemainder,
                                       iThisCount, iThisSize / iThisCount
               );
        }
        i += 1;
    }
    Com_Printf(c"---------------------------------------------------------------------------\n".as_ptr());

    Z_Stats_f();
}

// Shuts down the zone memory system and frees up all memory
pub unsafe fn Com_ShutdownZoneMemory() {
//	Com_Printf("Shutting down zone memory .....\n");

    Cmd_RemoveCommand(c"zone_stats".as_ptr());
    Cmd_RemoveCommand(c"zone_details".as_ptr());

    if TheZone.Stats.iCount != 0 {
        Com_Printf(c"Automatically freeing %d blocks making up %d bytes\n".as_ptr(),
            TheZone.Stats.iCount, TheZone.Stats.iCurrent);
        Z_TagFree(TAG_ALL);

        assert!(TheZone.Stats.iCount   == 0);
        assert!(TheZone.Stats.iCurrent == 0);
    }
}

// Initialises the zone memory system

pub unsafe fn Com_InitZoneMemory() {
    memset(addr_of_mut!(TheZone) as *mut c_void, 0, size_of::<zone_t>());
    TheZone.Header.iMagic = ZONE_MAGIC;

//#ifdef _DEBUG
//	com_validateZone = Cvar_Get("com_validateZone", "1", 0);
//#else
    com_validateZone = Cvar_Get(c"com_validateZone".as_ptr(), c"0".as_ptr(), 0);
//#endif

    Cmd_AddCommand(c"zone_stats".as_ptr(),   Z_Stats_f);
    Cmd_AddCommand(c"zone_details".as_ptr(), Z_Details_f);

// #ifdef _DEBUG
#[cfg(debug_assertions)]
    Cmd_AddCommand(c"zone_memrecovertest".as_ptr(), Z_MemRecoverTest_f);
// #endif
}




/*
========================
CopyString

 NOTE:	never write over the memory CopyString returns because
		memory from a memstatic_t might be returned
========================
*/
pub unsafe fn CopyString(in_: *const c_char) -> *mut c_char {
    let mut out: *mut c_char;

    if *in_ == 0 {
        return (addr_of_mut!(gEmptyString) as *mut c_char).add(size_of::<zoneHeader_t>());
    } else if *in_.add(1) == 0 {
        if *in_ >= b'0' as c_char && *in_ <= b'9' as c_char {
            return (addr_of_mut!(gNumberString[(*in_ - b'0' as c_char) as usize]) as *mut c_char)
                .add(size_of::<zoneHeader_t>());
        }
    }

    out = S_Malloc((strlen(in_) + 1) as c_int) as *mut c_char;
    strcpy(out, in_);
    out
}



static mut hunk_tag: memtag_t = 0 as memtag_t;


/*
===============
Com_TouchMemory

Touch all known used data to make sure it is paged in
===============
*/
pub unsafe fn Com_TouchMemory() {
//	int		start, end;
    let mut i: c_int;
    let mut j: c_int;
    let mut sum: c_int;

//	start = Sys_Milliseconds();
    Z_Validate();

    sum = 0;

    let mut pMemory: *mut zoneHeader_t = TheZone.Header.pNext;
    while !pMemory.is_null() {
        let pMem: *mut byte = pMemory.add(1) as *mut byte;
        j = (*pMemory).iSize >> 2;
        i = 0;
        while i < j {
            sum += *(pMem as *mut c_int).add(i as usize);
            i += 64;
        }

        pMemory = (*pMemory).pNext;
    }

//	end = Sys_Milliseconds();
//	Com_Printf( "Com_TouchMemory: %i msec\n", end - start );
}



pub unsafe fn Com_TheHunkMarkHasBeenMade() -> qboolean {
    if hunk_tag == TAG_HUNK_MARK2 {
        return qtrue;
    }
    qfalse
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
    //Er, ok. Clear it then I guess.
    Z_TagFree(TAG_HUNK_MARK1);
    Z_TagFree(TAG_HUNK_MARK2);
}

/*
====================
Hunk_MemoryRemaining
====================
*/
pub unsafe fn Hunk_MemoryRemaining() -> c_int {
    (64*1024*1024) - (Z_MemSize(TAG_HUNK_MARK1)+Z_MemSize(TAG_HUNK_MARK2)) //Yeah. Whatever. We've got no size now.
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
    assert!(hunk_tag == TAG_HUNK_MARK2); //if this is not true then no mark has been made
    Z_TagFree(TAG_HUNK_MARK2);
}

/*
=================
Hunk_CheckMark
=================
*/
pub unsafe fn Hunk_CheckMark() -> qboolean {
    //if( hunk_low.mark || hunk_high.mark ) {
    if hunk_tag != TAG_HUNK_MARK1 {
        return qtrue;
    }
    qfalse
}

// void CL_ShutdownCGame( void ); — hoisted to extern "C" block above
// void CL_ShutdownUI( void );    — hoisted to extern "C" block above
// void SV_ShutdownGameProgs( void ); — hoisted to extern "C" block above

/*
=================
Hunk_Clear

The server calls this before shutting down or loading a new map
=================
*/
// void R_HunkClearCrap(void); — hoisted to extern "C" block above
// #ifdef _FULL_G2_LEAK_CHECKING
// void G2_DEBUG_ReportLeaks(void); — hoisted to cfg(feature="full_g2_leak_checking") extern "C" block above
// #endif

pub unsafe fn Hunk_Clear() {

// #ifndef DEDICATED
#[cfg(not(feature = "dedicated"))]
    {
        CL_ShutdownCGame();
        CL_ShutdownUI();
    }
// #endif
    SV_ShutdownGameProgs();

// #ifndef DEDICATED
#[cfg(not(feature = "dedicated"))]
    {
        CIN_CloseAllVideos();
    }
// #endif

    hunk_tag = TAG_HUNK_MARK1;
    Z_TagFree(TAG_HUNK_MARK1);
    Z_TagFree(TAG_HUNK_MARK2);

    R_HunkClearCrap();

//	Com_Printf( "Hunk_Clear: reset the hunk ok\n" );
    VM_Clear();

//See if any ghoul2 stuff was leaked, at this point it should be all cleaned up.
// #ifdef _FULL_G2_LEAK_CHECKING
#[cfg(feature = "full_g2_leak_checking")]
    {
        assert!(g_Ghoul2Allocations == 0 && g_G2ClientAlloc == 0 && g_G2ServerAlloc == 0);
        if g_Ghoul2Allocations != 0 {
            Com_Printf(c"%i bytes leaked by ghoul2 routines (%i client, %i server)\n".as_ptr(),
                g_Ghoul2Allocations, g_G2ClientAlloc, g_G2ServerAlloc);
            G2_DEBUG_ReportLeaks();
        }
    }
// #endif
}

/*
=================
Hunk_Alloc

Allocate permanent (until the hunk is cleared) memory
=================
*/
pub unsafe fn Hunk_Alloc(size: c_int, _preference: ha_pref) -> *mut c_void {
    Z_Malloc(size, hunk_tag, qtrue, 4)
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
    Z_Malloc(size, TAG_TEMP_HUNKALLOC, qfalse, 4)
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
