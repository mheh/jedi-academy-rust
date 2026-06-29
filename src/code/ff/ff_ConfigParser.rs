#![allow(non_snake_case)]
#![cfg(feature = "_IMMERSION")]

use core::ffi::{c_char, c_void, c_int, c_uint, c_ushort};
use std::collections::{HashMap, HashSet};

// External C functions
extern "C" {
    fn COM_ParseExt(pos: *mut *const c_char, allow_line_break: c_int) -> *const c_char;
    fn LoadFile(filename: *const c_char) -> *mut c_void;
    fn FS_FreeFile(file: *mut c_void);
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn RightOf(effectname: *const c_char, setname: *const c_char) -> *const c_char;

    #[cfg(feature = "FF_PRINT")]
    fn ConsoleParseError(msg: *const c_char, token: *const c_char);

    // For CImmDevice::GetProductName and GetProductType
    fn GetProductName(device: *const c_void, name: *mut c_char, max_len: c_int);
    fn GetProductType(device: *const c_void) -> c_uint;
}

// Type definitions based on usage patterns
pub type qboolean = c_int;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

pub type TInclude = Vec<*const c_char>;
pub type TDeviceType = HashMap<c_ushort, String>;
pub type TDevice = HashSet<String>;

#[repr(C)]
pub struct TData {
    pub include: TInclude,
    pub device: TDevice,
}

impl TData {
    fn new() -> Self {
        TData {
            include: Vec::new(),
            device: HashSet::new(),
        }
    }
}

pub type TMap = HashMap<String, TData>;
pub type TDefaultMap = HashMap<i32, TDeviceType>;
pub type TPriorityList = Vec<i32>;

pub struct FFConfigParser {
    mMap: TMap,
    mDefaultSet: TDefaultMap,
    mDefaultPriority: TPriorityList,
}

impl FFConfigParser {
    pub fn new() -> Self {
        FFConfigParser {
            mMap: HashMap::new(),
            mDefaultSet: HashMap::new(),
            mDefaultPriority: Vec::new(),
        }
    }

    ////--------------------
    /// FFConfigParser::Init
    //------------------------
    //	Reads the force feedback configuration file. Call this once after the device
    //	is initialized.
    //
    //	Parameters:
    //	*	filename
    //
    //	Returns:
    //	*	qtrue - the effects set directory has been set according to the initialized
    //			device. (See base/fffx/fffx.cfg)
    //	*	qfalse - no effects set could be determined for this device.
    //
    pub fn Init(&mut self, filename: *const c_char) -> qboolean {
        self.Clear();	// Always cleanup

        unsafe {
            if filename.is_null() {
                qfalse
            } else {
                if self.Parse(LoadFile(filename)) != 0 { qtrue } else { qfalse }
            }
        }
    }

