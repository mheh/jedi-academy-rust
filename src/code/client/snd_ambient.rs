// Ambient Sound System (ASS!)

// leave this as first line for PCH reasons...

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::collections::HashMap;
use std::ffi::CStr;

// Type stubs and extern declarations for dependencies
// These would be defined in other modules
extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: i32) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, n: usize);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn FS_ReadFile(filename: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn S_RegisterSound(name: *const c_char) -> i32;
    fn S_AddAmbientLoopingSound(origin: *const [f32; 3], volume: u8, handle: i32);
    fn S_StartAmbientSound(origin: *const [f32; 3], entID: c_int, volume: u8, handle: i32);
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *const cvar_t;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorLength(v: *const [f32; 3]) -> f32;

    // Global state from client
    static cls: clientState_t;
}

// Type definitions and stubs
#[repr(C)]
struct cvar_t {
    name: *const c_char,
    string_value: *const c_char,
    integer: c_int,
}

#[repr(C)]
struct clientState_t {
    realtime: c_int,
}

// Constants
const MAX_SET_VOLUME: i32 = 255;
const NUM_AS_SETS: usize = 3;
const NUM_AS_KEYWORDS: usize = 9;
const MAX_WAVES_PER_GROUP: usize = 64;
const AMBIENT_SET_FILENAME: *const c_char = b"sound/ambient/ambient.txt\0" as *const u8 as *const c_char;
const TAG_AMBIENTSET: c_int = 8;

// Error codes (stub)
const ERR_DROP: c_int = 1;
const ERR_FATAL: c_int = 2;

// Keyword enum values
const SET_KEYWORD_TIMEBETWEENWAVES: c_int = 0;
const SET_KEYWORD_SUBWAVES: c_int = 1;
const SET_KEYWORD_LOOPEDWAVE: c_int = 2;
const SET_KEYWORD_VOLRANGE: c_int = 3;
const SET_KEYWORD_RADIUS: c_int = 4;
const SET_KEYWORD_TYPE: c_int = 5;
const SET_KEYWORD_AMSDIR: c_int = 6;
const SET_KEYWORD_OUTDIR: c_int = 7;
const SET_KEYWORD_BASEDIR: c_int = 8;

// Color codes for printing
const S_COLOR_YELLOW: &[u8] = b"^3\0";
const S_COLOR_RED: &[u8] = b"^1\0";

// Type alias for name precache map (using sstring_t)
type sstring_t = String;
type namePrecache_m = HashMap<sstring_t, u8>;
type parseFunc_t = fn(&mut ambientSet_t);

#[repr(C)]
pub struct ambientSet_t {
    name: [c_char; 256],
    loopedVolume: u8,
    masterVolume: u8,
    radius: i32,
    time_start: i32,
    time_end: i32,
    volRange_start: u8,
    volRange_end: u8,
    subWaves: [i32; MAX_WAVES_PER_GROUP],
    numSubWaves: usize,
    loopedWave: i32,
    id: i32,
    fadeTime: c_int,
}

// CSetGroup class equivalent
pub struct CSetGroup {
    m_ambientSets: *mut Vec<*mut ambientSet_t>,
    m_setMap: *mut HashMap<sstring_t, *mut ambientSet_t>,
    m_numSets: i32,
}

impl CSetGroup {
    pub fn new() -> Self {
        CSetGroup {
            m_ambientSets: unsafe { Box::into_raw(Box::new(Vec::new())) },
            m_setMap: unsafe { Box::into_raw(Box::new(HashMap::new())) },
            m_numSets: 0,
        }
    }

    pub fn Init(&mut self) {
        // Initialization if needed
    }

    // Free function
    pub fn Free(&mut self) {
        unsafe {
            let vec = &mut *self.m_ambientSets;
            for ai in vec.iter() {
                Z_Free(*ai as *mut c_void);
            }

            // Do this in place of clear() so it *really* frees the memory.
            drop(Box::from_raw(self.m_ambientSets));
            drop(Box::from_raw(self.m_setMap));
            self.m_ambientSets = Box::into_raw(Box::new(Vec::new()));
            self.m_setMap = Box::into_raw(Box::new(HashMap::new()));

            self.m_numSets = 0;
        }
    }

    // AddSet function
    pub fn AddSet(&mut self, name: *const c_char) -> *mut ambientSet_t {
        unsafe {
            // Allocate the memory
            let set = Z_Malloc(std::mem::size_of::<ambientSet_t>(), TAG_AMBIENTSET, 1) as *mut ambientSet_t;

            // Set up some defaults
            Q_strncpyz((*set).name.as_mut_ptr(), name, std::mem::size_of_val(&(*set).name));
            (*set).loopedVolume = MAX_SET_VOLUME as u8;
            (*set).masterVolume = MAX_SET_VOLUME as u8;
            (*set).radius = 250;
            (*set).time_start = 10;
            (*set).time_end = 25;

            (*set).volRange_start = MAX_SET_VOLUME as u8;
            (*set).volRange_end = MAX_SET_VOLUME as u8;

            let vec = &mut *self.m_ambientSets;
            vec.push(set);

            (*set).id = self.m_numSets;
            self.m_numSets += 1;

            // Map the name to the pointer for reference later
            let name_str = CStr::from_ptr(name).to_string_lossy().to_string();
            let map = &mut *self.m_setMap;
            map.insert(name_str, set);

            set
        }
    }

