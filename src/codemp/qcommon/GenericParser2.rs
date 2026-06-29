// this include must remain at the top of every CPP file

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #if !defined(GENERICPARSER2_H_INC)
// 	#include "GenericParser2.h"
// #endif

use core::ffi::c_char;
use std::ffi::CStr;
use std::os::raw::{c_int, c_void};

// Stub declarations for external functions - these would be declared in GenericParser2.h
pub type TGenericParser2 = *mut c_void;
pub type TGPGroup = *mut c_void;
pub type TGPValue = *mut c_void;

// Forward declarations
pub struct CGPObject;
pub struct CGPValue;
pub struct CGPGroup;
pub struct CTextPool;
pub struct CGenericParser2;

extern "C" {
    // Assumed external functions based on usage in the C++ code
    fn Z_Malloc(size: usize, tag: i32) -> *mut c_void;
    fn Z_Malloc_tagged(size: usize, tag: i32, b: i32) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn trap_Z_Malloc(size: usize, tag: i32) -> *mut c_void;
    fn trap_Z_Free(ptr: *mut c_void);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

const MAX_TOKEN_SIZE: usize = 1024;
static mut token: [c_char; MAX_TOKEN_SIZE] = [0; MAX_TOKEN_SIZE];

// TAG constants - stubbed out values
const TAG_GP2: i32 = 1;
const TAG_TEXTPOOL: i32 = 2;

#[allow(non_snake_case)]
fn GetToken(text: *mut *mut c_char, allowLineBreaks: bool, readUntilEOL: bool) -> *mut c_char {
    unsafe {
        let mut pointer = *text;
        let mut length: usize = 0;
        let mut c: i32 = 0;
        let mut foundLineBreak: bool;

        token[0] = 0;
        if pointer.is_null() {
            return &mut token[0];
        }

        loop {
            foundLineBreak = false;
            loop {
                c = *pointer as i32;
                if c > 32 {
                    break;
                }
                if c == 0 {
                    *text = std::ptr::null_mut();
                    return &mut token[0];
                }
                if c == '\n' as i32 {
                    foundLineBreak = true;
                }
                pointer = pointer.add(1);
            }
            if foundLineBreak && !allowLineBreaks {
                *text = pointer;
                return &mut token[0];
            }

            c = *pointer as i32;

            // skip single line comment
            if c == '/' as i32 && *pointer.add(1) as i32 == '/' as i32 {
                pointer = pointer.add(2);
                while !(*pointer).is_null() && *pointer as i32 != '\n' as i32 {
                    pointer = pointer.add(1);
                }
            }
            // skip multi line comments
            else if c == '/' as i32 && *pointer.add(1) as i32 == '*' as i32 {
                pointer = pointer.add(2);
                while !(*pointer).is_null()
                    && (*pointer as i32 != '*' as i32 || *pointer.add(1) as i32 != '/' as i32)
                {
                    pointer = pointer.add(1);
                }
                if !(*pointer).is_null() {
                    pointer = pointer.add(2);
                }
            } else {
                // found the start of a token
                break;
            }
        }

        if c == '"' as i32 {
            // handle a string
            pointer = pointer.add(1);
            loop {
                c = *pointer as i32;
                pointer = pointer.add(1);
                if c == '"' as i32 {
                    // token[length++] = c;
                    break;
                } else if c == 0 {
                    break;
                } else if length < MAX_TOKEN_SIZE {
                    token[length] = c as c_char;
                    length += 1;
                }
            }
        } else if readUntilEOL {
            // absorb all characters until EOL
            while c != '\n' as i32 && c != '\r' as i32 {
                if c == '/' as i32
                    && ((*pointer.add(1) as i32) == '/' as i32
                        || (*pointer.add(1) as i32) == '*' as i32)
                {
                    break;
                }

                if length < MAX_TOKEN_SIZE {
                    token[length] = c as c_char;
                    length += 1;
                }
                pointer = pointer.add(1);
                c = *pointer as i32;
            }
            // remove trailing white space
            while length > 0 && (token[length - 1] as i32) < 32 {
                length -= 1;
            }
        } else {
            while c > 32 {
                if length < MAX_TOKEN_SIZE {
                    token[length] = c as c_char;
                    length += 1;
                }
                pointer = pointer.add(1);
                c = *pointer as i32;
            }
        }

        if token[0] as i32 == '"' as i32 {
            // remove start quote
            length -= 1;
            // memmove(token, token+1, length);
            for i in 0..length {
                token[i] = token[i + 1];
            }

            if length > 0 && token[length - 1] as i32 == '"' as i32 {
                // remove end quote
                length -= 1;
            }
        }

        if length >= MAX_TOKEN_SIZE {
            length = 0;
        }
        token[length] = 0;
        *text = pointer as *mut c_char;

        return &mut token[0];
    }
}

#[repr(C)]
pub struct CTextPool {
    pub mNext: *mut CTextPool,
    pub mSize: usize,
    pub mUsed: usize,
    pub mPool: *mut c_char,
}

impl CTextPool {
    #[allow(non_snake_case)]
    pub fn new(initSize: usize) -> *mut CTextPool {
        unsafe {
            let pool = Box::new(CTextPool {
                mNext: std::ptr::null_mut(),
                mSize: initSize,
                mUsed: 0,
                mPool: Z_Malloc(initSize, TAG_TEXTPOOL),
            });
            Box::into_raw(pool)
        }
    }

    #[allow(non_snake_case)]
    pub fn AllocText(
        self: *mut CTextPool,
        text: *mut c_char,
        addNULL: bool,
        poolPtr: *mut *mut CTextPool,
    ) -> *mut c_char {
        unsafe {
            let this = &mut *self;
            let length = {
                let s = CStr::from_ptr(text);
                s.len() + if addNULL { 1 } else { 0 }
            };

            if this.mUsed + length + 1 > this.mSize {
                // extra 1 to put a null on the end
                if !poolPtr.is_null() {
                    (*poolPtr)
                        .write(Box::into_raw(Box::new(CTextPool::new(this.mSize))));
                    *poolPtr = (*poolPtr).read().as_mut().unwrap().mNext;

                    return (*poolPtr)
                        .as_mut()
                        .unwrap()
                        .AllocText(text, addNULL, poolPtr);
                }

                return std::ptr::null_mut();
            }

            // strcpy(mPool + mUsed, text);
            let src = CStr::from_ptr(text);
            let dest = this.mPool.add(this.mUsed);
            let bytes = src.to_bytes();
            std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, dest as *mut u8, bytes.len());
            if addNULL {
                *dest.add(bytes.len()) = 0;
            }

            this.mUsed += length;
            *this.mPool.add(this.mUsed) = 0;

            return this.mPool.add(this.mUsed - length);
        }
    }

    pub fn GetNext(&self) -> *mut CTextPool {
        self.mNext
    }

    pub fn SetNext(&mut self, next: *mut CTextPool) {
        self.mNext = next;
    }
}

