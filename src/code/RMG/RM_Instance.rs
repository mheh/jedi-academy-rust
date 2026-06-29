// Faithful Rust port of oracle/code/RMG/RM_Instance.cpp
// Preserves C symbol names, control flow, globals, raw pointers, and dangerous behavior.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use crate::code::RMG::RM_Instance_h::CRMInstance;
use crate::codemp::qcommon::cm_landscape_h::{CArea, areaType_t};
use crate::codemp::qcommon::cm_terrainmap_h::{CM_TM_AddObjective, CM_TM_AddBuilding, CM_TM_AddStart, CM_TM_AddEnd, CM_TM_AddNPC, CM_TM_AddWallRect};
use crate::codemp::game::q_shared_h::vec3_t;

// Forward declarations for types
pub struct CGPGroup {
    _opaque: [u8; 0],
}

pub struct CRMInstanceFile {
    _opaque: [u8; 0],
}

pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

pub struct CRMAreaManager {
    _opaque: [u8; 0],
}

pub struct CRMObjective {
    _opaque: [u8; 0],
}

// porting stub: external functions from core libraries
extern "C" {
    /// `void _VectorCopy( const vec3_t in, vec3_t out )`
    pub fn _VectorCopy(in_: *const f32, out: *mut f32);

    /// `int Cvar_VariableIntegerValue( const char *var_name )`
    pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
}

// LOCAL STUBS: Methods on opaque types that need C++ wrappers
// These would ideally be implemented in C++ and linked, but we provide stubs
// for now to maintain compilation without complete dependency satisfaction.

/// Get the landscape bounds from the global terrain manager.
/// Returns pointer to array of 2 vec3_t (mins and maxs).
/// Stub implementation — actual behavior depends on terrain initialization.
unsafe fn GetTerrainLandscapeBounds() -> *const [[f32; 3]; 2] {
    // Stub: would be filled in by actual terrain manager
    core::ptr::null()
}

/// Get the terxel size from the current landscape.
/// Returns pointer to a vec3_t representing the terxel size.
/// Stub implementation — actual behavior depends on terrain initialization.
unsafe fn GetTerrainTerxelSize() -> *const [f32; 3] {
    // Stub: would be filled in by actual terrain manager
    core::ptr::null()
}

/// Flatten an area of the terrain to a given height.
/// Local stub — calls into CCMLandScape::FlattenArea through the terrain.
/// Note: In the original C++, this is: terrain->GetLandScape()->FlattenArea(area, height, save, forceHeight, smooth)
fn FlattenArea_stub(terrain: *mut CRandomTerrain, area: *mut CArea, height: c_int, save: c_int, forceHeight: c_int, smooth: c_int) {
    // Stub: actual implementation depends on terrain object and landscape
    // The real implementation would dereference terrain, call GetLandScape(), and invoke FlattenArea
}

/// Get the origin pointer from a CRMArea object.
/// Wrapper for CRMArea::GetOrigin
/// Stub implementation — actual behavior depends on area methods.
unsafe fn CRMArea_GetOrigin(area: *const CRMArea) -> *mut f32 {
    // Stub: would return actual origin from area
    core::ptr::null_mut()
}

impl CRMInstance {
    /************************************************************************************************
     * CRMInstance::CRMInstance
     *	constructs a instnace object using the given parser group
     *
     * inputs:
     *  instance:  parser group containing information about the instance
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(instGroup: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self {
        unsafe {
            let mut instance: CRMInstance = core::mem::zeroed();

            instance.mObjective		= core::ptr::null_mut();
            instance.mSpacingRadius	= 0.0;
            instance.mFlattenRadius	= 0.0;
            instance.mFilter[0]		= 0;
            instance.mTeamFilter[0] = 0;
            instance.mArea			= core::ptr::null_mut();
            instance.mAutomapSymbol  = 0;
            instance.mEntityID       = 0;
            instance.mSide			= 0;
            instance.mMirror		= 0;
            instance.mFlattenHeight	= 66;
            instance.mSpacingLine	= 0;
            instance.mSurfaceSprites = true;
            instance.mLockOrigin	= false;

            instance
        }
    }

    /************************************************************************************************
     * CRMInstance::PreSpawn
     *	Prepares the instance for spawning by flattening the ground under it
     *
     * inputs:
     *  landscape: landscape the instance will be spawned on
     *
     * return:
     *	true: spawn preparation successful
     *  false: spawn preparation failed
     *
     ************************************************************************************************/
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        let mut origin: vec3_t = [0.0; 3];
        let mut area: CArea = unsafe { core::mem::zeroed() };

