//! Triangle/triangle intersection test routine,
//! by Tomas Moller, 1997.
//! See article "A Fast Triangle-Triangle Intersection Test",
//! Journal of Graphics Tools, 2(2), 1997
//!
//! ```text
//! int tri_tri_intersect(float V0[3],float V1[3],float V2[3],
//!                         float U0[3],float U1[3],float U2[3])
//! ```
//!
//! parameters: vertices of triangle 1: V0,V1,V2;
//!             vertices of triangle 2: U0,U1,U2.
//! result    : returns 1 if the triangles intersect, otherwise 0.
//!
//! A faithful port of `tri_coll_test.c`, compared against OpenJK (which is byte-identical
//! here). Function and variable names mirror the C originals and the original comments are
//! carried over, so the Rust can be diffed against the source. Every function is parity-tested
//! bit-exact against the extracted C oracle: `cargo test --features oracle`.
//!
//! The C builds the two routines out of preprocessor macros (`CROSS`/`DOT`/`SUB`/`ISECT`/
//! `COMPUTE_INTERVALS`/`EDGE_*`/`POINT_IN_TRI`); since several of them embed `return`s and
//! reference enclosing locals, they are expanded here either inline or into the file-private
//! helpers below ([`edge_edge_test`]/[`edge_against_tri_edges`]/[`point_in_tri`]/
//! [`compute_intervals`]), each carrying its originating macro for reference.

#![allow(non_snake_case)]

use super::q_shared_h::{qboolean, vec3_t, QFALSE, QTRUE};

// `#define USE_EPSILON_TEST 1` -- always enabled, so the `fabs(d*) < EPSILON` snap-to-zero
// coplanarity checks in [`tri_tri_intersect`] are unconditionally compiled in.

/// `#define EPSILON 0.000001` -- a C `double` literal. The `fabs(...) < EPSILON` coplanarity
/// checks are therefore evaluated in `f64`, exactly as the C does (see [`fabs`]).
const EPSILON: f64 = 0.000001;

/// C `fabs` -- `double fabs(double)` from `<math.h>`. tri_coll_test passes `float` args, so
/// each is promoted to `double` and the result is `double`. Callers that store into a `float`
/// (`A[i]=fabs(...)`, `max=fabs(...)`) narrow it back with `as f32` (which equals `f32::abs`
/// for finite floats), while the `fabs(d*) < EPSILON` test keeps it in `f64`: the `double`
/// threshold `EPSILON` differs from its `f32` rounding, so that comparison must not be narrowed.
#[inline]
fn fabs(x: f32) -> f64 {
    (x as f64).abs()
}

// this edge to edge test is based on Franlin Antonio's gem:
//    "Faster Line Segment Intersection", in Graphics Gems III,
//    pp. 199-202
//
// EDGE_EDGE_TEST(V0,U0,U1) -- in the C, `Ax`/`Ay` and the projection indices `i0`/`i1` come
// from the enclosing EDGE_AGAINST_TRI_EDGES expansion, so they are passed in explicitly here.
// A `true` return means the macro hit its `return 1`, i.e. the calling function returns qtrue.
fn edge_edge_test(
    i0: usize,
    i1: usize,
    Ax: f32,
    Ay: f32,
    V0: &vec3_t,
    U0: &vec3_t,
    U1: &vec3_t,
) -> bool {
    let Bx = U0[i0] - U1[i0];
    let By = U0[i1] - U1[i1];
    let Cx = V0[i0] - U0[i0];
    let Cy = V0[i1] - U0[i1];
    let f = Ay * Bx - Ax * By;
    let d = By * Cx - Bx * Cy;
    if (f > 0.0 && d >= 0.0 && d <= f) || (f < 0.0 && d <= 0.0 && d >= f) {
        let e = Ax * Cy - Ay * Cx;
        if f > 0.0 {
            if e >= 0.0 && e <= f {
                return true;
            }
        } else if e <= 0.0 && e >= f {
            return true;
        }
    }
    false
}

