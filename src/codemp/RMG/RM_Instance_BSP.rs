//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::exe_headers_h::*;

/************************************************************************************************
 *
 * RM_Instance_BSP.cpp
 *
 * Implements the CRMBSPInstance class.  This class is reponsible for parsing a
 * bsp instance as well as spawning it into a landscape.
 *
 ************************************************************************************************/

// #include "../qcommon/cm_local.h"
use crate::codemp::qcommon::cm_local_h::*;
// #include "../server/server.h"
use crate::codemp::server::server_h::*;
// #include "RM_Headers.h"
use crate::codemp::RMG::RM_Headers_h::*;
// #include "RM_Instance_BSP.h"
use crate::codemp::RMG::RM_Instance_BSP_h::*;

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut};

// C stdlib/stdio — system includes (<string.h>, <stdlib.h>, <stdio.h>), not ported modules
extern "C" {
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atof(s: *const c_char) -> f64;
    fn atoi(s: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn sprintf(str_: *mut c_char, fmt: *const c_char, ...) -> c_int;
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
// Porting note: C++ constructor body; the base-class init list `: CRMInstance(instGroup, instFile)`
// must be handled externally (by the caller) before invoking this ctor on the live Self.
pub unsafe fn ctor(&mut self, instGroup: *mut CGPGroup, instFile: &mut CRMInstanceFile) {
    strcpy(self.mBsp.as_mut_ptr(), (*instGroup).FindPairValue(c"file".as_ptr(), c"".as_ptr()));

    self.mAngleVariance = DEG2RAD(atof((*instGroup).FindPairValue(c"anglevariance".as_ptr(), c"0".as_ptr())));
    self.mBaseAngle     = DEG2RAD(atof((*instGroup).FindPairValue(c"baseangle".as_ptr(),     c"0".as_ptr())));
    self.mAngleDiff     = DEG2RAD(atof((*instGroup).FindPairValue(c"anglediff".as_ptr(),     c"0".as_ptr())));
    self.mSpacingRadius = atof((*instGroup).FindPairValue(c"spacing".as_ptr(),     c"100".as_ptr())) as f32;
    self.mSpacingLine   = atoi((*instGroup).FindPairValue(c"spacingline".as_ptr(), c"0".as_ptr()));
    // C++: (!Q_stricmp(x,"yes")) ? true : false  — Q_stricmp returns 0 on match; logical-NOT gives 1; ternary gives bool
    // Porting note: C `!` is logical NOT; translated as `== 0` to avoid Rust bitwise `!`
    self.mSurfaceSprites = Q_stricmp((*instGroup).FindPairValue(c"surfacesprites".as_ptr(), c"no".as_ptr()), c"yes".as_ptr()) == 0;
    self.mLockOrigin     = Q_stricmp((*instGroup).FindPairValue(c"lockorigin".as_ptr(),     c"no".as_ptr()), c"yes".as_ptr()) == 0;
    self.mFlattenRadius  = atof((*instGroup).FindPairValue(c"flatten".as_ptr(), c"0".as_ptr())) as f32;
    self.mHoleRadius     = atof((*instGroup).FindPairValue(c"hole".as_ptr(),    c"0".as_ptr())) as f32;

    let automapSymName: *const c_char = (*instGroup).FindPairValue(c"automap_symbol".as_ptr(), c"building".as_ptr());
    if      0 == Q_stricmp(automapSymName, c"none".as_ptr())      { self.mAutomapSymbol = AUTOMAP_NONE;   }
    else if 0 == Q_stricmp(automapSymName, c"building".as_ptr())  { self.mAutomapSymbol = AUTOMAP_BLD;    }
    else if 0 == Q_stricmp(automapSymName, c"objective".as_ptr()) { self.mAutomapSymbol = AUTOMAP_OBJ;    }
    else if 0 == Q_stricmp(automapSymName, c"start".as_ptr())     { self.mAutomapSymbol = AUTOMAP_START;  }
    else if 0 == Q_stricmp(automapSymName, c"end".as_ptr())       { self.mAutomapSymbol = AUTOMAP_END;    }
    else if 0 == Q_stricmp(automapSymName, c"enemy".as_ptr())     { self.mAutomapSymbol = AUTOMAP_ENEMY;  }
    else if 0 == Q_stricmp(automapSymName, c"friend".as_ptr())    { self.mAutomapSymbol = AUTOMAP_FRIEND; }
    else if 0 == Q_stricmp(automapSymName, c"wall".as_ptr())      { self.mAutomapSymbol = AUTOMAP_WALL;   }
    else { self.mAutomapSymbol = atoi(automapSymName); }

    // optional instance objective strings
    self.SetMessage((*instGroup).FindPairValue(c"objective_message".as_ptr(),     c"".as_ptr()));
    self.SetDescription((*instGroup).FindPairValue(c"objective_description".as_ptr(), c"".as_ptr()));
    self.SetInfo((*instGroup).FindPairValue(c"objective_info".as_ptr(),           c"".as_ptr()));

    self.mBounds[0][0] = 0.0;
    self.mBounds[0][1] = 0.0;
    self.mBounds[1][0] = 0.0;
    self.mBounds[1][1] = 0.0;

    // C++ float/int promotion: irand(int,int) receives mAngleVariance (float) cast to int;
    // result (int) minus mAngleVariance/2 (float) promotes int to float; added to mBaseAngle (float)
    self.mBaseAngle +=
        (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).irand(0, self.mAngleVariance as c_int) as f32
        - self.mAngleVariance / 2.0_f32;
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
pub unsafe fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
    #[cfg(not(feature = "pre_release_demo"))]
    {
//	TEntity*	ent;
        let mut yaw: f32 = 0.0;
        let mut temp: [c_char; 10000] = [0; 10000];
        let savePtr: *mut c_char;
        let mut origin: vec3_t = [0.0; 3];
        let mut notmirrored: vec3_t = [0.0; 3];
        let water_level: f32 = (*terrain).GetLandScape().GetWaterHeight();

        let terxelSize: &vec3_t    = (*terrain).GetLandScape().GetTerxelSize();
        let bounds: &vec3pair_t    = (*terrain).GetLandScape().GetBounds();

        // If this entity somehow lost its collision flag then boot it
        if !self.GetArea().IsCollisionEnabled() {
            return false;
        }

        // copy out the unmirrored version
        VectorCopy(self.GetOrigin(), notmirrored.as_mut_ptr());

        // we want to mirror it before determining the Z value just in case the landscape isn't perfectly mirrored
        if self.mMirror != 0 {
            *self.GetOrigin().add(0) = (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).GetBounds()[0][0]
                + (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).GetBounds()[1][0]
                - *self.GetOrigin().add(0);
            *self.GetOrigin().add(1) = (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).GetBounds()[0][1]
                + (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).GetBounds()[1][1]
                - *self.GetOrigin().add(1);
        }

        // Align the instance to the center of a terxel
        *self.GetOrigin().add(0) = bounds[0][0]
            + ((*self.GetOrigin().add(0) - bounds[0][0] + terxelSize[0] / 2.0) / terxelSize[0]) as c_int as f32 * terxelSize[0];
        *self.GetOrigin().add(1) = bounds[0][1]
            + ((*self.GetOrigin().add(1) - bounds[0][1] + terxelSize[1] / 2.0) / terxelSize[1]) as c_int as f32 * terxelSize[1];

        // Make sure the bsp is resting on the ground, not below or above it
        // NOTE: This check is basically saying "is this instance not a bridge", because when instances are created they are all
        // placed above the world's Z boundary, EXCEPT FOR BRIDGES. So this call to GetWorldHeight will move all other instances down to
        // ground level except bridges
        if *self.GetOrigin().add(2) > (*terrain).GetBounds()[1][2] {
            if self.GetFlattenRadius() != 0.0 {
                (*terrain).GetLandScape().GetWorldHeight(self.GetOrigin(), self.GetBounds(), false);
                *self.GetOrigin().add(2) += 5.0;
            } else if IsServer != 0 {
            // if this instance does not flatten the ground around it, do a trace to more accurately determine its Z value
                let mut tr: trace_t = core::mem::zeroed();
                let mut end: vec3_t = [0.0; 3];
                let mut start: vec3_t = [0.0; 3];

                VectorCopy(self.GetOrigin(), end.as_mut_ptr());
                VectorCopy(self.GetOrigin(), start.as_mut_ptr());
                // start the trace below the top height of the landscape
                start[2] = (*(*addr_of!(TheRandomMissionManager)).GetLandScape()).GetBounds()[1][2] - 1.0;
                // end the trace at the bottom of the world
                end[2] = MIN_WORLD_COORD;

                Com_Memset(&mut tr as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<trace_t>());
                SV_Trace(
                    &mut tr,
                    start.as_ptr(),
                    // Porting note: vec3_origin is a static extern array; addr_of! to get *const [f32;3], cast to *const f32
                    addr_of!(vec3_origin) as *const f32,
                    addr_of!(vec3_origin) as *const f32,
                    end.as_ptr(),
                    -1,
                    CONTENTS_TERRAIN|CONTENTS_SOLID,
                    qfalse,
                    0,
                    10,
                );

                if !(tr.contents & CONTENTS_TERRAIN != 0) || tr.fraction == 1.0 {
                    if false {
                    assert!(false); // this should never happen
                    }

                    // restore the unmirrored origin
                    VectorCopy(notmirrored.as_ptr(), self.GetOrigin());
                    // don't spawn
                    return false;
                }
                // assign the Z-value to wherever it hit the terrain
                *self.GetOrigin().add(2) = tr.endpos[2];
                // lower it a little, otherwise the bottom of the instance might be exposed if on some weird sloped terrain
                *self.GetOrigin().add(2) -= 16.0; // FIXME: would it be better to use a number related to the instance itself like 1/5 it's height or something...
            }

        } else {
            (*terrain).GetLandScape().GetWorldHeight(self.GetOrigin(), self.GetBounds(), true);
        }

        // save away the origin
        VectorCopy(self.GetOrigin(), origin.as_mut_ptr());
        // make sure not to spawn if in water
        if !self.HasObjective() && *self.GetOrigin().add(2) < water_level {
            return false;
        }
        // restore the origin
        VectorCopy(origin.as_ptr(), self.GetOrigin());

        if self.mMirror != 0 {
        // change blue things to red for symmetric maps
            if strlen(self.mFilter.as_ptr()) > 0 {
                let blue: *mut c_char = strstr(self.mFilter.as_ptr(), c"blue".as_ptr());
                if !blue.is_null() {
                    *blue = 0 as c_char;
                    strcat(self.mFilter.as_mut_ptr(), c"red".as_ptr());
                    self.SetSide(SIDE_RED);
                }
            }
            if strlen(self.mTeamFilter.as_ptr()) > 0 {
                let blue: *mut c_char = strstr(self.mTeamFilter.as_ptr(), c"blue".as_ptr());
                if !blue.is_null() {
                    strcpy(self.mTeamFilter.as_mut_ptr(), c"red".as_ptr());
                    self.SetSide(SIDE_RED);
                }
            }
            yaw = RAD2DEG((*self.mArea).GetAngle() + self.mBaseAngle) + 180.0;
        } else {
            yaw = RAD2DEG((*self.mArea).GetAngle() + self.mBaseAngle);
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
        sprintf(temp.as_mut_ptr(),
            c"{\n\"classname\"   \"misc_bsp\"\n\"bspmodel\"    \"%s\"\n\"origin\"      \"%f %f %f\"\n\"angles\"      \"0 %f 0\"\n\"filter\"      \"%s\"\n\"teamfilter\"  \"%s\"\n\"spacing\"\t \"%d\"\n\"flatten\"\t \"%d\"\n}\n".as_ptr(),
            self.mBsp.as_ptr(),
            *self.GetOrigin().add(0), *self.GetOrigin().add(1), *self.GetOrigin().add(2),
            AngleNormalize360(yaw),
            self.mFilter.as_ptr(),
            self.mTeamFilter.as_ptr(),
            self.GetSpacingRadius() as c_int,
            self.GetFlattenRadius() as c_int,
        );

        if IsServer != 0 {
        // only allow for true spawning on the server
            savePtr = (*addr_of_mut!(sv)).entityParsePoint;
            (*addr_of_mut!(sv)).entityParsePoint = temp.as_mut_ptr();
            VM_Call(*addr_of!(gvm), GAME_SPAWN_RMG_ENTITY as c_int);
            (*addr_of_mut!(sv)).entityParsePoint = savePtr;
        }

        self.DrawAutomapSymbol();

        Com_DPrintf(c"RMG:  Building '%s' spawned at (%f %f %f)\n".as_ptr(),
            self.mBsp.as_ptr(),
            *self.GetOrigin().add(0), *self.GetOrigin().add(1), *self.GetOrigin().add(2));
        // now restore the instances un-mirrored origin
        // NOTE: all this origin flipping, setting the side etc... should be done when mMirror is set
        // because right after this function is called, mMirror is set to 0 but all the instance data is STILL MIRRORED -- not good
        VectorCopy(notmirrored.as_ptr(), self.GetOrigin());

    } // #endif  // PRE_RELEASE_DEMO

    true
}

} // impl CRMBSPInstance
