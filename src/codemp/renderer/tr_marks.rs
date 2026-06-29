// tr_marks.c -- polygon projection on the world polygons

#![allow(non_snake_case)]

use core::ffi::c_int;

// Type aliases matching C definitions
pub type vec3_t = [f32; 3];
pub type vec_t = f32;
pub type surfaceType_t = c_int;

const MAX_VERTS_ON_POLY: usize = 64;

const MARKER_OFFSET: f32 = 0.0; // 1

// Surface type constants
const SF_FACE: c_int = 2;
const SF_GRID: c_int = 3;

// Surface flag constants
const SURF_NOIMPACT: c_int = 0x0010;
const SURF_NOMARKS: c_int = 0x0040;

// Content flag constants
const CONTENTS_FOG: c_int = 0x00000010;

// Vertex size constants
const MAXLIGHTMAPS: usize = 4;
#[cfg(feature = "xbox")]
const VERTEXSIZE: usize = 9 + (MAXLIGHTMAPS * 3);
#[cfg(not(feature = "xbox"))]
const VERTEXSIZE: usize = 6 + (MAXLIGHTMAPS * 3);

#[cfg(feature = "xbox")]
const NEXT_SURFPOINT_BASE: usize = 8;

// cplane_t structure from C headers
#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub plane_type: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

// shader_t structure (partial - from oracle/codemp/renderer/tr_local.h)
// Includes fields up to and including contentFlags
#[repr(C)]
pub struct shader_t {
    pub name: [u8; 64],
    pub lightmapIndex: [c_int; 4],
    pub styles: [u8; 4],
    pub index: c_int,
    pub sortedIndex: c_int,
    pub sort: f32,
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
}

// drawVert_t structure (opaque)
#[repr(C)]
pub struct drawVert_t {
    pub xyz: vec3_t,
    pub normal: vec3_t,
    _unused: [u8; 0],
}

// markFragment_t structure (opaque)
#[repr(C)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

// srfGridMesh_t structure (partial)
#[repr(C)]
pub struct srfGridMesh_t {
    pub surfaceType: surfaceType_t,
    pub dlightBits: c_int,
    pub meshBounds: [vec3_t; 2],
    pub localOrigin: vec3_t,
    pub meshRadius: f32,
    pub lodOrigin: vec3_t,
    pub lodRadius: f32,
    pub lodFixed: c_int,
    pub lodStitched: c_int,
    pub width: c_int,
    pub height: c_int,
    pub widthLodError: *mut f32,
    pub heightLodError: *mut f32,
    pub verts: *mut drawVert_t,
}

// srfSurfaceFace_t structure (platform-specific)
#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct srfSurfaceFace_t {
    pub surfaceType: surfaceType_t,
    pub plane: cplane_t,
    pub dlightBits: c_int,
    pub numPoints: u8,
    pub numIndices: u16,
    pub ofsIndices: u16,
    pub flags: u8,
    pub srfPoints: *mut u16,
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct srfSurfaceFace_t {
    pub surfaceType: surfaceType_t,
    pub plane: cplane_t,
    pub dlightBits: c_int,
    pub numPoints: c_int,
    pub numIndices: c_int,
    pub ofsIndices: c_int,
    pub points: *mut f32,
}

// msurface_t structure
#[repr(C)]
pub struct msurface_t {
    pub viewCount: c_int,
    pub shader: *mut shader_t,
    pub fogIndex: c_int,
    pub data: *mut surfaceType_t,
}

// mnode_t structure (platform-specific)
#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct mnode_t {
    pub contents: i8,
    pub visframe: c_int,
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub parent: *mut mnode_t,
    pub planeNum: c_int,
    pub children: [*mut mnode_t; 2],
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct mnode_t {
    pub contents: c_int,
    pub visframe: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub parent: *mut mnode_t,
    pub plane: *mut cplane_t,
    pub children: [*mut mnode_t; 2],
    pub cluster: c_int,
    pub area: c_int,
    pub firstmarksurface: *mut *mut msurface_t,
    pub nummarksurfaces: c_int,
}

