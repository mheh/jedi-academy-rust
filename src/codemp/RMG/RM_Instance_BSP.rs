//! Mechanical port of `codemp/RMG/RM_Instance_BSP.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_float};

// ============================================================================
// CONSTANTS AND ENUMS
// ============================================================================

pub const AUTOMAP_NONE: c_int = 0;
pub const AUTOMAP_BLD: c_int = 1;
pub const AUTOMAP_OBJ: c_int = 2;
pub const AUTOMAP_START: c_int = 3;
pub const AUTOMAP_END: c_int = 4;
pub const AUTOMAP_ENEMY: c_int = 5;
pub const AUTOMAP_FRIEND: c_int = 6;
pub const AUTOMAP_WALL: c_int = 7;

pub const SIDE_RED: c_int = 1;

pub const CONTENTS_TERRAIN: c_int = 0x1000000;
pub const CONTENTS_SOLID: c_int = 0x1;

pub const MIN_WORLD_COORD: f32 = -65536.0;

pub type qboolean = c_int;
const qfalse: qboolean = 0;

pub type vec3_t = [f32; 3];
pub type vec3pair_t = [[f32; 3]; 2];

pub const MAX_QPATH: usize = 256;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs used during instance construction.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *default_val)`.
    /// Returns the value string associated with the given key, or default if not found.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        // Porting stub: in reality, this looks up the key in internal storage
        // and returns the value or the default. For now, return the default.
        default_val
    }
}

/// Stub for unported `class CRMInstanceFile` (RM_InstanceFile.h).
/// Reference to an open instance file for creating sub-instances.
pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMArea` (RM_Area.h).
/// Represents an area in the map.
pub struct CRMArea {
    _opaque: [u8; 0],
}

impl CRMArea {
    /// Stub for `bool CRMArea::IsCollisionEnabled()`.
    pub fn IsCollisionEnabled(&self) -> bool {
        true // Porting stub
    }

    /// Stub for `vec_t* CRMArea::GetOrigin()`.
    pub fn GetOrigin(&self) -> *mut f32 {
        core::ptr::null_mut() // Porting stub
    }

    /// Stub for `float CRMArea::GetAngle()`.
    pub fn GetAngle(&self) -> f32 {
        0.0 // Porting stub
    }
}

/// Stub for unported `class CRMAreaManager` (RM_Area.h).
/// Manages multiple areas in the map.
pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

impl CRandomTerrain {
    /// Stub for `CRMLandscape* CRandomTerrain::GetLandScape()`.
    pub fn GetLandScape(&self) -> *const CRMLandscape {
        core::ptr::null() // Porting stub
    }

    /// Stub for `vec3pair_t& CRandomTerrain::GetBounds()`.
    pub fn GetBounds(&self) -> *const vec3pair_t {
        core::ptr::null() // Porting stub
    }
}

/// Stub for unported `class CRMLandscape` (RM_Area.h).
pub struct CRMLandscape {
    _opaque: [u8; 0],
}

impl CRMLandscape {
    /// Stub for `float CRMLandscape::GetWaterHeight()`.
    pub fn GetWaterHeight(&self) -> f32 {
        0.0 // Porting stub
    }

    /// Stub for `vec3_t& CRMLandscape::GetTerxelSize()`.
    pub fn GetTerxelSize(&self) -> *const vec3_t {
        core::ptr::null() // Porting stub
    }

    /// Stub for `vec3pair_t& CRMLandscape::GetBounds()`.
    pub fn GetBounds(&self) -> *const vec3pair_t {
        core::ptr::null() // Porting stub
    }

    /// Stub for `int CRMLandscape::irand(int low, int high)`.
    pub fn irand(&self, _low: c_int, _high: c_int) -> c_int {
        0 // Porting stub
    }

