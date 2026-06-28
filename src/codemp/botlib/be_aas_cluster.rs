// name:		be_aas_cluster.c
//
// desc:		area clustering
//
// $Archive: /MissionPack/code/botlib/be_aas_cluster.c $
// $Author: Ttimo $
// $Revision: 10 $
// $Modtime: 4/21/01 9:15a $
// $Date: 4/21/01 9:15a $

use core::ffi::{c_int, c_char, c_void};

// Stub declarations for external dependencies from included headers
extern "C" {
    pub static mut aasworld: aasworld_t;
    pub static botimport: botlib_import_t;

    fn AAS_Error(fmt: *const c_char, ...) -> ();
    fn Log_Write(fmt: *const c_char, ...) -> ();
    fn Com_Memset(dst: *mut c_void, c: c_int, size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void) -> ();
    fn GetClearedMemory(size: usize) -> *mut c_void;
    fn AAS_AreaReachability(areanum: c_int) -> c_int;
    fn AAS_AreaGrounded(areanum: c_int) -> c_int;
    fn AAS_ValueForBSPEpairKey(ent: *mut bsp_entity_t, key: *const c_char) -> *mut c_char;
    fn AAS_VectorForBSPEpairKey(ent: *mut bsp_entity_t, key: *const c_char, vec: *mut [f32; 3]) -> c_int;
    fn AAS_ParseBSPEntities() -> *mut bsp_entity_t;
    fn AAS_FreeBSPEntities(entities: *mut bsp_entity_t) -> ();
    fn AAS_TraceClientBBox(start: [f32; 3], end: [f32; 3], presencetype: c_int, passent: c_int) -> aas_trace_t;
    fn AAS_PointAreaNum(pos: [f32; 3]) -> c_int;
    fn AAS_PresenceTypeBoundingBox(presencetype: c_int, mins: *mut [f32; 3], maxs: *mut [f32; 3]) -> ();
    fn AAS_AASLinkEntity(mins: [f32; 3], maxs: [f32; 3], entnum: c_int) -> *mut aas_link_t;
    fn LibVarGetValue(var_name: *const c_char) -> f64;
}

// Type stubs for external structures
#[repr(C)]
pub struct aasworld_t {
    pub loaded: c_int,
    pub numareas: c_int,
    pub numportals: c_int,
    pub numclusters: c_int,
    pub portalindexsize: c_int,
    pub savefile: c_int,
    pub areasettings: *mut aas_areasettings_t,
    pub portals: *mut aas_portal_t,
    pub portalindex: *mut aas_portalindex_t,
    pub clusters: *mut aas_cluster_t,
    pub areas: *mut aas_area_t,
    pub faces: *mut aas_face_t,
    pub faceindex: *mut c_int,
    pub planes: *mut aas_plane_t,
    pub reachability: *mut aas_reachability_t,
    pub edges: *mut aas_edge_t,
    pub edgeindex: *mut c_int,
    pub vertexes: *mut [f32; 3],
}

#[repr(C)]
pub struct aas_areasettings_t {
    pub contents: c_int,
    pub areaflags: c_int,
    pub presencetype: c_int,
    pub cluster: c_int,
    pub clusterareanum: c_int,
    pub numreachableareas: c_int,
    pub firstreachablearea: c_int,
}

#[repr(C)]
pub struct aas_portal_t {
    pub areanum: c_int,
    pub frontcluster: c_int,
    pub backcluster: c_int,
    pub clusterareanum: [c_int; 2],
}

pub type aas_portalindex_t = c_int;

#[repr(C)]
pub struct aas_cluster_t {
    pub numareas: c_int,
    pub numreachabilityareas: c_int,
    pub firstportal: c_int,
    pub numportals: c_int,
}

#[repr(C)]
pub struct aas_area_t {
    pub firstface: c_int,
    pub numfaces: c_int,
}

#[repr(C)]
pub struct aas_face_t {
    pub planenum: c_int,
    pub faceflags: c_int,
    pub frontarea: c_int,
    pub backarea: c_int,
    pub numedges: c_int,
    pub firstedge: c_int,
}

#[repr(C)]
pub struct aas_plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}

#[repr(C)]
pub struct aas_reachability_t {
    pub areanum: c_int,
    pub traveltype: c_int,
}

#[repr(C)]
pub struct aas_edge_t {
    pub v: [c_int; 2],
}

#[repr(C)]
pub struct aas_trace_t {
    pub startsolid: c_int,
    pub endpos: [f32; 3],
}

#[repr(C)]
pub struct bsp_entity_t {
    pub next: *mut bsp_entity_t,
}

#[repr(C)]
pub struct aas_link_t {
    pub areanum: c_int,
    pub next_area: *mut aas_link_t,
}

#[repr(C)]
pub struct botlib_import_t {
    pub Print: extern "C" fn(c_int, *const c_char, ...) -> (),
}

const AAS_MAX_PORTALS: c_int = 65536;
const AAS_MAX_PORTALINDEXSIZE: c_int = 65536;
const AAS_MAX_CLUSTERS: c_int = 65536;
const MAX_PORTALAREAS: c_int = 1024;

const AREACONTENTS_CLUSTERPORTAL: c_int = 1;
const AREACONTENTS_ROUTEPORTAL: c_int = 2;
const AREACONTENTS_TELEPORTAL: c_int = 4;
const AREACONTENTS_VIEWPORTAL: c_int = 8;

const AREA_GROUNDED: c_int = 1;

const FACE_SOLID: c_int = 1;

const PRESENCE_CROUCH: c_int = 1;

const TRAVEL_TELEPORT: c_int = 1;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

const PRT_MESSAGE: c_int = 0;
const PRT_ERROR: c_int = 1;