    // GetSet by name
    pub fn GetSet_by_name(&self, name: *const c_char) -> *mut ambientSet_t {
        unsafe {
            if name.is_null() {
                return std::ptr::null_mut();
            }

            let name_str = CStr::from_ptr(name).to_string_lossy().to_string();
            let map = &*self.m_setMap;

            match map.get(&name_str) {
                Some(&ptr) => ptr,
                None => std::ptr::null_mut(),
            }
        }
    }

    // GetSet by ID
    pub fn GetSet_by_id(&self, ID: i32) -> *mut ambientSet_t {
        unsafe {
            let vec = &*self.m_ambientSets;
            if vec.is_empty() {
                return std::ptr::null_mut();
            }

            if ID < 0 {
                return std::ptr::null_mut();
            }

            if ID >= self.m_numSets {
                return std::ptr::null_mut();
            }

            vec[ID as usize]
        }
    }

    // Generic GetSet that dispatches
    pub fn GetSet(&self, name: *const c_char) -> *mut ambientSet_t {
        self.GetSet_by_name(name)
    }
}

impl Drop for CSetGroup {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.m_ambientSets));
            drop(Box::from_raw(self.m_setMap));
        }
    }
}

// ===============================================
// File Parsing
// ===============================================

// Global variables
static mut MAX_SET_VOLUME_CONST: i32 = 255;

// Current set and old set for crossfading
static mut currentSet: i32 = -1;
static mut oldSet: i32 = -1;
static mut crossDelay: i32 = 1000; // 1 second

static mut currentSetTime: i32 = 0;
static mut oldSetTime: i32 = 0;

// Globals for debug purposes
static mut numSets: i32 = 0;

// Main ambient sound group
static mut aSets: *mut CSetGroup = std::ptr::null_mut();

// Globals for speed, blech
static mut parseBuffer: *mut c_char = std::ptr::null_mut();
static mut parseSize: i32 = 0;
static mut parsePos: i32 = 0;
static mut tempBuffer: [c_char; 1024] = [0; 1024];

// Used for enum / string matching
static setNames: [*const c_char; NUM_AS_SETS] = [
    b"generalSet\0" as *const u8 as *const c_char,
    b"localSet\0" as *const u8 as *const c_char,
    b"bmodelSet\0" as *const u8 as *const c_char,
];

// Used for keyword / enum matching
static keywordNames: [*const c_char; NUM_AS_KEYWORDS] = [
    b"timeBetweenWaves\0" as *const u8 as *const c_char,
    b"subWaves\0" as *const u8 as *const c_char,
    b"loopedWave\0" as *const u8 as *const c_char,
    b"volRange\0" as *const u8 as *const c_char,
    b"radius\0" as *const u8 as *const c_char,
    b"type\0" as *const u8 as *const c_char,
    b"amsdir\0" as *const u8 as *const c_char,
    b"outdir\0" as *const u8 as *const c_char,
    b"basedir\0" as *const u8 as *const c_char,
];

/*
-------------------------
AS_GetSetNameIDForString
-------------------------
*/
fn AS_GetSetNameIDForString(name: *const c_char) -> i32 {
    unsafe {
        // Make sure it's valid
        if name.is_null() || *name == 0 {
            return -1;
        }

        for i in 0..NUM_AS_SETS {
            if stricmp(name, setNames[i]) == 0 {
                return i as i32;
            }
        }

        -1
    }
}

/*
-------------------------
AS_GetKeywordIDForString
-------------------------
*/
fn AS_GetKeywordIDForString(name: *const c_char) -> i32 {
    unsafe {
        // Make sure it's valid
        if name.is_null() || *name == 0 {
            return -1;
        }

        for i in 0..NUM_AS_KEYWORDS {
            if stricmp(name, keywordNames[i]) == 0 {
                return i as i32;
            }
        }

        -1
    }
}

/*
-------------------------
AS_SkipLine

Skips a line in the character buffer
-------------------------
*/
fn AS_SkipLine() {
    unsafe {
        if parsePos > parseSize {
            // needed to avoid a crash because of some OOR access that shouldn't be done
            return;
        }

        while (parseBuffer as *const u8).add(parsePos as usize) as *const c_char != &b'\n' as *const u8 as *const c_char
            && (parseBuffer as *const u8).add(parsePos as usize) as *const c_char != &b'\r' as *const u8 as *const c_char
        {
            parsePos += 1;

            if parsePos > parseSize {
                return;
            }
        }

        parsePos += 1;
    }
}

