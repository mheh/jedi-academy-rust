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

use core::ffi::c_int;
use core::ptr::addr_of_mut;
use std::collections::VecDeque;

type HANDLE = *mut core::ffi::c_void;
type DWORD = u32;
type LPVOID = *mut core::ffi::c_void;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const GENERIC_READ: DWORD = 0x80000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const OPEN_EXISTING: DWORD = 0x00000001;
const FILE_BEGIN: DWORD = 0;
const INFINITE: DWORD = 0xFFFFFFFF;

extern "C" {
    fn Z_SetNewDeleteTemporary(flag: bool);
    fn Sys_GetFileCodeName(code: c_int) -> *const u8;
    fn Sys_GetFileCodeSize(code: c_int) -> c_int;
    static mut Sys_FileStreamMutex: HANDLE;

    // Memory allocation
    fn Z_Malloc(size: c_int, tag: c_int, clear: bool) -> LPVOID;
    fn Z_Free(ptr: LPVOID);

    // Windows API
    fn CreateMutex(lpMutexAttributes: *const core::ffi::c_void, bInitialOwner: bool, lpName: *const u8) -> HANDLE;
    fn CreateSemaphore(lpSemaphoreAttributes: *const core::ffi::c_void, lInitialCount: c_int, lMaximumCount: c_int, lpName: *const u8) -> HANDLE;
    fn CreateThread(lpThreadAttributes: *const core::ffi::c_void, dwStackSize: u32, lpStartAddress: extern "C" fn(LPVOID) -> u32, lpParameter: LPVOID, dwCreationFlags: u32, lpThreadId: *mut u32) -> HANDLE;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn ReleaseMutex(hMutex: HANDLE) -> bool;
    fn ReleaseSemaphore(hSemaphore: HANDLE, lReleaseCount: c_int, lpPreviousCount: *mut c_int) -> bool;
    fn CloseHandle(hObject: HANDLE) -> bool;
    fn ExitThread(dwExitCode: u32) -> !;
    fn CreateFile(lpFileName: *const u8, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: *const core::ffi::c_void, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE) -> HANDLE;
    fn SetFilePointer(hFile: HANDLE, lDistanceToMove: i32, lpDistanceToMoveHigh: *mut i32, dwMoveMethod: DWORD) -> DWORD;
    fn ReadFile(hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: u32, lpNumberOfBytesRead: *mut u32, lpOverlapped: *const core::ffi::c_void) -> bool;
    fn Sleep(dwMilliseconds: u32);
}

const STREAM_SLOW_READ: c_int = 0;

const STREAM_MAX_OPEN: usize = 48;

// Tag for Z_Malloc
const TAG_FILESYS: c_int = 0; // Stub; actual tag value from engine

type streamHandle_t = c_int;

#[repr(C)]
struct StreamInfo {
    file: HANDLE,
    used: bool,
    error: bool,
    opening: bool,
    reading: bool,
    // Note: C version marks these fields as volatile for thread-safe access.
    // Rust does not support volatile struct fields. Access is protected by
    // synchronization primitives (mutexes/semaphores).
}

