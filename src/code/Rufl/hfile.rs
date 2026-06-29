////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD USEFUL FUNCTION LIBRARY
//  (c) 2002 Activision
//
//
// Handle File
// -----------
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// Import from the handle pool and hstring modules
use super::hstring_h::hstring;

////////////////////////////////////////////////////////////////////////////////////////
// Includes (external C functions)
////////////////////////////////////////////////////////////////////////////////////////
extern "C" {
    fn HFILEopen_read(handle: *mut c_int, filepath: *const c_char) -> bool;
    fn HFILEopen_write(handle: *mut c_int, filepath: *const c_char) -> bool;
    fn HFILEread(handle: *mut c_int, data: *mut c_void, size: c_int) -> bool;
    fn HFILEwrite(handle: *mut c_int, data: *const c_void, size: c_int) -> bool;
    fn HFILEclose(handle: *mut c_int) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
const MAX_OPEN_FILES: usize = 20;

////////////////////////////////////////////////////////////////////////////////////////
// Local stub for handle_pool_vs template
// This mimics the RATL handle_pool_vs<SOpenFile, MAX_OPEN_FILES> functionality
////////////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Clone, Copy)]
struct SOpenFile {
    mPath: hstring,
    mForRead: bool,
    mHandle: c_int,
    mVersion: f32,
    mChecksum: c_int,
}

impl Default for SOpenFile {
    fn default() -> Self {
        SOpenFile {
            mPath: unsafe { std::mem::zeroed() },  // hstring default
            mForRead: false,
            mHandle: 0,
            mVersion: 0.0,
            mChecksum: 0,
        }
    }
}

// Simple handle pool implementation for SOpenFile
struct TFilePool {
    items: [SOpenFile; MAX_OPEN_FILES],
    used: [bool; MAX_OPEN_FILES],
    size: usize,
}

impl TFilePool {
    fn new() -> Self {
        // Manually construct the items array to avoid Copy/Default issues
        let mut items: [SOpenFile; MAX_OPEN_FILES] = unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };

        for i in 0..MAX_OPEN_FILES {
            items[i] = SOpenFile {
                mPath: unsafe { std::mem::zeroed() },
                mForRead: false,
                mHandle: 0,
                mVersion: 0.0,
                mChecksum: 0,
            };
        }

        TFilePool {
            items,
            used: [false; MAX_OPEN_FILES],
            size: 0,
        }
    }

    fn full(&self) -> bool {
        self.size >= MAX_OPEN_FILES
    }

    fn alloc(&mut self) -> c_int {
        for i in 0..MAX_OPEN_FILES {
            if !self.used[i] {
                self.used[i] = true;
                self.size += 1;
                return i as c_int;
            }
        }
        0  // Should not reach here if full() is checked
    }

    fn free(&mut self, handle: c_int) {
        let idx = handle as usize;
        if idx < MAX_OPEN_FILES && self.used[idx] {
            self.used[idx] = false;
            self.size = self.size.saturating_sub(1);
        }
    }

    fn is_used(&self, handle: c_int) -> bool {
        let idx = handle as usize;
        idx < MAX_OPEN_FILES && self.used[idx]
    }

    fn get(&self, handle: c_int) -> Option<&SOpenFile> {
        let idx = handle as usize;
        if idx < MAX_OPEN_FILES && self.used[idx] {
            Some(&self.items[idx])
        } else {
            None
        }
    }

    fn get_mut(&mut self, handle: c_int) -> Option<&mut SOpenFile> {
        let idx = handle as usize;
        if idx < MAX_OPEN_FILES && self.used[idx] {
            Some(&mut self.items[idx])
        } else {
            None
        }
    }

    fn index(&self, handle: c_int) -> Option<&SOpenFile> {
        self.get(handle)
    }

    fn index_mut(&mut self, handle: c_int) -> Option<&mut SOpenFile> {
        self.get_mut(handle)
    }
}

// Static pool with lazy initialization using once_cell pattern
static mut TFP: Option<TFilePool> = None;
static mut TFP_INITIALIZED: bool = false;