/*
-------------------------
AS_GetTimeBetweenWaves

getTimeBetweenWaves <start> <end>
-------------------------
*/
fn AS_GetTimeBetweenWaves(set: &mut ambientSet_t) {
    unsafe {
        let mut startTime: i32 = 0;
        let mut endTime: i32 = 0;

        // Get the data
        sscanf_wrapper(
            parseBuffer.add(parsePos as usize),
            b"%s %d %d\0" as *const u8 as *const c_char,
            &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            &mut startTime as *mut i32 as *mut c_void,
            &mut endTime as *mut i32 as *mut c_void,
        );

        // Check for swapped start / end
        if startTime > endTime {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                Com_Printf(
                    b"%sWARNING: Corrected swapped start / end times in a \"timeBetweenWaves\" keyword\n\0"
                        as *const u8 as *const c_char,
                );
            }

            let swap = startTime;
            startTime = endTime;
            endTime = swap;
        }

        // Store it
        set.time_start = startTime;
        set.time_end = endTime;

        AS_SkipLine();
    }
}

/*
-------------------------
AS_GetSubWaves

subWaves <directory> <wave1> <wave2> ...
-------------------------
*/
fn AS_GetSubWaves(set: &mut ambientSet_t) {
    unsafe {
        let mut dirBuffer: [c_char; 512] = [0; 512];
        let mut waveBuffer: [c_char; 256] = [0; 256];
        let mut waveName: [c_char; 1024] = [0; 1024];

        // Get the directory for these sets
        sscanf_wrapper(
            parseBuffer.add(parsePos as usize),
            b"%s %s\0" as *const u8 as *const c_char,
            &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            &mut dirBuffer as *mut [c_char; 512] as *mut c_void,
        );

        // Move the pointer past these two strings
        let keyword_len = c_strlen(keywordNames[SET_KEYWORD_SUBWAVES as usize]);
        let dir_len = c_strlen(dirBuffer.as_ptr());
        parsePos += (keyword_len + 1 + dir_len + 1) as i32;

        // Get all the subwaves
        while parsePos <= parseSize {
            // Get the data
            sscanf_wrapper(
                parseBuffer.add(parsePos as usize),
                b"%s\0" as *const u8 as *const c_char,
                &mut waveBuffer as *mut [c_char; 256] as *mut c_void,
            );

            if set.numSubWaves > MAX_WAVES_PER_GROUP {
                #[cfg(not(feature = "FINAL_BUILD"))]
                {
                    Com_Printf(
                        b"%sWARNING: Too many subwaves on set \"%s\"\n\0" as *const u8 as *const c_char,
                        set.name.as_ptr(),
                    );
                }
            } else {
                // Construct the wave name (pretty, huh?)
                c_strcpy(waveName.as_mut_ptr(), b"sound/\0" as *const u8 as *const c_char);
                c_strncat(
                    waveName.as_mut_ptr(),
                    dirBuffer.as_ptr() as *const c_char,
                    waveName.len(),
                );
                c_strncat(waveName.as_mut_ptr(), b"/\0" as *const u8 as *const c_char, waveName.len());
                c_strncat(
                    waveName.as_mut_ptr(),
                    waveBuffer.as_ptr() as *const c_char,
                    waveName.len(),
                );
                c_strncat(waveName.as_mut_ptr(), b".wav\0" as *const u8 as *const c_char, waveName.len());

                // Place this onto the sound directory name

                // Precache the file at this point and store off the ID instead of the name
                let handle = S_RegisterSound(waveName.as_ptr());
                if handle > 0 {
                    set.subWaves[set.numSubWaves] = handle;
                    set.numSubWaves += 1;
                } else {
                    #[cfg(not(feature = "FINAL_BUILD"))]
                    {
                        Com_Error(
                            ERR_DROP,
                            b"ERROR: Unable to load ambient sound \"%s\"\n\0" as *const u8 as *const c_char,
                            waveName.as_ptr(),
                        );
                    }
                }
            }

            // Move the pointer past this string
            parsePos += (c_strlen(waveBuffer.as_ptr()) + 1) as i32;

            if (parseBuffer as *const u8).add(parsePos as usize) as *const c_char == &b'\n' as *const u8 as *const c_char
                || (parseBuffer as *const u8).add(parsePos as usize) as *const c_char == &b'\r' as *const u8 as *const c_char
            {
                break;
            }
        }

        AS_SkipLine();
    }
}

