/*
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 */

/*
 *	ZONE MEMORY MANAGER
 *
 *	Goals:
 *		1. Minimize overhead
 *		2. Minimize fragmentation
 *
 *	Constraints:
 *		1. Maximum allocated block size is 32MB
 *		2. Maximum 64 different memory tags supported
 *		3. Maximum 256 byte alignment
 *
 *	All memory required by the manager is allocated at startup in
 *	the form of one large pool.
 *
 *	Allocated blocks require a 4 byte header to store size, tag, and
 *	alignment information.  Blocks that need to support the Z_TagFree()
 *	feature require an additional 8 byte link list structure.
 *
 *	Free blocks require a 16 bytes of tracking information.  If possible
 *	this information is stored directly in the block (which is in the
 *	pool.)  If the free block is not large enough, its information is
 *	stored in an overflow buffer.
 *
 *	In an effort to reduce fragmentation, blocks allocated for a short
 *	period of time at the end of the pool.  All other blocks are allocated
 *	at the start.  Allocation is first fit.
 *
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// Where do hunk allocations go?
static mut hunk_tag: memtag_t = 0;

// Used to mark the start and end of blocks in debug mode
const ZONE_MAGIC: c_int = 0xfe;

// Size of the free block overflow buffer
const ZONE_FREE_OVERFLOW: usize = 4096;

// Indicates whether or not special (slow) debug code should be enabled
const ZONE_DEBUG: c_int = 0;

// Allocate all available memory minus this amount
#[cfg(target_os = "gamecube")]
const ZONE_HEAP_FREE_DEBUG: usize = 64 * 1024 * 4;
#[cfg(target_os = "gamecube")]
const ZONE_HEAP_FREE_RELEASE: usize = 0;

#[cfg(all(target_os = "gamecube", debug_assertions))]
const ZONE_HEAP_FREE: usize = ZONE_HEAP_FREE_DEBUG;
#[cfg(all(target_os = "gamecube", not(debug_assertions)))]
const ZONE_HEAP_FREE: usize = ZONE_HEAP_FREE_RELEASE;

#[cfg(not(target_os = "gamecube"))]
// Game needs about 8 MB for framebuffers audio, bink, etc., plus 24 MB
// for textures.
// #	define ZONE_HEAP_FREE (1024*1024*8 + 24*1024*1024)
const ZONE_HEAP_FREE: usize = 1024 * 1024 * 8 + 24 * 1024 * 1024 + 4 * 1024 * 1024;

// Should we emulate the smaller memory footprint of actual release systems?
const ZONE_EMULATE_SPACE: c_int = 0;

// All standard header data is crammed into 4 bytes
pub type ZoneHeader = c_int;

// Debug markers to check for overflow/underflow
pub type ZoneDebugHeader = c_int;
pub type ZoneDebugFooter = c_int;

// Extended header information for memory freed with TagFree()
#[repr(C)]
pub struct ZoneLinkHeader {
    m_Next: *mut ZoneLinkHeader,
    m_Prev: *mut ZoneLinkHeader,
}

static mut s_LinkBase: *mut ZoneLinkHeader = null_mut();

// Free memory block tracking information
#[repr(C)]
pub struct ZoneFreeBlock {
    m_Address: c_int,
    m_Size: c_int,
    m_Next: *mut ZoneFreeBlock,
    m_Prev: *mut ZoneFreeBlock,
}

// Buffer to hold free memory information that we can't
// fit directly in the pool
static mut s_FreeOverflow: [ZoneFreeBlock; ZONE_FREE_OVERFLOW] = [ZoneFreeBlock {
    m_Address: 0,
    m_Size: 0,
    m_Next: null_mut(),
    m_Prev: null_mut(),
}; ZONE_FREE_OVERFLOW];
static mut s_LastOverflowIndex: c_int = 0;

static mut s_FreeStart: ZoneFreeBlock = ZoneFreeBlock {
    m_Address: 0,
    m_Size: 0,
    m_Next: null_mut(),
    m_Prev: null_mut(),
};
static mut s_FreeEnd: ZoneFreeBlock = ZoneFreeBlock {
    m_Address: 0,
    m_Size: 0,
    m_Next: null_mut(),
    m_Prev: null_mut(),
};

// Various stats collected at runtime
#[repr(C)]
pub struct ZoneStats {
    m_CountAlloc: c_int,
    m_SizeAlloc: c_int,
    m_OverheadAlloc: c_int,
    m_PeakAlloc: c_int,
    m_CountFree: c_int,
    m_SizeFree: c_int,
    m_SizesPerTag: [c_int; TAG_COUNT as usize],
    m_CountsPerTag: [c_int; TAG_COUNT as usize],
}

static mut s_Stats: ZoneStats = ZoneStats {
    m_CountAlloc: 0,
    m_SizeAlloc: 0,
    m_OverheadAlloc: 0,
    m_PeakAlloc: 0,
    m_CountFree: 0,
    m_SizeFree: 0,
    m_SizesPerTag: [0; TAG_COUNT as usize],
    m_CountsPerTag: [0; TAG_COUNT as usize],
};

// Special empty block for zero size allocations
#[repr(C)]
pub struct ZoneEmptyBlock {
    header: ZoneHeader,
    #[cfg(debug_assertions)]
    start: ZoneDebugHeader,
    #[cfg(debug_assertions)]
    end: ZoneDebugFooter,
}

#[cfg(debug_assertions)]
static s_EmptyBlock: ZoneEmptyBlock = ZoneEmptyBlock {
    header: (TAG_STATIC << 25) as ZoneHeader,
    start: ZONE_MAGIC,
    end: ZONE_MAGIC,
};

#[cfg(not(debug_assertions))]
static s_EmptyBlock: ZoneEmptyBlock = ZoneEmptyBlock {
    header: (TAG_STATIC << 25) as ZoneHeader,
};

// Free block jump table for fast memory deallocation
const Z_JUMP_TABLE_SIZE: usize = 64;
static mut s_FreeJumpTable: [*mut ZoneFreeBlock; Z_JUMP_TABLE_SIZE] = [null_mut(); Z_JUMP_TABLE_SIZE];
static mut s_FreeJumpResolution: c_int = 0;

static mut s_PoolBase: *mut c_void = null_mut();
static mut s_Initialized: bool = false;
static mut s_IsNewDeleteTemp: bool = false;

#[cfg(not(target_os = "gamecube"))]
static mut s_Mutex: *mut c_void = null_mut(); // HANDLE on Windows

// Declare memtag_t if not already defined; stubs for external functions
pub type memtag_t = c_int;

const TAG_COUNT: c_int = 64;
const TAG_STATIC: c_int = 0;
const TAG_TEMP_WORKSPACE: c_int = 1;
const TAG_TEMP_SAVEGAME_WORKSPACE: c_int = 2;
const TAG_STRING: c_int = 3;
const TAG_GP2: c_int = 4;
const TAG_SND_RAWDATA: c_int = 5;
const TAG_ICARUS: c_int = 6;
const TAG_TEXTPOOL: c_int = 7;
const TAG_TEMP_HUNKALLOC: c_int = 8;
const TAG_LISTFILES: c_int = 9;
const TAG_BSP: c_int = 10;
const TAG_HUNKALLOC: c_int = 11;
const TAG_HUNKMISCMODELS: c_int = 12;
const TAG_G_ALLOC: c_int = 13;
const TAG_CG_UI_ALLOC: c_int = 14;
const TAG_BG_ALLOC: c_int = 15;
const TAG_HUNK_MARK1: c_int = 16;
const TAG_HUNK_MARK2: c_int = 17;
const TAG_UI_ALLOC: c_int = 18;
const TAG_CM_TERRAIN: c_int = 19;
const TAG_CM_TERRAIN_TEMP: c_int = 20;
const TAG_TERRAIN: c_int = 21;
const TAG_R_TERRAIN: c_int = 22;
const TAG_MODEL_GLM: c_int = 23;
const TAG_MODEL_GLA: c_int = 24;
const TAG_MODEL_MD3: c_int = 25;
const TAG_BINK: c_int = 26;
const TAG_NEWDEL: c_int = 27;
const TAG_SMALL: c_int = 28;
const TAG_CLIENTS: c_int = 29;
const TAG_ALL: c_int = -1;

// Forward declarations
extern "C" {
    fn Z_Stats_f();
    fn Z_Details_f();
    fn Z_DumpMemMap_f();
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn Cmd_RemoveCommand(name: *const c_char);
    fn Com_InitZoneMemory();
    fn Com_ShutdownZoneMemory();
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Sys_Log(file: *const c_char, msg: *const c_char, len: c_int, append: bool);
    fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn qglBeginFrame();
    fn qglClearColor(r: f32, g: f32, b: f32, a: f32);
    fn qglClear(mask: c_int);
    fn qglEndFrame();

    #[cfg(target_os = "windows")]
    fn GlobalAlloc(flags: c_int, size: usize) -> *mut c_void;

    #[cfg(target_os = "windows")]
    fn GlobalFree(mem: *mut c_void) -> *mut c_void;

    #[cfg(target_os = "windows")]
    fn CreateMutex(sa: *mut c_void, owner: bool, name: *const c_char) -> *mut c_void;

    #[cfg(target_os = "windows")]
    fn WaitForSingleObject(handle: *mut c_void, timeout: c_int) -> c_int;

    #[cfg(target_os = "windows")]
    fn ReleaseMutex(handle: *mut c_void) -> bool;

    #[cfg(target_os = "windows")]
    fn CloseHandle(handle: *mut c_void) -> bool;

    fn R_HunkClearCrap();
}

pub type qboolean = c_int;
const qfalse: qboolean = 0;
const qtrue: qboolean = 1;

const GL_COLOR_BUFFER_BIT: c_int = 0x4000;

const ERR_FATAL: c_int = 1;

const INVALID_HANDLE_VALUE: *mut c_void = -1 as *mut c_void;

#[cfg(target_os = "windows")]
const INFINITE: c_int = -1;

#[cfg(target_os = "xbox")]
extern "C" {
    fn GlobalMemoryStatus(stat: *mut c_void);
}

#[cfg(target_os = "xbox")]
#[repr(C)]
struct MEMORYSTATUS {
    dwTotalPhys: c_int,
    dwAvailPhys: c_int,
}

#[cfg(target_os = "xbox")]
pub fn ShowOSMemory() {
    unsafe {
        let mut stat: MEMORYSTATUS = core::mem::zeroed();
        GlobalMemoryStatus(&mut stat as *mut _ as *mut c_void);
        Com_Printf(
            b"     total mem: %d, free mem: %d\n\0".as_ptr() as *const c_char,
            stat.dwTotalPhys / 1024,
            stat.dwAvailPhys / 1024,
        );
        // File I/O would need to be implemented
    }
}

pub fn Z_MemFree() -> c_int {
    unsafe { s_Stats.m_SizeFree }
}

pub unsafe fn Com_InitZoneMemory() {
    // assert(!s_Initialized);
    // Zone now initializes on first use, can't reliably assume anything here
    if s_Initialized {
        return;
    }

    Com_Printf(b"Initialising zone memory .....\n\0".as_ptr() as *const c_char);

    // Clear some globals
    memset(addr_of_mut!(s_Stats) as *mut c_void, 0, core::mem::size_of::<ZoneStats>());
    memset(
        addr_of_mut!(s_FreeOverflow) as *mut c_void,
        0,
        core::mem::size_of_val(&s_FreeOverflow),
    );
    s_LastOverflowIndex = 0;
    s_LinkBase = null_mut();
    s_IsNewDeleteTemp = false;

    // Alloc the pool
    #[cfg(target_os = "xbox")]
    {
        let mut status: MEMORYSTATUS = core::mem::zeroed();
        GlobalMemoryStatus(&mut status as *mut _ as *mut c_void);

        // BTO : VVFIXME - Extra little note to see how much memory
        // is being used by globals/statics
        Com_Printf(
            b"*** PhysRAM: %d used, %d free\n\0".as_ptr() as *const c_char,
            status.dwTotalPhys - status.dwAvailPhys,
            status.dwAvailPhys,
        );

        let size: usize = if ZONE_EMULATE_SPACE != 0 {
            let exe = if cfg!(debug_assertions) {
                // Emulated space is always about 6 megs off from release build.  Try
                // to compensate.  This number may need tweaking in the future.
                6500 * 1024
            } else {
                0 // Exe size is already reflected in GlobalMemoryStatus().
            };
            0x4000000 - (exe + ZONE_HEAP_FREE)
        } else {
            status.dwAvailPhys - ZONE_HEAP_FREE
        };
        s_PoolBase = GlobalAlloc(0, size);
    }

    #[cfg(all(target_os = "windows", not(target_os = "xbox")))]
    {
        let size: usize = 50 * 1024 * 1024;
        s_PoolBase = GlobalAlloc(0, size);
    }

    // Setup the initial free block
    let base = s_PoolBase as *mut ZoneFreeBlock;
    (*base).m_Address = s_PoolBase as c_int;
    (*base).m_Size = if cfg!(target_os = "xbox") {
        0 // size would be from above
    } else if cfg!(target_os = "windows") {
        (50 * 1024 * 1024) as c_int
    } else {
        0
    };
    (*base).m_Next = addr_of_mut!(s_FreeEnd) as *mut ZoneFreeBlock;
    (*base).m_Prev = addr_of_mut!(s_FreeStart) as *mut ZoneFreeBlock;

    // Init the free block jump table
    memset(
        addr_of_mut!(s_FreeJumpTable) as *mut c_void,
        0,
        Z_JUMP_TABLE_SIZE * core::mem::size_of::<*mut ZoneFreeBlock>(),
    );
    s_FreeJumpResolution = ((*base).m_Size / (Z_JUMP_TABLE_SIZE as c_int)) + 1;
    s_FreeJumpTable[0] = base;

    // Setup free block dummies
    s_FreeStart.m_Address = 0;
    s_FreeStart.m_Size = 0;
    s_FreeStart.m_Next = base;
    s_FreeStart.m_Prev = null_mut();

    s_FreeEnd.m_Address = 0xFFFFFFFF;
    s_FreeEnd.m_Size = 0;
    s_FreeEnd.m_Next = null_mut();
    s_FreeEnd.m_Prev = base;

    s_Stats.m_CountFree = 1;
    s_Stats.m_SizeFree = (*base).m_Size;

    s_Initialized = true;

    // Add some commands
    Cmd_AddCommand(b"zone_stats\0".as_ptr() as *const c_char, Z_Stats_f);
    Cmd_AddCommand(b"zone_details\0".as_ptr() as *const c_char, Z_Details_f);
    Cmd_AddCommand(b"zone_memmap\0".as_ptr() as *const c_char, Z_DumpMemMap_f);

    #[cfg(not(target_os = "gamecube"))]
    {
        s_Mutex = CreateMutex(null_mut(), false, null_mut());
    }
}

pub unsafe fn Com_ShutdownZoneMemory() {
    assert!(s_Initialized);

    // Remove commands
    Cmd_RemoveCommand(b"zone_stats\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand(b"zone_details\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand(b"zone_memmap\0".as_ptr() as *const c_char);

    if s_Stats.m_CountAlloc != 0 {
        // Free all memory
        // CM_ReleaseVisData();
        Z_TagFree(TAG_ALL);
    }

    // Clear some globals
    memset(addr_of_mut!(s_Stats) as *mut c_void, 0, core::mem::size_of::<ZoneStats>());
    memset(
        addr_of_mut!(s_FreeOverflow) as *mut c_void,
        0,
        core::mem::size_of_val(&s_FreeOverflow),
    );
    s_LastOverflowIndex = 0;
    s_LinkBase = null_mut();

    // Free the pool
    #[cfg(not(target_os = "gamecube"))]
    {
        GlobalFree(s_PoolBase);
        CloseHandle(s_Mutex);
    }

    s_PoolBase = null_mut();
    s_Initialized = false;
}

// Determine if a tag should only be allocated for a very
// short period of time.
static fn Z_IsTagTemp(eTag: memtag_t) -> bool {
    eTag == TAG_TEMP_WORKSPACE
        || {
            #[cfg(not(feature = "jk2mp"))]
            {
                eTag == TAG_TEMP_SAVEGAME_WORKSPACE || eTag == TAG_STRING || eTag == TAG_GP2
            }
            #[cfg(feature = "jk2mp")]
            {
                false
            }
        }
        || eTag == TAG_SND_RAWDATA
        || eTag == TAG_ICARUS
        || {
            #[cfg(feature = "jk2mp")]
            {
                eTag == TAG_TEXTPOOL || eTag == TAG_TEMP_HUNKALLOC
            }
            #[cfg(not(feature = "jk2mp"))]
            {
                false
            }
        }
        || eTag == TAG_LISTFILES
}

// Determine if a tag needs TagFree() support.
static fn Z_IsTagLinked(eTag: memtag_t) -> bool {
    eTag == TAG_BSP
        || {
            #[cfg(not(feature = "jk2mp"))]
            {
                eTag == TAG_HUNKALLOC
                    || eTag == TAG_HUNKMISCMODELS
                    || eTag == TAG_G_ALLOC
            }
            #[cfg(feature = "jk2mp")]
            {
                false
            }
        }
        || {
            #[cfg(feature = "jk2mp")]
            {
                eTag == TAG_CG_UI_ALLOC
                    || eTag == TAG_BG_ALLOC
                    || eTag == TAG_HUNK_MARK1
                    || eTag == TAG_HUNK_MARK2
                    || eTag == TAG_TEMP_HUNKALLOC
            }
            #[cfg(not(feature = "jk2mp"))]
            {
                false
            }
        }
        || eTag == TAG_UI_ALLOC
}

static fn Z_CalcAlignmentPad(
    iAlign: c_int,
    iAddress: c_int,
    iOffset: c_int,
    iSize: c_int,
    iHeaderSize: c_int,
    iFooterSize: c_int,
) -> c_int {
    if iAlign == 0 {
        return 0;
    }

    let align_size = if iOffset == 0 {
        // Align data at low end of block
        iAlign - ((iAddress + iHeaderSize) % iAlign)
    } else {
        // Align data at high end of block
        let block_start = iAddress + iOffset - iSize + iHeaderSize;
        block_start % iAlign
    };

    if align_size == iAlign {
        0
    } else {
        align_size
    }
}

unsafe fn Z_GetOverflowBlock() -> *mut ZoneFreeBlock {
    for i in s_LastOverflowIndex as usize..ZONE_FREE_OVERFLOW {
        if s_FreeOverflow[i].m_Address == 0 {
            s_LastOverflowIndex = i as c_int;
            return addr_of_mut!(s_FreeOverflow[i]) as *mut ZoneFreeBlock;
        }
    }

    for j in 0..s_LastOverflowIndex as usize {
        if s_FreeOverflow[j].m_Address == 0 {
            s_LastOverflowIndex = j as c_int;
            return addr_of_mut!(s_FreeOverflow[j]) as *mut ZoneFreeBlock;
        }
    }

    null_mut()
}

static fn Z_IsFreeBlockLargeEnough(
    pBlock: *mut ZoneFreeBlock,
    iSize: c_int,
    iHeaderSize: c_int,
    iFooterSize: c_int,
    iAlign: c_int,
    bLow: bool,
    iAlignPad: &mut c_int,
) -> bool {
    unsafe {
        // Is the block large enough?
        if (*pBlock).m_Size >= iSize {
            if iAlign > 0 {
                // If we need some aligment, we need to check size
                // against that as well.
                *iAlignPad = Z_CalcAlignmentPad(
                    iAlign,
                    (*pBlock).m_Address,
                    if !bLow { (*pBlock).m_Size } else { 0 },
                    iSize,
                    iHeaderSize,
                    iFooterSize,
                );

                if (*pBlock).m_Size < *iAlignPad + iSize {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

unsafe fn Z_FindFirstFree(
    iSize: c_int,
    iHeaderSize: c_int,
    iFooterSize: c_int,
    iAlign: c_int,
    iAlignPad: &mut c_int,
) -> *mut ZoneFreeBlock {
    let mut block = s_FreeStart.m_Next;
    loop {
        if block.is_null() {
            return null_mut();
        }
        if Z_IsFreeBlockLargeEnough(block, iSize, iHeaderSize, iFooterSize, iAlign, true, iAlignPad) {
            return block;
        }
        block = (*block).m_Next;
    }
}

unsafe fn Z_FindLastFree(
    iSize: c_int,
    iHeaderSize: c_int,
    iFooterSize: c_int,
    iAlign: c_int,
    iAlignPad: &mut c_int,
) -> *mut ZoneFreeBlock {
    let mut block = s_FreeEnd.m_Prev;
    loop {
        if block.is_null() {
            return null_mut();
        }
        if Z_IsFreeBlockLargeEnough(block, iSize, iHeaderSize, iFooterSize, iAlign, false, iAlignPad) {
            return block;
        }
        block = (*block).m_Prev;
    }
}

static fn Z_ValidateFree() -> bool {
    if ZONE_DEBUG != 0 {
        // Make sure no free blocks are overlapping
        let mut a = addr_of_mut!(s_FreeStart);
        loop {
            if a.is_null() {
                break;
            }
            unsafe {
                if (*a).m_Address == 0 && (*a).m_Size != 0 {
                    return false;
                }

                let mut b = addr_of_mut!(s_FreeStart);
                loop {
                    if b.is_null() {
                        break;
                    }
                    if a != b
                        && (*a).m_Address >= (*b).m_Address
                        && (*a).m_Address < (*b).m_Address + (*b).m_Size
                    {
                        return false;
                    }
                    b = (*b).m_Next;
                }

                a = (*a).m_Next;
            }
        }
    }

    true
}

static fn Z_ValidateLinks() -> bool {
    if ZONE_DEBUG != 0 {
        // Make sure links are sane
        unsafe {
            let mut a = s_LinkBase;
            loop {
                if a.is_null() {
                    break;
                }
                if (!(*a).m_Next.is_null() && a != (*(*a).m_Next).m_Prev)
                    || (!(*a).m_Prev.is_null() && a != (*(*a).m_Prev).m_Next)
                {
                    return false;
                }
                a = (*a).m_Next;
            }
        }
    }

    true
}

static fn Z_GetJumpTableIndex(iAddress: c_int) -> c_int {
    unsafe {
        let index = (iAddress - s_PoolBase as c_int) / s_FreeJumpResolution;
        if index < 0 {
            return 0;
        }
        if index >= Z_JUMP_TABLE_SIZE as c_int {
            return (Z_JUMP_TABLE_SIZE - 1) as c_int;
        }
        index
    }
}

unsafe fn Z_GetFreeBlockBefore(iAddress: c_int) -> *mut ZoneFreeBlock {
    // Find this block's position in the jump table
    let mut index = Z_GetJumpTableIndex(iAddress) - 1;

    // Find a valid jump table entry
    while index >= 0 && s_FreeJumpTable[index as usize].is_null() {
        index -= 1;
    }

    if index < 0 {
        addr_of_mut!(s_FreeStart) as *mut ZoneFreeBlock
    } else {
        s_FreeJumpTable[index as usize]
    }
}

unsafe fn Z_RemoveFromJumpTable(pBlock: *mut ZoneFreeBlock) {
    // Is this block in the jump table?
    let index = Z_GetJumpTableIndex((*pBlock).m_Address) as usize;
    if s_FreeJumpTable[index] == pBlock {
        // See if the next block will fit in our slot
        if (*pBlock).m_Next != addr_of_mut!(s_FreeEnd) as *mut ZoneFreeBlock {
            let nindex = Z_GetJumpTableIndex((*(*pBlock).m_Next).m_Address) as usize;
            if nindex == index {
                s_FreeJumpTable[index] = (*pBlock).m_Next;
                return;
            }
        }

        // See if the previous block will fit in our slot
        if (*pBlock).m_Prev != addr_of_mut!(s_FreeStart) as *mut ZoneFreeBlock {
            let pindex = Z_GetJumpTableIndex((*(*pBlock).m_Prev).m_Address) as usize;
            if pindex == index {
                s_FreeJumpTable[index] = (*pBlock).m_Prev;
                return;
            }
        }

        // No other free blocks fit here, give up
        s_FreeJumpTable[index] = null_mut();
    }
}

unsafe fn Z_LinkFreeBlock(pBlock: *mut ZoneFreeBlock) {
    let mut cur = Z_GetFreeBlockBefore((*pBlock).m_Address);
    loop {
        if cur.is_null() {
            break;
        }
        // Find the correct position, ordered by address
        if (*cur).m_Address > (*pBlock).m_Address {
            // Link up the block
            (*pBlock).m_Next = cur;
            (*pBlock).m_Prev = (*cur).m_Prev;
            (*(*cur).m_Prev).m_Next = pBlock;
            (*cur).m_Prev = pBlock;

            // Update the jump table if necessary
            let index = Z_GetJumpTableIndex((*pBlock).m_Address) as usize;
            if s_FreeJumpTable[index].is_null() {
                s_FreeJumpTable[index] = pBlock;
            }

            s_Stats.m_CountFree += 1;
            s_Stats.m_SizeFree += (*pBlock).m_Size;

            assert!(Z_ValidateFree());
            break;
        }
        cur = (*cur).m_Next;
    }
}

unsafe fn Z_SplitFree(pBlock: *mut ZoneFreeBlock, iSize: c_int, bLow: bool) -> *mut c_void {
    assert!((*pBlock).m_Size >= iSize);

    Z_RemoveFromJumpTable(pBlock);

    // Delink the free block
    let fblock = *pBlock;
    (*(*pBlock).m_Prev).m_Next = (*pBlock).m_Next;
    (*(*pBlock).m_Next).m_Prev = (*pBlock).m_Prev;
    (*pBlock).m_Address = 0;

    s_Stats.m_CountFree -= 1;
    s_Stats.m_SizeFree -= fblock.m_Size;
    assert!(Z_ValidateFree());

    if fblock.m_Size > iSize {
        // Split the block into an allocated and free portion
        let remainder = fblock.m_Size - iSize;

        if remainder < core::mem::size_of::<ZoneFreeBlock>() as c_int {
            // Free portion is not large to hold free info --
            // we're going to have to use the overflow buffer.
            let nblock = Z_GetOverflowBlock();

            if nblock.is_null() {
                Z_Details_f();
                Com_Error(
                    ERR_FATAL,
                    b"Zone free overflow buffer overflowed!\0".as_ptr() as *const c_char,
                );
            }

            // Split the block
            let ret = if bLow {
                fblock.m_Address as *mut c_void
            } else {
                (fblock.m_Address + remainder) as *mut c_void
            };

            if bLow {
                (*nblock).m_Address = fblock.m_Address + iSize;
            } else {
                (*nblock).m_Address = fblock.m_Address;
            }

            (*nblock).m_Size = remainder;
            Z_LinkFreeBlock(nblock);

            return ret;
        } else {
            // Free portion is large enough -- split it
            let ret = if bLow {
                fblock.m_Address as *mut c_void
            } else {
                (fblock.m_Address + remainder) as *mut c_void
            };

            let nblock = if bLow {
                (fblock.m_Address + iSize) as *mut ZoneFreeBlock
            } else {
                fblock.m_Address as *mut ZoneFreeBlock
            };

            (*nblock).m_Address = nblock as c_int;
            (*nblock).m_Size = remainder;

            Z_LinkFreeBlock(nblock);

            return ret;
        }
    } else {
        // No need to split, just return block.
        fblock.m_Address as *mut c_void
    }
}

unsafe fn Z_SetupAlignmentPad(pBlock: *mut c_void, iAlignPad: c_int, bLow: bool) {
    // Clear alignment bytes
    memset(pBlock, 0, iAlignPad as usize);

    // If we have more than 1 alignment byte, the first align byte
    // tells us how many additional bytes we have.
    if iAlignPad > 1 {
        assert!((iAlignPad as c_int) < 256);
        let ptr = if bLow {
            (pBlock as *mut c_char).add((iAlignPad - 1) as usize)
        } else {
            pBlock as *mut c_char
        };
        *ptr = (iAlignPad - 1) as c_char;
    }
}

pub unsafe fn Z_MallocFail(pMessage: *const c_char, iSize: c_int, eTag: memtag_t) {
    // Report the error
    // Com_Printf("Z_Malloc(): %s : %d bytes and tag %d !!!!\n", pMessage, iSize, eTag);
    Com_Printf(
        b"Z_Malloc(): %s : %d bytes and tag %d !!!!\n\0".as_ptr() as *const c_char,
        pMessage,
        iSize,
        eTag,
    );
    Z_Details_f();
    Z_DumpMemMap_f();
    // Com_Printf("(Repeat): Z_Malloc(): %s : %d bytes and tag %d !!!!\n", pMessage, iSize, eTag);
    Com_Printf(
        b"(Repeat): Z_Malloc(): %s : %d bytes and tag %d !!!!\n\0".as_ptr() as *const c_char,
        pMessage,
        iSize,
        eTag,
    );

    // Clear the screen blue to indicate out of memory
    loop {
        qglBeginFrame();
        qglClearColor(0.0, 0.0, 1.0, 1.0);
        qglClear(GL_COLOR_BUFFER_BIT);
        qglEndFrame();
    }
}

pub unsafe fn Z_Malloc(iSize: c_int, mut eTag: memtag_t, bZeroit: qboolean, iAlign: c_int) -> *mut c_void {
    // assert(s_Initialized);
    // Zone now initializes on first use. (During static constructors)
    if !s_Initialized {
        Com_InitZoneMemory();
    }

    if iSize == 0 {
        return {
            #[cfg(debug_assertions)]
            {
                addr_of!(s_EmptyBlock.start).add(1) as *mut c_void
            }
            #[cfg(not(debug_assertions))]
            {
                addr_of!(s_EmptyBlock.header).add(1) as *mut c_void
            }
        };
    }

    if iSize < 0 {
        Z_MallocFail(
            b"Negative size\0".as_ptr() as *const c_char,
            iSize,
            eTag,
        );
        return null_mut();
    }

    #[cfg(not(target_os = "gamecube"))]
    {
        WaitForSingleObject(s_Mutex, INFINITE);
    }

    // Make new/delete memory temporary if requested
    if eTag == TAG_NEWDEL && s_IsNewDeleteTemp {
        eTag = TAG_TEMP_WORKSPACE;
    }

    // Determine how much space we need with headers and footers
    let mut header_size = core::mem::size_of::<ZoneHeader>() as c_int;
    let mut footer_size = 0;
    if Z_IsTagLinked(eTag) {
        header_size += core::mem::size_of::<ZoneLinkHeader>() as c_int;
    }
    #[cfg(debug_assertions)]
    {
        header_size += core::mem::size_of::<ZoneDebugHeader>() as c_int;
        footer_size += core::mem::size_of::<ZoneDebugFooter>() as c_int;
    }
    let real_size = iSize + header_size + footer_size;
    let mut align_pad = 0;

    // Get a bit of free memory.  Temporary memory is allocated
    // from the end.  More permanent allocations are done at the
    // begining of the pool.
    let fblock = if Z_IsTagTemp(eTag) {
        Z_FindLastFree(real_size, header_size, footer_size, iAlign, &mut align_pad)
    } else {
        Z_FindFirstFree(real_size, header_size, footer_size, iAlign, &mut align_pad)
    };

    // Did we actually find some memory?
    if fblock.is_null() {
        #[cfg(not(target_os = "gamecube"))]
        {
            ReleaseMutex(s_Mutex);
        }
        // if(eTag == TAG_TEMP_SND_RAWDATA) {
        if eTag == TAG_SND_RAWDATA {
            return null_mut();
        }

        Z_MallocFail(
            b"Out of memory\0".as_ptr() as *const c_char,
            iSize,
            eTag,
        );
        return null_mut();
    }

    // Add any alignment bytes
    let real_size = real_size + align_pad;

    // Split the free block and get a pointer to the start
    // allocated space.
    let mut ablock = if Z_IsTagTemp(eTag) {
        let ablock = Z_SplitFree(fblock, real_size, false);

        // Append align pad to end of block
        Z_SetupAlignmentPad(
            (ablock as *mut c_char).add((real_size - align_pad) as usize) as *mut c_void,
            align_pad,
            false,
        );
        ablock
    } else {
        let ablock = Z_SplitFree(fblock, real_size, true);

        // Insert align pad at block start
        Z_SetupAlignmentPad(ablock, align_pad, true);
        (ablock as *mut c_char).add(align_pad as usize) as *mut c_void
    };

    if ablock.is_null() {
        Z_MallocFail(
            b"Failed to split\0".as_ptr() as *const c_char,
            iSize,
            eTag,
        );
    }

    // Add linking header if necessary
    if Z_IsTagLinked(eTag) {
        let linked = ablock as *mut ZoneLinkHeader;
        (*linked).m_Next = s_LinkBase;
        (*linked).m_Prev = null_mut();
        if !s_LinkBase.is_null() {
            (*s_LinkBase).m_Prev = linked;
        }
        s_LinkBase = linked;

        assert!(Z_ValidateLinks());

        // Next...
        ablock = (ablock as *mut c_char).add(core::mem::size_of::<ZoneLinkHeader>()) as *mut c_void;
    }

    // Setup the header:
    //		31		- alignment flag
    //		25-30	- tag
    //		0-24	- size without headers/footers
    assert!(iSize >= 0 && iSize < (1 << 25));
    assert!(eTag >= 0 && eTag < 64);
    let header = ablock as *mut ZoneHeader;
    *header = ((eTag as c_int) << 25) | (iSize as c_int);

    if align_pad != 0 {
        *header |= 1 << 31;
    }

    // Next...
    ablock = (ablock as *mut c_char).add(core::mem::size_of::<ZoneHeader>()) as *mut c_void;

    #[cfg(debug_assertions)]
    {
        // Setup the debug markers
        let debug_header = ablock as *mut ZoneDebugHeader;

        let debug_footer = (debug_header as *mut c_char)
            .add((core::mem::size_of::<ZoneDebugHeader>() + iSize as usize) as usize)
            as *mut ZoneDebugFooter;

        *debug_header = ZONE_MAGIC;
        *debug_footer = ZONE_MAGIC;

        // Next...
        ablock = (ablock as *mut c_char).add(core::mem::size_of::<ZoneDebugHeader>()) as *mut c_void;
    }

    // Update the stats
    s_Stats.m_SizeAlloc += iSize;
    s_Stats.m_OverheadAlloc += header_size + footer_size + align_pad;
    s_Stats.m_SizesPerTag[eTag as usize] += iSize;
    s_Stats.m_CountAlloc += 1;
    s_Stats.m_CountsPerTag[eTag as usize] += 1;

    if s_Stats.m_SizeAlloc + s_Stats.m_OverheadAlloc > s_Stats.m_PeakAlloc {
        s_Stats.m_PeakAlloc = s_Stats.m_SizeAlloc + s_Stats.m_OverheadAlloc;
    }

    // Return a pointer to data memory
    if bZeroit != 0 {
        memset(ablock, 0, iSize as usize);
    }

    assert!(iAlign == 0 || (ablock as c_int) % iAlign == 0);

    /*
    This is useful for figuring out who's allocating a certain block of
    memory.  Please don't remove it.
    if(eTag == TAG_NEWDEL && (unsigned int)ablock >= 0x806c0000 &&
            (unsigned int)ablock <= 0x806c1000 && iSize == 24) {
        int suck = 0;
    }
    if(eTag == TAG_SMALL && (iSize == 7 || iSize == 96)) {
        int suck = 0;
    }
    if(eTag == TAG_CLIENTS) {
        int suck = 0;
    }
    */

    #[cfg(not(target_os = "gamecube"))]
    {
        ReleaseMutex(s_Mutex);
    }

    ablock
}

