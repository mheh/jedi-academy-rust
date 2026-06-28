//! Mechanical port of `code/RMG/RM_Terrain.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================
//
// These types are declared here to allow this file to compile structurally.
// Full definitions exist in the oracle but have not yet been ported.
// Porting these types is out of scope for this file.

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
/// Represents the landscape/terrain mesh for the map.
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// Stub for `byte* CCMLandScape::GetHeightMap()`.
    pub fn GetHeightMap(&self) -> *mut u8 {
        core::ptr::null_mut()
    }

    /// Stub for `int CCMLandScape::GetRealWidth()`.
    pub fn GetRealWidth(&self) -> c_int {
        0
    }

    /// Stub for `int CCMLandScape::GetRealHeight()`.
    pub fn GetRealHeight(&self) -> c_int {
        0
    }

    /// Stub for `int CCMLandScape::GetBlockCount()`.
    pub fn GetBlockCount(&self) -> c_int {
        0
    }

    /// Stub for `int CCMLandScape::GetBlockWidth()`.
    pub fn GetBlockWidth(&self) -> c_int {
        0
    }

    /// Stub for `int CCMLandScape::GetBlockHeight()`.
    pub fn GetBlockHeight(&self) -> c_int {
        0
    }

    /// Stub for `vec3_t CCMLandScape::GetSize()`.
    pub fn GetSize(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }

    /// Stub for `vec3_t CCMLandScape::GetMins()`.
    pub fn GetMins(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }

    /// Stub for `CArea* CCMLandScape::GetFirstArea()`.
    pub fn GetFirstArea(&self) -> *mut CArea {
        core::ptr::null_mut()
    }

    /// Stub for `CArea* CCMLandScape::GetNextArea()`.
    pub fn GetNextArea(&self) -> *mut CArea {
        core::ptr::null_mut()
    }

    /// Stub for `float CCMLandScape::GetPatchScalarSize()`.
    pub fn GetPatchScalarSize(&self) -> f32 {
        0.0
    }

    /// Stub for `int CCMLandScape::GetTerxels()`.
    pub fn GetTerxels(&self) -> c_int {
        0
    }

    /// Stub for `float CCMLandScape::GetPatchWidth()`.
    pub fn GetPatchWidth(&self) -> f32 {
        0.0
    }

    /// Stub for `float CCMLandScape::GetPatchHeight()`.
    pub fn GetPatchHeight(&self) -> f32 {
        0.0
    }

    /// Stub for `float CCMLandScape::GetWorldHeight(vec3_t pos, vec3_t *bounds, bool slope)`.
    pub fn GetWorldHeight(&self, _pos: [f32; 3], _bounds: *mut [f32; 3], _slope: bool) -> f32 {
        0.0
    }

    /// Stub for `float CCMLandScape::GetWaterHeight()`.
    pub fn GetWaterHeight(&self) -> f32 {
        0.0
    }

    /// Stub for `float CCMLandScape::CalcWorldHeight(int level)`.
    pub fn CalcWorldHeight(&self, _level: c_int) -> f32 {
        0.0
    }

    /// Stub for `bool CCMLandScape::AreaCollision(CArea *area, int *areaTypes, int typeCount)`.
    pub fn AreaCollision(&self, _area: *mut CArea, _areaTypes: *mut c_int, _typeCount: c_int) -> bool {
        false
    }

    /// Stub for `int CCMLandScape::irand(int min, int max)`.
    pub fn irand(&self, _min: c_int, _max: c_int) -> c_int {
        0
    }

    /// Stub for `float CCMLandScape::flrand(float min, float max)`.
    pub fn flrand(&self, _min: f32, _max: f32) -> f32 {
        0.0
    }
}

/// Stub for unported `class CRandomModel` (RM_Terrain.h).
/// Represents a random model that can be spawned on terrain.
pub struct CRandomModel {
    _opaque: [u8; 0],
}

impl CRandomModel {
    /// Stub for `void CRandomModel::SetModel(const char *model)`.
    pub fn SetModel(&mut self, _model: *const c_char) {}

    /// Stub for `void CRandomModel::SetFrequency(float frequency)`.
    pub fn SetFrequency(&mut self, _frequency: f32) {}

    /// Stub for `void CRandomModel::SetMinScale(float scale)`.
    pub fn SetMinScale(&mut self, _scale: f32) {}

    /// Stub for `void CRandomModel::SetMaxScale(float scale)`.
    pub fn SetMaxScale(&mut self, _scale: f32) {}

    /// Stub for `const bool CRandomModel::GetModel()`.
    pub fn GetModel(&self) -> bool {
        false
    }

    /// Stub for `float CRandomModel::GetFrequency()`.
    pub fn GetFrequency(&self) -> f32 {
        0.0
    }

    /// Stub for `float CRandomModel::GetMinScale()`.
    pub fn GetMinScale(&self) -> f32 {
        0.0
    }

