// tr_marks.c -- polygon projection on the world polygons

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut, copy_nonoverlapping};

// ============================================================================
// Types and constants
// ============================================================================

pub type vec3_t = [f32; 3];
pub type vec_t = f32;
pub type qboolean = c_int;
pub type surfaceType_t = c_int;

const MAX_VERTS_ON_POLY: usize = 64;
const MARKER_OFFSET: f32 = 0.0; // was 1

const SIDE_FRONT: c_int = 0;
const SIDE_BACK: c_int = 1;
const SIDE_ON: c_int = 2;

// Surface type constants
const SF_GRID: c_int = 3;
const SF_TRIANGLES: c_int = 4;
const SF_FACE: c_int = 2;

// Surface flags
const SURF_NOIMPACT: c_int = 0x0010;
const SURF_NOMARKS: c_int = 0x0040;
const CONTENTS_FOG: c_int = 0x10000000;

// ============================================================================
// Struct definitions (needed for this file)
// ============================================================================

#[repr(C)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: c_int,
    pub signbits: c_int,
    pub pad: [u8; 4],
}

#[repr(C)]
pub struct drawVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub lightmap: [f32; 2],
    pub normal: vec3_t,
    pub tangent: [f32; 4],
    pub bitangent: [f32; 3],
    pub color: [u8; 4],
    pub paintColor: [u8; 4],
}

#[repr(C)]
pub struct srfGridMesh_t {
    pub surfaceType: surfaceType_t,
    pub dlightBits: c_int,
    pub meshBounds: [vec3_t; 2],
    pub localOrigin: vec3_t,
    pub meshRadius: f32,
    pub lodOrigin: vec3_t,
    pub lodRadius: f32,
    pub width: c_int,
    pub height: c_int,
    pub widthLodError: *mut f32,
    pub heightLodError: *mut f32,
    pub verts: [drawVert_t; 1],
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    pub surfaceType: surfaceType_t,
    pub plane: cplane_t,
    pub dlightBits: c_int,
    pub numPoints: c_int,
    pub numIndices: c_int,
    pub ofsIndices: c_int,
    pub points: *mut [f32; 6],
}

#[repr(C)]
pub struct srfTriangles_t {
    pub surfaceType: surfaceType_t,
    pub dlightBits: c_int,
    pub bounds: [vec3_t; 2],
    pub numIndexes: c_int,
    pub indexes: *mut c_int,
    pub numVerts: c_int,
    pub verts: *mut drawVert_t,
}

#[repr(C)]
pub struct msurface_t {
    pub viewCount: c_int,
    pub shader: *mut c_void,
    pub fogIndex: c_int,
    pub data: *mut c_void,
}

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

#[repr(C)]
pub struct world_t {
    pub nodes: *mut mnode_t,
    // ... other fields not needed for this file
}

#[repr(C)]
pub struct trGlobals_t {
    pub registered: qboolean,
    pub visCount: c_int,
    pub frameCount: c_int,
    pub sceneCount: c_int,
    pub viewCount: c_int,
    pub frameSceneNum: c_int,
    pub worldMapLoaded: qboolean,
    pub world: *mut world_t,
    // ... other fields not needed for this file
}

#[repr(C)]
pub struct shader_t {
    pub name: [u8; 64],
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
    // ... other fields not needed for this file
}

// ============================================================================
// External declarations
// ============================================================================