static fn Z_GetTag(header: *const ZoneHeader) -> memtag_t {
    unsafe { (*header & 0x7E000000) >> 25 }
}

static fn Z_GetSize(header: *const ZoneHeader) -> c_int {
    unsafe { *header & 0x1FFFFFF }
}

unsafe fn Z_GetAlign(header: *const ZoneHeader) -> c_int {
    if *header & (1 << 31) != 0 {
        let mut ptr = header as *mut c_char;
        let tag = Z_GetTag(header);

        // point to the first alignment block
        if Z_IsTagTemp(tag) {
            ptr = ptr.add(core::mem::size_of::<ZoneHeader>() + Z_GetSize(header) as usize);
            #[cfg(debug_assertions)]
            {
                ptr = ptr.add(core::mem::size_of::<ZoneDebugHeader>() + core::mem::size_of::<ZoneDebugFooter>());
            }
        } else {
            if Z_IsTagLinked(tag) {
                // skip the link header
                ptr = ptr.sub(core::mem::size_of::<ZoneLinkHeader>());
            }
            ptr = ptr.sub(1);
        }

        (*ptr as c_int) + 1
    } else {
        0
    }
}

pub unsafe fn Z_Size(pvAddress: *mut c_void) -> c_int {
    assert!(s_Initialized);

    #[cfg(debug_assertions)]
    {
        let debug = (pvAddress as *mut ZoneDebugHeader).sub(1);

        if *debug != ZONE_MAGIC {
            Com_Error(
                ERR_FATAL,
                b"Z_Size(): Not a valid zone header!\0".as_ptr() as *const c_char,
            );
            return 0; // won't get here
        }

        let pvAddress = debug as *mut c_void;
    }

    #[cfg(not(debug_assertions))]
    let pvAddress = pvAddress;

    let header = (pvAddress as *mut ZoneHeader).sub(1);

    if Z_GetTag(header) == TAG_STATIC {
        return 0; // kind of
    }

    Z_GetSize(header)
}