// mleaf_s structure (platform-specific)
#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct mleaf_s {
    pub contents: i8,
    pub visframe: c_int,
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub parent: *mut mnode_t,
    pub cluster: i16,
    pub area: i8,
    pub firstMarkSurfNum: u16,
    pub nummarksurfaces: i16,
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct mleaf_s {
    pub contents: c_int,
    pub visframe: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub parent: *mut mnode_t,
    pub cluster: c_int,
    pub area: c_int,
    pub firstmarksurface: *mut *mut msurface_t,
    pub nummarksurfaces: c_int,
}

// world_t structure (partial - used via tr.world)
#[repr(C)]
pub struct world_t {
    _unused: [u8; 0],
}

// trGlobals_t structure (partial - used for tr.viewCount and tr.world)
#[repr(C)]
pub struct world_with_ptrs {
    pub nodes: *mut mnode_t,
    pub planes: *mut cplane_t,
    pub marksurfaces: *mut *mut msurface_t,
}

#[repr(C)]
pub struct trGlobals_t {
    pub viewCount: c_int,
    pub world: *mut world_with_ptrs,
}

// Global variables
extern "C" {
    pub static mut tr: trGlobals_t;
}

// External C functions
extern "C" {
    pub fn DotProduct(a: *const f32, b: *const f32) -> f32;
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    pub fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    pub fn CrossProduct(a: *const f32, b: *const f32, out: *mut f32);
    pub fn VectorNormalize2(vec: *const f32, out: *mut f32) -> f32;
    pub fn VectorNormalizeFast(v: *mut f32);
    pub fn VectorInverse(v: *mut f32);
    pub fn VectorMA(v: *const f32, s: f32, add: *const f32, out: *mut f32);
    pub fn AddPointToBounds(p: *const f32, mins: *mut f32, maxs: *mut f32);
    pub fn ClearBounds(mins: *mut f32, maxs: *mut f32);
    pub fn Com_Memcpy(dest: *mut u8, src: *const u8, count: usize);
    pub fn BoxOnPlaneSide(emins: *const f32, emaxs: *const f32, p: *const cplane_t) -> c_int;
    #[cfg(feature = "xbox")]
    pub fn Q_CastShort2Float(fVec: *mut f32, sVec: *mut i16);
}

/*
=============
R_ChopPolyBehindPlane

Out must have space for two more vertexes than in
=============
*/
unsafe fn R_ChopPolyBehindPlane(
    numInPoints: c_int,
    inPoints: *const [f32; 3],
    numOutPoints: *mut c_int,
    outPoints: *mut [f32; 3],
    normal: *const f32,
    dist: f32,
    epsilon: f32,
) {
    let mut dists = [0.0f32; MAX_VERTS_ON_POLY + 4];
    let mut sides = [0c_int; MAX_VERTS_ON_POLY + 4];
    let mut counts = [0c_int; 3];
    let mut dot: f32;
    let mut i: c_int;
    let mut j: c_int;
    let mut p1: *const f32;
    let mut p2: *const f32;
    let mut clip: *mut f32;
    let mut d: f32;

    // don't clip if it might overflow
    if numInPoints >= (MAX_VERTS_ON_POLY - 2) as c_int {
        *numOutPoints = 0;
        return;
    }

    counts[0] = 0;
    counts[1] = 0;
    counts[2] = 0;

    // determine sides for each point
    i = 0;
    while i < numInPoints {
        dot = DotProduct(&(*inPoints)[i as usize][0], normal);
        dot -= dist;
        dists[i as usize] = dot;
        if dot > epsilon {
            sides[i as usize] = SIDE_FRONT;
        } else if dot < -epsilon {
            sides[i as usize] = SIDE_BACK;
        } else {
            sides[i as usize] = SIDE_ON;
        }
        counts[sides[i as usize] as usize] += 1;
        i += 1;
    }
    sides[numInPoints as usize] = sides[0];
    dists[numInPoints as usize] = dists[0];

    *numOutPoints = 0;

    if counts[0] == 0 {
        return;
    }
    if counts[1] == 0 {
        *numOutPoints = numInPoints;
        Com_Memcpy(
            outPoints as *mut u8,
            inPoints as *const u8,
            (numInPoints as usize) * core::mem::size_of::<vec3_t>(),
        );
        return;
    }

    i = 0;
    while i < numInPoints {
        p1 = &(*inPoints)[i as usize][0];
        clip = &mut (*outPoints)[*numOutPoints as usize][0];

        if sides[i as usize] == SIDE_ON {
            VectorCopy(p1, clip);
            *numOutPoints += 1;
            i += 1;
            continue;
        }

        if sides[i as usize] == SIDE_FRONT {
            VectorCopy(p1, clip);
            *numOutPoints += 1;
            clip = &mut (*outPoints)[*numOutPoints as usize][0];
        }

        if sides[(i + 1) as usize] == SIDE_ON
            || sides[(i + 1) as usize] == sides[i as usize]
        {
            i += 1;
            continue;
        }

        // generate a split point
        p2 = &(*inPoints)[((i + 1) % numInPoints) as usize][0];

        d = dists[i as usize] - dists[(i + 1) as usize];
        if d == 0.0 {
            dot = 0.0;
        } else {
            dot = dists[i as usize] / d;
        }

        // clip xyz

        j = 0;
        while j < 3 {
            *clip.offset(j) = *p1.offset(j) + dot * (*p2.offset(j) - *p1.offset(j));
            j += 1;
        }

        *numOutPoints += 1;
        i += 1;
    }
}