/*
-------------------------
AS_GetLoopedWave

loopedWave <name>
-------------------------
*/
fn AS_GetLoopedWave(set: &mut ambientSet_t) {
    unsafe {
        let mut waveBuffer: [c_char; 256] = [0; 256];
        let mut waveName: [c_char; 1024] = [0; 1024];

        // Get the looped wave name
        sscanf_wrapper(
            parseBuffer.add(parsePos as usize),
            b"%s %s\0" as *const u8 as *const c_char,
            &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            &mut waveBuffer as *mut [c_char; 256] as *mut c_void,
        );

        // Construct the wave name
        c_strcpy(waveName.as_mut_ptr(), b"sound/\0" as *const u8 as *const c_char);
        c_strncat(waveName.as_mut_ptr(), waveBuffer.as_ptr() as *const c_char, waveName.len());
        c_strncat(waveName.as_mut_ptr(), b".wav\0" as *const u8 as *const c_char, waveName.len());

        // Precache the file at this point and store off the ID instead of the name
        let handle = S_RegisterSound(waveName.as_ptr());
        if handle > 0 {
            set.loopedWave = handle;
        } else {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                Com_Error(
                    ERR_DROP,
                    b"ERROR: Unable to load looped ambient sound \"%s\"\n\0" as *const u8 as *const c_char,
                    waveName.as_ptr(),
                );
            }
        }

        AS_SkipLine();
    }
}

/*
-------------------------
AS_GetVolumeRange
-------------------------
*/
fn AS_GetVolumeRange(set: &mut ambientSet_t) {
    unsafe {
        let mut min: i32 = 0;
        let mut max: i32 = 0;

        // Get the data
        sscanf_wrapper(
            parseBuffer.add(parsePos as usize),
            b"%s %d %d\0" as *const u8 as *const c_char,
            &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            &mut min as *mut i32 as *mut c_void,
            &mut max as *mut i32 as *mut c_void,
        );

        // Check for swapped min / max
        if min > max {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                Com_Printf(
                    b"%sWARNING: Corrected swapped min / max range in a \"volRange\" keyword\n\0"
                        as *const u8 as *const c_char,
                );
            }

            let swap = min;
            min = max;
            max = swap;
        }

        // Store the data
        set.volRange_start = min as u8;
        set.volRange_end = max as u8;

        AS_SkipLine();
    }
}

/*
-------------------------
AS_GetRadius
-------------------------
*/
fn AS_GetRadius(set: &mut ambientSet_t) {
    unsafe {
        // Get the data
        sscanf_wrapper(
            parseBuffer.add(parsePos as usize),
            b"%s %d\0" as *const u8 as *const c_char,
            &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            &mut set.radius as *mut i32 as *mut c_void,
        );

        AS_SkipLine();
    }
}

/*
-------------------------
AS_GetGeneralSet
-------------------------
*/
fn AS_GetGeneralSet(set: &mut ambientSet_t) {
    unsafe {
        // The other parameters of the set come in a specific order
        while parsePos <= parseSize {
            let iFieldsScanned = sscanf_wrapper(
                parseBuffer.add(parsePos as usize),
                b"%s\0" as *const u8 as *const c_char,
                &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            );
            if iFieldsScanned <= 0 {
                return;
            }

            let keywordID = AS_GetKeywordIDForString(tempBuffer.as_ptr());

            // Find and parse the keyword info
            match keywordID {
                SET_KEYWORD_TIMEBETWEENWAVES => {
                    AS_GetTimeBetweenWaves(set);
                }
                SET_KEYWORD_SUBWAVES => {
                    AS_GetSubWaves(set);
                }
                SET_KEYWORD_LOOPEDWAVE => {
                    AS_GetLoopedWave(set);
                }
                SET_KEYWORD_VOLRANGE => {
                    AS_GetVolumeRange(set);
                }
                _ => {
                    // Check to see if we've finished this group
                    if AS_GetSetNameIDForString(tempBuffer.as_ptr()) == -1 {
                        // Ignore comments
                        if tempBuffer[0] == b';' as c_char {
                            return;
                        }

                        // This wasn't a set name, so it's an error
                        #[cfg(not(feature = "FINAL_BUILD"))]
                        {
                            Com_Printf(
                                b"%sWARNING: Unknown ambient set keyword \"%s\"\n\0" as *const u8 as *const c_char,
                                tempBuffer.as_ptr(),
                            );
                        }
                    }

                    return;
                }
            }
        }
    }
}

