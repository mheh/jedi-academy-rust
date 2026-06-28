#![allow(non_snake_case)]

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

use core::ffi::c_int;
use std::ffi::CStr;

// mapPoolBlockCount is defined differently in the executable (sv_main.cpp) and the game dll (g_main.cpp) cuz
//we likely don't need as many blocks in the executable as we do in the game
extern "C" {
    pub static mapPoolBlockCount: c_int;
}

// Used to fool optimizer during compilation of mem touch routines.
static mut HaHaOptimizer2: c_int = 0;

const MAPBLOCK_SIZE_NODES: usize = 1024;
const MAPNODE_FREE: u8 = 0xa1;
const MAPNODE_INUSE: u8 = 0x94;
const MAP_NODE_SIZE: usize = 32;

#[repr(C)]
struct SMapNode {
    mData: [u8; MAP_NODE_SIZE - 2],
    mMapBlockNum: u8,
    mTag: u8,
}

struct CMapBlock {
    mId: c_int,
    mRaw: Vec<u8>,
    mLastNode: c_int,
}

impl CMapBlock {
    fn new(id: c_int, freeList: &mut Vec<*mut std::ffi::c_void>) -> Self {
        let mut block = CMapBlock {
            mId: id,
            mRaw: vec![0u8; (MAPBLOCK_SIZE_NODES + 1) * MAP_NODE_SIZE],
            mLastNode: 0,
        };

        // Alloc node storage for MAPBLOCK_SIZE_NODES worth of nodes.
        // In C++: mNodes=(SMapNode *)((((unsigned long)mRaw)+MAP_NODE_SIZE)&~(unsigned long)0x1f);
        let base_ptr = block.mRaw.as_mut_ptr();
        let aligned_offset = {
            let addr = base_ptr as usize + MAP_NODE_SIZE;
            (addr) & !0x1f
        };
        let mNodes = aligned_offset as *mut SMapNode;

        // Set all nodes to initially be free.
        for i in 0..MAPBLOCK_SIZE_NODES {
            unsafe {
                (*mNodes.add(i)).mMapBlockNum = id as u8;
                (*mNodes.add(i)).mTag = MAPNODE_FREE;
                freeList.push(&mut *mNodes.add(i) as *mut SMapNode as *mut std::ffi::c_void);
            }
        }

        block
    }

    fn bOwnsNode(&self, node: *mut std::ffi::c_void) -> bool {
        let base_ptr = self.mRaw.as_ptr();
        let aligned_offset = {
            let addr = base_ptr as usize + MAP_NODE_SIZE;
            (addr) & !0x1f
        };
        let mNodes = aligned_offset as *const SMapNode;

        let node_ptr = node as *const SMapNode;
        unsafe {
            node_ptr >= mNodes && node_ptr < mNodes.add(MAPBLOCK_SIZE_NODES)
        }
    }
}

struct CMapPoolLow {
    mMapBlocks: Vec<Box<CMapBlock>>,
    mFreeList: Vec<*mut std::ffi::c_void>,
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

    fn Alloc(&mut self) -> *mut std::ffi::c_void {
        // Try to request a node. First we look in the free-list, but if that
        // happens to be empty, we allocate more storage in the current CMapBlock.
        let mut node: *mut std::ffi::c_void = std::ptr::null_mut();
        if self.mFreeList.len() > 0 {
            // Retrieve the node to be recycled.
            node = self.mFreeList[self.mFreeList.len() - 1];
            self.mFreeList.pop();
        } else {
            // None free, so alloc another block.
            let mut block = Box::new(CMapBlock::new(self.mLastBlockNum + 1, &mut self.mFreeList));
            self.mMapBlocks.push(block);
            self.mLastBlockNum += 1;
            node = self.mFreeList[self.mFreeList.len() - 1];
            self.mFreeList.pop();
        }

        // Validate we aren't somehow grabbing something that is already in use
        // and also that the end marker is intact.
        unsafe {
            let smap_node = node as *mut SMapNode;
            debug_assert!((*smap_node).mTag == MAPNODE_FREE);
            debug_assert!((*smap_node).mMapBlockNum >= 0);
            debug_assert!((*smap_node).mMapBlockNum < 256);
            debug_assert!((*smap_node).mMapBlockNum <= self.mLastBlockNum as u8);
        }

        // Ok, mark the node as in use.
        unsafe {
            let smap_node = node as *mut SMapNode;
            (*smap_node).mTag = MAPNODE_INUSE;
        }

        node
    }

