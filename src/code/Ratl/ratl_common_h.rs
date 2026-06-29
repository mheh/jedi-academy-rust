////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Common
// ------
// The raven libraries contain a number of common defines, enums, and typedefs which
// need to be accessed by all templates.  Each of these is included here.
//
// Also included is a safeguarded assert file for all the asserts in RTL.
//
// This file is included in EVERY TEMPLATE, so it should be very light in order to
// reduce compile times.
//
//
// Format
// ------
// In order to simplify code and provide readability, the template library has some
// standard formats.  Any new templates or functions should adhere to these formats:
//
// - All memory is statically allocated, usually by parameter SIZE
// - All classes provide an enum which defines constant variables, including CAPACITY
// - All classes which moniter the number of items allocated provide the following functions:
//     size()   - the number of objects
//     empty()  - does the container have zero objects
//     full()   - does the container have any room left for more objects
//     clear()  - remove all objects
//
//
// - Functions are defined in the following order:
//     Capacity
//     Constructors  (copy, from string, etc...)
//     Range		 (size(), empty(), full(), clear(), etc...)
//     Access        (operator[], front(), back(), etc...)
//     Modification  (add(), remove(), push(), pop(), etc...)
//     Iteration     (begin(), end(), insert(), erase(), find(), etc...)
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

// Header guard equivalent; in C++ this was: #if !defined(RATL_COMMON_INC) #define RATL_COMMON_INC
// Pragma warnings for VC++ (suppressed in Rust):
// #if defined(_MSC_VER) && !defined(__MWERKS__)
// #pragma warning ( disable : 4786 )			// Truncated to 255 characters warning
// #pragma warning ( disable : 4284 )			// nevamind what this is
// #pragma warning ( disable : 4100 )			// unreferenced formal parameter
// #pragma warning ( disable : 4512 )			// unable to generate default operator=
// #pragma warning ( disable : 4130 )			// logical operation on address of string constant
// #pragma warning ( disable : 4127 )			// conditional expression is constant
// #endif

use core::ffi::{c_int, c_char, c_void};

////////////////////////////////////////////////////////////////////////////////////////
// Includes (C headers wrapped as extern "C")
////////////////////////////////////////////////////////////////////////////////////////
// #if !defined(ASSERT_H_INC)
// #include <assert.h>
// #define ASSERT_H_INC
// #endif
//
// #if !defined(STRING_H_INC)
// #include <string.h>
// #define STRING_H_INC
// #endif

extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, count: usize) -> *mut c_void;
    pub fn memcmp(buf1: *const c_void, buf2: *const c_void, count: usize) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strncpy(dest: *mut c_char, src: *const c_char, count: usize) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
}

////////////////////////////////////////////////////////////////////////////////////////
// Forward Dec.
////////////////////////////////////////////////////////////////////////////////////////
// class hfile;
// (forward declaration for hfile; implementation elsewhere)

// I don't know why this needs to be in the global namespace, but it does
// class TRatlNew;
// inline void *operator new(size_t,TRatlNew *where)
// {
//     return where;
// }
//
// inline void operator delete(void *, TRatlNew *)
// {
//     return;
// }

// Placement new marker struct in Rust (C++ placement new semantics)
#[repr(C)]
pub struct TRatlNew {
    _marker: c_void,
}

pub mod ratl {
    use super::*;

    // #ifdef _DEBUG
    // extern int HandleSaltValue; //this is used in debug for global uniqueness of handles
    // extern int FoolTheOptimizer; //this is used to make sure certain things aren't optimized out
    // #endif
    #[cfg(debug_assertions)]
    pub mod _debug {
        use super::*;
        pub static mut HandleSaltValue: c_int = 0;
        pub static mut FoolTheOptimizer: c_int = 0;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // All Raven Template Library Internal Memory Operations
    //
    // This is mostly for future use.  For now, they only provide a simple interface with
    // a couple extra functions (eql and clr).
    ////////////////////////////////////////////////////////////////////////////////////////
    pub mod mem {
        use super::*;

        ////////////////////////////////////////////////////////////////////////////////////////
        // The Align Struct Is The Root Memory Structure for Inheritance and Object Semantics
        //
        // In most cases, we just want a simple int.  However, sometimes we need to use an
        // unsigned character array
        //
        ////////////////////////////////////////////////////////////////////////////////////////
        // #if defined(_MSC_VER) && !defined(__MWERKS__)
        // 	struct alignStruct
        // 	{
        // 		int space;
        // 	};
        // #else
        // 	struct alignStruct
        // 	{
        // 		unsigned char space[16];
        // 	} __attribute__ ((aligned(16)));
        // #endif