// do not flood through area faces, only use reachabilities
pub static mut nofaceflood: c_int = qtrue;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_RemoveClusterAreas() {
    let mut i: c_int;

    i = 1;
    while i < aasworld.numareas {
        (*aasworld.areasettings.add(i as usize)).cluster = 0;
        i += 1;
    } //end for
} //end of the function AAS_RemoveClusterAreas

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ClearCluster(clusternum: c_int) {
    let mut i: c_int;

    i = 1;
    while i < aasworld.numareas {
        if (*aasworld.areasettings.add(i as usize)).cluster == clusternum {
            (*aasworld.areasettings.add(i as usize)).cluster = 0;
        } //end if
        i += 1;
    } //end for
} //end of the function AAS_ClearCluster

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_RemovePortalsClusterReference(clusternum: c_int) {
    let mut portalnum: c_int;

    portalnum = 1;
    while portalnum < aasworld.numportals {
        if (*aasworld.portals.add(portalnum as usize)).frontcluster == clusternum {
            (*aasworld.portals.add(portalnum as usize)).frontcluster = 0;
        } //end if
        if (*aasworld.portals.add(portalnum as usize)).backcluster == clusternum {
            (*aasworld.portals.add(portalnum as usize)).backcluster = 0;
        } //end if
        portalnum += 1;
    } //end for
} //end of the function AAS_RemovePortalsClusterReference

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_UpdatePortal(areanum: c_int, clusternum: c_int) -> c_int {
    let mut portalnum: c_int;
    let portal: *mut aas_portal_t;
    let cluster: *mut aas_cluster_t;

    //find the portal of the area
    portalnum = 1;
    while portalnum < aasworld.numportals {
        if (*aasworld.portals.add(portalnum as usize)).areanum == areanum {
            break;
        }
        portalnum += 1;
    } //end for
    //
    if portalnum == aasworld.numportals {
        let fmt = b"%s\0" as *const u8 as *const c_char;
        AAS_Error(fmt);
        return qtrue;
    } //end if
    //
    portal = aasworld.portals.add(portalnum as usize);
    //if the portal is already fully updated
    if (*portal).frontcluster == clusternum {
        return qtrue;
    }
    if (*portal).backcluster == clusternum {
        return qtrue;
    }
    //if the portal has no front cluster yet
    if (*portal).frontcluster == 0 {
        (*portal).frontcluster = clusternum;
    } //end if
    //if the portal has no back cluster yet
    else if (*portal).backcluster == 0 {
        (*portal).backcluster = clusternum;
    } //end else if
    else {
        //remove the cluster portal flag contents
        (*aasworld.areasettings.add(areanum as usize)).contents &= !AREACONTENTS_CLUSTERPORTAL;
        let fmt = b"portal area %d is seperating more than two clusters\r\n\0" as *const u8 as *const c_char;
        Log_Write(fmt, areanum);
        return qfalse;
    } //end else
    if aasworld.portalindexsize >= AAS_MAX_PORTALINDEXSIZE {
        let fmt = b"AAS_MAX_PORTALINDEXSIZE\0" as *const u8 as *const c_char;
        AAS_Error(fmt);
        return qtrue;
    } //end if
    //set the area cluster number to the negative portal number
    (*aasworld.areasettings.add(areanum as usize)).cluster = -portalnum;
    //add the portal to the cluster using the portal index
    cluster = aasworld.clusters.add(clusternum as usize);
    let idx = ((*cluster).firstportal + (*cluster).numportals) as usize;
    *aasworld.portalindex.add(idx) = portalnum;
    aasworld.portalindexsize += 1;
    (*cluster).numportals += 1;
    return qtrue;
} //end of the function AAS_UpdatePortal

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_FloodClusterAreas_r(areanum: c_int, clusternum: c_int) -> c_int {
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;
    let mut facenum: c_int;
    let mut i: c_int;

    //
    if areanum <= 0 || areanum >= aasworld.numareas {
        let fmt = b"AAS_FloodClusterAreas_r: areanum out of range\0" as *const u8 as *const c_char;
        AAS_Error(fmt);
        return qfalse;
    } //end if
    //if the area is already part of a cluster
    if (*aasworld.areasettings.add(areanum as usize)).cluster > 0 {
        if (*aasworld.areasettings.add(areanum as usize)).cluster == clusternum {
            return qtrue;
        }
        //
        //there's a reachability going from one cluster to another only in one direction
        //
        let fmt = b"cluster %d touched cluster %d at area %d\r\n\0" as *const u8 as *const c_char;
        AAS_Error(fmt, clusternum, (*aasworld.areasettings.add(areanum as usize)).cluster, areanum);
        return qfalse;
    } //end if
    //don't add the cluster portal areas to the clusters
    if (*aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
        return AAS_UpdatePortal(areanum, clusternum);
    } //end if
    //set the area cluster number
    (*aasworld.areasettings.add(areanum as usize)).cluster = clusternum;
    (*aasworld.areasettings.add(areanum as usize)).clusterareanum =
                (*aasworld.clusters.add(clusternum as usize)).numareas;
    //the cluster has an extra area
    (*aasworld.clusters.add(clusternum as usize)).numareas += 1;

    area = aasworld.areas.add(areanum as usize);
    //use area faces to flood into adjacent areas
    if nofaceflood == 0 {
        i = 0;
        while i < (*area).numfaces {
            facenum = (*aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            face = aasworld.faces.add(facenum as usize);
            if (*face).frontarea == areanum {
                if (*face).backarea != 0 {
                    if AAS_FloodClusterAreas_r((*face).backarea, clusternum) == 0 {
                        return qfalse;
                    }
                }
            } //end if
            else {
                if (*face).frontarea != 0 {
                    if AAS_FloodClusterAreas_r((*face).frontarea, clusternum) == 0 {
                        return qfalse;
                    }
                }
            } //end else
            i += 1;
        } //end for
    } //end if
    //use the reachabilities to flood into other areas
    i = 0;
    while i < (*aasworld.areasettings.add(areanum as usize)).numreachableareas {
        if (*aasworld.reachability.add(
                    ((*aasworld.areasettings.add(areanum as usize)).firstreachablearea + i) as usize
                )).areanum == 0
        {
            i += 1;
            continue;
        } //end if
        if AAS_FloodClusterAreas_r(
                (*aasworld.reachability.add(
                    ((*aasworld.areasettings.add(areanum as usize)).firstreachablearea + i) as usize
                )).areanum,
                clusternum
            ) == 0
        {
            return qfalse;
        }
        i += 1;
    } //end for
    return qtrue;
} //end of the function AAS_FloodClusterAreas_r

//===========================================================================
// try to flood from all areas without cluster into areas with a cluster set
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_FloodClusterAreasUsingReachabilities(clusternum: c_int) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut areanum: c_int;

    i = 1;
    while i < aasworld.numareas {
        //if this area already has a cluster set
        if (*aasworld.areasettings.add(i as usize)).cluster != 0 {
            i += 1;
            continue;
        }
        //if this area is a cluster portal
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            i += 1;
            continue;
        }
        //loop over the reachable areas from this area
        j = 0;
        while j < (*aasworld.areasettings.add(i as usize)).numreachableareas {
            //the reachable area
            areanum = (*aasworld.reachability.add(
                ((*aasworld.areasettings.add(i as usize)).firstreachablearea + j) as usize
            )).areanum;
            //if this area is a cluster portal
            if (*aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
                j += 1;
                continue;
            }
            //if this area has a cluster set
            if (*aasworld.areasettings.add(areanum as usize)).cluster != 0 {
                if AAS_FloodClusterAreas_r(i, clusternum) == 0 {
                    return qfalse;
                }
                i = 0;
                break;
            } //end if
            j += 1;
        } //end for
        i += 1;
    } //end for
    return qtrue;
} //end of the function AAS_FloodClusterAreasUsingReachabilities

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_NumberClusterPortals(clusternum: c_int) {
    let mut i: c_int;
    let mut portalnum: c_int;
    let cluster: *mut aas_cluster_t;
    let portal: *mut aas_portal_t;

    cluster = aasworld.clusters.add(clusternum as usize);
    i = 0;
    while i < (*cluster).numportals {
        portalnum = *aasworld.portalindex.add(((*cluster).firstportal + i) as usize);
        portal = aasworld.portals.add(portalnum as usize);
        if (*portal).frontcluster == clusternum {
            (*portal).clusterareanum[0] = (*cluster).numareas;
            (*cluster).numareas += 1;
        } //end if
        else {
            (*portal).clusterareanum[1] = (*cluster).numareas;
            (*cluster).numareas += 1;
        } //end else
        i += 1;
    } //end for
} //end of the function AAS_NumberClusterPortals

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_NumberClusterAreas(clusternum: c_int) {
    let mut i: c_int;
    let mut portalnum: c_int;
    let cluster: *mut aas_cluster_t;
    let portal: *mut aas_portal_t;

    (*aasworld.clusters.add(clusternum as usize)).numareas = 0;
    (*aasworld.clusters.add(clusternum as usize)).numreachabilityareas = 0;
    //number all areas in this cluster WITH reachabilities
    i = 1;
    while i < aasworld.numareas {
        //
        if (*aasworld.areasettings.add(i as usize)).cluster != clusternum {
            i += 1;
            continue;
        }
        //
        if AAS_AreaReachability(i) == 0 {
            i += 1;
            continue;
        }
        //
        (*aasworld.areasettings.add(i as usize)).clusterareanum = (*aasworld.clusters.add(clusternum as usize)).numareas;
        //the cluster has an extra area
        (*aasworld.clusters.add(clusternum as usize)).numareas += 1;
        (*aasworld.clusters.add(clusternum as usize)).numreachabilityareas += 1;
        i += 1;
    } //end for
    //number all portals in this cluster WITH reachabilities
    cluster = aasworld.clusters.add(clusternum as usize);
    i = 0;
    while i < (*cluster).numportals {
        portalnum = *aasworld.portalindex.add(((*cluster).firstportal + i) as usize);
        portal = aasworld.portals.add(portalnum as usize);
        if AAS_AreaReachability((*portal).areanum) == 0 {
            i += 1;
            continue;
        }
        if (*portal).frontcluster == clusternum {
            (*portal).clusterareanum[0] = (*cluster).numareas;
            (*cluster).numareas += 1;
            (*aasworld.clusters.add(clusternum as usize)).numreachabilityareas += 1;
        } //end if
        else {
            (*portal).clusterareanum[1] = (*cluster).numareas;
            (*cluster).numareas += 1;
            (*aasworld.clusters.add(clusternum as usize)).numreachabilityareas += 1;
        } //end else
        i += 1;
    } //end for
    //number all areas in this cluster WITHOUT reachabilities
    i = 1;
    while i < aasworld.numareas {
        //
        if (*aasworld.areasettings.add(i as usize)).cluster != clusternum {
            i += 1;
            continue;
        }
        //
        if AAS_AreaReachability(i) != 0 {
            i += 1;
            continue;
        }
        //
        (*aasworld.areasettings.add(i as usize)).clusterareanum = (*aasworld.clusters.add(clusternum as usize)).numareas;
        //the cluster has an extra area
        (*aasworld.clusters.add(clusternum as usize)).numareas += 1;
        i += 1;
    } //end for
    //number all portals in this cluster WITHOUT reachabilities
    cluster = aasworld.clusters.add(clusternum as usize);
    i = 0;
    while i < (*cluster).numportals {
        portalnum = *aasworld.portalindex.add(((*cluster).firstportal + i) as usize);
        portal = aasworld.portals.add(portalnum as usize);
        if AAS_AreaReachability((*portal).areanum) != 0 {
            i += 1;
            continue;
        }
        if (*portal).frontcluster == clusternum {
            (*portal).clusterareanum[0] = (*cluster).numareas;
            (*cluster).numareas += 1;
        } //end if
        else {
            (*portal).clusterareanum[1] = (*cluster).numareas;
            (*cluster).numareas += 1;
        } //end else
        i += 1;
    } //end for
} //end of the function AAS_NumberClusterAreas

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_FindClusters() -> c_int {
    let mut i: c_int;
    let cluster: *mut aas_cluster_t;

    AAS_RemoveClusterAreas();
    //
    i = 1;
    while i < aasworld.numareas {
        //if the area is already part of a cluster
        if (*aasworld.areasettings.add(i as usize)).cluster != 0 {
            i += 1;
            continue;
        }
        // if not flooding through faces only use areas that have reachabilities
        if nofaceflood != 0 {
            if (*aasworld.areasettings.add(i as usize)).numreachableareas == 0 {
                i += 1;
                continue;
            }
        } //end if
        //if the area is a cluster portal
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            i += 1;
            continue;
        }
        if aasworld.numclusters >= AAS_MAX_CLUSTERS {
            let fmt = b"AAS_MAX_CLUSTERS\0" as *const u8 as *const c_char;
            AAS_Error(fmt);
            return qfalse;
        } //end if
        cluster = aasworld.clusters.add(aasworld.numclusters as usize);
        (*cluster).numareas = 0;
        (*cluster).numreachabilityareas = 0;
        (*cluster).firstportal = aasworld.portalindexsize;
        (*cluster).numportals = 0;
        //flood the areas in this cluster
        if AAS_FloodClusterAreas_r(i, aasworld.numclusters) == 0 {
            return qfalse;
        }
        if AAS_FloodClusterAreasUsingReachabilities(aasworld.numclusters) == 0 {
            return qfalse;
        }
        //number the cluster areas
        //AAS_NumberClusterPortals(aasworld.numclusters);
        AAS_NumberClusterAreas(aasworld.numclusters);
        //Log_Write("cluster %d has %d areas\r\n", aasworld.numclusters, cluster->numareas);
        aasworld.numclusters += 1;
        i += 1;
    } //end for
    return qtrue;
} //end of the function AAS_FindClusters

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_CreatePortals() {
    let mut i: c_int;
    let portal: *mut aas_portal_t;

    i = 1;
    while i < aasworld.numareas {
        //if the area is a cluster portal
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            if aasworld.numportals >= AAS_MAX_PORTALS {
                let fmt = b"AAS_MAX_PORTALS\0" as *const u8 as *const c_char;
                AAS_Error(fmt);
                return;
            } //end if
            portal = aasworld.portals.add(aasworld.numportals as usize);
            (*portal).areanum = i;
            (*portal).frontcluster = 0;
            (*portal).backcluster = 0;
            aasworld.numportals += 1;
        } //end if
        i += 1;
    } //end for
} //end of the function AAS_CreatePortals