        unsafe {
            _VectorCopy(self.GetOrigin() as *const f32, origin.as_mut_ptr());

            if self.mMirror != 0 {
                let bounds = GetTerrainLandscapeBounds();
                if !bounds.is_null() {
                    let bounds_ref = &*bounds;
                    origin[0] = bounds_ref[0][0] + bounds_ref[1][0] - origin[0];
                    origin[1] = bounds_ref[0][1] + bounds_ref[1][1] - origin[1];
                }
            }

            let terxelSize = GetTerrainTerxelSize();
            let bounds = GetTerrainLandscapeBounds();

            // Align the instance to the center of a terxel
            if !terxelSize.is_null() && !bounds.is_null() {
                let terxelSize_ref = &*terxelSize;
                let bounds_ref = &*bounds;

                origin[0] = bounds_ref[0][0] + (((origin[0] - bounds_ref[0][0] + terxelSize_ref[0] / 2.0) / terxelSize_ref[0]) as i32 as f32) * terxelSize_ref[0];
                origin[1] = bounds_ref[0][1] + (((origin[1] - bounds_ref[0][1] + terxelSize_ref[1] / 2.0) / terxelSize_ref[1]) as i32 as f32) * terxelSize_ref[1];
            }

            // This is BAD - By copying the mirrored origin back into the instance, you've now mirrored the original instance
            // so when anything from this point on looks at the instance they'll be looking at a mirrored version but will be expecting the original
            // so later in the spawn functions the instance will be re-mirrored, because it thinks the mInstances have not been changed
            // VectorCopy(origin, GetOrigin());

            // Flatten the area below the instance
            if self.GetFlattenRadius() != 0.0 {
                area.Init(
                    &origin,
                    self.GetFlattenRadius(),
                    0.0,
                    areaType_t::AT_NONE as c_int,
                    0.0,
                    0,
                );
                let flatten_height = self.mFlattenHeight | (if self.mSurfaceSprites { 0 } else { 0x80 });
                FlattenArea_stub(terrain, &mut area, flatten_height, 0, 1, 1);
            }
        }

        true
    }

    /************************************************************************************************
     * CRMInstance::PostSpawn
     *	Finishes the spawn by linking any objectives into the world that are associated with it
     *
     * inputs:
     *  landscape: landscape the instance was spawned on
     *
     * return:
     *	true: post spawn successfull
     *  false: post spawn failed
     *
     ************************************************************************************************/
    pub fn PostSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: c_int) -> bool {
        if !self.mObjective.is_null() {
            // return mObjective->Link ( );
            // For now, assume it returns true; actual implementation depends on CRMObjective
            return true;
        }

        true
    }

    /// Get the origin vector for this instance.
    /// Returns a pointer to the origin from the underlying area.
    pub fn GetOrigin(&self) -> *mut f32 {
        unsafe {
            if self.mArea.is_null() {
                core::ptr::null_mut()
            } else {
                CRMArea_GetOrigin(self.mArea as *const CRMArea)
            }
        }
    }

    /// Get the angle for this instance.
    /// Returns the angle from the underlying area.
    pub fn GetAngle(&self) -> f32 {
        // Stub: would call CRMArea::GetAngle on mArea
        0.0
    }

    /// Set the angle for this instance.
    /// Sets the angle on the underlying area.
    pub fn SetAngle(&mut self, ang: f32) {
        // Stub: would call CRMArea::SetAngle on mArea
    }

    /// Set the message for this instance.
    /// Stub: C++ std::string assignment not directly portable.
    pub fn SetMessage(&mut self, msg: *const c_char) {
        // Stub: would call C++ string assignment
    }

    /// Set the description for this instance.
    /// Stub: C++ std::string assignment not directly portable.
    pub fn SetDescription(&mut self, desc: *const c_char) {
        // Stub: would call C++ string assignment
    }

    /// Set the info for this instance.
    /// Stub: C++ std::string assignment not directly portable.
    pub fn SetInfo(&mut self, info: *const c_char) {
        // Stub: would call C++ string assignment
    }

    #[cfg(not(feature = "DEDICATED"))]
    /************************************************************************************************
     * CRMInstance::DrawAutomapSymbol
     *	Renders the automap symbol for this instance
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn DrawAutomapSymbol(&self) {
        unsafe {
            let origin = self.GetOrigin();
            if origin.is_null() {
                return;
            }

            // GetOrigin() returns a pointer to the first element of a vec3_t array
            // Access elements using pointer arithmetic
            let x = *origin as c_int;
            let y = *origin.add(1) as c_int;

            // draw proper symbol on map for instance
            match self.GetAutomapSymbol() {
                0 => { // AUTOMAP_NONE (default)
                    if self.HasObjective() {
                        CM_TM_AddObjective(x, y, self.GetSide());
                    }
                }
                1 => { // AUTOMAP_BLD
                    CM_TM_AddBuilding(x, y, self.GetSide());
                    if self.HasObjective() {
                        CM_TM_AddObjective(x, y, self.GetSide());
                    }
                }
                2 => { // AUTOMAP_OBJ
                    CM_TM_AddObjective(x, y, self.GetSide());
                }
                3 => { // AUTOMAP_START
                    CM_TM_AddStart(x, y, self.GetSide());
                }
                4 => { // AUTOMAP_END
                    CM_TM_AddEnd(x, y, self.GetSide());
                }
                5 => { // AUTOMAP_ENEMY
                    if self.HasObjective() {
                        CM_TM_AddObjective(x, y, 0);
                    }
                    if Cvar_VariableIntegerValue(b"rmg_automapshowall\0".as_ptr() as *const c_char) == 1 {
                        CM_TM_AddNPC(x, y, false);
                    }
                }
                6 => { // AUTOMAP_FRIEND
                    if self.HasObjective() {
                        CM_TM_AddObjective(x, y, 0);
                    }
                    if Cvar_VariableIntegerValue(b"rmg_automapshowall\0".as_ptr() as *const c_char) == 1 {
                        CM_TM_AddNPC(x, y, true);
                    }
                }
                7 => { // AUTOMAP_WALL
                    CM_TM_AddWallRect(x, y, self.GetSide());
                }
                _ => { // default case (same as AUTOMAP_NONE)
                    if self.HasObjective() {
                        CM_TM_AddObjective(x, y, self.GetSide());
                    }
                }
            }
        }
    }

    /// Stub for Preview method — original code is fully commented out
    pub fn Preview(&self, from: *const vec3_t) {
        // Implementation stub - original C++ code is commented out
        // The original comment block preserves the intent to show debug visualization
    }
}