        #[repr(C, align(16))]
        pub struct alignStruct {
            pub space: [u8; 16],
        }

        #[inline]
        pub unsafe fn cpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void {
            memcpy(dest, src, count)
        }

        #[inline]
        pub unsafe fn set(dest: *mut c_void, c: c_int, count: usize) -> *mut c_void {
            memset(dest, c, count)
        }

        #[inline]
        pub unsafe fn cmp(buf1: *const c_void, buf2: *const c_void, count: usize) -> c_int {
            memcmp(buf1, buf2, count)
        }

        #[inline]
        pub unsafe fn eql(buf1: *const c_void, buf2: *const c_void, count: usize) -> bool {
            memcmp(buf1, buf2, count) == 0
        }

        #[inline]
        pub unsafe fn zero(dest: *mut c_void, count: usize) -> *mut c_void {
            memset(dest, 0, count)
        }

        // template<class T>
        // inline 	void	cpy( T *dest, const T *src)
        // {
        // 	cpy(dest, src, sizeof(T));
        // }
        #[inline]
        pub unsafe fn cpy_t<T>(dest: *mut T, src: *const T) {
            cpy(dest as *mut c_void, src as *const c_void, core::mem::size_of::<T>());
        }

        // template<class T>
        // inline 	void	set(T *dest, int c)
        // {
        // 	set(dest, c, sizeof(T));
        // }
        #[inline]
        pub unsafe fn set_t<T>(dest: *mut T, c: c_int) {
            set(dest as *mut c_void, c, core::mem::size_of::<T>());
        }

        // template<class T>
        // inline 	void	swap(T *s1, T *s2)
        // {
        // 	unsigned char temp[sizeof(T)];
        // 	cpy((T *)temp,s1);
        // 	cpy(s1,s2);
        // 	cpy(s2,(T *)temp);
        // }
        #[inline]
        pub unsafe fn swap<T>(s1: *mut T, s2: *mut T) {
            let size = core::mem::size_of::<T>();
            let mut temp: Vec<u8> = vec![0; size];
            cpy(temp.as_mut_ptr() as *mut c_void, s1 as *const c_void, size);
            cpy(s1 as *mut c_void, s2 as *const c_void, size);
            cpy(s2 as *mut c_void, temp.as_ptr() as *const c_void, size);
        }

        // template<class T>
        // inline 	int		cmp( const T *buf1, const T *buf2)
        // {
        // 	return cmp( buf1, buf2, sizeof(T) );
        // }
        #[inline]
        pub unsafe fn cmp_t<T>(buf1: *const T, buf2: *const T) -> c_int {
            cmp(buf1 as *const c_void, buf2 as *const c_void, core::mem::size_of::<T>())
        }

        // template<class T>
        // inline 	bool	eql( const T *buf1, const T *buf2)
        // {
        // 	return cmp( buf1, buf2,sizeof(T))==0;
        // }
        #[inline]
        pub unsafe fn eql_t<T>(buf1: *const T, buf2: *const T) -> bool {
            cmp(buf1 as *const c_void, buf2 as *const c_void, core::mem::size_of::<T>()) == 0
        }

        // template<class T>
        // inline 	void	zero( T *dest )
        // {
        // 	return set(dest, 0, sizeof(T));
        // }
        #[inline]
        pub unsafe fn zero_t<T>(dest: *mut T) {
            set(dest as *mut c_void, 0, core::mem::size_of::<T>());
        }
    }

    pub mod str {
        use super::*;

        // inline int		len(const char *src)
        // {
        // 	return strlen(src);
        // }
        #[inline]
        pub unsafe fn len(src: *const c_char) -> c_int {
            strlen(src) as c_int
        }

        // inline void	cpy(char *dest,const char *src)
        // {
        // 	strcpy(dest,src);
        // }
        #[inline]
        pub unsafe fn cpy(dest: *mut c_char, src: *const c_char) {
            strcpy(dest, src);
        }

        // inline void	ncpy(char *dest,const char *src,int destBufferLen)
        // {
        // 	strncpy(dest,src,destBufferLen);
        // }
        #[inline]
        pub unsafe fn ncpy(dest: *mut c_char, src: *const c_char, destBufferLen: c_int) {
            strncpy(dest, src, destBufferLen as usize);
        }

