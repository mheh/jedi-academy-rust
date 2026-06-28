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

use core::ffi::{c_char, c_void, c_int};
use core::ptr;

/***********************************************
*
* WINDOWS/XBOX VERSION
*
* Build a translation table, CRC -> file name.  We have the memory.
*
************************************************/

#[cfg(target_os = "windows")]
use winapi::um::winnt::HANDLE;
#[cfg(target_os = "windows")]
use winapi::um::fileapi::{FindFirstFileA, FindNextFileA, FindClose};
#[cfg(target_os = "windows")]
use winapi::um::fileapi::WIN32_FIND_DATAA;
#[cfg(target_os = "windows")]
use winapi::um::synchapi::{CreateMutexA, WaitForSingleObject, ReleaseMutex};
#[cfg(target_os = "windows")]
use winapi::um::handleapi::CloseHandle;
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::FALSE;
#[cfg(target_os = "windows")]
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
#[cfg(target_os = "windows")]
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
#[cfg(target_os = "windows")]
const INFINITE: u32 = 0xFFFFFFFF;

// Stubs for Windows types on non-Windows platforms (for structural coherence)
#[cfg(not(target_os = "windows"))]
pub type HANDLE = *mut c_void;
#[cfg(not(target_os = "windows"))]
const INVALID_HANDLE_VALUE: HANDLE = ptr::null_mut();
#[cfg(not(target_os = "windows"))]
const INFINITE: u32 = 0xFFFFFFFF;

#[repr(C)]
pub struct FileInfo {
    pub name: *mut c_char,
    pub size: c_int,
}

// VVFixedMap stub - unimplemented local stub for structural coherence
pub struct VVFixedMap<T, K> {
    _phantom: core::marker::PhantomData<(T, K)>,
}

impl<T, K> VVFixedMap<T, K> {
    pub fn new(_capacity: c_int) -> Box<Self> {
        Box::new(VVFixedMap {
            _phantom: core::marker::PhantomData,
        })
    }
    pub fn Insert(&mut self, _info: T, _code: K) {}
    pub fn Find(&self, _code: K) -> Option<*mut T> {
        None
    }
    pub fn Pop(&mut self) -> Option<Box<T>> {
        None
    }
    pub fn Sort(&mut self) {}
}

static mut s_Files: *mut Box<VVFixedMap<FileInfo, u32>> = ptr::null_mut();
static mut buffer: *mut u8 = ptr::null_mut();

#[cfg(target_os = "windows")]
static mut s_Mutex: HANDLE = -1isize as HANDLE;
#[cfg(not(target_os = "windows"))]
static mut s_Mutex: HANDLE = ptr::null_mut();

extern "C" {
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn crc32(crc: u32, buf: *const u8, len: usize) -> u32;
    fn strlwr(str_: *mut c_char) -> *mut c_char;
    fn CopyString(str_: *const c_char) -> *mut c_char;
    fn Z_Malloc(size: usize, tag: c_int, clear: bool, alignment: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Sys_Cwd() -> *const c_char;
    fn FS_BuildOSPath(name: *const c_char) -> *mut c_char;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut libc::FILE;
    fn fclose(stream: *mut libc::FILE) -> c_int;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut libc::FILE) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut libc::FILE) -> usize;
}

const MAX_OSPATH: usize = 256;

fn _buildFileList(path: *const c_char, insert: bool, buildList: bool) -> c_int {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut count: c_int = 0;
        let mut spec: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

        // Look for all files
        Com_sprintf(spec.as_mut_ptr(), spec.len(), b"%s\\*.*\0".as_ptr() as *const c_char, path);

        let mut data: WIN32_FIND_DATAA = core::mem::zeroed();
        let h = FindFirstFileA(spec.as_ptr(), &mut data);
        while h != INVALID_HANDLE_VALUE {
            let mut full: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
            Com_sprintf(full.as_mut_ptr(), full.len(), b"%s\\%s\0".as_ptr() as *const c_char, path, data.cFileName.as_ptr());

            if (data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                // Directory -- lets go recursive
                if data.cFileName[0] != b'.' as c_char {
                    count += _buildFileList(full.as_ptr(), insert, buildList);
                }
            } else {
                if insert || buildList {
                    // Regular file -- add it to the table
                    strlwr(full.as_mut_ptr());
                    let code = crc32(0, full.as_ptr() as *const u8, strlen(full.as_ptr()));

                    let mut info: FileInfo = FileInfo {
                        name: CopyString(full.as_ptr()),
                        size: data.nFileSizeLow as c_int,
                    };

                    if insert {
                        if !s_Files.is_null() {
                            if let Some(ref mut map) = s_Files.as_mut() {
                                map.Insert(info, code);
                            }
                        }
                    }

                    if buildList {
                        // get the length of the filename
                        let len: usize = strlen(info.name) + 1;

                        // save the file code
                        *(buffer as *mut u32) = code;
                        buffer = buffer.add(core::mem::size_of::<u32>());

                        // save the name of the file
                        strcpy(buffer as *mut c_char, info.name);
                        buffer = buffer.add(len);

                        // save the size of the file
                        *(buffer as *mut c_int) = info.size;
                        buffer = buffer.add(core::mem::size_of::<c_int>());
                    }
                }

                count += 1;
            }

            // Continue the loop
            if FindNextFileA(h, &mut data) == 0 {
                FindClose(h);
                return count;
            }
        }
        count
    }

    #[cfg(not(target_os = "windows"))]
    {
        0
    }
}

