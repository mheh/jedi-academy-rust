#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use core::ffi::c_char;
use core::ffi::c_void;
use std::ptr;

// #if !defined hString_H
// #define hString_H
//
// #pragma warning (push, 3)	//go back down to 3 for the stl include
// #include <string>
// #include <set>
// #include <list>
// #include <map>
// #pragma warning (pop)
//
// using namespace std;

pub struct hstring {
    mId: c_int,
}

impl hstring {
    // void Init(const char *str);

    // Original constructors translated to Rust functions
    pub fn new() -> Self {
        hstring { mId: 0 }
    }

    pub fn from_c_str(str: *const c_char) -> Self {
        let mut s = hstring { mId: 0 };
        s.Init(str);
        s
    }

    pub fn from_str_ref(str: &str) -> Self {
        let mut s = hstring { mId: 0 };
        // Convert Rust &str to c_char pointer for Init
        let c_str = std::ffi::CString::new(str).unwrap();
        s.Init(c_str.as_ptr());
        s
    }

    pub fn from_hstring(str: &hstring) -> Self {
        hstring { mId: str.mId }
    }

    fn Init(&mut self, str: *const c_char) {
        // extern implementation would be needed
        // This is a stub that marks the need for external implementation
    }

    // public:
    // operator string () const
    // {
    //     return str();
    // }

    // const char *c_str(void) const;
    pub fn c_str(&self) -> *const c_char {
        // extern implementation would be needed
        ptr::null()
    }

    // string str(void) const;
    pub fn str(&self) -> String {
        // extern implementation would be needed
        String::new()
    }

    // hstring& operator= (const char *str)
    // {
    //     Init(str);
    //     return *this;
    // }
    pub fn assign_c_str(&mut self, str: *const c_char) -> &mut Self {
        self.Init(str);
        self
    }

    // hstring& operator= (const string &str)
    // {
    //     Init(str.c_str());
    //     return *this;
    // }
    pub fn assign_str(&mut self, str: &str) -> &mut Self {
        let c_str = std::ffi::CString::new(str).unwrap();
        self.Init(c_str.as_ptr());
        self
    }

    // hstring& operator= (const hstring &str)
    // {
    //     mId=str.mId;
    //     return *this;
    // }
    pub fn assign_hstring(&mut self, str: &hstring) -> &mut Self {
        self.mId = str.mId;
        self
    }

    // bool operator== (const hstring &str) const
    // {
    //     return((mId==str.mId)?true:false);
    // }
    pub fn eq(&self, str: &hstring) -> bool {
        (self.mId == str.mId)
    }

    // int compare(const hstring &str) const
    // {
    //     return strcmp(c_str(),str.c_str());
    // }
    pub fn compare(&self, str: &hstring) -> c_int {
        // strcmp external call would be needed
        // This is a stub
        0
    }

    // bool operator< (const hstring &str) const
    // {
    //     return((mId<str.mId)?true:false);
    // }
    pub fn lt(&self, str: &hstring) -> bool {
        (self.mId < str.mId)
    }

    // int length() const
    // {
    //     return strlen(c_str());
    // }
    pub fn length(&self) -> c_int {
        // strlen external call would be needed
        // This is a stub
        0
    }
}

//
// void TouchStringPool(void);
pub extern "C" fn TouchStringPool() {
    // extern implementation
}

// ////////////
// // MapPool
// ////////////
// #define MAP_NODE_SIZE (32)
pub const MAP_NODE_SIZE: usize = 32;

// class CMapBlock;
pub struct CMapBlock;

// class CMapPoolLow
// {
//     vector <CMapBlock *>	mMapBlocks;
//     vector <void *>			mFreeList;
//     int						mLastBlockNum;
pub struct CMapPoolLow {
    mMapBlocks: Vec<*mut CMapBlock>,
    mFreeList: Vec<*mut c_void>,
    mLastBlockNum: c_int,
}

impl CMapPoolLow {
    // public:
    //     CMapPoolLow();
    pub fn new() -> Self {
        CMapPoolLow {
            mMapBlocks: Vec::new(),
            mFreeList: Vec::new(),
            mLastBlockNum: 0,
        }
    }

    //     ~CMapPoolLow();
    // Rust will drop automatically

    //     void *Alloc();
    pub fn Alloc(&mut self) -> *mut c_void {
        // extern implementation
        ptr::null_mut()
    }

    //     void Free(void *p);
    pub fn Free(&mut self, p: *mut c_void) {
        // extern implementation
    }

    //     void TouchMem();
    pub fn TouchMem(&mut self) {
        // extern implementation
    }
}

// CMapPoolLow &GetMapPool();
pub fn GetMapPool() -> &'static mut CMapPoolLow {
    // extern implementation
    // This is a stub that would need proper implementation
    unsafe { &mut *ptr::null_mut() }
}

// template<class T>
// class CMapPool
// {
//     CMapPoolLow &mPool;
// public:
//     CMapPool() : mPool(GetMapPool())
//     {
//
//     }
//     template <class U>
//     CMapPool(const U&) : mPool(GetMapPool())
//     {
//     }
//     ~CMapPool()
//     {
//     }
//
//     typedef T        value_type;
//     typedef T*       pointer;
//     typedef const T* const_pointer;
//     typedef T&       reference;
//     typedef const T& const_reference;
//     typedef size_t    size_type;
//     typedef ptrdiff_t difference_type;
//
//     template <class U>
//     struct rebind
//     {
//        typedef CMapPool<U> other;
//     };
pub struct CMapPool<T> {
    mPool: *mut CMapPoolLow,
    _phantom: std::marker::PhantomData<T>,
}

