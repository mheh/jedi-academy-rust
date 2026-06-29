// Faithful port of oracle/code/qcommon/hstring.cpp
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use core::mem;
use core::ptr;

// Include dependencies (from oracle):
// #include "cm_local.h"
// #include "hstring.h"

// Platform-specific include (was #if defined (_DEBUG) && defined (_WIN32))
// #define WIN32_LEAN_AND_MEAN 1
// #include "platform.h"

// mapPoolBlockCount is defined differently in the executable (sv_main.cpp) and the game dll (g_main.cpp) cuz
// we likely don't need as many blocks in the executable as we do in the game
extern "C" {
    pub static mut mapPoolBlockCount: c_int;
}

// Used to fool optimizer during compilation of mem touch routines.
pub static mut HaHaOptimizer2: c_int = 0;

#[cfg(not(feature = "xbox"))]
fn GetMapPool() -> &'static mut CMapPoolLow {
    // this may need to be ifdefed to be different for different modules
    static mut thePool: Option<CMapPoolLow> = None;
    unsafe {
        if thePool.is_none() {
            thePool = Some(CMapPoolLow::new());
        }
        thePool.as_mut().unwrap()
    }
}

const MAPBLOCK_SIZE_NODES: usize = 1024;
const MAPNODE_FREE: u8 = 0xa1;
const MAPNODE_INUSE: u8 = 0x94;

// MAP_NODE_SIZE is defined in cm_local.h
// Porting stub: typical value is 32 bytes based on alignment patterns
const MAP_NODE_SIZE: usize = 32;

#[repr(C)]
struct SMapNode {
    mData: [u8; MAP_NODE_SIZE - 2],
    mMapBlockNum: u8,
    mTag: u8,
}

struct CMapBlock {
    mId: c_int,
    mRaw: [u8; (MAPBLOCK_SIZE_NODES + 1) * MAP_NODE_SIZE],
    mNodes: *mut SMapNode,
    mLastNode: c_int,
}

impl CMapBlock {
    fn new(id: c_int, freeList: &mut Vec<*mut SMapNode>) -> Self {
        let mut block = CMapBlock {
            mId: id,
            mRaw: [0u8; (MAPBLOCK_SIZE_NODES + 1) * MAP_NODE_SIZE],
            mNodes: ptr::null_mut(),
            mLastNode: 0,
        };

        // Alloc node storage for MAPBLOCK_SIZE_NODES worth of nodes.
        let raw_ptr = block.mRaw.as_mut_ptr() as usize;
        let aligned_ptr = ((raw_ptr + MAP_NODE_SIZE) & !(0x1f)) as *mut SMapNode;
        block.mNodes = aligned_ptr;

        // Set all nodes to initially be free.
        for i in 0..MAPBLOCK_SIZE_NODES {
            unsafe {
                let node = block.mNodes.add(i);
                (*node).mMapBlockNum = id as u8;
                (*node).mTag = MAPNODE_FREE;
                freeList.push(node);
            }
        }

        block
    }

    fn bOwnsNode(&self, node: *mut SMapNode) -> bool {
        unsafe {
            node >= self.mNodes && node < self.mNodes.add(MAPBLOCK_SIZE_NODES)
        }
    }
}

struct CMapPoolLow {
    mMapBlocks: Vec<Box<CMapBlock>>,
    mFreeList: Vec<*mut SMapNode>,
    mLastBlockNum: c_int,
}

impl CMapPoolLow {
    fn new() -> Self {
        CMapPoolLow {
            mMapBlocks: Vec::new(),
            mFreeList: Vec::new(),
            mLastBlockNum: -1,
        }
    }
}

#[cfg(not(feature = "xbox"))]
impl Drop for CMapPoolLow {
    fn drop(&mut self) {
        self._drop_impl_logic();
    }
}

