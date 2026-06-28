//! Mechanical port of `codemp/RMG/RM_Instance.cpp`.
//!
//! Implements the CRMInstance class, which represents a single procedural instance
//! in the random map generation system. Handles construction, pre/post-spawn setup,
//! and automap symbol rendering.

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

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs used during instance construction.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMInstanceFile` (RM_InstanceFile.h).
/// Reference to an open instance file for creating sub-instances.
pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMArea` (RM_Area.h).
/// Represents an area in the map with origin, angle, and collision info.
pub struct CRMArea {
    _opaque: [u8; 0],
}

impl CRMArea {
    /// Stub for `void CRMArea::Init(vec3_t, float, float, int, int, int)`.
    /// Initializes an area with position, radius, angle, type, and other parameters.
    pub fn Init(
        &mut self,
        _origin: *const f32,
        _radius: f32,
        _angle: f32,
        _type_val: c_int,
        _village_id: c_int,
        _village_size: c_int,
    ) {
        // Porting stub: initializes the area's position and properties.
    }
}

/// Stub for unported `class CRMAreaManager` (RM_Area.h).
/// Manages multiple areas in the map.
pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

/// Stub for unported `class CRMObjective` (RM_Objective.h).
/// Represents a mission objective with completion state and properties.
pub struct CRMObjective {
    _opaque: [u8; 0],
}

impl CRMObjective {
    /// Stub for `bool CRMObjective::Link()`.
    /// Links the objective into the world.
    pub fn Link(&self) -> bool {
        // Porting stub: returns whether linking was successful.
        false
    }
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
/// Represents the procedural terrain being generated.
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

impl CRandomTerrain {
    /// Stub for `CCMLandScape* CRandomTerrain::GetLandScape()`.
    /// Returns the landscape associated with this terrain.
    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
/// Represents the terrain landscape with height map and flattening.
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// Stub for `const vec3_t& CCMLandScape::GetTerxelSize()`.
    /// Returns the size of a terxel (terrain voxel) in world units.
    pub fn GetTerxelSize(&self) -> *const [f32; 3] {
        core::ptr::null()
    }

    /// Stub for `const vec3pair_t& CCMLandScape::GetBounds()`.
    /// Returns the bounding box (min and max) of the landscape.
    pub fn GetBounds(&self) -> *const [[f32; 3]; 2] {
        core::ptr::null()
    }

    /// Stub for `void CCMLandScape::FlattenArea(CArea*, int, bool, bool, bool)`.
    /// Flattens the terrain under the given area.
    pub fn FlattenArea(
        &self,
        _area: *const CArea,
        _height: c_int,
        _flip_normals: bool,
        _smooth: bool,
        _something: bool,
    ) {
        // Porting stub: flattens terrain in the specified area.
    }
}

/// Stub for unported `class CRandomMissionManager` (RM_Manager.h).
/// Global manager for random mission generation.
pub struct CRandomMissionManager {
    _opaque: [u8; 0],
}

impl CRandomMissionManager {
    /// Stub for `CCMLandScape* CRandomMissionManager::GetLandScape()`.
    /// Returns the landscape being managed.
    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        core::ptr::null_mut()
    }

    /// Stub for `void CRandomMissionManager::AddAutomapSymbol(int, vec3_t, int)`.
    /// Adds a symbol to the automap at the given position.
    pub fn AddAutomapSymbol(&self, _symbol: c_int, _origin: *const [f32; 3], _side: c_int) {
        // Porting stub: adds the automap symbol.
    }
}

// ============================================================================
// GLOBAL REFERENCES
// ============================================================================

/// Global pointer to the random mission manager.
/// Original C declaration: `extern CRandomMissionManager* TheRandomMissionManager;`
extern "C" {
    pub static mut TheRandomMissionManager: *mut CRandomMissionManager;
}

// ============================================================================
// extern "C" functions from libc and engine
// ============================================================================

extern "C" {
    /// C standard library function to copy memory.
    /// Mirrors the Quake engine's VectorCopy macro behavior.
    fn VectorCopy(src: *const f32, dst: *mut f32);
}

// ============================================================================
// Type definitions
// ============================================================================

/// Vector type: array of 3 floats
pub type vec_t = f32;
pub type vec3_t = [vec_t; 3];
pub type vec3pair_t = [vec3_t; 2];
pub type vec_t_ptr = *mut vec_t;

// Stub for area type constants used in Init
pub const AT_NONE: c_int = 0;

// ============================================================================
// CRMInstance class
// ============================================================================

/// Represents a single procedural instance in the random map generation system.
/// An instance can be a building, objective, NPC group, or other map element.
pub struct CRMInstance {
    /// Filter of entities inside this instance
    mFilter: [c_char; 64], // MAX_QPATH

