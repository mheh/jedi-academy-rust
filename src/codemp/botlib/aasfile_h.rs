#![allow(non_snake_case)]

use core::ffi::c_int;

//NOTE:	int =	default signed
//				default long

// Type alias for vec3_t (3-element float array)
pub type vec3_t = [f32; 3];

pub const AASID: i32 = (('S' as i32) << 24) + (('A' as i32) << 16) + (('A' as i32) << 8) + ('E' as i32);
pub const AASVERSION_OLD: c_int = 4;
pub const AASVERSION: c_int = 5;

//presence types
pub const PRESENCE_NONE: c_int = 1;
pub const PRESENCE_NORMAL: c_int = 2;
pub const PRESENCE_CROUCH: c_int = 4;

//travel types
pub const MAX_TRAVELTYPES: c_int = 32;
pub const TRAVEL_INVALID: c_int = 1;		//temporary not possible
pub const TRAVEL_WALK: c_int = 2;		//walking
pub const TRAVEL_CROUCH: c_int = 3;		//crouching
pub const TRAVEL_BARRIERJUMP: c_int = 4;		//jumping onto a barrier
pub const TRAVEL_JUMP: c_int = 5;		//jumping
pub const TRAVEL_LADDER: c_int = 6;		//climbing a ladder
pub const TRAVEL_WALKOFFLEDGE: c_int = 7;		//walking of a ledge
pub const TRAVEL_SWIM: c_int = 8;		//swimming
pub const TRAVEL_WATERJUMP: c_int = 9;		//jump out of the water
pub const TRAVEL_TELEPORT: c_int = 10;		//teleportation
pub const TRAVEL_ELEVATOR: c_int = 11;		//travel by elevator
pub const TRAVEL_ROCKETJUMP: c_int = 12;		//rocket jumping required for travel
pub const TRAVEL_BFGJUMP: c_int = 13;		//bfg jumping required for travel
pub const TRAVEL_GRAPPLEHOOK: c_int = 14;		//grappling hook required for travel
pub const TRAVEL_DOUBLEJUMP: c_int = 15;		//double jump
pub const TRAVEL_RAMPJUMP: c_int = 16;		//ramp jump
pub const TRAVEL_STRAFEJUMP: c_int = 17;		//strafe jump
pub const TRAVEL_JUMPPAD: c_int = 18;		//jump pad
pub const TRAVEL_FUNCBOB: c_int = 19;		//func bob

//additional travel flags
pub const TRAVELTYPE_MASK: c_int = 0xFFFFFF;
pub const TRAVELFLAG_NOTTEAM1: c_int = 1 << 24;
pub const TRAVELFLAG_NOTTEAM2: c_int = 2 << 24;

//face flags
pub const FACE_SOLID: c_int = 1;		//just solid at the other side
pub const FACE_LADDER: c_int = 2;		//ladder
pub const FACE_GROUND: c_int = 4;		//standing on ground when in this face
pub const FACE_GAP: c_int = 8;		//gap in the ground
pub const FACE_LIQUID: c_int = 16;		//face seperating two areas with liquid
pub const FACE_LIQUIDSURFACE: c_int = 32;		//face seperating liquid and air
pub const FACE_BRIDGE: c_int = 64;		//can walk over this face if bridge is closed

//area contents
pub const AREACONTENTS_WATER: c_int = 1;
pub const AREACONTENTS_LAVA: c_int = 2;
pub const AREACONTENTS_SLIME: c_int = 4;
pub const AREACONTENTS_CLUSTERPORTAL: c_int = 8;
pub const AREACONTENTS_TELEPORTAL: c_int = 16;
pub const AREACONTENTS_ROUTEPORTAL: c_int = 32;
pub const AREACONTENTS_TELEPORTER: c_int = 64;
pub const AREACONTENTS_JUMPPAD: c_int = 128;
pub const AREACONTENTS_DONOTENTER: c_int = 256;
pub const AREACONTENTS_VIEWPORTAL: c_int = 512;
pub const AREACONTENTS_MOVER: c_int = 1024;
pub const AREACONTENTS_NOTTEAM1: c_int = 2048;
pub const AREACONTENTS_NOTTEAM2: c_int = 4096;
//number of model of the mover inside this area
pub const AREACONTENTS_MODELNUMSHIFT: c_int = 24;
pub const AREACONTENTS_MAXMODELNUM: c_int = 0xFF;
pub const AREACONTENTS_MODELNUM: c_int = (0xFF << 24);

//area flags
pub const AREA_GROUNDED: c_int = 1;		//bot can stand on the ground
pub const AREA_LADDER: c_int = 2;		//area contains one or more ladder faces
pub const AREA_LIQUID: c_int = 4;		//area contains a liquid
pub const AREA_DISABLED: c_int = 8;		//area is disabled for routing when set
pub const AREA_BRIDGE: c_int = 16;		//area ontop of a bridge

//aas file header lumps
pub const AAS_LUMPS: c_int = 14;
pub const AASLUMP_BBOXES: c_int = 0;
pub const AASLUMP_VERTEXES: c_int = 1;
pub const AASLUMP_PLANES: c_int = 2;
pub const AASLUMP_EDGES: c_int = 3;
pub const AASLUMP_EDGEINDEX: c_int = 4;
pub const AASLUMP_FACES: c_int = 5;
pub const AASLUMP_FACEINDEX: c_int = 6;
pub const AASLUMP_AREAS: c_int = 7;
pub const AASLUMP_AREASETTINGS: c_int = 8;
pub const AASLUMP_REACHABILITY: c_int = 9;
pub const AASLUMP_NODES: c_int = 10;
pub const AASLUMP_PORTALS: c_int = 11;
pub const AASLUMP_PORTALINDEX: c_int = 12;
pub const AASLUMP_CLUSTERS: c_int = 13;