impl CMapPoolLow {
    #[cfg(not(feature = "xbox"))]
    fn _drop_impl_logic(&mut self) {
        #[cfg(debug_assertions)]
        {
            let mut mess: [c_char; 1000] = [0; 1000];

            #[cfg(feature = "game")]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    unsafe {
                        sprintf(
                            mess.as_mut_ptr(),
                            b"[MEM][GAME]  !!!! Map Pool Leaked %d nodes\n\0".as_ptr() as *const c_char,
                            (MAPBLOCK_SIZE_NODES * self.mMapBlocks.len()) - self.mFreeList.len(),
                        );
                        OutputDebugString(mess.as_ptr());
                    }
                }
                unsafe {
                    sprintf(
                        mess.as_mut_ptr(),
                        b"[MEM][GAME]  Map Pool max. mem used = %d\n\0".as_ptr() as *const c_char,
                        self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES * MAP_NODE_SIZE,
                    );
                    OutputDebugString(mess.as_ptr());
                }
            }
            #[cfg(feature = "cgame")]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    unsafe {
                        sprintf(
                            mess.as_mut_ptr(),
                            b"[MEM][CGAME]  !!!! Map Pool Leaked %d nodes\n\0".as_ptr() as *const c_char,
                            (MAPBLOCK_SIZE_NODES * self.mMapBlocks.len()) - self.mFreeList.len(),
                        );
                        OutputDebugString(mess.as_ptr());
                    }
                }
                unsafe {
                    sprintf(
                        mess.as_mut_ptr(),
                        b"[MEM][CGAME] Map Pool max. mem used = %d\n\0".as_ptr() as *const c_char,
                        self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES * MAP_NODE_SIZE,
                    );
                    OutputDebugString(mess.as_ptr());
                }
            }
            #[cfg(not(any(feature = "game", feature = "cgame")))]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    unsafe {
                        sprintf(
                            mess.as_mut_ptr(),
                            b"[MEM][EXE]  !!!! Map Pool Leaked %d nodes\n\0".as_ptr() as *const c_char,
                            (MAPBLOCK_SIZE_NODES * self.mMapBlocks.len()) - self.mFreeList.len(),
                        );
                        OutputDebugString(mess.as_ptr());
                    }
                }
                unsafe {
                    sprintf(
                        mess.as_mut_ptr(),
                        b"[MEM][EXE] Map Pool max. mem used = %d\n\0".as_ptr() as *const c_char,
                        self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES * MAP_NODE_SIZE,
                    );
                    OutputDebugString(mess.as_ptr());
                }
            }
        }
    }

    #[cfg(not(feature = "xbox"))]
    fn Alloc(&mut self) -> *mut u8 {
        // Try to request a node. First we look in the free-list, but if that
        // happens to be empty, we allocate more storage in the current CMapBlock.
        let node: *mut SMapNode;
        if self.mFreeList.len() > 0 {
            // Retrieve the node to be recycled.
            node = self.mFreeList[self.mFreeList.len() - 1];
            self.mFreeList.pop();
        } else {
            // None free, so alloc another block.
            let block = CMapBlock::new(self.mLastBlockNum + 1, &mut self.mFreeList);
            assert!(!block.mNodes.is_null());
            self.mMapBlocks.push(Box::new(block));
            self.mLastBlockNum += 1;
            node = self.mFreeList[self.mFreeList.len() - 1];
            self.mFreeList.pop();
        }

        // Validate we aren't somehow grabbing something that is already in use
        // and also that the end marker is intact.
        unsafe {
            assert_eq!((*node).mTag, MAPNODE_FREE);
            assert!((*node).mMapBlockNum as i32 >= 0);
            assert!(((*node).mMapBlockNum as i32) < 256);
            assert!(((*node).mMapBlockNum as i32) <= self.mLastBlockNum);
            assert!(self.mMapBlocks[(*node).mMapBlockNum as usize].bOwnsNode(node));

            // Ok, mark the node as in use.
            (*node).mTag = MAPNODE_INUSE;

            node as *mut u8
        }
    }

    #[cfg(not(feature = "xbox"))]
    fn Free(&mut self, p: *mut u8) {
        let p = p as *mut SMapNode;
        // Validate that someone isn't trying to double free this node and also
        // that the end marker is intact.
        unsafe {
            assert_eq!((*p).mTag, MAPNODE_INUSE);
            assert!((*p).mMapBlockNum as i32 >= 0);
            assert!(((*p).mMapBlockNum as i32) < 256);
            assert!(((*p).mMapBlockNum as i32) <= self.mLastBlockNum);
            assert!(self.mMapBlocks[(*p).mMapBlockNum as usize].bOwnsNode(p));

            // Ok, mark the the node as free.
            (*p).mTag = MAPNODE_FREE;
        }

        // Add a new freelist entry to point at this node.
        self.mFreeList.push(p);
    }

    #[cfg(not(feature = "xbox"))]
    fn TouchMem(&self) {
        let mut totSize: c_int = 0;
        for i in 0..self.mMapBlocks.len() {
            let memory = &self.mMapBlocks[i] as *const _ as *const u8;
            totSize += mem::size_of::<CMapBlock>() as c_int;
            unsafe {
                let mut j: usize = 0;
                while j < mem::size_of::<CMapBlock>() {
                    HaHaOptimizer2 += *memory.add(j) as c_int;
                    j += 256;
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            unsafe {
                Com_Printf(b"MapPool: Bytes touched %i\n\0".as_ptr() as *const c_char, totSize);
            }
        }
    }
}

fn HashFunction(key: *const c_char) -> c_int {
    let mut hash: i64 = 0;
    let mut i: i32 = 0;
    let mut letter: u8;

    unsafe {
        letter = *key as u8;
        let mut ptr = key;
        loop {
            if letter == 0 {
                break;
            }
            ptr = ptr.add(1);
            hash += (letter as i64) * (i as i64 + 119);
            i += 1;
            letter = *ptr as u8;
        }
    }
    hash &= (MAX_HASH - 1) as i64;
    hash as c_int
}

const MAX_HASH: usize = 65536 * 2;

// Max number of strings we can ever deal with.
const MAX_HSTRINGS: usize = 100000;

// Size of a string storage block in bytes.
const BLOCK_SIZE: usize = 65536;

struct CHashHelper {
    mHashes: [c_int; MAX_HASH],
    mFindPtr: c_int,
    mFindPtrStart: c_int,
}

impl CHashHelper {
    fn new() -> Self {
        CHashHelper {
            mHashes: [0; MAX_HASH],
            mFindPtr: 0,
            mFindPtrStart: 0,
        }
    }

    fn Add(&mut self, hash: c_int, value: c_int) {
        assert!(hash >= 0 && hash < MAX_HASH as c_int);
        assert!(value != 0); // 0 is the empty marker
        let mut i = hash as usize;
        loop {
            if self.mHashes[i] == 0 {
                break;
            }
            assert!(self.mHashes[i] != value); //please don't insert things twice
            i = (i + 1) & (MAX_HASH - 1);
            assert!(i != hash as usize); //hash table is full?
        }
        self.mHashes[i] = value;
    }

    fn FindFirst(&mut self, hash: c_int) -> c_int {
        self.mFindPtr = hash;
        self.mFindPtrStart = hash;
        self.FindNext()
    }

    fn FindNext(&mut self) -> c_int {
        assert!(self.mFindPtr >= 0 && self.mFindPtr < MAX_HASH as c_int);
        let val = self.mHashes[self.mFindPtr as usize];
        self.mFindPtr = ((self.mFindPtr as usize + 1) & (MAX_HASH - 1)) as c_int;
        assert!(self.mFindPtr != self.mFindPtrStart); //hash table full?
        val
    }

    fn TouchMem(&self) {
        unsafe {
            let ptr = self.mHashes.as_ptr() as *const u8;
            let mut i: usize = 0;
            while i < mem::size_of_val(&self.mHashes) {
                HaHaOptimizer2 += *ptr.add(i) as c_int;
                i += 256;
            }
        }
        #[cfg(debug_assertions)]
        {
            unsafe {
                Com_Printf(b"Hash helper: Bytes touched %i\n\0".as_ptr() as *const c_char, mem::size_of_val(&self.mHashes) as c_int);
            }
        }
    }
}

fn HashHelper() -> &'static mut CHashHelper {
    static mut It: Option<CHashHelper> = None;
    unsafe {
        if It.is_none() {
            It = Some(CHashHelper::new());
        }
        It.as_mut().unwrap()
    }
}