/*
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
int AAS_MapContainsTeleporters(void)
{
	bsp_entity_t *entities, *ent;
	char *classname;

	entities = AAS_ParseBSPEntities();

	for (ent = entities; ent; ent = ent->next)
	{
		classname = AAS_ValueForBSPEpairKey(ent, "classname");
		if (classname && !strcmp(classname, "misc_teleporter"))
		{
			AAS_FreeBSPEntities(entities);
			return qtrue;
		} //end if
	} //end for
	return qfalse;
} //end of the function AAS_MapContainsTeleporters
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
int AAS_NonConvexFaces(aas_face_t *face1, aas_face_t *face2, int side1, int side2)
{
	int i, j, edgenum;
	aas_plane_t *plane1, *plane2;
	aas_edge_t *edge;


	plane1 = &aasworld.planes[face1->planenum ^ side1];
	plane2 = &aasworld.planes[face2->planenum ^ side2];

	//check if one of the points of face1 is at the back of the plane of face2
	for (i = 0; i < face1->numedges; i++)
	{
		edgenum = abs(aasworld.edgeindex[face1->firstedge + i]);
		edge = &aasworld.edges[edgenum];
		for (j = 0; j < 2; j++)
		{
			if (DotProduct(plane2->normal, aasworld.vertexes[edge->v[j]]) -
							plane2->dist < -0.01) return qtrue;
		} //end for
	} //end for
	for (i = 0; i < face2->numedges; i++)
	{
		edgenum = abs(aasworld.edgeindex[face2->firstedge + i]);
		edge = &aasworld.edges[edgenum];
		for (j = 0; j < 2; j++)
		{
			if (DotProduct(plane1->normal, aasworld.vertexes[edge->v[j]]) -
							plane1->dist < -0.01) return qtrue;
		} //end for
	} //end for

	return qfalse;
} //end of the function AAS_NonConvexFaces
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
qboolean AAS_CanMergeAreas(int *areanums, int numareas)
{
	int i, j, s, face1num, face2num, side1, side2, fn1, fn2;
	aas_face_t *face1, *face2;
	aas_area_t *area1, *area2;

	for (i = 0; i < numareas; i++)
	{
		area1 = &aasworld.areas[areanums[i]];
		for (fn1 = 0; fn1 < area1->numfaces; fn1++)
		{
			face1num = abs(aasworld.faceindex[area1->firstface + fn1]);
			face1 = &aasworld.faces[face1num];
			side1 = face1->frontarea != areanums[i];
			//check if the face isn't a shared one with one of the other areas
			for (s = 0; s < numareas; s++)
			{
				if (s == i) continue;
				if (face1->frontarea == s || face1->backarea == s) break;
			} //end for
			//if the face was a shared one
			if (s != numareas) continue;
			//
			for (j = 0; j < numareas; j++)
			{
				if (j == i) continue;
				area2 = &aasworld.areas[areanums[j]];
				for (fn2 = 0; fn2 < area2->numfaces; fn2++)
				{
					face2num = abs(aasworld.faceindex[area2->firstface + fn2]);
					face2 = &aasworld.faces[face2num];
					side2 = face2->frontarea != areanums[j];
					//check if the face isn't a shared one with one of the other areas
					for (s = 0; s < numareas; s++)
					{
						if (s == j) continue;
						if (face2->frontarea == s || face2->backarea == s) break;
					} //end for
					//if the face was a shared one
					if (s != numareas) continue;
					//
					if (AAS_NonConvexFaces(face1, face2, side1, side2)) return qfalse;
				} //end for
			} //end for
		} //end for
	} //end for
	return qtrue;
} //end of the function AAS_CanMergeAreas
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
qboolean AAS_NonConvexEdges(aas_edge_t *edge1, aas_edge_t *edge2, int side1, int side2, int planenum)
{
	int i;
	vec3_t edgevec1, edgevec2, normal1, normal2;
	float dist1, dist2;
	aas_plane_t *plane;

	plane = &aasworld.planes[planenum];
	VectorSubtract(aasworld.vertexes[edge1->v[1]], aasworld.vertexes[edge1->v[0]], edgevec1);
	VectorSubtract(aasworld.vertexes[edge2->v[1]], aasworld.vertexes[edge2->v[0]], edgevec2);
	if (side1) VectorInverse(edgevec1);
	if (side2) VectorInverse(edgevec2);
	//
	CrossProduct(edgevec1, plane->normal, normal1);
	dist1 = DotProduct(normal1, aasworld.vertexes[edge1->v[0]]);
	CrossProduct(edgevec2, plane->normal, normal2);
	dist2 = DotProduct(normal2, aasworld.vertexes[edge2->v[0]]);

	for (i = 0; i < 2; i++)
	{
		if (DotProduct(aasworld.vertexes[edge1->v[i]], normal2) - dist2 < -0.01) return qfalse;
	} //end for
	for (i = 0; i < 2; i++)
	{
		if (DotProduct(aasworld.vertexes[edge2->v[i]], normal1) - dist1 < -0.01) return qfalse;
	} //end for
	return qtrue;
} //end of the function AAS_NonConvexEdges
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
qboolean AAS_CanMergeFaces(int *facenums, int numfaces, int planenum)
{
	int i, j, s, edgenum1, edgenum2, side1, side2, en1, en2, ens;
	aas_face_t *face1, *face2, *otherface;
	aas_edge_t *edge1, *edge2;

	for (i = 0; i < numfaces; i++)
	{
		face1 = &aasworld.faces[facenums[i]];
		for (en1 = 0; en1 < face1->numedges; en1++)
		{
			edgenum1 = aasworld.edgeindex[face1->firstedge + en1];
			side1 = (edgenum1 < 0) ^ (face1->planenum != planenum);
			edgenum1 = abs(edgenum1);
			edge1 = &aasworld.edges[edgenum1];
			//check if the edge is shared with another face
			for (s = 0; s < numfaces; s++)
			{
				if (s == i) continue;
				otherface = &aasworld.faces[facenums[s]];
				for (ens = 0; ens < otherface->numedges; ens++)
				{
					if (edgenum1 == abs(aasworld.edgeindex[otherface->firstedge + ens])) break;
				} //end for
				if (ens != otherface->numedges) break;
			} //end for
			//if the edge was shared
			if (s != numfaces) continue;
			//
			for (j = 0; j < numfaces; j++)
			{
				if (j == i) continue;
				face2 = &aasworld.faces[facenums[j]];
				for (en2 = 0; en2 < face2->numedges; en2++)
				{
					edgenum2 = aasworld.edgeindex[face2->firstedge + en2];
					side2 = (edgenum2 < 0) ^ (face2->planenum != planenum);
					edgenum2 = abs(edgenum2);
					edge2 = &aasworld.edges[edgenum2];
					//check if the edge is shared with another face
					for (s = 0; s < numfaces; s++)
					{
						if (s == i) continue;
						otherface = &aasworld.faces[facenums[s]];
						for (ens = 0; ens < otherface->numedges; ens++)
						{
							if (edgenum2 == abs(aasworld.edgeindex[otherface->firstedge + ens])) break;
						} //end for
						if (ens != otherface->numedges) break;
					} //end for
					//if the edge was shared
					if (s != numfaces) continue;
					//
					if (AAS_NonConvexEdges(edge1, edge2, side1, side2, planenum)) return qfalse;
				} //end for
			} //end for
		} //end for
	} //end for
	return qtrue;
} //end of the function AAS_CanMergeFaces*/

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ConnectedAreas_r(areanums: *mut c_int, numareas: c_int, connectedareas: *mut c_int, curarea: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut otherareanum: c_int;
    let mut facenum: c_int;
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;

    *connectedareas.add(curarea as usize) = qtrue;
    area = aasworld.areas.add(*areanums.add(curarea as usize) as usize);
    i = 0;
    while i < (*area).numfaces {
        facenum = (*aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
        face = aasworld.faces.add(facenum as usize);
        //if the face is solid
        if (*face).faceflags & FACE_SOLID != 0 {
            i += 1;
            continue;
        }
        //get the area at the other side of the face
        if (*face).frontarea != *areanums.add(curarea as usize) {
            otherareanum = (*face).frontarea;
        } else {
            otherareanum = (*face).backarea;
        }
        //check if the face is leading to one of the other areas
        j = 0;
        while j < numareas {
            if *areanums.add(j as usize) == otherareanum {
                break;
            }
            j += 1;
        } //end for
        //if the face isn't leading to one of the other areas
        if j == numareas {
            i += 1;
            continue;
        }
        //if the other area is already connected
        if *connectedareas.add(j as usize) != 0 {
            i += 1;
            continue;
        }
        //recursively proceed with the other area
        AAS_ConnectedAreas_r(areanums, numareas, connectedareas, j);
        i += 1;
    } //end for
} //end of the function AAS_ConnectedAreas_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ConnectedAreas(areanums: *mut c_int, numareas: c_int) -> c_int {
    let mut connectedareas: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut i: c_int;

    Com_Memset(
        connectedareas.as_mut_ptr() as *mut c_void,
        0,
        std::mem::size_of_val(&connectedareas)
    );
    if numareas < 1 {
        return qfalse;
    }
    if numareas == 1 {
        return qtrue;
    }
    AAS_ConnectedAreas_r(areanums, numareas, connectedareas.as_mut_ptr(), 0);
    i = 0;
    while i < numareas {
        if connectedareas[i as usize] == 0 {
            return qfalse;
        }
        i += 1;
    } //end for
    return qtrue;
} //end of the function AAS_ConnectedAreas

