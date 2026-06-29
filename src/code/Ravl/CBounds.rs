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

use core::ffi::c_char;
use super::CBounds_h::{CBBox, CBTrace, CVec3, ESide, TPlanes, RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MAX};

extern "C" {
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> i32;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> i32;
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
// /*void CBBox::ThroughMatrix(const CBBox &from, const CMatrix4 &mat)
// {
// 	Clear();
// 	CVec3 bb,t;
// 	int i;
// 	const CVec3 &xmn=from.GetMin();
// 	const CVec3 &xmx=from.GetMax();
// 	for ( i = 0; i < 8; i++ )
// 	{
// 		if ( i & 1 )
// 			bb[0] = xmn[0];
// 		else
// 			bb[0] = xmx[0];
// 		if ( i & 2 )
// 			bb[1] = xmn[1];
// 		else
// 			bb[1] = xmx[1];
// 		if ( i & 4 )
// 			bb[2] = xmn[2];
// 		else
// 			bb[2] = xmx[2];
// 		mat.XFormPoint(t,bb);
// 		AddPoint(t);
// 	}
// }*/

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
impl CBBox {
    pub fn LargestAxisSize(&self) -> f32 {
        let mut Work = CVec3 { v: [self.mMax.v[0], self.mMax.v[1], self.mMax.v[2]] };
        Work.v[0] -= self.mMin.v[0];
        Work.v[1] -= self.mMin.v[1];
        Work.v[2] -= self.mMin.v[2];
        Work.MaxElement()
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn DistanceEstimate(&self, p: CVec3) -> f32 {
        let mut ret = 0.0f32;

        // X Axis
        //--------
        if p.v[0] > self.mMax.v[0] {
            ret = p.v[0] - self.mMax.v[0];
        } else if p.v[0] < self.mMin.v[0] {
            ret = self.mMax.v[0] - p.v[0];
        }

        // Y Axis
        //--------
        if p.v[1] > self.mMax.v[1] {
            ret += p.v[1] - self.mMax.v[1];
        } else if p.v[1] < self.mMin.v[1] {
            ret += self.mMax.v[1] - p.v[1];
        }

        // Z Axis
        //--------
        if p.v[2] > self.mMax.v[2] {
            ret += p.v[2] - self.mMax.v[2];
        } else if p.v[2] < self.mMin.v[2] {
            ret += self.mMax.v[2] - p.v[2];
        }
        ret
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn AreaEstimate(&self, p: CVec3) -> f32 {
        let Distance = self.DistanceEstimate(p);
        if Distance != 0.0 {
            return self.LargestAxisSize() / Distance;
        }
        0.0
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Intersect(&mut self, b2: &CBBox) {
        self.mMin.Max(&b2.mMin);
        self.mMax.Min(&b2.mMax);
        self.Validate();
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Union(&mut self, b2: &CBBox) {
        self.mMin.Min(&b2.mMin);
        self.mMax.Max(&b2.mMax);
        self.Validate();
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn InOutTest(&self, v: CVec3) -> ESide {
        if v.v[0] > self.mMin.v[0] && v.v[0] < self.mMax.v[0] &&
           v.v[1] > self.mMin.v[1] && v.v[1] < self.mMax.v[1] &&
           v.v[2] > self.mMin.v[2] && v.v[2] < self.mMax.v[2] {
            return ESide::Side_In;
        }
        ESide::Side_Out
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn InOutTest_tolerance(&self, v: CVec3, tolout: f32, tolin: f32) -> ESide {
        if v.v[0] < self.mMin.v[0] - tolout || v.v[0] > self.mMax.v[0] + tolout ||
           v.v[1] < self.mMin.v[1] - tolout || v.v[1] > self.mMax.v[1] + tolout ||
           v.v[2] < self.mMin.v[2] - tolout || v.v[2] > self.mMax.v[2] + tolout {
            return ESide::Side_Out;
        }
        if v.v[0] > self.mMin.v[0] + tolin && v.v[0] < self.mMax.v[0] - tolin &&
           v.v[1] > self.mMin.v[1] + tolin && v.v[1] < self.mMax.v[1] - tolin &&
           v.v[2] > self.mMin.v[2] + tolin && v.v[2] < self.mMax.v[2] - tolin {
            return ESide::Side_In;
        }
        ESide::Side_None
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn BoxTouchTest(&self, b2: &CBBox, tolout: f32) -> bool {
        if self.mMin.v[0] - tolout > b2.mMax.v[0] ||
           self.mMin.v[1] - tolout > b2.mMax.v[1] ||
           self.mMin.v[2] - tolout > b2.mMax.v[2] ||
           b2.mMin.v[0] - tolout > self.mMax.v[0] ||
           b2.mMin.v[1] - tolout > self.mMax.v[1] ||
           b2.mMin.v[2] - tolout > self.mMax.v[2] {
            return false;
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn SphereTouchTest(&self, v: CVec3, rad: f32) -> bool {
        if v.v[0] < self.mMin.v[0] - rad || v.v[0] > self.mMax.v[0] + rad ||
           v.v[1] < self.mMin.v[1] - rad || v.v[1] > self.mMax.v[1] + rad ||
           v.v[2] < self.mMin.v[2] - rad || v.v[2] > self.mMax.v[2] + rad {
            return false;
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn PlaneFlags(&self, p: CVec3) -> TPlanes {
        let mut ret: TPlanes = 0;
        if p.v[0] < self.mMin.v[0] {
            ret |= 1;
        } else if p.v[0] > self.mMax.v[0] {
            ret |= 2;
        }
        if p.v[1] < self.mMin.v[1] {
            ret |= 4;
        } else if p.v[1] > self.mMax.v[1] {
            ret |= 8;
        }
        if p.v[2] < self.mMin.v[2] {
            ret |= 16;
        } else if p.v[2] > self.mMax.v[2] {
            ret |= 32;
        }
        ret
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    // return true if the segment intersect the box, in that case, return the first contact.
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn HitTest(&self, Tr: &mut CBTrace) -> bool {
        // Quick Box Cull
        //----------------
        let mut tmp = CBBox {
            mMin: CVec3 {
                v: [RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN, RAVL_BB_EMPTY_MIN],
            },
            mMax: CVec3 {
                v: [RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX, RAVL_BB_EMPTY_MAX],
            },
        };
        tmp.AddPoint(Tr.mStart);
        tmp.AddPoint(Tr.mStop);
        if !self.BoxTouchTest(&tmp, 0.0) {
            return false;
        }

        // Initialize Our Ranges
        //-----------------------
        Tr.mRange = -1E30f32;
        Tr.mRangeMax = 1E30f32;

        // For Each Non Zero Axis Of The Aim Vector
        //------------------------------------------
        let mut tmax: f32;
        let mut tmin: f32;
        let mut temp: f32;
        for axis in 0..3 {
            if Tr.mAim.v[axis].abs() > 1E-6f32 {
                // Find Mins And Maxs From The Start Along The Axis Of Aim
                //---------------------------------------------------------
                tmax = (self.mMax.v[axis] - Tr.mStart.v[axis]) / Tr.mAim.v[axis];
                tmin = (self.mMin.v[axis] - Tr.mStart.v[axis]) / Tr.mAim.v[axis];
                if tmax < tmin {
                    temp = tmax;
                    tmax = tmin;
                    tmin = temp;
                }

                // Adjust Range Max
                //------------------
                if tmax < Tr.mRangeMax {
                    Tr.mRangeMax = tmax;
                }

                // Adjust Range Min
                //------------------
                if tmin > Tr.mRange {
                    Tr.mRange = tmin;
                    Tr.mNormal.v[0] = 0.0;
                    Tr.mNormal.v[1] = 0.0;
                    Tr.mNormal.v[2] = 0.0;
                    Tr.mNormal.v[axis] = -1.0f32;
                }
            }
        }

        // Missed?
        //---------
        if Tr.mRangeMax < Tr.mRange || Tr.mRangeMax < 0.0f32 || Tr.mRange > Tr.mLength {
            return false;
        }

        // Start Solid Conditions
        //------------------------
        if Tr.mRange < 0.0f32 {
            Tr.mRange = 0.0f32;
            Tr.mPoint = Tr.mStart;
            return true;
        }

        // Calculate The End Point
        //-------------------------
        Tr.mPoint.v[0] = Tr.mAim.v[0] * Tr.mRange + Tr.mStart.v[0];
        Tr.mPoint.v[1] = Tr.mAim.v[1] * Tr.mRange + Tr.mStart.v[1];
        Tr.mPoint.v[2] = Tr.mAim.v[2] * Tr.mRange + Tr.mStart.v[2];
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn FromStr(&mut self, s: *const c_char) {
        unsafe {
            // assert(s && s[0]);

            let mut MinS: [c_char; 256] = [0; 256];
            let mut MaxS: [c_char; 266] = [0; 266];
            sscanf(s, "(%s|%s)\0".as_ptr() as *const c_char, MinS.as_mut_ptr(), MaxS.as_mut_ptr());

            self.mMin.FromStr(MinS.as_ptr());
            self.mMax.FromStr(MaxS.as_ptr());
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ToStr(&self, s: *mut c_char) {
        unsafe {
            // assert(s && s[0]);

            let mut MinS: [c_char; 256] = [0; 256];
            let mut MaxS: [c_char; 266] = [0; 266];

            self.mMin.ToStr(MinS.as_mut_ptr());
            self.mMax.ToStr(MaxS.as_mut_ptr());
            sprintf(s, "(%s|%s)\0".as_ptr() as *const c_char, MinS.as_ptr(), MaxS.as_ptr());
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn Validate(&mut self) {
        // assert(mMax>=mMin);
    }
}

impl CVec3 {
    pub fn MaxElement(&self) -> f32 {
        self.v[self.MaxElementIndex()]
    }

    pub fn MaxElementIndex(&self) -> usize {
        if self.v[0] > self.v[1] {
            if self.v[0] > self.v[2] { 0 } else { 2 }
        } else if self.v[1] > self.v[2] {
            1
        } else {
            2
        }
    }
}