extern "C" {
    pub static mut tr: trGlobals_t;

    pub fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> vec_t;
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorNormalize2(in_: *const vec3_t, out: *mut vec3_t) -> vec_t;
    pub fn VectorNormalizeFast(v: *mut vec3_t);
    pub fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t);
    pub fn VectorMA(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t);
    pub fn VectorSubtract(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    pub fn VectorAdd(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    pub fn VectorInverse(v: *mut vec3_t);
    pub fn AddPointToBounds(pt: *const vec3_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn ClearBounds(mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn BoxOnPlaneSide(emins: *const vec3_t, emaxs: *const vec3_t, plane: *const cplane_t) -> c_int;
    pub fn Q_CastShort2Float(dst: *mut f32, src: *mut i16);

    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// ============================================================================
// Functions
// ============================================================================

/*
=============
R_ChopPolyBehindPlane

Out must have space for two more vertexes than in
=============
*/
unsafe fn R_ChopPolyBehindPlane(
    numInPoints: c_int,
    inPoints: *const [vec3_t; MAX_VERTS_ON_POLY],
    numOutPoints: *mut c_int,
    outPoints: *mut [vec3_t; MAX_VERTS_ON_POLY],
    normal: *const vec3_t,
    dist: vec_t,
    epsilon: vec_t,
) {
    let mut dists: [f32; MAX_VERTS_ON_POLY + 4] = [0.0; MAX_VERTS_ON_POLY + 4];
    let mut sides: [c_int; MAX_VERTS_ON_POLY + 4] = [0; MAX_VERTS_ON_POLY + 4];
    let mut counts: [c_int; 3] = [0; 3];
    let mut dot: f32;
    let mut i: c_int;
    let mut j: c_int;
    let mut p1: *const vec3_t;
    let mut p2: *const vec3_t;
    let mut clip: *mut vec3_t;
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
    let inPoints_ref = &*inPoints;
    i = 0;
    while i < numInPoints {
        dot = DotProduct(&inPoints_ref[i as usize], normal);
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
    sides[i as usize] = sides[0];
    dists[i as usize] = dists[0];

    *numOutPoints = 0;

    if counts[0] == 0 {
        return;
    }
    if counts[1] == 0 {
        *numOutPoints = numInPoints;
        memcpy(
            outPoints as *mut c_void,
            inPoints as *const c_void,
            (numInPoints as usize) * mem::size_of::<vec3_t>(),
        );
        return;
    }

    let mut outPoints_mut = &mut *outPoints;
    let numOutPoints_mut = addr_of_mut!(*numOutPoints);

    i = 0;
    while i < numInPoints {
        p1 = &inPoints_ref[i as usize];
        clip = &mut outPoints_mut[*numOutPoints_mut as usize];

        if sides[i as usize] == SIDE_ON {
            VectorCopy(p1, clip);
            *numOutPoints_mut += 1;
            i += 1;
            continue;
        }

        if sides[i as usize] == SIDE_FRONT {
            VectorCopy(p1, clip);
            *numOutPoints_mut += 1;
            clip = &mut outPoints_mut[*numOutPoints_mut as usize];
        }

        if sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
            i += 1;
            continue;
        }

        // generate a split point
        p2 = &inPoints_ref[((i + 1) % numInPoints) as usize];

        d = dists[i as usize] - dists[(i + 1) as usize];
        if d == 0.0 {
            dot = 0.0;
        } else {
            dot = dists[i as usize] / d;
        }

        // clip xyz
        j = 0;
        while j < 3 {
            clip[j as usize] = unsafe {
                *(&p1[j as usize] as *const f32 as *const f32)
                    + dot * (*(&p2[j as usize] as *const f32) - *(&p1[j as usize] as *const f32))
            };
            j += 1;
        }

        *numOutPoints_mut += 1;
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
    mins: *const vec3_t,
    maxs: *const vec3_t,
    list: *mut *mut surfaceType_t,
    listsize: c_int,
    listlength: *mut c_int,
    dir: *const vec3_t,
) {
    let mut s: c_int;
    let mut c: c_int;
    let mut surf: *mut msurface_t;
    let mut mark: *mut *mut msurface_t;

    let mut node = node;

    // do the tail recursion in a loop
    loop {
        let node_ref = &*node;
        if node_ref.contents != -1 {
            break;
        }

        s = BoxOnPlaneSide(mins, maxs, node_ref.plane);
        if s == 1 {
            node = node_ref.children[0];
        } else if s == 2 {
            node = node_ref.children[1];
        } else {
            R_BoxSurfaces_r(
                node_ref.children[0],
                mins,
                maxs,
                list,
                listsize,
                listlength,
                dir,
            );
            node = node_ref.children[1];
        }
    }

    // add the individual surfaces
    let node_ref = &*node;
    mark = node_ref.firstmarksurface;
    c = node_ref.nummarksurfaces;

    while c > 0 {
        c -= 1;

        if *listlength >= listsize {
            break;
        }

        surf = *mark;

        let surf_ref = &*surf;
        let tr_ref = &tr;

        // check if the surface has NOIMPACT or NOMARKS set
        let shader = surf_ref.shader as *const shader_t;
        let shader_ref = &*shader;
        if ((shader_ref.surfaceFlags & (SURF_NOIMPACT | SURF_NOMARKS)) != 0)
            || ((shader_ref.contentFlags & CONTENTS_FOG) != 0)
        {
            (*surf).viewCount = tr_ref.viewCount;
        }
        // extra check for surfaces to avoid list overflows
        else if *(surf_ref.data as *const surfaceType_t) == SF_FACE {
            // the face plane should go through the box
            let face_data = surf_ref.data as *const srfSurfaceFace_t;
            let face_ref = &*face_data;
            s = BoxOnPlaneSide(mins, maxs, &face_ref.plane);
            if s == 1 || s == 2 {
                (*surf).viewCount = tr_ref.viewCount;
            } else if DotProduct(&face_ref.plane.normal, dir) > -0.5 {
                // don't add faces that make sharp angles with the projection direction
                (*surf).viewCount = tr_ref.viewCount;
            }
        } else if *(surf_ref.data as *const surfaceType_t) != SF_GRID
            && *(surf_ref.data as *const surfaceType_t) != SF_TRIANGLES
        {
            (*surf).viewCount = tr_ref.viewCount;
        }

        // check the viewCount because the surface may have
        // already been added if it spans multiple leafs
        if (*surf).viewCount != tr_ref.viewCount {
            (*surf).viewCount = tr_ref.viewCount;
            *list.add(*listlength as usize) = surf_ref.data as *mut surfaceType_t;
            *listlength += 1;
        }

        mark = mark.add(1);
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
    normals: *const vec3_t,
    dists: *const f32,
    maxPoints: c_int,
    pointBuffer: *mut vec3_t,
    maxFragments: c_int,
    fragmentBuffer: *mut markFragment_t,
    returnedPoints: *mut c_int,
    returnedFragments: *mut c_int,
    mins: *const vec3_t,
    maxs: *const vec3_t,
) {
    let mut pingPong: c_int;
    let mut i: c_int;
    let mut mf: *mut markFragment_t;
    let mut numClipPoints_mut = numClipPoints;

    // chop the surface by all the bounding planes of the to be projected polygon
    pingPong = 0;

    i = 0;
    while i < numPlanes {
        let clipPoints_ref = &mut *clipPoints;
        R_ChopPolyBehindPlane(
            numClipPoints_mut,
            &clipPoints_ref[pingPong as usize],
            &mut numClipPoints_mut,
            &mut clipPoints_ref[1 - pingPong as usize],
            &*normals.add(i as usize),
            *dists.add(i as usize),
            0.5,
        );
        pingPong ^= 1;
        if numClipPoints_mut == 0 {
            break;
        }
        i += 1;
    }

    // completely clipped away?
    if numClipPoints_mut == 0 {
        return;
    }

    // add this fragment to the returned list
    if numClipPoints_mut + *returnedPoints > maxPoints {
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

    mf = fragmentBuffer.add(*returnedFragments as usize);
    (*mf).firstPoint = *returnedPoints;
    (*mf).numPoints = numClipPoints_mut;

    let clipPoints_ref = &*clipPoints;
    memcpy(
        pointBuffer.add(*returnedPoints as usize) as *mut c_void,
        &clipPoints_ref[pingPong as usize] as *const _ as *const c_void,
        (numClipPoints_mut as usize) * mem::size_of::<vec3_t>(),
    );

    *returnedPoints += numClipPoints_mut;
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
    projection: vec3_t,
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
    let mut surfaces: [*mut surfaceType_t; 64] = [core::ptr::null_mut(); 64];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut returnedFragments: c_int;
    let mut returnedPoints: c_int;
    let mut normals: [vec3_t; MAX_VERTS_ON_POLY + 2] = [[0.0; 3]; MAX_VERTS_ON_POLY + 2];
    let mut dists: [f32; MAX_VERTS_ON_POLY + 2] = [0.0; MAX_VERTS_ON_POLY + 2];
    let mut clipPoints: [[vec3_t; MAX_VERTS_ON_POLY]; 2] = [
        [[0.0; 3]; MAX_VERTS_ON_POLY],
        [[0.0; 3]; MAX_VERTS_ON_POLY],
    ];
    let mut normal: vec3_t = [0.0; 3];
    let mut projectionDir: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let mut numPoints = numPoints;

    // increment view count for double check prevention
    let tr_ref = &mut tr;
    tr_ref.viewCount += 1;

    //
    VectorNormalize2(&projection, &mut projectionDir);

    // find all the brushes that are to be considered
    ClearBounds(&mut mins, &mut maxs);
    i = 0;
    while i < numPoints {
        let mut temp: vec3_t = [0.0; 3];

        AddPointToBounds(&*points.add(i as usize), &mut mins, &mut maxs);
        VectorAdd(&*points.add(i as usize), &projection, &mut temp);
        AddPointToBounds(&temp, &mut mins, &mut maxs);
        // make sure we get all the leafs (also the one(s) in front of the hit surface)
        VectorMA(&*points.add(i as usize), -20.0, &projectionDir, &mut temp);
        AddPointToBounds(&temp, &mut mins, &mut maxs);
        i += 1;
    }

    if numPoints > MAX_VERTS_ON_POLY as c_int {
        numPoints = MAX_VERTS_ON_POLY as c_int;
    }

    // create the bounding planes for the to be projected polygon
    i = 0;
    while i < numPoints {
        VectorSubtract(
            &*points.add(((i + 1) % numPoints) as usize),
            &*points.add(i as usize),
            &mut v1,
        );
        VectorAdd(&*points.add(i as usize), &projection, &mut v2);
        VectorSubtract(&*points.add(i as usize), &v2, &mut v2);
        CrossProduct(&v1, &v2, &mut normals[i as usize]);
        VectorNormalizeFast(&mut normals[i as usize]);
        dists[i as usize] = DotProduct(&normals[i as usize], &*points.add(i as usize));
        i += 1;
    }

    // add near and far clipping planes for projection
    core::ptr::copy_nonoverlapping(
        &projectionDir,
        &mut normals[numPoints as usize],
        1,
    );
    dists[numPoints as usize] = DotProduct(&normals[numPoints as usize], &*points) - 32.0;
    core::ptr::copy_nonoverlapping(
        &projectionDir,
        &mut normals[(numPoints + 1) as usize],
        1,
    );
    VectorInverse(&mut normals[(numPoints + 1) as usize]);
    dists[(numPoints + 1) as usize] = DotProduct(&normals[(numPoints + 1) as usize], &*points) - 20.0;
    numPlanes = numPoints + 2;

    numsurfaces = 0;
    let tr_ref = &tr;
    R_BoxSurfaces_r(
        tr_ref.world.as_ref().unwrap().nodes,
        &mins,
        &maxs,
        &mut surfaces as *mut _ as *mut *mut surfaceType_t,
        64,
        &mut numsurfaces,
        &projectionDir,
    );

    returnedPoints = 0;
    returnedFragments = 0;

    i = 0;
    while i < numsurfaces {
        if *surfaces[i as usize] == SF_GRID as surfaceType_t {
            let cv = surfaces[i as usize] as *const srfGridMesh_t;
            let cv_ref = &*cv;
            m = 0;
            while m < cv_ref.height - 1 {
                n = 0;
                while n < cv_ref.width - 1 {
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

                    let numClipPoints = 3;

                    let dv = cv_ref.verts.as_ptr().add((m as usize) * (cv_ref.width as usize) + (n as usize));

                    VectorCopy(&(*dv).xyz, &mut clipPoints[0][0]);
                    VectorMA(
                        &mut clipPoints[0][0],
                        MARKER_OFFSET,
                        &(*dv).normal,
                        &mut clipPoints[0][0],
                    );
                    VectorCopy(&(*dv.add(cv_ref.width as usize)).xyz, &mut clipPoints[0][1]);
                    VectorMA(
                        &mut clipPoints[0][1],
                        MARKER_OFFSET,
                        &(*dv.add(cv_ref.width as usize)).normal,
                        &mut clipPoints[0][1],
                    );
                    VectorCopy(&(*dv.add(1)).xyz, &mut clipPoints[0][2]);
                    VectorMA(
                        &mut clipPoints[0][2],
                        MARKER_OFFSET,
                        &(*dv.add(1)).normal,
                        &mut clipPoints[0][2],
                    );

                    // check the normal of this triangle
                    VectorSubtract(&clipPoints[0][0], &clipPoints[0][1], &mut v1);
                    VectorSubtract(&clipPoints[0][2], &clipPoints[0][1], &mut v2);
                    CrossProduct(&v1, &v2, &mut normal);
                    VectorNormalizeFast(&mut normal);
                    if DotProduct(&normal, &projectionDir) < -0.1 {
                        // add the fragments of this triangle
                        R_AddMarkFragments(
                            numClipPoints,
                            &mut clipPoints,
                            numPlanes,
                            &normals[0],
                            &dists[0],
                            maxPoints,
                            pointBuffer,
                            maxFragments,
                            fragmentBuffer,
                            &mut returnedPoints,
                            &mut returnedFragments,
                            &mins,
                            &maxs,
                        );

                        if returnedFragments == maxFragments {
                            return returnedFragments; // not enough space for more fragments
                        }
                    }

                    VectorCopy(&(*dv.add(1)).xyz, &mut clipPoints[0][0]);
                    VectorMA(
                        &mut clipPoints[0][0],
                        MARKER_OFFSET,
                        &(*dv.add(1)).normal,
                        &mut clipPoints[0][0],
                    );
                    VectorCopy(&(*dv.add(cv_ref.width as usize)).xyz, &mut clipPoints[0][1]);
                    VectorMA(
                        &mut clipPoints[0][1],
                        MARKER_OFFSET,
                        &(*dv.add(cv_ref.width as usize)).normal,
                        &mut clipPoints[0][1],
                    );
                    VectorCopy(
                        &(*dv.add(cv_ref.width as usize + 1)).xyz,
                        &mut clipPoints[0][2],
                    );
                    VectorMA(
                        &mut clipPoints[0][2],
                        MARKER_OFFSET,
                        &(*dv.add(cv_ref.width as usize + 1)).normal,
                        &mut clipPoints[0][2],
                    );

                    // check the normal of this triangle
                    VectorSubtract(&clipPoints[0][0], &clipPoints[0][1], &mut v1);
                    VectorSubtract(&clipPoints[0][2], &clipPoints[0][1], &mut v2);
                    CrossProduct(&v1, &v2, &mut normal);
                    VectorNormalizeFast(&mut normal);
                    if DotProduct(&normal, &projectionDir) < -0.05 {
                        // add the fragments of this triangle
                        R_AddMarkFragments(
                            numClipPoints,
                            &mut clipPoints,
                            numPlanes,
                            &normals[0],
                            &dists[0],
                            maxPoints,
                            pointBuffer,
                            maxFragments,
                            fragmentBuffer,
                            &mut returnedPoints,
                            &mut returnedFragments,
                            &mins,
                            &maxs,
                        );

                        if returnedFragments == maxFragments {
                            return returnedFragments; // not enough space for more fragments
                        }
                    }

                    n += 1;
                }
                m += 1;
            }
        } else if *surfaces[i as usize] == SF_FACE as surfaceType_t {
            let surf = surfaces[i as usize] as *const srfSurfaceFace_t;
            let surf_ref = &*surf;

            // check the normal of this face
            if DotProduct(&surf_ref.plane.normal, &projectionDir) > -0.5 {
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

            let indexes = (surf as *mut u8).add(surf_ref.ofsIndices as usize) as *const c_int;

            k = 0;
            while k < surf_ref.numIndices {
                j = 0;
                while j < 3 {
                    let v = surf_ref.points.add(*indexes.add((k + j) as usize) as usize);
                    VectorMA(
                        &(*v)[0..3] as *const _ as *const vec3_t,
                        MARKER_OFFSET,
                        &surf_ref.plane.normal,
                        &mut clipPoints[0][j as usize],
                    );
                    j += 1;
                }

                // add the fragments of this face
                R_AddMarkFragments(
                    3,
                    &mut clipPoints,
                    numPlanes,
                    &normals[0],
                    &dists[0],
                    maxPoints,
                    pointBuffer,
                    maxFragments,
                    fragmentBuffer,
                    &mut returnedPoints,
                    &mut returnedFragments,
                    &mins,
                    &maxs,
                );

                if returnedFragments == maxFragments {
                    return returnedFragments; // not enough space for more fragments
                }

                k += 3;
            }
            i += 1;
            continue;
        } else if *surfaces[i as usize] == SF_TRIANGLES as surfaceType_t {
            let surf = surfaces[i as usize] as *const srfTriangles_t;
            let surf_ref = &*surf;

            k = 0;
            while k < surf_ref.numIndexes {
                let i1 = *surf_ref.indexes.add(k as usize);
                let i2 = *surf_ref.indexes.add((k + 1) as usize);
                let i3 = *surf_ref.indexes.add((k + 2) as usize);

                VectorSubtract(
                    &(*surf_ref.verts.add(i1 as usize)).xyz,
                    &(*surf_ref.verts.add(i2 as usize)).xyz,
                    &mut v1,
                );
                VectorSubtract(
                    &(*surf_ref.verts.add(i3 as usize)).xyz,
                    &(*surf_ref.verts.add(i2 as usize)).xyz,
                    &mut v2,
                );
                CrossProduct(&v1, &v2, &mut normal);
                VectorNormalizeFast(&mut normal);

                // check the normal of this triangle
                if DotProduct(&normal, &projectionDir) < -0.1 {
                    VectorMA(
                        &(*surf_ref.verts.add(i1 as usize)).xyz,
                        MARKER_OFFSET,
                        &normal,
                        &mut clipPoints[0][0],
                    );
                    VectorMA(
                        &(*surf_ref.verts.add(i2 as usize)).xyz,
                        MARKER_OFFSET,
                        &normal,
                        &mut clipPoints[0][1],
                    );
                    VectorMA(
                        &(*surf_ref.verts.add(i3 as usize)).xyz,
                        MARKER_OFFSET,
                        &normal,
                        &mut clipPoints[0][2],
                    );

                    // add the fragments of this triangle
                    R_AddMarkFragments(
                        3,
                        &mut clipPoints,
                        numPlanes,
                        &normals[0],
                        &dists[0],
                        maxPoints,
                        pointBuffer,
                        maxFragments,
                        fragmentBuffer,
                        &mut returnedPoints,
                        &mut returnedFragments,
                        &mins,
                        &maxs,
                    );

                    if returnedFragments == maxFragments {
                        return returnedFragments; // not enough space for more fragments
                    }
                }

                k += 3;
            }
        } else {
            // ignore all other world surfaces
            // might be cool to also project polygons on a triangle soup
            // however this will probably create huge amounts of extra polys
            // even more than the projection onto curves
        }

        i += 1;
    }

    returnedFragments
}