    fn Free(&mut self, p: *mut std::ffi::c_void) {
        // Validate that someone isn't trying to double free this node and also
        // that the end marker is intact.
        unsafe {
            let smap_node = p as *mut SMapNode;
            debug_assert!((*smap_node).mTag == MAPNODE_INUSE);
            debug_assert!((*smap_node).mMapBlockNum >= 0);
            debug_assert!((*smap_node).mMapBlockNum < 256);
            debug_assert!((*smap_node).mMapBlockNum <= self.mLastBlockNum as u8);
        }

        // Ok, mark the the node as free.
        unsafe {
            let smap_node = p as *mut SMapNode;
            (*smap_node).mTag = MAPNODE_FREE;
        }

        // Add a new freelist entry to point at this node.
        self.mFreeList.push(p);
    }

    fn TouchMem(&self) {
        let mut i: usize = 0;
        let mut totSize: c_int = 0;
        while i < self.mMapBlocks.len() {
            let memory = self.mMapBlocks[i].as_ref() as *const CMapBlock as *const u8;
            totSize += std::mem::size_of::<CMapBlock>() as c_int;
            let mut j: usize = 0;
            while j < std::mem::size_of::<CMapBlock>() {
                unsafe {
                    HaHaOptimizer2 += *memory.add(j) as c_int;
                }
                j += 256;
            }
            i += 1;
        }
        #[cfg(debug_assertions)]
        {
            //	Com_Printf("MapPool: Bytes touched %i\n",totSize);
        }
    }
}

impl Drop for CMapPoolLow {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            #[cfg(any(feature = "game"))]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    //		Com_Printf("[MEM][GAME]  !!!! Map Pool Leaked %d nodes\n",(MAPBLOCK_SIZE_NODES*mMapBlocks.size())-mFreeList.size());
                }
                //	Com_Printf("[MEM][GAME]  Map Pool max. mem used = %d\n",mMapBlocks.size()*MAPBLOCK_SIZE_NODES*MAP_NODE_SIZE);
            }
            #[cfg(any(feature = "cgame"))]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    //		Com_Printf("[MEM][CGAME]  !!!! Map Pool Leaked %d nodes\n",(MAPBLOCK_SIZE_NODES*mMapBlocks.size())-mFreeList.size());
                }
                //	Com_Printf("[MEM][CGAME] Map Pool max. mem used = %d\n",mMapBlocks.size()*MAPBLOCK_SIZE_NODES*MAP_NODE_SIZE);
            }
            #[cfg(not(any(feature = "game", feature = "cgame")))]
            {
                if self.mFreeList.len() < self.mMapBlocks.len() * MAPBLOCK_SIZE_NODES {
                    //		Com_Printf("[MEM][EXE]  !!!! Map Pool Leaked %d nodes\n",(MAPBLOCK_SIZE_NODES*mMapBlocks.size())-mFreeList.size());
                }
                //	Com_Printf("[MEM][EXE] Map Pool max. mem used = %d\n",mMapBlocks.size()*MAPBLOCK_SIZE_NODES*MAP_NODE_SIZE);
            }
        }
    }
}

fn GetMapPool() -> &'static mut CMapPoolLow {
    // this may need to be ifdefed to be different for different modules
    static mut THE_POOL: Option<CMapPoolLow> = None;
    unsafe {
        if THE_POOL.is_none() {
            THE_POOL = Some(CMapPoolLow::new());
        }
        THE_POOL.as_mut().unwrap()
    }
}

////////////
// hString stuff
////////////
const MAX_HASH: usize = 65536 * 2;

// Max number of strings we can ever deal with.
const MAX_STRINGS: usize = 100000;

// Size of a string storage block in bytes.
const BLOCK_SIZE: usize = 65536;

fn HashFunction(key: *const i8) -> c_int {
    let mut hash: i64 = 0;
    let mut i: c_int = 0;
    let mut letter: u8;

    unsafe {
        let mut key_ptr = key;
        letter = *key_ptr as u8;
        key_ptr = key_ptr.add(1);
        while letter != 0 {
            hash += (letter as i64) * ((i as i64) + 119);
            i += 1;
            letter = *key_ptr as u8;
            key_ptr = key_ptr.add(1);
        }
    }
    hash &= (MAX_HASH - 1) as i64;
    (hash) as c_int
}

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
        debug_assert!(hash >= 0 && hash < MAX_HASH as c_int);
        debug_assert!(value != 0); // 0 is the empty marker
        let mut i = hash;
        loop {
            if self.mHashes[i as usize] == 0 {
                break;
            }
            debug_assert!(self.mHashes[i as usize] != value); //please don't insert things twice
            i = ((i as usize + 1) & (MAX_HASH - 1)) as c_int;
            debug_assert!(i != hash); //hash table is full?
        }
        self.mHashes[i as usize] = value;
    }

    fn FindFirst(&mut self, hash: c_int) -> c_int {
        self.mFindPtr = hash;
        self.mFindPtrStart = hash;
        self.FindNext()
    }

    fn FindNext(&mut self) -> c_int {
        debug_assert!(self.mFindPtr >= 0 && self.mFindPtr < MAX_HASH as c_int);
        let val = self.mHashes[self.mFindPtr as usize];
        self.mFindPtr = ((self.mFindPtr as usize + 1) & (MAX_HASH - 1)) as c_int;
        debug_assert!(self.mFindPtr != self.mFindPtrStart); //hash table full?
        val
    }

    fn TouchMem(&self) {
        let mut i: usize = 0;
        while i < std::mem::size_of_val(&self.mHashes) {
            unsafe {
                HaHaOptimizer2 += ((&self.mHashes as *const _ as *const u8).add(i).read()) as c_int;
            }
            i += 256;
        }
        #[cfg(debug_assertions)]
        {
            //		Com_Printf("Hash helper: Bytes touched %i\n",sizeof(mHashes));
        }
    }
}