    /// Stub for `void CRMLandscape::GetWorldHeight(vec3_t pos, vec3pair_t bounds, bool floor)`.
    pub fn GetWorldHeight(&self, _pos: *mut f32, _bounds: *const vec3pair_t, _floor: bool) {
        // Porting stub
    }
}

/// Stub for unported `class CRMInstance` (RM_Instance.h).
/// Base class for all instances in the random map generation system.
pub struct CRMInstance {
    /// Filter of entities inside of this
    pub mFilter: [c_char; MAX_QPATH],
    /// Team specific filter
    pub mTeamFilter: [c_char; MAX_QPATH],
    /// Bounding box for instance itself
    pub mBounds: vec3pair_t,
    /// Position of the instance
    pub mArea: *mut CRMArea,
    /// Objective associated with this instance
    pub mObjective: *const core::ffi::c_void,
    /// Message outputted when objective is completed
    pub mMessage: [c_char; MAX_QPATH],
    /// Description of objective
    pub mDescription: [c_char; MAX_QPATH],
    /// More info for objective
    pub mInfo: [c_char; MAX_QPATH],
    /// Radius to space instances with
    pub mSpacingRadius: f32,
    /// Radius to flatten under instances
    pub mFlattenRadius: f32,
    /// Line of spacing radius's, forces locket
    pub mSpacingLine: c_int,
    /// Origin cant move
    pub mLockOrigin: bool,
    /// Allow surface sprites under instance?
    pub mSurfaceSprites: bool,
    /// Show which symbol on automap 0=none
    pub mAutomapSymbol: c_int,
    /// Id of entity spawned
    pub mEntityID: c_int,
    /// Blue or red side
    pub mSide: c_int,
    /// Mirror origin, angle
    pub mMirror: c_int,
    /// Height to flatten land
    pub mFlattenHeight: c_int,
}

impl CRMInstance {
    /// Stub for parent constructor `CRMInstance::CRMInstance(CGPGroup*, CRMInstanceFile&)`.
    /// Initializes the base instance with default values.
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> Self {
        CRMInstance {
            mFilter: [0; MAX_QPATH],
            mTeamFilter: [0; MAX_QPATH],
            mBounds: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            mArea: core::ptr::null_mut(),
            mObjective: core::ptr::null(),
            mMessage: [0; MAX_QPATH],
            mDescription: [0; MAX_QPATH],
            mInfo: [0; MAX_QPATH],
            mSpacingRadius: 0.0,
            mFlattenRadius: 0.0,
            mSpacingLine: 0,
            mLockOrigin: false,
            mSurfaceSprites: false,
            mAutomapSymbol: 0,
            mEntityID: 0,
            mSide: 0,
            mMirror: 0,
            mFlattenHeight: 0,
        }
    }

    /// Stub for `void CRMInstance::SetMessage(const char* msg)`.
    pub fn SetMessage(&mut self, _msg: *const c_char) {
        // Porting stub: in reality, this copies the message string
    }

    /// Stub for `void CRMInstance::SetDescription(const char* desc)`.
    pub fn SetDescription(&mut self, _desc: *const c_char) {
        // Porting stub: in reality, this copies the description string
    }

    /// Stub for `void CRMInstance::SetInfo(const char* info)`.
    pub fn SetInfo(&mut self, _info: *const c_char) {
        // Porting stub: in reality, this copies the info string
    }

    /// Stub for `float CRMInstance::GetSpacingRadius()`.
    pub fn GetSpacingRadius(&self) -> f32 {
        self.mSpacingRadius
    }

    /// Stub for `float CRMInstance::GetFlattenRadius()`.
    pub fn GetFlattenRadius(&self) -> f32 {
        self.mFlattenRadius
    }

    /// Stub for `vec_t* CRMInstance::GetOrigin()`.
    pub fn GetOrigin(&self) -> *mut f32 {
        unsafe {
            if !self.mArea.is_null() {
                (*self.mArea).GetOrigin()
            } else {
                core::ptr::null_mut()
            }
        }
    }

