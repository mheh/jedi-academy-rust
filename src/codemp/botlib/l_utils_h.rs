/*****************************************************************************
 * name:		l_util.h
 *
 * desc:		utils
 *
 * $Archive: /source/code/botlib/l_util.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::c_float;

extern "C" {
    // Vector2Angles(v,a) vectoangles(v,a)
    pub fn vectoangles(v: *const c_float, a: *mut c_float);
}

#[inline]
pub fn Vector2Angles(v: *const c_float, a: *mut c_float) {
    unsafe { vectoangles(v, a) }
}

// MAX_PATH MAX_QPATH
pub const MAX_PATH: usize = MAX_QPATH;

// Maximum(x,y) (x > y ? x : y)
#[inline]
pub fn Maximum<T: PartialOrd + Copy>(x: T, y: T) -> T {
    if x > y { x } else { y }
}

// Minimum(x,y) (x < y ? x : y)
#[inline]
pub fn Minimum<T: PartialOrd + Copy>(x: T, y: T) -> T {
    if x < y { x } else { y }
}