unsafe fn Z_Coalasce(pBlock: *mut ZoneFreeBlock) {
    let mut size = 0;

    // Find later free blocks adjacent to us
    let mut end = (*pBlock).m_Next;
    loop {
        if (*end).m_Next.is_null() {
            break;
        }
        if (*end).m_Address
            != (*(*end).m_Prev).m_Address + (*(*end).m_Prev).m_Size
        {
            break;
        }

        size += (*end).m_Size;

        Z_RemoveFromJumpTable(end);

        (*end).m_Address = 0; // invalidate block
        s_Stats.m_CountFree -= 1;

        end = (*end).m_Next;
    }

    // Find previous free blocks adjacent to us
    let mut start = pBlock;
    loop {
        if (*start).m_Prev.is_null() {
            break;
        }
        if (*(*start).m_Prev).m_Address + (*(*start).m_Prev).m_Size
            != (*start).m_Address
        {
            break;
        }

        size += (*start).m_Size;

        Z_RemoveFromJumpTable(start);

        (*start).m_Address = 0; // invalidate block
        s_Stats.m_CountFree -= 1;

        start = (*start).m_Prev;
    }

    // Do we need to coalesce some blocks?
    if (*start).m_Next != end {
        (*start).m_Next = end;
        (*end).m_Prev = start;
        (*start).m_Size += size;
    }
}

