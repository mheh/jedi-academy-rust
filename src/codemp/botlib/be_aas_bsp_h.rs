/*****************************************************************************
 * name:		be_aas_bsp.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_bsp.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void};

// Stub type declarations for types defined elsewhere
#[repr(C)]
pub struct bsp_link_t {
    // Structure defined in another module
    _opaque: c_int,
}

#[repr(C)]
pub struct bsp_trace_t {
    // Structure defined in another module
    _opaque: c_int,
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

extern "C" {
    // loads the given BSP file
    pub fn AAS_LoadBSPFile() -> c_int;
    // dump the loaded BSP data
    pub fn AAS_DumpBSPData();
    // unlink the given entity from the bsp tree leaves
    pub fn AAS_UnlinkFromBSPLeaves(leaves: *mut bsp_link_t);
    // link the given entity to the bsp tree leaves of the given model
    pub fn AAS_BSPLinkEntity(
        absmins: vec3_t,
        absmaxs: vec3_t,
        entnum: c_int,
        modelnum: c_int,
    ) -> *mut bsp_link_t;

    // calculates collision with given entity
    pub fn AAS_EntityCollision(
        entnum: c_int,
        start: vec3_t,
        boxmins: vec3_t,
        boxmaxs: vec3_t,
        end: vec3_t,
        contentmask: c_int,
        trace: *mut bsp_trace_t,
    ) -> qboolean;
    // for debugging
    pub fn AAS_PrintFreeBSPLinks(str_: *mut c_char);

    // trace through the world
    pub fn AAS_Trace(
        start: vec3_t,
        mins: vec3_t,
        maxs: vec3_t,
        end: vec3_t,
        passent: c_int,
        contentmask: c_int,
    ) -> bsp_trace_t;
    // returns the contents at the given point
    pub fn AAS_PointContents(point: vec3_t) -> c_int;
    // returns true when p2 is in the PVS of p1
    pub fn AAS_inPVS(p1: vec3_t, p2: vec3_t) -> qboolean;
    // returns true when p2 is in the PHS of p1
    pub fn AAS_inPHS(p1: vec3_t, p2: vec3_t) -> qboolean;
    // returns true if the given areas are connected
    pub fn AAS_AreasConnected(area1: c_int, area2: c_int) -> qboolean;
    // creates a list with entities totally or partly within the given box
    pub fn AAS_BoxEntities(
        absmins: vec3_t,
        absmaxs: vec3_t,
        list: *mut c_int,
        maxcount: c_int,
    ) -> c_int;
    // gets the mins, maxs and origin of a BSP model
    pub fn AAS_BSPModelMinsMaxsOrigin(
        modelnum: c_int,
        angles: vec3_t,
        mins: vec3_t,
        maxs: vec3_t,
        origin: vec3_t,
    );
    // handle to the next bsp entity
    pub fn AAS_NextBSPEntity(ent: c_int) -> c_int;
    // return the value of the BSP epair key
    pub fn AAS_ValueForBSPEpairKey(
        ent: c_int,
        key: *mut c_char,
        value: *mut c_char,
        size: c_int,
    ) -> c_int;
    // get a vector for the BSP epair key
    pub fn AAS_VectorForBSPEpairKey(ent: c_int, key: *mut c_char, v: vec3_t) -> c_int;
    // get a float for the BSP epair key
    pub fn AAS_FloatForBSPEpairKey(ent: c_int, key: *mut c_char, value: *mut f32) -> c_int;
    // get an integer for the BSP epair key
    pub fn AAS_IntForBSPEpairKey(ent: c_int, key: *mut c_char, value: *mut c_int) -> c_int;
}

pub const MAX_EPAIRKEY: c_int = 128;