#[derive(Clone, Copy)]
enum IORequestType {
    IOREQ_OPEN,
    IOREQ_READ,
    IOREQ_SHUTDOWN,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct IORequest {
    type_: IORequestType,
    handle: streamHandle_t,
    data: [DWORD; 3],
}

static mut s_Streams: *mut StreamInfo = core::ptr::null_mut();

static mut s_IORequestQueue: *mut VecDeque<IORequest> = core::ptr::null_mut();

static mut s_Thread: HANDLE = INVALID_HANDLE_VALUE;
static mut s_QueueMutex: HANDLE = INVALID_HANDLE_VALUE;
static mut s_QueueLen: HANDLE = INVALID_HANDLE_VALUE;


extern "C" fn _streamThread(_: LPVOID) -> u32 {
    loop {
        let mut req: IORequest;
        let mut bytes: u32 = 0;
        let strm: *mut StreamInfo;

        unsafe {
            // Wait for the IO queue to fill
            WaitForSingleObject(s_QueueLen, INFINITE);

            // Grab the next IO request
            WaitForSingleObject(s_QueueMutex, INFINITE);
            assert!(!(*s_IORequestQueue).is_empty());
            req = (*s_IORequestQueue).pop_front().unwrap();
            ReleaseMutex(s_QueueMutex);

            // Process request
            strm = &mut (*s_Streams)[req.handle as usize];

            match req.type_ {
                IORequestType::IOREQ_OPEN => {
                    assert!((*strm).used);

                    {
                        let name = Sys_GetFileCodeName(req.data[0] as c_int);

                        WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

                        (*strm).file =
                            CreateFile(name, GENERIC_READ,
                            FILE_SHARE_READ, core::ptr::null(), OPEN_EXISTING, 0, INVALID_HANDLE_VALUE);

                        ReleaseMutex(Sys_FileStreamMutex);
                    }

                    (*strm).error = ((*strm).file == INVALID_HANDLE_VALUE);
                    (*strm).opening = false;
                }

                IORequestType::IOREQ_READ => {
                    assert!((*strm).used);

                    WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

                    if STREAM_SLOW_READ != 0 {
                        Sleep(200);
                    }

                    (*strm).error =
                        (SetFilePointer((*strm).file, req.data[2] as i32, core::ptr::null_mut(), FILE_BEGIN) != req.data[2] ||
                        ReadFile((*strm).file, req.data[0] as LPVOID, req.data[1], &mut bytes, core::ptr::null()) == false);

                    ReleaseMutex(Sys_FileStreamMutex);

                    (*strm).reading = false;
                }

                IORequestType::IOREQ_SHUTDOWN => {
                    ExitThread(0);
                }
            }
        }
    }

    1
}


fn _sendIORequest(req: &IORequest) {
    unsafe {
        // Add request to queue
        WaitForSingleObject(s_QueueMutex, INFINITE);
        Z_SetNewDeleteTemporary(true);
        (*s_IORequestQueue).push_back(*req);
        Z_SetNewDeleteTemporary(false);
        ReleaseMutex(s_QueueMutex);

        // Let IO thread know it has one more pending request
        ReleaseSemaphore(s_QueueLen, 1, core::ptr::null_mut());
    }
}

pub fn Sys_IORequestQueueClear() {
    unsafe {
        WaitForSingleObject(s_QueueMutex, INFINITE);
        if !s_IORequestQueue.is_null() {
            let _ = Box::from_raw(s_IORequestQueue);
        }
        s_IORequestQueue = Box::into_raw(Box::new(VecDeque::new()));
        ReleaseMutex(s_QueueMutex);
    }
}

pub fn Sys_StreamInit() {
    unsafe {
        // Create array for storing open streams
        s_Streams = Z_Malloc(
            (STREAM_MAX_OPEN * core::mem::size_of::<StreamInfo>()) as c_int, TAG_FILESYS, false) as *mut StreamInfo;
        for i in 0..STREAM_MAX_OPEN {
            (*s_Streams.add(i)).used = false;
        }

        // Create queue to hold requests for IO thread
        s_IORequestQueue = Box::into_raw(Box::new(VecDeque::new()));

        // Create a thread to service IO
        s_QueueMutex = CreateMutex(core::ptr::null(), false, core::ptr::null());
        s_QueueLen = CreateSemaphore(core::ptr::null(), 0, (STREAM_MAX_OPEN * 3) as c_int, core::ptr::null());
        s_Thread = CreateThread(core::ptr::null(), 64*1024, _streamThread, core::ptr::null_mut(), 0, core::ptr::null_mut());
    }
}

pub fn Sys_StreamShutdown() {
    unsafe {
        // Tell the IO thread to shutdown
        let req = IORequest {
            type_: IORequestType::IOREQ_SHUTDOWN,
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
        if !s_IORequestQueue.is_null() {
            let _ = Box::from_raw(s_IORequestQueue);
        }

        // Remove streaming table
        Z_Free(s_Streams as LPVOID);
    }
}

fn GetFreeHandle() -> streamHandle_t {
    unsafe {
        for i in 1..STREAM_MAX_OPEN {
            if !(*s_Streams.add(i)).used {
                return i as streamHandle_t;
            }
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
        let strm = &mut *s_Streams.add(*handle as usize);
        strm.used = true;
        strm.opening = true;
        strm.reading = false;
        strm.error = false;

        // Send an open request to the thread
        let mut req = IORequest {
            type_: IORequestType::IOREQ_OPEN,
            handle: *handle,
            data: [code as DWORD, 0, 0],
        };
        _sendIORequest(&req);

        // Return file size
        size
    }
}

pub fn Sys_StreamRead(buffer: *mut core::ffi::c_void, size: c_int, pos: c_int, handle: streamHandle_t) -> bool {
    unsafe {
        assert!((buffer as usize) % 32 == 0);

        // Handle must be valid.  Do not allow multiple reads.
        if !(*s_Streams.add(handle as usize)).used || (*s_Streams.add(handle as usize)).reading {
            return false;
        }

        // Ready to read
        let strm = &mut *s_Streams.add(handle as usize);
        strm.reading = true;
        strm.error = false;

        // Request IO threading reading
        let req = IORequest {
            type_: IORequestType::IOREQ_READ,
            handle: handle,
            data: [buffer as DWORD, size as DWORD, pos as DWORD],
        };
        _sendIORequest(&req);

        true
    }
}

pub fn Sys_StreamIsReading(handle: streamHandle_t) -> bool {
    unsafe {
        (*s_Streams.add(handle as usize)).used && (*s_Streams.add(handle as usize)).reading
    }
}

pub fn Sys_StreamIsError(handle: streamHandle_t) -> bool {
    unsafe {
        (*s_Streams.add(handle as usize)).used && (*s_Streams.add(handle as usize)).error
    }
}

pub fn Sys_StreamClose(handle: streamHandle_t) {
    unsafe {
        let strm = &mut *s_Streams.add(handle as usize);
        if strm.used {
            // Block until read is done
            while strm.opening || strm.reading {}

            // Close the file
            CloseHandle(strm.file);
            strm.used = false;
        }
    }
}
