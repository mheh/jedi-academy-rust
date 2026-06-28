////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Vector Library
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::c_int;

//using namespace ravl;

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
const RAVL_VEC_UDF: f32 = 1.234567E-10;                              // Undefined Vector Value (for debugging)
const RAVL_VEC_PI: f32 = 3.1415926535;                              // Pi
const RAVL_VEC_DEGTORADCONST: f32 = 0.0174532925;                   // (RAVL_VEC_PI / 180.0f)
const RAVL_VEC_RADTODEGCONST: f32 = 57.295779514;                   // (180.0f / RAVL_VEC_PI)

// Quick Macro For Degrees -> Radians
fn RAVL_VEC_DEGTORAD(a: f32) -> f32 {
    a * RAVL_VEC_DEGTORADCONST
}

// Quick Macro For Radians -> Degrees
fn RAVL_VEC_RADTODEG(a: f32) -> f32 {
    a * RAVL_VEC_RADTODEGCONST
}

////////////////////////////////////////////////////////////////////////////////////////
// Enums And Typedefs
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy)]
pub struct CVec4 {
    pub v: [f32; 4],
}

impl CVec4 {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn new() -> Self {
        CVec4 {
            v: [0.0; 4],
        }
    }

    #[inline]
    pub fn from_val(val: f32) -> Self {
        CVec4 {
            v: [val, val, val, val],
        }
    }

    #[inline]
    pub fn from_xyzr(x: f32, y: f32, z: f32, r: f32) -> Self {
        CVec4 {
            v: [x, y, z, r],
        }
    }

    #[inline]
    pub fn from_cvec4(t: &CVec4) -> Self {
        CVec4 {
            v: [t.v[0], t.v[1], t.v[2], t.v[3]],
        }
    }