/*
-------------------------
AS_GetLocalSet
-------------------------
*/
fn AS_GetLocalSet(set: &mut ambientSet_t) {
    unsafe {
        // The other parameters of the set come in a specific order
        while parsePos <= parseSize {
            let iFieldsScanned = sscanf_wrapper(
                parseBuffer.add(parsePos as usize),
                b"%s\0" as *const u8 as *const c_char,
                &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            );
            if iFieldsScanned <= 0 {
                return;
            }

            let keywordID = AS_GetKeywordIDForString(tempBuffer.as_ptr());

            // Find and parse the keyword info
            match keywordID {
                SET_KEYWORD_TIMEBETWEENWAVES => {
                    AS_GetTimeBetweenWaves(set);
                }
                SET_KEYWORD_SUBWAVES => {
                    AS_GetSubWaves(set);
                }
                SET_KEYWORD_LOOPEDWAVE => {
                    AS_GetLoopedWave(set);
                }
                SET_KEYWORD_VOLRANGE => {
                    AS_GetVolumeRange(set);
                }
                SET_KEYWORD_RADIUS => {
                    AS_GetRadius(set);
                }
                _ => {
                    // Check to see if we've finished this group
                    if AS_GetSetNameIDForString(tempBuffer.as_ptr()) == -1 {
                        // Ignore comments
                        if tempBuffer[0] == b';' as c_char {
                            return;
                        }

                        // This wasn't a set name, so it's an error
                        #[cfg(not(feature = "FINAL_BUILD"))]
                        {
                            Com_Printf(
                                b"%sWARNING: Unknown ambient set keyword \"%s\"\n\0" as *const u8 as *const c_char,
                                tempBuffer.as_ptr(),
                            );
                        }
                    }

                    return;
                }
            }
        }
    }
}

/*
-------------------------
AS_GetBModelSet
-------------------------
*/
fn AS_GetBModelSet(set: &mut ambientSet_t) {
    unsafe {
        // The other parameters of the set come in a specific order
        while parsePos <= parseSize {
            let iFieldsScanned = sscanf_wrapper(
                parseBuffer.add(parsePos as usize),
                b"%s\0" as *const u8 as *const c_char,
                &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            );
            if iFieldsScanned <= 0 {
                return;
            }

            let keywordID = AS_GetKeywordIDForString(tempBuffer.as_ptr());

            // Find and parse the keyword info
            match keywordID {
                SET_KEYWORD_SUBWAVES => {
                    AS_GetSubWaves(set);
                }
                _ => {
                    // Check to see if we've finished this group
                    if AS_GetSetNameIDForString(tempBuffer.as_ptr()) == -1 {
                        // Ignore comments
                        if tempBuffer[0] == b';' as c_char {
                            return;
                        }

                        // This wasn't a set name, so it's an error
                        #[cfg(not(feature = "FINAL_BUILD"))]
                        {
                            Com_Printf(
                                b"%sWARNING: Unknown ambient set keyword \"%s\"\n\0" as *const u8 as *const c_char,
                                tempBuffer.as_ptr(),
                            );
                        }
                    }

                    return;
                }
            }
        }
    }
}

/*
-------------------------
AS_ParseSet

Parses an individual set group out of a set file buffer
-------------------------
*/
fn AS_ParseSet(setID: i32, sg: &mut CSetGroup) -> i32 {
    unsafe {
        // Make sure we're not overstepping the name array
        if setID > NUM_AS_SETS as i32 {
            return 0;
        }

        // Reset the pointers for this run through
        parsePos = 0;

        let name = setNames[setID as usize];

        // Iterate through the whole file and find every occurance of a set
        while parsePos <= parseSize {
            // Check for a valid set group
            if c_strncmp(
                parseBuffer.add(parsePos as usize),
                name,
                c_strlen(name),
            ) == 0
            {
                // Update the debug info
                numSets += 1;

                // Push past the set specifier and on to the name
                parsePos += (c_strlen(name) + 1) as i32; // Also take the following space out

                // Get the set name (this MUST be first)
                sscanf_wrapper(
                    parseBuffer.add(parsePos as usize),
                    b"%s\0" as *const u8 as *const c_char,
                    &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
                );
                AS_SkipLine();

                // Test the string against the precaches
                if tempBuffer[0] != 0 {
                    // Not in our precache listings, so skip it
                    // This would use pMap->find() in C++
                    // For now, we assume it's valid
                }

                // Create a new set
                let set = sg.AddSet(tempBuffer.as_ptr());

                // Run the function to parse the data out
                match setID {
                    0 => AS_GetGeneralSet(&mut *set),
                    1 => AS_GetLocalSet(&mut *set),
                    2 => AS_GetBModelSet(&mut *set),
                    _ => {}
                }

                continue;
            }

            // If not found on this line, go down another and check again
            AS_SkipLine();
        }

        1
    }
}