//========== bounding box =========

//bounding box
#[repr(C)]
pub struct aas_bbox_s {
	pub presencetype: c_int,
	pub flags: c_int,
	pub mins: vec3_t,
	pub maxs: vec3_t,
}

pub type aas_bbox_t = aas_bbox_s;

//============ settings ===========

//reachability to another area
#[repr(C)]
pub struct aas_reachability_s {
	pub areanum: c_int,						//number of the reachable area
	pub facenum: c_int,						//number of the face towards the other area
	pub edgenum: c_int,						//number of the edge towards the other area
	pub start: vec3_t,						//start point of inter area movement
	pub end: vec3_t,							//end point of inter area movement
	pub traveltype: c_int,					//type of travel required to get to the area
	pub traveltime: u16,//travel time of the inter area movement
}

pub type aas_reachability_t = aas_reachability_s;

//area settings
#[repr(C)]
pub struct aas_areasettings_s {
	//could also add all kind of statistic fields
	pub contents: c_int,						//contents of the area
	pub areaflags: c_int,						//several area flags
	pub presencetype: c_int,					//how a bot can be present in this area
	pub cluster: c_int,						//cluster the area belongs to, if negative it's a portal
	pub clusterareanum: c_int,				//number of the area in the cluster
	pub numreachableareas: c_int,			//number of reachable areas from this one
	pub firstreachablearea: c_int,			//first reachable area in the reachable area index
}

pub type aas_areasettings_t = aas_areasettings_s;

//cluster portal
#[repr(C)]
pub struct aas_portal_s {
	pub areanum: c_int,						//area that is the actual portal
	pub frontcluster: c_int,					//cluster at front of portal
	pub backcluster: c_int,					//cluster at back of portal
	pub clusterareanum: [c_int; 2],			//number of the area in the front and back cluster
}

pub type aas_portal_t = aas_portal_s;

//cluster portal index
pub type aas_portalindex_t = c_int;

//cluster
#[repr(C)]
pub struct aas_cluster_s {
	pub numareas: c_int,						//number of areas in the cluster
	pub numreachabilityareas: c_int,			//number of areas with reachabilities
	pub numportals: c_int,						//number of cluster portals
	pub firstportal: c_int,					//first cluster portal in the index
}

pub type aas_cluster_t = aas_cluster_s;

//============ 3d definition ============

pub type aas_vertex_t = vec3_t;

//just a plane in the third dimension
#[repr(C)]
pub struct aas_plane_s {
	pub normal: vec3_t,						//normal vector of the plane
	pub dist: f32,							//distance of the plane (normal vector * distance = point in plane)
	pub r#type: c_int,
}

pub type aas_plane_t = aas_plane_s;

//edge
#[repr(C)]
pub struct aas_edge_s {
	pub v: [c_int; 2],							//numbers of the vertexes of this edge
}

pub type aas_edge_t = aas_edge_s;

//edge index, negative if vertexes are reversed
pub type aas_edgeindex_t = c_int;

//a face bounds an area, often it will also seperate two areas
#[repr(C)]
pub struct aas_face_s {
	pub planenum: c_int,						//number of the plane this face is in
	pub faceflags: c_int,						//face flags (no use to create face settings for just this field)
	pub numedges: c_int,						//number of edges in the boundary of the face
	pub firstedge: c_int,						//first edge in the edge index
	pub frontarea: c_int,						//area at the front of this face
	pub backarea: c_int,						//area at the back of this face
}

pub type aas_face_t = aas_face_s;

//face index, stores a negative index if backside of face
pub type aas_faceindex_t = c_int;

//area with a boundary of faces
#[repr(C)]
pub struct aas_area_s {
	pub areanum: c_int,						//number of this area
	//3d definition
	pub numfaces: c_int,						//number of faces used for the boundary of the area
	pub firstface: c_int,						//first face in the face index used for the boundary of the area
	pub mins: vec3_t,						//mins of the area
	pub maxs: vec3_t,						//maxs of the area
	pub center: vec3_t,						//'center' of the area
}

pub type aas_area_t = aas_area_s;

//nodes of the bsp tree
#[repr(C)]
pub struct aas_node_s {
	pub planenum: c_int,
	pub children: [c_int; 2],					//child nodes of this node, or areas as leaves when negative
										//when a child is zero it's a solid leaf
}

pub type aas_node_t = aas_node_s;

//=========== aas file ===============

//header lump
#[repr(C)]
pub struct aas_lump_t {
	pub fileofs: c_int,
	pub filelen: c_int,
}

//aas file header
#[repr(C)]
pub struct aas_header_s {
	pub ident: c_int,
	pub version: c_int,
	pub bspchecksum: c_int,
	//data entries
	pub lumps: [aas_lump_t; 14],
}

pub type aas_header_t = aas_header_s;


//====== additional information ======
/*

-	when a node child is a solid leaf the node child number is zero
-	two adjacent areas (sharing a plane at opposite sides) share a face
	this face is a portal between the areas
-	when an area uses a face from the faceindex with a positive index
	then the face plane normal points into the area
-	the face edges are stored counter clockwise using the edgeindex
-	two adjacent convex areas (sharing a face) only share One face
	this is a simple result of the areas being convex
-	the areas can't have a mixture of ground and gap faces
	other mixtures of faces in one area are allowed
-	areas with the AREACONTENTS_CLUSTERPORTAL in the settings have
	the cluster number set to the negative portal number
-	edge zero is a dummy
-	face zero is a dummy
-	area zero is a dummy
-	node zero is a dummy
*/
