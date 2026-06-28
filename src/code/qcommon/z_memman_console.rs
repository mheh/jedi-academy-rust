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

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Used to mark the start and end of blocks in debug mode
const ZONE_MAGIC: u32 = 0xfe;

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

// Game needs about 8 MB for framebuffers audio, bink, etc., plus 17 MB (?)
// for textures. Leave lots more physical memory around when not in 64 MB
// map, so the profiler and other things work.
#[cfg(all(not(target_os = "gamecube"), feature = "final_build"))]
const ZONE_HEAP_FREE: usize = 1024 * 1024 * 8 + 17 * 1024 * 1024;

#[cfg(all(not(target_os = "gamecube"), not(feature = "final_build")))]
const ZONE_HEAP_FREE: usize = 1024 * 1024 * 16 + 16 * 1024 * 1024 + 16 * 1024 * 1024;

// Should we emulate the smaller memory footprint of actual release systems?
const ZONE_EMULATE_SPACE: c_int = 0;

// All standard header data is crammed into 4 bytes
type ZoneHeader = u32;

// Debug markers to check for overflow/underflow
type ZoneDebugHeader = u32;
type ZoneDebugFooter = u8;

// Extended header information for memory freed with TagFree()
#[repr(C)]
struct ZoneLinkHeader {
    m_Next: *mut ZoneLinkHeader,
    m_Prev: *mut ZoneLinkHeader,
}

static mut s_LinkBase: *mut ZoneLinkHeader = core::ptr::null_mut();

// Free memory block tracking information
#[repr(C)]
struct ZoneFreeBlock {
    m_Address: u32,
    m_Size: u32,
    m_Next: *mut ZoneFreeBlock,
    m_Prev: *mut ZoneFreeBlock,
}

// Buffer to hold free memory information that we can't
// fit directly in the pool
static mut s_FreeOverflow: [ZoneFreeBlock; ZONE_FREE_OVERFLOW] =
    [ZoneFreeBlock {
        m_Address: 0,
        m_Size: 0,
        m_Next: core::ptr::null_mut(),
        m_Prev: core::ptr::null_mut(),
    }; ZONE_FREE_OVERFLOW];
static mut s_LastOverflowIndex: c_int = 0;

static mut s_FreeStart: ZoneFreeBlock = ZoneFreeBlock {
    m_Address: 0,
    m_Size: 0,
    m_Next: core::ptr::null_mut(),
    m_Prev: core::ptr::null_mut(),
};

static mut s_FreeEnd: ZoneFreeBlock = ZoneFreeBlock {
    m_Address: 0,
    m_Size: 0,
    m_Next: core::ptr::null_mut(),
    m_Prev: core::ptr::null_mut(),
};

// Various stats collected at runtime
#[repr(C)]
#[allow(non_snake_case)]
struct ZoneStats {
    m_CountAlloc: c_int,
    m_SizeAlloc: c_int,
    m_OverheadAlloc: c_int,
    m_PeakAlloc: c_int,
    m_CountFree: c_int,
    m_SizeFree: c_int,
    m_SizesPerTag: [c_int; 64], // TAG_COUNT = 64
    m_CountsPerTag: [c_int; 64],
}

static mut s_Stats: ZoneStats = ZoneStats {
    m_CountAlloc: 0,
    m_SizeAlloc: 0,
    m_OverheadAlloc: 0,
    m_PeakAlloc: 0,
    m_CountFree: 0,
    m_SizeFree: 0,
    m_SizesPerTag: [0; 64],
    m_CountsPerTag: [0; 64],
};

// Special empty block for zero size allocations
#[repr(C)]
struct ZoneEmptyBlock {
    header: ZoneHeader,
    #[cfg(debug_assertions)]
    start: ZoneDebugHeader,
    #[cfg(debug_assertions)]
    end: ZoneDebugFooter,
}

#[cfg(debug_assertions)]
static s_EmptyBlock: ZoneEmptyBlock = ZoneEmptyBlock {
    header: 0 << 25, // TAG_STATIC << 25
    start: ZONE_MAGIC,
    end: ZONE_MAGIC as u8,
};

#[cfg(not(debug_assertions))]
static s_EmptyBlock: ZoneEmptyBlock = ZoneEmptyBlock {
    header: 0 << 25, // TAG_STATIC << 25
};

// Free block jump table for fast memory deallocation
const Z_JUMP_TABLE_SIZE: usize = 64;
static mut s_FreeJumpTable: [*mut ZoneFreeBlock; Z_JUMP_TABLE_SIZE] =
    [core::ptr::null_mut(); Z_JUMP_TABLE_SIZE];
static mut s_FreeJumpResolution: u32 = 0;

static mut s_PoolBase: *mut c_void = core::ptr::null_mut();
static mut s_Initialized: bool = false;
static mut s_IsNewDeleteTemp: bool = false;

#[cfg(not(target_os = "gamecube"))]
#[cfg(target_os = "windows")]
static mut s_Mutex: *mut core::ffi::c_void = core::ptr::null_mut();

extern "C" {
    fn Z_Stats_f();
    pub fn Z_Details_f();
    pub fn Z_DumpMemMap_f();
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn Cmd_RemoveCommand(name: *const c_char);
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Sys_Log(filename: *const c_char, msg: *const c_char, ...);
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
}

#[cfg(target_os = "windows")]
extern "C" {
    fn GlobalAlloc(flags: u32, size: usize) -> *mut c_void;
    fn GlobalFree(mem: *mut c_void) -> *mut c_void;
    fn CreateMutex(
        lpMutexAttributes: *mut c_void,
        bInitialOwner: c_int,
        lpName: *const c_char,
    ) -> *mut c_void;
    fn CloseHandle(hObject: *mut c_void) -> c_int;
    fn WaitForSingleObject(hHandle: *mut c_void, dwMilliseconds: u32) -> u32;
    fn ReleaseMutex(hMutex: *mut c_void) -> c_int;
    fn GlobalMemoryStatus(lpBuffer: *mut c_void);
}

#[cfg(target_os = "windows")]
extern "C" {
    fn qglBeginFrame();
    fn qglClearColor(r: f32, g: f32, b: f32, a: f32);
    fn qglClear(mask: u32);
    fn qglEndFrame();
}

const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const ERR_FATAL: c_int = 3;
const INVALID_HANDLE_VALUE: *mut c_void = core::ptr::null_mut();

// Tag type (opaque from this module's perspective)
type memtag_t = c_int;
type qboolean = c_int;
type ha_pref = c_int;

const TAG_STATIC: memtag_t = 0;

#[cfg(target_os = "xbox")]
#[repr(C)]
struct MEMORYSTATUS {
    dwLength: u32,
    dwMemoryLoad: u32,
    dwTotalPhys: u32,
    dwAvailPhys: u32,
    dwTotalPageFile: u32,
    dwAvailPageFile: u32,
    dwTotalVirtual: u32,
    dwAvailVirtual: u32,
}

#[cfg(target_os = "xbox")]
extern "C" {
    fn ShowOSMemory();
}

#[cfg(target_os = "xbox")]
#[allow(non_snake_case)]
fn ShowOSMemory() {
    // MEMORYSTATUS stat;
    // GlobalMemoryStatus(&stat);
    // Com_Printf("     total mem: %d, free mem: %d\n", stat.dwTotalPhys / 1024,
    //         stat.dwAvailPhys / 1024);
    // FILE *out = fopen("d:\\osmem.txt", "a");
    // if(out) {
    //     fprintf(out, "total mem: %d, free mem: %d\n", stat.dwTotalPhys / 1024,
    //             stat.dwAvailPhys / 1024);
    //     fclose(out);
    // }
}