// Return type of Z_Free differs in SP/MP. Macro hack to wrap it up
// #ifdef _JK2MP
// 		void Z_Free(void *pvAddress)
// 		#define Z_FREE_RETURN(x) return
// #else
// 		int Z_Free(void *pvAddress)
// 		#define Z_FREE_RETURN(x) return (x)
// #endif

#[cfg(feature = "jk2mp")]
pub unsafe fn Z_Free(pvAddress: *mut c_void) {
    #[cfg(target_os = "windows")]
    {
        if !s_Initialized {
            return;
        }
    }

    assert!(s_Initialized);

    #[cfg(debug_assertions)]
    {
        // check the header magic
        let debug_header = (pvAddress as *mut ZoneDebugHeader).sub(1);

        if *debug_header != ZONE_MAGIC {
            Com_Error(
                ERR_FATAL,
                b"Z_Free(): Corrupt zone header!\0".as_ptr() as *const c_char,
            );
            return;
        }

        let header = (debug_header as *mut ZoneHeader).sub(1);

        // check the footer magic
        let debug_footer = (pvAddress as *mut c_char)
            .add(Z_GetSize(header) as usize)
            as *mut ZoneDebugFooter;

        if *debug_footer != ZONE_MAGIC {
            Com_Error(
                ERR_FATAL,
                b"Z_Free(): Corrupt zone footer!\0".as_ptr() as *const c_char,
            );
            return;
        }
    }

    #[cfg(not(debug_assertions))]
    let header = (pvAddress as *mut ZoneHeader).sub(1);
    #[cfg(debug_assertions)]
    let header = (pvAddress as *mut ZoneHeader).sub(1);

    let tag = Z_GetTag(header);

    if tag != TAG_STATIC {
        #[cfg(not(target_os = "gamecube"))]
        {
            WaitForSingleObject(s_Mutex, INFINITE);
        }

        // Determine size of header and footer
        let mut header_size = core::mem::size_of::<ZoneHeader>() as c_int;
        let align_size = Z_GetAlign(header);
        let mut footer_size = 0;
        let data_size = Z_GetSize(header);
        if Z_IsTagLinked(tag) {
            header_size += core::mem::size_of::<ZoneLinkHeader>() as c_int;
        }
        if Z_IsTagTemp(tag) {
            footer_size += align_size;
        } else {
            header_size += align_size;
        }
        #[cfg(debug_assertions)]
        {
            header_size += core::mem::size_of::<ZoneDebugHeader>() as c_int;
            footer_size += core::mem::size_of::<ZoneDebugFooter>() as c_int;
        }
        let real_size = data_size + header_size + footer_size;

        // Update the stats
        s_Stats.m_SizeAlloc -= data_size;
        s_Stats.m_OverheadAlloc -= header_size + footer_size;
        s_Stats.m_SizesPerTag[tag as usize] -= data_size;
        s_Stats.m_CountAlloc -= 1;
        s_Stats.m_CountsPerTag[tag as usize] -= 1;

        // Delink block
        if Z_IsTagLinked(tag) {
            let linked = (header as *mut ZoneLinkHeader).sub(1);

            if linked == s_LinkBase {
                s_LinkBase = (*linked).m_Next;
                if !s_LinkBase.is_null() {
                    (*s_LinkBase).m_Prev = null_mut();
                }
            } else {
                if !(*linked).m_Next.is_null() {
                    (*(*linked).m_Next).m_Prev = (*linked).m_Prev;
                }
                (*(*linked).m_Prev).m_Next = (*linked).m_Next;
            }

            assert!(Z_ValidateLinks());
        }

        // Clear the block header for safety
        *header = 0;

        // Add block to free list
        let nblock = if real_size < core::mem::size_of::<ZoneFreeBlock>() as c_int {
            // Not enough space in block to put free information --
            // use overflow buffer.
            let nblock = Z_GetOverflowBlock();

            if nblock.is_null() {
                Z_Details_f();
                Com_Error(
                    ERR_FATAL,
                    b"Zone free overflow buffer overflowed!\0".as_ptr() as *const c_char,
                );
            }

            nblock
        } else {
            // Place free information in block
            (pvAddress as *mut c_char).sub(header_size as usize) as *mut ZoneFreeBlock
        };

        (*nblock).m_Address = pvAddress as c_int - header_size;
        (*nblock).m_Size = real_size;
        Z_LinkFreeBlock(nblock);

        // Coalesce any adjacent free blocks
        Z_Coalasce(nblock);
        #[cfg(not(target_os = "gamecube"))]
        {
            ReleaseMutex(s_Mutex);
        }
    }
}

