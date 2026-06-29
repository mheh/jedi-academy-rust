/* Triangle/triangle intersection test routine,
 * by Tomas Moller, 1997.
 * See article "A Fast Triangle-Triangle Intersection Test",
 * Journal of Graphics Tools, 2(2), 1997
 *
 * int tri_tri_intersect(float V0[3],float V1[3],float V2[3],
 *                         float U0[3],float U1[3],float U2[3])
 *
 * parameters: vertices of triangle 1: V0,V1,V2
 *             vertices of triangle 2: U0,U1,U2
 * result    : returns 1 if the triangles intersect, otherwise 0
 *
 */

use core::ffi::{c_int, c_float};

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

/* if USE_EPSILON_TEST is true then we do a check:
         if |dv|<EPSILON then dv=0.0;
   else no check is done (which is less robust)
*/
const USE_EPSILON_TEST: bool = true;
const EPSILON: f32 = 0.000001;

/* some macros */
/* CROSS: dest[0]=v1[1]*v2[2]-v1[2]*v2[1]; dest[1]=v1[2]*v2[0]-v1[0]*v2[2]; dest[2]=v1[0]*v2[1]-v1[1]*v2[0]; */
#[inline]
fn CROSS(dest: &mut vec3_t, v1: &vec3_t, v2: &vec3_t) {
    dest[0] = v1[1]*v2[2]-v1[2]*v2[1];
    dest[1] = v1[2]*v2[0]-v1[0]*v2[2];
    dest[2] = v1[0]*v2[1]-v1[1]*v2[0];
}

/* DOT: (v1[0]*v2[0]+v1[1]*v2[1]+v1[2]*v2[2]) */
#[inline]
fn DOT(v1: &vec3_t, v2: &vec3_t) -> f32 {
    v1[0]*v2[0]+v1[1]*v2[1]+v1[2]*v2[2]
}

/* SUB: dest[0]=v1[0]-v2[0]; dest[1]=v1[1]-v2[1]; dest[2]=v1[2]-v2[2]; */
#[inline]
fn SUB(dest: &mut vec3_t, v1: &vec3_t, v2: &vec3_t) {
    dest[0]=v1[0]-v2[0];
    dest[1]=v1[1]-v2[1];
    dest[2]=v1[2]-v2[2];
}

/* sort so that a<=b */
#[inline]
fn SORT(a: &mut f32, b: &mut f32) {
    if *a>*b {
        let c = *a;
        *a=*b;
        *b=c;
    }
}

/* ISECT: isect0=VV0+(VV1-VV0)*D0/(D0-D1); isect1=VV0+(VV2-VV0)*D0/(D0-D2); */
#[inline]
fn ISECT(vv0: f32, vv1: f32, vv2: f32, d0: f32, d1: f32, d2: f32) -> (f32, f32) {
    let isect0 = vv0+(vv1-vv0)*d0/(d0-d1);
    let isect1 = vv0+(vv2-vv0)*d0/(d0-d2);
    (isect0, isect1)
}

/* this edge to edge test is based on Franlin Antonio's gem:
   "Faster Line Segment Intersection", in Graphics Gems III,
   pp. 199-202 */
#[inline]
fn EDGE_EDGE_TEST(v0: &vec3_t, u0: &vec3_t, u1: &vec3_t, i0: usize, i1: usize, ax: f32, ay: f32) -> bool {
    let bx = u0[i0]-u1[i0];
    let by = u0[i1]-u1[i1];
    let cx = v0[i0]-u0[i0];
    let cy = v0[i1]-u0[i1];
    let f = ay*bx-ax*by;
    let d = by*cx-bx*cy;
    if (f>0.0 && d>=0.0 && d<=f) || (f<0.0 && d<=0.0 && d>=f) {
        let e = ax*cy-ay*cx;
        if f>0.0 {
            if e>=0.0 && e<=f { return true; }
        } else {
            if e<=0.0 && e>=f { return true; }
        }
    }
    false
}