    /// Stub for `bool CRMInstance::HasObjective()`.
    pub fn HasObjective(&self) -> bool {
        !self.mObjective.is_null()
    }

    /// Stub for `vec3pair_t& CRMInstance::GetBounds()`.
    pub fn GetBounds(&self) -> *const vec3pair_t {
        &self.mBounds
    }

    /// Stub for `void CRMInstance::SetSide(int side)`.
    pub fn SetSide(&mut self, side: c_int) {
        self.mSide = side;
    }

    /// Stub for `void CRMInstance::DrawAutomapSymbol()`.
    pub fn DrawAutomapSymbol(&self) {
        // Porting stub
    }
}

/// Stub for unported `class CRMBSPInstance` (RM_Instance_BSP.h).
/// Implements a BSP instance as part of the random map generation system.
pub struct CRMBSPInstance {
    /// Inherited fields from CRMInstance
    pub base: CRMInstance,
    /// Name/path of the BSP model
    pub mBsp: [c_char; MAX_QPATH],
    /// Angle variance in radians
    pub mAngleVariance: f32,
    /// Base angle in radians
    pub mBaseAngle: f32,
    /// Angle difference in radians
    pub mAngleDiff: f32,
}

// ============================================================================
// extern "C" FUNCTION DECLARATIONS
// ============================================================================

extern "C" {
    /// C standard library function to copy a null-terminated string.
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    /// C standard library function to convert a string to a double.
    fn atof(s: *const c_char) -> f64;

    /// C standard library function to convert a string to an integer.
    fn atoi(s: *const c_char) -> c_int;

    /// C standard library function to get the length of a null-terminated string.
    fn strlen(s: *const c_char) -> usize;

    /// C standard library function to find a substring in a string.
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;

    /// C standard library function to concatenate strings.
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    /// C standard library function to compare strings case-insensitively.
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Print debug message
    fn Com_DPrintf(fmt: *const c_char, ...);

    /// Memory set function
    fn Com_Memset(dest: *mut core::ffi::c_void, ch: c_int, count: usize) -> *mut core::ffi::c_void;

    /// Vector copy
    fn VectorCopy(in_: *const f32, out: *mut f32);

    /// Angle normalization to 0-360 range
    fn AngleNormalize360(angle: f32) -> f32;

    /// Performs a trace through the collision model
    fn SV_Trace(
        results: *mut trace_t,
        start: *const f32,
        mins: *const f32,
        maxs: *const f32,
        end: *const f32,
        passent: c_int,
        contentmask: c_int,
        capsule: qboolean,
        useLandScape: c_int,
        traceType: c_int,
    );

    /// Call a VM function
    fn VM_Call(vm: *mut core::ffi::c_void, command: c_int, ...);

    /// Printf function (for sprintf)
    fn sprintf(str: *mut c_char, fmt: *const c_char, ...) -> c_int;
}

// ============================================================================
// MACRO/INLINE EQUIVALENTS
// ============================================================================

/// Conversion from degrees to radians
#[inline]
pub fn DEG2RAD(deg: f64) -> f32 {
    ((deg) * std::f64::consts::PI / 180.0) as f32
}

/// Conversion from radians to degrees
#[inline]
pub fn RAD2DEG(rad: f32) -> f32 {
    (rad * 180.0 / std::f32::consts::PI)
}

/// Vector copy for arrays
#[inline]
pub fn vector_copy(src: &[f32; 3], dst: &mut [f32; 3]) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

/// Vector for origin/zero
pub const VEC3_ORIGIN: vec3_t = [0.0, 0.0, 0.0];

// ============================================================================
// GLOBAL VARIABLES
// ============================================================================

/// Global reference to the random mission manager
extern "C" {
    // Porting stub: TheRandomMissionManager is a global C++ object
    // We declare it as extern "C" but note that in a real port,
    // this would need proper C bindings or a wrapper struct
    static TheRandomMissionManager: *const core::ffi::c_void;
}