#[cfg(not(feature = "jk2mp"))]
pub unsafe fn Z_Free(pvAddress: *mut c_void) -> c_int {
    #[cfg(target_os = "windows")]
    {
        if !s_Initialized {
            return 0;
        }
    }

    assert!(s_Initialized);

    #[cfg(debug_assertions)]
    {
        // check the header magic
        let debug_header = (pvAddress as *mut ZoneDebugHeader).sub(1);

        if *debug_header != ZONE_MAGIC {
            Com_Error(
                ERR_FATAL,
                b"Z_Free(): Corrupt zone header!\0".as_ptr() as *const c_char,
            );
            return 0;
        }

        let header = (debug_header as *mut ZoneHeader).sub(1);

        // check the footer magic
        let debug_footer = (pvAddress as *mut c_char)
            .add(Z_GetSize(header) as usize)
            as *mut ZoneDebugFooter;

        if *debug_footer != ZONE_MAGIC {
            Com_Error(
                ERR_FATAL,
                b"Z_Free(): Corrupt zone footer!\0".as_ptr() as *const c_char,
            );
            return 0;
        }
    }

    #[cfg(not(debug_assertions))]
    let header = (pvAddress as *mut ZoneHeader).sub(1);
    #[cfg(debug_assertions)]
    let header = (pvAddress as *mut ZoneHeader).sub(1);

    let tag = Z_GetTag(header);

    if tag != TAG_STATIC {
        #[cfg(not(target_os = "gamecube"))]
        {
            WaitForSingleObject(s_Mutex, INFINITE);
        }

        // Determine size of header and footer
        let mut header_size = core::mem::size_of::<ZoneHeader>() as c_int;
        let align_size = Z_GetAlign(header);
        let mut footer_size = 0;
        let data_size = Z_GetSize(header);
        if Z_IsTagLinked(tag) {
            header_size += core::mem::size_of::<ZoneLinkHeader>() as c_int;
        }
        if Z_IsTagTemp(tag) {
            footer_size += align_size;
        } else {
            header_size += align_size;
        }
        #[cfg(debug_assertions)]
        {
            header_size += core::mem::size_of::<ZoneDebugHeader>() as c_int;
            footer_size += core::mem::size_of::<ZoneDebugFooter>() as c_int;
        }
        let real_size = data_size + header_size + footer_size;

        // Update the stats
        s_Stats.m_SizeAlloc -= data_size;
        s_Stats.m_OverheadAlloc -= header_size + footer_size;
        s_Stats.m_SizesPerTag[tag as usize] -= data_size;
        s_Stats.m_CountAlloc -= 1;
        s_Stats.m_CountsPerTag[tag as usize] -= 1;

        // Delink block
        if Z_IsTagLinked(tag) {
            let linked = (header as *mut ZoneLinkHeader).sub(1);

            if linked == s_LinkBase {
                s_LinkBase = (*linked).m_Next;
                if !s_LinkBase.is_null() {
                    (*s_LinkBase).m_Prev = null_mut();
                }
            } else {
                if !(*linked).m_Next.is_null() {
                    (*(*linked).m_Next).m_Prev = (*linked).m_Prev;
                }
                (*(*linked).m_Prev).m_Next = (*linked).m_Next;
            }

            assert!(Z_ValidateLinks());
        }

        // Clear the block header for safety
        *header = 0;

        // Add block to free list
        let nblock = if real_size < core::mem::size_of::<ZoneFreeBlock>() as c_int {
            // Not enough space in block to put free information --
            // use overflow buffer.
            let nblock = Z_GetOverflowBlock();

            if nblock.is_null() {
                Z_Details_f();
                Com_Error(
                    ERR_FATAL,
                    b"Zone free overflow buffer overflowed!\0".as_ptr() as *const c_char,
                );
            }

            nblock
        } else {
            // Place free information in block
            (pvAddress as *mut c_char).sub(header_size as usize) as *mut ZoneFreeBlock
        };

        (*nblock).m_Address = pvAddress as c_int - header_size;
        (*nblock).m_Size = real_size;
        Z_LinkFreeBlock(nblock);

        // Coalesce any adjacent free blocks
        Z_Coalasce(nblock);
        #[cfg(not(target_os = "gamecube"))]
        {
            ReleaseMutex(s_Mutex);
        }
    }

    0
}