fn HashHelper() -> &'static mut CHashHelper {
    static mut IT: Option<CHashHelper> = None;
    unsafe {
        if IT.is_none() {
            IT = Some(CHashHelper::new());
        }
        IT.as_mut().unwrap()
    }
}

static mut gCharPtrs: [*const i8; MAX_STRINGS] = [std::ptr::null(); MAX_STRINGS];

struct CHSBlock {
    mBytesUsed: c_int,
    mRaw: [i8; BLOCK_SIZE],
}

impl CHSBlock {
    fn new() -> Self {
        CHSBlock {
            mBytesUsed: 0,
            mRaw: [0; BLOCK_SIZE],
        }
    }

    fn Alloc(&mut self, sizeBytes: c_int) -> *mut i8 {
        // Remember to include 0 termintor in size.
        let mut size_bytes = sizeBytes + 1;

        // Is it WAAAY to big? If so we complain loudly.
        debug_assert!(size_bytes <= BLOCK_SIZE as c_int);

        // If we don't have space in the current block, return failure.
        if size_bytes > ((BLOCK_SIZE as c_int) - self.mBytesUsed) {
            return std::ptr::null_mut();
        }

        // Return the pointer to the start of allocated space.
        let ret = &mut self.mRaw[self.mBytesUsed as usize] as *mut i8;
        self.mBytesUsed += size_bytes;
        ret
    }
}

impl PartialEq for CHSBlock {
    fn eq(&self, block: &CHSBlock) -> bool {
        unsafe {
            libc::memcmp(
                self.mRaw.as_ptr() as *const std::ffi::c_void,
                block.mRaw.as_ptr() as *const std::ffi::c_void,
                BLOCK_SIZE,
            ) == 0
        }
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
            libc::memset(
                gCharPtrs.as_mut_ptr() as *mut std::ffi::c_void,
                0,
                MAX_STRINGS * std::mem::size_of::<*const i8>(),
            );
        }
        CPool {
            mBlockVec: Vec::new(),
            mNextStringId: 1,
            mLastBlockNum: -1,
        }
    }

    fn Alloc(&mut self, sizeBytes: c_int) -> (*mut i8, c_int) {
        // Can't alloc more than MAX_STRINGS.
        debug_assert!(self.mNextStringId < MAX_STRINGS as c_int);
        let mut raw: *mut i8 = std::ptr::null_mut();
        if self.mLastBlockNum >= 0 {
            // Get the pointer to the start of allocated space in the current block.
            raw = self.mBlockVec[self.mLastBlockNum as usize].Alloc(sizeBytes);
        }
        if raw.is_null() {
            // Ok, make a new empty block and append it.
            let block = Box::new(CHSBlock::new());
            self.mBlockVec.push(block);
            self.mLastBlockNum += 1;
            raw = self.mBlockVec[self.mLastBlockNum as usize].Alloc(sizeBytes);
        }
        // Should never really happen!!
        debug_assert!(!raw.is_null());

        let id = self.mNextStringId;
        unsafe {
            gCharPtrs[self.mNextStringId as usize] = raw as *const i8;
        }
        self.mNextStringId += 1;

        (raw, id)
    }
}

impl PartialEq for CPool {
    fn eq(&self, pool: &CPool) -> bool {
        for i in 0..self.mBlockVec.len() {
            if self.mBlockVec[i].as_ref() != pool.mBlockVec[i].as_ref() {
                return false;
            }
        }
        true
    }
}

impl CPool {
    fn TouchMem(&self) {
        let mut i: usize = 0;
        let mut totSize: c_int = 0;
        while i < self.mBlockVec.len() {
            let memory = self.mBlockVec[i].as_ref() as *const CHSBlock as *const u8;
            totSize += std::mem::size_of::<CHSBlock>() as c_int;
            let mut j: usize = 0;
            while j < std::mem::size_of::<CHSBlock>() {
                unsafe {
                    HaHaOptimizer2 += *memory.add(j) as c_int;
                }
                j += 256;
            }
            i += 1;
        }
        #[cfg(debug_assertions)]
        {
            //		Com_Printf("String Pool: Bytes touched %i\n",totSize);
        }
    }
}

