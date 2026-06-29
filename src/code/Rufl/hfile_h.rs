use core::ffi::{c_char, c_int, c_void};

////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD USEFUL FUNCTION LIBRARY
//  (c) 2002 Activision
//
//
// Handle File
// -----------
//
////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////
// HFile Bindings
//
// These are the standard C hfile bindings, copy these function wrappers to your .cpp
// before including hfile, and modify them if needed to support a different file
// system.
////////////////////////////////////////////////////////////////////////////////////////
//bool	HFILEopen_read(int& handle,	const char* filepath)		{handle=(int)fopen(filepath, "rb");	return (handle!=0);}
//bool	HFILEopen_write(int& handle, const char* filepath)		{handle=(int)fopen(filepath, "wb");	return (handle!=0);}
//bool	HFILEread(int& handle,		void*		data, int size)	{return (fread(data, size, 1, (FILE*)(handle))>0);}
//bool	HFILEwrite(int& handle,		const void* data, int size)	{return (fwrite(data, size, 1, (FILE*)(handle))>0);}
//bool	HFILEclose(int& handle)									{return (fclose((FILE*)handle)==0);}



////////////////////////////////////////////////////////////////////////////////////////
// The Handle String Class
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
#[repr(C)]
pub struct hfile {
    mHandle: c_int,
}

#[allow(non_snake_case)]
impl hfile {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new(file: *const c_char) -> Self {
        hfile {
            mHandle: 0,
        }
    }

    pub fn load(&mut self, data: *mut c_void, datasize: c_int) -> bool {
        false
    }

    pub fn save(&mut self, data: *mut c_void, datasize: c_int) -> bool {
        false
    }

    pub fn is_open(&self) -> bool {
        self.mHandle != 0
    }

    pub fn is_open_for_read(&self) -> bool {
        false
    }

    pub fn is_open_for_write(&self) -> bool {
        false
    }

    pub fn open_read(&mut self, version: f32, checksum: c_int) -> bool {
        self.open(version, checksum, true)
    }

    pub fn open_write(&mut self, version: f32, checksum: c_int) -> bool {
        self.open(version, checksum, false)
    }

    pub fn close(&mut self) -> bool {
        false
    }

    fn open(&mut self, version: f32, checksum: c_int, read: bool) -> bool {
        false
    }
}

impl Drop for hfile {
    fn drop(&mut self) {
    }
}
