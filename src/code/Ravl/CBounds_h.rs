////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Vector Library
// --------------
//
//
//
//
// NOTES:
// 05/31/02 - CREATED
//
//
////////////////////////////////////////////////////////////////////////////////////////
// namespace ravl
// {

////////////////////////////////////////////////////////////////////////////////////////
// Includes
////////////////////////////////////////////////////////////////////////////////////////
// #include "CVec.h"

////////////////////////////////////////////////////////////////////////////////////////
// LOCAL STUBS FOR CVec3 AND ESide
// (CVec.h not yet ported; see oracle/code/Ravl/CVec.h for full defs)
////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CVec3 {
    pub v: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum ESide {
    Side_None = 0,
    Side_Left = 1,
    Side_Right = 2,
    Side_In = 3,
    Side_Out = 4,
    Side_AllIn = 5,
}

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
pub const RAVL_BB_EMPTY_MIN: f32 = 1.234567E30f32; // Empty Value
pub const RAVL_BB_EMPTY_MAX: f32 = -1.234567E30f32; // Empty Value

////////////////////////////////////////////////////////////////////////////////////////
// Enums And Typedefs
////////////////////////////////////////////////////////////////////////////////////////
pub type TPlanes = u8;

////////////////////////////////////////////////////////////////////////////////////////
// The Bounds Trace
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
pub struct CBTrace {
    ////////////////////////////////////////////////////////////////////////////////////
    // Setup Values, Do Not Change
    ////////////////////////////////////////////////////////////////////////////////////
    pub mStart: CVec3,
    pub mStop: CVec3,
    pub mAim: CVec3,
    pub mLength: f32,

    ////////////////////////////////////////////////////////////////////////////////////
    // Results
    ////////////////////////////////////////////////////////////////////////////////////
    pub mRange: f32,
    pub mRangeMax: f32,
    pub mPoint: CVec3,
    pub mNormal: CVec3,
}

impl CBTrace {
    pub fn new(Start: CVec3, Stop: CVec3) -> Self {
        let mut aim = CVec3 { v: [Stop.v[0], Stop.v[1], Stop.v[2]] };
        aim.v[0] -= Start.v[0];
        aim.v[1] -= Start.v[1];
        aim.v[2] -= Start.v[2];
        let length = aim.Norm();

        CBTrace {
            mStart: Start,
            mStop: Stop,
            mAim: aim,
            mRange: 0.0,
            mRangeMax: 0.0,
            mPoint: Stop,
            mNormal: CVec3 { v: [0.0, 0.0, 0.0] },
            mLength: length,
        }
    }

