////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Vector Library
// --------------
// The base implimention of the Raven Vector object attempts to solve a number of
// high level problems as efficiently as possible.  Where ever feasible, functions have
// been included in the .h file so the compiler can inline them.
//
// The vectors define the following operations:
//  - Construction
//  - Initialization
//  - Member Access
//  - Equality / Inequality Operators
//  - Arithimitic Operators
//  - Length & Distance
//  - Normalization (Standard, Safe, Angular)
//  - Dot & Cross Product
//  - Perpendicular Vector
//  - Truncation
//  - Min & Max Element Analisis
//  - Interpolation
//  - Angle / Vector Conversion
//  - Translation & Rotation
//  - Point and Line Intersection Tests
//  - Left / Right Line Test
//  - String Operations
//  - Debug Routines
//  - "Standard" Vectors As Static Memebers
//
// As necessary, some projects may #define special faster versions of these routines to
// make better use of native hardware / software implimentations.
//
//
//
//
// NOTES:
// 05/29/02 - CREATED
// 05/30/02 - RotatePoint() is currently unimplimented.  Waiting for Matrix Library
//
//
////////////////////////////////////////////////////////////////////////////////////////
#![allow(non_snake_case)]

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
pub const RAVL_VEC_UDF: f32 = 1.234567E-10f32; // Undefined Vector Value (for debugging)
pub const RAVL_VEC_PI: f32 = 3.1415926535f32; // Pi
pub const RAVL_VEC_DEGTORADCONST: f32 = 0.0174532925f32; // (RAVL_VEC_PI / 180.0f)
pub const RAVL_VEC_RADTODEGCONST: f32 = 57.295779514f32; // (180.0f / RAVL_VEC_PI)

/// Quick Macro For Degrees -> Radians
#[inline]
pub const fn RAVL_VEC_DEGTORAD(a: f32) -> f32 {
    a * RAVL_VEC_DEGTORADCONST
}

/// Quick Macro For Radians -> Degrees
#[inline]
pub const fn RAVL_VEC_RADTODEG(a: f32) -> f32 {
    a * RAVL_VEC_RADTODEGCONST
}

////////////////////////////////////////////////////////////////////////////////////////
// Template Functions
////////////////////////////////////////////////////////////////////////////////////////
/// Generic Min function for comparable types
#[inline]
pub fn Min<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// Generic Max function for comparable types
#[inline]
pub fn Max<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if b < a { a } else { b }
}

////////////////////////////////////////////////////////////////////////////////////////
// Enums And Typedefs
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ESide {
    Side_None = 0,
    Side_Left = 1,
    Side_Right = 2,
    Side_In = 3,
    Side_Out = 4,
    Side_AllIn = 5,
}

////////////////////////////////////////////////////////////////////////////////////////
// The 4 Dimensional Vector
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CVec4 {
    v: [f32; 4],
}

impl CVec4 {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn new() -> Self {
        CVec4 { v: [0.0; 4] }
    }

    #[inline]
    pub fn from_scalar(val: f32) -> Self {
        CVec4 {
            v: [val, val, val, val],
        }
    }

    #[inline]
    pub fn from_components(x: f32, y: f32, z: f32, r: f32) -> Self {
        CVec4 { v: [x, y, z, r] }
    }

    #[inline]
    pub fn from_cvec4(t: &CVec4) -> Self {
        CVec4 { v: t.v }
    }

