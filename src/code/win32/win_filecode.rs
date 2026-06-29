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

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::ptr;

#[cfg(target_os = "windows")]
use winapi::um::winnt::HANDLE;
#[cfg(target_os = "windows")]
use winapi::um::fileapi::{FindFirstFileA, FindNextFileA, FindClose};
#[cfg(target_os = "windows")]
use winapi::um::synchapi::{CreateMutexA, WaitForSingleObject};
#[cfg(target_os = "windows")]
use winapi::um::handleapi::CloseHandle;
#[cfg(target_os = "windows")]
use winapi::um::fileapi::FILE_ATTRIBUTE_DIRECTORY;
#[cfg(target_os = "windows")]
use winapi::um::minwinbase::WIN32_FIND_DATAA;
#[cfg(target_os = "windows")]
use winapi::um::synchapi::ReleaseMutex;
#[cfg(target_os = "windows")]
const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

/***********************************************
*
* WINDOWS/XBOX VERSION
*
* Build a translation table, CRC -> file name.  We have the memory.
*
************************************************/

#[repr(C)]
struct FileInfo {
    name: *mut c_char,
    size: c_int,
}

// LOCAL STUB: VVFixedMap is a project-specific template type
// Declared as opaque for structural coherence
extern "C" {
    type VVFixedMap;
}

static mut s_Files: *mut VVFixedMap = ptr::null_mut();
static mut buffer: *mut u8 = ptr::null_mut();

static mut s_Mutex: HANDLE = INVALID_HANDLE_VALUE as HANDLE;

// External C functions
extern "C" {
    fn Com_sprintf(buf: *mut c_char, buf_size: usize, fmt: *const c_char, ...) -> c_int;
    fn crc32(crc: c_int, buf: *const u8, len: usize) -> c_int;
    fn CopyString(in_: *const c_char) -> *mut c_char;
    fn Z_Malloc(size: c_int, tag: c_int, zero: bool, alignment: c_int) -> *mut u8;
    fn Z_Free(ptr: *mut u8);
    fn strlwr(str_: *mut c_char) -> *mut c_char;
    fn FS_BuildOSPath(qpath: *const c_char) -> *mut c_char;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Sys_Cwd() -> *const c_char;

    // VVFixedMap methods (assuming C++ interface exposed via C)
    fn VVFixedMap_Insert(map: *mut VVFixedMap, info: *const FileInfo, code: c_int);
    fn VVFixedMap_Find(map: *mut VVFixedMap, code: c_int) -> *mut FileInfo;
    fn VVFixedMap_Pop(map: *mut VVFixedMap) -> *mut FileInfo;
    fn VVFixedMap_Sort(map: *mut VVFixedMap);
    fn VVFixedMap_New(capacity: c_int) -> *mut VVFixedMap;
    fn VVFixedMap_Delete(map: *mut VVFixedMap);
}