pub fn Z_MemSize(eTag: memtag_t) -> c_int {
    unsafe { s_Stats.m_SizesPerTag[eTag as usize] }
}

#[cfg(all(debug_assertions, ZONE_DEBUG))]
pub unsafe fn Z_FindLeak() {
    assert!(s_Initialized);

    static mut cycle_count: c_int = 0;
    const tag: memtag_t = TAG_NEWDEL;

    struct PointerInfo {
        data: *mut c_void,
        counter: c_int,
        mark: bool,
    }

    const max_pointers: c_int = 32768;
    static mut pointers: [PointerInfo; 32768] = [PointerInfo {
        data: null_mut(),
        counter: 0,
        mark: false,
    }; 32768];
    static mut num_pointers: c_int = 0;

    // Clear pointer existance
    for i in 0..num_pointers as usize {
        pointers[i].mark = false;
    }

    // Add all known pointers
    let start_num = num_pointers;
    let mut link = s_LinkBase;
    loop {
        if link.is_null() {
            break;
        }
        let header = (link as *mut ZoneHeader).add(1);
        link = (*link).m_Next;

        if Z_GetTag(header) == tag {
            // See if the pointer already is in the array
            let mut found = false;
            for k in start_num as usize..num_pointers as usize {
                if pointers[k].data == header as *mut c_void {
                    pointers[k].counter += 1;
                    pointers[k].mark = true;
                    found = true;
                    break;
                }
            }

            // If the pointer is not in the array, add it
            if !found {
                assert!((num_pointers as c_int) < max_pointers);
                pointers[num_pointers as usize].data = header as *mut c_void;
                pointers[num_pointers as usize].counter = 0;
                pointers[num_pointers as usize].mark = true;
                num_pointers += 1;
            }
        }
    }

    // Remove pointers that are no longer used
    let mut j = 0;
    while j < num_pointers as usize {
        if pointers[j].mark {
            if pointers[j].counter != cycle_count
                && pointers[j].counter != cycle_count - 1
                && pointers[j].counter != 0
            {
                Com_Printf(
                    b"Memory leak: %p\n\0".as_ptr() as *const c_char,
                    pointers[j].data,
                );
            }
        } else {
            let mut k = j;
            while k < num_pointers as usize {
                if pointers[k].mark {
                    break;
                }
                k += 1;
            }

            if k == num_pointers as usize {
                break;
            }

            memmove(
                addr_of_mut!(pointers[j]) as *mut c_void,
                addr_of!(pointers[k]) as *const c_void,
                ((num_pointers as usize) - k) * core::mem::size_of::<PointerInfo>(),
            );
            num_pointers -= (k - j) as c_int;
        }
        j += 1;
    }

    cycle_count += 1;
}

pub unsafe fn Z_TagPointers(eTag: memtag_t) {
    assert!(s_Initialized);

    #[cfg(not(target_os = "gamecube"))]
    {
        WaitForSingleObject(s_Mutex, INFINITE);
    }

    Com_Printf(b"Pointers for tag %d:\n\0".as_ptr() as *const c_char, eTag);

    let mut link = s_LinkBase;
    loop {
        if link.is_null() {
            break;
        }
        let header = (link as *mut ZoneHeader).add(1);
        link = (*link).m_Next;

        if eTag == TAG_ALL || Z_GetTag(header) == eTag {
            #[cfg(debug_assertions)]
            {
                Com_Printf(
                    b"%x - %d\n\0".as_ptr() as *const c_char,
                    (header as *mut c_char)
                        .add(core::mem::size_of::<ZoneHeader>() + core::mem::size_of::<ZoneDebugHeader>())
                        as *mut c_void,
                    Z_Size(
                        (header as *mut c_char)
                            .add(core::mem::size_of::<ZoneHeader>() + core::mem::size_of::<ZoneDebugHeader>())
                            as *mut c_void,
                    ),
                );
            }
            #[cfg(not(debug_assertions))]
            {
                Com_Printf(
                    b"%x - %d\n\0".as_ptr() as *const c_char,
                    (header as *mut ZoneHeader).add(1) as *mut c_void,
                    Z_Size((header as *mut ZoneHeader).add(1) as *mut c_void),
                );
            }
        }
    }

    #[cfg(not(target_os = "gamecube"))]
    {
        ReleaseMutex(s_Mutex);
    }
}

pub unsafe fn Z_TagFree(eTag: memtag_t) {
    assert!(s_Initialized);

    let mut link = s_LinkBase;
    loop {
        if link.is_null() {
            break;
        }
        let header = (link as *mut ZoneHeader).add(1);
        link = (*link).m_Next;

        if eTag == TAG_ALL || Z_GetTag(header) == eTag {
            #[cfg(debug_assertions)]
            {
                Z_Free(
                    (header as *mut c_char)
                        .add(core::mem::size_of::<ZoneHeader>() + core::mem::size_of::<ZoneDebugHeader>())
                        as *mut c_void,
                );
            }
            #[cfg(not(debug_assertions))]
            {
                Z_Free((header as *mut ZoneHeader).add(1) as *mut c_void);
            }
        }
    }
}

pub unsafe fn Z_SetNewDeleteTemporary(bTemp: bool) {
    // Catch nested uses that break when unwinding the stack
    assert!(bTemp != s_IsNewDeleteTemp);
    s_IsNewDeleteTemp = bTemp;
}

pub unsafe fn S_Malloc(iSize: c_int) -> *mut c_void {
    Z_Malloc(iSize, TAG_SMALL, qfalse, 0)
}