pub fn Z_MemFree() -> c_int {
    unsafe { (*addr_of!(s_Stats)).m_SizeFree }
}

pub fn Com_InitZoneMemory() {
    unsafe {
        if s_Initialized {
            return;
        }

        Com_Printf(b"Initialising zone memory .....\n\0" as *const u8 as *const c_char);

        // Clear some globals
        memset(addr_of_mut!(s_Stats) as *mut c_void, 0, core::mem::size_of::<ZoneStats>());
        memset(addr_of_mut!(s_FreeOverflow) as *mut c_void, 0,
               core::mem::size_of::<[ZoneFreeBlock; ZONE_FREE_OVERFLOW]>());
        s_LastOverflowIndex = 0;
        s_LinkBase = core::ptr::null_mut();
        s_IsNewDeleteTemp = false;

        // Alloc the pool
        #[cfg(target_os = "xbox")]
        {
            // MEMORYSTATUS status;
            // GlobalMemoryStatus(&status);
            //
            // // BTO : VVFIXME - Extra little note to see how much memory
            // // is being used by globals/statics
            // Com_Printf("*** PhysRAM: %d used, %d free\n",
            //             status.dwTotalPhys-status.dwAvailPhys,
            //             status.dwAvailPhys);
            // SIZE_T size;
            // #	if ZONE_EMULATE_SPACE
            // #ifdef _DEBUG
            // //Emulated space is always about 6 megs off from release build.  Try
            // //to compensate.  This number may need tweaking in the future.
            // SIZE_T exe = 6500 * 1024;
            // #else
            // SIZE_T exe = 0; //Exe size is already reflected in GlobalMemoryStatus().
            // #endif
            // size = 0x4000000 - (exe + ZONE_HEAP_FREE);
            // #	else
            // size = status.dwAvailPhys - ZONE_HEAP_FREE;
            // #	endif
            // s_PoolBase = GlobalAlloc(0, size);
        }

        #[cfg(target_os = "windows")]
        {
            let size: usize = 50 * 1024 * 1024;
            s_PoolBase = GlobalAlloc(0, size);
        }

        // Setup the initial free block
        let base = s_PoolBase as *mut ZoneFreeBlock;
        (*base).m_Address = s_PoolBase as u32;
        (*base).m_Size = 0; // Would be set to size in real code
        (*base).m_Next = addr_of_mut!(s_FreeEnd);
        (*base).m_Prev = addr_of_mut!(s_FreeStart);

        // Init the free block jump table
        memset(addr_of_mut!(s_FreeJumpTable) as *mut c_void, 0,
               Z_JUMP_TABLE_SIZE * core::mem::size_of::<*mut ZoneFreeBlock>());
        s_FreeJumpResolution = 0; // Would calculate from size
        s_FreeJumpTable[0] = base;

        // Setup free block dummies
        s_FreeStart.m_Address = 0;
        s_FreeStart.m_Size = 0;
        s_FreeStart.m_Next = base;
        s_FreeStart.m_Prev = core::ptr::null_mut();

        s_FreeEnd.m_Address = 0xFFFFFFFF;
        s_FreeEnd.m_Size = 0;
        s_FreeEnd.m_Next = core::ptr::null_mut();
        s_FreeEnd.m_Prev = base;

        (*addr_of_mut!(s_Stats)).m_CountFree = 1;
        (*addr_of_mut!(s_Stats)).m_SizeFree = 0; // Would be set to size

        s_Initialized = true;

        // Add some commands
        Cmd_AddCommand(b"zone_stats\0" as *const u8 as *const c_char, Z_Stats_f);
        Cmd_AddCommand(b"zone_details\0" as *const u8 as *const c_char, Z_Details_f);
        Cmd_AddCommand(b"zone_memmap\0" as *const u8 as *const c_char, Z_DumpMemMap_f);

        #[cfg(not(target_os = "gamecube"))]
        {
            #[cfg(target_os = "windows")]
            {
                s_Mutex = CreateMutex(core::ptr::null_mut(), 0, core::ptr::null());
            }
        }
    }
}

pub fn Com_ShutdownZoneMemory() {
    unsafe {
        assert!(s_Initialized);

        // Remove commands
        Cmd_RemoveCommand(b"zone_stats\0" as *const u8 as *const c_char);
        Cmd_RemoveCommand(b"zone_details\0" as *const u8 as *const c_char);
        Cmd_RemoveCommand(b"zone_memmap\0" as *const u8 as *const c_char);

        if (*addr_of!(s_Stats)).m_CountAlloc != 0 {
            Z_TagFree(64); // TAG_ALL
        }

        // Clear some globals
        memset(addr_of_mut!(s_Stats) as *mut c_void, 0, core::mem::size_of::<ZoneStats>());
        memset(addr_of_mut!(s_FreeOverflow) as *mut c_void, 0,
               core::mem::size_of::<[ZoneFreeBlock; ZONE_FREE_OVERFLOW]>());
        s_LastOverflowIndex = 0;
        s_LinkBase = core::ptr::null_mut();

        // Free the pool
        #[cfg(not(target_os = "gamecube"))]
        {
            #[cfg(target_os = "windows")]
            {
                GlobalFree(s_PoolBase);
                CloseHandle(s_Mutex);
            }
        }

        s_PoolBase = core::ptr::null_mut();
        s_Initialized = false;
    }
}

// Determine if a tag should only be allocated for a very
// short period of time.
fn Z_IsTagTemp(eTag: memtag_t) -> bool {
    eTag == 25 || // TAG_TEMP_WORKSPACE
    #[cfg(not(feature = "jk2mp"))]
    eTag == 40 || // TAG_TEMP_SAVEGAME_WORKSPACE
    #[cfg(not(feature = "jk2mp"))]
    eTag == 23 || // TAG_STRING
    eTag == 21 || // TAG_SND_RAWDATA
    eTag == 27 || // TAG_ICARUS
    eTag == 22 || // TAG_LISTFILES
    #[cfg(feature = "jk2mp")]
    eTag == 52 || // TAG_TEXTPOOL
    #[cfg(not(feature = "jk2mp"))]
    eTag == 30   // TAG_GP2
}

// Determine if a tag needs TagFree() support.
fn Z_IsTagLinked(eTag: memtag_t) -> bool {
    eTag == 5 || // TAG_BSP
    eTag == 12 || // TAG_HUNKALLOC
    #[cfg(not(feature = "jk2mp"))]
    eTag == 13 || // TAG_HUNKMISCMODELS
    #[cfg(not(feature = "jk2mp"))]
    eTag == 11 || // TAG_G_ALLOC
    #[cfg(feature = "jk2mp")]
    eTag == 50 || // TAG_CG_UI_ALLOC
    #[cfg(feature = "jk2mp")]
    eTag == 48 || // TAG_BG_ALLOC
    eTag == 49    // TAG_UI_ALLOC
}

fn Z_CalcAlignmentPad(iAlign: c_int, iAddress: u32, iOffset: u32,
    iSize: u32, iHeaderSize: u32, iFooterSize: u32) -> c_int {
    let mut align_size: c_int;

    if iAlign == 0 { return 0; }

    if iOffset == 0 {
        // Align data at low end of block
        align_size = iAlign -
            (((iAddress + iHeaderSize) % (iAlign as u32)) as c_int);
    }
    else {
        // Align data at high end of block
        let block_start = iAddress + iOffset -
            iSize + iHeaderSize;
        align_size = (block_start % (iAlign as u32)) as c_int;
    }

    if align_size == iAlign {
        return 0;
    }

    align_size
}

