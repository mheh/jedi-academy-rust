//! Ambient Sound System (ASS!)
//!
//! Mechanical port of `codemp/client/snd_ambient.cpp`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
use core::mem::{size_of, offset_of};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::client::snd_ambient_h::*;
use crate::codemp::game::q_math::{VectorSubtract, VectorLength};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::codemp::qcommon::sstring_h::sstring_t;
use crate::codemp::qcommon::tags_h::TAG_AMBIENTSET;

// ============================================================================
// Local stub types for unported STL containers
// ============================================================================

#[repr(C)]
pub struct std_map_sstring_uchar {
    _opaque: *mut c_void,
}

// ============================================================================
// External C dependencies (stubs/extern declarations)
// ============================================================================

extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn FS_ReadFile(filename: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Q_irand(min: c_int, max: c_int) -> c_int;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncat(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;
    fn S_RegisterSound(name: *const c_char) -> sfxHandle_t;
    fn S_AddAmbientLoopingSound(origin: vec3_t, volume: c_uchar, handle: sfxHandle_t);
    fn S_StartAmbientSound(origin: vec3_t, entID: c_int, volume: c_uchar, handle: sfxHandle_t);
}

// ============================================================================
// External globals/state (stubs for unported globals)
// ============================================================================

#[repr(C)]
pub struct clientActive_t {
    pub realtime: c_int,
    // Other fields omitted - only realtime is used
}

extern "C" {
    static mut cls: clientActive_t;
}

// Error codes
const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;

// Color codes
const S_COLOR_YELLOW: &[u8] = b"^3\0";
const S_COLOR_RED: &[u8] = b"^1\0";

const MAX_SET_VOLUME: c_int = 255;

// ============================================================================
// Forward declarations
// ============================================================================

unsafe fn AS_GetGeneralSet(set: *mut ambientSet_t);
unsafe fn AS_GetLocalSet(set: *mut ambientSet_t);
unsafe fn AS_GetBModelSet(set: *mut ambientSet_t);

// ============================================================================
// Static globals (mirroring C file-scope statics)
// ============================================================================

// Current set and old set for crossfading
static mut currentSet: c_int = -1;
static mut oldSet: c_int = -1;
static mut crossDelay: c_int = 1000; // 1 second

static mut currentSetTime: c_int = 0;
static mut oldSetTime: c_int = 0;

// Globals for debug purposes
static mut numSets: c_int = 0;

// Main ambient sound group
static mut aSets: *mut CSetGroup = null_mut();

// Globals for speed, blech
static mut parseBuffer: *mut c_char = null_mut();
static mut parseSize: c_int = 0;
static mut parsePos: c_int = 0;
static mut tempBuffer: [c_char; 1024] = [0; 1024];

// NOTENOTE: Be sure to change the mirrored code in g_spawn.cpp, and cg_main.cpp
// typedef map<sstring_t, unsigned char> namePrecache_m;
// static namePrecache_m* pMap = NULL;
static mut pMap: *mut std_map_sstring_uchar = null_mut();

// Used for enum / string matching
static SETNAMES: [*const c_char; NUM_AS_SETS as usize] = [
    b"generalSet\0".as_ptr() as *const c_char,
    b"localSet\0".as_ptr() as *const c_char,
    b"bmodelSet\0".as_ptr() as *const c_char,
];

// Used for enum / function matching
type ParseFuncRaw = unsafe fn(*mut ambientSet_t);

static PARSEFUNCS: [ParseFuncRaw; NUM_AS_SETS as usize] = [
    AS_GetGeneralSet,
    AS_GetLocalSet,
    AS_GetBModelSet,
];

// Used for keyword / enum matching
static KEYWORDNAMES: [*const c_char; NUM_AS_KEYWORDS as usize] = [
    b"timeBetweenWaves\0".as_ptr() as *const c_char,
    b"subWaves\0".as_ptr() as *const c_char,
    b"loopedWave\0".as_ptr() as *const c_char,
    b"volRange\0".as_ptr() as *const c_char,
    b"radius\0".as_ptr() as *const c_char,
    b"type\0".as_ptr() as *const c_char,
    b"amsdir\0".as_ptr() as *const c_char,
    b"outdir\0".as_ptr() as *const c_char,
    b"basedir\0".as_ptr() as *const c_char,
];

// ============================================================================
// CSetGroup implementation
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn CSetGroup_CSetGroup(this: *mut CSetGroup) {
    // Constructor: allocate vector and map
    (*this).m_ambientSets = Z_Malloc(size_of::<std_vector_ambientSet_ptr>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_vector_ambientSet_ptr;
    (*this).m_setMap = Z_Malloc(size_of::<std_map_sstring_t_ambientSet_ptr>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_map_sstring_t_ambientSet_ptr;
}

#[no_mangle]
pub unsafe extern "C" fn CSetGroup_destructor(this: *mut CSetGroup) {
    // Destructor: free vector and map
    Z_Free((*this).m_ambientSets as *mut c_void);
    Z_Free((*this).m_setMap as *mut c_void);
}

/*
-------------------------
Free
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn CSetGroup_Free(this: *mut CSetGroup) {
    // Iterate through ambientSets and free each one
    let ambientSets = &mut *(*this).m_ambientSets;
    // Placeholder: actual iteration would require vector implementation
    // For now, we rely on the opaque vector being handled by external code

    // Do this in place of clear() so it *really* frees the memory.
    Z_Free((*this).m_ambientSets as *mut c_void);
    Z_Free((*this).m_setMap as *mut c_void);

    (*this).m_ambientSets = Z_Malloc(size_of::<std_vector_ambientSet_ptr>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_vector_ambientSet_ptr;
    (*this).m_setMap = Z_Malloc(size_of::<std_map_sstring_t_ambientSet_ptr>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_map_sstring_t_ambientSet_ptr;

    (*this).m_numSets = 0;
}

/*
-------------------------
AddSet
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn CSetGroup_AddSet(this: *mut CSetGroup, name: *const c_char) -> *mut ambientSet_t {
    // Allocate the memory
    let set = Z_Malloc(size_of::<ambientSet_t>(), TAG_AMBIENTSET, 1 as c_int) as *mut ambientSet_t;

    // Set up some defaults
    Q_strncpyz(addr_of_mut!((*set).name[0]), name, size_of::<[c_char; MAX_SET_NAME_LENGTH as usize]>() as c_int);
    (*set).loopedVolume = MAX_SET_VOLUME as c_uchar;
    (*set).masterVolume = MAX_SET_VOLUME;
    (*set).radius = 250;
    (*set).time_start = 10;
    (*set).time_end = 25;

    (*set).volRange_start = MAX_SET_VOLUME as c_uint;
    (*set).volRange_end = MAX_SET_VOLUME as c_uint;

    // Insert into vector (placeholder - actual vector ops would go here)
    // m_ambientSets->insert( m_ambientSets->end(), set );

    (*set).id = (*this).m_numSets;
    (*this).m_numSets += 1;

    // Map the name to the pointer for reference later
    // (*m_setMap)[name] = set;

    set
}

/*
-------------------------
GetSet
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn CSetGroup_GetSet_name(this: *mut CSetGroup, name: *const c_char) -> *mut ambientSet_t {
    if name.is_null() {
        return null_mut();
    }

    // Placeholder: map lookup would go here
    // map<sstring_t, ambientSet_t*>::iterator mi;
    // mi = m_setMap->find(name);
    // if (mi == m_setMap->end()) return NULL;
    // return (*mi).second;

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn CSetGroup_GetSet_ID(this: *mut CSetGroup, ID: c_int) -> *mut ambientSet_t {
    // if (m_ambientSets->empty()) return NULL;
    if ID < 0 {
        return null_mut();
    }
    if ID >= (*this).m_numSets {
        return null_mut();
    }

    // return (*m_ambientSets)[ID];
    null_mut()
}

// ============================================================================
// File Parsing
// ============================================================================

/*
-------------------------
AS_GetSetNameIDForString
-------------------------
*/
unsafe fn AS_GetSetNameIDForString(name: *const c_char) -> c_int {
    // Make sure it's valid
    if name.is_null() || *name == 0 {
        return -1;
    }

    for i in 0..NUM_AS_SETS {
        if stricmp(name, SETNAMES[i as usize]) == 0 {
            return i;
        }
    }

    -1
}

/*
-------------------------
AS_GetKeywordIDForString
-------------------------
*/
unsafe fn AS_GetKeywordIDForString(name: *const c_char) -> c_int {
    // Make sure it's valid
    if name.is_null() || *name == 0 {
        return -1;
    }

    for i in 0..NUM_AS_KEYWORDS {
        if stricmp(name, KEYWORDNAMES[i as usize]) == 0 {
            return i;
        }
    }

    -1
}

/*
-------------------------
AS_SkipLine

Skips a line in the character buffer
-------------------------
*/
unsafe fn AS_SkipLine() {
    if parsePos > parseSize {
        // needed to avoid a crash because of some OOR access that shouldn't be done
        return;
    }

    while *(parseBuffer.add(parsePos as usize)) as c_int != '\n' as c_int
        && *(parseBuffer.add(parsePos as usize)) as c_int != '\r' as c_int {
        parsePos += 1;

        if parsePos > parseSize {
            return;
        }
    }

    parsePos += 1;
}

/*
-------------------------
AS_GetTimeBetweenWaves

getTimeBetweenWaves <start> <end>
-------------------------
*/
unsafe fn AS_GetTimeBetweenWaves(set: *mut ambientSet_t) {
    let mut startTime: c_int = 0;
    let mut endTime: c_int = 0;

    // Get the data
    sscanf(parseBuffer.add(parsePos as usize),
           b"%s %d %d\0".as_ptr() as *const c_char,
           addr_of_mut!(tempBuffer[0]),
           addr_of_mut!(startTime),
           addr_of_mut!(endTime));

    // Check for swapped start / end
    if startTime > endTime {
        // WARNING: Corrected swapped start / end times in a "timeBetweenWaves" keyword
        Com_Printf(b"^3WARNING: Corrected swapped start / end times in a \"timeBetweenWaves\" keyword\n\0".as_ptr() as *const c_char);

        let swap = startTime;
        startTime = endTime;
        endTime = swap;
    }

    // Store it
    (*set).time_start = startTime as c_uint;
    (*set).time_end = endTime as c_uint;

    AS_SkipLine();
}

/*
-------------------------
AS_GetSubWaves

subWaves <directory> <wave1> <wave2> ...
-------------------------
*/
unsafe fn AS_GetSubWaves(set: *mut ambientSet_t) {
    let mut dirBuffer: [c_char; 512] = [0; 512];
    let mut waveBuffer: [c_char; 256] = [0; 256];
    let mut waveName: [c_char; 1024] = [0; 1024];

    // Get the directory for these sets
    sscanf(parseBuffer.add(parsePos as usize),
           b"%s %s\0".as_ptr() as *const c_char,
           addr_of_mut!(tempBuffer[0]),
           addr_of_mut!(dirBuffer[0]));

    // Move the pointer past these two strings
    parsePos += ((strlen(KEYWORDNAMES[SET_KEYWORD_SUBWAVES as usize]) + 1) + (strlen(addr_of!(dirBuffer[0])) + 1)) as c_int;

    // Get all the subwaves
    while parsePos <= parseSize {
        // Get the data
        sscanf(parseBuffer.add(parsePos as usize),
               b"%s\0".as_ptr() as *const c_char,
               addr_of_mut!(waveBuffer[0]));

        if (*set).numSubWaves > MAX_WAVES_PER_GROUP as c_uchar {
            // WARNING: Too many subwaves on set
            Com_Printf(b"^3WARNING: Too many subwaves on set\n\0".as_ptr() as *const c_char);
        } else {
            // Construct the wave name (pretty, huh?)
            strcpy(addr_of_mut!(waveName[0]), b"sound/\0".as_ptr() as *const c_char);
            strncat(addr_of_mut!(waveName[0]), addr_of!(dirBuffer[0]), 1024);
            strncat(addr_of_mut!(waveName[0]), b"/\0".as_ptr() as *const c_char, 512);
            strncat(addr_of_mut!(waveName[0]), addr_of!(waveBuffer[0]), 512);
            strncat(addr_of_mut!(waveName[0]), b".wav\0".as_ptr() as *const c_char, 512);

            // Place this onto the sound directory name

            // Precache the file at this point and store off the ID instead of the name
            let sfxHandle = S_RegisterSound(addr_of!(waveName[0]));
            (*set).subWaves[(*set).numSubWaves as usize] = sfxHandle;
            if sfxHandle <= 0 {
                // WARNING: Unable to load ambient sound
                Com_Printf(b"^3WARNING: Unable to load ambient sound\n\0".as_ptr() as *const c_char);
            }
            (*set).numSubWaves += 1;
        }

        // Move the pointer past this string
        parsePos += strlen(addr_of!(waveBuffer[0])) as c_int + 1;

        if *(parseBuffer.add(parsePos as usize)) as c_int == '\n' as c_int
            || *(parseBuffer.add(parsePos as usize)) as c_int == '\r' as c_int {
            break;
        }
    }

    AS_SkipLine();
}

/*
-------------------------
AS_GetLoopedWave

loopedWave <name>
-------------------------
*/
unsafe fn AS_GetLoopedWave(set: *mut ambientSet_t) {
    let mut waveBuffer: [c_char; 256] = [0; 256];
    let mut waveName: [c_char; 1024] = [0; 1024];

    // Get the looped wave name
    sscanf(parseBuffer.add(parsePos as usize),
           b"%s %s\0".as_ptr() as *const c_char,
           addr_of_mut!(tempBuffer[0]),
           addr_of_mut!(waveBuffer[0]));

    // Construct the wave name
    strcpy(addr_of_mut!(waveName[0]), b"sound/\0".as_ptr() as *const c_char);
    strncat(addr_of_mut!(waveName[0]), addr_of!(waveBuffer[0]), 1024);
    strncat(addr_of_mut!(waveName[0]), b".wav\0".as_ptr() as *const c_char, 1024);

    // Precache the file at this point and store off the ID instead of the name
    let sfxHandle = S_RegisterSound(addr_of!(waveName[0]));
    (*set).loopedWave = sfxHandle;
    if sfxHandle <= 0 {
        // WARNING: Unable to load ambient sound
        Com_Printf(b"^3WARNING: Unable to load ambient sound\n\0".as_ptr() as *const c_char);
    }

    AS_SkipLine();
}

/*
-------------------------
AS_GetVolumeRange
-------------------------
*/
unsafe fn AS_GetVolumeRange(set: *mut ambientSet_t) {
    let mut min: c_int = 0;
    let mut max: c_int = 0;

    // Get the data
    sscanf(parseBuffer.add(parsePos as usize),
           b"%s %d %d\0".as_ptr() as *const c_char,
           addr_of_mut!(tempBuffer[0]),
           addr_of_mut!(min),
           addr_of_mut!(max));

    // Check for swapped min / max
    if min > max {
        // WARNING: Corrected swapped min / max range in a "volRange" keyword
        Com_Printf(b"^3WARNING: Corrected swapped min / max range in a \"volRange\" keyword\n\0".as_ptr() as *const c_char);

        let swap = min;
        min = max;
        max = swap;
    }

    // Store the data
    (*set).volRange_start = min as c_uint;
    (*set).volRange_end = max as c_uint;

    AS_SkipLine();
}

/*
-------------------------
AS_GetRadius
-------------------------
*/
unsafe fn AS_GetRadius(set: *mut ambientSet_t) {
    // Get the data
    sscanf(parseBuffer.add(parsePos as usize),
           b"%s %d\0".as_ptr() as *const c_char,
           addr_of_mut!(tempBuffer[0]),
           addr_of_mut!((*set).radius));

    AS_SkipLine();
}

/*
-------------------------
AS_GetGeneralSet
-------------------------
*/
unsafe fn AS_GetGeneralSet(set: *mut ambientSet_t) {
    // The other parameters of the set come in a specific order
    while parsePos <= parseSize {
        let iFieldsScanned = sscanf(parseBuffer.add(parsePos as usize),
                                     b"%s\0".as_ptr() as *const c_char,
                                     addr_of_mut!(tempBuffer[0]));
        if iFieldsScanned <= 0 {
            return;
        }

        let keywordID = AS_GetKeywordIDForString(addr_of!(tempBuffer[0]));

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
                if AS_GetSetNameIDForString(addr_of!(tempBuffer[0])) == -1 {
                    // Ignore comments
                    if tempBuffer[0] as c_int == ';' as c_int {
                        return;
                    }

                    // This wasn't a set name, so it's an error
                    // WARNING: Unknown ambient set keyword
                    Com_Printf(b"^3WARNING: Unknown ambient set keyword\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        }
    }
}

/*
-------------------------
AS_GetLocalSet
-------------------------
*/
unsafe fn AS_GetLocalSet(set: *mut ambientSet_t) {
    // The other parameters of the set come in a specific order
    while parsePos <= parseSize {
        let iFieldsScanned = sscanf(parseBuffer.add(parsePos as usize),
                                     b"%s\0".as_ptr() as *const c_char,
                                     addr_of_mut!(tempBuffer[0]));
        if iFieldsScanned <= 0 {
            return;
        }

        let keywordID = AS_GetKeywordIDForString(addr_of!(tempBuffer[0]));

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
                if AS_GetSetNameIDForString(addr_of!(tempBuffer[0])) == -1 {
                    // Ignore comments
                    if tempBuffer[0] as c_int == ';' as c_int {
                        return;
                    }

                    // This wasn't a set name, so it's an error
                    // WARNING: Unknown ambient set keyword
                    Com_Printf(b"^3WARNING: Unknown ambient set keyword\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        }
    }
}

/*
-------------------------
AS_GetBModelSet
-------------------------
*/
unsafe fn AS_GetBModelSet(set: *mut ambientSet_t) {
    // The other parameters of the set come in a specific order
    while parsePos <= parseSize {
        let iFieldsScanned = sscanf(parseBuffer.add(parsePos as usize),
                                     b"%s\0".as_ptr() as *const c_char,
                                     addr_of_mut!(tempBuffer[0]));
        if iFieldsScanned <= 0 {
            return;
        }

        let keywordID = AS_GetKeywordIDForString(addr_of!(tempBuffer[0]));

        // Find and parse the keyword info
        match keywordID {
            SET_KEYWORD_SUBWAVES => {
                AS_GetSubWaves(set);
            }
            _ => {
                // Check to see if we've finished this group
                if AS_GetSetNameIDForString(addr_of!(tempBuffer[0])) == -1 {
                    // Ignore comments
                    if tempBuffer[0] as c_int == ';' as c_int {
                        return;
                    }

                    // This wasn't a set name, so it's an error
                    // WARNING: Unknown ambient set keyword
                    Com_Printf(b"^3WARNING: Unknown ambient set keyword\n\0".as_ptr() as *const c_char);
                }

                return;
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
unsafe fn AS_ParseSet(setID: c_int, sg: *mut CSetGroup) -> c_int {
    // Make sure we're not overstepping the name array
    if setID > NUM_AS_SETS {
        return 0; // qfalse
    }

    // Reset the pointers for this run through
    parsePos = 0;

    let name = SETNAMES[setID as usize];

    // Iterate through the whole file and find every occurance of a set
    while parsePos <= parseSize {
        // Check for a valid set group
        if strncmp(parseBuffer.add(parsePos as usize), name, strlen(name)) == 0 {
            // Update the debug info
            numSets += 1;

            // Push past the set specifier and on to the name
            parsePos += strlen(name) as c_int + 1; // Also take the following space out

            // Get the set name (this MUST be first)
            sscanf(parseBuffer.add(parsePos as usize),
                   b"%s\0".as_ptr() as *const c_char,
                   addr_of_mut!(tempBuffer[0]));
            AS_SkipLine();

            // Test the string against the precaches
            if tempBuffer[0] != 0 {
                // Not in our precache listings, so skip it
                if pMap.is_null() || {
                    // Placeholder: map lookup would go here
                    // (pMap->find((const char*)&tempBuffer) == pMap->end())
                    true  // For now, assume not found
                } {
                    // continue;
                } else {
                    // Create a new set
                    let set = (*sg).AddSet(addr_of!(tempBuffer[0]));

                    // Run the function to parse the data out
                    (PARSEFUNCS[setID as usize])(set);
                    // continue;
                }
            }
        }

        // If not found on this line, go down another and check again
        AS_SkipLine();
    }

    1 // qtrue
}

/*
-------------------------
AS_ParseHeader

Parses the directory information out of the beginning of the file
-------------------------
*/
unsafe fn AS_ParseHeader() {
    let mut typeBuffer: [c_char; 128] = [0; 128];

    while parsePos <= parseSize {
        sscanf(parseBuffer.add(parsePos as usize),
               b"%s\0".as_ptr() as *const c_char,
               addr_of_mut!(tempBuffer[0]));

        let keywordID = AS_GetKeywordIDForString(addr_of!(tempBuffer[0]));

        match keywordID {
            SET_KEYWORD_TYPE => {
                sscanf(parseBuffer.add(parsePos as usize),
                       b"%s %s\0".as_ptr() as *const c_char,
                       addr_of_mut!(tempBuffer[0]),
                       addr_of_mut!(typeBuffer[0]));

                if stricmp(addr_of!(typeBuffer[0]), b"ambientSet\0".as_ptr() as *const c_char) == 0 {
                    return;
                }
                Com_Error(ERR_DROP, b"AS_ParseHeader: Set type is not a valid set type!\n\0".as_ptr() as *const c_char);
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

/*
-------------------------
AS_ParseFile

Opens and parses a sound set file
-------------------------
*/
unsafe fn AS_ParseFile(filename: *const c_char, sg: *mut CSetGroup) -> c_int {
    // Open the file and read the information from it
    parseSize = FS_ReadFile(filename, addr_of_mut!(parseBuffer) as *mut *mut c_void);

    if parseSize <= 0 {
        return 0; // qfalse
    }

    // Parse the directory information out of the file
    AS_ParseHeader();

    // Parse all the relevent sets out of it
    for i in 0..NUM_AS_SETS {
        AS_ParseSet(i, sg);
    }

    // Free the memory and close the file
    FS_FreeFile(parseBuffer as *mut c_void);

    1 // qtrue
}

// ============================================================================
// Main code
// ============================================================================

/*
-------------------------
AS_Init

Loads the ambient sound sets and prepares to play them when needed
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn AS_Init() {
    if aSets.is_null() {
        numSets = 0;

        pMap = Z_Malloc(size_of::<std_map_sstring_uchar>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_map_sstring_uchar;

        // Setup the structure
        aSets = Z_Malloc(size_of::<CSetGroup>(), TAG_AMBIENTSET, 1 as c_int) as *mut CSetGroup;
        CSetGroup_CSetGroup(aSets);
        (*aSets).Init();
    }
}

/*
-------------------------
AS_AddPrecacheEntry
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn AS_AddPrecacheEntry(name: *const c_char) {
    if stricmp(name, b"#clear\0".as_ptr() as *const c_char) == 0 {
        // pMap->clear();
    } else {
        // (*pMap)[name] = 1;
    }
}

/*
-------------------------
AS_ParseSets

Called on the client side to load and precache all the ambient sound sets
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn AS_ParseSets() {
    AS_Init();

    // Parse all the sets
    if AS_ParseFile(AMBIENT_SET_FILENAME.as_ptr() as *const c_char, aSets) == 0 { // qfalse
        Com_Error(ERR_FATAL, b"^1ERROR: Couldn't load ambient sound sets\n\0".as_ptr() as *const c_char);
    }

    // Com_Printf("AS_ParseFile: Loaded %d of %d ambient set(s)\n", pMap.size(), numSets);

    let mut iErrorsOccured = 0;
    // for (namePrecache_m::iterator it = pMap->begin(); it != pMap->end(); ++it)
    // {
    //     const char* str = (*it).first.c_str();
    //     ambientSet_t *aSet = aSets->GetSet(str);
    //     if (!aSet)
    //     {
    //         // I print these red instead of yellow because they're going to cause an ERR_DROP if they occur
    //         Com_Printf(b"^1ERROR: AS_ParseSets: Unable to find ambient soundset\n\0".as_ptr() as *const c_char);
    //         iErrorsOccured++;
    //     }
    // }

    if iErrorsOccured > 0 {
        Com_Error(ERR_DROP, b"Missing sound sets! (see above)\n\0".as_ptr() as *const c_char);
    }

    // //Done with the precache info, it will be rebuilt on a restart
    // //pMap->clear();	// do NOT do this here now
}

/*
-------------------------
AS_Free

Frees up the ambient sound system
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn AS_Free() {
    if !aSets.is_null() {
        (*aSets).Free();
        Z_Free(aSets as *mut c_void);
        aSets = null_mut();

        currentSet = -1;
        oldSet = -1;

        currentSetTime = 0;
        oldSetTime = 0;

        numSets = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn AS_FreePartial() {
    if !aSets.is_null() {
        (*aSets).Free();
        currentSet = -1;
        oldSet = -1;

        currentSetTime = 0;
        oldSetTime = 0;

        numSets = 0;

        Z_Free(pMap as *mut c_void);
        pMap = Z_Malloc(size_of::<std_map_sstring_uchar>(), TAG_AMBIENTSET, 1 as c_int) as *mut std_map_sstring_uchar;
    }
}

// ============================================================================
// Sound code
// ============================================================================

/*
-------------------------
AS_UpdateSetVolumes

Fades volumes up or down depending on the action being taken on them.
-------------------------
*/
unsafe fn AS_UpdateSetVolumes() {
    // Get the sets and validate them
    let current = (*aSets).GetSet_ID(currentSet);

    if current.is_null() {
        return;
    }

    if (*current).masterVolume < MAX_SET_VOLUME {
        let deltaTime = cls.realtime - (*current).fadeTime;
        let scale = (deltaTime as f64) / (crossDelay as f64);
        (*current).masterVolume = (scale * (MAX_SET_VOLUME as f64)) as c_int;
    }

    if (*current).masterVolume > MAX_SET_VOLUME {
        (*current).masterVolume = MAX_SET_VOLUME;
    }

    // Only update the old set if it's still valid
    if oldSet == -1 {
        return;
    }

    let old = (*aSets).GetSet_ID(oldSet);

    if old.is_null() {
        return;
    }

    // Update the volumes
    if (*old).masterVolume > 0 {
        let deltaTime = cls.realtime - (*old).fadeTime;
        let scale = (deltaTime as f64) / (crossDelay as f64);
        (*old).masterVolume = MAX_SET_VOLUME - (scale * (MAX_SET_VOLUME as f64)) as c_int;
    }

    if (*old).masterVolume <= 0 {
        (*old).masterVolume = 0;
        oldSet = -1;
    }
}

/*
-------------------------
S_UpdateCurrentSet

Does internal maintenance to keep track of changing sets.
-------------------------
*/
unsafe fn AS_UpdateCurrentSet(id: c_int) {
    // Check for a change
    if id != currentSet {
        // This is new, so start the fading
        oldSet = currentSet;
        currentSet = id;

        let old = (*aSets).GetSet_ID(oldSet);
        let current = (*aSets).GetSet_ID(currentSet);
        // Ste, I just put this null check in for now, not sure if there's a more graceful way to exit this function - dmv
        if current.is_null() {
            return;
        }
        if !old.is_null() {
            (*old).masterVolume = MAX_SET_VOLUME;
            (*old).fadeTime = cls.realtime;
        }

        (*current).masterVolume = 0;

        // Set the fading starts
        (*current).fadeTime = cls.realtime;
    }

    // Update their volumes if fading
    AS_UpdateSetVolumes();
}

/*
-------------------------
AS_PlayLocalSet

Plays a local set taking volume and subwave playing into account.
Alters lastTime to reflect the time updates.
-------------------------
*/
unsafe fn AS_PlayLocalSet(listener_origin: vec3_t, origin: vec3_t, set: *mut ambientSet_t, entID: c_int, lastTime: *mut c_int) {
    let mut dir: vec3_t = [0.0; 3];
    let time = cls.realtime;

    // Make sure it's valid
    if set.is_null() {
        return;
    }

    VectorSubtract(origin, listener_origin, &mut dir);
    let dist = VectorLength(&dir);

    // Determine the volume based on distance (NOTE: This sits on top of what SpatializeOrigin does)
    let distScale = if dist < (((*set).radius as f64) * 0.5) {
        1.0
    } else {
        (((*set).radius as f64) - dist) / (((*set).radius as f64) * 0.5)
    };
    let volume = if distScale > 1.0 || distScale < 0.0 {
        0 as c_uchar
    } else {
        (((*set).masterVolume as f64) * distScale) as c_uchar
    };

    // Add the looping sound
    if (*set).loopedWave != 0 {
        S_AddAmbientLoopingSound(origin, volume, (*set).loopedWave);
    }

    // Check the time to start another one-shot subwave
    if (time - *lastTime) < ((Q_irand((*set).time_start as c_int, (*set).time_end as c_int)) * 1000) {
        return;
    }

    // Update the time
    *lastTime = time;

    // Scale the volume ranges for the subwaves based on the overall master volume
    let volScale = (volume as f64) / (MAX_SET_VOLUME as f64);
    let volume_subwave = Q_irand(((volScale * ((*set).volRange_start as f64)) as c_int),
                                 ((volScale * ((*set).volRange_end as f64)) as c_int)) as c_uchar;

    // Add the random subwave
    if (*set).numSubWaves > 0 {
        let idx = Q_irand(0, ((*set).numSubWaves as c_int) - 1) as usize;
        S_StartAmbientSound(origin, entID, volume_subwave, (*set).subWaves[idx]);
    }
}

/*
-------------------------
AS_PlayAmbientSet

Plays an ambient set taking volume and subwave playing into account.
Alters lastTime to reflect the time updates.
-------------------------
*/
unsafe fn AS_PlayAmbientSet(origin: vec3_t, set: *mut ambientSet_t, lastTime: *mut c_int) {
    let time = cls.realtime;

    // Make sure it's valid
    if set.is_null() {
        return;
    }

    // Add the looping sound
    if (*set).loopedWave != 0 {
        S_AddAmbientLoopingSound(origin, (*set).masterVolume as c_uchar, (*set).loopedWave);
    }

    // Check the time to start another one-shot subwave
    if (time - *lastTime) < ((Q_irand((*set).time_start as c_int, (*set).time_end as c_int)) * 1000) {
        return;
    }

    // Update the time
    *lastTime = time;

    // Scale the volume ranges for the subwaves based on the overall master volume
    let volScale = ((*set).masterVolume as f64) / (MAX_SET_VOLUME as f64);
    let mut volume = Q_irand(((volScale * ((*set).volRange_start as f64)) as c_int),
                             ((volScale * ((*set).volRange_end as f64)) as c_int)) as c_uchar;

    // Allow for softer noises than the masterVolume, but not louder
    if volume > (*set).masterVolume as c_uchar {
        volume = (*set).masterVolume as c_uchar;
    }

    // Add the random subwave
    if (*set).numSubWaves > 0 {
        let idx = Q_irand(0, ((*set).numSubWaves as c_int) - 1) as usize;
        S_StartAmbientSound(origin, 0, volume, (*set).subWaves[idx]);
    }
}

/*
-------------------------
S_UpdateAmbientSet

Does maintenance and plays the ambient sets (two if crossfading)
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn S_UpdateAmbientSet(name: *const c_char, origin: vec3_t) {
    let current;
    let old;
    let set = (*aSets).GetSet(name);

    if set.is_null() {
        return;
    }

    // Update the current and old set for crossfading
    AS_UpdateCurrentSet((*set).id);

    current = (*aSets).GetSet_ID(currentSet);
    old = (*aSets).GetSet_ID(oldSet);

    if !current.is_null() {
        AS_PlayAmbientSet(origin, set, addr_of_mut!(currentSetTime));
    }

    if !old.is_null() {
        AS_PlayAmbientSet(origin, old, addr_of_mut!(oldSetTime));
    }
}

/*
-------------------------
S_AddLocalSet
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn S_AddLocalSet(name: *const c_char, listener_origin: vec3_t, origin: vec3_t, entID: c_int, time: c_int) -> c_int {
    let set;
    let mut currentTime = 0;

    set = (*aSets).GetSet(name);

    if set.is_null() {
        return cls.realtime;
    }

    currentTime = time;

    AS_PlayLocalSet(listener_origin, origin, set, entID, addr_of_mut!(currentTime));

    currentTime
}

/*
-------------------------
AS_GetBModelSound
-------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn AS_GetBModelSound(name: *const c_char, stage: c_int) -> sfxHandle_t {
    let set;

    set = (*aSets).GetSet(name);

    if set.is_null() {
        return -1;
    }

    // Stage must be within a valid range
    if stage > ((*set).numSubWaves as c_int - 1) || stage < 0 {
        return -1;
    }

    (*set).subWaves[stage as usize]
}