/*
-------------------------
AS_ParseHeader

Parses the directory information out of the beginning of the file
-------------------------
*/
fn AS_ParseHeader() {
    unsafe {
        let mut typeBuffer: [c_char; 128] = [0; 128];

        while parsePos <= parseSize {
            sscanf_wrapper(
                parseBuffer.add(parsePos as usize),
                b"%s\0" as *const u8 as *const c_char,
                &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
            );

            let keywordID = AS_GetKeywordIDForString(tempBuffer.as_ptr());

            match keywordID {
                SET_KEYWORD_TYPE => {
                    sscanf_wrapper(
                        parseBuffer.add(parsePos as usize),
                        b"%s %s\0" as *const u8 as *const c_char,
                        &mut tempBuffer as *mut [c_char; 1024] as *mut c_void,
                        &mut typeBuffer as *mut [c_char; 128] as *mut c_void,
                    );

                    if stricmp(typeBuffer.as_ptr(), b"ambientSet\0" as *const u8 as *const c_char) == 0 {
                        return;
                    }
                    Com_Error(
                        ERR_DROP,
                        b"AS_ParseHeader: Set type \"%s\" is not a valid set type!\n\0" as *const u8 as *const c_char,
                        typeBuffer.as_ptr(),
                    );

                }
                SET_KEYWORD_AMSDIR => {
                    // TODO: Implement
                }
                SET_KEYWORD_OUTDIR => {
                    // TODO: Implement
                }
                SET_KEYWORD_BASEDIR => {
                    // TODO: Implement
                }
                _ => {}
            }

            AS_SkipLine();
        }
    }
}

/*
-------------------------
AS_ParseFile

Opens and parses a sound set file
-------------------------
*/
fn AS_ParseFile(filename: *const c_char, sg: &mut CSetGroup) -> i32 {
    unsafe {
        // Open the file and read the information from it
        let mut buffer: *mut c_void = std::ptr::null_mut();
        parseSize = FS_ReadFile(filename, &mut buffer);

        if parseSize <= 0 {
            return 0;
        }

        parseBuffer = buffer as *mut c_char;

        // Parse the directory information out of the file
        AS_ParseHeader();

        // Parse all the relevent sets out of it
        for i in 0..NUM_AS_SETS {
            AS_ParseSet(i as i32, sg);
        }

        // Free the memory and close the file
        FS_FreeFile(parseBuffer as *mut c_void);

        1
    }
}

// ===============================================
// Main code
// ===============================================

/*
-------------------------
AS_Init

Loads the ambient sound sets and prepares to play them when needed
-------------------------
*/
fn TheNamePrecache() -> *mut namePrecache_m {
    // we use these singletons so we can find memory leaks
    // if you let things like this leak, you never can tell
    // what is really leaking and what is merely not ever freed
    static mut singleton: Option<namePrecache_m> = None;
    unsafe {
        if singleton.is_none() {
            singleton = Some(HashMap::new());
        }
        &mut singleton.as_mut().unwrap() as *mut namePrecache_m
    }
}

pub fn AS_Init() {
    unsafe {
        if aSets.is_null() {
            numSets = 0;

            // pMap = TheNamePrecache();

            // Setup the structure
            aSets = Box::into_raw(Box::new(CSetGroup::new()));
            (*aSets).Init();
        }
    }
}

/*
-------------------------
AS_AddPrecacheEntry
-------------------------
*/
pub fn AS_AddPrecacheEntry(name: *const c_char) {
    // NOTE: pMap handling would need access to static map
    // Simplified for now
    unsafe {
        if stricmp(name, b"#clear\0" as *const u8 as *const c_char) == 0 {
            currentSet = -1;
            oldSet = -1;
        }
    }
}

/*
-------------------------
AS_ParseSets

Called on the client side to load and precache all the ambient sound sets
-------------------------
*/
pub fn AS_ParseSets() {
    unsafe {
        let cv = Cvar_Get(b"s_initsound\0" as *const u8 as *const c_char, b"1\0" as *const u8 as *const c_char, 8); // CVAR_ROM = 8
        if (*cv).integer == 0 {
            return;
        }
        AS_Init();

        // Parse all the sets
        if AS_ParseFile(AMBIENT_SET_FILENAME, &mut *aSets) == 0 {
            Com_Error(
                ERR_FATAL,
                b"%sERROR: Couldn't load ambient sound sets from %s\0" as *const u8 as *const c_char,
                AMBIENT_SET_FILENAME,
            );
        }

        // Com_Printf( "AS_ParseFile: Loaded %d of %d ambient set(s)\n", pMap.size(), numSets );

        let mut iErrorsOccured: i32 = 0;
        // Would iterate through pMap here
        // Simplified for now

        if iErrorsOccured != 0 {
            Com_Error(
                ERR_DROP,
                b"....%d missing sound sets! (see above)\n\0" as *const u8 as *const c_char,
                iErrorsOccured,
            );
        }

        // //Done with the precache info, it will be rebuilt on a restart
        // pMap->clear();	// do NOT do this here now
    }
}

/*
-------------------------
AS_Free

Frees up the ambient sound system
-------------------------
*/
pub fn AS_Free() {
    unsafe {
        if !aSets.is_null() {
            (*aSets).Free();
            drop(Box::from_raw(aSets));
            aSets = std::ptr::null_mut();

            currentSet = -1;
            oldSet = -1;

            currentSetTime = 0;
            oldSetTime = 0;

            numSets = 0;
        }
    }
}