pub fn Z_GetLevelMemory() -> c_int {
    unsafe {
        #[cfg(feature = "jk2mp")]
        {
            s_Stats.m_SizesPerTag[TAG_BSP as usize]
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            s_Stats.m_SizesPerTag[TAG_HUNKALLOC as usize]
                + s_Stats.m_SizesPerTag[TAG_HUNKMISCMODELS as usize]
                + s_Stats.m_SizesPerTag[TAG_BSP as usize]
        }
    }
}

#[cfg(feature = "jk2mp")]
pub fn Z_GetHunkMemory() -> c_int {
    unsafe {
        s_Stats.m_SizesPerTag[TAG_HUNK_MARK1 as usize]
            + s_Stats.m_SizesPerTag[TAG_HUNK_MARK2 as usize]
            + s_Stats.m_SizesPerTag[TAG_TEMP_HUNKALLOC as usize]
    }
}

pub fn Z_GetTerrainMemory() -> c_int {
    unsafe {
        s_Stats.m_SizesPerTag[TAG_CM_TERRAIN as usize]
            + s_Stats.m_SizesPerTag[TAG_CM_TERRAIN_TEMP as usize]
            + {
                #[cfg(feature = "jk2mp")]
                {
                    s_Stats.m_SizesPerTag[TAG_TERRAIN as usize]
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    0
                }
            }
            + s_Stats.m_SizesPerTag[TAG_R_TERRAIN as usize]
    }
}

pub fn Z_GetMiscMemory() -> c_int {
    unsafe {
        s_Stats.m_SizeAlloc
            - (Z_GetLevelMemory()
                + {
                    #[cfg(feature = "jk2mp")]
                    {
                        Z_GetHunkMemory()
                    }
                    #[cfg(not(feature = "jk2mp"))]
                    {
                        0
                    }
                }
                + Z_GetTerrainMemory()
                + s_Stats.m_SizesPerTag[TAG_MODEL_GLM as usize]
                + s_Stats.m_SizesPerTag[TAG_MODEL_GLA as usize]
                + s_Stats.m_SizesPerTag[TAG_MODEL_MD3 as usize]
                + s_Stats.m_SizesPerTag[TAG_BINK as usize]
                + s_Stats.m_SizesPerTag[TAG_SND_RAWDATA as usize])
    }
}

#[cfg(target_os = "gamecube")]
static mut texMemSize: c_int = 0;
#[cfg(not(target_os = "gamecube"))]
extern "C" {
    static mut texMemSize: c_int;
}

pub unsafe fn Z_CompactStats() {
    assert!(s_Initialized);

    // This report is conservative.  Divides by 1000 instead of 1024 and
    // then rounds up.
    Sys_Log(
        b"memory-map.txt\0".as_ptr() as *const c_char,
        va(b"**Z_CompactStats Start**\n\0".as_ptr() as *const c_char),
        0,
        false,
    );
    Sys_Log(
        b"memory-map.txt\0".as_ptr() as *const c_char,
        va(
            b"map: %s\n\0".as_ptr() as *const c_char,
            Cvar_VariableString(b"mapname\0".as_ptr() as *const c_char),
        ),
        0,
        false,
    );

    Sys_Log(
        b"memory-map.txt\0".as_ptr() as *const c_char,
        va(
            b"OV: %d, LVL: %d, GLM: %d, GLA: %d, MD3: %d\n\0".as_ptr() as *const c_char,
            (s_Stats.m_OverheadAlloc / 1000) + 1,
            (Z_GetLevelMemory() / 1000) + 1,
            (s_Stats.m_SizesPerTag[TAG_MODEL_GLM as usize] / 1000) + 1,
            (s_Stats.m_SizesPerTag[TAG_MODEL_GLA as usize] / 1000) + 1,
            (s_Stats.m_SizesPerTag[TAG_MODEL_MD3 as usize] / 1000) + 1,
        ),
        0,
        false,
    );

    Sys_Log(
        b"memory-map.txt\0".as_ptr() as *const c_char,
        va(
            b"TER: %d, SND: %d, TEX: %d, FMV: %d, MSC: %d\n\0".as_ptr() as *const c_char,
            (Z_GetTerrainMemory() / 1000) + 1,
            (s_Stats.m_SizesPerTag[TAG_SND_RAWDATA as usize] / 1000) + 1,
            (texMemSize / 1000) + 1,
            (s_Stats.m_SizesPerTag[TAG_BINK as usize] / 1000) + 1,
            (Z_GetMiscMemory() / 1000) + 1,
        ),
        0,
        false,
    );

    #[cfg(feature = "jk2mp")]
    {
        Sys_Log(
            b"memory-map.txt\0".as_ptr() as *const c_char,
            va(
                b"HUNK: %d, THUNK: %d\n\0".as_ptr() as *const c_char,
                ((s_Stats.m_SizesPerTag[TAG_HUNK_MARK1 as usize]
                    + s_Stats.m_SizesPerTag[TAG_HUNK_MARK2 as usize])
                    / 1000)
                    + 1,
                (s_Stats.m_SizesPerTag[TAG_TEMP_HUNKALLOC as usize] / 1000) + 1,
            ),
            0,
            false,
        );
    }

    Sys_Log(
        b"memory-map.txt\0".as_ptr() as *const c_char,
        va(
            b"Free Zone: %d\n\0".as_ptr() as *const c_char,
            s_Stats.m_SizeFree,
        ),
        0,
        false,
    );
}

pub unsafe fn Z_Stats_f() {
    assert!(s_Initialized);
    // Display some memory usage summary information...

    Com_Printf(
        b"\nThe zone is using %d bytes (%.2fMB) in %d memory blocks\n\0".as_ptr()
            as *const c_char,
        s_Stats.m_SizeAlloc,
        s_Stats.m_SizeAlloc as f32 / 1024.0 / 1024.0,
        s_Stats.m_CountAlloc,
    );

    Com_Printf(
        b"Free memory is %d bytes (%.2fMB) in %d memory blocks\n\0".as_ptr()
            as *const c_char,
        s_Stats.m_SizeFree,
        s_Stats.m_SizeFree as f32 / 1024.0 / 1024.0,
        s_Stats.m_CountFree,
    );

    Com_Printf(
        b"The zone peaked at %d bytes (%.2fMB)\n\0".as_ptr() as *const c_char,
        s_Stats.m_PeakAlloc,
        s_Stats.m_PeakAlloc as f32 / 1024.0 / 1024.0,
    );

    Com_Printf(
        b"The zone overhead is %d bytes (%.2fMB)\n\0".as_ptr() as *const c_char,
        s_Stats.m_OverheadAlloc,
        s_Stats.m_OverheadAlloc as f32 / 1024.0 / 1024.0,
    );
}

pub unsafe fn Z_Details_f() {
    assert!(s_Initialized);
    // Display some tag specific information...

    Com_Printf(
        b"---------------------------------------------------------------------------\n\0"
            .as_ptr() as *const c_char,
    );
    Com_Printf(
        b"%20s %9s\n\0".as_ptr() as *const c_char,
        b"Zone Tag\0".as_ptr(),
        b"Bytes\0".as_ptr(),
    );
    Com_Printf(
        b"%20s %9s\n\0".as_ptr() as *const c_char,
        b"--------\0".as_ptr(),
        b"-----\0".as_ptr(),
    );
    for i in 0..TAG_COUNT as usize {
        let iThisCount = s_Stats.m_CountsPerTag[i];
        let iThisSize = s_Stats.m_SizesPerTag[i];

        if iThisCount != 0 {
            let fSize = iThisSize as f32 / 1024.0 / 1024.0;
            let iSize = fSize as c_int;
            let iRemainder = ((fSize - fSize.floor()) * 100.0) as c_int;
            Com_Printf(
                b"%d %9d (%2d.%02dMB) in %6d blocks (%9d average)\n\0".as_ptr()
                    as *const c_char,
                i,
                iThisSize,
                iSize,
                iRemainder,
                iThisCount,
                iThisSize / iThisCount,
            );
        }
    }
    Com_Printf(
        b"---------------------------------------------------------------------------\n\0"
            .as_ptr() as *const c_char,
    );

    Z_Stats_f();
}

