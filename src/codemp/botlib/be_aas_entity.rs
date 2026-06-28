/*****************************************************************************
 * name:		be_aas_entity.c
 *
 * desc:		AAS entities
 *
 * $Archive: /MissionPack/code/botlib/be_aas_entity.c $
 * $Author: Zaphod $
 * $Revision: 11 $
 * $Modtime: 11/22/00 8:50a $
 * $Date: 11/22/00 8:55a $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_void};
use std::ptr;

// Type aliases and stubs for external dependencies
pub type vec3_t = [f32; 3];

// External types (stubs for structural coherence)
#[repr(C)]
pub struct bot_entitystate_t {
    pub type_: c_int,
    pub flags: c_int,
    pub old_origin: vec3_t,
    pub solid: c_int,
    pub groundent: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub frame: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub angles: vec3_t,
    pub origin: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
}

#[repr(C)]
pub struct aas_entityinfo_t {
    pub update_time: i32,
    pub type_: c_int,
    pub flags: c_int,
    pub ltime: i32,
    pub lastvisorigin: vec3_t,
    pub old_origin: vec3_t,
    pub solid: c_int,
    pub groundent: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub frame: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub number: c_int,
    pub valid: i32,
    pub origin: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub angles: vec3_t,
}

#[repr(C)]
pub struct aas_link_t {
    pub entnum: c_int,
}

#[repr(C)]
pub struct aas_entity_t {
    pub i: aas_entityinfo_t,
    pub areas: *mut aas_link_t,
    pub leaves: *mut c_void,
}

#[repr(C)]
pub struct aas_world_t {
    pub loaded: i32,
    pub initialized: i32,
    pub maxentities: c_int,
    pub numframes: c_int,
    pub entities: *mut aas_entity_t,
}

#[repr(C)]
pub struct bsp_entdata_t {
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub absmins: vec3_t,
    pub absmaxs: vec3_t,
    pub solid: c_int,
    pub modelnum: c_int,
}

// External globals
extern "C" {
    pub static mut aasworld: aas_world_t;
}

// External functions and macros from headers
extern "C" {
    pub fn AAS_Time() -> i32;
    pub fn AAS_UnlinkFromAreas(areas: *mut aas_link_t);
    pub fn AAS_UnlinkFromBSPLeaves(leaves: *mut c_void);
    pub fn AAS_BSPModelMinsMaxsOrigin(modelindex: c_int, angles: *const f32, mins: *mut f32, maxs: *mut f32, origin: *mut c_void);
    pub fn AAS_LinkEntityClientBBox(absmins: *const f32, absmaxs: *const f32, entnum: c_int, presence: c_int) -> *mut aas_link_t;
    pub fn AAS_BSPLinkEntity(absmins: *const f32, absmaxs: *const f32, entnum: c_int, modelnum: c_int) -> *mut c_void;
    pub fn AAS_BestReachableLinkArea(areas: *mut aas_link_t) -> c_int;

    // Vector operations
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn VectorCompare(v1: *const f32, v2: *const f32) -> c_int;
    pub fn VectorAdd(v1: *const f32, v2: *const f32, out: *mut f32);
    pub fn VectorSubtract(v1: *const f32, v2: *const f32, out: *mut f32);
    pub fn VectorLength(v: *const f32) -> f32;
    pub fn VectorClear(v: *mut f32);

    // Memory operations
    pub fn Com_Memset(ptr: *mut c_void, val: c_int, size: usize);
    pub fn Com_Memcpy(dst: *mut c_void, src: *const c_void, size: usize);

    // Logging
    pub static mut botimport: botimport_t;
}

#[repr(C)]
pub struct botimport_t {
    pub Print: extern "C" fn(level: c_int, fmt: *const u8, ...) -> c_int,
}

// Constants
const CONTENTS_PLAYERCLIP: c_int = 16;
const MASK_SOLID: c_int = CONTENTS_PLAYERCLIP;

const BLERR_NOAASFILE: c_int = 1;
const BLERR_NOERROR: c_int = 0;

const PRT_MESSAGE: c_int = 1;
const PRT_FATAL: c_int = 2;

const SOLID_BSP: c_int = 3;
const SOLID_BBOX: c_int = 1;

const ENTITYNUM_WORLD: c_int = 0;
const PRESENCE_NORMAL: c_int = 1;

const QTrue: i32 = 1;
const QFalse: i32 = 0;

// FIXME: these might change
#[allow(non_upper_case_globals)]
pub mod ET {
    use core::ffi::c_int;
    pub const ET_GENERAL: c_int = 0;
    pub const ET_PLAYER: c_int = 1;
    pub const ET_ITEM: c_int = 2;
    pub const ET_MISSILE: c_int = 3;
    pub const ET_MOVER: c_int = 4;
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_UpdateEntity(entnum: c_int, state: *mut bot_entitystate_t) -> c_int {
    let mut relink: c_int;
    let ent: *mut aas_entity_t;
    let mut absmins: vec3_t;
    let mut absmaxs: vec3_t;

    unsafe {
        if aasworld.loaded == 0 {
            (botimport.Print)(PRT_MESSAGE, b"AAS_UpdateEntity: not loaded\n".as_ptr() as *const u8);
            return BLERR_NOAASFILE;
        } //end if

        ent = &mut *aasworld.entities.add(entnum as usize);

        if state.is_null() {
            //unlink the entity
            AAS_UnlinkFromAreas((*ent).areas);
            //unlink the entity from the BSP leaves
            AAS_UnlinkFromBSPLeaves((*ent).leaves);
            //
            (*ent).areas = ptr::null_mut();
            //
            (*ent).leaves = ptr::null_mut();
            return BLERR_NOERROR;
        }

        (*ent).i.update_time = AAS_Time() - (*ent).i.ltime;
        (*ent).i.type_ = (*state).type_;
        (*ent).i.flags = (*state).flags;
        (*ent).i.ltime = AAS_Time();
        VectorCopy((*ent).i.origin.as_ptr() as *const f32, (*ent).i.lastvisorigin.as_mut_ptr());
        VectorCopy((*state).old_origin.as_ptr() as *const f32, (*ent).i.old_origin.as_mut_ptr());
        (*ent).i.solid = (*state).solid;
        (*ent).i.groundent = (*state).groundent;
        (*ent).i.modelindex = (*state).modelindex;
        (*ent).i.modelindex2 = (*state).modelindex2;
        (*ent).i.frame = (*state).frame;
        (*ent).i.event = (*state).event;
        (*ent).i.eventParm = (*state).eventParm;
        (*ent).i.powerups = (*state).powerups;
        (*ent).i.weapon = (*state).weapon;
        (*ent).i.legsAnim = (*state).legsAnim;
        (*ent).i.torsoAnim = (*state).torsoAnim;
        //number of the entity
        (*ent).i.number = entnum;
        //updated so set valid flag
        (*ent).i.valid = QTrue;
        //link everything the first frame
        if aasworld.numframes == 1 {
            relink = QTrue;
        } else {
            relink = QFalse;
        }
        //
        if (*ent).i.solid == SOLID_BSP {
            //if the angles of the model changed
            if VectorCompare((*state).angles.as_ptr() as *const f32, (*ent).i.angles.as_ptr() as *const f32) == 0 {
                VectorCopy((*state).angles.as_ptr() as *const f32, (*ent).i.angles.as_mut_ptr());
                relink = QTrue;
            } //end if
            //get the mins and maxs of the model
            //FIXME: rotate mins and maxs
            AAS_BSPModelMinsMaxsOrigin(
                (*ent).i.modelindex,
                (*ent).i.angles.as_ptr() as *const f32,
                (*ent).i.mins.as_mut_ptr(),
                (*ent).i.maxs.as_mut_ptr(),
                ptr::null_mut(),
            );
        } //end if
        else if (*ent).i.solid == SOLID_BBOX {
            //if the bounding box size changed
            if VectorCompare((*state).mins.as_ptr() as *const f32, (*ent).i.mins.as_ptr() as *const f32) == 0
                || VectorCompare((*state).maxs.as_ptr() as *const f32, (*ent).i.maxs.as_ptr() as *const f32) == 0
            {
                VectorCopy((*state).mins.as_ptr() as *const f32, (*ent).i.mins.as_mut_ptr());
                VectorCopy((*state).maxs.as_ptr() as *const f32, (*ent).i.maxs.as_mut_ptr());
                relink = QTrue;
            } //end if
            VectorCopy((*state).angles.as_ptr() as *const f32, (*ent).i.angles.as_mut_ptr());
        } //end if
        //if the origin changed
        if VectorCompare((*state).origin.as_ptr() as *const f32, (*ent).i.origin.as_ptr() as *const f32) == 0 {
            VectorCopy((*state).origin.as_ptr() as *const f32, (*ent).i.origin.as_mut_ptr());
            relink = QTrue;
        } //end if
        //if the entity should be relinked
        if relink != 0 {
            //don't link the world model
            if entnum != ENTITYNUM_WORLD {
                //absolute mins and maxs
                VectorAdd((*ent).i.mins.as_ptr() as *const f32, (*ent).i.origin.as_ptr() as *const f32, absmins.as_mut_ptr());
                VectorAdd((*ent).i.maxs.as_ptr() as *const f32, (*ent).i.origin.as_ptr() as *const f32, absmaxs.as_mut_ptr());
                //unlink the entity
                AAS_UnlinkFromAreas((*ent).areas);
                //relink the entity to the AAS areas (use the larges bbox)
                (*ent).areas = AAS_LinkEntityClientBBox(absmins.as_ptr() as *const f32, absmaxs.as_ptr() as *const f32, entnum, PRESENCE_NORMAL);
                //unlink the entity from the BSP leaves
                AAS_UnlinkFromBSPLeaves((*ent).leaves);
                //link the entity to the world BSP tree
                (*ent).leaves = AAS_BSPLinkEntity(absmins.as_ptr() as *const f32, absmaxs.as_ptr() as *const f32, entnum, 0);
            } //end if
        } //end if
        BLERR_NOERROR
    }
} //end of the function AAS_UpdateEntity
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityInfo(entnum: c_int, info: *mut aas_entityinfo_t) {
    unsafe {
        if aasworld.initialized == 0 {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityInfo: aasworld not initialized\n".as_ptr() as *const u8);
            Com_Memset(info as *mut c_void, 0, std::mem::size_of::<aas_entityinfo_t>());
            return;
        } //end if

        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityInfo: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            Com_Memset(info as *mut c_void, 0, std::mem::size_of::<aas_entityinfo_t>());
            return;
        } //end if

        Com_Memcpy(
            info as *mut c_void,
            &(*aasworld.entities.add(entnum as usize)).i as *const aas_entityinfo_t as *const c_void,
            std::mem::size_of::<aas_entityinfo_t>(),
        );
    }
} //end of the function AAS_EntityInfo
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityOrigin(entnum: c_int, origin: *mut f32) {
    unsafe {
        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityOrigin: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            VectorClear(origin);
            return;
        } //end if

        VectorCopy(
            (*aasworld.entities.add(entnum as usize)).i.origin.as_ptr() as *const f32,
            origin,
        );
    }
} //end of the function AAS_EntityOrigin
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityModelindex(entnum: c_int) -> c_int {
    unsafe {
        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityModelindex: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            return 0;
        } //end if
        (*aasworld.entities.add(entnum as usize)).i.modelindex
    }
} //end of the function AAS_EntityModelindex
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityType(entnum: c_int) -> c_int {
    unsafe {
        if aasworld.initialized == 0 {
            return 0;
        }

        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityType: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            return 0;
        } //end if
        (*aasworld.entities.add(entnum as usize)).i.type_
    }
} //end of the AAS_EntityType
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityModelNum(entnum: c_int) -> c_int {
    unsafe {
        if aasworld.initialized == 0 {
            return 0;
        }

        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntityModelNum: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            return 0;
        } //end if
        (*aasworld.entities.add(entnum as usize)).i.modelindex
    }
} //end of the function AAS_EntityModelNum
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_OriginOfMoverWithModelNum(modelnum: c_int, origin: *mut f32) -> c_int {
    let mut i: c_int;
    let ent: *mut aas_entity_t;

    unsafe {
        i = 0;
        while i < aasworld.maxentities {
            ent = &mut *aasworld.entities.add(i as usize);
            if (*ent).i.type_ == ET::ET_MOVER {
                if (*ent).i.modelindex == modelnum {
                    VectorCopy((*ent).i.origin.as_ptr() as *const f32, origin);
                    return QTrue;
                } //end if
            } //end if
            i += 1;
        } //end for
        QFalse
    }
} //end of the function AAS_OriginOfMoverWithModelNum
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntitySize(entnum: c_int, mins: *mut f32, maxs: *mut f32) {
    let ent: *mut aas_entity_t;

    unsafe {
        if aasworld.initialized == 0 {
            return;
        }

        if entnum < 0 || entnum >= aasworld.maxentities {
            (botimport.Print)(PRT_FATAL, b"AAS_EntitySize: entnum %d out of range\n".as_ptr() as *const u8, entnum);
            return;
        } //end if

        ent = &mut *aasworld.entities.add(entnum as usize);
        VectorCopy((*ent).i.mins.as_ptr() as *const f32, mins);
        VectorCopy((*ent).i.maxs.as_ptr() as *const f32, maxs);
    }
} //end of the function AAS_EntitySize
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_EntityBSPData(entnum: c_int, entdata: *mut bsp_entdata_t) {
    let ent: *mut aas_entity_t;

    unsafe {
        ent = &mut *aasworld.entities.add(entnum as usize);
        VectorCopy((*ent).i.origin.as_ptr() as *const f32, (*entdata).origin.as_mut_ptr());
        VectorCopy((*ent).i.angles.as_ptr() as *const f32, (*entdata).angles.as_mut_ptr());
        VectorAdd(
            (*ent).i.origin.as_ptr() as *const f32,
            (*ent).i.mins.as_ptr() as *const f32,
            (*entdata).absmins.as_mut_ptr(),
        );
        VectorAdd(
            (*ent).i.origin.as_ptr() as *const f32,
            (*ent).i.maxs.as_ptr() as *const f32,
            (*entdata).absmaxs.as_mut_ptr(),
        );
        (*entdata).solid = (*ent).i.solid;
        (*entdata).modelnum = (*ent).i.modelindex - 1;
    }
} //end of the function AAS_EntityBSPData
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_ResetEntityLinks() {
    let mut i: c_int;
    unsafe {
        i = 0;
        while i < aasworld.maxentities {
            (*aasworld.entities.add(i as usize)).areas = ptr::null_mut();
            (*aasworld.entities.add(i as usize)).leaves = ptr::null_mut();
            i += 1;
        } //end for
    }
} //end of the function AAS_ResetEntityLinks
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_InvalidateEntities() {
    let mut i: c_int;
    unsafe {
        i = 0;
        while i < aasworld.maxentities {
            (*aasworld.entities.add(i as usize)).i.valid = QFalse;
            (*aasworld.entities.add(i as usize)).i.number = i;
            i += 1;
        } //end for
    }
} //end of the function AAS_InvalidateEntities
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_UnlinkInvalidEntities() {
    let mut i: c_int;
    let ent: *mut aas_entity_t;

    unsafe {
        i = 0;
        while i < aasworld.maxentities {
            ent = &mut *aasworld.entities.add(i as usize);
            if (*ent).i.valid == 0 {
                AAS_UnlinkFromAreas((*ent).areas);
                (*ent).areas = ptr::null_mut();
                AAS_UnlinkFromBSPLeaves((*ent).leaves);
                (*ent).leaves = ptr::null_mut();
            } //end for
            i += 1;
        } //end for
    }
} //end of the function AAS_UnlinkInvalidEntities
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_NearestEntity(origin: *const f32, modelindex: c_int) -> c_int {
    let mut i: c_int;
    let mut bestentnum: c_int;
    let mut dist: f32;
    let mut bestdist: f32;
    let ent: *mut aas_entity_t;
    let mut dir: vec3_t;

    unsafe {
        bestentnum = 0;
        bestdist = 99999.0;
        i = 0;
        while i < aasworld.maxentities {
            ent = &mut *aasworld.entities.add(i as usize);
            if (*ent).i.modelindex != modelindex {
                i += 1;
                continue;
            }
            VectorSubtract((*ent).i.origin.as_ptr() as *const f32, origin, dir.as_mut_ptr());
            if (dir[0] as i32).abs() < 40 {
                if (dir[1] as i32).abs() < 40 {
                    dist = VectorLength(dir.as_ptr() as *const f32);
                    if dist < bestdist {
                        bestdist = dist;
                        bestentnum = i;
                    } //end if
                } //end if
            } //end if
            i += 1;
        } //end for
        bestentnum
    }
} //end of the function AAS_NearestEntity
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_BestReachableEntityArea(entnum: c_int) -> c_int {
    let ent: *mut aas_entity_t;

    unsafe {
        ent = &mut *aasworld.entities.add(entnum as usize);
        AAS_BestReachableLinkArea((*ent).areas)
    }
} //end of the function AAS_BestReachableEntityArea
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub extern "C" fn AAS_NextEntity(mut entnum: c_int) -> c_int {
    unsafe {
        if aasworld.loaded == 0 {
            return 0;
        }

        if entnum < 0 {
            entnum = -1;
        }
        entnum += 1;
        while entnum < aasworld.maxentities {
            if (*aasworld.entities.add(entnum as usize)).i.valid != 0 {
                return entnum;
            }
            entnum += 1;
        } //end while
        0
    }
} //end of the function AAS_NextEntity