// EDGE_AGAINST_TRI_EDGES(V0,V1,U0,U1,U2) -- a `true` return propagates the macro's `return 1`.
fn edge_against_tri_edges(
    i0: usize,
    i1: usize,
    V0: &vec3_t,
    V1: &vec3_t,
    U0: &vec3_t,
    U1: &vec3_t,
    U2: &vec3_t,
) -> bool {
    let Ax = V1[i0] - V0[i0];
    let Ay = V1[i1] - V0[i1];
    // test edge U0,U1 against V0,V1
    if edge_edge_test(i0, i1, Ax, Ay, V0, U0, U1) {
        return true;
    }
    // test edge U1,U2 against V0,V1
    if edge_edge_test(i0, i1, Ax, Ay, V0, U1, U2) {
        return true;
    }
    // test edge U2,U1 against V0,V1
    if edge_edge_test(i0, i1, Ax, Ay, V0, U2, U0) {
        return true;
    }
    false
}

// POINT_IN_TRI(V0,U0,U1,U2) -- a `true` return propagates the macro's `return 1`.
fn point_in_tri(i0: usize, i1: usize, V0: &vec3_t, U0: &vec3_t, U1: &vec3_t, U2: &vec3_t) -> bool {
    // is T1 completly inside T2?
    // check if V0 is inside tri(U0,U1,U2)
    let a = U1[i1] - U0[i1];
    let b = -(U1[i0] - U0[i0]);
    let c = -a * U0[i0] - b * U0[i1];
    let d0 = a * V0[i0] + b * V0[i1] + c;

    let a = U2[i1] - U1[i1];
    let b = -(U2[i0] - U1[i0]);
    let c = -a * U1[i0] - b * U1[i1];
    let d1 = a * V0[i0] + b * V0[i1] + c;

    let a = U0[i1] - U2[i1];
    let b = -(U0[i0] - U2[i0]);
    let c = -a * U2[i0] - b * U2[i1];
    let d2 = a * V0[i0] + b * V0[i1] + c;
    if d0 * d1 > 0.0 {
        if d0 * d2 > 0.0 {
            return true;
        }
    }
    false
}

// COMPUTE_INTERVALS(VV0,VV1,VV2,D0,D1,D2,D0D1,D0D2,isect0,isect1) and the ISECT it dispatches
// to:
//   ISECT(VV0,VV1,VV2,D0,D1,D2,isect0,isect1):
//     isect0 = VV0+(VV1-VV0)*D0/(D0-D1)
//     isect1 = VV0+(VV2-VV0)*D0/(D0-D2)
// Returns `Some([isect0, isect1])` for the five computed cases; `None` is the macro's terminal
// `else` ("triangles are coplanar"), where the C does `return coplanar_tri_tri(...)` -- the
// caller maps `None` to exactly that return.
fn compute_intervals(
    VV0: f32,
    VV1: f32,
    VV2: f32,
    D0: f32,
    D1: f32,
    D2: f32,
    D0D1: f32,
    D0D2: f32,
) -> Option<[f32; 2]> {
    if D0D1 > 0.0 {
        // here we know that D0D2<=0.0
        // that is D0, D1 are on the same side, D2 on the other or on the plane
        // ISECT(VV2,VV0,VV1,D2,D0,D1,isect0,isect1)
        Some([
            VV2 + (VV0 - VV2) * D2 / (D2 - D0),
            VV2 + (VV1 - VV2) * D2 / (D2 - D1),
        ])
    } else if D0D2 > 0.0 {
        // here we know that d0d1<=0.0
        // ISECT(VV1,VV0,VV2,D1,D0,D2,isect0,isect1)
        Some([
            VV1 + (VV0 - VV1) * D1 / (D1 - D0),
            VV1 + (VV2 - VV1) * D1 / (D1 - D2),
        ])
    } else if D1 * D2 > 0.0 || D0 != 0.0 {
        // here we know that d0d1<=0.0 or that D0!=0.0
        // ISECT(VV0,VV1,VV2,D0,D1,D2,isect0,isect1)
        Some([
            VV0 + (VV1 - VV0) * D0 / (D0 - D1),
            VV0 + (VV2 - VV0) * D0 / (D0 - D2),
        ])
    } else if D1 != 0.0 {
        // ISECT(VV1,VV0,VV2,D1,D0,D2,isect0,isect1)
        Some([
            VV1 + (VV0 - VV1) * D1 / (D1 - D0),
            VV1 + (VV2 - VV1) * D1 / (D1 - D2),
        ])
    } else if D2 != 0.0 {
        // ISECT(VV2,VV0,VV1,D2,D0,D1,isect0,isect1)
        Some([
            VV2 + (VV0 - VV2) * D2 / (D2 - D0),
            VV2 + (VV1 - VV2) * D2 / (D2 - D1),
        ])
    } else {
        // triangles are coplanar
        None
    }
}