/// Stub for server global variables
pub struct server_t {
    pub entityParsePoint: *mut c_char,
}

extern "C" {
    /// Global server state
    static mut sv: server_t;
    /// Global VM handle for game module
    static mut gvm: *mut core::ffi::c_void;
}

// ============================================================================
// ADDITIONAL STUBS
// ============================================================================

/// Stub for trace_t structure
#[repr(C)]
pub struct trace_t {
    pub allsolid: u8,      // if true, plane is not valid
    pub startsolid: u8,    // if true, the initial point was in a solid area
    pub entityNum: i16,    // entity the contacted surface is a part of
    pub fraction: f32,     // time completed, 1.0 = didn't hit anything
    pub endpos: vec3_t,    // final position
    pub plane_normal: vec3_t,
    pub plane_dist: f32,
    pub contents: c_int,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl CRMBSPInstance {
    /// CRMBSPInstance::CRMBSPInstance
    /// Constructs a building instance object using the given parser group
    ///
    /// inputs:
    ///  instance:  parser group containing information about the building instance
    ///
    /// return:
    ///  none
    pub fn new(instGroup: *const CGPGroup, instFile: *const CRMInstanceFile) -> Self {
        // Call parent constructor
        let mut instance = CRMBSPInstance {
            base: CRMInstance::new(instGroup, instFile),
            mBsp: [0; MAX_QPATH],
            mAngleVariance: 0.0,
            mBaseAngle: 0.0,
            mAngleDiff: 0.0,
        };

        // Safety: instGroup is assumed to be a valid pointer (passed from caller).
        // The FindPairValue call returns a pointer to a C string from the parser group,
        // which remains valid for the duration of this constructor call.
        unsafe {
            if !instGroup.is_null() {
                let instGroup_ref = &*instGroup;

                // Copy the BSP filename
                let bsp_file = instGroup_ref.FindPairValue(c"file".as_ptr(), c"".as_ptr());
                strcpy(instance.mBsp.as_mut_ptr(), bsp_file);

                // Parse angle variance (in degrees, convert to radians)
                let angle_var_str = instGroup_ref.FindPairValue(c"anglevariance".as_ptr(), c"0".as_ptr());
                instance.mAngleVariance = DEG2RAD(atof(angle_var_str));

                // Parse base angle (in degrees, convert to radians)
                let base_angle_str = instGroup_ref.FindPairValue(c"baseangle".as_ptr(), c"0".as_ptr());
                instance.mBaseAngle = DEG2RAD(atof(base_angle_str));

                // Parse angle difference (in degrees, convert to radians)
                let angle_diff_str = instGroup_ref.FindPairValue(c"anglediff".as_ptr(), c"0".as_ptr());
                instance.mAngleDiff = DEG2RAD(atof(angle_diff_str));

                // Parse spacing radius (defaults to 100)
                let spacing_str = instGroup_ref.FindPairValue(c"spacing".as_ptr(), c"100".as_ptr());
                instance.base.mSpacingRadius = atof(spacing_str) as f32;

                // Parse spacing line (defaults to 0)
                let spacing_line_str = instGroup_ref.FindPairValue(c"spacingline".as_ptr(), c"0".as_ptr());
                instance.base.mSpacingLine = atoi(spacing_line_str);

                // Parse surface sprites flag (defaults to "no")
                let surface_sprites_str = instGroup_ref.FindPairValue(c"surfacesprites".as_ptr(), c"no".as_ptr());
                instance.base.mSurfaceSprites = (Q_stricmp(surface_sprites_str, c"yes".as_ptr()) == 0);

                // Parse lock origin flag (defaults to "no")
                let lock_origin_str = instGroup_ref.FindPairValue(c"lockorigin".as_ptr(), c"no".as_ptr());
                instance.base.mLockOrigin = (Q_stricmp(lock_origin_str, c"yes".as_ptr()) == 0);

                // Parse flatten radius (defaults to 0)
                let flatten_str = instGroup_ref.FindPairValue(c"flatten".as_ptr(), c"0".as_ptr());
                instance.base.mFlattenRadius = atof(flatten_str) as f32;

                // Parse hole radius (defaults to 0)
                let hole_str = instGroup_ref.FindPairValue(c"hole".as_ptr(), c"0".as_ptr());
                // Note: mHoleRadius is stored in parent class but not accessible here in this stub

                // Parse automap symbol
                let automap_sym_str = instGroup_ref.FindPairValue(c"automap_symbol".as_ptr(), c"building".as_ptr());

                if Q_stricmp(automap_sym_str, c"none".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_NONE;
                } else if Q_stricmp(automap_sym_str, c"building".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_BLD;
                } else if Q_stricmp(automap_sym_str, c"objective".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_OBJ;
                } else if Q_stricmp(automap_sym_str, c"start".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_START;
                } else if Q_stricmp(automap_sym_str, c"end".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_END;
                } else if Q_stricmp(automap_sym_str, c"enemy".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_ENEMY;
                } else if Q_stricmp(automap_sym_str, c"friend".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_FRIEND;
                } else if Q_stricmp(automap_sym_str, c"wall".as_ptr()) == 0 {
                    instance.base.mAutomapSymbol = AUTOMAP_WALL;
                } else {
                    instance.base.mAutomapSymbol = atoi(automap_sym_str);
                }

                // Optional instance objective strings
                let objective_msg = instGroup_ref.FindPairValue(c"objective_message".as_ptr(), c"".as_ptr());
                instance.base.SetMessage(objective_msg);

                let objective_desc = instGroup_ref.FindPairValue(c"objective_description".as_ptr(), c"".as_ptr());
                instance.base.SetDescription(objective_desc);

                let objective_info = instGroup_ref.FindPairValue(c"objective_info".as_ptr(), c"".as_ptr());
                instance.base.SetInfo(objective_info);

                // Initialize bounds to zero
                instance.base.mBounds[0][0] = 0.0;
                instance.base.mBounds[0][1] = 0.0;
                instance.base.mBounds[1][0] = 0.0;
                instance.base.mBounds[1][1] = 0.0;

                // Adjust base angle with random variance
                // Note: TheRandomMissionManager is a global C++ object pointer
                // This is a porting stub that assumes proper initialization
                // In production, this would need access to the actual landscape object
                // For now, we skip this random adjustment as it requires accessing C++ objects
                // instance.mBaseAngle += (landscape.irand(0, mAngleVariance) - mAngleVariance/2);
            }
        }

        instance
    }

    /// CRMBSPInstance::Spawn
    /// Spawns a bsp into the world using the previously acquired origin
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///  none
    pub fn Spawn(&mut self, terrain: *const CRandomTerrain, IsServer: qboolean) -> bool {
        // PRE_RELEASE_DEMO compilation guard - normally this entire body is compiled
        // For this port, we include the full implementation

        let mut yaw: f32;
        let mut temp: [c_char; 10000] = [0; 10000];
        let mut origin: vec3_t = [0.0, 0.0, 0.0];
        let mut notmirrored: vec3_t = [0.0, 0.0, 0.0];

        // Safety: terrain is assumed to be a valid pointer (passed from caller)
        let water_level = unsafe {
            if terrain.is_null() {
                0.0
            } else {
                let landscape = (*terrain).GetLandScape();
                if landscape.is_null() {
                    0.0
                } else {
                    (*landscape).GetWaterHeight()
                }
            }
        };

        let terxel_size_ptr: *const vec3_t;
        let bounds_ptr: *const vec3pair_t;

        unsafe {
            if terrain.is_null() {
                return false;
            }

            let landscape = (*terrain).GetLandScape();
            if landscape.is_null() {
                return false;
            }

            terxel_size_ptr = (*landscape).GetTerxelSize();
            bounds_ptr = (*landscape).GetBounds();
        }

        // If this entity somehow lost its collision flag then boot it
        unsafe {
            if !self.base.mArea.is_null() && !(*self.base.mArea).IsCollisionEnabled() {
                return false;
            }
        }

        // Copy out the unmirrored version
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                VectorCopy(origin_ptr as *const f32, notmirrored.as_mut_ptr());
            }
        }