fn _buildFileList(path: *const c_char, insert: bool, buildList: bool) -> c_int {
    let mut data: WIN32_FIND_DATAA = unsafe { std::mem::zeroed() };
    let mut spec: [c_char; 260] = [0; 260]; // MAX_OSPATH
    let mut count = 0;

    // Look for all files
    unsafe {
        Com_sprintf(
            spec.as_mut_ptr(),
            spec.len(),
            b"%s\\*.*\0".as_ptr() as *const c_char,
            path,
        );
    }

    let mut h = unsafe { FindFirstFileA(spec.as_ptr(), &mut data) };
    while h != INVALID_HANDLE_VALUE {
        let mut full: [c_char; 260] = [0; 260]; // MAX_OSPATH
        unsafe {
            Com_sprintf(
                full.as_mut_ptr(),
                full.len(),
                b"%s\\%s\0".as_ptr() as *const c_char,
                path,
                data.cFileName.as_ptr(),
            );
        }

        if unsafe { data.dwFileAttributes } & FILE_ATTRIBUTE_DIRECTORY != 0 {
            // Directory -- lets go recursive
            if unsafe { data.cFileName[0] as u8 } != b'.' as u8 {
                count += _buildFileList(full.as_ptr(), insert, buildList);
            }
        } else {

            if insert || buildList {
                // Regular file -- add it to the table
                unsafe {
                    strlwr(full.as_mut_ptr());
                    let len = libc::strlen(full.as_ptr() as *const i8);
                    let code = crc32(0, full.as_ptr() as *const u8, len) as c_int;

                    let mut info: FileInfo = std::mem::zeroed();
                    info.name = CopyString(full.as_ptr());
                    info.size = data.nFileSizeLow as c_int;

                    if insert {
                        VVFixedMap_Insert(s_Files, &info, code);
                    }

                    if buildList {
                        // get the length of the filename
                        let len = libc::strlen(info.name as *const i8) + 1;

                        // save the file code
                        *(buffer as *mut c_int) = code;
                        buffer = buffer.add(std::mem::size_of::<c_int>());

                        // save the name of the file
                        libc::strcpy(buffer as *mut i8, info.name as *const i8);
                        buffer = buffer.add(len);

                        // save the size of the file
                        *(buffer as *mut c_int) = info.size;
                        buffer = buffer.add(std::mem::size_of::<c_int>());
                    }
                }
            }

            count += 1;
        }

        // Continue the loop
        if !unsafe { FindNextFileA(h, &mut data) } {
            unsafe { FindClose(h) };
            return count;
        }
    }
    count
}

fn _buildFileListFromSavedList() -> bool {
    // open the file up for reading
    let in_file = unsafe { libc::fopen(b"d:\\xbx_filelist\0".as_ptr() as *const i8, b"rb\0".as_ptr() as *const i8) };
    if in_file.is_null() {
        return false;
    }

    // read in the number of files
    let mut count = 0i32;
    if unsafe { libc::fread(&mut count as *mut i32 as *mut c_void, std::mem::size_of::<i32>(), 1, in_file) } == 0 {
        unsafe { libc::fclose(in_file) };
        return false;
    }

    // allocate memory for a temp buffer
    let buffer_size = (count as usize) * (2 * std::mem::size_of::<c_int>() + 260); // MAX_OSPATH
    let base_addr = unsafe { Z_Malloc(buffer_size as c_int, 0, true, 32) }; // TAG_TEMP_WORKSPACE
    unsafe { buffer = base_addr; }

    // read the rest of the file into a big buffer
    if unsafe { libc::fread(buffer as *mut c_void, buffer_size, 1, in_file) } == 0 {
        unsafe { libc::fclose(in_file) };
        unsafe { Z_Free(base_addr) };
        return false;
    }

    // allocate some memory for s_Files
    unsafe { s_Files = VVFixedMap_New(count); }

    // loop through all the files write out the codes
    for _i in 0..count {
        let mut info: FileInfo = unsafe { std::mem::zeroed() };
        let mut code = 0u32;

        // read the code for the file
        unsafe {
            code = *(buffer as *const c_int) as u32;
            buffer = buffer.add(std::mem::size_of::<c_int>());

            // read the filename
            info.name = CopyString(buffer as *const c_char);
            buffer = buffer.add(libc::strlen(info.name as *const i8) + 1);

            // read the size of the file
            info.size = *(buffer as *const c_int);
            buffer = buffer.add(std::mem::size_of::<c_int>());

            // save the data
            VVFixedMap_Insert(s_Files, &info, code as c_int);
        }
    }

    unsafe { libc::fclose(in_file) };
    unsafe { Z_Free(base_addr) };
    true
}