// Type aliases from C++ template specializations
pub type value_type<T> = T;
pub type pointer<T> = *mut T;
pub type const_pointer<T> = *const T;
pub type reference<T> = T;
pub type const_reference<T> = T;
pub type size_type = usize;
pub type difference_type = isize;

impl<T> CMapPool<T> {
    //     CMapPool() : mPool(GetMapPool())
    //     {
    //
    //     }
    pub fn new() -> Self {
        CMapPool {
            mPool: GetMapPool() as *mut CMapPoolLow,
            _phantom: std::marker::PhantomData,
        }
    }

    //     template <class U>
    //     CMapPool(const U&) : mPool(GetMapPool())
    //     {
    //     }
    pub fn from_other<U>(_other: &CMapPool<U>) -> Self {
        CMapPool {
            mPool: GetMapPool() as *mut CMapPoolLow,
            _phantom: std::marker::PhantomData,
        }
    }

    //
    //     // return address of values
    //     pointer address (reference value) const
    //     {
    //        return &value;
    //     }
    pub fn address(value: *mut T) -> *mut T {
        value
    }

    //     const_pointer address (const_reference value) const
    //     {
    //        return &value;
    //     }
    pub fn address_const(value: *const T) -> *const T {
        value
    }

    //
    //     // return maximum number of elements that can be allocated
    //     size_type max_size () const
    //     {
    // //	   return mMaxSize;
    //        return 0xfffffff;	//uh, take a guess
    //     }
    pub fn max_size(&self) -> size_type {
        // //	   return mMaxSize;
        0xfffffff        // uh, take a guess
    }

    //
    //     // allocate but don't initialize num elements of type T
    //     pointer allocate (size_type num, const void* = 0)
    //     {
    //         assert(sizeof(T)<=(MAP_NODE_SIZE-2)); // to big for this pool
    //         assert(num==1); //allocator not design for this
    //         return (T*)mPool.Alloc();
    //     }
    pub fn allocate(&mut self, num: size_type) -> *mut T {
        assert!(std::mem::size_of::<T>() <= (MAP_NODE_SIZE - 2));        // to big for this pool
        assert!(num == 1);        // allocator not design for this
        unsafe { (*self.mPool).Alloc() as *mut T }
    }

    //     void *_Charalloc(size_type size)
    //     {
    //         assert(size<=(MAP_NODE_SIZE-2)); // to big for this pool
    //         return mPool.Alloc();
    //     }
    pub fn _Charalloc(&mut self, size: size_type) -> *mut c_void {
        assert!(size <= (MAP_NODE_SIZE - 2));        // to big for this pool
        unsafe { (*self.mPool).Alloc() }
    }

    //
    //     // initialize elements of allocated storage p with value value
    //     void construct (pointer p, const T& value)
    //     {
    //        // initialize memory with placement new
    //        new((void*)p)T(value);
    //     }
    pub fn construct(p: *mut T, value: T) {
        // initialize memory with placement new
        unsafe { ptr::write(p, value) };
    }

    //
    //     // destroy elements of initialized storage p
    //     void destroy (pointer p)
    //     {
    //        // destroy objects by calling their destructor
    //        p->~T();
    //     }
    pub fn destroy(p: *mut T) {
        // destroy objects by calling their destructor
        unsafe { ptr::drop_in_place(p) };
    }

    //
    //     // deallocate storage p of deleted elements
    //     template<class U>
    //     void deallocate (U *p, size_type num)
    //     {
    //         assert(num==1); //allocator not design for this
    //         mPool.Free(p);
    //     }
    pub fn deallocate<U>(&mut self, p: *mut U, num: size_type) {
        assert!(num == 1);        // allocator not design for this
        unsafe { (*self.mPool).Free(p as *mut c_void) };
    }
}

// template <class T1,class T2>
// bool operator== (const CMapPool<T1>&,
//                 const CMapPool<T2>&)
// {
//    return false;
// }
pub fn cmappool_eq<T1, T2>(_a: &CMapPool<T1>, _b: &CMapPool<T2>) -> bool {
    false
}

// template <class T1,class T2>
// bool operator!= (const CMapPool<T1>&,
//                 const CMapPool<T2>&)
// {
//    return true;
// }
pub fn cmappool_ne<T1, T2>(_a: &CMapPool<T1>, _b: &CMapPool<T2>) -> bool {
    true
}

// template <class K,class V,class Compare = less<K> >
// class hmap : public map<K,V,Compare,CMapPool<V> >{};
pub type hmap<K, V> = std::collections::BTreeMap<K, V>;

// template <class K,class V,class Compare = less<K> >
// class hmultimap : public multimap<K,V,Compare,CMapPool<V> >{};
pub type hmultimap<K, V> = std::collections::BTreeMap<K, V>;

// template <class K,class Compare = less<K> >
// class hset : public set<K,Compare,CMapPool<K> >{};
pub type hset<K> = std::collections::BTreeSet<K>;

// template <class K,class Compare = less<K> >
// class hmultiset : public multiset<K,Compare,CMapPool<K> >{};
pub type hmultiset<K> = std::collections::BTreeSet<K>;

// template <class K>
// class hlist : public list<K,CMapPool<K> >{};
pub type hlist<K> = std::collections::VecDeque<K>;

// #endif // hString_H
