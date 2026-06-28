/*****************************************************************************
 * name:		be_aas_def.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_def.h $
 * $Author: osman $
 * $Revision: 1.4 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 2003/03/15 23:43:54 $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_ushort};

// Stub types for external definitions used in this header
// These are defined in separate AAS modules
pub type vec3_t = [f32; 3];
pub type aas_entityinfo_t = c_int;
pub type aas_bbox_t = c_int;
pub type aas_vertex_t = c_int;
pub type aas_plane_t = c_int;
pub type aas_edge_t = c_int;
pub type aas_edgeindex_t = c_int;
pub type aas_face_t = c_int;
pub type aas_faceindex_t = c_int;
pub type aas_area_t = c_int;
pub type aas_areasettings_t = c_int;
pub type aas_reachability_t = c_int;
pub type aas_node_t = c_int;
pub type aas_portal_t = c_int;
pub type aas_portalindex_t = c_int;
pub type aas_cluster_t = c_int;
pub type byte = u8;
pub type qboolean = c_int;

// Stub constants for external definitions
pub const MAX_GENTITIES: c_int = 1024;
pub const MAX_QPATH: usize = 256;
pub const MAX_CONFIGSTRINGS: usize = 1024;
pub const MAX_TRAVELTYPES: usize = 32;

// debugging on
pub const AAS_DEBUG: bool = true;

// these are also in q_shared.h - argh (rjr)
#[cfg(feature = "xbox")]
pub const MAX_CLIENTS: c_int = 16;
#[cfg(not(feature = "xbox"))]
pub const MAX_CLIENTS: c_int = 32;

pub const MAX_RADAR_ENTITIES: c_int = MAX_GENTITIES;
// these are sent over the net as 8 bits
pub const MAX_MODELS: c_int = 512;
// so they cannot be blindly increased
pub const MAX_SOUNDS: c_int = 256;

// these are also in bg_public.h - argh (rjr)
pub const CS_SCORES: c_int = 32;
pub const CS_MODELS: c_int = CS_SCORES + MAX_CLIENTS;
pub const CS_SOUNDS: c_int = CS_MODELS + MAX_MODELS;

#[cfg(not(feature = "max_path_defined"))]
pub const MAX_PATH: usize = MAX_QPATH;

// Macro implementations - in C these were defined as:
// #define DF_AASENTNUMBER(x)		(x - aasworld.entities)
// #define DF_NUMBERAASENT(x)		(&aasworld.entities[x])
// #define DF_AASENTCLIENT(x)		(x - aasworld.entities - 1)
// #define DF_CLIENTAASENT(x)		(&aasworld.entities[x + 1])
// Note: These macros require access to the aasworld global and are implemented
// in the aasworld module as unsafe functions.

//string index (for model, sound and image index)
#[repr(C)]
pub struct aas_stringindex_s {
    pub numindexes: c_int,
    pub index: *mut *mut c_char,
}
pub type aas_stringindex_t = aas_stringindex_s;

//structure to link entities to areas and areas to entities
#[repr(C)]
pub struct aas_link_s {
    pub entnum: c_int,
    pub areanum: c_int,
    pub next_ent: *mut aas_link_s,
    pub prev_ent: *mut aas_link_s,
    pub next_area: *mut aas_link_s,
    pub prev_area: *mut aas_link_s,
}
pub type aas_link_t = aas_link_s;

//structure to link entities to leaves and leaves to entities
#[repr(C)]
pub struct bsp_link_s {
    pub entnum: c_int,
    pub leafnum: c_int,
    pub next_ent: *mut bsp_link_s,
    pub prev_ent: *mut bsp_link_s,
    pub next_leaf: *mut bsp_link_s,
    pub prev_leaf: *mut bsp_link_s,
}
pub type bsp_link_t = bsp_link_s;

#[repr(C)]
pub struct bsp_entdata_s {
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub absmins: vec3_t,
    pub absmaxs: vec3_t,
    pub solid: c_int,
    pub modelnum: c_int,
}
pub type bsp_entdata_t = bsp_entdata_s;

//entity
#[repr(C)]
pub struct aas_entity_s {
    //entity info
    pub i: aas_entityinfo_t,
    //links into the AAS areas
    pub areas: *mut aas_link_t,
    //links into the BSP leaves
    pub leaves: *mut bsp_link_t,
}
pub type aas_entity_t = aas_entity_s;

#[repr(C)]
pub struct aas_settings_s {
    pub phys_gravitydirection: vec3_t,
    pub phys_friction: f32,
    pub phys_stopspeed: f32,
    pub phys_gravity: f32,
    pub phys_waterfriction: f32,
    pub phys_watergravity: f32,
    pub phys_maxvelocity: f32,
    pub phys_maxwalkvelocity: f32,
    pub phys_maxcrouchvelocity: f32,
    pub phys_maxswimvelocity: f32,
    pub phys_walkaccelerate: f32,
    pub phys_airaccelerate: f32,
    pub phys_swimaccelerate: f32,
    pub phys_maxstep: f32,
    pub phys_maxsteepness: f32,
    pub phys_maxwaterjump: f32,
    pub phys_maxbarrier: f32,
    pub phys_jumpvel: f32,
    pub phys_falldelta5: f32,
    pub phys_falldelta10: f32,
    pub rs_waterjump: f32,
    pub rs_teleport: f32,
    pub rs_barrierjump: f32,
    pub rs_startcrouch: f32,
    pub rs_startgrapple: f32,
    pub rs_startwalkoffledge: f32,
    pub rs_startjump: f32,
    pub rs_rocketjump: f32,
    pub rs_bfgjump: f32,
    pub rs_jumppad: f32,
    pub rs_aircontrolledjumppad: f32,
    pub rs_funcbob: f32,
    pub rs_startelevator: f32,
    pub rs_falldamage5: f32,
    pub rs_falldamage10: f32,
    pub rs_maxfallheight: f32,
    pub rs_maxjumpfallheight: f32,
}
pub type aas_settings_t = aas_settings_s;

pub const CACHETYPE_PORTAL: c_int = 0;
pub const CACHETYPE_AREA: c_int = 1;

//routing cache
#[repr(C)]
pub struct aas_routingcache_s {
    pub r#type: byte,						//portal or area cache
    pub time: f32,						//last time accessed or updated
    pub size: c_int,						//size of the routing cache
    pub cluster: c_int,					//cluster the cache is for
    pub areanum: c_int,					//area the cache is created for
    pub origin: vec3_t,					//origin within the area
    pub starttraveltime: f32,				//travel time to start with
    pub travelflags: c_int,					//combinations of the travel flags
    pub prev: *mut aas_routingcache_s,
    pub next: *mut aas_routingcache_s,
    pub time_prev: *mut aas_routingcache_s,
    pub time_next: *mut aas_routingcache_s,
    pub reachabilities: *mut u8,			//reachabilities used for routing
    pub traveltimes: [c_ushort; 1],			//travel time for every area (variable sized)
}
pub type aas_routingcache_t = aas_routingcache_s;

//fields for the routing algorithm
#[repr(C)]
pub struct aas_routingupdate_s {
    pub cluster: c_int,
    pub areanum: c_int,					//area number of the update
    pub start: vec3_t,						//start point the area was entered
    pub tmptraveltime: c_ushort,			//temporary travel time
    pub areatraveltimes: *mut c_ushort,		//travel times within the area
    pub inlist: qboolean,					//true if the update is in the list
    pub next: *mut aas_routingupdate_s,
    pub prev: *mut aas_routingupdate_s,
}
pub type aas_routingupdate_t = aas_routingupdate_s;

//reversed reachability link
#[repr(C)]
pub struct aas_reversedlink_s {
    pub linknum: c_int,						//the aas_areareachability_t
    pub areanum: c_int,						//reachable from this area
    pub next: *mut aas_reversedlink_s,		//next link
}
pub type aas_reversedlink_t = aas_reversedlink_s;

//reversed area reachability
#[repr(C)]
pub struct aas_reversedreachability_s {
    pub numlinks: c_int,
    pub first: *mut aas_reversedlink_t,
}
pub type aas_reversedreachability_t = aas_reversedreachability_s;

//areas a reachability goes through
#[repr(C)]
pub struct aas_reachabilityareas_s {
    pub firstarea: c_int,
    pub numareas: c_int,
}
pub type aas_reachabilityareas_t = aas_reachabilityareas_s;

#[repr(C)]
pub struct aas_s {
    pub loaded: c_int,						//true when an AAS file is loaded
    pub initialized: c_int,					//true when AAS has been initialized
    pub savefile: c_int,					//set true when file should be saved
    pub bspchecksum: c_int,
    //current time
    pub time: f32,
    pub numframes: c_int,
    //name of the aas file
    pub filename: [c_char; MAX_PATH],
    pub mapname: [c_char; MAX_PATH],
    //bounding boxes
    pub numbboxes: c_int,
    pub bboxes: *mut aas_bbox_t,
    //vertexes
    pub numvertexes: c_int,
    pub vertexes: *mut aas_vertex_t,
    //planes
    pub numplanes: c_int,
    pub planes: *mut aas_plane_t,
    //edges
    pub numedges: c_int,
    pub edges: *mut aas_edge_t,
    //edge index
    pub edgeindexsize: c_int,
    pub edgeindex: *mut aas_edgeindex_t,
    //faces
    pub numfaces: c_int,
    pub faces: *mut aas_face_t,
    //face index
    pub faceindexsize: c_int,
    pub faceindex: *mut aas_faceindex_t,
    //convex areas
    pub numareas: c_int,
    pub areas: *mut aas_area_t,
    //convex area settings
    pub numareasettings: c_int,
    pub areasettings: *mut aas_areasettings_t,
    //reachablity list
    pub reachabilitysize: c_int,
    pub reachability: *mut aas_reachability_t,
    //nodes of the bsp tree
    pub numnodes: c_int,
    pub nodes: *mut aas_node_t,
    //cluster portals
    pub numportals: c_int,
    pub portals: *mut aas_portal_t,
    //cluster portal index
    pub portalindexsize: c_int,
    pub portalindex: *mut aas_portalindex_t,
    //clusters
    pub numclusters: c_int,
    pub clusters: *mut aas_cluster_t,
    //
    pub numreachabilityareas: c_int,
    pub reachabilitytime: f32,
    //enities linked in the areas
    pub linkheap: *mut aas_link_t,			//heap with link structures
    pub linkheapsize: c_int,					//size of the link heap
    pub freelinks: *mut aas_link_t,			//first free link
    pub arealinkedentities: *mut *mut aas_link_t,	//entities linked into areas
    //entities
    pub maxentities: c_int,
    pub maxclients: c_int,
    pub entities: *mut aas_entity_t,
    //string indexes
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    pub indexessetup: c_int,
    //index to retrieve travel flag for a travel type
    pub travelflagfortype: [c_int; MAX_TRAVELTYPES],
    //travel flags for each area based on contents
    pub areacontentstravelflags: *mut c_int,
    //routing update
    pub areaupdate: *mut aas_routingupdate_t,
    pub portalupdate: *mut aas_routingupdate_t,
    //number of routing updates during a frame (reset every frame)
    pub frameroutingupdates: c_int,
    //reversed reachability links
    pub reversedreachability: *mut aas_reversedreachability_t,
    //travel times within the areas
    pub areatraveltimes: *mut *mut *mut c_ushort,
    //array of size numclusters with cluster cache
    pub clusterareacache: *mut *mut *mut aas_routingcache_t,
    pub portalcache: *mut *mut aas_routingcache_t,
    //cache list sorted on time
    pub oldestcache: *mut aas_routingcache_t,	// start of cache list sorted on time
    pub newestcache: *mut aas_routingcache_t,	// end of cache list sorted on time
    //maximum travel time through portal areas
    pub portalmaxtraveltimes: *mut c_int,
    //areas the reachabilities go through
    pub reachabilityareaindex: *mut c_int,
    pub reachabilityareas: *mut aas_reachabilityareas_t,
}
pub type aas_t = aas_s;

pub const AASINTERN: bool = true;

// Note: The following are C #include directives from the original header:
// #include "be_aas_main.h"
// #include "be_aas_entity.h"
// #include "be_aas_sample.h"
// #include "be_aas_cluster.h"
// #include "be_aas_reach.h"
// #include "be_aas_route.h"
// #include "be_aas_routealt.h"
// #include "be_aas_debug.h"
// #include "be_aas_file.h"
// #include "be_aas_optimize.h"
// #include "be_aas_bsp.h"
// #include "be_aas_move.h"
// These are conditionally included only when BSPCINCLUDE is not defined.
// In the Rust port, these are imported as separate modules in the package structure.