    #[inline]
    pub fn from_ptr(t: *const f32) -> Self {
        unsafe {
            CVec4 {
                v: [*t, *t.add(1), *t.add(2), *t.add(3)],
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initializers
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Set(&mut self, t: f32) {
        self.v[0] = t;
        self.v[1] = t;
        self.v[2] = t;
        self.v[3] = t;
    }

    #[inline]
    pub fn Set_ptr(&mut self, t: *const f32) {
        unsafe {
            self.v[0] = *t;
            self.v[1] = *t.add(1);
            self.v[2] = *t.add(2);
            self.v[3] = *t.add(3);
        }
    }

    #[inline]
    pub fn Set_components(&mut self, x: f32, y: f32, z: f32, r: f32) {
        self.v[0] = x;
        self.v[1] = y;
        self.v[2] = z;
        self.v[3] = r;
    }

    #[inline]
    pub fn Clear(&mut self) {
        self.v[0] = 0.0;
        self.v[1] = 0.0;
        self.v[2] = 0.0;
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Member Accessors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn index(&self, i: usize) -> f32 {
        self.v[i]
    }

    #[inline]
    pub fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.v[i]
    }

    #[inline]
    pub fn pitch(&mut self) -> &mut f32 {
        &mut self.v[0]
    }

    #[inline]
    pub fn yaw(&mut self) -> &mut f32 {
        &mut self.v[1]
    }

    #[inline]
    pub fn roll(&mut self) -> &mut f32 {
        &mut self.v[2]
    }

    #[inline]
    pub fn radius(&mut self) -> &mut f32 {
        &mut self.v[3]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Equality / Inequality Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn is_null(&self) -> bool {
        !(self.v[0] != 0.0 && self.v[1] != 0.0 && self.v[2] != 0.0 && self.v[3] != 0.0)
    }

    #[inline]
    pub fn eq(&self, t: &CVec4) -> bool {
        self.v[0] == t.v[0]
            && self.v[1] == t.v[1]
            && self.v[2] == t.v[2]
            && self.v[3] == t.v[3]
    }

    #[inline]
    pub fn ne(&self, t: &CVec4) -> bool {
        !(self.v[0] == t.v[0]
            && self.v[1] == t.v[1]
            && self.v[2] == t.v[2]
            && self.v[3] == t.v[3])
    }

    #[inline]
    pub fn lt(&self, t: &CVec4) -> bool {
        self.v[0] < t.v[0] && self.v[1] < t.v[1] && self.v[2] < t.v[2] && self.v[3] < t.v[3]
    }

    #[inline]
    pub fn gt(&self, t: &CVec4) -> bool {
        self.v[0] > t.v[0] && self.v[1] > t.v[1] && self.v[2] > t.v[2] && self.v[3] > t.v[3]
    }

    #[inline]
    pub fn le(&self, t: &CVec4) -> bool {
        self.v[0] <= t.v[0] && self.v[1] <= t.v[1] && self.v[2] <= t.v[2] && self.v[3] <= t.v[3]
    }

    #[inline]
    pub fn ge(&self, t: &CVec4) -> bool {
        self.v[0] >= t.v[0] && self.v[1] >= t.v[1] && self.v[2] >= t.v[2] && self.v[3] >= t.v[3]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Basic Arithimitic Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn assign_scalar(&mut self, d: f32) {
        self.v[0] = d;
        self.v[1] = d;
        self.v[2] = d;
        self.v[3] = d;
    }

    #[inline]
    pub fn assign_vec(&mut self, t: &CVec4) {
        self.v[0] = t.v[0];
        self.v[1] = t.v[1];
        self.v[2] = t.v[2];
        self.v[3] = t.v[3];
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, d: f32) {
        self.v[0] += d;
        self.v[1] += d;
        self.v[2] += d;
        self.v[3] += d;
    }

    #[inline]
    pub fn add_assign_vec(&mut self, t: &CVec4) {
        self.v[0] += t.v[0];
        self.v[1] += t.v[1];
        self.v[2] += t.v[2];
        self.v[3] += t.v[3];
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, d: f32) {
        self.v[0] -= d;
        self.v[1] -= d;
        self.v[2] -= d;
        self.v[3] -= d;
    }

    #[inline]
    pub fn sub_assign_vec(&mut self, t: &CVec4) {
        self.v[0] -= t.v[0];
        self.v[1] -= t.v[1];
        self.v[2] -= t.v[2];
        self.v[3] -= t.v[3];
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, d: f32) {
        self.v[0] *= d;
        self.v[1] *= d;
        self.v[2] *= d;
        self.v[3] *= d;
    }

    #[inline]
    pub fn mul_assign_vec(&mut self, t: &CVec4) {
        self.v[0] *= t.v[0];
        self.v[1] *= t.v[1];
        self.v[2] *= t.v[2];
        self.v[3] *= t.v[3];
    }

    #[inline]
    pub fn div_assign_scalar(&mut self, d: f32) {
        self.v[0] /= d;
        self.v[1] /= d;
        self.v[2] /= d;
        self.v[3] /= d;
    }

    #[inline]
    pub fn div_assign_vec(&mut self, t: &CVec4) {
        self.v[0] /= t.v[0];
        self.v[1] /= t.v[1];
        self.v[2] /= t.v[2];
        self.v[3] /= t.v[3];
    }

    #[inline]
    pub fn add(&self, t: &CVec4) -> CVec4 {
        CVec4 {
            v: [
                self.v[0] + t.v[0],
                self.v[1] + t.v[1],
                self.v[2] + t.v[2],
                self.v[3] + t.v[3],
            ],
        }
    }

    #[inline]
    pub fn sub(&self, t: &CVec4) -> CVec4 {
        CVec4 {
            v: [
                self.v[0] - t.v[0],
                self.v[1] - t.v[1],
                self.v[2] - t.v[2],
                self.v[3] - t.v[3],
            ],
        }
    }

    #[inline]
    pub fn mul(&self, t: &CVec4) -> CVec4 {
        CVec4 {
            v: [
                self.v[0] * t.v[0],
                self.v[1] * t.v[1],
                self.v[2] * t.v[2],
                self.v[3] * t.v[3],
            ],
        }
    }

    #[inline]
    pub fn div(&self, t: &CVec4) -> CVec4 {
        CVec4 {
            v: [
                self.v[0] / t.v[0],
                self.v[1] / t.v[1],
                self.v[2] / t.v[2],
                self.v[3] / t.v[3],
            ],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Length And Distance Calculations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Len(&self) -> f32;

    #[inline]
    pub fn Len2(&self) -> f32 {
        self.v[0] * self.v[0]
            + self.v[1] * self.v[1]
            + self.v[2] * self.v[2]
            + self.v[3] * self.v[3]
    }

    pub fn Dist(&self, t: &CVec4) -> f32;

    #[inline]
    pub fn Dist2(&self, t: &CVec4) -> f32 {
        (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
            + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
            + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
            + (t.v[3] - self.v[3]) * (t.v[3] - self.v[3])
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Normalization
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Norm(&mut self) -> f32;
    pub fn SafeNorm(&mut self) -> f32;
    pub fn AngleNorm(&mut self);

    ////////////////////////////////////////////////////////////////////////////////////
    // Dot, Cross & Perpendicular Vector
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Dot(&self, t: &CVec4) -> f32 {
        self.v[0] * t.v[0] + self.v[1] * t.v[1] + self.v[2] * t.v[2] + self.v[3] * t.v[3]
    }

    pub fn Cross(&mut self, t: &CVec4) {
        let temp = *self;
        self.v[0] = temp.v[1] * t.v[2] - temp.v[2] * t.v[1];
        self.v[1] = temp.v[2] * t.v[0] - temp.v[0] * t.v[2];
        self.v[2] = temp.v[0] * t.v[1] - temp.v[1] * t.v[0];
        self.v[3] = 0.0;
    }

    pub fn Perp(&mut self);

    ////////////////////////////////////////////////////////////////////////////////////
    // Truncation & Element Analysis
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Min(&mut self, t: &CVec4) {
        if t.v[0] < self.v[0] {
            self.v[0] = t.v[0];
        }
        if t.v[1] < self.v[1] {
            self.v[1] = t.v[1];
        }
        if t.v[2] < self.v[2] {
            self.v[2] = t.v[2];
        }
        if t.v[3] < self.v[3] {
            self.v[3] = t.v[3];
        }
    }

    pub fn Max(&mut self, t: &CVec4) {
        if t.v[0] > self.v[0] {
            self.v[0] = t.v[0];
        }
        if t.v[1] > self.v[1] {
            self.v[1] = t.v[1];
        }
        if t.v[2] > self.v[2] {
            self.v[2] = t.v[2];
        }
        if t.v[3] > self.v[3] {
            self.v[3] = t.v[3];
        }
    }

    #[inline]
    pub fn MaxElement(&self) -> f32 {
        self.v[self.MaxElementIndex()]
    }

    pub fn MaxElementIndex(&self) -> usize;

    ////////////////////////////////////////////////////////////////////////////////////
    // Interpolation
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Interp(&mut self, v1: &CVec4, v2: &CVec4, t: f32) {
        *self = *v1;
        self.sub_assign_vec(v2);
        self.mul_assign_scalar(t);
        self.add_assign_vec(v2);
    }

    pub fn ScaleAdd(&mut self, t: &CVec4, scale: f32) {
        self.v[0] += scale * t.v[0];
        self.v[1] += scale * t.v[1];
        self.v[2] += scale * t.v[2];
        self.v[3] += scale * t.v[3];
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Angle To Vector (Angle In Degrees)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAng(&mut self);
    pub fn AngToVec(&mut self);
    pub fn AngToVec_with_dirs(&mut self, Right: &mut CVec4, Up: &mut CVec4);

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Angle To Vector (Angle In Radians)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAngRad(&mut self);
    pub fn AngToVecRad(&mut self);
    pub fn AngToVecRad_with_dirs(&mut self, Right: &mut CVec4, Up: &mut CVec4);

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Between Radians And Degrees
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ToRadians(&mut self);
    pub fn ToDegrees(&mut self);

    ////////////////////////////////////////////////////////////////////////////////////
    // Project
    //
    // Standard projection function.  Take the (this) and project it onto the vector
    // (U).  Imagine drawing a line perpendicular to U from the endpoint of the (this)
    // Vector.  That then becomes the new vector.
    //
    // The value returned is the scale of the new vector with respect to the one passed
    // to the function.  If the scale is less than (1.0) then the new vector is shorter
    // than (U).  If the scale is negative, then the vector is going in the opposite
    // direction of (U).
    //
    //               _  (U)
    //               /|
    //             /                                        _ (this)
    //           /                      RESULTS->           /|
    //         /                                          /
    //       /    __\ (this)                            /
    //     /___---  /                                 /
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Project(&mut self, U: &CVec4) -> f32 {
        let Scale = self.Dot(U) / U.Len2(); // Find the scale of this vector on U
        *self = *U; // Copy U onto this vector
        self.mul_assign_scalar(Scale); // Use the previously calculated scale to get the right length.
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Project To Line
    //
    // This function takes two other points in space as the start and end of a line
    // segment and projects the (this) point onto the line defined by (Start)->(Stop)
    //
    // RETURN VALUES:
    //   (-INF, 0.0)  : (this) landed on the line before (Start)
    //   (0.0, 1.0)   : (this) landed in the line segment between (Start) and (Stop)
    //   (1.0, INF)   : (this) landed on the line beyond (End)
    //
    //             (Stop)
    //               /
    //             /
    //           o _
    //         /  |\
    //       /     (this)
    //     /
    // (Start)
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLine(&mut self, Start: &CVec4, Stop: &CVec4) -> f32 {
        *self = self.sub(Start);
        let Scale = self.Project(&Stop.sub(Start));
        *self = self.add(Start);
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Project To Line Seg
    //
    // Same As Project To Line, Except It Will Clamp To Start And Stop
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLineSeg(&mut self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let Scale = self.ProjectToLine(Start, Stop);
        if Scale < 0.0f32 {
            *self = *Start;
        } else if Scale > 1.0f32 {
            *self = *Stop;
        }
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    //
    // Uses project to line and than calculates distance to the new point
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine(&self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let mut P = *self;
        P.ProjectToLineSeg(Start, Stop);

        self.Dist(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    //
    // Uses project to line and than calculates distance to the new point
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine2(&self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let mut P = *self;
        P.ProjectToLineSeg(Start, Stop);

        self.Dist2(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Translation & Rotation (2D)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn RotatePoint(&mut self, Angle: &CVec4, Origin: &CVec4);
    pub fn Reposition(&mut self, Translation: &CVec4, RotationDegrees: f32);

    ////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Parallel Pipid (2D)
    //
    // Given two more points, this function calculates the area of the parallel pipid
    // formed.
    //
    // Note: This function CAN return a negative "area" if (this) is above or right of
    // (A) and (B)...  We do not take the abs because the sign of the "area" is needed
    // for the left right test (see below)
    //
    //
    //               ___---( ... )
    //        (A)---/        /
    //        /             /
    //       /             /
    //      /             /
    //     /      ___---(B)
    //  (this)---/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaParallelPipid(&self, A: &CVec4, B: &CVec4) -> f32 {
        (A.v[0] * B.v[1] - A.v[1] * B.v[0])
            + (B.v[0] * self.v[1] - self.v[0] * B.v[1])
            + (self.v[0] * A.v[1] - A.v[0] * self.v[1])
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Triangle (2D)
    //
    // Given two more points, this function calculates the area of the triangle formed.
    //
    //        (A)
    //        /  \__
    //       /      \__
    //      /          \_
    //     /      ___---(B)
    //  (this)---/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaTriange(&self, A: &CVec4, B: &CVec4) -> f32 {
        self.AreaParallelPipid(A, B) * 0.5f32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Left Right Test (2D)
    //
    // Given a line segment (Start->End) and a tolerance for *right on*, this function
    // evaluates which side the point is of the line.  (Side_Left in this example)
    //
    //
    //
    //          (this)        ___---/(End)
    //                 ___---/
    //          ___---/
    //  (Start)/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn LRTest(&self, Start: &CVec4, End: &CVec4, Tolerance: f32) -> ESide {
        let Area = self.AreaParallelPipid(Start, End);
        if Area > Tolerance {
            return ESide::Side_Left;
        }
        if Area < (Tolerance * -1.0) {
            return ESide::Side_Right;
        }
        ESide::Side_None
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Point In Circumscribed Circle  (True/False)
    //
    //  Returns true if the given point is within the circumscribed
    //  circle of the given ABC Triangle:
    //         _____
    //        /   B \
    //      /   /   \ \
    //     |  /      \ |
    //     |A---------C|
    //      \    Pt   /
    //       \_______/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle_triangle(&self, A: &CVec4, B: &CVec4, C: &CVec4) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // Point In Standard Circle  (True/False)
    //
    //  Returns true if the given point is within the Circle
    //         _____
    //        /     \
    //      /         \
    //     |   Circle  |
    //     |           |
    //      \    Pt   /
    //       \_______/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle(&self, Circle: &CVec4, Radius: f32) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // Line Intersects Circle  (True/False)
    //
    //  r	- Radius Of The Circle
    //  A	- Start Of Line Segment
    //  B	- End Of Line Segment
    //
    //  P	- Projected Position Of Origin Onto Line AB
    //
    //
    //            (Stop)
    //              /
    //            /
    //         (P)
    //        /   \      \
    //      /   (this)-r->|
    //    /              /
    // (Start)
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle(&self, Start: &CVec4, Stop: &CVec4, Radius: f32) -> bool;
    pub fn LineInCircle_with_point(
        &self,
        Start: &CVec4,
        Stop: &CVec4,
        Radius: f32,
        PointOnLine: &mut CVec4,
    ) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // String Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: *const c_char);
    pub fn ToStr(&self, s: *mut c_char);

    ////////////////////////////////////////////////////////////////////////////////////
    // Debug Routines
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn IsFinite(&self) -> bool;
    #[cfg(debug_assertions)]
    pub fn IsInitialized(&self) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////
// The 3 Dimensional Vector
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CVec3 {
    pub v: [f32; 3],
}

impl CVec3 {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn new() -> Self {
        CVec3 { v: [0.0; 3] }
    }

    #[inline]
    pub fn from_scalar(val: f32) -> Self {
        CVec3 { v: [val, val, val] }
    }

    #[inline]
    pub fn from_components(x: f32, y: f32, z: f32) -> Self {
        CVec3 { v: [x, y, z] }
    }

    #[inline]
    pub fn from_cvec3(t: &CVec3) -> Self {
        CVec3 { v: t.v }
    }

    #[inline]
    pub fn from_ptr(t: *const f32) -> Self {
        unsafe {
            CVec3 {
                v: [*t, *t.add(1), *t.add(2)],
            }
        }
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.v[0]
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.v[1]
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.v[2]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initializers
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Set(&mut self, t: f32) {
        self.v[0] = t;
        self.v[1] = t;
        self.v[2] = t;
    }

    #[inline]
    pub fn Set_ptr(&mut self, t: *const f32) {
        unsafe {
            self.v[0] = *t;
            self.v[1] = *t.add(1);
            self.v[2] = *t.add(2);
        }
    }

    #[inline]
    pub fn Set_components(&mut self, x: f32, y: f32, z: f32) {
        self.v[0] = x;
        self.v[1] = y;
        self.v[2] = z;
    }

    #[inline]
    pub fn Clear(&mut self) {
        self.v[0] = 0.0;
        self.v[1] = 0.0;
        self.v[2] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Member Accessors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn index(&self, i: usize) -> f32 {
        self.v[i]
    }

    #[inline]
    pub fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.v[i]
    }

    #[inline]
    pub fn pitch(&mut self) -> &mut f32 {
        &mut self.v[0]
    }

    #[inline]
    pub fn yaw(&mut self) -> &mut f32 {
        &mut self.v[1]
    }

    #[inline]
    pub fn roll(&mut self) -> &mut f32 {
        &mut self.v[2]
    }

    #[inline]
    pub fn radius(&mut self) -> &mut f32 {
        // Note: CVec3 has v[3] access in original but only 3 elements
        // This maintains C++ behavior but is dangerous - keeping as-is for faithfulness
        unsafe {
            let ptr = &mut self.v as *mut [f32; 3] as *mut f32;
            &mut *ptr.add(3)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Equality / Inequality Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn is_null(&self) -> bool {
        !(self.v[0] != 0.0 && self.v[1] != 0.0 && self.v[2] != 0.0)
    }

    #[inline]
    pub fn eq(&self, t: &CVec3) -> bool {
        self.v[0] == t.v[0] && self.v[1] == t.v[1] && self.v[2] == t.v[2]
    }

    #[inline]
    pub fn ne(&self, t: &CVec3) -> bool {
        !(self.v[0] == t.v[0] && self.v[1] == t.v[1] && self.v[2] == t.v[2])
    }

    #[inline]
    pub fn lt(&self, t: &CVec3) -> bool {
        self.v[0] < t.v[0] && self.v[1] < t.v[1] && self.v[2] < t.v[2]
    }

    #[inline]
    pub fn gt(&self, t: &CVec3) -> bool {
        self.v[0] > t.v[0] && self.v[1] > t.v[1] && self.v[2] > t.v[2]
    }

    #[inline]
    pub fn le(&self, t: &CVec3) -> bool {
        self.v[0] <= t.v[0] && self.v[1] <= t.v[1] && self.v[2] <= t.v[2]
    }

    #[inline]
    pub fn ge(&self, t: &CVec3) -> bool {
        self.v[0] >= t.v[0] && self.v[1] >= t.v[1] && self.v[2] >= t.v[2]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Basic Arithimitic Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn assign_scalar(&mut self, d: f32) {
        self.v[0] = d;
        self.v[1] = d;
        self.v[2] = d;
    }

    #[inline]
    pub fn assign_vec(&mut self, t: &CVec3) {
        self.v[0] = t.v[0];
        self.v[1] = t.v[1];
        self.v[2] = t.v[2];
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, d: f32) {
        self.v[0] += d;
        self.v[1] += d;
        self.v[2] += d;
    }

    #[inline]
    pub fn add_assign_vec(&mut self, t: &CVec3) {
        self.v[0] += t.v[0];
        self.v[1] += t.v[1];
        self.v[2] += t.v[2];
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, d: f32) {
        self.v[0] -= d;
        self.v[1] -= d;
        self.v[2] -= d;
    }

    #[inline]
    pub fn sub_assign_vec(&mut self, t: &CVec3) {
        self.v[0] -= t.v[0];
        self.v[1] -= t.v[1];
        self.v[2] -= t.v[2];
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, d: f32) {
        self.v[0] *= d;
        self.v[1] *= d;
        self.v[2] *= d;
    }

    #[inline]
    pub fn mul_assign_vec(&mut self, t: &CVec3) {
        self.v[0] *= t.v[0];
        self.v[1] *= t.v[1];
        self.v[2] *= t.v[2];
    }

    #[inline]
    pub fn div_assign_scalar(&mut self, d: f32) {
        self.v[0] /= d;
        self.v[1] /= d;
        self.v[2] /= d;
    }

    #[inline]
    pub fn div_assign_vec(&mut self, t: &CVec3) {
        self.v[0] /= t.v[0];
        self.v[1] /= t.v[1];
        self.v[2] /= t.v[2];
    }

    #[inline]
    pub fn add(&self, t: &CVec3) -> CVec3 {
        CVec3 {
            v: [
                self.v[0] + t.v[0],
                self.v[1] + t.v[1],
                self.v[2] + t.v[2],
            ],
        }
    }

    #[inline]
    pub fn sub(&self, t: &CVec3) -> CVec3 {
        CVec3 {
            v: [
                self.v[0] - t.v[0],
                self.v[1] - t.v[1],
                self.v[2] - t.v[2],
            ],
        }
    }

    #[inline]
    pub fn mul(&self, t: &CVec3) -> CVec3 {
        CVec3 {
            v: [
                self.v[0] * t.v[0],
                self.v[1] * t.v[1],
                self.v[2] * t.v[2],
            ],
        }
    }

    #[inline]
    pub fn div(&self, t: &CVec3) -> CVec3 {
        CVec3 {
            v: [
                self.v[0] / t.v[0],
                self.v[1] / t.v[1],
                self.v[2] / t.v[2],
            ],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Length And Distance Calculations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Len(&self) -> f32;

    #[inline]
    pub fn Len2(&self) -> f32 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]
    }

    pub fn Dist(&self, t: &CVec3) -> f32;

    #[inline]
    pub fn Dist2(&self, t: &CVec3) -> f32 {
        (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
            + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
            + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Normalization
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Norm(&mut self) -> f32;
    pub fn SafeNorm(&mut self) -> f32;
    pub fn AngleNorm(&mut self);
    pub fn Truncate(&mut self, maxlen: f32) -> f32;

    ////////////////////////////////////////////////////////////////////////////////////
    // Dot, Cross & Perpendicular Vector
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Dot(&self, t: &CVec3) -> f32 {
        self.v[0] * t.v[0] + self.v[1] * t.v[1] + self.v[2] * t.v[2]
    }

    pub fn Cross(&mut self, t: &CVec3) {
        let temp = *self;
        self.v[0] = temp.v[1] * t.v[2] - temp.v[2] * t.v[1];
        self.v[1] = temp.v[2] * t.v[0] - temp.v[0] * t.v[2];
        self.v[2] = temp.v[0] * t.v[1] - temp.v[1] * t.v[0];
    }

    pub fn Perp(&mut self);

    ////////////////////////////////////////////////////////////////////////////////////
    // Truncation & Element Analysis
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Min(&mut self, t: &CVec3) {
        if t.v[0] < self.v[0] {
            self.v[0] = t.v[0];
        }
        if t.v[1] < self.v[1] {
            self.v[1] = t.v[1];
        }
        if t.v[2] < self.v[2] {
            self.v[2] = t.v[2];
        }
    }

    pub fn Max(&mut self, t: &CVec3) {
        if t.v[0] > self.v[0] {
            self.v[0] = t.v[0];
        }
        if t.v[1] > self.v[1] {
            self.v[1] = t.v[1];
        }
        if t.v[2] > self.v[2] {
            self.v[2] = t.v[2];
        }
    }

    #[inline]
    pub fn MaxElement(&self) -> f32 {
        self.v[self.MaxElementIndex()]
    }

    pub fn MaxElementIndex(&self) -> usize;

    ////////////////////////////////////////////////////////////////////////////////////
    // Interpolation
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Interp(&mut self, v1: &CVec3, v2: &CVec3, t: f32) {
        *self = *v1;
        self.sub_assign_vec(v2);
        self.mul_assign_scalar(t);
        self.add_assign_vec(v2);
    }

    pub fn ScaleAdd(&mut self, t: &CVec3, scale: f32) {
        self.v[0] += scale * t.v[0];
        self.v[1] += scale * t.v[1];
        self.v[2] += scale * t.v[2];
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Angle To Vector (Angle In Degrees)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAng(&mut self);
    pub fn AngToVec(&mut self);
    pub fn AngToVec_with_dirs(&mut self, Right: &mut CVec3, Up: &mut CVec3);

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Angle To Vector (Angle In Radians)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAngRad(&mut self);
    pub fn AngToVecRad(&mut self);
    pub fn AngToVecRad_with_dirs(&mut self, Right: &mut CVec3, Up: &mut CVec3);

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion Between Radians And Degrees
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ToRadians(&mut self);
    pub fn ToDegrees(&mut self);

    ////////////////////////////////////////////////////////////////////////////////////
    // Project
    //
    // Standard projection function.  Take the (this) and project it onto the vector
    // (U).  Imagine drawing a line perpendicular to U from the endpoint of the (this)
    // Vector.  That then becomes the new vector.
    //
    // The value returned is the scale of the new vector with respect to the one passed
    // to the function.  If the scale is less than (1.0) then the new vector is shorter
    // than (U).  If the scale is negative, then the vector is going in the opposite
    // direction of (U).
    //
    //               _  (U)
    //               /|
    //             /                                        _ (this)
    //           /                      RESULTS->           /|
    //         /                                          /
    //       /    __\ (this)                            /
    //     /___---  /                                 /
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Project(&mut self, U: &CVec3) -> f32 {
        let Scale = self.Dot(U) / U.Len2(); // Find the scale of this vector on U
        *self = *U; // Copy U onto this vector
        self.mul_assign_scalar(Scale); // Use the previously calculated scale to get the right length.
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Project To Line
    //
    // This function takes two other points in space as the start and end of a line
    // segment and projects the (this) point onto the line defined by (Start)->(Stop)
    //
    // RETURN VALUES:
    //   (-INF, 0.0)  : (this) landed on the line before (Start)
    //   (0.0, 1.0)   : (this) landed in the line segment between (Start) and (Stop)
    //   (1.0, INF)   : (this) landed on the line beyond (End)
    //
    //             (Stop)
    //               /
    //             /
    //           o _
    //         /  |\
    //       /     (this)
    //     /
    // (Start)
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLine(&mut self, Start: &CVec3, Stop: &CVec3) -> f32 {
        *self = self.sub(Start);
        let Scale = self.Project(&Stop.sub(Start));
        *self = self.add(Start);
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Project To Line Seg
    //
    // Same As Project To Line, Except It Will Clamp To Start And Stop
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLineSeg(&mut self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let Scale = self.ProjectToLine(Start, Stop);
        if Scale < 0.0f32 {
            *self = *Start;
        } else if Scale > 1.0f32 {
            *self = *Stop;
        }
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    //
    // Uses project to line and than calculates distance to the new point
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine(&self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let mut P = *self;
        P.ProjectToLineSeg(Start, Stop);

        self.Dist(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    //
    // Uses project to line and than calculates distance to the new point
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine2(&self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let mut P = *self;
        P.ProjectToLineSeg(Start, Stop);

        self.Dist2(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Translation & Rotation (2D)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn RotatePoint(&mut self, Angle: &CVec3, Origin: &CVec3);
    pub fn Reposition(&mut self, Translation: &CVec3, RotationDegrees: f32);

    ////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Parallel Pipid (2D)
    //
    // Given two more points, this function calculates the area of the parallel pipid
    // formed.
    //
    // Note: This function CAN return a negative "area" if (this) is above or right of
    // (A) and (B)...  We do not take the abs because the sign of the "area" is needed
    // for the left right test (see below)
    //
    //
    //               ___---( ... )
    //        (A)---/        /
    //        /             /
    //       /             /
    //      /             /
    //     /      ___---(B)
    //  (this)---/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaParallelPipid(&self, A: &CVec3, B: &CVec3) -> f32 {
        (A.v[0] * B.v[1] - A.v[1] * B.v[0])
            + (B.v[0] * self.v[1] - self.v[0] * B.v[1])
            + (self.v[0] * A.v[1] - A.v[0] * self.v[1])
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Triangle (2D)
    //
    // Given two more points, this function calculates the area of the triangle formed.
    //
    //        (A)
    //        /  \__
    //       /      \__
    //      /          \_
    //     /      ___---(B)
    //  (this)---/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaTriange(&self, A: &CVec3, B: &CVec3) -> f32 {
        self.AreaParallelPipid(A, B) * 0.5f32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Left Right Test (2D)
    //
    // Given a line segment (Start->End) and a tolerance for *right on*, this function
    // evaluates which side the point is of the line.  (Side_Left in this example)
    //
    //
    //
    //          (this)        ___---/(End)
    //                 ___---/
    //          ___---/
    //  (Start)/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn LRTest(&self, Start: &CVec3, End: &CVec3, Tolerance: f32) -> ESide {
        let Area = self.AreaParallelPipid(Start, End);
        if Area > Tolerance {
            return ESide::Side_Left;
        }
        if Area < (Tolerance * -1.0) {
            return ESide::Side_Right;
        }
        ESide::Side_None
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Point In Circumscribed Circle  (True/False)
    //
    //  Returns true if the given point is within the circumscribed
    //  circle of the given ABC Triangle:
    //         _____
    //        /   B \
    //      /   /   \ \
    //     |  /      \ |
    //     |A---------C|
    //      \    Pt   /
    //       \_______/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle_triangle(&self, A: &CVec3, B: &CVec3, C: &CVec3) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // Point In Standard Circle  (True/False)
    //
    //  Returns true if the given point is within the Circle
    //         _____
    //        /     \
    //      /         \
    //     |   Circle  |
    //     |           |
    //      \    Pt   /
    //       \_______/
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle(&self, Circle: &CVec3, Radius: f32) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // Line Intersects Circle  (True/False)
    //
    //  r	- Radius Of The Circle
    //  A	- Start Of Line Segment
    //  B	- End Of Line Segment
    //
    //  P	- Projected Position Of Origin Onto Line AB
    //
    //
    //            (Stop)
    //              /
    //            /
    //         (P)
    //        /   \      \
    //      /   (this)-r->|
    //    /              /
    // (Start)
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle(&self, Start: &CVec3, Stop: &CVec3, Radius: f32) -> bool;
    pub fn LineInCircle_with_point(
        &self,
        Start: &CVec3,
        Stop: &CVec3,
        Radius: f32,
        PointOnLine: &mut CVec3,
    ) -> bool;

    ////////////////////////////////////////////////////////////////////////////////////
    // String Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: *const c_char);
    pub fn ToStr(&self, s: *mut c_char);

    ////////////////////////////////////////////////////////////////////////////////////
    // Debug Routines
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn IsFinite(&self) -> bool;
    #[cfg(debug_assertions)]
    pub fn IsInitialized(&self) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////
// Static constant vectors for CVec4
////////////////////////////////////////////////////////////////////////////////////////
pub static mut CVec4_mX: CVec4 = CVec4 { v: [1.0, 0.0, 0.0, 0.0] };
pub static mut CVec4_mY: CVec4 = CVec4 { v: [0.0, 1.0, 0.0, 0.0] };
pub static mut CVec4_mZ: CVec4 = CVec4 { v: [0.0, 0.0, 1.0, 0.0] };
pub static mut CVec4_mW: CVec4 = CVec4 { v: [0.0, 0.0, 0.0, 1.0] };
pub static mut CVec4_mZero: CVec4 = CVec4 { v: [0.0, 0.0, 0.0, 0.0] };

////////////////////////////////////////////////////////////////////////////////////////
// Static constant vectors for CVec3
////////////////////////////////////////////////////////////////////////////////////////
pub static mut CVec3_mX: CVec3 = CVec3 { v: [1.0, 0.0, 0.0] };
pub static mut CVec3_mY: CVec3 = CVec3 { v: [0.0, 1.0, 0.0] };
pub static mut CVec3_mZ: CVec3 = CVec3 { v: [0.0, 0.0, 1.0] };
pub static mut CVec3_mZero: CVec3 = CVec3 { v: [0.0, 0.0, 0.0] };

use core::ffi::c_char;
