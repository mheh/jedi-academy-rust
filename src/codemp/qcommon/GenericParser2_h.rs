//! Mechanical port of `codemp/qcommon/GenericParser2.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

// `disablewarnings.h` only affects C/C++ compiler diagnostics.

#[repr(C)]
pub struct CTextPool {
    mPool: *mut c_char,
    mNext: *mut CTextPool,
    mSize: c_int,
    mUsed: c_int,
}

impl CTextPool {
    pub unsafe fn CTextPool(_initSize: c_int) -> Self {
        todo!("CTextPool::CTextPool is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetNext(&mut self) -> *mut CTextPool {
        self.mNext
    }

    pub unsafe fn SetNext(&mut self, which: *mut CTextPool) {
        self.mNext = which;
    }

    pub unsafe fn GetPool(&mut self) -> *mut c_char {
        self.mPool
    }

    pub unsafe fn GetUsed(&mut self) -> c_int {
        self.mUsed
    }

    pub unsafe fn AllocText(
        &mut self,
        _text: *mut c_char,
        _addNULL: bool,
        _poolPtr: *mut *mut CTextPool,
    ) -> *mut c_char {
        todo!("CTextPool::AllocText is implemented in GenericParser2.cpp")
    }
}

impl Drop for CTextPool {
    fn drop(&mut self) {
        todo!("CTextPool::~CTextPool is implemented in GenericParser2.cpp")
    }
}

pub unsafe fn CleanTextPool(_pool: *mut CTextPool) {
    todo!("CleanTextPool is implemented in GenericParser2.cpp")
}

#[repr(C)]
pub struct CGPObject {
    mName: *const c_char,
    mNext: *mut CGPObject,
    mInOrderNext: *mut CGPObject,
    mInOrderPrevious: *mut CGPObject,
}

impl CGPObject {
    pub unsafe fn CGPObject(_initName: *const c_char) -> Self {
        todo!("CGPObject::CGPObject is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetName(&mut self) -> *const c_char {
        self.mName
    }

    pub unsafe fn GetNext(&mut self) -> *mut CGPObject {
        self.mNext
    }

    pub unsafe fn SetNext(&mut self, which: *mut CGPObject) {
        self.mNext = which;
    }

    pub unsafe fn GetInOrderNext(&mut self) -> *mut CGPObject {
        self.mInOrderNext
    }

    pub unsafe fn SetInOrderNext(&mut self, which: *mut CGPObject) {
        self.mInOrderNext = which;
    }

    pub unsafe fn GetInOrderPrevious(&mut self) -> *mut CGPObject {
        self.mInOrderPrevious
    }

    pub unsafe fn SetInOrderPrevious(&mut self, which: *mut CGPObject) {
        self.mInOrderPrevious = which;
    }

    pub unsafe fn WriteText(
        &mut self,
        _textPool: *mut *mut CTextPool,
        _text: *const c_char,
    ) -> bool {
        todo!("CGPObject::WriteText is implemented in GenericParser2.cpp")
    }
}

#[repr(C)]
pub struct CGPValue {
    pub base: CGPObject,
    mList: *mut CGPObject,
}

impl CGPValue {
    pub unsafe fn CGPValue(_initName: *const c_char, _initValue: *const c_char) -> Self {
        todo!("CGPValue::CGPValue is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetName(&mut self) -> *const c_char {
        self.base.GetName()
    }

    pub unsafe fn GetNext(&mut self) -> *mut CGPValue {
        self.base.mNext as *mut CGPValue
    }

    pub unsafe fn SetNext(&mut self, which: *mut CGPObject) {
        self.base.SetNext(which);
    }

    pub unsafe fn GetInOrderNext(&mut self) -> *mut CGPObject {
        self.base.GetInOrderNext()
    }

    pub unsafe fn SetInOrderNext(&mut self, which: *mut CGPObject) {
        self.base.SetInOrderNext(which);
    }

    pub unsafe fn GetInOrderPrevious(&mut self) -> *mut CGPObject {
        self.base.GetInOrderPrevious()
    }

    pub unsafe fn SetInOrderPrevious(&mut self, which: *mut CGPObject) {
        self.base.SetInOrderPrevious(which);
    }

    pub unsafe fn Duplicate(&mut self, _textPool: *mut *mut CTextPool) -> *mut CGPValue {
        todo!("CGPValue::Duplicate is implemented in GenericParser2.cpp")
    }

    pub unsafe fn IsList(&mut self) -> bool {
        todo!("CGPValue::IsList is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetTopValue(&mut self) -> *const c_char {
        todo!("CGPValue::GetTopValue is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetList(&mut self) -> *mut CGPObject {
        self.mList
    }

    pub unsafe fn AddValue(&mut self, _newValue: *const c_char, _textPool: *mut *mut CTextPool) {
        todo!("CGPValue::AddValue is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Parse(
        &mut self,
        _dataPtr: *mut *mut c_char,
        _textPool: *mut *mut CTextPool,
    ) -> bool {
        todo!("CGPValue::Parse is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Write(&mut self, _textPool: *mut *mut CTextPool, _depth: c_int) -> bool {
        todo!("CGPValue::Write is implemented in GenericParser2.cpp")
    }
}

impl Drop for CGPValue {
    fn drop(&mut self) {
        todo!("CGPValue::~CGPValue is implemented in GenericParser2.cpp")
    }
}

#[repr(C)]
pub struct CGPGroup {
    pub base: CGPObject,
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
    unsafe fn SortObject(
        &mut self,
        _object: *mut CGPObject,
        _unsortedList: *mut *mut CGPObject,
        _sortedList: *mut *mut CGPObject,
        _lastObject: *mut *mut CGPObject,
    ) {
        todo!("CGPGroup::SortObject is implemented in GenericParser2.cpp")
    }

    pub unsafe fn CGPGroup(_initName: *const c_char, _initParent: *mut CGPGroup) -> Self {
        todo!("CGPGroup::CGPGroup is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetName(&mut self) -> *const c_char {
        self.base.GetName()
    }

    pub unsafe fn GetParent(&mut self) -> *mut CGPGroup {
        self.mParent
    }

    pub unsafe fn GetNext(&mut self) -> *mut CGPGroup {
        self.base.mNext as *mut CGPGroup
    }

    pub unsafe fn GetInOrderNext(&mut self) -> *mut CGPObject {
        self.base.GetInOrderNext()
    }

    pub unsafe fn SetInOrderNext(&mut self, which: *mut CGPObject) {
        self.base.SetInOrderNext(which);
    }

    pub unsafe fn GetInOrderPrevious(&mut self) -> *mut CGPObject {
        self.base.GetInOrderPrevious()
    }

    pub unsafe fn SetInOrderPrevious(&mut self, which: *mut CGPObject) {
        self.base.SetInOrderPrevious(which);
    }

    pub unsafe fn GetNumSubGroups(&mut self) -> c_int {
        todo!("CGPGroup::GetNumSubGroups is implemented in GenericParser2.cpp")
    }

    pub unsafe fn GetNumPairs(&mut self) -> c_int {
        todo!("CGPGroup::GetNumPairs is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Clean(&mut self) {
        todo!("CGPGroup::Clean is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Duplicate(
        &mut self,
        _textPool: *mut *mut CTextPool,
        _initParent: *mut CGPGroup,
    ) -> *mut CGPGroup {
        todo!("CGPGroup::Duplicate is implemented in GenericParser2.cpp")
    }

    pub unsafe fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    pub unsafe fn GetPairs(&mut self) -> *mut CGPValue {
        self.mPairs
    }

    pub unsafe fn GetInOrderPairs(&mut self) -> *mut CGPValue {
        self.mInOrderPairs
    }

    pub unsafe fn GetSubGroups(&mut self) -> *mut CGPGroup {
        self.mSubGroups
    }

    pub unsafe fn GetInOrderSubGroups(&mut self) -> *mut CGPGroup {
        self.mInOrderSubGroups
    }

    pub unsafe fn AddPair(
        &mut self,
        _name: *const c_char,
        _value: *const c_char,
        _textPool: *mut *mut CTextPool,
    ) -> *mut CGPValue {
        todo!("CGPGroup::AddPair is implemented in GenericParser2.cpp")
    }

    pub unsafe fn AddPair_CGPValue(&mut self, _NewPair: *mut CGPValue) {
        todo!("CGPGroup::AddPair(CGPValue *) is implemented in GenericParser2.cpp")
    }

    pub unsafe fn AddGroup(
        &mut self,
        _name: *const c_char,
        _textPool: *mut *mut CTextPool,
    ) -> *mut CGPGroup {
        todo!("CGPGroup::AddGroup is implemented in GenericParser2.cpp")
    }

    pub unsafe fn AddGroup_CGPGroup(&mut self, _NewGroup: *mut CGPGroup) {
        todo!("CGPGroup::AddGroup(CGPGroup *) is implemented in GenericParser2.cpp")
    }

    pub unsafe fn FindSubGroup(&mut self, _name: *const c_char) -> *mut CGPGroup {
        todo!("CGPGroup::FindSubGroup is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Parse(
        &mut self,
        _dataPtr: *mut *mut c_char,
        _textPool: *mut *mut CTextPool,
    ) -> bool {
        todo!("CGPGroup::Parse is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Write(&mut self, _textPool: *mut *mut CTextPool, _depth: c_int) -> bool {
        todo!("CGPGroup::Write is implemented in GenericParser2.cpp")
    }

    pub unsafe fn FindPair(&mut self, _key: *const c_char) -> *mut CGPValue {
        todo!("CGPGroup::FindPair is implemented in GenericParser2.cpp")
    }

    pub unsafe fn FindPairValue(
        &mut self,
        _key: *const c_char,
        _defaultVal: *const c_char,
    ) -> *const c_char {
        todo!("CGPGroup::FindPairValue is implemented in GenericParser2.cpp")
    }
}

impl Drop for CGPGroup {
    fn drop(&mut self) {
        todo!("CGPGroup::~CGPGroup is implemented in GenericParser2.cpp")
    }
}

#[repr(C)]
pub struct CGenericParser2 {
    mTopLevel: CGPGroup,
    mTextPool: *mut CTextPool,
    mWriteable: bool,
}

impl CGenericParser2 {
    pub unsafe fn CGenericParser2() -> Self {
        todo!("CGenericParser2::CGenericParser2 is implemented in GenericParser2.cpp")
    }

    pub unsafe fn SetWriteable(&mut self, writeable: bool) {
        self.mWriteable = writeable;
    }

    pub unsafe fn GetBaseParseGroup(&mut self) -> *mut CGPGroup {
        addr_of_mut!(self.mTopLevel)
    }

    pub unsafe fn Parse(
        &mut self,
        _dataPtr: *mut *mut c_char,
        _cleanFirst: bool,
        _writeable: bool,
    ) -> bool {
        todo!("CGenericParser2::Parse is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Parse_char(
        &mut self,
        dataPtr: *mut c_char,
        cleanFirst: bool,
        writeable: bool,
    ) -> bool {
        let mut dataPtr = dataPtr;
        self.Parse(addr_of_mut!(dataPtr), cleanFirst, writeable)
    }

    pub unsafe fn Clean(&mut self) {
        todo!("CGenericParser2::Clean is implemented in GenericParser2.cpp")
    }

    pub unsafe fn Write(&mut self, _textPool: *mut CTextPool) -> bool {
        todo!("CGenericParser2::Write is implemented in GenericParser2.cpp")
    }
}

impl Drop for CGenericParser2 {
    fn drop(&mut self) {
        todo!("CGenericParser2::~CGenericParser2 is implemented in GenericParser2.cpp")
    }
}

// The following groups of routines are used for a C interface into GP2.
// C++ users should just use the objects as normally and not call these routines below

pub type TGenericParser2 = *mut c_void;
pub type TGPGroup = *mut c_void;
pub type TGPValue = *mut c_void;

unsafe extern "C" {
    // CGenericParser2 (void *) routines
    pub fn GP_Parse(
        dataPtr: *mut *mut c_char,
        cleanFirst: bool,
        writeable: bool,
    ) -> TGenericParser2;
    pub fn GP_Clean(GP2: TGenericParser2);
    pub fn GP_Delete(GP2: *mut TGenericParser2);
    pub fn GP_GetBaseParseGroup(GP2: TGenericParser2) -> TGPGroup;

    // CGPGroup (void *) routines
    pub fn GPG_GetName(GPG: TGPGroup) -> *const c_char;
    #[link_name = "GPG_GetName"]
    pub fn GPG_GetName_Value(GPG: TGPGroup, Value: *mut c_char) -> bool;
    pub fn GPG_GetNext(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetInOrderNext(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetInOrderPrevious(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetPairs(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetInOrderPairs(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetSubGroups(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_GetInOrderSubGroups(GPG: TGPGroup) -> TGPGroup;
    pub fn GPG_FindSubGroup(GPG: TGPGroup, name: *const c_char) -> TGPGroup;
    pub fn GPG_FindPair(GPG: TGPGroup, key: *const c_char) -> TGPValue;
    pub fn GPG_FindPairValue(
        GPG: TGPGroup,
        key: *const c_char,
        defaultVal: *const c_char,
    ) -> *const c_char;
    #[link_name = "GPG_FindPairValue"]
    pub fn GPG_FindPairValue_Value(
        GPG: TGPGroup,
        key: *const c_char,
        defaultVal: *const c_char,
        Value: *mut c_char,
    ) -> bool;

    // CGPValue (void *) routines
    pub fn GPV_GetName(GPV: TGPValue) -> *const c_char;
    #[link_name = "GPV_GetName"]
    pub fn GPV_GetName_Value(GPV: TGPValue, Value: *mut c_char) -> bool;
    pub fn GPV_GetNext(GPV: TGPValue) -> TGPValue;
    pub fn GPV_GetInOrderNext(GPV: TGPValue) -> TGPValue;
    pub fn GPV_GetInOrderPrevious(GPV: TGPValue) -> TGPValue;
    pub fn GPV_IsList(GPV: TGPValue) -> bool;
    pub fn GPV_GetTopValue(GPV: TGPValue) -> *const c_char;
    #[link_name = "GPV_GetTopValue"]
    pub fn GPV_GetTopValue_Value(GPV: TGPValue, Value: *mut c_char) -> bool;
    pub fn GPV_GetList(GPV: TGPValue) -> TGPValue;
}

pub const CTextPool_initSize_DEFAULT: c_int = 10240;
pub const CTextPool_AllocText_addNULL_DEFAULT: bool = true;
pub const CTextPool_AllocText_poolPtr_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPValue_initValue_DEFAULT: *const c_char = core::ptr::null();
pub const CGPValue_Duplicate_textPool_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPValue_AddValue_textPool_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPGroup_initName_DEFAULT_BYTES: &[u8; 10] = b"Top Level\0";
pub const CGPGroup_initName_DEFAULT: *const c_char =
    CGPGroup_initName_DEFAULT_BYTES.as_ptr() as *const c_char;
pub const CGPGroup_initParent_DEFAULT: *mut CGPGroup = null_mut();
pub const CGPGroup_Duplicate_textPool_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPGroup_Duplicate_initParent_DEFAULT: *mut CGPGroup = null_mut();
pub const CGPGroup_AddPair_textPool_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPGroup_AddGroup_textPool_DEFAULT: *mut *mut CTextPool = null_mut();
pub const CGPGroup_FindPairValue_defaultVal_DEFAULT: *const c_char = core::ptr::null();
pub const CGenericParser2_Parse_cleanFirst_DEFAULT: bool = true;
pub const CGenericParser2_Parse_writeable_DEFAULT: bool = false;