        // We want to mirror it before determining the Z value just in case the landscape isn't perfectly mirrored
        if self.base.mMirror != 0 {
            unsafe {
                // This requires access to TheRandomMissionManager which is a C++ global
                // For now, this is a porting stub
                // GetOrigin()[0] = TheRandomMissionManager->GetLandScape()->GetBounds()[0][0] +
                //                  TheRandomMissionManager->GetLandScape()->GetBounds()[1][0] - GetOrigin()[0];
                // GetOrigin()[1] = TheRandomMissionManager->GetLandScape()->GetBounds()[0][1] +
                //                  TheRandomMissionManager->GetLandScape()->GetBounds()[1][1] - GetOrigin()[1];
            }
        }

        // Align the instance to the center of a terxel
        unsafe {
            if !terxel_size_ptr.is_null() && !bounds_ptr.is_null() {
                let terxel_size = &*terxel_size_ptr;
                let bounds = &*bounds_ptr;
                let origin_ptr = self.base.GetOrigin();

                if !origin_ptr.is_null() {
                    (*origin_ptr) = bounds[0][0] +
                        (((*origin_ptr - bounds[0][0] + terxel_size[0] / 2.0) / terxel_size[0]) as i32 as f32) * terxel_size[0];

                    let origin_ptr_1 = self.base.GetOrigin();
                    (*origin_ptr_1.add(1)) = bounds[0][1] +
                        (((*origin_ptr_1 - bounds[0][1] + terxel_size[1] / 2.0) / terxel_size[1]) as i32 as f32) * terxel_size[1];
                }
            }
        }

