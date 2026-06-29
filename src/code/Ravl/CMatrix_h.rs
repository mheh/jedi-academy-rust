////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Matrix Library
// --------------
//
//
//
// NOTES:
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

////////////////////////////////////////////////////////////////////////////////////////
// Includes
////////////////////////////////////////////////////////////////////////////////////////
// #if defined(RA_DEBUG_LINKING)
// 	#pragma message("...including CMatrix.h")
// #endif
// #if !defined(RAVL_VEC_INC)
// 	#include "CVec.h"
// #endif
//namespace ravl
//{

////////////////////////////////////////////////////////////////////////////////////////
// LOCAL STUB FOR CVec4
// (CVec.h ported to CVec.rs; this stub provides minimal interface for CMatrix)
////////////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CVec4 {
    pub v: [f32; 4],
}

impl CVec4 {
    #[inline]
    pub fn new() -> Self {
        CVec4 { v: [0.0, 0.0, 0.0, 0.0] }
    }

    #[inline]
    pub fn from_xyzr(x: f32, y: f32, z: f32, r: f32) -> Self {
        CVec4 { v: [x, y, z, r] }
    }

    #[inline]
    pub fn Set(&mut self, x: f32, y: f32, z: f32, r: f32) {
        self.v[0] = x;
        self.v[1] = y;
        self.v[2] = z;
        self.v[3] = r;
    }

    #[inline]
    pub fn eq(&self, t: &CVec4) -> bool {
        self.v[0] == t.v[0] && self.v[1] == t.v[1] && self.v[2] == t.v[2] && self.v[3] == t.v[3]
    }

    #[inline]
    pub fn add_assign_vec(&mut self, t: &CVec4) {
        self.v[0] += t.v[0];
        self.v[1] += t.v[1];
        self.v[2] += t.v[2];
        self.v[3] += t.v[3];
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
}




////////////////////////////////////////////////////////////////////////////////////////
// The Matrix
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CMatrix {
    pub v: [CVec4; 4],
}

impl CMatrix {
    ////////////////////////////////////////////////////////////////////////////////////
	// Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn new() -> Self {
        CMatrix {
            v: [CVec4::new(), CVec4::new(), CVec4::new(), CVec4::new()],
        }
    }

    #[inline]
    pub fn from_vecs(x: CVec4, y: CVec4, z: CVec4, w: CVec4) -> Self {
        CMatrix {
            v: [x, y, z, w],
        }
    }

    #[inline]
    pub fn from_matrix(t: &CMatrix) -> Self {
        CMatrix {
            v: [t.v[0], t.v[1], t.v[2], t.v[3]],
        }
    }

