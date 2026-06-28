// Filename:-	genericparser2.cpp

// leave this at the top for PCH reasons...

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};
use core::mem::size_of;

// Stubs for external engine functions (normally defined in qcommon or game headers)
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Z_Malloc/Z_Free and trap variants are game-specific allocators
    fn Z_Malloc(size: c_int, tag: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn trap_Z_Malloc(size: c_int, tag: c_int) -> *mut c_void;
    fn trap_Z_Free(ptr: *mut c_void);
}

// Case-insensitive string comparison
#[inline]
fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut c1 = *s1 as c_int;
        let mut c2 = *s2 as c_int;
        let mut offset = 0;
        loop {
            c1 = (c1 as u8).to_ascii_lowercase() as c_int;
            c2 = (c2 as u8).to_ascii_lowercase() as c_int;
            if c1 != c2 || c1 == 0 {
                return c1 - c2;
            }
            offset += 1;
            c1 = *s1.add(offset) as c_int;
            c2 = *s2.add(offset) as c_int;
        }
    }
}

#[inline]
fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    strcmpi(s1, s2)
}

const MAX_TOKEN_SIZE: usize = 1024;
static mut token: [c_char; MAX_TOKEN_SIZE] = [0; MAX_TOKEN_SIZE];

fn GetToken(text: *mut *mut c_char, allowLineBreaks: bool, readUntilEOL: bool) -> *mut c_char {
    let mut pointer = unsafe { *text };
    let mut length: usize = 0;
    let mut c: c_int;
    let mut foundLineBreak: bool;

    unsafe {
        token[0] = 0;
        if pointer.is_null() {
            return addr_of_mut!(token) as *mut c_char;
        }

        loop {
            foundLineBreak = false;
            loop {
                c = *pointer as c_int;
                if c > ' ' as c_int {
                    break;
                }
                if c == 0 {
                    *text = null_mut();
                    return addr_of_mut!(token) as *mut c_char;
                }
                if c == '\n' as c_int {
                    foundLineBreak = true;
                }
                pointer = pointer.add(1);
            }
            if foundLineBreak && !allowLineBreaks {
                *text = pointer;
                return addr_of_mut!(token) as *mut c_char;
            }

            c = *pointer as c_int;

            // skip single line comment
            if c == '/' as c_int && *pointer.add(1) as c_int == '/' as c_int {
                pointer = pointer.add(2);
                while *pointer as c_int != 0 && *pointer as c_int != '\n' as c_int {
                    pointer = pointer.add(1);
                }
            }
            // skip multi line comments
            else if c == '/' as c_int && *pointer.add(1) as c_int == '*' as c_int {
                pointer = pointer.add(2);
                while *pointer as c_int != 0 && (*pointer as c_int != '*' as c_int || *pointer.add(1) as c_int != '/' as c_int) {
                    pointer = pointer.add(1);
                }
                if *pointer as c_int != 0 {
                    pointer = pointer.add(2);
                }
            } else {
                // found the start of a token
                break;
            }
        }

        if c == '"' as c_int {
            // handle a string
            pointer = pointer.add(1);
            loop {
                c = *pointer as c_int;
                pointer = pointer.add(1);
                if c == '"' as c_int {
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
            while c != '\n' as c_int && c != '\r' as c_int {
                if *pointer.add(1) as c_int == '/' as c_int && (*pointer.add(2) as c_int == '/' as c_int || *pointer.add(2) as c_int == '*' as c_int) {
                    break;
                }

                if length < MAX_TOKEN_SIZE {
                    token[length] = c as c_char;
                    length += 1;
                }
                pointer = pointer.add(1);
                c = *pointer as c_int;
            }
            // remove trailing white space
            while length > 0 && token[length - 1] as c_int < ' ' as c_int {
                length -= 1;
            }
        } else {
            while c > ' ' as c_int {
                if length < MAX_TOKEN_SIZE {
                    token[length] = c as c_char;
                    length += 1;
                }
                pointer = pointer.add(1);
                c = *pointer as c_int;
            }
        }

        if token[0] as c_int == '"' as c_int {
            // remove start quote
            length -= 1;
            memmove(addr_of_mut!(token) as *mut c_void, addr_of_mut!(token[1]) as *mut c_void, length);

            if length > 0 && token[length - 1] as c_int == '"' as c_int {
                // remove end quote
                length -= 1;
            }
        }

        if length >= MAX_TOKEN_SIZE {
            length = 0;
        }
        token[length] = 0;
        *text = pointer;

        addr_of_mut!(token) as *mut c_char
    }
}

#[repr(C)]
pub struct CTextPool {
    mNext: *mut CTextPool,
    mSize: c_int,
    mUsed: c_int,
    mPool: *mut c_char,
}

impl CTextPool {
    fn new(initSize: c_int) -> *mut CTextPool {
        let pool = unsafe {
            let p = core::alloc::alloc(
                core::alloc::Layout::new::<CTextPool>()
            ) as *mut CTextPool;
            (*p).mNext = null_mut();
            (*p).mSize = initSize;
            (*p).mUsed = 0;

            #[cfg(feature = "_EXE")]
            {
                (*p).mPool = Z_Malloc(initSize, 1) as *mut c_char; // TAG_TEXTPOOL = 1 (guessed)
            }
            #[cfg(not(feature = "_EXE"))]
            {
                (*p).mPool = trap_Z_Malloc(initSize, 0) as *mut c_char; // TAG_GP2 = 0 (guessed)
            }
            p
        };
        pool
    }

    fn SetNext(&mut self, next: *mut CTextPool) {
        self.mNext = next;
    }

    fn GetNext(&self) -> *mut CTextPool {
        self.mNext
    }

    fn AllocText(&mut self, text: *mut c_char, addNULL: bool, poolPtr: *mut *mut CTextPool) -> *mut c_char {
        let length = unsafe { strlen(text) } + if addNULL { 1 } else { 0 };

        if self.mUsed + length as c_int + 1 > self.mSize {
            // extra 1 to put a null on the end
            if !poolPtr.is_null() {
                unsafe {
                    (*poolPtr).as_mut().unwrap().SetNext(CTextPool::new(self.mSize));
                    *poolPtr = (**poolPtr).GetNext();

                    return (**poolPtr).AllocText(text, addNULL, poolPtr);
                }
            }

            return null_mut();
        }

        unsafe {
            strcpy(self.mPool.add(self.mUsed as usize), text);
            self.mUsed += length as c_int;
            *self.mPool.add(self.mUsed as usize) = 0;

            self.mPool.add((self.mUsed as usize) - length)
        }
    }
}

impl Drop for CTextPool {
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "_EXE")]
            {
                Z_Free(self.mPool as *mut c_void);
            }
            #[cfg(not(feature = "_EXE"))]
            {
                trap_Z_Free(self.mPool as *mut c_void);
            }
        }
    }
}