impl Drop for CTextPool {
    fn drop(&mut self) {
        unsafe {
            Z_Free(self.mPool as *mut c_void);
        }
    }
}

#[allow(non_snake_case)]
pub fn CleanTextPool(mut pool: *mut CTextPool) {
    unsafe {
        while !pool.is_null() {
            let next = (*pool).GetNext();
            Box::from_raw(pool);
            pool = next;
        }
    }
}

#[repr(C)]
pub struct CGPObject {
    pub mName: *const c_char,
    pub mNext: *mut CGPObject,
    pub mInOrderNext: *mut CGPObject,
    pub mInOrderPrevious: *mut CGPObject,
}

impl CGPObject {
    #[allow(non_snake_case)]
    pub fn new(initName: *const c_char) -> *mut CGPObject {
        let obj = Box::new(CGPObject {
            mName: initName,
            mNext: std::ptr::null_mut(),
            mInOrderNext: std::ptr::null_mut(),
            mInOrderPrevious: std::ptr::null_mut(),
        });
        Box::into_raw(obj)
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName
    }

    pub fn GetNext(&self) -> *mut CGPObject {
        self.mNext
    }

    pub fn SetNext(&mut self, next: *mut CGPObject) {
        self.mNext = next;
    }

    pub fn GetInOrderNext(&self) -> *mut CGPObject {
        self.mInOrderNext
    }