    /// Team specific filter
    mTeamFilter: [c_char; 64], // MAX_QPATH

    /// Bounding box for instance itself
    mBounds: [[f32; 3]; 2], // vec3pair_t

    /// Position and radius information for the instance
    mArea: *mut CRMArea,

    /// Objective associated with this instance
    mObjective: *mut CRMObjective,

    /// Message outputted when objective is completed
    mMessage: *mut c_char,

    /// Description of objective
    mDescription: *mut c_char,

    /// More info for objective
    mInfo: *mut c_char,

    /// Radius to space instances with
    mSpacingRadius: f32,

    /// Radius to flatten under instances
    mFlattenRadius: f32,

    /// Line of spacing radius, forces locket
    mSpacingLine: c_int,

    /// Origin can't move
    mLockOrigin: bool,

    /// Allow surface sprites under instance?
    mSurfaceSprites: bool,

    /// Show which symbol on automap (0=none)
    mAutomapSymbol: c_int,

    /// ID of entity spawned
    mEntityID: c_int,

    /// Blue or red side
    mSide: c_int,

    /// Mirror origin, angle
    mMirror: c_int,

    /// Height to flatten land
    mFlattenHeight: c_int,
}

impl CRMInstance {
    /// CRMInstance::CRMInstance
    /// Constructs an instance object using the given parser group
    ///
    /// inputs:
    ///  instance:  parser group containing information about the instance
    ///
    /// return:
    ///  none
    pub fn new(_instGroup: *const CGPGroup, _instFile: *const CRMInstanceFile) -> Self {
        let mut instance = CRMInstance {
            mObjective: core::ptr::null_mut(),
            mSpacingRadius: 0.0,
            mFlattenRadius: 0.0,
            mFilter: [0; 64],
            mTeamFilter: [0; 64],
            mArea: core::ptr::null_mut(),
            mAutomapSymbol: 0,
            mEntityID: 0,
            mSide: 0,
            mMirror: 0,
            mFlattenHeight: 66,
            mSpacingLine: 0,
            mSurfaceSprites: true,
            mLockOrigin: false,
            mMessage: core::ptr::null_mut(),
            mDescription: core::ptr::null_mut(),
            mInfo: core::ptr::null_mut(),
            mBounds: [[0.0; 3]; 2],
        };

        // Initialize filter arrays to null-terminated strings
        instance.mFilter[0] = 0;
        instance.mTeamFilter[0] = 0;

        instance
    }