        // Make sure the bsp is resting on the ground, not below or above it
        // NOTE: This check is basically saying "is this instance not a bridge", because when instances are created they are all
        // placed above the world's Z boundary, EXCEPT FOR BRIDGES. So this call to GetWorldHeight will move all other instances down to
        // ground level except bridges
        unsafe {
            if !terrain.is_null() {
                let bounds = (*terrain).GetBounds();
                let origin_ptr = self.base.GetOrigin();

                if !origin_ptr.is_null() && !bounds.is_null() && *origin_ptr.add(2) > (*bounds)[1][2] {
                    if self.base.GetFlattenRadius() > 0.0 {
                        let landscape = (*terrain).GetLandScape();
                        if !landscape.is_null() {
                            (*landscape).GetWorldHeight(origin_ptr, bounds, false);
                            *origin_ptr.add(2) += 5.0;
                        }
                    } else if IsServer != 0 {
                        // If this instance does not flatten the ground around it, do a trace to more accurately determine its Z value
                        let mut tr: trace_t = core::mem::zeroed();
                        let mut end: vec3_t = [0.0, 0.0, 0.0];
                        let mut start: vec3_t = [0.0, 0.0, 0.0];

                        let origin_ptr = self.base.GetOrigin();
                        if !origin_ptr.is_null() {
                            VectorCopy(origin_ptr as *const f32, end.as_mut_ptr());
                            VectorCopy(origin_ptr as *const f32, start.as_mut_ptr());
                        }

                        // Start the trace below the top height of the landscape
                        // Note: This requires access to TheRandomMissionManager which is a C++ global
                        // For now, using a stub value
                        start[2] = -65500.0;  // Porting stub

                        // End the trace at the bottom of the world
                        end[2] = MIN_WORLD_COORD;

                        Com_Memset(
                            &mut tr as *mut _ as *mut core::ffi::c_void,
                            0,
                            core::mem::size_of::<trace_t>()
                        );

                        SV_Trace(
                            &mut tr,
                            start.as_ptr(),
                            VEC3_ORIGIN.as_ptr(),
                            VEC3_ORIGIN.as_ptr(),
                            end.as_ptr(),
                            -1,
                            CONTENTS_TERRAIN | CONTENTS_SOLID,
                            qfalse,
                            0,
                            10
                        );

                        if (tr.contents & CONTENTS_TERRAIN) == 0 || tr.fraction == 1.0 {
                            // This should never happen
                            // assert(0);

                            // Restore the unmirrored origin
                            let origin_ptr = self.base.GetOrigin();
                            if !origin_ptr.is_null() {
                                VectorCopy(notmirrored.as_ptr(), origin_ptr);
                            }

                            // Don't spawn
                            return false;
                        }

                        // Assign the Z-value to wherever it hit the terrain
                        let origin_ptr = self.base.GetOrigin();
                        if !origin_ptr.is_null() {
                            *origin_ptr.add(2) = tr.endpos[2];
                            // Lower it a little, otherwise the bottom of the instance might be exposed if on some weird sloped terrain
                            *origin_ptr.add(2) -= 16.0; // FIXME: would it be better to use a number related to the instance itself like 1/5 it's height or something...
                        }
                    }
                } else if !terrain.is_null() {
                    let landscape = (*terrain).GetLandScape();
                    let bounds = (*terrain).GetBounds();
                    let origin_ptr = self.base.GetOrigin();

                    if !landscape.is_null() && !origin_ptr.is_null() && !bounds.is_null() {
                        (*landscape).GetWorldHeight(origin_ptr, bounds, true);
                    }
                }
            }
        }