//===========================================================================
// gets adjacent areas with less presence types recursively
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_GetAdjacentAreasWithLessPresenceTypes_r(areanums: *mut c_int, mut numareas: c_int, curareanum: c_int) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let presencetype: c_int;
    let mut otherpresencetype: c_int;
    let mut otherareanum: c_int;
    let mut facenum: c_int;
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;

    *areanums.add(numareas as usize) = curareanum;
    numareas += 1;
    area = aasworld.areas.add(curareanum as usize);
    presencetype = (*aasworld.areasettings.add(curareanum as usize)).presencetype;
    i = 0;
    while i < (*area).numfaces {
        facenum = (*aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
        face = aasworld.faces.add(facenum as usize);
        //if the face is solid
        if (*face).faceflags & FACE_SOLID != 0 {
            i += 1;
            continue;
        }
        //the area at the other side of the face
        if (*face).frontarea != curareanum {
            otherareanum = (*face).frontarea;
        } else {
            otherareanum = (*face).backarea;
        }
        //
        otherpresencetype = (*aasworld.areasettings.add(otherareanum as usize)).presencetype;
        //if the other area has less presence types
        if ((presencetype & !otherpresencetype) != 0) &&
                ((otherpresencetype & !presencetype) == 0)
        {
            //check if the other area isn't already in the list
            j = 0;
            while j < numareas {
                if otherareanum == *areanums.add(j as usize) {
                    break;
                }
                j += 1;
            } //end for
            //if the other area isn't already in the list
            if j == numareas {
                if numareas >= MAX_PORTALAREAS {
                    let fmt = b"MAX_PORTALAREAS\0" as *const u8 as *const c_char;
                    AAS_Error(fmt);
                    return numareas;
                } //end if
                numareas = AAS_GetAdjacentAreasWithLessPresenceTypes_r(areanums, numareas, otherareanum);
            } //end if
        } //end if
        i += 1;
    } //end for
    return numareas;
} //end of the function AAS_GetAdjacentAreasWithLessPresenceTypes_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_CheckAreaForPossiblePortals(areanum: c_int) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut fen: c_int;
    let mut ben: c_int;
    let mut frontedgenum: c_int;
    let mut backedgenum: c_int;
    let mut facenum: c_int;
    let mut areanums: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut numareas: c_int;
    let mut otherareanum: c_int;
    let mut numareafrontfaces: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut numareabackfaces: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut frontfacenums: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut backfacenums: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut numfrontfaces: c_int;
    let mut numbackfaces: c_int;
    let mut frontareanums: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut backareanums: [c_int; MAX_PORTALAREAS as usize] = [0; MAX_PORTALAREAS as usize];
    let mut numfrontareas: c_int;
    let mut numbackareas: c_int;
    let mut frontplanenum: c_int;
    let mut backplanenum: c_int;
    let mut faceplanenum: c_int;
    let area: *mut aas_area_t;
    let frontface: *mut aas_face_t;
    let backface: *mut aas_face_t;
    let face: *mut aas_face_t;

    //if it isn't already a portal
    if (*aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
        return 0;
    }
    //it must be a grounded area
    if (*aasworld.areasettings.add(areanum as usize)).areaflags & AREA_GROUNDED == 0 {
        return 0;
    }
    //
    Com_Memset(
        numareafrontfaces.as_mut_ptr() as *mut c_void,
        0,
        std::mem::size_of_val(&numareafrontfaces)
    );
    Com_Memset(
        numareabackfaces.as_mut_ptr() as *mut c_void,
        0,
        std::mem::size_of_val(&numareabackfaces)
    );
    numareas = 0;
    numfrontfaces = 0;
    numbackfaces = 0;
    numfrontareas = 0;
    numbackareas = 0;
    frontplanenum = -1;
    backplanenum = -1;
    //add any adjacent areas with less presence types
    numareas = AAS_GetAdjacentAreasWithLessPresenceTypes_r(areanums.as_mut_ptr(), 0, areanum);
    //
    i = 0;
    while i < numareas {
        area = aasworld.areas.add(areanums[i as usize] as usize);
        j = 0;
        while j < (*area).numfaces {
            facenum = (*aasworld.faceindex.add(((*area).firstface + j) as usize)).abs();
            face = aasworld.faces.add(facenum as usize);
            //if the face is solid
            if (*face).faceflags & FACE_SOLID != 0 {
                j += 1;
                continue;
            }
            //check if the face is shared with one of the other areas
            k = 0;
            while k < numareas {
                if k == i {
                    k += 1;
                    continue;
                }
                if (*face).frontarea == areanums[k as usize] || (*face).backarea == areanums[k as usize] {
                    break;
                }
                k += 1;
            } //end for
            //if the face is shared
            if k != numareas {
                j += 1;
                continue;
            }
            //the number of the area at the other side of the face
            if (*face).frontarea == areanums[i as usize] {
                otherareanum = (*face).backarea;
            } else {
                otherareanum = (*face).frontarea;
            }
            //if the other area already is a cluter portal
            if (*aasworld.areasettings.add(otherareanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
                return 0;
            }
            //number of the plane of the area
            faceplanenum = (*face).planenum & !1;
            //
            if frontplanenum < 0 || faceplanenum == frontplanenum {
                frontplanenum = faceplanenum;
                frontfacenums[numfrontfaces as usize] = facenum;
                numfrontfaces += 1;
                k = 0;
                while k < numfrontareas {
                    if frontareanums[k as usize] == otherareanum {
                        break;
                    }
                    k += 1;
                } //end for
                if k == numfrontareas {
                    frontareanums[numfrontareas as usize] = otherareanum;
                    numfrontareas += 1;
                }
                numareafrontfaces[i as usize] += 1;
            } //end if
            else if backplanenum < 0 || faceplanenum == backplanenum {
                backplanenum = faceplanenum;
                backfacenums[numbackfaces as usize] = facenum;
                numbackfaces += 1;
                k = 0;
                while k < numbackareas {
                    if backareanums[k as usize] == otherareanum {
                        break;
                    }
                    k += 1;
                } //end for
                if k == numbackareas {
                    backareanums[numbackareas as usize] = otherareanum;
                    numbackareas += 1;
                }
                numareabackfaces[i as usize] += 1;
            } //end else
            else {
                return 0;
            } //end else
            j += 1;
        } //end for
        i += 1;
    } //end for
    //every area should have at least one front face and one back face
    i = 0;
    while i < numareas {
        if numareafrontfaces[i as usize] == 0 || numareabackfaces[i as usize] == 0 {
            return 0;
        }
        i += 1;
    } //end for
    //the front areas should all be connected
    if AAS_ConnectedAreas(frontareanums.as_mut_ptr(), numfrontareas) == 0 {
        return 0;
    }
    //the back areas should all be connected
    if AAS_ConnectedAreas(backareanums.as_mut_ptr(), numbackareas) == 0 {
        return 0;
    }
    //none of the front faces should have a shared edge with a back face
    i = 0;
    while i < numfrontfaces {
        frontface = aasworld.faces.add(frontfacenums[i as usize] as usize);
        fen = 0;
        while fen < (*frontface).numedges {
            frontedgenum = (*aasworld.edgeindex.add(((*frontface).firstedge + fen) as usize)).abs();
            j = 0;
            while j < numbackfaces {
                backface = aasworld.faces.add(backfacenums[j as usize] as usize);
                ben = 0;
                while ben < (*backface).numedges {
                    backedgenum = (*aasworld.edgeindex.add(((*backface).firstedge + ben) as usize)).abs();
                    if frontedgenum == backedgenum {
                        break;
                    }
                    ben += 1;
                } //end for
                if ben != (*backface).numedges {
                    break;
                }
                j += 1;
            } //end for
            if j != numbackfaces {
                break;
            }
            fen += 1;
        } //end for
        if fen != (*frontface).numedges {
            break;
        }
        i += 1;
    } //end for
    if i != numfrontfaces {
        return 0;
    }
    //set the cluster portal contents
    i = 0;
    while i < numareas {
        (*aasworld.areasettings.add(areanums[i as usize] as usize)).contents |= AREACONTENTS_CLUSTERPORTAL;
        //this area can be used as a route portal
        (*aasworld.areasettings.add(areanums[i as usize] as usize)).contents |= AREACONTENTS_ROUTEPORTAL;
        let fmt = b"possible portal: %d\r\n\0" as *const u8 as *const c_char;
        Log_Write(fmt, areanums[i as usize]);
        i += 1;
    } //end for
    //
    return numareas;
} //end of the function AAS_CheckAreaForPossiblePortals

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_FindPossiblePortals() {
    let mut i: c_int;
    let mut numpossibleportals: c_int;

    numpossibleportals = 0;
    i = 1;
    while i < aasworld.numareas {
        numpossibleportals += AAS_CheckAreaForPossiblePortals(i);
        i += 1;
    } //end for
    let fmt = b"\r%6d possible portal areas\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, numpossibleportals);
} //end of the function AAS_FindPossiblePortals

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_RemoveAllPortals() {
    let mut i: c_int;

    i = 1;
    while i < aasworld.numareas {
        (*aasworld.areasettings.add(i as usize)).contents &= !AREACONTENTS_CLUSTERPORTAL;
        i += 1;
    } //end for
} //end of the function AAS_RemoveAllPortals

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_TestPortals() -> c_int {
    let mut i: c_int;
    let portal: *mut aas_portal_t;

    i = 1;
    while i < aasworld.numportals {
        portal = aasworld.portals.add(i as usize);
        if (*portal).frontcluster == 0 {
            (*aasworld.areasettings.add((*portal).areanum as usize)).contents &= !AREACONTENTS_CLUSTERPORTAL;
            let fmt = b"portal area %d has no front cluster\r\n\0" as *const u8 as *const c_char;
            Log_Write(fmt, (*portal).areanum);
            return qfalse;
        } //end if
        if (*portal).backcluster == 0 {
            (*aasworld.areasettings.add((*portal).areanum as usize)).contents &= !AREACONTENTS_CLUSTERPORTAL;
            let fmt = b"portal area %d has no back cluster\r\n\0" as *const u8 as *const c_char;
            Log_Write(fmt, (*portal).areanum);
            return qfalse;
        } //end if
        i += 1;
    } //end for
    return qtrue;
} //end of the function

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_CountForcedClusterPortals() {
    let mut num: c_int;
    let mut i: c_int;

    num = 0;
    i = 1;
    while i < aasworld.numareas {
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            let fmt = b"area %d is a forced portal area\r\n\0" as *const u8 as *const c_char;
            Log_Write(fmt, i);
            num += 1;
        } //end if
        i += 1;
    } //end for
    let fmt = b"%6d forced portal areas\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, num);
} //end of the function AAS_CountForcedClusterPortals

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_CreateViewPortals() {
    let mut i: c_int;

    i = 1;
    while i < aasworld.numareas {
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            (*aasworld.areasettings.add(i as usize)).contents |= AREACONTENTS_VIEWPORTAL;
        } //end if
        i += 1;
    } //end for
} //end of the function AAS_CreateViewPortals

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_SetViewPortalsAsClusterPortals() {
    let mut i: c_int;

    i = 1;
    while i < aasworld.numareas {
        if (*aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_VIEWPORTAL != 0 {
            (*aasworld.areasettings.add(i as usize)).contents |= AREACONTENTS_CLUSTERPORTAL;
        } //end if
        i += 1;
    } //end for
} //end of the function AAS_SetViewPortalsAsClusterPortals

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_InitClustering() {
    let mut i: c_int;
    let mut removedPortalAreas: c_int;
    let mut n: c_int;
    let mut total: c_int;
    let mut numreachabilityareas: c_int;

    if aasworld.loaded == 0 {
        return;
    }
    //if there are clusters
    if aasworld.numclusters >= 1 {
        //if clustering isn't forced
        if LibVarGetValue(b"forceclustering\0" as *const u8 as *const c_char) as c_int == 0 &&
            LibVarGetValue(b"forcereachability\0" as *const u8 as *const c_char) as c_int == 0
        {
            return;
        }
    } //end if
    //set all view portals as cluster portals in case we re-calculate the reachabilities and clusters (with -reach)
    AAS_SetViewPortalsAsClusterPortals();
    //count the number of forced cluster portals
    AAS_CountForcedClusterPortals();
    //remove all area cluster marks
    AAS_RemoveClusterAreas();
    //find possible cluster portals
    AAS_FindPossiblePortals();
    //craete portals to for the bot view
    AAS_CreateViewPortals();
    //remove all portals that are not closing a cluster
    //AAS_RemoveNotClusterClosingPortals();
    //initialize portal memory
    if !aasworld.portals.is_null() {
        FreeMemory(aasworld.portals as *mut c_void);
    }
    aasworld.portals = GetClearedMemory((AAS_MAX_PORTALS as usize) * std::mem::size_of::<aas_portal_t>()) as *mut aas_portal_t;
    //initialize portal index memory
    if !aasworld.portalindex.is_null() {
        FreeMemory(aasworld.portalindex as *mut c_void);
    }
    aasworld.portalindex = GetClearedMemory((AAS_MAX_PORTALINDEXSIZE as usize) * std::mem::size_of::<aas_portalindex_t>()) as *mut aas_portalindex_t;
    //initialize cluster memory
    if !aasworld.clusters.is_null() {
        FreeMemory(aasworld.clusters as *mut c_void);
    }
    aasworld.clusters = GetClearedMemory((AAS_MAX_CLUSTERS as usize) * std::mem::size_of::<aas_cluster_t>()) as *mut aas_cluster_t;
    //
    removedPortalAreas = 0;
    let fmt = b"\r%6d removed portal areas\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, removedPortalAreas);
    loop {
        let fmt = b"\r%6d\0" as *const u8 as *const c_char;
        (botimport.Print)(PRT_MESSAGE, fmt, removedPortalAreas);
        //initialize the number of portals and clusters
        aasworld.numportals = 1;		//portal 0 is a dummy
        aasworld.portalindexsize = 0;
        aasworld.numclusters = 1;		//cluster 0 is a dummy
        //create the portals from the portal areas
        AAS_CreatePortals();
        //
        removedPortalAreas += 1;
        //find the clusters
        if AAS_FindClusters() == 0 {
            continue;
        }
        //test the portals
        if AAS_TestPortals() == 0 {
            continue;
        }
        //
        break;
    } //end while
    let fmt = b"\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt);
    //the AAS file should be saved
    aasworld.savefile = qtrue;
    //write the portal areas to the log file
    i = 1;
    while i < aasworld.numportals {
        let fmt = b"portal %d: area %d\r\n\0" as *const u8 as *const c_char;
        Log_Write(fmt, i, (*aasworld.portals.add(i as usize)).areanum);
        i += 1;
    } //end for
    // report cluster info
    let fmt = b"%6d portals created\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, aasworld.numportals);
    let fmt = b"%6d clusters created\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, aasworld.numclusters);
    i = 1;
    while i < aasworld.numclusters {
        let fmt = b"cluster %d has %d reachability areas\n\0" as *const u8 as *const c_char;
        (botimport.Print)(PRT_MESSAGE, fmt, i,
                (*aasworld.clusters.add(i as usize)).numreachabilityareas);
        i += 1;
    } //end for
    // report AAS file efficiency
    numreachabilityareas = 0;
    total = 0;
    i = 0;
    while i < aasworld.numclusters {
        n = (*aasworld.clusters.add(i as usize)).numreachabilityareas;
        numreachabilityareas += n;
        total += n * n;
        i += 1;
    }
    total += numreachabilityareas * aasworld.numportals;
    //
    let fmt = b"%6i total reachability areas\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, numreachabilityareas);
    let fmt = b"%6i AAS memory/CPU usage (the lower the better)\n\0" as *const u8 as *const c_char;
    (botimport.Print)(PRT_MESSAGE, fmt, total * 3);
} //end of the function AAS_InitClustering