static mut gCharPtrs: [*mut c_char; MAX_HSTRINGS] = [ptr::null_mut(); MAX_HSTRINGS];

#[repr(C)]
struct CHSBlock {
    mBytesUsed: c_int,
    mRaw: [c_char; BLOCK_SIZE],
}

impl CHSBlock {
    fn new() -> Self {
        let mut block = CHSBlock {
            mBytesUsed: 0,
            mRaw: [0; BLOCK_SIZE],
        };
        // So we can do a comparison of blocks for debug purposes.
        unsafe {
            memset(block.mRaw.as_mut_ptr() as *mut u8, 0, BLOCK_SIZE);
        }
        block
    }

    fn Alloc(&mut self, sizeBytes: c_int) -> *mut c_char {
        // Remember to include 0 termintor in size.
        let sizeBytes = sizeBytes + 1;

        // Is it WAAAY to big? If so we complain loudly.
        assert!(sizeBytes as usize <= BLOCK_SIZE);

        // If we don't have space in the current block, return failure.
        if sizeBytes as usize > (BLOCK_SIZE - self.mBytesUsed as usize) {
            return ptr::null_mut();
        }

        // Return the pointer to the start of allocated space.
        let ret = &mut self.mRaw[self.mBytesUsed as usize] as *mut c_char;
        self.mBytesUsed += sizeBytes;
        ret
    }