/*
=================
R_BoxSurfaces_r

=================
*/
pub unsafe fn R_BoxSurfaces_r(
    node: *mut mnode_t,
    mins: *const f32,
    maxs: *const f32,
    list: *mut *const surfaceType_t,
    listsize: c_int,
    listlength: *mut c_int,
    dir: *const f32,
) {
    let mut s: c_int;
    let mut c: c_int;
    let mut surf: *mut msurface_t;
    let mut mark: *mut *mut msurface_t;
    let mut node = node;

    // do the tail recursion in a loop
    while (*node).contents == -1 {
        #[cfg(feature = "xbox")]
        {
            let plane_ptr = (*tr.world).planes.offset((*node).planeNum as isize);
            s = BoxOnPlaneSide(mins, maxs, plane_ptr);
        }
        #[cfg(not(feature = "xbox"))]
        {
            s = BoxOnPlaneSide(mins, maxs, (*node).plane);
        }
        if s == 1 {
            node = (*node).children[0];
        } else if s == 2 {
            node = (*node).children[1];
        } else {
            R_BoxSurfaces_r(
                (*node).children[0],
                mins,
                maxs,
                list,
                listsize,
                listlength,
                dir,
            );
            node = (*node).children[1];
        }
    }

    // add the individual surfaces
    #[cfg(feature = "xbox")]
    {
        let leaf = node as *mut mleaf_s;
        mark = (*tr.world).marksurfaces.offset((*leaf).firstMarkSurfNum as isize);
        c = (*leaf).nummarksurfaces as c_int;
    }
    #[cfg(not(feature = "xbox"))]
    {
        mark = (*node).firstmarksurface;
        c = (*node).nummarksurfaces;
    }
    while c > 0 {
        //
        if *listlength >= listsize {
            break;
        }
        //
        surf = *mark;
        // check if the surface has NOIMPACT or NOMARKS set
        if ((*(*surf).shader).surfaceFlags & (SURF_NOIMPACT | SURF_NOMARKS)) != 0
            || ((*(*surf).shader).contentFlags & CONTENTS_FOG) != 0
        {
            (*surf).viewCount = tr.viewCount;
        }
        // extra check for surfaces to avoid list overflows
        else if *((*surf).data as *const surfaceType_t) == SF_FACE {
            // the face plane should go through the box
            s = BoxOnPlaneSide(
                mins,
                maxs,
                &(*((*surf).data as *mut srfSurfaceFace_t)).plane,
            );
            if s == 1 || s == 2 {
                (*surf).viewCount = tr.viewCount;
            } else if DotProduct(
                &(*((*surf).data as *mut srfSurfaceFace_t)).plane.normal[0] as *const f32,
                dir,
            ) > -0.5
            {
                // don't add faces that make sharp angles with the projection direction
                (*surf).viewCount = tr.viewCount;
            }
        } else if *((*surf).data as *const surfaceType_t) != SF_GRID {
            (*surf).viewCount = tr.viewCount;
        }
        // check the viewCount because the surface may have
        // already been added if it spans multiple leafs
        if (*surf).viewCount != tr.viewCount {
            (*surf).viewCount = tr.viewCount;
            *list.offset(*listlength as isize) = (*surf).data as *const surfaceType_t;
            *listlength += 1;
        }
        mark = mark.offset(1);
        c -= 1;
    }
}