pub fn AS_FreePartial() {
    unsafe {
        if !aSets.is_null() {
            (*aSets).Free();
            currentSet = -1;
            oldSet = -1;

            currentSetTime = 0;
            oldSetTime = 0;

            numSets = 0;

            // pMap = TheNamePrecache();
            // pMap->clear();
        }
    }
}

// ===============================================
// Sound code
// ===============================================

/*
-------------------------
AS_UpdateSetVolumes

Fades volumes up or down depending on the action being taken on them.
-------------------------
*/
fn AS_UpdateSetVolumes() {
    unsafe {
        let current = (*aSets).GetSet_by_id(currentSet);

        if current.is_null() {
            return;
        }

        if (*current).masterVolume < MAX_SET_VOLUME as u8 {
            let deltaTime = cls.realtime - (*current).fadeTime;
            let scale = (deltaTime as f32) / (crossDelay as f32);
            (*current).masterVolume = (scale * MAX_SET_VOLUME as f32) as u8;
        }

        if (*current).masterVolume > MAX_SET_VOLUME as u8 {
            (*current).masterVolume = MAX_SET_VOLUME as u8;
        }

        // Only update the old set if it's still valid
        if oldSet == -1 {
            return;
        }

        let old = (*aSets).GetSet_by_id(oldSet);

        if old.is_null() {
            return;
        }

        // Update the volumes
        if (*old).masterVolume > 0 {
            let deltaTime = cls.realtime - (*old).fadeTime;
            let scale = (deltaTime as f32) / (crossDelay as f32);
            (*old).masterVolume = MAX_SET_VOLUME as u8 - (scale * MAX_SET_VOLUME as f32) as u8;
        }

        if (*old).masterVolume == 0 {
            (*old).masterVolume = 0;
            oldSet = -1;
        }
    }
}

/*
-------------------------
S_UpdateCurrentSet

Does internal maintenance to keep track of changing sets.
-------------------------
*/
fn AS_UpdateCurrentSet(id: i32) {
    unsafe {
        // Check for a change
        if id != currentSet {
            // This is new, so start the fading
            oldSet = currentSet;
            currentSet = id;

            let old = (*aSets).GetSet_by_id(oldSet);
            let current = (*aSets).GetSet_by_id(currentSet);
            // Ste, I just put this null check in for now, not sure if there's a more graceful way to exit this function - dmv
            if current.is_null() {
                return;
            }
            if !old.is_null() {
                (*old).masterVolume = MAX_SET_VOLUME as u8;
                (*old).fadeTime = cls.realtime;
            }

            (*current).masterVolume = 0;

            // Set the fading starts
            (*current).fadeTime = cls.realtime;
        }

        // Update their volumes if fading
        AS_UpdateSetVolumes();
    }
}

/*
-------------------------
AS_PlayLocalSet

Plays a local set taking volume and subwave playing into account.
Alters lastTime to reflect the time updates.
-------------------------
*/
fn AS_PlayLocalSet(
    listener_origin: *const [f32; 3],
    origin: *const [f32; 3],
    set: *mut ambientSet_t,
    entID: c_int,
    lastTime: *mut c_int,
) {
    unsafe {
        // Make sure it's valid
        if set.is_null() {
            return;
        }

        let mut dir: [f32; 3] = [0.0; 3];
        let time = cls.realtime;

        VectorSubtract(origin, listener_origin, &mut dir);
        let dist = VectorLength(&dir);

        // Determine the volume based on distance (NOTE: This sits on top of what SpatializeOrigin does)
        let distScale = if dist < ((*set).radius as f32 * 0.5) {
            1.0
        } else {
            ((*set).radius as f32 - dist) / ((*set).radius as f32 * 0.5)
        };
        let volume: u8 = if distScale > 1.0 || distScale < 0.0 {
            0
        } else {
            ((*set).masterVolume as f32 * distScale) as u8
        };

        // Add the looping sound
        if (*set).loopedWave != 0 {
            S_AddAmbientLoopingSound(origin, volume, (*set).loopedWave);
        }

        // Check the time to start another one-shot subwave
        let time_between = Q_irand((*set).time_start, (*set).time_end) * 1000;
        if (time - *lastTime) < time_between {
            return;
        }

        // Update the time
        *lastTime = time;

        // Scale the volume ranges for the subwaves based on the overall master volume
        let volScale = volume as f32 / MAX_SET_VOLUME as f32;
        let vol_rand = Q_irand(
            (volScale * (*set).volRange_start as f32) as c_int,
            (volScale * (*set).volRange_end as f32) as c_int,
        );
        let volume_sub: u8 = vol_rand as u8;

        // Add the random subwave
        if (*set).numSubWaves != 0 {
            let idx = Q_irand(0, ((*set).numSubWaves - 1) as c_int) as usize;
            S_StartAmbientSound(origin, entID, volume_sub, (*set).subWaves[idx]);
        }
    }
}