    /// Stub for `float CRandomModel::GetMaxScale()`.
    pub fn GetMaxScale(&self) -> f32 {
        0.0
    }

    /// Stub for `const char* CRandomModel::GetModelName()`.
    pub fn GetModelName(&self) -> *const c_char {
        core::ptr::null()
    }
}

/// Stub for unported `class CCMPatch` (cm_landscape.h).
/// Represents a patch of terrain.
pub struct CCMPatch {
    _opaque: [u8; 0],
}

impl CCMPatch {
    /// Stub for `int CCMPatch::GetHeightMapX()`.
    pub fn GetHeightMapX(&self) -> c_int {
        0
    }

    /// Stub for `int CCMPatch::GetHeightMapY()`.
    pub fn GetHeightMapY(&self) -> c_int {
        0
    }

    /// Stub for `int CCMPatch::GetHeight(int index)`.
    pub fn GetHeight(&self, _index: c_int) -> c_int {
        0
    }

    /// Stub for `vec3_t CCMPatch::GetMins()`.
    pub fn GetMins(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }
}

/// Stub for unported `class CArea` (cm_landscape.h).
/// Represents an area on the map.
pub struct CArea {
    _opaque: [u8; 0],
}

impl CArea {
    /// Stub for `int CArea::GetType()`.
    pub fn GetType(&self) -> c_int {
        0
    }

    /// Stub for `vec3_t CArea::GetPosition()`.
    pub fn GetPosition(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }

    /// Stub for `float CArea::GetRadius()`.
    pub fn GetRadius(&self) -> f32 {
        0.0
    }

    /// Stub for `void CArea::Init(vec3_t pos, float radius)`.
    pub fn Init(&mut self, _pos: [f32; 3], _radius: f32) {}
}

/// Stub for unported `class CGPValue` (GenericParser2.h).
/// Represents a key-value pair in a parsed file.
pub struct CGPValue {
    _opaque: [u8; 0],
}

