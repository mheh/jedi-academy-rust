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

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::mem;
use core::ptr;

// Windows API types and constants
type HANDLE = *mut c_void;

const INVALID_HANDLE_VALUE: HANDLE = -1 as isize as *mut c_void;

const FILE_SHARE_READ: c_int = 1;
const GENERIC_READ: c_int = 0x80000000_u32 as c_int;
const GENERIC_WRITE: c_int = 0x40000000_u32 as c_int;
const OPEN_EXISTING: c_int = 3;
const OPEN_ALWAYS: c_int = 4;
const FILE_ATTRIBUTE_NORMAL: c_int = 128;
const FILE_FLAG_NO_BUFFERING: c_int = 0x20000000_u32 as c_int;
const FILE_CURRENT: c_int = 1;
const FILE_END: c_int = 2;
const FILE_BEGIN: c_int = 0;

const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;
const SEEK_SET: c_int = 0;

// Windows API functions
extern "C" {
    fn CreateFile(
        lpFileName: *const c_char,
        dwDesiredAccess: c_int,
        dwShareMode: c_int,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: c_int,
        dwFlagsAndAttributes: c_int,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    fn CloseHandle(hObject: HANDLE) -> c_int;

    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: *mut c_void,
        nNumberOfBytesToRead: c_int,
        lpNumberOfBytesRead: *mut c_int,
        lpOverlapped: *mut c_void,
    ) -> c_int;

    fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: c_int,
        lpNumberOfBytesWritten: *mut c_int,
        lpOverlapped: *mut c_void,
    ) -> c_int;

    fn SetFilePointer(
        hFile: HANDLE,
        lDistanceToMove: c_int,
        lpDistanceToMoveHigh: *mut c_int,
        dwMoveMethod: c_int,
    ) -> c_int;

    fn SetEndOfFile(hFile: HANDLE) -> c_int;
}

type wfhandle_t = c_int;

#[repr(C)]
struct FileTable {
    m_bUsed: bool,
    m_bErrorsFatal: bool,
    m_Handle: HANDLE,
}

static mut s_FileTable: *mut FileTable = ptr::null_mut();
const WF_MAX_OPEN_FILES: c_int = 8;

pub fn WF_Init() {
    unsafe {
        assert!(s_FileTable.is_null());

        let mut vec = vec![
            FileTable {
                m_bUsed: false,
                m_bErrorsFatal: false,
                m_Handle: ptr::null_mut(),
            };
            WF_MAX_OPEN_FILES as usize
        ];
        s_FileTable = vec.as_mut_ptr();
        mem::forget(vec);

        for i in 0..WF_MAX_OPEN_FILES {
            (*s_FileTable.add(i as usize)).m_bUsed = false;
        }
    }
}

pub fn WF_Shutdown() {
    unsafe {
        assert!(!s_FileTable.is_null());

        for i in 0..WF_MAX_OPEN_FILES {
            if (*s_FileTable.add(i as usize)).m_bUsed {
                WF_Close(i);
            }
        }

        let _ =
            Vec::from_raw_parts(s_FileTable, WF_MAX_OPEN_FILES as usize, WF_MAX_OPEN_FILES as usize);
        s_FileTable = ptr::null_mut();
    }
}

fn WF_GetFreeHandle() -> wfhandle_t {
    unsafe {
        for i in 0..WF_MAX_OPEN_FILES {
            if !(*s_FileTable.add(i as usize)).m_bUsed {
                return i;
            }
        }
    }

    return -1;
}

pub fn WF_Open(name: *const c_char, read: bool, aligned: bool) -> c_int {
    let handle = WF_GetFreeHandle();
    if handle == -1 {
        return -1;
    }

    unsafe {
        (*s_FileTable.add(handle as usize)).m_Handle = CreateFile(
            name,
            if read { GENERIC_READ } else { GENERIC_READ | GENERIC_WRITE },
            FILE_SHARE_READ,
            ptr::null_mut(),
            if read { OPEN_EXISTING } else { OPEN_ALWAYS },
            FILE_ATTRIBUTE_NORMAL | (if aligned { FILE_FLAG_NO_BUFFERING } else { 0 }),
            ptr::null_mut(),
        );

        if (*s_FileTable.add(handle as usize)).m_Handle != INVALID_HANDLE_VALUE {
            (*s_FileTable.add(handle as usize)).m_bUsed = true;

            // errors are fatal on game partition
            (*s_FileTable.add(handle as usize)).m_bErrorsFatal =
                *name == b'D' as c_char || *name == b'd' as c_char;

            return handle;
        }
    }

    return -1;
}

pub fn WF_Close(handle: wfhandle_t) {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        CloseHandle((*s_FileTable.add(handle as usize)).m_Handle);
        (*s_FileTable.add(handle as usize)).m_bUsed = false;
    }
}

pub fn WF_Read(buffer: *mut c_void, len: c_int, handle: wfhandle_t) -> c_int {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        let mut bytes: c_int = 0;
        if (ReadFile(
            (*s_FileTable.add(handle as usize)).m_Handle,
            buffer,
            len,
            &mut bytes,
            ptr::null_mut(),
        ) == 0)
            && (*s_FileTable.add(handle as usize)).m_bErrorsFatal
        {
            // VVFIXME
            // extern void ERR_DiscFail(bool);
            // ERR_DiscFail(false);
            assert!(false);
        }

        return bytes;
    }
}

pub fn WF_Write(buffer: *const c_void, len: c_int, handle: wfhandle_t) -> c_int {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        let mut bytes: c_int = 0;
        WriteFile(
            (*s_FileTable.add(handle as usize)).m_Handle,
            buffer,
            len,
            &mut bytes,
            ptr::null_mut(),
        );
        return bytes;
    }
}

pub fn WF_Seek(offset: c_int, mut origin: c_int, handle: wfhandle_t) -> c_int {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        match origin {
            SEEK_CUR => origin = FILE_CURRENT,
            SEEK_END => origin = FILE_END,
            SEEK_SET => origin = FILE_BEGIN,
            _ => assert!(false),
        }

        return (SetFilePointer((*s_FileTable.add(handle as usize)).m_Handle, offset, ptr::null_mut(), origin)
            < 0) as c_int;
    }
}

pub fn WF_Tell(handle: wfhandle_t) -> c_int {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        return SetFilePointer(
            (*s_FileTable.add(handle as usize)).m_Handle,
            0,
            ptr::null_mut(),
            FILE_CURRENT,
        );
    }
}

pub fn WF_Resize(size: c_int, handle: wfhandle_t) -> c_int {
    unsafe {
        assert!(
            handle >= 0
                && handle < WF_MAX_OPEN_FILES
                && (*s_FileTable.add(handle as usize)).m_bUsed
        );

        SetFilePointer(
            (*s_FileTable.add(handle as usize)).m_Handle,
            size,
            ptr::null_mut(),
            FILE_BEGIN,
        );
        return SetEndOfFile((*s_FileTable.add(handle as usize)).m_Handle);
    }
}