fn _buildFileListFromSavedList() -> bool {
    // open the file up for reading
    unsafe {
        let in_ = fopen(b"d:\\xbx_filelist\0".as_ptr() as *const c_char, b"rb\0".as_ptr() as *const c_char);
        if in_.is_null() {
            return false;
        }

        // read in the number of files
        let mut count: c_int = 0;
        if fread(&mut count as *mut c_int as *mut c_void, core::mem::size_of::<c_int>(), 1, in_) == 0 {
            fclose(in_);
            return false;
        }

        // allocate memory for a temp buffer
        let bufferSize: usize = (count as usize) * (2 * core::mem::size_of::<c_int>() + MAX_OSPATH);
        let baseAddr = Z_Malloc(bufferSize, 0, true, 32) as *mut u8; // TAG_TEMP_WORKSPACE
        buffer = baseAddr;

        // read the rest of the file into a big buffer
        if fread(buffer as *mut c_void, bufferSize, 1, in_) == 0 {
            fclose(in_);
            Z_Free(baseAddr as *mut c_void);
            return false;
        }

        // allocate some memory for s_Files
        let new_map = Box::new(VVFixedMap::<FileInfo, u32>::new(count));
        s_Files = Box::into_raw(Box::new(new_map));

        // loop through all the files write out the codes
        let mut i: c_int = 0;
        while i < count {
            let mut info: FileInfo = FileInfo {
                name: ptr::null_mut(),
                size: 0,
            };
            let mut code: u32 = 0;

            // read the code for the file
            code = *(buffer as *const u32);
            buffer = buffer.add(core::mem::size_of::<u32>());

            // read the filename
            info.name = CopyString(buffer as *const c_char);
            buffer = buffer.add(strlen(info.name) + 1);

            // read the size of the file
            info.size = *(buffer as *const c_int);
            buffer = buffer.add(core::mem::size_of::<c_int>());

            // save the data
            if !s_Files.is_null() {
                if let Some(ref mut m) = s_Files.as_mut() {
                    m.Insert(info, code);
                }
            }

            i += 1;
        }

        fclose(in_);
        Z_Free(baseAddr as *mut c_void);
        return true;
    }
}

pub fn Sys_SaveFileCodes() -> bool {
    unsafe {
        // get the number of files
        let count: c_int = _buildFileList(Sys_Cwd(), false, false);

        // open a file for writing
        let out = fopen(b"d:\\xbx_filelist\0".as_ptr() as *const c_char, b"wb\0".as_ptr() as *const c_char);
        if out.is_null() {
            return false;
        }

        // allocate a buffer for writing
        let bufferSize: usize = core::mem::size_of::<c_int>() + ((count as usize) * (2 * core::mem::size_of::<c_int>() + MAX_OSPATH));
        let baseAddr = Z_Malloc(bufferSize, 0, true, 32) as *mut u8; // TAG_TEMP_WORKSPACE
        buffer = baseAddr;

        // write the number of files to the buffer
        *(baseAddr as *mut c_int) = count;
        buffer = baseAddr.add(core::mem::size_of::<c_int>());

        // fill up the rest of the buffer
        let ret = _buildFileList(Sys_Cwd(), false, true);

        if !ret {
            // there was a problem
            fclose(out);
            Z_Free(baseAddr as *mut c_void);
            return false;
        }

        // attempt to write out the data
        if fwrite(baseAddr as *const c_void, bufferSize, 1, out) == 0 {
            // there was a problem
            fclose(out);
            Z_Free(baseAddr as *mut c_void);
            return false;
        }

        // everything went ok
        fclose(out);
        Z_Free(baseAddr as *mut c_void);
        return true;
    }
}