fn CleanTextPool(mut pool: *mut CTextPool) {
    while !pool.is_null() {
        unsafe {
            let next = (*pool).GetNext();
            drop(Box::from_raw(pool));
            pool = next;
        }
    }
}

#[repr(C)]
pub struct CGPObject {
    mName: *const c_char,
    mNext: *mut CGPObject,
    mInOrderNext: *mut CGPObject,
    mInOrderPrevious: *mut CGPObject,
}

impl CGPObject {
    fn new(initName: *const c_char) -> *mut CGPObject {
        let obj = unsafe {
            let p = core::alloc::alloc(
                core::alloc::Layout::new::<CGPObject>()
            ) as *mut CGPObject;
            (*p).mName = initName;
            (*p).mNext = null_mut();
            (*p).mInOrderNext = null_mut();
            (*p).mInOrderPrevious = null_mut();
            p
        };
        obj
    }

    fn GetName(&self) -> *const c_char {
        self.mName
    }

    fn GetNext(&self) -> *mut CGPObject {
        self.mNext
    }

    fn SetNext(&mut self, next: *mut CGPObject) {
        self.mNext = next;
    }

    fn GetInOrderNext(&self) -> *mut CGPObject {
        self.mInOrderNext
    }

    fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.mInOrderNext = next;
    }

    fn GetInOrderPrevious(&self) -> *mut CGPObject {
        self.mInOrderPrevious
    }

    fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.mInOrderPrevious = prev;
    }

    fn WriteText(&self, textPool: *mut *mut CTextPool, text: *const c_char) -> bool {
        unsafe {
            if !strchr(text, ' ' as c_int).is_null() || *text as c_int == 0 {
                (*textPool).as_mut().unwrap().AllocText(b"\"" as *const u8 as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(b"\"" as *const u8 as *mut c_char, false, textPool);
            } else {
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
            }
        }

        true
    }
}