    ////---------------------
    /// FFConfigParser::Clear
    //-------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn Clear(&mut self) {
        self.mMap.clear();
        self.mDefaultSet.clear();
    }

    ////---------------------
    /// FFConfigParser::Parse
    //-------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn Parse(&mut self, file: *mut c_void) -> qboolean {
        let mut result = if !file.is_null() { qtrue } else { qfalse };

        if !file.is_null() {
            let mut pos = file as *const c_char;
            unsafe {
                let mut token = COM_ParseExt(&mut pos, qtrue);
                loop {
                    if token.is_null() || *token == 0 {
                        break;
                    }
                    if result == qfalse {
                        break;	// fail if any problem
                    }

                    if stricmp(token, "ffdefaults\0".as_ptr() as *const c_char) == 0 {
                        result &= self.ParseDefaults(&mut pos);
                    } else if stricmp(token, "ffsets\0".as_ptr() as *const c_char) == 0 {
                        result &= self.ParseSets(&mut pos);
                    } else {
                        // unexpected field
                        result = qfalse;
                    }

                    token = COM_ParseExt(&mut pos, qtrue);
                }

                FS_FreeFile(file);
            }
        }

        result
    }

    ////---------------------------------
    /// FFConfigParser::ParseDefaultBlock
    //-------------------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn ParseDefault(&mut self, pos: &mut *const c_char, default_set: &mut TDeviceType) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        let mut device: i32 = 0;

                        if sscanf(token, "%d\0".as_ptr() as *const c_char, &mut device) != 0 {
                            let device_u16 = device as c_ushort;
                            if !default_set.contains_key(&device_u16) {
                                let str_val = COM_ParseExt(pos, qfalse);
                                if !str_val.is_null() && *str_val != 0 {
                                    let s = std::ffi::CStr::from_ptr(str_val)
                                        .to_str()
                                        .unwrap_or("")
                                        .to_string();
                                    default_set.insert(device_u16, s);
                                    result &= if s.len() > 0 { qtrue } else { qfalse };
                                } else {
                                    result &= qfalse;
                                }
                            } else {
                                result = qfalse;
                                #[cfg(feature = "FF_PRINT")]
                                {
                                    ConsoleParseError(
                                        "Redefinition of DeviceType index\0".as_ptr() as *const c_char,
                                        token,
                                    );
                                }
                            }
                        } else {
                            result = qfalse;
                            #[cfg(feature = "FF_PRINT")]
                            {
                                ConsoleParseError(
                                    "DeviceType field should begin with an integer\0".as_ptr() as *const c_char,
                                    token,
                                );
                            }
                        }
                    }
                }
            }
        }

        result
    }

    ////----------------------------
    /// FFConfigParser::ParseDefault
    //--------------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn ParseDefaults(&mut self, pos: &mut *const c_char) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        let mut tech_type: i32 = 0;

                        if sscanf(token, "%d\0".as_ptr() as *const c_char, &mut tech_type) != 0 {
                            if !self.mDefaultSet.contains_key(&tech_type) {
                                let device_type = self.mDefaultSet.entry(tech_type).or_insert_with(HashMap::new);
                                result &= self.ParseDefault(pos, device_type);
                                self.mDefaultPriority.push(tech_type);
                            } else {
                                result = qfalse;
                                #[cfg(feature = "FF_PRINT")]
                                {
                                    ConsoleParseError(
                                        "Redefinition of TechType index\0".as_ptr() as *const c_char,
                                        token,
                                    );
                                }
                            }
                        } else {
                            result = qfalse;
                            #[cfg(feature = "FF_PRINT")]
                            {
                                ConsoleParseError(
                                    "TechType fields should begin with integers\0".as_ptr() as *const c_char,
                                    token,
                                );
                            }
                        }
                    }
                } else {
                    // expected '{'
                    result = qfalse;
                }
            }
        }

        result
    }

    ////--------------------------
    /// FFConfigParser::RightOfSet
    //------------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn RightOfSet(&self, effectname: *const c_char) -> *const c_char {
        let mut s = effectname;

        // Check through all set names and test effectname against it
        for (set_name, _) in &self.mMap {
            unsafe {
                let right = RightOf(effectname, set_name.as_ptr() as *const c_char);
                if s == effectname && !right.is_null() {
                    s = right;
                }
            }
        }

        s
    }

    pub fn ParseSetDevices(&mut self, pos: &mut *const c_char, device: &mut TDevice) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        let s = std::ffi::CStr::from_ptr(token)
                            .to_str()
                            .unwrap_or("")
                            .to_string();
                        device.insert(s);
                    }

                    result = if !token.is_null() && *token != 0 { qtrue } else { qfalse };
                } else {
                    // expected '{'
                    result = qfalse;
                }
            }
        }

        result
    }

    pub fn ParseSetIncludes(&mut self, pos: &mut *const c_char, include: &mut TInclude) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        include.push(token);
                    }

                    result = if !token.is_null() && *token != 0 { qtrue } else { qfalse };
                } else {
                    // expected '{'
                    result = qfalse;
                }
            }
        }

        result
    }

    pub fn ParseSet(&mut self, pos: &mut *const c_char, data: &mut TData) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let oldpos = *pos;	// allows set declarations with no attributes to have no "{}"
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        if stricmp(token, "includes\0".as_ptr() as *const c_char) == 0 {
                            result &= self.ParseSetIncludes(pos, &mut data.include);
                        } else if stricmp(token, "devices\0".as_ptr() as *const c_char) == 0 {
                            result &= self.ParseSetDevices(pos, &mut data.device);
                        } else {
                            result = qfalse;
                            #[cfg(feature = "FF_PRINT")]
                            {
                                ConsoleParseError(
                                    "Invalid set parameter. Should be 'includes' or 'devices'\0".as_ptr() as *const c_char,
                                    token,
                                );
                            }
                        }
                    }
                } else {
                    // expected '{'		(no longer expected!)
                    //result = qfalse;	(no longer an error!)
                    *pos = oldpos;
                }
            }
        }

        result
    }

    ////-------------------------
    /// FFConfigParser::ParseSets
    //-----------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn ParseSets(&mut self, pos: &mut *const c_char) -> qboolean {
        let mut result = if pos.is_null() { qfalse } else { qtrue };

        if !pos.is_null() {
            unsafe {
                let mut token = COM_ParseExt(pos, qtrue);
                if !token.is_null() && *token == ('{' as c_char) {
                    loop {
                        token = COM_ParseExt(pos, qtrue);
                        if token.is_null() || *token == 0 || *token == ('}' as c_char) {
                            break;
                        }
                        if result == qfalse {
                            break;	// fail if any problem
                        }

                        let group_name = std::ffi::CStr::from_ptr(token)
                            .to_str()
                            .unwrap_or("")
                            .to_string();

                        let data = self.mMap.entry(group_name).or_insert_with(TData::new);
                        result &= self.ParseSet(pos, data);
                    }
                } else {
                    // expected '{'
                    result = qfalse;
                }
            }
        }

        result
    }

    ////---------------------------
    /// FFConfigParser::GetIncludes
    //-------------------------------
    //
    //
    //	Parameters:
    //
    //	Returns:
    //
    pub fn GetIncludes(&self, name: *const c_char) -> *const TInclude {
        let name_str = unsafe {
            std::ffi::CStr::from_ptr(name)
                .to_str()
                .unwrap_or("")
        };

        for (key, data) in &self.mMap {
            if key == name_str {
                return &data.include as *const TInclude;
            }
        }

        // No includes present
        static EMPTY_INCLUDE: TInclude = Vec::new();
        &EMPTY_INCLUDE as *const TInclude
    }

    pub fn GetFFSet(&self, device: *const c_void) -> *const c_char {
        const FF_MAX_PATH: usize = 260;
        let mut dev_name: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];
        let mut ffset: *const c_char = core::ptr::null();

        //
        //	Check explicit name
        //

        unsafe {
            GetProductName(device, &mut dev_name[0], (FF_MAX_PATH - 1) as c_int);

            let dev_name_str = std::ffi::CStr::from_ptr(&dev_name[0])
                .to_str()
                .unwrap_or("");

            for (set_name, data) in &self.mMap {
                if data.device.contains(dev_name_str) {
                    ffset = set_name.as_ptr() as *const c_char;
                    return ffset;
                }
            }
        }

        //
        //	Check device defaults
        //

        unsafe {
            for i in 0..self.mDefaultPriority.len() {
                if !ffset.is_null() {
                    break;
                }

                let default_tech_type = self.mDefaultPriority[i];
                let product_type = GetProductType(device);
                let device_type: c_ushort = ((product_type >> 16) & 0xFFFF) as c_ushort;
                let tech_type: c_ushort = (product_type & 0xFFFF) as c_ushort;

                //
                //	Check for minimum required features
                //

                if (tech_type as i32 & default_tech_type) >= default_tech_type {
                    //
                    //	Check that device exists in this technology section
                    //

                    if let Some(device_type_map) = self.mDefaultSet.get(&default_tech_type) {
                        if let Some(effect_set) = device_type_map.get(&device_type) {
                            ffset = effect_set.as_ptr() as *const c_char;
                        }
                    }
                }

                //
                //	If not, try next technology section
                //
            }
        }

        ffset
    }
}