fn Sys_SaveFileCodes() -> bool {
    // get the number of files
    let mut count = 0;
    unsafe {
        let cwd = Sys_Cwd();
        count = _buildFileList(cwd, false, false);
    }

    // open a file for writing
    let out_file = unsafe { libc::fopen(b"d:\\xbx_filelist\0".as_ptr() as *const i8, b"wb\0".as_ptr() as *const i8) };
    if out_file.is_null() {
        return false;
    }

    // allocate a buffer for writing
    let buffer_size = std::mem::size_of::<c_int>() + (count as usize) * (2 * std::mem::size_of::<c_int>() + 260); // MAX_OSPATH
    let base_addr = unsafe { Z_Malloc(buffer_size as c_int, 0, true, 32) }; // TAG_TEMP_WORKSPACE
    unsafe { buffer = base_addr; }

    // write the number of files to the buffer
    unsafe {
        *(buffer as *mut c_int) = count;
        buffer = buffer.add(std::mem::size_of::<c_int>());
    }

    // fill up the rest of the buffer
    let cwd = unsafe { Sys_Cwd() };
    let ret = _buildFileList(cwd, false, true);

    if !ret {
        // there was a problem
        unsafe { libc::fclose(out_file) };
        unsafe { Z_Free(base_addr) };
        return false;
    }

    // attempt to write out the data
    if unsafe { libc::fwrite(base_addr as *const c_void, buffer_size, 1, out_file) } == 0 {
        // there was a problem
        unsafe { libc::fclose(out_file) };
        unsafe { Z_Free(base_addr) };
        return false;
    }

    // everything went ok
    unsafe { libc::fclose(out_file) };
    unsafe { Z_Free(base_addr) };
    true
}

fn Sys_InitFileCodes() {
    // First: try to load an existing filecode cache
    let ret = _buildFileListFromSavedList();

    // if we had trouble building the list that way
    // we need to do it by searching the files
    if !ret {
        // There was no filelist cache, make one
        if !Sys_SaveFileCodes() {
            unsafe {
                Com_Error(1, b"ERROR: Couldn't create filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
            }
        }

        // Now re-read it
        if !_buildFileListFromSavedList() {
            unsafe {
                Com_Error(1, b"ERROR: Couldn't re-read filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
            }
        }
    }
    unsafe { VVFixedMap_Sort(s_Files); }

    // make it thread safe
    unsafe { s_Mutex = CreateMutexA(ptr::null_mut(), 0, ptr::null()); }
}

fn Sys_ShutdownFileCodes() {
    unsafe {
        let mut info = VVFixedMap_Pop(s_Files);
        while !info.is_null() {
            Z_Free((*info).name as *mut u8);
            (*info).name = ptr::null_mut();
            info = VVFixedMap_Pop(s_Files);
        }

        VVFixedMap_Delete(s_Files);
        s_Files = ptr::null_mut();

        CloseHandle(s_Mutex);
    }
}

fn Sys_GetFileCode(name: *const c_char) -> c_int {
    unsafe {
        WaitForSingleObject(s_Mutex, 0xFFFFFFFF); // INFINITE

        // Get system level path
        let osname = FS_BuildOSPath(name);

        // Generate hash for file name
        strlwr(osname);
        let len = libc::strlen(osname);
        let code = crc32(0, osname as *const u8, len);

        // Check if the file exists
        if VVFixedMap_Find(s_Files, code).is_null() {
            ReleaseMutex(s_Mutex);
            return -1;
        }

        ReleaseMutex(s_Mutex);
        code
    }
}

fn Sys_GetFileCodeName(code: c_int) -> *const c_char {
    unsafe {
        WaitForSingleObject(s_Mutex, 0xFFFFFFFF); // INFINITE

        let entry = VVFixedMap_Find(s_Files, code);
        if !entry.is_null() {
            ReleaseMutex(s_Mutex);
            return (*entry).name;
        }

        ReleaseMutex(s_Mutex);
        ptr::null()
    }
}

fn Sys_GetFileCodeSize(code: c_int) -> c_int {
    unsafe {
        WaitForSingleObject(s_Mutex, 0xFFFFFFFF); // INFINITE

        let entry = VVFixedMap_Find(s_Files, code);
        if !entry.is_null() {
            ReleaseMutex(s_Mutex);
            return (*entry).size;
        }

        ReleaseMutex(s_Mutex);
        -1
    }
}

// Quick function to re-scan for new files, update the filecode
// table, and dump the new one to disk
fn Sys_FilecodeScan_f() {
    // Make an updated filecode cache
    if !Sys_SaveFileCodes() {
        unsafe {
            Com_Error(1, b"ERROR: Couldn't create filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
        }
    }

    // Throw out our current list
    Sys_ShutdownFileCodes();

    // Re-init, which should use the new list we just made
    Sys_InitFileCodes();
}