    #[inline]
    pub fn from_slice(t: &[f32]) -> Self {
        CVec4 {
            v: [t[0], t[1], t[2], t[3]],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initializers
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Set_val(&mut self, t: f32) {
        self.v[0] = t;
        self.v[1] = t;
        self.v[2] = t;
        self.v[3] = t;
    }

    #[inline]
    pub fn Set_slice(&mut self, t: &[f32]) {
        self.v[0] = t[0];
        self.v[1] = t[1];
        self.v[2] = t[2];
        self.v[3] = t[3];
    }

    #[inline]
    pub fn Set(&mut self, x: f32, y: f32, z: f32, r: f32) {
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

    #[inline]
    pub fn pitch_const(&self) -> f32 {
        self.v[0]
    }

    #[inline]
    pub fn yaw_const(&self) -> f32 {
        self.v[1]
    }

    #[inline]
    pub fn roll_const(&self) -> f32 {
        self.v[2]
    }

    #[inline]
    pub fn radius_const(&self) -> f32 {
        self.v[3]
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Length
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Len(&self) -> f32 {
        (self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2] + self.v[3] * self.v[3]).sqrt()
    }

    #[inline]
    pub fn Len2(&self) -> f32 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2] + self.v[3] * self.v[3]
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Other Point
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Dist(&self, t: &CVec4) -> f32 {
        (
            (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
                + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
                + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
                + (t.v[3] - self.v[3]) * (t.v[3] - self.v[3])
        )
        .sqrt()
    }

    #[inline]
    pub fn Dist2(&self, t: &CVec4) -> f32 {
        (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
            + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
            + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
            + (t.v[3] - self.v[3]) * (t.v[3] - self.v[3])
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Normalize
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Norm(&mut self) -> f32 {
        let L = self.Len();
        self.v[0] /= L;
        self.v[1] /= L;
        self.v[2] /= L;
        self.v[3] /= L;
        L
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Safe Normalize
    //  Do Not Normalize If Length Is Too Small
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn SafeNorm(&mut self) -> f32 {
        let d = self.Len();
        if d > 1E-10 {
            self.v[0] /= d;
            self.v[1] /= d;
            self.v[2] /= d;
            self.v[3] /= d;
            d
        } else {
            self.v[0] = 0.0;
            self.v[1] = 0.0;
            self.v[2] = 0.0;
            self.v[3] = 0.0;
            0.0
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Angular Normalize
    //  All floats Exist(-180, +180)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngleNorm(&mut self) {
        self.v[0] = self.v[0].rem_euclid(360.0);
        if self.v[0] < -180.0 {
            self.v[0] += 360.0;
        }
        if self.v[0] > 180.0 {
            self.v[0] -= 360.0;
        }

        self.v[1] = self.v[1].rem_euclid(360.0);
        if self.v[1] < -180.0 {
            self.v[1] += 360.0;
        }
        if self.v[1] > 180.0 {
            self.v[1] -= 360.0;
        }

        self.v[2] = self.v[2].rem_euclid(360.0);
        if self.v[2] < -180.0 {
            self.v[2] += 360.0;
        }
        if self.v[2] > 180.0 {
            self.v[2] -= 360.0;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Perpendicular Vector
    //
    // This implimentation is a bit slow, needs some optimization work...
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Perp(&mut self) {
        let mut r = CVec4::from_cvec4(self);
        r.Cross(&CVec4::mX);
        let mut rlen = r.Len();
        let mut t = CVec4::from_cvec4(self);
        t.Cross(&CVec4::mY);
        let mut tlen = t.Len();
        if tlen > rlen {
            r = t;
            rlen = tlen;
        }
        t = CVec4::from_cvec4(self);
        t.Cross(&CVec4::mZ);
        tlen = t.Len();
        if tlen > rlen {
            r = t;
            rlen = tlen;
        }
        *self = r;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Find Largest Element (Ignores 4th component for now)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn MaxElementIndex(&self) -> c_int {
        if self.v[0].abs() > self.v[1].abs() && self.v[0].abs() > self.v[2].abs() {
            return 0;
        }
        if self.v[1].abs() > self.v[2].abs() {
            return 1;
        }
        2
    }

    #[inline]
    pub fn MaxElement(&self) -> f32 {
        self.v[self.MaxElementIndex() as usize]
    }

    #[inline]
    pub fn Dot(&self, t: &CVec4) -> f32 {
        self.v[0] * t.v[0] + self.v[1] * t.v[1] + self.v[2] * t.v[2] + self.v[3] * t.v[3]
    }

    pub fn Cross(&mut self, t: &CVec4) {
        let temp = CVec4::from_cvec4(self);
        self.v[0] = (temp.v[1] * t.v[2]) - (temp.v[2] * t.v[1]);
        self.v[1] = (temp.v[2] * t.v[0]) - (temp.v[0] * t.v[2]);
        self.v[2] = (temp.v[0] * t.v[1]) - (temp.v[1] * t.v[0]);
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Min and Max
    ////////////////////////////////////////////////////////////////////////////////////////
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

    ////////////////////////////////////////////////////////////////////////////////////////
    // Interpolation
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Interp(&mut self, v1: &CVec4, v2: &CVec4, t: f32) {
        self.v[0] = v1.v[0];
        self.v[1] = v1.v[1];
        self.v[2] = v1.v[2];
        self.v[3] = v1.v[3];
        self.v[0] -= v2.v[0];
        self.v[1] -= v2.v[1];
        self.v[2] -= v2.v[2];
        self.v[3] -= v2.v[3];
        self.v[0] *= t;
        self.v[1] *= t;
        self.v[2] *= t;
        self.v[3] *= t;
        self.v[0] += v2.v[0];
        self.v[1] += v2.v[1];
        self.v[2] += v2.v[2];
        self.v[3] += v2.v[3];
    }

    pub fn ScaleAdd(&mut self, t: &CVec4, scale: f32) {
        self.v[0] += scale * t.v[0];
        self.v[1] += scale * t.v[1];
        self.v[2] += scale * t.v[2];
        self.v[3] += scale * t.v[3];
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To {Pitch, Yaw}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAng(&mut self) {
        let yaw: f32;
        let pitch: f32;

        if self.v[1] == 0.0 && self.v[0] == 0.0 {
            yaw = 0.0;
            pitch = if self.v[2] > 0.0 { 90.0 } else { 270.0 };
        } else {
            // Calculate Yaw
            //---------------
            if self.v[0] != 0.0 {
                yaw = RAVL_VEC_RADTODEG(self.v[1].atan2(self.v[0]));
                let yaw_mut = if yaw < 0.0 { yaw + 360.0 } else { yaw };
            } else {
                yaw = if self.v[1] > 0.0 { 90.0 } else { 270.0 };
            }

            // Calculate Pitch
            //-----------------
            let forward = (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt();
            pitch = RAVL_VEC_RADTODEG(self.v[2].atan2(forward));
            let pitch_mut = if pitch < 0.0 { pitch + 360.0 } else { pitch };
        }

        // Copy Over Current Vector
        //--------------------------
        self.v[0] = -pitch;
        self.v[1] = yaw;
        self.v[2] = 0.0;
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVec(&mut self) {
        let angle: f32 = self.v[1] * RAVL_VEC_DEGTORADCONST;
        let sy = angle.sin();
        let cy = angle.cos();
        let angle2: f32 = self.v[0] * RAVL_VEC_DEGTORADCONST;
        let sp = angle2.sin();
        let cp = angle2.cos();

        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw, Roll}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVec_with_right_up(&mut self, Right: &mut CVec4, Up: &mut CVec4) {
        let angle: f32 = self.v[1] * RAVL_VEC_DEGTORADCONST;
        let sy = angle.sin();
        let cy = angle.cos();
        let angle2: f32 = self.v[0] * RAVL_VEC_DEGTORADCONST;
        let sp = angle2.sin();
        let cp = angle2.cos();
        let angle3: f32 = self.v[2] * RAVL_VEC_DEGTORADCONST;
        let sr = angle3.sin();
        let cr = angle3.cos();

        // Forward Vector Is Stored Here
        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
        self.v[3] = 0.0;

        // Calculate Right
        Right.v[0] = (-1.0 * sr * sp * cy + -1.0 * cr * -sy);
        Right.v[1] = (-1.0 * sr * sp * sy + -1.0 * cr * cy);
        Right.v[2] = -1.0 * sr * cp;
        Right.v[3] = 0.0;

        // Calculate Up
        Up.v[0] = (cr * sp * cy + -sr * -sy);
        Up.v[1] = (cr * sp * sy + -sr * cy);
        Up.v[2] = cr * cp;
        Up.v[3] = 0.0;
    }

    ///////////////////////////////////////////////////////////////////////////////////////
    // Convert To {Pitch, Yaw}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAngRad(&mut self) {
        let yaw: f32;
        let pitch: f32;

        if self.v[1] == 0.0 && self.v[0] == 0.0 {
            yaw = 0.0;
            pitch = if self.v[2] > 0.0 {
                RAVL_VEC_PI * 0.5
            } else {
                RAVL_VEC_PI * 1.5
            };
        } else {
            // Calculate Yaw
            //---------------
            if self.v[0] != 0.0 {
                yaw = self.v[1].atan2(self.v[0]);
            } else {
                yaw = if self.v[1] > 0.0 {
                    RAVL_VEC_PI * 0.5
                } else {
                    RAVL_VEC_PI * 1.5
                };
            }

            // Calculate Pitch
            //-----------------
            let forward = (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt();
            pitch = self.v[2].atan2(forward);
        }

        // Copy Over Current Vector
        //--------------------------
        self.v[0] = -pitch;
        self.v[1] = yaw;
        self.v[2] = 0.0;
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVecRad(&mut self) {
        let sy = self.v[1].sin();
        let cy = self.v[1].cos();
        let sp = self.v[0].sin();
        let cp = self.v[0].cos();

        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
        self.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw, Roll}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVecRad_with_right_up(&mut self, Right: &mut CVec4, Up: &mut CVec4) {
        let sy = self.v[1].sin();
        let cy = self.v[1].cos();
        let sp = self.v[0].sin();
        let cp = self.v[0].cos();
        let sr = self.v[2].sin();
        let cr = self.v[2].cos();

        // Forward Vector Is Stored Here
        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
        self.v[3] = 0.0;

        // Calculate Right
        Right.v[0] = (-1.0 * sr * sp * cy + -1.0 * cr * -sy);
        Right.v[1] = (-1.0 * sr * sp * sy + -1.0 * cr * cy);
        Right.v[2] = -1.0 * sr * cp;
        Right.v[3] = 0.0;

        // Calculate Up
        Up.v[0] = (cr * sp * cy + -sr * -sy);
        Up.v[1] = (cr * sp * sy + -sr * cy);
        Up.v[2] = cr * cp;
        Up.v[3] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To Radians
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToRadians(&mut self) {
        self.v[0] = RAVL_VEC_DEGTORAD(self.v[0]);
        self.v[1] = RAVL_VEC_DEGTORAD(self.v[1]);
        self.v[2] = RAVL_VEC_DEGTORAD(self.v[2]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To Degrees
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToDegrees(&mut self) {
        self.v[0] = RAVL_VEC_RADTODEG(self.v[0]);
        self.v[1] = RAVL_VEC_RADTODEG(self.v[1]);
        self.v[2] = RAVL_VEC_RADTODEG(self.v[2]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Project(&mut self, U: &CVec4) -> f32 {
        let Scale = self.Dot(U) / U.Len2();
        self.v[0] = U.v[0];
        self.v[1] = U.v[1];
        self.v[2] = U.v[2];
        self.v[3] = U.v[3];
        self.v[0] *= Scale;
        self.v[1] *= Scale;
        self.v[2] *= Scale;
        self.v[3] *= Scale;
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLine(&mut self, Start: &CVec4, Stop: &CVec4) -> f32 {
        self.v[0] -= Start.v[0];
        self.v[1] -= Start.v[1];
        self.v[2] -= Start.v[2];
        self.v[3] -= Start.v[3];
        let diff = CVec4::from_xyzr(
            Stop.v[0] - Start.v[0],
            Stop.v[1] - Start.v[1],
            Stop.v[2] - Start.v[2],
            Stop.v[3] - Start.v[3],
        );
        let Scale = self.Project(&diff);
        self.v[0] += Start.v[0];
        self.v[1] += Start.v[1];
        self.v[2] += Start.v[2];
        self.v[3] += Start.v[3];
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project To Line Seg
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLineSeg(&mut self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let Scale = self.ProjectToLine(Start, Stop);
        if Scale < 0.0 {
            *self = CVec4::from_cvec4(Start);
        } else if Scale > 1.0 {
            *self = CVec4::from_cvec4(Stop);
        }
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine(&self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let mut P = CVec4::from_cvec4(self);
        P.ProjectToLineSeg(Start, Stop);
        self.Dist(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine2(&self, Start: &CVec4, Stop: &CVec4) -> f32 {
        let mut P = CVec4::from_cvec4(self);
        P.ProjectToLineSeg(Start, Stop);
        self.Dist2(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Parallel Pipid (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaParallelPipid(&self, A: &CVec4, B: &CVec4) -> f32 {
        (A.v[0] * B.v[1] - A.v[1] * B.v[0])
            + (B.v[0] * self.v[1] - self.v[0] * B.v[1])
            + (self.v[0] * A.v[1] - A.v[0] * self.v[1])
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Triangle (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaTriange(&self, A: &CVec4, B: &CVec4) -> f32 {
        self.AreaParallelPipid(A, B) * 0.5
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Left Right Test (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LRTest(&self, Start: &CVec4, End: &CVec4, Tolerance: f32) -> ESide {
        let Area = self.AreaParallelPipid(Start, End);
        if Area > Tolerance {
            ESide::Side_Left
        } else if Area < (Tolerance * -1.0) {
            ESide::Side_Right
        } else {
            ESide::Side_None
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Values From String
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: &str) {
        let parts: Vec<&str> = s.trim_matches(|c| c == '(' || c == ')').split_whitespace().collect();
        if parts.len() >= 4 {
            self.v[0] = parts[0].parse().unwrap_or(0.0);
            self.v[1] = parts[1].parse().unwrap_or(0.0);
            self.v[2] = parts[2].parse().unwrap_or(0.0);
            self.v[3] = parts[3].parse().unwrap_or(0.0);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Write Values To A String
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToStr(&self) -> String {
        format!(
            "({:.3} {:.3} {:.3} {:.3})",
            self.v[0], self.v[1], self.v[2], self.v[3]
        )
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Point In Circumscribed Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle_tri(&self, A: &CVec4, B: &CVec4, C: &CVec4) -> bool {
        let tolerance = 0.00000005;

        let ax = A.v[0];
        let ay = A.v[1];
        let az = ax * ax + ay * ay;
        let bx = B.v[0];
        let by = B.v[1];
        let bz = bx * bx + by * by;
        let cx = C.v[0];
        let cy = C.v[1];
        let cz = cx * cx + cy * cy;
        let dx = self.v[0];
        let dy = self.v[1];
        let dz = dx * dx + dy * dy;

        let bxdx = bx - dx;
        let bydy = by - dy;
        let bzdz = bz - dz;
        let cxdx = cx - dx;
        let cydy = cy - dy;
        let czdz = cz - dz;
        let vol = (az - dz) * (bxdx * cydy - bydy * cxdx)
            + (ay - dy) * (bzdz * cxdx - bxdx * czdz)
            + (ax - dx) * (bydy * czdz - bzdz * cydy);

        if vol > tolerance {
            true
        } else if vol < -1.0 * tolerance {
            false
        } else {
            false
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Point In Standard Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle(&self, Circle: &CVec4, Radius: f32) -> bool {
        self.Dist2(Circle) < (Radius * Radius)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Line Intersects Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle_with_p(&self, A: &CVec4, B: &CVec4, r: f32, P: &mut CVec4) -> bool {
        *P = CVec4::from_cvec4(self);
        let Scale = P.ProjectToLine(A, B);

        // If The Projected Position Is Not On The Line Segment,
        // Check If It Is Within Radius Of Endpoints A and B.
        //-------------------------------------------------------
        if Scale < 0.0 || Scale > 1.0 {
            return P.PtInCircle(A, r) || P.PtInCircle(B, r);
        }

        // Otherwise, Check To See If P Is Within The Radius Of This Circle
        //------------------------------------------------------------------
        P.PtInCircle(P, r)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Same As Test Above, Just Don't Bother Returning P
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle(&self, A: &CVec4, B: &CVec4, r: f32) -> bool {
        let mut P = CVec4::from_cvec4(self);
        let Scale = P.ProjectToLine(A, B);

        // If The Projected Position Is Not On The Line Segment,
        // Check If It Is Within Radius Of Endpoints A and B.
        //-------------------------------------------------------
        if Scale < 0.0 || Scale > 1.0 {
            return P.PtInCircle(A, r) || P.PtInCircle(B, r);
        }

        // Otherwise, Check To See If P Is Within The Radius Of This Circle
        //------------------------------------------------------------------
        P.PtInCircle(&P, r)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Rotate
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn RotatePoint(&mut self, _: &CVec4, _: &CVec4) {
        // TO DO: Use Matrix Code To Rotate
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Reposition
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Reposition(&mut self, Translation: &CVec4, RotationDegrees: f32) {
        // Apply Any Rotation First
        //--------------------------
        if RotationDegrees != 0.0 {
            let Old = CVec4::from_cvec4(self);
            let Rotation = RAVL_VEC_DEGTORAD(RotationDegrees);
            self.v[0] = Old.v[0] * Rotation.cos() - Old.v[1] * Rotation.sin();
            self.v[1] = Old.v[0] * Rotation.sin() + Old.v[1] * Rotation.cos();
        }

        // Now Apply Translation
        //-----------------------
        self.v[0] += Translation.v[0];
        self.v[1] += Translation.v[1];
        self.v[2] += Translation.v[2];
        self.v[3] += Translation.v[3];
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Static Class Member Initialization
////////////////////////////////////////////////////////////////////////////////////////
impl CVec4 {
    pub const mX: CVec4 = CVec4 {
        v: [1.0, 0.0, 0.0, 0.0],
    };
    pub const mY: CVec4 = CVec4 {
        v: [0.0, 1.0, 0.0, 0.0],
    };
    pub const mZ: CVec4 = CVec4 {
        v: [0.0, 0.0, 1.0, 0.0],
    };
    pub const mW: CVec4 = CVec4 {
        v: [0.0, 0.0, 0.0, 1.0],
    };
    pub const mZero: CVec4 = CVec4 {
        v: [0.0, 0.0, 0.0, 0.0],
    };
}

////////////////////////////////////////////////////////////////////////////////////////
// The 3 Dimensional Vector
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    pub fn from_val(val: f32) -> Self {
        CVec3 {
            v: [val, val, val],
        }
    }

    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        CVec3 { v: [x, y, z] }
    }

    #[inline]
    pub fn from_cvec3(t: &CVec3) -> Self {
        CVec3 {
            v: [t.v[0], t.v[1], t.v[2]],
        }
    }

    #[inline]
    pub fn from_slice(t: &[f32]) -> Self {
        CVec3 {
            v: [t[0], t[1], t[2]],
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
    pub fn Set_val(&mut self, t: f32) {
        self.v[0] = t;
        self.v[1] = t;
        self.v[2] = t;
    }

    #[inline]
    pub fn Set_slice(&mut self, t: &[f32]) {
        self.v[0] = t[0];
        self.v[1] = t[1];
        self.v[2] = t[2];
    }

    #[inline]
    pub fn Set(&mut self, x: f32, y: f32, z: f32) {
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
        // Note: CVec3 only has 3 components, but we keep this for compatibility
        &mut self.v[2]
    }

    #[inline]
    pub fn pitch_const(&self) -> f32 {
        self.v[0]
    }

    #[inline]
    pub fn yaw_const(&self) -> f32 {
        self.v[1]
    }

    #[inline]
    pub fn roll_const(&self) -> f32 {
        self.v[2]
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Length
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Len(&self) -> f32 {
        (self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]).sqrt()
    }

    #[inline]
    pub fn Len2(&self) -> f32 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Other Point
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Dist(&self, t: &CVec3) -> f32 {
        (
            (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
                + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
                + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
        )
        .sqrt()
    }

    #[inline]
    pub fn Dist2(&self, t: &CVec3) -> f32 {
        (t.v[0] - self.v[0]) * (t.v[0] - self.v[0])
            + (t.v[1] - self.v[1]) * (t.v[1] - self.v[1])
            + (t.v[2] - self.v[2]) * (t.v[2] - self.v[2])
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Normalize
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Norm(&mut self) -> f32 {
        let L = self.Len();
        self.v[0] /= L;
        self.v[1] /= L;
        self.v[2] /= L;
        L
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Safe Normalize
    //  Do Not Normalize If Length Is Too Small
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn SafeNorm(&mut self) -> f32 {
        let d = self.Len();
        if d > 1E-10 {
            self.v[0] /= d;
            self.v[1] /= d;
            self.v[2] /= d;
            d
        } else {
            self.v[0] = 0.0;
            self.v[1] = 0.0;
            self.v[2] = 0.0;
            0.0
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Angular Normalize
    //  All floats Exist(-180, +180)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngleNorm(&mut self) {
        self.v[0] = self.v[0].rem_euclid(360.0);
        if self.v[0] < -180.0 {
            self.v[0] += 360.0;
        }
        if self.v[0] > 180.0 {
            self.v[0] -= 360.0;
        }

        self.v[1] = self.v[1].rem_euclid(360.0);
        if self.v[1] < -180.0 {
            self.v[1] += 360.0;
        }
        if self.v[1] > 180.0 {
            self.v[1] -= 360.0;
        }

        self.v[2] = self.v[2].rem_euclid(360.0);
        if self.v[2] < -180.0 {
            self.v[2] += 360.0;
        }
        if self.v[2] > 180.0 {
            self.v[2] -= 360.0;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Angular Normalize
    //  All floats Exist(-180, +180)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Truncate(&mut self, maxlen: f32) -> f32 {
        let len = self.Len();
        if len > maxlen && len > 1E-10 {
            let scale = maxlen / len;
            self.v[0] *= scale;
            self.v[1] *= scale;
            self.v[2] *= scale;
            maxlen
        } else {
            len
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Perpendicular Vector
    //
    // This implimentation is a bit slow, needs some optimization work...
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Perp(&mut self) {
        let mut r = CVec3::from_cvec3(self);
        r.Cross(&CVec3::mX);
        let mut rlen = r.Len();
        let mut t = CVec3::from_cvec3(self);
        t.Cross(&CVec3::mY);
        let mut tlen = t.Len();
        if tlen > rlen {
            r = t;
            rlen = tlen;
        }
        t = CVec3::from_cvec3(self);
        t.Cross(&CVec3::mZ);
        tlen = t.Len();
        if tlen > rlen {
            r = t;
            rlen = tlen;
        }
        *self = r;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Find Largest Element (Ignores 4th component for now)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn MaxElementIndex(&self) -> c_int {
        if self.v[0].abs() > self.v[1].abs() && self.v[0].abs() > self.v[2].abs() {
            return 0;
        }
        if self.v[1].abs() > self.v[2].abs() {
            return 1;
        }
        2
    }

    #[inline]
    pub fn MaxElement(&self) -> f32 {
        self.v[self.MaxElementIndex() as usize]
    }

    #[inline]
    pub fn Dot(&self, t: &CVec3) -> f32 {
        self.v[0] * t.v[0] + self.v[1] * t.v[1] + self.v[2] * t.v[2]
    }

    pub fn Cross(&mut self, t: &CVec3) {
        let temp = CVec3::from_cvec3(self);
        self.v[0] = (temp.v[1] * t.v[2]) - (temp.v[2] * t.v[1]);
        self.v[1] = (temp.v[2] * t.v[0]) - (temp.v[0] * t.v[2]);
        self.v[2] = (temp.v[0] * t.v[1]) - (temp.v[1] * t.v[0]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Min and Max
    ////////////////////////////////////////////////////////////////////////////////////////
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

    ////////////////////////////////////////////////////////////////////////////////////////
    // Interpolation
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Interp(&mut self, v1: &CVec3, v2: &CVec3, t: f32) {
        self.v[0] = v1.v[0];
        self.v[1] = v1.v[1];
        self.v[2] = v1.v[2];
        self.v[0] -= v2.v[0];
        self.v[1] -= v2.v[1];
        self.v[2] -= v2.v[2];
        self.v[0] *= t;
        self.v[1] *= t;
        self.v[2] *= t;
        self.v[0] += v2.v[0];
        self.v[1] += v2.v[1];
        self.v[2] += v2.v[2];
    }

    pub fn ScaleAdd(&mut self, t: &CVec3, scale: f32) {
        self.v[0] += scale * t.v[0];
        self.v[1] += scale * t.v[1];
        self.v[2] += scale * t.v[2];
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To {Pitch, Yaw}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAng(&mut self) {
        let yaw: f32;
        let pitch: f32;

        if self.v[1] == 0.0 && self.v[0] == 0.0 {
            yaw = 0.0;
            pitch = if self.v[2] > 0.0 { 90.0 } else { 270.0 };
        } else {
            // Calculate Yaw
            //---------------
            if self.v[0] != 0.0 {
                yaw = RAVL_VEC_RADTODEG(self.v[1].atan2(self.v[0]));
                let yaw_mut = if yaw < 0.0 { yaw + 360.0 } else { yaw };
            } else {
                yaw = if self.v[1] > 0.0 { 90.0 } else { 270.0 };
            }

            // Calculate Pitch
            //-----------------
            let forward = (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt();
            pitch = RAVL_VEC_RADTODEG(self.v[2].atan2(forward));
            let pitch_mut = if pitch < 0.0 { pitch + 360.0 } else { pitch };
        }

        // Copy Over Current Vector
        //--------------------------
        self.v[0] = -pitch;
        self.v[1] = yaw;
        self.v[2] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVec(&mut self) {
        let angle: f32 = self.v[1] * RAVL_VEC_DEGTORADCONST;
        let sy = angle.sin();
        let cy = angle.cos();
        let angle2: f32 = self.v[0] * RAVL_VEC_DEGTORADCONST;
        let sp = angle2.sin();
        let cp = angle2.cos();

        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw, Roll}   (DEGREES)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVec_with_right_up(&mut self, Right: &mut CVec3, Up: &mut CVec3) {
        let angle: f32 = self.v[1] * RAVL_VEC_DEGTORADCONST;
        let sy = angle.sin();
        let cy = angle.cos();
        let angle2: f32 = self.v[0] * RAVL_VEC_DEGTORADCONST;
        let sp = angle2.sin();
        let cp = angle2.cos();
        let angle3: f32 = self.v[2] * RAVL_VEC_DEGTORADCONST;
        let sr = angle3.sin();
        let cr = angle3.cos();

        // Forward Vector Is Stored Here
        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;

        // Calculate Right
        Right.v[0] = (-1.0 * sr * sp * cy + -1.0 * cr * -sy);
        Right.v[1] = (-1.0 * sr * sp * sy + -1.0 * cr * cy);
        Right.v[2] = -1.0 * sr * cp;

        // Calculate Up
        Up.v[0] = (cr * sp * cy + -sr * -sy);
        Up.v[1] = (cr * sp * sy + -sr * cy);
        Up.v[2] = cr * cp;
    }

    ///////////////////////////////////////////////////////////////////////////////////////
    // Convert To {Pitch, Yaw}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn VecToAngRad(&mut self) {
        let yaw: f32;
        let pitch: f32;

        if self.v[1] == 0.0 && self.v[0] == 0.0 {
            yaw = 0.0;
            pitch = if self.v[2] > 0.0 {
                RAVL_VEC_PI * 0.5
            } else {
                RAVL_VEC_PI * 1.5
            };
        } else {
            // Calculate Yaw
            //---------------
            if self.v[0] != 0.0 {
                yaw = self.v[1].atan2(self.v[0]);
            } else {
                yaw = if self.v[1] > 0.0 {
                    RAVL_VEC_PI * 0.5
                } else {
                    RAVL_VEC_PI * 1.5
                };
            }

            // Calculate Pitch
            //-----------------
            let forward = (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt();
            pitch = self.v[2].atan2(forward);
        }

        // Copy Over Current Vector
        //--------------------------
        self.v[0] = -pitch;
        self.v[1] = yaw;
        self.v[2] = 0.0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVecRad(&mut self) {
        let sy = self.v[1].sin();
        let cy = self.v[1].cos();
        let sp = self.v[0].sin();
        let cp = self.v[0].cos();

        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert From {Picth, Yaw, Roll}   (RADIANS)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AngToVecRad_with_right_up(&mut self, Right: &mut CVec3, Up: &mut CVec3) {
        let sy = self.v[1].sin();
        let cy = self.v[1].cos();
        let sp = self.v[0].sin();
        let cp = self.v[0].cos();
        let sr = self.v[2].sin();
        let cr = self.v[2].cos();

        // Forward Vector Is Stored Here
        self.v[0] = cp * cy;
        self.v[1] = cp * sy;
        self.v[2] = -sp;

        // Calculate Right
        Right.v[0] = (-1.0 * sr * sp * cy + -1.0 * cr * -sy);
        Right.v[1] = (-1.0 * sr * sp * sy + -1.0 * cr * cy);
        Right.v[2] = -1.0 * sr * cp;

        // Calculate Up
        Up.v[0] = (cr * sp * cy + -sr * -sy);
        Up.v[1] = (cr * sp * sy + -sr * cy);
        Up.v[2] = cr * cp;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To Radians
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToRadians(&mut self) {
        self.v[0] = RAVL_VEC_DEGTORAD(self.v[0]);
        self.v[1] = RAVL_VEC_DEGTORAD(self.v[1]);
        self.v[2] = RAVL_VEC_DEGTORAD(self.v[2]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Convert To Degrees
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToDegrees(&mut self) {
        self.v[0] = RAVL_VEC_RADTODEG(self.v[0]);
        self.v[1] = RAVL_VEC_RADTODEG(self.v[1]);
        self.v[2] = RAVL_VEC_RADTODEG(self.v[2]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Project(&mut self, U: &CVec3) -> f32 {
        let Scale = self.Dot(U) / U.Len2();
        self.v[0] = U.v[0];
        self.v[1] = U.v[1];
        self.v[2] = U.v[2];
        self.v[0] *= Scale;
        self.v[1] *= Scale;
        self.v[2] *= Scale;
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLine(&mut self, Start: &CVec3, Stop: &CVec3) -> f32 {
        self.v[0] -= Start.v[0];
        self.v[1] -= Start.v[1];
        self.v[2] -= Start.v[2];
        let diff = CVec3::from_xyz(
            Stop.v[0] - Start.v[0],
            Stop.v[1] - Start.v[1],
            Stop.v[2] - Start.v[2],
        );
        let Scale = self.Project(&diff);
        self.v[0] += Start.v[0];
        self.v[1] += Start.v[1];
        self.v[2] += Start.v[2];
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Project To Line Seg
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ProjectToLineSeg(&mut self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let Scale = self.ProjectToLine(Start, Stop);
        if Scale < 0.0 {
            *self = CVec3::from_cvec3(Start);
        } else if Scale > 1.0 {
            *self = CVec3::from_cvec3(Stop);
        }
        Scale
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine(&self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let mut P = CVec3::from_cvec3(self);
        P.ProjectToLineSeg(Start, Stop);
        self.Dist(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Distance To Line
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn DistToLine2(&self, Start: &CVec3, Stop: &CVec3) -> f32 {
        let mut P = CVec3::from_cvec3(self);
        P.ProjectToLineSeg(Start, Stop);
        self.Dist2(&P)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Parallel Pipid (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaParallelPipid(&self, A: &CVec3, B: &CVec3) -> f32 {
        (A.v[0] * B.v[1] - A.v[1] * B.v[0])
            + (B.v[0] * self.v[1] - self.v[0] * B.v[1])
            + (self.v[0] * A.v[1] - A.v[0] * self.v[1])
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Area Of The Triangle (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaTriange(&self, A: &CVec3, B: &CVec3) -> f32 {
        self.AreaParallelPipid(A, B) * 0.5
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Left Right Test (2D)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LRTest(&self, Start: &CVec3, End: &CVec3, Tolerance: f32) -> ESide {
        let Area = self.AreaParallelPipid(Start, End);
        if Area > Tolerance {
            ESide::Side_Left
        } else if Area < (Tolerance * -1.0) {
            ESide::Side_Right
        } else {
            ESide::Side_None
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Values From String
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: &str) {
        let parts: Vec<&str> = s.trim_matches(|c| c == '(' || c == ')').split_whitespace().collect();
        if parts.len() >= 3 {
            self.v[0] = parts[0].parse().unwrap_or(0.0);
            self.v[1] = parts[1].parse().unwrap_or(0.0);
            self.v[2] = parts[2].parse().unwrap_or(0.0);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Write Values To A String
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToStr(&self) -> String {
        format!("({:.3} {:.3} {:.3})", self.v[0], self.v[1], self.v[2])
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Point In Circumscribed Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle_tri(&self, A: &CVec3, B: &CVec3, C: &CVec3) -> bool {
        let tolerance = 0.00000005;

        let ax = A.v[0];
        let ay = A.v[1];
        let az = ax * ax + ay * ay;
        let bx = B.v[0];
        let by = B.v[1];
        let bz = bx * bx + by * by;
        let cx = C.v[0];
        let cy = C.v[1];
        let cz = cx * cx + cy * cy;
        let dx = self.v[0];
        let dy = self.v[1];
        let dz = dx * dx + dy * dy;

        let bxdx = bx - dx;
        let bydy = by - dy;
        let bzdz = bz - dz;
        let cxdx = cx - dx;
        let cydy = cy - dy;
        let czdz = cz - dz;
        let vol = (az - dz) * (bxdx * cydy - bydy * cxdx)
            + (ay - dy) * (bzdz * cxdx - bxdx * czdz)
            + (ax - dx) * (bydy * czdz - bzdz * cydy);

        if vol > tolerance {
            true
        } else if vol < -1.0 * tolerance {
            false
        } else {
            false
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Point In Standard Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn PtInCircle(&self, Circle: &CVec3, Radius: f32) -> bool {
        self.Dist2(Circle) < (Radius * Radius)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Line Intersects Circle  (True/False)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle_with_p(&self, A: &CVec3, B: &CVec3, r: f32, P: &mut CVec3) -> bool {
        *P = CVec3::from_cvec3(self);
        let Scale = P.ProjectToLine(A, B);

        // If The Projected Position Is Not On The Line Segment,
        // Check If It Is Within Radius Of Endpoints A and B.
        //-------------------------------------------------------
        if Scale < 0.0 || Scale > 1.0 {
            return P.PtInCircle(A, r) || P.PtInCircle(B, r);
        }

        // Otherwise, Check To See If P Is Within The Radius Of This Circle
        //------------------------------------------------------------------
        P.PtInCircle(P, r)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Same As Test Above, Just Don't Bother Returning P
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn LineInCircle(&self, A: &CVec3, B: &CVec3, r: f32) -> bool {
        let mut P = CVec3::from_cvec3(self);
        let Scale = P.ProjectToLine(A, B);

        // If The Projected Position Is Not On The Line Segment,
        // Check If It Is Within Radius Of Endpoints A and B.
        //-------------------------------------------------------
        if Scale < 0.0 || Scale > 1.0 {
            return P.PtInCircle(A, r) || P.PtInCircle(B, r);
        }

        // Otherwise, Check To See If P Is Within The Radius Of This Circle
        //------------------------------------------------------------------
        P.PtInCircle(&P, r)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Rotate
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn RotatePoint(&mut self, _: &CVec3, _: &CVec3) {
        // TO DO: Use Matrix Code To Rotate
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Reposition
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Reposition(&mut self, Translation: &CVec3, RotationDegrees: f32) {
        // Apply Any Rotation First
        //--------------------------
        if RotationDegrees != 0.0 {
            let Old = CVec3::from_cvec3(self);
            let Rotation = RAVL_VEC_DEGTORAD(RotationDegrees);
            self.v[0] = Old.v[0] * Rotation.cos() - Old.v[1] * Rotation.sin();
            self.v[1] = Old.v[0] * Rotation.sin() + Old.v[1] * Rotation.cos();
        }

        // Now Apply Translation
        //-----------------------
        self.v[0] += Translation.v[0];
        self.v[1] += Translation.v[1];
        self.v[2] += Translation.v[2];
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Static Class Member Initialization
////////////////////////////////////////////////////////////////////////////////////////
impl CVec3 {
    pub const mX: CVec3 = CVec3 { v: [1.0, 0.0, 0.0] };
    pub const mY: CVec3 = CVec3 { v: [0.0, 1.0, 0.0] };
    pub const mZ: CVec3 = CVec3 { v: [0.0, 0.0, 1.0] };
    pub const mZero: CVec3 = CVec3 { v: [0.0, 0.0, 0.0] };
}