    pub fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.mInOrderNext = next;
    }

    pub fn GetInOrderPrevious(&self) -> *mut CGPObject {
        self.mInOrderPrevious
    }

    pub fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.mInOrderPrevious = prev;
    }

    #[allow(non_snake_case)]
    pub fn WriteText(
        self: *mut CGPObject,
        textPool: *mut *mut CTextPool,
        text: *const c_char,
    ) -> bool {
        unsafe {
            if !strchr(text).is_null() || text.is_null() || *text as i32 == 0 {
                (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText("\"" as *const c_char as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(
                    text as *mut c_char,
                    false,
                    textPool,
                );
                (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText("\"" as *const c_char as *mut c_char, false, textPool);
            } else {
                (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText(text as *mut c_char, false, textPool);
            }

            true
        }
    }
}

fn strchr(s: *const c_char) -> *const c_char {
    unsafe {
        let mut p = s;
        while !p.is_null() && *p as i32 != 0 {
            if *p as i32 == ' ' as i32 {
                return p;
            }
            p = p.add(1);
        }
    }
    std::ptr::null()
}

#[repr(C)]
pub struct CGPValue {
    pub mName: *const c_char,
    pub mNext: *mut CGPObject,
    pub mInOrderNext: *mut CGPObject,
    pub mInOrderPrevious: *mut CGPObject,
    pub mList: *mut CGPObject,
}

impl CGPValue {
    #[allow(non_snake_case)]
    pub fn new(initName: *const c_char, initValue: *const c_char) -> *mut CGPValue {
        let val = Box::new(CGPValue {
            mName: initName,
            mNext: std::ptr::null_mut(),
            mInOrderNext: std::ptr::null_mut(),
            mInOrderPrevious: std::ptr::null_mut(),
            mList: std::ptr::null_mut(),
        });
        let ptr = Box::into_raw(val);
        unsafe {
            if !initValue.is_null() {
                (*ptr).AddValue(initValue, std::ptr::null_mut());
            }
        }
        ptr
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName
    }

    pub fn GetNext(&self) -> *mut CGPObject {
        self.mNext
    }

    pub fn SetNext(&mut self, next: *mut CGPObject) {
        self.mNext = next;
    }

    pub fn GetInOrderNext(&self) -> *mut CGPObject {
        self.mInOrderNext
    }

    pub fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.mInOrderNext = next;
    }

    pub fn GetInOrderPrevious(&self) -> *mut CGPObject {
        self.mInOrderPrevious
    }

    pub fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.mInOrderPrevious = prev;
    }

    pub fn GetList(&self) -> *mut CGPObject {
        self.mList
    }

    #[allow(non_snake_case)]
    pub fn Duplicate(self: *mut CGPValue, textPool: *mut *mut CTextPool) -> *mut CGPValue {
        unsafe {
            let this = &*self;
            let name = if !textPool.is_null() {
                (*textPool).as_mut().unwrap().AllocText(
                    this.mName as *mut c_char,
                    true,
                    textPool,
                )
            } else {
                this.mName as *mut c_char
            };

            let newValue = CGPValue::new(name, std::ptr::null());
            let mut iterator = this.mList;
            while !iterator.is_null() {
                let iter_name = if !textPool.is_null() {
                    (*textPool).as_mut().unwrap().AllocText(
                        (*iterator).GetName() as *mut c_char,
                        true,
                        textPool,
                    )
                } else {
                    (*iterator).GetName() as *mut c_char
                };
                (*newValue).AddValue(iter_name, textPool);
                iterator = (*iterator).GetNext();
            }

            newValue
        }
    }

    #[allow(non_snake_case)]
    pub fn IsList(&self) -> bool {
        if self.mList.is_null() || unsafe { (*self.mList).GetNext().is_null() } {
            return false;
        }

        true
    }

    pub fn GetTopValue(&self) -> *const c_char {
        unsafe {
            if !self.mList.is_null() {
                return (*self.mList).GetName();
            }
        }

        std::ptr::null()
    }

    #[allow(non_snake_case)]
    pub fn AddValue(
        &mut self,
        newValue: *const c_char,
        textPool: *mut *mut CTextPool,
    ) {
        unsafe {
            let mut new_val = newValue;
            if !textPool.is_null() {
                new_val = (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText(newValue as *mut c_char, true, textPool);
            }

            if self.mList.is_null() {
                self.mList = CGPObject::new(new_val);
                (*self.mList).SetInOrderNext(self.mList);
            } else {
                let next_obj = (*self.mList).GetInOrderNext();
                (*next_obj).SetNext(CGPObject::new(new_val));
                (*self.mList)
                    .SetInOrderNext((*next_obj).GetNext());
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn Parse(self: *mut CGPValue, dataPtr: *mut *mut c_char, textPool: *mut *mut CTextPool) -> bool {
        unsafe {
            loop {
                let token = GetToken(dataPtr, true, true);

                if token.is_null() || *token as i32 == 0 {
                    // end of data - error!
                    return false;
                } else if Q_stricmp(token, "]" as *const c_char) == 0 {
                    // ending brace for this list
                    break;
                }

                let value = (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText(token, true, textPool);
                (*self).AddValue(value, textPool);
            }

            true
        }
    }

    #[allow(non_snake_case)]
    pub fn Write(self: *mut CGPValue, textPool: *mut *mut CTextPool, depth: i32) -> bool {
        unsafe {
            if self.is_null() || (*self).mList.is_null() {
                return true;
            }

            for _i in 0..depth {
                (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText("\t" as *const c_char as *mut c_char, false, textPool);
            }

            (self as *mut CGPObject)
                .as_mut()
                .unwrap()
                .WriteText(textPool, (*self).mName);

            if (*(*self).mList).GetNext().is_null() {
                (*textPool).as_mut().unwrap().AllocText(
                    "\t\t" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );
                (*(*self).mList)
                    .WriteText(textPool, (*(*self).mList).GetName());
                (*textPool).as_mut().unwrap().AllocText(
                    "\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );
            } else {
                (*textPool).as_mut().unwrap().AllocText(
                    "\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );

                for _i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(
                        "\t" as *const c_char as *mut c_char,
                        false,
                        textPool,
                    );
                }
                (*textPool).as_mut().unwrap().AllocText(
                    "[\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );

                let mut next = (*self).mList;
                while !next.is_null() {
                    for _i in 0..depth + 1 {
                        (*textPool).as_mut().unwrap().AllocText(
                            "\t" as *const c_char as *mut c_char,
                            false,
                            textPool,
                        );
                    }
                    (*(*self).mList)
                        .WriteText(textPool, (*next).GetName());
                    (*textPool).as_mut().unwrap().AllocText(
                        "\r\n" as *const c_char as *mut c_char,
                        false,
                        textPool,
                    );

                    next = (*next).GetNext();
                }

                for _i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(
                        "\t" as *const c_char as *mut c_char,
                        false,
                        textPool,
                    );
                }
                (*textPool).as_mut().unwrap().AllocText(
                    "]\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );
            }

            true
        }
    }
}

impl Drop for CGPValue {
    fn drop(&mut self) {
        unsafe {
            let mut next = self.mList;
            while !next.is_null() {
                let temp = (*next).GetNext();
                Box::from_raw(next);
                next = temp;
            }
        }
    }
}

#[repr(C)]
pub struct CGPGroup {
    pub mName: *const c_char,
    pub mNext: *mut CGPObject,
    pub mInOrderNext: *mut CGPObject,
    pub mInOrderPrevious: *mut CGPObject,
    pub mPairs: *mut CGPValue,
    pub mInOrderPairs: *mut CGPValue,
    pub mCurrentPair: *mut CGPValue,
    pub mSubGroups: *mut CGPGroup,
    pub mInOrderSubGroups: *mut CGPGroup,
    pub mCurrentSubGroup: *mut CGPGroup,
    pub mParent: *mut CGPGroup,
    pub mWriteable: bool,
}

impl CGPGroup {
    #[allow(non_snake_case)]
    pub fn new(initName: *const c_char, initParent: *mut CGPGroup) -> *mut CGPGroup {
        let grp = Box::new(CGPGroup {
            mName: initName,
            mNext: std::ptr::null_mut(),
            mInOrderNext: std::ptr::null_mut(),
            mInOrderPrevious: std::ptr::null_mut(),
            mPairs: std::ptr::null_mut(),
            mInOrderPairs: std::ptr::null_mut(),
            mCurrentPair: std::ptr::null_mut(),
            mSubGroups: std::ptr::null_mut(),
            mInOrderSubGroups: std::ptr::null_mut(),
            mCurrentSubGroup: std::ptr::null_mut(),
            mParent: initParent,
            mWriteable: false,
        });
        Box::into_raw(grp)
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName
    }

    pub fn GetNext(&self) -> *mut CGPObject {
        self.mNext
    }

    pub fn SetNext(&mut self, next: *mut CGPObject) {
        self.mNext = next;
    }

    pub fn GetInOrderNext(&self) -> *mut CGPObject {
        self.mInOrderNext
    }

    pub fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.mInOrderNext = next;
    }

    pub fn GetInOrderPrevious(&self) -> *mut CGPObject {
        self.mInOrderPrevious
    }

    pub fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.mInOrderPrevious = prev;
    }

    pub fn GetPairs(&self) -> *mut CGPValue {
        self.mPairs
    }

    pub fn GetInOrderPairs(&self) -> *mut CGPValue {
        self.mInOrderPairs
    }

    pub fn GetSubGroups(&self) -> *mut CGPGroup {
        self.mSubGroups
    }

    pub fn GetInOrderSubGroups(&self) -> *mut CGPGroup {
        self.mInOrderSubGroups
    }

    pub fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    #[allow(non_snake_case)]
    pub fn GetNumSubGroups(&self) -> i32 {
        let mut count = 0;
        let mut group = self.mSubGroups;
        loop {
            if group.is_null() {
                break;
            }
            count += 1;
            unsafe {
                group = (*group).GetNext() as *mut CGPGroup;
            }
        }

        count
    }

    #[allow(non_snake_case)]
    pub fn GetNumPairs(&self) -> i32 {
        let mut count = 0;
        let mut pair = self.mPairs;
        loop {
            if pair.is_null() {
                break;
            }
            count += 1;
            unsafe {
                pair = (*pair).GetNext() as *mut CGPValue;
            }
        }

        count
    }

    #[allow(non_snake_case)]
    pub fn Clean(&mut self) {
        while !self.mPairs.is_null() {
            unsafe {
                self.mCurrentPair = (*self.mPairs).GetNext() as *mut CGPValue;
                Box::from_raw(self.mPairs);
                self.mPairs = self.mCurrentPair;
            }
        }

        while !self.mSubGroups.is_null() {
            unsafe {
                self.mCurrentSubGroup = (*self.mSubGroups).GetNext() as *mut CGPGroup;
                Box::from_raw(self.mSubGroups);
                self.mSubGroups = self.mCurrentSubGroup;
            }
        }

        self.mPairs = std::ptr::null_mut();
        self.mInOrderPairs = std::ptr::null_mut();
        self.mCurrentPair = std::ptr::null_mut();
        self.mSubGroups = std::ptr::null_mut();
        self.mInOrderSubGroups = std::ptr::null_mut();
        self.mCurrentSubGroup = std::ptr::null_mut();
        self.mParent = std::ptr::null_mut();
        self.mWriteable = false;
    }

    #[allow(non_snake_case)]
    pub fn Duplicate(
        self: *mut CGPGroup,
        textPool: *mut *mut CTextPool,
        initParent: *mut CGPGroup,
    ) -> *mut CGPGroup {
        unsafe {
            let this = &*self;
            let name = if !textPool.is_null() {
                (*textPool).as_mut().unwrap().AllocText(
                    this.mName as *mut c_char,
                    true,
                    textPool,
                )
            } else {
                this.mName as *mut c_char
            };

            let newGroup = CGPGroup::new(name, initParent);

            let mut subSub = this.mSubGroups;
            while !subSub.is_null() {
                let newSub = subSub.Duplicate(textPool, newGroup);
                (*newGroup).AddGroup(newSub);

                subSub = (*subSub).GetNext() as *mut CGPGroup;
            }

            let mut subPair = this.mPairs;
            while !subPair.is_null() {
                let newPair = subPair.Duplicate(textPool);
                (*newGroup).AddPair(newPair);

                subPair = (*subPair).GetNext() as *mut CGPValue;
            }

            newGroup
        }
    }

    #[allow(non_snake_case)]
    pub fn SortObject(
        &mut self,
        object: *mut CGPObject,
        unsortedList: *mut *mut CGPObject,
        sortedList: *mut *mut CGPObject,
        lastObject: *mut *mut CGPObject,
    ) {
        unsafe {
            if (*unsortedList).is_null() {
                *unsortedList = *sortedList = object;
            } else {
                (**lastObject).SetNext(object);

                let mut test = *sortedList;
                let mut last: *mut CGPObject = std::ptr::null_mut();
                while !test.is_null() {
                    if Q_stricmp((*object).GetName(), (*test).GetName()) < 0 {
                        break;
                    }

                    last = test;
                    test = (*test).GetInOrderNext();
                }

                if !test.is_null() {
                    (*test).SetInOrderPrevious(object);
                    (*object).SetInOrderNext(test);
                }
                if !last.is_null() {
                    (*last).SetInOrderNext(object);
                    (*object).SetInOrderPrevious(last);
                } else {
                    *sortedList = object;
                }
            }

            *lastObject = object;
        }
    }

    #[allow(non_snake_case)]
    pub fn AddPair(
        self: *mut CGPGroup,
        name: *const c_char,
        value: *const c_char,
        textPool: *mut *mut CTextPool,
    ) -> *mut CGPValue {
        unsafe {
            let this = &mut *self;
            let mut n = name;
            let mut v = value;

            if !textPool.is_null() {
                n = (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText(name as *mut c_char, true, textPool);
                if !v.is_null() {
                    v = (*textPool)
                        .as_mut()
                        .unwrap()
                        .AllocText(value as *mut c_char, true, textPool);
                }
            }

            let newPair = CGPValue::new(n, v);

            this.AddPair_internal(newPair);

            newPair
        }
    }

    #[allow(non_snake_case)]
    pub fn AddPair_internal(self: *mut CGPGroup, newPair: *mut CGPValue) {
        unsafe {
            let this = &mut *self;
            this.SortObject(
                newPair as *mut CGPObject,
                &mut this.mPairs as *mut *mut CGPValue as *mut *mut CGPObject,
                &mut this.mInOrderPairs as *mut *mut CGPValue as *mut *mut CGPObject,
                &mut this.mCurrentPair as *mut *mut CGPValue as *mut *mut CGPObject,
            );
        }
    }

    #[allow(non_snake_case)]
    pub fn AddGroup(
        self: *mut CGPGroup,
        name: *const c_char,
        textPool: *mut *mut CTextPool,
    ) -> *mut CGPGroup {
        unsafe {
            let this = &mut *self;
            let mut n = name;

            if !textPool.is_null() {
                n = (*textPool)
                    .as_mut()
                    .unwrap()
                    .AllocText(name as *mut c_char, true, textPool);
            }

            let newGroup = CGPGroup::new(n, self);

            this.AddGroup_internal(newGroup);

            newGroup
        }
    }

    #[allow(non_snake_case)]
    pub fn AddGroup_internal(self: *mut CGPGroup, newGroup: *mut CGPGroup) {
        unsafe {
            let this = &mut *self;
            this.SortObject(
                newGroup as *mut CGPObject,
                &mut this.mSubGroups as *mut *mut CGPGroup as *mut *mut CGPObject,
                &mut this.mInOrderSubGroups as *mut *mut CGPGroup as *mut *mut CGPObject,
                &mut this.mCurrentSubGroup as *mut *mut CGPGroup as *mut *mut CGPObject,
            );
        }
    }

    #[allow(non_snake_case)]
    pub fn FindSubGroup(&self, name: *const c_char) -> *mut CGPGroup {
        unsafe {
            let mut group = self.mSubGroups;
            while !group.is_null() {
                if stricmp(name, (*group).GetName()) == 0 {
                    return group;
                }
                group = (*group).GetNext() as *mut CGPGroup;
            }
            std::ptr::null_mut()
        }
    }

    #[allow(non_snake_case)]
    pub fn Parse(
        self: *mut CGPGroup,
        dataPtr: *mut *mut c_char,
        textPool: *mut *mut CTextPool,
    ) -> bool {
        unsafe {
            let this = &mut *self;
            let mut lastToken: [c_char; MAX_TOKEN_SIZE] = [0; MAX_TOKEN_SIZE];

            loop {
                let token = GetToken(dataPtr, true, false);

                if token.is_null() || *token as i32 == 0 {
                    // end of data - error!
                    if !this.mParent.is_null() {
                        return false;
                    } else {
                        break;
                    }
                } else if Q_stricmp(token, "}" as *const c_char) == 0 {
                    // ending brace for this group
                    break;
                }

                // strcpy(lastToken, token);
                let src = CStr::from_ptr(token);
                let bytes = src.to_bytes();
                for (i, &b) in bytes.iter().enumerate() {
                    if i < MAX_TOKEN_SIZE {
                        lastToken[i] = b as c_char;
                    }
                }
                if bytes.len() < MAX_TOKEN_SIZE {
                    lastToken[bytes.len()] = 0;
                }

                // read ahead to see what we are doing
                let token = GetToken(dataPtr, true, true);
                if Q_stricmp(token, "{" as *const c_char) == 0 {
                    // new sub group
                    let newSubGroup = this.AddGroup(&lastToken[0], textPool);
                    (*newSubGroup).SetWriteable(this.mWriteable);
                    if !newSubGroup.Parse(dataPtr, textPool) {
                        return false;
                    }
                } else if Q_stricmp(token, "[" as *const c_char) == 0 {
                    // new pair list
                    let newPair = this.AddPair(&lastToken[0], std::ptr::null(), textPool);
                    if !newPair.Parse(dataPtr, textPool) {
                        return false;
                    }
                } else {
                    // new pair
                    this.AddPair(&lastToken[0], token, textPool);
                }
            }

            true
        }
    }

    #[allow(non_snake_case)]
    pub fn Write(
        self: *mut CGPGroup,
        textPool: *mut *mut CTextPool,
        depth: i32,
    ) -> bool {
        unsafe {
            let this = &*self;
            let mut mPair = this.mPairs;
            let mut mSubGroup = this.mSubGroups;

            if depth >= 0 {
                for _i in 0..depth {
                    (*textPool)
                        .as_mut()
                        .unwrap()
                        .AllocText("\t" as *const c_char as *mut c_char, false, textPool);
                }
                (self as *mut CGPObject)
                    .as_mut()
                    .unwrap()
                    .WriteText(textPool, this.mName);
                (*textPool).as_mut().unwrap().AllocText(
                    "\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );

                for _i in 0..depth {
                    (*textPool)
                        .as_mut()
                        .unwrap()
                        .AllocText("\t" as *const c_char as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(
                    "{\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );
            }

            while !mPair.is_null() {
                mPair.Write(textPool, depth + 1);
                mPair = (*mPair).GetNext() as *mut CGPValue;
            }

            while !mSubGroup.is_null() {
                mSubGroup.Write(textPool, depth + 1);
                mSubGroup = (*mSubGroup).GetNext() as *mut CGPGroup;
            }

            if depth >= 0 {
                for _i in 0..depth {
                    (*textPool)
                        .as_mut()
                        .unwrap()
                        .AllocText("\t" as *const c_char as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(
                    "}\r\n" as *const c_char as *mut c_char,
                    false,
                    textPool,
                );
            }

            true
        }
    }

    /************************************************************************************************
     * CGPGroup::FindPair
     *    This function will search for the pair with the specified key name.  Multiple keys may be
     *    searched if you specify "||" inbetween each key name in the string.  The first key to be
     *    found (from left to right) will be returned.
     *
     * Input
     *    key: the name of the key(s) to be searched for.
     *
     * Output / Return
     *    the group belonging to the first key found or 0 if no group was found.
     *
     ************************************************************************************************/
    #[allow(non_snake_case)]
    pub fn FindPair(&self, key: *const c_char) -> *mut CGPValue {
        unsafe {
            let mut pos = key;
            while *pos as i32 != 0 {
                let separator = strstr(pos, "||" as *const c_char);
                let length: usize;
                let next: *const c_char;

                if !separator.is_null() {
                    length = (separator as usize) - (pos as usize);
                    next = separator.add(2);
                } else {
                    length = {
                        let s = CStr::from_ptr(pos);
                        s.len()
                    };
                    next = pos.add(length);
                }

                let mut pair = self.mPairs;
                while !pair.is_null() {
                    let pair_name_len = {
                        let s = CStr::from_ptr((*pair).GetName());
                        s.len()
                    };
                    if pair_name_len == length
                        && Q_stricmpn((*pair).GetName(), pos, length) == 0
                    {
                        return pair;
                    }

                    pair = (*pair).GetNext() as *mut CGPValue;
                }

                pos = next;
            }

            std::ptr::null_mut()
        }
    }

    pub fn FindPairValue(&self, key: *const c_char, defaultVal: *const c_char) -> *const c_char {
        unsafe {
            let pair = self.FindPair(key);

            if !pair.is_null() {
                return (*pair).GetTopValue();
            }

            defaultVal
        }
    }
}

impl Drop for CGPGroup {
    fn drop(&mut self) {
        self.Clean();
    }
}

fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char {
    unsafe {
        let needle_len = {
            let s = CStr::from_ptr(needle);
            s.len()
        };

        if needle_len == 0 {
            return haystack;
        }

        let mut p = haystack;
        while *p as i32 != 0 {
            if Q_stricmpn(p, needle, needle_len) == 0 {
                return p;
            }
            p = p.add(1);
        }
    }
    std::ptr::null()
}

fn stricmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe {
        let str1 = CStr::from_ptr(s1);
        let str2 = CStr::from_ptr(s2);

        let bytes1 = str1.to_bytes();
        let bytes2 = str2.to_bytes();

        for (a, b) in bytes1.iter().zip(bytes2.iter()) {
            let ca = (*a as u8).to_ascii_lowercase();
            let cb = (*b as u8).to_ascii_lowercase();
            if ca != cb {
                return if ca < cb { -1 } else { 1 };
            }
        }

        if bytes1.len() != bytes2.len() {
            return if bytes1.len() < bytes2.len() { -1 } else { 1 };
        }

        0
    }
}

#[repr(C)]
pub struct CGenericParser2 {
    pub mTextPool: *mut CTextPool,
    pub mTopLevel: CGPGroup,
    pub mWriteable: bool,
}

impl CGenericParser2 {
    pub fn new() -> *mut CGenericParser2 {
        let parser = Box::new(CGenericParser2 {
            mTextPool: std::ptr::null_mut(),
            mTopLevel: CGPGroup {
                mName: "" as *const c_char,
                mNext: std::ptr::null_mut(),
                mInOrderNext: std::ptr::null_mut(),
                mInOrderPrevious: std::ptr::null_mut(),
                mPairs: std::ptr::null_mut(),
                mInOrderPairs: std::ptr::null_mut(),
                mCurrentPair: std::ptr::null_mut(),
                mSubGroups: std::ptr::null_mut(),
                mInOrderSubGroups: std::ptr::null_mut(),
                mCurrentSubGroup: std::ptr::null_mut(),
                mParent: std::ptr::null_mut(),
                mWriteable: false,
            },
            mWriteable: false,
        });
        Box::into_raw(parser)
    }

    #[allow(non_snake_case)]
    pub fn Parse(
        self: *mut CGenericParser2,
        dataPtr: *mut *mut c_char,
        cleanFirst: bool,
        writeable: bool,
    ) -> bool {
        unsafe {
            let this = &mut *self;

            // #ifdef _XBOX
            // extern void Z_SetNewDeleteTemporary(bool bTemp);
            // Z_SetNewDeleteTemporary(true);
            // #endif

            if cleanFirst {
                this.Clean();
            }

            if this.mTextPool.is_null() {
                this.mTextPool = CTextPool::new(0x10000); // Default size
            }

            this.SetWriteable(writeable);
            this.mTopLevel.SetWriteable(writeable);
            let mut topPool = this.mTextPool;
            let ret = this.mTopLevel.Parse(dataPtr, &mut topPool);

            // #ifdef _XBOX
            // Z_SetNewDeleteTemporary(false);
            // #endif

            ret
        }
    }

    #[allow(non_snake_case)]
    pub fn Clean(&mut self) {
        self.mTopLevel.Clean();

        CleanTextPool(self.mTextPool);
        self.mTextPool = std::ptr::null_mut();
    }

    pub fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    pub fn Write(self: *mut CGenericParser2, textPool: *mut CTextPool) -> bool {
        unsafe {
            let this = &*self;
            let mut tp = textPool;
            this.mTopLevel.Write(&mut tp, -1)
        }
    }

    pub fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        &self.mTopLevel as *const CGPGroup as *mut CGPGroup
    }
}

impl Drop for CGenericParser2 {
    fn drop(&mut self) {
        self.Clean();
    }
}

// The following groups of routines are used for a C interface into GP2.
// C++ users should just use the objects as normally and not call these routines below
//
// CGenericParser2 (void *) routines
#[allow(non_snake_case)]
pub fn GP_Parse(
    dataPtr: *mut *mut c_char,
    cleanFirst: bool,
    writeable: bool,
) -> TGenericParser2 {
    unsafe {
        let parse = CGenericParser2::new();
        if (*parse).Parse(dataPtr, cleanFirst, writeable) {
            return parse as TGenericParser2;
        }

        Box::from_raw(parse);
        std::ptr::null_mut()
    }
}

#[allow(non_snake_case)]
pub fn GP_Clean(GP2: TGenericParser2) {
    if GP2.is_null() {
        return;
    }

    unsafe {
        (*(GP2 as *mut CGenericParser2)).Clean();
    }
}

#[allow(non_snake_case)]
pub fn GP_Delete(GP2: *mut TGenericParser2) {
    if GP2.is_null() || (*GP2).is_null() {
        return;
    }

    unsafe {
        Box::from_raw(*GP2 as *mut CGenericParser2);
        *GP2 = std::ptr::null_mut();
    }
}

#[allow(non_snake_case)]
pub fn GP_GetBaseParseGroup(GP2: TGenericParser2) -> TGPGroup {
    if GP2.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        (*(GP2 as *mut CGenericParser2)).GetBaseParseGroup() as TGPGroup
    }
}

// CGPGroup (void *) routines
#[allow(non_snake_case)]
pub fn GPG_GetName(GPG: TGPGroup) -> *const c_char {
    if GPG.is_null() {
        return "" as *const c_char;
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetName() }
}

#[allow(non_snake_case)]
pub fn GPG_GetName_buf(GPG: TGPGroup, Value: *mut c_char) -> bool {
    if GPG.is_null() {
        unsafe {
            *Value = 0;
        }
        return false;
    }

    unsafe {
        let name = (*(GPG as *mut CGPGroup)).GetName();
        let src = CStr::from_ptr(name);
        let bytes = src.to_bytes();
        std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, Value as *mut u8, bytes.len());
        *Value.add(bytes.len()) = 0;
    }
    true
}

#[allow(non_snake_case)]
pub fn GPG_GetNext(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetNext() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetInOrderNext(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetInOrderNext() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetInOrderPrevious(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetInOrderPrevious() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetPairs(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetPairs() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetInOrderPairs(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetInOrderPairs() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetSubGroups(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetSubGroups() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_GetInOrderSubGroups(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).GetInOrderSubGroups() as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_FindSubGroup(GPG: TGPGroup, name: *const c_char) -> TGPGroup {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).FindSubGroup(name) as TGPGroup }
}

#[allow(non_snake_case)]
pub fn GPG_FindPair(GPG: TGPGroup, key: *const c_char) -> TGPValue {
    if GPG.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPG as *mut CGPGroup)).FindPair(key) as TGPValue }
}

#[allow(non_snake_case)]
pub fn GPG_FindPairValue(GPG: TGPGroup, key: *const c_char, defaultVal: *const c_char) -> *const c_char {
    if GPG.is_null() {
        return defaultVal;
    }

    unsafe { (*(GPG as *mut CGPGroup)).FindPairValue(key, defaultVal) }
}

#[allow(non_snake_case)]
pub fn GPG_FindPairValue_buf(
    GPG: TGPGroup,
    key: *const c_char,
    defaultVal: *const c_char,
    Value: *mut c_char,
) -> bool {
    unsafe {
        let result = GPG_FindPairValue(GPG, key, defaultVal);
        let src = CStr::from_ptr(result);
        let bytes = src.to_bytes();
        std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, Value as *mut u8, bytes.len());
        *Value.add(bytes.len()) = 0;
    }

    true
}

// CGPValue (void *) routines
#[allow(non_snake_case)]
pub fn GPV_GetName(GPV: TGPValue) -> *const c_char {
    if GPV.is_null() {
        return "" as *const c_char;
    }

    unsafe { (*(GPV as *mut CGPValue)).GetName() }
}

#[allow(non_snake_case)]
pub fn GPV_GetName_buf(GPV: TGPValue, Value: *mut c_char) -> bool {
    if GPV.is_null() {
        unsafe {
            *Value = 0;
        }
        return false;
    }

    unsafe {
        let name = (*(GPV as *mut CGPValue)).GetName();
        let src = CStr::from_ptr(name);
        let bytes = src.to_bytes();
        std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, Value as *mut u8, bytes.len());
        *Value.add(bytes.len()) = 0;
    }
    true
}

#[allow(non_snake_case)]
pub fn GPV_GetNext(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPV as *mut CGPValue)).GetNext() as TGPValue }
}

#[allow(non_snake_case)]
pub fn GPV_GetInOrderNext(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPV as *mut CGPValue)).GetInOrderNext() as TGPValue }
}

#[allow(non_snake_case)]
pub fn GPV_GetInOrderPrevious(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPV as *mut CGPValue)).GetInOrderPrevious() as TGPValue }
}

#[allow(non_snake_case)]
pub fn GPV_IsList(GPV: TGPValue) -> bool {
    if GPV.is_null() {
        return false;
    }

    unsafe { (*(GPV as *mut CGPValue)).IsList() }
}

#[allow(non_snake_case)]
pub fn GPV_GetTopValue(GPV: TGPValue) -> *const c_char {
    if GPV.is_null() {
        return "" as *const c_char;
    }

    unsafe { (*(GPV as *mut CGPValue)).GetTopValue() }
}

#[allow(non_snake_case)]
pub fn GPV_GetTopValue_buf(GPV: TGPValue, Value: *mut c_char) -> bool {
    if GPV.is_null() {
        unsafe {
            *Value = 0;
        }
        return false;
    }

    unsafe {
        let top = (*(GPV as *mut CGPValue)).GetTopValue();
        if top.is_null() {
            *Value = 0;
        } else {
            let src = CStr::from_ptr(top);
            let bytes = src.to_bytes();
            std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, Value as *mut u8, bytes.len());
            *Value.add(bytes.len()) = 0;
        }
    }

    true
}

#[allow(non_snake_case)]
pub fn GPV_GetList(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*(GPV as *mut CGPValue)).GetList() as TGPValue }
}