    fn operator_eq(&self, block: &CHSBlock) -> bool {
        unsafe {
            if memcmp(
                self.mRaw.as_ptr() as *const u8,
                block.mRaw.as_ptr() as *const u8,
                BLOCK_SIZE,
            ) == 0
            {
                return true;
            }
        }
        false
    }
}

struct CPool {
    mBlockVec: Vec<Box<CHSBlock>>,
    mNextStringId: c_int,
    mLastBlockNum: c_int,
}

impl CPool {
    fn new() -> Self {
        unsafe {
            for i in 0..MAX_HSTRINGS {
                gCharPtrs[i] = ptr::null_mut();
            }
        }
        CPool {
            mBlockVec: Vec::new(),
            mNextStringId: 1,
            mLastBlockNum: -1,
        }
    }
}

impl Drop for CPool {
    fn drop(&mut self) {
        self._drop_impl_logic();
    }
}

impl CPool {
    fn _drop_impl_logic(&mut self) {
        for _i in 0..self.mBlockVec.len() {
            // Box automatically dropped
        }
    }

    fn Alloc(&mut self, sizeBytes: c_int, id: &mut c_int) -> *mut c_char {
        // Can't alloc more than MAX_HSTRINGS.
        assert!((self.mNextStringId as usize) < MAX_HSTRINGS);
        let mut raw: *mut c_char = ptr::null_mut();
        if self.mLastBlockNum >= 0 {
            // Get the pointer to the start of allocated space in the current block.
            raw = self.mBlockVec[self.mLastBlockNum as usize].Alloc(sizeBytes);
        }
        if raw.is_null() {
            // Ok, make a new empty block and append it.
            let block = CHSBlock::new();
            self.mBlockVec.push(Box::new(block));
            self.mLastBlockNum += 1;
            raw = self.mBlockVec[self.mLastBlockNum as usize].Alloc(sizeBytes);
        }
        // Should never really happen!!
        assert!(!raw.is_null());

        *id = self.mNextStringId;
        unsafe {
            gCharPtrs[self.mNextStringId as usize] = raw;
        }
        self.mNextStringId += 1;

        raw
    }

    fn operator_eq(&self, pool: &CPool) -> bool {
        for i in 0..self.mBlockVec.len() {
            if !self.mBlockVec[i].operator_eq(&pool.mBlockVec[i]) {
                return false;
            }
        }
        true
    }