pub fn Sys_InitFileCodes() {
    unsafe {
        // First: try to load an existing filecode cache
        let ret = _buildFileListFromSavedList();

        // if we had trouble building the list that way
        // we need to do it by searching the files
        if !ret {
            // There was no filelist cache, make one
            if !Sys_SaveFileCodes() {
                Com_Error(2, b"ERROR: Couldn't create filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
            }

            // Now re-read it
            if !_buildFileListFromSavedList() {
                Com_Error(2, b"ERROR: Couldn't re-read filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
            }
        }

        if !s_Files.is_null() {
            if let Some(ref mut m) = s_Files.as_mut() {
                m.Sort();
            }
        }

        // make it thread safe
        #[cfg(target_os = "windows")]
        {
            s_Mutex = CreateMutexA(ptr::null_mut(), FALSE, ptr::null());
        }
    }
}

pub fn Sys_ShutdownFileCodes() {
    unsafe {
        if !s_Files.is_null() {
            if let Some(ref mut m) = s_Files.as_mut() {
                loop {
                    if let Some(_boxed_info) = m.Pop() {
                        // info = Box::into_raw(boxed_info);
                        // Z_Free((*info).name as *mut c_void);
                        // (*info).name = ptr::null_mut();
                    } else {
                        break;
                    }
                }
            }
        }

        let raw_ptr = s_Files;
        if !raw_ptr.is_null() {
            let _ = Box::from_raw(raw_ptr);
        }
        s_Files = ptr::null_mut();

        #[cfg(target_os = "windows")]
        {
            CloseHandle(s_Mutex);
        }
    }
}

pub fn Sys_GetFileCode(name: *const c_char) -> c_int {
    unsafe {
        #[cfg(target_os = "windows")]
        {
            WaitForSingleObject(s_Mutex, INFINITE);
        }

        // Get system level path
        let osname = FS_BuildOSPath(name);

        // Generate hash for file name
        strlwr(osname);
        let code = crc32(0, osname as *const u8, strlen(osname));

        // Check if the file exists
        let found = if !s_Files.is_null() {
            if let Some(ref m) = s_Files.as_ref() {
                m.Find(code).is_some()
            } else {
                false
            }
        } else {
            false
        };

        if !found {
            #[cfg(target_os = "windows")]
            {
                ReleaseMutex(s_Mutex);
            }
            return -1;
        }

        #[cfg(target_os = "windows")]
        {
            ReleaseMutex(s_Mutex);
        }
        return code as c_int;
    }
}

pub fn Sys_GetFileCodeName(code: c_int) -> *const c_char {
    unsafe {
        #[cfg(target_os = "windows")]
        {
            WaitForSingleObject(s_Mutex, INFINITE);
        }

        let entry = if !s_Files.is_null() {
            if let Some(ref m) = s_Files.as_ref() {
                m.Find(code as u32)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(entry_ptr) = entry {
            #[cfg(target_os = "windows")]
            {
                ReleaseMutex(s_Mutex);
            }
            return (*entry_ptr).name;
        }

        #[cfg(target_os = "windows")]
        {
            ReleaseMutex(s_Mutex);
        }
        return ptr::null();
    }
}

pub fn Sys_GetFileCodeSize(code: c_int) -> c_int {
    unsafe {
        #[cfg(target_os = "windows")]
        {
            WaitForSingleObject(s_Mutex, INFINITE);
        }

        let entry = if !s_Files.is_null() {
            if let Some(ref m) = s_Files.as_ref() {
                m.Find(code as u32)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(entry_ptr) = entry {
            #[cfg(target_os = "windows")]
            {
                ReleaseMutex(s_Mutex);
            }
            return (*entry_ptr).size;
        }

        #[cfg(target_os = "windows")]
        {
            ReleaseMutex(s_Mutex);
        }
        return -1;
    }
}

// Quick function to re-scan for new files, update the filecode
// table, and dump the new one to disk
pub fn Sys_FilecodeScan_f() {
    unsafe {
        // Make an updated filecode cache
        if !Sys_SaveFileCodes() {
            Com_Error(2, b"ERROR: Couldn't create filecode cache\n\0".as_ptr() as *const c_char); // ERR_DROP
        }

        // Throw out our current list
        Sys_ShutdownFileCodes();

        // Re-init, which should use the new list we just made
        Sys_InitFileCodes();
    }
}
