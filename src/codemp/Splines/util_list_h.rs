use std::alloc::{alloc, dealloc, Layout};
use std::ptr::{null_mut, copy_nonoverlapping};
use core::ffi::{c_int, c_void};

extern "C" {
    pub fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: *const c_void,
    );
}

// typedef int cmp_t(const void *, const void *);
pub type cmp_t = extern "C" fn(*const c_void, *const c_void) -> c_int;

#[allow(non_snake_case)]
pub struct idList<T> {
    m_num: c_int,
    m_size: c_int,
    m_granularity: c_int,
    m_list: *mut T,
}

impl<T> idList<T> {
    /*
    ================
    idList<type>::idList( int )
    ================
    */
    #[inline]
    pub fn new(granularity: c_int) -> Self {
        assert!(granularity > 0);

        let mut list = idList {
            m_list: null_mut(),
            m_granularity: granularity,
            m_num: 0,
            m_size: 0,
        };
        list.Clear();
        list
    }

    /*
    ================
    idList<type>::~idList<type>
    ================
    */
    #[inline]
    fn drop_inner(&mut self) {
        self.Clear();
    }

    /*
    ================
    idList<type>::Clear
    ================
    */
    #[inline]
    pub fn Clear(&mut self) {
        if !self.m_list.is_null() {
            unsafe {
                let layout = Layout::array::<T>(self.m_size as usize).unwrap();
                dealloc(self.m_list as *mut u8, layout);
            }
        }

        self.m_list = null_mut();
        self.m_num = 0;
        self.m_size = 0;
    }

    /*
    ================
    idList<type>::Num
    ================
    */
    #[inline]
    pub fn Num(&self) -> c_int {
        self.m_num
    }

    /*
    ================
    idList<type>::SetNum
    ================
    */
    #[inline]
    pub fn SetNum(&mut self, num: c_int) {
        assert!(num >= 0);
        if num > self.m_size {
            // resize it up to the closest level of granularity
            self.Resize((((num + self.m_granularity - 1) / self.m_granularity) * self.m_granularity));
        }
        self.m_num = num;
    }

    /*
    ================
    idList<type>::SetGranularity
    ================
    */
    #[inline]
    pub fn SetGranularity(&mut self, granularity: c_int) {
        assert!(granularity > 0);
        self.m_granularity = granularity;

        if !self.m_list.is_null() {
            // resize it to the closest level of granularity
            let newsize = (((self.m_num + self.m_granularity - 1) / self.m_granularity) * self.m_granularity);
            if newsize != self.m_size {
                self.Resize(newsize);
            }
        }
    }

    /*
    ================
    idList<type>::Condense

    Resizes the array to exactly the number of elements it contains
    ================
    */
    #[inline]
    pub fn Condense(&mut self) {
        if !self.m_list.is_null() {
            if self.m_num != 0 {
                self.Resize(self.m_num);
            } else {
                self.Clear();
            }
        }
    }

    /*
    ================
    idList<type>::Size
    ================
    */
    #[inline]
    pub fn Size(&self) -> c_int {
        self.m_size
    }

    /*
    ================
    idList<type>::Resize
    ================
    */
    #[inline]
    pub fn Resize(&mut self, size: c_int) {
        assert!(size > 0);

        if size <= 0 {
            self.Clear();
            return;
        }

        let temp = self.m_list;
        let old_size = self.m_size;
        self.m_size = size;
        if self.m_size < self.m_num {
            self.m_num = self.m_size;
        }

        unsafe {
            let layout = Layout::array::<T>(self.m_size as usize).unwrap();
            self.m_list = alloc(layout) as *mut T;
            for i in 0..self.m_num as usize {
                std::ptr::write(self.m_list.add(i), std::ptr::read(temp.add(i)));
            }

            if !temp.is_null() {
                let old_layout = Layout::array::<T>(old_size as usize).unwrap();
                dealloc(temp as *mut u8, old_layout);
            }
        }
    }

    /*
    ================
    idList<type>::operator[] const
    ================
    */
    #[inline]
    pub fn get(&self, index: c_int) -> *const T {
        assert!(index >= 0);
        assert!(index < self.m_num);

        unsafe { self.m_list.add(index as usize) }
    }

    /*
    ================
    idList<type>::operator[]
    ================
    */
    #[inline]
    pub fn get_mut(&mut self, index: c_int) -> *mut T {
        assert!(index >= 0);
        assert!(index < self.m_num);

        unsafe { self.m_list.add(index as usize) }
    }

    /*
    ================
    idList<type>::Append
    ================
    */
    #[inline]
    pub fn Append(&mut self, obj: &T) -> c_int
    where
        T: Copy,
    {
        if self.m_list.is_null() {
            self.Resize(self.m_granularity);
        }

        if self.m_num == self.m_size {
            self.Resize(self.m_size + self.m_granularity);
        }

        unsafe {
            *self.m_list.add(self.m_num as usize) = *obj;
        }
        self.m_num += 1;

        self.m_num - 1
    }

    /*
    ================
    idList<type>::AddUnique
    ================
    */
    #[inline]
    pub fn AddUnique(&mut self, obj: &T) -> c_int
    where
        T: Copy + PartialEq,
    {
        let mut index: c_int = 0;

        if self.Find(obj, Some(&mut index)).is_null() {
            index = self.Append(obj);
        }

        index
    }

    /*
    ================
    idList<type>::Find
    ================
    */
    #[inline]
    pub fn Find(&self, obj: &T, index: Option<&mut c_int>) -> *mut T
    where
        T: PartialEq,
    {
        for i in 0..self.m_num as usize {
            unsafe {
                if *self.m_list.add(i) == *obj {
                    if let Some(idx) = index {
                        *idx = i as c_int;
                    }
                    return self.m_list.add(i);
                }
            }
        }

        null_mut()
    }

    /*
    ================
    idList<type>::RemoveIndex
    ================
    */
    #[inline]
    pub fn RemoveIndex(&mut self, index: c_int) -> bool {
        if self.m_list.is_null() || self.m_num == 0 {
            return false;
        }

        assert!(index >= 0);
        assert!(index < self.m_num);

        if (index < 0) || (index >= self.m_num) {
            return false;
        }

        self.m_num -= 1;
        unsafe {
            for i in index as usize..self.m_num as usize {
                std::ptr::write(self.m_list.add(i), std::ptr::read(self.m_list.add(i + 1)));
            }
        }

        true
    }

    /*
    ================
    idList<type>::Remove
    ================
    */
    #[inline]
    pub fn Remove(&mut self, obj: &T) -> bool
    where
        T: PartialEq,
    {
        let mut index: c_int = 0;

        if !self.Find(obj, Some(&mut index)).is_null() {
            return self.RemoveIndex(index);
        }

        false
    }

    /*
    ================
    idList<type>::Sort
    ================
    */
    #[inline]
    pub fn Sort(&mut self, compare: Option<cmp_t>) {
        if self.m_list.is_null() {
            return;
        }

        if let Some(cmp) = compare {
            unsafe {
                qsort(
                    self.m_list as *mut c_void,
                    self.m_num as usize,
                    std::mem::size_of::<T>(),
                    cmp as *const c_void,
                );
            }
        }
    }
}

impl<T> Drop for idList<T> {
    fn drop(&mut self) {
        self.drop_inner();
    }
}