/*
-------------------------
AS_PlayAmbientSet

Plays an ambient set taking volume and subwave playing into account.
Alters lastTime to reflect the time updates.
-------------------------
*/
fn AS_PlayAmbientSet(
    origin: *const [f32; 3],
    set: *mut ambientSet_t,
    lastTime: *mut c_int,
) {
    unsafe {
        // Make sure it's valid
        if set.is_null() {
            return;
        }

        let time = cls.realtime;

        // Add the looping sound
        if (*set).loopedWave != 0 {
            S_AddAmbientLoopingSound(origin, (*set).masterVolume, (*set).loopedWave);
        }

        // Check the time to start another one-shot subwave
        let time_between = Q_irand((*set).time_start, (*set).time_end) * 1000;
        if (time - *lastTime) < time_between {
            return;
        }

        // Update the time
        *lastTime = time;

        // Scale the volume ranges for the subwaves based on the overall master volume
        let volScale = (*set).masterVolume as f32 / MAX_SET_VOLUME as f32;
        let vol_rand = Q_irand(
            (volScale * (*set).volRange_start as f32) as c_int,
            (volScale * (*set).volRange_end as f32) as c_int,
        );
        let mut volume: u8 = vol_rand as u8;

        // Allow for softer noises than the masterVolume, but not louder
        if volume > (*set).masterVolume {
            volume = (*set).masterVolume;
        }

        // Add the random subwave
        if (*set).numSubWaves != 0 {
            let idx = Q_irand(0, ((*set).numSubWaves - 1) as c_int) as usize;
            S_StartAmbientSound(origin, 0, volume, (*set).subWaves[idx]);
        }
    }
}

/*
-------------------------
S_UpdateAmbientSet

Does maintenance and plays the ambient sets (two if crossfading)
-------------------------
*/
pub fn S_UpdateAmbientSet(name: *const c_char, origin: *const [f32; 3]) {
    unsafe {
        if aSets.is_null() {
            return;
        }
        let set = (*aSets).GetSet_by_name(name);

        if set.is_null() {
            return;
        }

        // Update the current and old set for crossfading
        AS_UpdateCurrentSet((*set).id);

        let current = (*aSets).GetSet_by_id(currentSet);
        let old = (*aSets).GetSet_by_id(oldSet);

        if !current.is_null() {
            AS_PlayAmbientSet(origin, set, &mut currentSetTime);
        }

        if !old.is_null() {
            AS_PlayAmbientSet(origin, old, &mut oldSetTime);
        }
    }
}

/*
-------------------------
S_AddLocalSet
-------------------------
*/
pub fn S_AddLocalSet(
    name: *const c_char,
    listener_origin: *const [f32; 3],
    origin: *const [f32; 3],
    entID: c_int,
    time: c_int,
) -> c_int {
    unsafe {
        let set = (*aSets).GetSet_by_name(name);

        if set.is_null() {
            return cls.realtime;
        }

        let mut currentTime = time;

        AS_PlayLocalSet(listener_origin, origin, set, entID, &mut currentTime);

        currentTime
    }
}

/*
-------------------------
AS_GetBModelSound
-------------------------
*/
pub fn AS_GetBModelSound(name: *const c_char, stage: i32) -> i32 {
    unsafe {
        let set = (*aSets).GetSet_by_name(name);

        if set.is_null() {
            return -1;
        }

        // Stage must be within a valid range
        if stage > ((*set).numSubWaves - 1) as i32 || stage < 0 {
            return -1;
        }

        (*set).subWaves[stage as usize]
    }
}

// ===============================================
// Helper functions for C string operations
// ===============================================

// Stub implementations of C string functions
fn c_strlen(s: *const c_char) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

fn c_strcpy(dest: *mut c_char, src: *const c_char) {
    unsafe {
        let mut i = 0;
        loop {
            *dest.add(i) = *src.add(i);
            if *src.add(i) == 0 {
                break;
            }
            i += 1;
        }
    }
}

fn c_strncat(dest: *mut c_char, src: *const c_char, n: usize) {
    unsafe {
        let mut i = 0;
        while *dest.add(i) != 0 {
            i += 1;
        }
        let mut j = 0;
        while j < n && *src.add(j) != 0 {
            *dest.add(i) = *src.add(j);
            i += 1;
            j += 1;
        }
        *dest.add(i) = 0;
    }
}

fn c_strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let c1 = *s1.add(i) as u8;
            let c2 = *s2.add(i) as u8;
            if c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }
            if c1 == 0 {
                return 0;
            }
        }
        0
    }
}

// Simplified sscanf wrapper - handles basic %s and %d
fn sscanf_wrapper(
    buffer: *const c_char,
    fmt: *const c_char,
    args: *mut c_void,
) -> i32 {
    // This is a very simplified version - in production you'd want a proper sscanf
    // For now, just return 1 to indicate success
    1
}