fn Pool() -> &'static mut TFilePool {
    unsafe {
        if !TFP_INITIALIZED {
            TFP = Some(TFilePool::new());
            TFP_INITIALIZED = true;
        }
        TFP.as_mut().unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Handle File Class
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
#[repr(C)]
pub struct hfile {
    mHandle: c_int,
}

#[allow(non_snake_case)]
impl hfile {
    ////////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    //
    // Allocates a new OpenFile structure and initializes it.  DOES NOT OPEN!
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn new(file: *const c_char) -> Self {
        let pool = Pool();

        if pool.full() {
            // HFILE: Too Many Files Open, Unable To Grab An Unused Handle
            assert!(false);
            return hfile { mHandle: 0 };
        }

        let mHandle = pool.alloc();

        if let Some(sfile) = pool.index_mut(mHandle) {
            // TODO: Need to assign hstring from *const c_char
            // sfile.mPath = hstring::from_cstr(file);
            sfile.mHandle = 0;
            sfile.mForRead = true;
        }

        hfile { mHandle }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn is_open(&self) -> bool {
        let pool = Pool();
        if self.mHandle != 0 && pool.is_used(self.mHandle) {
            if let Some(sfile) = pool.index(self.mHandle) {
                return sfile.mHandle != 0;
            }
        }
        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn is_open_for_read(&self) -> bool {
        let pool = Pool();
        if self.mHandle != 0 && pool.is_used(self.mHandle) {
            if let Some(sfile) = pool.index(self.mHandle) {
                return sfile.mHandle != 0 && sfile.mForRead;
            }
        }
        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn is_open_for_write(&self) -> bool {
        let pool = Pool();
        if self.mHandle != 0 && pool.is_used(self.mHandle) {
            if let Some(sfile) = pool.index(self.mHandle) {
                return sfile.mHandle != 0 && !sfile.mForRead;
            }
        }
        false
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn open(&mut self, version: f32, checksum: c_int, read: bool) -> bool {
        let pool = Pool();

        // Make Sure This Is A Valid Handle
        //----------------------------------
        if self.mHandle == 0 || !pool.is_used(self.mHandle) {
            // HFILE: Invalid Handle
            assert!(false);
            return false;
        }

        // Make Sure The File Is Not ALREADY Open
        //----------------------------------------
        if let Some(sfile) = pool.index(self.mHandle) {
            if sfile.mHandle != 0 {
                // HFILE: Attempt To Open An Already Open File
                assert!(false);
                return false;
            }
        } else {
            return false;
        }

        if let Some(sfile) = pool.index_mut(self.mHandle) {
            sfile.mForRead = read;
            if read {
                // TODO: Need to pass *sfile.mPath as c_str
                // HFILEopen_read(&mut sfile.mHandle, sfile.mPath.c_str());
            } else {
                // TODO: Need to pass *sfile.mPath as c_str
                // HFILEopen_write(&mut sfile.mHandle, sfile.mPath.c_str());
            }

            // If The Open Failed, Report It And Free The SOpenFile
            //------------------------------------------------------
            if sfile.mHandle == 0 {
                if !read {
                    // HFILE: Unable To Open File
                    assert!(false);
                }
                return false;
            }

            // Read The File's Header
            //------------------------
            if read {
                if !HFILEread(
                    &mut sfile.mHandle,
                    &mut sfile.mVersion as *mut f32 as *mut c_void,
                    std::mem::size_of::<f32>() as c_int,
                ) {
                    // HFILE: Unable To Read File Header
                    assert!(false);
                    let _ = self.close();
                    return false;
                }
                if !HFILEread(
                    &mut sfile.mHandle,
                    &mut sfile.mChecksum as *mut c_int as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                ) {
                    // HFILE: Unable To Read File Header
                    assert!(false);
                    let _ = self.close();
                    return false;
                }

                // Make Sure The Checksum & Version Match
                //----------------------------------------
                if sfile.mVersion != version || sfile.mChecksum != checksum {
                    let _ = self.close();
                    return false;  // Failed To Match Checksum Or Version Number -> Old Data
                }
            } else {
                sfile.mVersion = version;
                sfile.mChecksum = checksum;

                if !HFILEwrite(
                    &mut sfile.mHandle,
                    &sfile.mVersion as *const f32 as *const c_void,
                    std::mem::size_of::<f32>() as c_int,
                ) {
                    // HFILE: Unable To Write File Header
                    assert!(false);
                    let _ = self.close();
                    return false;
                }
                if !HFILEwrite(
                    &mut sfile.mHandle,
                    &sfile.mChecksum as *const c_int as *const c_void,
                    std::mem::size_of::<c_int>() as c_int,
                ) {
                    // HFILE: Unable To Write File Header
                    assert!(false);
                    let _ = self.close();
                    return false;
                }
            }
        }

        true
    }

    pub fn open_read(&mut self, version: f32, checksum: c_int) -> bool {
        self.open(version, checksum, true)
    }

    pub fn open_write(&mut self, version: f32, checksum: c_int) -> bool {
        self.open(version, checksum, false)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn close(&mut self) -> bool {
        let pool = Pool();

        if self.mHandle == 0 || !pool.is_used(self.mHandle) {
            // HFILE: Invalid Handle
            assert!(false);
            return false;
        }

        if let Some(sfile) = pool.index(self.mHandle) {
            if sfile.mHandle == 0 {
                // HFILE: Unable TO Close Unopened File
                assert!(false);
                return false;
            }
        } else {
            return false;
        }

        if let Some(sfile) = pool.index_mut(self.mHandle) {
            if !HFILEclose(&mut sfile.mHandle) {
                sfile.mHandle = 0;
                // HFILE: Unable To Close File
                assert!(false);
                return false;
            }
            sfile.mHandle = 0;
        }

        true
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Searches for the first block with the matching data size, and reads it in.
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn load(&mut self, data: *mut c_void, datasize: c_int) -> bool {
        // Go Ahead And Open The File For Reading
        //----------------------------------------
        let mut auto_opened = false;
        if !self.is_open() {
            if !self.open_read(1.0, 0) {
                return false;
            }
            auto_opened = true;
        }

        // Make Sure That The File Is Readable
        //-------------------------------------
        let pool = Pool();
        if let Some(sfile) = pool.index(self.mHandle) {
            if !sfile.mForRead {
                // HFILE: Unable to load from a file that is opened for save
                assert!(false);
                if auto_opened {
                    let _ = self.close();
                }
                return false;
            }
        } else {
            if auto_opened {
                let _ = self.close();
            }
            return false;
        }

        // Now Read It
        //-------------
        if let Some(sfile) = pool.index_mut(self.mHandle) {
            if !HFILEread(&mut sfile.mHandle, data, datasize) {
                // HFILE: Unable To Read Object
                assert!(false);
                if auto_opened {
                    let _ = self.close();
                }
                return false;
            }
        }

        // Success!
        //----------
        if auto_opened {
            let _ = self.close();
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn save(&mut self, data: *mut c_void, datasize: c_int) -> bool {
        // Go Ahead And Open The File For Reading
        //----------------------------------------
        let mut auto_opened = false;
        if !self.is_open() {
            if !self.open_write(1.0, 0) {
                return false;
            }
            auto_opened = true;
        }

        // Make Sure That The File Is Readable
        //-------------------------------------
        let pool = Pool();
        if let Some(sfile) = pool.index(self.mHandle) {
            if sfile.mForRead {
                // HFILE: Unable to save to a file that is opened for read
                assert!(false);
                if auto_opened {
                    let _ = self.close();
                }
                return false;
            }
        } else {
            if auto_opened {
                let _ = self.close();
            }
            return false;
        }

        // Write The Actual Object
        //-------------------------
        if let Some(sfile) = pool.index_mut(self.mHandle) {
            if !HFILEwrite(&mut sfile.mHandle, data as *const c_void, datasize) {
                // HFILE: Unable To Write File Data
                assert!(false);
                if auto_opened {
                    let _ = self.close();
                }
                return false;
            }
        }

        if auto_opened {
            let _ = self.close();
        }
        true
    }
}

impl Drop for hfile {
    ////////////////////////////////////////////////////////////////////////////////////////
    // Destructor
    //
    // Releases the open file structure for resue.  Also closes the file if open.
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    fn drop(&mut self) {
        if self.is_open() {
            let _ = self.close();
        }

        let pool = Pool();
        if self.mHandle != 0 && pool.is_used(self.mHandle) {
            pool.free(self.mHandle);
        }
        self.mHandle = 0;
    }
}
