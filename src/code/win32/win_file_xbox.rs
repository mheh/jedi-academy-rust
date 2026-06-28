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

use core::ffi::c_void;

// Windows API constants and types
type HANDLE = *mut c_void;
type DWORD = core::ffi::c_ulong;
type BOOL = core::ffi::c_int;

const GENERIC_READ: DWORD = 0x80000000;
const GENERIC_WRITE: DWORD = 0x40000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const OPEN_EXISTING: DWORD = 2;
const OPEN_ALWAYS: DWORD = 4;
const FILE_ATTRIBUTE_NORMAL: DWORD = 0x00000080;
const FILE_FLAG_NO_BUFFERING: DWORD = 0x20000000;
const INVALID_HANDLE_VALUE: HANDLE = -1 as isize as *mut c_void;

const SEEK_CUR: core::ffi::c_int = 1;
const SEEK_END: core::ffi::c_int = 2;
const SEEK_SET: core::ffi::c_int = 0;

const FILE_BEGIN: DWORD = 0;
const FILE_CURRENT: DWORD = 1;
const FILE_END: DWORD = 2;

extern "C" {
    fn CreateFile(
        lpFileName: *const core::ffi::c_char,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    fn CloseHandle(hObject: HANDLE) -> BOOL;

    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: *mut c_void,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: *const c_void,
    ) -> BOOL;

    fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *const c_void,
    ) -> BOOL;

    fn SetFilePointer(
        hFile: HANDLE,
        lDistanceToMove: core::ffi::c_long,
        lpDistanceToMoveHigh: *mut core::ffi::c_long,
        dwMoveMethod: DWORD,
    ) -> DWORD;

    fn SetEndOfFile(hFile: HANDLE) -> BOOL;
}

#[repr(C)]
struct FileTable {
    m_bUsed: bool,
    m_bErrorsFatal: bool,
    m_Handle: HANDLE,
}

static mut s_FileTable: *mut FileTable = core::ptr::null_mut();
const WF_MAX_OPEN_FILES: core::ffi::c_int = 8;

pub fn WF_Init() {
    unsafe {
        assert!(s_FileTable.is_null());

        let table_box: Box<[FileTable; 8]> = Box::new([
            FileTable {
                m_bUsed: false,
                m_bErrorsFatal: false,
                m_Handle: core::ptr::null_mut(),
            };
            8
        ]);
        s_FileTable = Box::into_raw(table_box) as *mut FileTable;

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

        let _ = Box::from_raw(s_FileTable);
        s_FileTable = core::ptr::null_mut();
    }
}

fn WF_GetFreeHandle() -> core::ffi::c_int {
    unsafe {
        for i in 0..WF_MAX_OPEN_FILES {
            if !(*s_FileTable.add(i as usize)).m_bUsed {
                return i;
            }
        }
    }

    -1
}

pub fn WF_Open(name: *const core::ffi::c_char, read: bool, aligned: bool) -> core::ffi::c_int {
    let handle = WF_GetFreeHandle();
    if handle == -1 {
        return -1;
    }

    unsafe {
        (*s_FileTable.add(handle as usize)).m_Handle = CreateFile(
            name,
            if read {
                GENERIC_READ
            } else {
                GENERIC_READ | GENERIC_WRITE
            },
            FILE_SHARE_READ,
            core::ptr::null(),
            if read { OPEN_EXISTING } else { OPEN_ALWAYS },
            FILE_ATTRIBUTE_NORMAL | (if aligned { FILE_FLAG_NO_BUFFERING } else { 0 }),
            core::ptr::null_mut(),
        );

        if (*s_FileTable.add(handle as usize)).m_Handle != INVALID_HANDLE_VALUE {
            (*s_FileTable.add(handle as usize)).m_bUsed = true;

            // errors are fatal on game partition
            (*s_FileTable.add(handle as usize)).m_bErrorsFatal =
                (*name == b'D' as core::ffi::c_char) || (*name == b'd' as core::ffi::c_char);

            return handle;
        }
    }

    -1
}

pub fn WF_Close(handle: core::ffi::c_int) {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        CloseHandle((*s_FileTable.add(handle as usize)).m_Handle);
        (*s_FileTable.add(handle as usize)).m_bUsed = false;
    }
}

pub fn WF_Read(buffer: *mut c_void, len: core::ffi::c_int, handle: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        let mut bytes: DWORD = 0;
        if ReadFile(
            (*s_FileTable.add(handle as usize)).m_Handle,
            buffer,
            len as DWORD,
            &mut bytes,
            core::ptr::null(),
        ) == 0
            && (*s_FileTable.add(handle as usize)).m_bErrorsFatal
        {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                assert!(false);
            }
            /*
            extern void ERR_DiscFail(bool);
            ERR_DiscFail(false);
            */
        }

        bytes as core::ffi::c_int
    }
}

pub fn WF_Write(buffer: *const c_void, len: core::ffi::c_int, handle: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        let mut bytes: DWORD = 0;
        WriteFile(
            (*s_FileTable.add(handle as usize)).m_Handle,
            buffer,
            len as DWORD,
            &mut bytes,
            core::ptr::null(),
        );
        bytes as core::ffi::c_int
    }
}

pub fn WF_Seek(offset: core::ffi::c_int, mut origin: core::ffi::c_int, handle: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        match origin {
            SEEK_CUR => {
                origin = FILE_CURRENT as core::ffi::c_int;
            }
            SEEK_END => {
                origin = FILE_END as core::ffi::c_int;
            }
            SEEK_SET => {
                origin = FILE_BEGIN as core::ffi::c_int;
            }
            _ => {
                assert!(false);
            }
        }

        let fp_result = SetFilePointer((*s_FileTable.add(handle as usize)).m_Handle, offset as core::ffi::c_long, core::ptr::null_mut(), origin as DWORD);
        (fp_result < (0 as DWORD)) as core::ffi::c_int
    }
}

pub fn WF_Tell(handle: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        SetFilePointer((*s_FileTable.add(handle as usize)).m_Handle, 0, core::ptr::null_mut(), FILE_CURRENT) as core::ffi::c_int
    }
}

pub fn WF_Resize(size: core::ffi::c_int, handle: core::ffi::c_int) -> core::ffi::c_int {
    unsafe {
        assert!(handle >= 0 && handle < WF_MAX_OPEN_FILES && (*s_FileTable.add(handle as usize)).m_bUsed);

        SetFilePointer((*s_FileTable.add(handle as usize)).m_Handle, size as core::ffi::c_long, core::ptr::null_mut(), FILE_BEGIN);
        SetEndOfFile((*s_FileTable.add(handle as usize)).m_Handle)
    }
}
