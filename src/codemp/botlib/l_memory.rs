
/*****************************************************************************
 * name:		l_memory.c
 *
 * desc:		memory allocation
 *
 * $Archive: /MissionPack/code/botlib/l_memory.c $
 * $Author: Ttimo $
 * $Revision: 6 $
 * $Modtime: 4/22/01 8:52a $
 * $Date: 4/22/01 8:52a $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void};

//#define MEMDEBUG
//#define MEMORYMANEGER

const MEM_ID: u32 = 0x12345678;
const HUNK_ID: u32 = 0x87654321;

pub static mut allocatedmemory: c_int = 0;
pub static mut totalmemorysize: c_int = 0;
pub static mut numblocks: c_int = 0;

#[repr(C)]
pub struct memoryblock_s {
    pub id: u32,
    pub ptr: *mut c_void,
    pub size: c_int,
    // #ifdef MEMDEBUG
    // pub label: *mut c_char,
    // pub file: *mut c_char,
    // pub line: c_int,
    // #endif //MEMDEBUG
    pub prev: *mut memoryblock_s,
    pub next: *mut memoryblock_s,
}

pub type memoryblock_t = memoryblock_s;

pub static mut memory: *mut memoryblock_t = core::ptr::null_mut();

// External bot import interface and other engine functions
extern "C" {
    pub static mut botimport: botimport_t;

    pub fn Com_Memset(ptr: *mut c_void, value: c_int, size: usize);
    pub fn Log_Write(format: *const c_char, ...);
}

#[repr(C)]
pub struct botimport_t {
    pub GetMemory: Option<extern "C" fn(usize) -> *mut c_void>,
    pub FreeMemory: Option<extern "C" fn(*mut c_void)>,
    pub HunkAlloc: Option<extern "C" fn(usize) -> *mut c_void>,
    pub AvailableMemory: Option<extern "C" fn() -> c_int>,
    pub Print: Option<extern "C" fn(c_int, *const c_char, ...)>,
}

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn LinkMemoryBlock(block: *mut memoryblock_t) {
    (*block).prev = core::ptr::null_mut();
    (*block).next = memory;
    if !memory.is_null() {
        (*memory).prev = block;
    }
    memory = block;
} //end of the function LinkMemoryBlock

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn UnlinkMemoryBlock(block: *mut memoryblock_t) {
    if !(*block).prev.is_null() {
        (*(*block).prev).next = (*block).next;
    } else {
        memory = (*block).next;
    }
    if !(*block).next.is_null() {
        (*(*block).next).prev = (*block).prev;
    }
} //end of the function UnlinkMemoryBlock

#[cfg(all(feature = "MEMORYMANEGER", feature = "MEMDEBUG"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = botimport.GetMemory.unwrap()(size + core::mem::size_of::<memoryblock_t>());
    let block = ptr as *mut memoryblock_t;
    (*block).id = MEM_ID;
    (*block).ptr = (ptr as *mut u8).add(core::mem::size_of::<memoryblock_t>()) as *mut c_void;
    (*block).size = (size + core::mem::size_of::<memoryblock_t>()) as c_int;
    // #ifdef MEMDEBUG
    // (*block).label = label;
    // (*block).file = file;
    // (*block).line = line;
    // #endif //MEMDEBUG
    LinkMemoryBlock(block);
    allocatedmemory += (*block).size;
    totalmemorysize += (*block).size + core::mem::size_of::<memoryblock_t>() as c_int;
    numblocks += 1;
    (*block).ptr
} //end of the function GetMemoryDebug

#[cfg(all(feature = "MEMORYMANEGER", not(feature = "MEMDEBUG")))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetMemory(size: usize) -> *mut c_void {
    let ptr = botimport.GetMemory.unwrap()(size + core::mem::size_of::<memoryblock_t>());
    let block = ptr as *mut memoryblock_t;
    (*block).id = MEM_ID;
    (*block).ptr = (ptr as *mut u8).add(core::mem::size_of::<memoryblock_t>()) as *mut c_void;
    (*block).size = (size + core::mem::size_of::<memoryblock_t>()) as c_int;
    LinkMemoryBlock(block);
    allocatedmemory += (*block).size;
    totalmemorysize += (*block).size + core::mem::size_of::<memoryblock_t>() as c_int;
    numblocks += 1;
    (*block).ptr
} //end of the function GetMemory

#[cfg(all(feature = "MEMORYMANEGER", feature = "MEMDEBUG"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetClearedMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = GetMemoryDebug(size, label, file, line);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedMemory

#[cfg(all(feature = "MEMORYMANEGER", not(feature = "MEMDEBUG")))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetClearedMemory(size: usize) -> *mut c_void {
    let ptr = GetMemory(size);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedMemory

#[cfg(all(feature = "MEMORYMANEGER", feature = "MEMDEBUG"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetHunkMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = botimport.HunkAlloc.unwrap()(size + core::mem::size_of::<memoryblock_t>());
    let block = ptr as *mut memoryblock_t;
    (*block).id = HUNK_ID;
    (*block).ptr = (ptr as *mut u8).add(core::mem::size_of::<memoryblock_t>()) as *mut c_void;
    (*block).size = (size + core::mem::size_of::<memoryblock_t>()) as c_int;
    // #ifdef MEMDEBUG
    // (*block).label = label;
    // (*block).file = file;
    // (*block).line = line;
    // #endif //MEMDEBUG
    LinkMemoryBlock(block);
    allocatedmemory += (*block).size;
    totalmemorysize += (*block).size + core::mem::size_of::<memoryblock_t>() as c_int;
    numblocks += 1;
    (*block).ptr
} //end of the function GetHunkMemoryDebug

#[cfg(all(feature = "MEMORYMANEGER", not(feature = "MEMDEBUG")))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetHunkMemory(size: usize) -> *mut c_void {
    let ptr = botimport.HunkAlloc.unwrap()(size + core::mem::size_of::<memoryblock_t>());
    let block = ptr as *mut memoryblock_t;
    (*block).id = HUNK_ID;
    (*block).ptr = (ptr as *mut u8).add(core::mem::size_of::<memoryblock_t>()) as *mut c_void;
    (*block).size = (size + core::mem::size_of::<memoryblock_t>()) as c_int;
    LinkMemoryBlock(block);
    allocatedmemory += (*block).size;
    totalmemorysize += (*block).size + core::mem::size_of::<memoryblock_t>() as c_int;
    numblocks += 1;
    (*block).ptr
} //end of the function GetHunkMemory

#[cfg(all(feature = "MEMORYMANEGER", feature = "MEMDEBUG"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetClearedHunkMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = GetHunkMemoryDebug(size, label, file, line);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedHunkMemory

#[cfg(all(feature = "MEMORYMANEGER", not(feature = "MEMDEBUG")))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn GetClearedHunkMemory(size: usize) -> *mut c_void {
    let ptr = GetHunkMemory(size);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedHunkMemory

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BlockFromPointer(ptr: *mut c_void, str_msg: *const c_char) -> *mut memoryblock_t {
    if ptr.is_null() {
        // #ifdef MEMDEBUG
        //char *crash = (char *) NULL;
        //crash[0] = 1;
        botimport.Print.unwrap()(2, b"%s: NULL pointer\n\0".as_ptr() as *const c_char, str_msg); // PRT_FATAL = 2
        // #endif // MEMDEBUG
        return core::ptr::null_mut();
    }
    let block = (ptr as *mut u8).sub(core::mem::size_of::<memoryblock_t>()) as *mut memoryblock_t;
    if (*block).id != MEM_ID && (*block).id != HUNK_ID {
        botimport.Print.unwrap()(2, b"%s: invalid memory block\n\0".as_ptr() as *const c_char, str_msg);
        return core::ptr::null_mut();
    }
    if (*block).ptr != ptr {
        botimport.Print.unwrap()(2, b"%s: memory block pointer invalid\n\0".as_ptr() as *const c_char, str_msg);
        return core::ptr::null_mut();
    }
    block
} //end of the function BlockFromPointer

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn FreeMemory(ptr: *mut c_void) {
    let block = BlockFromPointer(ptr, b"FreeMemory\0".as_ptr() as *const c_char);
    if block.is_null() {
        return;
    }
    UnlinkMemoryBlock(block);
    allocatedmemory -= (*block).size;
    totalmemorysize -= (*block).size + core::mem::size_of::<memoryblock_t>() as c_int;
    numblocks -= 1;
    //
    if (*block).id == MEM_ID {
        botimport.FreeMemory.unwrap()(block as *mut c_void);
    }
} //end of the function FreeMemory

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AvailableMemory() -> c_int {
    unsafe { botimport.AvailableMemory.unwrap()() }
} //end of the function AvailableMemory

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn MemoryByteSize(ptr: *mut c_void) -> c_int {
    let block = BlockFromPointer(ptr, b"MemoryByteSize\0".as_ptr() as *const c_char);
    if block.is_null() {
        return 0;
    }
    (*block).size
} //end of the function MemoryByteSize

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn PrintUsedMemorySize() {
    botimport.Print.unwrap()(1, b"total allocated memory: %d KB\n\0".as_ptr() as *const c_char, allocatedmemory >> 10);
    botimport.Print.unwrap()(1, b"total botlib memory: %d KB\n\0".as_ptr() as *const c_char, totalmemorysize >> 10);
    botimport.Print.unwrap()(1, b"total memory blocks: %d\n\0".as_ptr() as *const c_char, numblocks);
} //end of the function PrintUsedMemorySize

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn PrintMemoryLabels() {
    let mut block: *mut memoryblock_t = memory;
    let mut i: c_int = 0;

    PrintUsedMemorySize();
    i = 0;
    Log_Write(b"============= Botlib memory log ==============\r\n\0".as_ptr() as *const c_char);
    Log_Write(b"\r\n\0".as_ptr() as *const c_char);
    while !block.is_null() {
        // #ifdef MEMDEBUG
        // if (block->id == HUNK_ID)
        // {
        //     Log_Write("%6d, hunk %p, %8d: %24s line %6d: %s\r\n", i, block->ptr, block->size, block->file, block->line, block->label);
        // } //end if
        // else
        // {
        //     Log_Write("%6d,      %p, %8d: %24s line %6d: %s\r\n", i, block->ptr, block->size, block->file, block->line, block->label);
        // } //end else
        // #endif //MEMDEBUG
        i += 1;
        block = (*block).next;
    }
} //end of the function PrintMemoryLabels

#[cfg(feature = "MEMORYMANEGER")]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn DumpMemory() {
    let mut block: *mut memoryblock_t = memory;

    while !block.is_null() {
        let ptr = (*block).ptr;
        block = memory;
        FreeMemory(ptr);
    }
    totalmemorysize = 0;
    allocatedmemory = 0;
} //end of the function DumpMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "MEMDEBUG")]
pub unsafe extern "C" fn GetMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = botimport.GetMemory.unwrap()(size + core::mem::size_of::<u32>());
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let memid = ptr as *mut u32;
    *memid = MEM_ID;
    (ptr as *mut u8).add(core::mem::size_of::<u32>()) as *mut c_void
} //end of the function GetMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
#[cfg(not(feature = "MEMDEBUG"))]
pub unsafe extern "C" fn GetMemory(size: usize) -> *mut c_void {
    let ptr = botimport.GetMemory.unwrap()(size + core::mem::size_of::<u32>());
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let memid = ptr as *mut u32;
    *memid = MEM_ID;
    (ptr as *mut u8).add(core::mem::size_of::<u32>()) as *mut c_void
} //end of the function GetMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "MEMDEBUG")]
pub unsafe extern "C" fn GetClearedMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = GetMemoryDebug(size, label, file, line);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
#[cfg(not(feature = "MEMDEBUG"))]
pub unsafe extern "C" fn GetClearedMemory(size: usize) -> *mut c_void {
    let ptr = GetMemory(size);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "MEMDEBUG")]
pub unsafe extern "C" fn GetHunkMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = botimport.HunkAlloc.unwrap()(size + core::mem::size_of::<u32>());
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let memid = ptr as *mut u32;
    *memid = HUNK_ID;
    (ptr as *mut u8).add(core::mem::size_of::<u32>()) as *mut c_void
} //end of the function GetHunkMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
#[cfg(not(feature = "MEMDEBUG"))]
pub unsafe extern "C" fn GetHunkMemory(size: usize) -> *mut c_void {
    let ptr = botimport.HunkAlloc.unwrap()(size + core::mem::size_of::<u32>());
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let memid = ptr as *mut u32;
    *memid = HUNK_ID;
    (ptr as *mut u8).add(core::mem::size_of::<u32>()) as *mut c_void
} //end of the function GetHunkMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "MEMDEBUG")]
pub unsafe extern "C" fn GetClearedHunkMemoryDebug(
    size: usize,
    label: *mut c_char,
    file: *mut c_char,
    line: c_int,
) -> *mut c_void {
    let ptr = GetHunkMemoryDebug(size, label, file, line);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedHunkMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
#[cfg(not(feature = "MEMDEBUG"))]
pub unsafe extern "C" fn GetClearedHunkMemory(size: usize) -> *mut c_void {
    let ptr = GetHunkMemory(size);
    Com_Memset(ptr, 0, size);
    ptr
} //end of the function GetClearedHunkMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn FreeMemory(ptr: *mut c_void) {
    let memid = (ptr as *mut u8).sub(core::mem::size_of::<u32>()) as *mut u32;

    if *memid == MEM_ID {
        botimport.FreeMemory.unwrap()(memid as *mut c_void);
    }
} //end of the function FreeMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AvailableMemory() -> c_int {
    unsafe { botimport.AvailableMemory.unwrap()() }
} //end of the function AvailableMemory

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn PrintUsedMemorySize() {
} //end of the function PrintUsedMemorySize

#[cfg(not(feature = "MEMORYMANEGER"))]
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn PrintMemoryLabels() {
} //end of the function PrintMemoryLabels