        // inline void	cat(char *dest,const char *src)
        // {
        // 	strcat(dest,src);
        // }
        #[inline]
        pub unsafe fn cat(dest: *mut c_char, src: *const c_char) {
            strcat(dest, src);
        }

        // inline void	ncat(char *dest,const char *src,int destBufferLen)
        // {
        // 	ncpy(dest+len(dest),src,destBufferLen-len(dest));
        // }
        #[inline]
        pub unsafe fn ncat(dest: *mut c_char, src: *const c_char, destBufferLen: c_int) {
            let len_dest = len(dest);
            ncpy(
                dest.offset(len_dest as isize),
                src,
                destBufferLen - len_dest,
            );
        }

        // inline int		cmp(const char *s1,const char *s2)
        // {
        // 	return strcmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn cmp(s1: *const c_char, s2: *const c_char) -> c_int {
            strcmp(s1, s2)
        }

        // inline bool	eql(const char *s1,const char *s2)
        // {
        // 	return !strcmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn eql(s1: *const c_char, s2: *const c_char) -> bool {
            strcmp(s1, s2) == 0
        }

        // inline int		icmp(const char *s1,const char *s2)
        // {
        // 	return stricmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn icmp(s1: *const c_char, s2: *const c_char) -> c_int {
            stricmp(s1, s2)
        }

        // inline int		cmpi(const char *s1,const char *s2)
        // {
        // 	return stricmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn cmpi(s1: *const c_char, s2: *const c_char) -> c_int {
            stricmp(s1, s2)
        }

        // inline bool	ieql(const char *s1,const char *s2)
        // {
        // 	return !stricmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn ieql(s1: *const c_char, s2: *const c_char) -> bool {
            stricmp(s1, s2) == 0
        }

        // inline bool	eqli(const char *s1,const char *s2)
        // {
        // 	return !stricmp(s1,s2);
        // }
        #[inline]
        pub unsafe fn eqli(s1: *const c_char, s2: *const c_char) -> bool {
            stricmp(s1, s2) == 0
        }

        // inline char	*tok(char *s,const char *gap)
        // {
        // 	return strtok(s,gap);
        // }
        #[inline]
        pub unsafe fn tok(s: *mut c_char, gap: *const c_char) -> *mut c_char {
            strtok(s, gap)
        }

        // void	to_upper(char *dest);
        // void	to_lower(char *dest);
        // void	printf(char *dest,const char *formatS, ...);
        // (external function stubs; implementation elsewhere)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Raven Template Library Compile Assert
    //
    // If, during compile time the stuff under (condition) is zero, this code will not
    // compile.
    ////////////////////////////////////////////////////////////////////////////////////////
    // template<int condition>
    // class	compile_assert
    // {
    // #ifdef _DEBUG
    // 	int	junk[(1 - (2 * !condition))];		// Look At Where This Was Being Compiled
    // public:
    // 	compile_assert()
    // 	{
    // 		assert(condition);
    // 		junk[0]=FoolTheOptimizer++;
    // 	}
    // 	int operator()()
    // 	{
    // 		assert(condition);
    // 		FoolTheOptimizer++;
    // 		return junk[0];
    // 	}
    // #else
    // public:
    // 	int operator()()
    // 	{
    // 		return 1;
    // 	}
    // #endif;
    // };

    pub struct compile_assert<const CONDITION: bool> {
        _marker: core::marker::PhantomData<()>,
    }

    impl<const CONDITION: bool> compile_assert<CONDITION> {
        #[cfg(debug_assertions)]
        pub fn new() -> Self {
            compile_assert {
                _marker: core::marker::PhantomData,
            }
        }

        #[cfg(not(debug_assertions))]
        pub fn new() -> Self {
            compile_assert {
                _marker: core::marker::PhantomData,
            }
        }

        #[cfg(debug_assertions)]
        pub fn call(&self) -> c_int {
            1
        }

        #[cfg(not(debug_assertions))]
        pub fn call(&self) -> c_int {
            1
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Raven Template Library Base Class
    //
    // This is the base class for all the Raven Template Library container classes like
    // vector_vs and pool_vs.
    //
    // This class might be a good place to put memory profile code in the future.
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    // class	ratl_base
    // {
    // public:
    // #ifndef _XBOX
    // 	void	save(hfile& file);
    // 	void	load(hfile& file);
    // #endif
    //
    // 	void	ProfilePrint(const char * format, ...);
    //
    // public:
    // 	static	void*	OutputPrint;
    // };

    pub struct ratl_base;

    impl ratl_base {
        // #ifndef _XBOX
        // void	save(hfile& file);
        // void	load(hfile& file);
        // #endif
        // (methods not implemented in header)

        // void	ProfilePrint(const char * format, ...);
        // (method implementation elsewhere)
    }

    // static	void*	OutputPrint;
    pub static mut OutputPrint: *mut c_void = 0 as *mut c_void;

    ////////////////////////////////////////////////////////////////////////////////////////
    // this is a simplified version of bits_vs
    ////////////////////////////////////////////////////////////////////////////////////////
    // template <int	SZ>
    // class bits_base
    // {
    // protected:
    // 	enum
    // 	{
    // 		BITS_SHIFT		= 5,									// 5.  Such A Nice Number
    // 		BITS_INT_SIZE	= 32,									// Size Of A Single Word
    // 		BITS_AND		= (BITS_INT_SIZE - 1),					// Used For And Operation
    // 		ARRAY_SIZE		= ((SZ + BITS_AND)/(BITS_INT_SIZE)),	// Num Words Used
    // 		BYTE_SIZE		= (ARRAY_SIZE*sizeof(unsigned int)),	// Num Bytes Used
    // 	};
    //
    // 	unsigned int						mV[ARRAY_SIZE];
    // public:
    // 	enum
    // 	{
    // 		SIZE			= SZ,
    // 		CAPACITY		= SZ,
    // 	};

    pub struct bits_base<const SZ: usize> {
        mV: [u32; Self::ARRAY_SIZE],
    }

    impl<const SZ: usize> bits_base<SZ> {
        const BITS_SHIFT: usize = 5;  // 5.  Such A Nice Number
        const BITS_INT_SIZE: usize = 32;  // Size Of A Single Word
        const BITS_AND: usize = (Self::BITS_INT_SIZE - 1);  // Used For And Operation
        const ARRAY_SIZE: usize = ((SZ + Self::BITS_AND) / Self::BITS_INT_SIZE);  // Num Words Used
        const BYTE_SIZE: usize = (Self::ARRAY_SIZE * core::mem::size_of::<u32>());  // Num Bytes Used

        pub const SIZE: usize = SZ;
        pub const CAPACITY: usize = SZ;

        // bits_base(bool init=true,bool initValue=false)
        // {
        // 	if (init)
        // 	{
        // 		if (initValue)
        // 		{
        // 			set();
        // 		}
        // 		else
        // 		{
        // 			clear();
        // 		}
        // 	}
        // }
        pub fn new() -> Self {
            bits_base {
                mV: [0; Self::ARRAY_SIZE],
            }
        }

        pub fn new_with_init(init: bool, initValue: bool) -> Self {
            let mut result = bits_base {
                mV: [0; Self::ARRAY_SIZE],
            };
            if init {
                if initValue {
                    result.set();
                } else {
                    result.clear();
                }
            }
            result
        }

        // void clear()
        // {
        // 	mem::zero(&mV,BYTE_SIZE);
        // }
        pub fn clear(&mut self) {
            for i in 0..Self::ARRAY_SIZE {
                self.mV[i] = 0;
            }
        }

        // void		set()
        // {
        // 	mem::set(&mV, 0xff,BYTE_SIZE);
        // }
        pub fn set(&mut self) {
            for i in 0..Self::ARRAY_SIZE {
                self.mV[i] = 0xffffffff;
            }
        }

        // void set_bit(const int i)
        // {
        // 	assert(i>=0 && i < SIZE);
        // 	mV[i>>BITS_SHIFT] |=  (1<<(i&BITS_AND));
        // }
        pub fn set_bit(&mut self, i: c_int) {
            let i = i as usize;
            assert!(i >= 0 && i < Self::SIZE);
            self.mV[i >> Self::BITS_SHIFT] |= 1 << (i & Self::BITS_AND);
        }

        // void clear_bit(const int i)
        // {
        // 	assert(i>=0 && i < SIZE);
        // 	mV[i>>BITS_SHIFT] &= ~(1<<(i&BITS_AND));
        // }
        pub fn clear_bit(&mut self, i: c_int) {
            let i = i as usize;
            assert!(i >= 0 && i < Self::SIZE);
            self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
        }

        // void mark_bit(const int i, bool set)
        // {
        // 	assert(i>=0 && i < SIZE);
        // 	if (set)
        // 	{
        // 		mV[i>>BITS_SHIFT] |=  (1<<(i&BITS_AND));
        // 	}
        // 	else
        // 	{
        // 		mV[i>>BITS_SHIFT] &= ~(1<<(i&BITS_AND));
        // 	}
        // }
        pub fn mark_bit(&mut self, i: c_int, set: bool) {
            let i = i as usize;
            assert!(i >= 0 && i < Self::SIZE);
            if set {
                self.mV[i >> Self::BITS_SHIFT] |= 1 << (i & Self::BITS_AND);
            } else {
                self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
            }
        }

        // bool operator[](const int i) const
        // {
        // 	assert(i>=0 && i < SIZE);
        // 	return (mV[i>>BITS_SHIFT] & (1<<(i&BITS_AND)))!=0;
        // }
        pub fn get(&self, i: c_int) -> bool {
            let i = i as usize;
            assert!(i >= 0 && i < Self::SIZE);
            (self.mV[i >> Self::BITS_SHIFT] & (1 << (i & Self::BITS_AND))) != 0
        }

        // int	next_bit(int start=0,bool onBit=true) const
        // {
        // 	assert(start>=0&&start<=SIZE);  //we have to accept start==size for the way the loops are done
        // 	if (start>=SIZE)
        // 	{
        // 		return SIZE;			// Did Not Find
        // 	}
        // 	// Get The Word Which Contains The Start Bit & Mask Out Everything Before The Start Bit
        // 	//--------------------------------------------------------------------------------------
        // 	unsigned int	v = mV[start>>BITS_SHIFT];
        // 	if (!onBit)
        // 	{
        // 		v= (~v);
        // 	}
        // 	v >>= (start&31);
        //
        //
        // 	// Search For The First Non Zero Word In The Array
        // 	//-------------------------------------------------
        // 	while(!v)
        // 	{
        // 		start = (start & (~(BITS_INT_SIZE-1))) + BITS_INT_SIZE;
        // 		if (start>=SIZE)
        // 		{
        // 			return SIZE;			// Did Not Find
        // 		}
        // 		v = mV[start>>BITS_SHIFT];
        // 		if (!onBit)
        // 		{
        // 			v= (~v);
        // 		}
        // 	}
        //
        //
        // 	// So, We've Found A Non Zero Word, So Start Masking Against Parts To Skip Over Bits
        // 	//-----------------------------------------------------------------------------------
        // 	if (!(v&0xffff))
        // 	{
        // 		start+=16;
        // 		v>>=16;
        // 	}
        // 	if (!(v&0xff))
        // 	{
        // 		start+=8;
        // 		v>>=8;
        // 	}
        // 	if (!(v&0xf))
        // 	{
        // 		start+=4;
        // 		v>>=4;
        // 	}
        //
        // 	// Time To Search Each Bit
        // 	//-------------------------
        // 	while(!(v&1))
        // 	{
        // 		start++;
        // 		v>>=1;
        // 	}
        // 	if (start>=SIZE)
        // 	{
        // 		return SIZE;
        // 	}
        // 	return start;
        // }
        pub fn next_bit(&self, start: c_int, onBit: bool) -> c_int {
            let mut start = start as usize;
            assert!(start >= 0 && start <= Self::SIZE);  // we have to accept start==size for the way the loops are done
            if start >= Self::SIZE {
                return Self::SIZE as c_int;  // Did Not Find
            }
            // Get The Word Which Contains The Start Bit & Mask Out Everything Before The Start Bit
            //--------------------------------------------------------------------------------------
            let mut v = self.mV[start >> Self::BITS_SHIFT];
            if !onBit {
                v = !v;
            }
            v >>= (start & 31);

            // Search For The First Non Zero Word In The Array
            //-------------------------------------------------
            while v == 0 {
                start = (start & (!(Self::BITS_INT_SIZE - 1))) + Self::BITS_INT_SIZE;
                if start >= Self::SIZE {
                    return Self::SIZE as c_int;  // Did Not Find
                }
                v = self.mV[start >> Self::BITS_SHIFT];
                if !onBit {
                    v = !v;
                }
            }

            // So, We've Found A Non Zero Word, So Start Masking Against Parts To Skip Over Bits
            //-----------------------------------------------------------------------------------
            if (v & 0xffff) == 0 {
                start += 16;
                v >>= 16;
            }
            if (v & 0xff) == 0 {
                start += 8;
                v >>= 8;
            }
            if (v & 0xf) == 0 {
                start += 4;
                v >>= 4;
            }

            // Time To Search Each Bit
            //-------------------------
            while (v & 1) == 0 {
                start += 1;
                v >>= 1;
            }
            if start >= Self::SIZE {
                return Self::SIZE as c_int;
            }
            return start as c_int;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Raven Standard Compare Class
    ////////////////////////////////////////////////////////////////////////////////////////
    // struct ratl_compare
    // {
    // 	float	mCost;
    // 	int		mHandle;
    //
    // 	bool	operator<(const ratl_compare& t) const
    // 	{
    // 		return (mCost<t.mCost);
    // 	}
    // };

    #[repr(C)]
    pub struct ratl_compare {
        pub mCost: f32,
        pub mHandle: c_int,
    }

    impl ratl_compare {
        pub fn lt(&self, t: &ratl_compare) -> bool {
            self.mCost < t.mCost
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // this is used to keep track of the constuction state for things that are always constucted
    ////////////////////////////////////////////////////////////////////////////////////////
    // class bits_true
    // {
    // public:
    //
    // 	void clear()
    // 	{
    // 	}
    // 	void set()
    // 	{
    // 	}
    // 	void set_bit(const int i)
    // 	{
    // 	}
    // 	void clear_bit(const int i)
    // 	{
    // 	}
    // 	bool operator[](const int i) const
    // 	{
    // 		return true;
    // 	}
    // 	int	next_bit(int start=0,bool onBit=true) const
    // 	{
    // 		assert(onBit); ///I didn't want to add the sz template arg, you could though
    // 		return start;
    // 	}
    // };

    pub struct bits_true;

    impl bits_true {
        pub fn clear(&self) {}
        pub fn set(&self) {}
        pub fn set_bit(&self, _i: c_int) {}
        pub fn clear_bit(&self, _i: c_int) {}
        pub fn get(&self, _i: c_int) -> bool {
            true
        }
        pub fn next_bit(&self, start: c_int, onBit: bool) -> c_int {
            assert!(onBit);  // I didn't want to add the sz template arg, you could though
            start
        }
    }

    pub mod storage {
        use super::*;

        ////////////////////////////////////////////////////////////////////////////////////////
        // value_semantics: for value types with static allocation
        ////////////////////////////////////////////////////////////////////////////////////////
        // template<class T,int SIZE>
        // struct value_semantics
        // {
        // 	enum
        // 	{
        // 		CAPACITY		= SIZE,
        // 	};
        // 	typedef T TAlign;		// this is any type that has the right alignment
        // 	typedef T TValue;		// this is the actual thing the user uses
        // 	typedef T TStorage;		// this is what we make our array of
        //
        // 	typedef bits_true TConstructed;
        // 	typedef TStorage TArray[SIZE];
        //
        //
        // 	enum
        // 	{
        // 		NEEDS_CONSTRUCT=0,
        // 		TOTAL_SIZE=sizeof(TStorage),
        // 		VALUE_SIZE=sizeof(TStorage),
        // 	};
        // 	static void construct(TStorage *)
        // 	{
        //
        // 	}
        // 	static void construct(TStorage *me,const TValue &v)
        // 	{
        // 		*me=v;
        // 	}
        // 	static void destruct(TStorage *)
        // 	{
        //
        // 	}
        // 	static TRatlNew *raw(TStorage *me)
        // 	{
        // 		return (TRatlNew *)me;
        // 	}
        // 	static T * ptr(TStorage *me)
        // 	{
        // 		return me;
        // 	}
        // 	static const T * ptr(const TStorage *me)
        // 	{
        // 		return me;
        // 	}
        // 	static T & ref(TStorage *me)
        // 	{
        // 		return *me;
        // 	}
        // 	static const T & ref(const TStorage *me)
        // 	{
        // 		return *me;
        // 	}
        // 	static T *raw_array(TStorage *me)
        // 	{
        // 		return me;
        // 	}
        // 	static const T *raw_array(const TStorage *me)
        // 	{
        // 		return me;
        // 	}
        // 	static void swap(TStorage *s1,TStorage *s2)
        // 	{
        // 		mem::swap(ptr(s1),ptr(s2));
        // 	}
        // 	static int pointer_to_index(const void *s1,const void *s2)
        // 	{
        // 		return ((TStorage *)s1)-((TStorage *)s2);
        // 	}
        // };

        pub struct value_semantics<T, const SIZE: usize> {
            _marker: core::marker::PhantomData<T>,
        }

        pub mod value_semantics_traits {
            use super::*;

            pub struct ValueSemantics;

            impl ValueSemantics {
                pub const NEEDS_CONSTRUCT: c_int = 0;

                pub unsafe fn construct<T>(_me: *mut T) {
                    // no-op for POD types
                }

                pub unsafe fn construct_with_value<T>(me: *mut T, v: T) {
                    *me = v;
                }

                pub unsafe fn destruct<T>(_me: *mut T) {
                    // no-op for POD types
                }

                pub unsafe fn raw<T>(me: *mut T) -> *mut TRatlNew {
                    me as *mut TRatlNew
                }

                pub unsafe fn ptr<T>(me: *mut T) -> *mut T {
                    me
                }

                pub unsafe fn ptr_const<T>(me: *const T) -> *const T {
                    me
                }

                pub unsafe fn reference<'a, T>(me: *mut T) -> &'a T {
                    &*me
                }

                pub unsafe fn reference_const<'a, T>(me: *const T) -> &'a T {
                    &*me
                }

                pub unsafe fn raw_array<T>(me: *mut T) -> *mut T {
                    me
                }

                pub unsafe fn raw_array_const<T>(me: *const T) -> *const T {
                    me
                }

                pub unsafe fn swap<T>(s1: *mut T, s2: *mut T) {
                    mem::swap(s1, s2);
                }

                pub unsafe fn pointer_to_index<T>(s1: *const c_void, s2: *const T) -> c_int {
                    ((s1 as *const T).offset_from(s2)) as c_int
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // object_semantics: for non-POD types requiring construction
        ////////////////////////////////////////////////////////////////////////////////////////
        // template<class T,int SIZE>
        // struct object_semantics
        // {
        // 	enum
        // 	{
        // 		CAPACITY		= SIZE,
        // 	};
        // 	typedef mem::alignStruct TAlign;		// this is any type that has the right alignment
        // 	typedef T TValue;				// this is the actual thing the user uses
        //
        // 	typedef bits_base<SIZE> TConstructed;
        //
        // 	struct TStorage
        // 	{
        // 		TAlign mMemory[((sizeof(T) + sizeof(TAlign) -1 )/sizeof(TAlign))];
        // 	};
        // 	typedef TStorage TArray[SIZE];
        //
        // 	enum
        // 	{
        // 		NEEDS_CONSTRUCT=1,
        // 		TOTAL_SIZE=sizeof(TStorage),
        // 		VALUE_SIZE=sizeof(TStorage),
        // 	};
        //
        // 	static void construct(TStorage *me)
        // 	{
        // 		new(raw(me)) TValue();
        // 	}
        // 	static void construct(TStorage *me,const TValue &v)
        // 	{
        // 		new(raw(me)) TValue(v);
        // 	}
        // 	static void destruct(TStorage *me)
        // 	{
        // 		ptr(me)->~T();
        // 	}
        // 	// ... (other methods follow similar pattern)
        // };

        pub struct object_semantics<T, const SIZE: usize> {
            _marker: core::marker::PhantomData<T>,
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // virtual_semantics: for polymorphic types
        ////////////////////////////////////////////////////////////////////////////////////////
        pub struct virtual_semantics<T, const SIZE: usize, const MAX_CLASS_SIZE: usize> {
            _marker: core::marker::PhantomData<T>,
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // value_semantics_node: for node-based value semantics
        ////////////////////////////////////////////////////////////////////////////////////////
        pub struct value_semantics_node<T, const SIZE: usize, NODE> {
            _marker: core::marker::PhantomData<(T, NODE)>,
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // object_semantics_node: for node-based object semantics
        ////////////////////////////////////////////////////////////////////////////////////////
        pub struct object_semantics_node<T, const SIZE: usize, NODE> {
            _marker: core::marker::PhantomData<(T, NODE)>,
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // virtual_semantics_node: for node-based virtual semantics
        ////////////////////////////////////////////////////////////////////////////////////////
        pub struct virtual_semantics_node<T, const SIZE: usize, const MAX_CLASS_SIZE: usize, NODE> {
            _marker: core::marker::PhantomData<(T, NODE)>,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // The Array Base Class, used for most containers
    ////////////////////////////////////////////////////////////////////////////////////////
    // template<class T>
    // class array_base : public ratl_base
    // {
    // public:
    //     ////////////////////////////////////////////////////////////////////////////////////
    // 	// Capacity Enum
    //     ////////////////////////////////////////////////////////////////////////////////////
    //  	enum
    // 	{
    // 		CAPACITY	= T::CAPACITY,
    // 		SIZE		= T::CAPACITY,
    // 	};
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	// Data
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	typedef typename T					TStorageTraits;
    // 	typedef typename T::TArray			TTArray;
    // 	typedef typename T::TValue			TTValue;
    // 	typedef typename T::TConstructed	TTConstructed;
    //
    // private:
    // 	TTArray				mArray;
    // 	TTConstructed		mConstructed;
    //
    // public:
    //
    // 	array_base()
    // 	{
    // 	}
    //
    // 	~array_base()
    // 	{
    // 		clear();
    // 	}
    //
    // 	void clear()
    // 	{
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			int i=mConstructed.next_bit();
    // 			while (i<SIZE)
    // 			{
    // 				T::destruct(mArray+i);
    // 				i=mConstructed.next_bit(i+1);
    // 			}
    // 			mConstructed.clear();
    // 		}
    // 	}
    //
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	// Access Operator
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	TTValue&			operator[](int index)
    // 	{
    // 		assert(index>=0 && index<SIZE);
    // 		assert(mConstructed[index]);
    // 		return T::ref(mArray+index);
    // 	}
    //
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	// Const Access Operator
    // 	////////////////////////////////////////////////////////////////////////////////////
    // 	const TTValue&	operator[](int index) const
    // 	{
    // 		assert(index>=0 && index<SIZE);
    // 		assert(mConstructed[index]);
    // 		return T::ref(mArray+index);
    // 	}
    //
    // 	void construct(int i)
    // 	{
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			assert(!mConstructed[i]);
    // 			T::construct(mArray+i);
    // 			mConstructed.set_bit(i);
    // 		}
    // 	}
    // 	void construct(int i, const TTValue &v)
    // 	{
    // 		assert(i>=0 && i<SIZE);
    // 		T::construct(mArray+i,v);
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			assert(!mConstructed[i]);
    // 			mConstructed.set_bit(i);
    // 		}
    // 	}
    // 	void fill(const TTValue &v)
    // 	{
    // 		clear();
    // 		int i;
    // 		for (i=0;i<SIZE;i++)
    // 		{
    // 			T::construct(mArray+i,v);
    // 		}
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			mConstructed.set();
    // 		}
    // 	}
    // 	void swap(int i,int j)
    // 	{
    // 		assert(i>=0 && i<SIZE);
    // 		assert(j>=0 && j<SIZE);
    // 		assert(i!=j);
    // 		assert(mConstructed[i]);
    // 		assert(mConstructed[j]);
    // 		T::swap(mArray+i,mArray+j);
    // 	}
    //
    // 	TRatlNew	*alloc_raw(int i)
    // 	{
    // 		assert(i>=0 && i<SIZE);
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			assert(!mConstructed[i]);
    // 			mConstructed.set_bit(i);
    // 		}
    // 		return T::raw(mArray+i);
    // 	}
    // 	void	destruct(int i)
    // 	{
    // 		assert(i>=0 && i<SIZE);
    // 		assert(mConstructed[i]);
    // 		if (T::NEEDS_CONSTRUCT)
    // 		{
    // 			T::destruct(mArray+i);
    // 			mConstructed.clear_bit(i);
    // 		}
    // 	}
    // 	int pointer_to_index(const TTValue *me) const
    // 	{
    // 		int index=T::pointer_to_index(me,mArray);
    // 		assert(index>=0 && index<SIZE);
    // 		assert(mConstructed[index]);
    // 		return index;
    // 	}
    // 	int pointer_to_index(const TRatlNew *me) const
    // 	{
    // 		int index=T::pointer_to_index(me,mArray);
    // 		assert(index>=0 && index<SIZE);
    // 		assert(mConstructed[index]);
    // 		return index;
    // 	}
    // 	typename T::TValue *raw_array()
    // 	{
    // 		return T::raw_array(mArray);
    // 	}
    // 	const typename T::TValue *raw_array() const
    // 	{
    // 		return T::raw_array(mArray);
    // 	}
    // 	template<class CAST_TO>
    // 	CAST_TO *verify_alloc(CAST_TO *p) const
    // 	{
    // 		return T::verify_alloc(p);
    // 	}
    //
    // };

    pub struct array_base<T> {
        // Note: This is a placeholder for the template class.
        // Actual specializations and implementations would be generated
        // based on the storage traits T.
        _marker: core::marker::PhantomData<T>,
    }

    impl<T> array_base<T> {
        pub fn new() -> Self {
            array_base {
                _marker: core::marker::PhantomData,
            }
        }
    }
}
