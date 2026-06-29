#![allow(non_snake_case)]

// Preserved includes documentation from oracle/code/RMG/RM_Instance_BSP.cpp:
// #include "../server/exe_headers.h"
// #include "../qcommon/cm_local.h"
// #include "../server/server.h"
// #include "rm_headers.h"
// #include "rm_instance_bsp.h"
// #include "../client/vmachine.h"

/************************************************************************************************
 *
 * RM_Instance_BSP.cpp
 *
 * Implements the CRMBSPInstance class.  This class is reponsible for parsing a
 * bsp instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::ptr;

// LOCAL STUB: Forward declarations for types used in this file
pub struct CGPGroup;
pub struct CRMInstanceFile;
pub struct CRMInstance;
pub struct CRandomTerrain;
pub struct CRMArea;
pub struct CRMAreaManager;

// Type aliases matching the C definitions
pub type qboolean = c_int;
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [[f32; 3]; 2];

const MAX_QPATH: usize = 256;
const MIN_WORLD_COORD: f32 = -90000.0;
const ENTITYNUM_NONE: c_int = 1024;
const CONTENTS_TERRAIN: c_int = 0x8000;
const CONTENTS_SOLID: c_int = 0x0001;
const G2_NOCOLLIDE: c_int = 0;
const SIDE_RED: c_int = 0;
const AUTOMAP_NONE: c_int = 0;
const AUTOMAP_BLD: c_int = 1;
const AUTOMAP_OBJ: c_int = 2;
const AUTOMAP_START: c_int = 3;
const AUTOMAP_END: c_int = 4;
const AUTOMAP_ENEMY: c_int = 5;
const AUTOMAP_FRIEND: c_int = 6;
const AUTOMAP_WALL: c_int = 7;

extern "C" {
    /// strcpy - Copy C string
    /// char *strcpy(char *dest, const char *src);
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    /// strlen - Get length of C string
    /// size_t strlen(const char *s);
    fn strlen(s: *const c_char) -> usize;

    /// strcat - Concatenate C strings
    /// char *strcat(char *dest, const char *src);
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    /// strstr - Find substring in C string
    /// char *strstr(const char *haystack, const char *needle);
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;

    /// atof - Convert C string to floating point
    /// double atof(const char *nptr);
    fn atof(nptr: *const c_char) -> f64;

    /// atoi - Convert C string to integer
    /// int atoi(const char *nptr);
    fn atoi(nptr: *const c_char) -> c_int;

    /// strcmpi - Case-insensitive string comparison
    /// int strcmpi(const char *s1, const char *s2);
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;

    /// Q_stricmp - Case-insensitive string comparison (Quake version)
    /// int Q_stricmp(const char *s1, const char *s2);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// sprintf - Formatted string output
    /// int sprintf(char *str, const char *format, ...);
    fn sprintf(str: *mut c_char, format: *const c_char, ...) -> c_int;

    /// memset - Fill memory with a value
    /// void *memset(void *s, int c, size_t n);
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    /// Com_DPrintf - Debug printf
    /// void Com_DPrintf(const char *fmt, ...);
    fn Com_DPrintf(fmt: *const c_char, ...) -> ();

    /// SV_Trace - Server-side trace function
    /// void SV_Trace(trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passent, int contentmask, int g2TraceType, int useLod);
    fn SV_Trace(
        results: *mut trace_t,
        start: *const f32,
        mins: *const f32,
        maxs: *const f32,
        end: *const f32,
        passent: c_int,
        contentmask: c_int,
        g2TraceType: c_int,
        useLod: c_int,
    );

    /// VectorCopy macro
    fn VectorCopy(src: *const f32, dst: *mut f32);

    /// RAD2DEG macro - Radians to degrees conversion
    fn RAD2DEG(rad: f32) -> f32;

    /// DEG2RAD macro - Degrees to radians conversion
    fn DEG2RAD(deg: f32) -> f32;

    /// AngleNormalize360 - Normalize angle to 0-360 range
    /// float AngleNormalize360(float angle);
    fn AngleNormalize360(angle: f32) -> f32;

    /// CGPGroup::FindPairValue - Find value for key in parser group
    /// const char *CGPGroup::FindPairValue(const char *key, const char *default);
    fn CGPGroup_FindPairValue(
        this: *mut CGPGroup,
        key: *const c_char,
        default: *const c_char,
    ) -> *const c_char;

    /// CRMInstance base class methods
    fn CRMInstance_ctor(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> *mut CRMInstance;
    fn CRMInstance_SetMessage(this: *mut CRMInstance, msg: *const c_char);
    fn CRMInstance_SetDescription(this: *mut CRMInstance, desc: *const c_char);
    fn CRMInstance_SetInfo(this: *mut CRMInstance, info: *const c_char);
    fn CRMInstance_GetOrigin(this: *mut CRMInstance) -> *mut f32;
    fn CRMInstance_GetArea(this: *mut CRMInstance) -> *mut CRMArea;
    fn CRMInstance_GetBounds(this: *const CRMInstance) -> *const vec3pair_t;
    fn CRMInstance_GetSpacingRadius(this: *const CRMInstance) -> f32;
    fn CRMInstance_GetFlattenRadius(this: *const CRMInstance) -> f32;
    fn CRMInstance_HasObjective(this: *const CRMInstance) -> bool;
    fn CRMInstance_SetSide(this: *mut CRMInstance, side: c_int);
    fn CRMInstance_GetFilter(this: *const CRMInstance) -> *const c_char;
    fn CRMInstance_GetTeamFilter(this: *const CRMInstance) -> *const c_char;
    fn CRMInstance_DrawAutomapSymbol(this: *mut CRMInstance);

    /// CRMArea methods
    fn CRMArea_IsCollisionEnabled(this: *const CRMArea) -> bool;
    fn CRMArea_GetAngle(this: *const CRMArea) -> f32;

    /// CRandomTerrain methods
    fn CRandomTerrain_GetBounds(this: *const CRandomTerrain) -> *const vec3pair_t;
    fn CRandomTerrain_GetLandScape(this: *mut CRandomTerrain) -> *mut c_void;

    /// Landscape methods (from returned void pointer)
    fn CCMLandScape_GetTerxelSize(landscape: *mut c_void) -> *const vec3_t;
    fn CCMLandScape_GetBounds(landscape: *mut c_void) -> *const vec3pair_t;
    fn CCMLandScape_GetWaterHeight(landscape: *mut c_void) -> f32;
    fn CCMLandScape_GetWorldHeight(landscape: *mut c_void, origin: *mut f32, bounds: *const vec3pair_t, boolean: bool);
    fn CCMLandScape_irand(landscape: *mut c_void, min: c_int, max: c_int) -> c_int;

    /// Global TheRandomMissionManager
    static TheRandomMissionManager: *mut c_void;

    /// TheRandomMissionManager methods
    fn TheRandomMissionManager_GetLandScape(this: *mut c_void) -> *mut c_void;

    /// Game entity interface
    pub struct GameInterface;
    fn ge_GameSpawnRMGEntity(entity_str: *const c_char);

    /// Server entity parse point
    pub struct ServerState {
        entityParsePoint: *mut c_char,
    }

    static mut sv: ServerState;

    /// ge global - game interface
    pub static ge: *mut GameInterface;
}

// Trace result structure
#[repr(C)]
pub struct trace_t {
    allsolid: c_int,
    startsolid: c_int,
    fraction: f32,
    endpos: vec3_t,
    plane_normal: vec3_t,
    plane_dist: f32,
    surface_name: [c_char; 32],
    surfaceFlags: c_int,
    contents: c_int,
    entityNum: c_int,
    hitLoc: c_int,
}

// vec3_origin constant
pub const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

// CRMBSPInstance struct definition
#[repr(C)]
pub struct CRMBSPInstance {
    // Inherited from CRMInstance (embedded)
    mFilter: [c_char; MAX_QPATH],
    mTeamFilter: [c_char; MAX_QPATH],
    mBounds: vec3pair_t,
    mArea: *mut CRMArea,
    mObjective: *mut c_void,
    mSpacingRadius: f32,
    mFlattenRadius: f32,
    mSpacingLine: c_int,
    mLockOrigin: bool,
    mSurfaceSprites: bool,
    mAutomapSymbol: c_int,
    mEntityID: c_int,
    mSide: c_int,
    mMirror: c_int,
    mFlattenHeight: c_int,

    // CRMBSPInstance specific fields
    mBsp: [c_char; MAX_QPATH],
    mAngleVariance: f32,
    mBaseAngle: f32,
    mAngleDiff: f32,
    mHoleRadius: f32,
}

impl CRMBSPInstance {
    /************************************************************************************************
     * CRMBSPInstance::CRMBSPInstance
     *	constructs a building instance object using the given parser group
     *
     * inputs:
     *  instance:  parser group containing information about the building instance
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        unsafe {
            // Call parent constructor
            // In C++: CRMInstance ( instGroup, instFile )
            let mut instance: CRMBSPInstance = core::mem::zeroed();

            // Initialize parent class fields
            instance.mArea = ptr::null_mut();
            instance.mObjective = ptr::null_mut();
            instance.mSpacingRadius = 0.0;
            instance.mFlattenRadius = 0.0;
            instance.mFilter[0] = 0;
            instance.mTeamFilter[0] = 0;
            instance.mAutomapSymbol = 0;
            instance.mEntityID = 0;
            instance.mSide = 0;
            instance.mMirror = 0;
            instance.mFlattenHeight = 66;
            instance.mSpacingLine = 0;
            instance.mSurfaceSprites = true;
            instance.mLockOrigin = false;

            // strcpy(mBsp, instGroup->FindPairValue("file", ""));
            let file_str = CGPGroup_FindPairValue(
                instGroup,
                b"file\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );
            strcpy(instance.mBsp.as_mut_ptr(), file_str);

            // mAngleVariance	= DEG2RAD(atof(instGroup->FindPairValue("anglevariance", "0")));
            let anglevariance_str = CGPGroup_FindPairValue(
                instGroup,
                b"anglevariance\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mAngleVariance = DEG2RAD(atof(anglevariance_str) as f32);

            // mBaseAngle		= DEG2RAD(atof(instGroup->FindPairValue("baseangle", "0")));
            let baseangle_str = CGPGroup_FindPairValue(
                instGroup,
                b"baseangle\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mBaseAngle = DEG2RAD(atof(baseangle_str) as f32);

            // mAngleDiff		= DEG2RAD(atof(instGroup->FindPairValue("anglediff", "0")));
            let anglediff_str = CGPGroup_FindPairValue(
                instGroup,
                b"anglediff\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mAngleDiff = DEG2RAD(atof(anglediff_str) as f32);

            // mSpacingRadius	= atof( instGroup->FindPairValue ( "spacing", "100" ) );
            let spacing_str = CGPGroup_FindPairValue(
                instGroup,
                b"spacing\0".as_ptr() as *const c_char,
                b"100\0".as_ptr() as *const c_char,
            );
            instance.mSpacingRadius = atof(spacing_str) as f32;

            // mSpacingLine	= atoi( instGroup->FindPairValue ( "spacingline", "0" ) );
            let spacingline_str = CGPGroup_FindPairValue(
                instGroup,
                b"spacingline\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mSpacingLine = atoi(spacingline_str);

            // mSurfaceSprites = (!Q_stricmp ( instGroup->FindPairValue ( "surfacesprites", "no" ), "yes")) ? true : false;
            let surfacesprites_str = CGPGroup_FindPairValue(
                instGroup,
                b"surfacesprites\0".as_ptr() as *const c_char,
                b"no\0".as_ptr() as *const c_char,
            );
            instance.mSurfaceSprites = Q_stricmp(surfacesprites_str, b"yes\0".as_ptr() as *const c_char) == 0;

            // mLockOrigin     = (!Q_stricmp ( instGroup->FindPairValue ( "lockorigin", "no" ), "yes")) ? true : false;
            let lockorigin_str = CGPGroup_FindPairValue(
                instGroup,
                b"lockorigin\0".as_ptr() as *const c_char,
                b"no\0".as_ptr() as *const c_char,
            );
            instance.mLockOrigin = Q_stricmp(lockorigin_str, b"yes\0".as_ptr() as *const c_char) == 0;

            // mFlattenRadius	= atof( instGroup->FindPairValue ( "flatten", "0" ) );
            let flatten_str = CGPGroup_FindPairValue(
                instGroup,
                b"flatten\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mFlattenRadius = atof(flatten_str) as f32;

            // mHoleRadius		= atof( instGroup->FindPairValue ( "hole", "0" ) );
            let hole_str = CGPGroup_FindPairValue(
                instGroup,
                b"hole\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            instance.mHoleRadius = atof(hole_str) as f32;

            // const char * automapSymName = instGroup->FindPairValue ( "automap_symbol", "building" );
            let automap_sym_name = CGPGroup_FindPairValue(
                instGroup,
                b"automap_symbol\0".as_ptr() as *const c_char,
                b"building\0".as_ptr() as *const c_char,
            );

            // if (0 == strcmpi(automapSymName, "none"))	   	mAutomapSymbol = AUTOMAP_NONE ;
            if strcmpi(automap_sym_name, b"none\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_NONE;
            }
            // else if (0 == strcmpi(automapSymName, "building"))  	mAutomapSymbol = AUTOMAP_BLD  ;
            else if strcmpi(automap_sym_name, b"building\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_BLD;
            }
            // else if (0 == strcmpi(automapSymName, "objective")) 	mAutomapSymbol = AUTOMAP_OBJ  ;
            else if strcmpi(automap_sym_name, b"objective\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_OBJ;
            }
            // else if (0 == strcmpi(automapSymName, "start"))	   	mAutomapSymbol = AUTOMAP_START;
            else if strcmpi(automap_sym_name, b"start\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_START;
            }
            // else if (0 == strcmpi(automapSymName, "end"))	   	mAutomapSymbol = AUTOMAP_END  ;
            else if strcmpi(automap_sym_name, b"end\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_END;
            }
            // else if (0 == strcmpi(automapSymName, "enemy"))	   	mAutomapSymbol = AUTOMAP_ENEMY;
            else if strcmpi(automap_sym_name, b"enemy\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_ENEMY;
            }
            // else if (0 == strcmpi(automapSymName, "friend"))	   	mAutomapSymbol = AUTOMAP_FRIEND;
            else if strcmpi(automap_sym_name, b"friend\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_FRIEND;
            }
            // else if (0 == strcmpi(automapSymName, "wall"))	   	mAutomapSymbol = AUTOMAP_WALL;
            else if strcmpi(automap_sym_name, b"wall\0".as_ptr() as *const c_char) == 0 {
                instance.mAutomapSymbol = AUTOMAP_WALL;
            }
            // else mAutomapSymbol	= atoi( automapSymName );
            else {
                instance.mAutomapSymbol = atoi(automap_sym_name);
            }

            // optional instance objective strings
            // SetMessage(instGroup->FindPairValue("objective_message",""));
            let msg_str = CGPGroup_FindPairValue(
                instGroup,
                b"objective_message\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );
            // CRMInstance_SetMessage(&mut instance as *mut CRMBSPInstance as *mut CRMInstance, msg_str);

            // SetDescription(instGroup->FindPairValue("objective_description",""));
            let desc_str = CGPGroup_FindPairValue(
                instGroup,
                b"objective_description\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );
            // CRMInstance_SetDescription(&mut instance as *mut CRMBSPInstance as *mut CRMInstance, desc_str);

            // SetInfo(instGroup->FindPairValue("objective_info",""));
            let info_str = CGPGroup_FindPairValue(
                instGroup,
                b"objective_info\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );
            // CRMInstance_SetInfo(&mut instance as *mut CRMBSPInstance as *mut CRMInstance, info_str);

            // mBounds[0][0] = 0;
            // mBounds[0][1] = 0;
            // mBounds[1][0] = 0;
            // mBounds[1][1] = 0;
            instance.mBounds[0][0] = 0.0;
            instance.mBounds[0][1] = 0.0;
            instance.mBounds[1][0] = 0.0;
            instance.mBounds[1][1] = 0.0;

            // mBaseAngle += (TheRandomMissionManager->GetLandScape()->irand(0,mAngleVariance) - mAngleVariance/2);
            let landscape = TheRandomMissionManager_GetLandScape(TheRandomMissionManager);
            let angle_rand = CCMLandScape_irand(landscape, 0, (instance.mAngleVariance as c_int));
            instance.mBaseAngle += ((angle_rand as f32) - instance.mAngleVariance / 2.0);

            instance
        }
    }

    /************************************************************************************************
     * CRMBSPInstance::Spawn
     *	spawns a bsp into the world using the previously aquired origin
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
            {
                // float		yaw;
                // char		temp[10000];
                // char		*savePtr;
                // vec3_t		origin;
                // vec3_t		notmirrored;
                let mut yaw: f32;
                let mut temp: [c_char; 10000] = [0; 10000];
                let mut savePtr: *mut c_char;
                let mut origin: vec3_t = [0.0; 3];
                let mut notmirrored: vec3_t = [0.0; 3];

                // float	water_level = terrain->GetLandScape()->GetWaterHeight();
                let landscape = CRandomTerrain_GetLandScape(terrain);
                let water_level = CCMLandScape_GetWaterHeight(landscape);

                // const vec3_t&	  terxelSize = terrain->GetLandScape()->GetTerxelSize ( );
                let terxelSize = CCMLandScape_GetTerxelSize(landscape);

                // const vec3pair_t& bounds     = terrain->GetLandScape()->GetBounds();
                let bounds = CCMLandScape_GetBounds(landscape);

                // If this entity somehow lost its collision flag then boot it
                // if ( !GetArea().IsCollisionEnabled ( ) )
                if !CRMArea_IsCollisionEnabled(self.mArea) {
                    // {
                    //	return false;
                    // }
                    return false;
                }

                // copy out the unmirrored version
                // VectorCopy(GetOrigin(), notmirrored);
                VectorCopy(CRMInstance_GetOrigin(self as *mut CRMBSPInstance as *mut CRMInstance), notmirrored.as_mut_ptr());

                // we want to mirror it before determining the Z value just in case the landscape isn't perfectly mirrored
                // if (mMirror)
                if self.mMirror != 0 {
                    // {
                    //	GetOrigin()[0] = TheRandomMissionManager->GetLandScape()->GetBounds()[0][0] + TheRandomMissionManager->GetLandScape()->GetBounds()[1][0] - GetOrigin()[0];
                    let bounds_ptr = CCMLandScape_GetBounds(landscape);
                    if !bounds_ptr.is_null() {
                        let bounds_val = &*bounds_ptr;
                        let origin_ptr = CRMInstance_GetOrigin(self as *mut CRMBSPInstance as *mut CRMInstance);
                        *origin_ptr.add(0) = bounds_val[0][0] + bounds_val[1][0] - *origin_ptr.add(0);
                        //	GetOrigin()[1] = TheRandomMissionManager->GetLandScape()->GetBounds()[0][1] + TheRandomMissionManager->GetLandScape()->GetBounds()[1][1] - GetOrigin()[1];
                        *origin_ptr.add(1) = bounds_val[0][1] + bounds_val[1][1] - *origin_ptr.add(1);
                    }
                    // }
                }

                // Align the instance to the center of a terxel
                // GetOrigin ( )[0] = bounds[0][0] + (int)((GetOrigin ( )[0] - bounds[0][0] + terxelSize[0] / 2) / terxelSize[0]) * terxelSize[0];
                if !bounds.is_null() && !terxelSize.is_null() {
                    let bounds_val = &*bounds;
                    let terxelSize_val = &*terxelSize;
                    let origin_ptr = CRMInstance_GetOrigin(self as *mut CRMBSPInstance as *mut CRMInstance);
                    *origin_ptr.add(0) = bounds_val[0][0]
                        + (((*origin_ptr.add(0) - bounds_val[0][0] + terxelSize_val[0] / 2.0) / terxelSize_val[0]) as c_int as f32)
                            * terxelSize_val[0];
                    // GetOrigin ( )[1] = bounds[0][1] + (int)((GetOrigin ( )[1] - bounds[0][1] + terxelSize[1] / 2) / terxelSize[1]) * terxelSize[1];
                    *origin_ptr.add(1) = bounds_val[0][1]
                        + (((*origin_ptr.add(1) - bounds_val[0][1] + terxelSize_val[1] / 2.0) / terxelSize_val[1]) as c_int as f32)
                            * terxelSize_val[1];
                }

                // Make sure the bsp is resting on the ground, not below or above it
                // NOTE: This check is basically saying "is this instance not a bridge", because when instances are created they are all
                // placed above the world's Z boundary, EXCEPT FOR BRIDGES. So this call to GetWorldHeight will move all other instances down to
                // ground level except bridges
                let origin_ptr = CRMInstance_GetOrigin(self as *mut CRMBSPInstance as *mut CRMInstance);
                if !bounds.is_null() {
                    let bounds_val = &*bounds;
                    if *origin_ptr.add(2) > bounds_val[1][2] {
                        // {
                        if self.mFlattenRadius != 0.0 {
                            //	if( GetFlattenRadius() )
                            // {
                            let bounds_for_height = CRMInstance_GetBounds(self as *const CRMBSPInstance as *const CRMInstance);
                            CCMLandScape_GetWorldHeight(landscape, origin_ptr, bounds_for_height, false);
                            // terrain->GetLandScape()->GetWorldHeight ( GetOrigin(), GetBounds ( ), false );
                            *origin_ptr.add(2) += 5.0;
                            // GetOrigin()[2] += 5;
                            // }
                        } else if IsServer != 0 {
                            // else if (IsServer)
                            // {	// if this instance does not flatten the ground around it, do a trace to more accurately determine its Z value
                            let mut tr: trace_t = core::mem::zeroed();
                            // trace_t		tr;
                            let mut end: vec3_t = [0.0; 3];
                            // vec3_t		end;
                            let mut start: vec3_t = [0.0; 3];
                            // vec3_t		start;

                            VectorCopy(origin_ptr, end.as_mut_ptr());
                            // VectorCopy(GetOrigin(), end);
                            VectorCopy(origin_ptr, start.as_mut_ptr());
                            // VectorCopy(GetOrigin(), start);
                            // start the trace below the top height of the landscape
                            if !bounds.is_null() {
                                let bounds_val = &*bounds;
                                start[2] = bounds_val[1][2] - 1.0;
                            }
                            // start[2] = TheRandomMissionManager->GetLandScape()->GetBounds()[1][2] - 1;
                            // end the trace at the bottom of the world
                            end[2] = MIN_WORLD_COORD;
                            // end[2] = MIN_WORLD_COORD;

                            memset(&mut tr as *mut trace_t as *mut c_void, 0, std::mem::size_of::<trace_t>());
                            // memset ( &tr, 0, sizeof ( tr ) );
                            SV_Trace(
                                &mut tr,
                                start.as_ptr(),
                                vec3_origin.as_ptr(),
                                vec3_origin.as_ptr(),
                                end.as_ptr(),
                                ENTITYNUM_NONE,
                                CONTENTS_TERRAIN | CONTENTS_SOLID,
                                G2_NOCOLLIDE,
                                0,
                            );
                            // SV_Trace( &tr, start, vec3_origin, vec3_origin, end, ENTITYNUM_NONE, CONTENTS_TERRAIN|CONTENTS_SOLID, G2_NOCOLLIDE, 0);

                            if (tr.contents & CONTENTS_TERRAIN) == 0 || tr.fraction == 1.0 {
                                // if( !(tr.contents & CONTENTS_TERRAIN) || (tr.fraction == 1.0) )
                                // {
                                if 0 != 0 {
                                    // if ( 0 )
                                    // assert(0); // this should never happen
                                }

                                // restore the unmirrored origin
                                VectorCopy(notmirrored.as_ptr(), origin_ptr);
                                // VectorCopy( notmirrored, GetOrigin() );
                                // don't spawn
                                return false;
                                // return false;
                                // }
                            }
                            // assign the Z-value to wherever it hit the terrain
                            *origin_ptr.add(2) = tr.endpos[2];
                            // GetOrigin()[2] = tr.endpos[2];
                            // lower it a little, otherwise the bottom of the instance might be exposed if on some weird sloped terrain
                            *origin_ptr.add(2) -= 16.0;
                            // GetOrigin()[2] -= 16; // FIXME: would it be better to use a number related to the instance itself like 1/5 it's height or something...
                            // }
                        }
                        // }
                    } else {
                        // }
                        // else
                        // {
                        let bounds_for_height = CRMInstance_GetBounds(self as *const CRMBSPInstance as *const CRMInstance);
                        CCMLandScape_GetWorldHeight(landscape, origin_ptr, bounds_for_height, true);
                        // terrain->GetLandScape()->GetWorldHeight ( GetOrigin(), GetBounds ( ), true );
                        // }
                    }
                }

                // save away the origin
                VectorCopy(origin_ptr, origin.as_mut_ptr());
                // VectorCopy(GetOrigin(), origin);
                // make sure not to spawn if in water
                if !CRMInstance_HasObjective(self as *const CRMBSPInstance as *const CRMInstance)
                    && *origin_ptr.add(2) < water_level
                {
                    // if (!HasObjective() && GetOrigin()[2] < water_level)
                    return false;
                    // return false;
                }
                // restore the origin
                VectorCopy(origin.as_ptr(), origin_ptr);
                // VectorCopy(origin, GetOrigin());

                if self.mMirror != 0 {
                    // if (mMirror)
                    // {	// change blue things to red for symmetric maps
                    if strlen(self.mFilter.as_ptr()) > 0 {
                        // if (strlen(mFilter) > 0)
                        // {
                        let blue = strstr(self.mFilter.as_ptr(), b"blue\0".as_ptr() as *const c_char);
                        // char * blue = strstr(mFilter,"blue");
                        if !blue.is_null() {
                            // if (blue)
                            // {
                            *blue = 0;
                            // blue[0] = (char) 0;
                            strcat(self.mFilter.as_mut_ptr(), b"red\0".as_ptr() as *const c_char);
                            // strcat(mFilter, "red");
                            // SetSide(SIDE_RED);
                            self.mSide = SIDE_RED;
                            // }
                        }
                        // }
                    }
                    if strlen(self.mTeamFilter.as_ptr()) > 0 {
                        // if (strlen(mTeamFilter) > 0)
                        // {
                        let blue = strstr(self.mTeamFilter.as_ptr(), b"blue\0".as_ptr() as *const c_char);
                        // char * blue = strstr(mTeamFilter,"blue");
                        if !blue.is_null() {
                            // if (blue)
                            // {
                            strcpy(self.mTeamFilter.as_mut_ptr(), b"red\0".as_ptr() as *const c_char);
                            // strcpy(mTeamFilter, "red");
                            // SetSide(SIDE_RED);
                            self.mSide = SIDE_RED;
                            // }
                        }
                        // }
                    }
                    let area_angle = CRMArea_GetAngle(self.mArea);
                    yaw = RAD2DEG(area_angle + self.mBaseAngle) + 180.0;
                    // yaw = RAD2DEG(mArea->GetAngle() + mBaseAngle) + 180;
                } else {
                    // }
                    // else
                    // {
                    let area_angle = CRMArea_GetAngle(self.mArea);
                    yaw = RAD2DEG(area_angle + self.mBaseAngle);
                    // yaw = RAD2DEG(mArea->GetAngle() + mBaseAngle);
                    // }
                }

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
                sprintf(
                    temp.as_mut_ptr(),
                    b"{\n\"classname\"   \"misc_bsp\"\n\"bspmodel\"    \"%s\"\n\"origin\"      \"%f %f %f\"\n\"angles\"      \"0 %f 0\"\n\"filter\"      \"%s\"\n\"teamfilter\"  \"%s\"\n\"spacing\"\t \"%d\"\n\"flatten\"\t \"%d\"\n}\n\0".as_ptr() as *const c_char,
                    self.mBsp.as_ptr(),
                    *origin_ptr.add(0),
                    *origin_ptr.add(1),
                    *origin_ptr.add(2),
                    AngleNormalize360(yaw),
                    self.mFilter.as_ptr(),
                    self.mTeamFilter.as_ptr(),
                    self.mSpacingRadius as c_int,
                    self.mFlattenRadius as c_int,
                );

                if IsServer != 0 {
                    // if (IsServer)
                    // {	// only allow for true spawning on the server
                    // savePtr = sv.entityParsePoint;
                    // sv.entityParsePoint = temp;
                    // //		VM_Call( cgvm, GAME_SPAWN_RMG_ENTITY );
                    // //	char *s;
                    let bufferSize: c_int = 1024;
                    // int bufferSize = 1024;
                    let mut buffer: [c_char; 1024] = [0; 1024];
                    // char buffer[1024];

                    // //	s = COM_Parse( (const char **)&sv.entityParsePoint );
                    // Q_strncpyz( buffer, sv.entityParsePoint, bufferSize );
                    // if ( sv.entityParsePoint && sv.entityParsePoint[0] )
                    // {
                    //	ge->GameSpawnRMGEntity(sv.entityParsePoint);
                    // }
                    ge_GameSpawnRMGEntity(temp.as_ptr());
                    // sv.entityParsePoint = savePtr;
                    // }
                }

                #[cfg(not(feature = "DEDICATED"))]
                {
                    // #ifndef DEDICATED
                    CRMInstance_DrawAutomapSymbol(self as *mut CRMBSPInstance as *mut CRMInstance);
                    // DrawAutomapSymbol();
                    // #endif
                }

                Com_DPrintf(
                    b"RMG:  Building '%s' spawned at (%f %f %f)\n\0".as_ptr() as *const c_char,
                    self.mBsp.as_ptr(),
                    *origin_ptr.add(0),
                    *origin_ptr.add(1),
                    *origin_ptr.add(2),
                );
                // Com_DPrintf( "RMG:  Building '%s' spawned at (%f %f %f)\n", mBsp, GetOrigin()[0], GetOrigin()[1], GetOrigin()[2] );

                // now restore the instances un-mirrored origin
                // NOTE: all this origin flipping, setting the side etc... should be done when mMirror is set
                // because right after this function is called, mMirror is set to 0 but all the instance data is STILL MIRRORED -- not good
                VectorCopy(notmirrored.as_ptr(), origin_ptr);
                // VectorCopy(notmirrored, GetOrigin());

                return true;
            }

            #[cfg(feature = "PRE_RELEASE_DEMO")]
            {
                // #endif  // PRE_RELEASE_DEMO
                return false;
            }
        }
    }
}