#[repr(C)]
pub struct CGPValue {
    base: CGPObject,
    mList: *mut CGPObject,
}

impl CGPValue {
    fn new(initName: *const c_char, initValue: *const c_char) -> *mut CGPValue {
        let val = unsafe {
            let p = core::alloc::alloc(
                core::alloc::Layout::new::<CGPValue>()
            ) as *mut CGPValue;
            (*p).base.mName = initName;
            (*p).base.mNext = null_mut();
            (*p).base.mInOrderNext = null_mut();
            (*p).base.mInOrderPrevious = null_mut();
            (*p).mList = null_mut();

            if !initValue.is_null() {
                (*p).AddValue(initValue, null_mut());
            }
            p
        };
        val
    }

    fn GetName(&self) -> *const c_char {
        self.base.GetName()
    }

    fn GetNext(&self) -> *mut CGPValue {
        self.base.GetNext() as *mut CGPValue
    }

    fn SetNext(&mut self, next: *mut CGPObject) {
        self.base.SetNext(next);
    }

    fn GetInOrderNext(&self) -> *mut CGPValue {
        self.base.GetInOrderNext() as *mut CGPValue
    }

    fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.base.SetInOrderNext(next);
    }

    fn GetInOrderPrevious(&self) -> *mut CGPValue {
        self.base.GetInOrderPrevious() as *mut CGPValue
    }

    fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.base.SetInOrderPrevious(prev);
    }

    fn Duplicate(&self, textPool: *mut *mut CTextPool) -> *mut CGPValue {
        let mut name: *mut c_char;
        let mut iterator: *mut CGPObject;

        if !textPool.is_null() {
            unsafe {
                name = (*textPool).as_mut().unwrap().AllocText(self.base.mName as *mut c_char, true, textPool);
            }
        } else {
            name = self.base.mName as *mut c_char;
        }

        let newValue = CGPValue::new(name, null_mut());
        unsafe {
            iterator = self.mList;
            while !iterator.is_null() {
                if !textPool.is_null() {
                    name = (*textPool).as_mut().unwrap().AllocText((*iterator).GetName() as *mut c_char, true, textPool);
                } else {
                    name = (*iterator).GetName() as *mut c_char;
                }
                (*newValue).AddValue(name, null_mut());
                iterator = (*iterator).GetNext();
            }
        }

        newValue
    }

    fn IsList(&self) -> bool {
        unsafe {
            if self.mList.is_null() || (*self.mList).GetNext().is_null() {
                return false;
            }
        }

        true
    }

    fn GetTopValue(&self) -> *const c_char {
        unsafe {
            if !self.mList.is_null() {
                return (*self.mList).GetName();
            }
        }

        null_mut() as *const c_char
    }

    fn GetList(&self) -> *mut CGPObject {
        self.mList
    }

    fn AddValue(&mut self, newValue: *const c_char, textPool: *mut *mut CTextPool) {
        let mut new_value_ptr = newValue;
        if !textPool.is_null() {
            unsafe {
                new_value_ptr = (*textPool).as_mut().unwrap().AllocText(newValue as *mut c_char, true, textPool) as *const c_char;
            }
        }

        unsafe {
            if self.mList.is_null() {
                self.mList = CGPObject::new(new_value_ptr);
                (*self.mList).SetInOrderNext(self.mList);
            } else {
                (*(*self.mList).GetInOrderNext()).SetNext(CGPObject::new(new_value_ptr));
                (*self.mList).SetInOrderNext((*(*self.mList).GetInOrderNext()).GetNext());
            }
        }
    }

    fn Parse(&mut self, dataPtr: *mut *mut c_char, textPool: *mut *mut CTextPool) -> bool {
        let mut token: *mut c_char;
        let mut value: *mut c_char;

        loop {
            token = GetToken(dataPtr, true, true);

            unsafe {
                if *token as c_int == 0 {
                    // end of data - error!
                    return false;
                } else if strcmpi(token, b"]\0" as *const u8 as *mut c_char) == 0 {
                    // ending brace for this list
                    break;
                }

                value = (*textPool).as_mut().unwrap().AllocText(token, true, textPool);
                self.AddValue(value as *const c_char, null_mut());
            }
        }

        true
    }

    fn Write(&mut self, textPool: *mut *mut CTextPool, depth: c_int) -> bool {
        let mut i: c_int;
        let mut next: *mut CGPObject;

        if self.mList.is_null() {
            return true;
        }

        unsafe {
            for i in 0..depth {
                (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
            }
        }

        self.WriteText(textPool, self.GetName());

        unsafe {
            if (*self.mList).GetNext().is_null() {
                (*textPool).as_mut().unwrap().AllocText(b"\t\t\0" as *const u8 as *mut c_char, false, textPool);
                (*self.mList).WriteText(textPool, (*self.mList).GetName());
                (*textPool).as_mut().unwrap().AllocText(b"\r\n\0" as *const u8 as *mut c_char, false, textPool);
            } else {
                (*textPool).as_mut().unwrap().AllocText(b"\r\n\0" as *const u8 as *mut c_char, false, textPool);

                for i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(b"[\r\n\0" as *const u8 as *mut c_char, false, textPool);

                next = self.mList;
                while !next.is_null() {
                    for i in 0..depth + 1 {
                        (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                    }
                    self.mList.as_mut().unwrap().WriteText(textPool, (*next).GetName());
                    (*textPool).as_mut().unwrap().AllocText(b"\r\n\0" as *const u8 as *mut c_char, false, textPool);

                    next = (*next).GetNext();
                }

                for i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(b"]\r\n\0" as *const u8 as *mut c_char, false, textPool);
            }
        }

        true
    }

    fn WriteText(&self, textPool: *mut *mut CTextPool, text: *const c_char) -> bool {
        unsafe {
            if !strchr(text, ' ' as c_int).is_null() || *text as c_int == 0 {
                (*textPool).as_mut().unwrap().AllocText(b"\"\0" as *const u8 as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(b"\"\0" as *const u8 as *mut c_char, false, textPool);
            } else {
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
            }
        }

        true
    }
}

impl Drop for CGPValue {
    fn drop(&mut self) {
        let mut next: *mut CGPObject;

        while !self.mList.is_null() {
            unsafe {
                next = (*self.mList).GetNext();
                drop(Box::from_raw(self.mList));
                self.mList = next;
            }
        }
    }
}

#[repr(C)]
pub struct CGPGroup {
    base: CGPObject,
    mPairs: *mut CGPValue,
    mInOrderPairs: *mut CGPValue,
    mCurrentPair: *mut CGPValue,
    mSubGroups: *mut CGPGroup,
    mInOrderSubGroups: *mut CGPGroup,
    mCurrentSubGroup: *mut CGPGroup,
    mParent: *mut CGPGroup,
    mWriteable: bool,
}

impl CGPGroup {
    fn new(initName: *const c_char, initParent: *mut CGPGroup) -> *mut CGPGroup {
        let grp = unsafe {
            let p = core::alloc::alloc(
                core::alloc::Layout::new::<CGPGroup>()
            ) as *mut CGPGroup;
            (*p).base.mName = initName;
            (*p).base.mNext = null_mut();
            (*p).base.mInOrderNext = null_mut();
            (*p).base.mInOrderPrevious = null_mut();
            (*p).mPairs = null_mut();
            (*p).mInOrderPairs = null_mut();
            (*p).mCurrentPair = null_mut();
            (*p).mSubGroups = null_mut();
            (*p).mInOrderSubGroups = null_mut();
            (*p).mCurrentSubGroup = null_mut();
            (*p).mParent = initParent;
            (*p).mWriteable = false;
            p
        };
        grp
    }

    fn GetName(&self) -> *const c_char {
        self.base.GetName()
    }

    fn GetNext(&self) -> *mut CGPGroup {
        self.base.GetNext() as *mut CGPGroup
    }

    fn SetNext(&mut self, next: *mut CGPObject) {
        self.base.SetNext(next);
    }

    fn GetInOrderNext(&self) -> *mut CGPGroup {
        self.base.GetInOrderNext() as *mut CGPGroup
    }

    fn SetInOrderNext(&mut self, next: *mut CGPObject) {
        self.base.SetInOrderNext(next);
    }

    fn GetInOrderPrevious(&self) -> *mut CGPGroup {
        self.base.GetInOrderPrevious() as *mut CGPGroup
    }

    fn SetInOrderPrevious(&mut self, prev: *mut CGPObject) {
        self.base.SetInOrderPrevious(prev);
    }

    fn GetPairs(&self) -> *mut CGPValue {
        self.mPairs
    }

    fn GetInOrderPairs(&self) -> *mut CGPValue {
        self.mInOrderPairs
    }

    fn GetSubGroups(&self) -> *mut CGPGroup {
        self.mSubGroups
    }

    fn GetInOrderSubGroups(&self) -> *mut CGPGroup {
        self.mInOrderSubGroups
    }

    fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    fn GetNumSubGroups(&self) -> c_int {
        let mut count: c_int = 0;
        let mut group = self.mSubGroups;
        unsafe {
            while !group.is_null() {
                count += 1;
                group = (*group).GetNext();
            }
        }

        count
    }

    fn GetNumPairs(&self) -> c_int {
        let mut count: c_int = 0;
        let mut pair = self.mPairs;
        unsafe {
            while !pair.is_null() {
                count += 1;
                pair = (*pair).GetNext();
            }
        }

        count
    }

    fn Clean(&mut self) {
        while !self.mPairs.is_null() {
            unsafe {
                self.mCurrentPair = (*self.mPairs).GetNext();
                drop(Box::from_raw(self.mPairs));
                self.mPairs = self.mCurrentPair;
            }
        }

        while !self.mSubGroups.is_null() {
            unsafe {
                self.mCurrentSubGroup = (*self.mSubGroups).GetNext();
                drop(Box::from_raw(self.mSubGroups));
                self.mSubGroups = self.mCurrentSubGroup;
            }
        }

        self.mPairs = null_mut();
        self.mInOrderPairs = null_mut();
        self.mCurrentPair = null_mut();
        self.mSubGroups = null_mut();
        self.mInOrderSubGroups = null_mut();
        self.mCurrentSubGroup = null_mut();
        self.mParent = null_mut();
        self.mWriteable = false;
    }

    fn Duplicate(&self, textPool: *mut *mut CTextPool, initParent: *mut CGPGroup) -> *mut CGPGroup {
        let mut name: *mut c_char;
        let mut newGroup: *mut CGPGroup;
        let mut subSub: *mut CGPGroup;
        let mut newSub: *mut CGPGroup;
        let mut subPair: *mut CGPValue;
        let mut newPair: *mut CGPValue;

        if !textPool.is_null() {
            unsafe {
                name = (*textPool).as_mut().unwrap().AllocText(self.base.mName as *mut c_char, true, textPool);
            }
        } else {
            name = self.base.mName as *mut c_char;
        }

        newGroup = CGPGroup::new(name, initParent);

        unsafe {
            subSub = self.mSubGroups;
            while !subSub.is_null() {
                newSub = (*subSub).Duplicate(textPool, newGroup);
                (*newGroup).AddGroup(newSub);

                subSub = (*subSub).GetNext();
            }

            subPair = self.mPairs;
            while !subPair.is_null() {
                newPair = (*subPair).Duplicate(textPool);
                (*newGroup).AddPair(newPair);

                subPair = (*subPair).GetNext();
            }
        }

        newGroup
    }

    fn SortObject(&mut self, object: *mut CGPObject, unsortedList: *mut *mut CGPObject, sortedList: *mut *mut CGPObject, lastObject: *mut *mut CGPObject) {
        let mut test: *mut CGPObject;
        let mut last: *mut CGPObject;

        unsafe {
            if (*unsortedList).is_null() {
                *unsortedList = object;
                *sortedList = object;
            } else {
                (**lastObject).SetNext(object);

                test = *sortedList;
                last = null_mut();
                while !test.is_null() {
                    if strcmpi((*object).GetName(), (*test).GetName()) < 0 {
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

    fn AddPair(&mut self, name: *const c_char, value: *const c_char, textPool: *mut *mut CTextPool) -> *mut CGPValue {
        let mut name_ptr = name;
        let mut value_ptr = value;

        if !textPool.is_null() {
            unsafe {
                name_ptr = (*textPool).as_mut().unwrap().AllocText(name as *mut c_char, true, textPool) as *const c_char;
                if !value.is_null() {
                    value_ptr = (*textPool).as_mut().unwrap().AllocText(value as *mut c_char, true, textPool) as *const c_char;
                }
            }
        }

        let newPair = CGPValue::new(name_ptr, value_ptr);

        self.AddPair_impl(newPair);

        newPair
    }

    fn AddPair_impl(&mut self, NewPair: *mut CGPValue) {
        unsafe {
            self.SortObject(
                NewPair as *mut CGPObject,
                &mut self.mPairs as *mut *mut CGPValue as *mut *mut CGPObject,
                &mut self.mInOrderPairs as *mut *mut CGPValue as *mut *mut CGPObject,
                &mut self.mCurrentPair as *mut *mut CGPValue as *mut *mut CGPObject,
            );
        }
    }

    fn AddGroup(&mut self, name: *const c_char, textPool: *mut *mut CTextPool) -> *mut CGPGroup {
        let mut name_ptr = name;

        if !textPool.is_null() {
            unsafe {
                name_ptr = (*textPool).as_mut().unwrap().AllocText(name as *mut c_char, true, textPool) as *const c_char;
            }
        }

        let newGroup = CGPGroup::new(name_ptr, self as *mut CGPGroup);

        self.AddGroup_impl(newGroup);

        newGroup
    }

    fn AddGroup_impl(&mut self, NewGroup: *mut CGPGroup) {
        unsafe {
            self.SortObject(
                NewGroup as *mut CGPObject,
                &mut self.mSubGroups as *mut *mut CGPGroup as *mut *mut CGPObject,
                &mut self.mInOrderSubGroups as *mut *mut CGPGroup as *mut *mut CGPObject,
                &mut self.mCurrentSubGroup as *mut *mut CGPGroup as *mut *mut CGPObject,
            );
        }
    }

    fn FindSubGroup(&self, name: *const c_char) -> *mut CGPGroup {
        let mut group = self.mSubGroups;
        unsafe {
            while !group.is_null() {
                if stricmp(name, (*group).GetName()) == 0 {
                    return group;
                }
                group = (*group).GetNext();
            }
        }
        null_mut()
    }

    fn Parse(&mut self, dataPtr: *mut *mut c_char, textPool: *mut *mut CTextPool) -> bool {
        let mut token: *mut c_char;
        let mut lastToken: [c_char; MAX_TOKEN_SIZE] = [0; MAX_TOKEN_SIZE];
        let mut newSubGroup: *mut CGPGroup;
        let mut newPair: *mut CGPValue;

        loop {
            token = GetToken(dataPtr, true, false);

            unsafe {
                if *token as c_int == 0 {
                    // end of data - error!
                    if !self.mParent.is_null() {
                        return false;
                    } else {
                        break;
                    }
                } else if strcmpi(token, b"}\0" as *const u8 as *mut c_char) == 0 {
                    // ending brace for this group
                    break;
                }

                let token_len = strlen(token);
                memmove(addr_of_mut!(lastToken) as *mut c_void, token as *mut c_void, if token_len < MAX_TOKEN_SIZE - 1 { token_len } else { MAX_TOKEN_SIZE - 1 });
                lastToken[token_len] = 0;

                // read ahead to see what we are doing
                token = GetToken(dataPtr, true, true);
                if strcmpi(token, b"{\0" as *const u8 as *mut c_char) == 0 {
                    // new sub group
                    newSubGroup = self.AddGroup(addr_of!(lastToken) as *const c_char, textPool);
                    (*newSubGroup).SetWriteable(self.mWriteable);
                    if !(*newSubGroup).Parse(dataPtr, textPool) {
                        return false;
                    }
                } else if strcmpi(token, b"[\0" as *const u8 as *mut c_char) == 0 {
                    // new pair list
                    newPair = self.AddPair(addr_of!(lastToken) as *const c_char, null_mut(), textPool);
                    if !(*newPair).Parse(dataPtr, textPool) {
                        return false;
                    }
                } else {
                    // new pair
                    self.AddPair(addr_of!(lastToken) as *const c_char, token as *const c_char, textPool);
                }
            }
        }

        true
    }

    fn Write(&mut self, textPool: *mut *mut CTextPool, depth: c_int) -> bool {
        let mut i: c_int;
        let mut mPair = self.mPairs;
        let mut mSubGroup = self.mSubGroups;

        unsafe {
            if depth >= 0 {
                for i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                }
                self.WriteText(textPool, self.GetName());
                (*textPool).as_mut().unwrap().AllocText(b"\r\n\0" as *const u8 as *mut c_char, false, textPool);

                for i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(b"{\r\n\0" as *const u8 as *mut c_char, false, textPool);
            }

            while !mPair.is_null() {
                (*mPair).Write(textPool, depth + 1);
                mPair = (*mPair).GetNext();
            }

            while !mSubGroup.is_null() {
                (*mSubGroup).Write(textPool, depth + 1);
                mSubGroup = (*mSubGroup).GetNext();
            }

            if depth >= 0 {
                for i in 0..depth {
                    (*textPool).as_mut().unwrap().AllocText(b"\t\0" as *const u8 as *mut c_char, false, textPool);
                }
                (*textPool).as_mut().unwrap().AllocText(b"}\r\n\0" as *const u8 as *mut c_char, false, textPool);
            }
        }

        true
    }

    fn WriteText(&self, textPool: *mut *mut CTextPool, text: *const c_char) -> bool {
        unsafe {
            if !strchr(text, ' ' as c_int).is_null() || *text as c_int == 0 {
                (*textPool).as_mut().unwrap().AllocText(b"\"\0" as *const u8 as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
                (*textPool).as_mut().unwrap().AllocText(b"\"\0" as *const u8 as *mut c_char, false, textPool);
            } else {
                (*textPool).as_mut().unwrap().AllocText(text as *mut c_char, false, textPool);
            }
        }

        true
    }

    fn FindPair(&self, key: *const c_char) -> *mut CGPValue {
        let mut pair = self.mPairs;

        unsafe {
            while !pair.is_null() {
                if strcmpi((*pair).GetName(), key) == 0 {
                    return pair;
                }

                pair = (*pair).GetNext();
            }
        }

        null_mut()
    }

    fn FindPairValue(&self, key: *const c_char, defaultVal: *const c_char) -> *const c_char {
        let pair = self.FindPair(key);

        unsafe {
            if !pair.is_null() {
                return (*pair).GetTopValue();
            }
        }

        defaultVal
    }
}

impl Drop for CGPGroup {
    fn drop(&mut self) {
        self.Clean();
    }
}

#[repr(C)]
pub struct CGenericParser2 {
    mTextPool: *mut CTextPool,
    mWriteable: bool,
    mTopLevel: CGPGroup,
}

impl CGenericParser2 {
    fn new() -> *mut CGenericParser2 {
        let parser = unsafe {
            let p = core::alloc::alloc(
                core::alloc::Layout::new::<CGenericParser2>()
            ) as *mut CGenericParser2;
            (*p).mTextPool = null_mut();
            (*p).mWriteable = false;
            (*p).mTopLevel = CGPGroup {
                base: CGPObject {
                    mName: null_mut(),
                    mNext: null_mut(),
                    mInOrderNext: null_mut(),
                    mInOrderPrevious: null_mut(),
                },
                mPairs: null_mut(),
                mInOrderPairs: null_mut(),
                mCurrentPair: null_mut(),
                mSubGroups: null_mut(),
                mInOrderSubGroups: null_mut(),
                mCurrentSubGroup: null_mut(),
                mParent: null_mut(),
                mWriteable: false,
            };
            p
        };
        parser
    }

    fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        &self.mTopLevel as *const CGPGroup as *mut CGPGroup
    }

    fn Parse(&mut self, dataPtr: *mut *mut c_char, cleanFirst: bool, writeable: bool) -> bool {
        #[cfg(feature = "_XBOX")]
        {
            extern "C" {
                fn Z_SetNewDeleteTemporary(bTemp: bool);
            }
            unsafe {
                Z_SetNewDeleteTemporary(true);
            }
        }

        if cleanFirst {
            self.Clean();
        }

        if self.mTextPool.is_null() {
            self.mTextPool = CTextPool::new(0x4000); // Some reasonable default size
        }

        self.SetWriteable(writeable);
        unsafe {
            (*self.GetBaseParseGroup()).SetWriteable(writeable);
        }
        let mut topPool = self.mTextPool;
        let ret = unsafe {
            (*self.GetBaseParseGroup()).Parse(dataPtr, &mut topPool)
        };

        #[cfg(feature = "_XBOX")]
        {
            extern "C" {
                fn Z_SetNewDeleteTemporary(bTemp: bool);
            }
            unsafe {
                Z_SetNewDeleteTemporary(false);
            }
        }

        ret
    }

    fn Clean(&mut self) {
        unsafe {
            (*self.GetBaseParseGroup()).Clean();
        }

        CleanTextPool(self.mTextPool);
        self.mTextPool = null_mut();
    }

    fn Write(&mut self, textPool: *mut CTextPool) -> bool {
        unsafe {
            (*self.GetBaseParseGroup()).Write(&mut textPool, -1)
        }
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
pub type TGenericParser2 = *mut c_void;
pub type TGPGroup = *mut c_void;
pub type TGPValue = *mut c_void;

#[no_mangle]
pub extern "C" fn GP_Parse(dataPtr: *mut *mut c_char, cleanFirst: bool, writeable: bool) -> TGenericParser2 {
    let parse = CGenericParser2::new();
    unsafe {
        if (*parse).Parse(dataPtr, cleanFirst, writeable) {
            return parse as TGenericParser2;
        }

        drop(Box::from_raw(parse));
        null_mut()
    }
}

#[no_mangle]
pub extern "C" fn GP_Clean(GP2: TGenericParser2) {
    if GP2.is_null() {
        return;
    }

    unsafe {
        (*(GP2 as *mut CGenericParser2)).Clean();
    }
}

#[no_mangle]
pub extern "C" fn GP_Delete(GP2: *mut TGenericParser2) {
    if GP2.is_null() || (*GP2).is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(*GP2 as *mut CGenericParser2));
        *GP2 = null_mut();
    }
}

#[no_mangle]
pub extern "C" fn GP_GetBaseParseGroup(GP2: TGenericParser2) -> TGPGroup {
    if GP2.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GP2 as *mut CGenericParser2)).GetBaseParseGroup() as TGPGroup
    }
}

// CGPGroup (void *) routines
#[no_mangle]
pub extern "C" fn GPG_GetName(GPG: TGPGroup) -> *const c_char {
    if GPG.is_null() {
        return b"\0" as *const u8 as *const c_char;
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetName()
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetNext(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetNext() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetInOrderNext(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetInOrderNext() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetInOrderPrevious(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetInOrderPrevious() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetPairs(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetPairs() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetInOrderPairs(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetInOrderPairs() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetSubGroups(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetSubGroups() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_GetInOrderSubGroups(GPG: TGPGroup) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).GetInOrderSubGroups() as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_FindSubGroup(GPG: TGPGroup, name: *const c_char) -> TGPGroup {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).FindSubGroup(name) as TGPGroup
    }
}

#[no_mangle]
pub extern "C" fn GPG_FindPair(GPG: TGPGroup, key: *const c_char) -> TGPValue {
    if GPG.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).FindPair(key) as TGPValue
    }
}

#[no_mangle]
pub extern "C" fn GPG_FindPairValue(GPG: TGPGroup, key: *const c_char, defaultVal: *const c_char) -> *const c_char {
    if GPG.is_null() {
        return defaultVal;
    }

    unsafe {
        (*(GPG as *mut CGPGroup)).FindPairValue(key, defaultVal)
    }
}

// CGPValue (void *) routines
#[no_mangle]
pub extern "C" fn GPV_GetName(GPV: TGPValue) -> *const c_char {
    if GPV.is_null() {
        return b"\0" as *const u8 as *const c_char;
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetName()
    }
}

#[no_mangle]
pub extern "C" fn GPV_GetNext(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetNext() as TGPValue
    }
}

#[no_mangle]
pub extern "C" fn GPV_GetInOrderNext(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetInOrderNext() as TGPValue
    }
}

#[no_mangle]
pub extern "C" fn GPV_GetInOrderPrevious(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetInOrderPrevious() as TGPValue
    }
}

#[no_mangle]
pub extern "C" fn GPV_IsList(GPV: TGPValue) -> bool {
    if GPV.is_null() {
        return false;
    }

    unsafe {
        (*(GPV as *mut CGPValue)).IsList()
    }
}

#[no_mangle]
pub extern "C" fn GPV_GetTopValue(GPV: TGPValue) -> *const c_char {
    if GPV.is_null() {
        return b"\0" as *const u8 as *const c_char;
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetTopValue()
    }
}

#[no_mangle]
pub extern "C" fn GPV_GetList(GPV: TGPValue) -> TGPValue {
    if GPV.is_null() {
        return null_mut();
    }

    unsafe {
        (*(GPV as *mut CGPValue)).GetList() as TGPValue
    }
}

//////////////////// eof /////////////////////