pub unsafe fn Z_DumpMemMap_f() {
    macro_rules! WRITECHAR {
        ($C:expr) => {
            Sys_Log(
                b"memmap.txt\0".as_ptr() as *const c_char,
                $C as *const c_char,
                1,
                false,
            );
            cur += 1024;
            counter += 1;
            if (counter) % 81 == 0 {
                Sys_Log(
                    b"memmap.txt\0".as_ptr() as *const c_char,
                    b"\n\0".as_ptr() as *const c_char,
                    1,
                    false,
                );
            }
        };
    }

    let mut cur = s_PoolBase as c_int;
    let mut counter = 0;
    let mut fblock = addr_of_mut!(s_FreeStart);
    loop {
        if fblock == addr_of_mut!(s_FreeEnd) as *mut _ {
            break;
        }
        while (*fblock).m_Address > cur + 1024 {
            WRITECHAR!(b"*\0");
        }

        if (*fblock).m_Address > cur && (*fblock).m_Address < cur + 1024 {
            WRITECHAR!(b"+\0");
        }

        while (*fblock).m_Address + (*fblock).m_Size > cur + 1024 {
            WRITECHAR!(b"-\0");
        }

        if (*fblock).m_Address + (*fblock).m_Size > cur
            && (*fblock).m_Address + (*fblock).m_Size < cur + 1024
        {
            WRITECHAR!(b"+\0");
        }

        fblock = (*fblock).m_Next;
    }

    Sys_Log(
        b"memmap.txt\0".as_ptr() as *const c_char,
        b"\n\0".as_ptr() as *const c_char,
        1,
        false,
    );
}

pub unsafe fn Z_DisplayLevelMemory(size: c_int, surf: c_int, block: c_int) {
    Z_DumpMemMap_f();

    // Yes, it should be divided by 1024, but I'm going for a safety margin
    // by rounding down.
    // Com_Printf("level memory used: %d KB\n", size / 1000);
    // Z_CompactStats(size, surf, block);
    Z_CompactStats();
}

pub unsafe fn Z_DisplayLevelMemory_NoArgs() {
    #[cfg(target_os = "gamecube")]
    {
        extern "C" {
            fn R_SurfMramUsed(surface: *mut c_int, block: *mut c_int);
        }
        let mut surface: c_int = 0;
        let mut block: c_int = 0;
        R_SurfMramUsed(&mut surface, &mut block);
        Z_DisplayLevelMemory(Z_GetLevelMemory(), surface, block);
    }
    #[cfg(not(target_os = "gamecube"))]
    {
        Z_DisplayLevelMemory(Z_GetLevelMemory(), 0, 0);
    }
}

/*
========================
CopyString

 NOTE:	never write over the memory CopyString returns because
		memory from a memstatic_t might be returned
========================
*/
pub unsafe fn CopyString(in_str: *const c_char) -> *mut c_char {
    struct ZoneSingleChar {
        header: ZoneHeader,
        #[cfg(debug_assertions)]
        start: ZoneDebugHeader,
        data: [c_char; 2],
        #[cfg(debug_assertions)]
        end: ZoneDebugFooter,
    }

    #[cfg(debug_assertions)]
    static empty: ZoneSingleChar = ZoneSingleChar {
        header: (TAG_STATIC << 25) | 2,
        start: ZONE_MAGIC,
        data: [0, 0],
        end: ZONE_MAGIC,
    };
    #[cfg(debug_assertions)]
    static numbers: [ZoneSingleChar; 10] = [
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'0' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'1' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'2' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'3' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'4' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'5' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'6' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'7' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'8' as c_char, 0],
            end: ZONE_MAGIC,
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            start: ZONE_MAGIC,
            data: [b'9' as c_char, 0],
            end: ZONE_MAGIC,
        },
    ];

    #[cfg(not(debug_assertions))]
    static empty: ZoneSingleChar = ZoneSingleChar {
        header: (TAG_STATIC << 25) | 2,
        data: [0, 0],
    };
    #[cfg(not(debug_assertions))]
    static numbers: [ZoneSingleChar; 10] = [
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'0' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'1' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'2' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'3' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'4' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'5' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'6' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'7' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'8' as c_char, 0],
        },
        ZoneSingleChar {
            header: (TAG_STATIC << 25) | 2,
            data: [b'9' as c_char, 0],
        },
    ];

    if *in_str == 0 {
        return empty.data.as_ptr() as *mut c_char;
    } else if *(in_str.add(1)) == 0 {
        if *in_str >= b'0' as c_char && *in_str <= b'9' as c_char {
            return numbers[(*in_str - b'0' as c_char) as usize]
                .data
                .as_ptr() as *mut c_char;
        }
    }

    let out = S_Malloc((strlen(in_str) + 1) as c_int) as *mut c_char;
    strcpy(out, in_str);

    // Z_Label(out,in);

    out
}

pub fn Com_TouchMemory() {
    // Stub function. Do nothing.
}

pub unsafe fn Z_IsFromZone(pvAddress: *mut c_void, eTag: memtag_t) -> qboolean {
    assert!(s_Initialized);

    #[cfg(debug_assertions)]
    {
        let debug = (pvAddress as *mut ZoneDebugHeader).sub(1);

        if *debug != ZONE_MAGIC {
            return qfalse;
        }

        let pvAddress = debug as *mut c_void;
    }

    #[cfg(not(debug_assertions))]
    let pvAddress = pvAddress;

    let header = (pvAddress as *mut ZoneHeader).sub(1);

    if Z_GetTag(header) != eTag {
        return qfalse;
    }

    Z_GetSize(header)
}

/*
   Hunk emulation - PC switched to system similar to ours. I made the remaining
   changes so that the two are identical.
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Com_TheHunkMarkHasBeenMade() -> qboolean {
    if hunk_tag == TAG_HUNK_MARK2 {
        qtrue
    } else {
        qfalse
    }
}

/*
=================
Com_InitHunkMemory
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Com_InitHunkMemory() {
    hunk_tag = TAG_HUNK_MARK1;
    Hunk_Clear();
}

/*
====================
Hunk_MemoryRemaining
====================
*/
#[cfg(feature = "jk2mp")]
pub fn Hunk_MemoryRemaining() -> c_int {
    0
}

/*
===================
Hunk_SetMark

The server calls this after the level and game VM have been loaded
===================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_SetMark() {
    hunk_tag = TAG_HUNK_MARK2;
}

/*
=================
Hunk_ClearToMark

The client calls this before starting a vid_restart or snd_restart
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_ClearToMark() {
    assert!(hunk_tag == TAG_HUNK_MARK2); // if this is not true then no mark has been made
    Z_TagFree(TAG_HUNK_MARK2);
}

/*
=================
Hunk_CheckMark
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_CheckMark() -> qboolean {
    if hunk_tag != TAG_HUNK_MARK1 {
        qtrue
    } else {
        qfalse
    }
}

/*
=================
Hunk_Clear

The server calls this before shutting down or loading a new map
VVFIXME - PC version does lots of other things in here.
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_Clear() {
    hunk_tag = TAG_HUNK_MARK1;
    Z_TagFree(TAG_HUNK_MARK1);
    Z_TagFree(TAG_HUNK_MARK2);

    R_HunkClearCrap();
    /*
    Z_TagFree(TAG_HUNKALLOC);
    Z_TagFree(TAG_BSP_HUNK);
    Z_TagFree(TAG_BOT_HUNK);
    Z_TagFree(TAG_RENDERER_HUNK);
    Z_TagFree(TAG_SKELETON);
    Z_TagFree(TAG_MODEL_OTHER);
    Z_TagFree(TAG_MODEL_CHAR);
    VM_Clear();
    */
}

#[repr(C)]
pub enum ha_pref {
    preference = 0,
}

/*
=================
Hunk_Alloc

Allocate permanent (until the hunk is cleared) memory
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_Alloc(size: c_int, _preference: ha_pref) -> *mut c_void {
    Z_Malloc(size, hunk_tag, qtrue, 0)
}

/*
=================
Hunk_AllocateTempMemory

This is used by the file loading system.
Multiple files can be loaded in temporary memory.
When the files-in-use count reaches zero, all temp memory will be deleted
=================
*/
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_AllocateTempMemory(size: c_int) -> *mut c_void {
    // don't bother clearing, because we are going to load a file over it
    Z_Malloc(size, TAG_TEMP_HUNKALLOC, qfalse, 0)
}

/*
==================
Hunk_FreeTempMemory
==================
*/
#[cfg(feature = "jk2mp")]
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
#[cfg(feature = "jk2mp")]
pub unsafe fn Hunk_ClearTempMemory() {
    Z_TagFree(TAG_TEMP_HUNKALLOC);
}

/*
	XTL Replacement functions
	XMemAlloc
	XMemFree
	XMemSize

	Replacing these lets us intercept ALL memory allocation done by the XTL, and lets the
	Zone take pretty much all available memory at startup
*/
/* This still doesn't work. Numrous allocations still use internal functions, so there's
   little benefit right now.

XBOXAPI
LPVOID
WINAPI
XMemAlloc(SIZE_T dwSize, DWORD dwAllocAttributes)
{
	// We always give XTL 16 byte aligned memory
	return Z_Malloc(dwSize, TAG_XTL, ((PXALLOC_ATTRIBUTES)&dwAllocAttributes)->dwZeroInitialize, 16);
}

XBOXAPI
VOID
WINAPI
XMemFree(PVOID pAddress, DWORD dwAllocAttributes)
{
	Z_Free(pAddress);
}

XBOXAPI
SIZE_T
WINAPI
XMemSize(PVOID pAddress, DWORD dwAllocAttributes)
{
	return Z_Size(pAddress);
}

*/
