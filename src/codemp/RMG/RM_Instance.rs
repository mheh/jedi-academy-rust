//Anything above this #include will be ignored by the compiler
// (porting note: file-level comment preserved from oracle; original preprocessor line is not meaningful in Rust)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]

// #include "../qcommon/exe_headers.h"
use crate::codemp::qcommon::exe_headers_h::*;

// #include "RM_Headers.h"
use crate::codemp::RMG::RM_Headers_h::*;

// #include "../qcommon/cm_terrainmap.h"
use crate::codemp::qcommon::cm_terrainmap_h::*;

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
// porting note: C++ constructor translated as an &mut self init-method; allocation/placement is caller's responsibility
pub unsafe fn CRMInstance(&mut self, instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) {
    self.mObjective      = core::ptr::null_mut();
    self.mSpacingRadius  = 0;
    self.mFlattenRadius  = 0;
    // C: mFilter[0] = mTeamFilter[0] = 0; (chained assignment, right-to-left)
    self.mTeamFilter[0]  = 0;
    self.mFilter[0]      = 0;
    self.mArea           = core::ptr::null_mut();
    self.mAutomapSymbol  = 0;
    self.mEntityID       = 0;
    self.mSide           = 0;
    self.mMirror         = 0;
    self.mFlattenHeight  = 66;
    self.mSpacingLine    = 0;
    self.mSurfaceSprites = true;
    self.mLockOrigin     = false;
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
    let mut origin: vec3_t;
    let mut area: CArea;

    origin = core::mem::zeroed();
    area   = core::mem::zeroed();

    VectorCopy(self.GetOrigin(), origin.as_mut_ptr());

    if self.mMirror != 0 {
        origin[0] = (*(*TheRandomMissionManager).GetLandScape()).GetBounds()[0][0] + (*(*TheRandomMissionManager).GetLandScape()).GetBounds()[1][0] - origin[0];
        origin[1] = (*(*TheRandomMissionManager).GetLandScape()).GetBounds()[0][1] + (*(*TheRandomMissionManager).GetLandScape()).GetBounds()[1][1] - origin[1];
    }

    let terxelSize: &vec3_t  = &*(*(*terrain).GetLandScape()).GetTerxelSize();
    let bounds: &vec3pair_t  = &*(*(*terrain).GetLandScape()).GetBounds();

    // Align the instance to the center of a terxel
    origin[0] = bounds[0][0] + ((origin[0] - bounds[0][0] + terxelSize[0] / 2.0) / terxelSize[0]) as i32 as f32 * terxelSize[0];
    origin[1] = bounds[0][1] + ((origin[1] - bounds[0][1] + terxelSize[1] / 2.0) / terxelSize[1]) as i32 as f32 * terxelSize[1];


    // This is BAD - By copying the mirrored origin back into the instance, you've now mirrored the original instance
    // so when anything from this point on looks at the instance they'll be looking at a mirrored version but will be expecting the original
    // so later in the spawn functions the instance will be re-mirrored, because it thinks the mInstances have not been changed
//	VectorCopy(origin, GetOrigin());

    // Flatten the area below the instance
    if self.GetFlattenRadius() != 0.0 {
        area.Init( origin, self.GetFlattenRadius(), 0.0, AT_NONE, 0, 0 );
        (*(*terrain).GetLandScape()).FlattenArea( &area, self.mFlattenHeight | (if self.mSurfaceSprites { 0 } else { 0x80 }), false, true, true );
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

pub unsafe fn DrawAutomapSymbol(&mut self) {
    (*TheRandomMissionManager).AddAutomapSymbol( self.GetAutomapSymbol(), self.GetOrigin(), self.GetSide() );
/*
    // draw proper symbol on map for instance
    switch (GetAutomapSymbol())
    {
        default:
        case AUTOMAP_NONE:
            if (HasObjective())
                CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
        case AUTOMAP_BLD:
            CM_TM_AddBuilding(GetOrigin()[0], GetOrigin()[1], GetSide());
            if (HasObjective())
                CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
        case AUTOMAP_OBJ:
            CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
        case AUTOMAP_START:
            CM_TM_AddStart(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
        case AUTOMAP_END:
            CM_TM_AddEnd(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
        case AUTOMAP_ENEMY:
            if (HasObjective())
                CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1]);
            if (1 == Cvar_VariableIntegerValue("rmg_automapshowall"))
                CM_TM_AddNPC(GetOrigin()[0], GetOrigin()[1], false);
            break;
        case AUTOMAP_FRIEND:
            if (HasObjective())
                CM_TM_AddObjective(GetOrigin()[0], GetOrigin()[1]);
            if (1 == Cvar_VariableIntegerValue("rmg_automapshowall"))
                CM_TM_AddNPC(GetOrigin()[0], GetOrigin()[1], true);
            break;
        case AUTOMAP_WALL:
            CM_TM_AddWallRect(GetOrigin()[0], GetOrigin()[1], GetSide());
            break;
    }
*/
}

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
pub unsafe fn Preview(&mut self, from: *const vec3_t) {
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

} // impl CRMInstance
