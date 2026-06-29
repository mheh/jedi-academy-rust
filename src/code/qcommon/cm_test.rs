#![allow(non_snake_case)]

use core::ptr::{addr_of, addr_of_mut};

// Stubs for external types and functions from cm_local.h and elsewhere
use core::ffi::c_int;

type vec3_t = [f32; 3];
type clipHandle_t = i32;
type qboolean = i32;
type byte = u8;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const BOX_MODEL_HANDLE: clipHandle_t = 0;
const CONTENTS_TERRAIN: i32 = 0x8000000;  // stub value
const MAX_MAP_AREA_BYTES: usize = 32;
const ERR_DROP: i32 = 0;

// Stub types for structs defined elsewhere
#[repr(C)]
struct cNode_t {
    plane: *mut cplane_t,
    children: [i32; 2],
}

#[repr(C)]
struct cplane_t {
    normal: [f32; 3],
    dist: f32,
    type_: u8,
    _pad: [u8; 3],
}

#[repr(C)]
struct cLeaf_t {
    cluster: i32,
    area: i32,
    firstLeafBrush: i32,
    numLeafBrushes: i32,
}

#[repr(C)]
struct cbrush_t {
    sides: *mut cbrushside_t,
    numsides: i32,
    contents: i32,
    bounds: [[f32; 3]; 2],
    checkcount: i32,
}

#[repr(C)]
struct cbrushside_t {
    plane: *mut cplane_t,
    planeNum: i32,
}

#[repr(C)]
struct cArea_t {
    floodnum: i32,
    floodvalid: i32,
}

#[repr(C)]
struct cmodel_t {
    leaf: cLeaf_t,
}

#[repr(C)]
struct clipMap_t {
    name: [c_int; 128],
    numShaders: i32,
    numPlanes: i32,
    numNodes: i32,
    numLeafs: i32,
    numLeafBrushes: i32,
    numBrushes: i32,
    numBrushSides: i32,
    numAreas: i32,
    numVisibility: i32,
    clusterBytes: i32,
    numClusters: i32,
    numModels: i32,
    numEntityChars: i32,

    planes: *mut cplane_t,
    nodes: *mut cNode_t,
    leafs: *mut cLeaf_t,
    leafbrushes: *mut i32,
    brushes: *mut cbrush_t,
    brushsides: *mut cbrushside_t,
    visibility: *mut byte,
    entities: *mut c_int,

    areas: *mut cArea_t,
    areaPortals: *mut i32,

    floodvalid: i32,
    checkcount: i32,

    vised: qboolean,
    landScape: *mut core::ffi::c_void,
}

#[repr(C)]
struct leafList_t {
    count: i32,
    maxcount: i32,
    list: *mut i32,
    bounds: [[f32; 3]; 2],
    lastLeaf: i32,
    overflowed: qboolean,
    storeLeafs: Option<fn(*mut leafList_t, i32)>,
}

// Stub global for cmg
static mut cmg: clipMap_t = clipMap_t {
    name: [0; 128],
    numShaders: 0,
    numPlanes: 0,
    numNodes: 0,
    numLeafs: 0,
    numLeafBrushes: 0,
    numBrushes: 0,
    numBrushSides: 0,
    numAreas: 0,
    numVisibility: 0,
    clusterBytes: 0,
    numClusters: 0,
    numModels: 0,
    numEntityChars: 0,
    planes: core::ptr::null_mut(),
    nodes: core::ptr::null_mut(),
    leafs: core::ptr::null_mut(),
    leafbrushes: core::ptr::null_mut(),
    brushes: core::ptr::null_mut(),
    brushsides: core::ptr::null_mut(),
    visibility: core::ptr::null_mut(),
    entities: core::ptr::null_mut(),
    areas: core::ptr::null_mut(),
    areaPortals: core::ptr::null_mut(),
    floodvalid: 0,
    checkcount: 0,
    vised: 0,
    landScape: core::ptr::null_mut(),
};

// Stub global for tr (Xbox version)
#[cfg(feature = "xbox")]
struct trGlobals_t {
    world: *mut core::ffi::c_void,
}

#[cfg(feature = "xbox")]
static mut tr: trGlobals_t = trGlobals_t {
    world: core::ptr::null_mut(),
};