pub fn coplanar_tri_tri(
    N: &vec3_t,
    V0: &vec3_t,
    V1: &vec3_t,
    V2: &vec3_t,
    U0: &vec3_t,
    U1: &vec3_t,
    U2: &vec3_t,
) -> qboolean {
    let mut A: vec3_t = [0.0; 3];
    // C `short i0,i1;` -- used only as vec3_t indices, so `usize` here.
    let i0: usize;
    let i1: usize;
    // first project onto an axis-aligned plane, that maximizes the area
    // of the triangles, compute indices: i0,i1.
    A[0] = fabs(N[0]) as f32;
    A[1] = fabs(N[1]) as f32;
    A[2] = fabs(N[2]) as f32;
    if A[0] > A[1] {
        if A[0] > A[2] {
            i0 = 1; // A[0] is greatest
            i1 = 2;
        } else {
            i0 = 0; // A[2] is greatest
            i1 = 1;
        }
    } else {
        // A[0]<=A[1]
        if A[2] > A[1] {
            i0 = 0; // A[2] is greatest
            i1 = 1;
        } else {
            i0 = 0; // A[1] is greatest
            i1 = 2;
        }
    }

    // test all edges of triangle 1 against the edges of triangle 2
    if edge_against_tri_edges(i0, i1, V0, V1, U0, U1, U2) {
        return QTRUE;
    }
    if edge_against_tri_edges(i0, i1, V1, V2, U0, U1, U2) {
        return QTRUE;
    }
    if edge_against_tri_edges(i0, i1, V2, V0, U0, U1, U2) {
        return QTRUE;
    }

    // finally, test if tri1 is totally contained in tri2 or vice versa
    if point_in_tri(i0, i1, V0, U0, U1, U2) {
        return QTRUE;
    }
    if point_in_tri(i0, i1, U0, V0, V1, V2) {
        return QTRUE;
    }

    QFALSE
}