/*
=================
R_AddMarkFragments

=================
*/
pub unsafe fn R_AddMarkFragments(
    numClipPoints: c_int,
    clipPoints: *mut [[vec3_t; MAX_VERTS_ON_POLY]; 2],
    numPlanes: c_int,
    normals: *const [f32; 3],
    dists: *const f32,
    maxPoints: c_int,
    pointBuffer: *mut vec3_t,
    maxFragments: c_int,
    fragmentBuffer: *mut markFragment_t,
    returnedPoints: *mut c_int,
    returnedFragments: *mut c_int,
    mins: *const f32,
    maxs: *const f32,
) {
    let mut pingPong: c_int;
    let mut i: c_int;
    let mut mf: *mut markFragment_t;
    let mut numClipPoints = numClipPoints;

    // chop the surface by all the bounding planes of the to be projected polygon
    pingPong = 0;

    i = 0;
    while i < numPlanes {
        R_ChopPolyBehindPlane(
            numClipPoints,
            &(*clipPoints)[pingPong as usize],
            &mut numClipPoints,
            &mut (*clipPoints)[(!pingPong) as usize],
            normals.offset(i as isize),
            *dists.offset(i as isize),
            0.5,
        );
        pingPong ^= 1;
        if numClipPoints == 0 {
            break;
        }
        i += 1;
    }
    // completely clipped away?
    if numClipPoints == 0 {
        return;
    }

    // add this fragment to the returned list
    if numClipPoints + (*returnedPoints) > maxPoints {
        return; // not enough space for this polygon
    }
    /*
    // all the clip points should be within the bounding box
    for ( i = 0 ; i < numClipPoints ; i++ ) {
        int j;
        for ( j = 0 ; j < 3 ; j++ ) {
            if (clipPoints[pingPong][i][j] < mins[j] - 0.5) break;
            if (clipPoints[pingPong][i][j] > maxs[j] + 0.5) break;
        }
        if (j < 3) break;
    }
    if (i < numClipPoints) return;
    */

    mf = &mut *fragmentBuffer.offset(*returnedFragments as isize);
    (*mf).firstPoint = (*returnedPoints);
    (*mf).numPoints = numClipPoints;
    Com_Memcpy(
        pointBuffer.offset(*returnedPoints as isize) as *mut u8,
        &(*clipPoints)[pingPong as usize][0] as *const [f32; 3] as *const u8,
        (numClipPoints as usize) * core::mem::size_of::<vec3_t>(),
    );

    *returnedPoints += numClipPoints;
    *returnedFragments += 1;
}