// Stub globals
static mut c_pointcontents: i32 = 0;

// Stub cvar
struct cvar_t {
    integer: i32,
}

static mut cm_noAreas: cvar_t = cvar_t { integer: 0 };

// External stub functions
fn DotProduct(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn VectorCopy(src: &[f32; 3], dst: &mut [f32; 3]) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

fn VectorSubtract(a: &[f32; 3], b: &[f32; 3], c: &mut [f32; 3]) {
    c[0] = a[0] - b[0];
    c[1] = a[1] - b[1];
    c[2] = a[2] - b[2];
}

fn AngleVectors(angles: &[f32; 3], forward: &mut [f32; 3], right: &mut [f32; 3], up: &mut [f32; 3]) {
    // Stub implementation
    forward[0] = 0.0;
    forward[1] = 0.0;
    forward[2] = 0.0;
    right[0] = 0.0;
    right[1] = 0.0;
    right[2] = 0.0;
    up[0] = 0.0;
    up[1] = 0.0;
    up[2] = 0.0;
}

fn BoxOnPlaneSide(mins: &[f32; 3], maxs: &[f32; 3], plane: *const cplane_t) -> i32 {
    // Stub implementation
    0
}

fn CM_ClipHandleToModel(handle: clipHandle_t, local: *mut *mut clipMap_t) -> *mut cmodel_t {
    // Stub implementation
    core::ptr::null_mut()
}

fn CM_LeafArea(leaf: i32) -> i32 {
    // Stub implementation
    0
}

fn Com_Error(code: i32, msg: &str) {
    // Stub implementation
}

fn Com_Printf(msg: &str) {
    // Stub implementation
}

// CPoint class from original C++
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CPoint {
    x: f32,
    y: f32,
    z: f32,
}

impl CPoint {
    fn new(x: f32, y: f32, z: f32) -> Self {
        CPoint { x, y, z }
    }
}

// operator== from original: bool operator== (const CPoint& _P) const {return((x==_P.x)&&(y==_P.y)&&(z==_P.z));}
impl PartialEq for CPoint {
    fn eq(&self, other: &CPoint) -> bool {
        (self.x == other.x) && (self.y == other.y) && (self.z == other.z)
    }
}

/*
class CPointComparator
{
public:
	bool operator()(const CPoint& _A,const CPoint& _B) const {return((_A.x==_B.x)&&(_A.y==_B.y)&&(_A.z==_B.z));}
};
*/

// Fixed memory version of pointToLeaf that doesn't use STL
// cuts down on memory fragmentation
#[repr(C)]
struct PointAndLeaf {
    // Default constructor for array construction below
    point: CPoint,
    leaf: i32,
}

impl PointAndLeaf {
    const fn new() -> Self {
        PointAndLeaf {
            point: CPoint { x: 0.0, y: 0.0, z: 0.0 },
            leaf: 0,
        }
    }
}

// I think it is a patholoically bad idea to do a 64 item linear search for a cache,
// so I reduced this to something more manageable.
// hopefully getting rid of water checks on maps with no water will leave us less
// reliant on this cache. -gwg

const MAX_POINTS_TO_LEAVES: usize = 16;

static mut pointToLeaf: [PointAndLeaf; MAX_POINTS_TO_LEAVES] = [PointAndLeaf::new(); MAX_POINTS_TO_LEAVES];
static mut oldestPointToLeaf: i32 = 0;
static mut sizePointToLeaf: i32 = 0;

//static hlist<pair<CPoint,int> >				pointToLeaf;
//static hlist<pair<CPoint,int> >				pointToContents;

fn CM_CleanLeafCache() {
    unsafe {
        oldestPointToLeaf = 0;
        sizePointToLeaf = 0;
    }
    //	pointToLeaf.clear();
    // #if 0 // VVFIXME
    // 	hlist<pair<CPoint,int> >::iterator l;
    // 	for(l=pointToLeaf.begin();l!=pointToLeaf.end();l++)
    // 	{
    // 		pointToLeaf.erase(l);
    // 	}
    // #endif
    /*
        for(l=pointToContents.begin();l!=pointToContents.end();l++)
        {
            pointToContents.erase(l);
        }
    */
}

/*
==================
CM_PointLeafnum_r

==================
*/
fn CM_PointLeafnum_r(p: &[f32; 3], num: i32, local: *mut clipMap_t) -> i32 {
    let mut d: f32;
    let mut node: *mut cNode_t;
    let mut plane: *mut cplane_t;

    #[cfg(feature = "xbox")]
    {
        unsafe {
            if ((*addr_of!(tr)).world as *const _).is_null() {
                return 0;
            }
        }
    }

    let mut num_mut = num;
    while num_mut >= 0 {
        unsafe {
            node = (*local).nodes.add(num_mut as usize);
            #[cfg(feature = "xbox")]
            {
                plane = (*addr_of!(cmg)).planes.add((*(*addr_of!(tr)).world as *const core::ffi::c_void) as *const i32 as usize);
            }
            #[cfg(not(feature = "xbox"))]
            {
                plane = (*node).plane;
            }

            if (*plane).type_ < 3 {
                d = p[(*plane).type_ as usize] - (*plane).dist;
            } else {
                d = DotProduct(p, &(*plane).normal) - (*plane).dist;
            }
            if d < 0.0 {
                num_mut = (*node).children[1];
            } else {
                num_mut = (*node).children[0];
            }
        }
    }

    unsafe {
        *addr_of_mut!(c_pointcontents) += 1;		// optimize counter
    }

    return -1 - num_mut;
}

fn CM_PointLeafnum(p: &[f32; 3]) -> i32 {
    unsafe {
        if (*addr_of!(cmg)).numNodes == 0 {	// map not loaded
            return 0;
        }
        return CM_PointLeafnum_r(p, 0, addr_of_mut!(cmg));
    }
}


/*
======================================================================

LEAF LISTING

======================================================================
*/


fn CM_StoreLeafs(ll: *mut leafList_t, nodenum: i32) {
    let mut leafNum: i32;

    leafNum = -1 - nodenum;

    // store the lastLeaf even if the list is overflowed
    unsafe {
        if (*addr_of!(cmg)).leafs[leafNum as usize].cluster != -1 {
            (*ll).lastLeaf = leafNum;
        }

        if (*ll).count >= (*ll).maxcount {
            (*ll).overflowed = qtrue;
            return;
        }
        *(*ll).list.add((*ll).count as usize) = leafNum;
        (*ll).count += 1;
    }
}

fn CM_StoreBrushes(ll: *mut leafList_t, nodenum: i32) {
    let mut i: i32;
    let mut k: i32;
    let mut leafnum: i32;
    let mut brushnum: i32;
    let mut leaf: *mut cLeaf_t;
    let mut b: *mut cbrush_t;

    leafnum = -1 - nodenum;

    unsafe {
        leaf = &mut (*addr_of_mut!(cmg)).leafs[leafnum as usize];

        for k in 0..(*leaf).numLeafBrushes {
            brushnum = (*addr_of!(cmg)).leafbrushes[(*leaf).firstLeafBrush as usize + k as usize];
            b = &mut (*addr_of_mut!(cmg)).brushes[brushnum as usize];
            if (*b).checkcount == (*addr_of!(cmg)).checkcount {
                continue;	// already checked this brush in another leaf
            }
            (*b).checkcount = (*addr_of!(cmg)).checkcount;
            i = 0;
            while i < 3 {
                if (*b).bounds[0][i as usize] >= (*ll).bounds[1][i as usize] || (*b).bounds[1][i as usize] <= (*ll).bounds[0][i as usize] {
                    break;
                }
                i += 1;
            }
            if i != 3 {
                continue;
            }
            if (*ll).count >= (*ll).maxcount {
                (*ll).overflowed = qtrue;
                return;
            }
            let brush_ptr_ptr = (*ll).list as *mut *mut cbrush_t;
            *brush_ptr_ptr.add((*ll).count as usize) = b;
            (*ll).count += 1;
        }
    }
    // #if 0
    // 	// store patches?
    // 	for ( k = 0 ; k < leaf->numLeafSurfaces ; k++ ) {
    // 		patch = cmg.surfaces[ cmg.leafsurfaces[ leaf->firstleafsurface + k ] ];
    // 		if ( !patch ) {
    // 			continue;
    // 		}
    // 	}
    // #endif
}

/*
=============
CM_BoxLeafnums

Fills in a list of all the leafs touched
=============
*/
fn CM_BoxLeafnums_r(ll: *mut leafList_t, nodenum: i32) {
    let mut plane: *mut cplane_t;
    let mut node: *mut cNode_t;
    let mut s: i32;

    #[cfg(feature = "xbox")]
    {
        unsafe {
            if ((*addr_of!(tr)).world as *const _).is_null() {
                return;
            }
        }
    }

    let mut nodenum_mut = nodenum;
    loop {
        unsafe {
            if nodenum_mut < 0 {
                if let Some(store_leafs) = (*ll).storeLeafs {
                    store_leafs(ll, nodenum_mut);
                }
                return;
            }

            node = &mut (*addr_of_mut!(cmg)).nodes[nodenum_mut as usize];

            #[cfg(feature = "xbox")]
            {
                plane = (*addr_of!(cmg)).planes.add((*(*addr_of!(tr)).world as *const core::ffi::c_void) as *const i32 as usize);
            }
            #[cfg(not(feature = "xbox"))]
            {
                plane = (*node).plane;
            }
            s = BoxOnPlaneSide(&(*ll).bounds[0], &(*ll).bounds[1], plane);
            if s == 1 {
                nodenum_mut = (*node).children[0];
            } else if s == 2 {
                nodenum_mut = (*node).children[1];
            } else {
                // go down both
                CM_BoxLeafnums_r(ll, (*node).children[0]);
                nodenum_mut = (*node).children[1];
            }
        }
    }
}

/*
==================
CM_BoxLeafnums
==================
*/
fn CM_BoxLeafnums(mins: &[f32; 3], maxs: &[f32; 3], boxList: *mut i32, listsize: i32, lastLeaf: *mut i32) -> i32 {
    let mut ll: leafList_t;

    unsafe {
        (*addr_of_mut!(cmg)).checkcount += 1;

        ll = leafList_t {
            count: 0,
            maxcount: listsize,
            list: boxList,
            bounds: [[0.0; 3]; 2],
            lastLeaf: 0,
            overflowed: qfalse,
            storeLeafs: Some(CM_StoreLeafs),
        };

        VectorCopy(mins, &mut ll.bounds[0]);
        VectorCopy(maxs, &mut ll.bounds[1]);

        CM_BoxLeafnums_r(&mut ll, 0);

        *lastLeaf = ll.lastLeaf;
        return ll.count;
    }
}

/*
==================
CM_BoxBrushes
==================
*/
fn CM_BoxBrushes(mins: &[f32; 3], maxs: &[f32; 3], boxlist: *mut *mut cbrush_t, listsize: i32) -> i32 {
    let mut ll: leafList_t;

    unsafe {
        (*addr_of_mut!(cmg)).checkcount += 1;

        ll = leafList_t {
            count: 0,
            maxcount: listsize,
            list: boxlist as *mut i32,
            bounds: [[0.0; 3]; 2],
            lastLeaf: 0,
            overflowed: qfalse,
            storeLeafs: Some(CM_StoreBrushes),
        };

        VectorCopy(mins, &mut ll.bounds[0]);
        VectorCopy(maxs, &mut ll.bounds[1]);

        CM_BoxLeafnums_r(&mut ll, 0);

        return ll.count;
    }
}


//====================================================================


/*
==================
CM_PointContents

==================
*/

// #if 1

fn CM_PointContents(p: &[f32; 3], model: clipHandle_t) -> i32 {
    let mut leafnum: i32 = 0;
    let mut i: i32;
    let mut k: i32;
    let mut brushnum: i32;
    let mut leaf: *mut cLeaf_t;
    let mut b: *mut cbrush_t;
    let mut contents: i32;
    let mut d: f32;
    let mut clipm: *mut cmodel_t;
    let mut local: *mut clipMap_t;

    unsafe {
        if (*addr_of!(cmg)).numNodes == 0 {	// map not loaded
            return 0;
        }

        if model != 0 {
            clipm = CM_ClipHandleToModel(model, &mut local);
            leaf = &mut (*clipm).leaf as *mut cLeaf_t;
        }
        else
        {
            local = addr_of_mut!(cmg);
            let pt = CPoint::new(p[0], p[1], p[2]);
            /*		map<CPoint,int,CPointComparator>::iterator l=pointToLeaf.find(pt);
                if(l!=pointToLeaf.end())
                {
                    leafnum=(*l).second;
                }
                else
                {
                    if(pointToLeaf.size()>=64)
                    {
                        pointToLeaf.clear();
                        Com_Printf("Cleared cache\n");
                    }
                    leafnum=CM_PointLeafnum_r(p, 0);
                    pointToLeaf[pt]=leafnum;
                }*/

            let mut l: i32 = 0;
            while l < (*addr_of!(sizePointToLeaf)) {
                if pointToLeaf[l as usize].point == pt {
                    leafnum = pointToLeaf[l as usize].leaf;
                    break;
                }
                l += 1;
            }

            if l == (*addr_of!(sizePointToLeaf)) {
                // Didn't find it
                if (*addr_of!(sizePointToLeaf)) < MAX_POINTS_TO_LEAVES as i32 {
                    // We're adding a new one, rather than replacing
                    *addr_of_mut!(sizePointToLeaf) += 1;
                } else {
                    // Put it in the "oldest" slot
                    l = *addr_of!(oldestPointToLeaf);
                    *addr_of_mut!(oldestPointToLeaf) += 1;
                    *addr_of_mut!(oldestPointToLeaf) &= (MAX_POINTS_TO_LEAVES as i32 - 1);
                }

                leafnum = CM_PointLeafnum_r(p, 0, local);
                pointToLeaf[l as usize].leaf = leafnum;
                pointToLeaf[l as usize].point = pt;
            }

            /*
            hlist<pair<CPoint,int> >::iterator l;
            for(l=pointToLeaf.begin();l!=pointToLeaf.end();l++)
            {
                if((*l).first==pt)
                {
                    leafnum=(*l).second;
                    break;
                }
            }
            if(l==pointToLeaf.end())
            {
                if(pointToLeaf.size()>=64)
                {
                    pointToLeaf.pop_back();
                }
                leafnum=CM_PointLeafnum_r(p, 0, local);
                pointToLeaf.push_front(pair<CPoint,int>(pt,leafnum));
            }
            */
            leaf = &mut (*local).leafs[leafnum as usize];
        }

        contents = 0;
        for k in 0..(*leaf).numLeafBrushes {
            brushnum = (*local).leafbrushes[(*leaf).firstLeafBrush as usize + k as usize];
            b = &mut (*local).brushes[brushnum as usize];

            // see if the point is in the brush
            i = 0;
            while i < (*b).numsides {
                #[cfg(feature = "xbox")]
                {
                    d = DotProduct(p, &(*(*addr_of!(cmg)).planes.add((*b).sides[i as usize].planeNum as usize)).normal);
                }
                #[cfg(not(feature = "xbox"))]
                {
                    d = DotProduct(p, &(*(*b).sides[i as usize].plane).normal);
                }
                // FIXME test for Cash
                //			if ( d >= b->sides[i].plane->dist ) {
                #[cfg(feature = "xbox")]
                {
                    if d > (*(*addr_of!(cmg)).planes.add((*b).sides[i as usize].planeNum as usize)).dist {
                        break;
                    }
                }
                #[cfg(not(feature = "xbox"))]
                {
                    if d > (*(*b).sides[i as usize].plane).dist {
                        break;
                    }
                }
                i += 1;
            }

            if i == (*b).numsides {
                contents |= (*b).contents;
                #[cfg(not(feature = "xbox"))]
                {
                    // Removing terrain from Xbox
                    if (*addr_of!(cmg)).landScape as *const _ != core::ptr::null() && (contents & CONTENTS_TERRAIN) != 0 {
                        if p[2] < 0.0 {  // GetWaterHeight() stub
                            contents |= 0;  // GetWaterContents() stub
                        }
                    }
                }
            }
        }

        return contents;
    }
}

// #else

// fn CM_PointContents( const vec3_t p, clipHandle_t model ) {
// 	int			leafnum=0;
// 	int			i, k;
// 	int			brushnum;
// 	cLeaf_t		*leaf;
// 	cbrush_t	*b;
// 	int			contents;
// 	float		d;
// 	cmodel_t	*clipm;

// 	if (!cmg.numNodes) {	// map not loaded
// 		return 0;
// 	}

// 	CPoint pt(p[0],p[1],p[2]);
// 	if ( model )
// 	{
// 		clipm = CM_ClipHandleToModel( model );
// 		leaf = &clipm->leaf;
// 	}
// 	else
// 	{
// 		hlist<pair<CPoint,int> >::iterator l;
// 		for(l=pointToContents.begin();l!=pointToContents.end();l++)
// 		{
// 			if((*l).first==pt)
// 			{
// 				// Breakout early.
// 				return((*l).second);
// 			}
// 		}

// 		leafnum=CM_PointLeafnum_r(p, 0);
// 		leaf = &cmg.leafs[leafnum];
// 	}

// 	contents = 0;
// 	for (k=0 ; k<leaf->numLeafBrushes ; k++)
// 	{
// 		brushnum = cmg.leafbrushes[leaf->firstLeafBrush+k];
// 		b = &cmg.brushes[brushnum];

// 		// see if the point is in the brush
// 		for ( i = 0 ; i < b->numsides ; i++ )
// 		{
// 			d = DotProduct( p, b->sides[i].plane->normal );
// 			// FIXME test for Cash
// //			if ( d >= b->sides[i].plane->dist ) {
// 			if ( d > b->sides[i].plane->dist )
// 			{
// 				break;
// 			}
// 		}

// 		if ( i == b->numsides )
// 		{
// 			contents |= b->contents;
// 		}
// 	}

// 	// Cache the result for next time.
// 	if(!model)
// 	{
// 		if(pointToContents.size()>=64)
// 		{
// 			pointToContents.pop_back();
// 		}
// 		pointToContents.push_front(pair<CPoint,int>(pt,contents));
// 	}

// 	return contents;
// }

// #endif

/*
==================
CM_TransformedPointContents

Handles offseting and rotation of the end points for moving and
rotating entities
==================
*/
fn CM_TransformedPointContents(p: &[f32; 3], model: clipHandle_t, origin: &[f32; 3], angles: &[f32; 3]) -> i32 {
    let mut p_l: [f32; 3] = [0.0; 3];
    let mut temp: [f32; 3] = [0.0; 3];
    let mut forward: [f32; 3] = [0.0; 3];
    let mut right: [f32; 3] = [0.0; 3];
    let mut up: [f32; 3] = [0.0; 3];

    // subtract origin offset
    VectorSubtract(p, origin, &mut p_l);

    // rotate start and end into the models frame of reference
    if model != BOX_MODEL_HANDLE &&
    (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0) {
        AngleVectors(angles, &mut forward, &mut right, &mut up);

        VectorCopy(&p_l, &mut temp);
        p_l[0] = DotProduct(&temp, &forward);
        p_l[1] = -DotProduct(&temp, &right);
        p_l[2] = DotProduct(&temp, &up);
    }

    return CM_PointContents(&p_l, model);
}



/*
===============================================================================

PVS

===============================================================================
*/

#[cfg(feature = "xbox")]
fn CM_ClusterPVS(cluster: i32) -> *const byte {
    unsafe {
        if cluster < 0 || cluster >= (*addr_of!(cmg)).numClusters || (*addr_of!(cmg)).vised == 0 {
            return core::ptr::null();
        }

        // Decompress stub
        return (*addr_of!(cmg)).visibility.add((cluster as usize) * (*addr_of!(cmg)).clusterBytes as usize);
    }
}

#[cfg(not(feature = "xbox"))]
fn CM_ClusterPVS(cluster: i32) -> *mut byte {
    unsafe {
        if cluster < 0 || cluster >= (*addr_of!(cmg)).numClusters || (*addr_of!(cmg)).vised == 0 {
            return (*addr_of_mut!(cmg)).visibility;
        }

        return (*addr_of_mut!(cmg)).visibility.add((cluster as usize) * (*addr_of!(cmg)).clusterBytes as usize);
    }
}


/*
===============================================================================

AREAPORTALS

===============================================================================
*/

#[cfg(feature = "xbox")]
fn CM_FloodArea_r(areaNum: i32, floodnum: i32) {
    let mut i: i32;
    let mut area: *mut cArea_t;
    let mut con: *mut i32;

    unsafe {
        area = &mut (*addr_of_mut!(cmg)).areas[areaNum as usize];

        if (*area).floodvalid == (*addr_of!(cmg)).floodvalid {
            if (*area).floodnum == floodnum {
                return;
            }
            Com_Error(ERR_DROP, "FloodArea_r: reflooded");
        }

        (*area).floodnum = floodnum;
        (*area).floodvalid = (*addr_of!(cmg)).floodvalid;
        con = (*addr_of_mut!(cmg)).areaPortals.add(areaNum as usize * (*addr_of!(cmg)).numAreas as usize);
        for i in 0..(*addr_of!(cmg)).numAreas {
            if *con.add(i as usize) > 0 {
                CM_FloodArea_r(i, floodnum);
            }
        }
    }
}

#[cfg(not(feature = "xbox"))]
fn CM_FloodArea_r(areaNum: i32, floodnum: i32) {
    let mut i: i32;
    let mut area: *mut cArea_t;
    let mut con: *mut i32;

    unsafe {
        area = &mut (*addr_of_mut!(cmg)).areas[areaNum as usize];

        if (*area).floodvalid == (*addr_of!(cmg)).floodvalid {
            if (*area).floodnum == floodnum {
                return;
            }
            Com_Error(ERR_DROP, "FloodArea_r: reflooded");
        }

        (*area).floodnum = floodnum;
        (*area).floodvalid = (*addr_of!(cmg)).floodvalid;
        con = (*addr_of_mut!(cmg)).areaPortals.add(areaNum as usize * (*addr_of!(cmg)).numAreas as usize);
        for i in 0..(*addr_of!(cmg)).numAreas {
            if *con.add(i as usize) > 0 {
                CM_FloodArea_r(i, floodnum);
            }
        }
    }
}

/*
====================
CM_FloodAreaConnections

====================
*/
#[cfg(feature = "xbox")]
fn CM_FloodAreaConnections() {
    let mut i: i32;
    let mut area: *mut cArea_t;
    let mut floodnum: i32;

    unsafe {
        // all current floods are now invalid
        (*addr_of_mut!(cmg)).floodvalid += 1;
        floodnum = 0;

        for i in 0..(*addr_of!(cmg)).numAreas {
            area = &mut (*addr_of_mut!(cmg)).areas[i as usize];
            if (*area).floodvalid == (*addr_of!(cmg)).floodvalid {
                continue;		// already flooded into
            }
            floodnum += 1;
            CM_FloodArea_r(i, floodnum);
        }
    }
}

#[cfg(not(feature = "xbox"))]
fn CM_FloodAreaConnections() {
    let mut i: i32;
    let mut area: *mut cArea_t;
    let mut floodnum: i32;

    unsafe {
        // all current floods are now invalid
        (*addr_of_mut!(cmg)).floodvalid += 1;
        floodnum = 0;

        for i in 0..(*addr_of!(cmg)).numAreas {
            area = &mut (*addr_of_mut!(cmg)).areas[i as usize];
            if (*area).floodvalid == (*addr_of!(cmg)).floodvalid {
                continue;		// already flooded into
            }
            floodnum += 1;
            CM_FloodArea_r(i, floodnum);
        }
    }
}

/*
====================
CM_AdjustAreaPortalState

====================
*/
fn CM_AdjustAreaPortalState(area1: i32, area2: i32, open: qboolean) {
    unsafe {
        if area1 < 0 || area2 < 0 {
            return;
        }

        if area1 >= (*addr_of!(cmg)).numAreas || area2 >= (*addr_of!(cmg)).numAreas {
            Com_Error(ERR_DROP, "CM_ChangeAreaPortalState: bad area number");
        }

        if open != 0 {
            *(*addr_of_mut!(cmg)).areaPortals.add((area1 as usize) * (*addr_of!(cmg)).numAreas as usize + area2 as usize) += 1;
            *(*addr_of_mut!(cmg)).areaPortals.add((area2 as usize) * (*addr_of!(cmg)).numAreas as usize + area1 as usize) += 1;
        } else {
            *(*addr_of_mut!(cmg)).areaPortals.add((area1 as usize) * (*addr_of!(cmg)).numAreas as usize + area2 as usize) -= 1;
            *(*addr_of_mut!(cmg)).areaPortals.add((area2 as usize) * (*addr_of!(cmg)).numAreas as usize + area1 as usize) -= 1;
            if *(*addr_of_mut!(cmg)).areaPortals.add((area2 as usize) * (*addr_of!(cmg)).numAreas as usize + area1 as usize) < 0 {
                Com_Error(ERR_DROP, "CM_AdjustAreaPortalState: negative reference count");
            }
        }

        CM_FloodAreaConnections();
    }
}

/*
====================
CM_AreasConnected

====================
*/
fn CM_AreasConnected(area1: i32, area2: i32) -> qboolean {
    #[cfg(not(feature = "bspc"))]
    {
        unsafe {
            if (*addr_of!(cm_noAreas)).integer != 0 {
                return qtrue;
            }
        }
    }

    if area1 < 0 || area2 < 0 {
        return qfalse;
    }

    unsafe {
        if area1 >= (*addr_of!(cmg)).numAreas || area2 >= (*addr_of!(cmg)).numAreas {
            Com_Error(ERR_DROP, "area >= cmg.numAreas");
        }

        if (*addr_of!(cmg)).areas[area1 as usize].floodnum == (*addr_of!(cmg)).areas[area2 as usize].floodnum {
            return qtrue;
        }
    }
    return qfalse;
}


/*
=================
CM_WriteAreaBits

Writes a bit vector of all the areas
that are in the same flood as the area parameter
Returns the number of bytes needed to hold all the bits.

The bits are OR'd in, so you can CM_WriteAreaBits from multiple
viewpoints and get the union of all visible areas.

This is used to cull non-visible entities from snapshots
=================
*/
fn CM_WriteAreaBits(buffer: *mut byte, area: i32) -> i32 {
    let mut i: i32;
    let mut floodnum: i32;
    let mut bytes: i32;

    unsafe {
        bytes = ((*addr_of!(cmg)).numAreas + 7) >> 3;

        #[cfg(not(feature = "bspc"))]
        {
            if (*addr_of!(cm_noAreas)).integer != 0 || area == -1 {
                // for debugging, send everything
                core::ptr::write_bytes(buffer, 255, bytes as usize);
            } else {
                floodnum = (*addr_of!(cmg)).areas[area as usize].floodnum;
                for i in 0..(*addr_of!(cmg)).numAreas {
                    if (*addr_of!(cmg)).areas[i as usize].floodnum == floodnum || area == -1 {
                        let byte_idx = (i >> 3) as usize;
                        let bit_idx = (i & 7) as usize;
                        *buffer.add(byte_idx) |= 1 << bit_idx;
                    }
                }
            }
        }

        #[cfg(feature = "bspc")]
        {
            if area == -1 {
                // for debugging, send everything
                core::ptr::write_bytes(buffer, 255, bytes as usize);
            } else {
                floodnum = (*addr_of!(cmg)).areas[area as usize].floodnum;
                for i in 0..(*addr_of!(cmg)).numAreas {
                    if (*addr_of!(cmg)).areas[i as usize].floodnum == floodnum || area == -1 {
                        let byte_idx = (i >> 3) as usize;
                        let bit_idx = (i & 7) as usize;
                        *buffer.add(byte_idx) |= 1 << bit_idx;
                    }
                }
            }
        }

        return bytes;
    }
}

fn CM_SnapPVS(origin: &[f32; 3], buffer: *mut byte) {
    let mut clientarea: i32;
    let mut leafnum: i32;
    let mut i: i32;

    unsafe {
        leafnum = CM_PointLeafnum(origin);
        clientarea = CM_LeafArea(leafnum);

        // calculate the visible areas
        core::ptr::write_bytes(buffer, 0, MAX_MAP_AREA_BYTES);
        CM_WriteAreaBits(buffer, clientarea);
        for i in 0..(MAX_MAP_AREA_BYTES / 4) as i32 {
            let ptr = buffer as *mut i32;
            *ptr.add(i as usize) = *ptr.add(i as usize) ^ -1;
        }
    }
}