// `max=c` in the largest-component selection below is a faithful dead store (the C writes
// `max=c,index=2` but never reads `max` afterward); allow it so the port stays literal.
#[allow(unused_assignments)]
pub fn tri_tri_intersect(
    V0: &vec3_t,
    V1: &vec3_t,
    V2: &vec3_t,
    U0: &vec3_t,
    U1: &vec3_t,
    U2: &vec3_t,
) -> qboolean {
    let mut E1: vec3_t = [0.0; 3];
    let mut E2: vec3_t = [0.0; 3];
    let mut N1: vec3_t = [0.0; 3];
    let mut N2: vec3_t = [0.0; 3];

    // compute plane equation of triangle(V0,V1,V2)
    // SUB(E1,V1,V0)
    E1[0] = V1[0] - V0[0];
    E1[1] = V1[1] - V0[1];
    E1[2] = V1[2] - V0[2];
    // SUB(E2,V2,V0)
    E2[0] = V2[0] - V0[0];
    E2[1] = V2[1] - V0[1];
    E2[2] = V2[2] - V0[2];
    // CROSS(N1,E1,E2)
    N1[0] = E1[1] * E2[2] - E1[2] * E2[1];
    N1[1] = E1[2] * E2[0] - E1[0] * E2[2];
    N1[2] = E1[0] * E2[1] - E1[1] * E2[0];
    let d1 = -(N1[0] * V0[0] + N1[1] * V0[1] + N1[2] * V0[2]);
    // plane equation 1: N1.X+d1=0

    // put U0,U1,U2 into plane equation 1 to compute signed distances to the plane
    let mut du0 = (N1[0] * U0[0] + N1[1] * U0[1] + N1[2] * U0[2]) + d1;
    let mut du1 = (N1[0] * U1[0] + N1[1] * U1[1] + N1[2] * U1[2]) + d1;
    let mut du2 = (N1[0] * U2[0] + N1[1] * U2[1] + N1[2] * U2[2]) + d1;

    // coplanarity robustness check (USE_EPSILON_TEST == 1)
    if fabs(du0) < EPSILON {
        du0 = 0.0;
    }
    if fabs(du1) < EPSILON {
        du1 = 0.0;
    }
    if fabs(du2) < EPSILON {
        du2 = 0.0;
    }
    let du0du1 = du0 * du1;
    let du0du2 = du0 * du2;

    if du0du1 > 0.0 && du0du2 > 0.0 {
        // same sign on all of them + not equal 0 ?
        return QFALSE; // C: `return 0` -- no intersection occurs
    }

    // compute plane of triangle (U0,U1,U2)
    // SUB(E1,U1,U0)
    E1[0] = U1[0] - U0[0];
    E1[1] = U1[1] - U0[1];
    E1[2] = U1[2] - U0[2];
    // SUB(E2,U2,U0)
    E2[0] = U2[0] - U0[0];
    E2[1] = U2[1] - U0[1];
    E2[2] = U2[2] - U0[2];
    // CROSS(N2,E1,E2)
    N2[0] = E1[1] * E2[2] - E1[2] * E2[1];
    N2[1] = E1[2] * E2[0] - E1[0] * E2[2];
    N2[2] = E1[0] * E2[1] - E1[1] * E2[0];
    let d2 = -(N2[0] * U0[0] + N2[1] * U0[1] + N2[2] * U0[2]);
    // plane equation 2: N2.X+d2=0

    // put V0,V1,V2 into plane equation 2
    let mut dv0 = (N2[0] * V0[0] + N2[1] * V0[1] + N2[2] * V0[2]) + d2;
    let mut dv1 = (N2[0] * V1[0] + N2[1] * V1[1] + N2[2] * V1[2]) + d2;
    let mut dv2 = (N2[0] * V2[0] + N2[1] * V2[1] + N2[2] * V2[2]) + d2;

    if fabs(dv0) < EPSILON {
        dv0 = 0.0;
    }
    if fabs(dv1) < EPSILON {
        dv1 = 0.0;
    }
    if fabs(dv2) < EPSILON {
        dv2 = 0.0;
    }

    let dv0dv1 = dv0 * dv1;
    let dv0dv2 = dv0 * dv2;

    if dv0dv1 > 0.0 && dv0dv2 > 0.0 {
        // same sign on all of them + not equal 0 ?
        return QFALSE; // C: `return 0` -- no intersection occurs
    }

    // compute direction of intersection line
    let mut D: vec3_t = [0.0; 3];
    // CROSS(D,N1,N2)
    D[0] = N1[1] * N2[2] - N1[2] * N2[1];
    D[1] = N1[2] * N2[0] - N1[0] * N2[2];
    D[2] = N1[0] * N2[1] - N1[1] * N2[0];

    // compute and index to the largest component of D
    let mut max = fabs(D[0]) as f32;
    let mut index: usize = 0;
    let b = fabs(D[1]) as f32;
    let c = fabs(D[2]) as f32;
    if b > max {
        max = b;
        index = 1;
    }
    if c > max {
        max = c; // C: `max=c,index=2` -- `max` is dead after this (see the fn-level allow)
        index = 2;
    }

    // this is the simplified projection onto L
    let vp0 = V0[index];
    let vp1 = V1[index];
    let vp2 = V2[index];

    let up0 = U0[index];
    let up1 = U1[index];
    let up2 = U2[index];

    // compute interval for triangle 1
    let mut isect1 = match compute_intervals(vp0, vp1, vp2, dv0, dv1, dv2, dv0dv1, dv0dv2) {
        Some(v) => v,
        None => return coplanar_tri_tri(&N1, V0, V1, V2, U0, U1, U2),
    };

    // compute interval for triangle 2
    let mut isect2 = match compute_intervals(up0, up1, up2, du0, du1, du2, du0du1, du0du2) {
        Some(v) => v,
        None => return coplanar_tri_tri(&N1, V0, V1, V2, U0, U1, U2),
    };

    // SORT(isect1[0],isect1[1])
    if isect1[0] > isect1[1] {
        isect1.swap(0, 1);
    }
    // SORT(isect2[0],isect2[1])
    if isect2[0] > isect2[1] {
        isect2.swap(0, 1);
    }

    // NOTE (carried verbatim from JKA; OpenJK is byte-identical here): this returns qtrue when
    // the projected intervals are DISJOINT and qfalse when they overlap -- the inverse of the
    // textbook Moller test, and inconsistent with the two `return 0` early-outs above. The
    // saber-collision callers in w_saber.c depend on exactly this, so it is reproduced as-is.
    // See DEVIATIONS.md "Carried quirk: tri_tri_intersect inverted result".
    if isect1[1] < isect2[0] || isect2[1] < isect1[0] {
        return QTRUE;
    }
    QFALSE
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// A spread of triangles chosen to exercise the 3D-crossing, disjoint, coplanar (which
    /// routes `tri_tri_intersect` through `coplanar_tri_tri`), touching, tilted, and
    /// degenerate paths. Every ordered pair -- including self-pairs, which are coplanar -- is
    /// run through `tri_tri_intersect` and compared exactly against the C oracle.
    fn triangle_pool() -> [[vec3_t; 3]; 10] {
        [
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], // unit tri in z=0
            [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 0.0]], // crosses z=0
            [[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]], // far away
            [[0.25, 0.25, -1.0], [0.25, 0.25, 1.0], [0.5, 0.5, 0.0]], // vertical through z=0 tri
            [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]], // bigger, coplanar w/ first
            [[-1.0, -1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, 0.0, 0.0]], // coplanar z=0, overlaps
            [[0.1, 0.1, 0.0], [0.1, 0.1, 0.0], [0.1, 0.1, 0.0]], // degenerate point
            [[10.0, 0.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 10.0]], // tilted plane
            [[-5.0, -5.0, 2.0], [5.0, -5.0, 2.0], [0.0, 5.0, 2.0]], // z=2 plane
            [[0.3, 0.3, -2.0], [0.3, 0.3, 2.0], [0.31, 0.31, 0.0]], // near-degenerate sliver
        ]
    }

    #[test]
    fn tri_tri_intersect_matches_oracle() {
        let tris = triangle_pool();
        for v in &tris {
            for u in &tris {
                let r = tri_tri_intersect(&v[0], &v[1], &v[2], &u[0], &u[1], &u[2]);
                let c = unsafe {
                    oracle::tri_tri_intersect(
                        v[0].as_ptr(),
                        v[1].as_ptr(),
                        v[2].as_ptr(),
                        u[0].as_ptr(),
                        u[1].as_ptr(),
                        u[2].as_ptr(),
                    )
                };
                assert_eq!(r, c, "tri_tri_intersect mismatch for V={v:?} U={u:?}");
            }
        }
    }

    #[test]
    fn coplanar_tri_tri_matches_oracle() {
        // (N, [V0,V1,V2], [U0,U1,U2]). The N values span every i0/i1 projection-axis branch
        // (largest |component| on x, y, z), and the triangle pairs span overlapping and
        // separated 2D configurations so the EDGE_*/POINT_IN_TRI return-1 paths are hit.
        let cases: &[(vec3_t, [vec3_t; 3], [vec3_t; 3])] = &[
            // N largest on z -> project to xy; overlapping
            (
                [0.0, 0.0, 1.0],
                [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
                [[0.5, 0.5, 0.0], [1.5, 0.5, 0.0], [0.5, 1.5, 0.0]],
            ),
            // N largest on z -> project to xy; separated
            (
                [0.0, 0.0, 1.0],
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                [[5.0, 5.0, 0.0], [6.0, 5.0, 0.0], [5.0, 6.0, 0.0]],
            ),
            // N largest on x -> project to yz
            (
                [1.0, 0.0, 0.0],
                [[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]],
                [[0.0, 0.5, 0.5], [0.0, 1.5, 0.5], [0.0, 0.5, 1.5]],
            ),
            // N largest on y -> project to xz
            (
                [0.0, 1.0, 0.0],
                [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
                [[0.5, 0.0, 0.5], [1.5, 0.0, 0.5], [0.5, 0.0, 1.5]],
            ),
            // mixed normal, one tri fully inside the other (POINT_IN_TRI path)
            (
                [0.2, 0.3, 1.0],
                [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
                [[1.0, 1.0, 0.0], [1.2, 1.0, 0.0], [1.0, 1.2, 0.0]],
            ),
            // degenerate normal (all-zero) -> i0=0,i1=2 branch
            (
                [0.0, 0.0, 0.0],
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                [[0.5, 0.5, 0.0], [1.5, 0.5, 0.0], [0.5, 1.5, 0.0]],
            ),
        ];
        for (n, v, u) in cases {
            let r = coplanar_tri_tri(n, &v[0], &v[1], &v[2], &u[0], &u[1], &u[2]);
            let c = unsafe {
                oracle::coplanar_tri_tri(
                    n.as_ptr(),
                    v[0].as_ptr(),
                    v[1].as_ptr(),
                    v[2].as_ptr(),
                    u[0].as_ptr(),
                    u[1].as_ptr(),
                    u[2].as_ptr(),
                )
            };
            assert_eq!(r, c, "coplanar_tri_tri mismatch for N={n:?} V={v:?} U={u:?}");
        }
    }
}