impl CGPValue {
    /// Stub for `const char* CGPValue::GetName()`.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `const char* CGPValue::GetTopValue()`.
    pub fn GetTopValue(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `CGPValue* CGPValue::GetNext()`.
    pub fn GetNext(&self) -> *mut CGPValue {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds a group of configuration settings.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::GetName()`.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }

    /// Stub for `CGPGroup* CGPGroup::GetSubGroups()`.
    pub fn GetSubGroups(&self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }

    /// Stub for `CGPValue* CGPGroup::GetPairs()`.
    pub fn GetPairs(&self) -> *mut CGPValue {
        core::ptr::null_mut()
    }

    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *default_val)`.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        default_val
    }

    /// Stub for `CGPGroup* CGPGroup::GetNext()`.
    pub fn GetNext(&self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CGenericParser2` (GenericParser2.h).
/// Parses generic text files into a tree of groups and values.
pub struct CGenericParser2 {
    _opaque: [u8; 0],
}

impl CGenericParser2 {
    /// Stub for `CGPGroup* CGenericParser2::GetBaseParseGroup()`.
    pub fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        core::ptr::null_mut()
    }
}

/// Stub for unported `refEntity_t` (tr_types.h).
/// Reference entity structure for rendering.
#[repr(C)]
pub struct refEntity_t {
    pub hModel: *mut c_void,
    pub frame: c_int,
}

impl Default for refEntity_t {
    fn default() -> Self {
        refEntity_t {
            hModel: core::ptr::null_mut(),
            frame: 0,
        }
    }
}

pub type byte = u8;

// ============================================================================
// extern "C" functions from libc and engine
// ============================================================================

extern "C" {
    /// C standard library function to convert a string to a long integer.
    fn atol(s: *const c_char) -> c_int;

    /// C standard library function to convert a string to a double.
    fn atof(s: *const c_char) -> f64;

    /// C standard library function to convert a string to an unsigned long.
    fn strtoul(s: *const c_char, endp: *mut *const c_char, base: c_int) -> core::ffi::c_ulong;

    /// C standard library function to compare strings (case-insensitive).
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// C standard library function to find a substring.
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;

    /// C standard library function to get string length.
    fn strlen(s: *const c_char) -> core::ffi::c_size_t;

    /// C standard library function to set memory to a value.
    fn memset(s: *mut c_void, c: c_int, n: core::ffi::c_size_t) -> *mut c_void;

    /// Quake engine function to allocate memory with a tag.
    fn Z_Malloc(size: c_int, tag: c_int, qfalse: c_int) -> *mut c_void;

    /// Quake engine function to free allocated memory.
    fn Z_Free(ptr: *mut c_void);

    /// Quake engine function to format a string (like sprintf).
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);

    /// Quake engine function to parse debug output.
    fn Com_DPrintf(fmt: *const c_char, ...);

    /// Quake engine function to print output.
    fn Com_Printf(fmt: *const c_char, ...);

    /// Quake engine function to parse a text file.
    fn Com_ParseTextFile(filename: *const c_char, parser: &mut CGenericParser2) -> c_int;

    /// Quake engine function to destroy parsed text file.
    fn Com_ParseTextFileDestroy(parser: CGenericParser2);

    /// Quake engine function to get value from info string.
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;

    /// Quake engine function to clamp an integer value.
    fn Com_Clamp(min: c_int, max: c_int, value: c_int) -> c_int;

    /// Engine function to load a data image.
    fn R_LoadDataImage(name: *const c_char, pic: *mut *mut u8, width: *mut c_int, height: *mut c_int);

    /// Engine function to invert an image.
    fn R_InvertImage(data: *mut u8, width: c_int, height: c_int, depth: c_int);

    /// Engine function to resample an image.
    fn R_Resample(
        source: *mut u8,
        swidth: c_int,
        sheight: c_int,
        dest: *mut u8,
        dwidth: c_int,
        dheight: c_int,
        components: c_int,
    );

    /// Engine function to get model bounds.
    fn RE_GetModelBounds(refEnt: *mut refEntity_t, bounds1: *mut [f32; 3], bounds2: *mut [f32; 3]);

    /// Engine function to register a model.
    fn re_RegisterModel(name: *const c_char) -> *mut c_void;

    /// Collision/terrain function to iterate over patches.
    fn CM_TerrainPatchIterate(
        land: *mut CCMLandScape,
        callback: extern "C" fn(*mut CCMPatch, *mut c_void),
        userdata: *mut c_void,
    );

    /// Collision/terrain function to iterate circularly.
    fn CM_CircularIterate(
        density: *mut u8,
        width: c_int,
        height: c_int,
        x: c_int,
        y: c_int,
        startRadius: c_int,
        endRadius: c_int,
        userdata: *mut c_void,
        callback: extern "C" fn(*mut u8, f32, *mut c_int),
    );

    /// Server trace function for collision queries.
    fn SV_Trace(
        results: *mut TraceResult,
        start: *const [f32; 3],
        mins: *const [f32; 3],
        maxs: *const [f32; 3],
        end: *const [f32; 3],
        passent: c_int,
        contentmask: c_int,
    );

    /// Quake engine global cmg (collision manager global).
    static mut cmg: CMGlobal;
}

/// Stub for unported collision manager global.
pub struct CMGlobal {
    pub landScape: *mut CCMLandScape,
}

/// Stub for unported `trace_t` structure from collision module.
#[repr(C)]
pub struct TraceResult {
    pub surfaceFlags: c_int,
    pub startsolid: c_int,
}

// ============================================================================
// Constants
// ============================================================================

const MAX_QPATH: c_int = 64;
const MAX_RANDOM_MODELS: usize = 8;
const HEIGHT_RESOLUTION: c_int = 256;
const TAG_R_TERRAIN: c_int = 32;

// Surface flags
const SURF_NOMISCENTS: c_int = 0x4000;

// Collision contents/masks
const CONTENTS_SOLID: c_int = 0x0001;
const CONTENTS_PLAYERCLIP: c_int = 0x0010;
const CONTENTS_BODY: c_int = 0x0020;
const CONTENTS_TERRAIN: c_int = 0x0100;

// Terrain area types
const AT_GROUP: c_int = 1;

// ============================================================================
// Global statics
// ============================================================================

/// Static pointer to the landscape instance.
static mut rm_landscape: *mut CRMLandScape = core::ptr::null_mut();

/// Static reference to origin landscape used in callback.
static mut origin_land: *mut CCMLandScape = core::ptr::null_mut();

// ============================================================================
// CCGHeightDetails class
// ============================================================================

/// Height details structure holding random models for a specific height level.
pub struct CCGHeightDetails {
    /// Array of random models
    mModels: [CRandomModel; MAX_RANDOM_MODELS],
    /// Number of models stored
    mNumModels: c_int,
    /// Sum of all model frequencies
    mTotalFrequency: f32,
}

impl CCGHeightDetails {
    /// Constructor - initializes a new CCGHeightDetails instance.
    pub fn new() -> Self {
        CCGHeightDetails {
            mModels: unsafe { core::mem::zeroed() },
            mNumModels: 0,
            mTotalFrequency: 0.0,
        }
    }

    /// Add a random model to this height level.
    /// If the number of models exceeds MAX_RANDOM_MODELS, the model is ignored.
    pub fn AddModel(&mut self, hd: &CRandomModel) {
        if (self.mNumModels as usize) < MAX_RANDOM_MODELS {
            self.mTotalFrequency += hd.GetFrequency();
            self.mModels[self.mNumModels as usize] = *hd;
            self.mNumModels += 1;
        }
    }

    /// Get the number of models at this height level.
    pub fn GetNumModels(&self) -> c_int {
        self.mNumModels
    }

    /// Get the average frequency of models at this height.
    pub fn GetAverageFrequency(&self) -> c_int {
        if self.mNumModels > 0 {
            (self.mTotalFrequency / self.mNumModels as f32) as c_int
        } else {
            0
        }
    }

    /// Select a random model from this height level using weighted random selection.
    pub fn GetRandomModel(&self, land: *mut CCMLandScape) -> *mut CRandomModel {
        unsafe {
            let land_ref = &*land;
            let mut seek = land_ref.irand(0, (self.mTotalFrequency as c_int));
            for i in 0..(self.mNumModels as usize) {
                seek -= (self.mModels[i].GetFrequency() as c_int);
                if seek <= 0 {
                    return (self.mModels.as_ptr() as *mut CRandomModel).add(i);
                }
            }
            // Original code has assert(0) here
            core::ptr::null_mut()
        }
    }
}

// ============================================================================
// CRMLandScape class
// ============================================================================

/// Random landscape/terrain manager for spawning miscellaneous entities.
pub struct CRMLandScape {
    /// Pointer to the common/landscape instance
    common: *mut CCMLandScape,
    /// Density map for entity spawning
    mDensityMap: *mut byte,
    /// Height details for each height level
    mHeightDetails: [CCGHeightDetails; HEIGHT_RESOLUTION as usize],
    /// Count of spawned models
    mModelCount: c_int,
}

impl CRMLandScape {
    /// Constructor - initializes a new CRMLandScape instance.
    pub fn new() -> Self {
        CRMLandScape {
            common: core::ptr::null_mut(),
            mDensityMap: core::ptr::null_mut(),
            mHeightDetails: unsafe { core::mem::zeroed() },
            mModelCount: 0,
        }
    }

    /// Set the common landscape reference.
    pub fn SetCommon(&mut self, landscape: *mut CCMLandScape) {
        self.common = landscape;
    }

    /// Get the common landscape reference.
    pub fn GetCommon(&self) -> *mut CCMLandScape {
        self.common
    }

    /// Clear the model count.
    pub fn ClearModelCount(&mut self) {
        self.mModelCount = 0;
    }

    /// Get the total count of spawned models.
    pub fn GetModelCount(&self) -> c_int {
        self.mModelCount
    }

    /// Add a random model for a specific height range.
    /// The model will be available for spawning at heights between `height` and `maxheight`.
    pub fn AddModel(&mut self, height: c_int, mut maxheight: c_int, hd: &CRandomModel) {
        let mut i: c_int;

        if maxheight > HEIGHT_RESOLUTION {
            maxheight = HEIGHT_RESOLUTION;
        }

        i = height;
        while hd.GetModel() && (i < maxheight) {
            self.mHeightDetails[i as usize].AddModel(hd);
            i += 1;
        }
    }

    /// Load miscellaneous entity definitions from a miscent file.
    pub fn LoadMiscentDef(&mut self, td: *const c_char) {
        unsafe {
            let mut miscentDef: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut parse = CGenericParser2 { _opaque: [0; 0] };
            let mut basegroup: *mut CGPGroup;
            let mut classes: *mut CGPGroup;
            let mut items: *mut CGPGroup;
            let mut model: *mut CGPGroup;
            let mut pair: *mut CGPValue;

            Com_sprintf(
                miscentDef.as_mut_ptr(),
                MAX_QPATH,
                b"ext_data/RMG/%s.miscents\0".as_ptr() as *const c_char,
                Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
            );
            Com_DPrintf(
                b"CG_Terrain: Loading and parsing miscentDef %s.....\n\0".as_ptr() as *const c_char,
                Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
            );

            if Com_ParseTextFile(miscentDef.as_ptr(), &mut parse) == 0 {
                Com_sprintf(
                    miscentDef.as_mut_ptr(),
                    MAX_QPATH,
                    b"ext_data/arioche/%s.miscents\0".as_ptr() as *const c_char,
                    Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
                );
                if Com_ParseTextFile(miscentDef.as_ptr(), &mut parse) == 0 {
                    Com_Printf(b"Could not open %s\n\0".as_ptr() as *const c_char, miscentDef.as_ptr());
                    return;
                }
            }
            // The whole file....
            basegroup = parse.GetBaseParseGroup();

            // The root { } struct
            classes = (*basegroup).GetSubGroups();
            while !classes.is_null() {
                items = (*classes).GetSubGroups();
                while !items.is_null() {
                    if stricmp((*items).GetName(), b"miscent\0".as_ptr() as *const c_char) == 0 {
                        let height: c_int;
                        let maxheight: c_int;

                        // Height must exist - the rest are optional
                        height = atol((*items).FindPairValue(
                            b"height\0".as_ptr() as *const c_char,
                            b"0\0".as_ptr() as *const c_char,
                        ));
                        maxheight = atol((*items).FindPairValue(
                            b"maxheight\0".as_ptr() as *const c_char,
                            b"255\0".as_ptr() as *const c_char,
                        ));

                        model = (*items).GetSubGroups();
                        while !model.is_null() {
                            if stricmp((*model).GetName(), b"model\0".as_ptr() as *const c_char) == 0 {
                                let mut hd = CRandomModel { _opaque: [0; 0] };

                                // Set defaults
                                hd.SetModel(b"\0".as_ptr() as *const c_char);
                                hd.SetFrequency(1.0);
                                hd.SetMinScale(1.0);
                                hd.SetMaxScale(1.0);

                                pair = (*model).GetPairs();
                                while !pair.is_null() {
                                    if stricmp((*pair).GetName(), b"name\0".as_ptr() as *const c_char) == 0 {
                                        hd.SetModel((*pair).GetTopValue());
                                    } else if stricmp((*pair).GetName(), b"frequency\0".as_ptr() as *const c_char)
                                        == 0
                                    {
                                        hd.SetFrequency(atof((*pair).GetTopValue()) as f32);
                                    } else if stricmp(
                                        (*pair).GetName(),
                                        b"minscale\0".as_ptr() as *const c_char,
                                    ) == 0
                                    {
                                        hd.SetMinScale(atof((*pair).GetTopValue()) as f32);
                                    } else if stricmp(
                                        (*pair).GetName(),
                                        b"maxscale\0".as_ptr() as *const c_char,
                                    ) == 0
                                    {
                                        hd.SetMaxScale(atof((*pair).GetTopValue()) as f32);
                                    }
                                    pair = (*pair).GetNext() as *mut CGPValue;
                                }
                                self.AddModel(height, maxheight, &hd);
                            }
                            model = (*model).GetNext() as *mut CGPGroup;
                        }
                    }
                    items = (*items).GetNext() as *mut CGPGroup;
                }
                classes = (*classes).GetNext() as *mut CGPGroup;
            }
            Com_ParseTextFileDestroy(parse);
        }
    }

    /// Create a random density map based on height variation.
    pub fn CreateRandomDensityMap(&mut self, density: *mut byte, width: c_int, height: c_int, _seed: c_int) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let mut count: c_int;
            let area: *mut CArea;
            let mut derxelSize: [f32; 3] = [0.0; 3];
            let mut pos: [f32; 3] = [0.0; 3];
            let mut dmappos: [c_int; 3] = [0; 3];
            let hm_map = (*self.common).GetHeightMap();
            let hm_width = (*self.common).GetRealWidth();
            let hm_height = (*self.common).GetRealHeight();
            let mut xpos: c_int;
            let mut ypos: c_int;
            let mut dx: c_int;
            let mut dy: c_int;
            let mut densityPos = density;
            let mut foundUneven: bool;

            // Init to linear spread
            memset(density as *mut c_void, 0, (width * height) as core::ffi::c_size_t);

            /*	// Make more prevalent towards the edges
            border = Com_Clamp(6, 12, (width + height) >> 4);

            for(i = 0; i < border; i++)
            {
                inc = (border - i + 1) * 9;

                // Top line
                work = density + i + (i * width);
                for(x = i; x < width - i; x++, work++)
                {
                    *work += (byte)common->irand(inc >> 1, inc);
                }

                // Left and right edges
                work = density + i + ((i + 1) * width);
                work2 = density + (width - i) + ((i + 1) * width);
                for(y = i + 1; y < height - i - 2; y++, work += width, work2 += width)
                {
                    *work += (byte)common->irand(inc >> 1, inc);
                    *work2 += (byte)common->irand(inc >> 1, inc);
                }

                // Bottom line
                work = density + i + ((height - i - 1) * width);
                for(x = i; x < width - i; x++, work++)
                {
                    *work += (byte)common->irand(inc >> 1, inc);
                }
            }
            */
            count = 0;

            y = 0;
            while y < height {
                x = 0;
                while x < width {
                    xpos = (x * hm_width / width);
                    ypos = (y * hm_height / height);
                    ypos = hm_height - ypos - 1;

                    if *hm_map.add((ypos * hm_width + xpos) as usize) < 150 {
                        x += 1;
                        densityPos = densityPos.add(1);
                        continue;
                    }

                    foundUneven = false;
                    dx = -4;
                    while (dx <= 4) && !foundUneven {
                        dy = -4;
                        while (dy <= 4) && !foundUneven {
                            if dx == 0 && dy == 0 {
                                dy += 1;
                                continue;
                            }
                            if (xpos + dx) >= 0 && (xpos + dx) < hm_width && (ypos + dy) >= 0 && (ypos + dy) < hm_height
                            {
                                if *hm_map.add(((ypos + dy) * hm_width + (xpos + dx)) as usize) < 190 {
                                    *densityPos = 205;
                                    count += 1;
                                    foundUneven = true;
                                }
                            }
                            dy += 1;
                        }
                        dx += 1;
                    }

                    x += 1;
                    densityPos = densityPos.add(1);
                }
                y += 1;
            }

            /*	FILE	*FH;

            FH = fopen("c:\o.raw", "wb");
            fwrite(hm_map, 1, common->GetRealWidth() * common->GetRealHeight(), FH);
            fclose(FH);

            FH = fopen("c:\d.raw", "wb");
            fwrite(density, 1, width*height, FH);
            fclose(FH);
            */

            // Reduce severely for any settlements/buildings/objectives
            VectorScale((*self.common).GetSize(), 1.0 / width as f32, &mut derxelSize);

            origin_land = self.common;
            area = (*self.common).GetFirstArea();
            while !area.is_null() {
                // Skip group types since they encompass to much open area
                if (*area).GetType() == AT_GROUP {
                    area = (*self.common).GetNextArea();
                    continue;
                }

                VectorSubtract((*area).GetPosition(), (*self.common).GetMins(), &mut pos);
                VectorInverseScaleVector(pos, derxelSize, &mut dmappos);
                // Damn upside down gensurf
                dmappos[1] = height - dmappos[1];

                count = ceilf((*area).GetRadius() / derxelSize[1]) as c_int;

                while count > 0 {
                    CM_CircularIterate(
                        density,
                        width,
                        height,
                        dmappos[0],
                        dmappos[1],
                        0,
                        count,
                        core::ptr::null_mut(),
                        CG_Decrease,
                    );
                    count -= 1;
                }
                area = (*self.common).GetNextArea();
            }
        }
    }

    /// Load a density map from file or generate one randomly.
    pub fn LoadDensityMap(&mut self, td: *const c_char) {
        unsafe {
            let mut densityMap: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
            let mut imageData: *mut u8;
            let mut iWidth: c_int;
            let mut iHeight: c_int;
            let mut seed: c_int;
            let mut ptr: *mut c_char = core::ptr::null_mut();

            // Fill in with default values
            self.mDensityMap = Z_Malloc((*self.common).GetBlockCount(), TAG_R_TERRAIN, 0) as *mut u8;
            memset(
                self.mDensityMap as *mut c_void,
                128,
                (*self.common).GetBlockCount() as core::ffi::c_size_t,
            );

            // Load in density map (if any)
            Com_sprintf(
                densityMap.as_mut_ptr(),
                MAX_QPATH,
                b"%s\0".as_ptr() as *const c_char,
                Info_ValueForKey(td, b"densityMap\0".as_ptr() as *const c_char),
            );
            if strlen(densityMap.as_ptr()) > 0 {
                Com_DPrintf(
                    b"CG_Terrain: Loading density map %s.....\n\0".as_ptr() as *const c_char,
                    densityMap.as_ptr(),
                );
                imageData = core::ptr::null_mut();
                iWidth = 0;
                iHeight = 0;
                R_LoadDataImage(densityMap.as_ptr(), &mut imageData, &mut iWidth, &mut iHeight);
                if !imageData.is_null() {
                    if !strstr(densityMap.as_ptr(), b"density_\0".as_ptr() as *const c_char).is_null() {
                        seed = strtoul(
                            Info_ValueForKey(td, b"seed\0".as_ptr() as *const c_char),
                            &mut (ptr as *mut *const c_char),
                            10,
                        ) as c_int;
                        self.CreateRandomDensityMap(imageData, iWidth, iHeight, seed);
                    }
                    R_Resample(
                        imageData,
                        iWidth,
                        iHeight,
                        self.mDensityMap,
                        (*self.common).GetBlockWidth(),
                        (*self.common).GetBlockHeight(),
                        1,
                    );
                    R_InvertImage(
                        self.mDensityMap,
                        (*self.common).GetBlockWidth(),
                        (*self.common).GetBlockHeight(),
                        1,
                    );
                    Z_Free(imageData as *mut c_void);
                }
            }
        }
    }

    /// Sprinkle random models on a terrain patch based on height details.
    pub fn Sprinkle(&mut self, patch: *mut CCMPatch, hd: *mut CCGHeightDetails, level: c_int) {
        unsafe {
            let mut i: c_int;
            let mut count: c_int;
            let px: c_int;
            let py: c_int;
            let mut density: f32;
            let mut origin: [f32; 3] = [0.0; 3];
            let mut scale: [f32; 3] = [0.0; 3];
            let mut angles: [f32; 3] = [0.0; 3];
            let mut bounds: [[f32; 3]; 2] = [[0.0; 3]; 2];
            let mut refEnt = refEntity_t::default();
            let mut rm: *mut CRandomModel;
            let mut area = CArea { _opaque: [0; 0] };

            px = (*patch).GetHeightMapX() / (*self.common).GetTerxels();
            py = (*patch).GetHeightMapY() / (*self.common).GetTerxels();
            // Get a number -5.3f to 5.3f
            density = (*self.mDensityMap.add((px + ((*self.common).GetBlockWidth() * py)) as usize) as f32
                - 128.0)
                / 24.0;
            // ..and multiply that into the count
            count = Round(
                (*self.common).GetPatchScalarSize()
                    * (*hd).GetAverageFrequency() as f32
                    * powf(2.0, density)
                    * 0.001,
            );

            i = 0;
            while i < count {
                if (*self.common).irand(0, 10) != 0 {
                    i += 1;
                    continue;
                }

                let mut temp: [f32; 3] = [0.0; 3];
                let mut tr: TraceResult;
                let mut average: f32;

                rm = (*hd).GetRandomModel(self.common);

                refEnt.hModel = re_RegisterModel((*rm).GetModelName());
                refEnt.frame = 0;
                RE_GetModelBounds(&mut refEnt, bounds[0].as_mut_ptr(), bounds[1].as_mut_ptr());

                // Calculate the scale using some magic to help ensure that the
                // scales are never too different from eachother.  Otherwise you
                // could get an entity that is really small on one axis but huge
                // on another.
                temp[0] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());
                temp[1] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());
                temp[2] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());

                // Average of the three random numbers and divide that by two
                average = ((temp[0] + temp[1] + temp[2]) / 3.0) / 2.0;

                // Add in half of the other two numbers and then subtract half the average to prevent.
                // any number from going beyond the range. If all three numbers were the same then
                // they would remain unchanged after this calculation.
                scale[0] = temp[0] + (temp[1] + temp[2]) / 2.0 - average;
                scale[1] = temp[1] + (temp[0] + temp[2]) / 2.0 - average;
                scale[2] = temp[2] + (temp[0] + temp[1]) / 2.0 - average;

                angles[0] = 0.0;
                angles[1] = (*self.common).flrand(-M_PI, M_PI);
                angles[2] = 0.0;

                VectorCopy((*patch).GetMins(), &mut origin);
                origin[0] += (*self.common).flrand(0.0, (*self.common).GetPatchWidth());
                origin[1] += (*self.common).flrand(0.0, (*self.common).GetPatchHeight());
                // Get above world height
                let slope = (*self.common).GetWorldHeight(origin, bounds.as_mut_ptr(), true);

                if slope > 1.33 {
                    // spot has too steep of a slope
                    i += 1;
                    continue;
                }
                if origin[2] < (*self.common).GetWaterHeight() {
                    i += 1;
                    continue;
                }
                // very that we aren't dropped too low
                if origin[2] < (*self.common).CalcWorldHeight(level) {
                    i += 1;
                    continue;
                }

                // Hack-ariffic, don't allow them to drop below the big player clip brush.
                if origin[2] < 1280.0 {
                    i += 1;
                    continue;
                }
                // FIXME: shouldn't be using a hard-coded 1280 number, only allow to spawn if inside player clip brush?
                //		if( !(CONTENTS_PLAYERCLIP & VM_Call( cgvm, CG_POINT_CONTENTS )) )
                //		{
                //			continue;
                //		}
                // Simple radius check for buildings
                /*			area.Init(origin, VectorLength(bounds[0]));
                if(common->AreaCollision(&area, areaTypes, sizeof(areaTypes) / sizeof(int)))
                {
                    continue;
                }*/
                // Make sure there is no architecture around - doesn't work for ents though =(

                /*
                memset(td, sizeof(*td), 0);
                VectorCopy(origin, td->mStart);
                VectorCopy(bounds[0], td->mMins);
                VectorCopy(bounds[1], td->mMaxs);
                VectorCopy(origin, td->mEnd);
                td->mSkipNumber = -1;
                td->mMask = MASK_PLAYERSOLID;
                */
                tr = core::mem::zeroed();
                SV_Trace(
                    &mut tr,
                    &origin,
                    &bounds[0],
                    &bounds[1],
                    &origin,
                    -1,
                    CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN,
                );

                /*
                VM_Call( cgvm, CG_TRACE );
                if(td->mResult.surfaceFlags & SURF_NOMISCENTS)
                {
                    continue;
                }
                if(td->mResult.startsolid)
                {
    //				continue;
                }
                */
                if tr.surfaceFlags & SURF_NOMISCENTS != 0 {
                    i += 1;
                    continue;
                }
                if tr.startsolid != 0 {
                    //				continue;
                }

                // Get minimum height of area
                (*self.common).GetWorldHeight(origin, bounds.as_mut_ptr(), false);
                // Account for relative origin
                origin[2] -= bounds[0][2] * scale[2];
                origin[2] -= (*self.common).flrand(2.0, (bounds[1][2] - bounds[0][2]) / 4.0);

                //rwwFIXMEFIXME: Do this properly
                // Spawn the client model
                /*
                strcpy(data->mModel, rm->GetModelName());
                VectorCopy(origin, data->mOrigin);
                VectorCopy(angles, data->mAngles);
                VectorCopy(scale, data->mScale);
                VM_Call( cgvm, CG_MISC_ENT);
                */

                self.mModelCount += 1;

                i += 1;
            }
        }
    }

    /// Spawn miscellaneous models on a specific patch.
    pub fn SpawnPatchModels(&mut self, patch: *mut CCMPatch) {
        unsafe {
            let mut i: c_int;
            let hd: *mut CCGHeightDetails;

            //	Rand_Init(10);
            i = 0;
            while i < 4 {
                hd = self.mHeightDetails.as_mut_ptr().add((*patch).GetHeight(i) as usize);
                if (*hd).GetNumModels() > 0 {
                    self.Sprinkle(patch, hd, (*patch).GetHeight(i));
                }
                i += 1;
            }
        }
    }
}

impl Drop for CRMLandScape {
    fn drop(&mut self) {
        unsafe {
            if !self.mDensityMap.is_null() {
                Z_Free(self.mDensityMap as *mut c_void);
                self.mDensityMap = core::ptr::null_mut();
            }
        }
    }
}

// ============================================================================
// Callback wrapper function
// ============================================================================

/// Wrapper callback for spawning models on patches.
extern "C" fn SpawnPatchModelsWrapper(patch: *mut CCMPatch, userdata: *mut c_void) {
    unsafe {
        let landscape = userdata as *mut CRMLandScape;
        (*landscape).SpawnPatchModels(patch);
    }
}

// ============================================================================
// Density map reduction callback
// ============================================================================

/// Callback to decrease density at a specific point.
extern "C" fn CG_Decrease(work: *mut u8, _lerp: f32, _info: *mut c_int) {
    unsafe {
        let val = *work as c_int - (*origin_land).irand(2, 5);
        *work = Com_Clamp(1, 255, val) as u8;
    }
}

// ============================================================================
// Public interface functions
// ============================================================================

/// Initialize the terrain system.
pub fn RM_InitTerrain() {
    unsafe {
        rm_landscape = core::ptr::null_mut();
    }
}

/// Create random models for a terrain instance.
pub fn RM_CreateRandomModels(terrainId: c_int, terrainInfo: *const c_char) {
    unsafe {
        let landscape: *mut CRMLandScape = Box::into_raw(Box::new(CRMLandScape::new()));
        rm_landscape = landscape;
        (*landscape).SetCommon(cmg.landScape);

        Com_DPrintf(b"CG_Terrain: Creating random models.....\n\0".as_ptr() as *const c_char);
        (*landscape).LoadMiscentDef(terrainInfo);
        (*landscape).LoadDensityMap(terrainInfo);
        (*landscape).ClearModelCount();
        CM_TerrainPatchIterate(
            (*landscape).GetCommon(),
            SpawnPatchModelsWrapper,
            landscape as *mut c_void,
        );

        Com_DPrintf(
            b".....%d random client models spawned\n\0".as_ptr() as *const c_char,
            (*landscape).GetModelCount(),
        );
    }
}

/// Shutdown the terrain system and cleanup resources.
pub fn RM_ShutdownTerrain() {
    unsafe {
        let landscape: *mut CRMLandScape = rm_landscape;
        if !landscape.is_null() {
            //			CM_ShutdownTerrain(i);
            let _ = Box::from_raw(landscape);
            rm_landscape = core::ptr::null_mut();
        }
    }
}

// ============================================================================
// Utility functions (originally inlined or from other modules)
// ============================================================================

/// Copy a 3D vector.
#[inline(always)]
fn VectorCopy(src: [f32; 3], dest: &mut [f32; 3]) {
    dest[0] = src[0];
    dest[1] = src[1];
    dest[2] = src[2];
}

/// Subtract two 3D vectors.
#[inline(always)]
fn VectorSubtract(a: [f32; 3], b: [f32; 3], out: &mut [f32; 3]) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
}

/// Scale a 3D vector.
#[inline(always)]
fn VectorScale(v: [f32; 3], scale: f32, out: &mut [f32; 3]) {
    out[0] = v[0] * scale;
    out[1] = v[1] * scale;
    out[2] = v[2] * scale;
}

/// Inverse scale a 3D vector (divide by scale).
#[inline(always)]
fn VectorInverseScaleVector(v: [f32; 3], scale: [f32; 3], out: &mut [c_int; 3]) {
    if scale[0] != 0.0 {
        out[0] = (v[0] / scale[0]) as c_int;
    } else {
        out[0] = 0;
    }
    if scale[1] != 0.0 {
        out[1] = (v[1] / scale[1]) as c_int;
    } else {
        out[1] = 0;
    }
    if scale[2] != 0.0 {
        out[2] = (v[2] / scale[2]) as c_int;
    } else {
        out[2] = 0;
    }
}

/// Round a float to the nearest integer.
#[inline(always)]
fn Round(f: f32) -> c_int {
    (f + 0.5) as c_int
}

// Mathematical constants
const M_PI: f32 = 3.14159265358979323846;

/// Compute the ceiling of a float as a Rust function.
#[inline]
fn ceilf(x: f32) -> f32 {
    x.ceil()
}

/// Compute x^y.
#[inline]
fn powf(x: f32, y: f32) -> f32 {
    x.powf(y)
}

// end