    #[inline]
    pub fn from_array(t: &[f32; 16]) -> Self {
        CMatrix {
            v: [
                CVec4::from_xyzr(t[0], t[4], t[8], t[12]),
                CVec4::from_xyzr(t[1], t[5], t[9], t[13]),
                CVec4::from_xyzr(t[2], t[6], t[10], t[14]),
                CVec4::from_xyzr(t[3], t[7], t[11], t[15]),
            ],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
	// Initializers
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn Set(&mut self, x: CVec4, y: CVec4, z: CVec4, w: CVec4) {
        self.v[0] = x;
        self.v[1] = y;
        self.v[2] = z;
        self.v[3] = w;
    }

    #[inline]
    pub fn Set_matrix(&mut self, t: &CMatrix) {
        self.v[0] = t.v[0];
        self.v[1] = t.v[1];
        self.v[2] = t.v[2];
        self.v[3] = t.v[3];
    }

    #[inline]
    pub fn Set_array(&mut self, t: &[f32; 16]) {
        self.v[0] = CVec4::from_xyzr(t[0], t[4], t[8], t[12]);
        self.v[1] = CVec4::from_xyzr(t[1], t[5], t[9], t[13]);
        self.v[2] = CVec4::from_xyzr(t[2], t[6], t[10], t[14]);
        self.v[3] = CVec4::from_xyzr(t[3], t[7], t[11], t[15]);
    }

    #[inline]
    pub fn Clear(&mut self) {
        self.v[0].Set(0.0, 0.0, 0.0, 0.0);
        self.v[1].Set(0.0, 0.0, 0.0, 0.0);
        self.v[2].Set(0.0, 0.0, 0.0, 0.0);
        self.v[3].Set(0.0, 0.0, 0.0, 0.0);
    }

    #[inline]
    pub fn Itentity(&mut self) {
        self.v[0].Set(1.0, 0.0, 0.0, 0.0);
        self.v[1].Set(0.0, 1.0, 0.0, 0.0);
        self.v[2].Set(0.0, 0.0, 1.0, 0.0);
        self.v[3].Set(0.0, 0.0, 0.0, 1.0);
    }

    #[inline]
    pub fn Translate(&mut self, x: f32, y: f32, z: f32) {
        self.v[0].Set(1.0, 0.0, 0.0, 0.0);
        self.v[1].Set(0.0, 1.0, 0.0, 0.0);
        self.v[2].Set(0.0, 0.0, 1.0, 0.0);
        self.v[3].Set(x, y, z, 1.0);
    }

    #[inline]
    pub fn Scale(&mut self, x: f32, y: f32, z: f32) {
        self.v[0].Set(x, 0.0, 0.0, 0.0);
        self.v[1].Set(0.0, y, 0.0, 0.0);
        self.v[2].Set(0.0, 0.0, z, 0.0);
        self.v[3].Set(0.0, 0.0, 0.0, 1.0);
    }

    #[inline]
    pub fn Rotate(&mut self, axis: i32, s: f32, c: f32) {
        // s/*sin(angle)*/, c/*cos(angle)*/
        match axis {
            0 => {
                self.v[0].Set(1.0, 0.0, 0.0, 0.0);
                self.v[1].Set(0.0, c, -s, 0.0);
                self.v[2].Set(0.0, s, c, 0.0);
            }
            1 => {
                self.v[0].Set(c, 0.0, s, 0.0);
                self.v[1].Set(0.0, 1.0, 0.0, 0.0);
                self.v[2].Set(-s, 0.0, c, 0.0);
            }
            2 => {
                self.v[0].Set(c, -s, 0.0, 0.0);
                self.v[1].Set(s, c, 0.0, 0.0);
                self.v[2].Set(0.0, 0.0, 1.0, 0.0);
            }
            _ => {}
        }
        self.v[3].Set(0.0, 0.0, 0.0, 1.0);
    }


    ////////////////////////////////////////////////////////////////////////////////////
	// Member Accessors
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn index(&self, i: usize) -> &CVec4 {
        &self.v[i]
    }

    #[inline]
    pub fn index_mut(&mut self, i: usize) -> &mut CVec4 {
        &mut self.v[i]
    }

    #[inline]
    pub fn up(&mut self) -> &mut CVec4 {
        &mut self.v[0]
    }

    #[inline]
    pub fn left(&mut self) -> &mut CVec4 {
        &mut self.v[1]
    }

    #[inline]
    pub fn fwd(&mut self) -> &mut CVec4 {
        &mut self.v[2]
    }

    #[inline]
    pub fn origin(&mut self) -> &mut CVec4 {
        &mut self.v[3]
    }

    ////////////////////////////////////////////////////////////////////////////////////
	// Equality / Inequality Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn operator_eq(&self, t: &CMatrix) -> bool {
        self.v[0].eq(&t.v[0]) && self.v[1].eq(&t.v[1]) && self.v[2].eq(&t.v[2]) && self.v[3].eq(&t.v[3])
    }

    #[inline]
    pub fn operator_ne(&self, t: &CMatrix) -> bool {
        !(self.v[0].eq(&t.v[0]) && self.v[1].eq(&t.v[1]) && self.v[2].eq(&t.v[2]) && self.v[3].eq(&t.v[3]))
    }

    ////////////////////////////////////////////////////////////////////////////////////
	// Basic Arithimitic Operators
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn operator_assign(&mut self, t: &CMatrix) -> &mut Self {
        self.v[0] = t.v[0];
        self.v[1] = t.v[1];
        self.v[2] = t.v[2];
        self.v[3] = t.v[3];
        self
    }

    #[inline]
    pub fn operator_add_assign(&mut self, t: &CMatrix) -> &mut Self {
        self.v[0].add_assign_vec(&t.v[0]);
        self.v[1].add_assign_vec(&t.v[1]);
        self.v[2].add_assign_vec(&t.v[2]);
        self.v[3].add_assign_vec(&t.v[3]);
        self
    }

    #[inline]
    pub fn operator_sub_assign(&mut self, t: &CMatrix) -> &mut Self {
        self.v[0].sub_assign_vec(&t.v[0]);
        self.v[1].sub_assign_vec(&t.v[1]);
        self.v[2].sub_assign_vec(&t.v[2]);
        self.v[3].sub_assign_vec(&t.v[3]);
        self
    }

    #[inline]
    pub fn operator_add(&self, t: &CMatrix) -> CMatrix {
        CMatrix {
            v: [
                self.v[0].add(&t.v[0]),
                self.v[1].add(&t.v[1]),
                self.v[2].add(&t.v[2]),
                self.v[3].add(&t.v[3]),
            ],
        }
    }

    #[inline]
    pub fn operator_sub(&self, t: &CMatrix) -> CMatrix {
        CMatrix {
            v: [
                self.v[0].sub(&t.v[0]),
                self.v[1].sub(&t.v[1]),
                self.v[2].sub(&t.v[2]),
                self.v[3].sub(&t.v[3]),
            ],
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
	// Matrix Scale
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn operator_mul_assign_scalar(&mut self, d: f32) -> &mut Self {
        self.v[0].mul_assign_scalar(d);
        self.v[1].mul_assign_scalar(d);
        self.v[2].mul_assign_scalar(d);
        self.v[3].mul_assign_scalar(d);
        self
    }


    ////////////////////////////////////////////////////////////////////////////////////
	// Matrix To Matrix Multiply
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn operator_mul_matrix(&self, t: &CMatrix) -> CMatrix {
        // assert(this!=&t);				// Don't Multiply With Self

        let mut result = CMatrix::new();				// The Resulting Matrix
        // Counters
        let mut accumulator: f32;		// Current Value Of The Dot Product
        for i in 0..4 {
            for j in 0..4 {
                accumulator = 0.0;		// Reset The Accumulator
                for k in 0..4 {
                    accumulator += self.v[i].v[k] * t.v[k].v[j];		// Calculate Dot Product Of The Two Vectors
                }
                result.v[i].v[j] = accumulator;	// Place In Result
            }
        }

        result
    }

    ////////////////////////////////////////////////////////////////////////////////////
	// Vector To Matrix Multiply
    ////////////////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn operator_mul_vec(&self, t: &CVec4) -> CVec4 {
        let mut result = CVec4::new();

        result.v[0] = self.v[0].v[0] * t.v[0] + self.v[1].v[0] * t.v[1] + self.v[2].v[0] * t.v[2] + self.v[3].v[0];
        result.v[1] = self.v[0].v[1] * t.v[0] + self.v[1].v[1] * t.v[1] + self.v[2].v[1] * t.v[2] + self.v[3].v[1];
        result.v[2] = self.v[0].v[2] * t.v[0] + self.v[1].v[2] * t.v[1] + self.v[2].v[2] * t.v[2] + self.v[3].v[2];

        result
    }
}


//}