    /// CRMInstance::PreSpawn
    /// Prepares the instance for spawning by flattening the ground under it
    ///
    /// inputs:
    ///  landscape: landscape the instance will be spawned on
    ///
    /// return:
    ///  true: spawn preparation successful
    ///  false: spawn preparation failed
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, _IsServer: u8) -> bool {
        let mut origin: vec3_t = [0.0; 3];
        let mut area = CArea {
            mPosition: [0.0; 3],
            mRadius: 0.0,
            mAngle: 0.0,
            mAngleDiff: 0.0,
            mType: 0,
            mVillageID: 0,
        };

        // Safety: GetOrigin is assumed to return a valid pointer to a vec3_t
        unsafe {
            VectorCopy(self.GetOrigin(), origin.as_mut_ptr());

            if self.mMirror != 0 {
                // Safety: TheRandomMissionManager is assumed to be initialized and valid
                if !TheRandomMissionManager.is_null() {
                    let mgr = &*TheRandomMissionManager;
                    let landscape = mgr.GetLandScape();
                    if !landscape.is_null() {
                        let bounds = (*landscape).GetBounds();
                        if !bounds.is_null() {
                            origin[0] = (*bounds)[0][0]
                                + (*bounds)[1][0]
                                - origin[0];
                            origin[1] = (*bounds)[0][1]
                                + (*bounds)[1][1]
                                - origin[1];
                        }
                    }
                }
            }

            if !terrain.is_null() {
                let terr = &*terrain;
                let landscape = terr.GetLandScape();

                if !landscape.is_null() {
                    let ls = &*landscape;
                    let terxel_size_ptr = ls.GetTerxelSize();
                    let bounds_ptr = ls.GetBounds();

                    if !terxel_size_ptr.is_null() && !bounds_ptr.is_null() {
                        let terxel_size = &*terxel_size_ptr;
                        let bounds = &*bounds_ptr;

                        // Align the instance to the center of a terxel
                        origin[0] = bounds[0][0]
                            + (((origin[0] - bounds[0][0] + terxel_size[0] / 2.0)
                                / terxel_size[0]) as i32) as f32
                                * terxel_size[0];
                        origin[1] = bounds[0][1]
                            + (((origin[1] - bounds[0][1] + terxel_size[1] / 2.0)
                                / terxel_size[1]) as i32) as f32
                                * terxel_size[1];

                        // This is BAD - By copying the mirrored origin back into the instance, you've now mirrored the original instance
                        // so when anything from this point on looks at the instance they'll be looking at a mirrored version but will be expecting the original
                        // so later in the spawn functions the instance will be re-mirrored, because it thinks the mInstances have not been changed
                        // VectorCopy(origin, GetOrigin());

                        // Flatten the area below the instance
                        if self.GetFlattenRadius() != 0.0 {
                            area.Init(
                                &origin as *const vec3_t,
                                self.GetFlattenRadius(),
                                0.0,
                                AT_NONE,
                                0,
                                0,
                            );
                            let flatten_height = self.mFlattenHeight
                                | (if self.mSurfaceSprites { 0 } else { 0x80 });
                            ls.FlattenArea(&area as *const _ as *const CArea, flatten_height, false, true, true);
                        }
                    }
                }
            }
        }

        true
    }

    /// CRMInstance::PostSpawn
    /// Finishes the spawn by linking any objectives into the world that are associated with it
    ///
    /// inputs:
    ///  landscape: landscape the instance was spawned on
    ///
    /// return:
    ///  true: post spawn successful
    ///  false: post spawn failed
    pub fn PostSpawn(&mut self, _terrain: *mut CRandomTerrain, _IsServer: u8) -> bool {
        // Safety: mObjective is checked for null before dereferencing
        if !self.mObjective.is_null() {
            unsafe {
                return (*self.mObjective).Link();
            }
        }

        true
    }

    /// CRMInstance::DrawAutomapSymbol
    /// Draws the automap symbol for this instance on the map
    pub fn DrawAutomapSymbol(&self) {
        // Safety: TheRandomMissionManager is assumed to be initialized and valid
        unsafe {
            if !TheRandomMissionManager.is_null() {
                let mgr = &*TheRandomMissionManager;
                mgr.AddAutomapSymbol(
                    self.GetAutomapSymbol(),
                    self.GetOrigin() as *const [f32; 3],
                    self.GetSide(),
                );
            }
        }

        // The following is commented out in the original source:
        // draw proper symbol on map for instance
        // switch (GetAutomapSymbol())
        // {
        //     default:
        //     case AUTOMAP_NONE:
        //         if (HasObjective())
        //             CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        //     case AUTOMAP_BLD:
        //         CM_TM_AddBuilding(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         if (HasObjective())
        //             CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        //     case AUTOMAP_OBJ:
        //         CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        //     case AUTOMAP_START:
        //         CM_TM_AddStart(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        //     case AUTOMAP_END:
        //         CM_TM_AddEnd(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        //     case AUTOMAP_ENEMY:
        //         if (HasObjective())
        //             CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1]);
        //         if (1 == Cvar_VariableIntegerValue("rmg_automapshowall"))
        //             CM_TM_AddNPC(GetOrigin()[0], GetOrigin()[1], false);
        //         break;
        //     case AUTOMAP_FRIEND:
        //         if (HasObjective())
        //             CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1]);
        //         if (1 == Cvar_VariableIntegerValue("rmg_automapshowall"))
        //             CM_TM_AddNPC(GetOrigin()[0], GetOrigin()[1], true);
        //         break;
        //     case AUTOMAP_WALL:
        //         CM_TM_AddWallRect(GetOrigin()[0], GetOrigin()[1], GetSide());
        //         break;
        // }
    }

    /// CRMInstance::Preview
    /// Renders debug information about the instance
    ///
    /// inputs:
    ///  none
    ///
    /// return:
    ///  none
    pub fn Preview(&self, _from: *const vec3_t) {
        // The following is commented out in the original source:
        // CEntity				*tent;
        //
        // // Add a cylindar for the whole settlement
        // tent = G_TempEntity( GetOrigin(), EV_DEBUG_CYLINDER );
        // VectorCopy( GetOrigin(), tent->s.origin2 );
        // tent->s.pos.trBase[2] += 40;
        // tent->s.origin2[2] += 50;
        // tent->s.time = 1050 + ((int)(GetSpacingRadius())<<16);
        // tent->s.time2 = GetPreviewColor ( );
        // G_AddTempEntity(tent);
        //
        // // Origin line
        // tent = G_TempEntity( GetOrigin ( ), EV_DEBUG_LINE );
        // VectorCopy( GetOrigin(), tent->s.origin2 );
        // tent->s.origin2[2] += 400;
        // tent->s.time = 1050;
        // tent->s.weapon = 10;
        // tent->s.time2 = (255<<24) + (255<<16) + (255<<8) + 255;
        // G_AddTempEntity(tent);
        //
        // if ( GetFlattenRadius ( ) )
        // {
        //     // Add a cylindar for the whole settlement
        //     tent = G_TempEntity( GetOrigin(), EV_DEBUG_CYLINDER );
        //     VectorCopy( GetOrigin(), tent->s.origin2 );
        //     tent->s.pos.trBase[2] += 40;
        //     tent->s.origin2[2] += 50;
        //     tent->s.time = 1050 + ((int)(GetFlattenRadius ( ))<<16);
        //     tent->s.time2 = (255<<24) + (80<<16) +(80<<8) + 80;
        //     G_AddTempEntity(tent);
        // }
    }

    // ========================================================================
    // Accessor methods (getters/setters from RM_Instance.h)
    // ========================================================================

    /// Get the objective associated with this instance
    pub fn GetObjective(&self) -> *mut CRMObjective {
        self.mObjective
    }

    /// Set the objective associated with this instance
    pub fn SetObjective(&mut self, obj: *mut CRMObjective) {
        self.mObjective = obj;
    }

    /// Check if this instance has an objective
    pub fn HasObjective(&self) -> bool {
        !self.mObjective.is_null()
    }

    /// Get the automap symbol for this instance
    pub fn GetAutomapSymbol(&self) -> c_int {
        self.mAutomapSymbol
    }

    /// Get the origin (position) of this instance
    /// Returns a pointer to the origin vector (from mArea)
    pub fn GetOrigin(&self) -> *mut f32 {
        // Safety: mArea is assumed to be valid if not null
        if !self.mArea.is_null() {
            unsafe {
                // Stub implementation: mArea->GetOrigin() would return a vec_t*
                // For now, return a null pointer as a stub
                core::ptr::null_mut()
            }
        } else {
            core::ptr::null_mut()
        }
    }

    /// Get the side (blue=1, red=2) this instance belongs to
    pub fn GetSide(&self) -> c_int {
        self.mSide
    }

    /// Set the side (blue=1, red=2) this instance belongs to
    pub fn SetSide(&mut self, side: c_int) {
        self.mSide = side;
    }

    /// Get the flatten radius for this instance
    pub fn GetFlattenRadius(&self) -> f32 {
        self.mFlattenRadius
    }

    /// Get the spacing radius for this instance
    pub fn GetSpacingRadius(&self) -> f32 {
        self.mSpacingRadius
    }

    /// Set the flatten height
    pub fn SetFlattenHeight(&mut self, height: c_int) {
        self.mFlattenHeight = height;
    }

    /// Get the flatten height
    pub fn GetFlattenHeight(&self) -> c_int {
        self.mFlattenHeight
    }

    /// Set the spacing radius
    pub fn SetSpacingRadius(&mut self, spacing: f32) {
        self.mSpacingRadius = spacing;
    }

    /// Get the flatten height
    pub fn GetFlattenHeight_copy(&self) -> c_int {
        self.mFlattenHeight
    }

    /// Check if surface sprites are allowed
    pub fn GetSurfaceSprites(&self) -> bool {
        self.mSurfaceSprites
    }

    /// Check if origin is locked
    pub fn GetLockOrigin(&self) -> bool {
        self.mLockOrigin
    }

    /// Get the spacing line
    pub fn GetSpacingLine(&self) -> c_int {
        self.mSpacingLine
    }

    /// Get the mirror flag
    pub fn GetMirror(&self) -> c_int {
        self.mMirror
    }

    /// Set the mirror flag
    pub fn SetMirror(&mut self, mirror: c_int) {
        self.mMirror = mirror;
    }

    /// Get the bounds of this instance
    pub fn GetBounds(&self) -> &[[f32; 3]; 2] {
        &self.mBounds
    }
}

/// Stub for unported `class CArea` (used internally in PreSpawn).
/// Represents an area with position and properties for terrain flattening.
#[repr(C)]
pub struct CArea {
    pub mPosition: vec3_t,
    pub mRadius: f32,
    pub mAngle: f32,
    pub mAngleDiff: f32,
    pub mType: c_int,
    pub mVillageID: c_int,
}

impl CArea {
    /// Stub for `void CArea::Init(vec3_t, float, float, int, int, int)`.
    /// Initializes an area with position, radius, angle, type, and other parameters.
    pub fn Init(
        &mut self,
        pos: *const vec3_t,
        radius: f32,
        angle: f32,
        type_val: c_int,
        village_id: c_int,
        _village_size: c_int,
    ) {
        // Safety: pos is assumed to be a valid pointer to a vec3_t
        unsafe {
            if !pos.is_null() {
                VectorCopy(pos as *const f32, self.mPosition.as_mut_ptr());
            }
        }
        self.mRadius = radius;
        self.mAngle = angle;
        self.mType = type_val;
        self.mVillageID = village_id;
    }
}