/*
=================
R_MarkFragments

=================
*/
pub unsafe fn R_MarkFragments(
    numPoints: c_int,
    points: *const vec3_t,
    projection: *const vec3_t,
    maxPoints: c_int,
    pointBuffer: *mut vec3_t,
    maxFragments: c_int,
    fragmentBuffer: *mut markFragment_t,
) -> c_int {
    let mut numsurfaces: c_int;
    let mut numPlanes: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut m: c_int;
    let mut n: c_int;
    let mut surfaces = [0 as *const surfaceType_t; 64];
    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    let mut returnedFragments: c_int;
    let mut returnedPoints: c_int;
    let mut normals = [[0.0f32; 3]; MAX_VERTS_ON_POLY + 2];
    let mut dists = [0.0f32; MAX_VERTS_ON_POLY + 2];
    let mut clipPoints = [[[0.0f32; 3]; MAX_VERTS_ON_POLY]; 2];
    let mut numClipPoints: c_int;
    let mut v: *const f32;
    let mut surf: *mut srfSurfaceFace_t;
    let mut cv: *mut srfGridMesh_t;
    let mut dv: *mut drawVert_t;
    let mut normal = [0.0f32; 3];
    let mut projectionDir = [0.0f32; 3];
    let mut v1 = [0.0f32; 3];
    let mut v2 = [0.0f32; 3];
    let mut indexes: *const c_int;

    //increment view count for double check prevention
    tr.viewCount += 1;

    //
    VectorNormalize2(&(*points)[0][0], &mut projectionDir[0]);
    // find all the brushes that are to be considered
    ClearBounds(&mut mins[0], &mut maxs[0]);
    i = 0;
    while i < numPoints {
        let mut temp = [0.0f32; 3];

        AddPointToBounds(&(*points.offset(i as isize))[0], &mut mins[0], &mut maxs[0]);
        VectorAdd(
            &(*points.offset(i as isize))[0],
            &(*projection)[0],
            &mut temp[0],
        );
        AddPointToBounds(&temp[0], &mut mins[0], &mut maxs[0]);
        // make sure we get all the leafs (also the one(s) in front of the hit surface)
        VectorMA(
            &(*points.offset(i as isize))[0],
            -20.0,
            &projectionDir[0],
            &mut temp[0],
        );
        AddPointToBounds(&temp[0], &mut mins[0], &mut maxs[0]);
        i += 1;
    }

    let mut numPoints = numPoints;
    if numPoints > MAX_VERTS_ON_POLY as c_int {
        numPoints = MAX_VERTS_ON_POLY as c_int;
    }
    // create the bounding planes for the to be projected polygon
    i = 0;
    while i < numPoints {
        VectorSubtract(
            &(*points.offset((((i + 1) % numPoints) as isize)))[0],
            &(*points.offset(i as isize))[0],
            &mut v1[0],
        );
        VectorAdd(
            &(*points.offset(i as isize))[0],
            &(*projection)[0],
            &mut v2[0],
        );
        VectorSubtract(
            &(*points.offset(i as isize))[0],
            &v2[0],
            &mut v2[0],
        );
        CrossProduct(&v1[0], &v2[0], &mut normals[i as usize][0]);
        VectorNormalizeFast(&mut normals[i as usize][0]);
        dists[i as usize] = DotProduct(&normals[i as usize][0], &(*points.offset(i as isize))[0]);
        i += 1;
    }
    // add near and far clipping planes for projection
    VectorCopy(&projectionDir[0], &mut normals[numPoints as usize][0]);
    dists[numPoints as usize] = DotProduct(
        &normals[numPoints as usize][0],
        &(*points.offset(0))[0],
    ) - 32.0;
    VectorCopy(&projectionDir[0], &mut normals[(numPoints + 1) as usize][0]);
    VectorInverse(&mut normals[(numPoints + 1) as usize][0]);
    dists[(numPoints + 1) as usize] = DotProduct(
        &normals[(numPoints + 1) as usize][0],
        &(*points.offset(0))[0],
    ) - 20.0;
    numPlanes = numPoints + 2;

    numsurfaces = 0;
    R_BoxSurfaces_r(
        (*tr.world).nodes,
        &mins[0],
        &maxs[0],
        &mut surfaces[0],
        64,
        &mut numsurfaces,
        &projectionDir[0],
    );
    //assert(numsurfaces <= 64);
    //assert(numsurfaces != 64);

    returnedPoints = 0;
    returnedFragments = 0;

    i = 0;
    while i < numsurfaces {
        if *surfaces[i as usize] == SF_GRID {
            cv = surfaces[i as usize] as *mut srfGridMesh_t;
            m = 0;
            while m < (*cv).height - 1 {
                n = 0;
                while n < (*cv).width - 1 {
                    // We triangulate the grid and chop all triangles within
                    // the bounding planes of the to be projected polygon.
                    // LOD is not taken into account, not such a big deal though.
                    //
                    // It's probably much nicer to chop the grid itself and deal
                    // with this grid as a normal SF_GRID surface so LOD will
                    // be applied. However the LOD of that chopped grid must
                    // be synced with the LOD of the original curve.
                    // One way to do this; the chopped grid shares vertices with
                    // the original curve. When LOD is applied to the original
                    // curve the unused vertices are flagged. Now the chopped curve
                    // should skip the flagged vertices. This still leaves the
                    // problems with the vertices at the chopped grid edges.
                    //
                    // To avoid issues when LOD applied to "hollow curves" (like
                    // the ones around many jump pads) we now just add a 2 unit
                    // offset to the triangle vertices.
                    // The offset is added in the vertex normal vector direction
                    // so all triangles will still fit together.
                    // The 2 unit offset should avoid pretty much all LOD problems.

                    numClipPoints = 3;

                    dv = (*cv).verts.offset((m * (*cv).width + n) as isize);

                    VectorCopy(&(*dv).xyz[0], &mut clipPoints[0][0][0]);
                    VectorMA(
                        &clipPoints[0][0][0],
                        MARKER_OFFSET,
                        &(*dv).normal[0],
                        &mut clipPoints[0][0][0],
                    );
                    VectorCopy(
                        &(*dv.offset((*cv).width as isize)).xyz[0],
                        &mut clipPoints[0][1][0],
                    );
                    VectorMA(
                        &clipPoints[0][1][0],
                        MARKER_OFFSET,
                        &(*dv.offset((*cv).width as isize)).normal[0],
                        &mut clipPoints[0][1][0],
                    );
                    VectorCopy(&(*dv.offset(1)).xyz[0], &mut clipPoints[0][2][0]);
                    VectorMA(
                        &clipPoints[0][2][0],
                        MARKER_OFFSET,
                        &(*dv.offset(1)).normal[0],
                        &mut clipPoints[0][2][0],
                    );
                    // check the normal of this triangle
                    VectorSubtract(
                        &clipPoints[0][0][0],
                        &clipPoints[0][1][0],
                        &mut v1[0],
                    );
                    VectorSubtract(
                        &clipPoints[0][2][0],
                        &clipPoints[0][1][0],
                        &mut v2[0],
                    );
                    CrossProduct(&v1[0], &v2[0], &mut normal[0]);
                    VectorNormalizeFast(&mut normal[0]);
                    if DotProduct(&normal[0], &projectionDir[0]) < -0.1 {
                        // add the fragments of this triangle
                        R_AddMarkFragments(
                            numClipPoints,
                            &mut clipPoints,
                            numPlanes,
                            &normals,
                            &dists[0],
                            maxPoints,
                            pointBuffer,
                            maxFragments,
                            fragmentBuffer,
                            &mut returnedPoints,
                            &mut returnedFragments,
                            &mins[0],
                            &maxs[0],
                        );

                        if returnedFragments == maxFragments {
                            return returnedFragments; // not enough space for more fragments
                        }
                    }

                    VectorCopy(&(*dv.offset(1)).xyz[0], &mut clipPoints[0][0][0]);
                    VectorMA(
                        &clipPoints[0][0][0],
                        MARKER_OFFSET,
                        &(*dv.offset(1)).normal[0],
                        &mut clipPoints[0][0][0],
                    );
                    VectorCopy(
                        &(*dv.offset((*cv).width as isize)).xyz[0],
                        &mut clipPoints[0][1][0],
                    );
                    VectorMA(
                        &clipPoints[0][1][0],
                        MARKER_OFFSET,
                        &(*dv.offset((*cv).width as isize)).normal[0],
                        &mut clipPoints[0][1][0],
                    );
                    VectorCopy(
                        &(*dv.offset(((*cv).width + 1) as isize)).xyz[0],
                        &mut clipPoints[0][2][0],
                    );
                    VectorMA(
                        &clipPoints[0][2][0],
                        MARKER_OFFSET,
                        &(*dv.offset(((*cv).width + 1) as isize)).normal[0],
                        &mut clipPoints[0][2][0],
                    );
                    // check the normal of this triangle
                    VectorSubtract(
                        &clipPoints[0][0][0],
                        &clipPoints[0][1][0],
                        &mut v1[0],
                    );
                    VectorSubtract(
                        &clipPoints[0][2][0],
                        &clipPoints[0][1][0],
                        &mut v2[0],
                    );
                    CrossProduct(&v1[0], &v2[0], &mut normal[0]);
                    VectorNormalizeFast(&mut normal[0]);
                    if DotProduct(&normal[0], &projectionDir[0]) < -0.05 {
                        // add the fragments of this triangle
                        R_AddMarkFragments(
                            numClipPoints,
                            &mut clipPoints,
                            numPlanes,
                            &normals,
                            &dists[0],
                            maxPoints,
                            pointBuffer,
                            maxFragments,
                            fragmentBuffer,
                            &mut returnedPoints,
                            &mut returnedFragments,
                            &mins[0],
                            &maxs[0],
                        );

                        if returnedFragments == maxFragments {
                            return returnedFragments; // not enough space for more fragments
                        }
                    }
                    n += 1;
                }
                m += 1;
            }
        } else if *surfaces[i as usize] == SF_FACE {
            surf = surfaces[i as usize] as *mut srfSurfaceFace_t;
            // check the normal of this face
            if DotProduct(&(*surf).plane.normal[0], &projectionDir[0]) > -0.5 {
                i += 1;
                continue;
            }

            /*
            VectorSubtract(clipPoints[0][0], clipPoints[0][1], v1);
            VectorSubtract(clipPoints[0][2], clipPoints[0][1], v2);
            CrossProduct(v1, v2, normal);
            VectorNormalize(normal);
            if (DotProduct(normal, projectionDir) > -0.5) continue;
            */
            #[cfg(feature = "xbox")]
            {
                let const_indexes =
                    (surf as *const u8).offset((*surf).ofsIndices as isize) as *const u8;
                let nextSurfPoint = (NEXT_SURFPOINT_BASE + ((((*surf).flags as usize) & 0x7F) * 2) + (((((*surf).flags as usize) & 0x80) >> 7) * 4)) as isize;
                k = 0;
                while k < (*surf).numIndices as c_int {
                    j = 0;
                    while j < 3 {
                        let const_v = ((*surf).srfPoints as *const u16)
                            .offset((nextSurfPoint * *(const_indexes.offset((k + j) as isize) as *const u8) as isize) as isize);
                        let mut fVec = [0.0f32; 3];
                        Q_CastShort2Float(
                            &mut fVec[0],
                            const_v.offset(0) as *mut i16,
                        );
                        Q_CastShort2Float(
                            &mut fVec[1],
                            const_v.offset(1) as *mut i16,
                        );
                        Q_CastShort2Float(
                            &mut fVec[2],
                            const_v.offset(2) as *mut i16,
                        );
                        VectorMA(
                            &fVec[0],
                            MARKER_OFFSET,
                            &(*surf).plane.normal[0],
                            &mut clipPoints[0][j as usize][0],
                        );
                        j += 1;
                    }
                    // add the fragments of this face
                    R_AddMarkFragments(
                        3,
                        &mut clipPoints,
                        numPlanes,
                        &normals,
                        &dists[0],
                        maxPoints,
                        pointBuffer,
                        maxFragments,
                        fragmentBuffer,
                        &mut returnedPoints,
                        &mut returnedFragments,
                        &mins[0],
                        &maxs[0],
                    );
                    if returnedFragments == maxFragments {
                        return returnedFragments; // not enough space for more fragments
                    }
                    k += 3;
                }
            }
            #[cfg(not(feature = "xbox"))]
            {
                indexes = (surf as *const u8).offset((*surf).ofsIndices as isize) as *const c_int;
                k = 0;
                while k < (*surf).numIndices {
                    j = 0;
                    while j < 3 {
                        v = ((*surf).points as *const f32)
                            .offset(((VERTEXSIZE as c_int) * *indexes.offset((k + j) as isize)) as isize);
                        VectorMA(
                            v,
                            MARKER_OFFSET,
                            &(*surf).plane.normal[0],
                            &mut clipPoints[0][j as usize][0],
                        );
                        j += 1;
                    }
                    // add the fragments of this face
                    R_AddMarkFragments(
                        3,
                        &mut clipPoints,
                        numPlanes,
                        &normals,
                        &dists[0],
                        maxPoints,
                        pointBuffer,
                        maxFragments,
                        fragmentBuffer,
                        &mut returnedPoints,
                        &mut returnedFragments,
                        &mins[0],
                        &maxs[0],
                    );
                    if returnedFragments == maxFragments {
                        return returnedFragments; // not enough space for more fragments
                    }
                    k += 3;
                }
            }
            i += 1;
            continue;
        } else {
            // ignore all other world surfaces
            // might be cool to also project polygons on a triangle soup
            // however this will probably create huge amounts of extra polys
            // even more than the projection onto curves
            i += 1;
            continue;
        }
        i += 1;
    }
    returnedFragments
}