impl Drop for CPool {
    fn drop(&mut self) {
    }
}

#[cfg(debug_assertions)]
struct CPoolChecker;

#[cfg(debug_assertions)]
impl Drop for CPoolChecker {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            // #if 0
            // {
            //     let mut i: c_int = 1;
            //     while i < ThePool().mNextStringId {
            //         // OutputDebugString(gCharPtrs[i]);
            //         // OutputDebugString("\n");
            //         i += 1;
            //     }
            // }
            // #endif
            #[cfg(any(feature = "game"))]
            {
                //		Com_Printf("[MEM][GAME]  String Pool %d unique strings, %dK\n",ThePool().mNextStringId,(ThePool().mLastBlockNum+1)*BLOCK_SIZE/1024);
            }
            #[cfg(any(feature = "cgame"))]
            {
                //		Com_Printf("[MEM][CGAME]  String Pool %d unique strings, %dK\n",ThePool().mNextStringId,(ThePool().mLastBlockNum+1)*BLOCK_SIZE/1024);
            }
            #[cfg(not(any(feature = "game", feature = "cgame")))]
            {
                //		Com_Printf("[MEM][EXE]  String Pool %d unique strings, %dK\n",ThePool().mNextStringId,(ThePool().mLastBlockNum+1)*BLOCK_SIZE/1024);
            }
        }
        // if this fails it means the string storage is CORRUPTED, let someone know
        #[cfg(debug_assertions)]
        {
            debug_assert!(TheDebugPool() == ThePool());
        }
    }
}

#[cfg(debug_assertions)]
static THE_CPOOL_CHECKER: CPoolChecker = CPoolChecker;

fn TheDebugPool() -> &'static mut CPool {
    static mut THE_DEBUG_POOL: Option<CPool> = None;
    unsafe {
        if THE_DEBUG_POOL.is_none() {
            THE_DEBUG_POOL = Some(CPool::new());
        }
        THE_DEBUG_POOL.as_mut().unwrap()
    }
}

fn ThePool() -> &'static mut CPool {
    static mut THE_POOL: Option<CPool> = None;
    unsafe {
        if THE_POOL.is_none() {
            THE_POOL = Some(CPool::new());
        }
        THE_POOL.as_mut().unwrap()
    }
}

pub fn TouchStringPool() {
    ThePool().TouchMem();
    HashHelper().TouchMem();
}

//
// Now the rest of the hString class.
//

#[repr(C)]
pub struct hstring {
    mId: c_int,
}

impl hstring {
    pub fn new() -> Self {
        hstring { mId: 0 }
    }

    pub fn from_cstr(str_ptr: *const i8) -> Self {
        let mut hs = hstring { mId: 0 };
        hs.Init(str_ptr);
        hs
    }

    fn Init(&mut self, str_ptr: *const i8) {
        if str_ptr.is_null() {
            self.mId = 0;
            return;
        }
        let hash = HashFunction(str_ptr);
        let mut id = HashHelper().FindFirst(hash);
        while id != 0 {
            debug_assert!(id > 0 && id < ThePool().mNextStringId);
            unsafe {
                if libc::strcmp(str_ptr, gCharPtrs[id as usize]) == 0 {
                    self.mId = id;
                    return;
                }
            }
            id = HashHelper().FindNext();
        }
        let str_len = unsafe { libc::strlen(str_ptr) } as c_int;
        let (raw, new_id) = ThePool().Alloc(str_len);
        unsafe {
            libc::strcpy(raw, str_ptr);
        }
        self.mId = new_id;
        HashHelper().Add(hash, self.mId);
        #[cfg(debug_assertions)]
        {
            let (debug_raw, test) = TheDebugPool().Alloc(str_len);
            debug_assert!(test == self.mId);
            unsafe {
                libc::strcpy(debug_raw, str_ptr);
            }
        }
    }

    pub fn c_str(&self) -> *const i8 {
        if self.mId == 0 {
            return b"\0".as_ptr() as *const i8;
        }
        debug_assert!(self.mId > 0 && self.mId < ThePool().mNextStringId);
        unsafe { gCharPtrs[self.mId as usize] }
    }

    pub fn str(&self) -> String {
        if self.mId == 0 {
            return String::new();
        }
        debug_assert!(self.mId > 0 && self.mId < ThePool().mNextStringId);
        unsafe {
            let c_str = gCharPtrs[self.mId as usize];
            CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned()
        }
    }
}
