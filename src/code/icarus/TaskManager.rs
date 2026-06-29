// Task Manager
//
//	-- jweier

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_assignments,
    unused_imports,
    clippy::all
)]

// this include must remain at the top of every Icarus CPP file
use crate::code::icarus::stdafx_h::*;
use crate::code::icarus::IcarusImplementation_h::*;
use crate::code::icarus::BlockStream_h::*;
use crate::code::icarus::Sequence_h::*;
use crate::code::icarus::TaskManager_h::*;
use crate::code::icarus::Sequencer_h::*;

use core::ffi::{c_int, c_char, c_void};

extern "C" {
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

macro_rules! ICARUS_VALIDATE {
    ($a:expr) => {
        if ($a) == 0 {
            return TASK_FAILED;
        }
    };
}

// STL_ITERATE( a, b )		for ( a = b.begin(); a != b.end(); a++ )
// Translated inline at each use site.

// STL_INSERT( a, b )		a.insert( a.end(), b )
macro_rules! STL_INSERT {
    ($a:expr, $b:expr) => {
        $a.push_back($b)
    };
}

/*
=================================================

CTask

=================================================
*/

impl CTask {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn Create(GUID: c_int, block: *mut CBlock) -> *mut CTask {
        let task: *mut CTask = Box::into_raw(Box::new(CTask::new()));

        //TODO: Emit warning
        if task.is_null() {
            return core::ptr::null_mut();
        }

        unsafe {
            (*task).SetTimeStamp(0);
            (*task).SetBlock(block);
            (*task).SetGUID(GUID);
        }

        task
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub unsafe fn Free(self_: *mut CTask) {
        //NOTENOTE: The block is not consumed by the task, it is the sequencer's job to clean blocks up
        drop(Box::from_raw(self_));
    }
}

/*
=================================================

CTaskGroup

=================================================
*/

impl CTaskGroup {
    pub fn new() -> Self {
        let mut this: CTaskGroup = unsafe { core::mem::zeroed() };
        this.Init();
        this.m_GUID = 0;
        this.m_parent = core::ptr::null_mut();
        this
    }

    // ~CTaskGroup: m_completedTasks.clear() is handled by Rust's Drop automatically.

    /*
    -------------------------
    SetGUID
    -------------------------
    */

    pub fn SetGUID(&mut self, GUID: c_int) {
        self.m_GUID = GUID;
    }

    /*
    -------------------------
    Init
    -------------------------
    */

    pub fn Init(&mut self) {
        self.m_completedTasks.clear();

        self.m_numCompleted = 0;
        self.m_parent = core::ptr::null_mut();
    }

    /*
    -------------------------
    Add
    -------------------------
    */

    pub fn Add(&mut self, task: *mut CTask) -> c_int {
        unsafe { self.m_completedTasks.insert((*task).GetGUID(), false); }
        TASK_OK
    }

    /*
    -------------------------
    MarkTaskComplete
    -------------------------
    */

    pub fn MarkTaskComplete(&mut self, id: c_int) -> bool {
        if self.m_completedTasks.contains_key(&id) {
            self.m_completedTasks.insert(id, true);
            self.m_numCompleted += 1;

            return true;
        }

        false
    }
}

/*
=================================================

CTaskManager

=================================================
*/

impl CTaskManager {
    pub fn new() -> Self {
        // static int uniqueID = 0;
        static UNIQUE_ID: core::sync::atomic::AtomicI32 =
            core::sync::atomic::AtomicI32::new(0);
        let mut this: CTaskManager = unsafe { core::mem::zeroed() };
        this.m_id = UNIQUE_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        this
    }

    /*
    -------------------------
    Create
    -------------------------
    */

    pub fn Create() -> *mut CTaskManager {
        Box::into_raw(Box::new(CTaskManager::new()))
    }

    /*
    -------------------------
    Init
    -------------------------
    */

    pub fn Init(&mut self, owner: *mut CSequencer) -> c_int {
        //TODO: Emit warning
        if owner.is_null() {
            return TASK_FAILED;
        }

        self.m_tasks.clear();
        self.m_owner = owner;
        unsafe { self.m_ownerID = (*owner).GetOwnerID(); }
        self.m_curGroup = core::ptr::null_mut();
        self.m_GUID = 0;
        self.m_resident = false;

        TASK_OK
    }

    /*
    -------------------------
    Free
    -------------------------
    */
    pub fn Free(&mut self) -> c_int {
        assert!(!self.m_resident); //don't free me, i'm currently running!
        //Clear out all pending tasks
        // Porting note: m_tasks is iterated; each element is *mut CTask freed via CTask::Free.
        for ti in self.m_tasks.iter() {
            unsafe { CTask::Free(*ti); }
        }

        self.m_tasks.clear();

        //Clear out all taskGroups
        for gi in self.m_taskGroups.iter() {
            unsafe { drop(Box::from_raw(*gi)); }
        }

        self.m_taskGroups.clear();
        self.m_taskGroupNameMap.clear();
        self.m_taskGroupIDMap.clear();

        TASK_OK
    }

    /*
    -------------------------
    Flush
    -------------------------
    */

    pub fn Flush(&mut self) -> c_int {
        //FIXME: Rewrite

        1 // true
    }

    /*
    -------------------------
    AddTaskGroup
    -------------------------
    */

    pub fn AddTaskGroup(&mut self, name: *const c_char, icarus: *mut CIcarus) -> *mut CTaskGroup {
        let group: *mut CTaskGroup;

        //Collect any garbage
        let name_str = unsafe { core::ffi::CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();

        if let Some(&existing) = self.m_taskGroupNameMap.get(&name_str) {
            group = existing;

            //Clear it and just move on
            unsafe { (*group).Init(); }

            return group;
        }

        //Allocate a new one
        let group: *mut CTaskGroup = Box::into_raw(Box::new(CTaskGroup::new()));

        //TODO: Emit warning
        assert!(!group.is_null());
        if group.is_null() {
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_ERROR,
                    b"Unable to allocate task group \"%s\"\n\0".as_ptr() as *const c_char,
                    name,
                );
            }
            return core::ptr::null_mut();
        }

        //Setup the internal information
        unsafe { (*group).SetGUID(self.m_GUID); }
        self.m_GUID += 1;

        //Add it to the list and associate it for retrieval later
        self.m_taskGroups.push_back(group);
        self.m_taskGroupNameMap.insert(name_str, group);
        unsafe { self.m_taskGroupIDMap.insert((*group).GetGUID(), group); }

        group
    }

    /*
    -------------------------
    GetTaskGroup
    -------------------------
    */

    // Porting note: C++ has two overloads of GetTaskGroup (by name and by id). Rust does not
    // support overloading; the int-id overload is named GetTaskGroupByID here. All internal
    // call sites are updated accordingly.

    pub fn GetTaskGroup(&mut self, name: *const c_char, icarus: *mut CIcarus) -> *mut CTaskGroup {
        let name_str = unsafe { core::ffi::CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();

        if let Some(&group) = self.m_taskGroupNameMap.get(&name_str) {
            return group;
        }

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_WARNING,
                b"Could not find task group \"%s\"\n\0".as_ptr() as *const c_char,
                name,
            );
        }

        core::ptr::null_mut()
    }

    pub fn GetTaskGroupByID(&mut self, id: c_int, icarus: *mut CIcarus) -> *mut CTaskGroup {
        if let Some(&group) = self.m_taskGroupIDMap.get(&id) {
            return group;
        }

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_WARNING,
                b"Could not find task group \"%d\"\n\0".as_ptr() as *const c_char,
                id,
            );
        }

        core::ptr::null_mut()
    }

    /*
    -------------------------
    Update
    -------------------------
    */

    pub fn Update(&mut self, icarus: *mut CIcarus) -> c_int {
        if unsafe { (*(*icarus).GetGame()).IsFrozen(self.m_ownerID) } {
            return TASK_FAILED;
        }
        self.m_count = 0; //Needed for runaway init
        self.m_resident = true;

        let returnVal = self.Go(icarus);

        self.m_resident = false;

        returnVal
    }

    /*
    -------------------------
    Check
    -------------------------
    */

    #[inline]
    pub fn Check(&self, targetID: c_int, block: *mut CBlock, memberNum: c_int) -> bool {
        if unsafe { (*(*block).GetMember(memberNum)).GetID() == targetID } {
            return true;
        }

        false
    }

    /*
    -------------------------
    GetFloat
    -------------------------
    */

    pub fn GetFloat(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: &mut c_int,
        value: &mut f32,
        icarus: *mut CIcarus,
    ) -> c_int {
        let name: *mut c_char;
        let type_: c_int;

        //See if this is a get() command replacement
        if self.Check(CIcarus::ID_GET, block, *memberNum) {
            //Update the member past the header id
            *memberNum += 1;

            //get( TYPE, NAME )
            let type_: c_int = unsafe {
                *((*block).GetMemberData(*memberNum) as *const f32) as c_int
            };
            *memberNum += 1;
            let name: *mut c_char = unsafe { (*block).GetMemberData(*memberNum) as *mut c_char };
            *memberNum += 1;

            //TODO: Emit warning
            if type_ != CIcarus::TK_FLOAT {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Get() call tried to return a non-FLOAT parameter!\n\0".as_ptr()
                            as *const c_char,
                    );
                }
                return 0; // false
            }

            return unsafe { (*(*icarus).GetGame()).GetFloat(entID, name, value as *mut f32) };
        }

        //Look for a random() inline call
        if self.Check(CIcarus::ID_RANDOM, block, *memberNum) {
            let min: f32;
            let max: f32;

            *memberNum += 1;

            let min: f32 = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;
            let max: f32 = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;

            *value = unsafe { (*(*icarus).GetGame()).Random(min, max) };

            return 1; // true
        }

        //Look for a tag() inline call
        if self.Check(CIcarus::ID_TAG, block, *memberNum) {
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_WARNING,
                    b"Invalid use of \"tag\" inline.  Not a valid replacement for type FLOAT\n\0"
                        .as_ptr() as *const c_char,
                );
            }
            return 0; // false
        }

        let bm: *mut CBlockMember = unsafe { (*block).GetMember(*memberNum) };

        if unsafe { (*bm).GetID() } == CIcarus::TK_INT {
            *value = unsafe { *((*block).GetMemberData(*memberNum) as *const c_int) as f32 };
            *memberNum += 1;
        } else if unsafe { (*bm).GetID() } == CIcarus::TK_FLOAT {
            *value = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;
        } else {
            assert!(false);
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_WARNING,
                    b"Unexpected value; expected type FLOAT\n\0".as_ptr() as *const c_char,
                );
            }
            return 0; // false
        }

        1 // true
    }

    /*
    -------------------------
    GetVector
    -------------------------
    */

    pub fn GetVector(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: &mut c_int,
        value: &mut vec3_t,
        icarus: *mut CIcarus,
    ) -> c_int {
        let name: *mut c_char;
        let type_: c_int;
        let i: c_int;

        //See if this is a get() command replacement
        if self.Check(CIcarus::ID_GET, block, *memberNum) {
            //Update the member past the header id
            *memberNum += 1;

            //get( TYPE, NAME )
            let type_: c_int = unsafe {
                *((*block).GetMemberData(*memberNum) as *const f32) as c_int
            };
            *memberNum += 1;
            let name: *mut c_char = unsafe { (*block).GetMemberData(*memberNum) as *mut c_char };
            *memberNum += 1;

            //TODO: Emit warning
            if type_ != CIcarus::TK_VECTOR {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Get() call tried to return a non-VECTOR parameter!\n\0".as_ptr()
                            as *const c_char,
                    );
                }
            }

            return unsafe { (*(*icarus).GetGame()).GetVector(entID, name, value as *mut vec3_t) };
        }

        //Look for a random() inline call
        if self.Check(CIcarus::ID_RANDOM, block, *memberNum) {
            let min: f32;
            let max: f32;

            *memberNum += 1;

            let min: f32 = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;
            let max: f32 = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;

            for i in 0..3_c_int {
                value[i as usize] = unsafe {
                    (*(*icarus).GetGame()).Random(min, max) as f32 //FIXME: Just truncating it for now.. should be fine though
                };
            }

            return 1; // true
        }

        //Look for a tag() inline call
        if self.Check(CIcarus::ID_TAG, block, *memberNum) {
            let tagName: *mut c_char;
            let tagLookup: f32;

            *memberNum += 1;
            let mut tagName: *mut c_char = core::ptr::null_mut();
            let mut tagLookup: f32 = 0.0;
            ICARUS_VALIDATE!(self.Get(entID, block, memberNum, &mut tagName, icarus));
            ICARUS_VALIDATE!(self.GetFloat(entID, block, memberNum, &mut tagLookup, icarus));

            if unsafe {
                (*(*icarus).GetGame()).GetTag(
                    entID,
                    tagName,
                    tagLookup as c_int,
                    value as *mut vec3_t,
                ) == 0 // false
            } {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Unable to find tag \"%s\"!\n\0".as_ptr() as *const c_char,
                        tagName,
                    );
                }
                assert!(false, "Unable to find tag");
                return TASK_FAILED;
            }

            return 1; // true
        }

        //Check for a real vector here
        let type_: c_int = unsafe {
            *((*block).GetMemberData(*memberNum) as *const f32) as c_int
        };

        if type_ != CIcarus::TK_VECTOR {
            //		icarus->GetGame()->DPrintf( WL_WARNING, "Unexpected value; expected type VECTOR\n" );
            return 0; // false
        }

        *memberNum += 1;

        for i in 0..3_c_int {
            if self.GetFloat(entID, block, memberNum, &mut value[i as usize], icarus) == 0 {
                return 0; // false
            }
        }

        1 // true
    }

    /*
    -------------------------
    Get
    -------------------------
    */

    pub fn GetID(&mut self) -> c_int {
        self.m_id
    }

    pub fn Get(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: &mut c_int,
        value: *mut *mut c_char,
        icarus: *mut CIcarus,
    ) -> c_int {
        static mut tempBuffer: [c_char; 128] = [0; 128]; //FIXME: EEEK!
        let mut vector: vec3_t = [0.0; 3];
        let name: *mut c_char;
        let tagName: *mut c_char;
        let tagLookup: f32;
        let type_: c_int;

        //Look for a get() inline call
        if self.Check(CIcarus::ID_GET, block, *memberNum) {
            //Update the member past the header id
            *memberNum += 1;

            //get( TYPE, NAME )
            let type_: c_int = unsafe {
                *((*block).GetMemberData(*memberNum) as *const f32) as c_int
            };
            *memberNum += 1;
            let name: *mut c_char = unsafe { (*block).GetMemberData(*memberNum) as *mut c_char };
            *memberNum += 1;

            //Format the return properly
            //FIXME: This is probably doing double formatting in certain cases...
            //FIXME: STRING MANAGEMENT NEEDS TO BE IMPLEMENTED, MY CURRENT SOLUTION IS NOT ACCEPTABLE!!
            match type_ {
                t if t == CIcarus::TK_STRING => {
                    if unsafe { (*(*icarus).GetGame()).GetString(entID, name, value) } == 0 {
                        unsafe {
                            (*(*icarus).GetGame()).DebugPrint(
                                IGameInterface::WL_ERROR,
                                b"Get() parameter \"%s\" could not be found!\n\0".as_ptr()
                                    as *const c_char,
                                name,
                            );
                        }
                        return 0; // false
                    }

                    return 1; // true
                    // break (implicit in Rust match)
                }

                t if t == CIcarus::TK_FLOAT => {
                    let mut temp: f32 = 0.0;

                    if unsafe { (*(*icarus).GetGame()).GetFloat(entID, name, &mut temp) } == 0 {
                        unsafe {
                            (*(*icarus).GetGame()).DebugPrint(
                                IGameInterface::WL_ERROR,
                                b"Get() parameter \"%s\" could not be found!\n\0".as_ptr()
                                    as *const c_char,
                                name,
                            );
                        }
                        return 0; // false
                    }

                    unsafe {
                        sprintf(
                            core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                            b"%f\0".as_ptr() as *const c_char,
                            temp,
                        );
                        *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
                    }

                    return 1; // true
                    // break
                }

                t if t == CIcarus::TK_VECTOR => {
                    let mut vval: vec3_t = [0.0; 3];

                    if unsafe {
                        (*(*icarus).GetGame()).GetVector(entID, name, &mut vval as *mut vec3_t)
                    } == 0
                    {
                        unsafe {
                            (*(*icarus).GetGame()).DebugPrint(
                                IGameInterface::WL_ERROR,
                                b"Get() parameter \"%s\" could not be found!\n\0".as_ptr()
                                    as *const c_char,
                                name,
                            );
                        }
                        return 0; // false
                    }

                    unsafe {
                        sprintf(
                            core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                            b"%f %f %f\0".as_ptr() as *const c_char,
                            vval[0],
                            vval[1],
                            vval[2],
                        );
                        *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
                    }

                    return 1; // true
                    // break
                }

                _ => {
                    unsafe {
                        (*(*icarus).GetGame()).DebugPrint(
                            IGameInterface::WL_ERROR,
                            b"Get() call tried to return an unknown type!\n\0".as_ptr()
                                as *const c_char,
                        );
                    }
                    return 0; // false
                    // break
                }
            }
        }

        //Look for a random() inline call
        if self.Check(CIcarus::ID_RANDOM, block, *memberNum) {
            let mut min: f32 = 0.0;
            let mut max: f32 = 0.0;
            let ret: f32;

            *memberNum += 1;

            min = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;
            max = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;

            let ret: f32 = unsafe { (*(*icarus).GetGame()).Random(min, max) };

            unsafe {
                sprintf(
                    core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                    b"%f\0".as_ptr() as *const c_char,
                    ret,
                );
                *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
            }

            return 1; // true
        }

        //Look for a tag() inline call
        if self.Check(CIcarus::ID_TAG, block, *memberNum) {
            *memberNum += 1;
            let mut tagName: *mut c_char = core::ptr::null_mut();
            let mut tagLookup: f32 = 0.0;
            ICARUS_VALIDATE!(self.Get(entID, block, memberNum, &mut tagName, icarus));
            ICARUS_VALIDATE!(self.GetFloat(entID, block, memberNum, &mut tagLookup, icarus));

            if unsafe {
                (*(*icarus).GetGame()).GetTag(
                    entID,
                    tagName,
                    tagLookup as c_int,
                    &mut vector as *mut vec3_t,
                )
            } == 0
            {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Unable to find tag \"%s\"!\n\0".as_ptr() as *const c_char,
                        tagName,
                    );
                }
                assert!(false, "Unable to find tag");
                return 0; // false
            }

            unsafe {
                sprintf(
                    core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                    b"%f %f %f\0".as_ptr() as *const c_char,
                    vector[0],
                    vector[1],
                    vector[2],
                );
                *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
            }

            return 1; // true
        }

        //Get an actual piece of data

        let bm: *mut CBlockMember = unsafe { (*block).GetMember(*memberNum) };

        if unsafe { (*bm).GetID() } == CIcarus::TK_INT {
            let fval: f32 =
                unsafe { *((*block).GetMemberData(*memberNum) as *const c_int) as f32 };
            *memberNum += 1;
            unsafe {
                sprintf(
                    core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                    b"%f\0".as_ptr() as *const c_char,
                    fval,
                );
                *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
            }

            return 1; // true
        } else if unsafe { (*bm).GetID() } == CIcarus::TK_FLOAT {
            let fval: f32 = unsafe { *((*block).GetMemberData(*memberNum) as *const f32) };
            *memberNum += 1;
            unsafe {
                sprintf(
                    core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                    b"%f\0".as_ptr() as *const c_char,
                    fval,
                );
                *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
            }

            return 1; // true
        } else if unsafe { (*bm).GetID() } == CIcarus::TK_VECTOR {
            let mut vval: vec3_t = [0.0; 3];

            *memberNum += 1;

            for i in 0..3_c_int {
                if self.GetFloat(entID, block, memberNum, &mut vval[i as usize], icarus) == 0 {
                    return 0; // false
                }

                unsafe {
                    sprintf(
                        core::ptr::addr_of_mut!(tempBuffer) as *mut c_char,
                        b"%f %f %f\0".as_ptr() as *const c_char,
                        vval[0],
                        vval[1],
                        vval[2],
                    );
                    *value = core::ptr::addr_of_mut!(tempBuffer) as *mut c_char;
                }
            }

            return 1; // true
        } else if unsafe { (*bm).GetID() } == CIcarus::TK_STRING
            || unsafe { (*bm).GetID() } == CIcarus::TK_IDENTIFIER
        {
            unsafe {
                *value = (*block).GetMemberData(*memberNum) as *mut c_char;
            }
            *memberNum += 1;

            return 1; // true
        }

        //TODO: Emit warning
        assert!(false);
        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_WARNING,
                b"Unexpected value; expected type STRING\n\0".as_ptr() as *const c_char,
            );
        }

        0 // false
    }

    /*
    -------------------------
    Go
    -------------------------
    */

    pub fn Go(&mut self, icarus: *mut CIcarus) -> c_int {
        let mut task: *mut CTask = core::ptr::null_mut();
        let mut completed: bool = false;

        //Check for run away scripts
        self.m_count += 1;
        if self.m_count > RUNAWAY_LIMIT {
            assert!(false);
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_ERROR,
                    b"Runaway loop detected!\n\0".as_ptr() as *const c_char,
                );
            }
            return TASK_FAILED;
        }

        //If there are tasks to complete, do so
        if !self.m_tasks.is_empty() {
            //Get the next task
            task = self.PopTask(CSequence::POP_BACK);

            assert!(!task.is_null());
            if task.is_null() {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Invalid task found in Go()!\n\0".as_ptr() as *const c_char,
                    );
                }
                return TASK_FAILED;
            }

            //If this hasn't been stamped, do so
            if unsafe { (*task).GetTimeStamp() } == 0 {
                unsafe { (*task).SetTimeStamp((*(*icarus).GetGame()).GetTime()); }
            }

            //Switch and call the proper function
            match unsafe { (*task).GetID() } {
                id if id == CIcarus::ID_WAIT => {
                    self.Wait(task, &mut completed, icarus);

                    //Push it to consider it again on the next frame if not complete
                    if !completed {
                        self.PushTask(task, CSequence::PUSH_BACK);
                        return TASK_OK;
                    }

                    self.Completed(unsafe { (*task).GetGUID() });
                }

                id if id == CIcarus::ID_WAITSIGNAL => {
                    self.WaitSignal(task, &mut completed, icarus);

                    //Push it to consider it again on the next frame if not complete
                    if !completed {
                        self.PushTask(task, CSequence::PUSH_BACK);
                        return TASK_OK;
                    }

                    self.Completed(unsafe { (*task).GetGUID() });
                }

                id if id == CIcarus::ID_PRINT => { //print( STRING )
                    self.Print(task, icarus);
                }

                id if id == CIcarus::ID_SOUND => { //sound( name )
                    self.Sound(task, icarus);
                }

                id if id == CIcarus::ID_MOVE => { //move ( ORIGIN, ANGLES, DURATION )
                    self.Move(task, icarus);
                }

                id if id == CIcarus::ID_ROTATE => { //rotate( ANGLES, DURATION )
                    self.Rotate(task, icarus);
                }

                id if id == CIcarus::ID_KILL => { //kill( NAME )
                    self.Kill(task, icarus);
                }

                id if id == CIcarus::ID_REMOVE => { //remove( NAME )
                    self.Remove(task, icarus);
                }

                id if id == CIcarus::ID_CAMERA => { //camera( ? )
                    self.Camera(task, icarus);
                }

                id if id == CIcarus::ID_SET => { //set( NAME, ? )
                    self.Set(task, icarus);
                }

                id if id == CIcarus::ID_USE => { //use( NAME )
                    self.Use(task, icarus);
                }

                id if id == CIcarus::ID_DECLARE => { //declare( TYPE, NAME )
                    self.DeclareVariable(task, icarus);
                }

                id if id == CIcarus::ID_FREE => { //free( NAME )
                    self.FreeVariable(task, icarus);
                }

                id if id == CIcarus::ID_SIGNAL => { //signal( NAME )
                    self.Signal(task, icarus);
                }

                id if id == CIcarus::ID_PLAY => { //play ( NAME )
                    self.Play(task, icarus);
                }

                _ => {
                    assert!(false);
                    unsafe { CTask::Free(task); }
                    unsafe {
                        (*(*icarus).GetGame()).DebugPrint(
                            IGameInterface::WL_ERROR,
                            b"Found unknown task type!\n\0".as_ptr() as *const c_char,
                        );
                    }
                    return TASK_FAILED;
                    // break
                }
            }

            //Pump the sequencer for another task
            self.CallbackCommand(task, TASK_RETURN_COMPLETE, icarus);

            unsafe { CTask::Free(task); }
        }

        //FIXME: A command surge limiter could be implemented at this point to be sure a script doesn't
        //		 execute too many commands in one cycle.  This may, however, cause timing errors to surface.
        TASK_OK
    }

    /*
    -------------------------
    SetCommand
    -------------------------
    */

    pub fn SetCommand(&mut self, command: *mut CBlock, type_: c_int, icarus: *mut CIcarus) -> c_int {
        let guid = self.m_GUID;
        self.m_GUID += 1;
        let task: *mut CTask = CTask::Create(guid, command);

        //If this is part of a task group, add it in
        if !self.m_curGroup.is_null() {
            unsafe { (*self.m_curGroup).Add(task); }
        }

        //TODO: Emit warning
        assert!(!task.is_null());
        if task.is_null() {
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_ERROR,
                    b"Unable to allocate new task!\n\0".as_ptr() as *const c_char,
                );
            }
            return TASK_FAILED;
        }

        self.PushTask(task, type_);

        TASK_OK
    }

    /*
    -------------------------
    MarkTask
    -------------------------
    */

    pub fn MarkTask(&mut self, id: c_int, operation: c_int, icarus: *mut CIcarus) -> c_int {
        let group: *mut CTaskGroup = self.GetTaskGroupByID(id, icarus);

        assert!(!group.is_null());

        if group.is_null() {
            return TASK_FAILED;
        }

        if operation == TASK_START {
            //Reset all the completion information
            unsafe { (*group).Init(); }

            unsafe { (*group).SetParent(self.m_curGroup); }
            self.m_curGroup = group;
        } else if operation == TASK_END {
            assert!(!self.m_curGroup.is_null());
            if self.m_curGroup.is_null() {
                return TASK_FAILED;
            }

            self.m_curGroup = unsafe { (*self.m_curGroup).GetParent() };
        }

        #[cfg(debug_assertions)]
        {
            if operation != TASK_START && operation != TASK_END {
                assert!(false);
            }
        }

        TASK_OK
    }

    /*
    -------------------------
    Completed
    -------------------------
    */

    pub fn Completed(&mut self, id: c_int) -> c_int {
        //Mark the task as completed
        for tgi in self.m_taskGroups.iter() {
            //If this returns true, then the task was marked properly
            if unsafe { (**tgi).MarkTaskComplete(id) } {
                break;
            }
        }

        TASK_OK
    }

    /*
    -------------------------
    CallbackCommand
    -------------------------
    */

    pub fn CallbackCommand(
        &mut self,
        task: *mut CTask,
        returnCode: c_int,
        icarus: *mut CIcarus,
    ) -> c_int {
        // Porting note: original C++ uses comma operator bug:
        //   if ( m_owner->Callback(...) == CSequencer::SEQ_OK, icarus )
        // The comma operator discards the SEQ_OK comparison; the condition is `icarus`
        // (truthy if non-null). Preserved faithfully.
        let _cmp: bool = unsafe {
            (*self.m_owner).Callback(
                self as *mut CTaskManager,
                (*task).GetBlock(),
                returnCode,
                icarus,
            )
        } == CSequencer::SEQ_OK;
        if !icarus.is_null() {
            return self.Go(icarus);
        }

        assert!(false);

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_ERROR,
                b"Command callback failure!\n\0".as_ptr() as *const c_char,
            );
        }
        TASK_FAILED
    }

    /*
    -------------------------
    RecallTask
    -------------------------
    */

    pub fn RecallTask(&mut self) -> *mut CBlock {
        let task: *mut CTask;

        let task: *mut CTask = self.PopTask(CSequence::POP_BACK);

        if !task.is_null() {
            // fixed 2/12/2 to free the task that has been popped (called from sequencer Recall)
            let retBlock: *mut CBlock = unsafe { (*task).GetBlock() };
            unsafe { CTask::Free(task); }

            return retBlock;
            //	return task->GetBlock();
        }

        core::ptr::null_mut()
    }

    /*
    -------------------------
    PushTask
    -------------------------
    */

    pub fn PushTask(&mut self, task: *mut CTask, flag: c_int) -> c_int {
        assert!(
            flag == CSequence::PUSH_FRONT || flag == CSequence::PUSH_BACK
        );

        match flag {
            f if f == CSequence::PUSH_FRONT => {
                self.m_tasks.push_front(task);

                return TASK_OK;
                // break
            }

            f if f == CSequence::PUSH_BACK => {
                self.m_tasks.push_back(task);

                return TASK_OK;
                // break
            }

            _ => {}
        }

        //Invalid flag
        CSequencer::SEQ_FAILED
    }

    /*
    -------------------------
    PopTask
    -------------------------
    */

    pub fn PopTask(&mut self, flag: c_int) -> *mut CTask {
        let task: *mut CTask;

        assert!(flag == CSequence::POP_FRONT || flag == CSequence::POP_BACK);

        if self.m_tasks.is_empty() {
            return core::ptr::null_mut();
        }

        match flag {
            f if f == CSequence::POP_FRONT => {
                let task: *mut CTask = self.m_tasks.pop_front().unwrap_or(core::ptr::null_mut());

                return task;
                // break
            }

            f if f == CSequence::POP_BACK => {
                let task: *mut CTask = self.m_tasks.pop_back().unwrap_or(core::ptr::null_mut());

                return task;
                // break
            }

            _ => {}
        }

        //Invalid flag
        core::ptr::null_mut()
    }

    /*
    -------------------------
    GetCurrentTask
    -------------------------
    */

    pub fn GetCurrentTask(&mut self) -> *mut CBlock {
        let task: *mut CTask = self.PopTask(CSequence::POP_BACK);

        if task.is_null() {
            return core::ptr::null_mut();
        }
        // fixed 2/12/2 to free the task that has been popped (called from sequencer Interrupt)
        let retBlock: *mut CBlock = unsafe { (*task).GetBlock() };
        unsafe { CTask::Free(task); }

        retBlock
        //	return task->GetBlock();
    }
}

