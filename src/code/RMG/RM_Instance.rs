// oracle/code/RMG/RM_Instance.cpp -> src/code/RMG/RM_Instance.rs

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use core::ffi::c_char;

use crate::code::server::exe_headers_h::*;
use crate::code::RMG::RM_Headers_h::*;
use crate::code::qcommon::cm_terrainmap_h::*;
// CRMInstance is the paired class (defined in RM_Instance.h, which RM_Instance.cpp implements).
// Import it — do NOT redefine locally.
use crate::code::RMG::RM_Instance_h::*;

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
    pub unsafe fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        let mut self_: Self = core::mem::zeroed();
        self_.mObjective      = core::ptr::null_mut();
        self_.mSpacingRadius  = 0;
        self_.mFlattenRadius  = 0;
        // mFilter[0] = mTeamFilter[0] = 0  (C++ chained assignment, right-to-left)
        self_.mTeamFilter[0]  = 0;
        self_.mFilter[0]      = 0;
        self_.mArea           = core::ptr::null_mut();
        self_.mAutomapSymbol  = 0;
        self_.mEntityID       = 0;
        self_.mSide           = 0;
        self_.mMirror         = 0;
        self_.mFlattenHeight  = 66;
        self_.mSpacingLine    = 0;
        self_.mSurfaceSprites = true;
        self_.mLockOrigin     = false;
        self_
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
    pub unsafe fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        let mut origin: vec3_t = core::mem::zeroed();
        let mut area: CArea = core::mem::zeroed();

        VectorCopy(self.GetOrigin(), origin);

        if self.mMirror != 0 {
            origin[0] = (*TheRandomMissionManager).GetLandScape().GetBounds()[0][0] + (*TheRandomMissionManager).GetLandScape().GetBounds()[1][0] - origin[0];
            origin[1] = (*TheRandomMissionManager).GetLandScape().GetBounds()[0][1] + (*TheRandomMissionManager).GetLandScape().GetBounds()[1][1] - origin[1];
        }

        let terxelSize = (*terrain).GetLandScape().GetTerxelSize();
        let bounds     = (*terrain).GetLandScape().GetBounds();

        // Align the instance to the center of a terxel
        origin[0] = bounds[0][0] + ((origin[0] - bounds[0][0] + terxelSize[0] / 2.0) / terxelSize[0]) as i32 as f32 * terxelSize[0];
        origin[1] = bounds[0][1] + ((origin[1] - bounds[0][1] + terxelSize[1] / 2.0) / terxelSize[1]) as i32 as f32 * terxelSize[1];


        // This is BAD - By copying the mirrored origin back into the instance, you've now mirrored the original instance
        // so when anything from this point on looks at the instance they'll be looking at a mirrored version but will be expecting the original
        // so later in the spawn functions the instance will be re-mirrored, because it thinks the mInstances have not been changed
        //	VectorCopy(origin, GetOrigin());

        // Flatten the area below the instance
        if self.GetFlattenRadius() != 0 {
            area.Init( origin, self.GetFlattenRadius(), 0.0, AT_NONE, 0, 0 );
            (*terrain).GetLandScape().FlattenArea( &mut area, self.mFlattenHeight | if self.mSurfaceSprites { 0 } else { 0x80 }, false, true, true );
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
    pub unsafe fn PostSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        if !self.mObjective.is_null() {
            return (*self.mObjective).Link();
        }

        true
    }

    #[cfg(not(feature = "dedicated"))]
    pub unsafe fn DrawAutomapSymbol(&mut self) {
        // draw proper symbol on map for instance
        // Note: C++ switch has `default: case AUTOMAP_NONE:` sharing one body; translated here as `_` arm last.
        match self.GetAutomapSymbol() {
            AUTOMAP_BLD => {
                CM_TM_AddBuilding(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
                if self.HasObjective() {
                    CM_TM_AddObjective(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
                }
            }
            AUTOMAP_OBJ => {
                CM_TM_AddObjective(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
            }
            AUTOMAP_START => {
                CM_TM_AddStart(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
            }
            AUTOMAP_END => {
                CM_TM_AddEnd(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
            }
            AUTOMAP_ENEMY => {
                if self.HasObjective() {
                    CM_TM_AddObjective(self.GetOrigin()[0], self.GetOrigin()[1]);
                }
                if 1 == Cvar_VariableIntegerValue(b"rmg_automapshowall\0".as_ptr() as *const c_char) {
                    CM_TM_AddNPC(self.GetOrigin()[0], self.GetOrigin()[1], false);
                }
            }
            AUTOMAP_FRIEND => {
                if self.HasObjective() {
                    CM_TM_AddObjective(self.GetOrigin()[0], self.GetOrigin()[1]);
                }
                if 1 == Cvar_VariableIntegerValue(b"rmg_automapshowall\0".as_ptr() as *const c_char) {
                    CM_TM_AddNPC(self.GetOrigin()[0], self.GetOrigin()[1], true);
                }
            }
            AUTOMAP_WALL => {
                CM_TM_AddWallRect(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
            }
            // default: | case AUTOMAP_NONE:
            _ => {
                if self.HasObjective() {
                    CM_TM_AddObjective(self.GetOrigin()[0], self.GetOrigin()[1], self.GetSide());
                }
            }
        }
    }
    // #endif // !DEDICATED

    /************************************************************************************************
     * CRMInstance::Preview
     *	Renderings debug information about the instance
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn Preview(&mut self, from: vec3_t) {
    /*	CEntity				*tent;

    	// Add a cylindar for the whole settlement
    	tent = G_TempEntity( GetOrigin(), EV_DEBUG_CYLINDER );
    	VectorCopy( GetOrigin(), tent->s.origin2 );
    	tent->s.pos.trBase[2] += 40;
    	tent->s.origin2[2] += 50;
    	tent->s.time = 1050 + ((int)(GetSpacingRadius())<<16);
    	tent->s.time2 = GetPreviewColor ( );
    	G_AddTempEntity(tent);

    	// Origin line
    	tent = G_TempEntity( GetOrigin ( ), EV_DEBUG_LINE );
    	VectorCopy( GetOrigin(), tent->s.origin2 );
    	tent->s.origin2[2] += 400;
    	tent->s.time = 1050;
    	tent->s.weapon = 10;
    	tent->s.time2 = (255<<24) + (255<<16) + (255<<8) + 255;
    	G_AddTempEntity(tent);

    	if ( GetFlattenRadius ( ) )
    	{
    		// Add a cylindar for the whole settlement
    		tent = G_TempEntity( GetOrigin(), EV_DEBUG_CYLINDER );
    		VectorCopy( GetOrigin(), tent->s.origin2 );
    		tent->s.pos.trBase[2] += 40;
    		tent->s.origin2[2] += 50;
    		tent->s.time = 1050 + ((int)(GetFlattenRadius ( ))<<16);
    		tent->s.time2 = (255<<24) + (80<<16) +(80<<8) + 80;
    		G_AddTempEntity(tent);
    	}
    */
    }
}