fn Z_GetOverflowBlock() -> *mut ZoneFreeBlock {
    unsafe {
        for i in (s_LastOverflowIndex as usize)..ZONE_FREE_OVERFLOW {
            if (*addr_of!(s_FreeOverflow[i])).m_Address == 0 {
                s_LastOverflowIndex = i as c_int;
                return addr_of_mut!(s_FreeOverflow[i]);
            }
        }

        for j in 0..(s_LastOverflowIndex as usize) {
            if (*addr_of!(s_FreeOverflow[j])).m_Address == 0 {
                s_LastOverflowIndex = j as c_int;
                return addr_of_mut!(s_FreeOverflow[j]);
            }
        }

        core::ptr::null_mut()
    }
}

fn Z_IsFreeBlockLargeEnough(pBlock: *mut ZoneFreeBlock, iSize: c_int,
    iHeaderSize: c_int, iFooterSize: c_int, iAlign: c_int, bLow: bool,
    iAlignPad: &mut c_int) -> bool {
    unsafe {
        // Is the block large enough?
        if (*pBlock).m_Size >= (iSize as u32) {
            if iAlign > 0 {
                // If we need some aligment, we need to check size
                // against that as well.
                *iAlignPad = Z_CalcAlignmentPad(iAlign,
                    (*pBlock).m_Address, if !bLow { (*pBlock).m_Size } else { 0 },
                    iSize as u32, iHeaderSize as u32, iFooterSize as u32);

                if (*pBlock).m_Size < (*iAlignPad as u32) + (iSize as u32) {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

fn Z_FindFirstFree(iSize: c_int, iHeaderSize: c_int,
    iFooterSize: c_int, iAlign: c_int, iAlignPad: &mut c_int) -> *mut ZoneFreeBlock {
    unsafe {
        let mut block = (*addr_of!(s_FreeStart)).m_Next;
        while !block.is_null() {
            if Z_IsFreeBlockLargeEnough(block, iSize, iHeaderSize, iFooterSize,
                iAlign, true, iAlignPad) {
                return block;
            }
            block = (*block).m_Next;
        }
        core::ptr::null_mut()
    }
}

fn Z_FindLastFree(iSize: c_int, iHeaderSize: c_int,
    iFooterSize: c_int, iAlign: c_int, iAlignPad: &mut c_int) -> *mut ZoneFreeBlock {
    unsafe {
        let mut block = (*addr_of!(s_FreeEnd)).m_Prev;
        while !block.is_null() {
            if Z_IsFreeBlockLargeEnough(block, iSize, iHeaderSize, iFooterSize,
                iAlign, false, iAlignPad) {
                return block;
            }
            block = (*block).m_Prev;
        }
        core::ptr::null_mut()
    }
}

fn Z_ValidateFree() -> bool {
    #[cfg(test)]
    if ZONE_DEBUG != 0 {
        unsafe {
            // Make sure no free blocks are overlapping
            let mut a = addr_of_mut!(s_FreeStart);
            while !a.is_null() {
                if (*a).m_Address == 0 && (*a).m_Size != 0 {
                    return false;
                }

                let mut b = addr_of_mut!(s_FreeStart);
                while !b.is_null() {
                    if a != b &&
                        (*a).m_Address >= (*b).m_Address &&
                        (*a).m_Address < (*b).m_Address + (*b).m_Size {
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

fn Z_ValidateLinks() -> bool {
    #[cfg(test)]
    if ZONE_DEBUG != 0 {
        unsafe {
            // Make sure links are sane
            let mut a = s_LinkBase;
            while !a.is_null() {
                if (!(*a).m_Next.is_null() && a != (*(*a).m_Next).m_Prev) ||
                    (!(*a).m_Prev.is_null() && a != (*(*a).m_Prev).m_Next) {
                    return false;
                }
                a = (*a).m_Next;
            }
        }
    }

    true
}

fn Z_GetJumpTableIndex(iAddress: u32) -> c_int {
    unsafe {
        let index = ((iAddress - (s_PoolBase as u32)) / s_FreeJumpResolution) as c_int;
        if index < 0 { return 0; }
        if index >= (Z_JUMP_TABLE_SIZE as c_int) { return (Z_JUMP_TABLE_SIZE - 1) as c_int; }
        index
    }
}

fn Z_GetFreeBlockBefore(iAddress: u32) -> *mut ZoneFreeBlock {
    unsafe {
        // Find this block's position in the jump table
        let mut index = Z_GetJumpTableIndex(iAddress) - 1;

        // Find a valid jump table entry
        while index >= 0 && s_FreeJumpTable[index as usize].is_null() {
            index -= 1;
        }

        if index < 0 {
            return addr_of_mut!(s_FreeStart);
        }
        s_FreeJumpTable[index as usize]
    }
}

fn Z_RemoveFromJumpTable(pBlock: *mut ZoneFreeBlock) {
    unsafe {
        // Is this block in the jump table?
        let index = Z_GetJumpTableIndex((*pBlock).m_Address) as usize;
        if s_FreeJumpTable[index] == pBlock {
            // See if the next block will fit in our slot
            if (*pBlock).m_Next != addr_of_mut!(s_FreeEnd) {
                let nindex = Z_GetJumpTableIndex((*(*pBlock).m_Next).m_Address) as usize;
                if nindex == index {
                    s_FreeJumpTable[index] = (*pBlock).m_Next;
                    return;
                }
            }

            // See if the previous block will fit in our slot
            if (*pBlock).m_Prev != addr_of_mut!(s_FreeStart) {
                let pindex = Z_GetJumpTableIndex((*(*pBlock).m_Prev).m_Address) as usize;
                if pindex == index {
                    s_FreeJumpTable[index] = (*pBlock).m_Prev;
                    return;
                }
            }

            // No other free blocks fit here, give up
            s_FreeJumpTable[index] = core::ptr::null_mut();
        }
    }
}

fn Z_LinkFreeBlock(pBlock: *mut ZoneFreeBlock) {
    unsafe {
        let mut cur = Z_GetFreeBlockBefore((*pBlock).m_Address);
        while !cur.is_null() {
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

                (*addr_of_mut!(s_Stats)).m_CountFree += 1;
                (*addr_of_mut!(s_Stats)).m_SizeFree += (*pBlock).m_Size;

                assert!(Z_ValidateFree());
                break;
            }
            cur = (*cur).m_Next;
        }
    }
}

fn Z_SplitFree(pBlock: *mut ZoneFreeBlock, iSize: c_int, bLow: bool) -> *mut c_void {
    unsafe {
        assert!((*pBlock).m_Size >= (iSize as u32));

        Z_RemoveFromJumpTable(pBlock);

        // Delink the free block
        let fblock = *pBlock;
        (*(*pBlock).m_Prev).m_Next = (*pBlock).m_Next;
        (*(*pBlock).m_Next).m_Prev = (*pBlock).m_Prev;
        (*pBlock).m_Address = 0;

        (*addr_of_mut!(s_Stats)).m_CountFree -= 1;
        (*addr_of_mut!(s_Stats)).m_SizeFree -= (*pBlock).m_Size;
        assert!(Z_ValidateFree());

        if fblock.m_Size > (iSize as u32) {
            // Split the block into an allocated and free portion
            let remainder = (fblock.m_Size as c_int - iSize) as u32;

            if remainder < (core::mem::size_of::<ZoneFreeBlock>() as u32) {
                // Free portion is not large to hold free info --
                // we're going to have to use the overflow buffer.
                let nblock = Z_GetOverflowBlock();

                if nblock.is_null() {
                    Z_Details_f();
                    Com_Error(ERR_FATAL, b"Zone free overflow buffer overflowed!\0" as *const u8 as *const c_char);
                }

                // Split the block
                let ret: *mut c_void;
                if bLow {
                    ret = fblock.m_Address as *mut c_void;
                    (*nblock).m_Address = fblock.m_Address + (iSize as u32);
                }
                else {
                    ret = (fblock.m_Address + remainder) as *mut c_void;
                    (*nblock).m_Address = fblock.m_Address;
                }

                (*nblock).m_Size = remainder;
                Z_LinkFreeBlock(nblock);

                return ret;
            }
            else {
                // Free portion is large enough -- split it
                let ret: *mut c_void;
                let nblock: *mut ZoneFreeBlock;
                if bLow {
                    ret = fblock.m_Address as *mut c_void;
                    nblock = (fblock.m_Address + (iSize as u32)) as *mut ZoneFreeBlock;
                }
                else {
                    ret = (fblock.m_Address + remainder) as *mut c_void;
                    nblock = fblock.m_Address as *mut ZoneFreeBlock;
                }

                (*nblock).m_Address = nblock as u32;
                (*nblock).m_Size = remainder;

                Z_LinkFreeBlock(nblock);

                return ret;
            }
        }
        else {
            // No need to split, just return block.
            return fblock.m_Address as *mut c_void;
        }
    }
}

fn Z_SetupAlignmentPad(pBlock: *mut c_void, iAlignPad: c_int, bLow: bool) {
    // Clear alignment bytes
    memset(pBlock, 0, iAlignPad as usize);

    // If we have more than 1 alignment byte, the first align byte
    // tells us how many additional bytes we have.
    if iAlignPad > 1 {
        assert!((iAlignPad as u32) < 256);
        let ptr: *mut u8;
        unsafe {
            if bLow {
                ptr = (pBlock as *mut u8).add((iAlignPad - 1) as usize);
            }
            else {
                ptr = pBlock as *mut u8;
            }
            *ptr = (iAlignPad - 1) as u8;
        }
    }
}

pub fn Z_MallocFail(pMessage: *const c_char, iSize: c_int, eTag: memtag_t) {
    // Report the error
    Com_Printf(b"Z_Malloc(): %s : %d bytes and tag %d !!!!\n\0" as *const u8 as *const c_char, pMessage, iSize, eTag);
    Z_Details_f();
    Z_DumpMemMap_f();
    Com_Printf(b"(Repeat): Z_Malloc(): %s : %d bytes and tag %d !!!!\n\0" as *const u8 as *const c_char, pMessage, iSize, eTag);

    // Clear the screen blue to indicate out of memory
    #[cfg(target_os = "windows")]
    loop {
        unsafe {
            qglBeginFrame();
            qglClearColor(0.0, 0.0, 1.0, 1.0);
            qglClear(GL_COLOR_BUFFER_BIT);
            qglEndFrame();
        }
    }
}

pub fn Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean, iAlign: c_int) -> *mut c_void {
    unsafe {
        if !s_Initialized {
            Com_InitZoneMemory();
        }

        if iSize == 0 {
            #[cfg(debug_assertions)]
            {
                return addr_of!(s_EmptyBlock.start).add(1) as *mut c_void;
            }
            #[cfg(not(debug_assertions))]
            {
                return addr_of!(s_EmptyBlock.header).add(1) as *mut c_void;
            }
        }

        if iSize < 0 {
            Z_MallocFail(b"Negative size\0" as *const u8 as *const c_char, iSize, eTag);
            return core::ptr::null_mut();
        }

        #[cfg(not(target_os = "gamecube"))]
        #[cfg(target_os = "windows")]
        {
            WaitForSingleObject(s_Mutex, 0xFFFFFFFF);
        }

        // Make new/delete memory temporary if requested
        let mut eTag = eTag;
        if eTag == 3 && s_IsNewDeleteTemp { // TAG_NEWDEL == 3
            eTag = 25; // TAG_TEMP_WORKSPACE
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
        let fblock: *mut ZoneFreeBlock;
        if Z_IsTagTemp(eTag) {
            fblock = Z_FindLastFree(real_size, header_size, footer_size,
                iAlign, &mut align_pad);
        }
        else {
            fblock = Z_FindFirstFree(real_size, header_size, footer_size,
                iAlign, &mut align_pad);
        }

        // Did we actually find some memory?
        if fblock.is_null() {
            #[cfg(not(target_os = "gamecube"))]
            #[cfg(target_os = "windows")]
            {
                ReleaseMutex(s_Mutex);
            }

            if eTag == 21 { // TAG_SND_RAWDATA
                return core::ptr::null_mut();
            }

            Z_MallocFail(b"Out of memory\0" as *const u8 as *const c_char, iSize, eTag);
            return core::ptr::null_mut();
        }

        // Add any alignment bytes
        let real_size = real_size + align_pad;

        // Split the free block and get a pointer to the start
        // allocated space.
        let mut ablock: *mut c_void;
        if Z_IsTagTemp(eTag) {
            ablock = Z_SplitFree(fblock, real_size, false);

            // Append align pad to end of block
            Z_SetupAlignmentPad(
                (ablock as *mut u8).add(real_size as usize - align_pad as usize) as *mut c_void,
                align_pad, false);
        }
        else {
            ablock = Z_SplitFree(fblock, real_size, true);

            // Insert align pad at block start
            Z_SetupAlignmentPad(ablock, align_pad, true);
            ablock = (ablock as *mut u8).add(align_pad as usize) as *mut c_void;
        }

        if ablock.is_null() {
            Z_MallocFail(b"Failed to split\0" as *const u8 as *const c_char, iSize, eTag);
        }

        // Add linking header if necessary
        if Z_IsTagLinked(eTag) {
            let linked = ablock as *mut ZoneLinkHeader;
            (*linked).m_Next = s_LinkBase;
            (*linked).m_Prev = core::ptr::null_mut();
            if !s_LinkBase.is_null() {
                (*s_LinkBase).m_Prev = linked;
            }
            s_LinkBase = linked;

            assert!(Z_ValidateLinks());

            // Next...
            ablock = (ablock as *mut u8).add(core::mem::size_of::<ZoneLinkHeader>()) as *mut c_void;
        }

        // Setup the header:
        //		31		- alignment flag
        //		25-30	- tag
        //		0-24	- size without headers/footers
        assert!(iSize >= 0 && iSize < (1 << 25));
        assert!(eTag >= 0 && eTag < 64);
        let header = ablock as *mut ZoneHeader;
        *header =
            (((eTag as u32) << 25) |
            (iSize as u32));

        if align_pad != 0 {
            *header |= 1 << 31;
        }

        // Next...
        ablock = (ablock as *mut u8).add(core::mem::size_of::<ZoneHeader>()) as *mut c_void;

        #[cfg(debug_assertions)]
        {
            // Setup the debug markers
            let debug_header = ablock as *mut ZoneDebugHeader;

            let debug_footer = (ablock as *mut u8).add(
                core::mem::size_of::<ZoneDebugHeader>() + iSize as usize) as *mut ZoneDebugFooter;

            *debug_header = ZONE_MAGIC;
            *debug_footer = ZONE_MAGIC as u8;

            // Next...
            ablock = (ablock as *mut u8).add(core::mem::size_of::<ZoneDebugHeader>()) as *mut c_void;
        }

        // Update the stats
        (*addr_of_mut!(s_Stats)).m_SizeAlloc += iSize;
        (*addr_of_mut!(s_Stats)).m_OverheadAlloc += header_size + footer_size + align_pad;
        (*addr_of_mut!(s_Stats)).m_SizesPerTag[eTag as usize] += iSize;
        (*addr_of_mut!(s_Stats)).m_CountAlloc += 1;
        (*addr_of_mut!(s_Stats)).m_CountsPerTag[eTag as usize] += 1;

        if (*addr_of_mut!(s_Stats)).m_SizeAlloc + (*addr_of_mut!(s_Stats)).m_OverheadAlloc >
           (*addr_of_mut!(s_Stats)).m_PeakAlloc {
            (*addr_of_mut!(s_Stats)).m_PeakAlloc =
                (*addr_of_mut!(s_Stats)).m_SizeAlloc + (*addr_of_mut!(s_Stats)).m_OverheadAlloc;
        }

        // Return a pointer to data memory
        if bZeroit != 0 {
            memset(ablock, 0, iSize as usize);
        }

        assert!(iAlign == 0 || (ablock as u32) % (iAlign as u32) == 0);

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

        if ((unsigned)ablock >= 0x1eb0000 && (unsigned)ablock <= 0x1ec0000 && iSize == 48)
        {
            int suck = 0;
        }
        */

        #[cfg(not(target_os = "gamecube"))]
        #[cfg(target_os = "windows")]
        {
            ReleaseMutex(s_Mutex);
        }

        ablock
    }
}

fn Z_GetTag(header: *const ZoneHeader) -> memtag_t {
    unsafe {
        ((*header & 0x7E000000) >> 25) as memtag_t
    }
}

fn Z_GetSize(header: *const ZoneHeader) -> u32 {
    unsafe {
        *header & 0x1FFFFFF
    }
}

fn Z_GetAlign(header: *const ZoneHeader) -> c_int {
    unsafe {
        if *header & (1 << 31) != 0 {
            let ptr = header as *mut u8;
            let tag = Z_GetTag(header);

            // point to the first alignment block
            if Z_IsTagTemp(tag) {
                let mut p = ptr as usize;
                p += core::mem::size_of::<ZoneHeader>();
                p += Z_GetSize(header) as usize;
                #[cfg(debug_assertions)]
                {
                    p += core::mem::size_of::<ZoneDebugHeader>();
                    p += core::mem::size_of::<ZoneDebugFooter>();
                }
                return (*(p as *const u8) as c_int) + 1;
            }
            else {
                let mut p = ptr as usize;
                if Z_IsTagLinked(tag) {
                    // skip the link header
                    p -= core::mem::size_of::<ZoneLinkHeader>();
                }
                p -= 1;
                return (*(p as *const u8) as c_int) + 1;
            }
        }
        0
    }
}

pub fn Z_Size(pvAddress: *mut c_void) -> c_int {
    unsafe {
        assert!(s_Initialized);

        #[cfg(debug_assertions)]
        {
            let debug = (pvAddress as *mut ZoneDebugHeader).offset(-1);

            if *debug != ZONE_MAGIC {
                Com_Error(ERR_FATAL, b"Z_Size(): Not a valid zone header!\0" as *const u8 as *const c_char);
                return 0;
            }

            let pvAddress = debug as *mut c_void;
            let header = (pvAddress as *mut ZoneHeader).offset(-1);

            if Z_GetTag(header) == TAG_STATIC {
                return 0;
            }

            return Z_GetSize(header) as c_int;
        }

        #[cfg(not(debug_assertions))]
        {
            let header = (pvAddress as *mut ZoneHeader).offset(-1);

            if Z_GetTag(header) == TAG_STATIC {
                return 0;
            }

            Z_GetSize(header) as c_int
        }
    }
}

fn Z_Coalasce(pBlock: *mut ZoneFreeBlock) {
    unsafe {
        let mut size = 0u32;

        // Find later free blocks adjacent to us
        let mut end = (*pBlock).m_Next;
        while !(*end).m_Next.is_null() {
            if (*end).m_Address !=
                (*(*end).m_Prev).m_Address + (*(*end).m_Prev).m_Size {
                break;
            }

            size += (*end).m_Size;

            Z_RemoveFromJumpTable(end);

            (*end).m_Address = 0; // invalidate block
            (*addr_of_mut!(s_Stats)).m_CountFree -= 1;

            end = (*end).m_Next;
        }

        // Find previous free blocks adjacent to us
        let mut start = pBlock;
        while !(*start).m_Prev.is_null() {
            if (*(*start).m_Prev).m_Address + (*(*start).m_Prev).m_Size !=
                (*start).m_Address {
                break;
            }

            size += (*start).m_Size;

            Z_RemoveFromJumpTable(start);

            (*start).m_Address = 0; // invalidate block
            (*addr_of_mut!(s_Stats)).m_CountFree -= 1;

            start = (*start).m_Prev;
        }

        // Do we need to coalesce some blocks?
        if (*start).m_Next != end {
            (*start).m_Next = end;
            (*end).m_Prev = start;
            (*start).m_Size += size;
        }
    }
}

// Return type of Z_Free differs in SP/MP. Macro hack to wrap it up
#[cfg(feature = "jk2mp")]
pub fn Z_Free(pvAddress: *mut c_void) {
    Z_Free_impl(pvAddress);
}

#[cfg(not(feature = "jk2mp"))]
pub fn Z_Free(pvAddress: *mut c_void) -> c_int {
    Z_Free_impl(pvAddress);
    0
}

fn Z_Free_impl(pvAddress: *mut c_void) {
    unsafe {
        #[cfg(target_os = "windows")]
        {
            if !s_Initialized { return; }
        }

        assert!(s_Initialized);

        #[cfg(debug_assertions)]
        {
            // check the header magic
            let debug_header = (pvAddress as *mut ZoneDebugHeader).offset(-1);

            if *debug_header != ZONE_MAGIC {
                Com_Error(ERR_FATAL, b"Z_Free(): Corrupt zone header!\0" as *const u8 as *const c_char);
                return;
            }

            let header = (debug_header as *mut ZoneHeader).offset(-1);

            // check the footer magic
            let debug_footer = (pvAddress as *mut u8).add(Z_GetSize(header) as usize) as *mut ZoneDebugFooter;

            if *debug_footer != ZONE_MAGIC as u8 {
                Com_Error(ERR_FATAL, b"Z_Free(): Corrupt zone footer!\0" as *const u8 as *const c_char);
                return;
            }
        }

        #[cfg(not(debug_assertions))]
        let header = (pvAddress as *mut ZoneHeader).offset(-1);

        let tag = Z_GetTag(header);

        if tag != TAG_STATIC {
            #[cfg(not(target_os = "gamecube"))]
            #[cfg(target_os = "windows")]
            {
                WaitForSingleObject(s_Mutex, 0xFFFFFFFF);
            }

            // Determine size of header and footer
            let mut header_size = core::mem::size_of::<ZoneHeader>() as c_int;
            let align_size = Z_GetAlign(header);
            let mut footer_size = 0;
            let data_size = Z_GetSize(header) as c_int;
            if Z_IsTagLinked(tag) {
                header_size += core::mem::size_of::<ZoneLinkHeader>() as c_int;
            }
            if Z_IsTagTemp(tag) {
                footer_size += align_size;
            }
            else {
                header_size += align_size;
            }
            #[cfg(debug_assertions)]
            {
                header_size += core::mem::size_of::<ZoneDebugHeader>() as c_int;
                footer_size += core::mem::size_of::<ZoneDebugFooter>() as c_int;
            }
            let real_size = (data_size + header_size + footer_size) as u32;

            // Update the stats
            (*addr_of_mut!(s_Stats)).m_SizeAlloc -= data_size;
            (*addr_of_mut!(s_Stats)).m_OverheadAlloc -= header_size + footer_size;
            (*addr_of_mut!(s_Stats)).m_SizesPerTag[tag as usize] -= data_size;
            (*addr_of_mut!(s_Stats)).m_CountAlloc -= 1;
            (*addr_of_mut!(s_Stats)).m_CountsPerTag[tag as usize] -= 1;

            // Delink block
            if Z_IsTagLinked(tag) {
                let linked = (header as *mut ZoneLinkHeader).offset(-1);

                if linked == s_LinkBase {
                    s_LinkBase = (*linked).m_Next;
                    if !s_LinkBase.is_null() {
                        (*s_LinkBase).m_Prev = core::ptr::null_mut();
                    }
                }
                else {
                    if !(*linked).m_Next.is_null() {
                        (*(*linked).m_Next).m_Prev = (*linked).m_Prev;
                    }
                    (*(*linked).m_Prev).m_Next = (*linked).m_Next;
                }

                assert!(Z_ValidateLinks());
            }

            // Clear the block header for safety
            *(header as *mut ZoneHeader) = 0;

            // Add block to free list
            let nblock: *mut ZoneFreeBlock;
            if real_size < (core::mem::size_of::<ZoneFreeBlock>() as u32) {
                // Not enough space in block to put free information --
                // use overflow buffer.
                nblock = Z_GetOverflowBlock();

                if nblock.is_null() {
                    Z_Details_f();
                    Com_Error(ERR_FATAL, b"Zone free overflow buffer overflowed!\0" as *const u8 as *const c_char);
                }
            }
            else {
                // Place free information in block
                nblock = (pvAddress as *mut u8).sub(header_size as usize) as *mut ZoneFreeBlock;
            }

            (*nblock).m_Address = (pvAddress as u32).wrapping_sub(header_size as u32);
            (*nblock).m_Size = real_size;
            Z_LinkFreeBlock(nblock);

            // Coalesce any adjacent free blocks
            Z_Coalasce(nblock);
            #[cfg(not(target_os = "gamecube"))]
            #[cfg(target_os = "windows")]
            {
                ReleaseMutex(s_Mutex);
            }
        }
    }
}

pub fn Z_MemSize(eTag: memtag_t) -> c_int {
    unsafe {
        (*addr_of!(s_Stats)).m_SizesPerTag[eTag as usize]
    }
}

#[cfg(test)]
fn Z_FindLeak() {
    unsafe {
        assert!(s_Initialized);

        static mut cycle_count: c_int = 0;
        const tag: memtag_t = 3; // TAG_NEWDEL

        #[repr(C)]
        struct PointerInfo {
            data: *mut c_void,
            counter: c_int,
            mark: bool,
        }

        const max_pointers: usize = 32768;
        static mut pointers: [PointerInfo; max_pointers] =
            [PointerInfo {
                data: core::ptr::null_mut(),
                counter: 0,
                mark: false,
            }; max_pointers];
        static mut num_pointers: c_int = 0;

        // Clear pointer existance
        for i in 0..(num_pointers as usize) {
            pointers[i].mark = false;
        }

        // Add all known pointers
        let start_num = num_pointers;
        let mut link = s_LinkBase;
        while !link.is_null() {
            let header = (link as *mut ZoneHeader).offset(1);
            link = (*link).m_Next;

            if Z_GetTag(header) == tag {
                // See if the pointer already is in the array
                let mut found = false;
                for k in (start_num as usize)..(num_pointers as usize) {
                    if pointers[k].data == header as *mut c_void {
                        pointers[k].counter += 1;
                        pointers[k].mark = true;
                        found = true;
                        break;
                    }
                }

                // If the pointer is not in the array, add it
                if !found {
                    assert!((num_pointers as usize) < max_pointers);
                    pointers[num_pointers as usize].data = header as *mut c_void;
                    pointers[num_pointers as usize].counter = 0;
                    pointers[num_pointers as usize].mark = true;
                    num_pointers += 1;
                }
            }
        }

        // Remove pointers that are no longer used
        let mut j = 0;
        while j < (num_pointers as usize) {
            if pointers[j].mark {
                if pointers[j].counter != cycle_count &&
                    pointers[j].counter != cycle_count - 1 &&
                    pointers[j].counter != 0 {
                    Com_Printf(b"Memory leak: %p\n\0" as *const u8 as *const c_char, pointers[j].data);
                }
            }
            else {
                let mut k = j;
                while k < (num_pointers as usize) {
                    if pointers[k].mark { break; }
                    k += 1;
                }

                if k == (num_pointers as usize) { break; }

                memmove(pointers.as_mut_ptr().add(j) as *mut c_void,
                        pointers.as_ptr().add(k) as *const c_void,
                        ((num_pointers - k as c_int) as usize) * core::mem::size_of::<PointerInfo>());
                num_pointers -= (k - j) as c_int;
            }
            j += 1;
        }

        cycle_count += 1;
    }
}

pub fn Z_TagPointers(eTag: memtag_t) {
    unsafe {
        assert!(s_Initialized);

        #[cfg(not(target_os = "gamecube"))]
        #[cfg(target_os = "windows")]
        {
            WaitForSingleObject(s_Mutex, 0xFFFFFFFF);
        }

        Sys_Log(b"pointers.txt\0" as *const u8 as *const c_char,
                va(b"Pointers for tag %d:\n\0" as *const u8 as *const c_char, eTag));

        let mut link = s_LinkBase;
        while !link.is_null() {
            let header = (link as *mut ZoneHeader).offset(1);
            link = (*link).m_Next;

            if eTag == 64 || Z_GetTag(header) == eTag { // TAG_ALL = 64
                #[cfg(debug_assertions)]
                {
                    let ptr = (header as *mut u8).add(
                        core::mem::size_of::<ZoneHeader>() +
                        core::mem::size_of::<ZoneDebugHeader>()) as *mut c_void;
                    Sys_Log(b"pointers.txt\0" as *const u8 as *const c_char,
                            va(b"%x - %d\n\0" as *const u8 as *const c_char, ptr,
                            Z_Size(ptr)));
                }
                #[cfg(not(debug_assertions))]
                {
                    let ptr = (header as *mut u32).offset(1) as *mut c_void;
                    Sys_Log(b"pointers.txt\0" as *const u8 as *const c_char,
                            va(b"%x - %d\n\0" as *const u8 as *const c_char, ptr,
                            Z_Size(ptr)));
                }
            }
        }

        #[cfg(not(target_os = "gamecube"))]
        #[cfg(target_os = "windows")]
        {
            ReleaseMutex(s_Mutex);
        }
    }
}

pub fn Z_TagFree(eTag: memtag_t) {
    unsafe {
        assert!(s_Initialized);

        let mut link = s_LinkBase;
        while !link.is_null() {
            let header = (link as *mut ZoneHeader).offset(1);
            link = (*link).m_Next;

            if eTag == 64 || Z_GetTag(header) == eTag { // TAG_ALL = 64
                #[cfg(debug_assertions)]
                {
                    Z_Free((header as *mut u8).add(
                        core::mem::size_of::<ZoneHeader>() +
                        core::mem::size_of::<ZoneDebugHeader>()) as *mut c_void);
                }
                #[cfg(not(debug_assertions))]
                {
                    Z_Free((header as *mut u32).offset(1) as *mut c_void);
                }
            }
        }
    }
}

pub fn Z_SetNewDeleteTemporary(bTemp: bool) {
    // Catch nested uses that break when unwinding the stack
    unsafe {
        assert!(bTemp != s_IsNewDeleteTemp);
        s_IsNewDeleteTemp = bTemp;
    }
}

pub fn S_Malloc(iSize: c_int) -> *mut c_void {
    Z_Malloc(iSize, 2, 0, 0) // TAG_SMALL = 2
}

pub fn Z_GetLevelMemory() -> c_int {
    unsafe {
        #[cfg(feature = "jk2mp")]
        {
            (*addr_of!(s_Stats)).m_SizesPerTag[5] // TAG_BSP
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            (*addr_of!(s_Stats)).m_SizesPerTag[12] + // TAG_HUNKALLOC
                (*addr_of!(s_Stats)).m_SizesPerTag[13] + // TAG_HUNKMISCMODELS
                (*addr_of!(s_Stats)).m_SizesPerTag[5] // TAG_BSP
        }
    }
}

#[cfg(feature = "jk2mp")]
pub fn Z_GetHunkMemory() -> c_int {
    unsafe {
        (*addr_of!(s_Stats)).m_SizesPerTag[12] + // TAG_HUNKALLOC
            (*addr_of!(s_Stats)).m_SizesPerTag[14] // TAG_TEMP_HUNKALLOC
    }
}

pub fn Z_GetTerrainMemory() -> c_int {
    unsafe {
        let mut total =
            (*addr_of!(s_Stats)).m_SizesPerTag[38] + // TAG_CM_TERRAIN
                (*addr_of!(s_Stats)).m_SizesPerTag[39]; // TAG_CM_TERRAIN_TEMP
        #[cfg(feature = "jk2mp")]
        {
            total += (*addr_of!(s_Stats)).m_SizesPerTag[34]; // TAG_TERRAIN
        }
        total += (*addr_of!(s_Stats)).m_SizesPerTag[35]; // TAG_R_TERRAIN
        total
    }
}

pub fn Z_GetMiscMemory() -> c_int {
    unsafe {
        let mut excluded = Z_GetLevelMemory();
        #[cfg(feature = "jk2mp")]
        {
            excluded += Z_GetHunkMemory();
        }
        excluded += Z_GetTerrainMemory() +
            (*addr_of!(s_Stats)).m_SizesPerTag[7] + // TAG_MODEL_GLM
            (*addr_of!(s_Stats)).m_SizesPerTag[8] + // TAG_MODEL_GLA
            (*addr_of!(s_Stats)).m_SizesPerTag[9] + // TAG_MODEL_MD3
            (*addr_of!(s_Stats)).m_SizesPerTag[28] + // TAG_BINK
            (*addr_of!(s_Stats)).m_SizesPerTag[21]; // TAG_SND_RAWDATA

        (*addr_of!(s_Stats)).m_SizeAlloc - excluded
    }
}

#[cfg(target_os = "gamecube")]
static mut texMemSize: c_int = 0;

#[cfg(not(target_os = "gamecube"))]
extern "C" {
    static mut texMemSize: c_int;
}

pub fn Z_CompactStats() {
    unsafe {
        assert!(s_Initialized);

        //This report is conservative.  Divides by 1000 instead of 1024 and
        //then rounds up.
        static mut printHeader: c_int = 1;
        if printHeader != 0 {
            Sys_Log(b"memory-map.txt\0" as *const u8 as *const c_char,
                    b"**Z_CompactStats Start**\n\n\0" as *const u8 as *const c_char);
            Sys_Log(b"memory-map.txt\0" as *const u8 as *const c_char,
                    b"Map:\tOV:\tLVL:\tGLM:\tGLA:\tMD3:\tTER:\tSND:\tTEX:\tFMV:\tMSC:\tFrZN:\tFrPH:\n\0" as *const u8 as *const c_char);
            printHeader = 0;
        }

        // MEMORYSTATUS stat;
        // GlobalMemoryStatus(&stat);

        Sys_Log(b"memory-map.txt\0" as *const u8 as *const c_char,
                va(b"%s\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n\0" as *const u8 as *const c_char,
                Cvar_VariableString(b"mapname\0" as *const u8 as *const c_char),
                ((*addr_of!(s_Stats)).m_OverheadAlloc / 1000) + 1,
                (Z_GetLevelMemory() / 1000) + 1,
                ((*addr_of!(s_Stats)).m_SizesPerTag[7] / 1000) + 1, // TAG_MODEL_GLM
                ((*addr_of!(s_Stats)).m_SizesPerTag[8] / 1000) + 1, // TAG_MODEL_GLA
                ((*addr_of!(s_Stats)).m_SizesPerTag[9] / 1000) + 1, // TAG_MODEL_MD3
                (Z_GetTerrainMemory() / 1000) + 1,
                ((*addr_of!(s_Stats)).m_SizesPerTag[21] / 1000) + 1, // TAG_SND_RAWDATA
                (texMemSize / 1000) + 1,
                ((*addr_of!(s_Stats)).m_SizesPerTag[28] / 1000) + 1, // TAG_BINK
                (Z_GetMiscMemory() / 1000) + 1,
                (*addr_of!(s_Stats)).m_SizeFree,
                0)); // stat.dwAvailPhys would go here

        #[cfg(feature = "jk2mp")]
        {
            Sys_Log(b"memory-map.txt\0" as *const u8 as *const c_char,
                    va(b"HUNK: %d, THUNK: %d\n\0" as *const u8 as *const c_char,
                    ((*addr_of!(s_Stats)).m_SizesPerTag[12] / 1000) + 1, // TAG_HUNKALLOC
                    ((*addr_of!(s_Stats)).m_SizesPerTag[14] / 1000) + 1)); // TAG_TEMP_HUNKALLOC
        }
    }
}

fn Z_Stats_f() {
    unsafe {
        assert!(s_Initialized);
        // Display some memory usage summary information...

        Com_Printf(b"\nThe zone is using %d bytes (%.2fMB) in %d memory blocks\n\0" as *const u8 as *const c_char,
            (*addr_of!(s_Stats)).m_SizeAlloc,
            (*addr_of!(s_Stats)).m_SizeAlloc as f32 / 1024.0 / 1024.0,
            (*addr_of!(s_Stats)).m_CountAlloc);

        Com_Printf(b"Free memory is %d bytes (%.2fMB) in %d memory blocks\n\0" as *const u8 as *const c_char,
            (*addr_of!(s_Stats)).m_SizeFree,
            (*addr_of!(s_Stats)).m_SizeFree as f32 / 1024.0 / 1024.0,
            (*addr_of!(s_Stats)).m_CountFree);

        Com_Printf(b"The zone peaked at %d bytes (%.2fMB)\n\0" as *const u8 as *const c_char,
            (*addr_of!(s_Stats)).m_PeakAlloc,
            (*addr_of!(s_Stats)).m_PeakAlloc as f32 / 1024.0 / 1024.0);

        Com_Printf(b"The zone overhead is %d bytes (%.2fMB)\n\0" as *const u8 as *const c_char,
            (*addr_of!(s_Stats)).m_OverheadAlloc,
            (*addr_of!(s_Stats)).m_OverheadAlloc as f32 / 1024.0 / 1024.0);
    }
}

pub fn Z_Details_f() {
    unsafe {
        assert!(s_Initialized);
        // Display some tag specific information...

        Com_Printf(b"---------------------------------------------------------------------------\n\0" as *const u8 as *const c_char);
        Com_Printf(b"%20s %9s\n\0" as *const u8 as *const c_char, b"Zone Tag\0" as *const u8 as *const c_char, b"Bytes\0" as *const u8 as *const c_char);
        Com_Printf(b"%20s %9s\n\0" as *const u8 as *const c_char, b"--------\0" as *const u8 as *const c_char, b"-----\0" as *const u8 as *const c_char);
        for i in 0..64 { // TAG_COUNT = 64
            let iThisCount = (*addr_of!(s_Stats)).m_CountsPerTag[i];
            let iThisSize = (*addr_of!(s_Stats)).m_SizesPerTag[i];

            if iThisCount != 0 {
                let fSize: f32 = (iThisSize as f32) / 1024.0 / 1024.0;
                let iSize: c_int = fSize as c_int;
                let iRemainder: c_int = ((100.0 * (fSize - fSize.floor())) as c_int);
                Com_Printf(b"%d %9d (%2d.%02dMB) in %6d blocks (%9d average)\n\0" as *const u8 as *const c_char,
                    i, iThisSize, iSize, iRemainder, iThisCount, iThisSize / iThisCount);
            }
        }
        Com_Printf(b"---------------------------------------------------------------------------\n\0" as *const u8 as *const c_char);

        Z_Stats_f();
    }
}

pub fn Z_DumpMemMap_f() {
    unsafe {
        macro_rules! WRITECHAR {
            ($C:expr) => {{
                Sys_Log(b"memmap.txt\0" as *const u8 as *const c_char,
                        if $C == b'*' { b"*\0" } else if $C == b'+' { b"+\0" } else { b"-\0" } as *const u8 as *const c_char, 1);
                cur += 1024;
                counter += 1;
                if (counter) % 81 == 0 {
                    Sys_Log(b"memmap.txt\0" as *const u8 as *const c_char, b"\n\0" as *const u8 as *const c_char, 1);
                }
            }};
        }

        let mut cur = s_PoolBase as u32;
        let mut counter = 0;
        let mut fblock = addr_of!(s_FreeStart);
        while fblock != addr_of!(s_FreeEnd) {
            while (*fblock).m_Address > cur + 1024 {
                WRITECHAR!(b'*');
            }

            if (*fblock).m_Address > cur && (*fblock).m_Address < cur + 1024 {
                WRITECHAR!(b'+');
            }

            while (*fblock).m_Address + (*fblock).m_Size > cur + 1024 {
                WRITECHAR!(b'-');
            }

            if (*fblock).m_Address + (*fblock).m_Size > cur &&
                (*fblock).m_Address + (*fblock).m_Size < cur + 1024 {
                WRITECHAR!(b'+');
            }

            fblock = addr_of_mut!(s_FreeStart);
            let mut temp = fblock;
            for _ in 0..999 {
                if temp == addr_of!(s_FreeEnd) { break; }
                temp = (*temp).m_Next;
            }
        }

        Sys_Log(b"memmap.txt\0" as *const u8 as *const c_char, b"\n\0" as *const u8 as *const c_char);
    }
}

pub fn Z_DisplayLevelMemory(size: c_int, surf: c_int, block: c_int) {
    Z_DumpMemMap_f();

    //Yes, it should be divided by 1024, but I'm going for a safety margin
    //by rounding down.
    //Com_Printf("level memory used: %d KB\n", size / 1000);
    //Z_CompactStats(size, surf, block);
    Z_CompactStats();
}

#[cfg(target_os = "gamecube")]
extern "C" {
    fn R_SurfMramUsed(surface: *mut c_int, block: *mut c_int);
}

pub fn Z_DisplayLevelMemory_noargs() {
    #[cfg(target_os = "gamecube")]
    unsafe {
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
pub fn CopyString(in_str: *const c_char) -> *mut c_char {
    unsafe {
        #[repr(C)]
        struct ZoneSingleChar {
            header: ZoneHeader,
            #[cfg(debug_assertions)]
            start: ZoneDebugHeader,
            data: [c_char; 2],
            #[cfg(debug_assertions)]
            end: ZoneDebugFooter,
        }

        #[cfg(debug_assertions)]
        {
            static empty: ZoneSingleChar = ZoneSingleChar {
                header: 0 << 25 | 2, // TAG_STATIC << 25 | 2
                start: ZONE_MAGIC,
                data: [0, 0],
                end: ZONE_MAGIC as u8,
            };
            static numbers: [ZoneSingleChar; 10] = [
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'0' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'1' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'2' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'3' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'4' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'5' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'6' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'7' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'8' as c_char, 0], end: ZONE_MAGIC as u8 },
                ZoneSingleChar { header: 0 << 25 | 2, start: ZONE_MAGIC, data: [b'9' as c_char, 0], end: ZONE_MAGIC as u8 },
            ];
        }

        #[cfg(not(debug_assertions))]
        {
            static empty: ZoneSingleChar = ZoneSingleChar {
                header: 0 << 25 | 2, // TAG_STATIC << 25 | 2
                data: [0, 0],
            };
            static numbers: [ZoneSingleChar; 10] = [
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'0' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'1' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'2' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'3' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'4' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'5' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'6' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'7' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'8' as c_char, 0] },
                ZoneSingleChar { header: 0 << 25 | 2, data: [b'9' as c_char, 0] },
            ];
        }

        if *in_str == 0 {
            return empty.data.as_ptr() as *mut c_char;
        }
        else if *(in_str as *const u8).add(1) == 0 {
            if *in_str as u8 >= b'0' && *in_str as u8 <= b'9' {
                return numbers[(*in_str as u8 - b'0') as usize].data.as_ptr() as *mut c_char;
            }
        }

        let out = S_Malloc((strlen(in_str) + 1) as c_int) as *mut c_char;
        strcpy(out, in_str);

        out
    }
}

pub fn Com_TouchMemory() {
    // Stub function. Do nothing.
}

pub fn Z_IsFromZone(pvAddress: *mut c_void, eTag: memtag_t) -> qboolean {
    unsafe {
        assert!(s_Initialized);

        #[cfg(debug_assertions)]
        {
            let debug = (pvAddress as *mut ZoneDebugHeader).offset(-1);

            if *debug != ZONE_MAGIC {
                return 0; // qfalse
            }

            let pvAddress = debug as *mut c_void;
            let header = (pvAddress as *mut ZoneHeader).offset(-1);

            if Z_GetTag(header) != eTag {
                return 0; // qfalse
            }

            return Z_GetSize(header) as qboolean;
        }

        #[cfg(not(debug_assertions))]
        {
            let header = (pvAddress as *mut ZoneHeader).offset(-1);

            if Z_GetTag(header) != eTag {
                return 0; // qfalse
            }

            Z_GetSize(header) as qboolean
        }
    }
}

/*
   Hunk emulation

   The emulation is pretty bad right now, we just use two tags:
   TAG_HUNKALLOC and TAG_TEMP_HUNKALLOC, to represent the permanent and
   temporary sides of the hunk respectively. We should make the
   Hunk allocations tagged so we can do this better.
*/
#[cfg(feature = "jk2mp")]
pub fn Hunk_Clear() {
    Z_TagFree(14); // TAG_TEMP_HUNKALLOC
    Z_TagFree(12); // TAG_HUNKALLOC
}

#[cfg(feature = "jk2mp")]
pub fn Hunk_Alloc(size: c_int, preference: ha_pref) -> *mut c_void {
    Z_Malloc(size, 12, 1, 0) // TAG_HUNKALLOC, qtrue
}

pub fn Hunk_AllocateTempMemory(size: c_int) -> *mut c_void {
    Z_Malloc(size, 14, 1, 0) // TAG_TEMP_HUNKALLOC, qtrue
}

pub fn Hunk_FreeTempMemory(buf: *mut c_void) {
    Z_Free(buf);
}

#[cfg(feature = "jk2mp")]
pub fn Hunk_ClearTempMemory() {
    Z_TagFree(14); // TAG_TEMP_HUNKALLOC
}

pub fn Com_InitHunkMemory() {
}

pub fn Hunk_MemoryRemaining() -> c_int {
    0
}

pub fn Hunk_ClearToMark() {
}

pub fn Hunk_CheckMark() -> qboolean {
    0 // qfalse
}

pub fn Hunk_SetMark() {
}