    pub fn operator_assign(&mut self, T: &CBTrace) -> &mut Self {
        self.mStart = T.mStart;
        self.mStop = T.mStop;
        self.mAim = T.mAim;
        self.mRange = T.mRange;
        self.mRangeMax = T.mRangeMax;
        self.mPoint = T.mPoint;
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Bounding Box Class
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
pub struct CBBox {
    ////////////////////////////////////////////////////////////////////////////////////
    // Data
    ////////////////////////////////////////////////////////////////////////////////////
    pub mMin: CVec3,
    pub mMax: CVec3,
}

impl CBBox {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        CBBox {
            mMin: CVec3 {
                v: [RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN],
            },
            mMax: CVec3 {
                v: [RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX],
            },
        }
    }

    pub fn from_radius(Radius: f32) -> Self {
        CBBox {
            mMin: CVec3 {
                v: [-Radius, -Radius, -Radius],
            },
            mMax: CVec3 {
                v: [Radius, Radius, Radius],
            },
        }
    }

    pub fn from_vec(t: CVec3) -> Self {
        CBBox { mMin: t, mMax: t }
    }

    pub fn from_bounds(min: CVec3, max: CVec3) -> Self {
        CBBox { mMin: min, mMax: max }
    }

    pub fn from_bbox(t: &CBBox) -> Self {
        CBBox {
            mMin: t.mMin,
            mMax: t.mMax,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initializers
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Set(&mut self, min: CVec3, max: CVec3) {
        self.mMin = min;
        self.mMax = max;
        self.Validate();
    }

    pub fn Clear(&mut self) {
        self.mMin = CVec3 {
            v: [RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN],
        };
        self.mMax = CVec3 {
            v: [RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX],
        };
    }

    pub fn AddPoint(&mut self, p: CVec3) {
        self.mMin.Min(&p);
        self.mMax.Max(&p);
        self.Validate();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn IsEmpty(&self) -> bool {
        self.mMin.v[0] == RAVL_BB_EMPTY_MIN
    }

    pub fn Center(&self) -> CVec3 {
        CVec3 {
            v: [
                (self.mMin.v[0] + self.mMax.v[0]) * 0.5,
                (self.mMin.v[1] + self.mMax.v[1]) * 0.5,
                (self.mMin.v[2] + self.mMax.v[2]) * 0.5,
            ],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Translation, Rotation, Expansion
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Translate(&mut self, f: CVec3) {
        self.mMin.v[0] += f.v[0];
        self.mMin.v[1] += f.v[1];
        self.mMin.v[2] += f.v[2];
        self.mMax.v[0] += f.v[0];
        self.mMax.v[1] += f.v[1];
        self.mMax.v[2] += f.v[2];
    }

    pub fn Expand(&mut self, x: f32) {
        self.mMin.v[0] -= x;
        self.mMin.v[1] -= x;
        self.mMin.v[2] -= x;
        self.mMax.v[0] += x;
        self.mMax.v[1] += x;
        self.mMax.v[2] += x;
    }

    pub fn Expand_vec(&mut self, f: CVec3) {
        self.mMin.v[0] -= f.v[0];
        self.mMin.v[1] -= f.v[1];
        self.mMin.v[2] -= f.v[2];
        self.mMax.v[0] += f.v[0];
        self.mMax.v[1] += f.v[1];
        self.mMax.v[2] += f.v[2];
    }
    //	void	ThroughMatrix(const CBBox &from, const CMatrix4 &mat);

    ////////////////////////////////////////////////////////////////////////////////////
    // Volumetric & Area Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Volume(&self) -> f32 {
        (self.mMax.v[0] - self.mMin.v[0])
            * (self.mMax.v[1] - self.mMin.v[1])
            * (self.mMax.v[2] - self.mMin.v[2])
    }

    pub fn AxisSize(&self, axis: i32) -> f32 {
        self.mMax.v[axis as usize] - self.mMin.v[axis as usize]
    }

    pub fn LargestAxisSize(&self) -> f32 {
        unimplemented!()
    }

    pub fn DistanceEstimate(&self, p: CVec3) -> f32 {
        // Manhattan Distance
        unimplemented!()
    }

    pub fn AreaEstimate(&self, p: CVec3) -> f32 {
        // Manhattan Distance * LargestAxisSize()
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Set Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Intersect(&mut self, b2: &CBBox) {
        unimplemented!()
    }

    pub fn Union(&mut self, b2: &CBBox) {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Tests
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn InOutTest(&self, p: CVec3) -> ESide {
        unimplemented!()
    }

    pub fn InOutTest_tolerance(&self, p: CVec3, tolout: f32, tolin: f32) -> ESide {
        unimplemented!()
    }

    pub fn BoxTouchTest(&self, b2: &CBBox, tolout: f32) -> bool {
        unimplemented!()
    }

    pub fn SphereTouchTest(&self, c: CVec3, rad: f32) -> bool {
        unimplemented!()
    }

    pub fn PlaneFlags(&mut self, p: CVec3) -> TPlanes {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Hit Tests
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn HitTest(&self, Tr: &mut CBTrace) -> bool {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // String Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: *const i8) {
        unimplemented!()
    }

    pub fn ToStr(&self, s: *mut i8) {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Debug Operations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn Validate(&mut self) {
        unimplemented!()
    }
}

// LOCAL HELPER IMPL FOR CVec3 (STUB)
impl CVec3 {
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

    pub fn Norm(&self) -> f32 {
        (self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]).sqrt()
    }
}

// };