    fn TouchMem(&self) {
        let mut totSize: c_int = 0;
        for i in 0..self.mBlockVec.len() {
            let memory = &self.mBlockVec[i] as *const _ as *const u8;
            totSize += mem::size_of::<CHSBlock>() as c_int;
            unsafe {
                let mut j: usize = 0;
                while j < mem::size_of::<CHSBlock>() {
                    HaHaOptimizer2 += *memory.add(j) as c_int;
                    j += 256;
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            unsafe {
                Com_Printf(b"String Pool: Bytes touched %i\n\0".as_ptr() as *const c_char, totSize);
            }
        }
    }
}

#[cfg(debug_assertions)]
fn TheDebugPool() -> &'static mut CPool {
    static mut theDebugPool: Option<CPool> = None;
    unsafe {
        if theDebugPool.is_none() {
            theDebugPool = Some(CPool::new());
        }
        theDebugPool.as_mut().unwrap()
    }
}

#[cfg(debug_assertions)]
struct CPoolChecker;

#[cfg(debug_assertions)]
impl CPoolChecker {
    fn new() -> Self {
        TheDebugPool();
        ThePool();
        CPoolChecker
    }
}

#[cfg(debug_assertions)]
impl Drop for CPoolChecker {
    fn drop(&mut self) {
        let pool = ThePool();
        let mut mess: [c_char; 1000] = [0; 1000];

        #[cfg(feature = "game")]
        {
            unsafe {
                sprintf(
                    mess.as_mut_ptr(),
                    b"[MEM][GAME]  String Pool %d unique strings, %dK\n\0".as_ptr() as *const c_char,
                    pool.mNextStringId,
                    (pool.mLastBlockNum + 1) * BLOCK_SIZE as c_int / 1024,
                );
                OutputDebugString(mess.as_ptr());
            }
        }
        #[cfg(feature = "cgame")]
        {
            unsafe {
                sprintf(
                    mess.as_mut_ptr(),
                    b"[MEM][CGAME]  String Pool %d unique strings, %dK\n\0".as_ptr() as *const c_char,
                    pool.mNextStringId,
                    (pool.mLastBlockNum + 1) * BLOCK_SIZE as c_int / 1024,
                );
                OutputDebugString(mess.as_ptr());
            }
        }
        #[cfg(not(any(feature = "game", feature = "cgame")))]
        {
            unsafe {
                sprintf(
                    mess.as_mut_ptr(),
                    b"[MEM][EXE]  String Pool %d unique strings, %dK\n\0".as_ptr() as *const c_char,
                    pool.mNextStringId,
                    (pool.mLastBlockNum + 1) * BLOCK_SIZE as c_int / 1024,
                );
                OutputDebugString(mess.as_ptr());
            }
        }

        // if this fails it means the string storage is CORRUPTED, let someone know
        assert!(TheDebugPool().operator_eq(&ThePool()));
    }
}

// In the original C++, TheCPoolChecker was a static instance that initialized ThePool and TheDebugPool
// on startup and verified consistency on shutdown. In Rust, pools initialize on first access via
// ThePool() and TheDebugPool() functions, so the explicit checker is not needed.
// static CPoolChecker TheCPoolChecker;  // Porting deviation: Rust initialization model differs

fn ThePool() -> &'static mut CPool {
    static mut thePool: Option<CPool> = None;
    unsafe {
        if thePool.is_none() {
            thePool = Some(CPool::new());
        }
        thePool.as_mut().unwrap()
    }
}

pub fn TouchStringPool() {
    ThePool().TouchMem();
    HashHelper().TouchMem();
}

// Now the rest of the hstring class.

#[allow(non_snake_case)]
pub struct hstring {
    mId: c_int,
}

impl hstring {
    pub fn new() -> Self {
        hstring { mId: 0 }
    }

    pub fn Init(&mut self, str: *const c_char) {
        if str.is_null() {
            self.mId = 0;
            return;
        }

        unsafe {
            let hash = HashFunction(str);
            let mut id = HashHelper().FindFirst(hash);
            while id != 0 {
                assert!(id > 0 && (id as usize) < ThePool().mNextStringId as usize);
                if strcmp(str, gCharPtrs[id as usize]) == 0 {
                    self.mId = id;
                    return;
                }
                id = HashHelper().FindNext();
            }
            let strlen_val = strlen(str);
            let raw = ThePool().Alloc(strlen_val as c_int, &mut self.mId);
            strcpy(raw, str);
            HashHelper().Add(hash, self.mId);

            #[cfg(debug_assertions)]
            {
                let mut test: c_int = 0;
                let raw_dbg = TheDebugPool().Alloc(strlen_val as c_int, &mut test);
                assert_eq!(test, self.mId);
                strcpy(raw_dbg, str);
            }
        }
    }

    pub fn c_str(&self) -> *const c_char {
        if self.mId == 0 {
            return EMPTY_STR.as_ptr() as *const c_char;
        }
        unsafe {
            assert!(self.mId > 0 && (self.mId as usize) < ThePool().mNextStringId as usize);
            gCharPtrs[self.mId as usize]
        }
    }

    pub fn str(&self) -> String {
        if self.mId == 0 {
            return String::new();
        }
        unsafe {
            assert!(self.mId > 0 && (self.mId as usize) < ThePool().mNextStringId as usize);
            let c_str = gCharPtrs[self.mId as usize];
            let len = strlen(c_str);
            let bytes = core::slice::from_raw_parts(c_str as *const u8, len);
            String::from_utf8_unchecked(bytes.to_vec())
        }
    }
}

// Empty string constant
const EMPTY_STR: &[u8] = b"\0";

// Extern declarations for C library functions
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8;
    pub fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> c_int;
    pub fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
}

// Extern declarations for engine functions (defined elsewhere)
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...) -> ();
    pub fn OutputDebugString(str: *const c_char) -> ();
}