extern "C" {
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn CrossProduct(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn VectorMA(base: *const vec3_t, scale: f32, direction: *const vec3_t, out: *mut vec3_t);
    pub fn Distance(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn VectorCopy(src: *const vec3_t, dest: *mut vec3_t);
    pub fn G_FindClosestPointOnLineSegment(start: *const vec3_t, end: *const vec3_t, from: *const vec3_t, result: *mut vec3_t) -> qboolean;
}

pub const Q3_INFINITE: c_float = 16777216.0;

pub fn coplanar_tri_tri(n: &vec3_t, v0: &vec3_t, v1: &vec3_t, v2: &vec3_t,
                        u0: &vec3_t, u1: &vec3_t, u2: &vec3_t) -> qboolean {
    let mut a: f32;
    let mut b: f32;
    let mut c: f32;
    let mut d0: f32;
    let mut d1: f32;
    let mut d2: f32;
    let mut i0: usize = 0;
    let mut i1: usize = 0;

    /* first project onto an axis-aligned plane, that maximizes the area */
    /* of the triangles, compute indices: i0,i1. */
    let mut a_v = [0.0f32; 3];
    a_v[0] = n[0].abs();
    a_v[1] = n[1].abs();
    a_v[2] = n[2].abs();

    if a_v[0]>a_v[1] {
        if a_v[0]>a_v[2] {
            i0=1;      /* A[0] is greatest */
            i1=2;
        } else {
            i0=0;      /* A[2] is greatest */
            i1=1;
        }
    } else {   /* A[0]<=A[1] */
        if a_v[2]>a_v[1] {
            i0=0;      /* A[2] is greatest */
            i1=1;
        } else {
            i0=0;      /* A[1] is greatest */
            i1=2;
        }
    }

    /* test all edges of triangle 1 against the edges of triangle 2 */

    /* EDGE_AGAINST_TRI_EDGES(V0,V1,U0,U1,U2) */
    {
        let ax = v1[i0]-v0[i0];
        let ay = v1[i1]-v0[i1];
        /* test edge U0,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v0, u0, u1, i0, i1, ax, ay) { return 1; }
        /* test edge U1,U2 against V0,V1 */
        if EDGE_EDGE_TEST(v0, u1, u2, i0, i1, ax, ay) { return 1; }
        /* test edge U2,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v0, u2, u0, i0, i1, ax, ay) { return 1; }
    }

    /* EDGE_AGAINST_TRI_EDGES(V1,V2,U0,U1,U2) */
    {
        let ax = v2[i0]-v1[i0];
        let ay = v2[i1]-v1[i1];
        /* test edge U0,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v1, u0, u1, i0, i1, ax, ay) { return 1; }
        /* test edge U1,U2 against V0,V1 */
        if EDGE_EDGE_TEST(v1, u1, u2, i0, i1, ax, ay) { return 1; }
        /* test edge U2,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v1, u2, u0, i0, i1, ax, ay) { return 1; }
    }

    /* EDGE_AGAINST_TRI_EDGES(V2,V0,U0,U1,U2) */
    {
        let ax = v0[i0]-v2[i0];
        let ay = v0[i1]-v2[i1];
        /* test edge U0,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v2, u0, u1, i0, i1, ax, ay) { return 1; }
        /* test edge U1,U2 against V0,V1 */
        if EDGE_EDGE_TEST(v2, u1, u2, i0, i1, ax, ay) { return 1; }
        /* test edge U2,U1 against V0,V1 */
        if EDGE_EDGE_TEST(v2, u2, u0, i0, i1, ax, ay) { return 1; }
    }

    /* finally, test if tri1 is totally contained in tri2 or vice versa */

    /* POINT_IN_TRI(V0,U0,U1,U2) */
    {
        /* is T1 completly inside T2? */
        /* check if V0 is inside tri(U0,U1,U2) */
        a=u1[i1]-u0[i1];
        b=-(u1[i0]-u0[i0]);
        c=-a*u0[i0]-b*u0[i1];
        d0=a*v0[i0]+b*v0[i1]+c;

        a=u2[i1]-u1[i1];
        b=-(u2[i0]-u1[i0]);
        c=-a*u1[i0]-b*u1[i1];
        d1=a*v0[i0]+b*v0[i1]+c;

        a=u0[i1]-u2[i1];
        b=-(u0[i0]-u2[i0]);
        c=-a*u2[i0]-b*u2[i1];
        d2=a*v0[i0]+b*v0[i1]+c;
        if d0*d1>0.0 {
            if d0*d2>0.0 { return 1; }
        }
    }

    /* POINT_IN_TRI(U0,V0,V1,V2) */
    {
        /* is T1 completly inside T2? */
        /* check if V0 is inside tri(U0,U1,U2) */
        a=v1[i1]-v0[i1];
        b=-(v1[i0]-v0[i0]);
        c=-a*v0[i0]-b*v0[i1];
        d0=a*u0[i0]+b*u0[i1]+c;

        a=v2[i1]-v1[i1];
        b=-(v2[i0]-v1[i0]);
        c=-a*v1[i0]-b*v1[i1];
        d1=a*u0[i0]+b*u0[i1]+c;

        a=v0[i1]-v2[i1];
        b=-(v0[i0]-v2[i0]);
        c=-a*v2[i0]-b*v2[i1];
        d2=a*u0[i0]+b*u0[i1]+c;
        if d0*d1>0.0 {
            if d0*d2>0.0 { return 1; }
        }
    }

    return 0;
}

pub fn tri_tri_intersect(v0: &vec3_t, v1: &vec3_t, v2: &vec3_t,
                         u0: &vec3_t, u1: &vec3_t, u2: &vec3_t) -> qboolean {
    let mut e1: vec3_t = [0.0; 3];
    let mut e2: vec3_t = [0.0; 3];
    let mut n1: vec3_t = [0.0; 3];
    let mut n2: vec3_t = [0.0; 3];
    let mut d1: f32;
    let mut d2: f32;
    let mut du0: f32;
    let mut du1: f32;
    let mut du2: f32;
    let mut dv0: f32;
    let mut dv1: f32;
    let mut dv2: f32;
    let mut d: vec3_t = [0.0; 3];
    let mut isect1: [f32; 2] = [0.0; 2];
    let mut isect2: [f32; 2] = [0.0; 2];
    let mut du0du1: f32;
    let mut du0du2: f32;
    let mut dv0dv1: f32;
    let mut dv0dv2: f32;
    let mut index: usize;
    let mut vp0: f32;
    let mut vp1: f32;
    let mut vp2: f32;
    let mut up0: f32;
    let mut up1: f32;
    let mut up2: f32;
    let mut b: f32;
    let mut c: f32;
    let mut max: f32;

    /* compute plane equation of triangle(V0,V1,V2) */
    SUB(&mut e1,v1,v0);
    SUB(&mut e2,v2,v0);
    CROSS(&mut n1,&e1,&e2);
    d1=-DOT(&n1,v0);
    /* plane equation 1: N1.X+d1=0 */

    /* put U0,U1,U2 into plane equation 1 to compute signed distances to the plane*/
    du0=DOT(&n1,u0)+d1;
    du1=DOT(&n1,u1)+d1;
    du2=DOT(&n1,u2)+d1;

    /* coplanarity robustness check */
    if USE_EPSILON_TEST {
        if du0.abs()<EPSILON { du0=0.0; }
        if du1.abs()<EPSILON { du1=0.0; }
        if du2.abs()<EPSILON { du2=0.0; }
    }
    du0du1=du0*du1;
    du0du2=du0*du2;

    if du0du1>0.0f32 && du0du2>0.0f32 { /* same sign on all of them + not equal 0 ? */
        return 0;                    /* no intersection occurs */
    }

    /* compute plane of triangle (U0,U1,U2) */
    SUB(&mut e1,u1,u0);
    SUB(&mut e2,u2,u0);
    CROSS(&mut n2,&e1,&e2);
    d2=-DOT(&n2,u0);
    /* plane equation 2: N2.X+d2=0 */

    /* put V0,V1,V2 into plane equation 2 */
    dv0=DOT(&n2,v0)+d2;
    dv1=DOT(&n2,v1)+d2;
    dv2=DOT(&n2,v2)+d2;

    if USE_EPSILON_TEST {
        if dv0.abs()<EPSILON { dv0=0.0; }
        if dv1.abs()<EPSILON { dv1=0.0; }
        if dv2.abs()<EPSILON { dv2=0.0; }
    }

    dv0dv1=dv0*dv1;
    dv0dv2=dv0*dv2;

    if dv0dv1>0.0f32 && dv0dv2>0.0f32 { /* same sign on all of them + not equal 0 ? */
        return 0;                    /* no intersection occurs */
    }

    /* compute direction of intersection line */
    CROSS(&mut d,&n1,&n2);

    /* compute and index to the largest component of D */
    max=d[0].abs();
    index=0;
    b=d[1].abs();
    c=d[2].abs();
    if b>max { max=b; index=1; }
    if c>max { max=c; index=2; }

        /* this is the simplified projection onto L*/
        vp0=v0[index];
        vp1=v1[index];
        vp2=v2[index];

        up0=u0[index];
        up1=u1[index];
        up2=u2[index];

    /* compute interval for triangle 1 */
    /* COMPUTE_INTERVALS(vp0,vp1,vp2,dv0,dv1,dv2,dv0dv1,dv0dv2,isect1[0],isect1[1]) */
    if dv0dv1>0.0f32 {
        /* here we know that D0D2<=0.0 */
        /* that is D0, D1 are on the same side, D2 on the other or on the plane */
        let (i0, i1) = ISECT(vp2,vp0,vp1,dv2,dv0,dv1);
        isect1[0] = i0;
        isect1[1] = i1;
    } else if dv0dv2>0.0f32 {
        /* here we know that d0d1<=0.0 */
        let (i0, i1) = ISECT(vp1,vp0,vp2,dv1,dv0,dv2);
        isect1[0] = i0;
        isect1[1] = i1;
    } else if dv1*dv2>0.0f32 || dv0!=0.0f32 {
        /* here we know that d0d1<=0.0 or that D0!=0.0 */
        let (i0, i1) = ISECT(vp0,vp1,vp2,dv0,dv1,dv2);
        isect1[0] = i0;
        isect1[1] = i1;
    } else if dv1!=0.0f32 {
        let (i0, i1) = ISECT(vp1,vp0,vp2,dv1,dv0,dv2);
        isect1[0] = i0;
        isect1[1] = i1;
    } else if dv2!=0.0f32 {
        let (i0, i1) = ISECT(vp2,vp0,vp1,dv2,dv0,dv1);
        isect1[0] = i0;
        isect1[1] = i1;
    } else {
        /* triangles are coplanar */
        return coplanar_tri_tri(&n1, v0, v1, v2, u0, u1, u2);
    }

    /* compute interval for triangle 2 */
    /* COMPUTE_INTERVALS(up0,up1,up2,du0,du1,du2,du0du1,du0du2,isect2[0],isect2[1]) */
    if du0du1>0.0f32 {
        /* here we know that D0D2<=0.0 */
        /* that is D0, D1 are on the same side, D2 on the other or on the plane */
        let (i0, i1) = ISECT(up2,up0,up1,du2,du0,du1);
        isect2[0] = i0;
        isect2[1] = i1;
    } else if du0du2>0.0f32 {
        /* here we know that d0d1<=0.0 */
        let (i0, i1) = ISECT(up1,up0,up2,du1,du0,du2);
        isect2[0] = i0;
        isect2[1] = i1;
    } else if du1*du2>0.0f32 || du0!=0.0f32 {
        /* here we know that d0d1<=0.0 or that D0!=0.0 */
        let (i0, i1) = ISECT(up0,up1,up2,du0,du1,du2);
        isect2[0] = i0;
        isect2[1] = i1;
    } else if du1!=0.0f32 {
        let (i0, i1) = ISECT(up1,up0,up2,du1,du0,du2);
        isect2[0] = i0;
        isect2[1] = i1;
    } else if du2!=0.0f32 {
        let (i0, i1) = ISECT(up2,up0,up1,du2,du0,du1);
        isect2[0] = i0;
        isect2[1] = i1;
    } else {
        /* triangles are coplanar */
        return coplanar_tri_tri(&n1, v0, v1, v2, u0, u1, u2);
    }

    SORT(&mut isect1[0],&mut isect1[1]);
    SORT(&mut isect2[0],&mut isect2[1]);

    if isect1[1]<isect2[0] || isect2[1]<isect1[0] { return 1; }
    return 0;
}


pub fn LineSegmentDistance(a: &vec3_t, b: &vec3_t, c: &vec3_t, d: &vec3_t) -> f32 {
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let mut v3: vec3_t = [0.0; 3];
    let mut cross: vec3_t = [0.0; 3];
    //FIXME: what if parallel or intersect?
    //FIXME: this doesn't take into account the endpoints...

    //get the two lines
    unsafe { VectorSubtract(b as *const _, a as *const _, &mut v1 as *mut _); }
    unsafe { VectorSubtract(c as *const _, d as *const _, &mut v2 as *mut _); }

    //get their normalized cross product
    unsafe { CrossProduct(&v1 as *const _, &v2 as *const _, &mut cross as *mut _); }
    /*
    float crossLength = VectorLength( cross );
    if ( crossLength == 0 )
    {//intersect!  Or... parallel?
        return 0;
    }
    VectorScale( cross, 1/crossLength, cross );
    */
    unsafe { VectorNormalize(&mut cross as *mut _); }

    //now get a vector from v1 to v2
    unsafe { VectorSubtract(d as *const _, a as *const _, &mut v3 as *mut _); }

    //distance is dot product of that new vector and the normalized cross product
    let dist = unsafe { DotProduct(&v3 as *const _, &cross as *const _) }.abs();

    return dist;
}

pub fn ShortestLineSegBewteen2LineSegs(start1: &vec3_t, end1: &vec3_t, start2: &vec3_t, end2: &vec3_t, close_pnt1: &mut vec3_t, close_pnt2: &mut vec3_t) -> f32 {
    let mut current_dist: f32;
    let mut new_dist: f32;
    let mut new_pnt: vec3_t = [0.0; 3];
    //start1, end1 : the first segment
    //start2, end2 : the second segment

    //output, one point on each segment, the closest two points on the segments.

    //compute some temporaries:
    //vec start_dif = start2 - start1
    let mut start_dif: vec3_t = [0.0; 3];
    unsafe { VectorSubtract(start2 as *const _, start1 as *const _, &mut start_dif as *mut _); }
    //vec v1 = end1 - start1
    let mut v1: vec3_t = [0.0; 3];
    unsafe { VectorSubtract(end1 as *const _, start1 as *const _, &mut v1 as *mut _); }
    //vec v2 = end2 - start2
    let mut v2: vec3_t = [0.0; 3];
    unsafe { VectorSubtract(end2 as *const _, start2 as *const _, &mut v2 as *mut _); }
    //
    let v1v1 = unsafe { DotProduct(&v1 as *const _, &v1 as *const _) };
    let v2v2 = unsafe { DotProduct(&v2 as *const _, &v2 as *const _) };
    let v1v2 = unsafe { DotProduct(&v1 as *const _, &v2 as *const _) };

    //the main computation

    let denom = (v1v2 * v1v2) - (v1v1 * v2v2);

    //if denom is small, then skip all this and jump to the section marked below
    if denom.abs() > 0.001f32 {
        let s = -( (v2v2*unsafe { DotProduct(&v1 as *const _, &start_dif as *const _) }) - (v1v2*unsafe { DotProduct(&v2 as *const _, &start_dif as *const _) }) ) / denom;
        let mut s = s;
        let t = ( (v1v1*unsafe { DotProduct(&v2 as *const _, &start_dif as *const _) }) - (v1v2*unsafe { DotProduct(&v1 as *const _, &start_dif as *const _) }) ) / denom;
        let mut t = t;
        let mut done = true as c_int;

        if s < 0.0 {
            done = 0;
            s = 0.0;// and see note below
        }

        if s > 1.0 {
            done = 0;
            s = 1.0;// and see note below
        }

        if t < 0.0 {
            done = 0;
            t = 0.0;// and see note below
        }

        if t > 1.0 {
            done = 0;
            t = 1.0;// and see note below
        }

        //vec close_pnt1 = start1 + s * v1
        unsafe { VectorMA(start1 as *const _, s, &v1 as *const _, close_pnt1 as *mut _); }
        //vec close_pnt2 = start2 + t * v2
        unsafe { VectorMA(start2 as *const _, t, &v2 as *const _, close_pnt2 as *mut _); }

        current_dist = unsafe { Distance(close_pnt1 as *const _, close_pnt2 as *const _) };
        //now, if none of those if's fired, you are done.
        if done != 0 {
            return current_dist;
        }
        //If they did fire, then we need to do some additional tests.

        //What we are gonna do is see if we can find a shorter distance than the above
        //involving the endpoints.
    } else {
        //******start here for paralell lines with current_dist = infinity****
        current_dist = Q3_INFINITE;
    }

    //test 2 close_pnts first
    /*
    G_FindClosestPointOnLineSegment( start1, end1, close_pnt2, new_pnt );
    new_dist = Distance( close_pnt2, new_pnt );
    if ( new_dist < current_dist )
    {//then update close_pnt1 close_pnt2 and current_dist
        VectorCopy( new_pnt, close_pnt1 );
        VectorCopy( close_pnt2, close_pnt2 );
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment( start2, end2, close_pnt1, new_pnt );
    new_dist = Distance( close_pnt1, new_pnt );
    if ( new_dist < current_dist )
    {//then update close_pnt1 close_pnt2 and current_dist
        VectorCopy( close_pnt1, close_pnt1 );
        VectorCopy( new_pnt, close_pnt2 );
        current_dist = new_dist;
    }
    */
    //test all the endpoints
    new_dist = unsafe { Distance(start1 as *const _, start2 as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(start1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(start2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    new_dist = unsafe { Distance(start1 as *const _, end2 as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(start1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(end2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    new_dist = unsafe { Distance(end1 as *const _, start2 as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(end1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(start2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    new_dist = unsafe { Distance(end1 as *const _, end2 as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(end1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(end2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    //Then we have 4 more point / segment tests

    unsafe { G_FindClosestPointOnLineSegment(start2 as *const _, end2 as *const _, start1 as *const _, &mut new_pnt as *mut _); }
    new_dist = unsafe { Distance(start1 as *const _, &new_pnt as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(start1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(&new_pnt as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    unsafe { G_FindClosestPointOnLineSegment(start2 as *const _, end2 as *const _, end1 as *const _, &mut new_pnt as *mut _); }
    new_dist = unsafe { Distance(end1 as *const _, &new_pnt as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(end1 as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(&new_pnt as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    unsafe { G_FindClosestPointOnLineSegment(start1 as *const _, end1 as *const _, start2 as *const _, &mut new_pnt as *mut _); }
    new_dist = unsafe { Distance(start2 as *const _, &new_pnt as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(&new_pnt as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(start2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    unsafe { G_FindClosestPointOnLineSegment(start1 as *const _, end1 as *const _, end2 as *const _, &mut new_pnt as *mut _); }
    new_dist = unsafe { Distance(end2 as *const _, &new_pnt as *const _) };
    if new_dist < current_dist {
        //then update close_pnt1 close_pnt2 and current_dist
        unsafe { VectorCopy(&new_pnt as *const _, close_pnt1 as *mut _); }
        unsafe { VectorCopy(end2 as *const _, close_pnt2 as *mut _); }
        current_dist = new_dist;
    }

    return current_dist;
}