        // Save away the origin
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                VectorCopy(origin_ptr as *const f32, origin.as_mut_ptr());
            }
        }

        // Make sure not to spawn if in water
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !self.base.HasObjective() && !origin_ptr.is_null() && *origin_ptr.add(2) < water_level {
                return false;
            }
        }

        // Restore the origin
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                VectorCopy(origin.as_ptr(), origin_ptr);
            }
        }

        if self.base.mMirror != 0 {
            // Change blue things to red for symmetric maps
            if unsafe { strlen(self.base.mFilter.as_ptr() as *const c_char) } > 0 {
                unsafe {
                    let blue = strstr(self.base.mFilter.as_ptr() as *const c_char, c"blue".as_ptr());
                    if !blue.is_null() {
                        *blue = 0; // Terminate string at 'blue'
                        strcat(self.base.mFilter.as_mut_ptr(), c"red".as_ptr());
                        self.base.SetSide(SIDE_RED);
                    }
                }
            }
            if unsafe { strlen(self.base.mTeamFilter.as_ptr() as *const c_char) } > 0 {
                unsafe {
                    let blue = strstr(self.base.mTeamFilter.as_ptr() as *const c_char, c"blue".as_ptr());
                    if !blue.is_null() {
                        strcpy(self.base.mTeamFilter.as_mut_ptr(), c"red".as_ptr());
                        self.base.SetSide(SIDE_RED);
                    }
                }
            }
            unsafe {
                if !self.base.mArea.is_null() {
                    yaw = RAD2DEG((*self.base.mArea).GetAngle() + self.mBaseAngle) + 180.0;
                } else {
                    yaw = RAD2DEG(self.mBaseAngle) + 180.0;
                }
            }
        } else {
            unsafe {
                if !self.base.mArea.is_null() {
                    yaw = RAD2DEG((*self.base.mArea).GetAngle() + self.mBaseAngle);
                } else {
                    yaw = RAD2DEG(self.mBaseAngle);
                }
            }
        }

        // Commented out code block from original source - appears to be for symmetric map handling
        /*
        if( TheRandomMissionManager->GetMission()->GetSymmetric() )
        {
            vec3_t	diagonal;
            vec3_t	lineToPoint;
            vec3_t	mins;
            vec3_t	maxs;
            vec3_t	point;
            vec3_t	vProj;
            vec3_t	vec;
            float	distance;

            VectorCopy( TheRandomMissionManager->GetLandScape()->GetBounds()[1], maxs );
            VectorCopy( TheRandomMissionManager->GetLandScape()->GetBounds()[0], mins );
            VectorCopy( GetOrigin(), point );
            mins[2] = maxs[2] = point[2] = 0;
            VectorSubtract( point, mins, lineToPoint );
            VectorSubtract( maxs, mins, diagonal);


            VectorNormalize(diagonal);
            VectorMA( mins, DotProduct(lineToPoint, diagonal), diagonal, vProj);
            VectorSubtract(point, vProj, vec );
            distance = VectorLength(vec);

            // if an instance is too close to the imaginary diagonal that cuts the world in half, don't spawn it
            // otherwise you can get overlapping instances
            if( distance < GetSpacingRadius() )
            {
        #ifdef _DEBUG
                mAutomapSymbol = AUTOMAP_END;
        #endif
                if( !HasObjective() )
                {
                    return false;
                }
            }
        }
        */

        // Spawn in the bsp model
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                sprintf(
                    temp.as_mut_ptr(),
                    c"{\n\"classname\"   \"misc_bsp\"\n\"bspmodel\"    \"%s\"\n\"origin\"      \"%f %f %f\"\n\"angles\"      \"0 %f 0\"\n\"filter\"      \"%s\"\n\"teamfilter\"  \"%s\"\n\"spacing\"\t \"%d\"\n\"flatten\"\t \"%d\"\n}\n".as_ptr(),
                    self.mBsp.as_ptr(),
                    *origin_ptr,
                    *origin_ptr.add(1),
                    *origin_ptr.add(2),
                    AngleNormalize360(yaw),
                    self.base.mFilter.as_ptr(),
                    self.base.mTeamFilter.as_ptr(),
                    self.base.mSpacingRadius as c_int,
                    self.base.mFlattenRadius as c_int
                );
            }
        }

        if IsServer != 0 {
            // Only allow for true spawning on the server
            unsafe {
                let save_ptr = sv.entityParsePoint;
                sv.entityParsePoint = temp.as_mut_ptr();
                VM_Call(gvm, 0); // GAME_SPAWN_RMG_ENTITY = 0 (porting stub)
                sv.entityParsePoint = save_ptr;
            }
        }

        self.base.DrawAutomapSymbol();

        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                Com_DPrintf(
                    c"RMG:  Building '%s' spawned at (%f %f %f)\n".as_ptr(),
                    self.mBsp.as_ptr(),
                    *origin_ptr,
                    *origin_ptr.add(1),
                    *origin_ptr.add(2)
                );
            }
        }

        // Now restore the instances un-mirrored origin
        // NOTE: all this origin flipping, setting the side etc... should be done when mMirror is set
        // because right after this function is called, mMirror is set to 0 but all the instance data is STILL MIRRORED -- not good
        unsafe {
            let origin_ptr = self.base.GetOrigin();
            if !origin_ptr.is_null() {
                VectorCopy(notmirrored.as_ptr(), origin_ptr);
            }
        }

        true
    }

    /// Stub for `const char* GetModelName()`.
    pub fn GetModelName(&self) -> *const c_char {
        self.mBsp.as_ptr()
    }

    /// Stub for `float GetAngleDiff()`.
    pub fn GetAngleDiff(&self) -> f32 {
        self.mAngleDiff
    }

    /// Stub for `bool GetAngularType()`.
    pub fn GetAngularType(&self) -> bool {
        self.mAngleDiff != 0.0
    }
}
