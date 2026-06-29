#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

// #pragma once
// #if !defined(RM_INSTANCE_GROUP_H_INC)
// #define RM_INSTANCE_GROUP_H_INC

// #ifdef DEBUG_LINKING
// 	#pragma message("...including RM_Instance_Group.h")
// #endif

use crate::codemp::RMG::GenericParser2_h::*;
use crate::codemp::RMG::RM_InstanceFile_h::*;
use crate::codemp::RMG::RM_Area_h::*;
use crate::codemp::RMG::RM_Objective_h::*;
use crate::codemp::qcommon::cm_randomterrain_h::*;
use crate::codemp::RMG::RM_Instance_h::*;

use core::ffi::{c_char, c_int, c_float};

// class CRMGroupInstance : public CRMInstance
#[repr(C)]
pub struct CRMGroupInstance {
    // protected:

    pub mInstances: rmInstanceList_t,
    pub mConfineRadius: c_float,
    pub mPaddingSize: c_float,
}

impl CRMGroupInstance {
    // public:

    // CRMGroupInstance( CGPGroup* instGroup, CRMInstanceFile& instFile);
    pub unsafe fn new(instGroup: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self {
        unimplemented!()
    }

    // ~CRMGroupInstance();

    // virtual bool		PreSpawn			( CRandomTerrain* terrain, qboolean IsServer );
    pub unsafe fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unimplemented!()
    }

    // virtual bool		Spawn				( CRandomTerrain* terrain, qboolean IsServer );
    pub unsafe fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unimplemented!()
    }

    // virtual void		Preview				( const vec3_t from );
    pub unsafe fn Preview(&self, from: vec3_t) {
        unimplemented!()
    }

    // virtual void		SetFilter			( const char *filter );
    pub unsafe fn SetFilter(&mut self, filter: *const c_char) {
        unimplemented!()
    }

    // virtual void		SetTeamFilter		( const char *teamFilter );
    pub unsafe fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        unimplemented!()
    }

    // virtual void		SetArea				( CRMAreaManager* amanager, CRMArea* area );
    pub unsafe fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        unimplemented!()
    }

    // virtual int			GetPreviewColor		( )		{ return (255<<24)+(255<<8); }
    pub fn GetPreviewColor(&self) -> c_int {
        (255_i32).wrapping_shl(24).wrapping_add((255_i32).wrapping_shl(8))
    }

    // virtual float		GetSpacingRadius	( )		{ return 0; }
    pub fn GetSpacingRadius(&self) -> c_float { 0.0 }

    // virtual float		GetFlattenRadius	( )		{ return 0; }
    pub fn GetFlattenRadius(&self) -> c_float { 0.0 }

    // virtual void  		SetMirror(int mirror);
    pub unsafe fn SetMirror(&mut self, mirror: c_int) {
        unimplemented!()
    }

    // protected:

    // void	RemoveInstances	 ( );
    unsafe fn RemoveInstances(&mut self) {
        unimplemented!()
    }
}

// #endif
