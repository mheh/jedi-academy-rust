#![allow(non_snake_case)]

use core::ffi::{c_char, c_void, c_int};

// //#include "ff_public.h"

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// LOCAL STUB: Com_Printf from engine
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...) -> c_int;
}

// LOCAL STUB: FF_MAX_PATH constant
pub const FF_MAX_PATH: c_int = 256;

#[inline]
pub fn Clamp<Type: PartialOrd + Copy>(arg: Type, min: Type, max: Type) -> Type {
    // {;
    if arg <= min {
        return min;
    } else if arg > max {
        return max;
    }
    arg
}

#[inline]
pub fn Max<Type: PartialOrd + Copy>(arg: Type, arg2: Type) -> Type {
    if arg < arg2 {
        return arg2;
    }
    arg
}

#[inline]
pub fn Min<Type: PartialOrd + Copy>(arg: Type, arg2: Type) -> Type {
    if arg > arg2 {
        return arg2;
    }
    arg
}

#[inline]
pub fn InRange<Type: PartialOrd + Copy>(arg: Type, min: Type, max: Type, invalid: Type) -> Type {
    if arg < min || arg > max {
        return invalid;
    }
    arg
}

pub type TNameTable = Vec<String>;

extern "C" {
    pub fn _rcpos(string: *const c_char, c: c_char, pos: c_int) -> c_int;
    pub fn LoadFile(filename: *const c_char) -> *mut c_void;
    pub fn UncommonDirectory(target: *const c_char, comp: *const c_char) -> *const c_char;
    pub fn RightOf(str: *const c_char, str2: *const c_char) -> *const c_char;
}

#[inline]
pub fn DeletePointer<Type>(Pointer: &mut *mut Type, String: Option<*const c_char>) {
    if !Pointer.is_null() {
        #[cfg(feature = "FF_PRINT")]
        if let Some(s) = String {
            unsafe {
                Com_Printf(b"%s\n\0".as_ptr() as *const c_char, s);
            }
        }
        // NOTE (porting): C++ delete semantics. Original: delete Pointer; Pointer = NULL;
        unsafe {
            let _ = Box::from_raw(*Pointer);
        }
        *Pointer = core::ptr::null_mut();
    }
}

#[cfg(feature = "FF_PRINT")]
extern "C" {
    pub fn ConsoleParseError(message: *const c_char, line: *const c_char, pos: c_int);
}

extern "C" {
    pub fn FS_VerifyName(
        src: *const c_char,
        name: *const c_char,
        out: *mut c_char,
        maxlen: c_int,
    ) -> qboolean;
}

//===[multimapIterator]================================================/////////////
//
//	Convenience class for iterating through a multimap. It's not actually
//	all that convenient :(  It's slightly more intuitive than the
//	actual multimap iteration logic.
//
//====================================================================/////////////

#[allow(non_snake_case)]
pub struct multimapIterator<T> {
    mIt: *mut c_void,
    mMap: *mut T,
    mKey: *mut c_void,
}

impl<T> multimapIterator<T> {
    // NOTE (porting): C++ template operator overloads and specializations cannot be directly translated.
    // Original C++ methods:
    //   multimapIterator& operator ++ ()
    //   qboolean operator != ( T::iterator it )
    //   qboolean operator == ( T::iterator it )
    //   T::iterator operator * ()
    //   T::iterator operator = ( T::iterator it )
    //   operator qboolean ()
}

/*
template< class T >
class multimapIteratorIterator
{
protected:
    multimapIterator mIt;

public:
    multimapIteratorIterator( T &map )
    :	mIt( map, map.begin() )
    {
    }
    multimapIteratorIterator& operator ++ ()
    {
        for
        (	T::iterator last = *mIt
        ;	mIt
        ;	last = mIt, ++mIt
        );

        mIt = ++last;

        return *this;
    }
    multimapIterator& operator * ()
    {
        return mIt;
    }
};
*/