/*
=================================================

  Task Functions

=================================================
*/

impl CTaskManager {
    pub fn Wait(
        &mut self,
        task: *mut CTask,
        completed: &mut bool,
        icarus: *mut CIcarus,
    ) -> c_int {
        let bm: *mut CBlockMember;
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let dwtime: f32;
        let mut memberNum: c_int = 0;

        *completed = false;

        let bm: *mut CBlockMember = unsafe { (*block).GetMember(0) };

        //Check if this is a task completion wait
        if unsafe { (*bm).GetID() } == CIcarus::TK_STRING {
            let mut sVal: *mut c_char = core::ptr::null_mut();
            ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

            if unsafe { (*task).GetTimeStamp() } == unsafe { (*(*icarus).GetGame()).GetTime() } {
                //Print out the debug info
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d wait(\"%s\"); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        sVal,
                        (*task).GetTimeStamp(),
                    );
                }
            }

            let group: *mut CTaskGroup = self.GetTaskGroup(sVal, icarus);

            if group.is_null() {
                //TODO: Emit warning
                *completed = false;
                return TASK_FAILED;
            }

            *completed = unsafe { (*group).Complete() };
        } else { //Otherwise it's a time completion wait
            // Porting note: C++ declares dwtime outside the inner if/else so that the
            // timestamp check and completion check (below) can share it.  Mirrored here.
            let mut dwtime: f32 = 0.0;

            if self.Check(CIcarus::ID_RANDOM, block, memberNum) {
                //get it random only the first time
                let mut min: f32 = 0.0;
                let mut max: f32 = 0.0;

                dwtime = unsafe { *((*block).GetMemberData(memberNum) as *const f32) };
                memberNum += 1;
                if dwtime == unsafe { (*(*icarus).GetGame()).MaxFloat() } {
                    //we have not evaluated this random yet
                    min = unsafe { *((*block).GetMemberData(memberNum) as *const f32) };
                    memberNum += 1;
                    max = unsafe { *((*block).GetMemberData(memberNum) as *const f32) };
                    memberNum += 1;

                    dwtime = unsafe { (*(*icarus).GetGame()).Random(min, max) };

                    //store the result in the first member
                    unsafe {
                        (*bm).SetData(
                            &dwtime as *const f32 as *mut c_void,
                            core::mem::size_of_val(&dwtime),
                            icarus,
                        );
                    }
                }
            } else {
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut dwtime,
                    icarus
                ));
            }

            if unsafe { (*task).GetTimeStamp() }
                == unsafe { (*(*icarus).GetGame()).GetTime() }
            {
                //Print out the debug info
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d wait( %d ); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        dwtime as c_int,
                        (*task).GetTimeStamp(),
                    );
                }
            }

            if (unsafe { (*task).GetTimeStamp() } + dwtime as c_int) // Porting note: GetTimeStamp()+dwtime cast preserves C++ int arithmetic
                < unsafe { (*(*icarus).GetGame()).GetTime() }
            {
                *completed = true;
                memberNum = 0;
                if self.Check(CIcarus::ID_RANDOM, block, memberNum) {
                    //set the data back to 0 so it will be re-randomized next time
                    let dwtime_reset: f32 = unsafe { (*(*icarus).GetGame()).MaxFloat() };
                    unsafe {
                        (*bm).SetData(
                            &dwtime_reset as *const f32 as *mut c_void,
                            core::mem::size_of_val(&dwtime_reset),
                            icarus,
                        );
                    }
                }
            }
        }

        TASK_OK
    }

    /*
    -------------------------
    WaitSignal
    -------------------------
    */

    pub fn WaitSignal(
        &mut self,
        task: *mut CTask,
        completed: &mut bool,
        icarus: *mut CIcarus,
    ) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        *completed = false;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        if unsafe { (*task).GetTimeStamp() } == unsafe { (*(*icarus).GetGame()).GetTime() } {
            //Print out the debug info
            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_DEBUG,
                    b"%4d waitsignal(\"%s\"); [%d]\0".as_ptr() as *const c_char,
                    self.m_ownerID,
                    sVal,
                    (*task).GetTimeStamp(),
                );
            }
        }

        if unsafe { (*icarus).CheckSignal(sVal) } {
            *completed = true;
            unsafe { (*icarus).ClearSignal(sVal); }
        }

        TASK_OK
    }

    /*
    -------------------------
    Print
    -------------------------
    */

    pub fn Print(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d print(\"%s\"); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
        }

        unsafe { (*(*icarus).GetGame()).CenterPrint(sVal); }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Sound
    -------------------------
    */

    pub fn Sound(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let sVal2: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        let mut sVal2: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d sound(\"%s\", \"%s\"); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                sVal2,
                (*task).GetTimeStamp(),
            );
        }

        //Only instantly complete if the user has requested it
        if unsafe {
            (*(*icarus).GetGame()).PlaySound((*task).GetGUID(), self.m_ownerID, sVal2, sVal)
        } != 0
        {
            self.Completed(unsafe { (*task).GetGUID() });
        }

        TASK_OK
    }

    /*
    -------------------------
    Rotate
    -------------------------
    */

    pub fn Rotate(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let mut vector: vec3_t = [0.0; 3];
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let tagName: *mut c_char;
        let tagLookup: f32;
        let duration: f32;
        let mut memberNum: c_int = 0;

        //Check for a tag reference
        if self.Check(CIcarus::ID_TAG, block, memberNum) {
            memberNum += 1;

            let mut tagName: *mut c_char = core::ptr::null_mut();
            let mut tagLookup: f32 = 0.0;
            ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut tagName, icarus));
            ICARUS_VALIDATE!(self.GetFloat(
                self.m_ownerID,
                block,
                &mut memberNum,
                &mut tagLookup,
                icarus
            ));

            if unsafe {
                (*(*icarus).GetGame()).GetTag(
                    self.m_ownerID,
                    tagName,
                    tagLookup as c_int,
                    &mut vector as *mut vec3_t,
                )
            } == 0
            {
                //TODO: Emit warning
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_ERROR,
                        b"Unable to find tag \"%s\"!\n\0".as_ptr() as *const c_char,
                        tagName,
                    );
                }
                assert!(false);
                return TASK_FAILED;
            }
        } else {
            //Get a normal vector
            ICARUS_VALIDATE!(self.GetVector(
                self.m_ownerID,
                block,
                &mut memberNum,
                &mut vector,
                icarus
            ));
        }

        //Find the duration
        let mut duration: f32 = 0.0;
        ICARUS_VALIDATE!(self.GetFloat(
            self.m_ownerID,
            block,
            &mut memberNum,
            &mut duration,
            icarus
        ));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d rotate( <%f,%f,%f>, %d); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                vector[0],
                vector[1],
                vector[2],
                duration as c_int,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Lerp2Angles(
                (*task).GetGUID(),
                self.m_ownerID,
                &mut vector as *mut vec3_t,
                duration,
            );
        }

        TASK_OK
    }

    /*
    -------------------------
    Remove
    -------------------------
    */

    pub fn Remove(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d remove(\"%s\"); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Remove(self.m_ownerID, sVal);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Camera
    -------------------------
    */

    pub fn Camera(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let mut vector: vec3_t = [0.0; 3];
        let mut vector2: vec3_t = [0.0; 3];
        let mut type_: f32 = 0.0;
        let mut fVal: f32 = 0.0;
        let mut fVal2: f32 = 0.0;
        let mut fVal3: f32 = 0.0;
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        //Get the camera function type
        ICARUS_VALIDATE!(self.GetFloat(
            self.m_ownerID,
            block,
            &mut memberNum,
            &mut type_,
            icarus
        ));

        match type_ as c_int {
            t if t == CIcarus::TYPE_PAN => {
                ICARUS_VALIDATE!(self.GetVector(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut vector,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetVector(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut vector2,
                    icarus
                ));

                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( PAN, <%f %f %f>, <%f %f %f>, %f); [%d]\0".as_ptr()
                            as *const c_char,
                        self.m_ownerID,
                        vector[0],
                        vector[1],
                        vector[2],
                        vector2[0],
                        vector2[1],
                        vector2[2],
                        fVal,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraPan(
                        &mut vector as *mut vec3_t,
                        &mut vector2 as *mut vec3_t,
                        fVal,
                    );
                }
            }

            t if t == CIcarus::TYPE_ZOOM => {
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( ZOOM, %f, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraZoom(fVal, fVal2);
                }
            }

            t if t == CIcarus::TYPE_MOVE => {
                ICARUS_VALIDATE!(self.GetVector(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut vector,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( MOVE, <%f %f %f>, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        vector[0],
                        vector[1],
                        vector[2],
                        fVal,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraMove(&mut vector as *mut vec3_t, fVal);
                }
            }

            t if t == CIcarus::TYPE_ROLL => {
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( ROLL, %f, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraRoll(fVal, fVal2);
                }
            }

            t if t == CIcarus::TYPE_FOLLOW => {
                let mut sVal: *mut c_char = core::ptr::null_mut();
                ICARUS_VALIDATE!(self.Get(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut sVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( FOLLOW, \"%s\", %f, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        sVal,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraFollow(sVal as *const c_char, fVal, fVal2);
                }
            }

            t if t == CIcarus::TYPE_TRACK => {
                let mut sVal: *mut c_char = core::ptr::null_mut();
                ICARUS_VALIDATE!(self.Get(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut sVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( TRACK, \"%s\", %f, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        sVal,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraTrack(sVal as *const c_char, fVal, fVal2);
                }
            }

            t if t == CIcarus::TYPE_DISTANCE => {
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( DISTANCE, %f, %f); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraDistance(fVal, fVal2);
                }
            }

            t if t == CIcarus::TYPE_FADE => {
                ICARUS_VALIDATE!(self.GetVector(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut vector,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));

                ICARUS_VALIDATE!(self.GetVector(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut vector2,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal3,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( FADE, <%f %f %f>, %f, <%f %f %f>, %f, %f); [%d]\0"
                            .as_ptr() as *const c_char,
                        self.m_ownerID,
                        vector[0],
                        vector[1],
                        vector[2],
                        fVal,
                        vector2[0],
                        vector2[1],
                        vector2[2],
                        fVal2,
                        fVal3,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraFade(
                        vector[0], vector[1], vector[2], fVal,
                        vector2[0], vector2[1], vector2[2], fVal2,
                        fVal3,
                    );
                }
            }

            t if t == CIcarus::TYPE_PATH => {
                let mut sVal: *mut c_char = core::ptr::null_mut();
                ICARUS_VALIDATE!(self.Get(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut sVal,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( PATH, \"%s\"); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        sVal,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraPath(sVal);
                }
            }

            t if t == CIcarus::TYPE_ENABLE => {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( ENABLE ); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraEnable();
                }
            }

            t if t == CIcarus::TYPE_DISABLE => {
                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( DISABLE ); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraDisable();
                }
            }

            t if t == CIcarus::TYPE_SHAKE => {
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal,
                    icarus
                ));
                ICARUS_VALIDATE!(self.GetFloat(
                    self.m_ownerID,
                    block,
                    &mut memberNum,
                    &mut fVal2,
                    icarus
                ));

                unsafe {
                    (*(*icarus).GetGame()).DebugPrint(
                        IGameInterface::WL_DEBUG,
                        b"%4d camera( SHAKE, %f, %f ); [%d]\0".as_ptr() as *const c_char,
                        self.m_ownerID,
                        fVal,
                        fVal2,
                        (*task).GetTimeStamp(),
                    );
                    (*(*icarus).GetGame()).CameraShake(fVal, fVal2 as c_int);
                }
            }

            _ => {}
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Move
    -------------------------
    */

    pub fn Move(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let mut vector: vec3_t = [0.0; 3];
        let mut vector2: vec3_t = [0.0; 3];
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let mut duration: f32 = 0.0;
        let mut memberNum: c_int = 0;

        //Get the goal position
        ICARUS_VALIDATE!(self.GetVector(
            self.m_ownerID,
            block,
            &mut memberNum,
            &mut vector,
            icarus
        ));

        //Check for possible angles field
        if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector2, icarus) == 0 {
            ICARUS_VALIDATE!(self.GetFloat(
                self.m_ownerID,
                block,
                &mut memberNum,
                &mut duration,
                icarus
            ));

            unsafe {
                (*(*icarus).GetGame()).DebugPrint(
                    IGameInterface::WL_DEBUG,
                    b"%4d move( <%f %f %f>, %f ); [%d]\0".as_ptr() as *const c_char,
                    self.m_ownerID,
                    vector[0],
                    vector[1],
                    vector[2],
                    duration,
                    (*task).GetTimeStamp(),
                );
                (*(*icarus).GetGame()).Lerp2Pos(
                    (*task).GetGUID(),
                    self.m_ownerID,
                    &mut vector as *mut vec3_t,
                    core::ptr::null_mut(),
                    duration,
                );
            }

            return TASK_OK;
        }

        //Get the duration and make the call
        ICARUS_VALIDATE!(self.GetFloat(
            self.m_ownerID,
            block,
            &mut memberNum,
            &mut duration,
            icarus
        ));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d move( <%f %f %f>, <%f %f %f>, %f ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                vector[0],
                vector[1],
                vector[2],
                vector2[0],
                vector2[1],
                vector2[2],
                duration,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Lerp2Pos(
                (*task).GetGUID(),
                self.m_ownerID,
                &mut vector as *mut vec3_t,
                &mut vector2 as *mut vec3_t,
                duration,
            );
        }

        TASK_OK
    }

    /*
    -------------------------
    Kill
    -------------------------
    */

    pub fn Kill(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d kill( \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Kill(self.m_ownerID, sVal);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Set
    -------------------------
    */

    pub fn Set(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let sVal2: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        let mut sVal2: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d set( \"%s\", \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                sVal2,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Set((*task).GetGUID(), self.m_ownerID, sVal, sVal2);
        }

        TASK_OK
    }

    /*
    -------------------------
    Use
    -------------------------
    */

    pub fn Use(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d use( \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Use(self.m_ownerID, sVal);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    DeclareVariable
    -------------------------
    */

    pub fn DeclareVariable(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;
        let mut fVal: f32 = 0.0;

        ICARUS_VALIDATE!(self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal, icarus));
        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d declare( %d, \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                fVal as c_int,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).DeclareVariable(fVal as c_int, sVal);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    FreeVariable
    -------------------------
    */

    pub fn FreeVariable(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d free( \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).FreeVariable(sVal);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Signal
    -------------------------
    */

    pub fn Signal(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d signal( \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
            (*icarus).Signal(sVal as *const c_char);
        }

        self.Completed(unsafe { (*task).GetGUID() });

        TASK_OK
    }

    /*
    -------------------------
    Play
    -------------------------
    */

    pub fn Play(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        let block: *mut CBlock = unsafe { (*task).GetBlock() };
        let sVal: *mut c_char;
        let sVal2: *mut c_char;
        let mut memberNum: c_int = 0;

        let mut sVal: *mut c_char = core::ptr::null_mut();
        let mut sVal2: *mut c_char = core::ptr::null_mut();
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal, icarus));
        ICARUS_VALIDATE!(self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2, icarus));

        unsafe {
            (*(*icarus).GetGame()).DebugPrint(
                IGameInterface::WL_DEBUG,
                b"%4d play( \"%s\", \"%s\" ); [%d]\0".as_ptr() as *const c_char,
                self.m_ownerID,
                sVal,
                sVal2,
                (*task).GetTimeStamp(),
            );
            (*(*icarus).GetGame()).Play(
                (*task).GetGUID(),
                self.m_ownerID,
                sVal as *const c_char,
                sVal2 as *const c_char,
            );
        }

        TASK_OK
    }

    /*
    -------------------------
    SaveCommand
    -------------------------
    */

    //FIXME: ARGH!  This is duplicated from CSequence because I can't directly link it any other way...

    pub fn SaveCommand(&mut self, block: *mut CBlock) -> c_int {
        let pIcarus: *mut CIcarus =
            unsafe { IIcarusInterface::GetIcarus() as *mut CIcarus };

        let flags: u8;
        let numMembers: c_int;
        let bID: c_int;
        let size: c_int;
        let bm: *mut CBlockMember;

        //Save out the block ID
        let mut bID: c_int = unsafe { (*block).GetBlockID() };
        unsafe {
            (*pIcarus).BufferWrite(
                &mut bID as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Save out the block's flags
        let mut flags: u8 = unsafe { (*block).GetFlags() };
        unsafe {
            (*pIcarus).BufferWrite(
                &mut flags as *mut u8 as *mut c_void,
                core::mem::size_of::<u8>(),
            );
        }

        //Save out the number of members to read
        let mut numMembers: c_int = unsafe { (*block).GetNumMembers() };
        unsafe {
            (*pIcarus).BufferWrite(
                &mut numMembers as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        for i in 0..numMembers {
            let bm: *mut CBlockMember = unsafe { (*block).GetMember(i) };

            //Save the block id
            let mut bID: c_int = unsafe { (*bm).GetID() };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut bID as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Save out the data size
            let mut size: c_int = unsafe { (*bm).GetSize() };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut size as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Save out the raw data
            unsafe {
                (*pIcarus).BufferWrite((*bm).GetData(), size as usize);
            }
        }

        1 // true
    }

    /*
    -------------------------
    Save
    -------------------------
    */

    pub fn Save(&mut self) {
        let taskGroup: *mut CTaskGroup;
        let name: *const c_char;
        let block: *mut CBlock;
        let timeStamp: DWORD;
        let completed: bool;
        let id: c_int;
        let numCommands: c_int;
        let numWritten: c_int;

        // Data saved here.
        //	Taskmanager GUID.
        //	Number of Tasks.
        //	Tasks:
        //				- GUID.
        //				- Timestamp.
        //				- Block/Command.
        //	Number of task groups.
        //	Task groups ID's.
        //	Task groups (data).
        //				- Parent.
        //				- Number of Commands.
        //				- Commands:
        //						+ ID.
        //						+ State of Completion.
        //				- Number of Completed Commands.
        //	Currently active group.
        //	Task group names:
        //				- String Size.
        //				- String.
        //				- ID.

        let pIcarus: *mut CIcarus =
            unsafe { IIcarusInterface::GetIcarus() as *mut CIcarus };

        //Save the taskmanager's GUID
        unsafe {
            (*pIcarus).BufferWrite(
                &mut self.m_GUID as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Save out the number of tasks that will follow
        let mut iNumTasks: c_int = self.m_tasks.len() as c_int;
        unsafe {
            (*pIcarus).BufferWrite(
                &mut iNumTasks as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Save out all the tasks
        for ti in self.m_tasks.iter() {
            //Save the GUID
            let mut id: c_int = unsafe { (**ti).GetGUID() };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Save the timeStamp (FIXME: Although, this is going to be worthless if time is not consistent...)
            let mut timeStamp: DWORD = unsafe { (**ti).GetTimeStamp() };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut timeStamp as *mut DWORD as *mut c_void,
                    core::mem::size_of::<DWORD>(),
                );
            }

            //Save out the block
            let block: *mut CBlock = unsafe { (**ti).GetBlock() };
            self.SaveCommand(block);
        }

        //Save out the number of task groups
        let mut numTaskGroups: c_int = self.m_taskGroups.len() as c_int;
        unsafe {
            (*pIcarus).BufferWrite(
                &mut numTaskGroups as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Save out the IDs of all the task groups
        let mut numWritten: c_int = 0;
        for tgi in self.m_taskGroups.iter() {
            let mut id: c_int = unsafe { (**tgi).GetGUID() };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
            numWritten += 1;
        }
        assert!(numWritten == numTaskGroups);

        //Save out the task groups
        numWritten = 0;
        for tgi in self.m_taskGroups.iter() {
            //Save out the parent
            let mut id: c_int = if unsafe { (**tgi).GetParent() }.is_null() {
                -1
            } else {
                unsafe { (*(**tgi).GetParent()).GetGUID() }
            };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Save out the number of commands
            let mut numCommands: c_int =
                unsafe { (**tgi).m_completedTasks.len() as c_int };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut numCommands as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Save out the command map
            for (tci_key, tci_val) in unsafe { (**tgi).m_completedTasks.iter() } {
                //Write out the ID
                let mut id: c_int = *tci_key;
                unsafe {
                    (*pIcarus).BufferWrite(
                        &mut id as *mut c_int as *mut c_void,
                        core::mem::size_of::<c_int>(),
                    );
                }

                //Write out the state of completion
                let mut completed: bool = *tci_val;
                unsafe {
                    (*pIcarus).BufferWrite(
                        &mut completed as *mut bool as *mut c_void,
                        core::mem::size_of::<bool>(),
                    );
                }
            }

            //Save out the number of completed commands
            let mut id: c_int = unsafe { (**tgi).m_numCompleted };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
            numWritten += 1;
        }
        assert!(numWritten == numTaskGroups);

        //Only bother if we've got tasks present
        if !self.m_taskGroups.is_empty() {
            //Save out the currently active group
            let mut curGroupID: c_int = if self.m_curGroup.is_null() {
                -1
            } else {
                unsafe { (*self.m_curGroup).GetGUID() }
            };
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut curGroupID as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
        }

        //Save out the task group name maps
        numWritten = 0;
        for (tmi_key, tmi_val) in self.m_taskGroupNameMap.iter() {
            let name_cstr = tmi_key.as_str();
            // Porting note: original checks name != NULL && name[0] != NULL (non-null/non-empty).
            // In Rust, String is never null, and we check non-empty.
            assert!(!name_cstr.is_empty());

            let name_bytes = name_cstr.as_bytes();
            let mut length: c_int = (name_bytes.len() + 1) as c_int; // + 1 for NUL

            //Save out the string size
            //icarus->GetGame()->WriteSaveData( 'TGNL', &length, sizeof ( length ) );
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut length as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Write out the string
            // Porting note: C writes raw bytes including NUL terminator via strlen(name)+1 above.
            // We write name_bytes + NUL via a temporary buffer.
            let mut name_buf: Vec<u8> = Vec::with_capacity(length as usize);
            name_buf.extend_from_slice(name_bytes);
            name_buf.push(0u8); // NUL terminator
            unsafe {
                (*pIcarus).BufferWrite(
                    name_buf.as_mut_ptr() as *mut c_void,
                    length as usize,
                );
            }

            let taskGroup: *mut CTaskGroup = *tmi_val;

            let mut id: c_int = unsafe { (*taskGroup).GetGUID() };

            //Write out the ID
            unsafe {
                (*pIcarus).BufferWrite(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
            numWritten += 1;
        }
        assert!(numWritten == numTaskGroups);
    }

    /*
    -------------------------
    Load
    -------------------------
    */

    pub fn Load(&mut self, icarus: *mut CIcarus) {
        let mut flags: u8 = 0;
        let taskGroup: *mut CTaskGroup;
        let block: *mut CBlock;
        let task: *mut CTask;
        let mut timeStamp: DWORD = 0;
        let mut completed: bool = false;
        let bData: *mut c_void;
        let mut id: c_int = 0;
        let mut numTasks: c_int = 0;
        let mut numMembers: c_int = 0;
        let mut bID: c_int = 0;
        let mut bSize: c_int = 0;

        // Data expected/loaded here.
        //	Taskmanager GUID.
        //	Number of Tasks.
        //	Tasks:
        //				- GUID.
        //				- Timestamp.
        //				- Block/Command.
        //	Number of task groups.
        //	Task groups ID's.
        //	Task groups (data).
        //				- Parent.
        //				- Number of Commands.
        //				- Commands:
        //						+ ID.
        //						+ State of Completion.
        //				- Number of Completed Commands.
        //	Currently active group.
        //	Task group names:
        //				- String Size.
        //				- String.
        //				- ID.

        let pIcarus: *mut CIcarus =
            unsafe { IIcarusInterface::GetIcarus() as *mut CIcarus };

        //Get the GUID
        unsafe {
            (*pIcarus).BufferRead(
                &mut self.m_GUID as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Get the number of tasks to follow
        unsafe {
            (*pIcarus).BufferRead(
                &mut numTasks as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Reload all the tasks
        for i in 0..numTasks {
            let task: *mut CTask = Box::into_raw(Box::new(CTask::new()));

            assert!(!task.is_null());

            //Get the GUID
            unsafe {
                (*pIcarus).BufferRead(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
                (*task).SetGUID(id);
            }

            //Get the time stamp
            unsafe {
                (*pIcarus).BufferRead(
                    &mut timeStamp as *mut DWORD as *mut c_void,
                    core::mem::size_of::<DWORD>(),
                );
                (*task).SetTimeStamp(timeStamp);
            }

            //
            // BLOCK LOADING
            //

            //Get the block ID and create a new container
            unsafe {
                (*pIcarus).BufferRead(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
            let block: *mut CBlock = Box::into_raw(Box::new(unsafe { core::mem::zeroed::<CBlock>() }));

            unsafe { (*block).Create(id); }

            //Read the block's flags
            unsafe {
                (*pIcarus).BufferRead(
                    &mut flags as *mut u8 as *mut c_void,
                    core::mem::size_of::<u8>(),
                );
                (*block).SetFlags(flags);
            }

            //Get the number of block members
            unsafe {
                (*pIcarus).BufferRead(
                    &mut numMembers as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            for j in 0..numMembers {
                //Get the member ID
                unsafe {
                    (*pIcarus).BufferRead(
                        &mut bID as *mut c_int as *mut c_void,
                        core::mem::size_of::<c_int>(),
                    );
                }

                //Get the member size
                unsafe {
                    (*pIcarus).BufferRead(
                        &mut bSize as *mut c_int as *mut c_void,
                        core::mem::size_of::<c_int>(),
                    );
                }

                //Get the member's data
                let bData: *mut c_void =
                    unsafe { (*(*icarus).GetGame()).Malloc(bSize as usize) };
                if bData.is_null() {
                    assert!(false);
                    return;
                }

                //Get the actual raw data
                unsafe {
                    (*pIcarus).BufferRead(bData, bSize as usize);
                }

                //Write out the correct type
                match bID {
                    id if id == CIcarus::TK_FLOAT => {
                        unsafe {
                            (*block).Write(
                                CIcarus::TK_FLOAT,
                                *(bData as *const f32),
                                icarus,
                            );
                        }
                    }

                    id if id == CIcarus::TK_IDENTIFIER => {
                        unsafe {
                            (*block).Write(CIcarus::TK_IDENTIFIER, bData as *mut c_char, icarus);
                        }
                    }

                    id if id == CIcarus::TK_STRING => {
                        unsafe {
                            (*block).Write(CIcarus::TK_STRING, bData as *mut c_char, icarus);
                        }
                    }

                    id if id == CIcarus::TK_VECTOR => {
                        unsafe {
                            (*block).Write(
                                CIcarus::TK_VECTOR,
                                *(bData as *mut vec3_t),
                                icarus,
                            );
                        }
                    }

                    id if id == CIcarus::ID_RANDOM => {
                        unsafe {
                            (*block).Write(
                                CIcarus::ID_RANDOM,
                                *(bData as *const f32),
                                icarus,
                            ); //ID_RANDOM
                        }
                    }

                    id if id == CIcarus::ID_TAG => {
                        unsafe {
                            (*block).Write(
                                CIcarus::ID_TAG,
                                CIcarus::ID_TAG as f32,
                                icarus,
                            );
                        }
                    }

                    id if id == CIcarus::ID_GET => {
                        unsafe {
                            (*block).Write(
                                CIcarus::ID_GET,
                                CIcarus::ID_GET as f32,
                                icarus,
                            );
                        }
                    }

                    _ => {
                        unsafe {
                            (*(*icarus).GetGame()).DebugPrint(
                                IGameInterface::WL_ERROR,
                                b"Invalid Block id %d\n\0".as_ptr() as *const c_char,
                                bID,
                            );
                        }
                        assert!(false);
                        // break (no fallthrough in Rust match; continue to Free below)
                    }
                }

                //Get rid of the temp memory
                unsafe { (*(*icarus).GetGame()).Free(bData); }
            }

            unsafe { (*task).SetBlock(block); }

            STL_INSERT!(self.m_tasks, task);
        }

        //Load the task groups
        let mut numTaskGroups: c_int = 0;

        //icarus->GetGame()->ReadSaveData( 'TG#G', &numTaskGroups, sizeof( numTaskGroups ) );
        unsafe {
            (*pIcarus).BufferRead(
                &mut numTaskGroups as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        if numTaskGroups == 0 {
            return;
        }

        let mut taskIDs: Vec<c_int> = vec![0; numTaskGroups as usize];

        //Get the task group IDs
        for i in 0..numTaskGroups {
            //Creat a new task group
            let taskGroup: *mut CTaskGroup = Box::into_raw(Box::new(CTaskGroup::new()));
            assert!(!taskGroup.is_null());

            //Get this task group's ID
            unsafe {
                (*pIcarus).BufferRead(
                    &mut taskIDs[i as usize] as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
                taskGroup.as_mut().unwrap().m_GUID = taskIDs[i as usize];
            }

            self.m_taskGroupIDMap.insert(taskIDs[i as usize], taskGroup);

            STL_INSERT!(self.m_taskGroups, taskGroup);
        }

        //Recreate and load the task groups
        for i in 0..numTaskGroups {
            let taskGroup: *mut CTaskGroup = self.GetTaskGroupByID(taskIDs[i as usize], icarus);
            assert!(!taskGroup.is_null());

            //Load the parent ID
            unsafe {
                (*pIcarus).BufferRead(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            if id != -1 {
                let parent = self.GetTaskGroupByID(id, icarus);
                let parent2 = if !parent.is_null() {
                    self.GetTaskGroupByID(id, icarus)
                } else {
                    core::ptr::null_mut()
                };
                unsafe { (*taskGroup).m_parent = parent2; }
            }

            //Get the number of commands in this group
            unsafe {
                (*pIcarus).BufferRead(
                    &mut numMembers as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Get each command and its completion state
            for j in 0..numMembers {
                //Get the ID
                unsafe {
                    (*pIcarus).BufferRead(
                        &mut id as *mut c_int as *mut c_void,
                        core::mem::size_of::<c_int>(),
                    );
                }

                //Write out the state of completion
                unsafe {
                    (*pIcarus).BufferRead(
                        &mut completed as *mut bool as *mut c_void,
                        core::mem::size_of::<bool>(),
                    );
                }

                //Save it out
                unsafe { (*taskGroup).m_completedTasks.insert(id, completed); }
            }

            //Get the number of completed tasks
            unsafe {
                (*pIcarus).BufferRead(
                    &mut (*taskGroup).m_numCompleted as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }
        }

        //Reload the currently active group
        let mut curGroupID: c_int = 0;

        unsafe {
            (*pIcarus).BufferRead(
                &mut curGroupID as *mut c_int as *mut c_void,
                core::mem::size_of::<c_int>(),
            );
        }

        //Reload the map entries
        for i in 0..numTaskGroups {
            let mut name: [c_char; 1024] = [0; 1024];
            let mut length: c_int = 0;

            //Get the size of the string
            unsafe {
                (*pIcarus).BufferRead(
                    &mut length as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            //Get the string
            unsafe {
                (*pIcarus).BufferRead(
                    &mut name as *mut [c_char; 1024] as *mut c_void,
                    length as usize,
                );
            }

            //Get the id
            unsafe {
                (*pIcarus).BufferRead(
                    &mut id as *mut c_int as *mut c_void,
                    core::mem::size_of::<c_int>(),
                );
            }

            let taskGroup: *mut CTaskGroup = self.GetTaskGroupByID(id, icarus);
            assert!(!taskGroup.is_null());

            let name_str = unsafe {
                core::ffi::CStr::from_ptr(name.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };
            self.m_taskGroupNameMap.insert(name_str, taskGroup);
            unsafe {
                self.m_taskGroupIDMap
                    .insert((*taskGroup).GetGUID(), taskGroup);
            }
        }

        self.m_curGroup = if curGroupID == -1 {
            core::ptr::null_mut()
        } else {
            *self.m_taskGroupIDMap.get(&curGroupID).unwrap_or(&core::ptr::null_mut())
        };

        // delete[] taskIDs -> dropped automatically when taskIDs Vec goes out of scope
    }
}
