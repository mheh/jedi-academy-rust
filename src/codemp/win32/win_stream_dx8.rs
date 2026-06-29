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
use std::collections::VecDeque;

// Windows API types
type HANDLE = *mut c_void;
type DWORD = u32;
type LPVOID = *mut c_void;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as *mut c_void;
const INFINITE: DWORD = 0xffffffff;
const FILE_BEGIN: DWORD = 0;
const GENERIC_READ: DWORD = 0x80000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const OPEN_EXISTING: DWORD = 3;

const STREAM_SLOW_READ: i32 = 0;

const STREAM_MAX_OPEN: usize = 48;

type streamHandle_t = usize;

// External engine functions and symbols
extern "C" {
    fn Z_SetNewDeleteTemporary(temporary: bool);
    fn Sys_GetFileCodeName(code: c_int) -> *const c_char;
    fn Sys_GetFileCodeSize(code: c_int) -> c_int;
    static mut Sys_FileStreamMutex: HANDLE;
    fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> LPVOID;
    fn Z_Free(ptr: LPVOID);
}

// Windows API function stubs
extern "system" {
    fn CreateFile(
        lpFileName: *const c_char,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    fn SetFilePointer(
        hFile: HANDLE,
        lDistanceToMove: c_int,
        lpDistanceToMoveHigh: *mut c_int,
        dwMoveMethod: DWORD,
    ) -> DWORD;

    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: LPVOID,
    ) -> c_int;

    fn CreateMutex(
        lpMutexAttributes: LPVOID,
        bInitialOwner: c_int,
        lpName: *const c_char,
    ) -> HANDLE;

    fn CreateSemaphore(
        lpSemaphoreAttributes: LPVOID,
        lInitialCount: c_int,
        lMaximumCount: c_int,
        lpName: *const c_char,
    ) -> HANDLE;

    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;

    fn ReleaseMutex(hMutex: HANDLE) -> c_int;

    fn ReleaseSemaphore(
        hSemaphore: HANDLE,
        lReleaseCount: c_int,
        lpPreviousCount: *mut c_int,
    ) -> c_int;

    fn CloseHandle(hObject: HANDLE) -> c_int;

    fn CreateThread(
        lpThreadAttributes: LPVOID,
        dwStackSize: usize,
        lpStartAddress: unsafe extern "system" fn(LPVOID) -> DWORD,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;

    fn ExitThread(dwExitCode: DWORD) -> !;

    fn Sleep(dwMilliseconds: DWORD);
}

#[repr(C)]
struct StreamInfo {
    file: HANDLE,
    used: bool,
    error: bool,
    opening: bool,
    reading: bool,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum IORequestType {
    IOREQ_OPEN = 0,
    IOREQ_READ = 1,
    IOREQ_SHUTDOWN = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct IORequest {
    r#type: IORequestType,
    handle: streamHandle_t,
    data: [DWORD; 3],
}

static mut s_Streams: *mut StreamInfo = std::ptr::null_mut();
static mut s_IORequestQueue: *mut VecDeque<IORequest> = std::ptr::null_mut();

static mut s_Thread: HANDLE = INVALID_HANDLE_VALUE;
static mut s_QueueMutex: HANDLE = INVALID_HANDLE_VALUE;
static mut s_QueueLen: HANDLE = INVALID_HANDLE_VALUE;


unsafe extern "system" fn _streamThread(_param: LPVOID) -> DWORD {
    loop {
        let req: IORequest;
        let mut bytes: DWORD = 0;
        let strm: *mut StreamInfo;

        // Wait for the IO queue to fill
        WaitForSingleObject(s_QueueLen, INFINITE);

        // Grab the next IO request
        WaitForSingleObject(s_QueueMutex, INFINITE);
        assert!(!(*s_IORequestQueue).is_empty());
        req = (*s_IORequestQueue).pop_front().unwrap();
        ReleaseMutex(s_QueueMutex);

        // Process request
        match req.r#type {
        IORequestType::IOREQ_OPEN => {
            strm = &mut (*s_Streams.add(req.handle));
            assert!((*strm).used);

            {
                let name = Sys_GetFileCodeName(req.data[0] as c_int);

                WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

                (*strm).file =
                    CreateFile(name, GENERIC_READ,
                    FILE_SHARE_READ, std::ptr::null_mut(), OPEN_EXISTING, 0, std::ptr::null_mut());

                ReleaseMutex(Sys_FileStreamMutex);
            }

            (*strm).error = ((*strm).file == INVALID_HANDLE_VALUE);
            (*strm).opening = false;
        }

        IORequestType::IOREQ_READ => {
            strm = &mut (*s_Streams.add(req.handle));
            assert!((*strm).used);

            WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

            if STREAM_SLOW_READ != 0 {
                Sleep(200);
            }

            (*strm).error =
                (SetFilePointer((*strm).file, req.data[2] as c_int, std::ptr::null_mut(), FILE_BEGIN) != req.data[2] ||
                ReadFile((*strm).file, req.data[0] as LPVOID, req.data[1], &mut bytes, std::ptr::null_mut()) == 0);

            ReleaseMutex(Sys_FileStreamMutex);

            (*strm).reading = false;
        }

        IORequestType::IOREQ_SHUTDOWN => {
            ExitThread(0);
        }
        }
    }
}


fn _sendIORequest(req: &IORequest) {
    // Add request to queue
    unsafe {
        WaitForSingleObject(s_QueueMutex, INFINITE);
        Z_SetNewDeleteTemporary(true);
        (*s_IORequestQueue).push_back(*req);
        Z_SetNewDeleteTemporary(false);
        ReleaseMutex(s_QueueMutex);
    }

    // Let IO thread know it has one more pending request
    unsafe {
        ReleaseSemaphore(s_QueueLen, 1, std::ptr::null_mut());
    }
}

pub fn Sys_IORequestQueueClear() {
    unsafe {
        WaitForSingleObject(s_QueueMutex, INFINITE);
        let _ = Box::from_raw(s_IORequestQueue);
        s_IORequestQueue = Box::into_raw(Box::new(VecDeque::new()));
        ReleaseMutex(s_QueueMutex);
    }
}

pub fn Sys_StreamInit() {
    unsafe {
        // Create array for storing open streams
        s_Streams = Z_Malloc(
            (STREAM_MAX_OPEN * std::mem::size_of::<StreamInfo>()) as c_int,
            3, // TAG_FILESYS stub
            0 as c_int) as *mut StreamInfo;
        for i in 0..STREAM_MAX_OPEN {
            (*s_Streams.add(i)).used = false;
        }

        // Create queue to hold requests for IO thread
        s_IORequestQueue = Box::into_raw(Box::new(VecDeque::new()));

        // Create a thread to service IO
        s_QueueMutex = CreateMutex(std::ptr::null_mut(), 0, std::ptr::null());
        s_QueueLen = CreateSemaphore(std::ptr::null_mut(), 0, (STREAM_MAX_OPEN * 3) as c_int, std::ptr::null());
        s_Thread = CreateThread(std::ptr::null_mut(), 0, _streamThread, std::ptr::null_mut(), 0, std::ptr::null_mut());
    }
}

pub fn Sys_StreamShutdown() {
    unsafe {
        // Tell the IO thread to shutdown
        let req = IORequest {
            r#type: IORequestType::IOREQ_SHUTDOWN,
            handle: 0,
            data: [0, 0, 0],
        };
        _sendIORequest(&req);

        // Wait for thread to close
        WaitForSingleObject(s_Thread, INFINITE);

        // Kill IO thread
        CloseHandle(s_Thread);
        CloseHandle(s_QueueLen);
        CloseHandle(s_QueueMutex);

        // Remove queue of IO requests
        let _ = Box::from_raw(s_IORequestQueue);

        // Remove streaming table
        Z_Free(s_Streams as LPVOID);
    }
}

fn GetFreeHandle() -> streamHandle_t {
    unsafe {
        for i in 1..STREAM_MAX_OPEN {
            if !(*s_Streams.add(i)).used { return i; }
        }

        // handle 0 is invalid by convention
        0
    }
}

pub fn Sys_StreamOpen(code: c_int, handle: *mut streamHandle_t) -> c_int {
    unsafe {
        // Find a free handle
        *handle = GetFreeHandle();
        if *handle == 0 {
            return -1;
        }

        // Find the file size
        let size = Sys_GetFileCodeSize(code);
        if size < 0 {
            *handle = 0;
            return -1;
        }

        // Init stream data
        (*s_Streams.add(*handle)).used = true;
        (*s_Streams.add(*handle)).opening = true;
        (*s_Streams.add(*handle)).reading = false;
        (*s_Streams.add(*handle)).error = false;

        // Send an open request to the thread
        let req = IORequest {
            r#type: IORequestType::IOREQ_OPEN,
            handle: *handle,
            data: [code as DWORD, 0, 0],
        };
        _sendIORequest(&req);

        // Return file size
        size
    }
}

pub fn Sys_StreamRead(buffer: *mut c_void, size: c_int, pos: c_int, handle: streamHandle_t) -> bool {
    unsafe {
        assert!((buffer as usize) % 32 == 0);

        // Handle must be valid.  Do not allow multiple reads.
        if !(*s_Streams.add(handle)).used || (*s_Streams.add(handle)).reading { return false; }

        // Ready to read
        (*s_Streams.add(handle)).reading = true;
        (*s_Streams.add(handle)).error = false;

        // Request IO threading reading
        let req = IORequest {
            r#type: IORequestType::IOREQ_READ,
            handle,
            data: [buffer as DWORD, size as DWORD, pos as DWORD],
        };
        _sendIORequest(&req);

        true
    }
}

pub fn Sys_StreamIsReading(handle: streamHandle_t) -> bool {
    unsafe {
        (*s_Streams.add(handle)).used && (*s_Streams.add(handle)).reading
    }
}

pub fn Sys_StreamIsError(handle: streamHandle_t) -> bool {
    unsafe {
        (*s_Streams.add(handle)).used && (*s_Streams.add(handle)).error
    }
}

pub fn Sys_StreamClose(handle: streamHandle_t) {
    unsafe {
        if (*s_Streams.add(handle)).used {
            // Block until read is done
            while (*s_Streams.add(handle)).opening || (*s_Streams.add(handle)).reading {}

            // Close the file
            CloseHandle((*s_Streams.add(handle)).file);
            (*s_Streams.add(handle)).used = false;
        }
    }
}
